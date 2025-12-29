# GHC Dapp Frontend Integration Guide

This guide provides a comprehensive reference for frontend developers integrating with the GHC Dapp canisters on the Internet Computer.

**Last Updated**: December 2024

---

## 1. Prerequisites

### Required Dependencies

```bash
npm install @dfinity/agent @dfinity/candid @dfinity/principal @dfinity/auth-client
```

### Generate TypeScript Declarations

After deployment, generate TypeScript bindings for all canisters:

```bash
dfx generate
```

This creates declaration files in `src/declarations/` for each canister.

---

## 2. Canister Overview

### Token Distribution (9.5B GHC Total)

| Partition | Amount | Canister | Purpose |
|-----------|--------|----------|---------|
| **MUCs** | 4.75B | `staking_hub` | Mining rewards (earned via quizzes) |
| **Treasury** | 4.25B | `operational_governance` | DAO-controlled spending |
| **Founders** | 0.5B | `founder_vesting` | Time-locked (10%/year) |

### Canister Summary

| Canister | Purpose | User Interaction Level |
|----------|---------|------------------------|
| `user_profile` | User accounts, quizzes, balances | **High** - Primary entry point |
| `learning_engine` | Quiz content | **High** - Display learning material |
| `ghc_ledger` | Token ledger (ICRC-1/2) | **Medium** - Wallet operations |
| `operational_governance` | Treasury + Proposals | **Medium** - Admin/DAO dashboard |
| `founder_vesting` | Founder token claims | **Low** - Founder-only |
| `staking_hub` | Global stats | **Low** - Display only |
| `icrc1_index_canister` | Transaction history | **Medium** - Wallet history |

---

## 3. Deployed Canister IDs

> **Note**: IDs change on redeployment. Use `dfx canister id <name>` to get current IDs.

```javascript
// Example canister IDs (update after deployment)
const CANISTER_IDS = {
  user_profile: "ufxgi-4p777-77774-qaadq-cai",
  learning_engine: "umunu-kh777-77774-qaaca-cai",
  staking_hub: "ucwa4-rx777-77774-qaada-cai",
  operational_governance: "ulvla-h7777-77774-qaacq-cai",
  ghc_ledger: "u6s2n-gx777-77774-qaaba-cai",
  icrc1_index_canister: "vpyes-67777-77774-qaaeq-cai",
  founder_vesting: "<deploy to get id>",
  internet_identity: "uzt4z-lp777-77774-qaabq-cai",
};
```

---

## 4. Canister API Reference

### A. User Profile (`user_profile`)

**Purpose**: Primary user interface. Manages user state, quizzes, and staked balances.

#### Types

```typescript
type UserProfile = {
  email: string;
  name: string;
  education: string;
  gender: string;
  staked_balance: bigint;  // User's earned tokens (in e8s)
  staking_time: bigint;    // When staking started
  created_at: bigint;
};

type UserDailyStats = {
  quizzes_taken: bigint;
  tokens_earned: bigint;
  last_reset: bigint;
};

type TransactionRecord = {
  timestamp: bigint;
  tx_type: "QuizReward" | "Unstake" | "Interest";
  amount: bigint;
};
```

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `register_user(profile)` | Update | Register a new user |
| `update_profile(profile)` | Update | Update user info |
| `get_profile(principal)` | Query | Get user profile and balance |
| `submit_quiz(unit_id, answers)` | Update | Submit quiz, earn tokens |
| `unstake(amount)` | Update | Withdraw tokens to wallet |
| `get_user_daily_status(principal)` | Query | Get daily progress |
| `is_quiz_completed(principal, unit_id)` | Query | Check if quiz done |
| `get_user_transactions(principal)` | Query | Get transaction history |

#### Code Examples

```typescript
// Check if user is registered
const profile = await userProfileActor.get_profile(userPrincipal);
const isRegistered = profile.length > 0; // Option returns [] or [value]

// Register new user
await userProfileActor.register_user({
  email: "user@example.com",
  name: "John Doe",
  education: "Bachelor",
  gender: "Male"
});

// Submit quiz (answers as array of indices)
const result = await userProfileActor.submit_quiz("unit_1", [0, 2, 1, 3]);
if ('Ok' in result) {
  console.log(`Earned ${result.Ok} e8s`); // 100000000 = 1 GHC
}

// Check staked balance
const [userProfile] = await userProfileActor.get_profile(userPrincipal);
const stakedBalance = Number(userProfile.staked_balance) / 1e8; // Convert to GHC

// Unstake tokens (100% returned, no penalty)
const unstakeResult = await userProfileActor.unstake(BigInt(100_000_000)); // 1 GHC
```

---

### B. Learning Engine (`learning_engine`)

**Purpose**: Stateless content provider for educational material and quizzes.

#### Types

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
  content: string;
  paraphrase: string;
  quiz: Quiz[];
};

type Quiz = {
  question: string;
  options: string[];
  // Note: correct_answer is NOT exposed to frontend
};
```

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `get_learning_units_metadata()` | Query | List all available units |
| `get_learning_unit(unit_id)` | Query | Get full unit content + quiz |

#### Code Examples

```typescript
// Get curriculum menu
const units = await learningEngineActor.get_learning_units_metadata();
// Returns: [{ unit_id: "unit_1", title: "Intro to Climate", ... }]

// Get quiz content
const result = await learningEngineActor.get_learning_unit("unit_1");
if ('Ok' in result) {
  const unit = result.Ok;
  // Display unit.content, unit.quiz[].question, unit.quiz[].options
}
```

---

### C. GHC Ledger (`ghc_ledger`) - ICRC-1/ICRC-2

**Purpose**: Token ledger for wallet operations.

#### Configuration

| Property | Value |
|----------|-------|
| Token Symbol | GHC |
| Token Name | GreenHero Coin |
| Decimals | 8 |
| Transfer Fee | 0 |
| Total Supply | 9.5B |

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `icrc1_balance_of(account)` | Query | Get wallet balance |
| `icrc1_transfer(args)` | Update | Transfer tokens |
| `icrc1_total_supply()` | Query | Get total supply |
| `icrc1_fee()` | Query | Get transfer fee (0) |
| `icrc2_approve(args)` | Update | Approve spender |
| `icrc2_transfer_from(args)` | Update | Transfer on behalf |

#### Code Examples

```typescript
// Get wallet balance
const balance = await ledgerActor.icrc1_balance_of({
  owner: userPrincipal,
  subaccount: [] // null subaccount
});
const ghcBalance = Number(balance) / 1e8;

// Transfer tokens
const result = await ledgerActor.icrc1_transfer({
  to: { owner: recipientPrincipal, subaccount: [] },
  amount: BigInt(100_000_000), // 1 GHC
  fee: [], // optional, uses default
  memo: [], // optional
  from_subaccount: [], // optional
  created_at_time: [] // optional
});
```

---

### D. Operational Governance (`operational_governance`) - TREASURY

**Purpose**: Treasury management and DAO governance.

#### Treasury Constants

| Constant | Value | Description |
|----------|-------|-------------|
| Initial Balance | 4.25B MC | Total treasury tokens |
| Initial Allowance | 0.6B MC | Immediately spendable |
| MMCR Amount | 15.2M MC/month | Monthly release |
| MMCR Duration | 240 months | 20 years total |

#### Types

```typescript
type TreasuryState = {
  balance: bigint;           // Total MC held (decreases on transfers)
  allowance: bigint;         // Spendable amount (increases via MMCR)
  total_transferred: bigint; // Historical total spent
  mmcr_count: bigint;        // MMCR releases executed (0-240)
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

type Proposal = {
  id: bigint;
  proposer: Principal;
  recipient: Principal;
  amount: bigint;
  description: string;
  votes_yes: bigint;
  votes_no: bigint;
  executed: boolean;
  created_at: bigint;
};
```

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| **Treasury Queries** |||
| `get_treasury_state()` | Query | Full treasury state |
| `get_spendable_balance()` | Query | Current allowance |
| `get_treasury_balance()` | Query | Total balance |
| `get_mmcr_status()` | Query | MMCR progress |
| **Treasury Actions** |||
| `execute_mmcr()` | Update | Trigger monthly release |
| **Governance** |||
| `create_proposal(recipient, amount, desc)` | Update | Create spending proposal |
| `vote(proposal_id, approve)` | Update | Vote on proposal |
| `execute_proposal(proposal_id)` | Update | Execute approved proposal |
| `get_proposal(id)` | Query | Get proposal details |
| `get_total_spent()` | Query | Historical total spent |

#### Code Examples

```typescript
// === TREASURY DASHBOARD ===

// Get treasury overview
const treasuryState = await govActor.get_treasury_state();
console.log(`Balance: ${Number(treasuryState.balance) / 1e8} GHC`);
console.log(`Allowance: ${Number(treasuryState.allowance) / 1e8} GHC`);
console.log(`MMCR Count: ${treasuryState.mmcr_count}/240`);

// Get MMCR status
const mmcrStatus = await govActor.get_mmcr_status();
console.log(`Next release: ${Number(mmcrStatus.next_release_amount) / 1e8} GHC`);
console.log(`Time until next: ${mmcrStatus.seconds_until_next} seconds`);

// === GOVERNANCE ===

// Create a spending proposal
const proposalResult = await govActor.create_proposal(
  recipientPrincipal,
  BigInt(1_000_000_000_000), // 10,000 GHC
  "Fund community event"
);

// Vote on proposal
await govActor.vote(BigInt(1), true); // Vote yes on proposal #1

// Execute approved proposal
await govActor.execute_proposal(BigInt(1));
```

#### Treasury UI Components

```typescript
// Treasury Dashboard Component Data
interface TreasuryDashboard {
  totalBalance: string;      // "4,250,000,000 GHC"
  spendableAllowance: string; // "600,000,000 GHC" 
  lockedBalance: string;     // totalBalance - spendableAllowance
  monthlyRelease: string;    // "15,200,000 GHC"
  releasesCompleted: number; // 0-240
  releasesRemaining: number; // 240 - completed
  nextReleaseIn: string;     // "28 days" or "Available now"
}

// Calculate locked vs spendable
function getTreasuryBreakdown(state: TreasuryState) {
  const balance = Number(state.balance) / 1e8;
  const allowance = Number(state.allowance) / 1e8;
  return {
    total: balance,
    spendable: allowance,
    locked: balance - allowance,
    percentUnlocked: (allowance / balance * 100).toFixed(2)
  };
}
```

---

### E. Founder Vesting (`founder_vesting`)

**Purpose**: Time-locked founder token management.

#### Vesting Schedule

| Founder | Allocation | Year 1 | Year 5 | Year 10 |
|---------|------------|--------|--------|---------|
| Founder 1 | 0.35B MC | 35M | 175M | 350M |
| Founder 2 | 0.15B MC | 15M | 75M | 150M |

- **Vesting**: 10% unlocks per year
- **Duration**: 10 years to full vest

#### Types

```typescript
type VestingStatus = {
  founder: Principal;
  total_allocation: bigint;  // Total tokens allocated
  vested: bigint;            // Currently unlocked
  claimed: bigint;           // Already claimed
  claimable: bigint;         // Available to claim now
  years_elapsed: bigint;     // Years since genesis
  unlock_percentage: bigint; // 0-100
};
```

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `claim_vested()` | Update | Founder claims unlocked tokens |
| `get_vesting_status(principal)` | Query | Get founder's vesting status |
| `get_all_vesting_schedules()` | Query | Get all founders' status |
| `get_genesis_timestamp()` | Query | Vesting start time |
| `get_total_unclaimed()` | Query | Total unclaimed across founders |
| `is_founder(principal)` | Query | Check if principal is founder |

#### Code Examples

```typescript
// === FOUNDER DASHBOARD ===

// Check if current user is a founder
const isFounder = await vestingActor.is_founder(userPrincipal);

// Get founder's vesting status
const status = await vestingActor.get_vesting_status(founderPrincipal);
if (status.length > 0) {
  const [vesting] = status;
  console.log(`Total: ${Number(vesting.total_allocation) / 1e8} GHC`);
  console.log(`Vested: ${Number(vesting.vested) / 1e8} GHC`);
  console.log(`Claimed: ${Number(vesting.claimed) / 1e8} GHC`);
  console.log(`Claimable: ${Number(vesting.claimable) / 1e8} GHC`);
  console.log(`Unlock: ${vesting.unlock_percentage}%`);
}

// Claim vested tokens (founder only)
const claimResult = await vestingActor.claim_vested();
if ('Ok' in claimResult) {
  console.log(`Claimed ${Number(claimResult.Ok) / 1e8} GHC`);
}

// === ADMIN DASHBOARD ===

// Get all founders' vesting status
const allSchedules = await vestingActor.get_all_vesting_schedules();
allSchedules.forEach(schedule => {
  console.log(`${schedule.founder}: ${schedule.unlock_percentage}% unlocked`);
});
```

#### Vesting UI Component

```typescript
interface VestingProgress {
  totalAllocation: string;  // "350,000,000 GHC"
  vestedAmount: string;     // "35,000,000 GHC" (after 1 year)
  claimedAmount: string;    // "10,000,000 GHC"
  claimableNow: string;     // "25,000,000 GHC"
  yearsElapsed: number;     // 1
  unlockPercent: number;    // 10
  nextUnlockIn: string;     // "347 days"
}

// Calculate time until next unlock
function getNextUnlockCountdown(vestingStatus: VestingStatus, genesisTimestamp: bigint) {
  const yearsElapsed = Number(vestingStatus.years_elapsed);
  if (yearsElapsed >= 10) return "Fully vested";
  
  const genesis = Number(genesisTimestamp) / 1e9; // Convert ns to seconds
  const nextUnlockTime = genesis + ((yearsElapsed + 1) * 365 * 24 * 60 * 60);
  const now = Date.now() / 1000;
  const secondsRemaining = nextUnlockTime - now;
  
  return `${Math.ceil(secondsRemaining / 86400)} days`;
}
```

---

### F. Staking Hub (`staking_hub`)

**Purpose**: Global statistics and MUC token management.

#### Types

```typescript
type GlobalStats = {
  total_staked: bigint;     // Total MUCs staked across all users
  total_unstaked: bigint;   // Total MUCs unstaked
  total_allocated: bigint;  // Total MUCs allocated (vs MAX_SUPPLY)
};
```

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `get_global_stats()` | Query | Get platform-wide statistics |

#### Code Examples

```typescript
// Display global stats
const stats = await stakingHubActor.get_global_stats();

const totalStaked = Number(stats.total_staked) / 1e8;
const totalAllocated = Number(stats.total_allocated) / 1e8;
const maxSupply = 4_750_000_000; // 4.75B MUC
const remainingToMine = maxSupply - totalAllocated;

console.log(`Total Staked: ${totalStaked.toLocaleString()} GHC`);
console.log(`Mining Progress: ${(totalAllocated / maxSupply * 100).toFixed(2)}%`);
console.log(`Remaining to Mine: ${remainingToMine.toLocaleString()} GHC`);
```

---

### G. ICRC-1 Index (`icrc1_index_canister`)

**Purpose**: Transaction history for wallet operations.

#### Methods

| Method | Type | Description |
|--------|------|-------------|
| `get_account_transactions(args)` | Query | Get transaction history |
| `status()` | Query | Check indexer sync status |

#### Code Examples

```typescript
// Get wallet transaction history
const result = await indexActor.get_account_transactions({
  account: { owner: userPrincipal, subaccount: [] },
  start: [], // Most recent first
  max_results: BigInt(20)
});

if ('Ok' in result) {
  const { balance, transactions } = result.Ok;
  
  transactions.forEach(tx => {
    const { id, transaction } = tx;
    const timestamp = new Date(Number(transaction.timestamp) / 1e6);
    
    if (transaction.transfer?.[0]) {
      const t = transaction.transfer[0];
      console.log(`${timestamp}: Transfer ${Number(t.amount) / 1e8} GHC`);
    }
  });
}
```

---

## 5. Balance Display Guide

### User Has Two Balances

```typescript
interface UserBalances {
  // 1. Staked Balance (in user_profile canister)
  // - Earned via quizzes
  // - Not on the ledger yet
  // - Must unstake to access
  stakedBalance: bigint;
  
  // 2. Wallet Balance (in ghc_ledger)
  // - Real tokens on the ledger
  // - Can transfer to others
  // - Appears after unstaking
  walletBalance: bigint;
}

// Fetch both balances
async function getUserBalances(principal: Principal) {
  const [profile] = await userProfileActor.get_profile(principal);
  const walletBalance = await ledgerActor.icrc1_balance_of({
    owner: principal,
    subaccount: []
  });
  
  return {
    staked: Number(profile?.staked_balance || 0n) / 1e8,
    wallet: Number(walletBalance) / 1e8,
    total: (Number(profile?.staked_balance || 0n) + Number(walletBalance)) / 1e8
  };
}
```

### UI Layout Suggestion

```
┌────────────────────────────────────────────────────┐
│                  YOUR BALANCES                      │
├────────────────────────────────────────────────────┤
│                                                     │
│   💰 Staked Balance          🏦 Wallet Balance      │
│   ━━━━━━━━━━━━━━━━━          ━━━━━━━━━━━━━━━━━━    │
│   1,234.56 GHC               567.89 GHC             │
│   (Earned from quizzes)      (Transferrable)        │
│                                                     │
│   [Unstake →]                [Send] [Receive]       │
│                                                     │
├────────────────────────────────────────────────────┤
│   Total: 1,802.45 GHC                              │
└────────────────────────────────────────────────────┘
```

---

## 6. Complete Integration Example (React)

```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { Principal } from "@dfinity/principal";

// Import generated IDL factories
import { idlFactory as userProfileIdl } from "./declarations/user_profile";
import { idlFactory as learningEngineIdl } from "./declarations/learning_engine";
import { idlFactory as ledgerIdl } from "./declarations/ghc_ledger";
import { idlFactory as govIdl } from "./declarations/operational_governance";
import { idlFactory as vestingIdl } from "./declarations/founder_vesting";

const CANISTER_IDS = {
  user_profile: "ufxgi-4p777-77774-qaadq-cai",
  learning_engine: "umunu-kh777-77774-qaaca-cai",
  ghc_ledger: "u6s2n-gx777-77774-qaaba-cai",
  operational_governance: "ulvla-h7777-77774-qaacq-cai",
  founder_vesting: "<UPDATE_AFTER_DEPLOY>",
  internet_identity: "uzt4z-lp777-77774-qaabq-cai",
};

class GHCClient {
  private agent: HttpAgent;
  private identity: any;
  
  public userProfile: any;
  public learningEngine: any;
  public ledger: any;
  public governance: any;
  public vesting: any;
  
  async initialize() {
    // 1. Authenticate
    const authClient = await AuthClient.create();
    await new Promise((resolve) => {
      authClient.login({
        identityProvider: `http://${CANISTER_IDS.internet_identity}.localhost:4943/`,
        onSuccess: resolve,
      });
    });
    
    this.identity = authClient.getIdentity();
    this.agent = new HttpAgent({ identity: this.identity });
    
    // For local development only
    await this.agent.fetchRootKey();
    
    // 2. Create actors
    this.userProfile = Actor.createActor(userProfileIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.user_profile,
    });
    
    this.learningEngine = Actor.createActor(learningEngineIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.learning_engine,
    });
    
    this.ledger = Actor.createActor(ledgerIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.ghc_ledger,
    });
    
    this.governance = Actor.createActor(govIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.operational_governance,
    });
    
    this.vesting = Actor.createActor(vestingIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.founder_vesting,
    });
  }
  
  getPrincipal(): Principal {
    return this.identity.getPrincipal();
  }
  
  // === USER OPERATIONS ===
  
  async registerUser(profile: { email: string; name: string; education: string; gender: string }) {
    return this.userProfile.register_user(profile);
  }
  
  async submitQuiz(unitId: string, answers: number[]) {
    return this.userProfile.submit_quiz(unitId, answers);
  }
  
  async getBalances() {
    const principal = this.getPrincipal();
    const [profile] = await this.userProfile.get_profile(principal);
    const walletBalance = await this.ledger.icrc1_balance_of({
      owner: principal,
      subaccount: [],
    });
    
    return {
      staked: profile?.staked_balance || 0n,
      wallet: walletBalance,
    };
  }
  
  async unstake(amount: bigint) {
    return this.userProfile.unstake(amount);
  }
  
  // === TREASURY OPERATIONS ===
  
  async getTreasuryState() {
    return this.governance.get_treasury_state();
  }
  
  async getMMCRStatus() {
    return this.governance.get_mmcr_status();
  }
  
  // === FOUNDER OPERATIONS ===
  
  async isFounder() {
    return this.vesting.is_founder(this.getPrincipal());
  }
  
  async getVestingStatus() {
    return this.vesting.get_vesting_status(this.getPrincipal());
  }
  
  async claimVested() {
    return this.vesting.claim_vested();
  }
}

// Usage
const client = new GHCClient();
await client.initialize();

// Check balances
const balances = await client.getBalances();
console.log(`Staked: ${Number(balances.staked) / 1e8} GHC`);
console.log(`Wallet: ${Number(balances.wallet) / 1e8} GHC`);

// Submit quiz
const result = await client.submitQuiz("unit_1", [0, 2, 1, 3]);
if ('Ok' in result) {
  console.log(`Earned ${Number(result.Ok) / 1e8} GHC!`);
}
```

---

## 7. Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `"Caller is not a registered founder"` | Non-founder calling claim_vested | Check is_founder() first |
| `"Insufficient treasury allowance"` | Proposal exceeds allowance | Wait for MMCR or reduce amount |
| `"Too early for next MMCR"` | MMCR called before 28 days | Check get_mmcr_status().seconds_until_next |
| `"User not registered"` | Calling methods before register | Call register_user() first |
| `"Quiz already completed"` | Resubmitting same quiz | Check is_quiz_completed() |
| `"Daily limit reached"` | Exceeded daily quiz limit | Wait until next day reset |

### Error Handling Pattern

```typescript
async function safeCall<T>(fn: () => Promise<T>): Promise<{ ok: T } | { err: string }> {
  try {
    const result = await fn();
    if (typeof result === 'object' && result !== null) {
      if ('Ok' in result) return { ok: result.Ok };
      if ('Err' in result) return { err: result.Err };
    }
    return { ok: result };
  } catch (e: any) {
    return { err: e.message || 'Unknown error' };
  }
}

// Usage
const result = await safeCall(() => client.submitQuiz("unit_1", [0, 1, 2]));
if ('err' in result) {
  showToast(`Error: ${result.err}`);
} else {
  showToast(`Success! Earned ${result.ok} tokens`);
}
```

---

## 8. Recommended UI Pages

### For Regular Users

| Page | Canisters Used | Key Functions |
|------|----------------|---------------|
| **Dashboard** | user_profile, ghc_ledger | get_profile, icrc1_balance_of |
| **Learn** | learning_engine, user_profile | get_learning_units_metadata, submit_quiz |
| **Wallet** | ghc_ledger, icrc1_index | icrc1_balance_of, get_account_transactions |
| **Transfer** | ghc_ledger | icrc1_transfer |

### For Admin/DAO

| Page | Canisters Used | Key Functions |
|------|----------------|---------------|
| **Treasury** | operational_governance | get_treasury_state, get_mmcr_status |
| **Proposals** | operational_governance | get_proposal, create_proposal, vote |
| **Global Stats** | staking_hub | get_global_stats |

### For Founders Only

| Page | Canisters Used | Key Functions |
|------|----------------|---------------|
| **Vesting** | founder_vesting | get_vesting_status, claim_vested |

---

## 9. WebSocket/Polling Recommendations

| Data | Poll Interval | Reason |
|------|---------------|--------|
| User Profile | On demand | Changes only on user action |
| Wallet Balance | 5 seconds | After transfers |
| Treasury State | 1 minute | Slow-changing |
| MMCR Status | 1 hour | Daily checks sufficient |
| Global Stats | 1 minute | Real-time not critical |
| Proposals | 30 seconds | For active voting |

---

## 10. Testing Checklist

```bash
# After deployment, verify these queries work:

# User Profile
dfx canister call user_profile get_profile "(principal \"$PRINCIPAL\")"

# Learning Engine
dfx canister call learning_engine get_learning_units_metadata

# Treasury
dfx canister call operational_governance get_treasury_state
dfx canister call operational_governance get_mmcr_status

# Founder Vesting
dfx canister call founder_vesting get_all_vesting_schedules
dfx canister call founder_vesting is_founder "(principal \"$FOUNDER_PRINCIPAL\")"

# Ledger
dfx canister call ghc_ledger icrc1_total_supply
dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$CANISTER_ID\"; subaccount = null })"
```

---

## 11. Quick Reference Card

### Token Math

```typescript
const E8S = 100_000_000n; // 1 GHC = 10^8 smallest units

// Convert e8s to GHC (for display)
const toGHC = (e8s: bigint) => Number(e8s) / 1e8;

// Convert GHC to e8s (for API calls)
const toE8s = (ghc: number) => BigInt(Math.floor(ghc * 1e8));

// Format with commas
const formatGHC = (e8s: bigint) => toGHC(e8s).toLocaleString(undefined, {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2
});
```

### Key Constants

```typescript
const CONSTANTS = {
  // Token
  TOTAL_SUPPLY: 9_500_000_000, // 9.5B GHC
  DECIMALS: 8,
  
  // Treasury
  TREASURY_BALANCE: 4_250_000_000, // 4.25B MC
  TREASURY_INITIAL_ALLOWANCE: 600_000_000, // 0.6B MC
  MMCR_MONTHLY: 15_200_000, // 15.2M MC
  MMCR_TOTAL_MONTHS: 240,
  
  // Staking
  MUC_SUPPLY: 4_750_000_000, // 4.75B MUC
  
  // Founder Vesting
  FOUNDER_TOTAL: 500_000_000, // 0.5B MC
  FOUNDER_1: 350_000_000, // 0.35B MC
  FOUNDER_2: 150_000_000, // 0.15B MC
  VEST_PERCENT_PER_YEAR: 10,
  VEST_YEARS: 10,
};
```
