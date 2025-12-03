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
    last_reward_index: u128,
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
            PendingStats { staked_delta: 0, unstaked_delta: 0 }
        ).unwrap()
    );
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PendingStats {
    staked_delta: i64,
    unstaked_delta: u64,
}

impl Storable for PendingStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50,
        is_fixed_size: false,
    };
}

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    LEARNING_CONTENT_ID.with(|id| id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID"));
}

fn get_current_day() -> u64 {
    ic_cdk::api::time() / 86_400_000_000_000
}

#[update]
fn update_profile(profile: UserProfile) {
    let user = ic_cdk::caller();
    USER_PROFILES.with(|p| p.borrow_mut().insert(user, profile));
}

#[query]
fn get_profile(user: Principal) -> Option<UserProfile> {
    USER_PROFILES.with(|p| p.borrow().get(&user))
}

// Helper to sync with hub (Request Allowance + Report Stats)
async fn sync_with_hub_internal() -> Result<(), String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    let (staked_delta, unstaked_delta) = PENDING_STATS.with(|s| {
        let stats = s.borrow().get().clone();
        (stats.staked_delta, stats.unstaked_delta)
    });

    // Request enough for next 1000 quizzes (1000 * 1 Token)
    let requested_allowance: u64 = 1000 * 100_000_000;

    let result: Result<(Result<u64, String>,), _> = ic_cdk::call(
        staking_hub_id,
        "sync_shard",
        (staked_delta, unstaked_delta, requested_allowance)
    ).await;

    match result {
        Ok((Ok(granted),)) => {
            // Update Allowance
            MINTING_ALLOWANCE.with(|a| {
                let current = *a.borrow().get();
                a.borrow_mut().set(current + granted).expect("Failed to update allowance");
            });

            // Reset Pending Stats
            PENDING_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.staked_delta -= staked_delta;
                stats.unstaked_delta -= unstaked_delta;
                cell.set(stats).expect("Failed to update pending stats");
            });
            Ok(())
        },
        Ok((Err(msg),)) => Err(format!("Hub Rejected Sync: {}", msg)),
        Err((code, msg)) => Err(format!("Hub Call Failed: {:?} {}", code, msg))
    }
}

#[update]
async fn submit_quiz(unit_id: String, answers: Vec<u8>) -> Result<u64, String> {
    let user = ic_cdk::caller();
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
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        let mut profile = map.get(&user).unwrap_or(UserProfile {
            email: "".to_string(),
            name: "".to_string(),
            education: "".to_string(),
            gender: "".to_string(),
            staked_balance: 0,
            last_reward_index: 0,
        });
        
        profile.staked_balance += reward_amount;
        map.insert(user, profile);
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
    
    // 1. Check Balance
    let mut profile = USER_PROFILES.with(|p| p.borrow().get(&user).unwrap_or(UserProfile {
        email: "".to_string(),
        name: "".to_string(),
        education: "".to_string(),
        gender: "".to_string(),
        staked_balance: 0,
        last_reward_index: 0,
    }));

    if profile.staked_balance < amount {
        return Err(format!("Insufficient balance. Available: {}", profile.staked_balance));
    }

    // 2. Update Local State (Optimistic Update)
    profile.staked_balance -= amount;
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

#[update]
async fn debug_force_sync() -> Result<(), String> {
    sync_with_hub_internal().await
}

ic_cdk::export_candid!();
