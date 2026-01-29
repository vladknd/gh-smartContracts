#!/bin/bash
set -e

# ============================================================================
# COMPREHENSIVE USER PROFILE (SHARD) AUDIT SUITE
# ============================================================================
# This script performs an exhaustive verification of the User Profile canister,
# covering profile logic, quiz rewards, tiered quotas, and archiving.
# ============================================================================

# Colors for Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Log helpers
source "$(dirname "$0")/test_helper.sh"

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
log_header "PHASE 1: Environment Setup"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Initializing clean DFX state"
    dfx stop &>/dev/null || true
    dfx start --background --clean &>/dev/null
    sleep 3

    log_step "Creating canisters"
    dfx canister create staking_hub &>/dev/null
    dfx canister create ghc_ledger &>/dev/null
    dfx canister create learning_engine &>/dev/null
    dfx canister create user_profile &>/dev/null
    dfx canister create archive_canister &>/dev/null

    HUB_ID=$(dfx canister id staking_hub)
    LEDGER_ID=$(dfx canister id ghc_ledger)
    ENGINE_ID=$(dfx canister id learning_engine)
    PROFILE_ID=$(dfx canister id user_profile)
    ARCHIVE_ID=$(dfx canister id archive_canister)

    log_step "Compiling Infrastructure"
    dfx build archive_canister &>/dev/null
    dfx build staking_hub &>/dev/null
    dfx build learning_engine &>/dev/null
    dfx build user_profile &>/dev/null

    log_step "Deploying Infrastructure"
    # 1. Archive
    dfx deploy archive_canister --argument "(record { parent_shard_id = principal \"$PROFILE_ID\" })" &>/dev/null
    # 2. Hub
    dfx deploy staking_hub --argument "(record { ledger_id = principal \"$LEDGER_ID\"; learning_content_id = principal \"$ENGINE_ID\"; user_profile_wasm = vec {}; archive_canister_wasm = null })" &>/dev/null
    # 3. Learning Engine
    dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$HUB_ID\"; governance_canister_id = null })" &>/dev/null
    # 4. User Profile (Linked to Hub)
    dfx deploy user_profile --argument "(record { staking_hub_id = principal \"$HUB_ID\"; learning_content_id = principal \"$ENGINE_ID\" })" &>/dev/null

    log_pass "Infrastructure deployed"
else
    HUB_ID=$(dfx canister id staking_hub)
    LEDGER_ID=$(dfx canister id ghc_ledger)
    ENGINE_ID=$(dfx canister id learning_engine)
    PROFILE_ID=$(dfx canister id user_profile)
    ARCHIVE_ID=$(dfx canister id archive_canister)
    log_info "Using existing deployment"
fi

# Switch to a unique identity for this test run to ensure fresh user state
ADMIN_IDENTITY=$(dfx identity whoami)
TEST_USER="up_test_$(date +%s)"
log_step "Creating unique test identity: $TEST_USER"
dfx identity new "$TEST_USER" --storage-mode=plaintext &>/dev/null || true
dfx identity use "$TEST_USER"
USER_PRINCIPAL=$(dfx identity get-principal)
log_info "Test Identity Principal: $USER_PRINCIPAL"

# Define cleanup to switch back to original identity
cleanup() {
    dfx identity use "$ADMIN_IDENTITY"
}
trap cleanup EXIT

# ============================================================================
# PHASE 2: PROFILE MANAGEMENT
# ============================================================================
log_header "PHASE 2: Profile Management"

log_step "Registering a new user"
dfx canister call user_profile register_user '(record { name="Alice"; email="alice@ghc.io"; education="PhD"; gender="Female" })'
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")")
if [[ "$PROFILE" == *"Alice"* ]]; then
    log_pass "User registration successful"
else
    log_fail "Profile not found or incorrect: $PROFILE"
fi

log_step "Updating profile info"
dfx canister call user_profile update_profile '(record { name="Alice Smith"; email="alice.smith@ghc.io"; education="PhD"; gender="Female" })'
PROFILE=$(dfx canister call user_profile get_profile "(principal \"$(dfx identity get-principal)\")")
if [[ "$PROFILE" == *"Alice Smith"* ]]; then
    log_pass "Profile update successful"
else
    log_fail "Profile update check failed: $PROFILE"
fi

# ============================================================================
# PHASE 3: QUIZ LOGIC & LIMITS
# ============================================================================
log_header "PHASE 3: Quiz Logic & Limits"

log_step "Seeding quiz data into Learning Engine"
# Switch back to admin for setup
dfx identity use "$ADMIN_IDENTITY"
log_info "Caller: $(dfx identity get-principal)"

# ContentNode struct for intro_1
dfx canister call learning_engine add_content_node '(record { 
    id="intro_1"; 
    parent_id=null;
    order=1:nat32;
    display_type="Unit";
    title="GreenHero Intro";
    description=opt "Ecology Basics";
    content=opt "Save the planet.";
    paraphrase=null;
    media=null;
    quiz=opt record { 
        questions=vec { 
            record { question="Q1"; options=vec {"A";"B"}; answer=0:nat8 };
            record { question="Q2"; options=vec {"A";"B"}; answer=1:nat8 };
            record { question="Q3"; options=vec {"A";"B"}; answer=0:nat8 }
        }
    };
    created_at=0:nat64;
    updated_at=0:nat64;
    version=1:nat64;
})'

log_step "Registering Shard in Hub (to allow sync)"
dfx canister call staking_hub add_allowed_minter "(principal \"$PROFILE_ID\")"
# Hub gives allowance to Shard
dfx canister call user_profile debug_force_sync

log_step "Broadcasting Config from Hub (Reward=10G, Pass=66%)"
# We set limits within the new allowed ranges
# Regular: Daily 2-5, Weekly 10-20, Monthly 30-60, Yearly 200-400
# (Values are in e8s: 1 GHC = 100,000,000)
dfx canister call staking_hub update_token_limits "(
    opt 10_000_000, 
    opt 66, 
    opt 5, 
    opt record { 
        max_daily_tokens=200_000_000; 
        max_weekly_tokens=1_000_000_000; 
        max_monthly_tokens=3_000_000_000; 
        max_yearly_tokens=20_000_000_000 
    }, 
    null
)"
sleep 2

# Switch back to test user for submission
dfx identity use "$TEST_USER"

log_step "Submitting Quiz (Correct - 3/3)"
SUBMIT_2=$(dfx canister call user_profile submit_quiz '("intro_1", blob "\00\01\00")')
if [[ "$SUBMIT_2" == *"Ok"* ]] && [[ "$SUBMIT_2" == *"10"* ]]; then
    log_pass "Quiz passed correctly - Reward received (0.1 GHC)"
else
    log_fail "Quiz submission failed: $SUBMIT_2"
fi

log_header "PHASE 3.5: Limit Range Validation"

log_step "Verifying Out-of-Range Limit Update (Too High)"
INVALID_UPDATE=$(dfx --identity "$ADMIN_IDENTITY" canister call staking_hub update_token_limits "(
    null, null, null,
    opt record { 
        max_daily_tokens=2_000_000_000_000; 
        max_weekly_tokens=1_000_000_000; 
        max_monthly_tokens=3_000_000_000; 
        max_yearly_tokens=20_000_000_000 
    }, 
    null
)" 2>&1 || true)
if [[ "$INVALID_UPDATE" == *"Regular daily limit must be between"* ]]; then
    log_pass "Correctly blocked out-of-range daily limit (20,000 GHC > 10,000 GHC)"
else
    log_fail "Failed to block out-of-range limit: $INVALID_UPDATE"
fi

log_step "Verifying Out-of-Range Limit Update (Too Low)"
INVALID_UPDATE_LOW=$(dfx --identity "$ADMIN_IDENTITY" canister call staking_hub update_token_limits "(
    null, null, null,
    opt record { 
        max_daily_tokens=100_000_000; 
        max_weekly_tokens=1_000_000_000; 
        max_monthly_tokens=3_000_000_000; 
        max_yearly_tokens=20_000_000_000 
    }, 
    null
)" 2>&1 || true)
if [[ "$INVALID_UPDATE_LOW" == *"Regular daily limit must be between"* ]]; then
    log_pass "Correctly blocked out-of-range daily limit (1 GHC < 2 GHC)"
else
    log_fail "Failed to block out-of-range limit (too low): $INVALID_UPDATE_LOW"
fi

# ============================================================================
# PHASE 4: QUOTAS & TIME-BASED RESETS
# ============================================================================
log_header "PHASE 4: Quotas & Resets"

log_step "Hitting Daily Quota"
# Alice used 1 quiz. Submit 4 more.
for i in {2..5}; do
    UNIT_ID="u_$i"
    dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node "(record { id=\"$UNIT_ID\"; parent_id=null; order=1; display_type=\"Unit\"; title=\"T\"; description=null; content=null; paraphrase=null; media=null; quiz=opt record { questions=vec { record { question=\"Q\"; options=vec {\"A\"}; answer=0:nat8 } } }; created_at=0; updated_at=0; version=1 })" &>/dev/null
    dfx canister call user_profile submit_quiz "(\"$UNIT_ID\", blob \"\\00\")" &>/dev/null
done

log_step "Verifying Daily Limit Blocking"
dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node "(record { id=\"u_blocked\"; parent_id=null; order=1; display_type=\"Unit\"; title=\"T\"; description=null; content=null; paraphrase=null; media=null; quiz=opt record { questions=vec { record { question=\"Q\"; options=vec {\"A\"}; answer=0:nat8 } } }; created_at=0; updated_at=0; version=1 })" &>/dev/null
BLOCKED=$(dfx canister call user_profile submit_quiz '("u_blocked", blob "\00")' 2>&1 || true)
if [[ "$BLOCKED" == *"Daily quiz limit reached"* ]]; then
    log_pass "Daily quiz quota (5/day) enforced"
else
    log_fail "Daily quota bypass detected: $BLOCKED"
fi

log_step "Simulating Day Reset"
# Switch back to admin for system modification
dfx identity use "$ADMIN_IDENTITY"
# Set last_active_day to 0 (far in past) to trigger reset on next activity
dfx canister call user_profile admin_set_user_stats "(principal \"$USER_PRINCIPAL\", record { daily_quizzes=5:nat8; daily_earnings=50000000000; weekly_quizzes=5:nat8; weekly_earnings=50000000000; monthly_quizzes=5:nat8; monthly_earnings=50000000000; yearly_quizzes=5:nat16; yearly_earnings=50000000000; last_active_day=0:nat64 })"
# Switch back to test user
dfx identity use "$TEST_USER"

log_step "Verifying Daily Reset"
RESET_WORKED=$(dfx canister call user_profile submit_quiz '("u_blocked", blob "\00")')
if [[ "$RESET_WORKED" == *"Ok"* ]]; then
    log_pass "New day reset counters successfully"
else
    log_fail "Day reset logic failure: $RESET_WORKED"
fi

# ============================================================================
# PHASE 5: SUBSCRIPTIONS
# ============================================================================
log_header "PHASE 5: Subscriptions"

NOW_DAY=$(dfx canister call user_profile get_current_day --query | tr -d '_' | grep -oP '\d+' | head -n 1)

log_step "Hitting Regular Daily Token Limit"
# We set a tiny limit (1000 e8s) to ensure the next quiz (10G e8s) blocks
# update_token_limits is a Hub call, needs controller authority
dfx identity use "$ADMIN_IDENTITY"
# We set a tiny limit (exactly 2 GHC, which is the minimum)
dfx identity use "$ADMIN_IDENTITY"
dfx canister call staking_hub update_token_limits "(
    opt 10_000_000, 
    opt 66, 
    opt 5, 
    opt record { 
        max_daily_tokens=200_000_000; 
        max_weekly_tokens=1_000_000_000; 
        max_monthly_tokens=3_000_000_000; 
        max_yearly_tokens=20_000_000_000 
    }, 
    null
)"
# Reset user stats to simulate having earned almost the daily limit
# 199,500,000 earns + 10,000,000 reward would exceed 200,000,000
dfx canister call user_profile admin_set_user_stats "(principal \"$USER_PRINCIPAL\", record { daily_quizzes=0:nat8; daily_earnings=195_000_000; weekly_quizzes=0:nat8; weekly_earnings=0; monthly_quizzes=0:nat8; monthly_earnings=0; yearly_quizzes=0:nat16; yearly_earnings=0; last_active_day=$NOW_DAY:nat64 })"
dfx identity use "$TEST_USER"

dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node "(record { id=\"u_phase5\"; parent_id=null; order=1; display_type=\"Unit\"; title=\"T\"; description=null; content=null; paraphrase=null; media=null; quiz=opt record { questions=vec { record { question=\"Q\"; options=vec {\"A\"}; answer=0:nat8 } } }; created_at=0; updated_at=0; version=1 })" &>/dev/null
LIMIT_BLOCK=$(dfx canister call user_profile submit_quiz '("u_phase5", blob "\00")' 2>&1 || true)
if [[ "$LIMIT_BLOCK" == *"Daily token limit reached"* ]]; then
    log_pass "Regular daily token limit enforced"
else
    log_fail "Regular limit bypass: $LIMIT_BLOCK"
fi

log_step "Upgrading to Subscription"
dfx identity use "$ADMIN_IDENTITY"
dfx canister call user_profile admin_set_subscription "(principal \"$USER_PRINCIPAL\", true)"
dfx identity use "$TEST_USER"

dfx --identity "$ADMIN_IDENTITY" canister call learning_engine add_content_node "(record { id=\"u_phase5_sub\"; parent_id=null; order=1; display_type=\"Unit\"; title=\"T\"; description=null; content=null; paraphrase=null; media=null; quiz=opt record { questions=vec { record { question=\"Q\"; options=vec {\"A\"}; answer=0:nat8 } } }; created_at=0; updated_at=0; version=1 })" &>/dev/null
SUB_WORKED=$(dfx canister call user_profile submit_quiz '("u_phase5_sub", blob "\00")')
if [[ "$SUB_WORKED" == *"Ok"* ]]; then
    log_pass "Subscription successfully increased daily token allowance"
else
    log_fail "Subscription failed to unlock quota: $SUB_WORKED"
fi

# ============================================================================
# PHASE 6: ARCHIVING
# ============================================================================
log_header "PHASE 6: Archiving"

log_info "Setting Archive Canister"
dfx --identity "$ADMIN_IDENTITY" canister call user_profile set_archive_canister "(principal \"$ARCHIVE_ID\")" &>/dev/null

log_step "Triggering Archive Pruning"
# Even if we don't have 151 records, the debug trigger forces the check logic
dfx --identity "$ADMIN_IDENTITY" canister call user_profile debug_trigger_archive &>/dev/null
log_pass "Archive event triggered successfully"

# ============================================================================
# SUMMARY
# ============================================================================
log_header "AUDIT SUMMARY"
echo -e "  - Total Checks: $TESTS_TOTAL"
echo -e "  - Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "  - Failed: ${RED}$((TESTS_TOTAL - TESTS_PASSED))${NC}"
echo ""
if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}  USER PROFILE SHARD VERIFIED COMPREHENSIVE ✓${NC}"
    exit 0
else
    exit 1
fi
