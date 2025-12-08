use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use std::borrow::Cow;
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use candid::{Nat, Encode, Decode};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize)]
struct InitArgs {
    ledger_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserState {
    balance: u64,
    last_reward_index: u128,
}

impl Storable for UserState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct GlobalStatsV2 {
    total_staked: u64,
    interest_pool: u64,
    cumulative_reward_index: u128,
    total_unstaked: u64,
    total_mined: u64,
    total_rewards_distributed: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct GlobalStatsV1_5 {
    total_staked: u64,
    interest_pool: u64,
    cumulative_reward_index: u128,
    total_unstaked: u64,
    total_mined: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct GlobalStats {
    total_staked: u64,
    interest_pool: u64,
    cumulative_reward_index: u128, // Scaled by 1e18
    total_unstaked: u64,
    total_allocated: u64, // Renamed from total_mined: Tracked against MAX_SUPPLY
    total_rewards_distributed: u64, // Actual rewards given to users
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct GlobalStatsV1 {
    total_staked: u64,
    interest_pool: u64,
    cumulative_reward_index: u128,
}

impl Storable for GlobalStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Try decoding as current version
        if let Ok(stats) = Decode!(bytes.as_ref(), Self) {
            return stats;
        }

        // Try decoding as V2 (migration)
        if let Ok(v2) = Decode!(bytes.as_ref(), GlobalStatsV2) {
            return Self {
                total_staked: v2.total_staked,
                interest_pool: v2.interest_pool,
                cumulative_reward_index: v2.cumulative_reward_index,
                total_unstaked: v2.total_unstaked,
                total_allocated: v2.total_mined, // Map mined -> allocated
                total_rewards_distributed: v2.total_rewards_distributed,
            };
        }

        // Try decoding as V1.5 (migration)
        if let Ok(v1_5) = Decode!(bytes.as_ref(), GlobalStatsV1_5) {
            return Self {
                total_staked: v1_5.total_staked,
                interest_pool: v1_5.interest_pool,
                cumulative_reward_index: v1_5.cumulative_reward_index,
                total_unstaked: v1_5.total_unstaked,
                total_allocated: v1_5.total_mined,
                total_rewards_distributed: 0,
            };
        }
        
        // Try decoding as V1 (migration)
        if let Ok(v1) = Decode!(bytes.as_ref(), GlobalStatsV1) {
            return Self {
                total_staked: v1.total_staked,
                interest_pool: v1.interest_pool,
                cumulative_reward_index: v1.cumulative_reward_index,
                total_unstaked: 0, // Initialize new field
                total_allocated: 0, // Initialize new field
                total_rewards_distributed: 0, // Initialize new field
            };
        }

        panic!("Failed to decode GlobalStats");
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    static GLOBAL_STATS: RefCell<StableCell<GlobalStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            GlobalStats {
                total_staked: 0,
                interest_pool: 0,
                cumulative_reward_index: 0,
                total_unstaked: 0,
                total_allocated: 0,
                total_rewards_distributed: 0,
            }
        ).unwrap()
    );

    static ALLOWED_MINTERS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

const MAX_SUPPLY: u64 = 4_200_000_000 * 100_000_000; // 4.2B Tokens

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID");
    });
}

#[update]
fn add_allowed_minter(principal: Principal) {
    // In production, add admin check here!
    ALLOWED_MINTERS.with(|m| m.borrow_mut().insert(principal, true));
}

#[update]
fn remove_allowed_minter(principal: Principal) {
    // In production, add admin check here!
    ALLOWED_MINTERS.with(|m| m.borrow_mut().remove(&principal));
}

// Replaces report_stats: Handles both stats reporting and allowance requests
#[update]
fn sync_shard(staked_delta: i64, unstaked_delta: u64, distributed_delta: u64, requested_allowance: u64) -> Result<(u64, u128), String> {
    let caller = ic_cdk::caller();
    
    // Check if caller is an allowed minter (shard)
    let is_allowed = ALLOWED_MINTERS.with(|m| m.borrow().contains_key(&caller));
    if !is_allowed {
        return Err("Unauthorized: Caller is not an allowed shard".to_string());
    }
    
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        // 1. Update Stats (Batch Reporting)
        if staked_delta > 0 {
            stats.total_staked += staked_delta as u64;
        } else {
            let abs_delta = staked_delta.abs() as u64;
            if stats.total_staked >= abs_delta {
                stats.total_staked -= abs_delta;
            } else {
                stats.total_staked = 0;
            }
        }
        stats.total_unstaked += unstaked_delta;
        stats.total_rewards_distributed += distributed_delta;

        // 2. Handle Allowance Request (Hard Cap Check)
        let granted_allowance = if requested_allowance > 0 {
            let remaining = MAX_SUPPLY.saturating_sub(stats.total_allocated);
            let to_grant = if remaining >= requested_allowance {
                requested_allowance
            } else {
                remaining // Give whatever is left
            };
            
            stats.total_allocated += to_grant;
            to_grant
        } else {
            0
        };
        
        let current_index = stats.cumulative_reward_index;
        
        cell.set(stats).expect("Failed to update global stats");
        Ok((granted_allowance, current_index))
    })
}

#[update]
fn distribute_interest() -> Result<String, String> {
    // This moves funds from Interest Pool to the Reward Index
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        if stats.interest_pool == 0 {
            return Err("No interest to distribute".to_string());
        }
        
        if stats.total_staked == 0 {
            return Err("No stakers to distribute to".to_string());
        }

        // Calculate Index Increase: (Pool * 1e18) / TotalStaked
        let increase = (stats.interest_pool as u128 * 1_000_000_000_000_000_000) / stats.total_staked as u128;
        
        stats.cumulative_reward_index += increase;
        let distributed = stats.interest_pool;
        stats.interest_pool = 0;
        
        cell.set(stats).expect("Failed to update global stats");
        Ok(format!("Distributed {} tokens. Index increased by {}", distributed, increase))
    })
}

// Renamed from unstake: Now called by Shards, not Users directly
#[update]
async fn process_unstake(user: Principal, amount: u64) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // 1. Verify Caller is a valid Shard
    let is_allowed = ALLOWED_MINTERS.with(|m| m.borrow().contains_key(&caller));
    if !is_allowed {
        return Err("Unauthorized: Caller is not an allowed shard".to_string());
    }

    // 2. Calculate Split (Penalty Logic is now here, centralized)
    let penalty = amount / 10; // 10%
    let return_amount = amount - penalty;

    // 3. Update Global Stats
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        stats.interest_pool += penalty; // Add penalty to pool
        stats.total_unstaked += return_amount;
        cell.set(stats).expect("Failed to update global stats");
    });

    // 4. Ledger Transfer (Real Settlement)
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let user_account = Account {
        owner: user,
        subaccount: None,
    };

    let args = TransferArg {
        from_subaccount: None,
        to: user_account,
        amount: Nat::from(return_amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Rejection code: {:?}, message: {}", code, msg))?;

    match result {
        Ok(_) => Ok(return_amount),
        Err(e) => {
            // If transfer fails, we must tell Shard to rollback?
            // This is complex. For now, return Err and let Shard handle rollback.
            // We should rollback our local stats change (penalty).
             GLOBAL_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.interest_pool -= penalty;
                stats.total_unstaked -= return_amount;
                cell.set(stats).expect("Failed to rollback global stats");
            });
            Err(format!("Ledger transfer failed: {:?}", e))
        }
    }
}

// Removed get_user_stats and get_voting_power
// These must now be queried from the User Profile Shards directly.

#[query]
fn get_global_stats() -> GlobalStats {
    GLOBAL_STATS.with(|s| s.borrow().get().clone())
}

ic_cdk::export_candid!();
