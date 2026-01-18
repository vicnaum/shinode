# session

## Purpose
Shared configuration types for peer session management: timeouts and buffering limits that govern how session tasks communicate with the session manager and how long the node waits before treating peers as timed out or in protocol breach.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that exports the main session config types.
- **Key items**: `SessionsConfig`, `SessionLimits`

### `config.rs`
- **Role**: Session manager configuration: request timeouts, protocol breach window, pending session timeout, per-session command buffer sizing, and scalable event buffering.
- **Key items**: `SessionsConfig`, `SessionLimits`, `INITIAL_REQUEST_TIMEOUT`, `PROTOCOL_BREACH_REQUEST_TIMEOUT`, `PENDING_SESSION_TIMEOUT`, `with_upscaled_event_buffer()`
- **Knobs / invariants**: Two-stage timeout model: internal request timeout vs a longer "protocol breach" ceiling after which a non-responsive peer is considered violating.
