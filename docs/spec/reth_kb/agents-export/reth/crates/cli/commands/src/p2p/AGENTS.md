# p2p

## Purpose
Implements `reth p2p`: a CLI debugging tool for networking/P2P operations (fetching headers/bodies via the fetch client, simple RLPx handshakes, and running a discovery-only bootnode).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command definitions and dispatch for `reth p2p` subcommands (header/body download + RLPx + bootnode).
  - **Key items**: `Command<C>`, `Subcommands<C>`, `DownloadArgs<C>`, `DownloadArgs::launch_network()`, `DownloadArgs::backoff()`
- `bootnode.rs` - standalone bootnode: runs discv4 (and optional discv5) for discovery-only operation.
  - **Key items**: `bootnode::Command`, `Discv4`, `Discv5`, `NatResolver`
- `rlpx.rs` - RLPx utilities (currently a ping/handshake flow to print the peer's `HelloMessage`).
  - **Key items**: `rlpx::Command`, `ECIESStream`, `UnauthedP2PStream`, `HelloMessage`

## Key APIs (no snippets)
- **CLI types**: `Command<C>`, `Subcommands<C>`, `DownloadArgs<C>`, `bootnode::Command`, `rlpx::Command`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` as part of the CLI command suite.
- **Depends on**: `reth-network` + `reth-network-p2p` (network startup + fetch client), `reth-eth-wire` (RLPx handshake messages), `reth-discv4`/`reth-discv5` (discovery services).
