#!/bin/bash

# Destination Directories
DASHBOARD_ROOT="/mnt/c/LIB/CODE/gh-dashboard"
DIDS_DIR="$DASHBOARD_ROOT/dids"
CONFIG_FILE="$DASHBOARD_ROOT/ic.config.json"

# Ensure directories exist
mkdir -p "$DIDS_DIR"

echo "Copying DID files..."

# Copy DID files
cp src/user_profile/user_profile.did "$DIDS_DIR/"
cp src/staking_hub/staking_hub.did "$DIDS_DIR/"
cp src/learning_engine/learning_engine.did "$DIDS_DIR/"
cp src/operational_governance/operational_governance.did "$DIDS_DIR/"
cp src/content_governance/content_governance.did "$DIDS_DIR/"
# Add ledger DID if available
# cp src/ghc_ledger/ghc_ledger.did "$DIDS_DIR/" 

echo "DID files copied to $DIDS_DIR"

echo "Copying IC configuration..."

# Copy the entire ic.config.json file
# This includes network, host, all canister IDs, and founder addresses
cp ic.config.json "$CONFIG_FILE"

echo "IC configuration copied to $CONFIG_FILE"

echo "Copying integration documentation..."

# Create docs directory if it doesn't exist
DOCS_DIR="$DASHBOARD_ROOT/docs"
mkdir -p "$DOCS_DIR"

# Copy Frontend Integration guide
cp docs/FRONTEND_INTEGRATION.md "$DOCS_DIR/"

echo "Integration documentation copied to $DOCS_DIR"
echo "Done!"
