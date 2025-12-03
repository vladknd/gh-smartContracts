#!/bin/bash

# Comprehensive System Test Suite (Enhanced)
# Generates a detailed report in test_report.md

REPORT_FILE="test_report.md"
echo "# Comprehensive System Test Report" > $REPORT_FILE
echo "Date: $(date)" >> $REPORT_FILE
echo "-----------------------------------" >> $REPORT_FILE

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

function log_step {
    echo -e "\n## $1" >> $REPORT_FILE
    echo -e "${GREEN}>>> Executing: $1${NC}"
}

function log_result {
    if [ "$1" == "PASS" ]; then
        echo -e "- **Result**: ✅ PASS" >> $REPORT_FILE
        echo -e "- **Details**: $2" >> $REPORT_FILE
        echo -e "${GREEN}PASS: $2${NC}"
    else
        echo -e "- **Result**: ❌ FAIL" >> $REPORT_FILE
        echo -e "- **Details**: $2" >> $REPORT_FILE
        echo -e "${RED}FAIL: $2${NC}"
    fi
}

# 1. Deployment
log_step "System Deployment"
echo "Restarting DFX Clean..."
dfx stop
rm -rf .dfx
dfx start --background --clean

./deploy.sh >> deployment.log 2>&1
if [ $? -eq 0 ]; then
    log_result "PASS" "All canisters deployed successfully."
else
    log_result "FAIL" "Deployment failed. Check deployment.log."
    exit 1
fi

# Get IDs
STAKING_HUB_ID=$(dfx canister id staking_hub)
USER_PROFILE_ID=$(dfx canister id user_profile)
LEARNING_ENGINE_ID=$(dfx canister id learning_engine)

echo "- Staking Hub: \`$STAKING_HUB_ID\`" >> $REPORT_FILE
echo "- User Profile: \`$USER_PROFILE_ID\`" >> $REPORT_FILE

# 1b. Verify Internet Identity
log_step "Verification: Internet Identity"
II_STATUS=$(dfx canister status internet_identity 2>&1)
if [[ "$II_STATUS" == *"Status: Running"* ]]; then
    log_result "PASS" "Internet Identity is running."
else
    log_result "FAIL" "Internet Identity is NOT running. Status: $II_STATUS"
fi

# 2. Setup: Register Shard
log_step "Configuration: Register User Profile Shard"
OUT=$(dfx canister call staking_hub add_allowed_minter "(principal \"$USER_PROFILE_ID\")" 2>&1)
if [[ "$OUT" == *"()"* ]]; then
    log_result "PASS" "User Profile registered as Allowed Minter."
else
    log_result "FAIL" "Failed to register minter. Output: $OUT"
fi

# 3. Content Management
log_step "Content: Add Learning Unit"
OUT=$(dfx canister call learning_engine add_learning_unit '(record {
  unit_id = "unit_test_1";
  unit_title = "Test Unit";
  chapter_id = "chap_1";
  chapter_title = "Test Chapter";
  head_unit_id = "head_1";
  head_unit_title = "Test Head";
  content = "Content";
  paraphrase = "Para";
  quiz = vec { record { question = "Q?"; options = vec {"A"}; answer = 0 } }
})' 2>&1)

if [[ "$OUT" == *"Ok"* ]]; then
    log_result "PASS" "Learning Unit added."
else
    log_result "FAIL" "Failed to add unit. Output: $OUT"
fi

# 4. User Engagement (Quiz & Rewards)
log_step "User Engagement: Submit Quiz"
# Submit correct answer (index 0)
OUT=$(dfx canister call user_profile submit_quiz '("unit_test_1", vec { 0 })' 2>&1)

if [[ "$OUT" == *"Ok"* ]]; then
    log_result "PASS" "Quiz submitted successfully."
else
    log_result "FAIL" "Quiz submission failed. Output: $OUT"
fi

# 5. Verification: Local Balance
log_step "Verification: Check User Balance (1 Token)"
IDENTITY=$(dfx identity get-principal)
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$IDENTITY\")")

if [[ "$PROFILE" == *"staked_balance = 100_000_000"* ]]; then
    log_result "PASS" "User balance updated to 100,000,000 (1 Token)."
else
    log_result "FAIL" "User balance incorrect. Output: $PROFILE"
fi

# 6. Unstaking Flow
log_step "Economy: Unstake 0.5 Token"
# Unstake 50_000_000 (0.5 Token)
# Penalty: 5_000_000 (0.05 Token)
# Receive: 45_000_000 (0.45 Token)
OUT=$(dfx canister call user_profile unstake '(50_000_000)' 2>&1)

if [[ "$OUT" == *"Ok"* ]]; then
    log_result "PASS" "Unstake call successful."
else
    log_result "FAIL" "Unstake failed. Output: $OUT"
fi

# 7. Verification: Balance after Unstake
log_step "Verification: Check User Balance (0.5 Token)"
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$IDENTITY\")")

if [[ "$PROFILE" == *"staked_balance = 50_000_000"* ]]; then
    log_result "PASS" "User balance reduced to 50,000,000 (0.5 Token)."
else
    log_result "FAIL" "User balance incorrect after unstake. Output: $PROFILE"
fi

# 8. Verification: Global Stats & Interest Pool
log_step "Verification: Force Sync & Check Global Stats"
dfx canister call user_profile debug_force_sync >/dev/null

STATS=$(dfx canister call staking_hub get_global_stats)
# Expect interest_pool = 5_000_000
# Expect total_unstaked = 50_000_000

if [[ "$STATS" == *"interest_pool = 5_000_000"* ]]; then
    log_result "PASS" "Interest Pool updated correctly (Penalty collected)."
else
    log_result "FAIL" "Interest Pool incorrect. Stats: $STATS"
fi

if [[ "$STATS" == *"total_unstaked = 50_000_000"* ]]; then
    log_result "PASS" "Total Unstaked updated correctly."
else
    log_result "FAIL" "Total Unstaked incorrect. Stats: $STATS"
fi

# 9. Interest Distribution
log_step "Economy: Distribute Interest"
OUT=$(dfx canister call staking_hub distribute_interest 2>&1)

if [[ "$OUT" == *"Ok"* ]]; then
    log_result "PASS" "Interest distributed successfully."
else
    log_result "FAIL" "Interest distribution failed. Output: $OUT"
fi

# 10. Verification: Pool Reset
log_step "Verification: Interest Pool Reset"
STATS=$(dfx canister call staking_hub get_global_stats)

if [[ "$STATS" == *"interest_pool = 0"* ]]; then
    log_result "PASS" "Interest Pool reset to 0."
else
    log_result "FAIL" "Interest Pool not reset. Stats: $STATS"
fi

echo -e "\n-----------------------------------" >> $REPORT_FILE
echo "End of Report" >> $REPORT_FILE

echo -e "${GREEN}Test Suite Completed. Check test_report.md for details.${NC}"
cat test_report.md
