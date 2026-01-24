use crc32fast::Hasher;
use eyre::{Result, WrapErr};
use memmap2::Mmap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::Path;

#[derive(Debug)]
pub struct WalRecord {
    pub block_number: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct WalIndexEntry {
    pub block_number: u64,
    /// Offset of the record start (block_number field) within the WAL file.
    #[allow(dead_code)]
    pub record_offset: u64,
    /// Length of the record payload (not including the header or CRC trailer).
    #[allow(dead_code)]
    pub payload_len: u32,
}

pub fn append_records(path: &Path, records: &[WalRecord]) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).wrap_err("failed to create wal dir")?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .wrap_err("failed to open staging.wal")?;

    for record in records {
        let mut hasher = Hasher::new();
        hasher.update(&record.block_number.to_le_bytes());
        let len = record.payload.len() as u32;
        hasher.update(&len.to_le_bytes());
        hasher.update(&record.payload);
        let crc = hasher.finalize();

        file.write_all(&record.block_number.to_le_bytes())?;
        file.write_all(&len.to_le_bytes())?;
        file.write_all(&record.payload)?;
        file.write_all(&crc.to_le_bytes())?;
    }
    file.flush()?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_records(path: &Path) -> Result<Vec<WalRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .wrap_err("failed to open staging.wal")?;
    let mut records = Vec::new();
    let mut last_good_offset: u64 = 0;
    let mut buf = [0u8; 8];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        if read < 8 {
            break;
        }
        let block_number = u64::from_le_bytes(buf);
        let mut len_buf = [0u8; 4];
        if file.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        if file.read_exact(&mut payload).is_err() {
            break;
        }
        let mut crc_buf = [0u8; 4];
        if file.read_exact(&mut crc_buf).is_err() {
            break;
        }
        let crc_expected = u32::from_le_bytes(crc_buf);
        let mut hasher = Hasher::new();
        hasher.update(&block_number.to_le_bytes());
        hasher.update(&len_buf);
        hasher.update(&payload);
        let crc_actual = hasher.finalize();
        if crc_actual != crc_expected {
            break;
        }
        records.push(WalRecord {
            block_number,
            payload,
        });
        last_good_offset = file.stream_position()?;
    }

    if last_good_offset < file.metadata()?.len() {
        file.set_len(last_good_offset)?;
        file.seek(SeekFrom::End(0))?;
    }

    Ok(records)
}

/// Scans the WAL and returns record offsets + payload sizes without loading payloads into memory.
///
/// This also truncates an invalid / partial tail (same behavior as [`read_records`]), but it does
/// NOT validate per-record CRCs.
pub fn build_index(path: &Path) -> Result<Vec<WalIndexEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .wrap_err("failed to open staging.wal")?;

    let mut entries = Vec::new();
    let mut last_good_offset: u64 = 0;
    let mut buf = [0u8; 8];
    let mut len_buf = [0u8; 4];

    loop {
        let record_offset = file.stream_position()?;
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        if read < 8 {
            break;
        }

        let block_number = u64::from_le_bytes(buf);
        if file.read_exact(&mut len_buf).is_err() {
            break;
        }
        let payload_len = u32::from_le_bytes(len_buf);

        let file_len = file.metadata()?.len();
        let next_offset = record_offset
            .saturating_add(8)
            .saturating_add(4)
            .saturating_add(payload_len as u64)
            .saturating_add(4);
        if next_offset > file_len {
            break;
        }

        entries.push(WalIndexEntry {
            block_number,
            record_offset,
            payload_len,
        });

        file.seek(SeekFrom::Start(next_offset))?;
        last_good_offset = next_offset;
    }

    if last_good_offset < file.metadata()?.len() {
        file.set_len(last_good_offset)?;
        file.seek(SeekFrom::End(0))?;
    }

    Ok(entries)
}

#[derive(Debug, Clone, Copy)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

impl ByteRange {
    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn as_range(self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalBundleSlices {
    pub header: ByteRange,
    pub tx_hashes: ByteRange,
    pub tx_meta: ByteRange,
    pub tx_meta_uncompressed_len: u32,
    pub receipts: ByteRange,
    pub receipts_uncompressed_len: u32,
    pub size: ByteRange,
}

pub struct WalSliceIndex {
    pub mmap: Mmap,
    /// Indexed by local offset within a shard.
    pub entries: Vec<Option<WalBundleSlices>>,
}

fn read_u64(bytes: &[u8], pos: &mut usize, end: usize) -> Result<u64> {
    if *pos + 8 > end {
        return Err(eyre::eyre!("wal payload truncated while reading u64"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[*pos..*pos + 8]);
    *pos += 8;
    Ok(u64::from_le_bytes(buf))
}

fn read_u32(bytes: &[u8], pos: &mut usize, end: usize) -> Result<u32> {
    if *pos + 4 > end {
        return Err(eyre::eyre!("wal payload truncated while reading u32"));
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[*pos..*pos + 4]);
    *pos += 4;
    Ok(u32::from_le_bytes(buf))
}

fn read_bytes_range(bytes: &[u8], pos: &mut usize, end: usize) -> Result<ByteRange> {
    let len = read_u64(bytes, pos, end)? as usize;
    if *pos + len > end {
        return Err(eyre::eyre!(
            "wal payload truncated while reading bytes (need {}, have {})",
            len,
            end.saturating_sub(*pos)
        ));
    }
    let range = ByteRange {
        start: *pos,
        end: *pos + len,
    };
    *pos += len;
    Ok(range)
}

/// Builds an in-memory index of WAL record slices for a given shard.
///
/// This is a fast-path for compaction: instead of deserializing each record into fresh `Vec<u8>`
/// buffers, we memory-map the WAL file and parse the bincode payload layout to obtain byte ranges
/// for each segment.
///
/// Payload format MUST match the bincode encoding of `WalBundleRecord` in `mod.rs`.
pub fn build_slice_index(
    path: &Path,
    shard_start: u64,
    shard_size: usize,
) -> Result<Option<WalSliceIndex>> {
    if !path.exists() {
        return Ok(None);
    }

    // Ensure we don't have a partial tail record.
    let _ = build_index(path)?;

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .wrap_err("failed to open staging.wal for mmap")?;
    // SAFETY: read-only mapping; file handle outlives mmap via ownership in WalSliceIndex.
    let mmap = unsafe { Mmap::map(&file).wrap_err("failed to mmap staging.wal")? };
    let bytes: &[u8] = &mmap;

    let mut entries: Vec<Option<WalBundleSlices>> = vec![None; shard_size];
    let mut pos: usize = 0;
    let mut any = false;

    while pos + 8 + 4 + 4 <= bytes.len() {
        let record_offset = pos;
        let block_number = {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[pos..pos + 8]);
            u64::from_le_bytes(buf)
        };
        let payload_len = {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[pos + 8..pos + 12]);
            u32::from_le_bytes(buf) as usize
        };

        let payload_start = pos + 12;
        let payload_end = payload_start.saturating_add(payload_len);
        let crc_start = payload_end;
        let next = crc_start.saturating_add(4);
        if next > bytes.len() {
            break;
        }

        let crc_expected = {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[crc_start..crc_start + 4]);
            u32::from_le_bytes(buf)
        };
        let crc_actual = {
            // CRC is computed over (block_number, payload_len, payload bytes) in little-endian.
            let mut hasher = Hasher::new();
            hasher.update(&bytes[record_offset..payload_end]);
            hasher.finalize()
        };
        if crc_actual != crc_expected {
            // Treat as a corrupted tail and stop parsing; earlier records are still usable.
            break;
        }

        // Parse payload (bincode-encoded struct fields).
        let mut p = payload_start;
        let number_in_payload = read_u64(bytes, &mut p, payload_end)?;
        if number_in_payload != block_number {
            return Err(eyre::eyre!(
                "wal payload number mismatch at offset {}: header {} != payload {}",
                record_offset,
                block_number,
                number_in_payload
            ));
        }

        let header = read_bytes_range(bytes, &mut p, payload_end)?;
        let tx_hashes = read_bytes_range(bytes, &mut p, payload_end)?;
        let tx_meta = read_bytes_range(bytes, &mut p, payload_end)?;
        let tx_meta_uncompressed_len = read_u32(bytes, &mut p, payload_end)?;
        let receipts = read_bytes_range(bytes, &mut p, payload_end)?;
        let receipts_uncompressed_len = read_u32(bytes, &mut p, payload_end)?;
        let size = read_bytes_range(bytes, &mut p, payload_end)?;

        // Ensure we consumed the full payload (no trailing bytes).
        if p != payload_end {
            return Err(eyre::eyre!(
                "wal payload decode mismatch at offset {}: expected end {}, got {}",
                record_offset,
                payload_end,
                p
            ));
        }

        if block_number >= shard_start {
            let local = (block_number - shard_start) as usize;
            if local < shard_size {
                entries[local] = Some(WalBundleSlices {
                    header,
                    tx_hashes,
                    tx_meta,
                    tx_meta_uncompressed_len,
                    receipts,
                    receipts_uncompressed_len,
                    size,
                });
                any = true;
            }
        }

        pos = next;
    }

    if !any {
        return Ok(None);
    }
    Ok(Some(WalSliceIndex { mmap, entries }))
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
        path.push(format!("stateless-history-wal-{now}-{suffix}"));
        path
    }

    #[test]
    fn wal_truncates_partial_tail() {
        let path = temp_path();
        append_records(
            &path,
            &[
                WalRecord {
                    block_number: 1,
                    payload: b"hello".to_vec(),
                },
                WalRecord {
                    block_number: 2,
                    payload: b"world".to_vec(),
                },
            ],
        )
        .expect("append records");

        let len = fs::metadata(&path).expect("meta").len();
        let truncated = len.saturating_sub(3);
        let file = OpenOptions::new().write(true).open(&path).expect("open");
        file.set_len(truncated).expect("truncate");

        let records = read_records(&path).expect("read records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].block_number, 1);

        let _ = fs::remove_file(&path);
    }
}
