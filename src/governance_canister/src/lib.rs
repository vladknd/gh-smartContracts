mod types;
mod state;
mod constants;

use ic_cdk::{init, query, update, post_upgrade};
use candid::Principal;
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

use types::*;
use state::*;
use constants::*;
mod service;
use service::*;

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    TREASURY_CANISTER_ID.with(|id| id.borrow_mut().set(args.treasury_canister_id).expect("Failed to set Treasury Canister ID"));
    if let Some(learning_engine_id) = args.learning_engine_id {
        LEARNING_ENGINE_ID.with(|id| id.borrow_mut().set(learning_engine_id).expect("Failed to set Learning Engine ID"));
    }
    
    start_timers();
}

#[post_upgrade]
fn post_upgrade() {
    start_timers();
}

fn start_timers() {
    // Proposal finalization timer (every hour)
    set_timer_interval(Duration::from_secs(60 * 60), || {
        finalize_expired_proposals();
    });
}


/// Get the voting power of a specific user
/// 
/// This is an update method because it may need to make inter-canister calls
/// to the staking hub to fetch VUC or staked balances.
#[update]
async fn get_user_voting_power(user: Principal) -> Result<u64, String> {
    fetch_voting_power(user).await
}

/// Get the voting power of the caller
#[update]
async fn get_my_voting_power() -> Result<u64, String> {
    fetch_voting_power(ic_cdk::caller()).await
}

// ============================================================================
// PROPOSAL CREATION
// ============================================================================

/// Create a treasury spending proposal
#[update]
async fn create_treasury_proposal(input: CreateTreasuryProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    if input.amount == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // Check if proposer is a board member (local check)
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power (anti-spam)
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Check treasury allowance (for GHC) - call treasury canister
    if input.token_type == TokenType::GHC {
        let treasury_id = TREASURY_CANISTER_ID.with(|id| *id.borrow().get());
        let (can_transfer,): (bool,) = ic_cdk::call(
            treasury_id,
            "can_transfer",
            (input.amount, input.token_type.clone())
        ).await.map_err(|e| format!("Failed to check allowance: {:?}", e))?;
        
        if !can_transfer {
            return Err("Amount exceeds treasury allowance".to_string());
        }
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    // Determine initial status and voting period
    // For board members, also calculate the required_yes_votes immediately
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        // Board members skip Proposed state, go directly to Active
        // Calculate required_yes_votes at this moment
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        // Regular users go to Proposed state with a support period deadline
        // required_yes_votes will be calculated when moving to Active
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::Treasury,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: Some(input.recipient),
        amount: Some(input.amount),
        token_type: Some(input.token_type),
        category: Some(input.category),
        execute_method: input.execute_method,
        execute_payload: input.execute_payload,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}


/// Create a proposal to add a new board member
/// 
/// This creates a proposal that, if approved, will:
/// 1. Add the specified wallet as a new board member
/// 2. Allocate the specified percentage to them
/// 3. Diminish existing board members' shares equally to accommodate the new percentage
#[update]
async fn create_board_member_proposal(input: CreateBoardMemberProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    if input.percentage == 0 || input.percentage > 99 {
        return Err("Percentage must be between 1 and 99".to_string());
    }
    
    // Check if the new member is already a board member (local check)
    if is_board_member_local(&input.new_member) {
        return Err("The specified address is already a board member".to_string());
    }
    
    // Check if proposer is a board member (local check)
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power (anti-spam)
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    // Determine initial status and voting period
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::AddBoardMember,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: Some(AddBoardMemberPayload {
            new_member: input.new_member,
            percentage: input.percentage,
        }),
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    

    Ok(id)
}

// ============================================================================
// ADMIN GOVERNANCE PROPOSAL CREATION
// ============================================================================

/// Create a proposal to remove a board member
/// Their share is redistributed equally among remaining members
#[update]
async fn create_remove_board_member_proposal(input: CreateRemoveBoardMemberProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    
    // Verify the member exists
    if !is_board_member_local(&input.member_to_remove) {
        return Err("The specified address is not a board member".to_string());
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::RemoveBoardMember,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: Some(RemoveBoardMemberPayload {
            member_to_remove: input.member_to_remove,
        }),
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

/// Create a proposal to update a board member's percentage share
#[update]
async fn create_update_board_member_share_proposal(input: CreateUpdateBoardMemberShareProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    if input.new_percentage == 0 || input.new_percentage > 99 {
        return Err("New percentage must be between 1 and 99".to_string());
    }
    
    // Verify the member exists
    if !is_board_member_local(&input.member) {
        return Err("The specified address is not a board member".to_string());
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::UpdateBoardMemberShare,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: Some(UpdateBoardMemberSharePayload {
            member: input.member,
            new_percentage: input.new_percentage,
        }),
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

/// Create a proposal to update governance configuration
/// (min voting power, support threshold, approval percentage, timing settings)
#[update]
async fn create_update_governance_config_proposal(input: CreateUpdateGovernanceConfigProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    
    // At least one config value must be specified
    if input.new_min_voting_power.is_none() 
        && input.new_support_threshold.is_none() 
        && input.new_approval_percentage.is_none()
        && input.new_support_period_days.is_none()
        && input.new_voting_period_days.is_none()
        && input.new_resubmission_cooldown_days.is_none() {
        return Err("At least one configuration value must be specified".to_string());
    }
    
    // Validate approval percentage if specified
    if let Some(pct) = input.new_approval_percentage {
        if pct < MIN_APPROVAL_PERCENTAGE || pct > MAX_APPROVAL_PERCENTAGE {
            return Err(format!(
                "Approval percentage must be between {} and {}",
                MIN_APPROVAL_PERCENTAGE, MAX_APPROVAL_PERCENTAGE
            ));
        }
    }
    
    // Validate timing values if specified (must be at least 1 day, max 365 days)
    if let Some(days) = input.new_support_period_days {
        if days == 0 || days > 365 {
            return Err("Support period must be between 1 and 365 days".to_string());
        }
    }
    if let Some(days) = input.new_voting_period_days {
        if days == 0 || days > 365 {
            return Err("Voting period must be between 1 and 365 days".to_string());
        }
    }
    if let Some(days) = input.new_resubmission_cooldown_days {
        if days == 0 || days > 365 {
            return Err("Resubmission cooldown must be between 1 and 365 days".to_string());
        }
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::UpdateGovernanceConfig,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: Some(UpdateGovernanceConfigPayload {
            new_min_voting_power: input.new_min_voting_power,
            new_support_threshold: input.new_support_threshold,
            new_approval_percentage: input.new_approval_percentage,
            new_support_period_days: input.new_support_period_days,
            new_voting_period_days: input.new_voting_period_days,
            new_resubmission_cooldown_days: input.new_resubmission_cooldown_days,
        }),
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

// ============================================================================
// CONTENT GOVERNANCE PROPOSAL CREATION
// ============================================================================

/// Create a proposal to add new content from staging canister
#[update]
async fn create_add_content_proposal(input: CreateAddContentProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    if input.unit_count == 0 {
        return Err("Unit count must be greater than 0".to_string());
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power (anti-spam)
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    // Determine initial status
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::AddContentFromStaging,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: Some(AddContentFromStagingPayload {
            staging_canister: input.staging_canister,
            staging_path: input.staging_path,
            content_hash: input.content_hash,
            content_title: input.content_title,
            unit_count: input.unit_count,
        }),
        update_token_limits_payload: None,
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

/// Create a proposal to update global token limits and reward configuration
#[update]
async fn create_update_token_limits_proposal(input: CreateUpdateTokenLimitsProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    
    // At least one config value must be specified
    if input.new_reward_amount.is_none() 
        && input.new_pass_threshold.is_none() 
        && input.new_max_attempts.is_none()
        && input.new_regular_limits.is_none()
        && input.new_subscribed_limits.is_none()
    {
        return Err("At least one configuration value must be specified".to_string());
    }
    
    if let Some(threshold) = input.new_pass_threshold {
        if threshold > 100 {
            return Err("Pass threshold cannot exceed 100%".to_string());
        }
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::UpdateTokenLimits,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: Some(UpdateTokenLimitsPayload {
            new_reward_amount: input.new_reward_amount,
            new_pass_threshold: input.new_pass_threshold,
            new_max_attempts: input.new_max_attempts,
            new_regular_limits: input.new_regular_limits,
            new_subscribed_limits: input.new_subscribed_limits,
        }),
        delete_content_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

/// Create a proposal to delete a content node
#[update]
async fn create_delete_content_proposal(input: CreateDeleteContentProposalInput) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Validate input
    if input.title.is_empty() || input.title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    if input.description.is_empty() || input.description.len() > 5000 {
        return Err("Description must be 1-5000 characters".to_string());
    }
    if input.content_id.is_empty() {
        return Err("Content ID is required".to_string());
    }
    if input.reason.is_empty() {
        return Err("Deletion reason is required".to_string());
    }
    
    // Check if proposer is a board member
    let proposer_is_board_member = is_board_member_local(&proposer);

    // Check voting power
    let voting_power = fetch_voting_power(proposer).await?;
    
    let min_power = get_min_voting_power_to_propose();
    if voting_power < min_power {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            min_power / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Create proposal
    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });
    
    let (status, voting_ends_at, required_yes_votes) = if proposer_is_board_member {
        let threshold = calculate_approval_threshold().await?;
        (ProposalStatus::Active, now + get_voting_period(), threshold)
    } else {
        (ProposalStatus::Proposed, now + get_support_period(), 0)
    };
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at,
        proposal_type: ProposalType::DeleteContentNode,
        title: input.title,
        description: input.description,
        external_link: input.external_link,
        recipient: None,
        amount: None,
        token_type: None,
        category: None,
        board_member_payload: None,
        remove_board_member_payload: None,
        update_board_member_payload: None,
        update_governance_config_payload: None,
        add_content_payload: None,
        update_token_limits_payload: None,
        delete_content_payload: Some(DeleteContentNodePayload {
            content_id: input.content_id,
            reason: input.reason,
        }),
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        required_yes_votes,
        status,
        execute_method: None,
        execute_payload: None,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

#[update]
async fn support_proposal(proposal_id: u64) -> Result<(), String> {
    let supporter = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Get proposal
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
        
    // Check status
    if proposal.status != ProposalStatus::Proposed {
        return Err("Proposal is not in Proposed state".to_string());
    }
    
    // Check if already supported
    let vote_key = VoteKey { proposal_id, voter: supporter };
    if SUPPORT_RECORDS.with(|r| r.borrow().contains_key(&vote_key)) {
        return Err("Already supported this proposal".to_string());
    }
    
    // Get voting power
    let voting_power = fetch_voting_power(supporter).await?;
    
    if voting_power == 0 {
        return Err("No voting power".to_string());
    }
    
    // Record support
    let record = SupportRecord {
        supporter,
        proposal_id,
        support_amount: voting_power,
        timestamp: now,
    };
    SUPPORT_RECORDS.with(|r| r.borrow_mut().insert(vote_key, record));
    
    // Update proposal
    proposal.support_amount += voting_power;
    proposal.supporter_count += 1;
    
    // Check threshold (configurable VP and 2 users)
    let support_threshold = get_support_threshold();
    
    if proposal.support_amount >= support_threshold && proposal.supporter_count >= 2 {
        // Transition to Active - calculate and fix the required_yes_votes at this moment
        let required_votes = calculate_approval_threshold().await?;
        proposal.required_yes_votes = required_votes;
        proposal.status = ProposalStatus::Active;
        proposal.voting_ends_at = now + get_voting_period();
    }
    
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}

// ============================================================================
// VOTING
// ============================================================================

#[update]
async fn vote(proposal_id: u64, approve: bool) -> Result<(), String> {
    let voter = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Get proposal
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    // Check proposal is active
    if proposal.status != ProposalStatus::Active {
        return Err("Proposal is not active".to_string());
    }
    
    // Check voting period
    if now > proposal.voting_ends_at {
        return Err("Voting period has ended".to_string());
    }
    
    // Check if already voted
    let vote_key = VoteKey { proposal_id, voter };
    if VOTE_RECORDS.with(|v| v.borrow().contains_key(&vote_key)) {
        return Err("Already voted on this proposal".to_string());
    }
    
    // Get voting power
    let voting_power = fetch_voting_power(voter).await?;
    
    if voting_power == 0 {
        return Err("No voting power".to_string());
    }
    
    // Record vote
    let vote_record = VoteRecord {
        voter,
        proposal_id,
        vote: approve,
        voting_power,
        timestamp: now,
    };
    VOTE_RECORDS.with(|v| v.borrow_mut().insert(vote_key, vote_record));
    
    // Update proposal
    if approve {
        proposal.votes_yes += voting_power;
    } else {
        proposal.votes_no += voting_power;
    }
    proposal.voter_count += 1;
    
    // Threshold check is handled in finalize_proposal, which allows early execution.
    // We do not change status here to avoid blocking finalize_proposal (which requires Active status).
    
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}

// ============================================================================
// PROPOSAL FINALIZATION
// ============================================================================

/// Finalize proposals whose voting or support period has ended
fn finalize_expired_proposals() {
    let now = ic_cdk::api::time();
    
    let proposals_to_finalize: Vec<u64> = PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .filter(|(_, prop)| {
                // Active proposals that have ended voting period
                (prop.status == ProposalStatus::Active && now > prop.voting_ends_at) ||
                // Proposed proposals that have ended support period
                (prop.status == ProposalStatus::Proposed && now > prop.voting_ends_at)
            })
            .map(|(id, _)| id)
            .collect()
    });
    
    for id in proposals_to_finalize {
        let _ = finalize_proposal(id);
    }
}

// finalize_proposal uses the stored required_yes_votes that was fixed at activation time
#[update]
fn finalize_proposal(proposal_id: u64) -> Result<ProposalStatus, String> {
    let now = ic_cdk::api::time();
    
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    // Check if already finalized
    if proposal.status == ProposalStatus::Executed || proposal.status == ProposalStatus::Rejected || proposal.status == ProposalStatus::Approved {
        return Ok(proposal.status);
    }
    
    // Handle Proposed state - reject if support period expired
    if proposal.status == ProposalStatus::Proposed {
        if now > proposal.voting_ends_at {
            proposal.status = ProposalStatus::Rejected;
            PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal.clone()));
        } else {
            return Err("Support period not ended yet".to_string());
        }
    }
    
    // Handle Active state - approve or reject based on votes
    if proposal.status == ProposalStatus::Active {
        // Use the required_yes_votes that was fixed when proposal moved to Active
        let approval_threshold = proposal.required_yes_votes;
        
        // Check voting period ended
        // We allow early finalization if the approval threshold is met, enabling "Fast Track" execution.
        if now <= proposal.voting_ends_at && proposal.votes_yes < approval_threshold {
             return Err(format!(
                 "Voting period not ended yet. Current Yes votes: {}, Required: {}",
                 proposal.votes_yes / 100_000_000, 
                 approval_threshold / 100_000_000
             ));
        }
        
        // Determine outcome
        if proposal.votes_yes >= approval_threshold {
            proposal.status = ProposalStatus::Approved;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }
        PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal.clone()));
    }
    
    Ok(proposal.status)
}

#[update]
async fn execute_proposal(proposal_id: u64) -> Result<(), String> {
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
        
    if proposal.status != ProposalStatus::Approved {
        return Err("Proposal is not Approved".to_string());
    }
    
    // Execute based on proposal type
    match proposal.proposal_type {
        ProposalType::Treasury => execute_treasury_proposal_internal(&proposal).await?,
        ProposalType::AddBoardMember => execute_board_member_proposal_internal(&proposal)?,
        ProposalType::RemoveBoardMember => execute_remove_board_member_proposal_internal(&proposal)?,
        ProposalType::UpdateBoardMemberShare => execute_update_board_member_share_proposal_internal(&proposal)?,
        ProposalType::UpdateGovernanceConfig => execute_update_governance_config_proposal_internal(&proposal)?,
        ProposalType::AddContentFromStaging => execute_add_content_proposal_internal(&proposal).await?,
        ProposalType::UpdateTokenLimits => execute_update_token_limits_proposal_internal(&proposal).await?,
        ProposalType::DeleteContentNode => execute_delete_content_proposal_internal(&proposal).await?,
    }
    
    let mut proposal = proposal; // Get a mutable copy
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}

/// Execute a treasury spending proposal by calling the treasury canister
async fn execute_treasury_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let token_type = proposal.token_type.as_ref()
        .ok_or("Treasury proposal missing token_type")?;
    let amount = proposal.amount
        .ok_or("Treasury proposal missing amount")?;
    let recipient = proposal.recipient
        .ok_or("Treasury proposal missing recipient")?;
    
    let treasury_id = TREASURY_CANISTER_ID.with(|id| *id.borrow().get());
    
    let transfer_input = ExecuteTransferInput {
        recipient,
        amount,
        token_type: token_type.clone(),
        proposal_id: proposal.id,
    };
    
    let (result,): (Result<u64, String>,) = ic_cdk::call(
        treasury_id,
        "execute_transfer",
        (transfer_input,)
    ).await.map_err(|(code, msg)| format!("Treasury call failed: {:?} {}", code, msg))?;
    
    let _transfer_result_idx = match result {
        Ok(idx) => idx,
        Err(e) => return Err(e),
    };
    
    // If successful and execute_method is present, call the recipient
    if let Some(method) = &proposal.execute_method {
        let payload = proposal.execute_payload.clone().unwrap_or_default();
        
        let exec_result: Result<(Result<(), String>,), _> = ic_cdk::call(
            recipient,
            method,
            (payload,)
        ).await;
        
        // We log the error but do not fail the whole proposal execution state
        // because the funds have already been transferred.
        // In a more advanced system, we might want to store this result in the proposal history.
        if let Err((code, msg)) = exec_result {
            ic_cdk::print(format!("Failed to execute action on recipient: {:?} {}", code, msg));
        }
    }

    Ok(())
}

// ============================================================================
// CONTENT GOVERNANCE EXECUTION
// ============================================================================

/// Execute AddContentFromStaging proposal
async fn execute_add_content_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.add_content_payload.as_ref()
        .ok_or("AddContentFromStaging proposal missing payload")?;
    
    let learning_engine_id = LEARNING_ENGINE_ID.with(|id| *id.borrow().get());
    
    if learning_engine_id == Principal::anonymous() {
        return Err("Learning engine ID not configured".to_string());
    }
    
    // First, mark content as loading in staging canister
    let staging_result: Result<(Result<(), String>,), _> = ic_cdk::call(
        payload.staging_canister,
        "mark_loading",
        (payload.content_hash.clone(),)
    ).await;
    
    if let Err((code, msg)) = staging_result {
        return Err(format!("Failed to mark staging content as loading: {:?} {}", code, msg));
    }
    
    // Call learning_engine.start_content_load
    let result: Result<(Result<(), String>,), _> = ic_cdk::call(
        learning_engine_id,
        "start_content_load",
        (
            proposal.id,
            payload.staging_canister,
            payload.content_hash.clone(),  // Use content_hash as the path
            payload.content_hash.clone(),
            payload.unit_count,
        )
    ).await;
    
    match result {
        Ok((inner_result,)) => inner_result,
        Err((code, msg)) => Err(format!("Learning engine call failed: {:?} {}", code, msg)),
    }
}


/// Execute UpdateTokenLimits proposal
async fn execute_update_token_limits_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.update_token_limits_payload.as_ref()
        .ok_or("UpdateTokenLimits proposal missing payload")?;
    
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    if staking_hub_id == Principal::anonymous() {
        return Err("Staking Hub ID not configured".to_string());
    }
    
    // Call staking_hub.update_token_limits
    let result: Result<(Result<(), String>,), _> = ic_cdk::call(
        staking_hub_id,
        "update_token_limits",
        (
            payload.new_reward_amount,
            payload.new_pass_threshold,
            payload.new_max_attempts,
            payload.new_regular_limits.clone(),
            payload.new_subscribed_limits.clone(),
        )
    ).await;
    
    match result {
        Ok((inner_result,)) => inner_result,
        Err((code, msg)) => Err(format!("Staking Hub call failed: {:?} {}", code, msg)),
    }
}


/// Execute DeleteContentNode proposal
async fn execute_delete_content_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.delete_content_payload.as_ref()
        .ok_or("DeleteContentNode proposal missing payload")?;
    
    let learning_engine_id = LEARNING_ENGINE_ID.with(|id| *id.borrow().get());
    
    if learning_engine_id == Principal::anonymous() {
        return Err("Learning engine ID not configured".to_string());
    }
    
    // Call learning_engine.delete_content_node
    let result: Result<(Result<(), String>,), _> = ic_cdk::call(
        learning_engine_id,
        "delete_content_node",
        (
            payload.content_id.clone(),
            proposal.id,
        )
    ).await;
    
    match result {
        Ok((inner_result,)) => inner_result,
        Err((code, msg)) => Err(format!("Learning engine call failed: {:?} {}", code, msg)),
    }
}

/// Execute a board member addition proposal
/// 
/// This function:
/// 1. Gets the current board member shares
/// 2. Calculates proportional reduction for each existing member
/// 3. Adds the new member with their allocated percentage
fn execute_board_member_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.board_member_payload.as_ref()
        .ok_or("Board member proposal missing payload")?;
    
    // Validate percentage
    if payload.percentage == 0 || payload.percentage > 99 {
        return Err("Percentage must be between 1 and 99".to_string());
    }
    
    // Check if already a board member
    if BOARD_MEMBER_SHARES.with(|b| b.borrow().contains_key(&payload.new_member)) {
        return Err("Address is already a board member".to_string());
    }
    
    // Get current board members and their shares
    let current_shares: Vec<(Principal, u8)> = BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter().collect()
    });
    
    if current_shares.is_empty() {
        return Err("No existing board members to redistribute from".to_string());
    }
    
    // Calculate new shares for existing members using the Largest Remainder Method
    let remaining_percentage = 100 - payload.percentage;
    let mut new_shares: Vec<(Principal, u8)> = Vec::new();
    
    // 1. Calculate the exact portion for each member (floor + remainder)
    let mut distribution: Vec<(Principal, u8, u16)> = Vec::new();
    let mut distributed_total: u16 = 0;
    
    for (member, old_share) in current_shares.iter() {
        let raw_value = (*old_share as u16) * (remaining_percentage as u16);
        let floor = (raw_value / 100) as u8;
        let remainder = raw_value % 100;
        
        distribution.push((*member, floor, remainder));
        distributed_total += floor as u16;
    }
    
    // 2. Distribute the remaining points to those with the largest remainders
    let points_needed = (remaining_percentage as u16).saturating_sub(distributed_total);
    
    // Sort by remainder descending, then by Principal for determinism
    distribution.sort_by(|a, b| {
        b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0))
    });
    
    // 3. Assign final shares
    for (i, (member, floor, _)) in distribution.iter().enumerate() {
        let extra = if i < points_needed as usize { 1 } else { 0 };
        new_shares.push((*member, floor + extra));
    }
    
    // Add the new member
    new_shares.push((payload.new_member, payload.percentage));
    
    // Verify total is exactly 100
    let total: u16 = new_shares.iter().map(|(_, p)| *p as u16).sum();
    if total != 100 {
        // Adjust the largest share to make total exactly 100
        let diff = total as i16 - 100;
        if diff > 0 {
            // Need to reduce by diff
            let max_idx = new_shares.iter()
                .enumerate()
                .max_by_key(|(_, (_, p))| *p)
                .map(|(i, _)| i)
                .unwrap_or(0);
            new_shares[max_idx].1 = new_shares[max_idx].1.saturating_sub(diff as u8);
        } else {
            // Need to increase by -diff
            let max_idx = new_shares.iter()
                .enumerate()
                .max_by_key(|(_, (_, p))| *p)
                .map(|(i, _)| i)
                .unwrap_or(0);
            new_shares[max_idx].1 = new_shares[max_idx].1.saturating_add((-diff) as u8);
        }
    }
    
    // Update the shares atomically
    BOARD_MEMBER_SHARES.with(|b| {
        let mut map = b.borrow_mut();
        
        // Clear all existing entries
        let existing_keys: Vec<Principal> = map.iter().map(|(k, _)| k).collect();
        for key in existing_keys {
            map.remove(&key);
        }
        
        // Insert new shares
        for (member, share) in new_shares {
            map.insert(member, share);
        }
    });
    
    Ok(())
}

/// Execute a board member removal proposal
/// 
/// When removing a board member, their percentage share is distributed
/// equally among the remaining board members.
fn execute_remove_board_member_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.remove_board_member_payload.as_ref()
        .ok_or("RemoveBoardMember proposal missing payload")?;
    
    // Check if the member exists
    let member_share_opt = BOARD_MEMBER_SHARES.with(|b| b.borrow().get(&payload.member_to_remove));
    if member_share_opt.is_none() {
        return Err("Board member not found".to_string());
    }
    
    // Get remaining board members (excluding the one to remove)
    let remaining_members: Vec<(Principal, u8)> = BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter()
            .filter(|(member, _)| *member != payload.member_to_remove)
            .collect()
    });
    
    if remaining_members.is_empty() {
        return Err("Cannot remove the last board member".to_string());
    }
    
    // Proportional Redistribution Logic
    // Goal: Scale existing shares up so they total 100, maintaining relative ratios.
    // Formula: NewShare = OldShare * (100 / OldTotal)
    
    let current_total: u16 = remaining_members.iter().map(|(_, s)| *s as u16).sum();
    // Sanity check: This should be (100 - share_removed), but we calculate from source
    
    if current_total == 0 {
        // Edge case: multiple members but all have 0%? Should not happen if strictly enforced 1-100.
        return Err("Remaining members have 0 total share".to_string());
    }
    
    // 1. Calculate ideal values and remainders
    let mut distribution: Vec<(Principal, u8, u32)> = Vec::new(); // (Principal, Floor, Remainder)
    let mut floor_total: u16 = 0;
    
    for (member, current_share) in remaining_members.iter() {
        // We want (current_share * 100) / current_total
        // Calculate with higher precision for remainder
        let value_x1000 = (*current_share as u32 * 100 * 1000) / current_total as u32;
        let floor = (value_x1000 / 1000) as u8;
        let remainder = value_x1000 % 1000;
        
        distribution.push((*member, floor, remainder));
        floor_total += floor as u16;
    }
    
    // 2. Distribute points to those with largest remainders
    let points_needed = 100_u16.saturating_sub(floor_total);
    
    // Sort: Remainder DESC, then Member ASC
    distribution.sort_by(|a, b| {
        b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0))
    });
    
    let mut new_shares: Vec<(Principal, u8)> = Vec::new();
    
    for (i, (member, floor, _)) in distribution.iter().enumerate() {
        let extra = if i < points_needed as usize { 1 } else { 0 };
        new_shares.push((*member, floor + extra));
    }
    
    // Update the shares atomically
    BOARD_MEMBER_SHARES.with(|b| {
        let mut map = b.borrow_mut();
        
        // Clear all existing entries
        let existing_keys: Vec<Principal> = map.iter().map(|(k, _)| k).collect();
        for key in existing_keys {
            map.remove(&key);
        }
        
        // Insert new shares
        for (member, share) in new_shares {
            map.insert(member, share);
        }
    });
    
    Ok(())
}

/// Execute an update board member share proposal
/// 
/// Adjusts the specified board member's share to the new value,
/// redistributing the difference across other members proportionally.
fn execute_update_board_member_share_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.update_board_member_payload.as_ref()
        .ok_or("UpdateBoardMemberShare proposal missing payload")?;
    
    // Validate new percentage
    if payload.new_percentage == 0 || payload.new_percentage > 99 {
        return Err("New percentage must be between 1 and 99".to_string());
    }
    
    // Get current share
    let current_share = BOARD_MEMBER_SHARES.with(|b| b.borrow().get(&payload.member))
        .ok_or("Board member not found")?;
    
    if payload.new_percentage == current_share {
        return Ok(()); // No change needed
    }
    
    // Get all other board members
    let other_members: Vec<(Principal, u8)> = BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter()
            .filter(|(member, _)| *member != payload.member)
            .collect()
    });
    
    if other_members.is_empty() {
        return Err("Cannot update share when there's only one board member".to_string());
    }
    
    // Calculate the difference (unused here but kept for clarity)
    let _diff = payload.new_percentage as i16 - current_share as i16;
    let remaining_for_others = (100 - payload.new_percentage) as u16;
    let current_others_total: u16 = other_members.iter().map(|(_, p)| *p as u16).sum();
    
    // Redistribute among other members proportionally
    let mut new_shares: Vec<(Principal, u8)> = Vec::new();
    let mut distributed_total: u16 = 0;
    
    for (i, (member, old_share)) in other_members.iter().enumerate() {
        let new_share = if i == other_members.len() - 1 {
            // Last member gets the remainder
            remaining_for_others - distributed_total
        } else {
            let proportion = (*old_share as u128 * remaining_for_others as u128) / current_others_total as u128;
            proportion as u16
        };
        
        new_shares.push((*member, new_share as u8));
        distributed_total += new_share;
    }
    
    // Add the updated member
    new_shares.push((payload.member, payload.new_percentage));
    
    // Update the shares atomically
    BOARD_MEMBER_SHARES.with(|b| {
        let mut map = b.borrow_mut();
        
        let existing_keys: Vec<Principal> = map.iter().map(|(k, _)| k).collect();
        for key in existing_keys {
            map.remove(&key);
        }
        
        for (member, share) in new_shares {
            map.insert(member, share);
        }
    });
    
    Ok(())
}

/// Execute an update governance config proposal
/// 
/// Updates the mutable governance configuration:
/// - min_voting_power: minimum voting power to create proposals
/// - support_threshold: voting power needed to move from Proposed to Active
/// - approval_percentage: percentage of total staked needed for YES votes to pass
/// - support_period_days: time for proposals to gather support
/// - voting_period_days: duration for active voting
/// - resubmission_cooldown_days: time before rejected proposal can be resubmitted
fn execute_update_governance_config_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.update_governance_config_payload.as_ref()
        .ok_or("UpdateGovernanceConfig proposal missing payload")?;
    
    // Validate: at least one value must be specified
    if payload.new_min_voting_power.is_none() 
        && payload.new_support_threshold.is_none() 
        && payload.new_approval_percentage.is_none()
        && payload.new_support_period_days.is_none()
        && payload.new_voting_period_days.is_none()
        && payload.new_resubmission_cooldown_days.is_none() {
        return Err("At least one configuration value must be specified".to_string());
    }
    
    // Validate approval percentage if specified
    if let Some(pct) = payload.new_approval_percentage {
        if pct < MIN_APPROVAL_PERCENTAGE || pct > MAX_APPROVAL_PERCENTAGE {
            return Err(format!(
                "Approval percentage must be between {} and {}",
                MIN_APPROVAL_PERCENTAGE, MAX_APPROVAL_PERCENTAGE
            ));
        }
    }
    
    // Validate timing values if specified
    if let Some(days) = payload.new_support_period_days {
        if days == 0 || days > 365 {
            return Err("Support period must be between 1 and 365 days".to_string());
        }
    }
    if let Some(days) = payload.new_voting_period_days {
        if days == 0 || days > 365 {
            return Err("Voting period must be between 1 and 365 days".to_string());
        }
    }
    if let Some(days) = payload.new_resubmission_cooldown_days {
        if days == 0 || days > 365 {
            return Err("Resubmission cooldown must be between 1 and 365 days".to_string());
        }
    }
    
    // Apply the updates
    if let Some(min_power) = payload.new_min_voting_power {
        // Convert from tokens to e8s
        let value_e8s = min_power * 100_000_000;
        MIN_VOTING_POWER_CONFIG.with(|c| {
            c.borrow_mut().set(value_e8s).expect("Failed to update min voting power");
        });
    }
    
    if let Some(threshold) = payload.new_support_threshold {
        // Convert from tokens to e8s
        let value_e8s = threshold * 100_000_000;
        SUPPORT_THRESHOLD_CONFIG.with(|c| {
            c.borrow_mut().set(value_e8s).expect("Failed to update support threshold");
        });
    }
    
    if let Some(pct) = payload.new_approval_percentage {
        APPROVAL_PERCENTAGE_CONFIG.with(|c| {
            c.borrow_mut().set(pct).expect("Failed to update approval percentage");
        });
    }
    
    // Apply timing updates (convert days to nanoseconds)
    if let Some(days) = payload.new_support_period_days {
        let nanos = (days as u64) * NANOS_PER_DAY;
        SUPPORT_PERIOD_CONFIG.with(|c| {
            c.borrow_mut().set(nanos).expect("Failed to update support period");
        });
    }
    
    if let Some(days) = payload.new_voting_period_days {
        let nanos = (days as u64) * NANOS_PER_DAY;
        VOTING_PERIOD_CONFIG.with(|c| {
            c.borrow_mut().set(nanos).expect("Failed to update voting period");
        });
    }
    
    if let Some(days) = payload.new_resubmission_cooldown_days {
        let nanos = (days as u64) * NANOS_PER_DAY;
        RESUBMISSION_COOLDOWN_CONFIG.with(|c| {
            c.borrow_mut().set(nanos).expect("Failed to update resubmission cooldown");
        });
    }
    
    Ok(())
}


// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

#[query]
fn get_proposal(id: u64) -> Option<Proposal> {
    PROPOSALS.with(|p| p.borrow().get(&id))
}

#[query]
fn get_active_proposals() -> Vec<Proposal> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .filter(|(_, prop)| prop.status == ProposalStatus::Active)
            .map(|(_, prop)| prop)
            .collect()
    })
}

#[query]
fn get_all_proposals() -> Vec<Proposal> {
    PROPOSALS.with(|p| {
        p.borrow().iter().map(|(_, prop)| prop).collect()
    })
}

#[query]
fn get_proposal_supporters(proposal_id: u64) -> Vec<SupportRecord> {
    SUPPORT_RECORDS.with(|r| {
        r.borrow()
            .iter()
            .filter(|(key, _)| key.proposal_id == proposal_id)
            .map(|(_, record)| record)
            .collect()
    })
}

#[query]
fn get_proposal_votes(proposal_id: u64) -> Vec<VoteRecord> {
    VOTE_RECORDS.with(|v| {
        v.borrow()
            .iter()
            .filter(|(key, _)| key.proposal_id == proposal_id)
            .map(|(_, record)| record)
            .collect()
    })
}

#[query]
fn has_voted(proposal_id: u64, voter: Principal) -> bool {
    let vote_key = VoteKey { proposal_id, voter };
    VOTE_RECORDS.with(|v| v.borrow().contains_key(&vote_key))
}

#[query]
fn get_governance_config() -> (u64, u64, u64, u64, u64, u8) {
    (
        get_min_voting_power_to_propose() / 100_000_000, // In tokens
        get_support_threshold() / 100_000_000,           // In tokens
        get_support_period() / NANOS_PER_DAY,            // In days
        get_voting_period() / NANOS_PER_DAY,             // In days
        get_resubmission_cooldown() / NANOS_PER_DAY,     // In days
        get_approval_percentage(), // Approval percentage (1-100)
    )
}

// ============================================================================
// BOARD MEMBER MANAGEMENT
// ============================================================================

/// Set all board member shares atomically (admin only)
/// 
/// This replaces ALL existing board members with the new list.
/// Total percentages must equal exactly 100.
/// Cannot be called if shares are locked.
#[update]
fn set_board_member_shares(shares: Vec<BoardMemberShare>) -> Result<(), String> {
    // Only controllers can set shares
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can set board member shares".to_string());
    }
    
    // Check if locked
    let is_locked = BOARD_SHARES_LOCKED.with(|l| *l.borrow().get());
    if is_locked {
        return Err("Board member shares are locked. Use governance proposals to add new members.".to_string());
    }
    
    // Validate: no empty list
    if shares.is_empty() {
        return Err("Must have at least one board member".to_string());
    }
    
    // Validate: no duplicates
    let mut seen = std::collections::HashSet::new();
    for share in &shares {
        if !seen.insert(share.member) {
            return Err(format!("Duplicate member: {}", share.member));
        }
    }
    
    // Validate: each percentage is 1-100
    for share in &shares {
        if share.percentage == 0 || share.percentage > 100 {
            return Err(format!(
                "Invalid percentage {} for {}. Must be 1-100.",
                share.percentage, share.member
            ));
        }
    }
    
    // Validate: total equals 100
    let total: u16 = shares.iter().map(|s| s.percentage as u16).sum();
    if total != 100 {
        return Err(format!(
            "Total percentages must equal 100. Got: {}",
            total
        ));
    }
    
    // Clear existing and insert new
    BOARD_MEMBER_SHARES.with(|b| {
        let mut map = b.borrow_mut();
        
        // Clear all existing entries
        let existing_keys: Vec<Principal> = map.iter().map(|(k, _)| k).collect();
        for key in existing_keys {
            map.remove(&key);
        }
        
        // Insert new shares
        for share in shares {
            map.insert(share.member, share.percentage);
        }
    });
    
    Ok(())
}

/// Lock board member shares (admin only)
/// 
/// Once locked, shares can only be modified via governance proposals (AddBoardMember).
#[update]
fn lock_board_member_shares() -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can lock board member shares".to_string());
    }
    
    // Verify shares are set before locking
    let total: u16 = BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter().map(|(_, pct)| pct as u16).sum()
    });
    
    if total != 100 {
        return Err(format!(
            "Cannot lock: Board member shares must total 100%. Current total: {}",
            total
        ));
    }
    
    BOARD_SHARES_LOCKED.with(|l| {
        l.borrow_mut().set(true).expect("Failed to lock board member shares")
    });
    
    Ok(())
}

/// Check if board member shares are locked
#[query]
fn are_board_shares_locked() -> bool {
    BOARD_SHARES_LOCKED.with(|l| *l.borrow().get())
}

/// Get all board members with their voting power percentages
#[query]
fn get_board_member_shares() -> Vec<BoardMemberShare> {
    BOARD_MEMBER_SHARES.with(|b| {
        b.borrow().iter().map(|(member, percentage)| {
            BoardMemberShare { member, percentage }
        }).collect()
    })
}

/// Get a specific board member's percentage
#[query]
fn get_board_member_share(principal: Principal) -> Option<u8> {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().get(&principal))
}

/// Get number of board members
#[query]
fn get_board_member_count() -> u64 {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().len())
}

/// Check if a principal is a board member
#[query]
fn is_board_member(principal: Principal) -> bool {
    is_board_member_local(&principal)
}

// ============================================================================
// CONFIGURATION QUERIES
// ============================================================================

#[query]
fn get_treasury_canister_id() -> Principal {
    TREASURY_CANISTER_ID.with(|id| *id.borrow().get())
}

#[query]
fn get_staking_hub_id() -> Principal {
    STAKING_HUB_ID.with(|id| *id.borrow().get())
}

#[query]
fn get_learning_engine_id() -> Principal {
    LEARNING_ENGINE_ID.with(|id| *id.borrow().get())
}

/// Set learning engine canister ID (controller only)
#[update]
fn set_learning_engine_id(new_id: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can update learning engine ID".to_string());
    }
    
    LEARNING_ENGINE_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set learning engine ID")
    });
    
    Ok(())
}

// ============================================================================
// ADMIN DEBUG FUNCTIONS (For Testing Only)
// ============================================================================

/// Force expire a proposal (set voting end time to past)
/// This allows "fast-forwarding" time for a single proposal.
#[update]
fn admin_expire_proposal(proposal_id: u64) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    let now = ic_cdk::api::time();
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    // Set End Time to 1 nanosecond ago    
    proposal.voting_ends_at = now.saturating_sub(1);
    
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    Ok(())
}

/// Force set a proposal's status
/// Useful for testing execution without gathering votes
#[update]
fn admin_set_proposal_status(proposal_id: u64, status: ProposalStatus) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
        
    proposal.status = status;
    
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    Ok(())
}

/// Set treasury canister ID (controller only)
#[update]
fn set_treasury_canister_id(new_id: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can update treasury canister ID".to_string());
    }
    
    TREASURY_CANISTER_ID.with(|id| {
        id.borrow_mut().set(new_id).expect("Failed to set treasury canister ID")
    });
    
    Ok(())
}

ic_cdk::export_candid!();
