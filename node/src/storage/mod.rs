//! Storage bootstrap and metadata.

use crate::cli::NodeConfig;
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

#[derive(Debug)]
pub struct Storage {
    db: DatabaseEnv,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StoredConfig {
    chain_id: u64,
    data_dir: PathBuf,
    rpc_bind: SocketAddr,
}

impl From<&NodeConfig> for StoredConfig {
    fn from(config: &NodeConfig) -> Self {
        Self {
            chain_id: config.chain_id,
            data_dir: config.data_dir.clone(),
            rpc_bind: config.rpc_bind,
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

        match schema_bytes {
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
                tx.commit()?;
                info!(db_path = %db_path.display(), "initialized storage metadata");
                Ok(())
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

                tx.commit()?;
                Ok(())
            }
        }
    }
}

fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).wrap_err("failed to serialize metadata")
}

fn decode_json<T: DeserializeOwned>(bytes: Vec<u8>) -> Result<T> {
    serde_json::from_slice(&bytes).wrap_err("failed to deserialize metadata")
}
