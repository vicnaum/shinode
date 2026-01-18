# src

## Purpose
IPC transport and server/client implementations for JSON-RPC over local sockets.

## Contents (one hop)
### Subdirectories
- [x] `client/` - IPC client transport wrappers and builders.
- [x] `server/` - IPC server, connection handling, and request processing.

### Files
- `lib.rs` - Crate entrypoint with module wiring and feature flags.
  - **Key items**: `client`, `server`, `stream_codec`
- `stream_codec.rs` - Streaming JSON codec for IPC/tcp transports.
  - **Key items**: `StreamCodec`, `Separator`, `stream_incoming()`
  - **Interactions**: Used by both client and server framing paths.

## Key APIs (no snippets)
- `StreamCodec`, `Separator`
