#!/bin/bash
set -e

# ============================================================================
# COMPREHENSIVE FOUNDER VESTING AUDIT SUITE
# ============================================================================
# This script performs an exhaustive verification of the Founder Vesting,
# covering 10-year vesting schedule, multiple founders, and claim logic.
# ============================================================================

# Log helpers
source "$(dirname "$0")/test_helper.sh"

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
setup_environment "$@"

# Ensure identities exist for principal extraction
if ! dfx identity list | grep -q "founder1"; then
    dfx identity new founder1 --storage-mode=plaintext &>/dev/null || true
fi
if ! dfx identity list | grep -q "founder2"; then
    dfx identity new founder2 --storage-mode=plaintext &>/dev/null || true
fi

F1_P=$(dfx --identity founder1 identity get-principal)
F2_P=$(dfx --identity founder2 identity get-principal)

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying Full System"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
    
    LEDGER_ID=$(dfx canister id ghc_ledger)
    VESTING_ID=$(dfx canister id founder_vesting)

    # Deploy vesting with ONLY ledger id
    log_step "Configuring Vesting for Test"
    dfx deploy founder_vesting --argument "(record { 
        ledger_id = principal \"$LEDGER_ID\"; 
    })" &>/dev/null

    # Register founders manually
    log_step "Registering Founders via Admin"
    dfx canister call founder_vesting admin_register_founder "(principal \"$F1_P\", 35000000000000000)" &>/dev/null
    dfx canister call founder_vesting admin_register_founder "(principal \"$F2_P\", 15000000000000000)" &>/dev/null

    # Mint total tokens to Vesting Canister
    # 0.35B + 0.15B = 0.5B = 50,000,000,000,000,000 e8s
    dfx canister call ghc_ledger icrc1_transfer "(record { to = record { owner = principal \"$VESTING_ID\" }; amount = 50000000000000000 : nat })" &>/dev/null

    log_pass "Infrastructure deployed & funded"
else
    LEDGER_ID=$(dfx canister id ghc_ledger 2>/dev/null)
    VESTING_ID=$(dfx canister id founder_vesting 2>/dev/null)
    log_info "Using existing deployment"
fi

# Make founder1 a controller to allow calling admin test functions (time travel)
dfx canister update-settings founder_vesting --add-controller "$F1_P"

# ============================================================================
# PHASE 2: INITIAL STATE
# ============================================================================
log_header "PHASE 2: Initial State"

log_step "Verifying Founder Registration"
S_F1=$(dfx canister call founder_vesting get_vesting_status "(principal \"$F1_P\")")
if [[ "$S_F1" == *"total_allocation = 35_000_000_000_000_000"* ]]; then
    log_pass "Founder 1 registered with correct allocation (350M GHC)"
else
    log_fail "Founder 1 registration check failed: $S_F1"
fi

S_F2=$(dfx canister call founder_vesting get_vesting_status "(principal \"$F2_P\")")
if [[ "$S_F2" == *"total_allocation = 15_000_000_000_000_000"* ]]; then
    log_pass "Founder 2 registered with correct allocation (150M GHC)"
else
    log_fail "Founder 2 registration check failed: $S_F2"
fi

log_step "Verifying Initial Claimable (Year 0 - 10% immediate)"
# Check if already claimed
STATUS=$(dfx canister call founder_vesting get_vesting_status "(principal \"$F1_P\")")
CLAIMED=$(echo "$STATUS" | grep -oP 'claimed = \K[0-9_]+' | tr -d '_')
EXPECTED_Y0=3500000000000000

if [ "$CLAIMED" -ge "$EXPECTED_Y0" ]; then
    log_pass "Year 0: Already claimed ($CLAIMED >= 35M GHC)"
else
    CLAIMABLE_0=$(dfx --identity founder1 canister call founder_vesting claim_vested 2>&1 || true)
    if [[ "$CLAIMABLE_0" =~ "3_500_000_000_000_000" ]]; then
        log_pass "Year 0: 10% (35M GHC) claimed immediately"
    else
        log_fail "Year 0 claim failed: $CLAIMABLE_0"
    fi
fi

# ============================================================================
# PHASE 3: VESTING PROGRESSION
# ============================================================================
log_header "PHASE 3: Vesting Progression"

GENESIS=$(dfx canister call founder_vesting get_genesis_timestamp | tr -d '_' | grep -oP '\d+' | head -n 1)
YEAR_NANOS=$((365 * 24 * 60 * 60 * 1000000000))

log_step "Simulating Year 1 (Additional 10% -> 20% Total)"
YEAR1_TIME=$((GENESIS + YEAR_NANOS + 1000000)) # Margin
EXPECTED_Y1=7000000000000000 # 20% = 70M GHC
if [ "$CLAIMED" -ge "$EXPECTED_Y1" ]; then
   log_pass "Year 1: Already claimed ($CLAIMED >= 70M GHC)"
else
    S_F1_Y1=$(dfx --identity founder1 canister call founder_vesting admin_claim_vested_at "($YEAR1_TIME)" 2>&1 || true)
    if [[ "$S_F1_Y1" =~ "3_500_000_000_000_000" ]] || [[ "$S_F1_Y1" =~ "Ok" ]]; then
        log_pass "Year 1: Claimed successfully"
    else
        log_fail "Year 1 claim failed: $S_F1_Y1"
    fi
fi

log_step "Simulating Year 5 (Additional 40% -> 60% Total)"
YEAR5_TIME=$((GENESIS + (5 * YEAR_NANOS) + 1000000))
EXPECTED_Y5=21000000000000000 # 60% = 210M GHC
if [ "$CLAIMED" -ge "$EXPECTED_Y5" ]; then
   log_pass "Year 5: Already claimed ($CLAIMED >= 210M GHC)"
else
    S_F1_Y5=$(dfx --identity founder1 canister call founder_vesting admin_claim_vested_at "($YEAR5_TIME)" 2>&1 || true)
    if [[ "$S_F1_Y5" =~ "14_000_000_000_000_000" ]] || [[ "$S_F1_Y5" =~ "Ok" ]]; then
        log_pass "Year 5: Claimed successfully"
    else
        log_fail "Year 5 claim failed: $S_F1_Y5"
    fi
fi

log_step "Verifying Ledger Balance for Founder 1"
F1_BAL=$(dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"$F1_P\" })")
# Minimum 210M if Year 5 passed
if [[ "$F1_BAL" =~ "21_000_000_000_000_000" ]] || [[ "$F1_BAL" =~ "_" ]]; then
    # Just check if balance is generally correct?
    # Strip underscores
    VAL=$(echo "$F1_BAL" | tr -d '_ :nat()')
    if [ "$VAL" -ge 21000000000000000 ]; then
         log_pass "Founder 1 ledger balance matches expectations (>= 210M GHC): $VAL"
    else
         log_fail "Ledger balance too low: $VAL (Expected >= 210M)"
    fi
else
    log_fail "Ledger balance mismatch format: $F1_BAL"
fi

log_step "Simulating Year 11 (100% cap)"
YEAR11_TIME=$((GENESIS + (11 * YEAR_NANOS) + 1000000))
EXPECTED_Y11=35000000000000000 # 100% = 350M GHC
if [ "$CLAIMED" -ge "$EXPECTED_Y11" ]; then
    log_pass "Year 11: Already claimed ($CLAIMED >= 350M GHC)"
else
    S_F1_Y11=$(dfx --identity founder1 canister call founder_vesting admin_claim_vested_at "($YEAR11_TIME)" 2>&1 || true)
    if [[ "$S_F1_Y11" =~ "14_000_000_000_000_000" ]] || [[ "$S_F1_Y11" =~ "Ok" ]]; then
        log_pass "Year 11: Claimed successfully"
    else
        log_fail "Year 11 claim failed: $S_F1_Y11"
    fi
fi

log_step "Verifying Fully Vested State"
FINAL_S=$(dfx canister call founder_vesting get_vesting_status "(principal \"$F1_P\")")
# Total claimed should be 350M tokens (35_000_000_000_000_000 e8s)
if [[ "$FINAL_S" == *"claimed = 35_000_000_000_000_000"* && "$FINAL_S" == *"claimable = 0"* ]]; then
    log_pass "Founder 1 fully vested and emptied"
else
    log_fail "Final state check failed: $FINAL_S"
fi

# ============================================================================
# PHASE 4: SECURITY & EDGE CASES
# ============================================================================
log_header "PHASE 4: Security & Edge Cases"

log_step "Verifying Non-Founder cannot claim"
dfx identity new attacker --storage-mode=plaintext &>/dev/null || true
ATTACK=$(dfx --identity attacker canister call founder_vesting claim_vested 2>&1 || true)
if [[ "$ATTACK" == *"is not a registered founder"* ]]; then
    log_pass "Security: Attacker rejected correctly"
else
    log_fail "Security breach: Attacker allowed to claim! $ATTACK"
fi

summary
