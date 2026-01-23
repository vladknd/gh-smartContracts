use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::caller;
use ic_cdk_macros::{init, query, update};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};

#[derive(CandidType, Deserialize, Clone, Debug)]
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

static mut STATE: Option<IcoState> = None;

#[init]
fn init(args: InitArgs) {
    unsafe {
        STATE = Some(IcoState {
            admin_principal: args.admin_principal,
            treasury_principal: args.treasury_principal,
            ghc_ledger_id: args.ghc_ledger_id,
            ckusdc_ledger_id: args.ckusdc_ledger_id,
            price_per_token_e6: args.price_per_token_e6,
            ghc_decimals: args.ghc_decimals,
            total_raised_usdc: Nat::from(0u64),
            total_sold_ghc: Nat::from(0u64),
        });
    }
}

#[update]
async fn buy_ghc(amount_ghc: Nat) -> Result<String, String> {
    let state = unsafe { STATE.as_ref().unwrap().clone() };
    
    // 1. Calculate Cost
    // Formula: cost = (amount_ghc * price_e6) / 10^ghc_decimals
    // Example: Buy 100 GHC (100 * 10^8) at 0.05 USDC (50_000).
    // Cost = (100 * 10^8 * 50_000) / 10^8 = 5,000,000 (5 USDC).
    let decimals_pow = Nat::from(10u64).pow(state.ghc_decimals as u32);
    let cost_usdc = (amount_ghc.clone() * state.price_per_token_e6.clone()) / decimals_pow;

    if cost_usdc == Nat::from(0u64) {
        return Err("Amount too small".to_string());
    }

    let user = caller();
    let canister_id = ic_cdk::api::id();

    // 2. Pull USDC from User (ICRC-2)
    // User must have approved this canister to spend their USDC beforehand.
    let transfer_from_args = TransferFromArgs {
        spender_subaccount: None,
        from: Account { owner: user, subaccount: None },
        to: Account { owner: canister_id, subaccount: None },
        amount: cost_usdc.clone(),
        fee: None, // Use default ledger fee
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferFromError>,) = ic_cdk::call(
        state.ckusdc_ledger_id,
        "icrc2_transfer_from",
        (transfer_from_args,),
    )
    .await
    .map_err(|e| format!("Failed to call USDC ledger: {:?}", e))?;

    match result {
        Ok(_) => {
            // Payment success, update stats
            unsafe {
                if let Some(s) = STATE.as_mut() {
                    s.total_raised_usdc += cost_usdc.clone();
                    s.total_sold_ghc += amount_ghc.clone();
                }
            }
        }
        Err(e) => return Err(format!("USDC Transfer failed: {:?}", e)),
    }

    // 3. Send GHC to User
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: user, subaccount: None },
        amount: amount_ghc.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (ghc_result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        state.ghc_ledger_id,
        "icrc1_transfer",
        (transfer_args,),
    )
    .await
    .map_err(|e| format!("Failed to call GHC ledger: {:?}", e))?;

    match ghc_result {
        Ok(_) => Ok("Purchase successful".to_string()),
        Err(e) => {
            // CRITICAL: Refund logic should be here in production if GHC transfer fails!
            // For this MVP, we report error. In real ICO, utilize a saga pattern or ensuring supply first.
            Err(format!("GHC Transfer failed (Funds deducted! Contact support): {:?}", e))
        },
    }
}

#[update]
async fn withdraw_usdc(destination: Principal, amount: Nat) -> Result<String, String> {
    let state = unsafe { STATE.as_ref().unwrap().clone() };
    if caller() != state.admin_principal {
        return Err("Unauthorized".to_string());
    }

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: destination, subaccount: None },
        amount,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        state.ckusdc_ledger_id,
        "icrc1_transfer",
        (transfer_args,),
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    match result {
        Ok(_) => Ok("Withdrawal successful".to_string()),
        Err(e) => Err(format!("Transfer error: {:?}", e)),
    }
}

#[update]
async fn withdraw_ghc(destination: Principal, amount: Nat) -> Result<String, String> {
    let state = unsafe { STATE.as_ref().unwrap().clone() };
    if caller() != state.admin_principal {
        return Err("Unauthorized".to_string());
    }

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: destination, subaccount: None },
        amount,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        state.ghc_ledger_id,
        "icrc1_transfer",
        (transfer_args,),
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    match result {
        Ok(_) => Ok("Withdrawal successful".to_string()),
        Err(e) => Err(format!("Transfer error: {:?}", e)),
    }
}

#[update]
async fn end_sale() -> Result<String, String> {
    let state = unsafe { STATE.as_ref().unwrap().clone() };
    if caller() != state.admin_principal {
        return Err("Unauthorized".to_string());
    }

    // 1. Get Current USDC Balance of ICO Canister from Ledger
    let canister_id = ic_cdk::api::id();
    let balance_args = Account { owner: canister_id, subaccount: None };
    
    let (balance_usdc,): (Nat,) = ic_cdk::call(
        state.ckusdc_ledger_id,
        "icrc1_balance_of",
        (balance_args,),
    )
    .await
    .map_err(|e| format!("Failed to fetch USDC balance: {:?}", e))?;

    // 2. Sweep USDC to Treasury
    if balance_usdc > Nat::from(0u64) {
        let transfer_args = TransferArg {
            from_subaccount: None,
            to: Account { owner: state.treasury_principal, subaccount: None },
            amount: balance_usdc.clone(),
            fee: None,
            memo: None,
            created_at_time: None,
        };

        let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
            state.ckusdc_ledger_id,
            "icrc1_transfer",
            (transfer_args,),
        )
        .await
        .map_err(|e| format!("USDC transfer call failed: {:?}", e))?;

        if let Err(e) = result {
             return Err(format!("USDC sweep failed: {:?}", e));
        }
    }

    // 3. Get Current GHC Balance (Unsold inventory)
    let balance_args_ghc = Account { owner: canister_id, subaccount: None };
    
    let (balance_ghc,): (Nat,) = ic_cdk::call(
        state.ghc_ledger_id,
        "icrc1_balance_of",
        (balance_args_ghc,),
    )
    .await
    .map_err(|e| format!("Failed to fetch GHC balance: {:?}", e))?;

    // 4. Return Unsold GHC to Treasury
    if balance_ghc > Nat::from(0u64) {
        let transfer_args = TransferArg {
            from_subaccount: None,
            to: Account { owner: state.treasury_principal, subaccount: None },
            amount: balance_ghc.clone(),
            fee: None,
            memo: None,
            created_at_time: None,
        };

        let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
            state.ghc_ledger_id,
            "icrc1_transfer",
            (transfer_args,),
        )
        .await
        .map_err(|e| format!("GHC transfer call failed: {:?}", e))?;

        if let Err(e) = result {
             return Err(format!("GHC seep failed: {:?}", e));
        }
    }

    Ok("Sale ended. All assets swept to Treasury.".to_string())
}

#[query]
fn get_ico_stats() -> IcoState {
    unsafe { STATE.as_ref().unwrap().clone() }
}
