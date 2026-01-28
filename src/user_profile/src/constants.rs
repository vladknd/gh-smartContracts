// Archive retention: keep last 100 transactions per user locally
pub const TRANSACTION_RETENTION_LIMIT: u64 = 100;

// Archive trigger: when user exceeds this, trigger immediate async archive
// Set higher than RETENTION_LIMIT to avoid archiving on every transaction
pub const ARCHIVE_TRIGGER_THRESHOLD: u64 = 150;

// Periodic archive check interval (6 hours in seconds)
pub const ARCHIVE_CHECK_INTERVAL_SECS: u64 = 6 * 60 * 60;

// Sync Timer: Every 5 seconds, sync pending stats with staking_hub
pub const _SYNC_INTERVAL_SECS: u64 = 5;

// Minting Allowance Thresholds
pub const ALLOWANCE_LOW_THRESHOLD: u64 = 500 * 100_000_000; // 500 Tokens
pub const ALLOWANCE_REFILL_AMOUNT: u64 = 1000 * 100_000_000; // 1000 Tokens

// Days in months (standard, non-leap)
pub const DAYS_IN_MONTHS: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
// Days in months (leap year)
pub const DAYS_IN_MONTHS_LEAP: [u64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
