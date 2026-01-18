# ws

## Purpose
WebSocket client utilities for streaming flashblocks and decoding payloads.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for websocket stream and decoder types.
- **Key items**: `WsFlashBlockStream`, `WsConnect`, `FlashBlockDecoder`
- **Interactions**: Re-exports `decoding` and `stream` modules.

### `decoding.rs`
- **Role**: Decoder trait and helpers to parse flashblock payloads from bytes.
- **Key items**: `FlashBlockDecoder`, `decode_flashblock()`, `try_parse_message()`
- **Interactions**: Used by `WsFlashBlockStream` to decode incoming frames.
- **Knobs / invariants**: Accepts JSON payloads directly or brotli-decompresses non-JSON frames.

### `stream.rs`
- **Role**: Stateful websocket stream that connects, decodes messages, and handles ping/pong.
- **Key items**: `WsFlashBlockStream`, `WsConnect`, `WsConnector`, `State`
- **Interactions**: Uses `FlashBlockDecoder` for payload decoding and `tokio_tungstenite` for IO.
- **Knobs / invariants**: Retries connections indefinitely; responds to ping with pong.

## End-to-end flow (high level)
- Create `WsFlashBlockStream` with a websocket URL and optional decoder.
- Connect via `WsConnector` (or custom `WsConnect`) and split into sink/stream.
- Decode incoming frames into `FlashBlock` values.
- Handle ping/pong and reconnect on connection errors.

## Key APIs (no snippets)
- `WsFlashBlockStream`, `WsConnect`, `FlashBlockDecoder`
