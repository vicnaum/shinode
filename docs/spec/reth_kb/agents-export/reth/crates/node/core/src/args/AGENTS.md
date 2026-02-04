# args

## Purpose
CLI argument structs and parsing helpers for configuring node subsystems (network, RPC, engine, DB, pruning, tracing, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and re-exports for CLI argument groups.
  - **Key items**: `NetworkArgs`, `RpcServerArgs`, `TxPoolArgs`, `PayloadBuilderArgs`, `EngineArgs`, `PruningArgs`
- `benchmark_args.rs` - Benchmark CLI arguments.
  - **Key items**: `BenchmarkArgs`
- `database.rs` - Database CLI configuration and size parsing.
  - **Key items**: `DatabaseArgs`, `ByteSize`
- `datadir_args.rs` - Data directory CLI arguments.
  - **Key items**: `DatadirArgs`
- `debug.rs` - Debug CLI options and invalid-block hook selection.
  - **Key items**: `DebugArgs`, `InvalidBlockSelection`, `InvalidBlockHookType`
- `dev.rs` - Dev-mode CLI configuration.
  - **Key items**: `DevArgs`
- `engine.rs` - Engine CLI configuration.
  - **Key items**: `EngineArgs`, `DefaultEngineValues`
- `era.rs` - ERA import CLI arguments and source selection.
  - **Key items**: `EraArgs`, `EraSourceArgs`, `DefaultEraHost`
- `error.rs` - Parsing errors for receipt log prune configs.
  - **Key items**: `ReceiptsLogError`
- `gas_price_oracle.rs` - Gas price oracle CLI arguments.
  - **Key items**: `GasPriceOracleArgs`
- `log.rs` - Logging CLI configuration.
  - **Key items**: `LogArgs`, `ColorMode`, `Verbosity`
- `metric.rs` - Metrics CLI configuration.
  - **Key items**: `MetricArgs`
- `network.rs` - Network and discovery CLI configuration.
  - **Key items**: `NetworkArgs`, `DiscoveryArgs`
- `payload_builder.rs` - Payload builder CLI configuration.
  - **Key items**: `PayloadBuilderArgs`, `DefaultPayloadBuilderValues`
- `pruning.rs` - Pruning CLI configuration.
  - **Key items**: `PruningArgs`
- `ress_args.rs` - RESS subprotocol CLI configuration.
  - **Key items**: `RessArgs`
- `rpc_server.rs` - RPC server CLI configuration.
  - **Key items**: `RpcServerArgs`, `DefaultRpcServerArgs`
- `rpc_state_cache.rs` - RPC state cache CLI configuration.
  - **Key items**: `RpcStateCacheArgs`
- `stage.rs` - Stage selection CLI enum.
  - **Key items**: `StageEnum`
- `static_files.rs` - Static file CLI configuration and defaults.
  - **Key items**: `StaticFilesArgs`, `MINIMAL_BLOCKS_PER_FILE`
- `trace.rs` - Tracing/OTLP CLI configuration.
  - **Key items**: `TraceArgs`, `OtlpInitStatus`, `OtlpLogsStatus`
- `txpool.rs` - Transaction pool CLI configuration.
  - **Key items**: `TxPoolArgs`, `DefaultTxPoolValues`
- `types.rs` - CLI parsing helper types (zero-as-none, max).
  - **Key items**: `ZeroAsNoneU64`, `ZeroAsNoneU32`, `MaxU64`, `MaxU32`, `MaxOr`

## Key APIs (no snippets)
- `NetworkArgs`, `RpcServerArgs`, `TxPoolArgs`, `PayloadBuilderArgs`
