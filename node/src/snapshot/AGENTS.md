# snapshot

Base snapshot import through the Rust binary. No Python or external downloader is
required. Read `docs/base-snapshot-download.md` for the supported format and CLI.

- `args.rs`: `SnapshotArgs`, resource limits, offline and dry-run switches.
- `manifest.rs`: SHA-pinned Base V2 manifest inspection, component selection with
  the genesis prefix dependency, output paths/checksums and aligned reader layout.
- `download.rs`: staging identity and exclusive lock, bounded HTTP workers,
  strong-ETag resume, retries, free-space/transfer limits, streamed tar/zstd
  extraction and verified chunk publication. `download/tests.rs` uses local HTTP
  servers and generated archives.
- `reader.rs`: NippyJar/Compact reader, sequential transaction boundaries, parent
  and transaction-number continuity, typed Base validation, pre-Canyon gate.
- `mod.rs`: dry-run output and import orchestration. Validates the full required
  prefix before creating a fresh destination, then streams bounded batches through
  `BaseChain::process_ingest` and storage WAL/compaction/sealing. Reopens and checks
  coverage before writing the success report.
- `tests.rs`: generated cross-archive fixtures, invalid-input/output preservation,
  CLI identity, gas-boundary and format-gate tests, growing-shard space budgets,
  and symlink/parent destination resolution.

Keep the Reth pin shared with the rest of the node. Do not infer later-fork or
arbitrary body-index compatibility from the tested pre-Canyon reader. Staging
resume is supported; import into existing output, durable import checkpoints,
automatic archive cleanup and P2P handoff belong to subsequent work. Do not make
the Python prototypes or local raw evidence runtime/CI dependencies.
