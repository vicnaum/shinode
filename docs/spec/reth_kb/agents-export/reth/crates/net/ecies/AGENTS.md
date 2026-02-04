# ecies

## Purpose
`reth-ecies` crate: RLPx ECIES framed transport implementation (Ethereum devp2p) providing handshake (AUTH/ACK), encrypted framing (header/body), and Tokio-friendly codecs/streams for use over TCP transports.

## Contents (one hop)
### Subdirectories
- [x] `src/` - core ECIES crypto + framing, Tokio codec, and async stream wrapper.

### Files
- `Cargo.toml` - crate manifest.
  - **Key items**: deps `tokio-util` (codec), `secp256k1`, `aes`/`ctr`, `sha2`/`hmac`, `reth-network-peers`

## Key APIs (no snippets)
- `ECIESStream`
- `ECIESCodec`
- `ECIES` (handshake/framing engine)
