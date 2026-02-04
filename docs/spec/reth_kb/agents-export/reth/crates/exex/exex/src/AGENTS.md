# src

## Purpose
Implements the `reth-exex` crate: execution extensions ("ExEx") runtime for reth. Provides the ExEx manager, ExEx context (typed and dynamic), notification streams with optional backfilling, pruning progress signaling via `FinishedHeight`, and a WAL to persist notifications and support recovery.

## Contents (one hop)
### Subdirectories
- [x] `backfill/` - backfill jobs and async streams to replay historical blocks and catch an ExEx up to a head.
- [x] `wal/` - write-ahead log for ExEx notifications (disk storage + block cache + metrics).

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines the public ExEx API surface and re-exports key modules/types (including `reth-exex-types`).
- **Key items**: `ExExContext`, `ExExEvent`, `ExExManager`, `ExExNotification(s)`, WAL and backfill APIs.

#### `event.rs`
- **Role**: Defines events emitted by an ExEx back to the node (currently `FinishedHeight`).
- **Key items**: `ExExEvent::FinishedHeight`

#### `context.rs`
- **Role**: Typed ExEx runtime context: exposes node head/config, node components, notification stream, and an event channel for communicating progress (pruning) back to the manager.
- **Key items**: `ExExContext`, `ExExContext::send_finished_height()`, accessors `pool()`, `provider()`, `network()`, `payload_builder_handle()`, `task_executor()`

#### `dyn_context.rs`
- **Role**: Dynamic (type-erased) `ExExContext` for consumers that can't or don't want to carry the full `FullNodeComponents` generic parameter.
- **Key items**: `ExExContextDyn`

#### `notifications.rs`
- **Role**: Notification stream implementation: provides `ExExNotifications` and `ExExNotificationsStream` that can be configured "with head" to backfill and then follow live notifications, or "without head" to receive all notifications.
- **Key items**: `ExExNotifications`, `ExExNotificationsStream`, `ExExNotificationsWithoutHead`, `ExExNotificationsWithHead`
- **Interactions**: uses `BackfillJobFactory`/`StreamBackfillJob` for replay; integrates with WAL via `WalHandle`.

#### `manager.rs`
- **Role**: ExEx manager runtime: buffers notifications, sends them to registered ExExes with backpressure, tracks per-ExEx finished heights, and manages the WAL/finalization safety thresholds.
- **Key items**: `ExExManager`, `ExExHandle`, `ExExManagerHandle`, `ExExNotificationSource`, `DEFAULT_EXEX_MANAGER_CAPACITY`, `DEFAULT_WAL_BLOCKS_WARNING`
- **Interactions**: consumes node streams (canonical state + finalized headers) and fans out to ExExes; relies on WAL for persistence and catch-up.

## Key APIs (no snippets)
- `ExExContext`, `ExExContextDyn`
- `ExExNotifications` / `ExExNotificationsStream`
- `ExExManager`
- `Wal`

## Relationships
- **Used by**: `reth-node-builder` / node implementations to host user-provided ExEx tasks alongside the node and drive pruning decisions based on `FinishedHeight` events.
