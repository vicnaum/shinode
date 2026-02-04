# src

## Purpose
Optimism chain specs and base fee utilities, including Base/OP mainnet & testnet specs.

## Contents (one hop)
### Subdirectories
- [x] `superchain/` - Superchain registry-derived chain specs (feature-gated).

### Files
- `lib.rs` - Crate entrypoint and `OpChainSpec` builder/types.
  - **Key items**: `OpChainSpecBuilder`, `OpChainSpec`, `BASE_MAINNET`, `OP_MAINNET`, `OP_SEPOLIA`, `BASE_SEPOLIA`, `OP_DEV`
- `constants.rs` - OP/Base chain constants and dev L1 info tx.
  - **Key items**: `BASE_MAINNET_MAX_GAS_LIMIT`, `BASE_SEPOLIA_MAX_GAS_LIMIT`, `TX_SET_L1_BLOCK_OP_MAINNET_BLOCK_124665056`
- `base.rs` - Base mainnet chain spec.
  - **Key items**: `BASE_MAINNET`
- `base_sepolia.rs` - Base Sepolia chain spec.
  - **Key items**: `BASE_SEPOLIA`
- `op.rs` - Optimism mainnet chain spec.
  - **Key items**: `OP_MAINNET`
- `op_sepolia.rs` - Optimism Sepolia chain spec.
  - **Key items**: `OP_SEPOLIA`
- `dev.rs` - Dev chain spec with prefunded accounts.
  - **Key items**: `OP_DEV`
- `basefee.rs` - Base fee calculation helpers for Holocene/Jovian.
  - **Key items**: `decode_holocene_base_fee()`, `compute_jovian_base_fee()`

## Key APIs (no snippets)
- `OpChainSpec`, `OpChainSpecBuilder`
