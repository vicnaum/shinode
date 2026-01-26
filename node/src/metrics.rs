//! Lightweight metrics helpers.

use std::ops::RangeInclusive;
use std::sync::Mutex;

pub fn range_len(range: &RangeInclusive<u64>) -> u64 {
    range.end().saturating_sub(*range.start()).saturating_add(1)
}

pub fn rate_per_sec(value: u64, elapsed_ms: u64) -> f64 {
    if elapsed_ms == 0 {
        return 0.0;
    }
    value as f64 / (elapsed_ms as f64 / 1000.0)
}

pub fn percentile_triplet(values: &Mutex<Vec<u64>>) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mut data = match values.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => Vec::new(),
    };
    if data.is_empty() {
        return (None, None, None);
    }
    data.sort_unstable();
    let p50 = percentile(&data, 0.50);
    let p95 = percentile(&data, 0.95);
    let p99 = percentile(&data, 0.99);
    (p50, p95, p99)
}

pub fn percentile(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let clamped = p.clamp(0.0, 1.0);
    let idx = ((sorted.len() - 1) as f64 * clamped).round() as usize;
    sorted.get(idx).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_len_counts_inclusive() {
        assert_eq!(range_len(&(5..=5)), 1);
        assert_eq!(range_len(&(5..=7)), 3);
    }
}
