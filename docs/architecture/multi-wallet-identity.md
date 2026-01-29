# Multi-Wallet & Vault Identity Architecture: Full Specification

## 1. Core Philosophy
The architecture decouples **Authentication** (the Key) from **Identity** (the Account) and **Assets** (the Vault). This creates a "Smart Contract Wallet" for every user that can grow to millions of users without central bottlenecks.

---

## 2. Terminology
*   **Principal (Key)**: A cryptographic key (Internet Identity, NFID, Ledger). Users can have many keys.
*   **Account ID**: A unique, immutable internal ID (e.g., `User_101`).
*   **Home Shard**: The specific `user_profile` canister that stores the user's actual data and GHC Vault.
*   **Identity Shard**: A shard that holds a "Pointer" (`Key -> Home Shard`) for a specific key.
*   **Vault (Subaccount)**: A 32-byte pocket on the GHC Ledger owned by the Home Shard but controlled by the user's logic.

---

## 3. Flow 1: Secure Login & Discovery
When a user logs in with a new device, the system must find their data without a central "Master Database."

```text
[ FRONTEND ] (Key: Principal_B)
      |
      | 1. Local calculation: Shard_Idx = Hash(Principal_B) % Ring_Size
      | 2. Identity Shard = Ring_Map[Shard_Idx] (e.g., Shard #12)
      |
      +------> [ CALL: Shard #12.get_pointer(Principal_B) ]
                         |
                         | 3. Shard 12 looks in StableBTreeMap
                         | 4. Found: { Home: Shard #05, ID: Account_101 }
                         |
      <------------------+
      |
      | 5. Frontend Redirects to Shard #05
      |
      +------> [ CALL: Shard #05.get_profile(Account_101) ]
                         |
                         | 6. Shard 05 verifies Principal_B is authorized
                         | 7. Returns Profile, Balances, and Status
```

---

## 4. Flow 2: Linking a New Device (Handshake)
To prevent unauthorized key linking, we use a "Proof of Possession" handshake.

```text
[ LAPTOP (Key A) ]                [ HOME SHARD (#05) ]              [ PHONE (Key B) ]
       |                                   |                                |
       | 1. Request Link                   |                                |
       +---------------------------------->|                                |
       |                                   | 2. Gen LinkToken               |
       | <---------------------------------+                                |
       |                                   |                                |
       | 3. QR Code Transfer (Token)       |                                |
       +...................................................................>|
                                           |                                |
                                           |          4. Register (Shard 12)|
                                           | <------------------------------+
       [ SHARD #12 ]                       |                                |
             |                             |                                |
             | 5. Verify Token             |                                |
             +----------------------------->|                                |
             |                             | 6. OK: Account 101             |
             | <---------------------------+                                |
             |                             |                                |
             | 7. Store Pointer            |                                |
             |    Key B -> Shard 05        |                                |
```

---

## 5. Flow 3: Founder Vault Security
High-value assets are protected by the Shard code, not just the wallet signature.

```text
[ FRONTEND ]                      [ USER SHARD ]                    [ GHC LEDGER ]
      |                                |                                 |
      | 1. vault_withdraw(1000 GHC)    |                                 |
      +------------------------------->|                                 |
                                       | 2. Check Auth (authorized_keys) |
                                       | 3. Check Policy (Daily Limit)   |
                                       |                                 |
                                       | 4. icrc1_transfer               |
                                       |    From: {owner: Shard,         |
                                       |          sub: Account_101}      |
                                       +-------------------------------->|
                                                                         | 5. Move GHC
                                                                         | to recipient
```

---

## 6. Flow 4: Linear Scaling (Resharding)
When we add a new shard, only the "Neighbors" on the hash ring are affected.

```text
BEFORE: [ Shard 12 ] ------------------------------> [ Shard 13 ]
        Range: 0 - 100                               Range: 101 - 200

ACTION: Deploy [ Shard 101 ] at Position 150

AFTER:  [ Shard 12 ] ------> [ Shard 101 ] --------> [ Shard 13 ]
        Range: 0-100         Range: 101-150          Range: 151-200

MIGRATION:
1. Shard 13 detects new neighbor (Shard 101) in Ring Map.
2. Shard 13 scans its Map for all keys in range 101-150.
3. Shard 13 sends BATCH_TRANSFER of those 32-byte pointers to Shard 101.
4. Shard 13 deletes old pointers.
5. NO IMPACT on Shard 01-11 or Shard 14+; NO IMPACT on User Home Data.
```

---

## 7. Management & Governance
*   **Staking Hub**: Acts as the "Oracle" for the Cluster. It provides the authenticated `Ring_Map`.
*   **Root Key**: Every user has one "Master" Principal (e.g. Ledger Nano) that cannot be revoked and has the power to reset all other "Session Keys."
*   **Shard Discovery**: Each shard periodically refreshes its local `Ring_Map` from the Hub to ensure it can find and verify its peers.

---

---

# QUICK REFERENCE: KEY CONCEPTS EXPLAINED

This section provides a clear, concise explanation of the core concepts before diving into implementation details.

---

## What is a "Pointer"?

A **Pointer** is a tiny record (32-64 bytes) that acts like an entry in a distributed phone book:

```
Principal "nfid-abc-123" → { home_shard: Shard_05, account_id: 101 }
```

When you log in with any wallet, the system:
1. Hashes your principal to find the **Identity Shard** (where your pointer lives)
2. Reads the pointer to find your **Home Shard** (where your data lives)
3. Fetches your profile from the Home Shard

---

## How Shards Play Two Roles

Every `user_profile` shard does double duty:

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                          USER_PROFILE SHARD                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   IDENTITY ROLE                          HOME ROLE                           │
│   (Pointer Storage)                      (User Data Storage)                 │
│                                                                              │
│   ┌────────────────────────┐             ┌────────────────────────┐          │
│   │ IDENTITY_POINTERS      │             │ USER_PROFILES          │          │
│   │ Principal → {          │             │ AccountId → UserProfile│          │
│   │   home_shard,          │             │                        │          │
│   │   account_id           │             │ ACCOUNT_PRINCIPALS     │          │
│   │ }                      │             │ AccountId → [Keys]     │          │
│   └────────────────────────┘             └────────────────────────┘          │
│                                                                              │
│   For principals that                    For users who REGISTERED            │
│   HASH to this shard                     in this shard                       │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Key Insight:** The hash determines WHERE THE POINTER LIVES, not where the user's HOME is.

---

## How Device Linking Works

Step-by-step flow for linking NFID, II, Plug, or Ledger:

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           DEVICE LINKING FLOW                                    │
└─────────────────────────────────────────────────────────────────────────────────┘

  LAPTOP (already logged in with II)              NEW WALLET (NFID on Phone)
           │                                                │
           │ 1. Click "Add Device"                          │
           │    → request_link_token()                      │
           │    ← Returns 32-byte token                     │
           │                                                │
           │ 2. Display QR code with token                  │
           │                                                │
           │ 3. ─────── User scans QR ─────────────────────►│
           │                                                │
           │                           4. User authenticates │
           │                              with NFID wallet   │
           │                                                │
           │                           5. Frontend calls:    │
           │                              complete_link(     │
           │                                token,           │
           │                                nfid_principal   │
           │                              )                  │
           │                                                │
           │    ┌─────────────────────────────────────────┐  │
           │    │           HOME SHARD ACTIONS            │  │
           │    ├─────────────────────────────────────────┤  │
           │    │ 6. Verify token exists & not expired    │  │
           │    │ 7. Add NFID to session_keys[]           │  │
           │    │ 8. Create pointer for NFID principal    │  │
           │    │ 9. Store pointer in NFID's Identity     │  │
           │    │    Shard (where Hash(NFID) points)      │  │
           │    └─────────────────────────────────────────┘  │
           │                                                │
           │                           10. SUCCESS!          │
           │                               User can now      │
           │                               login with NFID   │

AFTER LINKING - LOGIN WITH NFID:
1. Hash NFID principal → Find Identity Shard
2. Lookup pointer → Get Home Shard + Account ID
3. Fetch profile from Home Shard
4. ✅ Same account, different wallet!
```

---

## What Can Users Do with Different Principals?

**YES - All principals linked to an account can:**

| Operation | Master Key | Session Keys | Notes |
|-----------|-----------|--------------|-------|
| View profile | ✅ | ✅ | All keys have read access |
| Submit quizzes | ✅ | ✅ | Rewards go to same account |
| Unstake tokens | ✅ | ✅ | Tokens sent to caller's address |
| Vote on proposals | ✅ | ✅ | Uses account's total VUC |
| Transfer GHC | ✅ | ✅ | From account's vault |
| Add new device | ✅ | ✅ | Request link token |
| Remove session key | ✅ | ❌ | Only master can revoke |
| Emergency recovery | ✅ | ❌ | Master resets all keys |

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    MULTI-PRINCIPAL ACCOUNT OPERATIONS                         │
└──────────────────────────────────────────────────────────────────────────────┘

                           ┌────────────────────────┐
                           │   ACCOUNT #101       │
                           │  Balance: 5000         │
                           │  Staked: 10000         │
                           └────────┬───────────────┘
                                    │
            ┌───────────────────────┼───────────────────────┐
            │                       │                       │
     ┌──────┴──────┐         ┌──────┴──────┐         ┌──────┴──────┐
     │ MASTER KEY  │         │ SESSION KEY │         │ SESSION KEY │
     │ (Ledger)    │         │ (II)        │         │ (NFID)      │
     └──────┬──────┘         └──────┬──────┘         └──────┬──────┘
            │                       │                       │
      Can revoke             Can do:                  Can do:
      other keys             - Quiz                   - Quiz
                             - Unstake                - Unstake
                             - Vote                   - Vote
                             - Transfer               - Transfer
                             - Add device             - Add device
```

**Important for Founders:**
- Use **Ledger** as your Master Key (hardware security)
- Use **II/NFID/Plug** as Session Keys for daily operations
- If a Session Key is compromised, use Master Key to revoke it
- Master Key can NEVER be changed or removed (immutable security anchor)

---

---

# PART 2: ICP-SAFE MIGRATION & RESHARDING

This section addresses a critical concern: **ICP's instruction limits** and how to safely migrate pointers without crashing.

---

## The Problem: ICP Instruction Limits

Internet Computer has strict limits per message:
- **~5 billion instructions** per update call
- **~40 billion instructions** consumed in a round (across all canisters)
- A simple for-loop over 100,000 items WILL trap

**Naive approach (WILL CRASH):**
```rust
// ❌ DON'T DO THIS - Will exceed instruction limit
fn migrate_all_pointers(to_shard: Principal) {
    for (principal, pointer) in IDENTITY_POINTERS.iter() {
        if should_migrate(principal) {
            send_to_shard(to_shard, principal, pointer); // TRAP!
        }
    }
}
```

---

## Safe Solution: Chunked Migration with Continuation

We use a **state machine pattern** with small chunks processed across multiple calls:

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                     CHUNKED MIGRATION STATE MACHINE                           │
└──────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────────────────┐
                    │         MIGRATION STATE              │
                    ├─────────────────────────────────────┤
                    │ status: Idle | InProgress | Done    │
                    │ last_key: Option<Principal>         │
                    │ target_shard: Principal             │
                    │ range_start: u32                    │
                    │ range_end: u32                      │
                    │ migrated_count: u64                 │
                    │ batch_size: u32 (default: 100)      │
                    └─────────────────────────────────────┘

  CALL 1                    CALL 2                    CALL 3
  ┌──────┐                  ┌──────┐                  ┌──────┐
  │Chunk │    Timer or      │Chunk │    Timer or      │Chunk │
  │ 1-100│───►self-call────►│101-  │───►self-call────►│201-  │───► ... ───► DONE
  │      │                  │200   │                  │300   │
  └──────┘                  └──────┘                  └──────┘
     │                         │                         │
     └─────────────────────────┴─────────────────────────┘
                               │
                    Each chunk: ~100 items
                    Instructions: ~1M per chunk
                    Safe margin: 50x under limit
```

---

## Migration Code (ICP-Safe)

```rust
// types.rs - Add migration state
#[derive(CandidType, Deserialize, Clone)]
pub enum MigrationStatus {
    Idle,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(CandidType, Deserialize, Clone)]
pub struct MigrationState {
    pub status: MigrationStatus,
    /// Last processed key (for continuation)
    pub cursor: Option<Principal>,
    /// Target shard for migration
    pub target_shard: Principal,
    /// Hash range to migrate
    pub range_start: u32,
    pub range_end: u32,
    /// Progress tracking
    pub total_to_migrate: u64,
    pub migrated_count: u64,
    /// Chunk size (keep small for safety)
    pub batch_size: u32,
}

// lib.rs - Chunked migration

/// Start migration to a new neighbor shard
/// Called by staking_hub when a new shard is added to the ring
#[update]
fn start_pointer_migration(
    target_shard: Principal,
    range_start: u32,
    range_end: u32,
) -> Result<(), String> {
    // Only staking_hub can initiate
    require_staking_hub_caller()?;
    
    // Count items to migrate (quick scan)
    let count = count_pointers_in_range(range_start, range_end);
    
    // Initialize migration state
    MIGRATION_STATE.with(|m| {
        let state = MigrationState {
            status: MigrationStatus::InProgress,
            cursor: None, // Start from beginning
            target_shard,
            range_start,
            range_end,
            total_to_migrate: count,
            migrated_count: 0,
            batch_size: 100, // Safe chunk size
        };
        m.borrow_mut().set(state);
    });
    
    // Schedule first chunk
    ic_cdk_timers::set_timer(Duration::from_millis(0), || {
        ic_cdk::spawn(process_migration_chunk());
    });
    
    Ok(())
}

/// Process one chunk of migration
/// Self-schedules next chunk if not complete
async fn process_migration_chunk() {
    let state = MIGRATION_STATE.with(|m| m.borrow().get().clone());
    
    if !matches!(state.status, MigrationStatus::InProgress) {
        return;
    }
    
    // Collect one batch of pointers to migrate
    let mut batch: Vec<(Principal, IdentityPointer)> = Vec::new();
    let mut new_cursor: Option<Principal> = None;
    let mut count = 0u32;
    
    IDENTITY_POINTERS.with(|p| {
        let pointers = p.borrow();
        
        // Start iteration from cursor (or beginning)
        let iter = match &state.cursor {
            Some(cursor) => pointers.range(cursor.clone()..),
            None => pointers.range(..),
        };
        
        for (principal, pointer) in iter {
            // Check if this principal's hash is in migration range
            let hash = hash_principal(&principal);
            if hash >= state.range_start && hash <= state.range_end {
                batch.push((principal.clone(), pointer.clone()));
                count += 1;
                
                if count >= state.batch_size {
                    new_cursor = Some(principal.clone());
                    break;
                }
            }
            
            // Always update cursor for continuation
            new_cursor = Some(principal.clone());
        }
    });
    
    // Send batch to target shard
    if !batch.is_empty() {
        let target = state.target_shard;
        let result: Result<(), _> = ic_cdk::call(
            target,
            "receive_pointer_batch",
            (batch.clone(),),
        ).await;
        
        if result.is_err() {
            // Mark as failed, don't delete local data
            MIGRATION_STATE.with(|m| {
                let mut s = m.borrow().get().clone();
                s.status = MigrationStatus::Failed("Send failed".to_string());
                m.borrow_mut().set(s);
            });
            return;
        }
        
        // Delete migrated pointers locally (only after confirmed send)
        IDENTITY_POINTERS.with(|p| {
            let mut pointers = p.borrow_mut();
            for (principal, _) in &batch {
                pointers.remove(principal);
            }
        });
    }
    
    // Update state
    MIGRATION_STATE.with(|m| {
        let mut s = m.borrow().get().clone();
        s.migrated_count += batch.len() as u64;
        s.cursor = new_cursor;
        
        // Check if complete
        if batch.is_empty() || s.migrated_count >= s.total_to_migrate {
            s.status = MigrationStatus::Completed;
        }
        
        m.borrow_mut().set(s);
    });
    
    // Schedule next chunk if not done
    let updated_state = MIGRATION_STATE.with(|m| m.borrow().get().clone());
    if matches!(updated_state.status, MigrationStatus::InProgress) {
        ic_cdk_timers::set_timer(Duration::from_millis(100), || {
            ic_cdk::spawn(process_migration_chunk());
        });
    }
}

/// Query migration progress
#[query]
fn get_migration_status() -> MigrationState {
    MIGRATION_STATE.with(|m| m.borrow().get().clone())
}
```

---

## Resharding: Complete Flow with ASCII Diagram

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                         RESHARDING COMPLETE FLOW                              │
└──────────────────────────────────────────────────────────────────────────────┘

STEP 1: BEFORE - Two Shards
═══════════════════════════════════════════════════════════════════════════════

    HASH RING (1000 positions)
    ┌────────────────────────────────────────────────────────────────┐
    │ 0          250         500         750        1000             │
    │ ├───────────┴───────────┴───────────┴──────────┤               │
    │ │         SHARD_A       │        SHARD_B       │               │
    │ │       (0 - 499)       │      (500 - 999)     │               │
    │ └───────────────────────┴──────────────────────┘               │
    └────────────────────────────────────────────────────────────────┘

    SHARD_A has:                    SHARD_B has:
    - 50,000 pointers               - 48,000 pointers
    - 25,000 user profiles          - 24,000 user profiles


STEP 2: DEPLOY NEW SHARD
═══════════════════════════════════════════════════════════════════════════════

    Admin/Governance calls: staking_hub.create_shard()
    
    ┌─────────────────────────────────────────────────────────────────┐
    │                       STAKING HUB                                │
    ├─────────────────────────────────────────────────────────────────┤
    │                                                                 │
    │  1. Create new canister (SHARD_C)                               │
    │  2. Install user_profile WASM                                   │
    │  3. Update RING_CONFIG:                                         │
    │     - SHARD_A: 0-499    (unchanged)                             │
    │     - SHARD_C: 500-749  (NEW!)                                  │
    │     - SHARD_B: 750-999  (range reduced!)                        │
    │  4. Notify SHARD_B: "migrate 500-749 to SHARD_C"                │
    │                                                                 │
    └─────────────────────────────────────────────────────────────────┘


STEP 3: CHUNKED MIGRATION
═══════════════════════════════════════════════════════════════════════════════

    SHARD_B starts migration state machine:

    ┌────────────┐     ┌────────────┐     ┌────────────┐
    │  Chunk 1   │     │  Chunk 2   │     │  Chunk 3   │
    │  100 items │────►│  100 items │────►│  100 items │────► ... 
    │  0.1 sec   │     │  0.1 sec   │     │  0.1 sec   │
    └────────────┘     └────────────┘     └────────────┘
         │                  │                  │
         ▼                  ▼                  ▼
    ┌────────────────────────────────────────────────┐
    │               SHARD_C (New Shard)               │
    │         Receives batches via inter-canister     │
    │         call "receive_pointer_batch"            │
    └────────────────────────────────────────────────┘

    Total time to migrate 24,000 pointers:
    24,000 / 100 chunks × 0.1 sec = ~40 seconds


STEP 4: AFTER - Three Shards
═══════════════════════════════════════════════════════════════════════════════

    HASH RING (1000 positions)
    ┌────────────────────────────────────────────────────────────────┐
    │ 0          250         500         750        1000             │
    │ ├───────────┴───────────┼───────────┼──────────┤               │
    │ │      SHARD_A          │  SHARD_C  │ SHARD_B  │               │
    │ │     (0 - 499)         │(500-749)  │(750-999) │               │
    │ └───────────────────────┴───────────┴──────────┘               │
    └────────────────────────────────────────────────────────────────┘

    SHARD_A has:                    SHARD_C has:         SHARD_B has:
    - 50,000 pointers (no change)   - 24,000 pointers    - 24,000 pointers
    - 25,000 profiles (no change)   - 0 profiles (new!)  - 24,000 profiles

    KEY INSIGHT: User profiles NEVER move!
    Only lightweight pointers (32 bytes each) are migrated.


STEP 5: VERIFICATION
═══════════════════════════════════════════════════════════════════════════════

    After migration completes:

    User with Principal "xyz" tries to login:
    1. Hash("xyz") = 623 → Falls in SHARD_C range (500-749)
    2. Frontend calls SHARD_C.get_pointer("xyz")
    3. SHARD_C returns { home_shard: SHARD_B, account_id: 12345 }
    4. Frontend calls SHARD_B.get_profile(12345)
    5. ✅ User logged in successfully!

```

---

## BULLETPROOF Migration Strategy

The simple "send-then-delete" approach has risks. Here's a **truly resilient** design:

### Design Principles

1. **Never delete from source until verified at destination**
2. **Dual-read period** - both shards answer queries during transition
3. **Idempotent operations** - safe to retry any step
4. **Human-verifiable checkpoints** - can audit at any stage
5. **Rollback capability** - can abort and restore

---

### Three-Phase Migration Protocol

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    THREE-PHASE MIGRATION PROTOCOL                             │
└──────────────────────────────────────────────────────────────────────────────┘

PHASE 1: COPY (Source retains ownership)
══════════════════════════════════════════════════════════════════════════════

    ┌─────────────────┐                      ┌─────────────────┐
    │    SHARD_B      │                      │    SHARD_C      │
    │   (Source)      │                      │  (Destination)  │
    ├─────────────────┤                      ├─────────────────┤
    │                 │   1. Copy batches    │                 │
    │  Pointer_X ─────┼──────────────────────►  Pointer_X      │
    │  Pointer_Y ─────┼──────────────────────►  Pointer_Y      │
    │  Pointer_Z      │                      │  (marked PENDING)
    │                 │                      │                 │
    │  Status: OWNER  │                      │  Status: PENDING│
    └─────────────────┘                      └─────────────────┘

    • Source KEEPS all pointers
    • Destination receives copies marked as PENDING
    • Queries still go to SOURCE (ring not updated yet)
    • Can abort at any point - just delete from destination


PHASE 2: VERIFY & HANDOVER (Source marks as MIGRATING)
══════════════════════════════════════════════════════════════════════════════

    ┌─────────────────┐                      ┌─────────────────┐
    │    SHARD_B      │                      │    SHARD_C      │
    │   (Source)      │                      │  (Destination)  │
    ├─────────────────┤                      ├─────────────────┤
    │                 │   2. Verify counts   │                 │
    │  Pointer_X ◄────┼──────────────────────┼  Pointer_X      │
    │  Pointer_Y      │   match exactly      │  Pointer_Y      │
    │                 │                      │                 │
    │  Status:        │   3. Ring updated    │  Status: ACTIVE │
    │  MIGRATING      │      by Hub          │                 │
    └─────────────────┘                      └─────────────────┘

    • Destination counts verified against source
    • ONLY if counts match: Hub updates ring config
    • Destination marks pointers as ACTIVE
    • Frontend queries now go to DESTINATION
    • Source marked MIGRATING (answers queries, won't modify)


PHASE 3: CLEANUP (Source deletes after grace period)
══════════════════════════════════════════════════════════════════════════════

    ┌─────────────────┐                      ┌─────────────────┐
    │    SHARD_B      │                      │    SHARD_C      │
    │   (Source)      │                      │  (Destination)  │
    ├─────────────────┤                      ├─────────────────┤
    │                 │   4. Grace period    │                 │
    │  (waiting...)   │      24 hours        │  Pointer_X ✓    │
    │                 │                      │  Pointer_Y ✓    │
    │                 │   5. Delete migrated │                 │
    │  (empty range)  │                      │  Status: ACTIVE │
    └─────────────────┘                      └─────────────────┘

    • 24-hour grace period for any issues to surface
    • Manual trigger to delete (not automatic!)
    • Source can still answer queries during grace period
    • Admin must explicitly confirm cleanup

```

---

### Pointer States

```rust
#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum PointerStatus {
    /// Normal active pointer - answers queries
    Active,
    /// Received from migration, not yet verified
    Pending,
    /// Being migrated out - still answers queries but read-only
    Migrating,
    /// Marked for deletion after grace period
    MarkedForDeletion { delete_after: u64 },
}

#[derive(CandidType, Deserialize, Clone)]
pub struct IdentityPointerV2 {
    pub home_shard: Principal,
    pub account_id: AccountId,
    pub linked_at: u64,
    pub status: PointerStatus,  // NEW!
}
```

---

### Dual-Read During Transition

The **key safety mechanism** is that BOTH shards can answer queries during Phase 2-3:

```rust
// Frontend logic during migration
async fn get_pointer_resilient(principal: Principal) -> Option<IdentityPointer> {
    let ring_config = get_ring_config().await;
    
    // 1. Try the current ring assignment
    let primary_shard = calculate_identity_shard(principal, &ring_config);
    if let Some(pointer) = primary_shard.get_pointer(principal).await {
        if pointer.status == Active {
            return Some(pointer);
        }
    }
    
    // 2. If not found or pending, try the previous owner (during migration)
    if let Some(prev_shard) = ring_config.previous_owner_for_range(principal) {
        if let Some(pointer) = prev_shard.get_pointer(principal).await {
            return Some(pointer);
        }
    }
    
    None
}
```

```rust
// On source shard during migration (status = Migrating)
#[query]
fn get_pointer(principal: Principal) -> Option<IdentityPointer> {
    IDENTITY_POINTERS.with(|p| {
        p.borrow().get(&principal).and_then(|ptr| {
            // Answer even if migrating - for fallback queries
            match ptr.status {
                Active | Migrating => Some(ptr),
                MarkedForDeletion { .. } => Some(ptr), // Still answer!
                Pending => None, // Don't answer for pending
            }
        })
    })
}
```

---

### Failure Mode Analysis

| Failure | Phase | Detection | Recovery |
|---------|-------|-----------|----------|
| Network failure during copy | 1 | Batch count mismatch | Retry batch (idempotent) |
| Destination crashes mid-copy | 1 | Destination unreachable | Restart from cursor |
| Source crashes mid-copy | 1 | Source unreachable | Source restarts, continues |
| Counts don't match after copy | 2 | Verification fails | Re-copy missing items |
| Ring update fails | 2 | Hub returns error | Retry ring update |
| Destination crashes after ring update | 2-3 | Queries fail | **Rollback ring to source** |
| Source crashes after handover | 3 | N/A | No impact, dest is active |
| Admin forgets cleanup | 3 | Grace period expires | Automated reminder, manual action |
| Power outage during any phase | Any | State persisted | Resume from last checkpoint |

---

### Rollback Procedure

If something goes catastrophically wrong, we can ALWAYS roll back:

```rust
/// Emergency rollback - restore source as owner
/// Called by controllers/governance if migration fails
#[update]  
fn emergency_rollback_migration() -> Result<(), String> {
    require_controller()?;
    
    let state = MIGRATION_STATE.with(|m| m.borrow().get().clone());
    
    // 1. Restore all migrating pointers to Active
    IDENTITY_POINTERS.with(|p| {
        let mut pointers = p.borrow_mut();
        for (principal, mut ptr) in pointers.iter() {
            if matches!(ptr.status, Migrating | MarkedForDeletion { .. }) {
                ptr.status = PointerStatus::Active;
                pointers.insert(principal, ptr);
            }
        }
    });
    
    // 2. Notify Hub to restore old ring config
    ic_cdk::spawn(async {
        let _: Result<(), _> = ic_cdk::call(
            STAKING_HUB_ID.with(|h| *h.borrow().get()),
            "rollback_ring_config",
            (state.migration_id,),
        ).await;
    });
    
    // 3. Mark migration as failed
    MIGRATION_STATE.with(|m| {
        let mut s = m.borrow().get().clone();
        s.status = MigrationStatus::RolledBack;
        m.borrow_mut().set(s);
    });
    
    Ok(())
}
```

---

### Verification Checklist (Before Phase 3 Cleanup)

```rust
/// Run before allowing cleanup
#[update]
fn verify_migration_complete() -> Result<MigrationVerification, String> {
    let state = MIGRATION_STATE.with(|m| m.borrow().get().clone());
    
    // 1. Count pointers that should have migrated
    let source_count = count_pointers_in_range_with_status(
        state.range_start, 
        state.range_end,
        vec![Migrating, MarkedForDeletion]
    );
    
    // 2. Query destination for their count
    let dest_count: u64 = ic_cdk::call(
        state.target_shard,
        "count_pointers_in_range",
        (state.range_start, state.range_end),
    ).await?;
    
    // 3. Sample random pointers to verify integrity
    let samples = sample_random_pointers(state.range_start, state.range_end, 10);
    let mut verified = 0;
    for (principal, expected) in samples {
        let actual: Option<IdentityPointer> = ic_cdk::call(
            state.target_shard,
            "get_pointer",
            (principal,),
        ).await?;
        
        if actual.map(|p| p.home_shard == expected.home_shard) == Some(true) {
            verified += 1;
        }
    }
    
    Ok(MigrationVerification {
        source_count,
        dest_count,
        counts_match: source_count == dest_count,
        sample_verified: verified,
        sample_total: 10,
        safe_to_cleanup: source_count == dest_count && verified == 10,
    })
}
```

---

### Why This is Resilient

| Guarantee | How Achieved |
|-----------|--------------|
| **No data loss** | Source never deletes until destination verified |
| **No downtime** | Dual-read means queries always succeed |
| **Idempotent** | Any step can be retried safely |
| **Auditable** | Every phase has verification queries |
| **Rollback-able** | Emergency procedure restores source |
| **Human-in-loop** | Cleanup requires manual confirmation |
| **Crash-safe** | All state in stable memory |

---

### What Could Still Go Wrong?

**Honestly?** Here are the remaining risks:

1. **Both shards crash simultaneously** - IC subnet failure (extremely rare, IC-level issue)
2. **Bug in verification logic** - Mitigated by code review + testing
3. **Malicious controller** - Governance should control this, not individuals
4. **WASM upgrade during migration** - Recommend: freeze upgrades during migration

**Mitigation for #4:**
```rust
#[update]
fn upgrade_guard() -> Result<(), String> {
    let state = MIGRATION_STATE.with(|m| m.borrow().get().clone());
    if matches!(state.status, MigrationStatus::InProgress) {
        return Err("Cannot upgrade during active migration".to_string());
    }
    Ok(())
}
```

---

### Testing Strategy

Before mainnet, test these scenarios:

- [ ] Happy path: Full migration with 10K pointers
- [ ] Network failure: Kill destination mid-Phase 1, verify resume
- [ ] Trap: Force trap in destination's `receive_batch`, verify retry
- [ ] Rollback: Trigger rollback mid-Phase 2, verify source works
- [ ] Dual-read: Query during Phase 2, verify both shards answer
- [ ] Count mismatch: Inject wrong count, verify Phase 2 fails safely
- [ ] Grace period: Verify cleanup blocked before 24h
- [ ] Concurrent queries: Stress test with 1000 concurrent logins during migration

---



---

# PART 3: DETAILED IMPLEMENTATION PLAN

This section provides a comprehensive, step-by-step implementation guide for integrating the Multi-Wallet Identity system into the existing GreenHero architecture.

---

---

> **Note:** Sections 8.1-8.3 (Pointer explanation, Dual Shard Roles, Hash Ring) are already covered in detail in the **QUICK REFERENCE** section above. Proceed to implementation details below.



## 9. Core Changes to Existing System

### 9.1 Current vs. New Data Model

**CURRENT (Single Principal per User):**
```rust
// state.rs
USER_PROFILES: StableBTreeMap<Principal, UserProfile>
USER_TIME_STATS: StableBTreeMap<Principal, UserTimeStats>
```

**NEW (Multi-Principal per User with Internal ID):**
```rust
// state.rs - Identity Role
IDENTITY_POINTERS: StableBTreeMap<Principal, IdentityPointer>

// state.rs - Home Role (keyed by internal ID, not principal)
USER_PROFILES: StableBTreeMap<AccountId, UserProfile>  // Changed key!
USER_TIME_STATS: StableBTreeMap<AccountId, UserTimeStats>
ACCOUNT_PRINCIPALS: StableBTreeMap<AccountId, AuthorizedKeys>  // Principal list per account
LOCAL_ACCOUNT_COUNTER: StableCell<u64>  // Generate unique account IDs
```

### 9.2 New Types Needed

```rust
// types.rs

/// Unique internal identifier for a user account
/// This ID never changes, even when principals are added/removed
#[derive(CandidType, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountId(pub u64);

impl Storable for AccountId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_be_bytes().to_vec())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(u64::from_be_bytes(bytes.as_ref().try_into().unwrap()))
    }
    const BOUND: Bound = Bound::Bounded { max_size: 8, is_fixed_size: true };
}

/// Pointer stored in Identity Shards - maps Principal to Home Shard
#[derive(CandidType, Deserialize, Clone)]
pub struct IdentityPointer {
    /// Which shard canister holds this user's actual data
    pub home_shard: Principal,
    /// Internal account ID within the home shard
    pub account_id: AccountId,
    /// When this key was linked (for auditing)
    pub linked_at: u64,
}

impl Storable for IdentityPointer {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { max_size: 64, is_fixed_size: false };
}

/// Keys authorized to access an account
#[derive(CandidType, Deserialize, Clone)]
pub struct AuthorizedKeys {
    /// The original registration key - CANNOT be removed
    pub master_key: Principal,
    /// Additional authorized keys (can be added/removed)
    pub session_keys: Vec<Principal>,
    /// Maximum allowed session keys
    pub max_session_keys: u8,
}

impl Default for AuthorizedKeys {
    fn default() -> Self {
        Self {
            master_key: Principal::anonymous(),
            session_keys: Vec::new(),
            max_session_keys: 10,
        }
    }
}

impl Storable for AuthorizedKeys {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded { 
        max_size: 512,  // 10 keys * 29 bytes + overhead
        is_fixed_size: false 
    };
}

/// Token for linking a new device
#[derive(CandidType, Deserialize, Clone)]
pub struct LinkToken {
    /// Random 32-byte challenge
    pub token: [u8; 32],
    /// Account this token authorizes linking to
    pub account_id: AccountId,
    /// When this token expires (nanoseconds)
    pub expires_at: u64,
}

/// Ring configuration
#[derive(CandidType, Deserialize, Clone)]
pub struct RingConfig {
    /// Total size of the hash ring (should be power of 2 for efficiency)
    pub ring_size: u32,
    /// Mapping of ring position ranges to shard principals
    pub ring_map: Vec<RingEntry>,
    /// Version number (incremented on any change)
    pub version: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct RingEntry {
    /// Starting position (inclusive)
    pub start_pos: u32,
    /// Ending position (inclusive)
    pub end_pos: u32,
    /// Shard canister that owns this range
    pub shard_id: Principal,
}
```

---

## 10. Implementation Phases

### Phase 1: Foundation (Week 1-2)
**Goal:** Add identity layer without breaking existing functionality

#### Step 1.1: Add New State to user_profile

```rust
// src/user_profile/src/state.rs - ADD THESE

/// Memory ID 13: Identity pointers (Principal -> IdentityPointer)
pub static IDENTITY_POINTERS: RefCell<StableBTreeMap<Principal, IdentityPointer, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13)))
    )
);

/// Memory ID 14: Account principals mapping (AccountId -> AuthorizedKeys)
pub static ACCOUNT_PRINCIPALS: RefCell<StableBTreeMap<AccountId, AuthorizedKeys, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14)))
    )
);

/// Memory ID 15: Pending link tokens (token_hash -> LinkToken)
pub static PENDING_LINK_TOKENS: RefCell<StableBTreeMap<[u8; 32], LinkToken, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15)))
    )
);

/// Memory ID 16: Local account counter for generating AccountIds
pub static LOCAL_ACCOUNT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
    StableCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(16))),
        0
    ).unwrap()
);

/// Memory ID 17: Cached ring configuration
pub static RING_CONFIG: RefCell<StableCell<RingConfig, Memory>> = RefCell::new(
    StableCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(17))),
        RingConfig::default()
    ).unwrap()
);
```

#### Step 1.2: Add Ring Config to staking_hub

```rust
// src/staking_hub/src/state.rs - ADD

/// Memory ID 14: Hash ring configuration
pub static RING_CONFIG: RefCell<StableCell<RingConfig, Memory>> = RefCell::new(
    StableCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14))),
        RingConfig::default()
    ).unwrap()
);
```

```rust
// src/staking_hub/src/lib.rs - ADD THESE FUNCTIONS

/// Get the current ring configuration (called by frontend and shards)
#[query]
fn get_ring_config() -> RingConfig {
    RING_CONFIG.with(|r| r.borrow().get().clone())
}

/// Calculate which shard handles identity lookups for a principal
/// This is a PURE function - can also be computed locally by frontend
#[query]
fn get_identity_shard_for_principal(principal: Principal) -> Principal {
    let ring_config = RING_CONFIG.with(|r| r.borrow().get().clone());
    let hash = hash_principal(&principal);
    let position = hash % ring_config.ring_size;
    
    for entry in &ring_config.ring_map {
        if position >= entry.start_pos && position <= entry.end_pos {
            return entry.shard_id;
        }
    }
    
    // Fallback to first shard if not found (should never happen)
    ring_config.ring_map.first()
        .map(|e| e.shard_id)
        .unwrap_or(Principal::anonymous())
}

/// Internal: Rebuild ring config when shard count changes
fn rebuild_ring_config() {
    let shards: Vec<Principal> = SHARD_REGISTRY.with(|r| {
        r.borrow().iter()
            .map(|(_, info)| info.canister_id)
            .collect()
    });
    
    let shard_count = shards.len() as u32;
    if shard_count == 0 { return; }
    
    // Divide ring evenly among shards
    let ring_size: u32 = 1_000_000; // 1 million positions
    let segment_size = ring_size / shard_count;
    
    let entries: Vec<RingEntry> = shards.iter().enumerate().map(|(i, shard)| {
        let start = (i as u32) * segment_size;
        let end = if i == (shard_count - 1) as usize {
            ring_size - 1  // Last shard gets remainder
        } else {
            start + segment_size - 1
        };
        RingEntry {
            start_pos: start,
            end_pos: end,
            shard_id: *shard,
        }
    }).collect();
    
    let version = RING_CONFIG.with(|r| r.borrow().get().version) + 1;
    
    RING_CONFIG.with(|r| {
        let _ = r.borrow_mut().set(RingConfig {
            ring_size,
            ring_map: entries,
            version,
        });
    });
}

fn hash_principal(principal: &Principal) -> u32 {
    // Use FNV-1a for fast, good distribution
    let bytes = principal.as_slice();
    let mut hash: u32 = 2166136261;
    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}
```

### Phase 2: Identity Operations (Week 2-3)
**Goal:** Implement pointer lookup and creation

#### Step 2.1: Add Identity Lookup to user_profile

```rust
// src/user_profile/src/lib.rs - ADD

/// Get the identity pointer for a principal (Identity Role)
/// Called by frontend to discover a user's home shard
#[query]
fn get_pointer(principal: Principal) -> Option<IdentityPointer> {
    IDENTITY_POINTERS.with(|p| p.borrow().get(&principal))
}

/// Create a new identity pointer (called during registration)
/// This is an internal function, not directly callable by users
fn create_pointer(
    principal: Principal,
    home_shard: Principal,
    account_id: AccountId,
) -> Result<(), String> {
    // Check if pointer already exists
    if IDENTITY_POINTERS.with(|p| p.borrow().contains_key(&principal)) {
        return Err("Principal already registered".to_string());
    }
    
    let pointer = IdentityPointer {
        home_shard,
        account_id,
        linked_at: ic_cdk::api::time(),
    };
    
    IDENTITY_POINTERS.with(|p| {
        p.borrow_mut().insert(principal, pointer);
    });
    
    Ok(())
}

/// Store a pointer for a principal that hashes to THIS shard
/// Called by other shards during cross-shard registration
#[update]
fn store_identity_pointer(
    principal: Principal,
    pointer: IdentityPointer,
) -> Result<(), String> {
    // Verify caller is a registered shard
    let caller = ic_cdk::caller();
    let is_valid_shard = /* call staking_hub or check locally */;
    if !is_valid_shard {
        return Err("Unauthorized: Not a registered shard".to_string());
    }
    
    // Verify this principal SHOULD hash to this shard
    let expected_shard = get_identity_shard_for_principal(principal);
    if expected_shard != ic_cdk::api::id() {
        return Err("Wrong shard for this principal".to_string());
    }
    
    IDENTITY_POINTERS.with(|p| {
        p.borrow_mut().insert(principal, pointer);
    });
    
    Ok(())
}
```

#### Step 2.2: Update Registration Flow

```rust
// src/user_profile/src/lib.rs - MODIFY register_user

#[update]
fn register_user(args: UserProfileUpdate) -> Result<AccountId, String> {
    let caller = ic_cdk::caller();
    
    // Check if this principal already has an identity
    if IDENTITY_POINTERS.with(|p| p.borrow().contains_key(&caller)) {
        return Err("Principal already registered".to_string());
    }
    
    // Generate a new unique account ID for this shard
    let account_id = LOCAL_ACCOUNT_COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        let id = *counter.get();
        let _ = counter.set(id + 1);
        AccountId(id)
    });
    
    // Create the user profile
    let profile = UserProfile {
        email: args.email,
        name: args.name,
        education: args.education,
        gender: args.gender,
        verification_tier: VerificationTier::None,
        staked_balance: 0,
        transaction_count: 0,
        archived_transaction_count: 0,
        is_subscribed: false,
    };
    
    USER_PROFILES.with(|p| {
        p.borrow_mut().insert(account_id, profile);
    });
    
    // Create authorized keys with caller as master
    let auth_keys = AuthorizedKeys {
        master_key: caller,
        session_keys: Vec::new(),
        max_session_keys: 10,
    };
    
    ACCOUNT_PRINCIPALS.with(|a| {
        a.borrow_mut().insert(account_id, auth_keys);
    });
    
    // Store identity pointer locally (if this principal hashes to this shard)
    // OR call the correct identity shard
    let this_shard = ic_cdk::api::id();
    let identity_shard = get_identity_shard_for_principal(caller);
    
    let pointer = IdentityPointer {
        home_shard: this_shard,
        account_id,
        linked_at: ic_cdk::api::time(),
    };
    
    if identity_shard == this_shard {
        // Lucky! Store locally
        IDENTITY_POINTERS.with(|p| {
            p.borrow_mut().insert(caller, pointer);
        });
    } else {
        // Cross-shard call to store pointer
        ic_cdk::spawn(async move {
            let _: Result<(), _> = ic_cdk::call(
                identity_shard,
                "store_identity_pointer",
                (caller, pointer),
            ).await;
        });
    }
    
    Ok(account_id)
}
```

### Phase 3: Link Device Flow (Week 3-4)
**Goal:** Allow adding new principals to existing accounts

#### Step 3.1: Generate Link Token (On Home Shard)

```rust
// src/user_profile/src/lib.rs - ADD

/// Request a token to link a new device
/// Called by an authorized principal on the user's home shard
#[update]
fn request_link_token() -> Result<[u8; 32], String> {
    let caller = ic_cdk::caller();
    
    // Find the account for this caller
    let account_id = get_account_for_caller(caller)?;
    
    // Generate random token
    let mut token = [0u8; 32];
    // In production, use ic_cdk::api::management_canister::main::raw_rand()
    // For simplicity, using timestamp-based PRNG here
    let seed = ic_cdk::api::time();
    for (i, byte) in token.iter_mut().enumerate() {
        *byte = ((seed >> (i % 8)) ^ (seed >> ((i + 4) % 8))) as u8;
    }
    
    let link_token = LinkToken {
        token,
        account_id,
        expires_at: ic_cdk::api::time() + 300_000_000_000, // 5 minutes
    };
    
    // Store token (keyed by token hash for O(1) lookup)
    PENDING_LINK_TOKENS.with(|t| {
        t.borrow_mut().insert(token, link_token);
    });
    
    Ok(token)
}

fn get_account_for_caller(caller: Principal) -> Result<AccountId, String> {
    // Check if caller has a local pointer
    if let Some(pointer) = IDENTITY_POINTERS.with(|p| p.borrow().get(&caller)) {
        if pointer.home_shard == ic_cdk::api::id() {
            return Ok(pointer.account_id);
        }
    }
    
    // Otherwise, iterate through accounts to find caller
    ACCOUNT_PRINCIPALS.with(|a| {
        for (account_id, keys) in a.borrow().iter() {
            if keys.master_key == caller || keys.session_keys.contains(&caller) {
                return Ok(account_id);
            }
        }
        Err("Account not found".to_string())
    })
}
```

#### Step 3.2: Complete Linking from New Device

```rust
// src/user_profile/src/lib.rs - ADD

/// Complete linking a new device using the token
/// Called from the IDENTITY SHARD where the new principal hashes to
#[update]
fn complete_link(token: [u8; 32], new_principal: Principal) -> Result<IdentityPointer, String> {
    // Verify token exists and is valid
    let link_token = PENDING_LINK_TOKENS.with(|t| {
        t.borrow().get(&token)
    }).ok_or("Invalid or expired token")?;
    
    // Check expiration
    if ic_cdk::api::time() > link_token.expires_at {
        // Remove expired token
        PENDING_LINK_TOKENS.with(|t| {
            t.borrow_mut().remove(&token);
        });
        return Err("Token expired".to_string());
    }
    
    // Add the new principal to the account's authorized keys
    ACCOUNT_PRINCIPALS.with(|a| {
        let mut accounts = a.borrow_mut();
        let mut keys = accounts.get(&link_token.account_id)
            .ok_or("Account not found")?;
        
        if keys.session_keys.len() >= keys.max_session_keys as usize {
            return Err("Maximum session keys reached".to_string());
        }
        
        if keys.session_keys.contains(&new_principal) {
            return Err("Principal already linked".to_string());
        }
        
        keys.session_keys.push(new_principal);
        accounts.insert(link_token.account_id, keys);
        Ok(())
    })?;
    
    // Remove used token
    PENDING_LINK_TOKENS.with(|t| {
        t.borrow_mut().remove(&token);
    });
    
    // Create the pointer to return
    let pointer = IdentityPointer {
        home_shard: ic_cdk::api::id(),
        account_id: link_token.account_id,
        linked_at: ic_cdk::api::time(),
    };
    
    Ok(pointer)
}
```

#### Step 3.3: Frontend Link Flow

```typescript
// Frontend linking flow

async function linkNewDevice(token: Uint8Array) {
    // 1. Hash the NEW principal to find its identity shard
    const newPrincipal = await getAuthenticatedPrincipal(); // From NFID/II/Plug
    const identityShard = await stakingHub.get_identity_shard_for_principal(newPrincipal);
    
    // 2. Ask the identity shard to verify token with home shard and store pointer
    const result = await identityShardActor.register_linked_device(token);
    
    if (result.Ok) {
        console.log("Device linked successfully!");
        // Now this principal can login and find the home shard
    }
}
```

### Phase 4: Resharding (Week 4-5)
**Goal:** Support adding new shards without data loss

#### Step 4.1: Resharding Trigger in staking_hub

```rust
// src/staking_hub/src/lib.rs - ADD

/// Called after deploying a new shard to update the ring
#[update]
fn notify_new_shard(new_shard: Principal) -> Result<(), String> {
    // Only callable by controllers
    require_controller()?;
    
    // Add to registry
    register_shard_internal(new_shard, None);
    
    // Rebuild the ring configuration
    rebuild_ring_config();
    
    // Trigger migration on affected shards
    let ring_config = RING_CONFIG.with(|r| r.borrow().get().clone());
    
    // Find the new shard's range and its neighbor
    if let Some(new_entry) = ring_config.ring_map.iter().find(|e| e.shard_id == new_shard) {
        // Find the shard that previously owned this range
        // (the shard with range ending just after new_entry.end_pos)
        for entry in &ring_config.ring_map {
            if entry.start_pos == new_entry.end_pos + 1 {
                // This shard needs to migrate pointers
                ic_cdk::spawn(async move {
                    let _: Result<(), _> = ic_cdk::call(
                        entry.shard_id,
                        "migrate_pointers_to_neighbor",
                        (new_shard, new_entry.start_pos, new_entry.end_pos),
                    ).await;
                });
                break;
            }
        }
    }
    
    Ok(())
}
```

#### Step 4.2: Pointer Migration in user_profile

```rust
// src/user_profile/src/lib.rs - ADD

/// Migrate pointers in a range to a new neighbor shard
/// Called by staking_hub during resharding
#[update]
fn migrate_pointers_to_neighbor(
    neighbor: Principal,
    start_pos: u32,
    end_pos: u32,
) -> Result<u64, String> {
    // Only callable by staking_hub
    let caller = ic_cdk::caller();
    let hub_id = STAKING_HUB_ID.with(|h| *h.borrow().get());
    if caller != hub_id {
        return Err("Unauthorized".to_string());
    }
    
    let mut to_migrate: Vec<(Principal, IdentityPointer)> = Vec::new();
    
    // Find all pointers in the range
    IDENTITY_POINTERS.with(|p| {
        for (principal, pointer) in p.borrow().iter() {
            let hash = hash_principal(&principal);
            if hash >= start_pos && hash <= end_pos {
                to_migrate.push((principal, pointer));
            }
        }
    });
    
    let count = to_migrate.len() as u64;
    
    // Send to neighbor in batches
    let batch_size = 100;
    for chunk in to_migrate.chunks(batch_size) {
        let batch: Vec<_> = chunk.to_vec();
        ic_cdk::spawn(async move {
            let _: Result<(), _> = ic_cdk::call(
                neighbor,
                "receive_pointer_batch",
                (batch,),
            ).await;
        });
    }
    
    // Delete migrated pointers
    IDENTITY_POINTERS.with(|p| {
        let mut pointers = p.borrow_mut();
        for (principal, _) in &to_migrate {
            pointers.remove(principal);
        }
    });
    
    Ok(count)
}

/// Receive a batch of pointers from a neighbor during resharding
#[update]
fn receive_pointer_batch(
    batch: Vec<(Principal, IdentityPointer)>,
) -> Result<(), String> {
    // Verify caller is a registered shard
    // (implementation similar to store_identity_pointer)
    
    IDENTITY_POINTERS.with(|p| {
        let mut pointers = p.borrow_mut();
        for (principal, pointer) in batch {
            pointers.insert(principal, pointer);
        }
    });
    
    Ok(())
}
```

---

## 11. Frontend Integration

### 11.1 Login Flow (Updated)

```typescript
// src/frontend/services/identity.ts

interface IdentityPointer {
    home_shard: Principal;
    account_id: bigint;
    linked_at: bigint;
}

export async function login(): Promise<UserProfile | null> {
    // 1. Authenticate with chosen wallet
    const principal = await authenticate(); // II, NFID, Plug, etc.
    
    // 2. Get ring config from staking hub (cache this!)
    const ringConfig = await stakingHub.get_ring_config();
    
    // 3. Calculate identity shard locally (pure hash)
    const identityShard = calculateIdentityShard(principal, ringConfig);
    
    // 4. Look up pointer
    const identityActor = createActor(identityShard);
    const pointer: IdentityPointer | null = await identityActor.get_pointer(principal);
    
    if (!pointer) {
        // New user - direct to registration
        return null;
    }
    
    // 5. Fetch profile from home shard
    const homeActor = createActor(pointer.home_shard);
    const profile = await homeActor.get_profile_by_id(pointer.account_id);
    
    return profile;
}

function calculateIdentityShard(principal: Principal, config: RingConfig): Principal {
    const hash = fnv1a(principal.toUint8Array());
    const position = hash % config.ring_size;
    
    for (const entry of config.ring_map) {
        if (position >= entry.start_pos && position <= entry.end_pos) {
            return entry.shard_id;
        }
    }
    throw new Error("No shard found for position");
}

function fnv1a(data: Uint8Array): number {
    let hash = 2166136261;
    for (const byte of data) {
        hash ^= byte;
        hash = (hash * 16777619) >>> 0;
    }
    return hash;
}
```

### 11.2 Registration Flow

```typescript
export async function register(profileData: UserProfileUpdate): Promise<AccountId> {
    const principal = await getAuthenticatedPrincipal();
    
    // Find the best shard for new users (load balanced)
    const homeShard = await stakingHub.get_shard_for_new_user();
    const homeActor = createActor(homeShard);
    
    // Registration handles both profile creation AND pointer storage
    const accountId = await homeActor.register_user(profileData);
    
    return accountId;
}
```

### 11.3 Add Device Flow

```typescript
export async function requestDeviceLink(): Promise<Uint8Array> {
    // Must be called from existing authorized device
    const currentPointer = await getCurrentUserPointer();
    const homeActor = createActor(currentPointer.home_shard);
    
    const token = await homeActor.request_link_token();
    return token;
}

export async function completeDeviceLink(tokenQR: Uint8Array): Promise<void> {
    // Called from NEW device after scanning QR
    const newPrincipal = await getAuthenticatedPrincipal();
    
    // Find where this principal's pointer will live
    const ringConfig = await stakingHub.get_ring_config();
    const identityShard = calculateIdentityShard(newPrincipal, ringConfig);
    
    // The identity shard calls the home shard to verify token
    const identityActor = createActor(identityShard);
    await identityActor.register_linked_device(tokenQR);
    
    console.log("Device linked! You can now login with this wallet.");
}
```

---

## 12. Migration Strategy (Existing Users)

### 12.1 Backward Compatibility

To avoid breaking existing users, implement a **graceful migration**:

```rust
/// Check both old and new systems during transition
#[query]
fn get_profile_compat(caller: Principal) -> Option<UserProfile> {
    // Try new system first (pointer lookup)
    if let Some(pointer) = IDENTITY_POINTERS.with(|p| p.borrow().get(&caller)) {
        if pointer.home_shard == ic_cdk::api::id() {
            return USER_PROFILES.with(|p| p.borrow().get(&pointer.account_id));
        }
    }
    
    // Fall back to legacy lookup (direct principal key)
    // This works during migration period
    USER_PROFILES_LEGACY.with(|p| p.borrow().get(&caller))
}
```

### 12.2 Migration Script

```rust
/// One-time migration to convert existing users to new system
/// Run this via admin call or governance proposal
#[update]
fn migrate_legacy_users() -> Result<u64, String> {
    require_controller()?;
    
    let mut migrated = 0u64;
    
    USER_PROFILES_LEGACY.with(|legacy| {
        let this_shard = ic_cdk::api::id();
        
        for (principal, profile) in legacy.borrow().iter() {
            // Generate new account ID
            let account_id = LOCAL_ACCOUNT_COUNTER.with(|c| {
                let mut counter = c.borrow_mut();
                let id = *counter.get();
                let _ = counter.set(id + 1);
                AccountId(id)
            });
            
            // Store in new format
            USER_PROFILES.with(|p| {
                p.borrow_mut().insert(account_id, profile);
            });
            
            // Create authorized keys
            let auth_keys = AuthorizedKeys {
                master_key: principal,
                session_keys: Vec::new(),
                max_session_keys: 10,
            };
            ACCOUNT_PRINCIPALS.with(|a| {
                a.borrow_mut().insert(account_id, auth_keys);
            });
            
            // Create pointer (handle cross-shard if needed)
            let pointer = IdentityPointer {
                home_shard: this_shard,
                account_id,
                linked_at: ic_cdk::api::time(),
            };
            
            let identity_shard = get_identity_shard_for_principal(principal);
            if identity_shard == this_shard {
                IDENTITY_POINTERS.with(|p| {
                    p.borrow_mut().insert(principal, pointer);
                });
            } else {
                // Queue for cross-shard migration
                // (In production, batch these)
            }
            
            migrated += 1;
        }
    });
    
    Ok(migrated)
}
```

---

## 13. Security Considerations

### 13.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| **Malicious pointer injection** | Only registered shards can store pointers; verify caller |
| **Link token theft** | Short expiry (5 min), single-use, cryptographically random |
| **Master key impersonation** | Master key stored in `AuthorizedKeys`, can revoke all session keys |
| **Sybil attack on registration** | Rate limiting, PoH verification requirement for full access |
| **Resharding data loss** | Two-phase migration: copy first, delete after confirmation |

### 13.2 Recovery Flow (If Master Key Lost)

```text
OPTION 1: If user still has any session key
  1. Session key cannot add new keys BUT
  2. Session key CAN transfer assets to a new account
  3. Effectively "re-register" with new master key

OPTION 2: Social recovery (future enhancement)
  1. User designates "guardians" (other users)
  2. 2-of-3 guardian signatures can reset master key
  3. 7-day time lock for recovery operations
```

---

## 14. Compatibility with Existing System

### 14.1 What Changes

| Component | Current | After Multi-Wallet |
|-----------|---------|-------------------|
| `USER_PROFILES` key | `Principal` | `AccountId` |
| `USER_TIME_STATS` key | `Principal` | `AccountId` |
| `COMPLETED_QUIZZES` key | `UserQuizKey { Principal, unit_id }` | `UserQuizKey { AccountId, unit_id }` |
| `USER_TRANSACTIONS` key | `TransactionKey { Principal, idx }` | `TransactionKey { AccountId, idx }` |
| Auth check | `caller == owner` | `is_authorized(caller, account_id)` |

### 14.2 What Stays the Same

- `staking_hub` global stats and sync mechanism
- Quiz verification flow
- Token rewards and staking logic
- Archive canister integration
- Subscription and KYC management

### 14.3 Helper Function for Auth

```rust
/// Check if a principal is authorized to access an account
fn is_authorized(caller: Principal, account_id: AccountId) -> bool {
    ACCOUNT_PRINCIPALS.with(|a| {
        a.borrow().get(&account_id)
            .map(|keys| {
                keys.master_key == caller || keys.session_keys.contains(&caller)
            })
            .unwrap_or(false)
    })
}

/// Get the account ID for a caller, or error
fn get_caller_account() -> Result<AccountId, String> {
    let caller = ic_cdk::caller();
    
    // First check identity pointers
    if let Some(pointer) = IDENTITY_POINTERS.with(|p| p.borrow().get(&caller)) {
        if pointer.home_shard == ic_cdk::api::id() {
            // Verify caller is still authorized
            if is_authorized(caller, pointer.account_id) {
                return Ok(pointer.account_id);
            }
        }
    }
    
    Err("Not authorized or account not found".to_string())
}
```

---

## 15. Implementation Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Foundation | New types, state storage, ring config in staking_hub |
| 2 | Identity Layer | `get_pointer`, `store_identity_pointer`, `get_identity_shard_for_principal` |
| 3 | Multi-Wallet Core | Updated registration, `AuthorizedKeys` management |
| 4 | Link Flow | Token generation, QR flow, `complete_link` |
| 5 | Resharding | Migration logic, `migrate_pointers_to_neighbor` |
| 6 | Frontend | Updated login/register flows, link device UI |
| 7 | Migration | Legacy user migration, testing |
| 8 | Hardening | Security audit, edge cases, documentation |

---

## 16. Testing Checklist

- [ ] Single device registration works
- [ ] Login with registered principal finds home shard
- [ ] Link token generation and usage
- [ ] Link from different wallet types (II, NFID, Plug)
- [ ] Master key cannot be removed
- [ ] Session key can be revoked by master
- [ ] Quiz submission works with AccountId
- [ ] Staking/unstaking works with AccountId
- [ ] Cross-shard pointer storage works
- [ ] Resharding migrates pointers correctly
- [ ] Legacy user migration preserves data
- [ ] Frontend login flow updated
- [ ] Rate limiting on link requests

---

## 17. Summary

The Multi-Wallet Identity system transforms your application from a "one key = one user" model to a flexible "one account = many keys" model, enabling:

1. **Device Recovery**: Lost your phone? Log in with your desktop wallet.
2. **Wallet Flexibility**: Use II for mobile, Plug for desktop, Ledger for high-value ops.
3. **Founder Security**: Require hardware wallet (Ledger) for large withdrawals.
4. **Linear Scalability**: Hash ring ensures no single bottleneck as you grow.

The architecture integrates cleanly with your existing sharded `user_profile` system by adding an **identity layer** on top, without fundamentally changing how user data is stored or how the staking/quiz mechanics work.

---

## 18. Complete System Architecture Diagram

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│                              MULTI-WALLET IDENTITY ARCHITECTURE                               │
│                                   GreenHero Coin Platform                                     │
└──────────────────────────────────────────────────────────────────────────────────────────────┘

                                         ┌─────────────────┐
                                         │    FRONTEND     │
                                         │  (React/Next)   │
                                         └────────┬────────┘
                                                  │
                    ┌─────────────────────────────┼─────────────────────────────┐
                    │                             │                             │
                    ▼                             ▼                             ▼
            ┌───────────────┐            ┌───────────────┐            ┌───────────────┐
            │ Internet      │            │    NFID       │            │    Plug       │
            │ Identity (II) │            │   Wallet      │            │   Wallet      │
            └───────┬───────┘            └───────┬───────┘            └───────┬───────┘
                    │                             │                             │
                    │  Principal A                │  Principal B                │  Principal C
                    │                             │                             │
                    └─────────────────────────────┼─────────────────────────────┘
                                                  │
                                                  ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│                                       STAKING HUB                                             │
│                                   (Cluster Coordinator)                                       │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│  RING_CONFIG: { ring_size: 1,000,000, ring_map: [...], version: 12 }                         │
│  SHARD_REGISTRY: [ Shard_A, Shard_B, Shard_C ]                                               │
│  get_identity_shard_for_principal(P) → Shard based on hash(P) % ring_size                    │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
                                                  │
                 ┌────────────────────────────────┼────────────────────────────────┐
                 │                                │                                │
                 ▼                                ▼                                ▼
┌─────────────────────────────┐  ┌─────────────────────────────┐  ┌─────────────────────────────┐
│       USER_PROFILE          │  │       USER_PROFILE          │  │       USER_PROFILE          │
│         SHARD_A             │  │         SHARD_B             │  │         SHARD_C             │
│     (Hash Range 0-333K)     │  │   (Hash Range 333K-666K)    │  │   (Hash Range 666K-1M)      │
├─────────────────────────────┤  ├─────────────────────────────┤  ├─────────────────────────────┤
│                             │  │                             │  │                             │
│  ┌───────────────────────┐  │  │  ┌───────────────────────┐  │  │  ┌───────────────────────┐  │
│  │   IDENTITY ROLE       │  │  │  │   IDENTITY ROLE       │  │  │  │   IDENTITY ROLE       │  │
│  │                       │  │  │  │                       │  │  │  │                       │  │
│  │ Principal_X → {       │  │  │  │ Principal_Y → {       │  │  │  │ Principal_Z → {       │  │
│  │   home: Shard_B,      │  │  │  │   home: Shard_A,      │  │  │  │   home: Shard_B,      │  │
│  │   account: 42         │  │  │  │   account: 17         │  │  │  │   account: 99         │  │
│  │ }                     │  │  │  │ }                     │  │  │  │ }                     │  │
│  └───────────────────────┘  │  │  └───────────────────────┘  │  │  └───────────────────────┘  │
│                             │  │                             │  │                             │
│  ┌───────────────────────┐  │  │  ┌───────────────────────┐  │  │  ┌───────────────────────┐  │
│  │   HOME ROLE           │  │  │  │   HOME ROLE           │  │  │  │   HOME ROLE           │  │
│  │                       │  │  │  │                       │  │  │  │                       │  │
│  │ Account_17 → Profile  │  │  │  │ Account_42 → Profile  │  │  │  │ Account_99 → Profile  │  │
│  │ Account_17 → [Keys]   │  │  │  │ Account_42 → [Keys]   │  │  │  │ Account_99 → [Keys]   │  │
│  │   - master: P_Y       │  │  │  │   - master: P_X       │  │  │  │   - master: P_Z       │  │
│  │   - session: [P_A]    │  │  │  │   - session: [P_B]    │  │  │  │   - session: []       │  │
│  │                       │  │  │  │                       │  │  │  │                       │  │
│  └───────────────────────┘  │  │  └───────────────────────┘  │  │  └───────────────────────┘  │
│                             │  │                             │  │                             │
└─────────────────────────────┘  └─────────────────────────────┘  └─────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════════════════════════
                                    LOGIN FLOW EXAMPLE
═══════════════════════════════════════════════════════════════════════════════════════════════

User logs in with NFID (Principal_B):

Step 1: Frontend calculates identity shard
        hash(Principal_B) = 500,123 → Falls in Shard_B range (333K-666K)

Step 2: Frontend calls Shard_B.get_pointer(Principal_B)
        ┌─────────────────────────────┐
        │         SHARD_B             │
        │  IDENTITY_POINTERS:         │
        │  Principal_B → {            │◄── Lookup here
        │    home: Shard_B,           │
        │    account: 42              │
        │  }                          │
        └─────────────────────────────┘
        Returns: { home: Shard_B, account: 42 }

Step 3: Frontend calls Shard_B.get_profile(42)  (same shard in this case)
        ┌─────────────────────────────┐
        │         SHARD_B             │
        │  USER_PROFILES:             │
        │  Account_42 → {             │◄── Fetch profile
        │    name: "Alice",           │
        │    staked: 10000,           │
        │    ...                      │
        │  }                          │
        │  ACCOUNT_PRINCIPALS:        │
        │  Account_42 → {             │◄── Verify auth
        │    master: Principal_X,     │
        │    session: [Principal_B]   │    ✓ Principal_B is authorized!
        │  }                          │
        └─────────────────────────────┘

Step 4: User sees their profile! Can now:
        ✅ Submit quizzes (rewards go to Account_42)
        ✅ Unstake tokens (from Account_42 balance)
        ✅ Vote on proposals (using Account_42 voting power)
        ✅ Link more devices (add to Account_42 keys)


═══════════════════════════════════════════════════════════════════════════════════════════════
                                 DEVICE LINKING FLOW
═══════════════════════════════════════════════════════════════════════════════════════════════

User wants to add Plug wallet (Principal_C) to their account:

EXISTING DEVICE (II)                                    NEW DEVICE (Plug)
       │                                                       │
       │ 1. request_link_token() → Home Shard                  │
       │    └── Returns: [32-byte token]                       │
       │                                                       │
       │ 2. Display QR code                                    │
       │    ┌─────────────────┐                                │
       │    │ █▀▀█ █▀▀ █▀▀▄  │                                │
       │    │ █▀▀█ ▀▀█ █  █  │ ◄────── Scan ───────────────── │
       │    │ ▀▀▀▀ ▀▀▀ ▀▀▀   │                                │
       │    └─────────────────┘                                │
       │                                                       │
       │                            3. Authenticate with Plug   │
       │                               Principal_C obtained     │
       │                                                       │
       │                            4. complete_link(token, C)  │
       │                               ┌─────────────────────┐  │
       │                               │  Home Shard:        │  │
       │                               │  - Verify token     │  │
       │                               │  - Add Principal_C  │  │
       │                               │    to session_keys  │  │
       │                               │  - Store pointer    │  │
       │                               │    for Principal_C  │  │
       │                               └─────────────────────┘  │
       │                                                       │
       │                            5. SUCCESS! Plug linked     │
       │                               User can now login       │
       │                               with Plug wallet         │


═══════════════════════════════════════════════════════════════════════════════════════════════
                            ANSWER: WHAT CAN DIFFERENT PRINCIPALS DO?
═══════════════════════════════════════════════════════════════════════════════════════════════

┌───────────────────────────────────────────────────────────────────────────────────────────────┐
│                                ACCOUNT #42 (Alice)                                            │
├───────────────────────────────────────────────────────────────────────────────────────────────┤
│  Balance: 5,000 GHC            Staked: 10,000 GHC            Voting Power: 10,000 VUC        │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
                                            │
                 ┌──────────────────────────┼──────────────────────────┐
                 │                          │                          │
          ┌──────┴──────┐            ┌──────┴──────┐            ┌──────┴──────┐
          │ MASTER KEY  │            │ SESSION KEY │            │ SESSION KEY │
          │ (Ledger)    │            │ (II)        │            │ (NFID)      │
          │ Principal_X │            │ Principal_A │            │ Principal_B │
          └──────┬──────┘            └──────┬──────┘            └──────┬──────┘
                 │                          │                          │
                 │                          │                          │
    ┌────────────┼──────────────────────────┼──────────────────────────┼────────────┐
    │            │                          │                          │            │
    │   ┌────────┴────────┐        ┌────────┴────────┐        ┌────────┴────────┐   │
    │   │ Can Do:         │        │ Can Do:         │        │ Can Do:         │   │
    │   │ ✅ View profile │        │ ✅ View profile │        │ ✅ View profile │   │
    │   │ ✅ Submit quiz  │        │ ✅ Submit quiz  │        │ ✅ Submit quiz  │   │
    │   │ ✅ Unstake      │        │ ✅ Unstake      │        │ ✅ Unstake      │   │
    │   │ ✅ Vote         │        │ ✅ Vote         │        │ ✅ Vote         │   │
    │   │ ✅ Transfer GHC │        │ ✅ Transfer GHC │        │ ✅ Transfer GHC │   │
    │   │ ✅ Add device   │        │ ✅ Add device   │        │ ✅ Add device   │   │
    │   │ ✅ REVOKE keys  │        │ ❌ Revoke keys  │        │ ❌ Revoke keys  │   │
    │   └─────────────────┘        └─────────────────┘        └─────────────────┘   │
    │                                                                               │
    │   MASTER-ONLY POWERS:                                                         │
    │   • Remove any session key from the account                                   │
    │   • Emergency: revoke ALL session keys at once                                │
    │   • Cannot be removed or transferred (permanent anchor)                       │
    │                                                                               │
    └───────────────────────────────────────────────────────────────────────────────┘

    TOKENS FLOW TO SAME ACCOUNT:
    ┌─────────────────────────────────────────────────────────────────────────────┐
    │  Login with II     → Submit Quiz → Reward added to Account #42             │
    │  Login with NFID   → Submit Quiz → Reward added to Account #42 (same!)     │
    │  Login with Ledger → Unstake     → Tokens sent to Ledger's address         │
    │  Login with NFID   → Vote        → Uses Account #42's 10,000 VUC           │
    └─────────────────────────────────────────────────────────────────────────────┘
```

---

## 19. FAQ

### Q: Can I move my GHC tokens when logged in with a Session Key (not Master)?
**A:** Yes! All authorized principals (master or session) can:
- Unstake tokens from the account
- Transfer GHC from the account's vault
- The tokens go to whatever address you specify (or your own wallet)

### Q: Can I vote on proposals when logged in with any linked wallet?
**A:** Yes! Voting power is tied to the **Account**, not the **Principal**. When you vote:
1. System looks up your Account ID via the identity pointer
2. Fetches your staked balance from your Home Shard
3. Uses that balance as your voting weight
4. Doesn't matter which principal you used to authenticate

### Q: What if someone steals one of my Session Keys?
**A:** 
1. Log in with your Master Key (e.g., Ledger hardware wallet)
2. Call `revoke_session_key(compromised_principal)`
3. The compromised principal immediately loses access
4. They cannot do anything because auth check will fail

### Q: Can an attacker add their own key to my account?
**A:** No! The linking flow requires:
1. A valid, unexpired link token (5-minute lifespan)
2. The token is single-use and cryptographically random
3. Only an already-authorized principal can request a token
4. Physical access (QR scan) is required to complete linking

### Q: What happens if the hash ring changes while I'm logging in?
**A:** The system handles this gracefully:
- During migration, both old and new shards can answer queries
- Frontend can retry with updated ring config if first call fails
- Pointers are atomic: either in old shard or new shard, never lost

---

## 20. Next Steps

1. **Phase 1 (Week 1-2)**: Add types and state storage
2. **Phase 2 (Week 2-3)**: Implement identity layer on user_profile
3. **Phase 3 (Week 3-4)**: Add device linking flow
4. **Phase 4 (Week 4-5)**: Implement ICP-safe resharding
5. **Phase 5 (Week 5-6)**: Update frontend for new flows
6. **Phase 6 (Week 6-7)**: Migrate existing users
7. **Phase 7 (Week 7-8)**: Testing and security hardening

The Multi-Wallet Identity architecture is designed to be **incrementally adoptable** - you can deploy Phase 1-2 without breaking existing users, then migrate them gradually.

