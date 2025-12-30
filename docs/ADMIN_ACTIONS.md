# Admin Actions Guide

This guide details administrative actions available to controllers and founders for managing the GreenHero ecosystem.

## 👥 Founder Management

These commands must be run by a controller (admin) of the `staking_hub` canister.

### Add a Founder
Founders have special voting rights (VUC) in the governance system.

```bash
# 1. Ask the founder to log in and provide their Principal ID
# 2. Run the add_founder command
dfx canister call staking_hub add_founder '(principal "FOUNDER_PRINCIPAL_ID")'
```

### Remove a Founder
```bash
dfx canister call staking_hub remove_founder '(principal "FOUNDER_PRINCIPAL_ID")'
```

### Check Registered Founders
```bash
dfx canister call staking_hub get_founders
```

### Check Founder Vesting Status
```bash
dfx canister call founder_vesting get_all_vesting_schedules
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
dfx canister call operational_governance finalize_proposal '(PROPOSAL_ID)'
```

If the threshold is met, this will change the status to `Executed` and transfer the funds immediately.

### Trigger MMCR (Manual)
The Monthly Market Coin Release (MMCR) releases funds to the treasury allowance every 30 days.
If the time has passed but the timer hasn't triggered it yet, you can force it:

```bash
dfx canister call operational_governance execute_mmcr
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
