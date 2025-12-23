# GHC Interest Flow & Distribution Mechanics

> **Last Updated:** December 14, 2025

> **Note:** For the new tier-based interest system that rewards long-term stakers, see [ADVANCED_INTEREST_FLOW.md](./ADVANCED_INTEREST_FLOW.md). This document describes the base mechanics.

This document explains how interest is generated, distributed, and claimed in the GHC staking system.

---

## 1. Overview

The GHC system uses a **Global Reward Index Model** to distribute yield (from penalties) to stakers efficiently. This allows for scalable interest distribution without iterating through millions of user accounts.

### Key Principles

- **Interest Source**: All interest comes from **unstaking penalties** (10% of unstaked amount)
- **Distribution Method**: Global Index model (O(1) scalability)
- **Claiming**: Manual (users must explicitly claim rewards)

---

## 2. Complete Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    INTEREST FLOW DIAGRAM                        │
└─────────────────────────────────────────────────────────────────┘

    ╔═════════════════╗
    ║   User A        ║
    ║   Unstakes      ║
    ║   100 GHC       ║
    ╚════════╤════════╝
             │
             ▼
    ┌────────────────────┐
    │  10% Penalty       │ ◄──── 10 GHC
    │  (90 GHC returned) │
    └────────┬───────────┘
             │
             ▼
   ╔═════════════════════════╗
   ║    STAKING HUB          ║
   ║  ┌─────────────────┐    ║
   ║  │ interest_pool   │ +10║
   ║  │ = 10 GHC        │    ║
   ║  └─────────────────┘    ║
   ╚════════════╤════════════╝
                │
                │ distribute_interest() called
                ▼
   ╔═════════════════════════════════════╗
   ║  GlobalIndex += Pool / TotalStaked  ║
   ║  (Example: 10 / 100 = 0.1)          ║
   ║  InterestPool = 0                   ║
   ╚════════════╤════════════════════════╝
                │
                │ Syncs every 5 seconds
                ▼
   ╔═══════════════════════════════════════════╗
   ║     USER PROFILE SHARD                    ║
   ║  ┌────────────────────────────────────┐   ║
   ║  │ User B (staked 50 GHC):           │   ║
   ║  │   IndexDiff = 0.1 - 0 = 0.1       │   ║
   ║  │   Interest = 50 * 0.1 = 5 GHC     │   ║
   ║  │   unclaimed_interest += 5         │   ║
   ║  └────────────────────────────────────┘   ║
   ║  ┌────────────────────────────────────┐   ║
   ║  │ User C (staked 50 GHC):           │   ║
   ║  │   Interest = 50 * 0.1 = 5 GHC     │   ║
   ║  └────────────────────────────────────┘   ║
   ╚═══════════════════════════════════════════╝
                │
                │ claim_rewards()
                ▼
   ┌───────────────────────────────────────┐
   │  User B's staked_balance += 5 GHC    │
   │  User B's unclaimed_interest = 0     │
   └───────────────────────────────────────┘
```

---

## 3. Step-by-Step Explanation

### Step 1: Penalty Accumulation

When any user unstakes tokens, a **10% penalty** is applied:

```
User requests unstake of 100 GHC
├── User receives: 90 GHC (transferred to wallet)
└── Penalty: 10 GHC (added to interest_pool)
```

**Code Location:** `staking_hub/src/lib.rs` → `process_unstake()`
```rust
let penalty = amount / 10; // 10%
let return_amount = amount - penalty;
stats.interest_pool += penalty;
```

### Step 2: Interest Distribution (Admin Trigger)

An admin (or scheduled job) calls `distribute_interest()` to convert the accumulated penalty pool into yield for all stakers:

**Formula:**
```
IndexIncrease = (interest_pool × 1e18) / total_staked
GlobalIndex += IndexIncrease
interest_pool = 0
```

**Example:**
- `interest_pool = 10 GHC`
- `total_staked = 100 GHC`
- `IndexIncrease = 10/100 = 0.1` (scaled by 1e18)

**Code Location:** `staking_hub/src/lib.rs` → `distribute_interest()`

### Step 3: Index Synchronization

The User Profile shard automatically syncs with the Staking Hub every **5 seconds** and retrieves the latest `cumulative_reward_index`.

**Code Location:** `user_profile/src/lib.rs` → `sync_with_hub_internal()`

### Step 4: Lazy Interest Calculation

User interest is calculated **only when needed** (on profile view, quiz submit, unstake, or claim). This is called "lazy evaluation":

**Formula:**
```
IndexDiff = GlobalIndex - UserLastRewardIndex
UserInterest = (staked_balance × IndexDiff) / 1e18
unclaimed_interest += UserInterest
last_reward_index = GlobalIndex
```

**Example (User B with 50 GHC staked):**
- `IndexDiff = 0.1 - 0 = 0.1`
- `Interest = 50 × 0.1 = 5 GHC`

**Code Location:** `user_profile/src/lib.rs` → `compound_interest()`

### Step 5: Claiming Rewards

Users must explicitly call `claim_rewards()` to move interest to their staked balance:

```
unclaimed_interest → staked_balance
unclaimed_interest = 0
```

**Code Location:** `user_profile/src/lib.rs` → `claim_rewards()`

---

## 4. User Profile Data Structure

Each user profile tracks three key fields for interest:

| Field | Type | Description |
|-------|------|-------------|
| `staked_balance` | `u64` | Principal amount (tokens earned from quizzes + claimed interest) |
| `unclaimed_interest` | `u64` | Pending rewards waiting to be claimed |
| `last_reward_index` | `u128` | The GlobalIndex value when interest was last calculated |

---

## 5. Canister Methods Reference

### Staking Hub (`staking_hub`)

| Method | Type | Description |
|--------|------|-------------|
| `get_global_stats()` | Query | Returns `interest_pool`, `total_staked`, `cumulative_reward_index` |
| `distribute_interest()` | Update | Converts pool to index increase. **Admin only.** |

### User Profile (`user_profile`)

| Method | Type | Description |
|--------|------|-------------|
| `get_profile(principal)` | Query | Returns user data including calculated `unclaimed_interest` |
| `claim_rewards()` | Update | Moves `unclaimed_interest` → `staked_balance` |

---

## 6. Frontend Integration

### Display User Balances

```javascript
const profile = await userProfileActor.get_profile(userPrincipal);

if (profile[0]) {
    const stakedGHC = Number(profile[0].staked_balance) / 1e8;
    const pendingGHC = Number(profile[0].unclaimed_interest) / 1e8;
    
    console.log(`Staked: ${stakedGHC} GHC`);
    console.log(`Pending Interest: ${pendingGHC} GHC`);
}
```

### Claim Rewards

```javascript
async function claimRewards() {
    const result = await userProfileActor.claim_rewards();
    
    if ('Ok' in result) {
        const claimed = Number(result.Ok) / 1e8;
        alert(`Claimed ${claimed} GHC!`);
    } else {
        alert(result.Err); // "No rewards to claim"
    }
}
```

### Display Global Interest Pool

```javascript
const stats = await stakingHubActor.get_global_stats();

const poolGHC = Number(stats.interest_pool) / 1e8;
const stakedGHC = Number(stats.total_staked) / 1e8;

console.log(`Interest Pool: ${poolGHC} GHC`);
console.log(`Total Staked: ${stakedGHC} GHC`);

// Estimated yield if distributed now
if (stakedGHC > 0) {
    const yieldPercent = (poolGHC / stakedGHC) * 100;
    console.log(`Pending Yield: ${yieldPercent.toFixed(4)}%`);
}
```

---

## 7. Important Notes

### Interest is Passive, Claiming is Active

- Interest **accrues automatically** based on staked balance and global index
- Users **must click "Claim"** to add interest to their staked balance
- The `get_profile()` query shows the **calculated** pending interest (real-time)

### Scalability

- **O(1) Distribution**: No iteration through users needed
- **Lazy Evaluation**: Interest only calculated when user interacts
- **Batched Sync**: Shards batch-report stats to Hub every 5 seconds

### No "Free Money"

Interest is **zero-sum redistribution** from unstakers to stakers:
- User A leaves early → pays 10% penalty
- Users B & C who stay → earn that 10% proportionally

---

## 8. Example Scenario

**Setup:**
- User B has 50 GHC staked
- User C has 50 GHC staked
- Total staked: 100 GHC

**User A unstakes 100 GHC:**
1. User A receives 90 GHC
2. 10 GHC added to interest_pool

**Admin calls `distribute_interest()`:**
1. `IndexIncrease = 10 / 100 = 0.1`
2. `GlobalIndex += 0.1`
3. `interest_pool = 0`

**User B views profile:**
1. `IndexDiff = 0.1 - 0 = 0.1`
2. `Interest = 50 × 0.1 = 5 GHC`
3. `unclaimed_interest = 5 GHC`

**User B claims rewards:**
1. `staked_balance += 5` → now 55 GHC
2. `unclaimed_interest = 0`
3. `last_reward_index = 0.1`

**Result:**
- User B: 55 GHC staked (was 50)
- User C: 55 GHC staked (was 50) — if they also claim
- Total redistributed: 10 GHC from User A's penalty

---

## 9. Related Documentation

- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) - Detailed staking mechanics
- [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) - Complete API reference
