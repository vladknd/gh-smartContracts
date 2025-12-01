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

### 1. GHC Ledger (ICRC-1)
- **Standard**: ICRC-1.
- **Function**: Records all token balances and transfers.
- **Configuration**:
    -   Transfer Fee: 0.0001 GHC.
    -   Archive Canister: Enabled.

### 2. Staking & Mining Hub
- **Role**: Central custodian and logic engine.
- **Responsibilities**:
    -   Holds the 4.1B Utility tokens.
    -   Manages **Virtual User Balances** (Internal Ledger).
    -   Tracks "Virtual Utility Coin" (VUC) for founders.
    -   Handles "Auto-Stake" (Mining) and "Unstake" operations.
    -   Manages **Interest Pool** and **Lazy Distribution**.
- **Data Structures (Stable Memory)**:
    -   `user_state`: Map<Principal, UserState { balance, last_reward_index }>
    -   `global_stats`: { total_staked, interest_pool, cumulative_reward_index }

### 3. Operational Governance
- **Role**: Manages the Treasury (3.6B GHC).
- **Responsibilities**:
    -   Proposals for spending.
    -   Voting power calculation (queries Staking Hub).
    -   Enforces 16M GHC/month spending limit.

### 4. Content Governance
- **Role**: Manages educational content.
- **Responsibilities**:
    -   Proposals to whitelist books/NFTs.
    -   Voting power calculation (queries Staking Hub).

### 5. Learning Engine
- **Role**: Manages quizzes and user progress.
- **Responsibilities**:
    -   Verifies quiz answers (Passing Threshold: 60%).
    -   Enforces Daily Limits (Max 5 quizzes/day).
    -   Tracks user progress and triggers rewards.
- **Architecture**: Scalable "Bucket" system.
    -   **Master Router**: Directs users to specific buckets.
    -   **Buckets**: Store quiz history for ~10k users each.

## Key Workflows

### Mining (Virtual Staking)
1.  **Quiz**: User completes quiz in `Learning Engine`.
2.  **Verify**: Engine verifies answers.
3.  **Auto-Stake**: Engine calls `staking_hub.stake_rewards(user, amount)`.
4.  **Update**: Hub updates user's virtual balance and applies any pending interest.

### Governance Voting
- **Voting Power**:
    -   **Founders**: Proportional to `Hub_Main_Balance` (VUC).
    -   **Community**: Proportional to `Hub_User_Virtual_Balance`.
-   **Process**: Governance canister queries `Staking Hub` for voting power.

### Unstaking & Interest
1.  **Request**: User requests to unstake $X$ GHC.
2.  **Penalty**: 10% deducted and added to `Interest Pool`.
3.  **Transfer**:
    -   90% -> User's Liquid Wallet (Market Partition) via Ledger Transfer.
    -   10% -> Remains in Hub (Interest Pool).
4.  **Distribution**: Periodically, `Interest Pool` is moved to `Global Reward Index`.
5.  **Payout**: Users automatically claim their share of the index growth upon their next interaction.

## Technology Stack
-   **Blockchain**: Internet Computer (ICP).
-   **Language**: Rust.
-   **Standards**: ICRC-1 (Token), ICRC-7 (NFTs - potentially for books).
-   **Storage**: `ic-stable-structures` for scalable stable memory.
