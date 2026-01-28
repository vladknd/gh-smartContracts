# Subscriptions Architecture

This document describes the design and implementation of the subscription system in the GHC ecosystem.

## Overview

The subscription system is designed to be **cross-canister** and **scalable**, supporting multiple user profile shards while maintaining a single source of truth for commercial logic.

### Components

1.  **Subscription Canister (Mutable/Upgradable)**: 
    *   The "Master" of commercial logic. 
    *   Handles integration with Web2 payment providers (Stripe).
    *   Verifies payments via HTTPS Outcalls.
    *   Directs User Shards to update user subscription status.
2.  **User Profile Shards (Immutable)**: 
    *   The "Slaves" of user state.
    *   Hold the `is_subscribed` flag for each user.
    *   Only trust calls from the authorized `subscription_manager_id`.
3.  **Staking Hub (The Orchestrator)**:
    *   Manages the configuration.
    *   Broadcasts the `subscription_manager_id` to all shards when it changes.

## The Subscription Flow (Stripe)

1.  **Selection**: The user selects a plan on the frontend.
2.  **Request Session**: The frontend calls `subscription_canister.request_checkout(user_shard_id)`.
3.  **Checkout Redirect**: The canister returns a Stripe Checkout URL (generated via a Web2 bridge).
4.  **Payment**: The user completes payment on the Stripe-hosted page.
5.  **Webhook Notification**: Stripe sends a webhook to the Bridge, which calls `subscription_canister.confirm_payment(session_id)`.
6.  **Direct Verification**: The `subscription_canister` performs an **HTTPS Outcall** to `api.stripe.com` to verify the session status and amount.
7.  **Remote Activation**: Once verified, the canister calls the user's specific shard: `shard.internal_set_subscription(user_principal, true)`.

## Security Model

### Trust Chain
`Governance/Admin` $\rightarrow$ `Staking Hub` $\rightarrow$ `User Shards` $\rightarrow$ `Trusts Subscription Canister Principal`

### Access Control
*   **User Shards**: `internal_set_subscription` is restricted strictly to the `subscription_manager_id`.
*   **Staking Hub**: `admin_broadcast_subscription_manager` is restricted to canister controllers.
*   **Subscription Canister**: Uses HTTPS Outcalls to ensure that even if the Webhook Bridge is compromised, payments cannot be spoofed.

## Configuration & Management

To update or change the subscription canister:
1.  Deploy the new Subscription Canister.
2.  Call the Staking Hub:
    ```bash
    dfx canister call staking_hub admin_broadcast_subscription_manager '(principal "NEW_CANISTER_ID")'
    ```
3.  The Hub will automatically update all existing shards. Any new shards created by the Hub will receive the correct ID during initialization.
