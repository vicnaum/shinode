//! Run report types and generation for benchmarking runs.

// Verbose mode outputs JSON summary to stdout for user inspection
#![expect(clippy::print_stdout, reason = "verbose summary output to stdout")]

use crate::cli::NodeConfig;
use crate::p2p::p2p_limits;
use crate::sync::historical::{BenchEventLogger, IngestBenchSummary, PeerHealthTracker};
use eyre::Result;
use serde::Serialize;
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{info, warn};

use super::json::JsonLogWriter;

/// Complete run report containing metadata, config, and results.
#[derive(Debug, Serialize)]
pub struct RunReport {
    pub meta: RunMeta,
    pub argv: Vec<String>,
    pub config: NodeConfig,
    pub derived: RunDerived,
    pub results: IngestBenchSummary,
    pub peer_health_all: Vec<PeerHealthSummary>,
    pub peer_health_top: Vec<PeerHealthSummary>,
    pub peer_health_worst: Vec<PeerHealthSummary>,
    pub events: Option<EventsSummary>,
    pub logs: Option<LogsSummary>,
}

/// Metadata about the run.
#[derive(Debug, Serialize)]
pub struct RunMeta {
    pub timestamp_utc: String,
    pub run_id: String,
    pub run_name: String,
    pub build: BuildInfo,
    pub env: EnvInfo,
}

/// Build information.
#[derive(Debug, Serialize)]
pub struct BuildInfo {
    pub profile: String,
    pub debug_assertions: bool,
    pub features: Vec<String>,
    pub version: String,
}

/// Environment information.
#[derive(Debug, Serialize)]
pub struct EnvInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub pid: u32,
}

/// Derived values computed from config and runtime state.
#[derive(Debug, Serialize)]
pub struct RunDerived {
    pub range_start: u64,
    pub range_end: u64,
    pub head_at_startup: u64,
    pub safe_head: u64,
    pub rollback_window_applied: bool,
    pub worker_count: usize,
    pub p2p_limits: P2pLimitsSummary,
}

/// P2P configuration limits summary.
#[derive(Debug, Serialize)]
pub struct P2pLimitsSummary {
    pub max_outbound: usize,
    pub max_concurrent_dials: usize,
    pub peer_refill_interval_ms: u64,
    pub request_timeout_ms: u64,
    pub max_headers_per_request: usize,
    pub peer_cache_ttl_days: u64,
    pub peer_cache_max: usize,
}

/// Summary of a peer's health metrics.
#[derive(Debug, Serialize)]
pub struct PeerHealthSummary {
    pub peer_id: String,
    pub is_cooling_down: bool,
    pub is_stale_head: bool,
    pub backoff_remaining_ms: Option<u64>,
    pub backoff_duration_ms: u64,
    pub consecutive_failures: u32,
    pub consecutive_partials: u32,
    pub successes: u64,
    pub failures: u64,
    pub partials: u64,
    pub assignments: u64,
    pub assigned_blocks: u64,
    pub inflight_blocks: u64,
    pub batch_limit: u64,
    pub batch_limit_max: u64,
    pub batch_limit_avg: Option<f64>,
    pub last_assigned_age_ms: Option<u64>,
    pub last_success_age_ms: Option<u64>,
    pub last_failure_age_ms: Option<u64>,
    pub last_partial_age_ms: Option<u64>,
    pub quality_score: f64,
    pub quality_samples: u64,
    pub last_error: Option<String>,
    pub last_error_age_ms: Option<u64>,
    pub last_error_count: u64,
}

/// Summary of events log file.
#[derive(Debug, Serialize)]
pub struct EventsSummary {
    pub total_events: usize,
    pub dropped_events: u64,
    pub path: String,
}

/// Summary of JSON logs file.
#[derive(Debug, Serialize)]
pub struct LogsSummary {
    pub total_events: usize,
    pub dropped_events: u64,
    pub path: String,
}

/// Context for a single run, including paths for all log artifacts.
#[derive(Debug)]
pub struct RunContext {
    pub timestamp_utc: String,
    pub run_id: String,
    pub run_name: String,
    pub output_dir: PathBuf,
    pub argv: Vec<String>,
    pub trace_tmp_path: Option<PathBuf>,
    pub events_tmp_path: Option<PathBuf>,
    pub logs_tmp_path: Option<PathBuf>,
    pub resources_tmp_path: Option<PathBuf>,
}

/// Generate a UTC timestamp string from a SystemTime.
pub fn run_timestamp_utc(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}__{hour:02}-{min:02}-{sec:02}")
}

/// Convert days since Unix epoch to civil date (year, month, day).
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
    let year = y + i64::from(m <= 2);
    (year as i32, m as i32, d as i32)
}

/// Get build information.
pub fn build_info() -> BuildInfo {
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

/// Get environment information.
pub fn env_info() -> EnvInfo {
    EnvInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        cpu_count: std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1),
        pid: process::id(),
    }
}

/// Generate the base name for log files.
pub fn run_base_name(
    _run_name: &str,
    timestamp: &str,
    _range: &std::ops::RangeInclusive<u64>,
    _config: &NodeConfig,
    _build: &BuildInfo,
) -> String {
    timestamp.to_string()
}

/// Write the run report to a JSON file.
pub fn write_run_report(output_dir: &Path, base_name: &str, report: &RunReport) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(format!("{base_name}.json"));
    let file = fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(path)
}

/// Rename a temporary log file to its final path.
#[expect(clippy::cognitive_complexity, reason = "simple if-else with logging")]
fn rename_log_file(tmp_path: &Path, final_path: &Path, log_type: &str) {
    if let Err(err) = fs::rename(tmp_path, final_path) {
        warn!(error = %err, log_type, "failed to rename log file");
    } else {
        info!(path = %final_path.display(), log_type, "log file written");
    }
}

/// Finalize log files by renaming tmp files to final names.
/// Called on any clean exit (Ctrl-C or natural completion).
#[expect(clippy::cognitive_complexity, reason = "handles 4 log types with optional loggers")]
pub fn finalize_log_files(
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
            rename_log_file(tmp_path, &final_path, "events");
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
            rename_log_file(tmp_path, &final_path, "logs");
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
            rename_log_file(tmp_path, &final_path, "resources");
        }
    }

    // Finalize Chrome trace
    if let Some(tmp_path) = run_context.trace_tmp_path.as_ref() {
        let final_path = run_context
            .output_dir
            .join(format!("{base_name}.trace.json"));
        drop(chrome_guard.take());
        rename_log_file(tmp_path, &final_path, "trace");
    }
}

/// Generate and save run report. Returns the base_name used for file naming.
/// Report is saved only if `config.log_report` is true.
/// Summary is printed to console only if `config.verbosity >= 1`.
#[expect(
    clippy::too_many_lines,
    reason = "report generation has sequential formatting steps that are clearer inline"
)]
pub async fn generate_run_report(
    run_context: &RunContext,
    config: &NodeConfig,
    range: &std::ops::RangeInclusive<u64>,
    head_at_startup: u64,
    peer_health: &PeerHealthTracker,
    summary: &IngestBenchSummary,
    _peers_total: u64,
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
                .map(std::num::NonZero::get)
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
        macro_rules! peer_health_summary {
            ($dump:expr) => {
                PeerHealthSummary {
                    peer_id: format!("{:?}", $dump.peer_id),
                    is_cooling_down: $dump.is_cooling_down,
                    is_stale_head: $dump.is_stale_head,
                    backoff_remaining_ms: $dump.backoff_remaining_ms,
                    backoff_duration_ms: $dump.backoff_duration_ms,
                    consecutive_failures: $dump.consecutive_failures,
                    consecutive_partials: $dump.consecutive_partials,
                    successes: $dump.successes,
                    failures: $dump.failures,
                    partials: $dump.partials,
                    assignments: $dump.assignments,
                    assigned_blocks: $dump.assigned_blocks,
                    inflight_blocks: $dump.inflight_blocks,
                    batch_limit: $dump.batch_limit as u64,
                    batch_limit_max: $dump.batch_limit_max as u64,
                    batch_limit_avg: $dump.batch_limit_avg,
                    last_assigned_age_ms: $dump.last_assigned_age_ms,
                    last_success_age_ms: $dump.last_success_age_ms,
                    last_failure_age_ms: $dump.last_failure_age_ms,
                    last_partial_age_ms: $dump.last_partial_age_ms,
                    quality_score: $dump.quality_score,
                    quality_samples: $dump.quality_samples,
                    last_error: $dump.last_error.clone(),
                    last_error_age_ms: $dump.last_error_age_ms,
                    last_error_count: $dump.last_error_count,
                }
            };
        }
        let peer_health_all: Vec<PeerHealthSummary> = snapshot
            .iter()
            .filter(|dump| dump.quality_samples > 0)
            .map(|dump| peer_health_summary!(dump))
            .collect();

        let top: Vec<PeerHealthSummary> = snapshot
            .iter()
            .filter(|dump| dump.quality_samples > 0)
            .take(10)
            .map(|dump| peer_health_summary!(dump))
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
            .map(|dump| peer_health_summary!(dump))
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
