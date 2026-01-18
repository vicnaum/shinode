# Q026 - Metrics + Tracing Hooks in Network + Sync

Status: verified

## Question
Where are metrics/tracing hooks in network + sync?
I want where metrics are defined, how they are registered/emitted, and which code paths (network + sync) emit them.

## Context Packs
- `spec/reth_repo_context/context-sync.xml`
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML packs:
- spec/reth_repo_context/context-sync.xml
- spec/reth_repo_context/context-network.xml

Question (Q026):
Where are metrics/tracing hooks in network + sync?
I want where metrics are defined, how they are registered/emitted, and which code paths (network + sync) emit them.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Cover network (peers, sessions, request handler) and sync (pipeline, stages, downloaders).
3. Identify any tracing spans/events and how they are hooked/used.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Metrics/tracing flow (step-by-step)
4. Key abstractions/types (with file paths)
5. "Where to look next"
6. Suggested follow-up questions
```

## Gemini Answer
Based on the provided XML packs, here is the analysis of the metrics and tracing hooks in Reth's network and sync components.

### 1. Summary
*   **Centralized Definitions**: Metrics are defined in dedicated `metrics.rs` modules within their respective crates (`reth-network`, `reth-downloaders`, `reth-stages-api`) using the `#[derive(Metrics)]` macro from `reth_metrics`.
*   **Scope-Based Registry**: Metrics are automatically registered under specific scopes (e.g., `network`, `downloaders.bodies`, `sync`) via the `#[metrics(scope = "...")]` attribute.
*   **Event-Driven Sync Metrics**: The Sync Pipeline uses a decoupled `MetricsListener` pattern. Stages and the Pipeline emit `MetricEvent`s over a channel, which the listener consumes to update Prometheus gauges.
*   **Direct Network Metrics**: Network components (Manager, Session, Request Handler) typically hold a metrics struct instance directly and update counters/gauges inline during event processing or polling loops.
*   **Tracing Conventions**: Tracing is pervasive, utilizing the `target` field (e.g., `target: "net::session"`, `target: "sync::pipeline"`) to allow granular filtering of logs.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/metrics.rs` | `NetworkMetrics` | Defines global network stats (connected peers, bandwidth, active sessions). |
| `crates/net/downloaders/src/metrics.rs` | `HeaderDownloaderMetrics` | Defines stats for header downloads (throughput, concurrent requests). |
| `crates/stages/api/src/metrics/sync_metrics.rs` | `StageMetrics` | Defines stats per stage (checkpoint number, entities processed). |
| `crates/stages/api/src/metrics/listener.rs` | `MetricsListener` | A background future that consumes `MetricEvent`s and updates `SyncMetrics`. |
| `crates/net/network/src/manager.rs` | `NetworkManager` | Holds `NetworkMetrics` and updates them inside the main event loop. |

---

### 3. Metrics & Tracing Flow

#### A. Network Layer (Direct Updates)

In the networking stack, components own their metrics structs and update them immediately when events occur.

1.  **Definitions**:
    *   `NetworkMetrics`, `SessionManagerMetrics`, `DisconnectMetrics` are defined in `crates/net/network/src/metrics.rs`.
    *   `EthRequestHandlerMetrics` is defined in the same file.

2.  **Instantiation**:
    *   `NetworkManager` creates these structs in `new()` (`crates/net/network/src/manager.rs`).

3.  **Update Flow (Example: Session Established)**:
    *   **Source**: `Swarm` emits `SwarmEvent::SessionEstablished`.
    *   **Handler**: `NetworkManager::on_swarm_event` handles this event.
    *   **Metric Update**:
        ```rust
        // crates/net/network/src/manager.rs
        self.metrics.connected_peers.set(total_active as f64);
        ```
    *   **Tracing**:
        ```rust
        debug!(target: "net", ?remote_addr, ... "Session established");
        ```

4.  **Update Flow (Example: Request Handling)**:
    *   `EthRequestHandler` (`crates/net/network/src/eth_requests.rs`) holds `EthRequestHandlerMetrics`.
    *   When `on_headers_request` is called:
        ```rust
        // crates/net/network/src/eth_requests.rs
        self.metrics.eth_headers_requests_received_total.increment(1);
        ```

#### B. Sync Layer (Event-Driven Updates)

The Sync Pipeline uses a channel-based approach to decouple execution from metric recording.

1.  **Definitions**:
    *   `StageMetrics` is defined in `crates/stages/api/src/metrics/sync_metrics.rs`. It tracks `checkpoint`, `entities_processed`, `entities_total`, `total_elapsed`.

2.  **Instantiation**:
    *   `Pipeline` holds a `metrics_tx: Option<MetricEventsSender>` (`crates/stages/api/src/pipeline/mod.rs`).
    *   A `MetricsListener` is created to consume the receiver end (`crates/stages/api/src/metrics/listener.rs`).

3.  **Update Flow (Example: Stage Completion)**:
    *   **Source**: `Pipeline::execute_stage_to_completion`.
    *   **Action**: Upon stage completion, it sends an event:
        ```rust
        // crates/stages/api/src/pipeline/mod.rs
        metrics_tx.send(MetricEvent::StageCheckpoint { stage_id, checkpoint, ... });
        ```
    *   **Listener**: The `MetricsListener::poll` loop receives the event.
    *   **Metric Update**:
        ```rust
        // crates/stages/api/src/metrics/listener.rs
        let stage_metrics = self.sync_metrics.get_stage_metrics(stage_id);
        stage_metrics.checkpoint.set(checkpoint.block_number as f64);
        ```

#### C. Downloaders (Hybrid)

Downloaders hold their own metrics structs but are driven by the pipeline.

1.  **Definitions**:
    *   `BodyDownloaderMetrics` in `crates/net/downloaders/src/metrics.rs`.
2.  **Usage**:
    *   `BodiesDownloader` (`crates/net/downloaders/src/bodies/bodies.rs`) updates metrics during polling.
    *   Example: `self.metrics.total_flushed.increment(next_batch.len() as u64);`.

---

### 4. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `NetworkMetrics` | `crates/net/network/src/metrics.rs` | High-level network stats (IO, connection counts). Scope: `network`. |
| `DisconnectMetrics` | `crates/net/network/src/metrics.rs` | Counters for each `DisconnectReason`. Scope: `network`. |
| `MetricEvent` | `crates/stages/api/src/metrics/listener.rs` | Enum used to pass metric updates from Pipeline to Listener (`SyncHeight`, `StageCheckpoint`). |
| `SyncMetrics` | `crates/stages/api/src/metrics/sync_metrics.rs` | Container for `HashMap<StageId, StageMetrics>`. Scope: `sync`. |
| `EthRequestHandlerMetrics` | `crates/net/network/src/metrics.rs` | Stats for `eth` protocol requests (GetHeaders, GetBodies). |

---

### 5. Where to Look Next

1.  **`crates/net/network/src/manager.rs`**: Look for `on_swarm_event` to see how nearly all high-level network metrics are updated in reaction to swarm events.
2.  **`crates/stages/api/src/metrics/listener.rs`**: To see the logic mapping high-level pipeline events to specific Prometheus gauge updates.
3.  **`crates/net/network/src/session/active.rs`**: To see deep tracing hooks (e.g., `trace!(target: "net::session", ...)` which are useful for debugging specific connection issues.
4.  **`crates/net/downloaders/src/metrics.rs`**: Defines the specific counters used for measuring download throughput and buffer sizes.

---

### 6. Suggested Follow-up Questions

*   "How are the `poll_durations` metrics in `NetworkManager` calculated, and what do they measure specifically?"
*   "Where are the `TxTypesCounter` metrics updated when a new transaction is received via broadcast?"
*   "How does `MetricsListener` handle dynamic stages, or is the list of stages fixed at startup for metric registration?"

## Verification
- Network metrics are defined with `#[derive(Metrics)]` and `#[metrics(scope = "network")]`, including `NetworkMetrics`, `SessionManagerMetrics`, and `EthRequestHandlerMetrics`.```12:78:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/metrics.rs
#[derive(Metrics)]
#[metrics(scope = "network")]
pub struct NetworkMetrics { ... }

#[derive(Metrics)]
#[metrics(scope = "network")]
pub struct SessionManagerMetrics { ... }
```
```315:334:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/metrics.rs
#[derive(Metrics)]
#[metrics(scope = "network")]
pub struct EthRequestHandlerMetrics {
    pub(crate) eth_headers_requests_received_total: Counter,
    pub(crate) eth_receipts_requests_received_total: Counter,
    pub(crate) eth_bodies_requests_received_total: Counter,
    pub(crate) eth_node_data_requests_received_total: Counter,
    pub(crate) acc_duration_poll_eth_req_handler: Gauge,
}
```
- Downloader metrics are scoped via attributes, and both headers and bodies have their own metric structs.```19:52:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/metrics.rs
#[derive(Clone, Metrics)]
#[metrics(scope = "downloaders.bodies")]
pub struct BodyDownloaderMetrics { ... }
```
```90:123:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/metrics.rs
#[derive(Clone, Metrics)]
#[metrics(scope = "downloaders.headers")]
pub struct HeaderDownloaderMetrics { ... }
```
- Sync metrics are defined in `StageMetrics` with `sync` scope and are keyed by stage ID.```11:31:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/metrics/sync_metrics.rs
pub(crate) fn get_stage_metrics(&mut self, stage_id: StageId) -> &mut StageMetrics { ... }

#[derive(Metrics)]
#[metrics(scope = "sync")]
pub(crate) struct StageMetrics {
    pub(crate) checkpoint: Gauge,
    pub(crate) entities_processed: Gauge,
    pub(crate) entities_total: Gauge,
    pub(crate) total_elapsed: Gauge,
}
```
- The sync metrics channel uses `MetricEvent` and is consumed by `MetricsListener`, which updates gauges and traces on `sync::metrics`.```13:73:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/metrics/listener.rs
pub type MetricEventsSender = UnboundedSender<MetricEvent>;

pub enum MetricEvent { ... }

fn handle_event(&mut self, event: MetricEvent) {
    trace!(target: "sync::metrics", ?event, "Metric event received");
    match event {
        MetricEvent::StageCheckpoint { stage_id, checkpoint, max_block_number, elapsed } => {
            let stage_metrics = self.sync_metrics.get_stage_metrics(stage_id);
            stage_metrics.total_elapsed.increment(elapsed.as_secs_f64());
            stage_metrics.checkpoint.set(checkpoint.block_number as f64);
            // ...
        }
        // ...
    }
}
```
- The pipeline wires metrics into execution: `PipelineBuilder::with_metrics_tx` stores a sender, and `Pipeline::register_metrics` and `execute_stage_to_completion` emit `MetricEvent::StageCheckpoint`.```58:90:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/builder.rs
pub fn with_metrics_tx(mut self, metrics_tx: MetricEventsSender) -> Self {
    self.metrics_tx = Some(metrics_tx);
    self
}
```
```132:146:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
pub fn register_metrics(&mut self) -> Result<(), PipelineError> {
    let Some(metrics_tx) = &mut self.metrics_tx else { return Ok(()) };
    // ...
    let _ = metrics_tx.send(MetricEvent::StageCheckpoint { ... });
    Ok(())
}
```
```490:505:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
if let Some(metrics_tx) = &mut self.metrics_tx {
    let _ = metrics_tx.send(MetricEvent::StageCheckpoint {
        stage_id,
        checkpoint,
        max_block_number: target,
        elapsed: stage_started_at.elapsed(),
    });
}
```
- The metrics listener task is spawned in node launch code (outside the packs).```694:709:/Users/vicnaum/github/stateless-history-node/reth/crates/node/builder/src/launch/common.rs
/// This spawns a metrics task that listens for metrics related events and updates metrics for
/// prometheus.
pub fn with_metrics_task(self) -> LaunchContextWith<...> {
    let (metrics_sender, metrics_receiver) = unbounded_channel();
    debug!(target: "reth::cli", "Spawning stages metrics listener task");
    let sync_metrics_listener = reth_stages::MetricsListener::new(metrics_receiver);
    self.task_executor().spawn_critical("stages metrics listener task", sync_metrics_listener);
    // ...
}
```
- Network metrics are updated directly during swarm event handling (e.g., incoming connections and session establishment).```755:812:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
SwarmEvent::IncomingTcpConnection { remote_addr, session_id } => {
    trace!(target: "net", ?remote_addr, "Incoming connection");
    self.metrics.total_incoming_connections.increment(1);
    self.metrics.incoming_connections.set(
        self.swarm.state().peers().num_inbound_connections() as f64
    );
}
SwarmEvent::SessionEstablished { .. } => {
    let total_active = self.num_active_peers.fetch_add(1, Ordering::Relaxed) + 1;
    self.metrics.connected_peers.set(total_active as f64);
    // ...
}
```
- `EthRequestHandler` increments request counters on each request type.```151:169:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
fn on_headers_request(...) {
    self.metrics.eth_headers_requests_received_total.increment(1);
    // ...
}

fn on_bodies_request(...) {
    self.metrics.eth_bodies_requests_received_total.increment(1);
    // ...
}
```
- Downloaders update their metrics inline during polling (examples from bodies and headers).```255:263:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
let next_batch = self.queued_bodies.drain(..self.stream_batch_size).collect::<Vec<_>>();
self.metrics.total_flushed.increment(next_batch.len() as u64);
self.metrics.queued_blocks.set(self.queued_bodies.len() as f64);
```
```897:900:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
trace!(target: "downloaders::headers", batch=%next_batch.len(), "Returning validated batch");
this.metrics.total_flushed.increment(next_batch.len() as u64);
```
- Tracing uses explicit targets for filtering (examples in session and pipeline).```262:267:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
trace!(
    target: "net::session",
    ?remote_addr,
    ?session_id,
    "new pending incoming session"
);
```
```174:176:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
trace!(target: "sync::pipeline", ?target, ?result, "Pipeline finished");
```

## Corrections / Caveats
- Sync metrics live in `crates/stages/api/src/metrics/sync_metrics.rs` and `metrics/listener.rs`, not a single `metrics.rs` file.
- The listener task wiring is outside the two packs (node launch code in `crates/node/builder/src/launch/common.rs`).

## Actionable Pointers
- Network metrics definitions and updates: `reth/crates/net/network/src/metrics.rs` and `reth/crates/net/network/src/manager.rs`.
- Request handler metrics: `reth/crates/net/network/src/eth_requests.rs`.
- Downloader metrics + usage: `reth/crates/net/downloaders/src/metrics.rs`, `reth/crates/net/downloaders/src/headers/reverse_headers.rs`, `reth/crates/net/downloaders/src/bodies/bodies.rs`.
- Sync metrics channel and listener: `reth/crates/stages/api/src/metrics/listener.rs` and `reth/crates/stages/api/src/pipeline/mod.rs`.
