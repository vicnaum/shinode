# src

## Purpose
Implements `reth-consensus-common`: reusable, chain-spec-aware block/header/body validation helpers (pre-execution / "stateless" checks) that are shared across consensus implementations.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint exposing the `validation` module.
- **Key items**: `validation` module
- **Interactions**: consumed by consensus/validation callers that want shared helper functions.
- **Knobs / invariants**: `no_std` when `std` feature is disabled.

#### `validation.rs`
- **Role**: Collection of validation routines that check header/body invariants and fork-activated fields without requiring state execution (e.g. gas limits, roots, withdrawals, blob gas, block size limits).
- **Key items**: `validate_header_gas()`, `validate_header_base_fee()`, `validate_body_against_header()`, `validate_block_pre_execution()`, `post_merge_hardfork_fields()`, `validate_4844_header_standalone()`, `MAX_RLP_BLOCK_SIZE`
- **Interactions**: relies on `reth_chainspec::EthereumHardforks` to gate validations by fork activation; returns `reth_consensus::ConsensusError` to unify error reporting.
- **Knobs / invariants**: enforces fork-specific requirements (Shanghai withdrawals, Cancun blob gas fields, Osaka block size / tx gas-limit constraints) based on the provided chainspec and header timestamps/numbers.

## Key APIs (no snippets)
- **Constants**: `MAX_RLP_BLOCK_SIZE`
- **Header validation**: `validate_header_gas()`, `validate_header_base_fee()`, `validate_header_extra_data()`, `validate_against_parent_hash_number()`
- **Body/block validation**: `validate_body_against_header()`, `validate_block_pre_execution()`, `post_merge_hardfork_fields()`
- **Fork-specific**: `validate_shanghai_withdrawals()`, `validate_cancun_gas()`, `validate_4844_header_standalone()`

## Relationships
- **Used by**: consensus engines and other components that need consistent "pre-exec" validation behavior.
- **Depends on**: `reth-consensus` for shared error types, `reth-chainspec` for fork activation checks, and primitives traits for block/header/body abstractions.

## End-to-end flow (high level)
- Take a sealed header/block + chainspec.
- Apply basic invariants (gas limit bounds, required header fields).
- Apply fork-activated rules (withdrawals, blob gas, size limits) based on block number/timestamp.
- Compare body-derived roots/hashes to header roots/hashes.
- Return a `ConsensusError` describing the first violated invariant.
