# src

## Purpose
Implements the RLPx ECIES framed transport used by Ethereum's devp2p stack: performs the AUTH/ACK handshake, derives symmetric keys, encrypts/decrypts framed messages (header/body), and exposes Tokio `codec` + `Stream`/`Sink` wrappers for use over async I/O.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `algorithm.rs`
- **Role**: Core ECIES handshake + frame crypto: parses and validates encrypted messages, derives symmetric keys, and reads/writes AUTH, ACK, header, and body frames.
- **Key items**: `ECIES`, `EncryptedMessage`, `RLPxSymmetricKeys`, `new_client()`, `new_server()`, `write_auth()`/`read_auth()`, `write_ack()`/`read_ack()`, `write_header()`/`read_header()`, `write_body()`/`read_body()`
- **Interactions**: Used by `ECIESCodec` (`codec.rs`) as the stateful cryptographic engine for framing.
- **Knobs / invariants**: Message integrity is enforced via tag checks (HMAC-SHA256 for ECIES messages; Ethereum MAC for RLPx frames); errors map to `ECIESErrorImpl`.

### `codec.rs`
- **Role**: Tokio `Decoder`/`Encoder` implementation for the ECIES protocol: drives the handshake state machine and emits/accepts high-level ingress/egress values.
- **Key items**: `ECIESCodec`, `ECIESState` (`Auth`, `Ack`, `InitialHeader`, `Header`, `Body`), `MAX_INITIAL_HANDSHAKE_SIZE`
- **Interactions**: Wraps `ECIES` (`algorithm.rs`); produces `IngressECIESValue` and consumes `EgressECIESValue` from `lib.rs`.
- **Knobs / invariants**: Initial post-handshake header is bounded (`MAX_INITIAL_HANDSHAKE_SIZE`) to limit oversized initial payloads.

### `error.rs`
- **Role**: Error types for ECIES handshake and framing failures (cryptographic tag checks, malformed frames, IO/codec boundary conditions, and handshake mismatches).
- **Key items**: `ECIESError`, `ECIESErrorImpl`, `TagCheck*Failed`, `InvalidHandshake`, `UnreadableStream`, `StreamTimeout`

### `lib.rs`
- **Role**: Crate entrypoint and shared enums for message exchange at the codec boundary.
- **Key items**: `EgressECIESValue` (`Auth`, `Ack`, `Message`), `IngressECIESValue` (`AuthReceive`, `Ack`, `Message`), re-exports `ECIESError`/`ECIESErrorImpl`

### `mac.rs`
- **Role**: Ethereum's nonstandard RLPx MAC construction (AES-256 + Keccak-256) used to authenticate framed headers and bodies.
- **Key items**: `MAC`, `update()`, `update_header()`, `update_body()`, `digest()`
- **Knobs / invariants**: Designed specifically for the RLPx framing rules (128-bit digest slices, AES-ECB-like block usage).

### `stream.rs`
- **Role**: High-level async wrapper over `AsyncRead`/`AsyncWrite`: performs the ECIES handshake (client/server) and exposes encrypted message exchange as a `Stream`/`Sink`.
- **Key items**: `ECIESStream`, `connect()` / `connect_with_timeout()`, `incoming()`, `HANDSHAKE_TIMEOUT`
- **Interactions**: Wraps `Framed<Io, ECIESCodec>`; uses `IngressECIESValue`/`EgressECIESValue` to coordinate handshake.
- **Knobs / invariants**: Enforces handshake timeouts and maps "remote closed before readable" into `UnreadableStream`.

### `util.rs`
- **Role**: Small cryptographic helpers used by the ECIES implementation.
- **Key items**: `sha256()`, `hmac_sha256()`

## End-to-end flow (high level)
- Client creates `ECIESStream::connect(...)` with its secret key and the remote peer ID.
- `ECIESCodec` begins in `Auth` state and sends `EgressECIESValue::Auth`; server reads it as `IngressECIESValue::AuthReceive(peer_id)`.
- Server responds with `EgressECIESValue::Ack`; client validates it as `IngressECIESValue::Ack`.
- After handshake, both sides derive symmetric keys and transition to reading/writing framed messages (`Header`/`Body`).
- Each outbound message is encoded as header + body with integrity tags and encrypted payload; inbound frames are verified and decrypted before yielding bytes.

## Key APIs (no snippets)
- **Types**: `ECIES`, `ECIESCodec`, `ECIESStream`, `MAC`
- **Enums**: `ECIESState`, `EgressECIESValue`, `IngressECIESValue`
