# Board Member Share Reallocation Options

## Overview

This document outlines the options for reallocating VUC voting power shares when adding new board members.

When a new board member is added, their percentage must come from existing members. Below are the strategies for handling this reallocation.

---

## Options for Adding New Board Members

### Option 1: Proportional Dilution (Democratic)

When a new board member is added with X%, **all existing members are diluted proportionally**.

**Example**: Adding Board Member D with 20%

| Member | Before | After (×0.8) |
|--------|--------|--------------|
| A      | 60%    | 48%          |
| B      | 30%    | 24%          |
| C      | 10%    | 8%           |
| D      | —      | 20%          |
| **Total** | **100%** | **100%** |

**Pros**:
- Fair and automatic
- No favoritism in dilution
- Simple mental model

**Cons**:
- Existing members lose power without explicit consent
- May not reflect actual agreements between members

---

### Option 2: Manual Reallocation (Explicit Control) ⭐ IMPLEMENTED

Admin must explicitly specify ALL board member percentages when making any change. The system validates they sum to 100%.

**Example**: 
```bash
dfx canister call staking_hub set_board_member_shares '(vec { 
  record { member = principal "aaaaa-aa"; percentage = 50 : nat8 };
  record { member = principal "bbbbb-bb"; percentage = 25 : nat8 };
  record { member = principal "ccccc-cc"; percentage = 10 : nat8 };
  record { member = principal "ddddd-dd"; percentage = 15 : nat8 };
})'
```

**Pros**:
- Full control over allocation
- Explicit consent required
- Clear audit trail
- No surprises for existing members
- Simple and reliable implementation

**Cons**:
- Requires coordination among board members
- Admin must decide allocation

---

### Option 3: Reserved Pool (Pre-planned Growth)

Start with a "reserve" pool. Board members don't initially take 100%—they take less (e.g., 80%). New members draw from the reserve.

**Example**: Initial setup

| Holder   | Percentage |
|----------|------------|
| A        | 48%        |
| B        | 24%        |
| C        | 8%         |
| Reserve  | 20%        |
| **Total** | **100%** |

Board Member D can claim up to 20% from reserve without diluting A, B, or C.

**Pros**:
- Pre-planned for growth
- Existing members are protected
- Clear expectation setting

**Cons**:
- Reserve pool can't vote (or needs special handling)
- Less flexible if reserve runs out
- More complex initial setup

---

### Option 4: Governance Proposal (Decentralized)

Adding or modifying board members requires a **governance proposal** that existing board members must vote on.

**Workflow**:
1. Create proposal: "Add Board Member D with 15%, reduce A→51%, B→26%, C→8%"
2. Existing board members vote using their VUC power
3. If approved (meets threshold), changes take effect automatically

**Pros**:
- Democratic and transparent
- On-chain governance
- Full audit trail
- Board members collectively decide

**Cons**:
- More complex implementation
- Slower (requires voting period)
- Requires proposal infrastructure for board member management

---

### Option 5: Hybrid - Admin Reallocation with Governance Guardrails

Admin can adjust shares, but with additional safeguards:
- Total must always equal 100%
- Each adjustment logs an on-chain event
- Optional: require multi-sig approval
- Optional: time-lock for changes (e.g., 24-hour delay)

**Pros**:
- Balance of control and safety
- Audit trail
- Can add more guardrails over time

**Cons**:
- More implementation complexity
- Multi-sig requires external tooling

---

## Current Implementation: Option 2

**Manual Reallocation** was chosen because:

| Criterion        | Why Option 2 Wins |
|------------------|-------------------|
| **Safety**       | Explicit allocation prevents accidental dilution |
| **Reliability**  | Simple logic, no complex calculations |
| **Efficiency**   | Single atomic call updates all shares |
| **Flexibility**  | Works for any number of board members, any distribution |
| **Auditability** | Clear record of who set what and when |
| **Trust**        | Requires deliberate decision, no surprises |

---

## Future Enhancement: Governance Integration

The system is designed to support **Option 4 (Governance Proposal)** in the future:

1. Create a new proposal type: `BoardMemberChange`
2. Proposal includes new share distribution
3. Existing board members vote using VUC power
4. If approved, internal function updates shares (bypassing lock)
5. Could require supermajority or unanimous approval

This allows the lock mechanism to block admin changes while still permitting governance-approved modifications.

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-01-08 | Document created | Evaluating options for board member share reallocation |
| 2026-01-08 | Chose Option 2 | Simplest, most explicit, future-proof for governance |
| 2026-01-08 | Removed legacy functions | Clean API with only board member terminology |

---

## Related Documents

- [BOARD_MEMBER_VOTING_POWER.md](./BOARD_MEMBER_VOTING_POWER.md) - Implementation details
- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) - Staking and voting power overview
