## Where the “UI bars” live
- **All terminal UI is in** `node/src/main.rs` (indicatif progress bars + two custom ANSI “color bars”).
- **The state it renders is defined in** `node/src/sync/mod.rs` (`SyncStatus`, `SyncProgressStats`).
- **That state is driven by** `node/src/sync/historical/mod.rs` (fast-sync pipeline) and `node/src/sync/historical/follow.rs` (follow loop).

## What bars/stages exist, and when they show

### 1) Yellow startup status bar (single-line, overwrites itself)
**What it is**: `print_status_bar()` prints a 50-char black-on-yellow bar to **stderr** using `\r` (no newline).  
**When it shows (startup phases only)**:
- **“Opening storage…”**: right before `Storage::open()`
- **“Compacting N shards…”**: only if `storage.dirty_complete_shards()` is non-empty
- **“Connecting to P2P network…”**: before `connect_mainnet_peers()`
- **“Waiting for peers… x/y”**: while `peer_pool.len() < --min-peers`
- **“Discovering chain head… {best_head}”**: while `best_head - rollback_window < start_block` (so the node can safely start)

### 2) Cyan/blue progress bar (fast-sync “main bar”)
**What it is**: indicatif bar `"{bar:40.cyan/blue} … | {msg}"` drawn to **stderr**, updated ~10Hz.  
**When it shows**:
- Only when **`stderr` is a TTY** (`std::io::stderr().is_terminal()`).
- It is created after computing `missing_ranges_in_range()`; its initial length is `missing_total` for the initial target range.

### 3) Red/black “failed recovery” bar
**What it is**: a second indicatif bar `"{bar:40.red/black} {pos}/{len} | {msg}"` (drawn ~2Hz).  
**What it actually tracks**:
- Not “any transient fetch failure”.
- It tracks **blocks that exceeded `max_attempts_per_block` and were put into the scheduler’s “failed set”**, and then counts them down as they’re recovered.

**Why you may never see it**:
- Default `max_attempts_per_block` is **3** (fast-sync); many runs won’t hit it.
- In **follow mode**, the code sets `max_attempts_per_block = u32::MAX`, so blocks **never** become “failed” near the tip → the red bar won’t appear during following.

### 4) Green “follow/status” bar (single-line, always includes a green background segment)
**What it is**: a custom-formatted message (not a normal progress bar) that includes a green `[ head_block ]` segment plus text like “Synced / Catching up / Finalizing”. Updated ~2Hz.  
**When it shows**:
- Only after `spawn_progress_updater()` decides **“initial sync done”** via **`processed >= total_len`** (where `processed` is `SyncProgressStats.processed`).

This is the logic that flips from the cyan bar to the green bar:

```2344:2395:/Users/vicnaum/github/stateless-history-node/node/src/main.rs
                // Transition to follow mode when initial sync completes
                if processed >= total_len {
                    initial_sync_done = true;
                    bar.finish_and_clear();

                    // Create the live follow status bar (message only, we format the bar ourselves)
                    let fb = ProgressBar::with_draw_target(
                        Some(100),
                        ProgressDrawTarget::stderr_with_hz(2),
                    );
                    let style = ProgressStyle::with_template("{msg}").expect("progress style");
                    fb.set_style(style);
                    follow_bar = Some(fb);
                }
            } else {
                // Live follow mode - show synced status with block number in a colored bar
                if let Some(ref fb) = follow_bar {
                    let head_block = snapshot.head_block;
                    let head_seen = snapshot.head_seen;
                    let peers_connected = peer_pool.len() as u64;
                    let peers_available = snapshot.peers_total.min(peers_connected);
                    let active_fetch = snapshot.peers_active;

                    let (status_str, status_detail) = match snapshot.status {
                        SyncStatus::Following | SyncStatus::UpToDate => ("Synced", String::new()),
                        SyncStatus::Fetching => (
                            "Catching up",
                            format!(" ({} blocks left)", head_seen.saturating_sub(head_block)),
                        ),
                        SyncStatus::Finalizing => {
                            ("Finalizing", " (flushing/compacting...)".to_string())
                        }
                        SyncStatus::LookingForPeers => ("Waiting for peers", String::new()),
                    };

                    // ANSI: force truecolor white text on truecolor green background, then reset.
                    let bar = format!(
                        "\x1b[38;2;255;255;255;48;2;0;128;0m{:>width_l$}{}{:<width_r$}\x1b[0m",
                        "",
                        content,
                        "",
                        width_l = left_pad,
                        width_r = right_pad,
                    );
```

## Sync “stages” (the `SyncStatus` states) and how they’re set
- **LookingForPeers**:
  - Default at UI init.
  - Set in follow loop when head discovery returns `None` (no usable peers).
- **Fetching**:
  - Set at start of `run_ingest_pipeline()` when there’s work.
  - Set again whenever tailing appends new ranges while following.
- **Finalizing**:
  - Set once fetch tasks are done, while draining processor workers + flushing DB writer + (fast-sync) compaction/sealing.
- **UpToDate**:
  - Set when there are **zero missing blocks** and no tailing.
  - Also used in follow loop when `start > head` between epochs.
- **Following**:
  - Used in follow-mode ingest pipeline when it’s caught up and is basically waiting for more safe head (tailing mode).

## Why “Finalizing shows as status finalizing but not as a green bar” can happen
The green bar switch is gated on **`processed >= total_len`**, but `processed` only increments on *successful* block processing.

This is where `processed` is incremented (only on `process_ingest(..) == Ok(..)`):

```433:456:/Users/vicnaum/github/stateless-history-node/node/src/sync/historical/mod.rs
                let result = span.in_scope(|| process_ingest(payload, bench_ref));
                match result {
                    Ok((bundle, log_count)) => {
                        logs_total.fetch_add(log_count, Ordering::SeqCst);
                        if let Some(progress) = progress.as_ref() {
                            progress.inc(1);
                        }
                        if let Some(stats) = stats.as_ref() {
                            stats.inc_processed(1);
                            stats.update_head_block_max(bundle.number);
                        }
                        if tx.send(DbWriterMessage::Block(bundle)).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "ingest processing failed");
                    }
```

And `process_ingest()` can fail (one concrete reason is tx-count vs receipts-count mismatch):

```87:102:/Users/vicnaum/github/stateless-history-node/node/src/sync/historical/process.rs
    let tx_hashes = body
        .transactions
        .iter()
        .map(|tx| tx_hash_fast(&mut tx_hasher, tx))
        .collect::<Vec<_>>();
    if tx_hashes.len() != receipts.len() {
        return Err(eyre!(
            "tx hash count {} does not match receipts count {} for block {}",
            tx_hashes.len(),
            receipts.len(),
            header.number
        ));
    }
```

So one likely real-world failure mode is:
- A block is considered “completed” by the fetch/scheduler side, so the pipeline can reach **Finalizing**,
- but **processing failed for ≥1 block**, so `processed` never reaches `total_len`,
- therefore **the UI never transitions to the green follow bar** (it stays in the cyan bar showing `status finalizing`).

If you’ve ever seen `warn: ingest processing failed` in logs around those runs, that would directly match this.

## Why the UI feels “shaky”
Two structural reasons in the current design:
- **Mixed rendering systems**: raw `\r` + ANSI (yellow bar) plus indicatif bars, all writing to stderr.
- **Multiple independent indicatif `ProgressBar`s without a `MultiProgress` coordinator** (main bar + failed bar + follow bar). They can compete for the same terminal lines, especially at different update Hz.

If you want, next step can be: decide what the “single source of truth” for UI completion should be (scheduler completion vs processed vs DB-committed), then we can design a small dedicated UI module around that.