//! Resource monitoring for CPU, memory, and disk I/O.

use crate::p2p::{P2pStats, PeerPool};
use crate::sync::historical::PeerHealthTracker;
use crate::sync::SyncProgressStats;
use parking_lot::Mutex;
use reth_eth_wire::EthNetworkPrimitives;
use reth_network::NetworkHandle;
use reth_network::PeersInfo;
use serde::Serialize;
use std::fs;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// ============================================================================
// Resources logger (writes compact JSON directly to file)
// ============================================================================

/// A single resource sample with all metrics.
#[derive(Debug, Clone, Serialize)]
pub struct ResourcesSample {
    pub t_ms: u64,
    pub rss_kb: u64,
    pub swap_kb: u64,
    pub rss_anon_kb: u64,
    pub rss_file_kb: u64,
    pub rss_shmem_kb: u64,
    pub cpu_busy_pct: f64,
    pub cpu_iowait_pct: f64,
    pub disk_read_mib_s: f64,
    pub disk_write_mib_s: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inflight: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers_active: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_seen: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reth_connected: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_mismatches: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions_established: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions_closed: Option<usize>,
}

/// Logger that writes resource samples to a JSONL file in compact format.
#[derive(Debug)]
pub struct ResourcesLogger {
    started_at: Instant,
    sender: Mutex<Option<mpsc::Sender<ResourcesSample>>>,
    handle: Mutex<Option<JoinHandle<eyre::Result<()>>>>,
    dropped_samples: AtomicU64,
    total_samples: AtomicU64,
}

impl ResourcesLogger {
    /// Create a new resources logger that writes to the given path.
    pub fn new(path: &std::path::Path) -> eyre::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        let (tx, rx) = mpsc::channel::<ResourcesSample>();
        let handle = std::thread::spawn(move || -> eyre::Result<()> {
            let mut since_flush = 0usize;
            for sample in rx {
                serde_json::to_writer(&mut writer, &sample)?;
                writer.write_all(b"\n")?;
                since_flush = since_flush.saturating_add(1);
                if since_flush >= 64 {
                    // Flush more frequently for resources (1 sample/sec)
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
            dropped_samples: AtomicU64::new(0),
            total_samples: AtomicU64::new(0),
        })
    }

    /// Record a resource sample (adds timestamp automatically).
    pub fn record(&self, mut sample: ResourcesSample) {
        sample.t_ms = self.started_at.elapsed().as_millis() as u64;
        let sender = self.sender.lock().as_ref().cloned();
        if let Some(sender) = sender {
            match sender.send(sample) {
                Ok(()) => {
                    self.total_samples.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    self.dropped_samples.fetch_add(1, Ordering::SeqCst);
                }
            }
        } else {
            self.dropped_samples.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Finish writing and close the file.
    pub fn finish(&self) -> eyre::Result<()> {
        let sender = self.sender.lock().take();
        drop(sender);

        let handle = self.handle.lock().take();
        if let Some(handle) = handle {
            match handle.join() {
                Ok(res) => res?,
                Err(_) => return Err(eyre::eyre!("resources writer thread panicked")),
            }
        }
        Ok(())
    }
}

impl Drop for ResourcesLogger {
    fn drop(&mut self) {
        let sender = self.sender.lock().take();
        drop(sender);
        let handle = self.handle.lock().take();
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}

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
    resources_logger: Option<Arc<ResourcesLogger>>,
    network: Option<NetworkHandle<EthNetworkPrimitives>>,
    p2p_stats: Option<Arc<P2pStats>>,
) {
    // Skip spawning if no logger is provided
    let Some(logger) = resources_logger else {
        return;
    };

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

            let reth_connected = network.as_ref().map(|n| n.num_connected_peers());
            let discovered = p2p_stats.as_ref().map(|s| s.discovered_count.load(Ordering::Relaxed));
            let genesis_mismatches = p2p_stats.as_ref().map(|s| s.genesis_mismatch_count.load(Ordering::Relaxed));
            let sessions_established = p2p_stats.as_ref().map(|s| s.sessions_established.load(Ordering::Relaxed));
            let sessions_closed = p2p_stats.as_ref().map(|s| s.sessions_closed.load(Ordering::Relaxed));

            let sample = if let Some(stats) = stats.as_ref() {
                let snapshot = stats.snapshot();
                ResourcesSample {
                    t_ms: 0, // Will be set by logger
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
                    reth_connected,
                    discovered,
                    genesis_mismatches,
                    sessions_established,
                    sessions_closed,
                }
            } else {
                ResourcesSample {
                    t_ms: 0,
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
                    reth_connected,
                    discovered,
                    genesis_mismatches,
                    sessions_established,
                    sessions_closed,
                }
            };
            logger.record(sample);
        }
    });
}

// ============================================================================
// Non-Linux resource monitoring (uses sysinfo crate)
// ============================================================================

#[cfg(not(target_os = "linux"))]
#[expect(
    clippy::too_many_lines,
    reason = "resource monitoring loop with OS-specific metric collection is clearer inline"
)]
pub fn spawn_resource_logger(
    stats: Option<Arc<SyncProgressStats>>,
    resources_logger: Option<Arc<ResourcesLogger>>,
    network: Option<NetworkHandle<EthNetworkPrimitives>>,
    p2p_stats: Option<Arc<P2pStats>>,
) {
    use sysinfo::{Disks, Pid, ProcessesToUpdate, System};

    // Skip spawning if no logger is provided
    let Some(logger) = resources_logger else {
        return;
    };

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

            let Some(proc) = sys.process(pid) else {
                continue;
            };

            let rss_kb = proc.memory().saturating_div(1024);
            let cpu_count = sys.cpus().len().max(1) as f64;
            let cpu_busy_pct = f64::from(sys.global_cpu_usage()) / cpu_count;
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

            let reth_connected = network.as_ref().map(|n| n.num_connected_peers());
            let discovered = p2p_stats.as_ref().map(|s| s.discovered_count.load(Ordering::Relaxed));
            let genesis_mismatches = p2p_stats.as_ref().map(|s| s.genesis_mismatch_count.load(Ordering::Relaxed));
            let sessions_established = p2p_stats.as_ref().map(|s| s.sessions_established.load(Ordering::Relaxed));
            let sessions_closed = p2p_stats.as_ref().map(|s| s.sessions_closed.load(Ordering::Relaxed));

            let sample = if let Some(stats) = stats.as_ref() {
                let snapshot = stats.snapshot();
                ResourcesSample {
                    t_ms: 0, // Will be set by logger
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
                    reth_connected,
                    discovered,
                    genesis_mismatches,
                    sessions_established,
                    sessions_closed,
                }
            } else {
                ResourcesSample {
                    t_ms: 0,
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
                    reth_connected,
                    discovered,
                    genesis_mismatches,
                    sessions_established,
                    sessions_closed,
                }
            };
            logger.record(sample);
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
                let cooling_down = health.count_cooling_down_peers(&peer_ids).await;
                info!(
                    connected = peers.len(),
                    cooling_down,
                    available = peers.len().saturating_sub(cooling_down as usize),
                    "SIGUSR1: peer pool"
                );

                let dump = health.snapshot().await;
                let total = dump.len();
                let banned_total = dump.iter().filter(|p| p.is_cooling_down).count();
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
                        cooling_down = entry.is_cooling_down,
                        backoff_remaining_ms = entry.backoff_remaining_ms,
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
                        cooling_down = entry.is_cooling_down,
                        backoff_remaining_ms = entry.backoff_remaining_ms,
                        last_error = entry.last_error.as_deref().unwrap_or(""),
                        "SIGUSR1: peer health (worst)"
                    );
                }
            }
        }
    });
}
