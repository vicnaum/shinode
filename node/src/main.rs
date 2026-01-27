mod cli;
mod logging;
mod metrics;
mod p2p;
mod rpc;
mod run;
mod storage;
mod sync;
mod ui;
#[cfg(test)]
mod test_utils;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use cli::NodeConfig;
use eyre::Result;
use indicatif::ProgressBar;
use std::env;
#[cfg(test)]
use sync::SyncStatus;

impl sync::ProgressReporter for ProgressBar {
    fn set_length(&self, len: u64) {
        self.set_length(len);
    }

    fn inc(&self, delta: u64) {
        self.inc(delta);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = NodeConfig::from_args();
    let argv: Vec<String> = env::args().collect();

    // Handle subcommands
    if let Some(command) = &config.command {
        match command {
            cli::Command::Db(cli::DbCommand::Stats(args)) => {
                return run::handle_db_stats(args, &config);
            }
        }
    }

    // Handle repair mode
    if config.repair {
        return run::handle_repair(&config);
    }

    // Run main sync
    run::run_sync(config, argv).await
}

#[cfg(test)]
fn total_blocks_to_head(start_from: u64, head: u64) -> u64 {
    if head >= start_from {
        head.saturating_sub(start_from).saturating_add(1)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::total_blocks_to_head;

    #[test]
    fn total_blocks_handles_empty_range() {
        assert_eq!(total_blocks_to_head(10, 9), 0);
        assert_eq!(total_blocks_to_head(10, 10), 1);
        assert_eq!(total_blocks_to_head(10, 12), 3);
    }
}
