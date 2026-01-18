# src

## Purpose
Implements `reth-chainspec` core APIs for describing an Ethereum-like network: chain identity, hardfork schedule, genesis/header construction, and chain-spec access traits.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `api.rs` - `EthChainSpec` trait: chain-spec queries used across the codebase (fork activation, params lookups, etc.).
- `constants.rs` - chain-specific constants and convenience re-exports.
- `info.rs` - `ChainInfo` type (network identity/metadata used by the node).
- `lib.rs` - crate entrypoint; re-exports `ChainSpec`, `ChainSpecBuilder`, fork types, and built-in chain presets (e.g. `MAINNET`, `SEPOLIA`, `HOLESKY`).
- `spec.rs` - `ChainSpec` / `ChainSpecBuilder` implementations and helpers (genesis header creation, base-fee params, deposit contract metadata, etc.).

## Key APIs (no snippets)
- **Traits**: `EthChainSpec`, `ChainSpecProvider`
- **Types**: `ChainSpec`, `ChainSpecBuilder`, `ChainInfo`, `BaseFeeParams`, `BaseFeeParamsKind`, `ForkBaseFeeParams`, `DepositContract`
- **Constants/presets**: `MAINNET`, `SEPOLIA`, `HOLESKY`, `HOODI`, `DEV`
- **Helpers**: `make_genesis_header()`, `create_chain_config()`, `mainnet_chain_config()`

## Relationships
- **Used by**: most subsystems that need fork activation checks and chain parameters (consensus, engine, EVM config, network handshake/status, etc.).
- **Depends on**: `reth-ethereum-forks` (hardfork types/schedule), `alloy-genesis` + `res/genesis/*.json` (genesis inputs), `reth-network-peers` (peer/network integration helpers).
