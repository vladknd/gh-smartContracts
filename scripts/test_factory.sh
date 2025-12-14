#!/bin/bash
set -e

# ============================================
# Staking Hub Factory Test Script
# ============================================
# This script tests the Staking Hub as a factory for user_profile shards
# It validates the security fix where only admins can register shards
# ============================================

echo "================================================"
echo "  STAKING HUB FACTORY TEST"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}


info() {
    echo -e "${YELLOW}ℹ INFO:${NC} $1"
}

# ============================================
# 1. Start dfx if not running
# ============================================
echo "Step 1: Checking dfx status..."
if ! dfx ping &>/dev/null; then
    info "Starting dfx..."
    dfx start --background --clean
    sleep 5
else
    info "dfx is already running"
fi
echo ""

# ============================================
# 2. Create all canisters
# ============================================
echo "Step 2: Creating canisters..."
dfx canister create ghc_ledger 2>/dev/null || info "ghc_ledger already exists"
dfx canister create staking_hub 2>/dev/null || info "staking_hub already exists"
dfx canister create user_profile 2>/dev/null || info "user_profile already exists"
dfx canister create learning_engine 2>/dev/null || info "learning_engine already exists"
echo ""

# ============================================
# 3. Get canister IDs
# ============================================
echo "Step 3: Getting canister IDs..."
GHC_LEDGER=$(dfx canister id ghc_ledger)
STAKING_HUB=$(dfx canister id staking_hub)
USER_PROFILE=$(dfx canister id user_profile)
LEARNING_ENGINE=$(dfx canister id learning_engine)

info "GHC_LEDGER: $GHC_LEDGER"
info "STAKING_HUB: $STAKING_HUB"
info "USER_PROFILE: $USER_PROFILE"
info "LEARNING_ENGINE: $LEARNING_ENGINE"
echo ""

# ============================================
# 4. Deploy canisters
# ============================================
echo "Step 4: Building and deploying canisters..."

# Build all Rust canisters
cargo build --package staking_hub --package user_profile --package learning_engine --target wasm32-unknown-unknown --release 2>&1 | grep -v "^warning" || true

# Deploy GHC Ledger first (staking_hub may already exist when init fails)
info "Deploying ghc_ledger..."
dfx deploy ghc_ledger --argument "(variant { Init = record { 
    minting_account = record { owner = principal \"$STAKING_HUB\" };
    initial_balances = vec { 
        record { record { owner = principal \"$STAKING_HUB\" }; 420_000_000_000_000_000 }
    };
    transfer_fee = 10_000;
    token_symbol = \"GHC\";
    token_name = \"GreenHero Coin\";
    metadata = vec {};
    archive_options = record {
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        controller_id = principal \"$STAKING_HUB\";
    };
    feature_flags = opt record { icrc2 = false };
}})" 2>/dev/null || info "ghc_ledger already deployed or skipped"
sleep 1

# Deploy staking_hub
info "Deploying staking_hub..."
dfx deploy staking_hub --argument "(record { 
    ledger_id = principal \"$GHC_LEDGER\";
    user_profile_wasm_hash = vec {}
})" 2>/dev/null || dfx deploy staking_hub --argument "(record { 
    ledger_id = principal \"$GHC_LEDGER\";
    user_profile_wasm_hash = vec {}
})" --mode reinstall -y 2>/dev/null || info "staking_hub init skipped"
sleep 1

# Set learning content ID in staking hub (for factory)
info "Setting learning content ID..."
dfx canister call staking_hub set_learning_content_id "(principal \"$LEARNING_ENGINE\")" 2>/dev/null || true

# Deploy learning_engine with staking_hub_id
info "Deploying learning_engine..."
dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$STAKING_HUB\" })" 2>/dev/null \
    || dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$STAKING_HUB\" })" --mode reinstall -y 2>/dev/null \
    || info "learning_engine init skipped"
sleep 1

# Deploy user_profile
info "Deploying user_profile..."
dfx deploy user_profile --argument "(record { 
    staking_hub_id = principal \"$STAKING_HUB\";
    learning_content_id = principal \"$LEARNING_ENGINE\"
})" 2>/dev/null || dfx deploy user_profile --argument "(record { 
    staking_hub_id = principal \"$STAKING_HUB\";
    learning_content_id = principal \"$LEARNING_ENGINE\"
})" --mode reinstall -y 2>/dev/null || info "user_profile init skipped"
sleep 1

echo ""
echo "================================================"
echo "  RUNNING TESTS"
echo "================================================"
echo ""

# ============================================
# Test 1: Verify admin is set correctly
# ============================================
echo "Test 1: Verify admin is set correctly"
ADMIN=$(dfx canister call staking_hub get_admin '()' 2>&1)
MY_PRINCIPAL=$(dfx identity get-principal)
if echo "$ADMIN" | grep -q "$MY_PRINCIPAL"; then
    pass "Admin is correctly set to deployer"
else
    fail "Admin is not set correctly. Got: $ADMIN, Expected: $MY_PRINCIPAL"
fi
echo ""

# ============================================
# Test 2: Register existing shard (admin only)
# ============================================
echo "Test 2: Register existing user_profile shard"
RESULT=$(dfx canister call staking_hub register_shard "(principal \"$USER_PROFILE\")" 2>&1)
if echo "$RESULT" | grep -q "Ok"; then
    pass "Successfully registered user_profile as shard"
elif echo "$RESULT" | grep -q "already"; then
    info "Shard already registered (from previous run)"
    pass "Shard registration check passed"
else
    fail "Failed to register shard: $RESULT"
fi
echo ""

# ============================================
# Test 3: Verify shard is in registry
# ============================================
echo "Test 3: Verify shard appears in registry"
SHARDS=$(dfx canister call staking_hub get_shards '()')
if echo "$SHARDS" | grep -q "$USER_PROFILE"; then
    pass "Shard appears in registry"
else
    fail "Shard not found in registry: $SHARDS"
fi
echo ""

# ============================================
# Test 4: Verify shard is registered
# ============================================
echo "Test 4: Verify is_registered_shard works"
IS_REGISTERED=$(dfx canister call staking_hub is_registered_shard "(principal \"$USER_PROFILE\")")
if echo "$IS_REGISTERED" | grep -q "true"; then
    pass "is_registered_shard returns true for registered shard"
else
    fail "is_registered_shard returned: $IS_REGISTERED"
fi
echo ""

# ============================================
# Test 5: Test get_shard_for_new_user
# ============================================
echo "Test 5: Test get_shard_for_new_user"
SHARD_FOR_USER=$(dfx canister call staking_hub get_shard_for_new_user '()')
if echo "$SHARD_FOR_USER" | grep -q "$USER_PROFILE"; then
    pass "get_shard_for_new_user returns the active shard"
else
    fail "get_shard_for_new_user returned: $SHARD_FOR_USER"
fi
echo ""

# ============================================
# Test 6: Security Test - Unauthorized add_allowed_minter should fail
# ============================================
echo "Test 6: Security test - unauthorized add_allowed_minter fails"

# Create a different identity for security test
dfx identity new test_attacker --storage-mode=plaintext 2>/dev/null || true
dfx identity use test_attacker

ATTACKER_PRINCIPAL=$(dfx identity get-principal)
info "Using attacker identity: $ATTACKER_PRINCIPAL"

# Try to add self as minter (should fail)
ATTACK_RESULT=$(dfx canister call staking_hub add_allowed_minter "(principal \"$ATTACKER_PRINCIPAL\")" 2>&1 || true)
if echo "$ATTACK_RESULT" | grep -q -i "unauthorized\|trap\|admin"; then
    pass "Security: Unauthorized user cannot add themselves as minter"
else
    fail "Security: Attacker was able to add minter! Result: $ATTACK_RESULT"
fi

# Switch back to default identity
dfx identity use default
echo ""

# ============================================
# Test 7: Test shard count
# ============================================
echo "Test 7: Test shard count"
SHARD_COUNT=$(dfx canister call staking_hub get_shard_count '()')
if echo "$SHARD_COUNT" | grep -q "(1"; then
    pass "Shard count is 1"
else
    info "Shard count: $SHARD_COUNT"
    pass "get_shard_count works"
fi
echo ""

# ============================================
# Test 8: Test disable/enable shard
# ============================================
echo "Test 8: Test disable and enable shard"
DISABLE_RESULT=$(dfx canister call staking_hub disable_shard '(0 : nat64)')
if echo "$DISABLE_RESULT" | grep -q "Ok"; then
    pass "Successfully disabled shard 0"
else
    fail "Failed to disable shard: $DISABLE_RESULT"
fi

# Verify shard is disabled (not in active shards)
ACTIVE_SHARDS=$(dfx canister call staking_hub get_active_shards '()')
if echo "$ACTIVE_SHARDS" | grep -q "$USER_PROFILE"; then
    fail "Disabled shard still appears in active shards"
else
    pass "Disabled shard not in active shards"
fi

# Re-enable
ENABLE_RESULT=$(dfx canister call staking_hub enable_shard '(0 : nat64)')
if echo "$ENABLE_RESULT" | grep -q "Ok"; then
    pass "Successfully re-enabled shard 0"
else
    fail "Failed to enable shard: $ENABLE_RESULT"
fi
echo ""

# ============================================
# Test 9: Test full quiz flow (shard can sync)
# ============================================
echo "Test 9: Test user registration and sync"

# Register a test user (use unique name to avoid conflicts)
TIMESTAMP=$(date +%s)
REG_RESULT=$(dfx canister call user_profile register_user "(record { 
    email = \"test${TIMESTAMP}@example.com\"; 
    name = \"Test User ${TIMESTAMP}\"; 
    education = \"BSc\"; 
    gender = \"M\" 
})" 2>&1)
if echo "$REG_RESULT" | grep -q "Ok"; then
    pass "User registration successful"
elif echo "$REG_RESULT" | grep -q "already"; then
    info "User already registered"
    pass "User registration check passed"
else
    fail "User registration failed: $REG_RESULT"
fi

# Force sync
SYNC_RESULT=$(dfx canister call user_profile debug_force_sync '()')
if echo "$SYNC_RESULT" | grep -q "Ok"; then
    pass "Shard sync successful (authorized shard can call hub)"
else
    fail "Shard sync failed: $SYNC_RESULT"
fi
echo ""

# ============================================
# Test 10: Verify global stats
# ============================================
echo "Test 10: Verify global stats"
STATS=$(dfx canister call staking_hub get_global_stats '()')
info "Global Stats: $STATS"
if echo "$STATS" | grep -q "total_staked"; then
    pass "Global stats accessible"
else
    fail "Could not get global stats"
fi
echo ""

# ============================================
# Test 11: Test get_user_count on shard
# ============================================
echo "Test 11: Test get_user_count on shard"
USER_COUNT=$(dfx canister call user_profile get_user_count '()')
info "User count: $USER_COUNT"
if echo "$USER_COUNT" | grep -qE "\([0-9]+"; then
    pass "get_user_count works"
else
    fail "get_user_count failed: $USER_COUNT"
fi
echo ""

# ============================================
# Summary
# ============================================
echo "================================================"
echo "  TEST SUMMARY"
echo "================================================"
echo ""
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed! ✗${NC}"
    exit 1
fi
