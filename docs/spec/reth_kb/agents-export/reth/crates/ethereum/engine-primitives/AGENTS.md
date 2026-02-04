# engine-primitives

## Purpose
`reth-ethereum-engine-primitives` crate: Ethereum-specific Engine API payload types and conversions (including envelope V1-V5), bridging reth blocks/payload builders to `alloy_rpc_types_engine` representations.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthEngineTypes`/`EthBuiltPayload` and conversion helpers.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-engine-primitives`, `reth-payload-primitives`, and alloy Engine API types).

## Key APIs (no snippets)
- `EthEngineTypes`, `EthPayloadTypes`
- `EthBuiltPayload`, `BlobSidecars`
