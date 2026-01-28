use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;
use candid::Principal;
use crate::types::*;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// All persistent state is stored in stable memory using ic_stable_structures.
// Each storage item is assigned a unique MemoryId for isolation.
// 
// Memory IDs:
//   0 - LEDGER_ID: GHC ledger canister principal
//   1 - GLOBAL_STATS: Aggregate staking statistics
//   3 - REGISTERED_SHARDS: Set of authorized shard canisters
//   4 - SHARD_REGISTRY: Detailed info about each shard
//   5 - SHARD_COUNT: Number of shards created
//   6 - LEARNING_CONTENT_ID: Learning engine canister principal
//   7 - EMBEDDED_WASM: User profile WASM for auto-deployment
//   8 - INITIALIZED: Whether init() has been called
//   9 - EMBEDDED_ARCHIVE_WASM: Archive canister WASM for auto-deployment
//   10 - TOKEN_LIMITS_CONFIG: Global token limits configuration
//   11 - USER_SHARD_MAP: User to shard mapping

thread_local! {
    // ─────────────────────────────────────────────────────────────────────
    // Memory Management
    // ─────────────────────────────────────────────────────────────────────
    
    /// Memory manager for allocating virtual memory regions to each storage
    /// Allows multiple stable data structures to coexist
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // ─────────────────────────────────────────────────────────────────────
    // Configuration (Set once during init, immutable after)
    // ─────────────────────────────────────────────────────────────────────

    /// Principal ID of the GHC ICRC-1 ledger canister
    /// Used for token transfers during unstaking
    pub static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the learning_engine canister
    /// Passed to new shard canisters during auto-deployment
    pub static LEARNING_CONTENT_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Embedded user_profile WASM binary for auto-deploying new shards
    /// Stored in stable memory to survive upgrades
    /// Empty if auto-scaling is not enabled
    pub static EMBEDDED_WASM: RefCell<StableCell<WasmBlob, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            WasmBlob::default()
        ).unwrap()
    );

    /// Embedded archive_canister WASM binary for auto-deploying archive canisters
    /// Each user_profile shard gets its own archive canister
    pub static EMBEDDED_ARCHIVE_WASM: RefCell<StableCell<WasmBlob, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            WasmBlob::default()
        ).unwrap()
    );

    /// Flag indicating whether init() has been called
    /// Prevents re-initialization attacks
    pub static INITIALIZED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            false
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Global Economy State
    // ─────────────────────────────────────────────────────────────────────

    /// Aggregate staking statistics across all shards
    /// This is the source of truth for:
    /// - Total staked tokens
    /// - Total unstaked tokens
    /// - Total allocated tokens (against MAX_SUPPLY cap)
    pub static GLOBAL_STATS: RefCell<StableCell<GlobalStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            GlobalStats {
                total_staked: 0,
                total_unstaked: 0,
                total_allocated: 0,
            }
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Shard Management
    // ─────────────────────────────────────────────────────────────────────

    /// Set of registered shard canister principals: Principal -> true
    /// Only registered shards can call sync_shard and process_unstake
    /// Used for O(1) authorization checks
    pub static REGISTERED_SHARDS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    /// Detailed information about each shard: index -> ShardInfo
    /// Used for shard discovery and load balancing
    /// Index is sequential, starting at 0
    pub static SHARD_REGISTRY: RefCell<StableBTreeMap<u64, ShardInfo, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// Counter for the number of shards created
    /// Used to generate sequential shard indexes
    /// Also used to iterate over SHARD_REGISTRY
    pub static SHARD_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );
    
    // ─────────────────────────────────────────────────────────────────────
    // User Registry
    // ─────────────────────────────────────────────────────────────────────
    
    /// Map of user principal -> shard principal
    /// Updated when users register in shards
    /// Enables O(1) lookup for voting power queries
    pub static USER_SHARD_MAP: RefCell<StableBTreeMap<Principal, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11)))
        )
    );

    /// Global token limits and reward configuration 
    /// Source of truth for all shards
    pub static TOKEN_LIMITS_CONFIG: RefCell<StableCell<TokenLimitsConfig, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            TokenLimitsConfig::default()
        ).unwrap()
    );

    /// Principal ID of the subscription manager canister
    /// Broadcast to all shards to authorize subscription updates
    pub static SUBSCRIPTION_MANAGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the KYC manager canister
    /// Broadcast to all shards to authorize KYC status updates
    pub static KYC_MANAGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13))),
            Principal::anonymous()
        ).unwrap()
    );
}
