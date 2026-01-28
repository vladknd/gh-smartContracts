use candid::{CandidType, Deserialize, Nat, Principal, Encode, Decode};
use std::borrow::Cow;
use ic_stable_structures::{Storable, storable::Bound};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub admin_principal: Principal,
    pub treasury_principal: Principal,
    pub ghc_ledger_id: Principal,
    pub ckusdc_ledger_id: Principal,
    pub price_per_token_e6: Nat, // Price of 1 GHC in USDC (e6 format). E.g. 0.05 USDC = 50_000
    pub ghc_decimals: u8,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct IcoState {
    pub admin_principal: Principal,
    pub treasury_principal: Principal,
    pub ghc_ledger_id: Principal,
    pub ckusdc_ledger_id: Principal,
    pub price_per_token_e6: Nat,
    pub ghc_decimals: u8,
    pub total_raised_usdc: Nat,
    pub total_sold_ghc: Nat,
}

impl Storable for IcoState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1024, // Generous estimate, Nat can be large but likely small here
        is_fixed_size: false,
    };
}

impl Default for IcoState {
    fn default() -> Self {
        Self {
            admin_principal: Principal::anonymous(),
            treasury_principal: Principal::anonymous(),
            ghc_ledger_id: Principal::anonymous(),
            ckusdc_ledger_id: Principal::anonymous(),
            price_per_token_e6: Nat::from(0u64),
            ghc_decimals: 0,
            total_raised_usdc: Nat::from(0u64),
            total_sold_ghc: Nat::from(0u64),
        }
    }
}
