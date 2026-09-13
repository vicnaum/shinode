# snapshot

Base snapshot import through the Rust binary. No Python or external downloader is
required. Read `docs/base-snapshot-download.md` for the supported format and CLI.

- `package/`: pinned local native history export/restore, bounded tar/zstd,
  sealed coverage and overlap verification, recipient RPC/P2P acceptance tests.
  See `package/AGENTS.md` and `docs/base-history-packages.md`.

- `args.rs`: `SnapshotArgs`, resource limits, offline/dry-run and keep-scratch switches.
  `--checkpoint-hash` pins the independently verified end block; `--follow`
  enters the normal Base runtime after durable import and cleanup.
- `manifest.rs`: SHA-pinned Base V2 manifest inspection, component selection with
  the selected archive prefix dependency (legacy staging retains genesis), output paths/checksums and aligned reader layout.
- `download.rs`: staging identity and exclusive lock, bounded HTTP workers,
  strong-ETag resume, retries, free-space/transfer limits, streamed tar/zstd
  extraction and verified chunk publication. `download/tests.rs` uses local HTTP
  servers and generated archives.
- `reader.rs`: NippyJar/Compact reader, sequential transaction boundaries, parent
  and transaction-number continuity, typed Base fork-aware receipt/withdrawals validation.
- `mod.rs`: dry-run output and import orchestration. Validates the full required
  prefix, recovers WAL/compaction, checks all overlaps, then streams missing blocks
  through `BaseChain::process_ingest` and storage WAL/compaction/sealing. Keeps the
  destination writer lease across reopen and verifies the full output digest.
  Uses `ui::archive::Progress` for live phases, bytes and safe cancellation;
  the same TUI controller continues into P2P history/follow. Blocking reads/compaction may
  finish before cancellation is observed. Dry-run keeps its JSON-only output.
- `journal.rs`: versioned manifest/range/chain/reader-bound checkpoints, complete
  stored-bundle comparison, coverage reconciliation and durable completion digest.
- `scratch.rs`: exact owned input paths and fingerprints, conservative cleanup
  after durable completion; adopted and changed files are preserved.
- `tests.rs`: generated cross-archive fixtures, invalid-input/output preservation,
  CLI identity, gas-boundary and format-gate tests, growing-shard space budgets,
  and symlink/parent destination resolution.
- `resume_tests.rs`: interruption/restart, torn WAL/checkpoint, compaction phases,
  overlap/gap/conflict, writer lease, cleanup and keep-scratch regressions.
- `handoff_tests.rs`: checkpoint rejection, typed mock-peer continuation through
  the production ingest pipeline, cross-boundary RPC, restart/repair, batched gap
  fills across existing suffixes, and reorg fencing.
- `pipeline_tests.rs`: mixed historical/recent peer backfill, incomplete WAL
  recovery before scheduling (including deferred compaction), and cancellation
  of buffered gaps or replacement-chain blocks during worker drain, with queued
  reorg rollback versus strict normal completion.

Keep the Reth pin shared with the rest of the node. Real-source scale evidence
remains pre-Canyon; generated Compact fixtures cover later fork reconstruction. Conversion
resume still rescans the required prefix; reclamation follows full job completion,
not rolling archive consumption. P2P handoff requires an independently trusted hash
and a peer serving the successor; checksums do not establish finality. Do not make
the Python prototypes or local raw evidence runtime/CI dependencies.

- `bootstrap.rs` / `bootstrap/peers.rs`: `--auto-handoff` probes actual peer payload
  availability, verifies the chosen boundary against `--checkpoint-rpc`, persists
  the choice for stable retries, and implies normal history/follow. No fixed peer
  retention assumption. Dedicated staging, resource limits and cancellation apply.
