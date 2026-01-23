# Board Member Share Calculations

## Overview

Allocations for board members in the GreenHero governance system must always sum to exactly **100%**. When members are added or removed, the system must redistribute percentages while handling integer math and rounding errors fairly.

To ensure fairness and determinism, we use the **Largest Remainder Method**.

## 1. Adding a Board Member
**Goal**: Make room for the new member's share (e.g., 10%) by reducing existing members' shares proportionally.

### Algorithm
1.  Target total for existing members = `100 - NewMemberPercentage`.
2.  Scale each existing member: `NewShare = OldShare * (TargetTotal / 100)`.
3.  Calculate floor values and remainders.
4.  Distribute any missing points (due to floor rounding) to members with the largest remainders.
5.  Insert the new member.

### Example
-   **Initial**: A (60%), B (30%), C (10%). Total 100%.
-   **Action**: Add D with 10%.
-   **Target for A,B,C**: 90%.
-   **Calculation**:
    -   A: 60 * 0.9 = 54.0
    -   B: 30 * 0.9 = 27.0
    -   C: 10 * 0.9 = 9.0
-   **Result**: A (54%), B (27%), C (9%), D (10%). Total 100%.

## 2. Removing a Board Member (UPDATED)
**Goal**: Redistribute the removed member's share back to the remaining members **proportionally**, maintaining their relative power ratios.

> **Note**: Previous implementations used "Equal Split", which caused distortion (small stakeholders gained disproportionately). The current implementation uses proportional scaling.

### Algorithm
1.  Calculate current total of remaining members (`CurrentTotal`).
2.  Scale each member to 100: `NewShare = (CurrentShare * 100) / CurrentTotal`.
3.  Use the **Largest Remainder Method** to handle rounding:
    -   Calculate exact values with high precision.
    -   Take the floor.
    -   Distribute remaining points to members with the largest fractional remainders.

### Example (Proportional Restoration)
-   **Initial**: A (54%), B (27%), C (9%), D (10%). Total 100%.
-   **Action**: Remove D (10%).
-   **Remaining**: A, B, C.
-   **Current Total**: 54 + 27 + 9 = 90.
-   **Calculation**:
    -   A: 54 * (100/90) = 60.0
    -   B: 27 * (100/90) = 30.0
    -   C: 9  * (100/90) = 10.0
-   **Result**: A (60%), B (30%), C (10%). **Original state restored.**

### Why Largest Remainder Method?
Integer division always rounds down (`floor`). Simply rounding to the nearest integer doesn't guarantee the sum is 100.
-   Example: Split 100 among 3 people (33.33...).
-   Floor: 33, 33, 33 -> Sum 99. (Missing 1).
-   Round: 33, 33, 33 -> Sum 99.
-   Largest Remainder:
    -   Values: 33.33, 33.33, 33.33.
    -   Floors: 33, 33, 33.
    -   Remainders: 0.33, 0.33, 0.33.
    -   Sort by Remainder: Tie.
    -   Deterministically give +1 to first in list: 34, 33, 33. Sum 100.

This ensures the system never breaks invariant of Sum == 100.
