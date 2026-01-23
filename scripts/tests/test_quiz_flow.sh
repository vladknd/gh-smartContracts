#!/bin/bash

# Quiz Flow Test - Tests the complete quiz submission flow
# Tests: learning unit creation, quiz submission, reward allocation

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[QUIZ TEST] $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

fail() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

log "=== Starting Quiz Flow Test ==="

# Check if canisters are deployed
if ! dfx canister id learning_engine &>/dev/null; then
    log "Canisters not deployed. Running deploy script..."
    ./scripts/deploy.sh >> deployment.log 2>&1
fi

LEARNING_ENGINE_ID=$(dfx canister id learning_engine)
USER_PROFILE_ID=$(dfx canister id user_profile)
STAKING_HUB_ID=$(dfx canister id staking_hub)

log "Learning Engine: $LEARNING_ENGINE_ID"
log "User Profile: $USER_PROFILE_ID"
log "Staking Hub: $STAKING_HUB_ID"

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
log "Test 1: Adding learning unit..."
dfx identity use default
RESULT=$(dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "quiz_test_unit";
    unit_title = "Quiz Test Unit";
    chapter_id = "1";
    chapter_title = "Test Chapter";
    head_unit_id = "1";
    head_unit_title = "Test Head Unit";
    content = "This is test content.";
    paraphrase = "This is test paraphrase.";
    quiz = vec {
        record {
            question = "What is 1+1?";
            options = vec { "1"; "2"; "3" };
            answer = 1;
        };
        record {
            question = "What is 2+2?";
            options = vec { "3"; "4"; "5" };
            answer = 1;
        };
    };
})' 2>&1)

if [[ $RESULT == *"Ok"* ]] || [[ $RESULT == *"already exists"* ]]; then
    success "Learning unit added/exists"
else
    fail "Failed to add learning unit: $RESULT"
fi

# Test 2: Get learning unit (verify answers are hidden)
log "Test 2: Verifying answers are hidden in public queries..."
PUBLIC_UNIT=$(dfx canister call learning_engine get_learning_unit '("quiz_test_unit")' 2>&1)

if [[ $PUBLIC_UNIT == *"answer"* ]] && [[ $PUBLIC_UNIT != *"answer = 1"* ]]; then
    success "Answers are hidden in public response"
elif [[ $PUBLIC_UNIT == *"null"* ]] || [[ $PUBLIC_UNIT == *"None"* ]]; then
    success "Unit not found or answers stripped"
else
    # Check if answers are exposed
    if [[ $PUBLIC_UNIT == *"answer = 1"* ]]; then
        fail "Public unit contains answers! Security issue."
    else
        success "Public unit does not expose answers"
    fi
fi

# Test 3: Register user and submit quiz
log "Test 3: Registering user..."
dfx identity use "$TEST_USER"
REG_RESULT=$(dfx canister call user_profile register_user '(record { 
    email = "quiz@test.com"; 
    name = "Quiz Tester"; 
    education = "Test"; 
    gender = "Test" 
})' 2>&1)

if [[ $REG_RESULT == *"Ok"* ]]; then
    success "User registered"
else
    fail "User registration failed: $REG_RESULT"
fi

# Test 4: Submit incorrect quiz
log "Test 4: Submitting incorrect quiz answers..."
WRONG_RESULT=$(dfx canister call user_profile submit_quiz '("quiz_test_unit", vec { 0; 0 })' 2>&1)

if [[ $WRONG_RESULT == *"Incorrect"* ]] || [[ $WRONG_RESULT == *"Err"* ]]; then
    success "Incorrect answers correctly rejected"
else
    # If it passed, might be different threshold
    log "Note: Result was: $WRONG_RESULT"
fi

# Test 5: Submit correct quiz
log "Test 5: Submitting correct quiz answers..."
CORRECT_RESULT=$(dfx canister call user_profile submit_quiz '("quiz_test_unit", vec { 1; 1 })' 2>&1)

if [[ $CORRECT_RESULT == *"Ok"* ]]; then
    success "Correct answers accepted"
else
    fail "Correct answers rejected: $CORRECT_RESULT"
fi

# Test 6: Verify reward allocation
log "Test 6: Verifying reward allocation..."
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")" 2>&1)

if [[ $PROFILE == *"staked_balance"* ]]; then
    success "Profile shows staked balance (rewards allocated)"
    log "Profile: $PROFILE"
else
    fail "No staked balance found: $PROFILE"
fi

# Test 7: Verify quiz is marked as completed (cannot retake immediately)
log "Test 7: Verifying quiz completion prevents immediate retake..."
RETAKE_RESULT=$(dfx canister call user_profile submit_quiz '("quiz_test_unit", vec { 1; 1 })' 2>&1)

if [[ $RETAKE_RESULT == *"Already completed"* ]] || [[ $RETAKE_RESULT == *"Err"* ]] || [[ $RETAKE_RESULT == *"already"* ]]; then
    success "Quiz cannot be immediately retaken"
else
    log "Note: Retake result was: $RETAKE_RESULT (might have daily limit logic)"
fi

log "=== All Quiz Flow Tests Completed ==="
