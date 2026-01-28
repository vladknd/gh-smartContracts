mod constants;
mod types;
mod state;
mod service;

use candid::{Nat, Principal};
use ic_cdk_macros::{init, query, update};
use crate::types::{InitArgs, IcoState};
use crate::state::STATE;

#[init]
fn init(args: InitArgs) {
    STATE.with(|s| {
        let mut cell = s.borrow_mut();
        cell.set(IcoState {
            admin_principal: args.admin_principal,
            treasury_principal: args.treasury_principal,
            ghc_ledger_id: args.ghc_ledger_id,
            ckusdc_ledger_id: args.ckusdc_ledger_id,
            price_per_token_e6: args.price_per_token_e6,
            ghc_decimals: args.ghc_decimals,
            total_raised_usdc: Nat::from(0u64),
            total_sold_ghc: Nat::from(0u64),
        }).expect("Failed to initialize state");
    });
}

#[update]
async fn buy_ghc(amount_ghc: Nat) -> Result<String, String> {
    service::buy_ghc_impl(ic_cdk::caller(), amount_ghc).await
}

#[update]
async fn withdraw_usdc(destination: Principal, amount: Nat) -> Result<String, String> {
    service::withdraw_usdc_impl(ic_cdk::caller(), destination, amount).await
}

#[update]
async fn withdraw_ghc(destination: Principal, amount: Nat) -> Result<String, String> {
    service::withdraw_ghc_impl(ic_cdk::caller(), destination, amount).await
}

#[update]
async fn end_sale() -> Result<String, String> {
    service::end_sale_impl(ic_cdk::caller()).await
}

#[query]
fn get_ico_stats() -> IcoState {
    STATE.with(|s| s.borrow().get().clone())
}

ic_cdk::export_candid!();
