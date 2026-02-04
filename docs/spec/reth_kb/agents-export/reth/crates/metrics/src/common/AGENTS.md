# common

## Purpose
Common metric utilities for `reth-metrics`, primarily metered wrappers around tokio mpsc channels to expose counters for sends/receives and errors.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - exports common utilities (currently the `mpsc` module).
- `mpsc.rs` - metered tokio mpsc wrappers (`MeteredSender`/`MeteredReceiver` and unbounded variants), plus helpers to construct metered channels.
  - **Key items**: `metered_channel()`, `metered_unbounded_channel()`, `MeteredSender`, `MeteredReceiver`, `UnboundedMeteredSender`, `UnboundedMeteredReceiver`
