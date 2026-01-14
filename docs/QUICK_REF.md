# Quick Reference

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

## Token Distribution (9.5B Total)
- **MUC (Utility Coins)**: 4.75B (staking_hub) - Mining rewards
- **MC (Market Coins)**: 4.75B
  - Treasury: 4.25B (operational_governance)
  - Founder 1: 0.35B (founder_vesting)
  - Founder 2: 0.15B (founder_vesting)

## Governance Parameters
- **Min voting power to propose**: 150 tokens
- **Approval threshold**: 15,000 YES votes
- **Voting period**: 14 days
- **Resubmission cooldown**: 6 months

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

### Board Members (in operational_governance)
```bash
# Get all board members with their shares
dfx canister call operational_governance get_board_member_shares '()'

# Check if user is a board member
dfx canister call operational_governance is_board_member "(principal \"$(dfx identity get-principal)\")"

# Get board member count
dfx canister call operational_governance get_board_member_count '()'

# Check if shares are locked
dfx canister call operational_governance are_board_shares_locked '()'

# Set board member shares (admin only, before lock)
dfx canister call operational_governance set_board_member_shares '(vec { 
  record { member = principal "xxx-xxx-xxx"; percentage = 60 : nat8 };
  record { member = principal "yyy-yyy-yyy"; percentage = 30 : nat8 };
  record { member = principal "zzz-zzz-zzz"; percentage = 10 : nat8 };
})'

# Lock board member shares (use governance proposals after this)
dfx canister call operational_governance lock_board_member_shares '()'
```

### Proposals
```bash
# Get governance config
dfx canister call operational_governance get_governance_config '()'

# Create a treasury proposal (spending from treasury)
dfx canister call operational_governance create_treasury_proposal '(record {
  title = "Marketing Q1";
  description = "Fund marketing initiatives";
  recipient = principal "xxx-xxx-xxx";
  amount = 1000000000000 : nat64;
  token_type = variant { GHC };
  category = variant { Marketing };
  external_link = null
})'

# Create a board member proposal (add new board member)
dfx canister call operational_governance create_board_member_proposal '(record {
  title = "Add New Board Member";
  description = "Proposing to add Alice as a board member with 15% share";
  new_member = principal "xxx-xxx-xxx";
  percentage = 15 : nat8;
  external_link = null
})'

# Vote on proposal (YES)
dfx canister call operational_governance vote '(0 : nat64, true)'

# Support a proposal (for non-board members in Proposed state)
dfx canister call operational_governance support_proposal '(0 : nat64)'

# Get proposal
dfx canister call operational_governance get_proposal '(0)'

# Get all active proposals
dfx canister call operational_governance get_active_proposals '()'

# Get all proposals
dfx canister call operational_governance get_all_proposals '()'

# See who voted
dfx canister call operational_governance get_proposal_votes '(0)'

# Execute approved proposal
dfx canister call operational_governance execute_proposal '(0 : nat64)'
```

### Treasury
```bash
# Check Treasury State
dfx canister call operational_governance get_treasury_state '()'

# Check MMCR Status
dfx canister call operational_governance get_mmcr_status '()'

# Check Spendable Balance
dfx canister call operational_governance get_spendable_balance '()'
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
