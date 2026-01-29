use candid::{CandidType, Deserialize, Principal, Encode, Decode, Nat};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;
use crate::constants::*;

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize, Clone)]
pub struct InitArgs {
    /// Principal ID of the GHC ledger canister (ICRC1)
    pub ledger_id: Principal,
    /// Principal ID of the learning content canister
    pub learning_content_id: Principal,
    /// Embedded WASM binary for auto-deploying user_profile shard canisters
    pub user_profile_wasm: Vec<u8>,
    /// Embedded WASM binary for auto-deploying archive canisters (optional)
    pub archive_canister_wasm: Option<Vec<u8>>,
}

/// Global statistics tracked by the staking hub
/// 
/// This is the central source of truth for all staking-related metrics.
/// Simplified version without interest/penalty system.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GlobalStats {
    /// Total tokens currently staked across all users
    pub total_staked: u64,
    
    /// Total tokens that have been unstaked
    pub total_unstaked: u64,
    
    /// Total tokens allocated for minting (tracked against MAX_SUPPLY cap)
    pub total_allocated: u64,
}

impl Storable for GlobalStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode GlobalStats")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

// ─────────────────────────────────────────────────────────────────
// Verification Levels
// ─────────────────────────────────────────────────────────────────
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationTier {
    None,       // 0: Fresh user
    Human,      // 1: DecideID verified (Not a bot)
    KYC,        // 2: Full Legal KYC (Passport/AML)
}

/// Operational status of a shard canister
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ShardStatus {
    /// Shard is accepting new user registrations
    Active,
    /// Shard has reached capacity and is not accepting new users
    Full,
}

/// Information about a user_profile shard canister
/// 
/// Shards are automatically deployed by the staking hub to distribute
/// user load. Each shard can hold up to SHARD_HARD_LIMIT (100K) users.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardInfo {
    /// Principal ID of the shard canister
    pub canister_id: Principal,
    /// Timestamp when this shard was created (nanoseconds)
    pub created_at: u64,
    /// Current number of registered users in this shard
    pub user_count: u64,
    /// Operational status of the shard
    pub status: ShardStatus,
    /// Principal ID of the associated archive canister (if created)
    pub archive_canister_id: Option<Principal>,
}

impl Storable for ShardInfo {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ShardInfo")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Wrapper for storing large WASM binary in stable memory
/// 
/// The embedded WASM is used by the hub to deploy new shard canisters
/// automatically when existing shards reach capacity.
#[derive(CandidType, Deserialize, Clone, Default)]
pub struct WasmBlob {
    pub data: Vec<u8>,
}

impl Storable for WasmBlob {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.data.clone())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self { data: bytes.to_vec() }
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TokenLimits {
    pub max_daily_tokens: u64,
    pub max_weekly_tokens: u64,
    pub max_monthly_tokens: u64,
    pub max_yearly_tokens: u64,
}

impl Default for TokenLimits {
    fn default() -> Self {
        Self {
            max_daily_tokens: MIN_REGULAR_DAILY,   // 2 GHC
            max_weekly_tokens: MIN_REGULAR_WEEKLY,  // 10 GHC
            max_monthly_tokens: MIN_REGULAR_MONTHLY, // 30 GHC
            max_yearly_tokens: MIN_REGULAR_YEARLY, // 200 GHC
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TokenLimitsConfig {
    pub reward_amount: u64,
    pub pass_threshold_percent: u8,
    pub max_daily_attempts: u8,
    pub regular_limits: TokenLimits,
    pub subscribed_limits: TokenLimits,
    pub version: u64,
}

impl Default for TokenLimitsConfig {
    fn default() -> Self {
        Self {
            reward_amount: 10_000_000,      // 0.1 GHC (Default reward reduced to fit new limits)
            pass_threshold_percent: 80,
            max_daily_attempts: 5,
            regular_limits: TokenLimits::default(),
            subscribed_limits: TokenLimits {
                max_daily_tokens: MIN_SUBSCRIBED_DAILY,    // 5 GHC
                max_weekly_tokens: MIN_SUBSCRIBED_WEEKLY, // 20 GHC
                max_monthly_tokens: MIN_SUBSCRIBED_MONTHLY, // 60 GHC
                max_yearly_tokens: MIN_SUBSCRIBED_YEARLY, // 400 GHC
            },
            version: 1, // Start with version 1 to indicate it's valid
        }
    }
}

impl Storable for TokenLimitsConfig {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode TokenLimitsConfig")
    }
    const BOUND: Bound = Bound::Bounded { max_size: 300, is_fixed_size: false };
}

/// Quiz cache data structure for distribution
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizCacheData {
    pub content_id: String,
    pub answer_hashes: Vec<[u8; 32]>,
    pub question_count: u8,
    pub version: u64,
}

// ============================================================================
// INTERNAL STRUCTS FOR INTER-CANISTER CALLS
// ============================================================================

#[derive(CandidType)]
pub struct CreateCanisterArgs {
    pub settings: Option<CanisterSettings>,
}

#[derive(CandidType, Clone)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}

#[derive(CandidType, Deserialize)]
pub struct CreateCanisterResult {
    pub canister_id: Principal,
}

#[derive(CandidType)]
pub struct InstallCodeArgs {
    pub mode: InstallMode,
    pub canister_id: Principal,
    pub wasm_module: Vec<u8>,
    pub arg: Vec<u8>,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize)]
pub enum InstallMode {
    install,
    reinstall,
    upgrade,
}

#[derive(CandidType)]
pub struct ArchiveInitArgs {
    pub parent_shard_id: Principal,
}

#[derive(CandidType)]
pub struct UserProfileInitArgs {
    pub staking_hub_id: Principal,
    pub learning_content_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct UserProfilePartial {
    pub email: String,
    pub name: String,
    pub education: String,
    pub gender: String,
    pub verification_tier: VerificationTier,
    pub staked_balance: u64,
    pub transaction_count: u64,
    pub archived_transaction_count: u64,
    pub is_subscribed: bool,
}
