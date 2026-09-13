//! Opt-in production snapshot-import proof against verified external inputs.

use super::{module, RpcConfig, RpcContext};
use crate::{
    cli::{Command, NodeConfig},
    snapshot,
    storage::Storage,
};
use eyre::{ensure, eyre, Result};
use op_alloy_consensus::OpReceipt;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};

async fn rpc_request(
    rpc: &jsonrpsee::RpcModule<RpcContext>,
    method: &str,
    params: Value,
) -> Result<Value> {
    let request = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    let (response, _) = rpc.raw_json_request(&request.to_string(), 1).await?;
    let value: Value = serde_json::from_str(response.get())?;
    ensure!(value.get("error").is_none(), "RPC error: {value}");
    Ok(value["result"].clone())
}

#[tokio::test]
#[ignore = "requires verified external snapshot staging and an unused output directory"]
async fn import_snapshot_sample() -> Result<()> {
    let root = PathBuf::from(std::env::var("SHINODE_SNAPSHOT_SAMPLE")?);
    let output = PathBuf::from(std::env::var("SHINODE_SNAPSHOT_OUTPUT")?);
    let config = NodeConfig::try_from_args(vec![
        "shinode".to_owned(),
        "--shard-size".to_owned(),
        "8".to_owned(),
        "snapshot-import".to_owned(),
        "--manifest".to_owned(),
        root.join("manifest.json").display().to_string(),
        "--manifest-sha256".to_owned(),
        "d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c".to_owned(),
        "--start-block".to_owned(),
        "4984".to_owned(),
        "--end-block".to_owned(),
        "5527".to_owned(),
        "--checkpoint-hash".to_owned(),
        "0x2246525ba06cf8771e47607736d502e446cf5cffe00a2646ceb12de043d4205f".to_owned(),
        "--staging-dir".to_owned(),
        root.display().to_string(),
        "--output-dir".to_owned(),
        output.display().to_string(),
        "--offline".to_owned(),
    ])?;
    let Some(Command::SnapshotImport(args)) = config.command.clone() else {
        return Err(eyre!("snapshot command missing"));
    };
    snapshot::run(config.clone(), args).await?;
    let storage = Arc::new(Storage::open(&config)?);
    storage.validate_snapshot_boundary()?;
    ensure!(storage.last_indexed_block()? == Some(5527), "wrong anchored head");
    let rpc = module(RpcContext {
        config: RpcConfig::from(&config),
        storage: Arc::clone(&storage),
        stats: None,
    })?;
    ensure!(
        rpc_request(&rpc, "eth_chainId", json!([])).await? == "0x2105",
        "wrong chain ID"
    );
    let mut evidence = Vec::new();
    for number in (4984..4992).chain(5520..5528) {
        let tag = format!("0x{number:x}");
        let block = rpc_request(&rpc, "eth_getBlockByNumber", json!([tag, false])).await?;
        let logs = rpc_request(
            &rpc,
            "eth_getLogs",
            json!([{"fromBlock":tag,"toBlock":tag}]),
        )
        .await?;
        let receipts = storage
            .block_receipts(number)?
            .ok_or_else(|| eyre!("missing receipts"))?;
        let deposits: Vec<_> = receipts.iter().enumerate().filter_map(|(index, receipt)| {
            if let crate::storage::StoredReceiptRef::Op(OpReceipt::Deposit(r)) = receipt {
                Some(json!({"transaction_index":index,"nonce":r.deposit_nonce,"receipt_version":r.deposit_receipt_version}))
            } else { None }
        }).collect();
        evidence.push(json!({"number":number,"block":block,"logs":logs,"deposits":deposits}));
    }
    std::fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(&json!({
            "blocks": evidence, "imported_blocks": 544, "range": [4984,5527],
            "production_importer": true, "queried_after_reopen": true,
        }))?,
    )?;
    Ok(())
}
