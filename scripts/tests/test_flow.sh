#!/bin/bash

# Basic Flow Test - Tests the simplified tokenomics flow
# User registers -> Submits quiz -> Gets staked balance -> Unstakes (100% return)

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[FLOW] $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

fail() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

log "=== Starting GHC Basic Flow Test ==="

# Generate a unique identity for this test run
TEST_IDENTITY="test_user_$(date +%s)"
log "Creating temporary identity: $TEST_IDENTITY"

# Create new identity (plaintext to avoid password prompt)
dfx identity new "$TEST_IDENTITY" --storage-mode plaintext 2>/dev/null || true
dfx identity use "$TEST_IDENTITY"

USER_PRINCIPAL=$(dfx identity get-principal)
log "Testing as User: $USER_PRINCIPAL"

# Cleanup function to run on exit
cleanup() {
    echo -e "\n=== Cleaning up identity... ==="
    dfx identity use default
    dfx identity remove "$TEST_IDENTITY" 2>/dev/null || true
}
trap cleanup EXIT

# 1. Register User
log "[Step 1] Registering User..."
REG=$(dfx canister call user_profile register_user '(record { 
    email = "test@example.com"; 
    name = "Test User"; 
    education = "PhD"; 
    gender = "Non-binary" 
})' 2>&1)

if [[ $REG == *"Ok"* ]]; then
    success "User registered"
else
    fail "Registration failed: $REG"
fi

# 2. Submit Quiz (as admin first to add unit)
log "[Step 2] Adding learning unit (if not exists)..."
dfx identity use default
ADD=$(dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "flow_test_unit";
    unit_title = "Flow Test Unit";
    chapter_id = "1";
    chapter_title = "Test Chapter";
    head_unit_id = "1";
    head_unit_title = "Test Head Unit";
    content = "Content";
    paraphrase = "Paraphrase";
    quiz = vec { record { question = "Q?"; options = vec {"A"}; answer = 0 } };
})' 2>&1)
dfx identity use "$TEST_IDENTITY"

if [[ $ADD == *"Ok"* ]] || [[ $ADD == *"exists"* ]]; then
    success "Learning unit ready"
else
    fail "Failed to add unit: $ADD"
fi

# 3. Submit Quiz
log "[Step 3] Submitting Quiz..."
RESULT=$(dfx canister call user_profile submit_quiz '("flow_test_unit", vec {0})' 2>&1)

if [[ $RESULT == *"Ok"* ]]; then
    success "Quiz submitted successfully"
else
    fail "Quiz submission failed: $RESULT"
fi

# 4. Check Staked Balance
log "[Step 4] Checking Staked Balance..."
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")")

if [[ $PROFILE == *"staked_balance = 100_000_000"* ]] || [[ $PROFILE == *"staked_balance = 100000000"* ]]; then
    success "Staked balance correct (1 GHC = 100,000,000 e8s)"
else
    fail "Staked balance incorrect: $PROFILE"
fi

# 5. Unstake (No Penalty - 100% return)
log "[Step 5] Unstaking 0.5 GHC (100% return, no penalty)..."
UNSTAKE=$(dfx canister call user_profile unstake '(50_000_000)' 2>&1)

if [[ $UNSTAKE == *"Ok"* ]]; then
    success "Unstake successful (100% returned)"
else
    fail "Unstake failed: $UNSTAKE"
fi

# 6. Check Wallet Balance
log "[Step 6] Checking Wallet Balance..."
BALANCE=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })")

if [[ $BALANCE == *"50_000_000"* ]] || [[ $BALANCE == *"50000000"* ]]; then
    success "Wallet balance correct (0.5 GHC received)"
else
    fail "Wallet balance incorrect: $BALANCE"
fi

# 7. Check Remaining Staked Balance
log "[Step 7] Checking Remaining Staked Balance..."
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")")

if [[ $PROFILE == *"staked_balance = 50_000_000"* ]] || [[ $PROFILE == *"staked_balance = 50000000"* ]]; then
    success "Remaining staked balance correct (0.5 GHC)"
else
    fail "Remaining staked balance incorrect: $PROFILE"
fi

log "=== Flow Test Complete: ALL TESTS PASSED ==="
