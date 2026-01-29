#!/bin/bash
set -e

# ============================================================================
# COMPREHENSIVE LEARNING ENGINE AUDIT SUITE
# ============================================================================
# This script performs an exhaustive verification of the Learning Engine,
# covering tree structure, public/private quiz data, and versioning.
# ============================================================================

# Colors for Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Log helpers
source "$(dirname "$0")/test_helper.sh"

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
log_header "PHASE 1: Environment Setup"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Initializing clean DFX state"
    dfx stop &>/dev/null || true
    dfx start --background --clean &>/dev/null
    sleep 3

    log_step "Deploying Learning Engine"
    HUB_ID=$(dfx identity get-principal) # Mock Hub ID
    GOV_ID=$(dfx identity get-principal) # Mock Gov ID
    dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$HUB_ID\"; governance_canister_id = opt principal \"$GOV_ID\" })" &>/dev/null

    log_pass "Learning Engine deployed"
else
    log_info "Using existing deployment"
fi

ADMIN_IDENTITY=$(dfx identity whoami)

# ============================================================================
# PHASE 2: TREE STRUCTURE & CONTENT
# ============================================================================
log_header "PHASE 2: Tree Structure & Content"

log_step "Verifying authorization (Security Check)"
dfx identity new attacker --storage-mode=plaintext &>/dev/null || true
ATTACK_RES=$(dfx --identity attacker canister call learning_engine add_content_node '(record { id="hack"; parent_id=null; order=1; display_type="Book"; title="Hacked"; description=null; content=null; paraphrase=null; media=null; quiz=null; created_at=0; updated_at=0; version=1 })' 2>&1 || true)
if [[ "$ATTACK_RES" == *"Unauthorized"* ]]; then
    log_pass "Security: Non-admin cannot add content"
else
    log_fail "Security check failed: $ATTACK_RES"
fi

log_step "Creating a Book -> Chapter -> Unit hierarchy"
# ROOT: Book
dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node '(record { id="book_1"; parent_id=null; order=1; display_type="Book"; title="Climate Change 101"; description=null; content=opt "Overview of climate science"; paraphrase=null; media=null; quiz=null; created_at=0; updated_at=0; version=1 })' &>/dev/null
# CHILD: Chapter 1
dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node '(record { id="ch_1"; parent_id=opt "book_1"; order=1; display_type="Chapter"; title="The Greenhouse Effect"; description=null; content=null; paraphrase=null; media=null; quiz=null; created_at=0; updated_at=0; version=1 })' &>/dev/null
# GRANDCHILD: Unit 1.1 with Quiz
dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node '(record { 
    id="unit_1.1"; 
    parent_id=opt "ch_1"; 
    order=1; 
    display_type="Unit"; 
    title="CO2 and Heat"; 
    description=null; 
    content=opt "CO2 traps heat in the atmosphere."; 
    paraphrase=null; 
    media=null; 
    quiz=opt record { 
        questions=vec { record { question="Is CO2 a GHG?"; options=vec {"Yes";"No"}; answer=0:nat8 } } 
    }; 
    created_at=0; 
    updated_at=0; 
    version=1 
})' &>/dev/null

log_step "Verifying Root Nodes lookup"
ROOTS=$(dfx canister call learning_engine get_root_nodes)
if [[ "$ROOTS" == *"Climate Change 101"* ]]; then
    # Verify ch_1 is NOT a root (it should be a child of book_1)
    if [[ "$ROOTS" == *"The Greenhouse Effect"* ]]; then
        log_fail "Hierarchy error: ch_1 (Greenhouse) detected as a root node"
    else
        log_pass "Hierarchy correctly identified (Roots isolated)"
    fi
else
    log_fail "Root nodes lookup failed: book_1 (Climate Change 101) not found in $ROOTS"
fi

log_step "Verifying Children traversal"
CHILDREN=$(dfx canister call learning_engine get_children '("ch_1")')
if [[ "$CHILDREN" == *"CO2 and Heat"* ]]; then
    log_pass "Tree traversal successful (Book -> Chapter -> Unit)"
else
    log_fail "Children traversal failed for ch_1: $CHILDREN"
fi

# ============================================================================
# PHASE 3: PUBLIC VS PRIVATE QUIZ DATA
# ============================================================================
log_header "PHASE 3: Public vs Private Quiz Data"

log_step "Querying Public Node (should omit answers)"
PUBLIC_NODE=$(dfx canister call learning_engine get_content_node '("unit_1.1")')
if [[ "$PUBLIC_NODE" == *"answer"* ]]; then
    log_fail "Security Breach: Public query returned quiz answers!"
else
    log_pass "Quiz answers correctly hidden in public queries"
fi

log_step "Verifying Quiz Cache Data (for shards)"
CACHE_DATA=$(dfx canister call learning_engine get_quiz_data '("unit_1.1")')
if [[ "$CACHE_DATA" == *"answer_hashes"* && "$CACHE_DATA" != *"answer ="* ]]; then
    log_pass "Quiz cache data correctly exposed for shards (hashes only)"
else
    log_fail "Quiz cache data missing or leaking answers: $CACHE_DATA"
fi

log_step "Verifying direct Quiz Verification"
# Correct answer is 0
PASSED=$(dfx canister call learning_engine verify_quiz '("unit_1.1", blob "\00")')
if [[ "$PASSED" =~ "true" ]] && [[ "$PASSED" =~ "1" ]]; then
    log_pass "Quiz verification logic sound (Correct answers)"
else
    log_fail "Quiz verification failed for correct answer: $PASSED"
fi

FAILED=$(dfx canister call learning_engine verify_quiz '("unit_1.1", blob "\01")')
if [[ "$FAILED" =~ "false" ]] && [[ "$FAILED" =~ "0" ]] && [[ "$FAILED" =~ "1" ]]; then
    log_pass "Quiz verification logic sound (Incorrect answers)"
else
    log_fail "Quiz verification failed to reject incorrect answer: $FAILED"
fi

# ============================================================================
# PHASE 4: VERSIONING & AUDIT TRAIL
# ============================================================================
log_header "PHASE 4: Versioning & Audit Trail"

log_step "Updating a content node (Creating New Version)"
# Get current version first. Output format: ( N : nat64 )
RAW_V=$(dfx canister call learning_engine get_content_current_version '("book_1")')
CURRENT_V=$(echo "$RAW_V" | grep -oP '^\(\s*\K\d+' || echo "0")
NEXT_V=$((CURRENT_V + 1))
log_info "Current version: $CURRENT_V, Updating to: $NEXT_V"

dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node "(record { id=\"book_1\"; parent_id=null; order=1; display_type=\"Book\"; title=\"Climate Change 101 (Revised $NEXT_V)\"; description=null; content=opt \"Updated content v$NEXT_V\"; paraphrase=null; media=null; quiz=null; created_at=0; updated_at=0; version=$NEXT_V })" &>/dev/null

log_step "Verifying Current Version"
NEW_CUR_VER=$(dfx canister call learning_engine get_content_current_version '("book_1")')
if [[ "$NEW_CUR_VER" == *"($NEXT_V : nat64)"* ]]; then
    log_pass "Current version incremented to $NEXT_V"
else
    log_fail "Version did not increment to $NEXT_V. Got: $NEW_CUR_VER"
fi

log_step "Retrieving Historical Version (Version 1)"
HIST_NODE=$(dfx canister call learning_engine get_content_at_version '("book_1", 1)')
if [[ "$HIST_NODE" == *"Climate Change 101"* && "$HIST_NODE" != *"Revised"* ]]; then
    log_pass "Historical snapshot retrieval successful (Audit Trail Intact)"
else
    log_fail "Historical version retrieval failed or corrupted: $HIST_NODE"
fi

log_step "Checking Version History List"
HISTORY=$(dfx canister call learning_engine get_content_version_history '("book_1")')
if [[ "$HISTORY" == *"modified_by_proposal = 0"* ]]; then
     log_pass "Version history list correctly populated"
else
     log_fail "Version history list empty or incorrect: $HISTORY"
fi

# ============================================================================
# PHASE 5: STATISTICS
# ============================================================================
log_header "PHASE 5: Statistics"

log_step "Querying Content Statistics"
STATS=$(dfx canister call learning_engine get_content_stats)
# As many modules add content, we just verify they are non-zero and present
if [[ "$STATS" != *"(0 : nat64, 0 : nat64)"* ]]; then
    log_pass "Content statistics available and non-zero: $STATS"
else
    log_fail "Content statistics empty or zero: $STATS"
fi

# ============================================================================
# SUMMARY
# ============================================================================
log_header "AUDIT SUMMARY"

echo -e "  - Total Checks: $TESTS_TOTAL"
echo -e "  - Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "  - Failed: ${RED}$((TESTS_TOTAL - TESTS_PASSED))${NC}"
echo ""

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  LEARNING ENGINE VERIFIED COMPREHENSIVE - AUDIT READY ✓${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}  AUDIT FAILED - CRITICAL ISSUES DETECTED ✗${NC}"
    exit 1
fi
