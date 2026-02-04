# builder

## Purpose
Configuration types for `eth` namespace RPC behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for builder/config types.
- **Key items**: `config`

### `config.rs`
- **Role**: Defines `eth` RPC configuration structures and defaults.
- **Key items**: `EthConfig`, `EthFilterConfig`, `PendingBlockKind`, `DEFAULT_STALE_FILTER_TTL`
- **Knobs / invariants**: Gas caps, proof window, tracing limits, cache sizes, and pending block behavior.

## End-to-end flow (high level)
- Build `EthConfig` with defaults or CLI overrides.
- Derive `EthFilterConfig` and pending-block behavior from config.
- Feed config into RPC handlers and caches.

## Key APIs (no snippets)
- `EthConfig`, `EthFilterConfig`, `PendingBlockKind`
