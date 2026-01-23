# Board Member Voting Power Distribution

## Overview

This document outlines the design and implementation of the weighted VUC (Volume of Unmined Coins) voting power system for board members.

### Terminology

| Term | Definition |
|------|------------|
| **Founders** | Receive token allocation through vesting (0.5B MC over 10 years) via `founder_vesting` canister |
| **Board Members** | Exercise VUC voting power for governance decisions via `operational_governance` canister |

These are **separate systems** — a person can be a founder, a board member, both, or neither.

### Architecture (Updated January 2026)

Board member management has been moved from `staking_hub` to `operational_governance` to:
- Eliminate circular dependencies between canisters
- Keep governance data with the governance canister
- Enable governance proposals to manage board membership

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      OPERATIONAL GOVERNANCE                              │
├─────────────────────────────────────────────────────────────────────────┤
│  Board Member Storage:                                                  │
│    - BOARD_MEMBER_SHARES (Principal → percentage)                       │
│    - BOARD_SHARES_LOCKED (bool)                                         │
│                                                                         │
│  fetch_voting_power(user):                                              │
│    if is_board_member_local(user):                                      │
│      vuc ← call staking_hub.get_vuc()  ──────────────────┐              │
│      return vuc * percentage / 100                       │              │
│    else:                                                 │              │
│      return call staking_hub.fetch_user_voting_power() ──┼──────────┐   │
└──────────────────────────────────────────────────────────┼──────────┼───┘
                                                           │          │
                                                           ▼          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          STAKING HUB                                     │
├─────────────────────────────────────────────────────────────────────────┤
│  get_vuc(): Returns VUC = MAX_SUPPLY - total_allocated                  │
│  fetch_user_voting_power(user): Queries shard for staked_balance        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Current Implementation

Board members are registered in `operational_governance` with weighted percentage shares of VUC voting power:

| Board Member | VUC Share |
|--------------|-----------|
| Board Member 1 | 60% |
| Board Member 2 | 30% |
| Board Member 3 | 10% |
| **Total** | **100%** |

When `fetch_voting_power()` is called for a board member:
```
voting_power = VUC × (percentage / 100)
```

---

## Implementation

### Storage (in operational_governance)

```rust
// BOARD_MEMBER_SHARES: Principal -> u8 (percentage 1-100)
static BOARD_MEMBER_SHARES: StableBTreeMap<Principal, u8, Memory>

// BOARD_SHARES_LOCKED: bool (once true, shares cannot be changed via admin)
static BOARD_SHARES_LOCKED: StableCell<bool, Memory>
```

### Functions (in operational_governance)

| Function | Description |
|----------|-------------|
| `set_board_member_shares(Vec<BoardMemberShare>)` | Set all shares atomically (must sum to 100) |
| `lock_board_member_shares()` | Lock shares (after lock, use governance proposals) |
| `are_board_shares_locked()` | Check if shares are locked |
| `get_board_member_shares()` | Get all board members with percentages |
| `get_board_member_share(principal)` | Get specific member's percentage |
| `get_board_member_count()` | Get number of board members |
| `is_board_member(principal)` | Check if principal is a board member |

### Voting Power Calculation

```rust
async fn fetch_voting_power(user: Principal) -> Result<u64, String> {
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Check if user is a board member - return weighted VUC
    if let Some(percentage) = get_board_member_percentage_local(&user) {
        // Get VUC from staking hub
        let (vuc,): (u64,) = ic_cdk::call(
            staking_hub_id,
            "get_vuc",
            ()
        ).await.map_err(|e| format!("Failed to get VUC: {:?}", e))?;
        
        // Calculate weighted voting power: VUC * percentage / 100
        return Ok(((vuc as u128 * percentage as u128) / 100) as u64);
    }
    
    // For regular users, query their staked balance from staking hub
    let (voting_power,): (u64,) = ic_cdk::call(
        staking_hub_id,
        "fetch_user_voting_power",
        (user,)
    ).await.map_err(|e| format!("Failed to get voting power: {:?}", e))?;
    
    Ok(voting_power)
}
```

---

## Admin Commands

### Set Board Member Shares

```bash
# Set board member shares (must total 100%)
dfx canister call operational_governance set_board_member_shares '(vec { 
  record { member = principal "BOARD_MEMBER_1_PRINCIPAL"; percentage = 60 : nat8 };
  record { member = principal "BOARD_MEMBER_2_PRINCIPAL"; percentage = 30 : nat8 };
  record { member = principal "BOARD_MEMBER_3_PRINCIPAL"; percentage = 10 : nat8 };
})'
```

### Query Board Members

```bash
# Get all board members with their shares
dfx canister call operational_governance get_board_member_shares

# Check if shares are locked
dfx canister call operational_governance are_board_shares_locked

# Get a specific member's share
dfx canister call operational_governance get_board_member_share '(principal "PRINCIPAL_HERE")'

# Check if someone is a board member
dfx canister call operational_governance is_board_member '(principal "PRINCIPAL_HERE")'

# Get count of board members
dfx canister call operational_governance get_board_member_count
```

### Lock Shares

```bash
# WARNING: After locking, use governance proposals to add new members
dfx canister call operational_governance lock_board_member_shares
```

---

## Adding New Board Members

### Via Admin (Before Locking)

When shares are not locked, admins can call `set_board_member_shares()` with the complete new list.

### Via Governance Proposal (After Locking)

Once shares are locked, new board members can only be added via governance proposal:

1. Create a board member proposal with `create_board_member_proposal()`
2. Proposal goes through voting (board members vote or regular users support then vote)
3. If approved, call `execute_proposal()` to add the new member
4. Existing members' shares are proportionally reduced

**Example**: Adding Board Member D with 20%

| Member | Before | After (proportional reduction) |
|--------|--------|--------------------------------|
| A | 60% | 48% (60 × 0.80) |
| B | 30% | 24% (30 × 0.80) |
| C | 10% | 8% (10 × 0.80) |
| D | — | 20% |
| **Total** | **100%** | **100%** |

---

## Validation Rules

The `set_board_member_shares()` function enforces:

1. ✅ At least one board member required
2. ✅ No duplicate principals
3. ✅ Each percentage must be 1-100
4. ✅ Total must equal exactly 100
5. ✅ Cannot modify if shares are locked (must use governance proposals)
6. ✅ Only controllers can call

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-01-08 | Separate founders from board members | Token vesting ≠ governance power |
| 2026-01-08 | Implement weighted VUC shares | Fair distribution of voting power |
| 2026-01-08 | Add lock mechanism | Immutability option for security |
| 2026-01-08 | Design for governance integration | Future-proof the system |
| 2026-01-13 | **Move board members to operational_governance** | Eliminate circular dependency, single source of truth |
| 2026-01-13 | **Add AddBoardMember proposal type** | Enable governance-driven board changes |

---

## Related Documents

- [BOARD_MEMBER_SHARE_OPTIONS.md](./BOARD_MEMBER_SHARE_OPTIONS.md) - Design options for reallocation
- [PROPOSAL_VOTING_FLOW.md](./PROPOSAL_VOTING_FLOW.md) - Proposal lifecycle and board member proposals
- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) - Staking and voting power overview
- [QUICK_REF.md](./QUICK_REF.md) - Admin command reference
