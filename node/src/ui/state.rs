//! UI state machine types.

use crate::sync::{FinalizePhase, SyncStatus};

/// High-level UI state representing the current phase of operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIState {
    /// Startup phase - initializing storage, connecting to peers, etc.
    /// The String is the status message to display.
    Startup,
    /// Actively syncing blocks from peers.
    Syncing,
    /// Compacting shards (during finalize).
    Compacting,
    /// Sealing completed shards (during finalize).
    Sealing,
    /// Synced and following the chain head.
    Following,
}

impl UIState {
    /// Derive UIState from SyncProgressSnapshot.
    pub fn from_sync_snapshot(
        status: SyncStatus,
        finalize_phase: FinalizePhase,
        fetch_complete: bool,
        processed: u64,
        total_blocks: u64,
    ) -> Self {
        match status {
            SyncStatus::LookingForPeers => UIState::Startup,
            SyncStatus::Fetching => {
                // Still syncing if we haven't reached target
                if processed < total_blocks && !fetch_complete {
                    UIState::Syncing
                } else {
                    UIState::Following
                }
            }
            SyncStatus::Finalizing => match finalize_phase {
                FinalizePhase::Compacting => UIState::Compacting,
                FinalizePhase::Sealing => UIState::Sealing,
            },
            SyncStatus::UpToDate | SyncStatus::Following => UIState::Following,
        }
    }
}
