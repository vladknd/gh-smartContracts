mod types;
mod state;
mod constants;
mod service;

use ic_cdk::{init, query, update};
use candid::Principal;

use types::*;
use state::*;
use service::*;

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    PARENT_SHARD_ID.with(|id| {
        id.borrow_mut().set(args.parent_shard_id).expect("Failed to set parent shard ID");
    });
}

// ============================================================================
// WRITE OPERATIONS (Only parent shard can call)
// ============================================================================

#[update]
fn receive_archive_batch(user: Principal, transactions: Vec<TransactionToArchive>) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    let parent_id = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    
    // Authorization check
    if caller != parent_id {
        return Err("Unauthorized: Only parent shard can archive data".to_string());
    }
    
    receive_archive_batch_internal(user, transactions)
}

#[update]
fn set_next_archive(next_id: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let parent_id = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    
    // Authorization check (only parent shard or controller)
    if caller != parent_id && !ic_cdk::api::is_controller(&caller) {
        return Err("Unauthorized".to_string());
    }
    
    NEXT_ARCHIVE.with(|n| {
        n.borrow_mut().set(Some(next_id)).expect("Failed to set next archive");
    });
    
    Ok(())
}

// ============================================================================
// READ OPERATIONS (Queries)
// ============================================================================

#[query]
fn get_archived_transactions(user: Principal, start_sequence: Option<u64>, limit: u64) -> Vec<ArchivedTransaction> {
    get_user_history_archived_internal(user, start_sequence, limit)
}

#[query]
fn get_stats() -> ArchiveStats {
    get_archive_stats_internal()
}

#[query]
fn get_parent_shard() -> Principal {
    PARENT_SHARD_ID.with(|id| *id.borrow().get())
}

#[query]
fn get_total_archived_count() -> u64 {
    TOTAL_ENTRY_COUNT.with(|c| *c.borrow().get())
}

#[query]
fn get_archived_count(user: Principal) -> u64 {
    ARCHIVE_STORAGE.with(|s| {
        let map = s.borrow();
        let start_key = ArchiveKey { user, sequence: 0 };
        let end_key = ArchiveKey { user, sequence: u64::MAX };
        map.range(start_key..=end_key).count() as u64
    })
}

ic_cdk::export_candid!();
