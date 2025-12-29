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
FOUNDER_VESTING_ID=$(dfx canister id founder_vesting)

echo "Ledger: $LEDGER_ID"
echo "Staking Hub: $STAKING_HUB_ID"
echo "Founder Vesting: $FOUNDER_VESTING_ID"
echo "Internet Identity: $II_ID"

# ============================================================================
# Deploy Ledger - NEW TOKENOMICS (9.5B Total)
# ============================================================================
# Total: 9.5 Billion Tokens
# Decimals: 8 (1 GHC = 100,000,000 smallest units, same as ICP/BTC)
# Transfer Fee: 0
#
# Market Coins (4.75B MC):
#   - Founder Vesting: 0.5B (F1: 0.35B + F2: 0.15B, time-locked 10%/year)
#   - Treasury (Op Gov): 4.25B (initial allowance: 0.6B)
#
# Utility Coins (4.75B MUC):
#   - Staking Hub: 4.75B (for mining rewards)
# ============================================================================

echo "Deploying Internet Identity..."
dfx deploy internet_identity

echo "Deploying Ledger..."

# Helper to format amounts with 8 decimals (e8s)
# 1 token = 10^8 smallest units (same as ICP and Bitcoin)
function to_e8s {
    # Multiply by 100,000,000 (10^8)
    echo "$1"00000000
}

# Token allocations (in whole tokens, converted to e8s)
# Total: 9B tokens (0.5B founder vesting will be added when canister is ready)
FOUNDER_VESTING_AMT="50000000000000000"     # 0.5B * 10^8 = 5 * 10^16
TREASURY_AMT="425000000000000000"           # 4.25B * 10^8 = 4.25 * 10^17
HUB_AMT="475000000000000000"                # 4.75B * 10^8 = 4.75 * 10^17

# Init Args for ICRC-1 Ledger
# Decimals: 8 (same as ICP, fits in u64)
# Transfer Fee: 0 (no fees)
# Initial Balances: 9.5B Total
#   - Treasury (4.25B) + Staking Hub (4.75B) + Founder Vesting (0.5B) = 9.5B

INIT_ARGS="(variant { Init = record {
     token_symbol = \"GHC\";
     token_name = \"GreenHero Coin\";
     decimals = opt 8;
     minting_account = record { owner = principal \"$DEFAULT\"; subaccount = null; };
     transfer_fee = 0;
     metadata = vec {};
     initial_balances = vec {
         record { record { owner = principal \"$OP_GOV_ID\"; subaccount = null; }; $TREASURY_AMT : nat };
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

# Deploy Staking Hub
# Note: user_profile_wasm is empty for now - we'll manually register shards
echo "Deploying Staking Hub..."
dfx deploy staking_hub --argument "(record { ledger_id = principal \"$LEDGER_ID\"; learning_content_id = principal \"$LEARNING_ID\"; user_profile_wasm = vec {} })"

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

# Register User Profile as Allowed Minter (Shard) in Staking Hub
echo "Registering User Profile as Allowed Minter..."
USER_PROFILE_ID=$(dfx canister id user_profile)
dfx canister call staking_hub add_allowed_minter "(principal \"$USER_PROFILE_ID\")"

# Deploy Founder Vesting
# Note: This canister manages time-locked founder tokens (0.5B MC total)
# F1: 0.35B MC, F2: 0.15B MC, 10%/year vesting over 10 years
echo "Deploying Founder Vesting..."
dfx deploy founder_vesting --argument "(record { 
    ledger_id = principal \"$LEDGER_ID\"; 
    founder1 = principal \"$F1\"; 
    founder2 = principal \"$F2\" 
})"

echo ""
echo "============================================================================"
echo "Deployment Complete!"
echo "============================================================================"
echo ""
echo "Token Distribution (9.5B Total):"
echo "  - Treasury (operational_governance): 4.25B MC"
echo "  - Staking Hub: 4.75B MUC"
echo "  - Founder Vesting: 0.5B MC (time-locked)"
echo "    - Founder 1 ($F1): 0.35B MC"
echo "    - Founder 2 ($F2): 0.15B MC"
echo ""
echo "Treasury Status:"
echo "  - Initial Balance: 4.25B MC"
echo "  - Initial Allowance: 0.6B MC (spendable)"
echo "  - MMCR: 15.2M MC/month for 240 months"
echo ""
echo "Founder Vesting:"
echo "  - Vesting Schedule: 10%/year over 10 years"
echo "  - Founders can claim via: dfx canister call founder_vesting claim_vested"
echo ""

