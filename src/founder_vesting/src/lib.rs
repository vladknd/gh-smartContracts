//! # Founder Vesting Canister
//! 
//! This canister manages time-locked founder token allocations.
//! 
//! ## Allocation
//! - Founder 1: 0.35B MC (350,000,000 tokens)
//! - Founder 2: 0.15B MC (150,000,000 tokens)
//! - Total: 0.5B MC (500,000,000 tokens)
//! 
//! ## Vesting Schedule
//! - 10% unlocks per year
//! - Full vesting over 10 years
//! - Founders call `claim_vested()` to withdraw unlocked tokens
//! 
//! ## Key Functions
//! - `claim_vested()`: Founders claim their unlocked tokens
//! - `get_vesting_status(founder)`: Query vesting progress
//! - `get_all_vesting_schedules()`: Query all founders' status

use ic_cdk::{init, query, update};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use std::borrow::Cow;
use candid::{Encode, Decode, Nat};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// CONSTANTS
// ============================================================================

/// One year in nanoseconds (365 days)
const YEAR_IN_NANOS: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;

/// Annual unlock rate: 10% = 1000 basis points
const ANNUAL_UNLOCK_BPS: u64 = 1000;

/// Maximum unlock: 100% = 10000 basis points (after 10 years)
const MAX_UNLOCK_BPS: u64 = 10000;

/// Founder 1 allocation: 0.35B MC (in e8s)
const FOUNDER_1_ALLOCATION: u64 = 350_000_000 * 100_000_000; // 3.5 * 10^16

/// Founder 2 allocation: 0.15B MC (in e8s)
const FOUNDER_2_ALLOCATION: u64 = 150_000_000 * 100_000_000; // 1.5 * 10^16

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Vesting schedule for a founder
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VestingSchedule {
    /// Founder principal ID
    pub founder: Principal,
    
    /// Total tokens allocated (never changes)
    pub total_allocation: u64,
    
    /// Tokens already claimed
    pub claimed: u64,
    
    /// Vesting start timestamp (set at canister init)
    pub vesting_start: u64,
}

impl VestingSchedule {
    /// Calculate currently vested (unlocked) tokens based on elapsed time
    pub fn vested_amount(&self, current_time: u64) -> u64 {
        if current_time <= self.vesting_start {
            return 0;
        }
        
        let elapsed_nanos = current_time - self.vesting_start;
        let elapsed_years = elapsed_nanos / YEAR_IN_NANOS;
        
        // 10% per year, max 100% after 10 years
        let unlock_bps = (elapsed_years * ANNUAL_UNLOCK_BPS).min(MAX_UNLOCK_BPS);
        
        // Calculate vested amount: (total * unlock_bps) / 10000
        ((self.total_allocation as u128 * unlock_bps as u128) / 10000) as u64
    }
    
    /// Calculate claimable tokens (vested - already claimed)
    pub fn claimable(&self, current_time: u64) -> u64 {
        self.vested_amount(current_time).saturating_sub(self.claimed)
    }
}

impl Storable for VestingSchedule {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Public vesting status for queries
#[derive(CandidType, Clone, Debug)]
pub struct VestingStatus {
    pub founder: Principal,
    pub total_allocation: u64,
    pub vested: u64,
    pub claimed: u64,
    pub claimable: u64,
    pub years_elapsed: u64,
    pub unlock_percentage: u64,
}

/// Initialization arguments
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    /// GHC Ledger canister ID
    pub ledger_id: Principal,
    /// Founder 1 principal
    pub founder1: Principal,
    /// Founder 2 principal
    pub founder2: Principal,
}

// ============================================================================
// STORAGE
// ============================================================================

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// GHC Ledger canister ID
    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Vesting schedules: Principal -> VestingSchedule
    static VESTING_SCHEDULES: RefCell<StableBTreeMap<Principal, VestingSchedule, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    /// Genesis timestamp (when vesting started)
    static GENESIS_TIMESTAMP: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            0
        ).unwrap()
    );
}

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    let now = ic_cdk::api::time();
    
    // Set ledger ID
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID")
    });
    
    // Set genesis timestamp
    GENESIS_TIMESTAMP.with(|ts| {
        ts.borrow_mut().set(now).expect("Failed to set genesis timestamp")
    });
    
    // Initialize Founder 1 vesting schedule
    let founder1_schedule = VestingSchedule {
        founder: args.founder1,
        total_allocation: FOUNDER_1_ALLOCATION,
        claimed: 0,
        vesting_start: now,
    };
    
    // Initialize Founder 2 vesting schedule
    let founder2_schedule = VestingSchedule {
        founder: args.founder2,
        total_allocation: FOUNDER_2_ALLOCATION,
        claimed: 0,
        vesting_start: now,
    };
    
    // Store vesting schedules
    VESTING_SCHEDULES.with(|v| {
        let mut schedules = v.borrow_mut();
        schedules.insert(args.founder1, founder1_schedule);
        schedules.insert(args.founder2, founder2_schedule);
    });
    
    ic_cdk::println!(
        "Founder Vesting initialized. F1: {} ({} e8s), F2: {} ({} e8s). Genesis: {}",
        args.founder1, FOUNDER_1_ALLOCATION,
        args.founder2, FOUNDER_2_ALLOCATION,
        now
    );
}

// ============================================================================
// UPDATE FUNCTIONS
// ============================================================================

/// Claim vested tokens
/// 
/// Called by founders to withdraw their unlocked tokens.
/// Transfers tokens from this canister to the caller's wallet.
#[update]
async fn claim_vested() -> Result<u64, String> {
    let caller = ic_cdk::caller();
    let current_time = ic_cdk::api::time();
    
    // Find founder's vesting schedule
    let schedule = VESTING_SCHEDULES.with(|v| v.borrow().get(&caller))
        .ok_or("Caller is not a registered founder")?;
    
    // Calculate claimable amount
    let claimable = schedule.claimable(current_time);
    
    if claimable == 0 {
        let vested = schedule.vested_amount(current_time);
        return Err(format!(
            "No tokens available to claim. Vested: {} e8s, Already claimed: {} e8s",
            vested, schedule.claimed
        ));
    }
    
    // Execute ICRC-1 transfer to founder
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None, // From this canister's main account
        to: Account { owner: caller, subaccount: None },
        amount: Nat::from(claimable),
        fee: None, // Transfer fee is 0
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer call failed: {:?} {}", code, msg))?;

    match result {
        Ok(_block_index) => {
            // Update claimed amount
            VESTING_SCHEDULES.with(|v| {
                let mut schedules = v.borrow_mut();
                if let Some(mut sched) = schedules.get(&caller) {
                    sched.claimed += claimable;
                    schedules.insert(caller, sched);
                }
            });
            
            ic_cdk::println!(
                "Founder {} claimed {} e8s. Total claimed: {} e8s",
                caller, claimable, schedule.claimed + claimable
            );
            
            Ok(claimable)
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}

// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

/// Query vesting status for a specific founder
#[query]
fn get_vesting_status(founder: Principal) -> Option<VestingStatus> {
    let current_time = ic_cdk::api::time();
    
    VESTING_SCHEDULES.with(|v| {
        v.borrow().get(&founder).map(|schedule| {
            let years_elapsed = if current_time > schedule.vesting_start {
                (current_time - schedule.vesting_start) / YEAR_IN_NANOS
            } else {
                0
            };
            
            VestingStatus {
                founder: schedule.founder,
                total_allocation: schedule.total_allocation,
                vested: schedule.vested_amount(current_time),
                claimed: schedule.claimed,
                claimable: schedule.claimable(current_time),
                years_elapsed,
                unlock_percentage: (years_elapsed * 10).min(100),
            }
        })
    })
}

/// Query all vesting schedules (for admin/dashboard)
#[query]
fn get_all_vesting_schedules() -> Vec<VestingStatus> {
    let current_time = ic_cdk::api::time();
    
    VESTING_SCHEDULES.with(|v| {
        v.borrow().iter().map(|(_, schedule)| {
            let years_elapsed = if current_time > schedule.vesting_start {
                (current_time - schedule.vesting_start) / YEAR_IN_NANOS
            } else {
                0
            };
            
            VestingStatus {
                founder: schedule.founder,
                total_allocation: schedule.total_allocation,
                vested: schedule.vested_amount(current_time),
                claimed: schedule.claimed,
                claimable: schedule.claimable(current_time),
                years_elapsed,
                unlock_percentage: (years_elapsed * 10).min(100),
            }
        }).collect()
    })
}

/// Get genesis timestamp (when vesting started)
#[query]
fn get_genesis_timestamp() -> u64 {
    GENESIS_TIMESTAMP.with(|ts| *ts.borrow().get())
}

/// Get total tokens held by this canister (sum of all unclaimed allocations)
#[query]
fn get_total_unclaimed() -> u64 {
    VESTING_SCHEDULES.with(|v| {
        v.borrow().iter().map(|(_, schedule)| {
            schedule.total_allocation - schedule.claimed
        }).sum()
    })
}

/// Check if a principal is a registered founder
#[query]
fn is_founder(principal: Principal) -> bool {
    VESTING_SCHEDULES.with(|v| v.borrow().contains_key(&principal))
}

ic_cdk::export_candid!();
