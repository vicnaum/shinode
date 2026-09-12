use alloy_primitives::B256;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct ExportArgs {
    /// New local directory for manifest.json and compressed sealed shards.
    #[arg(long)]
    pub output_dir: PathBuf,
    /// First block; must start a complete shard.
    #[arg(long)]
    pub start_block: u64,
    /// Last block; must end a complete shard.
    #[arg(long)]
    pub end_block: u64,
    /// Independent end-block hash; defaults to the saved boundary at this height.
    #[arg(long)]
    pub checkpoint_hash: Option<B256>,
    #[arg(long, default_value_t = 1_073_741_824)]
    pub max_extracted_bytes: u64,
    #[arg(long, default_value_t = 5_368_709_120)]
    pub min_free_bytes: u64,
}

#[derive(Args, Debug, Clone)]
pub struct RestoreArgs {
    /// Local distribution manifest; shard archives must be beside it.
    #[arg(long)]
    pub manifest: PathBuf,
    /// SHA-256 of the manifest from a trusted publisher.
    #[arg(long)]
    pub manifest_sha256: String,
    /// Independently verified hash of the manifest's end block.
    #[arg(long)]
    pub checkpoint_hash: B256,
    /// Fresh or matching Base history directory.
    #[arg(long)]
    pub output_dir: PathBuf,
    #[arg(long, default_value_t = 536_870_912)]
    pub max_archive_bytes: u64,
    #[arg(long, default_value_t = 1_073_741_824)]
    pub max_extracted_bytes: u64,
    #[arg(long, default_value_t = 5_368_709_120)]
    pub min_free_bytes: u64,
    #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=1024))]
    pub batch_blocks: u16,
}
