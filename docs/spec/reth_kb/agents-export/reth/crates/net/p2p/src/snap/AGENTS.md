# snap

## Purpose
Traits for SNAP sync requests and response types in the p2p layer.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for SNAP client traits.
- **Key items**: `client`

### `client.rs`
- **Role**: Defines `SnapClient` and response variants for SNAP requests with priority support.
- **Key items**: `SnapClient`, `SnapResponse`, `get_account_range*()`, `get_storage_ranges*()`, `get_byte_codes*()`, `get_trie_nodes*()`

## End-to-end flow (high level)
- A `SnapClient` issues SNAP requests (accounts, storage, bytecodes, trie nodes) with optional priority.
- The client returns a `SnapResponse` variant for the requested data type.

## Key APIs (no snippets)
- `SnapClient`, `SnapResponse`
