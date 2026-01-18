# src

## Purpose
Stream utilities for consuming a `BeaconEngineMessage` stream: wrappers for skipping messages, storing Engine API calls to disk, and simulating reorg behavior for testing/experimentation.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `engine_store.rs` - on-disk message store + `EngineStoreStream` wrapper that persists FCU/newPayload calls as JSON.
- `lib.rs` - `EngineMessageStreamExt` extension trait (combinators: skip/store/reorg).
- `reorg.rs` - `EngineReorg` stream wrapper that injects reorg sequences at a configured frequency/depth.
- `skip_fcu.rs` - `EngineSkipFcu` wrapper that skips a number of forkchoiceUpdated messages (responds with syncing).
- `skip_new_payload.rs` - `EngineSkipNewPayload` wrapper that skips a number of newPayload messages (responds with syncing).

## Key APIs (no snippets)
- **Traits**: `EngineMessageStreamExt` (extension methods for `Stream<Item = BeaconEngineMessage<_>>`)
- **Types**: `EngineMessageStore`, `EngineStoreStream`, `StoredEngineApiMessage`, `EngineReorg`, `EngineSkipFcu`, `EngineSkipNewPayload`

## Relationships
- **Depends on**: `reth-engine-primitives` (message types), `reth-engine-tree` (validator used by reorg simulation), `reth-fs-util` (filesystem helpers), `serde` (JSON serialization).
- **Used by**: tooling/tests/bench scenarios that want deterministic Engine API input playback or fault injection.

## Notes
- The "skip" wrappers send immediate SYNCING-style responses via the oneshot channels, so the upstream caller doesn't hang when messages are intentionally dropped.
