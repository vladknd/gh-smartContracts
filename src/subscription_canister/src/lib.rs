use candid::Principal;
use ic_cdk::{init, query, update};
// use ic_cdk::api::management_canister::http_request::{
//     http_request, CanisterHttpRequestArgument, HttpMethod, HttpHeader, HttpResponse,
// };

mod types;
mod state;

use types::*;
use state::*;

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
}

#[update]
async fn request_checkout(shard_id: Principal) -> Result<String, String> {
    let user = ic_cdk::caller();
    if user == Principal::anonymous() {
        return Err("Anonymous users cannot start checkout".to_string());
    }

    // In a real implementation:
    // 1. Call your Stripe Bridge (Web2 service) via HTTPS Outcall
    // 2. Receive a Checkout URL and Session ID
    // 3. Store the session_id -> user mapping
    
    // For this demonstration and testing, we generate a mock session ID
    let session_id = format!("sess_{}_{}", user, ic_cdk::api::time());
    
    PENDING_SESSIONS.with(|s| {
        s.borrow_mut().insert(session_id.clone(), user);
    });

    SUBSCRIPTIONS.with(|s| {
        let record = SubscriptionRecord {
            user,
            shard_id,
            session_id: session_id.clone(),
            amount: 1000, // $10.00
            timestamp: ic_cdk::api::time(),
            status: "pending".to_string(),
        };
        s.borrow_mut().insert(user, record);
    });

    // Return the "Stripe URL" (mocked)
    Ok(format!("https://checkout.stripe.com/pay/{}", session_id))
}

#[update]
async fn confirm_payment(session_id: String) -> Result<(), String> {
    // 1. Retrieve the user principal for this session
    let user = PENDING_SESSIONS.with(|s| {
        s.borrow().get(&session_id)
    }).ok_or("Session not found".to_string())?;

    // 2. IMPORTANT: Verify with Stripe directly via HTTPS Outcall
    // this prevents spoofing from the Bridge.
    verify_with_stripe(&session_id).await?;

    // 3. Update local state
    let shard_id = SUBSCRIPTIONS.with(|s| {
        let mut map = s.borrow_mut();
        if let Some(mut record) = map.get(&user) {
            record.status = "paid".to_string();
            let shard = record.shard_id;
            map.insert(user, record);
            Ok(shard)
        } else {
            Err("Subscription record missing".to_string())
        }
    })?;

    // 4. Activate the subscription on the user's specific shard
    let shard_call: Result<(Result<(), String>,), _> = ic_cdk::call(
        shard_id,
        "internal_set_subscription",
        (user, true)
    ).await;

    match shard_call {
        Ok((Ok(()),)) => {
            // Success! Clean up pending session
            PENDING_SESSIONS.with(|s| s.borrow_mut().remove(&session_id));
            Ok(())
        },
        Ok((Err(e),)) => Err(format!("Shard activation failed: {}", e)),
        Err((code, msg)) => Err(format!("Shard call failed: {:?} {}", code, msg)),
    }
}

async fn verify_with_stripe(session_id: &str) -> Result<(), String> {
    // This is where you would call the actual Stripe API
    // GET https://api.stripe.com/v1/checkout/sessions/{session_id}
    
    // For this architecture demo, we'll implement a mock check.
    // In production, you would use ic_cdk::api::management_canister::http_request::http_request
    
    /* 
    let request_headers = vec![
        HttpHeader { name: "Authorization".to_string(), value: "Bearer sk_test_...".to_string() },
    ];

    let arg = CanisterHttpRequestArgument {
        url: format!("https://api.stripe.com/v1/checkout/sessions/{}", session_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2000),
        transform: None, // Or use a transform function to normalize response
        headers: request_headers,
    };

    match http_request(arg).await {
        Ok((response,)) => {
            // parse response.body and check if status is "paid"
            Ok(())
        },
        Err(_) => Err("Stripe verification failed".to_string()),
    }
    */
    
    // Mocking success for now
    if session_id.contains("fail") {
        return Err("Stripe reported payment failed".to_string());
    }
    
    Ok(())
}

#[query]
fn get_subscription_status(user: Principal) -> bool {
    SUBSCRIPTIONS.with(|s| {
        s.borrow().get(&user).map(|r| r.status == "paid").unwrap_or(false)
    })
}

#[update]
async fn admin_sync_user_subscription(user: Principal, shard_id: Principal, active: bool) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can perform manual sync".to_string());
    }

    let shard_call: Result<(Result<(), String>,), _> = ic_cdk::call(
        shard_id,
        "internal_set_subscription",
        (user, active)
    ).await;

    match shard_call {
        Ok((Ok(()),)) => Ok(()),
        Ok((Err(e),)) => Err(format!("Shard activation failed: {}", e)),
        Err((code, msg)) => Err(format!("Shard call failed: {:?} {}", code, msg)),
    }
}

#[update]
fn admin_update_staking_hub(new_id: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can update Hub ID".to_string());
    }
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(new_id).expect("Failed to update Hub ID"));
    Ok(())
}

#[query]
fn get_staking_hub() -> Principal {
    STAKING_HUB_ID.with(|id| *id.borrow().get())
}
