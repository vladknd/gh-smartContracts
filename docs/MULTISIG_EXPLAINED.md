# Multi-Signature Explained: How It Actually Works

This document explains how multi-signature (multi-sig) works in the context of Internet Computer canisters and your GreenHero dApp.

---

## 🔑 Understanding Principals and Wallets

### What is a Principal?

On the Internet Computer, every user has a **Principal** - this is their unique identifier/address.

```
Example Principals:
- Alice:   brcis-myp3t-sgc2i-7fzce-onoy6-4cknk-o7zrq-rp2yj-r3adh-wwjm5-2ae
- Bob:     cjd7b-pozyi-lvcpt-i2dnz-7ubh5-v5xgm-imvzz-46ecb-2ytr4-ypfor-yqe
- Charlie: bqnyh-6ivts-3zmpt-ykoof-ggqza-2jgjp-wri4l-nxhn6-iv4ni-owd53-iae
```

When a user:
- **Logs into your dApp** → They authenticate with Internet Identity (or other wallet)
- **Makes a canister call** → The canister sees their Principal as `ic_cdk::caller()`
- **Each action is signed** → Using their private key (handled by their wallet/II)

---

## 💰 Who Actually Holds the Funds?

### **Critical Point: The CANISTER holds the funds, not individual users!**

```
❌ WRONG (Traditional Blockchain Multi-Sig):
┌─────────────────────────────────────────┐
│  Shared Multi-Sig Wallet Address       │
│  (Requires 2-of-3 signatures to spend)  │
│                                         │
│  Joint custody by Alice, Bob, Charlie   │
└─────────────────────────────────────────┘

✅ CORRECT (ICP Canister Multi-Sig):
┌─────────────────────────────────────────┐
│  Operational Governance Canister        │
│  Principal: x7gvf-...                   │
│                                         │
│  Balance: 1,000,000 GHC                 │
│  (Canister controls these funds)        │
└─────────────────────────────────────────┘
         ↓
    Spending requires:
    - Alice approves (calls approve_execution)
    - Bob approves (calls approve_execution)
    - Then canister executes transfer
```

---

## 🎯 How Multi-Sig Works in Your System

### Current System (Without Multi-Sig)

```
┌──────────────────────────────────────────────────────────────┐
│                  OPERATIONAL GOVERNANCE CANISTER             │
│                  Principal: x7gvf-...                        │
│                                                              │
│  Treasury Balance: 1,000,000 GHC                             │
│                                                              │
│  Flow:                                                       │
│  1. Community votes on proposal → Approved (15,000 VP)       │
│  2. Anyone calls execute_proposal(5) → ✅ Funds transfer     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Problem**: Once approved by community vote, **anyone** can execute and the funds immediately transfer. No additional security check.

---

### With Multi-Sig (2-of-3 Signers)

```
┌──────────────────────────────────────────────────────────────┐
│                  OPERATIONAL GOVERNANCE CANISTER             │
│                  Principal: x7gvf-...                        │
│                                                              │
│  Treasury Balance: 1,000,000 GHC                             │
│                                                              │
│  Authorized Signers (stored in canister state):              │
│    [Alice's Principal, Bob's Principal, Charlie's Principal] │
│                                                              │
│  Flow:                                                       │
│  1. Community votes → Approved (15,000 VP)                   │
│  2. Alice calls: approve_execution(5) ✅                     │
│  3. Bob calls: approve_execution(5) ✅                       │
│  4. Threshold reached (2-of-3) → Funds transfer              │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 👥 How Each Board Member Interacts

### Step-by-Step Example

#### **Setup Phase (One-time, by controllers):**

```bash
# Controllers set the authorized signers
dfx canister call operational_governance set_authorized_signers '(vec {
  principal "brcis-myp3t-sgc2i-7fzce-onoy6-4cknk-o7zrq-rp2yj-r3adh-wwjm5-2ae";  # Alice
  principal "cjd7b-pozyi-lvcpt-i2dnz-7ubh5-v5xgm-imvzz-46ecb-2ytr4-ypfor-yqe";  # Bob
  principal "bqnyh-6ivts-3zmpt-ykoof-ggqza-2jgjp-wri4l-nxhn6-iv4ni-owd53-iae";  # Charlie
})'
```

Now Alice, Bob, and Charlie are authorized signers. **They don't share a wallet - each has their own Principal/identity**.

---

#### **Execution Phase (When a proposal is approved):**

**Scenario**: Proposal #5 was approved by community vote to send 10,000 GHC to a recipient.

**Alice's action** (From her computer, logged in as Alice):
```bash
# Alice checks the proposal details first
dfx canister call operational_governance get_proposal '(5 : nat64)'

# Alice verifies the recipient address and amount, then approves
dfx identity use alice  # Uses Alice's identity/principal
dfx canister call operational_governance approve_execution '(5 : nat64)'

# Response: "Approval recorded. 1 of 2 signatures collected."
```

**What just happened?**
- Alice's wallet/browser signed the request using her private key
- The canister received the call from Alice's Principal
- The canister recorded: "Alice approved proposal #5"
- Alice's account/balance is unchanged - she didn't send any funds

---

**Bob's action** (From his computer, logged in as Bob):
```bash
# Bob also reviews the proposal
dfx canister call operational_governance get_proposal '(5 : nat64)'

# Bob approves
dfx identity use bob  # Uses Bob's identity/principal  
dfx canister call operational_governance approve_execution '(5 : nat64)'

# Response: "Approval recorded. 2 of 2 signatures collected. Transfer executing..."
```

**What just happened?**
- Bob's wallet signed the request using his private key
- The canister received the call from Bob's Principal
- The canister recorded: "Bob approved proposal #5"
- **Threshold reached!** (2 approvals out of 3 required signers)
- The canister **itself** initiates the ICRC-1 transfer from its own balance
- Bob's account/balance is unchanged

---

**Charlie** (Didn't need to approve this time):
```bash
# Charlie could have approved instead of Bob, but since 2-of-3 was already met,
# his approval is not needed. He can check the status:
dfx identity use charlie
dfx canister call operational_governance get_execution_approvals '(5 : nat64)'

# Response: [Alice's Principal, Bob's Principal]
```

---

## 🏦 The Canister is the Treasury

### Key Concept: Canister as the "Vault"

```
Traditional Bank Analogy:
┌─────────────────────────────────────┐
│         Bank Vault                  │
│    Contains: $1,000,000             │
│                                     │
│    Dual-Control System:             │
│    - Requires 2 keys to open        │
│    - Alice has Key #1               │
│    - Bob has Key #2                 │
│    - Charlie has Key #3             │
│    - Any 2 keys open the vault      │
└─────────────────────────────────────┘
```

**In your ICP canister:**

```
Operational Governance Canister:
┌─────────────────────────────────────┐
│  Treasury Balance: 1,000,000 GHC    │
│  (Stored in canister's account)     │
│                                     │
│  Authorization System:              │
│  - Requires 2 approvals to spend    │
│  - Alice can approve                │
│  - Bob can approve                  │
│  - Charlie can approve              │
│  - Any 2 approvals execute transfer │
└─────────────────────────────────────┘
```

The canister account on the GHC ledger looks like:
```
Account: {
  owner: operational_governance_canister_principal,
  subaccount: None
}
```

**Nobody individually controls these funds** - only the canister can spend them, and the canister's code enforces the multi-sig rules.

---

## 🔐 How Authentication Works

### The Magic: `ic_cdk::caller()`

When Alice calls `approve_execution(5)`:

```rust
#[update]
fn approve_execution(proposal_id: u64) -> Result<u8, String> {
    let caller = ic_cdk::caller();  // Returns Alice's Principal
    
    // Check if caller is an authorized signer
    let is_authorized = AUTHORIZED_SIGNERS.with(|s| {
        s.borrow().get().contains(&caller)  // Is Alice in the list?
    });
    
    if !is_authorized {
        return Err("You are not an authorized signer".to_string());
    }
    
    // Record approval FROM THIS SPECIFIC PRINCIPAL
    EXECUTION_APPROVALS.with(|a| {
        let mut map = a.borrow_mut();
        let mut approvals = map.get(&proposal_id).unwrap_or_default();
        approvals.insert(caller);  // Add Alice's principal to the set
        map.insert(proposal_id, approvals);
    });
    
    // Check if threshold is met...
}
```

**The Internet Computer guarantees:**
- Only Alice can make a call that shows `caller == Alice's Principal`
- Alice must sign the call with her private key (via her wallet/II)
- The canister can trust `ic_cdk::caller()` - it's cryptographically verified

---

## 📱 How Users Actually Sign

### In Your Frontend (React/Vue/etc.)

When Alice clicks "Approve Execution":

```javascript
// frontend/src/components/ProposalDetails.tsx

const handleApprove = async (proposalId) => {
  try {
    // User must be authenticated (Internet Identity, Plug, etc.)
    const actor = await createActor(operationalGovernanceCanisterId, {
      agentOptions: {
        identity: getCurrentIdentity(), // Alice's identity from her wallet
      },
    });
    
    // This call is signed by Alice's private key (handled by her wallet)
    const result = await actor.approve_execution(proposalId);
    
    alert(`Approval recorded! ${result} of 2 signatures collected.`);
  } catch (error) {
    alert(`Error: ${error.message}`);
  }
};
```

**Behind the scenes:**
1. Alice's browser/wallet holds her private key (or connects to II)
2. The frontend creates a signed message: "Alice approves proposal #5"
3. The Internet Computer verifies the signature
4. The canister receives: `caller = Alice's Principal`
5. No shared wallet or shared keys!

---

## 🆚 Comparison: Traditional vs ICP Multi-Sig

### Traditional Blockchain (e.g., Ethereum Gnosis Safe)

```
┌─────────────────────────────────────────────┐
│  Multi-Sig Contract Address: 0x1234...      │
│  Balance: 100 ETH                           │
│                                             │
│  Required Signatures: 2-of-3               │
│  Owners: [0xAlice, 0xBob, 0xCharlie]       │
│                                             │
│  To spend:                                  │
│  1. Propose transaction                     │
│  2. Alice signs with her private key → Sig1│
│  3. Bob signs with his private key → Sig2  │
│  4. Submit transaction with both signatures│
│  5. Contract verifies sigs → Funds transfer│
└─────────────────────────────────────────────┘

Problems:
- Complex cryptographic signature aggregation
- Gas fees for each signature submission
- Requires off-chain coordination
```

### ICP Canister (Your System)

```
┌─────────────────────────────────────────────┐
│  Operational Governance Canister: x7gvf... │
│  Balance: 1,000,000 GHC                     │
│                                             │
│  Required Approvals: 2-of-3                │
│  Authorized: [Alice, Bob, Charlie]          │
│                                             │
│  To spend:                                  │
│  1. Proposal approved by community          │
│  2. Alice calls: approve_execution(5)       │
│  3. Bob calls: approve_execution(5)         │
│  4. Canister checks: 2 approvals? ✅        │
│  5. Canister executes transfer              │
└─────────────────────────────────────────────┘

Advantages:
- Simple: Just count principals in a set
- No signature aggregation needed
- Caller authentication built into IC
- Native canister function calls
```

---

## 🎭 Real-World Example Walkthrough

### Scenario: Treasury wants to pay a developer 5,000 GHC

**Cast:**
- **Alice**: Board member, has Principal ending in ...5-2ae
- **Bob**: Board member, has Principal ending in ...r-yqe  
- **Charlie**: Board member, has Principal ending in ...3-iae
- **Developer**: Recipient, Principal: dev-...

---

**Step 1: Community Proposal**
- Someone creates proposal: "Pay developer 5,000 GHC for work"
- Community votes → Proposal #12 reaches Approved status

**Step 2: Multi-Sig Approvals**

Alice logs into the dApp:
```
1. Alice opens her browser
2. Authenticates with Internet Identity (or Plug wallet)
3. Her identity/principal is loaded: ...5-2ae
4. She navigates to Proposal #12
5. Clicks "Approve Execution"
6. Her wallet signs the request
7. Backend calls: approve_execution(12) as Alice
8. Canister records: approvals[12] = {Alice}
```

Bob (from a different computer, different location):
```
1. Bob opens his browser  
2. Authenticates with his own Internet Identity
3. His identity/principal is loaded: ...r-yqe
4. He navigates to Proposal #12
5. Sees: "1 of 2 approvals (Alice approved)"
6. Clicks "Approve Execution"
7. His wallet signs the request
8. Backend calls: approve_execution(12) as Bob
9. Canister records: approvals[12] = {Alice, Bob}
10. ✅ Threshold met! Canister executes transfer
11. Developer receives 5,000 GHC
```

Charlie (optional - didn't need to participate):
```
Charlie could have approved instead of Bob.
Any 2 out of the 3 authorized signers is sufficient.
```

---

## 🔒 Security Properties

### What Multi-Sig Protects Against:

✅ **Compromised Single Account**
- If Alice's laptop is hacked, attacker still needs Bob or Charlie
- Single stolen key cannot drain treasury

✅ **Malicious Insider**
- If Alice goes rogue, she can't execute transfers alone
- Requires collusion of at least 2 signers

✅ **Phishing/Social Engineering**
- Attacker tricks Alice into approving → Still needs Bob/Charlie
- Multiple parties review transaction details

✅ **Fat-Finger Errors**
- Alice accidentally approves wrong amount
- Bob reviews and refuses to approve → Transaction blocked

✅ **Regulatory Compliance**
- Dual control requirement for financial transactions
- Clear audit trail of who approved what

---

### What Multi-Sig Does NOT Protect Against:

❌ **Smart Contract Bugs**
- If the canister code has a vulnerability, multi-sig doesn't help
- All signers are trusting the canister's logic

❌ **Majority Collusion**
- If 2 out of 3 signers collude, they can steal funds
- Choose trustworthy signers!

❌ **Governance Takeover**
- If attackers gain 15,000 voting power, they can create malicious proposals
- Multi-sig only protects execution, not proposal approval

---

## 📝 Summary

### Key Takeaways:

1. **Each signer has their own Principal/identity** - no shared wallets
2. **The canister holds the funds**, not individuals
3. **Signers approve by calling a canister function** - `approve_execution(proposal_id)`
4. **The canister tracks approvals** - using `ic_cdk::caller()` to identify who approved
5. **When threshold is met**, the canister executes the transfer from its own balance
6. **Each approval is signed** by the individual's private key (via their wallet)

### It's Not About Shared Keys, It's About Distributed Authorization

```
Think of it like:
- A vault with $1M inside
- The vault has a computer lock with 3 fingerprint sensors
- Any 2 fingerprints open the vault
- Alice, Bob, Charlie each have unique fingerprints
- No one shares fingerprints, but any 2 can collaborate to open

The canister = vault with computer lock
Authorized signers = fingerprints in the system  
approve_execution() = placing finger on sensor
Threshold met = vault opens and executes transfer
```

---

*Last updated: January 2026*
