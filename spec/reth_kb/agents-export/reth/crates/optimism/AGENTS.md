# optimism

## Purpose
Optimism-specific crates for op-reth: chain specs, consensus/EVM, node wiring, RPC, and supporting tooling.

## Contents (one hop)
### Subdirectories
- [x] `bin/` - op-reth binary entrypoint and feature wiring.
- [x] `chainspec/` - OP/Base chain specs and superchain registry helpers.
- [x] `cli/` - op-reth CLI with OP import codecs and commands.
- [x] `consensus/` - OP consensus rules and receipt root handling.
- [x] `evm/` - OP EVM config, L1 info parsing, and block assembly.
- [x] `flashblocks/` - Flashblocks streaming, pending block assembly, and consensus updates.
- [x] `hardforks/` - OP/Base hardfork schedule definitions.
- [x] `node/` - OP node types, builders, engine/RPC wiring, and tests.
- [x] `payload/` - OP payload builder and validator.
- [x] `primitives/` - OP primitive types, receipts, and transactions.
- [x] `reth/` - OP meta-crate re-exports.
- [x] `rpc/` - OP RPC implementations for engine/eth/sequencer endpoints.
- [x] `storage/` - OP storage alias types.
- [x] `txpool/` - OP transaction pool types, validation, and maintenance.

### Files
- (none)
