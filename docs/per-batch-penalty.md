# Per-Batch vs Weighted Average: Penalty Calculation Analysis

> **Status:** Architecture Decision Record  
> **Last Updated:** December 2025  
> **Related:** [ADVANCED_INTEREST_FLOW.md](./ADVANCED_INTEREST_FLOW.md), [STAKING_MECHANICS.md](./STAKING_MECHANICS.md)

---

## Table of Contents

1. [Overview](#1-overview)
2. [The Two Approaches](#2-the-two-approaches)
3. [Scalability Problems](#3-scalability-problems)
4. [Security Problems](#4-security-problems)
5. [Reliability Problems](#5-reliability-problems)
6. [Comparison Summary](#6-comparison-summary)
7. [Recommendation](#7-recommendation)

---

## 1. Overview

This document analyzes the trade-offs between two approaches for calculating **variable unstaking penalties** based on staking duration:

1. **Per-Batch Tracking:** Store each deposit separately and calculate penalties per-deposit
2. **Weighted Average:** Maintain a single "virtual timestamp" representing the average age of all tokens

This analysis is particularly relevant for systems where:
- Interest distribution does NOT depend on tiers (everyone shares equally)
- But unstaking penalties DO vary based on how long tokens were staked

---

## 2. The Two Approaches

### 2.1 Penalty Tiers (Example Configuration)

```text
┌─────────────────────┬────────────────┐
│ Staking Duration    │ Penalty Rate   │
├─────────────────────┼────────────────┤
│ 0 - 30 days         │ 15%            │
│ 30 - 90 days        │ 10%            │
│ 90 - 365 days       │ 5%             │
│ 365+ days           │ 2%             │
└─────────────────────┴────────────────┘
```

### 2.2 Per-Batch Approach

Store every deposit as a separate record:

```rust
struct UserDeposits {
    deposits: Vec<Deposit>,  // Grows with every deposit
}

struct Deposit {
    amount: u64,
    timestamp: u64,
}
```

**On unstake:** Iterate through all deposits, calculate penalty for each based on its individual age.

### 2.3 Weighted Average Approach

Store only two fields:

```rust
struct UserStake {
    staked_balance: u64,      // Total tokens
    staking_time: u64,  // Weighted average timestamp
}
```

**On unstake:** Calculate age from the single timestamp, apply single penalty rate to entire unstake amount.

---

## 3. Scalability Problems

### 3.1 Storage Growth (Linear, Unbounded)

```text
+=======================================================================+
|                    STORAGE EXPLOSION                                   |
+=======================================================================+

ICP Canister Limits:
- Stable memory: 400 GB max (but costly at ~$5/GB/year)
- Heap memory: 4 GB max

Per-Batch Storage:
- 1 user × 5 years × 365 days = 1,825 records
- 1M users × 1,825 records × ~24 bytes = 43.8 GB

PROBLEM: You'll exhaust heap quickly, forcing complex stable
memory management or sharding by DEPOSITS (not just users).

Weighted Average Storage:
- 1M users × 16 bytes = 16 MB

DIFFERENCE: 2,700x more storage required for per-batch
```

### 3.2 Instruction Limits Per Call

```text
+=======================================================================+
|                    INSTRUCTION LIMITS                                  |
+=======================================================================+

ICP Limit: ~2 billion instructions per update call

Unstake operation with per-batch:
- Must iterate through all deposits to calculate total penalty
- Must decide which deposits to remove (FIFO/LIFO/etc.)
- Must update remaining deposits

For a power user with 5,000+ deposits (10+ years):
- Each deposit: ~1,000 instructions for penalty calc
- Total: 5,000,000 instructions just for iteration
- Add serialization/deserialization overhead

RISK: Heavy users could hit instruction limits on unstake

Weighted Average:
- Fixed ~100 instructions regardless of staking history
```

### 3.3 Upgrade Performance

```text
+=======================================================================+
|                    UPGRADE TIMEOUT RISK                                |
+=======================================================================+

Pre-upgrade hook must serialize all data to stable memory
Post-upgrade hook must deserialize back

With 1M users:
- Weighted: ~32 MB to serialize (seconds)
- Per-Batch: ~43 GB to serialize (could TIMEOUT)

RISK: Canister upgrades could fail, requiring complex chunked
migration strategies.

ICP upgrade timeout: ~30 seconds
Serialization speed: ~100 MB/s (optimistic)
Per-Batch time: 43 GB / 100 MB/s = 430 seconds = FAILURE
```

---

## 4. Security Problems

### 4.2 Cherry-Picking / Gaming the Penalty

```text
+=======================================================================+
|                    PENALTY GAMING                                      |
+=======================================================================+

If users can choose WHICH deposits to unstake:

OPTIMAL STRATEGY:
1. Unstake oldest deposits first (lowest penalty)
2. Leave newest deposits staked
3. Re-stake the withdrawn tokens immediately
4. Wait, then repeat

RESULT: Users systematically pay minimum penalties while
maintaining effective liquidity. Defeats penalty mechanism.

If system forces LIFO (newest first):
- Users game by depositing large amounts right before unstaking
- These new deposits "shield" old deposits

If system forces FIFO (oldest first):
- Users might prefer this anyway (lowest penalty)
- Still gameable with timed deposits

ANY deterministic order can be gamed once known.

Weighted Average:
- All tokens treated as single pool
- No cherry-picking possible
- Penalty reflects true average behavior
```

### 4.3 Integer Overflow / Precision Loss

```text
+=======================================================================+
|                    PRECISION ISSUES                                    |
+=======================================================================+

Summing thousands of small penalty calculations:

for deposit in deposits:
    penalty += deposit.amount * penalty_rate

RISKS:
- Floating point: Accumulated rounding errors
- Fixed point: Need careful decimal handling
- Integer: Potential overflow if not using u128/checked ops

Example:
- 10,000 deposits of 1 token each
- 10,000 multiplications and additions
- Each with potential rounding error
- Errors can compound

Weighted Average:
- Single multiplication: total_balance * penalty_rate
- No accumulation, no compounding errors
```

---

## 5. Reliability Problems

### 5.1 Atomicity of Multi-Record Updates

```text
+=======================================================================+
|                    ATOMICITY ISSUES                                    |
+=======================================================================+

When unstaking with per-batch:
- Calculate penalty for each deposit
- Remove/reduce multiple deposit records
- Update total balance
- Transfer tokens

FAILURE SCENARIOS:
- Trap after removing 50/100 deposits → partial state
- Token transfer fails after deposit removal → lost tokens
- Canister upgrade mid-operation → corrupted state

MITIGATION: Transactional patterns, but ICP doesn't have
automatic rollback. Requires careful manual state management.

Weighted Average:
- Update 2 fields (balance, timestamp)
- Single atomic operation
- Easy to verify success/failure
```

### 5.2 Data Integrity Across Records

```text
+=======================================================================+
|                    DATA INTEGRITY                                      |
+=======================================================================+

INVARIANT: sum(all deposits) MUST equal staked_balance

With 1,825 records per user, bugs can cause:
- Duplicate records (counted twice)
- Orphaned records (deposit exists but no corresponding balance)
- Negative amounts (if subtraction bugs)
- Timestamp ordering violations

DETECTION: Would need periodic reconciliation jobs
RECOVERY: Complex logic to identify and fix inconsistencies

Example Bug Scenario:
1. User has 100 deposits totaling 500 GHC
2. Bug causes deposit #47 to be duplicated
3. Sum of deposits = 505 GHC, but staked_balance = 500 GHC
4. User can claim 5 GHC they don't own

Weighted Average:
- Only 2 fields: balance and timestamp
- Trivial to validate: balance >= 0, timestamp <= now
- No cross-record invariants to maintain
```

### 5.3 Query Performance Degradation

```text
+=======================================================================+
|                    QUERY PERFORMANCE                                   |
+=======================================================================+

User wants to view their staking dashboard:

PER-BATCH:
- Fetch all 1,825 deposits
- Calculate current age for each
- Group by penalty bracket
- Calculate projected penalties for each bracket
- Return aggregated view

Query time grows with deposit count → poor UX for long-term users

WEIGHTED AVERAGE:
- Fetch 2 fields
- Calculate single age
- Return one penalty rate

Constant time regardless of how long user has been staking.

IMPACT ON FRONTEND:
- Per-Batch: Loading spinner for loyal users
- Weighted: Instant response for everyone
```

### 5.4 Migration Complexity

```text
+=======================================================================+
|                    SCHEMA MIGRATION                                    |
+=======================================================================+

If you need to change deposit record structure later:

WEIGHTED AVERAGE: Migrate 2 fields per user
PER-BATCH: Migrate 1,825 records per user

Schema change example: Add "source" field to track quiz vs reward

PER-BATCH MIGRATION:
- 1M users × 1,825 records = 1.825 billion records to touch
- Must be done in chunks across multiple upgrade cycles
- Dual-format support during transition
- Risk of data loss if migration fails mid-way
- Estimated time: weeks of careful deployment

WEIGHTED AVERAGE MIGRATION:
- 1M users × 2 fields = 2 million fields
- Single upgrade cycle
- Estimated time: seconds
```

---

## 6. Comparison Summary

| Problem Area | Per-Batch | Weighted Average |
|--------------|-----------|------------------|
| **Storage per user** | O(n) - 43.8 GB for 1M users | O(1) - 16 MB for 1M users |
| **Computation per unstake** | O(n) - can hit limits | O(1) - always fast |
| **Upgrade performance** | Risk of timeout | Fast and safe |
| **DoS vulnerability** | High (spam deposits) | None |
| **Gaming potential** | High (cherry-picking) | Low (single pool) |
| **Precision** | Accumulation errors | Single calculation |
| **Atomicity** | Multi-record updates | 2-field update |
| **Data integrity** | Complex invariants | Trivial validation |
| **Query performance** | Degrades over time | Constant |
| **Migration effort** | Billions of records | Simple field update |

---

## 7. Recommendation

### 7.1 Use Weighted Average Unless...

The weighted average approach is the recommended choice for virtually all staking systems. Use per-batch tracking **only if** your business requirements explicitly demand:

1. Users must be able to **selectively unstake specific deposits**
2. Regulatory requirements mandate **per-deposit audit trails**
3. Different deposit sources have **fundamentally different rules** (unlikely)

### 7.2 The Trade-Off

The only downside of weighted average is **precision loss**:

```text
PRECISION LOSS EXAMPLE:

User deposits 100 GHC on Day 1, 100 GHC on Day 99

PER-BATCH (Day 100):
- 100 GHC @ 99 days → 5% penalty = 5 GHC
- 100 GHC @ 1 day → 15% penalty = 15 GHC
- TOTAL: 20 GHC penalty

WEIGHTED AVERAGE (Day 100):
- Coin-Days: (100 × 99) + (100 × 1) = 10,000
- Total: 200 GHC
- Average Age: 50 days
- 200 GHC @ 10% = 20 GHC penalty

DIFFERENCE: 0 GHC (often close or identical)

In edge cases, difference is typically <5% of penalty amount.
```

This small precision trade-off is **vastly outweighed** by:
- **2,700x storage reduction**
- **O(1) vs O(n) computation**
- **Elimination of gaming vectors**
- **Simpler, more reliable code**
- **Faster upgrades and migrations**

### 7.3 Conclusion

The per-batch approach introduces **significant scalability, security, and reliability problems** that make it unsuitable for production blockchain systems. The weighted average is not just a performance optimization—it's a **fundamentally safer and more robust architecture** for canister environments where resources are constrained and state consistency is critical.

---

## Related Documents

- [ADVANCED_INTEREST_FLOW.md](./ADVANCED_INTEREST_FLOW.md) - Comprehensive interest system documentation
- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) - Staking implementation details
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Overall system architecture
