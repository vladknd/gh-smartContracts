# DEX Integration Guide

> **Last Updated:** December 2024  
> **Purpose:** How to add/remove DEX liquidity through governance

---

## Table of Contents

1. [Overview](#1-overview)
2. [DEX Liquidity Basics](#2-dex-liquidity-basics)
3. [Governance-Controlled Liquidity](#3-governance-controlled-liquidity)
4. [Adding Liquidity](#4-adding-liquidity)
5. [Removing Liquidity](#5-removing-liquidity)
6. [ICP DEX Options](#6-icp-dex-options)
7. [ICPSwap Integration](#7-icpswap-integration)
8. [Security Considerations](#8-security-considerations)
9. [Implementation Guide](#9-implementation-guide)

---

## 1. Overview

The GreenHero Treasury (4.25B GHC) can provide liquidity to decentralized exchanges (DEXes) through governance proposals. This:

- **Creates trading markets** for GHC token
- **Earns trading fees** for the treasury
- **Establishes price discovery** mechanism
- **Enables token utility** for holders

All liquidity operations require governance approval to ensure democratic control over treasury funds.

---

## 2. DEX Liquidity Basics

### What is a Liquidity Pool?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    LIQUIDITY POOL CONCEPT                                    │
└─────────────────────────────────────────────────────────────────────────────┘

  A pool of TWO tokens that enables trading between them using an
  Automated Market Maker (AMM) algorithm.

      ┌──────────────────────────────────────────┐
      │           GHC / ICP POOL                 │
      │                                          │
      │    ┌──────────────┐  ┌──────────────┐   │
      │    │     GHC      │  │     ICP      │   │
      │    │  1,000,000   │  │   10,000     │   │
      │    │   tokens     │  │   tokens     │   │
      │    └──────────────┘  └──────────────┘   │
      │                                          │
      │    Constant Product: GHC × ICP = k      │
      │    1,000,000 × 10,000 = 10,000,000,000  │
      │                                          │
      │    Price: 1 GHC = 0.01 ICP              │
      │           1 ICP = 100 GHC               │
      │                                          │
      └──────────────────────────────────────────┘
                        │
                        │ Traders swap here
                        ▼
            User sells 1000 GHC → Gets ~9.99 ICP
            User sells 100 ICP  → Gets ~9,900 GHC
            
            (Actual amounts depend on pool size and slippage)
```

### LP (Liquidity Provider) Tokens

When you add liquidity, you receive LP tokens that represent your share of the pool.

```
                    ADD LIQUIDITY
                    ═════════════
                    
  Treasury provides:                  Treasury receives:
  ┌─────────────────────┐             ┌─────────────────────┐
  │    1,000,000 GHC    │             │                     │
  │         +           │  ────────►  │   100,000 LP Tokens │
  │     10,000 ICP      │             │   (1% of pool)      │
  │                     │             │                     │
  └─────────────────────┘             └─────────────────────┘
  
  
                    REMOVE LIQUIDITY
                    ════════════════
                    
  Treasury provides:                  Treasury receives:
  ┌─────────────────────┐             ┌─────────────────────┐
  │                     │             │   ~1,050,000 GHC    │
  │   100,000 LP Tokens │  ────────►  │         +           │
  │                     │             │    ~9,500 ICP       │
  │                     │             │   (depends on       │
  │                     │             │    price movement)  │
  └─────────────────────┘             └─────────────────────┘
```

### Impermanent Loss

When you provide liquidity and the price ratio changes, you may experience "impermanent loss" compared to just holding the tokens.

```
  Example: Added 1M GHC + 10K ICP when 1 GHC = 0.01 ICP
  
  If GHC price doubles (1 GHC = 0.02 ICP):
  
  ┌─────────────────────────────────────────────────────────┐
  │  Just Holding          │  Providing Liquidity          │
  ├─────────────────────────────────────────────────────────┤
  │  1M GHC = 20,000 ICP   │  ~707K GHC + ~14.1K ICP       │
  │  10K ICP = 10,000 ICP  │  Total value: ~28,200 ICP     │
  │  Total: 30,000 ICP     │                               │
  │                        │  Impermanent Loss: ~6%        │
  └─────────────────────────────────────────────────────────┘
  
  Note: Trading fees earned may offset or exceed impermanent loss
```

---

## 3. Governance-Controlled Liquidity

### Why Governance Control?

| Reason | Explanation |
|--------|-------------|
| **Democratic Oversight** | Community decides how treasury is used |
| **Transparency** | All liquidity actions are public proposals |
| **Security** | Multiple stakeholders must approve large allocations |
| **Accountability** | Audit trail for all treasury movements |

### Proposal Types

```rust
// Adding liquidity to a DEX pool
AddDexLiquidity {
    dex_canister: Principal,     // DEX router/pool canister ID
    pool_id: String,             // Pool identifier (e.g., "GHC/ICP")
    ghc_amount: u64,             // Amount of GHC to add
    paired_token: Principal,      // Other token canister (e.g., ICP ledger)
    paired_amount: u64,          // Amount of paired token
    min_lp_tokens: u64,          // Minimum LP tokens to accept (slippage protection)
}

// Removing liquidity from a DEX pool
RemoveDexLiquidity {
    dex_canister: Principal,     // DEX router/pool canister ID
    pool_id: String,             // Pool identifier
    lp_amount: u64,              // Amount of LP tokens to redeem
    min_token0_return: u64,      // Minimum token0 to receive
    min_token1_return: u64,      // Minimum token1 to receive
}
```

---

## 4. Adding Liquidity

### Step-by-Step Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ADD LIQUIDITY GOVERNANCE FLOW                             │
└─────────────────────────────────────────────────────────────────────────────┘

  STEP 1: Create Proposal
  ═══════════════════════
  
  Proposer calls:
  
      operational_governance.create_proposal(
          AddDexLiquidity {
              dex_canister: principal "mxzaz-hqaaa-aaaar-qaada-cai",  // ICPSwap
              pool_id: "GHC_ICP",
              ghc_amount: 1_000_000_00000000,      // 1M GHC (with decimals)
              paired_token: principal "ryjl3-tyaaa-aaaaa-aaaba-cai",  // ICP ledger
              paired_amount: 10_000_00000000,      // 10K ICP (with decimals)
              min_lp_tokens: 99_000_00000000,      // Accept 1% slippage
          },
          "Add 1M GHC to ICPSwap",
          "Initial liquidity provision to establish GHC/ICP trading pair"
      )
  
  
  STEP 2: Voting Period (7 days)
  ══════════════════════════════
  
      Founders (VUC) + Stakers vote
      
      Requirements:
      • Quorum: 5% of total voting power
      • Threshold: 60% approval
  
  
  STEP 3: Timelock (2 days)
  ═════════════════════════
  
      If passed, proposal enters 2-day waiting period
      Community can exit positions if they disagree
  
  
  STEP 4: Execution
  ═════════════════
  
      Anyone can call execute_proposal(proposal_id)
      
      Governance canister performs:
      
      a) Approve DEX to spend GHC:
         ghc_ledger.icrc2_approve({
             spender: dex_canister,
             amount: ghc_amount + fee,
         })
      
      b) Approve DEX to spend ICP (if treasury holds ICP):
         icp_ledger.icrc2_approve({
             spender: dex_canister,
             amount: icp_amount + fee,
         })
         
         Or transfer ICP directly if DEX requires:
         icp_ledger.icrc1_transfer({
             to: dex_canister,
             amount: icp_amount,
         })
      
      c) Call DEX to add liquidity:
         dex_canister.addLiquidity({
             token0: ghc_ledger,
             token1: icp_ledger,
             amount0: ghc_amount,
             amount1: icp_amount,
             amount0_min: ghc_amount * 99 / 100,
             amount1_min: icp_amount * 99 / 100,
             to: operational_governance,  // LP tokens go here
             deadline: now + 10_minutes,
         })
      
      d) Store LP position:
         LIQUIDITY_POSITIONS.insert(Position {
             pool_id,
             lp_token_canister,
             lp_amount: received_lp_tokens,
             added_at: now,
             ghc_added: ghc_amount,
             paired_added: icp_amount,
         })
```

---

## 5. Removing Liquidity

### Step-by-Step Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REMOVE LIQUIDITY GOVERNANCE FLOW                          │
└─────────────────────────────────────────────────────────────────────────────┘

  STEP 1: Create Proposal
  ═══════════════════════
  
      operational_governance.create_proposal(
          RemoveDexLiquidity {
              dex_canister: principal "mxzaz-hqaaa-aaaar-qaada-cai",
              pool_id: "GHC_ICP",
              lp_amount: 50_000_00000000,         // Redeem 50K LP tokens
              min_token0_return: 450_000_00000000, // Expect ~450K GHC
              min_token1_return: 4_500_00000000,   // Expect ~4.5K ICP
          },
          "Remove 50% liquidity from ICPSwap",
          "Rebalancing due to impermanent loss. Will move to Sonic."
      )
  
  
  STEP 2-3: Voting + Timelock (same as adding)
  ════════════════════════════════════════════
  
  
  STEP 4: Execution
  ═════════════════
  
      Governance canister performs:
      
      a) Approve DEX to spend LP tokens:
         lp_token.icrc2_approve({
             spender: dex_canister,
             amount: lp_amount,
         })
      
      b) Call DEX to remove liquidity:
         dex_canister.removeLiquidity({
             token0: ghc_ledger,
             token1: icp_ledger,
             lp_amount: lp_amount,
             amount0_min: min_token0_return,
             amount1_min: min_token1_return,
             to: operational_governance,
             deadline: now + 10_minutes,
         })
      
      c) Update position tracking:
         LIQUIDITY_POSITIONS.remove(pool_id)
         
         Or update remaining amount if partial removal
      
      d) Optionally transfer recovered tokens to treasury wallet
```

### Reasons to Remove Liquidity

| Reason | Description |
|--------|-------------|
| **Impermanent Loss** | Price diverged significantly, better to hold |
| **Rebalancing** | Move to different pool or DEX with better rates |
| **Market Conditions** | Bearish market, reduce exposure |
| **DEX Upgrade** | Migrate to newer version of DEX |
| **Security Concern** | Vulnerability discovered in DEX |
| **Strategic Shift** | Project direction changed |
| **Fee Collection** | Some DEXes require removal to claim fees |

---

## 6. ICP DEX Options

### Major DEXes on Internet Computer

| DEX | Type | Interface | TVL | Notes |
|-----|------|-----------|-----|-------|
| **ICPSwap** | AMM (V2/V3) | ICRC-2 | Highest | Most recommended |
| **Sonic** | AMM | DIP20 + ICRC-1 | Medium | Simple API |
| **ICDex** | Order Book | Custom | Medium | Not AMM, more complex |
| **KongSwap** | AMM | ICRC-2 | Growing | Newer option |

### Recommendation

For GHC, we recommend **ICPSwap** because:
- ✅ Largest TVL and user base
- ✅ ICRC-2 compatible (same as GHC)
- ✅ Well-documented APIs
- ✅ Multiple pool types (V2 constant product, V3 concentrated)
- ✅ Established security track record

---

## 7. ICPSwap Integration

### ICPSwap Canister IDs (Mainnet)

```
ICPSwap Router:     mxzaz-hqaaa-aaaar-qaada-cai
ICPSwap Factory:    4mmnk-kiaaa-aaaag-qbllq-cai
ICPSwap Position:   7n4mf-myaaa-aaaag-qtaaq-cai
```

### Interface (Simplified)

```candid
// ICPSwap V2 Pool Interface
type AddLiquidityArgs = record {
    token0: principal;
    token1: principal;
    amount0Desired: nat;
    amount1Desired: nat;
    amount0Min: nat;
    amount1Min: nat;
};

type RemoveLiquidityArgs = record {
    token0: principal;
    token1: principal;
    liquidity: nat;
    amount0Min: nat;
    amount1Min: nat;
};

service : {
    // Add liquidity to a pool
    "deposit": (AddLiquidityArgs) -> (variant { ok: nat; err: text });
    
    // Remove liquidity from a pool
    "withdraw": (RemoveLiquidityArgs) -> (variant { ok: record { nat; nat }; err: text });
    
    // Query pool info
    "getPool": (principal, principal) -> (opt Pool) query;
    
    // Get LP token balance
    "getUserLiquidity": (principal) -> (nat) query;
}
```

### Rust Integration Code

```rust
// Types for ICPSwap interaction
#[derive(CandidType, Deserialize)]
struct AddLiquidityArgs {
    token0: Principal,
    token1: Principal,
    amount0_desired: Nat,
    amount1_desired: Nat,
    amount0_min: Nat,
    amount1_min: Nat,
}

#[derive(CandidType, Deserialize)]
struct RemoveLiquidityArgs {
    token0: Principal,
    token1: Principal,
    liquidity: Nat,
    amount0_min: Nat,
    amount1_min: Nat,
}

// Execute add liquidity
async fn execute_add_liquidity(
    dex_canister: Principal,
    ghc_ledger: Principal,
    paired_ledger: Principal,
    ghc_amount: u64,
    paired_amount: u64,
) -> Result<u64, String> {
    // 1. Approve GHC spending
    let approve_args = ApproveArgs {
        spender: Account { owner: dex_canister, subaccount: None },
        amount: Nat::from(ghc_amount),
        // ... other fields
    };
    
    let _: () = ic_cdk::call(ghc_ledger, "icrc2_approve", (approve_args,))
        .await
        .map_err(|e| format!("GHC approve failed: {:?}", e))?;
    
    // 2. Approve paired token spending
    let approve_args2 = ApproveArgs {
        spender: Account { owner: dex_canister, subaccount: None },
        amount: Nat::from(paired_amount),
    };
    
    let _: () = ic_cdk::call(paired_ledger, "icrc2_approve", (approve_args2,))
        .await
        .map_err(|e| format!("Paired token approve failed: {:?}", e))?;
    
    // 3. Call DEX to add liquidity
    let add_args = AddLiquidityArgs {
        token0: ghc_ledger,
        token1: paired_ledger,
        amount0_desired: Nat::from(ghc_amount),
        amount1_desired: Nat::from(paired_amount),
        amount0_min: Nat::from(ghc_amount * 99 / 100),  // 1% slippage
        amount1_min: Nat::from(paired_amount * 99 / 100),
    };
    
    let (result,): (Result<Nat, String>,) = ic_cdk::call(
        dex_canister,
        "deposit",
        (add_args,)
    ).await.map_err(|e| format!("DEX call failed: {:?}", e))?;
    
    match result {
        Ok(lp_tokens) => Ok(lp_tokens.0.try_into().unwrap_or(0)),
        Err(e) => Err(format!("DEX returned error: {}", e)),
    }
}

// Execute remove liquidity
async fn execute_remove_liquidity(
    dex_canister: Principal,
    lp_token_canister: Principal,
    ghc_ledger: Principal,
    paired_ledger: Principal,
    lp_amount: u64,
    min_ghc: u64,
    min_paired: u64,
) -> Result<(u64, u64), String> {
    // 1. Approve LP token spending
    let approve_args = ApproveArgs {
        spender: Account { owner: dex_canister, subaccount: None },
        amount: Nat::from(lp_amount),
    };
    
    let _: () = ic_cdk::call(lp_token_canister, "icrc2_approve", (approve_args,))
        .await
        .map_err(|e| format!("LP approve failed: {:?}", e))?;
    
    // 2. Call DEX to remove liquidity
    let remove_args = RemoveLiquidityArgs {
        token0: ghc_ledger,
        token1: paired_ledger,
        liquidity: Nat::from(lp_amount),
        amount0_min: Nat::from(min_ghc),
        amount1_min: Nat::from(min_paired),
    };
    
    let (result,): (Result<(Nat, Nat), String>,) = ic_cdk::call(
        dex_canister,
        "withdraw",
        (remove_args,)
    ).await.map_err(|e| format!("DEX call failed: {:?}", e))?;
    
    match result {
        Ok((amount0, amount1)) => Ok((
            amount0.0.try_into().unwrap_or(0),
            amount1.0.try_into().unwrap_or(0),
        )),
        Err(e) => Err(format!("DEX returned error: {}", e)),
    }
}
```

---

## 8. Security Considerations

### Slippage Protection

Always set minimum amounts to protect against price manipulation:

```rust
// BAD: No slippage protection
min_amount: 0  // Attacker can sandwich attack!

// GOOD: 1% slippage tolerance
min_amount: expected_amount * 99 / 100

// BETTER: Calculate based on current pool state
min_amount: query_pool_price() * amount * 99 / 100
```

### Deadline Protection

Set transaction deadlines to prevent delayed execution attacks:

```rust
deadline: ic_cdk::api::time() + 10 * 60 * 1_000_000_000  // 10 minutes
```

### Approval Hygiene

Reset approvals after use to minimize exposure:

```rust
// After adding liquidity, reset approval to 0
let reset_approval = ApproveArgs {
    spender: dex_canister,
    amount: Nat::from(0u64),
};
ic_cdk::call(ghc_ledger, "icrc2_approve", (reset_approval,)).await?;
```

### Multi-DEX Diversification

Don't put all liquidity in one DEX:

```
Recommended Distribution:
├── ICPSwap:    60%  (main liquidity)
├── Sonic:      25%  (backup)
└── Reserve:    15%  (treasury buffer)
```

---

## 9. Implementation Guide

### Governance Canister Updates

To support DEX operations, add to `operational_governance`:

```rust
// Track liquidity positions
#[derive(CandidType, Deserialize, Clone)]
struct LiquidityPosition {
    pool_id: String,
    dex_canister: Principal,
    lp_token_canister: Principal,
    lp_amount: u64,
    token0: Principal,
    token1: Principal,
    token0_added: u64,
    token1_added: u64,
    added_at: u64,
    added_by_proposal: u64,
}

// Store positions
static LIQUIDITY_POSITIONS: RefCell<StableBTreeMap<String, LiquidityPosition, Memory>> = ...;

// Query current positions
#[query]
fn get_liquidity_positions() -> Vec<LiquidityPosition> {
    LIQUIDITY_POSITIONS.with(|p| {
        p.borrow().iter().map(|(_, v)| v).collect()
    })
}

// Query total value locked
#[update]
async fn get_total_liquidity_value() -> Result<u64, String> {
    let positions = get_liquidity_positions();
    let mut total = 0u64;
    
    for pos in positions {
        // Query current pool price and calculate position value
        let value = query_position_value(pos).await?;
        total += value;
    }
    
    Ok(total)
}
```

### Testing Checklist

Before mainnet deployment, test on local network:

- [ ] Add liquidity with valid parameters
- [ ] Remove liquidity partially
- [ ] Remove liquidity fully
- [ ] Slippage protection triggers (set very high min)
- [ ] Deadline protection triggers (set past deadline)
- [ ] Unauthorized caller rejected
- [ ] Insufficient balance handling
- [ ] Multiple positions tracking
- [ ] Position value querying

---

## Summary

| Aspect | Details |
|--------|---------|
| **Add Liquidity** | Proposal → Voting (7d) → Timelock (2d) → Execute |
| **Remove Liquidity** | Same flow, can be partial or full |
| **Recommended DEX** | ICPSwap (largest, ICRC-2 compatible) |
| **Slippage Protection** | Always set min amounts (typically 1%) |
| **LP Token Storage** | Governance canister holds LP tokens |
| **Position Tracking** | Store all positions for transparency |
| **Diversification** | Spread across multiple DEXes |

---

*This guide should be updated as DEX integrations are implemented and tested.*
