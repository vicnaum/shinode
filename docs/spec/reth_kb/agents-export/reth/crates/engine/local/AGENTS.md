# local

## Purpose
`reth-engine-local` crate root: local mining/dev-chain utilities that generate Engine API messages (FCU/newPayload) without an external CL.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `LocalMiner`/`MiningMode` loop and payload-attribute builders.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-local` (optional `op` feature for OP payload attributes).

## Key APIs (no snippets)
- **Exports**: `LocalMiner`, `MiningMode`, `LocalPayloadAttributesBuilder`
