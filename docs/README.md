# Documentation

Stateless History Node is a minimal Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset.

## Quick Links

- [Getting Started](getting-started.md) - Installation and first run
- [Configuration](configuration.md) - Full CLI options reference
- [Architecture](../ARCHITECTURE.md) - System overview and module structure
- [Specification](../SPEC.md) - Detailed system specification
- [Roadmap](../ROADMAP.md) - Current and planned features

## Key Concepts

### Stateless by Design
The node does not execute transactions or maintain state. It ingests history artifacts (headers, bodies, receipts) from the P2P network and derives logs on demand. This keeps storage requirements minimal.

### Sharded Storage
Data is stored in fixed-size shards (default: 10,000 blocks per shard). Each shard contains compressed segments for headers, transaction metadata, and receipts. Out-of-order ingestion is supported via a write-ahead log (WAL).

### Indexer-Compatible RPC
The node exposes a minimal JSON-RPC subset designed for event indexers like rindexer:
- `eth_chainId` - Chain ID
- `eth_blockNumber` - Highest present block
- `eth_getBlockByNumber` - Block by number
- `eth_getLogs` - Filtered event logs

### Sync Modes
1. **Fast-Sync** - Concurrent fetching from multiple peers for historical blocks
2. **Follow Mode** - Sequential fetching near chain head with reorg handling

## Support

For issues and feature requests, see the [GitHub repository](https://github.com/vicnaum/stateless-history-node).
