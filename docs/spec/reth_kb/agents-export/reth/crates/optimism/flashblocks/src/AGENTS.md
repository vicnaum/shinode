# src

## Purpose
Flashblocks integration for OP: stream and decode flashblocks, assemble pending blocks, and optionally drive consensus updates.

## Contents (one hop)
### Subdirectories
- [x] `ws/` - Websocket stream and decoder for flashblock payloads.

### Files
- `lib.rs` - Crate wiring, re-exports, and channel type aliases.
  - **Key items**: `FlashblocksListeners`, `PendingBlockRx`, `FlashBlockCompleteSequenceRx`, `FlashBlockRx`
- `consensus.rs` - Consensus client that submits flashblock-derived payloads to the engine.
  - **Key items**: `FlashBlockConsensusClient`, `submit_new_payload()`, `submit_forkchoice_update()`
- `payload.rs` - Flashblock payload types and pending block wrapper.
  - **Key items**: `FlashBlock`, `PendingFlashBlock`
- `sequence.rs` - Sequence tracking and validation for flashblocks.
  - **Key items**: `FlashBlockPendingSequence`, `FlashBlockCompleteSequence`, `SequenceExecutionOutcome`
- `service.rs` - Main service loop for processing flashblocks and building pending blocks.
  - **Key items**: `FlashBlockService`, `FlashBlockBuildInfo`, `run()`
- `worker.rs` - Flashblock block builder and execution worker.
  - **Key items**: `FlashBlockBuilder`, `BuildArgs`, `execute()`
- `cache.rs` - Sequence cache manager and build selection logic.
  - **Key items**: `SequenceManager`, `FLASHBLOCK_BLOCK_TIME`, `CACHE_SIZE`
- `test_utils.rs` - Test-only helpers for building flashblocks.
  - **Key items**: `TestFlashBlockFactory`, `TestFlashBlockBuilder`

## Key APIs (no snippets)
- `FlashBlockService`, `FlashBlockConsensusClient`
- `FlashblocksListeners`, `PendingFlashBlock`
- `WsFlashBlockStream`
