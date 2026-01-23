use ic_cdk::{init, query, update, post_upgrade};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use candid::Nat;
use std::borrow::Cow;
use candid::{Encode, Decode};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// TREASURY CONSTANTS (in e8s - 8 decimals)
// ============================================================================

const INITIAL_TREASURY_BALANCE: u64 = 425_000_000_000_000_000; // 4.25B * 10^8
const INITIAL_TREASURY_ALLOWANCE: u64 = 60_000_000_000_000_000; // 0.6B * 10^8
const MMCR_AMOUNT: u64 = 1_520_000_000_000_000; // 15.2M * 10^8
const FINAL_MMCR_AMOUNT: u64 = 1_720_000_000_000_000; // 17.2M * 10^8
const TOTAL_MMCR_RELEASES: u64 = 240;

// Minimum interval between MMCR releases (25 days - safety margin for calendar-based scheduling)
const MMCR_MIN_INTERVAL_NANOS: u64 = 25 * 24 * 60 * 60 * 1_000_000_000;

// ============================================================================
// TIME CONSTANTS
// ============================================================================

// Nanoseconds per unit
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MINUTE: u64 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: u64 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: u64 = 24 * NANOS_PER_HOUR;

// UTC offset in nanoseconds
const EST_OFFSET_NANOS: i64 = -5 * 60 * 60 * NANOS_PER_SECOND as i64; // UTC-5:00
const EDT_OFFSET_NANOS: i64 = -4 * 60 * 60 * NANOS_PER_SECOND as i64; // UTC-4:00

// Epoch reference: January 1, 1970 00:00:00 UTC (Unix epoch)
const DAYS_PER_400_YEARS: u64 = 146097;
const DAYS_PER_100_YEARS: u64 = 36524;
const DAYS_PER_4_YEARS: u64 = 1461;
const DAYS_PER_YEAR: u64 = 365;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Token types for treasury spending
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    GHC,
    USDC,
    ICP,
}

/// Treasury state - tracks balance and allowance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    pub balance: u64,
    pub allowance: u64,
    pub total_transferred: u64,
    pub mmcr_count: u64,
    pub last_mmcr_timestamp: u64,
    pub genesis_timestamp: u64,
    /// Month of the last MMCR execution (1-12), used for calendar-based scheduling
    pub last_mmcr_month: u8,
    /// Year of the last MMCR execution, used for calendar-based scheduling  
    pub last_mmcr_year: u16,
}

impl Storable for TreasuryState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 250, is_fixed_size: false };
}

#[derive(CandidType, Clone, Debug)]
pub struct MMCRStatus {
    pub releases_completed: u64,
    pub releases_remaining: u64,
    pub last_release_timestamp: u64,
    pub next_release_amount: u64,
    pub seconds_until_next: u64,
    /// The month of the next scheduled MMCR (1-12)
    pub next_scheduled_month: u8,
    /// The year of the next scheduled MMCR
    pub next_scheduled_year: u16,
}

/// Input for executing a treasury transfer
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ExecuteTransferInput {
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub proposal_id: u64,
}

#[derive(CandidType, Deserialize)]
struct InitArgs {
    ledger_id: Principal,
    governance_canister_id: Principal,
}

/// Date/time components for internal use
struct DateTimeComponents {
    year: u16,
    month: u8,   // 1-12
    day: u8,     // 1-31
    hour: u8,    // 0-23
    minute: u8,  // 0-59
    second: u8,  // 0-59
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // Configuration
    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), Principal::anonymous()).unwrap()
    );
    
    static GOVERNANCE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), Principal::anonymous()).unwrap()
    );
    
    // Treasury
    static TREASURY_STATE: RefCell<StableCell<TreasuryState, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            TreasuryState {
                balance: INITIAL_TREASURY_BALANCE,
                allowance: INITIAL_TREASURY_ALLOWANCE,
                total_transferred: 0,
                mmcr_count: 0,
                last_mmcr_timestamp: 0,
                genesis_timestamp: 0,
                last_mmcr_month: 0,
                last_mmcr_year: 0,
            }
        ).unwrap()
    );
}

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID"));
    GOVERNANCE_CANISTER_ID.with(|id| id.borrow_mut().set(args.governance_canister_id).expect("Failed to set Governance Canister ID"));
    
    let now = ic_cdk::api::time();
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        if state.genesis_timestamp == 0 {
            state.genesis_timestamp = now;
            cell.set(state).expect("Failed to initialize treasury state");
        }
    });
    
    start_timers();
}

#[post_upgrade]
fn post_upgrade() {
    start_timers();
}

fn start_timers() {
    // MMCR timer - check every hour to ensure we don't miss the 1st of the month
    set_timer_interval(Duration::from_secs(60 * 60), || {
        let _ = try_execute_mmcr();
    });
}

// ============================================================================
// ACCESS CONTROL
// ============================================================================

/// Check if the caller is the governance canister
fn is_governance_canister() -> bool {
    let caller = ic_cdk::caller();
    let governance_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    caller == governance_id
}

/// Ensure caller is governance canister or controller
fn require_governance_or_controller() -> Result<(), String> {
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
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a month
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30, // Default fallback
    }
}

/// Convert nanoseconds since Unix epoch to date/time components (UTC)
fn nanos_to_datetime(nanos: u64) -> DateTimeComponents {
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
fn datetime_to_nanos(dt: &DateTimeComponents) -> u64 {
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
fn day_of_week(year: u16, month: u8, day: u8) -> u8 {
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
fn find_nth_weekday(year: u16, month: u8, weekday: u8, n: u8) -> u8 {
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
fn is_daylight_saving_time(nanos_utc: u64) -> bool {
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
fn get_eastern_offset_nanos(nanos_utc: u64) -> i64 {
    if is_daylight_saving_time(nanos_utc) {
        EDT_OFFSET_NANOS
    } else {
        EST_OFFSET_NANOS
    }
}

/// Convert UTC nanoseconds to Eastern Time date components
fn utc_to_eastern(nanos_utc: u64) -> DateTimeComponents {
    let offset = get_eastern_offset_nanos(nanos_utc);
    let eastern_nanos = if offset < 0 {
        nanos_utc.saturating_sub((-offset) as u64)
    } else {
        nanos_utc + offset as u64
    };
    nanos_to_datetime(eastern_nanos)
}

/// Get the UTC timestamp for midnight (12:00 AM) Eastern Time on the 1st of a given month/year
fn get_first_of_month_midnight_et_utc(year: u16, month: u8) -> u64 {
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
fn seconds_until_next_first_of_month(current_nanos_utc: u64) -> u64 {
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
/// (1st of the month, within the first hour at midnight Eastern Time)
fn is_in_mmcr_window(nanos_utc: u64) -> bool {
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
fn should_execute_mmcr(current_nanos: u64, last_month: u8, last_year: u16) -> bool {
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
// TREASURY FUNCTIONS
// ============================================================================

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
#[update]
async fn execute_transfer(input: ExecuteTransferInput) -> Result<u64, String> {
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

// ============================================================================
// MMCR (Monthly Minimum Capital Release)
// ============================================================================

fn try_execute_mmcr() -> Result<u64, String> {
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
