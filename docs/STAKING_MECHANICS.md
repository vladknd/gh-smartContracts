# Staking & Interest Mechanics

## 1. Overview
The system uses a **Global Reward Index** model to distribute yield (from penalties) to stakers efficiently. This allows for scalable interest distribution without iterating through millions of user accounts.

**Key Architecture Change (Micro-Bank):**
- **User Balances** are stored in **Sharded User Profile Canisters**.
- **Global Stats** (Total Staked, Interest Pool) are stored in the **Staking Hub**.
- **Synchronization**: Shards lazily sync with the Hub to update the Global Index.

## 2. The Global Index Model
- **Concept**: The `GlobalIndex` represents the cumulative earnings per staked token since the beginning of time.
- **Source of Funds**: The Interest Pool is funded **ONLY** by exit penalties (10% of unstaked amount).
- **Distribution**: When `distribute_interest()` is called on the Hub:
  - `IndexIncrease = InterestPool / TotalStaked`
  - `GlobalIndex += IndexIncrease`
  - `InterestPool = 0`

## 3. User Mechanics (Manual Claim)

### A. Data Structure (Stored in `user_profile` Canister)
Each user profile tracks:
- `staked_balance`: The principal amount (Tokens earned from quizzes).
- `unclaimed_interest`: The pending rewards waiting to be claimed.
- `last_reward_index`: The value of the Global Index the last time we calculated their interest.

### B. Earning Interest (Passive)
Whenever a user interacts (Submit Quiz, Unstake, View Profile), we run a background calculation:
1. `IndexDiff = CurrentGlobalIndex - UserLastRewardIndex`
2. `NewInterest = UserStakedBalance * IndexDiff`
3. `unclaimed_interest += NewInterest`
4. `last_reward_index = CurrentGlobalIndex`

This ensures that interest is captured accurately over time, even if the user does nothing.

### C. Claiming Interest (Active)
Users must explicitly click "Claim Rewards" to move money to their balance.
1. User calls `claim_rewards()`.
2. System calculates any final pending interest (Step B).
3. System moves `unclaimed_interest` -> `staked_balance`.
4. `unclaimed_interest` resets to 0.

### D. Unstaking
- Users can only unstake from their `staked_balance`.
- Unclaimed interest remains safely in `unclaimed_interest` until claimed.
- Unstaking triggers a 10% penalty, which feeds the Interest Pool for everyone else.

## 4. Future Considerations (Variable Rates)
*Discussion from Dec 2025*

### Variable Penalties (Time-Based)
- **Goal**: Lower penalties for long-term holders (e.g., 0% after 5 years).
- **Implementation**: Requires replacing `staked_balance` (u64) with a `Vec<DepositBucket>` (FIFO).
- **Logic**: When unstaking, burn oldest tokens first.

### Variable Interest (Time-Based)
- **Goal**: Higher yield for long-term holders.
- **Challenge**: Hard to implement with a Global Index (which assumes pro-rata).
- **Recommendation**: Keep interest flat. The benefit of holding is the lower penalty (higher realized yield).

## 5. Scalability
- **Minting**: Batched via Shards (Infinite Scale).
- **Interest**: O(1) Distribution via Index. Lazy Evaluation on Shards.
- **Unstaking**: Direct Ledger Call (Limited by Ledger throughput, but low frequency).
