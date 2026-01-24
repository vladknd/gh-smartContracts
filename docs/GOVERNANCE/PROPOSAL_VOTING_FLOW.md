# Proposal & Voting Flow

This document describes the complete lifecycle of proposals in the GHC Operational Governance system.

---

## Table of Contents

1. [Overview](#overview)
2. [Proposal Types](#proposal-types)
3. [Proposal States](#proposal-states)
4. [State Machine Diagram](#state-machine-diagram)
5. [Detailed Flow](#detailed-flow)
6. [Key Constants](#key-constants)
7. [API Reference](#api-reference)

---

## Overview

The governance system allows users to propose changes to the GHC system. The process differs based on whether the proposer is a **Board Member** or a **Regular User**.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PROPOSAL LIFECYCLE SUMMARY                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Board Member ──────────────────────────────────► ACTIVE (Voting)          │
│       │                                               │                     │
│       │                                               │  2 weeks            │
│       │                                               ▼                     │
│       │                                      ┌────────────────┐             │
│       │                                      │   APPROVED or  │             │
│       │                                      │    REJECTED    │             │
│       │                                      └────────────────┘             │
│       │                                               │                     │
│       │                                               ▼                     │
│       │                                      ┌────────────────┐             │
│       │                                      │    EXECUTED    │             │
│       │                                      │  (if approved) │             │
│                                              └────────────────┘             │
│                                                                             │
│   Regular User ────► PROPOSED ────► ACTIVE ────► APPROVED ────► EXECUTED    │
│                      (support)      (voting)                                │
│                      1 week         2 weeks                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Proposal Types

The governance system supports two types of proposals:

### 1. Treasury Proposals

Treasury proposals request spending from the GHC treasury. They transfer tokens to a specified recipient address.

**Parameters:**
| Parameter     | Description                                                    |
|---------------|----------------------------------------------------------------|
| `title`       | Title of the proposal (1-200 characters)                       |
| `description` | Detailed proposal description (1-5000 characters)              |
| `recipient`   | Wallet address to receive the funds                            |
| `amount`      | Amount of tokens to transfer (in e8s - 8 decimals)             |
| `token_type`  | Type of token: `GHC`, `USDC`, or `ICP` (only GHC supported now)|
| `category`    | Spending category (Marketing, Development, etc.)               |
| `external_link`| Optional link to supporting documentation                     |

**Execution:** Transfers the specified amount from the treasury to the recipient.

### 2. Board Member Proposals

Board member proposals add a new member to the governance board. The new member receives a percentage share of the VUC (Volume of Unmined Coins) voting power.

**Parameters:**
| Parameter     | Description                                                    |
|---------------|----------------------------------------------------------------|
| `title`       | Title of the proposal (1-200 characters)                       |
| `description` | Detailed proposal description (1-5000 characters)              |
| `new_member`  | Wallet address of the person to add as a board member          |
| `percentage`  | Percentage share to allocate (1-99%)                           |
| `external_link`| Optional link to supporting documentation                     |

**Execution:** 
1. Calculates proportional reduction for each existing board member
2. Reduces existing members' shares equally to accommodate the new percentage
3. Adds the new member with their allocated percentage
4. Ensures total shares remain at exactly 100%

**Example:** Adding a new member with 20% share to a board with members at 60%, 30%, 10%:
- Each existing member's share is multiplied by 0.8 (100% - 20% = 80%)
- New shares: 48%, 24%, 8%, 20% = 100%

---

## Proposal States

| State       | Description                                                                 |
|-------------|-----------------------------------------------------------------------------|
| `Proposed`  | Initial state for regular user proposals. Gathering community support (1 week). |
| `Active`    | Voting is open. Users can cast YES/NO votes for 2 weeks.                    |
| `Approved`  | Voting passed threshold. Awaiting manual execution.                         |
| `Rejected`  | Voting failed to meet threshold OR support period expired. Terminal state.  |
| `Executed`  | Proposal was approved and action completed (funds transferred or board member added). Terminal state. |

---

## State Machine Diagram

```
                              ┌───────────────────────────────────────────────┐
                              │           PROPOSAL CREATION                   │
                              │                                               │
                              │   create_proposal(input)                      │
                              │   - Requires 150+ VP to create                │
                              │   - Validates inputs                          │
                              └───────────────────────────────────────────────┘
                                                   │
                                                   │
                              ┌────────────────────┴────────────────────┐
                              │                                         │
                              ▼                                         ▼
                    ┌─────────────────┐                       ┌─────────────────┐
                    │  Board Member   │                       │  Regular User   │
                    │     Proposer    │                       │    Proposer     │
                    └─────────────────┘                       └─────────────────┘
                              │                                         │
                              │ Skip support phase                      │
                              │                                         ▼
                              │                               ┌─────────────────┐
                              │                               │    PROPOSED     │
                              │                               │                 │
                              │                               │ Gather Support: │
                              │                               │ - 15,000+ VP    │
                              │                               │ - 2+ supporters │
                              │                               │ - 1 week period │
                              │                               └─────────────────┘
                              │                                         │
                              │                                         │ support_proposal()
                              │                                         │ threshold met
                              │                                         │
                              ▼                                         ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │                         ACTIVE                          │
                    │                                                         │
                    │   Voting Period: 2 weeks (14 days)                      │
                    │   - Users vote YES or NO                                │
                    │   - Vote weight = voting power at time of vote          │
                    │                                                         │
                    └─────────────────────────────────────────────────────────┘
                                                   │
                                                   │ Timer expires OR
                                                   │ finalize_proposal() called
                                                   │
                              ┌────────────────────┴────────────────────┐
                              │                                         │
                              ▼                                         ▼
                    ┌─────────────────┐                       ┌─────────────────┐
                    │    APPROVED     │                       │    REJECTED     │
                    │                 │                       │                 │
                    │ votes_yes >=    │                       │ votes_yes <     │
                    │ 15,000 VP       │                       │ 15,000 VP       │
                    └─────────────────┘                       └─────────────────┘
                              │                                         │
                              │ execute_proposal()                      │
                              │ (manual)                                │ Terminal
                              ▼                                         │
                    ┌─────────────────┐                                 │
                    │    EXECUTED     │                                 │
                    │                 │◄────────────────────────────────┘
                    │ Funds sent to   │        (No transition possible)
                    │ recipient       │
                    └─────────────────┘
```

---

## Detailed Flow

### Phase 1: Proposal Creation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PROPOSAL CREATION                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   User calls: create_proposal(input)                                        │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ VALIDATION CHECKS                                                   │   │
│   │                                                                     │   │
│   │  1. Title: 1-200 characters                                         │   │
│   │  2. Description: 1-5000 characters                                  │   │
│   │  3. Amount: > 0                                                     │   │
│   │  4. Voting Power: >= 150 tokens (anti-spam)                         │   │
│   │  5. Treasury Allowance: amount <= allowance (for GHC)               │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ DETERMINE INITIAL STATE                                             │   │
│   │                                                                     │   │
│   │  IF proposer is Board Member:                                       │   │
│   │    → Status = ACTIVE                                                │   │
│   │    → voting_ends_at = now + 14 days                                 │   │
│   │                                                                     │   │
│   │  ELSE (Regular User):                                               │   │
│   │    → Status = PROPOSED                                              │   │
│   │    → voting_ends_at = now + 7 days (support period)                 │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 2: Support Phase (Regular Users Only)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SUPPORT PHASE                                     │
│                       (Status: PROPOSED)                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   User calls: support_proposal(proposal_id)                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ SUPPORT RULES                                                       │   │
│   │                                                                     │   │
│   │  • Only works when status == PROPOSED                               │   │
│   │  • Each user can only support once per proposal                     │   │
│   │  • Must have voting power > 0                                       │   │
│   │  • Support weight = user's current voting power                     │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ ACCUMULATION                                                        │   │
│   │                                                                     │   │
│   │  proposal.support_amount += user_voting_power                       │   │
│   │  proposal.supporter_count += 1                                      │   │
│   │                                                                     │   │
│   │  ┌─────────────────────────────────────────┐                        │   │
│   │  │ Example:                                │                        │   │
│   │  │   User A supports with 8,000 VP         │                        │   │
│   │  │   User B supports with 4,000 VP         │                        │   │
│   │  │   User C supports with 5,000 VP         │                        │   │
│   │  │   ─────────────────────────             │                        │   │
│   │  │   Total: 17,000 VP, 3 supporters        │                        │   │
│   │  │   → Threshold MET!                      │                        │   │
│   │  └─────────────────────────────────────────┘                        │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ THRESHOLD CHECK                                                     │   │
│   │                                                                     │   │
│   │  IF support_amount >= 15,000 VP  AND  supporter_count >= 2:         │   │
│   │    → Status = ACTIVE                                                │   │
│   │    → voting_ends_at = now + 14 days                                 │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   NOTE: Supporters do NOT automatically become YES voters.                  │
│         They must vote separately during the ACTIVE phase.                  │
│                                                                             │
│   EXPIRATION: If the support threshold is not met within 1 week (7 days),  │
│               the proposal is automatically REJECTED.                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 3: Voting Phase

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           VOTING PHASE                                      │
│                         (Status: ACTIVE)                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Duration: 2 weeks (14 days) from activation                               │
│                                                                             │
│   User calls: vote(proposal_id, approve: bool)                              │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ VOTING RULES                                                        │   │
│   │                                                                     │   │
│   │  • Only works when status == ACTIVE                                 │   │
│   │  • Only works within voting period (now <= voting_ends_at)          │   │
│   │  • Each user can only vote once per proposal                        │   │
│   │  • Must have voting power > 0                                       │   │
│   │  • Vote weight = user's current voting power                        │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ VOTE RECORDING                                                      │   │
│   │                                                                     │   │
│   │  IF approve == true:                                                │   │
│   │    proposal.votes_yes += voting_power                               │   │
│   │                                                                     │   │
│   │  IF approve == false:                                               │   │
│   │    proposal.votes_no += voting_power                                │   │
│   │                                                                     │   │
│   │  proposal.voter_count += 1                                          │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ VOTING POWER SOURCES                                                │   │
│   │                                                                     │   │
│   │  ┌─────────────────────┐    ┌─────────────────────┐                 │   │
│   │  │   Regular Users     │    │   Board Members     │                 │   │
│   │  ├─────────────────────┤    ├─────────────────────┤                 │   │
│   │  │                     │    │                     │                 │   │
│   │  │  Voting Power =     │    │  Voting Power =     │                 │   │
│   │  │  staked_balance     │    │  VUC × share%       │                 │   │
│   │  │                     │    │                     │                 │   │
│   │  │  (tokens earned     │    │  (weighted portion  │                 │   │
│   │  │   from quizzes,     │    │   of unmined coins) │                 │   │
│   │  │   currently staked) │    │                     │                 │   │
│   │  │                     │    │                     │                 │   │
│   │  └─────────────────────┘    └─────────────────────┘                 │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 4: Finalization

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FINALIZATION                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Triggered by:                                                             │
│     • Automatic timer (runs every hour)                                     │
│     • Manual call: finalize_proposal(proposal_id)                           │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ PRECONDITIONS                                                       │   │
│   │                                                                     │   │
│   │  • Status must be ACTIVE                                            │   │
│   │  • Current time must be > voting_ends_at                            │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ OUTCOME DETERMINATION                                               │   │
│   │                                                                     │   │
│   │                                                                     │   │
│   │      votes_yes >= 15,000 VP?                                        │   │
│   │              │                                                      │   │
│   │      ┌───────┴───────┐                                              │   │
│   │      │               │                                              │   │
│   │     YES             NO                                              │   │
│   │      │               │                                              │   │
│   │      ▼               ▼                                              │   │
│   │  ┌────────┐    ┌────────┐                                           │   │
│   │  │APPROVED│    │REJECTED│                                           │   │
│   │  └────────┘    └────────┘                                           │   │
│   │                                                                     │   │
│   │  NOTE: votes_no does NOT directly affect outcome.                   │   │
│   │        Approval requires YES votes >= threshold.                    │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 5: Execution (Manual)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           EXECUTION                                         │
│                       (Status: APPROVED)                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   User calls: execute_proposal(proposal_id)                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ PRECONDITIONS                                                       │   │
│   │                                                                     │   │
│   │  • Status must be APPROVED                                          │   │
│   │  • Treasury allowance must cover the amount                         │   │
│   │  • Token type must be supported (currently GHC only)                │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ EXECUTION STEPS                                                     │   │
│   │                                                                     │   │
│   │  1. Verify treasury allowance >= proposal.amount                    │   │
│   │  2. Call ICRC-1 ledger: icrc1_transfer                              │   │
│   │     - From: Governance canister                                     │   │
│   │     - To: proposal.recipient                                        │   │
│   │     - Amount: proposal.amount                                       │   │
│   │  3. Update treasury state:                                          │   │
│   │     - balance -= amount                                             │   │
│   │     - allowance -= amount                                           │   │
│   │     - total_transferred += amount                                   │   │
│   │  4. Set status = EXECUTED                                           │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   WHY MANUAL EXECUTION?                                                     │
│   ─────────────────────                                                     │
│   • Allows time for review before funds leave treasury                      │
│   • Provides opportunity to verify recipient address                        │
│   • Enables multi-sig or DAO approval in future iterations                  │
│   • Prevents immediate fund loss from malicious proposals                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Constants

| Constant                     | Value           | Description                                      |
|------------------------------|-----------------|--------------------------------------------------|
| `MIN_VOTING_POWER_TO_PROPOSE`| 150 tokens      | Minimum VP required to create a proposal         |
| `SUPPORT_PERIOD_NANOS`       | 7 days          | Duration of support phase for regular users      |
| `SUPPORT_THRESHOLD`          | 15,000 tokens   | VP required to move from Proposed → Active       |
| `MIN_SUPPORTERS`             | 2 users         | Minimum unique supporters required               |
| `APPROVAL_THRESHOLD`         | 15,000 tokens   | YES votes required for approval                  |
| `VOTING_PERIOD_NANOS`        | 14 days         | Duration of voting phase                         |
| `RESUBMISSION_COOLDOWN`      | 180 days        | Wait time before resubmitting rejected proposal  |

---

## API Reference

### Create Treasury Proposal
```
create_treasury_proposal(input: CreateTreasuryProposalInput) → Result<nat64, text>
```
Creates a new treasury spending proposal. Returns proposal ID on success.

### Create Board Member Proposal
```
create_board_member_proposal(input: CreateBoardMemberProposalInput) → Result<nat64, text>
```
Creates a proposal to add a new board member. Returns proposal ID on success.

### Create Proposal (Legacy)
```
create_proposal(input: CreateTreasuryProposalInput) → Result<nat64, text>
```
Alias for `create_treasury_proposal` for backward compatibility.

### Support Proposal
```
support_proposal(proposal_id: nat64) → Result<null, text>
```
Add your voting power as support for a proposal in `Proposed` state.

### Vote
```
vote(proposal_id: nat64, approve: bool) → Result<null, text>
```
Cast your vote on an `Active` proposal. `true` = YES, `false` = NO.

### Finalize Proposal
```
finalize_proposal(proposal_id: nat64) → Result<ProposalStatus, text>
```
Finalize voting and determine outcome. Usually called automatically.

### Execute Proposal
```
execute_proposal(proposal_id: nat64) → Result<null, text>
```
Execute an `Approved` proposal. For Treasury proposals, transfers funds. For Board Member proposals, adds the new member.

### Query Functions
```
get_proposal(id: nat64) → Option<Proposal>
get_active_proposals() → Vec<Proposal>
get_all_proposals() → Vec<Proposal>
get_proposal_votes(proposal_id: nat64) → Vec<VoteRecord>
get_proposal_supporters(proposal_id: nat64) → Vec<SupportRecord>
has_voted(proposal_id: nat64, voter: Principal) → bool
get_governance_config() → (nat64, nat64, nat64, nat64, nat64)
// Returns: (min_vp_to_propose, approval_threshold, support_period_days, voting_period_days, resubmission_cooldown_days)
```

---

## Complete Flow Diagram

```
                                    ┌──────────────┐
                                    │    START     │
                                    └──────┬───────┘
                                           │
                                           ▼
                              ┌────────────────────────┐
                              │   create_proposal()    │
                              │                        │
                              │  - Validate inputs     │
                              │  - Check 150+ VP       │
                              │  - Check board member  │
                              └────────────┬───────────┘
                                           │
                          ┌────────────────┴────────────────┐
                          │                                 │
                   Board Member?                     Regular User
                          │                                 │
                          │                                 ▼
                          │                    ┌────────────────────────┐
                          │                    │       PROPOSED         │
                          │                    │                        │
                          │                    │  support_proposal()    │
                          │                    │  - 15,000 VP needed    │
                          │                    │  - 2+ supporters       │
                          │                    │  - 7 day period        │
                          │                    └────────────┬───────────┘
                          │                                 │
                          │                        Threshold Met?
                          │                                 │
                          │                    ┌────────────┴───────────┐
                          │                    │                        │
                          │                   YES                       NO
                          │                    │                        │
                          │                    │               (stays in PROPOSED)
                          │                    │
                          └────────────────────┼────────────────────────┘
                                               │
                                               ▼
                              ┌────────────────────────────────┐
                              │            ACTIVE              │
                              │                                │
                              │   vote(proposal_id, bool)      │
                              │   - 14 day voting period       │
                              │   - YES or NO votes            │
                              │                                │
                              └────────────────┬───────────────┘
                                               │
                                        Timer Expires
                                               │
                                               ▼
                              ┌────────────────────────────────┐
                              │     finalize_proposal()        │
                              │                                │
                              │   votes_yes >= 15,000 VP?     │
                              └────────────────┬───────────────┘
                                               │
                          ┌────────────────────┴────────────────┐
                          │                                     │
                         YES                                   NO
                          │                                     │
                          ▼                                     ▼
                   ┌────────────┐                        ┌────────────┐
                   │  APPROVED  │                        │  REJECTED  │
                   └──────┬─────┘                        └────────────┘
                          │                                    │
                          │                                    │
                   execute_proposal()                     Terminal State
                          │
                          ▼
                   ┌────────────┐
                   │  EXECUTED  │
                   │            │
                   │ Funds sent │
                   └────────────┘
                          │
                          ▼
                   ┌────────────┐
                   │    END     │
                   └────────────┘
```

---

*Last updated: January 2026*
