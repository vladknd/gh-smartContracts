#!/bin/bash

# Stop on error
set -e

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
echo "Creating canisters..."
dfx canister create --all

LEDGER_ID=$(dfx canister id ghc_ledger)
STAKING_HUB_ID=$(dfx canister id staking_hub)
OP_GOV_ID=$(dfx canister id operational_governance)
CONTENT_GOV_ID=$(dfx canister id content_governance)
LEARNING_ID=$(dfx canister id learning_engine)
II_ID=$(dfx canister id internet_identity)

echo "Ledger: $LEDGER_ID"
echo "Staking Hub: $STAKING_HUB_ID"
echo "Internet Identity: $II_ID"

# Deploy Ledger
# 8.2 Billion Tokens = 8,200,000,000
# Decimals = 8
# Total Supply in e8s = 8,200,000,000 * 10^8
# Market Partition: 4.1B
#   - Founder 1: 0.3B
#   - Founder 2: 0.2B
#   - Treasury (Op Gov): 3.6B
# Utility Partition: 4.1B (Staking Hub)

# Utility Partition: 4.1B (Staking Hub)

echo "Deploying Internet Identity..."
dfx deploy internet_identity

echo "Deploying Ledger..."

# Helper to format e8s
function to_e8s {
    echo "$100000000"
}

F1_AMT=$(to_e8s 300000000)
F2_AMT=$(to_e8s 200000000)
TREASURY_AMT=$(to_e8s 3600000000)
HUB_AMT=$(to_e8s 4100000000)

# Init Args for ICRC-1 Ledger
# minting_account: null (Fixed Supply)
# initial_balances: List of (Account, Amount)
# transfer_fee: 10_000 e8s (0.0001 GHC)
# token_name: "GreenHero Coin"
# token_symbol: "GHC"
# metadata: []

INIT_ARGS="(variant { Init = record {
     token_symbol = \"GHC\";
     token_name = \"GreenHero Coin\";
     decimals = opt 8;
     minting_account = record { owner = principal \"$DEFAULT\"; subaccount = null; };
     transfer_fee = 10_000;
     metadata = vec {};
     initial_balances = vec {
         record { record { owner = principal \"$F1\"; subaccount = null; }; $F1_AMT };
         record { record { owner = principal \"$F2\"; subaccount = null; }; $F2_AMT };
         record { record { owner = principal \"$OP_GOV_ID\"; subaccount = null; }; $TREASURY_AMT };
         record { record { owner = principal \"$STAKING_HUB_ID\"; subaccount = null; }; $HUB_AMT };
     };
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 2000;
         controller_id = principal \"$DEFAULT\";
     };
 }})"

dfx deploy ghc_ledger --argument "$INIT_ARGS"

# Deploy Staking Hub
echo "Deploying Staking Hub..."
dfx deploy staking_hub --argument "(record { ledger_id = principal \"$LEDGER_ID\" })"

# Deploy Operational Governance
echo "Deploying Operational Governance..."
dfx deploy operational_governance --argument "(record { ledger_id = principal \"$LEDGER_ID\"; staking_hub_id = principal \"$STAKING_HUB_ID\" })"

# Deploy Content Governance
echo "Deploying Content Governance..."
# Assuming similar init args or empty for now if not implemented fully
# But I defined it in did as empty service? No, I didn't update did for content_gov yet.
# Let's check content_governance.did. It was: service : { "get_book_count": ... }
# So no init args needed unless I add them.
dfx deploy content_governance

# Deploy Learning Engine
echo "Deploying Learning Engine..."
dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

# Deploy User Profile
echo "Deploying User Profile..."
dfx deploy user_profile --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\"; learning_content_id = principal \"$LEARNING_ID\" })"

echo "Deployment Complete!"
