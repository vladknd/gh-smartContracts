# Treasury Allowance Scheduling: Technical & Legal Analysis

**Last Updated:** January 17, 2026

**Status:** ✅ IMPLEMENTED - Calendar-based scheduling (1st of month at 12:00 AM ET) has been implemented in the treasury_canister.

---

## Executive Summary

This document analyzes two approaches for scheduling treasury allowance increases (MMCR - Monthly Maintenance & Capital Release):

1. **Previous System:** Fixed 30-day intervals from genesis
2. **Current (Implemented):** 1st day of calendar month at 12:00 AM Eastern Time

We examine technical implementation, business requirements, legal considerations, and regulatory compliance to determine the optimal approach.

---

## Table of Contents

1. [Current System Overview](#current-system-overview)
2. [Proposed Calendar Month System](#proposed-calendar-month-system)
3. [Side-by-Side Comparison](#side-by-side-comparison)
4. [Legal & Regulatory Considerations](#legal--regulatory-considerations)
5. [Accounting & Financial Reporting](#accounting--financial-reporting)
6. [Implementation Analysis](#implementation-analysis)
7. [Hybrid Solutions](#hybrid-solutions)
8. [Recommendations](#recommendations)

---

## Current System Overview

### How It Works

The operational governance canister implements a **fixed 30-day interval** system:

```rust
const MMCR_MIN_INTERVAL_NANOS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000; // 30 days
```

**Schedule:**
- **Genesis:** 600M GHC available immediately as initial allowance
- **Every 30 days:** +15.2M GHC added to allowance
- **Final release (240th):** +17.2M GHC
- **Total duration:** 240 releases over approximately 20 years

**Trigger Mechanism:**
- Anyone can call `execute_mmcr()` update function
- System checks: `current_time >= last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS`
- If eligible, allowance increases and timestamp updates

### Technical Characteristics

- ✅ **Timezone-agnostic** - Uses Internet Computer's UTC timestamps
- ✅ **Mathematically precise** - Exactly 2,592,000,000,000 nanoseconds between releases
- ✅ **Simple implementation** - Single timestamp comparison
- ✅ **Predictable** - Can calculate exact future dates: `genesis + (30 days × N)`
- ✅ **No edge cases** - Works identically every cycle

### Example Release Schedule (Current System)

| Release # | Date & Time (UTC) | Days Since Last | Allowance Increase |
|-----------|-------------------|-----------------|-------------------|
| Genesis | Jan 1, 2026 00:00:00 | - | 600M GHC |
| 1 | Jan 31, 2026 00:00:00 | 30 | +15.2M GHC |
| 2 | Mar 2, 2026 00:00:00 | 30 | +15.2M GHC |
| 3 | Apr 1, 2026 00:00:00 | 30 | +15.2M GHC |
| 4 | May 1, 2026 00:00:00 | 30 | +15.2M GHC |
| 12 | Dec 30, 2026 00:00:00 | 30 | +15.2M GHC |

*Note: Dates drift from calendar months but intervals are always exactly 30 days.*

---

## Proposed Calendar Month System

### How It Would Work

The allowance would increase on the **last day of each calendar month at 12:00 AM Eastern Time (ET)**.

**Example Schedule:**
- January 31, 2026 at 12:00:00 AM ET → +15.2M GHC
- February 28, 2026 at 12:00:00 AM ET → +15.2M GHC
- March 31, 2026 at 12:00:00 AM ET → +15.2M GHC
- April 30, 2026 at 12:00:00 AM ET → +15.2M GHC

### Technical Requirements

Would require:

1. **Timezone Conversion Logic**
   ```rust
   // Pseudo-code - NOT current implementation
   fn is_last_day_of_month_et(timestamp: u64) -> bool {
       let utc_time = from_nanos(timestamp);
       let et_time = convert_utc_to_et(utc_time); // Handle EST vs EDT
       
       let is_midnight = et_time.hour() == 0 && et_time.minute() == 0;
       let is_last_day = et_time.day() == get_last_day_of_month(et_time);
       
       is_midnight && is_last_day
   }
   ```

2. **Calendar Logic**
   - Detect month boundaries
   - Handle leap years (Feb 28 vs Feb 29)
   - Account for different month lengths (28-31 days)

3. **Daylight Saving Time Handling**
   - Spring: EST becomes EDT (March) - skip 1 hour
   - Fall: EDT becomes EST (November) - repeat 1 hour
   - Risk of executing twice or missing execution during transitions

### Example Release Schedule (Calendar Month System)

| Release # | Date & Time (ET) | Days Since Last | Allowance Increase |
|-----------|------------------|-----------------|-------------------|
| Genesis | Jan 1, 2026 00:00:00 | - | 600M GHC |
| 1 | Jan 31, 2026 00:00:00 | 30 | +15.2M GHC |
| 2 | Feb 28, 2026 00:00:00 | **28** | +15.2M GHC |
| 3 | Mar 31, 2026 00:00:00 | **31** | +15.2M GHC |
| 4 | Apr 30, 2026 00:00:00 | 30 | +15.2M GHC |
| 5 | May 31, 2026 00:00:00 | **31** | +15.2M GHC |
| 12 | Dec 31, 2026 00:00:00 | **31** | +15.2M GHC |

*Note: Intervals vary from 28 to 31 days depending on month.*

---

## Side-by-Side Comparison

| Aspect | 30-Day Fixed Interval | Calendar Month (Last Day) |
|--------|----------------------|---------------------------|
| **Interval Consistency** | Always 30 days | 28-31 days (varies) |
| **Fairness** | Equal wait for everyone | Shorter in Feb, longer in 31-day months |
| **Implementation Complexity** | Simple (1 comparison) | Complex (calendar + timezone logic) |
| **Timezone Issues** | None (UTC native) | Must handle ET, EST/EDT transitions |
| **DST Risk** | None | High (2x/year edge cases) |
| **Predictability** | Exact timestamp calculable | Exact date, but implementation-dependent |
| **Testing** | Easy | Difficult (DST edge cases) |
| **Blockchain Native** | Yes | No (calendars are human constructs) |
| **Financial Reporting Alignment** | No | Yes (monthly statements align) |
| **Maintenance** | Zero | Ongoing (timezone rule changes) |
| **Gas/Compute Cost** | Minimal | Higher (complex calculations) |
| **Audit Trail** | Perfect | Complex (timezone conversions in logs) |

### Variance Over Time

**Calendar Month Approach:** Over one year (12 releases)
- Shortest interval: February (28 days in 2026)
- Longest interval: 31-day months (7 occurrences)
- Total variation: **3-day swing** between fastest and slowest

**30-Day Approach:** Over one year (12.17 releases)
- Every interval: Exactly 30 days
- Total variation: **0 days**

Over the full 240-release period (20 years):
- Calendar: ~174 releases at 31 days, ~51 at 30 days, ~15 at 28/29 days
- Fixed: All 240 releases at exactly 30 days

---

## Legal & Regulatory Considerations

### When Calendar Months ARE Required

#### 1. **Securities Regulations (If GHC is Classified as a Security)**

**USA - Securities and Exchange Commission (SEC):**
- If GHC is deemed a security, you may need to file periodic reports (Form 10-Q quarterly, Form 10-K annually)
- **Requirement:** Financial statements must use calendar months/quarters
- **Impact:** Treasury activity would need to align with standard accounting periods

**Determination Needed:**
- Has GHC passed the Howey Test? (investment of money, common enterprise, expectation of profits from others' efforts)
- Consult a securities attorney

#### 2. **Tax Reporting Requirements**

**Corporate Income Tax:**
- Most jurisdictions require **monthly or quarterly** tax filings for corporations
- Canada (CRA) and USA (IRS) use calendar months for corporate tax periods
- If your DAO is structured as a corporation, trust, or taxable entity, calendar alignment may be required

**Cryptocurrency Tax Reporting:**
- IRS Notice 2014-21: Crypto is property, transactions must be reported
- If the treasury generates taxable events (spending, transfers), **calendar-based accounting simplifies reporting**

#### 3. **DAO Legal Entity Structure**

**If Registered as a Legal Entity:**

| Jurisdiction | Entity Type | Monthly Reporting Required? |
|--------------|-------------|----------------------------|
| Wyoming, USA | DAO LLC | No, but annual reports required |
| Vermont, USA | Blockchain-Based LLC | No, but annual reports required |
| Switzerland | Foundation (Stiftung) | Yes, monthly financial accounting |
| Cayman Islands | Foundation Company | Quarterly minimum |
| Marshall Islands | DAO LLC | No |

**Key Question:** What is your DAO's current legal structure?

#### 4. **Financial Disclosure Requirements**

**If You Conduct an ICO/Token Sale:**
- Many jurisdictions require **monthly disclosures** to token holders
- Example: Switzerland FINMA guidelines for token issuers require regular financial reporting
- **Calendar month alignment makes these reports cleaner**

### When Calendar Months Are NOT Required

#### You Can Use 30-Day Intervals If:

1. ✅ **Fully Decentralized DAO** - No legal entity registered anywhere
2. ✅ **GHC is a Utility Token** - Not a security, purely functional
3. ✅ **No Regulated Business Activities** - Not offering financial services
4. ✅ **Operating Purely On-Chain** - No traditional banking, corporate accounts
5. ✅ **Transparent Blockchain Accounting** - All transactions auditable on-chain

**Most Likely:** If your DAO is a **pure crypto protocol** without a legal entity, you have flexibility to use 30-day intervals.

---

## Accounting & Financial Reporting

### Generally Accepted Accounting Principles (GAAP)

**US GAAP and IFRS Requirements:**
- Both require **monthly closing periods** for financial statements
- Balance sheets, income statements, and cash flow statements are prepared **monthly**
- Public companies MUST align with calendar months
- Private companies have more flexibility but standardize on calendar months

### Implications for Treasury Management

**If Using Calendar Months:**
- ✅ Clean monthly financial statements (Jan 1-31, Feb 1-28, etc.)
- ✅ Easy to track: "In March, we spent X and received Y allowance"
- ✅ Auditors prefer calendar-aligned data
- ✅ Comparable to industry standards

**If Using 30-Day Intervals:**
- ⚠️ Financial periods don't align with calendar (Jan 1-30, Jan 31-Mar 1, etc.)
- ⚠️ More complex reconciliation for external reporting
- ✅ Still perfectly auditable (blockchain provides complete trail)
- ✅ Can convert to calendar months for reporting (track both internally)

### Professional Auditor Perspective

**What Auditors Care About:**
1. **Consistency** - Use the same method every period ✅ (Both approaches satisfy this)
2. **Traceability** - Can all transactions be verified? ✅ (Both approaches satisfy this)
3. **Standard Periods** - Calendar months are easier ⚠️ (30-day requires extra work)

**Bottom Line:** Auditors *prefer* calendar months but will accept 30-day intervals if:
- Clearly documented in financial policies
- Consistently applied
- Reconciliation to calendar months is provided for annual reports

---

## Implementation Analysis

### Current Implementation (Already Built)

**File:** `src/treasury_canister/src/lib.rs`

```rust
const MMCR_MIN_INTERVAL_NANOS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;

fn try_execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    TREASURY_STATE.with(|s| {
        let mut cell = s.borrow_mut();
        let mut state = cell.get().clone();
        
        // Simple check: has 30 days passed?
        if state.last_mmcr_timestamp > 0 && 
           current_time < state.last_mmcr_timestamp + MMCR_MIN_INTERVAL_NANOS {
            return Err("Too early for next MMCR".to_string());
        }
        
        // Execute release...
        state.allowance += MMCR_AMOUNT;
        state.last_mmcr_timestamp = current_time;
        // ...
    })
}
```

**Complexity:** ~10 lines of logic  
**Dependencies:** None (native IC API)  
**Edge Cases:** None

### Calendar Month Implementation (Would Need to Build)

**Estimated Complexity:** ~100-150 lines of logic

**Required Components:**

1. **Timezone Conversion Library**
   ```rust
   // Would need to add dependency
   use chrono::{DateTime, Utc, TimeZone, FixedOffset};
   use chrono_tz::America::New_York;
   ```

2. **Month-End Detection**
   ```rust
   fn get_last_day_of_month(year: i32, month: u32) -> u32 {
       match month {
           1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
           4 | 6 | 9 | 11 => 30,
           2 => if is_leap_year(year) { 29 } else { 28 },
           _ => panic!("Invalid month"),
       }
   }
   
   fn is_leap_year(year: i32) -> bool {
       (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
   }
   ```

3. **DST Handling**
   ```rust
   fn convert_utc_to_et(utc_timestamp: u64) -> DateTime<FixedOffset> {
       // EST (Standard): UTC-5
       // EDT (Daylight): UTC-4
       // Transition dates vary by year (2nd Sunday of March / 1st Sunday of November)
       // Must handle edge cases during the 1-hour transition window
   }
   ```

4. **Execution Window Logic**
   ```rust
   fn is_mmcr_eligible(current_time: u64, last_execution: u64) -> bool {
       let current_et = convert_utc_to_et(current_time);
       let last_et = convert_utc_to_et(last_execution);
       
       // Has a month boundary been crossed?
       let month_crossed = current_et.month() != last_et.month() || 
                          current_et.year() != last_et.year();
       
       // Are we in the execution window? (last day, midnight hour)
       let in_window = is_last_day_of_month_et(current_time);
       
       month_crossed && in_window
   }
   ```

**Challenges:**

- **DST Transition Bugs:** March and November have ambiguous/skipped hours
- **Canister Upgrades:** If timezone rules change, need to update canister
- **Testing:** Must simulate every month type, leap years, DST transitions
- **Gas Costs:** More computation per call

### Code Maintenance Over 20 Years

**30-Day Approach:**
- Code remains unchanged for 20+ years
- No timezone rule updates needed
- No DST logic to debug

**Calendar Month Approach:**
- Must update if US Congress changes DST rules (happened in 2007)
- Must update if Eastern Time zone boundaries change
- Requires ongoing maintenance for edge cases found in production

---

## Hybrid Solutions

If you need calendar alignment but want to keep simplicity:

### Option 1: Dual Tracking

**Implementation:**
- Keep 30-day intervals for on-chain execution
- Track calendar month equivalents for reporting

```rust
struct TreasuryState {
    allowance: u64,
    balance: u64,
    last_mmcr_timestamp: u64,  // Actual execution time (30-day intervals)
    last_mmcr_month: u8,        // Month number for reporting (1-12)
    last_mmcr_year: u16,        // Year for reporting
    // ...
}
```

**Benefits:**
- ✅ Simple on-chain execution (30-day logic)
- ✅ Clean monthly reports for external stakeholders
- ✅ Best of both worlds

### Option 2: Scheduled Execution with Buffer

**Implementation:**
- Calculate "target calendar dates" off-chain
- Allow execution within a **3-day window** around end of month

```rust
const EXECUTION_WINDOW_DAYS: u64 = 3;

// Can execute MMCR between day 28-31 of any month (if interval passed)
fn try_execute_mmcr() -> Result<u64, String> {
    let current_time = ic_cdk::api::time();
    
    // Minimum 28 days must have passed (shortest month)
    let min_interval = 28 * 24 * 60 * 60 * 1_000_000_000;
    if current_time < last_mmcr_timestamp + min_interval {
        return Err("Too early for next MMCR".to_string());
    }
    
    // Must be within last 4 days of any month (28-31)
    if !is_near_month_end(current_time) {
        return Err("Must execute near end of month".to_string());
    }
    
    // Execute...
}
```

**Benefits:**
- ✅ Allowances cluster around month-end (satisfies calendar alignment)
- ✅ Simpler than exact midnight ET implementation
- ⚠️ Still requires some calendar logic

### Option 3: Off-Chain Scheduler + On-Chain Execution

**Implementation:**
- Keep simple 30-day on-chain logic
- Run an **off-chain service** that calls `execute_mmcr()` on calendar schedule

**Example:**
```javascript
// Off-chain cron job (runs on server/GitHub Actions/etc.)
// Executes on last day of each month at 12:00 AM ET

const cron = require('node-cron');
const { Actor } = require('@dfinity/agent');

// Run at midnight ET on last day of month
cron.schedule('0 0 28-31 * *', async () => {
    const today = new Date();
    const tomorrow = new Date(today);
    tomorrow.setDate(today.getDate() + 1);
    
    // Only execute if tomorrow is a new month (today is last day)
    if (tomorrow.getDate() === 1) {
        await governanceCanister.execute_mmcr();
    }
});
```

**Benefits:**
- ✅ Keeps on-chain logic simple (30-day minimum check)
- ✅ Off-chain scheduler handles calendar complexity
- ⚠️ Requires trusted execution service (centralization risk)
- ⚠️ Fallback: Anyone can still call manually if scheduler fails

---

## Recommendations

### Scenario-Based Guidance

#### **Scenario 1: Pure DeFi Protocol (No Legal Entity)**

**Situation:**
- DAO is fully decentralized, no legal entity
- GHC is a utility token, not a security
- No plans for traditional VC funding or IPO
- Operating purely on-chain

**Recommendation:** ✅ **Keep 30-day fixed intervals**

**Reasoning:**
- No legal requirement for calendar months
- Maximum fairness and predictability
- Lowest implementation risk
- Blockchain-native approach aligns with decentralization ethos

---

#### **Scenario 2: Registered Entity with Potential ICO**

**Situation:**
- DAO is registered as a Foundation (Switzerland, Cayman, etc.)
- Planning ICO/token sale requiring regulatory compliance
- Need to file monthly financial reports
- May seek traditional financing or partnerships

**Recommendation:** ⚠️ **Consider calendar month alignment**

**Recommended Approach:**
1. Implement **Hybrid Option 1** (dual tracking)
2. Keep 30-day intervals on-chain
3. Add reporting fields to track calendar month equivalents
4. Frontend displays both: "Release #23 (March 2026)"

**Reasoning:**
- Satisfies regulatory reporting (can map to calendar months)
- Maintains on-chain simplicity
- Provides flexibility for future compliance needs

---

#### **Scenario 3: US-Based DAO LLC**

**Situation:**
- Registered as Wyoming DAO LLC or similar
- Subject to US tax reporting requirements
- Holds USD bank accounts in addition to crypto
- Board members are US persons

**Recommendation:** ⚠️ **Calendar month alignment may be beneficial**

**Recommended Approach:**
1. Consult with crypto-specialized CPA/tax attorney
2. If tax reporting requires calendar months, implement **Hybrid Option 2** (execution window)
3. Allow MMCR execution during last 3 days of month (28-31)
4. Minimum 28-day interval enforced

**Reasoning:**
- Simplifies tax preparation and IRS reporting
- Reduces accountant fees (less reconciliation work)
- Still maintains blockchain security and predictability

---

### Questions to Answer Before Deciding

**Talk to your legal/accounting team about:**

1. ❓ **Legal Structure:** Is the DAO a registered legal entity anywhere?
2. ❓ **Token Classification:** Could GHC be classified as a security?
3. ❓ **Regulatory Jurisdiction:** Which countries' regulations apply?
4. ❓ **Tax Reporting:** What tax filing requirements exist?
5. ❓ **Future Plans:** ICO, traditional financing, public listing?
6. ❓ **Stakeholder Expectations:** Do investors/partners expect calendar reporting?

---

## Technical Decision Matrix

| If your answer is... | Then choose... |
|---------------------|---------------|
| "We're a pure DeFi protocol with no legal entity" | 30-day fixed intervals |
| "We have a registered entity but no reporting requirements" | 30-day fixed intervals |
| "We file monthly financial reports but they're internal only" | 30-day + dual tracking |
| "We must provide monthly reports to regulators" | Calendar month or hybrid |
| "We're subject to securities regulations" | Calendar month (consult attorney) |
| "We're doing an ICO in a regulated market" | Calendar month (consult attorney) |
| "We hold both crypto and fiat, file taxes" | Calendar month or hybrid |

---

## Conclusion

**From a purely technical perspective:** The 30-day fixed interval approach is superior in every way (simplicity, fairness, security, maintainability).

**From a legal/regulatory perspective:** Calendar months may be required or strongly beneficial depending on your specific situation.

**Recommended Action Items:**

1. ✅ **Immediate:** Identify your DAO's legal structure and regulatory obligations
2. ✅ **Short-term:** Consult with a crypto-specialized attorney and accountant
3. ✅ **Decision:** Based on legal guidance, choose one of:
   - Keep 30-day intervals (if legally permissible)
   - Implement hybrid dual-tracking (compromise)
   - Full calendar month implementation (if legally required)

4. ✅ **Document:** Whatever you choose, document the decision and rationale for future audits

---

## Resources

### Legal Consultation Recommended

- **Securities Law:** Determine if GHC is a security in relevant jurisdictions
- **Tax Compliance:** Understand monthly vs. annual reporting requirements
- **DAO Structure:** Confirm legal entity status and obligations

### Accounting Consultation Recommended

- **Financial Reporting:** Determine if calendar alignment is necessary for your business
- **Audit Requirements:** Understand auditor expectations
- **Tax Preparation:** Simplify year-end tax filing

### Regulatory References

- **USA - SEC:** [Framework for "Investment Contract" Analysis of Digital Assets](https://www.sec.gov/corpfin/framework-investment-contract-analysis-digital-assets)
- **USA - IRS:** [Notice 2014-21 (Virtual Currency Guidance)](https://www.irs.gov/pub/irs-drop/n-14-21.pdf)
- **Switzerland - FINMA:** [Guidelines for ICO Inquiries](https://www.finma.ch/en/finma/finma-world/publications/)
- **Wyoming - DAO LLC:** [Wyoming DAO Supplement](https://wyoleg.gov/Legislation/2021/SF0038)

---

**Document Version:** 1.0  
**Next Review:** Upon legal/accounting consultation completion
