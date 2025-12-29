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

## Testing
```bash
# Get canister IDs
dfx canister id staking_hub
dfx canister id user_profile
dfx canister id operational_governance
dfx canister id founder_vesting

# Register user
dfx canister call user_profile register_user '(record { email = "test@example.com"; name = "Test"; education = "Test"; gender = "Test" })'

# Submit a quiz (Mine tokens)
dfx canister call user_profile submit_quiz '("unit_id", vec {0})'

# Check User Profile (Staked Balance)
dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")"

# Check Global Stats
dfx canister call staking_hub get_global_stats

# Unstake (100% returned, no penalty)
dfx canister call user_profile unstake '(100000000)'

# Check Treasury State
dfx canister call operational_governance get_treasury_state

# Check MMCR Status
dfx canister call operational_governance get_mmcr_status

# Check Spendable Balance
dfx canister call operational_governance get_spendable_balance

# Check Founder Vesting
dfx canister call founder_vesting get_all_vesting_schedules
```

## Token Distribution (9.5B Total)
- **MUC (Mined Utility Coins)**: 4.75B (staking_hub)
- **MC (Market Coins)**: 4.75B
  - Treasury: 4.25B (operational_governance)
  - Founder 1: 0.35B (founder_vesting)
  - Founder 2: 0.15B (founder_vesting)
