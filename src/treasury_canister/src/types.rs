use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Token types for treasury spending
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    GHC,
    USDC,
    ICP,
}

/// Treasury state - tracks balance and allowance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    pub balance: u64,
    pub allowance: u64,
    pub total_transferred: u64,
    pub mmcr_count: u64,
    pub last_mmcr_timestamp: u64,
    pub genesis_timestamp: u64,
    /// Month of the last MMCR execution (1-12), used for calendar-based scheduling
    pub last_mmcr_month: u8,
    /// Year of the last MMCR execution, used for calendar-based scheduling  
    pub last_mmcr_year: u16,
}

impl Storable for TreasuryState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 250, is_fixed_size: false };
}

#[derive(CandidType, Clone, Debug)]
pub struct MMCRStatus {
    pub releases_completed: u64,
    pub releases_remaining: u64,
    pub last_release_timestamp: u64,
    pub next_release_amount: u64,
    pub seconds_until_next: u64,
    /// The month of the next scheduled MMCR (1-12)
    pub next_scheduled_month: u8,
    /// The year of the next scheduled MMCR
    pub next_scheduled_year: u16,
}

/// Input for executing a treasury transfer
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ExecuteTransferInput {
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub proposal_id: u64,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub ledger_id: Principal,
    pub governance_canister_id: Principal,
}

/// Date/time components for internal use
pub struct DateTimeComponents {
    pub year: u16,
    pub month: u8,   // 1-12
    pub day: u8,     // 1-31
    pub hour: u8,    // 0-23
    pub minute: u8,  // 0-59
    pub second: u8,  // 0-59
}
