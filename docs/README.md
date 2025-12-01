# GreenHero Coin (GHC) Smart Contracts

A comprehensive Internet Computer Protocol (ICP) project for the GreenHero Coin educational platform. This project implements a "Pre-Mint & Allocate" tokenomics model where users earn GHC tokens by completing educational activities.

## 🏗️ Architecture

This project consists of 5 core smart contracts (canisters):

### Core Financial & Governance Canisters

- **ghc_ledger** - ICRC-1 compliant token ledger recording all balances and transfers.
- **staking_hub** - Central custodian holding the "Mined Utility Partition" (4.1B tokens), managing user subaccounts, and handling staking/mining logic.
- **operational_governance** - Manages the Treasury (3.6B tokens) and handles spending proposals.
- **content_governance** - Manages educational content proposals (whitelisting books/NFTs).

### Educational Canisters

- **learning_engine** - Manages quizzes, user progress, and verifies learning activities to trigger rewards.

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
./deploy.sh
```

## 📚 Documentation

- [Architecture & Design](ARCHITECTURE.md) - Detailed system architecture and tokenomics
- [Deployment Guide](DEPLOYMENT.md) - Instructions for deployment
- [Verification Guide](VERIFICATION.md) - How to verify the system state
- [Frontend Integration](FRONTEND_INTEGRATION.md) - Guide for connecting a frontend
- [Quick Reference](QUICK_REF.md) - Handy command reference

## 🛠️ Management & Verification

### Run Comprehensive Tests

The project includes a comprehensive test script that verifies the entire flow, including deployment, initialization, and interaction between canisters.

```bash
./comprehensive_test.sh
```

### Test Specific Flows

```bash
# Test the quiz and reward flow
./test_quiz_flow.sh

# Test general system flow
./test_flow.sh
```

### Load Initial Data

```bash
# Load learning materials into the Learning Engine
./load_data.sh
```

## 📦 Project Structure

```
gh-smartContracts/
├── src/
│   ├── ghc_ledger/             # ICRC-1 Token Ledger
│   ├── staking_hub/            # Staking & Mining Hub
│   ├── operational_governance/ # Treasury & Ops Governance
│   ├── content_governance/     # Content Governance
│   └── learning_engine/        # Learning & Quiz Engine
├── deploy.sh                   # Main deployment script
├── comprehensive_test.sh       # Full system verification
├── test_quiz_flow.sh           # Quiz flow tests
├── load_data.sh                # Data loading script
├── dfx.json                    # DFX configuration
├── Cargo.toml                  # Rust workspace config
├── ARCHITECTURE.md             # Architecture documentation
└── README.md                   # This file
```

## 🔗 Canister Dependencies

```
┌──────────────────────────────┐       ┌──────────────────────────────┐
│    operational_governance    │       │      content_governance      │
│ (Controls Treasury: 3.6B GHC)│       │  (Stores Content Proposals)  │
└──────────────┬───────────────┘       └──────────────┬───────────────┘
               │ (queries voting power)               │ (queries voting power)
               ▼                                      ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                        staking_hub                          │
    │              (Holds 4.1B Utility Tokens)                    │
    │  [ Internal Ledger: User Virtual Balances + Voting Power ]  │
    └──────────────┬──────────────────────────────▲───────────────┘
                   │                              │ (adds rewards /
                   ▼                              │  virtual stake)
    ┌──────────────────────────────┐       ┌──────┴───────────────────────┐
    │          ghc_ledger          │       │       learning_engine        │
    │    (Total Supply: 8.2B)      │       │    (Stores User Progress)    │
    └──────────────────────────────┘       └──────────────────────────────┘
```

### 🔄 Interaction Details

**1. Mining Flow ("Virtual Staking")**
The interaction between `learning_engine` and `staking_hub` is designed for maximum efficiency and user experience:
1.  **Verify**: User submits quiz answers to `learning_engine`.
    *   **Passing Threshold**: Must answer at least **60%** correctly (e.g., 3/5).
    *   **Daily Limit**: Users can take up to **5 quizzes per day**.
2.  **Auto-Stake**: If correct, `learning_engine` calls `staking_hub.stake_rewards(user, amount)`.
3.  **Instant Utility**: The user's "Virtual Balance" in the Staking Hub is immediately updated. This balance **counts as Voting Power** instantly.
4.  **No Gas Fees**: There is no interaction with the `ghc_ledger` at this stage. The tokens remain in the Staking Hub's main account on the Ledger, but are logically attributed to the user internally.

**2. Interest Payouts (Monthly Dividends)**
The system incentivizes long-term holding through a "Lazy Distribution" model:
*   **Collection**: When a user unstakes, a **10% penalty** is deducted and added to a `Pending Interest Pool`.
*   **Distribution Rounds**: Periodically (e.g., monthly), an admin triggers a distribution. This moves the accumulated penalties into a `Cumulative Reward Index`.
*   **Lazy Payout**: Users do not see their balance change instantly. Instead, the next time they interact (mine, vote, or unstake), their balance is automatically updated based on the `Reward Index` growth since their last interaction. This allows O(1) scalability for millions of users.

## 🧪 Testing

### Run Unit Tests

```bash
cargo test
```

### Run Integration Tests

Use the provided shell scripts to run integration tests against a local replica:

```bash
# Verify local deployment and flows
./comprehensive_test.sh
```

### Manual Verification

You can interact with the canisters using `dfx`:

```bash
# Check Staking Hub status
dfx canister call staking_hub get_stats

# Check Ledger Balance
dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"...\" })"

# Check Learning Engine Stats
dfx canister call learning_engine get_stats
```

## 🔐 Security

- **Stable Structures**: Uses `ic-stable-structures` for scalable and safe memory management.
- **Governance Control**: Treasury and Content decisions are managed via governance canisters.
- **Access Control**: Critical functions in `staking_hub` are restricted to specific caller canisters (e.g., `learning_engine`).

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test locally with `./comprehensive_test.sh`
5. Submit a pull request

## 📄 License

[Your License Here]

## 🆘 Support

For issues or questions:

1. Check [ARCHITECTURE.md](ARCHITECTURE.md) and [DEPLOYMENT.md](DEPLOYMENT.md)
2. Run `./comprehensive_test.sh` to check system health
3. Open an issue on GitHub

## 🗺️ Roadmap

- [x] Phase 1: Core Infrastructure (Ledger, Staking Hub, Governance)
- [x] Phase 2: Learning Engine & Quiz Logic
- [ ] Phase 3: Advanced Content Governance Features
- [ ] Phase 4: Frontend Dashboard Integration
- [ ] Phase 5: Mainnet Deployment

## 🌐 Resources

- [Internet Computer Docs](https://internetcomputer.org/docs)
- [Rust CDK Documentation](https://docs.rs/ic-cdk/)
- [ICRC-1 Standard](https://github.com/dfinity/ICRC-1)
