# ethereum

## Purpose
Ethereum-specific crates for reth: Ethereum node wiring, EVM configuration, consensus validation, payload handling, primitives/types, hardfork schedules, and CLI integration for running a canonical Ethereum reth node.

## Contents (one hop)
### Subdirectories
- [x] `cli/` - Ethereum `reth` CLI surface: clap `Cli` type, Ethereum chain spec parsing, tracing/metrics init, and command dispatch using Ethereum node components.
- [x] `consensus/` - Ethereum beacon consensus validator: hardfork-aware header/body checks and post-execution validation (receipts root/bloom, gas used, Prague requests hash).
- [x] `engine-primitives/` - Ethereum Engine API primitives: `EthEngineTypes`/`EthBuiltPayload` and conversions to execution payload envelopes (V1-V5), including blob sidecars and requests handling.
- [x] `evm/` - Ethereum EVM config: `EthEvmConfig` for env derivation and execution, block assembly (roots + fork-gated fields), receipt building, and fork-behaviour tests.
- [x] `hardforks/` - Ethereum fork types/schedules: `ChainHardforks` + `Hardforks` trait, fork-id/filter utilities, dev hardfork schedule, and pretty-printing of fork activation timelines.
- [x] `node/` - Ethereum node wiring: `EthereumNode` type config, component builders (pool/network/executor/consensus), payload builder + Engine API validators, and RPC add-ons for a full Ethereum node.
- [x] `payload/` - Ethereum payload builder: selects best txs from the pool, executes with Ethereum EVM config, enforces gas/RLP/blob limits, produces `EthBuiltPayload`, and validates well-formed Engine API payloads.
- [x] `primitives/` - Ethereum primitives: canonical block/tx/receipt types (`EthPrimitives`) based on alloy envelopes, plus receipt encoding/decoding and receipt-root helpers.
- [x] `reth/` - Feature-gated meta crate: re-exports Ethereum primitives and (optionally) consensus/EVM/node/network/provider/RPC subsystems behind a single dependency.

### Files
- (none)

## Notes
- This directory is an "umbrella" group; details live in each child crate's `AGENTS.md`.
