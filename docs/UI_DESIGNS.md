# UI Dashboard Designs

This document contains the planned UI designs for the Stateless History Node dashboard using Ratatui.

## Overview

The UI is a fullscreen terminal dashboard with:
- Phase indicator at top (SYNC → COMPACT → SEAL → FOLLOW)
- Progress visualization
- Speed graph with history
- Network/Queue/Storage panels
- Log panel at bottom

## Color Scheme

| Element | Color | Notes |
|---------|-------|-------|
| SYNC phase | Cyan | Active during fetch |
| COMPACT phase | Magenta | Active during compaction |
| SEAL phase | Green | Active during sealing |
| FOLLOW phase | Bright Green | Active when synced |
| Completed phase | Dim/Gray | ✓ checkmark |
| Current speed | Yellow | Stands out in stats |
| Peak speed | Bright Cyan | Achievement highlight |
| Average speed | White | Baseline |
| Warnings in logs | Yellow | WARN level |
| Errors in logs | Red | ERROR level |
| Progress bar fill | Phase color | Matches current phase |

## Blocks Map Legend

The "Blocks" visualization shows sync coverage with shading density:

```
  ░░░░  = Not yet fetched (empty/dim)
  ▒▒▒▒  = Partially fetched (some blocks in range)
  ▓▓▓▓  = Mostly fetched (most blocks in range)
  ████  = Complete (all blocks in range, committed to storage)
```

---

## SYNC Mode

```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  STATELESS HISTORY NODE                                            14:32:05  ┃
┃                                                                               ┃
┃  Phase:  [■ SYNC]  [□ COMPACT]  [□ SEAL]  [□ FOLLOW]                         ┃
┃           cyan      magenta      green     bright green                       ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃                                                                               ┃
┃  Progress   ████████████████████████████░░░░░░░░░░░░░  68.5%     ETA 42m 18s ┃
┃            10,000,000 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━▶ 21,000,000            ┃
┃                              3,152,769 blocks synced                          ┃
┃                                                                               ┃
┃  Blocks    ░░░░████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃
┃            ▁▁▁▁████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃
┃            10M   ↑fetching                                              21M  ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃                                                                               ┃
┃  ┌─ Speed (blocks/s) ──────────────────────────────────────────────────────┐ ┃
┃  │ 2.0K ┤                                                                  │ ┃
┃  │      │         ▄▄▄▄      ▄▄▄▄▄▄              ▄▄▄▄                       │ ┃
┃  │ 1.5K ┤    ▄▄▄▄██████▄▄▄▄████████▄▄▄▄    ▄▄▄▄██████▄▄▄▄                  │ ┃
┃  │      │▄▄▄███████████████████████████▄▄▄███████████████▄▄▄▄▄▄▄▄▄        │ ┃
┃  │ 1.0K ┤████████████████████████████████████████████████████████████▄▄   │ ┃
┃  │      │                                                            ██   │ ┃
┃  │  0.5K┤                                                            ██   │ ┃
┃  │    0 ┼──────────────────────────────────────────────────────────────── │ ┃
┃  │       -5m                                                          now │ ┃
┃  └─────────────────────────────────────────────────────────────────────────┘ ┃
┃   ● Current: 1,247/s     ◆ Average: 1,180/s     ★ Peak: 1,892/s             ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  NETWORK           ┃  QUEUE              ┃  STORAGE                          ┃
┃  ───────           ┃  ─────              ┃  ───────                          ┃
┃  Peers  ●●●●●●○○○○ ┃  Pending      284   ┃  Headers      2.4 GiB             ┃
┃          6 / 10    ┃  Inflight      48   ┃  Receipts    18.7 GiB             ┃
┃                    ┃  Retry          3   ┃  Tx Hashes    1.2 GiB             ┃
┃  ↓ Rx    892 KB/s  ┃  Failed         0   ┃  ─────────────────────            ┃
┃  Chain  21,453,892 ┃                     ┃  Total       22.3 GiB             ┃
┃                    ┃  Batch Size   128   ┃  Write Rate  12.4 MB/s            ┃
┃                    ┃                     ┃                                   ┃
┣━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  Logs                                                                        ┃
┃  ────                                                                        ┃
┃  14:32:05 INFO  Fetched batch 1847: blocks 13,245,000..13,245,128           ┃
┃  14:32:04 INFO  Peer 0x7f3a..2c1b connected (eth/68)                        ┃
┃  14:32:03 INFO  Committed shard 1320 (13,200,000..13,209,999)               ┃
┃  14:32:01 WARN  Peer 0x2d8c..9e4a timeout, retrying batch 1843              ┃
┃                                                                               ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

---

## COMPACT Mode

```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  STATELESS HISTORY NODE                                            14:45:22  ┃
┃                                                                               ┃
┃  Phase:  [✓ SYNC]  [■ COMPACT]  [□ SEAL]  [□ FOLLOW]                         ┃
┃           dim       magenta      green     bright green                       ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃                                                                               ┃
┃  Compacting   ████████████████░░░░░░░░░░░░░░░░░░░░░  42%      18 / 43 shards ┃
┃                                                                               ┃
┃  Shards      [✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓◉◉○○○○○○○○○○○○○○○○○○○○○○○○○○○]               ┃
┃              ✓ = done   ◉ = compacting   ○ = pending                          ┃
┃                                                                               ┃
┃  Blocks      ████████████████████████████████████████▓▓▓▓▓▓░░░░░░░░░░░░░░░░░ ┃
┃              10M          compacted ↑    ↑ compacting                    21M  ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  Session Summary                                                              ┃
┃  ───────────────                                                              ┃
┃  Synced       3,152,769 blocks    Duration    2h 14m 33s                     ┃
┃  Avg Speed        1,180 blk/s     Peak            1,892 blk/s                ┃
┃  Storage         22.3 GiB         Peers              6 / 10                  ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  Logs                                                                        ┃
┃  14:45:22 INFO  Compacting shard 1328 (13,280,000..13,289,999)              ┃
┃  14:45:20 INFO  Compacted shard 1327 in 2.3s                                ┃
┃  14:45:17 INFO  Compacted shard 1326 in 2.1s                                ┃
┃                                                                               ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

---

## FOLLOW Mode

```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  STATELESS HISTORY NODE                                            14:52:18  ┃
┃                                                                               ┃
┃  Phase:  [✓ SYNC]  [✓ COMPACT]  [✓ SEAL]  [■ FOLLOW]                         ┃
┃           dim       dim          dim       bright green                       ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃                                                                               ┃
┃     ✓ SYNCED                                                                  ┃
┃                                                                               ┃
┃     Chain Head      21,453,947          Last Block    3s ago                  ┃
┃     Our Head        21,453,947  ✓       Reorgs        0                       ┃
┃                                                                               ┃
┃  Coverage    ████████████████████████████████████████████████████████████████ ┃
┃              10,000,000 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━▶ 21,453,947  100%   ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃                                                                               ┃
┃  ┌─ New Blocks (last 5 min) ───────────────────────────────────────────────┐ ┃
┃  │                    ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  ▄▄  │ ┃
┃  │                    ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  │ ┃
┃  │ ──────────────────────────────────────────────────────────────────────  │ ┃
┃  │  -5m                                                              now   │ ┃
┃  └─────────────────────────────────────────────────────────────────────────┘ ┃
┃   ● Incoming: ~0.08 blk/s (1 block every ~12s)                               ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  SESSION STATS                           ┃  LIVE                              ┃
┃  ──────────────                          ┃  ────                              ┃
┃  Total Synced    11,453,947 blocks       ┃  Peers      ●●●●●●●●○○    8 / 10   ┃
┃  Duration             2h 31m 04s         ┃  Storage          22.8 GiB         ┃
┃  Avg Sync Speed       1,180 blk/s        ┃  RPC Requests        1,247         ┃
┃  Peak Speed           1,892 blk/s        ┃  RPC Latency      12ms avg         ┃
┃                                                                               ┃
┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
┃  Logs                                                                        ┃
┃  14:52:18 INFO  New block 21,453,947 (0x7f3a..2c1b) - 142 txs, 12.4M gas    ┃
┃  14:52:06 INFO  New block 21,453,946 (0x2d8c..9e4a) - 98 txs, 8.7M gas      ┃
┃  14:51:54 INFO  New block 21,453,945 (0x9c2f..1d3e) - 156 txs, 14.2M gas    ┃
┃                                                                               ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

---

## Implementation Notes

### Dependencies
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation (backend for ratatui)

### Key Features
- Fullscreen alternate screen mode
- 10 Hz refresh rate for smooth updates
- Log panel captures tracing events
- Graceful terminal restoration on exit
