use candid::{Principal, Nat};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use crate::state::STATE;
use crate::constants::DEFAULT_DECIMALS_POW_BASE;

pub async fn buy_ghc_impl(caller: Principal, amount_ghc: Nat) -> Result<String, String> {
    let state = STATE.with(|s| s.borrow().get().clone());
    
    // 1. Calculate Cost
    // Formula: cost = (amount_ghc * price_e6) / 10^ghc_decimals
    let decimals_pow = Nat::from(DEFAULT_DECIMALS_POW_BASE).0.pow(state.ghc_decimals as u32);
    let decimals_pow = Nat::from(decimals_pow); // Wrap back to Nat
    let cost_usdc = (amount_ghc.clone() * state.price_per_token_e6.clone()) / decimals_pow; // Integer division

    if cost_usdc == Nat::from(0u64) {
        return Err("Amount too small".to_string());
    }

    let canister_id = ic_cdk::api::id();

    // 2. Pull USDC from User (ICRC-2)
    // User must have approved this canister to spend their USDC beforehand.
    let transfer_from_args = TransferFromArgs {
        spender_subaccount: None,
        from: Account { owner: caller, subaccount: None },
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
            STATE.with(|s| {
                let mut cell = s.borrow_mut();
                let mut current_state = cell.get().clone();
                current_state.total_raised_usdc += cost_usdc.clone();
                current_state.total_sold_ghc += amount_ghc.clone();
                cell.set(current_state).unwrap();
            });
        }
        Err(e) => return Err(format!("USDC Transfer failed: {:?}", e)),
    }

    // 3. Send GHC to User
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: caller, subaccount: None },
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
            Err(format!("GHC Transfer failed (Funds deducted! Contact support): {:?}", e))
        },
    }
}

pub async fn withdraw_usdc_impl(caller: Principal, destination: Principal, amount: Nat) -> Result<String, String> {
    let state = STATE.with(|s| s.borrow().get().clone());
    if caller != state.admin_principal {
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

pub async fn withdraw_ghc_impl(caller: Principal, destination: Principal, amount: Nat) -> Result<String, String> {
    let state = STATE.with(|s| s.borrow().get().clone());
    if caller != state.admin_principal {
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

pub async fn end_sale_impl(caller: Principal) -> Result<String, String> {
    let state = STATE.with(|s| s.borrow().get().clone());
    if caller != state.admin_principal {
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
