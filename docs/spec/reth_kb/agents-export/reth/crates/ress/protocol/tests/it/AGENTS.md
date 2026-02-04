# it

## Purpose
Integration tests for RESS protocol behavior and message flow.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test harness module wiring.
- **Key items**: module `e2e`

### `e2e.rs`
- **Role**: End-to-end protocol tests using the network test harness.
- **Key items**: `disconnect_on_stateful_pair`, `message_exchange`, `witness_fetching_does_not_block`
- **Interactions**: Uses `RessProtocolHandler` and mock providers to validate message flow.

## Key APIs (no snippets)
- `RessProtocolHandler`, `ProtocolEvent`
