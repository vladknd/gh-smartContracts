# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "STAKING HUB SCALING TEST"

# ============================================
# 1. Environment Setup
# ============================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System with embedded WASMs"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
fi

GHC_LEDGER=$(dfx canister id ghc_ledger 2>/dev/null)
STAKING_HUB=$(dfx canister id staking_hub 2>/dev/null)

if [ -z "$STAKING_HUB" ]; then
    log_fail "Infrastructure not deployed"
fi

log_info "Ledger ID: $GHC_LEDGER"
log_info "Staking Hub ID: $STAKING_HUB"


# ============================================
# 7. Running Functional Tests
# ============================================
log_step "Test 1: Global Configuration"
CONFIG=$(dfx canister call staking_hub get_config)
if [[ "$CONFIG" == *"$GHC_LEDGER"* ]]; then
    log_pass "Ledger ID correctly configured"
else
    log_fail "Ledger ID mismatch in config: $CONFIG"
fi

log_step "Test 2: Auto-Scaling (ensure_capacity)"
log_info "Triggering auto-scaling (ensuring at least one shard exists)..."
ENSURE_RESULT=$(dfx canister call staking_hub ensure_capacity)

if [[ "$ENSURE_RESULT" == *"Ok ="* ]]; then
    # Extract shard ID from variant { Ok = opt principal "..." } or variant { Ok = null }
    SHARD_ID=$(echo "$ENSURE_RESULT" | grep -oP 'principal "[^"]+"' | head -1 | cut -d'"' -f2)
    
    if [ -n "$SHARD_ID" ]; then
        log_pass "Successfully created a new shard: $SHARD_ID"
    else
        log_info "No new shard created (already at capacity). Checking for existing shards..."
        SHARDS_LIST=$(dfx canister call staking_hub get_shards)
        # Extract the first principal that looks like a canister ID
        SHARD_ID=$(echo "$SHARDS_LIST" | grep -oP 'canister_id = principal "[^"]+"' | head -1 | cut -d'"' -f2)
        
        if [ -n "$SHARD_ID" ]; then
            log_pass "Found existing shard: $SHARD_ID"
        else
            log_fail "No shards found in registry. Output: $SHARDS_LIST"
        fi
    fi
else
    log_fail "Auto-scaling failed: $ENSURE_RESULT"
fi

log_step "Test 3: Shard Registration & Archive Linking"
SHARDS=$(dfx canister call staking_hub get_shards)
if [[ "$SHARDS" == *"$SHARD_ID"* ]]; then
    log_pass "Shard $SHARD_ID registered in hub"
else
    log_fail "Shard ID $SHARD_ID not found in registry"
fi

# Verification of Archive Linking
if [[ "$SHARDS" == *"archive_canister_id"* ]]; then
    log_pass "Registry includes archive_canister_id field"
    # Extract the archive ID for THIS specific shard using python for robust multi-record parsing
    ARCHIVE_ID=$(echo "$SHARDS" | python3 -c "
import sys, re
raw = sys.stdin.read()
# Look for the record containing our shard_id and capture the archive_id within it
# We use a pattern that matches the record start/end roughly
pattern = r'record\s*\{[^{}]*canister_id\s*=\s*principal\s*\"$SHARD_ID\"[^{}]*archive_canister_id\s*=\s*opt\s*principal\s*\"([^\"]+)\"'
m = re.search(pattern, raw, re.DOTALL)
if m:
    print(m.group(1))
else:
    # Try a simpler fallback if the above is too strict
    m2 = re.search(r'canister_id\s*=\s*principal\s*\"$SHARD_ID\".*?archive_canister_id\s*=\s*opt\s*principal\s*\"([^\"]+)\"', raw, re.DOTALL)
    print(m2.group(1) if m2 else '')
")
    if [ -n "$ARCHIVE_ID" ]; then
        log_pass "Successfully identified linked archive canister: $ARCHIVE_ID"
    else
        # If it's opt null, it might be correctly absent if not using archives, but we expect it here
        if [[ "$SHARDS" == *"archive_canister_id = null"* ]] || [[ "$SHARDS" == *"archive_canister_id = opt null"* ]]; then
            log_fail "Archive canister is null for shard $SHARD_ID"
        else
            log_fail "Could not parse archive_canister_id for shard $SHARD_ID from output"
        fi
    fi
else
    log_fail "Registry is missing archive_canister_id field (Phase 2 failure)"
fi

log_step "Test 4: User Location Tracking"
log_info "Testing user location registration (admin)..."
# Create a test user principal
TEST_USER="2vxsx-fae"
# Use the shard we found/created
REG_RESULT=$(dfx canister call staking_hub admin_set_user_shard "(principal \"$TEST_USER\", principal \"$SHARD_ID\")" 2>&1)
if [[ "$REG_RESULT" == *"Ok"* ]]; then
    log_pass "admin_set_user_shard successful"
    # Verify mapping
    MAPPING=$(dfx canister call staking_hub get_user_shard "(principal \"$TEST_USER\")")
    if [[ "$MAPPING" == *"$SHARD_ID"* ]]; then
        log_pass "User location correctly mapped in hub"
    else
        log_fail "User location mapping failed: $MAPPING"
    fi
else
    log_fail "admin_set_user_shard failed: $REG_RESULT"
fi

summary
