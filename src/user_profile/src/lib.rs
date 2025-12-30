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
    // Economy State
    // ─────────────────────────────────────────────────────────────────
    /// Total tokens currently staked by this user
    staked_balance: u64,
    
    /// Number of transactions for this user (used for indexing)
    transaction_count: u64,
}

impl Storable for UserProfile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode UserProfile")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 600,
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

/// Daily statistics for a user (resets each day)
/// 
/// Used to enforce daily limits on quiz submissions and earnings.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserDailyStats {
    /// Day index (days since Unix epoch, used to detect day rollover)
    day_index: u64,
    /// Number of quizzes taken today
    quizzes_taken: u8,
    /// Total tokens earned today (in e8s)
    tokens_earned: u64,
}

impl Storable for UserDailyStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode UserDailyStats")
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

    // ─────────────────────────────────────────────────────────────────────
    // User Data Storage
    // ─────────────────────────────────────────────────────────────────────

    /// Map of user Principal -> UserProfile
    /// Contains all registered users and their staking state
    /// This is the primary user data store
    static USER_PROFILES: RefCell<StableBTreeMap<Principal, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Map of user Principal -> UserDailyStats
    /// Tracks daily quiz/earnings limits per user
    /// Resets when day_index changes
    static USER_DAILY_STATS: RefCell<StableBTreeMap<Principal, UserDailyStats, Memory>> = RefCell::new(
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
}

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    LEARNING_CONTENT_ID.with(|id| id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID"));

    // Start Sync Timer (Every 5 seconds for testing)
    set_timer_interval(Duration::from_secs(5), || {
        ic_cdk::spawn(async {
            let _ = sync_with_hub_internal().await;
        });
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Restart Sync Timer (Every 5 seconds for testing)
    set_timer_interval(Duration::from_secs(5), || {
        ic_cdk::spawn(async {
            let _ = sync_with_hub_internal().await;
        });
    });
}

fn get_current_day() -> u64 {
    ic_cdk::api::time() / 86_400_000_000_000
}

#[update]
async fn register_user(args: UserProfileUpdate) -> Result<(), String> {
    let user = ic_cdk::caller();
    
    if USER_PROFILES.with(|p| p.borrow().contains_key(&user)) {
        return Err("User already registered".to_string());
    }

    let new_profile = UserProfile {
        email: args.email,
        name: args.name,
        education: args.education,
        gender: args.gender,
        staked_balance: 0,
        transaction_count: 0,
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

#[update]
async fn submit_quiz(unit_id: String, answers: Vec<u8>) -> Result<u64, String> {
    let user = ic_cdk::caller();
    
    // 0. Check Registration
    if !USER_PROFILES.with(|p| p.borrow().contains_key(&user)) {
        return Err("User not registered".to_string());
    }

    let key = UserQuizKey { user, unit_id: unit_id.clone() };
    
    // 1. Check Daily Limit
    let current_day = get_current_day();
    let mut daily_stats = USER_DAILY_STATS.with(|s| {
        let s = s.borrow();
        match s.get(&user) {
            Some(stats) if stats.day_index == current_day => stats,
            _ => UserDailyStats { day_index: current_day, quizzes_taken: 0, tokens_earned: 0 }
        }
    });

    if daily_stats.quizzes_taken >= 5 {
        return Err("Daily quiz limit reached (5/5)".to_string());
    }

    // 2. Check if already completed
    if COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key)) {
        return Err("Quiz already completed".to_string());
    }

    // 3. Check Minting Allowance (Hard Cap Enforcement)
    let reward_amount = 10_000_000_000; // 100 Tokens
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

    // 4. Verify Answers (Call Learning Content)
    let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    
    let (passed, correct_count, total_questions): (bool, u64, u64) = ic_cdk::call::<(String, Vec<u8>), (bool, u64, u64)>(
        learning_content_id,
        "verify_quiz",
        (unit_id.clone(), answers)
    ).await.map_err(|(code, msg)| format!("Failed to verify quiz: {:?} {}", code, msg))?;

    if total_questions == 0 {
        return Err("Unit not found or empty quiz".to_string());
    }

    // 5. Update Stats
    daily_stats.quizzes_taken += 1;

    if !passed {
        USER_DAILY_STATS.with(|s| s.borrow_mut().insert(user, daily_stats));
        return Err(format!("Quiz failed. Score: {}/{}. Need 60% to pass.", correct_count, total_questions));
    }

    // 6. Reward & Update Balance LOCALLY
    daily_stats.tokens_earned += reward_amount;
    USER_DAILY_STATS.with(|s| s.borrow_mut().insert(user, daily_stats));

    // Update User Profile Balance
    USER_PROFILES.with(|p| {
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
            
            map.insert(user, profile);
        } else {
            // This should be unreachable if we enforce registration at the start
            // But for safety/logic consistency:
            ic_cdk::trap("User not registered (state inconsistency)");
        }
    });

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

    // 7. Mark Completed
    COMPLETED_QUIZZES.with(|q| q.borrow_mut().insert(key, true));

    Ok(reward_amount)
}

#[query]
fn get_user_daily_status(user: Principal) -> UserDailyStats {
    let current_day = get_current_day();
    USER_DAILY_STATS.with(|s| {
        let s = s.borrow();
        match s.get(&user) {
            Some(stats) if stats.day_index == current_day => stats.clone(),
            _ => UserDailyStats {
                day_index: current_day,
                quizzes_taken: 0,
                tokens_earned: 0,
            }
        }
    })
}

#[query]
fn is_quiz_completed(user: Principal, unit_id: String) -> bool {
    let key = UserQuizKey { user, unit_id };
    COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key))
}

#[update]
async fn unstake(amount: u64) -> Result<u64, String> {
    let user = ic_cdk::caller();
    
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

ic_cdk::export_candid!();
