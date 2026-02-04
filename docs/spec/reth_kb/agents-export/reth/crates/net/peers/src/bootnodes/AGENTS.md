# bootnodes

## Purpose
Bootnode lists and helpers for constructing initial peer sets for different networks (Ethereum mainnet/testnets and OP-stack networks).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - exposes bootnode lists and helpers that parse them into `NodeRecord` values.
  - **Key items**: `mainnet_nodes()`, `sepolia_nodes()`, `holesky_nodes()`, `hoodi_nodes()`, `op_nodes()`, `op_testnet_nodes()`, `parse_nodes()`
- `ethereum.rs` - Ethereum Foundation bootnode lists for mainnet and testnets.
  - **Key items**: `MAINNET_BOOTNODES`, `SEPOLIA_BOOTNODES`, `HOLESKY_BOOTNODES`, `HOODI_BOOTNODES`
- `optimism.rs` - OP-stack bootnode lists for mainnet/testnet (including Base endpoints).
  - **Key items**: `OP_BOOTNODES`, `OP_TESTNET_BOOTNODES`
