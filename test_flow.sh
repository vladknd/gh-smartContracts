#!/bin/bash

# Stop on error
set -e

echo "=== Starting GHC Dapp Verification Flow ==="

# Generate a unique identity for this test run
TEST_IDENTITY="test_user_$(date +%s)"
echo "Creating temporary identity: $TEST_IDENTITY"

# Create new identity (plaintext to avoid password prompt)
dfx identity new "$TEST_IDENTITY" --storage-mode plaintext || true
dfx identity use "$TEST_IDENTITY"

USER_PRINCIPAL=$(dfx identity get-principal)
echo "Testing as User: $USER_PRINCIPAL"

# Cleanup function to run on exit
cleanup() {
    echo -e "\n=== Cleaning up identity... ==="
    dfx identity use default
    dfx identity remove "$TEST_IDENTITY"
}
trap cleanup EXIT

# 1. Register & Mining
echo -e "\n[Step 1] Registering & Submitting Quiz..."

# Register first
dfx canister call user_profile register_user '(record { email = "test@example.com"; name = "Test User"; education = "PhD"; gender = "Non-binary" })'

# Quiz ID "unit_1", Answer [0] (Correct) - Calling user_profile, not learning_engine
RESULT=$(dfx canister call user_profile submit_quiz '("unit_1", vec {0})')
echo "Result: $RESULT"
if [[ $RESULT == *"Ok"* ]]; then
    echo "✅ Quiz submitted successfully"
else
    echo "❌ Quiz submission failed"
    exit 1
fi

# 2. Check Pending
echo -e "\n[Step 2] Checking Pending Rewards..."
PENDING=$(dfx canister call staking_hub get_pending_reward "(principal \"$USER_PRINCIPAL\")")
echo "Pending: $PENDING"
if [[ $PENDING == *"(500_000_000 : nat64)"* ]] || [[ $PENDING == *"(500000000 : nat64)"* ]]; then
    echo "✅ Pending reward is correct (5 GHC)"
else
    echo "❌ Pending reward incorrect"
    exit 1
fi

# 3. Claim & Stake
echo -e "\n[Step 3] Claiming & Staking..."
CLAIM=$(dfx canister call staking_hub claim_and_stake)
echo "Claim Result: $CLAIM"
if [[ $CLAIM == *"Ok"* ]]; then
    echo "✅ Claim successful"
else
    echo "❌ Claim failed"
    exit 1
fi

# 4. Verify Staked Balance
echo -e "\n[Step 4] Verifying Staked Balance..."
STAKED=$(dfx canister call staking_hub get_user_staked_balance "(principal \"$USER_PRINCIPAL\")")
echo "Staked: $STAKED"
if [[ $STAKED == *"(500000000 : nat64)"* ]] || [[ $STAKED == *"(500_000_000 : nat64)"* ]]; then
    echo "✅ Staked balance correct"
else
    echo "❌ Staked balance incorrect"
    exit 1
fi

# 5. Unstake
echo -e "\n[Step 5] Unstaking 1 GHC..."
# Unstake 100,000,000 e8s
UNSTAKE=$(dfx canister call staking_hub unstake '(100000000)')
echo "Unstake Result: $UNSTAKE"
# Expected return: 90,000,000 (90%)
if [[ $UNSTAKE == *"90000000"* ]] || [[ $UNSTAKE == *"90_000_000"* ]]; then
    echo "✅ Unstake successful (Received 90%)"
else
    echo "❌ Unstake failed or incorrect amount"
    exit 1
fi

# 6. Check Wallet Balance
echo -e "\n[Step 6] Checking Wallet Balance..."
BALANCE=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })")
echo "Wallet Balance: $BALANCE"
if [[ $BALANCE == *"(90000000 : nat)"* ]] || [[ $BALANCE == *"(90_000_000 : nat)"* ]]; then
    echo "✅ Wallet balance correct"
else
    echo "❌ Wallet balance incorrect"
    exit 1
fi

echo -e "\n=== Verification Complete: ALL TESTS PASSED ==="
