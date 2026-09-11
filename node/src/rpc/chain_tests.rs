use super::*;
use crate::chain::{BaseChain, ChainKind, EthereumChain};
use crate::storage::BlockBundle;
use crate::sync::historical::IngestChain;
use crate::sync::validation::tests::{base_payload, commit_payload, ethereum_payload, ISTHMUS};
use alloy_consensus::SignableTransaction;
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Log, Signature, U256};
use eyre::ensure;
use op_alloy_consensus::OpTxEnvelope;
use reth_optimism_primitives::OpReceipt;

fn fixture_logs() -> Vec<Log> {
    (0_u8..4)
        .map(|i| {
            Log::new_unchecked(
                Address::repeat_byte(if i < 2 { 0x11 } else { 0x22 }),
                vec![B256::repeat_byte(0x77), B256::repeat_byte(i)],
                Bytes::from(vec![0xa0 + i]),
            )
        })
        .collect()
}

fn base_fixture() -> Result<BlockBundle> {
    let logs = fixture_logs();
    let mut payload = base_payload(ISTHMUS);
    let OpReceipt::Deposit(receipt) = &mut payload.receipts[0] else {
        eyre::bail!("missing deposit");
    };
    receipt.inner.logs = logs[..2].to_vec();
    let tx = alloy_consensus::TxLegacy {
        nonce: 1,
        chain_id: Some(8453),
        ..alloy_consensus::TxLegacy::default()
    }
    .into_signed(Signature::new(U256::from(1), U256::from(2), false));
    payload.body.transactions.push(OpTxEnvelope::Legacy(tx));
    payload
        .receipts
        .push(OpReceipt::Legacy(alloy_consensus::Receipt {
            status: true.into(),
            cumulative_gas_used: 42_000,
            logs: logs[2..].to_vec(),
        }));
    commit_payload(&mut payload);
    let expected_hashes: Vec<_> = payload
        .body
        .transactions
        .iter()
        .map(|tx| alloy_primitives::keccak256(tx.encoded_2718()))
        .collect();
    let (bundle, _) = BaseChain::process_ingest(payload, None)?;
    ensure!(
        bundle.tx_hashes.hashes == expected_hashes,
        "Base transaction hashes changed"
    );
    Ok(bundle)
}

fn ethereum_fixture() -> Result<BlockBundle> {
    let logs = fixture_logs();
    let mut payload = ethereum_payload();
    payload.receipts[0].logs = logs[..2].to_vec();
    let tx = alloy_consensus::TxLegacy {
        nonce: 1,
        ..alloy_consensus::TxLegacy::default()
    }
    .into_signed(Signature::new(U256::from(1), U256::from(2), false));
    payload.body.transactions.push(tx.into());
    payload.receipts.push(reth_ethereum_primitives::Receipt {
        logs: logs[2..].to_vec(),
        ..reth_ethereum_primitives::Receipt::default()
    });
    commit_payload(&mut payload);
    Ok(EthereumChain::process_ingest(payload, None)?.0)
}

async fn request(module: &RpcModule<RpcContext>, method: &str, params: Value) -> Result<Value> {
    let raw = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
        .to_string();
    let (response, _) = module.raw_json_request(&raw, 1).await?;
    Ok(serde_json::from_str(response.get())?)
}

fn reference_logs(bundle: &BlockBundle) -> Vec<Value> {
    (0_u8..4).map(|i| serde_json::json!({
        "address": format!("{:#x}", Address::repeat_byte(if i < 2 { 0x11 } else { 0x22 })),
        "topics": [format!("{:#x}", B256::repeat_byte(0x77)), format!("{:#x}", B256::repeat_byte(i))],
        "data": format!("0x{:02x}", 0xa0 + i),
        "blockNumber": format!("0x{:x}", bundle.number),
        "blockHash": format!("{:#x}", bundle.header.hash_slow()),
        "transactionHash": format!("{:#x}", bundle.tx_hashes.hashes[usize::from(i / 2)]),
        "transactionIndex": format!("0x{:x}", i / 2),
        "logIndex": format!("0x{i:x}"),
        "removed": false,
    })).collect()
}

async fn check_block_contract(module: &RpcModule<RpcContext>, bundle: &BlockBundle) -> Result<()> {
    let block = request(
        module,
        "eth_getBlockByNumber",
        serde_json::json!(["0x2a", false]),
    )
    .await?;
    ensure!(
        block["result"]["hash"] == format!("{:#x}", bundle.header.hash_slow()),
        "wrong block hash"
    );
    let hashes: Vec<_> = bundle
        .tx_hashes
        .hashes
        .iter()
        .map(|hash| format!("{hash:#x}"))
        .collect();
    ensure!(
        block["result"]["transactions"] == serde_json::json!(hashes),
        "wrong transaction hashes"
    );
    ensure!(
        block["result"]["size"] == format!("0x{:x}", bundle.size.size),
        "wrong block size"
    );
    ensure!(
        block["result"]["withdrawals"].is_null(),
        "withdrawals contract changed"
    );
    if let Some(root) = bundle.header.withdrawals_root {
        ensure!(
            block["result"]["withdrawalsRoot"] == format!("{root:#x}"),
            "withdrawals storage commitment lost"
        );
    }
    let full = request(
        module,
        "eth_getBlockByNumber",
        serde_json::json!(["0x2a", true]),
    )
    .await?;
    ensure!(
        full["error"]["code"] == -32602,
        "full-transaction contract changed"
    );
    let missing = request(
        module,
        "eth_getBlockByNumber",
        serde_json::json!(["0x29", false]),
    )
    .await?;
    ensure!(
        missing["result"].is_null() && missing.get("error").is_none(),
        "missing block contract changed"
    );
    let safe = request(
        module,
        "eth_getBlockByNumber",
        serde_json::json!(["safe", false]),
    )
    .await?;
    ensure!(
        safe["error"]["code"] == -32602,
        "P2P view exposed as safe head"
    );
    Ok(())
}

async fn check_rpc(chain: ChainKind, bundle: BlockBundle) -> Result<()> {
    let root = crate::test_utils::temp_dir("chain-rpc");
    let mut config = crate::test_utils::base_config(root.clone());
    config.chain = chain;
    config.chain_id = chain.chain_id();
    config.genesis_hash = chain.genesis_hash();
    config.shard_size = 2;
    let storage = Storage::open(&config)?;
    let mut next = bundle.clone();
    next.number += 1;
    next.header.number += 1;
    next.header.parent_hash = bundle.header.hash_slow();
    next.header.timestamp += 2;
    storage.write_block_bundles_wal(&[next.clone(), bundle.clone()])?;
    storage.compact_all_dirty()?;
    storage.seal_completed_shards()?;
    drop(storage);
    let module = module(RpcContext {
        storage: Arc::new(Storage::open(&config)?),
        config: RpcConfig::from(&config),
        stats: None,
    })?;
    let response = request(&module, "eth_chainId", serde_json::json!([])).await?;
    ensure!(
        response["result"] == format!("0x{:x}", chain.chain_id()),
        "wrong chain ID"
    );
    let latest = request(&module, "eth_blockNumber", serde_json::json!([])).await?;
    ensure!(latest["result"] == "0x2b", "wrong indexed head");
    check_block_contract(&module, &bundle).await?;
    let logs = request(
        &module,
        "eth_getLogs",
        serde_json::json!([{"fromBlock":"0x2a","toBlock":"0x2b"}]),
    )
    .await?;
    let mut expected = reference_logs(&bundle);
    expected.extend(reference_logs(&next));
    ensure!(
        logs["result"] == serde_json::json!(expected),
        "complete logs differ: {logs}"
    );
    let filtered = request(&module, "eth_getLogs", serde_json::json!([{
        "fromBlock":"0x2a","toBlock":"0x2a", "address":format!("{:#x}", Address::repeat_byte(0x22)),
        "topics":[null,format!("{:#x}", B256::repeat_byte(3))]
    }])).await?;
    ensure!(
        filtered["result"] == serde_json::json!([reference_logs(&bundle)[3]]),
        "filter changed log indices: {filtered}"
    );
    drop(module);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn base_rpc_preserves_mixed_deposit_and_user_block_contract() -> Result<()> {
    check_rpc(ChainKind::Base, base_fixture()?).await
}

#[tokio::test]
async fn ethereum_rpc_uses_block_wide_log_indices() -> Result<()> {
    check_rpc(ChainKind::Mainnet, ethereum_fixture()?).await
}
