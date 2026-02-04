# client

## Purpose
IPC client transport for `jsonrpsee`, providing sender/receiver adapters and a client builder.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Implements the IPC transport sender/receiver and `IpcClientBuilder` wrapper around `jsonrpsee` client creation.
- **Key items**: `Sender`, `Receiver`, `IpcTransportClientBuilder`, `IpcClientBuilder`, `IpcError`
- **Interactions**: Uses `StreamCodec` for framing, `interprocess` sockets for connection, and `jsonrpsee` client traits.
- **Knobs / invariants**: `request_timeout` default is 60s; `send_ping` is not supported.

## End-to-end flow (high level)
- Build an IPC transport pair with `IpcTransportClientBuilder`.
- Connect via `LocalSocketStream` and wrap the receiver with `StreamCodec`.
- Use `IpcClientBuilder` to construct a `jsonrpsee` `Client`.
- Send requests through `Sender` and receive responses with `Receiver`.

## Key APIs (no snippets)
- `IpcClientBuilder`, `IpcTransportClientBuilder`
- `Sender`, `Receiver`, `IpcError`
