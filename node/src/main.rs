mod cli;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::{compute_target_range, BenchmarkMode, NodeConfig};
use eyre::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use metrics::range_len;
use serde::Serialize;
use std::{
    collections::VecDeque,
    env,
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use p2p::{PeerPool, p2p_limits};
use reth_network_api::PeerId;
use sync::{
    BlockPayloadSource, ProgressReporter, SyncProgressStats, SyncStatus,
};
use sync::historical::{BenchEventLogger, IngestBenchStats, PeerHealthTracker, ProbeStats};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

async fn apply_cached_peer_limits(
    storage: &storage::Storage,
    peer_health: &PeerHealthTracker,
) {
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
struct BenchmarkRunReport {
    meta: BenchmarkMeta,
    argv: Vec<String>,
    config: NodeConfig,
    derived: BenchmarkDerived,
    results: sync::historical::IngestBenchSummary,
    peer_health_all: Vec<PeerHealthSummary>,
    peer_health_top: Vec<PeerHealthSummary>,
    peer_health_worst: Vec<PeerHealthSummary>,
    events: Option<BenchmarkEventsSummary>,
}

#[derive(Debug, Serialize)]
struct BenchmarkMeta {
    timestamp_utc: String,
    run_id: String,
    benchmark_name: String,
    benchmark_mode: String,
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
struct BenchmarkDerived {
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
struct BenchmarkEventsSummary {
    total_events: usize,
    dropped_events: u64,
    path: String,
}

#[derive(Debug)]
struct BenchmarkRunContext {
    timestamp_utc: String,
    run_id: String,
    benchmark_name: String,
    output_dir: PathBuf,
    argv: Vec<String>,
    trace_tmp_path: Option<PathBuf>,
}

fn bench_timestamp_utc(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
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

fn sanitize_label(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn arg_present(args: &[String], flag: &str) -> bool {
    let flag_eq = format!("{flag}=");
    args.iter().any(|arg| arg == flag || arg.starts_with(&flag_eq))
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
    let features = Vec::new();
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

fn benchmark_mode_str(mode: BenchmarkMode) -> &'static str {
    match mode {
        BenchmarkMode::Disabled => "disabled",
        BenchmarkMode::Probe => "probe",
        BenchmarkMode::Ingest => "ingest",
    }
}

fn benchmark_base_name(
    benchmark_name: &str,
    timestamp: &str,
    range: &std::ops::RangeInclusive<u64>,
    config: &NodeConfig,
    build: &BuildInfo,
) -> String {
    let base = sanitize_label(benchmark_name);
    let profile = sanitize_label(&build.profile);
    let alloc = if build.features.iter().any(|feat| feat == "jemalloc") {
        "jemalloc"
    } else {
        "system"
    };
    let chunk_max = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4));
    let chunk_max_suffix = if chunk_max != config.fast_sync_chunk_size {
        format!("__chunkmax{chunk_max}")
    } else {
        String::new()
    };
    format!(
        "{base}__{timestamp}__range-{}-{}__chunk{}{}__inflight{}__timeout{}__profile-{profile}__alloc-{alloc}",
        range.start(),
        range.end(),
        config.fast_sync_chunk_size,
        chunk_max_suffix,
        config.fast_sync_max_inflight,
        config.fast_sync_batch_timeout_ms
    )
}

fn write_benchmark_report(
    output_dir: &Path,
    base_name: &str,
    report: &BenchmarkRunReport,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(format!("{base_name}.json"));
    let file = fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(path)
}

async fn emit_benchmark_artifacts(
    bench_context: &BenchmarkRunContext,
    config: &NodeConfig,
    range: &std::ops::RangeInclusive<u64>,
    head_at_startup: u64,
    peer_health: &PeerHealthTracker,
    summary: sync::historical::IngestBenchSummary,
    events: Option<&BenchEventLogger>,
    chrome_guard: &mut Option<tracing_chrome::FlushGuard>,
) -> Result<()> {
    let build = build_info();
    let env = env_info();
    let base_name = benchmark_base_name(
        &bench_context.benchmark_name,
        &bench_context.timestamp_utc,
        range,
        config,
        &build,
    );
    let limits = p2p_limits();
    let derived = BenchmarkDerived {
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

    let events_summary = if let Some(logger) = events {
        let path = bench_context
            .output_dir
            .join(format!("{base_name}.events.jsonl"));
        let total_events = logger.dump_jsonl(&path)?;
        Some(BenchmarkEventsSummary {
            total_events,
            dropped_events: logger.dropped_events(),
            path: path.display().to_string(),
        })
    } else {
        None
    };

    let report = BenchmarkRunReport {
        meta: BenchmarkMeta {
            timestamp_utc: bench_context.timestamp_utc.clone(),
            run_id: bench_context.run_id.clone(),
            benchmark_name: bench_context.benchmark_name.clone(),
            benchmark_mode: benchmark_mode_str(config.benchmark).to_string(),
            build,
            env,
        },
        argv: bench_context.argv.clone(),
        config: config.clone(),
        derived,
        results: summary,
        peer_health_all,
        peer_health_top: top,
        peer_health_worst: worst,
        events: events_summary,
    };
    let report_path = write_benchmark_report(&bench_context.output_dir, &base_name, &report)?;
    info!(path = %report_path.display(), "benchmark summary written");

    if let Some(tmp_path) = bench_context.trace_tmp_path.as_ref() {
        let final_path = bench_context
            .output_dir
            .join(format!("{base_name}.trace.json"));
        drop(chrome_guard.take());
        match fs::rename(tmp_path, &final_path) {
            Ok(()) => {
                info!(path = %final_path.display(), "benchmark trace written");
            }
            Err(err) => {
                warn!(error = %err, tmp = %tmp_path.display(), "failed to rename trace output");
            }
        }
    }

    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    let mut config = NodeConfig::from_args();
    let argv: Vec<String> = env::args().collect();
    let data_dir_specified = arg_present(&argv, "--data-dir");
    let is_benchmark = matches!(config.benchmark, BenchmarkMode::Probe | BenchmarkMode::Ingest);
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
    let bench_context = if is_benchmark {
        let timestamp_utc = bench_timestamp_utc(SystemTime::now());
        let run_id = format!("{}-{}", timestamp_utc, process::id());
        let benchmark_name = config
            .benchmark_name
            .clone()
            .unwrap_or_else(|| format!("{}-bench", benchmark_mode_str(config.benchmark)));
        let output_dir = config.benchmark_output_dir.clone();
        let trace_tmp_path = if config.benchmark_trace {
            Some(output_dir.join(format!("trace__{}__tmp.json", timestamp_utc)))
        } else {
            None
        };
        fs::create_dir_all(&output_dir)?;
        Some(BenchmarkRunContext {
            timestamp_utc,
            run_id,
            benchmark_name,
            output_dir,
            argv,
            trace_tmp_path,
        })
    } else {
        None
    };
    if is_benchmark && !data_dir_specified {
        if let Some(ctx) = &bench_context {
            config.data_dir = ctx.output_dir.join("data").join(&ctx.run_id);
        }
    }
    if config.peer_cache_dir.is_none() {
        config.peer_cache_dir = default_peer_cache_dir();
    }
    let mut chrome_guard = init_tracing(
        config.verbosity,
        bench_context.as_ref().and_then(|ctx| ctx.trace_tmp_path.clone()),
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

    if matches!(config.benchmark, BenchmarkMode::Probe) {
        info!(
            chain_id = config.chain_id,
            data_dir = %config.data_dir.display(),
            "starting stateless history node (benchmark probe)"
        );
        if !matches!(config.head_source, cli::HeadSource::P2p) {
            return Err(eyre::eyre!("benchmark probe requires --head-source p2p"));
        }
        let peer_cache = storage::Storage::open_existing(&config)?
            .map(Arc::new);
        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(peer_cache).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let head_at_startup = source.head().await?;
        let range = compute_target_range(
            config.start_block,
            config.end_block,
            head_at_startup,
            config.rollback_window,
        );
        let blocks_total = range_len(&range);
        if blocks_total == 0 {
            return Err(eyre::eyre!("benchmark range is empty"));
        }
        let stats = Arc::new(ProbeStats::new(blocks_total));
        let probe_future = sync::historical::run_benchmark_probe(
            &config,
            Arc::clone(&session.pool),
            range.clone(),
            head_at_startup,
            Arc::clone(&stats),
        );
        tokio::pin!(probe_future);
        tokio::select! {
            res = &mut probe_future => {
                res?;
            }
            _ = tokio::signal::ctrl_c() => {
                let summary = stats.summary(
                    *range.start(),
                    *range.end(),
                    head_at_startup,
                    config.rollback_window > 0,
                    session.pool.len() as u64,
                );
                let summary_json = serde_json::to_string_pretty(&summary)?;
                println!("{summary_json}");
                if let Err(err) = session.flush_peer_cache() {
                    warn!(error = %err, "failed to flush peer cache");
                }
                return Ok(());
            }
        }
        if let Err(err) = session.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache");
        }
        return Ok(());
    }

    if matches!(config.benchmark, BenchmarkMode::Ingest) {
        info!(
            chain_id = config.chain_id,
            data_dir = %config.data_dir.display(),
            "starting stateless history node (benchmark ingest)"
        );
        if !matches!(config.head_source, cli::HeadSource::P2p) {
            return Err(eyre::eyre!("benchmark ingest requires --head-source p2p"));
        }

        let storage = Arc::new(storage::Storage::open(&config)?);

        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
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
        let bench_context = bench_context.as_ref().expect("benchmark context");
        let total_len = range_len(&range);

        let progress_stats = if std::io::stderr().is_terminal() {
            Some(Arc::new(SyncProgressStats::default()))
        } else {
            None
        };
        let peer_health_local = sync::historical::build_peer_health_tracker(&config);
        apply_cached_peer_limits(&storage, &peer_health_local).await;
        #[cfg(unix)]
        spawn_usr1_state_logger(
            progress_stats.clone(),
            Some(Arc::clone(&session.pool)),
            Some(Arc::clone(&peer_health_local)),
        );
        let events = config
            .benchmark_events
            .then(|| Arc::new(BenchEventLogger::new()));

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

        let bench = Arc::new(IngestBenchStats::new(total_len));
        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let ingest_future = sync::historical::run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            range.clone(),
            head_at_startup,
            progress_ref,
            progress_stats.clone(),
            Some(Arc::clone(&bench)),
            None,
            Arc::clone(&peer_health_local),
            events.clone(),
        );
        tokio::pin!(ingest_future);
        let outcome = tokio::select! {
            res = &mut ingest_future => res?,
            _ = tokio::signal::ctrl_c() => {
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
                    bench.logs_total(),
                    storage_stats,
                );
                let summary_json = serde_json::to_string_pretty(&summary)?;
                println!("{summary_json}");
                if let Err(err) = emit_benchmark_artifacts(
                    bench_context,
                    &config,
                    &range,
                    head_at_startup,
                    &peer_health_local,
                    summary,
                    events.as_deref(),
                    &mut chrome_guard,
                )
                .await
                {
                    warn!(error = %err, "failed to write benchmark artifacts");
                }
                flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
                return Ok(());
            }
        };

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
        let summary_json = serde_json::to_string_pretty(&summary)?;
        println!("{summary_json}");
        if let Err(err) = emit_benchmark_artifacts(
            bench_context,
            &config,
            &range,
            head_at_startup,
            &peer_health_local,
            summary,
            events.as_deref(),
            &mut chrome_guard,
        )
        .await
        {
            warn!(error = %err, "failed to write benchmark artifacts");
        }
        flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
        return Ok(());
    }

    info!(
        chain_id = config.chain_id,
        rpc_bind = %config.rpc_bind,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );

    let storage = Arc::new(storage::Storage::open(&config)?);
    let head_seen = storage.head_seen()?;
    let last_indexed = storage.last_indexed_block()?;
    info!(
        head_seen = ?head_seen,
        last_indexed = ?last_indexed,
        "sync checkpoints loaded"
    );

    let rpc_handle =
        rpc::start(config.rpc_bind, rpc::RpcConfig::from(&config), Arc::clone(&storage)).await?;
    info!(rpc_bind = %config.rpc_bind, "rpc server started");

    let mut _network_session = None;
    let mut peer_health: Option<Arc<PeerHealthTracker>> = None;
    let mut follow_error: Option<eyre::Report> = None;
    let mut progress: Option<Arc<IngestProgress>> = None;
    if matches!(config.head_source, cli::HeadSource::P2p) {
        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
        let last_indexed = storage.last_indexed_block()?;
        let start_from = select_start_from(config.start_block, last_indexed);
        let initial_head = wait_for_peer_head(
            &session.pool,
            start_from,
            config.end_block,
            config.rollback_window,
        )
        .await;
        let initial_range = compute_target_range(
            start_from,
            config.end_block,
            initial_head,
            config.rollback_window,
        );
        let total_len = range_len(&initial_range);

        let progress_stats = if std::io::stderr().is_terminal() {
            Some(Arc::new(SyncProgressStats::default()))
        } else {
            None
        };
        let peer_health_local = sync::historical::build_peer_health_tracker(&config);
        apply_cached_peer_limits(&storage, &peer_health_local).await;
        peer_health = Some(Arc::clone(&peer_health_local));
        #[cfg(unix)]
        spawn_usr1_state_logger(
            progress_stats.clone(),
            Some(Arc::clone(&session.pool)),
            Some(Arc::clone(&peer_health_local)),
        );

        progress = if std::io::stderr().is_terminal() {
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

        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let follow_future = sync::historical::run_follow_loop(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            progress_ref,
            progress_stats.clone(),
            peer_health_local,
        );

        _network_session = Some(session);
        tokio::pin!(follow_future);
        tokio::select! {
            res = &mut follow_future => {
                if let Err(err) = res {
                    follow_error = Some(err);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                warn!("shutdown signal received");
            }
        }
    } else {
        tokio::signal::ctrl_c().await?;
        warn!("shutdown signal received");
    }

    if let Some(tracker) = progress.as_ref() {
        tracker.finish();
    }
    if let Err(err) = rpc_handle.stop() {
        warn!(error = %err, "failed to stop rpc server");
    }
    rpc_handle.stopped().await;
    if let Some(session) = _network_session.as_ref() {
        flush_peer_cache_with_limits(
            session,
            &storage,
            peer_health.as_ref().map(|health| health.as_ref()),
        )
        .await;
    }
    drop(_network_session);
    drop(storage);
    warn!("shutdown complete");

    if let Some(err) = follow_error {
        return Err(err);
    }

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
        let best_head = peers
            .iter()
            .map(|peer| peer.head_number)
            .max()
            .unwrap_or(0);
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

fn init_tracing(
    verbosity: u8,
    chrome_trace_path: Option<PathBuf>,
) -> Option<tracing_chrome::FlushGuard> {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => {
            let (global, local) = match verbosity {
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
        .with_filter(filter);

    let mut chrome_guard = None;
    if let Some(path) = chrome_trace_path {
        let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(path)
            .include_args(true)
            .build();
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(chrome_layer.with_filter(LevelFilter::TRACE))
            .init();
        chrome_guard = Some(guard);
    } else {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .init();
    }
    chrome_guard
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

fn select_start_from(start_block: u64, last_indexed: Option<u64>) -> u64 {
    last_indexed
        .map(|block| block.saturating_add(1))
        .unwrap_or(start_block)
        .max(start_block)
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
    speed: f64,
    eta: &str,
) -> String {
    format!(
        "status {} | peers {}/{} | queue {} | inflight {} | failed {} | speed {:.1}/s | eta {}",
        status.as_str(),
        peers_active,
        peers_total,
        queue,
        inflight,
        failed,
        speed,
        eta
    )
}

fn print_db_stats(
    data_dir: &Path,
    stats: &storage::StorageDiskStats,
    json: bool,
) -> Result<()> {
    if json {
        let payload = serde_json::to_string_pretty(stats)?;
        println!("{payload}");
        return Ok(());
    }

    println!("DB stats for {}", data_dir.display());
    println!(
        "{:<18} {:>14} {:>12}",
        "Component", "Bytes", "Size"
    );
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
    println!(
        "{:<18} {:>14} {:>12}",
        label,
        bytes,
        human_bytes(bytes)
    );
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
        let mut catchup_start_block: Option<u64> = None;
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
                let style = ProgressStyle::with_template(
                    "{bar:40.red/black} {pos}/{len} | {msg}",
                )
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
                let speed = if let (Some((t0, v0)), Some((t1, v1))) =
                    (window.front(), window.back())
                {
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
                    let style = ProgressStyle::with_template("{msg}")
                        .expect("progress style");
                    fb.set_style(style);
                    follow_bar = Some(fb);
                }
            } else {
                // Live follow mode - show synced status with block number in a colored bar
                if let Some(ref fb) = follow_bar {
                    let head_block = snapshot.head_block;
                    let head_seen = snapshot.head_seen;
                    let peers_active = snapshot.peers_active.min(snapshot.peers_total);
                    let peers_total = snapshot.peers_total;
                    
                    let status_str = if snapshot.status == SyncStatus::Following {
                        "Synced"
                    } else if snapshot.status == SyncStatus::Fetching {
                        "Catching up"
                    } else if snapshot.status == SyncStatus::LookingForPeers {
                        "Waiting for peers"
                    } else {
                        snapshot.status.as_str()
                    };

                    let catching_up =
                        snapshot.status == SyncStatus::Fetching || snapshot.status == SyncStatus::Finalizing;
                    let catchup_suffix = if catching_up {
                        let start = catchup_start_block.get_or_insert(head_block);
                        let done = head_block.saturating_sub(*start);
                        let remaining = head_seen.saturating_sub(head_block);
                        Some(format!(" {done}/{remaining}"))
                    } else {
                        catchup_start_block = None;
                        None
                    };
                    
                    // Format block number centered in a fixed-width field
                    let content = match catchup_suffix {
                        Some(suffix) => format!("[ {} ]{}", head_block, suffix),
                        None => format!("[ {} ]", head_block),
                    };
                    let bar_width: usize = 40; // Match main progress bar width
                    let padding = bar_width.saturating_sub(content.len());
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    
                    // ANSI: force truecolor white text on truecolor green background, then reset.
                    // (Some terminals/themes remap ANSI "white" to a dark color; truecolor avoids that.)
                    let bar = format!(
                        "\x1b[38;2;255;255;255;48;2;0;128;0m{:>width_l$}{}{:<width_r$}\x1b[0m",
                        "", content, "",
                        width_l = left_pad,
                        width_r = right_pad,
                    );
                    
                    let msg = format!(
                        "{} {} | head {} | peers {}/{} | failed {}",
                        bar,
                        status_str,
                        head_seen,
                        peers_active,
                        peers_total,
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
        use super::{format_progress_message, select_start_from, total_blocks_to_head, SyncStatus};

    #[test]
    fn start_from_respects_start_block() {
        assert_eq!(select_start_from(100, None), 100);
        assert_eq!(select_start_from(100, Some(50)), 100);
        assert_eq!(select_start_from(100, Some(100)), 101);
        assert_eq!(select_start_from(100, Some(150)), 151);
    }

    #[test]
    fn total_blocks_handles_empty_range() {
        assert_eq!(total_blocks_to_head(10, 9), 0);
        assert_eq!(total_blocks_to_head(10, 10), 1);
        assert_eq!(total_blocks_to_head(10, 12), 3);
    }

    #[test]
    fn progress_message_formats_status() {
        assert_eq!(
            format_progress_message(SyncStatus::Fetching, 2, 5, 10, 1, 0, 1.5, "12s"),
            "status fetching | peers 2/5 | queue 10 | inflight 1 | failed 0 | speed 1.5/s | eta 12s"
        );
    }
}
