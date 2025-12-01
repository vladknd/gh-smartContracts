# Quick Reference

## Prerequisites
- `dfx` (DFINITY SDK)
- `rust` (cargo)

## Build & Deploy
```bash
# Start local replica
dfx start --background --clean

# Deploy everything
./deploy.sh
```

## Testing
```bash
# Get canister IDs
dfx canister id learning_engine
dfx canister id staking_hub

# Submit a quiz (Mine tokens)
dfx canister call learning_engine submit_quiz '("1.0", vec {0})'

# Check Virtual Balance (Auto-Staked)
dfx canister call staking_hub get_user_stats "(principal \"$(dfx identity get-principal)\")"

# Vote on proposal
dfx canister call operational_governance vote '(1, true)'

# Unstake (10% Penalty)
dfx canister call staking_hub unstake '(100000000)'

# Check Interest Pool
dfx canister call staking_hub get_global_stats

# Distribute Interest (Admin)
dfx canister call staking_hub distribute_interest
```
