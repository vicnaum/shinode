use alloy_primitives::B256;
use futures::StreamExt;
use reth_chainspec::MAINNET;
use reth_eth_wire::EthNetworkPrimitives;
use reth_eth_wire_types::{
    BlockHashOrNumber, EthVersion, GetBlockHeaders, GetReceipts, GetReceipts70, HeadersDirection,
    Receipts69, Receipts70,
};
use reth_network::config::{rng_secret_key, NetworkConfigBuilder};
use reth_network::import::ProofOfStakeBlockImport;
use reth_network_api::{
    DiscoveredEvent, DiscoveryEvent, NetworkEvent, NetworkEventListenerProvider, PeerId,
    PeerRequest, PeerRequestSender,
};
use reth_primitives_traits::{Header, SealedHeader};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let run_secs = parse_run_secs();
    let run_start = Instant::now();

    let out_dir = PathBuf::from("output");
    std::fs::create_dir_all(&out_dir)?;
    let known_path = out_dir.join("known_blocks.jsonl");
    let known_blocks = load_known_blocks(&known_path);
    let targets = build_anchor_targets();
    let pending_targets: VecDeque<u64> = targets
        .into_iter()
        .filter(|block| !known_blocks.contains(block))
        .collect();

    let context = Arc::new(RunContext {
        stats: Mutex::new(Stats::default()),
        queue: Mutex::new(pending_targets),
        known_blocks: Mutex::new(known_blocks),
        peer_health: Mutex::new(HashMap::new()),
        logger: Logger::new(&out_dir)?,
        request_id: AtomicU64::new(1),
    });
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel::<PeerHandle>();

    let scheduler_context = Arc::clone(&context);
    let scheduler_ready_tx = ready_tx.clone();
    tokio::spawn(async move {
        let mut warmed_up = false;
        while let Some(peer) = ready_rx.recv().await {
            if !warmed_up {
                if WARMUP_SECS > 0 {
                    tokio::time::sleep(Duration::from_secs(WARMUP_SECS)).await;
                }
                warmed_up = true;
            }

            if let Some(remaining) = scheduler_context.peer_ban_remaining(&peer.peer_key).await {
                let ready_tx = scheduler_ready_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(remaining).await;
                    let _ = ready_tx.send(peer);
                });
                continue;
            }

            let Some(block_number) = scheduler_context
                .pop_next_block_for_head(peer.head_number)
                .await
            else {
                continue;
            };

            let context = Arc::clone(&scheduler_context);
            let ready_tx = scheduler_ready_tx.clone();
            tokio::spawn(async move {
                let peer_for_probe = peer.clone();
                probe_block(
                    peer_for_probe.peer_id,
                    peer_for_probe.peer_key,
                    peer_for_probe.messages,
                    peer_for_probe.eth_version,
                    block_number,
                    Arc::clone(&context),
                )
                .await;
                let _ = ready_tx.send(peer);
            });
        }
    });

    let secret_key = rng_secret_key();
    let config = NetworkConfigBuilder::<EthNetworkPrimitives>::new(secret_key)
        .mainnet_boot_nodes()
        .with_unused_ports()
        .disable_tx_gossip(true)
        .block_import(Box::new(ProofOfStakeBlockImport::default()))
        .build_with_noop_provider(MAINNET.clone());

    let handle = config.start_network().await?;

    info!(peer_id = ?handle.peer_id(), "network started");
    println!("receipt-harness: network started");

    let mut discovery_events = handle.discovery_listener();
    tokio::spawn(async move {
        let mut discovered = 0usize;
        while let Some(event) = discovery_events.next().await {
            match event {
                DiscoveryEvent::NewNode(DiscoveredEvent::EventQueued {
                    peer_id,
                    addr,
                    fork_id,
                }) => {
                    discovered += 1;
                    info!(
                        peer_id = ?peer_id,
                        addr = ?addr,
                        fork_id = ?fork_id,
                        discovered,
                        "peer discovered"
                    );
                }
                DiscoveryEvent::EnrForkId(peer_id, fork_id) => {
                    info!(
                        peer_id = ?peer_id,
                        fork_id = ?fork_id,
                        "peer forkid from ENR"
                    );
                }
            }
        }
    });

    let mut event_listener = handle.event_listener();
    let context_for_events = Arc::clone(&context);
    let ready_tx_for_events = ready_tx.clone();
    tokio::spawn(async move {
        while let Some(event) = event_listener.next().await {
            if let NetworkEvent::ActivePeerSession { info, messages } = event {
                let matches_genesis = info.status.genesis == MAINNET.genesis_hash();
                if !matches_genesis {
                    warn!(
                        peer_id = ?info.peer_id,
                        genesis = ?info.status.genesis,
                        "peer genesis mismatch"
                    );
                    continue;
                }

                info!(
                    peer_id = ?info.peer_id,
                    client_version = %info.client_version,
                    remote_addr = ?info.remote_addr,
                    eth_version = ?info.version,
                    chain = ?info.status.chain,
                    head_hash = ?info.status.blockhash,
                    "peer session established"
                );

                context_for_events
                    .stats
                    .lock()
                    .await
                    .record_peer(info.client_version.as_ref());

                let peer_id = info.peer_id;
                let peer_key = format!("{:?}", peer_id);
                let head_hash = info.status.blockhash;
                let messages = messages.clone();
                let context = Arc::clone(&context_for_events);
                let ready_tx = ready_tx_for_events.clone();

                tokio::spawn(async move {
                    let head_start = Instant::now();
                    let head_number = match request_head_number(
                        peer_id,
                        info.version,
                        BlockHashOrNumber::Hash(head_hash),
                        &messages,
                        context.as_ref(),
                    )
                    .await
                    {
                        Ok(number) => {
                            info!(
                                peer_id = ?peer_id,
                                head_number = number,
                                head_ms = head_start.elapsed().as_millis(),
                                "peer head resolved"
                            );
                            number
                        }
                        Err(err) => {
                            warn!(peer_id = ?peer_id, error = %err, "head header request failed");
                            return;
                        }
                    };

                    let _ = ready_tx.send(PeerHandle {
                        peer_id,
                        peer_key,
                        eth_version: info.version,
                        head_number,
                        messages,
                    });
                });
            }
        }
    });

    if let Some(seconds) = run_secs {
        info!(run_secs = seconds, "auto-shutdown enabled");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = tokio::time::sleep(Duration::from_secs(seconds)) => {
                info!("auto-shutdown reached");
            }
        }
    } else {
        tokio::signal::ctrl_c().await?;
    }

    let summary = summarize_stats(&context.stats, run_start.elapsed()).await;
    println!("{}", summary);
    Ok(())
}

fn parse_run_secs() -> Option<u64> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--run-secs=") {
            return value.parse().ok();
        }
        if arg == "--run-secs" {
            return args.next().and_then(|value| value.parse().ok());
        }
    }
    None
}

async fn request_head_number(
    peer_id: PeerId,
    eth_version: EthVersion,
    start_block: BlockHashOrNumber,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    context: &RunContext,
) -> Result<u64, String> {
    let request_id = context.next_request_id();
    let sent_at = now_ms();
    context
        .log_request(json!({
            "event": "request_sent",
            "request_id": request_id,
            "peer_id": format!("{:?}", peer_id),
            "block": format!("{start_block:?}"),
            "kind": "head_header",
            "eth_version": format!("{eth_version}"),
            "sent_at_ms": sent_at,
        }))
        .await;

    let request = GetBlockHeaders {
        start_block,
        limit: 1,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| format!("send error: {err:?}"))?;

    let response = rx
        .await
        .map_err(|err| format!("response dropped: {err:?}"))?;

    let received_at = now_ms();
    let duration_ms = received_at.saturating_sub(sent_at);

    match response {
        Ok(headers) => {
            let head_number = headers
                .0
                .first()
                .map(|header| header.number)
                .ok_or_else(|| "empty head header response".to_string())?;
            context
                .log_request(json!({
                    "event": "response_ok",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "kind": "head_header",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "items": headers.0.len(),
                    "head_number": head_number,
                }))
                .await;
            Ok(head_number)
        }
        Err(err) => {
            context
                .log_request(json!({
                    "event": "response_err",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "kind": "head_header",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "error": format!("{err:?}"),
                }))
                .await;
            Err(format!("{err:?}"))
        }
    }
}

fn build_anchor_targets() -> Vec<u64> {
    const ANCHOR_WINDOW: u64 = 1_000;
    const ANCHORS: [u64; 5] = [10_000_835, 11_362_579, 12_369_621, 16_291_127, 21_688_329];

    let mut targets = Vec::new();
    for anchor in ANCHORS {
        for offset in 0..ANCHOR_WINDOW {
            targets.push(anchor + offset);
        }
    }

    targets.sort_unstable();
    targets.dedup();
    targets
}

async fn probe_block(
    peer_id: PeerId,
    peer_key: String,
    messages: PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    eth_version: EthVersion,
    block_number: u64,
    context: Arc<RunContext>,
) {
    let header_start = Instant::now();
    let header =
        match request_header(peer_id, eth_version, &messages, block_number, context.as_ref()).await
    {
        Ok(header) => header,
        Err(err) => {
            let header_ms = header_start.elapsed().as_millis();
            context
                .stats
                .lock()
                .await
                .record_probe(block_number, false, false, header_ms, None);
            if let Some(ban) = context.record_peer_failure(&peer_key).await {
                context
                    .log_request(json!({
                        "event": "peer_ban",
                        "peer_id": peer_key,
                        "reason": "header",
                        "ban_secs": ban.ban_secs,
                        "failures": ban.failures,
                        "at_ms": now_ms(),
                    }))
                    .await;
                warn!(
                    peer_id = ?peer_id,
                    ban_secs = ban.ban_secs,
                    failures = ban.failures,
                    "peer temporarily banned"
                );
            }
            warn!(
                peer_id = ?peer_id,
                block = block_number,
                header_ms,
                error = %err,
                "probe header failed"
            );
            let payload = json!({
                "event": "probe",
                "peer_id": format!("{:?}", peer_id),
                "block": block_number,
                "eth_version": format!("{eth_version}"),
                "header_ok": false,
                "receipts_ok": false,
                "header_ms": header_ms,
                "error": err,
            });
            println!("{}", payload);
            context.log_probe(payload).await;
            return;
        }
    };
    let header_ms = header_start.elapsed().as_millis();

    if header.number != block_number {
        warn!(
            peer_id = ?peer_id,
            requested = block_number,
            received = header.number,
            "probe header mismatch"
        );
    }

    let block_hash: B256 = SealedHeader::seal_slow(header).hash();
    let receipts_start = Instant::now();
    let receipts_result = match eth_version {
        EthVersion::Eth69 => {
            request_receipts69(
                peer_id,
                eth_version,
                &messages,
                block_hash,
                context.as_ref(),
                block_number,
            )
                .await
        }
        EthVersion::Eth70 => {
            request_receipts70(
                peer_id,
                eth_version,
                &messages,
                block_hash,
                context.as_ref(),
                block_number,
            )
                .await
        }
        _ => {
            request_receipts_legacy(
                peer_id,
                eth_version,
                &messages,
                block_hash,
                context.as_ref(),
                block_number,
            )
                .await
        }
    };
    let receipts_ms = receipts_start.elapsed().as_millis();

    match receipts_result {
        Ok(receipt_count) => {
            context
                .stats
                .lock()
                .await
                .record_probe(block_number, true, true, header_ms, Some(receipts_ms));
            context.record_peer_success(&peer_key).await;
            context.mark_known_block(block_number).await;
            info!(
                peer_id = ?peer_id,
                block = block_number,
                header_ms,
                receipts_ms,
                receipts = receipt_count,
                "probe result"
            );
            let payload = json!({
                "event": "probe",
                "peer_id": format!("{:?}", peer_id),
                "block": block_number,
                "eth_version": format!("{eth_version}"),
                "header_ok": true,
                "receipts_ok": true,
                "header_ms": header_ms,
                "receipts_ms": receipts_ms,
                "receipts": receipt_count,
            });
            println!("{}", payload);
            context.log_probe(payload).await;
        }
        Err(err) => {
            context
                .stats
                .lock()
                .await
                .record_probe(block_number, true, false, header_ms, Some(receipts_ms));
            if let Some(ban) = context.record_peer_failure(&peer_key).await {
                context
                    .log_request(json!({
                        "event": "peer_ban",
                        "peer_id": peer_key,
                        "reason": "receipts",
                        "ban_secs": ban.ban_secs,
                        "failures": ban.failures,
                        "at_ms": now_ms(),
                    }))
                    .await;
                warn!(
                    peer_id = ?peer_id,
                    ban_secs = ban.ban_secs,
                    failures = ban.failures,
                    "peer temporarily banned"
                );
            }
            warn!(
                peer_id = ?peer_id,
                block = block_number,
                header_ms,
                error = %err,
                "receipts request failed"
            );
            let payload = json!({
                "event": "probe",
                "peer_id": format!("{:?}", peer_id),
                "block": block_number,
                "eth_version": format!("{eth_version}"),
                "header_ok": true,
                "receipts_ok": false,
                "header_ms": header_ms,
                "receipts_ms": receipts_ms,
                "error": err,
            });
            println!("{}", payload);
            context.log_probe(payload).await;
        }
    }
}

async fn request_header(
    peer_id: PeerId,
    eth_version: EthVersion,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    block_number: u64,
    context: &RunContext,
) -> Result<Header, String> {
    let request_id = context.next_request_id();
    let sent_at = now_ms();
    context
        .log_request(json!({
            "event": "request_sent",
            "request_id": request_id,
            "peer_id": format!("{:?}", peer_id),
            "block": block_number,
            "kind": "header",
            "eth_version": format!("{eth_version}"),
            "sent_at_ms": sent_at,
        }))
        .await;

    let request = GetBlockHeaders {
        start_block: BlockHashOrNumber::Number(block_number),
        limit: 1,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| format!("send error: {err:?}"))?;

    let response = rx
        .await
        .map_err(|err| format!("response dropped: {err:?}"))?;

    let received_at = now_ms();
    let duration_ms = received_at.saturating_sub(sent_at);

    match response {
        Ok(headers) => {
            let header = headers
                .0
                .first()
                .cloned()
                .ok_or_else(|| "empty header response".to_string())?;
            context
                .log_request(json!({
                    "event": "response_ok",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "header",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "items": headers.0.len(),
                }))
                .await;
            Ok(header)
        }
        Err(err) => {
            context
                .log_request(json!({
                    "event": "response_err",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "header",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "error": format!("{err:?}"),
                }))
                .await;
            Err(format!("{err:?}"))
        }
    }
}

struct Logger {
    requests: Mutex<BufWriter<File>>,
    probes: Mutex<BufWriter<File>>,
    known: Mutex<BufWriter<File>>,
}

impl Logger {
    fn new(out_dir: &PathBuf) -> std::io::Result<Self> {
        let requests = open_log(out_dir.join("requests.jsonl"))?;
        let probes = open_log(out_dir.join("probes.jsonl"))?;
        let known = open_log(out_dir.join("known_blocks.jsonl"))?;
        Ok(Self { requests, probes, known })
    }

    async fn log_request(&self, value: serde_json::Value) {
        write_json_line(&self.requests, value).await;
    }

    async fn log_probe(&self, value: serde_json::Value) {
        write_json_line(&self.probes, value).await;
    }

    async fn log_known(&self, block: u64) {
        write_json_line(&self.known, json!({
            "block": block,
            "at_ms": now_ms(),
        }))
        .await;
    }
}

struct RunContext {
    stats: Mutex<Stats>,
    queue: Mutex<VecDeque<u64>>,
    known_blocks: Mutex<HashSet<u64>>,
    peer_health: Mutex<HashMap<String, PeerHealth>>,
    logger: Logger,
    request_id: AtomicU64,
}

impl RunContext {
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    async fn pop_next_block_for_head(&self, head_number: u64) -> Option<u64> {
        let mut queue = self.queue.lock().await;
        let next = queue.front().copied();
        match next {
            Some(block) if block <= head_number => queue.pop_front(),
            _ => None,
        }
    }

    async fn log_request(&self, value: serde_json::Value) {
        self.logger.log_request(value).await;
    }

    async fn log_probe(&self, value: serde_json::Value) {
        self.logger.log_probe(value).await;
    }

    async fn mark_known_block(&self, block: u64) {
        let mut known = self.known_blocks.lock().await;
        if known.insert(block) {
            self.logger.log_known(block).await;
        }
    }

    async fn peer_ban_remaining(&self, peer_key: &str) -> Option<Duration> {
        let mut health = self.peer_health.lock().await;
        let entry = health.get_mut(peer_key)?;
        if let Some(until) = entry.ban_until {
            let remaining = until.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                entry.ban_until = None;
                None
            } else {
                Some(remaining)
            }
        } else {
            None
        }
    }

    async fn record_peer_failure(&self, peer_key: &str) -> Option<PeerBan> {
        let mut health = self.peer_health.lock().await;
        let entry = health.entry(peer_key.to_string()).or_default();
        entry.consecutive_failures += 1;
        if entry.consecutive_failures >= PEER_FAILURE_THRESHOLD {
            let failures = entry.consecutive_failures;
            entry.consecutive_failures = 0;
            entry.ban_until = Some(Instant::now() + Duration::from_secs(PEER_BAN_SECS));
            return Some(PeerBan { ban_secs: PEER_BAN_SECS, failures });
        }
        None
    }

    async fn record_peer_success(&self, peer_key: &str) {
        let mut health = self.peer_health.lock().await;
        if let Some(entry) = health.get_mut(peer_key) {
            entry.consecutive_failures = 0;
            entry.ban_until = None;
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn open_log(path: PathBuf) -> std::io::Result<Mutex<BufWriter<File>>> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(Mutex::new(BufWriter::new(file)))
}

async fn write_json_line(writer: &Mutex<BufWriter<File>>, value: serde_json::Value) {
    let mut guard = writer.lock().await;
    if let Err(err) = writeln!(guard, "{value}") {
        warn!(error = ?err, "failed to write log line");
    }
    if let Err(err) = guard.flush() {
        warn!(error = ?err, "failed to flush log");
    }
}

fn load_known_blocks(path: &PathBuf) -> HashSet<u64> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return HashSet::new(),
    };
    let reader = BufReader::new(file);
    let mut blocks = HashSet::new();
    for line in reader.lines().flatten() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(block) = value.get("block").and_then(|v| v.as_u64()) {
                blocks.insert(block);
            }
        }
    }
    blocks
}

#[derive(Clone)]
struct PeerHandle {
    peer_id: PeerId,
    peer_key: String,
    eth_version: EthVersion,
    head_number: u64,
    messages: PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
}

const WARMUP_SECS: u64 = 3;
const PEER_FAILURE_THRESHOLD: u32 = 5;
const PEER_BAN_SECS: u64 = 120;

#[derive(Default)]
struct PeerHealth {
    consecutive_failures: u32,
    ban_until: Option<Instant>,
}

struct PeerBan {
    ban_secs: u64,
    failures: u32,
}

#[derive(Default)]
struct BlockStats {
    total: u64,
    header_ok: u64,
    receipts_ok: u64,
    header_ms_sum: u128,
    receipts_ms_sum: u128,
}

#[derive(Default)]
struct Stats {
    by_block: HashMap<u64, BlockStats>,
    clients: HashMap<String, u64>,
    total_probes: u64,
    receipts_ok_total: u64,
}

impl Stats {
    fn record_peer(&mut self, client_version: &str) {
        *self.clients.entry(client_version.to_string()).or_insert(0) += 1;
    }

    fn record_probe(
        &mut self,
        block: u64,
        header_ok: bool,
        receipts_ok: bool,
        header_ms: u128,
        receipts_ms: Option<u128>,
    ) {
        let stats = self.by_block.entry(block).or_default();
        stats.total += 1;
        self.total_probes += 1;
        if header_ok {
            stats.header_ok += 1;
            stats.header_ms_sum += header_ms;
        }
        if receipts_ok {
            stats.receipts_ok += 1;
            self.receipts_ok_total += 1;
            if let Some(ms) = receipts_ms {
                stats.receipts_ms_sum += ms;
            }
        }
    }
}

async fn summarize_stats(stats: &Mutex<Stats>, elapsed: Duration) -> serde_json::Value {
    let stats = stats.lock().await;
    let elapsed_secs = elapsed.as_secs_f64().max(0.001);
    let throughput = stats.receipts_ok_total as f64 / elapsed_secs;
    let mut blocks: Vec<_> = stats.by_block.iter().map(|(block, stats)| {
        let header_rate = if stats.total > 0 {
            stats.header_ok as f64 / stats.total as f64
        } else {
            0.0
        };
        let receipts_rate = if stats.total > 0 {
            stats.receipts_ok as f64 / stats.total as f64
        } else {
            0.0
        };
        let avg_header_ms = if stats.header_ok > 0 {
            stats.header_ms_sum as f64 / stats.header_ok as f64
        } else {
            0.0
        };
        let avg_receipts_ms = if stats.receipts_ok > 0 {
            stats.receipts_ms_sum as f64 / stats.receipts_ok as f64
        } else {
            0.0
        };
        json!({
            "block": block,
            "total": stats.total,
            "header_ok": stats.header_ok,
            "receipts_ok": stats.receipts_ok,
            "header_rate": header_rate,
            "receipts_rate": receipts_rate,
            "avg_header_ms": avg_header_ms,
            "avg_receipts_ms": avg_receipts_ms,
        })
    }).collect();

    blocks.sort_by_key(|entry| entry["block"].as_u64().unwrap_or_default());

    json!({
        "event": "summary",
        "duration_secs": elapsed_secs,
        "total_probes": stats.total_probes,
        "receipts_ok_total": stats.receipts_ok_total,
        "throughput_receipts_per_sec": throughput,
        "peers": stats.clients,
        "by_block": blocks,
    })
}

async fn request_receipts_legacy(
    peer_id: PeerId,
    eth_version: EthVersion,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    block_hash: B256,
    context: &RunContext,
    block_number: u64,
) -> Result<usize, String> {
    let request_id = context.next_request_id();
    let sent_at = now_ms();
    context
        .log_request(json!({
            "event": "request_sent",
            "request_id": request_id,
            "peer_id": format!("{:?}", peer_id),
            "block": block_number,
            "kind": "receipts",
            "variant": "legacy",
            "eth_version": format!("{eth_version}"),
            "sent_at_ms": sent_at,
        }))
        .await;

    let request = GetReceipts(vec![block_hash]);
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetReceipts { request, response: tx })
        .map_err(|err| format!("send error: {err:?}"))?;

    let response = rx
        .await
        .map_err(|err| format!("response dropped: {err:?}"))?;

    let received_at = now_ms();
    let duration_ms = received_at.saturating_sub(sent_at);

    match response {
        Ok(receipts) => {
            let count = receipts.0.first().map(|r| r.len()).unwrap_or(0);
            context
                .log_request(json!({
                    "event": "response_ok",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "legacy",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "receipts": count,
                }))
                .await;
            Ok(count)
        }
        Err(err) => {
            context
                .log_request(json!({
                    "event": "response_err",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "legacy",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "error": format!("{err:?}"),
                }))
                .await;
            Err(format!("{err:?}"))
        }
    }
}

async fn request_receipts69(
    peer_id: PeerId,
    eth_version: EthVersion,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    block_hash: B256,
    context: &RunContext,
    block_number: u64,
) -> Result<usize, String> {
    let request_id = context.next_request_id();
    let sent_at = now_ms();
    context
        .log_request(json!({
            "event": "request_sent",
            "request_id": request_id,
            "peer_id": format!("{:?}", peer_id),
            "block": block_number,
            "kind": "receipts",
            "variant": "eth69",
            "eth_version": format!("{eth_version}"),
            "sent_at_ms": sent_at,
        }))
        .await;

    let request = GetReceipts(vec![block_hash]);
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetReceipts69 { request, response: tx })
        .map_err(|err| format!("send error: {err:?}"))?;

    let response = rx
        .await
        .map_err(|err| format!("response dropped: {err:?}"))?;

    let received_at = now_ms();
    let duration_ms = received_at.saturating_sub(sent_at);

    match response {
        Ok(Receipts69(receipts)) => {
            let count = receipts.first().map(|r| r.len()).unwrap_or(0);
            context
                .log_request(json!({
                    "event": "response_ok",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "eth69",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "receipts": count,
                }))
                .await;
            Ok(count)
        }
        Err(err) => {
            context
                .log_request(json!({
                    "event": "response_err",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "eth69",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "error": format!("{err:?}"),
                }))
                .await;
            Err(format!("{err:?}"))
        }
    }
}

async fn request_receipts70(
    peer_id: PeerId,
    eth_version: EthVersion,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    block_hash: B256,
    context: &RunContext,
    block_number: u64,
) -> Result<usize, String> {
    let request_id = context.next_request_id();
    let sent_at = now_ms();
    context
        .log_request(json!({
            "event": "request_sent",
            "request_id": request_id,
            "peer_id": format!("{:?}", peer_id),
            "block": block_number,
            "kind": "receipts",
            "variant": "eth70",
            "eth_version": format!("{eth_version}"),
            "sent_at_ms": sent_at,
        }))
        .await;

    let request = GetReceipts70 {
        first_block_receipt_index: 0,
        block_hashes: vec![block_hash],
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetReceipts70 { request, response: tx })
        .map_err(|err| format!("send error: {err:?}"))?;

    let response = rx
        .await
        .map_err(|err| format!("response dropped: {err:?}"))?;

    let received_at = now_ms();
    let duration_ms = received_at.saturating_sub(sent_at);

    match response {
        Ok(Receipts70 { receipts, .. }) => {
            let count = receipts.first().map(|r| r.len()).unwrap_or(0);
            context
                .log_request(json!({
                    "event": "response_ok",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "eth70",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "receipts": count,
                }))
                .await;
            Ok(count)
        }
        Err(err) => {
            context
                .log_request(json!({
                    "event": "response_err",
                    "request_id": request_id,
                    "peer_id": format!("{:?}", peer_id),
                    "block": block_number,
                    "kind": "receipts",
                    "variant": "eth70",
                    "eth_version": format!("{eth_version}"),
                    "received_at_ms": received_at,
                    "duration_ms": duration_ms,
                    "error": format!("{err:?}"),
                }))
                .await;
            Err(format!("{err:?}"))
        }
    }
}

