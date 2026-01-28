use candid::{Principal, Nat};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use crate::state::{VESTING_SCHEDULES, LEDGER_ID};

pub async fn claim_vested_tokens(caller: Principal, current_time: u64) -> Result<u64, String> {
    // Find founder's vesting schedule
    // Use a block to scope the borrow
    let (claimable, schedule_claimed, vested_amount) = VESTING_SCHEDULES.with(|v| {
        let schedules = v.borrow();
        let schedule = schedules.get(&caller).clone(); // Clone to drop borrow
        match schedule {
            Some(sched) => {
                let claimable = sched.claimable(current_time);
                Ok((claimable, sched.claimed, sched.vested_amount(current_time)))
            },
            None => Err("Caller is not a registered founder".to_string())
        }
    })?;

    if claimable == 0 {
        return Err(format!(
            "No tokens available to claim. Vested: {} e8s, Already claimed: {} e8s",
            vested_amount, schedule_claimed
        ));
    }
    
    // Execute ICRC-1 transfer to founder
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None, // From this canister's main account
        to: Account { owner: caller, subaccount: None },
        amount: Nat::from(claimable),
        fee: None, // Transfer fee is 0
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer call failed: {:?} {}", code, msg))?;

    match result {
        Ok(_block_index) => {
            // Update claimed amount
            VESTING_SCHEDULES.with(|v| {
                let mut schedules = v.borrow_mut();
                if let Some(mut sched) = schedules.get(&caller) {
                    sched.claimed += claimable;
                    schedules.insert(caller, sched);
                }
            });
            
            ic_cdk::println!(
                "Founder {} claimed {} e8s. Total claimed: {} e8s",
                caller, claimable, schedule_claimed + claimable
            );
            
            Ok(claimable)
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}
