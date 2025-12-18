# 🧪 Complete System Testing Guide

> **Last Updated:** December 14, 2025

This guide explains how to test the GHC staking system with the new Discrete Tier System.

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
- ✅ Quiz submission & rewards
- ✅ Staking mechanics
- ✅ Unstaking & penalty collection
- ✅ Interest distribution
- ✅ Reward claiming

**Output:** Creates `test_report.md` with detailed results.

---

### Option 2: Tier System Test Only

If the system is already deployed, run just the tier-specific tests:

```bash
cd /mnt/c/LIB/CODE/gh-smartContracts
./tests/tier_system_test.sh
```

**What it tests:**
- ✅ Tier configuration in hub
- ✅ `tier_staked` and `tier_reward_indexes` fields
- ✅ User profile tier fields
- ✅ Sync mechanism
- ✅ Tier thresholds and weights

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
./deploy.sh
```

Or manually:
```bash
dfx deploy internet_identity
dfx deploy ghc_ledger
dfx deploy learning_engine
dfx deploy staking_hub
dfx deploy user_profile
```

### Step 3: Verify Deployment

```bash
# Check all canisters are running
dfx canister status staking_hub
dfx canister status user_profile

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

### Step 6: Test Tier System

```bash
# Check global stats (shows tier_staked and tier_reward_indexes)
dfx canister call staking_hub get_global_stats

# Force sync to ensure data is up to date
dfx canister call user_profile debug_force_sync

# Check user tier info
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"
# Look for: current_tier, tier_start_index, initial_stake_time
```

### Step 7: Test Unstaking & Interest

```bash
# Unstake half (creates 10% penalty for interest pool)
dfx canister call user_profile unstake '(50_000_000)'

# Force sync to report to hub
dfx canister call user_profile debug_force_sync

# Check interest pool has penalty
dfx canister call staking_hub get_global_stats
# Look for: interest_pool = 5_000_000

# Distribute interest
dfx canister call staking_hub distribute_interest

# Force sync to get new indexes
dfx canister call user_profile debug_force_sync

# Check unclaimed_interest
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"
# Should show unclaimed_interest > 0

# Claim rewards
dfx canister call user_profile claim_rewards
```

---

## 📊 Key Commands Reference

| Command | Description |
|---------|-------------|
| `dfx canister call staking_hub get_global_stats` | View all hub statistics including tier data |
| `dfx canister call user_profile get_profile "(principal \"...\")"` | View user profile with tier info |
| `dfx canister call user_profile debug_force_sync` | Force shard to sync with hub |
| `dfx canister call staking_hub distribute_interest` | Distribute interest pool to stakers |
| `dfx canister call user_profile claim_rewards` | Claim unclaimed interest to staked balance |

---

## 🔍 What to Verify

### GlobalStats (staking_hub)

```
tier_staked = vec { <bronze>; <silver>; <gold>; <diamond> }
tier_reward_indexes = vec { <bronze_idx>; <silver_idx>; <gold_idx>; <diamond_idx> }
```

After distribute_interest:
- `interest_pool` should decrease
- `tier_reward_indexes` should increase for tiers with stakers

### UserProfile (user_profile)

```
current_tier = 0                    # 0=Bronze, 1=Silver, 2=Gold, 3=Diamond
tier_start_index = <number>         # Index when entered tier
initial_stake_time = <timestamp>    # When first staked
```

---

## ⏱️ Tier Upgrade Testing

Since tier upgrades are based on time, you can:

1. **Mock time in tests** (requires code changes)
2. **Wait for actual time** (30 days for Silver)
3. **Modify thresholds for testing** (lower them temporarily)

To test with shorter thresholds for development, you could temporarily change in both canisters:

```rust
// For testing only - change back before production!
pub const TIER_THRESHOLDS_NANOS: [u64; 4] = [
    0,                           // Bronze
    60 * 1_000_000_000,          // Silver: 1 minute
    180 * 1_000_000_000,         // Gold: 3 minutes
    300 * 1_000_000_000,         // Diamond: 5 minutes
];
```

---

## 🐛 Troubleshooting

### "User not registered"
```bash
dfx canister call user_profile register_user '(record { email = "x"; name = "x"; education = "x"; gender = "x" })'
```

### "No interest to distribute"
The interest pool is empty. Unstake some tokens first to create penalty.

### "Unauthorized: Caller is not a registered shard"
The user_profile canister is not registered as a shard in the hub. Check your deploy script.

### Sync not working
```bash
dfx canister call user_profile debug_force_sync
```

---

## 📁 Test Files Location

| File | Description |
|------|-------------|
| `scripts/tests/comprehensive_system_test.sh` | Full end-to-end test |
| `tests/tier_system_test.sh` | Tier-specific tests |
| `scripts/tests/test_quiz_flow.sh` | Quiz flow tests |
| `scripts/tests/comprehensive_test.sh` | Alternative comprehensive test |

---

## ✅ Test Checklist

- [ ] Canisters deploy successfully
- [ ] User registration works
- [ ] Quiz submission rewards tokens
- [ ] Tokens are staked (balance increases)
- [ ] Tier fields exist in user profile
- [ ] Unstaking deducts 10% penalty
- [ ] Interest pool receives penalties
- [ ] distribute_interest updates tier indexes
- [ ] Unclaimed interest calculates correctly
- [ ] claim_rewards moves interest to staked balance
- [ ] GlobalStats shows tier_staked and tier_reward_indexes
