# Hetzner OOM crash reports (2026-01-23)

This folder archives the raw crash report bundle collected from a Hetzner AX41-NVME run where `stateless-history-node` was OOM-killed around ~67% sync progress.

- The raw logs and system snapshots are in `crash_reports/`.
- Large trace JSON files are **not** copied here; see `crash_reports/trace_files_list.txt` for their names/sizes.

## What we found (2026-01-23)

From the benchmark artifacts (events/logs/trace + resource samples):

- RSS ramps quickly early in the run (peaks around ~55-60 GiB) and then stays elevated; later swap grows.
- Memory growth correlates strongly with shard compactions (compaction phases ratchet RSS upward).
- Compactions were able to overlap (2 concurrent), which amplifies peak memory when two shards are compacted at once.
- Near the end of the run (~1050-1100s), CPU rises significantly, DB flush latency increases, and the pipeline enters an unhealthy backlog pattern; shortly after, the process is OOM-killed.
- The crash bundle shows very large WALs for two shards around the crash window (multi-GiB). These shards did not emit a matching `compaction_end` event in the surviving logs.

## Fixes applied (2026-01-23)

Implemented changes to bound compaction memory and reduce allocator fragmentation:

- Stream WAL during compaction via an on-disk index (scan headers/offsets first; read/decode individual records on-demand) instead of loading the entire WAL into RAM.
- Avoid cloning large payload buffers during segment append; write in bounded chunks.
- Serialize compactions to 1-at-a-time (queue) to prevent overlapping shard compactions from doubling peak memory.
- Add an optional jemalloc build feature (`--features jemalloc`) and document `MALLOC_ARENA_MAX=2` (Linux/glibc) as a knob to reduce RSS fragmentation on long runs.

Code changes:
- `a2831f0` Reduce compaction memory; serialize compactions
- `4f8cbd9` feat(node): add optional jemalloc allocator

## How to reproduce / compare (Hetzner)

Run the same benchmark range under different allocators/knobs:

```bash
# baseline (glibc)
cargo run --manifest-path node/Cargo.toml --release -- --benchmark ingest ...

# glibc with fewer arenas (often lower RSS fragmentation)
MALLOC_ARENA_MAX=2 cargo run --manifest-path node/Cargo.toml --release -- --benchmark ingest ...

# jemalloc (compile-time)
cargo run --manifest-path node/Cargo.toml --release --features jemalloc -- --benchmark ingest ...
```
