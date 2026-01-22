use crc32fast::Hasher;
use eyre::{Result, WrapErr};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug)]
pub struct WalRecord {
    pub block_number: u64,
    pub payload: Vec<u8>,
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
        records.push(WalRecord { block_number, payload });
        last_good_offset = file.stream_position()?;
    }

    if last_good_offset < file.metadata()?.len() {
        file.set_len(last_good_offset)?;
        file.seek(SeekFrom::End(0))?;
    }

    Ok(records)
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
        let file = OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("open");
        file.set_len(truncated).expect("truncate");

        let records = read_records(&path).expect("read records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].block_number, 1);

        let _ = fs::remove_file(&path);
    }
}
