# Archive Canister Architecture

**Document Version**: 2.2  
**Last Updated**: January 2026  
**Status**: ✅ IMPLEMENTED (Phase 2 Complete - staking_hub Integration)  
**Priority**: High - Required for Long-term Scalability

---

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Problem Statement](#problem-statement)
3. [Architecture Decision](#architecture-decision)
4. [Data Structures](#data-structures)
5. [Archive Creation Model](#archive-creation-model)
6. [Archive Flow](#archive-flow)
7. [Data Access Patterns](#data-access-patterns)
8. [Capacity Analysis](#capacity-analysis)
9. [Reliability & Safety](#reliability--safety)
10. [Security](#security)
11. [Implementation Plan](#implementation-plan)
12. [API Reference](#api-reference)
13. [Future: Archive Chaining](#future-archive-chaining)

---

## Executive Summary

### What We're Archiving

| Data | Current Location | Problem | Solution |
|------|-----------------|---------|----------|
| `USER_TRANSACTIONS` | user_profile shards | Unbounded growth | Keep last 100 per user, archive rest |
| `COMPLETED_QUIZZES` | user_profile shards | Unbounded growth | Keep hash set locally, archive full records |

### Architecture Decision

**One Archive Canister Per Shard** - Each user_profile shard gets its own dedicated archive canister, created together by the staking_hub.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ARCHIVE ARCHITECTURE                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                           ┌─────────────────┐                               │
│                           │   staking_hub   │                               │
│                           │                 │                               │
│                           │ Creates BOTH:   │                               │
│                           │ • user_profile  │                               │
│                           │ • archive       │                               │
│                           └────────┬────────┘                               │
│                                    │                                         │
│         ┌──────────────────────────┼──────────────────────────┐             │
│         │                          │                          │             │
│         ▼                          ▼                          ▼             │
│   ┌───────────┐              ┌───────────┐              ┌───────────┐       │
│   │  SHARD 0  │              │  SHARD 1  │              │  SHARD N  │       │
│   │           │              │           │              │           │       │
│   │ user_prof │──────►       │ user_prof │──────►       │ user_prof │──────►│
│   │   ↓       │              │   ↓       │              │   ↓       │       │
│   │ archive_0 │              │ archive_1 │              │ archive_N │       │
│   └───────────┘              └───────────┘              └───────────┘       │
│                                                                              │
│   Benefits:                                                                  │
│   • Same user's data always in same archive (no routing complexity)        │
│   • 1:1 relationship is simple to manage                                    │
│   • Parallel scaling (more shards = more archives)                          │
│   • Fault isolation                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Problem Statement

### Current State: Unbounded Growth

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    UNBOUNDED GROWTH PROBLEM                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  USER_TRANSACTIONS: Map<(Principal, u64), TransactionRecord>                │
│  ══════════════════════════════════════════════════════════                 │
│                                                                              │
│  Stores ALL transaction history for each user. No eviction policy.          │
│                                                                              │
│  Year 1:   10K users × 100 txns   = 1M entries     (~200 MB)               │
│  Year 5:   50K users × 500 txns   = 25M entries    (~5 GB) ⚠️              │
│  Year 10: 100K users × 1000 txns  = 100M entries   (~20 GB) ❌             │
│  Year 20: 100K users × 5000 txns  = 500M entries   (~100 GB) ❌            │
│                                                                              │
│  COMPLETED_QUIZZES: Map<(Principal, String), bool>                          │
│  ══════════════════════════════════════════════════                         │
│                                                                              │
│  Stores record of every quiz completion. No eviction policy.                │
│                                                                              │
│  Year 1:   10K users × 50 quizzes  = 500K entries   (~50 MB)               │
│  Year 5:   50K users × 200 quizzes = 10M entries    (~1 GB)                │
│  Year 10: 100K users × 500 quizzes = 50M entries    (~5 GB) ⚠️             │
│  Year 20: 100K users × 1000 quizzes = 100M entries  (~10 GB) ❌            │
│                                                                              │
│  PROBLEM: Canister upgrades serialize all data. Large datasets cause        │
│  upgrade failures (timeout, out of cycles). Practical limit ~2-4 GB.        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why NOT Just Use Stable Memory's 400GB Limit?

The 400GB stable memory limit is theoretical. Practical limits exist:

1. **Upgrade time** - Larger datasets = longer upgrade serialization
2. **Query performance** - Smaller working sets = faster queries
3. **Cost efficiency** - Cycles scale with data size
4. **Canister complexity** - Single responsibility principle

---

## Architecture Decision

### One Archive Per Shard (NOT One Global Archive)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    WHY ONE ARCHIVE PER SHARD?                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ✅ ONE ARCHIVE PER SHARD (Recommended)                                     │
│  ══════════════════════════════════════                                     │
│                                                                              │
│  ┌───────────────┐        ┌───────────────┐                                 │
│  │ user_profile  │───────►│   archive     │                                 │
│  │   Shard 0     │        │   Shard 0     │                                 │
│  └───────────────┘        └───────────────┘                                 │
│                                                                              │
│  ┌───────────────┐        ┌───────────────┐                                 │
│  │ user_profile  │───────►│   archive     │                                 │
│  │   Shard 1     │        │   Shard 1     │                                 │
│  └───────────────┘        └───────────────┘                                 │
│                                                                              │
│  Pros:                                                                      │
│  • Simple 1:1 relationship                                                  │
│  • Same user's data always in same archive                                  │
│  • No cross-shard routing needed                                            │
│  • Horizontal scaling                                                       │
│  • Fault isolation                                                          │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  ❌ ONE GLOBAL ARCHIVE (Not Recommended)                                    │
│  ═══════════════════════════════════════                                    │
│                                                                              │
│  ┌───────────────┐                                                          │
│  │   Shard 0     │──────┐                                                   │
│  │   Shard 1     │──────┼────►  ONE ARCHIVE  ← Bottleneck!                 │
│  │   Shard N     │──────┘                                                   │
│  └───────────────┘                                                          │
│                                                                              │
│  Cons:                                                                      │
│  • Single point of failure                                                  │
│  • Write contention from multiple shards                                    │
│  • Doesn't scale horizontally                                               │
│  • Complex user→source routing                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Structures

### Storage: StableBTreeMap (Same as Existing Codebase)

We use `StableBTreeMap` from `ic_stable_structures` - the same pattern already used throughout the codebase. This provides:

- ✅ Stable memory storage (survives upgrades)
- ✅ Up to 400GB capacity
- ✅ Sorted keys (enables efficient range queries)
- ✅ O(log n) lookup and insert

### Archive Canister Storage Schema

```rust
// archive_canister/src/lib.rs

use ic_stable_structures::{StableBTreeMap, StableCell, DefaultMemoryImpl, memory_manager::*};
use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// KEY STRUCTURE
// ============================================================================

/// Key for archived transactions
/// Sorted by (user, sequence) for efficient per-user range queries
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ArchiveKey {
    user: Principal,      // Primary sort key - groups user's data together
    sequence: u64,        // Secondary sort key - order within user
}

// ============================================================================
// VALUE STRUCTURE
// ============================================================================

/// Archived transaction record (compact)
#[derive(CandidType, Deserialize, Clone)]
struct ArchivedTransaction {
    timestamp: u64,       // Original transaction timestamp
    tx_type: u8,          // 0 = QuizReward, 1 = Unstake, etc.
    amount: u64,          // Token amount (e8s)
    archived_at: u64,     // When this was archived (for audit)
}

// Size: ~30 bytes per record

// ============================================================================
// STORABLE IMPLEMENTATIONS
// ============================================================================

impl Storable for ArchiveKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { 
        max_size: 50,        // Principal (29) + u64 (8) + overhead
        is_fixed_size: false 
    };
}

impl Storable for ArchivedTransaction {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { 
        max_size: 50,        // 3 × u64 (24) + u8 (1) + overhead
        is_fixed_size: false 
    };
}

// ============================================================================
// STORAGE DECLARATIONS
// ============================================================================

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Main archive storage: (user, sequence) -> transaction
    /// Memory ID 0: Primary data store
    static ARCHIVED_TRANSACTIONS: RefCell<StableBTreeMap<ArchiveKey, ArchivedTransaction, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        ));

    /// User archive counts: user -> number of archived transactions
    /// Memory ID 1: Enables O(1) count lookup
    static USER_ARCHIVE_COUNTS: RefCell<StableBTreeMap<Principal, u64, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));

    /// Parent shard ID - only this canister can write to us
    /// Memory ID 2: Authorization reference
    static PARENT_SHARD_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Total entries in this archive (for capacity monitoring)
    /// Memory ID 3: Quick capacity check
    static TOTAL_ENTRY_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            0u64
        ).unwrap()
    );

    /// Next archive in chain (for future extension - initially None)
    /// Memory ID 4: Chain link for future scaling
    static NEXT_ARCHIVE: RefCell<StableCell<Option<Principal>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            None
        ).unwrap()
    );
}
```

### Why This Key Design?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     KEY DESIGN: (Principal, u64)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  StableBTreeMap stores entries sorted by key.                               │
│  By making Principal the first component, all of a user's data              │
│  is stored contiguously:                                                     │
│                                                                              │
│  ARCHIVED_TRANSACTIONS (sorted by key):                                     │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │ Key                          │ Value                               │     │
│  ├────────────────────────────────────────────────────────────────────┤     │
│  │ (alice-principal, 0)        │ { ts, type, amount, archived_at }   │     │
│  │ (alice-principal, 1)        │ { ts, type, amount, archived_at }   │     │
│  │ (alice-principal, 2)        │ { ts, type, amount, archived_at }   │     │
│  │ (bob-principal, 0)          │ { ts, type, amount, archived_at }   │ ←── │
│  │ (bob-principal, 1)          │ { ts, type, amount, archived_at }   │ Bob │
│  │ (charlie-principal, 0)      │ { ts, type, amount, archived_at }   │     │
│  │ ...                         │ ...                                 │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  QUERY: Get Bob's transactions (offset=0, limit=10)                         │
│                                                                              │
│  let start = ArchiveKey { user: bob, sequence: 0 };                         │
│  map.range(start..)                         // Start at (bob, 0)           │
│     .take_while(|(k, _)| k.user == bob)     // Stop at next user           │
│     .take(10)                               // Limit results               │
│                                                                              │
│  Complexity: O(log n + limit) - very efficient!                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Archive Creation Model

### Created by staking_hub (Not by user_profile)

The staking_hub already manages shard creation. We extend this to create archive canisters alongside user_profile shards:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ARCHIVE CREATION FLOW                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  WHEN: staking_hub creates a new user_profile shard                         │
│  ═══════════════════════════════════════════════                            │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                          staking_hub                                 │    │
│  │                                                                      │    │
│  │   async fn create_shard_internal() {                                │    │
│  │       // 1. Create user_profile canister (existing)                 │    │
│  │       let user_profile_id = create_canister(USER_PROFILE_WASM);     │    │
│  │                                                                      │    │
│  │       // 2. Create archive canister (NEW)                           │    │
│  │       let archive_id = create_canister(ARCHIVE_WASM);               │    │
│  │                                                                      │    │
│  │       // 3. Initialize archive with parent shard ID                 │    │
│  │       call(archive_id, "init", { parent: user_profile_id });        │    │
│  │                                                                      │    │
│  │       // 4. Tell user_profile about its archive                     │    │
│  │       call(user_profile_id, "set_archive_canister", archive_id);    │    │
│  │                                                                      │    │
│  │       // 5. Register in shard registry (updated)                    │    │
│  │       SHARD_REGISTRY.insert(shard_index, ShardInfo {                │    │
│  │           canister_id: user_profile_id,                             │    │
│  │           archive_id: Some(archive_id),  // NEW FIELD               │    │
│  │           ...                                                        │    │
│  │       });                                                            │    │
│  │   }                                                                  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│                                                                              │
│  RESULT:                                                                    │
│  ═══════                                                                    │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         SHARD_REGISTRY                                 │  │
│  ├───────────────────────────────────────────────────────────────────────┤  │
│  │ Index │ user_profile_id │ archive_id  │ user_count │ created_at     │  │
│  ├───────────────────────────────────────────────────────────────────────┤  │
│  │ 0     │ abc-123         │ def-456     │ 50,000     │ 1705750000...  │  │
│  │ 1     │ ghi-789         │ jkl-012     │ 48,000     │ 1705850000...  │  │
│  │ 2     │ mno-345         │ pqr-678     │ 15,000     │ 1705950000...  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Updated ShardInfo Structure

```rust
// In staking_hub/src/lib.rs

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardInfo {
    pub canister_id: Principal,         // user_profile canister ID
    pub archive_id: Option<Principal>,  // NEW: archive canister ID
    pub created_at: u64,
    pub user_count: u64,
    pub status: ShardStatus,
}
```

---

## Archive Flow

### Timer-Based Archiving (in user_profile)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ARCHIVE FLOW                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  RETENTION POLICY:                                                          │
│  ═════════════════                                                          │
│  Keep last 100 transactions per user in user_profile shard                  │
│  Archive everything older                                                    │
│                                                                              │
│  TIMER: Every hour, check for archivable data                               │
│  ═══════════════════════════════════════════                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │   user_profile shard                                                 │    │
│  │                                                                      │    │
│  │   fn archive_check_timer() {  // Runs every hour                    │    │
│  │       for user in users_needing_archive() {                          │    │
│  │           let txn_count = get_transaction_count(user);               │    │
│  │           if txn_count > RETENTION_LIMIT {                           │    │
│  │               let excess = txn_count - RETENTION_LIMIT;              │    │
│  │               archive_old_transactions(user, excess);                │    │
│  │           }                                                          │    │
│  │       }                                                              │    │
│  │   }                                                                  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│                                                                              │
│  ARCHIVE TRANSACTION FLOW:                                                  │
│  ══════════════════════════                                                 │
│                                                                              │
│  Step 1: Collect old transactions                                           │
│  ────────────────────────────────                                           │
│                                                                              │
│  USER_TRANSACTIONS for Alice:                                               │
│  ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐                 │
│  │ idx0 │ idx1 │ idx2 │ ...  │ idx97│ idx98│ idx99│idx100│                 │
│  │ OLD  │ OLD  │ OLD  │      │ KEEP │ KEEP │ KEEP │ KEEP │                 │
│  └──┬───┴──┬───┴──┬───┴──────┴──────┴──────┴──────┴──────┘                 │
│     │      │      │                                                         │
│     └──────┴──────┴────► batch = transactions[0..excess]                   │
│                                                                              │
│                                                                              │
│  Step 2: Send batch to archive                                              │
│  ──────────────────────────────                                             │
│                                                                              │
│  ┌─────────────────────┐         ┌─────────────────────┐                   │
│  │    user_profile     │────────►│      archive        │                   │
│  │                     │  batch  │                     │                   │
│  │  await call(        │         │  receive_batch() {  │                   │
│  │    archive_id,      │         │    insert each txn  │                   │
│  │    "receive_batch", │         │    update counts    │                   │
│  │    (user, batch)    │         │    return Ok(count) │                   │
│  │  )                  │         │  }                  │                   │
│  └─────────────────────┘         └─────────────────────┘                   │
│                                                                              │
│                                                                              │
│  Step 3: On success, delete from source                                     │
│  ───────────────────────────────────────                                    │
│                                                                              │
│  USER_TRANSACTIONS for Alice (after archive):                               │
│  ┌──────┬──────┬──────┬──────┐                                             │
│  │ idx0 │ idx1 │ idx2 │ idx3 │  ← Re-indexed, only 100 remain             │
│  │(was97)(was98)(was99)(was100)                                            │
│  └──────┴──────┴──────┴──────┘                                             │
│                                                                              │
│  user.archived_transaction_count += archived_count  // NEW field           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Access Patterns

### How Frontend Accesses Historical Data

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       ACCESSING ARCHIVED DATA                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  USER STORY:                                                                │
│  "As a user, I want to see my complete transaction history,                │
│   including old transactions that have been archived"                       │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  API DESIGN: Paginated with source indicator                                │
│  ═══════════════════════════════════════════                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ // Response from user_profile.get_transactions_page()               │    │
│  │                                                                      │    │
│  │ TransactionPage {                                                    │    │
│  │     transactions: Vec<TransactionRecord>,  // Actual data           │    │
│  │     total_count: 500,           // Total across all sources         │    │
│  │     local_count: 100,           // In user_profile (hot)            │    │
│  │     archived_count: 400,        // In archive (cold)                │    │
│  │     archive_canister_id: "...", // Where to get archived data       │    │
│  │     source: "local" | "archive" // Where this page came from        │    │
│  │ }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  FRONTEND LOGIC:                                                            │
│  ═══════════════                                                            │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ async function getTransactionHistory(user, page, pageSize = 20) {   │    │
│  │                                                                      │    │
│  │   // 1. Get page info from user_profile                              │    │
│  │   const info = await userProfile.get_transactions_page(user, page);  │    │
│  │                                                                      │    │
│  │   // 2. If this page is in local storage, return it                  │    │
│  │   if (info.source === "local") {                                     │    │
│  │     return info.transactions;                                        │    │
│  │   }                                                                   │    │
│  │                                                                      │    │
│  │   // 3. Otherwise, query archive directly                            │    │
│  │   const archiveOffset = (page * pageSize) - info.local_count;        │    │
│  │   const archive = createActor(info.archive_canister_id);             │    │
│  │   return await archive.get_archived_transactions(                    │    │
│  │     user, archiveOffset, pageSize                                    │    │
│  │   );                                                                  │    │
│  │ }                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  EXAMPLE FLOW:                                                              │
│  ═════════════                                                              │
│                                                                              │
│  User has 500 total transactions (100 local, 400 archived)                 │
│  page_size = 20                                                             │
│                                                                              │
│  Page 0: offset 0-19   → local (most recent 20)                            │
│  Page 1: offset 20-39  → local                                              │
│  Page 2: offset 40-59  → local                                              │
│  Page 3: offset 60-79  → local                                              │
│  Page 4: offset 80-99  → local (oldest in hot storage)                     │
│  Page 5: offset 100-119 → ARCHIVE (archive_offset = 0-19)                  │
│  Page 6: offset 120-139 → ARCHIVE (archive_offset = 20-39)                 │
│  ...                                                                        │
│  Page 24: offset 480-499 → ARCHIVE (oldest transactions)                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Capacity Analysis

### How Much Data Fits In One Archive?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       CAPACITY ANALYSIS                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ARCHIVE CANISTER LIMITS:                                                   │
│  ═════════════════════════                                                  │
│  • Stable memory limit: 400 GB (theoretical)                                │
│  • Practical limit: ~300 GB (leave buffer for overhead)                     │
│                                                                              │
│  RECORD SIZE:                                                               │
│  ════════════                                                               │
│  • Key (ArchiveKey): ~50 bytes                                              │
│  • Value (ArchivedTransaction): ~50 bytes                                   │
│  • Total per entry: ~100 bytes                                              │
│                                                                              │
│  CAPACITY:                                                                  │
│  ═════════                                                                  │
│  300 GB / 100 bytes = 3,000,000,000 (3 BILLION) transactions               │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  PROJECTION FOR ONE SHARD (100K users per shard):                          │
│  ═════════════════════════════════════════════════                          │
│                                                                              │
│  │ Year │ Users │ Txns/User │ Total Txns  │ Storage  │ Status    │         │
│  │──────│───────│───────────│─────────────│──────────│───────────│         │
│  │   1  │ 100K  │    100    │    10M      │   1 GB   │ ✅ OK     │         │
│  │   5  │ 100K  │    500    │    50M      │   5 GB   │ ✅ OK     │         │
│  │  10  │ 100K  │  1,000    │   100M      │  10 GB   │ ✅ OK     │         │
│  │  20  │ 100K  │  5,000    │   500M      │  50 GB   │ ✅ OK     │         │
│  │  50  │ 100K  │ 10,000    │  1,000M     │ 100 GB   │ ✅ OK     │         │
│  │ 100  │ 100K  │ 20,000    │  2,000M     │ 200 GB   │ ✅ OK     │         │
│  │ 300  │ 100K  │ 50,000    │  5,000M     │ 500 GB   │ ⚠️ FULL   │         │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  REALISTIC ACTIVITY LEVELS:                                                 │
│  ═══════════════════════════                                                │
│                                                                              │
│  Quiz Platform: ~2 transactions per user per week                           │
│                = ~100 transactions per user per year                         │
│                                                                              │
│  To fill ONE archive canister:                                              │
│  • 100K users × 100 txns/year = 10M txns/year                              │
│  • 3 billion / 10M = 300 YEARS to fill one archive                         │
│                                                                              │
│  CONCLUSION: One archive per shard is sufficient for centuries!            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Reliability & Safety

### Transaction Safety Pattern

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     RELIABILITY GUARANTEES                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  PRINCIPLE: Never lose data - archive first, delete after confirmation     │
│  ══════════════════════════════════════════════════════════════════════    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │   SAFE ARCHIVE FLOW                                                  │    │
│  │   ═════════════════                                                  │    │
│  │                                                                      │    │
│  │   1. Collect transactions to archive                                 │    │
│  │   2. Send to archive canister                                        │    │
│  │   3. WAIT for confirmation (Ok(count))                               │    │
│  │   4. ONLY THEN delete from source                                    │    │
│  │                                                                      │    │
│  │   If step 2 or 3 fails:                                              │    │
│  │   • Data remains in user_profile                                     │    │
│  │   • Retry on next timer interval                                     │    │
│  │   • No data loss!                                                    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  FAILURE SCENARIOS AND MITIGATIONS:                                        │
│  ══════════════════════════════════                                         │
│                                                                              │
│  1. Archive call times out                                                  │
│     → Retry later, original data still in user_profile                     │
│                                                                              │
│  2. Archive canister is being upgraded                                      │
│     → Call fails, retry later, data persists in stable memory              │
│                                                                              │
│  3. Partial batch write (some entries written, crash)                       │
│     → Archive writes are idempotent (upsert)                                │
│     → Re-sending same batch = no duplicates                                 │
│                                                                              │
│  4. Success response lost (archive wrote but response lost)                 │
│     → User_profile retries, archive handles duplicate keys                  │
│     → User_profile gets success, deletes local copies                       │
│                                                                              │
│  5. Archive runs out of cycles                                              │
│     → Call fails, retry later                                               │
│     → staking_hub should monitor archive cycle balance                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation: Safe Archive Function

```rust
// In user_profile canister

const RETENTION_LIMIT: u64 = 100;  // Keep last 100 transactions per user

async fn archive_old_transactions(user: Principal, excess: u64) -> Result<u64, String> {
    // 1. Collect transactions to archive (do NOT delete yet)
    let batch: Vec<TransactionToArchive> = USER_TRANSACTIONS.with(|t| {
        let map = t.borrow();
        (0..excess)
            .filter_map(|idx| {
                let key = TransactionKey { user, index: idx };
                map.get(&key).map(|txn| TransactionToArchive {
                    timestamp: txn.timestamp,
                    tx_type: txn.tx_type as u8,
                    amount: txn.amount,
                })
            })
            .collect()
    });
    
    // 2. Send to archive
    let archive_id = ARCHIVE_CANISTER_ID.get();
    let result = ic_cdk::call::<_, (Result<u64, String>,)>(
        archive_id,
        "receive_archive_batch",
        (user, batch)
    ).await;
    
    match result {
        Ok((Ok(archived_count),)) => {
            // 3. SUCCESS: Now safe to delete from source
            delete_and_reindex_transactions(user, archived_count);
            
            // 4. Update user's archived count
            update_archived_count(user, archived_count);
            
            Ok(archived_count)
        }
        Ok((Err(e),)) => {
            // Archive rejected - will retry later
            ic_cdk::print(format!("Archive rejected: {}", e));
            Err(e)
        }
        Err((code, msg)) => {
            // Call failed - will retry later
            ic_cdk::print(format!("Archive call failed: {:?} {}", code, msg));
            Err(msg)
        }
    }
}
```

---

## Security

### Authorization Model

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       SECURITY MODEL                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  WRITE ACCESS (Who can add data to archive):                                │
│  ════════════════════════════════════════════                               │
│                                                                              │
│  ONLY the parent user_profile shard can write to its archive                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ #[update]                                                            │    │
│  │ fn receive_archive_batch(                                            │    │
│  │     user: Principal,                                                 │    │
│  │     batch: Vec<TransactionToArchive>                                 │    │
│  │ ) -> Result<u64, String> {                                           │    │
│  │                                                                      │    │
│  │     let caller = ic_cdk::caller();                                   │    │
│  │     let parent = PARENT_SHARD_ID.get();                              │    │
│  │                                                                      │    │
│  │     if caller != parent {                                            │    │
│  │         return Err("Unauthorized: Only parent shard can archive");   │    │
│  │     }                                                                │    │
│  │                                                                      │    │
│  │     // Proceed with archiving...                                     │    │
│  │ }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  READ ACCESS (Who can read archived data):                                  │
│  ═════════════════════════════════════════                                  │
│                                                                              │
│  Option A: PUBLIC READS (Recommended for blockchain transparency)           │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ #[query]  // Anyone can read                                         │    │
│  │ fn get_archived_transactions(                                        │    │
│  │     user: Principal,                                                 │    │
│  │     offset: u64,                                                     │    │
│  │     limit: u64                                                       │    │
│  │ ) -> Vec<ArchivedTransaction>                                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  Rationale: Transaction data is already on-chain and technically public.   │
│  Hiding it provides false sense of privacy.                                 │
│                                                                              │
│  Option B: OWNER-ONLY READS (For stricter privacy)                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ #[query]                                                             │    │
│  │ fn get_my_archived_transactions(                                     │    │
│  │     offset: u64,                                                     │    │
│  │     limit: u64                                                       │    │
│  │ ) -> Vec<ArchivedTransaction> {                                      │    │
│  │     let caller = ic_cdk::caller();                                   │    │
│  │     // Only returns caller's own data                                │    │
│  │ }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  DATA INTEGRITY:                                                            │
│  ═══════════════                                                            │
│                                                                              │
│  • Archive is APPEND-ONLY (no updates, no deletes)                          │
│  • Each entry includes original timestamp (audit trail)                     │
│  • Each entry includes archived_at timestamp (when archived)                │
│  • Stable memory ensures persistence across upgrades                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Archive Canister (Week 1)

**Create new canister:**
```
src/
├── archive_canister/
│   ├── Cargo.toml
│   ├── archive_canister.did
│   └── src/
│       └── lib.rs
```

**Tasks:**
1. Create canister skeleton with memory manager
2. Implement storage structures (ARCHIVED_TRANSACTIONS, USER_ARCHIVE_COUNTS)
3. Implement `init` (set parent shard)
4. Implement `receive_archive_batch`
5. Implement `get_archived_transactions`
6. Implement `get_archived_count`
7. Implement `get_stats` (total count, capacity percentage)

### Phase 2: User Profile Integration (Week 2)

**Tasks:**
1. Add `ARCHIVE_CANISTER_ID` to user_profile storage
2. Add `archived_transaction_count` field to UserProfile
3. Implement `set_archive_canister` function
4. Implement `archive_check_timer` (hourly check)
5. Implement `archive_old_transactions` with safe pattern
6. Update `get_transactions_page` for combined queries
7. Add `get_archive_canister` query

### Phase 3: Staking Hub Integration (Week 2)

**Tasks:**
1. Add archive wasm storage to staking_hub
2. Update `create_shard_internal` to create both canisters
3. Add `archive_id` to ShardInfo
4. Add `get_archive_for_shard` query
5. Update shard sync to include archive info

### Phase 4: Testing (Week 3)

**Tasks:**
1. Unit tests for archive canister
2. Integration tests for archive flow
3. Test failure/retry scenarios
4. Test pagination across hot/cold data
5. Performance testing with large datasets
6. Capacity monitoring tests

---

## API Reference

### Archive Canister API

```candid
// archive_canister.did

type TransactionToArchive = record {
    timestamp: nat64;
    tx_type: nat8;        // 0 = QuizReward, 1 = Unstake
    amount: nat64;
};

type ArchivedTransaction = record {
    timestamp: nat64;
    tx_type: nat8;
    amount: nat64;
    archived_at: nat64;
};

type ArchiveStats = record {
    total_entries: nat64;
    capacity_percent: nat8;
    user_count: nat64;
    parent_shard: principal;
    next_archive: opt principal;
};

service : {
    // Initialize with parent shard
    init: (principal) -> ();
    
    // Write (only parent shard)
    receive_archive_batch: (principal, vec TransactionToArchive) -> (variant { Ok: nat64; Err: text });
    
    // Read
    get_archived_transactions: (principal, nat64, nat64) -> (vec ArchivedTransaction) query;
    get_archived_count: (principal) -> (nat64) query;
    get_stats: () -> (ArchiveStats) query;
    
    // Admin
    get_parent_shard: () -> (principal) query;
}
```

### Updated User Profile API

```candid
// Updated user_profile.did additions

type TransactionPage = record {
    transactions: vec TransactionRecord;
    total_count: nat64;
    local_count: nat64;
    archived_count: nat64;
    archive_canister_id: principal;
    source: text;  // "local" or "archive"
};

service : {
    // ... existing endpoints ...
    
    // NEW: Paginated transaction access
    get_transactions_page: (principal, nat32) -> (TransactionPage) query;
    
    // NEW: Get archive canister ID
    get_archive_canister: () -> (principal) query;
    
    // NEW: Admin - set archive canister
    set_archive_canister: (principal) -> ();
    
    // NEW: Admin - trigger archive manually
    trigger_archive: () -> (variant { Ok: nat64; Err: text });
}
```

---

## Future: Archive Chaining

### Not Needed Now, But Here's How It Would Work

Based on capacity analysis, one archive per shard will last for hundreds of years with normal usage. However, if you ever need to extend beyond 3 billion transactions per shard, here's the archive chaining architecture:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FUTURE: ARCHIVE CHAINING                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  WHEN TO IMPLEMENT:                                                         │
│  ══════════════════                                                         │
│  • When archive reaches ~80% capacity (monitored via get_stats)             │
│  • Likely 100+ years from now with normal usage                             │
│                                                                              │
│  HOW IT WORKS:                                                              │
│  ══════════════                                                             │
│                                                                              │
│  ┌─────────────────┐                                                        │
│  │  user_profile   │                                                        │
│  │    Shard 0      │                                                        │
│  │                 │                                                        │
│  │ ARCHIVE_ID ─────┼───┐  (always points to first archive)                 │
│  └─────────────────┘   │                                                    │
│                        │                                                    │
│                        ▼                                                    │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐       │
│  │   archive_0     │────►│   archive_0_1   │────►│   archive_0_2   │       │
│  │                 │NEXT │                 │NEXT │                 │       │
│  │ Status: FULL    │     │ Status: FULL    │     │ Status: ACTIVE  │       │
│  │ Entries: 0-3B   │     │ Entries: 3B-6B  │     │ Entries: 6B-9B  │       │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘       │
│          │                       │                       │                  │
│          ▼                       ▼                       ▼                  │
│      Years 1-100            Years 100-200           Years 200-300          │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  IMPLEMENTATION APPROACH:                                                   │
│  ═══════════════════════                                                    │
│                                                                              │
│  1. Add NEXT_ARCHIVE field to archive canister (already in schema)          │
│                                                                              │
│  2. When archive is full, it creates its successor:                         │
│     ┌─────────────────────────────────────────────────────────────────┐     │
│     │ async fn receive_archive_batch(...) {                            │     │
│     │     if is_full() {                                               │     │
│     │         let next = ensure_next_archive().await;                  │     │
│     │         return forward_to(next, batch);                          │     │
│     │     }                                                            │     │
│     │     // ... normal insert ...                                     │     │
│     │ }                                                                │     │
│     └─────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  3. For reads, first archive knows which segment has data:                  │
│     • Range 0 - 3B: this archive                                            │
│     • Range 3B - 6B: next archive                                           │
│     • etc.                                                                   │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  WHY DEFER THIS:                                                            │
│  ════════════════                                                           │
│                                                                              │
│  1. Adds complexity (self-replicating canisters, chain traversal)           │
│  2. Not needed for realistic usage (100+ year horizon)                      │
│  3. Easy to add later (NEXT_ARCHIVE field already in schema)                │
│  4. IC may have better solutions by then (larger memory, etc.)              │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  CAPACITY MONITORING (Implement Now):                                       │
│  ═════════════════════════════════════                                      │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ #[query]                                                             │    │
│  │ fn get_stats() -> ArchiveStats {                                     │    │
│  │     ArchiveStats {                                                   │    │
│  │         total_entries: TOTAL_ENTRY_COUNT.get(),                      │    │
│  │         capacity_percent: (TOTAL_ENTRY_COUNT.get() * 100 / MAX) as u8│    │
│  │         user_count: USER_ARCHIVE_COUNTS.len(),                       │    │
│  │         parent_shard: PARENT_SHARD_ID.get(),                         │    │
│  │         next_archive: NEXT_ARCHIVE.get(),                            │    │
│  │     }                                                                │    │
│  │ }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  Monitor this and implement chaining when capacity_percent > 80%            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

| Question | Answer |
|----------|--------|
| **Data structure?** | `StableBTreeMap` from `ic_stable_structures` |
| **Key design?** | `(Principal, u64)` - user first for efficient range queries |
| **One archive or per-shard?** | **One per shard** - created together by staking_hub |
| **Capacity per archive?** | ~3 billion transactions (300 GB) |
| **Time to fill?** | 100-300 years with realistic usage |
| **Need chaining now?** | **No** - but NEXT_ARCHIVE field included for future |
| **Reliability?** | Archive first, delete after confirmation |
| **Security?** | Only parent shard can write; reads can be public or owner-only |

---

**Document Status**: Ready for Implementation  
**Estimated Effort**: 2-3 weeks  
**Next Step**: Implement archive canister (Phase 1)
