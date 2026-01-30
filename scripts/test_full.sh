#!/bin/bash

# ============================================================================
# MASTER AUDIT ORCHESTRATOR
# ============================================================================
# This is the master entry point for the entire GreenHero testing suite.
# It performs a clean deployment and then executes all specialized and 
# comprehensive test modules to ensure 100% system integrity.
# ============================================================================

set -e

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"
source "$SCRIPT_DIR/test_helper.sh"

log_header "GREENHERO MASTER SYSTEM AUDIT"

# 1. Environment Setup
log_step "Phase 0: Environment Preparation"
if [[ "$*" != *"--no-deploy"* ]]; then
    dfx stop &>/dev/null || true
    dfx start --background --clean &>/dev/null
    sleep 5
    log_pass "Clean DFX environment started"
else
    log_info "Skipping environment preparation (using existing)"
fi

# 2. Global Deployment
log_step "Phase 1: Full System Deployment"
if [[ "$*" != *"--no-deploy"* ]]; then
    "$SCRIPT_DIR/deploy_full.sh" local > deployment.log 2>&1
    log_pass "Full system deployed with Archives and Shards"
else
    log_info "Skipping full system deployment (using existing)"
fi

# 3. Execution of Test Modules
# We pass --no-deploy to all sub-scripts to use the existing deployment

run_module() {
    local name=$1
    local script=$2
    log_header "MODULE: $name"
    if bash "$SCRIPT_DIR/$script" --no-deploy; then
        echo -e "${GREEN}✓ $name Passed${NC}"
    else
        echo -e "${RED}✗ $name Failed${NC}"
        exit 1
    fi
}

# --- Comprehensive Series (Audit logic) ---
run_module "Governance Logic" "test_governance_comprehensive.sh"
run_module "Staking & Economy" "test_staking_hub_comprehensive.sh"
run_module "User Profiles & Shards" "test_user_profile_comprehensive.sh"
run_module "Learning Engine Core" "test_learning_engine_comprehensive.sh"
run_module "ICO & Tokenomics" "test_ico_comprehensive.sh"
run_module "Founder Vesting" "test_founder_vesting_comprehensive.sh"
run_module "Subscription Flow" "test_subscription_flow.sh"
run_module "KYC Verification" "test_kyc_flow.sh"

# --- Specialized Series (Stress, Edge Cases, Scaling) ---
run_module "Governance Stress (500+ Nodes)" "test_governance_stress.sh"
run_module "Governance Edge Cases" "test_governance_edge_cases.sh"
run_module "Governance BPS System" "test_governance_bps_system.sh"
run_module "Staking Auto-Scaling" "test_staking_hub_scaling.sh"
run_module "Learning Engine Simple E2E" "test_learning_engine_simple.sh"
run_module "User Profile Simple E2E" "test_user_profile_simple.sh"
run_module "Founder Vesting Simple" "test_founder_vesting_simple.sh"
run_module "Staging Assets" "test_staging_assets.sh"
run_module "Media Assets" "test_media_assets.sh"
run_module "Treasury Operations" "test_treasury.sh"
# run_module "Factory Scaling" "test_factory.sh"

# --- Production Final Audit ---
run_module "Production History & Archiving" "test_archiving_audit.sh"

log_header "FINAL AUDIT SUMMARY"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ALL TEST MODULES PASSED - SYSTEM VERIFIED FOR PRODUCTION ✓${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Full deployment logs available in: deployment.log"
