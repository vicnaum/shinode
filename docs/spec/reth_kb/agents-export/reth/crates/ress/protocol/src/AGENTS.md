# src

## Purpose
RESS protocol definitions, message codecs, and network handlers.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and module wiring for RESS protocol.
  - **Key items**: `NodeType`, `RessProtocolMessage`, `RessProtocolProvider`, `RessProtocolConnection`
- `types.rs` - Node type enum and handshake compatibility checks.
  - **Key items**: `NodeType`, `is_valid_connection()`
- `message.rs` - RLP message definitions and encoding helpers.
  - **Key items**: `RessProtocolMessage`, `RessMessage`, `RessMessageID`, `GetHeaders`
- `provider.rs` - Provider trait for serving headers, bodies, bytecode, and witnesses.
  - **Key items**: `RessProtocolProvider`
  - **Knobs / invariants**: Respects `MAX_HEADERS_SERVE`, `MAX_BODIES_SERVE`, `SOFT_RESPONSE_LIMIT`.
- `handlers.rs` - Protocol handler for connection lifecycle and max-connection limits.
  - **Key items**: `RessProtocolHandler`, `ProtocolEvent`, `ProtocolState`
- `connection.rs` - Connection state machine for request/response handling.
  - **Key items**: `RessProtocolConnection`, `RessPeerRequest`
  - **Interactions**: Routes `GetHeaders`, `GetBlockBodies`, `GetBytecode`, `GetWitness` messages.
- `test_utils.rs` - Mock and noop provider implementations for tests.
  - **Key items**: `MockRessProtocolProvider`, `NoopRessProtocolProvider`

## Key APIs (no snippets)
- `RessProtocolMessage`, `RessProtocolHandler`, `RessProtocolProvider`
- `NodeType`
