+-------------------------------------------------------+       +-------------------------------------------------------+
|                operational_governance                 |       |                  content_governance                   |
|           (Controls Treasury: 4.25B GHC)              |       |              (Manages Content Updates)                |
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
            |                                 (The Central Bank & Treasury)                               |
            |                                                                                             |
            |  [STORES]                                                                                   |
            |   - Global Stats (Total Staked, Total Unstaked, Total Allocated)                              |
            |   - Allowed Minters (List of trusted User Profile Shards)                                   |
            |   - MAX_SUPPLY (4.75B MUC Hard Cap)                                                          |
            |                                                                                             |
            |  [READS]                                                                                    |
            |   - get_global_stats()                                                                      |
            |                                                                                             |
            |  [ACTIONS]                                                                                  |
            |   - sync_shard(stats, request_allowance) -> Grants Allowance                                |
            |   - process_unstake(user, amount) -> Sends Real Tokens                                      |
            +-------------+---------------------------------------------------------------+---------------+
                          |                                                               ^
                          | (grants allowance / receives stats)                           | (reports stats / requests allowance)
                          v                                                               |
+-------------------------------------------------------+       +-------------------------+-----------------------------+
|                      ghc_ledger                       |       |               user_profile (SHARDED)                  |
|               (ICRC-1 Token Standard)                 |       |             (The Retail Bank & User State)            |
|                                                       |       |                                                       |
|  [STORES]                                             |       |  [STORES]                                             |
|   - Real Token Balances (Founders, Treasury, Hub)     |       |   - User Profile (Email, Name, etc.)                  |
|   - Transaction History                               |       |   - User Balance (Staked + Pending Rewards)           |
|                                                       |       |   - Minting Allowance (Local Batch)                   |
|  [READS]                                              |       |                                                       |
|   - icrc1_balance_of(account)                         |       |  [ACTIONS]                                            |
|   - icrc1_total_supply()                              |       |   - submit_quiz() -> Updates Local Balance            |
|                                                       |       |   - sync_with_hub() -> Batches Updates                |
+-------------------------------------------------------+       +-------------------------+-----------------------------+
                                                                                          |
                                                                                          | (verifies answers)
                                                                                          v
                                                                +-------------------------+-----------------------------+
                                                                |                  learning_engine                      |
                                                                |             (Stateless Content Oracle)                |
                                                                |                                                       |
                                                                |  [STORES]                                             |
                                                                |   - Learning Units (Content, Quizzes)                 |
                                                                |                                                       |
                                                                |  [READS]                                              |
                                                                |   - get_learning_unit(id)                             |
                                                                |   - verify_quiz(answers) -> (Pass/Fail)               |
                                                                +-------------------------------------------------------+
