# Identity Recovery & Security Strategies for Multi-Sig

This document addresses critical security scenarios: what happens when a board member's principal is compromised, lost, or needs to be replaced.

> **⚠️ ARCHITECTURE UPDATE (January 2026)**
>
> This document references `operational_governance` which has been refactored into:
> - **`treasury_canister`**: Token custody, multi-sig for treasury transfers
> - **`governance_canister`**: Proposals, voting, board member management
>
> Multi-sig and board member management are now in `governance_canister`.

---

## 🚨 The Problem

### Scenario 1: Compromised Principal (Hacked)
```
Alice's laptop is stolen → Attacker has access to her Internet Identity
→ Attacker can approve proposals as Alice!
→ If 1 more signer is compromised, treasury is at risk
```

### Scenario 2: Lost Principal (Lost Access)
```
Bob forgets his Internet Identity recovery phrase
→ Bob can never approve again
→ System now requires 2-of-2 instead of 2-of-3
→ System becomes MORE fragile
```

### Scenario 3: Personnel Change
```
Charlie leaves the organization
→ Need to remove his approval authority
→ Need to add replacement board member
→ How to do this securely?
```

---

## ✅ Solutions Overview

| Solution | Security | Complexity | Best For |
|----------|----------|------------|----------|
| **1. Multiple Recovery Principals** | ⭐⭐⭐⭐ | Low | Most systems |
| **2. Governance-Based Removal** | ⭐⭐⭐⭐⭐ | Medium | Decentralized DAOs |
| **3. Emergency Controller Override** | ⭐⭐⭐ | Low | Early stage projects |
| **4. Time-Locked Rotation** | ⭐⭐⭐⭐ | High | High-security systems |

---

## Solution 1: Multiple Recovery Principals (RECOMMENDED)

### Concept
Each board member can register **multiple principals** as backup/recovery addresses.

```
Alice (Board Member):
├─ Primary Principal:   alice-main-principal...2ae
├─ Recovery Principal:  alice-backup-principal...7xy
└─ Hardware Principal:  alice-ledger-principal...9qz
```

### Implementation

#### Data Structure

```rust
use std::collections::HashSet;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BoardMember {
    pub name: String,                    // "Alice"
    pub principals: HashSet<Principal>,  // Multiple principals for this person
    pub percentage: u8,                  // Voting share (for governance)
}

thread_local! {
    // Map: Any principal → Board Member ID
    static PRINCIPAL_TO_MEMBER: RefCell<StableBTreeMap<Principal, String, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12)))
        ));
    
    // Map: Member ID → Board Member data
    static BOARD_MEMBERS: RefCell<StableBTreeMap<String, BoardMember, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13)))
        ));
    
    // Authorized signers for multi-sig (any principal from any board member)
    static AUTHORIZED_SIGNERS: RefCell<StableCell<HashSet<Principal>, Memory>> = /* ... */;
}
```

#### Admin Functions

```rust
/// Add a board member with multiple principals
#[update]
fn add_board_member_with_recovery(
    member_id: String,
    name: String,
    primary_principal: Principal,
    recovery_principals: Vec<Principal>,
    percentage: u8,
) -> Result<(), String> {
    // Only controllers can add members
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    // Create set of all principals
    let mut all_principals = HashSet::new();
    all_principals.insert(primary_principal);
    for p in recovery_principals {
        all_principals.insert(p);
    }
    
    // Create board member
    let member = BoardMember {
        name,
        principals: all_principals.clone(),
        percentage,
    };
    
    // Store member
    BOARD_MEMBERS.with(|m| {
        m.borrow_mut().insert(member_id.clone(), member)
    });
    
    // Map each principal to this member
    PRINCIPAL_TO_MEMBER.with(|p| {
        let mut map = p.borrow_mut();
        for principal in all_principals.iter() {
            map.insert(*principal, member_id.clone());
        }
    });
    
    // Add all principals as authorized signers
    AUTHORIZED_SIGNERS.with(|s| {
        let mut signers = s.borrow().get().clone();
        signers.extend(all_principals);
        s.borrow_mut().set(signers).expect("Failed to update signers");
    });
    
    Ok(())
}

/// Add a recovery principal for an existing member
#[update]
fn add_recovery_principal(
    member_id: String,
    new_principal: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Get the member
    let mut member = BOARD_MEMBERS.with(|m| {
        m.borrow().get(&member_id)
    }).ok_or("Member not found")?;
    
    // Verify caller is one of this member's existing principals
    if !member.principals.contains(&caller) {
        return Err("Only the member can add recovery principals".to_string());
    }
    
    // Check if already exists
    if member.principals.contains(&new_principal) {
        return Err("Principal already registered for this member".to_string());
    }
    
    // Add new principal
    member.principals.insert(new_principal);
    
    // Update member
    BOARD_MEMBERS.with(|m| {
        m.borrow_mut().insert(member_id.clone(), member)
    });
    
    // Map new principal to member
    PRINCIPAL_TO_MEMBER.with(|p| {
        p.borrow_mut().insert(new_principal, member_id)
    });
    
    // Add to authorized signers
    AUTHORIZED_SIGNERS.with(|s| {
        let mut signers = s.borrow().get().clone();
        signers.insert(new_principal);
        s.borrow_mut().set(signers).expect("Failed to update signers");
    });
    
    Ok(())
}

/// Remove a compromised principal
#[update]
fn remove_compromised_principal(
    member_id: String,
    compromised_principal: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Get the member
    let mut member = BOARD_MEMBERS.with(|m| {
        m.borrow().get(&member_id)
    }).ok_or("Member not found")?;
    
    // Verify caller is one of this member's OTHER principals (not the compromised one)
    if !member.principals.contains(&caller) || caller == compromised_principal {
        return Err("Only the member (using a different principal) can remove a compromised principal".to_string());
    }
    
    // Ensure at least one principal remains
    if member.principals.len() <= 1 {
        return Err("Cannot remove the last principal. Add a recovery principal first.".to_string());
    }
    
    // Remove principal
    member.principals.remove(&compromised_principal);
    
    // Update member
    BOARD_MEMBERS.with(|m| {
        m.borrow_mut().insert(member_id.clone(), member)
    });
    
    // Remove mapping
    PRINCIPAL_TO_MEMBER.with(|p| {
        p.borrow_mut().remove(&compromised_principal)
    });
    
    // Remove from authorized signers
    AUTHORIZED_SIGNERS.with(|s| {
        let mut signers = s.borrow().get().clone();
        signers.remove(&compromised_principal);
        s.borrow_mut().set(signers).expect("Failed to update signers");
    });
    
    Ok(())
}
```

#### Modified Approval Function

```rust
/// Approve execution (works with any of the member's principals)
#[update]
fn approve_execution(proposal_id: u64) -> Result<u8, String> {
    let caller = ic_cdk::caller();
    
    // Get the member ID for this principal
    let member_id = PRINCIPAL_TO_MEMBER.with(|p| {
        p.borrow().get(&caller)
    }).ok_or("Caller is not an authorized board member")?;
    
    // Check if this member has already approved (via ANY of their principals)
    let already_approved = EXECUTION_APPROVALS.with(|a| {
        let map = a.borrow();
        if let Some(approvals) = map.get(&proposal_id) {
            // Get all principals for this member
            let member_principals = BOARD_MEMBERS.with(|m| {
                m.borrow()
                    .get(&member_id)
                    .map(|member| member.principals.clone())
                    .unwrap_or_default()
            });
            
            // Check if any of this member's principals already approved
            approvals.iter().any(|p| member_principals.contains(p))
        } else {
            false
        }
    });
    
    if already_approved {
        return Err("You have already approved this proposal (from another principal)".to_string());
    }
    
    // Record approval from THIS specific principal
    let approval_count = EXECUTION_APPROVALS.with(|a| {
        let mut map = a.borrow_mut();
        let mut approvals = map.get(&proposal_id).unwrap_or_default();
        approvals.insert(caller);
        let count = approvals.len() as u8;
        map.insert(proposal_id, approvals);
        count
    });
    
    // Check threshold (based on number of MEMBERS, not principals)
    let unique_members_approved = count_unique_members_who_approved(proposal_id);
    
    if unique_members_approved >= REQUIRED_SIGNATURES {
        // Execute proposal
        ic_cdk::spawn(async move {
            let _ = execute_proposal_internal(proposal_id).await;
        });
    }
    
    Ok(unique_members_approved)
}

/// Helper: Count how many unique board members have approved
fn count_unique_members_who_approved(proposal_id: u64) -> u8 {
    EXECUTION_APPROVALS.with(|a| {
        let approvals = a.borrow()
            .get(&proposal_id)
            .unwrap_or_default();
        
        let mut unique_members = HashSet::new();
        
        for principal in approvals {
            if let Some(member_id) = PRINCIPAL_TO_MEMBER.with(|p| p.borrow().get(&principal)) {
                unique_members.insert(member_id);
            }
        }
        
        unique_members.len() as u8
    })
}
```

#### Query Functions

```rust
#[query]
fn get_my_board_member_info() -> Option<BoardMember> {
    let caller = ic_cdk::caller();
    
    let member_id = PRINCIPAL_TO_MEMBER.with(|p| p.borrow().get(&caller))?;
    
    BOARD_MEMBERS.with(|m| m.borrow().get(&member_id))
}

#[query]
fn get_all_board_members() -> Vec<(String, BoardMember)> {
    BOARD_MEMBERS.with(|m| m.borrow().iter().collect())
}
```

---

## Solution 2: Governance-Based Emergency Removal

### Concept
Use the existing governance voting system to replace compromised signers.

```
1. Alice's account is compromised
2. Bob creates emergency proposal: "Remove Alice's principal ...2ae"
3. Charlie votes YES → Approved
4. Execute → Alice's compromised principal is removed
5. Create new proposal: "Add Alice's new principal ...5xy"
6. Vote → Approved → Alice regains access with new principal
```

### Implementation

```rust
/// New proposal type
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ProposalType {
    Treasury,
    AddBoardMember,
    RemoveSignerPrincipal,  // NEW
    AddSignerPrincipal,     // NEW
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RemoveSignerPayload {
    pub principal: Principal,
    pub reason: String,  // "Compromised" or "Lost access"
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AddSignerPayload {
    pub member_name: String,
    pub new_principal: Principal,
}

/// Create proposal to remove compromised signer
#[update]
async fn create_remove_signer_proposal(
    principal_to_remove: Principal,
    reason: String,
) -> Result<u64, String> {
    // Create proposal through normal governance flow
    // Requires community vote to approve
    // When executed, removes the principal from authorized signers
    
    // ... (similar to create_treasury_proposal)
}

/// Execute removal
fn execute_remove_signer_proposal(proposal: &Proposal) -> Result<(), String> {
    let payload = proposal.remove_signer_payload.as_ref()
        .ok_or("Missing payload")?;
    
    AUTHORIZED_SIGNERS.with(|s| {
        let mut signers = s.borrow().get().clone();
        signers.remove(&payload.principal);
        s.borrow_mut().set(signers).expect("Failed to update signers");
    });
    
    Ok(())
}
```

**Advantages:**
- Decentralized decision
- Clear audit trail
- Community oversight

**Disadvantages:**
- Takes 2 weeks (voting period)
- Might be too slow for emergencies

---

## Solution 3: Emergency Controller Override

### Concept
Controllers (admins) can perform emergency removals with a time delay.

```rust
const EMERGENCY_TIMELOCK: u64 = 48 * 60 * 60 * 1_000_000_000; // 48 hours

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EmergencyAction {
    pub action: String,
    pub principal: Principal,
    pub proposed_at: u64,
    pub executed: bool,
}

thread_local! {
    static EMERGENCY_ACTIONS: RefCell<StableBTreeMap<u64, EmergencyAction, Memory>> = /* ... */;
}

/// Propose emergency removal (controller only)
#[update]
fn propose_emergency_removal(principal: Principal) -> Result<u64, String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    let action_id = generate_action_id();
    let action = EmergencyAction {
        action: "remove_signer".to_string(),
        principal,
        proposed_at: ic_cdk::api::time(),
        executed: false,
    };
    
    EMERGENCY_ACTIONS.with(|a| {
        a.borrow_mut().insert(action_id, action)
    });
    
    Ok(action_id)
}

/// Execute emergency removal (after timelock)
#[update]
fn execute_emergency_removal(action_id: u64) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    
    let mut action = EMERGENCY_ACTIONS.with(|a| {
        a.borrow().get(&action_id)
    }).ok_or("Action not found")?;
    
    if action.executed {
        return Err("Already executed".to_string());
    }
    
    let now = ic_cdk::api::time();
    if now < action.proposed_at + EMERGENCY_TIMELOCK {
        let hours_remaining = (action.proposed_at + EMERGENCY_TIMELOCK - now) / (60 * 60 * 1_000_000_000);
        return Err(format!("Timelock not expired. {} hours remaining.", hours_remaining));
    }
    
    // Remove signer
    AUTHORIZED_SIGNERS.with(|s| {
        let mut signers = s.borrow().get().clone();
        signers.remove(&action.principal);
        s.borrow_mut().set(signers).expect("Failed to update signers");
    });
    
    // Mark as executed
    action.executed = true;
    EMERGENCY_ACTIONS.with(|a| {
        a.borrow_mut().insert(action_id, action)
    });
    
    Ok(())
}
```

**Advantages:**
- Fast response to emergencies
- 48hr timelock gives community time to object

**Disadvantages:**
- Centralized (requires trusted controllers)
- Can be abused if controllers are malicious

---

## Solution 4: Internet Identity Recovery Integration

### Concept
Leverage Internet Identity's built-in recovery mechanisms.

Internet Identity already supports:
- **Recovery phrases** (24-word seed)
- **Recovery devices** (add phone, laptop, hardware key)
- **Social recovery** (with delegates)

**Recommendation**: Educate board members to:
1. Set up multiple recovery devices in Internet Identity
2. Store recovery phrase securely (written down, not digital)
3. Add a hardware key (like YubiKey or Ledger) as a device

**No code changes needed** - this is handled by Internet Identity infrastructure.

---

## 🎯 RECOMMENDED APPROACH FOR YOUR SYSTEM

### **Use a Hybrid Strategy:**

1. **Primary**: Multiple Recovery Principals (Solution 1)
   - Each board member registers 2-3 principals
   - Easy self-service recovery

2. **Backup**: Governance-Based Removal (Solution 2)
   - If member loses ALL principals
   - Community can vote to replace them

3. **Emergency**: Controller Override with Timelock (Solution 3)
   - For critical security incidents
   - 48-hour timelock for transparency

---

## 📋 Implementation Checklist

### Phase 1: Basic Recovery (Recommended for MVP)
- [x] Support multiple principals per board member
- [x] Allow members to add recovery principals themselves
- [x] Allow members to remove compromised principals
- [x] Update approval logic to count unique members, not principals

### Phase 2: Governance Safety Net
- [ ] Add proposal type for removing/adding signers
- [ ] Implement governance-based signer management
- [ ] Create emergency proposal fast-track (shorter voting period)

### Phase 3: Advanced Security
- [ ] Emergency controller override with timelock
- [ ] Audit log of all signer changes
- [ ] Notification system for security events

---

## 🛡️ Best Practices for Board Members

### Setup Checklist for Each Board Member:

1. **Primary Internet Identity**
   - Create with strong anchor
   - Add recovery phrase (write it down!)
   - Never share recovery phrase

2. **Add Recovery Devices**
   - Add phone as recovery device
   - Add hardware key (recommended)
   - Add laptop as backup

3. **Register Recovery Principal in Canister**
   ```bash
   # Login with primary identity
   dfx identity use alice-primary
   
   # Get your member ID
   dfx canister call governance_canister get_my_board_member_info
   
   # Add recovery principal
   dfx canister call governance_canister add_recovery_principal '(
     "alice",  
     principal "alice-backup-principal..."
   )'
   ```

4. **Test Recovery**
   - Verify you can approve with recovery principal
   - Document the process

5. **Secure Storage**
   - Physical copy of recovery phrase in safe
   - Hardware key in separate location
   - Document what family should do if you're incapacitated

---

## 🚨 Incident Response Procedures

### If a Principal is Compromised:

```
1. IMMEDIATE (within 1 hour):
   - Use recovery principal to call remove_compromised_principal()
   - This removes attacker's access instantly
   
2. SHORT-TERM (within 24 hours):
   - Generate new primary principal
   - Add as recovery principal
   - Verify approval still works
   
3. FOLLOW-UP (within 1 week):
   - Review all approvals made by compromised principal
   - Check for any suspicious activity
   - Document incident for audit trail
```

### If a Member Loses All Access:

```
1. Contact other board members
2. Create governance proposal to remove old principals
3. Create second proposal to add new principals
4. Vote on both proposals
5. Member regains access with new identity
6. Timeline: ~2 weeks (governance voting period)
```

---

## 📊 Comparison of Strategies

| Scenario | Solution 1 (Recovery) | Solution 2 (Governance) | Solution 3 (Emergency) |
|----------|----------------------|------------------------|------------------------|
| **Principal compromised** | ✅ Instant self-service | ⏰ 2 weeks | ⏰ 48 hours |
| **Lost all access** | ❌ Can't help | ✅ Community can replace | ✅ Admin can replace |
| **Decentralization** | ✅ Self-service | ✅ Community decision | ❌ Centralized |
| **Speed** | ✅ Immediate | ❌ Slow (2 weeks) | ⏰ Medium (48hrs) |
| **Complexity** | 🟡 Medium | 🟡 Medium | 🟢 Low |

---

## 💡 Summary & Recommendation

**For your GreenHero system, implement:**

1. **NOW (MVP)**: Multiple recovery principals per board member
   - Low complexity, high security
   - Self-service for compromised accounts
   - Each member maintains 2-3 principals

2. **SOON (After launch)**: Governance-based replacement
   - Safety net for lost access
   - Decentralized decision making
   - Uses existing proposal system

3. **FUTURE (If needed)**: Emergency override
   - Only if high treasury value (>$500k)
   - With timelock and transparency
   - Last resort for critical incidents

**This gives you:**
- ✅ Fast self-service recovery (instant)
- ✅ Community safety net (2 weeks)
- ✅ Emergency backstop (48 hours)
- ✅ Decentralized when possible
- ✅ Practical for real-world use

---

*Last updated: January 2026*
