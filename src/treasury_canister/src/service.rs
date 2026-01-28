use candid::Nat;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};

use crate::types::*;
use crate::state::*;
use crate::constants::*;

// ============================================================================
// ACCESS CONTROL
// ============================================================================

/// Check if the caller is the governance canister
pub fn is_governance_canister() -> bool {
    let caller = ic_cdk::caller();
    let governance_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    caller == governance_id
}

/// Ensure caller is governance canister or controller
pub fn require_governance_or_controller() -> Result<(), String> {
    if is_governance_canister() || ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("Unauthorized: Only governance canister or controller can execute transfers".to_string())
    }
}

// ============================================================================
// DATE/TIME FUNCTIONS
// ============================================================================

/// Check if a year is a leap year
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a month
pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30, // Default fallback
    }
}

/// Convert nanoseconds since Unix epoch to date/time components (UTC)
pub fn nanos_to_datetime(nanos: u64) -> DateTimeComponents {
    let total_seconds = nanos / NANOS_PER_SECOND;
    let seconds_in_day = total_seconds % 86400;
    let mut total_days = (nanos / NANOS_PER_DAY) as u64;
    
    let hour = (seconds_in_day / 3600) as u8;
    let minute = ((seconds_in_day % 3600) / 60) as u8;
    let second = (seconds_in_day % 60) as u8;
    
    // Calculate year, month, day from total days since epoch (Jan 1, 1970)
    let mut year: u16 = 1970;
    
    // Process 400-year cycles
    let cycles_400 = total_days / DAYS_PER_400_YEARS;
    total_days %= DAYS_PER_400_YEARS;
    year += (cycles_400 * 400) as u16;
    
    // Process 100-year cycles
    let mut cycles_100 = total_days / DAYS_PER_100_YEARS;
    if cycles_100 == 4 { cycles_100 = 3; } // Handle leap year at end of 400-year cycle
    total_days -= cycles_100 * DAYS_PER_100_YEARS;
    year += (cycles_100 * 100) as u16;
    
    // Process 4-year cycles
    let cycles_4 = total_days / DAYS_PER_4_YEARS;
    total_days %= DAYS_PER_4_YEARS;
    year += (cycles_4 * 4) as u16;
    
    // Process remaining years
    let mut remaining_years = total_days / DAYS_PER_YEAR;
    if remaining_years == 4 { remaining_years = 3; } // Handle leap year at end of 4-year cycle
    total_days -= remaining_years * DAYS_PER_YEAR;
    year += remaining_years as u16;
    
    // Now total_days is the day of year (0-indexed)
    let mut month: u8 = 1;
    while month <= 12 {
        let days_this_month = days_in_month(year, month) as u64;
        if total_days < days_this_month {
            break;
        }
        total_days -= days_this_month;
        month += 1;
    }
    
    let day = (total_days + 1) as u8; // 1-indexed
    
    DateTimeComponents { year, month, day, hour, minute, second }
}

/// Convert date/time components to nanoseconds since Unix epoch (UTC)
pub fn datetime_to_nanos(dt: &DateTimeComponents) -> u64 {
    // Days from year
    let year = dt.year;
    let mut days: u64 = 0;
    
    // Add days for years since 1970
    if year >= 1970 {
        let year_offset = (year - 1970) as u64;
        // Approximate, then adjust
        days = year_offset * 365;
        // Add leap years
        let leap_years = (year - 1) / 4 - 1969 / 4 - (year - 1) / 100 + 1969 / 100 + (year - 1) / 400 - 1969 / 400;
        days += leap_years as u64;
    }
    
    // Add days for months
    for m in 1..dt.month {
        days += days_in_month(dt.year, m) as u64;
    }
    
    // Add days (0-indexed from month start)
    days += (dt.day - 1) as u64;
    
    // Convert to nanoseconds and add time components
    days * NANOS_PER_DAY 
        + (dt.hour as u64) * NANOS_PER_HOUR 
        + (dt.minute as u64) * NANOS_PER_MINUTE 
        + (dt.second as u64) * NANOS_PER_SECOND
}

/// Get the day of week for a given date (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
pub fn day_of_week(year: u16, month: u8, day: u8) -> u8 {
    // Zeller's congruence (adjusted for Gregorian calendar)
    let mut y = year as i32;
    let mut m = month as i32;
    
    if m < 3 {
        m += 12;
        y -= 1;
    }
    
    let k = y % 100;
    let j = y / 100;
    
    let h = (day as i32 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    
    // Convert from Zeller result (0=Saturday) to standard (0=Sunday)
    ((h + 6) % 7) as u8
}

/// Find the Nth occurrence of a specific day of week in a month
/// Returns the day of month (1-31)
pub fn find_nth_weekday(year: u16, month: u8, weekday: u8, n: u8) -> u8 {
    // Find first occurrence of weekday in the month
    let first_day_dow = day_of_week(year, month, 1);
    let first_occurrence = if weekday >= first_day_dow {
        weekday - first_day_dow + 1
    } else {
        7 - first_day_dow + weekday + 1
    };
    
    // Find nth occurrence
    first_occurrence + (n - 1) * 7
}

/// Determine if a given UTC timestamp falls within US Daylight Saving Time
/// DST starts: 2nd Sunday of March at 2:00 AM local time (EST -> EDT)
/// DST ends: 1st Sunday of November at 2:00 AM local time (EDT -> EST)
pub fn is_daylight_saving_time(nanos_utc: u64) -> bool {
    let dt = nanos_to_datetime(nanos_utc);
    let year = dt.year;
    
    // DST start: 2nd Sunday of March at 2:00 AM EST (7:00 AM UTC)
    let dst_start_day = find_nth_weekday(year, 3, 0, 2); // 2nd Sunday
    let dst_start = DateTimeComponents { year, month: 3, day: dst_start_day, hour: 7, minute: 0, second: 0 };
    let dst_start_nanos = datetime_to_nanos(&dst_start);
    
    // DST end: 1st Sunday of November at 2:00 AM EDT (6:00 AM UTC)
    let dst_end_day = find_nth_weekday(year, 11, 0, 1); // 1st Sunday
    let dst_end = DateTimeComponents { year, month: 11, day: dst_end_day, hour: 6, minute: 0, second: 0 };
    let dst_end_nanos = datetime_to_nanos(&dst_end);
    
    nanos_utc >= dst_start_nanos && nanos_utc < dst_end_nanos
}

/// Get the UTC offset for Eastern Time in nanoseconds (negative value)
pub fn get_eastern_offset_nanos(nanos_utc: u64) -> i64 {
    if is_daylight_saving_time(nanos_utc) {
        EDT_OFFSET_NANOS
    } else {
        EST_OFFSET_NANOS
    }
}

/// Convert UTC nanoseconds to Eastern Time date components
pub fn utc_to_eastern(nanos_utc: u64) -> DateTimeComponents {
    let offset = get_eastern_offset_nanos(nanos_utc);
    let eastern_nanos = if offset < 0 {
        nanos_utc.saturating_sub((-offset) as u64)
    } else {
        nanos_utc + offset as u64
    };
    nanos_to_datetime(eastern_nanos)
}

/// Get the UTC timestamp for midnight (12:00 AM) Eastern Time on the 1st of a given month/year
pub fn get_first_of_month_midnight_et_utc(year: u16, month: u8) -> u64 {
    // First, calculate what the UTC time would be for midnight ET on the 1st
    // We need to check if that specific moment is in DST or not
    
    // Start with an approximation: midnight on the 1st in UTC
    let approx_dt = DateTimeComponents { year, month, day: 1, hour: 5, minute: 0, second: 0 };
    let approx_nanos = datetime_to_nanos(&approx_dt);
    
    // Check if that time is during DST
    let is_dst = is_daylight_saving_time(approx_nanos);
    
    // Calculate the correct UTC hour for midnight ET
    // If DST (EDT/UTC-4): midnight ET = 4:00 AM UTC
    // If no DST (EST/UTC-5): midnight ET = 5:00 AM UTC
    let utc_hour: u8 = if is_dst { 4 } else { 5 };
    
    let target_dt = DateTimeComponents { year, month, day: 1, hour: utc_hour, minute: 0, second: 0 };
    datetime_to_nanos(&target_dt)
}

/// Calculate seconds until the next 1st of month at midnight ET
pub fn seconds_until_next_first_of_month(current_nanos_utc: u64) -> u64 {
    let current_et = utc_to_eastern(current_nanos_utc);
    
    // Determine next month
    let (next_year, next_month) = if current_et.month == 12 {
        (current_et.year + 1, 1)
    } else {
        (current_et.year, current_et.month + 1)
    };
    
    // If we're already on the 1st and before midnight has passed, calculate until midnight
    // Otherwise, calculate until next month's 1st
    let target_nanos = if current_et.day == 1 && current_et.hour == 0 && current_et.minute == 0 {
        // We're exactly at midnight on the 1st - return 0
        return 0;
    } else if current_et.day == 1 && current_et.hour < 1 {
        // We're on the 1st in the first hour, the window might still be open
        // Return 0 to indicate MMCR can execute
        return 0;
    } else {
        get_first_of_month_midnight_et_utc(next_year, next_month)
    };
    
    if target_nanos > current_nanos_utc {
        (target_nanos - current_nanos_utc) / NANOS_PER_SECOND
    } else {
        0
    }
}

/// Check if the current time is within the MMCR execution window
pub fn is_in_mmcr_window(nanos_utc: u64) -> bool {
    let et = utc_to_eastern(nanos_utc);
    
    // Must be the 1st of the month
    if et.day != 1 {
        return false;
    }
    
    // Must be within the first hour (00:00 - 00:59 ET)
    // We use a 1-hour window to account for timer drift and ensure we don't miss the execution
    et.hour == 0
}

/// Check if MMCR should execute for a new month
pub fn should_execute_mmcr(current_nanos: u64, last_month: u8, last_year: u16) -> bool {
    // Check if we're in the execution window (1st of month, midnight hour ET)
    if !is_in_mmcr_window(current_nanos) {
        return false;
    }
    
    let current_et = utc_to_eastern(current_nanos);
    
    // Check if this is a new month (different from last execution)
    if last_month == 0 && last_year == 0 {
        // First MMCR ever - allow it
        return true;
    }
    
    // Only execute if we've moved to a new month
    current_et.month != last_month || current_et.year != last_year
}

// ============================================================================
// MMCR LOGIC
// ============================================================================

pub fn try_execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Check if all releases are completed
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err("All MMCR releases completed".to_string());
        }
        
        // Check minimum interval (safety check to prevent multiple executions)
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            return Err("Too early for next MMCR (minimum interval not reached)".to_string());
        }
        
        // Check if we should execute based on calendar schedule
        if !should_execute_mmcr(current_time, state.last_mmcr_month, state.last_mmcr_year) {
            let current_et = utc_to_eastern(current_time);
            return Err(format!(
                "Not in MMCR execution window. Current ET: {}/{}/{} {:02}:{:02}. MMCR executes on 1st of each month at 12:00 AM ET. Last executed: {}/{}",
                current_et.year, current_et.month, current_et.day, current_et.hour, current_et.minute,
                state.last_mmcr_year, state.last_mmcr_month
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

// ============================================================================
// EXECUTED TRANSFERS
// ============================================================================

/// Execute a transfer from treasury (called by governance canister after proposal approval)
pub async fn execute_transfer_impl(input: ExecuteTransferInput) -> Result<u64, String> {
    require_governance_or_controller()?;
    
    // Only execute GHC for now (USDC/ICP requires additional ledger setup)
    if input.token_type != TokenType::GHC {
        return Err("Only GHC transfers are supported currently".to_string());
    }
    
    // Check treasury allowance
    let current_allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
    if input.amount > current_allowance {
        return Err(format!(
            "Insufficient treasury allowance. Available: {} GHC, Requested: {} GHC",
            current_allowance / 100_000_000,
            input.amount / 100_000_000
        ));
    }
    
    // Execute transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: input.recipient, subaccount: None },
        amount: Nat::from(input.amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    
    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer failed: {:?} {}", code, msg))?;
    
    match result {
        Ok(block_index) => {
            // Update treasury state
            TREASURY_STATE.with(|s| {
                let mut cell = s.borrow_mut();
                let mut state = cell.get().clone();
                state.balance = state.balance.saturating_sub(input.amount);
                state.allowance = state.allowance.saturating_sub(input.amount);
                state.total_transferred += input.amount;
                cell.set(state).expect("Failed to update treasury state");
            });
            
            // Convert Nat to u64 for block index
            let block: u64 = block_index.0.try_into().unwrap_or(0);
            Ok(block)
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}
