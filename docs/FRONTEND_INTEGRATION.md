# GHC Dapp Frontend Integration Guide

Complete API reference for integrating React/TypeScript frontends with the GreenHero canister ecosystem.

**Last Updated**: December 2024

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Architecture Overview](#2-architecture-overview)
3. [Canister IDs & Setup](#3-canister-ids--setup)
4. [User Profile Canister](#4-user-profile-canister)
5. [Learning Engine Canister](#5-learning-engine-canister)
6. [Staking Hub Canister](#6-staking-hub-canister)
7. [Operational Governance Canister](#7-operational-governance-canister)
8. [GHC Ledger (ICRC-1)](#8-ghc-ledger-icrc-1)
9. [Founder Vesting Canister](#9-founder-vesting-canister)
10. [ICRC-1 Index Canister](#10-icrc-1-index-canister)
11. [Complete React Integration](#11-complete-react-integration)
12. [Error Handling](#12-error-handling)
13. [UI Pages Reference](#13-ui-pages-reference)

---

## 1. Prerequisites

### Required Dependencies

```bash
npm install @dfinity/agent @dfinity/candid @dfinity/principal @dfinity/auth-client
```

### Generate TypeScript Declarations

```bash
dfx generate
```

This creates declaration files in `src/declarations/` for each canister with TypeScript types.

---

## 2. Architecture Overview

### Token Distribution (9.5B GHC Total)

| Partition | Amount | Canister | Purpose |
|-----------|--------|----------|---------|
| **Utility Coins (MUC)** | 4.75B | `staking_hub` | Mining rewards via quizzes |
| **Treasury (MC)** | 4.25B | `operational_governance` | DAO-controlled spending |
| **Founders (MC)** | 0.5B | `founder_vesting` | Time-locked (10%/year) |

### Canister Interaction Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FRONTEND APPLICATION                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        ▼                           ▼                           ▼
┌───────────────┐         ┌─────────────────┐         ┌─────────────────┐
│ user_profile  │         │ learning_engine │         │   ghc_ledger    │
│               │         │                 │         │                 │
│ • Registration│         │ • Get content   │         │ • Wallet balance│
│ • Submit quiz │         │ • Quiz data     │         │ • Transfers     │
│ • Staked bal  │         │                 │         │                 │
│ • Unstake     │         │                 │         │                 │
└───────┬───────┘         └────────┬────────┘         └─────────────────┘
        │                          │
        └─────────┬────────────────┘
                  ▼
        ┌─────────────────┐          ┌─────────────────────────┐
        │  staking_hub    │◀────────▶│ operational_governance  │
        │                 │          │                         │
        │ • Global stats  │          │ • Treasury state        │
        │ • VUC voting    │          │ • Create proposals      │
        │ • Founder mgmt  │          │ • Vote on proposals     │
        │ • Voting power  │          │ • MMCR releases         │
        └─────────────────┘          └─────────────────────────┘
```

---

## 3. Canister IDs & Setup

> **Note**: IDs change on redeployment. Get current IDs with `dfx canister id <name>`.

```typescript
// canister-ids.ts
export const CANISTER_IDS = {
  user_profile: "vg3po-ix777-77774-qaafa-cai",
  learning_engine: "ufxgi-4p777-77774-qaadq-cai",
  staking_hub: "vpyes-67777-77774-qaaeq-cai",
  operational_governance: "vizcg-th777-77774-qaaea-cai",
  ghc_ledger: "ulvla-h7777-77774-qaacq-cai",
  icrc1_index_canister: "ucwa4-rx777-77774-qaada-cai",
  founder_vesting: "umunu-kh777-77774-qaaca-cai",
  internet_identity: "uxrrr-q7777-77774-qaaaq-cai",
};
```

---

## 4. User Profile Canister

**Canister**: `user_profile`  
**Purpose**: User registration, quiz submission, token earnings, and staking management.

### Types

```typescript
type UserProfile = {
  email: string;
  name: string;
  education: string;
  gender: string;
  staked_balance: bigint;    // Tokens earned (in e8s)
  transaction_count: bigint;
};

type UserDailyStats = {
  day_index: bigint;
  quizzes_taken: number;     // 0-5 per day
  tokens_earned: bigint;
};

type TransactionRecord = {
  timestamp: bigint;
  tx_type: { QuizReward: null } | { Unstake: null };
  amount: bigint;
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_profile` | `principal` | `Option<UserProfile>` | Get user's profile and balance |
| `get_user_daily_status` | `principal` | `UserDailyStats` | Daily quiz/earning limits |
| `is_quiz_completed` | `principal, unit_id: string` | `bool` | Check if quiz was completed |
| `get_user_transactions` | `principal` | `Vec<TransactionRecord>` | Transaction history |
| `get_user_count` | - | `nat64` | Total users in this shard |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `register_user` | `UserProfileUpdate` | `Result<(), String>` | Register new user |
| `update_profile` | `UserProfileUpdate` | `Result<(), String>` | Update profile info |
| `submit_quiz` | `unit_id: string, answers: Vec<u8>` | `Result<nat64, String>` | Submit quiz, earn tokens |
| `unstake` | `amount: nat64` | `Result<nat64, String>` | Withdraw tokens to wallet |

### Code Examples

```typescript
// Check if user is registered
const profile = await userProfileActor.get_profile(userPrincipal);
const isRegistered = profile.length > 0;

// Register new user
await userProfileActor.register_user({
  email: "user@example.com",
  name: "John Doe",
  education: "Bachelor's",
  gender: "Male"
});

// Get staked balance
const [userProfile] = await userProfileActor.get_profile(userPrincipal);
const stakedBalance = Number(userProfile.staked_balance) / 1e8;

// Submit quiz (answers as array of option indices)
const result = await userProfileActor.submit_quiz("unit_1", [0, 2, 1, 3]);
if ('Ok' in result) {
  console.log(`Earned ${Number(result.Ok) / 1e8} GHC`); // 1 GHC per quiz
}

// Check daily limits
const dailyStats = await userProfileActor.get_user_daily_status(userPrincipal);
const quizzesRemaining = 5 - dailyStats.quizzes_taken;

// Unstake tokens (100% returned, no penalty)
const unstakeResult = await userProfileActor.unstake(BigInt(100_000_000)); // 1 GHC
```

---

## 5. Learning Engine Canister

**Canister**: `learning_engine`  
**Purpose**: Educational content and quiz data.

### Types

```typescript
type LearningUnitMetadata = {
  unit_id: string;
  title: string;
  chapter: string;
  description: string;
};

type PublicLearningUnit = {
  unit_id: string;
  title: string;
  content: string;      // Educational content
  paraphrase: string;   // Summary
  quiz: Quiz[];
};

type Quiz = {
  question: string;
  options: string[];
  // Note: correct_answer is NOT exposed
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_learning_units_metadata` | - | `Vec<LearningUnitMetadata>` | List all units |
| `get_learning_unit` | `unit_id: string` | `Result<PublicLearningUnit, String>` | Get full unit + quiz |

### Code Examples

```typescript
// Get curriculum menu
const units = await learningEngineActor.get_learning_units_metadata();

// Get specific unit content
const result = await learningEngineActor.get_learning_unit("unit_1");
if ('Ok' in result) {
  const unit = result.Ok;
  // Display: unit.content, unit.quiz[].question, unit.quiz[].options
}
```

---

## 6. Staking Hub Canister

**Canister**: `staking_hub`  
**Purpose**: Global statistics, voting power oracle, and founder management.

### Types

```typescript
type GlobalStats = {
  total_staked: bigint;    // Total tokens staked
  total_unstaked: bigint;  // Total tokens unstaked
  total_allocated: bigint; // Total tokens mined
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_global_stats` | - | `GlobalStats` | Platform-wide statistics |
| `get_vuc` | - | `nat64` | VUC (founder voting power) |
| `get_total_voting_power` | - | `nat64` | VUC + total_staked |
| `get_tokenomics` | - | `(nat64, nat64, nat64, nat64)` | (max_supply, allocated, vuc, total_power) |
| `is_founder` | `principal` | `bool` | Check if principal is founder |
| `get_founders` | - | `Vec<Principal>` | List all founders |
| `get_founder_count` | - | `nat64` | Number of founders |
| `get_user_shard` | `principal` | `Option<Principal>` | Get user's shard canister |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `fetch_voting_power` | `principal` | `nat64` | Get voting power (async) |
| `add_founder` | `principal` | `Result<(), String>` | Add founder (admin only) |
| `remove_founder` | `principal` | `Result<(), String>` | Remove founder (admin only) |

### Code Examples

```typescript
// Get global stats for dashboard
const stats = await stakingHubActor.get_global_stats();
const totalStaked = Number(stats.total_staked) / 1e8;
const totalAllocated = Number(stats.total_allocated) / 1e8;
const maxSupply = 4_750_000_000; // 4.75B MUC
const miningProgress = (totalAllocated / maxSupply * 100).toFixed(2);

// Get tokenomics for governance
const [max, allocated, vuc, totalPower] = await stakingHubActor.get_tokenomics();

// Check if user is a founder
const isFounder = await stakingHubActor.is_founder(userPrincipal);

// Get voting power for governance
const votingPower = await stakingHubActor.fetch_voting_power(userPrincipal);

// List all founders (admin dashboard)
const founders = await stakingHubActor.get_founders();
```

---

## 7. Operational Governance Canister

**Canister**: `operational_governance`  
**Purpose**: Treasury management, DAO proposals, and voting.

### Governance Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Min voting power to propose | 150 tokens | Required to create proposals |
| Approval threshold | 15,000 tokens | YES votes needed for approval |
| Voting period | 14 days | Time to vote |
| Resubmission cooldown | 180 days | Wait time for rejected proposals |

### Types

```typescript
type ProposalStatus = 
  | { Active: null }    // Voting in progress
  | { Approved: null }  // Passed, pending execution
  | { Rejected: null }  // Failed
  | { Executed: null }; // Successfully executed

type TokenType = 
  | { GHC: null }
  | { USDC: null }
  | { ICP: null };

type ProposalCategory = 
  | { Marketing: null }
  | { Development: null }
  | { Partnership: null }
  | { Liquidity: null }
  | { CommunityGrant: null }
  | { Operations: null }
  | { Custom: string };

type Proposal = {
  id: bigint;
  proposer: Principal;
  created_at: bigint;
  voting_ends_at: bigint;
  title: string;
  description: string;
  recipient: Principal;
  amount: bigint;
  token_type: TokenType;
  category: ProposalCategory;
  external_link: [] | [string];
  votes_yes: bigint;
  votes_no: bigint;
  voter_count: bigint;
  status: ProposalStatus;
};

type VoteRecord = {
  voter: Principal;
  proposal_id: bigint;
  vote: boolean;        // true = YES, false = NO
  voting_power: bigint;
  timestamp: bigint;
};

type CreateProposalInput = {
  title: string;
  description: string;
  recipient: Principal;
  amount: bigint;
  token_type: TokenType;
  category: ProposalCategory;
  external_link: [] | [string];
};

type TreasuryState = {
  balance: bigint;
  allowance: bigint;
  total_transferred: bigint;
  mmcr_count: bigint;
  last_mmcr_timestamp: bigint;
  genesis_timestamp: bigint;
};

type MMCRStatus = {
  releases_completed: bigint;
  releases_remaining: bigint;
  last_release_timestamp: bigint;
  next_release_amount: bigint;
  seconds_until_next: bigint;
};
```

### Query Methods - Proposals

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_proposal` | `id: nat64` | `Option<Proposal>` | Get single proposal |
| `get_active_proposals` | - | `Vec<Proposal>` | All active proposals |
| `get_all_proposals` | - | `Vec<Proposal>` | All proposals ever |
| `get_proposal_votes` | `id: nat64` | `Vec<VoteRecord>` | Who voted on proposal |
| `has_voted` | `id: nat64, voter: Principal` | `bool` | Check if user voted |
| `get_governance_config` | - | `(nat64, nat64, nat64, nat64)` | (min_power, threshold, days, cooldown) |

### Query Methods - Treasury

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_treasury_state` | - | `TreasuryState` | Full treasury state |
| `get_spendable_balance` | - | `nat64` | Current allowance |
| `get_mmcr_status` | - | `MMCRStatus` | MMCR progress |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `create_proposal` | `CreateProposalInput` | `Result<nat64, String>` | Create proposal |
| `vote` | `id: nat64, approve: bool` | `Result<(), String>` | Vote on proposal |
| `finalize_proposal` | `id: nat64` | `Result<ProposalStatus, String>` | Force finalization |
| `execute_mmcr` | - | `Result<nat64, String>` | Trigger monthly release |

### Code Examples - Proposals

```typescript
// Get governance config
const [minPower, threshold, votingDays, cooldown] = 
  await govActor.get_governance_config();

// Check user's voting power before proposing
const votingPower = await stakingHubActor.fetch_voting_power(userPrincipal);
const canPropose = votingPower >= BigInt(minPower * 100_000_000n);

// Create a proposal
const result = await govActor.create_proposal({
  title: "Marketing Campaign Q1",
  description: "Fund marketing initiatives for Q1 2025",
  recipient: recipientPrincipal,
  amount: BigInt(10_000 * 100_000_000), // 10,000 GHC
  token_type: { GHC: null },
  category: { Marketing: null },
  external_link: ["https://forum.example.com/proposal/123"]
});

if ('Ok' in result) {
  console.log(`Proposal created with ID: ${result.Ok}`);
}

// Get all active proposals
const activeProposals = await govActor.get_active_proposals();

// Vote on a proposal
await govActor.vote(BigInt(0), true); // Vote YES on proposal #0

// Check if already voted
const hasVoted = await govActor.has_voted(BigInt(0), userPrincipal);

// See who voted on a proposal
const votes = await govActor.get_proposal_votes(BigInt(0));
votes.forEach(v => {
  console.log(`${v.voter}: ${v.vote ? 'YES' : 'NO'} (${Number(v.voting_power) / 1e8} tokens)`);
});
```

### Code Examples - Treasury

```typescript
// Get treasury overview
const treasury = await govActor.get_treasury_state();
console.log(`Balance: ${Number(treasury.balance) / 1e8} GHC`);
console.log(`Spendable: ${Number(treasury.allowance) / 1e8} GHC`);
console.log(`MMCR Progress: ${treasury.mmcr_count}/240`);

// Get MMCR status
const mmcr = await govActor.get_mmcr_status();
const daysUntilNext = Number(mmcr.seconds_until_next) / 86400;
console.log(`Next release: ${Number(mmcr.next_release_amount) / 1e8} GHC in ${daysUntilNext.toFixed(1)} days`);
```

---

## 8. GHC Ledger (ICRC-1)

**Canister**: `ghc_ledger`  
**Purpose**: Token ledger for wallet operations.

### Token Configuration

| Property | Value |
|----------|-------|
| Symbol | GHC |
| Name | GreenHero Coin |
| Decimals | 8 |
| Transfer Fee | 0 |
| Total Supply | 9.5B |

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `icrc1_balance_of` | `Account` | `nat` | Get balance |
| `icrc1_total_supply` | - | `nat` | Total supply |
| `icrc1_fee` | - | `nat` | Transfer fee (0) |
| `icrc1_decimals` | - | `nat8` | Decimals (8) |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `icrc1_transfer` | `TransferArg` | `Result<nat, TransferError>` | Transfer tokens |

### Code Examples

```typescript
// Get wallet balance
const balance = await ledgerActor.icrc1_balance_of({
  owner: userPrincipal,
  subaccount: []
});
const ghcBalance = Number(balance) / 1e8;

// Transfer tokens
const result = await ledgerActor.icrc1_transfer({
  to: { owner: recipientPrincipal, subaccount: [] },
  amount: BigInt(100_000_000), // 1 GHC
  fee: [],
  memo: [],
  from_subaccount: [],
  created_at_time: []
});
```

---

## 9. Founder Vesting Canister

**Canister**: `founder_vesting`  
**Purpose**: Time-locked founder token management.

### Vesting Schedule

| Founder | Allocation | Year 1 | Year 5 | Year 10 |
|---------|------------|--------|--------|---------|
| Founder 1 | 0.35B MC | 35M | 175M | 350M |
| Founder 2 | 0.15B MC | 15M | 75M | 150M |

- **Rate**: 10% unlocks per year
- **Duration**: 10 years to full vest

### Types

```typescript
type VestingStatus = {
  founder: Principal;
  total_allocation: bigint;
  vested: bigint;
  claimed: bigint;
  claimable: bigint;
  years_elapsed: bigint;
  unlock_percentage: bigint;
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_vesting_status` | `principal` | `Option<VestingStatus>` | Founder's vesting |
| `get_all_vesting_schedules` | - | `Vec<VestingStatus>` | All founders |
| `is_founder` | `principal` | `bool` | Check founder status |
| `get_genesis_timestamp` | - | `nat64` | Vesting start time |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `claim_vested` | - | `Result<nat64, String>` | Claim unlocked tokens |

### Code Examples

```typescript
// Check if founder
const isFounder = await vestingActor.is_founder(userPrincipal);

// Get vesting status
const status = await vestingActor.get_vesting_status(founderPrincipal);
if (status.length > 0) {
  const [v] = status;
  console.log(`Claimable: ${Number(v.claimable) / 1e8} GHC`);
  console.log(`Unlock: ${v.unlock_percentage}%`);
}

// Claim vested tokens
const result = await vestingActor.claim_vested();
if ('Ok' in result) {
  console.log(`Claimed ${Number(result.Ok) / 1e8} GHC`);
}
```

---

## 10. ICRC-1 Index Canister

**Canister**: `icrc1_index_canister`  
**Purpose**: Transaction history for wallet.

### Code Examples

```typescript
// Get transaction history
const result = await indexActor.get_account_transactions({
  account: { owner: userPrincipal, subaccount: [] },
  start: [],
  max_results: BigInt(20)
});

if ('Ok' in result) {
  result.Ok.transactions.forEach(tx => {
    const timestamp = new Date(Number(tx.transaction.timestamp) / 1e6);
    if (tx.transaction.transfer?.[0]) {
      const t = tx.transaction.transfer[0];
      console.log(`${timestamp}: ${Number(t.amount) / 1e8} GHC`);
    }
  });
}
```

---

## 11. Complete React Integration

```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { Principal } from "@dfinity/principal";

// Import generated IDLs
import { idlFactory as userProfileIdl } from "./declarations/user_profile";
import { idlFactory as learningEngineIdl } from "./declarations/learning_engine";
import { idlFactory as stakingHubIdl } from "./declarations/staking_hub";
import { idlFactory as govIdl } from "./declarations/operational_governance";
import { idlFactory as ledgerIdl } from "./declarations/ghc_ledger";
import { idlFactory as vestingIdl } from "./declarations/founder_vesting";

import { CANISTER_IDS } from "./canister-ids";

class GHCClient {
  private agent: HttpAgent;
  private identity: any;
  
  public userProfile: any;
  public learningEngine: any;
  public stakingHub: any;
  public governance: any;
  public ledger: any;
  public vesting: any;
  
  async initialize() {
    const authClient = await AuthClient.create();
    await new Promise((resolve) => {
      authClient.login({
        identityProvider: `http://${CANISTER_IDS.internet_identity}.localhost:4943/`,
        onSuccess: resolve,
      });
    });
    
    this.identity = authClient.getIdentity();
    this.agent = new HttpAgent({ identity: this.identity });
    await this.agent.fetchRootKey(); // Local only
    
    this.userProfile = Actor.createActor(userProfileIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.user_profile,
    });
    
    this.learningEngine = Actor.createActor(learningEngineIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.learning_engine,
    });
    
    this.stakingHub = Actor.createActor(stakingHubIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.staking_hub,
    });
    
    this.governance = Actor.createActor(govIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.operational_governance,
    });
    
    this.ledger = Actor.createActor(ledgerIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.ghc_ledger,
    });
    
    this.vesting = Actor.createActor(vestingIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.founder_vesting,
    });
  }
  
  getPrincipal(): Principal {
    return this.identity.getPrincipal();
  }
  
  // === BALANCES ===
  
  async getBalances() {
    const principal = this.getPrincipal();
    const [profile] = await this.userProfile.get_profile(principal);
    const walletBalance = await this.ledger.icrc1_balance_of({
      owner: principal,
      subaccount: [],
    });
    
    return {
      staked: Number(profile?.staked_balance || 0n) / 1e8,
      wallet: Number(walletBalance) / 1e8,
    };
  }
  
  // === GOVERNANCE ===
  
  async getVotingPower() {
    const power = await this.stakingHub.fetch_voting_power(this.getPrincipal());
    return Number(power) / 1e8;
  }
  
  async createProposal(input: any) {
    return this.governance.create_proposal(input);
  }
  
  async vote(proposalId: bigint, approve: boolean) {
    return this.governance.vote(proposalId, approve);
  }
}

export const ghcClient = new GHCClient();
```

---

## 12. Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `"User not registered"` | Calling before registration | Call `register_user` first |
| `"Quiz already completed"` | Resubmitting same quiz | Check `is_quiz_completed` |
| `"Daily quiz limit reached"` | 5 quizzes per day limit | Wait until next day |
| `"Insufficient voting power"` | < 150 tokens to propose | Earn more tokens |
| `"Already voted on this proposal"` | Double voting | Check `has_voted` first |
| `"Proposal is not active"` | Voting on concluded proposal | Check `status` |
| `"Voting period has ended"` | Late vote | Proposal already finalized |
| `"No voting power"` | 0 staked tokens | Stake tokens first |
| `"Insufficient treasury allowance"` | Amount > allowance | Wait for MMCR |

### Error Handling Pattern

```typescript
async function safeCall<T>(fn: () => Promise<T>): Promise<{ ok: T } | { err: string }> {
  try {
    const result = await fn();
    if (typeof result === 'object' && result !== null) {
      if ('Ok' in result) return { ok: (result as any).Ok };
      if ('Err' in result) return { err: (result as any).Err };
    }
    return { ok: result };
  } catch (e: any) {
    return { err: e.message || 'Unknown error' };
  }
}
```

---

## 13. UI Pages Reference

### User Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Dashboard** | user_profile, ghc_ledger | `get_profile`, `icrc1_balance_of` |
| **Learn** | learning_engine, user_profile | `get_learning_units_metadata`, `submit_quiz` |
| **Wallet** | ghc_ledger, icrc1_index | `icrc1_balance_of`, `get_account_transactions` |
| **Transfer** | ghc_ledger | `icrc1_transfer` |

### Governance Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Proposals** | operational_governance, staking_hub | `get_active_proposals`, `fetch_voting_power` |
| **Create Proposal** | operational_governance, staking_hub | `create_proposal`, `fetch_voting_power` |
| **Vote** | operational_governance | `vote`, `get_proposal_votes` |
| **Treasury** | operational_governance | `get_treasury_state`, `get_mmcr_status` |

### Admin/Founder Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Global Stats** | staking_hub | `get_global_stats`, `get_tokenomics` |
| **Founder Vesting** | founder_vesting | `get_vesting_status`, `claim_vested` |
| **Founder Management** | staking_hub | `add_founder`, `remove_founder` |

---

## Quick Reference

### Token Math

```typescript
const E8S = 100_000_000n; // 1 GHC = 10^8 e8s

const toGHC = (e8s: bigint) => Number(e8s) / 1e8;
const toE8s = (ghc: number) => BigInt(Math.floor(ghc * 1e8));

const formatGHC = (e8s: bigint) => toGHC(e8s).toLocaleString(undefined, {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2
});
```

### Constants

```typescript
const CONSTANTS = {
  TOTAL_SUPPLY: 9_500_000_000,       // 9.5B GHC
  DECIMALS: 8,
  
  // Treasury
  TREASURY_BALANCE: 4_250_000_000,   // 4.25B MC
  TREASURY_ALLOWANCE: 600_000_000,   // 0.6B MC initial
  MMCR_MONTHLY: 15_200_000,          // 15.2M MC
  MMCR_MONTHS: 240,
  
  // Staking
  MUC_SUPPLY: 4_750_000_000,         // 4.75B MUC
  
  // Governance
  MIN_VOTING_POWER: 150,             // Tokens to propose
  APPROVAL_THRESHOLD: 15_000,        // YES votes needed
  VOTING_DAYS: 14,
  COOLDOWN_DAYS: 180,
  
  // Founder Vesting
  FOUNDER_TOTAL: 500_000_000,        // 0.5B MC
  VEST_PERCENT_YEAR: 10,
  VEST_YEARS: 10,
};
```

### Polling Intervals

| Data | Interval | Reason |
|------|----------|--------|
| User Profile | On demand | User action only |
| Wallet Balance | 5s | After transfers |
| Proposals | 30s | Active voting |
| Treasury | 1 min | Slow-changing |
| MMCR Status | 1 hour | Daily check enough |
| Global Stats | 1 min | Not real-time critical |
