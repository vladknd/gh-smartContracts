# GreenHero Coin Architecture Diagram

**Last Updated**: January 2026  
**Status**: Reflects current refactored architecture

```mermaid
graph TD
    subgraph "Treasury & Governance"
        TREASURY[<b>treasury_canister</b><br/><i>Controls Treasury: 4.25B MC</i>]
        TREASURY_STORE[<b>Stores:</b><br/>- Balance (4.25B MC)<br/>- Allowance (0.6B initial)<br/>- MMCR Progress]
        TREASURY_READ[<b>Reads:</b><br/>- get_treasury_state<br/>- get_spendable_balance]
        TREASURY --- TREASURY_STORE
        TREASURY --- TREASURY_READ

        GOV[<b>governance_canister</b><br/><i>Manages Proposals & Voting</i>]
        GOV_STORE[<b>Stores:</b><br/>- Proposals & Votes<br/>- Board Members<br/>- Content Proposals]
        GOV_READ[<b>Reads:</b><br/>- get_all_proposals<br/>- get_board_members]
        GOV --- GOV_STORE
        GOV --- GOV_READ
    end

    subgraph "Content Asset Canisters"
        MEDIA[<b>media_assets</b><br/><i>Permanent Media Storage</i>]
        MEDIA_STORE[<b>Stores:</b><br/>- Videos/Audio/Images/PDFs<br/>- Immutable & Deduplicated]
        MEDIA --- MEDIA_STORE

        STAGING[<b>staging_assets</b><br/><i>Temporary Content Storage</i>]
        STAGING_STORE[<b>Stores:</b><br/>- Content awaiting approval<br/>- Staging metadata]
        STAGING --- STAGING_STORE
    end

    subgraph "Staking Hub"
        STAKE[<b>staking_hub</b><br/><i>The Central Bank: 4.75B MUC</i>]
        STAKE_STORE[<b>Stores:</b><br/>- Global Stats (Total Staked, Mined, Allocated)<br/>- Allowed Minters (User Profile Shards)<br/>- MAX_SUPPLY: 4.75B MUC Hard Cap]
        STAKE_READ[<b>Reads:</b><br/>- get_global_stats<br/>- get_vuc (Voting Power)]
        STAKE_ACTIONS[<b>Actions:</b><br/>- sync_shard (stats)<br/>- process_unstake → Sends Real Tokens]
        STAKE --- STAKE_STORE
        STAKE --- STAKE_READ
        STAKE --- STAKE_ACTIONS
    end

    subgraph "GHC Ledger"
        LEDGER[<b>ghc_ledger</b><br/><i>ICRC-1 Token Standard</i>]
        LEDGER_STORE[<b>Stores:</b><br/>- Real Token Balances<br/>- Transaction History]
        LEDGER_READ[<b>Reads:</b><br/>- icrc1_balance_of(account)<br/>- icrc1_total_supply]
        LEDGER --- LEDGER_STORE
        LEDGER --- LEDGER_READ
    end

    subgraph "User Profile (SHARDED)"
        USER[<b>user_profile</b><br/><i>The Retail Bank & User State</i>]
        USER_STORE[<b>Stores:</b><br/>- User Profiles (Email, Name, etc.)<br/>- User Balance (Staked + Pending Rewards)<br/>- Minting Allowance (Local Batch)]
        USER_ACTIONS[<b>Actions:</b><br/>- submit_quiz → Updates Local Balance<br/>- sync_with_hub → Batches Updates]
        USER --- USER_STORE
        USER --- USER_ACTIONS
    end

    subgraph "Learning Engine"
        LEARN[<b>learning_engine</b><br/><i>Content & Quiz Oracle</i>]
        LEARN_STORE[<b>Stores:</b><br/>- ContentNodes (Tree Structure)<br/>- Quiz Index (O(1) Lookup)<br/>- Version History]
        LEARN_READ[<b>Reads:</b><br/>- get_content_node(id)<br/>- get_quiz_data(id)<br/>- get_global_quiz_config]
        LEARN --- LEARN_STORE
        LEARN --- LEARN_READ
    end

    subgraph "Founder Vesting"
        FOUNDER[<b>founder_vesting</b><br/><i>Time-Locked Tokens: 0.5B MC</i>]
        FOUNDER_STORE[<b>Stores:</b><br/>- Vesting Schedules<br/>- Claimed Amounts]
        FOUNDER --- FOUNDER_STORE
    end

    %% Relationships
    GOV -- "request_transfer" --> TREASURY
    GOV -- "queries voting power" --> STAKE
    GOV -- "load_content_from_staging" --> LEARN
    STAGING -- "provides content chunks" --> LEARN
    TREASURY -- "icrc1_transfer" --> LEDGER
    STAKE -- "icrc1_transfer" --> LEDGER
    USER -- "reports stats / requests allowance" --> STAKE
    USER -- "fetches quiz data" --> LEARN
    FOUNDER -- "icrc1_transfer" --> LEDGER

    style TREASURY fill:#f9f,stroke:#333,stroke-width:2px
    style GOV fill:#f9f,stroke:#333,stroke-width:2px
    style MEDIA fill:#bbf,stroke:#333,stroke-width:2px
    style STAGING fill:#bbf,stroke:#333,stroke-width:2px
    style STAKE fill:#bfb,stroke:#333,stroke-width:2px
    style LEDGER fill:#fbb,stroke:#333,stroke-width:2px
    style USER fill:#fcf,stroke:#333,stroke-width:2px
    style LEARN fill:#ff9,stroke:#333,stroke-width:2px
    style FOUNDER fill:#ffa,stroke:#333,stroke-width:2px
```

## Canister Summary

| Canister | Purpose | Token Holdings |
|----------|---------|----------------|
| `treasury_canister` | Token custody, MMCR, transfer execution | 4.25B MC |
| `governance_canister` | Proposals, voting, board management, content governance | - |
| `staking_hub` | Central bank, VUC provider, global stats | 4.75B MUC |
| `user_profile` | User accounts (sharded), quiz submissions, staked balances | Per-user allocations |
| `learning_engine` | Content storage, quiz management, version history | - |
| `media_assets` | Permanent media storage (videos, images, PDFs) | - |
| `staging_assets` | Temporary content before governance approval | - |
| `ghc_ledger` | ICRC-1 token ledger, transaction history | Total supply tracking |
| `founder_vesting` | Time-locked founder tokens (10% annual unlock) | 0.5B MC |

## Key Flows

1. **Treasury Spending**: `governance_canister.execute_proposal()` → `treasury_canister.request_transfer()` → `ghc_ledger.icrc1_transfer()`
2. **Voting Power**: `governance_canister.get_user_voting_power()` → `staking_hub.get_vuc()` → aggregates from `user_profile` shards
3. **Content Loading**: `governance_canister.execute_proposal(AddContentFromStaging)` → `learning_engine.start_content_load()` → pulls from `staging_assets`
4. **Staking Rewards**: `user_profile.submit_quiz()` → internal balance update → periodic `sync_with_hub()` → `staking_hub.sync_shard()`
