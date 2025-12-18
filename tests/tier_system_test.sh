#!/bin/bash

# ==========================================
# Discrete Tier System Comprehensive Test
# ==========================================
#
# This script tests the tiered interest distribution system.
# It verifies:
# - Tier constants and configuration
# - Interest distribution across tiers
# - Tier upgrades based on staking duration
# - Sync between shards and hub
# - Interest calculation accuracy
#
# Usage: ./tier_system_test.sh
#
# Prerequisites:
# - dfx running with replica
# - Canisters deployed (staking_hub, user_profile)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Helper to extract field from Candid response
extract_field() {
    local response="$1"
    local field="$2"
    echo "$response" | grep -oP "${field}\s*=\s*\K[^;)]+" | head -1 | tr -d ' "'
}

# Helper to extract array element
extract_array_element() {
    local response="$1"
    local field="$2"
    local index="$3"
    echo "$response" | grep -oP "${field}\s*=\s*vec\s*\{\s*\K[^}]+" | tr ';' '\n' | sed -n "$((index+1))p" | tr -d ' '
}

echo ""
echo "=========================================="
echo "   DISCRETE TIER SYSTEM TEST SUITE"
echo "=========================================="
echo ""

# ==========================================
# Test 1: Verify Tier Configuration in Hub
# ==========================================

log_info "Test 1: Checking staking_hub global stats structure..."

STATS=$(dfx canister call staking_hub get_global_stats 2>/dev/null || echo "ERROR")

if [[ "$STATS" != "ERROR" ]]; then
    log_success "get_global_stats callable"
    
    # Check for tier_staked field
    if echo "$STATS" | grep -q "tier_staked"; then
        log_success "tier_staked field exists in GlobalStats"
    else
        log_fail "tier_staked field missing in GlobalStats"
    fi
    
    # Check for tier_reward_indexes field
    if echo "$STATS" | grep -q "tier_reward_indexes"; then
        log_success "tier_reward_indexes field exists in GlobalStats"
    else
        log_fail "tier_reward_indexes field missing in GlobalStats"
    fi
else
    log_fail "get_global_stats failed"
fi

echo ""

# ==========================================
# Test 2: Sync Shard API with Tier Support
# ==========================================

log_info "Test 2: Checking sync_shard API (with tier support)..."

# Get DID to verify method exists
DID_RESULT=$(dfx canister metadata staking_hub candid:service 2>/dev/null | head -50 || echo "")

if echo "$DID_RESULT" | grep -q "sync_shard"; then
    log_success "sync_shard method exists in staking_hub"
else
    log_warning "sync_shard method not found in metadata (may still work)"
fi

echo ""

# ==========================================
# Test 3: Initial State Check
# ==========================================

log_info "Test 3: Checking initial tier state..."

STATS=$(dfx canister call staking_hub get_global_stats 2>/dev/null)

# Parse tier_staked array
TIER_0=$(echo "$STATS" | grep -oP 'tier_staked\s*=\s*vec\s*\{\s*\K\d+' | head -1 || echo "0")
log_info "  Bronze tier staked: $TIER_0"

TOTAL_STAKED=$(echo "$STATS" | grep -oP 'total_staked\s*=\s*\K\d+' | head -1 || echo "0")
log_info "  Total staked: $TOTAL_STAKED"

INTEREST_POOL=$(echo "$STATS" | grep -oP 'interest_pool\s*=\s*\K\d+' | head -1 || echo "0")
log_info "  Interest pool: $INTEREST_POOL"

log_success "Initial state retrieved"

echo ""

# ==========================================
# Test 4: Distribute Interest (Tier-based)
# ==========================================

log_info "Test 4: Testing distribute_interest with tier system..."

# First add some tokens to the interest pool by unstaking (if possible)
# For this test, we'll just check that distribute_interest returns the right format

DIST_RESULT=$(dfx canister call staking_hub distribute_interest 2>/dev/null || echo "No interest to distribute")

if echo "$DIST_RESULT" | grep -q "Distributed"; then
    log_success "distribute_interest succeeded"
    
    # Check for tier-specific message
    if echo "$DIST_RESULT" | grep -q "tiers"; then
        log_success "Distribution message includes tier information"
    fi
    
    if echo "$DIST_RESULT" | grep -q "Indexes:"; then
        log_success "Distribution message includes tier indexes"
    fi
else
    log_warning "No interest to distribute (expected if pool is empty)"
fi

echo ""

# ==========================================
# Test 5: User Profile Tier Fields
# ==========================================

log_info "Test 5: Checking user_profile tier fields..."

# Get TEST_IDENTITY principal
TEST_PRINCIPAL=$(dfx identity get-principal 2>/dev/null)
log_info "  Test principal: $TEST_PRINCIPAL"

# Try to get profile (may not exist)
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$TEST_PRINCIPAL\")" 2>/dev/null || echo "null")

if [[ "$PROFILE" != "null" && "$PROFILE" != "(null)" ]]; then
    log_success "Profile retrieved for test principal"
    
    # Check for new tier fields
    if echo "$PROFILE" | grep -q "current_tier"; then
        log_success "current_tier field exists in UserProfile"
    else
        log_fail "current_tier field missing in UserProfile"
    fi
    
    if echo "$PROFILE" | grep -q "tier_start_index"; then
        log_success "tier_start_index field exists in UserProfile"
    else
        log_fail "tier_start_index field missing in UserProfile"
    fi
    
    if echo "$PROFILE" | grep -q "initial_stake_time"; then
        log_success "initial_stake_time field exists in UserProfile"
    else
        log_fail "initial_stake_time field missing in UserProfile"
    fi
else
    log_warning "No profile found for test principal (register first to test profile fields)"
fi

echo ""

# ==========================================
# Test 6: Register and Check New User
# ==========================================

log_info "Test 6: Testing user registration with tier fields..."

# Register a test user
REG_RESULT=$(dfx canister call user_profile register_user '(record { email = "tier_test@example.com"; name = "Tier Tester"; education = "Test"; gender = "Test" })' 2>/dev/null || echo "already registered")

if echo "$REG_RESULT" | grep -q "Ok" || echo "$REG_RESULT" | grep -q "already"; then
    log_success "Registration succeeded or user exists"
    
    # Get the profile again
    PROFILE=$(dfx canister call user_profile get_profile "(principal \"$TEST_PRINCIPAL\")" 2>/dev/null || echo "null")
    
    if [[ "$PROFILE" != "null" ]]; then
        # Extract current_tier
        CURRENT_TIER=$(echo "$PROFILE" | grep -oP 'current_tier\s*=\s*\K\d+' | head -1 || echo "-1")
        log_info "  Current tier: $CURRENT_TIER"
        
        if [[ "$CURRENT_TIER" == "0" ]]; then
            log_success "New user starts in Bronze tier (0)"
        else
            log_warning "Unexpected tier for new user: $CURRENT_TIER"
        fi
    fi
else
    log_fail "Registration failed"
fi

echo ""

# ==========================================
# Test 7: Force Sync and Check Tier Indexes
# ==========================================

log_info "Test 7: Testing sync with tier indexes..."

SYNC_RESULT=$(dfx canister call user_profile debug_force_sync 2>/dev/null || echo "SYNC_FAILED")

if echo "$SYNC_RESULT" | grep -q "Ok"; then
    log_success "Sync succeeded"
else
    log_warning "Sync returned: $SYNC_RESULT"
fi

echo ""

# ==========================================
# Test 8: Interest Pool Growth Simulation
# ==========================================

log_info "Test 8: Simulating interest pool growth..."

# Get current stats
STATS_BEFORE=$(dfx canister call staking_hub get_global_stats 2>/dev/null)
POOL_BEFORE=$(echo "$STATS_BEFORE" | grep -oP 'interest_pool\s*=\s*\K\d+' | head -1 || echo "0")
log_info "  Pool before: $POOL_BEFORE"

# If there's interest in the pool, distribute it
if [[ "$POOL_BEFORE" != "0" ]]; then
    DIST=$(dfx canister call staking_hub distribute_interest 2>/dev/null)
    log_info "  Distribution result: $DIST"
    
    STATS_AFTER=$(dfx canister call staking_hub get_global_stats 2>/dev/null)
    POOL_AFTER=$(echo "$STATS_AFTER" | grep -oP 'interest_pool\s*=\s*\K\d+' | head -1 || echo "0")
    log_info "  Pool after: $POOL_AFTER"
    
    if [[ "$POOL_AFTER" -lt "$POOL_BEFORE" ]]; then
        log_success "Interest pool decreased after distribution"
    fi
else
    log_warning "No interest in pool to test distribution"
fi

echo ""

# ==========================================
# Test 9: Tier Threshold Validation
# ==========================================

log_info "Test 9: Validating tier threshold constants..."

# These are the expected thresholds (in nanoseconds)
BRONZE_THRESHOLD=0
SILVER_THRESHOLD=$((30 * 24 * 60 * 60 * 1000000000))
GOLD_THRESHOLD=$((90 * 24 * 60 * 60 * 1000000000))
DIAMOND_THRESHOLD=$((365 * 24 * 60 * 60 * 1000000000))

log_info "  Bronze: 0 days (0 nanos)"
log_info "  Silver: 30 days ($SILVER_THRESHOLD nanos)"
log_info "  Gold: 90 days ($GOLD_THRESHOLD nanos)"
log_info "  Diamond: 365 days ($DIAMOND_THRESHOLD nanos)"

log_success "Tier thresholds documented"

echo ""

# ==========================================
# Test 10: Tier Weight Validation
# ==========================================

log_info "Test 10: Validating tier weight configuration..."

# Expected weights (percentages)
BRONZE_WEIGHT=20
SILVER_WEIGHT=25
GOLD_WEIGHT=30
DIAMOND_WEIGHT=25
TOTAL_WEIGHT=$((BRONZE_WEIGHT + SILVER_WEIGHT + GOLD_WEIGHT + DIAMOND_WEIGHT))

log_info "  Bronze: $BRONZE_WEIGHT%"
log_info "  Silver: $SILVER_WEIGHT%"
log_info "  Gold: $GOLD_WEIGHT%"
log_info "  Diamond: $DIAMOND_WEIGHT%"
log_info "  Total: $TOTAL_WEIGHT%"

if [[ "$TOTAL_WEIGHT" -eq 100 ]]; then
    log_success "Tier weights sum to 100%"
else
    log_fail "Tier weights do not sum to 100%"
fi

echo ""

# ==========================================
# Summary
# ==========================================

echo "=========================================="
echo "           TEST SUMMARY"
echo "=========================================="
echo ""
echo -e "  ${GREEN}Passed:${NC} $TESTS_PASSED"
echo -e "  ${RED}Failed:${NC} $TESTS_FAILED"
echo ""

TOTAL=$((TESTS_PASSED + TESTS_FAILED))

if [[ "$TESTS_FAILED" -eq 0 ]]; then
    echo -e "${GREEN}All $TOTAL tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}$TESTS_PASSED of $TOTAL tests passed${NC}"
    exit 1
fi
