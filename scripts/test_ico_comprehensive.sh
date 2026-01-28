#!/bin/bash
set -e

# ============================================================================
# COMPREHENSIVE ICO AUDIT SUITE
# ============================================================================
# Verifies ICO purchase flow with mocked USDC and GHC ledgers.
# ============================================================================

# Log helpers
source "$(dirname "$0")/test_helper.sh"

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
    log_pass "Infrastructure deployed"
fi

GHC_ID=$(dfx canister id ghc_ledger 2>/dev/null)
ICO_ID=$(dfx canister id ico_canister 2>/dev/null)

if [ -z "$ICO_ID" ]; then
    log_fail "ICO Canister not deployed"
fi


# ============================================================================
# PHASE 2: INITIAL FUNDING
# ============================================================================
log_header "PHASE 2: Funding"

# 1. Fund ICO Canister with GHC (Inventory)
log_step "Funding ICO with GHC Inventory (10,000 GHC)"
dfx canister call ghc_ledger icrc1_transfer "(record { 
    to = record { owner = principal \"$ICO_ID\" }; 
    amount = 1000000000000 : nat;
})" &>/dev/null

# Create Buyer Identity
dfx identity new buyer --storage-mode=plaintext &>/dev/null || true
BUYER_P=$(dfx --identity buyer identity get-principal)

# 2. Fund Buyer with "USDC"
log_step "Minting 'USDC' to Buyer ($BUYER_P)"
dfx canister call ghc_ledger icrc1_transfer "(record { 
    to = record { owner = principal \"$BUYER_P\" }; 
    amount = 100000000000 : nat;
})" &>/dev/null

# ============================================================================
# PHASE 3: PURCHASE FLOW
# ============================================================================
log_header "PHASE 3: Purchase GHC"

# 1. Approve ICO Canister to spend Buyer's USDC
log_step "Approving ICO to spend USDC (100 'USDC')"
dfx --identity buyer canister call ghc_ledger icrc2_approve "(record {
    spender = record { owner = principal \"$ICO_ID\" };
    amount = 10000000000 : nat;
})"

log_step "Buying 50 GHC"
BUY=$(dfx --identity buyer canister call ico_canister buy_ghc "(5000000000 : nat)")
if [[ "$BUY" == *"(variant { Ok = \"Purchase successful\" })"* ]]; then
    log_pass "Purchase successful"
else
    log_fail "Purchase failed: $BUY"
fi

# 3. Verify Stats
STATS=$(dfx canister call ico_canister get_ico_stats)
# Check if total_sold_ghc is at least 5_000_000_000 (50 GHC)
SOLD_AMOUNT=$(echo "$STATS" | grep -oP 'total_sold_ghc = \K[0-9_]+' | tr -d '_')
if [ "$SOLD_AMOUNT" -ge 5000000000 ]; then
    log_pass "Stats updated correctly (Sold: $SOLD_AMOUNT)"
else
    log_fail "Stats mismatch: $STATS"
fi

# ============================================================================
# PHASE 4: END SALE (SWEEP)
# ============================================================================
log_header "PHASE 4: End Sale (Sweep)"

# The ICO canister should have:
# - GHC Inventory: Initial - Sold
# - USDC Revenue: Cost of Sold
# Admin/Treasury should be empty of GHC before this? No.
# Only checking that the call succeeds for now.

SWEEP=$(dfx canister call ico_canister end_sale)
if [[ "$SWEEP" == *"(variant { Ok"* ]]; then
    log_pass "End sale sweep successful"
else
    log_fail "Sweep failed: $SWEEP"
fi

summary
