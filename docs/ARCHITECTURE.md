# GreenHero Coin (GHC) Dapp Architecture

## Overview

The GreenHero Coin (GHC) dapp is a blockchain-based educational platform built on the Internet Computer (ICP). It incentivizes learning through a "Pre-Mint & Allocate" tokenomics model, where users earn GHC tokens by completing educational activities (quizzes).

---

## Tokenomics (Updated December 2024)

### Total Supply: 9.5 Billion GHC

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         TOTAL SUPPLY: 9.5B GHC                               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────┐  ┌────────────────────────────────┐ │
│  │     UTILITY COINS (MUCs)            │  │     MARKET COINS (MCs)         │ │
│  │           4.75B                     │  │           4.75B                │ │
│  │                                     │  │                                │ │
│  │  Purpose: Staking/Mining Rewards    │  │  Distribution:                 │ │
│  │  Location: Staking Hub              │  │  ├─ Founders: 0.5B             │ │
│  │  (Earned via quizzes)               │  │  │  ├─ Founder 1: 0.35B        │ │
│  │                                     │  │  │  └─ Founder 2: 0.15B        │ │
│  │                                     │  │  │  (Time-locked: 10%/year)    │ │
│  │                                     │  │  │                             │ │
│  │                                     │  │  └─ Treasury: 4.25B            │ │
│  │                                     │  │     (Initial Allowance: 0.6B)  │ │
│  └─────────────────────────────────────┘  └────────────────────────────────┘ │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Token Configuration
- **Decimals**: 8 (1 GHC = 100,000,000 smallest units)
- **Transfer Fee**: 0 (no fees)
- **Standard**: ICRC-1 / ICRC-2 compatible

### Partitions

| Partition | Amount | Purpose | Location |
|-----------|--------|---------|----------|
| **MUCs (Utility Coins)** | 4.75B | Mining rewards via quizzes | `staking_hub` |
| **MCs (Market Coins)** | 4.75B | Market circulation | Split below |

### Market Coins Distribution

| Recipient | Amount | Notes | Location |
|-----------|--------|-------|----------|
| **Founder 1** | 0.35B | 10%/year vesting (10 years) | `founder_vesting` |
| **Founder 2** | 0.15B | 10%/year vesting (10 years) | `founder_vesting` |
| **Treasury** | 4.25B | MMCR: 15.2M/month release | `operational_governance` |

---

## System Architecture

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│                           GHC SYSTEM ARCHITECTURE                                     │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────┐    │
│   │                              GHC LEDGER (ICRC-1/2)                           │    │
│   │                              Total: 9.5B GHC                                 │    │
│   └────────────────────────────────────┬────────────────────────────────────────┘    │
│                                        │                                             │
│         ┌──────────────────────────────┼──────────────────────────────┐              │
│         ▼                              ▼                              ▼              │
│   ┌──────────────┐          ┌───────────────────┐          ┌─────────────────┐       │
│   │ STAKING_HUB  │          │ OPERATIONAL_GOV   │          │ FOUNDER_VESTING │       │
│   │   4.75B MUC  │          │   (Treasury)      │          │    0.5B MC      │       │
│   │              │          │   4.25B MC        │          │  ├─ F1: 0.35B   │       │
│   │  Central     │          │                   │          │  └─ F2: 0.15B   │       │
│   │  Bank        │          │  Balance: 4.25B   │          │                 │       │
│   │              │          │  Allowance: 0.6B  │          │  10%/year vest  │       │
│   └──────┬───────┘          │  MMCR: 15.2M/mo   │          └────────┬────────┘       │
│          │                  └─────────┬─────────┘                   │               │
│          │                            │                             │               │
│          ▼                            ▼                             ▼               │
│   ┌──────────────┐          ┌───────────────────┐          ┌─────────────────┐       │
│   │ USER_PROFILE │          │   GOVERNANCE      │          │ FOUNDER WALLETS │       │
│   │   (Shards)   │◄────────▶│   PROPOSALS       │          │ (After Claim)   │       │
│   └──────┬───────┘          └───────────────────┘          └─────────────────┘       │
│          │                                                                           │
│          ▼                                                                           │
│   ┌──────────────┐                                                                   │
│   │ LEARNING_ENG │                                                                   │
│   │ (Stateless)  │                                                                   │
│   └──────────────┘                                                                   │
│                                                                                       │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

---

## System Components

### 1. GHC Ledger (ICRC-1/ICRC-2)

- **Type**: Standard DFINITY Ledger Canister
- **Standards**: ICRC-1 (transfers), ICRC-2 (approve/transfer_from)
- **Configuration**:
  - Decimals: 8
  - Transfer Fee: 0
  - Total Supply: 9.5B GHC

### 2. Staking Hub (`staking_hub`)

- **Role**: Central Bank & MUC Token Controller
- **Token Balance**: 4.75B MUC
- **Responsibilities**:
  - **Global Stats**: Tracks `total_staked`, `total_unstaked`, `total_allocated`
  - **Allowance Manager**: Grants minting allowances to User Profile shards
  - **Settlement**: Handles real ledger transfers during unstaking
  - **Hard Cap Enforcement**: Never allocates beyond MAX_SUPPLY (4.75B)

### 3. Operational Governance (`operational_governance`)

- **Role**: Treasury Management & Spending Governance
- **Token Balance**: 4.25B MC
- **Treasury Features**:
  - **Balance**: 4.25B MC (decreases only on transfers)
  - **Allowance**: 0.6B MC initial (increases via MMCR)
  - **MMCR**: Monthly Market Coin Release (15.2M/month for 240 months)
- **Governance Features**:
  - Spending proposals
  - Voting (requires staking power)
  - Proposal execution (checks allowance)

#### MMCR (Monthly Market Coin Release)

| Parameter | Value |
|-----------|-------|
| **Monthly Amount** | 15.2M MC |
| **Duration** | 240 months (20 years) |
| **Initial Allowance** | 0.6B MC |
| **Final Month** | 17.2M MC (adjusted) |
| **Total Released** | 4.25B MC |

### 4. Founder Vesting (`founder_vesting`)

- **Role**: Time-Locked Founder Token Management
- **Token Balance**: 0.5B MC
- **Allocations**:
  - Founder 1: 0.35B MC
  - Founder 2: 0.15B MC
- **Vesting Schedule**: 10% unlocks per year, full vesting in 10 years
- **Key Functions**:
  - `claim_vested()`: Founders claim unlocked tokens
  - `get_vesting_status(principal)`: Query vesting progress

### 5. User Profile (`user_profile`) - Sharded

- **Role**: User's Personal Record & Micro-Bank
- **Responsibilities**:
  - Primary entry point for user interactions
  - Stores user profile and quiz progress
  - **Micro-Bank**: Manages `staked_balance` locally
  - **Minting**: Requests allowance from Staking Hub
- **Scaling**: Horizontal sharding for millions of users

### 6. Learning Engine (`learning_engine`)

- **Role**: Stateless Content Provider
- **Responsibilities**:
  - Stores learning units and quizzes
  - Provides `verify_quiz(answers)` for validation
  - Does NOT store user data

### 7. Content Governance (`content_governance`)

- **Role**: Educational Content Management
- **Responsibilities**:
  - Proposals to whitelist books/content
  - Content moderation

---

## Key Workflows

### Mining (Virtual Staking)

```
┌─────────┐    ┌──────────────┐    ┌───────────────┐    ┌─────────────┐
│  User   │───▶│ User Profile │───▶│Learning Engine│    │ Staking Hub │
│         │    │   (Shard)    │    │  (Stateless)  │    │  (Central)  │
└─────────┘    └──────┬───────┘    └───────────────┘    └──────┬──────┘
                      │                                        │
 1. Submit Quiz ──────┤                                        │
                      │ 2. Verify ────────────────────────────▶│
                      │◀────────────────────────── 3. Result ──│
 4. Mint Tokens ◀─────┤                                        │
 (Local Balance)      │                                        │
                      │ 5. Sync (Periodic) ───────────────────▶│
                      │◀────────────── 6. More Allowance ──────│
```

### Unstaking

```
┌─────────┐    ┌──────────────┐    ┌─────────────┐    ┌────────────┐
│  User   │───▶│ User Profile │───▶│ Staking Hub │───▶│ GHC Ledger │
└─────────┘    └──────────────┘    └─────────────┘    └─────┬──────┘
                                                            │
 1. Request Unstake ─────────────────────────────────────────│
                     2. Validate ────────────────────────────│
                                   3. Transfer Tokens ───────│
 4. Receive Tokens ◀─────────────────────────────────────────┘
```

### Treasury Spending

```
┌──────────┐    ┌───────────────────┐    ┌────────────┐
│ Proposer │───▶│ Operational Gov   │───▶│ GHC Ledger │
└──────────┘    │   (Treasury)      │    └─────┬──────┘
                └─────────┬─────────┘          │
                          │                    │
 1. Create Proposal ──────┤                    │
 2. Voting Period ────────┤                    │
 3. Check Allowance ──────┤                    │
 4. Execute Transfer ─────┼───────────────────▶│
 5. Update Treasury ◀─────┤                    │
   (Balance -= amount)    │                    │
   (Allowance -= amount)  │                    │
```

### MMCR Execution

```
 Anyone can trigger (time-gated)
              │
              ▼
┌───────────────────────────┐
│   execute_mmcr()          │
│                           │
│ 1. Check: mmcr_count < 240│
│ 2. Check: 28+ days passed │
│ 3. Increase allowance     │
│    by 15.2M (or 17.2M)    │
│ 4. Update mmcr_count      │
│                           │
│ Result: More spendable    │
│         tokens available  │
└───────────────────────────┘
```

### Founder Claim

```
┌──────────┐    ┌─────────────────┐    ┌────────────┐
│ Founder  │───▶│ Founder Vesting │───▶│ GHC Ledger │
└──────────┘    └────────┬────────┘    └─────┬──────┘
                         │                   │
 1. claim_vested() ──────┤                   │
 2. Calculate Claimable ─┤                   │
 3. Transfer Tokens ─────┼──────────────────▶│
 4. Receive in Wallet ◀──┴───────────────────┘
```

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| **Blockchain** | Internet Computer (ICP) |
| **Language** | Rust |
| **Token Standard** | ICRC-1 / ICRC-2 |
| **Storage** | `ic-stable-structures` |
| **Authentication** | Internet Identity |

---

## Security Considerations

### Treasury Security

- **Allowance Limit**: Can only spend up to current allowance
- **Time-Gated MMCR**: Minimum 28 days between releases
- **Governance Required**: All spending requires proposal approval

### Founder Vesting Security

- **Time-Locked**: Tokens unlock 10% per year
- **Caller Check**: Only registered founders can claim
- **Immutable Schedule**: Vesting start is fixed at genesis

### Staking Security

- **Hard Cap**: Never allocates beyond MAX_SUPPLY (4.75B)
- **Allowance Control**: Shards can only mint from granted allowance
- **Balance Tracking**: Global stats track all allocations

---

## Canister Summary

| Canister | Purpose | Token Balance |
|----------|---------|---------------|
| `ghc_ledger` | ICRC-1/2 Token Ledger | N/A (tracks all) |
| `staking_hub` | MUC Central Bank | 4.75B MUC |
| `operational_governance` | Treasury + Governance | 4.25B MC |
| `founder_vesting` | Time-Locked Founder Tokens | 0.5B MC |
| `user_profile` | User Accounts (Sharded) | Virtual balances |
| `learning_engine` | Quiz Content | None |
| `content_governance` | Content Management | None |

---

## Related Documentation

- [Treasury Implementation Plan](./TREASURY_IMPLEMENTATION_PLAN.md)
- [Token Decimals Decision](./TOKEN_DECIMALS_DECISION.md)
- [Staking Mechanics](./STAKING_MECHANICS.md)
- [Governance Plan](./GOVERNANCE_PLAN.md)
