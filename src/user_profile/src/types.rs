use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize, Clone)]
pub struct InitArgs {
    /// Principal ID of the staking hub canister
    pub staking_hub_id: Principal,
    /// Principal ID of the learning content canister
    pub learning_content_id: Principal,
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

/// User profile containing personal info and staking state
/// 
/// Simplified version without interest/tier system.
/// Users earn tokens from quizzes and can unstake them at any time with no penalty.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserProfile {
    // ─────────────────────────────────────────────────────────────────
    // Personal Information
    // ─────────────────────────────────────────────────────────────────
    pub email: String,
    pub name: String,
    pub education: String,
    pub gender: String,
    
    // ─────────────────────────────────────────────────────────────────
    // Verification
    // ─────────────────────────────────────────────────────────────────
    pub verification_tier: VerificationTier,

    // ─────────────────────────────────────────────────────────────────
    // Economy State
    // ─────────────────────────────────────────────────────────────────
    /// Total tokens currently staked by this user
    pub staked_balance: u64,
    
    /// Number of transactions for this user in local storage (used for indexing)
    pub transaction_count: u64,
    
    /// Number of transactions that have been archived (for pagination)
    pub archived_transaction_count: u64,

    /// Subscription status
    pub is_subscribed: bool,
}

impl Storable for UserProfile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode UserProfile")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1024, // Increased size for future proofing
        is_fixed_size: false,
    };
}

/// Input for updating user profile (personal info only, not economy state)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserProfileUpdate {
    pub email: String,
    pub name: String,
    pub education: String,
    pub gender: String,
}

/// Types of transactions that can be recorded
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransactionType {
    /// Tokens earned from completing quizzes
    QuizReward,
    /// Tokens withdrawn from staking
    Unstake,
}

/// A record of a single transaction for a user
/// 
/// Transactions are stored for auditing and UI display purposes.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionRecord {
    /// When the transaction occurred (nanoseconds since epoch)
    pub timestamp: u64,
    /// Type of transaction (QuizReward or Unstake)
    pub tx_type: TransactionType,
    /// Amount of tokens involved (in e8s = 1/100,000,000 of a token)
    pub amount: u64,
}

impl Storable for TransactionRecord {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode TransactionRecord")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

/// Composite key for transaction storage: (user principal, index)
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionKey {
    pub user: Principal,
    pub index: u64,
}

impl Storable for TransactionKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode TransactionKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserQuizKey {
    pub user: Principal,
    pub unit_id: String,
}

impl Storable for UserQuizKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode UserQuizKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

/// Pending statistics to be synced with the staking hub
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PendingStats {
    /// Change in staked amount since last sync
    pub staked_delta: i64,
    
    /// Total amount unstaked since last sync
    pub unstaked_delta: u64,
}

impl Storable for PendingStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode PendingStats")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
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
            max_daily_tokens: 200_000_000,      // 2 GHC
            max_weekly_tokens: 1_000_000_000,    // 10 GHC
            max_monthly_tokens: 3_000_000_000,   // 30 GHC
            max_yearly_tokens: 20_000_000_000,   // 200 GHC
        }
    }
}

/// Cached quiz configuration - stored locally to avoid inter-canister calls
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
            reward_amount: 10_000_000,      // 0.1 GHC
            pass_threshold_percent: 80,
            max_daily_attempts: 5,
            regular_limits: TokenLimits::default(),
            subscribed_limits: TokenLimits {
                max_daily_tokens: 500_000_000,    // 5 GHC
                max_weekly_tokens: 2_000_000_000,  // 20 GHC
                max_monthly_tokens: 6_000_000_000,  // 60 GHC
                max_yearly_tokens: 40_000_000_000, // 400 GHC
            },
            version: 0,
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

    const BOUND: Bound = Bound::Bounded {
        max_size: 300,
        is_fixed_size: false,
    };
}

/// Quiz data cached locally for O(1) verification
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizCacheData {
    pub content_id: String,
    pub answer_hashes: Vec<[u8; 32]>,
    pub question_count: u8,
    pub version: u64,
}

impl Storable for QuizCacheData {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode QuizCacheData")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000, 
        is_fixed_size: false,
    };
}

/// Comprehensive time-based statistics for a user
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserTimeStats {
    /// The last day this user was active (used to trigger resets)
    pub last_active_day: u64,
    
    // Daily Counters
    pub daily_quizzes: u8,
    pub daily_earnings: u64,
    
    // Weekly Counters
    pub weekly_quizzes: u8,
    pub weekly_earnings: u64,
    
    // Monthly Counters
    pub monthly_quizzes: u8,
    pub monthly_earnings: u64,
    
    // Yearly Counters
    pub yearly_quizzes: u16, // u16 because limit is 600+
    pub yearly_earnings: u64,
}

impl Default for UserTimeStats {
    fn default() -> Self {
        Self {
            last_active_day: 0,
            daily_quizzes: 0,
            daily_earnings: 0,
            weekly_quizzes: 0,
            weekly_earnings: 0,
            monthly_quizzes: 0,
            monthly_earnings: 0,
            yearly_quizzes: 0,
            yearly_earnings: 0,
        }
    }
}

impl Storable for UserTimeStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap_or_default()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Transaction page for paginated access with archive info
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionPage {
    pub transactions: Vec<TransactionRecord>,
    pub total_count: u64,
    pub local_count: u64,
    pub archived_count: u64,
    pub archive_canister_id: Option<Principal>,
    pub source: String,
    pub current_page: u32,
    pub total_pages: u32,
    pub has_archive_data: bool, 
}

/// Archive configuration info for frontends
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ArchiveConfig {
    pub retention_limit: u64,
    pub trigger_threshold: u64,
    pub check_interval_secs: u64,
    pub archive_canister_id: Option<Principal>,
    pub is_configured: bool,
}

/// Summary of a registered user for admin listing
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserSummary {
    pub user_principal: Principal,
    pub name: String,
    pub email: String,
    pub staked_balance: u64,
    pub verification_tier: VerificationTier,
    pub is_subscribed: bool,
}

/// Result of admin user listing with pagination info
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserListResult {
    pub users: Vec<UserSummary>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
    pub total_pages: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionToArchive {
    pub sequence: u64,
    pub timestamp: u64,
    pub transaction_type: String, 
    pub amount: u64,
    pub metadata: String,
}
