# Staking Mechanics (Simplified)

## 1. Overview

The system uses a **simple staking model** where users earn tokens by completing educational quizzes. There are **no interest payouts** and **no penalties** for unstaking.

**Key Architecture:**
- **User Balances** are stored in **Sharded User Profile Canisters**.
- **Global Stats** (Total Staked, Total Unstaked, Total Allocated) are stored in the **Staking Hub**.
- **Synchronization**: Shards periodically sync with the Hub to report balance changes and request minting allowance.

## 2. Token Flow

### A. Earning Tokens (Quiz Rewards)
1. User completes a quiz in the `learning_engine`.
2. If passed, `user_profile` adds tokens to their `staked_balance`.
3. The shard deducts from its local minting allowance.
4. Periodically, the shard reports earnings to `staking_hub`.

### B. Unstaking (100% - No Penalty)
1. User requests unstake from their `staked_balance`.
2. `user_profile` calls `staking_hub.process_unstake()`.
3. Hub transfers the **full amount** (100%) via the ICRC-1 ledger.
4. User receives real GHC tokens in their wallet.

## 3. Data Structure

### User Profile (Stored in `user_profile` Canister)
Each user profile tracks:
- `staked_balance`: Tokens earned from quizzes (virtual balance).
- `transaction_count`: Number of transactions for history lookup.

### Global Stats (Stored in `staking_hub`)
- `total_staked`: Sum of all staked balances across all shards.
- `total_unstaked`: Total tokens that have been withdrawn.
- `total_allocated`: Total tokens allocated for minting (against 4.75B cap).

## 4. Daily Limits
- **5 quizzes per day** per user.
- **1 token (100_000_000 e8s)** reward per passed quiz.

## 5. Scalability
- **Minting**: Batched via Shards (Infinite Scale via micro-bank pattern).
- **Sync**: Shards sync every 5 seconds to report changes and request allowance.
- **Unstaking**: Direct Ledger Call via Hub (no penalty deduction).

## 6. Security
- **Hard Cap**: The Hub enforces the 4.75B MUC token maximum supply.
- **Allowance System**: Shards can only mint up to their granted allowance.
- **Authorized Shards**: Only registered shards can call Hub functions.
