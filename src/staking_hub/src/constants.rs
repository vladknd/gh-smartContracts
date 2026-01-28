// ===============================
// CONSTANTS
// ===============================

// MAX_SUPPLY: 4.75B MUC tokens with 8 decimals
// 4.75B * 10^8 = 4.75 * 10^17 (fits comfortably in u64 max of ~1.8 * 10^19)
pub const MAX_SUPPLY: u64 = 4_750_000_000 * 100_000_000; // 4.75B MUC Tokens (8 decimals)
pub const SHARD_SOFT_LIMIT: u64 = 90_000;  // Start creating new shard at 90K users
pub const SHARD_HARD_LIMIT: u64 = 100_000; // Max users per shard
pub const AUTO_SCALE_INTERVAL_SECS: u64 = 60; // Check every minute
