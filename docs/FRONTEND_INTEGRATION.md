# GHC Dapp Frontend Integration Guide

Complete API reference for integrating React/TypeScript frontends with the GreenHero canister ecosystem.

**Last Updated**: January 2026

> **New in January 2026**: Treasury and Governance have been refactored into separate canisters for improved security and maintainability. The `treasury_canister` handles token custody and transfers, while the `governance_canister` manages proposals, voting, and board members. Additionally:
> - **Configurable governance timings** (support_period, voting_period, resubmission_cooldown) via governance proposals
> - **Quiz limits** (daily, weekly, monthly, yearly) for rate limiting
> - **ICO canister** for fixed-price token sales with ckUSDC payments (MoonPay integration-ready)
> - **Sonic adapter** for DEX liquidity and swaps
> - **Archive canister** for transaction history archival
> - **Verification tiers** (None, Human, KYC) for user profiles

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Architecture Overview](#2-architecture-overview)
3. [Canister IDs & Setup](#3-canister-ids--setup)
4. [User Profile Canister](#4-user-profile-canister)
5. [Learning Engine Canister](#5-learning-engine-canister)
6. [Staking Hub Canister](#6-staking-hub-canister)
7. [Treasury Canister](#7-treasury-canister)
8. [Governance Canister](#8-governance-canister)
9. [Content Governance (Staging & Media)](#9-content-governance-canisters)
10. [KYC Canister](#10-kyc-canister) *(NEW)*
11. [Subscription Canister](#11-subscription-canister) *(NEW)*
12. [GHC Ledger (ICRC-1)](#12-ghc-ledger-icrc-1)
13. [Founder Vesting Canister](#13-founder-vesting-canister)
14. [ICRC-1 Index Canister](#14-icrc-1-index-canister)
15. [ICO Canister](#15-ico-canister)
16. [Sonic Adapter Canister](#16-sonic-adapter-canister)
17. [Archive Canister](#17-archive-canister)
18. [Complete React Integration](#18-complete-react-integration)
19. [Error Handling](#19-error-handling)
20. [UI Pages Reference](#20-ui-pages-reference)
21. [Migration Guide](#21-migration-from-operational_governance)

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
| **Treasury (MC)** | 4.25B | `treasury_canister` | Token custody & transfers |
| **Founders (MC)** | 0.5B | `founder_vesting` | Time-locked (10%/year) |

### Canister Interaction Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FRONTEND APPLICATION                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
    ┌────────────┬────────────┬─────┼─────┬────────────┬────────────┐
    ▼            ▼            ▼     ▼     ▼            ▼            ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│ user_   │ │learning │ │  ghc_   │ │treasury │ │governan │ │  ico_   │
│ profile │ │ engine  │ │ ledger  │ │canister │ │canister │ │canister │
├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤
│Register │ │Content  │ │Balance  │ │State    │ │Propose  │ │buy_ghc  │
│Quiz     │ │Quiz data│ │Transfer │ │MMCR     │ │Vote     │ │Stats    │
│StakedBal│ │         │ │         │ │         │ │Execute  │ │         │
│Unstake  │ │         │ │         │ │         │ │         │ │         │
└────┬────┘ └────┬────┘ └─────────┘ └────▲────┘ └────┬────┘ └─────────┘
     │           │                       │          │
     └─────┬─────┘                       └────┬─────┘
           ▼                                  │
     ┌─────────────┐                          │
     │ staking_hub │──────────────────────────┘
     │             │
     │• Global stat│  ┌──────────────────────────────────────────┐
     │• get_vuc()  │  │ Additional Canisters:                    │
     │• VotingPower│  │ • archive_canister: Transaction history  │
     └──────┬──────┘  │ • sonic_adapter: DEX integration         │
            │         │ • staging_assets: Content staging        │
            ▼         │ • media_assets: Approved media storage   │
     ┌─────────────┐  └──────────────────────────────────────────┘
     │   archive   │
     │   canister  │
     └─────────────┘
```

### Canister Responsibilities (14 Canisters Total)

| Canister | Responsibilities |
|----------|------------------|
| **user_profile** | User registration, quiz submission, token earnings, and staking management. |
| **staking_hub** | Global stats, VUC provider, user voting power oracle, shard management, and token limits. |
| **learning_engine** | Content storage, quiz data, hierarchical curriculum, and content loading. |
| **treasury_canister** | Token custody, balance tracking, MMCR releases, transfer execution. |
| **governance_canister** | Proposals, voting, board member management, configurable timings. |
| **kyc_canister** | Identity verification management and tier updates. |
| **subscription_canister** | Paid subscription management and Stripe/Checkout integration. |
| **ghc_ledger** | ICRC-1 token ledger for GHC. |
| **founder_vesting** | Time-locked founder token vesting. |
| **icrc1_index_canister** | Transaction history indexing. |
| **media_assets** | Permanent storage for approved media files. |
| **staging_assets** | Temporary storage for content awaiting governance approval. |
| **archive_canister** | Long-term transaction history archival from user_profile shards. |
| **ico_canister** | Fixed-price token sales with ckUSDC payments. |
| **sonic_adapter** | DEX integration for adding liquidity and swaps. |
| **internet_identity** | User authentication. |

---

## 3. Canister IDs & Setup

> **Note**: IDs change on redeployment. Get current IDs with `dfx canister id <name>`.

```typescript
// canister-ids.ts
export const CANISTER_IDS = {
  // Core User Experience
  user_profile: "vg3po-ix777-77774-qaafa-cai",
  learning_engine: "ufxgi-4p777-77774-qaadq-cai",
  staking_hub: "vpyes-67777-77774-qaaeq-cai",
  
  // Governance & Treasury
  treasury_canister: "xxxxx-xxxxx-xxxxx-xxxxx-cai",
  governance_canister: "yyyyy-yyyyy-yyyyy-yyyyy-cai",
  
  // Token Infrastructure
  ghc_ledger: "ulvla-h7777-77774-qaacq-cai",
  icrc1_index_canister: "ucwa4-rx777-77774-qaada-cai",
  founder_vesting: "umunu-kh777-77774-qaaca-cai",
  
  // Content Governance
  media_assets: "zzzzz-zzzzz-zzzzz-zzzzz-cai",
  staging_assets: "aaaaa-aaaaa-aaaaa-aaaaa-cai",
  
  // ICO & DEX
  ico_canister: "bbbbb-bbbbb-bbbbb-bbbbb-cai",
  sonic_adapter: "ccccc-ccccc-ccccc-ccccc-cai",
  
  // Archive & History
  archive_canister: "ddddd-ddddd-ddddd-ddddd-cai",
  
  // KYC & Subscriptions
  kyc_canister: "eeeee-eeeee-eeeee-eeeee-cai",
  subscription_canister: "fffff-fffff-fffff-fffff-cai",
  
  // Authentication
  internet_identity: "uxrrr-q7777-77774-qaaaq-cai",
};
```

---

## 4. User Profile Canister

**Canister**: `user_profile`  
**Purpose**: User registration, quiz submission, token earnings, and staking management.

### Types

```typescript
// Verification tiers for user profiles
type VerificationTier = 
  | { None: null }    // Fresh user - no verification
  | { Human: null }   // DecideID verified (Not a bot)
  | { KYC: null };    // Full legal KYC (Passport/AML)

type UserProfile = {
  email: string;
  name: string;
  education: string;
  gender: string;
  verification_tier: VerificationTier;  // User verification level
  staked_balance: bigint;               // Tokens earned (in e8s)
  transaction_count: bigint;            // Local transactions
  archived_transaction_count: bigint;   // Transactions moved to archive
};

type UserTimeStats = {
  last_active_day: bigint;
  // Daily limits
  daily_quizzes: number;     // Current day quiz count
  daily_earnings: bigint;    // Current day earnings
  // Weekly limits
  weekly_quizzes: number;    // Current week quiz count
  weekly_earnings: bigint;
  // Monthly limits
  monthly_quizzes: number;   // Current month quiz count
  monthly_earnings: bigint;
  // Yearly limits
  yearly_quizzes: number;    // Current year quiz count (u16)
  yearly_earnings: bigint;
};

type TransactionRecord = {
  timestamp: bigint;
  tx_type: { QuizReward: null } | { Unstake: null };
  amount: bigint;
};

type TransactionPage = {
  transactions: TransactionRecord[];
  local_count: bigint;
  archived_count: bigint;
  total_count: bigint;
  current_page: number;
  total_pages: number;
  has_archive_data: boolean;
  archive_canister_id: Principal | null;
  source: string;
};

```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_profile` | `principal` | `Option<UserProfile>` | Get user's profile and balance |
| `get_user_stats` | `principal` | `UserTimeStats` | Periodic quiz/earning limits |
| `is_quiz_completed` | `principal, unit_id: string` | `bool` | Check if quiz was completed |
| `get_user_transactions` | `principal` | `Vec<TransactionRecord>` | Transaction history (local only) |
| `get_transactions_page` | `principal, page: nat32` | `TransactionPage` | Paginated history (local + archive) |
| `get_user_count` | - | `nat64` | Total users in this shard |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `register_user` | `UserProfileUpdate` | `Result<(), String>` | Register new user |
| `update_profile` | `UserProfileUpdate` | `Result<(), String>` | Update profile info |
| `submit_quiz` | `unit_id: string, answers: Blob` | `Result<nat64, String>` | Submit quiz, earn tokens |
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

// Submit quiz (answers as encoded Blob)
const quizBlob = new Uint8Array([0, 2, 1, 3]);
const result = await userProfileActor.submit_quiz("unit_1", quizBlob);
if ('Ok' in result) {
  console.log(`Earned ${Number(result.Ok) / 1e8} GHC`); 
}


// Check limits
const stats = await userProfileActor.get_user_stats(userPrincipal);
const quizzesRemaining = 5 - stats.daily_quizzes;

// Unstake tokens (100% returned, no penalty)
const unstakeResult = await userProfileActor.unstake(BigInt(100_000_000)); // 1 GHC
```

---

## 5. Learning Engine Canister

**Canister**: `learning_engine`  
**Purpose**: Hierarchical educational content and quiz management.

> **Updated**: January 2026 - Uses new `ContentNode` structure for hierarchical content (CHAPTER → SECTION → UNIT). Quiz rewards are now globally configured at 100 GHC per quiz.

### Types

```typescript
// Media attachment types
type MediaType = 
  | { Video: null }
  | { Audio: null }
  | { Image: null }
  | { PDF: null };

type MediaContent = {
  media_type: MediaType;
  url: string;
  thumbnail_url: string | null;
  duration_seconds: number | null;
  file_hash: string | null;
};

// Quiz types (answers NOT exposed to frontend)
type PublicQuizQuestion = {
  question: string;
  options: string[];
  // Note: answer is NOT included for security
};

type PublicQuizData = {
  questions: PublicQuizQuestion[];
};

// Main content node type (hierarchical)
type PublicContentNode = {
  id: string;                    // Unique identifier
  parent_id: string | null;      // Parent node ID (null for root chapters)
  order: number;                 // Display order within parent
  display_type: string;          // "CHAPTER" | "SECTION" | "UNIT"
  title: string;
  description: string | null;
  content: string | null;        // Educational content (typically for UNITs)
  paraphrase: string | null;     // Summary/key points
  media: MediaContent | null;    // Video/audio/image attachment
  quiz: PublicQuizData | null;   // Quiz (typically for UNITs)
  created_at: bigint;
  updated_at: bigint;
  version: bigint;
};

```

### Content Hierarchy

```
CHAPTER (root node, parent_id = null)
├── SECTION (optional grouping)
│   ├── UNIT (has content + quiz)
│   └── UNIT
├── UNIT (direct child of chapter)
└── SECTION
    └── UNIT
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_root_nodes` | - | `Vec<PublicContentNode>` | Get all chapters (nodes without parents) |
| `get_children` | `parent_id: string` | `Vec<PublicContentNode>` | Get children of a node, sorted by order |
| `get_content_node` | `id: string` | `Option<PublicContentNode>` | Get single node by ID |
| `get_content_stats` | - | `(nat64, nat64)` | (total_nodes, quizzes_count) |
| `verify_quiz` | `content_id: string, answers: Blob` | `(bool, nat64, nat64)` | (passed, correct, total) |


### Code Examples

```typescript
// Get curriculum hierarchy
async function loadCurriculum() {
  // 1. Get all chapters (root nodes)
  const chapters = await learningEngineActor.get_root_nodes();
  
  // 2. For each chapter, load children
  for (const chapter of chapters) {
    const children = await learningEngineActor.get_children(chapter.id);
    
    // Children could be SECTIONs or UNITs
    for (const child of children) {
      if (child.display_type === "SECTION") {
        // Load units within section
        const units = await learningEngineActor.get_children(child.id);
        // units have quiz data
      } else if (child.display_type === "UNIT") {
        // Direct unit under chapter
        console.log(`Unit: ${child.title}, has quiz: ${child.quiz !== null}`);
      }
    }
  }
}

// Get a specific content node
const node = await learningEngineActor.get_content_node("chapter_1");
if (node.length > 0) {
  const [content] = node;
  console.log(`Title: ${content.title}`);
  console.log(`Type: ${content.display_type}`);
}

// Get statistics
const [nodeCount, quizCount] = await learningEngineActor.get_content_stats();
console.log(`${nodeCount} content nodes, ${quizCount} quizzes`);

```

### Quiz Submission (via user_profile)

> **Note**: Quiz submission is done through `user_profile` canister, NOT learning_engine.

```typescript
// Submit quiz answers (0-indexed option numbers)
const result = await userProfileActor.submit_quiz("unit_1", [0, 2, 1, 3]);
if ('Ok' in result) {
  console.log(`Earned ${Number(result.Ok) / 1e8} GHC`); // 100 GHC per quiz
} else {
  console.log(`Error: ${result.Err}`);
}
```


---

## 6. Staking Hub Canister

**Canister**: `staking_hub`  
**Purpose**: Global statistics, VUC provider, and user voting power oracle.

> **Note**: Board member management is handled by `governance_canister`.

### Types

```typescript
type GlobalStats = {
  total_staked: bigint;    // Total tokens staked
  total_unstaked: bigint;  // Total tokens unstaked
  total_allocated: bigint; // Total tokens mined
};

type TokenLimits = {
  max_daily_tokens: bigint;
  max_weekly_tokens: bigint;
  max_monthly_tokens: bigint;
  max_yearly_tokens: bigint;
};

type TokenLimitsConfig = {
  reward_amount: bigint;
  pass_threshold_percent: number;
  max_daily_attempts: number;
  regular_limits: TokenLimits;
  subscribed_limits: TokenLimits;
  version: bigint;
};

```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_global_stats` | - | `GlobalStats` | Platform-wide statistics |
| `get_vuc` | - | `nat64` | VUC (board member voting power pool) |
| `get_total_voting_power` | - | `nat64` | VUC + total_staked |
| `get_tokenomics` | - | `(nat64, nat64, nat64, nat64)` | (max_supply, allocated, vuc, total_power) |
| `get_user_shard` | `principal` | `Option<Principal>` | Get user's shard canister |
| `get_token_limits` | - | `TokenLimitsConfig` | Current quiz reward and token limit settings |


### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `fetch_user_voting_power` | `principal` | `nat64` | Get user's staked balance (async) |

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

// Get VUC (board member voting power pool)
const vuc = await stakingHubActor.get_vuc();

// Get user's staked balance (regular users)
const stakedBalance = await stakingHubActor.fetch_user_voting_power(userPrincipal);

// Get token limits and rewards
const limits = await stakingHubActor.get_token_limits();
console.log(`Daily Max: ${Number(limits.regular_limits.max_daily_tokens) / 1e8} GHC`);

```

---

## 7. Treasury Canister

**Canister**: `treasury_canister`  
**Purpose**: Token custody, balance tracking, MMCR releases, and transfer execution.

> **Security Model**: The treasury canister only accepts transfer execution calls from the governance canister. This separation ensures that proposals must go through the full voting process before funds can be moved.

### Types

```typescript
type TokenType = 
  | { GHC: null }
  | { USDC: null }
  | { ICP: null };

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

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_treasury_state` | - | `TreasuryState` | Full treasury state |
| `get_spendable_balance` | - | `nat64` | Current allowance |
| `get_treasury_balance` | - | `nat64` | Total balance |
| `get_mmcr_status` | - | `MMCRStatus` | MMCR progress |
| `can_transfer` | `amount: nat64, token_type: TokenType` | `bool` | Check if transfer is within allowance |
| `get_governance_canister_id` | - | `Principal` | Get linked governance canister |
| `get_ledger_id` | - | `Principal` | Get linked ledger canister |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `execute_transfer` | `ExecuteTransferInput` | `Result<nat64, String>` | Execute transfer (governance only) |
| `execute_mmcr` | - | `Result<nat64, String>` | Trigger monthly release |
| `set_governance_canister_id` | `Principal` | `Result<(), String>` | Set governance canister (admin) |

### Code Examples

```typescript
// Get treasury overview
const treasury = await treasuryActor.get_treasury_state();
console.log(`Balance: ${Number(treasury.balance) / 1e8} GHC`);
console.log(`Spendable: ${Number(treasury.allowance) / 1e8} GHC`);
console.log(`MMCR Progress: ${treasury.mmcr_count}/240`);

// Get MMCR status
const mmcr = await treasuryActor.get_mmcr_status();
const daysUntilNext = Number(mmcr.seconds_until_next) / 86400;
console.log(`Next release: ${Number(mmcr.next_release_amount) / 1e8} GHC in ${daysUntilNext.toFixed(1)} days`);

// Check if a transfer is within allowance
const canTransfer = await treasuryActor.can_transfer(
  BigInt(1_000_000 * 100_000_000), // 1M GHC
  { GHC: null }
);
```

---

## 8. Governance Canister

**Canister**: `governance_canister`  
**Purpose**: Proposal lifecycle, voting, and board member management.

> **Inter-canister Flow**: When a treasury proposal is executed, the governance canister calls `treasury_canister.execute_transfer()` to move funds. This ensures proper authorization.

### Governance Parameters (Configurable via Proposals)

All timing parameters are configurable via `UpdateGovernanceConfig` proposals.

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Min voting power to propose | 150 tokens | Required to create proposals |
| Support threshold | 15,000 tokens | Voting power needed to move from Proposed to Active |
| Approval percentage | 30% | Percentage of total staked needed for YES votes to pass |
| **Board Member Rule** | **At least 1** | A proposal MUST receive at least one YES vote from a board member identity to be approved. |
| **Support period** | 7 days | Time for regular user proposals to gather support (configurable) |
| **Voting period** | 14 days | Duration for active voting (configurable) |
| **Resubmission cooldown** | 180 days | Wait time for rejected proposals (configurable) |

### Types

```typescript
type ProposalStatus = 
  | { Proposed: null }   // Gathering support (regular users)
  | { Active: null }     // Voting in progress
  | { Approved: null }   // Passed, pending execution
  | { Rejected: null }   // Failed
  | { Executed: null };  // Successfully executed

type ProposalType = 
  | { Treasury: null }            // Token transfer proposal
  | { AddBoardMember: null }      // Add new board member
  | { RemoveBoardMember: null }   // Remove existing board member
  | { UpdateBoardMemberShare: null } // Update board member's share
  | { UpdateGovernanceConfig: null } // Update governance parameters
  | { AddContentFromStaging: null } // Approve educational content
  | { DeleteContentNode: null }     // Delete educational content
  | { UpdateTokenLimits: null }    // Update quiz rewards/limits
  | { UpdateSentinel: null };       // Update sentinel member

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
  proposal_type: ProposalType;
  title: string;
  description: string;
  recipient: [] | [Principal];
  amount: [] | [bigint];
  token_type: [] | [TokenType];
  category: [] | [ProposalCategory];
  external_link: [] | [string];
  board_member_payload: [] | [AddBoardMemberPayload];
  remove_board_member_payload: [] | [RemoveBoardMemberPayload];
  update_board_member_payload: [] | [UpdateBoardMemberSharePayload];
  votes_yes: bigint;
  votes_no: bigint;
  voter_count: bigint;
  board_member_yes_count: number;
  support_amount: bigint;
  supporter_count: bigint;
  status: ProposalStatus;
};

type AddBoardMemberPayload = {
  new_member: Principal;
  share_bps: number;    // Basis points (100 = 1%)
};

type RemoveBoardMemberPayload = {
  member_to_remove: Principal;
};

type UpdateBoardMemberSharePayload = {
  member: Principal;
  new_share_bps: number; // Basis points
};


type VoteRecord = {
  voter: Principal;
  proposal_id: bigint;
  vote: boolean;        // true = YES, false = NO
  voting_power: bigint;
  timestamp: bigint;
};

type SupportRecord = {
  supporter: Principal;
  proposal_id: bigint;
  support_amount: bigint;
  timestamp: bigint;
};

type TreasuryProposalInput = {
  title: string;
  description: string;
  recipient: Principal;
  amount: bigint;
  token_type: TokenType;
  category: ProposalCategory;
  external_link: [] | [string];
};

type BoardMemberProposalInput = {
  title: string;
  description: string;
  new_member: Principal;
  share_bps: number;   // Share in basis points (1-9900)
  external_link: [] | [string];
};

type RemoveBoardMemberProposalInput = {
  title: string;
  description: string;
  member_to_remove: Principal;
  external_link: [] | [string];
};

type UpdateBoardMemberShareProposalInput = {
  title: string;
  description: string;
  member: Principal;
  new_share_bps: number; // New share in basis points (1-9900)
  external_link: [] | [string];
};

type BoardMemberShare = {
  member: Principal;
  share_bps: number;    // Basis points (10000 = 100%)
  is_sentinel: boolean; // True for the sentinel member (1 unit power)
};

```

### Query Methods - Proposals

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_proposal` | `id: nat64` | `Option<Proposal>` | Get single proposal |
| `get_active_proposals` | - | `Vec<Proposal>` | All active proposals |
| `get_all_proposals` | - | `Vec<Proposal>` | All proposals ever |
| `get_proposal_votes` | `id: nat64` | `Vec<VoteRecord>` | Who voted on proposal |
| `get_proposal_supporters` | `id: nat64` | `Vec<SupportRecord>` | Who supported proposal |
| `has_voted` | `id: nat64, voter: Principal` | `bool` | Check if user voted |
| `get_governance_config` | - | `(nat64, nat64, nat64, nat64, nat64, nat8)` | support_threshold, min_voting_power, support_period, voting_period, resubmission_cooldown, approval_percentage |

### Governance Config Structure (from get_governance_config)

1. **support_threshold**: Voting power (e8s) needed to move from Proposed to Active.
2. **min_voting_power**: Min power (e8s) to create proposals.
3. **support_period**: Duration in days for gathering support.
4. **voting_period**: Duration in days for active voting.
5. **resubmission_cooldown**: Days to wait before resubmitting rejected proposal.
6. **approval_percentage**: Percentage of total staked for YES votes (1-100).


### Query Methods - Board Members

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_board_member_shares` | - | `Vec<BoardMemberShare>` | All board members (includes sentinel) |
| `get_all_board_member_voting_powers` | - | `Result<Vec<(Principal, u16, u64, bool)>, String>` | Members with shares, voting power, and sentinel status |
| `get_board_member_share` | `principal` | `Option<nat16>` | Member's BPS share |
| `get_board_member_count` | - | `nat64` | Number of board members |
| `is_board_member` | `principal` | `bool` | Check if principal is board member |
| `are_board_shares_locked` | - | `bool` | Check if shares are locked |
| `get_treasury_canister_id` | - | `Principal` | Get linked treasury canister |
| `get_staking_hub_id` | - | `Principal` | Get linked staking hub |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `create_treasury_proposal` | `TreasuryProposalInput` | `Result<nat64, String>` | Create treasury proposal |
| `create_board_member_proposal` | `BoardMemberProposalInput` | `Result<nat64, String>` | Create add board member proposal |
| `create_remove_board_member_proposal` | `RemoveBoardMemberProposalInput` | `Result<nat64, String>` | Create remove board member proposal |
| `create_update_board_member_share_proposal` | `UpdateBoardMemberShareProposalInput` | `Result<nat64, String>` | Create update board member share proposal |
| `create_update_governance_config_proposal` | `UpdateGovernanceConfigProposalInput` | `Result<nat64, String>` | Update timings/thresholds |
| `create_update_token_limits_proposal` | `UpdateTokenLimitsProposalInput` | `Result<nat64, String>` | Update quiz rewards/limits |
| `create_update_sentinel_proposal` | `UpdateSentinelProposalInput` | `Result<nat64, String>` | Update sentinel member |
| `support_proposal` | `id: nat64` | `Result<(), String>` | Support proposal (non-board) |
| `vote` | `id: nat64, approve: bool` | `Result<(), String>` | Vote on proposal |
| `execute_proposal` | `id: nat64` | `Result<(), String>` | Execute approved proposal |
| `finalize_proposal` | `id: nat64` | `Result<ProposalStatus, String>` | Force finalization |
| `set_board_member_shares` | `Vec<BoardMemberShare>` | `Result<(), String>` | Set board members (admin) |
| `lock_board_member_shares` | - | `Result<(), String>` | Lock shares (admin) |
| `set_sentinel_member` | `Principal` | `Result<(), String>` | Set sentinel (admin) |


### Code Examples - Proposals

```typescript
// Get governance config
const [minPower, threshold, supportDays, votingDays, cooldown] = 
  await govActor.get_governance_config();

// Check if user is a board member
const isBoardMember = await govActor.is_board_member(userPrincipal);

// Get board members with their shares and voting powers
const result = await govActor.get_all_board_member_voting_powers();
if ('Ok' in result) {
  result.Ok.forEach(([principal, shareBps, votingPower, isSentinel]) => {
    const percentage = shareBps / 100;
    const powerGhc = Number(votingPower) / 1e8;
    console.log(`${principal.toText()}:`);
    console.log(`  - Role: ${isSentinel ? 'Sentinel' : 'Board Member'}`);
    console.log(`  - Share: ${percentage}% (${shareBps} BPS)`);
    console.log(`  - Voting Power: ${powerGhc} GHC`);
  });
}

// Create a treasury proposal
const result = await govActor.create_treasury_proposal({
  title: "Marketing Campaign Q1",
  description: "Fund marketing initiatives for Q1 2026",
  recipient: recipientPrincipal,
  amount: BigInt(10_000 * 100_000_000), // 10,000 GHC
  token_type: { GHC: null },
  category: { Marketing: null },
  external_link: ["https://forum.example.com/proposal/123"]
});

// Create an add board member proposal
const addBoardResult = await govActor.create_board_member_proposal({
  title: "Add Alice to Board",
  description: "Proposing to add Alice as board member with 15% share",
  new_member: alicePrincipal,
  share_bps: 1500, // 15% (1500 BPS)
  external_link: []
});

// Create a proposal to update the sentinel member
const updateSentinelResult = await govActor.create_update_sentinel_proposal({
  title: "Update Sentinel Member",
  description: "Proposing to change the sentinel member to Carol.",
  new_sentinel: carolPrincipal,
  external_link: []
});

// Create a remove board member proposal
const removeBoardResult = await govActor.create_remove_board_member_proposal({
  title: "Remove Bob from Board",
  description: "Proposing to remove Bob from the board. His share will be redistributed.",
  member_to_remove: bobPrincipal,
  external_link: []
});

// Create an update board member share proposal
const updateShareResult = await govActor.create_update_board_member_share_proposal({
  title: "Update Alice's Board Share",
  description: "Proposing to update Alice's board share from 15% to 20%",
  member: alicePrincipal,
  new_share_bps: 2000, // 20%
  external_link: []
});


if ('Ok' in result) {
  console.log(`Proposal created with ID: ${result.Ok}`);
}

// Support a proposal (non-board members in Proposed state)
await govActor.support_proposal(BigInt(0));

// Vote on a proposal
await govActor.vote(BigInt(0), true); // Vote YES on proposal #0

// Execute an approved proposal (triggers treasury transfer)
await govActor.execute_proposal(BigInt(0));

// See who voted on a proposal
const votes = await govActor.get_proposal_votes(BigInt(0));
votes.forEach(v => {
  console.log(`${v.voter}: ${v.vote ? 'YES' : 'NO'} (${Number(v.voting_power) / 1e8} tokens)`);
});
```

---

## 9. Content Governance Canisters

> **New in January 2026**: Content governance enables decentralized management of educational content through proposals.

### Overview

Two new canisters handle content staging and approval:

| Canister | Purpose |
|----------|---------|
| `staging_assets` | Temporary storage for content awaiting governance approval |
| `media_assets` | Permanent storage for approved media (images, videos) |

### Content Proposal Flow

```
1. Author stages content    → staging_assets.stage_content()
2. Creates proposal         → governance_canister.create_add_content_proposal()
3. Board votes              → governance_canister.vote()
4. Proposal approved        → governance_canister.execute_proposal()
5. Content loaded           → learning_engine (automatic)
6. Staged content cleaned   → staging_assets.delete_staged_content()
```

### staging_assets Types

```typescript
type StagingStatus = 
  | { Pending: null }        // Awaiting proposal
  | { ProposalCreated: null } // Has proposal
  | { Loading: null }        // Being loaded to learning_engine
  | { Loaded: null }         // Successfully loaded
  | { Rejected: null };      // Proposal rejected

type StagedContentInfo = {
  content_hash: string;      // SHA256 of content
  title: string;
  description: string;
  node_count: number;        // Number of content nodes
  stager: Principal;         // Who staged it
  staged_at: bigint;
  proposal_id: bigint | null;
  status: StagingStatus;
};
```

### staging_assets Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_staged_content_info` | `hash: string` | `Option<StagedContentInfo>` | Get staging info |
| `staged_content_exists` | `hash: string` | `bool` | Check if content exists |
| `list_staged_content` | - | `Vec<StagedContentInfo>` | List all staged content |
| `get_staged_by_stager` | `principal` | `Vec<StagedContentInfo>` | Get user's staged content |
| `get_staged_count` | - | `nat64` | Total staged items |

### staging_assets Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `stage_content` | `title, description, nodes: Vec<ContentNode>` | `Result<string, String>` | Stage content, returns hash |
| `delete_staged_content` | `hash: string` | `Result<(), String>` | Delete staged content |

### Content Proposal Types in governance_canister

```typescript
type CreateAddContentProposalInput = {
  title: string;
  description: string;
  staging_canister: Principal;
  staging_path: string;       // Content hash
  content_hash: string;
  content_title: string;
  unit_count: number;
  external_link: string | null;
};

type CreateDeleteContentProposalInput = {
  title: string;
  description: string;
  content_id: string;         // ID of content to delete
  reason: string;
  external_link: string | null;
};

type CreateUpdateTokenLimitsProposalInput = {
  title: string;
  description: string;
  new_reward_amount: bigint | null;     // In e8s
  new_pass_threshold: number | null;    // 0-100
  new_max_attempts: number | null;
  new_regular_limits: TokenLimits | null;
  new_subscribed_limits: TokenLimits | null;
  external_link: string | null;
};

```

### Code Examples

```typescript
// Check staged content
const stagedList = await stagingActor.list_staged_content();
console.log(`${stagedList.length} items pending approval`);

// Get specific staged content
const info = await stagingActor.get_staged_content_info(contentHash);
if (info.length > 0) {
  const [staged] = info;
  console.log(`Status: ${Object.keys(staged.status)[0]}`);
  console.log(`Nodes: ${staged.node_count}`);
}

// Create content proposal (after staging)
const result = await governanceActor.create_add_content_proposal({
  title: "Add Climate Science Chapter",
  description: "New educational content about climate change",
  staging_canister: stagingCanisterId,
  staging_path: contentHash,
  content_hash: contentHash,
  content_title: "Climate Science",
  unit_count: 5,
  external_link: []
});

// Create token limits update proposal
const limitResult = await governanceActor.create_update_token_limits_proposal({
  title: "Increase Quiz Rewards",
  description: "Raise quiz rewards to 150 GHC",
  new_reward_amount: [BigInt(150 * 1e8)],  // 150 GHC
  new_pass_threshold: [],                   // Keep current
  new_max_attempts: [],                     // Keep current
  new_regular_limits: [],
  new_subscribed_limits: [],
  external_link: []
});

```

---

---

## 10. KYC Canister

**Canister**: `kyc_canister`  
**Purpose**: Identity verification management and tier updates.

### Types

```typescript
type VerificationTier = { KYC: null } | { None: null } | { Human: null };

type KycStatus = {
  user: Principal;
  tier: VerificationTier;
  provider: string;
  verified_at: bigint;
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_user_kyc_status` | `Principal` | `Option<KycStatus>` | Get user's KYC details |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `submit_kyc_data` | `data: string` | `Result<string, String>` | Submit KYC data for review |
| `verify_identity` | `Principal` | `Result<VerificationTier, String>` | Internal/Admin: verify user |

---

## 11. Subscription Canister

**Canister**: `subscription_canister`  
**Purpose**: Paid subscription management.

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_subscription_status` | `Principal` | `bool` | Check if user is subscriber |
| `get_staking_hub` | - | `Principal` | Get linked Staking Hub |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `request_checkout` | `Principal` | `Result<string, String>` | Start subscription flow (returns session ID) |
| `confirm_payment` | `session_id: string` | `Result<(), String>` | Confirm successful payment |

---

## 12. GHC Ledger (ICRC-1)


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

## 13. Founder Vesting Canister


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

## 14. ICRC-1 Index Canister

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

## 15. ICO Canister


**Canister**: `ico_canister`  
**Purpose**: Fixed-price token sales with ckUSDC payments.

> **Note**: This canister is designed for credit card integration via MoonPay. Users purchase ckUSDC through MoonPay, then use it to buy GHC tokens at a fixed price.

### Types

```typescript
type IcoState = {
  admin_principal: Principal;
  treasury_principal: Principal;
  ghc_ledger_id: Principal;
  ckusdc_ledger_id: Principal;
  price_per_token_e6: bigint;    // Price of 1 GHC in USDC (e6 format, e.g., 0.05 USDC = 50,000)
  ghc_decimals: number;           // Typically 8
  total_raised_usdc: bigint;     // Total USDC raised
  total_sold_ghc: bigint;        // Total GHC sold
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_ico_stats` | - | `IcoState` | Get ICO statistics |

### Update Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `buy_ghc` | `amount_ghc: nat` | `Result<string, String>` | Purchase GHC with ckUSDC |
| `withdraw_usdc` | `destination: Principal, amount: nat` | `Result<string, String>` | Admin: withdraw USDC |
| `withdraw_ghc` | `destination: Principal, amount: nat` | `Result<string, String>` | Admin: withdraw unsold GHC |
| `end_sale` | - | `Result<string, String>` | Admin: end sale, sweep funds to treasury |

### Code Examples

```typescript
// Prerequisites: User must first approve ICO canister to spend their ckUSDC via icrc2_approve

// Get ICO stats
const stats = await icoActor.get_ico_stats();
console.log(`Price: ${Number(stats.price_per_token_e6) / 1e6} USDC per GHC`);
console.log(`Total raised: ${Number(stats.total_raised_usdc) / 1e6} USDC`);
console.log(`Total sold: ${Number(stats.total_sold_ghc) / 1e8} GHC`);

// Buy GHC tokens
const amountGhc = BigInt(100 * 1e8); // Buy 100 GHC
const result = await icoActor.buy_ghc(amountGhc);
if ('Ok' in result) {
  console.log('Purchase successful!');
}
```

### MoonPay Integration Flow

```
1. User connects wallet (Internet Identity / NFID)
2. User clicks "Buy with Credit Card"
3. Frontend opens MoonPay widget configured for ckUSDC
4. User completes payment, receives ckUSDC to their wallet
5. User approves ICO canister to spend ckUSDC (icrc2_approve)
6. User calls buy_ghc with desired amount
7. ICO canister pulls ckUSDC, sends GHC to user
```

---

## 16. Sonic Adapter Canister


**Canister**: `sonic_adapter`  
**Purpose**: DEX integration for adding liquidity and token swaps on Sonic.

> **Note**: This canister is owned by the governance canister and can only be called through governance proposals for security.

### Types

```typescript
type LaunchIcoArgs = {
  ghc_amount: bigint;   // Amount of GHC to add to pool
  usdc_amount: bigint;  // Amount of USDC to add to pool
};
```

### Update Methods (Governance Only)

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `launch_ico` | `LaunchIcoArgs` | `Result<string, String>` | Add initial liquidity to Sonic pool |
| `add_liquidity` | `token_a, token_b, amount_a, amount_b` | `Result<string, String>` | Add liquidity to any pair |
| `swap` | `token_in, amount_in, token_out, min_amount_out` | `Result<string, String>` | Swap tokens |

### Code Examples

```typescript
// NOTE: These are typically called via governance proposals, not directly from frontend

// Add liquidity (governance proposal execution)
const result = await sonicAdapter.add_liquidity(
  ghcLedgerId,        // Token A
  usdcLedgerId,       // Token B
  BigInt(1000000e8),  // 1M GHC
  BigInt(50000e6),    // 50K USDC (at $0.05/GHC)
);

// Swap tokens
const swapResult = await sonicAdapter.swap(
  usdcLedgerId,       // Token in
  BigInt(100e6),      // 100 USDC
  ghcLedgerId,        // Token out
  BigInt(1900e8),     // Min 1900 GHC (with slippage)
);
```

---

## 17. Archive Canister


**Canister**: `archive_canister`  
**Purpose**: Long-term storage for transaction history from user_profile shards.

> **Note**: The archive canister receives transaction data from user_profile shards when users exceed the local retention limit (100 transactions). This ensures transaction history is preserved indefinitely without overwhelming shard storage.

### Types

```typescript
type ArchiveKey = {
  user: Principal;
  sequence: bigint;   // Transaction sequence number
};

type ArchivedTransaction = {
  timestamp: bigint;
  transaction_type: string;
  amount: bigint;
  metadata: string;
  sequence: bigint;
  archived_at: bigint;
};

type ArchiveStats = {
  entry_count: bigint;
  size_bytes: bigint;
  is_full: boolean;
  parent_shard: Principal;
  next_archive: Principal | null;  // For chaining archives
};
```

### Query Methods

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `get_archived_transactions` | `user: Principal, start: nat64 | null, limit: nat64` | `Vec<ArchivedTransaction>` | Get archived transactions |
| `get_stats` | - | `ArchiveStats` | Get archive statistics |
| `get_archived_count` | `user: Principal` | `nat64` | Get count for specific user |
| `get_total_archived_count` | - | `nat64` | Get total entries in archive |


### Update Methods (Shard Only)

| Method | Arguments | Returns | Description |
|--------|-----------|---------|-------------|
| `receive_archive_batch` | `user: Principal, transactions: Vec<...>` | `Result<nat64, String>` | Receive transactions from shard |

### Code Examples

```typescript
// Get user's archived transaction history
const archivedTxs = await archiveActor.get_user_transactions(
  userPrincipal,
  BigInt(0),   // Start from first
  20           // Limit to 20 results
);

archivedTxs.forEach(tx => {
  const date = new Date(Number(tx.timestamp) / 1e6);
  console.log(`${date}: ${tx.tx_type} - ${Number(tx.amount) / 1e8} GHC`);
});

// Check archive capacity
const stats = await archiveActor.get_stats();
console.log(`Archive entries: ${stats.entry_count}`);
if (stats.is_full && stats.next_archive) {
  console.log(`Overflow to: ${stats.next_archive}`);
}

```

### Complete Transaction History

To get a user's complete transaction history, combine local and archived:

```typescript
async function getCompleteHistory(userPrincipal: Principal) {
  // 1. Get local transactions from user_profile
  const localTxs = await userProfileActor.get_user_transactions(userPrincipal);
  
  // 2. Get archived transactions
  const archivedTxs = await archiveActor.get_archived_transactions(
    userPrincipal, [BigInt(0)], BigInt(1000)
  );

  
  // 3. Combine and sort by timestamp
  const allTxs = [...archivedTxs, ...localTxs];
  allTxs.sort((a, b) => Number(a.timestamp - b.timestamp));
  
  return allTxs;
}
```

---

## 18. Complete React Integration


```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { Principal } from "@dfinity/principal";

// Import generated IDLs
import { idlFactory as userProfileIdl } from "./declarations/user_profile";
import { idlFactory as learningEngineIdl } from "./declarations/learning_engine";
import { idlFactory as stakingHubIdl } from "./declarations/staking_hub";
import { idlFactory as treasuryIdl } from "./declarations/treasury_canister";
import { idlFactory as govIdl } from "./declarations/governance_canister";
import { idlFactory as ledgerIdl } from "./declarations/ghc_ledger";
import { idlFactory as vestingIdl } from "./declarations/founder_vesting";
import { idlFactory as kycIdl } from "./declarations/kyc_canister";
import { idlFactory as subscriptionIdl } from "./declarations/subscription_canister";

import { CANISTER_IDS } from "./canister-ids";


class GHCClient {
  private agent: HttpAgent;
  private identity: any;
  
  public userProfile: any;
  public learningEngine: any;
  public stakingHub: any;
  public treasury: any;     // NEW: Treasury canister
  public governance: any;   // NEW: Governance canister
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
    
    // NEW: Separate Treasury and Governance canisters
    this.treasury = Actor.createActor(treasuryIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.treasury_canister,
    });
    
    this.governance = Actor.createActor(govIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.governance_canister,
    });
    
    this.ledger = Actor.createActor(ledgerIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.ghc_ledger,
    });
    
    this.vesting = Actor.createActor(vestingIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.founder_vesting,
    });

    this.kyc = Actor.createActor(kycIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.kyc_canister,
    });

    this.subscription = Actor.createActor(subscriptionIdl, {
      agent: this.agent,
      canisterId: CANISTER_IDS.subscription_canister,
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
      isSubscribed: profile?.is_subscribed || false,
    };
  }

  
  // === GOVERNANCE ===
  
  async isBoardMember() {
    return this.governance.is_board_member(this.getPrincipal());
  }
  
  async getUserVotingPower() {
    // Get effective voting power (handles both Regular Users and Board Members)
    const result = await this.governance.get_my_voting_power();
    if ('Ok' in result) {
        return Number(result.Ok) / 1e8;
    }
    return 0;
  }
  
  async getBoardMemberShares() {
    return this.governance.get_board_member_shares();
  }
  
  async createTreasuryProposal(input: any) {
    return this.governance.create_treasury_proposal(input);
  }
  
  async createBoardMemberProposal(input: any) {
    return this.governance.create_board_member_proposal(input);
  }
  
  async vote(proposalId: bigint, approve: boolean) {
    return this.governance.vote(proposalId, approve);
  }
  
  async executeProposal(proposalId: bigint) {
    return this.governance.execute_proposal(proposalId);
  }
}

export const ghcClient = new GHCClient();
```

---

## 19. Error Handling


### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `"User not registered"` | Calling before registration | Call `register_user` first |
| `"Quiz already completed"` | Resubmitting same quiz | Check `is_quiz_completed` |
| `"Daily quiz limit reached"` | Daily limit exceeded | Wait until next day |
| `"Weekly quiz limit reached"` | Weekly limit exceeded | Wait until next week |
| `"Monthly quiz limit reached"` | Monthly limit exceeded | Wait until next month |
| `"Yearly quiz limit reached"` | Yearly limit exceeded | Wait until next year |
| `"Insufficient voting power"` | < 150 tokens to propose | Earn more tokens |
| `"Already voted on this proposal"` | Double voting | Check `has_voted` first |
| `"Proposal is not active"` | Voting on concluded proposal | Check `status` |
| `"Voting period has ended"` | Late vote | Proposal already finalized |
| `"No voting power"` | 0 staked tokens | Stake tokens first |
| `"Insufficient treasury allowance"` | Amount > allowance | Wait for MMCR |
| `"Unauthorized"` | Calling admin-only method | Must be admin/governance |

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

## 20. UI Pages Reference


### User Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Dashboard** | user_profile, ghc_ledger | `get_profile`, `icrc1_balance_of` |
| **Learn** | learning_engine, user_profile | `get_root_nodes`, `get_children`, `submit_quiz` |
| **Wallet** | ghc_ledger, icrc1_index, archive_canister | `icrc1_balance_of`, `get_account_transactions` |
| **Transfer** | ghc_ledger | `icrc1_transfer` |
| **Buy GHC** | ico_canister, ckusdc_ledger | `get_ico_stats`, `buy_ghc` |

### Governance Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Proposals** | governance_canister | `get_active_proposals`, `get_my_voting_power` |
| **Create Proposal** | governance_canister | `create_treasury_proposal`, `create_board_member_proposal` |
| **Vote** | governance_canister | `vote`, `get_proposal_votes` |
| **Treasury** | treasury_canister | `get_treasury_state`, `get_mmcr_status` |
| **Governance Config** | governance_canister | `get_governance_config` |

### Admin/Founder Pages

| Page | Canisters | Key Methods |
|------|-----------|-------------|
| **Global Stats** | staking_hub | `get_global_stats`, `get_tokenomics` |
| **Founder Vesting** | founder_vesting | `get_vesting_status`, `claim_vested` |
| **Board Member Management** | governance_canister | `get_board_member_shares`, `set_board_member_shares` |
| **ICO Management** | ico_canister | `get_ico_stats`, `withdraw_usdc`, `end_sale` |
| **Archive Stats** | archive_canister | `get_archive_stats` |

---

## 21. Migration Guide


### Overview

The `operational_governance` canister has been refactored into two separate canisters:

| Old | New | Purpose |
|-----|-----|---------|
| `operational_governance` | `treasury_canister` | Token custody, MMCR, transfer execution |
| `operational_governance` | `governance_canister` | Proposals, voting, board members |

### Method Migration Map

#### Treasury Methods (now in treasury_canister)

| Old Method | New Method | Notes |
|------------|------------|-------|
| `get_treasury_state()` | `get_treasury_state()` | Same signature |
| `get_spendable_balance()` | `get_spendable_balance()` | Same signature |
| `get_mmcr_status()` | `get_mmcr_status()` | Same signature |
| `execute_mmcr()` | `execute_mmcr()` | Same signature |
| *(internal)* | `execute_transfer()` | New - governance canister calls this |
| *(new)* | `can_transfer()` | New - check allowance |
| *(new)* | `get_treasury_balance()` | New - get total balance |

#### Governance Methods (now in governance_canister)

| Old Method | New Method | Notes |
|------------|------------|-------|
| `create_treasury_proposal()` | `create_treasury_proposal()` | Same, but calls treasury for allowance check |
| `create_board_member_proposal()` | `create_board_member_proposal()` | Same signature |
| `vote()` | `vote()` | Same signature |
| `support_proposal()` | `support_proposal()` | Same signature |
| `execute_proposal()` | `execute_proposal()` | Now calls treasury_canister for transfers |
| `finalize_proposal()` | `finalize_proposal()` | Same signature |
| `get_proposal()` | `get_proposal()` | Same signature |
| `get_active_proposals()` | `get_active_proposals()` | Same signature |
| `get_all_proposals()` | `get_all_proposals()` | Same signature |
| `get_proposal_votes()` | `get_proposal_votes()` | Same signature |
| `get_proposal_supporters()` | `get_proposal_supporters()` | Same signature |
| `has_voted()` | `has_voted()` | Same signature |
| `get_governance_config()` | `get_governance_config()` | Now returns 5 values (added support_days) |
| `get_board_member_shares()` | `get_board_member_shares()` | Same signature |
| `get_board_member_share()` | `get_board_member_share()` | Same signature |
| `get_board_member_count()` | `get_board_member_count()` | Same signature |
| `is_board_member()` | `is_board_member()` | Same signature |
| `are_board_shares_locked()` | `are_board_shares_locked()` | Same signature |
| `set_board_member_shares()` | `set_board_member_shares()` | Same signature |
| `lock_board_member_shares()` | `lock_board_member_shares()` | Same signature |
| `get_user_voting_power()` | `get_user_voting_power()` | Same signature |
| `get_my_voting_power()` | `get_my_voting_power()` | Same signature |

### Frontend Code Changes

```typescript
// OLD CODE
import { idlFactory as govIdl } from "./declarations/operational_governance";

const governance = Actor.createActor(govIdl, {
  agent: this.agent,
  canisterId: CANISTER_IDS.operational_governance,
});

// Get treasury state (old way)
const treasury = await governance.get_treasury_state();

// Create proposal (old way)
await governance.create_treasury_proposal(input);
```

```typescript
// NEW CODE
import { idlFactory as treasuryIdl } from "./declarations/treasury_canister";
import { idlFactory as govIdl } from "./declarations/governance_canister";

const treasury = Actor.createActor(treasuryIdl, {
  agent: this.agent,
  canisterId: CANISTER_IDS.treasury_canister,
});

const governance = Actor.createActor(govIdl, {
  agent: this.agent,
  canisterId: CANISTER_IDS.governance_canister,
});

// Get treasury state (new way)
const treasuryState = await treasury.get_treasury_state();

// Create proposal (new way - same API)
await governance.create_treasury_proposal(input);
```

### Benefits of the New Architecture

1. **Security**: Treasury canister only accepts transfers from governance canister
2. **Separation of Concerns**: Cleaner code organization
3. **Independent Upgrades**: Can upgrade governance without touching treasury
4. **Multi-sig Ready**: Easier to implement execution approvals
5. **Auditability**: Clearer inter-canister call patterns
