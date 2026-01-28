use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{Storable};
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;
use ic_stable_structures::{DefaultMemoryImpl};
use ic_stable_structures::memory_manager::VirtualMemory;

use crate::constants::{YEAR_IN_NANOS, ANNUAL_UNLOCK_BPS, MAX_UNLOCK_BPS};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

/// Vesting schedule for a founder
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VestingSchedule {
    /// Founder principal ID
    pub founder: Principal,
    
    /// Total tokens allocated (never changes)
    pub total_allocation: u64,
    
    /// Tokens already claimed
    pub claimed: u64,
    
    /// Vesting start timestamp (set at canister init)
    pub vesting_start: u64,
}

impl VestingSchedule {
    /// Calculate currently vested (unlocked) tokens based on elapsed time
    pub fn vested_amount(&self, current_time: u64) -> u64 {
        if current_time <= self.vesting_start {
            return 0;
        }
        
        let elapsed_nanos = current_time - self.vesting_start;
        let elapsed_years = elapsed_nanos / YEAR_IN_NANOS;
        
        // 10% immediately, then 10% per year.
        // Year 0 (immediate): 10%
        // Year 1: 20%
        // ...
        // Year 9: 100%
        let unlock_bps = ((elapsed_years + 1) * ANNUAL_UNLOCK_BPS).min(MAX_UNLOCK_BPS);
        
        // Calculate vested amount: (total * unlock_bps) / 10000
        ((self.total_allocation as u128 * unlock_bps as u128) / 10000) as u64
    }
    
    /// Calculate claimable tokens (vested - already claimed)
    pub fn claimable(&self, current_time: u64) -> u64 {
        self.vested_amount(current_time).saturating_sub(self.claimed)
    }
}

impl Storable for VestingSchedule {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Public vesting status for queries
#[derive(CandidType, Clone, Debug)]
pub struct VestingStatus {
    pub founder: Principal,
    pub total_allocation: u64,
    pub vested: u64,
    pub claimed: u64,
    pub claimable: u64,
    pub years_elapsed: u64,
    pub unlock_percentage: u64,
}

/// Initialization arguments
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    /// GHC Ledger canister ID
    pub ledger_id: Principal,
}
