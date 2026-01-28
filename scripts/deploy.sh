#!/bin/bash

# ============================================================================
# GreenHero Coin - Full Deployment Script
# ============================================================================
# This script deploys all canisters and sets up the necessary links between them.
# Run from the project root: ./scripts/deploy.sh
# ============================================================================

# Stop on error
set -e

echo ""
echo "============================================================================"
echo "GreenHero Coin - Full Deployment"
echo "============================================================================"
echo ""

# Check if dfx is running
if ! pgrep -x "dfx" > /dev/null; then
    echo "Starting dfx..."
    dfx start --background --clean
fi

# Create identities
echo "Creating identities..."
dfx identity new founder1 --storage-mode plaintext || true
dfx identity new founder2 --storage-mode plaintext || true
dfx identity new treasury_admin --storage-mode plaintext || true

F1=$(dfx identity get-principal --identity founder1)
F2=$(dfx identity get-principal --identity founder2)
TREASURY_ADMIN=$(dfx identity get-principal --identity treasury_admin)
DEFAULT=$(dfx identity get-principal --identity default)

echo "Founder 1: $F1"
echo "Founder 2: $F2"
echo "Default (Deployer): $DEFAULT"

# Create canisters to get IDs
echo ""
echo "Creating canisters..."
dfx canister create --all

LEDGER_ID=$(dfx canister id ghc_ledger)
STAKING_HUB_ID=$(dfx canister id staking_hub)
TREASURY_ID=$(dfx canister id treasury_canister)
GOVERNANCE_ID=$(dfx canister id governance_canister)
LEARNING_ID=$(dfx canister id learning_engine)
II_ID=$(dfx canister id internet_identity)
FOUNDER_VESTING_ID=$(dfx canister id founder_vesting)
MEDIA_ASSETS_ID=$(dfx canister id media_assets)
STAGING_ASSETS_ID=$(dfx canister id staging_assets)
USER_PROFILE_ID=$(dfx canister id user_profile)
SUBSCRIPTION_CANISTER_ID=$(dfx canister id subscription_canister)
KYC_CANISTER_ID=$(dfx canister id kyc_canister)

echo ""
echo "============================================================================"
echo "Canister IDs"
echo "============================================================================"
echo "  Ledger:              $LEDGER_ID"
echo "  Treasury:            $TREASURY_ID"
echo "  Governance:          $GOVERNANCE_ID"
echo "  Staking Hub:         $STAKING_HUB_ID"
echo "  Learning Engine:     $LEARNING_ID"
echo "  Media Assets:        $MEDIA_ASSETS_ID"
echo "  Staging Assets:      $STAGING_ASSETS_ID"
echo "  User Profile:        $USER_PROFILE_ID"
echo "  Founder Vesting:     $FOUNDER_VESTING_ID"
echo "  Internet Identity:   $II_ID"
echo "  Subscription:        $SUBSCRIPTION_CANISTER_ID"
echo "  KYC Canister:        $KYC_CANISTER_ID"
echo ""

# ============================================================================
# Deploy Ledger - TOKENOMICS (9.5B Total)
# ============================================================================
# Total: 9.5 Billion Tokens
# Decimals: 8 (1 GHC = 100,000,000 smallest units, same as ICP/BTC)
# Transfer Fee: 0
#
# Market Coins (4.75B MC):
#   - Founder Vesting: 0.5B (F1: 0.35B + F2: 0.15B, time-locked 10%/year)
#   - Treasury: 4.25B (initial allowance: 0.6B)
#
# Utility Coins (4.75B MUC):
#   - Staking Hub: 4.75B (for mining rewards)
# ============================================================================

echo "Deploying Internet Identity..."
dfx deploy internet_identity

echo "Deploying Ledger..."

# Token allocations (in e8s)
FOUNDER_VESTING_AMT="50000000000000000"     # 0.5B * 10^8
TREASURY_AMT="425000000000000000"           # 4.25B * 10^8
HUB_AMT="475000000000000000"                # 4.75B * 10^8

INIT_ARGS="(variant { Init = record {
     token_symbol = \"GHC\";
     token_name = \"GreenHero Coin\";
     decimals = opt 8;
     minting_account = record { owner = principal \"$DEFAULT\"; subaccount = null; };
     transfer_fee = 0;
     metadata = vec {};
     initial_balances = vec {
         record { record { owner = principal \"$TREASURY_ID\"; subaccount = null; }; $TREASURY_AMT : nat };
         record { record { owner = principal \"$STAKING_HUB_ID\"; subaccount = null; }; $HUB_AMT : nat };
         record { record { owner = principal \"$FOUNDER_VESTING_ID\"; subaccount = null; }; $FOUNDER_VESTING_AMT : nat };
     };
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 2000;
         controller_id = principal \"$DEFAULT\";
     };
 }})"

dfx deploy ghc_ledger --argument "$INIT_ARGS"

# Deploy ICRC-1 Indexer
echo "Deploying ICRC-1 Indexer..."
dfx deploy icrc1_index_canister --argument "(opt variant { Init = record { ledger_id = principal \"$LEDGER_ID\" } })"

# ============================================================================
# Deploy Core Canisters
# ============================================================================

echo "Deploying Staking Hub..."
dfx deploy staking_hub --argument "(record { ledger_id = principal \"$LEDGER_ID\"; learning_content_id = principal \"$LEARNING_ID\"; user_profile_wasm = vec {}; archive_canister_wasm = null })"

echo "Deploying Treasury Canister..."
dfx deploy treasury_canister --argument "(record { ledger_id = principal \"$LEDGER_ID\"; governance_canister_id = principal \"$GOVERNANCE_ID\" })"

echo "Deploying Governance Canister..."
dfx deploy governance_canister --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; treasury_canister_id = principal \"$TREASURY_ID\" })"

# Link Treasury and Governance
echo "Linking Treasury and Governance canisters..."
dfx canister call treasury_canister set_governance_canister_id "(principal \"$GOVERNANCE_ID\")"
dfx canister call governance_canister set_treasury_canister_id "(principal \"$TREASURY_ID\")"

# ============================================================================
# Deploy Content Governance Canisters
# ============================================================================

echo "Deploying Media Assets (permanent media storage)..."
dfx deploy media_assets --argument "(record { allowed_uploaders = vec {} })"

echo "Deploying Staging Assets (temporary content storage)..."
dfx deploy staging_assets --argument "(record { governance_canister_id = principal \"$GOVERNANCE_ID\"; learning_engine_id = principal \"$LEARNING_ID\" })"

echo "Deploying Learning Engine..."
dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; governance_canister_id = opt principal \"$GOVERNANCE_ID\" })"

# Set Learning Engine ID in Governance
echo "Setting Learning Engine ID in Governance Canister..."
dfx canister call governance_canister set_learning_engine_id "(principal \"$LEARNING_ID\")"

# ============================================================================
# Deploy User Profile & Founder Vesting
# ============================================================================

echo "Deploying User Profile..."
dfx deploy user_profile --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; learning_content_id = principal \"$LEARNING_ID\" })"

echo "Registering User Profile as Allowed Minter..."
dfx canister call staking_hub add_allowed_minter "(principal \"$USER_PROFILE_ID\")"

echo "Deploying Founder Vesting..."
dfx deploy founder_vesting --argument "(record { 
    ledger_id = principal \"$LEDGER_ID\"; 
})"

echo "Registering founders..."
dfx canister call founder_vesting admin_register_founder "(principal \"$F1\", 35000000000000000)"
dfx canister call founder_vesting admin_register_founder "(principal \"$F2\", 15000000000000000)"

echo "Deploying Subscription Canister..."
dfx deploy subscription_canister --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

echo "Broadcasting Subscription Canister ID to Shards..."
dfx canister call staking_hub admin_broadcast_subscription_manager "(principal \"$SUBSCRIPTION_CANISTER_ID\")"

echo "Deploying KYC Canister..."
dfx deploy kyc_canister --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

echo "Broadcasting KYC Canister ID to Shards..."
dfx canister call staking_hub admin_broadcast_kyc_manager "(principal \"$KYC_CANISTER_ID\")"

# ============================================================================
# Generate Frontend Configuration
# ============================================================================
echo ""
echo "Generating frontend configuration..."

INDEX_ID=$(dfx canister id icrc1_index_canister)

cat > ic.config.json <<EOF
{
    "network": "local",
    "host": "http://localhost:4943",
    "canisters": {
        "icrc1_index": "$INDEX_ID",
        "user_profile": "$USER_PROFILE_ID",
        "learning_engine": "$LEARNING_ID",
        "staking_hub": "$STAKING_HUB_ID",
        "treasury_canister": "$TREASURY_ID",
        "governance_canister": "$GOVERNANCE_ID",
        "ghc_ledger": "$LEDGER_ID",
        "media_assets": "$MEDIA_ASSETS_ID",
        "staging_assets": "$STAGING_ASSETS_ID",
        "internet_identity": "$II_ID",
        "founder_vesting": "$FOUNDER_VESTING_ID",
        "subscription_canister": "$SUBSCRIPTION_CANISTER_ID",
        "kyc_canister": "$KYC_CANISTER_ID"
    },
    "founders": [
        {
            "name": "founder1",
            "principal": "$F1",
            "allocation": "350000000"
        },
        {
            "name": "founder2",
            "principal": "$F2",
            "allocation": "150000000"
        }
    ]
}
EOF

echo "Generated ic.config.json"

# Run update_frontend.sh if it exists
if [ -f "./scripts/update_frontend.sh" ]; then
    echo "Running update_frontend.sh..."
    ./scripts/update_frontend.sh
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "============================================================================"
echo "Deployment Complete!"
echo "============================================================================"
echo ""
echo "Token Distribution (9.5B Total):"
echo "  - Treasury:        4.25B MC"
echo "  - Staking Hub:     4.75B MUC"
echo "  - Founder Vesting: 0.5B MC (time-locked)"
echo "    - Founder 1: 0.35B MC"
echo "    - Founder 2: 0.15B MC"
echo ""
echo "Canister Architecture:"
echo "  ┌─────────────────────────────────────────────────────────────┐"
echo "  │                     CORE SYSTEM                             │"
echo "  ├─────────────────────────────────────────────────────────────┤"
echo "  │  treasury_canister  ◄──► governance_canister                │"
echo "  │         │                        │                          │"
echo "  │         │                        ▼                          │"
echo "  │         │                 learning_engine ◄── staging_assets│"
echo "  │         │                        │                          │"
echo "  │         ▼                        ▼                          │"
echo "  │    ghc_ledger              user_profile                     │"
echo "  │         │                        │                          │"
echo "  │         └───────► staking_hub ◄──┘                          │"
echo "  │                                                             │"
echo "  │  media_assets (permanent storage)                           │"
echo "  │  founder_vesting (time-locked tokens)                       │"
echo "  └─────────────────────────────────────────────────────────────┘"
echo ""
echo "Canister IDs saved to: ic.config.json"
echo ""
