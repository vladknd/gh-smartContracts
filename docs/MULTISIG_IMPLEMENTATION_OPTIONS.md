# Multi-Signature Implementation Options

This document compares how to implement multi-signature security for treasury operations in both **unified** (current) and **split** canister architectures.

> **⚠️ ARCHITECTURE UPDATE (January 2026)**
>
> The **split architecture** described in Approach 2 is now implemented:
> - **`treasury_canister`**: Token custody (4.25B MC), executes transfers
> - **`governance_canister`**: Proposals, voting, calls treasury for transfers
>
> Multi-sig implementation should follow Approach 2 (Split Architecture).

---

## Table of Contents

1. [Overview](#overview)
2. [Approach 1: Multi-Sig in Unified Canister](#approach-1-multi-sig-in-unified-canister-current-architecture)
3. [Approach 2: Multi-Sig in Split Architecture](#approach-2-multi-sig-in-split-architecture)
4. [Comparison](#comparison)
5. [Recommended Implementation](#recommended-implementation)

---

## Overview

**Multi-signature (multi-sig)** adds an additional security layer where multiple authorized parties must approve treasury transactions before execution.

### Common Multi-Sig Requirements:
- **2-of-3**: Any 2 out of 3 designated signers must approve
- **3-of-5**: Any 3 out of 5 designated signers must approve
- **M-of-N**: Any M out of N signers must approve

### Why Multi-Sig?
1. **Prevents single point of failure** - No single person can drain treasury
2. **Protects against compromised keys** - One stolen key is insufficient
3. **Adds human review** - Multiple eyes verify recipient address
4. **Satisfies audit requirements** - Many organizations require multi-sig for large transfers

---

## Approach 1: Multi-Sig in Unified Canister (Current Architecture)

In the current unified architecture, multi-sig is **added as an additional step after governance approval**.

### Current Flow:
```
1. create_treasury_proposal()
2. support_proposal() [if regular user]
3. vote() [YES/NO voting]
4. finalize_proposal() → Status: Approved
5. execute_proposal() → [FUNDS TRANSFER]
```

### With Multi-Sig:
```
1. create_treasury_proposal()
2. support_proposal() [if regular user]
3. vote() [YES/NO voting]
4. finalize_proposal() → Status: Approved
5. approve_execution() [NEW: Multi-sig approvals]
6. approve_execution() [2nd signer]
7. execute_proposal() → [FUNDS TRANSFER] (auto-executes when threshold met)
```

---

### Implementation (Unified Architecture)

#### 1. Add Multi-Sig State

```rust
use std::collections::HashSet;

// Multi-sig configuration
const REQUIRED_SIGNATURES: u8 = 2;
const TOTAL_SIGNERS: u8 = 3;

thread_local! {
    // Authorized signers (controllers or designated trustees)
    static AUTHORIZED_SIGNERS: RefCell<StableCell<Vec<Principal>, Memory>> = 
        RefCell::new(StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            vec![]
        ).expect("Failed to initialize signers"));
    
    // Tracks approvals: proposal_id -> Set<signer_principal>
    static EXECUTION_APPROVALS: RefCell<StableBTreeMap<u64, HashSet<Principal>, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11)))
        ));
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ExecutionApproval {
    pub proposal_id: u64,
    pub signer: Principal,
    pub timestamp: u64,
}
```

#### 2. Admin Functions to Manage Signers

```rust
/// Set authorized signers (admin only)
#[update]
fn set_authorized_signers(signers: Vec<Principal>) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    if signers.len() < REQUIRED_SIGNATURES as usize {
        return Err(format!(
            "Must have at least {} signers for {}-of-{} multi-sig",
            REQUIRED_SIGNATURES, REQUIRED_SIGNATURES, TOTAL_SIGNERS
        ));
    }
    
    AUTHORIZED_SIGNERS.with(|s| {
        s.borrow_mut()
            .set(signers)
            .expect("Failed to set signers")
    });
    
    Ok(())
}

#[query]
fn get_authorized_signers() -> Vec<Principal> {
    AUTHORIZED_SIGNERS.with(|s| s.borrow().get().clone())
}
```

#### 3. Approval Function

```rust
/// Approve execution of an approved proposal (multi-sig)
#[update]
fn approve_execution(proposal_id: u64) -> Result<u8, String> {
    let caller = ic_cdk::caller();
    let now = ic_cdk::api::time();
    
    // Verify caller is authorized signer
    let is_authorized = AUTHORIZED_SIGNERS.with(|s| {
        s.borrow().get().contains(&caller)
    });
    
    if !is_authorized {
        return Err("Caller is not an authorized signer".to_string());
    }
    
    // Get proposal
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    // Verify proposal is approved
    if proposal.status != ProposalStatus::Approved {
        return Err("Proposal must be Approved before execution approval".to_string());
    }
    
    // Record approval
    let approval_count = EXECUTION_APPROVALS.with(|a| {
        let mut map = a.borrow_mut();
        let mut approvals = map.get(&proposal_id).unwrap_or_default();
        
        if approvals.contains(&caller) {
            return Err("Already approved".to_string());
        }
        
        approvals.insert(caller);
        let count = approvals.len() as u8;
        map.insert(proposal_id, approvals);
        
        Ok(count)
    })?;
    
    // If threshold reached, auto-execute
    if approval_count >= REQUIRED_SIGNATURES {
        // Execute the proposal automatically
        ic_cdk::spawn(async move {
            let _ = execute_proposal_internal(proposal_id).await;
        });
    }
    
    Ok(approval_count)
}

/// Get approvers for a proposal
#[query]
fn get_execution_approvals(proposal_id: u64) -> Vec<Principal> {
    EXECUTION_APPROVALS.with(|a| {
        a.borrow()
            .get(&proposal_id)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    })
}
```

#### 4. Modify `execute_proposal` to Check Multi-Sig

```rust
#[update]
async fn execute_proposal(proposal_id: u64) -> Result<(), String> {
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    if proposal.status != ProposalStatus::Approved {
        return Err("Proposal is not Approved".to_string());
    }
    
    // CHECK MULTI-SIG REQUIREMENT
    let approval_count = EXECUTION_APPROVALS.with(|a| {
        a.borrow()
            .get(&proposal_id)
            .map(|set| set.len() as u8)
            .unwrap_or(0)
    });
    
    if approval_count < REQUIRED_SIGNATURES {
        return Err(format!(
            "Insufficient execution approvals. Required: {}, Got: {}",
            REQUIRED_SIGNATURES, approval_count
        ));
    }
    
    // Proceed with execution...
    match proposal.proposal_type {
        ProposalType::Treasury => execute_treasury_proposal_internal(&proposal).await?,
        ProposalType::AddBoardMember => execute_board_member_proposal_internal(&proposal)?,
    }
    
    let mut proposal = proposal;
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}
```

---

### Drawbacks of Unified Multi-Sig:

1. **Mixing Concerns**: Governance voting and treasury security are in the same codebase
2. **Complexity**: More state to manage in a single canister
3. **Upgrade Risk**: Changes to governance logic could inadvertently affect multi-sig
4. **Limited Flexibility**: Hard to have different multi-sig rules for different amount thresholds

---

## Approach 2: Multi-Sig in Split Architecture

With split canisters, the **Treasury Canister** handles multi-sig independently.

### Architecture:

```
┌─────────────────────────────┐
│   Governance Canister       │
│                             │
│  • Proposals               │
│  • Voting                  │
│  • Approval Status         │
└─────────────┬───────────────┘
              │
              │ Inter-canister call:
              │ request_transfer(proposal_id, recipient, amount)
              │
              ▼
┌─────────────────────────────┐
│   Treasury Canister         │
│                             │
│  • Balance tracking        │
│  • Multi-sig approvals     │
│  • ICRC-1 transfers        │
│  • MMCR logic              │
└─────────────────────────────┘
```

---

### Implementation (Split Architecture)

#### Treasury Canister

```rust
// treasury/src/lib.rs

use ic_cdk::{init, query, update};
use candid::{CandidType, Deserialize, Principal, Nat};
use std::collections::HashSet;

// Multi-sig configuration
const REQUIRED_SIGNATURES: u8 = 2;

thread_local! {
    // Authorized governance canister(s)
    static GOVERNANCE_CANISTER: RefCell<StableCell<Principal, Memory>> = /* ... */;
    
    // Authorized signers
    static AUTHORIZED_SIGNERS: RefCell<StableCell<Vec<Principal>, Memory>> = /* ... */;
    
    // Pending transfer requests from governance
    static PENDING_TRANSFERS: RefCell<StableBTreeMap<u64, TransferRequest, Memory>> = /* ... */;
    
    // Multi-sig approvals: request_id -> Set<signer_principal>
    static TRANSFER_APPROVALS: RefCell<StableBTreeMap<u64, HashSet<Principal>, Memory>> = /* ... */;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransferRequest {
    pub request_id: u64,
    pub proposal_id: u64,  // From governance canister
    pub recipient: Principal,
    pub amount: u64,
    pub created_at: u64,
    pub approved_by_governance: bool,
    pub executed: bool,
}

/// Called by Governance Canister when a proposal is approved
#[update]
fn request_transfer(
    proposal_id: u64,
    recipient: Principal,
    amount: u64
) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is authorized governance canister
    let governance = GOVERNANCE_CANISTER.with(|g| *g.borrow().get());
    if caller != governance {
        return Err("Unauthorized: Only governance canister can request transfers".to_string());
    }
    
    // Create transfer request
    let request_id = generate_request_id();
    let request = TransferRequest {
        request_id,
        proposal_id,
        recipient,
        amount,
        created_at: ic_cdk::api::time(),
        approved_by_governance: true,
        executed: false,
    };
    
    PENDING_TRANSFERS.with(|t| {
        t.borrow_mut().insert(request_id, request)
    });
    
    Ok(request_id)
}

/// Multi-sig approval by authorized signers
#[update]
async fn approve_transfer(request_id: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is authorized signer
    let is_authorized = AUTHORIZED_SIGNERS.with(|s| {
        s.borrow().get().contains(&caller)
    });
    
    if !is_authorized {
        return Err("Unauthorized: Not an authorized signer".to_string());
    }
    
    // Get request
    let request = PENDING_TRANSFERS.with(|t| t.borrow().get(&request_id))
        .ok_or("Transfer request not found")?;
    
    if request.executed {
        return Err("Transfer already executed".to_string());
    }
    
    // Record approval
    let approval_count = TRANSFER_APPROVALS.with(|a| {
        let mut map = a.borrow_mut();
        let mut approvals = map.get(&request_id).unwrap_or_default();
        
        if approvals.contains(&caller) {
            return Err("Already approved".to_string());
        }
        
        approvals.insert(caller);
        let count = approvals.len() as u8;
        map.insert(request_id, approvals);
        
        Ok(count)
    })?;
    
    // If threshold reached, execute transfer
    if approval_count >= REQUIRED_SIGNATURES {
        execute_transfer_internal(request_id).await?;
    }
    
    Ok(())
}

/// Internal function to execute transfer
async fn execute_transfer_internal(request_id: u64) -> Result<(), String> {
    let request = PENDING_TRANSFERS.with(|t| t.borrow().get(&request_id))
        .ok_or("Transfer request not found")?;
    
    // Execute ICRC-1 transfer
    let ledger_id = /* get ledger */;
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: request.recipient, subaccount: None },
        amount: Nat::from(request.amount),
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
            // Mark as executed
            PENDING_TRANSFERS.with(|t| {
                let mut map = t.borrow_mut();
                if let Some(mut req) = map.get(&request_id) {
                    req.executed = true;
                    map.insert(request_id, req);
                }
            });
            
            // Update treasury balance
            update_balance_after_transfer(request.amount);
            
            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}

/// Query pending transfers
#[query]
fn get_pending_transfers() -> Vec<TransferRequest> {
    PENDING_TRANSFERS.with(|t| {
        t.borrow()
            .iter()
            .filter(|(_, req)| !req.executed)
            .map(|(_, req)| req)
            .collect()
    })
}

/// Query approvals for a transfer
#[query]
fn get_transfer_approvals(request_id: u64) -> Vec<Principal> {
    TRANSFER_APPROVALS.with(|a| {
        a.borrow()
            .get(&request_id)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    })
}
```

#### Governance Canister (Modified)

```rust
// operational_governance/src/lib.rs

/// Execute an approved proposal
#[update]
async fn execute_proposal(proposal_id: u64) -> Result<(), String> {
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;
    
    if proposal.status != ProposalStatus::Approved {
        return Err("Proposal is not Approved".to_string());
    }
    
    // Execute based on type
    match proposal.proposal_type {
        ProposalType::Treasury => {
            // Call Treasury Canister instead of executing locally
            let treasury_canister = TREASURY_CANISTER.with(|t| *t.borrow().get());
            
            let recipient = proposal.recipient.ok_or("Missing recipient")?;
            let amount = proposal.amount.ok_or("Missing amount")?;
            
            // Request transfer from treasury (multi-sig happens there)
            let (result,): (Result<u64, String>,) = ic_cdk::call(
                treasury_canister,
                "request_transfer",
                (proposal_id, recipient, amount)
            ).await.map_err(|(code, msg)| {
                format!("Treasury call failed: {:?} {}", code, msg)
            })?;
            
            result?;
        }
        ProposalType::AddBoardMember => {
            // Board member additions stay in governance
            execute_board_member_proposal_internal(&proposal)?;
        }
    }
    
    let mut proposal = proposal;
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.with(|p| p.borrow_mut().insert(proposal_id, proposal));
    
    Ok(())
}
```

---

## Comparison

| Aspect | Unified (Current + Multi-Sig) | Split (Treasury + Multi-Sig) |
|--------|-------------------------------|------------------------------|
| **Code Organization** | Mixed: governance + treasury + multi-sig | Separated: treasury has its own multi-sig |
| **Security Layers** | 1 canister with 2 authorization steps | 2 canisters with independent security |
| **Flexibility** | Limited: same multi-sig for all transfers | Flexible: can have tiered multi-sig (e.g., >$10k needs 3-of-5) |
| **Upgrade Risk** | High: governance changes might affect treasury | Low: can upgrade independently |
| **Attack Surface** | Larger: both systems in one canister | Smaller: treasury is isolated |
| **Complexity** | Medium: more code in one file | Medium-High: inter-canister calls |
| **Performance** | Fast: no inter-canister calls | Slower: requires cross-canister messaging |
| **Testability** | Harder: tightly coupled | Easier: can test treasury in isolation |

---

## Recommended Implementation

### **Short Term (Next 3-6 months):**
**Stay unified, add multi-sig if needed**

If you need multi-sig quickly and your treasury value is moderate (<$50k):
- Implement Approach 1 (unified multi-sig)
- Keep code modular (separate functions for treasury vs governance)
- Document the separation for future split

### **Long Term (6-12 months+):**
**Split architecture when:**

1. **Treasury value exceeds $100k** - justify the added complexity
2. **You need advanced treasury features:**
   - Tiered multi-sig ($1k = 2-of-3, $10k = 3-of-5)
   - Time-locks (24hr delay for large transfers)
   - Rate limiting (max $X per day)
   - Multi-token support (GHC, USDC, ICP, BTC)
3. **Separate upgrade cycles** - treasury logic changes frequently
4. **Regulatory compliance** - auditors require isolated treasury

---

## Example: Tiered Multi-Sig (Only Possible in Split)

This shows the flexibility advantage of split architecture:

```rust
// treasury/src/lib.rs

fn required_signatures_for_amount(amount: u64) -> u8 {
    // Amount in tokens (divide by 10^8)
    let tokens = amount / 100_000_000;
    
    match tokens {
        0..=1000 => 2,        // Under $1k: 2-of-5
        1001..=10000 => 3,    // $1k-$10k: 3-of-5
        10001..=100000 => 4,  // $10k-$100k: 4-of-5
        _ => 5,               // Over $100k: 5-of-5 (all signers)
    }
}

#[update]
async fn approve_transfer(request_id: u64) -> Result<(), String> {
    // ... approval logic ...
    
    let request = PENDING_TRANSFERS.with(|t| t.borrow().get(&request_id))
        .ok_or("Transfer request not found")?;
    
    // Dynamic threshold based on amount
    let required_sigs = required_signatures_for_amount(request.amount);
    
    if approval_count >= required_sigs {
        execute_transfer_internal(request_id).await?;
    }
    
    Ok(())
}
```

This level of sophistication is **much cleaner in a split architecture** because:
- Treasury canister focuses solely on financial security
- Can have different multi-sig policies without touching governance
- Auditors can review treasury in isolation

---

## Summary

**Why Multi-Sig is Easier in Split Architecture:**

1. ✅ **Clear Separation**: Governance handles voting, Treasury handles security
2. ✅ **Independent Policies**: Can change multi-sig rules without touching governance
3. ✅ **Better Security**: Treasury is isolated from governance bugs
4. ✅ **Flexibility**: Easy to add features like tiered multi-sig, time-locks, rate limits
5. ✅ **Auditability**: Treasury can be audited independently

**But for now:**
- Your current unified architecture works well
- If you need multi-sig **immediately**, add it to the current canister
- **Plan for a split when treasury value/complexity justifies it**

---

*Last updated: January 2026*
