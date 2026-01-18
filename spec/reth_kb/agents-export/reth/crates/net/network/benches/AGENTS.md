# benches

## Purpose
Criterion benchmarks for network transaction propagation and fetch behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `broadcast.rs`
- **Role**: Benchmarks broadcast ingress by sending transactions between two testnet peers and measuring receive throughput.
- **Key items**: `broadcast_ingress_bench()`, `Testnet::create_with()`, `TransactionGenerator`, `send_transactions()`
- **Interactions**: Uses `Testnet` from `test_utils` and transaction pool generators to simulate broadcasts.

### `tx_manager_hash_fetching.rs`
- **Role**: Benchmarks transaction hash fetching and pending-hash processing at various peer counts.
- **Key items**: `benchmark_fetch_pending_hashes()`, `fetch_pending_hashes()`, `tx_fetch_bench()`, `TransactionFetcher`, `TransactionsManagerConfig`
- **Interactions**: Uses `TransactionFetcher` and `Testnet` harness; leverages helpers from `test_utils::transactions`.

## End-to-end flow (high level)
- Build a `Testnet` with mock providers and transaction pools.
- For broadcast bench, send transactions between peers and measure receive loop throughput.
- For fetch benches, populate the fetcher with pending hashes and measure time to schedule requests.

## Key APIs (no snippets)
- `broadcast_ingress_bench()`
- `benchmark_fetch_pending_hashes()`, `tx_fetch_bench()`
