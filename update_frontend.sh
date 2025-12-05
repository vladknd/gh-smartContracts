#!/bin/bash

# Destination Directories
DASHBOARD_ROOT="/Users/vladknd/LIB/CODE/gh-dashboard"
DIDS_DIR="$DASHBOARD_ROOT/dids"
IDS_FILE="$DASHBOARD_ROOT/canister_ids.json"

# Ensure directories exist
mkdir -p "$DIDS_DIR"

echo "Copying DID files..."

# Copy DID files
cp src/user_profile/user_profile.did "$DIDS_DIR/"
cp src/staking_hub/staking_hub.did "$DIDS_DIR/"
cp src/learning_engine/learning_engine.did "$DIDS_DIR/"
# Add others as needed
# cp src/ghc_ledger/ghc_ledger.did "$DIDS_DIR/" 

echo "DID files copied to $DIDS_DIR"

echo "Exporting Canister IDs..."

# Get IDs
USER_PROFILE_ID=$(dfx canister id user_profile)
STAKING_HUB_ID=$(dfx canister id staking_hub)
LEARNING_ENGINE_ID=$(dfx canister id learning_engine)
LEDGER_ID=$(dfx canister id ghc_ledger)
II_ID=$(dfx canister id internet_identity)

# Create JSON content
cat > "$IDS_FILE" <<EOF
{
  "user_profile": "$USER_PROFILE_ID",
  "staking_hub": "$STAKING_HUB_ID",
  "learning_engine": "$LEARNING_ENGINE_ID",
  "ghc_ledger": "$LEDGER_ID",
  "internet_identity": "$II_ID"
}
EOF

echo "Canister IDs saved to $IDS_FILE"
echo "Done!"
