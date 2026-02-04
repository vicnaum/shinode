# src

## Purpose
Optimism payload builder implementation, payload types, and validation utilities.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and `OpPayloadTypes` aggregation.
  - **Key items**: `OpPayloadTypes`, `OpPayloadBuilder`, `OpExecutionPayloadValidator`
- `builder.rs` - Optimism payload builder logic.
  - **Key items**: `OpPayloadBuilder`, `build_payload()`
- `config.rs` - Builder configuration for data availability and gas limits.
  - **Key items**: `OpBuilderConfig`, `OpDAConfig`, `OpGasLimitConfig`
- `error.rs` - Optimism payload builder errors.
  - **Key items**: `OpPayloadBuilderError`
- `payload.rs` - Payload attribute types, payload ID derivation, and built payload wrapper.
  - **Key items**: `OpPayloadBuilderAttributes`, `OpBuiltPayload`, `payload_id_optimism()`
- `traits.rs` - Helper traits for OP payload primitives and attributes.
  - **Key items**: `OpPayloadPrimitives`, `OpAttributes`
- `validator.rs` - Execution payload validator for OP hardfork rules.
  - **Key items**: `OpExecutionPayloadValidator`, `ensure_well_formed_payload()`

## Key APIs (no snippets)
- `OpPayloadBuilder`, `OpPayloadTypes`
- `OpPayloadBuilderAttributes`, `OpBuiltPayload`
