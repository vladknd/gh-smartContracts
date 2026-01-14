# GHC System Architecture - ASCII Diagram

This document provides a comprehensive visual representation of how all canisters in the GreenHero Coin (GHC) system communicate with each other.

---

## Complete System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                           GHC SYSTEM ARCHITECTURE                                                │
│                                              (January 2026)                                                      │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

                                    ┌─────────────────────────────────────┐
                                    │          INTERNET IDENTITY          │
                                    │         (Authentication)            │
                                    └──────────────────┬──────────────────┘
                                                       │ authenticates
                                                       ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                              USER / FRONTEND                                                      │
└───────┬───────────────────────────────────────────┬─────────────────────────────────────────────────┬────────────┘
        │                                           │                                                 │
        │ proposals, votes                          │ register, quiz, unstake                         │ claim tokens
        ▼                                           ▼                                                 ▼
┌───────────────────────────┐    ┌──────────────────────────────────────┐    ┌───────────────────────────────────┐
│  OPERATIONAL_GOVERNANCE   │    │           USER_PROFILE               │    │        FOUNDER_VESTING            │
│   (Treasury + Board)      │    │      (Sharded User Accounts)         │    │     (Time-Locked Tokens)          │
├───────────────────────────┤    ├──────────────────────────────────────┤    ├───────────────────────────────────┤
│ [STORES]                  │    │ [STORES]                             │    │ [STORES]                          │
│ • Board Member Shares     │    │ • User profiles (email, name)        │    │ • Founder 1: 0.35B MC             │
│ • Proposals + Votes       │    │ • staked_balance (virtual)           │    │ • Founder 2: 0.15B MC             │
│ • Treasury State (4.25B)  │    │ • Completed quizzes                  │    │ • Vesting timestamps              │
│                           │    │ • Transaction history                │    │                                   │
│ [ACTIONS]                 │    │                                      │    │ [ACTIONS]                         │
│ • create_treasury_        │    │ [ACTIONS]                            │    │ • claim_vested()                  │
│   proposal()              │    │ • register_user()                    │    │ • get_vesting_status()            │
│ • create_board_member_    │    │ • submit_quiz()                      │    │                                   │
│   proposal()              │    │ • unstake()                          │    └─────────────┬─────────────────────┘
│ • vote()                  │    │                                      │                  │
│ • set_board_member_       │    └───────┬────────────────┬─────────────┘                  │
│   shares()                │            │                │                                │
└───────────┬───────────────┘            │                │                                │
            │                            │                │                                │
            │ get_vuc()                  │ verify_quiz()  │                                │ claim_vested()
            │ fetch_user_voting_power()  │                │                                │ transfer tokens
            │     -----------------------│----------------│ sync_shard()                   │
            |     | process_unstake()    |                |                                |
            ▼     ▼                      ▼                ▼                                ▼
┌───────────────────────┐    ┌────────────────────┐    ┌─────────────────────────────────────────────────────────┐
│     STAKING_HUB       │    │  LEARNING_ENGINE   │    │                       GHC_LEDGER                        │
│   (Central Bank)      │    │ (Stateless Content)│    │                      (ICRC-1/2)                         │
├───────────────────────┤    ├────────────────────┤    ├─────────────────────────────────────────────────────────┤
│ [STORES]              │    │ [STORES]           │    │ [TOKEN BALANCES]                                        │
│ • Global Stats        │    │ • Learning Units   │    │  ┌───────────────┐ ┌────────────────┐ ┌───────────────┐ │
│ • MAX_SUPPLY (4.75B)  │    │ • Quizzes + Answers│    │  │staking_hub    │ │operational_gov │ │founder_vesting│ │
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

                        ┌────────────────────────────────┐
                        │      CONTENT_GOVERNANCE        │
                        │    (Content Management)        │
                        ├────────────────────────────────┤
                        │ [STORES]                       │
                        │ • Book proposals               │
                        │ • Approved content             │
                        │                                │
                        │ [FUTURE]                       │
                        │ • Content voting               │
                        │ • Moderation                   │
                        └────────────────────────────────┘
```

                        │        (Content Management)             │
                        ├─────────────────────────────────────────┤
                        │ [STORES]                                │
                        │  • Book proposals                       │
                        │  • Book count                           │
                        │                                         │
                        │ [FUTURE]                                │
                        │  • Content voting                       │
                        │  • Content moderation                   │
                        └─────────────────────────────────────────┘
```

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
  │    │       OPERATIONAL_GOVERNANCE          │
  │    │                                       │
  │    │  2. Check: is_board_member_local()   │
  │    │  3. Query voting power ──────────────┼───────┐
  │    └──────────────────────────────────────┘       │
  │                                                   │
  │                                                   ▼
  │                                      ┌────────────────────────┐
  │                                      │      STAKING_HUB       │
  │                                      │                        │
  │                                      │ If board member:       │
  │                                      │   4a. get_vuc()        │
  │                                      │ Else:                  │
  │                                      │   4b. fetch_user_      │
  │                                      │       voting_power()   │
  │                                      │                        │
  │                                      │ 5. Return voting power │
  │                                      └────────────────────────┘
  │
  ├──6. vote(proposal_id, approve) or support_proposal(proposal_id)
  │                  │
  │                  ▼
  │    ┌──────────────────────────────────────┐
  │    │       OPERATIONAL_GOVERNANCE          │
  │    │                                       │
  │    │  7. Record vote/support              │
  │    │  8. Query voting power ──────────────┼───────▶ STAKING_HUB
  │    │  9. Update proposal state            │
  │    └──────────────────────────────────────┘
```

### 2. Proposal Execution

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         PROPOSAL EXECUTION FLOW                               │
└──────────────────────────────────────────────────────────────────────────────┘

TREASURY PROPOSAL:
                    ┌─────────────────────────────────┐
execute_proposal()──▶│   OPERATIONAL_GOVERNANCE        │
                    │                                 │
                    │ 1. Verify status == Approved    │
                    │ 2. Check treasury allowance     │
                    │ 3. Execute transfer ────────────┼──────▶ GHC_LEDGER
                    │ 4. Update treasury state        │◀────── (transfer success)
                    │ 5. Set status = Executed        │
                    └─────────────────────────────────┘

BOARD MEMBER PROPOSAL:
                    ┌─────────────────────────────────┐
execute_proposal()──▶│   OPERATIONAL_GOVERNANCE        │
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
│  OPERATIONAL_GOVERNANCE (4.25B MC) ──────────────────────▶ RECIPIENT WALLETS            │
│                              treasury spending proposals                                 │
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
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                          │
│                           ┌─────────────────────────────┐                               │
│                           │        GHC_LEDGER           │                               │
│                           │    (no dependencies)        │                               │
│                           └────────────────▲────────────┘                               │
│                                            │                                            │
│         ┌──────────────────────────────────┼──────────────────────────────────┐         │
│         │                                  │                                  │         │
│ ┌───────┴───────┐               ┌──────────┴──────────┐            ┌─────────┴───────┐ │
│ │ STAKING_HUB   │               │  OPERATIONAL_GOV    │            │ FOUNDER_VESTING │ │
│ │ depends on:   │               │  depends on:        │            │ depends on:     │ │
│ │  • ledger     │               │   • ledger          │            │  • ledger       │ │
│ └───────▲───────┘               │   • staking_hub     │◀───────────┴─────────────────┘ │
│         │                       └─────────────────────┘                                 │
│         │                                                                               │
│ ┌───────┴───────┐               ┌─────────────────────┐                                │
│ │ USER_PROFILE  │               │   LEARNING_ENGINE   │                                │
│ │ depends on:   │───────────────▶│   (no dependencies) │                                │
│ │  • staking_hub│               └─────────────────────┘                                │
│ │  • learning   │                                                                       │
│ └───────────────┘                                                                       │
│                                                                                          │
│ KEY: operational_governance → staking_hub (one-way only, no circular dependency!)       │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Memory IDs (Stable Storage)

| Canister | Memory ID | Purpose |
|----------|-----------|---------|
| **operational_governance** | 0 | LEDGER_ID |
| | 1 | STAKING_HUB_ID |
| | 2 | PROPOSALS |
| | 3 | PROPOSAL_COUNT |
| | 4 | VOTE_RECORDS |
| | 5 | SUPPORT_RECORDS |
| | 6 | TREASURY_STATE |
| | 7 | BOARD_MEMBER_SHARES |
| | 8 | BOARD_SHARES_LOCKED |
| **staking_hub** | 0-9 | Various storage |
| | 10, 11, 12 | Reserved (previously board member storage) |

---

## Related Documents

- [ARCHITECTURE.md](./ARCHITECTURE.md) - High-level architecture overview
- [BOARD_MEMBER_VOTING_POWER.md](./BOARD_MEMBER_VOTING_POWER.md) - Board member voting power system
- [PROPOSAL_VOTING_FLOW.md](./PROPOSAL_VOTING_FLOW.md) - Proposal lifecycle details
- [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) - Frontend API integration guide
