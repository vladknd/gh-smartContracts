use ic_cdk::{init, query, update, post_upgrade};
use candid::Principal;

mod types;
mod state;
mod constants;
mod service;

use types::*;
use state::*;
use constants::*;
use service::*;


// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    LEARNING_CONTENT_ID.with(|id| id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID"));
}

#[post_upgrade]
fn post_upgrade() {
    // Migration logic if needed
}

// ============================================================================
// UPDATE OPERATIONS
// ============================================================================

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
        is_subscribed: false,
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

/// Receive token limits update from staking_hub
#[update]
fn receive_token_limits(config: TokenLimitsConfig) {
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    if ic_cdk::caller() != hub_id {
         ic_cdk::trap("Unauthorized config update");
    }
    
    TOKEN_LIMITS_CONFIG.with(|c| {
        c.borrow_mut().set(config).expect("Failed to set token limits config");
    });
}

/// Get the locally cached token limits
#[query]
fn get_token_limits() -> TokenLimitsConfig {
    TOKEN_LIMITS_CONFIG.with(|c| c.borrow().get().clone())
}

#[query]
fn get_subscription_manager_id() -> Principal {
    SUBSCRIPTION_MANAGER_ID.with(|id| *id.borrow().get())
}

#[query]
fn get_kyc_manager_id() -> Principal {
    KYC_MANAGER_ID.with(|id| *id.borrow().get())
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
    let config = TOKEN_LIMITS_CONFIG.with(|c| c.borrow().get().clone());
    
    // We expect the config to be synced from the Hub during shard creation or via broadcast.
    // If version is 0, it means we haven't received the config yet.
    if config.version == 0 {
        return Err("Configuration not yet initialized from Staking Hub".to_string());
    }

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
    let reward_amount = config.reward_amount;
    
    if stats.daily_quizzes >= config.max_daily_attempts {
        return Err("Daily quiz limit reached".to_string());
    }

    let is_subscribed = USER_PROFILES.with(|p| {
        p.borrow().get(&user).map(|profile| profile.is_subscribed).unwrap_or(false)
    });

    let limits = if is_subscribed {
        &config.subscribed_limits
    } else {
        &config.regular_limits
    };
    
    if stats.daily_earnings + reward_amount > limits.max_daily_tokens {
        return Err(format!("Daily token limit reached ({}/{})", stats.daily_earnings, limits.max_daily_tokens));
    }
    if stats.weekly_earnings + reward_amount > limits.max_weekly_tokens {
        return Err(format!("Weekly token limit reached ({}/{})", stats.weekly_earnings, limits.max_weekly_tokens));
    }
    if stats.monthly_earnings + reward_amount > limits.max_monthly_tokens {
         return Err(format!("Monthly token limit reached ({}/{})", stats.monthly_earnings, limits.max_monthly_tokens));
    }
    if stats.yearly_earnings + reward_amount > limits.max_yearly_tokens {
         return Err(format!("Yearly token limit reached ({}/{})", stats.yearly_earnings, limits.max_yearly_tokens));
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
            Err((_code, _msg)) => {
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

#[query]
fn get_current_day() -> u64 {
    service::get_current_day()
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

/// Synchronize the subscription manager ID from the Staking Hub
/// 
/// SECURITY: Strictly only callable by the Staking Hub
#[update]
fn internal_sync_subscription_manager(new_id: Principal) {
    let caller = ic_cdk::caller();
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    if caller != hub_id {
        ic_cdk::trap("Unauthorized: Only Staking Hub can sync subscription manager");
    }
    
    SUBSCRIPTION_MANAGER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set subscription manager ID");
    });
}

/// Set subscription status for a user
/// 
/// SECURITY: Strictly only callable by the authorized Subscription Manager canister
#[update]
fn internal_set_subscription(user: Principal, active: bool) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let manager_id = SUBSCRIPTION_MANAGER_ID.with(|id| *id.borrow().get());
    
    if caller != manager_id {
        return Err("Unauthorized: Caller is not the authorized Subscription Manager".to_string());
    }
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            profile.is_subscribed = active;
            map.insert(user, profile);
            Ok(())
        } else {
            Err("User not found in this shard".to_string())
        }
    })
}

/// Internal: Update the trusted KYC manager ID
/// Called by the Staking Hub when the manager principal changes
#[update]
fn internal_sync_kyc_manager(new_id: Principal) {
    if ic_cdk::caller() != STAKING_HUB_ID.with(|id| *id.borrow().get()) {
        ic_cdk::trap("Unauthorized: Only Staking Hub can sync KYC manager");
    }
    
    KYC_MANAGER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to sync KYC manager ID");
    });
}

/// Internal: Set a user's KYC status
/// Strictly restricted to the authorized KYC manager canister
#[update]
fn internal_set_kyc_status(user: Principal, tier: VerificationTier) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let manager_id = KYC_MANAGER_ID.with(|id| *id.borrow().get());
    
    if caller != manager_id {
        return Err(format!("Unauthorized: Caller {} is not the authorized KYC manager {}", caller, manager_id));
    }
    
    USER_PROFILES.with(|p| {
        let mut profiles = p.borrow_mut();
        let mut profile = profiles.get(&user).ok_or("User not found")?;
        profile.verification_tier = tier;
        profiles.insert(user, profile);
        Ok(())
    })
}


// ============================================================================
// ARCHIVE INTEGRATION
// ============================================================================


/// Set archive canister ID (called by staking_hub during shard creation)
#[update]
fn set_archive_canister(archive_id: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    if caller != hub_id && !ic_cdk::api::is_controller(&caller) {
        return Err("Unauthorized: Only staking_hub or a controller can set archive canister".to_string());
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


/// Get archive configuration
#[query]
fn get_archive_config() -> ArchiveConfig {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    ArchiveConfig {
        retention_limit: TRANSACTION_RETENTION_LIMIT,
        trigger_threshold: ARCHIVE_TRIGGER_THRESHOLD,
        check_interval_secs: ARCHIVE_CHECK_INTERVAL_SECS,
        archive_canister_id: Some(archive_id),
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
            archive_canister_id: Some(archive_id),
            source: "local".to_string(),
            current_page: page,
            total_pages: 1, 
            has_archive_data: false,
        }
    } else {
        // This page is in archive - return empty with archive info
        TransactionPage {
            transactions: vec![],
            total_count,
            local_count,
            archived_count,
            archive_canister_id: Some(archive_id),
            source: "archive".to_string(),
            current_page: page,
            total_pages: 1,
            has_archive_data: true,
        }
    }
}

/// Transaction data structure for archiving

/// Manually trigger archiving for testing purposes
/// Archives old transactions for all users who exceed the retention limit
#[update]
async fn debug_trigger_archive() -> Result<u64, String> {
    // Security: only controllers
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    run_periodic_archive().await
}

// ============================================================================
// ADMIN DEBUG ENDPOINTS
// ============================================================================
// These endpoints are controller-only and used for debugging authentication issues


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
                is_subscribed: profile.is_subscribed,
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
        total_pages: (total_count as f64 / page_size as f64).ceil() as u32,
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

/// Set user time stats for testing resets (ADMIN ONLY)
#[update]
fn admin_set_user_stats(user: Principal, stats: UserTimeStats) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    USER_TIME_STATS.with(|s| {
        s.borrow_mut().insert(user, stats);
    });
    Ok(())
}

/// Manually set subscription status for a user (ADMIN ONLY - Controllers)
/// 
/// Used for testing purposes or manual corrections by developers.
#[update]
fn admin_set_subscription(user: Principal, active: bool) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can manually set subscriptions".to_string());
    }
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            profile.is_subscribed = active;
            map.insert(user, profile);
            Ok(())
        } else {
            Err("User not found in this shard".to_string())
        }
    })
}

/// Admin: Set a user's KYC status (manual override)
#[update]
fn admin_set_kyc_status(user: Principal, tier: VerificationTier) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    USER_PROFILES.with(|p| {
        let mut profiles = p.borrow_mut();
        let mut profile = profiles.get(&user).ok_or("User not found")?;
        profile.verification_tier = tier;
        profiles.insert(user, profile);
        Ok(())
    })
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
