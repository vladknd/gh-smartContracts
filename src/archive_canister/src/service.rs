use candid::Principal;
use crate::types::*;
use crate::state::*;
use crate::constants::*;

// ============================================================================
// WRITE OPERATIONS
// ============================================================================

pub fn receive_archive_batch_internal(user: Principal, transactions: Vec<TransactionToArchive>) -> Result<u64, String> {
    let now = ic_cdk::api::time();
    let mut count = 0;
    
    ARCHIVE_STORAGE.with(|storage| {
        let mut map = storage.borrow_mut();
        
        for tx in transactions {
            let key = ArchiveKey {
                user,
                sequence: tx.sequence,
            };
            
            // Deduplication check
            if map.contains_key(&key) {
                continue;
            }
            
            let record = ArchivedTransaction {
                sequence: tx.sequence,
                timestamp: tx.timestamp,
                transaction_type: tx.transaction_type,
                amount: tx.amount,
                metadata: tx.metadata,
                archived_at: now,
            };
            
            map.insert(key, record);
            count += 1;
        }
    });
    
    // Update total count
    TOTAL_ENTRY_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let new_count = *cell.get() + count;
        cell.set(new_count).expect("Failed to update entry count");
    });
    
    Ok(count)
}

// ============================================================================
// READ OPERATIONS
// ============================================================================

pub fn get_user_history_archived_internal(user: Principal, start_sequence: Option<u64>, limit: u64) -> Vec<ArchivedTransaction> {
    let start_seq = start_sequence.unwrap_or(0);
    let max_results = limit.min(100); // Cap page size
    
    ARCHIVE_STORAGE.with(|s| {
        let map = s.borrow();
        
        // Range query is efficient here because keys are sorted by (user, sequence)
        let start_key = ArchiveKey { user, sequence: start_seq };
        let end_key = ArchiveKey { user, sequence: u64::MAX };
        
        map.range(start_key..=end_key)
            .take(max_results as usize)
            .map(|(_, v)| v)
            .collect()
    })
}

pub fn get_archive_stats_internal() -> ArchiveStats {
    let parent_shard = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    let entry_count = TOTAL_ENTRY_COUNT.with(|c| *c.borrow().get());
    let next_archive = NEXT_ARCHIVE.with(|n| *n.borrow().get());
    
    ArchiveStats {
        parent_shard,
        entry_count,
        size_bytes: entry_count * 200, // Rough estimate
        is_full: entry_count >= MAX_ENTRIES,
        next_archive,
    }
}
