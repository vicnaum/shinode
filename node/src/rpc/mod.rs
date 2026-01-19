//! JSON-RPC server.

use crate::storage::{Storage, StoredLog};
use alloy_primitives::{Address, B256};
use eyre::{Result, WrapErr};
use jsonrpsee::{
    server::{ServerBuilder, ServerHandle},
    types::ErrorObjectOwned,
    RpcModule,
};
use reth_primitives_traits::SealedHeader;
use serde::Serialize;
use serde_json::Value;
use std::{collections::{BTreeSet, HashMap}, net::SocketAddr, str::FromStr, sync::Arc};

#[derive(Clone)]
pub struct RpcContext {
    chain_id: u64,
    storage: Arc<Storage>,
}

/// Start the JSON-RPC server and return its handle.
pub async fn start(bind: SocketAddr, chain_id: u64, storage: Arc<Storage>) -> Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(bind)
        .await
        .wrap_err("failed to bind RPC server")?;

    Ok(server.start(module(RpcContext { chain_id, storage })?))
}

pub fn module(ctx: RpcContext) -> Result<RpcModule<RpcContext>> {
    let mut module = RpcModule::new(ctx);
    module
        .register_method(
            "eth_chainId",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
            let params: Option<Value> = params.parse()?;
            ensure_empty_params(params, "eth_chainId expects no params")?;

            Ok(format!("0x{:x}", ctx.chain_id))
            },
        )
        .wrap_err("failed to register eth_chainId")?;

    module
        .register_method(
            "eth_blockNumber",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
            let params: Option<Value> = params.parse()?;
            ensure_empty_params(params, "eth_blockNumber expects no params")?;

            let latest = ctx
                .storage
                .last_indexed_block()
                .map_err(internal_error)?
                .unwrap_or(0);
            Ok(format!("0x{:x}", latest))
            },
        )
        .wrap_err("failed to register eth_blockNumber")?;

    module
        .register_method(
            "eth_getBlockByNumber",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
            let params: Vec<Value> = params.parse().map_err(|_| {
                invalid_params("eth_getBlockByNumber expects [blockTag, includeTransactions]")
            })?;
            if params.len() != 2 {
                return Err(invalid_params(
                    "eth_getBlockByNumber expects [blockTag, includeTransactions]",
                ));
            }

            let include_txs = params[1]
                .as_bool()
                .ok_or_else(|| invalid_params("includeTransactions must be a boolean"))?;
            if include_txs {
                return Err(invalid_params(
                    "eth_getBlockByNumber only supports includeTransactions=false",
                ));
            }

            let tag = parse_block_tag(&params[0])?;
            let latest = ctx.storage.last_indexed_block().map_err(internal_error)?;
            let Some(number) = resolve_block_tag_optional(tag, latest)? else {
                return Ok(Value::Null);
            };

            let Some(header) = ctx.storage.block_header(number).map_err(internal_error)? else {
                return Ok(Value::Null);
            };

            let hash = SealedHeader::seal_slow(header.clone()).hash();
            let tx_hashes = ctx
                .storage
                .block_tx_hashes(number)
                .map_err(internal_error)?
                .map(|stored| stored.hashes)
                .unwrap_or_default();
            let txs = tx_hashes
                .into_iter()
                .map(|tx| format!("{:#x}", tx))
                .collect::<Vec<_>>();

            let response = RpcBlock {
                number: format!("0x{:x}", header.number),
                hash: format!("{:#x}", hash),
                parent_hash: format!("{:#x}", header.parent_hash),
                timestamp: format!("0x{:x}", header.timestamp),
                logs_bloom: format!("{:#x}", header.logs_bloom),
                transactions: txs,
            };

            serde_json::to_value(response).map_err(internal_error)
            },
        )
        .wrap_err("failed to register eth_getBlockByNumber")?;

    module
        .register_method(
            "eth_getLogs",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
            let params: Vec<Value> = params.parse().map_err(|_| {
                invalid_params("eth_getLogs expects a single filter object")
            })?;
            if params.len() != 1 {
                return Err(invalid_params("eth_getLogs expects a single filter object"));
            }
            let filter = params[0]
                .as_object()
                .ok_or_else(|| invalid_params("eth_getLogs expects a filter object"))?;
            if filter.contains_key("blockHash") {
                return Err(invalid_params("eth_getLogs does not support blockHash yet"));
            }

            let latest = ctx.storage.last_indexed_block().map_err(internal_error)?.unwrap_or(0);
            let from_block = filter
                .get("fromBlock")
                .map(parse_block_tag)
                .transpose()?
                .map(|tag| resolve_block_tag_required(tag, latest))
                .transpose()?
                .unwrap_or(0);
            let to_block = filter
                .get("toBlock")
                .map(parse_block_tag)
                .transpose()?
                .map(|tag| resolve_block_tag_required(tag, latest))
                .transpose()?
                .unwrap_or(latest);
            if from_block > to_block {
                return Ok(Value::Array(Vec::new()));
            }

            let address_filter = filter.get("address").map(parse_address_filter).transpose()?;
            let topics_filter = filter.get("topics").map(parse_topics_filter).transpose()?;
            let topic0_filter = topics_filter
                .as_ref()
                .and_then(|topics| topics.first())
                .and_then(|entry| entry.clone());

            let mut candidates: Option<BTreeSet<(u64, u64)>> = None;
            if let Some(addresses) = address_filter.as_ref().filter(|a| !a.is_empty()) {
                let mut set = BTreeSet::new();
                for address in addresses {
                    for entry in ctx
                        .storage
                        .log_index_by_address_range(*address, from_block..=to_block)
                        .map_err(internal_error)?
                    {
                        set.insert((entry.block_number, entry.log_index));
                    }
                }
                candidates = Some(set);
            }
            if let Some(topic0s) = topic0_filter.as_ref().filter(|t| !t.is_empty()) {
                let mut set = BTreeSet::new();
                for topic0 in topic0s {
                    for entry in ctx
                        .storage
                        .log_index_by_topic0_range(*topic0, from_block..=to_block)
                        .map_err(internal_error)?
                    {
                        set.insert((entry.block_number, entry.log_index));
                    }
                }
                candidates = Some(match candidates {
                    Some(prev) => prev.intersection(&set).cloned().collect(),
                    None => set,
                });
            }

            let logs = match candidates {
                Some(entries) => {
                    let mut by_block = HashMap::<u64, Vec<StoredLog>>::new();
                    let mut out = Vec::new();
                    for (block_number, log_index) in entries {
                        if !by_block.contains_key(&block_number) {
                            let stored = ctx
                                .storage
                                .block_logs(block_number)
                                .map_err(internal_error)?
                                .map(|stored| stored.logs)
                                .unwrap_or_default();
                            by_block.insert(block_number, stored);
                        }
                        let logs = by_block
                            .get(&block_number)
                            .expect("block logs cached");
                        if let Some(log) = logs.iter().find(|log| log.log_index == log_index) {
                            if log_matches(log, address_filter.as_deref(), topics_filter.as_deref()) {
                                out.push(format_log(log));
                            }
                        }
                    }
                    out
                }
                None => {
                    let mut out = Vec::new();
                    for (_, stored) in ctx
                        .storage
                        .block_logs_range(from_block..=to_block)
                        .map_err(internal_error)?
                    {
                        for log in stored.logs {
                            if log_matches(&log, address_filter.as_deref(), topics_filter.as_deref())
                            {
                                out.push(format_log(&log));
                            }
                        }
                    }
                    out
                }
            };

            serde_json::to_value(logs).map_err(internal_error)
            },
        )
        .wrap_err("failed to register eth_getLogs")?;

    Ok(module)
}

fn ensure_empty_params(params: Option<Value>, message: &str) -> Result<(), ErrorObjectOwned> {
    let empty = match params {
        None | Some(Value::Null) => true,
        Some(Value::Array(values)) => values.is_empty(),
        Some(Value::Object(values)) => values.is_empty(),
        Some(_) => false,
    };
    if !empty {
        return Err(invalid_params(message));
    }
    Ok(())
}

fn invalid_params(message: &str) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, message, None::<()>)
}

fn internal_error(err: impl std::fmt::Display) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32000, "internal error", Some(err.to_string()))
}

#[derive(Debug)]
enum BlockTag {
    Number(u64),
    Latest,
    Earliest,
}

fn parse_block_tag(value: &Value) -> Result<BlockTag, ErrorObjectOwned> {
    let tag = value
        .as_str()
        .ok_or_else(|| invalid_params("block tag must be a string"))?;
    match tag {
        "latest" => Ok(BlockTag::Latest),
        "earliest" => Ok(BlockTag::Earliest),
        "pending" => Err(invalid_params("pending block tag is not supported")),
        other => Ok(BlockTag::Number(parse_quantity(other)?)),
    }
}

fn resolve_block_tag_optional(
    tag: BlockTag,
    latest: Option<u64>,
) -> Result<Option<u64>, ErrorObjectOwned> {
    Ok(match tag {
        BlockTag::Latest => latest,
        BlockTag::Earliest => Some(0),
        BlockTag::Number(value) => Some(value),
    })
}

fn resolve_block_tag_required(tag: BlockTag, latest: u64) -> Result<u64, ErrorObjectOwned> {
    Ok(match tag {
        BlockTag::Latest => latest,
        BlockTag::Earliest => 0,
        BlockTag::Number(value) => value,
    })
}

fn parse_quantity(value: &str) -> Result<u64, ErrorObjectOwned> {
    if !value.starts_with("0x") {
        return Err(invalid_params("block number must be hex quantity"));
    }
    let raw = value.trim_start_matches("0x");
    if raw.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(raw, 16).map_err(|_| invalid_params("invalid block number"))
}

fn parse_address_filter(value: &Value) -> Result<Vec<Address>, ErrorObjectOwned> {
    match value {
        Value::String(_) => Ok(vec![parse_address(value)?]),
        Value::Array(values) => {
            let mut out = Vec::new();
            for entry in values {
                out.push(parse_address(entry)?);
            }
            Ok(out)
        }
        _ => Err(invalid_params("address must be a string or array")),
    }
}

fn parse_address(value: &Value) -> Result<Address, ErrorObjectOwned> {
    let raw = value
        .as_str()
        .ok_or_else(|| invalid_params("address must be a string"))?;
    Address::from_str(raw).map_err(|_| invalid_params("invalid address"))
}

fn parse_topics_filter(value: &Value) -> Result<Vec<Option<Vec<B256>>>, ErrorObjectOwned> {
    let topics = value
        .as_array()
        .ok_or_else(|| invalid_params("topics must be an array"))?;
    let mut out = Vec::with_capacity(topics.len());
    for entry in topics {
        match entry {
            Value::Null => out.push(None),
            Value::String(_) => out.push(Some(vec![parse_b256(entry)?])),
            Value::Array(values) => {
                if values.is_empty() {
                    out.push(None);
                    continue;
                }
                let mut entries = Vec::new();
                for value in values {
                    entries.push(parse_b256(value)?);
                }
                out.push(Some(entries));
            }
            _ => return Err(invalid_params("topic entries must be null, string, or array")),
        }
    }
    Ok(out)
}

fn parse_b256(value: &Value) -> Result<B256, ErrorObjectOwned> {
    let raw = value
        .as_str()
        .ok_or_else(|| invalid_params("topic must be a string"))?;
    B256::from_str(raw).map_err(|_| invalid_params("invalid topic hash"))
}

fn log_matches(
    log: &StoredLog,
    addresses: Option<&[Address]>,
    topics: Option<&[Option<Vec<B256>>]>,
) -> bool {
    if let Some(addresses) = addresses {
        if !addresses.contains(&log.address) {
            return false;
        }
    }

    if let Some(filters) = topics {
        for (idx, filter) in filters.iter().enumerate() {
            let Some(filter_values) = filter else { continue };
            let Some(topic) = log.topics.get(idx) else { return false };
            if !filter_values.contains(topic) {
                return false;
            }
        }
    }

    true
}

fn format_log(log: &StoredLog) -> RpcLog {
    RpcLog {
        address: format!("{:#x}", log.address),
        topics: log.topics.iter().map(|topic| format!("{:#x}", topic)).collect(),
        data: format!("{:x}", log.data),
        block_number: format!("0x{:x}", log.block_number),
        block_hash: format!("{:#x}", log.block_hash),
        transaction_hash: format!("{:#x}", log.transaction_hash),
        transaction_index: format!("0x{:x}", log.transaction_index),
        log_index: format!("0x{:x}", log.log_index),
        removed: log.removed,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RpcBlock {
    number: String,
    hash: String,
    parent_hash: String,
    timestamp: String,
    logs_bloom: String,
    transactions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RpcLog {
    address: String,
    topics: Vec<String>,
    data: String,
    block_number: String,
    block_hash: String,
    transaction_hash: String,
    transaction_index: String,
    log_index: String,
    removed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::rpc_params;
    use reth_primitives_traits::Header;
    use std::path::PathBuf;
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

    fn header_with_number(number: u64) -> Header {
        let mut header = Header::default();
        header.number = number;
        header
    }

    async fn start_test_server(chain_id: u64, storage: Arc<Storage>) -> (SocketAddr, ServerHandle) {
        let server = ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = server.local_addr().expect("local addr");
        let handle = server.start(module(RpcContext { chain_id, storage }).expect("module"));
        (addr, handle)
    }

    #[tokio::test]
    async fn eth_chain_id_success() {
        let dir = temp_dir();
        let storage = Arc::new(Storage::open(&base_config(dir.clone())).expect("storage"));
        let (addr, handle) = start_test_server(1, storage).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let result: String = client
            .request("eth_chainId", rpc_params![])
            .await
            .expect("chain id");
        assert_eq!(result, "0x1");

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_chain_id_rejects_params() {
        let dir = temp_dir();
        let storage = Arc::new(Storage::open(&base_config(dir.clone())).expect("storage"));
        let (addr, handle) = start_test_server(1, storage).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let err = client
            .request::<String, _>("eth_chainId", rpc_params![1])
            .await
            .expect_err("invalid params");
        match err {
            jsonrpsee::core::ClientError::Call(err_obj) => {
                assert_eq!(err_obj.code(), -32602);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let dir = temp_dir();
        let storage = Arc::new(Storage::open(&base_config(dir.clone())).expect("storage"));
        let (addr, handle) = start_test_server(1, storage).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let err = client
            .request::<String, _>("web3_clientVersion", rpc_params![])
            .await
            .expect_err("method not found");
        match err {
            jsonrpsee::core::ClientError::Call(err_obj) => {
                assert_eq!(err_obj.code(), -32601);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_block_number_returns_last_indexed() {
        let dir = temp_dir();
        let storage = Storage::open(&base_config(dir.clone())).expect("storage");
        storage
            .set_last_indexed_block(42)
            .expect("set last indexed");
        let (addr, handle) = start_test_server(1, Arc::new(storage)).await;

        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let result: String = client
            .request("eth_blockNumber", rpc_params![])
            .await
            .expect("block number");
        assert_eq!(result, "0x2a");

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_get_block_by_number_returns_header() {
        let dir = temp_dir();
        let storage = Storage::open(&base_config(dir.clone())).expect("storage");

        let mut header = header_with_number(7);
        header.timestamp = 1000;
        storage
            .write_block_header(7, header.clone())
            .expect("write header");
        storage
            .write_block_tx_hashes(
                7,
                crate::storage::StoredTxHashes {
                    hashes: vec![B256::from([0x11u8; 32]), B256::from([0x22u8; 32])],
                },
            )
            .expect("write tx hashes");
        storage
            .set_last_indexed_block(7)
            .expect("set last indexed");

        let (addr, handle) = start_test_server(1, Arc::new(storage)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let result: Value = client
            .request("eth_getBlockByNumber", rpc_params!["latest", false])
            .await
            .expect("block by number");

        assert_eq!(result["number"], Value::String("0x7".to_string()));
        assert_eq!(result["timestamp"], Value::String("0x3e8".to_string()));
        assert!(result["hash"].as_str().is_some());
        assert!(result["logsBloom"].as_str().is_some());
        assert_eq!(result["transactions"].as_array().unwrap().len(), 2);

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_get_logs_filters_address_and_topic0() {
        let dir = temp_dir();
        let storage = Storage::open(&base_config(dir.clone())).expect("storage");

        let topic0 = B256::from([0x11u8; 32]);
        let log = StoredLog {
            address: Address::ZERO,
            topics: vec![topic0],
            block_number: 5,
            block_hash: B256::ZERO,
            transaction_hash: B256::from([0x33u8; 32]),
            transaction_index: 0,
            log_index: 0,
            removed: false,
            data: alloy_primitives::Bytes::from(vec![0x01]),
        };
        storage
            .write_block_logs(5, crate::storage::StoredLogs { logs: vec![log.clone()] })
            .expect("write logs");
        storage
            .write_log_indexes(&[log.clone()])
            .expect("write log indexes");
        storage
            .set_last_indexed_block(5)
            .expect("set last indexed");

        let (addr, handle) = start_test_server(1, Arc::new(storage)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");

        let filter = serde_json::json!({
            "fromBlock": "0x5",
            "toBlock": "0x5",
            "address": format!("{:#x}", Address::ZERO),
            "topics": [format!("{:#x}", topic0)],
        });

        let result: Vec<Value> = client
            .request("eth_getLogs", rpc_params![filter])
            .await
            .expect("get logs");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["blockNumber"], Value::String("0x5".to_string()));

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
