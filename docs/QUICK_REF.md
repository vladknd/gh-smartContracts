# Quick Reference

**Last Updated**: January 2026

## Prerequisites
- `dfx` (DFINITY SDK)
- `rust` (cargo)

## Build & Deploy
```bash
# Start local replica
dfx start --background --clean

# Deploy everything
./scripts/deploy.sh
```

## Canister Architecture (14 Canisters)
| Canister | Purpose |
|----------|---------|
| `governance_canister` | Proposals, voting, board management |
| `treasury_canister` | Token custody, transfers, MMCR |
| `learning_engine` | Content storage, quiz management |
| `media_assets` | Permanent media file storage |
| `staging_assets` | Temporary content before approval |
| `staking_hub` | Token staking, VUC management, shard coordination |
| `user_profile` | User data, quiz submissions, verification tiers |
| `founder_vesting` | Time-locked founder tokens |
| `ghc_ledger` | ICRC-1 token ledger |
| `icrc1_index_canister` | Transaction history indexing |
| `archive_canister` | Long-term transaction archival |
| `ico_canister` | Fixed-price token sales (ckUSDC) |
| `sonic_adapter` | DEX integration (Sonic) |
| `internet_identity` | User authentication |

## Token Distribution (9.5B Total)
- **MUC (Utility Coins)**: 4.75B (staking_hub) - Mining rewards
- **MC (Market Coins)**: 4.75B
  - Treasury: 4.25B (treasury_canister)
  - Founder 1: 0.35B (founder_vesting)
  - Founder 2: 0.15B (founder_vesting)

## Governance Parameters (Configurable via Proposals)
| Parameter | Default | Configurable |
|-----------|---------|--------------|
| Min voting power to propose | 150 tokens | ✅ |
| Support threshold | 15,000 tokens | ✅ |
| Approval percentage | 30% | ✅ |
| Support period | 7 days | ✅ |
| Voting period | 14 days | ✅ |
| Resubmission cooldown | 180 days | ✅ |

## Quiz Limits (Configurable via Proposals)
| Limit | Default |
|-------|---------|
| Max daily quizzes | 5 |
| Max weekly quizzes | 25 |
| Max monthly quizzes | 70 |
| Max yearly quizzes | 600 |

---

## Common Commands

### User Registration & Quiz
```bash
# Register user
dfx canister call user_profile register_user '(record { email = "test@example.com"; name = "Test"; education = "Test"; gender = "Test" })'

# Submit a quiz (Mine tokens)
dfx canister call user_profile submit_quiz '("unit_id", vec {0})'

# Check User Profile (Staked Balance)
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"

# Unstake (100% returned, no penalty)
dfx canister call user_profile unstake '(100000000)'
```

### Governance
```bash
# Get tokenomics (max_supply, allocated, vuc, total_power)
dfx canister call staking_hub get_tokenomics '()'

# Get VUC (board member voting power pool)
dfx canister call staking_hub get_vuc '()'

# Get user voting power (from their staked balance)
dfx canister call staking_hub fetch_user_voting_power "(principal \"$(dfx identity get-principal)\")"
```

### Board Members
```bash
# Get all board members with their shares
dfx canister call governance_canister get_board_member_shares '()'

# Check if user is a board member
dfx canister call governance_canister is_board_member "(principal \"$(dfx identity get-principal)\")"

# Get board member count
dfx canister call governance_canister get_board_member_count '()'

# Check if shares are locked
dfx canister call governance_canister are_board_shares_locked '()'

# Set board member shares (admin only, before lock)
dfx canister call governance_canister set_board_member_shares '(vec { 
  record { member = principal "xxx-xxx-xxx"; percentage = 60 : nat8 };
  record { member = principal "yyy-yyy-yyy"; percentage = 30 : nat8 };
  record { member = principal "zzz-zzz-zzz"; percentage = 10 : nat8 };
})'

# Lock board member shares (use governance proposals after this)
dfx canister call governance_canister lock_board_member_shares '()'
```

### Proposals
```bash
# Get governance config
dfx canister call governance_canister get_governance_config '()'

# Create a treasury proposal (spending from treasury)
dfx canister call governance_canister create_treasury_proposal '(record {
  title = "Marketing Q1";
  description = "Fund marketing initiatives";
  recipient = principal "xxx-xxx-xxx";
  amount = 1000000000000 : nat64;
  token_type = variant { GHC };
  category = variant { Marketing };
  external_link = null
})'

# Create a board member proposal (add new board member)
dfx canister call governance_canister create_board_member_proposal '(record {
  title = "Add New Board Member";
  description = "Proposing to add Alice as a board member with 15% share";
  new_member = principal "xxx-xxx-xxx";
  percentage = 15 : nat8;
  external_link = null
})'

# Vote on proposal (YES)
dfx canister call governance_canister vote '(0 : nat64, true)'

# Support a proposal (for non-board members in Proposed state)
dfx canister call governance_canister support_proposal '(0 : nat64)'

# Get proposal
dfx canister call governance_canister get_proposal '(0)'

# Get all active proposals
dfx canister call governance_canister get_active_proposals '()'

# Get all proposals
dfx canister call governance_canister get_all_proposals '()'

# See who voted
dfx canister call governance_canister get_proposal_votes '(0)'

# Execute approved proposal
dfx canister call governance_canister execute_proposal '(0 : nat64)'
```

### Treasury
```bash
# Check Treasury State
dfx canister call treasury_canister get_treasury_state '()'

# Check MMCR Status
dfx canister call treasury_canister get_mmcr_status '()'

# Check Spendable Balance
dfx canister call treasury_canister get_spendable_balance '()'
```

### Content Governance
```bash
# Stage content for approval
dfx canister call staging_assets stage_content '("Title", "Description", vec {...})'

# Create content proposal
dfx canister call governance_canister create_add_content_proposal '(record {
  title = "Add Environmental Course";
  description = "New course content";
  staging_canister = principal "STAGING_ID";
  content_hash = "abc123...";
  content_title = "Env Science 101";
  unit_count = 50 : nat32;
  external_link = null
})'

# Check content loading status
dfx canister call learning_engine get_loading_status '(0 : nat64)'

# Get content node
dfx canister call learning_engine get_content_node '("book:1:ch:1")'
```

### Founder Vesting
```bash
# Check Founder Vesting
dfx canister call founder_vesting get_all_vesting_schedules '()'

# Claim vested tokens (founder only)
dfx canister call founder_vesting claim_vested '()'
```

### Global Stats
```bash
# Check Global Stats
dfx canister call staking_hub get_global_stats '()'

# Get VUC (board member voting power pool)
dfx canister call staking_hub get_vuc '()'

# Get total voting power in system
dfx canister call staking_hub get_total_voting_power '()'
```

### ICO Canister
```bash
# Get ICO stats
dfx canister call ico_canister get_ico_stats '()'

# Buy GHC (after approving ckUSDC)
# First approve ICO canister to spend your ckUSDC, then:
dfx canister call ico_canister buy_ghc '(10000000000 : nat)'  # Buy 100 GHC

# Admin: End sale and sweep funds to treasury
dfx canister call ico_canister end_sale '()'
```

### Archive Canister
```bash
# Get archive stats
dfx canister call archive_canister get_archive_stats '()'

# Get archived transactions for a user
dfx canister call archive_canister get_user_transactions "(principal \"$(dfx identity get-principal)\", 0 : nat64, 100 : nat32)"
```

### Quiz Configuration
```bash
# Get global quiz config
dfx canister call learning_engine get_global_quiz_config '()'

# Get user's quiz stats (includes daily/weekly/monthly/yearly counts)
dfx canister call user_profile get_user_stats "(principal \"$(dfx identity get-principal)\")"
```
