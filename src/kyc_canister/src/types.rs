use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationTier {
    None,
    Human,
    KYC,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct KycStatus {
    pub user: Principal,
    pub tier: VerificationTier,
    pub verified_at: u64,
    pub provider: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct InitArgs {
    pub staking_hub_id: Principal,
}
