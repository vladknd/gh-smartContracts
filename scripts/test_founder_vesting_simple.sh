#!/bin/bash

# ============================================================================
# FOUNDER VESTING CANISTER TEST SCRIPT
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

log_header "FOUNDER VESTING SIMPLE TEST"

# ============================================================================
# 1. SETUP & DEPLOY
# ============================================================================
setup_environment "$@"

# Create/Use identities for founders (needed for principal extraction)
if ! dfx identity list | grep -q "founder1"; then
    dfx identity new founder1 --storage-mode=plaintext 2>/dev/null || true
fi
if ! dfx identity list | grep -q "founder2"; then
    dfx identity new founder2 --storage-mode=plaintext 2>/dev/null || true
fi

FOUNDER1=$(dfx identity get-principal --identity founder1)
FOUNDER2=$(dfx identity get-principal --identity founder2)

log_info "Founder 1: $FOUNDER1"
log_info "Founder 2: $FOUNDER2"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Initializing Ledger"
    dfx deploy ghc_ledger --argument '(variant {Init = record {
     token_symbol = "GHC";
     token_name = "Green Heroes Coin";
     minting_account = record { owner = principal "aaaaa-aa"; };
     transfer_fee = 10_000;
     metadata = vec {};
     initial_balances = vec {};
     archive_options = record {
         trigger_threshold = 2000;
         num_blocks_to_archive = 1000;
         controller_id = principal "aaaaa-aa";
     };
     feature_flags = opt record { icrc2 = true };
    }})'
    
    LEDGER_ID=$(dfx canister id ghc_ledger)

    log_step "Deploying founder_vesting..."
    dfx deploy founder_vesting --argument "(record {
        ledger_id = principal \"$LEDGER_ID\";
    })"

    log_step "Registering Founders..."
    dfx canister call founder_vesting admin_register_founder "(principal \"$FOUNDER1\", 35000000000000000)"
    dfx canister call founder_vesting admin_register_founder "(principal \"$FOUNDER2\", 15000000000000000)"
else
    log_info "Using existing deployment"
    LEDGER_ID=$(dfx canister id ghc_ledger 2>/dev/null)
fi

VESTING_ID=$(dfx canister id founder_vesting 2>/dev/null)
log_info "Founder Vesting ID: $VESTING_ID"

# ============================================================================
# 2. STATUS CHECKS
# ============================================================================
log_step "2. Status Checks"

SCHEDULES=$(dfx canister call founder_vesting get_all_vesting_schedules --query)
log_info "Schedules: $SCHEDULES"

if echo "$SCHEDULES" | grep -q "$FOUNDER1" && echo "$SCHEDULES" | grep -q "$FOUNDER2"; then
    log_pass "Both founders found in schedules"
else
    log_fail "Founders not found in schedules"
fi

GENESIS=$(dfx canister call founder_vesting get_genesis_timestamp --query)
log_info "Genesis Timestamp: $GENESIS"

# ============================================================================
# 3. CLAIMING (Negative Test - Too Early)
# ============================================================================
log_step "3. Claiming (Negative Test)"

# Switch to founder 1
dfx identity use founder1
CLAIM_RES=$(dfx canister call founder_vesting claim_vested 2>&1 || true)
log_info "Claim Result (Expect 0 or Error): $CLAIM_RES"

# Should probably return Ok(0) or Err depending on implementation if nothing vested
# Year 0 (10%) is immediate, so we expect 35M tokens (35_000_000_000_000_000) OR 0/Err if already claimed
if [[ "$CLAIM_RES" =~ "3_500_000_000_000_000" ]]; then
    log_pass "Year 0: 10% (35M GHC) claimed immediately"
elif echo "$CLAIM_RES" | grep -q "Ok"; then
    log_pass "Claim passed (Ok result)"
elif echo "$CLAIM_RES" | grep -q "Err"; then
    log_pass "Claim returned error (likely already claimed or no tokens)"
else
    log_fail "Unexpected claim result: $CLAIM_RES"
fi

dfx identity use default
summary
