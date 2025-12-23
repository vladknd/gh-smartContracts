# GHC Advanced Interest System: Complete Technical Guide

> **Status:** Approved Architecture  
> **Last Updated:** December 2025  
> **Related:** [STAKING_MECHANICS.md](./STAKING_MECHANICS.md), [INTEREST_FLOW.md](./INTEREST_FLOW.md)

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Key Definitions](#2-key-definitions)
3. [The Zero-Sum Economic Model](#3-the-zero-sum-economic-model)
4. [Tier System Architecture](#4-tier-system-architecture)
5. [Staking Age Calculation](#5-staking-age-calculation)
6. [Weighted Average Age Formula](#6-weighted-average-age-formula)
7. [Interest Distribution Mechanics](#7-interest-distribution-mechanics)
8. [The Global Reward Index](#8-the-global-reward-index)
9. [User Interest Calculation](#9-user-interest-calculation)
10. [Complete Flow Diagrams](#10-complete-flow-diagrams)
11. [Edge Cases and Examples](#11-edge-cases-and-examples)
12. [Implementation Reference](#12-implementation-reference)

---

## 1. System Overview

The GHC Interest System rewards users who stake their tokens longer. It operates on a **Zero-Sum, Non-Inflationary** model, meaning:

- **No new tokens are ever created** to pay interest
- All interest comes from **penalties** collected when users unstake
- Longer stakers earn more because they share a pool with fewer people

```text
+=====================================================================+
|                    HIGH-LEVEL SYSTEM OVERVIEW                        |
+=====================================================================+

    USER UNSTAKES                           STAKERS RECEIVE
    (Pays Penalty)                          (Earn Interest)
         |                                        ^
         |                                        |
         v                                        |
    +----------+       +-----------+       +-------------+
    | 10% Fee  | ----> | INTEREST  | ----> | DISTRIBUTED |
    | Collected|       |   POOL    |       | TO 4 TIERS  |
    +----------+       +-----------+       +-------------+
                            |
                            v
              +---------------------------+
              |  Split by Tier Weights:   |
              |  Bronze: 20%              |
              |  Silver: 25%              |
              |  Gold:   30%              |
              |  Diamond: 25%             |
              +---------------------------+
```

---

## 2. Key Definitions

Before diving into the mechanics, here are the critical terms used throughout this document:

### 2.1 Staking Age

**Definition:** The amount of time (in days) that has passed since the beginning of staking process.

**Formula:**
```text
Staking_Age = Current_Time - Staking_Time
```
### 2.2 Staking_Time:
**Definition:** The virtual (averaged) timestamp representing the beggining of staking process of user's tokens. This value is adjusted when new tokens are added.
**Purpose:** Determines the start of staking period for Staking Age calculation.

**Example:**
- User started staking on January 1, 2025
- Today is April 15, 2025
- Staking Age = 104 days
- User qualifies for **Gold Tier** (90-365 days)

**Formula:**
```text
staking_time = Current_Time - Weighted_Avg_Age
```
---

### 2.3 Weighted Average Age

**Definition:** A single number representing the "effective maturity" of a user's entire token balance, accounting for tokens deposited at different times.

**Why Needed:** Users earn tokens daily from quizzes. Each batch of tokens has a different "birthday." Instead of tracking thousands of individual deposits, we calculate one weighted average.

**Key Design: 80% Maturity Factor**

To reward loyal stakers and prevent excessive age dilution from small daily deposits, we apply an **80% Multiplication Factor (α = 0.8)**. This means new tokens only contribute **20%** of their dilution effect, preserving most of the existing tokens' earned age.

**Formula:**
```text
MULTIPLICATION_FACTOR = 0.8  (configurable, currently 80%)

                         (staked_balance * Staking_Age)
Weighted_Avg_Age = -----------------------------------------------
                    staked_balance + New_Tokens * (1 - MULTIPLICATION_FACTOR)


Simplified (with α = 0.8):
                         (staked_balance * Staking_Age)
Weighted_Avg_Age = -----------------------------------------
                         staked_balance + New_Tokens * 0.2


staking_time x  = Current_Time - Weighted_Avg_Age
```

**Why 80%?**
- **Without offset (50%):** User earning daily takes ~730 days to reach Diamond
- **With 80% offset:** User earning daily reaches Diamond in ~456 days
- The 80% factor rewards consistent participation while still requiring genuine time commitment
- Prevents gaming: depositing 1 token doesn't "game" the age, but also doesn't destroy it

**How to Calculate Each Piece:**

| Component | What It Is | How to Get It |
|-----------|------------|---------------|
| `staking_time` | Timestamp representing a virtual(averagge) age of tokens | Read from user profile (recalculated after each deposit) |
| `staked_balance` | Tokens already staked | Read from user profile |
| `Staking_Age` | Duration (days) since staking began | Calculate: `Current_Time - staking_time` |
| `New_Tokens` | Tokens being added now | The amount earned from quiz |
| `New_Age` | Age of new tokens | Always 0 (brand new) |

**Data Storage Reference:**
```text
+=====================+============================+==========================================+
|     DATA ITEM       |     WHERE STORED           |     NOTES                                 |
+=====================+============================+==========================================+
| staking_time        | user_profile canister      | Recalculated and updated after deposit    |
| staked_balance      | user_profile canister      | Updated after each deposit                |
| Staking_Age         | Runtime calculation        | = Current_Time - staking_time             |
| Weighted_Avg_Age    | Runtime calculation        | Used to update staking_time               |
| Current_Time        | System clock (ic_cdk::time)| Always "now"                              |
+=====================+============================+===========================================+

**Example: Growing from Zero (Step-by-Step Daily Deposits)**
```text
+=======================================================================+
|                 DAY 1: USER'S FIRST DEPOSIT                           |
+=======================================================================+

BEFORE (from user_profile canister):
  staked_balance     = 0 GHC     (no prior balance)
  staking_time       = 0         (never staked before)

EVENT: User earns 5 GHC from their first quiz

CALCULATION:
  This is the FIRST deposit, so:
  - No weighted average needed
  - staking_time is set to NOW (Day 1) which is the as first deposit staking time 

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 5 GHC              │ ← STORED
  │ staking_time       = Day 1              │ ← STORED
  └─────────────────────────────────────────┘
```
TIMELINE:
Day 1     (+5GHC)         
           │                        
           ▼                       
           Day(1)      
─────────────────────────────────────────────────────────────────────▶
           ^
           |
       staking_time

```
+=======================================================================+
|                 DAY 2: SECOND DEPOSIT                                 |
+=======================================================================+

BEFORE (from user_profile canister):
  staked_balance     = 5 GHC
  staking_time       = Day 1

EVENT: User earns 5 GHC from today's quiz
  Current_Time       = Day 2
  New_Tokens         = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Current_Time - staking_time
          = Day 2 - Day 1
          = 1 day

STEP 2: Calculate Coin-Days (numerator)
  Coin_Days = staked_balance * Staking_Age
            = 5 * 1
            = 5 coin-days

STEP 3: Calculate Effective Denominator (with 80% maturity factor)
  Effective_Denominator = staked_balance + (New_Tokens * 0.2)
                        = 5 + (5 * 0.2)
                        = 5 + 1 = 6

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = Coin_Days / Effective_Denominator
                   = 5 / 6
                   = 0.833 days

STEP 5: Convert Age back to Timestamp
  new_staking_time = Current_Time - Weighted_Avg_Age
                         = Day 2 - 0.833
                         = Day 1.167

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 10 GHC             │ ← STORED
  │ staking_time       = Day 1.167          │ ← STORED (drifted +0.167)
  └─────────────────────────────────────────┘

```
TIMELINE: 
        (+5GHC)                   (+5GHC)               
           │                         │                   
           ▼                         ▼                   
         Day(1)  Day(1.167)        Day(2)          
─────────────────────────────────────────────────────────────────────▶
                    ^
                    |
                staking_time
```


+=======================================================================+
|                 DAY 3: THIRD DEPOSIT                                  |
+=======================================================================+

BEFORE (from user_profile canister):
  staked_balance     = 10 GHC
  staking_time       = Day 1.167

EVENT: User earns 5 GHC from today's quiz
  Current_Time       = Day 3
  New_Tokens         = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 3 - Day 1.167 = 1.833 days

STEP 2: Calculate Coin-Days
  Coin_Days = 10 * 1.833 = 18.33 coin-days

STEP 3: Calculate Effective Denominator (with 80% maturity factor)
  Effective_Denominator = 10 + (5 * 0.2) = 11

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 18.33 / 11 = 1.667 days

STEP 5: Convert to Timestamp
  new_staking_time = Day 3 - 1.667 = Day 1.333

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 15 GHC             │ ← STORED
  │ staking_time       = Day 1.333          │ ← STORED (drifted +0.166)
  └─────────────────────────────────────────┘
```
TIMELINE: 
        (+5GHC)                   (+5GHC)               (+5GHC)
           │                         │                    │
           ▼                         ▼                    ▼
         Day(1)  Day(1.333)        Day(2)               Day(3)
─────────────────────────────────────────────────────────────────────▶
                    ^
                    |
                staking_time

                    |◄───── ~1.67 days (Staking_Age)─────►|
```

+=======================================================================+
|                 DAY 10: PATTERN CONTINUES                             |
+=======================================================================+

Let's fast-forward to see the accumulated effect WITH the 80% maturity factor:

BEFORE (from user_profile canister):
  staked_balance     = 45 GHC    (9 days × 5 GHC)
  staking_time       = Day 2.2   (drifted from Day 1 - much slower drift!)

EVENT: User earns 5 GHC from today's quiz
  Current_Time = Day 10
  New_Tokens   = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 10 - Day 2.2 = 7.8 days

STEP 2: Calculate Coin-Days
  Coin_Days = 45 * 7.8 = 351 coin-days

STEP 3: Calculate Effective Denominator (with 80% maturity factor)
  Effective_Denominator = 45 + (5 * 0.2) = 46

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 351 / 46 = 7.63 days

STEP 5: Convert to Timestamp
  new_staking_time = Day 10 - 7.63 = Day 2.37

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 50 GHC             │ ← STORED
  │ staking_time       = Day 2.37           │ ← STORED (drifted +0.17)
  └─────────────────────────────────────────┘
  
```
TIMELINE (80% Maturity Factor): 
        (+5GHC)                                         (+5GHC)
           │                         ...                   │
           ▼                                               ▼
         Day(1)   Day(2.37)                              Day(10)
─────────────────────────────────────────────────────────────────────▶
                     ^
                     |
                 staking_time 

                     |◄───── ~7.63 days (Staking_Age)─────►|
``` 

+=======================================================================+
|                 DAY 10: USER DECIDES TO UNSTAKE                       |
+=======================================================================+

Now imagine the user decides to unstake some or all of their tokens.

CURRENT STATE (from user_profile canister):
  staked_balance     = 50 GHC
  staking_time       = Day 5.95
  Current_Time       = Day 10

PENALTY TIERS (for reference):
┌─────────────────────┬────────────────┐
│ Staking Duration    │ Penalty Rate   │
├─────────────────────┼────────────────┤
│ 0 - 30 days         │ 15%            │
│ 30 - 90 days        │ 10%            │
│ 90 - 365 days       │ 5%             │
│ 365+ days           │ 2%             │
└─────────────────────┴────────────────┘

─────────────────────────────────────────────────────────────────────────
SCENARIO A: PARTIAL UNSTAKE (User unstakes 20 GHC, keeps 30 GHC)
─────────────────────────────────────────────────────────────────────────

STEP 1: Calculate Staking_Age
  Staking_Age = Current_Time - staking_time
              = Day 10 - Day 5.95
              = 4.05 days

STEP 2: Determine Penalty Bracket
  4.05 days falls in "0-30 days" bracket → 15% penalty

STEP 3: Calculate Penalty
  Unstake_Amount = 20 GHC
  Penalty = 20 × 15% = 3 GHC
  User_Receives = 20 - 3 = 17 GHC
  Interest_Pool_Receives = 3 GHC

STEP 4: Update State
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 30 GHC             │ ← REDUCED by 20
  │ staking_time       = Day 5.95           │ ← UNCHANGED!
  └─────────────────────────────────────────┘

NOTE: staking_time stays the same! The remaining 30 GHC keeps its age.
      If user deposits more later, weighted average will be applied.

─────────────────────────────────────────────────────────────────────────
SCENARIO B: FULL UNSTAKE (User unstakes all 50 GHC)
─────────────────────────────────────────────────────────────────────────

STEP 1: Calculate Staking_Age
  Staking_Age = Day 10 - Day 5.95 = 4.05 days

STEP 2: Determine Penalty Bracket
  4.05 days falls in "0-30 days" bracket → 15% penalty

STEP 3: Calculate Penalty
  Unstake_Amount = 50 GHC
  Penalty = 50 × 15% = 7.5 GHC
  User_Receives = 50 - 7.5 = 42.5 GHC
  Interest_Pool_Receives = 7.5 GHC

STEP 4: Update State (full reset)
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 0                  │ ← RESET to 0
  │ staking_time       = 0                  │ ← RESET to 0
  └─────────────────────────────────────────┘

NOTE: Full unstake = clean slate. Next deposit will be treated as
      the user's first deposit again (no history carried over).


WHY staking_time UNCHANGED ON PARTIAL UNSTAKE?
  - Unstaking doesn't add new tokens (no dilution)
  - Remaining tokens earned their age fairly
  - If we reset staking_time, we'd punish users for partial withdrawals
  - The math is simpler and fairer


|                 DAY 100: LONG-TERM VIEW                               |
+=======================================================================+

After 100 daily deposits of 5 GHC each (WITH 80% Maturity Factor):

BEFORE (from user_profile canister):
  staked_balance     = 495 GHC
  staking_time       = Day 20.5 (drifted from Day 1 - much slower!)

EVENT: User earns 5 GHC from today's quiz
  Current_Time = Day 100
  New_Tokens   = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 100 - Day 20.5 = 79.5 days

STEP 2: Calculate Coin-Days
  Coin_Days = 495 * 79.5 = 39,352.5 coin-days

STEP 3: Calculate Effective Denominator (with 80% maturity factor)
  Effective_Denominator = 495 + (5 * 0.2) = 496

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 39,352.5 / 496 = 79.34 days

STEP 5: Convert to Timestamp
  new_staking_time = Day 100 - 79.34 = Day 20.66

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 500 GHC            │ ← STORED
  │ staking_time       = Day 20.66          │ ← STORED (drifted +0.16)
  └─────────────────────────────────────────┘

```
TIMELINE (80% Maturity Factor - 100 Days): 
        (+5GHC)                                                        (+5GHC)
           │              ...daily deposits...                            │
           ▼                                                              ▼
         Day(1)             Day(20.66)                                  Day(100)
─────────────────────────────────────────────────────────────────────────────────▶
                               ^
                               |
                           staking_time (only ~21% drift after 100 days!)
                                   
                                |◄─────── ~79 days (Staking_Age) ───────►|
```

+=======================================================================+
|                 OBSERVATION (WITH 80% MATURITY FACTOR)                |
+=======================================================================+

After 100 days of continuous daily deposits:
- The effective staking age is ~79.34 days (~80% of elapsed time!)
- Each new deposit only reduces the age by ~0.16 days (vs ~0.5 with 50%)

COMPARISON:
┌─────────────────────┬──────────────────┬──────────────────┐
│ Metric              │ No Offset (50%)  │ With 80% Factor  │
├─────────────────────┼──────────────────┼──────────────────┤
│ Day 100 Age         │ ~49 days         │ ~79 days         │
│ Drift per deposit   │ ~0.5 days        │ ~0.16 days       │
│ Days to reach Gold  │ ~180 days        │ ~112 days        │
│ Days to reach Diamond│ ~730 days       │ ~456 days        │
└─────────────────────┴──────────────────┴──────────────────┘

The 80% maturity factor rewards consistent participation MORE STRONGLY
while still preventing gaming through tiny deposits.
```

**Why Not Track Each Batch Separately?**

You might ask: *"Why not just record each deposit as a separate entry and calculate interest for each one individually?"*

Here's why that approach is **not feasible**:

```text
+=======================================================================+
|         ALTERNATIVE APPROACH: PER-BATCH TRACKING (INFEASIBLE)         |
+=======================================================================+

SCENARIO: User earns 5 GHC daily for 1 year

STORAGE REQUIRED (in user_profile canister):
┌──────────────────────────────────────────────────────────────────────┐
│  Instead of 2 fields (staked_balance, staking_time), you need: │
│                                                                      │
│  deposits: Vec<Deposit> = [                                          │
│    { amount: 5, timestamp: Day 1   },   ← Record #1                  │
│    { amount: 5, timestamp: Day 2   },   ← Record #2                  │
│    { amount: 5, timestamp: Day 3   },   ← Record #3                  │
│    ...                                                               │
│    { amount: 5, timestamp: Day 365 },   ← Record #365                │
│  ]                                                                   │
│                                                                      │
│  STORAGE COST PER USER:                                              │
│  - Weighted Average: ~16 bytes (1 u64 balance + 1 u64 timestamp)     │
│  - Per-Batch:        ~5,840 bytes (365 × 16 bytes) for Year 1 ONLY   │
│                                                                      │
│  For 1 million users over 5 years:                                   │
│  - Weighted Average: 16 MB total                                     │
│  - Per-Batch:        29.2 GB total (1825 records × 16 bytes × 1M)    │
└──────────────────────────────────────────────────────────────────────┘

COMPUTATION REQUIRED (on every interest calculation):
┌──────────────────────────────────────────────────────────────────────┐
│  WEIGHTED AVERAGE APPROACH:                                          │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │ fn calculate_interest(user):                                    │ │
│  │   age = now - user.staking_time        // 1 subtraction  │ │
│  │   tier = get_tier(age)                       // 1 lookup       │ │
│  │   index_diff = global_index - user.last_index                  │ │
│  │   interest = user.staked_balance * index_diff                  │ │
│  │   return interest                                               │ │
│  │                                                                 │ │
│  │   TIME COMPLEXITY: O(1) - constant time                        │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  PER-BATCH APPROACH:                                                 │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │ fn calculate_interest(user):                                    │ │
│  │   total_interest = 0                                            │ │
│  │   for deposit in user.deposits:      // Loop 365+ times/year   │ │
│  │     age = now - deposit.timestamp                               │ │
│  │     tier = get_tier(age)             // Could be different!    │ │
│  │     index = get_tier_index(tier)     // Fetch per deposit      │ │
│  │     interest = deposit.amount * (index - deposit.last_index)   │ │
│  │     total_interest += interest                                  │ │
│  │   return total_interest                                         │ │
│  │                                                                 │ │
│  │   TIME COMPLEXITY: O(n) - grows with every deposit             │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘

ADDITIONAL COMPLEXITY:
┌──────────────────────────────────────────────────────────────────────┐
│     UNSTAKING NIGHTMARE                                              │
│     When user unstakes, which deposits do you remove first?         │
│     FIFO? LIFO? Oldest first? Newest first?          │
│     Each choice has different economic implications.                 │
└──────────────────────────────────────────────────────────────────────┘

COMPARISON SUMMARY:
┌────────────────────┬──────────────────────┬──────────────────────────┐
│ Metric             │ Weighted Average     │ Per-Batch Tracking       │
├────────────────────┼──────────────────────┼──────────────────────────┤
│ Storage per user   │ O(1) - 16 bytes      │ O(n) - 16 bytes × days   │
│ Interest calc time │ O(1) - constant      │ O(n) - linear in deposits│
│ Sync operations    │ 1 per deposit event  │ n per interest calc      │
│ Code complexity    │ Simple               │ Very complex             │
│ Unstaking logic    │ Single deduction     │ Multi-record management  │
└────────────────────┴──────────────────────┴──────────────────────────┘

CONCLUSION:
The weighted average approach trades a small amount of precision
(~0.16 day drift per deposit with 80% factor) for massive gains in:
- Storage efficiency (1800x less data)
- Computational efficiency (O(1) vs O(n))
- Code simplicity and maintainability
- Predictable gas/cycle costs

This is why virtually all staking systems use a weighted average or
similar aggregation technique rather than tracking individual deposits.
```

---

### 2.3 Coin-Days

**Definition:** A unit measuring the "investment weight" of tokens over time.

**Formula:**
```text
Coin_Days = Number_of_Tokens * Days_Held
```

**Example:**
- 100 tokens held for 10 days = 1,000 Coin-Days
- 10 tokens held for 100 days = 1,000 Coin-Days
- Both have equal "investment weight" to the system

**Purpose:** Used as the numerator when calculating Weighted Average Age.

---

### 2.4 Tier

**Definition:** One of four categories that users are sorted into based on their Staking Age.

```text
+===========+================+==============+
|   TIER    |  REQUIRED AGE  |  POOL SHARE  |
+===========+================+==============+
|  Bronze   |   0 - 30 days  |     20%      |
+-----------+----------------+--------------+
|  Silver   |  30 - 90 days  |     25%      |
+-----------+----------------+--------------+
|  Gold     | 90 - 365 days  |     30%      |
+-----------+----------------+--------------+
|  Diamond  |    365+ days   |     25%      |
+-----------+----------------+--------------+
```

---

### 2.5 Pool Share (Tier Weight)

**Definition:** The fixed percentage of the total Interest Pool allocated to a specific tier.

**Important:** This is NOT the APY. This is the slice of the pie that goes to that tier's room. The actual APY depends on how many people are in that room.

---

### 2.6 Global Reward Index

**Definition:** A cumulative counter that tracks "total earnings per token" for each tier since the system began.

**Formula:**
```text
Index_Increase = Tier_Pool_Amount / Tier_Total_Staked
New_Index = Old_Index + Index_Increase
```

**Purpose:** Allows O(1) interest calculation for millions of users without iterating through each one.

---

### 2.7 Interest Pool

**Definition:** The accumulated GHC tokens collected from unstaking penalties, waiting to be distributed.

**Source:** 10% penalty applied when any user unstakes.

---

## 3. The Zero-Sum Economic Model

### 3.1 Why Zero-Sum?

The system is intentionally designed so that **total tokens never increase**. Every token paid as interest was first taken from someone else as a penalty.

```text
+=======================================================================+
|                         ZERO-SUM PRINCIPLE                             |
+=======================================================================+

  BEFORE DISTRIBUTION:
  +------------------+------------------+------------------+
  |  User A (Staker) |  User B (Staker) |  Interest Pool   |
  |    1,000 GHC     |    1,000 GHC     |     100 GHC      |
  +------------------+------------------+------------------+
  TOTAL IN SYSTEM: 2,100 GHC

  AFTER DISTRIBUTION (50/50 split for simplicity):
  +------------------+------------------+------------------+
  |  User A (Staker) |  User B (Staker) |  Interest Pool   |
  |    1,050 GHC     |    1,050 GHC     |       0 GHC      |
  +------------------+------------------+------------------+
  TOTAL IN SYSTEM: 2,100 GHC  <-- UNCHANGED!

  The 100 GHC moved from the pool to the users.
  No new tokens were created.
```

### 3.2 The Source of Funds: Penalties

Every time a user unstakes, they pay a penalty:

```text
  User requests to unstake 1,000 GHC
           |
           v
  +-------------------+
  | Apply 10% Penalty |
  +-------------------+
           |
           +---> 900 GHC returned to user's wallet
           |
           +---> 100 GHC sent to Interest Pool
```

### 3.3 Why This Matters

- **Guaranteed Solvency:** The system can never "run out" of money to pay interest
- **No Inflation:** Token supply stays constant, protecting value
- **Fair Redistribution:** Those who leave early subsidize those who stay

---

## 4. Tier System Architecture

### 4.1 The "Four Rooms" Concept

Think of the system as a building with 4 separate rooms. Each room receives a fixed delivery of rewards, but the rewards are shared among everyone in that room.

```text
+=======================================================================+
|                    THE FOUR ROOMS (TIERS)                              |
+=======================================================================+

                        INTEREST POOL: 1,000 GHC
                               | 
           +-------------------+-------------------+-------------------+
           |                   |                   |                   |
           v                   v                   v                   v
    +-----------+       +-----------+       +-----------+       +-----------+
    |  BRONZE   |       |  SILVER   |       |   GOLD    |       |  DIAMOND  |
    |   ROOM    |       |   ROOM    |       |   ROOM    |       |   ROOM    |
    +-----------+       +-----------+       +-----------+       +-----------+
    |  Gets 20% |       |  Gets 25% |       |  Gets 30% |       |  Gets 25% |
    |  200 GHC  |       |  250 GHC  |       |  300 GHC  |       |  250 GHC  |
    +-----------+       +-----------+       +-----------+       +-----------+
    | Population|       | Population|       | Population|       | Population|
    | 10,000    |       |  5,000    |       |  2,000    |       |    500    |
    | stakers   |       |  stakers  |       |  stakers  |       |  stakers  |
    +-----------+       +-----------+       +-----------+       +-----------+
    |Per Person:|       |Per Person:|       |Per Person:|       |Per Person:|
    | 0.02 GHC  |       | 0.05 GHC  |       | 0.15 GHC  |       | 0.50 GHC  |
    +-----------+       +-----------+       +-----------+       +-----------+
         ^                                                            ^
         |                                                            |
         +-------- SAME POOL SHARE CAN MEAN VERY DIFFERENT YIELD -----+
```



### 4.3 The "Effective Multiplier" 

The difference in yield between tiers is achieved through the**population scarcity of the tier**.

```text
+=======================================================================+
|                    EFFECTIVE MULTIPLIER EXAMPLE                        |
+=======================================================================+

  ASSUMPTIONS:
  - Total Interest Pool: 1,000 GHC
  - Each tier receives its fixed share

  BRONZE TIER:
  - Pool Share: 200 GHC
  - Total Staked in Tier: 100,000 GHC
  - Yield per Token: 200 / 100,000 = 0.002 GHC (0.2%)

  DIAMOND TIER:
  - Pool Share: 250 GHC  
  - Total Staked in Tier: 1,000 GHC (very few long-term holders)
  - Yield per Token: 250 / 1,000 = 0.25 GHC (25%)

  EFFECTIVE MULTIPLIER:
  Diamond Yield / Bronze Yield = 0.25 / 0.002 = 125x

  CONCLUSION:
  Diamond users earn 125 TIMES more per token than Bronze users,
  even though the pool allocation is similar (25% vs 20%).
```

---

## 5. Staking Age Calculation

### 5.1 The Simple Case (Single Deposit)

If a user makes one deposit and never adds more, the age calculation is simple:

```text
  Staking_Age = Current_Time - Staking_Time

  EXAMPLE:
  - Staking_Time: Day 0 (January 1, 2025)
  - Current_Time: Day 100 (April 11, 2025)
  - Staking_Age: 100 days
  - Tier: GOLD (90-365 days)
```

### 5.2 The Complex Case (Daily Earnings)

Users earn ~5 GHC per day from quizzes. Each day's earnings have a different "birthday":

```text
  DAY 0:   Earn 5 GHC   (Age: 100 days by Day 100)
  DAY 1:   Earn 5 GHC   (Age: 99 days by Day 100)
  DAY 2:   Earn 5 GHC   (Age: 98 days by Day 100)
  ...
  DAY 99:  Earn 5 GHC   (Age: 1 day by Day 100)
  DAY 100: Earn 5 GHC   (Age: 0 days)
```

**Problem:** Tracking 100 separate deposit records is not efficient.

**Solution:** Use a single "Virtual Timestamp" that represents the weighted average.

---

## 6. Weighted Average Age Formula

### 6.1 The Core Formula

When a user deposits new tokens, we recalculate their "Virtual Start Date" using the **80% Maturity Factor**:

```text
FORMULA (with 80% Maturity Factor):

MATURITY_FACTOR = 0.8

                         (Current_Balance * Staking_Age)
New_Average_Age  =  -----------------------------------------------
                    Current_Balance + Amount_Added * (1 - MATURITY_FACTOR)

Simplified:
                         (Current_Balance * Staking_Age)
New_Average_Age  =  -----------------------------------------
                         Current_Balance + Amount_Added * 0.2

WHERE:
- Current_Balance: Tokens already staked
- Staking_Age: Days since staking_time (calculated as Now - Staking_Time)
- Amount_Added: New tokens being deposited (always Age = 0)
- MATURITY_FACTOR: 0.8 (80%) - reduces dilution from new deposits
```

**Why the 0.2 multiplier on new tokens?**
With MATURITY_FACTOR = 0.8, we only count 20% of new tokens in the denominator. This means:
- New tokens still "dilute" the average age, but only by 20%
- Existing tokens' age is preserved much more strongly
- Users reach higher tiers faster with consistent participation

### 6.2 Formula Breakdown (Each Piece Explained)

```text
+=======================================================================+
|                    FORMULA COMPONENT BREAKDOWN                         |
|                    (WITH 80% MATURITY FACTOR)                          |
+=======================================================================+

NUMERATOR: (Current_Balance * Staking_Age)
+-----------------------------------------------------------------------+
| This calculates the total "Coin-Days" you have accumulated.           |
|                                                                       |
| Example:                                                              |
| - You have 100 tokens                                                 |
| - They are 50 days old                                                |
| - Coin-Days = 100 * 50 = 5,000                                        |
|                                                                       |
| This number represents your "Investment Weight" or "Loyalty Score"    |
+-----------------------------------------------------------------------+

DENOMINATOR: (Current_Balance + Amount_Added * 0.2)
+-----------------------------------------------------------------------+
| This is your balance PLUS 20% of new tokens (reduced dilution!)       |
|                                                                       |
| Example:                                                              |
| - Old balance: 100 tokens                                             |
| - New deposit: 100 tokens                                             |
| - Effective denominator: 100 + (100 * 0.2) = 120                     |
|                                                                       |
| NOTE: Without the 80% factor, denominator would be 200.               |
|       With 80% factor, it's only 120 - much less dilution!            |
+-----------------------------------------------------------------------+

THE DIVISION:
+-----------------------------------------------------------------------+
| We spread the existing Coin-Days across the effective balance.        |
|                                                                       |
| Example (WITH 80% Maturity Factor):                                   |
| - Coin-Days: 5,000                                                    |
| - Effective Denominator: 120                                          |
| - New Average Age: 5,000 / 120 = 41.67 days                          |
|                                                                       |
| COMPARISON:                                                           |
| - Without factor: 5,000 / 200 = 25 days (50% retention)              |
| - With 80% factor: 5,000 / 120 = 41.67 days (83% retention)          |
|                                                                       |
| The 80% factor preserves much more of your earned age!               |
+-----------------------------------------------------------------------+
```

### 6.3 Visual Representation

```text
+=======================================================================+
|                    BEFORE DEPOSIT                                      |
+=======================================================================+
     
  TOKEN PILE:  [████████████████████] 100 Tokens
  AGE:         50 Days Old
  COIN-DAYS:   100 * 50 = 5,000
  TIER:        SILVER (30-90 days)

+=======================================================================+
|                    DEPOSIT 100 NEW TOKENS                              |
+=======================================================================+

  OLD PILE:    [████████████████████] 100 Tokens @ 50 days = 5,000 coin-days
  NEW PILE:    [████████████████████] 100 Tokens @  0 days =     0 coin-days
               ─────────────────────────────────────────────────────────────
  COMBINED:    [████████████████████████████████████████] 200 Tokens
  TOTAL COIN-DAYS: 5,000 + 0 = 5,000

+=======================================================================+
|                    AFTER CALCULATION (WITH 80% FACTOR)                 |
+=======================================================================+

  EFFECTIVE DENOMINATOR: 100 + (100 * 0.2) = 120
  NEW AVERAGE AGE:       5,000 / 120 = 41.67 days
  
  TIER:        SILVER (30-90 days)  <-- STAYS IN SILVER!
  
  COMPARISON:
  ┌──────────────────┬────────────────────┬────────────────────┐
  │                  │ Without 80% Factor │ With 80% Factor    │
  ├──────────────────┼────────────────────┼────────────────────┤
  │ New Average Age  │ 25 days            │ 41.67 days         │
  │ Age Retained     │ 50%                │ 83%                │
  │ Tier After       │ BRONZE (dropped!)  │ SILVER (stays!)    │
  └──────────────────┴────────────────────┴────────────────────┘
```

### 6.4 Converting Age to Virtual Timestamp

After calculating the new age, we store it as a timestamp:

```text
  New_Staking_Time = Current_Time - New_Average_Age

  EXAMPLE (with 80% factor):
  - Current Day: Day 100
  - New Average Age: 41.67 days
  - New Virtual Start: Day 100 - 41.67 = Day 58.33

  The system now thinks this user "started staking" on Day 58.33,
  preserving much more of their earned maturity.
```

---

## 7. Interest Distribution Mechanics

### 7.1 The Distribution Trigger

An admin (or automated timer) calls `distribute_interest()` to process the accumulated penalty pool.

```text
+=======================================================================+
|                    DISTRIBUTION TRIGGER                                |
+=======================================================================+

  BEFORE DISTRIBUTION:
  +-------------------------------------------+
  |  INTEREST POOL                            |
  |  Accumulated from penalties: 10,000 GHC   |
  +-------------------------------------------+

  ADMIN CALLS:  distribute_interest()

  AFTER DISTRIBUTION:
  +-------------------------------------------+
  |  INTEREST POOL                            |
  |  Remaining: 0 GHC                         |
  +-------------------------------------------+
  
  WHERE DID THE 10,000 GHC GO? The balance of interest pool was reset to zero and indexes were updated allowing users to earn interest again based on updated indexes.
  - Bronze Index increased
  - Silver Index increased  
  - Gold Index increased
  - Diamond Index increased
```

### 7.2 The Split Process

```text
+=======================================================================+
|                    INTEREST POOL SPLITTING                             |
+=======================================================================+

                        INTEREST POOL: 10,000 GHC
                                  |
                                  v
              +-------------------------------------------+
              |           APPLY TIER WEIGHTS              |
              +-------------------------------------------+
                                  |
        +------------+------------+------------+------------+
        |            |            |            |            |
        v            v            v            v            v
  +---------+  +---------+  +---------+  +---------+
  | BRONZE  |  | SILVER  |  |  GOLD   |  | DIAMOND |
  |  20%    |  |  25%    |  |  30%    |  |  25%    |
  +---------+  +---------+  +---------+  +---------+
  |2,000 GHC|  |2,500 GHC|  |3,000 GHC|  |2,500 GHC|
  +---------+  +---------+  +---------+  +---------+
```

### 7.3 Index Update for Each Tier

After splitting, each tier's Global Reward Index is updated:

```text
+=======================================================================+
|                    INDEX UPDATE CALCULATION                            |
+=======================================================================+

FOR EACH TIER:

  Index_Increase = Tier_Pool_Amount / Tier_Total_Staked

EXAMPLE CALCULATIONS:

  BRONZE TIER:
  +----------------------------------+
  | Pool Amount: 2,000 GHC           |
  | Total Staked: 1,000,000 GHC      |
  | Index Increase: 2,000/1,000,000  |
  |               = 0.002            |
  +----------------------------------+

  SILVER TIER:
  +----------------------------------+
  | Pool Amount: 2,500 GHC           |
  | Total Staked: 500,000 GHC        |
  | Index Increase: 2,500/500,000    |
  |               = 0.005            |
  +----------------------------------+

  GOLD TIER:
  +----------------------------------+
  | Pool Amount: 3,000 GHC           |
  | Total Staked: 100,000 GHC        |
  | Index Increase: 3,000/100,000    |
  |               = 0.03             |
  +----------------------------------+

  DIAMOND TIER:
  +----------------------------------+
  | Pool Amount: 2,500 GHC           |
  | Total Staked: 10,000 GHC         |
  | Index Increase: 2,500/10,000     |
  |               = 0.25             |
  +----------------------------------+
```

---

## 8. The Global Reward Index

### 8.1 What Is It?

The Global Reward Index is a **cumulative counter** that only ever increases. It represents the total earnings-per-token since the system began.

```text
+=======================================================================+
|                    INDEX OVER TIME                                     |
+=======================================================================+

  TIME  -->  Distribution 1  -->  Distribution 2  -->  Distribution 3

  BRONZE INDEX:
  [0.000] -----> [0.002] ---------> [0.004] ---------> [0.007]
                 +0.002              +0.002              +0.003

  DIAMOND INDEX:
  [0.000] -----> [0.250] ---------> [0.480] ---------> [0.750]
                 +0.250              +0.230              +0.270

  NOTE: Diamond index grows MUCH faster because fewer tokens share the pool.
```

### 8.2 Why Use an Index Instead of Per-User Updates?

**Without Index (O(n) - Slow):**
```text
  For every distribution:
    For every user (could be millions):
      Calculate their share
      Update their balance
  
  TIME COMPLEXITY: O(n) per distribution, n is the number of users
  PROBLEM: Doesn't scale
```

**With Index (O(1) - Fast):**
```text
  For every distribution:
    Update 4 index numbers (one per tier)
  
  When user checks balance:
    Calculate interest locally using index difference
  
  TIME COMPLEXITY: O(1) per distribution
  ADVANTAGE: Scales to millions of users
```

---

## 9. User Interest Calculation

### 9.1 The "Lazy Evaluation" Approach

User interest is NOT calculated during distribution. It is calculated **on-demand** when the user:
- Views their profile
- Claims rewards
- Stakes/Unstakes
- Submits a quiz

### 9.2 The Calculation Formula

```text
FORMULA:

  My_Interest = My_Balance * (Current_Tier_Index - My_Last_Recorded_Index)

WHERE:
- My_Balance: How many tokens I have staked
- Current_Tier_Index: The global index for my current tier (from staking_hub)
- My_Last_Recorded_Index: Snapshot of the index when I last calculated
```

### 9.3 Step-by-Step Calculation

```text
+=======================================================================+
|                    USER INTEREST CALCULATION                           |
+=======================================================================+

STEP 1: DETERMINE USER'S TIER
+-----------------------------------------------------------------------+
| Calculate: Staking_Age = Current_Time - Staking_Time            |
|                                                                       |
| Example:                                                              |
| - Staking_Time: Day 0                                           |
| - Current_Time: Day 400                                               |
| - Staking_Age: 400 days                                               |
| - Tier: DIAMOND (365+ days)                                           |
+-----------------------------------------------------------------------+

STEP 2: FETCH CURRENT GLOBAL INDEX FOR THAT TIER
+-----------------------------------------------------------------------+
| The staking_hub provides the current Diamond Index.                   |
|                                                                       |
| Example:                                                              |
| - Current_Diamond_Index: 0.750                                        |
+-----------------------------------------------------------------------+

STEP 3: CALCULATE INDEX DIFFERENCE
+-----------------------------------------------------------------------+
| Index_Diff = Current_Index - User_Last_Recorded_Index                 |
|                                                                       |
| Example:                                                              |
| - Current_Index: 0.750                                                |
| - User's Last Index: 0.500 (from when they last checked)              |
| - Index_Diff: 0.750 - 0.500 = 0.250                                   |
+-----------------------------------------------------------------------+

STEP 4: CALCULATE INTEREST EARNED
+-----------------------------------------------------------------------+
| Interest = User_Balance * Index_Diff                                  |
|                                                                       |
| Example:                                                              |
| - User Balance: 1,000 GHC                                             |
| - Index_Diff: 0.250                                                   |
| - Interest: 1,000 * 0.250 = 250 GHC                                   |
+-----------------------------------------------------------------------+

STEP 5: UPDATE USER'S SNAPSHOT
+-----------------------------------------------------------------------+
| User's Last Index = Current Index                                     |
|                                                                       |
| So next time, they only earn from NEW index growth.                   |
+-----------------------------------------------------------------------+
```

---

## 10. Complete Flow Diagrams

### 10.1 End-to-End Flow: From Penalty to Interest

```text
+=======================================================================+
|                    COMPLETE INTEREST LIFECYCLE                         |
+=======================================================================+

PHASE 1: REVENUE GENERATION
============================

  User A decides to unstake 10,000 GHC
                |
                v
  +---------------------------+
  |  STAKING HUB PROCESSES    |
  |  UNSTAKE REQUEST          |
  +---------------------------+
                |
                v
  +---------------------------+
  |  APPLY 10% PENALTY        |
  |  Penalty = 1,000 GHC      |
  +---------------------------+
                |
         +------+------+
         |             |
         v             v
  +-----------+  +-----------+
  | Return    |  | Add to    |
  | 9,000 GHC |  | Interest  |
  | to User A |  | Pool      |
  +-----------+  +-----------+


PHASE 2: INTEREST POOL ACCUMULATION
====================================

  INTEREST POOL
  +-------------------------------------------+
  | Previous Balance:        5,000 GHC        |
  | + Penalty from User A:   1,000 GHC        |
  | + Penalty from User B:     500 GHC        |
  | + Penalty from User C:     800 GHC        |
  +-------------------------------------------+
  | Current Balance:         7,300 GHC        |
  +-------------------------------------------+
  
  (Pool accumulates until distribution is triggered)


PHASE 3: DISTRIBUTION
======================

  Admin calls distribute_interest()
                |
                v
  +-------------------------------------------+
  |  SPLIT 7,300 GHC BY TIER WEIGHTS          |
  +-------------------------------------------+
                |
  +-------------+-------------+-------------+-------------+
  |             |             |             |             |
  v             v             v             v
+---------+  +---------+  +---------+  +---------+
| BRONZE  |  | SILVER  |  |  GOLD   |  | DIAMOND |
| 1,460   |  | 1,825   |  | 2,190   |  | 1,825   |
| GHC     |  | GHC     |  | GHC     |  | GHC     |
+---------+  +---------+  +---------+  +---------+
     |            |            |            |
     v            v            v            v
  Update      Update       Update       Update
  Bronze      Silver       Gold         Diamond
  Index       Index        Index        Index


PHASE 4: USER CLAIMS INTEREST
==============================

  User B (Diamond, 1,000 GHC staked) views profile
                |
                v
  +-------------------------------------------+
  |  1. Calculate Staking Age: 400 days       |
  |  2. Determine Tier: DIAMOND               |
  |  3. Fetch Diamond Index: 0.750            |
  |  4. Get User's Last Index: 0.500          |
  |  5. Index Diff: 0.250                     |
  |  6. Interest: 1,000 * 0.250 = 250 GHC     |
  +-------------------------------------------+
                |
                v
  +-------------------------------------------+
  |  User's unclaimed_interest += 250 GHC     |
  |  User's last_index = 0.750                |
  +-------------------------------------------+
                |
                v
  User clicks "Claim Rewards"
                |
                v
  +-------------------------------------------+
  |  staked_balance += unclaimed_interest     |
  |  unclaimed_interest = 0                   |
  +-------------------------------------------+
```

### 10.2 Tier Upgrade Flow

```text
+=======================================================================+
|                    TIER UPGRADE PROCESS                                |
+=======================================================================+

USER STATUS:
- Staking Age: 29 days (BRONZE)
- Balance: 1,000 GHC
- Last Bronze Index: 0.100
                |
                | (1 day passes)
                v
USER STATUS:
- Staking Age: 30 days (qualifies for SILVER!)
                |
                v
UPGRADE DETECTED ON NEXT INTERACTION:
+---------------------------------------------------+
|  STEP 1: CLAIM PENDING BRONZE INTEREST            |
|  - Current Bronze Index: 0.105                    |
|  - User's Last Index: 0.100                       |
|  - Diff: 0.005                                    |
|  - Interest: 1,000 * 0.005 = 5 GHC                |
|  - Add to unclaimed_interest: +5 GHC              |
+---------------------------------------------------+
                |
                v
+---------------------------------------------------+
|  STEP 2: MOVE TO SILVER TIER                      |
|  - Set current_tier = SILVER                      |
|  - Snapshot Silver Index for future calculations  |
|  - User's Last_Silver_Index = Current Silver Index|
+---------------------------------------------------+
                |
                v
+---------------------------------------------------+
|  STEP 3: UPDATE GLOBAL TIER POPULATIONS           |
|  - Bronze total staked: -1,000 GHC                |
|  - Silver total staked: +1,000 GHC                |
+---------------------------------------------------+
```

### 10.3 Weighted Age Update on Deposit

```text
+=======================================================================+
|                    AGE RECALCULATION ON DEPOSIT                        |
|                    (WITH 80% MATURITY FACTOR)                          |
+=======================================================================+

BEFORE DEPOSIT:
+-------------------------------------------+
|  User Status:                             |
|  - Balance: 500 GHC                       |
|  - Staking_Time: Day 0                    |
|  - Current Day: Day 100                   |
|  - Staking Age: 100 days                  |
|  - Current Tier: GOLD (90-365)            |
+-------------------------------------------+

USER EARNS 5 GHC FROM QUIZ:
+-------------------------------------------+
|  Amount_Added: 5 GHC                      |
|  Age of New Tokens: 0 days                |
|  Maturity Factor: 0.8 (80%)               |
+-------------------------------------------+

CALCULATION (WITH 80% FACTOR):
+-------------------------------------------+
|  Old Coin-Days: 500 * 100 = 50,000        |
|  Effective Denominator: 500 + (5 * 0.2)   |
|                       = 500 + 1 = 501     |
|  New Average Age: 50,000 / 501 = 99.80    |
+-------------------------------------------+

UPDATE VIRTUAL TIMESTAMP:
+-------------------------------------------+
|  New_Staking_Time = 100 - 99.80           |
|                   = Day 0.20              |
|                   ~ Day 0.2               |
+-------------------------------------------+

AFTER DEPOSIT:
+-------------------------------------------+
|  User Status:                             |
|  - Balance: 505 GHC                       |
|  - Staking_Time: Day 0.2 (barely moved!)  |
|  - Current Day: Day 100                   |
|  - Staking Age: 99.8 days                 |
|  - Current Tier: GOLD (still!)            |
+-------------------------------------------+

IMPACT: Minimal! Age only dropped by 0.2 days (vs 1 day without factor).
```

---

## 11. Edge Cases and Examples

### 11.1 Small Daily Earnings (Normal Case)

**Scenario:** User with large mature stack earns 5 tokens daily.

```text
  Before: 2,000 GHC @ 400 days (Diamond)
  Earn: 5 GHC
  
  Calculation (WITH 80% MATURITY FACTOR):
  - Old Coin-Days: 2,000 * 400 = 800,000
  - Effective Denominator: 2,000 + (5 * 0.2) = 2,001
  - New Age: 800,000 / 2,001 = 399.80 days
  
  Result: Still Diamond (dropped by only 0.2 days!)
  
  COMPARISON:
  - Without 80% factor: 800,000 / 2,005 = 399.00 days (dropped 1 day)
  - With 80% factor: 800,000 / 2,001 = 399.80 days (dropped 0.2 days)
```

### 11.2 Consistent Daily Earnings (Realistic Case)

**Scenario:** User earns 5 tokens every day for a full year while already having a mature stack.

```text
  Starting Point: 500 GHC @ 365 days (Diamond)
  Daily Earnings: 5 GHC per day for 365 days = 1,825 GHC total added
  
  DAY 1 (WITH 80% MATURITY FACTOR):
  - Old Coin-Days: 500 * 365 = 182,500
  - Effective Denominator: 500 + (5 * 0.2) = 501
  - New Age: 182,500 / 501 = 364.27 days (still Diamond!)
  
  DAY 100:
  - Balance has grown to ~1,000 GHC
  - Age has stabilized around 360+ days
  - Still Diamond
  
  DAY 365 (End of Year):
  - Balance: ~2,325 GHC
  - Age: Still well above 365 days
  
  Result: User REMAINS Diamond throughout the year.
  
  WHY: With the 80% factor, each daily 5-token addition causes
       minimal dilution. The natural passage of time (1 day)
       more than offsets the reduced dilution effect.
```

### 11.3 New User Building Up (Starting from Zero)

**Scenario:** Brand new user earns 5 tokens daily. When do they reach each tier?

```text
  WITH 80% MATURITY FACTOR:
  
  DAY 1: 5 GHC @ 0 days = BRONZE
  DAY 30: ~150 GHC @ weighted age ~24 days = BRONZE
  
  WHY 24 DAYS (NOT 30 OR 15)?
  - Every day's 5 tokens are "age 0" when earned
  - Day 1 tokens are 30 days old
  - But with 80% factor, dilution is reduced by 80%
  - Effective age is ~80% of elapsed time
  
  TIER PROGRESSION WITH 80% FACTOR:
  ┌─────────────────────┬──────────────────┬──────────────────┐
  │ Tier                │ Without Factor   │ With 80% Factor  │
  ├─────────────────────┼──────────────────┼──────────────────┤
  │ SILVER (~30 days)   │ ~Day 60          │ ~Day 38          │
  │ GOLD (~90 days)     │ ~Day 180         │ ~Day 112         │
  │ DIAMOND (~365 days) │ ~Day 730         │ ~Day 456         │
  └─────────────────────┴──────────────────┴──────────────────┘
  
  BENEFIT: The 80% factor means users reach higher tiers in ~1.25x
           the actual time, instead of ~2x the time.
           
  This rewards consistent daily participation MORE FAIRLY while still
  ensuring genuine time commitment is required for higher tiers.
```
```

### 11.3 Unstaking Does NOT Change Age

**Scenario:** User unstakes some tokens.

```text
  Before: 1,000 GHC @ 200 days (Gold)
  Unstake: 500 GHC
  
  Age Calculation: UNCHANGED
  - Staking_Time stays the same
  - Age is still 200 days
  
  Result: Still Gold with 500 GHC @ 200 days
  
  WHY: Removing tokens doesn't add "new" age-0 tokens.
       The remaining tokens keep their maturity.
```

### 11.4 Empty Tier Edge Case

**Scenario:** What if nobody is in the Diamond tier?

```text
  Distribution: 10,000 GHC
  Diamond Share: 2,500 GHC
  Diamond Total Staked: 0 GHC
  
  Problem: Division by zero!
  
  Solution: Skip index update for empty tiers.
            The 2,500 GHC stays in the Interest Pool
            OR is redistributed to other tiers (implementation choice).
```

---

## 12. Implementation Reference

### 12.1 Data Structures

**staking_hub - Global State:**
```text
GlobalStats {
    total_staked: u64,              // Sum of all staked tokens
    interest_pool: u64,             // Accumulated penalties waiting to distribute
    tier_staked: [u64; 4],          // [Bronze, Silver, Gold, Diamond] totals
    tier_reward_indexes: [u128; 4], // Cumulative index per tier (scaled by 1e18)
}
```

**user_profile - User State:**
```text
UserProfile {
    staked_balance: u64,            // User's staked tokens
    unclaimed_interest: u64,        // Pending rewards to claim
    current_tier: u8,               // 0=Bronze, 1=Silver, 2=Gold, 3=Diamond
    tier_start_index: u128,         // Index snapshot when entered current tier
    staking_time: u64,        // Virtual timestamp for age calculation
}
```

### 12.2 Key Functions

**On Deposit (Earning/Staking):**
1. Calculate new weighted average age
2. Update `staking_time` (virtual timestamp)
3. Check for tier upgrade
4. Update balance

**On Distribution:**
1. Split `interest_pool` by tier weights
2. For each non-empty tier: `tier_index += tier_share / tier_staked`
3. Reset `interest_pool` to 0

**On Profile View / Claim:**
1. Calculate current staking age
2. Determine current tier
3. If tier changed: claim old tier interest, move to new tier
4. Calculate interest: `balance * (current_index - last_index)`
5. Update user's `tier_start_index`

---

## Summary

The GHC Interest System achieves fair, scalable, and sustainable reward distribution through:

1. **Zero-Sum Model:** No inflation; all yields come from penalties
2. **Tier System:** 4 pools with different populations create natural yield multipliers
3. **Weighted Average Age:** Single timestamp tracks maturity across multiple deposits
4. **Global Reward Index:** O(1) distribution scales to millions of users
5. **Lazy Evaluation:** Interest calculated on-demand, not per-block

This design ensures that loyal, long-term stakers are rewarded with significantly higher yields without creating unsustainability or token inflation.
