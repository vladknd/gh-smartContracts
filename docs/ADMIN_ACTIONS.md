# Admin Actions Guide

This guide details administrative actions available to controllers for managing the GreenHero ecosystem.

---

## 👥 Board Member Management

Board members exercise VUC (Volume of Unmined Coins) voting power for governance decisions. Each board member has a percentage share of the total VUC.

> **Note**: Board members are separate from founders. Founders receive token vesting through `founder_vesting`. Board members exercise voting power through `staking_hub`.

### Set Board Member Shares

Sets all board members atomically. Percentages must sum to exactly 100%.

```bash
# Set board members with their voting power percentages
dfx canister call staking_hub set_board_member_shares '(vec { 
  record { member = principal "BOARD_MEMBER_1_PRINCIPAL"; percentage = 60 : nat8 };
  record { member = principal "BOARD_MEMBER_2_PRINCIPAL"; percentage = 30 : nat8 };
  record { member = principal "BOARD_MEMBER_3_PRINCIPAL"; percentage = 10 : nat8 };
})'
```

### Query Board Members

```bash
# Get all board members with their percentages
dfx canister call staking_hub get_board_member_shares

# Check if someone is a board member
dfx canister call staking_hub is_board_member '(principal "PRINCIPAL_ID")'

# Get specific member's percentage
dfx canister call staking_hub get_board_member_share '(principal "PRINCIPAL_ID")'

# Get count of board members
dfx canister call staking_hub get_board_member_count
```

### Lock Board Member Shares

⚠️ **WARNING**: This is **IRREVERSIBLE** through admin functions!

Once locked, shares can only be changed via governance proposals (future feature).

```bash
# Check if already locked
dfx canister call staking_hub are_board_shares_locked

# Lock permanently (cannot be undone via admin!)
dfx canister call staking_hub lock_board_member_shares
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

# Get specific user's voting power
dfx canister call staking_hub fetch_voting_power '(principal "PRINCIPAL_ID")'

# Get total voting power in system
dfx canister call staking_hub get_total_voting_power
```

---

## 📋 Quick Reference

| Action | Command |
|--------|---------|
| Set board members | `set_board_member_shares` |
| Query board members | `get_board_member_shares` |
| Lock shares (irreversible) | `lock_board_member_shares` |
| Check if locked | `are_board_shares_locked` |
| Check voting power | `fetch_voting_power` |
| Founder vesting status | `get_all_vesting_schedules` |
| Finalize proposal early | `finalize_proposal` |
| Manual MMCR | `execute_mmcr` |
| Treasury status | `get_treasury_state` |
