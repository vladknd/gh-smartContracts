mod types;
mod state;
mod service;

use ic_cdk::{init, update};
use candid::{Principal};
use crate::types::*;
use crate::state::*;

#[init]
fn init(args: InitArgs) {
    SONIC_CANISTER_ID.with(|id| id.borrow_mut().set(args.sonic_canister_id).expect("Failed to set sonic id"));
    GHC_LEDGER_ID.with(|id| id.borrow_mut().set(args.ghc_ledger_id).expect("Failed to set ghc id"));
    USDC_LEDGER_ID.with(|id| id.borrow_mut().set(args.usdc_ledger_id).expect("Failed to set usdc id"));
    OWNER.with(|id| id.borrow_mut().set(args.owner).expect("Failed to set owner"));
}

#[update]
async fn launch_ico(args: LaunchIcoArgs) -> Result<String, String> {
    service::launch_ico_impl(ic_cdk::caller(), args).await
}

#[update]
async fn add_liquidity(token_a: Principal, token_b: Principal, amount_a: u64, amount_b: u64) -> Result<String, String> {
    service::add_liquidity_impl(ic_cdk::caller(), token_a, token_b, amount_a, amount_b).await
}

#[update]
async fn swap(token_in: Principal, amount_in: u64, token_out: Principal, min_amount_out: u64) -> Result<String, String> {
    service::swap_impl(ic_cdk::caller(), token_in, amount_in, token_out, min_amount_out).await
}

ic_cdk::export_candid!();
