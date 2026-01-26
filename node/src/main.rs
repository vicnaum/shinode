mod cli;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use cli::{compute_target_range, NodeConfig, DEFAULT_LOG_JSON_FILTER, DEFAULT_LOG_TRACE_FILTER};
use eyre::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use metrics::range_len;
use p2p::{p2p_limits, PeerPool};
use reth_network_api::PeerId;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::{
    collections::VecDeque,
    env, fs,
    io::{BufWriter, IsTerminal, Write},
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use sync::historical::{BenchEvent, BenchEventLogger, IngestBenchStats, PeerHealthTracker};
use sync::{ProgressReporter, SyncProgressStats, SyncStatus};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, info, warn, Event};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

const LOG_BUFFER: usize = 10_000;

async fn apply_cached_peer_limits(storage: &storage::Storage, peer_health: &PeerHealthTracker) {
    let peers = storage.peer_cache_snapshot();
    for peer in peers {
        let Some(limit) = peer.aimd_batch_limit else {
            continue;
        };
        let Ok(peer_id) = PeerId::from_str(&peer.peer_id) else {
            continue;
        };
        peer_health.set_batch_limit(peer_id, limit).await;
    }
}

async fn persist_peer_limits(storage: &storage::Storage, peer_health: &PeerHealthTracker) {
    let snapshot = peer_health.snapshot().await;
    let limits: Vec<(String, u64)> = snapshot
        .into_iter()
        .map(|entry| (entry.peer_id.to_string(), entry.batch_limit as u64))
        .collect();
    storage.update_peer_batch_limits(&limits);
}

async fn flush_peer_cache_with_limits(
    session: &p2p::NetworkSession,
    storage: &storage::Storage,
    peer_health: Option<&PeerHealthTracker>,
) {
    if let Err(err) = session.flush_peer_cache() {
        warn!(error = %err, "failed to flush peer cache");
        return;
    }
    if let Some(peer_health) = peer_health {
        persist_peer_limits(storage, peer_health).await;
        if let Err(err) = storage.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache with limits");
        }
    }
}

impl sync::ProgressReporter for ProgressBar {
    fn set_length(&self, len: u64) {
        self.set_length(len);
    }

    fn inc(&self, delta: u64) {
        self.inc(delta);
    }
}

struct IngestProgress {
    bar: ProgressBar,
    total_len: AtomicU64,
}

impl IngestProgress {
    fn new(bar: ProgressBar, total_len: u64) -> Self {
        bar.set_length(total_len);
        Self {
            bar,
            total_len: AtomicU64::new(total_len),
        }
    }

    fn finish(&self) {
        self.bar.finish_and_clear();
    }
}

impl sync::ProgressReporter for IngestProgress {
    fn set_length(&self, len: u64) {
        let current = self.total_len.load(Ordering::SeqCst);
        if len > current {
            self.total_len.store(len, Ordering::SeqCst);
            self.bar.set_length(len);
        }
    }

    fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }
}

#[derive(Debug, Serialize)]
struct RunReport {
    meta: RunMeta,
    argv: Vec<String>,
    config: NodeConfig,
    derived: RunDerived,
    results: sync::historical::IngestBenchSummary,
    peer_health_all: Vec<PeerHealthSummary>,
    peer_health_top: Vec<PeerHealthSummary>,
    peer_health_worst: Vec<PeerHealthSummary>,
    events: Option<EventsSummary>,
    logs: Option<LogsSummary>,
}

#[derive(Debug, Serialize)]
struct RunMeta {
    timestamp_utc: String,
    run_id: String,
    run_name: String,
    build: BuildInfo,
    env: EnvInfo,
}

#[derive(Debug, Serialize)]
struct BuildInfo {
    profile: String,
    debug_assertions: bool,
    features: Vec<String>,
    version: String,
}

#[derive(Debug, Serialize)]
struct EnvInfo {
    os: String,
    arch: String,
    cpu_count: usize,
    pid: u32,
}

#[derive(Debug, Serialize)]
struct RunDerived {
    range_start: u64,
    range_end: u64,
    head_at_startup: u64,
    safe_head: u64,
    rollback_window_applied: bool,
    worker_count: usize,
    p2p_limits: P2pLimitsSummary,
}

#[derive(Debug, Serialize)]
struct P2pLimitsSummary {
    max_outbound: usize,
    max_concurrent_dials: usize,
    peer_refill_interval_ms: u64,
    request_timeout_ms: u64,
    max_headers_per_request: usize,
    peer_cache_ttl_days: u64,
    peer_cache_max: usize,
}

#[derive(Debug, Serialize)]
struct PeerHealthSummary {
    peer_id: String,
    is_banned: bool,
    ban_remaining_ms: Option<u64>,
    consecutive_failures: u32,
    consecutive_partials: u32,
    successes: u64,
    failures: u64,
    partials: u64,
    assignments: u64,
    assigned_blocks: u64,
    inflight_blocks: u64,
    batch_limit: u64,
    batch_limit_max: u64,
    batch_limit_avg: Option<f64>,
    last_assigned_age_ms: Option<u64>,
    last_success_age_ms: Option<u64>,
    last_failure_age_ms: Option<u64>,
    last_partial_age_ms: Option<u64>,
    quality_score: f64,
    quality_samples: u64,
    last_error: Option<String>,
    last_error_age_ms: Option<u64>,
    last_error_count: u64,
}

#[derive(Debug, Serialize)]
struct EventsSummary {
    total_events: usize,
    dropped_events: u64,
    path: String,
}

#[derive(Debug, Serialize)]
struct LogsSummary {
    total_events: usize,
    dropped_events: u64,
    path: String,
}

#[derive(Debug)]
struct RunContext {
    timestamp_utc: String,
    run_id: String,
    run_name: String,
    output_dir: PathBuf,
    argv: Vec<String>,
    trace_tmp_path: Option<PathBuf>,
    events_tmp_path: Option<PathBuf>,
    logs_tmp_path: Option<PathBuf>,
    resources_tmp_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct LogRecord {
    t_ms: u64,
    level: String,
    target: String,
    file: Option<String>,
    line: Option<u32>,
    message: Option<String>,
    fields: JsonMap<String, JsonValue>,
}

#[derive(Default)]
struct JsonLogVisitor {
    fields: JsonMap<String, JsonValue>,
}

impl tracing::field::Visit for JsonLogVisitor {
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Number(value.into()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Number(value.into()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let number = serde_json::Number::from_f64(value)
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(value.to_string()));
        self.fields.insert(field.name().to_string(), number);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(
            field.name().to_string(),
            JsonValue::String(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            JsonValue::String(format!("{value:?}")),
        );
    }
}

#[derive(Debug)]
struct JsonLogWriter {
    started_at: Instant,
    sender: Mutex<Option<SyncSender<LogRecord>>>,
    handle: Mutex<Option<JoinHandle<eyre::Result<()>>>>,
    dropped_events: AtomicU64,
    total_events: AtomicU64,
}

impl JsonLogWriter {
    fn new(path: PathBuf, capacity: usize) -> eyre::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let (tx, rx) = mpsc::sync_channel::<LogRecord>(capacity);
        let handle = std::thread::spawn(move || -> eyre::Result<()> {
            let mut since_flush = 0usize;
            for record in rx {
                serde_json::to_writer(&mut writer, &record)?;
                writer.write_all(b"\n")?;
                since_flush = since_flush.saturating_add(1);
                if since_flush >= 4096 {
                    writer.flush()?;
                    since_flush = 0;
                }
            }
            writer.flush()?;
            Ok(())
        });

        Ok(Self {
            started_at: Instant::now(),
            sender: Mutex::new(Some(tx)),
            handle: Mutex::new(Some(handle)),
            dropped_events: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
        })
    }

    fn record(&self, record: LogRecord) {
        let sender = self
            .sender
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned());
        if let Some(sender) = sender {
            match sender.try_send(record) {
                Ok(()) => {
                    self.total_events.fetch_add(1, Ordering::SeqCst);
                }
                Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                    self.dropped_events.fetch_add(1, Ordering::SeqCst);
                }
            }
        } else {
            self.dropped_events.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn finish(&self) -> eyre::Result<()> {
        let sender = match self.sender.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        };
        drop(sender);
        let handle = match self.handle.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        };
        if let Some(handle) = handle {
            match handle.join() {
                Ok(res) => res?,
                Err(_) => return Err(eyre::eyre!("json log writer thread panicked")),
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::SeqCst)
    }
}

/// Filter mode for JSON log layers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum JsonLogFilter {
    /// Accept all events.
    All,
    /// Accept only events where message == "resources".
    ResourcesOnly,
    /// Accept all events except where message == "resources".
    ExcludeResources,
}

#[derive(Clone)]
struct JsonLogLayer {
    writer: Arc<JsonLogWriter>,
    filter: JsonLogFilter,
}

impl JsonLogLayer {
    #[allow(dead_code)]
    fn new(writer: Arc<JsonLogWriter>) -> Self {
        Self {
            writer,
            filter: JsonLogFilter::All,
        }
    }

    fn with_filter(writer: Arc<JsonLogWriter>, filter: JsonLogFilter) -> Self {
        Self { writer, filter }
    }
}

impl<S> tracing_subscriber::Layer<S> for JsonLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let meta = event.metadata();
        let mut visitor = JsonLogVisitor::default();
        event.record(&mut visitor);
        let mut fields = visitor.fields;
        let message = match fields.remove("message") {
            Some(JsonValue::String(value)) => Some(value),
            Some(value) => Some(value.to_string()),
            None => None,
        };

        // Apply filter based on message content
        let is_resources = message.as_deref() == Some("resources");
        match self.filter {
            JsonLogFilter::All => {}
            JsonLogFilter::ResourcesOnly if !is_resources => return,
            JsonLogFilter::ExcludeResources if is_resources => return,
            _ => {}
        }

        let record = LogRecord {
            t_ms: self.writer.started_at.elapsed().as_millis() as u64,
            level: meta.level().as_str().to_string(),
            target: meta.target().to_string(),
            file: meta.file().map(|file| file.to_string()),
            line: meta.line(),
            message,
            fields,
        };
        self.writer.record(record);
    }
}

fn run_timestamp_utc(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}__{:02}-{:02}-{:02}",
        year, month, day, hour, min, sec
    )
}

fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as i32, d as i32)
}

#[allow(dead_code)]
fn sanitize_label(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn default_peer_cache_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".stateless-history-node"))
}

fn build_info() -> BuildInfo {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let mut features = Vec::new();
    if cfg!(feature = "jemalloc") {
        features.push("jemalloc".to_string());
    }
    BuildInfo {
        profile: profile.to_string(),
        debug_assertions: cfg!(debug_assertions),
        features,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn env_info() -> EnvInfo {
    EnvInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        cpu_count: std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1),
        pid: process::id(),
    }
}

async fn wait_for_min_peers(pool: &Arc<PeerPool>, min_peers: usize) {
    if min_peers == 0 {
        return;
    }
    let mut last_log = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    loop {
        let peers = pool.len();
        if peers >= min_peers {
            return;
        }
        if last_log.elapsed() >= Duration::from_secs(5) {
            info!(peers, min_peers, "waiting for peer warmup");
            last_log = Instant::now();
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

fn run_base_name(
    _run_name: &str,
    timestamp: &str,
    _range: &std::ops::RangeInclusive<u64>,
    _config: &NodeConfig,
    _build: &BuildInfo,
) -> String {
    timestamp.to_string()
}

fn write_run_report(output_dir: &Path, base_name: &str, report: &RunReport) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(format!("{base_name}.json"));
    let file = fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(path)
}

/// Finalize log files by renaming tmp files to final names.
/// Called on any clean exit (Ctrl-C or natural completion).
fn finalize_log_files(
    run_context: &RunContext,
    base_name: &str,
    events: Option<&BenchEventLogger>,
    log_writer: Option<&JsonLogWriter>,
    resources_writer: Option<&JsonLogWriter>,
    chrome_guard: &mut Option<tracing_chrome::FlushGuard>,
) {
    // Finalize events log
    if let Some(logger) = events {
        if let Err(err) = logger.finish() {
            warn!(error = %err, "failed to finalize event log");
        }
        if let Some(tmp_path) = run_context.events_tmp_path.as_ref() {
            let final_path = run_context
                .output_dir
                .join(format!("{base_name}.events.jsonl"));
            if let Err(err) = fs::rename(tmp_path, &final_path) {
                warn!(error = %err, "failed to rename event log");
            } else {
                info!(path = %final_path.display(), "event log written");
            }
        }
    }

    // Finalize JSON logs
    if let Some(logger) = log_writer {
        if let Err(err) = logger.finish() {
            warn!(error = %err, "failed to finalize json logs");
        }
        if let Some(tmp_path) = run_context.logs_tmp_path.as_ref() {
            let final_path = run_context
                .output_dir
                .join(format!("{base_name}.logs.jsonl"));
            if let Err(err) = fs::rename(tmp_path, &final_path) {
                warn!(error = %err, "failed to rename json log");
            } else {
                info!(path = %final_path.display(), "json log written");
            }
        }
    }

    // Finalize resources log
    if let Some(logger) = resources_writer {
        if let Err(err) = logger.finish() {
            warn!(error = %err, "failed to finalize resources log");
        }
        if let Some(tmp_path) = run_context.resources_tmp_path.as_ref() {
            let final_path = run_context
                .output_dir
                .join(format!("{base_name}.resources.jsonl"));
            if let Err(err) = fs::rename(tmp_path, &final_path) {
                warn!(error = %err, "failed to rename resources log");
            } else {
                info!(path = %final_path.display(), "resources log written");
            }
        }
    }

    // Finalize Chrome trace
    if let Some(tmp_path) = run_context.trace_tmp_path.as_ref() {
        let final_path = run_context
            .output_dir
            .join(format!("{base_name}.trace.json"));
        drop(chrome_guard.take());
        if let Err(err) = fs::rename(tmp_path, &final_path) {
            warn!(error = %err, "failed to rename trace");
        } else {
            info!(path = %final_path.display(), "trace written");
        }
    }
}

/// Generate and save run report. Returns the base_name used for file naming.
/// Report is saved only if `config.log_report` is true.
/// Summary is printed to console only if `config.verbosity >= 1`.
async fn generate_run_report(
    run_context: &RunContext,
    config: &NodeConfig,
    range: &std::ops::RangeInclusive<u64>,
    head_at_startup: u64,
    peer_health: &PeerHealthTracker,
    summary: &sync::historical::IngestBenchSummary,
) -> Result<String> {
    let build = build_info();
    let env = env_info();
    let base_name = run_base_name(
        &run_context.run_name,
        &run_context.timestamp_utc,
        range,
        config,
        &build,
    );

    // Print summary to console if verbose
    if config.verbosity >= 1 {
        let summary_json = serde_json::to_string_pretty(summary)?;
        println!("{summary_json}");
    }

    // Save report file if --log-report is enabled
    if config.log_report {
        let limits = p2p_limits();
        let derived = RunDerived {
            range_start: *range.start(),
            range_end: *range.end(),
            head_at_startup,
            safe_head: head_at_startup.saturating_sub(config.rollback_window),
            rollback_window_applied: config.rollback_window > 0,
            worker_count: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(4)
                .max(1),
            p2p_limits: P2pLimitsSummary {
                max_outbound: limits.max_outbound,
                max_concurrent_dials: limits.max_concurrent_dials,
                peer_refill_interval_ms: limits.peer_refill_interval_ms,
                request_timeout_ms: limits.request_timeout_ms,
                max_headers_per_request: limits.max_headers_per_request,
                peer_cache_ttl_days: limits.peer_cache_ttl_days,
                peer_cache_max: limits.peer_cache_max,
            },
        };

        let mut snapshot = peer_health.snapshot().await;
        let peer_health_all: Vec<PeerHealthSummary> = snapshot
            .iter()
            .filter(|dump| dump.quality_samples > 0)
            .map(|dump| PeerHealthSummary {
                peer_id: format!("{:?}", dump.peer_id),
                is_banned: dump.is_banned,
                ban_remaining_ms: dump.ban_remaining_ms,
                consecutive_failures: dump.consecutive_failures,
                consecutive_partials: dump.consecutive_partials,
                successes: dump.successes,
                failures: dump.failures,
                partials: dump.partials,
                assignments: dump.assignments,
                assigned_blocks: dump.assigned_blocks,
                inflight_blocks: dump.inflight_blocks,
                batch_limit: dump.batch_limit as u64,
                batch_limit_max: dump.batch_limit_max as u64,
                batch_limit_avg: dump.batch_limit_avg,
                last_assigned_age_ms: dump.last_assigned_age_ms,
                last_success_age_ms: dump.last_success_age_ms,
                last_failure_age_ms: dump.last_failure_age_ms,
                last_partial_age_ms: dump.last_partial_age_ms,
                quality_score: dump.quality_score,
                quality_samples: dump.quality_samples,
                last_error: dump.last_error.clone(),
                last_error_age_ms: dump.last_error_age_ms,
                last_error_count: dump.last_error_count,
            })
            .collect();

        let top: Vec<PeerHealthSummary> = snapshot
            .iter()
            .filter(|dump| dump.quality_samples > 0)
            .take(10)
            .map(|dump| PeerHealthSummary {
                peer_id: format!("{:?}", dump.peer_id),
                is_banned: dump.is_banned,
                ban_remaining_ms: dump.ban_remaining_ms,
                consecutive_failures: dump.consecutive_failures,
                consecutive_partials: dump.consecutive_partials,
                successes: dump.successes,
                failures: dump.failures,
                partials: dump.partials,
                assignments: dump.assignments,
                assigned_blocks: dump.assigned_blocks,
                inflight_blocks: dump.inflight_blocks,
                batch_limit: dump.batch_limit as u64,
                batch_limit_max: dump.batch_limit_max as u64,
                batch_limit_avg: dump.batch_limit_avg,
                last_assigned_age_ms: dump.last_assigned_age_ms,
                last_success_age_ms: dump.last_success_age_ms,
                last_failure_age_ms: dump.last_failure_age_ms,
                last_partial_age_ms: dump.last_partial_age_ms,
                quality_score: dump.quality_score,
                quality_samples: dump.quality_samples,
                last_error: dump.last_error.clone(),
                last_error_age_ms: dump.last_error_age_ms,
                last_error_count: dump.last_error_count,
            })
            .collect();

        snapshot.sort_by(|a, b| {
            a.quality_score
                .partial_cmp(&b.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let worst: Vec<PeerHealthSummary> = snapshot
            .iter()
            .filter(|dump| dump.quality_samples > 0)
            .take(10)
            .map(|dump| PeerHealthSummary {
                peer_id: format!("{:?}", dump.peer_id),
                is_banned: dump.is_banned,
                ban_remaining_ms: dump.ban_remaining_ms,
                consecutive_failures: dump.consecutive_failures,
                consecutive_partials: dump.consecutive_partials,
                successes: dump.successes,
                failures: dump.failures,
                partials: dump.partials,
                assignments: dump.assignments,
                assigned_blocks: dump.assigned_blocks,
                inflight_blocks: dump.inflight_blocks,
                batch_limit: dump.batch_limit as u64,
                batch_limit_max: dump.batch_limit_max as u64,
                batch_limit_avg: dump.batch_limit_avg,
                last_assigned_age_ms: dump.last_assigned_age_ms,
                last_success_age_ms: dump.last_success_age_ms,
                last_failure_age_ms: dump.last_failure_age_ms,
                last_partial_age_ms: dump.last_partial_age_ms,
                quality_score: dump.quality_score,
                quality_samples: dump.quality_samples,
                last_error: dump.last_error.clone(),
                last_error_age_ms: dump.last_error_age_ms,
                last_error_count: dump.last_error_count,
            })
            .collect();

        let report = RunReport {
            meta: RunMeta {
                timestamp_utc: run_context.timestamp_utc.clone(),
                run_id: run_context.run_id.clone(),
                run_name: run_context.run_name.clone(),
                build,
                env,
            },
            argv: run_context.argv.clone(),
            config: config.clone(),
            derived,
            results: summary.clone(),
            peer_health_all,
            peer_health_top: top,
            peer_health_worst: worst,
            events: None,
            logs: None,
        };
        let report_path = write_run_report(&run_context.output_dir, &base_name, &report)?;
        info!(path = %report_path.display(), "run report written");
    }

    Ok(base_name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = NodeConfig::from_args();
    let argv: Vec<String> = env::args().collect();
    // Check if any log artifacts are enabled
    let log_artifacts_enabled =
        config.log_trace || config.log_events || config.log_json || config.log_report || config.log_resources;
    let chunk_max = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1);
    if chunk_max < config.fast_sync_chunk_size.max(1) {
        warn!(
            initial = config.fast_sync_chunk_size,
            max = chunk_max,
            "fast-sync-chunk-max is below fast-sync-chunk-size; clamping initial to max"
        );
        config.fast_sync_chunk_size = chunk_max;
    }
    config.fast_sync_chunk_max = Some(chunk_max);
    let run_context = if log_artifacts_enabled {
        let timestamp_utc = run_timestamp_utc(SystemTime::now());
        let run_id = format!("{}-{}", timestamp_utc, process::id());
        let run_name = config
            .run_name
            .clone()
            .unwrap_or_else(|| "sync".to_string());
        let output_dir = config.log_output_dir.clone();
        let trace_tmp_path = if config.log_trace {
            Some(output_dir.join(format!("{}.trace.json.part", timestamp_utc)))
        } else {
            None
        };
        let events_tmp_path = if config.log_events {
            Some(output_dir.join(format!("{}.events.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        let logs_tmp_path = if config.log_json {
            Some(output_dir.join(format!("{}.logs.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        let resources_tmp_path = if config.log_resources {
            Some(output_dir.join(format!("{}.resources.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        fs::create_dir_all(&output_dir)?;
        Some(RunContext {
            timestamp_utc,
            run_id,
            run_name,
            output_dir,
            argv,
            trace_tmp_path,
            events_tmp_path,
            logs_tmp_path,
            resources_tmp_path,
        })
    } else {
        None
    };
    if config.peer_cache_dir.is_none() {
        config.peer_cache_dir = default_peer_cache_dir();
    }
    let mut tracing_guards = init_tracing(
        &config,
        run_context
            .as_ref()
            .and_then(|ctx| ctx.trace_tmp_path.clone()),
        run_context
            .as_ref()
            .and_then(|ctx| ctx.logs_tmp_path.clone()),
        run_context
            .as_ref()
            .and_then(|ctx| ctx.resources_tmp_path.clone()),
    );

    if let Some(command) = &config.command {
        match command {
            cli::Command::Db(cli::DbCommand::Stats(args)) => {
                let data_dir = args
                    .data_dir
                    .clone()
                    .unwrap_or_else(|| config.data_dir.clone());
                let stats = storage::Storage::disk_usage_at(&data_dir)?;
                print_db_stats(&data_dir, &stats, args.json)?;
                return Ok(());
            }
        }
    }

    // Main ingest path
    info!(
        chain_id = config.chain_id,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );
    if !matches!(config.head_source, cli::HeadSource::P2p) {
        return Err(eyre::eyre!("ingest requires --head-source p2p"));
    }

    let storage = Arc::new(storage::Storage::open(&config)?);

    // Resume behavior: if the previous run exited before compaction finished, ensure we
    // compact any completed shards up-front so we don't keep reprocessing WAL-heavy shards
    // across restarts.
    let dirty_shards = storage.dirty_complete_shards()?;
    if !dirty_shards.is_empty() {
        let storage = Arc::clone(&storage);
        let shard_count = dirty_shards.len();
        info!(shard_count, "startup: compacting completed dirty shards");
        tokio::task::spawn_blocking(move || -> Result<()> {
            for shard_start in dirty_shards {
                storage.compact_shard(shard_start)?;
            }
            Ok(())
        })
        .await??;
        info!(shard_count, "startup: completed shard compaction done");
    }

    info!("starting p2p network");
    let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
    info!(peers = session.pool.len(), "p2p peers connected");
    let min_peers = config.min_peers as usize;
    wait_for_min_peers(&session.pool, min_peers).await;
    info!(
        peers = session.pool.len(),
        min_peers, "peer warmup complete"
    );
    let follow_after = config.end_block.is_none();

    let head_at_startup = wait_for_peer_head(
        &session.pool,
        config.start_block,
        config.end_block,
        config.rollback_window,
    )
    .await;
    let range = compute_target_range(
        config.start_block,
        config.end_block,
        head_at_startup,
        config.rollback_window,
    );
    let run_ctx = run_context.as_ref();
    let missing_started = Instant::now();
    let (missing_ranges, missing_total) = storage.missing_ranges_in_range(range.clone())?;
    let missing_elapsed_ms = missing_started.elapsed().as_millis() as u64;
    let requested_total = range_len(&range);
    let already_present = requested_total.saturating_sub(missing_total);
    if already_present > 0 {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            already_present,
            missing_total,
            elapsed_ms = missing_elapsed_ms,
            "resume: found existing blocks in range; skipping already present blocks"
        );
    } else {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            missing_total,
            elapsed_ms = missing_elapsed_ms,
            "resume: no existing blocks in range"
        );
    }
    let total_len = missing_total;
    let blocks = sync::historical::MissingBlocks::Ranges {
        ranges: missing_ranges,
        total: missing_total,
    };

    let progress_stats = if std::io::stderr().is_terminal() {
        Some(Arc::new(SyncProgressStats::default()))
    } else {
        None
    };
    if let Some(stats) = progress_stats.as_ref() {
        stats.set_head_seen(head_at_startup);
    }
    let events = if config.log_events {
        let run_ctx = run_ctx.expect("run context required for log_events");
        let tmp_path = run_ctx
            .events_tmp_path
            .clone()
            .expect("events tmp path");
        Some(Arc::new(BenchEventLogger::new(tmp_path)?))
    } else {
        None
    };
    spawn_resource_logger(progress_stats.clone(), events.clone());
    let peer_health_local = sync::historical::build_peer_health_tracker(&config);
    apply_cached_peer_limits(&storage, &peer_health_local).await;
    #[cfg(unix)]
    spawn_usr1_state_logger(
        progress_stats.clone(),
        Some(Arc::clone(&session.pool)),
        Some(Arc::clone(&peer_health_local)),
    );
    let progress: Option<Arc<IngestProgress>> = if std::io::stderr().is_terminal() {
        let bar = ProgressBar::new(total_len);
        bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        let style = ProgressStyle::with_template(
            "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
        )
        .expect("progress style");
        bar.set_style(style.progress_chars("█▉░"));
        bar.set_message(format_progress_message(
            SyncStatus::LookingForPeers,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0.0,
            "--",
        ));
        if let Some(stats) = progress_stats.as_ref() {
            stats.set_peers_active(0);
            stats.set_peers_total(session.pool.len() as u64);
            spawn_progress_updater(
                bar.clone(),
                Arc::clone(stats),
                total_len,
                Arc::clone(&session.pool),
                Some(Arc::clone(&peer_health_local)),
            );
        }
        Some(Arc::new(IngestProgress::new(bar, total_len)))
    } else {
        None
    };

    let mut head_tracker_handle = None;
    let mut tail_feeder_handle = None;
    let mut head_stop_tx = None;
    let mut tail_stop_tx = None;
    let mut tail_config = None;
    if follow_after {
            let (head_stop, head_stop_rx) = tokio::sync::watch::channel(false);
            let (tail_stop, tail_stop_rx) = tokio::sync::watch::channel(false);
            head_stop_tx = Some(head_stop.clone());
            tail_stop_tx = Some(tail_stop.clone());

            let (head_seen_tx, head_seen_rx) = tokio::sync::watch::channel(head_at_startup);
            let pool = Arc::clone(&session.pool);
            let storage = Arc::clone(&storage);
            let stats = progress_stats.clone();
            let rollback_window = config.rollback_window;
            let mut stop_rx_head = head_stop_rx.clone();
            head_tracker_handle = Some(tokio::spawn(async move {
                let mut last_head = head_at_startup;
                loop {
                    if *stop_rx_head.borrow() {
                        break;
                    }
                    let snapshot = pool.snapshot();
                    let best_head = snapshot
                        .iter()
                        .map(|peer| peer.head_number)
                        .max()
                        .unwrap_or(last_head);
                    if best_head > last_head {
                        last_head = best_head;
                        let _ = head_seen_tx.send(best_head);
                        if let Err(err) = storage.set_head_seen(best_head) {
                            tracing::debug!(error = %err, "head tracker: failed to persist head_seen");
                        }
                        if let Some(stats) = stats.as_ref() {
                            stats.set_head_seen(best_head);
                        }
                    }
                    tokio::select! {
                        _ = stop_rx_head.changed() => {
                            if *stop_rx_head.borrow() {
                                break;
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                    }
                }
            }));

            let (tail_tx, tail_rx) = tokio::sync::mpsc::unbounded_channel();
            let mut stop_rx_tail = tail_stop_rx.clone();
            let mut head_seen_rx_tail = head_seen_rx.clone();
            let mut next_to_schedule = range.end().saturating_add(1);
            tail_feeder_handle = Some(tokio::spawn(async move {
                loop {
                    if *stop_rx_tail.borrow() {
                        break;
                    }
                    let head_seen = *head_seen_rx_tail.borrow();
                    let safe_head = head_seen.saturating_sub(rollback_window);
                    if safe_head >= next_to_schedule {
                        let start = next_to_schedule;
                        let end = safe_head;
                        let _ = tail_tx.send(start..=end);
                        next_to_schedule = end.saturating_add(1);
                    }
                    tokio::select! {
                        _ = head_seen_rx_tail.changed() => {}
                        _ = stop_rx_tail.changed() => {
                            if *stop_rx_tail.borrow() {
                                break;
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                    }
                }
            }));

            tail_config = Some(sync::historical::TailIngestConfig {
                ranges_rx: tail_rx,
                head_seen_rx,
                stop_tx: tail_stop,
                stop_when_caught_up: true,
                head_offset: config.rollback_window,
            });
        }

        let bench = Arc::new(IngestBenchStats::new(total_len));
        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let head_stop_tx_for_shutdown = head_stop_tx.clone();
        let tail_stop_tx_for_shutdown = tail_stop_tx.clone();
        let shutdown_requested = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_flag = Arc::clone(&shutdown_requested);
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                warn!("shutdown signal received; stopping ingest after draining");
                let _ = stop_tx.send(true);
                if let Some(stop_tx) = head_stop_tx_for_shutdown {
                    let _ = stop_tx.send(true);
                }
                if let Some(stop_tx) = tail_stop_tx_for_shutdown {
                    let _ = stop_tx.send(true);
                }
                if tokio::signal::ctrl_c().await.is_ok() {
                    warn!("second shutdown signal received; forcing exit");
                    process::exit(130);
                }
            }
        });
        let outcome = sync::historical::run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            range.clone(),
            blocks,
            head_at_startup,
            progress_ref,
            progress_stats.clone(),
            Some(Arc::clone(&bench)),
            None,
            None,
            Arc::clone(&peer_health_local),
            events.clone(),
            Some(stop_rx),
            tail_config,
        )
        .await?;

        let finalize_stats = match &outcome {
            sync::historical::IngestPipelineOutcome::RangeApplied { finalize, .. } => {
                Some(*finalize)
            }
            _ => None,
        };
        if follow_after {
            let last_indexed = storage.last_indexed_block().ok().flatten();
            let head_seen = storage.head_seen().ok().flatten();
            let safe_head = head_seen.map(|head| head.saturating_sub(config.rollback_window));
            let dirty = storage.dirty_shards().unwrap_or_default();
            let total_wal_bytes: u64 = dirty.iter().map(|info| info.wal_bytes).sum();
            let total_wal_mib = (total_wal_bytes as f64 / (1024.0 * 1024.0)).round();
            tracing::info!(
                follow_after,
                shutdown_requested = shutdown_requested.load(std::sync::atomic::Ordering::SeqCst),
                switch_tip = ?last_indexed,
                head_seen = ?head_seen,
                safe_head = ?safe_head,
                dirty_shards = dirty.len(),
                total_wal_mib,
                drain_workers_ms = finalize_stats.map(|s| s.drain_workers_ms),
                db_flush_ms = finalize_stats.map(|s| s.db_flush_ms),
                compactions_wait_ms = finalize_stats.map(|s| s.compactions_wait_ms),
                compact_all_dirty_ms = finalize_stats.map(|s| s.compact_all_dirty_ms),
                seal_completed_ms = finalize_stats.map(|s| s.seal_completed_ms),
                total_finalize_ms = finalize_stats.map(|s| s.total_ms),
                "fast-sync -> follow boundary"
            );
        }

        if follow_after && !shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(stop_tx) = tail_stop_tx.take() {
                let _ = stop_tx.send(true);
            }
            if let Some(handle) = tail_feeder_handle.take() {
                let _ = handle.await;
            }

            info!("fast-sync complete; switching to follow mode");

            // Generate report at fast-sync completion (saved if --log-report, printed if -v).
            // Log files continue into follow mode - they'll be finalized on exit.
            let logs_total = match &outcome {
                sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => *logs,
                sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
            };
            let storage_stats = match storage.disk_usage() {
                Ok(stats) => Some(stats),
                Err(err) => {
                    warn!(error = %err, "failed to collect storage disk stats");
                    None
                }
            };
            let summary = bench.summary(
                *range.start(),
                *range.end(),
                head_at_startup,
                config.rollback_window > 0,
                session.pool.len() as u64,
                logs_total,
                storage_stats,
            );
            let report_base_name = if let Some(run_ctx) = run_context.as_ref() {
                match generate_run_report(
                    run_ctx,
                    &config,
                    &range,
                    head_at_startup,
                    &peer_health_local,
                    &summary,
                )
                .await
                {
                    Ok(name) => Some(name),
                    Err(err) => {
                        warn!(error = %err, "failed to generate run report");
                        None
                    }
                }
            } else {
                None
            };

            let progress_ref = progress
                .as_ref()
                .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
            let (synced_tx, synced_rx) = tokio::sync::oneshot::channel();
            let follow_future = sync::historical::run_follow_loop(
                Arc::clone(&storage),
                Arc::clone(&session.pool),
                &config,
                progress_ref,
                progress_stats.clone(),
                Arc::clone(&peer_health_local),
                Some(synced_tx),
            );
            tokio::pin!(follow_future);

            let mut rpc_handle = None;
            let mut synced_rx = Some(synced_rx);
            loop {
                tokio::select! {
                    res = &mut follow_future => {
                        if let Err(err) = res {
                            warn!(error = %err, "follow loop exited");
                        }
                        break;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        warn!("shutdown signal received");
                        break;
                    }
                    _ = async {
                        if let Some(rx) = synced_rx.take() {
                            let _ = rx.await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    } => {
                        if rpc_handle.is_none() {
                            let handle = rpc::start(
                                config.rpc_bind,
                                rpc::RpcConfig::from(&config),
                                Arc::clone(&storage),
                            )
                            .await?;
                            info!(rpc_bind = %config.rpc_bind, "rpc server started");
                            rpc_handle = Some(handle);
                        }
                    }
                }
            }

            if let Some(tracker) = progress.as_ref() {
                tracker.finish();
            }
            if let Some(handle) = rpc_handle.take() {
                if let Err(err) = handle.stop() {
                    warn!(error = %err, "failed to stop rpc server");
                }
                handle.stopped().await;
            }
            if let Some(stop_tx) = head_stop_tx.take() {
                let _ = stop_tx.send(true);
            }
            if let Some(handle) = head_tracker_handle.take() {
                let _ = handle.await;
            }
            // Finalize log files on exit from follow mode (report already generated)
            if let Some(run_ctx) = run_context.as_ref() {
                if let Some(base_name) = report_base_name.as_ref() {
                    finalize_log_files(
                        run_ctx,
                        base_name,
                        events.as_deref(),
                        tracing_guards.log_writer.as_deref(),
                        tracing_guards.resources_writer.as_deref(),
                        &mut tracing_guards.chrome_guard,
                    );
                }
            }
            flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
            return Ok(());
        }

        if shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(stop_tx) = tail_stop_tx.take() {
                let _ = stop_tx.send(true);
            }
            if let Some(handle) = tail_feeder_handle.take() {
                let _ = handle.await;
            }
            if let Some(stop_tx) = head_stop_tx.take() {
                let _ = stop_tx.send(true);
            }
            if let Some(handle) = head_tracker_handle.take() {
                let _ = handle.await;
            }
            if let Some(tracker) = progress.as_ref() {
                tracker.finish();
            }
            // Generate report and finalize logs on Ctrl-C during fast-sync
            if let Some(run_ctx) = run_context.as_ref() {
                let logs_total = match &outcome {
                    sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => *logs,
                    sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
                };
                let storage_stats = match storage.disk_usage() {
                    Ok(stats) => Some(stats),
                    Err(err) => {
                        warn!(error = %err, "failed to collect storage disk stats");
                        None
                    }
                };
                let summary = bench.summary(
                    *range.start(),
                    *range.end(),
                    head_at_startup,
                    config.rollback_window > 0,
                    session.pool.len() as u64,
                    logs_total,
                    storage_stats,
                );
                let base_name = match generate_run_report(
                    run_ctx,
                    &config,
                    &range,
                    head_at_startup,
                    &peer_health_local,
                    &summary,
                )
                .await
                {
                    Ok(name) => name,
                    Err(err) => {
                        warn!(error = %err, "failed to generate run report");
                        // Fallback base name
                        run_ctx.timestamp_utc.clone()
                    }
                };
                finalize_log_files(
                    run_ctx,
                    &base_name,
                    events.as_deref(),
                    tracing_guards.log_writer.as_deref(),
                    tracing_guards.resources_writer.as_deref(),
                    &mut tracing_guards.chrome_guard,
                );
            }
            flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
            return Ok(());
        }

        if let Some(tracker) = progress.as_ref() {
            tracker.finish();
        }

        let logs_total = match outcome {
            sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => logs,
            sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
        };
        let storage_stats = match storage.disk_usage() {
            Ok(stats) => Some(stats),
            Err(err) => {
                warn!(error = %err, "failed to collect storage disk stats");
                None
            }
        };
        let summary = bench.summary(
            *range.start(),
            *range.end(),
            head_at_startup,
            config.rollback_window > 0,
            session.pool.len() as u64,
            logs_total,
            storage_stats,
        );
    // Generate report (if --log-report) and finalize log files.
    if let Some(run_ctx) = run_ctx {
        let base_name = match generate_run_report(
            run_ctx,
            &config,
            &range,
            head_at_startup,
            &peer_health_local,
            &summary,
        )
        .await
        {
            Ok(name) => name,
            Err(err) => {
                warn!(error = %err, "failed to generate run report");
                // Fallback base name
                run_ctx.timestamp_utc.clone()
            }
        };
        finalize_log_files(
            run_ctx,
            &base_name,
            events.as_deref(),
            tracing_guards.log_writer.as_deref(),
            tracing_guards.resources_writer.as_deref(),
            &mut tracing_guards.chrome_guard,
        );
    }
    flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
    Ok(())
}

async fn wait_for_peer_head(
    pool: &Arc<PeerPool>,
    start_block: u64,
    end_block: Option<u64>,
    rollback_window: u64,
) -> u64 {
    let mut last_log = Instant::now() - Duration::from_secs(10);
    loop {
        let peers = pool.snapshot();
        let best_head = peers.iter().map(|peer| peer.head_number).max().unwrap_or(0);
        let safe_head = best_head.saturating_sub(rollback_window);
        let requested_end = end_block.unwrap_or(safe_head);
        let end = requested_end.min(safe_head);
        if start_block <= end {
            return best_head;
        }
        if last_log.elapsed() >= Duration::from_secs(5) {
            info!(
                peers = peers.len(),
                best_head,
                safe_head,
                start_block,
                requested_end,
                "waiting for peer head to reach start block"
            );
            last_log = Instant::now();
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

struct TracingGuards {
    chrome_guard: Option<tracing_chrome::FlushGuard>,
    log_writer: Option<Arc<JsonLogWriter>>,
    resources_writer: Option<Arc<JsonLogWriter>>,
}

fn init_tracing(
    config: &NodeConfig,
    chrome_trace_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    resources_path: Option<PathBuf>,
) -> TracingGuards {
    let log_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => {
            let (global, local) = match config.verbosity {
                0 => ("error", "error"),
                1 => ("warn", "info"),
                2 => ("warn", "debug"),
                _ => ("warn", "trace"),
            };
            EnvFilter::new(format!("{global},stateless_history_node={local}"))
        }
    };
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(log_filter);

    // JSON log file uses its own filter (defaults to DEBUG level).
    let json_log_filter = EnvFilter::try_new(&config.log_json_filter)
        .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_JSON_FILTER));

    let log_writer = log_path.and_then(|path| match JsonLogWriter::new(path, LOG_BUFFER) {
        Ok(writer) => Some(Arc::new(writer)),
        Err(err) => {
            warn!(error = %err, "failed to initialize json log writer");
            None
        }
    });

    // Resources log file writer (separate file for resource metrics).
    let resources_writer = resources_path.and_then(|path| match JsonLogWriter::new(path, LOG_BUFFER) {
        Ok(writer) => Some(Arc::new(writer)),
        Err(err) => {
            warn!(error = %err, "failed to initialize resources log writer");
            None
        }
    });

    // Determine JSON log filter mode: exclude resources if we have a separate resources file.
    let json_filter_mode = if resources_writer.is_some() {
        JsonLogFilter::ExcludeResources
    } else {
        JsonLogFilter::All
    };

    let mut chrome_guard = None;
    if let Some(path) = chrome_trace_path {
        let trace_filter = EnvFilter::try_new(&config.log_trace_filter)
            .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_TRACE_FILTER));
        let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(path)
            .include_args(config.log_trace_include_args)
            .include_locations(config.log_trace_include_locations)
            .build();
        let log_layer = log_writer
            .as_ref()
            .map(|writer| JsonLogLayer::with_filter(Arc::clone(writer), json_filter_mode).with_filter(json_log_filter.clone()));
        let resources_layer = resources_writer
            .as_ref()
            .map(|writer| JsonLogLayer::with_filter(Arc::clone(writer), JsonLogFilter::ResourcesOnly).with_filter(json_log_filter));
        let registry = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(log_layer)
            .with(resources_layer);
        registry.with(chrome_layer.with_filter(trace_filter)).init();
        chrome_guard = Some(guard);
    } else {
        let log_layer = log_writer
            .as_ref()
            .map(|writer| JsonLogLayer::with_filter(Arc::clone(writer), json_filter_mode).with_filter(json_log_filter.clone()));
        let resources_layer = resources_writer
            .as_ref()
            .map(|writer| JsonLogLayer::with_filter(Arc::clone(writer), JsonLogFilter::ResourcesOnly).with_filter(json_log_filter));
        let registry = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(log_layer)
            .with(resources_layer);
        registry.init();
    }
    TracingGuards {
        chrome_guard,
        log_writer,
        resources_writer,
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct ProcMemSample {
    rss_kb: u64,
    swap_kb: u64,
    rss_anon_kb: u64,
    rss_file_kb: u64,
    rss_shmem_kb: u64,
}

#[cfg(target_os = "linux")]
fn read_proc_status_mem_kb() -> Option<ProcMemSample> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let mut rss = None;
    let mut swap = None;
    let mut rss_anon = None;
    let mut rss_file = None;
    let mut rss_shmem = None;
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            rss = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok());
        } else if line.starts_with("VmSwap:") {
            swap = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok());
        } else if line.starts_with("RssAnon:") {
            rss_anon = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok());
        } else if line.starts_with("RssFile:") {
            rss_file = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok());
        } else if line.starts_with("RssShmem:") {
            rss_shmem = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok());
        }
    }
    if rss.is_none()
        && swap.is_none()
        && rss_anon.is_none()
        && rss_file.is_none()
        && rss_shmem.is_none()
    {
        return None;
    }
    Some(ProcMemSample {
        rss_kb: rss.unwrap_or(0),
        swap_kb: swap.unwrap_or(0),
        rss_anon_kb: rss_anon.unwrap_or(0),
        rss_file_kb: rss_file.unwrap_or(0),
        rss_shmem_kb: rss_shmem.unwrap_or(0),
    })
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
    iowait: u64,
}

#[cfg(target_os = "linux")]
fn read_proc_cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }
    let mut parts = line.split_whitespace();
    parts.next()?;
    let mut values = Vec::with_capacity(8);
    for _ in 0..8 {
        let value = parts.next()?.parse::<u64>().ok()?;
        values.push(value);
    }
    let user = values[0];
    let nice = values[1];
    let system = values[2];
    let idle = values[3];
    let iowait = values[4];
    let irq = values[5];
    let softirq = values[6];
    let steal = values[7];
    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    Some(CpuSample {
        total,
        idle: idle + iowait,
        iowait,
    })
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct DiskSample {
    read_sectors: u64,
    write_sectors: u64,
}

#[cfg(target_os = "linux")]
fn is_disk_device(name: &str) -> bool {
    if name.starts_with("loop") || name.starts_with("ram") {
        return false;
    }
    if name.starts_with("dm-") {
        return false;
    }
    if name.starts_with("nvme") {
        return !name.contains('p');
    }
    if name.starts_with("sd") || name.starts_with("vd") || name.starts_with("xvd") {
        return name
            .chars()
            .last()
            .map(|c| !c.is_ascii_digit())
            .unwrap_or(false);
    }
    if name.starts_with("md") {
        return true;
    }
    true
}

#[cfg(target_os = "linux")]
fn read_proc_disk_sample() -> Option<DiskSample> {
    let stats = fs::read_to_string("/proc/diskstats").ok()?;
    let mut read_sectors = 0u64;
    let mut write_sectors = 0u64;
    let mut found = false;
    for line in stats.lines() {
        let mut parts = line.split_whitespace();
        let _major = parts.next();
        let _minor = parts.next();
        let name = match parts.next() {
            Some(name) => name,
            None => continue,
        };
        if !is_disk_device(name) {
            continue;
        }
        let fields: Vec<&str> = parts.collect();
        if fields.len() < 7 {
            continue;
        }
        if let (Ok(sectors_read), Ok(sectors_written)) =
            (fields[2].parse::<u64>(), fields[6].parse::<u64>())
        {
            read_sectors = read_sectors.saturating_add(sectors_read);
            write_sectors = write_sectors.saturating_add(sectors_written);
            found = true;
        }
    }
    if !found {
        return None;
    }
    Some(DiskSample {
        read_sectors,
        write_sectors,
    })
}

#[cfg(target_os = "linux")]
fn spawn_resource_logger(
    stats: Option<Arc<SyncProgressStats>>,
    events: Option<Arc<BenchEventLogger>>,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        let mut prev_cpu = read_proc_cpu_sample();
        let mut prev_disk = read_proc_disk_sample();
        let mut prev_tick = Instant::now();
        loop {
            ticker.tick().await;
            let now = Instant::now();
            let dt_s = now.duration_since(prev_tick).as_secs_f64();
            prev_tick = now;

            let mem = read_proc_status_mem_kb();
            let cur_cpu = read_proc_cpu_sample();
            let cur_disk = read_proc_disk_sample();

            let cpu_metrics = match (prev_cpu, cur_cpu) {
                (Some(prev), Some(cur)) => {
                    let delta_total = cur.total.saturating_sub(prev.total);
                    let delta_idle = cur.idle.saturating_sub(prev.idle);
                    let delta_iowait = cur.iowait.saturating_sub(prev.iowait);
                    if delta_total > 0 {
                        let busy_pct =
                            (delta_total - delta_idle) as f64 / delta_total as f64 * 100.0;
                        let iowait_pct = delta_iowait as f64 / delta_total as f64 * 100.0;
                        Some((busy_pct, iowait_pct))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            prev_cpu = cur_cpu;

            let disk_metrics = match (prev_disk, cur_disk) {
                (Some(prev), Some(cur)) if dt_s > 0.0 => {
                    let delta_read = cur.read_sectors.saturating_sub(prev.read_sectors);
                    let delta_write = cur.write_sectors.saturating_sub(prev.write_sectors);
                    let read_mib_s = (delta_read as f64 * 512.0) / (1024.0 * 1024.0) / dt_s;
                    let write_mib_s = (delta_write as f64 * 512.0) / (1024.0 * 1024.0) / dt_s;
                    Some((read_mib_s, write_mib_s))
                }
                _ => None,
            };
            prev_disk = cur_disk;

            if mem.is_none() && cpu_metrics.is_none() && disk_metrics.is_none() {
                continue;
            }

            let (rss_kb, swap_kb, rss_anon_kb, rss_file_kb, rss_shmem_kb) = mem
                .map(|sample| {
                    (
                        sample.rss_kb,
                        sample.swap_kb,
                        sample.rss_anon_kb,
                        sample.rss_file_kb,
                        sample.rss_shmem_kb,
                    )
                })
                .unwrap_or((0, 0, 0, 0, 0));
            let (cpu_busy_pct, cpu_iowait_pct) = cpu_metrics.unwrap_or((0.0, 0.0));
            let (disk_read_mib_s, disk_write_mib_s) = disk_metrics.unwrap_or((0.0, 0.0));

            if let Some(stats) = stats.as_ref() {
                let snapshot = stats.snapshot();
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::ResourcesSample {
                        rss_kb,
                        swap_kb,
                        rss_anon_kb,
                        rss_file_kb,
                        rss_shmem_kb,
                        cpu_busy_pct,
                        cpu_iowait_pct,
                        disk_read_mib_s,
                        disk_write_mib_s,
                        status: Some(snapshot.status.as_str().to_string()),
                        processed: Some(snapshot.processed),
                        queue: Some(snapshot.queue),
                        inflight: Some(snapshot.inflight),
                        failed: Some(snapshot.failed),
                        peers_active: Some(snapshot.peers_active),
                        peers_total: Some(snapshot.peers_total),
                        head_block: Some(snapshot.head_block),
                        head_seen: Some(snapshot.head_seen),
                    });
                }
                debug!(
                    rss_kb,
                    swap_kb,
                    rss_anon_kb,
                    rss_file_kb,
                    rss_shmem_kb,
                    cpu_busy_pct,
                    cpu_iowait_pct,
                    disk_read_mib_s,
                    disk_write_mib_s,
                    status = snapshot.status.as_str(),
                    processed = snapshot.processed,
                    queue = snapshot.queue,
                    inflight = snapshot.inflight,
                    failed = snapshot.failed,
                    peers_active = snapshot.peers_active,
                    peers_total = snapshot.peers_total,
                    head_block = snapshot.head_block,
                    head_seen = snapshot.head_seen,
                    "resources"
                );
            } else {
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::ResourcesSample {
                        rss_kb,
                        swap_kb,
                        rss_anon_kb,
                        rss_file_kb,
                        rss_shmem_kb,
                        cpu_busy_pct,
                        cpu_iowait_pct,
                        disk_read_mib_s,
                        disk_write_mib_s,
                        status: None,
                        processed: None,
                        queue: None,
                        inflight: None,
                        failed: None,
                        peers_active: None,
                        peers_total: None,
                        head_block: None,
                        head_seen: None,
                    });
                }
                debug!(
                    rss_kb,
                    swap_kb,
                    rss_anon_kb,
                    rss_file_kb,
                    rss_shmem_kb,
                    cpu_busy_pct,
                    cpu_iowait_pct,
                    disk_read_mib_s,
                    disk_write_mib_s,
                    "resources"
                );
            }
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn spawn_resource_logger(
    stats: Option<Arc<SyncProgressStats>>,
    events: Option<Arc<BenchEventLogger>>,
) {
    use sysinfo::{Disks, Pid, ProcessesToUpdate, System};

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        let pid = Pid::from_u32(process::id());
        let pids = [pid];
        let mut sys = System::new();
        let mut disks = Disks::new_with_refreshed_list();
        let mut prev_tick = Instant::now();

        // Prime CPU usage (diff-based in sysinfo).
        sys.refresh_cpu_usage();

        loop {
            ticker.tick().await;
            let now = Instant::now();
            let dt_s = now.duration_since(prev_tick).as_secs_f64().max(1e-9);
            prev_tick = now;

            // Refresh process + system swap stats.
            sys.refresh_processes(ProcessesToUpdate::Some(&pids), true);
            sys.refresh_memory();
            sys.refresh_cpu_usage();
            disks.refresh(false);

            let proc = match sys.process(pid) {
                Some(proc) => proc,
                None => continue,
            };

            let rss_kb = proc.memory().saturating_div(1024);
            let cpu_count = sys.cpus().len().max(1) as f64;
            let cpu_busy_pct = sys.global_cpu_usage() as f64 / cpu_count;
            let swap_kb = sys.used_swap().saturating_div(1024);
            let cpu_iowait_pct = 0.0;
            let rss_anon_kb = 0u64;
            let rss_file_kb = 0u64;
            let rss_shmem_kb = 0u64;

            let mut disk_read_bytes = 0u64;
            let mut disk_write_bytes = 0u64;
            for disk in disks.list() {
                let usage = disk.usage();
                disk_read_bytes = disk_read_bytes.saturating_add(usage.read_bytes);
                disk_write_bytes = disk_write_bytes.saturating_add(usage.written_bytes);
            }
            let disk_read_mib_s = disk_read_bytes as f64 / (1024.0 * 1024.0) / dt_s;
            let disk_write_mib_s = disk_write_bytes as f64 / (1024.0 * 1024.0) / dt_s;

            if let Some(stats) = stats.as_ref() {
                let snapshot = stats.snapshot();
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::ResourcesSample {
                        rss_kb,
                        swap_kb,
                        rss_anon_kb,
                        rss_file_kb,
                        rss_shmem_kb,
                        cpu_busy_pct,
                        cpu_iowait_pct,
                        disk_read_mib_s,
                        disk_write_mib_s,
                        status: Some(snapshot.status.as_str().to_string()),
                        processed: Some(snapshot.processed),
                        queue: Some(snapshot.queue),
                        inflight: Some(snapshot.inflight),
                        failed: Some(snapshot.failed),
                        peers_active: Some(snapshot.peers_active),
                        peers_total: Some(snapshot.peers_total),
                        head_block: Some(snapshot.head_block),
                        head_seen: Some(snapshot.head_seen),
                    });
                }
                debug!(
                    rss_kb,
                    swap_kb,
                    rss_anon_kb,
                    rss_file_kb,
                    rss_shmem_kb,
                    cpu_busy_pct,
                    cpu_iowait_pct,
                    disk_read_mib_s,
                    disk_write_mib_s,
                    status = snapshot.status.as_str(),
                    processed = snapshot.processed,
                    queue = snapshot.queue,
                    inflight = snapshot.inflight,
                    failed = snapshot.failed,
                    peers_active = snapshot.peers_active,
                    peers_total = snapshot.peers_total,
                    head_block = snapshot.head_block,
                    head_seen = snapshot.head_seen,
                    "resources"
                );
            } else {
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::ResourcesSample {
                        rss_kb,
                        swap_kb,
                        rss_anon_kb,
                        rss_file_kb,
                        rss_shmem_kb,
                        cpu_busy_pct,
                        cpu_iowait_pct,
                        disk_read_mib_s,
                        disk_write_mib_s,
                        status: None,
                        processed: None,
                        queue: None,
                        inflight: None,
                        failed: None,
                        peers_active: None,
                        peers_total: None,
                        head_block: None,
                        head_seen: None,
                    });
                }
                debug!(
                    rss_kb,
                    swap_kb,
                    rss_anon_kb,
                    rss_file_kb,
                    rss_shmem_kb,
                    cpu_busy_pct,
                    cpu_iowait_pct,
                    disk_read_mib_s,
                    disk_write_mib_s,
                    "resources"
                );
            }
        }
    });
}

#[cfg(unix)]
fn spawn_usr1_state_logger(
    stats: Option<Arc<SyncProgressStats>>,
    peer_pool: Option<Arc<PeerPool>>,
    peer_health: Option<Arc<PeerHealthTracker>>,
) {
    tokio::spawn(async move {
        let mut stream = match signal(SignalKind::user_defined1()) {
            Ok(stream) => stream,
            Err(err) => {
                warn!(error = %err, "failed to install SIGUSR1 handler");
                return;
            }
        };
        while stream.recv().await.is_some() {
            if let Some(stats) = stats.as_ref() {
                let snapshot = stats.snapshot();
                info!(
                    status = snapshot.status.as_str(),
                    processed = snapshot.processed,
                    queue = snapshot.queue,
                    inflight = snapshot.inflight,
                    failed = snapshot.failed,
                    peers_active = snapshot.peers_active,
                    peers_total = snapshot.peers_total,
                    head_block = snapshot.head_block,
                    head_seen = snapshot.head_seen,
                    "SIGUSR1: state dump"
                );
            } else {
                info!("SIGUSR1: state dump (no progress stats available)");
            }

            if let (Some(pool), Some(health)) = (peer_pool.as_ref(), peer_health.as_ref()) {
                let peers = pool.snapshot();
                let peer_ids: Vec<_> = peers.iter().map(|p| p.peer_id).collect();
                let banned = health.count_banned_peers(&peer_ids).await;
                info!(
                    connected = peers.len(),
                    banned,
                    available = peers.len().saturating_sub(banned as usize),
                    "SIGUSR1: peer pool"
                );

                let dump = health.snapshot().await;
                let total = dump.len();
                let banned_total = dump.iter().filter(|p| p.is_banned).count();
                let top = dump.iter().take(3).collect::<Vec<_>>();
                let worst = dump.iter().rev().take(3).collect::<Vec<_>>();
                info!(total, banned_total, "SIGUSR1: peer health snapshot");
                for entry in top {
                    info!(
                        peer_id = ?entry.peer_id,
                        score = entry.quality_score,
                        samples = entry.quality_samples,
                        batch_limit = entry.batch_limit,
                        inflight_blocks = entry.inflight_blocks,
                        is_banned = entry.is_banned,
                        ban_remaining_ms = entry.ban_remaining_ms,
                        "SIGUSR1: peer health (top)"
                    );
                }
                for entry in worst {
                    info!(
                        peer_id = ?entry.peer_id,
                        score = entry.quality_score,
                        samples = entry.quality_samples,
                        batch_limit = entry.batch_limit,
                        inflight_blocks = entry.inflight_blocks,
                        is_banned = entry.is_banned,
                        ban_remaining_ms = entry.ban_remaining_ms,
                        last_error = entry.last_error.as_deref().unwrap_or(""),
                        "SIGUSR1: peer health (worst)"
                    );
                }
            }
        }
    });
}

#[allow(dead_code)]
fn total_blocks_to_head(start_from: u64, head: u64) -> u64 {
    if head >= start_from {
        head.saturating_sub(start_from).saturating_add(1)
    } else {
        0
    }
}

fn format_progress_message(
    status: SyncStatus,
    peers_active: u64,
    peers_total: u64,
    queue: u64,
    inflight: u64,
    failed: u64,
    compactions_done: u64,
    compactions_total: u64,
    speed: f64,
    eta: &str,
) -> String {
    let compact = if status == SyncStatus::Finalizing && compactions_total > 0 {
        format!(" | compact {compactions_done}/{compactions_total}")
    } else {
        String::new()
    };
    format!(
        "status {} | peers {}/{} | queue {} | inflight {} | failed {}{} | speed {:.1}/s | eta {}",
        status.as_str(),
        peers_active,
        peers_total,
        queue,
        inflight,
        failed,
        compact,
        speed,
        eta
    )
}

fn print_db_stats(data_dir: &Path, stats: &storage::StorageDiskStats, json: bool) -> Result<()> {
    if json {
        let payload = serde_json::to_string_pretty(stats)?;
        println!("{payload}");
        return Ok(());
    }

    println!("DB stats for {}", data_dir.display());
    println!("{:<18} {:>14} {:>12}", "Component", "Bytes", "Size");
    println!("{}", "-".repeat(46));
    print_row("total", stats.total_bytes);
    print_row("static", stats.static_total_bytes);
    print_row("meta", stats.meta_bytes);
    print_row("peers", stats.peers_bytes);

    println!();
    println!(
        "{:<18} {:>14} {:>12} {:>8}",
        "Segment", "Bytes", "Size", "Share"
    );
    println!("{}", "-".repeat(54));
    for segment in &stats.segments {
        let share = if stats.static_total_bytes > 0 {
            segment.bytes as f64 / stats.static_total_bytes as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "{:<18} {:>14} {:>12} {:>7.2}%",
            segment.name,
            segment.bytes,
            human_bytes(segment.bytes),
            share
        );
    }
    Ok(())
}

fn print_row(label: &str, bytes: u64) {
    println!("{:<18} {:>14} {:>12}", label, bytes, human_bytes(bytes));
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut idx = 0;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    format!("{value:.2} {}", UNITS[idx])
}

fn spawn_progress_updater(
    bar: ProgressBar,
    stats: Arc<SyncProgressStats>,
    total_len: u64,
    peer_pool: Arc<PeerPool>,
    peer_health: Option<Arc<PeerHealthTracker>>,
) {
    tokio::spawn(async move {
        let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        let mut initial_sync_done = false;
        let mut follow_bar: Option<ProgressBar> = None;
        let mut failed_bar: Option<ProgressBar> = None;
        let mut failed_total = 0u64;
        let mut last_peer_update = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .unwrap_or_else(Instant::now);

        loop {
            ticker.tick().await;
            let now = Instant::now();
            if now.duration_since(last_peer_update) >= Duration::from_millis(500) {
                last_peer_update = now;
                let connected = peer_pool.len() as u64;
                let available = if let Some(peer_health) = peer_health.as_ref() {
                    let peers = peer_pool.snapshot();
                    let peer_ids: Vec<_> = peers.iter().map(|peer| peer.peer_id).collect();
                    let banned = peer_health.count_banned_peers(&peer_ids).await;
                    connected.saturating_sub(banned)
                } else {
                    connected
                };
                stats.set_peers_total(available);
            }
            let snapshot = stats.snapshot();
            let failed = snapshot.failed;

            if failed > failed_total {
                failed_total = failed;
                if let Some(ref bar) = failed_bar {
                    bar.set_length(failed_total.max(1));
                }
            }
            if failed_total > 0 && failed_bar.is_none() {
                let fb = ProgressBar::with_draw_target(
                    Some(failed_total.max(1)),
                    ProgressDrawTarget::stderr_with_hz(2),
                );
                let style = ProgressStyle::with_template("{bar:40.red/black} {pos}/{len} | {msg}")
                    .expect("progress style")
                    .progress_chars("▓▒░");
                fb.set_style(style);
                failed_bar = Some(fb);
            }
            if let Some(ref fb) = failed_bar {
                let remaining = failed;
                let done = failed_total.saturating_sub(remaining);
                fb.set_length(failed_total.max(1));
                fb.set_position(done);
                fb.set_message(format!(
                    "recovering failed: remaining {remaining}/{failed_total}"
                ));
                if remaining == 0 {
                    fb.finish_and_clear();
                    failed_bar = None;
                    failed_total = 0;
                }
            }

            // Check if initial sync is complete and we're now in follow mode
            if !initial_sync_done {
                let processed = snapshot.processed.min(total_len);
                window.push_back((now, processed));
                while let Some((t, _)) = window.front() {
                    if now.duration_since(*t) > Duration::from_secs(1) && window.len() > 1 {
                        window.pop_front();
                    } else {
                        break;
                    }
                }
                let speed =
                    if let (Some((t0, v0)), Some((t1, v1))) = (window.front(), window.back()) {
                        let dt = t1.duration_since(*t0).as_secs_f64();
                        if dt > 0.0 && v1 >= v0 {
                            (v1 - v0) as f64 / dt
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                let remaining = total_len.saturating_sub(processed) as f64;
                let eta = if speed > 0.0 {
                    sync::format_eta_seconds(remaining / speed)
                } else {
                    "--".to_string()
                };
                let peers_active = snapshot.peers_active.min(snapshot.peers_total);
                let msg = format_progress_message(
                    snapshot.status,
                    peers_active,
                    snapshot.peers_total,
                    snapshot.queue,
                    snapshot.inflight,
                    snapshot.failed,
                    snapshot.compactions_done,
                    snapshot.compactions_total,
                    speed,
                    &eta,
                );
                bar.set_message(msg);
                bar.set_position(processed);

                // Transition to follow mode when initial sync completes
                if processed >= total_len {
                    initial_sync_done = true;
                    bar.finish_and_clear();

                    // Create the live follow status bar (message only, we format the bar ourselves)
                    let fb = ProgressBar::with_draw_target(
                        Some(100),
                        ProgressDrawTarget::stderr_with_hz(2),
                    );
                    let style = ProgressStyle::with_template("{msg}").expect("progress style");
                    fb.set_style(style);
                    follow_bar = Some(fb);
                }
            } else {
                // Live follow mode - show synced status with block number in a colored bar
                if let Some(ref fb) = follow_bar {
                    let head_block = snapshot.head_block;
                    let head_seen = snapshot.head_seen;
                    let peers_connected = peer_pool.len() as u64;
                    let peers_available = snapshot.peers_total.min(peers_connected);
                    let active_fetch = snapshot.peers_active;

                    let (status_str, status_detail) = match snapshot.status {
                        SyncStatus::Following | SyncStatus::UpToDate => ("Synced", String::new()),
                        SyncStatus::Fetching => (
                            "Catching up",
                            format!(" ({} blocks left)", head_seen.saturating_sub(head_block)),
                        ),
                        SyncStatus::Finalizing => {
                            ("Finalizing", " (flushing/compacting...)".to_string())
                        }
                        SyncStatus::LookingForPeers => ("Waiting for peers", String::new()),
                    };

                    // Format block number centered in a fixed-width field
                    let content = format!("[ {} ]", head_block);
                    let bar_width: usize = 40; // Match main progress bar width
                    let padding = bar_width.saturating_sub(content.len());
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;

                    // ANSI: force truecolor white text on truecolor green background, then reset.
                    // (Some terminals/themes remap ANSI "white" to a dark color; truecolor avoids that.)
                    let bar = format!(
                        "\x1b[38;2;255;255;255;48;2;0;128;0m{:>width_l$}{}{:<width_r$}\x1b[0m",
                        "",
                        content,
                        "",
                        width_l = left_pad,
                        width_r = right_pad,
                    );

                    let compact = if snapshot.status == SyncStatus::Finalizing
                        && snapshot.compactions_total > 0
                    {
                        let done = snapshot.compactions_done.min(snapshot.compactions_total);
                        format!(" | compact {done}/{}", snapshot.compactions_total)
                    } else {
                        String::new()
                    };

                    let msg = format!(
                        "{} {}{}{} | head {} | peers {}/{} | fetch {} | failed {}",
                        bar,
                        status_str,
                        status_detail,
                        compact,
                        head_seen,
                        peers_available,
                        peers_connected,
                        active_fetch,
                        failed,
                    );
                    fb.set_message(msg);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{format_progress_message, total_blocks_to_head, SyncStatus};

    #[test]
    fn total_blocks_handles_empty_range() {
        assert_eq!(total_blocks_to_head(10, 9), 0);
        assert_eq!(total_blocks_to_head(10, 10), 1);
        assert_eq!(total_blocks_to_head(10, 12), 3);
    }

    #[test]
    fn progress_message_formats_status() {
        assert_eq!(
            format_progress_message(SyncStatus::Fetching, 2, 5, 10, 1, 0, 0, 0, 1.5, "12s"),
            "status fetching | peers 2/5 | queue 10 | inflight 1 | failed 0 | speed 1.5/s | eta 12s"
        );
    }
}
