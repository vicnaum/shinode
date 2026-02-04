# src

## Purpose
Implements the "local engine" components used to drive a dev chain without an external consensus client: a local miner loop and payload-attribute builders for producing Engine API traffic.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring; exports `LocalMiner`, `MiningMode`, and `LocalPayloadAttributesBuilder`.
- `miner.rs` - `LocalMiner` and `MiningMode`: drive payload building (instant or interval) and send FCU/newPayload to the engine.
- `payload.rs` - `LocalPayloadAttributesBuilder`: builds payload attributes (timestamp, withdrawals, etc.) for local payload jobs (Ethereum; optional OP support).

## Key APIs (no snippets)
- **Types**: `LocalMiner`, `MiningMode`, `LocalPayloadAttributesBuilder`
- **Methods**: `LocalMiner::new()`, `LocalMiner::run()`

## Relationships
- **Uses**: `reth-engine-primitives::ConsensusEngineHandle` to send `fork_choice_updated` / `new_payload` messages to the engine, and `reth-payload-builder::PayloadBuilderHandle` to resolve payload jobs.
- **Intended for**: local dev/test setups where the node itself generates Engine API inputs.
