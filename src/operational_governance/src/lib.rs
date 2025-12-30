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
const MMCR_MIN_INTERVAL_NANOS: u64 = 28 * 24 * 60 * 60 * 1_000_000_000;

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

/// Proposal categories
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

/// Treasury spending proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Principal,
    pub created_at: u64,
    pub voting_ends_at: u64,
    
    // Proposal details
    pub title: String,
    pub description: String,
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub category: ProposalCategory,
    pub external_link: Option<String>,
    
    // Voting state
    pub votes_yes: u64,
    pub votes_no: u64,
    pub voter_count: u64,
    
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

/// Input for creating a proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateProposalInput {
    pub title: String,
    pub description: String,
    pub recipient: Principal,
    pub amount: u64,
    pub token_type: TokenType,
    pub category: ProposalCategory,
    pub external_link: Option<String>,
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
    
    // Vote records (for transparency - see who voted)
    static VOTE_RECORDS: RefCell<StableBTreeMap<VoteKey, VoteRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
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
// PROPOSAL CREATION
// ============================================================================

#[update]
async fn create_proposal(input: CreateProposalInput) -> Result<u64, String> {
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
    
    // Check proposer has enough voting power
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "fetch_voting_power",
        (proposer,)
    ).await.map_err(|e| format!("Failed to get voting power: {:?}", e))?;
    
    if voting_power < MIN_VOTING_POWER_TO_PROPOSE {
        return Err(format!(
            "Insufficient voting power. Required: {}, You have: {}",
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
    
    let proposal = Proposal {
        id,
        proposer,
        created_at: now,
        voting_ends_at: now + VOTING_PERIOD_NANOS,
        title: input.title,
        description: input.description,
        recipient: input.recipient,
        amount: input.amount,
        token_type: input.token_type,
        category: input.category,
        external_link: input.external_link,
        votes_yes: 0,
        votes_no: 0,
        voter_count: 0,
        status: ProposalStatus::Active,
    };
    
    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
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
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "fetch_voting_power",
        (voter,)
    ).await.map_err(|e| format!("Failed to get voting power: {:?}", e))?;
    
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

/// Finalize proposals whose voting period has ended
async fn finalize_expired_proposals() {
    let now = ic_cdk::api::time();
    
    let proposals_to_finalize: Vec<u64> = PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .filter(|(_, prop)| prop.status == ProposalStatus::Active && now > prop.voting_ends_at)
            .map(|(id, _)| id)
            .collect()
    });
    
    for id in proposals_to_finalize {
        let _ = finalize_proposal(id).await;
    }
}

#[update]
async fn finalize_proposal(proposal_id: u64) -> Result<ProposalStatus, String> {
    let now = ic_cdk::api::time();
    
    let mut proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    // Check if already finalized (Executed or Rejected)
    if proposal.status == ProposalStatus::Executed || proposal.status == ProposalStatus::Rejected {
        return Err(format!("Proposal is already finalized with status: {:?}", proposal.status));
    }
    
    // If Active, handle voting outcome logic
    if proposal.status == ProposalStatus::Active {
        // Check voting period ended (or threshold already reached)
        if now <= proposal.voting_ends_at && proposal.votes_yes < APPROVAL_THRESHOLD {
            return Err("Voting period not ended yet".to_string());
        }
        
        // Determine outcome
        if proposal.votes_yes >= APPROVAL_THRESHOLD {
            proposal.status = ProposalStatus::Approved;
            
            // Save state as Approved before execution attempt
            PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal.clone()));
        } else {
            proposal.status = ProposalStatus::Rejected;
            PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal.clone()));
            return Ok(ProposalStatus::Rejected);
        }
    }
    
    // If Approved (either just now or previously), try to Execute
    if proposal.status == ProposalStatus::Approved {
        // Execute the proposal
        let exec_result = execute_proposal_internal(&proposal).await;
        
        match exec_result {
            Ok(_) => {
                proposal.status = ProposalStatus::Executed;
                PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal.clone()));
                return Ok(ProposalStatus::Executed);
            },
            Err(e) => {
                // Return error but keep status as Approved so it can be retried
                return Err(format!("Execution failed: {}", e));
            }
        }
    }
    
    Ok(proposal.status)
}

async fn execute_proposal_internal(proposal: &Proposal) -> Result<(), String> {
    // Only execute GHC for now (USDC/ICP requires additional ledger setup)
    if proposal.token_type != TokenType::GHC {
        return Err("Only GHC transfers are supported currently".to_string());
    }
    
    // Check treasury allowance
    let current_allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
    if proposal.amount > current_allowance {
        return Err("Insufficient treasury allowance".to_string());
    }
    
    // Execute transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: proposal.recipient, subaccount: None },
        amount: Nat::from(proposal.amount),
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
                state.balance = state.balance.saturating_sub(proposal.amount);
                state.allowance = state.allowance.saturating_sub(proposal.amount);
                state.total_transferred += proposal.amount;
                cell.set(state).expect("Failed to update treasury state");
            });
            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
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
fn get_governance_config() -> (u64, u64, u64, u64) {
    (
        MIN_VOTING_POWER_TO_PROPOSE / 100_000_000, // In tokens
        APPROVAL_THRESHOLD / 100_000_000,          // In tokens
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
            0
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

ic_cdk::export_candid!();
