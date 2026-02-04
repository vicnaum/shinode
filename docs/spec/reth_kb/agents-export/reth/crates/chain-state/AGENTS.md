# chain-state

## Purpose
`reth-chain-state` crate: provides chain-state tracking primitives for reth, especially around the canonical chain's "recent, not-yet-persisted" blocks, forkchoice-related heads (head/safe/finalized), and notification streams for consumers like the engine and RPC.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for canonical-chain helpers over in-memory overlays.
- [x] `src/` - Canonical in-memory state tracking, chain-info tracking, notifications, and overlay state providers.

### Files
- `Cargo.toml` - crate manifest (features like `serde` and `test-utils`, bench registration).

## Key APIs (no snippets)
- **Types**: `CanonicalInMemoryState`, `ChainInfoTracker`, `CanonStateNotification`, `MemoryOverlayStateProvider`, `DeferredTrieData`
- **Streams/subscriptions**: canonical-state notifications (`CanonStateNotificationStream`) and forkchoice head subscriptions (`ForkChoiceSubscriptions`).

## Relationships
- **Used by**: engine components that need quick access to recent canonical data and to publish chain-change notifications.
