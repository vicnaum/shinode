//! Canonical chain tracking.

use alloy_primitives::B256;
use std::collections::{BTreeMap, HashMap};

/// Basic head tracker abstraction for v0.1.
pub trait HeadTracker: Send + Sync {
    /// Returns the latest observed head block number, if any.
    fn head(&self) -> Option<u64>;
}

/// In-memory head tracker for early development.
#[derive(Debug, Default, Clone)]
pub struct MemoryHeadTracker {
    head: Option<u64>,
}

impl MemoryHeadTracker {
    pub fn new(head: Option<u64>) -> Self {
        Self { head }
    }

    #[allow(dead_code)]
    pub fn set_head(&mut self, head: u64) {
        self.head = Some(head);
    }
}

impl HeadTracker for MemoryHeadTracker {
    fn head(&self) -> Option<u64> {
        self.head
    }
}

/// Minimal header representation for canonical tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderStub {
    pub number: u64,
    pub hash: B256,
    pub parent_hash: B256,
}

/// Chain update result after inserting a header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainUpdate {
    Initialized { head: HeaderStub },
    Extended { new_head: HeaderStub },
    Reorg {
        ancestor_number: u64,
        removed_numbers: Vec<u64>,
        new_head: HeaderStub,
    },
}

/// Errors for header insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    UnknownParent(B256),
    NonContiguousNumber { expected: u64, got: u64 },
}

/// In-memory canonical chain tracker with reorg detection.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ChainTracker {
    headers: HashMap<B256, HeaderStub>,
    canonical: BTreeMap<u64, B256>,
    canonical_by_hash: HashMap<B256, u64>,
    head: Option<HeaderStub>,
}

#[allow(dead_code)]
impl ChainTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn head(&self) -> Option<HeaderStub> {
        self.head
    }

    pub fn canonical_hash(&self, number: u64) -> Option<B256> {
        self.canonical.get(&number).copied()
    }

    pub fn insert_header(&mut self, header: HeaderStub) -> Result<ChainUpdate, ChainError> {
        self.headers.insert(header.hash, header);

        match self.head {
            None => {
                self.insert_canonical(header.number, header.hash);
                self.head = Some(header);
                return Ok(ChainUpdate::Initialized { head: header });
            }
            Some(current_head) => {
                if header.parent_hash == current_head.hash {
                    let expected = current_head.number + 1;
                    if header.number != expected {
                        return Err(ChainError::NonContiguousNumber {
                            expected,
                            got: header.number,
                        });
                    }
                    self.insert_canonical(header.number, header.hash);
                    self.head = Some(header);
                    return Ok(ChainUpdate::Extended { new_head: header });
                }
            }
        }

        let ancestor_number = match self.canonical_by_hash.get(&header.parent_hash) {
            Some(number) => *number,
            None => return Err(ChainError::UnknownParent(header.parent_hash)),
        };
        let expected = ancestor_number + 1;
        if header.number != expected {
            return Err(ChainError::NonContiguousNumber {
                expected,
                got: header.number,
            });
        }

        let removed_numbers: Vec<u64> = self
            .canonical
            .range((ancestor_number + 1)..)
            .map(|(number, _)| *number)
            .collect();
        for number in &removed_numbers {
            if let Some(hash) = self.canonical.remove(number) {
                self.canonical_by_hash.remove(&hash);
            }
        }

        self.insert_canonical(header.number, header.hash);
        self.head = Some(header);

        Ok(ChainUpdate::Reorg {
            ancestor_number,
            removed_numbers,
            new_head: header,
        })
    }

    fn insert_canonical(&mut self, number: u64, hash: B256) {
        self.canonical.insert(number, hash);
        self.canonical_by_hash.insert(hash, number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_from_u64(value: u64) -> B256 {
        let mut bytes = [0u8; 32];
        bytes[24..].copy_from_slice(&value.to_be_bytes());
        B256::from(bytes)
    }

    #[test]
    fn memory_head_tracker_roundtrip() {
        let mut tracker = MemoryHeadTracker::default();
        assert_eq!(tracker.head(), None);
        tracker.set_head(42);
        assert_eq!(tracker.head(), Some(42));
    }

    #[test]
    fn chain_tracker_extends_canonical() {
        let mut tracker = ChainTracker::new();
        let genesis = HeaderStub {
            number: 0,
            hash: hash_from_u64(1),
            parent_hash: B256::ZERO,
        };
        let one = HeaderStub {
            number: 1,
            hash: hash_from_u64(2),
            parent_hash: hash_from_u64(1),
        };

        assert_eq!(
            tracker.insert_header(genesis),
            Ok(ChainUpdate::Initialized { head: genesis })
        );
        assert_eq!(
            tracker.insert_header(one),
            Ok(ChainUpdate::Extended { new_head: one })
        );
        assert_eq!(tracker.canonical_hash(1), Some(hash_from_u64(2)));
    }

    #[test]
    fn chain_tracker_detects_reorg() {
        let mut tracker = ChainTracker::new();
        let genesis = HeaderStub {
            number: 0,
            hash: hash_from_u64(1),
            parent_hash: B256::ZERO,
        };
        let one = HeaderStub {
            number: 1,
            hash: hash_from_u64(2),
            parent_hash: hash_from_u64(1),
        };
        let two = HeaderStub {
            number: 2,
            hash: hash_from_u64(3),
            parent_hash: hash_from_u64(2),
        };
        let two_alt = HeaderStub {
            number: 2,
            hash: hash_from_u64(4),
            parent_hash: hash_from_u64(2),
        };

        tracker.insert_header(genesis).unwrap();
        tracker.insert_header(one).unwrap();
        tracker.insert_header(two).unwrap();

        let update = tracker.insert_header(two_alt).unwrap();
        assert_eq!(
            update,
            ChainUpdate::Reorg {
                ancestor_number: 1,
                removed_numbers: vec![2],
                new_head: two_alt,
            }
        );
        assert_eq!(tracker.head(), Some(two_alt));
        assert_eq!(tracker.canonical_hash(2), Some(hash_from_u64(4)));
    }

    #[test]
    fn chain_tracker_rejects_unknown_parent() {
        let mut tracker = ChainTracker::new();
        let genesis = HeaderStub {
            number: 0,
            hash: hash_from_u64(1),
            parent_hash: B256::ZERO,
        };
        tracker.insert_header(genesis).unwrap();
        let header = HeaderStub {
            number: 2,
            hash: hash_from_u64(10),
            parent_hash: hash_from_u64(9),
        };
        assert_eq!(
            tracker.insert_header(header),
            Err(ChainError::UnknownParent(hash_from_u64(9)))
        );
    }
}
