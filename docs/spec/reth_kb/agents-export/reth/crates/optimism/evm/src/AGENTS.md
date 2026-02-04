# src

## Purpose
Optimism EVM configuration and block execution helpers, including L1 info parsing and OP block assembly.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and `OpEvmConfig` implementation.
  - **Key items**: `OpEvmConfig`, `OpBlockAssembler`, `OpRethReceiptBuilder`, `OpBlockExecutionError`
- `config.rs` - OP next-block env attributes and revm spec helpers.
  - **Key items**: `OpNextBlockEnvAttributes`, `revm_spec`, `revm_spec_by_timestamp_after_bedrock`
- `execute.rs` - Executor helpers and backwards-compatible provider alias.
  - **Key items**: `OpExecutorProvider`
- `l1.rs` - L1 block info parsing for Bedrock/Ecotone/Isthmus/Jovian.
  - **Key items**: `extract_l1_info()`, `parse_l1_info_tx_ecotone()`, `parse_l1_info_tx_isthmus()`
- `receipts.rs` - Receipt builder for OP transactions.
  - **Key items**: `OpRethReceiptBuilder`
- `error.rs` - OP execution error types.
  - **Key items**: `L1BlockInfoError`, `OpBlockExecutionError`
- `build.rs` - OP block assembler implementation.
  - **Key items**: `OpBlockAssembler`

## Key APIs (no snippets)
- `OpEvmConfig`, `OpBlockAssembler`
- `OpRethReceiptBuilder`, `OpNextBlockEnvAttributes`
