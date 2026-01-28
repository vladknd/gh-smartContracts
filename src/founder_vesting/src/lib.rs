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

mod constants;
mod types;
mod state;
mod service;

use ic_cdk::{init, query, update};
use candid::Principal;
use crate::constants::*;
use crate::types::*;
use crate::state::*;

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
    
    ic_cdk::println!("Founder Vesting initialized with ledger: {}. Genesis: {}", args.ledger_id, now);
}

// ============================================================================
// UPDATE FUNCTIONS
// ============================================================================

/// Claim vested tokens
#[update]
async fn claim_vested() -> Result<u64, String> {
    service::claim_vested_tokens(ic_cdk::caller(), ic_cdk::api::time()).await
}

/// Register a new founder with a specific allocation
#[update]
fn admin_register_founder(founder: Principal, allocation_e8s: u64) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can register founders".to_string());
    }
    
    let genesis = GENESIS_TIMESTAMP.with(|ts| *ts.borrow().get());
    
    let schedule = VestingSchedule {
        founder,
        total_allocation: allocation_e8s,
        claimed: 0,
        vesting_start: genesis,
    };
    
    VESTING_SCHEDULES.with(|v| {
        v.borrow_mut().insert(founder, schedule);
    });
    
    ic_cdk::println!(
        "Founder {} registered with allocation {} e8s. Vesting starts at {}",
        founder, allocation_e8s, genesis
    );
    
    Ok(())
}

/// Admin function to claim vested tokens at a specific timestamp (FOR TESTING)
#[update]
async fn admin_claim_vested_at(timestamp: u64) -> Result<u64, String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    service::claim_vested_tokens(ic_cdk::caller(), timestamp).await
}

/// Admin function to set genesis timestamp (FOR TESTING)
#[update]
fn admin_set_genesis_timestamp(timestamp: u64) {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        ic_cdk::trap("Unauthorized");
    }
    GENESIS_TIMESTAMP.with(|ts| {
        ts.borrow_mut().set(timestamp).expect("Failed to set genesis timestamp");
    });
    
    // Also update all schedules to start at this genesis
    VESTING_SCHEDULES.with(|v| {
        let mut map = v.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k).collect();
        for k in keys {
            if let Some(mut sched) = map.get(&k) {
                sched.vesting_start = timestamp;
                map.insert(k, sched);
            }
        }
    });
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
