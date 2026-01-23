# Treasury Implementation Plan

## Executive Summary

This document outlines the implementation plan for the new GHC tokenomics with Treasury functionality. The Treasury manages Market Coins (MCs) with a sophisticated **Balance vs. Allowance** mechanism.

> **⚠️ ARCHITECTURE UPDATE (January 2026)**
>
> This document was originally written with the assumption that treasury functionality would be integrated into a single `operational_governance` canister. The system has since been **refactored** to use two separate canisters:
>
> - **`treasury_canister`**: Holds the 4.25B MC balance, manages allowance, executes MMCR, processes transfers
> - **`governance_canister`**: Manages proposals, voting, board members, and content governance
>
> Where this document references `operational_governance`, it should be interpreted as follows:
> - **Treasury state, MMCR, get_spendable_balance** → `treasury_canister`
> - **Proposals, voting, execute_proposal** → `governance_canister` (which calls `treasury_canister` for transfers)
>
> The core tokenomics and mechanisms described remain accurate.

---

## 1. New Tokenomics Overview

### 1.1 Total Supply: 9.5 Billion GHC

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         TOTAL SUPPLY: 9.5B GHC                               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────┐  ┌────────────────────────────────┐ │
│  │     UTILITY COINS (MUCs)            │  │     MARKET COINS (MCs)         │ │
│  │           4.75B                     │  │           4.75B                │ │
│  │                                     │  │                                │ │
│  │  Purpose: Staking Rewards           │  │  Distribution:                 │ │
│  │  Location: Staking Hub              │  │  ├─ Founders: 0.5B             │ │
│  │  (Mining via quizzes)               │  │  │  ├─ Founder 1: 0.35B        │ │
│  │                                     │  │  │  └─ Founder 2: 0.15B        │ │
│  │                                     │  │  │  (Time-locked: 10%/year)    │ │
│  │                                     │  │  │                             │ │
│  │                                     │  │  └─ Treasury: 4.25B            │ │
│  │                                     │  │     (Initial Allowance: 0.6B)  │ │
│  └─────────────────────────────────────┘  └────────────────────────────────┘ │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Token Partitions

| Partition | Amount | Purpose |
|-----------|--------|---------|
| **MUCs (Utility Coins)** | 4.75B | Staking rewards via quizzes |
| **MCs (Market Coins)** | 4.75B | Tradeable market tokens |

### 1.3 Market Coins Distribution

| Recipient | Amount | Notes |
|-----------|--------|-------|
| **Founder 1** | 0.35B | Time-locked (10% unlocks per year) |
| **Founder 2** | 0.15B | Time-locked (10% unlocks per year) |
| **Treasury** | 4.25B | Initial allowance: 0.6B |

---

## 2. Treasury Mechanics

### 2.1 Concepts

#### Treasury Balance
- **Definition**: Total volume of Market Coins held within the Treasury wallet
- **Behavior**: Static unless a physical transfer is executed
- **Does NOT decrease** when allowance increases

#### Treasury Allowance
- **Definition**: Dynamic spending limit ("liquid allocation")
- **Behavior**: Defines the portion of Balance authorized for immediate use
- **Initial Value**: 0.6B MC

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TREASURY STATE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     TREASURY BALANCE: 4.25B                          │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │              ALLOWANCE: 0.6B (Spendable)                     │   │   │
│   │   │                                                              │   │   │
│   │   └──────────────────────────────────────────────────────────────┘   │   │
│   │                                                                      │   │
│   │                    LOCKED: 3.65B (Not yet allocated)                 │   │
│   │                                                                      │   │
│   └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Balance vs Allowance Mechanics

```
┌────────────────────────────────────────────────────────────────────────────┐
│                   ALLOWANCE vs BALANCE STATE MACHINE                        │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   MONTHLY MARKET COIN RELEASE (MMCR)                                        │
│   ─────────────────────────────────────                                     │
│   • Allowance += MMCR_Amount (15.2M monthly, except final month)            │
│   • Balance: UNCHANGED (funds unlocked, not moved)                          │
│   • Executes: 1st of each month at 12:00 AM Eastern Time                    │
│   • Handles EST (UTC-5) and EDT (UTC-4) transitions automatically           │
│                                                                             │
│   APPROVED TRANSFER/SPENDING                                                │
│   ───────────────────────────                                               │
│   • Allowance -= Transfer_Amount                                            │
│   • Balance -= Transfer_Amount                                              │
│   • Physical tokens moved from Treasury wallet                              │
│                                                                             │
│   INVARIANTS                                                                │
│   ──────────                                                                │
│   • Allowance <= Balance (always)                                           │
│   • Balance <= Initial_Treasury_Deposit (4.25B) - Total_Transferred         │
│   • Cannot spend more than Allowance                                        │
│                                                                             │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. MMCR (Monthly Market Coin Release) Schedule

### 3.1 Release Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Monthly Amount** | 15.2M MC | Fixed for months 1-239 |
| **Release Schedule** | 240 months | 20 years total |
| **Execution Time** | 1st of month, 12:00 AM Eastern | UTC-5 (Standard) / UTC-4 (DST) |
| **Initial Allowance** | 0.6B MC | Available at genesis |
| **Remaining to Release** | 3.65B MC | Via MMCR over 240 months |

### 3.2 Release Math Verification

```
Initial Allowance:           600,000,000 MC
MMCR × 239 months:           15,200,000 × 239 = 3,632,800,000 MC
240th Month (Final):         4,250,000,000 - 600,000,000 - 3,632,800,000 = 17,200,000 MC
─────────────────────────────────────────────────────────────────────────────
Total Treasury Allowance:    4,250,000,000 MC ✓ (equals Treasury Balance)
```

### 3.3 MMCR Timeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MMCR RELEASE TIMELINE (20 YEARS)                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Genesis (Month 0)                                                           │
│  ─────────────────                                                           │
│  • Treasury Balance: 4.25B                                                   │
│  • Treasury Allowance: 0.6B (initial liquid allocation)                      │
│                                                                              │
│  Month 1-239 (1st of each month @ 12:00 AM Eastern)                         │
│  ─────────────────────────────────────────────────────                       │
│  • Allowance += 15.2M MC                                                     │
│  • Balance: UNCHANGED                                                        │
│                                                                              │
│  Month 240 (Final Release)                                                   │
│  ─────────────────────────                                                   │
│  • Allowance += 17.2M MC (adjusted to reach exactly 4.25B total)            │
│  • After this: Allowance == Balance (all funds liquid)                       │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Year 1    Year 5    Year 10   Year 15   Year 20                     │    │
│  │   │         │         │         │         │                         │    │
│  │   ▼         ▼         ▼         ▼         ▼                         │    │
│  │ ░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████ │    │
│  │ └──┘       └────────────────────────────────────────────────────┘   │    │
│  │ 0.6B                    3.65B released via MMCR                     │    │
│  │ Initial                                                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3.4 Eastern Time Zone Handling

The MMCR executes at **12:00 AM Eastern Time** on the 1st of each month:
- **Standard Time (EST)**: UTC-5 (November - March)
- **Daylight Saving Time (EDT)**: UTC-4 (March - November)

```rust
// Constants for Eastern timezone handling
const EST_OFFSET_HOURS: i64 = -5;  // UTC-5
const EDT_OFFSET_HOURS: i64 = -4;  // UTC-4

/// Determine if a given UTC timestamp is in EDT (Daylight Saving Time)
/// US DST: 2nd Sunday of March to 1st Sunday of November
fn is_eastern_dst(utc_timestamp_nanos: u64) -> bool {
    // Calculate month and day from timestamp
    // Return true if in DST period
    // (Full implementation in code)
}

/// Get next MMCR execution time in UTC nanoseconds
fn get_next_mmcr_time(current_utc_nanos: u64) -> u64 {
    // 1. Find 1st day of next month
    // 2. Determine if that date is in EST or EDT
    // 3. Return midnight Eastern as UTC nanoseconds
}
```

---

## 4. Founder Time-Lock Mechanism

### 4.1 Vesting Schedule

| Year | Unlock % | Founder 1 Unlocked | Founder 2 Unlocked |
|------|----------|-------------------|-------------------|
| 0 | 0% | 0 | 0 |
| 1 | 10% | 0.035B | 0.015B |
| 2 | 20% | 0.07B | 0.03B |
| 3 | 30% | 0.105B | 0.045B |
| 4 | 40% | 0.14B | 0.06B |
| 5 | 50% | 0.175B | 0.075B |
| ... | ... | ... | ... |
| 10 | 100% | 0.35B | 0.15B |

### 4.2 Time-Lock Implementation

Create a `founder_vesting` canister that:
- Holds all 0.5B founder tokens in custody
- Tracks vesting schedule per founder
- Releases 10% of initial allocation per year
- Founders call `claim_vested()` to withdraw unlocked tokens

---

## 5. Implementation Changes

### 5.1 Canister Strategy

**Use existing `operational_governance` canister as the Treasury:**

| Approach | Benefit |
|----------|---------|
| Single canister | Simpler architecture |
| Already holds tokens | No migration needed |
| Already has proposals | Reuse governance logic |
| Already has ledger integration | Less code to write |

### 5.2 Files to Modify

| File | Changes |
|------|---------|
| `operational_governance/src/lib.rs` | Add Treasury state (Balance/Allowance), MMCR logic |
| `staking_hub/src/lib.rs` | Update MAX_SUPPLY from 4.2B to 4.75B |
| `scripts/deploy.sh` | Update initial token distribution |
| `docs/ARCHITECTURE.md` | Update tokenomics documentation |

### 5.3 New Canister to Create

| Canister | Purpose |
|----------|---------|
| `founder_vesting` | Time-locked founder token management |

---

## 6. Detailed Implementation Steps

### Phase 1: Update `operational_governance` to be Treasury (Week 1-2)

#### Step 1.1: Add Treasury State to `operational_governance`

```rust
// Add to src/operational_governance/src/lib.rs

// ============================================================================
// TREASURY CONSTANTS
// ============================================================================

/// Initial treasury balance: 4.25B MC (in e8s)
const INITIAL_TREASURY_BALANCE: u64 = 4_250_000_000 * 100_000_000;

/// Initial treasury allowance: 0.6B MC (in e8s)
const INITIAL_TREASURY_ALLOWANCE: u64 = 600_000_000 * 100_000_000;

/// Monthly Market Coin Release: 15.2M MC (in e8s)
const MMCR_AMOUNT: u64 = 15_200_000 * 100_000_000;

/// Total MMCR releases over 20 years
const TOTAL_MMCR_RELEASES: u64 = 240;

/// Final month adjusted amount: 17.2M MC (in e8s)
const FINAL_MMCR_AMOUNT: u64 = 17_200_000 * 100_000_000;

// ============================================================================
// TREASURY STATE
// ============================================================================

/// Treasury state - tracks balance and allowance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    /// Total MC balance held (decreases only on transfers)
    pub balance: u64,
    
    /// Current spending allowance (liquid allocation)
    pub allowance: u64,
    
    /// Total amount transferred out historically
    pub total_transferred: u64,
    
    /// Number of MMCR executions completed (0-240)
    pub mmcr_count: u64,
    
    /// Timestamp of last MMCR execution (nanoseconds)
    pub last_mmcr_timestamp: u64,
    
    /// Genesis timestamp (when Treasury was initialized)
    pub genesis_timestamp: u64,
}

impl Default for TreasuryState {
    fn default() -> Self {
        Self {
            balance: INITIAL_TREASURY_BALANCE,
            allowance: INITIAL_TREASURY_ALLOWANCE,
            total_transferred: 0,
            mmcr_count: 0,
            last_mmcr_timestamp: 0,
            genesis_timestamp: 0,
        }
    }
}
```

#### Step 1.2: Add Treasury Storage

```rust
// Add to thread_local! block in operational_governance

/// Treasury state tracking balance and allowance
static TREASURY_STATE: RefCell<StableCell<TreasuryState, Memory>> = RefCell::new(
    StableCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
        TreasuryState::default()
    ).unwrap()
);
```

#### Step 1.3: Treasury Query Functions

```rust
/// Get current treasury state
#[query]
fn get_treasury_state() -> TreasuryState {
    TREASURY_STATE.with(|s| s.borrow().get().clone())
}

/// Get spendable balance (allowance)
#[query]
fn get_spendable_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().allowance)
}

/// Get treasury balance (total held)
#[query]
fn get_treasury_balance() -> u64 {
    TREASURY_STATE.with(|s| s.borrow().get().balance)
}

/// Get MMCR status
#[query]
fn get_mmcr_status() -> (u64, u64, u64) {
    TREASURY_STATE.with(|s| {
        let state = s.borrow().get().clone();
        (state.mmcr_count, state.last_mmcr_timestamp, TOTAL_MMCR_RELEASES - state.mmcr_count)
    })
}
```

#### Step 1.4: MMCR Execution Logic

```rust
/// Execute Monthly Market Coin Release
/// Called automatically by timer or manually triggered
/// Increases allowance without decreasing balance
#[update]
fn execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Check if all releases completed
        if state.mmcr_count >= TOTAL_MMCR_RELEASES {
            return Err("All MMCR releases completed".to_string());
        }
        
        // Check if it's time for next release (1st of month at midnight Eastern)
        // For MVP: check if 28+ days since last release
        let min_interval = 28 * 24 * 60 * 60 * 1_000_000_000u64; // 28 days in nanos
        
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + min_interval {
            return Err("Too early for next MMCR".to_string());
        }
        
        // Determine release amount (final month is adjusted)
        let release_amount = if state.mmcr_count == TOTAL_MMCR_RELEASES - 1 {
            FINAL_MMCR_AMOUNT
        } else {
            MMCR_AMOUNT
        };
        
        // Increase allowance (cannot exceed balance)
        let new_allowance = (state.allowance + release_amount).min(state.balance);
        let actual_release = new_allowance - state.allowance;
        
        state.allowance = new_allowance;
        state.mmcr_count += 1;
        state.last_mmcr_timestamp = current_time;
        
        cell.set(state).expect("Failed to update treasury state");
        
        Ok(actual_release)
    })
}
```

#### Step 1.5: Update `execute_proposal` to Check Allowance

```rust
// Modify existing execute_proposal function

#[update]
async fn execute_proposal(proposal_id: u64) -> Result<(), String> {
    let proposal = PROPOSALS.with(|p| p.borrow().get(&proposal_id))
        .ok_or("Proposal not found")?;

    if proposal.executed {
        return Err("Already executed".to_string());
    }

    if proposal.votes_yes <= proposal.votes_no {
        return Err("Proposal not approved".to_string());
    }

    // NEW: Check treasury allowance before transfer
    let current_allowance = TREASURY_STATE.with(|s| s.borrow().get().allowance);
    if proposal.amount > current_allowance {
        return Err(format!(
            "Insufficient treasury allowance. Requested: {}, Available: {}",
            proposal.amount, current_allowance
        ));
    }

    // Execute transfer (existing code)
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: proposal.recipient, subaccount: None },
        amount: Nat::from(proposal.amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Rejection code: {:?}, message: {}", code, msg))?;

    match result {
        Ok(_) => {
            // Mark executed
            PROPOSALS.with(|p| {
                let mut map = p.borrow_mut();
                if let Some(mut prop) = map.get(&proposal_id) {
                    prop.executed = true;
                    map.insert(proposal_id, prop);
                }
            });
            
            // NEW: Update treasury state (decrease both balance and allowance)
            TREASURY_STATE.with(|s| {
                let mut cell = s.borrow_mut();
                let mut state = cell.get().clone();
                state.balance = state.balance.saturating_sub(proposal.amount);
                state.allowance = state.allowance.saturating_sub(proposal.amount);
                state.total_transferred += proposal.amount;
                cell.set(state).expect("Failed to update treasury state");
            });
            
            TOTAL_SPENT.with(|t| {
                let mut cell = t.borrow_mut();
                let current = *cell.get();
                cell.set(current + proposal.amount).expect("Failed to update total spent");
            });

            Ok(())
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}
```

### Phase 2: Founder Vesting Canister (Week 2-3)

#### Step 2.1: Create Canister Scaffold

```bash
# Create new canister directory
mkdir -p src/founder_vesting/src
touch src/founder_vesting/Cargo.toml
touch src/founder_vesting/src/lib.rs
touch src/founder_vesting/founder_vesting.did
```

#### Step 2.2: Vesting Data Structures

```rust
// src/founder_vesting/src/lib.rs

use ic_cdk::{init, query, update};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use std::cell::RefCell;
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// CONSTANTS
// ============================================================================

/// One year in nanoseconds
const YEAR_IN_NANOS: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;

/// Annual unlock: 10% = 1000 basis points
const ANNUAL_UNLOCK_BPS: u16 = 1000;

/// Founder 1 allocation: 0.35B MC (in e8s)
const FOUNDER_1_ALLOCATION: u64 = 350_000_000 * 100_000_000;

/// Founder 2 allocation: 0.15B MC (in e8s)
const FOUNDER_2_ALLOCATION: u64 = 150_000_000 * 100_000_000;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Vesting schedule for a founder
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VestingSchedule {
    /// Founder principal ID
    pub founder: Principal,
    
    /// Total tokens allocated (never changes)
    pub total_allocation: u64,
    
    /// Tokens already claimed
    pub claimed: u64,
    
    /// Vesting start timestamp (genesis)
    pub vesting_start: u64,
}

impl VestingSchedule {
    /// Calculate currently vested (unlocked) tokens
    pub fn vested_amount(&self, current_time: u64) -> u64 {
        if current_time <= self.vesting_start {
            return 0;
        }
        
        let elapsed_nanos = current_time - self.vesting_start;
        let elapsed_years = elapsed_nanos / YEAR_IN_NANOS;
        
        // 10% per year, max 100%
        let unlock_bps = (elapsed_years * ANNUAL_UNLOCK_BPS as u64).min(10000);
        
        (self.total_allocation * unlock_bps) / 10000
    }
    
    /// Calculate claimable tokens
    pub fn claimable(&self, current_time: u64) -> u64 {
        self.vested_amount(current_time).saturating_sub(self.claimed)
    }
}

/// Public vesting status for queries
#[derive(CandidType, Clone, Debug)]
pub struct VestingStatus {
    pub founder: Principal,
    pub total_allocation: u64,
    pub vested: u64,
    pub claimed: u64,
    pub claimable: u64,
    pub years_elapsed: u64,
    pub unlock_percentage: u64,
}
```

#### Step 2.3: Vesting Functions

```rust
/// Claim vested tokens
/// Called by founders to withdraw unlocked tokens
#[update]
async fn claim_vested() -> Result<u64, String> {
    let caller = ic_cdk::caller();
    let current_time = ic_cdk::api::time();
    
    // Find founder's vesting schedule
    let schedule = VESTING_SCHEDULES.with(|v| v.borrow().get(&caller))
        .ok_or("Caller is not a registered founder")?;
    
    // Calculate claimable amount
    let claimable = schedule.claimable(current_time);
    
    if claimable == 0 {
        return Err("No tokens available to claim".to_string());
    }
    
    // Execute ICRC-1 transfer to founder
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: caller, subaccount: None },
        amount: Nat::from(claimable),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer failed: {:?} {}", code, msg))?;

    match result {
        Ok(_) => {
            // Update claimed amount
            VESTING_SCHEDULES.with(|v| {
                let mut schedules = v.borrow_mut();
                if let Some(mut sched) = schedules.get(&caller) {
                    sched.claimed += claimable;
                    schedules.insert(caller, sched);
                }
            });
            
            Ok(claimable)
        }
        Err(e) => Err(format!("Ledger transfer error: {:?}", e)),
    }
}

/// Query vesting status for a founder
#[query]
fn get_vesting_status(founder: Principal) -> Option<VestingStatus> {
    let current_time = ic_cdk::api::time();
    
    VESTING_SCHEDULES.with(|v| {
        v.borrow().get(&founder).map(|schedule| {
            let years_elapsed = if current_time > schedule.vesting_start {
                (current_time - schedule.vesting_start) / YEAR_IN_NANOS
            } else {
                0
            };
            
            VestingStatus {
                founder: schedule.founder,
                total_allocation: schedule.total_allocation,
                vested: schedule.vested_amount(current_time),
                claimed: schedule.claimed,
                claimable: schedule.claimable(current_time),
                years_elapsed,
                unlock_percentage: (years_elapsed * 10).min(100),
            }
        })
    })
}

/// Query all vesting schedules (for admin/dashboard)
#[query]
fn get_all_vesting_schedules() -> Vec<VestingStatus> {
    let current_time = ic_cdk::api::time();
    
    VESTING_SCHEDULES.with(|v| {
        v.borrow().iter().map(|(_, schedule)| {
            let years_elapsed = if current_time > schedule.vesting_start {
                (current_time - schedule.vesting_start) / YEAR_IN_NANOS
            } else {
                0
            };
            
            VestingStatus {
                founder: schedule.founder,
                total_allocation: schedule.total_allocation,
                vested: schedule.vested_amount(current_time),
                claimed: schedule.claimed,
                claimable: schedule.claimable(current_time),
                years_elapsed,
                unlock_percentage: (years_elapsed * 10).min(100),
            }
        }).collect()
    })
}
```

### Phase 3: Update Staking Hub & Deploy Script (Week 3)

#### Step 3.1: Update Staking Hub MAX_SUPPLY

```rust
// In staking_hub/src/lib.rs, change line 21:

// OLD:
const MAX_SUPPLY: u64 = 4_200_000_000 * 100_000_000; // 4.2B Tokens

// NEW:
const MAX_SUPPLY: u64 = 4_750_000_000 * 100_000_000; // 4.75B MUC Tokens
```

#### Step 3.2: Update Deploy Script

```bash
# Updated deploy.sh token distribution

# ============================================================================
# NEW TOKENOMICS (9.5B Total Supply)
# ============================================================================
#
# Market Coins (4.75B MC):
#   - Founder Vesting: 0.5B (holds both founders' allocations)
#     - Founder 1: 0.35B (10%/year vesting)
#     - Founder 2: 0.15B (10%/year vesting)
#   - Treasury (op_gov): 4.25B (initial allowance: 0.6B)
#
# Utility Coins (4.75B MUC):
#   - Staking Hub: 4.75B (for mining rewards)
#
# ============================================================================

FOUNDER_VESTING_AMT=$(to_e8s 500000000)  # 0.5B -> Founder Vesting Canister
TREASURY_AMT=$(to_e8s 4250000000)        # 4.25B -> Op Gov (Treasury)
HUB_AMT=$(to_e8s 4750000000)             # 4.75B -> Staking Hub

INIT_ARGS="(variant { Init = record {
     token_symbol = \"GHC\";
     token_name = \"GreenHero Coin\";
     decimals = opt 8;
     minting_account = record { owner = principal \"$DEFAULT\"; subaccount = null; };
     transfer_fee = 10_000;
     metadata = vec {};
     initial_balances = vec {
         record { record { owner = principal \"$FOUNDER_VESTING_ID\"; subaccount = null; }; $FOUNDER_VESTING_AMT };
         record { record { owner = principal \"$OP_GOV_ID\"; subaccount = null; }; $TREASURY_AMT };
         record { record { owner = principal \"$STAKING_HUB_ID\"; subaccount = null; }; $HUB_AMT };
     };
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 2000;
         controller_id = principal \"$DEFAULT\";
     };
 }})"
```

---

## 7. Architecture Diagram (Updated)

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│                           GHC TOKEN DISTRIBUTION (9.5B)                              │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────┐    │
│   │                              GHC LEDGER (ICRC-1)                             │    │
│   │                              Total: 9.5B GHC                                 │    │
│   └────────────────────────────────────┬────────────────────────────────────────┘    │
│                                        │                                             │
│                           ┌────────────┼────────────┐                               │
│                           ▼            ▼            ▼                               │
│   ┌─────────────────────────┐ ┌─────────────────────────┐ ┌──────────────────────┐  │
│   │      STAKING_HUB        │ │   OPERATIONAL_GOV       │ │  FOUNDER_VESTING     │  │
│   │                         │ │     (Treasury)          │ │                      │  │
│   │  Balance: 4.75B MUC     │ │  Balance: 4.25B MC      │ │  Balance: 0.5B MC    │  │
│   │                         │ │  Allowance: 0.6B MC     │ │  ├─ F1: 0.35B        │  │
│   │  Purpose: Mining        │ │                         │ │  └─ F2: 0.15B        │  │
│   │  rewards via quizzes    │ │  MMCR: 15.2M/month      │ │  (10%/year unlock)   │  │
│   │                         │ │  Duration: 240 months   │ │                      │  │
│   └────────────┬────────────┘ └────────────┬────────────┘ └──────────┬───────────┘  │
│                │                           │                         │              │
│                ▼                           ▼                         ▼              │
│   ┌─────────────────────────┐ ┌─────────────────────────┐ ┌──────────────────────┐  │
│   │    USER_PROFILE         │ │   GOVERNANCE            │ │  FOUNDER WALLETS     │  │
│   │      SHARDS             │ │   PROPOSALS             │ │  (After Claim)       │  │
│   │  (User staked balance)  │ │  (Spending requests)    │ │                      │  │
│   └─────────────────────────┘ └─────────────────────────┘ └──────────────────────┘  │
│                                                                                       │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Implementation Checklist

### Phase 1: Treasury Logic in `operational_governance` (Week 1-2)
- [ ] Add TreasuryState struct with balance/allowance
- [ ] Add TREASURY_STATE storage (MemoryId 6)
- [ ] Implement `get_treasury_state()` query
- [ ] Implement `get_spendable_balance()` query
- [ ] Implement `get_treasury_balance()` query
- [ ] Implement `execute_mmcr()` update function
- [ ] Modify `execute_proposal()` to check allowance and update treasury state
- [ ] Add MMCR timer (1st of each month)
- [ ] Update initialization to set genesis timestamp

### Phase 2: Founder Vesting Canister (Week 2-3)
- [ ] Create `founder_vesting` canister scaffold
- [ ] Add Cargo.toml and dfx.json entry
- [ ] Implement VestingSchedule struct
- [ ] Implement `claim_vested()` function
- [ ] Implement `get_vesting_status()` query
- [ ] Implement `get_all_vesting_schedules()` query
- [ ] Initialize with founder principals and allocations

### Phase 3: Integration (Week 3)
- [ ] Update `staking_hub` MAX_SUPPLY to 4.75B
- [ ] Update `deploy.sh` for new token distribution
- [ ] Update dfx.json with new canister
- [ ] Test full deployment flow

### Phase 4: Documentation & Testing (Week 4)
- [ ] Unit tests for treasury logic
- [ ] Unit tests for vesting calculations
- [ ] Integration tests for MMCR
- [ ] Integration tests for founder claims
- [ ] Update ARCHITECTURE.md
- [ ] Update README.md

---

## 9. Security Considerations

### 9.1 Access Control
| Function | Access |
|----------|--------|
| `execute_mmcr` | Anyone (idempotent, time-gated) |
| `execute_proposal` | Anyone (requires governance approval) |
| `claim_vested` | Founders only (checks caller) |
| `get_*` queries | Public |

### 9.2 Invariants to Enforce
```
allowance <= balance                         // Can't spend more than we have
balance >= 0                                 // Can't go negative
allowance >= 0                               // Can't go negative
total_transferred + balance == initial       // Conservation of tokens
mmcr_count <= 240                            // Max releases
claimed <= vested_amount                     // Can't claim more than vested
```

### 9.3 Rate Limiting
- MMCR: Once per month (minimum 28 days between releases)
- Governance proposals: Voting periods
- Founder claims: Rate-limited by vesting schedule (10%/year)

---

## 10. Next Steps

1. **Review this plan** - Any changes to the tokenomics or mechanics?
2. **Confirm founder principals** - We need the actual Principal IDs for founders
3. **Start implementation** - Begin with Phase 1 (Treasury in `operational_governance`)

Would you like me to start implementing Phase 1 now?

