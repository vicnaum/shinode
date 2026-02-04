# src

## Purpose
Implements Ethereum beacon-chain style consensus validation (`reth-ethereum-consensus`): header/body pre-execution checks and post-execution checks aligned with Ethereum hardfork rules (Merge/Shanghai/Cancun/Prague, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines `EthBeaconConsensus` and implements `reth_consensus::{Consensus, HeaderValidator, FullConsensus}` traits for Ethereum chainspecs.
- **Key items**: `EthBeaconConsensus`, `EthBeaconConsensus::new()`, `HeaderValidator::validate_header()`, `HeaderValidator::validate_header_against_parent()`
- **Interactions**: delegates to `reth_consensus_common::validation::*` helpers; uses chainspec hardfork queries to gate EIP-specific header fields.
- **Knobs / invariants**: `max_extra_data_size` (defaults to `MAXIMUM_EXTRA_DATA_SIZE`); enforces Merge-specific header constraints (difficulty/nonce/ommers).

#### `validation.rs`
- **Role**: Implements post-execution validation: receipts root + logs bloom correctness, gas used correctness, and Prague requests hash validation.
- **Key items**: `validate_block_post_execution()`
- **Interactions**: consumes execution receipts + `Requests` and compares against header fields; returns `ConsensusError` variants with `GotExpected` diffs.

## Key APIs (no snippets)
- `EthBeaconConsensus`
- `validate_block_post_execution()`

## Relationships
- **Builds on**: `reth-consensus-common` (pre-exec/header helpers) and `reth-consensus` traits.
