# KYC & AI Verification Architecture

This document describes the design and implementation of the KYC (Know Your Customer) and AI verification system in the GreenHero (GHC) ecosystem.

## Overview

The KYC system follows the **"Controller-Slave" pattern**, abstracting verification logic into a mutable canister while keeping critical user state in immutable shards.

### Components

1.  **KYC Canister (Mutable/Upgradable)**: 
    *   The "Brain" for verification logic.
    *   Handles integration with external AI verification providers (via HTTPS Outcalls).
    *   Maintains local records of verification attempts and metadata.
    *   Authorized to update the `verification_tier` on user shards.
2.  **User Profile Shards (Immutable)**: 
    *   Hold the `verification_tier` for each user.
    *   Only trust calls from the authorized `kyc_manager_id`.
3.  **Staking Hub (The Orchestrator)**:
    *   Central registry for the trusted `kyc_manager_id`.
    *   Broadcasts updates to all shards when the KYC canister is replaced or upgraded.

---

## Verification Tiers

The system supports three progressive tiers:
*   `None`: Initial status for all new users.
*   `Human`: Verified as a real human (e.g., via simple proof-of-personhood or bot detection).
*   `KYC`: Full legal verification (Passport, AML checks, etc.).

---

## The Verification Flow

1.  **Submission**: User provides verification data to the KYC canister via `submit_kyc_data(blob)`.
2.  **AI Analysis**: The KYC canister initiates an AI check (currently mocked, ready for HTTPS Outcalls to providers like Rekognition or Sumsub).
3.  **Approval**: Once the AI (or manual admin) approves the identity, the KYC canister:
    *   Queries `staking_hub.get_user_shard(principal)` to find the user's location.
    *   Calls `shard.internal_set_kyc_status(principal, KYC_TIER)`.
4.  **Enforcement**: The Shard verifies that the caller matches its stored `kyc_manager_id` before updating the user profile.

---

## Security Model

### Trust Chain
`Governance/Admin` $\rightarrow$ `Staking Hub` $\rightarrow$ `User Shards` $\rightarrow$ `Trusts KYC Canister Principal`

### Access Control
*   **User Shards**: `internal_set_kyc_status` is strictly restricted to the `kyc_manager_id`.
*   **Staking Hub**: `admin_broadcast_kyc_manager` is restricted to canister controllers.
*   **KYC Canister**: Designed to handle complex logic and external integrations safely without compromising user staking balances.

---

## Management & Upgrades

To upgrade the KYC logic:
1.  Deploy the new KYC Canister.
2.  Update the Staking Hub:
    ```bash
    dfx canister call staking_hub admin_broadcast_kyc_manager '(principal "NEW_KYC_CANISTER_ID")'
    ```
3.  The Hub automatically propagates the new ID to all shards.

---

## Testing

A dedicated test script `scripts/test_kyc_flow.sh` is provided to verify the entire trust chain, from broadcasting the ID to remote activation of a user's verification tier.
