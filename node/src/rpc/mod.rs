//! JSON-RPC server.

use crate::{
    cli::{
        NodeConfig, DEFAULT_RPC_MAX_BATCH_REQUESTS, DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
        DEFAULT_RPC_MAX_CONNECTIONS, DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
        DEFAULT_RPC_MAX_REQUEST_BODY_BYTES, DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
    },
    storage::Storage,
};
use alloy_primitives::{Address, Bytes, B256};
use eyre::{Result, WrapErr};
use jsonrpsee::{
    server::{BatchRequestConfig, ServerBuilder, ServerConfig, ServerHandle},
    types::ErrorObjectOwned,
    RpcModule,
};
use reth_primitives_traits::SealedHeader;
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use tracing::{info, warn};

#[derive(Clone)]
pub struct RpcConfig {
    chain_id: u64,
    max_request_body_bytes: u32,
    max_response_body_bytes: u32,
    max_connections: u32,
    max_batch_requests: u32,
    max_blocks_per_filter: u64,
    max_logs_per_response: u64,
}

impl From<&NodeConfig> for RpcConfig {
    fn from(config: &NodeConfig) -> Self {
        Self {
            chain_id: config.chain_id,
            max_request_body_bytes: config.rpc_max_request_body_bytes,
            max_response_body_bytes: config.rpc_max_response_body_bytes,
            max_connections: config.rpc_max_connections,
            max_batch_requests: config.rpc_max_batch_requests,
            max_blocks_per_filter: config.rpc_max_blocks_per_filter,
            max_logs_per_response: config.rpc_max_logs_per_response,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            max_request_body_bytes: DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
            max_response_body_bytes: DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
            max_connections: DEFAULT_RPC_MAX_CONNECTIONS,
            max_batch_requests: DEFAULT_RPC_MAX_BATCH_REQUESTS,
            max_blocks_per_filter: DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
            max_logs_per_response: DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
        }
    }
}

#[derive(Clone)]
pub struct RpcContext {
    config: RpcConfig,
    storage: Arc<Storage>,
}

/// Start the JSON-RPC server and return its handle.
pub async fn start(
    bind: SocketAddr,
    config: RpcConfig,
    storage: Arc<Storage>,
) -> Result<ServerHandle> {
    let batch_config = if config.max_batch_requests == 0 {
        BatchRequestConfig::Unlimited
    } else {
        BatchRequestConfig::Limit(config.max_batch_requests)
    };
    let server_config = ServerConfig::builder()
        .max_request_body_size(config.max_request_body_bytes)
        .max_response_body_size(config.max_response_body_bytes)
        .max_connections(config.max_connections)
        .set_batch_request_config(batch_config)
        .build();
    let server = ServerBuilder::new()
        .set_config(server_config)
        .build(bind)
        .await
        .wrap_err("failed to bind RPC server")?;

    Ok(server.start(module(RpcContext { config, storage })?))
}

pub fn module(ctx: RpcContext) -> Result<RpcModule<RpcContext>> {
    let mut module = RpcModule::new(ctx);
    module
        .register_method(
            "eth_chainId",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
                let params: Option<Value> = params.parse()?;
                ensure_empty_params(params, "eth_chainId expects no params")?;

                info!(method = "eth_chainId", "rpc request");
                Ok(format!("0x{:x}", ctx.config.chain_id))
            },
        )
        .wrap_err("failed to register eth_chainId")?;

    module
        .register_method(
            "eth_blockNumber",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
                let params: Option<Value> = params.parse()?;
                ensure_empty_params(params, "eth_blockNumber expects no params")?;

                info!(method = "eth_blockNumber", "rpc request");
                let latest = ctx
                    .storage
                    .last_indexed_block()
                    .map_err(internal_error)?
                    .unwrap_or(0);
                info!(method = "eth_blockNumber", latest, "rpc response");
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
                info!(
                    method = "eth_getBlockByNumber",
                    block_tag = ?params[0],
                    include_txs,
                    "rpc request"
                );
                if include_txs {
                    return Err(invalid_params(
                        "eth_getBlockByNumber only supports includeTransactions=false",
                    ));
                }

                let tag = parse_block_tag(&params[0])?;
                let latest = ctx.storage.last_indexed_block().map_err(internal_error)?;
                let Some(number) = resolve_block_tag_optional(tag, latest)? else {
                    info!(
                        method = "eth_getBlockByNumber",
                        found = false,
                        "rpc response"
                    );
                    return Ok(Value::Null);
                };

                let Some(header) = ctx.storage.block_header(number).map_err(internal_error)? else {
                    info!(
                        method = "eth_getBlockByNumber",
                        found = false,
                        "rpc response"
                    );
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
                let size = ctx
                    .storage
                    .block_size(number)
                    .map_err(internal_error)?
                    .map(|stored| stored.size)
                    .unwrap_or(0);
                let withdrawals = None;

                let response = RpcBlock {
                    number: format!("0x{:x}", header.number),
                    hash: format!("{:#x}", hash),
                    parent_hash: format!("{:#x}", header.parent_hash),
                    nonce: format!("{:#x}", header.nonce),
                    sha3_uncles: format!("{:#x}", header.ommers_hash),
                    timestamp: format!("0x{:x}", header.timestamp),
                    logs_bloom: format!("{:#x}", header.logs_bloom),
                    transactions_root: format!("{:#x}", header.transactions_root),
                    state_root: format!("{:#x}", header.state_root),
                    receipts_root: format!("{:#x}", header.receipts_root),
                    miner: format!("{:#x}", header.beneficiary),
                    difficulty: format!("{:#x}", header.difficulty),
                    total_difficulty: "0x0".to_string(),
                    extra_data: format!("{:#x}", header.extra_data),
                    size: format!("0x{:x}", size),
                    gas_limit: format!("0x{:x}", header.gas_limit),
                    gas_used: format!("0x{:x}", header.gas_used),
                    transactions: txs,
                    uncles: Vec::new(),
                    mix_hash: format!("{:#x}", header.mix_hash),
                    base_fee_per_gas: header.base_fee_per_gas.map(|fee| format!("0x{:x}", fee)),
                    withdrawals_root: header.withdrawals_root.map(|root| format!("{:#x}", root)),
                    withdrawals,
                    blob_gas_used: header.blob_gas_used.map(|gas| format!("0x{:x}", gas)),
                    excess_blob_gas: header.excess_blob_gas.map(|gas| format!("0x{:x}", gas)),
                    parent_beacon_block_root: header
                        .parent_beacon_block_root
                        .map(|root| format!("{:#x}", root)),
                };

                info!(
                    method = "eth_getBlockByNumber",
                    found = true,
                    number = header.number,
                    "rpc response"
                );
                serde_json::to_value(response).map_err(internal_error)
            },
        )
        .wrap_err("failed to register eth_getBlockByNumber")?;

    module
        .register_method(
            "eth_getLogs",
            |params, ctx, _| -> Result<_, ErrorObjectOwned> {
                let params: Vec<Value> = params
                    .parse()
                    .map_err(|_| invalid_params("eth_getLogs expects a single filter object"))?;
                if params.len() != 1 {
                    return Err(invalid_params("eth_getLogs expects a single filter object"));
                }
                let filter = params[0]
                    .as_object()
                    .ok_or_else(|| invalid_params("eth_getLogs expects a filter object"))?;
                if filter.contains_key("blockHash") {
                    return Err(invalid_params("eth_getLogs does not support blockHash yet"));
                }

                let latest = ctx
                    .storage
                    .last_indexed_block()
                    .map_err(internal_error)?
                    .unwrap_or(0);
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
                let address_filter = filter
                    .get("address")
                    .map(parse_address_filter)
                    .transpose()?;
                let topics_filter = filter.get("topics").map(parse_topics_filter).transpose()?;
                let topic0_filter = topics_filter
                    .as_ref()
                    .and_then(|topics| topics.first())
                    .and_then(|entry| entry.clone());
                let address_count = address_filter.as_ref().map(|v| v.len()).unwrap_or(0);
                let topic0_count = topic0_filter.as_ref().map(|v| v.len()).unwrap_or(0);
                info!(
                    method = "eth_getLogs",
                    from_block, to_block, address_count, topic0_count, "rpc request"
                );
                if from_block > to_block {
                    return Ok(Value::Array(Vec::new()));
                }
                let block_span = to_block.saturating_sub(from_block).saturating_add(1);
                if ctx.config.max_blocks_per_filter != 0
                    && block_span > ctx.config.max_blocks_per_filter
                {
                    return Err(invalid_params("block range exceeds max_blocks_per_filter"));
                }

                for block in from_block..=to_block {
                    if !ctx.storage.has_block(block).map_err(internal_error)? {
                        return Err(missing_block_error(block));
                    }
                }

                let headers = ctx
                    .storage
                    .block_headers_range(from_block..=to_block)
                    .map_err(internal_error)?;
                let tx_hashes = ctx
                    .storage
                    .block_tx_hashes_range(from_block..=to_block)
                    .map_err(internal_error)?;
                let receipts = ctx
                    .storage
                    .block_receipts_range(from_block..=to_block)
                    .map_err(internal_error)?;
                let headers_by_number: HashMap<u64, _> = headers.into_iter().collect();
                let tx_hashes_by_number: HashMap<u64, _> = tx_hashes.into_iter().collect();

                let mut out = Vec::new();
                for (block_number, stored) in receipts {
                    let Some(header) = headers_by_number.get(&block_number) else {
                        continue;
                    };
                    let Some(tx_hashes) = tx_hashes_by_number.get(&block_number) else {
                        continue;
                    };
                    if stored.receipts.len() != tx_hashes.hashes.len() {
                        warn!(
                            block_number,
                            receipts = stored.receipts.len(),
                            tx_hashes = tx_hashes.hashes.len(),
                            "receipt/tx hash count mismatch while deriving logs"
                        );
                        continue;
                    }
                    let block_hash = SealedHeader::seal_slow(header.clone()).hash();
                    for (tx_index, (receipt, tx_hash)) in stored
                        .receipts
                        .iter()
                        .zip(tx_hashes.hashes.iter())
                        .enumerate()
                    {
                        for (log_index, log) in receipt.logs.iter().cloned().enumerate() {
                            let alloy_primitives::Log { address, data } = log;
                            let (topics, data) = data.split();
                            let derived_log = DerivedLog {
                                address,
                                topics,
                                data,
                                block_number,
                                block_hash,
                                transaction_hash: *tx_hash,
                                transaction_index: tx_index as u64,
                                log_index: log_index as u64,
                                removed: false,
                            };
                            if log_matches(
                                &derived_log,
                                address_filter.as_deref(),
                                topics_filter.as_deref(),
                            ) {
                                out.push(format_log(&derived_log));
                                if ctx.config.max_logs_per_response != 0
                                    && out.len() > ctx.config.max_logs_per_response as usize
                                {
                                    return Err(invalid_params(
                                        "response exceeds max_logs_per_response",
                                    ));
                                }
                            }
                        }
                    }
                }
                let logs = out;

                info!(method = "eth_getLogs", logs = logs.len(), "rpc response");
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

fn missing_block_error(block: u64) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        -32001,
        "missing block in requested range",
        Some(json!({ "missing_block": format!("0x{:x}", block) })),
    )
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
            _ => {
                return Err(invalid_params(
                    "topic entries must be null, string, or array",
                ))
            }
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

#[derive(Debug)]
struct DerivedLog {
    address: Address,
    topics: Vec<B256>,
    data: Bytes,
    block_number: u64,
    block_hash: B256,
    transaction_hash: B256,
    transaction_index: u64,
    log_index: u64,
    removed: bool,
}

fn log_matches(
    log: &DerivedLog,
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
            let Some(filter_values) = filter else {
                continue;
            };
            let Some(topic) = log.topics.get(idx) else {
                return false;
            };
            if !filter_values.contains(topic) {
                return false;
            }
        }
    }

    true
}

fn format_log(log: &DerivedLog) -> RpcLog {
    RpcLog {
        address: format!("{:#x}", log.address),
        topics: log
            .topics
            .iter()
            .map(|topic| format!("{:#x}", topic))
            .collect(),
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
    nonce: String,
    sha3_uncles: String,
    timestamp: String,
    logs_bloom: String,
    transactions_root: String,
    state_root: String,
    receipts_root: String,
    miner: String,
    difficulty: String,
    total_difficulty: String,
    extra_data: String,
    size: String,
    gas_limit: String,
    gas_used: String,
    transactions: Vec<String>,
    uncles: Vec<String>,
    mix_hash: String,
    base_fee_per_gas: Option<String>,
    withdrawals_root: Option<String>,
    /// Withdrawals are not supported yet; we currently always return `null`.
    withdrawals: Option<Vec<Value>>,
    blob_gas_used: Option<String>,
    excess_blob_gas: Option<String>,
    parent_beacon_block_root: Option<String>,
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
    use crate::storage::BlockBundle;
    use crate::test_utils::{base_config, temp_dir};
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::rpc_params;
    use reth_ethereum_primitives::{Receipt, TxType};
    use reth_primitives_traits::Header;

    fn header_with_number(number: u64) -> Header {
        let mut header = Header::default();
        header.number = number;
        header
    }

    fn write_bundle(storage: &Storage, bundle: BlockBundle) {
        storage
            .write_block_bundle_follow(&bundle)
            .expect("write bundle");
    }

    fn bundle_with_number(
        number: u64,
        header: Header,
        tx_hashes: Vec<B256>,
        receipts: Vec<reth_ethereum_primitives::Receipt>,
        size: u64,
    ) -> BlockBundle {
        BlockBundle {
            number,
            header,
            tx_hashes: crate::storage::StoredTxHashes { hashes: tx_hashes },
            size: crate::storage::StoredBlockSize { size },
            receipts: crate::storage::StoredReceipts { receipts },
        }
    }

    fn receipt_with_logs(logs: Vec<alloy_primitives::Log>) -> Receipt {
        Receipt {
            tx_type: TxType::Legacy,
            success: true,
            cumulative_gas_used: 0,
            logs,
        }
    }

    async fn start_test_server(
        chain_id: u64,
        storage: Arc<Storage>,
        rpc_config: RpcConfig,
    ) -> (SocketAddr, ServerHandle) {
        let server = ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = server.local_addr().expect("local addr");
        let handle = server.start(
            module(RpcContext {
                config: RpcConfig {
                    chain_id,
                    ..rpc_config
                },
                storage,
            })
            .expect("module"),
        );
        (addr, handle)
    }

    #[tokio::test]
    async fn eth_chain_id_success() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("storage"));
        let (addr, handle) = start_test_server(1, storage, RpcConfig::from(&config)).await;
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
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("storage"));
        let (addr, handle) = start_test_server(1, storage, RpcConfig::from(&config)).await;
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
    async fn rpc_shutdown_stops_server() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("storage"));
        let (addr, handle) = start_test_server(1, storage, RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let _: String = client
            .request("eth_chainId", rpc_params![])
            .await
            .expect("chain id");

        handle.stop().expect("stop server");
        handle.stopped().await;

        let err = client
            .request::<String, _>("eth_chainId", rpc_params![])
            .await;
        assert!(err.is_err(), "expected request to fail after shutdown");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("storage"));
        let (addr, handle) = start_test_server(1, storage, RpcConfig::from(&config)).await;
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
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");
        storage
            .set_last_indexed_block(42)
            .expect("set last indexed");
        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;

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
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let mut header = header_with_number(7);
        header.timestamp = 1000;
        write_bundle(
            &storage,
            bundle_with_number(
                7,
                header.clone(),
                vec![B256::from([0x11u8; 32]), B256::from([0x22u8; 32])],
                Vec::new(),
                123,
            ),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
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
        assert_eq!(result["totalDifficulty"], Value::String("0x0".to_string()));
        assert_eq!(result["size"], Value::String("0x7b".to_string()));
        assert!(result["nonce"].as_str().is_some());
        assert!(result["sha3Uncles"].as_str().is_some());
        assert!(result["stateRoot"].as_str().is_some());
        assert!(result["transactionsRoot"].as_str().is_some());
        assert!(result["receiptsRoot"].as_str().is_some());
        assert!(result["mixHash"].as_str().is_some());

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_get_block_by_number_rejects_include_transactions() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");
        write_bundle(
            &storage,
            bundle_with_number(1, header_with_number(1), Vec::new(), Vec::new(), 0),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");

        let err = client
            .request::<Value, _>("eth_getBlockByNumber", rpc_params!["latest", true])
            .await
            .expect_err("includeTransactions=true should fail");
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
    async fn eth_get_block_by_number_includes_withdrawals() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let mut header = header_with_number(9);
        header.withdrawals_root = Some(B256::from([0x55u8; 32]));
        write_bundle(
            &storage,
            bundle_with_number(9, header.clone(), Vec::new(), Vec::new(), 0),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let result: Value = client
            .request("eth_getBlockByNumber", rpc_params!["latest", false])
            .await
            .expect("block by number");

        assert!(result["withdrawals"].is_null());
        assert_eq!(
            result["withdrawalsRoot"],
            Value::String(format!("{:#x}", header.withdrawals_root.unwrap()))
        );

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_get_logs_filters_address_and_topic0() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let topic0 = B256::from([0x11u8; 32]);
        let header = header_with_number(5);
        let tx_hash = B256::from([0x33u8; 32]);
        let log = alloy_primitives::Log::new_unchecked(
            Address::ZERO,
            vec![topic0],
            alloy_primitives::Bytes::from(vec![0x01]),
        );
        write_bundle(
            &storage,
            bundle_with_number(
                5,
                header,
                vec![tx_hash],
                vec![receipt_with_logs(vec![log.clone()])],
                0,
            ),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
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

    #[tokio::test]
    async fn eth_get_logs_rejects_large_range() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");
        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");

        let to_block = format!("0x{:x}", config.rpc_max_blocks_per_filter + 1);
        let filter = serde_json::json!({
            "fromBlock": "0x0",
            "toBlock": to_block,
        });

        let err = client
            .request::<Vec<Value>, _>("eth_getLogs", rpc_params![filter])
            .await
            .expect_err("range too large");
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
    async fn eth_get_logs_errors_on_missing_block() {
        let dir = temp_dir("rpc");
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let header = header_with_number(1);
        write_bundle(
            &storage,
            bundle_with_number(1, header, Vec::new(), Vec::new(), 0),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");

        let filter = serde_json::json!({
            "fromBlock": "0x0",
            "toBlock": "0x1",
        });

        let err = client
            .request::<Vec<Value>, _>("eth_getLogs", rpc_params![filter])
            .await
            .expect_err("missing block should error");
        match err {
            jsonrpsee::core::ClientError::Call(err_obj) => {
                assert_eq!(err_obj.code(), -32001);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn eth_get_logs_allows_unlimited_limits() {
        let dir = temp_dir("rpc");
        let mut config = base_config(dir.clone());
        config.rpc_max_blocks_per_filter = 0;
        config.rpc_max_logs_per_response = 0;
        let storage = Storage::open(&config).expect("storage");

        let topic0 = B256::from([0x42u8; 32]);
        let header = header_with_number(1);
        let tx_hash = B256::from([0x33u8; 32]);
        let log = alloy_primitives::Log::new_unchecked(
            Address::from([0x11u8; 20]),
            vec![topic0],
            alloy_primitives::Bytes::from(vec![0x01]),
        );
        write_bundle(
            &storage,
            bundle_with_number(
                1,
                header,
                vec![tx_hash],
                vec![receipt_with_logs(vec![log.clone()])],
                0,
            ),
        );

        let (addr, handle) =
            start_test_server(1, Arc::new(storage), RpcConfig::from(&config)).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");

        let filter = serde_json::json!({
            "fromBlock": "0x1",
            "toBlock": "0x1",
        });
        let result: Vec<Value> = client
            .request("eth_getLogs", rpc_params![filter])
            .await
            .expect("get logs");
        assert_eq!(result.len(), 1);

        handle.stop().expect("stop server");
        handle.stopped().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
