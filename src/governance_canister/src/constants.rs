// ============================================================================
// GOVERNANCE CONSTANTS (Defaults - can be modified via admin proposals)
// ============================================================================

/// Default minimum voting power required to create a proposal
pub const DEFAULT_MIN_VOTING_POWER_TO_PROPOSE: u64 = 150 * 100_000_000; // 150 tokens in e8s

/// Default support threshold (voting power needed to move from Proposed to Active)
pub const DEFAULT_SUPPORT_THRESHOLD: u64 = 15_000 * 100_000_000; // 15,000 tokens in e8s

/// Default approval percentage of total staked tokens (30%)
/// Proposals need at least this percentage of YES votes to pass
pub const DEFAULT_APPROVAL_PERCENTAGE: u8 = 30;

/// Default support period for proposals in Proposed state: 1 week in nanoseconds
pub const DEFAULT_SUPPORT_PERIOD_NANOS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;

/// Default voting period duration: 2 weeks in nanoseconds
pub const DEFAULT_VOTING_PERIOD_NANOS: u64 = 14 * 24 * 60 * 60 * 1_000_000_000;

/// Default cooldown before a rejected proposal can be resubmitted: 6 months in nanoseconds
pub const DEFAULT_RESUBMISSION_COOLDOWN_NANOS: u64 = 180 * 24 * 60 * 60 * 1_000_000_000;

/// Nanoseconds per day (for converting days to nanos)
pub const NANOS_PER_DAY: u64 = 24 * 60 * 60 * 1_000_000_000;
