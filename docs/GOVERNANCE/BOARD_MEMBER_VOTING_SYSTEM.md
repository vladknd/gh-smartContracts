# Board Member Voting Power System

## Overview

The GreenHero governance system uses a sophisticated board member voting power system based on **Basis Points (BPS)** for precision and a special **Sentinel** role to enable "board member approval" without significantly affecting voting outcomes.

## Key Concepts

### 1. Basis Points (BPS)

Instead of using percentages (0-100), the system uses **Basis Points** where:
- **10,000 BPS = 100.00%**
- **100 BPS = 1.00%**
- **1 BPS = 0.01%**

This provides 100x more precision than simple percentages, allowing for fair distribution among any number of board members.

### 2. VUC (Volume of Unmined Coins)

VUC represents the voting power allocated to the board as a whole. Each regular board member receives a portion of the VUC based on their BPS share. The total VUC is queried from the staking hub.

### 3. The Sentinel Role

The **Sentinel** is a special board member who:
- Has **exactly 1 unit of VUC** voting power (1 e8s)
- Is **NOT** part of the BPS distribution
- Can vote on proposals to satisfy "requires board member vote" rules
- Does not significantly affect voting outcomes due to minimal power

**Why a Sentinel?**
Many governance systems require at least one board member to vote for a proposal to be considered valid. With regular board members having substantial voting power, their vote would heavily bias the outcome. The sentinel provides a way to "approve" proposals without tipping the scales.

---

## Board Member Types

| Type | Voting Power | Stored In | Affected by Redistribution |
|------|--------------|-----------|---------------------------|
| **Regular Member** | `(VUC × share_bps) / 10,000` | `BOARD_MEMBER_SHARES` map | Yes |
| **Sentinel** | Exactly 1 e8s | `SENTINEL_MEMBER` cell | No |

---

## Voting Power Calculation

### Cumulative Partitioning (Zero-Dust Algorithm)

The system uses **Cumulative Partitioning** to ensure the sum of all member powers exactly equals the total VUC, with no "dust" lost to rounding.

**Traditional (Bad) Approach:**
```
Power_A = (VUC × 3333) / 10000 → rounds down
Power_B = (VUC × 3333) / 10000 → rounds down
Power_C = (VUC × 3334) / 10000 → rounds down
Total = Power_A + Power_B + Power_C ≠ VUC (dust lost!)
```

**Cumulative Partitioning (Good) Approach:**
```
cumulative[0] = 0
cumulative[1] = (VUC × 3333) / 10000
cumulative[2] = (VUC × (3333 + 3333)) / 10000
cumulative[3] = (VUC × (3333 + 3333 + 3334)) / 10000 = VUC

Power_A = cumulative[1] - cumulative[0]
Power_B = cumulative[2] - cumulative[1]
Power_C = cumulative[3] - cumulative[2]
Total = Power_A + Power_B + Power_C = VUC ✓
```

---

## Adding a Board Member

When adding a new board member via `create_board_member_proposal`:

### Input
- `new_member`: Principal of the new member
- `share_bps`: BPS share to allocate (1-9900)

### Algorithm (Largest Remainder Method)

1. **Calculate remaining BPS**: `remaining = 10,000 - new_member_share`
2. **Scale existing members proportionally**:
   ```
   new_share = old_share × (remaining / current_total)
   ```
3. **Use Largest Remainder Method** to distribute any fractional BPS:
   - Calculate floor value and remainder for each member
   - Sort by remainder descending
   - Give +1 BPS to members with largest remainders until target is reached
4. **Add new member** with their specified share
5. **Verify total** equals exactly 10,000 BPS

### Example

**Before**: A (6,000), B (3,000), C (1,000) = 10,000 BPS
**Add D with 1,000 BPS**:

1. Remaining for A, B, C = 10,000 - 1,000 = 9,000 BPS
2. Current total of A, B, C = 10,000 BPS
3. Scale:
   - A: 6,000 × (9,000 / 10,000) = 5,400
   - B: 3,000 × (9,000 / 10,000) = 2,700
   - C: 1,000 × (9,000 / 10,000) = 900
4. D: 1,000
5. **After**: A (5,400), B (2,700), C (900), D (1,000) = 10,000 BPS ✓

---

## Removing a Board Member

When removing a board member via `create_remove_board_member_proposal`:

### Input
- `member_to_remove`: Principal of the member to remove

### Constraints
- **Cannot remove the Sentinel** (use `UpdateSentinel` proposal instead)
- **Cannot remove the last regular member**

### Algorithm

1. **Get remaining members** (exclude the one being removed)
2. **Scale remaining members to 10,000 BPS**:
   ```
   new_share = old_share × (10,000 / remaining_total)
   ```
3. **Use Largest Remainder Method** to handle fractional BPS

### Example

**Before**: A (5,400), B (2,700), C (900), D (1,000) = 10,000 BPS
**Remove D**:

1. Remaining: A, B, C with total = 9,000 BPS
2. Scale to 10,000:
   - A: 5,400 × (10,000 / 9,000) = 6,000
   - B: 2,700 × (10,000 / 9,000) = 3,000
   - C: 900 × (10,000 / 9,000) = 1,000
3. **After**: A (6,000), B (3,000), C (1,000) = 10,000 BPS ✓

---

## Updating a Board Member's Share

When updating a share via `create_update_board_member_share_proposal`:

### Input
- `member`: Principal of the member to update
- `new_share_bps`: New BPS share (1-9900)

### Constraints
- **Cannot update the Sentinel's share** (sentinel always has 1 unit)

### Algorithm

1. **Calculate remaining BPS for others**: `remaining = 10,000 - new_share`
2. **Scale other members proportionally**
3. **Use Largest Remainder Method**
4. **Add the updated member** with their new share

---

## Changing the Sentinel

When changing the sentinel via `create_update_sentinel_proposal`:

### Input
- `new_sentinel`: Principal of the new sentinel

### Constraints
- **Cannot be anonymous**
- **Cannot be an existing regular board member**

### Effect
- Old sentinel loses their role (no voting power)
- New sentinel gains exactly 1 unit of VUC voting power

---

## Admin Setup (Before Locking)

Before the system is locked, administrators can set up the initial configuration:

### 1. Set Regular Board Members
```
set_board_member_shares([
    { member: Principal_A, share_bps: 5000, is_sentinel: false },
    { member: Principal_B, share_bps: 3000, is_sentinel: false },
    { member: Principal_C, share_bps: 2000, is_sentinel: false },
])
```
Total must equal 10,000 BPS.

### 2. Set Sentinel
```
set_sentinel_member(Sentinel_Principal)
```
Must be different from all regular members.

### 3. Lock the System
```
lock_board_member_shares()
```
After locking:
- All changes require governance proposals
- Both regular shares AND sentinel require proposals to modify

---

## Query Functions

| Function | Returns |
|----------|---------|
| `get_board_member_shares()` | All members including sentinel with BPS shares |
| `get_board_member_share(principal)` | BPS share for a specific regular member |
| `get_sentinel_member()` | Principal of the sentinel (or None) |
| `get_board_member_count()` | Total count (regular + sentinel if set) |
| `is_board_member(principal)` | True if principal is any type of board member |
| `are_board_shares_locked()` | Whether admin changes are locked |

---

## Proposal Types

| Proposal Type | Description | Can Remove Sentinel? |
|--------------|-------------|---------------------|
| `AddBoardMember` | Add new regular member with BPS share | N/A |
| `RemoveBoardMember` | Remove regular member, redistribute BPS | No |
| `UpdateBoardMemberShare` | Change regular member's BPS share | No |
| `UpdateSentinel` | Change who holds the sentinel role | N/A (replaces) |

---

## Invariants

The system maintains these invariants:

1. **BPS Total = 10,000**: Sum of all regular member shares always equals 10,000
2. **Sentinel is Separate**: Sentinel is never in the BPS map
3. **Sentinel Power = 1**: Sentinel always has exactly 1 e8s of voting power
4. **Voting Power Sum = VUC + 1**: Total voting power of all board members equals VUC + 1 (the sentinel's 1 unit)
5. **No Dust**: Cumulative partitioning ensures exact distribution
6. **Deterministic**: Same inputs always produce same outputs (sorted by Principal)

---

## Implementation Files

- **`types.rs`**: Data structures (`BoardMemberShare`, `AddBoardMemberPayload`, etc.)
- **`state.rs`**: Storage (`BOARD_MEMBER_SHARES`, `SENTINEL_MEMBER`)
- **`service.rs`**: Core logic (`fetch_voting_power`, `is_sentinel_local`, etc.)
- **`lib.rs`**: Public API (proposal creation, execution, queries)
- **`constants.rs`**: Configuration (`BPS_TOTAL`, `MAX_MEMBER_SHARE_BPS`, etc.)

---

## Example: Full Lifecycle

1. **Initial Setup** (Admin):
   ```
   set_sentinel_member(Alice)  // Alice gets 1 unit of VUC
   set_board_member_shares([
       { Bob, 5000 BPS },
       { Carol, 3000 BPS },
       { Dave, 2000 BPS }
   ])
   lock_board_member_shares()
   ```

2. **Add Eve** (Proposal):
   - Eve wants 2000 BPS
   - After approval and execution:
     - Bob: 4000 BPS
     - Carol: 2400 BPS
     - Dave: 1600 BPS
     - Eve: 2000 BPS
     - Alice: Still sentinel (1 unit)

3. **Remove Dave** (Proposal):
   - After approval and execution:
     - Bob: 4762 BPS
     - Carol: 2857 BPS
     - Eve: 2381 BPS
     - Alice: Still sentinel (1 unit)

4. **Change Sentinel** (Proposal):
   - Frank becomes sentinel
   - Alice loses voting power
   - Frank gets 1 unit of VUC

---

## Security Considerations

1. **Sentinel cannot be anonymous** - Prevents orphaned voting power
2. **Sentinel cannot be regular member** - Prevents double-counting
3. **Cannot remove sentinel directly** - Must use UpdateSentinel
4. **Locked state requires proposals** - Ensures governance for all changes
5. **Deterministic redistribution** - Sorted by Principal for reproducibility
