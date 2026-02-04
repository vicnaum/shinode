//! Subcommand handlers for db stats and repair.

// CLI commands output directly to stdout for user feedback
#![expect(clippy::print_stdout, reason = "CLI commands require stdout output")]

use crate::cli::{DbCompactArgs, DbStatsArgs, NodeConfig};
use crate::logging::{JsonLogFilter, JsonLogLayer, JsonLogWriter, LOG_BUFFER};
use crate::storage::Storage;
use crate::ui;
use eyre::Result;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Handle the `db stats` subcommand.
pub fn handle_db_stats(args: &DbStatsArgs, config: &NodeConfig) -> Result<()> {
    let data_dir = args
        .data_dir
        .clone()
        .unwrap_or_else(|| config.data_dir.clone());
    let stats = Storage::disk_usage_at(&data_dir)?;
    ui::print_db_stats(&data_dir, &stats, args.json)?;
    Ok(())
}

/// Handle the `db compact` subcommand.
pub fn handle_db_compact(args: &DbCompactArgs, config: &NodeConfig) -> Result<()> {
    // Initialize tracing if --log-json is specified
    let _log_writer = init_compact_tracing(args);

    let mut config = config.clone();
    if let Some(dir) = &args.data_dir {
        config.data_dir = dir.clone();
    }

    // Auto-detect shard size from existing storage
    if let Some(stored_shard_size) = Storage::read_shard_size(&config.data_dir)? {
        config.shard_size = stored_shard_size;
    }

    info!(data_dir = %config.data_dir.display(), shard_size = config.shard_size, "starting db compact");

    let storage = Storage::open_with_progress(&config, |current, total| {
        if total > 0 {
            print!("\rOpening storage... {current}/{total} shards");
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    })?;
    println!(); // newline after progress

    let pre = storage.aggregate_stats();
    let dirty = storage.dirty_shards()?;
    info!(
        total_shards = pre.total_shards,
        dirty_count = dirty.len(),
        compacted_count = pre.compacted_shards,
        "storage opened"
    );
    println!(
        "{} total shard(s), {} dirty, {} already compacted.\n",
        pre.total_shards,
        dirty.len(),
        pre.compacted_shards
    );

    if dirty.is_empty() {
        println!("Nothing to do.");
        return Ok(());
    }

    let mut compacted = 0u64;
    storage.compact_all_dirty_with_progress(|shard_start| {
        compacted += 1;
        println!("  Compacted shard {shard_start} ({compacted}/{})", dirty.len());
    })?;

    let to_seal = storage.count_shards_to_seal()?;
    if to_seal > 0 {
        println!("\n{to_seal} shard(s) to seal.\n");
    }
    let mut sealed = 0u64;
    storage.seal_completed_shards_with_progress(|shard_start| {
        sealed += 1;
        println!("  Sealed shard {shard_start} ({sealed}/{to_seal})");
    })?;

    let post = storage.aggregate_stats();
    info!(
        compacted_count = compacted,
        sealed_count = sealed,
        total_blocks = post.total_blocks,
        total_txs = post.total_transactions,
        total_logs = post.total_logs,
        disk_bytes = post.disk_bytes_total,
        "db compact complete"
    );
    println!("\nDone. Compacted {compacted} shard(s), sealed {sealed} shard(s).");
    println!(
        "Storage: {} blocks, {} txs, {} logs, {}",
        post.total_blocks,
        post.total_transactions,
        post.total_logs,
        ui::human_bytes(post.disk_bytes_total)
    );
    Ok(())
}

/// Initialize tracing for db compact command.
/// Returns the log writer guard (must be kept alive for logging to work).
fn init_compact_tracing(args: &DbCompactArgs) -> Option<Arc<JsonLogWriter>> {
    // Determine log level from verbosity
    let (global, local) = match args.verbose {
        0 => ("warn", "info"),
        1 => ("warn", "debug"),
        2 => ("info", "trace"),
        _ => ("debug", "trace"),
    };
    let log_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(format!("{global},stateless_history_node={local}")));

    // Create JSON log writer if enabled
    let log_writer = if args.log_json {
        // Generate timestamped filename in logs directory
        let timestamp = crate::logging::run_timestamp_utc(std::time::SystemTime::now());
        let log_path = args.log_output_dir.join(format!("compact-{timestamp}.logs.jsonl"));

        // Create output directory if needed
        if let Err(err) = std::fs::create_dir_all(&args.log_output_dir) {
            eprintln!("Warning: failed to create log directory: {err}");
            None
        } else {
            match JsonLogWriter::new(&log_path, LOG_BUFFER) {
                Ok(writer) => {
                    println!("Writing logs to: {}", log_path.display());
                    Some(Arc::new(writer))
                }
                Err(err) => {
                    eprintln!("Warning: failed to initialize log writer: {err}");
                    None
                }
            }
        }
    } else {
        None
    };

    // Build tracing subscriber
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(log_filter);

    let json_filter = EnvFilter::new("warn,stateless_history_node=debug");
    let log_layer = log_writer.as_ref().map(|writer| {
        JsonLogLayer::with_filter(Arc::clone(writer), JsonLogFilter::All).with_filter(json_filter)
    });

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(log_layer)
        .init();

    log_writer
}

/// Handle the `--repair` flag.
pub fn handle_repair(config: &NodeConfig) -> Result<()> {
    println!("Checking storage integrity...\n");
    let report = Storage::repair(config)?;

    if report.shards.is_empty() {
        println!("No shards found in {}", config.data_dir.display());
        return Ok(());
    }

    for info in &report.shards {
        if info.result.needs_recovery() {
            let phase_info = info
                .original_phase
                .as_ref()
                .map(|p| format!(" (phase: {p})"))
                .unwrap_or_default();
            println!(
                "Shard {}: {}{}",
                info.shard_start,
                info.result.description(),
                phase_info
            );
        } else {
            println!("Shard {}: OK", info.shard_start);
        }
    }

    println!(
        "\nRepair complete. {}/{} shards repaired, {} already clean.",
        report.repaired_count(),
        report.total_count(),
        report.clean_count()
    );
    Ok(())
}
