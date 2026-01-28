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
    -p governance_canister \
    -p ico_canister \
    -p founder_vesting \
    -p staging_assets \
    -p media_assets \
    -p subscription_canister \
    -p kyc_canister || error "Build failed"

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

# Create canisters to get IDs
info "Creating canisters..."
dfx canister create --all --network "$NETWORK"

LEDGER_ID=$(dfx canister id ghc_ledger --network "$NETWORK")
STAKING_HUB_ID=$(dfx canister id staking_hub --network "$NETWORK")
TREASURY_ID=$(dfx canister id treasury_canister --network "$NETWORK")
GOVERNANCE_ID=$(dfx canister id governance_canister --network "$NETWORK")
LEARNING_ID=$(dfx canister id learning_engine --network "$NETWORK")
USER_PROFILE_ID=$(dfx canister id user_profile --network "$NETWORK")
ARCHIVE_ID=$(dfx canister id archive_canister --network "$NETWORK")
ICO_ID=$(dfx canister id ico_canister --network "$NETWORK")
FOUNDER_VESTING_ID=$(dfx canister id founder_vesting --network "$NETWORK")
STAGING_ASSETS_ID=$(dfx canister id staging_assets --network "$NETWORK")
MEDIA_ASSETS_ID=$(dfx canister id media_assets --network "$NETWORK")
SUBSCRIPTION_CANISTER_ID=$(dfx canister id subscription_canister --network "$NETWORK")
KYC_CANISTER_ID=$(dfx canister id kyc_canister --network "$NETWORK")
DEFAULT=$(dfx identity get-principal)

info "Canister IDs:"
info "  Ledger: $LEDGER_ID"
info "  Staking Hub: $STAKING_HUB_ID"
info "  Treasury: $TREASURY_ID"
info "  Learning Engine: $LEARNING_ID"

# GHC Ledger - Tokenomics (9.5B Total)
info "Deploying GHC Ledger..."

# Token allocations (in e8s)
TREASURY_AMT="425000000000000000"           # 4.25B * 10^8
HUB_AMT="475000000000000000"                # 4.75B * 10^8
FOUNDER_AMT="50000000000000000"             # 0.5B * 10^8

dfx deploy ghc_ledger --network "$NETWORK" --argument "(variant { Init = record { 
    token_symbol = \"GHC\";
    token_name = \"GreenHero Coin\";
    decimals = opt 8;
    minting_account = record { owner = principal \"$DEFAULT\"; subaccount = null; };
    transfer_fee = 0;
    metadata = vec {};
    initial_balances = vec {
        record { record { owner = principal \"$TREASURY_ID\"; subaccount = null; }; $TREASURY_AMT : nat };
        record { record { owner = principal \"$STAKING_HUB_ID\"; subaccount = null; }; $HUB_AMT : nat };
        record { record { owner = principal \"$FOUNDER_VESTING_ID\"; subaccount = null; }; $FOUNDER_AMT : nat };
    };
    archive_options = record {
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        controller_id = principal \"$DEFAULT\";
    };
    feature_flags = opt record { icrc2 = true; };
 }})"

success "GHC Ledger deployed"

# Internet Identity
info "Deploying Internet Identity..."
dfx deploy internet_identity --network "$NETWORK"

# Learning Engine
info "Deploying Learning Engine..."
dfx deploy learning_engine --network "$NETWORK" --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; governance_canister_id = opt principal \"$GOVERNANCE_ID\" })"

success "Core infrastructure deployed"

# ==============================================================================
# PHASE 4: Deploy Staking Hub with Embedded WASMs
# ==============================================================================

header "PHASE 4: Deploying Staking Hub with Embedded WASMs"

info "Converting WASM files to blob format..."
USER_PROFILE_WASM="target/wasm32-unknown-unknown/release/user_profile.wasm"
ARCHIVE_WASM="target/wasm32-unknown-unknown/release/archive_canister.wasm"

# Use hex for the blob argument
UP_WASM_HEX=$(xxd -p "$USER_PROFILE_WASM" | tr -d '\n' | sed 's/../\\&/g')
AR_WASM_HEX=$(xxd -p "$ARCHIVE_WASM" | tr -d '\n' | sed 's/../\\&/g')

info "Deploying staking_hub with embedded WASMs..."

# Use argument-file to avoid command line length limits
dfx deploy staking_hub --network "$NETWORK" --argument-file <(cat <<EOF
(record { 
    ledger_id = principal "$LEDGER_ID"; 
    learning_content_id = principal "$LEARNING_ID"; 
    user_profile_wasm = blob "$UP_WASM_HEX";
    archive_canister_wasm = opt blob "$AR_WASM_HEX"
})
EOF
)

success "Staking Hub deployed with embedded WASMs"

# ==============================================================================
# PHASE 5: Deploy User Profile and Archive (Manual Linking)
# ==============================================================================

header "PHASE 5: Deploying User Profile Shard with Archive"

# Deploy user_profile
info "Deploying user_profile shard..."
dfx deploy user_profile --network "$NETWORK" --argument "(record { 
    staking_hub_id = principal \"$STAKING_HUB_ID\"; 
    learning_content_id = principal \"$LEARNING_ID\" 
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
dfx canister call staking_hub register_shard "(principal \"$USER_PROFILE_ID\", opt principal \"$ARCHIVE_ID\")" --network "$NETWORK"

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
dfx deploy treasury_canister --network "$NETWORK" --argument "(record { ledger_id = principal \"$LEDGER_ID\"; governance_canister_id = principal \"$GOVERNANCE_ID\" })"

TREASURY_ID=$(dfx canister id treasury_canister --network "$NETWORK" 2>/dev/null || echo "not-deployed")
info "Treasury ID: $TREASURY_ID"

# Governance
info "Deploying Governance..."
dfx deploy governance_canister --network "$NETWORK" --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; treasury_canister_id = principal \"$TREASURY_ID\" })"

GOVERNANCE_ID=$(dfx canister id governance_canister --network "$NETWORK" 2>/dev/null || echo "not-deployed")
info "Governance ID: $GOVERNANCE_ID"

# Link Treasury and Governance
info "Linking Treasury and Governance canisters..."
dfx canister call treasury_canister set_governance_canister_id "(principal \"$GOVERNANCE_ID\")" --network "$NETWORK"
dfx canister call governance_canister set_treasury_canister_id "(principal \"$TREASURY_ID\")" --network "$NETWORK"

# Deploy ICO
info "Deploying ICO Canister..."
dfx deploy ico_canister --network "$NETWORK" --argument "(record { 
    admin_principal = principal \"$DEFAULT\";
    ghc_ledger_id = principal \"$LEDGER_ID\";
    ckusdc_ledger_id = principal \"$LEDGER_ID\";
    price_per_token_e6 = 1000000; 
    ghc_decimals = 8;
    treasury_principal = principal \"$TREASURY_ID\"
})"

# Set Learning Engine ID in Governance
info "Setting Learning Engine ID in Governance Canister..."
dfx canister call governance_canister set_learning_engine_id "(principal \"$LEARNING_ID\")" --network "$NETWORK"

# Deploy Assets
info "Deploying Assets..."
dfx deploy staging_assets --network "$NETWORK" --argument "(record { 
    governance_canister_id = principal \"$GOVERNANCE_ID\";
    learning_engine_id = principal \"$LEARNING_ID\"
})"

dfx deploy media_assets --network "$NETWORK" --argument "(record { 
    allowed_uploaders = vec { principal \"$DEFAULT\" }
})"

# Deploy Founder Vesting
info "Deploying Founder Vesting..."
# Create temporary identities for founders if they don't exist
dfx identity new founder1 || true
dfx identity new founder2 || true
FOUNDER1=$(dfx identity get-principal --identity founder1)
FOUNDER2=$(dfx identity get-principal --identity founder2)

dfx deploy founder_vesting --network "$NETWORK" --argument "(record { 
    ledger_id = principal \"$LEDGER_ID\";
})"

info "Registering founders in Founder Vesting..."
# Founder 1: 350M tokens (in e8s)
dfx canister call founder_vesting admin_register_founder "(principal \"$FOUNDER1\", 35000000000000000)" --network "$NETWORK"
dfx canister call founder_vesting admin_register_founder "(principal \"$FOUNDER2\", 15000000000000000)" --network "$NETWORK"

# Deploy Subscription Canister
info "Deploying Subscription Canister..."
dfx deploy subscription_canister --network "$NETWORK" --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

info "Broadcasting Subscription Canister ID to Shards..."
dfx canister call staking_hub admin_broadcast_subscription_manager "(principal \"$SUBSCRIPTION_CANISTER_ID\")" --network "$NETWORK"

# Deploy KYC Canister
info "Deploying KYC Canister..."
dfx deploy kyc_canister --network "$NETWORK" --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

info "Broadcasting KYC Canister ID to Shards..."
dfx canister call staking_hub admin_broadcast_kyc_manager "(principal \"$KYC_CANISTER_ID\")" --network "$NETWORK"

success "All canisters deployed and linked"

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
echo "  │ Learning Engine     │ $LEARNING_ID"
echo "  │ Staking Hub         │ $STAKING_HUB_ID"
echo "  │ User Profile        │ $USER_PROFILE_ID"
echo "  │ Archive             │ $ARCHIVE_ID"
echo "  │ Treasury            │ $TREASURY_ID"
echo "  │ Governance          │ $GOVERNANCE_ID"
echo "  │ Subscription        │ $SUBSCRIPTION_CANISTER_ID"
echo "  │ KYC                 │ $KYC_CANISTER_ID"
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
