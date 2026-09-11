use super::*;
use crate::chain::{BaseChain, ChainKind, EthereumChain};
use crate::sync::historical::IngestChain;
use crate::sync::validation::tests::{
    base_payload, commit_payload, ethereum_payload, CANYON, ISTHMUS,
};
use alloy_consensus::{SignableTransaction, Transaction, TxReceipt};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{keccak256, Sealable, TxKind};
use eyre::ensure;
use op_alloy_consensus::{OpTxEnvelope, TxDeposit};

fn config(root: std::path::PathBuf, chain: ChainKind) -> NodeConfig {
    let mut config = crate::test_utils::base_config(root);
    config.chain = chain;
    config.chain_id = chain.chain_id();
    config.genesis_hash = chain.genesis_hash();
    config.start_block = 0;
    config.shard_size = 2;
    config
}

fn base_bundle(number: u64, nonce: Option<u64>, version: Option<u64>) -> Result<BlockBundle> {
    let mut payload = base_payload(if version.is_some() {
        ISTHMUS
    } else {
        CANYON - 1
    });
    payload.header.number = number;
    payload.body.transactions[0] = OpTxEnvelope::Deposit(
        TxDeposit {
            source_hash: B256::repeat_byte(9),
            from: Address::repeat_byte(7),
            to: if nonce.is_none() {
                TxKind::Create
            } else {
                Address::repeat_byte(8).into()
            },
            value: U256::from(123),
            mint: 456,
            gas_limit: 100_000,
            ..TxDeposit::default()
        }
        .seal_slow(),
    );
    let OpReceipt::Deposit(receipt) = &mut payload.receipts[0] else {
        eyre::bail!("missing deposit fixture");
    };
    receipt.deposit_nonce = nonce;
    receipt.deposit_receipt_version = version;
    let signature = Signature::new(U256::from(1), U256::from(2), false);
    let ordinary = [
        OpTxEnvelope::Legacy(alloy_consensus::TxLegacy::default().into_signed(signature)),
        OpTxEnvelope::Eip2930(alloy_consensus::TxEip2930::default().into_signed(signature)),
        OpTxEnvelope::Eip1559(alloy_consensus::TxEip1559::default().into_signed(signature)),
        OpTxEnvelope::Eip7702(alloy_consensus::TxEip7702::default().into_signed(signature)),
    ];
    let receipts = [
        OpReceipt::Legacy(alloy_consensus::Receipt::default()),
        OpReceipt::Eip2930(alloy_consensus::Receipt::default()),
        OpReceipt::Eip1559(alloy_consensus::Receipt::default()),
        OpReceipt::Eip7702(alloy_consensus::Receipt::default()),
    ];
    payload.body.transactions.extend(ordinary);
    payload.receipts.extend(receipts);
    commit_payload(&mut payload);
    crate::sync::validation::validate_payload::<BaseChain>(
        &payload.header,
        &payload.body,
        &payload.receipts,
    )
    .map_err(|e| eyre::eyre!(e))?;
    let encoded_size = alloy_rlp::encode(&alloy_consensus::Block {
        header: payload.header.clone(),
        body: payload.body.clone(),
    })
    .len() as u64;
    let (bundle, logs) = BaseChain::process_ingest(payload.clone(), None)?;
    ensure!(
        bundle.size.size == encoded_size && logs == 1,
        "block size/log count mismatch"
    );
    for (tx, stored) in payload
        .body
        .transactions
        .iter()
        .zip(&bundle.transactions.txs)
    {
        ensure!(
            stored.hash == keccak256(tx.encoded_2718()),
            "noncanonical transaction hash"
        );
        ensure!(
            stored.to == tx.to() && stored.value == tx.value(),
            "transaction fields changed"
        );
    }
    let deposit = &bundle.transactions.txs[0];
    ensure!(
        deposit.from == Some(Address::repeat_byte(7)) && deposit.nonce == nonce,
        "deposit sender/nonce lost"
    );
    ensure!(
        deposit.signature.is_none() && deposit.signing_hash.is_none(),
        "deposit has fabricated signature"
    );
    ensure!(
        bundle.transactions.txs[1..]
            .iter()
            .all(|tx| tx.nonce == Some(0)
                && tx.signature == Some(signature)
                && tx.signing_hash.is_some()),
        "ordinary signatures/nonces lost"
    );
    assert_deposit_receipt(&bundle)?;
    Ok(bundle)
}

fn assert_deposit_receipt(bundle: &BlockBundle) -> Result<()> {
    let view = bundle
        .receipts
        .iter()
        .next()
        .ok_or_else(|| eyre::eyre!("missing receipt"))?;
    ensure!(
        view.status() && view.cumulative_gas_used() == 21_000 && view.logs().len() == 1,
        "common receipt accessors disagree"
    );
    Ok(())
}

fn assert_bundle(storage: &Storage, bundle: &BlockBundle) -> Result<()> {
    ensure!(
        storage.block_header(bundle.number)? == Some(bundle.header.clone()),
        "header changed"
    );
    ensure!(
        storage.block_tx_hashes(bundle.number)? == Some(bundle.tx_hashes.clone()),
        "transaction hashes changed"
    );
    ensure!(
        storage.block_transactions(bundle.number)? == Some(bundle.transactions.clone()),
        "transaction metadata changed"
    );
    ensure!(
        storage.block_receipts(bundle.number)? == Some(bundle.receipts.clone()),
        "receipt fields changed"
    );
    ensure!(
        storage.block_size(bundle.number)? == Some(bundle.size),
        "block size changed"
    );
    Ok(())
}

#[test]
fn base_deposits_survive_wal_compaction_sealing_follow_and_rollback() -> Result<()> {
    let root = crate::test_utils::temp_dir("base-storage-roundtrip");
    let config = config(root.clone(), ChainKind::Base);
    let bundles = [
        base_bundle(0, None, None)?,
        base_bundle(1, Some(7), None)?,
        base_bundle(2, Some(8), Some(1))?,
    ];
    let storage = Storage::open(&config)?;
    storage.write_block_bundles_wal(&[bundles[1].clone(), bundles[0].clone()])?;
    drop(storage);
    let storage = Storage::open(&config)?;
    ensure!(
        storage.missing_blocks_in_range(0..=1)?.is_empty(),
        "WAL recovery lost pending blocks"
    );
    storage.compact_all_dirty()?;
    storage.seal_completed_shards()?;
    drop(storage);
    let storage = Storage::open(&config)?;
    assert_bundle(&storage, &bundles[0])?;
    assert_bundle(&storage, &bundles[1])?;
    storage.write_block_bundle_follow(&bundles[2])?;
    assert_bundle(&storage, &bundles[2])?;
    let stats = storage.aggregate_stats();
    ensure!(
        stats.total_receipts == 15 && stats.total_transactions == 15 && stats.total_logs == 3,
        "Base aggregate counts wrong"
    );
    // Roll back through a sealed shard and a pending WAL tail, then reopen.
    let pending = base_bundle(3, Some(9), Some(1))?;
    storage.write_block_bundles_wal(&[pending])?;
    storage.rollback_to(0)?;
    drop(storage);
    let storage = Storage::open(&config)?;
    assert_bundle(&storage, &bundles[0])?;
    for number in 1..=3 {
        ensure!(
            storage.block_receipts(number)?.is_none(),
            "rollback resurrected receipts"
        );
        ensure!(
            storage.block_transactions(number)?.is_none(),
            "rollback resurrected transactions"
        );
    }
    ensure!(
        storage.missing_blocks_in_range(1..=3)? == vec![1, 2, 3],
        "rollback retained pending coverage"
    );
    let stats = storage.aggregate_stats();
    ensure!(
        stats.total_blocks == 1 && stats.total_receipts == 5 && stats.total_logs == 1,
        "rollback counts stale"
    );
    storage.write_block_bundle_follow(&bundles[1])?;
    assert_bundle(&storage, &bundles[1])?;
    drop(storage);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn ethereum_codec_preserves_legacy_bytes_and_roundtrips() -> Result<()> {
    let payload = ethereum_payload();
    let old_receipts = payload
        .receipts
        .iter()
        .map(SerdeBincodeCompat::as_repr)
        .collect::<Vec<_>>();
    let expected_receipts = bincode::serialize(&old_receipts)?;
    let (bundle, _) = EthereumChain::process_ingest(payload, None)?;
    let tx = &bundle.transactions.txs[0];
    // Historical StoredTransactions was a one-field Vec wrapper with a u64 nonce.
    let old_transactions = vec![(
        tx.hash,
        tx.from,
        tx.to,
        tx.value,
        0_u64,
        tx.signature,
        tx.signing_hash,
    )];
    let expected_transactions = bincode::serialize(&old_transactions)?;
    let codec = StorageCodec::EthereumV1;
    ensure!(
        codec.encode_receipts(&bundle.receipts)? == expected_receipts,
        "legacy receipt bytes changed"
    );
    ensure!(
        codec.encode_transactions(&bundle.transactions)? == expected_transactions,
        "legacy transaction bytes changed"
    );
    ensure!(
        codec.decode_receipts(&expected_receipts)? == bundle.receipts,
        "legacy receipts unreadable"
    );
    ensure!(
        codec.decode_transactions(&expected_transactions)? == bundle.transactions,
        "legacy transactions unreadable"
    );
    let root = crate::test_utils::temp_dir("ethereum-codec-roundtrip");
    let config = config(root.clone(), ChainKind::Mainnet);
    let storage = Storage::open(&config)?;
    storage.write_block_bundles_wal(std::slice::from_ref(&bundle))?;
    storage.compact_all_dirty()?;
    drop(storage);
    // Legacy metadata had no codec field. It must still select identical Ethereum bytes.
    let path = root.join("meta.json");
    let mut meta: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    meta.as_object_mut()
        .ok_or_else(|| eyre::eyre!("invalid metadata"))?
        .remove("storage_codec");
    std::fs::write(path, serde_json::to_vec(&meta)?)?;
    let storage = Storage::open(&config)?;
    assert_bundle(&storage, &bundle)?;
    drop(storage);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn codecs_reject_foreign_receipts_before_writes() -> Result<()> {
    let root = crate::test_utils::temp_dir("storage-codec-mismatch");
    let config = config(root.clone(), ChainKind::Mainnet);
    let storage = Storage::open(&config)?;
    let bundle = base_bundle(0, None, None)?;
    ensure!(
        storage
            .write_block_bundles_wal(std::slice::from_ref(&bundle))
            .is_err(),
        "OP WAL accepted by Ethereum"
    );
    ensure!(
        storage.write_block_bundle_follow(&bundle).is_err(),
        "OP follow write accepted by Ethereum"
    );
    ensure!(
        storage.aggregate_stats().total_blocks == 0 && storage.dirty_shards()?.is_empty(),
        "rejected write changed storage"
    );
    ensure!(
        StorageCodec::OpV1
            .encode_receipts(&StoredReceipts::Ethereum(Vec::new()))
            .is_err(),
        "empty foreign receipts accepted"
    );
    drop(storage);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn base_requires_explicit_codec_metadata() -> Result<()> {
    let root = crate::test_utils::temp_dir("base-codec-metadata");
    let config = config(root.clone(), ChainKind::Base);
    drop(Storage::open(&config)?);
    let path = root.join("meta.json");
    let mut meta: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    ensure!(meta["storage_codec"] == "op_v1", "Base codec not versioned");
    for value in [
        serde_json::Value::Null,
        serde_json::json!("ethereum_v1"),
        serde_json::json!("op_v999"),
    ] {
        meta["storage_codec"] = value;
        let bytes = serde_json::to_vec(&meta)?;
        std::fs::write(&path, &bytes)?;
        ensure!(
            Storage::open(&config).is_err(),
            "incompatible Base codec accepted"
        );
        ensure!(std::fs::read(&path)? == bytes, "invalid metadata rewritten");
    }
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn base_processing_rejects_receipt_count_and_type_mismatch() -> Result<()> {
    let mut payload = base_payload(ISTHMUS);
    payload.receipts.clear();
    ensure!(
        BaseChain::process_ingest(payload, None).is_err(),
        "missing receipt accepted"
    );
    let mut payload = base_payload(ISTHMUS);
    payload.receipts[0] = OpReceipt::Legacy(alloy_consensus::Receipt::default());
    ensure!(
        BaseChain::process_ingest(payload, None).is_err(),
        "deposit matched ordinary receipt"
    );
    Ok(())
}
