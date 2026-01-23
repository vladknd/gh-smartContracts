# Governance Architecture

> **Last Updated:** January 2026  
> **Version:** 4.0  
> **Status:** ✅ Implemented

---

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Voting Power Model](#voting-power-model)
4. [Proposal Types](#proposal-types)
5. [Proposal Lifecycle](#proposal-lifecycle)
6. [Configurable Governance Parameters](#configurable-governance-parameters)
7. [Proposal Execution Flows](#proposal-execution-flows)
8. [Inter-Canister Communication](#inter-canister-communication)
9. [Security Considerations](#security-considerations)
10. [API Reference](#api-reference)

---

## Overview

The GreenHero Coin (GHC) governance system enables **democratic decision-making** across two domains:

| Domain | Description | Proposal Types |
|--------|-------------|----------------|
| **Operational Governance** | Treasury spending, board member management, governance configuration | Treasury, AddBoardMember, RemoveBoardMember, UpdateBoardMemberShare, UpdateGovernanceConfig |
| **Content Governance** | Educational content management | AddContentFromStaging, UpdateGlobalQuizConfig, DeleteContentNode |

### Key Principles

- **Progressive Decentralization**: Board members (founders) initially control governance via VUC (Volume of Unmined Coins), but as more tokens are minted and staked by users, power shifts to the community
- **Anti-Spam Protection**: Minimum voting power required to create proposals
- **Transparency**: All proposals, votes, and executions are recorded on-chain
- **Predictable Thresholds**: Approval requirements are fixed when voting begins

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         GOVERNANCE SYSTEM ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│                              ┌─────────────────────────────────────┐             │
│                              │         governance_canister         │             │
│                              │         ═══════════════════         │             │
│                              │                                     │             │
│                              │  • Proposal creation & storage      │             │
│                              │  • Voting & support logic           │             │
│                              │  • Board member management          │             │
│                              │  • Governance config management     │             │
│                              │  • Proposal execution orchestration │             │
│                              │                                     │             │
│                              └──────────────┬──────────────────────┘             │
│                                             │                                    │
│            ┌────────────────────────────────┼────────────────────────────────┐   │
│            │                                │                                │   │
│            ▼                                ▼                                ▼   │
│  ┌──────────────────┐            ┌──────────────────┐            ┌──────────────┐│
│  │   staking_hub    │            │ treasury_canister │           │learning_engine││
│  │   ═══════════    │            │ ═════════════════ │           │══════════════││
│  │                  │            │                   │           │              ││
│  │  • VUC (Volume   │            │  • Token custody  │           │  • Content   ││
│  │    of Unmined    │            │  • Transfer       │           │    storage   ││
│  │    Coins)        │            │    execution      │           │  • Quiz      ││
│  │  • Total staked  │            │  • Allowance      │           │    config    ││
│  │  • User voting   │            │    management     │           │              ││
│  │    power lookup  │            │                   │           │              ││
│  │                  │            │                   │           │              ││
│  └──────────────────┘            └───────────────────┘           └──────────────┘│
│            ▲                                                            ▲        │
│            │                                                            │        │
│  ┌──────────────────────────────────────────────────────────────────────┘        │
│  │                                                                               │
│  │  ┌──────────────────┐            ┌──────────────────┐                         │
│  │  │ staging_assets   │            │   media_assets   │                         │
│  │  │ ════════════════ │            │ ════════════════ │                         │
│  │  │                  │            │                  │                         │
│  │  │  • Staged content│            │  • Permanent     │                         │
│  │  │  • Chunk retrieval│           │    media storage │                         │
│  │  │  • Status tracking│           │  • Immutable     │                         │
│  │  │                  │            │                  │                         │
│  │  └──────────────────┘            └──────────────────┘                         │
│  │                                                                               │
└──┴───────────────────────────────────────────────────────────────────────────────┘
```

### Canister Responsibilities

| Canister | Responsibility |
|----------|----------------|
| `governance_canister` | Central hub for all governance logic: proposal CRUD, voting, execution dispatch |
| `staking_hub` | Provides voting power data: VUC for board members, staked balances for users |
| `treasury_canister` | Holds treasury funds, executes approved transfers |
| `learning_engine` | Stores educational content, receives content updates from governance |
| `staging_assets` | Temporary storage for content awaiting approval |
| `media_assets` | Permanent storage for video/audio/image files |

---

## Voting Power Model

The governance system uses a **dual-track voting power model**:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VOTING POWER MODEL                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   MAX_SUPPLY = 4.75 Billion GHC                                                  │
│                                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │   VUC (Volume of Unmined Coins) = MAX_SUPPLY - total_allocated          │   │
│   │                                                                          │   │
│   │   This is the pool of voting power shared by BOARD MEMBERS only         │   │
│   │   As more tokens are minted/allocated, VUC decreases                    │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│   ╔═══════════════════════════╗     ╔═══════════════════════════╗               │
│   ║     BOARD MEMBERS         ║     ║     REGULAR USERS         ║               │
│   ╠═══════════════════════════╣     ╠═══════════════════════════╣               │
│   ║                           ║     ║                           ║               │
│   ║  Voting Power =           ║     ║  Voting Power =           ║               │
│   ║    VUC × (share% / 100)   ║     ║    staked_balance         ║               │
│   ║                           ║     ║                           ║               │
│   ║  Example:                 ║     ║  Example:                 ║               │
│   ║    VUC = 3.75B            ║     ║    User staked = 10,000   ║               │
│   ║    Share = 40%            ║     ║    Voting Power = 10,000  ║               │
│   ║    VP = 1.5B              ║     ║                           ║               │
│   ║                           ║     ║                           ║               │
│   ╚═══════════════════════════╝     ╚═══════════════════════════╝               │
│                                                                                  │
│   PROGRESSIVE DECENTRALIZATION:                                                  │
│   ─────────────────────────────                                                  │
│                                                                                  │
│   Early Stage:     VUC = 4.5B (95%)  │  Users = 0.25B (5%)                       │
│   Mid Stage:       VUC = 2.5B (53%)  │  Users = 2.25B (47%)                      │
│   Mature Stage:    VUC = 0    (0%)   │  Users = 4.75B (100%)                     │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Board Member Shares

Board members share the VUC voting power based on their **percentage share**:

| Member | Share | If VUC = 3.75B |
|--------|-------|----------------|
| Member A | 50% | 1.875B voting power |
| Member B | 30% | 1.125B voting power |
| Member C | 20% | 0.75B voting power |
| **Total** | **100%** | **3.75B** |

**Important:** Board member shares always sum to exactly 100%.

---

## Proposal Types

### Overview Table

| Type | Category | Execution Target | Description |
|------|----------|------------------|-------------|
| `Treasury` | Operational | treasury_canister | Transfer tokens from treasury |
| `AddBoardMember` | Operational | governance_canister | Add new board member with share |
| `RemoveBoardMember` | Operational | governance_canister | Remove board member, redistribute share |
| `UpdateBoardMemberShare` | Operational | governance_canister | Change a member's percentage share |
| `UpdateGovernanceConfig` | Operational | governance_canister | Modify governance parameters |
| `AddContentFromStaging` | Content | learning_engine | Load content from staging |
| `UpdateGlobalQuizConfig` | Content | learning_engine | Change quiz settings |
| `DeleteContentNode` | Content | learning_engine | Remove content node |

---

### Treasury Proposal

Transfers tokens from the treasury to a recipient.

```
┌─────────────────────────────────────────────────────────────────┐
│  TREASURY PROPOSAL EXECUTION                                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  governance_canister                    treasury_canister        │
│         │                                      │                 │
│         │  execute_transfer(                   │                 │
│         │    recipient,                        │                 │
│         │    amount,                           │                 │
│         │    token_type,                       │                 │
│         │    proposal_id                       │                 │
│         │  )                                   │                 │
│         │─────────────────────────────────────►│                 │
│         │                                      │                 │
│         │                                      │ 1. Validate     │
│         │                                      │    allowance    │
│         │                                      │ 2. Transfer     │
│         │                                      │    tokens       │
│         │                                      │ 3. Record TX    │
│         │                                      │                 │
│         │         Result<block_index>          │                 │
│         │◄─────────────────────────────────────│                 │
│         │                                      │                 │
│         │ Set status = Executed                │                 │
│         │                                      │                 │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - recipient: Principal      // Wallet to receive tokens
  - amount: u64               // Amount in e8s (8 decimals)
  - token_type: TokenType     // GHC, USDC, or ICP
  - category: ProposalCategory // Marketing, Development, etc.
```

---

### AddBoardMember Proposal

Adds a new board member and redistributes shares proportionally.

```
┌─────────────────────────────────────────────────────────────────┐
│  ADD BOARD MEMBER EXECUTION                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  BEFORE:                        AFTER:                           │
│  ────────                       ──────                           │
│  Member A: 60%                  Member A: 48% (60% × 0.8)        │
│  Member B: 40%                  Member B: 32% (40% × 0.8)        │
│  ────────────                   New Member: 20%                  │
│  Total: 100%                    ─────────────                    │
│                                 Total: 100%                      │
│                                                                  │
│  Algorithm (Largest Remainder Method):                           │
│  1. Calculate remaining_pct = 100 - new_member_pct               │
│  2. For each existing member:                                    │
│     new_share = floor(old_share × remaining_pct / 100)           │
│  3. Distribute remainder points to members with                  │
│     largest fractional parts                                     │
│  4. Insert new member with their percentage                      │
│                                                                  │
│  Execution: LOCAL to governance_canister (no inter-canister call)│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - new_member: Principal    // Wallet address of new member
  - percentage: u8           // Share (1-99%)
```

---

### RemoveBoardMember Proposal  

Removes a board member and redistributes their share **equally** among remaining members.

```
┌─────────────────────────────────────────────────────────────────┐
│  REMOVE BOARD MEMBER EXECUTION                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  BEFORE:                        AFTER:                           │
│  ────────                       ──────                           │
│  Member A: 40%                  Member A: 50% (+10%)             │
│  Member B: 30%                  Member B: 40% (+10%)             │
│  Member C: 30%  ← REMOVE        (removed)                        │
│  ────────────                   ─────────────                    │
│  Total: 100%                    Total: 100%                      │
│                                                                  │
│  Algorithm (Equal Distribution):                                 │
│  1. Get removed member's share (e.g., 30%)                       │
│  2. Divide equally: 30% / 2 remaining = 15% each                 │
│  3. Handle remainder: first members get +1 if not divisible      │
│  4. Remove the member from storage                               │
│                                                                  │
│  Validation:                                                     │
│  - Cannot remove if only 1 member remains                        │
│  - Member must exist                                             │
│                                                                  │
│  Execution: LOCAL to governance_canister                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - member_to_remove: Principal   // The board member to remove
```

---

### UpdateBoardMemberShare Proposal

Updates an existing board member's percentage share.

```
┌─────────────────────────────────────────────────────────────────┐
│  UPDATE BOARD MEMBER SHARE EXECUTION                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  BEFORE:                        AFTER (Member A → 50%):          │
│  ────────                       ──────                           │
│  Member A: 40%                  Member A: 50%                    │
│  Member B: 35%                  Member B: 29% (proportional)     │
│  Member C: 25%                  Member C: 21% (proportional)     │
│  ────────────                   ─────────────                    │
│  Total: 100%                    Total: 100%                      │
│                                                                  │
│  Algorithm:                                                      │
│  1. Calculate remaining for others = 100 - new_percentage        │
│  2. Sum current other members' shares                            │
│  3. Proportionally redistribute among other members              │
│  4. Last member gets remainder to ensure total = 100%            │
│                                                                  │
│  Execution: LOCAL to governance_canister                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - member: Principal        // Board member to update
  - new_percentage: u8       // New share (1-99%)
```

---

different values of governance parameters can be changed

```
┌─────────────────────────────────────────────────────────────────┐
│  UPDATE GOVERNANCE CONFIG EXECUTION                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Configurable Parameters:                                        │
│  ─────────────────────────                                       │
│                                                                  │
│  ┌────────────────────────┬───────────────┬───────────────────┐  │
│  │ Parameter              │ Default       │ Description       │  │
│  ├────────────────────────┼───────────────┼───────────────────┤  │
│  │ min_voting_power       │ 150 tokens    │ Min VP to propose │  │
│  │ support_threshold      │ 15,000 tokens │ VP to go Active   │  │
│  │ approval_percentage    │ 30%           │ % of staked to    │  │
│  │                        │               │ pass proposal     │  │
│  └────────────────────────┴───────────────┴───────────────────┘  │
│                                                                  │
│  Validation:                                                     │
│  - At least one parameter must be specified                      │
│  - approval_percentage must be 1-100                             │
│                                                                  │
│  Execution: LOCAL - updates stable storage cells                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - new_min_voting_power: Option<u64>      // In tokens (not e8s)
  - new_support_threshold: Option<u64>     // In tokens (not e8s)
  - new_approval_percentage: Option<u8>    // 1-100
```

---

### AddContentFromStaging Proposal

Loads educational content from staging into learning engine.

```
┌─────────────────────────────────────────────────────────────────┐
│  ADD CONTENT FROM STAGING EXECUTION                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  governance_canister        staging_assets       learning_engine │
│         │                        │                      │        │
│         │ mark_loading()         │                      │        │
│         │───────────────────────►│                      │        │
│         │        OK              │                      │        │
│         │◄───────────────────────│                      │        │
│         │                        │                      │        │
│         │ start_content_load(proposal_id, ...)          │        │
│         │──────────────────────────────────────────────►│        │
│         │                        │                      │        │
│         │                        │  get_content_chunk() │        │
│         │                        │◄─────────────────────│        │
│         │                        │   ContentNode[]      │        │
│         │                        │─────────────────────►│        │
│         │                        │                      │        │
│         │                        │     (loop until      │        │
│         │                        │      all loaded)     │        │
│         │                        │                      │        │
│         │                        │  mark_loaded()       │        │
│         │                        │◄─────────────────────│        │
│         │        OK              │                      │        │
│         │◄──────────────────────────────────────────────│        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - staging_canister: Principal   // staging_assets canister ID
  - staging_path: String          // Content hash in staging
  - content_hash: String          // SHA256 verification
  - content_title: String         // Human-readable title
  - unit_count: u32               // Number of ContentNodes
```

---

### UpdateGlobalQuizConfig Proposal

Updates quiz configuration for ALL quizzes in the system.

```
┌─────────────────────────────────────────────────────────────────┐
│  UPDATE GLOBAL QUIZ CONFIG EXECUTION                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  governance_canister                    learning_engine          │
│         │                                      │                 │
│         │  update_global_quiz_config(          │                 │
│         │    new_reward_amount,                │                 │
│         │    new_pass_threshold,               │                 │
│         │    new_max_attempts                  │                 │
│         │  )                                   │                 │
│         │─────────────────────────────────────►│                 │
│         │                                      │                 │
│         │                                      │ Updates         │
│         │                                      │ GLOBAL_QUIZ_    │
│         │                                      │ CONFIG stable   │
│         │                                      │ cell            │
│         │                                      │                 │
│         │         Result<()>                   │                 │
│         │◄─────────────────────────────────────│                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - new_reward_amount: Option<u64>    // Quiz reward in e8s
  - new_pass_threshold: Option<u8>    // Pass threshold (e.g., 60%)
  - new_max_attempts: Option<u8>      // Daily attempts per quiz
  - new_max_daily_quizzes: Option<u8> // Max quizzes per day
  - new_max_weekly_quizzes: Option<u8>// Max quizzes per week
  - new_max_monthly_quizzes: Option<u8>// Max quizzes per month
  - new_max_yearly_quizzes: Option<u16>// Max quizzes per year
```

---

### DeleteContentNode Proposal

Removes a content node from the learning engine (with audit trail).

```
┌─────────────────────────────────────────────────────────────────┐
│  DELETE CONTENT NODE EXECUTION                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  governance_canister                    learning_engine          │
│         │                                      │                 │
│         │  delete_content_node(                │                 │
│         │    content_id,                       │                 │
│         │    proposal_id                       │                 │
│         │  )                                   │                 │
│         │─────────────────────────────────────►│                 │
│         │                                      │                 │
│         │                                      │ 1. Record to    │
│         │                                      │    VERSION_     │
│         │                                      │    HISTORY      │
│         │                                      │ 2. Remove from  │
│         │                                      │    CONTENT_     │
│         │                                      │    NODES        │
│         │                                      │ 3. Update       │
│         │                                      │    indexes      │
│         │                                      │                 │
│         │         Result<()>                   │                 │
│         │◄─────────────────────────────────────│                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Payload:
  - content_id: String       // ID of the node to delete
  - reason: String           // Reason for deletion (audit)
```

---

## Proposal Lifecycle

### State Machine

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           PROPOSAL STATE MACHINE                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│    ╔═══════════════════╗                           ╔═══════════════════╗        │
│    ║   BOARD MEMBER    ║                           ║   REGULAR USER    ║        │
│    ║     creates       ║                           ║     creates       ║        │
│    ╚═════════╤═════════╝                           ╚═════════╤═════════╝        │
│              │                                               │                   │
│              │ skip support phase                            ▼                   │
│              │                               ┌─────────────────────────┐         │
│              │                               │        PROPOSED         │         │
│              │                               │  ─────────────────────  │         │
│              │                               │  Support Period: 1 week │         │
│              │                               │  Needs: 15K VP + 2 users│         │
│              │                               └────────────┬────────────┘         │
│              │                                            │                      │
│              │                                   ┌────────┴────────┐             │
│              │                                   │                 │             │
│              │                        threshold met         period expires       │
│              │                                   │                 │             │
│              │                                   ▼                 ▼             │
│              │                           ┌─────────────┐   ┌─────────────┐       │
│              │                           │             │   │  REJECTED   │       │
│              └──────────────────────────►│   ACTIVE    │   │  (terminal) │       │
│                                          │             │   └─────────────┘       │
│              ┌───────────────────────────┤ Voting: 2wk │                         │
│              │                           │ required_   │                         │
│              │ At activation:            │ yes_votes   │                         │
│              │ Calculate                 │ is FIXED    │                         │
│              │ required_yes_votes =      └──────┬──────┘                         │
│              │ total_staked × 30%               │                                │
│              │ (or current %)                   │                                │
│              └──────────────────────────────────┤                                │
│                                                 │                                │
│                           ┌─────────────────────┴─────────────────────┐          │
│                           │                                           │          │
│                votes_yes >= required               votes_yes < required          │
│                           │                                           │          │
│                           ▼                                           ▼          │
│                 ┌─────────────────┐                         ┌─────────────┐      │
│                 │    APPROVED     │                         │  REJECTED   │      │
│                 │ (awaiting exec) │                         │  (terminal) │      │
│                 └────────┬────────┘                         └─────────────┘      │
│                          │                                                       │
│                execute_proposal()                                                │
│                          │                                                       │
│                          ▼                                                       │
│                 ┌─────────────────┐                                              │
│                 │    EXECUTED     │                                              │
│                 │   (terminal)    │                                              │
│                 └─────────────────┘                                              │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Proposal States

| State | Description | Transitions |
|-------|-------------|-------------|
| `Proposed` | Initial state for regular users. Gathering support. | → Active (threshold met) / → Rejected (timeout) |
| `Active` | Voting in progress. Board members start here. | → Approved (votes pass) / → Rejected (votes fail) |
| `Approved` | Voting passed, awaiting execution. | → Executed (after execute_proposal) |
| `Rejected` | Failed or expired. Terminal state. | None |
| `Executed` | Successfully completed. Terminal state. | None |

### Time Periods

| Period | Duration | Description |
|--------|----------|-------------|
| Support Period | 1 week | Time for regular user proposals to gather support |
| Voting Period | 2 weeks | Time for voting once proposal is Active |
| Resubmission Cooldown | 6 months | Time before a rejected proposal can be resubmitted |

---

## Configurable Governance Parameters

These parameters can be modified via `UpdateGovernanceConfig` proposals:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     CONFIGURABLE GOVERNANCE PARAMETERS                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │  PARAMETER                  │  DEFAULT            │  STORAGE              │   │
│  ├──────────────────────────────────────────────────────────────────────────┤   │
│  │  min_voting_power           │  150 tokens         │  MIN_VOTING_POWER_    │   │
│  │                             │  (150 × 10^8 e8s)   │  CONFIG               │   │
│  │                             │                     │                       │   │
│  │  Minimum voting power       │                     │                       │   │
│  │  required to create any     │                     │                       │   │
│  │  proposal (anti-spam)       │                     │                       │   │
│  ├──────────────────────────────────────────────────────────────────────────┤   │
│  │  support_threshold          │  15,000 tokens      │  SUPPORT_THRESHOLD_   │   │
│  │                             │  (15K × 10^8 e8s)   │  CONFIG               │   │
│  │                             │                     │                       │   │
│  │  Total VP needed to move    │  Also requires      │                       │   │
│  │  from Proposed → Active     │  2+ unique          │                       │   │
│  │                             │  supporters         │                       │   │
│  ├──────────────────────────────────────────────────────────────────────────┤   │
│  │  approval_percentage        │  30%                │  APPROVAL_PERCENTAGE_ │   │
│  │                             │                     │  CONFIG               │   │
│  │                             │                     │                       │   │
│  │  Percentage of total        │  Calculated when    │                       │   │
│  │  staked tokens required     │  proposal becomes   │                       │   │
│  │  for YES votes to pass      │  Active, stored in  │                       │   │
│  │                             │  required_yes_votes │                       │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│  IMPORTANT: required_yes_votes is FIXED when proposal moves to Active           │
│  ══════════                                                                      │
│                                                                                  │
│  This ensures:                                                                   │
│  • Predictability - voters know the target at voting start                       │
│  • Fairness - threshold can't be gamed by timing                                 │
│  • Stability - result doesn't change with staking activity                       │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Query Governance Config

```candid
get_governance_config() -> (
  nat64,   // min_voting_power (in tokens)
  nat64,   // support_threshold (in tokens)  
  nat64,   // support_period (in days)
  nat64,   // voting_period (in days)
  nat64,   // resubmission_cooldown (in days)
  nat8     // approval_percentage (1-100)
)
```

---

## Proposal Execution Flows

### Complete Flow for Each Proposal Type

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    PROPOSAL EXECUTION DISPATCH                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  execute_proposal(proposal_id)                                                   │
│         │                                                                        │
│         ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  MATCH proposal.proposal_type                                            │    │
│  └───────────────────────────────┬─────────────────────────────────────────┘    │
│                                  │                                               │
│    ┌─────────────────────────────┼─────────────────────────────────────────┐    │
│    │         │         │         │         │         │         │           │    │
│    ▼         ▼         ▼         ▼         ▼         ▼         ▼           ▼    │
│ Treasury  Add     Remove   Update   Update   Add      Update   Delete       │    │
│           Board   Board    Share    Gov      Content  Quiz     Content      │    │
│           Member  Member            Config   Staging  Config   Node         │    │
│    │         │         │         │         │         │         │           │    │
│    ▼         ▼         ▼         ▼         ▼         ▼         ▼           ▼    │
│ ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐         │
│ │Call │  │Local│  │Local│  │Local│  │Local│  │Call │  │Call │  │Call │         │
│ │treas│  │exec │  │exec │  │exec │  │exec │  │learn│  │learn│  │learn│         │
│ │ury  │  │     │  │     │  │     │  │     │  │ing  │  │ing  │  │ing  │         │
│ └─────┘  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘         │
│                                                                                  │
│  Then: proposal.status = Executed                                                │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Inter-Canister Communication

### Voting Power Lookup

```
┌─────────────────────────────────────────────────────────────────┐
│  FETCH VOTING POWER                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  governance_canister                      staking_hub            │
│         │                                      │                 │
│         │                                      │                 │
│  if is_board_member(user):                     │                 │
│         │                                      │                 │
│         │  get_vuc()                           │                 │
│         │─────────────────────────────────────►│                 │
│         │        VUC value                     │                 │
│         │◄─────────────────────────────────────│                 │
│         │                                      │                 │
│         │  voting_power = VUC × share% / 100   │                 │
│         │                                      │                 │
│  else:                                         │                 │
│         │                                      │                 │
│         │  fetch_user_voting_power(user)       │                 │
│         │─────────────────────────────────────►│                 │
│         │        staked_balance                │                 │
│         │◄─────────────────────────────────────│                 │
│         │                                      │                 │
│         │  voting_power = staked_balance       │                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Approval Threshold Calculation

```
┌─────────────────────────────────────────────────────────────────┐
│  CALCULATE APPROVAL THRESHOLD                                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  When proposal moves: Proposed → Active                          │
│         OR                                                       │
│  When board member creates proposal (starts Active)              │
│                                                                  │
│  governance_canister                      staking_hub            │
│         │                                      │                 │
│         │  get_global_stats()                  │                 │
│         │─────────────────────────────────────►│                 │
│         │        { total_staked, ... }         │                 │
│         │◄─────────────────────────────────────│                 │
│         │                                      │                 │
│         │  approval_pct = APPROVAL_PERCENTAGE_CONFIG             │
│         │                                      │                 │
│         │  required_yes_votes =                │                 │
│         │    total_staked × approval_pct / 100 │                 │
│         │                                      │                 │
│         │  proposal.required_yes_votes = ^^^   │ (STORED!)       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Security Considerations

### Access Control

| Action | Who Can Perform |
|--------|-----------------|
| Create proposal | Anyone with min voting power (150 tokens) |
| Support proposal | Any user with voting power |
| Vote on proposal | Any user with voting power |
| Finalize proposal | Anyone (timer also runs hourly) |
| Execute proposal | Anyone (only works on Approved proposals) |
| Set board shares (admin) | Controller, before lock |
| Lock board shares | Controller (one-time action) |

### Validation

- **Proposal creation**: Title 1-200 chars, description 1-5000 chars
- **Board member ops**: Shares always sum to 100%, minimum 1 member
- **Treasury**: Amount > 0, within allowance
- **Governance config**: approval_percentage 1-100

### Immutability

- Once a proposal is executed, it cannot be undone
- Version history is maintained for content changes
- Vote records are permanent

---

## API Reference

### Proposal Creation

```candid
// Treasury
create_treasury_proposal: (CreateTreasuryProposalInput) -> (Result<nat64, text>)

// Board Member Management
create_board_member_proposal: (CreateBoardMemberProposalInput) -> (Result<nat64, text>)
create_remove_board_member_proposal: (CreateRemoveBoardMemberProposalInput) -> (Result<nat64, text>)
create_update_board_member_share_proposal: (CreateUpdateBoardMemberShareProposalInput) -> (Result<nat64, text>)

// Governance Config
create_update_governance_config_proposal: (CreateUpdateGovernanceConfigProposalInput) -> (Result<nat64, text>)

// Content
create_add_content_proposal: (CreateAddContentProposalInput) -> (Result<nat64, text>)
create_update_quiz_config_proposal: (CreateUpdateQuizConfigProposalInput) -> (Result<nat64, text>)
create_delete_content_proposal: (CreateDeleteContentProposalInput) -> (Result<nat64, text>)
```

### Proposal Actions

```candid
support_proposal: (nat64) -> (Result<null, text>)
vote: (nat64, bool) -> (Result<null, text>)
finalize_proposal: (nat64) -> (Result<ProposalStatus, text>)
execute_proposal: (nat64) -> (Result<null, text>)
```

### Queries

```candid
get_proposal: (nat64) -> (opt Proposal) query
get_active_proposals: () -> (vec Proposal) query
get_all_proposals: () -> (vec Proposal) query
get_proposal_votes: (nat64) -> (vec VoteRecord) query
get_proposal_supporters: (nat64) -> (vec SupportRecord) query
has_voted: (nat64, principal) -> (bool) query
get_governance_config: () -> (nat64, nat64, nat64, nat64, nat64, nat8) query
get_user_voting_power: (principal) -> (Result<nat64, text>)
get_board_member_shares: () -> (vec BoardMemberShare) query
```

---

## Summary

The GreenHero Coin governance system provides:

- ✅ **10 Proposal Types** covering treasury, board management, config, and content
- ✅ **Progressive Decentralization** via VUC/staked balance model
- ✅ **Configurable Parameters** that can be updated via proposals
- ✅ **Predictable Thresholds** fixed at voting start
- ✅ **Anti-Spam Protection** via minimum voting power requirements
- ✅ **Full Audit Trail** for all proposals and votes
- ✅ **Resilient Content Loading** that survives upgrades
