#!/bin/bash

# Script to test the subscription flow across Staking Hub, Subscription Canister, and User Shards.

set -e

# 1. Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Subscription Flow Test...${NC}"

# 2. Get Principal IDs
STAKING_HUB_ID=$(dfx canister id staking_hub)
SUBSCRIPTION_CANISTER_ID=$(dfx canister id subscription_canister)
USER_PROFILE_ID=$(dfx canister id user_profile)
CALLER_PRINCIPAL=$(dfx identity get-principal)

echo "Staking Hub: $STAKING_HUB_ID"
echo "Subscription Canister: $SUBSCRIPTION_CANISTER_ID"
echo "User Profile Shard: $USER_PROFILE_ID"
echo "My Principal: $CALLER_PRINCIPAL"

# 3. Setup: Broadcast the Subscription Manager ID to the Shards via the Hub
echo -e "\n${GREEN}[Step 1] Broadcasting Subscription Manager ID from Hub...${NC}"
dfx canister call staking_hub admin_broadcast_subscription_manager "(principal \"$SUBSCRIPTION_CANISTER_ID\")"

# Verify shard received the ID
SHARD_MANAGER_ID=$(dfx canister call user_profile get_subscription_manager_id | grep -oP 'principal "\K[^"]+')
if [ "$SHARD_MANAGER_ID" == "$SUBSCRIPTION_CANISTER_ID" ]; then
    echo -e "${GREEN}Success: Shard is now trusting the Subscription Canister.${NC}"
else
    echo -e "${RED}Error: Shard manager mismatch. Got $SHARD_MANAGER_ID, expected $SUBSCRIPTION_CANISTER_ID${NC}"
    exit 1
fi

# 4. Mock User Registration
echo -e "\n${GREEN}[Step 2] Registering user in shard...${NC}"
dfx canister call user_profile register_user '(record { email="test@example.com"; name="Sub User"; education="Test"; gender="Other" })' || echo "User already registered"

# 5. Initiate Checkout
echo -e "\n${GREEN}[Step 3] Initiating checkout from Subscription Canister...${NC}"
CHECKOUT_RESULT=$(dfx canister call subscription_canister request_checkout "(principal \"$USER_PROFILE_ID\")")
echo "Result: $CHECKOUT_RESULT"

# Extract session ID from mock URL (e.g. sess_...)
SESSION_ID=$(echo $CHECKOUT_RESULT | grep -oP 'pay/\Ksess_[^"]+')
echo "Extracted Session ID: $SESSION_ID"

# 6. Verify Payment (Simulation)
echo -e "\n${GREEN}[Step 4] Confirming payment (triggers shard update)...${NC}"
dfx canister call subscription_canister confirm_payment "(\"$SESSION_ID\")"

# 7. Final Assertions
echo -e "\n${GREEN}[Step 5] Verifying user subscription status on shard...${NC}"
IS_SUBSCRIBED=$(dfx canister call user_profile get_profile "(principal \"$CALLER_PRINCIPAL\")" | grep "is_subscribed = true")

if [ -n "$IS_SUBSCRIBED" ]; then
    echo -e "${GREEN}SUCCESS: User is now subscribed on the immutable shard!${NC}"
else
    echo -e "${RED}FAILURE: User is NOT subscribed on the shard.${NC}"
    exit 1
fi

echo -e "\n${GREEN}[Step 6] Testing Security: Unauthorized activation attempt...${NC}"
# Use a different identity or just try to call shard from terminal (which is you, not the sub canister)
HACKER_ATTEMPT=$(dfx canister call user_profile internal_set_subscription "(principal \"$CALLER_PRINCIPAL\", true)" 2>&1 || true)

if [[ $HACKER_ATTEMPT == *"Unauthorized"* ]]; then
    echo -e "${GREEN}SUCCESS: Unauthorized attempt was blocked.${NC}"
else
    echo -e "${RED}FAILURE: Unauthorized attempt was NOT blocked! Output: $HACKER_ATTEMPT${NC}"
    exit 1
fi

echo -e "\n${GREEN}All subscription tests passed!${NC}"
