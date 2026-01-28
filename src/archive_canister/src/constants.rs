/// Maximum entries before archive is considered full
/// 3 billion entries ≈ 300GB (100 bytes per entry) - Theoretical limit
/// In practice, we'll cap it lower for safety, say 1M for now during dev/test
pub const MAX_ENTRIES: u64 = 1_000_000;
