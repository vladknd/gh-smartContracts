# GreenHero Coin (GHC) Smart Contracts

**Last Updated**: January 2026

A comprehensive Internet Computer Protocol (ICP) project for the GreenHero Coin educational platform. This project implements a "Pre-Mint & Allocate" tokenomics model where users earn GHC tokens by completing educational activities.

## 🏗️ Architecture
 
This project consists of 10 core smart contracts (canisters):
 
### Core Financial & Governance Canisters
 
- **ghc_ledger** - ICRC-1 compliant token ledger recording all balances and transfers.
- **staking_hub** - Central Bank holding the "Mined Utility Coins" (4.75B MUC), managing Global Stats, and handling real settlement.
- **treasury_canister** - Manages the Treasury (4.25B MC) with MMCR (Monthly Market Coin Release).
- **governance_canister** - Handles proposals, voting, and board member management.
- **founder_vesting** - Manages time-locked founder allocations (0.5B MC with 10%/year vesting).
 
### Content Governance Canisters
 
- **learning_engine** - Content storage with tree-based structure, quiz management, version history.
- **media_assets** - Permanent storage for videos, images, audio, PDFs.
- **staging_assets** - Temporary storage for content pending governance approval.
 
### User Canisters
 
- **user_profile** (Sharded) - The main entry point. Manages user state, "Micro-Bank" balances, and minting logic.
 
## 🚀 Quick Start
 
### Prerequisites
 
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install) (Internet Computer SDK)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (optional, for frontend integration)
 
### Installation
 
```bash
# Clone the repository
git clone <your-repo-url>
cd gh-smartContracts
 
# Install dfx if not already installed
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```
 
### Deploy Locally
 
```bash
# Start local replica and deploy all canisters
./scripts/deploy.sh
```
 
## 📚 Documentation
 
- [Architecture & Design](ARCHITECTURE.md) - Detailed system architecture and tokenomics
- [Content Governance](CONTENT_GOVERNANCE.md) - Content proposal and upload flow
- [Learning Engine](LEARNING_ENGINE_ARCHITECTURE.md) - Content structure and quiz system
- [Proposal Voting](PROPOSAL_VOTING_FLOW.md) - Governance proposal lifecycle
- [Quick Reference](QUICK_REF.md) - Handy command reference
- [Deployment Guide](DEPLOYMENT.md) - Instructions for deployment
 
## 📦 Project Structure
 
```
gh-smartContracts/
├── src/
│   ├── ghc_ledger/             # ICRC-1 Token Ledger (external)
│   ├── staking_hub/            # Staking & Mining Hub
│   ├── treasury_canister/      # Treasury Management
│   ├── governance_canister/    # Proposals & Voting
│   ├── learning_engine/        # Content Storage & Quizzes
│   ├── media_assets/           # Permanent Media Storage
│   ├── staging_assets/         # Temporary Content Staging
│   ├── user_profile/           # User State & Micro-Bank
│   └── founder_vesting/        # Founder Token Vesting
├── scripts/
│   ├── deploy.sh               # Main deployment script
│   └── tests/                  # Test scripts
├── docs/                       # Documentation
├── dfx.json                    # DFX configuration
├── Cargo.toml                  # Rust workspace config
└── README.md                   # This file
```
 
## 🔗 Canister Dependencies
 
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           CANISTER ARCHITECTURE                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────────────────┐        ┌─────────────────────┐                        │
│   │  treasury_canister  │◄──────►│ governance_canister │                        │
│   │  ─────────────────  │        │  ──────────────────  │                        │
│   │  • Token custody    │        │  • Proposals        │                        │
│   │  • Transfers        │        │  • Voting           │                        │
│   │  • MMCR             │        │  • Board mgmt       │                        │
│   └─────────┬───────────┘        └──────────┬──────────┘                        │
│             │                               │                                    │
│             │                    ┌──────────▼──────────┐                        │
│             │                    │   learning_engine   │◄── staging_assets      │
│             │                    │  ──────────────────  │                        │
│             │                    │  • Content storage  │                        │
│             │                    │  • Quiz management  │                        │
│             │                    │  • Version history  │                        │
│             │                    └──────────┬──────────┘                        │
│             │                               │                                    │
│             │                    ┌──────────▼──────────┐                        │
│             │                    │    user_profile     │                        │
│             │                    │  ──────────────────  │                        │
│             │                    │  • User state       │                        │
│             │                    │  • Quiz submissions │                        │
│             │                    │  • Staked balance   │                        │
│             │                    └──────────┬──────────┘                        │
│             │                               │                                    │
│             │                    ┌──────────▼──────────┐                        │
│             └───────────────────►│     staking_hub     │                        │
│                                  │  ──────────────────  │                        │
│                                  │  • Global stats     │                        │
│                                  │  • VUC management   │                        │
│                                  │  • Minting          │                        │
│                                  └──────────┬──────────┘                        │
│                                             │                                    │
│                                  ┌──────────▼──────────┐                        │
│                                  │      ghc_ledger     │                        │
│                                  │  ──────────────────  │                        │
│                                  │  • Token balances   │                        │
│                                  │  • Transfers        │                        │
│                                  └─────────────────────┘                        │
│                                                                                  │
│   ┌─────────────────────┐        ┌─────────────────────┐                        │
│   │    media_assets     │        │   founder_vesting   │                        │
│   │  ─────────────────  │        │  ──────────────────  │                        │
│   │  • Permanent media  │        │  • Time-locked MC   │                        │
│   │  • Immutable        │        │  • 10%/year vest    │                        │
│   └─────────────────────┘        └─────────────────────┘                        │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```
 
### 🔄 Key Flows
 
**1. Mining Flow ("Virtual Staking")**
1. **Submit**: User submits quiz answers to `user_profile`.
2. **Verify**: `user_profile` calls `learning_engine.verify_quiz(answers)`.
3. **Local Mint**: If correct, `user_profile` updates the user's local balance.
4. **Batch Sync**: Periodically, stats are reported to `staking_hub`.
 
**2. Unstaking (No Penalty)**
Users can withdraw their earned tokens at any time with 100% returned.
 
**3. Content Governance**
1. **Upload**: Creator uploads media to `media_assets`, stages content in `staging_assets`.
2. **Propose**: Creator creates proposal in `governance_canister`.
3. **Vote**: Board members vote on content.
4. **Load**: When approved, `learning_engine` loads from staging.
 
## 🧪 Testing
 
### Run Unit Tests
 
```bash
cargo test
```
 
### Manual Verification
 
```bash
# Check Staking Hub status
dfx canister call staking_hub get_global_stats
 
# Check Treasury status
dfx canister call treasury_canister get_treasury_state

# Check Governance config
dfx canister call governance_canister get_governance_config
```
 
## 🔐 Security
 
- **Stable Structures**: Uses `ic-stable-structures` for scalable and safe memory management.
- **Governance Control**: Treasury and Content decisions are managed via governance proposals.
- **Access Control**: Critical functions are restricted to specific caller canisters.

## 🗺️ Roadmap

- [x] Phase 1: Core Infrastructure (Ledger, Staking Hub)
- [x] Phase 2: Governance (Treasury + Governance canisters)
- [x] Phase 3: Learning Engine & Quiz System
- [x] Phase 4: Content Governance (Media + Staging assets)
- [ ] Phase 5: Frontend Dashboard Integration
- [ ] Phase 6: Mainnet Deployment

## 🌐 Resources

- [Internet Computer Docs](https://internetcomputer.org/docs)
- [Rust CDK Documentation](https://docs.rs/ic-cdk/)
- [ICRC-1 Standard](https://github.com/dfinity/ICRC-1)
