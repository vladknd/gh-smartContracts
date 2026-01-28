use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::types::*;

thread_local! {
    pub static STAKING_HUB_ID: RefCell<Principal> = RefCell::new(Principal::anonymous());
    pub static KYC_RECORDS: RefCell<HashMap<Principal, KycStatus>> = RefCell::new(HashMap::new());
}
