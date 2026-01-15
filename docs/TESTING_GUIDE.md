# 🧪 Complete System Testing Guide

> **Last Updated:** January 2026

This guide explains how to test the GHC staking system (simplified model without interest/penalties).

---

## 📋 Prerequisites

1. **DFX installed** (`dfx --version`)
2. **Canisters compiled** (run `cargo check` first)
3. **Local replica** or testnet environment

---

## 🚀 Quick Start: Full System Test

### Option 1: Comprehensive System Test (Recommended)

This runs a complete end-to-end test including deployment:

```bash
cd /mnt/c/LIB/CODE/gh-smartContracts
./scripts/tests/comprehensive_system_test.sh
```

**What it tests:**
- ✅ System deployment
- ✅ User registration
- ✅ Quiz submission & token rewards
- ✅ Staking mechanics
- ✅ Unstaking (100% return, no penalty)
- ✅ Global stats tracking

**Output:** Creates `test_report.md` with detailed results.

---

## 🔨 Manual Testing Steps

### Step 1: Start Local Replica

```bash
# Clean start (recommended for fresh testing)
dfx stop
rm -rf .dfx
dfx start --background --clean
```

### Step 2: Deploy All Canisters

```bash
./scripts/deploy.sh
```

Or manually:
```bash
dfx deploy internet_identity
dfx deploy ghc_ledger
dfx deploy learning_engine
dfx deploy staking_hub
dfx deploy user_profile
dfx deploy operational_governance
dfx deploy founder_vesting
```

### Step 3: Verify Deployment

```bash
# Check all canisters are running
dfx canister status staking_hub
dfx canister status user_profile
dfx canister status operational_governance
dfx canister status founder_vesting

# Get canister IDs
dfx canister id staking_hub
dfx canister id user_profile
```

### Step 4: Test User Registration

```bash
# Register a test user
dfx canister call user_profile register_user '(record { 
  email = "test@example.com"; 
  name = "Test User"; 
  education = "Test"; 
  gender = "Test" 
})'

# Verify registration
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"
```

### Step 5: Test Quiz & Staking

```bash
# Add a test learning unit first
dfx canister call learning_engine add_learning_unit '(record {
  unit_id = "test_unit";
  unit_title = "Test Unit";
  chapter_id = "chap_1";
  chapter_title = "Test Chapter";
  head_unit_id = "head_1";
  head_unit_title = "Test Head";
  content = "Content";
  paraphrase = "Para";
  quiz = vec { record { question = "What is 1+1?"; options = vec {"2", "3"}; answer = 0 } }
})'

# Submit quiz (correct answer = 0)
dfx canister call user_profile submit_quiz '("test_unit", vec { 0 })'

# Check balance (should be 100_000_000 = 1 token)
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"
```

### Step 6: Test Global Stats

```bash
# Check global stats (shows total_staked, total_unstaked, total_allocated)
dfx canister call staking_hub get_global_stats

# Force sync to ensure data is up to date
dfx canister call user_profile debug_force_sync
```

### Step 7: Test Unstaking (No Penalty)

```bash
# Unstake tokens (returns 100% - no penalty)
dfx canister call user_profile unstake '(50_000_000)'

# Check wallet balance increased
dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$(dfx identity get-principal)\"; subaccount = null })"

# Verify global stats updated
dfx canister call staking_hub get_global_stats
```

### Step 8: Test Treasury Functions

```bash
# Check treasury state
dfx canister call operational_governance get_treasury_state

# Check MMCR status
dfx canister call operational_governance get_mmcr_status

# Check spendable balance
dfx canister call operational_governance get_spendable_balance
```

### Step 9: Test Founder Vesting

```bash
# Check vesting schedules
dfx canister call founder_vesting get_all_vesting_schedules

# Check genesis timestamp
dfx canister call founder_vesting get_genesis_timestamp

# Check total unclaimed
dfx canister call founder_vesting get_total_unclaimed
```

---

## 📊 Key Commands Reference

| Command | Description |
|---------|-------------|
| `dfx canister call staking_hub get_global_stats` | View hub statistics (staked, unstaked, allocated) |
| `dfx canister call user_profile get_profile "(principal \"...\")"` | View user profile |
| `dfx canister call user_profile debug_force_sync` | Force shard to sync with hub |
| `dfx canister call operational_governance get_treasury_state` | View treasury balance & allowance |
| `dfx canister call operational_governance get_mmcr_status` | View MMCR progress |
| `dfx canister call founder_vesting get_all_vesting_schedules` | View founder vesting status |

---

## 🔍 What to Verify

### GlobalStats (staking_hub)

```
total_staked = <sum of all staked balances>
total_unstaked = <sum of all unstaked tokens>
total_allocated = <total tokens allocated for minting>
```

After quiz completion:
- `total_staked` should increase
- `total_allocated` should increase

After unstaking:
- `total_staked` decreases
- `total_unstaked` increases

### UserProfile (user_profile)

```
staked_balance = <tokens earned from quizzes>
transaction_count = <number of transactions>
```

### TreasuryState (operational_governance)

```
balance = 4.25B (initial)
allowance = 0.6B (initial, increases via MMCR)
mmcr_count = 0-240 (MMCR releases executed)
```

---

## 🐛 Troubleshooting

### "User not registered"
```bash
dfx canister call user_profile register_user '(record { email = "x"; name = "x"; education = "x"; gender = "x" })'
```

### "Unauthorized: Caller is not a registered shard"
The user_profile canister is not registered as a shard in the hub. Check your deploy script.

### Sync not working
```bash
dfx canister call user_profile debug_force_sync
```

### "Insufficient treasury allowance"
Wait for MMCR to increase allowance or check current allowance:
```bash
dfx canister call operational_governance get_spendable_balance
```

---

## 📁 Test Files Location

| File | Description |
|------|-------------|
| `scripts/tests/comprehensive_system_test.sh` | Full end-to-end test |
| `scripts/tests/test_quiz_flow.sh` | Quiz flow tests |
| `scripts/deploy.sh` | Deployment script |

---

## ✅ Test Checklist

- [ ] Canisters deploy successfully
- [ ] User registration works
- [ ] Quiz submission rewards tokens
- [ ] Tokens are staked (balance increases)
- [ ] Unstaking returns 100% (no penalty)
- [ ] GlobalStats shows correct totals
- [ ] Treasury state is initialized correctly
- [ ] MMCR status shows correct values
- [ ] Founder vesting schedules are set up
