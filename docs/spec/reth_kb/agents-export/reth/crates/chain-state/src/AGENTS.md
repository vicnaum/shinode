# src

## Purpose
Implements `reth-chain-state` core types: in-memory canonical chain tracking, chain-info (head/safe/finalized) tracking, canonical-state notifications, deferred trie data helpers, and state-provider overlays that combine persisted state with unpersisted in-memory blocks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `chain_info.rs` - `ChainInfoTracker`: tracks canonical head + safe/finalized/persisted headers and forkchoice timing via `watch` channels.
- `deferred_trie.rs` - `DeferredTrieData` + `ComputedTrieData`: lazy/async trie-data handle with deadlock-avoiding synchronous fallback and anchored overlay reuse.
- `in_memory.rs` - `CanonicalInMemoryState` internals for tracking unpersisted canonical blocks, pending block state, and emitting canon-state notifications.
- `lib.rs` - module wiring and public re-exports.
- `memory_overlay.rs` - `MemoryOverlayStateProvider{,Ref}`: state provider overlaying in-memory executed blocks over a historical provider (incl. roots/proofs APIs).
- `noop.rs` - no-op/placeholder pieces used where a chain-state implementation is required but functionality is intentionally disabled.
- `notifications.rs` - `CanonStateNotification` + subscription/stream types (`CanonStateSubscriptions`, `ForkChoiceSubscriptions`, `WatchValueStream`, etc.).
- `test_utils.rs` - test helpers (compiled in `test-utils` / tests).

## Key APIs (no snippets)
- **Types**: `CanonicalInMemoryState`, `ChainInfoTracker`, `CanonStateNotification`, `CanonStateNotificationStream`, `DeferredTrieData`, `ComputedTrieData`, `AnchoredTrieInput`, `MemoryOverlayStateProvider`, `MemoryOverlayStateProviderRef`
- **Traits**: `CanonStateSubscriptions`, `ForkChoiceSubscriptions`
- **Methods**: `ChainInfoTracker::set_canonical_head()`, `ChainInfoTracker::set_safe()`, `ChainInfoTracker::set_finalized()`, `DeferredTrieData::wait_cloned()`, `MemoryOverlayStateProviderRef::canonical_hashes_range()`

## Relationships
- **Used by**: engine-tree and other subsystems that need fast access to "recent canonical blocks not yet persisted" and/or need to emit canonical-chain change notifications.
- **Depends on**: `reth-storage-api` (provider traits), `reth-trie` (hashed state/trie inputs), `tokio` (`watch`/`broadcast` for subscriptions), `reth-execution-types` (canonical chain segments/receipts).

## Notes
- `DeferredTrieData` is designed to avoid deadlocks by not blocking on async completion: it tries to lock and can compute synchronously from stored inputs if needed.
