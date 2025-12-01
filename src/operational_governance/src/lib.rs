use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
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

type Memory = VirtualMemory<DefaultMemoryImpl>;

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

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );

    static PROPOSALS: RefCell<StableBTreeMap<u64, Proposal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static PROPOSAL_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            0
        ).unwrap()
    );
    
    // Track votes to prevent double voting: ProposalId -> Vec<Principal>
    // For scalability, this should be a separate map (ProposalId, Voter) -> Vote
    // But for MVP we'll skip double-voting check or implement it simply.
    // Using a separate map is better.
    static VOTES: RefCell<StableBTreeMap<(u64, Principal), bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );
}

#[init]
fn init(args: InitArgs) {
    LEDGER_ID.with(|id| id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID"));
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
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
            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}

#[query]
fn get_proposal(id: u64) -> Option<Proposal> {
    PROPOSALS.with(|p| p.borrow().get(&id))
}

ic_cdk::export_candid!();
