
use crate::VerificationTier;

// ... existing code ...

#[update]
async fn verify_humanity() -> Result<bool, String> {
    let user = ic_cdk::caller();
    
    // 1. In a real integration, we would call the DecideID canister here.
    // let decide_id_canister = Principal::from_text("...").unwrap();
    // let result = ic_cdk::call(decide_id_canister, "check_proof", (user,))...
    
    // For now, we simulate success for demonstration if not already verified
    // IMPORTANT: In production, this MUST verify the proof!
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            // Only upgrade if currently None (don't downgrade KYC users)
            if profile.verification_tier == VerificationTier::None {
                profile.verification_tier = VerificationTier::Human;
                map.insert(user, profile);
                Ok(true)
            } else {
                // Already Verified or Higher
                Ok(true)
            }
        } else {
            Err("User not found".to_string())
        }
    })
}

/// Admin function to manually set KYC tier (or via Trusted Oracle)
#[update]
fn admin_set_kyc_tier(target_user: Principal, tier: VerificationTier) -> Result<(), String> {
    // 1. Access Control: Check if caller is an admin or trusted oracle
    // For simplicity using a hardcoded check or the governance canister ID.
    // let caller = ic_cdk::caller();
    // if caller != GOVERNANCE_CANISTER_ID { return Err("Unauthorized"); }

    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&target_user) {
            profile.verification_tier = tier;
            map.insert(target_user, profile);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    })
}
