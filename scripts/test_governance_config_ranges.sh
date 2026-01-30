#!/bin/bash
set -e

# ============================================================================
# GOVERNANCE CONFIGURATION RANGE AUDIT
# ============================================================================
# Verifies that Approval Threshold (percentage) can only be changed within
# the allowed range of 30% to 51%.
# ============================================================================

# Colors for Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Log helpers
source "$(dirname "$0")/test_helper.sh"

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Infrastructure"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
    log_pass "Infrastructure deployed"
fi

GOV_ID=$(dfx canister id governance_canister)
USER_P=$(dfx identity get-principal)

if [ -z "$GOV_ID" ]; then
    log_fail "Infrastructure not deployed properly (GOV_ID missing)"
fi

# ============================================================================
# PHASE 2: INITIAL BOARD SETUP
# ============================================================================
log_header "PHASE 2: Initial Board Setup"

log_step "Setting initial board members (Controller Only)"
# Ensure we are controller
dfx canister update-settings governance_canister --add-controller "$USER_P" &>/dev/null || true

RES=$(dfx canister call governance_canister set_board_member_shares "(vec { record { member = principal \"$USER_P\"; share_bps = 10000 : nat16; is_sentinel = false } })" --candid src/governance_canister/governance_canister.did 2>&1 || true)
if [[ "$RES" == *"Ok"* ]]; then
    log_pass "Initial board configuration successful"
elif [[ "$RES" == *"locked"* ]]; then
    log_info "Board shares already locked (idempotent)"
else
    log_fail "Board configuration failed: $RES"
fi

log_step "Setting sentinel member (Required for locking)"
SENTINEL_P="rrkah-fqaaa-aaaaa-aaaaq-cai"
dfx canister call governance_canister set_sentinel_member "(principal \"$SENTINEL_P\")" --candid src/governance_canister/governance_canister.did &>/dev/null || true

log_step "Locking board member shares"
dfx canister call governance_canister lock_board_member_shares --candid src/governance_canister/governance_canister.did &>/dev/null || true
log_pass "Board configuration locked"

# ============================================================================
# PHASE 3: RANGE VALIDATION (PROPOSAL CREATION)
# ============================================================================
log_header "PHASE 3: Range Validation"

log_step "Verifying Out-of-Range Approval Threshold (Too Low: 29%)"
FAIL_LOW=$(dfx canister call governance_canister create_update_governance_config_proposal '(record { 
    title="Invalid Config - Low"; 
    description="Attempting to set threshold to 29%"; 
    new_min_voting_power=null;
    new_support_threshold=null;
    new_approval_percentage=opt 29;
    new_support_period_days=null;
    new_voting_period_days=null;
    new_resubmission_cooldown_days=null;
    external_link=null 
})' 2>&1 || true)

if [[ "$FAIL_LOW" == *"Approval percentage must be between 30 and 50"* ]]; then
    log_pass "Correctly blocked threshold update below 30%"
else
    log_fail "Failed to block low threshold: $FAIL_LOW"
fi
FAIL_HIGH=$(dfx canister call governance_canister create_update_governance_config_proposal '(record { 
    title="Invalid Config - High"; 
    description="Attempting to set threshold to 51%"; 
    new_min_voting_power=null;
    new_support_threshold=null;
    new_approval_percentage=opt 51;
    new_support_period_days=null;
    new_voting_period_days=null;
    new_resubmission_cooldown_days=null;
    external_link=null 
})' 2>&1 || true)

if [[ "$FAIL_HIGH" == *"Approval percentage must be between 30 and 50"* ]]; then
    log_pass "Correctly blocked threshold update above 50%"
else
    log_fail "Failed to block high threshold: $FAIL_HIGH"
fi

log_step "Verifying Valid Approval Threshold (50%)"
PROP_ID=$(dfx canister call governance_canister create_update_governance_config_proposal '(record { 
    title="Max Config - 50%"; 
    description="Setting threshold to 50%"; 
    new_min_voting_power=null;
    new_support_threshold=null;
    new_approval_percentage=opt 50;
    new_support_period_days=null;
    new_voting_period_days=null;
    new_resubmission_cooldown_days=null;
    external_link=null 
})' | grep -oP 'Ok = \K\d+' | head -1)

if [ -n "$PROP_ID" ]; then
    log_pass "Successfully created proposal for 50% threshold (ID: $PROP_ID)"
else
    log_fail "Failed to create valid proposal (50%)"
fi

# ============================================================================
# PHASE 4: EXECUTION & VERIFICATION
# ============================================================================
log_header "PHASE 4: Execution & Verification"

log_step "Approving Proposal (Board YES Vote)"
dfx canister call governance_canister vote "($PROP_ID, true)" &>/dev/null
log_pass "Board vote recorded"

log_step "Finalizing and Executing Proposal"
dfx canister call governance_canister admin_expire_proposal "($PROP_ID)" &>/dev/null
dfx canister call governance_canister finalize_proposal "($PROP_ID)" &>/dev/null
EXEC=$(dfx canister call governance_canister execute_proposal "($PROP_ID)" 2>&1)

if [[ "$EXEC" == *"Ok"* ]]; then
    log_pass "Proposal executed successfully"
else
    log_fail "Execution failed: $EXEC"
fi

log_step "Verifying Configuration Update"
CONFIG=$(dfx canister call governance_canister get_governance_config)
if [[ "$CONFIG" == *"50"* ]]; then
    log_pass "Approval percentage verified as 50% in state"
else
    log_fail "Configuration state mismatch (Expected 50): $CONFIG"
fi

log_header "PHASE 5: Verified +1 Logic"

log_step "Checking Required YES Votes for 50% Proposal"
# Fetch total staked to predict. 
# Our board member has 100% board power, but we need total STAKED from staking hub.
# In deploy_full.sh, we fund the Hub with 4.75B tokens.
# The calculate_approval_threshold calls Hub get_global_stats.
# Let's see what the proposal required_yes_votes is.
STAKED=$(dfx canister call staking_hub get_global_stats | grep -oP 'total_staked = \K[\d_]+' | tr -d '_')
EXPECTED=$(( (STAKED / 2) + 1 ))

# Create a new proposal now that config is 50%
NEW_PROP_ID=$(dfx canister call governance_canister create_treasury_proposal '(record { 
    title="Test 50+1"; 
    description="Check threshold"; 
    recipient=principal "2vxsx-fae"; 
    amount=100; 
    token_type=variant { GHC }; 
    category=variant { Marketing }; 
    external_link=null 
})' | grep -oP 'Ok = \K\d+' | head -1)

REQ_VOTES=$(dfx canister call governance_canister get_proposal "($NEW_PROP_ID)" | grep -oP 'required_yes_votes = \K[\d_]+' | tr -d '_')

if [ "$REQ_VOTES" -eq "$EXPECTED" ]; then
    log_pass "50% + 1 logic verified: Total Staked=$STAKED, Required=$REQ_VOTES"
else
    log_fail "50% + 1 logic failure: Total Staked=$STAKED, Expected=$EXPECTED, Got=$REQ_VOTES"
fi

summary
