# GHC Token ICO Strategy

> **Last Updated:** January 2026  
> **Purpose:** Comprehensive plan for GHC token Initial Coin Offering (ICO)

> **⚠️ ARCHITECTURE UPDATE (January 2026)**
>
> This document references `operational_governance` which has been refactored into:
> - **`treasury_canister`**: Token custody (4.25B MC), MMCR, transfer execution
> - **`governance_canister`**: Proposals, voting, ICO funding proposals
>
> For ICO operations: `FundIco` and `CreateLbp` proposals go to `governance_canister`, which calls `treasury_canister` for token transfers.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current State](#2-current-state)
3. [ICO Strategy Overview](#3-ico-strategy-overview)
4. [Phase 1: Custom ICO Canister](#4-phase-1-custom-ico-canister)
5. [Phase 2: Sonic LBP (Liquidity Bootstrapping Pool)](#5-phase-2-sonic-lbp-liquidity-bootstrapping-pool)
6. [Phase 3: Multi-DEX Liquidity](#6-phase-3-multi-dex-liquidity)
7. [Financial Flow](#7-financial-flow)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Risk Considerations](#9-risk-considerations)

---

## 1. Executive Summary

This document outlines a **dual-phase ICO strategy** for the GHC (GreenHero Coin) token:

| Phase | Method | Purpose |
|-------|--------|---------|
| **Phase 1** | Custom ICO Canister | Fixed-price sale with fiat on-ramp |
| **Phase 2** | Sonic LBP | Fair price discovery + automatic pool creation |
| **Phase 3** | Multi-DEX Liquidity | Ongoing liquidity provision via governance |

**Key Decision**: We will perform both a custom ICO (for fiat access and controlled distribution) AND an LBP (for fair price discovery and instant trading pool creation).

---

## 2. Current State

### 2.1 Treasury Holdings

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TREASURY STATE (GENESIS)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Canister: operational_governance                                           │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     TREASURY BALANCE: 4.25B GHC                      │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │              ALLOWANCE: 0.6B GHC (Spendable)                 │   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   │                                                                      │   │
│   │                    LOCKED: 3.65B GHC (MMCR over 20 years)            │   │
│   │                                                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Available for ICO: 0.6B GHC from initial allowance                        │
│   + 15.2M GHC/month via MMCR                                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Existing Infrastructure

| Component | Status | Purpose |
|-----------|--------|---------|
| `operational_governance` | ✅ Deployed | Treasury + governance proposals |
| `ghc_ledger` | ✅ Deployed | ICRC-1/ICRC-2 compliant token |
| DEX Integration Guide | ✅ Documented | See `DEX_INTEGRATION.md` |
| Governance Proposals | ✅ Implemented | Vote + timelock + execute |

---

## 3. ICO Strategy Overview

### 3.1 Phased Approach

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      GHC ICO STRATEGY (PHASED APPROACH)                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  PHASE 1: CUSTOM ICO CANISTER (Months 1-3)                                       │
│  ═════════════════════════════════════════                                       │
│  • Fixed-rate sale with fiat on-ramp                                             │
│  • Accept ckUSDC, ckUSDT, ICP, and Credit/Debit cards                            │
│  • Controlled distribution (build initial holder base)                           │
│  • Treasury provides GHC at fixed price (e.g., 1 GHC = $0.01)                    │
│  • Proceeds go to Treasury (in ckUSDC) for operations                            │
│                                                                                  │
│                                    │                                             │
│                                    ▼                                             │
│                                                                                  │
│  PHASE 2: DEX LAUNCHPAD / LBP (Month 4+)                                         │
│  ═══════════════════════════════════════                                         │
│  • Use Sonic LBP (Liquidity Bootstrapping Pool) for fair launch                  │
│  • Price discovery mechanism (starts high, drops to market equilibrium)          │
│  • Simultaneously creates trading pool on Sonic                                  │
│  • Use ckUSDC from Phase 1 as seed capital                                       │
│                                                                                  │
│                                    │                                             │
│                                    ▼                                             │
│                                                                                  │
│  PHASE 3: MULTI-DEX LIQUIDITY (Ongoing)                                          │
│  ═══════════════════════════════════════                                         │
│  • Governance proposals to add liquidity to multiple DEXes                       │
│  • Treasury earns trading fees                                                   │
│  • Community controls liquidity allocation via voting                            │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Why Both Approaches?

| Approach | Strengths | Weaknesses |
|----------|-----------|------------|
| **Custom ICO Only** | Predictable raise, fiat access | No price discovery, need separate DEX pool |
| **LBP Only** | Fair pricing, auto pool creation | No fiat, price uncertainty |
| **Both (Recommended)** | Best of both worlds | More complexity |

---

## 4. Phase 1: Custom ICO Canister

### 4.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           ICO CANISTER ARCHITECTURE                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│                        ┌──────────────────────────┐                              │
│                        │     FRONTEND / DAPP      │                              │
│                        │   (GreenHero Website)    │                              │
│                        └─────────────┬────────────┘                              │
│                                      │                                           │
│           ┌──────────────────────────┼──────────────────────────┐                │
│           │                          │                          │                │
│           ▼                          ▼                          ▼                │
│  ┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐     │
│  │  FIAT ON-RAMP   │    │     ICO_CANISTER     │    │  CRYPTO PAYMENT     │     │
│  │    (icRamp,     │    │                      │    │    (Direct)         │     │
│  │   MoonPay, etc) │    │  • Fixed Rate: $0.01 │    │                     │     │
│  │                 │    │  • Accept: ckUSDC,   │    │  • ICP Transfer     │     │
│  │  Credit Card ──►│────│    ckUSDT, ICP       │────│  • ckUSDC Transfer  │     │
│  │  → ckUSDC       │    │  • KYC optional      │    │                     │     │
│  │                 │    │  • Vesting optional  │    │                     │     │
│  └─────────────────┘    │  • Max/Min purchase  │    └─────────────────────┘     │
│                         │                      │                                 │
│                         │      PURCHASE()      │                                 │
│                         │          │           │                                 │
│                         └──────────┼───────────┘                                 │
│                                    │                                             │
│                    ┌───────────────┼───────────────┐                            │
│                    ▼               ▼               ▼                            │
│       ┌────────────────┐  ┌──────────────────┐  ┌──────────────────┐            │
│       │  GHC LEDGER    │  │ TREASURY (op_gov)│  │    PROCEEDS      │            │
│       │                │  │                  │  │    (ckUSDC)      │            │
│       │ Transfer GHC   │  │ Allowance check  │  │                  │            │
│       │ to Buyer       │  │ & deduct         │  │ For operations,  │            │
│       │                │  │                  │  │ DEX liquidity    │            │
│       └────────────────┘  └──────────────────┘  └──────────────────┘            │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 ICO Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Token Price** | $0.01 per GHC | Fixed rate, denominated in USD (via ckUSDC) |
| **ICO Supply** | 100,000,000 GHC | From 0.6B allowance |
| **Target Raise** | $1,000,000 | 100M GHC × $0.01 |
| **Min Purchase** | $10 (1,000 GHC) | Accessibility for small buyers |
| **Max Purchase** | $50,000 (5M GHC) | Anti-whale, may require vesting |
| **Duration** | 60-90 days | Or until sold out |

### 4.3 Payment Options

| Method | Asset | Flow |
|--------|-------|------|
| **Credit/Debit Card** | Fiat → ckUSDC | Via fiat on-ramp (icRamp, MoonPay, Transak) |
| **Direct Crypto** | ckUSDC | Direct transfer to ICO canister |
| **Direct Crypto** | ckUSDT | Direct transfer to ICO canister |
| **Direct Crypto** | ICP | Converted to ckUSDC at market rate |

### 4.4 Fiat On-Ramp Options

| Provider | Type | ICP Support | Notes |
|----------|------|-------------|-------|
| **icRamp** | Decentralized | Native | Uses HTTPS outcalls, no CEX |
| **MoonPay** | CEX | Via ICP | Well-known, easy integration |
| **Transak** | CEX | Direct ICP | Supports many fiat currencies |
| **ChangeNOW** | Instant swap | ICP | Buy ICP with card |

**Recommended Flow**: User pays with credit card → icRamp converts to ckUSDC → ICO canister receives ckUSDC → User receives GHC

### 4.5 ICO Canister Interface

```rust
// Core purchase function
async fn purchase(payment_amount: u64, payment_token: PaymentToken) -> Result<PurchaseReceipt, String>;

// Query current ICO status
fn get_ico_status() -> IcoStatus;

// Query user's purchases
fn get_user_purchases(user: Principal) -> Vec<Purchase>;

// Admin: Update ICO parameters (governance controlled)
async fn update_ico_config(config: IcoConfig) -> Result<(), String>;
```

### 4.6 Vesting for Large Purchases

For purchases above $10,000:

| Amount | Vesting Schedule |
|--------|------------------|
| $10K - $25K | 25% immediate, 25% monthly over 3 months |
| $25K - $50K | 20% immediate, 20% monthly over 4 months |
| >$50K | Requires governance approval |

---

## 5. Phase 2: Sonic LBP (Liquidity Bootstrapping Pool)

### 5.1 What is an LBP?

An LBP is a **Dutch Auction-style token sale** with automatic price discovery. Unlike a fixed-price sale, the price starts HIGH and decreases over time until buyers find it attractive.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     HOW LBP WEIGHT SHIFTING WORKS                                 │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                   │
│  TRADITIONAL AMM POOL (50/50):                                                    │
│  ═════════════════════════════                                                    │
│                                                                                   │
│    ┌─────────────────────────────────────┐                                        │
│    │   GHC: 50%    │    ckUSDC: 50%       │  ← Constant ratio                     │
│    │   1,000,000   │    $10,000           │                                       │
│    └─────────────────────────────────────┘                                        │
│                                                                                   │
│    Price = ckUSDC_amount / GHC_amount = $0.01/GHC (fixed by supply)              │
│                                                                                   │
│                                                                                   │
│  LBP POOL (Dynamic Weights):                                                      │
│  ═══════════════════════════                                                      │
│                                                                                   │
│  DAY 1 START:                            DAY 5 END:                               │
│    ┌─────────────────────────┐             ┌─────────────────────────┐            │
│    │ GHC: 96% │ ckUSDC: 4%   │   ────►     │ GHC: 50% │ ckUSDC: 50%  │            │
│    │ 50M      │ $50,000      │             │ 50M      │ $50,000      │            │
│    └─────────────────────────┘             └─────────────────────────┘            │
│                                                                                   │
│    Price Formula: ckUSDC × (GHC_weight / ckUSDC_weight) / GHC                     │
│                                                                                   │
│    Day 1: $50,000 × (96/4) / 50,000,000 = $0.024/GHC  (HIGH)                      │
│    Day 5: $50,000 × (50/50) / 50,000,000 = $0.001/GHC (LOW if no buys)            │
│                                                                                   │
│  THE MAGIC: Weight change → Price drops automatically over time                   │
│  Buying pressure → Price stabilizes at market equilibrium                         │
│                                                                                   │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 LBP Price Curve

```
  PRICE ($)
      │
 0.05 ┤  ▓▓▓▓                                         START: Weights 96/4
      │      ▓▓▓                                      Price artificially HIGH
 0.04 ┤         ▓▓▓                                   (discourages early whales)
      │            ▓▓▓
 0.03 ┤               ▓▓▓
      │                  ▓▓▓▓     ← Buying starts here (price people think is fair)
 0.02 ┤                      ▓▓▓▓
      │                          ▓▓▓▓▓▓▓▓▓            ← Buying pressure stops decline
 0.01 ┤                                   ▓▓▓▓▓▓▓▓▓▓▓  END: Market price discovered
      │                                               ← Pool becomes normal AMM
      └────────────────────────────────────────────────────────────
        Day 1      Day 2      Day 3      Day 4      Day 5

      Legend:
      ▓▓▓ = Price trajectory
      
      Without buying: Price falls to near-zero
      With buying: Price stabilizes at market equilibrium
```

### 5.3 Why GHC/ckUSDC Pair?

| Pair | Pros | Cons |
|------|------|------|
| **GHC/ICP** | Higher liquidity, native asset | ICP volatility affects GHC price perception |
| **GHC/ckUSDC** | Stable pricing, clear USD value | Requires holding ckUSDC |

**Recommendation**: Use **GHC/ckUSDC** for cleaner price communication.

### 5.4 LBP Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| **GHC in Pool** | 50,000,000 (50M) | From Treasury allowance |
| **ckUSDC Seed** | $50,000 - $100,000 | From Phase 1 proceeds |
| **Start Weights** | 96% GHC / 4% ckUSDC | High initial price |
| **End Weights** | 50% GHC / 50% ckUSDC | Standard AMM ratio |
| **Duration** | 5-7 days | Balance price discovery vs. market fatigue |

### 5.5 LBP Execution Steps

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      LBP EXECUTION CHECKLIST                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  PRE-LBP PREPARATION:                                                            │
│  ════════════════════                                                            │
│  □ Phase 1 ICO completed (Treasury has ckUSDC)                                   │
│  □ Decide LBP parameters (see 5.4)                                               │
│  □ Create governance proposal for LBP allocation                                 │
│  □ Vote passes + timelock completed                                              │
│  □ Verify GHC token is listed on Sonic                                           │
│                                                                                  │
│  LBP SETUP:                                                                      │
│  ══════════                                                                      │
│  □ operational_governance approves Sonic to spend GHC                            │
│  □ operational_governance approves Sonic to spend ckUSDC                         │
│  □ Call Sonic LBP creation function with parameters                              │
│  □ Announce LBP start time to community                                          │
│                                                                                  │
│  DURING LBP (5-7 Days):                                                          │
│  ══════════════════════                                                          │
│  □ Monitor price trajectory                                                      │
│  □ Community support (answer questions)                                          │
│  □ No intervention needed (automatic)                                            │
│                                                                                  │
│  POST-LBP:                                                                       │
│  ═════════                                                                       │
│  □ LBP ends automatically                                                        │
│  □ Pool converts to standard AMM (50/50)                                         │
│  □ Treasury receives LP tokens                                                   │
│  □ Trading is now live on Sonic                                                  │
│  □ Record final price for documentation                                          │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 5.6 Token Flow During LBP

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      TOKEN FLOW FOR LBP                                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  STEP 1: FUND LBP                                                                │
│  ─────────────────                                                               │
│                                                                                  │
│  ┌─────────────────────────┐         ┌─────────────────────────┐                 │
│  │   TREASURY              │   ━━━►  │   SONIC LBP POOL        │                 │
│  │   (op_governance)       │         │                         │                 │
│  │                         │         │                         │                 │
│  │   - 50M GHC             │         │   + 50M GHC             │                 │
│  │   - $100K ckUSDC        │         │   + $100K ckUSDC        │                 │
│  │                         │         │                         │                 │
│  │   Allowance: 0.6B→0.55B │         │   Weights: 96/4 → 50/50 │                 │
│  └─────────────────────────┘         └─────────────────────────┘                 │
│                                                                                  │
│                                                                                  │
│  STEP 2: LBP RUNS (5-7 DAYS)                                                     │
│  ───────────────────────────                                                     │
│                                                                                  │
│  ┌─────────────────────────┐         ┌─────────────────────────┐                 │
│  │   BUYERS                │   ━━━►  │   SONIC LBP POOL        │                 │
│  │                         │         │                         │                 │
│  │   Send ckUSDC           │         │   GHC decreases         │                 │
│  │   Receive GHC           │         │   ckUSDC increases      │                 │
│  │                         │         │                         │                 │
│  └─────────────────────────┘         └─────────────────────────┘                 │
│                                                                                  │
│                                                                                  │
│  STEP 3: LBP ENDS                                                                │
│  ────────────────                                                                │
│                                                                                  │
│  ┌─────────────────────────┐         ┌─────────────────────────┐                 │
│  │   SONIC AMM POOL        │◄──────► │   TREASURY              │                 │
│  │   (Permanent)           │         │   (op_governance)       │                 │
│  │                         │         │                         │                 │
│  │   ~10M GHC remaining    │         │   Holds: 500K LP Tokens │                 │
│  │   ~$400K ckUSDC gained  │         │   Earns: Trading fees   │                 │
│  │                         │         │                         │                 │
│  │   Price: ~$0.01/GHC     │         │   Can: Add more later   │                 │
│  └─────────────────────────┘         └─────────────────────────┘                 │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 5.7 LBP Benefits

| Benefit | Description |
|---------|-------------|
| **Fair Price Discovery** | Market determines value, not arbitrary decision |
| **Anti-Whale Protection** | Buying early = overpaying; discourages front-running |
| **Low Capital Requirement** | Only 4-10% seed capital needed vs 50% for normal pool |
| **Automatic Pool Creation** | Trading market exists immediately after LBP |
| **Credibility** | Industry-standard fair launch mechanism |
| **No Price Controversy** | "Market decided" vs "founders picked price" |

### 5.8 LBP Risks

| Risk | Mitigation |
|------|------------|
| **Price lower than expected** | Phase 1 ICO already captured value at fixed price |
| **Low participation** | Strong marketing + Phase 1 builds community |
| **Technical failure** | Test on testnet first, small initial allocation |
| **Whale manipulation** | LBP mechanics inherently discourage this |

---

## 6. Phase 3: Multi-DEX Liquidity

After Phase 2, we have:
- A trading pool on Sonic with LP tokens held by Treasury
- Established market price from LBP
- ckUSDC proceeds for further liquidity provision

### 6.1 Integration with Existing Governance

See `DEX_INTEGRATION.md` for full details. The flow:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                POST-LBP LIQUIDITY MANAGEMENT                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  TREASURY STATE AFTER LBP:                                                       │
│  ═════════════════════════                                                       │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐         │
│  │   OPERATIONAL_GOVERNANCE (Treasury)                                  │         │
│  │                                                                      │         │
│  │   GHC Balance: ~0.5B (remaining allowance)                           │         │
│  │   ckUSDC Balance: ~$800K (ICO proceeds minus LBP seed)               │         │
│  │   Sonic LP Tokens: 500,000 (from LBP)                                │         │
│  │                                                                      │         │
│  └─────────────────────────────────────────────────────────────────────┘         │
│                                                                                  │
│                                                                                  │
│  ADDING MORE LIQUIDITY (VIA GOVERNANCE):                                         │
│  ═══════════════════════════════════════                                         │
│                                                                                  │
│  1. Create Proposal:                                                             │
│     AddDexLiquidity {                                                            │
│         dex_canister: sonic_router_id,                                           │
│         pool_id: "GHC_ckUSDC",                                                   │
│         ghc_amount: 100_000_000_00000000,   // 100M GHC                          │
│         paired_token: ckusdc_ledger_id,                                          │
│         paired_amount: 1_000_000_00000000,  // $1M ckUSDC                        │
│     }                                                                            │
│                                                                                  │
│  2. Community Votes (7 days)                                                     │
│                                                                                  │
│  3. Timelock (2 days)                                                            │
│                                                                                  │
│  4. Execute → Treasury receives more LP tokens                                   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Multi-DEX Strategy

| DEX | Allocation | Purpose |
|-----|------------|---------|
| **Sonic** | 50% | Primary from LBP, main trading venue |
| **ICPSwap** | 35% | Largest ICP DEX, additional reach |
| **KongSwap** | 15% | Emerging DEX, diversification |

### 6.3 LP Token Management

Treasury holds LP tokens from each DEX, which:
- Represent ownership share in liquidity pools
- Earn trading fees (typically 0.3% of trades)
- Can be redeemed via governance vote

---

## 7. Financial Flow

### 7.1 Phase 1 (Custom ICO)

```
User pays $1,000          ICO Canister            Treasury
      │                        │                      │
      │   ────$1,000 ckUSDC───►│                      │
      │                        │                      │
      │   ◄──100,000 GHC──────│──Deduct from────────►│
      │                        │  allowance           │
      │                        │                      │
      │                        │   ──$1,000 ckUSDC──►│
      │                        │   (proceeds)         │
      │                        │                      │

Result:
• User: +100,000 GHC
• Treasury: -100,000 GHC allowance, +$1,000 ckUSDC
```

### 7.2 Phase 2 (LBP)

```
Treasury                    Sonic LBP                  Buyers
    │                           │                         │
    │──50M GHC + $100K ckUSDC──►│                         │
    │                           │                         │
    │                           │◄──ckUSDC (at varying───│
    │                           │      prices)            │
    │                           │                         │
    │                           │────GHC to buyers───────►│
    │                           │                         │
    │◄──LP tokens──────────────│                         │
    │   (after LBP ends)        │                         │

Result:
• Treasury: -50M GHC, +LP tokens (worth pool share)
• Buyers: +GHC at market-discovered price
• Pool: Now permanent 50/50 AMM with ~$400K+ TVL
```

### 7.3 Combined Timeline

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         ICO FINANCIAL TIMELINE                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  MONTH 1-3: PHASE 1 (Custom ICO)                                                 │
│  ═══════════════════════════════                                                 │
│  • Sell 100M GHC at $0.01 = $1M raised                                           │
│  • Treasury: 0.6B → 0.5B GHC allowance                                           │
│  • Treasury: +$1M ckUSDC                                                         │
│                                                                                  │
│  MONTH 4: PHASE 2 (LBP)                                                          │
│  ═══════════════════════                                                         │
│  • Allocate: 50M GHC + $100K ckUSDC to LBP                                       │
│  • LBP runs 5-7 days                                                             │
│  • Result: Price discovery (assume ~$0.012/GHC settled)                          │
│  • Treasury: 0.5B → 0.45B GHC, +LP tokens + remaining ckUSDC                     │
│                                                                                  │
│  MONTH 5+: PHASE 3 (Multi-DEX)                                                   │
│  ═════════════════════════════                                                   │
│  • Add 100M GHC + $1.2M ckUSDC to ICPSwap                                        │
│  • Add 50M GHC + $600K ckUSDC to KongSwap                                        │
│  • Total TVL across DEXes: ~$3-4M                                                │
│  • Treasury earns: ~0.3% × daily volume in fees                                  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Implementation Roadmap

### 8.1 Development Timeline

| Phase | Duration | Deliverables | Effort |
|-------|----------|--------------|--------|
| **1a** | 2-3 weeks | ICO canister (accept ckUSDC, ICP) | Medium |
| **1b** | 1-2 weeks | Fiat on-ramp integration (icRamp/MoonPay) | Medium |
| **1c** | 1 week | Frontend for ICO purchase page | Medium |
| **2** | 1 week | Sonic LBP configuration | Low |
| **3** | Ongoing | DEX liquidity via governance | Already built |

### 8.2 Canister Development

```
src/
├── ico_canister/               # NEW - Phase 1
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── ico_canister.did
│
├── operational_governance/     # EXISTING - Add LBP proposal support
│   └── src/
│       └── lib.rs             # Add ExecuteLBP proposal type
│
└── ... (existing canisters)
```

### 8.3 Governance Updates Needed

Add to `operational_governance`:

```rust
// New proposal types for ICO/LBP
enum ProposalAction {
    // Existing
    Transfer { recipient: Principal, amount: u64 },
    AddDexLiquidity { /* ... */ },
    RemoveDexLiquidity { /* ... */ },
    
    // New
    FundIco { ico_canister: Principal, ghc_amount: u64 },
    CreateLbp { 
        dex_canister: Principal,
        ghc_amount: u64,
        seed_token: Principal,
        seed_amount: u64,
        start_weight_ghc: u8,
        end_weight_ghc: u8,
        duration_days: u8,
    },
}
```

---

## 9. Risk Considerations

### 9.1 Regulatory Risk

| Risk | Mitigation |
|------|------------|
| Securities laws | Utility token focus, no profit promises, legal review |
| KYC requirements | Optional KYC layer in ICO canister |
| Jurisdiction | Terms of service, geo-blocking if needed |

### 9.2 Technical Risk

| Risk | Mitigation |
|------|------------|
| Smart contract bugs | Audits, testnet deployment, gradual rollout |
| DEX integration failure | Test with small amounts first |
| On-ramp issues | Multiple provider fallbacks |

### 9.3 Market Risk

| Risk | Mitigation |
|------|------------|
| Low ICO demand | Marketing, community building, competitive pricing |
| LBP price crash | Phase 1 already captured value at fixed price |
| Impermanent loss | Diversified DEX strategy, long-term view |

---

## Summary

| Phase | Method | Amount | Expected Outcome |
|-------|--------|--------|------------------|
| **Phase 1** | Custom ICO | 100M GHC | $1M raised, initial holders |
| **Phase 2** | Sonic LBP | 50M GHC | Market price discovery, trading pool |
| **Phase 3** | Multi-DEX | 200M+ GHC | Deep liquidity, fee earnings |

**Total GHC Allocated**: ~350M (from 0.6B initial allowance + monthly MMCR releases)

**Next Steps**:
1. Design ICO canister in detail
2. Select and integrate fiat on-ramp provider
3. Prepare marketing materials
4. Deploy and test on testnet
5. Legal review
6. Mainnet launch

---

*This document should be updated as implementation progresses and market conditions change.*
