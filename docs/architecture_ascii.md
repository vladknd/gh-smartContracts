```text
+-------------------------------------------------------+       +-------------------------------------------------------+
|                operational_governance                 |       |                  content_governance                   |
|           (Controls Treasury: 3.6B GHC)               |       |              (Manages Content Updates)                |
|                                                       |       |                                                       |
|  [STORES]                                             |       |  [STORES]                                             |
|   - Proposals (ID, Amount, Votes)                     |       |   - Book Count                                        |
|   - Total Spent Counter                               |       |   - (Future) Content Proposals                        |
|                                                       |       |                                                       |
|  [READS]                                              |       |  [READS]                                              |
|   - get_proposal(id)                                  |       |   - get_book_count()                                  |
|   - get_total_spent()                                 |       |                                                       |
+--------------------------+----------------------------+       +---------------------------+---------------------------+
                           |                                                                |
                           | (queries voting power)                                         | (queries voting power)
                           v                                                                v
            +---------------------------------------------------------------------------------------------+
            |                                         staking_hub                                         |
            |                                 (Holds 4.1B Utility Tokens)                                 |
            |                                                                                             |
            |  [STORES]                                                                                   |
            |   - User Virtual Balances (Staked + Pending Rewards)                                        |
            |   - Global Stats (Total Staked, Interest Pool, Total Unstaked)                              |
            |                                                                                             |
            |  [READS]                                                                                    |
            |   - get_user_stats(user) -> (Balance, Pending)                                              |
            |   - get_global_stats() -> (Staked, Pool, Unstaked)                                          |
            |   - get_voting_power(user)                                                                  |
            +-------------+---------------------------------------------------------------+---------------+
                          |                                                               ^
                          | (unstakes / transfers real tokens)                            | (stakes rewards / mints virtual tokens)
                          v                                                               |
+-------------------------------------------------------+       +-------------------------+-----------------------------+
|                      ghc_ledger                       |       |                      learning_engine                  |
|               (ICRC-1 Token Standard)                 |       |                 (User Education Platform)             |
|                                                       |       |                                                       |
|  [STORES]                                             |       |  [STORES]                                             |
|   - Real Token Balances (Founders, Treasury, Hub)     |       |   - Learning Units (Content, Quizzes)                 |
|   - Transaction History                               |       |   - User Progress (Completed Quizzes)                 |
|                                                       |       |   - Daily Stats (Attempts, Earned)                    |
|                                                       |       |                                                       |
|  [READS]                                              |       |  [READS]                                              |
|   - icrc1_balance_of(account)                         |       |   - get_learning_unit(id)                             |
|   - icrc1_total_supply()                              |       |   - get_user_daily_status(user)                       |
+-------------------------------------------------------+       +-------------------------------------------------------+
```
