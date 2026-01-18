# debug-client

## Purpose
`reth-consensus-debug-client` crate: debug-only "consensus driver" that sources recent blocks from an external provider (RPC or Etherscan) and feeds them into the engine API (`newPayload` + `forkchoiceUpdated`) to exercise an execution client without a real consensus node.

## Contents (one hop)
### Subdirectories
- [x] `src/` - debug consensus worker and block-provider backends.

### Files
- `Cargo.toml` - crate manifest (RPC/Etherscan client dependencies and engine API types).
  - **Key items**: `alloy-provider (ws)`, `reqwest (rustls-tls)`

## Key APIs (no snippets)
- **Worker**: `DebugConsensusClient`
- **Sources**: `BlockProvider`, `RpcBlockProvider`, `EtherscanBlockProvider`

## Relationships
- **Connects**: external block sources <-> engine API handle (`reth-node-api`) for driving payload + forkchoice messages.
