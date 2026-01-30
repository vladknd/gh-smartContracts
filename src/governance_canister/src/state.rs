use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;
use candid::Principal;
use crate::types::*;
use crate::constants::*;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // Configuration
    pub static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), Principal::anonymous()).unwrap()
    );
    
    pub static TREASURY_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), Principal::anonymous()).unwrap()
    );

    // Proposals
    pub static PROPOSALS: RefCell<StableBTreeMap<u64, Proposal, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );
    
    pub static PROPOSAL_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))), 0).unwrap()
    );
    
    // Vote records
    pub static VOTE_RECORDS: RefCell<StableBTreeMap<VoteKey, VoteRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );

    // Support records
    pub static SUPPORT_RECORDS: RefCell<StableBTreeMap<VoteKey, SupportRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );
    
    // Board Member Management
    // Board members exercise VUC (Volume of Unmined Coins) voting power
    // Regular members have a BPS share of the total VUC
    // The sentinel member has exactly 1 unit of VUC
    
    /// Board member voting power shares: Principal -> share in basis points (0-10000)
    /// Each board member gets (share_bps / 10000) * VUC voting power
    /// Total of all shares must equal exactly 10,000 BPS (100%)
    /// NOTE: The sentinel is NOT stored here - they are stored in SENTINEL_MEMBER
    pub static BOARD_MEMBER_SHARES: RefCell<StableBTreeMap<Principal, u16, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))))
    );
    
    /// The Sentinel Member: A special board member with exactly 1 unit of VUC voting power
    /// This member exists to satisfy the "requires board member vote" rule without 
    /// significantly affecting voting outcomes. They are NOT included in BPS calculations.
    pub static SENTINEL_MEMBER: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15))),
            Principal::anonymous()
        ).unwrap()
    );
    
    /// Lock flag for board member shares
    /// Once locked, shares can only be changed via governance proposal
    pub static BOARD_SHARES_LOCKED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            false
        ).unwrap()
    );
    
    /// Learning engine canister ID
    pub static LEARNING_ENGINE_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))), Principal::anonymous()).unwrap()
    );
    
    // =========================================================================
    // MUTABLE GOVERNANCE CONFIGURATION (modifiable via admin proposals)
    // =========================================================================
    
    /// Minimum voting power required to create a proposal (in e8s)
    pub static MIN_VOTING_POWER_CONFIG: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            DEFAULT_MIN_VOTING_POWER_TO_PROPOSE
        ).unwrap()
    );
    
    /// Support threshold: voting power needed to move from Proposed to Active (in e8s)
    pub static SUPPORT_THRESHOLD_CONFIG: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            DEFAULT_SUPPORT_THRESHOLD
        ).unwrap()
    );
    
    /// Approval percentage: percentage of total staked needed for YES votes to pass (1-100)
    pub static APPROVAL_PERCENTAGE_CONFIG: RefCell<StableCell<u8, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))),
            DEFAULT_APPROVAL_PERCENTAGE
        ).unwrap()
    );
    
    /// Support period: time for proposals to gather support before expiring (in nanoseconds)
    pub static SUPPORT_PERIOD_CONFIG: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12))),
            DEFAULT_SUPPORT_PERIOD_NANOS
        ).unwrap()
    );
    
    /// Voting period: duration for active voting on proposals (in nanoseconds)
    pub static VOTING_PERIOD_CONFIG: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13))),
            DEFAULT_VOTING_PERIOD_NANOS
        ).unwrap()
    );
    
    /// Resubmission cooldown: time before a rejected proposal can be resubmitted (in nanoseconds)
    pub static RESUBMISSION_COOLDOWN_CONFIG: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14))),
            DEFAULT_RESUBMISSION_COOLDOWN_NANOS
        ).unwrap()
    );
}
