# Q039 - CL Ingress and Engine API (AuthRPC)

Status: verified

## Question
Consensus Layer (CL) ingress: does Reth require a separate CL client, how does it receive CL updates, and what is it used for (new blocks, forkchoice)?

## Context Packs
- `spec/reth_repo_context/context-engine.xml`
- `spec/reth_repo_context/context-rpc.xml`

## Gemini Prompt
N/A (local code analysis)

## Answer (Local Analysis)

### Summary
- Reth expects an external CL that connects over the authenticated Engine API (authrpc) and sends `engine_newPayload*` + `engine_forkchoiceUpdated*`.
- The Engine API implementation forwards these requests into a `ConsensusEngineHandle`, which emits `BeaconEngineMessage` items on an internal channel.
- The engine service consumes a stream of `BeaconEngineMessage` and drives the engine tree/live sync loop from those CL messages.
- AuthRPC uses JWT authentication; it is a separate server from the public RPC endpoints.
- There is no built-in CL or 3rd-party API provider; the CL is an external process (or a custom driver that speaks Engine API).

### Entry Points
- `crates/rpc/rpc-engine-api/src/engine_api.rs`: Engine API RPC implementation that receives CL calls and forwards them into the engine handle.
- `crates/engine/primitives/src/message.rs`: Defines `BeaconEngineMessage` and the `ConsensusEngineHandle` sender.
- `crates/engine/service/src/service.rs`: Engine service consumes `EngineMessageStream<BeaconEngineMessage>` to drive the chain.
- `crates/node/builder/src/launch/engine.rs`: Wires the authrpc JWT secret, creates the engine message channel, and constructs the engine service.
- `crates/rpc/rpc-builder/src/auth.rs`: AuthRPC server with JWT auth layer.

### How CL Messages Flow
1. The CL calls Engine API methods (e.g., `engine_newPayloadV*`, `engine_forkchoiceUpdatedV*`) on the authrpc server.
2. `EngineApi` forwards those calls into `ConsensusEngineHandle`, which sends `BeaconEngineMessage::NewPayload` / `::ForkchoiceUpdated` through an internal channel.
3. The engine service consumes these messages as an `EngineMessageStream` and feeds them into the engine handler/tree loop.

### What It Is Used For
- **New blocks:** `engine_newPayloadV*` delivers execution payloads from the CL.
- **Canonical head & finalization:** `engine_forkchoiceUpdatedV*` tells the EL which head/safe/finalized hashes to follow.
- **Payload building (if enabled):** `engine_getPayload*` is part of the same Engine API surface (used by block producers).

## Corrections / Caveats
- This is not a "built-in CL" or a third-party API integration; the code explicitly models CL ingress as Engine API RPC calls that are forwarded into the engine via `ConsensusEngineHandle`.
- Engine API is served on an authenticated (JWT) RPC server; it is separate from the public HTTP/WS RPC servers.

## Verification
- Engine API is "the entrypoint for engine API processing" for the consensus layer, and forwards `new_payload_v*` into `ConsensusEngineHandle`.```48:153:reth/crates/rpc/rpc-engine-api/src/engine_api.rs
```
- `BeaconEngineMessage` is explicitly described as engine RPC API invoked by the consensus layer, and `ConsensusEngineHandle` sends these messages.```145:257:reth/crates/engine/primitives/src/message.rs
```
- The engine service consumes a stream of `BeaconEngineMessage` items (`EngineMessageStream`) and drives the engine loop.```36:122:reth/crates/engine/service/src/service.rs
```
- The node launcher wires the engine message channel and creates the `ConsensusEngineHandle` used by Engine API.```148:205:reth/crates/node/builder/src/launch/engine.rs
```
- AuthRPC server uses JWT auth middleware.```84:96:reth/crates/rpc/rpc-builder/src/auth.rs
```

## Actionable Pointers
- Engine API ingress: `reth/crates/rpc/rpc-engine-api/src/engine_api.rs`.
- CL → EL message types: `reth/crates/engine/primitives/src/message.rs`.
- Engine service entry: `reth/crates/engine/service/src/service.rs`.
- AuthRPC JWT config: `reth/crates/rpc/rpc-builder/src/auth.rs`.
