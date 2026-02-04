# ipc

## Purpose
`reth-ipc` crate: IPC transport support for `jsonrpsee` clients and servers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - IPC client/server implementations and stream codec.

### Files
- `Cargo.toml` - Manifest for IPC transport dependencies.
  - **Key items**: `jsonrpsee` (client/server), `interprocess`, `tokio`, `tokio-util`
- `README.md` - Crate overview and JSON-RPC IPC description.
  - **Key items**: `jsonrpsee` IPC transport
