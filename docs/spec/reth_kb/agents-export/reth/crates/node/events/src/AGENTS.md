# src

## Purpose
Node event handlers for logging and health monitoring, including consensus-layer health checks and pipeline progress reporting.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring for node and consensus-layer event handlers.
  - **Key items**: `cl`, `node`
- `node.rs` - Processes pipeline/network/engine events and emits periodic status logs.
  - **Key items**: `NodeState`, `INFO_MESSAGE_INTERVAL`, event handlers for `PipelineEvent` and `ConsensusEngineEvent`
- `cl.rs` - Consensus Layer health event stream with periodic checks.
  - **Key items**: `ConsensusLayerHealthEvents`, `ConsensusLayerHealthEvent`, `CHECK_INTERVAL`

## Key APIs (no snippets)
- `ConsensusLayerHealthEvents`
- `NodeState` (internal event handler)
