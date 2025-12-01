# Deployment Guide

## Local Deployment
1. Ensure `dfx` is running: `dfx start --background --clean`
2. Run the deployment script: `./deploy.sh`

## Mainnet Deployment
1. Remove `--clean` from start command or use a running mainnet connection.
2. Update `deploy.sh` to use `--network ic`.
3. Ensure you have cycles in your wallet.
4. Run `./deploy.sh`.

## Post-Deployment Verification
1. Check Ledger balances:
   ```bash
   dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$(dfx canister id staking_hub)\"; subaccount = null })"
   ```
   Should return 4.1 Billion tokens (The entire Utility Partition).

2. Check Treasury balance:
   ```bash
   dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$(dfx canister id operational_governance)\"; subaccount = null })"
   ```
   Should return 3.6 Billion tokens.

3. Check Staking Hub Internal Stats:
   ```bash
   dfx canister call staking_hub get_global_stats
   ```
   Should return `total_staked = 0`, `interest_pool = 0`.
