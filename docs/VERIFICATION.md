# Verification Plan

This document outlines how to verify the GreenHero Coin (GHC) dapp.

## 1. Automated Verification (Script)
We have provided a script `test_flow.sh` that automates the core user flows:
1.  **Mining**: Submitting a quiz and earning rewards.
2.  **Staking**: Claiming rewards and verifying they are staked.
3.  **Governance**: Checking voting power.
4.  **Unstaking**: Unstaking tokens and verifying the 10% penalty deduction.

### Running the Test Script
```bash
chmod +x test_flow.sh
./test_flow.sh
```

## 2. Manual Verification Steps

If you prefer to run steps manually, follow this guide.

### Prerequisites
- Local replica running: `dfx start --background --clean`
- Canisters deployed: `./deploy.sh`

### Scenario A: Mining & Staking
1.  **Check Initial Balance**:
    ```bash
    dfx canister call staking_hub get_user_stats "(principal \"$(dfx identity get-principal)\")"
    ```
    *Expected Output*: `(0 : nat64, 0 : nat64)`

2.  **Submit Quiz**:
    ```bash
    dfx canister call learning_engine submit_quiz '("1.0", vec {0})'
    ```
    *Expected Output*: `(Ok (500000000 : nat64))` (5 GHC)

3.  **Verify Virtual Balance (Auto-Staked)**:
    ```bash
    dfx canister call staking_hub get_user_stats "(principal \"$(dfx identity get-principal)\")"
    ```
    *Expected Output*: `(500000000 : nat64, 0 : nat64)`
    *Note*: The balance is updated immediately without a manual claim step.

### Scenario B: Unstaking
1.  **Unstake 1 GHC** (100,000,000 e8s):
    ```bash
    dfx canister call staking_hub unstake '(100000000)'
    ```
    *Expected Output*: `(Ok (90000000 : nat64))`
    *Note*: You receive 0.9 GHC (90,000,000 e8s). 0.1 GHC is the 10% penalty.

2.  **Verify Wallet Balance**:
    ```bash
    dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$(dfx identity get-principal)\"; subaccount = null })"
    ```
    *Expected Output*: `(90000000 : nat)`

## 3. Code Verification
- **Compilation**: Run `cargo build --target wasm32-unknown-unknown --release` to verify the Rust code builds without errors.
- **Linting**: Run `cargo clippy` to check for code quality issues.
