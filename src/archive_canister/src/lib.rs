// ============================================================================
// Archive Canister
// ============================================================================
//
// This canister provides persistent storage for archived transaction history.
// It is paired 1:1 with a user_profile shard canister.
//
// Key Features:
// - Append-only storage for data integrity
// - StableBTreeMap for efficient range queries by user
// - Authorization: only parent shard can write
// - Prepared for future archive chaining (NEXT_ARCHIVE field)
//
// Memory Layout:
//   0 - ARCHIVED_TRANSACTIONS: Main archive storage
//   1 - USER_ARCHIVE_COUNTS: Per-user entry counts
//   2 - PARENT_SHARD_ID: Parent shard reference
//   3 - TOTAL_ENTRY_COUNT: Global entry count
//   4 - NEXT_ARCHIVE: Next archive in chain (for future scaling)
// ============================================================================

use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::{init, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use candid::{CandidType, Deserialize, Principal, Encode, Decode};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum entries before archive is considered full
/// 3 billion entries ≈ 300GB (100 bytes per entry)
const MAX_ENTRIES: u64 = 3_000_000_000;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    parent_shard_id: Principal,
}

/// Key for archived transactions
/// Sorted by (user, sequence) for efficient per-user range queries
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ArchiveKey {
    user: Principal,
    sequence: u64,
}

impl Storable for ArchiveKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ArchiveKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50,
        is_fixed_size: false,
    };
}

/// Transaction data received from user_profile shard for archiving
#[derive(CandidType, Deserialize, Clone, Debug)]
struct TransactionToArchive {
    timestamp: u64,
    tx_type: u8,
    amount: u64,
}

/// Archived transaction record (includes archive metadata)
#[derive(CandidType, Deserialize, Clone, Debug)]
struct ArchivedTransaction {
    timestamp: u64,
    tx_type: u8,
    amount: u64,
    archived_at: u64,
}

impl Storable for ArchivedTransaction {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ArchivedTransaction")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50,
        is_fixed_size: false,
    };
}

/// Archive statistics for monitoring
#[derive(CandidType, Deserialize, Clone, Debug)]
struct ArchiveStats {
    total_entries: u64,
    capacity_percent: u8,
    user_count: u64,
    parent_shard: Principal,
    next_archive: Option<Principal>,
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================

thread_local! {
    /// Memory manager for allocating virtual memory regions
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Main archive storage: (user, sequence) -> transaction
    /// Uses StableBTreeMap for sorted access and persistence
    static ARCHIVED_TRANSACTIONS: RefCell<StableBTreeMap<ArchiveKey, ArchivedTransaction, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        ));

    /// User archive counts: user -> count of archived transactions
    /// Enables O(1) count lookup without iterating
    static USER_ARCHIVE_COUNTS: RefCell<StableBTreeMap<Principal, u64, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));

    /// Parent shard ID - only this canister can write to us
    static PARENT_SHARD_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Total entries in this archive (for capacity monitoring)
    static TOTAL_ENTRY_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            0u64
        ).unwrap()
    );

    /// Next archive in chain (for future extension - initially None)
    static NEXT_ARCHIVE: RefCell<StableCell<Option<Principal>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            None
        ).unwrap()
    );
}

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    PARENT_SHARD_ID.with(|id| {
        id.borrow_mut().set(args.parent_shard_id).expect("Failed to set parent shard ID")
    });
}

// ============================================================================
// WRITE OPERATIONS (Only parent shard can call)
// ============================================================================

/// Receive a batch of transactions to archive
/// 
/// This is the main entry point for archiving data.
/// Only the parent shard canister is authorized to call this.
/// 
/// # Arguments
/// * `user` - The user whose transactions are being archived
/// * `transactions` - Vector of transactions to archive
/// 
/// # Returns
/// * `Ok(count)` - Number of transactions successfully archived
/// * `Err(msg)` - Error message if archiving failed
#[update]
fn receive_archive_batch(user: Principal, transactions: Vec<TransactionToArchive>) -> Result<u64, String> {
    // Authorization: Only parent shard can write
    let caller = ic_cdk::caller();
    let parent = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    
    if caller != parent {
        return Err("Unauthorized: Only parent shard can archive".to_string());
    }
    
    // Check capacity
    let current_count = TOTAL_ENTRY_COUNT.with(|c| *c.borrow().get());
    let batch_size = transactions.len() as u64;
    
    if current_count + batch_size > MAX_ENTRIES {
        return Err(format!("Archive full: {} entries, cannot add {} more", current_count, batch_size));
    }
    
    // Get current user count and archive timestamp
    let user_count = USER_ARCHIVE_COUNTS.with(|c| c.borrow().get(&user).unwrap_or(0));
    let archived_at = ic_cdk::api::time();
    
    // Archive each transaction
    let mut archived_count = 0u64;
    
    ARCHIVED_TRANSACTIONS.with(|t| {
        let mut map = t.borrow_mut();
        
        for (i, tx) in transactions.into_iter().enumerate() {
            let key = ArchiveKey {
                user,
                sequence: user_count + i as u64,
            };
            
            let archived = ArchivedTransaction {
                timestamp: tx.timestamp,
                tx_type: tx.tx_type,
                amount: tx.amount,
                archived_at,
            };
            
            map.insert(key, archived);
            archived_count += 1;
        }
    });
    
    // Update user count
    USER_ARCHIVE_COUNTS.with(|c| {
        c.borrow_mut().insert(user, user_count + archived_count);
    });
    
    // Update total count
    TOTAL_ENTRY_COUNT.with(|c| {
        let new_total = current_count + archived_count;
        c.borrow_mut().set(new_total).expect("Failed to update total count");
    });
    
    Ok(archived_count)
}

// ============================================================================
// READ OPERATIONS (Public)
// ============================================================================

/// Get archived transactions for a user (paginated)
/// 
/// # Arguments
/// * `user` - User principal to query
/// * `offset` - Starting sequence number
/// * `limit` - Maximum number of transactions to return
/// 
/// # Returns
/// Vector of archived transactions
#[query]
fn get_archived_transactions(user: Principal, offset: u64, limit: u64) -> Vec<ArchivedTransaction> {
    ARCHIVED_TRANSACTIONS.with(|t| {
        let map = t.borrow();
        let start_key = ArchiveKey { user, sequence: offset };
        
        map.range(start_key..)
            .take_while(|(k, _)| k.user == user)
            .take(limit as usize)
            .map(|(_, v)| v)
            .collect()
    })
}

/// Get count of archived transactions for a user
#[query]
fn get_archived_count(user: Principal) -> u64 {
    USER_ARCHIVE_COUNTS.with(|c| c.borrow().get(&user).unwrap_or(0))
}

/// Get total count of all archived transactions
#[query]
fn get_total_archived_count() -> u64 {
    TOTAL_ENTRY_COUNT.with(|c| *c.borrow().get())
}

/// Get archive statistics for monitoring
#[query]
fn get_stats() -> ArchiveStats {
    let total_entries = TOTAL_ENTRY_COUNT.with(|c| *c.borrow().get());
    let capacity_percent = ((total_entries as f64 / MAX_ENTRIES as f64) * 100.0) as u8;
    let user_count = USER_ARCHIVE_COUNTS.with(|c| c.borrow().len());
    let parent_shard = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    let next_archive = NEXT_ARCHIVE.with(|n| *n.borrow().get());
    
    ArchiveStats {
        total_entries,
        capacity_percent,
        user_count,
        parent_shard,
        next_archive,
    }
}

// ============================================================================
// ADMIN OPERATIONS
// ============================================================================

/// Get parent shard ID
#[query]
fn get_parent_shard() -> Principal {
    PARENT_SHARD_ID.with(|id| *id.borrow().get())
}

/// Set next archive in chain (for future scaling)
/// Only parent shard can set this
#[update]
fn set_next_archive(next: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let parent = PARENT_SHARD_ID.with(|id| *id.borrow().get());
    
    if caller != parent {
        return Err("Unauthorized: Only parent shard can set next archive".to_string());
    }
    
    NEXT_ARCHIVE.with(|n| {
        n.borrow_mut().set(Some(next)).expect("Failed to set next archive");
    });
    
    Ok(())
}

// ============================================================================
// Candid export
// ============================================================================

ic_cdk::export_candid!();
