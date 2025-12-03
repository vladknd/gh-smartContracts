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
    *   **Usage**: Call when user clicks "Submit" on a quiz.

2.  **`unstake(amount: nat64) -> Result<nat64, text>`**
    *   **Type**: Update
    *   **Description**: Unstakes a specific amount from the user's local balance. **Applies a 10% penalty.**
    *   **Returns**: `Ok(amount_received)` (90% of requested) or `Err(message)`.
    *   **Usage**: Call when user wants to withdraw funds to their main wallet.

3.  **`get_profile(user: principal) -> opt UserProfile`**
    *   **Type**: Query
    *   **Description**: Returns the user's profile and **Staked Balance**.
    *   **Returns**: `record { email; name; staked_balance; ... }`.
    *   **Usage**: Display "Staked Balance" from this record.

4.  **`get_user_daily_status(user: principal) -> UserDailyStats`**
    *   **Type**: Query
    *   **Description**: Returns the user's daily progress.
    *   **Returns**: `record { quizzes_taken; tokens_earned; ... }`.
    *   **Usage**: Display "Quizzes Today: X/5".

5.  **`is_quiz_completed(user: principal, unit_id: text) -> bool`**
    *   **Type**: Query
    *   **Description**: Checks if a user has already completed a quiz.
    *   **Usage**: Disable "Submit" button if true.

6.  **`debug_force_sync() -> Result<null, text>`**
    *   **Type**: Update
    *   **Description**: **DEV ONLY**. Forces the shard to sync pending stats with the Staking Hub immediately.
    *   **Usage**: Call this after `submit_quiz` if you need to verify Global Stats updates immediately during testing.

---

### B. Learning Engine (`learning_engine`)

**Purpose**: Stores educational content.

#### Methods

1.  **`get_learning_units_metadata() -> vec LearningUnitMetadata`**
    *   **Type**: Query
    *   **Description**: Returns a list of all available learning units.
    *   **Usage**: Build the curriculum menu.

---

### C. Staking Hub (`staking_hub`)

**Purpose**: Central Treasury and Global Stats.

**Note on Eventual Consistency**: Global stats (`total_staked`, `total_mined`) are updated in **batches** by the User Profile shards. You may not see immediate changes after a single quiz submission. This is by design for scalability.

#### Methods

1.  **`get_global_stats() -> GlobalStats`**
    *   **Type**: Query
    *   **Description**: Returns total staked amount, total mined, and interest pool.
    *   **Usage**: Display "Total Value Locked" or "Global Stats".

---

### D. GHC Ledger (`ghc_ledger`)

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

    // 3. Submit Quiz
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
