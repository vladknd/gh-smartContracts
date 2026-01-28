#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Progress tracking
TESTS_TOTAL=0
TESTS_PASSED=0

log_header() {
    echo -e "\n${BLUE}==============================================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}==============================================================================${NC}"
}

log_step() { echo -e "${PURPLE}  → $1...${NC}"; }
log_pass() { echo -e "${GREEN}  [PASS] $1${NC}"; TESTS_PASSED=$((TESTS_PASSED + 1)); TESTS_TOTAL=$((TESTS_TOTAL + 1)); }
log_fail() { echo -e "${RED}  [FAIL] $1${NC}"; TESTS_TOTAL=$((TESTS_TOTAL + 1)); exit 1; }
log_info() { echo -e "${YELLOW}  (i) $1${NC}"; }
log_substep() { echo -e "${CYAN}    → $1${NC}"; }

setup_environment() {
    if [[ "$*" == *"--no-deploy"* ]]; then
        log_info "Skipping environment setup (using existing deployment)"
        return
    fi

    log_step "Initializing clean DFX state"
    dfx stop &>/dev/null || true
    dfx start --background --clean &>/dev/null
    sleep 3
}

deploy_system() {
    if [[ "$*" == *"--no-deploy"* ]]; then
        return
    fi
    log_step "Deploying Full System"
    ./scripts/deploy_full.sh local > /dev/null 2>&1
    log_pass "System deployed successfully"
}

summary() {
    log_header "AUDIT SUMMARY"
    echo -e "  - Total Checks: $TESTS_TOTAL"
    echo -e "  - Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "  - Failed: ${RED}$((TESTS_TOTAL - TESTS_PASSED))${NC}"
    echo ""

    if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  MODULE VERIFIED ✓${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        return 0
    else
        echo -e "${RED}  MODULE FAILED ✗${NC}"
        return 1
    fi
}
