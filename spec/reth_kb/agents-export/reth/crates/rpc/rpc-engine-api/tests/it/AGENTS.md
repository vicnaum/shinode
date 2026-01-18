# it

## Purpose
Integration tests for engine payload encoding and validation helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for payload tests.
- **Key items**: `payload`

### `payload.rs`
- **Role**: Payload conversion and validation test cases.
- **Key items**: `payload_body_roundtrip()`, `payload_validation_conversion()`, `ExecutionPayloadV1`

## End-to-end flow (high level)
- Generate random blocks and payload bodies.
- Verify payload body transaction decoding roundtrips.
- Validate payload conversion error cases.

## Key APIs (no snippets)
- `ExecutionPayloadV1`, `ExecutionPayloadBodyV1`
