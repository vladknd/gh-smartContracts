# GHC System Architecture - ASCII Diagram

This document provides a comprehensive visual representation of how all canisters in the GreenHero Coin (GHC) system communicate with each other.

---

## Complete System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                           GHC SYSTEM ARCHITECTURE                                                │
│                                              (January 2026)                                                      │
│                                      *** REFACTORED: Separate Treasury & Governance ***                         │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

                                    ┌─────────────────────────────────────┐
                                    │          INTERNET IDENTITY          │
                                    │         (Authentication)            │
                                    └──────────────────┬──────────────────┘
                                                       │ authenticates
                                                       ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                              USER / FRONTEND                                                      │
└───────┬─────────────────────────┬─────────────────────────────┬─────────────────────────────────┬────────────────┘
        │                         │                             │                                 │
        │ proposals, votes        │ treasury state, MMCR        │ register, quiz, unstake         │ claim tokens
        ▼                         ▼                             ▼                                 ▼
┌────────────────────────┐ ┌────────────────────────┐  ┌──────────────────────────────┐  ┌───────────────────────────┐
│  GOVERNANCE_CANISTER   │ │   TREASURY_CANISTER    │  │        USER_PROFILE          │  │    FOUNDER_VESTING        │
│  (Proposals & Voting)  │ │   (Token Custody)      │  │   (Sharded User Accounts)    │  │   (Time-Locked Tokens)    │
├────────────────────────┤ ├────────────────────────┤  ├──────────────────────────────┤  ├───────────────────────────┤
│ [STORES]               │ │ [STORES]               │  │ [STORES]                     │  │ [STORES]                  │
│ • Board Member Shares  │ │ • Treasury Balance     │  │ • User profiles              │  │ • Founder 1: 0.35B MC     │
│ • Proposals + Votes    │ │   (4.25B MC)           │  │ • staked_balance (virtual)   │  │ • Founder 2: 0.15B MC     │
│ • Support Records      │ │ • Allowance (spendable)│  │ • Completed quizzes          │  │ • Vesting timestamps      │
│                        │ │ • MMCR state           │  │ • Transaction history        │  │                           │
│ [ACTIONS]              │ │                        │  │                              │  │ [ACTIONS]                 │
│ • create_treasury_     │ │ [ACTIONS]              │  │ [ACTIONS]                    │  │ • claim_vested()          │
│   proposal()           │ │ • execute_transfer()   │  │ • register_user()            │  │ • get_vesting_status()    │
│ • create_board_member_ │ │   (governance only!)   │  │ • submit_quiz()              │  │                           │
│   proposal()           │ │ • execute_mmcr()       │  │ • unstake()                  │  └─────────────┬─────────────┘
│ • vote()               │ │ • get_treasury_state() │  │                              │                │
│ • execute_proposal() ──┼─┼▶                       │  └───────┬───────────────────┬──┘                │
│                        │ └────────────┬───────────┘          │      │            │                   │
│ • set_board_member_    │              │                      │      │            │                   │claim_vested()
│   shares()             │              │                      │      │            │                   │
└───────────┬────────────┘              │                      │      │            │                   │
            │                           │ icrc1_transfer()     │      │            │                   │
            │ get_vuc()                 │                      │      │            │                   │
            │ fetch_user_voting_power() │                      │      │verify_quiz │transfer tokens()  │
            │                           │                      │      │            │                   │
            │     ┌─────────────────────┼──────────────────────┘      │            │                   │
            |     │                     │                             │            │                   │
            |     │ sync_shard()        │      ┌──────────────────────┘            │                   │
            |     │ process_unstake()   │      │                                   │                   │
            |     │                     │      │                                   │                   │
            ▼     ▼                     ▼      ▼                                   ▼                   ▼
┌───────────────────────┐    ┌────────────────────┐    ┌─────────────────────────────────────────────────────────┐
│     STAKING_HUB       │    │  LEARNING_ENGINE   │    │                       GHC_LEDGER                        │
│   (Central Bank)      │    │ (Stateless Content)│    │                      (ICRC-1/2)                         │
├───────────────────────┤    ├────────────────────┤    ├─────────────────────────────────────────────────────────┤
│ [STORES]              │    │ [STORES]           │    │ [TOKEN BALANCES]                                        │
│ • Global Stats        │    │ • Learning Units   │    │  ┌───────────────┐ ┌────────────────┐ ┌───────────────┐ │
│ • MAX_SUPPLY (4.75B)  │    │ • Quizzes + Answers│    │  │staking_hub    │ │treasury_canister│ │founder_vesting│ │
│ • Registered Shards   │    │                    │    │  │   4.75B MUC   │ │   4.25B MC     │ │   0.5B MC     │ │
│ • User→Shard mapping  │    │ [QUERIES]          │    │  └───────────────┘ └────────────────┘ └───────────────┘ │
│                       │    │ • get_learning_    │    │                                                         │
│ [PROVIDES]            │    │   unit(id)         │    │ [ACTIONS]                                               │
│ • get_vuc()           │    │ • verify_quiz()    │    │ • icrc1_transfer()                                      │
│ • fetch_user_voting_  │    │   → (pass, score)  │    │ • icrc2_approve() / icrc2_transfer_from()               │
│   power()             │    │                    │    │                                                         │
│ • get_tokenomics()    │    └────────────────────┘    │ [QUERIES]                                               │
│                       │                              │ • icrc1_balance_of() • icrc1_total_supply()             │
│ [ACTIONS]             │────────────────────────────▶│ (transfers for unstaking)                               │
│ • sync_shard()        │                              │                                                         │
│ • process_unstake()   │                              └─────────────────────────────────────────────────────────┘
└───────────────────────┘

                        ┌────────────────────────────────────────────────────────────────────────┐
                        │                      CONTENT GOVERNANCE (REFACTORED)                    │
                        ├────────────────────────────────────────────────────────────────────────┤
                        │                                                                         │
                        │  ┌─────────────────┐   ┌──────────────────┐   ┌──────────────────────┐│
                        │  │  MEDIA_ASSETS   │   │  STAGING_ASSETS  │   │  GOVERNANCE_CANISTER ││
                        │  │   (Permanent)   │   │   (Temporary)    │   │  (Content Proposals) ││
                        │  ├─────────────────┤   ├──────────────────┤   ├──────────────────────┤│
                        │  │ • Videos/Audio  │   │ • Content awaiting│──▶│ AddContentFromStaging││
                        │  │ • Images/PDFs   │   │   approval       │   │ UpdateContentNode    ││
                        │  │ • Immutable     │   │ • Metadata chunks │   │ UpdateGlobalQuizConfig│
                        │  │ • Deduplicated  │   │                  │   │ DeleteContentNode    ││
                        │  └─────────────────┘   └──────────────────┘   └──────────────────────┘│
                        │                                   │                      │            │
                        │                                   │   execute_proposal() │            │
                        │                                   ▼                      ▼            │
                        │                        ┌─────────────────────────────────────────┐    │
                        │                        │            LEARNING_ENGINE             │    │
                        │                        │         (ContentNode Tree)             │    │
                        │                        │  • Quiz Index • Version History       │    │
                        │                        └─────────────────────────────────────────┘    │
                        │                                                                         │
                        └────────────────────────────────────────────────────────────────────────┘

---

## Inter-Canister Call Flow

### 1. Proposal Creation & Voting

```
USER
  │
  ├──1. create_treasury_proposal() or create_board_member_proposal()
  │                  │
  │                  ▼
  │    ┌──────────────────────────────────────┐
  │    │       GOVERNANCE_CANISTER             │
  │    │                                       │
  │    │  2. Check: is_board_member_local()   │
  │    │  3. Query voting power ──────────────┼───────┐
  │    │                                       │       │
  │    │  4. (For treasury proposals)          │       │
  │    │     Check allowance ─────────────────┼───────┼───▶ TREASURY_CANISTER
  │    │                                       │       │       (can_transfer)
  │    └──────────────────────────────────────┘       │
  │                                                   │
  │                                                   ▼
  │                                      ┌────────────────────────┐
  │                                      │      STAKING_HUB       │
  │                                      │                        │
  │                                      │ If board member:       │
  │                                      │   5a. get_vuc()        │
  │                                      │ Else:                  │
  │                                      │   5b. fetch_user_      │
  │                                      │       voting_power()   │
  │                                      │                        │
  │                                      │ 6. Return voting power │
  │                                      └────────────────────────┘
  │
  ├──7. vote(proposal_id, approve) or support_proposal(proposal_id)
  │                  │
  │                  ▼
  │    ┌──────────────────────────────────────┐
  │    │       GOVERNANCE_CANISTER             │
  │    │                                       │
  │    │  8. Record vote/support              │
  │    │  9. Query voting power ──────────────┼───────▶ STAKING_HUB
  │    │ 10. Update proposal state            │
  │    └──────────────────────────────────────┘
```

### 2. Proposal Execution (NEW: Inter-canister flow)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         PROPOSAL EXECUTION FLOW                               │
│                    *** NEW: Separate Treasury & Governance ***               │
└──────────────────────────────────────────────────────────────────────────────┘

TREASURY PROPOSAL:
                    ┌─────────────────────────────────┐
execute_proposal()──▶│     GOVERNANCE_CANISTER         │
                    │                                 │
                    │ 1. Verify status == Approved    │
                    │ 2. Prepare transfer input       │
                    │ 3. Call treasury canister ──────┼──────▶ TREASURY_CANISTER
                    │                                 │         │
                    │                                 │         │ 4. Check caller == governance
                    │                                 │         │ 5. Check allowance
                    │                                 │         │ 6. Execute transfer ──▶ GHC_LEDGER
                    │                                 │         │ 7. Update treasury state
                    │                                 │◀────────┘ 8. Return success
                    │ 9. Set status = Executed        │
                    └─────────────────────────────────┘

BOARD MEMBER PROPOSAL:
                    ┌─────────────────────────────────┐
execute_proposal()──▶│     GOVERNANCE_CANISTER         │
                    │                                 │
                    │ 1. Verify status == Approved    │
                    │ 2. Calculate new shares         │
                    │    (proportional reduction)     │
                    │ 3. Update BOARD_MEMBER_SHARES   │
                    │ 4. Set status = Executed        │
                    └─────────────────────────────────┘
                    (No external canister calls needed!)
```

### 3. Mining (Quiz Rewards)

```
USER
  │
  ├──1. submit_quiz(answers)
  │           │
  │           ▼
  │    ┌────────────────────────────┐
  │    │       USER_PROFILE          │────2. verify_quiz()────▶ LEARNING_ENGINE
  │    │                            │◀───3. Pass/Fail ────────            
  │    │ 4. If Pass:                │
  │    │    Mint tokens locally     │
  │    │    (staked_balance +=)     │
  │    │                            │
  │    │ 5. Periodic sync ──────────┼──────────────────────────▶ STAKING_HUB
  │    │    (report stats,          │                           (update global stats,
  │    │     request allowance)     │◀──────────────────────────  grant more allowance)
  │    └────────────────────────────┘
```

### 4. Unstaking

```
USER
  │
  ├──1. unstake(amount)
  │           │
  │           ▼
  │    ┌────────────────────────────┐
  │    │       USER_PROFILE          │
  │    │                            │
  │    │ 2. Validate balance        │
  │    │ 3. Call staking_hub ───────┼───────▶ STAKING_HUB
  │    │                            │            │
  │    │                            │            │ 4. process_unstake()
  │    │                            │            │
  │    │                            │            ▼
  │    │                            │         GHC_LEDGER
  │    │                            │      (icrc1_transfer)
  │    │                            │            │
  │    │ 6. Deduct from balance ◀───┼───────5. Success
  │    └────────────────────────────┘
```

---

## Token Flow Summary

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                    TOKEN MOVEMENTS                                       │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                          │
│  STAKING_HUB (4.75B MUC) ─────────────────────────────────▶ USER WALLETS                │
│           │                   unstaking                                                  │
│           │                                                                              │
│           └───────────────────────────────────────────────▶ USER_PROFILE shards          │
│                           virtual minting                    (staked_balance)            │
│                                                                                          │
│  TREASURY_CANISTER (4.25B MC) ────────────────────────────▶ RECIPIENT WALLETS           │
│           ▲                      treasury spending proposals                             │
│           │                                                                              │
│  GOVERNANCE_CANISTER ─────────────────────────────────────▶ (calls execute_transfer)    │
│                              approved proposals trigger                                  │
│                                                                                          │
│  FOUNDER_VESTING (0.5B MC) ──────────────────────────────▶ FOUNDER WALLETS              │
│                              claim_vested() over 10 years                               │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Canister Dependencies (No Circular Dependencies!)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              CANISTER DEPENDENCY GRAPH                                   │
│                            (arrows show "depends on")                                    │
│                       *** UPDATED for new architecture ***                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                          │
│                           ┌─────────────────────────────┐                               │
│                           │        GHC_LEDGER           │                               │
│                           │    (no dependencies)        │                               │
│                           └────────────────▲────────────┘                               │
│                                            │                                            │
│         ┌──────────────────────────────────┼──────────────────────────────────┐         │
│         │                                  │                                  │         │
│ ┌───────┴───────┐            ┌─────────────┴──────────────┐         ┌─────────┴───────┐ │
│ │ STAKING_HUB   │            │   TREASURY_CANISTER        │         │ FOUNDER_VESTING │ │
│ │ depends on:   │            │   depends on:              │         │ depends on:     │ │
│ │  • ledger     │            │    • ledger                │         │  • ledger       │ │
│ └───────▲───────┘            └─────────────▲──────────────┘         └─────────────────┘ │
│         │                                  │                                            │
│         │                    ┌─────────────┴──────────────┐                             │
│         │                    │   GOVERNANCE_CANISTER      │                             │
│         │                    │   depends on:              │                             │
│         └────────────────────┤    • staking_hub           │                             │
│                              │    • treasury_canister     │                             │
│                              └─────────────────────────────┘                             │
│                                                                                          │
│ ┌───────────────┐               ┌─────────────────────┐                                │
│ │ USER_PROFILE  │               │   LEARNING_ENGINE   │                                │
│ │ depends on:   │───────────────▶│   (no dependencies) │                                │
│ │  • staking_hub│               └─────────────────────┘                                │
│ │  • learning   │                                                                       │
│ └───────────────┘                                                                       │
│                                                                                          │
│ KEY INSIGHT:                                                                            │
│  • governance_canister → treasury_canister (one-way only)                               │
│  • treasury_canister verifies caller == governance_canister (security!)                 │
│  • No circular dependencies in the entire system                                        │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Memory IDs (Stable Storage)

| Canister | Memory ID | Purpose |
|----------|-----------|---------|
| **treasury_canister** | 0 | LEDGER_ID |
| | 1 | GOVERNANCE_CANISTER_ID |
| | 2 | TREASURY_STATE |
| **governance_canister** | 0 | STAKING_HUB_ID |
| | 1 | TREASURY_CANISTER_ID |
| | 2 | PROPOSALS |
| | 3 | PROPOSAL_COUNT |
| | 4 | VOTE_RECORDS |
| | 5 | SUPPORT_RECORDS |
| | 6 | BOARD_MEMBER_SHARES |
| | 7 | BOARD_SHARES_LOCKED |
| **staking_hub** | 0-9 | Various storage |
| | 10, 11, 12 | Reserved (previously board member storage) |

> **Note**: The old `operational_governance` canister is deprecated. Its memory layout is preserved for reference but new deployments should use the separated canisters.

---

## Related Documents

- [ARCHITECTURE.md](./ARCHITECTURE.md) - High-level architecture overview
- [BOARD_MEMBER_VOTING_POWER.md](./BOARD_MEMBER_VOTING_POWER.md) - Board member voting power system
- [PROPOSAL_VOTING_FLOW.md](./PROPOSAL_VOTING_FLOW.md) - Proposal lifecycle details
- [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) - Frontend API integration guide (includes migration guide)
