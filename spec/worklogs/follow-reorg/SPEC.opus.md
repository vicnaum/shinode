I've created a comprehensive spec document at `LIVE_FOLLOW_REORG_SPEC.md` and updated the `ROADMAP.md` to reference it.

The spec includes:

**Structure (similar to the historical sync spec):**
- Context / problem statement
- Goals and non-goals
- Research findings (reth + rindexer approaches)
- Design overview with state machine diagram
- Core types and contracts (`ChainTracker`, `ReorgDetector`, `HeaderStub`, etc.)
- Detailed live follow loop pseudocode
- Reorg detection algorithm (parent hash walk-back)
- Storage rollback implementation (NippyJar truncation)
- Configuration options and CLI flags
- RPC behavior during live mode
- Metrics to expose
- Error handling (recoverable vs deep reorg)
- **9-phase implementation plan with checkboxes**
- Acceptance criteria
- Risks and mitigations
- Future considerations

**Key design decisions captured:**
1. **Safe head**: always stay `rollback_window` blocks behind network head
2. **Reorg detection**: compare new block's parent hash against our stored head hash
3. **Walk-back search**: find common ancestor within rollback_window
4. **Rollback**: truncate NippyJar segments + update checkpoint + update ChainTracker
5. **Deep reorg**: error out if reorg exceeds rollback_window (manual intervention)
6. **RPC**: `eth_blockNumber` can decrease after rollback (matches rindexer expectations)

The ROADMAP now has expanded v0.1.6 checkboxes that map to the spec phases, making it easy to track progress

---


## Live Follow + Reorg Handling Spec

### Context / problem statement
We currently have a historical sync pipeline that:
- Fetches blocks from `start_block` to `head - rollback_window`
- Writes to NippyJar static files
- Exits when the historical range is complete

What we lack:
- **Live "follow" mode**: continuous syncing that tracks the chain head after historical catch-up
- **Reorg handling**: detecting and recovering from chain reorganizations

This spec defines the implementation for:
- **Live follow loop** that continuously monitors chain head and ingests new blocks
- **Reorg detection** by comparing parent hashes with stored headers
- **Rollback mechanism** that truncates static files back to the common ancestor
- **Safe head calculation** to avoid indexing blocks likely to be reorged

---

### Goals (what "done" means)
- **Live follow**: after historical sync completes, node continuously monitors chain head and ingests new blocks
- **Reorg detection**: when a new block's parent hash doesn't match our stored head, detect this as a reorg
- **Common ancestor search**: walk back through stored headers to find the fork point (up to `rollback_window`)
- **Rollback capability**: truncate static files to common ancestor and re-ingest from there
- **Safe head boundary**: never index blocks within `rollback_window` of the network head
- **RPC consistency**: `eth_blockNumber` returns the last indexed block (can decrease after rollback)
- **Rindexer compatibility**: our reorg handling must be compatible with rindexer's expectations

---

### Non-goals (this spec)
- Deep reorgs beyond `rollback_window` (we'll error/alert and require manual intervention)
- Mempool/pending transaction tracking
- Engine API / CL integration (we follow EL head via P2P)
- Multi-chain support (single-chain focus for now)

---

### Research findings

#### How reth handles reorgs
Reth uses a two-layer architecture:
1. **In-memory canonical chain**: keeps recent blocks/headers in RAM for fast reorg detection
2. **Persistence layer**: writes to DB only after blocks are considered "safe"

Reorg detection:
- Walk back parent hashes to find common ancestor
- Engine API provides `forkchoiceUpdated` with `safe_head` and `finalized_head`
- Reth maintains a configurable "persistence threshold" (typically 128 blocks) before writing to disk

#### How rindexer handles reorgs
Rindexer's approach:
- Compares block numbers: if new block number <= current indexed head, it's a potential reorg
- Uses "reorg safe distance":
  - Mainnet: 12 blocks
  - Other chains: 64 blocks
- Only indexes up to `latest_block - reorg_safe_distance`
- On reorg: rolls back indexed data to the common ancestor

**Compatibility requirement**: our `rollback_window` serves the same purpose as rindexer's `reorg_safe_distance`.

---

### Design overview (target architecture)

#### High-level state machine

```
                    ┌─────────────────┐
                    │  HISTORICAL     │
                    │  SYNC           │
                    └────────┬────────┘
                             │ caught up to safe_head
                             ▼
                    ┌─────────────────┐
          ┌────────▶│  LIVE FOLLOW    │◀────────┐
          │         │  (polling)      │         │
          │         └────────┬────────┘         │
          │                  │ new block        │
          │                  ▼                  │
          │         ┌─────────────────┐         │
          │         │  CHECK PARENT   │         │
          │         │  HASH           │         │
          │         └───┬─────────┬───┘         │
          │ match       │         │ mismatch    │
          │             ▼         ▼             │
          │    ┌────────────┐  ┌────────────┐   │
          │    │ INGEST     │  │ DETECT     │   │
          │    │ BLOCK      │  │ REORG      │   │
          │    └─────┬──────┘  └─────┬──────┘   │
          │          │               │          │
          │          │               ▼          │
          │          │      ┌────────────────┐  │
          │          │      │ FIND COMMON    │  │
          │          │      │ ANCESTOR       │  │
          │          │      └───────┬────────┘  │
          │          │              │           │
          │          │              ▼           │
          │          │      ┌────────────────┐  │
          │          │      │ ROLLBACK TO    │  │
          │          │      │ ANCESTOR       │  │
          │          │      └───────┬────────┘  │
          │          │              │           │
          └──────────┴──────────────┴───────────┘
```

#### Key components

1. **LiveFollowRunner**: main loop that polls for new blocks
2. **ChainTracker**: minimal in-memory structure tracking recent headers for reorg detection
3. **ReorgDetector**: compares new blocks against stored headers
4. **RollbackExecutor**: truncates static files to a given block number
5. **SafeHeadCalculator**: computes `safe_head = network_head - rollback_window`

---

### Core types and contracts

#### ChainTracker (in-memory header cache)

```rust
/// Minimal header info needed for reorg detection
pub struct HeaderStub {
    pub number: u64,
    pub hash: B256,
    pub parent_hash: B256,
}

/// Tracks recent headers for reorg detection
/// Keeps at most `rollback_window` headers in memory
pub struct ChainTracker {
    /// Recent headers by block number
    headers: BTreeMap<u64, HeaderStub>,
    /// Current indexed head
    head: u64,
    /// Maximum headers to keep (= rollback_window)
    max_depth: u64,
}

impl ChainTracker {
    /// Add a new header (after successful ingest)
    fn advance(&mut self, stub: HeaderStub);

    /// Get our stored header hash at a block number
    fn get_hash(&self, number: u64) -> Option<B256>;

    /// Get our current head
    fn head(&self) -> u64;

    /// Rollback to a block number (remove all headers > number)
    fn rollback_to(&mut self, number: u64);

    /// Prune old headers beyond rollback_window
    fn prune(&mut self);
}
```

#### ReorgDetector

```rust
pub enum ReorgStatus {
    /// No reorg, parent hash matches
    NoReorg,
    /// Reorg detected, found common ancestor at this block
    ReorgDetected { ancestor_block: u64, depth: u64 },
    /// Deep reorg beyond rollback_window - requires manual intervention
    DeepReorg { depth: u64 },
}

impl ReorgDetector {
    /// Check if a new block causes a reorg
    /// Returns the common ancestor if a reorg is detected
    fn check(
        &self,
        new_header: &Header,
        chain_tracker: &ChainTracker,
        storage: &Storage,
        rollback_window: u64,
    ) -> Result<ReorgStatus>;
}
```

#### SafeHead calculation

```rust
/// Calculate the safe head we should index up to
fn compute_safe_head(network_head: u64, rollback_window: u64) -> u64 {
    network_head.saturating_sub(rollback_window)
}
```

#### Rollback execution

```rust
impl Storage {
    /// Truncate all static files to block `number` (exclusive)
    /// After this call, the highest stored block is `number`
    async fn rollback_to(&self, number: u64) -> Result<()>;
}
```

---

### Live follow loop (detailed flow)

#### Polling strategy

```rust
// Configurable poll interval (default: 12 seconds for mainnet)
const DEFAULT_POLL_INTERVAL_MS: u64 = 12_000;

// Faster polling when catching up after a gap
const CATCHUP_POLL_INTERVAL_MS: u64 = 1_000;
```

#### Main loop pseudocode

```rust
async fn run_live_follow(
    storage: &Storage,
    network: &Network,
    chain_tracker: &mut ChainTracker,
    config: &LiveFollowConfig,
) -> Result<()> {
    let rollback_window = config.rollback_window;

    loop {
        // 1. Get current network head
        let network_head = network.head().await?;
        let safe_head = compute_safe_head(network_head, rollback_window);

        // 2. Get our current indexed head
        let our_head = chain_tracker.head();

        // 3. If we're caught up, wait and poll again
        if our_head >= safe_head {
            tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
            continue;
        }

        // 4. Fetch the next block to ingest
        let next_block = our_head + 1;
        let (header, body, receipts) = network.fetch_block(next_block).await?;

        // 5. Check for reorg
        let reorg_status = reorg_detector.check(&header, chain_tracker, storage, rollback_window)?;

        match reorg_status {
            ReorgStatus::NoReorg => {
                // 6a. Normal case: ingest the block
                let payload = BlockPayload { header, body, receipts };
                storage.write_block(&payload).await?;

                // 7a. Update chain tracker
                chain_tracker.advance(HeaderStub {
                    number: header.number,
                    hash: header.hash(),
                    parent_hash: header.parent_hash,
                });

                // 8a. Update metrics/progress
                metrics::indexed_block.set(header.number);
            }

            ReorgStatus::ReorgDetected { ancestor_block, depth } => {
                // 6b. Reorg detected: rollback and re-sync
                tracing::warn!(
                    depth,
                    ancestor = ancestor_block,
                    "Reorg detected, rolling back"
                );

                // 7b. Rollback storage to common ancestor
                storage.rollback_to(ancestor_block).await?;

                // 8b. Rollback chain tracker
                chain_tracker.rollback_to(ancestor_block);

                // 9b. Metrics
                metrics::reorgs_total.inc();
                metrics::reorg_depth_max.observe(depth);

                // 10b. Continue loop - will re-fetch from ancestor+1
            }

            ReorgStatus::DeepReorg { depth } => {
                // Deep reorg beyond our rollback window
                // This is a serious error that requires manual intervention
                tracing::error!(
                    depth,
                    rollback_window,
                    "Deep reorg detected beyond rollback window! Manual intervention required."
                );
                return Err(eyre::eyre!(
                    "Deep reorg of {} blocks exceeds rollback_window of {}",
                    depth,
                    rollback_window
                ));
            }
        }
    }
}
```

---

### Reorg detection algorithm

#### Parent hash walk-back

```rust
impl ReorgDetector {
    fn check(
        &self,
        new_header: &Header,
        chain_tracker: &ChainTracker,
        storage: &Storage,
        rollback_window: u64,
    ) -> Result<ReorgStatus> {
        let our_head = chain_tracker.head();

        // Case 1: New block extends our chain (normal case)
        if new_header.number == our_head + 1 {
            let our_head_hash = chain_tracker.get_hash(our_head)
                .ok_or_else(|| eyre::eyre!("Missing head in chain tracker"))?;

            if new_header.parent_hash == our_head_hash {
                return Ok(ReorgStatus::NoReorg);
            }
            // Parent hash mismatch - reorg at our head
        }

        // Case 2: Block number <= our head means reorg
        // Case 3: Parent hash mismatch at our_head + 1 means reorg

        // Walk back to find common ancestor
        let min_block = our_head.saturating_sub(rollback_window);
        let mut search_block = our_head;

        while search_block >= min_block {
            // Get our stored hash at this block
            let our_hash = match chain_tracker.get_hash(search_block) {
                Some(h) => h,
                None => {
                    // Need to read from storage if not in chain_tracker
                    storage.get_header_hash(search_block).await?
                }
            };

            // Get the new chain's hash at this block
            // We need to walk up the new chain to find what they have at this block
            let new_chain_hash = self.get_new_chain_hash_at(new_header, search_block)?;

            if our_hash == new_chain_hash {
                // Found common ancestor
                let depth = our_head - search_block;
                return Ok(ReorgStatus::ReorgDetected {
                    ancestor_block: search_block,
                    depth,
                });
            }

            search_block = search_block.saturating_sub(1);
            if search_block == 0 && min_block > 0 {
                break; // Don't go below min_block
            }
        }

        // Couldn't find common ancestor within rollback_window
        Ok(ReorgStatus::DeepReorg {
            depth: our_head - min_block + 1,
        })
    }
}
```

#### Fetching reorged chain headers

When we detect a reorg, we need to fetch headers from the new chain to compare:

```rust
impl ReorgDetector {
    /// Fetch headers from the network to find what the new chain has at a given block
    async fn get_new_chain_hash_at(
        &self,
        tip_header: &Header,
        target_block: u64,
    ) -> Result<B256> {
        // Option 1: If we have the full headers cached during reorg detection
        // Option 2: Fetch headers from network walking back from tip

        // For efficiency, we can fetch a batch of headers once and cache them
        // during the reorg detection process

        // Implementation: fetch headers from tip_header.number down to target_block
        // and return the hash at target_block
    }
}
```

---

### Storage rollback implementation

#### NippyJar truncation

```rust
impl Storage {
    /// Truncate static files to remove all data after `block_number`
    pub async fn rollback_to(&self, block_number: u64) -> Result<()> {
        // 1. Truncate headers segment
        self.headers_writer.truncate_to(block_number)?;

        // 2. Truncate bodies segment (transactions)
        self.transactions_writer.truncate_to(block_number)?;

        // 3. Truncate receipts segment
        self.receipts_writer.truncate_to(block_number)?;

        // 4. Update checkpoint
        self.update_checkpoint(block_number)?;

        // 5. Sync all files
        self.sync_all()?;

        Ok(())
    }
}
```

#### NippyJar segment truncation

NippyJar files have an index that maps row numbers to offsets. Truncation requires:

```rust
impl NippyJarWriter {
    /// Truncate the segment to only include rows <= max_row
    pub fn truncate_to(&mut self, max_row: u64) -> Result<()> {
        // 1. Read current total rows from config
        let current_rows = self.config.total_rows();

        if max_row >= current_rows {
            return Ok(()); // Nothing to truncate
        }

        // 2. Find the data offset for max_row + 1
        // This is where we'll truncate the data file
        let truncate_offset = self.index.offset_for_row(max_row + 1)?;

        // 3. Truncate the data file
        self.data_file.set_len(truncate_offset)?;

        // 4. Update the config with new row count
        self.config.set_total_rows(max_row + 1);

        // 5. Rebuild/truncate the index to match
        self.index.truncate_to_row(max_row)?;

        // 6. Sync changes
        self.sync()?;

        Ok(())
    }
}
```

---

### Configuration

#### New config options

```rust
pub struct LiveFollowConfig {
    /// How many blocks behind network head to stay (reorg safety window)
    /// Default: 64 (matches rindexer's default for non-mainnet)
    pub rollback_window: u64,

    /// Poll interval when caught up (milliseconds)
    /// Default: 12000 (12 seconds, ~1 block time on mainnet)
    pub poll_interval_ms: u64,

    /// Poll interval when catching up (milliseconds)
    /// Default: 1000 (1 second)
    pub catchup_poll_interval_ms: u64,

    /// Maximum headers to keep in memory for reorg detection
    /// Should be >= rollback_window
    /// Default: rollback_window
    pub chain_tracker_depth: u64,

    /// Enable live follow mode (vs exit after historical sync)
    /// Default: true
    pub enabled: bool,
}

impl Default for LiveFollowConfig {
    fn default() -> Self {
        Self {
            rollback_window: 64,
            poll_interval_ms: 12_000,
            catchup_poll_interval_ms: 1_000,
            chain_tracker_depth: 64,
            enabled: true,
        }
    }
}
```

#### CLI flags

```
--follow                     Enable live follow mode (default: true)
--no-follow                  Disable live follow mode (exit after historical sync)
--rollback-window <N>        Reorg safety window in blocks (default: 64)
--poll-interval-ms <N>       Poll interval when caught up (default: 12000)
```

---

### RPC behavior during live follow

#### `eth_blockNumber`
- Returns the last successfully indexed block number
- **Can decrease** after a rollback (this is intentional and correct)
- Clients must handle this (rindexer does)

#### `eth_getBlockByNumber`
- Returns data only for indexed blocks
- Blocks beyond our head return an error or empty response
- After rollback, rolled-back blocks return error/empty

#### `eth_getLogs`
- Filters only search indexed blocks
- `toBlock` is capped at our indexed head
- After rollback, logs from rolled-back blocks are gone

---

### Metrics

```rust
// Gauge: current indexed block number
indexed_block: Gauge<u64>,

// Counter: total reorgs detected
reorgs_total: Counter,

// Histogram: reorg depth (how many blocks rolled back)
reorg_depth: Histogram,

// Gauge: blocks behind network head
blocks_behind: Gauge<u64>,

// Gauge: network head block number
network_head: Gauge<u64>,

// Counter: poll cycles
poll_cycles_total: Counter,

// Counter: blocks ingested in follow mode
follow_blocks_ingested: Counter,
```

---

### Error handling

#### Recoverable errors
- Network timeouts: retry with backoff
- Peer disconnects: switch to another peer
- Partial receipt responses: retry from different peer

#### Non-recoverable errors
- Deep reorg beyond `rollback_window`: log error, alert, exit
- Storage corruption: log error, exit
- Consistent hash mismatches after multiple retries: log error, exit

#### Graceful shutdown
- On SIGINT/SIGTERM: finish current block, flush storage, exit
- Don't leave storage in inconsistent state

---

### Implementation plan (checkboxes, execution-ready)

#### Phase 1 — Configuration and CLI plumbing
- [ ] **Add LiveFollowConfig struct** in `node/src/cli/mod.rs`
  - [ ] Define all config fields with defaults
  - [ ] Add CLI flags: `--follow`, `--no-follow`, `--rollback-window`, `--poll-interval-ms`
  - [ ] Update help text and README
- [ ] **Update main.rs to parse live follow config**
  - [ ] Wire config through to where it will be used

#### Phase 2 — ChainTracker (in-memory header cache)
- [ ] **Create `node/src/chain/mod.rs`** (re-implement cleaned up version)
  - [ ] Define `HeaderStub` struct
  - [ ] Define `ChainTracker` struct with `BTreeMap<u64, HeaderStub>`
  - [ ] Implement `advance()` - add new header
  - [ ] Implement `get_hash()` - lookup by block number
  - [ ] Implement `head()` - get current head
  - [ ] Implement `rollback_to()` - remove headers above block
  - [ ] Implement `prune()` - remove headers beyond rollback_window
- [ ] **Unit tests for ChainTracker**
  - [ ] Test advance and lookup
  - [ ] Test rollback_to removes correct headers
  - [ ] Test prune keeps only rollback_window headers

#### Phase 3 — Reorg detector
- [ ] **Create `node/src/sync/live/reorg.rs`**
  - [ ] Define `ReorgStatus` enum
  - [ ] Implement `ReorgDetector` struct
  - [ ] Implement `check()` method with parent hash walk-back
  - [ ] Handle edge case: first block after startup
  - [ ] Handle edge case: gaps in chain tracker (fallback to storage)
- [ ] **Unit tests for ReorgDetector**
  - [ ] Test no-reorg case (parent hash matches)
  - [ ] Test simple reorg (1 block deep)
  - [ ] Test multi-block reorg (N blocks deep)
  - [ ] Test deep reorg detection (beyond rollback_window)

#### Phase 4 — Storage rollback capability
- [ ] **Add `rollback_to()` to Storage** in `node/src/storage/mod.rs`
  - [ ] Implement header segment truncation
  - [ ] Implement transaction/body segment truncation
  - [ ] Implement receipt segment truncation
  - [ ] Update checkpoint after rollback
  - [ ] Ensure atomicity (all-or-nothing)
- [ ] **Implement NippyJar truncation helper**
  - [ ] Calculate truncation offset from row number
  - [ ] Truncate data file
  - [ ] Update config/metadata
  - [ ] Rebuild index if needed
- [ ] **Unit tests for storage rollback**
  - [ ] Test truncation removes correct data
  - [ ] Test checkpoint updates correctly
  - [ ] Test reading after truncation works

#### Phase 5 — Live follow runner
- [ ] **Create `node/src/sync/live/mod.rs`**
  - [ ] Define module structure
  - [ ] Re-export key types
- [ ] **Create `node/src/sync/live/runner.rs`**
  - [ ] Implement `LiveFollowRunner` struct
  - [ ] Implement main polling loop
  - [ ] Implement safe head calculation
  - [ ] Wire reorg detector
  - [ ] Handle normal block ingestion
  - [ ] Handle reorg detection and rollback
  - [ ] Handle deep reorg error
  - [ ] Implement graceful shutdown
- [ ] **Integration with historical sync**
  - [ ] After historical sync completes, transition to live follow
  - [ ] Initialize ChainTracker from storage (last N headers)
  - [ ] Seamless handoff without missing blocks

#### Phase 6 — Metrics and observability
- [ ] **Add live follow metrics** in `node/src/metrics.rs`
  - [ ] `indexed_block` gauge
  - [ ] `reorgs_total` counter
  - [ ] `reorg_depth` histogram
  - [ ] `blocks_behind` gauge
  - [ ] `network_head` gauge
  - [ ] `poll_cycles_total` counter
  - [ ] `follow_blocks_ingested` counter
- [ ] **Add structured logging**
  - [ ] Log each block ingestion at DEBUG
  - [ ] Log reorg detection at WARN
  - [ ] Log deep reorg at ERROR
  - [ ] Log poll cycle stats periodically

#### Phase 7 — RPC updates for live mode
- [ ] **Update `eth_blockNumber`** in `node/src/rpc/mod.rs`
  - [ ] Return indexed head (can decrease after rollback)
- [ ] **Update `eth_getBlockByNumber`**
  - [ ] Handle requests for non-indexed blocks gracefully
- [ ] **Update `eth_getLogs`**
  - [ ] Cap `toBlock` at indexed head
  - [ ] Handle rolled-back block ranges

#### Phase 8 — End-to-end testing
- [ ] **Integration test: normal follow mode**
  - [ ] Verify blocks are ingested continuously
  - [ ] Verify stays `rollback_window` behind head
- [ ] **Integration test: reorg handling**
  - [ ] Simulate a reorg by feeding conflicting blocks
  - [ ] Verify rollback occurs
  - [ ] Verify re-ingestion from common ancestor
- [ ] **Integration test: deep reorg**
  - [ ] Simulate reorg deeper than rollback_window
  - [ ] Verify error is raised
  - [ ] Verify storage is not corrupted

#### Phase 9 — Documentation and cleanup
- [ ] **Update README.md**
  - [ ] Document live follow mode
  - [ ] Document reorg handling
  - [ ] Document CLI flags
- [ ] **Update ROADMAP.md**
  - [ ] Mark v0.1.6 items as complete
- [ ] **Code cleanup**
  - [ ] Remove any temporary/debug code
  - [ ] Ensure consistent error handling
  - [ ] Add doc comments to public APIs

---

### Acceptance criteria (explicit)

- [ ] **Live follow works**: node continuously ingests blocks after historical sync
- [ ] **Safe head respected**: never indexes blocks within `rollback_window` of network head
- [ ] **Reorg detection works**: detects when new block's parent doesn't match stored head
- [ ] **Rollback works**: successfully truncates storage to common ancestor
- [ ] **Re-ingestion works**: after rollback, correctly re-ingests new chain
- [ ] **Deep reorg handled**: errors out cleanly for reorgs beyond rollback_window
- [ ] **RPC correct**: `eth_blockNumber` returns indexed head (can decrease)
- [ ] **Metrics exposed**: all live follow metrics are exported
- [ ] **Graceful shutdown**: SIGINT/SIGTERM cleanly exits without data corruption
- [ ] **Rindexer compatible**: reorg handling matches rindexer's expectations

---

### Risks and mitigations

- **Risk: Frequent small reorgs causing thrashing**
  - Mitigation: `rollback_window` keeps us far enough behind that most reorgs resolve before we index

- **Risk: Storage truncation not atomic**
  - Mitigation: Implement proper transactional rollback; use checkpoints to detect incomplete rollbacks on restart

- **Risk: ChainTracker memory growth**
  - Mitigation: Strict pruning to `rollback_window` headers; use HeaderStub (minimal data)

- **Risk: Network partition causes us to follow wrong fork**
  - Mitigation: Multiple peer validation; prefer peers with higher total difficulty; alert on unusual behavior

- **Risk: NippyJar truncation bugs**
  - Mitigation: Extensive testing; verify data integrity after truncation; checksums

---

### Future considerations (out of scope for v0.1.6)

- **Finality integration**: use finalized head from CL when available
- **Optimistic indexing**: index tentatively and mark as "pending" until confirmed
- **Fork choice awareness**: handle multiple competing chain tips
- **Checkpointing**: periodic snapshots for faster recovery
- **Archive node sync**: supporting full historical + live for archive use cases
