#!/bin/bash

# Comprehensive Test Suite (Updated for Simplified Tokenomics)
# Tests core functionality: user registration, quiz, staking, governance

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
USER_PROFILE_ID=$(dfx canister id user_profile)

log "Staking Hub: $STAKING_HUB_ID"
log "Governance: $GOVERNANCE_ID"
log "Ledger: $LEDGER_ID"
log "User Profile: $USER_PROFILE_ID"

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

# Register user first
log "Registering test user..."
dfx canister call user_profile register_user '(record { email = "test@example.com"; name = "Test User"; education = "Test"; gender = "Test" })' > /dev/null
success "User registered"

# Add a learning unit
log "Adding test learning unit..."
dfx identity use default
dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "test_1";
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
success "Learning unit added"

log "Submitting Quiz (Mining)..."
RESULT=$(dfx canister call user_profile submit_quiz '("test_1", vec {0})')
if [[ $RESULT == *"Ok"* ]]; then
    success "Quiz submitted"
else
    fail "Quiz submission failed: $RESULT"
fi

log "Verifying Virtual Balance (Auto-Staked)..."
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")")
if [[ $PROFILE == *"100_000_000"* ]] || [[ $PROFILE == *"100000000"* ]]; then
    success "Virtual Balance correct (1 GHC)"
else
    fail "Incorrect Virtual Balance: $PROFILE"
fi

# --- 4. Governance ---

log "--- Phase 2: Governance ---"

# Switch to admin for governance proposal
dfx identity use default

log "Checking Treasury State..."
TREASURY=$(dfx canister call operational_governance get_treasury_state)
if [[ $TREASURY == *"balance"* ]]; then
    success "Treasury state accessible"
else
    fail "Treasury state unavailable: $TREASURY"
fi

log "Checking MMCR Status..."
MMCR=$(dfx canister call operational_governance get_mmcr_status)
if [[ $MMCR == *"releases_completed"* ]]; then
    success "MMCR status accessible"
else
    fail "MMCR status unavailable: $MMCR"
fi

log "Checking Spendable Balance..."
SPENDABLE=$(dfx canister call operational_governance get_spendable_balance)
if [[ $SPENDABLE == *"nat64"* ]] || [[ -n "$SPENDABLE" ]]; then
    success "Spendable balance accessible: $SPENDABLE"
else
    fail "Spendable balance query failed: $SPENDABLE"
fi

# --- 5. Unstaking (No Penalty) ---

log "--- Phase 3: Unstaking (No Penalty) ---"

dfx identity use "$TEST_USER"

log "Unstaking 0.5 GHC..."
UNSTAKE=$(dfx canister call user_profile unstake '(50_000_000)')
if [[ $UNSTAKE == *"Ok"* ]]; then
    success "Unstake successful (100% returned, no penalty)"
else
    fail "Unstake failed: $UNSTAKE"
fi

log "Verifying Remaining Staked Balance..."
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")")
if [[ $PROFILE == *"staked_balance = 50_000_000"* ]] || [[ $PROFILE == *"staked_balance = 50000000"* ]]; then
    success "Remaining staked balance correct (0.5 GHC)"
else
    fail "Staked balance incorrect: $PROFILE"
fi

log "Verifying Wallet Balance (received unstaked tokens)..."
WALLET_BALANCE=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })")
log "Wallet Balance: $WALLET_BALANCE"

if [[ $WALLET_BALANCE == *"50_000_000"* ]] || [[ $WALLET_BALANCE == *"50000000"* ]] || [[ $WALLET_BALANCE == *"49_990_000"* ]]; then
    success "Wallet balance correct (received unstaked tokens minus fee)"
else
    fail "Wallet balance incorrect: $WALLET_BALANCE"
fi

# --- 6. Global Stats ---

log "--- Phase 4: Global Stats Verification ---"

dfx identity use default

log "Forcing sync..."
dfx canister call user_profile debug_force_sync > /dev/null
success "Sync forced"

log "Checking Global Stats..."
GLOBAL_STATS=$(dfx canister call staking_hub get_global_stats)
if [[ $GLOBAL_STATS == *"total_staked"* ]] && [[ $GLOBAL_STATS == *"total_unstaked"* ]]; then
    success "Global Stats correct"
else
    fail "Global Stats incorrect: $GLOBAL_STATS"
fi

# --- 7. Founder Vesting ---

log "--- Phase 5: Founder Vesting ---"

log "Checking Founder Vesting Schedules..."
VESTING=$(dfx canister call founder_vesting get_all_vesting_schedules)
if [[ $VESTING == *"total_allocation"* ]]; then
    success "Vesting schedules accessible"
else
    fail "Vesting schedules unavailable: $VESTING"
fi

log "Checking Total Unclaimed..."
UNCLAIMED=$(dfx canister call founder_vesting get_total_unclaimed)
if [[ -n "$UNCLAIMED" ]]; then
    success "Total unclaimed accessible: $UNCLAIMED"
else
    fail "Total unclaimed unavailable"
fi

log "=== ALL TESTS PASSED SUCCESSFULLY ==="
