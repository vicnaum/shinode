# src

## Purpose
Payload primitives and validation helpers for execution payloads, attributes, and builder errors.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint defining `PayloadTypes` and validation helpers across engine versions.
- **Key items**: `PayloadTypes`, `validate_payload_timestamp()`, `validate_withdrawals_presence()`, `validate_parent_beacon_block_root_presence()`, `validate_version_specific_fields()`
- **Interactions**: Re-exports `PayloadBuilderError` and validation error types from `error.rs`.

### `payload.rs`
- **Role**: Execution payload trait abstraction and wrapper for payloads vs attributes.
- **Key items**: `ExecutionPayload`, `PayloadOrAttributes`
- **Interactions**: Implements `ExecutionPayload` for `ExecutionData` and OP execution data (feature-gated).

### `traits.rs`
- **Role**: Core payload traits for built payloads and attribute types.
- **Key items**: `BuiltPayload`, `BuiltPayloadExecutedBlock`, `PayloadBuilderAttributes`, `PayloadAttributes`, `PayloadAttributesBuilder`, `BuildNextEnv`
- **Interactions**: Bridges execution results and trie updates into `BuiltPayloadExecutedBlock`.

### `error.rs`
- **Role**: Error types for payload building and validation.
- **Key items**: `PayloadBuilderError`, `EngineObjectValidationError`, `VersionSpecificValidationError`, `NewPayloadError`

## End-to-end flow (high level)
- Define payload types via `PayloadTypes` and related trait implementations.
- Validate payload/attribute fields against engine version rules.
- Use payload traits to expose built payload data and execution outcomes.

## Key APIs (no snippets)
- `PayloadTypes`, `ExecutionPayload`, `BuiltPayload`
- `PayloadBuilderError`, `EngineObjectValidationError`
