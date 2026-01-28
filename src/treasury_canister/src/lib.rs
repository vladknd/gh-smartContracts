use ic_cdk::{init, query, update, post_upgrade};
use candid::Principal;

mod types;
mod state;
mod constants;
mod service;

use types::*;
use state::*;
use constants::*;
use service::*;


#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set ledger ID");
    });
    GOVERNANCE_CANISTER_ID.with(|id| {
        id.borrow_mut().set(args.governance_canister_id).expect("Failed to set governance canister ID");
    });
}

#[post_upgrade]
fn post_upgrade(args: Option<InitArgs>) {
    if let Some(args) = args {
        LEDGER_ID.with(|id| {
            let _ = id.borrow_mut().set(args.ledger_id);
        });
        GOVERNANCE_CANISTER_ID.with(|id| {
            let _ = id.borrow_mut().set(args.governance_canister_id);
        });
    }
}


#[query]
fn get_treasury_state() -> TreasuryState {
    TREASURY_STATE.with(|s| s.borrow().get().clone())
}

#[query]
fn get_spendable_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().allowance)
}

#[query]
fn get_treasury_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().balance)
}

/// Check if a transfer amount is within allowance
#[query]
fn can_transfer(amount: u64, token_type: TokenType) -> bool {
    if token_type == TokenType::GHC {
        let allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
        amount <= allowance
    } else {
        // For other tokens, we don't have allowance tracking yet
        // Return true for now, but actual transfer might fail
        true
    }
}

/// Execute a transfer from treasury (called by governance canister after proposal approval)
// ============================================================================
// TREASURY FUNCTIONS
// ============================================================================

#[update]
async fn execute_transfer(input: ExecuteTransferInput) -> Result<u64, String> {
    execute_transfer_impl(input).await
}

// ============================================================================
// MMCR (Monthly Minimum Capital Release)
// ============================================================================

#[update]
fn execute_mmcr() -> Result<u64, String> {
    try_execute_mmcr()
}

#[query]
fn get_mmcr_status() -> MMCRStatus {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let state = s.borrow().get().clone();
        let releases_remaining = TOTAL_MMCR_RELEASES.saturating_sub(state.mmcr_count);
        
        let next_release_amount = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        // Calculate next scheduled month
        let current_et = utc_to_eastern(current_time);
        let (next_year, next_month) = if state.last_mmcr_month == 0 || state.last_mmcr_year == 0 {
            // First MMCR - will be this month if we're on the 1st, otherwise next month
            if current_et.day == 1 && current_et.hour == 0 {
                (current_et.year, current_et.month)
            } else {
                if current_et.month == 12 {
                    (current_et.year + 1, 1)
                } else {
                    (current_et.year, current_et.month + 1)
                }
            }
        } else if state.last_mmcr_month == current_et.month && state.last_mmcr_year == current_et.year {
            // Already executed this month - next is next month
            if current_et.month == 12 {
                (current_et.year + 1, 1)
            } else {
                (current_et.year, current_et.month + 1)
            }
        } else {
            // Haven't executed this month yet - could be this month if we're on the 1st
            if current_et.day == 1 && current_et.hour == 0 {
                (current_et.year, current_et.month)
            } else {
                if current_et.month == 12 {
                    (current_et.year + 1, 1)
                } else {
                    (current_et.year, current_et.month + 1)
                }
            }
        };
        
        let seconds_until_next = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else {
            seconds_until_next_first_of_month(current_time)
        };
        
        MMCRStatus {
            releases_completed: state.mmcr_count,
            releases_remaining,
            last_release_timestamp: state.last_mmcr_timestamp,
            next_release_amount,
            seconds_until_next,
            next_scheduled_month: next_month,
            next_scheduled_year: next_year,
        }
    })
}

// ============================================================================
// TIME UTILITY QUERIES
// ============================================================================

/// Get the current time in Eastern Time zone (for debugging/display)
#[query]
fn get_current_eastern_time() -> (u16, u8, u8, u8, u8, u8, bool) {
    let current_time = ic_cdk::api::time();
    let et = utc_to_eastern(current_time);
    let is_dst = is_daylight_saving_time(current_time);
    (et.year, et.month, et.day, et.hour, et.minute, et.second, is_dst)
}

/// Check if MMCR can be executed right now
#[query]
fn can_execute_mmcr_now() -> (bool, String) {
    let current_time = ic_cdk::api::time();
    
    let state = TREASURY_STATE.with(|s| s.borrow().get().clone());
    
    if state.mmcr_count >= TOTAL_MMCR_RELEASES {
        return (false, "All MMCR releases completed".to_string());
    }
    
    if state.last_mmcr_timestamp > 0 && 
       current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
        return (false, "Minimum interval not reached".to_string());
    }
    
    let can_execute = should_execute_mmcr(current_time, state.last_mmcr_month, state.last_mmcr_year);
    let current_et = utc_to_eastern(current_time);
    
    if can_execute {
        (true, format!(
            "MMCR can execute now. Current ET: {}/{}/{} {:02}:{:02}",
            current_et.year, current_et.month, current_et.day, current_et.hour, current_et.minute
        ))
    } else {
        (false, format!(
            "Not in execution window. Current ET: {}/{}/{} {:02}:{:02}. MMCR executes on 1st at 12:00 AM ET.",
            current_et.year, current_et.month, current_et.day, current_et.hour, current_et.minute
        ))
    }
}

// ============================================================================
// CONFIGURATION QUERIES
// ============================================================================

#[query]
fn get_governance_canister_id() -> Principal {
    GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get())
}

#[query]
fn get_ledger_id() -> Principal {
    LEDGER_ID.with(|id| *id.borrow().get())
}

// ============================================================================
// ADMIN FUNCTIONS
// ============================================================================

/// Update the governance canister ID (controller only)
#[update]
fn set_governance_canister_id(new_id: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can update governance canister ID".to_string());
    }
    
    GOVERNANCE_CANISTER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set governance canister ID")
    });
    
    Ok(())
}

// ============================================================================
// TESTING FUNCTIONS (Controller Only)
// ============================================================================

/// Force execute MMCR bypassing calendar check (CONTROLLER ONLY)
/// This is for TESTING PURPOSES ONLY - bypasses the 1st-of-month requirement
/// Still enforces: minimum interval (25 days) and max releases (240)
#[update]
fn force_execute_mmcr() -> Result<u64, String> {
    // Only controllers can force execute
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can force execute MMCR".to_string());
    }
    
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Check if all releases are completed
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err("All MMCR releases completed".to_string());
        }
        
        // NOTE: We skip the calendar check but keep the minimum interval for safety
        // This prevents accidental multiple executions in testing
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            return Err(format!(
                "Too early for next MMCR (minimum interval: 25 days). Last: {}, Now: {}, Need to wait: {} seconds",
                state.last_mmcr_timestamp,
                current_time,
                (state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS - current_time) / NANOS_PER_SECOND
            ));
        }
        
        // Calculate release amount
        let release_amount = if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        // Update allowance (capped at balance)
        let new_allowance = (state.allowance + release_amount).min(state.balance);
        let actual_release = new_allowance - state.allowance;
        
        // Get current month/year in Eastern Time for tracking
        let current_et = utc_to_eastern(current_time);
        
        // Update state
        state.allowance = new_allowance;
        state.mmcr_count += 1;
        state.last_mmcr_timestamp = current_time;
        state.last_mmcr_month = current_et.month;
        state.last_mmcr_year = current_et.year;
        
        cell.set(state).expect("Failed to update treasury state");
        
        Ok(actual_release)
    })
}

/// Simulate MMCR execution at a specific UTC timestamp (QUERY - no state change)
/// Useful for testing: "What would happen if we tried to execute MMCR at time X?"
/// Input: UTC timestamp in nanoseconds
#[query]
fn simulate_mmcr_at_time(utc_timestamp_nanos: u64) -> (bool, String, u8, u16) {
    let state = TREASURY_STATE.with(|s| s.borrow().get().clone());
    
    if state.mmcr_count >= TOTAL_MMCR_RELEASES {
        return (false, "All MMCR releases completed".to_string(), 0, 0);
    }
    
    // Check minimum interval
    if state.last_mmcr_timestamp > 0 && 
       utc_timestamp_nanos < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
        return (false, "Minimum interval not reached".to_string(), 0, 0);
    }
    
    let et = utc_to_eastern(utc_timestamp_nanos);
    let can_execute = should_execute_mmcr(utc_timestamp_nanos, state.last_mmcr_month, state.last_mmcr_year);
    
    let message = if can_execute {
        format!(
            "MMCR would execute. Time ET: {}/{}/{} {:02}:{:02}",
            et.year, et.month, et.day, et.hour, et.minute
        )
    } else {
        format!(
            "MMCR would NOT execute. Time ET: {}/{}/{} {:02}:{:02}. Must be 1st of month at 00:xx ET.",
            et.year, et.month, et.day, et.hour, et.minute
        )
    };
    
    (can_execute, message, et.month, et.year)
}

/// Test date parsing: convert a UTC timestamp to date components
/// Returns: (year, month, day, hour, minute, second, is_dst, eastern_hour)
#[query]
fn test_date_parsing(utc_timestamp_nanos: u64) -> (u16, u8, u8, u8, u8, u8, bool, u8) {
    let utc = nanos_to_datetime(utc_timestamp_nanos);
    let is_dst = is_daylight_saving_time(utc_timestamp_nanos);
    let et = utc_to_eastern(utc_timestamp_nanos);
    
    (utc.year, utc.month, utc.day, utc.hour, utc.minute, utc.second, is_dst, et.hour)
}

/// Test DST detection for a specific year
/// Returns DST start and end dates for that year
/// Format: (start_month, start_day, end_month, end_day)
#[query]
fn test_dst_boundaries(year: u16) -> (u8, u8, u8, u8) {
    // DST starts: 2nd Sunday of March
    let dst_start_day = find_nth_weekday(year, 3, 0, 2);
    // DST ends: 1st Sunday of November
    let dst_end_day = find_nth_weekday(year, 11, 0, 1);
    
    (3, dst_start_day, 11, dst_end_day)
}

/// Test MMCR window detection for a specific date/time in Eastern Time
/// Input: year, month, day, hour, minute (all in ET)
/// Returns: (is_in_window, utc_timestamp, message)
#[query]
fn test_mmcr_window(year: u16, month: u8, day: u8, hour: u8, minute: u8) -> (bool, u64, String) {
    // Construct the UTC timestamp for this Eastern Time
    // First approximation
    let approx_dt = DateTimeComponents { year, month, day, hour: hour + 5, minute, second: 0 };
    let approx_nanos = datetime_to_nanos(&approx_dt);
    
    // Check if DST applies
    let is_dst = is_daylight_saving_time(approx_nanos);
    
    // Adjust for actual offset
    let utc_hour = if is_dst { hour + 4 } else { hour + 5 };
    let target_dt = DateTimeComponents { year, month, day, hour: utc_hour, minute, second: 0 };
    let utc_nanos = datetime_to_nanos(&target_dt);
    
    let in_window = is_in_mmcr_window(utc_nanos);
    
    let message = format!(
        "ET: {}/{}/{} {:02}:{:02} -> UTC: {:02}:{:02} (DST: {}). In MMCR window: {}",
        year, month, day, hour, minute, utc_hour, minute, is_dst, in_window
    );
    
    (in_window, utc_nanos, message)
}

/// Get test timestamps for the 1st of specific months (for testing)
/// Returns UTC timestamps for midnight ET on the 1st of each month for the given year
#[query]
fn get_test_timestamps_for_year(year: u16) -> Vec<(u8, u64, bool)> {
    (1..=12).map(|month| {
        let utc_nanos = get_first_of_month_midnight_et_utc(year, month);
        let is_dst = is_daylight_saving_time(utc_nanos);
        (month, utc_nanos, is_dst)
    }).collect()
}

/// Reset MMCR state for testing (CONTROLLER ONLY)
/// WARNING: This resets the MMCR count and timestamp - use with caution!
#[update]
fn reset_mmcr_for_testing() -> Result<String, String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can reset MMCR state".to_string());
    }
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        let old_count = state.mmcr_count;
        let old_timestamp = state.last_mmcr_timestamp;
        
        // Reset MMCR tracking (but preserve allowance/balance/total_transferred)
        state.mmcr_count = 0;
        state.last_mmcr_timestamp = 0;
        state.last_mmcr_month = 0;
        state.last_mmcr_year = 0;
        
        cell.set(state).expect("Failed to reset treasury state");
        
        Ok(format!(
            "MMCR state reset. Previous: count={}, last_timestamp={}",
            old_count, old_timestamp
        ))
    })
}

ic_cdk::export_candid!();
