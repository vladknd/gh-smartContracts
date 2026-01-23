# Admin Actions & Initial Setup

This guide details administrative actions available to canister controllers (admins). These actions are primarily used for system initialization, emergency maintenance, and testing.

## ⚠️ Important Note
Most actions listed here are **Controller Only** actions. This means they must be executed by the identity that controls the canisters (usually the identity that deployed them).

Once the system is fully initialized and locked, day-to-day management moves to the **Governance Proposal System**.

---

## 1. Board Member Initialization

**Goal**: Assign the initial board members (e.g., the first 3 founders) and then LOCK the configuration.
**Critical**: After locking, board members can **only** be added or removed via `AddBoardMember` governance proposals. This ensures decentralization.

### Step 1: Assign Initial Board Members
Use `set_board_member_shares` on the `governance_canister`.

**Rules:**
- Total percentage sum MUST equal exactly `100`.
- All percentages must be integers (`1` to `100`).

**Command (Bash):**
```bash
# Example: 3 Founders
# Founder 1: 50%
# Founder 2: 30%
# Founder 3: 20%

FOUNDER1="n7kyi-a2ccw-erzef-tzywo-kuqyh-4x6on-6ltec-flkee-5zcdi-62drw-gae"
FOUNDER2="vlg44-yhcay-gabeb-4qse6-k4tvu-ujdun-jyzjc-aq5nm-45hye-2qsr5-3ae"
FOUNDER3="zgmvl-h4shw-3tgr2-sunlv-nujve-qh7o4-mwolg-hmais-ymglc-yt4q7-cqe"

dfx canister call governance_canister set_board_member_shares "(vec {
  record { member = principal \"$FOUNDER1\"; percentage = 60 };
  record { member = principal \"$FOUNDER2\"; percentage = 30 };
  record { member = principal \"$FOUNDER3\"; percentage = 10 };
})"
```

### Step 2: Verify Configuration
Check that the shares are set correctly.

```bash
dfx canister call governance_canister get_board_member_shares
```

### Step 3: Lock Board Configuration
Once the initial board is verified, you **MUST** lock it to transfer control to the governance protocol.

**Command:**
```bash
dfx canister call governance_canister lock_board_member_shares
```

**Verify Lock Status:**
```bash
dfx canister call governance_canister are_board_shares_locked
# Output should be: (true)
```

---

## 2. Testing & Debugging (Admin Only)

These commands are strictly for testing and debugging purposes. They allow skipping voting delays.

### Force Proposal Status
Useful for testing execution logic without waiting 2 weeks for voting.

```bash
# Force Approve Proposal ID 1
dfx canister call governance_canister admin_set_proposal_status '(0 : nat64, variant { Approved })'

# Force Reject Proposal ID 1
dfx canister call governance_canister admin_set_proposal_status '(0 : nat64, variant { Rejected })'
```

---

## 3. Staking Hub Administration

### Manually Set User Shard
In rare cases (e.g., shard migration or error recovery), an admin may need to manually map a user to a specific shard.

```bash
dfx canister call staking_hub admin_set_user_shard '(principal "USER_ID", principal "SHARD_ID")'
```

---

## 4. Emergency & Upgrades

### Canister Upgrades
Controllers can upgrade canister code using `dfx`.

```bash
dfx canister install governance_canister --mode upgrade
```

### Re-Linking Canisters (If IDs Change)
If you redeploy a canister and its ID changes, you may need to update the references in other canisters.

**Governance Canister:**
```bash
# Update Treasury Link
dfx canister call governance_canister set_treasury_canister_id '(principal "NEW_TREASURY_ID")'
```

**Treasury Canister:**
```bash
# Update Governance Link
dfx canister call treasury_canister set_governance_canister_id '(principal "NEW_GOVERNANCE_ID")'
```
