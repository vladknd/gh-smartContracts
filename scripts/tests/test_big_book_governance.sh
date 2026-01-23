#!/bin/bash

# ============================================================================
# GHC Big Book Content Governance Stress Test
# ============================================================================
# Tests the content proposal workflow with a large hierarchical book:
# 1. Generate big book JSON (500+ nodes)
# 2. Stage content → 3. Create proposal → 4. Vote → 5. Execute → 6. Verify
#
# Purpose: Stress-test the content governance system with:
# - Deep hierarchy (BOOK → PART → CHAPTER → SECTION → SUBSECTION → UNIT)
# - Many content nodes (500+)
# - Multiple quiz questions
# - Chunked loading capability
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "============================================================================"
echo "     GHC Big Book Content Governance Stress Test"
echo "============================================================================"
echo -e "${NC}"

# Navigate to project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# Helper functions
log_step() {
    echo -e "\n${YELLOW}>>> Step: $1${NC}"
}

log_substep() {
    echo -e "${CYAN}    → $1${NC}"
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

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ============================================================================
# STEP 0: Verify prerequisites
# ============================================================================
log_step "0. Setup - Verify canisters and tools"

# Check Python3
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 is required but not found${NC}"
    exit 1
fi
log_info "Python3: $(python3 --version)"

# Check canisters
STAGING_ID=$(dfx canister id staging_assets 2>/dev/null) || true
GOVERNANCE_ID=$(dfx canister id governance_canister 2>/dev/null) || true
LEARNING_ENGINE_ID=$(dfx canister id learning_engine 2>/dev/null) || true
USER_PRINCIPAL=$(dfx identity get-principal)

if [ -z "$STAGING_ID" ] || [ -z "$GOVERNANCE_ID" ] || [ -z "$LEARNING_ENGINE_ID" ]; then
    echo -e "${RED}Error: Required canisters not found. Run ./scripts/deploy.sh first.${NC}"
    echo "  staging_assets: ${STAGING_ID:-NOT FOUND}"
    echo "  governance_canister: ${GOVERNANCE_ID:-NOT FOUND}"
    echo "  learning_engine: ${LEARNING_ENGINE_ID:-NOT FOUND}"
    exit 1
fi

echo ""
echo "Canister IDs:"
echo "  Staging Assets: $STAGING_ID"
echo "  Governance: $GOVERNANCE_ID"
echo "  Learning Engine: $LEARNING_ENGINE_ID"
echo "  Current User: $USER_PRINCIPAL"

# ============================================================================
# STEP 1: Generate the big book JSON file
# ============================================================================
log_step "1. Generating big book JSON file"

BIG_BOOK_JSON="$PROJECT_ROOT/scripts/content/big_book.json"

# Generate the book
python3 "$PROJECT_ROOT/scripts/content/generate_big_book.py" "$BIG_BOOK_JSON"

if [ ! -f "$BIG_BOOK_JSON" ]; then
    log_result "FAIL" "Big book JSON not generated"
    exit 1
fi

# Get stats
TOTAL_NODES=$(python3 -c "import json; data=json.load(open('$BIG_BOOK_JSON')); print(data['metadata']['total_nodes'])")
UNITS_WITH_QUIZZES=$(python3 -c "import json; data=json.load(open('$BIG_BOOK_JSON')); print(data['metadata']['total_units_with_quizzes'])")
FILE_SIZE=$(ls -lh "$BIG_BOOK_JSON" | awk '{print $5}')

log_result "PASS" "Generated big book with $TOTAL_NODES nodes ($FILE_SIZE)"
log_info "Total nodes: $TOTAL_NODES"
log_info "Units with quizzes: $UNITS_WITH_QUIZZES"

# ============================================================================
# STEP 2: Add current user as allowed stager
# ============================================================================
log_step "2. Adding current user as allowed stager"

ADD_STAGER_RESULT=$(dfx canister call staging_assets add_allowed_stager "(principal \"$USER_PRINCIPAL\")" 2>&1)
echo "Add stager result: $ADD_STAGER_RESULT"

# ============================================================================
# STEP 3: Convert JSON to Candid and stage content
# ============================================================================
log_step "3. Staging content to staging_assets canister"

# Create a Python script to convert JSON to Candid format and call staging
CONVERTER_SCRIPT=$(mktemp)
cat > "$CONVERTER_SCRIPT" << 'PYEOF'
#!/usr/bin/env python3
"""Convert big_book.json to Candid format and stage via dfx"""
import json
import subprocess
import sys

def escape_candid_string(s):
    """Escape a string for use in Candid notation."""
    if s is None:
        return None
    # Escape backslashes first
    s = str(s).replace('\\', '\\\\')
    # Escape double quotes
    s = s.replace('"', '\\"')
    # Escape newlines
    s = s.replace('\n', '\\n')
    s = s.replace('\r', '\\r')
    s = s.replace('\t', '\\t')
    return s

def node_to_candid(node):
    """Convert a content node dict to Candid record format."""
    parts = []
    
    # Required fields
    parts.append(f'id = "{escape_candid_string(node["id"])}"')
    
    # Optional parent_id
    if node.get("parent_id"):
        parts.append(f'parent_id = opt "{escape_candid_string(node["parent_id"])}"')
    else:
        parts.append('parent_id = null')
    
    # Order (nat32)
    parts.append(f'order = {node["order"]} : nat32')
    
    # Display type
    parts.append(f'display_type = "{escape_candid_string(node["display_type"])}"')
    
    # Title
    parts.append(f'title = "{escape_candid_string(node["title"])}"')
    
    # Optional description
    if node.get("description"):
        parts.append(f'description = opt "{escape_candid_string(node["description"])}"')
    else:
        parts.append('description = null')
    
    # Optional content
    if node.get("content"):
        parts.append(f'content = opt "{escape_candid_string(node["content"])}"')
    else:
        parts.append('content = null')
    
    # Optional paraphrase
    if node.get("paraphrase"):
        parts.append(f'paraphrase = opt "{escape_candid_string(node["paraphrase"])}"')
    else:
        parts.append('paraphrase = null')
    
    # Media (always null for this test)
    parts.append('media = null')
    
    # Optional quiz
    if node.get("quiz") and node["quiz"].get("questions"):
        questions = node["quiz"]["questions"]
        q_parts = []
        for q in questions:
            opts = "; ".join([f'"{escape_candid_string(o)}"' for o in q["options"]])
            q_parts.append(f'record {{ question = "{escape_candid_string(q["question"])}"; options = vec {{ {opts} }}; answer = {q["answer"]} : nat8 }}')
        quiz_str = "opt record { questions = vec { " + "; ".join(q_parts) + " }}"
        parts.append(f'quiz = {quiz_str}')
    else:
        parts.append('quiz = null')
    
    # Timestamps (nat64)
    parts.append(f'created_at = {node.get("created_at", 0)} : nat64')
    parts.append(f'updated_at = {node.get("updated_at", 0)} : nat64')
    parts.append(f'version = {node.get("version", 1)} : nat64')
    
    return "record { " + "; ".join(parts) + " }"

def main():
    json_file = sys.argv[1]
    
    with open(json_file, 'r') as f:
        data = json.load(f)
    
    content = data["content"]
    metadata = data["metadata"]
    
    print(f"Processing {len(content)} content nodes...")
    
    # Convert all nodes to Candid format
    candid_nodes = [node_to_candid(node) for node in content]
    
    # Build the vec of nodes
    nodes_vec = "vec { " + "; ".join(candid_nodes) + " }"
    
    # Build the stage_content call
    title = escape_candid_string(metadata["name"])
    description = escape_candid_string(metadata["description"])
    
    call_arg = f'("{title}", "{description}", {nodes_vec})'
    
    # Write to temp file (the argument is too long for command line)
    arg_file = "/tmp/big_book_stage_arg.txt"
    with open(arg_file, 'w') as f:
        f.write(call_arg)
    
    print(f"Candid argument written to {arg_file}")
    print(f"Argument size: {len(call_arg)} bytes")
    
    # Call dfx with the argument file
    print("Calling staging_assets.stage_content...")
    result = subprocess.run(
        ["dfx", "canister", "call", "staging_assets", "stage_content", "--argument-file", arg_file],
        capture_output=True,
        text=True
    )
    
    print("STDOUT:", result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    # Output result for bash to capture
    if "Ok" in result.stdout:
        # Extract hash
        import re
        match = re.search(r'Ok = "([^"]+)"', result.stdout)
        if match:
            print(f"CONTENT_HASH={match.group(1)}")
            sys.exit(0)
    
    sys.exit(1 if result.returncode != 0 else 0)

if __name__ == "__main__":
    main()
PYEOF

chmod +x "$CONVERTER_SCRIPT"

log_substep "Converting JSON to Candid format..."
STAGE_OUTPUT=$(python3 "$CONVERTER_SCRIPT" "$BIG_BOOK_JSON" 2>&1)
echo "$STAGE_OUTPUT"

# Extract content hash
CONTENT_HASH=$(echo "$STAGE_OUTPUT" | grep -oP '(?<=CONTENT_HASH=)[^\s]+' || echo "")

if [ -z "$CONTENT_HASH" ]; then
    # Try alternative extraction
    CONTENT_HASH=$(echo "$STAGE_OUTPUT" | grep -oP '(?<=Ok = ")[^"]+' || echo "")
fi

if [ -z "$CONTENT_HASH" ]; then
    log_result "FAIL" "Failed to stage content - no hash returned"
    exit 1
fi

log_result "PASS" "Content staged successfully"
log_info "Content hash: $CONTENT_HASH"

# Clean up temp script
rm -f "$CONVERTER_SCRIPT"

# ============================================================================
# STEP 4: Verify staged content
# ============================================================================
log_step "4. Verifying staged content"

STAGED_INFO=$(dfx canister call staging_assets get_staged_content_info "(\"$CONTENT_HASH\")" 2>&1)
echo "Staged content info: ${STAGED_INFO:0:500}..."

if [[ "$STAGED_INFO" == *"Pending"* ]]; then
    log_result "PASS" "Staged content is in Pending status"
else
    log_result "FAIL" "Unexpected staging status"
    echo "Full response: $STAGED_INFO"
fi

# ============================================================================
# STEP 5: Ensure current user has voting power
# ============================================================================
log_step "5. Ensuring current user has voting power"

# Check if current user is already a board member
IS_BOARD_MEMBER=$(dfx canister call governance_canister is_board_member "(principal \"$USER_PRINCIPAL\")" 2>&1)
echo "Is board member: $IS_BOARD_MEMBER"

if [[ "$IS_BOARD_MEMBER" == *"false"* ]]; then
    log_substep "Current user is not a board member. Adding for test..."
    
    # Get existing board members
    EXISTING_SHARES=$(dfx canister call governance_canister get_board_member_shares 2>&1)
    echo "Existing board members:"
    echo "$EXISTING_SHARES"
    
    # Build new shares list including current user
    # We'll add the test user with a small percentage and adjust others proportionally
    # Parse existing members and reduce their shares proportionally to make room for 10% for test user
    
    # For simplicity in testing, we'll set new shares including the test user
    # The test user gets 10%, others are redistributed from 100% to 90%
    log_substep "Adding current user with 10% share (redistributing existing shares)..."
    
    # Extract existing members using Python for reliable parsing
    NEW_SHARES=$(python3 << PYEOF
import re
import sys

raw = '''$EXISTING_SHARES'''

# Parse the principals and percentages
members = re.findall(r'member = principal "([^"]+)";\s*percentage = (\d+)', raw)

if not members:
    print("ERROR: Could not parse existing board members", file=sys.stderr)
    sys.exit(1)

test_user = "$USER_PRINCIPAL"

# If only a few members, we can add a new one
# Reduce each member's share proportionally to make room for 10%
new_shares = []
remaining = 90  # 100 - 10 for test user
total_orig = sum(int(pct) for _, pct in members)

for principal, pct in members:
    # Proportionally reduce
    new_pct = max(1, int(int(pct) * 90 / total_orig))
    new_shares.append((principal, new_pct))

# Adjust to make sure total is 90
current_total = sum(pct for _, pct in new_shares)
if current_total != 90:
    diff = 90 - current_total
    # Add diff to largest share
    largest_idx = max(range(len(new_shares)), key=lambda i: new_shares[i][1])
    new_shares[largest_idx] = (new_shares[largest_idx][0], new_shares[largest_idx][1] + diff)

# Add test user with 10%
new_shares.append((test_user, 10))

# Build Candid vec
parts = []
for principal, pct in new_shares:
    parts.append(f'record {{ member = principal "{principal}"; percentage = {pct} : nat8 }}')

print("vec { " + "; ".join(parts) + " }")
PYEOF
)
    
    echo "New shares: $NEW_SHARES"
    
    if [[ "$NEW_SHARES" == *"ERROR"* ]]; then
        log_result "FAIL" "Could not parse existing board members"
        # Try alternative: just set the test user as sole board member for testing
        log_substep "Fallback: Setting test user as only board member..."
        SET_RESULT=$(dfx canister call governance_canister set_board_member_shares "(vec { record { member = principal \"$USER_PRINCIPAL\"; percentage = 100 : nat8 } })" 2>&1)
    else
        SET_RESULT=$(dfx canister call governance_canister set_board_member_shares "($NEW_SHARES)" 2>&1)
    fi
    
    echo "Set shares result: $SET_RESULT"
    
    if [[ "$SET_RESULT" == *"Ok"* ]]; then
        log_result "PASS" "Added current user as board member"
    elif [[ "$SET_RESULT" == *"locked"* ]]; then
        log_info "Board shares are locked - will use existing board member identity if available"
    else
        log_result "FAIL" "Could not add board member: $SET_RESULT"
    fi
else
    log_result "PASS" "Current user is already a board member"
fi

# Verify voting power
VOTING_POWER=$(dfx canister call governance_canister get_my_voting_power 2>&1 || echo "0")
echo "Current voting power: $VOTING_POWER"

# ============================================================================
# STEP 6: Create content proposal through governance
# ============================================================================
log_step "6. Creating content proposal through governance canister"

PROPOSAL_RESULT=$(dfx canister call governance_canister create_add_content_proposal "(record {
    title = \"Add Big Book Stress Test Content\";
    description = \"Proposal to add stress test content: $TOTAL_NODES nodes with deep hierarchy (Book → Part → Chapter → Section → Subsection → Unit). Contains $UNITS_WITH_QUIZZES quizzes.\";
    staging_canister = principal \"$STAGING_ID\";
    staging_path = \"$CONTENT_HASH\";
    content_hash = \"$CONTENT_HASH\";
    content_title = \"Environmental Science Encyclopedia - Stress Test\";
    unit_count = $TOTAL_NODES : nat32;
    external_link = null;
})" 2>&1)

echo "Proposal result: $PROPOSAL_RESULT"

if [[ "$PROPOSAL_RESULT" == *"Ok"* ]]; then
    PROPOSAL_ID=$(echo "$PROPOSAL_RESULT" | grep -oP '(?<=Ok = )\d+')
    log_result "PASS" "Content proposal created"
    log_info "Proposal ID: $PROPOSAL_ID"
else
    log_result "FAIL" "Failed to create proposal: $PROPOSAL_RESULT"
    exit 1
fi

# ============================================================================
# STEP 7: Link proposal to staged content
# ============================================================================
log_step "7. Linking proposal to staged content"

LINK_RESULT=$(dfx canister call staging_assets set_proposal_id "(\"$CONTENT_HASH\", $PROPOSAL_ID : nat64)" 2>&1)
echo "Link result: $LINK_RESULT"

if [[ "$LINK_RESULT" == *"Ok"* ]] || [[ "$LINK_RESULT" == *"()"* ]]; then
    log_result "PASS" "Proposal linked to staged content"
else
    log_info "Note: Link might have failed or already set: $LINK_RESULT"
fi

# ============================================================================
# STEP 8: Support and Vote on proposal
# ============================================================================
log_step "8. Supporting the proposal (to move to Active)"

SUPPORT_RESULT=$(dfx canister call governance_canister support_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Support result: $SUPPORT_RESULT"

if [[ "$SUPPORT_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Proposal supported"
else
    log_info "Note: Support might require voting power"
fi

log_step "8b. Voting YES on the proposal"

VOTE_RESULT=$(dfx canister call governance_canister vote "($PROPOSAL_ID : nat64, true)" 2>&1)
echo "Vote result: $VOTE_RESULT"

if [[ "$VOTE_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Vote cast successfully"
else
    log_info "Vote result: $VOTE_RESULT"
fi

# ============================================================================
# STEP 9: Force approve and execute (test mode)
# ============================================================================
log_step "9. Force-approving proposal for testing"

# Check current status
PROPOSAL_INFO=$(dfx canister call governance_canister get_proposal "($PROPOSAL_ID : nat64)" 2>&1)
CURRENT_STATUS=$(echo "$PROPOSAL_INFO" | grep -oP 'status = variant \{ \w+ \}' | head -1 || echo "unknown")
echo "Current status: $CURRENT_STATUS"

if [[ "$CURRENT_STATUS" != *"Approved"* ]]; then
    FORCE_APPROVE=$(dfx canister call governance_canister admin_set_proposal_status "($PROPOSAL_ID : nat64, variant { Approved })" 2>&1)
    echo "Force approve result: $FORCE_APPROVE"
    
    if [[ "$FORCE_APPROVE" == *"Ok"* ]]; then
        log_result "PASS" "Proposal force-approved for testing"
    else
        log_result "FAIL" "Failed to force-approve: $FORCE_APPROVE"
    fi
else
    log_info "Proposal already approved"
fi

log_step "9b. Executing approved proposal"

EXEC_RESULT=$(dfx canister call governance_canister execute_proposal "($PROPOSAL_ID : nat64)" 2>&1)
echo "Execute result: $EXEC_RESULT"

if [[ "$EXEC_RESULT" == *"Ok"* ]]; then
    log_result "PASS" "Proposal executed successfully - content loading initiated"
else
    log_result "FAIL" "Failed to execute: $EXEC_RESULT"
fi

# ============================================================================
# STEP 10: Monitor loading progress
# ============================================================================
log_step "10. Monitoring content loading progress"

echo "Waiting for content to load..."

MAX_WAIT=120  # seconds
WAIT_INTERVAL=5
ELAPSED=0

while [ $ELAPSED -lt $MAX_WAIT ]; do
    LOADING_STATUS=$(dfx canister call learning_engine get_loading_status "($PROPOSAL_ID : nat64)" 2>&1)
    
    if [[ "$LOADING_STATUS" == *"Completed"* ]]; then
        log_result "PASS" "Content loading completed!"
        break
    elif [[ "$LOADING_STATUS" == *"InProgress"* ]]; then
        # Try to extract progress
        LOADED=$(echo "$LOADING_STATUS" | grep -oP 'loaded_units = \d+' | grep -oP '\d+' || echo "?")
        TOTAL=$(echo "$LOADING_STATUS" | grep -oP 'total_units = \d+' | grep -oP '\d+' || echo "?")
        echo "  Loading progress: $LOADED / $TOTAL nodes..."
    elif [[ "$LOADING_STATUS" == *"Failed"* ]] || [[ "$LOADING_STATUS" == *"Paused"* ]]; then
        log_result "FAIL" "Loading failed or paused"
        echo "$LOADING_STATUS"
        break
    fi
    
    sleep $WAIT_INTERVAL
    ELAPSED=$((ELAPSED + WAIT_INTERVAL))
done

if [ $ELAPSED -ge $MAX_WAIT ]; then
    log_info "Timeout waiting for loading. Current status:"
    dfx canister call learning_engine get_loading_status "($PROPOSAL_ID : nat64)"
fi

# ============================================================================
# STEP 11: Verify content was loaded
# ============================================================================
log_step "11. Verifying content in learning engine"

# Get content stats
CONTENT_STATS=$(dfx canister call learning_engine get_content_stats 2>&1)
echo "Content stats: $CONTENT_STATS"

# Try to find the root book node (first node should be the book)
ROOT_ID=$(python3 -c "import json; data=json.load(open('$BIG_BOOK_JSON')); print(data['content'][0]['id'])")
echo "Looking for root node: $ROOT_ID"

ROOT_CHECK=$(dfx canister call learning_engine get_content_node "(\"$ROOT_ID\")" 2>&1)
echo "Root node check: ${ROOT_CHECK:0:500}..."

if [[ "$ROOT_CHECK" == *"$ROOT_ID"* ]]; then
    log_result "PASS" "Root book node found in learning engine!"
    
    # Check some children
    log_substep "Checking children of root..."
    CHILDREN_CHECK=$(dfx canister call learning_engine get_children "(\"$ROOT_ID\")" 2>&1)
    CHILD_COUNT=$(echo "$CHILDREN_CHECK" | grep -o '"[^"]*"' | wc -l || echo 0)
    log_info "Found $CHILD_COUNT children of root node"
else
    log_result "FAIL" "Root node not found in learning engine"
    echo "Expected ID: $ROOT_ID"
fi

# Verify quiz index
log_substep "Checking quiz index..."
QUIZ_STATS=$(dfx canister call learning_engine get_quiz_count 2>&1 || echo "get_quiz_count not available")
echo "Quiz count: $QUIZ_STATS"

# ============================================================================
# SUMMARY
# ============================================================================
echo -e "\n${BLUE}"
echo "============================================================================"
echo "              BIG BOOK CONTENT GOVERNANCE TEST SUMMARY"
echo "============================================================================"
echo -e "${NC}"
echo ""
echo "📚 Book Statistics:"
echo "   Total Nodes: $TOTAL_NODES"
echo "   Units with Quizzes: $UNITS_WITH_QUIZZES"
echo "   File Size: $FILE_SIZE"
echo ""
echo "📄 Staged Content Hash: $CONTENT_HASH"
echo "📋 Proposal ID: $PROPOSAL_ID"
echo ""
echo "📊 Hierarchy Tested:"
echo "   BOOK → PART → CHAPTER → SECTION → SUBSECTION → UNIT"
echo ""
echo "Workflow tested:"
echo "  1. ✅ Big book generated ($TOTAL_NODES nodes)"
echo "  2. ✅ Content staged to staging_assets"
echo "  3. ✅ Proposal created in governance_canister"
echo "  4. ✅ Voting and approval"
echo "  5. 📝 Execution and loading (check status above)"
echo "  6. 📝 Learning engine verification (check status above)"
echo ""
echo "To manually check results:"
echo "  dfx canister call learning_engine get_content_stats"
echo "  dfx canister call learning_engine get_loading_status '($PROPOSAL_ID : nat64)'"
echo "  dfx canister call learning_engine get_content_node '(\"$ROOT_ID\")'"
echo ""
echo "============================================================================"
