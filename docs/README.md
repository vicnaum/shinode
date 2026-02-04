# Documentation

Stateless History Node is a minimal Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset.

## Quick Links

- [Getting Started](getting-started.md) - Installation and first run
- [Configuration](configuration.md) - Full CLI options reference
- [Architecture](architecture.md) - System overview, design, and data model
- [Roadmap](../ROADMAP.md) - Current and planned features
- [UI Designs](UI_DESIGNS.md) - TUI dashboard design reference
- [Spec Archive](spec/) - Research docs and development history

## Key Concepts

### Stateless by Design
The node does not execute transactions or maintain state. It ingests history artifacts (headers, bodies, receipts) from the P2P network and derives logs on demand. This keeps storage requirements minimal.

### Sharded Storage
Data is stored in fixed-size shards (default: 10,000 blocks per shard). Each shard contains compressed segments for headers, transaction metadata, and receipts. Out-of-order ingestion is supported via a write-ahead log (WAL). Per-shard compaction triggers as shards fill during sync.

`ShardMeta` tracks per-shard statistics: total transactions, receipts, logs, and per-segment disk bytes. `StorageAggregateStats` provides cheap cross-shard rollup.

### Indexer-Compatible RPC
The node exposes a minimal JSON-RPC subset designed for event indexers like rindexer:
- `eth_chainId` - Chain ID
- `eth_blockNumber` - Highest present block
- `eth_getBlockByNumber` - Block by number
- `eth_getLogs` - Filtered event logs

RPC request counters are tracked and displayed in the TUI dashboard during follow mode.

### Sync Modes
1. **Fast-Sync** - Concurrent fetching from multiple peers for historical blocks, with per-shard compaction
2. **Follow Mode** - Sequential fetching near chain head with reorg handling

### TUI Dashboard
The default UI is a fullscreen terminal dashboard (powered by ratatui) with real-time visualization of sync progress, speed charts, peer status, storage statistics, and log viewing. Disable with `--no-tui` for headless environments.

## Support

For issues and feature requests, see the [GitHub repository](https://github.com/vicnaum/stateless-history-node).
