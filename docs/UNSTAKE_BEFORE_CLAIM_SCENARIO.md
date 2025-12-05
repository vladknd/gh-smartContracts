# Scenario: Unstaking BEFORE Claiming Interest

## Your Concern

> "If cumulative_reward_index was updated, then user unstaked most tokens, and later claims interest - won't they get LESS interest than if they claimed first?"

## The Answer: NO! They Get Full Interest

The code has a **critical safeguard** that prevents this problem.

---

## Step-by-Step Scenario

### Initial State

```rust
User Alice {
    staked_balance: 100 GHC,
    unclaimed_interest: 0,
    last_reward_index: 0
}

// GLOBAL
cumulative_reward_index = 0
```

---

### Step 1: Admin Calls `distribute_interest()`

Someone unstaked and created a 10 GHC penalty pool. Admin distributes it:

```rust
// GLOBAL (in staking_hub)
cumulative_reward_index = 0.1  ← UPDATED!
interest_pool = 0

// User Alice (hasn't interacted yet)
staked_balance: 100 GHC,
unclaimed_interest: 0,
last_reward_index: 0  ← Outdated!
```

**At this moment, Alice is "entitled" to:**
```
interest = 100 GHC × 0.1 = 10 GHC
```

But it hasn't been **calculated** yet (lazy evaluation).

---

### Step 2: Alice Decides to Unstake 90 GHC (Before Claiming!)

Alice calls `unstake(90)`. Here's what happens **inside the code**:

```rust
#[update]
async fn unstake(amount: u64) -> Result<u64, String> {
    let user = ic_cdk::caller();  // Alice
    
    // ⚠️ CRITICAL LINE 606: compound_interest() runs FIRST!
    compound_interest(user);
    
    // ... then unstakes are processed
}
```

### Inside `compound_interest()` (Runs BEFORE Unstaking):

```rust
fn compound_interest(user: Principal) {
    let global_index = 0.1;  // Current global value
    let profile = get_user_profile(user);  // Alice
    
    if global_index > profile.last_reward_index {
        // Calculate based on CURRENT staked_balance (still 100 GHC!)
        let index_diff = 0.1 - 0 = 0.1;
        let interest = (100 GHC × 0.1) / 1e18 = 10 GHC;  ← Full amount!
        
        profile.unclaimed_interest += 10 GHC;  // ← Interest secured!
        profile.last_reward_index = 0.1;       // ← Index synced!
        
        save_profile(profile);
    }
}
```

**State AFTER `compound_interest()` but BEFORE unstake:**
```rust
User Alice {
    staked_balance: 100 GHC,        ← Still full amount
    unclaimed_interest: 10 GHC,     ← Interest SECURED! ✅
    last_reward_index: 0.1          ← Synced!
}
```

### Then Unstake Continues:

```rust
// NOW reduce the staked balance
profile.staked_balance -= 90;  // 100 - 90 = 10 GHC

// Process the unstake (transfer tokens, etc.)
```

**State AFTER unstake completes:**
```rust
User Alice {
    staked_balance: 10 GHC,         ← REDUCED
    unclaimed_interest: 10 GHC,     ← Still has full 10 GHC! ✅
    last_reward_index: 0.1           ← Synced
}
```

---

### Step 3: Few Days Later, Alice Claims

Alice calls `claim_rewards()`:

```rust
fn claim_rewards() {
    // compound_interest() runs again
    let global_index = 0.1;  // (assume no new distributions)
    let index_diff = 0.1 - 0.1 = 0;  ← No NEW interest
    let interest = 10 GHC × 0 = 0;   ← No new calculation
    
    // unclaimed_interest stays at 10 GHC
    
    // Move to staked balance
    profile.staked_balance += 10 GHC;  // 10 + 10 = 20 GHC
    profile.unclaimed_interest = 0;
}
```

**Final State:**
```rust
User Alice {
    staked_balance: 20 GHC,         ← Got full 10 GHC interest! ✅
    unclaimed_interest: 0,
    last_reward_index: 0.1
}
```

---

## Summary: Alice Got Full Interest!

| Event | `staked_balance` | `unclaimed_interest` | Interest Earned |
|-------|-----------------|---------------------|----------------|
| **Initial** | 100 GHC | 0 | - |
| **distribute_interest() runs** | 100 GHC | 0 (not calculated yet) | (entitled to 10 GHC) |
| **Alice unstakes 90 GHC** | 10 GHC | **10 GHC** ✅ | 10 GHC secured |
| **Alice claims later** | **20 GHC** | 0 | - |

Alice received:
- 90 GHC back from unstake (minus 10% penalty = 81 GHC to wallet)
- 10 GHC still staked
- **+ 10 GHC interest** (full amount based on 100 GHC)

**Total:** She got the full interest she earned while she had 100 GHC staked!

---

## Why This Works: Every Update Function Calls `compound_interest()` First

The key is that **every function that might change the balance** calls `compound_interest()` first:

### Functions That Call `compound_interest()` Before Changing Balance:

1. **`unstake()`** - Line 606
   ```rust
   compound_interest(user);  // ← Calculate interest BEFORE reducing balance
   profile.staked_balance -= amount;
   ```

2. **`claim_rewards()`** - Line 713
   ```rust
   compound_interest(user);  // ← Calculate any pending interest first
   profile.staked_balance += profile.unclaimed_interest;
   ```

3. **`submit_quiz()`** - Line 523
   ```rust
   compound_interest(user);  // ← Calculate interest before adding quiz reward
   profile.staked_balance += reward_amount;
   ```

This ensures that:
- **Interest is always calculated based on the balance they HAD**
- **Before the balance changes**
- **Users can't "lose" interest by unstaking before claiming**

---

## Different Scenario: What If Alice Waits to Claim Until AFTER New Distribution?

Let's see a more complex case:

### Timeline:

```
T0: Alice has 100 GHC staked
    cumulative_reward_index = 0

T1: First distribution (+0.1)
    cumulative_reward_index = 0.1

T2: Alice unstakes 90 GHC
    → compound_interest() runs
    → unclaimed_interest = 10 GHC (based on 100 GHC × 0.1)
    → staked_balance = 10 GHC
    → last_reward_index = 0.1

T3: Second distribution (+0.1)
    cumulative_reward_index = 0.2  ← NEW distribution!

T4: Alice claims
    → compound_interest() runs
    → index_diff = 0.2 - 0.1 = 0.1
    → NEW interest = 10 GHC × 0.1 = 1 GHC  ← Based on CURRENT balance!
    → unclaimed_interest = 10 + 1 = 11 GHC
```

**Final Result:**
- 10 GHC interest from first distribution (when she had 100 GHC)
- 1 GHC interest from second distribution (when she had 10 GHC) ✅

This is **correct behavior**! She earned:
- Full interest on 100 GHC for the first period
- Proportional interest on 10 GHC for the second period

---

## Comparison: If Alice Had Claimed Before Unstaking

Let's compare to see if order matters:

### Scenario A: Unstake First, Claim Later (Your Concern)

```
1. distribute_interest() → index = 0.1
2. Alice unstakes 90 GHC
   → compound_interest() secures 10 GHC
   → balance = 10 GHC
3. Alice claims
   → Gets 10 GHC interest ✅
```

### Scenario B: Claim First, Then Unstake

```
1. distribute_interest() → index = 0.1
2. Alice claims
   → compound_interest() calculates 10 GHC
   → balance = 100 + 10 = 110 GHC
3. Alice unstakes 90 GHC
   → compound_interest() runs (no new index change, so 0 new interest)
   → balance = 110 - 90 = 20 GHC
```

### Final Balances Comparison:

| Scenario | Staked Balance | Claimed Interest |
|----------|---------------|------------------|
| **A: Unstake then claim** | 20 GHC | 10 GHC |
| **B: Claim then unstake** | 20 GHC | 10 GHC |

**They're identical!** ✅

The order doesn't matter because `compound_interest()` is called before ANY balance-changing operation.

---

## The Edge Case: Unstaking Between Distributions

The ONLY difference in final outcome is if a second distribution happens between unstake and claim:

### Scenario C: Unstake, New Distribution, Then Claim

```
1. First distribution → index = 0.1
2. Alice unstakes 90 GHC
   → Secures 10 GHC interest (100 × 0.1)
   → Balance = 10 GHC
3. Second distribution → index = 0.2  ← NEW!
4. Alice claims
   → Gets 10 + (10 × 0.1) = 11 GHC total
```

### Scenario D: Claim, Then Unstake, Then New Distribution

```
1. First distribution → index = 0.1
2. Alice claims
   → Gets 10 GHC (100 × 0.1)
   → Balance = 110 GHC
3. Alice unstakes 90 GHC
   → Balance = 20 GHC
4. Second distribution → index = 0.2
5. Alice claims again
   → Gets 20 × 0.1 = 2 GHC
```

### Comparison:

| Scenario | Total Interest Earned |
|----------|----------------------|
| **C: Unstake before 2nd distribution** | 11 GHC (10 + 1) |
| **D: Unstake after 1st claim** | 12 GHC (10 + 2) |

**Scenario D earns more!** ✅

Why? Because in D, she kept 20 GHC staked during the second distribution, whereas in C, she only had 10 GHC staked.

---

## Key Takeaway

### Your concern was:
> "Won't I lose interest if I unstake before claiming?"

### The answer:
**No, you won't lose PAST interest**, because `compound_interest()` is called BEFORE the unstake reduces your balance.

**However**, you WILL earn less FUTURE interest because you'll have less staked going forward.

### Rules:

1. **Past interest is always secured** - `compound_interest()` calculates based on your balance BEFORE it changes
2. **Future interest depends on current balance** - If you unstake, you earn less on future distributions
3. **Order doesn't matter for past interest** - Claim first vs unstake first gives same result
4. **Order DOES matter for future interest** - Keeping more staked = more future interest

---

## Code Proof

Look at the `unstake()` function in `user_profile/src/lib.rs`:

```rust
#[update]
async fn unstake(amount: u64) -> Result<u64, String> {
    let user = ic_cdk::caller();
    
    // Line 606: Apply Interest First! ← THE KEY LINE
    compound_interest(user);
    
    // NOW check and reduce balance
    let mut profile = USER_PROFILES.with(...);
    if profile.staked_balance < amount {
        return Err(...);
    }
    
    // Line 615: Reduce balance AFTER interest calculated
    profile.staked_balance -= amount;
    ...
}
```

This ensures:
1. Interest is calculated using **old balance** (100 GHC)
2. Saved to `unclaimed_interest`
3. `last_reward_index` is updated
4. **THEN** balance is reduced (100 → 10 GHC)

So you get full credit for the time you HAD the larger stake! ✅
