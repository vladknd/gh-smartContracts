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

**Definition:** The amount of time (in days) that has passed since a user began staking.

**Formula:**
```text
Staking_Age = Current_Time - Staking_Time
```

**Where:**
- **Current_Time:** The current global system time (e.g., `ic_cdk::time`).
- **Staking_Time:** The virtual (averaged) timestamp representing the duration of the user's staking. This value is adjusted when new tokens are added (see Weighted Average Age).


### 2.2 Weighted Average Age

**Definition:** A single number representing the "effective maturity" of a user's entire token balance, accounting for tokens deposited at different times.

**Why Needed:** Users earn tokens daily from quizzes. Each batch of tokens has a different "birthday." Instead of tracking thousands of individual deposits, we calculate one weighted average.

**Formula:**
```text
                    (staked_balance * Old_Age) + (New_Tokens * New_Age)
Weighted_Avg_Age = --------------------------------------------------
                              staked_balance + New_Tokens
```




Since new tokens always have Age = 0:

User mines 5 coins (New_Tokens), before he had 5 coins staked.
```text
                    (staked_balance * Staking_Age)
Weighted_Avg_Age = --------------------------
                    staked_balance + New_Tokens


staking_time = Current_Time - Weighted_Avg_Age
```

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
```

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
  - staking_time is set to NOW (Day 1)

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 5 GHC              │ ← STORED
  │ staking_time       = Day 1              │ ← STORED
  └─────────────────────────────────────────┘

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

STEP 3: Calculate New Total Balance (denominator)
  New_Balance = staked_balance + New_Tokens
              = 5 + 5
              = 10 GHC

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = Coin_Days / New_Balance
                   = 5 / 10
                   = 0.5 days

STEP 5: Convert Age back to Timestamp
  new_staking_time = Current_Time - Weighted_Avg_Age
                         = Day 2 - 0.5
                         = Day 1.5

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 10 GHC             │ ← STORED
  │ staking_time       = Day 1.5            │ ← STORED (drifted +0.5)
  └─────────────────────────────────────────┘

+=======================================================================+
|                 DAY 3: THIRD DEPOSIT                                  |
+=======================================================================+

BEFORE (from user_profile canister):
  staked_balance     = 10 GHC
  staking_time       = Day 1.5

EVENT: User earns 5 GHC from today's quiz
  Current_Time       = Day 3
  New_Tokens         = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 3 - Day 1.5 = 1.5 days

STEP 2: Calculate Coin-Days
  Coin_Days = 10 * 1.5 = 15 coin-days

STEP 3: Calculate New Total Balance
  New_Balance = 10 + 5 = 15 GHC

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 15 / 15 = 1.0 day

STEP 5: Convert to Timestamp
  new_staking_time = Day 3 - 1.0 = Day 2

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 15 GHC             │ ← STORED
  │ staking_time       = Day 2              │ ← STORED (drifted +0.5)
  └─────────────────────────────────────────┘

+=======================================================================+
|                 DAY 10: PATTERN CONTINUES                             |
+=======================================================================+

Let's fast-forward to see the accumulated effect:

BEFORE (from user_profile canister):
  staked_balance     = 45 GHC    (9 days × 5 GHC)
  staking_time = Day 5.5   (drifted from Day 1)

EVENT: User earns 5 GHC from today's quiz
  Current_Time = Day 10
  New_Tokens   = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 10 - Day 5.5 = 4.5 days

STEP 2: Calculate Coin-Days
  Coin_Days = 45 * 4.5 = 202.5 coin-days

STEP 3: Calculate New Total Balance
  New_Balance = 45 + 5 = 50 GHC

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 202.5 / 50 = 4.05 days

STEP 5: Convert to Timestamp
  new_staking_time = Day 10 - 4.05 = Day 5.95

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 50 GHC             │ ← STORED
  │ staking_time = Day 5.95           │ ← STORED (drifted +0.45)
  └─────────────────────────────────────────┘

+=======================================================================+
|                 DAY 100: LONG-TERM VIEW                               |
+=======================================================================+

After 100 daily deposits of 5 GHC each:

BEFORE (from user_profile canister):
  staked_balance     = 495 GHC
  staking_time = Day 50.25 (drifted from Day 1)

EVENT: User earns 5 GHC from today's quiz
  Current_Time = Day 100
  New_Tokens   = 5 GHC

STEP 1: Calculate Staking_Age
  Staking_Age = Day 100 - Day 50.25 = 49.75 days

STEP 2: Calculate Coin-Days
  Coin_Days = 495 * 49.75 = 24,626.25 coin-days

STEP 3: Calculate New Total Balance
  New_Balance = 495 + 5 = 500 GHC

STEP 4: Calculate Weighted Average Age
  Weighted_Avg_Age = 24,626.25 / 500 = 49.25 days

STEP 5: Convert to Timestamp
  new_staking_time = Day 100 - 49.25 = Day 50.75

AFTER UPDATE (written to user_profile canister):
  ┌─────────────────────────────────────────┐
  │ staked_balance     = 500 GHC            │ ← STORED
  │ staking_time = Day 50.75          │ ← STORED (drifted +0.50)
  └─────────────────────────────────────────┘

+=======================================================================+
|                 OBSERVATION                                           |
+=======================================================================+

Even after 100 days of continuous daily deposits:
- The effective staking age is ~49.25 days (almost exactly half!)
- Each new deposit only reduces the age by ~0.5 days
- The user's tier: SILVER (30-90 days) ← Not Bronze!

This shows why the weighted average age is important:
Without it, adding ANY new tokens would either:
- Reset age to 0 (unfair to loyal users)
- Keep age unchanged (unfair to the system)

The weighted average strikes a balance that rewards consistent participation.
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
│  1. TIER TRANSITIONS PER BATCH                                       │
│     Each deposit ages independently. On Day 100:                     │
│     - Deposit from Day 1 is 99 days old (GOLD tier)                 │
│     - Deposit from Day 70 is 30 days old (SILVER tier)              │
│     - Deposit from Day 100 is 0 days old (BRONZE tier)              │
│                                                                      │
│     You'd need to track SEPARATE tier indexes for EACH deposit,     │
│     and update them whenever a deposit crosses a tier boundary.      │
│                                                                      │
│  2. INTER-CANISTER CALL EXPLOSION                                    │
│     Each deposit may need its own sync with staking_hub to:         │
│     - Register in the correct tier's total_staked                   │
│     - Track its own reward index snapshot                           │
│                                                                      │
│     365 deposits = 365 potential sync operations per year!          │
│                                                                      │
│  3. UNSTAKING NIGHTMARE                                              │
│     When user unstakes, which deposits do you remove first?         │
│     FIFO? LIFO? Oldest (highest tier) first? Newest first?          │
│     Each choice has different economic implications.                 │
└──────────────────────────────────────────────────────────────────────┘

COMPARISON SUMMARY:
┌────────────────────┬──────────────────────┬──────────────────────────┐
│ Metric             │ Weighted Average     │ Per-Batch Tracking       │
├────────────────────┼──────────────────────┼──────────────────────────┤
│ Storage per user   │ O(1) - 16 bytes      │ O(n) - 16 bytes × days   │
│ Interest calc time │ O(1) - constant      │ O(n) - linear in deposits│
│ Tier tracking      │ 1 tier per user      │ n tiers (1 per deposit)  │
│ Sync operations    │ 1 per deposit event  │ n per interest calc      │
│ Code complexity    │ Simple               │ Very complex             │
│ Unstaking logic    │ Single deduction     │ Multi-record management  │
└────────────────────┴──────────────────────┴──────────────────────────┘

CONCLUSION:
The weighted average approach trades a small amount of precision
(~0.5 day drift per deposit) for massive gains in:
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

When a user deposits new tokens, we recalculate their "Virtual Start Date":

```text
FORMULA:

                     (Current_Balance * Staking_Age)
New_Average_Age  =  ----------------------------------
                     (Current_Balance + Amount_Added)

WHERE:
- Current_Balance: Tokens already staked
- Staking_Age: Days since staking_time (calculated as Now - Staking_Time)
- Amount_Added: New tokens being deposited (always Age = 0)
```

### 6.2 Formula Breakdown (Each Piece Explained)

```text
+=======================================================================+
|                    FORMULA COMPONENT BREAKDOWN                         |
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

DENOMINATOR: (Current_Balance + Amount_Added)
+-----------------------------------------------------------------------+
| This is simply your NEW total balance after the deposit.              |
|                                                                       |
| Example:                                                              |
| - Old balance: 100 tokens                                             |
| - New deposit: 100 tokens                                             |
| - New total: 200 tokens                                               |
+-----------------------------------------------------------------------+

THE DIVISION:
+-----------------------------------------------------------------------+
| We spread the existing Coin-Days across the new larger balance.       |
|                                                                       |
| Example:                                                              |
| - Coin-Days: 5,000                                                    |
| - New Balance: 200                                                    |
| - New Average Age: 5,000 / 200 = 25 days                              |
|                                                                       |
| Your effective age DROPPED from 50 days to 25 days because you        |
| added tokens that are 0 days old.                                     |
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
|                    AFTER CALCULATION                                   |
+=======================================================================+

  NEW AVERAGE AGE:  5,000 / 200 = 25 days
  
  TIER:        BRONZE (0-30 days)  <-- DROPPED FROM SILVER!
```

### 6.4 Converting Age to Virtual Timestamp

After calculating the new age, we store it as a timestamp:

```text
  New_Staking_Time = Current_Time - New_Average_Age

  EXAMPLE:
  - Current Day: Day 100
  - New Average Age: 25 days
  - New Virtual Start: Day 100 - 25 = Day 75

  The system now thinks this user "started staking" on Day 75,
  even though they actually started on Day 0.
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
+=======================================================================+

BEFORE DEPOSIT:
+-------------------------------------------+
|  User Status:                             |
|  - Balance: 500 GHC                       |
|  - Staking_Time: Day 0              |
|  - Current Day: Day 100                   |
|  - Staking Age: 100 days                  |
|  - Current Tier: GOLD (90-365)            |
+-------------------------------------------+

USER EARNS 5 GHC FROM QUIZ:
+-------------------------------------------+
|  Amount_Added: 5 GHC                      |
|  Age of New Tokens: 0 days                |
+-------------------------------------------+

CALCULATION:
+-------------------------------------------+
|  Old Coin-Days: 500 * 100 = 50,000        |
|  New Coin-Days: 5 * 0 = 0                 |
|  Total Coin-Days: 50,000                  |
|  New Balance: 500 + 5 = 505               |
|  New Average Age: 50,000 / 505 = 99.01    |
+-------------------------------------------+

UPDATE VIRTUAL TIMESTAMP:
+-------------------------------------------+
|  New_Staking_Time = 100 - 99.01     |
|                         = Day 0.99        |
|                         ~ Day 1           |
+-------------------------------------------+

AFTER DEPOSIT:
+-------------------------------------------+
|  User Status:                             |
|  - Balance: 505 GHC                       |
|  - Staking_Time: Day 1 (shifted!)   |
|  - Current Day: Day 100                   |
|  - Staking Age: 99 days                   |
|  - Current Tier: GOLD (still!)            |
+-------------------------------------------+

IMPACT: Minimal! Age only dropped by 1 day.
```

---

## 11. Edge Cases and Examples

### 11.1 Small Daily Earnings (Normal Case)

**Scenario:** User with large mature stack earns 5 tokens daily.

```text
  Before: 2,000 GHC @ 400 days (Diamond)
  Earn: 5 GHC
  
  Calculation:
  - Old Coin-Days: 2,000 * 400 = 800,000
  - New Balance: 2,005
  - New Age: 800,000 / 2,005 = 399.00 days
  
  Result: Still Diamond (dropped by only 1 day)
```

### 11.2 Consistent Daily Earnings (Realistic Case)

**Scenario:** User earns 5 tokens every day for a full year while already having a mature stack.

```text
  Starting Point: 500 GHC @ 365 days (Diamond)
  Daily Earnings: 5 GHC per day for 365 days = 1,825 GHC total added
  
  DAY 1:
  - Old Coin-Days: 500 * 365 = 182,500
  - New Balance: 505
  - New Age: 182,500 / 505 = 361.4 days (still Diamond)
  
  DAY 100:
  - Balance has grown to ~1,000 GHC
  - Age has stabilized around 350+ days
  - Still Diamond
  
  DAY 365 (End of Year):
  - Balance: ~2,325 GHC
  - Age: Still well above 365 days
  
  Result: User REMAINS Diamond throughout the year.
  
  WHY: Each daily 5-token addition is so small relative to the
       existing balance that the age dilution is negligible.
       The natural passage of time (1 day) offsets the dilution.
```

### 11.3 New User Building Up (Starting from Zero)

**Scenario:** Brand new user earns 5 tokens daily. When do they reach each tier?

```text
  DAY 1: 5 GHC @ 0 days = BRONZE
  DAY 30: ~150 GHC @ weighted age ~15 days = BRONZE
  
  WHY NOT 30 DAYS OLD?
  - Every day's 5 tokens are "age 0" when earned
  - Day 1 tokens are 30 days old
  - Day 30 tokens are 0 days old
  - Weighted average is roughly half: ~15 days
  
  ACTUAL TIER PROGRESSION (Approximate):
  - Reach SILVER (~30 day avg): Around Day 60
  - Reach GOLD (~90 day avg): Around Day 180
  - Reach DIAMOND (~365 day avg): Around Day 730 (2 years)
  
  IMPORTANT: Because new tokens constantly dilute the average,
             it takes roughly TWICE as long to reach each tier
             compared to a single-deposit scenario.
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
