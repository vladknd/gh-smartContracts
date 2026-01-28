use candid::Principal;
use ic_cdk::{init, query, update};

mod types;
mod state;

use types::*;
use state::*;

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| {
        *id.borrow_mut() = args.staking_hub_id;
    });
}

#[update]
async fn submit_kyc_data(data: String) -> Result<String, String> {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return Err("Anonymous caller not allowed".to_string());
    }

    // Mock: Store submission or process metadata
    ic_cdk::print(format!("User {} submitted KYC data: {}", caller, data));
    
    Ok(format!("KYC data received for {}. Please call verify_identity to proceed.", caller))
}

#[update]
async fn verify_identity(user: Principal) -> Result<VerificationTier, String> {
    // In production, this would perform an AI check via HTTPS Outcalls
    // OR verify a signature from a trusted AI provider.
    
    let tier = VerificationTier::KYC; // Mock: Everyone gets KYC tier for testing
    let now = ic_cdk::api::time();
    
    // 1. Update local state
    KYC_RECORDS.with(|r| {
        let mut records = r.borrow_mut();
        records.insert(user, KycStatus {
            user,
            tier: tier.clone(),
            verified_at: now,
            provider: "MockAI".to_string(),
        });
    });

    // 2. Resolve the user's shard from the Staking Hub
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow());
    let (shard_id_opt,): (Option<Principal>,) = ic_cdk::call(hub_id, "get_user_shard", (user,))
        .await
        .map_err(|e| format!("Hub call failed: {:?}", e))?;
    
    let shard_id = shard_id_opt.ok_or("User not registered in any shard")?;

    // 3. Update the shard (Remote Write)
    let _: (Result<(), String>,) = ic_cdk::call(shard_id, "internal_set_kyc_status", (user, tier.clone()))
        .await
        .map_err(|e| format!("Shard call failed: {:?}", e))?;

    Ok(tier)
}

#[query]
fn get_user_kyc_status(user: Principal) -> Option<KycStatus> {
    KYC_RECORDS.with(|r| r.borrow().get(&user).cloned())
}

#[update]
fn admin_set_staking_hub(new_id: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    STAKING_HUB_ID.with(|id| {
        *id.borrow_mut() = new_id;
    });
    Ok(())
}

ic_cdk::export_candid!();
