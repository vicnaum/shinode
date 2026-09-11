use super::*;
use crate::{chain::ChainKind, test_utils};
use eyre::ensure;

fn stored_peer(id: &str, last_seen_ms: u64) -> Result<StoredPeer> {
    Ok(StoredPeer {
        peer_id: id.to_string(),
        tcp_addr: "127.0.0.1:30303".parse()?,
        udp_addr: Some("127.0.0.1:30304".parse()?),
        last_seen_ms,
        aimd_batch_limit: Some(32),
    })
}

#[test]
fn shared_cache_isolates_chain_identity_and_preserves_legacy_ethereum() -> Result<()> {
    let root = test_utils::temp_dir("peer-cache-chains");
    fs::create_dir_all(&root)?;
    persist_peer_cache(&root, &[stored_peer("legacy", 100)?])?;
    let mut mainnet = test_utils::base_config(root.join("mainnet"));
    mainnet.peer_cache_dir = Some(root.clone());
    let eth = Storage::open(&mainnet)?;
    ensure!(
        eth.peer_cache_snapshot().len() == 1,
        "legacy Ethereum cache was lost"
    );
    eth.flush_peer_cache()?;

    let mut base = test_utils::base_config(root.join("base"));
    base.chain = ChainKind::Base;
    base.chain_id = base.chain.chain_id();
    base.genesis_hash = base.chain.genesis_hash();
    base.peer_cache_dir = Some(root.clone());
    let base_store = Storage::open(&base)?;
    ensure!(
        base_store.peer_cache_snapshot().is_empty(),
        "legacy peers leaked into Base"
    );
    ensure!(
        base_store.peer_cache_dir != eth.peer_cache_dir,
        "chain namespaces collided"
    );
    base_store.upsert_peer(stored_peer("base-only", 200)?)?;
    base_store.flush_peer_cache()?;
    ensure!(
        base_store
            .validate_peer_cache_identity(mainnet.chain_id, mainnet.genesis_hash)
            .is_err(),
        "Base cache allowed Ethereum network"
    );
    ensure!(
        eth.validate_peer_cache_identity(base.chain_id, base.genesis_hash)
            .is_err(),
        "Ethereum cache allowed Base network"
    );
    base_store.validate_peer_cache_identity(base.chain_id, base.genesis_hash)?;
    let reopened = Storage::open(&base)?;
    ensure!(
        reopened.peer_cache_snapshot()[0].peer_id == "base-only",
        "Base cache did not persist"
    );
    let reopened_eth = Storage::open(&mainnet)?;
    ensure!(
        reopened_eth.peer_cache_snapshot()[0].peer_id == "legacy",
        "Base overwrote Ethereum peers"
    );
    ensure!(
        load_peer_cache(&root)?.len() == 1,
        "legacy file was changed"
    );

    // The genesis hash is part of the namespace even when chain IDs match.
    let mut other = base;
    other.data_dir = root.join("different-genesis");
    other.genesis_hash = alloy_primitives::B256::repeat_byte(1);
    let foreign = Storage::open(&other)?;
    ensure!(
        foreign.peer_cache_dir != base_store.peer_cache_dir,
        "genesis namespaces collided"
    );
    ensure!(
        foreign.peer_cache_snapshot().is_empty(),
        "foreign genesis loaded Base peers"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn namespaced_cache_preserves_ttl_cap_addresses_and_aimd() -> Result<()> {
    let root = test_utils::temp_dir("peer-cache-policy");
    let config = test_utils::base_config(root.clone());
    let storage = Storage::open(&config)?;
    for (id, seen) in [("expired", 10), ("older", 110), ("newest", 120)] {
        storage.upsert_peer(stored_peer(id, seen)?)?;
    }
    storage.update_peer_batch_limits(&[("newest".to_string(), 64)]);
    let mut refreshed = stored_peer("newest", 130)?;
    refreshed.aimd_batch_limit = None;
    storage.upsert_peer(refreshed)?;
    let loaded = storage.load_peers(100, 1)?;
    ensure!(
        loaded.total == 3 && loaded.expired == 1 && loaded.capped == 1,
        "TTL/cap policy changed"
    );
    ensure!(
        loaded.peers.len() == 1 && loaded.peers[0].peer_id == "newest",
        "wrong peer retained"
    );
    let persisted = Storage::open(&config)?.peer_cache_snapshot();
    ensure!(
        persisted[0].aimd_batch_limit == Some(64),
        "AIMD was lost on refresh/reopen"
    );
    ensure!(
        persisted[0].udp_addr == Some("127.0.0.1:30304".parse()?),
        "discovery address lost"
    );
    // A populated legacy file must not override an existing empty namespace.
    persist_peer_cache(&root, &[stored_peer("legacy", 999)?])?;
    storage.load_peers(1000, 1)?;
    ensure!(
        Storage::open(&config)?.peer_cache_snapshot().is_empty(),
        "legacy peers resurrected after TTL expiry"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn disk_usage_reports_namespaced_cache_instead_of_stale_legacy_file() -> Result<()> {
    for chain in [ChainKind::Mainnet, ChainKind::Base] {
        let root = test_utils::temp_dir("peer-cache-disk-usage");
        let mut config = test_utils::base_config(root.clone());
        config.chain = chain;
        config.chain_id = chain.chain_id();
        config.genesis_hash = chain.genesis_hash();
        let storage = Storage::open(&config)?;
        storage.upsert_peer(stored_peer("namespaced-peer", 200)?)?;
        storage.flush_peer_cache()?;
        let expected = file_size(&peer_path(&storage.peer_cache_dir))?;
        ensure!(expected > 0, "cache fixture is empty");
        ensure!(
            Storage::disk_usage_at(&root)?.peers_bytes == expected,
            "fresh namespaced cache was missed"
        );

        persist_peer_cache(&root, &[stored_peer("legacy", 100)?])?;
        ensure!(
            file_size(&peer_path(&root))? != expected,
            "fixture sizes must differ"
        );
        ensure!(
            Storage::disk_usage_at(&root)?.peers_bytes == expected,
            "stale legacy size was reported after migration"
        );
        ensure!(
            storage.disk_usage_cached()?.peers_bytes == expected,
            "cached and scanned statistics differ"
        );
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn disk_usage_resolves_legacy_ethereum_metadata_before_and_after_migration() -> Result<()> {
    let root = test_utils::temp_dir("peer-cache-legacy-disk-usage");
    let config = test_utils::base_config(root.clone());
    let storage = Storage::open(&config)?;
    let mut meta = load_meta(&meta_path(&root))?;
    meta.genesis_hash = None;
    persist_meta(&meta_path(&root), &meta)?;
    persist_peer_cache(&root, &[stored_peer("legacy", 100)?])?;
    let legacy_bytes = file_size(&peer_path(&root))?;
    ensure!(
        Storage::disk_usage_at(&root)?.peers_bytes == legacy_bytes,
        "unmigrated Ethereum cache was missed"
    );

    storage.flush_peer_cache()?;
    let migrated_bytes = file_size(&peer_path(&storage.peer_cache_dir))?;
    ensure!(migrated_bytes != legacy_bytes, "fixture sizes must differ");
    ensure!(
        Storage::disk_usage_at(&root)?.peers_bytes == migrated_bytes,
        "legacy metadata did not resolve the migrated cache"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}
