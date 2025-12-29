# GreenHero Coin (GHC) Smart Contracts

A comprehensive Internet Computer Protocol (ICP) project for the GreenHero Coin educational platform. This project implements a "Pre-Mint & Allocate" tokenomics model where users earn GHC tokens by completing educational activities.

## 🏗️ Architecture
 
 This project consists of 7 core smart contracts (canisters):
 
 ### Core Financial & Governance Canisters
 
 - **ghc_ledger** - ICRC-1 compliant token ledger recording all balances and transfers.
 - **staking_hub** - Central Bank holding the "Mined Utility Coins" (4.75B MUC), managing Global Stats, and handling real settlement.
 - **operational_governance** - Manages the Treasury (4.25B MC) and handles spending proposals with MMCR (Monthly Market Coin Release).
 - **founder_vesting** - Manages time-locked founder allocations (0.5B MC with 10%/year vesting).
 - **content_governance** - Manages educational content proposals (whitelisting books/NFTs).
 
 ### User & Educational Canisters
 
 - **user_profile** (Sharded) - The main entry point. Manages user state, "Micro-Bank" balances, and minting logic.
 - **learning_engine** - Stateless content provider. Stores quizzes and verifies answers.
 
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
 │   ├── learning_engine/        # Learning Content Provider
 │   └── user_profile/           # User State & Micro-Bank
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
 +-------------------------------------------------------+       +-------------------------------------------------------+
 |                operational_governance                 |       |                  content_governance                   |
 |           (Controls Treasury: 4.25B GHC)              |       |              (Manages Content Updates)                |
 +--------------------------+----------------------------+       +---------------------------+---------------------------+
                            |                                                                |
                            | (queries voting power)                                         | (queries voting power)
                            v                                                                v
             +---------------------------------------------------------------------------------------------+
             |                                         staking_hub                                         |
             |                                 (Central Bank & Global Stats)                               |
             |                                                                                             |
             |  [STORES]                                                                                   |
             |   - Global Stats (Total Staked, Total Unstaked, Total Allocated)                            |
             |   - Minting Allowances                                                                      |
             |                                                                                             |
             |  [READS]                                                                                    |
             |   - get_global_stats()                                                                      |
             +-------------+---------------------------------------------------------------+---------------+
                           |                                                               ^
                           | (transfers real tokens on unstake - 100%)                     | (reports stats / requests allowance)
                           v                                                               |
 +-------------------------------------------------------+       +-------------------------+-----------------------------+
 |                      ghc_ledger                       |       |                      user_profile                     |
 |               (ICRC-1 Token Standard)                 |       |                 (User State & Micro-Bank)             |
 |                                                       |       |                                                       |
 |  [STORES]                                             |       |  [STORES]                                             |
 |   - Real Token Balances                               |       |   - User Profile & Progress                           |
 |                                                       |       |   - Staked Balance (Virtual)                          |
 |                                                       |       |                                                       |
 |  [READS]                                              |       |  [READS]                                              |
 |   - icrc1_balance_of(account)                         |       |   - get_profile(user)                                 |
 +-------------------------------------------------------+       +-------------------------+-----------------------------+
                                                                                           |
                                                                                           | (verifies answers)
                                                                                           v
                                                                 +-------------------------+-----------------------------+
                                                                 |                     learning_engine                   |
                                                                 |               (Stateless Content Provider)            |
                                                                 |                                                       |
                                                                 |  [STORES]                                             |
                                                                 |   - Learning Units (Content, Quizzes)                 |
                                                                 |                                                       |
                                                                 |  [READS]                                              |
                                                                 |   - verify_quiz(answers)                              |
                                                                 +-------------------------------------------------------+
 ```
 
 ### 🔄 Interaction Details
 
 **1. Mining Flow ("Virtual Staking")**
 The interaction between `user_profile` and `learning_engine` is designed for maximum efficiency:
 1.  **Submit**: User submits quiz answers to their `user_profile` shard.
 2.  **Verify**: `user_profile` calls `learning_engine.verify_quiz(answers)`.
 3.  **Local Mint**: If correct, `user_profile` updates the user's local balance immediately.
 4.  **Batch Sync**: Periodically, the shard reports stats to `staking_hub` and requests a new "Minting Allowance".
 
 **2. Unstaking (No Penalty)**
 Users can withdraw their earned tokens at any time:
 *   **Request**: User calls `unstake(amount)` on their shard.
 *   **Transfer**: The Hub transfers **100%** of the amount via the ICRC-1 ledger.
 *   **Receipt**: User receives real GHC tokens in their wallet.

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
 dfx canister call staking_hub get_global_stats
 
 # Check Ledger Balance
 dfx canister call ghc_ledger icrc1_balance_of "(record { owner = principal \"...\" })"
 ```
 
 ## 🔐 Security
 
 - **Stable Structures**: Uses `ic-stable-structures` for scalable and safe memory management.
 - **Governance Control**: Treasury and Content decisions are managed via governance canisters.
 - **Access Control**: Critical functions in `staking_hub` are restricted to specific caller canisters (e.g., `user_profile` shards).

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
