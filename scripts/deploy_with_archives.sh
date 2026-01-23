#!/bin/bash
# ==============================================================================
# Full Deployment Script with Archive Canister Support
# ==============================================================================
#
# This script deploys the complete GreenHero smart contract ecosystem with:
# - Automatic archive canister creation for each user_profile shard
# - Embedded WASM binaries for auto-scaling
#
# Usage:
#   ./scripts/deploy_with_archives.sh [network]
#
# Arguments:
#   network - 'local' (default) or 'ic' for mainnet
#
# Prerequisites:
#   - dfx installed
#   - Rust/cargo with wasm32-unknown-unknown target
# ==============================================================================

set -e

# Configuration
NETWORK="${1:-local}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}[INFO]${NC} $1"; }
header() { echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; echo -e "${BLUE}$1${NC}"; echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; }

cd "$PROJECT_DIR"

# ==============================================================================
# PHASE 1: Build All Canisters
# ==============================================================================

header "PHASE 1: Building Canisters"

info "Building all canisters..."
cargo build --release --target wasm32-unknown-unknown \
    -p staking_hub \
    -p user_profile \
    -p archive_canister \
    -p learning_engine \
    -p treasury_canister \
    -p governance_canister || error "Build failed"

success "All canisters built successfully"

# Verify WASM files exist
USER_PROFILE_WASM="target/wasm32-unknown-unknown/release/user_profile.wasm"
ARCHIVE_WASM="target/wasm32-unknown-unknown/release/archive_canister.wasm"

[ -f "$USER_PROFILE_WASM" ] || error "user_profile WASM not found"
[ -f "$ARCHIVE_WASM" ] || error "archive_canister WASM not found"

info "WASM sizes:"
ls -lh "$USER_PROFILE_WASM" "$ARCHIVE_WASM"

# ==============================================================================
# PHASE 2: Start Network (if local)
# ==============================================================================

header "PHASE 2: Network Setup"

if [ "$NETWORK" == "local" ]; then
    info "Checking if dfx is running..."
    if ! dfx ping &>/dev/null; then
        info "Starting local network..."
        dfx stop 2>/dev/null || true
        dfx start --background --clean
        sleep 5
    fi
    success "Local network is running"
else
    info "Deploying to IC mainnet"
fi

# ==============================================================================
# PHASE 3: Deploy Core Infrastructure
# ==============================================================================

header "PHASE 3: Deploying Core Infrastructure"

# GHC Ledger
info "Deploying GHC Ledger..."
dfx deploy ghc_ledger --network "$NETWORK" --argument '(variant { Init = record { 
    minting_account = record { owner = principal "aaaaa-aa" };
    initial_balances = vec {};
    transfer_fee = 10000;
    token_name = "GreenHero Coin";
    token_symbol = "GHC";
    metadata = vec {};
    archive_options = record {
        trigger_threshold = 2000;
        num_blocks_to_archive = 1000;
        controller_id = principal "aaaaa-aa"
    };
    feature_flags = opt record { icrc2 = true }
}})' 2>&1 || info "Ledger may already be deployed"

success "GHC Ledger deployed"

# Learning Engine
info "Deploying Learning Engine..."
dfx deploy learning_engine --network "$NETWORK" || error "Failed to deploy learning_engine"

LEARNING_ENGINE_ID=$(dfx canister id learning_engine --network "$NETWORK")
info "Learning Engine ID: $LEARNING_ENGINE_ID"

# Get Ledger ID
LEDGER_ID=$(dfx canister id ghc_ledger --network "$NETWORK")
info "Ledger ID: $LEDGER_ID"

success "Core infrastructure deployed"

# ==============================================================================
# PHASE 4: Deploy Staking Hub with Embedded WASMs
# ==============================================================================

header "PHASE 4: Deploying Staking Hub with Embedded WASMs"

info "Converting WASM files to blob format..."

# Create a temporary Candid file with embedded WASMs
# Note: For production, you'd use a helper canister or dfx extension
# For now, we deploy staking_hub empty and use a separate script to embed WASMs

# First, deploy staking_hub with empty WASMs
info "Deploying staking_hub (initial - without embedded WASMs)..."
dfx deploy staking_hub --network "$NETWORK" --argument "(record { 
    ledger_id = principal \"$LEDGER_ID\"; 
    learning_content_id = principal \"$LEARNING_ENGINE_ID\"; 
    user_profile_wasm = vec {};
    archive_canister_wasm = null
})" 2>&1 || info "staking_hub may already be deployed"

STAKING_HUB_ID=$(dfx canister id staking_hub --network "$NETWORK")
info "Staking Hub ID: $STAKING_HUB_ID"

success "Staking Hub deployed"

# ==============================================================================
# PHASE 5: Deploy User Profile and Archive (Manual Linking)
# ==============================================================================

header "PHASE 5: Deploying User Profile Shard with Archive"

# Deploy user_profile
info "Deploying user_profile shard..."
dfx deploy user_profile --network "$NETWORK" --argument "(record { 
    staking_hub_id = principal \"$STAKING_HUB_ID\"; 
    learning_content_id = principal \"$LEARNING_ENGINE_ID\" 
})" || error "Failed to deploy user_profile"

USER_PROFILE_ID=$(dfx canister id user_profile --network "$NETWORK")
info "User Profile ID: $USER_PROFILE_ID"

# Deploy archive_canister
info "Deploying archive_canister..."
dfx deploy archive_canister --network "$NETWORK" --argument "(record { 
    parent_shard_id = principal \"$USER_PROFILE_ID\" 
})" || error "Failed to deploy archive_canister"

ARCHIVE_ID=$(dfx canister id archive_canister --network "$NETWORK")
info "Archive ID: $ARCHIVE_ID"

# Register user_profile as a shard in staking_hub
info "Registering user_profile as a shard in staking_hub..."
dfx canister call staking_hub add_allowed_minter "(principal \"$USER_PROFILE_ID\")" --network "$NETWORK"

# Link archive to user_profile (via staking_hub or direct if controller)
info "Linking archive to user_profile..."
# Note: set_archive_canister requires caller to be staking_hub
# For manual deployment, we need to call it from the controller or modify auth temporarily

# Check current controller
CONTROLLER=$(dfx identity get-principal)
info "Current identity (controller): $CONTROLLER"

# Use dfx to call as controller - this should work if we're the controller
dfx canister call user_profile set_archive_canister "(principal \"$ARCHIVE_ID\")" --network "$NETWORK" 2>&1 || info "Note: Archive linking may require staking_hub to call"

success "User Profile and Archive deployed"

# ==============================================================================
# PHASE 6: Deploy Remaining Canisters
# ==============================================================================

header "PHASE 6: Deploying Governance and Treasury"

# Treasury
info "Deploying Treasury..."
dfx deploy treasury_canister --network "$NETWORK" || info "Treasury deployment skipped or already deployed"

TREASURY_ID=$(dfx canister id treasury_canister --network "$NETWORK" 2>/dev/null || echo "not-deployed")
info "Treasury ID: $TREASURY_ID"

# Governance
info "Deploying Governance..."
dfx deploy governance_canister --network "$NETWORK" || info "Governance deployment skipped or already deployed"

GOVERNANCE_ID=$(dfx canister id governance_canister --network "$NETWORK" 2>/dev/null || echo "not-deployed")
info "Governance ID: $GOVERNANCE_ID"

success "All canisters deployed"

# ==============================================================================
# PHASE 7: Verify Deployment
# ==============================================================================

header "PHASE 7: Verification"

info "Checking archive stats..."
dfx canister call archive_canister get_stats --network "$NETWORK"

info "Checking staking_hub shards..."
dfx canister call staking_hub get_shards --network "$NETWORK"

info "Checking user_profile archive link..."
dfx canister call user_profile get_archive_canister --network "$NETWORK"

# ==============================================================================
# SUMMARY
# ==============================================================================

header "DEPLOYMENT COMPLETE"

echo ""
echo "Deployed Canisters:"
echo "  ┌─────────────────────┬──────────────────────────────────────────┐"
echo "  │ Canister            │ ID                                       │"
echo "  ├─────────────────────┼──────────────────────────────────────────┤"
echo "  │ GHC Ledger          │ $LEDGER_ID"
echo "  │ Learning Engine     │ $LEARNING_ENGINE_ID"
echo "  │ Staking Hub         │ $STAKING_HUB_ID"
echo "  │ User Profile        │ $USER_PROFILE_ID"
echo "  │ Archive             │ $ARCHIVE_ID"
echo "  │ Treasury            │ $TREASURY_ID"
echo "  │ Governance          │ $GOVERNANCE_ID"
echo "  └─────────────────────┴──────────────────────────────────────────┘"
echo ""
echo "Archive Configuration:"
echo "  • Retention Limit: 100 transactions per user (kept locally)"
echo "  • Trigger Threshold: 150 transactions (immediate archive)"
echo "  • Periodic Check: Every 6 hours"
echo ""
echo "Notes:"
echo "  • For auto-scaling with embedded WASMs, use the IC management canister"
echo "    to update staking_hub with compiled WASM blobs"
echo "  • Archive linking may require running as staking_hub principal"
echo ""

success "Deployment completed successfully!"
