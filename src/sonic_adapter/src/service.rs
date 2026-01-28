use candid::{Principal, Nat, Int};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::approve::{ApproveArgs};
use crate::state::*;
use crate::types::*;

pub fn verify_owner(caller: Principal) -> Result<(), String> {
    let owner = OWNER.with(|id| *id.borrow().get());
    if caller != owner {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

pub async fn approve_token(ledger: Principal, spender: Principal, amount: u64) -> Result<(), String> {
    let approve_args = ApproveArgs {
        from_subaccount: None,
        spender: Account { owner: spender, subaccount: None },
        amount: Nat::from(amount),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, icrc_ledger_types::icrc2::approve::ApproveError>,) = ic_cdk::call(
        ledger,
        "icrc2_approve",
        (approve_args,)
    ).await.map_err(|(code, msg)| format!("Approve call failed: {:?} {}", code, msg))?;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Approve failed: {:?}", e)),
    }
}

pub async fn launch_ico_impl(caller: Principal, args: LaunchIcoArgs) -> Result<String, String> {
    // 1. Verify caller is owner (Governance)
    verify_owner(caller)?;

    let sonic_id = SONIC_CANISTER_ID.with(|id| *id.borrow().get());
    let ghc_ledger = GHC_LEDGER_ID.with(|id| *id.borrow().get());
    let usdc_ledger = USDC_LEDGER_ID.with(|id| *id.borrow().get());

    // 2. Approve Sonic to spend our tokens
    approve_token(ghc_ledger, sonic_id, args.ghc_amount).await?;
    approve_token(usdc_ledger, sonic_id, args.usdc_amount).await?;

    // 3. Call Sonic to add liquidity
    // For ICO launch, we set min amounts to 0 or very low, assuming we validly set the price
    let deadline = Int::from(ic_cdk::api::time() + 600_000_000_000); // +10 mins

    let (result,): (Result<Nat, String>,) = ic_cdk::call(
        sonic_id, 
        "addLiquidity", // This name must match Sonic's actual method
        (ghc_ledger, usdc_ledger, Nat::from(args.ghc_amount), Nat::from(args.usdc_amount), Nat::from(0u64), Nat::from(0u64), deadline)
    ).await.map_err(|(code, msg)| format!("Call to Sonic addLiquidity failed: {:?} {}", code, msg))?;

    match result {
        Ok(_) => Ok(format!("ICO Launched: Added {} GHC and {} USDC liquidity", args.ghc_amount, args.usdc_amount)),
        Err(e) => Err(format!("Sonic addLiquidity error: {}", e)),
    }
}

pub async fn add_liquidity_impl(caller: Principal, token_a: Principal, token_b: Principal, amount_a: u64, amount_b: u64) -> Result<String, String> {
    verify_owner(caller)?;
    let sonic_id = SONIC_CANISTER_ID.with(|id| *id.borrow().get());

    // Approve both
    approve_token(token_a, sonic_id, amount_a).await?;
    approve_token(token_b, sonic_id, amount_b).await?;

    let deadline = Int::from(ic_cdk::api::time() + 600_000_000_000);

    // Using tuple arg style as that's often how dynamic calls work if struct definition isn't shared
    // Signature: (tokenA, tokenB, amountADesired, amountBDesired, amountAMin, amountBMin, deadline)
    let (result,): (Result<Nat, String>,) = ic_cdk::call(
        sonic_id,
        "addLiquidity",
        (token_a, token_b, Nat::from(amount_a), Nat::from(amount_b), Nat::from(0u64), Nat::from(0u64), deadline)
    ).await.map_err(|(code, msg)| format!("Sonic call failed: {:?} {}", code, msg))?;

    match result {
        Ok(lp_tokens) => Ok(format!("Liquidity Added. LP Tokens: {}", lp_tokens)),
        Err(e) => Err(e),
    }
}

pub async fn swap_impl(caller: Principal, token_in: Principal, amount_in: u64, token_out: Principal, min_amount_out: u64) -> Result<String, String> {
    verify_owner(caller)?;
    let sonic_id = SONIC_CANISTER_ID.with(|id| *id.borrow().get());

    // Approve token_in
    approve_token(token_in, sonic_id, amount_in).await?;

    let deadline = Int::from(ic_cdk::api::time() + 600_000_000_000);
    let path = vec![token_in, token_out];
    let to = ic_cdk::id(); // Send tokens back to this canister (adapter)

    // Signature: (amountIn, amountOutMin, path, to, deadline)
    let (result,): (Result<Vec<Nat>, String>,) = ic_cdk::call(
        sonic_id,
        "swapExactTokensForTokens",
        (Nat::from(amount_in), Nat::from(min_amount_out), path, to, deadline)
    ).await.map_err(|(code, msg)| format!("Sonic swap failed: {:?} {}", code, msg))?;

    match result {
        Ok(amounts) => Ok(format!("Swap complete. Output amounts: {:?}", amounts)),
        Err(e) => Err(e),
    }
}
