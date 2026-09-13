//! Opt-in comparison of imported/restored history with a saved independent RPC reference.
use crate::{
    cli::NodeConfig,
    storage::{Storage, StoredReceiptRef},
};
use eyre::{ensure, eyre, Result};
use op_alloy_consensus::OpReceipt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::Arc};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    range: [u64; 2],
    reference_rpc: String,
    blocks: Vec<Sample>,
    logs: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    number: u64,
    block: Value,
    receipts: Vec<Value>,
}

fn quantity(value: &Value) -> Result<Option<u64>> {
    value
        .as_str()
        .map(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).map_err(Into::into))
        .transpose()
}

fn standard_logs(logs: &[Value]) -> Vec<Value> {
    logs.iter()
        .map(|log| {
            let fields = [
                "address",
                "topics",
                "data",
                "blockNumber",
                "blockHash",
                "transactionHash",
                "transactionIndex",
                "logIndex",
                "removed",
            ];
            Value::Object(
                fields
                    .into_iter()
                    .map(|key| (key.to_owned(), log[key].clone()))
                    .collect(),
            )
        })
        .collect()
}

#[tokio::test]
#[ignore = "requires an imported database and saved independent block/receipt/complete-log reference"]
async fn history_matches_independent_reference() -> Result<()> {
    let data = std::env::var("SHINODE_HISTORY_DATA")?;
    let path = PathBuf::from(std::env::var("SHINODE_HISTORY_REFERENCE")?);
    ensure!(
        fs::metadata(&path)?.len() <= 8 * 1024 * 1024,
        "reference exceeds 8 MiB"
    );
    let reference: Reference = serde_json::from_slice(&fs::read(&path)?)?;
    let [start, end] = reference.range;
    ensure!(
        start <= end && end - start < 100_000,
        "invalid reference range"
    );
    ensure!(
        !reference.reference_rpc.is_empty()
            && reference.blocks.len() >= 2
            && reference.blocks.first().is_some_and(|b| b.number == start)
            && reference.blocks.last().is_some_and(|b| b.number == end)
            && reference
                .blocks
                .windows(2)
                .all(|b| b[0].number < b[1].number),
        "reference must include ordered samples and both endpoints"
    );
    let mut config =
        NodeConfig::try_from_args(["shinode", "--chain", "base", "--data-dir", &data])?;
    config.rpc_max_blocks_per_filter = end - start + 1;
    let storage = Arc::new(Storage::open(&config)?);
    storage.validate_snapshot_boundary()?;
    ensure!(
        storage.last_indexed_block()? == Some(end),
        "unexpected indexed head"
    );
    ensure!(
        storage.missing_ranges_in_range(start..=end)?.1 == 0,
        "missing stored blocks"
    );
    let rpc = super::test_module(&config, Arc::clone(&storage))?;
    let mut deposits = 0;
    for sample in &reference.blocks {
        deposits += verify_sample(&rpc, &storage, sample).await?;
    }
    let logs = request(
        &rpc,
        "eth_getLogs",
        json!([{"fromBlock":format!("0x{start:x}"),"toBlock":format!("0x{end:x}")}]),
    )
    .await?;
    let logs = logs
        .as_array()
        .ok_or_else(|| eyre!("logs response is not an array"))?;
    ensure!(
        standard_logs(logs) == standard_logs(&reference.logs),
        "complete range logs differ from independent RPC"
    );
    fs::write(
        std::env::var("SHINODE_HISTORY_PROOF")?,
        serde_json::to_vec_pretty(&json!({
            "range":reference.range, "reference_rpc":reference.reference_rpc,
            "sampled_blocks":reference.blocks.len(), "sampled_deposits":deposits,
            "complete_logs":logs.len(), "stored_blocks":storage.aggregate_stats().total_blocks,
            "checkpoint_hash":reference.blocks.last().map(|b| &b.block["hash"]),
            "passed":true
        }))?,
    )?;
    Ok(())
}

async fn request(
    rpc: &jsonrpsee::RpcModule<super::RpcContext>,
    method: &str,
    params: Value,
) -> Result<Value> {
    let (response, _) = rpc
        .raw_json_request(
            &json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}).to_string(),
            1,
        )
        .await?;
    let response: Value = serde_json::from_str(response.get())?;
    ensure!(response.get("error").is_none(), "RPC error: {response}");
    Ok(response["result"].clone())
}

async fn verify_sample(
    rpc: &jsonrpsee::RpcModule<super::RpcContext>,
    storage: &Storage,
    sample: &Sample,
) -> Result<usize> {
    let mut deposits = 0;
    let actual = request(
        rpc,
        "eth_getBlockByNumber",
        json!([format!("0x{:x}", sample.number), false]),
    )
    .await?;
    for field in [
        "number",
        "hash",
        "parentHash",
        "stateRoot",
        "transactionsRoot",
        "receiptsRoot",
        "logsBloom",
        "timestamp",
        "gasLimit",
        "gasUsed",
        "size",
        "transactions",
    ] {
        ensure!(
            !sample.block[field].is_null() && actual[field] == sample.block[field],
            "block {} field {field} differs from independent RPC",
            sample.number
        );
    }
    let receipts = storage
        .block_receipts(sample.number)?
        .ok_or_else(|| eyre!("missing receipts"))?;
    ensure!(
        sample.receipts.len() == receipts.len(),
        "receipt count mismatch"
    );
    for (index, receipt) in receipts.iter().enumerate() {
        let expected = &sample.receipts[index];
        ensure!(
            expected["transactionHash"] == sample.block["transactions"][index],
            "receipt identity mismatch"
        );
        deposits += verify_receipt(&receipt, expected, sample.number, index)?;
    }
    Ok(deposits)
}

fn verify_receipt(
    receipt: &StoredReceiptRef<'_>,
    expected: &Value,
    block: u64,
    index: usize,
) -> Result<usize> {
    let stored_type = match receipt {
        StoredReceiptRef::Ethereum(receipt) => u8::from(receipt.tx_type),
        StoredReceiptRef::Op(receipt) => u8::from(receipt.tx_type()),
    };
    ensure!(
        quantity(&expected["type"])? == Some(u64::from(stored_type)),
        "receipt type differs at block {block} transaction {index}"
    );
    if let StoredReceiptRef::Op(OpReceipt::Deposit(deposit)) = receipt {
        ensure!(
            quantity(&expected["depositNonce"])? == deposit.deposit_nonce
                && quantity(&expected["depositReceiptVersion"])? == deposit.deposit_receipt_version,
            "deposit fields differ at block {block} transaction {index}"
        );
        return Ok(1);
    }
    Ok(0)
}

#[test]
fn reference_receipt_type_detects_lost_deposit_variant() -> Result<()> {
    let deposit = op_alloy_consensus::OpDepositReceipt {
        inner: alloy_consensus::Receipt::default(),
        deposit_nonce: None,
        deposit_receipt_version: None,
    };
    // Pre-Canyon deposit fields may be absent and logs unchanged after a type loss.
    let ordinary = OpReceipt::Legacy(deposit.inner.clone());
    let stored_deposit = OpReceipt::Deposit(deposit);
    let reference = json!({"type":"0x7e"});
    ensure!(
        verify_receipt(&StoredReceiptRef::Op(&stored_deposit), &reference, 1, 0)? == 1,
        "matching reference deposit must be counted"
    );
    let error = verify_receipt(&StoredReceiptRef::Op(&ordinary), &reference, 1, 0)
        .err()
        .ok_or_else(|| eyre!("a reference deposit cannot match an ordinary stored receipt"))?;
    ensure!(
        error.to_string().contains("receipt type differs"),
        "wrong rejection reason"
    );
    ensure!(
        verify_receipt(
            &StoredReceiptRef::Op(&stored_deposit),
            &json!({"type":"0x0"}),
            1,
            0
        )
        .is_err(),
        "an ordinary reference cannot match a stored deposit"
    );
    for invalid in [json!({}), json!({"type":126}), json!({"type":"invalid"})] {
        ensure!(
            verify_receipt(&StoredReceiptRef::Op(&stored_deposit), &invalid, 1, 0).is_err(),
            "missing or malformed reference type must fail"
        );
    }
    Ok(())
}
