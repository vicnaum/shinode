# src

## Purpose
EthStats client implementation: websocket connection management, protocol message types, and the main reporting service.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and public re-exports for EthStats service and message types.
  - **Key items**: `ethstats`, `events`
- `ethstats.rs` - Main EthStats service: connects, authenticates, reports stats, and streams block/tx updates.
  - **Key items**: `EthStatsService`, `HISTORY_UPDATE_RANGE`, `REPORT_INTERVAL`, `connect()`, `login()`
- `connection.rs` - WebSocket connection wrapper with JSON read/write helpers.
  - **Key items**: `ConnWrapper`, `WsStream`, `read_json()`, `write_json()`
- `credentials.rs` - Parses and stores EthStats connection credentials.
  - **Key items**: `EthstatsCredentials`
- `events.rs` - EthStats protocol message structures for node/block/pending/stats updates.
  - **Key items**: `NodeInfo`, `AuthMsg`, `BlockMsg`, `HistoryMsg`, `PendingMsg`, `StatsMsg`, `LatencyMsg`
- `error.rs` - Connection and service error types.
  - **Key items**: `ConnectionError`, `EthStatsError`

## Key APIs (no snippets)
- `EthStatsService`
- `EthStatsError`
