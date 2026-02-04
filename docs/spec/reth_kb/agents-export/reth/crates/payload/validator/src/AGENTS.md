# src

## Purpose
Payload validation helpers for Shanghai, Cancun, and Prague protocol rules.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring for payload validation rules.
- **Key items**: modules `shanghai`, `cancun`, `prague`

### `shanghai.rs`
- **Role**: Validation for withdrawals field based on Shanghai activation.
- **Key items**: `ensure_well_formed_fields()`
- **Knobs / invariants**: Withdrawals must be present post-Shanghai and absent pre-Shanghai.

### `cancun.rs`
- **Role**: Validation for Cancun header fields, sidecar fields, and blob tx hashes.
- **Key items**: `ensure_well_formed_fields()`, `ensure_well_formed_header_and_sidecar_fields()`, `ensure_matching_blob_versioned_hashes()`
- **Knobs / invariants**: Blob gas fields and versioned hashes must match Cancun activation.

### `prague.rs`
- **Role**: Validation for Prague sidecar fields and EIP-7702 tx presence.
- **Key items**: `ensure_well_formed_fields()`, `ensure_well_formed_sidecar_fields()`
- **Knobs / invariants**: Prague request fields must be absent before Prague.

## End-to-end flow (high level)
- Determine hardfork activation for the payload timestamp.
- Apply Shanghai/Cancun/Prague checks to block body and sidecar fields.
- Reject payloads with unsupported fields for the active fork.

## Key APIs (no snippets)
- `ensure_well_formed_fields` (Shanghai/Cancun/Prague)
