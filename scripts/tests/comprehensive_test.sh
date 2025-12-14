#!/bin/bash

# Stop on error
set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[TEST] $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

fail() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

log "=== Starting GHC Dapp Comprehensive Test Suite ==="

# --- 1. Setup & Admin Phase ---

log "Setting up environment..."

# Use default identity for admin tasks
dfx identity use default
ADMIN_PRINCIPAL=$(dfx identity get-principal)
log "Admin Principal: $ADMIN_PRINCIPAL"

# Get Canister IDs
STAKING_HUB_ID=$(dfx canister id staking_hub)
GOVERNANCE_ID=$(dfx canister id operational_governance)
LEDGER_ID=$(dfx canister id ghc_ledger)

log "Staking Hub: $STAKING_HUB_ID"
log "Governance: $GOVERNANCE_ID"
log "Ledger: $LEDGER_ID"

# Fund Canisters (Ensure they have GHC to pay out)
# 100 GHC = 10,000,000,000 e8s
FUND_AMOUNT="10000000000"

log "Funding Staking Hub..."
dfx canister call ghc_ledger icrc1_transfer "(record { 
    to = record { owner = principal \"$STAKING_HUB_ID\"; subaccount = null }; 
    amount = $FUND_AMOUNT; 
})" > /dev/null
success "Staking Hub Funded"

log "Funding Operational Governance..."
dfx canister call ghc_ledger icrc1_transfer "(record { 
    to = record { owner = principal \"$GOVERNANCE_ID\"; subaccount = null }; 
    amount = $FUND_AMOUNT; 
})" > /dev/null
success "Operational Governance Funded"

# --- 2. User Phase: Setup ---

# Generate unique identity
TEST_USER="ghc_test_user_$(date +%s)"
log "Creating temporary test user: $TEST_USER"

dfx identity new "$TEST_USER" --storage-mode plaintext || true
dfx identity use "$TEST_USER"
USER_PRINCIPAL=$(dfx identity get-principal)
log "Test User Principal: $USER_PRINCIPAL"

# Cleanup trap
cleanup() {
    echo -e "\n${BLUE}=== Cleaning up... ===${NC}"
    dfx identity use default
    dfx identity remove "$TEST_USER"
    success "Identity removed"
}
trap cleanup EXIT

# --- 3. Learning & Earning (Virtual Staking) ---

log "--- Phase 1: Learning & Earning (Virtual Staking) ---"

log "Submitting Quiz (Mining)..."
# Quiz ID 1, Answer "A" (Correct) - Assuming unit 1 exists or using a mock call if needed.
# Note: In a real run, we need to ensure unit 1 exists. 
# For this test, we assume the previous setup or a fresh deploy has unit 1.
# If not, we might fail here. 
# Let's add a unit just in case (using admin identity temporarily)
dfx identity use default
dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "1.0";
    unit_title = "Test Unit";
    chapter_id = "1";
    chapter_title = "Test Chapter";
    head_unit_id = "1";
    head_unit_title = "Test Head Unit";
    content = "Content";
    paraphrase = "Paraphrase";
    quiz = vec { record { question = "Q"; options = vec {"A"}; answer = 0 } };
})' > /dev/null
dfx identity use "$TEST_USER"

RESULT=$(dfx canister call learning_engine submit_quiz '("1.0", vec {0})')
if [[ $RESULT == *"Ok"* ]]; then
    success "Quiz submitted"
else
    fail "Quiz submission failed: $RESULT"
fi

log "Verifying Virtual Balance (Auto-Staked)..."
# get_user_stats returns (balance, pending_rewards)
STATS=$(dfx canister call staking_hub get_user_stats "(principal \"$USER_PRINCIPAL\")")
# Expected: (100_000_000 : nat64, 0 : nat64)
if [[ $STATS == *"100_000_000"* ]] || [[ $STATS == *"100000000"* ]]; then
    success "Virtual Balance correct (1 GHC)"
else
    fail "Incorrect Virtual Balance: $STATS"
fi

log "Verifying Voting Power..."
POWER=$(dfx canister call staking_hub get_voting_power "(principal \"$USER_PRINCIPAL\")")
if [[ $POWER == *"100_000_000"* ]] || [[ $POWER == *"100000000"* ]]; then
    success "Voting power correct (1 GHC)"
else
    fail "Incorrect voting power: $POWER"
fi

# --- 4. Governance ---

log "--- Phase 2: Governance ---"

log "Creating Proposal (Grant 1 GHC to self)..."
# Propose to send 1 GHC (100,000,000) to USER_PRINCIPAL
PROPOSAL_ID=$(dfx canister call operational_governance create_proposal "(principal \"$USER_PRINCIPAL\", 100000000, \"Grant for being awesome\")")
# Extract ID from Result: (Ok = 1 : nat64) -> 1
PROP_ID_NUM=$(echo "$PROPOSAL_ID" | grep -oE '[0-9]+' | head -1)
log "Proposal Created with ID: $PROP_ID_NUM"

if [[ -z "$PROP_ID_NUM" ]]; then
    fail "Failed to create proposal: $PROPOSAL_ID"
fi

log "Voting 'Yes'..."
VOTE=$(dfx canister call operational_governance vote "($PROP_ID_NUM, true)")
if [[ $VOTE == *"Ok"* ]]; then
    success "Vote cast successfully"
else
    fail "Voting failed: $VOTE"
fi

log "Executing Proposal..."
EXEC=$(dfx canister call operational_governance execute_proposal "($PROP_ID_NUM)")
if [[ $EXEC == *"Ok"* ]]; then
    success "Proposal executed successfully"
else
    fail "Execution failed: $EXEC"
fi

log "Verifying Grant Receipt..."
BALANCE=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })")
# Should be 1 GHC (100,000,000)
if [[ $BALANCE == *"100_000_000"* ]] || [[ $BALANCE == *"100000000"* ]]; then
    success "Grant received (1 GHC)"
else
    fail "Grant not received. Balance: $BALANCE"
fi

# --- 5. Unstaking & Interest ---

log "--- Phase 3: Unstaking & Interest ---"

log "Unstaking 1 GHC..."
# Unstake 1 GHC (100,000,000). 10% Penalty = 10,000,000. Return = 90,000,000.
UNSTAKE=$(dfx canister call staking_hub unstake '(100000000)')
if [[ $UNSTAKE == *"Ok"* ]]; then
    success "Unstake successful"
else
    fail "Unstake failed: $UNSTAKE"
fi

log "Verifying Final Wallet Balance..."
# Expected: 1 GHC (Grant) + 0.9 GHC (Unstake return) = 1.9 GHC (190,000,000)
FINAL_BALANCE=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })")
log "Final Balance: $FINAL_BALANCE"

if [[ $FINAL_BALANCE == *"190_000_000"* ]] || [[ $FINAL_BALANCE == *"190000000"* ]]; then
    success "Final balance correct (1.9 GHC)"
else
    fail "Final balance incorrect"
fi

log "Verifying Interest Pool..."
GLOBAL_STATS=$(dfx canister call staking_hub get_global_stats)
# Interest Pool should have 10,000,000 (10% of 100,000,000)
if [[ $GLOBAL_STATS == *"10_000_000"* ]] || [[ $GLOBAL_STATS == *"10000000"* ]]; then
    success "Interest Pool correct (0.1 GHC)"
else
    fail "Interest Pool incorrect: $GLOBAL_STATS"
fi

log "Distributing Interest..."
DIST=$(dfx canister call staking_hub distribute_interest)
if [[ $DIST == *"Ok"* ]]; then
    success "Interest distributed"
else
    fail "Distribution failed: $DIST"
fi

log "=== ALL TESTS PASSED SUCCESSFULLY ==="
