use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
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
    pub static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), Principal::anonymous()).unwrap()
    );
    
    pub static GOVERNANCE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), Principal::anonymous()).unwrap()
    );
    
    // Treasury
    pub static TREASURY_STATE: RefCell<StableCell<TreasuryState, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            TreasuryState {
                balance: INITIAL_TREASURY_BALANCE,
                allowance: INITIAL_TREASURY_ALLOWANCE,
                total_transferred: 0,
                mmcr_count: 0,
                last_mmcr_timestamp: 0,
                genesis_timestamp: 0,
                last_mmcr_month: 0,
                last_mmcr_year: 0,
            }
        ).unwrap()
    );
}
