#!/bin/bash

# ============================================================================
# GHC PRODUCTION AUDIT TEST SUITE
# ============================================================================
# This script performs a full system verification using the production
# deployment architecture (Archives, Shards, Mixed Limits, Governance).
#
# Sections:
# 1. Clean Deployment (with Archives)
# 2. Governance Setup (Token Limits & Content)
# 3. User Subscription Flow
# 4. Archiving Pressure Test (150+ transactions)
# 5. History Retrieval & Pruning Verification
# ============================================================================

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "PRODUCTION ARCHIVING AUDIT"

# ============================================================================
# 1. CLEAN DEPLOYMENT
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System with Archives"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
fi


# Get IDs
USER_PROFILE_ID=$(dfx canister id user_profile)
ARCHIVE_ID=$(dfx canister id archive_canister)
STAKING_HUB_ID=$(dfx canister id staking_hub)
GOVERNANCE_ID=$(dfx canister id governance_canister)
DEFAULT_PRINCIPAL=$(dfx identity get-principal)

log_pass "System deployed with Archive link: $USER_PROFILE_ID <-> $ARCHIVE_ID"

# ============================================================================
# 2. GOVERNANCE SETUP
# ============================================================================
log_step "Governance Setup (Tokens & Content)"

# 2a. Setup voting power
log_info "Setting up Board Member permissions..."
dfx canister call governance_canister set_board_member_shares "(vec { record { member = principal \"$DEFAULT_PRINCIPAL\"; percentage = 100 : nat8 } })" >/dev/null

# 2b. Enable high activity (Update Token Limits via Proposal)
log_info "Proposing higher daily attempts (200) for stress testing..."
PROPOSAL_RESULT=$(dfx canister call governance_canister create_update_token_limits_proposal "(record {
    title = \"Audit Stress Test Config\";
    description = \"Increase daily attempts to 200 and reward to 100 GHC for testing.\";
    new_reward_amount = opt (10_000_000_000 : nat64);
    new_pass_threshold = opt (80 : nat8);
    new_max_attempts = opt (200 : nat8);
    new_regular_limits = opt record {
        max_daily_tokens = 100_000_000_000 : nat64;
        max_weekly_tokens = 500_000_000_000 : nat64;
        max_monthly_tokens = 2_000_000_000_000 : nat64;
        max_yearly_tokens = 10_000_000_000_000 : nat64;
    };
    new_subscribed_limits = opt record {
        max_daily_tokens = 2_000_000_000_000 : nat64;
        max_weekly_tokens = 10_000_000_000_000 : nat64;
        max_monthly_tokens = 40_000_000_000_000 : nat64;
        max_yearly_tokens = 500_000_000_000_000 : nat64;
    };
    external_link = null;
})" 2>&1)

PROP_ID=$(echo "$PROPOSAL_RESULT" | grep -oP '(?<=Ok = )\d+')
log_info "Voting and executing Proposal #$PROP_ID..."
dfx canister call governance_canister support_proposal "($PROP_ID : nat64)" >/dev/null
dfx canister call governance_canister vote "($PROP_ID : nat64, true)" >/dev/null
dfx canister call governance_canister admin_set_proposal_status "($PROP_ID : nat64, variant { Approved })" >/dev/null
dfx canister call governance_canister execute_proposal "($PROP_ID : nat64)" >/dev/null

log_pass "New token limits active in Staking Hub and User Profile Shard"

# ============================================================================
# 3. SUBSCRIPTION FEATURE FLOW
# ============================================================================
log_step "User Subscription Flow Verification"

# Setup a test user
log_info "Registering audit test user..."
dfx identity new audit_user --storage-mode plaintext 2>/dev/null || true
dfx identity use audit_user
USER_PRINCIPAL=$(dfx identity get-principal)
dfx canister call user_profile register_user "(record { email = \"audit_$(date +%s)@ghc.com\"; name = \"Audit Tester\"; education = \"QA\"; gender = \"Other\" })"

# Check regular limit (1000 GHC)
log_info "Hitting regular token limit (1000 GHC)..."
dfx identity use default
for i in $(seq 1 11); do
    TEMP_ID="unit_$i"
    dfx canister call learning_engine add_content_node "(record { id = \"$TEMP_ID\"; parent_id = null; order = 1; display_type = \"UNIT\"; title = \"Unit $i\"; description = null; content = null; paraphrase = null; media = null; quiz = opt record { questions = vec { record { question = \"1+1?\"; options = vec { \"2\"; \"3\" }; answer = 0 } } }; created_at = 0; updated_at = 0; version = 1; })" >/dev/null
    
    if [ $i -le 10 ]; then
        dfx identity use audit_user
        RES=$(dfx canister call user_profile submit_quiz "(\"$TEMP_ID\", vec {0})" 2>&1)
        if [[ ! "$RES" == *"Ok"* ]]; then log_fail "Quiz $i failed: $RES"; fi
        dfx identity use default
    fi
done

# 11th should fail
dfx identity use audit_user
RES=$(dfx canister call user_profile submit_quiz "(\"unit_11\", vec {0})" 2>&1)
if [[ "$RES" == *"Daily token limit reached"* ]]; then
    log_pass "Regular limit correctly blocked user at 1000 GHC"
else
    log_fail "User was NOT blocked at limit: $RES"
fi

# Upgrade to Subscribed
log_info "Upgrading user to SUBSCRIBED..."
dfx identity use default
dfx canister call user_profile admin_set_subscription "(principal \"$USER_PRINCIPAL\", true)"

# 11th should now work
dfx identity use audit_user
RES=$(dfx canister call user_profile submit_quiz "(\"unit_11\", vec {0})" 2>&1)
if [[ "$RES" == *"Ok"* ]]; then
    log_pass "Subscription upgrade allowed user to exceed regular limit"
else
    log_fail "User still blocked after upgrade: $RES"
fi

# ============================================================================
# 4. ARCHIVING PRESSURE TEST
# ============================================================================
log_step "Archiving Pressure Test (Threshold: 150 Transactions)"

log_info "Generating ~150 transactions to trigger archiving..."
# We already have ~11 transactions. We need 140 more.
# To speed up, we'll use a loop and many small units.
dfx identity use default
for b in $(seq 12 160); do
    if [ $((b % 20)) -eq 0 ]; then echo "  Processing batch $b..."; fi
    TEMP_ID="stress_$b"
    dfx canister call learning_engine add_content_node "(record { id = \"$TEMP_ID\"; parent_id = null; order = 1; display_type = \"UNIT\"; title = \"Stress $b\"; description = null; content = null; paraphrase = null; media = null; quiz = opt record { questions = vec { record { question = \"1+1?\"; options = vec { \"2\"; \"3\" }; answer = 0 } } }; created_at = 0; updated_at = 0; version = 1; })" >/dev/null
    
    dfx identity use audit_user
    dfx canister call user_profile submit_quiz "(\"$TEMP_ID\", vec {0})"
    dfx identity use default
done

COUNT=$(dfx canister call user_profile get_profile "(principal \"$USER_PRINCIPAL\")" | grep "transaction_count" | grep -oP '\d+' | head -1)
log_info "Total transactions generated: $COUNT"

# Trigger archiving
log_info "Manually triggering archiving event..."
dfx canister call user_profile debug_trigger_archive >/dev/null

# ============================================================================
# 5. VERIFICATION
# ============================================================================
log_step "Final Production Audit Verification"

# 5a. Check Archive Stats
log_info "Verifying Archive Canister storage..."
ARCHIVE_STATS=$(dfx canister call archive_canister get_stats)
ENTRY_COUNT=$(echo "$ARCHIVE_STATS" | grep "entry_count" | grep -oP '\d+' | head -1)

if [ "$ENTRY_COUNT" -gt 0 ]; then
    log_pass "Archive canister now holds $ENTRY_COUNT transactions"
else
    log_info "Archive count is 0. Attempting one more trigger..."
    dfx canister call user_profile debug_trigger_archive >/dev/null
    ARCHIVE_STATS=$(dfx canister call archive_canister get_stats)
    ENTRY_COUNT=$(echo "$ARCHIVE_STATS" | grep "entry_count" | grep -oP '\d+' | head -1)
    if [ "$ENTRY_COUNT" -gt 0 ]; then
        log_pass "Archive canister now holds $ENTRY_COUNT transactions"
    else
        log_fail "Archiving failed to move data. Stats: $ARCHIVE_STATS"
    fi
fi

# 5b. Verify Pruning (User Profile local storage should be reduced)
log_info "Verifying Shard pruning..."
LOCAL_TX_COUNT=$(dfx canister call user_profile get_user_transactions "(principal \"$USER_PRINCIPAL\")" | grep "record" | wc -l)
log_info "Local transactions remaining: $LOCAL_TX_COUNT"
if [ "$LOCAL_TX_COUNT" -lt "$COUNT" ]; then
    log_pass "Shard correctly pruned transactions after archiving"
else
    log_fail "Shard did not prune transactions (Local: $LOCAL_TX_COUNT, Total: $COUNT)"
fi

# 5c. Verify Integrated Retrieval
log_info "Verifying integrated history retrieval (Page 0)..."
PAGE_RESULT=$(dfx canister call user_profile get_transactions_page "(principal \"$USER_PRINCIPAL\", 0)")
if [[ "$PAGE_RESULT" == *"archived_count"* ]]; then
    log_pass "get_transactions_page correctly identifies archived history"
else
    log_fail "Integrated retrieval failed to show archive info"
fi

summary

# Cleanup
dfx identity use default
dfx identity remove audit_user 2>/dev/null || true
