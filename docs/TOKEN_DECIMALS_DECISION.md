# Token Decimals Decision: Why 8 Instead of 10

## Executive Summary

GHC uses **8 decimals** instead of 10 due to **u64 overflow constraints** when representing large token supplies. This document explains the technical reasoning and trade-offs.

---

## The Problem: u64 Integer Overflow

The ICP ecosystem (and our codebase) uses `u64` (unsigned 64-bit integers) to represent token amounts in their smallest units. This creates a fundamental constraint:

```
u64 Maximum Value: 18,446,744,073,709,551,615 (~1.8 × 10^19)
```

### Impact on Token Supply Representation

| Decimals | 1 Token = | 9.5B Tokens = | Fits in u64? |
|----------|-----------|---------------|--------------|
| **6** | 10^6 | 9.5 × 10^15 | ✅ Yes |
| **8** | 10^8 | 9.5 × 10^17 | ✅ Yes |
| **10** | 10^10 | 9.5 × 10^19 | ❌ **NO - OVERFLOW!** |
| **12** | 10^12 | 9.5 × 10^21 | ❌ No |

With **10 decimals** and a **9.5 billion token supply**:
```
9,500,000,000 × 10,000,000,000 = 95,000,000,000,000,000,000 (9.5 × 10^19)
```

This exceeds the u64 maximum by approximately **5x**, causing integer overflow.

---

## Comparison: 8 vs 10 Decimals

### Option A: 8 Decimals (CHOSEN ✅)

| Aspect | Details |
|--------|---------|
| **Representation** | 1 GHC = 100,000,000 smallest units |
| **Max Supply** | 9.5B × 10^8 = 9.5 × 10^17 ✅ |
| **u64 Capacity Used** | ~5% of max (plenty of headroom) |
| **Industry Standard** | Same as **ICP**, **Bitcoin** |
| **Smallest Unit** | 0.00000001 GHC |
| **Code Changes** | None required |

**Pros:**
- ✅ Works with existing u64 infrastructure
- ✅ Compatible with ICP wallets and DEXs
- ✅ Same precision as Bitcoin (satoshis)
- ✅ No risk of overflow in calculations
- ✅ Battle-tested in production systems

**Cons:**
- ❌ Less precision than 10+ decimals (rarely needed)

---

### Option B: 10 Decimals (REJECTED ❌)

| Aspect | Details |
|--------|---------|
| **Representation** | 1 GHC = 10,000,000,000 smallest units |
| **Max Supply** | 9.5B × 10^10 = 9.5 × 10^19 ❌ |
| **u64 Capacity Used** | 530% of max (OVERFLOW) |
| **Industry Standard** | Non-standard |
| **Smallest Unit** | 0.0000000001 GHC |
| **Code Changes** | Massive refactor required |

**Required Changes for 10 Decimals:**
1. Change all `u64` balance fields to `u128` in:
   - `staking_hub/src/lib.rs`
   - `user_profile/src/lib.rs`
   - `operational_governance/src/lib.rs`
   - All Candid interface files (`.did`)
2. Update all arithmetic operations
3. Update serialization/deserialization logic
4. Test all edge cases with larger number handling
5. Verify wallet/DEX compatibility with u128

**Pros:**
- ✅ More precision (10 decimal places)
- ✅ Future-proof for micro-transactions

**Cons:**
- ❌ Causes u64 overflow with our supply
- ❌ Requires u128 across entire codebase
- ❌ Non-standard (wallets may not display correctly)
- ❌ Increased complexity in all calculations
- ❌ Higher risk of arithmetic bugs

---

### Option C: 10 Decimals with Workarounds (REJECTED ❌)

Potential workarounds were considered but rejected:

#### C1: Track in Whole Tokens Internally
- Store amounts as whole tokens, convert at boundaries
- **Problem**: Loses precision, error-prone conversions

#### C2: Use u128 Throughout
- Replace all u64 with u128
- **Problem**: Massive refactor, unclear wallet compatibility

#### C3: Use String/BigInt Representation
- Store as strings, parse when needed
- **Problem**: Performance impact, complex parsing logic

---

## The Math: Why 8 Decimals Is Safe

### Total Supply Calculation

```
Total Supply: 9.5 Billion GHC
Decimals: 8
Multiplier: 10^8 = 100,000,000

Smallest Unit Representation:
9,500,000,000 × 100,000,000 = 950,000,000,000,000,000 (9.5 × 10^17)

u64 Max: 18,446,744,073,709,551,615 (~1.8 × 10^19)

Headroom: 1.8 × 10^19 / 9.5 × 10^17 ≈ 19x safety margin
```

### Per-Partition Breakdown

| Partition | Tokens | In e8s (smallest units) | % of u64 Max |
|-----------|--------|-------------------------|--------------|
| MUC (Staking Hub) | 4.75B | 4.75 × 10^17 | 2.6% |
| MC (Treasury) | 4.25B | 4.25 × 10^17 | 2.3% |
| MC (Founder Vesting) | 0.5B | 5.0 × 10^16 | 0.3% |
| **Total** | **9.5B** | **9.5 × 10^17** | **5.2%** |

**Conclusion**: With 8 decimals, we use only ~5% of u64 capacity, leaving ample room for:
- Intermediate calculations
- Interest/reward accumulation
- Future supply expansions (if ever needed)

---

## Industry Comparison

| Token | Decimals | Notes |
|-------|----------|-------|
| **Bitcoin (BTC)** | 8 | 1 BTC = 100,000,000 satoshis |
| **ICP** | 8 | Native IC token |
| **Ethereum (ETH)** | 18 | Uses u256 (256-bit integers) |
| **USDC** | 6 | Lower precision for stablecoin |
| **GHC (Ours)** | 8 | Matches ICP/BTC standard |

Note: Ethereum uses 18 decimals because it uses u256 integers, which can handle astronomically large numbers. ICP uses u64, so we follow the ICP/Bitcoin standard of 8 decimals.

---

## Conclusion

**8 decimals is the correct choice** for GHC because:

1. **Technical Necessity**: 10 decimals causes u64 overflow with our 9.5B supply
2. **Industry Alignment**: Matches ICP and Bitcoin standards
3. **Ecosystem Compatibility**: Works with all ICP wallets and DEXs
4. **Sufficient Precision**: 0.00000001 GHC smallest unit is more than adequate
5. **No Code Changes**: Works with existing infrastructure

The decision to use 8 decimals is not a limitation but a pragmatic engineering choice that ensures reliability, compatibility, and safety.

---

## References

- [ICP Ledger Specification](https://internetcomputer.org/docs/current/references/icrc1-standard)
- [Bitcoin Satoshi Definition](https://en.bitcoin.it/wiki/Satoshi_(unit))
- [u64 Maximum Value](https://doc.rust-lang.org/std/primitive.u64.html#associatedconstant.MAX)
