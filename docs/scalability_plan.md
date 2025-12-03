# Scalability & Architecture Plan: "Limitless" Growth

## 1. The Bottlenecks of the Current System
Currently, your architecture places user state and content in the same canisters. This works for thousands of users but will fail at millions due to:
- **Storage Limits**: While `StableBTreeMap` is efficient, a single canister is capped (currently ~400GB, but practical performance limits exist).
- **Throughput Limits**: A single canister can process only a limited number of update calls (writes) per second. If 1 million users try to submit a quiz at the same time, the `learning_engine` will halt.
- **Cycle Limits**: Managing cycles for a massive monolithic canister is risky.

## 2. Proposed Solution: The "Sharded User" Architecture

To handle **limitless users**, we must decouple **Content** (Global, Read-Heavy) from **User State** (Personal, Write-Heavy).

### A. The `learning_content` Canister (formerly `learning_engine`)
**Role**: The Source of Truth for Education.
- **Stores**: Learning Units, Quizzes, Correct Answers.
- **Responsibility**: 
    - Serve content to the frontend.
    - Verify answers (stateless verification).
- **Scaling**: 
    - Since this is mostly **Read-Only** (users reading), it scales well via **HTTP Caching** and **Query Calls** (which are fast and cheap).
    - If content grows too large (e.g., thousands of books), we shard by **Subject** (e.g., `content_math`, `content_history`).

### B. The `user_profile` Canisters (New!)
**Role**: The User's Personal Record.
- **Stores**: 
    - User Profile (Email, Name, Gender, Education).
    - Quiz Progress (List of completed Unit IDs).
    - Daily Stats (Attempts today, Tokens earned).
- **Scaling**: **Dynamic Sharding**.
    - We do not have just one `user_profile` canister. We have **many**.
    - **User Indexing**: A simple logic (or a `registry` canister) maps a User Principal to their specific `user_profile` canister.
    - Example: Users starting with `a-m` go to Canister 1, `n-z` go to Canister 2. Or simply spawn a new canister every 100,000 users.

### C. The Flow
1. **User Login**: Frontend determines which `user_profile` canister belongs to the user.
2. **Submit Quiz**:
    - Frontend sends answers to the user's `user_profile` canister.
    - `user_profile` checks Daily Limits.
    - `user_profile` calls `learning_content.verify_quiz(answers)`.
    - If valid: `user_profile` updates progress and calls `staking_hub` to mint rewards.

## 3. Implementation Steps

### Step 1: Create `user_profile` Canister
We will create a new canister that holds the logic currently inside `learning_engine` related to:
- `COMPLETED_QUIZZES`
- `USER_DAILY_STATS`
- New fields: `email`, `name`, `education`, `gender`.

### Step 2: Refactor `learning_engine`
- Rename to `learning_content` (conceptually).
- Remove `COMPLETED_QUIZZES` and `USER_DAILY_STATS`.
- Expose a public `verify_quiz_answers` method (can be guarded so only valid `user_profile` canisters can call it, or kept open if answers aren't leaked).

### Step 3: Define Sharding Strategy
For now, we can start with **one** `user_profile` canister. The code will be designed such that deploying a second one is trivial (same Wasm). We will add a `Manager` or `Registry` logic in the frontend or a small canister to route users.

## 4. Data Structures for `user_profile`

```rust
struct UserProfile {
    // Personal Data
    email: String,
    name: String,
    education: String,
    gender: String,
    
    // App State
    completed_quizzes: Vec<String>, // Unit IDs
    daily_stats: UserDailyStats,
}
```

## 5. Summary
- **Sharding Learning Engine?** -> Yes, split Content from User State.
- **Separate Canister for User State?** -> Yes, this is critical for "limitless" scaling.
