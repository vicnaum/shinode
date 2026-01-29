# UI Dashboard Designs

This document contains the UI designs for the Stateless History Node dashboard using Ratatui. These designs have been **implemented** in `node/src/ui/tui.rs`.

## Status

**Implemented.** The TUI dashboard is the default UI mode. Disable with `--no-tui` to fall back to legacy indicatif progress bars.

Preview the dashboard without running sync:
```bash
cargo run --manifest-path node/Cargo.toml --bin ui-mock
```

## Overview

The UI is a fullscreen terminal dashboard with:
- Phase indicator at top (STARTUP > SYNC > RETRY > COMPACT > SEAL > FOLLOW)
- Progress visualization (bar + blocks coverage map with braille characters)
- Speed chart with 1-minute history (braille line graph with filled area gradient)
- Network/Queue/Storage/DB panels (4-column grid)
- RPC panel (follow mode only)
- Log panel at bottom (real-time tracing events)
- DOS-style splash screen during initial startup

## Color Scheme

| Element | Color | Notes |
|---------|-------|-------|
| STARTUP phase | Yellow | Splash screen, connecting |
| SYNC phase | Cyan | Active during fetch |
| RETRY phase | Red | Escalation queue active |
| COMPACT phase | Magenta | Active during compaction |
| SEAL phase | Green | Active during sealing |
| FOLLOW phase | Bright Green | Active when synced |
| Completed phase | Dim/Gray | checkmark |
| Current speed | Yellow | Stands out in stats |
| Peak speed | Bright Cyan | Achievement highlight |
| Average speed | White | Baseline |
| Warnings in logs | Yellow | WARN level |
| Errors in logs | Red | ERROR level |
| Progress bar fill | Phase color | Matches current phase |

## Phase Transitions

Phases are ordered (0-5) and never transition backwards:
- Exception: Startup can transition to any phase
- Exception: Transition to Follow is always allowed

This prevents race conditions from snapshot updates arriving out of order.

## Blocks Map Legend

The "Blocks" visualization shows sync coverage using braille characters in a 2-row grid:

- Column-major layout (top-bottom, left-right)
- Each character represents a coverage bucket (0-100%)
- Color gradient: Dark Gray (0%) > Yellow (1-40%) > Light Green (41-80%) > Green (81-100%)
- Labels: Start block number (left), end block (right)

## Peer Visualization

The Network panel shows peer dots that never shrink (uses peak high-water mark):
- `active` (green filled) - Actively fetching blocks
- `idle` (green circle) - Healthy, available
- `stale` (gray) - Banned for stale head (cooling down)
- `gone` (very dim) - Disconnected since peak

## Speed Chart

7-row block with braille line graph:
- Y-axis: Smart scaling with labels (top, mid, bottom)
- X-axis: "-1m" to "now" time range
- Line: Filled area under curve with gradient colors (red > orange > yellow-green > green)
- Marker: White dot on current value
- Stats line: "Cur: X/s  Avg: Y/s  Peak: Z/s  ETA: hh:mm:ss"

---

## SYNC Mode

```
+------------------------------------------------------------------------------+
|  STATELESS HISTORY NODE                                            14:32:05  |
|                                                                              |
|  Phase:  [# SYNC]  [. COMPACT]  [. SEAL]  [. FOLLOW]                        |
|           cyan      magenta      green     bright green                      |
+------------------------------------------------------------------------------+
|                                                                              |
|  Progress   ==================-----------  68.5%     ETA 42m 18s             |
|            10,000,000 --------------------------------> 21,000,000           |
|                              3,152,769 blocks synced                         |
|                                                                              |
|  Blocks    ....==========================================.....................|
|  Map       ....====================================......  .................. |
|            10M                                                          21M  |
|                                                                              |
+------------------------------------------------------------------------------+
|                                                                              |
|  Speed (blocks/s)                                                            |
|  2.0K |                                                                      |
|       |         ____      ______              ____                           |
|  1.5K |    ____======____========____    ____======____                      |
|       |___============================================___________            |
|  1.0K |==============================================================__     |
|       |                                                            ==       |
|  0.5K |                                                            ==       |
|     0 +----------------------------------------------------------------     |
|        -1m                                                          now      |
|   * Current: 1,247/s     + Average: 1,180/s     * Peak: 1,892/s             |
|                                                                              |
+------------------------------------------------------------------------------+
|  NETWORK           |  QUEUE              |  STORAGE            |  DB         |
|  -------           |  -----              |  -------            |  --         |
|  Peers  @@@@@@oooo |  Remaining    284   |  Headers   2.4 GiB  |  Blocks  3M |
|          6 / 10    |  Inflight      48   |  Txns     18.7 GiB  |  Txns  412M |
|  Stale         0   |  Retry          3   |  Receipts  1.2 GiB  |  Logs 1.2B  |
|  Chain  21,453,892 |                     |  Total    22.3 GiB  |  Shards 4/8 |
|                    |                     |  Write   12.4 MB/s  |             |
+--------------------+---------------------+---------------------+-------------+
|  Logs                                                                        |
|  ----                                                                        |
|  14:32:05 INFO  Fetched batch 1847: blocks 13,245,000..13,245,128           |
|  14:32:04 INFO  Peer 0x7f3a..2c1b connected (eth/68)                        |
|  14:32:03 INFO  Committed shard 1320 (13,200,000..13,209,999)               |
|  14:32:01 WARN  Peer 0x2d8c..9e4a timeout, retrying batch 1843              |
|                                                                              |
+------------------------------------------------------------------------------+
```

---

## COMPACT Mode

```
+------------------------------------------------------------------------------+
|  STATELESS HISTORY NODE                                            14:45:22  |
|                                                                              |
|  Phase:  [v SYNC]  [# COMPACT]  [. SEAL]  [. FOLLOW]                        |
|           dim       magenta      green     bright green                      |
+------------------------------------------------------------------------------+
|                                                                              |
|  Compacting   ================---------------------  42%      18 / 43 shards |
|                                                                              |
|  Shards      [==========--------=ooooooooooooooooooooooooo]                  |
|              = done   - compacting   o pending                               |
|                                                                              |
+------------------------------------------------------------------------------+
|  NETWORK           |  COMPACTION         |  STORAGE            |  DB         |
|  -------           |  ----------         |  -------            |  --         |
|  Peers  @@@@@@oooo |  Done          18   |  Headers   2.4 GiB  |  Blocks  3M |
|          6 / 10    |  Current        2   |  Txns     18.7 GiB  |  Txns  412M |
|  Chain  21,453,892 |  Pending       23   |  Receipts  1.2 GiB  |  Logs 1.2B  |
|                    |                     |  Total    22.3 GiB  |  Shards 4/8 |
+--------------------+---------------------+---------------------+-------------+
|  Logs                                                                        |
|  14:45:22 INFO  Compacting shard 1328 (13,280,000..13,289,999)              |
|  14:45:20 INFO  Compacted shard 1327 in 2.3s                                |
|  14:45:17 INFO  Compacted shard 1326 in 2.1s                                |
|                                                                              |
+------------------------------------------------------------------------------+
```

---

## FOLLOW Mode

```
+------------------------------------------------------------------------------+
|  STATELESS HISTORY NODE                                            14:52:18  |
|                                                                              |
|  Phase:  [v SYNC]  [v COMPACT]  [v SEAL]  [# FOLLOW]                        |
|           dim       dim          dim       bright green                      |
+------------------------------------------------------------------------------+
|                                                                              |
|     Ethereum Mainnet                                                         |
|     v SYNCED                                        222   111   444          |
|     Last block 3s ago                                 2     1     4          |
|                                                     222   111   444          |
|                                                       2     1   4            |
|                                                     222   111   444          |
|                                                                              |
+------------------------------------------------------------------------------+
|  NETWORK           |  STORAGE            |  DB               |  RPC          |
|  -------           |  -------            |  --               |  ---          |
|  Peers  @@@@@@oooo |  Headers   2.4 GiB  |  Blocks      3M   |  * Active    |
|          8 / 10    |  Txns     18.7 GiB  |  Txns      412M   |  2.4 req/s   |
|  Stale         0   |  Receipts  1.2 GiB  |  Logs      1.2B   |  Total 1,247 |
|  Chain  21,453,947 |  Total    22.8 GiB  |  Shards     8/8   |  getLogs 892 |
|                    |                     |                   |  Errors    0 |
+--------------------+---------------------+-------------------+---------------+
|  Logs                                                                        |
|  14:52:18 INFO  New block 21,453,947 (0x7f3a..2c1b)                         |
|  14:52:06 INFO  New block 21,453,946 (0x2d8c..9e4a)                         |
|  14:51:54 INFO  New block 21,453,945 (0x9c2f..1d3e)                         |
|                                                                              |
+------------------------------------------------------------------------------+
```

---

## STARTUP Mode (Splash Screen)

On initial launch (0 peers), the node displays a DOS-style splash screen:
- DOS blue background (RGB 0,0,170) with golden ASCII art (RGB 255,255,85)
- "STATELESS HISTORY NODE" ASCII art logo (80x23 characters)
- Animated status message ("Connecting to Ethereum P2P Network...")
- Best chain head display once peers respond
- Credit line and version number
- Optional double-line border frame on larger terminals (84x28+)

The splash transitions to the config/startup panel once peers connect, then to the sync dashboard.

## Implementation Notes

### Dependencies
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation (backend for ratatui)

### Key Features
- Fullscreen alternate screen mode
- 10 Hz refresh rate for smooth updates
- Log panel captures tracing events via `TuiLogBuffer`
- Graceful terminal restoration on exit (also on Drop)
- Phase enum prevents backwards transitions (race condition protection)
- Speed calculated as 1-second windowed average; peak never decreases
- Peer visualization never shrinks (uses high-water mark)
- Coverage buckets from `SyncProgressStats` for blocks map
- Follow mode staleness check: 30s threshold for "SYNCED" vs "CATCHING UP"
- Press `q` to quit; waits up to 3 seconds for graceful sync completion

### Demo Binary
Preview the full dashboard design without running sync:
```bash
cargo run --manifest-path node/Cargo.toml --bin ui-mock
```
Controls: `q` = quit, `n` = next phase.
