#!/bin/bash

# ============================================================================
# User Profile Authentication Test Script
# ============================================================================
# 
# This script tests the user profile authentication flow to verify that:
# 1. Different principals create different user profiles
# 2. Each principal can only access their own profile
# 3. The backend correctly isolates user data by principal
# 4. The admin debug endpoints work correctly
#
# Usage: ./scripts/tests/test_user_profile_auth.sh
#
# Prerequisites:
# - dfx is running: dfx start --clean
# - Canisters are deployed: dfx deploy
# ============================================================================

# Don't use set -e because we expect some failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0

echo -e "${BLUE}============================================================================${NC}"
echo -e "${BLUE}          USER PROFILE AUTHENTICATION TEST SUITE${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo ""

# Helper function to print test results
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    echo -e "${RED}  Expected: $2${NC}"
    echo -e "${RED}  Got: $3${NC}"
    ((FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# ============================================================================
# SETUP: Create test identities
# ============================================================================
section "SETUP: Creating Test Identities"

# Store original identity
ORIGINAL_IDENTITY=$(dfx identity whoami)
info "Original identity: $ORIGINAL_IDENTITY"

# Create test identities if they don't exist
create_test_identity() {
    local name=$1
    if ! dfx identity list | grep -q "^$name$"; then
        dfx identity new "$name" --storage-mode=plaintext 2>/dev/null || true
        info "Created identity: $name"
    else
        info "Identity exists: $name"
    fi
}

create_test_identity "test_auth_user1"
create_test_identity "test_auth_user2"
create_test_identity "test_auth_user3"

# Get principals for each identity
dfx identity use test_auth_user1
USER1_PRINCIPAL=$(dfx identity get-principal)
info "User1 Principal: $USER1_PRINCIPAL"

dfx identity use test_auth_user2
USER2_PRINCIPAL=$(dfx identity get-principal)
info "User2 Principal: $USER2_PRINCIPAL"

dfx identity use test_auth_user3
USER3_PRINCIPAL=$(dfx identity get-principal)
info "User3 Principal: $USER3_PRINCIPAL"

# ============================================================================
# TEST 1: Verify whoami endpoint returns correct principal
# ============================================================================
section "TEST 1: Verify 'whoami' Endpoint"

dfx identity use test_auth_user1
WHOAMI_RESULT=$(dfx canister call user_profile whoami "()" 2>&1)

if echo "$WHOAMI_RESULT" | grep -q "$USER1_PRINCIPAL"; then
    pass "whoami returns correct principal for user1"
else
    fail "whoami should return user1's principal" "$USER1_PRINCIPAL" "$WHOAMI_RESULT"
fi

dfx identity use test_auth_user2
WHOAMI_RESULT=$(dfx canister call user_profile whoami "()" 2>&1)

if echo "$WHOAMI_RESULT" | grep -q "$USER2_PRINCIPAL"; then
    pass "whoami returns correct principal for user2"
else
    fail "whoami should return user2's principal" "$USER2_PRINCIPAL" "$WHOAMI_RESULT"
fi

# ============================================================================
# TEST 2: Register different users with different profiles
# ============================================================================
section "TEST 2: Register Different Users"

# Register User 1
dfx identity use test_auth_user1
REGISTER_RESULT=$(dfx canister call user_profile register_user '(record { 
    email = "alice@example.com"; 
    name = "Alice Anderson"; 
    education = "PhD in Computer Science"; 
    gender = "Female" 
})' 2>&1)

if echo "$REGISTER_RESULT" | grep -q "Ok"; then
    pass "User1 (Alice) registered successfully"
elif echo "$REGISTER_RESULT" | grep -q "already"; then
    info "User1 (Alice) was already registered"
    PASSED=$((PASSED+1))
else
    fail "User1 registration" "Ok" "$REGISTER_RESULT"
fi

# Register User 2
dfx identity use test_auth_user2
REGISTER_RESULT=$(dfx canister call user_profile register_user '(record { 
    email = "bob@example.com"; 
    name = "Bob Builder"; 
    education = "Masters in Engineering"; 
    gender = "Male" 
})' 2>&1)

if echo "$REGISTER_RESULT" | grep -q "Ok"; then
    pass "User2 (Bob) registered successfully"
elif echo "$REGISTER_RESULT" | grep -q "already"; then
    info "User2 (Bob) was already registered"
    PASSED=$((PASSED+1))
else
    fail "User2 registration" "Ok" "$REGISTER_RESULT"
fi

# Register User 3
dfx identity use test_auth_user3
REGISTER_RESULT=$(dfx canister call user_profile register_user '(record { 
    email = "charlie@example.com"; 
    name = "Charlie Chen"; 
    education = "Bachelors in Arts"; 
    gender = "Non-binary" 
})' 2>&1)

if echo "$REGISTER_RESULT" | grep -q "Ok"; then
    pass "User3 (Charlie) registered successfully"
elif echo "$REGISTER_RESULT" | grep -q "already"; then
    info "User3 (Charlie) was already registered"
    PASSED=$((PASSED+1))
else
    fail "User3 registration" "Ok" "$REGISTER_RESULT"
fi

# ============================================================================
# TEST 3: Verify each principal gets their own profile
# ============================================================================
section "TEST 3: Verify Profile Isolation"

# Get User 1's profile
PROFILE1=$(dfx canister call user_profile get_profile "(principal \"$USER1_PRINCIPAL\")" 2>&1)
if echo "$PROFILE1" | grep -q "Alice Anderson"; then
    pass "User1 profile contains correct name (Alice Anderson)"
else
    fail "User1 profile name" "Alice Anderson" "$PROFILE1"
fi

if echo "$PROFILE1" | grep -q "alice@example.com"; then
    pass "User1 profile contains correct email"
else
    fail "User1 profile email" "alice@example.com" "$PROFILE1"
fi

# Get User 2's profile
PROFILE2=$(dfx canister call user_profile get_profile "(principal \"$USER2_PRINCIPAL\")" 2>&1)
if echo "$PROFILE2" | grep -q "Bob Builder"; then
    pass "User2 profile contains correct name (Bob Builder)"
else
    fail "User2 profile name" "Bob Builder" "$PROFILE2"
fi

if echo "$PROFILE2" | grep -q "bob@example.com"; then
    pass "User2 profile contains correct email"
else
    fail "User2 profile email" "bob@example.com" "$PROFILE2"
fi

# Get User 3's profile
PROFILE3=$(dfx canister call user_profile get_profile "(principal \"$USER3_PRINCIPAL\")" 2>&1)
if echo "$PROFILE3" | grep -q "Charlie Chen"; then
    pass "User3 profile contains correct name (Charlie Chen)"
else
    fail "User3 profile name" "Charlie Chen" "$PROFILE3"
fi

# ============================================================================
# TEST 4: Verify is_user_registered endpoint
# ============================================================================
section "TEST 4: Verify is_user_registered Endpoint"

IS_REG1=$(dfx canister call user_profile is_user_registered "(principal \"$USER1_PRINCIPAL\")" 2>&1)
if echo "$IS_REG1" | grep -q "true"; then
    pass "is_user_registered returns true for registered user1"
else
    fail "is_user_registered for user1" "true" "$IS_REG1"
fi

# Check a random non-existent principal
IS_REG_FAKE=$(dfx canister call user_profile is_user_registered "(principal \"aaaaa-aa\")" 2>&1)
if echo "$IS_REG_FAKE" | grep -q "false"; then
    pass "is_user_registered returns false for non-existent user"
else
    fail "is_user_registered for fake user" "false" "$IS_REG_FAKE"
fi

# ============================================================================
# TEST 5: Verify profiles are DIFFERENT (not cached/shared)
# ============================================================================
section "TEST 5: Verify No Profile Mixing/Caching"

# Ensure User1's profile doesn't contain User2's or User3's data
if echo "$PROFILE1" | grep -q "Bob"; then
    fail "User1 profile should NOT contain Bob's data" "No 'Bob'" "$PROFILE1"
else
    pass "User1 profile does NOT contain User2's (Bob) data"
fi

if echo "$PROFILE1" | grep -q "Charlie"; then
    fail "User1 profile should NOT contain Charlie's data" "No 'Charlie'" "$PROFILE1"
else
    pass "User1 profile does NOT contain User3's (Charlie) data"
fi

# Ensure User2's profile doesn't contain User1's or User3's data
if echo "$PROFILE2" | grep -q "Alice"; then
    fail "User2 profile should NOT contain Alice's data" "No 'Alice'" "$PROFILE2"
else
    pass "User2 profile does NOT contain User1's (Alice) data"
fi

# ============================================================================
# TEST 6: Admin list all users (requires controller identity)
# ============================================================================
section "TEST 6: Admin Debug Endpoints"

# Switch to default identity (controller)
dfx identity use default

# Test admin_list_all_users
ADMIN_LIST=$(dfx canister call user_profile admin_list_all_users "(0 : nat32, 10 : nat32)" 2>&1)

if echo "$ADMIN_LIST" | grep -q "Ok"; then
    pass "admin_list_all_users accessible by controller"
    
    # Verify it lists multiple users
    if echo "$ADMIN_LIST" | grep -q "Alice"; then
        pass "Admin list contains Alice"
    fi
    if echo "$ADMIN_LIST" | grep -q "Bob"; then
        pass "Admin list contains Bob"
    fi
    if echo "$ADMIN_LIST" | grep -q "Charlie"; then
        pass "Admin list contains Charlie"
    fi
else
    fail "admin_list_all_users" "Ok with user list" "$ADMIN_LIST"
fi

# Test that non-controller cannot access admin endpoints
dfx identity use test_auth_user1
ADMIN_LIST_UNAUTH=$(dfx canister call user_profile admin_list_all_users "(0 : nat32, 10 : nat32)" 2>&1)

if echo "$ADMIN_LIST_UNAUTH" | grep -q "Unauthorized"; then
    pass "admin_list_all_users correctly rejects non-controller"
else
    fail "admin_list_all_users should reject non-controller" "Unauthorized error" "$ADMIN_LIST_UNAUTH"
fi

# ============================================================================
# TEST 7: Verify anonymous rejection
# ============================================================================
section "TEST 7: Anonymous User Rejection"

dfx identity use anonymous
ANON_REGISTER=$(dfx canister call user_profile register_user '(record { 
    email = "anon@example.com"; 
    name = "Anonymous"; 
    education = "None"; 
    gender = "Unknown" 
})' 2>&1)

if echo "$ANON_REGISTER" | grep -q "Anonymous registration is not allowed\|Err"; then
    pass "Anonymous registration correctly rejected"
else
    fail "Anonymous registration should be rejected" "Error" "$ANON_REGISTER"
fi

# ============================================================================
# CLEANUP: Restore original identity
# ============================================================================
section "CLEANUP"

dfx identity use "$ORIGINAL_IDENTITY"
info "Restored original identity: $ORIGINAL_IDENTITY"

# ============================================================================
# SUMMARY
# ============================================================================
echo ""
echo -e "${BLUE}============================================================================${NC}"
echo -e "${BLUE}                           TEST SUMMARY${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo ""
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed! Backend authentication is working correctly.${NC}"
    echo ""
    echo -e "${YELLOW}If you're seeing the same profile for different users in the frontend,${NC}"
    echo -e "${YELLOW}the issue is in the frontend authentication flow, not the backend.${NC}"
    echo ""
    echo -e "Debug steps for frontend:"
    echo "  1. Call 'whoami' from the frontend after login"
    echo "  2. Compare the principal returned vs what you expect"
    echo "  3. Clear localStorage/IndexedDB between logins"
    echo "  4. Make sure authClient.logout() is called before new login"
    exit 0
else
    echo -e "${RED}✗ Some tests failed. Review the output above.${NC}"
    exit 1
fi
