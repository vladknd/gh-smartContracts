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
struct GlobalStats {
    total_staked: u64,
    interest_pool: u64,
    cumulative_reward_index: u128, // Scaled by 1e18
}

impl Storable for GlobalStats {
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
            }
        ).unwrap()
    );

    static USER_STATE: RefCell<StableBTreeMap<Principal, UserState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );
}

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID");
    });
}

// Helper: Calculate pending rewards for a user based on index growth
fn calculate_rewards(user_state: &UserState, current_index: u128) -> u64 {
    if current_index <= user_state.last_reward_index {
        return 0;
    }
    let index_diff = current_index - user_state.last_reward_index;
    // Rewards = Balance * (IndexDiff / 1e18)
    // We do (Balance * IndexDiff) / 1e18
    let rewards = (user_state.balance as u128 * index_diff) / 1_000_000_000_000_000_000;
    rewards as u64
}

// Helper: Update user state with pending rewards and new index
fn update_user_state(user: Principal) -> UserState {
    let current_index = GLOBAL_STATS.with(|s| s.borrow().get().cumulative_reward_index);
    
    USER_STATE.with(|map| {
        let mut m = map.borrow_mut();
        let mut state = m.get(&user).unwrap_or(UserState {
            balance: 0,
            last_reward_index: current_index,
        });

        let rewards = calculate_rewards(&state, current_index);
        if rewards > 0 {
            state.balance += rewards;
        }
        state.last_reward_index = current_index;
        
        // We return the updated state but don't save it yet, caller decides
        state
    })
}

#[update]
fn stake_rewards(user: Principal, amount: u64) {
    // In production, check caller is Learning Engine!
    
    let mut state = update_user_state(user);
    state.balance += amount;
    
    USER_STATE.with(|map| map.borrow_mut().insert(user, state));
    
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.total_staked += amount;
        cell.set(stats).expect("Failed to update global stats");
    });
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

#[update]
async fn unstake(amount: u64) -> Result<u64, String> {
    let user = ic_cdk::caller();
    
    // 1. Update user state (claim pending rewards first)
    let mut state = update_user_state(user);
    
    if state.balance < amount {
        return Err(format!("Insufficient balance. Available: {}", state.balance));
    }

    // 2. Calculate Split
    let penalty = amount / 10; // 10%
    let return_amount = amount - penalty;

    // 3. Update Internal State
    state.balance -= amount;
    USER_STATE.with(|map| map.borrow_mut().insert(user, state));

    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.total_staked -= amount;
        stats.interest_pool += penalty; // Add penalty to pool
        cell.set(stats).expect("Failed to update global stats");
    });

    // 4. Ledger Transfer (Real Settlement)
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    // Transfer from Hub Main (Subaccount None) to User Main (Subaccount None)
    // Note: In this model, we assume Hub holds all tokens in its main account.
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
            // Rollback state if transfer fails (Critical!)
            // Re-fetch state to be safe
            let mut state = update_user_state(user);
            state.balance += amount; // Give it back
            USER_STATE.with(|map| map.borrow_mut().insert(user, state));
            
            GLOBAL_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.total_staked += amount;
                stats.interest_pool -= penalty;
                cell.set(stats).expect("Failed to rollback global stats");
            });
            
            Err(format!("Ledger transfer failed: {:?}", e))
        }
    }
}

#[query]
fn get_user_stats(user: Principal) -> (u64, u64) {
    // Returns (Current Balance, Pending Rewards not yet credited)
    let current_index = GLOBAL_STATS.with(|s| s.borrow().get().cumulative_reward_index);
    
    USER_STATE.with(|map| {
        if let Some(state) = map.borrow().get(&user) {
            let pending = calculate_rewards(&state, current_index);
            (state.balance, pending)
        } else {
            (0, 0)
        }
    })
}

#[query]
fn get_global_stats() -> GlobalStats {
    GLOBAL_STATS.with(|s| s.borrow().get().clone())
}

#[query]
fn get_voting_power(user: Principal) -> u64 {
    let (balance, pending) = get_user_stats(user);
    balance + pending
}

ic_cdk::export_candid!();
