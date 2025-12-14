use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use std::borrow::Cow;
use candid::{Encode, Decode};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize)]
struct InitArgs {
    staking_hub_id: Principal,
    learning_content_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserProfile {
    email: String,
    name: String,
    education: String,
    gender: String,
    // Economy State
    staked_balance: u64,
    unclaimed_interest: u64, // New field for manual claim
    last_reward_index: u128,
    transaction_count: u64,
}

impl Storable for UserProfile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 500, // Reasonable limit for profile data
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserProfileUpdate {
    email: String,
    name: String,
    education: String,
    gender: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
enum TransactionType {
    QuizReward,
    Unstake,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct TransactionRecord {
    timestamp: u64,
    tx_type: TransactionType,
    amount: u64,
}

impl Storable for TransactionRecord {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

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
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserDailyStats {
    day_index: u64,
    quizzes_taken: u8,
    tokens_earned: u64,
}

impl Storable for UserDailyStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

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
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

// V1 for migration
#[derive(CandidType, Deserialize, Clone, Debug)]
struct PendingStatsV1 {
    staked_delta: i64,
    unstaked_delta: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PendingStats {
    staked_delta: i64,
    unstaked_delta: u64,
    rewards_delta: u64, // New field
}

impl Storable for PendingStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        if let Ok(stats) = Decode!(bytes.as_ref(), Self) {
            return stats;
        }
        if let Ok(v1) = Decode!(bytes.as_ref(), PendingStatsV1) {
            return Self {
                staked_delta: v1.staked_delta,
                unstaked_delta: v1.unstaked_delta,
                rewards_delta: 0,
            };
        }
        panic!("Failed to decode PendingStats");
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

// Thread-local static variables for stable structures
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    static LEARNING_CONTENT_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );

    static USER_PROFILES: RefCell<StableBTreeMap<Principal, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static USER_DAILY_STATS: RefCell<StableBTreeMap<Principal, UserDailyStats, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    static COMPLETED_QUIZZES: RefCell<StableBTreeMap<UserQuizKey, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    // New: Shard State for Allowance and Batching
    static MINTING_ALLOWANCE: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );

    static PENDING_STATS: RefCell<StableCell<PendingStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            PendingStats { staked_delta: 0, unstaked_delta: 0, rewards_delta: 0 }
        ).unwrap()
    );

    static USER_TRANSACTIONS: RefCell<StableBTreeMap<TransactionKey, TransactionRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );

    static GLOBAL_REWARD_INDEX: RefCell<StableCell<u128, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            0
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
fn register_user(args: UserProfileUpdate) -> Result<(), String> {
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
        unclaimed_interest: 0,
        last_reward_index: 0,
        transaction_count: 0,
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

#[query]
fn get_profile(user: Principal) -> Option<UserProfile> {
    USER_PROFILES.with(|p| {
        if let Some(profile) = p.borrow().get(&user) {
            // Calculate pending interest for display
            let global_index = GLOBAL_REWARD_INDEX.with(|i| *i.borrow().get());
            let mut display_profile = profile.clone();
            
            if global_index > profile.last_reward_index {
                let index_diff = global_index - profile.last_reward_index;
                let interest = (profile.staked_balance as u128 * index_diff) / 1_000_000_000_000_000_000;
                display_profile.unclaimed_interest += interest as u64;
            }
            Some(display_profile)
        } else {
            None
        }
    })
}

// Helper to compound interest for a user (Updates Unclaimed Interest)
fn compound_interest(user: Principal) {
    let global_index = GLOBAL_REWARD_INDEX.with(|i| *i.borrow().get());
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            if global_index > profile.last_reward_index {
                let index_diff = global_index - profile.last_reward_index;
                let interest = (profile.staked_balance as u128 * index_diff) / 1_000_000_000_000_000_000;
                
                if interest > 0 {
                    profile.unclaimed_interest += interest as u64;
                }
                profile.last_reward_index = global_index;
                map.insert(user, profile);
            } else if profile.last_reward_index == 0 {
                // Initialize index for new users/first time
                profile.last_reward_index = global_index;
                map.insert(user, profile);
            }
        }
    });
}

// Helper to sync with hub (Request Allowance + Report Stats)
async fn sync_with_hub_internal() -> Result<(), String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // 1. Optimistic Reset (Atomic Read-and-Clear)
    let (staked_delta, unstaked_delta, rewards_delta) = PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        let s_delta = stats.staked_delta;
        let u_delta = stats.unstaked_delta;
        let r_delta = stats.rewards_delta;
        
        // Reset to 0 (we are taking these values to report)
        stats.staked_delta = 0;
        stats.unstaked_delta = 0;
        stats.rewards_delta = 0;
        
        cell.set(stats).expect("Failed to update pending stats");
        
        (s_delta, u_delta, r_delta)
    });

    // Smart Allowance Request
    let current_allowance = MINTING_ALLOWANCE.with(|a| *a.borrow().get());
    let low_threshold = 500 * 100_000_000; // 500 Tokens
    let refill_amount = 1000 * 100_000_000; // 1000 Tokens

    let requested_allowance: u64 = if current_allowance < low_threshold {
        refill_amount
    } else {
        0
    };

    let result: Result<(Result<(u64, u128), String>,), _> = ic_cdk::call(
        staking_hub_id,
        "sync_shard",
        (staked_delta, unstaked_delta, rewards_delta, requested_allowance)
    ).await;

    match result {
        Ok((Ok((granted, global_index)),)) => {
            // Success! Just update allowance.
            MINTING_ALLOWANCE.with(|a| {
                let current = *a.borrow().get();
                a.borrow_mut().set(current + granted).expect("Failed to update allowance");
            });

            // Update Global Index
            GLOBAL_REWARD_INDEX.with(|i| {
                i.borrow_mut().set(global_index).expect("Failed to update global index");
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
            // Hub Logic Error: Rollback Stats
            PENDING_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.staked_delta += staked_delta;
                stats.unstaked_delta += unstaked_delta;
                stats.rewards_delta += rewards_delta;
                cell.set(stats).expect("Failed to rollback pending stats");
            });
            Err(format!("Hub Rejected Sync: {}", msg))
        },
        Err((code, msg)) => {
            // Network/System Error: Rollback Stats
            PENDING_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.staked_delta += staked_delta;
                stats.unstaked_delta += unstaked_delta;
                stats.rewards_delta += rewards_delta;
                cell.set(stats).expect("Failed to rollback pending stats");
            });
            Err(format!("Hub Call Failed: {:?} {}", code, msg))
        }
    }
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
        // We expect the user to exist because we checked at the start (or should have)
        // But wait, we didn't check at the start of submit_quiz yet.
        // Let's rely on the check we are about to add at the top of submit_quiz.
        if let Some(mut profile) = map.get(&user) {
            profile.staked_balance += reward_amount;
            
            // Log Transaction
            let tx_index = profile.transaction_count;
            profile.transaction_count += 1;
            
            let tx_record = TransactionRecord {
                timestamp: ic_cdk::api::time(),
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

