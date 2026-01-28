#!/bin/bash

# ============================================================================
# STAGING ASSETS CANISTER TEST SCRIPT
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "STAGING ASSETS CANISTER TEST"

# ============================================================================
# 1. SETUP
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    GOV_ID=$(dfx canister id governance_canister 2>/dev/null)
    if [ -z "$GOV_ID" ]; then GOV_ID=$(dfx identity get-principal); fi

    LEARN_ID=$(dfx canister id learning_engine 2>/dev/null)
    if [ -z "$LEARN_ID" ]; then LEARN_ID=$(dfx identity get-principal); fi

    log_step "Deploying staging_assets..."
    dfx deploy staging_assets --argument "(record {
        governance_canister_id = principal \"$GOV_ID\";
        learning_engine_id = principal \"$LEARN_ID\"
    })"
else
    log_info "Using existing deployment of staging_assets"
    GOV_ID=$(dfx canister id governance_canister 2>/dev/null)
    if [ -z "$GOV_ID" ]; then GOV_ID=$(dfx identity get-principal); fi
fi

STAGING_ID=$(dfx canister id staging_assets 2>/dev/null)
log_info "Staging Assets ID: $STAGING_ID"

# ============================================================================
# 2. STAGE CONTENT
# ============================================================================
log_step "2. Stage Content"

# Define a simple content node
NODE_VEC="vec {
    record {
        id = \"node_1\";
        parent_id = opt \"root\";
        order = 1 : nat32;
        display_type = \"text\";
        title = \"Test Node\";
        description = opt \"A test description $(date +%s)\";
        content = opt \"Some content text\";
        paraphrase = null;
        media = null;
        quiz = null;
        created_at = 1700000000000000000 : nat64;
        updated_at = 1700000000000000000 : nat64;
        version = 1 : nat64;
    }
}"

STAGE_RES=$(dfx canister call staging_assets stage_content "(
    \"Test Title\",
    \"Test Description\",
    $NODE_VEC
)")
log_info "Stage Result: $STAGE_RES"

if echo "$STAGE_RES" | grep -q "Ok"; then
    log_pass "Content staged successfully"
    HASH=$(echo "$STAGE_RES" | sed -n 's/.*Ok = "\(.*\)".*/\1/p')
    log_info "Content Hash: $HASH"
else
    log_fail "Staging failed"
fi

# ============================================================================
# 3. QUERY STAGED
# ============================================================================
log_step "3. Query Staged Content"

INFO=$(dfx canister call staging_assets get_staged_content_info "(\"$HASH\")")
log_info "Staged Info: $INFO"

if echo "$INFO" | grep -q "Test Title"; then
    log_pass "Staged info verified"
else
    log_fail "Staged info mismatch"
fi

# ============================================================================
# 4. ADMIN ACTIONS
# ============================================================================
log_step "4. Admin Actions (Status Change)"

# Only governance can call `mark_loading`. 
# If GOV_ID is our current identity (mock setup), this will work. 
# If GOV_ID is real governance, this will fail.

if [ "$GOV_ID" == "$(dfx identity get-principal)" ]; then
    MARK_RES=$(dfx canister call staging_assets mark_loading "(\"$HASH\")")
    log_info "Mark Loading Result: $MARK_RES"
    
    if echo "$MARK_RES" | grep -q "Ok"; then
        log_pass "Marked as loading (Mock Gov)"
    else
        log_fail "Mark loading failed"
    fi
else
    log_info "Skipping Mark Loading check as current identity is not configured Governance ($GOV_ID)"
fi

summary
