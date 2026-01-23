#!/bin/bash

# ============================================================================
# GHC Content Governance Test
# Tests the full content proposal workflow:
# 1. Stage content → 2. Create proposal → 3. Vote → 4. Execute → 5. Verify
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "============================================================================"
echo "       GHC Content Governance Proposal Test"
echo "============================================================================"
echo -e "${NC}"

# Helper functions
log_step() {
    echo -e "\n${YELLOW}>>> Step: $1${NC}"
}

log_result() {
    local status=$1
    local message=$2
    if [ "$status" == "PASS" ]; then
        echo -e "${GREEN}✅ PASS: $message${NC}"
    else
        echo -e "${RED}❌ FAIL: $message${NC}"
    fi
}

# ============================================================================
# SETUP: Get canister IDs and check prerequisites
# ============================================================================
log_step "0. Setup - Verify canisters are running"

STAGING_ID=$(dfx canister id staging_assets 2>/dev/null)
GOVERNANCE_ID=$(dfx canister id governance_canister 2>/dev/null)
LEARNING_ENGINE_ID=$(dfx canister id learning_engine 2>/dev/null)
USER_PRINCIPAL=$(dfx identity get-principal)

if [ -z "$STAGING_ID" ] || [ -z "$GOVERNANCE_ID" ] || [ -z "$LEARNING_ENGINE_ID" ]; then
    echo -e "${RED}Error: Required canisters not found. Run ./scripts/deploy.sh first.${NC}"
    exit 1
fi

echo "Staging Assets: $STAGING_ID"
echo "Governance: $GOVERNANCE_ID"
echo "Learning Engine: $LEARNING_ENGINE_ID"
echo "Current User: $USER_PRINCIPAL"

# ============================================================================
# STEP 1: Prepare content nodes
# ============================================================================
log_step "1. Preparing test content"

# Note: We use hardcoded test content below for reliability
# If you want to load from sample_curriculum.json, install jq:
#   sudo apt install jq

echo "Using hardcoded test content (3 nodes: 1 chapter + 2 units)"
echo "For full curriculum loading from JSON, install jq and modify this script."

# ============================================================================
# STEP 2: Stage content to staging_assets canister
# ============================================================================
log_step "2. Staging content to staging_assets canister"

# First, make sure the current user is an allowed stager
echo "Adding current user as allowed stager..."
ADD_STAGER_RESULT=$(dfx canister call staging_assets add_allowed_stager "(principal \"$USER_PRINCIPAL\")" 2>&1)
echo "Add stager result: $ADD_STAGER_RESULT"

# Generate unique timestamp for this test run
TIMESTAMP=$(date +%s)
echo "Using timestamp: $TIMESTAMP for unique content IDs"

# Build a simplified content set for testing (just 3 nodes to speed up test)
# Using timestamp in IDs to ensure uniqueness
SIMPLE_CONTENT="vec {
    record {
        id = \"test_gov_ch_${TIMESTAMP}\";
        parent_id = null;
        order = 1 : nat32;
        display_type = \"CHAPTER\";
        title = \"Test Governance Chapter ${TIMESTAMP}\";
        description = opt \"Testing content governance flow\";
        content = null;
        paraphrase = null;
        media = null;
        quiz = null;
        created_at = 0 : nat64;
        updated_at = 0 : nat64;
        version = 1 : nat64;
    };
    record {
        id = \"test_gov_unit1_${TIMESTAMP}\";
        parent_id = opt \"test_gov_ch_${TIMESTAMP}\";
        order = 1 : nat32;
        display_type = \"UNIT\";
        title = \"Governance Test Unit 1\";
        description = opt \"First test unit\";
        content = opt \"This is test content for the governance flow.\";
        paraphrase = opt \"Test content for governance approval flow.\";
        media = null;
        quiz = opt record { questions = vec { 
            record { question = \"Is this a governance test?\"; options = vec { \"Yes\"; \"No\" }; answer = 0 : nat8 };
        }};
        created_at = 0 : nat64;
        updated_at = 0 : nat64;
        version = 1 : nat64;
    };
    record {
        id = \"test_gov_unit2_${TIMESTAMP}\";
        parent_id = opt \"test_gov_ch_${TIMESTAMP}\";
        order = 2 : nat32;
        display_type = \"UNIT\";
        title = \"Governance Test Unit 2\";
        description = opt \"Second test unit\";
        content = opt \"Second unit content for governance testing.\";
        paraphrase = opt \"Second test unit summary.\";
        media = null;
        quiz = opt record { questions = vec { 
            record { question = \"How many units in this test?\"; options = vec { \"1\"; \"2\"; \"3\" }; answer = 1 : nat8 };
        }};
        created_at = 0 : nat64;
        updated_at = 0 : nat64;
        version = 1 : nat64;
    }
}"

echo "Staging content..."
STAGE_RESULT=$(dfx canister call staging_assets stage_content \
    "(\"Governance Test Content\", \"Testing content governance proposal flow\", $SIMPLE_CONTENT)" 2>&1)

echo "Stage result: $STAGE_RESULT"

if [[ "$STAGE_RESULT" == *"Ok"* ]]; then
    # Extract content hash
    CONTENT_HASH=$(echo "$STAGE_RESULT" | grep -oP '(?<=Ok = ")[^"]+')
    log_result "PASS" "Content staged successfully. Hash: $CONTENT_HASH"
else
    log_result "FAIL" "Failed to stage content: $STAGE_RESULT"
    exit 1
fi

# Verify staged content
log_step "2b. Verifying staged content"
STAGED_INFO=$(dfx canister call staging_assets get_staged_content_info "(\"$CONTENT_HASH\")" 2>&1)
echo "Staged content info: $STAGED_INFO"

if [[ "$STAGED_INFO" == *"Pending"* ]]; then
    log_result "PASS" "Staged content is in Pending status"
else
    log_result "FAIL" "Unexpected staging status: $STAGED_INFO"
fi

# ============================================================================
# STEP 3: Create content proposal through governance
# ============================================================================
log_step "3. Creating content proposal through governance canister"

PROPOSAL_RESULT=$(dfx canister call governance_canister create_add_content_proposal "(record {
    title = \"Add Governance Test Content\";
    description = \"Proposal to add test content for governance flow testing. Contains 1 chapter and 2 units.\";
    staging_canister = principal \"$STAGING_ID\";
    staging_path = \"$CONTENT_HASH\";
    content_hash = \"$CONTENT_HASH\";
    content_title = \"Governance Test Content\";
    unit_count = 3 : nat32;
    external_link = null;
})" 2>&1)

echo "Proposal result: $PROPOSAL_RESULT"

if [[ "$PROPOSAL_RESULT" == *"Ok"* ]]; then
    PROPOSAL_ID=$(echo "$PROPOSAL_RESULT" | grep -oP '(?<=Ok = )\d+')
    log_result "PASS" "Content proposal created. Proposal ID: $PROPOSAL_ID"
else
    log_result "FAIL" "Failed to create proposal: $PROPOSAL_RESULT"
    exit 1
fi

# Check proposal status
log_step "3b. Checking proposal status"
PROPOSAL_INFO=$(dfx canister call governance_canister get_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Proposal info (truncated): ${PROPOSAL_INFO:0:500}..."

if [[ "$PROPOSAL_INFO" == *"AddContentFromStaging"* ]]; then
    log_result "PASS" "Proposal is of type AddContentFromStaging"
else
    log_result "FAIL" "Proposal type mismatch"
fi

# ============================================================================
# STEP 4: Support and Vote on proposal
# ============================================================================
log_step "4. Supporting the proposal (to move to Active)"

SUPPORT_RESULT=$(dfx canister call governance_canister support_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Support result: $SUPPORT_RESULT"

if [[ "$SUPPORT_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Proposal supported"
else
    echo "Note: Support might require voting power. Checking if we can vote..."
fi

log_step "4b. Voting YES on the proposal"

VOTE_RESULT=$(dfx canister call governance_canister vote "($PROPOSAL_ID : nat64, true)" 2>&1)
echo "Vote result: $VOTE_RESULT"

if [[ "$VOTE_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Vote cast successfully"
else
    log_result "FAIL" "Failed to vote: $VOTE_RESULT"
fi

# Check updated proposal status
PROPOSAL_AFTER_VOTE=$(dfx canister call governance_canister get_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Proposal after vote (status): $(echo "$PROPOSAL_AFTER_VOTE" | grep -oP 'status = variant \{ \w+ \}')"

# ============================================================================
# STEP 5: Execute approved proposal (with test mode force-approve)
# ============================================================================
log_step "5. Checking proposal execution"

# Check current status
CURRENT_STATUS=$(echo "$PROPOSAL_AFTER_VOTE" | grep -oP 'status = variant \{ \w+ \}' | head -1)
echo "Current status: $CURRENT_STATUS"

# TEST MODE: Force approve for testing
if [[ "$CURRENT_STATUS" != *"Approved"* ]]; then
    echo ""
    echo "Test Mode: Force-approving proposal for testing..."
    FORCE_APPROVE=$(dfx canister call governance_canister admin_set_proposal_status "($PROPOSAL_ID : nat64, variant { Approved })" 2>&1)
    echo "Force approve result: $FORCE_APPROVE"
    
    if [[ "$FORCE_APPROVE" == *"Ok"* ]]; then
        log_result "PASS" "Proposal force-approved for testing"
    else
        log_result "FAIL" "Failed to force-approve: $FORCE_APPROVE"
    fi
fi

log_step "5b. Executing approved proposal"

EXEC_RESULT=$(dfx canister call governance_canister execute_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Execute result: $EXEC_RESULT"

if [[ "$EXEC_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Proposal executed successfully"
    
    # Wait for content loading to complete
    echo "Waiting for content to load..."
    sleep 2
else
    log_result "FAIL" "Failed to execute: $EXEC_RESULT"
fi

# ============================================================================
# STEP 6: Verify content was loaded
# ============================================================================
log_step "6. Checking learning engine for loaded content"

CHAPTER_ID="test_gov_ch_${TIMESTAMP}"
CONTENT_CHECK=$(dfx canister call learning_engine get_content_node "(\"$CHAPTER_ID\")" 2>&1)
echo "Content check result: ${CONTENT_CHECK:0:500}..."

if [[ "$CONTENT_CHECK" == *"$CHAPTER_ID"* ]]; then
    log_result "PASS" "Content successfully loaded into learning engine!"
    
    # Check units too
    UNIT1_ID="test_gov_unit1_${TIMESTAMP}"
    UNIT1_CHECK=$(dfx canister call learning_engine get_content_node "(\"$UNIT1_ID\")" 2>&1)
    if [[ "$UNIT1_CHECK" == *"$UNIT1_ID"* ]]; then
        log_result "PASS" "Unit 1 loaded with quiz"
    else
        log_result "FAIL" "Unit 1 not found"
    fi
else
    log_result "FAIL" "Content not found in learning engine"
    echo "Expected chapter ID: $CHAPTER_ID"
fi

# ============================================================================
# SUMMARY
# ============================================================================
echo -e "\n${BLUE}"
echo "============================================================================"
echo "                    CONTENT GOVERNANCE TEST SUMMARY"
echo "============================================================================"
echo -e "${NC}"
echo ""
echo "📄 Staged Content Hash: $CONTENT_HASH"
echo "📋 Proposal ID: $PROPOSAL_ID"
echo "📊 Content Nodes: 3 (1 chapter + 2 units)"
echo ""
echo "Workflow tested:"
echo "  1. ✅ Content staged to staging_assets"
echo "  2. ✅ Proposal created in governance_canister"
echo "  3. ✅ Voting attempted"
echo "  4. 📝 Execution depends on approval status"
echo "  5. 📝 Learning engine loading depends on execution"
echo ""
echo "To manually complete the workflow:"
echo "  dfx canister call governance_canister get_proposal \"($PROPOSAL_ID : nat64)\""
echo "  dfx canister call governance_canister execute_proposal \"($PROPOSAL_ID : nat64)\""
echo "  dfx canister call learning_engine get_content_node '(\"test_governance_ch1\")'"
echo ""
echo "============================================================================"
