use eyre::{eyre, Result, WrapErr};
use reth_nippy_jar::{compression::Compressors, DataReader, CONFIG_FILE_EXTENSION};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const OFFSETS_FILE_EXTENSION: &str = "off";
const OFFSET_SIZE_BYTES: u8 = 8;

/// Minimal mirror of `reth_nippy_jar::NippyJar` config as it is serialized in `.conf`.
///
/// We need this because the upstream `NippyJar` struct has private fields and no public API to
/// write a config for a jar we build via raw byte copy.
#[derive(Debug, Serialize, Deserialize)]
pub struct NippyJarConfig<H> {
    pub version: usize,
    pub user_header: H,
    pub columns: usize,
    pub rows: usize,
    pub compressor: Option<Compressors>,
    pub max_row_size: usize,
}

pub fn load_config<H: DeserializeOwned>(data_path: &Path) -> Result<NippyJarConfig<H>> {
    let config_path = data_path.with_extension(CONFIG_FILE_EXTENSION);
    let file = File::open(&config_path)
        .wrap_err_with(|| format!("failed to open nippy config {}", config_path.display()))?;
    bincode::deserialize_from(file)
        .wrap_err_with(|| format!("failed to decode nippy config {}", config_path.display()))
}

pub struct SegmentRawSource<H> {
    data_path: PathBuf,
    config: NippyJarConfig<H>,
    reader: DataReader,
}

impl<H: DeserializeOwned> SegmentRawSource<H> {
    pub fn open(data_path: &Path) -> Result<Self> {
        let config = load_config(data_path)?;
        let reader = DataReader::new(data_path).wrap_err_with(|| {
            format!("failed to open nippy data reader {}", data_path.display())
        })?;
        Ok(Self {
            data_path: data_path.to_path_buf(),
            config,
            reader,
        })
    }

    pub const fn config(&self) -> &NippyJarConfig<H> {
        &self.config
    }

    /// Returns the stored (potentially compressed) bytes for the given row, if present.
    pub fn row_bytes(&self, row: usize) -> Result<Option<&[u8]>> {
        if self.config.columns != 1 {
            return Err(eyre!(
                "unsupported nippy jar columns={} for {} (expected 1)",
                self.config.columns,
                self.data_path.display()
            ));
        }
        if row >= self.config.rows {
            return Ok(None);
        }

        let offset_pos = row; // 1 column => offset position is the row index.
        let start = self.reader.offset(offset_pos)? as usize;
        let end = self.reader.offset(offset_pos + 1)? as usize;
        let bytes = self.reader.data(start..end);
        if bytes.is_empty() {
            return Ok(None);
        }
        Ok(Some(bytes))
    }
}

pub struct SegmentRawWriter<H> {
    data_path: PathBuf,
    data_file: BufWriter<File>,
    offsets_file: BufWriter<File>,
    config: NippyJarConfig<H>,
    offset: u64,
}

impl<H: Serialize> SegmentRawWriter<H> {
    pub fn create(data_path: &Path, mut config: NippyJarConfig<H>) -> Result<Self> {
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("failed to create {}", parent.display()))?;
        }
        // Always rebuild from scratch.
        if data_path.exists() {
            fs::remove_file(data_path)
                .wrap_err_with(|| format!("failed to remove {}", data_path.display()))?;
        }
        let offsets_path = data_path.with_extension(OFFSETS_FILE_EXTENSION);
        if offsets_path.exists() {
            fs::remove_file(&offsets_path)
                .wrap_err_with(|| format!("failed to remove {}", offsets_path.display()))?;
        }
        let config_path = data_path.with_extension(CONFIG_FILE_EXTENSION);
        if config_path.exists() {
            fs::remove_file(&config_path)
                .wrap_err_with(|| format!("failed to remove {}", config_path.display()))?;
        }

        let data_file = BufWriter::new(
            File::create(data_path)
                .wrap_err_with(|| format!("failed to create {}", data_path.display()))?,
        );
        let mut offsets_file = BufWriter::new(
            File::create(&offsets_path)
                .wrap_err_with(|| format!("failed to create {}", offsets_path.display()))?,
        );

        // Offsets file: [offset_size][offset_0=0][offset_1][...]
        offsets_file
            .write_all(&[OFFSET_SIZE_BYTES])
            .wrap_err("failed to write offset size")?;
        offsets_file
            .write_all(&0u64.to_le_bytes())
            .wrap_err("failed to write initial offset")?;

        config.version = 1;
        config.columns = 1;
        config.rows = 0;

        Ok(Self {
            data_path: data_path.to_path_buf(),
            data_file,
            offsets_file,
            config,
            offset: 0,
        })
    }

    pub fn push_bytes(&mut self, bytes: &[u8], uncompressed_len: usize) -> Result<()> {
        self.data_file.write_all(bytes)?;
        self.offset = self.offset.saturating_add(bytes.len() as u64);
        self.offsets_file.write_all(&self.offset.to_le_bytes())?;
        self.config.rows = self.config.rows.saturating_add(1);
        self.config.max_row_size = self.config.max_row_size.max(uncompressed_len);
        Ok(())
    }

    pub fn push_empty(&mut self) -> Result<()> {
        self.offsets_file.write_all(&self.offset.to_le_bytes())?;
        self.config.rows = self.config.rows.saturating_add(1);
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.data_file.flush()?;
        self.offsets_file.flush()?;

        let config_path = self.data_path.with_extension(CONFIG_FILE_EXTENSION);
        let file = File::create(&config_path)
            .wrap_err_with(|| format!("failed to create {}", config_path.display()))?;
        bincode::serialize_into(file, &self.config)
            .wrap_err_with(|| format!("failed to encode {}", config_path.display()))?;
        Ok(())
    }
}
