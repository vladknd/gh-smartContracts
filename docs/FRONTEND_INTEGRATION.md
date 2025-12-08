# GHC Dapp Frontend Integration Guide

This guide provides a reference for frontend developers integrating with the GHC Dapp canisters on the Internet Computer.

## 1. Prerequisites

To interact with the canisters, your frontend project should have the following dependencies installed:

```bash
npm install @dfinity/agent @dfinity/candid @dfinity/principal @dfinity/auth-client
```

## 2. Canister Overview

| Canister Name | Description | Key Features |
|---|---|---|
| **User Profile (Sharded)** | **Primary User Interface**. Manages user state, quizzes, and balances. | Submit quizzes, check progress, unstake tokens. |
| **Learning Engine** | **Stateless Content Provider**. Stores quizzes and verifies answers. | Fetch learning units. |
| **Staking Hub** | **Central Bank**. Manages global stats and treasury. | View global stats. (Users rarely interact directly). |
| **Operational Governance** | Manages DAO proposals and voting. | Create proposals, vote. |
| **GHC Ledger** | The ICRC-1 Token Ledger. | Transfer tokens, check wallet balances. |

## 3. Deployed Canister IDs (Local Network)

*Note: IDs may change upon redeployment. Check `dfx canister id <name>`.*

| Canister Name | Canister ID |
|---|---|
| **User Profile** | `ufxgi-4p777-77774-qaadq-cai` |
| **Learning Engine** | `umunu-kh777-77774-qaaca-cai` |
| **Staking Hub** | `ucwa4-rx777-77774-qaada-cai` |
| **Operational Governance** | `ulvla-h7777-77774-qaacq-cai` |
| **GHC Ledger** | `u6s2n-gx777-77774-qaaba-cai` |
| **ICRC-1 Index** | `vpyes-67777-77774-qaaeq-cai` |
| **Content Governance** | `uxrrr-q7777-77774-qaaaq-cai` |
| **Internet Identity** | `uzt4z-lp777-77774-qaabq-cai` |

---

## 4. Canister API Reference

### A. User Profile (`user_profile`)

**Purpose**: The main entry point for user interactions. Handles "Learn to Earn" and "Micro-Bank" balances.

#### Methods

1.  **`submit_quiz(unit_id: text, answers: blob) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Submits answers for a specific quiz unit.
    *   **Returns**: `Ok(reward_amount)` (e.g., 100_000_000 for 1 GHC), or `Err(message)`.
    *   **Usage**: Call when user clicks "Submit" on a quiz. **Requires Registration.**

2.  **`unstake(amount: nat64) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Unstakes a specific amount from the user's local balance. **Applies a 10% penalty.**
    *   **Returns**: `Ok(amount_received)` (90% of requested) or `Err(message)`.
    *   **Usage**: Call when user wants to withdraw funds to their main wallet. **Requires Registration.**

3.  **`claim_rewards() -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Claims pending interest rewards and adds them to the staked balance.
    *   **Returns**: `Ok(amount_claimed)` or `Err(message)`.
    *   **Usage**: Call when user clicks "Claim Rewards".

4.  **`get_profile(user: principal) -> opt UserProfile`**
    *   **Type**: Query
    *   **Description**: Returns the user's profile and **Staked Balance**.
<<<<<<< HEAD
    *   **Returns**: `record { email; name; staked_balance; ... }`.
    *   **Usage**: Check if user exists (returns `null` if not registered). Display "Staked Balance".
=======
    *   **Returns**: `record { email; name; staked_balance; unclaimed_interest; ... }`.
    *   **Usage**: Check if user exists (returns `null` if not registered). Display "Staked Balance" and "Unclaimed Interest".
>>>>>>> afc629286f236b3624dded6017d55bcd21b9fd31

4.  **`register_user(profile: UserProfileUpdate) -> Result<null, text>`**
    *   **Type**: Update
        *   **Description**: Registers a new user. Fails if user already exists. Initial balance is 0.
    *   **Input**: `record { email; name; education; gender }`.

5.  **`update_profile(profile: UserProfileUpdate) -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: Updates personal info for an existing user. Fails if user not registered. **Does not affect balance.**
    *   **Input**: `record { email; name; education; gender }`.

<<<<<<< HEAD
6.  **`get_user_daily_status(user: principal) -> UserDailyStats`**
=======
7.  **`get_user_daily_status(user: principal) -> UserDailyStats`**
>>>>>>> afc629286f236b3624dded6017d55bcd21b9fd31
    *   **Type**: Query
    *   **Description**: Returns the user's daily progress.
    *   **Returns**: `record { quizzes_taken; tokens_earned; ... }`.
    *   **Usage**: Display "Quizzes Today: X/5".

<<<<<<< HEAD
7.  **`is_quiz_completed(user: principal, unit_id: text) -> bool`**
=======
8.  **`is_quiz_completed(user: principal, unit_id: text) -> bool`**
>>>>>>> afc629286f236b3624dded6017d55bcd21b9fd31
    *   **Type**: Query
    *   **Description**: Checks if a user has already completed a quiz.
    *   **Usage**: Disable "Submit" button if true.

<<<<<<< HEAD
8.  **`debug_force_sync() -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: **DEV ONLY**. Forces the shard to sync pending stats with the Staking Hub immediately.
    *   **Usage**: Call this after `submit_quiz` if you need to verify Global Stats updates immediately during testing.
    
=======
>>>>>>> afc629286f236b3624dded6017d55bcd21b9fd31
9.  **`get_user_transactions(user: principal) -> vec TransactionRecord`**
    *   **Type**: Query
    *   **Description**: Returns a history of internal game transactions (Quiz Rewards, Unstaking).
    *   **Returns**: `vec { record { timestamp; tx_type; amount } }`.
    *   **Usage**: Display "Game History" (e.g., "Earned 10 GHC", "Unstaked 5 GHC").

10. **`debug_force_sync() -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: **DEV ONLY**. Forces the shard to sync pending stats with the Staking Hub immediately.

---

### B. Learning Engine (`learning_engine`)

**Purpose**: Stores educational content.

#### Methods

1.  **`get_learning_units_metadata() -> vec LearningUnitMetadata`**
    *   **Type**: Query
    *   **Description**: Returns a list of all available learning units (titles, IDs, chapters).
    *   **Usage**: Build the curriculum menu.

2.  **`get_learning_unit(unit_id: text) -> Result<PublicLearningUnit, text>`**
    *   **Type**: Query
    *   **Description**: Returns the full content of a learning unit, including the quiz.
    *   **Returns**: `Ok(record { title; content; paraphrase; quiz: vec { question; options }; ... })`.
    *   **Usage**: Display the learning page and quiz form.

---

### C. Staking Hub (`staking_hub`)

**Purpose**: Manages global stats, interest rates, and the treasury.

#### Methods

1.  **`get_global_stats() -> GlobalStats`**
    *   **Type**: Query
    *   **Description**: Returns global platform statistics.
    *   **Returns**: `record { total_staked; interest_pool; total_rewards_distributed; ... }`.
    *   **Usage**: Display "Total Value Locked" or "Global Stats".

---

### D. Operational Governance (`operational_governance`)

**Purpose**: Manages DAO proposals and voting.

#### Methods

1.  **`create_proposal(recipient: principal, amount: nat64, description: text) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Creates a new proposal to spend funds from the treasury.
    *   **Returns**: `Ok(proposal_id)`.

2.  **`vote(proposal_id: nat64, approve: bool) -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: Votes on an active proposal.

3.  **`get_proposal(id: nat64) -> opt Proposal`**
    *   **Type**: Query
    *   **Description**: Returns details of a specific proposal.
    *   **Returns**: `record { proposer; recipient; amount; votes_yes; votes_no; executed; ... }`.

---

### E. GHC Ledger (`ghc_ledger`)

**Purpose**: The ICRC-1 Token Standard implementation.

#### Methods

1.  **`icrc1_balance_of(account: Account) -> nat64`**
    *   **Type**: Query
    *   **Description**: Checks the liquid (wallet) balance of a user.
    *   **Usage**: Display "Wallet Balance".

2.  **`icrc1_transfer(args: TransferArg) -> Result<nat64, TransferError>`**
    *   **Type**: Update
    *   **Description**: Transfers tokens to another user.

---

### F. ICRC-1 Index Canister (`icrc1_index_canister`)

**Purpose**: Indexes transactions from the GHC Ledger, enabling efficient queries for account-specific transaction history.

**Canister ID**: `vpyes-67777-77774-qaaeq-cai`

#### Methods

1.  **`get_account_transactions(args: GetAccountTransactionsArgs) -> GetTransactionsResult`**
    *   **Type**: Query
    *   **Description**: Returns transactions for a specific account (sends, receives, mints, burns).
    *   **Input**: 
        ```candid
        record {
            account: record { owner: principal; subaccount: opt blob };
            start: opt nat;  // Transaction ID to start from (for pagination)
            max_results: nat
        }
        ```
    *   **Returns**: 
        ```candid
        variant {
            Ok: record {
                balance: nat;
                transactions: vec record {
                    id: nat;
                    transaction: record {
                        kind: text;  // "mint", "transfer", "burn", "approve"
                        timestamp: nat64;
                        burn: opt Burn;
                        mint: opt Mint;
                        transfer: opt Transfer;
                        approve: opt Approve;
                    }
                };
                oldest_tx_id: opt nat;
            };
            Err: record { message: text }
        }
        ```
    *   **Usage**: Display wallet transaction history (sends/receives).

2.  **`get_blocks(args: GetBlocksRequest) -> GetBlocksResponse`**
    *   **Type**: Query
    *   **Description**: Returns raw blocks from the ledger.
    *   **Input**: `record { start: nat; length: nat }`
    *   **Returns**: `record { chain_length: nat64; blocks: vec Block }`

3.  **`status() -> Status`**
    *   **Type**: Query
    *   **Description**: Returns sync status of the indexer.
    *   **Returns**: `record { num_blocks_synced: nat }`
    *   **Usage**: Verify indexer is synced before querying.

4.  **`ledger_id() -> principal`**
    *   **Type**: Query
    *   **Description**: Returns the ledger canister ID this indexer is tracking.
    *   **Returns**: `principal "u6s2n-gx777-77774-qaaba-cai"` (the GHC ledger)

5.  **`icrc1_balance_of(account: Account) -> nat`**
    *   **Type**: Query
    *   **Description**: Returns the balance for an account (mirrors the ledger).

---

### G. Transaction History (Summary)

1.  **Internal Game History (Minting/Unstaking)**
    *   Use `user_profile.get_user_transactions(user)` to get the history of rewards earned and tokens unstaked.
    *   This data is stored directly in the `user_profile` canister.

2.  **Ledger History (Sending/Receiving) - Using ICRC-1 Index**
    *   Use `icrc1_index_canister.get_account_transactions(...)` to query wallet transaction history.
    *   **DID File**: Located at `src/icrc1_index/index-ng.did`
    *   **Generate TypeScript declarations**:
        ```bash
        dfx generate icrc1_index_canister
        ```

#### Example: Fetching Transaction History (JavaScript)

```javascript
import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
// Import generated IDL
import { idlFactory as indexIdl } from "./declarations/icrc1_index_canister";

const ICRC1_INDEX_CANISTER_ID = "vpyes-67777-77774-qaaeq-cai";

async function getTransactionHistory(identity, maxResults = 20) {
    const agent = new HttpAgent({ identity });
    await agent.fetchRootKey(); // Only for local dev

    const indexActor = Actor.createActor(indexIdl, {
        agent,
        canisterId: ICRC1_INDEX_CANISTER_ID,
    });

    const result = await indexActor.get_account_transactions({
        account: {
            owner: identity.getPrincipal(),
            subaccount: [] // null subaccount
        },
        start: [], // Start from most recent
        max_results: BigInt(maxResults)
    });

    if ('Ok' in result) {
        const { balance, transactions, oldest_tx_id } = result.Ok;
        console.log("Current Balance:", balance.toString());
        
        transactions.forEach(tx => {
            const { id, transaction } = tx;
            console.log(`TX #${id}: ${transaction.kind} at ${new Date(Number(transaction.timestamp) / 1_000_000)}`);
            
            if (transaction.transfer && transaction.transfer.length > 0) {
                const t = transaction.transfer[0];
                console.log(`  From: ${t.from.owner.toText()}`);
                console.log(`  To: ${t.to.owner.toText()}`);
                console.log(`  Amount: ${t.amount.toString()}`);
            }
            if (transaction.mint && transaction.mint.length > 0) {
                const m = transaction.mint[0];
                console.log(`  To: ${m.to.owner.toText()}`);
                console.log(`  Amount: ${m.amount.toString()}`);
            }
        });

        return { balance, transactions, oldest_tx_id };
    } else {
        console.error("Error fetching transactions:", result.Err.message);
        throw new Error(result.Err.message);
    }
}
```

---

## 5. Integration Example (React)

```javascript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
// Import IDL from your generated declarations
import { idlFactory as userProfileIdl } from "./declarations/user_profile";

// 1. Authenticate User
const authClient = await AuthClient.create();
await authClient.login({
  identityProvider: "http://uzt4z-lp777-77774-qaabq-cai.localhost:4943/",
  onSuccess: () => {
    const identity = authClient.getIdentity();
    
    // 2. Create Actor for User Profile
    const agent = new HttpAgent({ identity });
    await agent.fetchRootKey(); // Only for local dev

    const userActor = Actor.createActor(userProfileIdl, {
      agent,
      canisterId: "ufxgi-4p777-77774-qaadq-cai",
    });

    // 3. Check Registration & Register if needed
    const profile = await userActor.get_profile(identity.getPrincipal());
    
    if (profile.length === 0) { // Option type returns [] or [value]
        console.log("User not registered. Registering...");
        await userActor.register_user({
            email: "user@example.com",
            name: "New User",
            education: "Bachelor",
            gender: "Other"
        });
    }

    // 4. Submit Quiz
    submitQuiz(userActor);
  },
});

async function submitQuiz(actor) {
  try {
    const unitId = "unit_1";
    const answers = [0]; // Vector of u8 indices (Blob/Vec<u8>)
    // Note: 'answers' might need to be passed as a Uint8Array or number[] depending on codegen
    const result = await actor.submit_quiz(unitId, answers);
    
    if ('Ok' in result) {
      console.log("Success! Reward:", result.Ok);
    } else {
      console.error("Error:", result.Err);
    }
  } catch (e) {
    console.error("Call failed:", e);
  }
}
```
