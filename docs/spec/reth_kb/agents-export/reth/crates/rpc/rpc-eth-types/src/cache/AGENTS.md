# cache

## Purpose
Async caching layer for `eth` RPC data (blocks, receipts, headers).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Cache service frontend and task wiring.
- **Key items**: `EthStateCache`, `EthStateCacheConfig`, `CacheAction`
- **Interactions**: Spawns cache service tasks and handles requests via channels.

### `config.rs`
- **Role**: Cache size and concurrency configuration.
- **Key items**: `EthStateCacheConfig`, `max_blocks`, `max_receipts`, `max_headers`

### `db.rs`
- **Role**: Helper types for state provider trait objects.
- **Key items**: `StateCacheDb`, `StateProviderTraitObjWrapper`
- **Knobs / invariants**: Wraps trait objects to avoid HRTB lifetime issues.

### `metrics.rs`
- **Role**: Metrics for cache hit/miss and memory usage.
- **Key items**: `CacheMetrics`

### `multi_consumer.rs`
- **Role**: LRU cache with queued consumers on miss.
- **Key items**: `MultiConsumerLruCache`, `queue()`, `insert()`, `update_cached_metrics()`

## End-to-end flow (high level)
- Create `EthStateCache` with size and concurrency limits.
- Cache service fetches blocks/receipts/headers from provider on misses.
- LRU and queued consumers coordinate concurrent requests.
- Metrics report cache hits/misses and memory usage.

## Key APIs (no snippets)
- `EthStateCache`, `EthStateCacheConfig`, `MultiConsumerLruCache`
