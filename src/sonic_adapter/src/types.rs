use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub sonic_canister_id: Principal,
    pub ghc_ledger_id: Principal,
    pub usdc_ledger_id: Principal,
    pub owner: Principal, // Governance canister
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LaunchIcoArgs {
    pub ghc_amount: u64,
    pub usdc_amount: u64,
}
