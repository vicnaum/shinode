//! Reorg detection and common-ancestor search for follow mode.

use crate::p2p::{request_headers_batch, request_headers_chunked_strict, NetworkPeer, PeerPool};
use crate::storage::Storage;
use alloy_primitives::B256;
use eyre::{eyre, Result};
use reth_primitives_traits::SealedHeader;
use std::collections::HashMap;

#[derive(Debug)]
pub enum ReorgCheck {
    NoReorg,
    ReorgDetected { anchor: NetworkPeer },
    Inconclusive,
}

pub async fn preflight_reorg(
    storage: &Storage,
    pool: &PeerPool,
    last_indexed: u64,
    start_block: u64,
    probe_peers: usize,
) -> Result<ReorgCheck> {
    let stored_header = storage
        .block_header(last_indexed)?
        .ok_or_else(|| eyre!("missing stored header for block {last_indexed}"))?;
    let stored_hash = SealedHeader::seal_slow(stored_header).hash();

    let peers = pool.snapshot();
    if peers.is_empty() {
        return Ok(ReorgCheck::Inconclusive);
    }

    let mut mismatch_anchor: Option<NetworkPeer> = None;
    let probe_peers = probe_peers.max(1);
    for peer in peers.iter().take(probe_peers) {

        if let Some(header) = fetch_single_header(peer, last_indexed).await? {
            let network_hash = SealedHeader::seal_slow(header).hash();
            if network_hash == stored_hash {
                return Ok(ReorgCheck::NoReorg);
            }
            mismatch_anchor.get_or_insert_with(|| peer.clone());
            continue;
        }

        if let Some(header) = fetch_single_header(peer, start_block).await? {
            if header.parent_hash == stored_hash {
                return Ok(ReorgCheck::NoReorg);
            }
            mismatch_anchor.get_or_insert_with(|| peer.clone());
        }
    }

    if let Some(anchor) = mismatch_anchor {
        return Ok(ReorgCheck::ReorgDetected { anchor });
    }

    Ok(ReorgCheck::Inconclusive)
}

pub async fn find_common_ancestor(
    storage: &Storage,
    anchor: &NetworkPeer,
    low: u64,
    high: u64,
) -> Result<Option<u64>> {
    if low > high {
        return Ok(None);
    }
    let count = (high - low + 1) as usize;
    let headers = request_headers_chunked_strict(anchor, low, count).await?;
    let network_hashes = headers
        .into_iter()
        .map(|header| (header.number, SealedHeader::seal_slow(header).hash()))
        .collect::<HashMap<u64, B256>>();

    let mut stored_hashes = HashMap::new();
    for number in low..=high {
        if let Some(header) = storage.block_header(number)? {
            let hash = SealedHeader::seal_slow(header).hash();
            stored_hashes.insert(number, hash);
        }
    }

    Ok(find_common_ancestor_number(
        &stored_hashes,
        &network_hashes,
        low,
        high,
    ))
}

fn find_common_ancestor_number(
    stored_hashes: &HashMap<u64, B256>,
    network_hashes: &HashMap<u64, B256>,
    low: u64,
    high: u64,
) -> Option<u64> {
    for number in (low..=high).rev() {
        match (stored_hashes.get(&number), network_hashes.get(&number)) {
            (Some(stored), Some(network)) if stored == network => return Some(number),
            _ => {}
        }
    }
    None
}

async fn fetch_single_header(
    peer: &NetworkPeer,
    number: u64,
) -> Result<Option<reth_primitives_traits::Header>> {
    let mut headers = request_headers_batch(peer, number, 1).await?;
    Ok(headers.pop())
}

#[cfg(test)]
mod tests {
    use super::{find_common_ancestor_number, preflight_reorg, ReorgCheck};
    use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use crate::p2p::{peer_pool_for_tests, NetworkPeer};
    use crate::storage::Storage;
    use alloy_primitives::B256;
    use reth_eth_wire::EthVersion;
    use reth_eth_wire_types::{BlockHashOrNumber, BlockHeaders};
    use reth_network_api::{PeerId, PeerRequest, PeerRequestSender};
    use reth_primitives_traits::{Header, SealedHeader};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::mpsc;

    fn b(val: u8) -> B256 {
        B256::from([val; 32])
    }

    fn temp_dir() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "stateless-history-node-reorg-test-{now}-{}-{suffix}",
            std::process::id()
        ));
        path
    }

    fn base_config(dir: std::path::PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir: dir,
            peer_cache_dir: None,
            rpc_bind: "127.0.0.1:0".parse().expect("rpc bind"),
            start_block: 0,
            shard_size: crate::cli::DEFAULT_SHARD_SIZE,
            end_block: None,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            run_name: None,
            log: false,
            log_output_dir: PathBuf::from(crate::cli::DEFAULT_LOG_OUTPUT_DIR),
            log_trace: false,
            log_trace_filter: crate::cli::DEFAULT_LOG_TRACE_FILTER.to_string(),
            log_trace_include_args: false,
            log_trace_include_locations: false,
            log_events: false,
            log_events_verbose: false,
            log_json: false,
            log_json_filter: crate::cli::DEFAULT_LOG_JSON_FILTER.to_string(),
            log_report: false,
            log_resources: false,
            min_peers: 1,
            repair: false,
            no_tui: false,
            command: None,
            rpc_max_request_body_bytes: 0,
            rpc_max_response_body_bytes: 0,
            rpc_max_connections: 0,
            rpc_max_batch_requests: 0,
            rpc_max_blocks_per_filter: 0,
            rpc_max_logs_per_response: 0,
            fast_sync_chunk_size: 16,
            fast_sync_chunk_max: None,
            fast_sync_max_inflight: 2,
            fast_sync_max_buffered_blocks: 64,
            db_write_batch_blocks: 1,
            db_write_flush_interval_ms: None,
            defer_compaction: false,
        }
    }

    fn header_with_number(number: u64, parent_hash: B256) -> Header {
        let mut header = Header::default();
        header.number = number;
        header.parent_hash = parent_hash;
        header
    }

    fn write_bundle(storage: &Storage, header: Header) {
        let bundle = crate::storage::BlockBundle {
            number: header.number,
            header,
            tx_hashes: crate::storage::StoredTxHashes { hashes: Vec::new() },
            transactions: crate::storage::StoredTransactions { txs: Vec::new() },
            size: crate::storage::StoredBlockSize { size: 0 },
            receipts: crate::storage::StoredReceipts {
                receipts: Vec::new(),
            },
        };
        storage
            .write_block_bundle_follow(&bundle)
            .expect("write bundle");
    }

    #[test]
    fn ancestor_picks_highest_match() {
        let stored = HashMap::from([(1u64, b(1)), (2, b(2)), (3, b(3))]);
        let network = HashMap::from([(1u64, b(1)), (2, b(9)), (3, b(3))]);
        assert_eq!(
            find_common_ancestor_number(&stored, &network, 1, 3),
            Some(3)
        );
    }

    #[test]
    fn ancestor_returns_none_when_no_match() {
        let stored = HashMap::from([(1u64, b(1)), (2, b(2))]);
        let network = HashMap::from([(1u64, b(3)), (2, b(4))]);
        assert_eq!(find_common_ancestor_number(&stored, &network, 1, 2), None);
    }

    #[tokio::test]
    async fn preflight_returns_no_reorg_on_matching_tip() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let stored_header = header_with_number(10, b(1));
        write_bundle(&storage, stored_header.clone());
        let stored_hash = SealedHeader::seal_slow(stored_header.clone()).hash();

        let (tx, mut rx) = mpsc::channel(8);
        let peer_id = PeerId::random();
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 12,
        };

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockHeaders { request, response } => {
                        let number = match request.start_block {
                            BlockHashOrNumber::Number(start) => start,
                            _ => 0,
                        };
                        let headers = match number {
                            10 => vec![stored_header.clone()],
                            11 => vec![header_with_number(11, stored_hash)],
                            _ => Vec::new(),
                        };
                        let _ = response.send(Ok(BlockHeaders::from(headers)));
                    }
                    _ => {}
                }
            }
        });

        let pool = peer_pool_for_tests(vec![peer]);
        let result = preflight_reorg(&storage, &pool, 10, 11, 1)
            .await
            .expect("preflight");
        assert!(matches!(result, ReorgCheck::NoReorg));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn preflight_detects_same_height_mismatch() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let stored_header = header_with_number(10, b(1));
        write_bundle(&storage, stored_header.clone());

        let (tx, mut rx) = mpsc::channel(8);
        let peer_id = PeerId::random();
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 12,
        };

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockHeaders { request, response } => {
                        let number = match request.start_block {
                            BlockHashOrNumber::Number(start) => start,
                            _ => 0,
                        };
                        let headers = match number {
                            10 => vec![header_with_number(10, b(2))],
                            11 => vec![header_with_number(11, b(3))],
                            _ => Vec::new(),
                        };
                        let _ = response.send(Ok(BlockHeaders::from(headers)));
                    }
                    _ => {}
                }
            }
        });

        let pool = peer_pool_for_tests(vec![peer]);
        let result = preflight_reorg(&storage, &pool, 10, 11, 1)
            .await
            .expect("preflight");
        assert!(matches!(result, ReorgCheck::ReorgDetected { .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn preflight_inconclusive_when_no_headers() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let stored_header = header_with_number(10, b(1));
        write_bundle(&storage, stored_header);

        let (tx, mut rx) = mpsc::channel(8);
        let peer_id = PeerId::random();
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 12,
        };

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let PeerRequest::GetBlockHeaders { response, .. } = request {
                    let _ = response.send(Ok(BlockHeaders::from(Vec::new())));
                }
            }
        });

        let pool = peer_pool_for_tests(vec![peer]);
        let result = preflight_reorg(&storage, &pool, 10, 11, 1)
            .await
            .expect("preflight");
        assert!(matches!(result, ReorgCheck::Inconclusive));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
