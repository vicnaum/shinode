use crate::chain::ChainKind;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent CLI switches for inspection, offline inputs, retention and follow"
)]
pub struct SnapshotArgs {
    /// Snapshot network (currently Base mainnet only).
    #[arg(long, value_enum, default_value = "base")]
    pub chain: ChainKind,
    /// Local V2 manifest; pin its SHA-256 independently.
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub manifest_sha256: String,
    #[arg(long)]
    pub start_block: u64,
    #[arg(
        long,
        required = false,
        required_unless_present = "auto_handoff",
        default_value_if("auto_handoff", "true", "0")
    )]
    pub end_block: u64,
    /// Choose the archive end from actual peer history, then sync history and follow.
    #[arg(long, requires = "checkpoint_rpc", conflicts_with_all = ["end_block", "checkpoint_hash", "offline", "dry_run"])]
    pub auto_handoff: bool,
    /// Independent RPC used to authenticate the automatically chosen archive boundary.
    #[arg(long, requires = "auto_handoff")]
    pub checkpoint_rpc: Option<String>,
    /// Blocks of archive/P2P overlap to allow for a moving peer history window.
    #[arg(long, default_value_t = 43_200)]
    pub handoff_overlap_blocks: u64,
    /// Private resumable input staging directory, separate from node storage.
    #[arg(long, required_unless_present = "dry_run")]
    pub staging_dir: Option<PathBuf>,
    /// Node data directory; matching Base history is reused and gaps are filled.
    #[arg(long, required_unless_present = "dry_run")]
    pub output_dir: Option<PathBuf>,
    /// Inspect dependencies and resource requirements without writing or downloading.
    #[arg(long)]
    pub dry_run: bool,
    /// Use verified staged chunks only; make no archive requests.
    #[arg(long)]
    pub offline: bool,
    /// Retain importer-created input files after verified durable completion.
    #[arg(long)]
    pub keep_scratch: bool,
    /// Independently verified hash of --end-block; pins the P2P handoff boundary.
    #[arg(long)]
    pub checkpoint_hash: Option<alloy_primitives::B256>,
    /// Start P2P sync/follow after import and boundary verification.
    #[arg(long, requires = "checkpoint_hash", conflicts_with = "dry_run")]
    pub follow: bool,
    #[arg(long, default_value_t = 536_870_912)]
    pub max_download_bytes: u64,
    #[arg(long, default_value_t = 1_073_741_824)]
    pub max_extracted_bytes: u64,
    #[arg(long, default_value_t = 5_368_709_120)]
    pub min_free_bytes: u64,
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=4))]
    pub concurrency: u8,
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..=10))]
    pub retries: u8,
    /// Whole HTTP attempt timeout, including response reads.
    #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u64).range(1..))]
    pub timeout_seconds: u64,
    #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=1024))]
    pub batch_blocks: u16,
    #[arg(long, default_value_t = 10_000, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_transactions_per_block: u32,
}
