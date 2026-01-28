// ============================================================================
// TREASURY CONSTANTS (in e8s - 8 decimals)
// ============================================================================

pub const INITIAL_TREASURY_BALANCE: u64 = 425_000_000_000_000_000; // 4.25B * 10^8
pub const INITIAL_TREASURY_ALLOWANCE: u64 = 60_000_000_000_000_000; // 0.6B * 10^8
pub const MMCR_AMOUNT: u64 = 1_520_000_000_000_000; // 15.2M * 10^8
pub const FINAL_MMCR_AMOUNT: u64 = 1_720_000_000_000_000; // 17.2M * 10^8
pub const TOTAL_MMCR_RELEASES: u64 = 240;

// Minimum interval between MMCR releases (25 days - safety margin for calendar-based scheduling)
pub const MMCR_MIN_INTERVAL_NANOS: u64 = 25 * 24 * 60 * 60 * 1_000_000_000;

// ============================================================================
// TIME CONSTANTS
// ============================================================================

// Nanoseconds per unit
pub const NANOS_PER_SECOND: u64 = 1_000_000_000;
pub const NANOS_PER_MINUTE: u64 = 60 * NANOS_PER_SECOND;
pub const NANOS_PER_HOUR: u64 = 60 * NANOS_PER_MINUTE;
pub const NANOS_PER_DAY: u64 = 24 * NANOS_PER_HOUR;

// UTC offset in nanoseconds
pub const EST_OFFSET_NANOS: i64 = -5 * 60 * 60 * NANOS_PER_SECOND as i64; // UTC-5:00
pub const EDT_OFFSET_NANOS: i64 = -4 * 60 * 60 * NANOS_PER_SECOND as i64; // UTC-4:00

// Epoch reference: January 1, 1970 00:00:00 UTC (Unix epoch)
pub const DAYS_PER_400_YEARS: u64 = 146097;
pub const DAYS_PER_100_YEARS: u64 = 36524;
pub const DAYS_PER_4_YEARS: u64 = 1461;
pub const DAYS_PER_YEAR: u64 = 365;
