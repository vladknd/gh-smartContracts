use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub parent_shard_id: Principal,
}

/// Key for archived transactions
/// Sorted by (user, sequence) for efficient per-user range queries
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArchiveKey {
    pub user: Principal,
    pub sequence: u64,
}

impl Storable for ArchiveKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        // Simple serialization - user bytes + u64 bytes
        // Principal is max 29 bytes. u64 is 8 bytes.
        // Prefix with length byte to avoid ambiguity if needed, but here we know Principal structure
        // Actually, just using candid is easier and safer for variable length principal
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ArchiveKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

/// Transaction data received from user_profile shard for archiving
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionToArchive {
    pub sequence: u64,
    pub timestamp: u64,
    pub transaction_type: String, // "Stake", "Unstake", "Reward", etc.
    pub amount: u64,
    pub metadata: String, // JSON or formatted string
}

/// Archived transaction record (includes archive metadata)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ArchivedTransaction {
    pub sequence: u64,
    pub timestamp: u64,
    pub transaction_type: String,
    pub amount: u64,
    pub metadata: String,
    pub archived_at: u64,
}

impl Storable for ArchivedTransaction {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ArchivedTransaction")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1024, // Increased to accommodate metadata
        is_fixed_size: false,
    };
}

/// Archive statistics for monitoring
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ArchiveStats {
    pub parent_shard: Principal,
    pub entry_count: u64,
    pub size_bytes: u64, // Approximate
    pub is_full: bool,
    pub next_archive: Option<Principal>,
}
