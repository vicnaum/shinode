# constants

## Purpose
Protocol constants and gas unit helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Protocol constants and re-exports for gas units.
- **Key items**: `RETH_CLIENT_VERSION`, `MINIMUM_GAS_LIMIT`, `MAXIMUM_GAS_LIMIT_BLOCK`, `GAS_LIMIT_BOUND_DIVISOR`
- **Interactions**: Re-exports `KILOGAS`, `MEGAGAS`, `GIGAGAS`.

### `gas_units.rs`
- **Role**: Gas unit constants and formatting helpers.
- **Key items**: `KILOGAS`, `MEGAGAS`, `GIGAGAS`, `format_gas()`, `format_gas_throughput()`

## End-to-end flow (high level)
- Use gas unit constants for display or calculations.
- Format gas values and throughput strings for logging.

## Key APIs (no snippets)
- `format_gas()`, `format_gas_throughput()`
