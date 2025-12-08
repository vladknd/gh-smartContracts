# GreenHero Coin (GHC) Dapp Architecture

## Overview
The GreenHero Coin (GHC) dapp is a blockchain-based educational platform built on the Internet Computer (ICP). It incentivizes learning through a "Pre-Mint & Allocate" tokenomics model, where users earn GHC tokens by completing educational activities (quizzes).

## Tokenomics
- **Total Supply**: 8.2 Billion GHC (Fixed at Genesis).
- **Partitions**:
    1.  **Market Partition (4.1B)**:
        -   **Founders**: 0.5B (0.3B + 0.2B).
        -   **Treasury**: 3.6B (Managed by Operational Governance).
    2.  **Mined Utility Partition (4.1B)**:
        -   Held by the **Staking & Mining Hub**.
        -   Users "mine" these tokens by completing quizzes.

## System Components

### 1. User Profile (Sharded)
- **Role**: The User's Personal Record & Micro-Bank.
- **Responsibilities**:
    -   **Primary Entry Point** for all user interactions.
    -   Stores User Profile (Email, Name, etc.) and Quiz Progress.
    -   **Micro-Bank**: Manages the user's "Staked Balance" and "Unclaimed Interest" locally.
    -   **Minting**: Requests "Minting Allowance" from Staking Hub to mint tokens locally (batching).
-   **Scaling**: Designed for horizontal scaling (sharding) to handle millions of users.

### 2. Learning Engine
- **Role**: Stateless Content Provider.
- **Responsibilities**:
    -   Stores Learning Units, Quizzes, and Correct Answers.
    -   **Verify Quiz**: Provides a pure query method `verify_quiz(answers)` used by User Profile shards.
    -   Does **NOT** store user data or progress.

### 3. Staking & Mining Hub
- **Role**: Central Bank & Global Stats.
- **Responsibilities**:
    -   Holds the 4.1B Utility tokens (Real Treasury).
    -   **Global Stats**: Tracks `total_staked`, `interest_pool`, `cumulative_reward_index`.
    -   **Allowance Manager**: Grants minting allowances to User Profile shards.
    -   **Settlement**: Handles real ledger transfers during unstaking.
    -   **Interest Distribution**: Calculates global reward index updates.

### 4. Operational Governance
- **Role**: Manages the Treasury (3.6B GHC).
- **Responsibilities**:
    -   Proposals for spending.
    -   Voting power calculation.
    -   Enforces 16M GHC/month spending limit.

### 5. Content Governance
- **Role**: Manages educational content.
- **Responsibilities**:
    -   Proposals to whitelist books/NFTs.

### 6. GHC Ledger (ICRC-1)
- **Standard**: ICRC-1.
- **Function**: Records all token balances and transfers.

## Key Workflows

### Mining (Virtual Staking)
1.  **Submit**: User submits quiz answers to their `User Profile` shard.
2.  **Verify**: `User Profile` calls `learning_engine.verify_quiz(answers)`.
3.  **Reward**: If passed, `User Profile` mints tokens locally (updates `staked_balance`).
4.  **Sync**: `User Profile` periodically reports stats to `Staking Hub` and requests more minting allowance.

### Unstaking
1.  **Request**: User calls `unstake(amount)` on `User Profile`.
2.  **Update**: `User Profile` reduces local `staked_balance`.
3.  **Process**: `User Profile` calls `staking_hub.process_unstake(user, amount)`.
4.  **Transfer**: `Staking Hub` sends real tokens (minus 10% penalty) from its Ledger account to the user's wallet.
5.  **Penalty**: The 10% penalty is added to the Global Interest Pool.

### Interest Distribution (Lazy)
1.  **Global**: `Staking Hub` updates `Global Reward Index` based on Interest Pool size.
2.  **Local**: When a user interacts with `User Profile`, it checks the Global Index.
3.  **Compound**: `User Profile` calculates pending interest (`Balance * IndexDiff`) and adds it to `unclaimed_interest`.

## Technology Stack
-   **Blockchain**: Internet Computer (ICP).
-   **Language**: Rust.
-   **Standards**: ICRC-1 (Token).
-   **Storage**: `ic-stable-structures` for scalable stable memory.
