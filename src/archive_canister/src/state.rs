use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;
use candid::Principal;
use crate::types::*;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    /// Memory manager for allocating virtual memory regions
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Main archive storage (Stable BTree)
    /// Key: ArchiveKey (user + sequence)
    /// Value: ArchivedTransaction
    pub static ARCHIVE_STORAGE: RefCell<StableBTreeMap<ArchiveKey, ArchivedTransaction, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    /// Configuration: Parent shard ID (immutable after init)
    pub static PARENT_SHARD_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Total entries in this archive (for capacity monitoring)
    pub static TOTAL_ENTRY_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            0u64
        ).unwrap()
    );

    /// Pointer to the next archive canister (if this one is full)
    /// Used for linked-list style traversal if needed
    pub static NEXT_ARCHIVE: RefCell<StableCell<Option<Principal>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            None
        ).unwrap()
    );
}
