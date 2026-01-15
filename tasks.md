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

## Task 3: Refactor Operational Governance into Separate Canisters

### Description
Split the current `operational_governance` canister into two specialized canisters: `governance_canister` and `treasury_canister`. This improves separation of concerns, security, and maintainability.

### Proposed Structure

#### Governance Canister
Responsibilities:
- Proposal creation, voting, and approval logic
- Board member management (add, remove, update shares)
- Voting power calculations
- Proposal state management (Proposed → Active → Approved/Rejected)
- Configuration management

#### Treasury Canister
Responsibilities:
- Token custody and transfers
- Execution of approved treasury proposals
- Balance tracking (treasury, founders, etc.)
- Token allocation and distribution
- Financial reporting and queries

### Tips & Considerations
- Inter-canister communication: governance canister calls treasury canister to execute approved proposals
- Treasury canister should ONLY accept execution calls from governance canister (verify caller principal)
- Migration strategy: deploy new canisters, migrate data, update frontend
- Benefits: clearer code organization, better security isolation, easier testing, allows independent upgrades
- See `docs/MULTISIG_EXPLAINED.md` for why this makes multi-sig easier to implement

### Questions to Clarify
- Should the treasury canister be controlled by the governance canister (as controller), or just verify caller identity?
- How should we handle the data migration from the existing unified canister?
- Should we keep the old canister for historical data, or migrate everything?
- What inter-canister call patterns should we use? (async/await, one-way calls, etc.)
- Should board member data live in governance, treasury, or both?

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

## Priority Recommendations

Based on security and functionality needs:

1. **High Priority**: Task 2 (Multi-signature for board members) - Security foundation
2. **High Priority**: Task 4A & 4B (Remove/Update board member proposals) - Critical governance functions
3. **Medium Priority**: Task 1 (Multi-sig for execution) - Enhanced security for treasury
4. **Medium Priority**: Task 3 (Canister refactoring) - Improves architecture but can be done incrementally
5. **Low Priority**: Task 4C-F (Additional board member features) - Nice-to-have features

---

## Notes

- Tasks 1 and 2 are related to multi-signature functionality covered in `docs/MULTISIG_EXPLAINED.md`
- Task 3 refactoring will make Tasks 1 and 2 easier to implement correctly
- Review `docs/ADMIN_ACTIONS.md` for current admin/board member capabilities
- Check conversation history for context on governance system design decisions
