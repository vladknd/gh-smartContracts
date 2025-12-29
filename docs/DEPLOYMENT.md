# Deployment Guide

## Local Deployment
1. Ensure `dfx` is running: `dfx start --background --clean`
2. Run the deployment script: `./scripts/deploy.sh`

## Mainnet Deployment
1. Remove `--clean` from start command or use a running mainnet connection.
2. Update `deploy.sh` to use `--network ic`.
3. Ensure you have cycles in your wallet.
4. Run `./scripts/deploy.sh`.

## Post-Deployment Verification

### 1. Check Staking Hub Balance (4.75B MUC)
```bash
dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$(dfx canister id staking_hub)\"; subaccount = null })"
```
Should return 4.75 Billion tokens (Mined Utility Coins partition).

### 2. Check Treasury Balance (4.25B MC)
```bash
dfx canister call operational_governance get_treasury_state
```
Should return balance = 4.25B, allowance = 0.6B initially.

### 3. Check Founder Vesting (0.5B MC)
```bash
dfx canister call founder_vesting get_all_vesting_schedules
```
Should return two schedules: Founder 1 (0.35B) and Founder 2 (0.15B).

### 4. Check Staking Hub Global Stats
```bash
dfx canister call staking_hub get_global_stats
```
Should return `total_staked = 0`, `total_unstaked = 0`, `total_allocated = 0`.

### 5. Check MMCR Status
```bash
dfx canister call operational_governance get_mmcr_status
```
Should return `releases_completed = 0`, `next_release_time`, etc.
