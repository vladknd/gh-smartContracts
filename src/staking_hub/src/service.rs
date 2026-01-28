use candid::{Principal, Encode};
use crate::types::*;
use crate::state::*;
use crate::constants::*;

// ===============================
// Auto-Scaling Functions
// ===============================

pub async fn ensure_capacity_internal() -> Result<Option<Principal>, String> {
    // 1. Check if WASM is embedded
    let has_wasm = EMBEDDED_WASM.with(|w| !w.borrow().get().data.is_empty());
    if !has_wasm {
        return Err("No WASM embedded - cannot auto-create shards".to_string());
    }
    
    // 2. Get current shards
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    // 3. If no shards exist, create the first one
    if shard_count == 0 {
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    // 4. Check if all active shards are near capacity
    let active_shards = get_active_shards_internal();
    
    if active_shards.is_empty() {
        // All shards are full, need a new one
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    // Check if the best shard is approaching soft limit
    let min_user_count = active_shards.iter().map(|s| s.user_count).min().unwrap_or(0);
    
    if min_user_count >= SHARD_SOFT_LIMIT {
        // Proactively create a new shard
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    Ok(None) // No new shard needed
}

pub async fn create_shard_internal() -> Result<Principal, String> {

    // 1. Get embedded WASMs
    let user_profile_wasm = EMBEDDED_WASM.with(|w| w.borrow().get().data.clone());
    let archive_wasm = EMBEDDED_ARCHIVE_WASM.with(|w| w.borrow().get().data.clone());
    
    if user_profile_wasm.is_empty() {
        return Err("No user_profile WASM embedded".to_string());
    }
    
    // 2. Get required IDs
    let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    let staking_hub_id = ic_cdk::id();
    
    // 4. Create user_profile canister
    let create_args = CreateCanisterArgs {
        settings: Some(CanisterSettings {
            controllers: Some(vec![staking_hub_id]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };
    
    let (user_profile_result,): (CreateCanisterResult,) = ic_cdk::api::call::call_with_payment128(
        Principal::management_canister(),
        "create_canister",
        (create_args,),
        1_000_000_000_000 // 1T cycles
    ).await.map_err(|(code, msg)| format!("Failed to create user_profile canister: {:?} {}", code, msg))?;
    
    let user_profile_id = user_profile_result.canister_id;
    
    // 5. Create archive canister (if WASM is available)
    let archive_canister_id: Option<Principal> = if !archive_wasm.is_empty() {
        let archive_create_args = CreateCanisterArgs {
            settings: Some(CanisterSettings {
                controllers: Some(vec![staking_hub_id]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
            }),
        };
        
        let (archive_result,): (CreateCanisterResult,) = ic_cdk::api::call::call_with_payment128(
            Principal::management_canister(),
            "create_canister",
            (archive_create_args,),
            1_000_000_000_000 // 1T cycles
        ).await.map_err(|(code, msg)| format!("Failed to create archive canister: {:?} {}", code, msg))?;
        
        let archive_id = archive_result.canister_id;
        
        // Install archive canister code
        let archive_init = ArchiveInitArgs {
            parent_shard_id: user_profile_id,
        };
        
        let archive_install_args = InstallCodeArgs {
            mode: InstallMode::install,
            canister_id: archive_id,
            wasm_module: archive_wasm,
            arg: Encode!(&archive_init).map_err(|e| format!("Failed to encode archive init args: {}", e))?,
        };
        
        let _: () = ic_cdk::call(
            Principal::management_canister(),
            "install_code",
            (archive_install_args,)
        ).await.map_err(|(code, msg)| format!("Failed to install archive code: {:?} {}", code, msg))?;
        
        Some(archive_id)
    } else {
        None
    };
    
    // 6. Install user_profile code
    let user_profile_init = UserProfileInitArgs {
        staking_hub_id,
        learning_content_id,
    };
    
    let user_profile_install_args = InstallCodeArgs {
        mode: InstallMode::install,
        canister_id: user_profile_id,
        wasm_module: user_profile_wasm,
        arg: Encode!(&user_profile_init).map_err(|e| format!("Failed to encode user_profile init args: {}", e))?,
    };
    
    let _: () = ic_cdk::call(
        Principal::management_canister(),
        "install_code",
        (user_profile_install_args,)
    ).await.map_err(|(code, msg)| format!("Failed to install user_profile code: {:?} {}", code, msg))?;
    
    // 7. Link archive canister to user_profile (if archive was created)
    if let Some(archive_id) = archive_canister_id {
        let _: Result<(), _> = ic_cdk::call(
            user_profile_id,
            "set_archive_canister",
            (archive_id,)
        ).await;
        // Note: We ignore errors here as the shard is still functional without the link
    }
    
    // 8. Register the new shard with archive info
    register_shard_internal(user_profile_id, archive_canister_id);

    // 9. Sync initial quiz cache (async, ignore error)
    ic_cdk::spawn(async move {
        let _ = sync_new_shard(user_profile_id).await;
    });
    
    Ok(user_profile_id)
}

pub fn register_shard_internal(canister_id: Principal, archive_canister_id: Option<Principal>) {
    // Check if already registered
    if REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&canister_id)) {
        // If already registered but has no archive ID, update it
        if let Some(archive_id) = archive_canister_id {
            SHARD_REGISTRY.with(|r| {
                let mut registry = r.borrow_mut();
                let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
                for i in 0..shard_count {
                    if let Some(mut info) = registry.get(&i) {
                        if info.canister_id == canister_id && info.archive_canister_id.is_none() {
                            info.archive_canister_id = Some(archive_id);
                            registry.insert(i, info);
                            break;
                        }
                    }
                }
            });
        }
        return;
    }

    let shard_index = SHARD_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let idx = *cell.get();
        cell.set(idx + 1).expect("Failed to increment shard count");
        idx
    });
    
    let shard_info = ShardInfo {
        canister_id,
        created_at: ic_cdk::api::time(),
        user_count: 0,
        status: ShardStatus::Active,
        archive_canister_id,
    };
    
    REGISTERED_SHARDS.with(|m| m.borrow_mut().insert(canister_id, true));
    SHARD_REGISTRY.with(|r| r.borrow_mut().insert(shard_index, shard_info));
}

pub fn get_active_shards_internal() -> Vec<ShardInfo> {
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    SHARD_REGISTRY.with(|r| {
        let registry = r.borrow();
        (0..shard_count)
            .filter_map(|i| registry.get(&i))
            .filter(|s| s.status == ShardStatus::Active)
            .collect()
    })
}

// Helper to sync a newly created shard with full quiz cache AND config
pub async fn sync_new_shard(shard_id: Principal) -> Result<(), String> {
    let learning_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    
    // 1. Fetch all quiz question hashes from learning engine
    let (all_caches,): (Vec<(String, QuizCacheData)>,) = ic_cdk::call(
        learning_id,
        "get_all_quiz_cache_data",
        ()
    ).await.map_err(|(c, m)| format!("Sync failed: {:?} {}", c, m))?;

    // 2. Push questions to new shard
    let _ = ic_cdk::call::<_, ()>(
        shard_id,
        "receive_full_quiz_cache",
        (all_caches,)
    ).await;
    
    // 3. Push token limits from local state
    let config = TOKEN_LIMITS_CONFIG.with(|c| c.borrow().get().clone());
    
    let _ = ic_cdk::call::<_, ()>(
        shard_id,
        "receive_token_limits",
        (config,)
    ).await;
    
    Ok(())
}
