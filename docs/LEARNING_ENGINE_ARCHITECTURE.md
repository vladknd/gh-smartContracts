# Learning Engine Architecture

**Last Updated**: January 2026  
**Status**: Planning Phase  
**Priority**: High - Foundation for Scalable Content System

---

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Canister Architecture](#canister-architecture)
3. [Flexible Content Data Structure](#flexible-content-data-structure)
4. [Quiz Index for Efficient Lookup](#quiz-index-for-efficient-lookup)
5. [Quiz Caching in User Profile Shards](#quiz-caching-in-user-profile-shards)
6. [Off-Chain Indexer Architecture](#off-chain-indexer-architecture)
7. [Content Loading Flow](#content-loading-flow)
8. [Resilient Content Loading](#resilient-content-loading)
9. [Implementation Plan](#implementation-plan)

---

## Executive Summary

### Key Design Principles
1. **Flexible content structure** - Generic tree of content nodes supporting any hierarchy depth
2. **Direct shard-to-engine communication** - User profile shards query learning engine directly (no staking_hub intermediary for quiz data)
3. **Pull-based caching** - Shards fetch quiz data on-demand, cache locally with TTL
4. **Separate asset canisters** - Media files in asset canister, metadata in regular canister
5. **Resilient loading** - Content loading survives crashes and upgrades

### Canister Overview

| Canister | Type | Purpose |
|----------|------|---------|
| `frontend_assets` | Asset | Web app files (React, etc.) |
| `media_assets` | Asset | Videos, audio, images for courses |
| `staging_assets` | Asset | Temporary storage for content awaiting approval |
| `learning_engine` | Regular | Content metadata, quiz logic, quiz index |
| `governance_canister` | Regular | Proposals, voting |
| `treasury_canister` | Regular | Token management |
| `staking_hub` | Regular | Shard management (NOT involved in quiz flow) |
| `user_profile_*` | Regular | User data, quiz cache, local verification |

---

## Canister Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    COMPLETE CANISTER ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ASSET CANISTERS (3):                                                       │
│   ════════════════════                                                       │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  1. frontend_assets                                                  │   │
│   │     • Your React/web app                                            │   │
│   │     • Served at: https://your-app.icp0.io/                          │   │
│   │     • Permanent, updated on deploys                                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  2. media_assets                                                     │   │
│   │     • Approved videos, audio, images                                │   │
│   │     • Served at: https://media-xxx.icp0.io/videos/lesson1.mp4      │   │
│   │     • Permanent storage for approved content                        │   │
│   │     • learning_engine stores URLs pointing here                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  3. staging_assets                                                   │   │
│   │     • Content metadata awaiting approval (JSON)                     │   │
│   │     • Temporary - deleted after loading to learning_engine         │   │
│   │     • Authors upload here before creating proposal                  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   REGULAR CANISTERS:                                                        │
│   ══════════════════                                                        │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  learning_engine                                                     │   │
│   │     • Content nodes (tree structure)                                │   │
│   │     • Quiz index (for O(1) lookup)                                  │   │
│   │     • Content version tracking                                      │   │
│   │     • Directly queried by user_profile shards                       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  user_profile shards                                                 │   │
│   │     • User data, balances                                           │   │
│   │     • Local quiz cache (pulled from learning_engine)               │   │
│   │     • Local verification (no inter-canister call for verification) │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why Asset Canister vs Regular Canister?

| Aspect | Asset Canister | Regular Canister |
|--------|----------------|------------------|
| **Purpose** | Serve static files over HTTP | Run business logic |
| **Code** | Pre-built by DFINITY | Your custom Rust/Motoko |
| **HTTP** | Serves files directly to browsers | No HTTP (unless you add handler) |
| **Use for** | Frontend, videos, images | Learning engine, user profile, governance |

**Learning engine is NOT an asset canister** - it needs custom quiz verification and content logic.

---

## Flexible Content Data Structure

### Content Node: The Universal Building Block

Every piece of content is a `ContentNode`. Nodes link together via `parent_id` to form a tree.

```rust
/// The universal content node - can represent any content type
#[derive(CandidType, Deserialize, Clone)]
struct ContentNode {
    // Identity
    id: String,                    // Unique ID like "book:1:ch:2:sec:3"
    
    // Tree Structure
    parent_id: Option<String>,     // Points to parent (None = root)
    order: u32,                    // Order among siblings (1, 2, 3...)
    
    // Display Info (call it whatever you want!)
    display_type: String,          // "Book", "Chapter", "Unit", "Lesson", "Module", etc.
    title: String,                 // "Chapter I: Consumers, Be Aware!"
    description: Option<String>,   // Optional description
    
    // Content (optional - containers don't need it)
    content: Option<String>,       // Main text/markdown
    paraphrase: Option<String>,    // Summary
    
    // Media (optional - just URLs, files stored in asset canister)
    media: Option<MediaContent>,
    
    // Quiz (optional - ANY node at ANY level can have a quiz)
    quiz: Option<QuizData>,
    
    // Metadata
    created_at: u64,
    updated_at: u64,
    version: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct MediaContent {
    media_type: MediaType,
    url: String,                    // URL to asset canister or CDN
    thumbnail_url: Option<String>,
    duration_seconds: Option<u32>,  // For video/audio
    file_hash: Option<String>,      // For verification
}

#[derive(CandidType, Deserialize, Clone)]
enum MediaType {
    Video,
    Audio,
    Image,
    PDF,
}

/// Quiz data - just questions (config is GLOBAL, not per-quiz)
#[derive(CandidType, Deserialize, Clone)]
struct QuizData {
    questions: Vec<QuizQuestion>,
    // NOTE: No config here! All quizzes use GLOBAL_QUIZ_CONFIG
}

#[derive(CandidType, Deserialize, Clone)]
struct QuizQuestion {
    question: String,
    options: Vec<String>,
    answer: u8,  // Index of correct option
}

/// Global quiz configuration - ONE config for ALL quizzes (simplest & safest)
#[derive(CandidType, Deserialize, Clone)]
struct QuizConfig {
    reward_amount: u64,            // Tokens for completing ANY quiz
    pass_threshold_percent: u8,    // e.g., 60%
    max_daily_attempts: u8,        // e.g., 5 per quiz per day
}
```

### How Nodes Link Together (Tree Structure)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HOW NODES LINK TOGETHER                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Each node points to its PARENT via parent_id:                             │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  id: "book:textile"                                                  │   │
│   │  parent_id: None  ← ROOT (no parent)                                │   │
│   │  display_type: "Book"                                                │   │
│   │  quiz: None  ← No quiz at book level                                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                    │                                                         │
│        ┌───────────┴───────────┐                                            │
│        ▼                       ▼                                            │
│   ┌─────────────────┐    ┌─────────────────┐                               │
│   │ id: "...ch:1"   │    │ id: "...ch:2"   │                               │
│   │ parent_id:      │    │ parent_id:      │                               │
│   │ "book:textile"  │    │ "book:textile"  │  ← Both point to same parent │
│   │ display_type:   │    │ display_type:   │                               │
│   │ "Chapter"       │    │ "Chapter"       │                               │
│   │ quiz: None      │    │ quiz: Some(...) │  ← Chapter CAN have quiz!    │
│   └────────┬────────┘    └─────────────────┘                               │
│            │                                                                 │
│   ┌────────┴────────┐                                                       │
│   ▼                 ▼                                                       │
│   ┌──────────────┐  ┌──────────────┐                                       │
│   │id: "..sec:1" │  │id: "..sec:2" │                                       │
│   │parent_id:    │  │parent_id:    │                                       │
│   │"...ch:1"     │  │"...ch:1"     │  ← Both point to Chapter 1           │
│   │display_type: │  │display_type: │                                       │
│   │"Section"     │  │"Topic"       │  ← Call it anything!                 │
│   │quiz:Some(...)│  │quiz:Some(...)│  ← Both have quizzes                 │
│   └──────────────┘  └──────────────┘                                       │
│                                                                              │
│   KEY INSIGHTS:                                                              │
│   • Any node can have a quiz (quiz is Optional)                             │
│   • display_type is just a label - call it anything                         │
│   • Unlimited hierarchy depth - just keep adding parent_id links            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Storage in Learning Engine

```rust
thread_local! {
    // All content nodes in one map
    static CONTENT_NODES: RefCell<StableBTreeMap<String, ContentNode, Memory>> = ...;
    
    // Index: parent_id -> list of child IDs (for tree traversal)
    static CHILDREN_INDEX: RefCell<StableBTreeMap<String, Vec<String>, Memory>> = ...;
    
    // Quiz index: content_id -> quiz cache data (for O(1) lookup)
    static QUIZ_INDEX: RefCell<StableBTreeMap<String, QuizCacheData, Memory>> = ...;
    
    // GLOBAL quiz config - ONE config for ALL quizzes (simplest & safest!)
    static GLOBAL_QUIZ_CONFIG: RefCell<StableCell<QuizConfig, Memory>> = ...;
    
    // Global version (increments on any change)
    static CONTENT_VERSION: RefCell<StableCell<u64, Memory>> = ...;
    
    // Loading jobs (for resilient content loading)
    static LOADING_JOBS: RefCell<StableBTreeMap<u64, LoadingJob, Memory>> = ...;
}
```

**Note**: `GLOBAL_QUIZ_CONFIG` contains the reward, pass threshold, and daily attempt limit for ALL quizzes. This is the simplest and safest approach - any change to rewards requires a governance proposal to update this single config.


---

## Quiz Index for Efficient Lookup

### Problem: Traversing All Nodes is O(n)

Since quizzes can be at any level, we need efficient lookup without scanning all nodes.

### Solution: Maintain a Separate Quiz Index

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    QUIZ INDEX (O(1) lookup!)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONTENT_NODES: Map<id, ContentNode>    ← All content                      │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  "book:textile"           → ContentNode { quiz: None }              │   │
│   │  "book:textile:ch:1"      → ContentNode { quiz: None }              │   │
│   │  "book:textile:ch:1:u:1"  → ContentNode { quiz: Some(...) }         │   │
│   │  "book:textile:ch:1:u:2"  → ContentNode { quiz: Some(...) }         │   │
│   │  "book:textile:ch:2"      → ContentNode { quiz: Some(...) }         │   │
│   │  ...hundreds more...                                                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   QUIZ_INDEX: Map<id, QuizCacheData>     ← ONLY quizzes (fast!)            │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  "book:textile:ch:1:u:1"  → QuizCacheData { hashes, config }        │   │
│   │  "book:textile:ch:1:u:2"  → QuizCacheData { hashes, config }        │   │
│   │  "book:textile:ch:2"      → QuizCacheData { hashes, config }        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ✅ get_quiz_data(id): O(1) direct lookup in QUIZ_INDEX                   │
│   ✅ No traversal needed!                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Quiz Index Updated at Insert Time

```rust
/// Add or update a content node
fn add_content_node_internal(node: ContentNode) -> Result<(), String> {
    let id = node.id.clone();
    
    // 1. Add to main content map - O(1)
    CONTENT_NODES.with(|c| c.borrow_mut().insert(id.clone(), node.clone()));
    
    // 2. Update children index - O(1)
    if let Some(parent_id) = &node.parent_id {
        CHILDREN_INDEX.with(|idx| {
            let mut index = idx.borrow_mut();
            let mut children = index.get(parent_id).unwrap_or_default();
            if !children.contains(&id) {
                children.push(id.clone());
                index.insert(parent_id.clone(), children);
            }
        });
    }
    
    // 3. If has quiz, update quiz index - O(1)
    if let Some(quiz) = &node.quiz {
        let cache_data = QuizCacheData {
            content_id: id.clone(),
            answer_hashes: quiz.questions.iter()
                .map(|q| sha256(&q.answer.to_le_bytes()))
                .collect(),
            question_count: quiz.questions.len() as u8,
            // NOTE: No config here - we use GLOBAL_QUIZ_CONFIG
            version: increment_version(),
        };
        QUIZ_INDEX.with(|q| q.borrow_mut().insert(id, cache_data));
    } else {
        // Remove from quiz index if quiz was removed
        QUIZ_INDEX.with(|q| q.borrow_mut().remove(&id));
    }
    
    Ok(())
}
```

---

## Quiz Caching in User Profile Shards

### Architecture: Direct Communication (No Staking Hub)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    QUIZ DATA FLOW: DIRECT CONNECTION                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      learning_engine                                 │   │
│   │                   (Single Source of Truth)                           │   │
│   │                                                                      │   │
│   │   CONTENT_NODES: Map<ContentId, ContentNode>                        │   │
│   │   QUIZ_INDEX: Map<ContentId, QuizCacheData>                         │   │
│   │                                                                      │   │
│   │   Queries (FREE):                                                    │   │
│   │   • get_quiz_data(id) → answer_hashes + config                      │   │
│   │   • get_content_version() → u64                                     │   │
│   └──────────────────────────────▲──────────────────────────────────────┘   │
│                                  │                                          │
│                    Query (FREE!) │                                          │
│             ┌────────────────────┼────────────────────┐                     │
│             │                    │                    │                     │
│             ▼                    ▼                    ▼                     │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  user_profile   │  │  user_profile   │  │  user_profile   │            │
│   │    shard 1      │  │    shard 2      │  │    shard N      │            │
│   │                 │  │                 │  │                 │            │
│   │ LOCAL_CACHE     │  │ LOCAL_CACHE     │  │ LOCAL_CACHE     │            │
│   │ (pull on demand)│  │ (pull on demand)│  │ (pull on demand)│            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                              │
│   staking_hub is NOT involved in quiz data flow!                            │
│   It only handles: shard management, minting allowance, stats               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why Direct Connection is Better

| Aspect | Via Staking Hub (Old) | Direct Connection (New) |
|--------|----------------------|------------------------|
| **Complexity** | High - extra hop | Low - direct query |
| **Load on staking_hub** | Unnecessary load | Zero quiz-related load |
| **Reward changes** | Must push to all shards | Instant - fetch current config |
| **New shards** | Must sync manually | Auto-fetch on first request |

### User Profile Shard: Pull-on-Demand Quiz Cache

```rust
// In user_profile shard

#[update]
async fn submit_quiz(content_id: String, answers: Vec<u8>) -> Result<QuizResult, String> {
    let user = ic_cdk::caller();
    
    // 1. Get quiz cache (answer hashes) from learning_engine
    let cache = get_or_fetch_cache(&content_id).await?;
    
    // 2. Get GLOBAL config (same for all quizzes)
    let config = get_or_fetch_global_config().await?;
    
    // 3. Verify answers locally (no inter-canister call!)
    let correct_count = verify_answers(&answers, &cache.answer_hashes);
    
    // 4. Check pass (using GLOBAL config)
    let passed = (correct_count * 100) / cache.question_count as u64 
                 >= config.pass_threshold_percent as u64;
    
    // 5. Apply reward (GLOBAL config.reward_amount - same for all quizzes!)
    let reward = if passed && !is_completed(&content_id) {
        mark_completed(&content_id);
        add_balance(config.reward_amount);
        config.reward_amount
    } else {
        0
    };
    
    Ok(QuizResult { passed, correct_count, reward_earned: reward, .. })
}

async fn get_or_fetch_cache(content_id: &str) -> Result<QuizCacheData, String> {
    // Check local cache with TTL
    if let Some(cached) = LOCAL_QUIZ_CACHE.with(|c| c.borrow().get(content_id)) {
        if is_cache_fresh(&cached) {
            return Ok(cached.data);
        }
    }
    
    // Cache miss or stale: fetch from learning_engine (QUERY = FREE!)
    let cache: QuizCacheData = ic_cdk::call(
        LEARNING_ENGINE_ID,
        "get_quiz_data",
        (content_id.to_string(),)
    ).await.map_err(|e| format!("Failed to fetch: {:?}", e))?;
    
    // Store locally with timestamp
    LOCAL_QUIZ_CACHE.with(|c| {
        c.borrow_mut().insert(content_id.to_string(), CachedQuiz {
            data: cache.clone(),
            fetched_at: ic_cdk::api::time(),
        });
    });
    
    Ok(cache)
}

async fn get_or_fetch_global_config() -> Result<QuizConfig, String> {
    // Check local cache
    if let Some(cached) = CACHED_GLOBAL_CONFIG.with(|c| c.borrow().clone()) {
        if is_config_fresh(&cached) {
            return Ok(cached.config);
        }
    }
    
    // Fetch global config (QUERY = FREE!)
    let config: QuizConfig = ic_cdk::call(
        LEARNING_ENGINE_ID,
        "get_global_quiz_config",
        ()
    ).await.map_err(|e| format!("Failed to fetch config: {:?}", e))?;
    
    // Cache locally
    CACHED_GLOBAL_CONFIG.with(|c| {
        *c.borrow_mut() = Some(CachedConfig {
            config: config.clone(),
            fetched_at: ic_cdk::api::time(),
        });
    });
    
    Ok(config)
}

const CACHE_TTL_NS: u64 = 3600_000_000_000; // 1 hour

fn is_cache_fresh(cached: &CachedQuiz) -> bool {
    ic_cdk::api::time() - cached.fetched_at < CACHE_TTL_NS
}
```

---

## Off-Chain Indexer Architecture

For content browsing and full-text search, use an off-chain indexer:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PULL-BASED INDEXER ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        INTERNET COMPUTER                             │   │
│   │                                                                      │   │
│   │   ┌─────────────────┐      ┌─────────────────┐                      │   │
│   │   │ learning_engine │      │  user_profile   │                      │   │
│   │   │                 │      │    (shards)     │                      │   │
│   │   │ • Content nodes │      │                 │                      │   │
│   │   │ • Quiz index    │      │ • Quiz cache    │                      │   │
│   │   │ • Version       │◄────►│ • Local verify  │                      │   │
│   │   │                 │query │                 │                      │   │
│   │   └────────▲────────┘      └─────────────────┘                      │   │
│   │            │                                                         │   │
│   └────────────┼─────────────────────────────────────────────────────────┘   │
│                │ Query (FREE)                                               │
│   ┌────────────┼───────────────────────────────────────────────────────────┐ │
│   │            │         OFF-CHAIN LAYER                                   │ │
│   │   ┌────────┴─────────┐                                                │ │
│   │   │  Indexer Service │                                                │ │
│   │   │  • Polls every N minutes                                         │ │
│   │   │  • Detects new content via CONTENT_VERSION                       │ │
│   │   │  • Syncs to database                                             │ │
│   │   └────────┬─────────┘                                                │ │
│   │            │                                                          │ │
│   │            ▼                                                          │ │
│   │   ┌──────────────────┐                                                │ │
│   │   │  Database        │                                                │ │
│   │   │  (Supabase/Redis)│                                                │ │
│   │   │  • Full-text search                                              │ │
│   │   │  • Fast queries                                                  │ │
│   │   └──────────────────┘                                                │ │
│   │                                                                        │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│   Frontend:                                                                  │
│   • Browse content → Off-chain DB (fast)                                    │
│   • Full-text search → Off-chain DB (fast)                                  │
│   • Submit quiz → user_profile shard (on-chain)                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Content Loading Flow

### Two-Phase Upload

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CONTENT UPLOAD FLOW                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   PHASE 1: Upload BEFORE Proposal                                           │
│   ═══════════════════════════════                                            │
│                                                                              │
│   1. Author uploads MEDIA to media_assets canister                          │
│      videos/lesson1.mp4 → https://media-xxx.icp0.io/videos/lesson1.mp4     │
│                                                                              │
│   2. Author uploads METADATA (JSON) to staging_assets canister              │
│      500 content nodes → staging_assets:/book1.json                         │
│      Returns: content_hash = "sha256:abc123..."                             │
│                                                                              │
│   PHASE 2: Proposal (Tiny - Just Reference)                                 │
│   ═════════════════════════════════════════                                  │
│                                                                              │
│   3. Author creates proposal with just:                                     │
│      • content_hash: "sha256:abc123..."                                     │
│      • staging_path: "/book1.json"                                          │
│      • total_units: 500                                                     │
│                                                                              │
│   4. Board votes → Approved ✓                                               │
│                                                                              │
│   PHASE 3: Automatic Loading                                                │
│   ═══════════════════════════                                                │
│                                                                              │
│   5. governance_canister calls learning_engine.start_content_load()         │
│                                                                              │
│   6. learning_engine uses self-calling pattern:                             │
│      • Fetch 10 units from staging                                          │
│      • Process and save                                                     │
│      • Call self to continue                                                │
│      • Repeat until done                                                    │
│                                                                              │
│   7. Staging content deleted after loading complete                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Resilient Content Loading

### Problem: Self-Call Loop Can Break

The self-calling pattern can fail due to:
- Cycle exhaustion
- Network issues
- Canister upgrade
- Bugs

### Solution: Persistent State + Resume

```rust
// Loading job stored in STABLE storage (survives crashes/upgrades)
#[derive(CandidType, Deserialize, Clone)]
struct LoadingJob {
    proposal_id: u64,
    staging_canister: Principal,
    staging_path: String,
    content_hash: String,
    total_units: u32,
    loaded_units: u32,        // ← Progress saved here!
    status: LoadingStatus,
    last_error: Option<String>,
    started_at: u64,
    updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
enum LoadingStatus {
    InProgress,
    Completed,
    Failed,
    Paused,  // Manual pause or error recovery
}
```

### How It Works

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    RESILIENT LOADING FLOW                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. Create job in STABLE storage (loaded_units = 0)                        │
│   2. Fetch chunk (10 units)                                                 │
│   3. Process nodes, save to CONTENT_NODES                                   │
│   4. Update progress in STABLE storage (loaded_units += 10)                 │
│   5. If more remaining → SELF-CALL to continue                              │
│   6. If done → Mark complete                                                │
│                                                                              │
│   IF CRASH/UPGRADE HAPPENS:                                                  │
│   • State is in STABLE storage (survives!)                                  │
│   • post_upgrade() calls resume_incomplete_jobs()                           │
│   • Loading continues automatically from saved progress                     │
│                                                                              │
│   IF ERROR OCCURS:                                                           │
│   • Save error and pause job                                                │
│   • Admin can call resume_loading(proposal_id) to retry                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
/// Start loading (called by governance on approval)
#[update]
async fn start_content_load(
    proposal_id: u64,
    staging_canister: Principal,
    staging_path: String,
    content_hash: String,
) -> Result<(), String> {
    // Get total count
    let total: u32 = ic_cdk::call(staging_canister, "get_content_count", (staging_path.clone(),))
        .await?;
    
    // Create job in STABLE storage
    let job = LoadingJob {
        proposal_id,
        staging_canister,
        staging_path,
        content_hash,
        total_units: total,
        loaded_units: 0,
        status: LoadingStatus::InProgress,
        last_error: None,
        started_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };
    
    LOADING_JOBS.with(|jobs| jobs.borrow_mut().insert(proposal_id, job));
    
    // Start processing
    continue_loading_internal(proposal_id).await
}

/// Continue loading (self-call pattern)
async fn continue_loading_internal(proposal_id: u64) -> Result<(), String> {
    const BATCH_SIZE: u32 = 10;
    
    let job = LOADING_JOBS.with(|jobs| jobs.borrow().get(&proposal_id))
        .ok_or("Job not found")?;
    
    // Fetch next batch
    let batch: Vec<ContentNode> = ic_cdk::call(
        job.staging_canister,
        "get_content_chunk",
        (job.staging_path.clone(), job.loaded_units, BATCH_SIZE)
    ).await?;
    
    // Process each node
    for node in &batch {
        add_content_node_internal(node.clone())?;
    }
    
    // Update progress in STABLE storage
    let new_loaded = job.loaded_units + batch.len() as u32;
    let is_complete = new_loaded >= job.total_units;
    
    LOADING_JOBS.with(|jobs| {
        if let Some(mut j) = jobs.borrow().get(&proposal_id) {
            j.loaded_units = new_loaded;
            j.status = if is_complete { LoadingStatus::Completed } else { LoadingStatus::InProgress };
            j.updated_at = ic_cdk::api::time();
            jobs.borrow_mut().insert(proposal_id, j);
        }
    });
    
    if !is_complete {
        // Continue with self-call
        let self_id = ic_cdk::api::id();
        let _ = ic_cdk::call::<_, ()>(self_id, "continue_loading", (proposal_id,)).await;
    }
    
    Ok(())
}

/// Resume after crash/upgrade
#[post_upgrade]
fn post_upgrade() {
    ic_cdk::spawn(async {
        let incomplete: Vec<u64> = LOADING_JOBS.with(|jobs| {
            jobs.borrow()
                .iter()
                .filter(|(_, job)| job.status == LoadingStatus::InProgress)
                .map(|(id, _)| id)
                .collect()
        });
        
        for proposal_id in incomplete {
            let _ = continue_loading_internal(proposal_id).await;
        }
    });
}

/// Manual resume after error
#[update]
async fn resume_loading(proposal_id: u64) -> Result<(), String> {
    LOADING_JOBS.with(|jobs| {
        if let Some(mut j) = jobs.borrow().get(&proposal_id) {
            j.status = LoadingStatus::InProgress;
            j.last_error = None;
            jobs.borrow_mut().insert(proposal_id, j);
        }
    });
    
    continue_loading_internal(proposal_id).await
}

/// Query loading progress
#[query]
fn get_loading_status(proposal_id: u64) -> Option<LoadingJob> {
    LOADING_JOBS.with(|jobs| jobs.borrow().get(&proposal_id))
}
```

---

## Implementation Plan

### Phase 1: Flexible Content Structure (3-4 days)
1. Create `ContentNode`, `QuizData`, `MediaContent` structures
2. Implement `CONTENT_NODES` and `CHILDREN_INDEX` storage
3. Implement `QUIZ_INDEX` with insert-time updates
4. Add content query functions (get_children, get_quiz_data)
5. Update Candid interface

### Phase 2: Quiz Caching in User Profile (2-3 days)
1. Add `LOCAL_QUIZ_CACHE` to user_profile shards
2. Implement pull-on-demand with TTL
3. Update `submit_quiz` for local verification
4. Remove staking_hub from quiz data flow

### Phase 3: Asset Canisters Setup (1-2 days)
1. Deploy `media_assets` canister
2. Deploy `staging_assets` canister
3. Implement chunked upload for staging

### Phase 4: Resilient Content Loading (2-3 days)
1. Add `LOADING_JOBS` stable storage
2. Implement `start_content_load` with self-calling
3. Implement `resume_loading` and `post_upgrade` auto-resume
4. Add progress monitoring queries

### Phase 5: Content Governance Integration (2-3 days)
1. Update governance for content proposals
2. Connect proposal execution to loading flow
3. Test end-to-end flow

### Phase 6: Off-Chain Indexer (2-3 days)
1. Create indexer service
2. Set up database
3. Implement polling and sync

**Total**: 12-18 days

---

## Cost Estimates

| Component | Monthly Cost |
|-----------|--------------|
| Storage (5GB content + media) | ~$0.10 |
| Timer-based processing (occasional) | ~$0.05 |
| Query calls | FREE |
| Update calls (user submissions) | ~$1-5 (depends on usage) |
| **Total** | **$2-6/month** |

**Note**: Using timers instead of heartbeat saves ~$3/month of constant compute costs.

---

## Related Documents

- [Content Governance](./CONTENT_GOVERNANCE.md) - Proposal-based content modifications
- [Quiz Caching Implementation Plan](./QUIZ_CACHING_IMPLEMENTATION_PLAN.md) - Detailed caching strategy
- [Scalability Plan](./scalability_plan.md) - Sharded architecture overview

---

**Document Status**: Ready for Review  
**Author**: AI Assistant  
**Reviewers**: @vladknd
