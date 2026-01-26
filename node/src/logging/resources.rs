//! Resource monitoring for CPU, memory, and disk I/O.

use crate::p2p::PeerPool;
use crate::sync::historical::{BenchEvent, BenchEventLogger, PeerHealthTracker};
use crate::sync::SyncProgressStats;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// ============================================================================
// Linux-specific resource monitoring
// ============================================================================

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
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
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
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
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
    let stats = std::fs::read_to_string("/proc/diskstats").ok()?;
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
pub fn spawn_resource_logger(
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
                        escalation: Some(snapshot.escalation),
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
                    escalation = snapshot.escalation,
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
                        escalation: None,
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

// ============================================================================
// Non-Linux resource monitoring (uses sysinfo crate)
// ============================================================================

#[cfg(not(target_os = "linux"))]
pub fn spawn_resource_logger(
    stats: Option<Arc<SyncProgressStats>>,
    events: Option<Arc<BenchEventLogger>>,
) {
    use sysinfo::{Disks, Pid, ProcessesToUpdate, System};

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        let pid = Pid::from_u32(std::process::id());
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
                        escalation: Some(snapshot.escalation),
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
                    escalation = snapshot.escalation,
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
                        escalation: None,
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

// ============================================================================
// SIGUSR1 handler (Unix only)
// ============================================================================

#[cfg(unix)]
pub fn spawn_usr1_state_logger(
    stats: Option<Arc<SyncProgressStats>>,
    peer_pool: Option<Arc<PeerPool>>,
    peer_health: Option<Arc<PeerHealthTracker>>,
) {
    use tokio::signal::unix::{signal, SignalKind};

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
                    escalation = snapshot.escalation,
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
