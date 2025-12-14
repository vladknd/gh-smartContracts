# Governance Implementation Plan

> **Last Updated:** December 2024  
> **Status:** Proposed  
> **Version:** 1.0

---

## Table of Contents

1. [Overview](#1-overview)
2. [Voting Power Model](#2-voting-power-model)
3. [Architecture](#3-architecture)
4. [Operational Governance](#4-operational-governance)
5. [Content Governance](#5-content-governance)
6. [Snapshot-Based Voting](#6-snapshot-based-voting)
7. [User Registry](#7-user-registry)
8. [Security Assessment](#8-security-assessment)
9. [Implementation Plan](#9-implementation-plan)
10. [Data Structures](#10-data-structures)
11. [API Reference](#11-api-reference)

---

## 1. Overview

The GreenHero governance system implements **progressive decentralization** through a dual voting power model:

| Voting Power Source | Who Exercises | Behavior |
|---------------------|---------------|----------|
| **VUC (Volume of Unmined Coins)** | Founders | Decreases as tokens are mined |
| **Staked Utility Tokens** | Users | Increases as users mine & stake |

This design ensures:
- **Early stage**: Founders have majority control to guide development
- **Over time**: Power gradually shifts to active users
- **Maturity**: Fully decentralized - users have 100% control

### Governance Domains

| Domain | Purpose | Manages |
|--------|---------|---------|
| **Operational Governance** | Treasury management | 3.6B Market Coin Treasury |
| **Content Governance** | Educational content | Learning units, quizzes, books |

Both governance canisters use the same voting power sources: **VUC (founders) + staked tokens (users)**.


---

## 2. Voting Power Model

### Core Principle: Progressive Decentralization

The governance system uses a **dual voting power model** that progressively shifts control from founders to users as the ecosystem grows.

```
TOTAL VOTING POWER = VUC (Founders) + Staked Tokens (Users)

Where:
  VUC = 4.2B - total_allocated (Utility Partition CAP minus mined tokens)
  Staked Tokens = Sum of all users' staked_balance

As more tokens are mined → VUC decreases → Users gain more control
```

### The Two Voting Power Sources

| Source | Who Exercises | Formula | Behavior |
|--------|---------------|---------|----------|
| **VUC** | Founders | 4.2B - total_allocated | Decreases as tokens are mined |
| **Staked Tokens** | Users | Sum of staked_balance | Increases as users mine & stake |

> **Note:** `unclaimed_interest` does NOT count as voting power. Interest must be claimed (added to staked_balance) to exercise voting power.

### Progressive Decentralization Visualization

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PROGRESSIVE DECENTRALIZATION                              │
└─────────────────────────────────────────────────────────────────────────────┘

  UTILITY PARTITION CAP: 4.2 BILLION GHC
  
  ═══════════════════════════════════════════════════════════════════════════
  
  EARLY STAGE (Few users, little mining)
  ──────────────────────────────────────
  
  total_allocated = 100M (example)
  
  ┌───────────────────────────────────────────────────────────────────────┐
  │████████████████████████████████████████████████████████████████│░░░░░│
  │              VUC: 4.1B (98%)                                   │Users│
  │              (Founder voting power)                            │100M │
  │                                                                │(2%) │
  └───────────────────────────────────────────────────────────────────────┘
  
  → Founders have dominant control in early stage
  
  
  MID STAGE (Growing user base)
  ─────────────────────────────
  
  total_allocated = 1B (example)
  
  ┌───────────────────────────────────────────────────────────────────────┐
  │████████████████████████████████████████████████████████│░░░░░░░░░░░░░│
  │              VUC: 3.2B (76%)                           │ Users: 1B   │
  │              (Founder voting power)                    │   (24%)     │
  └───────────────────────────────────────────────────────────────────────┘
  
  → Users gaining significant influence
  
  
  MATURE STAGE (Mass adoption)
  ────────────────────────────
  
  total_allocated = 3B (example)
  
  ┌───────────────────────────────────────────────────────────────────────┐
  │████████████████████████████│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
  │      VUC: 1.2B (29%)       │           Users: 3B (71%)               │
  └───────────────────────────────────────────────────────────────────────┘
  
  → Users have majority control
  
  
  FINAL STAGE (All tokens mined)
  ──────────────────────────────
  
  total_allocated = 4.2B
  
  ┌───────────────────────────────────────────────────────────────────────┐
  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
  │                     Users: 4.2B (100%)                               │
  │                  VUC: 0 (Founders have no voting power)              │
  └───────────────────────────────────────────────────────────────────────┘
  
  → Fully decentralized - users have complete control
```

### What Counts as Voting Power

| Token Type | Voting Power | Who | Explanation |
|------------|--------------|-----|-------------|
| **VUC** | ✅ YES | Founders | 4.2B - total_allocated (unmined tokens) |
| **staked_balance** | ✅ YES | Users | User's staked tokens in user_profile |
| `unclaimed_interest` | ❌ NO | - | Must claim (add to staked_balance) first |
| Unstaked tokens | ❌ NO | - | Becomes market coins, loses voting power |
| Founder's 0.5B | ❌ NO | - | Pre-allocated market coins, not VUC |
| Treasury 3.6B | ❌ NO | - | Managed BY governance, not voting WITH it |

### Voting Power Formula

```rust
// Total voting power in the system
fn get_total_voting_power() -> u64 {
    let vuc = get_vuc();              // Founder voting power
    let staked = get_total_staked();  // User voting power
    
    vuc + staked
}

// VUC = Unmined tokens (founders' voting power)
fn get_vuc() -> u64 {
    const UTILITY_CAP: u64 = 4_200_000_000 * 100_000_000; // 4.2B
    let total_allocated = GLOBAL_STATS.total_allocated;
    
    UTILITY_CAP.saturating_sub(total_allocated)
}

// User voting power = their staked balance only
fn get_user_voting_power(user: Principal) -> u64 {
    // Query user's shard
    let profile = get_user_profile(user);
    
    // ONLY staked_balance counts (NOT unclaimed_interest)
    profile.staked_balance
}

// Founder voting power = VUC split between founders
fn get_founder_voting_power(founder: Principal) -> u64 {
    let vuc = get_vuc();
    
    match founder {
        FOUNDER_1 => vuc * 60 / 100,  // 60% of VUC
        FOUNDER_2 => vuc * 40 / 100,  // 40% of VUC
        _ => 0,
    }
}
```

### Example: Voting Power Distribution Over Time

| Stage | total_allocated | VUC (Founders) | Staked (Users) | Founder % | User % |
|-------|-----------------|----------------|----------------|-----------|--------|
| Launch | 0 | 4.2B | 0 | 100% | 0% |
| Year 1 | 500M | 3.7B | 500M | 88% | 12% |
| Year 2 | 1.5B | 2.7B | 1.5B | 64% | 36% |
| Year 3 | 2.5B | 1.7B | 2.5B | 40% | 60% |
| Year 5 | 3.5B | 700M | 3.5B | 17% | 83% |
| Maturity | 4.2B | 0 | 4.2B | 0% | 100% |

### Why This Design?

1. **Early Protection**: Founders maintain control during vulnerable early stages
2. **Earned Transition**: Users must actively participate (mine + stake) to gain power
3. **Inevitable Decentralization**: As tokens are mined, power shifts to users
4. **No Gaming**: Can't buy voting power with market coins
5. **Commitment Required**: Must keep tokens staked to maintain power


---

## 3. Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    COMPLETE GOVERNANCE ARCHITECTURE                          │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────┐     ┌─────────────────────┐
                    │  FOUNDERS           │     │   USERS             │
                    │                     │     │                     │
                    │  Voting Power =     │     │  Voting Power =     │
                    │  VUC (4.2B - mined) │     │  staked_balance     │
                    │                     │     │  (from user_profile)│
                    └──────────┬──────────┘     └──────────┬──────────┘
                               │                           │
                               │     VOTING ON PROPOSALS   │
                               └─────────────┬─────────────┘
                                             │
                    ┌────────────────────────┴────────────────────────┐
                    │                                                 │
                    ▼                                                 ▼
       ┌────────────────────────┐                ┌────────────────────────┐
       │  OPERATIONAL_GOVERNANCE│                │   CONTENT_GOVERNANCE   │
       ├────────────────────────┤                ├────────────────────────┤
       │                        │                │                        │
       │ Manages:               │                │ Manages:               │
       │ • Treasury (3.6B)      │                │ • Learning units       │
       │ • DEX liquidity        │                │ • Quiz content         │
       │ • Partnerships         │                │ • Book whitelist       │
       │ • Marketing spend      │                │ • Content moderation   │
       │                        │                │                        │
       └───────────┬────────────┘                └───────────┬────────────┘
                   │                                         │
                   │ Query voting power                      │
                   └─────────────────┬───────────────────────┘
                                     │
                                     ▼
                    ┌────────────────────────────────────────┐
                    │            STAKING_HUB                 │
                    │      (Voting Power Oracle)             │
                    ├────────────────────────────────────────┤
                    │                                        │
                    │  VUC Calculation:                      │
                    │  get_vuc() = 4.2B - total_allocated    │
                    │                                        │
                    │  User Voting Power:                    │
                    │  get_user_voting_power(user)           │
                    │  → queries user's shard                │
                    │                                        │
                    │  Total Voting Power:                   │
                    │  get_total_voting_power()              │
                    │  = VUC + total_staked                  │
                    │                                        │
                    │  USER_SHARD_REGISTRY:                  │
                    │  User → Shard mapping (O(1) lookup)    │
                    │                                        │
                    └──────────────────┬─────────────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         │                             │                             │
         ▼                             ▼                             ▼
   ┌───────────────┐           ┌───────────────┐           ┌───────────────┐
   │  USER_PROFILE │           │  USER_PROFILE │           │  USER_PROFILE │
   │    SHARD 1    │           │    SHARD 2    │           │    SHARD N    │
   ├───────────────┤           ├───────────────┤           ├───────────────┤
   │               │           │               │           │               │
   │ staked_balance│           │ staked_balance│           │ staked_balance│
   │ (voting power)│           │ (voting power)│           │ (voting power)│
   │               │           │               │           │               │
   └───────────────┘           └───────────────┘           └───────────────┘
```


---

## 4. Operational Governance

### Purpose

Manages the **3.6B Market Coin Treasury** with democratic oversight from founders (VUC) and stakers (users).

### Proposal Types

```rust
enum OperationalProposal {
    // Treasury Spending
    TreasurySpend {
        recipient: Principal,
        amount: u64,
        category: SpendingCategory,
        description: String,
    },
    
    // DEX Operations
    AddDexLiquidity {
        dex_canister: Principal,
        ghc_amount: u64,
        paired_token: Principal,
        paired_amount: u64,
    },
    
    RemoveDexLiquidity {
        dex_canister: Principal,
        lp_token_amount: u64,
    },
    
    // Partnership/Grants
    PartnershipGrant {
        recipient: Principal,
        amount: u64,
        vesting_months: u8,
        description: String,
    },
    
    // Emergency Actions
    EmergencyPause {
        target_canister: Principal,
        reason: String,
    },
}

enum SpendingCategory {
    Marketing,
    Development,
    Partnership,
    LiquidityProvision,
    CommunityGrant,
    Operations,
}
```

### Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **Voting Period** | 7 days | Time to review financial decisions |
| **Execution Delay** | 2 days | Time to exit if disagreement |
| **Emergency Delay** | 4 hours | Fast response to critical issues |
| **Quorum** | 5% of total staked | Ensure sufficient participation |
| **Pass Threshold** | 60% | Supermajority for financial decisions |
| **Proposal Cost** | 1,000 GHC | Prevent spam proposals |
| **Monthly Limit** | 16M GHC | Cap treasury outflow |

---

## 5. Content Governance

### Purpose

Manages what **educational content** appears in the learning_engine canister.

### Proposal Types

```rust
enum ContentProposal {
    // Content Addition
    AddLearningUnit {
        unit_id: String,
        unit_title: String,
        chapter_id: String,
        content_hash: [u8; 32],
        quiz_questions: u8,
        submitter: Principal,
    },
    
    // Content Removal
    RemoveLearningUnit {
        unit_id: String,
        reason: String,
    },
    
    // Quiz Updates
    UpdateQuiz {
        unit_id: String,
        new_quiz_hash: [u8; 32],
    },
    
    // Source Whitelist
    WhitelistSource {
        source_id: String,
        source_type: SourceType,
        metadata: String,
    },
    
    BlacklistSource {
        source_id: String,
        reason: String,
    },
    
    // Moderation
    AddContentModerator {
        moderator: Principal,
        permissions: ModeratorPermissions,
    },
    
    RemoveContentModerator {
        moderator: Principal,
    },
}
```

### Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **Voting Period** | 3 days | Content less critical than treasury |
| **Execution Delay** | 1 day | Quick content updates |
| **Quorum** | 3% of total staked | Lower barrier for content |
| **Pass Threshold** | 55% | Simple majority sufficient |
| **Proposal Cost** | 100 GHC | Encourage content contributions |

### Moderator System

Trusted moderators can add content without full governance vote for efficiency:

- Maximum 10 moderators
- 5 actions per day limit
- Governance can override moderator actions
- Governance can remove moderators

---

## 6. Snapshot-Based Voting

### The Problem

Voting power constantly changes as users mine tokens. This creates vulnerabilities:

1. **Front-Running**: See proposal, mine heavily to influence vote
2. **Flash Staking**: Stake huge amount, vote, unstake immediately
3. **Moving Quorum**: Total staked grows, quorum keeps changing
4. **Inconsistent Tallies**: Vote weight changes mid-voting period

### The Solution

**Freeze voting power at proposal creation using snapshots.**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SNAPSHOT VOTING TIMELINE                                  │
└─────────────────────────────────────────────────────────────────────────────┘

  Day 0                      Day 1-7                    Day 8-9          Day 10
    │                           │                          │               │
    ▼                           ▼                          ▼               ▼
    
 ┌──────────┐             ┌──────────┐              ┌──────────┐      ┌──────────┐
 │ SNAPSHOT │     →       │  VOTING  │      →       │ TIMELOCK │  →   │ EXECUTE  │
 │ TAKEN    │             │  PERIOD  │              │  PERIOD  │      │          │
 └──────────┘             └──────────┘              └──────────┘      └──────────┘
      │                        │                         │
      │                        │                         │
  Record:               All votes use             Wait period
  • total_staked        SNAPSHOT values           for exit
  • user balances       (not current)
  • quorum threshold
      │
      ▼
  Voting power FROZEN
  Mining after this does NOT affect votes
```

### How It Works

```rust
struct Proposal {
    id: u64,
    proposal_type: ProposalType,
    
    // SNAPSHOT DATA (frozen at creation)
    snapshot_timestamp: u64,
    snapshot_total_voting_power: u64,
    snapshot_quorum: u64,  // Fixed at 5% of snapshot_total
    
    // Voting data
    votes_for: u64,
    votes_against: u64,
    voters: HashMap<Principal, VoteCast>,
    
    status: ProposalStatus,
}

// Voting uses snapshot balance, not current balance
async fn vote(proposal_id: u64, vote_for: bool) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let proposal = get_proposal(proposal_id)?;
    
    // Get balance AT SNAPSHOT TIME (not current!)
    let voting_power = get_voting_power_at_snapshot(
        caller, 
        proposal.snapshot_timestamp
    ).await?;
    
    record_vote(proposal_id, caller, vote_for, voting_power);
    Ok(())
}
```

---

## 7. User Registry

### The Problem

With many shards, finding which shard has a specific user is expensive:

- Must query ALL shards to find user
- O(n) where n = number of shards
- Slow and costly at scale

### The Solution

**Maintain a user → shard mapping in staking_hub.**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    USER REGISTRY IN STAKING HUB                              │
└─────────────────────────────────────────────────────────────────────────────┘

  STAKING_HUB
  ┌────────────────────────────────────────┐
  │                                        │
  │  USER_SHARD_REGISTRY:                  │
  │  ┌──────────────────────────────────┐  │
  │  │ User A  →  Shard 1               │  │
  │  │ User B  →  Shard 1               │  │
  │  │ User C  →  Shard 2               │  │
  │  │ User D  →  Shard 3               │  │
  │  │ ...                              │  │
  │  └──────────────────────────────────┘  │
  │                                        │
  │  get_user_shard(user) → Principal      │
  │  O(1) lookup!                          │
  │                                        │
  └────────────────────────────────────────┘
  
  
  VOTING FLOW (Optimized):
  ═════════════════════════
  
  1. User votes → Governance receives
  2. Governance: get_user_shard(user) → Shard 2 (O(1))
  3. Governance: Shard2.get_voting_power(user) → 5000
  4. Record vote with 5000 power
  
  Total: 2 calls (not N calls!)
```

### Storage Requirements

| Users | Registry Size |
|-------|---------------|
| 100,000 | 6 MB |
| 1,000,000 | 60 MB |
| 10,000,000 | 600 MB |

Well within stable memory limits.

---

## 8. Security Assessment

### 8.1 Attack Vectors & Mitigations

#### Attack: Front-Running

```
Scenario: Attacker sees favorable proposal, mines heavily to increase voting power

Mitigation: SNAPSHOT VOTING
  └── Voting power frozen at proposal creation
  └── Mining AFTER proposal doesn't affect vote
  └── ✅ FULLY MITIGATED
```

#### Attack: Flash Staking

```
Scenario: Attacker borrows/stakes large amount, votes, unstakes immediately

Mitigation: SNAPSHOT VOTING + UNSTAKING PENALTY
  └── Must have balance BEFORE proposal creation
  └── 10% penalty discourages short-term staking games
  └── ✅ FULLY MITIGATED
```

#### Attack: Quorum Manipulation

```
Scenario: Total staked grows during voting, quorum becomes unreachable

Mitigation: FIXED QUORUM AT SNAPSHOT
  └── Quorum calculated from snapshot_total_voting_power
  └── Does not change during voting period
  └── ✅ FULLY MITIGATED
```

#### Attack: Sybil Attack (Many Fake Accounts)

```
Scenario: Create many accounts, earn small amounts each, vote many times

Mitigation: VOTING POWER = ACTUAL STAKE
  └── 100 accounts with 10 GHC each = 1 account with 1000 GHC
  └── No advantage to splitting accounts
  └── Mining requires quiz completion (human effort)
  └── ✅ FULLY MITIGATED
```

#### Attack: Vote Buying

```
Scenario: Attacker pays users to vote a certain way

Mitigation: PARTIAL (inherent to token voting)
  └── Vote buying is possible in any token-based governance
  └── Stake requirements make it expensive
  └── Time-locked execution allows exit
  └── ⚠️ PARTIALLY MITIGATED (economic deterrent)
```

#### Attack: Governance Takeover (51% Attack)

```
Scenario: Entity acquires >50% of staked tokens to control all votes

Mitigation: ECONOMIC + STRUCTURAL
  └── Acquiring 50%+ of staked tokens is extremely expensive
  └── Mining is rate-limited (can't rapidly accumulate)
  └── 60% threshold for operational (need supermajority)
  └── Time-locked execution (2 days to respond)
  └── ⚠️ PARTIALLY MITIGATED (economic deterrent)
```

#### Attack: Proposal Spam

```
Scenario: Flood governance with low-quality proposals

Mitigation: PROPOSAL COST
  └── 1000 GHC for operational proposals
  └── 100 GHC for content proposals
  └── Spam becomes expensive
  └── ✅ FULLY MITIGATED
```

#### Attack: Execution Front-Running

```
Scenario: See passed proposal, take action before it executes

Mitigation: TIME-LOCK EXECUTION
  └── 2-day delay for operational
  └── 1-day delay for content
  └── Public visibility of pending execution
  └── ✅ FULLY MITIGATED
```

### 8.2 Security Matrix

| Attack Vector | Risk Level | Mitigation | Status |
|---------------|------------|------------|--------|
| Front-Running | HIGH | Snapshot voting | ✅ Secure |
| Flash Staking | HIGH | Snapshot + penalty | ✅ Secure |
| Quorum Manipulation | MEDIUM | Fixed quorum | ✅ Secure |
| Sybil Attack | MEDIUM | Stake-weighted voting | ✅ Secure |
| Vote Buying | LOW | Economic deterrent | ⚠️ Acceptable |
| 51% Attack | LOW | Economic + thresholds | ⚠️ Acceptable |
| Proposal Spam | LOW | Proposal cost | ✅ Secure |
| Execution Front-Run | LOW | Time-lock | ✅ Secure |

### 8.3 Reliability Assessment

| Component | Reliability Measure | Status |
|-----------|---------------------|--------|
| **Voting Power Source** | Single source of truth (staking_hub) | ✅ Reliable |
| **User Lookup** | O(1) via registry | ✅ Reliable |
| **Snapshot Storage** | Stable memory, persists upgrades | ✅ Reliable |
| **Cross-Canister Calls** | Async with error handling | ✅ Reliable |
| **Proposal Storage** | Stable memory, immutable after creation | ✅ Reliable |
| **Vote Counting** | Deterministic, auditable | ✅ Reliable |
| **Execution** | Authorized callers only | ✅ Reliable |

### 8.4 Failure Modes

| Failure | Impact | Recovery |
|---------|--------|----------|
| Shard offline during vote | User can't vote | Retry when shard recovers |
| Hub offline | No voting power queries | Governance paused until recovery |
| Governance canister upgrade | In-progress proposals preserved | Continue after upgrade |
| Network partition | Votes may fail | Retry mechanism built-in |

---

## 9. Implementation Plan

### Phase 1: Staking Hub Updates (Week 1)

**Files to modify:** `src/staking_hub/src/lib.rs`

```
Add:
├── USER_SHARD_REGISTRY: Principal → Principal
├── register_user_location(user) - called by shards
├── get_user_shard(user) - query for governance  
├── get_user_voting_power(user) - queries correct shard
├── get_total_voting_power() - returns total_staked
└── get_voting_power_at_timestamp(user, ts) - for snapshots
```

### Phase 2: User Profile Updates (Week 1)

**Files to modify:** `src/user_profile/src/lib.rs`

```
Add:
├── On register_user: call hub.register_user_location()  
├── get_user_voting_power(user) -> staked + unclaimed
└── get_voting_power_at_timestamp(user, ts) -> historical balance
```

### Phase 3: Operational Governance (Week 2-3)

**Files to create:** `src/operational_governance/`

```
Create:
├── Cargo.toml
├── src/lib.rs
│   ├── Proposal storage with snapshots
│   ├── Voting logic with snapshot verification
│   ├── Quorum checking
│   ├── Time-locked execution
│   ├── Treasury transfer integration
│   └── Monthly spending limit enforcement
└── operational_governance.did
```

### Phase 4: Content Governance (Week 3)

**Files to create:** `src/content_governance/`

```
Create:
├── Cargo.toml  
├── src/lib.rs
│   ├── Same voting mechanics as operational
│   ├── Content-specific proposal types
│   ├── Learning engine integration
│   └── Moderator management
└── content_governance.did
```

### Phase 5: Learning Engine Integration (Week 4)

**Files to modify:** `src/learning_engine/src/lib.rs`

```
Add:
├── Set content_governance as authorized caller
├── governance_add_unit() - called by governance only
├── governance_remove_unit() - called by governance only
└── moderator_add_unit() - called by approved moderators
```

### Phase 6: Testing & Deployment (Week 4-5)

```
Create test scripts:
├── test_operational_governance.sh
├── test_content_governance.sh
└── test_governance_security.sh

Test scenarios:
├── Snapshot accuracy (votes use correct historical balance)
├── Front-running prevention (mining after proposal doesn't help)
├── Quorum stability (fixed at snapshot)
├── Cross-shard voting (users in different shards can vote)
├── Execution authorization (only passed proposals execute)
└── Time-lock enforcement (can't execute early)
```

---

## 10. Data Structures

### Proposal

```rust
struct Proposal {
    // Identity
    id: u64,
    proposer: Principal,
    proposal_type: ProposalType,
    title: String,
    description: String,
    
    // Snapshot (frozen at creation)
    snapshot_timestamp: u64,
    snapshot_total_voting_power: u64,
    snapshot_quorum: u64,
    
    // Timeline
    created_at: u64,
    voting_ends_at: u64,
    execution_available_at: u64,
    executed_at: Option<u64>,
    
    // Votes
    votes_for: u64,
    votes_against: u64,
    voter_count: u64,
    
    // Status
    status: ProposalStatus,
}

enum ProposalStatus {
    Active,           // Voting in progress
    Passed,           // Voting passed, awaiting execution
    Rejected,         // Voting failed
    Executed,         // Successfully executed
    Expired,          // Passed but not executed in time
    Cancelled,        // Cancelled by proposer (before voting ends)
}
```

### Vote Record

```rust
struct VoteRecord {
    proposal_id: u64,
    voter: Principal,
    vote: bool,  // true = for, false = against
    voting_power: u64,
    timestamp: u64,
}
```

### Governance Config

```rust
struct GovernanceConfig {
    // Voting parameters
    voting_period_ns: u64,
    execution_delay_ns: u64,
    emergency_execution_delay_ns: u64,
    
    // Thresholds
    quorum_percent: u8,
    pass_threshold_percent: u8,
    
    // Costs
    min_proposal_stake: u64,
    
    // Limits (operational only)
    monthly_spending_limit: Option<u64>,
}
```

---

## 11. API Reference

### Operational Governance

```candid
service : {
    // Proposal Management
    "create_proposal": (OperationalProposal, text, text) -> (variant { Ok: nat64; Err: text });
    "vote": (nat64, bool) -> (variant { Ok: null; Err: text });
    "execute_proposal": (nat64) -> (variant { Ok: null; Err: text });
    "cancel_proposal": (nat64) -> (variant { Ok: null; Err: text });
    
    // Queries
    "get_proposal": (nat64) -> (opt Proposal) query;
    "get_active_proposals": () -> (vec Proposal) query;
    "get_proposal_votes": (nat64) -> (vec VoteRecord) query;
    "get_my_voting_power": () -> (nat64);
    "has_voted": (nat64) -> (bool) query;
    
    // Stats
    "get_monthly_spending": () -> (nat64) query;
    "get_treasury_balance": () -> (nat64);
}
```

### Content Governance

```candid
service : {
    // Proposal Management  
    "create_content_proposal": (ContentProposal, text, text) -> (variant { Ok: nat64; Err: text });
    "vote": (nat64, bool) -> (variant { Ok: null; Err: text });
    "execute_proposal": (nat64) -> (variant { Ok: null; Err: text });
    
    // Moderator Actions (faster, limited)
    "moderator_add_unit": (LearningUnit) -> (variant { Ok: null; Err: text });
    "moderator_remove_unit": (text) -> (variant { Ok: null; Err: text });
    
    // Queries
    "get_proposal": (nat64) -> (opt Proposal) query;
    "get_active_proposals": () -> (vec Proposal) query;
    "get_moderators": () -> (vec Principal) query;
    "is_moderator": (principal) -> (bool) query;
}
```

### Staking Hub (Voting Power Oracle)

```candid
service : {
    // Voting Power Queries
    "get_total_voting_power": () -> (nat64) query;  // VUC + total_staked
    "get_vuc": () -> (nat64) query;                  // Founder voting power
    "get_user_voting_power": (principal) -> (nat64); // User staked_balance
    "get_founder_voting_power": (principal) -> (nat64) query;
    "get_user_shard": (principal) -> (opt principal) query;
    
    // Shard Registration
    "register_user_location": (principal) -> (variant { Ok: null; Err: text });
}
```

---

## Summary

| Aspect | Implementation |
|--------|----------------|
| **Voting Power Model** | Progressive Decentralization (VUC + Staked) |
| **VUC (Founders)** | 4.2B - total_allocated (decreases as tokens mined) |
| **Staked Tokens (Users)** | Sum of staked_balance (increases as users mine) |
| **unclaimed_interest** | NOT voting power (must claim first) |
| **Unstaked Tokens** | NO voting power (market coins) |
| **Snapshot Voting** | Power frozen at proposal creation |
| **User Registry** | O(1) lookup via staking_hub |
| **Source of Truth** | staking_hub for all voting power |
| **Operational Quorum** | 5% of snapshot total |
| **Content Quorum** | 3% of snapshot total |
| **Execution Delay** | 2 days (operational), 1 day (content) |
| **Security Level** | HIGH - all major attacks mitigated |

### Progressive Decentralization Summary

| Stage | VUC (Founders) | Staked (Users) | Control |
|-------|----------------|----------------|---------|
| Launch | 100% | 0% | Founders |
| Year 2 | ~64% | ~36% | Founders (declining) |
| Year 3 | ~40% | ~60% | Users (majority) |
| Maturity | 0% | 100% | Fully Decentralized |

---

*This document should be updated as governance is implemented and refined.*

