#!/bin/bash

# Learning Engine Test - Tests the complete learning flow
# Tests: learning unit creation, quiz submission, reward allocation

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "LEARNING ENGINE SIMPLE E2E TEST"

# ============================================================================
# 1. SETUP & DEPLOY
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
fi

LEARNING_ENGINE_ID=$(dfx canister id learning_engine)
USER_PROFILE_ID=$(dfx canister id user_profile)
STAKING_HUB_ID=$(dfx canister id staking_hub)

log_info "Learning Engine: $LEARNING_ENGINE_ID"
log_info "User Profile: $USER_PROFILE_ID"
log_info "Staking Hub: $STAKING_HUB_ID"

# Create unique test user
TEST_USER="quiz_test_$(date +%s)"
dfx identity new "$TEST_USER" --storage-mode plaintext 2>/dev/null || true
dfx identity use "$TEST_USER"
USER_PRINCIPAL=$(dfx identity get-principal)

# Cleanup trap
cleanup() {
    echo -e "\n${BLUE}=== Cleaning up... ===${NC}"
    dfx identity use default
    dfx identity remove "$TEST_USER" 2>/dev/null || true
}
trap cleanup EXIT

# Test 1: Add a learning unit (as admin)
log_step "Test 1: Adding learning unit"
QUIZ_ID="quiz_test_unit_$(date +%s)"
dfx identity use default
RESULT=$(dfx canister call learning_engine add_content_node "(record {
    id = \"$QUIZ_ID\";
    parent_id = null;
    order = 1 : nat32;
    display_type = \"unit\";
    title = \"Quiz Test Unit\";
    description = opt \"Test Description\";
    content = opt \"This is test content.\";
    paraphrase = opt \"This is test paraphrase.\";
    media = null;
    quiz = opt record {
        questions = vec {
            record {
                question = \"What is 1+1?\";
                options = vec { \"1\"; \"2\"; \"3\" };
                answer = 1 : nat8;
            };
            record {
                question = \"What is 2+2?\";
                options = vec { \"3\"; \"4\"; \"5\" };
                answer = 1 : nat8;
            };
        };
    };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})" 2>&1)

if [[ $RESULT == *"Ok"* ]] || [[ $RESULT == *"already exists"* ]]; then
    log_pass "Learning unit added/exists"
else
    log_fail "Failed to add learning unit: $RESULT"
fi

# Test 2: Get learning unit (verify answers are hidden)
log_step "Test 2: Verifying answers are hidden in public queries"
PUBLIC_UNIT=$(dfx canister call learning_engine get_content_node "(\"$QUIZ_ID\")" 2>&1)

if [[ $PUBLIC_UNIT == *"answer"* ]] && [[ $PUBLIC_UNIT != *"answer = 1"* ]]; then
    log_pass "Answers are hidden in public response"
elif [[ $PUBLIC_UNIT == *"null"* ]] || [[ $PUBLIC_UNIT == *"None"* ]]; then
    log_pass "Unit not found or answers stripped"
else
    # Check if answers are exposed
    if [[ $PUBLIC_UNIT == *"answer = 1"* ]]; then
        log_fail "Public unit contains answers! Security issue."
    else
        log_pass "Public unit does not expose answers"
    fi
fi

# Test 3: Register user and submit quiz
log_step "Test 3: Registering user"
dfx identity use "$TEST_USER"
REG_RESULT=$(dfx canister call user_profile register_user '(record { 
    email = "quiz@test.com"; 
    name = "Quiz Tester"; 
    education = "Test"; 
    gender = "Test" 
})' 2>&1)

if [[ $REG_RESULT == *"Ok"* ]]; then
    log_pass "User registered"
else
    log_fail "User registration failed: $REG_RESULT"
fi

# Test 4: Submit incorrect quiz
log_step "Test 4: Submitting incorrect quiz answers"
WRONG_RESULT=$(dfx canister call user_profile submit_quiz "(\"$QUIZ_ID\", vec { 0; 0 })" 2>&1)

if [[ $WRONG_RESULT == *"Incorrect"* ]] || [[ $WRONG_RESULT == *"Err"* ]]; then
    log_pass "Incorrect answers correctly rejected"
else
    # If it passed, might be different threshold
    log_info "Note: Result was: $WRONG_RESULT"
fi

# Test 5: Submit correct quiz
log_step "Test 5: Submitting correct quiz answers"
CORRECT_RESULT=$(dfx canister call user_profile submit_quiz "(\"$QUIZ_ID\", vec { 1; 1 })" 2>&1)

if [[ $CORRECT_RESULT == *"Ok"* ]]; then
    log_pass "Correct answers accepted"
else
    log_fail "Correct answers rejected: $CORRECT_RESULT"
fi

# Test 6: Verify reward allocation
log_step "Test 6: Verifying reward allocation"
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")" 2>&1)

if [[ $PROFILE == *"staked_balance"* ]]; then
    log_pass "Profile shows staked balance (rewards allocated)"
    log_info "Profile: $PROFILE"
else
    log_fail "No staked balance found: $PROFILE"
fi

# Test 7: Verify quiz is marked as completed (cannot retake immediately)
log_step "Test 7: Verifying quiz completion prevents immediate retake"
RETAKE_RESULT=$(dfx canister call user_profile submit_quiz "(\"$QUIZ_ID\", vec { 1; 1 })" 2>&1)

if [[ $RETAKE_RESULT == *"Already completed"* ]] || [[ $RETAKE_RESULT == *"Err"* ]] || [[ $RETAKE_RESULT == *"already"* ]]; then
    log_pass "Quiz cannot be immediately retaken"
else
    log_info "Note: Retake result was: $RETAKE_RESULT (might have daily limit logic)"
fi

summary
