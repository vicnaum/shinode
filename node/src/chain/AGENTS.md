# chain

Shared Ethereum/Base identity, wire-type bundles, and fork-aware payload helpers.
Exported by `src/lib.rs` for both the binary and protocol integration tests.

- `mod.rs`: `ChainKind` (mainnet default, Base), resolved chain ID/genesis, and `ChainTypes`.
- `ethereum.rs`: `EthereumChain`, stock Reth mainnet spec and receipt/withdrawals rules.
- `base.rs`: `BaseChain`, OP primitives, verified Azul/Beryl overrides, Base execution bootnodes and `basev0` discovery settings.
- `op_stack.rs`: timestamp-aware OP receipt roots and Canyon/Isthmus withdrawals handling.

Adapted from Sieve 914d29f, MIT OR Apache-2.0. See `docs/base-protocol.md` for
the pinned upstream revision and compatibility boundary. The binary dispatches `ChainKind` once into the matching typed runtime.
`ChainKind::display_name()` supplies user-facing chain names; configuration must
never relabel Ethereum networking as Base.

`cargo test --lib` runs fork/genesis/withdrawals tests. The `base_protocol`
integration test consumes these definitions rather than a second chainspec.

`ChainTypes::Receipt` requires `FullReceipt`, exposing encoding and receipt traits to the generic sync pipeline. Receipt-root helpers recompute blooms from logs.
