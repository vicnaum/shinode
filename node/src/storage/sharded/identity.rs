//! Read-only identity checks shared by startup, resume, and maintenance.

use super::{load_meta, meta_path, static_dir, validate_meta, MetaState, Storage, SCHEMA_VERSION};
use crate::chain::ChainKind;
use crate::{cli::NodeConfig, storage::StorageCodec};
use eyre::{eyre, Result};
use std::{fs, path::Path};

pub(super) fn stored_chain(meta: &MetaState) -> Result<ChainKind> {
    eyre::ensure!(
        meta.schema_version == SCHEMA_VERSION,
        "unsupported storage schema {}",
        meta.schema_version
    );
    eyre::ensure!(meta.shard_size > 0, "stored shard size must be positive");
    let chain = match meta.chain_id {
        1 => ChainKind::Mainnet,
        8453 => ChainKind::Base,
        id => {
            return Err(eyre!(
            "unsupported or legacy-mislabeled storage chain ID {id}; use a separate data directory"
        ))
        }
    };
    match meta.genesis_hash {
        Some(hash) => eyre::ensure!(hash == chain.genesis_hash(), "storage genesis does not match chain ID; use a separate data directory"),
        None => eyre::ensure!(chain == ChainKind::Mainnet, "legacy storage is Ethereum data, even if labelled with a Base chain ID; use a separate data directory"),
    }
    let codec = meta.storage_codec.unwrap_or(StorageCodec::EthereumV1);
    eyre::ensure!(
        codec == StorageCodec::for_chain(chain),
        "storage codec mismatch for {}; use a separate data directory",
        chain.name()
    );
    Ok(chain)
}

pub(super) fn existing_meta(data_dir: &Path) -> Result<Option<MetaState>> {
    let path = meta_path(data_dir);
    if path.try_exists()? {
        let meta = load_meta(&path)?;
        stored_chain(&meta)?;
        return Ok(Some(meta));
    }
    let static_path = static_dir(data_dir);
    if static_path.try_exists()? {
        eyre::ensure!(fs::read_dir(static_path)?.next().is_none(), "storage metadata is missing but static data exists; restore meta.json before opening or repairing this directory");
    }
    Ok(None)
}

pub(super) fn validate_config_identity(config: &NodeConfig) -> Result<()> {
    eyre::ensure!(
        config.chain_id == config.chain.chain_id()
            && config.genesis_hash == config.chain.genesis_hash(),
        "configuration identity does not match the selected chain"
    );
    eyre::ensure!(config.shard_size > 0, "shard size must be positive");
    Ok(())
}

impl Storage {
    /// Resolve maintenance against metadata without opening or recovering shards.
    pub fn configure_maintenance(config: &mut NodeConfig, explicit_chain: bool) -> Result<()> {
        if let Some(meta) = existing_meta(&config.data_dir)? {
            let chain = stored_chain(&meta)?;
            eyre::ensure!(!explicit_chain || config.chain == chain, "selected chain {} does not match stored {}; use the stored chain or a separate data directory", config.chain.name(), chain.name());
            config.chain = chain;
            config.chain_id = chain.chain_id();
            config.genesis_hash = chain.genesis_hash();
            config.shard_size = meta.shard_size;
            validate_meta(config, &meta)?;
        }
        Ok(())
    }

    /// Validate existing storage before callers create logs, caches, or recover files.
    pub fn validate_existing(config: &NodeConfig) -> Result<()> {
        validate_config_identity(config)?;
        if let Some(meta) = existing_meta(&config.data_dir)? {
            validate_meta(config, &meta)?;
        }
        Ok(())
    }
}
