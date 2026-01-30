use candid::{Principal, CandidType, Deserialize};
use crate::state::*;
use crate::constants::*;

// ============================================================================
// BOARD MEMBER HELPERS
// ============================================================================

/// Check if a principal is any type of board member (regular or sentinel)
pub fn is_board_member_local(principal: &Principal) -> bool {
    // Check if sentinel
    let sentinel = SENTINEL_MEMBER.with(|s| *s.borrow().get());
    if sentinel != Principal::anonymous() && sentinel == *principal {
        return true;
    }
    // Check if regular board member
    BOARD_MEMBER_SHARES.with(|b| b.borrow().contains_key(principal))
}

/// Check if a principal is the sentinel member
pub fn is_sentinel_local(principal: &Principal) -> bool {
    let sentinel = SENTINEL_MEMBER.with(|s| *s.borrow().get());
    sentinel != Principal::anonymous() && sentinel == *principal
}

/// Get the sentinel member's principal (returns None if not set)
pub fn get_sentinel_local() -> Option<Principal> {
    let sentinel = SENTINEL_MEMBER.with(|s| *s.borrow().get());
    if sentinel == Principal::anonymous() {
        None
    } else {
        Some(sentinel)
    }
}

/// Get a regular board member's BPS share (returns None for sentinel or non-member)
pub fn get_board_member_share_bps_local(principal: &Principal) -> Option<u16> {
    // Sentinel is not in BPS map
    if is_sentinel_local(principal) {
        return None;
    }
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
    let mut threshold = (total_staked * approval_pct) / 100;
    
    // If threshold is set to 50%, add 1 unit of voting power to require a strict majority
    if approval_pct == 50 {
        threshold += 1;
    }
    
    Ok(threshold)
}

// ============================================================================
// VOTING POWER CALCULATION (Cumulative Partitioning - Zero Dust)
// ============================================================================

/// Fetch voting power for a user
/// 
/// - Sentinel member: returns exactly 1 unit of VUC (1 e8s)
/// - Regular board members: calculated using cumulative partitioning for zero dust
/// - Regular users: returns staked balance from staking hub
pub async fn fetch_voting_power(user: Principal) -> Result<u64, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Check if sentinel first - sentinel gets exactly 1 unit of VUC
    if is_sentinel_local(&user) {
        return Ok(1); // Exactly 1 e8s of voting power
    }
    
    // Check if regular board member
    if let Some(_user_share_bps) = get_board_member_share_bps_local(&user) {
        // It's a regular board member - use cumulative partitioning
        
        // Fetch VUC from staking hub
        let result: Result<(u64,), _> = ic_cdk::call(
            staking_hub_id,
            "get_vuc",
            (),
        ).await;
        
        let vuc = match result {
            Ok((v,)) => v,
            Err((code, msg)) => return Err(format!("Failed to fetch VUC: {:?} {}", code, msg)),
        };
        
        // Get all board members sorted by Principal for determinism
        let mut all_members: Vec<(Principal, u16)> = BOARD_MEMBER_SHARES.with(|b| {
            b.borrow().iter().collect()
        });
        all_members.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Use cumulative partitioning to calculate power without dust
        // Formula: power[i] = (VUC * cumulative_bps[0..i+1]) / BPS_TOTAL - (VUC * cumulative_bps[0..i]) / BPS_TOTAL
        let mut cumulative_bps: u32 = 0;
        let mut prev_boundary: u64 = 0;
        
        for (member, share_bps) in all_members {
            cumulative_bps += share_bps as u32;
            let current_boundary = (vuc as u128 * cumulative_bps as u128 / BPS_TOTAL as u128) as u64;
            let power = current_boundary - prev_boundary;
            
            if member == user {
                return Ok(power);
            }
            
            prev_boundary = current_boundary;
        }
        
        // Should not reach here if user is in the map
        return Err(format!("Board member {} not found in shares map", user));
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

/// Calculate all board member voting powers at once (for display/debugging)
/// Returns Vec of (Principal, share_bps, voting_power, is_sentinel)
pub async fn calculate_all_board_member_powers() -> Result<Vec<(Principal, u16, u64, bool)>, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Fetch VUC from staking hub
    let result: Result<(u64,), _> = ic_cdk::call(
        staking_hub_id,
        "get_vuc",
        (),
    ).await;
    
    let vuc = match result {
        Ok((v,)) => v,
        Err((code, msg)) => return Err(format!("Failed to fetch VUC: {:?} {}", code, msg)),
    };
    
    let mut powers: Vec<(Principal, u16, u64, bool)> = Vec::new();
    
    // Add sentinel first if set
    if let Some(sentinel) = get_sentinel_local() {
        powers.push((sentinel, 0, 1, true)); // 0 BPS, 1 unit power, is_sentinel = true
    }
    
    // Get all regular board members sorted
    let mut all_members: Vec<(Principal, u16)> = BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter().collect()
    });
    all_members.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Calculate using cumulative partitioning
    let mut cumulative_bps: u32 = 0;
    let mut prev_boundary: u64 = 0;
    
    for (member, share_bps) in all_members {
        cumulative_bps += share_bps as u32;
        let current_boundary = (vuc as u128 * cumulative_bps as u128 / BPS_TOTAL as u128) as u64;
        let power = current_boundary - prev_boundary;
        
        powers.push((member, share_bps, power, false));
        prev_boundary = current_boundary;
    }
    
    Ok(powers)
}
