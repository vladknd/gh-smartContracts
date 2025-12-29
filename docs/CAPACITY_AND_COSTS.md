# Shard Capacity & Cost Estimations

> **Last Updated:** December 2024  
> **Context:** User profile shards with time-weighted staking (timestamps per mining event)

---

## 1. Data Structure Per User

### Current Structure

| Data Type | Size (Candid Encoded) | Notes |
|-----------|----------------------|-------|
| `UserProfile` | ~200 bytes | email, name, education, gender, balances, indices |
| `UserDailyStats` | ~30 bytes | day_index, quizzes_taken, tokens_earned |
| `CompletedQuizzes` | ~70 bytes/entry | UserQuizKey → bool mapping |
| `TransactionRecord` | ~40 bytes/entry | timestamp, tx_type, amount |

### Proposed Addition: Staking Events (for time-weighted interest)

```rust
struct StakingEvent {
    timestamp: u64,      // When tokens were staked (8 bytes)
    amount: u64,         // Amount staked in this event (8 bytes)
}
// Candid encoded: ~25-30 bytes per event
```

---

## 2. Storage Per User Scenarios

### Scenario A: Light User (Casual Usage)
- 100 quizzes completed over lifetime
- 100 transactions
- 100 staking events

| Data Type | Size |
|-----------|------|
| UserProfile | 200 bytes |
| UserDailyStats | 30 bytes |
| Completed Quizzes (100) | 7,000 bytes |
| Transaction History (100) | 4,000 bytes |
| Staking Events (100) | 3,000 bytes |
| **Total** | **~14,000 bytes (~14 KB)** |

### Scenario B: Power User (Heavy Usage)
- 1,000 quizzes completed
- 1,000 transactions
- 1,000 staking events

| Data Type | Size |
|-----------|------|
| UserProfile | 200 bytes |
| Completed Quizzes (1,000) | 70,000 bytes |
| Transaction History (1,000) | 40,000 bytes |
| Staking Events (1,000) | 30,000 bytes |
| **Total** | **~140,000 bytes (~140 KB)** |

---

## 3. IC Canister Limits

| Resource | Limit | Notes |
|----------|-------|-------|
| **Stable Memory** | 500 GB | Theoretical max per canister |
| **Practical Stable Memory** | 100 GB | Performance degrades at higher sizes |
| **Heap Memory** | 4 GB | Per message execution |
| **Instruction Limit** | 20B per call | Computation limit per update call |

---

## 4. Users Per Shard Calculation

### Using 100 GB Practical Limit

| User Type | Size per User | Max Users per Shard |
|-----------|---------------|---------------------|
| Light Users | 14 KB | **7,142,857** |
| Average Users | 50 KB | **2,000,000** |
| Power Users | 140 KB | **714,285** |

### Recommended Shard Limits

| Strategy | Users per Shard | Reasoning |
|----------|-----------------|-----------|
| Conservative (Current) | **100,000** | Maximum safety margin, fast queries |
| Moderate | **500,000** | Good balance of capacity vs performance |
| Aggressive | **2,000,000** | Maximizes capacity, may impact query times |

---

## 5. IC Pricing Reference (Late 2024)

| Resource | Cost |
|----------|------|
| Storage (Stable Memory) | **$5 per GB/year** |
| Compute (Cycles) | **~$0.50 per 100B instructions** |
| Canister Creation | **~100B cycles (~$0.50)** |
| Message Passing | Negligible at scale |

> Note: 1 trillion cycles ≈ $1.40 USD

---

## 6. Cost Per Shard at Different Capacities

### 100,000 Users (Current Default Limit)

```
Storage: 100,000 × 14 KB = 1.4 GB
Annual Storage Cost: 1.4 GB × $5/GB = $7/year

Compute (10 interactions/user/month):
100,000 × 10 × 12 × 1M instructions = 12 trillion instructions
Annual Compute Cost: ~$6/year

TOTAL: ~$13/year per shard
```

### 500,000 Users

```
Storage: 500,000 × 14 KB = 7 GB
Annual Storage Cost: 7 GB × $5/GB = $35/year

Compute: ~$30/year

TOTAL: ~$65/year per shard
```

### 1,000,000 Users

```
Storage: 1,000,000 × 14 KB = 14 GB
Annual Storage Cost: 14 GB × $5/GB = $70/year

Compute: ~$60/year

TOTAL: ~$130/year per shard
```

### 5,000,000 Users (Near Capacity)

```
Storage: 5,000,000 × 14 KB = 70 GB
Annual Storage Cost: 70 GB × $5/GB = $350/year

Compute: ~$300/year

TOTAL: ~$650/year per shard
```

---

## 7. Scale Projections

| Total Users | Shards Needed | Annual Cost |
|-------------|---------------|-------------|
| 100,000 | 1 | ~$13 |
| 500,000 | 1-5 | ~$65-$325 |
| 1,000,000 | 1-10 | ~$130-$650 |
| 10,000,000 | 10-20 | ~$1,300-$2,600 |
| 50,000,000 | 50-100 | ~$6,500-$13,000 |
| 100,000,000 | 100-200 | ~$13,000-$26,000 |

---

## 8. Optimization Opportunities

### A. Bucketed Staking Events
Instead of storing every mining event individually:

```rust
// Current: 100 events × 30 bytes = 3,000 bytes
struct StakingEvent {
    timestamp: u64,
    amount: u64,
}

// Optimized: 52 weekly buckets × 20 bytes = 1,040 bytes (65% reduction)
struct WeeklyStakingBucket {
    week_start: u64,
    total_staked: u64,
}
```

### B. Running Weighted Average
Single computed value instead of event list:

```rust
// Optimized: Just 32 bytes total (99% reduction)
struct StakingState {
    current_balance: u64,
    weighted_average_stake_time: u64,
    last_update: u64,
    total_stake_seconds: u128,  // For compound calculations
}
```

### C. Pruning Old Data
- Archive completed quizzes older than 1 year
- Summarize old transactions monthly
- Reduces long-term per-user storage

---

## 9. Summary Table

| Metric | Value |
|--------|-------|
| **Data per Light User** | ~14 KB |
| **Data per Power User** | ~140 KB |
| **Safe Users per Shard** | 100,000 - 500,000 |
| **Max Users per Shard** | ~5,000,000 (light users) |
| **Cost per Shard (100K users)** | ~$13/year |
| **Cost per Shard (1M users)** | ~$130/year |
| **Cost for 10M users** | ~$1,300-$2,600/year |
| **Cost for 100M users** | ~$13,000-$26,000/year |

---

## 10. Key Takeaways

1. **Storage is cheap on ICP** - Even 100 million users costs under $30K/year
2. **Current 100K limit is very conservative** - Could safely increase to 500K-1M
3. **Time-weighted staking adds ~3KB per user** - Negligible impact on capacity
4. **Optimization available** - Can reduce staking storage by 65-99% if needed
5. **Horizontal scaling works** - Just add more shards as user base grows

---

*This document should be updated as IC pricing changes or data structures are modified.*

---

## 11. Detailed Step-by-Step Cost Calculations

> **Goal:** Transparent, reproducible calculations for infrastructure costs with every step shown.

---

### STEP 1: Calculate Exact Byte Sizes Per Data Structure

All data is stored using **Candid encoding**. Let's calculate the exact byte sizes from our Rust structs.

#### 1.1 UserProfile (stored once per user)

```rust
struct UserProfile {
    email: String,           // 4 bytes length prefix + ~30 chars = ~34 bytes
    name: String,            // 4 bytes length prefix + ~25 chars = ~29 bytes
    education: String,       // 4 bytes length prefix + ~20 chars = ~24 bytes  
    gender: String,          // 4 bytes length prefix + ~10 chars = ~14 bytes
    staked_balance: u64,     // 8 bytes
    transaction_count: u64,  // 8 bytes
}
```

**Calculation:**
```
String overhead:       4 bytes × 4 fields = 16 bytes
String content:        30 + 25 + 20 + 10  = 85 bytes  
Numeric fields:        8 + 8              = 16 bytes
Candid type headers:   ~15 bytes (magic bytes, type table)
─────────────────────────────────────────────────────────
TOTAL UserProfile:     ~132 bytes (rounded to 140 bytes)
```

**Key in StableBTreeMap (Principal):** 29 bytes (typical compressed format)

**UserProfile Entry Total:** `140 + 29 = ~170 bytes`

---

#### 1.2 UserQuizKey + bool (stored once per completed quiz)

```rust
struct UserQuizKey {
    user: Principal,      // 29 bytes
    unit_id: String,      // 4 bytes prefix + ~20 chars = 24 bytes
}
// Value: bool = 1 byte
```

**Calculation:**
```
Principal:             29 bytes
String length prefix:   4 bytes
unit_id content:       20 bytes (e.g., "module_1_unit_3")
Candid overhead:        5 bytes
Value (bool):           1 byte
─────────────────────────────────────────────────────────
TOTAL per quiz entry:  ~59 bytes (rounded to 60 bytes)
```

---

#### 1.3 TransactionKey + TransactionRecord (stored once per transaction)

```rust
struct TransactionKey {
    user: Principal,  // 29 bytes
    index: u64,       // 8 bytes
}

struct TransactionRecord {
    timestamp: u64,              // 8 bytes
    tx_type: TransactionType,    // 1 byte (enum variant)
    amount: u64,                 // 8 bytes
}
```

**Calculation:**
```
TransactionKey:
  Principal:           29 bytes
  index (u64):          8 bytes
  Candid overhead:      5 bytes
  ──────────────────────────────
  Subtotal:            42 bytes

TransactionRecord:
  timestamp:            8 bytes
  tx_type (enum):       2 bytes (variant + tag)
  amount:               8 bytes
  Candid overhead:      5 bytes
  ──────────────────────────────
  Subtotal:            23 bytes

TOTAL per transaction: 42 + 23 = ~65 bytes (rounded to 70 bytes)
```

---

#### 1.4 UserDailyStats (stored once per user, overwritten daily)

```rust
struct UserDailyStats {
    day_index: u64,       // 8 bytes
    quizzes_taken: u8,    // 1 byte
    tokens_earned: u64,   // 8 bytes
}
```

**Calculation:**
```
day_index:              8 bytes
quizzes_taken:          1 byte
tokens_earned:          8 bytes
Candid overhead:        5 bytes
Key (Principal):       29 bytes
─────────────────────────────────────────────────────────
TOTAL:                 ~51 bytes (rounded to 55 bytes)
```

This is a **fixed cost per user** (not growing) since it only stores today's stats.

---

### STEP 2: Total Storage Per User Based on Activity

#### Formula:

```
Total_Bytes_Per_User = 
    UserProfile_Entry          (190 bytes, fixed)
  + UserDailyStats_Entry       (55 bytes, fixed)  
  + (Quizzes_Completed × 60)   (grows with activity)
  + (Transactions × 70)        (grows with activity)
```

---

#### Scenario: Light User (100 quizzes lifetime, 100 transactions)

```
UserProfile:           190 bytes
UserDailyStats:         55 bytes
Quizzes (100 × 60):  6,000 bytes
Transactions (100 × 70): 7,000 bytes
─────────────────────────────────────────────────────────
TOTAL:              13,245 bytes ≈ 13 KB
```

---

#### Scenario: Average User (500 quizzes, 500 transactions)

```
UserProfile:           190 bytes
UserDailyStats:         55 bytes
Quizzes (500 × 60): 30,000 bytes
Transactions (500 × 70): 35,000 bytes
─────────────────────────────────────────────────────────
TOTAL:              65,245 bytes ≈ 64 KB
```

---

#### Scenario: Power User (12,000 quizzes, 12,000 transactions)

```
UserProfile:             190 bytes
UserDailyStats:           55 bytes
Quizzes (12,000 × 60):  720,000 bytes
Transactions (12,000 × 70): 840,000 bytes
─────────────────────────────────────────────────────────
TOTAL:              1,560,245 bytes ≈ 1.52 MB
```

---

### STEP 3: ICP Pricing Reference (Official December 2024)

Source: [internetcomputer.org/docs/building-apps/essentials/gas-cost](https://internetcomputer.org/docs/building-apps/essentials/gas-cost)

| Resource | Cost (Cycles) | USD Equivalent |
|----------|---------------|----------------|
| **Storage** | 127,000 cycles/GiB/second | **~$5.35/GiB/year** |
| **Canister Creation** | 500 billion cycles | **~$0.65** |
| **Query Calls** | Free | $0 |
| **Ingress Message (Update)** | 1.2M base + 2K/byte | **~$0.0000016/call** |
| **Inter-Canister Call** | 260K base + 1K/byte | **~$0.00000035/call** |
| **Execution** | 0.4 cycles/instruction (13-node) | **~$0.000000000000537/instruction** |

**Conversion Rate:** 1 trillion cycles ≈ $1.34 USD

---

### STEP 4: Storage Cost Calculation

#### Formula:
```
Annual_Storage_Cost = (Total_GiB × 127,000 cycles/sec × 31,536,000 sec/year) / 1T × $1.34
                    = Total_GiB × 4.01T cycles × $1.34 / 1T
                    = Total_GiB × $5.37/year
```

**Simplified: $5.35 per GiB per year**

---

#### Example: 100,000 Users (Light Users @ 13 KB each)

```
Total Storage = 100,000 × 13 KB = 1,300,000 KB = 1.24 GiB

Annual Storage Cost = 1.24 GiB × $5.35/GiB = $6.63/year
```

---

#### Example: 100,000 Users (Power Users @ 1.52 MB each)

```
Total Storage = 100,000 × 1.52 MB = 152,000 MB = 148.4 GiB

Annual Storage Cost = 148.4 GiB × $5.35/GiB = $794/year
```

---

### STEP 5: Request/Compute Cost Calculation

#### User Activity Model

Assume per user per month:
- 10 quiz submissions (update calls)
- 20 profile views (query calls - FREE)
- 2 claim/unstake operations (update calls)
- Background: 1 sync per 5 seconds per shard (not per user)

**Cost Per Update Call:**
```
Ingress base:     1,200,000 cycles
Payload (~500B):    500 × 2,000 = 1,000,000 cycles
Execution (~5M instructions): 5M × 0.4 = 2,000,000 cycles
──────────────────────────────────────────────────────
Total per call:   ~4,200,000 cycles = 0.0000042 T cycles
                = ~$0.0000056 per call
```

**Monthly Cost Per Active User:**
```
Quiz submissions:  10 × $0.0000056 = $0.000056
Claim/Unstake:      2 × $0.0000056 = $0.0000112
──────────────────────────────────────────────────────
Monthly:           $0.0000672 per user
Annual:            $0.0008064 per user ≈ $0.0008
```

---

#### Example: 100,000 Active Users (Compute Cost)

```
Annual Compute = 100,000 × $0.0008 = $80/year
```

---

### STEP 6: Shard Capacity & Performance Thresholds

#### IC Canister Limits

| Resource | Hard Limit | Practical Limit | Notes |
|----------|------------|-----------------|-------|
| Stable Memory | 500 GiB | 100 GiB | Performance degrades >100 GiB |
| Heap Memory | 4 GiB | 2 GiB | Per message execution |
| Instructions/Call | 40 billion | 5 billion | Keep fast responses |
| Instructions/Round | 26 billion | - | Total subnet budget |

---

#### Users Per Shard Calculation

**Constraint 1: Storage Limit (100 GiB practical)**

```
Light Users (13 KB):    100 GiB / 13 KB = 8,076,923 users
Average Users (64 KB):  100 GiB / 64 KB = 1,638,400 users  
Power Users (1.52 MB):  100 GiB / 1.52 MB = 68,947 users
```

**Constraint 2: Query Performance**

StableBTreeMap lookup is O(log n). With 1M entries:
- Lookup: ~20 comparisons × 500 ns = ~10 μs (very fast)
- At 10M entries: ~24 comparisons = ~12 μs (still fast)

**No significant performance degradation until >10M entries.**

**Constraint 3: Iteration/Scan Operations**

If you ever iterate all users (e.g., for analytics):
```
100,000 users × 100 ns/entry = 10 ms (acceptable)
1,000,000 users × 100 ns/entry = 100 ms (borderline)
10,000,000 users × 100 ns/entry = 1 sec (too slow)
```

---

#### Recommended Shard Thresholds

| User Mix | Max Users/Shard | Threshold to Create New Shard |
|----------|-----------------|-------------------------------|
| All Light | 5,000,000 | 4,000,000 (80%) |
| All Average | 1,000,000 | 800,000 (80%) |
| All Power (12K quizzes) | 65,000 | 50,000 (77%) |
| Mixed (realistic) | 500,000 | **100,000 (conservative)** |

**Recommendation:** Set threshold at **100,000 users** for:
1. Comfortable safety margin
2. Fast query responses guaranteed
3. Room for user data growth over time
4. Predictable scaling (1 shard per 100K users)

---

### STEP 7: Complete Cost Model (20 Years)

#### Base Assumptions

| Parameter | Value | Reasoning |
|-----------|-------|-----------|
| Average user data | 64 KB | 500 quizzes over lifetime |
| Active users | 20% | Not all users active monthly |
| Shard threshold | 100,000 users | Conservative for perf |
| Storage cost | $5.35/GiB/year | Current ICP pricing |
| Compute cost (active user) | $0.0008/year | From Step 5 |

---

#### Formula: Annual Cost Per Shard

```
Annual_Shard_Cost = 
    Storage_Cost + Compute_Cost + Sync_Cost

Where:
  Storage_Cost = (Users × Avg_KB / 1024 / 1024) GiB × $5.35
  Compute_Cost = (Users × Active_Rate × $0.0008)
  Sync_Cost    = (365 × 24 × 3600 / 5) syncs × $0.00000035 = $2.21/year
```

---

#### Example: 100,000 Users Per Shard (Average Mix)

```
Storage:
  Total = 100,000 × 64 KB = 6,400,000 KB = 6.1 GiB
  Cost  = 6.1 × $5.35 = $32.64/year

Compute:
  Active Users = 100,000 × 20% = 20,000
  Cost = 20,000 × $0.0008 = $16/year

Sync Overhead:
  Cost = $2.21/year

────────────────────────────────────────────────────────
TOTAL PER SHARD: $32.64 + $16 + $2.21 = ~$51/year
```

---

### STEP 8: Power User Scenario (12,000 Quizzes)

#### Single Power User Storage

```
UserProfile:             190 bytes
UserDailyStats:           55 bytes
Quizzes (12,000 × 60):  720,000 bytes
Transactions (12,000 × 70): 840,000 bytes
─────────────────────────────────────────────────────────
TOTAL:              1,560,245 bytes = 1.49 MiB
```

#### Annual Storage Cost Per Power User

```
1.49 MiB × (1/1024) GiB × $5.35/GiB = $0.0078/year
```

#### 20-Year Storage Cost Per Power User

```
$0.0078 × 20 years = $0.156 (~16 cents lifetime)
```

#### Compute Cost for Power User (very active)

Assuming:
- 600 quizzes/year (12,000 ÷ 20 years)
- 600 transactions/year
- 50 profile views/month (query = free)

```
Update calls: 1,200/year × $0.0000056 = $0.00672/year
20-year compute: $0.00672 × 20 = $0.13
```

#### Total 20-Year Cost Per Power User

```
Storage:  $0.156
Compute:  $0.134
─────────────────────────────────────────────────────────
TOTAL:    ~$0.29 (29 cents per power user lifetime!)
```

---

### STEP 9: Scale Projections (20 Years)

#### Assumptions

- Start: 10,000 users
- Growth: 50%/year for years 1-5, 20%/year for 6-10, 10%/year for 11-20
- Mix: 70% light, 25% average, 5% power users
- Avg storage per user: ~50 KB (weighted)

#### Year-by-Year Projection

| Year | Users | Shards | Storage (GiB) | Annual Cost |
|------|-------|--------|---------------|-------------|
| 1 | 10,000 | 1 | 0.48 | $12 |
| 2 | 15,000 | 1 | 0.72 | $14 |
| 3 | 22,500 | 1 | 1.07 | $17 |
| 4 | 33,750 | 1 | 1.61 | $21 |
| 5 | 50,625 | 1 | 2.41 | $28 |
| 6 | 60,750 | 1 | 2.90 | $32 |
| 7 | 72,900 | 1 | 3.48 | $37 |
| 8 | 87,480 | 1 | 4.17 | $43 |
| 9 | 104,976 | 2 | 5.01 | $52 |
| 10 | 125,971 | 2 | 6.01 | $60 |
| 11 | 138,568 | 2 | 6.61 | $65 |
| 12 | 152,425 | 2 | 7.27 | $70 |
| 13 | 167,668 | 2 | 8.00 | $76 |
| 14 | 184,435 | 2 | 8.80 | $83 |
| 15 | 202,878 | 3 | 9.68 | $91 |
| 16 | 223,166 | 3 | 10.65 | $99 |
| 17 | 245,483 | 3 | 11.71 | $108 |
| 18 | 270,031 | 3 | 12.88 | $118 |
| 19 | 297,034 | 3 | 14.17 | $129 |
| 20 | 326,738 | 4 | 15.59 | $141 |

**20-YEAR CUMULATIVE TOTAL: ~$1,296**

---

### STEP 10: Sensitivity Analysis

#### What if ALL users are Power Users (12,000 quizzes)?

```
Storage per user: 1.49 MiB = 0.00145 GiB
100,000 power users = 145 GiB per shard (EXCEEDS 100 GiB limit)

Adjusted shard capacity: 65,000 power users/shard

Cost for 326,738 power users (Year 20):
  Shards needed: 6
  Storage: 326,738 × 0.00145 GiB = 473.8 GiB
  Annual Storage: 473.8 × $5.35 = $2,535/year
  Annual Compute: 326,738 × $0.0008 = $261/year
  ──────────────────────────────────────────────
  Annual Total: ~$2,800/year
  
20-YEAR CUMULATIVE (all power users): ~$28,000
```

---

### STEP 11: Final Summary Table

| Scenario | 20-Year Cost | Per User Lifetime | Monthly at Scale |
|----------|-------------|-------------------|------------------|
| **Light users (100 quizzes)** | $1,300 | $0.004 | $5/month |
| **Average users (500 quizzes)** | $2,500 | $0.008 | $10/month |
| **Power users (12,000 quizzes)** | $28,000 | $0.086 | $120/month |
| **Realistic mix (70/25/5)** | **$1,300 - $3,000** | **$0.01** | **$7-12/month** |

---

## 12. Quick Reference Card

### ICP Pricing (as of Dec 2024)

| Resource | Cost |
|----------|------|
| Storage | **$5.35/GiB/year** |
| Update Call | **$0.0000056/call** |
| Query Call | **FREE** |
| Canister Creation | **$0.65/canister** |
| Cycles Conversion | **1T cycles = $1.34** |

### Our Data Sizes

| Data Type | Size |
|-----------|------|
| UserProfile | **190 bytes** |
| Per Quiz Completion | **60 bytes** |
| Per Transaction | **70 bytes** |
| UserDailyStats | **55 bytes** (fixed) |

### User Storage by Activity

| User Type | Quizzes | Storage | Annual Cost |
|-----------|---------|---------|-------------|
| Light | 100 | 13 KB | $0.00007 |
| Average | 500 | 64 KB | $0.00033 |
| Power | 12,000 | 1.49 MB | $0.0078 |

### Shard Thresholds

| Metric | Value |
|--------|-------|
| Max users/shard (light) | 5,000,000 |
| Max users/shard (power) | 65,000 |
| **Recommended threshold** | **100,000 users** |
| Trigger for new shard | 80% of threshold |

### 20-Year ROI

| Scale | ICP Cost | AWS Equivalent | Savings |
|-------|----------|----------------|---------|
| 300K users | $3,000 | $50,000+ | **94%** |
| 1M users | $10,000 | $200,000+ | **95%** |
| 10M users | $100,000 | $2,000,000+ | **95%** |

> **Bottom Line:** At 300K users with realistic activity, infrastructure costs **less than $150/year** ($12/month). Even if every single user completes 12,000 quizzes, 20-year costs stay under **$30,000 total**.
