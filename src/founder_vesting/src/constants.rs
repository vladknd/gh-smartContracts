
/// One year in nanoseconds (365 days)
pub const YEAR_IN_NANOS: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;

/// Annual unlock rate: 10% = 1000 basis points
pub const ANNUAL_UNLOCK_BPS: u64 = 1000;

/// Maximum unlock: 100% = 10000 basis points (after 10 years)
pub const MAX_UNLOCK_BPS: u64 = 10000;

#[allow(dead_code)]
pub const FOUNDER_1_ALLOCATION: u64 = 350_000_000 * 100_000_000; // 3.5 * 10^16

#[allow(dead_code)]
pub const FOUNDER_2_ALLOCATION: u64 = 150_000_000 * 100_000_000; // 1.5 * 10^16
