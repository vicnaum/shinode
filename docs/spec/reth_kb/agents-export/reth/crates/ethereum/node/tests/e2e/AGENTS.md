# e2e

## Purpose
End-to-end tests for `reth-node-ethereum` using `reth-e2e-test-utils`: spin up real in-process nodes (often with Engine API) and validate Ethereum-specific behaviours across blobs, RPC, P2P sync, reorgs, txpool maintenance, and debug tracing.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test module root that composes the `e2e` test suite.
- `utils.rs` - shared helpers (payload attributes builder, random-tx chain advancement, and related utility helpers).
- `eth.rs` - "node can run" tests: basic tx inclusion, Engine API over IPC variants, graceful engine shutdown persistence, and other core ETH-node flows.
- `blobs.rs` - blob-transaction lifecycle tests (pool injection, payload building/submission, sidecar presence/conversion, and reorg behaviour restoring txs).
- `custom_genesis.rs` - tests for custom genesis parameters (genesis block number boundaries and stage checkpoint initialization).
- `dev.rs` - dev-mode node tests (dev chain config, debug capabilities, payload attribute mutation).
- `p2p.rs` - P2P sync/reorg end-to-end tests (gossip sync, explicit sync-to-head, long reorg and backfill-related reorg scenarios).
- `pool.rs` - txpool maintenance behaviour tests (stale eviction and handling canonical reorg notifications).
- `prestate.rs` - debug tracing regression test: `debug_traceCall` prestate is compared against a captured Geth snapshot.
- `rpc.rs` - RPC surface end-to-end tests (fee history correctness and Flashbots builder submission validation endpoints).

## Relationships
- **Uses**: `reth-e2e-test-utils` to provision nodes and helpers, plus `alloy` RPC providers to drive RPC/Engine API calls.
