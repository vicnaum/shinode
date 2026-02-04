# src

## Purpose
Implements `reth-ethereum-engine-primitives`: Ethereum-specific Engine API primitive types and conversions, especially around payload building/output (`EthBuiltPayload`) and mapping sealed blocks into Engine API execution payload envelopes (V1-V5, including Cancun/Prague-era fields).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `EthEngineTypes`/`EthPayloadTypes` (bindings between reth payload types and Engine API types) and re-exports payload helpers and conversion errors.
- **Key items**: `EthEngineTypes`, `EthPayloadTypes`, `payload_id()`, `EthBuiltPayload`, `EthPayloadBuilderAttributes`, `BlobSidecars`, `EthPayloadAttributes`
- **Interactions**: implements `reth_payload_primitives::PayloadTypes` and `reth_engine_primitives::EngineTypes` for Ethereum nodes; converts `SealedBlock` -> `ExecutionData` (`ExecutionPayload` + sidecar).

#### `payload.rs`
- **Role**: Ethereum payload building and Engine API envelope conversion: holds `EthBuiltPayload` (sealed block + fees + sidecars + optional requests) and converts it into `ExecutionPayloadV1` / envelope V2-V5 with correct blobs bundle shape.
- **Key items**: `EthBuiltPayload`, `EthBuiltPayload::try_into_v3()`, `try_into_v4()`, `try_into_v5()`, `BlobSidecars`
- **Interactions**: uses `alloy_rpc_types_engine` envelope types and `alloy_eips` sidecar/request types; enforces "expected sidecar variant" rules across fork versions.

#### `error.rs`
- **Role**: Conversion error types for built-payload -> envelope conversions.
- **Key items**: `BuiltPayloadConversionError`

## Key APIs (no snippets)
- `EthEngineTypes`, `EthPayloadTypes`
- `EthBuiltPayload`, `BlobSidecars`
- `BuiltPayloadConversionError`

## Relationships
- **Used by**: Ethereum engine/payload builders and Engine API server logic that needs to serve `engine_getPayload*` responses for Ethereum forks.
