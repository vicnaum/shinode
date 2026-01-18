# state

## Purpose
State provider implementations for latest, historical, and overlay views.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: State provider module entrypoint and exports.
- **Key items**: `HistoricalStateProvider`, `LatestStateProvider`, `OverlayStateProvider`

### `historical.rs`
- **Role**: Historical state provider implementation and helpers.
- **Key items**: `HistoryInfo`, `LowestAvailableBlocks`

### `latest.rs`
- **Role**: Latest state provider implementation.
- **Key items**: `LatestStateProvider`

### `overlay.rs`
- **Role**: Overlay state provider for temporary state.
- **Key items**: `OverlayStateProvider`, `OverlayStateProviderFactory`
