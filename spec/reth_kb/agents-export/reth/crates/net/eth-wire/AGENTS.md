# eth-wire

## Purpose
`reth-eth-wire` crate: implements RLPx `p2p` + `eth` wire protocol stream plumbing-hello/capability negotiation, message-id multiplexing, ping/pong liveness, eth status handshake, and typed `EthStream`/`P2PStream` adapters over encrypted transports (ECIES).

## Contents (one hop)
### Subdirectories
- [x] `src/` - core stream/handshake/multiplexing implementations and error types.
- [x] `tests/` - integration tests and fuzz/roundtrip tests for on-wire payloads.
- [x] `testdata/` - (skip: test fixtures) captured/serialized network payload hex used by tests.

### Files
- `Cargo.toml` - crate manifest and feature flags.
  - **Key items**: features `serde`, `arbitrary`; deps `reth-ecies`, `reth-eth-wire-types`, `snap` (snappy), `tokio-util` (codec)

## Key APIs (no snippets)
- `UnauthedP2PStream` / `P2PStream`
- `UnauthedEthStream` / `EthStream`
- `RlpxProtocolMultiplexer`
