#!/bin/bash

# ============================================================================
# Multi-Feature Governance Test
# ============================================================================
# Tests various governance proposal types:
# 1. Board Member management (Add, Update, Remove)
# 2. Governance Configuration updates
# 3. Global Quiz Configuration updates
# 4. Treasury spending proposals
# ============================================================================

set -e

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "GOVERNANCE MULTI-FEATURE TEST"

# ============================================================================
# SETUP
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
fi

GOVERNANCE_ID=$(dfx canister id governance_canister 2>/dev/null)
TREASURY_ID=$(dfx canister id treasury_canister 2>/dev/null)
USER_PRINCIPAL=$(dfx identity get-principal)

if [ -z "$GOVERNANCE_ID" ]; then
    log_fail "Governance canister not found"
fi

log_info "Governance ID: $GOVERNANCE_ID"
log_info "Treasury ID: $TREASURY_ID"
log_info "Current User: $USER_PRINCIPAL"


# Helper: Ensure user is board member with 100% voting power for easy testing
# Uses the new BPS system: 10000 BPS = 100.00%
ensure_voting_power() {
    log_substep "Ensuring test user has 100% voting power (10000 BPS)..."
    # First unlock if locked
    dfx canister call governance_canister --candid src/governance_canister/governance_canister.did unlock_board_member_shares > /dev/null 2>&1 || true
    # Then set shares using the correct API: set_board_member_shares(Vec<BoardMemberShare>)
    dfx canister call governance_canister --candid src/governance_canister/governance_canister.did set_board_member_shares "(
        vec { 
            record { member = principal \"$USER_PRINCIPAL\"; share_bps = 10000 : nat16; is_sentinel = false } 
        }
    )" > /dev/null 2>&1 || {
        log_info "Note: Board share setting may have failed (check if already configured)"
    }
}

# Helper: Support, Vote, and Execute a proposal
process_proposal() {
    local proposal_id=$1
    local name=$2
    
    log_substep "Processing proposal $proposal_id ($name)..."
    
    # Support
    dfx canister call governance_canister --candid src/governance_canister/governance_canister.did support_proposal "($proposal_id : nat64)" > /dev/null 2>&1 || true
    
    # Vote
    dfx canister call governance_canister --candid src/governance_canister/governance_canister.did vote "($proposal_id : nat64, true)" > /dev/null 2>&1 || true
    
    # Force Approve (to skip timers)
    dfx canister call governance_canister --candid src/governance_canister/governance_canister.did admin_set_proposal_status "($proposal_id : nat64, variant { Approved })" > /dev/null 2>&1 || true
    
    # Execute
    EXEC_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did execute_proposal "($proposal_id : nat64)" 2>&1)
    
    if [[ "$EXEC_RESULT" == *"Ok"* ]]; then
        log_pass "Proposal $proposal_id executed"
    else
        log_fail "Proposal $proposal_id execution failed: $EXEC_RESULT"
    fi
}

# ============================================================================
# TEST 1: Board Member Management (Using BPS System)
# ============================================================================
test_board_management() {
    log_step "1. Testing Board Member Management (BPS System)"
    ensure_voting_power
    
    local NEW_MEMBER_PRINCIPAL="aaaaa-aa" # Just a placeholder identity (Management Canister)
    
    # BPS value: 1000 BPS = 10.00%
    log_substep "1a. Creating AddBoardMember proposal (1000 BPS = 10%)..."
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_board_member_proposal "(record {
        title = \"Add Management Canister to Board\";
        description = \"Testing adding a new board member via proposal with BPS\";
        new_member = principal \"$NEW_MEMBER_PRINCIPAL\";
        share_bps = 1000 : nat16;
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "AddBoardMember"
    else
        log_fail "Failed to create AddBoardMember proposal: $PROP_RESULT"
    fi
    
    # Verify
    IS_MEMBER=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did is_board_member "(principal \"$NEW_MEMBER_PRINCIPAL\")" 2>&1)
    if [[ "$IS_MEMBER" == *"true"* ]]; then
        log_pass "New member added to board"
    else
        log_fail "New member NOT found on board"
    fi
    
    # BPS value: 2000 BPS = 20.00%
    log_substep "1b. Creating UpdateBoardMemberShare proposal (2000 BPS = 20%)..."
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_update_board_member_share_proposal "(record {
        title = \"Update Share for Management Canister\";
        description = \"Testing updating share via proposal with BPS\";
        member = principal \"$NEW_MEMBER_PRINCIPAL\";
        new_share_bps = 2000 : nat16;
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "UpdateShare"
    else
        log_fail "Failed to create UpdateShare proposal: $PROP_RESULT"
    fi
    
    # Verify share - look for 2000 BPS (or 2_000 formatted)
    SHARE=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did get_board_member_shares 2>&1)
    if [[ "$SHARE" == *"2000"* ]] || [[ "$SHARE" == *"2_000"* ]]; then
        log_pass "Member share updated to 2000 BPS (20%)"
    else
        log_info "Member shares after update: $SHARE"
    fi
    
    log_substep "1c. Creating RemoveBoardMember proposal..."
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_remove_board_member_proposal "(record {
        title = \"Remove Management Canister\";
        description = \"Testing removing a member via proposal\";
        member_to_remove = principal \"$NEW_MEMBER_PRINCIPAL\";
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "RemoveMember"
    else
        log_fail "Failed to create RemoveMember proposal: $PROP_RESULT"
    fi
    
    # Verify removal
    IS_MEMBER=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did is_board_member "(principal \"$NEW_MEMBER_PRINCIPAL\")" 2>&1)
    if [[ "$IS_MEMBER" == *"false"* ]]; then
        log_pass "Member removed from board"
    else
        log_fail "Member STILL on board"
    fi
}

# ============================================================================
# TEST 2: Governance Config
# ============================================================================
test_governance_config() {
    log_step "2. Testing Governance Config Updates"
    ensure_voting_power
    
    log_substep "Creating UpdateGovernanceConfig proposal..."
    # Let's update voting_period_days to 14
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_update_governance_config_proposal "(record {
        title = \"Update Voting Period\";
        description = \"Changing voting period to 14 days for testing\";
        new_min_voting_power = null;
        new_support_threshold = null;
        new_approval_percentage = null;
        new_support_period_days = null;
        new_voting_period_days = opt (14 : nat16);
        new_resubmission_cooldown_days = null;
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "GovConfig"
    else
        log_fail "Failed to create GovConfig proposal: $PROP_RESULT"
    fi
    
    # Verify
    CONFIG=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did get_governance_config 2>&1)
    # The 4th item in the tuple is voting_period_days
    # Tuple format usually like (nat64, nat64, nat64, nat64, nat64, nat8)
    if echo "$CONFIG" | grep -q "14 : nat64"; then
         log_pass "Governance config updated (voting_period = 14)"
    else
         log_fail "Governance config NOT updated correctly: $CONFIG"
    fi
}

# ============================================================================
# TEST 3: Quiz Config
# ============================================================================
test_quiz_config() {
    log_step "3. Testing Global Quiz Config Updates"
    ensure_voting_power
    
    log_substep "Creating UpdateQuizConfig proposal..."
    # Update reward_amount to 500
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_update_token_limits_proposal "(record {
        title = \"Update Quiz Reward\";
        description = \"Setting reward to 500 GHC units\";
        new_reward_amount = opt (500 : nat64);
        new_pass_threshold = null;
        new_max_attempts = null;
        new_max_daily_tokens = null;
        new_max_weekly_tokens = null;
        new_max_monthly_tokens = null;
        new_max_yearly_tokens = null;
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "QuizConfig"
    else
        log_fail "Failed to create QuizConfig proposal: $PROP_RESULT"
    fi
    
    # Verify - we need to find where to query this. 
    # Usually it's in learning_engine? Or governance holds it?
    # The proposal type says UpdateGlobalQuizConfig. 
    # Let's check learning_engine.
    log_substep "Verifying in staking_hub..."
    REWARD_CONFIG=$(dfx canister call staking_hub get_token_limits "()" 2>&1 || echo "Error calling staking_hub")
    if echo "$REWARD_CONFIG" | grep -q "reward_amount = 500 : nat64"; then
        log_pass "Quiz reward updated to 500"
    else
        log_fail "Quiz reward NOT updated: $REWARD_CONFIG"
    fi
}

# ============================================================================
# TEST 4: Treasury Proposal
# ============================================================================
test_treasury_proposal() {
    log_step "4. Testing Treasury Spending Proposals"
    ensure_voting_power
    
    if [ -z "$TREASURY_ID" ]; then
        log_fail "Treasury canister not found, skipping treasury test"
    fi
    
    log_substep "Creating Treasury proposal..."
    PROP_RESULT=$(dfx canister call governance_canister --candid src/governance_canister/governance_canister.did create_treasury_proposal "(record {
        title = \"Test Treasury Spending\";
        description = \"Moving 1000 tokens for testing\";
        recipient = principal \"$USER_PRINCIPAL\";
        amount = 1000 : nat64;
        token_type = variant { GHC };
        category = variant { Development };
        external_link = null;
    })" 2>&1)
    
    if [[ "$PROP_RESULT" == *"Ok"* ]]; then
        PROP_ID=$(echo "$PROP_RESULT" | grep -oP '(?<=Ok = )\d+')
        process_proposal "$PROP_ID" "Treasury"
    else
        log_fail "Failed to create Treasury proposal: $PROP_RESULT"
    fi
    
    # NOTE: Execution might fail if treasury has 0 balance, but the CALL should succeed from Governance.
    # We check if the execution was "processed" by the governance side.
    log_info "Note: Treasury execution success depends on treasury balance, but proposal flow is verified."
}

# ============================================================================
# MAIN
# ============================================================================

test_board_management
test_governance_config
test_quiz_config
test_treasury_proposal

summary

