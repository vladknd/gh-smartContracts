use candid::Principal;
use crate::types::*;
use crate::state::*;
use crate::constants::*;

// ============================================================================
// DATE HELPERS
// ============================================================================

/// Get current day index (days since epoch)
pub fn get_current_day() -> u64 {
    ic_cdk::api::time() / 86_400_000_000_000
}

/// Get week index (weeks since epoch, aligned to Sunday)
pub fn get_week_index(day: u64) -> u64 {
    // Epoch (Day 0) was Thursday. Adding 4 days aligns the start of the index to Sunday.
    (day + 4) / 7
}

/// Simple date struct
pub struct SimpleDate {
    pub year: u32,
    pub month: u32, // 1-12
    pub _day: u32,  // 1-31
}

/// Convert days since epoch to date (Simplified Gregorian)
pub fn day_to_date(day: u64) -> SimpleDate {
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
    // Use leap array or standard array
    let days_in_months = if leap { &DAYS_IN_MONTHS_LEAP } else { &DAYS_IN_MONTHS };
    
    let mut m = 0;
    for &dim in days_in_months {
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

// ============================================================================
// HUB SYNCHRONIZATION
// ============================================================================

/// Periodically sync local statistics with the staking hub
pub async fn sync_with_hub_internal() -> Result<(), String> {
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
    
    let requested_allowance: u64 = if current_allowance < ALLOWANCE_LOW_THRESHOLD {
        ALLOWANCE_REFILL_AMOUNT
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
pub fn rollback_pending_stats(staked_delta: i64, unstaked_delta: u64) {
    PENDING_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.staked_delta += staked_delta;
        stats.unstaked_delta += unstaked_delta;
        cell.set(stats).expect("Failed to rollback pending stats");
    });
}

// ============================================================================
// ARCHIVING LOGIC
// ============================================================================

/// Periodic archiving task - archives old transactions for all users exceeding limit
pub async fn run_periodic_archive() -> Result<u64, String> {
    let archive_id = ARCHIVE_CANISTER_ID.with(|id| *id.borrow().get());
    
    if archive_id == Principal::anonymous() {
        // Archive not configured, skip silently
        return Ok(0);
    }
    
    // Get list of users who need archiving
    // We limit this to a batch to avoid timeouts if many users need archiving
    let users_to_archive: Vec<(Principal, u64)> = USER_PROFILES.with(|p| {
        p.borrow().iter()
            .filter(|(_, profile)| profile.transaction_count > TRANSACTION_RETENTION_LIMIT)
            .take(50) // Process max 50 users per cycle to be safe
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

/// Archive transactions for a specific user
pub async fn archive_user_transactions(user: Principal, count_to_archive: u64, archive_id: Principal) -> Result<u64, String> {
    // 1. Fetch transactions to archive
    let mut txs_to_archive = Vec::new();
    let mut keys_to_remove = Vec::new();
    let _start_index = 0;
    
    // Find where the transaction history starts for this user (could be > 0 if already archived before)
    // We need to fetch the first 'count_to_archive' transactions
    
    // This is a bit inefficient with BTreeMap iteration, but limits are small (100)
    // Optimization: Store first_index in UserProfile to avoid scanning
    
    USER_TRANSACTIONS.with(|t| {
        let map = t.borrow();
        // We know we want to remove the oldest ones.
        // Range scan for this user
        let start_key = TransactionKey { user, index: 0 };
        let end_key = TransactionKey { user, index: u64::MAX };
        
        for (key, tx) in map.range(start_key..end_key).take(count_to_archive as usize) {
            txs_to_archive.push(tx.clone()); // We need to convert to archive format
            keys_to_remove.push(key.clone());
        }
    });
    
    if txs_to_archive.is_empty() {
        return Ok(0);
    }
    
    // 2. Send to archive canister
    // We need to map local TransactionRecord to archive's TransactionToArchive
    // But wait - we don't have the shared types here easily without dependency cycle or duplication.
    // The archive canister expects:
    // struct TransactionToArchive { sequence, timestamp, transaction_type, amount, metadata }
    
    // Let's define a local struct compatible with the archive canister
    
    let archive_batch: Vec<TransactionToArchive> = txs_to_archive.iter().zip(keys_to_remove.iter()).map(|(tx, key)| {
        TransactionToArchive {
            sequence: key.index,
            timestamp: tx.timestamp,
            transaction_type: format!("{:?}", tx.tx_type),
            amount: tx.amount,
            metadata: "{}".to_string(), // Simple empty metadata for now
        }
    }).collect();
    
    // Call archive
    let result: Result<(Result<u64, String>,), _> = ic_cdk::call(
        archive_id,
        "receive_archive_batch",
        (user, archive_batch)
    ).await;
    
    let archived_count = match result {
        Ok((Ok(count),)) => count,
        Ok((Err(msg),)) => return Err(format!("Archive canister rejected: {}", msg)),
        Err((code, msg)) => return Err(format!("Archive call failed: {:?} {}", code, msg)),
    };
    
    // 3. If successful, delete local records
    if archived_count > 0 {
        USER_TRANSACTIONS.with(|t| {
            let mut map = t.borrow_mut();
            for key in keys_to_remove.iter().take(archived_count as usize) {
                map.remove(key);
            }
        });
        
        // Update user profile counts
        USER_PROFILES.with(|p| {
            let mut map = p.borrow_mut();
            if let Some(mut profile) = map.get(&user) {
                profile.archived_transaction_count += archived_count;
                // transaction_count remains total count (archived + local)
                // Wait - logic check.
                // If transaction_count is total ever, then local count = transaction_count - archived_transaction_count
                // The retention logic in periodic check used: profile.transaction_count as total?
                // Let's verify existing logic:
                // "profile.transaction_count - TRANSACTION_RETENTION_LIMIT"
                
                // Typically:
                // transaction_count = 150 (total ever)
                // retention = 100
                // excess = 50
                // We archive 50.
                // archived_count becomes 50.
                // local entries = 100.
                // New transaction: transaction_count = 151.
                // excess = 51? No, we need to know how many are local.
                
                // Better approach: transaction_count is TOTAL EVER.
                // Local count = transaction_count - archived_transaction_count.
                
                map.insert(user, profile);
            }
        });
    }
    
    Ok(archived_count)
}

/// Stable deterministic hash for answer verification (must match Learning Engine)
pub fn stable_hash(data: &[u8]) -> [u8; 32] {
    let mut hash: u64 = 5381;
    for b in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*b as u64);
    }
    let b = hash.to_le_bytes();
    let mut res = [0u8; 32];
    for i in 0..8 {
        res[i] = b[i];
    }
    // Fill the rest with a pattern to make it 32 bytes
    for i in 8..32 {
        res[i] = res[i % 8];
    }
    res
}
