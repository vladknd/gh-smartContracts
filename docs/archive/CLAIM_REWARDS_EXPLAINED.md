# Understanding User Interest Claims - Correcting Misconceptions

## ❌ Common Misconceptions

Let's address the misconceptions in your question:

### Misconception 1: "cumulative_reward_index changes for each user"
**❌ FALSE**

`cumulative_reward_index` is a **SINGLE GLOBAL VALUE** stored in the `staking_hub` canister. It's the same for everyone.

### Misconception 2: "Each user has their own cumulative_reward_index"
**❌ FALSE** 

Each user has a `last_reward_index` field (not `cumulative_reward_index`). This is different.

### Misconception 3: "User's index turns to 0 after claiming"
**❌ FALSE**

The user's `last_reward_index` is **updated to match the current global index**, not reset to 0.

---

## ✅ The Correct Model

### Two Different Indexes:

| Index | Location | Scope | Purpose |
|-------|----------|-------|---------|
| `cumulative_reward_index` | `staking_hub` | **GLOBAL** | Single value shared by everyone |
| `last_reward_index` | `user_profile` | **PER-USER** | Tracks when user last calculated interest |

---

## Step-by-Step: What Actually Happens

### Initial State

```rust
// In staking_hub (GLOBAL - everyone sees same value)
cumulative_reward_index = 0

// In user_profile for User A (PERSONAL)
User A {
    staked_balance: 50 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0  // ← This is PER-USER
}

// In user_profile for User B (PERSONAL)
User B {
    staked_balance: 50 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0
}
```

---

### Step 1: Someone Unstakes (Penalty Collected)

```rust
// User C unstakes 100 GHC → 10 GHC penalty to pool

// In staking_hub
interest_pool = 10 GHC
cumulative_reward_index = 0  // unchanged yet
```

**User A and User B:** No changes yet

---

### Step 2: Admin Calls `distribute_interest()`

```rust
// In staking_hub
interest_pool = 0                    // ← Emptied
cumulative_reward_index = 0.1        // ← GLOBAL VALUE CHANGED
                                     //   (10 GHC / 100 GHC total_staked)
```

**User A state:** Still unchanged (they haven't synced yet)
```rust
User A {
    staked_balance: 50 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0  // ← Still 0! (outdated)
}
```

**User B state:** Also unchanged
```rust
User B {
    staked_balance: 50 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0  // ← Still 0! (outdated)
}
```

**Key Point:** `distribute_interest()` **ONLY** updates the global index in `staking_hub`. It does **NOT** update any individual user profiles.

---

### Step 3: User A Calls `claim_rewards()` (or any update function)

When User A calls `claim_rewards()`, it triggers `compound_interest()` first:

```rust
fn compound_interest(user: Principal) {
    let global_index = 0.1;  // Fetch from staking_hub (synced every 5s)
    
    let mut profile = get_user_profile(user);  // User A's profile
    
    // Calculate how much index has changed since user last updated
    if global_index > profile.last_reward_index {
        let index_diff = global_index - profile.last_reward_index;
        //                    0.1      -           0           = 0.1
        
        let interest = (staked_balance * index_diff) / 1e18;
        //             (50 GHC        * 0.1        ) / 1e18  = 5 GHC
        
        profile.unclaimed_interest += 5 GHC;  // Add to pending
        profile.last_reward_index = global_index;  // ← UPDATE TO 0.1 (NOT 0!)
    }
}
```

**After `compound_interest()` but before claiming:**
```rust
User A {
    staked_balance: 50 GHC,           // unchanged
    unclaimed_interest: 5 GHC,        // ← CALCULATED!
    last_reward_index: 0.1             // ← UPDATED to current global index
}
```

**Then `claim_rewards()` executes:**
```rust
fn claim_rewards() {
    profile.staked_balance += profile.unclaimed_interest;
    //                     50 + 5 = 55 GHC
    
    profile.unclaimed_interest = 0;  // Reset to 0
    
    // last_reward_index stays at 0.1 (already updated by compound_interest)
}
```

**Final state for User A:**
```rust
User A {
    staked_balance: 55 GHC,           // ← INCREASED (50 → 55)
    unclaimed_interest: 0,             // ← RESET to 0
    last_reward_index: 0.1             // ← Still 0.1 (NOT reset to 0!)
}
```

---

### Step 4: User B Claims Later

User B still has outdated index:
```rust
User B {
    staked_balance: 50 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0  // ← Still old value
}
```

When User B calls `claim_rewards()`:

```rust
// compound_interest() runs first
index_diff = 0.1 - 0 = 0.1  // Same calculation as User A
interest = 50 * 0.1 = 5 GHC
```

**After claiming:**
```rust
User B {
    staked_balance: 55 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0.1  // ← Updated to current global index
}
```

---

## Visual Timeline

```
Time →

T0: Initial State
┌─────────────────────────────────────────────────────────┐
│ STAKING HUB (Global)                                    │
│   cumulative_reward_index = 0                           │
├─────────────────────────────────────────────────────────┤
│ USER A (Personal)           │ USER B (Personal)         │
│   staked_balance = 50       │   staked_balance = 50     │
│   unclaimed_interest = 0    │   unclaimed_interest = 0  │
│   last_reward_index = 0     │   last_reward_index = 0   │
└─────────────────────────────────────────────────────────┘

T1: distribute_interest() called
┌─────────────────────────────────────────────────────────┐
│ STAKING HUB (Global)                                    │
│   cumulative_reward_index = 0.1  ← CHANGED!             │
├─────────────────────────────────────────────────────────┤
│ USER A (Personal)           │ USER B (Personal)         │
│   staked_balance = 50       │   staked_balance = 50     │
│   unclaimed_interest = 0    │   unclaimed_interest = 0  │
│   last_reward_index = 0     │   last_reward_index = 0   │
│   (outdated!)               │   (outdated!)             │
└─────────────────────────────────────────────────────────┘

T2: User A claims rewards
┌─────────────────────────────────────────────────────────┐
│ STAKING HUB (Global)                                    │
│   cumulative_reward_index = 0.1  (unchanged)            │
├─────────────────────────────────────────────────────────┤
│ USER A (Personal)           │ USER B (Personal)         │
│   staked_balance = 55  ←!   │   staked_balance = 50     │
│   unclaimed_interest = 0    │   unclaimed_interest = 0  │
│   last_reward_index = 0.1←! │   last_reward_index = 0   │
│   (synced!)                 │   (still outdated)        │
└─────────────────────────────────────────────────────────┘

T3: User B claims rewards
┌─────────────────────────────────────────────────────────┐
│ STAKING HUB (Global)                                    │
│   cumulative_reward_index = 0.1  (unchanged)            │
├─────────────────────────────────────────────────────────┤
│ USER A (Personal)           │ USER B (Personal)         │
│   staked_balance = 55       │   staked_balance = 55  ←! │
│   unclaimed_interest = 0    │   unclaimed_interest = 0  │
│   last_reward_index = 0.1   │   last_reward_index = 0.1←│
│   (synced)                  │   (now synced too!)       │
└─────────────────────────────────────────────────────────┘
```

---

## The Math Behind Index Difference

### Why We Use `index_diff` Instead of Reset to 0

The beauty of this system is that users can claim **at different times** and still get the correct amount.

**Example: Second Distribution**

Let's say User A claims, then another distribution happens, then User A claims again:

```rust
// T0: Initial
cumulative_reward_index = 0
User A.last_reward_index = 0

// T1: First distribution
cumulative_reward_index = 0.1
User A claims → last_reward_index = 0.1, earns 5 GHC

// T2: Second distribution (another 10 GHC distributed)
cumulative_reward_index = 0.1 + 0.1 = 0.2

// T3: User A claims again
index_diff = 0.2 - 0.1 = 0.1  // ← Only counts NEW increase!
interest = 55 * 0.1 = 5.5 GHC  // (now on 55 GHC balance)

User A.last_reward_index = 0.2  // ← Update to new global value
```

**If we reset to 0, this would break:**
```rust
// ❌ WRONG: If we reset to 0
User A.last_reward_index = 0  // After first claim

// T2: Second distribution
cumulative_reward_index = 0.2

// T3: User A claims
index_diff = 0.2 - 0 = 0.2  // ← WRONG! Counts BOTH distributions!
interest = 55 * 0.2 = 11 GHC  // ← Double-counted! Should be 5.5!
```

---

## Summary of How It Actually Works

### When `distribute_interest()` Runs:

```
✅ cumulative_reward_index (GLOBAL) increases
❌ User profiles are NOT touched
❌ Individual balances do NOT change yet
```

### When a User Calls `claim_rewards()`:

```
1. compound_interest() runs first:
   ✅ Calculate: index_diff = global_index - last_reward_index
   ✅ Calculate: interest = staked_balance * index_diff / 1e18
   ✅ Add to: unclaimed_interest += interest
   ✅ Update: last_reward_index = global_index (NOT 0!)

2. claim_rewards() completes:
   ✅ Move: staked_balance += unclaimed_interest
   ✅ Reset: unclaimed_interest = 0
   ✅ Keep: last_reward_index (stays at global value)
```

---

## Corrected Answers to Your Questions

### Q1: "Can we say that after distribute_interest() runs, cumulative_reward_index changed for each user?"

**A: NO.** There is only **ONE** `cumulative_reward_index` in the entire system (stored in `staking_hub`). It's a global value, not per-user.

What DOES change per-user is when they **sync** their `last_reward_index` to catch up with the global value (when they claim or interact).

---

### Q2: "How does staked_balance increase?"

**A: By moving unclaimed_interest to staked_balance:**

```rust
staked_balance += unclaimed_interest;
//    50       +         5          = 55
```

The `unclaimed_interest` was calculated based on:
- How much was staked (`staked_balance`)
- How much the global index increased (`index_diff`)

---

### Q3: "Does cumulative_reward_index turn to 0 after claiming?"

**A: NO.** 

- `cumulative_reward_index` (global) **never resets** - it only goes up
- `last_reward_index` (per-user) **updates to match global** (NOT to 0)
- `unclaimed_interest` (per-user) **resets to 0** ← This is what resets!

---

## Code Reference

### In `staking_hub/src/lib.rs`:
```rust
// GLOBAL VALUE - Single value for entire system
GLOBAL_STATS {
    cumulative_reward_index: u128,  // ← ONE value
    interest_pool: u64,
    total_staked: u64,
}
```

### In `user_profile/src/lib.rs`:
```rust
// PER-USER VALUE - Each user has their own
struct UserProfile {
    staked_balance: u64,
    unclaimed_interest: u64,
    last_reward_index: u128,  // ← Each user tracks their own
}
```

### Key Function - `compound_interest()`:
```rust
fn compound_interest(user: Principal) {
    let global_index = GLOBAL_REWARD_INDEX.with(|i| *i.borrow().get());
    
    if global_index > profile.last_reward_index {
        let index_diff = global_index - profile.last_reward_index;
        let interest = (profile.staked_balance as u128 * index_diff) / 1e18;
        
        profile.unclaimed_interest += interest as u64;
        profile.last_reward_index = global_index;  // ← Sync to global (NOT 0)
    }
}
```

---

## Testing to Verify

Run these commands to see the actual values:

```bash
# 1. Check global index (same for everyone)
dfx canister call staking_hub get_global_stats

# 2. Check User A's personal index
dfx canister call user_profile get_profile '(principal "user-a-principal")'
# Look for: last_reward_index (this is User A's personal tracker)

# 3. Distribute interest
./distribute_interest.sh

# 4. Check global index again (should be increased)
dfx canister call staking_hub get_global_stats

# 5. Check User A again (last_reward_index still outdated until they interact)
dfx canister call user_profile get_profile '(principal "user-a-principal")'

# 6. User A claims
dfx canister call user_profile claim_rewards

# 7. Check User A final state (last_reward_index now matches global)
dfx canister call user_profile get_profile '(principal "user-a-principal")'
```

---

## Conceptual Model

Think of it like a **water meter**:

- **Global Index** = Total water that has flowed through the city pipes (always increasing)
- **Last Reward Index** = Your meter reading last time you checked
- **Index Diff** = How much water flowed since your last reading
- **Your Share** = Your portion based on your stake percentage

When you "claim", you're not resetting the city's total water flow to 0—you're just updating your personal meter reading to match the current city total!
