//! Logging, reporting, and resource monitoring module.
//!
//! This module provides:
//! - JSON structured logging with async file writing
//! - Run reports for benchmarking and diagnostics
//! - Resource monitoring (CPU, memory, disk I/O)
//! - SIGUSR1 signal handler for state dumps

mod json;
mod report;
mod resources;

pub use json::{JsonLogFilter, JsonLogLayer, JsonLogWriter, TuiLogBuffer, TuiLogLayer, LOG_BUFFER};
pub use report::{finalize_log_files, generate_run_report, run_timestamp_utc, RunContext};
pub use resources::spawn_resource_logger;
#[cfg(unix)]
pub use resources::spawn_usr1_state_logger;

use crate::cli::{NodeConfig, DEFAULT_LOG_JSON_FILTER, DEFAULT_LOG_TRACE_FILTER};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

/// Guards that must be held to keep logging active.
pub struct TracingGuards {
    pub chrome_guard: Option<tracing_chrome::FlushGuard>,
    pub log_writer: Option<Arc<JsonLogWriter>>,
    pub resources_writer: Option<Arc<JsonLogWriter>>,
    /// Shared buffer for TUI log capture (only present when TUI mode is active).
    pub tui_log_buffer: Option<Arc<TuiLogBuffer>>,
}

/// Initialize the tracing subscriber with optional Chrome trace and JSON logging.
///
/// When `tui_mode` is true, the stdout fmt_layer is suppressed to avoid
/// corrupting the TUI display.
pub fn init_tracing(
    config: &NodeConfig,
    chrome_trace_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    resources_path: Option<PathBuf>,
    tui_mode: bool,
) -> TracingGuards {
    // Default verbosity is now INFO level (previously required -v)
    let log_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let (global, local) = match config.verbosity {
            0 => ("warn", "info"),   // INFO default (was error/error)
            1 => ("warn", "debug"),  // -v = debug (was info)
            2 => ("info", "trace"),  // -vv = trace (was debug)
            _ => ("debug", "trace"), // -vvv = verbose trace
        };
        EnvFilter::new(format!("{global},stateless_history_node={local}"))
    });

    // When TUI mode is active, suppress stdout logging to avoid corrupting display
    // Instead, capture logs to a shared buffer for the TUI to display
    let (fmt_layer, tui_log_buffer) = if tui_mode {
        let buffer = Arc::new(TuiLogBuffer::new());
        (None, Some(buffer))
    } else {
        (
            Some(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_filter(log_filter),
            ),
            None,
        )
    };

    // TUI log layer captures logs for display in the TUI
    // Use minimum level based on verbosity (matches the local filter level)
    let tui_min_level = match config.verbosity {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    let show_warn = config.verbosity >= 1;
    let tui_layer = tui_log_buffer
        .as_ref()
        .map(|buffer| TuiLogLayer::new(Arc::clone(buffer), tui_min_level, show_warn));

    // JSON log file uses its own filter (defaults to DEBUG level).
    let json_log_filter = EnvFilter::try_new(&config.log_json_filter)
        .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_JSON_FILTER));

    let log_writer = log_path.and_then(|path| match JsonLogWriter::new(&path, LOG_BUFFER) {
        Ok(writer) => Some(Arc::new(writer)),
        Err(err) => {
            warn!(error = %err, "failed to initialize json log writer");
            None
        }
    });

    // Resources log file writer (separate file for resource metrics).
    let resources_writer =
        resources_path.and_then(|path| match JsonLogWriter::new(&path, LOG_BUFFER) {
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
        let log_layer = log_writer.as_ref().map(|writer| {
            JsonLogLayer::with_filter(Arc::clone(writer), json_filter_mode)
                .with_filter(json_log_filter.clone())
        });
        let resources_layer = resources_writer.as_ref().map(|writer| {
            JsonLogLayer::with_filter(Arc::clone(writer), JsonLogFilter::ResourcesOnly)
                .with_filter(json_log_filter)
        });
        let registry = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(tui_layer)
            .with(log_layer)
            .with(resources_layer);
        registry.with(chrome_layer.with_filter(trace_filter)).init();
        chrome_guard = Some(guard);
    } else {
        let log_layer = log_writer.as_ref().map(|writer| {
            JsonLogLayer::with_filter(Arc::clone(writer), json_filter_mode)
                .with_filter(json_log_filter.clone())
        });
        let resources_layer = resources_writer.as_ref().map(|writer| {
            JsonLogLayer::with_filter(Arc::clone(writer), JsonLogFilter::ResourcesOnly)
                .with_filter(json_log_filter)
        });
        let registry = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(tui_layer)
            .with(log_layer)
            .with(resources_layer);
        registry.init();
    }
    TracingGuards {
        chrome_guard,
        log_writer,
        resources_writer,
        tui_log_buffer,
    }
}
