# 🧪 Complete System Testing Guide

> **Last Updated:** January 17, 2026

This guide explains how to test the GHC staking system and treasury functionality (including MMCR scheduling).

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
dfx deploy treasury_canister
dfx deploy governance_canister
dfx deploy founder_vesting
```

### Step 3: Verify Deployment

```bash
# Check all canisters are running
dfx canister status staking_hub
dfx canister status user_profile
dfx canister status treasury_canister
dfx canister status governance_canister
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
# Option A: Load the full test curriculum (recommended)
./scripts/content/load_test_curriculum.sh

# Option B: Add a single test content node manually
dfx canister call learning_engine add_content_node '(record {
  id = "test_unit";
  parent_id = null;
  order = 1 : nat32;
  display_type = "UNIT";
  title = "Test Unit";
  description = opt "Test content for verification";
  content = opt "This is test content.";
  paraphrase = opt "Test paraphrase.";
  media = null;
  quiz = opt record { questions = vec { 
    record { question = "What is 1+1?"; options = vec {"2"; "3"; "4"}; answer = 0 : nat8 } 
  }};
  created_at = 0 : nat64;
  updated_at = 0 : nat64;
  version = 1 : nat64;
})'

# Submit quiz (correct answer = 0)
dfx canister call user_profile submit_quiz '("test_unit", vec { 0 : nat8 })'

# Check balance (should be 10_000_000_000 = 100 GHC based on global quiz config)
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"

# Verify content was loaded
dfx canister call learning_engine get_content_stats
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
dfx canister call treasury_canister get_treasury_state

# Check MMCR status
dfx canister call treasury_canister get_mmcr_status

# Check spendable balance
dfx canister call treasury_canister get_spendable_balance
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
| `dfx canister call treasury_canister get_treasury_state` | View treasury balance & allowance |
| `dfx canister call treasury_canister get_mmcr_status` | View MMCR progress |
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

### TreasuryState (treasury_canister)

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
dfx canister call treasury_canister get_spendable_balance
```

---

## �️ MMCR Scheduling Tests (Treasury Canister)

The MMCR (Monthly Minimum Capital Release) executes on the **1st of each month at 12:00 AM Eastern Time**. Since waiting for the 1st of each month is impractical, the treasury canister includes dedicated testing functions.

### Testing Functions Overview

| Function | Type | Purpose |
|----------|------|---------|
| `force_execute_mmcr` | Update (Controller) | Execute MMCR bypassing calendar check |
| `simulate_mmcr_at_time` | Query | Check if MMCR would execute at any timestamp |
| `test_date_parsing` | Query | Verify date/time conversion logic |
| `test_dst_boundaries` | Query | Get DST start/end dates for any year |
| `test_mmcr_window` | Query | Check if a specific ET date/time is in MMCR window |
| `get_test_timestamps_for_year` | Query | Get UTC timestamps for all 12 months' 1st days |
| `reset_mmcr_for_testing` | Update (Controller) | Reset MMCR count for re-testing |

### Test 1: Verify Current Time Detection

```bash
# Check what the canister sees as current Eastern Time
dfx canister call treasury_canister get_current_eastern_time

# Expected format: (year, month, day, hour, minute, second, is_dst)
# Example: (2_026 : nat16, 1 : nat8, 17 : nat8, 21 : nat8, 11 : nat8, 23 : nat8, false : bool)
# Meaning: January 17, 2026, 9:11:23 PM ET, DST=false (currently EST)
```

### Test 2: Verify DST Boundaries

```bash
# Get DST start/end dates for 2026
dfx canister call treasury_canister test_dst_boundaries '(2026 : nat16)'

# Expected: (3 : nat8, 8 : nat8, 11 : nat8, 1 : nat8)
# Meaning: DST starts March 8, ends November 1

# Test other years
dfx canister call treasury_canister test_dst_boundaries '(2027 : nat16)'
dfx canister call treasury_canister test_dst_boundaries '(2030 : nat16)'
```

### Test 3: Test MMCR Window Detection

```bash
# Test Feb 1, 2026 at 00:30 ET - SHOULD be in window
dfx canister call treasury_canister test_mmcr_window '(2026 : nat16, 2 : nat8, 1 : nat8, 0 : nat8, 30 : nat8)'
# Expected: (true, <timestamp>, "...In MMCR window: true")

# Test Feb 1, 2026 at 01:00 ET - SHOULD NOT be in window (hour > 0)
dfx canister call treasury_canister test_mmcr_window '(2026 : nat16, 2 : nat8, 1 : nat8, 1 : nat8, 0 : nat8)'
# Expected: (false, <timestamp>, "...In MMCR window: false")

# Test Feb 2, 2026 at 00:30 ET - SHOULD NOT be in window (day != 1)
dfx canister call treasury_canister test_mmcr_window '(2026 : nat16, 2 : nat8, 2 : nat8, 0 : nat8, 30 : nat8)'
# Expected: (false, <timestamp>, "...In MMCR window: false")

# Test during DST (July 1, 2026 at 00:30 ET)
dfx canister call treasury_canister test_mmcr_window '(2026 : nat16, 7 : nat8, 1 : nat8, 0 : nat8, 30 : nat8)'
# Expected: (true, <timestamp>, "...In MMCR window: true")
```

### Test 4: Get Test Timestamps for All Months

```bash
# Get UTC timestamps for midnight ET on 1st of each month in 2026
dfx canister call treasury_canister get_test_timestamps_for_year '(2026 : nat16)'

# Returns 12 records: (month, utc_timestamp, is_dst_at_that_time)
# Use these timestamps with simulate_mmcr_at_time to test each month
```

### Test 5: Simulate MMCR at Specific Times

```bash
# First, get timestamps for the year
dfx canister call treasury_canister get_test_timestamps_for_year '(2026 : nat16)'

# Then simulate MMCR execution at each timestamp
# Example: simulate for February 1, 2026 (use timestamp from previous call)
dfx canister call treasury_canister simulate_mmcr_at_time '(1738393200000000000 : nat64)'
# Expected: (true, "MMCR would execute. Time ET: 2026/2/1 00:00", 2, 2026)
```

### Test 6: Test Date Parsing Logic

```bash
# Test a known timestamp
# January 1, 2026 05:00:00 UTC = January 1, 2026 00:00:00 EST
dfx canister call treasury_canister test_date_parsing '(1735711200000000000 : nat64)'

# Returns: (year, month, day, hour, minute, second, is_dst, eastern_hour)
# Expected: similar to (2026, 1, 1, 5, 0, 0, false, 0)
# UTC hour is 5, Eastern hour is 0 (midnight), not DST
```

### Test 7: Force Execute MMCR (Controller Only)

```bash
# Force execute MMCR bypassing the calendar check
# NOTE: Still enforces 25-day minimum interval and 240 max releases
dfx canister call treasury_canister force_execute_mmcr

# Check the result
dfx canister call treasury_canister get_treasury_state
dfx canister call treasury_canister get_mmcr_status
```

### Test 8: Reset and Retest (Controller Only)

```bash
# Reset MMCR state to test multiple times
dfx canister call treasury_canister reset_mmcr_for_testing

# Verify reset
dfx canister call treasury_canister get_mmcr_status
# Expected: releases_completed = 0
```

### MMCR Test Scenarios

| Test Case | Commands | Expected Result |
|-----------|----------|-----------------|
| Window on 1st at 00:xx ET | `test_mmcr_window '(2026, 2, 1, 0, 30)'` | `(true, ...)` |
| Outside window (day != 1) | `test_mmcr_window '(2026, 2, 2, 0, 30)'` | `(false, ...)` |
| Outside window (hour != 0) | `test_mmcr_window '(2026, 2, 1, 1, 0)'` | `(false, ...)` |
| DST transition (March) | `test_dst_boundaries '(2026)'` | Start: March 8 |
| DST transition (November) | `test_dst_boundaries '(2026)'` | End: November 1 |
| Force execute | `force_execute_mmcr` | Allowance +15.2M |
| Reset for retest | `reset_mmcr_for_testing` | Count reset to 0 |

### Important Notes

1. **Query functions** (`simulate_*`, `test_*`, `get_*`) don't modify state - safe to call anytime
2. **Controller-only functions** (`force_execute_mmcr`, `reset_mmcr_for_testing`) require controller identity
3. **25-day minimum interval** is enforced in `force_execute_mmcr` to prevent accidents
4. **Reset function** only resets MMCR tracking, not allowance/balance/total_transferred

---

## �📁 Test Files Location

| File | Description |
|------|-------------|
| `scripts/tests/comprehensive_system_test.sh` | Full end-to-end test |
| `scripts/tests/test_quiz_flow.sh` | Quiz flow tests |
| `scripts/deploy.sh` | Deployment script |

---

## ✅ Test Checklist

### Core Functionality
- [ ] Canisters deploy successfully
- [ ] User registration works
- [ ] Quiz submission rewards tokens
- [ ] Tokens are staked (balance increases)
- [ ] Unstaking returns 100% (no penalty)
- [ ] GlobalStats shows correct totals

### Treasury & MMCR
- [ ] Treasury state is initialized correctly (4.25B balance, 0.6B allowance)
- [ ] MMCR status shows correct values (240 releases remaining)
- [ ] `get_current_eastern_time` returns correct current time
- [ ] DST boundaries are calculated correctly for current year
- [ ] MMCR window detection works (1st of month, 00:xx ET = true)
- [ ] MMCR window detection rejects invalid times (other days/hours = false)
- [ ] `force_execute_mmcr` increases allowance by 15.2M
- [ ] `reset_mmcr_for_testing` resets MMCR count to 0

### Founder Vesting
- [ ] Founder vesting schedules are set up
- [ ] Genesis timestamp is recorded

