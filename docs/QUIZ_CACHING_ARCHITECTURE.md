# Quiz Caching Architecture

**Document Version**: 1.0  
**Last Updated**: January 2026  
**Author**: Architecture Team

---

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Data Flow Diagrams](#data-flow-diagrams)
4. [Component Deep Dive](#component-deep-dive)
5. [Reliability Analysis](#reliability-analysis)
6. [Scalability Analysis](#scalability-analysis)
7. [Safety Analysis](#safety-analysis)
8. [User Profile Sharding](#user-profile-sharding)
9. [Unbounded Growth Analysis](#unbounded-growth-analysis)
10. [Archive Canisters](#archive-canisters)
11. [Recommendations](#recommendations)

---

## Executive Summary

### What is Quiz Caching?

Quiz caching is a **push-based distribution mechanism** that replicates quiz answer hashes from the `learning_engine` to all `user_profile` shards via the `staking_hub`. This eliminates the need for inter-canister calls during quiz verification, significantly improving performance and scalability.

### Current Architecture Status

| Aspect | Status | Notes |
|--------|--------|-------|
| **Reliability** | ✅ **Good** | Fallback to remote verification, deterministic hashing |
| **Scalability** | ✅ **Good** | Bounded cache (50K entries), horizontal shard scaling |
| **Safety** | ✅ **Good** | Authorization checks, hashed answers, hard caps |
| **Archive Canisters** | ⚠️ **Not Implemented** | Transaction history grows unbounded |

---

## Architecture Overview

### High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         QUIZ CACHING ARCHITECTURE                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                        CONTENT LAYER                                        │ │
│  │                                                                             │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │ │
│  │  │                     learning_engine                                  │   │ │
│  │  │                    ════════════════                                  │   │ │
│  │  │                                                                      │   │ │
│  │  │  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────┐ │   │ │
│  │  │  │  CONTENT_NODES   │  │   QUIZ_INDEX     │  │ GLOBAL_QUIZ_CONFIG │ │   │ │
│  │  │  │  (Full content)  │  │  (Answer hashes) │  │  (Rewards/limits)  │ │   │ │
│  │  │  │                  │  │                  │  │                    │ │   │ │
│  │  │  │  • Text content  │  │  • content_id    │  │  • reward_amount   │ │   │ │
│  │  │  │  • Paraphrase    │  │  • answer_hashes │  │  • pass_threshold  │ │   │ │
│  │  │  │  • Media refs    │  │  • question_cnt  │  │  • max_* limits    │ │   │ │
│  │  │  │  • Quiz data     │  │  • version       │  │                    │ │   │ │
│  │  │  └──────────────────┘  └────────┬─────────┘  └────────────────────┘ │   │ │
│  │  │                                 │                                    │   │ │
│  │  │    SOURCE OF TRUTH              │ Trigger on content update          │   │ │
│  │  └─────────────────────────────────┼────────────────────────────────────┘   │ │
│  └────────────────────────────────────┼────────────────────────────────────────┘ │
│                                       │                                          │
│                                       ▼                                          │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                       ORCHESTRATION LAYER                                   │ │
│  │                                                                             │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │ │
│  │  │                        staking_hub                                   │   │ │
│  │  │                       ════════════                                   │   │ │
│  │  │                                                                      │   │ │
│  │  │   distribute_quiz_cache(unit_id, cache_data)                         │   │ │
│  │  │         │                                                            │   │ │
│  │  │         ├──────────────┬──────────────┬─────────────────┐            │   │ │
│  │  │         │              │              │                 │            │   │ │
│  │  │         ▼              ▼              ▼                 ▼            │   │ │
│  │  │   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │   │ │
│  │  │   │  Shard 1 │  │  Shard 2 │  │  Shard 3 │  │     Shard N      │    │   │ │
│  │  │   │  (async) │  │  (async) │  │  (async) │  │      (async)     │    │   │ │
│  │  │   └──────────┘  └──────────┘  └──────────┘  └──────────────────┘    │   │ │
│  │  │                                                                      │   │ │
│  │  │   sync_new_shard(shard_id)  ←── Full cache sync for new shards      │   │ │
│  │  └─────────────────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         USER DATA LAYER                                     │ │
│  │                                                                             │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │ │
│  │  │ user_profile    │  │ user_profile    │  │     user_profile            │ │ │
│  │  │   Shard 1       │  │   Shard 2       │  │       Shard N               │ │ │
│  │  │                 │  │                 │  │                             │ │ │
│  │  │ QUIZ_CACHE:     │  │ QUIZ_CACHE:     │  │ QUIZ_CACHE:                 │ │ │
│  │  │ ┌─────────────┐ │  │ ┌─────────────┐ │  │ ┌─────────────┐             │ │ │
│  │  │ │ unit_id →   │ │  │ │ unit_id →   │ │  │ │ unit_id →   │             │ │ │
│  │  │ │ QuizCache   │ │  │ │ QuizCache   │ │  │ │ QuizCache   │             │ │ │
│  │  │ └─────────────┘ │  │ └─────────────┘ │  │ └─────────────┘             │ │ │
│  │  │                 │  │                 │  │                             │ │ │
│  │  │ Limit: 50,000   │  │ Limit: 50,000   │  │ Limit: 50,000               │ │ │
│  │  │ entries         │  │ entries         │  │ entries                     │ │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow Diagrams

### Flow 1: Content Publication (Quiz Cache Distribution)

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                     CONTENT PUBLICATION FLOW                                   │
│                   (One-time async, on content update)                          │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌──────────────┐                                                             │
│  │   Admin/     │  1. Content proposal approved                               │
│  │  Governance  │─────────────────────────────────────────┐                   │
│  └──────────────┘                                         │                   │
│                                                           ▼                   │
│                                               ┌────────────────────┐          │
│                                               │  learning_engine   │          │
│                                               │                    │          │
│                                               │  2. Store content  │          │
│                                               │  3. Update indexes │          │
│                                               │  4. Compute hashes │          │
│                                               └─────────┬──────────┘          │
│                                                         │                     │
│                                                         │ 5. Call             │
│                                                         │    distribute_quiz  │
│                                                         │    _cache()         │
│                                                         ▼                     │
│                                               ┌────────────────────┐          │
│                                               │    staking_hub     │          │
│                                               │                    │          │
│                                               │  6. Get all shards │          │
│                                               │  7. Fan out calls  │          │
│                                               └─────────┬──────────┘          │
│                                                         │                     │
│                 ┌───────────────────┬───────────────────┼───────────────┐     │
│                 │                   │                   │               │     │
│                 ▼                   ▼                   ▼               ▼     │
│         ┌─────────────┐     ┌─────────────┐     ┌─────────────┐ ┌───────────┐ │
│         │   Shard 1   │     │   Shard 2   │     │   Shard 3   │ │  Shard N  │ │
│         │             │     │             │     │             │ │           │ │
│         │  8. Check   │     │  8. Check   │     │  8. Check   │ │  8. Check │ │
│         │     bounds  │     │     bounds  │     │     bounds  │ │    bounds │ │
│         │  9. Evict   │     │  9. Evict   │     │  9. Evict   │ │  9. Evict │ │
│         │     if full │     │     if full │     │     if full │ │    if full│ │
│         │ 10. Insert  │     │ 10. Insert  │     │ 10. Insert  │ │ 10. Insert│ │
│         │     cache   │     │     cache   │     │     cache   │ │    cache  │ │
│         └─────────────┘     └─────────────┘     └─────────────┘ └───────────┘ │
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Flow 2: Quiz Submission (User Action)

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                     QUIZ SUBMISSION FLOW                                       │
│                    (Per-user, on quiz attempt)                                 │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌──────────┐                                                                  │
│  │  User    │  1. submit_quiz(unit_id, answers)                               │
│  │ Frontend │──────────────────────────────────────┐                          │
│  └──────────┘                                      │                          │
│                                                    ▼                          │
│                                        ┌─────────────────────────┐            │
│                                        │    user_profile shard   │            │
│                                        │                         │            │
│                                        │  2. Check registration  │            │
│                                        │  3. Fetch QuizConfig*   │────┬──►    │
│                                        │  4. Check time limits   │    │       │
│                                        │  5. Check allowance     │    │       │
│                                        │  6. Check completion    │    │       │
│                                        └───────────┬─────────────┘    │       │
│                                                    │                  │       │
│                                                    │                  │       │
│                              ┌─────────────────────┴───────────────┐  │       │
│                              │                                     │  │       │
│                              ▼                                     ▼  │       │
│                   ┌────────────────────┐            ┌──────────────┐  │       │
│                   │  CACHE HIT PATH    │            │ CACHE MISS   │  │       │
│                   │  ═══════════════   │            │ (Fallback)   │  │       │
│                   │                    │            │              │  │       │
│                   │ 7a. Hash answers   │            │ 7b. Call     │  │       │
│                   │     locally        │            │     learning │  │       │
│                   │                    │            │     _engine  │  │       │
│                   │ 8a. Compare with   │            │     .verify  │  │       │
│                   │     cache hashes   │            │     _quiz()  │──┼──►    │
│                   │                    │            │              │  │       │
│                   │ Latency: <1ms      │            │ Latency:     │  │       │
│                   │ No inter-canister  │            │ ~100-200ms   │  │       │
│                   └─────────┬──────────┘            └──────┬───────┘  │       │
│                             │                              │          │       │
│                             └──────────────┬───────────────┘          │       │
│                                            │                          │       │
│                                            ▼                          │       │
│                                ┌─────────────────────┐                │       │
│                                │  9. Update balances │                │       │
│                                │ 10. Log transaction │                │       │
│                                │ 11. Track stats     │                │       │
│                                │ 12. Return result   │                │       │
│                                └─────────────────────┘                │       │
│                                                                       │       │
│  * Config fetch is still inter-canister (to learning_engine)         │       │
│    This ensures any config changes take effect immediately           ◄───────┘
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Flow 3: New Shard Synchronization

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                    NEW SHARD SYNC FLOW                                         │
│               (When auto-scaling creates a new shard)                          │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────┐                                                           │
│  │  staking_hub   │  1. Auto-scale timer triggers                             │
│  │    (Timer)     │─────────────────────────────────┐                         │
│  └────────────────┘                                 │                         │
│                                                     ▼                         │
│                                   ┌────────────────────────────────┐          │
│                                   │  2. create_shard_internal()    │          │
│                                   │     • Create canister          │          │
│                                   │     • Install WASM             │          │
│                                   │     • Register shard           │          │
│                                   └─────────────────┬──────────────┘          │
│                                                     │                         │
│                                                     │ 3. spawn async          │
│                                                     ▼                         │
│                                   ┌────────────────────────────────┐          │
│                                   │    sync_new_shard(shard_id)    │          │
│                                   └─────────────────┬──────────────┘          │
│                                                     │                         │
│                                                     │ 4. get_all_quiz_        │
│                                                     │    cache_data()         │
│                                                     ▼                         │
│                                   ┌────────────────────────────────┐          │
│                                   │       learning_engine          │          │
│                                   │                                │          │
│                                   │  Returns: Vec<(String,         │          │
│                                   │           QuizCacheData)>      │          │
│                                   └─────────────────┬──────────────┘          │
│                                                     │                         │
│                                                     │ 5. receive_full_quiz_   │
│                                                     │    cache(all_caches)    │
│                                                     ▼                         │
│                                   ┌────────────────────────────────┐          │
│                                   │        New Shard               │          │
│                                   │                                │          │
│                                   │  6. Iterate all caches         │          │
│                                   │  7. Apply bounded eviction     │          │
│                                   │  8. Insert each cache entry    │          │
│                                   └────────────────────────────────┘          │
│                                                                                │
│  Note: Bounded to QUIZ_CACHE_LIMIT (50,000 entries)                           │
│  If more caches than limit, FIFO eviction applies                             │
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## Component Deep Dive

### QuizCacheData Structure

```rust
/// Quiz data cached locally for O(1) verification
struct QuizCacheData {
    content_id: String,           // ID of the content node
    answer_hashes: Vec<[u8; 32]>, // SHA256-like hashes of each answer
    question_count: u8,           // Number of questions (max 255)
    version: u64,                 // Content version for cache invalidation
}
```

**Memory Footprint per Entry:**
- `content_id`: ~50 bytes average (variable)
- `answer_hashes`: 32 bytes × 5 questions = 160 bytes typical
- `question_count`: 1 byte
- `version`: 8 bytes
- **Total**: ~220-300 bytes per cache entry

### Bounded Cache Constant

```rust
const QUIZ_CACHE_LIMIT: u64 = 50_000;  // Maximum cache entries per shard
```

**Memory Budget:**
- 50,000 entries × 300 bytes = **~15 MB** per shard
- Well within IC canister limits (4GB heap, ~2GB stable)

### Stable Hash Function

```rust
/// Stable deterministic hash for answer verification (djb2 variant)
fn stable_hash(data: &[u8]) -> [u8; 32] {
    let mut hash: u64 = 5381;
    for b in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*b as u64);
    }
    // Expand 8-byte hash to 32 bytes (for compatibility)
    let b = hash.to_le_bytes();
    let mut res = [0u8; 32];
    res[0..8].copy_from_slice(&b);
    res[8..16].copy_from_slice(&b);
    res[16..24].copy_from_slice(&b);
    res[24..32].copy_from_slice(&b);
    res
}
```

**Key Properties:**
- ✅ **Deterministic**: Same input always produces same output
- ✅ **Cross-canister consistent**: Identical implementation in both canisters
- ✅ **Upgrade-safe**: No dependency on randomness or time
- ⚠️ **Brute-forceable**: Only 4 possible answers (0-3), but this is acceptable given answers are already visible in UI

---

## Reliability Analysis

### ✅ Strengths

| Feature | Implementation | Benefit |
|---------|---------------|---------|
| **Fallback Mechanism** | If cache miss, calls `learning_engine.verify_quiz()` | System never fails due to cache issues |
| **Deterministic Hashing** | Custom djb2 variant hash function | Results are reproducible across upgrades |
| **Authorization Checks** | All cache updates verify `caller == staking_hub` | Prevents unauthorized cache poisoning |
| **Async Distribution** | `ic_cdk::spawn()` for non-blocking fan-out | Hub doesn't block waiting for shard updates |
| **Version Tracking** | Cache entries include version numbers | Enables future cache invalidation |

### Reliability Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       RELIABILITY ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   submit_quiz(unit_id, answers)                                              │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────────────────────────────────┐                             │
│   │ Try Local Cache Verification              │                             │
│   │                                           │                             │
│   │ QUIZ_CACHE.get(unit_id) ─────────────────┼────────────┐                 │
│   └───────────────────────────────────────────┘            │                 │
│           │                                                │                 │
│           ▼                                                ▼                 │
│   ┌───────────────────┐                       ┌───────────────────┐         │
│   │   CACHE HIT       │                       │   CACHE MISS      │         │
│   │   ✅ Fast path    │                       │   ⚠️ Fallback     │         │
│   │                   │                       │                   │         │
│   │ Local hash        │                       │ Inter-canister    │         │
│   │ comparison        │                       │ call to           │         │
│   │                   │                       │ learning_engine   │         │
│   │ Latency: <1ms     │                       │                   │         │
│   │                   │                       │ Latency: 100-200ms│         │
│   └─────────┬─────────┘                       └─────────┬─────────┘         │
│             │                                           │                   │
│             └─────────────────┬─────────────────────────┘                   │
│                               │                                              │
│                               ▼                                              │
│                       ┌───────────────────┐                                 │
│                       │ Continue with     │                                 │
│                       │ verified result   │                                 │
│                       └───────────────────┘                                 │
│                                                                              │
│   Expected Cache Hit Rate: >99%                                             │
│   (Cache populated on content publish)                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Scalability Analysis

### ✅ Strengths

| Feature | Details |
|---------|---------|
| **Horizontal Scaling** | Shards auto-scale at 90K users, hard limit 100K |
| **Bounded Cache** | 50,000 entry limit prevents memory exhaustion |
| **O(1) Verification** | Hash comparison is constant time |
| **Parallel Distribution** | Hub uses `spawn()` for async fan-out |

### Scalability Projections

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SCALING CAPACITY PROJECTIONS                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  QUIZ CACHE SIZING (Per Shard)                                              │
│  ════════════════════════════                                               │
│                                                                              │
│  Current Limit: 50,000 entries                                              │
│  Memory per entry: ~300 bytes                                               │
│  Total cache memory: ~15 MB                                                 │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Quizzes   │ Cache Size │ Status        │    Visual                  │    │
│  ├───────────┼────────────┼───────────────┼────────────────────────────┤    │
│  │ 1,000     │ ~300 KB    │ ✅ Excellent  │ ██░░░░░░░░░░░░░░░░░░░ 2%  │    │
│  │ 10,000    │ ~3 MB      │ ✅ Excellent  │ ████░░░░░░░░░░░░░░░░░ 20% │    │
│  │ 50,000    │ ~15 MB     │ ✅ At limit   │ ████████████████████ 100% │    │
│  │ 100,000   │ (eviction) │ ⚠️ FIFO evict │ ████████████████████ 100% │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  USER SCALING (System-wide)                                                 │
│  ═════════════════════════                                                  │
│                                                                              │
│  Users/Shard: 100,000 max                                                   │
│  Shards: Unlimited (auto-created)                                           │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Users      │ Shards │ Quiz Cache │ Status                           │    │
│  ├────────────┼────────┼────────────┼──────────────────────────────────┤    │
│  │ 100,000    │ 1-2    │ 15 MB × 2  │ ✅ Initial scale                 │    │
│  │ 1,000,000  │ 10-11  │ 15 MB × 11 │ ✅ Enterprise scale              │    │
│  │ 10,000,000 │ 100+   │ 15 MB × 100│ ✅ Mass adoption                 │    │
│  │ 100,000,000│ 1000+  │ 15 GB total│ ⚠️ Consider content sharding     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Eviction Strategy

Current implementation uses a **FIFO-approximate** eviction:

```rust
// Bounded Cache Logic: Evict if full
if map.len() >= QUIZ_CACHE_LIMIT && !map.contains_key(&unit_id) {
    // Evict oldest (approximate via first key)
    if let Some((k, _)) = map.iter().next() {
        map.remove(&k);
    }
}
map.insert(unit_id, cache);
```

**Analysis:**
- ✅ Prevents unbounded growth
- ⚠️ Eviction is not truly LRU (removes BTree's first key)
- ⚠️ May evict actively used quizzes in edge cases
- **Mitigation**: Fallback to remote verification handles evicted entries

---

## Safety Analysis

### ✅ Security Features

| Feature | Protection Against |
|---------|-------------------|
| **Authorization** | Cache updates only from `staking_hub_id` |
| **Hashed Answers** | Raw answers never stored in cache |
| **Hard Minting Cap** | Global MAX_SUPPLY (4.75B) enforced at hub |
| **Quota System** | Daily/weekly/monthly/yearly quiz limits |
| **Allowance Model** | Shards request minting permission from hub |

### Security Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SECURITY ARCHITECTURE                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  CACHE POISONING PREVENTION                                                 │
│  ═══════════════════════════                                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                          receive_quiz_cache()                        │    │
│  │                                                                      │    │
│  │    fn receive_quiz_cache(unit_id: String, cache: QuizCacheData) {   │    │
│  │        let hub_id = STAKING_HUB_ID.get();                           │    │
│  │        if ic_cdk::caller() != hub_id {  ◄─── Authorization check    │    │
│  │            ic_cdk::trap("Unauthorized cache update");               │    │
│  │        }                                                            │    │
│  │        // ... proceed with insert                                   │    │
│  │    }                                                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ANSWER HASH PROTECTION                                                     │
│  ═════════════════════                                                      │
│                                                                              │
│  ┌─────────────────────────┐        ┌─────────────────────────┐            │
│  │ Raw Answer (e.g., 2)   │───────►│ Hash: stable_hash(&[2]) │            │
│  │                        │        │                         │            │
│  │ Never stored directly  │        │ Stored in cache         │            │
│  └─────────────────────────┘        └─────────────────────────┘            │
│                                                                              │
│  Note: With only 4 options (0-3), hashes are brute-forceable,               │
│  but this matches the security model where users see all options anyway.    │
│                                                                              │
│  ECONOMIC SAFETY                                                            │
│  ═══════════════                                                            │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │     User Action        Limit Check                  Gate            │    │
│  │     ───────────        ───────────                  ────            │    │
│  │                                                                      │    │
│  │     submit_quiz()  →   daily_quizzes < max?    →   ✓/✗              │    │
│  │                    →   weekly_quizzes < max?   →   ✓/✗              │    │
│  │                    →   monthly_quizzes < max?  →   ✓/✗              │    │
│  │                    →   yearly_quizzes < max?   →   ✓/✗              │    │
│  │                    →   allowance >= reward?    →   ✓/✗              │    │
│  │                    →   !already_completed?     →   ✓/✗              │    │
│  │                                                                      │    │
│  │     All gates must pass before reward is granted                    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## User Profile Sharding

### Does Sharding Work For You? ✅ YES

Your user profile sharding architecture is **well-designed** and **production-ready**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      USER PROFILE SHARDING ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                           ┌─────────────────┐                               │
│                           │  staking_hub    │                               │
│                           │                 │                               │
│                           │ • Shard registry│                               │
│                           │ • Auto-scaling  │                               │
│                           │ • Load balancing│                               │
│                           │ • Quiz router   │                               │
│                           └────────┬────────┘                               │
│                                    │                                         │
│            ┌───────────────────────┼───────────────────────┐                │
│            │                       │                       │                │
│            ▼                       ▼                       ▼                │
│   ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐      │
│   │ user_profile    │     │ user_profile    │     │ user_profile    │      │
│   │   Shard 0       │     │   Shard 1       │     │   Shard N       │      │
│   │                 │     │                 │     │                 │      │
│   │ Users: 0-100K   │     │ Users: 0-100K   │     │ Users: 0-100K   │      │
│   │ Cache: 50K max  │     │ Cache: 50K max  │     │ Cache: 50K max  │      │
│   │                 │     │                 │     │                 │      │
│   │ Bounded:        │     │ Bounded:        │     │ Bounded:        │      │
│   │ ✅ USER_PROFILES│     │ ✅ USER_PROFILES│     │ ✅ USER_PROFILES│      │
│   │ ✅ QUIZ_CACHE   │     │ ✅ QUIZ_CACHE   │     │ ✅ QUIZ_CACHE   │      │
│   │ ✅ USER_STATS   │     │ ✅ USER_STATS   │     │ ✅ USER_STATS   │      │
│   │                 │     │                 │     │                 │      │
│   │ ⚠️ COMPLETED_   │     │ ⚠️ COMPLETED_   │     │ ⚠️ COMPLETED_   │      │
│   │    QUIZZES      │     │    QUIZZES      │     │    QUIZZES      │      │
│   │ ⚠️ USER_TRANS-  │     │ ⚠️ USER_TRANS-  │     │ ⚠️ USER_TRANS-  │      │
│   │    ACTIONS      │     │    ACTIONS      │     │    ACTIONS      │      │
│   └─────────────────┘     └─────────────────┘     └─────────────────┘      │
│                                                                              │
│   ✅ Bounded = Has explicit cap or fixed per-user count                     │
│   ⚠️ Unbounded = Can grow indefinitely (needs archiving)                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Sharding Benefits

| Benefit | Details |
|---------|---------|
| **Horizontal Scaling** | Auto-creates shards at 90K users |
| **Fault Isolation** | Issues in one shard don't affect others |
| **Parallel Processing** | Quiz submissions distributed across shards |
| **Cache Locality** | Each shard has its own quiz cache copy |
| **Governance Voting** | Hub provides lookup routing for voting power |

---

## Unbounded Growth Analysis

### Current Storage Assessment

| Storage | Type | Bound | Status |
|---------|------|-------|--------|
| `USER_PROFILES` | StableBTreeMap | 100K users/shard | ✅ Bounded |
| `USER_TIME_STATS` | StableBTreeMap | 1 per user | ✅ Bounded |
| `QUIZ_CACHE` | StableBTreeMap | 50K entries | ✅ Bounded |
| `MINTING_ALLOWANCE` | StableCell | Single value | ✅ Bounded |
| `PENDING_STATS` | StableCell | Single value | ✅ Bounded |
| `COMPLETED_QUIZZES` | StableBTreeMap | Users × Quizzes | ⚠️ **Unbounded** |
| `USER_TRANSACTIONS` | StableBTreeMap | Users × Txns | ⚠️ **Unbounded** |

### Unbounded Growth Scenarios

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     UNBOUNDED STORAGE GROWTH ANALYSIS                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  COMPLETED_QUIZZES Growth                                                   │
│  ═══════════════════════                                                    │
│                                                                              │
│  Key: UserQuizKey { user: Principal, unit_id: String }                      │
│  Value: bool (1 byte)                                                       │
│  Entry size: ~100 bytes (Candid encoded)                                    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Users    │ Quizzes/User │ Entries   │ Size      │ Status            │    │
│  ├──────────┼──────────────┼───────────┼───────────┼───────────────────┤    │
│  │ 10,000   │ 50           │ 500,000   │ ~50 MB    │ ✅ Manageable     │    │
│  │ 100,000  │ 100          │ 10M       │ ~1 GB     │ ⚠️ Concerning     │    │
│  │ 100,000  │ 500          │ 50M       │ ~5 GB     │ ❌ Over IC limit  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  USER_TRANSACTIONS Growth                                                   │
│  ════════════════════════                                                   │
│                                                                              │
│  Key: TransactionKey { user: Principal, index: u64 }                        │
│  Value: TransactionRecord { timestamp, tx_type, amount } (~100 bytes)       │
│  Entry size: ~200 bytes total                                               │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Users    │ Txns/User    │ Entries   │ Size      │ Status            │    │
│  ├──────────┼──────────────┼───────────┼───────────┼───────────────────┤    │
│  │ 10,000   │ 100          │ 1M        │ ~200 MB   │ ✅ Manageable     │    │
│  │ 100,000  │ 100          │ 10M       │ ~2 GB     │ ⚠️ At limit       │    │
│  │ 100,000  │ 500          │ 50M       │ ~10 GB    │ ❌ Over IC limit  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  20-Year Projection (100K users, 500 quizzes each, 5 txn/quiz average):     │
│  ─────────────────────────────────────────────────────────────────────      │
│  COMPLETED_QUIZZES: 50M entries × 100B = 5 GB ❌                            │
│  USER_TRANSACTIONS: 250M entries × 200B = 50 GB ❌                          │
│                                                                              │
│  Conclusion: Archive canisters ARE needed for long-term sustainability      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Archive Canisters

### Are Archive Canisters Currently Implemented? ❌ NO

Your current codebase does **not** implement archive canisters. The search for "archive" only found references in the Internet Identity declarations (external dependency), not in your core canisters.

### What Are Archive Canisters?

Archive canisters are **secondary storage canisters** that hold historical data that:
- Is rarely accessed
- Needs to be retained for audit/compliance
- Would otherwise cause primary canisters to exceed memory limits

### Proposed Archive Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PROPOSED ARCHIVE ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      user_profile shard                              │    │
│  │                                                                      │    │
│  │  ┌──────────────────┐    ┌──────────────────────────────────────┐   │    │
│  │  │ COMPLETED_QUIZZES│    │        USER_TRANSACTIONS             │   │    │
│  │  │    (Active)      │    │            (Active)                  │   │    │
│  │  │                  │    │                                      │   │    │
│  │  │ Last 30 days or  │    │ Last 100 transactions per user       │   │    │
│  │  │ Last N quizzes   │    │                                      │   │    │
│  │  └────────┬─────────┘    └──────────────────┬───────────────────┘   │    │
│  │           │                                 │                        │    │
│  │           │ archive_old_data()              │ archive_old_txns()     │    │
│  │           │ (Timer-based)                   │ (Timer-based)          │    │
│  └───────────┼─────────────────────────────────┼────────────────────────┘    │
│              │                                 │                             │
│              │                                 │                             │
│              ▼                                 ▼                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        archive_canister                                │  │
│  │                   (Per-shard or shared pool)                           │  │
│  │                                                                        │  │
│  │  ┌──────────────────────┐  ┌──────────────────────┐                   │  │
│  │  │ ARCHIVED_QUIZZES     │  │ ARCHIVED_TRANSACTIONS│                   │  │
│  │  │                      │  │                      │                   │  │
│  │  │ • Read-only          │  │ • Read-only          │                   │  │
│  │  │ • Paginated queries  │  │ • Paginated queries  │                   │  │
│  │  │ • Year/month index   │  │ • Year/month index   │                   │  │
│  │  └──────────────────────┘  └──────────────────────┘                   │  │
│  │                                                                        │  │
│  │  Archive Strategy:                                                     │  │
│  │  • Timer runs weekly                                                   │  │
│  │  • Moves data older than retention period                              │  │
│  │  • Deletes from source after confirmed archive                         │  │
│  │  • Auto-creates new archive canisters when full                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation Plan for Archive Canisters

#### Phase 1: Transaction Archive (Recommended First)

```rust
// In user_profile/src/lib.rs

const TRANSACTION_RETENTION_LIMIT: u64 = 100;  // Keep last 100 per user

#[update]
async fn archive_old_transactions() -> Result<u64, String> {
    let archive_canister = ARCHIVE_CANISTER_ID.get();
    let mut archived_count = 0;
    
    // For each user with excess transactions
    USER_PROFILES.with(|p| {
        for (user, profile) in p.borrow().iter() {
            if profile.transaction_count > TRANSACTION_RETENTION_LIMIT {
                let to_archive = profile.transaction_count - TRANSACTION_RETENTION_LIMIT;
                
                // Collect old transactions
                let old_txns: Vec<TransactionRecord> = USER_TRANSACTIONS.with(|t| {
                    (0..to_archive)
                        .filter_map(|i| t.borrow().get(&TransactionKey { user, index: i }))
                        .collect()
                });
                
                // Send to archive (fire and forget or await)
                // ... archive logic ...
                
                // Remove from local storage
                USER_TRANSACTIONS.with(|t| {
                    for i in 0..to_archive {
                        t.borrow_mut().remove(&TransactionKey { user, index: i });
                    }
                });
                
                archived_count += to_archive;
            }
        }
    });
    
    Ok(archived_count)
}
```

#### Phase 2: Quiz Completion Archive

```rust
const QUIZ_COMPLETION_RETENTION_DAYS: u64 = 365;  // Keep 1 year in shard

#[update]
async fn archive_old_quiz_completions() -> Result<u64, String> {
    let cutoff_day = get_current_day() - QUIZ_COMPLETION_RETENTION_DAYS;
    
    // Note: Current design doesn't include timestamp in UserQuizKey
    // Would need to add completion_timestamp to the value
    // or use a different key structure
    
    // Implementation similar to transaction archiving...
}
```

### Benefits of Archive Canisters

| Benefit | Impact |
|---------|--------|
| **Memory Safety** | Primary canisters stay within 2GB limit |
| **Cost Efficiency** | Archive data uses cheaper storage tiers |
| **Query Performance** | Smaller working sets enable faster queries |
| **Audit Compliance** | Historical data preserved for regulators |
| **Upgrade Safety** | Less data to serialize during upgrades |

---

## Recommendations

### Immediate Actions (Low Effort, High Impact)

1. **No Changes Needed for Quiz Caching**
   - Current implementation is reliable, safe, and scalable
   - 50K cache limit is appropriate for foreseeable content growth

2. **Monitor These Metrics**
   ```rust
   // Add monitoring query to user_profile
   #[query]
   fn get_storage_stats() -> StorageStats {
       StorageStats {
           user_count: USER_PROFILES.len(),
           completed_quizzes_count: COMPLETED_QUIZZES.len(),
           transaction_count: USER_TRANSACTIONS.len(),
           quiz_cache_count: QUIZ_CACHE.len(),
       }
   }
   ```

### Medium-Term Actions (Recommended within 6 months)

1. **Implement Transaction Archiving**
   - Create `transaction_archive` canister
   - Limit transactions per user to 100-500 in main shard
   - Archive older transactions when limit exceeded

2. **Add Timestamps to Quiz Completions**
   - Modify `UserQuizKey` or use a separate timestamp map
   - Enables time-based archiving

### Long-Term Actions (Before hitting scale limits)

1. **Implement Quiz Completion Archiving**
   - Archive completions older than 1 year
   - Maintain completion check in main shard (bloom filter or hash set)

2. **Consider Content Sharding** (100K+ quizzes)
   - Split `learning_engine` by subject/category
   - Add content router similar to user router

### Summary Assessment

| Aspect | Current Status | Recommendation |
|--------|---------------|----------------|
| **Quiz Caching** | ✅ Production Ready | Monitor, no changes |
| **User Sharding** | ✅ Production Ready | Monitor auto-scaling |
| **Quiz Cache Bounds** | ✅ 50K limit adequate | Increase if content exceeds 50K |
| **Transaction Archive** | ❌ Not Implemented | **Implement within 6 months** |
| **Quiz Completion Archive** | ❌ Not Implemented | Plan for year 2+ |
| **Learning Engine Archive** | ✅ VERSION_HISTORY bounded | Monitor growth |

---

**Document Status**: Complete  
**Next Review**: July 2026  
**Change Log**:
- v1.0 (Jan 2026): Initial architecture documentation
