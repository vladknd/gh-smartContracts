# Frontend Update Prompt for GHC Smart Contract Changes

**Date**: January 2026  
**Context**: The GreenHero Coin (GHC) smart contract backend has undergone significant architectural changes. This prompt provides everything a frontend developer/AI needs to update the React/TypeScript frontend application.

---

## 1. Summary of Backend Changes

### Architecture Refactoring
The system now uses **14 canisters** (up from ~6):

| Canister | Purpose | Status |
|----------|---------|--------|
| `user_profile` | User registration, quiz submission, staking, verification tiers | Existing (updated) |
| `staking_hub` | Global stats, VUC, voting power oracle, shard management | Existing |
| `learning_engine` | Educational content, quizzes | **Major update** |
| `governance_canister` | Proposals, voting, board members, configurable timings | **NEW** (split from operational_governance) |
| `treasury_canister` | Token custody, MMCR, transfers | **NEW** (split from operational_governance) |
| `ghc_ledger` | ICRC-1 token ledger | Existing |
| `founder_vesting` | Founder token vesting | Existing |
| `icrc1_index_canister` | Transaction history | Existing |
| `media_assets` | Approved media storage | **NEW** |
| `staging_assets` | Content staging for governance | **NEW** |
| `archive_canister` | Long-term transaction archival | **NEW** |
| `ico_canister` | Fixed-price token sales (ckUSDC/MoonPay) | **NEW** |
| `sonic_adapter` | DEX integration (Sonic) | **NEW** |
| `internet_identity` | Authentication | Existing |

### DEPRECATED - Do NOT use:
- `operational_governance` → replaced by `governance_canister` + `treasury_canister`
- `content_governance` → replaced by `media_assets` + `staging_assets`

### Key New Features:
- **Verification Tiers**: Users now have `None`, `Human` (PoH), or `KYC` verification levels
- **Quiz Limits**: Daily, weekly, monthly, and yearly quiz attempt limits (configurable via governance)
- **Configurable Governance Timings**: Support period, voting period, and resubmission cooldown are modifiable via proposals
- **ICO Support**: Fixed-price token sales with ckUSDC payments (MoonPay-ready)
- **Transaction Archiving**: Old transactions archived to archive_canister for unlimited history

---

## 2. TypeScript Declarations Location

TypeScript declarations are now provided in `src/declarations/<canister_name>/`:

```
src/declarations/
├── governance_canister/
│   ├── governance_canister.did.d.ts   # TypeScript types
│   ├── governance_canister.did.js     # Candid IDL factory
│   ├── index.js                       # Actor creation helpers
│   └── index.d.ts
├── treasury_canister/
│   ├── ...
├── learning_engine/
│   ├── ...
├── media_assets/
│   ├── ...
├── staging_assets/
│   ├── ...
├── archive_canister/
│   ├── ...
├── ico_canister/
│   ├── ...
├── sonic_adapter/
│   ├── ...
└── ... (other canisters)
```

### How to Import

```typescript
// Import IDL factory for actor creation
import { idlFactory as governanceIdl } from './declarations/governance_canister';

// Or import the pre-built actor creator
import { createActor as createGovernanceActor } from './declarations/governance_canister';
```

---

## 3. Key API Changes

### 3.1 User Profile - Verification Tiers (NEW)

**NEW**: Users now have verification tiers:

```typescript
type VerificationTier = 
  | { None: null }   // Fresh user - no verification
  | { Human: null }  // DecideID verified (Proof of Humanity)
  | { KYC: null };   // Full legal KYC verification

type UserProfile = {
  email: string;
  name: string;
  education: string;
  gender: string;
  verification_tier: VerificationTier;  // NEW field
  staked_balance: bigint;
  transaction_count: bigint;
  archived_transaction_count: bigint;   // NEW: for tracking archived history
};
```

### 3.2 Learning Engine - ContentNode Structure (MAJOR CHANGE)

**OLD**: Used `LearningUnit` with `unit_id`, `chapter_id`, `quiz[]`  
**NEW**: Uses hierarchical `ContentNode` structure

```typescript
// NEW ContentNode type
type ContentNode = {
  id: string;                  // Unique identifier
  parent_id: string | null;    // Parent node (for hierarchy)
  order: number;               // Display order
  display_type: "CHAPTER" | "SECTION" | "UNIT" | "BOOK" | "PART";
  title: string;
  description: string | null;
  content: string | null;      // Educational content (for UNITs)
  paraphrase: string | null;   // Summary
  media: MediaContent | null;  // Video/audio/image
  quiz: QuizData | null;       // Quiz (for UNITs)
  created_at: bigint;
  updated_at: bigint;
  version: bigint;
};

type QuizData = {
  questions: QuizQuestion[];
};

type QuizQuestion = {
  question: string;
  options: string[];
  // NOTE: answer is NOT exposed to frontend (security)
};
```

**NEW Query Methods:**
| Method | Returns | Description |
|--------|---------|-------------|
| `get_root_nodes()` | `Vec<PublicContentNode>` | Get all chapters (root level) |
| `get_children(parent_id)` | `Vec<PublicContentNode>` | Get children of a node |
| `get_content_node(id)` | `Option<PublicContentNode>` | Get single node |
| `get_content_stats()` | `(node_count, quiz_count)` | Statistics |
| `get_global_quiz_config()` | `QuizConfig` | Global quiz reward settings |

**DEPRECATED Methods:**
- `get_learning_units_metadata()` → use `get_root_nodes()` + `get_children()`
- `get_learning_unit(unit_id)` → use `get_content_node(id)`

### 3.3 Quiz Configuration (Updated)

**NEW**: Quiz limits include daily, weekly, monthly, and yearly:

```typescript
type QuizConfig = {
  reward_amount: bigint;           // Tokens per quiz (default: 100 GHC = 100*1e8 e8s)
  pass_threshold_percent: number;  // Min score (default: 60%)
  max_daily_attempts: number;      // Legacy, per-quiz limit
  // NEW time-based limits:
  max_daily_quizzes: number;       // Total quizzes per day (default: 5)
  max_weekly_quizzes: number;      // Total quizzes per week (default: 25)
  max_monthly_quizzes: number;     // Total quizzes per month (default: 70)
  max_yearly_quizzes: number;      // Total quizzes per year (default: 600)
};
```

### 3.4 Quiz Submission

**OLD**: Called `learning_engine.submit_quiz()`  
**NEW**: Call `user_profile.submit_quiz()`

```typescript
// Submit quiz (answers are 0-indexed option numbers)
const result = await userProfileActor.submit_quiz("unit_id_here", [0, 1, 2, 0]);
if ('Ok' in result) {
  console.log(`Earned ${Number(result.Ok) / 1e8} GHC`);
} else {
  // Error may be "Daily quiz limit reached", "Weekly quiz limit reached", etc.
  console.log(`Error: ${result.Err}`);
}
```

### 3.5 Treasury & Governance Split

**Treasury Canister** (token custody):
```typescript
// Get treasury state
const state = await treasuryActor.get_treasury_state();
// Returns: { balance, allowance, total_transferred, mmcr_count, ... }

// Get MMCR status
const mmcr = await treasuryActor.get_mmcr_status();
// Returns: { releases_completed, next_release_amount, seconds_until_next, ... }
```

**Governance Canister** (proposals & voting):
```typescript
// Create treasury proposal
await governanceActor.create_treasury_proposal({
  title: "Marketing Budget Q1",
  description: "Allocate funds for marketing",
  recipient: principalId,
  amount: BigInt(10_000 * 1e8),
  token_type: { GHC: null },
  category: { Marketing: null },
  external_link: []
});

// Vote on proposal
await governanceActor.vote(proposalId, true); // true = YES

// Execute approved proposal
await governanceActor.execute_proposal(proposalId);

// Get governance config (NEW: includes timing parameters)
const config = await governanceActor.get_governance_config();
// Returns: { min_voting_power, support_threshold, approval_percentage,
//            support_period_days, voting_period_days, resubmission_cooldown_days }
```

### 3.6 Content Governance (NEW Feature)

Content proposals allow governance-controlled content updates:

```typescript
// 1. Stage content to staging_assets
const hash = await stagingActor.stage_content(title, description, contentNodes);

// 2. Create content proposal
await governanceActor.create_add_content_proposal({
  title: "Add Climate Chapter",
  description: "New educational content about climate",
  staging_canister: stagingCanisterId,
  staging_path: hash,
  content_hash: hash,
  content_title: "Climate Science",
  unit_count: 5,
  external_link: []
});

// 3. After approval & execution, content is loaded into learning_engine
```

### 3.7 ICO Canister (NEW)

Fixed-price token sales with ckUSDC:

```typescript
// Get ICO stats
const stats = await icoActor.get_ico_stats();
console.log(`Price: ${Number(stats.price_per_token_e6) / 1e6} USDC per GHC`);
console.log(`Total sold: ${Number(stats.total_sold_ghc) / 1e8} GHC`);

// Buy GHC (user must first approve ckUSDC via icrc2_approve)
const result = await icoActor.buy_ghc(BigInt(100 * 1e8)); // Buy 100 GHC
```

---

## 4. Actor Initialization Template

```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";

// Import all IDL factories
import { idlFactory as userProfileIdl } from './declarations/user_profile';
import { idlFactory as stakingHubIdl } from './declarations/staking_hub';
import { idlFactory as learningEngineIdl } from './declarations/learning_engine';
import { idlFactory as governanceIdl } from './declarations/governance_canister';
import { idlFactory as treasuryIdl } from './declarations/treasury_canister';
import { idlFactory as ledgerIdl } from './declarations/ghc_ledger';
import { idlFactory as vestingIdl } from './declarations/founder_vesting';
import { idlFactory as mediaAssetsIdl } from './declarations/media_assets';
import { idlFactory as stagingAssetsIdl } from './declarations/staging_assets';
import { idlFactory as indexIdl } from './declarations/icrc1_index_canister';
import { idlFactory as archiveIdl } from './declarations/archive_canister';
import { idlFactory as icoIdl } from './declarations/ico_canister';

// Canister IDs from ic.config.json
import config from './ic.config.json';

export async function createActors(identity: any) {
  const agent = new HttpAgent({ identity, host: config.host });
  
  // Fetch root key for local development only
  if (process.env.NODE_ENV !== 'production') {
    await agent.fetchRootKey();
  }
  
  return {
    userProfile: Actor.createActor(userProfileIdl, {
      agent,
      canisterId: config.canisters.user_profile,
    }),
    stakingHub: Actor.createActor(stakingHubIdl, {
      agent,
      canisterId: config.canisters.staking_hub,
    }),
    learningEngine: Actor.createActor(learningEngineIdl, {
      agent,
      canisterId: config.canisters.learning_engine,
    }),
    governance: Actor.createActor(governanceIdl, {
      agent,
      canisterId: config.canisters.governance_canister,
    }),
    treasury: Actor.createActor(treasuryIdl, {
      agent,
      canisterId: config.canisters.treasury_canister,
    }),
    ledger: Actor.createActor(ledgerIdl, {
      agent,
      canisterId: config.canisters.ghc_ledger,
    }),
    vesting: Actor.createActor(vestingIdl, {
      agent,
      canisterId: config.canisters.founder_vesting,
    }),
    mediaAssets: Actor.createActor(mediaAssetsIdl, {
      agent,
      canisterId: config.canisters.media_assets,
    }),
    stagingAssets: Actor.createActor(stagingAssetsIdl, {
      agent,
      canisterId: config.canisters.staging_assets,
    }),
    index: Actor.createActor(indexIdl, {
      agent,
      canisterId: config.canisters.icrc1_index_canister,
    }),
    archive: Actor.createActor(archiveIdl, {
      agent,
      canisterId: config.canisters.archive_canister,
    }),
    ico: Actor.createActor(icoIdl, {
      agent,
      canisterId: config.canisters.ico_canister,
    }),
  };
}
```

---

## 5. UI Components to Update

### 5.1 Curriculum/Learning Page
- **OLD**: Fetched flat list with `get_learning_units_metadata()`
- **NEW**: Build hierarchical tree using `get_root_nodes()` → `get_children()`

```typescript
// Fetch curriculum hierarchy
const chapters = await learningEngine.get_root_nodes();
for (const chapter of chapters) {
  const sections = await learningEngine.get_children(chapter.id);
  for (const section of sections) {
    const units = await learningEngine.get_children(section.id);
    // units have quiz data
  }
}
```

### 5.2 Quiz Component
- Answers are submitted as `Vec<u8>` (array of option indices 0-based)
- Quiz always goes through `user_profile.submit_quiz()`
- Default reward is 100 GHC (10,000,000,000 e8s)
- **NEW**: Check time-based limits before allowing quiz:

```typescript
const stats = await userProfile.get_user_stats(userPrincipal);
const config = await learningEngine.get_global_quiz_config();

const canTakeQuiz = 
  stats.daily_quizzes < config.max_daily_quizzes &&
  stats.weekly_quizzes < config.max_weekly_quizzes &&
  stats.monthly_quizzes < config.max_monthly_quizzes &&
  stats.yearly_quizzes < config.max_yearly_quizzes;
```

### 5.3 User Profile/Dashboard
- **NEW**: Display verification tier (None/Human/KYC)
- **NEW**: Show quiz limits remaining

```typescript
const profile = await userProfile.get_profile(principal);
const stats = await userProfile.get_user_stats(principal);

console.log(`Verification: ${Object.keys(profile.verification_tier)[0]}`);
console.log(`Daily quizzes: ${stats.daily_quizzes}/5`);
console.log(`Weekly quizzes: ${stats.weekly_quizzes}/25`);
```

### 5.4 Governance Section
- **Separate actors** for treasury queries vs governance actions
- **NEW**: `get_governance_config()` now returns configurable timing parameters
- New proposal types: `AddContentFromStaging`, `UpdateGlobalQuizConfig`, `DeleteContentNode`, `UpdateGovernanceConfig`

### 5.5 Treasury Dashboard
- Use `treasury_canister` for state/balance/MMCR queries
- Use `governance_canister` for proposal creation/voting

### 5.6 ICO/Buy Page (NEW)
- Integrate MoonPay widget for ckUSDC purchases
- Display ICO stats and pricing
- Handle ICRC-2 approval flow before purchase

```typescript
// 1. User approves ICO canister to spend their ckUSDC
await ckusdcLedger.icrc2_approve({
  spender: { owner: icoCanisterId, subaccount: [] },
  amount: BigInt(cost * 1e6),
  // ...
});

// 2. User buys GHC
await icoCanister.buy_ghc(BigInt(amountGhc * 1e8));
```

### 5.7 Transaction History (Updated)
- Combine local and archived transactions for complete history:

```typescript
async function getFullHistory(user: Principal) {
  const local = await userProfile.get_user_transactions(user);
  const archived = await archive.get_user_transactions(user, 0n, 1000);
  return [...archived, ...local].sort((a, b) => 
    Number(a.timestamp - b.timestamp)
  );
}
```

---

## 6. Migration Checklist

- [ ] Remove imports for `operational_governance`
- [ ] Add imports for `governance_canister` and `treasury_canister`
- [ ] Add imports for `archive_canister`, `ico_canister` (if using those features)
- [ ] Update canister ID references from `ic.config.json`
- [ ] Replace `get_learning_units_metadata()` with `get_root_nodes()`
- [ ] Update curriculum rendering to handle hierarchical structure
- [ ] Test quiz submission flow with new reward amount (100 GHC)
- [ ] Add quiz limit checking before submission attempts
- [ ] Update treasury dashboard to use separate treasury actor
- [ ] Add new proposal type handling (content proposals, governance config)
- [ ] Update user profile display to show verification tier
- [ ] (Optional) Implement ICO purchase flow with MoonPay
- [ ] Update TypeScript types from new declarations

---

## 7. Testing Commands

```bash
# Check current canister IDs
dfx canister id governance_canister
dfx canister id treasury_canister
dfx canister id archive_canister
dfx canister id ico_canister

# Test learning engine
dfx canister call learning_engine get_root_nodes
dfx canister call learning_engine get_children '("chapter_1")'
dfx canister call learning_engine get_global_quiz_config

# Test quiz submission
dfx canister call user_profile submit_quiz '("unit_id", vec { 0; 1; 2 })'

# Test user stats (includes quiz limits)
dfx canister call user_profile get_user_stats '(principal "your-principal-here")'

# Test treasury
dfx canister call treasury_canister get_treasury_state
dfx canister call treasury_canister get_mmcr_status

# Test governance
dfx canister call governance_canister get_active_proposals
dfx canister call governance_canister get_my_voting_power
dfx canister call governance_canister get_governance_config

# Test ICO
dfx canister call ico_canister get_ico_stats

# Test archive
dfx canister call archive_canister get_archive_stats
```

---

## 8. Files Provided

The following files have been copied to your project:

```
src/declarations/           # TypeScript declarations for all 14 canisters
ic.config.json              # Canister IDs and network config
docs/FRONTEND_INTEGRATION.md  # Full API documentation
docs/TESTING_GUIDE.md       # Testing procedures
docs/CONTENT_GOVERNANCE.md  # Content governance flow
docs/GOVERNANCE_ARCHITECTURE.md  # Governance system details
docs/QUIZ_CACHING_ARCHITECTURE.md  # Quiz caching and limits
```

---

## Questions?

The smart contract team is available to clarify any API details. Key documentation:
- `FRONTEND_INTEGRATION.md` - Complete API reference
- Candid `.did` files in each canister's `src/` directory
- TypeScript declarations in `src/declarations/`
