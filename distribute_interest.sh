#!/bin/bash

# Script to manually trigger interest distribution
# In production, this would run monthly as a scheduled job

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   GHC Interest Distribution - Manual Trigger"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Get staking_hub canister ID
STAKING_HUB=$(jq -r '.canisters.staking_hub.local' ic.config.json)

if [ -z "$STAKING_HUB" ] || [ "$STAKING_HUB" = "null" ]; then
    echo "❌ Error: Could not find staking_hub canister ID in ic.config.json"
    exit 1
fi

echo "📊 Staking Hub Canister: $STAKING_HUB"
echo ""

# Check current global stats BEFORE distribution
echo "📈 BEFORE Distribution:"
echo "─────────────────────────────────────────────────────"
STATS_BEFORE=$(dfx canister call $STAKING_HUB get_global_stats)
echo "$STATS_BEFORE"
echo ""

# Parse interest pool (basic extraction - you may need to adjust)
INTEREST_POOL=$(echo "$STATS_BEFORE" | grep -o 'interest_pool = [0-9_]*' | sed 's/interest_pool = //' | tr -d '_')

if [ -z "$INTEREST_POOL" ] || [ "$INTEREST_POOL" = "0" ]; then
    echo "⚠️  Interest pool is empty (0 tokens)"
    echo "💡 No penalties have been collected yet."
    echo ""
    echo "To test this function:"
    echo "  1. Stake some tokens by completing quizzes"
    echo "  2. Unstake a portion (10% penalty goes to interest_pool)"
    echo "  3. Run this script again"
    echo ""
    exit 0
fi

echo "💰 Interest Pool: $INTEREST_POOL (will be distributed)"
echo ""

# Call distribute_interest()
echo "🔄 Calling distribute_interest()..."
RESULT=$(dfx canister call $STAKING_HUB distribute_interest 2>&1)

# Check if successful
if echo "$RESULT" | grep -q "Ok"; then
    echo "✅ SUCCESS!"
    echo "$RESULT"
else
    echo "❌ FAILED!"
    echo "$RESULT"
    exit 1
fi

echo ""
echo "─────────────────────────────────────────────────────"

# Check global stats AFTER distribution
echo "📉 AFTER Distribution:"
echo "─────────────────────────────────────────────────────"
STATS_AFTER=$(dfx canister call $STAKING_HUB get_global_stats)
echo "$STATS_AFTER"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✨ Distribution Complete!"
echo ""
echo "📝 What happened:"
echo "  • Interest pool funds moved to cumulative_reward_index"
echo "  • interest_pool is now 0"
echo "  • All stakers can now claim proportional rewards"
echo ""
echo "🔍 Next steps:"
echo "  • User profiles will auto-sync within 5 seconds"
echo "  • Users can call claim_rewards() to move interest → staked_balance"
echo "  • Check individual profiles with: dfx canister call user_profile get_profile '(principal \"...\")'"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
