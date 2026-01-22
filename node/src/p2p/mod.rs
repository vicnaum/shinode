//! P2P subsystem.

use crate::storage::{Storage, StoredPeer};
use crate::sync::{BlockPayload, BlockPayloadSource};
use alloy_primitives::B256;
use async_trait::async_trait;
use eyre::{eyre, Result, WrapErr};
use futures::StreamExt;
use reth_chainspec::MAINNET;
use reth_eth_wire::{EthNetworkPrimitives, EthVersion};
use reth_eth_wire_types::{
    BlockHashOrNumber, GetBlockBodies, GetBlockHeaders, GetReceipts, GetReceipts70,
    HeadersDirection,
};
use reth_network::config::{rng_secret_key, NetworkConfigBuilder};
use reth_network::PeersConfig;
use reth_network::import::ProofOfStakeBlockImport;
use reth_network::NetworkHandle;
use reth_network_api::{
    events::PeerEvent,
    DiscoveryEvent,
    DiscoveredEvent,
    NetworkEvent,
    NetworkEventListenerProvider,
    PeerId,
    PeerKind,
    PeerRequest,
    PeerRequestSender,
    Peers,
};
use tracing::{info, warn};
use reth_primitives_traits::{Header, SealedHeader};
use reth_ethereum_primitives::Receipt;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::oneshot;
use tokio::sync::Semaphore;
use tokio::time::{sleep, timeout, Duration, Instant};

/// Identifier for a peer in the selection pool.
#[allow(dead_code)]
pub type SelectorPeerId = String;

/// Peer selector abstraction.
#[allow(dead_code)]
pub trait PeerSelector: Send + Sync {
    fn next_peer(&mut self) -> Option<SelectorPeerId>;
}

/// Round-robin peer selector for test scaffolding.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RoundRobinPeerSelector {
    peers: Vec<SelectorPeerId>,
    next_index: usize,
}

#[allow(dead_code)]
impl RoundRobinPeerSelector {
    pub fn new(peers: Vec<SelectorPeerId>) -> Self {
        Self { peers, next_index: 0 }
    }
}

impl PeerSelector for RoundRobinPeerSelector {
    fn next_peer(&mut self) -> Option<SelectorPeerId> {
        if self.peers.is_empty() {
            return None;
        }
        let peer = self.peers[self.next_index].clone();
        self.next_index = (self.next_index + 1) % self.peers.len();
        Some(peer)
    }
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
const MIN_PEER_START: usize = 1;
const PEER_DISCOVERY_TIMEOUT: Option<Duration> = None;
const PEER_START_WARMUP_SECS: u64 = 2;
const MAX_OUTBOUND: usize = 400;
const MAX_CONCURRENT_DIALS: usize = 100;
const PEER_REFILL_INTERVAL_MS: u64 = 500;
const MAX_HEADERS_PER_REQUEST: usize = 1024;
const PEER_CACHE_TTL_DAYS: u64 = 7;
const PEER_CACHE_MAX: usize = 5000;
static HEAD_PROBE_CURSOR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub(crate) struct P2pLimits {
    pub max_outbound: usize,
    pub max_concurrent_dials: usize,
    pub peer_refill_interval_ms: u64,
    pub request_timeout_ms: u64,
    pub max_headers_per_request: usize,
    pub peer_cache_ttl_days: u64,
    pub peer_cache_max: usize,
}

pub(crate) fn p2p_limits() -> P2pLimits {
    P2pLimits {
        max_outbound: MAX_OUTBOUND,
        max_concurrent_dials: MAX_CONCURRENT_DIALS,
        peer_refill_interval_ms: PEER_REFILL_INTERVAL_MS,
        request_timeout_ms: REQUEST_TIMEOUT.as_millis() as u64,
        max_headers_per_request: MAX_HEADERS_PER_REQUEST,
        peer_cache_ttl_days: PEER_CACHE_TTL_DAYS,
        peer_cache_max: PEER_CACHE_MAX,
    }
}

#[derive(Debug)]
struct PeerCacheBuffer {
    closed: AtomicBool,
    peers: RwLock<HashMap<String, StoredPeer>>,
}

impl PeerCacheBuffer {
    fn new() -> Self {
        Self {
            closed: AtomicBool::new(false),
            peers: RwLock::new(HashMap::new()),
        }
    }

    fn upsert(&self, peer: StoredPeer) {
        if self.closed.load(Ordering::Relaxed) {
            return;
        }
        let mut peers = self.peers.write().expect("peer cache lock");
        match peers.get_mut(&peer.peer_id) {
            Some(existing) => {
                existing.last_seen_ms = existing.last_seen_ms.max(peer.last_seen_ms);
                existing.tcp_addr = peer.tcp_addr;
                if peer.udp_addr.is_some() {
                    existing.udp_addr = peer.udp_addr;
                }
            }
            None => {
                peers.insert(peer.peer_id.clone(), peer);
            }
        }
    }

    fn close_and_drain(&self) -> Vec<StoredPeer> {
        self.closed.store(true, Ordering::SeqCst);
        let mut peers = self.peers.write().expect("peer cache lock");
        peers.drain().map(|(_, peer)| peer).collect()
    }
}

#[derive(Debug)]
struct PeerCacheHandle {
    storage: Arc<Storage>,
    buffer: Arc<PeerCacheBuffer>,
}

/// Active peer session information used for requests.
#[derive(Clone, Debug)]
pub struct NetworkPeer {
    pub peer_id: PeerId,
    pub eth_version: EthVersion,
    pub messages: PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    pub head_number: u64,
}

/// Wrapper that keeps the network handle alive.
#[derive(Debug)]
pub struct NetworkSession {
    pub _handle: NetworkHandle<EthNetworkPrimitives>,
    pub pool: Arc<PeerPool>,
    peer_cache: Option<PeerCacheHandle>,
}

/// Start the devp2p network and wait for compatible peers.
pub async fn connect_mainnet_peers(storage: Option<Arc<Storage>>) -> Result<NetworkSession> {
    let secret_key = rng_secret_key();
    let peers_config = PeersConfig::default()
        .with_max_outbound(MAX_OUTBOUND)
        .with_max_concurrent_dials(MAX_CONCURRENT_DIALS)
        .with_refill_slots_interval(Duration::from_millis(PEER_REFILL_INTERVAL_MS));
    let net_config = NetworkConfigBuilder::<EthNetworkPrimitives>::new(secret_key)
        .mainnet_boot_nodes()
        .with_unused_ports()
        .peer_config(peers_config)
        .disable_tx_gossip(true)
        .block_import(Box::new(ProofOfStakeBlockImport::default()))
        .build_with_noop_provider(MAINNET.clone());

    let handle = net_config
        .start_network()
        .await
        .wrap_err("failed to start p2p network")?;
    let pool = Arc::new(PeerPool::new());
    let peer_cache = storage.map(|storage| PeerCacheHandle {
        storage,
        buffer: Arc::new(PeerCacheBuffer::new()),
    });
    if let Some(cache) = peer_cache.as_ref() {
        seed_peer_cache(&handle, &cache.storage)?;
        spawn_peer_discovery_watcher(handle.clone(), Arc::clone(&cache.buffer));
    }
    spawn_peer_watcher(
        handle.clone(),
        Arc::clone(&pool),
        peer_cache.as_ref().map(|cache| Arc::clone(&cache.buffer)),
    );
    let warmup_started = Instant::now();
    let _connected = wait_for_peer_pool(
        Arc::clone(&pool),
        MIN_PEER_START,
        PEER_DISCOVERY_TIMEOUT,
    )
    .await?;
    if PEER_START_WARMUP_SECS > 0 {
        let min = Duration::from_secs(PEER_START_WARMUP_SECS);
        let elapsed = warmup_started.elapsed();
        if elapsed < min {
            sleep(min - elapsed).await;
        }
    }
    Ok(NetworkSession {
        _handle: handle,
        pool,
        peer_cache,
    })
}

impl NetworkSession {
    pub fn flush_peer_cache(&self) -> Result<()> {
        let Some(cache) = self.peer_cache.as_ref() else {
            return Ok(());
        };
        let peers = cache.buffer.close_and_drain();
        if peers.is_empty() {
            info!("peer cache flush: no entries");
            return Ok(());
        }
        let mut failed = 0usize;
        for peer in peers.iter() {
            if let Err(err) = cache.storage.upsert_peer(peer.clone()) {
                failed += 1;
                warn!(peer_id = peer.peer_id, error = %err, "failed to persist cached peer");
            }
        }
        if let Err(err) = cache.storage.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache");
        }
        info!(
            cache_flush_total = peers.len(),
            cache_flush_failed = failed,
            "peer cache flush complete"
        );
        Ok(())
    }
}

#[derive(Debug)]
pub struct PeerPool {
    peers: RwLock<Vec<NetworkPeer>>,
}

impl PeerPool {
    fn new() -> Self {
        Self {
            peers: RwLock::new(Vec::new()),
        }
    }

    fn best_head(&self) -> Option<u64> {
        let peers = self.peers.read().expect("peer pool lock");
        peers.iter().map(|peer| peer.head_number).max()
    }

    pub fn len(&self) -> usize {
        let peers = self.peers.read().expect("peer pool lock");
        peers.len()
    }

    pub fn snapshot(&self) -> Vec<NetworkPeer> {
        let peers = self.peers.read().expect("peer pool lock");
        peers.clone()
    }

    fn add_peer(&self, peer: NetworkPeer) {
        let mut peers = self.peers.write().expect("peer pool lock");
        if peers.iter().any(|existing| existing.peer_id == peer.peer_id) {
            return;
        }
        peers.push(peer);
    }

    fn update_peer_head(&self, peer_id: PeerId, head_number: u64) {
        let mut peers = self.peers.write().expect("peer pool lock");
        if let Some(peer) = peers.iter_mut().find(|peer| peer.peer_id == peer_id) {
            peer.head_number = head_number;
        }
    }

    fn remove_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.write().expect("peer pool lock");
        peers.retain(|peer| peer.peer_id != peer_id);
    }
}

#[cfg(test)]
pub(crate) fn peer_pool_for_tests(peers: Vec<NetworkPeer>) -> PeerPool {
    let pool = PeerPool::new();
    for peer in peers {
        pool.add_peer(peer);
    }
    pool
}

/// P2P-backed block payload source with multi-peer retries.
#[derive(Clone, Debug)]
pub struct MultiPeerBlockPayloadSource {
    pool: Arc<PeerPool>,
}

impl MultiPeerBlockPayloadSource {
    pub fn new(pool: Arc<PeerPool>) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl BlockPayloadSource for MultiPeerBlockPayloadSource {
    async fn head(&self) -> Result<u64> {
        self.pool
            .best_head()
            .ok_or_else(|| eyre!("no peers available for head"))
    }
}

fn spawn_peer_watcher(
    handle: NetworkHandle<EthNetworkPrimitives>,
    pool: Arc<PeerPool>,
    peer_cache: Option<Arc<PeerCacheBuffer>>,
) {
    tokio::spawn(async move {
        let mut events = handle.event_listener();
        let head_probe_semaphore = Arc::new(Semaphore::new(24));
        while let Some(event) = events.next().await {
            match event {
                NetworkEvent::ActivePeerSession { info, messages } => {
                    if info.status.genesis != MAINNET.genesis_hash() {
                        continue;
                    }
                    let peer_id = info.peer_id;
                    let head_hash = info.status.blockhash;
                    let messages_for_peer = messages.clone();
                    let messages_for_probe = messages.clone();
                    pool.add_peer(NetworkPeer {
                        peer_id,
                        eth_version: info.version,
                        messages: messages_for_peer,
                        head_number: 0,
                    });
                    let pool_for_probe = Arc::clone(&pool);
                    let head_probe_semaphore = Arc::clone(&head_probe_semaphore);
                    tokio::spawn(async move {
                        let Ok(_permit) = head_probe_semaphore.acquire_owned().await else {
                            return;
                        };
                        match request_head_number(peer_id, head_hash, &messages_for_probe).await {
                            Ok(head_number) => {
                                pool_for_probe.update_peer_head(peer_id, head_number);
                            }
                            Err(err) => {
                                tracing::debug!(
                                    peer_id = ?peer_id,
                                    error = %err,
                                    "failed to probe peer head; keeping peer with unknown head"
                                );
                            }
                        }
                    });
                    if let Some(peer_cache) = peer_cache.as_ref() {
                        let peer = StoredPeer {
                            peer_id: info.peer_id.to_string(),
                            tcp_addr: info.remote_addr,
                            udp_addr: None,
                            last_seen_ms: now_ms(),
                            aimd_batch_limit: None,
                        };
                        peer_cache.upsert(peer);
                    }
                }
                NetworkEvent::Peer(PeerEvent::SessionClosed { peer_id, .. })
                | NetworkEvent::Peer(PeerEvent::PeerRemoved(peer_id)) => {
                    pool.remove_peer(peer_id);
                }
                _ => {}
            }
        }
    });
}

fn spawn_peer_discovery_watcher(
    handle: NetworkHandle<EthNetworkPrimitives>,
    peer_cache: Arc<PeerCacheBuffer>,
) {
    tokio::spawn(async move {
        let mut events = handle.discovery_listener();
        while let Some(event) = events.next().await {
            if let DiscoveryEvent::NewNode(DiscoveredEvent::EventQueued { peer_id, addr, .. }) =
                event
            {
                let peer = StoredPeer {
                    peer_id: peer_id.to_string(),
                    tcp_addr: addr.tcp(),
                    udp_addr: addr.udp(),
                    last_seen_ms: now_ms(),
                    aimd_batch_limit: None,
                };
                peer_cache.upsert(peer);
            }
        }
    });
}

fn seed_peer_cache(
    handle: &NetworkHandle<EthNetworkPrimitives>,
    storage: &Storage,
) -> Result<()> {
    let ttl_ms = Duration::from_secs(PEER_CACHE_TTL_DAYS * 24 * 60 * 60)
        .as_millis() as u64;
    let expire_before_ms = now_ms().saturating_sub(ttl_ms);
    let load = storage.load_peers(expire_before_ms, PEER_CACHE_MAX)?;
    let mut seeded = 0usize;
    let mut invalid = 0usize;
    let kept = load.peers.len();
    for peer in &load.peers {
        match PeerId::from_str(&peer.peer_id) {
            Ok(peer_id) => {
                handle.add_peer_kind(peer_id, PeerKind::Static, peer.tcp_addr, peer.udp_addr);
                seeded += 1;
            }
            Err(err) => {
                invalid += 1;
                warn!(peer_id = peer.peer_id, error = %err, "invalid cached peer id");
            }
        }
    }
    info!(
        cache_total = load.total,
        cache_expired = load.expired,
        cache_corrupted = load.corrupted,
        cache_capped = load.capped,
        cache_kept = kept,
        cache_seeded = seeded,
        cache_invalid = invalid,
        "peer cache summary"
    );
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn wait_for_peer_pool(
    pool: Arc<PeerPool>,
    target: usize,
    timeout_after: Option<Duration>,
) -> Result<usize> {
    let deadline = timeout_after.map(|duration| Instant::now() + duration);

    loop {
        let peers = pool.len();
        if peers >= target {
            return Ok(peers);
        }

        if let Some(deadline) = deadline {
            let now = Instant::now();
            if now >= deadline {
                if peers == 0 {
                    return Err(eyre!(
                        "no peers connected within {:?}; check network access",
                        timeout_after.unwrap_or_default()
                    ));
                }
                return Ok(peers);
            }
        }

        sleep(Duration::from_millis(200)).await;
    }
}

async fn request_head_number(
    peer_id: PeerId,
    head_hash: B256,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<u64> {
    let headers = request_headers_by_hash(peer_id, head_hash, messages).await?;
    let header = headers
        .get(0)
        .ok_or_else(|| eyre!("empty header response for head"))?;
    Ok(header.number)
}

pub(crate) async fn request_headers_batch(
    peer: &NetworkPeer,
    start_block: u64,
    limit: usize,
) -> Result<Vec<Header>> {
    request_headers_by_number(peer.peer_id, start_block, limit, &peer.messages)
        .await
}

pub(crate) async fn discover_head_p2p(
    pool: &PeerPool,
    baseline: u64,
    probe_peers: usize,
    probe_limit: usize,
) -> Result<Option<u64>> {
    let peers = pool.snapshot();
    if peers.is_empty() {
        return Ok(None);
    }

    // IMPORTANT: do not trust `peer.head_number` as a head signal for follow mode.
    //
    // Many peers will return a `Status` best hash, but later refuse to serve headers by number
    // (or will be behind / on a different fork). If we treat `head_number` as authoritative, we
    // will tip-chase and spam `GetBlockHeaders` beyond the peer's view.
    //
    // Instead, only advance the observed head if we can actually fetch headers above `baseline`.
    let mut best = baseline;
    let probe_peers = probe_peers.max(1);
    let probe_limit = probe_limit.max(1).min(MAX_HEADERS_PER_REQUEST);
    let start = baseline.saturating_add(1);

    let mut probed = 0usize;
    let len = peers.len();
    let start_idx = HEAD_PROBE_CURSOR.fetch_add(1, Ordering::Relaxed) % len;
    for offset in 0..len {
        if probed >= probe_peers {
            break;
        }
        let peer = &peers[(start_idx + offset) % len];
        probed += 1;
        match request_headers_batch(peer, start, probe_limit).await {
            Ok(headers) => {
                if let Some(last) = headers.last() {
                    best = best.max(last.number);
                }
            }
            Err(err) => {
                tracing::debug!(
                    peer_id = ?peer.peer_id,
                    error = %err,
                    "head probe failed"
                );
            }
        }
    }

    Ok(Some(best))
}

#[derive(Debug)]
pub(crate) struct HeaderCountMismatch {
    expected: usize,
    got: usize,
}

impl std::fmt::Display for HeaderCountMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "header count mismatch: expected {}, got {}",
            self.expected, self.got
        )
    }
}

impl std::error::Error for HeaderCountMismatch {}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FetchStageStats {
    pub headers_ms: u64,
    pub bodies_ms: u64,
    pub receipts_ms: u64,
    pub headers_requests: u64,
    pub bodies_requests: u64,
    pub receipts_requests: u64,
}

#[derive(Debug)]
pub(crate) struct HeadersChunkedResponse {
    pub headers: Vec<Header>,
    pub requests: u64,
}

#[cfg(test)]
pub(crate) async fn request_headers_chunked(
    peer: &NetworkPeer,
    start_block: u64,
    count: usize,
) -> Result<Vec<Header>> {
    Ok(
        request_headers_chunked_with_stats(peer, start_block, count)
            .await?
            .headers,
    )
}

pub(crate) async fn request_headers_chunked_with_stats(
    peer: &NetworkPeer,
    start_block: u64,
    count: usize,
) -> Result<HeadersChunkedResponse> {
    if count == 0 {
        return Ok(HeadersChunkedResponse {
            headers: Vec::new(),
            requests: 0,
        });
    }
    let mut headers = Vec::with_capacity(count);
    let mut current = start_block;
    let mut remaining = count;
    let mut requests = 0u64;
    while remaining > 0 {
        let batch = remaining.min(MAX_HEADERS_PER_REQUEST);
        let mut batch_headers = request_headers_batch(peer, current, batch).await?;
        requests = requests.saturating_add(1);
        if batch_headers.is_empty() {
            break;
        }
        let received = batch_headers.len();
        headers.append(&mut batch_headers);
        if received < batch {
            break;
        }
        current = current.saturating_add(batch as u64);
        remaining = remaining.saturating_sub(batch);
    }
    Ok(HeadersChunkedResponse { headers, requests })
}

pub(crate) async fn request_headers_chunked_strict(
    peer: &NetworkPeer,
    start_block: u64,
    count: usize,
) -> Result<Vec<Header>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut headers = Vec::with_capacity(count);
    let mut current = start_block;
    let mut remaining = count;
    while remaining > 0 {
        let batch = remaining.min(MAX_HEADERS_PER_REQUEST);
        let mut batch_headers = request_headers_batch(peer, current, batch).await?;
        if batch_headers.len() != batch {
            return Err(HeaderCountMismatch {
                expected: batch,
                got: batch_headers.len(),
            }
            .into());
        }
        headers.append(&mut batch_headers);
        current = current.saturating_add(batch as u64);
        remaining = remaining.saturating_sub(batch);
    }
    Ok(headers)
}

#[derive(Debug)]
pub(crate) struct PayloadFetchOutcome {
    pub payloads: Vec<BlockPayload>,
    pub missing_blocks: Vec<u64>,
    pub fetch_stats: FetchStageStats,
}

pub(crate) async fn fetch_payloads_for_peer(
    peer: &NetworkPeer,
    range: std::ops::RangeInclusive<u64>,
) -> Result<PayloadFetchOutcome> {
    let start = *range.start();
    let end = *range.end();
    let count = (end - start + 1) as usize;
    let headers_start = Instant::now();
    let headers_response = request_headers_chunked_with_stats(peer, start, count).await?;
    let headers_ms = headers_start.elapsed().as_millis() as u64;
    let headers_requests = headers_response.requests;
    let headers = headers_response.headers;

    let mut headers_by_number = HashMap::new();
    for header in headers {
        headers_by_number.insert(header.number, header);
    }

    let mut ordered_headers = Vec::new();
    let mut missing_blocks = Vec::new();
    for number in start..=end {
        if let Some(header) = headers_by_number.remove(&number) {
            ordered_headers.push(header);
        } else {
            missing_blocks.push(number);
        }
    }

    if ordered_headers.is_empty() {
        return Ok(PayloadFetchOutcome {
            payloads: Vec::new(),
            missing_blocks,
            fetch_stats: FetchStageStats {
                headers_ms,
                headers_requests,
                ..FetchStageStats::default()
            },
        });
    }

    let mut hashes = Vec::with_capacity(ordered_headers.len());
    for header in &ordered_headers {
        let hash = SealedHeader::seal_slow(header.clone()).hash();
        hashes.push(hash);
    }

    let bodies_fut = async {
        let started = Instant::now();
        let resp = request_bodies_chunked_partial_with_stats(peer, &hashes).await?;
        Ok::<_, eyre::Report>((resp, started.elapsed().as_millis() as u64))
    };
    let receipts_fut = async {
        let started = Instant::now();
        let resp = request_receipts_chunked_partial_with_stats(peer, &hashes).await?;
        Ok::<_, eyre::Report>((resp, started.elapsed().as_millis() as u64))
    };
    let ((bodies, bodies_ms), (receipts, receipts_ms)) = tokio::try_join!(bodies_fut, receipts_fut)?;
    let bodies_requests = bodies.requests;
    let receipts_requests = receipts.requests;
    let mut bodies = bodies.results;
    let mut receipts = receipts.results;

    let mut payloads = Vec::with_capacity(ordered_headers.len());
    for (idx, header) in ordered_headers.into_iter().enumerate() {
        let number = header.number;
        let body = bodies.get_mut(idx).and_then(|body| body.take());
        let receipts = receipts.get_mut(idx).and_then(|receipts| receipts.take());

        match (body, receipts) {
            (Some(body), Some(receipts)) => {
                if body.transactions.len() != receipts.len() {
                    missing_blocks.push(number);
                    continue;
                }
                payloads.push(BlockPayload {
                    header,
                    body,
                    receipts,
                });
            }
            _ => {
                missing_blocks.push(number);
            }
        }
    }

    Ok(PayloadFetchOutcome {
        payloads,
        missing_blocks,
        fetch_stats: FetchStageStats {
            headers_ms,
            bodies_ms,
            receipts_ms,
            headers_requests,
            bodies_requests,
            receipts_requests,
        },
    })
}

async fn request_headers_by_number(
    peer_id: PeerId,
    start_block: u64,
    limit: usize,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<Vec<Header>> {
    let request = GetBlockHeaders {
        start_block: BlockHashOrNumber::Number(start_block),
        limit: limit as u64,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| eyre!("failed to send header request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("header request to {peer_id:?} timed out"))??;
    let headers = response.map_err(|err| eyre!("header response error from {peer_id:?}: {err:?}"))?;
    Ok(headers.0)
}

async fn request_headers_by_hash(
    peer_id: PeerId,
    hash: B256,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<Vec<Header>> {
    let request = GetBlockHeaders {
        start_block: BlockHashOrNumber::Hash(hash),
        limit: 1,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| eyre!("failed to send header request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("header request to {peer_id:?} timed out"))??;
    let headers = response.map_err(|err| eyre!("header response error from {peer_id:?}: {err:?}"))?;
    Ok(headers.0)
}

async fn request_bodies(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<reth_ethereum_primitives::BlockBody>> {
    let request = GetBlockBodies::from(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetBlockBodies { request, response: tx })
        .map_err(|err| eyre!("failed to send body request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("body request to {:?} timed out", peer.peer_id))??;
    let bodies = response.map_err(|err| eyre!("body response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(bodies.0)
}

#[derive(Debug)]
pub(crate) struct ChunkedResponse<T> {
    pub results: Vec<Option<T>>,
    pub requests: u64,
}

#[cfg(test)]
async fn request_bodies_chunked_partial(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Option<reth_ethereum_primitives::BlockBody>>> {
    Ok(request_bodies_chunked_partial_with_stats(peer, hashes)
        .await?
        .results)
}

async fn request_bodies_chunked_partial_with_stats(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<ChunkedResponse<reth_ethereum_primitives::BlockBody>> {
    if hashes.is_empty() {
        return Ok(ChunkedResponse {
            results: Vec::new(),
            requests: 0,
        });
    }

    let mut results: Vec<Option<reth_ethereum_primitives::BlockBody>> = vec![None; hashes.len()];
    let mut cursor = 0usize;
    let mut requests = 0u64;
    while cursor < hashes.len() {
        let slice = &hashes[cursor..];
        let requested = slice.len();
        let bodies = request_bodies(peer, slice).await?;
        requests = requests.saturating_add(1);
        if bodies.is_empty() {
            break;
        }
        if bodies.len() > slice.len() {
            return Err(eyre!(
                "body count mismatch: expected <= {}, got {}",
                slice.len(),
                bodies.len()
            ));
        }
        let received = bodies.len();
        for (offset, body) in bodies.into_iter().enumerate() {
            results[cursor + offset] = Some(body);
        }
        cursor = cursor.saturating_add(received);
        if received < requested {
            break;
        }
    }

    Ok(ChunkedResponse { results, requests })
}

pub(crate) async fn request_receipts(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    match peer.eth_version {
        EthVersion::Eth70 => request_receipts70(peer, hashes).await,
        EthVersion::Eth69 => request_receipts69(peer, hashes).await,
        _ => request_receipts_legacy(peer, hashes).await,
    }
}

pub(crate) async fn request_receipt_counts(peer: &NetworkPeer, hashes: &[B256]) -> Result<Vec<usize>> {
    match peer.eth_version {
        EthVersion::Eth70 => request_receipt_counts70(peer, hashes).await,
        EthVersion::Eth69 => request_receipt_counts69(peer, hashes).await,
        _ => request_receipt_counts_legacy(peer, hashes).await,
    }
}

async fn request_receipt_counts_legacy(peer: &NetworkPeer, hashes: &[B256]) -> Result<Vec<usize>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0.iter().map(|block| block.len()).collect())
}

async fn request_receipt_counts69(peer: &NetworkPeer, hashes: &[B256]) -> Result<Vec<usize>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts69 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts69 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts69 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts69 response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0.iter().map(|block| block.len()).collect())
}

async fn request_receipt_counts70(peer: &NetworkPeer, hashes: &[B256]) -> Result<Vec<usize>> {
    let request = GetReceipts70 {
        first_block_receipt_index: 0,
        block_hashes: hashes.to_vec(),
    };
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts70 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts70 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts70 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts70 response error from {:?}: {err:?}", peer.peer_id))?;
    // Note: eth/70 can flag `last_block_incomplete` (partial). Harness treats this as a partial
    // response and requeues the missing block(s), rather than failing the whole batch.
    Ok(receipts.receipts.iter().map(|block| block.len()).collect())
}

#[cfg(test)]
#[allow(dead_code)]
async fn request_receipts_chunked_partial(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Option<Vec<Receipt>>>> {
    Ok(request_receipts_chunked_partial_with_stats(peer, hashes)
        .await?
        .results)
}

async fn request_receipts_chunked_partial_with_stats(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<ChunkedResponse<Vec<Receipt>>> {
    if hashes.is_empty() {
        return Ok(ChunkedResponse {
            results: Vec::new(),
            requests: 0,
        });
    }

    let mut results: Vec<Option<Vec<Receipt>>> = vec![None; hashes.len()];
    let mut cursor = 0usize;
    let mut requests = 0u64;
    while cursor < hashes.len() {
        let slice = &hashes[cursor..];
        let requested = slice.len();
        let receipts = request_receipts(peer, slice).await?;
        requests = requests.saturating_add(1);
        if receipts.is_empty() {
            break;
        }
        if receipts.len() > slice.len() {
            return Err(eyre!(
                "receipt count mismatch: expected <= {}, got {}",
                slice.len(),
                receipts.len()
            ));
        }
        let received = receipts.len();
        for (offset, receipts) in receipts.into_iter().enumerate() {
            results[cursor + offset] = Some(receipts);
        }
        cursor = cursor.saturating_add(received);
        if received < requested {
            break;
        }
    }

    Ok(ChunkedResponse { results, requests })
}

async fn request_receipts_legacy(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0
        .into_iter()
        .map(|block| block.into_iter().map(|r| r.receipt).collect())
        .collect())
}

async fn request_receipts69(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts69 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts69 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts69 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts69 response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0)
}

async fn request_receipts70(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts70 {
        first_block_receipt_index: 0,
        block_hashes: hashes.to_vec(),
    };
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts70 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts70 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts70 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts70 response error from {:?}: {err:?}", peer.peer_id))?;
    // eth/70 can flag `last_block_incomplete` (partial). Treat this as a partial response:
    // downstream will requeue any missing blocks.
    Ok(receipts.receipts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_eth_wire_types::{BlockBodies, BlockHeaders};
    use reth_network_api::PeerRequestSender;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[test]
    fn round_robin_cycles_peers() {
        let mut selector = RoundRobinPeerSelector::new(vec![
            "peer-a".to_string(),
            "peer-b".to_string(),
            "peer-c".to_string(),
        ]);
        let selected = vec![
            selector.next_peer(),
            selector.next_peer(),
            selector.next_peer(),
            selector.next_peer(),
        ];
        assert_eq!(
            selected,
            vec![
                Some("peer-a".to_string()),
                Some("peer-b".to_string()),
                Some("peer-c".to_string()),
                Some("peer-a".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn request_bodies_chunked_splits_partial_responses() {
        let peer_id = PeerId::random();
        let (tx, mut rx) = mpsc::channel(8);
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 0,
        };

        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_task = Arc::clone(&request_count);
        tokio::spawn(async move {
            let mut first = true;
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockBodies { request, response } => {
                        request_count_task.fetch_add(1, Ordering::SeqCst);
                        let count = request.0.len();
                        let body_count = if first {
                            first = false;
                            count.saturating_sub(1)
                        } else {
                            count
                        };
                        let bodies =
                            vec![reth_ethereum_primitives::BlockBody::default(); body_count];
                        let _ = response.send(Ok(BlockBodies::from(bodies)));
                    }
                    _ => {}
                }
            }
        });

        let hashes = vec![
            B256::from([0x01u8; 32]),
            B256::from([0x02u8; 32]),
            B256::from([0x03u8; 32]),
            B256::from([0x04u8; 32]),
        ];
        let bodies = request_bodies_chunked_partial(&peer, &hashes)
            .await
            .expect("bodies");
        assert_eq!(bodies.len(), hashes.len());
        assert!(bodies.iter().take(3).all(|body| body.is_some()));
        assert!(bodies[3].is_none());
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn request_headers_chunked_splits_large_requests() {
        let peer_id = PeerId::random();
        let (tx, mut rx) = mpsc::channel(8);
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 0,
        };

        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_task = Arc::clone(&request_count);
        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockHeaders { request, response } => {
                        request_count_task.fetch_add(1, Ordering::SeqCst);
                        let count = request.limit as usize;
                        let headers = vec![Header::default(); count];
                        let _ = response.send(Ok(BlockHeaders::from(headers)));
                    }
                    _ => {}
                }
            }
        });

        let count = MAX_HEADERS_PER_REQUEST + 1;
        let headers = request_headers_chunked(&peer, 0, count)
            .await
            .expect("headers");
        assert_eq!(headers.len(), count);
        assert_eq!(request_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn discover_head_p2p_uses_header_probes() {
        let peer_id = PeerId::random();
        let (tx, mut rx) = mpsc::channel(8);
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 90,
        };

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockHeaders { request, response } => {
                        let start = match request.start_block {
                            BlockHashOrNumber::Number(start) => start,
                            _ => 0,
                        };
                        let mut headers = Vec::new();
                        for idx in 0..2u64 {
                            let mut header = Header::default();
                            header.number = start + idx;
                            headers.push(header);
                        }
                        let _ = response.send(Ok(BlockHeaders::from(headers)));
                    }
                    _ => {}
                }
            }
        });

        let pool = peer_pool_for_tests(vec![peer]);
        let head = discover_head_p2p(&pool, 100, 1, 3)
            .await
            .expect("discover head")
            .expect("head");
        assert_eq!(head, 102);
    }
}
