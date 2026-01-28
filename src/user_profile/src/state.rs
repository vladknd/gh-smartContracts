use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;
use candid::Principal;
use crate::types::*;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    /// Memory manager for formulating virtual memory regions to each storage
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // ─────────────────────────────────────────────────────────────────────
    // Configuration (Set once during init, immutable after)
    // ─────────────────────────────────────────────────────────────────────

    /// Principal ID of the staking_hub canister
    pub static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the learning_engine canister
    pub static LEARNING_CONTENT_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );
    
    // ─────────────────────────────────────────────────────────────────────
    // User Data
    // ─────────────────────────────────────────────────────────────────────

    /// Map of user Principal -> UserProfile
    pub static USER_PROFILES: RefCell<StableBTreeMap<Principal, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Map of user Principal -> UserTimeStats
    pub static USER_TIME_STATS: RefCell<StableBTreeMap<Principal, UserTimeStats, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    /// Set of completed quiz keys: (user, unit_id) -> true
    pub static COMPLETED_QUIZZES: RefCell<StableBTreeMap<UserQuizKey, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// Map of TransactionKey -> TransactionRecord
    pub static USER_TRANSACTIONS: RefCell<StableBTreeMap<TransactionKey, TransactionRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );

    // ─────────────────────────────────────────────────────────────────────
    // Economy State
    // ─────────────────────────────────────────────────────────────────────

    /// Minting allowance granted by the staking hub
    pub static MINTING_ALLOWANCE: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );

    /// Pending statistics to report to the hub on next sync
    pub static PENDING_STATS: RefCell<StableCell<PendingStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            PendingStats { staked_delta: 0, unstaked_delta: 0 }
        ).unwrap()
    );

    /// Quiz Cache: unit_id -> QuizCacheData
    pub static QUIZ_CACHE: RefCell<StableBTreeMap<String, QuizCacheData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );

    /// Archive canister ID
    pub static ARCHIVE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            Principal::anonymous()
        ).unwrap()
    );
    
    /// Cached global token limits configuration
    pub static TOKEN_LIMITS_CONFIG: RefCell<StableCell<TokenLimitsConfig, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            TokenLimitsConfig::default()
        ).unwrap()
    );

    /// Principal ID of the subscription manager canister
    /// Only this canister is allowed to flip the is_subscribed flag
    pub static SUBSCRIPTION_MANAGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the KYC manager canister
    /// Only this canister is allowed to update kyc_status
    pub static KYC_MANAGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12))),
            Principal::anonymous()
        ).unwrap()
    );
}
