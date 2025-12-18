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
// TIER SYSTEM CONSTANTS
// ============================================================================
// These must match the constants in staking_hub

/// Number of tiers in the system
pub const NUM_TIERS: usize = 4;

/// Tier thresholds based on continuous staking duration (in nanoseconds)
/// 
/// Users automatically progress through tiers as they stake longer:
/// - Bronze (Tier 0): 0+ days - New stakers
/// - Silver (Tier 1): 30+ days - Committed stakers
/// - Gold (Tier 2): 90+ days - Long-term stakers  
/// - Diamond (Tier 3): 365+ days - Diamond hands
pub const TIER_THRESHOLDS_NANOS: [u64; 4] = [
    0,                                      // Bronze: 0+ days
    30 * 24 * 60 * 60 * 1_000_000_000,      // Silver: 30+ days
    90 * 24 * 60 * 60 * 1_000_000_000,      // Gold: 90+ days
    365 * 24 * 60 * 60 * 1_000_000_000,     // Diamond: 365+ days
];

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
/// The economy state tracks the user's staked tokens, unclaimed interest,
/// and their current tier in the staking system. Tier is determined by
/// how long they've been continuously staking (initial_stake_time).
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
    
    /// Interest that has been calculated but not yet claimed/compounded
    unclaimed_interest: u64,
    
    /// Legacy reward index (kept for compatibility, not used in tier system)
    last_reward_index: u128,
    
    /// Number of transactions for this user (used for indexing)
    transaction_count: u64,
    
    // ─────────────────────────────────────────────────────────────────
    // Tier Tracking
    // ─────────────────────────────────────────────────────────────────
    /// Current tier: 0=Bronze, 1=Silver, 2=Gold, 3=Diamond
    current_tier: u8,
    
    /// Reward index of user's current tier when they entered it
    /// Used to calculate interest: (current_index - tier_start_index) * stake
    tier_start_index: u128,
    
    /// Timestamp (nanoseconds) when user first staked
    /// Tier is calculated as: now - initial_stake_time
    initial_stake_time: u64,
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
    
    /// Total amount unstaked since last sync (before penalty)
    unstaked_delta: u64,
    
    /// Total rewards distributed to users since last sync
    rewards_delta: u64,
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
//   8 - GLOBAL_REWARD_INDEX: Interest calculation (legacy)
// 
// Non-stable (heap) storage:
//   TIER_REWARD_INDEXES: Refreshed from hub on each sync
//   PENDING_TIER_DELTAS: Accumulated tier changes for next sync

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
    /// Accumulates stake changes, unstakes, and reward distributions
    /// Cleared after successful sync, rolled back on failure
    static PENDING_STATS: RefCell<StableCell<PendingStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            PendingStats { staked_delta: 0, unstaked_delta: 0, rewards_delta: 0 }
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Interest Calculation State
    // ─────────────────────────────────────────────────────────────────────

    /// Legacy global reward index (kept for backwards compatibility)
    /// New code uses TIER_REWARD_INDEXES instead
    static GLOBAL_REWARD_INDEX: RefCell<StableCell<u128, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            0
        ).unwrap()
    );

    /// Current reward index for each tier: [Bronze, Silver, Gold, Diamond]
    /// Synced from the staking hub on each sync cycle
    /// Used to calculate unclaimed interest: (current_index - user.tier_start_index) * stake
    /// NOT stored in stable memory - refreshed from hub on every sync
    static TIER_REWARD_INDEXES: RefCell<[u128; 4]> = RefCell::new([0; 4]);
    
    /// Pending tier deltas to report to hub: [Bronze, Silver, Gold, Diamond]
    /// Accumulated when users move between tiers
    /// Sent to hub during sync, then cleared
    /// NOT stored in stable memory - volatile between syncs
    static PENDING_TIER_DELTAS: RefCell<[i64; 4]> = RefCell::new([0; 4]);
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
fn register_user(args: UserProfileUpdate) -> Result<(), String> {
    let user = ic_cdk::caller();
    
    if USER_PROFILES.with(|p| p.borrow().contains_key(&user)) {
        return Err("User already registered".to_string());
    }

    // Get current tier indexes to set tier_start_index
    let bronze_index = TIER_REWARD_INDEXES.with(|i| i.borrow()[0]);

    let new_profile = UserProfile {
        email: args.email,
        name: args.name,
        education: args.education,
        gender: args.gender,
        staked_balance: 0,
        unclaimed_interest: 0,
        last_reward_index: 0,
        transaction_count: 0,
        // New tier fields
        current_tier: 0,              // Start in Bronze
        tier_start_index: bronze_index, // Will receive interest from current index
        initial_stake_time: 0,        // Set on first stake
    };

    USER_PROFILES.with(|p| p.borrow_mut().insert(user, new_profile));
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

// ===============================
// Tier Helper Functions
// ===============================

/// Calculate which tier a user should be in based on staking duration
fn get_tier_for_duration(duration_nanos: u64) -> u8 {
    for tier in (0..NUM_TIERS).rev() {
        if duration_nanos >= TIER_THRESHOLDS_NANOS[tier] {
            return tier as u8;
        }
    }
    0 // Default to Bronze
}

/// Get user's expected tier based on their staking duration
fn get_expected_tier(profile: &UserProfile, now: u64) -> u8 {
    if profile.initial_stake_time == 0 || profile.staked_balance == 0 {
        return 0; // Bronze for new users or zero balance
    }
    let duration = now.saturating_sub(profile.initial_stake_time);
    get_tier_for_duration(duration)
}

#[query]
fn get_profile(user: Principal) -> Option<UserProfile> {
    let now = ic_cdk::api::time();
    
    USER_PROFILES.with(|p| {
        if let Some(profile) = p.borrow().get(&user) {
            let mut display_profile = profile.clone();
            
            // Calculate pending interest using tier indexes
            let tier_indexes = TIER_REWARD_INDEXES.with(|i| i.borrow().clone());
            let current_tier = profile.current_tier as usize;
            let current_index = tier_indexes[current_tier];
            
            if current_index > profile.tier_start_index && profile.staked_balance > 0 {
                let index_diff = current_index - profile.tier_start_index;
                let interest = (profile.staked_balance as u128 * index_diff) / 1_000_000_000_000_000_000;
                display_profile.unclaimed_interest = display_profile.unclaimed_interest.saturating_add(interest as u64);
            }
            
            // Update display tier based on expected tier (for display only, actual upgrade happens on compound)
            let expected_tier = get_expected_tier(&profile, now);
            display_profile.current_tier = expected_tier;
            
            Some(display_profile)
        } else {
            None
        }
    })
}

/// Check if user should be upgraded and handle the upgrade
fn check_and_handle_tier_upgrade(profile: &mut UserProfile, now: u64) -> bool {
    let expected_tier = get_expected_tier(profile, now);
    
    if expected_tier > profile.current_tier {
        let tier_indexes = TIER_REWARD_INDEXES.with(|i| i.borrow().clone());
        let old_tier = profile.current_tier as usize;
        let old_index = tier_indexes[old_tier];
        
        // Claim interest from old tier before upgrading
        if old_index > profile.tier_start_index && profile.staked_balance > 0 {
            let index_diff = old_index - profile.tier_start_index;
            let interest = (profile.staked_balance as u128 * index_diff) / 1_000_000_000_000_000_000;
            profile.unclaimed_interest = profile.unclaimed_interest.saturating_add(interest as u64);
        }
        
        // Update pending tier deltas for sync
        let balance = profile.staked_balance as i64;
        PENDING_TIER_DELTAS.with(|d| {
            let mut deltas = d.borrow_mut();
            deltas[old_tier] -= balance;           // Remove from old tier
            deltas[expected_tier as usize] += balance; // Add to new tier
        });
        
        // Move to new tier
        let new_tier = expected_tier as usize;
        profile.current_tier = expected_tier;
        profile.tier_start_index = tier_indexes[new_tier];
        
        return true;
    }
    
    false
}

// Helper to compound interest for a user (Updates Unclaimed Interest)
fn compound_interest(user: Principal) {
    let now = ic_cdk::api::time();
    let tier_indexes = TIER_REWARD_INDEXES.with(|i| i.borrow().clone());
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            // 1. Check for and handle tier upgrade
            let upgraded = check_and_handle_tier_upgrade(&mut profile, now);
            
            // 2. Calculate interest in current tier
            let current_tier = profile.current_tier as usize;
            let current_index = tier_indexes[current_tier];
            
            if current_index > profile.tier_start_index && profile.staked_balance > 0 {
                let index_diff = current_index - profile.tier_start_index;
                let interest = (profile.staked_balance as u128 * index_diff) / 1_000_000_000_000_000_000;
                
                if interest > 0 {
                    profile.unclaimed_interest = profile.unclaimed_interest.saturating_add(interest as u64);
                }
                profile.tier_start_index = current_index;
            } else if profile.tier_start_index == 0 && profile.staked_balance > 0 {
                // Initialize index for first time
                profile.tier_start_index = current_index;
            }
            
            // Always save if we did any work
            if upgraded || profile.tier_start_index == current_index {
                map.insert(user, profile);
            }
        }
    });
}

// ============================================================================
// HUB SYNCHRONIZATION
// ============================================================================

/// Periodically sync local statistics with the staking hub
/// 
/// This function runs every 5 seconds and performs these operations:
/// 1. Reports tier-specific stake changes to the hub
/// 2. Reports unstaking and reward distribution
/// 3. Requests more minting allowance if running low
/// 4. Receives updated tier reward indexes for interest calculation
/// 
/// Uses optimistic reset pattern: values are cleared before the call
/// and rolled back if the call fails.
async fn sync_with_hub_internal() -> Result<(), String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // ─────────────────────────────────────────────────────────────────
    // Step 1: Optimistic Reset - Capture and clear pending stats
    // ─────────────────────────────────────────────────────────────────
    let (staked_delta, unstaked_delta, rewards_delta) = PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        let s_delta = stats.staked_delta;
        let u_delta = stats.unstaked_delta;
        let r_delta = stats.rewards_delta;
        
        // Clear for next cycle
        stats.staked_delta = 0;
        stats.unstaked_delta = 0;
        stats.rewards_delta = 0;
        
        cell.set(stats).expect("Failed to update pending stats");
        
        (s_delta, u_delta, r_delta)
    });

    // Capture tier deltas (from tier upgrades)
    let tier_deltas = PENDING_TIER_DELTAS.with(|d| {
        let mut deltas = d.borrow_mut();
        let captured: [i64; 4] = *deltas;
        *deltas = [0; 4];
        captured
    });

    // Combine regular staked_delta with tier 0 (Bronze) for new stakes
    let mut combined_tier_deltas = tier_deltas;
    combined_tier_deltas[0] += staked_delta;

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
    // Step 3: Call hub with tier-aware sync
    // ─────────────────────────────────────────────────────────────────
    let result: Result<(Result<(u64, [u128; 4]), String>,), _> = ic_cdk::call(
        staking_hub_id,
        "sync_shard",
        (combined_tier_deltas, unstaked_delta, rewards_delta, requested_allowance)
    ).await;

    match result {
        Ok((Ok((granted, tier_indexes)),)) => {
            // Success! Update local state with hub data
            
            // Update minting allowance
            MINTING_ALLOWANCE.with(|a| {
                let current = *a.borrow().get();
                a.borrow_mut().set(current + granted).expect("Failed to update allowance");
            });

            // Update tier reward indexes (for interest calculation)
            TIER_REWARD_INDEXES.with(|i| {
                *i.borrow_mut() = tier_indexes;
            });

            // Update legacy global index (for backwards compatibility)
            let avg_index: u128 = tier_indexes.iter().sum::<u128>() / 4;
            GLOBAL_REWARD_INDEX.with(|i| {
                i.borrow_mut().set(avg_index).expect("Failed to update global index");
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
            rollback_pending_stats(staked_delta, unstaked_delta, rewards_delta, tier_deltas);
            Err(format!("Hub Rejected Sync: {}", msg))
        },
        Err((code, msg)) => {
            // Network/system error - rollback local stats
            rollback_pending_stats(staked_delta, unstaked_delta, rewards_delta, tier_deltas);
            Err(format!("Hub Call Failed: {:?} {}", code, msg))
        }
    }
}

/// Rollback pending stats after a failed sync attempt
fn rollback_pending_stats(staked_delta: i64, unstaked_delta: u64, rewards_delta: u64, tier_deltas: [i64; 4]) {
    PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.staked_delta += staked_delta;
        stats.unstaked_delta += unstaked_delta;
        stats.rewards_delta += rewards_delta;
        cell.set(stats).expect("Failed to rollback pending stats");
    });
    
    PENDING_TIER_DELTAS.with(|d| {
        let mut deltas = d.borrow_mut();
        for i in 0..NUM_TIERS {
            deltas[i] += tier_deltas[i];
        }
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
    let reward_amount = 100_000_000; // 1 Token
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
    compound_interest(user); // Apply interest first!
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            // Set initial_stake_time on first stake (for tier calculation)
            let now = ic_cdk::api::time();
            if profile.initial_stake_time == 0 {
                profile.initial_stake_time = now;
                // Initialize tier_start_index if needed
                if profile.tier_start_index == 0 {
                    let bronze_index = TIER_REWARD_INDEXES.with(|i| i.borrow()[0]);
                    profile.tier_start_index = bronze_index;
                }
            }
            
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
        stats.rewards_delta += reward_amount;
        cell.set(stats).expect("Failed to update pending stats");
    });

    // Trigger Refill if Low (Async/Fire-and-forget ideally, but here we just check)
    // For simplicity, we rely on the check at the start of the next call.
    // Or we could spawn a timer.

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
    // Apply Interest First!
    compound_interest(user);
    
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
        amount: amount,
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

    // 4. Call Hub to Process Unstake (Transfer Tokens)
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

#[update]
fn claim_rewards() -> Result<u64, String> {
    let user = ic_cdk::caller();
    
    // 1. Calculate latest interest
    compound_interest(user);
    
    // 2. Move Unclaimed -> Staked
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            let amount = profile.unclaimed_interest;
            
            if amount == 0 {
                return Err("No rewards to claim".to_string());
            }
            
            profile.staked_balance += amount;
            profile.unclaimed_interest = 0;
            
            // Log Transaction
            let tx_index = profile.transaction_count;
            profile.transaction_count += 1;
            
            let tx_record = TransactionRecord {
                timestamp: ic_cdk::api::time(),
                tx_type: TransactionType::QuizReward, // Using QuizReward as placeholder for now
                amount: amount,
            };
            
            USER_TRANSACTIONS.with(|t| t.borrow_mut().insert(TransactionKey { user, index: tx_index }, tx_record));
            
            map.insert(user, profile);
            Ok(amount)
        } else {
            Err("User not registered".to_string())
        }
    })
}

/// Get total number of registered users in this shard
#[query]
fn get_user_count() -> u64 {
    USER_PROFILES.with(|p| p.borrow().len())
}

ic_cdk::export_candid!();

