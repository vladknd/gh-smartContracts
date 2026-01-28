#!/bin/bash

# Script to test the KYC flow across Staking Hub, KYC Canister, and User Shards.

set -e

# 1. Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting KYC Flow Test...${NC}"

# 2. Get Principal IDs
STAKING_HUB_ID=$(dfx canister id staking_hub)
KYC_CANISTER_ID=$(dfx canister id kyc_canister)
USER_PROFILE_ID=$(dfx canister id user_profile)
CALLER_PRINCIPAL=$(dfx identity get-principal)

echo "Staking Hub: $STAKING_HUB_ID"
echo "KYC Canister: $KYC_CANISTER_ID"
echo "User Profile Shard: $USER_PROFILE_ID"
echo "My Principal: $CALLER_PRINCIPAL"

# 3. Setup: Broadcast the KYC Manager ID to the Shards via the Hub
echo -e "\n${GREEN}[Step 1] Broadcasting KYC Manager ID from Hub...${NC}"
dfx canister call staking_hub admin_broadcast_kyc_manager "(principal \"$KYC_CANISTER_ID\")"

# Verify shard received the ID
SHARD_MANAGER_ID=$(dfx canister call user_profile get_kyc_manager_id | grep -oP 'principal "\K[^"]+')
if [ "$SHARD_MANAGER_ID" == "$KYC_CANISTER_ID" ]; then
    echo -e "${GREEN}Success: Shard is now trusting the KYC Canister.${NC}"
else
    echo -e "${RED}Error: Shard manager mismatch. Got $SHARD_MANAGER_ID, expected $KYC_CANISTER_ID${NC}"
    exit 1
fi

# 4. Mock User Registration
echo -e "\n${GREEN}[Step 2] Registering user in shard...${NC}"
dfx canister call user_profile register_user '(record { email="kyc@example.com"; name="KYC User"; education="Master"; gender="Male" })' || echo "User already registered"

# 5. Submit KYC Data
echo -e "\n${GREEN}[Step 3] Submitting KYC data...${NC}"
dfx canister call kyc_canister submit_kyc_data '("Passport ID: 123456789")'

# 6. Trigger AI Verification
echo -e "\n${GREEN}[Step 4] Triggering AI verification (triggers shard update)...${NC}"
dfx canister call kyc_canister verify_identity "(principal \"$CALLER_PRINCIPAL\")"

# 7. Final Assertions
echo -e "\n${GREEN}[Step 5] Verifying user KYC status on shard...${NC}"
PROFILE_RESULT=$(dfx canister call user_profile get_profile "(principal \"$CALLER_PRINCIPAL\")")
echo "Profile: $PROFILE_RESULT"

if echo "$PROFILE_RESULT" | grep -q "verification_tier = variant { KYC }"; then
    echo -e "${GREEN}SUCCESS: User is now KYC verified on the immutable shard!${NC}"
else
    echo -e "${RED}FAILURE: User is NOT KYC verified on the shard.${NC}"
    exit 1
fi

echo -e "\n${GREEN}[Step 6] Testing Security: Unauthorized KYC update attempt...${NC}"
# Attempting to call shard directly as ourselves
HACKER_ATTEMPT=$(dfx canister call user_profile internal_set_kyc_status "(principal \"$CALLER_PRINCIPAL\", variant { KYC })" 2>&1 || true)

if echo "$HACKER_ATTEMPT" | grep -q "Unauthorized"; then
    echo -e "${GREEN}SUCCESS: Unauthorized attempt was blocked.${NC}"
else
    echo -e "${RED}FAILURE: Unauthorized attempt was NOT blocked! Output: $HACKER_ATTEMPT${NC}"
    exit 1
fi

echo -e "\n${GREEN}All KYC flow tests passed!${NC}"
