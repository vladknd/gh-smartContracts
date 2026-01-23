# 🏗️ GreenHero Coin (GHC) Architecture Analysis

**Generated**: December 7, 2025  
**Last Updated**: January 2026  
**Status**: Pre-Mainnet Review

> **Note**: This document was originally written in December 2025. The system has since been **refactored** in January 2026 to use separate `treasury_canister` and `governance_canister` instead of a single `operational_governance`. The old `content_governance` has been replaced with `media_assets`, `staging_assets`, and content governance functionality integrated into `governance_canister`. Some recommendations in this analysis may have already been addressed.

---

## Executive Summary

The system implements a **"Pre-Mint & Allocate" tokenomics model** on the Internet Computer Protocol (ICP). The architecture is well-designed with a clear separation of concerns, using a **Micro-Bank + Batching** model for scalability. 

### Current Architecture (January 2026 Refactoring)

The system has been refactored into these core canisters:
- **treasury_canister** - Token custody (4.25B MC), MMCR management, transfer execution
- **governance_canister** - Proposals, voting, board member management, content governance
- **staking_hub** - Central bank for mining (4.75B MUC), VUC provider, global stats
- **learning_engine** - Content storage, quiz management, version history
- **media_assets** - Permanent storage for videos/images/audio/PDFs
- **staging_assets** - Temporary content before approval
- **user_profile** - Sharded user accounts, quiz submissions, staked balances
- **founder_vesting** - Time-locked founder tokens (0.5B MC)

---

## 📊 Current Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              GOVERNANCE & TREASURY LAYER                              │
│  ┌────────────────────────────────┐    ┌────────────────────────────────┐           │
│  │     governance_canister        │◄──►│     treasury_canister          │           │
│  │   (Proposals & Voting)         │    │   (Token Custody: 4.25B MC)    │           │
│  │   • Board member management    │    │   • MMCR (15.2M/month)          │           │
│  │   • Calls treasury on execute  │    │   • Only accepts calls from    │           │
│  └────────────────────────────────┘    │     governance for transfers   │           │
│                                         └────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                          │
                                          ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                   CENTRAL BANKING LAYER                               │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐│
│  │                              staking_hub (Central Bank)                          ││
│  │   • Holds 4.75B "Mined Utility" tokens      • Global Stats (Staked, Allocated)  ││
│  │   • VUC Provider for governance             • Settlement (Unstaking)             ││
│  └─────────────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────────────┘
                      │                                            ▲
                      ▼                                            │
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────────────────────┐    ┌───────────────────────────────┐             │
│  │        ghc_ledger             │    │      user_profile (SHARD)     │             │
│  │        (ICRC-1 Token)         │◄───│      (Micro-Bank + State)     │             │
│  │   Real Token Balances         │    │                               │             │
│  └───────────────────────────────┘    └───────────────────────────────┘             │
│                                                     │                                │
│                                                     ▼                                │
│  ┌───────────────────────────────┐    ┌───────────────────────────────┐             │
│  │      learning_engine          │◄───│      staging_assets           │             │
│  │   (Content + Quiz Storage)    │    │   (Temporary Content)         │             │
│  └───────────────────────────────┘    └───────────────────────────────┘             │
│                                                     ▲                                │
│                                        ┌───────────────────────────────┐             │
│                                        │      media_assets            │             │
│                                        │   (Permanent Media Files)    │             │
│                                        └───────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. 🔐 SECURITY ANALYSIS

### 1.1 Critical Security Issues

#### **A. Missing Admin Controls (HIGH PRIORITY)**
**Location**: `staking_hub/src/lib.rs` lines 130-140

```rust
#[update]
fn add_allowed_minter(principal: Principal) {
    // In production, add admin check here!  <-- Comment exists but no check!
    ALLOWED_MINTERS.with(|m| m.borrow_mut().insert(principal, true));
}

#[update]
fn remove_allowed_minter(principal: Principal) {
    // In production, add admin check here!  <-- Same issue!
    ALLOWED_MINTERS.with(|m| m.borrow_mut().remove(&principal));
}
```

**Risk**: **CRITICAL** - Anyone can add themselves as an allowed minter and drain tokens from the system!

**Recommendation**:
```rust
thread_local! {
    static ADMIN: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            Principal::anonymous()  // Set during init
        ).unwrap()
    );
}

fn require_admin() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let admin = ADMIN.with(|a| *a.borrow().get());
    if caller != admin {
        return Err("Unauthorized: Not admin".to_string());
    }
    Ok(())
}

#[update]
fn add_allowed_minter(principal: Principal) -> Result<(), String> {
    require_admin()?;
    ALLOWED_MINTERS.with(|m| m.borrow_mut().insert(principal, true));
    Ok(())
}
```

---

#### **B. No Principal Validation in `add_learning_unit`**
**Location**: `learning_engine/src/lib.rs` lines 101-106

```rust
#[update]
fn add_learning_unit(unit: LearningUnit) -> Result<(), String> {
    // In a production environment, you should check if the caller is an admin.
    LEARNING_UNITS.with(|u| u.borrow_mut().insert(unit.unit_id.clone(), unit));
    Ok(())
}
```

**Risk**: **HIGH** - Anyone can add or overwrite learning content with malicious data!

---

#### **C. Voting Power Function** ✅ RESOLVED (January 2026)
**Location**: `governance_canister/src/lib.rs`

> **Note**: This issue was **resolved** in January 2026. The `governance_canister` now has `get_user_voting_power` and `get_my_voting_power` update methods that query `staking_hub`, which in turn aggregates voting power from user profile shards.

The original concern was that the governance canister called `staking_hub.get_voting_power()`, but this function was **removed** from `staking_hub`. This has since been resolved.

**Current Flow**:
1. `governance_canister.get_user_voting_power(principal)` calls `staking_hub.get_vuc(principal)`
2. `staking_hub` aggregates VUC from the appropriate `user_profile` shard
3. Result is returned to governance canister for voting weight calculation

---

#### **D. No Rate Limiting on Quiz Submissions**
**Location**: `user_profile/src/lib.rs`

The daily quiz limit (5/day) is good, but there's no:
- **Per-second/minute rate limiting** to prevent spam
- **Cooldown between retries** when a quiz fails

**Risk**: An attacker can spam `submit_quiz` with different unit_ids, causing cross-canister call flooding.

---

### 1.2 Medium Security Issues

#### **E. Denial of Service via Pending Stats Manipulation**
**Location**: `user_profile/src/lib.rs` lines 530-536

When a quiz is submitted, `staked_delta` is incremented as `i64`. If an attacker orchestrates millions of quick operations across multiple shards, the batched sync could potentially overflow or cause inconsistencies.

**Recommendation**: Add overflow checks and upper bounds:
```rust
stats.staked_delta = stats.staked_delta.checked_add(reward_amount as i64)
    .ok_or("Staked delta overflow")?;
```

---

#### **F. No Signature/Proof on Cross-Canister Calls**
The `sync_shard` function only checks if the caller is in `ALLOWED_MINTERS`. This is good, but:
- There's no audit trail of what was synced
- State changes are trusted without verification

**Recommendation**: Log all sync operations with timestamps for auditability.

---

## 2. 📈 SCALABILITY ANALYSIS

### 2.1 Current Scalability Strengths ✅

| Aspect | Implementation | Rating |
|--------|---------------|--------|
| **Sharded User State** | `user_profile` designed for horizontal scaling | ⭐⭐⭐⭐ |
| **Stateless Learning Engine** | Pure query verification | ⭐⭐⭐⭐⭐ |
| **Batched Sync** | Periodic allowance refresh (5 min) | ⭐⭐⭐⭐ |
| **Lazy Interest** | O(1) distribution via Global Index | ⭐⭐⭐⭐⭐ |
| **Stable Structures** | `ic-stable-structures` for scalable storage | ⭐⭐⭐⭐ |

### 2.2 Scalability Bottlenecks & Improvements

#### **A. Single Point of Failure: `staking_hub`**
**Problem**: All shards sync to **one** `staking_hub`. If there are 100 shards syncing every 5 minutes, that's 20 calls/min to a single canister.

**Risk**: As scale increases, `staking_hub` becomes a bottleneck.

**Recommendations**:
1. **Implement Hub Replication**: Have multiple read-replicas for queries with a single leader for writes.
2. **Increase Sync Interval**: At scale, consider 15-30 minute sync intervals.
3. **Shard the Hub**: For millions of users, consider a hub-per-region model.

---

#### **B. No Shard Registry or User Routing**
**Problem**: The documentation mentions sharding but there's **no router canister** to map users to shards.

**Current State**: One `user_profile` canister exists. When you scale to multiple shards, how does the frontend know which shard to call?

**Recommendation**: Implement a **lightweight Router Canister**:
```rust
// router canister
#[query]
fn get_shard_for_user(user: Principal) -> Principal {
    // Deterministic routing based on principal hash
    let hash = hash(user);
    SHARDS.with(|s| s.borrow()[hash % s.borrow().len()])
}
```

---

#### **C. Learning Content Scaling**
**Problem**: All learning units are in **one** `learning_engine` canister. The `max_size: 50000` per unit is generous, but with thousands of books, storage will hit limits.

**Recommendation**: 
- Shard by **subject/category** (e.g., `learning_math`, `learning_science`)
- Add content metadata to a router for lookup

---

#### **D. Transaction History Scaling**
**Location**: `user_profile/src/lib.rs` - `USER_TRANSACTIONS`

**Problem**: Every quiz and unstake adds a `TransactionRecord`. For active users with 365 quizzes/year × 20 years = 7,300+ records per user.

**Risks**:
- `get_user_transactions` returns **ALL** records (no pagination)
- Storage bloat over time

**Recommendation**:
```rust
// Add pagination
#[query]
fn get_user_transactions(user: Principal, offset: u64, limit: u64) -> Vec<TransactionRecord>
```

---

### 2.3 Scaling Numbers Projection

| Users | Shards Needed | Hub Sync Calls/Min | Status |
|-------|--------------|-------------------|--------|
| 10K | 1 | 0.2 | ✅ Comfortable |
| 100K | 1-2 | 0.4 | ✅ OK |
| 1M | 10-20 | 4 | ⚠️ Monitor Hub |
| 10M | 100+ | 40+ | ❌ Need Hub Sharding |

---

## 3. 🛡️ RESILIENCE & RELIABILITY

### 3.1 Rollback Mechanics ✅
Your unstake rollback logic is well-implemented:
```rust
Err((code, msg)) => {
    // Rollback Local State
    profile.staked_balance += amount;
    profile.transaction_count -= 1;
    ...
}
```

### 3.2 Issues & Improvements

#### **A. Partial Failure in `process_unstake`**
**Location**: `staking_hub/src/lib.rs`

**Problem**: You update `GLOBAL_STATS` **before** the ledger transfer. If the transfer fails, you rollback, but what if rollback itself fails (e.g., trap)?

```rust
// Current flow:
GLOBAL_STATS.with(|s| { ... stats.total_unstaked += amount ... }); // Step 1
let result = ic_cdk::call(ledger).await;  // Step 2 (can fail)
if Err { rollback(); } // Step 3 (can trap?)
```

**Recommendation**: Use a two-phase commit or saga pattern:
1. Mark operation as "pending" 
2. Execute transfer
3. Mark as "completed" or "rolled_back"

---

#### **B. Timer Reliability Post-Upgrade**
**Good**: You restart the sync timer in `post_upgrade`! ✅

**Missing**: No persistent tracking of when last sync occurred. If upgrade happens mid-sync, pending stats could be lost.

**Recommendation**: Store `last_sync_time` in stable storage.

---

#### **C. No Health Check Endpoints**
**Problem**: No way to programmatically verify canister health.

**Recommendation**: Add health check queries:
```rust
#[query]
fn health_check() -> HealthStatus {
    HealthStatus {
        is_healthy: true,
        last_sync: LAST_SYNC_TIME.with(|t| *t.borrow().get()),
        allowance_remaining: MINTING_ALLOWANCE.with(|a| *a.borrow().get()),
        users_count: USER_PROFILES.with(|p| p.borrow().len()),
    }
}
```

---

## 4. 💡 DESIGN IMPROVEMENTS

### 4.1 Missing Features

#### **A. Transaction Type for Claimed Rewards**
**Location**: `user_profile/src/lib.rs` lines 702-706

```rust
let tx_record = TransactionRecord {
    tx_type: TransactionType::QuizReward, // Using QuizReward as placeholder for now
    amount: amount,
};
```

**This is incorrect** - claimed rewards should have their own type!

**Recommendation**:
```rust
enum TransactionType {
    QuizReward,
    Unstake,
    InterestClaim,  // ADD THIS!
}
```

---

#### **B. Content Governance** ✅ RESOLVED (January 2026)
**Location**: Previously `content_governance/src/lib.rs`

> **Note**: This issue was **resolved** in January 2026. The old `content_governance` canister was replaced with:
> - **`media_assets`**: Permanent, immutable storage for videos/audio/images/PDFs
> - **`staging_assets`**: Temporary content storage awaiting governance approval
> - **`governance_canister`**: Content proposal types integrated (AddContentFromStaging, UpdateContentNode, etc.)
> - **`learning_engine`**: ContentNode tree structure with version history

The new architecture provides comprehensive content governance with proper staging, approval flow, and automatic loading.

---

#### **C. No Event/Log System**
**Problem**: No canister-level logging for important actions.

**Recommendation**: Implement an event log:
```rust
#[derive(CandidType, Deserialize)]
struct CanisterEvent {
    timestamp: u64,
    event_type: String,
    details: String,
}

static EVENTS: RefCell<Vec<CanisterEvent>> = ...;
```

---

### 4.2 Code Quality Issues

#### **A. Magic Numbers**
```rust
const MAX_SUPPLY: u64 = 4_750_000_000 * 100_000_000; // Good! Named constant
let reward_amount = 100_000_000; // Bad! Magic number
let penalty = amount / 10; // Bad! Magic number (10%)
```

**Recommendation**: Define all economic parameters as constants or configurable:
```rust
// Economic constants
const QUIZ_REWARD: u64 = 100_000_000; // 1 GHC
const UNSTAKE_PENALTY_PERCENT: u64 = 10;
const DAILY_QUIZ_LIMIT: u8 = 5;
const PASS_THRESHOLD_PERCENT: u64 = 60;
```

---

#### **B. Duplicate Code in Rollback Logic**
The unstake rollback code in `user_profile` is repeated twice (lines 620-632 and 636-648). Extract to a helper function.

---

## 5. 📋 RECOMMENDED ACTION PLAN

### **Phase 1: Security (URGENT - Before Mainnet)**
| Priority | Issue | Effort |
|----------|-------|--------|
| 🔴 P0 | Add admin controls to `staking_hub` | 2 hours |
| 🔴 P0 | Add admin controls to `learning_engine` | 1 hour |
| 🔴 P0 | Fix `get_voting_power` in governance | 3 hours |
| 🟡 P1 | Add rate limiting to `submit_quiz` | 2 hours |
| 🟡 P1 | Add audit logging | 4 hours |

### **Phase 2: Scalability (Before 100K Users)**
| Priority | Issue | Effort |
|----------|-------|--------|
| 🟡 P1 | Implement User Shard Router | 1 day |
| 🟡 P1 | Add pagination to transactions | 2 hours |
| 🟢 P2 | Plan Learning Engine sharding | Design only |

### **Phase 3: Reliability & Operations**
| Priority | Issue | Effort |
|----------|-------|--------|
| 🟡 P1 | Add health check endpoints | 2 hours |
| 🟡 P1 | Track `last_sync_time` in stable storage | 1 hour |
| 🟢 P2 | Implement proper saga pattern for unstake | 4 hours |

### **Phase 4: Code Quality**
| Priority | Issue | Effort |
|----------|-------|--------|
| 🟢 P2 | Extract magic numbers to constants | 1 hour |
| 🟢 P2 | Add `InterestClaim` transaction type | 30 min |
| 🟢 P2 | DRY up rollback code | 30 min |

---

## 6. 🏆 SUMMARY

### Strengths
- ✅ **Excellent architectural design** with clear separation of concerns
- ✅ **Lazy interest distribution** (O(1) global index model)
- ✅ **Batched sync** reduces cross-canister traffic by 1000x+
- ✅ **Proper rollback handling** for failed operations
- ✅ **Timer restart in post_upgrade**

### Critical Gaps
- ❌ **No admin controls** on minter management (SECURITY CRITICAL)
- ❌ **Broken governance voting** (missing `get_voting_power`)
- ❌ **No shard router** for production scaling
- ❌ **No health monitoring** endpoints

### Overall Assessment
The architecture is **sound and well-designed** for scale, but requires **critical security hardening** before any mainnet deployment. The sharding model and lazy interest distribution are excellent design choices that will serve the system well at scale.

---

## Appendix: Files Analyzed

### Current Canister Structure (January 2026)
- `src/staking_hub/src/lib.rs`
- `src/user_profile/src/lib.rs`
- `src/learning_engine/src/lib.rs`
- `src/treasury_canister/src/lib.rs` (formerly part of operational_governance)
- `src/governance_canister/src/lib.rs` (formerly part of operational_governance)
- `src/founder_vesting/src/lib.rs`

### Asset Canisters
- `media_assets` (certified asset canister)
- `staging_assets` (certified asset canister)

### Documentation
- `docs/ARCHITECTURE.md`
- `docs/scalability_plan.md`
- `docs/STAKING_MECHANICS.md`
- `docs/README.md`
- `dfx.json`
