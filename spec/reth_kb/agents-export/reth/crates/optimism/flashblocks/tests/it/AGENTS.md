# it

## Purpose
Integration tests for flashblock websocket streaming.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness for integration tests.
- **Key items**: module `stream`
- **Interactions**: Aggregates websocket stream tests.

### `stream.rs`
- **Role**: Verifies that websocket streaming yields decodable flashblocks from a remote source.
- **Key items**: `test_streaming_flashblocks_from_remote_source_is_successful`
- **Interactions**: Uses `WsFlashBlockStream` to fetch and decode items.

## End-to-end flow (high level)
- Construct a `WsFlashBlockStream` targeting a flashblocks endpoint.
- Consume a small number of items and assert they decode successfully.

## Key APIs (no snippets)
- `WsFlashBlockStream`
