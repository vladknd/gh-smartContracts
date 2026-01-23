# Archive Canister Testing Guide

**Document Version**: 1.0  
**Last Updated**: January 2026

---

## Overview

This guide explains how to test the archive canister functionality and what each test verifies.

## Testing Components

### Components Under Test

| Component | Location | Description |
|-----------|----------|-------------|
| `archive_canister` | `src/archive_canister/` | Stores archived transaction history |
| `user_profile` | `src/user_profile/` | Source of transactions, integrates with archive |
| `staking_hub` | `src/staking_hub/` | Creates shard+archive pairs |

### Test Script

```bash
./scripts/test_archive.sh
```

---

## Test Cases Explained

### TEST 1: Archive Canister Deployment

**What We're Testing:**
- Archive canister can be deployed with a parent shard ID
- Initialization correctly stores the parent shard reference
- Initial stats show 0 entries

**How It Works:**
```bash
# Deploy with parent shard
dfx deploy archive_canister --argument "(record { 
    parent_shard_id = principal \"<user_profile_id>\" 
})"

# Verify parent is set
dfx canister call archive_canister get_parent_shard --query

# Check initial stats
dfx canister call archive_canister get_stats --query
```

**Expected Results:**
```
(
  record {
    total_entries = 0;
    capacity_percent = 0;
    user_count = 0;
    parent_shard = principal "...";
    next_archive = null;
  }
)
```

---

### TEST 2: Basic Archive Operations

**What We're Testing:**
- `receive_archive_batch` correctly stores transactions
- Authorization prevents unauthorized callers
- `get_archived_transactions` returns stored data

**How It Works:**
```bash
# Attempt to archive as unauthorized caller (should fail)
dfx canister call archive_canister receive_archive_batch "(
    principal \"<user>\",
    vec { 
        record { 
            timestamp = 1705000000000000000; 
            tx_type = 0;  # QuizReward
            amount = 100_000_000  # 1 token
        }
    }
)"
# Expected: "Unauthorized: Only parent shard can archive"

# Query archived transactions
dfx canister call archive_canister get_archived_transactions "(
    principal \"<user>\",
    0,    # offset
    10    # limit
)" --query
```

---

### TEST 3: User Profile Archive Integration

**What We're Testing:**
- User profile stores `archived_transaction_count`
- `get_transactions_page` returns correct pagination info
- `trigger_archive` moves old transactions to archive

**How It Works:**
```bash
# Register a user
dfx canister call user_profile register_user "(record {
    email = \"test@example.com\";
    name = \"Test User\";
    education = \"CS\";
    gender = \"Other\"
})"

# Get paginated transactions
dfx canister call user_profile get_transactions_page "(
    principal \"<user>\",
    0  # page number
)" --query
```

**Expected Results:**
```
(
  record {
    transactions = vec { ... };
    total_count = 0;
    local_count = 0;
    archived_count = 0;
    archive_canister_id = principal "...";
    source = "local";
  }
)
```

---

### TEST 4: Archive Triggering

**What We're Testing:**
- `trigger_archive` identifies users with excess transactions
- Transactions are sent to archive canister
- Local transactions are deleted and re-indexed

**Retention Policy:**
```
TRANSACTION_RETENTION_LIMIT = 100

If user has 150 transactions:
- Keep last 100 locally (indices 0-99)
- Archive oldest 50 to archive canister
```

**How It Works:**
```bash
# Trigger archiving
dfx canister call user_profile trigger_archive

# Expected response for 50 archived transactions
# (variant { Ok = 50 })
```

---

### TEST 5: Capacity Monitoring

**What We're Testing:**
- `get_stats` returns current capacity information
- `capacity_percent` correctly calculates usage

**How It Works:**
```bash
# Get archive stats
dfx canister call archive_canister get_stats --query
```

**Expected Results:**
```
(
  record {
    total_entries = 50;
    capacity_percent = 0;  # < 1% of 3 billion
    user_count = 1;
    parent_shard = principal "...";
    next_archive = null;
  }
)
```

---

## Manual Testing Scenarios

### Scenario A: Full Archive Flow

This scenario requires all canisters set up correctly:

```bash
# 1. Start dfx
dfx start --background --clean

# 2. Deploy all canisters
dfx deploy

# 3. Create a user and generate transactions
dfx identity new test_user
dfx identity use test_user

dfx canister call user_profile register_user "(record {
    email = \"test@test.com\";
    name = \"Test\";
    education = \"None\";
    gender = \"Other\"
})"

# 4. Submit quizzes (generates quiz reward transactions)
# Note: Requires learning_engine setup with quizzes
for i in {1..150}; do
    dfx canister call user_profile submit_quiz "(
        \"quiz_$i\",
        vec { 1; 2; 3 }  # answers
    )"
done

# 5. Check transaction count
dfx canister call user_profile get_transactions_page "(
    principal \"$(dfx identity get-principal)\",
    0
)" --query

# 6. Trigger archive
dfx identity use default
dfx canister call user_profile trigger_archive

# 7. Verify archive
dfx canister call archive_canister get_stats --query
dfx canister call archive_canister get_archived_transactions "(
    principal \"...\",
    0,
    10
)" --query
```

### Scenario B: Pagination Across Hot/Cold Data

Tests that frontend can access both local and archived transactions:

```bash
# After archiving (user has 100 local, 50 archived = 150 total)

# Page 0: transactions 0-19 → LOCAL
dfx canister call user_profile get_transactions_page "(user, 0)" --query
# source = "local", transactions = [...]

# Page 4: transactions 80-99 → LOCAL  
dfx canister call user_profile get_transactions_page "(user, 4)" --query
# source = "local", transactions = [...]

# Page 5: transactions 100-119 → ARCHIVE
dfx canister call user_profile get_transactions_page "(user, 5)" --query
# source = "archive", transactions = []
# Frontend should then query archive directly:
dfx canister call archive_canister get_archived_transactions "(user, 0, 20)" --query
```

---

## Key Verification Points

### ✅ Archive Correctly Initialized

```rust
// In archive_canister/src/lib.rs
assert!(PARENT_SHARD_ID.get() != Principal::anonymous());
assert!(TOTAL_ENTRY_COUNT.get() == 0);
```

### ✅ Authorization Working

```rust
// Only parent shard can write
if caller != parent_shard_id {
    return Err("Unauthorized");
}
```

### ✅ Transaction Counts Match

```
user_profile.transaction_count + user_profile.archived_transaction_count 
== archive.get_archived_count(user) + local_count
```

### ✅ Re-indexing Correct

After archiving 50 transactions:
- Old indices 0-49: deleted
- Old indices 50-149: become 0-99
- User's `transaction_count` decremented by 50
- User's `archived_transaction_count` incremented by 50

---

## Troubleshooting

### "Unauthorized: Only parent shard can archive"

This is expected when calling from dfx directly. The archive can only receive data from its parent shard (user_profile canister).

### "Archive canister not configured"

The user_profile's `ARCHIVE_CANISTER_ID` hasn't been set. In production, staking_hub sets this when creating the shard pair. For testing:

```bash
# This requires being the staking_hub principal
dfx canister call user_profile set_archive_canister "(
    principal \"<archive_canister_id>\"
)"
```

### Empty Archived Transactions

If `trigger_archive` returns Ok(0), check:
1. Does the user have > 100 transactions?
2. Is `ARCHIVE_CANISTER_ID` set?
3. Is the archive accessible?

---

## Architecture Reminder

```
┌─────────────────┐        ┌─────────────────┐
│  user_profile   │───────►│    archive      │
│   Shard 0       │        │    Shard 0      │
│                 │        │                 │
│ • Last 100 txns │        │ • All old txns  │
│ • trigger_arch()│        │ • Append-only   │
│ • get_page()    │        │ • Read queries  │
└─────────────────┘        └─────────────────┘
         │                          │
         │    Same user's data      │
         │    always together       │
         └──────────────────────────┘
```
