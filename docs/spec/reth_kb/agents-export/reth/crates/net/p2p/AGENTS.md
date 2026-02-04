# p2p

## Purpose
`reth-network-p2p` crate: shared traits and types for p2p download clients, errors, and sync state interfaces.

## Contents (one hop)
### Subdirectories
- [x] `src/` - P2P client traits, error types, full-block helpers, SNAP interfaces, and test utils.

### Files
- `Cargo.toml` - Crate manifest defining p2p trait dependencies and feature flags.
  - **Key items**: features `test-utils`, `std`; deps `reth-consensus`, `reth-network-types`, `reth-eth-wire-types`

## Key APIs (no snippets)
- `DownloadClient`, `BodiesClient`, `HeadersClient`, `SnapClient`
- `FullBlockClient`, `BlockClient`, `NetworkSyncUpdater`
