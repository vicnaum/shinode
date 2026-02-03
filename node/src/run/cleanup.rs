//! Unified cleanup and finalization logic.
//!
//! This module consolidates the duplicated cleanup paths from main.rs into a single
//! `finalize_session` function.

use crate::cli::NodeConfig;
use crate::logging::{finalize_log_files, generate_run_report, ResourcesLogger, RunContext, TracingGuards};
use crate::p2p;
use crate::storage::Storage;
use crate::sync::historical::{BenchEventLogger, IngestBenchStats, PeerHealthTracker, SummaryInput};
use reth_network_api::PeerId;
use std::ops::RangeInclusive;
use std::str::FromStr;
use tracing::warn;

/// All data needed for report generation and log finalization.
pub struct FinalizeContext<'a> {
    pub run_context: Option<&'a RunContext>,
    pub config: &'a NodeConfig,
    pub range: &'a RangeInclusive<u64>,
    pub head_at_startup: u64,
    pub peer_health: &'a PeerHealthTracker,
    pub network: &'a p2p::NetworkSession,
    pub storage: &'a Storage,
    pub bench: &'a IngestBenchStats,
    pub events: Option<&'a BenchEventLogger>,
    pub resources_logger: Option<&'a ResourcesLogger>,
    pub tracing_guards: &'a mut TracingGuards,
}

/// Single function replacing 3 duplicated cleanup paths.
///
/// This generates the run report (if enabled), finalizes log files, and flushes the peer cache.
#[expect(clippy::cognitive_complexity, reason = "cleanup orchestrates report, logs, and cache")]
pub async fn finalize_session(ctx: FinalizeContext<'_>, logs_total: u64) {
    // Build summary from bench stats
    let storage_stats = match ctx.storage.disk_usage() {
        Ok(stats) => Some(stats),
        Err(err) => {
            warn!(error = %err, "failed to collect storage disk stats");
            None
        }
    };

    let summary = ctx.bench.summary(SummaryInput {
        range_start: *ctx.range.start(),
        range_end: *ctx.range.end(),
        head_at_startup: ctx.head_at_startup,
        rollback_window_applied: ctx.config.rollback_window > 0,
        peers_used: ctx.network.pool.len() as u64,
        logs_total,
        storage_stats,
    });

    // Generate report if run_context exists
    let base_name = if let Some(run_ctx) = ctx.run_context {
        match generate_run_report(
            run_ctx,
            ctx.config,
            ctx.range,
            ctx.head_at_startup,
            ctx.peer_health,
            &summary,
            ctx.network.pool.len() as u64,
        )
        .await
        {
            Ok(name) => name,
            Err(err) => {
                warn!(error = %err, "failed to generate run report");
                // Fallback base name
                run_ctx.timestamp_utc.clone()
            }
        }
    } else {
        String::new()
    };

    // Finalize log files
    if let Some(run_ctx) = ctx.run_context {
        finalize_log_files(
            run_ctx,
            &base_name,
            ctx.events,
            ctx.tracing_guards.log_writer.as_deref(),
            ctx.resources_logger,
            &mut ctx.tracing_guards.chrome_guard,
        );
    }

    // Flush peer cache with limits
    flush_peer_cache_with_limits(ctx.network, ctx.storage, Some(ctx.peer_health)).await;
}

/// Apply cached peer batch limits from storage to peer health tracker.
pub async fn apply_cached_peer_limits(storage: &Storage, peer_health: &PeerHealthTracker) {
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

/// Persist peer batch limits to storage.
async fn persist_peer_limits(storage: &Storage, peer_health: &PeerHealthTracker) {
    let snapshot = peer_health.snapshot().await;
    let limits: Vec<(String, u64)> = snapshot
        .into_iter()
        .map(|entry| (entry.peer_id.to_string(), entry.batch_limit as u64))
        .collect();
    storage.update_peer_batch_limits(&limits);
}

/// Flush peer cache with batch limits.
#[expect(clippy::cognitive_complexity, reason = "cache flush with health snapshot")]
pub async fn flush_peer_cache_with_limits(
    session: &p2p::NetworkSession,
    storage: &Storage,
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
