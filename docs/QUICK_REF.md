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

# Check if user is founder
dfx canister call staking_hub is_founder "(principal \"$(dfx identity get-principal)\")"

# Get voting power
dfx canister call staking_hub fetch_voting_power "(principal \"$(dfx identity get-principal)\")"

# List founders
dfx canister call staking_hub get_founders '()'

# Add founder (admin only)
dfx canister call staking_hub add_founder '(principal "xxx-xxx-xxx")'
```

### Proposals
```bash
# Get governance config
dfx canister call operational_governance get_governance_config '()'

# Create proposal
dfx canister call operational_governance create_proposal '(record {
  title = "Marketing Q1";
  description = "Fund marketing initiatives";
  recipient = principal "xxx-xxx-xxx";
  amount = 1000000000000 : nat64;
  token_type = variant { GHC };
  category = variant { Marketing };
  external_link = null
})'

# Vote on proposal (YES)
dfx canister call operational_governance vote '(0 : nat64, true)'

# Get proposal
dfx canister call operational_governance get_proposal '(0)'

# Get all active proposals
dfx canister call operational_governance get_active_proposals '()'

# See who voted
dfx canister call operational_governance get_proposal_votes '(0)'
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

# Get VUC (founder voting power)
dfx canister call staking_hub get_vuc '()'
```
