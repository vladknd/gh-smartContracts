# Admin Actions Guide

This guide details administrative actions available to controllers for managing the GreenHero ecosystem.

---

## 👥 Board Member Management

Board members exercise VUC (Volume of Unmined Coins) voting power for governance decisions. Each board member has a percentage share of the total VUC.

> **Note**: Board members are separate from founders. Founders receive token vesting through `founder_vesting`. Board members exercise voting power and are managed through `operational_governance`.

### Set Board Member Shares

Sets all board members atomically. Percentages must sum to exactly 100%.

```bash
# Set board members with their voting power percentages
dfx canister call operational_governance set_board_member_shares '(vec { 
  record { member = principal "brcis-myp3t-sgc2i-7fzce-onoy6-4cknk-o7zrq-rp2yj-r3adh-wwjm5-2ae"; percentage = 60 : nat8 };
  record { member = principal "cjd7b-pozyi-lvcpt-i2dnz-7ubh5-v5xgm-imvzz-46ecb-2ytr4-ypfor-yqe"; percentage = 30 : nat8 };
  record { member = principal "bqnyh-6ivts-3zmpt-ykoof-ggqza-2jgjp-wri4l-nxhn6-iv4ni-owd53-iae"; percentage = 10 : nat8 };
})'
```

### Query Board Members

```bash
# Get all board members with their percentages
dfx canister call operational_governance get_board_member_shares

# Check if someone is a board member
dfx canister call operational_governance is_board_member '(principal "PRINCIPAL_ID")'

# Get specific member's percentage
dfx canister call operational_governance get_board_member_share '(principal "PRINCIPAL_ID")'

# Get count of board members
dfx canister call operational_governance get_board_member_count
```

### Lock Board Member Shares

⚠️ **WARNING**: After locking, use governance proposals to add new board members!

Once locked, shares can only be changed via `AddBoardMember` governance proposals.

```bash
# Check if already locked
dfx canister call operational_governance are_board_shares_locked

# Lock shares (use governance proposals after this)
dfx canister call operational_governance lock_board_member_shares
```

### Add Board Member via Governance (After Lock)

```bash
# Create a proposal to add a new board member
dfx canister call operational_governance create_board_member_proposal '(record {
  title = "Add New Board Member";
  description = "Proposing to add Alice with 15% share";
  new_member = principal "NEW_MEMBER_PRINCIPAL";
  percentage = 15 : nat8;
  external_link = null
})'
```

---

## 💰 Founder Vesting

Founders receive token allocations through a separate vesting system.

### Check Founder Vesting Status

```bash
# View all founders' vesting schedules
dfx canister call founder_vesting get_all_vesting_schedules

# Check specific founder's status
dfx canister call founder_vesting get_vesting_status '(principal "FOUNDER_PRINCIPAL_ID")'
```

---

## 🏛️ Operational Governance

### Execute a Proposal Early

Proposals normally have a 14-day voting period. However, if a proposal reaches the **Approval Threshold (15,000 YES votes)**, it can be executed immediately.

**Steps:**
1. **Vote**: Ensure enough voting power has voted YES to reach 15,000 tokens.
2. **Finalize**: Normally, a background timer checks this every hour. To execute *immediately*, call:

```bash
# Replace PROPOSAL_ID with the actual ID (e.g., 0, 1, 2)
dfx canister call operational_governance finalize_proposal '(PROPOSAL_ID : nat64)'
```

If the threshold is met, this will change the status to `Executed` and transfer the funds immediately.

### Trigger MMCR (Manual)

The Monthly Market Coin Release (MMCR) releases funds to the treasury allowance every 30 days.
If the time has passed but the timer hasn't triggered it yet, you can force it:

```bash
dfx canister call operational_governance execute_mmcr
```

### Check Treasury Status

```bash
# View treasury balance and allowance
dfx canister call operational_governance get_treasury_state

# View MMCR release status
dfx canister call operational_governance get_mmcr_status
```

---

## ⚙️ System Management

### Manually Add a Shard (Minter)

If you deploy a custom `user_profile` canister and want it to be recognized as a valid shard (able to mint tokens):

```bash
# 1. Deploy the new canister
# 2. Register it in the hub
dfx canister call staking_hub add_allowed_minter '(principal "NEW_CANISTER_ID")'
```

### Check Global Stats

View the total staked, unstaked, and allocated tokens across the entire ecosystem.

```bash
dfx canister call staking_hub get_global_stats
```

### Check Tokenomics

```bash
# Returns: (max_supply, total_allocated, vuc, total_voting_power)
dfx canister call staking_hub get_tokenomics
```

### Check Voting Power

```bash
# Get VUC (total board member voting power pool)
dfx canister call staking_hub get_vuc

# Get user's staked balance (regular users)
dfx canister call staking_hub fetch_user_voting_power '(principal "PRINCIPAL_ID")'

# Get total voting power in system
dfx canister call staking_hub get_total_voting_power
```

---

## 📋 Quick Reference

| Action | Canister | Command |
|--------|----------|---------|
| Set board members | `operational_governance` | `set_board_member_shares` |
| Query board members | `operational_governance` | `get_board_member_shares` |
| Lock shares | `operational_governance` | `lock_board_member_shares` |
| Check if locked | `operational_governance` | `are_board_shares_locked` |
| Add board member (after lock) | `operational_governance` | `create_board_member_proposal` |
| Get VUC | `staking_hub` | `get_vuc` |
| Get user voting power | `staking_hub` | `fetch_user_voting_power` |
| Founder vesting status | `founder_vesting` | `get_all_vesting_schedules` |
| Finalize proposal early | `operational_governance` | `finalize_proposal` |
| Manual MMCR | `operational_governance` | `execute_mmcr` |
| Treasury status | `operational_governance` | `get_treasury_state` |
