#!/bin/bash

# Comprehensive Test Suite (Updated for New Canister Architecture)
# Tests core functionality: user registration, quiz, staking, governance
# This script assumes canisters are already deployed

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
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

warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log "=== Starting GHC Dapp Comprehensive Test Suite ==="
log "=== (Assumes canisters are already deployed) ==="

# --- 1. Setup & Admin Phase ---

log "Setting up environment..."

# Use default identity for admin tasks
dfx identity use default
ADMIN_PRINCIPAL=$(dfx identity get-principal)
log "Admin Principal: $ADMIN_PRINCIPAL"

# Get Canister IDs
STAKING_HUB_ID=$(dfx canister id staking_hub)
TREASURY_ID=$(dfx canister id treasury_canister)
GOVERNANCE_ID=$(dfx canister id governance_canister)
LEDGER_ID=$(dfx canister id ghc_ledger)
USER_PROFILE_ID=$(dfx canister id user_profile)
LEARNING_ID=$(dfx canister id learning_engine)
FOUNDER_VESTING_ID=$(dfx canister id founder_vesting)

log "Staking Hub: $STAKING_HUB_ID"
log "Treasury: $TREASURY_ID"
log "Governance: $GOVERNANCE_ID"
log "Ledger: $LEDGER_ID"
log "User Profile: $USER_PROFILE_ID"

# --- 2. User Phase: Setup ---

# Generate unique identity
TEST_USER="ghc_test_user_$(date +%s)"
log "Creating temporary test user: $TEST_USER"

dfx identity new "$TEST_USER" --storage-mode plaintext 2>/dev/null || true
dfx identity use "$TEST_USER"
USER_PRINCIPAL=$(dfx identity get-principal)
log "Test User Principal: $USER_PRINCIPAL"

# Cleanup trap
cleanup() {
    echo -e "\n${BLUE}=== Cleaning up... ===${NC}"
    dfx identity use default
    dfx identity remove "$TEST_USER" 2>/dev/null || true
    success "Identity removed"
}
trap cleanup EXIT

# --- 3. Learning & Earning (Virtual Staking) ---

log "--- Phase 1: Learning & Earning (Virtual Staking) ---"

# Register user first
log "Registering test user..."
REG_RESULT=$(dfx canister call user_profile register_user '(record { email = "test@example.com"; name = "Test User"; education = "Test"; gender = "Test" })' 2>&1)
if [[ $REG_RESULT == *"Ok"* ]]; then
    success "User registered"
else
    fail "User registration failed: $REG_RESULT"
fi

# Add a learning unit (as admin)
log "Adding test learning unit..."
dfx identity use default
ADD_UNIT=$(dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "test_1";
    unit_title = "Test Unit";
    chapter_id = "1";
    chapter_title = "Test Chapter";
    head_unit_id = "1";
    head_unit_title = "Test Head Unit";
    content = "Content";
    paraphrase = "Paraphrase";
    quiz = vec { record { question = "Q"; options = vec {"A"}; answer = 0 } };
})' 2>&1)
dfx identity use "$TEST_USER"

if [[ $ADD_UNIT == *"Ok"* ]] || [[ $ADD_UNIT == *"already exists"* ]]; then
    success "Learning unit ready"
else
    fail "Failed to add learning unit: $ADD_UNIT"
fi

log "Submitting Quiz (Mining)..."
RESULT=$(dfx canister call user_profile submit_quiz '("test_1", vec {0})' 2>&1)
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

# Switch to admin for governance checks
dfx identity use default

log "Checking Treasury State..."
TREASURY=$(dfx canister call treasury_canister get_treasury_state 2>&1)
if [[ $TREASURY == *"balance"* ]]; then
    success "Treasury state accessible"
else
    fail "Treasury state unavailable: $TREASURY"
fi

log "Checking MMCR Status..."
MMCR=$(dfx canister call treasury_canister get_mmcr_status 2>&1)
if [[ $MMCR == *"releases_completed"* ]]; then
    success "MMCR status accessible"
else
    fail "MMCR status unavailable: $MMCR"
fi

log "Checking Spendable Balance..."
SPENDABLE=$(dfx canister call treasury_canister get_spendable_balance 2>&1)
if [[ -n "$SPENDABLE" ]]; then
    success "Spendable balance accessible: $SPENDABLE"
else
    fail "Spendable balance query failed: $SPENDABLE"
fi

log "Checking Governance Config..."
GOV_CONFIG=$(dfx canister call governance_canister get_governance_config 2>&1)
if [[ $GOV_CONFIG == *"votingDays"* ]] || [[ $GOV_CONFIG == *"supportDays"* ]]; then
    success "Governance config accessible"
else
    fail "Governance config unavailable: $GOV_CONFIG"
fi

log "Checking Board Members..."
BOARD=$(dfx canister call governance_canister get_board_members 2>&1)
if [[ $BOARD == *"vec"* ]]; then
    success "Board members query successful"
else
    fail "Board members query failed: $BOARD"
fi

# --- 5. Unstaking (No Penalty) ---

log "--- Phase 3: Unstaking (No Penalty) ---"

dfx identity use "$TEST_USER"

log "Unstaking 0.5 GHC..."
UNSTAKE=$(dfx canister call user_profile unstake '(50_000_000)' 2>&1)
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

if [[ $WALLET_BALANCE == *"50_000_000"* ]] || [[ $WALLET_BALANCE == *"50000000"* ]]; then
    success "Wallet balance correct (received unstaked tokens)"
else
    fail "Wallet balance incorrect: $WALLET_BALANCE"
fi

# --- 6. Global Stats ---

log "--- Phase 4: Global Stats Verification ---"

dfx identity use default

log "Forcing sync..."
dfx canister call user_profile debug_force_sync > /dev/null 2>&1
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

# --- 8. Content Governance Canisters ---

log "--- Phase 6: Content Governance Canisters ---"

log "Checking Media Assets..."
MEDIA_STATUS=$(dfx canister status media_assets 2>&1)
if [[ $MEDIA_STATUS == *"Status: Running"* ]]; then
    success "Media Assets canister running"
else
    fail "Media Assets not running: $MEDIA_STATUS"
fi

log "Checking Staging Assets..."
STAGING_STATUS=$(dfx canister status staging_assets 2>&1)
if [[ $STAGING_STATUS == *"Status: Running"* ]]; then
    success "Staging Assets canister running"
else
    fail "Staging Assets not running: $STAGING_STATUS"
fi

log "=== ALL TESTS PASSED SUCCESSFULLY ==="
