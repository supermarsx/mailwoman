//! Shared bounded time-range labels used by operational charts.
pub fn parse_range(range: &str) -> Option<u64> {
    match range {
        "15m" => Some(15 * 60),
        "1h" => Some(60 * 60),
        "6h" => Some(6 * 60 * 60),
        "24h" => Some(24 * 60 * 60),
        _ => None,
    }
}
