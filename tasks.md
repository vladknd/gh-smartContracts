# Development Tasks

Last Updated: 2026-01-15

---

## Task 1: Implement Multi-Signature for Proposal Execution

### Description
Add multi-signature functionality for executing approved proposals. Currently, any board member can execute an approved proposal independently. This should require multiple board member signatures before execution.

### Tips & Considerations
- Multi-sig execution is different from multi-sig voting - voting already works, but execution needs additional approval
- Consider implementing a threshold system (e.g., 2-of-3, 3-of-5)
- On Internet Computer, the canister itself holds funds, so multi-sig is about authorization logic, not wallet sharing
- Implementation approach: Add an `execution_approvals` field to proposals, requiring M-of-N board members to call `approve_execution()` before final execution
- See `docs/MULTISIG_EXPLAINED.md` and `docs/MULTISIG_IMPLEMENTATION_OPTIONS.md` for implementation details

### Questions to Clarify
- What should the multi-sig threshold be? (e.g., 2-of-3, 3-of-5, majority of board members?)
- Should the threshold be configurable via governance proposal?
- Should different proposal types require different thresholds? (e.g., high-value treasury proposals need more signatures)
- What happens if board member composition changes after proposal approval but before execution?

---

## Task 2: Multi-Signature for Board Member Principals

### Description
Enable each board member to register multiple wallet principals for enhanced security. If one wallet is compromised or lost, board members can still participate using their other registered principals.

### Tips & Considerations
- Each board member should have 2-5 registered principals (wallets)
- All principals belonging to a board member should share the same voting power and permissions
- Implement principal management: add principal, remove principal, set primary principal
- Consider requiring multi-sig from existing principals to add/remove new principals (prevents single wallet compromise from taking over)
- Store principal-to-board-member mapping in a stable data structure
- Update all governance functions to look up board member ID from any of their registered principals

### Questions to Clarify
- How many principals should each board member be allowed to register? (Recommended: 2-5)
- Should adding a new principal to a board member account require approval from other existing principals?
- What happens if a board member loses access to ALL their registered principals?
- Should there be a "primary" principal with special privileges, or should all principals be equal?
- Should removal of a principal require multi-sig, or can any single principal remove others?

---

## Task 3: Refactor Operational Governance into Separate Canisters ✅ COMPLETED

### Description
Split the legacy `operational_governance` canister into two specialized canisters: `governance_canister` and `treasury_canister`. This improves separation of concerns, security, and maintainability.

**STATUS: COMPLETED** - The `operational_governance` canister has been removed. The system now uses:
- `governance_canister` - Proposals, voting, board member management
- `treasury_canister` - Token custody, transfers, MMCR

Additionally, content governance has been integrated:
- `media_assets` - Permanent media file storage
- `staging_assets` - Temporary content before governance approval
- `learning_engine` - Content storage with version history

### Proposed Structure

#### Governance Canister ✅ Implemented
Responsibilities:
- Proposal creation, voting, and approval logic
- Board member management (add, remove, update shares)
- Voting power calculations
- Proposal state management (Proposed → Active → Approved/Rejected)
- Configuration management
- Content governance proposals (add content, update quiz config, delete content)

#### Treasury Canister ✅ Implemented
Responsibilities:
- Token custody and transfers
- Execution of approved treasury proposals
- Balance tracking
- MMCR (Monthly Maximum Cumulative Release)
- Financial reporting and queries

### Completed Work
- ✅ Created `governance_canister` with full proposal system
- ✅ Created `treasury_canister` with token management
- ✅ Removed `operational_governance` canister completely
- ✅ Created `media_assets` canister for permanent media storage
- ✅ Created `staging_assets` canister for content staging
- ✅ Updated `learning_engine` with content governance integration
- ✅ Updated deployment script with all canister linkages
- ✅ Updated frontend configuration

---

## Task 4: Expand Board Member Management Proposals

### Description
Add comprehensive board member management functionality through governance proposals. Currently limited board member operations exist - expand this to full lifecycle management.

### Proposed Functions

#### A. Remove Board Member
- Proposal type: `RemoveBoardMember`
- Input: `board_member_id` or `principal`
- Redistributes removed member's voting shares proportionally to remaining members
- Handles edge cases: removing last member, removing self, removing during active proposals

#### B. Update Board Member Shares
- Proposal type: `UpdateBoardMemberShares`
- Input: `board_member_id`, `new_share_amount`
- Recalculates percentages to maintain 100% total
- Validates minimum/maximum share limits

#### C. Update Board Member Metadata
- Proposal type: `UpdateBoardMemberInfo`
- Input: `board_member_id`, optional fields (`name`, `description`, `contact_info`)
- Updates non-critical information without affecting voting power

#### D. Transfer Board Member Role
- Proposal type: `TransferBoardMember`
- Input: `old_principal`, `new_principal`
- Transfers entire board member status to new principal (for account migration)
- Security: requires approval from both old and new principals

#### E. Suspend/Reactivate Board Member
- Proposal type: `SuspendBoardMember` / `ReactivateBoardMember`
- Temporarily removes voting power without removing from board
- Use case: member taking leave, investigation, temporary conflicts of interest

#### F. Set Board Member Quorum
- Proposal type: `UpdateQuorumThreshold`
- Input: `quorum_percentage` (e.g., 51%, 66%)
- Defines minimum board participation needed for proposal validity

### Tips & Considerations
- All board member changes should go through governance proposals (prevent unilateral changes)
- Consider: should board members be able to vote on proposals about themselves? (conflict of interest)
- Share redistribution math must be precise and maintain 100% total
- Handle edge cases: empty board, single board member, ties in voting
- Some actions might need higher thresholds (e.g., removal needs 66% vs normal 51%)

### Questions to Clarify
- Which of these board member functions are highest priority?
- Should board members be able to vote on proposals directly affecting them (e.g., removal, share changes)?
- What should be the minimum number of board members required?
- Should certain board member proposals require higher approval thresholds?
- Should there be a "founder" or "admin" role with special irremovable status?
- Do we need emergency procedures if board becomes deadlocked or non-functional?

---

## Task 5: Multi-Token Treasury Support

### Description
Extend the treasury canister to support multiple token types beyond GHC. Initially support USDC (stablecoin) for ICO proceeds, with architecture to easily add more tokens (ICP, ckBTC, etc.) in the future.

### Tips & Considerations
- Use a `HashMap<TokenType, Balance>` or similar structure to track multiple token balances
- Each token type needs its own ledger canister principal for transfers
- Implement token-agnostic transfer functions that route to appropriate ledger
- Treasury proposals should specify token type for transfers (e.g., "Send 1000 USDC to X" vs "Send 100 GHC to Y")
- Add query functions to get treasury balance by token type
- Consider implementing token swap proposals (exchange X amount of token A for token B via DEX)
- Track token inflows/outflows separately for accounting and reporting
- Integration with ICRC-1 or ICRC-2 token standards for compatibility

### Token Types to Support (Priority Order)
1. **GHC** (native token) - already implemented
2. **USDC** (stablecoin) - needed for ICO
3. **ICP** (Internet Computer native) - ecosystem integration
4. **ckBTC** (wrapped Bitcoin) - future treasury diversification
5. **ckETH** (wrapped Ethereum) - future treasury diversification

### Questions to Clarify
- Should the treasury support any ICRC-1 token, or only whitelisted tokens approved via governance?
- How should token balances be displayed in the frontend dashboard?
- Should there be separate governance thresholds for different token types? (e.g., spending USDC requires higher approval)
- Do we need exchange rate oracles to value different tokens in a common denominator?
- Should token deposits require approval, or can anyone send tokens to treasury?
- How should we handle token standards (ICRC-1, ICRC-2, DIP-20)?

---

## Task 6: Create ICO Canister

### Description
Build a dedicated ICO (Initial Coin Offering) canister to conduct the first phase of token sale. Users send USDC to the ICO canister and receive GHC tokens at a fixed price. All USDC proceeds go directly to the treasury.

### Core Functionality

#### Phase 1: Fixed Price Sale
- Set a fixed GHC/USDC exchange rate (e.g., 1 GHC = 0.10 USDC)
- Users call `buy_tokens(usdc_amount)` and receive GHC in return
- Automatically transfer USDC to treasury canister
- Mint or transfer GHC tokens to buyer
- Track total tokens sold and total USDC raised

#### ICO Configuration (via Governance)
- Start time and end time
- Fixed price per token
- Minimum purchase amount (prevent spam)
- Maximum purchase per user (optional - promote fair distribution)
- Total token allocation for sale
- Whitelist mode (optional - restrict to approved participants)

#### Safety Features
- Pause/resume functionality (emergency stop via governance)
- Purchase limits per transaction
- Time-based phases (pre-sale, public sale, etc.)
- Automatic closure when token allocation is sold out
- Refund mechanism if ICO is cancelled

### Tips & Considerations
- ICO canister needs approval to transfer GHC from treasury or have pre-allocated supply
- Requires integration with USDC ledger canister for receiving payments
- Consider implementing purchase history queries for transparency
- Add events/logs for all purchases for audit trail
- Frontend integration: simple "Buy GHC" interface with USDC wallet connection
- Post-ICO: canister can be frozen or repurposed for future sale rounds
- Security: validate all USDC transfers before minting GHC (prevent double-spend)
- Consider vesting schedules for large purchases (optional)

### Questions to Clarify
- What should the fixed price be for GHC in USDC? (e.g., $0.10 per GHC)
- How many GHC tokens should be allocated for the ICO? (e.g., 10% of total supply)
- Should there be a minimum/maximum purchase limit per user?
- Do we need KYC/whitelist for ICO participants, or open to anyone?
- Should the ICO have multiple phases (pre-sale, public sale) with different prices?
- What happens to unsold tokens after ICO ends? (burn, return to treasury, save for future rounds)
- Should early buyers get bonuses/discounts?
- How should we handle failed transactions or refunds?

---

## Task 7: DEX Integration Canister

### Description
Create a canister that integrates with decentralized exchanges (DEX) on the Internet Computer ecosystem. This enables automated token swaps, liquidity provision, and price discovery for GHC and other treasury tokens.

### Core Functionality

#### Swap Integration
- Integrate with major IC DEXes (ICPSwap, Sonic, etc.)
- Execute token swaps via governance proposals (e.g., "Swap 1000 USDC for ICP")
- Query current market prices for supported token pairs
- Calculate slippage and estimated output before swaps

#### Liquidity Management (Future)
- Add/remove liquidity to GHC trading pairs
- Track LP token positions
- Collect trading fees from liquidity pools
- Automated liquidity rebalancing strategies

#### Price Oracle
- Fetch current GHC market price from DEX
- Provide price feeds for treasury valuation
- Historical price tracking for analytics

### Tips & Considerations
- Start with read-only integration (price queries) before implementing swaps
- Each DEX has different APIs - may need adapters for each
- Slippage protection: set maximum acceptable slippage for swaps
- All swaps should go through governance proposals for transparency
- Consider MEV (Maximal Extractable Value) protection
- Test thoroughly on testnet before mainnet integration
- May need to handle failed swaps and retries
- Frontend: display current GHC price, trading volume, liquidity

### DEX Platforms to Consider
1. **ICPSwap** - largest DEX on Internet Computer
2. **Sonic** - high-performance DEX
3. **ICDex** - order book based DEX

### Questions to Clarify
- Which DEX should we integrate with first? (Recommended: ICPSwap)
- Should DEX swaps require governance proposals, or can board members execute directly?
- Do we want to provide liquidity (become a liquidity provider), or just use DEX for swaps?
- Should the system automatically create GHC/USDC and GHC/ICP liquidity pools?
- What percentage of treasury should be available for DEX operations?
- How should we handle price impact on large swaps (split into smaller trades)?
- Should we implement automated trading strategies, or keep it manual via governance?

---

## Priority Recommendations

Based on security and functionality needs:

1. **High Priority**: Task 2 (Multi-signature for board members) - Security foundation
2. **High Priority**: Task 4A & 4B (Remove/Update board member proposals) - Critical governance functions
3. **Medium Priority**: Task 1 (Multi-sig for execution) - Enhanced security for treasury
4. **Medium Priority**: Task 3 (Canister refactoring) - Improves architecture but can be done incrementally
5. **Medium Priority**: Task 5 (Multi-token treasury support) - Needed before ICO
6. **Medium Priority**: Task 6 (ICO canister) - Revenue generation for project
7. **Low Priority**: Task 4C-F (Additional board member features) - Nice-to-have features
8. **Low Priority**: Task 7 (DEX integration) - Useful but not critical initially

---

## Notes

- Tasks 1 and 2 are related to multi-signature functionality covered in `docs/MULTISIG_EXPLAINED.md`
- Task 3 refactoring will make Tasks 1 and 2 easier to implement correctly
- Review `docs/ADMIN_ACTIONS.md` for current admin/board member capabilities
- Check conversation history for context on governance system design decisions
