//! Subcommand handlers for db stats and repair.

// CLI commands output directly to stdout for user feedback
#![expect(clippy::print_stdout, reason = "CLI commands require stdout output")]

use crate::cli::{DbStatsArgs, NodeConfig};
use crate::storage::Storage;
use crate::ui;
use eyre::Result;

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
