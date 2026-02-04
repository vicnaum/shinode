# consensus

## Purpose
Consensus/validation subsystem crates: core consensus trait surface and errors, shared Ethereum validation helpers, and debug tooling for driving the engine API from external block sources.

## Contents (one hop)
### Subdirectories
- [x] `common/` - `reth-consensus-common`: shared pre-execution validation helpers (header/body/fork checks).
- [x] `consensus/` - `reth-consensus`: consensus traits (`HeaderValidator`/`Consensus`/`FullConsensus`) and shared `ConsensusError` model (plus noop/test implementations).
- [x] `debug-client/` - `reth-consensus-debug-client`: debug worker that feeds `newPayload`/`forkchoiceUpdated` to an execution client using blocks from RPC/Etherscan.

### Files
- (none)

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`
- **Helpers**: `reth-consensus-common::validation::*`
- **Debug tooling**: `DebugConsensusClient`, `BlockProvider`

## Relationships
- **Used by**: sync/pipeline and engine/execution flows for validating headers/blocks (and, in debug scenarios, for simulating consensus inputs).
