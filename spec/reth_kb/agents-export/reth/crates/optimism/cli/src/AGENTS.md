# src

## Purpose
Optimism CLI wiring: command parsing, chain spec selection, and import codecs.

## Contents (one hop)
### Subdirectories
- [x] `commands/` - CLI subcommands for node, imports, init-state, and test vectors.

### Files
- `lib.rs` - CLI entrypoint and `Cli` struct for op-reth.
  - **Key items**: `Cli`, `Cli::run()`, re-exports `CliApp`, `ImportOpCommand`
- `app.rs` - CLI runner wrapper that initializes tracing and executes commands.
  - **Key items**: `CliApp`
- `chainspec.rs` - Optimism chain spec parser and value parser.
  - **Key items**: `OpChainSpecParser`, `chain_value_parser()`
- `ovm_file_codec.rs` - OVM block/transaction file codec for pre-bedrock imports.
  - **Key items**: `OvmBlockFileCodec`, `OvmBlock`, `OvmTransactionSigned`
- `receipt_file_codec.rs` - Receipt file codec for op-geth export format.
  - **Key items**: `OpGethReceiptFileCodec`, `OpGethReceipt`

## Key APIs (no snippets)
- `Cli`, `CliApp`
- `OpChainSpecParser`
