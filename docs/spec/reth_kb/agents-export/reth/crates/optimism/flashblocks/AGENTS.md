# flashblocks

## Purpose
`reth-optimism-flashblocks` crate: flashblocks stream decoding, pending block assembly, and optional consensus submission.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Flashblocks service, sequence management, builders, and websocket client.
- [x] `tests/` - Integration tests for websocket streaming.

### Files
- `Cargo.toml` - Manifest for flashblocks dependencies (reth execution, websocket IO, codecs).
  - **Key items**: deps `reth-evm`, `reth-optimism-payload-builder`, `tokio-tungstenite`
