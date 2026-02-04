# actions

## Purpose
Action library for the e2e testing framework: composable async steps that operate on a `testsuite::Environment` to drive nodes via RPC/Engine API, produce blocks, create forks/reorgs, and assert expected behavior.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Defines the `Action` trait and core action containers (`ActionBox`, `Sequence`), re-exports all concrete actions, and provides common helpers for validating forkchoice-update results.
- **Key items**: `Action<I>`, `ActionBox<I>`, `Sequence<I>`, `MakeCanonical`, `CaptureBlock`, `validate_fcu_response()`, `expect_fcu_valid()`
- **Interactions**: actions operate on `testsuite::Environment` and often call `EngineApiClient` for FCU/newPayload flows.
- **Knobs / invariants**: requires `EngineTypes` (and sometimes `PayloadTypes`) bounds for actions that build payload attributes or convert payload envelopes.

#### `produce_blocks.rs`
- **Role**: Block production and propagation actions: generate payload attributes, request payload building, broadcast payloads/FCUs, and assert acceptance/canonicalization.
- **Key items**: `AssertMineBlock`, `ProduceBlocks`, `ProduceBlocksLocally`, `ProduceInvalidBlocks`, `GeneratePayloadAttributes`, `GenerateNextPayload`, `BroadcastLatestForkchoice`, `BroadcastNextNewPayload`
- **Interactions**: uses both `EthApiClient` (to read latest blocks) and `EngineApiClient` (to send FCU/getPayload/newPayload); updates per-node `Environment` state (block info, payload IDs, tags).
- **Knobs / invariants**: some actions fall back between Engine API versions (v2 <-> v3) depending on fork support; relies on environment-managed timestamp increments and tag registry.

#### `engine_api.rs`
- **Role**: Engine API-specific actions for explicitly sending `newPayload` to nodes (including ordering/buffering scenarios) and asserting expected payload status outcomes.
- **Key items**: `SendNewPayload`, `SendNewPayloads`, `ExpectedPayloadStatus`
- **Interactions**: fetches blocks from a "source" node via `EthApiClient`, converts to `ExecutionPayloadV3`, and calls `EngineApiClient::new_payload_v3`.
- **Knobs / invariants**: expected-status matching supports "valid", "invalid", and "syncing/accepted" (buffered) scenarios.

#### `custom_fcu.rs`
- **Role**: Custom forkchoice-update actions to craft FCUs with explicit head/safe/finalized references (by hash/tag/latest) and validate response status.
- **Key items**: `BlockReference`, `SendForkchoiceUpdate`, `FinalizeBlock`, `resolve_block_reference()`
- **Interactions**: resolves tags via `Environment`'s registry, sends FCUs via `EngineApiClient::fork_choice_updated_v3`.
- **Knobs / invariants**: optional expected status for response validation; optional target node (defaults to active node).

#### `fork.rs`
- **Role**: Fork creation and validation actions: choose a fork base (block number or tag), update environment forkchoice state, produce blocks on the fork, and verify chain ancestry expectations.
- **Key items**: `ForkBase`, `CreateFork`, `SetForkBase`, `SetForkBaseFromBlockInfo`, `ValidateFork`
- **Interactions**: uses `EthApiClient` to locate fork base blocks and updates `Environment`'s per-node fork tracking (`current_fork_base`, forkchoice state).
- **Knobs / invariants**: fork base is stored for later validation; actions assume the environment's active node is the target for fork operations unless overridden upstream.

#### `reorg.rs`
- **Role**: Reorg actions: set a reorg target (typically via a previously captured/tagged block) and broadcast forkchoice to make a different head canonical.
- **Key items**: `ReorgTarget`, `ReorgTo`, `SetReorgTarget`
- **Interactions**: composes smaller actions (e.g. `BroadcastLatestForkchoice`) via `Sequence`.
- **Knobs / invariants**: "direct hash" reorgs are explicitly rejected; requires tagging/capturing the target block first.

#### `node_ops.rs`
- **Role**: Multi-node coordination actions: select active node, compare tips, tag blocks from specific nodes, wait for sync, and assert per-node chain state.
- **Key items**: `SelectActiveNode`, `CompareNodeChainTips`, `CaptureBlockOnNode`, `ValidateBlockTag`, `WaitForSync`
- **Interactions**: uses `EthApiClient` queries across nodes and updates the environment's active-node index and tag registry.
- **Knobs / invariants**: retry/timeout driven polling for "wait for sync" style actions.

## Key APIs (no snippets)
- **Action surface**: `Action<I>`, `ActionBox<I>`, `Sequence<I>`
- **Core orchestration**: `MakeCanonical`, `CaptureBlock`
- **Engine API actions**: `SendNewPayload`, `SendForkchoiceUpdate`, `FinalizeBlock`
- **Chain topology**: `CreateFork`, `ReorgTo`, `WaitForSync`

## Relationships
- **Used by**: `reth/crates/e2e-test-utils/src/testsuite/*` and workspace e2e test binaries (`tests/e2e-testsuite/*`).
- **Depends on**: `reth-rpc-api` clients (`EthApiClient`, `EngineApiClient`) plus engine/eth RPC types (`alloy_rpc_types_*`).

## End-to-end flow (high level)
- Build an e2e `Environment` with one or more nodes and RPC/Engine API clients.
- Execute one action (or a `Sequence`) at a time against the environment.
- Actions drive nodes via Engine API (FCU/newPayload/getPayload) and/or regular RPC reads, updating environment state (tips, tags, payload IDs).
- Assertions are embedded in actions (status checks, ancestry checks, expected equality/difference of tips).
