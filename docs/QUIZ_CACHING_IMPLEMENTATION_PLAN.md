# Quiz Caching & Learning Engine Optimization Plan

**Last Updated**: January 2026  
**Status**: Planning Phase  
**Priority**: High - Critical for Scalability

---

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Learning Engine Data Storage Review](#learning-engine-data-storage-review)
4. [Proposed Quiz Caching Architecture](#proposed-quiz-caching-architecture)
5. [Implementation Phases](#implementation-phases)
6. [Data Structure Optimizations](#data-structure-optimizations)
7. [Security Considerations](#security-considerations)
8. [Migration Strategy](#migration-strategy)

---

## Executive Summary

### The Problem
Currently, every quiz submission requires an inter-canister call from `user_profile` shards to `learning_engine`:

```
user_profile (shard) ──► inter-canister call ──► learning_engine.verify_quiz()
                              │
                              ▼
                    BOTTLENECK: Single point of failure
                    LATENCY: ~100-200ms per call
                    THROUGHPUT: Limited by single canister
```

### The Solution
Implement a **push-based quiz caching architecture** where:
1. Quiz answer data is cached in each `user_profile` shard
2. Verification happens locally (no inter-canister call)
3. New content is pushed from `learning_engine` → `staking_hub` → all shards

### Expected Benefits
| Metric | Current | After Caching |
|--------|---------|---------------|
| Quiz verification latency | ~100-200ms | <1ms |
| Throughput per quiz | 1 canister limit | N shards × capacity |
| Single point of failure | Yes | No |
| Inter-canister calls/quiz | 1 | 0 |

---

## Current Architecture Analysis

### Current Quiz Submission Flow
```
┌──────────────────────────────────────────────────────────────────────────────┐
│                     CURRENT QUIZ SUBMISSION FLOW                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────┐        ┌─────────────────┐        ┌────────────────────┐   │
│  │   Frontend  │ ──1──► │  user_profile   │ ──2──► │  learning_engine   │   │
│  │             │        │    (shard)      │ ◄──3── │                    │   │
│  └─────────────┘        │                 │        └────────────────────┘   │
│                         │  ──4── Reward   │                                  │
│                         └─────────────────┘                                  │
│                                                                              │
│  Steps:                                                                      │
│  1. User submits quiz answers to their shard                                │
│  2. Shard calls learning_engine.verify_quiz(unit_id, answers)              │
│  3. learning_engine returns (passed: bool, correct: u64, total: u64)       │
│  4. Shard rewards user locally if passed                                    │
│                                                                              │
│  BOTTLENECK: Step 2-3 is an inter-canister call                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Current Learning Engine Data Structure

```rust
struct LearningUnit {
    // Hierarchy (for navigation/display)
    unit_id: String,           // e.g., "1.4.2"
    unit_title: String,        // e.g., "Phase-Change-Material (PCM) technology"
    chapter_id: String,        // e.g., "I"
    chapter_title: String,     // e.g., "Chapter I"
    head_unit_id: String,      // e.g., "1.4"
    head_unit_title: String,   // e.g., "Artificially Conceptual Value (ACV)"
    
    // Content (large text blocks)
    content: String,           // Main educational content (~2-10KB)
    paraphrase: String,        // Summary (~1-5KB)
    
    // Quiz data (what we need to cache)
    quiz: Vec<QuizQuestion>,   // 5 questions per unit
}

struct QuizQuestion {
    question: String,          // Question text
    options: Vec<String>,      // 4 options typically
    answer: u8,                // Correct answer index (0-3)
}
```

### Storage: `StableBTreeMap<String, LearningUnit>`
- **Key**: `unit_id` (String, e.g., "1.4.2")
- **Value**: Full `LearningUnit` struct
- **Lookup**: O(log n) - efficient for direct lookups
- **Current Capacity**: ~50,000 bytes per unit max

---

## Learning Engine Data Storage Review

### Current Storage Assessment

#### ✅ Strengths
| Aspect | Assessment |
|--------|------------|
| **Lookup by unit_id** | O(log n) - Excellent |
| **Stable storage** | StableBTreeMap survives upgrades |
| **Single-key queries** | Very efficient |

#### ⚠️ Weaknesses & Scalability Concerns

| Issue | Current State | Problem |
|-------|---------------|---------|
| **Flat structure** | No hierarchy indexing | Cannot efficiently query "all units in chapter I" |
| **Large values** | Content + paraphrase = 5-15KB per unit | BTreeMap performance degrades with large values |
| **No versioning** | No version tracking | Cannot detect when content changes |
| **Monolithic content** | Content stored with quiz | Quiz data (tiny) bundled with content (large) |

### Storage Metrics for Current Data

Based on `learning_materials.json` analysis:
```
Total Units: ~100+ units
Average content size: ~5-8KB per unit
Average paraphrase size: ~2-4KB per unit
Quiz data per unit: ~500 bytes (5 questions × ~100 bytes)
Total content size: ~800KB - 1.2MB

Quiz-only data: ~50KB total (what we need to cache)
```

### Scalability Projections

| Scale | Units | Content Size | Quiz Cache Size | Notes |
|-------|-------|--------------|-----------------|-------|
| Current | ~100 | ~1MB | ~50KB | Fine |
| 1,000 units | 1,000 | ~10MB | ~500KB | Still fine |
| 10,000 units | 10,000 | ~100MB | ~5MB | Approaching concern |
| 100,000 units | 100,000 | ~1GB | ~50MB | Need sharding |

**Conclusion**: Learning engine content storage is adequate for 10,000+ units. The bottleneck is the inter-canister call pattern, not storage.

---

## Proposed Quiz Caching Architecture

### New Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PROPOSED: PUSH-BASED QUIZ CACHING                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONTENT PUBLISHING FLOW (Async, happens once per new content):            │
│   ═══════════════════════════════════════════════════════════               │
│                                                                              │
│   ┌─────────────────────┐                                                   │
│   │   learning_engine   │  1. Admin adds new learning unit                  │
│   │   (Source of Truth) │  2. Triggers cache distribution                   │
│   │                     │                                                   │
│   │  • Full content     │                                                   │
│   │  • Raw answers      │                                                   │
│   │  • Version tracking │──────────────────────────────────────┐            │
│   └─────────────────────┘                                      │            │
│                                                                 │            │
│                push_quiz_cache(unit_id, answer_hashes, version) │            │
│                                                                 ▼            │
│   ┌──────────────────────────────────────────────────────────────┐          │
│   │                       staking_hub                             │          │
│   │                 (Orchestrator / Registry)                     │          │
│   │                                                               │          │
│   │   • Knows all registered shards                               │          │
│   │   • Broadcasts to all active shards                           │          │
│   │   • Tracks content version per shard                          │          │
│   └───────┬─────────────────────┬─────────────────────┬──────────┘          │
│           │                     │                     │                      │
│           ▼                     ▼                     ▼                      │
│   ┌─────────────┐       ┌─────────────┐       ┌─────────────┐               │
│   │ user_profile│       │ user_profile│       │ user_profile│               │
│   │   shard 1   │       │   shard 2   │       │   shard N   │               │
│   │             │       │             │       │             │               │
│   │ QUIZ_CACHE: │       │ QUIZ_CACHE: │       │ QUIZ_CACHE: │               │
│   │ HashMap of  │       │ HashMap of  │       │ HashMap of  │               │
│   │ unit_id →   │       │ unit_id →   │       │ unit_id →   │               │
│   │ QuizCache   │       │ QuizCache   │       │ QuizCache   │               │
│   └─────────────┘       └─────────────┘       └─────────────┘               │
│                                                                              │
│   QUIZ SUBMISSION FLOW (User action, no inter-canister call):               │
│   ════════════════════════════════════════════════════════════              │
│                                                                              │
│   ┌─────────────┐        ┌─────────────────┐                                │
│   │   Frontend  │ ──1──► │  user_profile   │                                │
│   │             │        │    (shard)      │                                │
│   │             │ ◄──2── │                 │                                │
│   └─────────────┘        │  verify_local() │  ← No inter-canister call!     │
│                          │  reward_user()  │                                │
│                          └─────────────────┘                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### New Data Structures

#### In `learning_engine` (Source of Truth)
```rust
// Existing - No change
struct LearningUnit { ... }

// NEW: Content versioning
static CONTENT_VERSION: RefCell<StableCell<u64, Memory>> = ...;

// NEW: Quiz-only cache for distribution
struct QuizCacheData {
    answer_hashes: Vec<[u8; 32]>,  // SHA256 of each answer index
    question_count: u8,
    version: u64,
}
```

#### In `staking_hub` (Orchestrator)
```rust
// NEW: Track which shards have which content version
struct ShardContentVersion {
    shard_id: Principal,
    content_version: u64,
    last_sync: u64,  // timestamp
}

static SHARD_CONTENT_VERSIONS: RefCell<StableBTreeMap<Principal, ShardContentVersion, Memory>> = ...;
```

#### In `user_profile` (Cache Consumer)
```rust
// NEW: Local quiz cache
struct QuizCache {
    answer_hashes: Vec<[u8; 32]>,  // SHA256 hashes
    question_count: u8,
}

static QUIZ_CACHE: RefCell<StableBTreeMap<String, QuizCache, Memory>> = ...;
static QUIZ_CACHE_VERSION: RefCell<StableCell<u64, Memory>> = ...;
```

---

## Implementation Phases

### Phase 1: Learning Engine Enhancements
**Duration**: 1-2 days  
**Risk**: Low - Additive changes only

#### Tasks:
1. **Add content versioning**
   ```rust
   // learning_engine/src/lib.rs
   static CONTENT_VERSION: RefCell<StableCell<u64, Memory>> = ...;
   
   fn increment_version() -> u64 {
       CONTENT_VERSION.with(|v| {
           let current = *v.borrow().get();
           let new_version = current + 1;
           v.borrow_mut().set(new_version).expect("Failed to set version");
           new_version
       })
   }
   ```

2. **Add quiz cache extraction endpoint**
   ```rust
   #[query]
   fn get_quiz_cache_data(unit_id: String) -> Option<QuizCacheData> {
       LEARNING_UNITS.with(|u| {
           u.borrow().get(&unit_id).map(|unit| {
               QuizCacheData {
                   answer_hashes: unit.quiz.iter()
                       .map(|q| sha256(&[q.answer]))
                       .collect(),
                   question_count: unit.quiz.len() as u8,
                   version: CONTENT_VERSION.with(|v| *v.borrow().get()),
               }
           })
       })
   }
   
   #[query]
   fn get_all_quiz_cache_data() -> Vec<(String, QuizCacheData)> {
       // Return all quiz caches for full sync
   }
   ```

3. **Trigger cache push on content update**
   ```rust
   #[update]
   async fn add_learning_unit(unit: LearningUnit) -> Result<(), String> {
       // Existing logic...
       LEARNING_UNITS.with(|u| u.borrow_mut().insert(unit.unit_id.clone(), unit.clone()));
       
       // NEW: Trigger cache push
       let version = increment_version();
       let cache_data = QuizCacheData {
           answer_hashes: unit.quiz.iter().map(|q| sha256(&[q.answer])).collect(),
           question_count: unit.quiz.len() as u8,
           version,
       };
       
       // Push to staking_hub for distribution
       let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
       let _ = ic_cdk::call::<_, ()>(
           hub_id,
           "distribute_quiz_cache",
           (unit.unit_id.clone(), cache_data)
       ).await;
       
       Ok(())
   }
   ```

#### Deliverables:
- [ ] Content versioning system
- [ ] Quiz cache extraction endpoints
- [ ] Push trigger on content changes
- [ ] Updated Candid interface

---

### Phase 2: Staking Hub Distribution Logic
**Duration**: 1-2 days  
**Risk**: Medium - Core orchestration changes

#### Tasks:
1. **Add quiz cache distribution method**
   ```rust
   #[update]
   async fn distribute_quiz_cache(unit_id: String, cache_data: QuizCacheData) -> Result<u64, String> {
       // Only learning_engine can call this
       let caller = ic_cdk::caller();
       if caller != LEARNING_CONTENT_ID.with(|id| *id.borrow().get()) {
           return Err("Unauthorized".to_string());
       }
       
       // Get all registered shards
       let shards = get_all_registered_shards();
       
       // Fan out to all shards (async)
       let mut success_count = 0u64;
       for shard_id in shards {
           if ic_cdk::call::<_, ()>(
               shard_id,
               "receive_quiz_cache",
               (unit_id.clone(), cache_data.clone())
           ).await.is_ok() {
               success_count += 1;
           }
       }
       
       Ok(success_count)
   }
   ```

2. **Add full sync endpoint for new shards**
   ```rust
   #[update]
   async fn sync_new_shard(shard_id: Principal) -> Result<(), String> {
       // Called when a new shard is created
       // Fetch all quiz caches from learning_engine
       let learning_engine_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
       let all_caches: Vec<(String, QuizCacheData)> = ic_cdk::call(
           learning_engine_id,
           "get_all_quiz_cache_data",
           ()
       ).await.map_err(|e| format!("{:?}", e))?;
       
       // Push all to new shard
       ic_cdk::call::<_, ()>(
           shard_id,
           "receive_full_quiz_cache",
           (all_caches,)
       ).await.map_err(|e| format!("{:?}", e))?;
       
       Ok(())
   }
   ```

#### Deliverables:
- [ ] `distribute_quiz_cache` method
- [ ] `sync_new_shard` method
- [ ] Shard content version tracking
- [ ] Updated Candid interface

---

### Phase 3: User Profile Cache Implementation
**Duration**: 2-3 days  
**Risk**: Medium - Critical path changes

#### Tasks:
1. **Add quiz cache storage**
   ```rust
   // Memory ID for quiz cache
   static QUIZ_CACHE: RefCell<StableBTreeMap<String, QuizCache, Memory>> = RefCell::new(
       StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))))
   );
   
   static QUIZ_CACHE_VERSION: RefCell<StableCell<u64, Memory>> = RefCell::new(
       StableCell::init(
           MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))),
           0u64
       ).unwrap()
   );
   ```

2. **Add cache receiver methods**
   ```rust
   #[update]
   fn receive_quiz_cache(unit_id: String, cache_data: QuizCacheData) -> Result<(), String> {
       // Only staking_hub can call
       let caller = ic_cdk::caller();
       if caller != STAKING_HUB_ID.with(|id| *id.borrow().get()) {
           return Err("Unauthorized".to_string());
       }
       
       let cache = QuizCache {
           answer_hashes: cache_data.answer_hashes,
           question_count: cache_data.question_count,
       };
       
       QUIZ_CACHE.with(|c| c.borrow_mut().insert(unit_id, cache));
       QUIZ_CACHE_VERSION.with(|v| v.borrow_mut().set(cache_data.version));
       
       Ok(())
   }
   
   #[update]
   fn receive_full_quiz_cache(caches: Vec<(String, QuizCacheData)>) -> Result<(), String> {
       // Only staking_hub can call
       // ... authorization check
       
       for (unit_id, cache_data) in caches {
           let cache = QuizCache {
               answer_hashes: cache_data.answer_hashes,
               question_count: cache_data.question_count,
           };
           QUIZ_CACHE.with(|c| c.borrow_mut().insert(unit_id, cache));
       }
       
       Ok(())
   }
   ```

3. **Modify submit_quiz for local verification with fallback**
   ```rust
   #[update]
   async fn submit_quiz(unit_id: String, answers: Vec<u8>) -> Result<u64, String> {
       let user = ic_cdk::caller();
       
       // ... existing checks (registration, daily limits, already completed) ...
       
       // NEW: Try local verification first
       let verification_result = verify_quiz_local(&unit_id, &answers);
       
       let (passed, correct_count, total_questions) = match verification_result {
           Some(result) => result,
           None => {
               // Cache miss: Fall back to inter-canister call
               let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
               ic_cdk::call::<_, (bool, u64, u64)>(
                   learning_content_id,
                   "verify_quiz",
                   (unit_id.clone(), answers.clone())
               ).await.map_err(|(code, msg)| format!("Verification failed: {:?} {}", code, msg))?
           }
       };
       
       // ... rest of existing logic (reward, update stats, etc.) ...
   }
   
   fn verify_quiz_local(unit_id: &str, answers: &[u8]) -> Option<(bool, u64, u64)> {
       QUIZ_CACHE.with(|c| {
           c.borrow().get(&unit_id.to_string()).map(|cache| {
               if cache.question_count as usize != answers.len() {
                   return (false, 0, cache.question_count as u64);
               }
               
               let mut correct = 0u64;
               for (i, answer) in answers.iter().enumerate() {
                   let answer_hash = sha256(&[*answer]);
                   if answer_hash == cache.answer_hashes[i] {
                       correct += 1;
                   }
               }
               
               let passed = if cache.question_count > 0 {
                   (correct * 100) / cache.question_count as u64 >= 60
               } else {
                   false
               };
               
               (passed, correct, cache.question_count as u64)
           })
       })
   }
   ```

#### Deliverables:
- [ ] Quiz cache storage structures
- [ ] `receive_quiz_cache` method
- [ ] `receive_full_quiz_cache` method
- [ ] Modified `submit_quiz` with local verification
- [ ] Fallback to remote verification
- [ ] Updated Candid interface

---

### Phase 4: Testing & Validation
**Duration**: 2-3 days  
**Risk**: Low

#### Tasks:
1. Create comprehensive test scripts
2. Test cache distribution flow
3. Test local verification
4. Test fallback mechanism
5. Performance benchmarking

#### Test Scenarios:
```bash
# Test 1: Cache distribution
dfx canister call learning_engine add_learning_unit '(...)'
# Verify cache appears in all shards

# Test 2: Local verification
dfx canister call user_profile submit_quiz '("unit_id", vec {0, 1, 2, 0, 1})'
# Should complete without inter-canister call latency

# Test 3: Fallback (cache miss)
# Clear cache, then submit quiz
# Should fall back to remote verification

# Test 4: New shard sync
# Create new shard, verify it receives full cache
```

---

## Data Structure Optimizations

### Recommendation: Separate Content from Quiz Data

For better scalability, consider splitting `LearningUnit`:

```rust
// Large content - stays in learning_engine only
struct LearningContent {
    unit_id: String,
    unit_title: String,
    chapter_id: String,
    chapter_title: String,
    head_unit_id: String,
    head_unit_title: String,
    content: String,      // Large
    paraphrase: String,   // Large
}

// Small quiz data - distributed to shards
struct QuizData {
    questions: Vec<QuizQuestion>,
}

// Combined view for API
struct LearningUnit {
    content: LearningContent,
    quiz: QuizData,
}
```

### Recommendation: Add Hierarchy Indexing

For efficient content browsing, add secondary indexes:

```rust
// Index: chapter_id -> Vec<unit_id>
static CHAPTER_INDEX: RefCell<StableBTreeMap<String, Vec<String>, Memory>> = ...;

// Index: head_unit_id -> Vec<unit_id>
static HEAD_UNIT_INDEX: RefCell<StableBTreeMap<String, Vec<String>, Memory>> = ...;

// Query: Get all units in a chapter
#[query]
fn get_units_by_chapter(chapter_id: String) -> Vec<String> {
    CHAPTER_INDEX.with(|i| i.borrow().get(&chapter_id).unwrap_or_default())
}
```

### Recommendation: Implement Content Sharding Strategy

For 100,000+ units, shard by subject:

```
┌─────────────────────────────────────────────────────────────┐
│                  CONTENT SHARDING STRATEGY                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │ learning_engine │  │ learning_engine │  │ learning_   │  │
│  │    (textile)    │  │   (chemistry)   │  │  engine_N   │  │
│  │                 │  │                 │  │             │  │
│  │ Units 1.x       │  │ Units 2.x       │  │ Units N.x   │  │
│  └────────┬────────┘  └────────┬────────┘  └──────┬──────┘  │
│           │                    │                   │         │
│           └────────────┬───────┴───────────────────┘         │
│                        │                                     │
│                        ▼                                     │
│              ┌─────────────────────┐                        │
│              │  content_registry   │                        │
│              │    (router)         │                        │
│              │                     │                        │
│              │ unit_id → canister  │                        │
│              └─────────────────────┘                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Security Considerations

### Answer Protection

**Problem**: Raw answer indices could be discovered if cached.

**Solution**: Hash-based verification

```rust
fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

// Cache stores hashed answers
struct QuizCache {
    answer_hashes: Vec<[u8; 32]>,  // sha256(answer_index)
    question_count: u8,
}

// Verification compares hashes
fn verify_answer(submitted: u8, stored_hash: [u8; 32]) -> bool {
    sha256(&[submitted]) == stored_hash
}
```

**Why this works**:
- SHA256 is one-way; can't derive answer from hash
- There are only 4 options (0-3), so technically brute-forceable
- BUT: This is already the case with current public quiz display
- Users already see the options; security is in not knowing which is correct

**Enhanced security** (optional):
```rust
// Add salt per question
struct QuizCache {
    answer_hashes: Vec<[u8; 32]>,  // sha256(answer_index + salt)
    salts: Vec<[u8; 16]>,          // Random salt per question
    question_count: u8,
}
```

### Authorization

All cache update endpoints must verify caller:

```rust
#[update]
fn receive_quiz_cache(...) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let authorized = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    if caller != authorized {
        return Err("Unauthorized: Only staking_hub can update cache".to_string());
    }
    
    // ... proceed
}
```

---

## Migration Strategy

### Phase A: Parallel Operation (Zero Downtime)
1. Deploy updated `learning_engine` with versioning
2. Deploy updated `staking_hub` with distribution logic
3. Deploy updated `user_profile` with cache + fallback
4. **Both paths work**: Cache hit → local, Cache miss → remote

### Phase B: Cache Population
1. Trigger full sync for all existing shards
2. Verify cache integrity across shards
3. Monitor cache hit rate

### Phase C: Optimization
1. Analyze cache hit rate metrics
2. Remove fallback if 99%+ cache hits
3. Add cache refresh timers if needed

### Rollback Plan
1. Revert `user_profile` to always use remote verification
2. No data loss; cache is supplementary

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Cache hit rate | >99% |
| Quiz verification latency | <5ms |
| Cache propagation time | <30 seconds |
| Zero failed quiz submissions due to cache issues | 100% |

---

## Timeline Summary

| Phase | Duration | Focus |
|-------|----------|-------|
| Phase 1 | 1-2 days | Learning Engine enhancements |
| Phase 2 | 1-2 days | Staking Hub distribution |
| Phase 3 | 2-3 days | User Profile cache + local verification |
| Phase 4 | 2-3 days | Testing & validation |
| **Total** | **6-10 days** | Full implementation |

---

## Appendix: Code Snippets

### SHA256 Implementation for IC

```rust
// Add to Cargo.toml:
// sha2 = "0.10"

use sha2::{Sha256, Digest};

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}
```

### QuizCache Storable Implementation

```rust
use candid::{Encode, Decode};

#[derive(CandidType, Deserialize, Clone)]
struct QuizCache {
    answer_hashes: Vec<[u8; 32]>,
    question_count: u8,
}

impl Storable for QuizCache {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode QuizCache")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1024,  // 32 bytes × 10 questions max + overhead
        is_fixed_size: false,
    };
}
```

---

**Document Status**: Ready for Review  
**Author**: AI Assistant  
**Reviewers**: @vladknd
