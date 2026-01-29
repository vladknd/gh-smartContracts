// ===============================
// CONSTANTS
// ===============================

// MAX_SUPPLY: 4.75B MUC tokens with 8 decimals
// 4.75B * 10^8 = 4.75 * 10^17 (fits comfortably in u64 max of ~1.8 * 10^19)
pub const MAX_SUPPLY: u64 = 4_750_000_000 * 100_000_000; // 4.75B MUC Tokens (8 decimals)
pub const SHARD_SOFT_LIMIT: u64 = 90_000;  // Start creating new shard at 90K users
pub const SHARD_HARD_LIMIT: u64 = 100_000; // Max users per shard
pub const AUTO_SCALE_INTERVAL_SECS: u64 = 60; // Check every minute

// Token Limit Ranges (8 decimals)
pub const MIN_REGULAR_DAILY: u64 = 2 * 100_000_000;
pub const MAX_REGULAR_DAILY: u64 = 10_000 * 100_000_000;   // Up to 10,000 GHC
pub const MIN_REGULAR_WEEKLY: u64 = 10 * 100_000_000;
pub const MAX_REGULAR_WEEKLY: u64 = 50_000 * 100_000_000;  // Up to 50,000 GHC
pub const MIN_REGULAR_MONTHLY: u64 = 30 * 100_000_000;
pub const MAX_REGULAR_MONTHLY: u64 = 200_000 * 100_000_000; // Up to 200,000 GHC
pub const MIN_REGULAR_YEARLY: u64 = 200 * 100_000_000;
pub const MAX_REGULAR_YEARLY: u64 = 2_000_000 * 100_000_000; // Up to 2M GHC

pub const MIN_SUBSCRIBED_DAILY: u64 = 5 * 100_000_000;
pub const MAX_SUBSCRIBED_DAILY: u64 = 100_000 * 100_000_000;  // Up to 100,000 GHC
pub const MIN_SUBSCRIBED_WEEKLY: u64 = 20 * 100_000_000;
pub const MAX_SUBSCRIBED_WEEKLY: u64 = 500_000 * 100_000_000; // Up to 500,000 GHC
pub const MIN_SUBSCRIBED_MONTHLY: u64 = 60 * 100_000_000;
pub const MAX_SUBSCRIBED_MONTHLY: u64 = 2_000_000 * 100_000_000; // Up to 2M GHC
pub const MIN_SUBSCRIBED_YEARLY: u64 = 400 * 100_000_000;
pub const MAX_SUBSCRIBED_YEARLY: u64 = 10_000_000 * 100_000_000; // Up to 10M GHC
