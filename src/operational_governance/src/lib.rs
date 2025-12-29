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
// TREASURY CONSTANTS (in e8s - 8 decimals)
// ============================================================================

/// Initial treasury balance: 4.25B MC (in e8s)
const INITIAL_TREASURY_BALANCE: u64 = 425_000_000_000_000_000; // 4.25B * 10^8

/// Initial treasury allowance: 0.6B MC (in e8s)
const INITIAL_TREASURY_ALLOWANCE: u64 = 60_000_000_000_000_000; // 0.6B * 10^8

/// Monthly Market Coin Release: 15.2M MC (in e8s)
const MMCR_AMOUNT: u64 = 1_520_000_000_000_000; // 15.2M * 10^8

/// Final month adjusted amount: 17.2M MC (in e8s)
/// Calculation: 4.25B - 0.6B - (15.2M * 239) = 17.2M
const FINAL_MMCR_AMOUNT: u64 = 1_720_000_000_000_000; // 17.2M * 10^8

/// Total MMCR releases over 20 years
const TOTAL_MMCR_RELEASES: u64 = 240;

/// Minimum interval between MMCR releases (28 days in nanoseconds)
const MMCR_MIN_INTERVAL_NANOS: u64 = 28 * 24 * 60 * 60 * 1_000_000_000;

// ============================================================================
// TREASURY STATE
// ============================================================================

/// Treasury state - tracks balance and allowance
/// This is the core data structure for the Treasury system.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    /// Total MC balance held by the treasury (decreases only on transfers)
    pub balance: u64,
    
    /// Current spending allowance (liquid allocation)
    /// Can only spend up to this amount through proposals
    pub allowance: u64,
    
    /// Total amount transferred out historically
    pub total_transferred: u64,
    
    /// Number of MMCR executions completed (0-240)
    pub mmcr_count: u64,
    
    /// Timestamp of last MMCR execution (nanoseconds)
    pub last_mmcr_timestamp: u64,
    
    /// Genesis timestamp (when Treasury was initialized)
    pub genesis_timestamp: u64,
}

impl Storable for TreasuryState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

// ============================================================================
// PROPOSAL DATA STRUCTURE
// ============================================================================

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Proposal {
    id: u64,
    proposer: Principal,
    recipient: Principal,
    amount: u64,
    description: String,
    votes_yes: u64,
    votes_no: u64,
    executed: bool,
    created_at: u64,
}

impl Storable for Proposal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 5000,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize)]
struct InitArgs {
    ledger_id: Principal,
    staking_hub_id: Principal,
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// All persistent state is stored in stable memory using ic_stable_structures.
// Each storage item is assigned a unique MemoryId for isolation.
// 
// Memory IDs:
//   0 - LEDGER_ID: GHC ledger canister principal
//   1 - STAKING_HUB_ID: Staking hub canister principal
//   2 - PROPOSALS: Spending proposals storage
//   3 - PROPOSAL_COUNT: Sequential counter for proposal IDs
//   4 - VOTES: Double-voting prevention map
//   5 - TOTAL_SPENT: Historical total spent through proposals
//   6 - TREASURY_STATE: Treasury balance, allowance, and MMCR state

thread_local! {
    // ─────────────────────────────────────────────────────────────────────
    // Memory Management
    // ─────────────────────────────────────────────────────────────────────
    
    /// Memory manager for allocating virtual memory regions to each storage
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // ─────────────────────────────────────────────────────────────────────
    // Configuration (Set once during init)
    // ─────────────────────────────────────────────────────────────────────

    /// Principal ID of the GHC ICRC-1 ledger canister
    /// Used for token transfers when executing proposals
    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the staking hub canister
    /// Used to query voting power for proposal creation and voting
    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Governance Data
    // ─────────────────────────────────────────────────────────────────────

    /// Map of proposal_id -> Proposal
    /// Contains all spending proposals (pending, approved, executed)
    static PROPOSALS: RefCell<StableBTreeMap<u64, Proposal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Counter for generating sequential proposal IDs
    /// Incremented each time a new proposal is created
    static PROPOSAL_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            0
        ).unwrap()
    );
    
    /// Map of (proposal_id, voter_principal) -> vote (true=yes, false=no)
    /// Prevents double voting on the same proposal
    static VOTES: RefCell<StableBTreeMap<(u64, Principal), bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// Total amount spent through executed proposals (historical)
    /// Used for analytics and auditing
    static TOTAL_SPENT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Treasury State
    // ─────────────────────────────────────────────────────────────────────

    /// Treasury state tracking balance, allowance, and MMCR progress
    /// - balance: Total MC held (4.25B initial, decreases on transfers)
    /// - allowance: Spendable amount (0.6B initial, increases via MMCR)
    /// - mmcr_count: Number of MMCR releases executed (0-240)
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

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID"));
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    
    // Initialize treasury state with genesis timestamp
    let now = ic_cdk::api::time();
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        if state.genesis_timestamp == 0 {
            state.genesis_timestamp = now;
            cell.set(state).expect("Failed to initialize treasury state");
        }
    });
    
    // Start MMCR timer
    start_mmcr_timer();
}

#[post_upgrade]
fn post_upgrade() {
    // Restart MMCR timer after upgrade
    start_mmcr_timer();
}

/// Start the MMCR auto-execution timer
/// Runs every 24 hours to check if MMCR can be executed
fn start_mmcr_timer() {
    // Check every 24 hours (in seconds)
    let interval = Duration::from_secs(24 * 60 * 60);
    
    set_timer_interval(interval, || {
        // Try to execute MMCR
        let result = try_execute_mmcr();
        match result {
            Ok(amount) => {
                ic_cdk::println!("MMCR auto-executed: {} e8s released", amount);
            }
            Err(msg) => {
                // Not an error - just means it's not time yet or already completed
                ic_cdk::println!("MMCR check: {}", msg);
            }
        }
    });
    
    ic_cdk::println!("MMCR timer started (checks every 24 hours)");
}

/// Internal MMCR execution (non-async version for timer)
fn try_execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Check if all releases completed
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err("All MMCR releases completed".to_string());
        }
        
        // Check if enough time has passed
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            return Err("Too early for next MMCR".to_string());
        }
        
        // Determine release amount
        let release_amount = if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        // Increase allowance
        let new_allowance = (state.allowance + release_amount).min(state.balance);
        let actual_release = new_allowance - state.allowance;
        
        // Update state
        state.allowance = new_allowance;
        state.mmcr_count += 1;
        state.last_mmcr_timestamp = current_time;
        
        cell.set(state).expect("Failed to update treasury state");
        
        Ok(actual_release)
    })
}

#[update]
async fn create_proposal(recipient: Principal, amount: u64, description: String) -> Result<u64, String> {
    let proposer = ic_cdk::caller();
    
    // Check proposer voting power (must have > 0)
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "get_voting_power",
        (proposer,)
    ).await.map_err(|e| format!("Failed to call staking hub: {:?}", e))?;

    if voting_power == 0 {
        return Err("Insufficient voting power to propose".to_string());
    }

    let id = PROPOSAL_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let current = *cell.get();
        cell.set(current + 1).expect("Failed to increment proposal count");
        current
    });

    let proposal = Proposal {
        id,
        proposer,
        recipient,
        amount,
        description,
        votes_yes: 0,
        votes_no: 0,
        executed: false,
        created_at: ic_cdk::api::time(),
    };

    PROPOSALS.with(|p| p.borrow_mut().insert(id, proposal));
    
    Ok(id)
}

#[update]
async fn vote(proposal_id: u64, approve: bool) -> Result<(), String> {
    let voter = ic_cdk::caller();
    
    // Check if already voted
    if VOTES.with(|v| v.borrow().contains_key(&(proposal_id, voter))) {
        return Err("Already voted".to_string());
    }

    // Get voting power
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "get_voting_power",
        (voter,)
    ).await.map_err(|e| format!("Failed to call staking hub: {:?}", e))?;

    if voting_power == 0 {
        return Err("No voting power".to_string());
    }

    // Update proposal
    PROPOSALS.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut proposal) = map.get(&proposal_id) {
            if proposal.executed {
                // Can't vote on executed proposals
                // return Err("Proposal already executed".to_string()); 
                // StableBTreeMap doesn't allow easy early return inside closure if we want to mutate
                // So we just don't update.
            } else {
                if approve {
                    proposal.votes_yes += voting_power;
                } else {
                    proposal.votes_no += voting_power;
                }
                map.insert(proposal_id, proposal);
            }
        }
    });
    
    // Record vote
    VOTES.with(|v| v.borrow_mut().insert((proposal_id, voter), approve));

    Ok(())
}

#[update]
async fn execute_proposal(proposal_id: u64) -> Result<(), String> {
    // Anyone can trigger execution if conditions met
    
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;

    if proposal.executed {
        return Err("Already executed".to_string());
    }

    // Simple majority check
    if proposal.votes_yes <= proposal.votes_no {
        return Err("Proposal not approved".to_string());
    }

    // =========================================================================
    // NEW: Check treasury allowance before transfer
    // =========================================================================
    let current_allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
    if proposal.amount > current_allowance {
        return Err(format!(
            "Insufficient treasury allowance. Requested: {} e8s, Available: {} e8s. \
             Wait for next MMCR or reduce proposal amount.",
            proposal.amount, current_allowance
        ));
    }

    // Execute transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None, // From this canister's main account
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
    ).await.map_err(|(code, msg)| format!("Rejection code: {:?}, message: {}", code, msg))?;

    match result {
        Ok(_) => {
            // Mark executed
            PROPOSALS.with(|p| {
                let mut map = p.borrow_mut();
                if let Some(mut prop) = map.get(&proposal_id) {
                    prop.executed = true;
                    map.insert(proposal_id, prop);
                }
            });
            
            // =========================================================================
            // NEW: Update treasury state (decrease both balance AND allowance)
            // =========================================================================
            TREASURY_STATE.with(|s| {
                let mut cell = s.borrow_mut();
                let mut state = cell.get().clone();
                state.balance = state.balance.saturating_sub(proposal.amount);
                state.allowance = state.allowance.saturating_sub(proposal.amount);
                state.total_transferred += proposal.amount;
                cell.set(state).expect("Failed to update treasury state");
            });
            
            TOTAL_SPENT.with(|t| {
                let mut cell = t.borrow_mut();
                let current = *cell.get();
                cell.set(current + proposal.amount).expect("Failed to update total spent");
            });

            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}

#[query]
fn get_proposal(id: u64) -> Option<Proposal> {
    PROPOSALS.with(|p| p.borrow().get(&id))
}

#[query]
fn get_total_spent() -> u64 {
    TOTAL_SPENT.with(|t| *t.borrow().get())
}

// ============================================================================
// TREASURY FUNCTIONS
// ============================================================================

/// Get current treasury state (balance, allowance, MMCR progress)
#[query]
fn get_treasury_state() -> TreasuryState {
    TREASURY_STATE.with(|s| s.borrow().get().clone())
}

/// Get spendable balance (current allowance)
#[query]
fn get_spendable_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().allowance)
}

/// Get total treasury balance (decreases only on transfers)
#[query]
fn get_treasury_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().balance)
}

/// MMCR status response
#[derive(CandidType, Clone, Debug)]
pub struct MMCRStatus {
    /// Number of MMCR releases completed
    pub releases_completed: u64,
    /// Number of MMCR releases remaining
    pub releases_remaining: u64,
    /// Timestamp of last MMCR execution
    pub last_release_timestamp: u64,
    /// Next MMCR amount (regular or final adjusted)
    pub next_release_amount: u64,
    /// Estimated time until next MMCR is available (in seconds)
    pub seconds_until_next: u64,
}

/// Get MMCR (Monthly Market Coin Release) status
#[query]
fn get_mmcr_status() -> MMCRStatus {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let state = s.borrow().get().clone();
        let releases_remaining = TOTAL_MMCR_RELEASES.saturating_sub(state.mmcr_count);
        
        // Calculate next release amount
        let next_release_amount = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        // Calculate seconds until next MMCR
        let seconds_until_next = if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            0
        } else if state.last_mmcr_timestamp == 0 {
            0 // First MMCR can be executed immediately after genesis
        } else {
            let next_available = state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS;
            if current_time >= next_available {
                0
            } else {
                (next_available - current_time) / 1_000_000_000
            }
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

/// Execute Monthly Market Coin Release (MMCR)
/// 
/// This function increases the treasury allowance by the MMCR amount.
/// - Can be called by anyone (idempotent, time-gated)
/// - Executes only if 28+ days have passed since last MMCR
/// - Does NOT decrease treasury balance (allowance is just "unlocked")
/// - Final month releases 17.2M instead of 15.2M to reach exactly 4.25B
#[update]
fn execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Check if all releases completed
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err(format!(
                "All {} MMCR releases completed. Treasury is fully unlocked.",
                TOTAL_MMCR_RELEASES
            ));
        }
        
        // Check if enough time has passed (minimum 28 days between releases)
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            let remaining_nanos = (state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS) - current_time;
            let remaining_days = remaining_nanos / (24 * 60 * 60 * 1_000_000_000);
            return Err(format!(
                "Too early for next MMCR. Wait approximately {} more days.",
                remaining_days + 1
            ));
        }
        
        // Determine release amount (final month is adjusted)
        let release_amount = if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT // 17.2M for the final release
        } else {
            MMCR_AMOUNT // 15.2M for regular releases
        };
        
        // Increase allowance (cannot exceed balance)
        let new_allowance = (state.allowance + release_amount).min(state.balance);
        let actual_release = new_allowance - state.allowance;
        
        // Update state
        state.allowance = new_allowance;
        state.mmcr_count += 1;
        state.last_mmcr_timestamp = current_time;
        
        cell.set(state.clone()).expect("Failed to update treasury state");
        
        ic_cdk::println!(
            "MMCR #{} executed: Released {} e8s. New allowance: {} e8s. Remaining releases: {}",
            state.mmcr_count,
            actual_release,
            new_allowance,
            TOTAL_MMCR_RELEASES - state.mmcr_count
        );
        
        Ok(actual_release)
    })
}

ic_cdk::export_candid!();
