//! Storage bootstrap and metadata.

use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
use eyre::{eyre, Result, WrapErr};
use reth_db::{
    mdbx::{init_db_for, DatabaseArguments, DatabaseEnv},
    ClientVersion, Database,
};
use reth_db_api::transaction::{DbTx, DbTxMut};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tracing::info;

mod tables {
    use reth_db_api::{table::TableInfo, tables, TableSet, TableType, TableViewer};
    use std::fmt;

    tables! {
        /// Stateless history node metadata.
        table Meta {
            type Key = String;
            type Value = Vec<u8>;
        }
    }
}

const SCHEMA_VERSION: u64 = 1;
const META_SCHEMA_VERSION_KEY: &str = "schema_version";
const META_CHAIN_ID_KEY: &str = "chain_id";
const META_CONFIG_KEY: &str = "config";
const META_LAST_INDEXED_BLOCK_KEY: &str = "last_indexed_block";
const META_HEAD_SEEN_KEY: &str = "head_seen";

#[derive(Debug)]
pub struct Storage {
    db: DatabaseEnv,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct StoredConfig {
    chain_id: u64,
    data_dir: PathBuf,
    rpc_bind: SocketAddr,
    start_block: u64,
    rollback_window: u64,
    retention_mode: RetentionMode,
    head_source: HeadSource,
    reorg_strategy: ReorgStrategy,
}

impl From<&NodeConfig> for StoredConfig {
    fn from(config: &NodeConfig) -> Self {
        Self {
            chain_id: config.chain_id,
            data_dir: config.data_dir.clone(),
            rpc_bind: config.rpc_bind,
            start_block: config.start_block,
            rollback_window: config.rollback_window,
            retention_mode: config.retention_mode,
            head_source: config.head_source,
            reorg_strategy: config.reorg_strategy,
        }
    }
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            data_dir: PathBuf::from("data"),
            rpc_bind: "127.0.0.1:8545".parse().expect("valid default rpc bind"),
            start_block: 0,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
        }
    }
}

impl Storage {
    /// Open the MDBX environment and bootstrap metadata if needed.
    pub fn open(config: &NodeConfig) -> Result<Self> {
        let db_path = config.data_dir.join("db");
        let args = DatabaseArguments::new(ClientVersion::default());
        let db = init_db_for::<_, tables::Tables>(&db_path, args)
            .wrap_err("failed to open MDBX environment")?;
        let storage = Self { db };
        storage.bootstrap(config, &db_path)?;
        Ok(storage)
    }

    fn bootstrap(&self, config: &NodeConfig, db_path: &Path) -> Result<()> {
        let tx = self.db.tx()?;
        let schema_bytes = tx.get::<tables::Meta>(META_SCHEMA_VERSION_KEY.to_string())?;
        tx.commit()?;

        let (needs_last_indexed, needs_head_seen) = match schema_bytes {
            None => {
                let tx = self.db.tx_mut()?;
                tx.put::<tables::Meta>(
                    META_SCHEMA_VERSION_KEY.to_string(),
                    encode_json(&SCHEMA_VERSION)?,
                )?;
                tx.put::<tables::Meta>(
                    META_CHAIN_ID_KEY.to_string(),
                    encode_json(&config.chain_id)?,
                )?;
                tx.put::<tables::Meta>(
                    META_CONFIG_KEY.to_string(),
                    encode_json(&StoredConfig::from(config))?,
                )?;
                tx.put::<tables::Meta>(
                    META_LAST_INDEXED_BLOCK_KEY.to_string(),
                    encode_json(&Option::<u64>::None)?,
                )?;
                tx.put::<tables::Meta>(
                    META_HEAD_SEEN_KEY.to_string(),
                    encode_json(&Option::<u64>::None)?,
                )?;
                tx.commit()?;
                info!(db_path = %db_path.display(), "initialized storage metadata");
                return Ok(());
            }
            Some(bytes) => {
                let schema_version: u64 = decode_json(bytes)?;
                if schema_version != SCHEMA_VERSION {
                    return Err(eyre!(
                        "unsupported schema version {schema_version} (expected {SCHEMA_VERSION})"
                    ));
                }

                let tx = self.db.tx()?;
                let chain_bytes = tx
                    .get::<tables::Meta>(META_CHAIN_ID_KEY.to_string())?
                    .ok_or_else(|| eyre!("missing chain_id metadata"))?;
                let chain_id: u64 = decode_json(chain_bytes)?;
                if chain_id != config.chain_id {
                    return Err(eyre!(
                        "chain_id mismatch: db={chain_id} config={}",
                        config.chain_id
                    ));
                }

                let config_bytes = tx
                    .get::<tables::Meta>(META_CONFIG_KEY.to_string())?
                    .ok_or_else(|| eyre!("missing config metadata"))?;
                let stored_config: StoredConfig = decode_json(config_bytes)?;
                let expected = StoredConfig::from(config);
                if stored_config != expected {
                    return Err(eyre!("config mismatch: db={stored_config:?} config={expected:?}"));
                }

                let last_indexed = tx.get::<tables::Meta>(META_LAST_INDEXED_BLOCK_KEY.to_string())?;
                let head_seen = tx.get::<tables::Meta>(META_HEAD_SEEN_KEY.to_string())?;
                tx.commit()?;
                (last_indexed.is_none(), head_seen.is_none())
            }
        };

        if needs_last_indexed || needs_head_seen {
            let tx = self.db.tx_mut()?;
            if needs_last_indexed {
                tx.put::<tables::Meta>(
                    META_LAST_INDEXED_BLOCK_KEY.to_string(),
                    encode_json(&Option::<u64>::None)?,
                )?;
            }
            if needs_head_seen {
                tx.put::<tables::Meta>(
                    META_HEAD_SEEN_KEY.to_string(),
                    encode_json(&Option::<u64>::None)?,
                )?;
            }
            tx.commit()?;
        }

        Ok(())
    }

    /// Returns the last fully indexed block, if any.
    pub fn last_indexed_block(&self) -> Result<Option<u64>> {
        self.read_optional_u64(META_LAST_INDEXED_BLOCK_KEY)
    }

    /// Persist the last fully indexed block.
    #[allow(dead_code)]
    pub fn set_last_indexed_block(&self, value: u64) -> Result<()> {
        self.write_optional_u64(META_LAST_INDEXED_BLOCK_KEY, Some(value))
    }

    /// Returns the latest head observed from the head source.
    pub fn head_seen(&self) -> Result<Option<u64>> {
        self.read_optional_u64(META_HEAD_SEEN_KEY)
    }

    /// Persist the latest head observed from the head source.
    #[allow(dead_code)]
    pub fn set_head_seen(&self, value: u64) -> Result<()> {
        self.write_optional_u64(META_HEAD_SEEN_KEY, Some(value))
    }

    fn read_optional_u64(&self, key: &str) -> Result<Option<u64>> {
        let tx = self.db.tx()?;
        let bytes = tx.get::<tables::Meta>(key.to_string())?;
        tx.commit()?;
        match bytes {
            Some(value) => decode_json::<Option<u64>>(value),
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    fn write_optional_u64(&self, key: &str, value: Option<u64>) -> Result<()> {
        let tx = self.db.tx_mut()?;
        tx.put::<tables::Meta>(key.to_string(), encode_json(&value)?)?;
        tx.commit()?;
        Ok(())
    }
}

fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).wrap_err("failed to serialize metadata")
}

fn decode_json<T: DeserializeOwned>(bytes: Vec<u8>) -> Result<T> {
    serde_json::from_slice(&bytes).wrap_err("failed to deserialize metadata")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("stateless-history-node-test-{now}-{}", std::process::id()));
        path
    }

    fn base_config(data_dir: PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
        }
    }

    #[test]
    fn storage_bootstrap_and_config_validation() {
        let dir = temp_dir();
        let config = base_config(dir.clone());

        let storage = Storage::open(&config).expect("open storage");
        assert_eq!(storage.last_indexed_block().unwrap(), None);
        assert_eq!(storage.head_seen().unwrap(), None);
        storage.set_last_indexed_block(10).expect("set last indexed");
        storage.set_head_seen(12).expect("set head seen");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(10));
        assert_eq!(storage.head_seen().unwrap(), Some(12));
        drop(storage);

        let storage_again = Storage::open(&config).expect("reopen with same config");
        assert_eq!(storage_again.last_indexed_block().unwrap(), Some(10));
        assert_eq!(storage_again.head_seen().unwrap(), Some(12));
        drop(storage_again);

        let mut changed = config.clone();
        changed.chain_id = 2;
        let err = Storage::open(&changed).expect_err("chain id mismatch should error");
        let err_string = format!("{err:?}");
        assert!(
            err_string.contains("chain_id mismatch"),
            "unexpected error: {err_string}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
