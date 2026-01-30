# Admin Actions & Initial Setup

This guide details administrative actions available to canister controllers (admins). These actions are primarily used for system initialization, emergency maintenance, and testing.

## ⚠️ Important Note
Most actions listed here are **Controller Only** actions. This means they must be executed by the identity that controls the canisters (usually the identity that deployed them).

Once the system is fully initialized and locked, day-to-day management moves to the **Governance Proposal System**.

---

## 1. Board Member & Sentinel Initialization

**Goal**: Assign the sentinel member and initial board members, then LOCK the configuration.
**Critical**: Once locked, membership can **only** be changed via governance proposals.

### Step 1: Set the Sentinel Member
The **Sentinel** Role is special: they always have exactly **1 unit of voting power** (1 e8s). This ensures someone always has "loading power" even if zero tokens are staked.

**Command:**
```bash
SENTINEL="txfzn-yybdp-iygwe-ugtgi-v5lrs-l7ru7-gcbf7-zcp7a-alp22-i54sp-uae"

dfx canister call governance_canister set_sentinel_member "(principal \"$SENTINEL\")"
```

### Step 2: Assign Regular Board Member Shares
Use `set_board_member_shares`. These members share the **Voting Unit Capacity (VUC)** pool based on **Basis Points (BPS)** (10,000 = 100.00%).

**Rules:**
- Total BPS must equal exactly **10,000**.
- Each record must set `is_sentinel = false`.
- The Sentinel principal **cannot** be in this list.

**Command (Bash):**
```bash
# Example: 3 Regular Founders
# Founder 1: 60% (6000 BPS)
# Founder 2: 30% (3000 BPS)
# Founder 3: 10% (1000 BPS)
FOUNDER1="2kqdk-6gsni-zdcwc-pvwtw-a23zt-lh6li-tfoi3-ao3ct-yrrxx-kpsdx-dqe"
FOUNDER2="nb6zq-6blt3-uedgt-5ig4p-nybsn-3m7qo-f2hmn-he7ag-yyn3j-tiu4v-lae"
FOUNDER3="myoet-kkxy7-i5aiu-5mkcc-2fb7k-qlhaq-5wz4h-zuw5i-unnn6-fhwl7-jae"

dfx canister call governance_canister set_board_member_shares "(vec {
  record { member = principal \"$FOUNDER1\"; share_bps = 6000:nat16; is_sentinel = false };
  record { member = principal \"$FOUNDER2\"; share_bps = 3000:nat16; is_sentinel = false };
  record { member = principal \"$FOUNDER3\"; share_bps = 1000:nat16; is_sentinel = false };
})"
```

### Step 3: Verify & Lock Board
Verify actual voting power and lock the configuration.

```bash
# Verify actual voting power in GHC
dfx canister call governance_canister get_all_board_member_voting_powers

# Lock board (requires Sentinel + 10,000 total regular BPS)
dfx canister call governance_canister lock_board_member_shares
```

**Note**: Before locking, use `unlock_board_member_shares` or `clear_sentinel_member` for emergency fixes.

---

## 2. Testing & Debugging (Admin Only)

These commands allow skipping voting delays during development.

### Skip Voting Periods
```bash
# Force Approve/Reject (Status override)
dfx canister call governance_canister admin_set_proposal_status '(0 : nat64, variant { Approved })'

# Force Expiration (Set end time to past)
dfx canister call governance_canister admin_expire_proposal '(0 : nat64)'
```

---

## 3. Staking Hub Administration

### Manually Set User Shard
In rare cases (e.g., shard migration or error recovery), an admin may need to manually map a user to a specific shard.

```bash
dfx canister call staking_hub admin_set_user_shard '(principal "USER_ID", principal "SHARD_ID")'
```

---

## 4. Founder Vesting Administration

### Register a Founder
After deploying the `founder_vesting` canister, you must manually register each founder and their token allocation.

**Rules:**
- Must be executed by the canister controller.
- Allocation is in `e8s` (1 token = 100,000,000 e8s).
- Vesting starts relative to the genesis timestamp (set at canister initialization).

**Command (Bash):**
```bash
# Founder 1: 350M tokens
F1_PRINCIPAL="n7kyi-a2ccw-erzef-tzywo-kuqyh-4x6on-6ltec-flkee-5zcdi-62drw-gae"
F1_ALLOCATION=35000000000000000

# Founder 2: 150M tokens
F2_PRINCIPAL="vlg44-yhcay-gabeb-4qse6-k4tvu-ujdun-jyzjc-aq5nm-45hye-2qsr5-3ae"
F2_ALLOCATION=15000000000000000

# Register
dfx canister call founder_vesting admin_register_founder "(principal \"$F1_PRINCIPAL\", $F1_ALLOCATION)"
dfx canister call founder_vesting admin_register_founder "(principal \"$F2_PRINCIPAL\", $F2_ALLOCATION)"
```

---

## 5. Emergency & Upgrades

### Canister Upgrades
Controllers can upgrade canister code using `dfx`.

```bash
dfx canister install governance_canister --mode upgrade
```

### Re-Linking Canisters (If IDs Change)
If you redeploy a canister and its ID changes, update the internal references:

**Governance Canister:**
```bash
# Update Treasury Link
dfx canister call governance_canister set_treasury_canister_id '(principal "NEW_TREASURY_ID")'

# Update Learning Engine Link
dfx canister call governance_canister set_learning_engine_id '(principal "NEW_LE_ID")'
```

**Treasury Canister:**
```bash
# Update Governance Link
dfx canister call treasury_canister set_governance_canister_id '(principal "NEW_GOV_ID")'
```
