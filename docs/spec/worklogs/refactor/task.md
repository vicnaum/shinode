I want you to compare the code of @harness and the code of @node  - because we're having problems with historical fetching. I asked to implement the same fetching strategy in node as we had in harness - it was fast (1200 blocks per second) and multi-peer, etc etc. But in node we have it superslow for some reason, and I don't know why.
I want you to compare the two approaches and tell me what's the differences and where the speed is lost. Node implements some weird things like batches/etc while I feel harness was just working block by block (only requested many blocks/receipts from peers in a batch to save requests, but processing was done per block I think).
We implemented the --debug-receipts-only flag for node to mimic the resourses of harness (cause it didn't save the blocks/receipts, just fetched them and counted) - but even with that flag - fetching 11000 blocks was taking the node 3 minutes (while the harness could done the same in 11 seconds).
Please compare the two fetching mechanisms and enlighten what can be the problem

---

I compared the two fetch paths. They’re not equivalent: `harness/` is a lightweight, peer‑driven probe that fetches headers/receipts and logs counts, while `node/` is a full ingest pipeline that enforces ordering, validates chain continuity, computes blooms, and persists to the DB even in `--debug-receipts-only`. That mismatch alone can explain the 10–20× gap you’re seeing.

## Key Differences (code‑backed)

- **Scheduling/parallelism model**
  - Harness keeps many peers busy concurrently: each ready peer gets a batch, a per‑peer task is spawned, and the peer is requeued immediately after completion.  
```125:175:harness/src/main.rs
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel::<PeerHandle>();

    let scheduler_context = Arc::clone(&context);
    let scheduler_ready_tx = ready_tx.clone();
    tokio::spawn(async move {
        let mut warmed_up = false;
        while let Some(peer) = ready_rx.recv().await {
            if !warmed_up {
                if WARMUP_SECS > 0 {
                    tokio::time::sleep(Duration::from_secs(WARMUP_SECS)).await;
                }
                warmed_up = true;
            }

            if let Some(remaining) = scheduler_context.peer_ban_remaining(&peer.peer_key).await {
                let ready_tx = scheduler_ready_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(remaining).await;
                    let _ = ready_tx.send(peer);
                });
                continue;
            }

            let batch = scheduler_context
                .next_batch_for_peer(&peer.peer_key, peer.head_number)
                .await;
            if batch.blocks.is_empty() {
                continue;
            }
            scheduler_context.record_peer_job(&peer.peer_key).await;
            scheduler_context.inc_in_flight();

            let context = Arc::clone(&scheduler_context);
            let ready_tx = scheduler_ready_tx.clone();
            tokio::spawn(async move {
                let peer_for_probe = peer.clone();
                let should_requeue = probe_block_batch(
                    peer_for_probe.peer_id,
                    peer_for_probe.peer_key,
                    peer_for_probe.messages,
                    peer_for_probe.eth_version,
                    batch.blocks,
                    batch.mode,
                    Arc::clone(&context),
                )
                .await;
                context.dec_in_flight();
                if should_requeue {
                    let _ = ready_tx.send(peer);
                }
            });
        }
    });
```
  - Node fetches ranges concurrently but **processes in order** and gates on `next_expected`, which can stall if any chunk is slow.  
```498:593:node/src/sync/mod.rs
    async fn run_fast_range(
        &mut self,
        storage: &Storage,
        range: RangeInclusive<u64>,
        progress: Option<&dyn ProgressReporter>,
        stats: Option<&SyncProgressStats>,
    ) -> Result<IngestOutcome> {
        let mut pending = VecDeque::from(chunk_ranges(&range, self.fast_sync.chunk_size));
        let mut in_flight: FuturesUnordered<
            tokio::task::JoinHandle<(RangeInclusive<u64>, Result<Vec<BlockPayload>>)>,
        > = FuturesUnordered::new();
        let mut buffered: BTreeMap<u64, Vec<BlockPayload>> = BTreeMap::new();
        let mut buffered_blocks = 0u64;
        let mut derived_logs = Vec::new();
        let mut next_expected = *range.start();
        let mut attempts: BTreeMap<u64, u32> = BTreeMap::new();
        const MAX_CHUNK_ATTEMPTS: u32 = 3;
        if let Some(stats) = stats {
            stats.set_queue(range_len(&range));
            stats.set_inflight(0);
        }

        while !pending.is_empty() || !in_flight.is_empty() || !buffered.is_empty() {
            let max_workers = self.fast_sync.max_inflight;
            while in_flight.len() < max_workers && !pending.is_empty() {
                let can_buffer_more = buffered_blocks < self.fast_sync.max_buffered_blocks
                    || !buffered.contains_key(&next_expected);
                if !can_buffer_more {
                    break;
                }
                let chunk = pending.pop_front().expect("pending is not empty");
                let source = self.source.clone();
                in_flight.push(tokio::spawn(async move {
                    let payloads = source.blocks_by_number(chunk.clone()).await;
                    (chunk, payloads)
                }));
            }
            if let Some(stats) = stats {
                stats.set_inflight(in_flight.len() as u64);
                let queue = pending.iter().map(range_len).sum::<u64>();
                stats.set_queue(queue);
            }

            let Some(result) = in_flight.next().await else {
                break;
            };
            let (chunk, payloads) = result.map_err(|err| eyre::eyre!("chunk task failed: {err}"))?;
            match payloads {
                Ok(mut payloads) => {
                    if let Some(stats) = stats {
                        stats.inc_processed(payloads.len() as u64);
                    }
                    payloads.sort_by_key(|payload| payload.header.number);
                    buffered_blocks = buffered_blocks.saturating_add(payloads.len() as u64);
                    buffered.insert(*chunk.start(), payloads);
                }
                Err(err) => {
                    let key = *chunk.start();
                    let attempt = attempts.entry(key).or_insert(0);
                    *attempt += 1;
                    if *attempt <= MAX_CHUNK_ATTEMPTS {
                        pending.push_back(chunk);
                        continue;
                    }
                    return Err(err);
                }
            }

            while let Some(payloads) = buffered.remove(&next_expected) {
                let mut bundles = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    match self.process_payload(storage, payload)? {
                        PayloadResult::Reorg { ancestor_number } => {
                            if let Some(progress) = progress {
                                progress.finish();
                            }
                            return Ok(IngestOutcome::Reorg { ancestor_number });
                        }
                        PayloadResult::Bundle { bundle, logs } => {
                            bundles.push(bundle);
                            derived_logs.extend(logs);
                            if let Some(progress) = progress {
                                progress.inc(1);
                            }
                        }
                    }
                }
                buffered_blocks = buffered_blocks.saturating_sub(bundles.len() as u64);
                if !bundles.is_empty() {
                    next_expected = bundles
                        .last()
                        .expect("bundles is not empty")
                        .number
                        .saturating_add(1);
                    storage.write_block_bundle_batch(&bundles)?;
                    if let Some(stats) = stats {
                        let remaining = if next_expected <= *range.end() {
                            range
                                .end()
                                .saturating_sub(next_expected)
                                .saturating_add(1)
                        } else {
                            0
                        };
                        stats.set_queue(remaining);
                    }
                }
            }
        }
```

- **Work per block**
  - Harness just seals headers to compute hashes and requests receipts in chunks; it’s essentially a probe/logger with no heavy derivation or persistence in the hot path.  
```834:945:harness/src/main.rs
    let mut hashes = Vec::new();
    let mut missing = Vec::new();
    for block in &blocks {
        match headers_by_number.remove(block) {
            Some(header) => {
                if header.number != *block {
                    warn!(
                        peer_id = ?peer_id,
                        requested = block,
                        received = header.number,
                        "probe header mismatch"
                    );
                }
                let hash = SealedHeader::seal_slow(header).hash();
                hashes.push((*block, hash));
            }
            None => missing.push(*block),
        }
    }

    if !missing.is_empty() {
        warn!(
            peer_id = ?peer_id,
            missing = missing.len(),
            "probe header missing in batch"
        );
        for block in &missing {
            let (attempt, retry) = if is_escalation {
                context.record_failure_reason("escalation_missing_header").await;
                context.requeue_escalation_block(*block).await;
                (None, false)
            } else {
                let attempt = context.record_attempt(*block).await;
                let retry = attempt <= context.config.max_attempts;
                if retry {
                    context.requeue_blocks(&[*block]).await;
                    context.record_retry().await;
                } else {
                    context.mark_failed_block(*block).await;
                }
                context.record_failure_reason("missing_header").await;
                (Some(attempt), retry)
            };
            context
                .stats
                .lock()
                .await
                .record_probe(&peer_key, *block, false, false, header_ms, None);
            context
                .update_window(&peer_key, header_ms, None, false)
                .await;
            context
                .update_window(&peer_key, header_ms, None, false)
                .await;
            let payload = json!({
                "event": "probe",
                "peer_id": format!("{:?}", peer_id),
                "block": block,
                "eth_version": format!("{eth_version}"),
                "header_ok": false,
                "receipts_ok": false,
                "header_ms": header_ms,
                "attempt": attempt,
                "will_retry": retry,
                "mode": if is_escalation { "escalation" } else { "normal" },
                "error": "missing_header",
            });
            if !context.config.quiet && context.config.verbosity >= Verbosity::V3 {
                println!("{}", payload);
            }
            context.log_probe(payload).await;
        }
    }

    for chunk in hashes.chunks(context.config.receipts_per_request.max(1)) {
        let chunk_blocks: Vec<u64> = chunk.iter().map(|(block, _)| *block).collect();
        let chunk_hashes: Vec<B256> = chunk.iter().map(|(_, hash)| *hash).collect();
        let receipts_start = Instant::now();
        let receipts_result = match eth_version {
            EthVersion::Eth69 => {
                request_receipts69(
                    peer_id,
                    eth_version,
                    &messages,
                    &chunk_blocks,
                    &chunk_hashes,
                    context.as_ref(),
                )
                .await
            }
            EthVersion::Eth70 => {
                request_receipts70(
                    peer_id,
                    eth_version,
                    &messages,
                    &chunk_blocks,
                    &chunk_hashes,
                    context.as_ref(),
                )
                .await
            }
            _ => {
                request_receipts_legacy(
                    peer_id,
                    eth_version,
                    &messages,
                    &chunk_blocks,
                    &chunk_hashes,
                    context.as_ref(),
                )
                .await
            }
        };
```
  - Node **receipts‑only** still does chain insertion, bloom computation, and DB writes per batch.  
```831:885:node/src/sync/mod.rs
            while let Some(payloads) = buffered.remove(&next_expected) {
                let mut bundles = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    let ReceiptPayload { header, receipts } = payload;
                    let header_hash = SealedHeader::seal_slow(header.clone()).hash();
                    let header_stub = HeaderStub {
                        number: header.number,
                        hash: header_hash,
                        parent_hash: header.parent_hash,
                    };
                    match self.chain.insert_header(header_stub) {
                        Ok(ChainUpdate::Reorg { ancestor_number, .. }) => {
                            storage.rollback_to(ancestor_number)?;
                            if let Some(progress) = progress {
                                progress.finish();
                            }
                            return Ok(IngestOutcome::Reorg { ancestor_number });
                        }
                        Ok(_) => {}
                        Err(ChainError::UnknownParent(parent)) => {
                            return Err(eyre::eyre!("unknown parent {parent}"));
                        }
                        Err(ChainError::NonContiguousNumber { expected, got }) => {
                            return Err(eyre::eyre!(
                                "non-contiguous header number: expected {expected}, got {got}"
                            ));
                        }
                    }

                    let computed_bloom = logs_bloom(
                        receipts
                            .iter()
                            .flat_map(|receipt| receipt.logs.iter().map(|log| log.as_ref())),
                    );
                    let mut stored_header = header.clone();
                    stored_header.logs_bloom = computed_bloom;
                    bundles.push(ReceiptBundle {
                        number: header.number,
                        header: stored_header,
                        receipts: StoredReceipts { receipts },
                    });
                    if let Some(progress) = progress {
                        progress.inc(1);
                    }
                }

                buffered_blocks = buffered_blocks.saturating_sub(bundles.len() as u64);
                if !bundles.is_empty() {
                    next_expected = bundles
                        .last()
                        .expect("bundles is not empty")
                        .number
                        .saturating_add(1);
                    storage.write_header_receipts_batch(&bundles)?;
                    if let Some(stats) = stats {
                        let remaining = if next_expected <= *range.end() {
                            range
                                .end()
                                .saturating_sub(next_expected)
                                .saturating_add(1)
                        } else {
                            0
                        };
                        stats.set_queue(remaining);
                    }
                }
            }
```

- **Peer usage and retries**
  - Node selects a single peer per range attempt and retries sequentially with backoff. That limits per‑range parallelism and makes you sensitive to slow peers.  
```246:285:node/src/p2p/mod.rs
    async fn blocks_by_number(&self, range: std::ops::RangeInclusive<u64>) -> Result<Vec<BlockPayload>> {
        let mut last_err = None;
        let attempts = self.pool.attempts();

        for attempt in 0..attempts {
            let Some(peer) = self.pool.next_peer() else {
                return Err(eyre!("no peers available for request"));
            };
            match fetch_payloads_for_peer(&peer, range.clone()).await {
                Ok(payloads) => return Ok(payloads),
                Err(err) => {
                    last_err = Some(err);
                    if attempt + 1 < attempts {
                        sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| eyre!("all peer attempts failed")))
    }

    async fn receipts_only_by_number(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<ReceiptPayload>> {
        let mut last_err = None;
        let attempts = self.pool.attempts();

        for attempt in 0..attempts {
            let Some(peer) = self.pool.next_peer() else {
                return Err(eyre!("no peers available for request"));
            };
            match fetch_receipts_only_for_peer(&peer, range.clone()).await {
                Ok(payloads) => return Ok(payloads),
                Err(err) => {
                    last_err = Some(err);
                    if attempt + 1 < attempts {
                        sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
                    }
                }
            }
        }
```

- **Extra hashing (done twice in node)**
  - Node hashes headers in the P2P fetch to request receipts, then hashes again in the receipts‑only processing step.  
```452:468:node/src/p2p/mod.rs
async fn fetch_receipts_only_for_peer(
    peer: &NetworkPeer,
    range: std::ops::RangeInclusive<u64>,
) -> Result<Vec<ReceiptPayload>> {
    let start = *range.start();
    let end = *range.end();
    let count = (end - start + 1) as usize;

    let headers = request_headers_chunked(peer, start, count).await?;

    let mut hashes = Vec::with_capacity(count);
    for header in &headers {
        let hash = SealedHeader::seal_slow(header.clone()).hash();
        hashes.push(hash);
    }

    let receipts = request_receipts_chunked(peer, &hashes).await?;
```

## Where the speed is likely lost in `node/`

- **DB persistence + bloom computation per block** even in receipts‑only mode (see `logs_bloom` + `write_header_receipts_batch`) — these are absent in harness.  
```831:885:node/src/sync/mod.rs
            while let Some(payloads) = buffered.remove(&next_expected) {
                let mut bundles = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    let ReceiptPayload { header, receipts } = payload;
                    let header_hash = SealedHeader::seal_slow(header.clone()).hash();
                    let header_stub = HeaderStub {
                        number: header.number,
                        hash: header_hash,
                        parent_hash: header.parent_hash,
                    };
                    match self.chain.insert_header(header_stub) {
                        Ok(ChainUpdate::Reorg { ancestor_number, .. }) => {
                            storage.rollback_to(ancestor_number)?;
                            if let Some(progress) = progress {
                                progress.finish();
                            }
                            return Ok(IngestOutcome::Reorg { ancestor_number });
                        }
                        Ok(_) => {}
                        Err(ChainError::UnknownParent(parent)) => {
                            return Err(eyre::eyre!("unknown parent {parent}"));
                        }
                        Err(ChainError::NonContiguousNumber { expected, got }) => {
                            return Err(eyre::eyre!(
                                "non-contiguous header number: expected {expected}, got {got}"
                            ));
                        }
                    }

                    let computed_bloom = logs_bloom(
                        receipts
                            .iter()
                            .flat_map(|receipt| receipt.logs.iter().map(|log| log.as_ref())),
                    );
                    let mut stored_header = header.clone();
                    stored_header.logs_bloom = computed_bloom;
                    bundles.push(ReceiptBundle {
                        number: header.number,
                        header: stored_header,
                        receipts: StoredReceipts { receipts },
                    });
                    if let Some(progress) = progress {
                        progress.inc(1);
                    }
                }

                buffered_blocks = buffered_blocks.saturating_sub(bundles.len() as u64);
                if !bundles.is_empty() {
                    next_expected = bundles
                        .last()
                        .expect("bundles is not empty")
                        .number
                        .saturating_add(1);
                    storage.write_header_receipts_batch(&bundles)?;
                    if let Some(stats) = stats {
                        let remaining = if next_expected <= *range.end() {
                            range
                                .end()
                                .saturating_sub(next_expected)
                                .saturating_add(1)
                        } else {
                            0
                        };
                        stats.set_queue(remaining);
                    }
                }
            }
```

- **In‑order processing stall**: node won’t apply any buffered chunk until `next_expected` arrives; a single slow chunk stalls all later chunks.  
```498:593:node/src/sync/mod.rs
        while !pending.is_empty() || !in_flight.is_empty() || !buffered.is_empty() {
            let max_workers = self.fast_sync.max_inflight;
            while in_flight.len() < max_workers && !pending.is_empty() {
                let can_buffer_more = buffered_blocks < self.fast_sync.max_buffered_blocks
                    || !buffered.contains_key(&next_expected);
                if !can_buffer_more {
                    break;
                }
                let chunk = pending.pop_front().expect("pending is not empty");
                let source = self.source.clone();
                in_flight.push(tokio::spawn(async move {
                    let payloads = source.blocks_by_number(chunk.clone()).await;
                    (chunk, payloads)
                }));
            }
            ...
            while let Some(payloads) = buffered.remove(&next_expected) {
                let mut bundles = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    match self.process_payload(storage, payload)? {
                        ...
                    }
                }
                ...
                if !bundles.is_empty() {
                    ...
                    storage.write_block_bundle_batch(&bundles)?;
                    ...
                }
            }
        }
```

- **Single‑peer per range + sequential retries/backoff**: if that chosen peer is slow, the whole chunk is slow.  
```246:285:node/src/p2p/mod.rs
    async fn blocks_by_number(&self, range: std::ops::RangeInclusive<u64>) -> Result<Vec<BlockPayload>> {
        let mut last_err = None;
        let attempts = self.pool.attempts();

        for attempt in 0..attempts {
            let Some(peer) = self.pool.next_peer() else {
                return Err(eyre!("no peers available for request"));
            };
            match fetch_payloads_for_peer(&peer, range.clone()).await {
                Ok(payloads) => return Ok(payloads),
                Err(err) => {
                    last_err = Some(err);
                    if attempt + 1 < attempts {
                        sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
                    }
                }
            }
        }
```

- **Duplicate header hashing**: node seals headers in the P2P fetch and then again during receipts‑only processing.  
```452:468:node/src/p2p/mod.rs
    let headers = request_headers_chunked(peer, start, count).await?;

    let mut hashes = Vec::with_capacity(count);
    for header in &headers {
        let hash = SealedHeader::seal_slow(header.clone()).hash();
        hashes.push(hash);
    }

    let receipts = request_receipts_chunked(peer, &hashes).await?;
```
```831:841:node/src/sync/mod.rs
                for payload in payloads {
                    let ReceiptPayload { header, receipts } = payload;
                    let header_hash = SealedHeader::seal_slow(header.clone()).hash();
                    let header_stub = HeaderStub {
                        number: header.number,
                        hash: header_hash,
                        parent_hash: header.parent_hash,
                    };
```

## Bottom line

`harness/` is essentially a high‑throughput network probe with minimal CPU/IO, while `node/` still does **chain validation + bloom computation + DB writes + ordered application**, even in receipts‑only mode. Those are the most probable culprits for the 11s vs 3m gap.

If you want, I can next:
- Instrument `node/` to time **network fetch vs processing vs DB commits**, or
- Propose a true “probe mode” in `node/` that skips chain insert + bloom + DB writes to more closely match harness behavior.

---

Can we do the following:
1) List the things that need to be changed to make the node process stuff in the same way as Harness does - scheduling/parallelism, work per block toggle (harness-probe style VS full processing), peer usage and retries, etc)
2) List the problems in node: extra hashing, in-order processing stall, single-peer per range + sequential retries/backoff, duplicate header hashing (which shouldn't be done at all if we have true probe-mode)
3) Make a plan on fully isolating and rewriting the historical fetching
4) Try to envision/brainstorm some kind of an independent "block processing" from "block fetching". I.e. queue for block fetching (as in harness) is processed independently from queue of "block processing" (hashing, bloom, saving to db, etc). So the flow must be this -> we have a number of blocks from the range that are getting in the queue for fetching -> after fetching each block is put in the queue for processing. All that should happen in memory and in parallel (independently), in an async way (in parallel), so multi-threading could be used here for processing, etc. And also these should be different kinds of functions/modules/parts, that are easier to inspect/code/replace. Don't worry about reorgs - cause this applies only for historical sync so far (which is older than reorg threshold)
5) Let's replace --debug-receipts-only mode with --probe-mode or even better: --benchmark mode. --benchmark [type]. And types could be: [probe] or [harness] - for the behavior harness has (only applies fetching of headers and receipts (check what harness fetches), and not saving to db, measures stats the harness measures - and outputs it after finishing), then could be some other mode that runs the full processing, but also measures each part independently - how long fetching takes, how long bloom generation, how long hashing of something, how long saving to db, etc - to determine the bottleneck (but this we can do later, for now just probe-mode please). This probe-mode should be applied during the "block processing" queue - and just turn off parts of the processing (or all - just mark it done - depending on what harness does).
6) Please brainstorm all this well, and let's construct some kind of a spec/plan for this refactor - cause we also have tests, and we probably have to change many things - please be VERY specific and detailed in your planning - so each item would be self-explanatory and I should not double-guess that "oh, it says this, but I should had also done this!" - no, everything that should be done - must be in the plan.

Please try to do this good!