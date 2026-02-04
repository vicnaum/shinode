# src

## Purpose
Implements `reth-exex-types`: shared, dependency-light types for ExEx integration-notifications sent to ExExes (`ExExNotification`), ExEx progress/head markers, and the aggregated "finished height" signal used for pruning decisions.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring + exports; optional `serde_bincode_compat` module for always-serializing wrapper types.
  - **Key items**: `ExExNotification`, `ExExHead`, `FinishedExExHeight`
- `notification.rs` - ExEx notification enum (commit/reorg/revert) wrapping `Arc<Chain<N>>`, helpers to access committed/reverted chains and invert notifications; conversion from `CanonStateNotification`.
  - **Key items**: `ExExNotification`, `committed_chain()`, `reverted_chain()`, `into_inverted()`
- `head.rs` - ExEx head marker (highest processed host block).
  - **Key items**: `ExExHead`
- `finished_height.rs` - aggregated finished height across all ExExes (NoExExs/NotReady/Height).
  - **Key items**: `FinishedExExHeight`
