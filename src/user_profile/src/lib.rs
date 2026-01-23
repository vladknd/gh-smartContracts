use std::time::Duration;
use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use ic_cdk_timers::set_timer_interval;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use candid::{CandidType, Deserialize, Principal};
use candid::{Encode, Decode};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    /// Principal ID of the staking hub canister
    staking_hub_id: Principal,
    /// Principal ID of the learning content canister
    learning_content_id: Principal,
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
struct UserProfile {
    // ─────────────────────────────────────────────────────────────────
    // Personal Information
    // ─────────────────────────────────────────────────────────────────
    email: String,
    name: String,
    education: String,
    gender: String,
    
    // ─────────────────────────────────────────────────────────────────
    // Verification
    // ─────────────────────────────────────────────────────────────────
    pub verification_tier: VerificationTier,

    // ─────────────────────────────────────────────────────────────────
    // Economy State
    // ─────────────────────────────────────────────────────────────────
    /// Total tokens currently staked by this user
    staked_balance: u64,
    
    /// Number of transactions for this user in local storage (used for indexing)
    transaction_count: u64,
    
    /// Number of transactions that have been archived (for pagination)
    archived_transaction_count: u64,
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
struct UserProfileUpdate {
    email: String,
    name: String,
    education: String,
    gender: String,
}

/// Types of transactions that can be recorded
#[derive(CandidType, Deserialize, Clone, Debug)]
enum TransactionType {
    /// Tokens earned from completing quizzes
    QuizReward,
    /// Tokens withdrawn from staking
    Unstake,
}

/// A record of a single transaction for a user
/// 
/// Transactions are stored for auditing and UI display purposes.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct TransactionRecord {
    /// When the transaction occurred (nanoseconds since epoch)
    timestamp: u64,
    /// Type of transaction (QuizReward or Unstake)
    tx_type: TransactionType,
    /// Amount of tokens involved (in e8s = 1/100,000,000 of a token)
    amount: u64,
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
struct TransactionKey {
    user: Principal,
    index: u64,
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



/// Composite key for quiz completion tracking: (user principal, unit_id)
/// 
/// Used to ensure users can only complete each quiz once.
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UserQuizKey {
    user: Principal,
    unit_id: String,
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
/// 
/// These values accumulate between sync cycles (every 5 seconds)
/// and are reported to the hub in batch for efficiency.
/// This is the "micro-bank" pattern that enables eventual consistency.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct PendingStats {
    /// Change in staked amount since last sync
    /// Positive = new stakes from quiz rewards
    /// Negative = unstakes
    staked_delta: i64,
    
    /// Total amount unstaked since last sync
    unstaked_delta: u64,
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

/// Cached quiz configuration - stored locally to avoid inter-canister calls
/// Updated when governance submits a proposal or on periodic sync
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CachedQuizConfig {
    pub reward_amount: u64,
    pub pass_threshold_percent: u8,
    pub max_daily_attempts: u8,
    pub max_daily_quizzes: u8,
    pub max_weekly_quizzes: u8,
    pub max_monthly_quizzes: u8,
    pub max_yearly_quizzes: u16,
    /// Version number to track updates
    pub version: u64,
}

impl Default for CachedQuizConfig {
    fn default() -> Self {
        // Reasonable defaults - will be overwritten on first sync
        Self {
            reward_amount: 10_000_000,      // 0.1 GHC
            pass_threshold_percent: 80,
            max_daily_attempts: 3,
            max_daily_quizzes: 10,
            max_weekly_quizzes: 50,
            max_monthly_quizzes: 150,
            max_yearly_quizzes: 1000,
            version: 0,
        }
    }
}

impl Storable for CachedQuizConfig {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode CachedQuizConfig")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// All persistent state is stored in stable memory using ic_stable_structures.
// Each storage item is assigned a unique MemoryId for isolation.
// 
// Memory IDs:
//   0 - STAKING_HUB_ID: Configuration
//   1 - LEARNING_CONTENT_ID: Configuration
//   2 - USER_PROFILES: User data
//   3 - USER_DAILY_STATS: Daily rate limiting
//   4 - COMPLETED_QUIZZES: Quiz completion tracking
//   5 - MINTING_ALLOWANCE: Economy state
//   6 - PENDING_STATS: Batched sync data
//   7 - USER_TRANSACTIONS: Transaction history

thread_local! {
    // ─────────────────────────────────────────────────────────────────────
    // Memory Management
    // ─────────────────────────────────────────────────────────────────────
    
    /// Memory manager for allocating virtual memory regions to each storage
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // ─────────────────────────────────────────────────────────────────────
    // Configuration (Set once during init, immutable after)
    // ─────────────────────────────────────────────────────────────────────

    /// Principal ID of the staking_hub canister
    /// Used for sync_shard calls and authorization
    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the learning_engine canister
    /// Used to verify quiz answers
    static LEARNING_CONTENT_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );
}

// ============================================================================
// DATE HELPERS
// ============================================================================

/// Get current day index (days since epoch)
fn get_current_day() -> u64 {
    ic_cdk::api::time() / 86_400_000_000_000
}

/// Get week index (weeks since epoch)
fn get_week_index(day: u64) -> u64 {
    day / 7
}

/// Simple date struct
struct SimpleDate {
    year: u32,
    month: u32, // 1-12
    _day: u32,  // 1-31
}

/// Convert days since epoch to date (Simplified Gregorian)
/// This is sufficient for determining if we are in a new month/year
fn day_to_date(day: u64) -> SimpleDate {
    let mut d = day;
    let mut y = 1970;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    
    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let days_in_months = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31
    ];
    
    let mut m = 0;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        m += 1;
    }
    
    SimpleDate {
        year: y,
        month: m + 1,
        _day: (d + 1) as u32,
    }
}

/// Comprehensive time-based statistics for a user
/// Tracks activity across Daily, Weekly, Monthly, and Yearly periods.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserTimeStats {
    /// The last day this user was active (used to trigger resets)
    last_active_day: u64,
    
    // Daily Counters
    daily_quizzes: u8,
    daily_earnings: u64,
    
    // Weekly Counters
    weekly_quizzes: u8,
    weekly_earnings: u64,
    
    // Monthly Counters
    monthly_quizzes: u8,
    monthly_earnings: u64,
    
    // Yearly Counters
    yearly_quizzes: u16, // u16 because limit is 600+
    yearly_earnings: u64,
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

// ============================================================================
// USER DATA STORAGE
// ============================================================================

thread_local! {
    /// Map of user Principal -> UserProfile
    /// Contains all registered users and their staking state
    /// This is the primary user data store
    static USER_PROFILES: RefCell<StableBTreeMap<Principal, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Map of user Principal -> UserTimeStats
    /// Tracks quiz/earnings limits per user across all timeframes
    static USER_TIME_STATS: RefCell<StableBTreeMap<Principal, UserTimeStats, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    /// Set of completed quiz keys: (user, unit_id) -> true
    /// Prevents users from re-completing the same quiz
    static COMPLETED_QUIZZES: RefCell<StableBTreeMap<UserQuizKey, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// Map of TransactionKey -> TransactionRecord
    /// Stores transaction history for each user
    /// Key is (user, index) where index is auto-incremented
    static USER_TRANSACTIONS: RefCell<StableBTreeMap<TransactionKey, TransactionRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );

    // ─────────────────────────────────────────────────────────────────────
    // Economy State (Micro-Bank Pattern)
    // ─────────────────────────────────────────────────────────────────────

    /// Minting allowance granted by the staking hub
    /// This shard can mint up to this amount without asking the hub
    /// Refilled automatically when running low (< 500 tokens)
    static MINTING_ALLOWANCE: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );

    /// Pending statistics to report to the hub on next sync
    /// Accumulates stake changes and unstakes
    /// Cleared after successful sync, rolled back on failure
    static PENDING_STATS: RefCell<StableCell<PendingStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            PendingStats { staked_delta: 0, unstaked_delta: 0 }
        ).unwrap()
    );

    /// Quiz Cache: unit_id -> QuizCacheData
    /// Optimizes verification to avoid cross-canister calls
    static QUIZ_CACHE: RefCell<StableBTreeMap<String, QuizCacheData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );

    /// Archive canister ID - stores archived transaction history
    /// Set by staking_hub when creating this shard
    static ARCHIVE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            Principal::anonymous()
        ).unwrap()
    );
    
    /// Cached global quiz config - avoids inter-canister calls on every quiz submission
    /// Updated via receive_quiz_config endpoint from staking_hub
    static CACHED_QUIZ_CONFIG: RefCell<StableCell<CachedQuizConfig, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            CachedQuizConfig::default()
        ).unwrap()
    );
}

// Archive retention: keep last 100 transactions per user locally
const TRANSACTION_RETENTION_LIMIT: u64 = 100;

// Archive trigger: when user exceeds this, trigger immediate async archive
// Set higher than RETENTION_LIMIT to avoid archiving on every transaction
const ARCHIVE_TRIGGER_THRESHOLD: u64 = 150;

// Periodic archive check interval (6 hours in seconds)
const ARCHIVE_CHECK_INTERVAL_SECS: u64 = 6 * 60 * 60;

// Note: No artificial limit on quiz cache - stable memory can handle millions of entries
// The quiz cache is a copy of learning_engine data, same on all shards

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

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    LEARNING_CONTENT_ID.with(|id| id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID"));

    start_timers();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    start_timers();
}

/// Start all periodic timers
fn start_timers() {
    // Sync Timer: Every 5 seconds, sync pending stats with staking_hub
    set_timer_interval(Duration::from_secs(5), || {
        ic_cdk::spawn(async {
            let _ = sync_with_hub_internal().await;
        });
    });
    
    // Archive Timer: Every 6 hours, archive old transactions for all users
    set_timer_interval(Duration::from_secs(ARCHIVE_CHECK_INTERVAL_SECS), || {
        ic_cdk::spawn(async {
            let _ = run_periodic_archive().await;
        });
    });
}

/// Periodic archiving task - archives old transactions for all users exceeding limit
async fn run_periodic_archive() -> Result<u64, String> {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    if archive_id == Principal::anonymous() {
        // Archive not configured, skip silently
        return Ok(0);
    }
    
    // Get list of users who need archiving
    let users_to_archive: Vec<(Principal, u64)> = USER_PROFILES.with(|p| {
        p.borrow().iter()
            .filter(|(_, profile)| profile.transaction_count > TRANSACTION_RETENTION_LIMIT)
            .map(|(user, profile)| (user, profile.transaction_count - TRANSACTION_RETENTION_LIMIT))
            .collect()
    });
    
    let mut total_archived = 0u64;
    
    for (user, excess) in users_to_archive {
        match archive_user_transactions(user, excess, archive_id).await {
            Ok(count) => total_archived += count,
            Err(e) => ic_cdk::print(format!("Periodic archive failed for {}: {}", user, e)),
        }
    }
    
    if total_archived > 0 {
        ic_cdk::print(format!("Periodic archive completed: {} transactions archived", total_archived));
    }
    
    Ok(total_archived)
}

#[update]
async fn register_user(args: UserProfileUpdate) -> Result<(), String> {
    let user = ic_cdk::caller();
    
    if user == Principal::anonymous() {
        return Err("Anonymous registration is not allowed. Please ensure your frontend is passing the authenticated identity.".to_string());
    }
    
    if USER_PROFILES.with(|p| p.borrow().contains_key(&user)) {
        return Err("User already registered".to_string());
    }

    let new_profile = UserProfile {
        email: args.email,
        name: args.name,
        education: args.education,
        gender: args.gender,
        verification_tier: VerificationTier::None,
        staked_balance: 0,
        transaction_count: 0,
        archived_transaction_count: 0,
    };

    USER_PROFILES.with(|p| p.borrow_mut().insert(user, new_profile));
    
    // Register user's shard location with staking_hub (for governance voting power lookup)
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let _: Result<(Result<(), String>,), _> = ic_cdk::call(
        staking_hub_id,
        "register_user_location",
        (user,)
    ).await;
    // Note: We don't fail registration if this call fails - it's not critical for user operations
    // The user registry is only needed for governance voting
    
    Ok(())
}

#[update]
fn update_profile(args: UserProfileUpdate) -> Result<(), String> {
    let user = ic_cdk::caller();
    if user == Principal::anonymous() {
        return Err("Anonymous actions are not allowed.".to_string());
    }
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            profile.email = args.email;
            profile.name = args.name;
            profile.education = args.education;
            profile.gender = args.gender;
            map.insert(user, profile);
            Ok(())
        } else {
            Err("User not registered".to_string())
        }
    })
}

#[query]
fn get_profile(user: Principal) -> Option<UserProfile> {
    USER_PROFILES.with(|p| p.borrow().get(&user))
}

// ============================================================================
// HUB SYNCHRONIZATION
// ============================================================================

/// Periodically sync local statistics with the staking hub
/// 
/// This function runs every 5 seconds and performs these operations:
/// 1. Reports stake changes to the hub
/// 2. Reports unstaking activity
/// 3. Requests more minting allowance if running low
/// 
/// Uses optimistic reset pattern: values are cleared before the call
/// and rolled back if the call fails.
async fn sync_with_hub_internal() -> Result<(), String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // ─────────────────────────────────────────────────────────────────
    // Step 1: Optimistic Reset - Capture and clear pending stats
    // ─────────────────────────────────────────────────────────────────
    let (staked_delta, unstaked_delta) = PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        let s_delta = stats.staked_delta;
        let u_delta = stats.unstaked_delta;
        
        // Clear for next cycle
        stats.staked_delta = 0;
        stats.unstaked_delta = 0;
        
        cell.set(stats).expect("Failed to update pending stats");
        
        (s_delta, u_delta)
    });

    // ─────────────────────────────────────────────────────────────────
    // Step 2: Smart Allowance Request
    // ─────────────────────────────────────────────────────────────────
    let current_allowance = MINTING_ALLOWANCE.with(|a| *a.borrow().get());
    let low_threshold = 500 * 100_000_000; // 500 Tokens
    let refill_amount = 1000 * 100_000_000; // 1000 Tokens

    let requested_allowance: u64 = if current_allowance < low_threshold {
        refill_amount
    } else {
        0
    };

    // ─────────────────────────────────────────────────────────────────
    // Step 3: Call hub with simplified sync
    // ─────────────────────────────────────────────────────────────────
    let result: Result<(Result<u64, String>,), _> = ic_cdk::call(
        staking_hub_id,
        "sync_shard",
        (staked_delta, unstaked_delta, requested_allowance)
    ).await;

    match result {
        Ok((Ok(granted),)) => {
            // Success! Update local state with hub data
            
            // Update minting allowance
            MINTING_ALLOWANCE.with(|a| {
                let current = *a.borrow().get();
                a.borrow_mut().set(current + granted).expect("Failed to update allowance");
            });

            // Report user count to hub for load balancing
            let user_count = USER_PROFILES.with(|p| p.borrow().len());
            let _ : Result<(Result<(), String>,), _> = ic_cdk::call(
                staking_hub_id,
                "update_shard_user_count",
                (user_count,)
            ).await;

            Ok(())
        },
        Ok((Err(msg),)) => {
            // Hub rejected our request - rollback local stats
            rollback_pending_stats(staked_delta, unstaked_delta);
            Err(format!("Hub Rejected Sync: {}", msg))
        },
        Err((code, msg)) => {
            // Network/system error - rollback local stats
            rollback_pending_stats(staked_delta, unstaked_delta);
            Err(format!("Hub Call Failed: {:?} {}", code, msg))
        }
    }
}

/// Rollback pending stats after a failed sync attempt
fn rollback_pending_stats(staked_delta: i64, unstaked_delta: u64) {
    PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.staked_delta += staked_delta;
        stats.unstaked_delta += unstaked_delta;
        cell.set(stats).expect("Failed to rollback pending stats");
    });
}


// Local struct to match Learning Engine's QuizConfig return
#[derive(CandidType, Deserialize, Clone, Debug)]
struct RemoteQuizConfig {
    pub reward_amount: u64,
    pub pass_threshold_percent: u8,
    pub max_daily_attempts: u8,
    pub max_daily_quizzes: u8,
    pub max_weekly_quizzes: u8,
    pub max_monthly_quizzes: u8,
    pub max_yearly_quizzes: u16,
}

// ===============================
// QUIZ CACHING & VERIFICATION
// ===============================

/// Stable deterministic hash for answer verification (must match Learning Engine)
fn stable_hash(data: &[u8]) -> [u8; 32] {
    let mut hash: u64 = 5381;
    for b in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*b as u64);
    }
    let b = hash.to_le_bytes();
    let mut res = [0u8; 32];
    res[0..8].copy_from_slice(&b);
    res[8..16].copy_from_slice(&b);
    res[16..24].copy_from_slice(&b);
    res[24..32].copy_from_slice(&b);
    res
}

/// Receive a single quiz cache update from the Hub
#[update]
fn receive_quiz_cache(unit_id: String, cache: QuizCacheData) {
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    if ic_cdk::caller() != hub_id {
         ic_cdk::trap("Unauthorized cache update");
    }
    
    QUIZ_CACHE.with(|q| {
        q.borrow_mut().insert(unit_id, cache);
    });
}

/// Receive full cache sync from Hub (for new shards)
#[update]
fn receive_full_quiz_cache(caches: Vec<(String, QuizCacheData)>) {
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    if ic_cdk::caller() != hub_id {
         ic_cdk::trap("Unauthorized cache sync");
    }

    QUIZ_CACHE.with(|q| {
        let mut map = q.borrow_mut();
        for (id, data) in caches {
            map.insert(id, data);
        }
    });
}

/// Receive quiz config update from staking_hub
/// Called when governance approves UpdateGlobalQuizConfig proposal
#[update]
fn receive_quiz_config(config: CachedQuizConfig) {
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    if ic_cdk::caller() != hub_id {
         ic_cdk::trap("Unauthorized config update");
    }
    
    CACHED_QUIZ_CONFIG.with(|c| {
        c.borrow_mut().set(config).expect("Failed to set quiz config");
    });
}

/// Get the locally cached quiz config
#[query]
fn get_cached_quiz_config() -> CachedQuizConfig {
    CACHED_QUIZ_CONFIG.with(|c| c.borrow().get().clone())
}

#[update]
async fn submit_quiz(unit_id: String, answers: Vec<u8>) -> Result<u64, String> {
    let user = ic_cdk::caller();
    if user == Principal::anonymous() {
        return Err("Anonymous actions are not allowed.".to_string());
    }
    
    // 0. Check Registration
    if !USER_PROFILES.with(|p| p.borrow().contains_key(&user)) {
        return Err("User not registered".to_string());
    }

    let key = UserQuizKey { user, unit_id: unit_id.clone() };
    
    // 1. Get Config from LOCAL CACHE (no inter-canister call!)
    let config = CACHED_QUIZ_CONFIG.with(|c| c.borrow().get().clone());
    
    // Check if config has been initialized (version > 0 means it's been synced)
    // If not initialized, try to fetch from learning engine as fallback
    let config = if config.version == 0 {
        let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
        let (remote_config,): (RemoteQuizConfig,) = ic_cdk::call(
            learning_content_id,
            "get_global_quiz_config",
            ()
        ).await.map_err(|(code, msg)| format!("Failed to fetch quiz config: {:?} {}", code, msg))?;
        
        // Cache it locally for future use
        let cached = CachedQuizConfig {
            reward_amount: remote_config.reward_amount,
            pass_threshold_percent: remote_config.pass_threshold_percent,
            max_daily_attempts: remote_config.max_daily_attempts,
            max_daily_quizzes: remote_config.max_daily_quizzes,
            max_weekly_quizzes: remote_config.max_weekly_quizzes,
            max_monthly_quizzes: remote_config.max_monthly_quizzes,
            max_yearly_quizzes: remote_config.max_yearly_quizzes,
            version: 1, // Mark as initialized
        };
        CACHED_QUIZ_CONFIG.with(|c| c.borrow_mut().set(cached.clone()).expect("Failed to cache config"));
        cached
    } else {
        config
    };

    // 2. Check Time Limits
    let current_day = get_current_day();
    let current_week = get_week_index(current_day);
    let current_date = day_to_date(current_day);
    
    let mut stats = USER_TIME_STATS.with(|s| {
        s.borrow().get(&user).unwrap_or_default()
    });

    // Check for Resets
    let last_date = day_to_date(stats.last_active_day);
    let last_week = get_week_index(stats.last_active_day);
    
    // If we've moved to a new day (or user was never active)
    // Note: If last_active_day is 0 (default), this logic handles it correctly
    if current_day > stats.last_active_day {
        stats.daily_quizzes = 0;
        stats.daily_earnings = 0;
        
        // Check Weekly Reset
        if current_week > last_week {
            stats.weekly_quizzes = 0;
            stats.weekly_earnings = 0;
        }
        
        // Check Monthly Reset (Month or Year changed)
        if current_date.month != last_date.month || current_date.year != last_date.year {
            stats.monthly_quizzes = 0;
            stats.monthly_earnings = 0;
        }
        
        // Check Yearly Reset
        if current_date.year != last_date.year {
            stats.yearly_quizzes = 0;
            stats.yearly_earnings = 0;
        }
        
        // Update last active day
        stats.last_active_day = current_day;
    }

    // Enforce Limits
    if stats.daily_quizzes >= config.max_daily_quizzes {
        return Err(format!("Daily quiz limit reached ({}/{})", stats.daily_quizzes, config.max_daily_quizzes));
    }
    if stats.weekly_quizzes >= config.max_weekly_quizzes {
        return Err(format!("Weekly quiz limit reached ({}/{})", stats.weekly_quizzes, config.max_weekly_quizzes));
    }
    if stats.monthly_quizzes >= config.max_monthly_quizzes {
         return Err(format!("Monthly quiz limit reached ({}/{})", stats.monthly_quizzes, config.max_monthly_quizzes));
    }
    if stats.yearly_quizzes >= config.max_yearly_quizzes {
         return Err(format!("Yearly quiz limit reached ({}/{})", stats.yearly_quizzes, config.max_yearly_quizzes));
    }

    // 3. Check Minting Allowance (Hard Cap Enforcement)
    let reward_amount = config.reward_amount;
    let current_allowance = MINTING_ALLOWANCE.with(|a| *a.borrow().get());
    
    if current_allowance < reward_amount {
        // Try to refill immediately (blocking for user, but ensures safety)
        if let Err(e) = sync_with_hub_internal().await {
            return Err(e);
        }
        // Re-check
        let new_allowance = MINTING_ALLOWANCE.with(|a| *a.borrow().get());
        if new_allowance < reward_amount {
            return Err("Global minting limit reached or Hub unavailable.".to_string());
        }
    }
    
    // 4. Check if already completed
    if COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key)) {
        return Err("Quiz already completed".to_string());
    }

    // 5. Verify Answers
    // Try Local Cache First
    let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    
    let (passed, correct_count, total_questions) = if let Some(cache) = QUIZ_CACHE.with(|q| q.borrow().get(&unit_id)) {
        // Local Verification - no inter-canister call!
        let total = cache.question_count as u64;
        
        if total != answers.len() as u64 {
             (false, 0, total)
        } else {
             let mut correct = 0;
             for (i, ans) in answers.iter().enumerate() {
                 if stable_hash(&ans.to_le_bytes()) == cache.answer_hashes[i] {
                     correct += 1;
                 }
             }
             let threshold = config.pass_threshold_percent as u64;
             let passed = if total > 0 { (correct * 100 / total) >= threshold } else { false };
             (passed, correct, total)
        }
    } else {
        // Cache miss - fetch from learning engine, store locally, then verify
        // 1. Fetch quiz cache data from learning engine
        let fetch_result: Result<(Option<QuizCacheData>,), _> = ic_cdk::call(
            learning_content_id,
            "get_quiz_data",
            (unit_id.clone(),)
        ).await;
        
        match fetch_result {
            Ok((Some(cache_data),)) => {
                // 2. Store in local cache for future use
                QUIZ_CACHE.with(|q| {
                    q.borrow_mut().insert(unit_id.clone(), cache_data.clone());
                });
                
                // 3. Verify locally with the fetched data
                let total = cache_data.question_count as u64;
                
                if total != answers.len() as u64 {
                    (false, 0, total)
                } else {
                    let mut correct = 0;
                    for (i, ans) in answers.iter().enumerate() {
                        if stable_hash(&ans.to_le_bytes()) == cache_data.answer_hashes[i] {
                            correct += 1;
                        }
                    }
                    let threshold = config.pass_threshold_percent as u64;
                    let passed = if total > 0 { (correct * 100 / total) >= threshold } else { false };
                    (passed, correct, total)
                }
            }
            Ok((None,)) => {
                // Quiz not found
                return Err("Quiz not found in learning engine".to_string());
            }
            Err((code, msg)) => {
                // Fallback to remote verification if cache fetch fails
                ic_cdk::call::<(String, Vec<u8>), (bool, u64, u64)>(
                    learning_content_id,
                    "verify_quiz",
                    (unit_id.clone(), answers)
                ).await.map_err(|(code, msg)| format!("Failed to verify quiz: {:?} {}", code, msg))?
            }
        }
    };

    if total_questions == 0 {
        return Err("Unit not found or empty quiz".to_string());
    }
    
    // 6. Update Stats
    stats.daily_quizzes += 1;
    stats.weekly_quizzes += 1;
    stats.monthly_quizzes += 1;
    stats.yearly_quizzes += 1;

    if !passed {
        USER_TIME_STATS.with(|s| s.borrow_mut().insert(user, stats));
        return Err(format!("Quiz failed. Score: {}/{}. Need {}% to pass.", correct_count, total_questions, config.pass_threshold_percent));
    }

    // 7. Reward & Update Balance LOCALLY
    stats.daily_earnings += reward_amount;
    stats.weekly_earnings += reward_amount;
    stats.monthly_earnings += reward_amount;
    stats.yearly_earnings += reward_amount;
    
    USER_TIME_STATS.with(|s| s.borrow_mut().insert(user, stats));

    // Update User Profile Balance
    let should_archive = USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            let now = ic_cdk::api::time();
            
            profile.staked_balance += reward_amount;
            
            // Log Transaction
            let tx_index = profile.transaction_count;
            profile.transaction_count += 1;
            
            let tx_record = TransactionRecord {
                timestamp: now,
                tx_type: TransactionType::QuizReward,
                amount: reward_amount,
            };
            
            USER_TRANSACTIONS.with(|t| t.borrow_mut().insert(TransactionKey { user, index: tx_index }, tx_record));
            
            // Check if user exceeds archive trigger threshold
            let needs_archive = profile.transaction_count > ARCHIVE_TRIGGER_THRESHOLD;
            
            map.insert(user, profile);
            needs_archive
        } else {
            false
        }
    });
    
    // Trigger async archive if threshold exceeded (non-blocking)
    if should_archive {
        let user_to_archive = user;
        ic_cdk::spawn(async move {
            let _ = maybe_archive_user(user_to_archive).await;
        });
    }

    // Deduct Allowance
    MINTING_ALLOWANCE.with(|a| {
        let current = *a.borrow().get();
        a.borrow_mut().set(current - reward_amount).expect("Failed to update allowance");
    });

    // Update Pending Stats (Batching)
    PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.staked_delta += reward_amount as i64;
        cell.set(stats).expect("Failed to update pending stats");
    });

    // 8. Mark Completed
    COMPLETED_QUIZZES.with(|q| q.borrow_mut().insert(key, true));

    Ok(reward_amount)
}

/// Check if a user needs archiving and archive if so
async fn maybe_archive_user(user: Principal) -> Result<u64, String> {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    if archive_id == Principal::anonymous() {
        return Ok(0);
    }
    
    let excess = USER_PROFILES.with(|p| {
        p.borrow().get(&user).map(|profile| {
            if profile.transaction_count > TRANSACTION_RETENTION_LIMIT {
                profile.transaction_count - TRANSACTION_RETENTION_LIMIT
            } else {
                0
            }
        }).unwrap_or(0)
    });
    
    if excess > 0 {
        archive_user_transactions(user, excess, archive_id).await
    } else {
        Ok(0)
    }
}

#[query]
fn get_user_stats(user: Principal) -> UserTimeStats {
    let current_day = get_current_day();
    let current_week = get_week_index(current_day);
    let current_date = day_to_date(current_day);
    
    USER_TIME_STATS.with(|s| {
        let s = s.borrow();
        if let Some(mut stats) = s.get(&user) {
            // Return 'projected' stats (reset if needed) so UI sees correct available quota
            // This is just a view, doesn't modify state
             let last_date = day_to_date(stats.last_active_day);
             let last_week = get_week_index(stats.last_active_day);
             
             if current_day > stats.last_active_day {
                stats.daily_quizzes = 0;
                stats.daily_earnings = 0;
                
                if current_week > last_week {
                    stats.weekly_quizzes = 0;
                    stats.weekly_earnings = 0;
                }
                
                if current_date.month != last_date.month || current_date.year != last_date.year {
                    stats.monthly_quizzes = 0;
                    stats.monthly_earnings = 0;
                }
                
                if current_date.year != last_date.year {
                    stats.yearly_quizzes = 0;
                    stats.yearly_earnings = 0;
                }
             }
             stats
        } else {
            UserTimeStats::default()
        }
    })
}

/// Deprecated alias for get_user_stats - kept for frontend compatibility
#[query]
fn get_user_daily_status(user: Principal) -> UserTimeStats {
    get_user_stats(user)
}

#[query]
fn is_quiz_completed(user: Principal, unit_id: String) -> bool {
    let key = UserQuizKey { user, unit_id };
    COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key))
}

#[update]
async fn unstake(amount: u64) -> Result<u64, String> {
    let user = ic_cdk::caller();
    if user == Principal::anonymous() {
        return Err("Anonymous actions are not allowed.".to_string());
    }
    
    // 1. Check Balance and Registration
    let mut profile = USER_PROFILES.with(|p| p.borrow().get(&user).ok_or("User not registered".to_string()))?;

    if profile.staked_balance < amount {
        return Err(format!("Insufficient balance. Available: {}", profile.staked_balance));
    }

    // 2. Update Local State (Optimistic Update)
    profile.staked_balance -= amount;
    
    // Log Transaction (Optimistic)
    let tx_index = profile.transaction_count;
    profile.transaction_count += 1;
    
    let tx_record = TransactionRecord {
        timestamp: ic_cdk::api::time(),
        tx_type: TransactionType::Unstake,
        amount,
    };
    
    USER_TRANSACTIONS.with(|t| t.borrow_mut().insert(TransactionKey { user, index: tx_index }, tx_record));
    
    USER_PROFILES.with(|p| p.borrow_mut().insert(user, profile.clone()));

    // 3. Update Pending Stats (Reduce Total Staked)
    PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.staked_delta -= amount as i64;
        cell.set(stats).expect("Failed to update pending stats");
    });

    // 4. Call Hub to Process Unstake (Transfer Tokens) - No penalty!
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let result: Result<(Result<u64, String>,), _> = ic_cdk::call(
        staking_hub_id,
        "process_unstake",
        (user, amount)
    ).await;

    match result {
        Ok((Ok(return_amount),)) => Ok(return_amount),
        Ok((Err(msg),)) => {
            // Rollback Local State
            profile.staked_balance += amount;
            profile.transaction_count -= 1; // Revert count
            USER_TRANSACTIONS.with(|t| t.borrow_mut().remove(&TransactionKey { user, index: tx_index }));
            
            USER_PROFILES.with(|p| p.borrow_mut().insert(user, profile));
            
            PENDING_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.staked_delta += amount as i64;
                cell.set(stats).expect("Failed to rollback pending stats");
            });
            Err(format!("Hub Rejected Unstake: {}", msg))
        },
        Err((code, msg)) => {
            // Rollback Local State
            profile.staked_balance += amount;
            profile.transaction_count -= 1; // Revert count
            USER_TRANSACTIONS.with(|t| t.borrow_mut().remove(&TransactionKey { user, index: tx_index }));
            
            USER_PROFILES.with(|p| p.borrow_mut().insert(user, profile));
            
            PENDING_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.staked_delta += amount as i64;
                cell.set(stats).expect("Failed to rollback pending stats");
            });
            Err(format!("Hub Call Failed: {:?} {}", code, msg))
        }
    }
}

#[query]
fn get_user_transactions(user: Principal) -> Vec<TransactionRecord> {
    let count = USER_PROFILES.with(|p| {
        p.borrow().get(&user).map(|profile| profile.transaction_count).unwrap_or(0)
    });
    
    let mut transactions = Vec::new();
    USER_TRANSACTIONS.with(|t| {
        let t = t.borrow();
        for i in 0..count {
            if let Some(record) = t.get(&TransactionKey { user, index: i }) {
                transactions.push(record);
            }
        }
    });
    
    transactions
}

#[update]
async fn debug_force_sync() -> Result<(), String> {
    sync_with_hub_internal().await
}

/// Get total number of registered users in this shard
#[query]
fn get_user_count() -> u64 {
    USER_PROFILES.with(|p| p.borrow().len())
}

// ─────────────────────────────────────────────────────────────────
// Verification Logic
// ─────────────────────────────────────────────────────────────────

#[update]
async fn verify_humanity() -> Result<bool, String> {
    let user = ic_cdk::caller();
    if user == Principal::anonymous() {
        return Err("Anonymous actions are not allowed.".to_string());
    }
    
    // 1. In a future integration, we would call the DecideID canister here.
    // let decide_id_canister = Principal::from_text("...").unwrap();
    // let result = ic_cdk::call(decide_id_canister, "check_proof", (user,))...
    
    // For demonstration, we check if the user exists and upgrade them to Tier 1 (Human) 
    // if they are currently unverified.
    // TODO: Connect this to actual DecideID canister call.
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            // Only upgrade if currently None to avoid downgrading existing higher tiers
            if profile.verification_tier == VerificationTier::None {
                profile.verification_tier = VerificationTier::Human;
                map.insert(user, profile);
                Ok(true)
            } else {
                // Already Verified or Higher (e.g. KYC)
                Ok(true)
            }
        } else {
            Err("User not found".to_string())
        }
    })
}

/// Admin function to manually set KYC tier (or via Trusted Oracle)
#[update]
fn admin_set_kyc_tier(target_user: Principal, tier: VerificationTier) -> Result<(), String> {
    // Access Control: In a real app, strict checks here!
    // For now, we allow the Governance Canister or a specific admin implementation to call this.
    // let caller = ic_cdk::caller();
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&target_user) {
            profile.verification_tier = tier;
            map.insert(target_user, profile);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    })
}


// ============================================================================
// ARCHIVE INTEGRATION
// ============================================================================

/// Transaction page for paginated access with archive info
#[derive(CandidType, Deserialize, Clone, Debug)]
struct TransactionPage {
    transactions: Vec<TransactionRecord>,
    total_count: u64,
    local_count: u64,
    archived_count: u64,
    archive_canister_id: Principal,
    source: String,
}

/// Set archive canister ID (called by staking_hub during shard creation)
#[update]
fn set_archive_canister(archive_id: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    if caller != hub_id {
        return Err("Unauthorized: Only staking_hub can set archive canister".to_string());
    }
    
    ARCHIVE_CANISTER_ID.with(|id| {
        id.borrow_mut().set(archive_id).expect("Failed to set archive canister ID");
    });
    
    Ok(())
}

/// Get archive canister ID
#[query]
fn get_archive_canister() -> Principal {
    ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get())
}

/// Archive configuration info for frontends
#[derive(CandidType, Deserialize, Clone, Debug)]
struct ArchiveConfig {
    retention_limit: u64,
    trigger_threshold: u64,
    check_interval_secs: u64,
    archive_canister_id: Principal,
    is_configured: bool,
}

/// Get archive configuration
#[query]
fn get_archive_config() -> ArchiveConfig {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    ArchiveConfig {
        retention_limit: TRANSACTION_RETENTION_LIMIT,
        trigger_threshold: ARCHIVE_TRIGGER_THRESHOLD,
        check_interval_secs: ARCHIVE_CHECK_INTERVAL_SECS,
        archive_canister_id: archive_id,
        is_configured: archive_id != Principal::anonymous(),
    }
}

/// Get transactions page with pagination info
/// Returns local transactions for recent pages, indicates archive for older pages
#[query]
fn get_transactions_page(user: Principal, page: u32) -> TransactionPage {
    let page_size: u64 = 20;
    let offset = (page as u64) * page_size;
    
    let (local_count, archived_count) = USER_PROFILES.with(|p| {
        p.borrow().get(&user).map(|profile| {
            (profile.transaction_count, profile.archived_transaction_count)
        }).unwrap_or((0, 0))
    });
    
    let total_count = local_count + archived_count;
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    // Determine source and fetch data
    if offset < local_count {
        // This page is in local storage
        let transactions = USER_TRANSACTIONS.with(|t| {
            let map = t.borrow();
            let start_key = TransactionKey { user, index: offset };
            
            map.range(start_key..)
                .take_while(|(k, _)| k.user == user)
                .take(page_size as usize)
                .map(|(_, v)| v)
                .collect()
        });
        
        TransactionPage {
            transactions,
            total_count,
            local_count,
            archived_count,
            archive_canister_id: archive_id,
            source: "local".to_string(),
        }
    } else {
        // This page is in archive - return empty with archive info
        TransactionPage {
            transactions: vec![],
            total_count,
            local_count,
            archived_count,
            archive_canister_id: archive_id,
            source: "archive".to_string(),
        }
    }
}

/// Transaction data structure for archiving
#[derive(CandidType, Deserialize, Clone, Debug)]
struct TransactionToArchive {
    timestamp: u64,
    tx_type: u8,
    amount: u64,
}

/// Manually trigger archiving for testing purposes
/// Archives old transactions for all users who exceed the retention limit
#[update]
async fn trigger_archive() -> Result<u64, String> {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    if archive_id == Principal::anonymous() {
        return Err("Archive canister not configured".to_string());
    }
    
    let mut total_archived = 0u64;
    
    // Get list of users who need archiving
    let users_to_archive: Vec<(Principal, u64)> = USER_PROFILES.with(|p| {
        p.borrow().iter()
            .filter(|(_, profile)| profile.transaction_count > TRANSACTION_RETENTION_LIMIT)
            .map(|(user, profile)| (user, profile.transaction_count - TRANSACTION_RETENTION_LIMIT))
            .collect()
    });
    
    for (user, excess) in users_to_archive {
        match archive_user_transactions(user, excess, archive_id).await {
            Ok(count) => total_archived += count,
            Err(e) => ic_cdk::print(format!("Failed to archive for {}: {}", user, e)),
        }
    }
    
    Ok(total_archived)
}

/// Archive old transactions for a specific user
async fn archive_user_transactions(user: Principal, count: u64, archive_id: Principal) -> Result<u64, String> {
    // 1. Collect transactions to archive (oldest first)
    let batch: Vec<TransactionToArchive> = USER_TRANSACTIONS.with(|t| {
        let map = t.borrow();
        (0..count)
            .filter_map(|idx| {
                let key = TransactionKey { user, index: idx };
                map.get(&key).map(|txn| TransactionToArchive {
                    timestamp: txn.timestamp,
                    tx_type: match txn.tx_type {
                        TransactionType::QuizReward => 0,
                        TransactionType::Unstake => 1,
                    },
                    amount: txn.amount,
                })
            })
            .collect()
    });
    
    if batch.is_empty() {
        return Ok(0);
    }
    
    // 2. Send to archive canister
    let result: Result<(Result<u64, String>,), _> = ic_cdk::call(
        archive_id,
        "receive_archive_batch",
        (user, batch)
    ).await;
    
    match result {
        Ok((Ok(archived_count),)) => {
            // 3. SUCCESS: Delete archived entries and re-index remaining
            delete_and_reindex_transactions(user, archived_count);
            Ok(archived_count)
        }
        Ok((Err(e),)) => Err(format!("Archive rejected: {}", e)),
        Err((code, msg)) => Err(format!("Archive call failed: {:?} {}", code, msg)),
    }
}

/// Delete archived transactions and re-index remaining ones
fn delete_and_reindex_transactions(user: Principal, archived_count: u64) {
    USER_TRANSACTIONS.with(|t| {
        let mut map = t.borrow_mut();
        
        // Get current transaction count
        let current_count = USER_PROFILES.with(|p| {
            p.borrow().get(&user).map(|profile| profile.transaction_count).unwrap_or(0)
        });
        
        // Delete archived entries
        for idx in 0..archived_count {
            let key = TransactionKey { user, index: idx };
            map.remove(&key);
        }
        
        // Re-index remaining entries (shift down)
        for old_idx in archived_count..current_count {
            let old_key = TransactionKey { user, index: old_idx };
            if let Some(txn) = map.remove(&old_key) {
                let new_key = TransactionKey { user, index: old_idx - archived_count };
                map.insert(new_key, txn);
            }
        }
    });
    
    // Update user profile counts
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            profile.transaction_count -= archived_count;
            profile.archived_transaction_count += archived_count;
            map.insert(user, profile);
        }
    });
}

// ============================================================================
// ADMIN DEBUG ENDPOINTS
// ============================================================================
// These endpoints are controller-only and used for debugging authentication issues

/// Summary of a registered user for admin listing
#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserSummary {
    user_principal: Principal,
    name: String,
    email: String,
    staked_balance: u64,
    verification_tier: VerificationTier,
}

/// Result of admin user listing with pagination info
#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserListResult {
    users: Vec<UserSummary>,
    total_count: u64,
    page: u32,
    page_size: u32,
    has_more: bool,
}

/// List all registered users with pagination (ADMIN ONLY)
/// 
/// This endpoint is useful for debugging authentication issues where
/// all users seem to see the same profile.
/// 
/// # Security
/// Only canister controllers can call this function.
#[query]
fn admin_list_all_users(page: u32, page_size: u32) -> Result<UserListResult, String> {
    // Security: Only controllers can list all users
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can list all users".to_string());
    }
    
    let page_size = page_size.min(100).max(1); // Clamp to 1-100
    let offset = (page as u64) * (page_size as u64);
    
    let (users, total_count) = USER_PROFILES.with(|p| {
        let map = p.borrow();
        let total = map.len();
        
        let users: Vec<UserSummary> = map.iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .map(|(principal, profile)| UserSummary {
                user_principal: principal,
                name: profile.name.clone(),
                email: profile.email.clone(),
                staked_balance: profile.staked_balance,
                verification_tier: profile.verification_tier.clone(),
            })
            .collect();
        
        (users, total)
    });
    
    let has_more = offset + (users.len() as u64) < total_count;
    
    Ok(UserListResult {
        users,
        total_count,
        page,
        page_size,
        has_more,
    })
}

/// Get a specific user by their principal (ADMIN ONLY)
/// 
/// Returns full profile details for a specific principal.
/// Useful for verifying that different principals have different profiles.
#[query]
fn admin_get_user_details(user: Principal) -> Result<Option<UserProfile>, String> {
    // Security: Only controllers can access other users' full details
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can view user details".to_string());
    }
    
    Ok(USER_PROFILES.with(|p| p.borrow().get(&user)))
}

/// Debug info showing caller's principal (useful for frontend debugging)
/// 
/// This can be called by anyone and returns the caller's principal
/// as seen by the canister. Helps verify authentication is working.
#[query]
fn whoami() -> Principal {
    ic_cdk::caller()
}

/// Check if a principal is registered in this shard
#[query]
fn is_user_registered(user: Principal) -> bool {
    USER_PROFILES.with(|p| p.borrow().contains_key(&user))
}

ic_cdk::export_candid!();
