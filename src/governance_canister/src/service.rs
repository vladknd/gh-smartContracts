use candid::{Principal, CandidType, Deserialize};
use crate::state::*;

// ============================================================================
// BOARD MEMBER HELPERS
// ============================================================================

/// Check if a principal is a board member (local check)
pub fn is_board_member_local(principal: &Principal) -> bool {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().contains_key(principal))
}

/// Get a board member's percentage share (internal)
pub fn get_board_member_percentage_local(principal: &Principal) -> Option<u8> {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().get(principal))
}

// ============================================================================
// GOVERNANCE CONFIG HELPERS
// ============================================================================

/// Get current minimum voting power required to create a proposal (in e8s)
pub fn get_min_voting_power_to_propose() -> u64 {
    MIN_VOTING_POWER_CONFIG.with(|c| *c.borrow().get())
}

/// Get current support threshold (voting power to move from Proposed to Active)
pub fn get_support_threshold() -> u64 {
    SUPPORT_THRESHOLD_CONFIG.with(|c| *c.borrow().get())
}

/// Get current approval percentage (percentage of total staked for YES votes to pass)
pub fn get_approval_percentage() -> u8 {
    APPROVAL_PERCENTAGE_CONFIG.with(|c| *c.borrow().get())
}

/// Get current support period (time for proposals to gather support, in nanoseconds)
pub fn get_support_period() -> u64 {
    SUPPORT_PERIOD_CONFIG.with(|c| *c.borrow().get())
}

/// Get current voting period (duration for active voting, in nanoseconds)
pub fn get_voting_period() -> u64 {
    VOTING_PERIOD_CONFIG.with(|c| *c.borrow().get())
}

/// Get current resubmission cooldown (time before rejected proposal can be resubmitted, in nanoseconds)
pub fn get_resubmission_cooldown() -> u64 {
    RESUBMISSION_COOLDOWN_CONFIG.with(|c| *c.borrow().get())
}

/// Calculate the required YES votes for approval based on total staked tokens
/// This queries the staking hub for total staked and applies the approval percentage
pub async fn calculate_approval_threshold() -> Result<u64, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // We need GlobalStats from staking hub to get total_staked
    // struct GlobalStats { total_staked: u64, ... }
    // Let's assume we can get it via get_global_stats
    
    // Using a simpler query for total staked if available, or fetch full stats
    #[derive(CandidType, Deserialize, Clone, Debug)]
    struct GlobalStatsPartial {
        total_staked: u64,
        total_unstaked: u64,
        total_allocated: u64,
    }
    
    let result: Result<(GlobalStatsPartial,), _> = ic_cdk::call(
        staking_hub_id,
        "get_global_stats",
        (),
    ).await;
    
    let total_staked = match result {
        Ok((stats,)) => stats.total_staked,
        Err((code, msg)) => return Err(format!("Failed to query staking hub: {:?} {}", code, msg)),
    };
    
    // Get approval percentage config (default 30%)
    let approval_pct = get_approval_percentage() as u64;
    
    // Calculate threshold
    let threshold = (total_staked * approval_pct) / 100;
    
    Ok(threshold)
}

/// Fetch voting power for a user
/// - Board members: returns VUC * percentage / 100 (queries staking hub for VUC)
/// - Regular users: returns staked balance (queries staking hub)
pub async fn fetch_voting_power(user: Principal) -> Result<u64, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Check if board member first
    if let Some(percentage) = get_board_member_percentage_local(&user) {
        // It's a board member!
        // Their power is (percentage / 100) * VUC
        
        // Fetch VUC from staking hub
        let result: Result<(u64,), _> = ic_cdk::call(
            staking_hub_id,
            "get_vuc",
            (),
        ).await;
        
        match result {
            Ok((vuc,)) => {
                let power = (vuc * percentage as u64) / 100;
                return Ok(power);
            },
            Err((code, msg)) => return Err(format!("Failed to fetch VUC: {:?} {}", code, msg)),
        }
    }
    
    // Not a board member, check user staked balance
    let result: Result<(u64,), _> = ic_cdk::call(
        staking_hub_id,
        "fetch_user_voting_power",
        (user,),
    ).await;
    
    match result {
        Ok((power,)) => Ok(power),
        Err((code, msg)) => Err(format!("Failed to fetch user power: {:?} {}", code, msg)),
    }
}
