# Performance Notes

This file is a running log of notable benchmark runs, key throughput numbers, and suspected
bottlenecks. The goal is to make it easy to compare versions/flags over time.

## How To Add A New Run

- Run an ingest benchmark with `--benchmark-events` and (optionally) `--benchmark-trace`.
- Capture shard sizes after the run (example):

```bash
du -h benchmarks/data/<RUN_ID>/static/shards/* | sort -h > benchmarks/file_sizes.txt
```

- Paste a new section below and fill in the numbers from the benchmark summary JSON plus
  a quick scan of the `.events.jsonl` (compaction + resource stats).

## Hetzner ingest-1M (v0.2) - 2026-01-24T14:00:54Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2__20260124T140054Z__range-23283452-24283451__chunk16__chunkmax64__inflight15__timeout5000__profile-release__alloc-system.json`
- `benchmarks_hetzner/ingest_1M_v0_2__20260124T140054Z__range-23283452-24283451__chunk16__chunkmax64__inflight15__timeout5000__profile-release__alloc-system.events.jsonl`
- `benchmarks_hetzner/file_sizes.txt`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=16, chunk_max=64, inflight=15, timeout=5000ms, lookahead=50,000, buffered=2048
- db writer: batch_blocks=512
- build: release, allocator=system

Top-line results (from summary JSON):
- elapsed: 1,698.054s (28m 18s)
- throughput: 588.9 blocks/s avg, 469,794 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,735,611 logs total
- fetch failures: 13,367 failed blocks across 6,254 failures; 0 process failures
- peers: 47 peers_used; 278 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 2.34 GiB; swap max: 0
- cpu_busy_pct: avg 19.8%, p95 27.9%, max 37.8%
- cpu_iowait_pct: avg 0.24%, p95 1.10% (spiky during final compactions)

Fetch characteristics:
- 119,205 batches total; avg 273,352 bytes/block (headers+bodies+receipts payload sizes)
- per-batch blocks: p50=9, p95=14, max=20
- per-batch duration_ms: p50=41ms, p95=889ms, p99=1601ms, max=4939ms
- timeouts: 6 (`timeout_ms=5000`)

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,275us, p95=2,761us, p99=3,823us, max=16,435us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167
- avg processing breakdown (us/block): header_hash=10, tx_hashes=649, transactions=557, block_size=136

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=334ms, p95=434ms, max=621ms
- flush size (bytes_total): p50=107.8 MiB, p95=205.0 MiB, max=334.8 MiB
- flush throughput: p50=327.6 MiB/s, p95=617.4 MiB/s

Compaction characteristics:
- shard dirs: 101/101 have `sorted/`, 0 `staging.wal` left (verified via `benchmarks_hetzner/file_sizes.txt`)
- compaction duration_ms: p50=1002ms, p95=1753ms, p99=1851ms, max=3444ms
- range completion (`pending_total=0`) at ~1680.1s; 89/101 compactions finished by then
- finalizing time was dominated by the remaining compactions; `compact_all_dirty` window: 18.48s

Notable dynamics / bottleneck guess:
- This run is fetch-bound, not CPU-bound:
  - fetch total_us / wall_clock ~= 13.7 average parallel fetches (close to `--fast-sync-max-inflight=15`)
  - cpu_busy_pct stays <30% most of the time
  - DB flush p95 ~434ms with high MiB/s; iowait is low except during final compactions
- Throughput dips line up with “heavier” blocks, not a systemic stall:
  - 60s windows show bytes/block varying from ~213k to ~451k
  - correlation(bytes_per_block, blocks/s) ~= -0.65 (negative: bigger blocks -> fewer blocks/s)

If we want to go faster (hypotheses to test):
- Increase `--fast-sync-max-inflight` (and possibly peers) to see if blocks/s scales while CPU stays low.
- Increase `--fast-sync-chunk-size` / `--fast-sync-chunk-max` to raise avg blocks/batch (currently ~9 vs chunk_size=16).
- Improve peer selection / reduce long-tail batch latencies (p95 fetch batch is ~0.9s, max ~4.9s).

## Hetzner ingest-1M (v0.2, jemalloc) - 2026-01-24T15:00:45Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2_jemalloc__20260124T150045Z__range-23283452-24283451__chunk16__chunkmax64__inflight15__timeout5000__profile-release__alloc-jemalloc.json`
- `benchmarks_hetzner/ingest_1M_v0_2_jemalloc__20260124T150045Z__range-23283452-24283451__chunk16__chunkmax64__inflight15__timeout5000__profile-release__alloc-jemalloc.events.jsonl`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=16, chunk_max=64, inflight=15, timeout=5000ms, lookahead=50,000, buffered=2048
- db writer: batch_blocks=512
- build: release, allocator=jemalloc

Top-line results (from summary JSON):
- elapsed: 1,416.141s (23m 36s)
- throughput: 706.1 blocks/s avg, 563,321 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,742,245 logs total
- fetch failures: 13,103 failed blocks across 6,071 failures; 0 process failures
- peers: 58 peers_used; 140 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 1.91 GiB; swap max: 0
- cpu_busy_pct: avg 23.3%, p95 32.2%, max 44.9%
- cpu_iowait_pct: p95 1.34%

Fetch characteristics:
- 118,066 batches total; avg 273,353 bytes/block (headers+bodies+receipts payload sizes)
- per-batch blocks: p50=9, p95=14, max=21
- per-batch duration_ms: p50=68ms, p95=583ms, p99=1150ms, max=4978ms
- timeouts: 14 (`timeout_ms=5000`)

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,218us, p95=2,723us, p99=3,825us, max=18,521us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=334ms, p95=429ms, max=617ms
- flush size (bytes_total): p50=107.9 MiB, p95=207.3 MiB, max=337.1 MiB
- flush throughput: p50=329.2 MiB/s, p95=632.2 MiB/s

Compaction characteristics:
- compaction_end count: 101
- compaction duration_ms: p50=1033ms, p95=1770ms, max=1891ms
- range completion (`pending_total=0`) at ~1410.1s; 96/101 compactions finished by then
- `compact_all_dirty` window: 13.13s
- NOTE: `benchmarks_hetzner/file_sizes.txt` in this repo snapshot only contains the 14:00:54Z run paths, so the on-disk shard/WAL cleanup for this run is inferred from events + summary, not verified from `du` output here.

Comparison vs 2026-01-24T14:00:54Z (system allocator):
- wall time: 1698s -> 1416s (faster by ~16.6%)
- throughput: 588.9 -> 706.1 blocks/s (~+20%)
- fetch avg_us/block: 23,230 -> 20,634 (~-11%)
- fetch p95 request latencies improved significantly (headers/bodies/receipts p95: 283/430/656ms -> 129/293/416ms)
- peers_used increased (47 -> 58) and peer_failures_total decreased (278 -> 140)
- rss max decreased (2.34 -> 1.91 GiB)

Attribution caveat:
- The throughput gain is dominated by “better fetch” (lower avg/p95 fetch times + more usable peers). That may reflect network/peer variance in addition to allocator choice; repeat A/B runs to isolate jemalloc’s impact.

## Hetzner ingest-1M (v0.2, inflight30/chunk32) - 2026-01-24T15:31:21Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2_inflight30_chunk32__20260124T153121Z__range-23283452-24283451__chunk32__chunkmax128__inflight30__timeout10000__profile-release__alloc-system.json`
- `benchmarks_hetzner/ingest_1M_v0_2_inflight30_chunk32__20260124T153121Z__range-23283452-24283451__chunk32__chunkmax128__inflight30__timeout10000__profile-release__alloc-system.events.jsonl`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=32, chunk_max=128, inflight=30, timeout=10000ms, lookahead=50,000, buffered=4096
- db writer: batch_blocks=512
- build: release, allocator=system

Top-line results (from summary JSON):
- elapsed: 1,298.953s (21m 39s)
- throughput: 769.9 blocks/s avg, 614,140 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,738,745 logs total
- fetch failures: 14,587 failed blocks across 6,233 failures; 0 process failures
- peers: 51 peers_used; 284 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 3.74 GiB; swap max: 0
- cpu_busy_pct: avg 26.5%, p95 34.7%, max 44.6%
- cpu_iowait_pct: p95 1.51% (spiky during the last compactions / heavy write bursts)

Fetch characteristics:
- 118,256 batches total; avg 273,352 bytes/block (headers+bodies+receipts payload sizes)
- per-batch blocks: p50=9, p95=14, max=32
- per-batch duration_ms: p50=43ms, p95=837ms, p99=1607ms, max=5641ms
- timeouts: 0 (timeout was raised to 10s)

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,303us, p95=2,865us, p99=3,998us, max=16,820us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=351ms, p95=448ms, max=633ms
- flush throughput: p50=313.1 MiB/s, p95=596.3 MiB/s

Compaction characteristics:
- compaction_end count: 101
- compaction duration_ms: p50=1026ms, p95=1616ms, max=4259ms
- range completion (`pending_total=0`) at ~1270.1s; 80/101 compactions finished by then
- final compaction window (`compact_all_dirty` duration): 29.91s

Comparison vs v0.2 baseline (system allocator, inflight15/chunk16 @ 2026-01-24T14:00:54Z):
- throughput: 588.9 -> 769.9 blocks/s (~+30.7%)
- effective fetch parallelism (fetch_total_us / wall): ~13.7 -> ~18.0 (closer to the higher inflight cap)
- rss max: 2.34 -> 3.74 GiB (still safe)
- finalizing overhead increased (more shards compacting after `pending_total=0`): 18.48s -> 29.91s

Working theory:
- This tuning improves throughput primarily by increasing concurrent fetch work (more peers active, more in-flight batches), not by making individual fetch requests faster.
- It also increases fetch failures/missing blocks slightly, which likely delays some shard completions and pushes more compactions into finalizing.

## Hetzner ingest-1M (v0.2, inflight30/chunk32, jemalloc) - 2026-01-24T16:00:02Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2_inflight30_chunk32_jemalloc__20260124T160002Z__range-23283452-24283451__chunk32__chunkmax128__inflight30__timeout10000__profile-release__alloc-jemalloc.json`
- `benchmarks_hetzner/ingest_1M_v0_2_inflight30_chunk32_jemalloc__20260124T160002Z__range-23283452-24283451__chunk32__chunkmax128__inflight30__timeout10000__profile-release__alloc-jemalloc.events.jsonl`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=32, chunk_max=128, inflight=30, timeout=10000ms, lookahead=50,000, buffered=4096
- db writer: batch_blocks=512
- build: release, allocator=jemalloc

Top-line results (from summary JSON):
- elapsed: 1,115.908s (18m 36s)
- throughput: 896.1 blocks/s avg, 714,881 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,736,289 logs total
- fetch failures: 14,367 failed blocks across 6,071 failures; 0 process failures
- peers: 54 peers_used; 221 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 5.25 GiB; swap max: 0
- cpu_busy_pct: avg 29.7%, p95 40.1%, max 49.5%
- cpu_iowait_pct: p95 1.50% (max 15.47%; spiky during heavy write bursts)
- peers_active: max 30, p95 30 (saturates `--fast-sync-max-inflight`)

Fetch characteristics:
- 117,271 batches total; avg 273,352 bytes/block (headers+bodies+receipts payload sizes)
- per-batch blocks: p50=9, p95=14, max=32
- per-batch duration_ms: p50=59ms, p95=873ms, p99=1887ms, max=6223ms
- request durations (p50/p95/p99): headers=5/165/281ms, bodies=36/471/1063ms, receipts=52/712/1585ms
- timeouts: 0 (`timeout_ms=10000`)

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,217us, p95=2,783us, p99=3,934us, max=17,378us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167
- avg processing breakdown (us/block): header_hash=5, tx_hashes=645, transactions=549, block_size=121

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=345ms, p95=449ms, max=1080ms
- flush size (bytes_total): p50=107.8 MiB, p95=203.0 MiB, max=333.8 MiB
- flush throughput: p50=318.0 MiB/s, p95=574.8 MiB/s

Compaction characteristics:
- compaction_end count: 101
- compaction duration_ms: p50=1073ms, p95=1750ms, max=4493ms
- range completion (`pending_total=0`) at ~1100.1s; 91/101 compactions finished by then
- final compaction window (`compact_all_dirty` duration): 20.82s
- NOTE: this repo snapshot does not include the `benchmarks/data/<run_id>` dir for this run, so shard/WAL cleanup is inferred from events + summary, not verified from `du` output.

Comparison vs 2026-01-24T15:31:21Z (system allocator, inflight30/chunk32):
- wall time: 1299s -> 1116s (faster by ~14.1%)
- throughput: 769.9 -> 896.1 blocks/s (~+16.4%)
- effective fetch parallelism (fetch_total_us / wall): ~18.0 -> ~22.9
- rss max: 3.74 -> 5.25 GiB
- compactions done by `pending_total=0`: 80/101 -> 91/101; final compaction window: 29.91s -> 20.82s

Attribution caveat (again):
- The throughput gain is mostly “more parallel fetch work” (peers_active saturates at 30 and effective fetch parallelism is ~22.9).
- This may reflect peer/network variance in addition to allocator choice; repeat A/B runs to isolate jemalloc’s impact.

## Hetzner ingest-1M (v0.2, inflight60/chunk64, jemalloc) - 2026-01-24T17:23:39Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2_inflight60_chunk64__20260124T172339Z__range-23283452-24283451__chunk64__chunkmax256__inflight60__timeout15000__profile-release__alloc-jemalloc.json`
- `benchmarks_hetzner/ingest_1M_v0_2_inflight60_chunk64__20260124T172339Z__range-23283452-24283451__chunk64__chunkmax256__inflight60__timeout15000__profile-release__alloc-jemalloc.events.jsonl`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=64, chunk_max=256, inflight=60, timeout=15000ms, lookahead=200,000, buffered=8192
- db writer: batch_blocks=512
- build: release, allocator=jemalloc

Top-line results (from summary JSON):
- elapsed: 1,106.129s (18m 26s)
- throughput: 904.0 blocks/s avg, 721,195 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,734,479 logs total
- fetch failures: 15,676 failed blocks across 5,977 failures; 0 process failures
- peers: 44 peers_used; 228 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 3.33 GiB; swap max: 0
- cpu_busy_pct: avg 29.6%, p95 38.6%, max 46.8%
- cpu_iowait_pct: avg 0.32%, p95 1.42% (max 16.44%; spiky during flush bursts)
- peers_active: max 22, p95 21

Fetch characteristics:
- 115,300 batches total; avg 273,352 bytes/block; download 15.26 MiB/s avg
- per-batch blocks: p50=9, p95=14, max=64
- per-batch duration_ms: p50=35ms, p95=646ms, p99=1491ms, max=6857ms
- request durations (p50/p95/p99): headers=3/161/299ms, bodies=22/335/818ms, receipts=32/481/1247ms
- timeouts: 0
- fetch backlog (cum_fetch - cum_processed): max ~2,136 blocks

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,216us, p95=2,787us, p99=3,945us, max=18,441us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167
- avg processing breakdown (us/block): header_hash=5, tx_hashes=645, transactions=549, block_size=120

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=341ms, p95=444ms, max=754ms
- flush size (bytes_total): p50=107.8 MiB, p95=204.6 MiB, max=340.3 MiB
- flush throughput: p50=321.7 MiB/s, p95=602.5 MiB/s

Compaction characteristics:
- compaction_end count: 100
- compaction duration_ms: p50=1052ms, p95=1559ms, max=5484ms
- range completion (`pending_total=0`) at ~1080.1s; 82 compactions finished by then
- final compaction window (`compact_all_dirty` duration): 24.07s

## Hetzner ingest-1M (v0.2, inflight60/chunk64, jemalloc, buffered=16384) - 2026-01-24T18:36:30Z

Artifacts:
- `benchmarks_hetzner/ingest_1M_v0_2_inflight60_chunk64_jemalloc_buffered_16384__20260124T183630Z__range-23283452-24283451__chunk64__chunkmax256__inflight60__timeout15000__profile-release__alloc-jemalloc.json`
- `benchmarks_hetzner/ingest_1M_v0_2_inflight60_chunk64_jemalloc_buffered_16384__20260124T183630Z__range-23283452-24283451__chunk64__chunkmax256__inflight60__timeout15000__profile-release__alloc-jemalloc.events.jsonl`

Environment / config:
- env: linux x86_64, cpu_count=12
- range: 23,283,452..24,283,451 (1,000,000 blocks)
- shard_size: 10,000
- fast-sync: chunk=64, chunk_max=256, inflight=60, timeout=15000ms, lookahead=200,000, buffered=16384
- db writer: batch_blocks=512
- build: release, allocator=jemalloc

Top-line results (from summary JSON):
- elapsed: 1,201.835s (20m 02s)
- throughput: 832.1 blocks/s avg, 663,766 logs/s avg
- totals: 1,000,000 blocks fetched/processed/written; 797,736,715 logs total
- fetch failures: 15,273 failed blocks across 6,116 failures; 0 process failures
- peers: 38 peers_used; 154 peer_failures_total
- storage on disk (compacted): 74.49 GB (`storage.total_bytes`)

Resource envelope (from `.events.jsonl`):
- rss max: 10.19 GiB; swap max: 0
- cpu_busy_pct: avg 27.6%, p95 35.9%, max 44.5%
- cpu_iowait_pct: avg 0.31%, p95 1.35% (max 16.68%; spiky during flush bursts)
- peers_active: max 24, p95 24

Fetch characteristics:
- 117,939 batches total; avg 273,352 bytes/block; download 12.16 MiB/s avg
- per-batch blocks: p50=9, p95=14, max=64
- per-batch duration_ms: p50=67ms, p95=591ms, p99=1303ms, max=6600ms
- request durations (p50/p95/p99): headers=25/141/257ms, bodies=34/329/728ms, receipts=54/502/1057ms
- timeouts: 0
- fetch backlog (cum_fetch - cum_processed): max ~16,480 blocks (RSS peaks alongside this)

Processing characteristics (per-block `process_end` events):
- duration_us: p50=1,224us, p95=2,789us, p99=3,947us, max=41,282us
- logs/block: p50=597, p95=1,810, p99=6,551, max=24,167
- avg processing breakdown (us/block): header_hash=5, tx_hashes=646, transactions=553, block_size=123

DB writer flush characteristics (`db_flush_end` events):
- 1,954 flushes of 512 blocks
- flush duration_ms: p50=340ms, p95=443ms, max=579ms
- flush size (bytes_total): p50=108.0 MiB, p95=202.7 MiB, max=341.9 MiB
- flush throughput: p50=321.9 MiB/s, p95=629.4 MiB/s

Compaction characteristics:
- compaction_end count: 101
- compaction duration_ms: p50=1045ms, p95=1456ms, max=6148ms
- range completion (`pending_total=0`) at ~1180.1s; 87/101 compactions finished by then
- final compaction window (`compact_all_dirty` duration): 21.77s

Comparison vs 2026-01-24T17:23:39Z (inflight60/chunk64, buffered=8192):
- wall time: 1106s -> 1202s (slower by ~8.7%)
- throughput: 904.0 -> 832.1 blocks/s (~-8.0%)
- fetch backlog max: ~2.1k -> ~16.5k blocks; rss max: 3.33 -> 10.19 GiB
- despite the deeper buffer, this run is slower due to worse fetch conditions (batch duration p50 35ms -> 67ms and fewer peers_used: 44 -> 38)
