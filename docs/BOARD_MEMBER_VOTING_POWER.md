# Board Member Voting Power Distribution

## Overview

This document outlines the design and implementation of the weighted VUC (Volume of Unmined Coins) voting power system for board members.

### Terminology

| Term | Definition |
|------|------------|
| **Founders** | Receive token allocation through vesting (0.5B MC over 10 years) via `founder_vesting` canister |
| **Board Members** | Exercise VUC voting power for governance decisions via `staking_hub` canister |

These are **separate systems** — a person can be a founder, a board member, both, or neither.

### Current Implementation

Board members are registered in `staking_hub` with weighted percentage shares of VUC voting power:

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

### Storage

```rust
// BOARD_MEMBER_SHARES: Principal -> u8 (percentage 1-100)
static BOARD_MEMBER_SHARES: StableBTreeMap<Principal, u8, Memory>

// BOARD_SHARES_LOCKED: bool (once true, shares cannot be changed via admin)
static BOARD_SHARES_LOCKED: StableCell<bool, Memory>
```

### Functions

| Function | Description |
|----------|-------------|
| `set_board_member_shares(Vec<BoardMemberShare>)` | Set all shares atomically (must sum to 100) |
| `lock_board_member_shares()` | Lock shares permanently (irreversible via admin) |
| `are_board_shares_locked()` | Check if shares are locked |
| `get_board_member_shares()` | Get all board members with percentages |
| `get_board_member_share(principal)` | Get specific member's percentage |
| `get_board_member_count()` | Get number of board members |
| `is_board_member(principal)` | Check if principal is a board member |
| `fetch_voting_power(principal)` | Get weighted VUC power for board members |

### Voting Power Calculation

```rust
async fn fetch_voting_power(user: Principal) -> u64 {
    // Check if user is a board member - return weighted VUC
    if let Some(percentage) = get_board_member_percentage(&user) {
        let vuc = get_vuc();
        // VUC * percentage / 100 (using u128 to avoid overflow)
        return ((vuc as u128 * percentage as u128) / 100) as u64;
    }
    
    // Otherwise, return user's staked balance...
}
```

---

## Admin Commands

### Set Board Member Shares

```bash
# Set board member shares (must total 100%)
dfx canister call staking_hub set_board_member_shares '(vec { 
  record { member = principal "BOARD_MEMBER_1_PRINCIPAL"; percentage = 60 : nat8 };
  record { member = principal "BOARD_MEMBER_2_PRINCIPAL"; percentage = 30 : nat8 };
  record { member = principal "BOARD_MEMBER_3_PRINCIPAL"; percentage = 10 : nat8 };
})'
```

### Query Board Members

```bash
# Get all board members with their shares
dfx canister call staking_hub get_board_member_shares

# Check if shares are locked
dfx canister call staking_hub are_board_shares_locked

# Get a specific member's share
dfx canister call staking_hub get_board_member_share '(principal "PRINCIPAL_HERE")'

# Check if someone is a board member
dfx canister call staking_hub is_board_member '(principal "PRINCIPAL_HERE")'

# Get count of board members
dfx canister call staking_hub get_board_member_count
```

### Lock Shares (Irreversible!)

```bash
# WARNING: This cannot be undone via admin functions!
# Future: governance proposals can modify locked shares
dfx canister call staking_hub lock_board_member_shares
```

---

## Adding New Board Members

When adding a new board member, their percentage must come from existing members.

### Process (Manual Reallocation)

1. Decide on new distribution with all board members
2. Call `set_board_member_shares()` with complete new list
3. Verify with `get_board_member_shares()`

**Example**: Adding Board Member D with 15%

| Member | Before | After |
|--------|--------|-------|
| A | 60% | 51% |
| B | 30% | 26% |
| C | 10% | 8% |
| D | — | 15% |
| **Total** | **100%** | **100%** |

```bash
dfx canister call staking_hub set_board_member_shares '(vec { 
  record { member = principal "board-a-principal"; percentage = 51 : nat8 };
  record { member = principal "board-b-principal"; percentage = 26 : nat8 };
  record { member = principal "board-c-principal"; percentage = 8 : nat8 };
  record { member = principal "board-d-principal"; percentage = 15 : nat8 };
})'
```

---

## Immutability & Governance

### Lock Mechanism

The `lock_board_member_shares()` function prevents admin changes:

```rust
if is_locked {
    return Err("Board member shares are permanently locked. Use governance to propose changes.");
}
```

### Future Governance Integration

The system is designed for future governance control:

1. **Current**: Controller can call `set_board_member_shares()`
2. **After Lock**: Admin functions blocked
3. **Future**: Governance proposals can modify shares by:
   - Creating a new proposal type for board member changes
   - If approved, calling an internal function to update shares
   - Requiring supermajority or unanimous board member approval

---

## Validation Rules

The `set_board_member_shares()` function enforces:

1. ✅ At least one board member required
2. ✅ No duplicate principals
3. ✅ Each percentage must be 1-100
4. ✅ Total must equal exactly 100
5. ✅ Cannot modify if shares are locked
6. ✅ Only controllers can call

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-01-08 | Separate founders from board members | Token vesting ≠ governance power |
| 2026-01-08 | Implement weighted VUC shares | Fair distribution of voting power |
| 2026-01-08 | Add lock mechanism | Immutability option for security |
| 2026-01-08 | Design for governance integration | Future-proof the system |
| 2026-01-08 | Remove legacy founder functions | Clean API, no confusion |

---

## Related Documents

- [BOARD_MEMBER_SHARE_OPTIONS.md](./BOARD_MEMBER_SHARE_OPTIONS.md) - Design options for reallocation
- [STAKING_MECHANICS.md](./STAKING_MECHANICS.md) - Staking and voting power overview
- [QUICK_REF.md](./QUICK_REF.md) - Admin command reference
