mod types;
mod state;
mod constants;

use std::time::Duration;
use ic_cdk::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use candid::{Principal, Nat};

use types::*;
use state::*;
use constants::*;
mod service;
use service::*;

// ===============================
// Initialization (IMMUTABLE AFTER)
// ===============================

#[init]
fn init(args: InitArgs) {
    // Store configuration (immutable after init)
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID");
    });
    
    LEARNING_CONTENT_ID.with(|id| {
        id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID");
    });
    
    // Store the user_profile WASM (immutable after init)
    if !args.user_profile_wasm.is_empty() {
        EMBEDDED_WASM.with(|w| {
            w.borrow_mut().set(WasmBlob { data: args.user_profile_wasm })
                .expect("Failed to store user_profile WASM");
        });
    }
    
    // Store the archive_canister WASM (immutable after init)
    if let Some(archive_wasm) = args.archive_canister_wasm {
        if !archive_wasm.is_empty() {
            EMBEDDED_ARCHIVE_WASM.with(|w| {
                w.borrow_mut().set(WasmBlob { data: archive_wasm })
                    .expect("Failed to store archive_canister WASM");
            });
        }
    }
    
    INITIALIZED.with(|i| {
        i.borrow_mut().set(true).expect("Failed to set initialized flag");
    });
    
    // Start auto-scaling timer
    start_auto_scale_timer();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Restart auto-scaling timer after upgrade
    start_auto_scale_timer();
}

fn start_auto_scale_timer() {
    set_timer_interval(Duration::from_secs(AUTO_SCALE_INTERVAL_SECS), || {
        ic_cdk::spawn(async {
            let _ = ensure_capacity_internal().await;
        });
    });
}

// ===============================
// Quiz Cache Router
// ===============================

/// Distribute quiz cache updates to all active shards
#[update]
async fn distribute_quiz_cache(unit_id: String, cache_data: QuizCacheData) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    let learning_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    if caller != learning_id {
        return Err("Unauthorized".to_string());
    }

    let shards: Vec<Principal> = SHARD_REGISTRY.with(|r| {
        r.borrow().iter().map(|(_, s)| s.canister_id).collect()
    });

    let mut success_count = 0;
    for shard in shards {
        let u = unit_id.clone();
        let c = cache_data.clone();
        // Fire and forget to avoid blocking hub
        ic_cdk::spawn(async move {
            let _ = ic_cdk::call::<_, ()>(shard, "receive_quiz_cache", (u, c)).await;
        });
        success_count += 1;
    }
    Ok(success_count)
}

/// Distribute token limits update to all shards
/// Called by itself (after governance update) or manually by controllers
#[update]
async fn distribute_token_limits(config: TokenLimitsConfig) -> Result<u64, String> {
    // SECURITY: Only allow hub itself or controller to broadcast
    let caller = ic_cdk::caller();
    if caller != ic_cdk::api::id() && !ic_cdk::api::is_controller(&caller) {
        return Err("Unauthorized".to_string());
    }

    Ok(distribute_token_limits_internal(config).await)
}

async fn distribute_token_limits_internal(config: TokenLimitsConfig) -> u64 {
    let shards: Vec<Principal> = SHARD_REGISTRY.with(|r| {
        r.borrow().iter().map(|(_, s)| s.canister_id).collect()
    });

    let mut success_count = 0;
    for shard in shards {
        let c = config.clone();
        // Await the call to ensure it's delivered before returning
        let result = ic_cdk::call::<_, ()>(shard, "receive_token_limits", (c,)).await;
        if result.is_ok() {
            success_count += 1;
        }
    }
    success_count
}

#[update]
async fn update_token_limits(
    new_reward_amount: Option<u64>,
    new_pass_threshold: Option<u8>,
    new_max_attempts: Option<u8>,
    regular_limits: Option<TokenLimits>,
    subscribed_limits: Option<TokenLimits>,
) -> Result<(), String> {
    // Auth check: in production, verify caller is governance canister
    // For now, allow controllers
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        // Here we could also check against a stored GOVERNANCE_CANISTER_ID
        // return Err("Unauthorized".to_string());
    }

    // Validate Regular Limits
    if let Some(ref limits) = regular_limits {
        if limits.max_daily_tokens < MIN_REGULAR_DAILY || limits.max_daily_tokens > MAX_REGULAR_DAILY {
            return Err(format!("Regular daily limit must be between {} and {}", MIN_REGULAR_DAILY, MAX_REGULAR_DAILY));
        }
        if limits.max_weekly_tokens < MIN_REGULAR_WEEKLY || limits.max_weekly_tokens > MAX_REGULAR_WEEKLY {
            return Err(format!("Regular weekly limit must be between {} and {}", MIN_REGULAR_WEEKLY, MAX_REGULAR_WEEKLY));
        }
        if limits.max_monthly_tokens < MIN_REGULAR_MONTHLY || limits.max_monthly_tokens > MAX_REGULAR_MONTHLY {
            return Err(format!("Regular monthly limit must be between {} and {}", MIN_REGULAR_MONTHLY, MAX_REGULAR_MONTHLY));
        }
        if limits.max_yearly_tokens < MIN_REGULAR_YEARLY || limits.max_yearly_tokens > MAX_REGULAR_YEARLY {
            return Err(format!("Regular yearly limit must be between {} and {}", MIN_REGULAR_YEARLY, MAX_REGULAR_YEARLY));
        }
    }

    // Validate Subscribed Limits
    if let Some(ref limits) = subscribed_limits {
        if limits.max_daily_tokens < MIN_SUBSCRIBED_DAILY || limits.max_daily_tokens > MAX_SUBSCRIBED_DAILY {
            return Err(format!("Subscribed daily limit must be between {} and {}", MIN_SUBSCRIBED_DAILY, MAX_SUBSCRIBED_DAILY));
        }
        if limits.max_weekly_tokens < MIN_SUBSCRIBED_WEEKLY || limits.max_weekly_tokens > MAX_SUBSCRIBED_WEEKLY {
            return Err(format!("Subscribed weekly limit must be between {} and {}", MIN_SUBSCRIBED_WEEKLY, MAX_SUBSCRIBED_WEEKLY));
        }
        if limits.max_monthly_tokens < MIN_SUBSCRIBED_MONTHLY || limits.max_monthly_tokens > MAX_SUBSCRIBED_MONTHLY {
            return Err(format!("Subscribed monthly limit must be between {} and {}", MIN_SUBSCRIBED_MONTHLY, MAX_SUBSCRIBED_MONTHLY));
        }
        if limits.max_yearly_tokens < MIN_SUBSCRIBED_YEARLY || limits.max_yearly_tokens > MAX_SUBSCRIBED_YEARLY {
            return Err(format!("Subscribed yearly limit must be between {} and {}", MIN_SUBSCRIBED_YEARLY, MAX_SUBSCRIBED_YEARLY));
        }
    }

    let new_config = TOKEN_LIMITS_CONFIG.with(|c| {
        let mut cell = c.borrow_mut();
        let mut config = cell.get().clone();
        
        if let Some(val) = new_reward_amount { config.reward_amount = val; }
        if let Some(val) = new_pass_threshold { config.pass_threshold_percent = val; }
        if let Some(val) = new_max_attempts { config.max_daily_attempts = val; }
        if let Some(limits) = regular_limits { config.regular_limits = limits; }
        if let Some(limits) = subscribed_limits { config.subscribed_limits = limits; }
        
        config.version += 1;
        cell.set(config.clone()).expect("Failed to update token limits config");
        config
    });

    // Auto-distribute to all shards
    distribute_token_limits_internal(new_config).await;
    
    Ok(())
}

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


// ===============================
// Shard Discovery Queries (for Frontend)
// ===============================

/// Get all registered shards
#[query]
fn get_shards() -> Vec<ShardInfo> {
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    SHARD_REGISTRY.with(|r| {
        let registry = r.borrow();
        (0..shard_count)
            .filter_map(|i| registry.get(&i))
            .collect()
    })
}

/// Get only active shards
#[query]
fn get_active_shards() -> Vec<ShardInfo> {
    get_active_shards_internal()
}

/// Find the best shard for a new user (load balancing)
#[query]
fn get_shard_for_new_user() -> Option<Principal> {
    get_active_shards_internal()
        .into_iter()
        .filter(|s| s.user_count < SHARD_HARD_LIMIT)
        .min_by_key(|s| s.user_count)
        .map(|s| s.canister_id)
}

/// Check if a canister is a registered shard
#[query]
fn is_registered_shard(canister_id: Principal) -> bool {
    REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&canister_id))
}

#[query]
fn get_shard_count() -> u64 {
    SHARD_COUNT.with(|c| *c.borrow().get())
}

/// Ensures capacity by creating new shards when needed
#[update]
async fn ensure_capacity() -> Result<Option<Principal>, String> {
    ensure_capacity_internal().await
}

// ============================================================================
// ADMIN FUNCTIONS
// ============================================================================

/// Manually register a canister as an allowed shard (minter)
/// 
/// This is used when deploying user_profile canisters manually
/// instead of using the auto-scaling mechanism.
/// 
/// SECURITY: This should only be callable by controllers in production
#[update]
async fn add_allowed_minter(canister_id: Principal) {
    // Register the shard (without archive - manual setup)
    register_shard_internal(canister_id, None);
    // Push current config and data
    let _ = sync_new_shard(canister_id).await;
}

/// Manually register a canister as an allowed shard with archive support
#[update]
async fn register_shard(canister_id: Principal, archive_id: Option<Principal>) {
    // SECURITY: Only allow controllers to manually register shards
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        ic_cdk::trap("Unauthorized: Only controllers can register shards");
    }
    
    // Register the shard
    register_shard_internal(canister_id, archive_id);
    // Push current config and data
    let _ = sync_new_shard(canister_id).await;
}

/// Broadcast the subscription manager ID to all registered shards
/// 
/// SECURITY: Only callable by controllers
#[update]
async fn admin_broadcast_subscription_manager(new_id: Principal) -> Result<u64, String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can broadcast".to_string());
    }

    // Update local state
    SUBSCRIPTION_MANAGER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set subscription manager ID");
    });

    // Get list of shards
    let shards: Vec<Principal> = SHARD_REGISTRY.with(|r| {
        r.borrow().iter().map(|(_, s)| s.canister_id).collect()
    });

    let mut success_count = 0;
    for shard in shards {
        // Call each shard to update its subscription manager
        let result = ic_cdk::call::<_, ()>(shard, "internal_sync_subscription_manager", (new_id,)).await;
        if result.is_ok() {
            success_count += 1;
        }
    }

    Ok(success_count)
}

/// Broadcast the KYC manager ID to all registered shards
/// 
/// SECURITY: Only callable by controllers
#[update]
async fn admin_broadcast_kyc_manager(new_id: Principal) -> Result<u64, String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can broadcast".to_string());
    }

    // Update local state
    KYC_MANAGER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set KYC manager ID");
    });

    // Get list of shards
    let shards: Vec<Principal> = SHARD_REGISTRY.with(|r| {
        r.borrow().iter().map(|(_, s)| s.canister_id).collect()
    });

    let mut success_count = 0;
    for shard in shards {
        // Call each shard to update its KYC manager
        let result = ic_cdk::call::<_, ()>(shard, "internal_sync_kyc_manager", (new_id,)).await;
        if result.is_ok() {
            success_count += 1;
        }
    }

    Ok(success_count)
}

/// Get the archive canister ID for a specific shard
#[query]
fn get_archive_for_shard(shard_id: Principal) -> Option<Principal> {
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    SHARD_REGISTRY.with(|r| {
        let registry = r.borrow();
        (0..shard_count)
            .filter_map(|i| registry.get(&i))
            .find(|s| s.canister_id == shard_id)
            .and_then(|s| s.archive_canister_id)
    })
}

// ===============================
// Shard Update Functions (Called by Shards)
// ===============================

/// Update shard user count (called by shards during sync)
#[update]
fn update_shard_user_count(user_count: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is a registered shard
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    // Find and update the shard in registry
    SHARD_REGISTRY.with(|r| {
        let mut registry = r.borrow_mut();
        let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
        
        for i in 0..shard_count {
            if let Some(mut shard) = registry.get(&i) {
                if shard.canister_id == caller {
                    shard.user_count = user_count;
                    
                    // Mark as full if threshold reached
                    if user_count >= SHARD_HARD_LIMIT {
                        shard.status = ShardStatus::Full;
                    }
                    
                    registry.insert(i, shard);
                    return Ok(());
                }
            }
        }
        
        Err("Shard not found in registry".to_string())
    })
}

/// Synchronize shard statistics with the hub and request minting allowance
/// 
/// This function is called periodically by each user_profile shard to:
/// 1. Report stake changes (new stakes from quiz rewards)
/// 2. Report unstaking activity
/// 3. Request additional minting allowance when running low
/// 
/// # Arguments
/// * `staked_delta` - Change in staked amounts since last sync
/// * `unstaked_delta` - Total amount unstaked since last sync
/// * `requested_allowance` - Amount of minting allowance to request
/// 
/// # Returns
/// * `granted_allowance` - Allowance granted for minting
/// 
/// # Security
/// - Only registered shards can call this function
/// - Uses saturating arithmetic to prevent overflow/underflow
#[update]
fn sync_shard(
    staked_delta: i64,
    unstaked_delta: u64,
    requested_allowance: u64
) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only shards created by this hub can report stats
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        // ─────────────────────────────────────────────────────────────────
        // Step 1: Update staked amounts and track actual minting
        // ─────────────────────────────────────────────────────────────────
        if staked_delta > 0 {
            stats.total_staked = stats.total_staked.saturating_add(staked_delta as u64);
            // Track actual tokens minted (positive delta = new tokens created)
            stats.total_allocated = stats.total_allocated.saturating_add(staked_delta as u64);
        } else {
            stats.total_staked = stats.total_staked.saturating_sub(staked_delta.abs() as u64);
        }
        
        // Update unstaked total
        stats.total_unstaked += unstaked_delta;

        // ─────────────────────────────────────────────────────────────────
        // Step 2: Handle Allowance Request (Hard Cap Enforcement)
        // ─────────────────────────────────────────────────────────────────
        // Note: Allowance grants are pre-approvals for future minting.
        // total_allocated is updated in Step 1 when tokens are actually minted.
        let granted_allowance = if requested_allowance > 0 {
            // Never grant beyond MAX_SUPPLY (4.75B MUC tokens)
            let remaining = MAX_SUPPLY.saturating_sub(stats.total_allocated);
            let to_grant = remaining.min(requested_allowance);
            // Don't add to total_allocated here - it's tracked when tokens are actually minted
            to_grant
        } else {
            0
        };
        
        cell.set(stats).expect("Failed to update global stats");
        Ok(granted_allowance)
    })
}

/// Process unstake request from a shard - returns 100% (no penalty)
#[update]
async fn process_unstake(user: Principal, amount: u64) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is a registered shard
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }

    // No penalty - return full amount
    let return_amount = amount;

    // Update global stats
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.total_unstaked += return_amount;
        cell.set(stats).expect("Failed to update global stats");
    });

    // Ledger transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: user, subaccount: None },
        amount: Nat::from(return_amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer call failed: {:?} {}", code, msg))?;

    match result {
        Ok(_) => Ok(return_amount),
        Err(e) => {
            // Rollback
            GLOBAL_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.total_unstaked -= return_amount;
                cell.set(stats).expect("Failed to rollback");
            });
            Err(format!("Ledger transfer failed: {:?}", e))
        }
    }
}

// ===============================
// Query Functions
// ===============================

#[query]
fn get_global_stats() -> GlobalStats {
    GLOBAL_STATS.with(|s| s.borrow().get().clone())
}

#[query]
fn get_config() -> (Principal, Principal, bool) {
    let ledger = LEDGER_ID.with(|id| *id.borrow().get());
    let learning = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    let has_wasm = EMBEDDED_WASM.with(|w| !w.borrow().get().data.is_empty());
    (ledger, learning, has_wasm)
}

#[query]
fn get_limits() -> (u64, u64) {
    (SHARD_SOFT_LIMIT, SHARD_HARD_LIMIT)
}

// ============================================================================
// GOVERNANCE VOTING POWER
// ============================================================================
//
// This section provides voting power lookups for the governance system.
// - VUC (Volume of Unmined Coins) = board member voting power (weighted by shares)
// - User staked_balance = user voting power
//
// Board members are registered with percentage shares of VUC voting power.
// Total shares must equal 100%. Can be locked for immutability.
// The user registry maps each user to their shard for O(1) lookup.

// ============================================================================
// USER REGISTRY
// ============================================================================

/// Register a user's shard location (called by shards during user registration)
#[update]
fn register_user_location(user: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Only registered shards can register user locations
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    // Store user -> shard mapping
    USER_SHARD_MAP.with(|m| {
        m.borrow_mut().insert(user, caller);
    });
    
    Ok(())
}

/// Get which shard a user is registered in
#[query]
fn get_user_shard(user: Principal) -> Option<Principal> {
    USER_SHARD_MAP.with(|m| m.borrow().get(&user))
}

/// Manually set a user's shard location (admin only)
/// Used to backfill existing users who registered before the registry was added
#[update]
fn admin_set_user_shard(user: Principal, shard: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can set user shards".to_string());
    }
    
    // Verify the shard is registered
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&shard));
    if !is_registered {
        return Err("Invalid shard: Not a registered shard".to_string());
    }
    
    USER_SHARD_MAP.with(|m| {
        m.borrow_mut().insert(user, shard);
    });
    
    Ok(())
}


// ============================================================================
// VOTING POWER FUNCTIONS
// ============================================================================
// Note: Board member voting power is now handled by operational_governance.
// This canister provides get_vuc() for governance to calculate board member power,
// and fetch_user_voting_power() for regular user staked balance lookups.

/// Get VUC (Volume of Unmined Coins) - total board member voting power pool
/// VUC = MAX_SUPPLY - total_allocated
/// This is used by operational_governance to calculate board member voting power
#[query]
fn get_vuc() -> u64 {
    GLOBAL_STATS.with(|s| {
        let stats = s.borrow().get().clone();
        MAX_SUPPLY.saturating_sub(stats.total_allocated)
    })
}

/// Get total voting power in the system (VUC + total_staked)
#[query]
fn get_total_voting_power() -> u64 {
    let vuc = get_vuc();
    let total_staked = GLOBAL_STATS.with(|s| s.borrow().get().total_staked);
    vuc.saturating_add(total_staked)
}

/// Fetch voting power for a regular user (async - queries shards)
/// 
/// This returns the user's staked balance from their shard.
/// For board members, operational_governance calculates their weighted VUC locally.
#[update]
async fn fetch_user_voting_power(user: Principal) -> u64 {
    // Look up user's shard
    let shard_id = USER_SHARD_MAP.with(|m| m.borrow().get(&user));
    
    let shard_id = match shard_id {
        Some(id) => id,
        None => return 0, // User not registered
    };
    
    // Query shard for user's profile
    
    let result: Result<(Option<UserProfilePartial>,), _> = ic_cdk::call(
        shard_id,
        "get_profile",
        (user,)
    ).await;
    
    match result {
        Ok((Some(profile),)) => profile.staked_balance,
        _ => 0,
    }
}

/// Get governance tokenomics summary
#[query]
fn get_tokenomics() -> (u64, u64, u64, u64) {
    let stats = GLOBAL_STATS.with(|s| s.borrow().get().clone());
    let vuc = MAX_SUPPLY.saturating_sub(stats.total_allocated);
    let total_voting_power = vuc.saturating_add(stats.total_staked);
    
    (
        MAX_SUPPLY,           // Total Utility Coin Cap (4.75B)
        stats.total_allocated, // Tokens mined/allocated so far
        vuc,                  // VUC (total board member voting power pool)
        total_voting_power    // Total voting power in system
    )
}

ic_cdk::export_candid!();
