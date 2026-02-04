use eyre::{Result, WrapErr};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Bitset {
    bytes: Vec<u8>,
    size_bits: usize,
}

impl Bitset {
    pub fn new(size_bits: usize) -> Self {
        let bytes_len = size_bits.div_ceil(8);
        Self {
            bytes: vec![0u8; bytes_len],
            size_bits,
        }
    }

    pub fn load(path: &Path, size_bits: usize) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new(size_bits));
        }
        let bytes = fs::read(path).wrap_err("failed to read present.bitset")?;
        let expected_len = size_bits.div_ceil(8);
        if bytes.len() != expected_len {
            return Err(eyre::eyre!(
                "bitset size mismatch: expected {}, got {}",
                expected_len,
                bytes.len()
            ));
        }
        Ok(Self { bytes, size_bits })
    }

    /// Create a bitset from raw bytes (for cache loading).
    ///
    /// # Panics
    /// Panics if `bytes.len()` does not match `size_bits.div_ceil(8)`.
    pub fn from_bytes(bytes: Vec<u8>, size_bits: usize) -> Self {
        let expected_len = size_bits.div_ceil(8);
        assert_eq!(
            bytes.len(),
            expected_len,
            "bitset size mismatch: expected {expected_len}, got {}",
            bytes.len()
        );
        Self { bytes, size_bits }
    }

    pub fn flush(&self, path: &Path) -> Result<()> {
        fs::write(path, &self.bytes).wrap_err("failed to write present.bitset")
    }

    pub fn is_set(&self, offset: usize) -> bool {
        if offset >= self.size_bits {
            return false;
        }
        let byte = self.bytes[offset / 8];
        let bit = 1u8 << (offset % 8);
        byte & bit != 0
    }

    /// Sets the bit and returns true if it changed from 0 -> 1.
    pub fn set(&mut self, offset: usize) -> bool {
        if offset >= self.size_bits {
            return false;
        }
        let idx = offset / 8;
        let bit = 1u8 << (offset % 8);
        let was_set = (self.bytes[idx] & bit) != 0;
        if !was_set {
            self.bytes[idx] |= bit;
        }
        !was_set
    }

    /// Clears the bit and returns true if it changed from 1 -> 0.
    pub fn clear(&mut self, offset: usize) -> bool {
        if offset >= self.size_bits {
            return false;
        }
        let idx = offset / 8;
        let bit = 1u8 << (offset % 8);
        let was_set = (self.bytes[idx] & bit) != 0;
        if was_set {
            self.bytes[idx] &= !bit;
        }
        was_set
    }

    pub fn count_ones(&self) -> u32 {
        self.bytes.iter().map(|b| b.count_ones()).sum()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!("stateless-history-bitset-{now}-{suffix}"));
        path
    }

    #[test]
    fn bitset_set_clear_and_count() {
        let mut bitset = Bitset::new(10);
        assert_eq!(bitset.count_ones(), 0);
        assert!(bitset.set(3));
        assert!(bitset.is_set(3));
        assert_eq!(bitset.count_ones(), 1);
        assert!(bitset.clear(3));
        assert!(!bitset.is_set(3));
        assert_eq!(bitset.count_ones(), 0);
    }

    #[test]
    fn bitset_flush_and_load() {
        let path = temp_path();
        let mut bitset = Bitset::new(16);
        bitset.set(5);
        bitset.flush(&path).expect("flush");

        let loaded = Bitset::load(&path, 16).expect("load");
        assert!(loaded.is_set(5));

        let _ = std::fs::remove_file(&path);
    }
}
