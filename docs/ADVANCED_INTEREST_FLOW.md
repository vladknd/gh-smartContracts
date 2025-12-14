# Advanced Interest Flow: Discrete Tier System

> **Last Updated:** December 13, 2025  
> **Status:** Proposed Enhancement  
> **Depends On:** [INTEREST_FLOW.md](./INTEREST_FLOW.md)

This document describes an advanced interest distribution mechanism that rewards long-term stakers with higher yields through a **Discrete Tier System**.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Solution: Discrete Tier System](#3-solution-discrete-tier-system)
4. [How It Works](#4-how-it-works)
5. [Architecture](#5-architecture)
6. [Implementation Details](#6-implementation-details)
7. [Why Separate Pools Per Tier?](#7-why-separate-pools-per-tier)
8. [Comparison with Alternatives](#8-comparison-with-alternatives)
9. [Migration Plan](#9-migration-plan)
10. [Frontend Integration](#10-frontend-integration)
11. [Security Analysis](#11-security-analysis)

---

## 1. Executive Summary

The Discrete Tier System enhances our existing Global Reward Index model by introducing **time-based loyalty tiers**. Users who stake longer receive a higher share of the interest pool through a fair, scalable, and transparent mechanism.

### Key Features

| Feature | Description |
|---------|-------------|
| **4 Loyalty Tiers** | Bronze → Silver → Gold → Diamond |
| **Time-Based Progression** | Tier determined by continuous staking duration |
| **Fair Distribution** | Each tier has dedicated pool share, no token inflation |
| **O(1) Scalability** | Only 4 tier totals to track globally |
| **Automatic Upgrades** | Users progress through tiers automatically |

### Tier Configuration

```
┌─────────────────────────────────────────────────────────────────┐
│                     LOYALTY TIER STRUCTURE                       │
├──────────┬──────────────┬──────────────┬────────────────────────┤
│   TIER   │   DURATION   │  POOL SHARE  │      DESCRIPTION       │
├──────────┼──────────────┼──────────────┼────────────────────────┤
│  Bronze  │   0-30 days  │     20%      │  New stakers           │
│  Silver  │  30-90 days  │     25%      │  Committed stakers     │
│  Gold    │ 90-365 days  │     30%      │  Loyal stakers         │
│  Diamond │   365+ days  │     25%      │  Long-term holders     │
└──────────┴──────────────┴──────────────┴────────────────────────┘
```

---

## 2. Problem Statement

### Current System Limitation

The existing Global Reward Index model distributes interest **proportionally to staked balance only**:

```
user_interest = staked_balance × index_increase
```

**Everyone with the same balance receives the same interest, regardless of loyalty.**

### The Challenge

```
┌─────────────────────────────────────────────────────────────────┐
│                     CURRENT DISTRIBUTION                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Alice (100 GHC, staking 7 days)    ────────▶  10 GHC interest │
│                                                                  │
│   Bob (100 GHC, staking 365 days)    ────────▶  10 GHC interest │
│                                                                  │
│          Same balance = Same reward (unfair to Bob!)             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Why This Matters

1. **No incentive to hold long-term** — Users can jump in/out for quick gains
2. **Increased volatility** — No loyalty benefits encourage speculative behavior
3. **Unfair to early supporters** — Those who took early risk aren't rewarded
4. **Missed gamification** — Tier progression is a powerful engagement tool

---

## 3. Solution: Discrete Tier System

### Core Concept

Instead of one shared pool, we partition the interest pool into **4 tier-specific pools**. Each tier has its own reward index that depends on:

1. The tier's allocated share of the penalty pool
2. The total tokens staked by users in that tier

```
                         PENALTY POOL
                        (from unstaking)
                              │
                              │ 100 GHC
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
           ▼                  ▼                  ▼                  ▼
    ┌────────────┐     ┌────────────┐     ┌────────────┐     ┌────────────┐
    │   BRONZE   │     │   SILVER   │     │    GOLD    │     │  DIAMOND   │
    │   20 GHC   │     │   25 GHC   │     │   30 GHC   │     │   25 GHC   │
    │  (20%)     │     │  (25%)     │     │  (30%)     │     │  (25%)     │
    ├────────────┤     ├────────────┤     ├────────────┤     ├────────────┤
    │ Staked:    │     │ Staked:    │     │ Staked:    │     │ Staked:    │
    │ 500 GHC    │     │ 300 GHC    │     │ 150 GHC    │     │  50 GHC    │
    ├────────────┤     ├────────────┤     ├────────────┤     ├────────────┤
    │ Index +=   │     │ Index +=   │     │ Index +=   │     │ Index +=   │
    │ 20/500     │     │ 25/300     │     │ 30/150     │     │ 25/50      │
    │ = 0.04     │     │ = 0.083    │     │ = 0.20     │     │ = 0.50     │
    ├────────────┤     ├────────────┤     ├────────────┤     ├────────────┤
    │ Effective  │     │ Effective  │     │ Effective  │     │ Effective  │
    │ Rate: 4%   │     │ Rate: 8.3% │     │ Rate: 20%  │     │ Rate: 50%  │
    └────────────┘     └────────────┘     └────────────┘     └────────────┘
```

### Result: Loyalty Pays

```
┌─────────────────────────────────────────────────────────────────┐
│                   TIER-BASED DISTRIBUTION                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Alice (100 GHC, Bronze, 7 days)     ──▶  4 GHC interest       │
│                                                                  │
│   Bob (100 GHC, Diamond, 365 days)    ──▶  50 GHC interest      │
│                                                                  │
│           Same balance, but Bob gets 12.5x more!                 │
│               (Because fewer users share Diamond pool)           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. How It Works

### User Lifecycle Flow

```
                                 USER JOURNEY
═══════════════════════════════════════════════════════════════════

 Day 1         Day 30         Day 90         Day 365
   │             │              │               │
   ▼             ▼              ▼               ▼
┌──────┐     ┌──────┐       ┌──────┐       ┌──────────┐
│BRONZE│────▶│SILVER│──────▶│ GOLD │──────▶│ DIAMOND  │
└──────┘     └──────┘       └──────┘       └──────────┘
   │             │              │               │
   │  Automatic  │   Automatic  │   Automatic   │
   │  Promotion  │   Promotion  │   Promotion   │
   │             │              │               │
   ▼             ▼              ▼               ▼
 20% of       25% of         30% of         25% of
 pool         pool           pool           pool
 shared       shared         shared         shared
 among        among          among          among
 Bronze       Silver         Gold           Diamond
 stakers      stakers        stakers        stakers
```

### Step-by-Step: Interest Distribution

```
═══════════════════════════════════════════════════════════════════
                    DISTRIBUTION FLOW
═══════════════════════════════════════════════════════════════════

Step 1: PENALTY COLLECTION
─────────────────────────────────────────────────────────────────
        User unstakes 100 GHC
              │
              ▼
        ┌─────────────────┐
        │ 10% Penalty     │
        │ = 10 GHC        │
        │ → interest_pool │
        └─────────────────┘


Step 2: ADMIN TRIGGERS DISTRIBUTION
─────────────────────────────────────────────────────────────────
        distribute_interest() called
              │
              ▼
        ┌─────────────────────────────────────────────┐
        │  interest_pool = 100 GHC                    │
        │                                             │
        │  For each tier:                             │
        │    tier_share = pool × tier_weight          │
        │    tier_index += tier_share / tier_staked   │
        └─────────────────────────────────────────────┘
              │
              ├──────▶ Bronze Index += 20/500 = 0.04
              ├──────▶ Silver Index += 25/300 = 0.083
              ├──────▶ Gold Index += 30/150 = 0.20
              └──────▶ Diamond Index += 25/50 = 0.50


Step 3: SHARD SYNC (Every 5 seconds)
─────────────────────────────────────────────────────────────────
        ┌─────────────┐         ┌─────────────┐
        │   Shard 1   │◀───────▶│ STAKING_HUB │
        └─────────────┘         └─────────────┘
              │
              ▼
        Receives: [bronze_idx, silver_idx, gold_idx, diamond_idx]
        Stores locally for user calculations


Step 4: USER INTEREST CALCULATION (Lazy)
─────────────────────────────────────────────────────────────────
        User views profile / claims rewards
              │
              ▼
        ┌─────────────────────────────────────────────┐
        │  1. Check if tier upgrade needed            │
        │  2. If upgrading:                           │
        │     - Calculate pending in old tier         │
        │     - Add to unclaimed_interest             │
        │     - Move to new tier                      │
        │  3. Calculate current tier interest:        │
        │     index_diff = current_idx - start_idx    │
        │     interest = balance × index_diff         │
        └─────────────────────────────────────────────┘


Step 5: USER CLAIMS REWARDS
─────────────────────────────────────────────────────────────────
        claim_rewards() called
              │
              ▼
        unclaimed_interest ──▶ staked_balance
        unclaimed_interest = 0
```

### Tier Upgrade Process

```
═══════════════════════════════════════════════════════════════════
                    TIER UPGRADE FLOW
═══════════════════════════════════════════════════════════════════

User has been staking for 31 days (just passed Silver threshold)

┌─────────────────────────────────────────────────────────────────┐
│  BEFORE UPGRADE                                                  │
│  ├─ current_tier: Bronze (0)                                    │
│  ├─ staked_balance: 100 GHC                                     │
│  ├─ tier_start_index: 0.50 (Bronze index when they joined)      │
│  └─ Bronze current index: 0.90                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  UPGRADE DETECTION                                               │
│  ├─ stake_duration = now - initial_stake_time = 31 days         │
│  ├─ 31 days >= 30 days (Silver threshold)                       │
│  └─ new_tier = Silver (1)                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  CLAIM BRONZE INTEREST                                           │
│  ├─ index_diff = 0.90 - 0.50 = 0.40                             │
│  ├─ interest = 100 × 0.40 = 40 GHC                              │
│  └─ unclaimed_interest += 40 GHC                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  MOVE TO SILVER                                                  │
│  ├─ current_tier = Silver (1)                                   │
│  ├─ tier_start_index = current Silver index (e.g., 1.20)        │
│  └─ Queue tier delta for sync:                                  │
│       Bronze: -100 GHC                                          │
│       Silver: +100 GHC                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  AFTER UPGRADE                                                   │
│  ├─ current_tier: Silver (1)                                    │
│  ├─ staked_balance: 100 GHC (unchanged)                         │
│  ├─ tier_start_index: 1.20 (Silver index at upgrade time)       │
│  └─ unclaimed_interest: 40 GHC (from Bronze period)             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Architecture

### Global State (staking_hub)

```
┌─────────────────────────────────────────────────────────────────┐
│                        STAKING HUB                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   GlobalStats {                                                  │
│       total_staked: u64,                                        │
│       interest_pool: u64,                                       │
│       total_unstaked: u64,                                      │
│       total_allocated: u64,                                     │
│       total_rewards_distributed: u64,                           │
│                                                                  │
│       // NEW: Per-tier tracking                                  │
│       tier_staked: [u64; 4],           // [Bronze, Silver, ...]  │
│       tier_reward_indexes: [u128; 4],  // Scaled by 1e18        │
│   }                                                              │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│   │ tier_staked │  │tier_indexes │  │interest_pool│             │
│   ├─────────────┤  ├─────────────┤  ├─────────────┤             │
│   │[0]: 500 GHC │  │[0]: 1.5e18  │  │   100 GHC   │             │
│   │[1]: 300 GHC │  │[1]: 2.1e18  │  │             │             │
│   │[2]: 150 GHC │  │[2]: 3.8e18  │  │ (pending    │             │
│   │[3]:  50 GHC │  │[3]: 9.2e18  │  │  distribution)            │
│   └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Per-User State (user_profile shard)

```
┌─────────────────────────────────────────────────────────────────┐
│                      USER PROFILE SHARD                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   UserProfile {                                                  │
│       // Existing fields                                         │
│       staked_balance: u64,                                      │
│       unclaimed_interest: u64,                                  │
│       transaction_count: u64,                                   │
│                                                                  │
│       // NEW: Tier tracking                                      │
│       current_tier: u8,          // 0=Bronze, 1=Silver, etc.    │
│       tier_start_index: u128,    // Index when entered tier     │
│       initial_stake_time: u64,   // First stake timestamp       │
│       last_tier_check: u64,      // Last upgrade check          │
│   }                                                              │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Thread-Local Storage:                                          │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ TIER_REWARD_INDEXES: [u128; 4]                          │   │
│   │   (synced from hub every 5 seconds)                     │   │
│   ├─────────────────────────────────────────────────────────┤   │
│   │ PENDING_TIER_DELTAS: [i64; 4]                           │   │
│   │   (batched for sync: [+5, -3, +2, 0])                   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Sync Protocol

```
┌─────────────────────────────────────────────────────────────────┐
│                        SYNC PROTOCOL                             │
└─────────────────────────────────────────────────────────────────┘

     USER_PROFILE SHARD                         STAKING_HUB
            │                                        │
            │  sync_shard_v2(                        │
            │    staked_delta: i64,                  │
            │    unstaked_delta: u64,                │
            │    tier_deltas: [i64; 4],  ◄─── NEW    │
            │    requested_allowance: u64            │
            │  )                                     │
            │ ─────────────────────────────────────▶ │
            │                                        │
            │                                        │  Update:
            │                                        │  - tier_staked[i] += delta[i]
            │                                        │  - total_staked += sum(deltas)
            │                                        │
            │  Result<(                              │
            │    granted_allowance: u64,             │
            │    tier_indexes: [u128; 4]  ◄─── NEW   │
            │  ), Error>                             │
            │ ◀───────────────────────────────────── │
            │                                        │
            │  Store tier_indexes locally            │
            │  for interest calculations             │
            │                                        │
```

---

## 7. Why Separate Pools Per Tier?

### The Mathematical Problem

With a **single global pool**, applying multipliers breaks the zero-sum property:

```
═══════════════════════════════════════════════════════════════════
              BROKEN: SINGLE POOL WITH MULTIPLIERS
═══════════════════════════════════════════════════════════════════

Pool = 100 GHC
Total Staked = 200 GHC

User A: 100 GHC × 1.0x multiplier = 100 effective
User B: 100 GHC × 2.0x multiplier = 200 effective

Naive calculation:
  A's share = 100 × (100/200) × 1.0 = 50 GHC
  B's share = 100 × (100/200) × 2.0 = 100 GHC
                                      ─────────
  Total distributed:                  150 GHC  ❌

  WE DISTRIBUTED 50 GHC MORE THAN THE POOL!
```

### The Solution: Isolated Tier Pools

```
═══════════════════════════════════════════════════════════════════
              CORRECT: SEPARATE POOLS PER TIER
═══════════════════════════════════════════════════════════════════

Pool = 100 GHC
├── Bronze Pool: 40 GHC (40%)
└── Diamond Pool: 60 GHC (60%)

Bronze Tier:
  User A: 100 GHC staked
  Other Bronze users: 300 GHC staked
  Total Bronze: 400 GHC
  
  A's share = 40 × (100/400) = 10 GHC ✓

Diamond Tier:
  User B: 100 GHC staked
  Other Diamond users: 100 GHC staked
  Total Diamond: 200 GHC
  
  B's share = 60 × (100/200) = 30 GHC ✓

Total distributed: 10 + 30 + (other users) = 100 GHC ✓

EXACTLY THE POOL AMOUNT — NO INFLATION!
```

### Benefits of Separate Pools

| Benefit | Explanation |
|---------|-------------|
| **Zero-Sum Guarantee** | Total distributed = Total collected (always) |
| **No Inflation** | No new tokens created, just redistribution |
| **Transparent Rates** | Each tier's effective APY is calculable |
| **Isolated Risk** | One tier's behavior doesn't affect other tiers |
| **Simple Auditing** | Sum of tier distributions = pool amount |

---

## 8. Comparison with Alternatives

### Alternative 1: Centralized Effective Staked

**Approach:** Track `total_effective_staked = Σ(balance × multiplier)` globally.

```
┌─────────────────────────────────────────────────────────────────┐
│              CENTRALIZED EFFECTIVE STAKED                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   index_increase = pool / total_effective_staked                 │
│   user_interest = (balance × multiplier) × index_increase        │
│                                                                  │
│   Problem: Multipliers change continuously!                      │
│                                                                  │
│   Time ────────────────────────────────────────────▶            │
│                                                                  │
│   total_staked:           ━━━━━━━━━━━━━━━━━━━━━━                │
│                           (stable between stakes)                │
│                                                                  │
│   total_effective:        ╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱                  │
│                           (constantly increasing!)               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Problems:**

| Issue | Severity | Description |
|-------|----------|-------------|
| **Sync Frequency** | 🔴 Critical | Need continuous sync as multipliers grow |
| **State Drift** | 🔴 Critical | Global total becomes stale between syncs |
| **Scalability** | 🔴 Critical | O(n) recalculation on every change |
| **Complexity** | 🟠 High | Each shard must track effective totals |

### Alternative 2: Epoch-Based Snapshots

**Approach:** Freeze multipliers at epoch boundaries (e.g., weekly).

```
┌─────────────────────────────────────────────────────────────────┐
│                   EPOCH-BASED SNAPSHOTS                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Week 1: Snapshot all user multipliers                          │
│           │                                                      │
│           ▼                                                      │
│   ┌───────────────────┐                                         │
│   │ Alice: 1.2x       │  Frozen for entire week                 │
│   │ Bob:   1.8x       │                                         │
│   │ Carol: 2.0x       │                                         │
│   └───────────────────┘                                         │
│           │                                                      │
│           ▼  (distribute based on snapshots)                    │
│                                                                  │
│   Week 2: Re-snapshot with updated multipliers                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Problems:**

| Issue | Severity | Description |
|-------|----------|-------------|
| **Delayed Updates** | 🟠 Medium | Users wait until epoch for new multiplier |
| **Gaming Risk** | 🟠 Medium | Users time stakes around snapshot dates |
| **UX Confusion** | 🟡 Low | "Why hasn't my multiplier updated?" |

### Alternative 3: Bonus Pool (Inflationary)

**Approach:** Penalty pool distributed equally; extra bonus minted for loyalty.

```
┌─────────────────────────────────────────────────────────────────┐
│                   BONUS POOL (INFLATIONARY)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Penalty Pool (100 GHC) ──▶ Distributed by balance (fair)      │
│                                +                                 │
│   Minted Bonus (50 GHC)  ──▶ Distributed by loyalty (extra)     │
│                                                                  │
│   Total: 150 GHC distributed, but 50 GHC is NEW supply          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Problems:**

| Issue | Severity | Description |
|-------|----------|-------------|
| **Inflation** | 🔴 Critical | Increases token supply, dilutes value |
| **Treasury Drain** | 🔴 Critical | Bonus comes from somewhere |
| **Economic Risk** | 🟠 High | Unpredictable long-term effects |

### Why Discrete Tiers Win

```
═══════════════════════════════════════════════════════════════════
                    COMPARISON MATRIX
═══════════════════════════════════════════════════════════════════

                    │ Scalable │ Reliable │ Zero-Sum │ Simple │
────────────────────┼──────────┼──────────┼──────────┼────────┤
Centralized Eff.    │    ❌    │    ❌    │    ✅    │   ❌   │
Epoch Snapshots     │    ✅    │    ⚠️    │    ✅    │   ⚠️   │
Bonus Pool          │    ✅    │    ✅    │    ❌    │   ✅   │
DISCRETE TIERS      │    ✅    │    ✅    │    ✅    │   ✅   │
────────────────────┴──────────┴──────────┴──────────┴────────┘
```

### Key Advantages of Discrete Tiers

1. **O(1) Complexity**: Only 4 tier totals, not per-user multipliers
2. **Stable Between Syncs**: Tier totals only change on stake/unstake/upgrade
3. **Zero-Sum Guaranteed**: Each tier pool is fully distributed within tier
4. **Intuitive UX**: Clear progression (Bronze → Silver → Gold → Diamond)
5. **Gamification**: Tier badges, progress bars, milestone celebrations
6. **Predictable Rates**: Users can see each tier's current APY

---

## 6. Implementation Details

### Data Structures

**staking_hub — GlobalStats (updated)**

```rust
pub const NUM_TIERS: usize = 4;
pub const TIER_WEIGHTS: [u8; 4] = [20, 25, 30, 25]; // % of pool per tier

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GlobalStats {
    // Existing
    pub total_staked: u64,
    pub interest_pool: u64,
    pub total_unstaked: u64,
    pub total_allocated: u64,
    pub total_rewards_distributed: u64,
    
    // NEW
    pub tier_staked: [u64; 4],
    pub tier_reward_indexes: [u128; 4],
}
```

**user_profile — UserProfile (updated)**

```rust
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserProfile {
    // Existing
    pub email: String,
    pub name: String,
    pub education: String,
    pub gender: String,
    pub staked_balance: u64,
    pub unclaimed_interest: u64,
    pub transaction_count: u64,
    
    // NEW (replacing last_reward_index)
    pub current_tier: u8,
    pub tier_start_index: u128,
    pub initial_stake_time: u64,
    pub last_tier_check: u64,
}
```

### Core Functions

**distribute_interest() — staking_hub**

```rust
#[update]
fn distribute_interest() -> Result<String, String> {
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        if stats.interest_pool == 0 {
            return Err("No interest to distribute".to_string());
        }
        
        let pool = stats.interest_pool;
        let mut distributed = 0u64;
        
        for tier in 0..NUM_TIERS {
            if stats.tier_staked[tier] > 0 {
                let tier_pool = (pool as u128 * TIER_WEIGHTS[tier] as u128 / 100) as u64;
                let index_increase = (tier_pool as u128 * 1e18 as u128) 
                                     / stats.tier_staked[tier] as u128;
                stats.tier_reward_indexes[tier] += index_increase;
                distributed += tier_pool;
            }
        }
        
        stats.interest_pool = pool - distributed;
        stats.total_rewards_distributed += distributed;
        
        cell.set(stats).unwrap();
        Ok(format!("Distributed {} GHC", distributed))
    })
}
```

**compound_interest() — user_profile**

```rust
fn compound_interest(user: Principal) {
    let now = ic_cdk::api::time();
    
    // 1. Check for tier upgrade
    check_and_handle_tier_upgrade(user, now);
    
    // 2. Calculate interest in current tier
    let tier_indexes = TIER_REWARD_INDEXES.with(|i| i.borrow().clone());
    
    USER_PROFILES.with(|p| {
        let mut map = p.borrow_mut();
        if let Some(mut profile) = map.get(&user) {
            let tier = profile.current_tier as usize;
            let current_index = tier_indexes[tier];
            
            if current_index > profile.tier_start_index {
                let index_diff = current_index - profile.tier_start_index;
                let interest = (profile.staked_balance as u128 * index_diff) / 1e18 as u128;
                
                if interest > 0 {
                    profile.unclaimed_interest += interest as u64;
                    profile.tier_start_index = current_index;
                    map.insert(user, profile);
                }
            }
        }
    });
}
```

---

## 9. Migration Plan

### Phase 1: Update staking_hub

1. Add `tier_staked` and `tier_reward_indexes` to GlobalStats
2. Initialize all existing stakers in Bronze tier
3. Set Bronze index equal to current cumulative_reward_index
4. Deploy and verify

### Phase 2: Update user_profile Shards

1. Add new fields to UserProfile
2. Migrate existing users:
   - `current_tier = 0` (Bronze)
   - `tier_start_index = last_reward_index`
   - `initial_stake_time = now` (or estimate from history)
3. Update sync protocol to use `sync_shard_v2`
4. Deploy shards one-by-one

### Phase 3: Gradual Tier Population

After migration:
- All users start in Bronze
- Over 30/90/365 days, users naturally progress
- Tier distribution stabilizes organically

```
═══════════════════════════════════════════════════════════════════
                    MIGRATION TIMELINE
═══════════════════════════════════════════════════════════════════

Day 0:    All users in Bronze
          ├── Bronze: 100% of stakers
          └── Others: 0%

Day 30:   Early stakers reach Silver
          ├── Bronze: 70%
          ├── Silver: 30%
          └── Others: 0%

Day 90:   First Gold members
          ├── Bronze: 50%
          ├── Silver: 35%
          ├── Gold: 15%
          └── Diamond: 0%

Day 365:  Full tier distribution
          ├── Bronze: 40%
          ├── Silver: 25%
          ├── Gold: 20%
          └── Diamond: 15%
```

---

## 10. Frontend Integration

### Display User Tier

```javascript
const TIER_NAMES = ['Bronze', 'Silver', 'Gold', 'Diamond'];
const TIER_COLORS = ['#CD7F32', '#C0C0C0', '#FFD700', '#B9F2FF'];
const TIER_THRESHOLDS_DAYS = [0, 30, 90, 365];

const profile = await userProfileActor.get_profile(userPrincipal);

const currentTier = TIER_NAMES[profile.current_tier];
const tierColor = TIER_COLORS[profile.current_tier];

const daysStaked = Math.floor(
    (Date.now() * 1_000_000 - Number(profile.initial_stake_time)) 
    / (86400 * 1e9)
);

const nextTier = profile.current_tier < 3 
    ? TIER_NAMES[profile.current_tier + 1] 
    : null;
const daysToNext = nextTier 
    ? TIER_THRESHOLDS_DAYS[profile.current_tier + 1] - daysStaked 
    : 0;

console.log(`🏆 Tier: ${currentTier}`);
console.log(`📅 Days Staked: ${daysStaked}`);
if (nextTier) {
    console.log(`⏳ ${daysToNext} days until ${nextTier}`);
}
```

### Display Tier APY Comparison

```javascript
const stats = await stakingHubActor.get_global_stats();
const TIER_WEIGHTS = [20, 25, 30, 25];

console.log('📊 Current Tier Rates:');

for (let i = 0; i < 4; i++) {
    const staked = Number(stats.tier_staked[i]) / 1e8;
    const poolShare = (Number(stats.interest_pool) / 1e8) * TIER_WEIGHTS[i] / 100;
    const apy = staked > 0 
        ? ((poolShare / staked) * 365 * 100).toFixed(2) 
        : '∞';
    
    console.log(`  ${TIER_NAMES[i]}: ${staked.toFixed(0)} GHC staked → ~${apy}% APY`);
}
```

---

## 11. Security Analysis

This section provides a comprehensive security assessment of the Discrete Tier System, covering potential attack vectors, mitigations, and security guarantees.

### Threat Model Overview

```
╔═══════════════════════════════════════════════════════════════════╗
║                      THREAT MODEL                                  ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                    ║
║   ADVERSARY GOALS:                                                ║
║   ├─ Steal interest that belongs to others                       ║
║   ├─ Inflate their tier status without staking duration          ║
║   ├─ Manipulate tier totals to increase their share              ║
║   ├─ Game the system by timing stakes around distributions       ║
║   └─ Cause denial of service or state corruption                 ║
║                                                                    ║
║   TRUST ASSUMPTIONS:                                              ║
║   ├─ IC consensus is secure                                       ║
║   ├─ Canister code executes as written                           ║
║   ├─ Time source (ic_cdk::api::time) is reliable                 ║
║   └─ Inter-canister calls are authenticated                      ║
║                                                                    ║
╚═══════════════════════════════════════════════════════════════════╝
```

### Security Properties

| Property | Status | Description |
|----------|--------|-------------|
| **Tier Integrity** | ✅ Secure | Users cannot fake their tier status |
| **Interest Accuracy** | ✅ Secure | Users receive exactly their share |
| **Zero-Sum Guarantee** | ✅ Secure | Total distributed = Total collected |
| **Time Manipulation** | ✅ Secure | Uses IC system time, not user input |
| **Sybil Resistance** | ✅ Secure | Splitting accounts provides no benefit |
| **Front-Running** | ✅ Secure | Distribution timing is unpredictable |

---

### Detailed Threat Analysis

#### Threat 1: Tier Status Forgery

**Attack:** Attacker tries to claim a higher tier than earned.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Attacker stakes on Day 1                                       │
│   On Day 5, tries to claim Diamond tier interest                │
│                                                                  │
│   Expected: Bronze rate (4%)                                     │
│   Attempted: Diamond rate (50%)                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mitigation:**

```rust
// Tier is calculated from initial_stake_time, not user input
fn get_user_tier(profile: &UserProfile, now: u64) -> u8 {
    let duration = now.saturating_sub(profile.initial_stake_time);
    get_tier_for_duration(duration)  // Deterministic calculation
}

// initial_stake_time is set ONCE when user first stakes
// Cannot be modified by user afterward
```

**Security Guarantee:** Tier status is derived from immutable, system-controlled timestamps.

---

#### Threat 2: Time Manipulation

**Attack:** Attacker tries to manipulate the time source to accelerate tier progression.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Attacker attempts to:                                          │
│   ├─ Pass fake timestamp to tier calculation                    │
│   ├─ Modify initial_stake_time to earlier date                  │
│   └─ Exploit clock skew between canisters                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mitigation:**

```rust
// Time comes from IC system, not user input
let now = ic_cdk::api::time();  // Nanoseconds since epoch

// initial_stake_time is set internally, never from user input
fn add_tokens_to_user(user: Principal, amount: u64) {
    let now = ic_cdk::api::time();  // System time
    
    if profile.initial_stake_time == 0 {
        profile.initial_stake_time = now;  // Set once, immutable
    }
}
```

**Security Guarantee:** The IC provides a consistent, tamper-proof time source across all subnet replicas.

---

#### Threat 3: Sybil Attack (Account Splitting)

**Attack:** Attacker splits stake across multiple accounts to gain advantage.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Instead of: 1 account with 100 GHC                            │
│   Attacker creates: 10 accounts with 10 GHC each                │
│                                                                  │
│   Goal: Get more interest by having multiple tier entries       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Analysis:**

```
Single account:
  100 GHC in Bronze → earns share of Bronze pool based on 100 GHC

10 accounts:
  10 GHC × 10 in Bronze → each earns share based on 10 GHC
  Total: Same as single account (100 GHC worth of shares)
```

**Security Guarantee:** Interest is proportional to staked amount, not account count. Splitting provides zero benefit and actually costs gas for multiple transactions.

---

#### Threat 4: Front-Running Distribution

**Attack:** Attacker stakes right before distribution, claims, then unstakes.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   T=0:  Attacker monitors for pending distribution              │
│   T=1:  Stakes 1000 GHC just before distribute_interest()       │
│   T=2:  Distribution happens, attacker gets large share         │
│   T=3:  Attacker unstakes immediately                           │
│         (pays 10% penalty but keeps interest profit)            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mitigations:**

1. **Tier System Itself:** New stakers are in Bronze (lowest reward rate)
2. **10% Unstake Penalty:** Attacker loses 10% of principal immediately
3. **Unpredictable Timing:** Admin can trigger distribution at any time
4. **Sync Delay:** Staked amount isn't reflected until next sync (5 sec)

**Economic Analysis:**

```
Attacker stakes 1000 GHC, gets Bronze rate (4%)
If pool has 100 GHC and Bronze has 500 GHC staked:
  Attacker's share = 20 × (1000/1500) = 13.3 GHC

But attacker loses on unstake:
  Penalty = 1000 × 10% = 100 GHC

Net loss: 100 - 13.3 = 86.7 GHC ❌
```

**Security Guarantee:** Front-running is economically unprofitable due to the 10% unstake penalty.

---

#### Threat 5: Tier Total Manipulation

**Attack:** Malicious shard reports fake tier_staked values to inflate attacker's share.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Malicious shard reports:                                       │
│   tier_staked[Diamond] = -99999 (reduce total)                  │
│                                                                  │
│   Result: Attacker's 100 GHC becomes larger share               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mitigations:**

```rust
// In staking_hub: Only registered shards can sync
#[update]
fn sync_shard_v2(...) -> Result<..., String> {
    let caller = ic_cdk::caller();
    
    // CRITICAL: Verify caller is a registered shard
    let is_registered = REGISTERED_SHARDS.with(|m| 
        m.borrow().contains_key(&caller)
    );
    
    if !is_registered {
        return Err("Unauthorized".to_string());
    }
    
    // Additional: Use saturating arithmetic to prevent underflow
    stats.tier_staked[tier] = stats.tier_staked[tier]
        .saturating_sub(delta.abs() as u64);
}
```

**Security Guarantees:**

1. Only shards created by staking_hub can report stats
2. Shards are deployed with embedded WASM from hub
3. `saturating_sub` prevents underflow attacks
4. Hub is the controller of all shards

---

#### Threat 6: Interest Calculation Overflow

**Attack:** Large values cause arithmetic overflow, corrupting interest calculations.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   If: staked_balance × index_diff > u128::MAX                   │
│   Result: Overflow, incorrect interest calculated               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mitigations:**

```rust
// Use checked/saturating arithmetic
let interest = (profile.staked_balance as u128)
    .saturating_mul(index_diff)
    .checked_div(1_000_000_000_000_000_000)
    .unwrap_or(0);  // Safe default on division issues

// Maximum values analysis:
//   Max staked_balance: 4.2B × 1e8 = 4.2e17
//   Max index_diff (realistic): 1e20 (100x total supply distributed)
//   Product: 4.2e37 < u128::MAX (3.4e38) ✓
```

**Security Guarantee:** Using u128 for intermediate calculations and saturating arithmetic prevents overflow.

---

#### Threat 7: Tier Downgrade on Unstake

**Attack:** User unstakes partial amount, resets tier to Bronze.

```
┌─────────────────────────────────────────────────────────────────┐
│  ATTACK SCENARIO                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   User at Diamond tier (365+ days) with 100 GHC                 │
│   Unstakes 10 GHC                                                │
│   Still has 90 GHC staked                                        │
│                                                                  │
│   Question: Should they stay Diamond or reset to Bronze?        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Design Decision:**

```rust
// POLICY: Tier is based on CONTINUOUS staking duration
// Partial unstake maintains tier if balance remains positive

fn process_unstake(user: Principal, amount: u64) {
    // After unstake, check if balance > 0
    if profile.staked_balance > amount {
        // Keep current tier, no reset
        profile.staked_balance -= amount;
    } else {
        // Full unstake: reset everything
        profile.staked_balance = 0;
        profile.initial_stake_time = 0;  // Reset timer
        profile.current_tier = 0;         // Back to Bronze when restaking
    }
}
```

**Security Guarantee:** Clear, documented policy for tier behavior on unstake prevents ambiguity.

---

### Security Checklist

```
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY CHECKLIST                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ ✅ Input Validation                                              │
│    ├─ All amounts validated (> 0, <= balance)                   │
│    ├─ Principal verification on all shard calls                 │
│    └─ Tier values bounded to 0-3                                │
│                                                                  │
│ ✅ Arithmetic Safety                                             │
│    ├─ saturating_add/sub for balance changes                    │
│    ├─ u128 for intermediate calculations                        │
│    └─ Division-by-zero checks on tier_staked                    │
│                                                                  │
│ ✅ Access Control                                                │
│    ├─ Only registered shards can sync                           │
│    ├─ distribute_interest() callable by admin only              │
│    └─ Users can only modify their own profile                   │
│                                                                  │
│ ✅ State Consistency                                             │
│    ├─ Tier deltas sum to zero on upgrade                        │
│    ├─ Interest claimed before tier change                       │
│    └─ Atomic operations on profile updates                      │
│                                                                  │
│ ✅ Economic Security                                             │
│    ├─ 10% penalty prevents stake-and-run attacks                │
│    ├─ Zero-sum distribution prevents inflation                  │
│    └─ Tier pools are isolated (no cross-contamination)          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Invariants to Maintain

The system must maintain these invariants at all times:

```rust
// INVARIANT 1: Tier totals match sum of user balances
assert_eq!(
    sum(tier_staked),
    sum(all_user_staked_balances)
);

// INVARIANT 2: User is in exactly one tier
assert!(profile.current_tier <= 3);

// INVARIANT 3: Distribution is zero-sum
assert_eq!(
    interest_pool_before,
    sum(tier_distributions) + interest_pool_after
);

// INVARIANT 4: Tier progression is monotonic (within a staking session)
// User can only move UP in tiers while continuously staking
assert!(new_tier >= old_tier || profile.staked_balance == 0);

// INVARIANT 5: initial_stake_time is immutable once set
assert!(
    profile.initial_stake_time == old_initial_stake_time ||
    old_staked_balance == 0
);
```

---

### Comparison: Security vs. Alternatives

```
═══════════════════════════════════════════════════════════════════
                    SECURITY COMPARISON
═══════════════════════════════════════════════════════════════════

                     │ Discrete │ Centralized │ Epoch  │ Bonus  │
                     │  Tiers   │  Effective  │ Snap   │  Pool  │
─────────────────────┼──────────┼─────────────┼────────┼────────┤
 Time Manipulation   │    ✅    │      ✅     │   ⚠️   │   ✅   │
 Front-Running       │    ✅    │      ⚠️     │   ❌   │   ✅   │
 Sybil Resistance    │    ✅    │      ✅     │   ✅   │   ✅   │
 Overflow Safety     │    ✅    │      ⚠️     │   ✅   │   ✅   │
 State Consistency   │    ✅    │      ❌     │   ✅   │   ✅   │
 Zero-Sum Guarantee  │    ✅    │      ✅     │   ✅   │   ❌   │
─────────────────────┴──────────┴─────────────┴────────┴────────┘

Legend:
  ✅ = Secure by design
  ⚠️ = Requires additional mitigations
  ❌ = Vulnerable or not applicable
```

**Key Insight:** The Discrete Tier System has the best security profile because:
1. Tier status is derived from immutable system time
2. Isolated pools prevent cross-tier contamination
3. Simple state model reduces consistency bugs
4. Economic penalties deter gaming

---

## Summary

The Discrete Tier System provides:

| Property | Value |
|----------|-------|
| **Fairness** | Long-term stakers earn more per token |
| **Sustainability** | Zero-sum distribution, no inflation |
| **Scalability** | O(1) global tracking (just 4 tiers) |
| **Simplicity** | Clear tier progression, easy to understand |
| **Gamification** | Badges, progress, milestone celebrations |

```
╔═══════════════════════════════════════════════════════════════════╗
║                                                                    ║
║   "The longer you stake, the more you make"                       ║
║                                                                    ║
║   Bronze ──▶ Silver ──▶ Gold ──▶ Diamond                          ║
║     20%       25%       30%       25%                              ║
║                                                                    ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## Related Documentation

- [INTEREST_FLOW.md](./INTEREST_FLOW.md) — Base interest mechanics
- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) — Staking operations
- [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) — API reference
