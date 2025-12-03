```mermaid
graph TD
    subgraph "Operational Governance"
        OP_GOV[<b>operational_governance</b><br/><i>Controls Treasury (3.6B GHC)</i>]
        OP_STORE[<b>Stores:</b><br/>- Proposals (ID, Amount, Votes)<br/>- Total Spent (Cumulative)<br/>- Vote History]
        OP_READ[<b>Reads:</b><br/>- get_proposal(id)<br/>- get_total_spent()]
        OP_GOV --- OP_STORE
        OP_GOV --- OP_READ
    end

    subgraph "Content Governance"
        CONT_GOV[<b>content_governance</b><br/><i>Manages Content Updates</i>]
        CONT_STORE[<b>Stores:</b><br/>- Book Count (Placeholder)<br/>- Content Proposals (Future)]
        CONT_READ[<b>Reads:</b><br/>- get_book_count()]
        CONT_GOV --- CONT_STORE
        CONT_GOV --- CONT_READ
    end

    subgraph "Staking Hub"
        STAKE[<b>staking_hub</b><br/><i>Holds 4.1B Utility Tokens</i>]
        STAKE_STORE[<b>Stores:</b><br/>- User Virtual Balances<br/>- Global Stats (Staked, Pool, Unstaked)<br/>- Reward Indices]
        STAKE_READ[<b>Reads:</b><br/>- get_user_stats(user)<br/>- get_global_stats()<br/>- get_voting_power(user)]
        STAKE --- STAKE_STORE
        STAKE --- STAKE_READ
    end

    subgraph "GHC Ledger"
        LEDGER[<b>ghc_ledger</b><br/><i>ICRC-1 Token Standard</i>]
        LEDGER_STORE[<b>Stores:</b><br/>- Real Token Balances (Founders, Treasury, Hub)<br/>- Transaction History]
        LEDGER_READ[<b>Reads:</b><br/>- icrc1_balance_of(account)<br/>- icrc1_total_supply()]
        LEDGER --- LEDGER_STORE
        LEDGER --- LEDGER_READ
    end

    subgraph "Learning Engine"
        LEARN[<b>learning_engine</b><br/><i>User Education Platform</i>]
        LEARN_STORE[<b>Stores:</b><br/>- Learning Units (Content, Quizzes)<br/>- User Progress (Completed Quizzes)<br/>- Daily Stats (Attempts, Earned)]
        LEARN_READ[<b>Reads:</b><br/>- get_learning_unit(id)<br/>- get_user_daily_status(user)<br/>- is_quiz_completed(user, unit)]
        LEARN --- LEARN_STORE
        LEARN --- LEARN_READ
    end

    %% Relationships
    OP_GOV -- "Queries Voting Power" --> STAKE
    CONT_GOV -- "Queries Voting Power" --> STAKE
    LEARN -- "Stakes Rewards (Virtual)" --> STAKE
    STAKE -- "Unstakes (Real Transfer)" --> LEDGER
    OP_GOV -- "Executes Proposals (Real Transfer)" --> LEDGER

    style OP_GOV fill:#f9f,stroke:#333,stroke-width:2px
    style CONT_GOV fill:#bbf,stroke:#333,stroke-width:2px
    style STAKE fill:#bfb,stroke:#333,stroke-width:2px
    style LEDGER fill:#fbb,stroke:#333,stroke-width:2px
    style LEARN fill:#ff9,stroke:#333,stroke-width:2px
```
