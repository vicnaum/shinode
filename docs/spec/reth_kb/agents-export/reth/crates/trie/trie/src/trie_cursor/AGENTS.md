# trie_cursor

## Purpose
Trie cursor traits and implementations for iterating account and storage tries, including
in-memory overlays, depth-first traversal, and metrics instrumentation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Defines core trie cursor traits and exports cursor helpers.
- **Key items**: `TrieCursorFactory`, `TrieCursor`, `TrieStorageCursor`, `TrieCursorIter`,
  `DepthFirstTrieIterator`, `CursorSubNode`
- **Interactions**: Implemented by `in_memory.rs`, `noop.rs`, and `mock.rs`.

### `depth_first.rs`
- **Role**: Depth-first post-order iterator over trie nodes.
- **Key items**: `DepthFirstTrieIterator`, `cmp()`
- **Interactions**: Used by verification routines to compare branch nodes in depth-first order.

### `in_memory.rs`
- **Role**: Overlay cursors that merge trie updates with database cursors.
- **Key items**: `InMemoryTrieCursorFactory`, `InMemoryTrieCursor`, `new_account()`, `new_storage()`
- **Interactions**: Uses `ForwardInMemoryCursor` and `TrieUpdatesSorted` to override DB nodes.
- **Knobs / invariants**: Overlay entries take precedence; wiped storage bypasses DB cursor.

### `metrics.rs`
- **Role**: Metrics wrappers and caches for trie cursor operations.
- **Key items**: `TrieCursorMetrics`, `TrieCursorMetricsCache`, `InstrumentedTrieCursor`
- **Interactions**: Wraps `TrieCursor`/`TrieStorageCursor` to record seek/next timings and counts.

### `mock.rs`
- **Role**: Mock trie cursor factory and cursors for tests with visit tracking.
- **Key items**: `MockTrieCursorFactory`, `MockTrieCursor`, `visited_account_keys()`,
  `visited_storage_keys()`
- **Interactions**: Builds cursors from `TrieUpdates` to drive tests.

### `noop.rs`
- **Role**: Noop trie cursor implementations for empty datasets or tests.
- **Key items**: `NoopTrieCursorFactory`, `NoopAccountTrieCursor`, `NoopStorageTrieCursor`
- **Interactions**: Satisfies `TrieCursorFactory`/`TrieStorageCursor` traits.

### `subnode.rs`
- **Role**: Subtrie cursor node representation with mask and hash accessors.
- **Key items**: `CursorSubNode`, `SubNodePosition`, `full_key_is_only_nonremoved_child()`,
  `hash_flag()`, `maybe_hash()`
- **Interactions**: Used by `TrieWalker` and progress checkpoints.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `TrieCursorFactory`, `TrieCursor`, `TrieStorageCursor`,
  `TrieCursorIter`, `DepthFirstTrieIterator`, `CursorSubNode`
- **Modules / Packages**: `in_memory`, `depth_first`, `metrics`, `subnode`
- **Functions**: `seek()`, `seek_exact()`, `next()`, `set_hashed_address()`

## Relationships
- **Depends on**: `reth-trie-common` for `BranchNodeCompact` and `Nibbles`.
- **Data/control flow**: in-memory overlays merge updates with DB cursor output, yielding a
  unified ordered stream of nodes.
- **Data/control flow**: depth-first iterators and subnode helpers feed root verification and
  trie walking logic.

## End-to-end flow (high level)
- Create a cursor factory (DB-backed, in-memory overlay, or noop).
- Use cursors to seek/iterate nodes in lexicographic order.
- Optionally wrap cursors with metrics instrumentation.
- Feed cursor output into walkers, proof generators, or verification routines.
