//! Lightweight metrics helpers.

use std::ops::RangeInclusive;
use std::time::Duration;

#[allow(dead_code)]
pub fn lag_to_head(head_seen: Option<u64>, last_indexed: Option<u64>) -> Option<u64> {
    match (head_seen, last_indexed) {
        (Some(head), Some(indexed)) => Some(head.saturating_sub(indexed)),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn rate_per_sec(count: u64, elapsed: Duration) -> Option<f64> {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 {
        Some(count as f64 / secs)
    } else {
        None
    }
}

pub fn range_len(range: &RangeInclusive<u64>) -> u64 {
    range.end().saturating_sub(*range.start()).saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_to_head_handles_missing_values() {
        assert_eq!(lag_to_head(None, Some(1)), None);
        assert_eq!(lag_to_head(Some(2), None), None);
        assert_eq!(lag_to_head(Some(10), Some(7)), Some(3));
        assert_eq!(lag_to_head(Some(7), Some(10)), Some(0));
    }

    #[test]
    fn rate_per_sec_handles_zero_duration() {
        assert_eq!(rate_per_sec(10, Duration::from_secs(0)), None);
        let rate = rate_per_sec(10, Duration::from_secs(2)).expect("rate");
        assert!((rate - 5.0).abs() < 1e-6);
    }

    #[test]
    fn range_len_counts_inclusive() {
        assert_eq!(range_len(&(5..=5)), 1);
        assert_eq!(range_len(&(5..=7)), 3);
    }
}
