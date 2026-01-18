# crates

## Purpose
Reth's Rust workspace crates: domain-grouped libraries and binaries that make up the node (networking, engine, storage, RPC, sync stages, primitives, etc.).

## Contents (one hop)
### Subdirectories
- [x] `chain-state/` - Canonical-chain in-memory state tracking, forkchoice head tracking (head/safe/finalized), and notification streams for consumers like engine/RPC.
- [x] `chainspec/` - Ethereum network spec definitions: genesis inputs, hardfork schedule, and chain-spec query traits/presets used across reth.
- [x] `cli/` - CLI subsystem crates: shared CLI traits/parsers, a tokio runner, reusable parsing/runtime utilities, and the concrete `reth ...` command suite (node/db/stage/import/export/etc.).
- [x] `config/` - Reth configuration schema (`Config`): stage/pipeline knobs, pruning/static-file settings, and peer/session config (with optional TOML load/save via `serde` feature).
- [x] `consensus/` - Consensus/validation crates: core consensus traits+errors, shared Ethereum block/header validation helpers, and a debug client for driving the engine API from external block sources.
- [x] `e2e-test-utils/` - E2E testing framework and helpers: spin up in-process nodes, drive them via RPC/Engine API, and express tests as composable "actions" (produce blocks, forks/reorgs, sync assertions).
- [x] `engine/` - Engine subsystem crates: engine-tree, engine service wiring, engine API primitives/config, and testing/debug utilities.
- [x] `era/` - E2Store-based history file formats: core read/write and compression logic for `.era` (CL) and `.era1` (EL) files, including indices and standardized naming.
- [x] `era-downloader/` - Async ERA file downloader/streamer: fetches index/checksums (for `.era1`), downloads files with concurrency + retention limits, and supports local `.era1` directory streaming with checksum validation.
- [x] `era-utils/` - ERA storage utilities: import `.era1` history into reth storage (static files/DB/checkpoints) and export stored history back into `.era1` files with configurable chunking/naming.
- [x] `errors/` - High-level error umbrella crate: `RethError` and common error/result re-exports across consensus/execution/storage subsystems.
- [x] `ethereum/` - Ethereum-specific node stack: Ethereum CLI, consensus validation, Engine API primitives, EVM config, fork schedules, payload builder/validator, Ethereum node wiring (components/RPC add-ons), and canonical primitive types.
- [x] `etl/` - ETL collector: buffers unsorted key/value pairs, sorts + spills to temp files, and iterates a merged sorted stream (often used for efficient sorted DB inserts / indexing).
- [x] `evm/` - EVM/execution subsystem: core `reth-evm` execution traits/config, shared execution error types, and shared execution outcome/result types (`ExecutionOutcome`, `Chain`, etc.).
- [x] `exex/` - Execution extensions subsystem: ExEx manager/runtime, shared ExEx notification/progress types, and test utilities; includes notification backfill + WAL persistence to support recovery and pruning-safe progress.
- [x] `fs-util/` - Path-aware filesystem utilities: wraps `std::fs` with richer errors and provides common helpers like atomic writes and JSON read/write.
- [x] `metrics/` - Metrics utilities: re-exports the `Metrics` derive macro and provides common helpers like metered tokio mpsc channels (feature-gated).
- [x] `net/` - Networking stack: discovery, peer/session management, wire protocols, downloaders, network manager, and shared networking types/APIs.
- [x] `node/` - Node-layer crates: builder/launch, core config/CLI utils, metrics/events, EthStats, and node API/types.
- [x] `optimism/` - OP-specific crates: chain specs, consensus/EVM, node wiring, RPC, flashblocks, and txpool support.
- [x] `payload/` - Payload builder subsystem: jobs/services, primitives, validators, and utilities.
- [x] `primitives/` - Common block/tx/receipt types, transaction helpers, and primitives benchmarks.
- [x] `primitives-traits/` - Core traits/utilities for blocks, headers, txs, receipts, and serialization helpers.
- [x] `prune/` - Pruning engine, segment definitions, and shared prune configuration types.
- [x] `ress/` - RESS protocol and provider crates for stateless witness/bytecode fetching.
- [x] `revm/` - Reth-specific revm adapters, cached reads, and witness helpers.
- [x] `rpc/` - RPC subsystem crates: interfaces, handlers, builders, and supporting types/tests.
- [x] `stages/` - Staged sync APIs/types and concrete stage implementations with benchmarks.
- [x] `stateless/` - Stateless validation pipeline, trie helpers, and witness-backed DB.
- [x] `static-file/` - Static file producer and shared segment/type definitions.
- [x] `storage/` - Storage crates for DB access, providers, codecs, and static files.
- [x] `tasks/` - Task management, executors, shutdown signaling, and metrics.
- [x] `tokio-util/` - Tokio event broadcasting utilities and optional rate limiter.
- [x] `tracing/` - Tracing configuration, formatter, layer builders, and throttle macro.
- [x] `tracing-otlp/` - OTLP tracing/logging layer builders and config types.
- [x] `transaction-pool/` - Transaction pool implementation with validation, subpools, and metrics.
- [x] `trie/` - Trie subsystem crates: common types, DB integration, sparse/parallel, and core roots/proofs.

### Files
- (none)

## Notes
- This `AGENTS.md` is intentionally high-level; details are pushed down into each subdirectory's own `AGENTS.md`.
