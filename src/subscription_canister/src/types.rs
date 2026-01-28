use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub staking_hub_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SubscriptionRecord {
    pub user: Principal,
    pub shard_id: Principal,
    pub session_id: String,
    pub amount: u64,
    pub timestamp: u64,
    pub status: String, // "pending", "paid", "expired"
}

impl Storable for SubscriptionRecord {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode SubscriptionRecord")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1024,
        is_fixed_size: false,
    };
}
