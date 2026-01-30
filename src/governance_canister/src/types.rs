use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;
use candid::{Encode, Decode};

/// Token types for treasury spending
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    GHC,
    USDC,
    ICP,
}

/// Proposal status
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Proposed,    // Initial state for regular users, gathering support
    Active,      // Voting in progress
    Approved,    // Voting passed, pending execution
    Rejected,    // Voting failed
    Executed,    // Successfully executed
}

/// Proposal categories (for treasury proposals)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ProposalCategory {
    Marketing,
    Development,
    Partnership,
    Liquidity,
    CommunityGrant,
    Operations,
    Custom(String),
}

/// Type of proposal - determines what action is taken on execution
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    /// Treasury spending proposal - transfers tokens to recipient
    Treasury,
    /// Add a new board member with specified share in basis points
    AddBoardMember,
    /// Remove a board member and redistribute their share proportionally
    RemoveBoardMember,
    /// Update an existing board member's share in basis points
    UpdateBoardMemberShare,
    /// Update governance configuration (thresholds, approval percentage)
    UpdateGovernanceConfig,
    /// Add new content from staging (books, courses, etc.)
    AddContentFromStaging,
    /// Update global token limits and reward configuration
    UpdateTokenLimits,
    /// Delete a content node
    DeleteContentNode,
    /// Update the sentinel member (the member with 1 unit of voting power)
    UpdateSentinel,
}

/// Payload for AddBoardMember proposals
/// Uses Basis Points (BPS) where 10,000 BPS = 100.00%
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AddBoardMemberPayload {
    /// Wallet address of the new board member
    pub new_member: Principal,
    /// Share in basis points (1-9900, where 100 = 1.00%, 10000 = 100%)
    /// This share is taken proportionally from existing members (excluding sentinel)
    pub share_bps: u16,
}

/// Payload for RemoveBoardMember proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RemoveBoardMemberPayload {
    /// Wallet address of the board member to remove
    pub member_to_remove: Principal,
}

/// Payload for UpdateBoardMemberShare proposals
/// Uses Basis Points (BPS) where 10,000 BPS = 100.00%
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateBoardMemberSharePayload {
    /// Wallet address of the board member to update (cannot be sentinel)
    pub member: Principal,
    /// New share in basis points (1-9900, where 100 = 1.00%)
    /// The difference is taken from or distributed to other members proportionally
    pub new_share_bps: u16,
}

/// Payload for UpdateSentinel proposals
/// The sentinel is a special board member with exactly 1 unit of voting power
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateSentinelPayload {
    /// New wallet address for the sentinel role
    pub new_sentinel: Principal,
}

/// Payload for UpdateGovernanceConfig proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateGovernanceConfigPayload {
    /// New minimum voting power required to create a proposal (in e8s)
    pub new_min_voting_power: Option<u64>,
    /// New support threshold for moving proposals from Proposed to Active (in e8s)
    pub new_support_threshold: Option<u64>,
    /// New approval percentage (1-100) - percentage of total staked needed to pass
    pub new_approval_percentage: Option<u8>,
    
    // =========================================================================
    // Timing Configuration - Proposal Lifecycle Durations
    // =========================================================================
    
    /// New support period in days (time for proposals to gather support)
    pub new_support_period_days: Option<u16>,
    /// New voting period in days (duration for active voting)
    pub new_voting_period_days: Option<u16>,
    /// New resubmission cooldown in days (time before rejected proposal can be resubmitted)
    pub new_resubmission_cooldown_days: Option<u16>,
}

// ============================================================================
// CONTENT GOVERNANCE PAYLOADS
// ============================================================================


/// Payload for AddContentFromStaging proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AddContentFromStagingPayload {
    /// Principal of the staging canister
    pub staging_canister: Principal,
    /// Path in the staging canister
    pub staging_path: String,
    /// SHA256 hash for content verification
    pub content_hash: String,
    /// Human-readable title for the content
    pub content_title: String,
    /// Total number of units for progress tracking
    pub unit_count: u32,
}


#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TokenLimits {
    pub max_daily_tokens: u64,
    pub max_weekly_tokens: u64,
    pub max_monthly_tokens: u64,
    pub max_yearly_tokens: u64,
}

/// Payload for UpdateTokenLimits proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateTokenLimitsPayload {
    /// New reward amount for ALL quizzes (in e8s)
    pub new_reward_amount: Option<u64>,
    /// New pass threshold percentage
    pub new_pass_threshold: Option<u8>,
    /// New max daily attempts per quiz
    pub new_max_attempts: Option<u8>,
    
    // =========================================================================
    // Token Limits - Maximum tokens a user can earn in each time period
    // =========================================================================
    
    pub new_regular_limits: Option<TokenLimits>,
    pub new_subscribed_limits: Option<TokenLimits>,
}


/// Payload for DeleteContentNode proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DeleteContentNodePayload {
    /// ID of the content node to delete
    pub content_id: String,
    /// Reason for deletion
    pub reason: String,
}

/// Governance proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Principal,
    pub created_at: u64,
    pub voting_ends_at: u64,
    
    // Type of proposal
    pub proposal_type: ProposalType,
    
    // Common proposal details
    pub title: String,
    pub description: String,
    pub external_link: Option<String>,
    
    // Treasury-specific fields (only for Treasury proposals)
    pub recipient: Option<Principal>,
    pub amount: Option<u64>,
    pub token_type: Option<TokenType>,
    pub category: Option<ProposalCategory>,
    
    // Execution action (for Treasury + Action flow)
    pub execute_method: Option<String>,
    pub execute_payload: Option<Vec<u8>>,
    
    // Board member-specific fields
    pub board_member_payload: Option<AddBoardMemberPayload>,
    pub remove_board_member_payload: Option<RemoveBoardMemberPayload>,
    pub update_board_member_payload: Option<UpdateBoardMemberSharePayload>,
    
    // Governance configuration payload
    pub update_governance_config_payload: Option<UpdateGovernanceConfigPayload>,
    
    // Content governance payloads
    pub add_content_payload: Option<AddContentFromStagingPayload>,
    pub update_token_limits_payload: Option<UpdateTokenLimitsPayload>,
    pub delete_content_payload: Option<DeleteContentNodePayload>,
    
    // Sentinel payload
    pub update_sentinel_payload: Option<UpdateSentinelPayload>,
    
    // Voting state
    pub votes_yes: u64,
    pub votes_no: u64,
    pub voter_count: u64,
    pub board_member_yes_count: u32,
    
    // Support state (for Proposed phase)
    pub support_amount: u64,
    pub supporter_count: u64,
    
    /// Required YES votes for approval (calculated when proposal moves to Active)
    /// This is fixed at activation time: (total_staked * approval_percentage / 100)
    pub required_yes_votes: u64,
    
    pub status: ProposalStatus,
}

impl Storable for Proposal {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 10000, is_fixed_size: false };
}

/// Vote record for transparency
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VoteRecord {
    pub voter: Principal,
    pub proposal_id: u64,
    pub vote: bool,  // true = YES, false = NO
    pub voting_power: u64,
    pub timestamp: u64,
}

impl Storable for VoteRecord {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 200, is_fixed_size: false };
}

/// Support record to track who supported a proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SupportRecord {
    pub supporter: Principal,
    pub proposal_id: u64,
    pub support_amount: u64,
    pub timestamp: u64,
}

impl Storable for SupportRecord {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 200, is_fixed_size: false };
}

/// Composite key for vote storage: (proposal_id, voter)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VoteKey {
    pub proposal_id: u64,
    pub voter: Principal,
}

impl Storable for VoteKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut bytes = self.proposal_id.to_be_bytes().to_vec();
        bytes.extend_from_slice(self.voter.as_slice());
        Cow::Owned(bytes)
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let proposal_id = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let voter = Principal::from_slice(&bytes[8..]);
        Self { proposal_id, voter }
    }
    const BOUND: Bound = Bound::Bounded { max_size: 100, is_fixed_size: false };
}

/// Input for creating a treasury spending proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateTreasuryProposalInput {
    pub title: String,
    pub description: String,
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub category: ProposalCategory,
    pub external_link: Option<String>,
    pub execute_method: Option<String>,
    pub execute_payload: Option<Vec<u8>>,
}

/// Input for creating a board member proposal
/// Uses Basis Points (BPS) where 10,000 BPS = 100.00%
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateBoardMemberProposalInput {
    pub title: String,
    pub description: String,
    /// Wallet address of the new board member to add
    pub new_member: Principal,
    /// Share in basis points (1-9900, where 100 = 1.00%)
    /// This share is taken proportionally from existing members (excluding sentinel)
    pub share_bps: u16,
    pub external_link: Option<String>,
}

/// Board member share entry for query results and admin input
/// Uses Basis Points (BPS) where 10,000 BPS = 100.00%
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BoardMemberShare {
    pub member: Principal,
    /// Share in basis points (0-10000)
    pub share_bps: u16,
    /// Is this the sentinel member (1 unit of VUC, not BPS-based)
    pub is_sentinel: bool,
}

impl BoardMemberShare {}

/// Treasury transfer input (for calling treasury canister)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ExecuteTransferInput {
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub proposal_id: u64,
}

// ============================================================================
// CONTENT GOVERNANCE INPUT TYPES
// ============================================================================

/// Input for creating an AddContentFromStaging proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateAddContentProposalInput {
    pub title: String,
    pub description: String,
    pub staging_canister: Principal,
    pub staging_path: String,
    pub content_hash: String,
    pub content_title: String,
    pub unit_count: u32,
    pub external_link: Option<String>,
}

/// Input for creating an UpdateTokenLimits proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateUpdateTokenLimitsProposalInput {
    pub title: String,
    pub description: String,
    /// New reward amount for ALL quizzes (in e8s)
    pub new_reward_amount: Option<u64>,
    /// New pass threshold percentage
    pub new_pass_threshold: Option<u8>,
    /// New max daily attempts per quiz
    pub new_max_attempts: Option<u8>,
    pub new_regular_limits: Option<TokenLimits>,
    pub new_subscribed_limits: Option<TokenLimits>,
    pub external_link: Option<String>,
}

/// Input for creating a DeleteContentNode proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateDeleteContentProposalInput {
    pub title: String,
    pub description: String,
    /// ID of the content node to delete
    pub content_id: String,
    /// Reason for deletion
    pub reason: String,
    pub external_link: Option<String>,
}

// ============================================================================
// ADMIN GOVERNANCE INPUT TYPES
// ============================================================================

/// Input for creating a RemoveBoardMember proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateRemoveBoardMemberProposalInput {
    pub title: String,
    pub description: String,
    /// Wallet address of the board member to remove
    pub member_to_remove: Principal,
    pub external_link: Option<String>,
}

/// Input for creating an UpdateBoardMemberShare proposal
/// Uses Basis Points (BPS) where 10,000 BPS = 100.00%
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateUpdateBoardMemberShareProposalInput {
    pub title: String,
    pub description: String,
    /// Wallet address of the board member to update (cannot be sentinel)
    pub member: Principal,
    /// New share in basis points (1-9900, where 100 = 1.00%)
    pub new_share_bps: u16,
    pub external_link: Option<String>,
}

/// Input for creating an UpdateSentinel proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateUpdateSentinelProposalInput {
    pub title: String,
    pub description: String,
    /// New wallet address for the sentinel role
    pub new_sentinel: Principal,
    pub external_link: Option<String>,
}

/// Input for creating an UpdateGovernanceConfig proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateUpdateGovernanceConfigProposalInput {
    pub title: String,
    pub description: String,
    /// New minimum voting power required to create a proposal (in tokens, not e8s)
    pub new_min_voting_power: Option<u64>,
    /// New support threshold for moving proposals from Proposed to Active (in tokens, not e8s)
    pub new_support_threshold: Option<u64>,
    /// New approval percentage (1-100) - percentage of total staked needed to pass
    pub new_approval_percentage: Option<u8>,
    
    // =========================================================================
    // Timing Configuration - Proposal Lifecycle Durations  
    // =========================================================================
    
    /// New support period in days (time for proposals to gather support, 1-365)
    pub new_support_period_days: Option<u16>,
    /// New voting period in days (duration for active voting, 1-365)
    pub new_voting_period_days: Option<u16>,
    /// New resubmission cooldown in days (time before rejected proposal can be resubmitted, 1-365)
    pub new_resubmission_cooldown_days: Option<u16>,
    
    pub external_link: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub staking_hub_id: Principal,
    pub treasury_canister_id: Principal,
    pub learning_engine_id: Option<Principal>,
}

/// GlobalStats struct for querying staking hub
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct _GlobalStats {
    pub total_staked: u64,
    pub total_unstaked: u64,
    pub total_allocated: u64,
}
