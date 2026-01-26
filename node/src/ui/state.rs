//! UI state machine types.

use crate::sync::SyncStatus;

/// High-level UI state representing the current phase of operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIState {
    /// Startup phase - initializing storage, connecting to peers, etc.
    Startup(StartupPhase),
    /// Actively syncing blocks from peers.
    Syncing,
    /// Fetch complete, finalizing DB (flushing WAL, compacting shards).
    Finalizing,
    /// Synced and following the chain head.
    Following,
}

/// Sub-states during the startup phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupPhase {
    /// Opening the storage database.
    OpeningStorage,
    /// Recovering a shard from interrupted compaction.
    RecoveringShard { shard: u64, phase: String },
    /// Compacting dirty shards from previous run.
    CompactingShards(usize),
    /// Connecting to the P2P network.
    ConnectingP2P,
    /// Waiting for minimum peer count.
    WaitingForPeers { current: usize, min: usize },
    /// Discovering the chain head from peers.
    DiscoveringHead(u64),
}

impl UIState {
    /// Derive UIState from SyncStatus and other signals.
    pub fn from_sync_status(status: SyncStatus, fetch_complete: bool) -> Self {
        match status {
            SyncStatus::LookingForPeers => UIState::Startup(StartupPhase::WaitingForPeers {
                current: 0,
                min: 0,
            }),
            SyncStatus::Fetching => UIState::Syncing,
            SyncStatus::Finalizing => UIState::Finalizing,
            SyncStatus::UpToDate | SyncStatus::Following => {
                if fetch_complete {
                    UIState::Following
                } else {
                    UIState::Syncing
                }
            }
        }
    }
}

impl StartupPhase {
    /// Get the display message for this startup phase.
    pub fn message(&self) -> String {
        match self {
            StartupPhase::OpeningStorage => "Opening storage...".to_string(),
            StartupPhase::RecoveringShard { shard, phase } => {
                format!("Recovering shard {}: {}", shard, phase)
            }
            StartupPhase::CompactingShards(count) => format!("Compacting {} shards...", count),
            StartupPhase::ConnectingP2P => "Connecting to P2P network...".to_string(),
            StartupPhase::WaitingForPeers { current, min } => {
                format!("Waiting for peers... {}/{}", current, min)
            }
            StartupPhase::DiscoveringHead(head) => format!("Discovering chain head... {}", head),
        }
    }

    /// Returns true if this phase should be displayed with orange (recovery) color.
    pub fn is_recovery(&self) -> bool {
        matches!(self, StartupPhase::RecoveringShard { .. })
    }
}
