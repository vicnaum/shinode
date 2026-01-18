# hashed_cursor

## Purpose
Hashed state cursor traits and overlay implementations used to iterate hashed accounts and
storages, plus noop/mock cursors and metrics instrumentation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Defines core hashed cursor traits and re-exports implementations and metrics helpers.
- **Key items**: `HashedCursorFactory`, `HashedCursor`, `HashedStorageCursor`,
  `HashedCursorMetricsCache`, `InstrumentedHashedCursor`
- **Interactions**: Implemented by `post_state.rs`, `noop.rs`, and `mock.rs`.

### `post_state.rs`
- **Role**: Overlay cursors that merge database state with in-memory post-state updates.
- **Key items**: `HashedPostStateCursorFactory`, `HashedPostStateCursor`,
  `HashedPostStateCursorValue`, `new_account()`, `new_storage()`
- **Interactions**: Uses `ForwardInMemoryCursor` to merge sorted updates with DB cursors.
- **Knobs / invariants**: Post-state entries take precedence; deletions use `None` or `U256::ZERO`;
  storage cursors must `set_hashed_address()` before seeking.

### `noop.rs`
- **Role**: Noop hashed cursor implementations for empty datasets or tests.
- **Key items**: `NoopHashedCursorFactory`, `NoopHashedCursor`, `is_storage_empty()`
- **Interactions**: Satisfies `HashedCursorFactory`/`HashedStorageCursor` traits.

### `metrics.rs`
- **Role**: Metrics wrappers and caches for hashed cursor operations.
- **Key items**: `HashedCursorMetrics`, `HashedCursorMetricsCache`, `InstrumentedHashedCursor`
- **Interactions**: Wraps a `HashedCursor` to record seek/next timings and counts.

### `mock.rs`
- **Role**: Mock hashed cursor factory and cursors for tests with visit tracking.
- **Key items**: `MockHashedCursorFactory`, `MockHashedCursor`, `MockHashedCursorType`,
  `visited_account_keys()`, `visited_storage_keys()`
- **Interactions**: Builds cursors from `HashedPostState` and records key visits for assertions.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `HashedCursorFactory`, `HashedCursor`, `HashedStorageCursor`,
  `HashedPostStateCursor`, `InstrumentedHashedCursor`
- **Modules / Packages**: `post_state`, `noop`, `metrics`, `mock`
- **Functions**: `hashed_account_cursor()`, `hashed_storage_cursor()`, `set_hashed_address()`

## Relationships
- **Depends on**: `reth-trie-common::HashedPostStateSorted` for post-state overlays.
- **Data/control flow**: post-state overlays merge in-memory updates with DB cursors, yielding a
  single ordered stream of hashed entries.
- **Data/control flow**: instrumentation wrappers record operation counts/latency for metrics.

## End-to-end flow (high level)
- Create a cursor factory (DB-backed, overlay, or noop).
- Iterate hashed accounts and storage entries with `seek`/`next`.
- For overlays, in-memory updates override DB entries and deletions are filtered out.
- Optional metrics wrappers record operation counts and timing.
