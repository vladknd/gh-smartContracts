use ic_cdk::{init, query, update, post_upgrade};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use candid::Nat;
use std::borrow::Cow;
use candid::{Encode, Decode};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// GOVERNANCE CONSTANTS
// ============================================================================

/// Minimum voting power required to create a proposal
const MIN_VOTING_POWER_TO_PROPOSE: u64 = 150 * 100_000_000; // 150 tokens in e8s

/// Minimum YES votes required for proposal approval
const APPROVAL_THRESHOLD: u64 = 15_000 * 100_000_000; // 15,000 tokens in e8s

/// Support period for proposals in Proposed state: 1 week in nanoseconds
const SUPPORT_PERIOD_NANOS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;

/// Voting period duration: 2 weeks in nanoseconds
const VOTING_PERIOD_NANOS: u64 = 14 * 24 * 60 * 60 * 1_000_000_000;

/// Cooldown before a rejected proposal can be resubmitted: 6 months in nanoseconds
const RESUBMISSION_COOLDOWN_NANOS: u64 = 180 * 24 * 60 * 60 * 1_000_000_000;

// ============================================================================
// TREASURY CONSTANTS (in e8s - 8 decimals)
// ============================================================================

const INITIAL_TREASURY_BALANCE: u64 = 425_000_000_000_000_000; // 4.25B * 10^8
const INITIAL_TREASURY_ALLOWANCE: u64 = 60_000_000_000_000_000; // 0.6B * 10^8
const MMCR_AMOUNT: u64 = 1_520_000_000_000_000; // 15.2M * 10^8
const FINAL_MMCR_AMOUNT: u64 = 1_720_000_000_000_000; // 17.2M * 10^8
const TOTAL_MMCR_RELEASES: u64 = 240;
const MMCR_MIN_INTERVAL_NANOS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Treasury state - tracks balance and allowance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    pub balance: u64,
    pub allowance: u64,
    pub total_transferred: u64,
    pub mmcr_count: u64,
    pub last_mmcr_timestamp: u64,
    pub genesis_timestamp: u64,
}

impl Storable for TreasuryState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 200, is_fixed_size: false };
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

/// Token types for treasury spending
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    GHC,
    USDC,
    ICP,
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
    /// Add a new board member with specified percentage share
    AddBoardMember,
}

/// Payload for AddBoardMember proposals
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AddBoardMemberPayload {
    /// Wallet address of the new board member
    pub new_member: Principal,
    /// Percentage share to allocate to the new member (1-99)
    /// This percentage is taken equally from all existing members
    pub percentage: u8,
}

/// Treasury spending proposal
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
    
    // Board member-specific fields (only for AddBoardMember proposals)
    pub board_member_payload: Option<AddBoardMemberPayload>,
    
    // Voting state
    pub votes_yes: u64,
    pub votes_no: u64,
    pub voter_count: u64,
    
    // Support state (for Proposed phase)
    pub support_amount: u64,
    pub supporter_count: u64,
    
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct VoteKey {
    proposal_id: u64,
    voter: Principal,
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
}

/// Input for creating a board member proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateBoardMemberProposalInput {
    pub title: String,
    pub description: String,
    /// Wallet address of the new board member to add
    pub new_member: Principal,
    /// Percentage share to allocate to the new member (1-99)
    /// This percentage is taken equally from all existing members
    pub percentage: u8,
    pub external_link: Option<String>,
}

/// Board member share entry for query results and admin input
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BoardMemberShare {
    pub member: Principal,
    pub percentage: u8,
}

#[derive(CandidType, Deserialize)]
struct InitArgs {
    ledger_id: Principal,
    staking_hub_id: Principal,
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // Configuration
    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), Principal::anonymous()).unwrap()
    );
    
    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), Principal::anonymous()).unwrap()
    );

    // Proposals
    static PROPOSALS: RefCell<StableBTreeMap<u64, Proposal, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );
    
    static PROPOSAL_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))), 0).unwrap()
    );
    
    // Vote records
    static VOTE_RECORDS: RefCell<StableBTreeMap<VoteKey, VoteRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );

    // Support records
    static SUPPORT_RECORDS: RefCell<StableBTreeMap<VoteKey, SupportRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );
    
    // Treasury
    static TREASURY_STATE: RefCell<StableCell<TreasuryState, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            TreasuryState {
                balance: INITIAL_TREASURY_BALANCE,
                allowance: INITIAL_TREASURY_ALLOWANCE,
                total_transferred: 0,
                mmcr_count: 0,
                last_mmcr_timestamp: 0,
                genesis_timestamp: 0,
            }
        ).unwrap()
    );
    
    // Board Member Management
    // Board members exercise VUC (Volume of Unmined Coins) voting power
    // Each member has a percentage share of the total VUC
    
    /// Board member voting power shares: Principal -> percentage (1-100)
    /// Each board member gets (percentage / 100) * VUC voting power
    /// Total of all percentages must equal exactly 100
    static BOARD_MEMBER_SHARES: RefCell<StableBTreeMap<Principal, u8, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))))
    );
    
    /// Lock flag for board member shares
    /// Once locked, shares can only be changed via governance proposal
    static BOARD_SHARES_LOCKED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            false
        ).unwrap()
    );
}

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID"));
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    
    let now = ic_cdk::api::time();
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        if state.genesis_timestamp == 0 {
            state.genesis_timestamp = now;
            cell.set(state).expect("Failed to initialize treasury state");
        }
    });
    
    start_timers();
}

#[post_upgrade]
fn post_upgrade() {
    start_timers();
}

fn start_timers() {
    // MMCR timer (every 24 hours)
    set_timer_interval(Duration::from_secs(24 * 60 * 60), || {
        let _ = try_execute_mmcr();
    });
    
    // Proposal finalization timer (every hour)
    set_timer_interval(Duration::from_secs(60 * 60), || {
        ic_cdk::spawn(async {
            finalize_expired_proposals().await;
        });
    });
}

// ============================================================================
// BOARD MEMBER HELPERS
// ============================================================================

/// Check if a principal is a board member (local check)
fn is_board_member_local(principal: &Principal) -> bool {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().contains_key(principal))
}

/// Get a board member's percentage share (internal)
fn get_board_member_percentage_local(principal: &Principal) -> Option<u8> {
    BOARD_MEMBER_SHARES.with(|b| b.borrow().get(principal))
}

/// Fetch voting power for a user
/// - Board members: returns VUC * percentage / 100 (queries staking hub for VUC)
/// - Regular users: returns staked balance (queries staking hub)
async fn fetch_voting_power(user: Principal) -> Result<u64, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Check if user is a board member - return weighted VUC
    if let Some(percentage) = get_board_member_percentage_local(&user) {
        // Get VUC from staking hub
        let (vuc,): (u64,) = ic_cdk::call(
            staking_hub_id,
            "get_vuc",
            ()
        ).await.map_err(|e| format!("Failed to get VUC: {:?}", e))?;
        
        // Calculate weighted voting power: VUC * percentage / 100
        // Using u128 to avoid overflow during multiplication
        return Ok(((vuc as u128 * percentage as u128) / 100) as u64);
    }
    
    // For regular users, query their staked balance from staking hub
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "fetch_user_voting_power",
        (user,)
    ).await.map_err(|e| format!("Failed to get voting power: {:?}", e))?;
    
    Ok(voting_power)
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
    
    if voting_power < MIN_VOTING_POWER_TO_PROPOSE {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            MIN_VOTING_POWER_TO_PROPOSE / 100_000_000,
            voting_power / 100_000_000
        ));
    }
    
    // Check treasury allowance (for GHC)
    if input.token_type == TokenType::GHC {
        let allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
        if input.amount > allowance {
            return Err(format!(
                "Amount exceeds treasury allowance. Available: {} GHC",
                allowance / 100_000_000
            ));
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
    let (status, voting_ends_at) = if proposer_is_board_member {
        // Board members skip Proposed state, go directly to Active
        (ProposalStatus::Active, now + VOTING_PERIOD_NANOS)
    } else {
        // Regular users go to Proposed state with a support period deadline
        (ProposalStatus::Proposed, now + SUPPORT_PERIOD_NANOS)
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
        board_member_payload: None,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        status,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

/// Create a board member addition proposal (legacy alias for backward compatibility)
#[update]
async fn create_proposal(input: CreateTreasuryProposalInput) -> Result<u64, String> {
    create_treasury_proposal(input).await
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
    
    if voting_power < MIN_VOTING_POWER_TO_PROPOSE {
        return Err(format!(
            "Insufficient voting power to propose. Required: {}, You have: {}",
            MIN_VOTING_POWER_TO_PROPOSE / 100_000_000,
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
    let (status, voting_ends_at) = if proposer_is_board_member {
        // Board members skip Proposed state, go directly to Active
        (ProposalStatus::Active, now + VOTING_PERIOD_NANOS)
    } else {
        // Regular users go to Proposed state with a support period deadline
        (ProposalStatus::Proposed, now + SUPPORT_PERIOD_NANOS)
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
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        support_amount: 0,
        supporter_count: 0,
        status,
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
    
    // Check threshold (15,000 VP and 2 users)
    // 15,000 tokens * 10^8
    let support_threshold = 15_000 * 100_000_000;
    
    if proposal.support_amount >= support_threshold && proposal.supporter_count >= 2 {
        proposal.status = ProposalStatus::Active;
        proposal.voting_ends_at = now + VOTING_PERIOD_NANOS;
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
async fn finalize_expired_proposals() {
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

// Changed to synchronous status update as no inter-canister calls needed for status flip
// Execution is now separate
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
        // Check voting period ended
        // We allow early finalization if the approval threshold is met, enabling "Fast Track" execution.
        if now <= proposal.voting_ends_at && proposal.votes_yes < APPROVAL_THRESHOLD {
             return Err(format!(
                 "Voting period not ended yet. Current Yes votes: {}, Required: {}",
                 proposal.votes_yes / 100_000_000, 
                 APPROVAL_THRESHOLD / 100_000_000
             ));
        }
        
        // Determine outcome
        if proposal.votes_yes >= APPROVAL_THRESHOLD {
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
    }
    
    let mut proposal = proposal; // Get a mutable copy
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}

/// Execute a treasury spending proposal
async fn execute_treasury_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    let token_type = proposal.token_type.as_ref()
        .ok_or("Treasury proposal missing token_type")?;
    let amount = proposal.amount
        .ok_or("Treasury proposal missing amount")?;
    let recipient = proposal.recipient
        .ok_or("Treasury proposal missing recipient")?;
    
    // Only execute GHC for now (USDC/ICP requires additional ledger setup)
    if *token_type != TokenType::GHC {
        return Err("Only GHC transfers are supported currently".to_string());
    }
    
    // Check treasury allowance
    let current_allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
    if amount > current_allowance {
        return Err("Insufficient treasury allowance".to_string());
    }
    
    // Execute transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: recipient, subaccount: None },
        amount: Nat::from(amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    
    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer failed: {:?} {}", code, msg))?;
    
    match result {
        Ok(_) => {
            // Update treasury state
            TREASURY_STATE.with(|s| {
                let mut cell = s.borrow_mut();
                let mut state = cell.get().clone();
                state.balance = state.balance.saturating_sub(amount);
                state.allowance = state.allowance.saturating_sub(amount);
                state.total_transferred += amount;
                cell.set(state).expect("Failed to update treasury state");
            });
            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
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
    
    // Calculate new shares for existing members
    // We need to reduce total existing shares from 100% to (100 - new_percentage)%
    // Each member's new share = old_share * (100 - new_percentage) / 100
    // Calculate new shares for existing members using the Largest Remainder Method
    // This ensures fair distribution of rounding errors instead of dumping them on the last member
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
fn get_governance_config() -> (u64, u64, u64, u64, u64) {
    (
        MIN_VOTING_POWER_TO_PROPOSE / 100_000_000, // In tokens
        APPROVAL_THRESHOLD / 100_000_000,          // In tokens
        SUPPORT_PERIOD_NANOS / (24 * 60 * 60 * 1_000_000_000), // In days
        VOTING_PERIOD_NANOS / (24 * 60 * 60 * 1_000_000_000), // In days
        RESUBMISSION_COOLDOWN_NANOS / (24 * 60 * 60 * 1_000_000_000), // In days
    )
}

// ============================================================================
// TREASURY FUNCTIONS
// ============================================================================

#[query]
fn get_treasury_state() -> TreasuryState {
    TREASURY_STATE.with(|s| s.borrow().get().clone())
}

#[query]
fn get_spendable_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().allowance)
}

fn try_execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err("All MMCR releases completed".to_string());
        }
        
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            return Err("Too early for next MMCR".to_string());
        }
        
        let release_amount = if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        let new_allowance = (state.allowance + release_amount).min(state.balance);
        let actual_release = new_allowance - state.allowance;
        
        state.allowance = new_allowance;
        state.mmcr_count += 1;
        state.last_mmcr_timestamp = current_time;
        
        cell.set(state).expect("Failed to update treasury state");
        
        Ok(actual_release)
    })
}

#[update]
fn execute_mmcr() -> Result<u64, String> {
    try_execute_mmcr()
}

#[derive(CandidType, Clone, Debug)]
pub struct MMCRStatus {
    pub releases_completed: u64,
    pub releases_remaining: u64,
    pub last_release_timestamp: u64,
    pub next_release_amount: u64,
    pub seconds_until_next: u64,
}

#[query]
fn get_mmcr_status() -> MMCRStatus {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let state = s.borrow().get().clone();
        let releases_remaining = TOTAL_MMCR_RELEASES.saturating_sub(state.mmcr_count);
        
        let next_release_amount = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        let seconds_until_next = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else if state.last_mmcr_timestamp == 0 {
            // No MMCR has ever occurred - calculate time from genesis
            // Initial allowance is available immediately, first MMCR unlocks 30 days after genesis
            let first_mmcr_available = state.genesis_timestamp + MMCR_MIN_INTERVAL_NANOS;
            if current_time >= first_mmcr_available { 0 } else { (first_mmcr_available - current_time) / 1_000_000_000 }
        } else {
            let next_available = state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS;
            if current_time >= next_available { 0 } else { (next_available - current_time) / 1_000_000_000 }
        };
        
        MMCRStatus {
            releases_completed: state.mmcr_count,
            releases_remaining,
            last_release_timestamp: state.last_mmcr_timestamp,
            next_release_amount,
            seconds_until_next,
        }
    })
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

ic_cdk::export_candid!();
