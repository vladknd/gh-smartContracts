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
| **Learning Engine** | Manages quizzes and learning progress. | Submit quizzes, check completion status. |
| **Staking Hub** | Manages user rewards and staking. | Claim rewards, stake, unstake, view balances. |
| **Operational Governance** | Manages DAO proposals and voting. | Create proposals, vote, execute proposals. |
| **GHC Ledger** | The ICRC-1 Token Ledger. | Transfer tokens, check wallet balances. |

## 3. Deployed Canister IDs (Local Network)

Use these IDs when configuring your frontend agent.

| Canister Name | Canister ID |
|---|---|
| **Learning Engine** | `uzt4z-lp777-77774-qaabq-cai` |
| **Staking Hub** | `ulvla-h7777-77774-qaacq-cai` |
| **Operational Governance** | `umunu-kh777-77774-qaaca-cai` |
| **GHC Ledger** | `u6s2n-gx777-77774-qaaba-cai` |
| **Content Governance** | `uxrrr-q7777-77774-qaaaq-cai` |
| **Internet Identity** | `ufxgi-4p777-77774-qaadq-cai` |

---

## 4. Canister API Reference

### A. Learning Engine (`learning_engine`)

**Purpose**: Handles the "Learn to Earn" mechanics.

#### Methods

1.  **`submit_quiz(quiz_id: nat64, answers: vec text) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Submits answers for a specific quiz.
    *   **Returns**: `Ok(reward_amount)` if successful (e.g., 500_000_000 for 5 GHC), or `Err(message)`.
    *   **Usage**: Call when user clicks "Submit" on a quiz.

3.  **`get_learning_units_metadata() -> vec LearningUnitMetadata`**
    *   **Type**: Query
    *   **Description**: Returns a list of all available learning units with basic metadata (ID, Title, Chapter).
    *   **Returns**: `vec { record { unit_id; unit_title; chapter_id; chapter_title } }`.
    *   **Usage**: Call this on app load to build the curriculum menu/navigation.

4.  **`get_user_daily_status(user: principal) -> DailyStatus`**
    *   **Type**: Query
    *   **Description**: Returns the user's daily progress.
    *   **Returns**: `record { quizzes_taken: nat8; daily_limit: nat8; tokens_earned: nat64 }`.
    *   **Usage**: Display "Quizzes Today: X/5" and "Earned Today: Y GHC".

5.  **`is_quiz_completed(user: principal, quiz_id: nat64) -> bool`**
    *   **Type**: Query
    *   **Description**: Checks if a user has already completed a quiz.
    *   **Returns**: `true` if completed, `false` otherwise.
    *   **Usage**: Use to disable the "Submit" button or show a "Completed" badge.

---

### B. Staking Hub (`staking_hub`)

**Purpose**: Central hub for rewards and staking.

#### Methods

1.  **`get_user_stats(user: principal) -> (nat64, nat64)`**
    *   **Type**: Query
    *   **Description**: Gets the user's current virtual balance and any pending rewards (from interest) that haven't been credited yet.
    *   **Returns**: `(current_balance, pending_interest)`.
    *   **Usage**: Display "Total Staked" as `current_balance + pending_interest`.

2.  **`unstake(amount: nat64) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Unstakes a specific amount. **Applies a 10% penalty.**
    *   **Returns**: `Ok(amount_received)` (90% of requested) or `Err(message)`.
    *   **Usage**: Call when user wants to withdraw funds. **Warn them about the 10% fee!**

3.  **`get_voting_power(user: principal) -> nat64`**
    *   **Type**: Query
    *   **Description**: Gets the user's voting power (currently 1:1 with staked balance + pending interest).
    *   **Usage**: Display in the Governance section.

4.  **`get_global_stats() -> GlobalStats`**
    *   **Type**: Query
    *   **Description**: Returns total staked amount and current interest pool size.
    *   **Usage**: Display "Total Value Locked" or "Next Dividend Pool".

---

### C. Operational Governance (`operational_governance`)

**Purpose**: DAO functionality for managing the ecosystem.

#### Methods

1.  **`create_proposal(recipient: principal, amount: nat64, description: text) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Creates a new proposal to transfer funds from the Governance canister to a recipient.
    *   **Returns**: `Ok(proposal_id)` or `Err(message)`.
    *   **Usage**: Call from a "Create Proposal" form.

2.  **`vote(proposal_id: nat64, approve: bool) -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: Casts a vote (Yes/No) on a proposal.
    *   **Returns**: `Ok(null)` or `Err(message)`.
    *   **Usage**: Call when user clicks "Vote Yes" or "Vote No".

3.  **`execute_proposal(proposal_id: nat64) -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: Executes a passed proposal (transfers funds).
    *   **Returns**: `Ok(null)` or `Err(message)`.
    *   **Usage**: Call via an "Execute" button (visible if `votes_yes > votes_no`).

4.  **`get_proposal(id: nat64) -> opt Proposal`**
    *   **Type**: Query
    *   **Description**: Fetches details of a specific proposal.
    *   **Returns**: `Some(Proposal)` or `None`.
    *   **Usage**: Display proposal details (Description, Votes, Status).

---

### D. GHC Ledger (`ghc_ledger`)

**Purpose**: The ICRC-1 Token Standard implementation.

#### Methods

1.  **`icrc1_balance_of(account: Account) -> nat64`**
    *   **Type**: Query
    *   **Description**: Checks the liquid (wallet) balance of a user.
    *   **Input**: `record { owner = principal; subaccount = null }`
    *   **Usage**: Display "Wallet Balance".

2.  **`icrc1_transfer(args: TransferArg) -> Result<nat64, TransferError>`**
    *   **Type**: Update
    *   **Description**: Transfers tokens to another user.
    *   **Usage**: Standard token transfer feature.

---

## 5. Integration Example (React)

Here is a simplified example of how to connect and call a method.

```javascript
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { idlFactory as learningIdl } from "./declarations/learning_engine";

// 1. Authenticate User
const authClient = await AuthClient.create();
await authClient.login({
  identityProvider: "http://ufxgi-4p777-77774-qaadq-cai.localhost:4943/",
  onSuccess: () => {
    const identity = authClient.getIdentity();
    const principal = identity.getPrincipal();
    console.log("Logged in as:", principal.toText());
    
    // 2. Create Actor
    const agent = new HttpAgent({ identity });
    // Only for local dev:
    await agent.fetchRootKey(); 

    const learningActor = Actor.createActor(learningIdl, {
      agent,
      canisterId: "uzt4z-lp777-77774-qaabq-cai", // Use actual ID
    });

    // 3. Call Method
    submitQuiz(learningActor);
  },
});

async function submitQuiz(actor) {
  try {
    const quizId = "1.0"; // String ID
    const answers = [0]; // Vector of u8 indices
    const result = await actor.submit_quiz(quizId, answers);
    
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

## 6. Common Error Handling

| Error Message | Cause | Solution |
|---|---|---|
| `Quiz already completed` | User tries to submit the same quiz twice. | Disable the submit button if `is_quiz_completed` returns true. |
| `Insufficient balance` | User tries to unstake more than they have. | Validate input amount against `get_user_stats`. |
| `No interest to distribute` | Admin calls `distribute_interest` with empty pool. | Check `get_global_stats` first. |

