# src

## Purpose
Reth-specific adapters and helpers around the `revm` EVM, including storage-backed databases, cached reads, cancellation flags, and witness recording.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint, module wiring, and re-exports for revm integration.
- **Key items**: `cached`, `cancelled`, `database`, `witness` (feature), `test_utils`
- **Interactions**: Re-exports `revm` types and local helpers.

### `database.rs`
- **Role**: Glue layer that adapts reth state providers to `revm::Database`/`DatabaseRef`.
- **Key items**: `EvmStateProvider`, `StateProviderDatabase`
- **Interactions**: Uses `AccountReader`, `BlockHashReader`, `BytecodeReader`, `StateProvider`.
- **Knobs / invariants**: Missing data returns defaults (e.g., empty bytecode, zero hash).

### `cached.rs`
- **Role**: Read-through caching wrappers for repeated payload building.
- **Key items**: `CachedReads`, `CachedReadsDbMut`, `CachedReadsDBRef`, `CachedAccount`
- **Interactions**: Wraps a `DatabaseRef` to cache accounts, storage, bytecode, and block hashes.
- **Knobs / invariants**: `CachedReads` must outlive wrappers; cache assumes same base state.

### `cancelled.rs`
- **Role**: Cancellation markers for long-running execution/payload jobs.
- **Key items**: `CancelOnDrop`, `ManualCancel`
- **Knobs / invariants**: `CancelOnDrop` sets flag on drop; `ManualCancel` sets only on `cancel()`.

### `witness.rs`
- **Role**: Records accessed state during execution for witness generation (feature `witness`).
- **Key items**: `ExecutionWitnessRecord`, `record_executed_state()`, `from_executed_state()`
- **Interactions**: Reads from `revm::database::State` and builds `HashedPostState`.

### `test_utils.rs`
- **Role**: In-memory state provider for tests and fixtures.
- **Key items**: `StateProviderTest`
- **Interactions**: Implements `StateProvider` + proof/root traits with stubs.
- **Knobs / invariants**: Proof/root methods are `unimplemented!` for tests only.

## End-to-end flow (high level)
- Wrap a reth `StateProvider` in `StateProviderDatabase` to satisfy `revm::Database`.
- Optionally layer `CachedReads` over the database for repeated payload building.
- Use `CancelOnDrop`/`ManualCancel` to abort long-running jobs safely.
- Record execution access patterns via `ExecutionWitnessRecord` when witness generation is enabled.
- Use `StateProviderTest` for unit tests without full storage backends.

## Key APIs (no snippets)
- `StateProviderDatabase`, `EvmStateProvider`
- `CachedReads`, `CachedReadsDbMut`, `CachedReadsDBRef`
- `CancelOnDrop`, `ManualCancel`, `ExecutionWitnessRecord`
