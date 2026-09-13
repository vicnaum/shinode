# package

Local Base native-history distribution; no execution snapshot or Python dependency.

- `args.rs`: `history-export` / `history-restore` arguments, independent end hash,
  publisher manifest pin, disk/extraction/batch limits.
- `format.rs`: version 1 manifest, schema 2 / Base `op_v1` identity, aligned
  contiguous coverage, exact native file whitelist, SHA-256 and source boundary.
- `archive.rs`: streamed zstd level 3 / tar packaging, regular-file-only bounded
  extraction, archive/file hashes, padding and decoder limits, disk reserve.
- `mod.rs`: exclusive source/export, private staging, native seal and bundle
  verification, overlap/combined-parent preflight, missing-block WAL merge,
  compaction-aware disk budget, reopen verification and trusted-boundary pinning.
  Export/restore run through the archive TUI controller, reporting verification,
  packaging/extraction, merge and publication phases. Shared cancellation checks
  stop between blocks and file buffers; active storage work finishes before exit.
- `tests.rs`: standalone restore, malformed manifests/archives, conflict/gap
  rejection, interrupted WAL recovery, CLI, adjacent/older packages, production
  typed-peer continuation and RPC/restart. The ignored native sample proof uses
  `SHINODE_HISTORY_SOURCE` and `SHINODE_HISTORY_OUTPUT`; no execution input access.

Only complete, verified sealed shards are distributable. Preserve the storage
writer lease through restore reopen. No direct archive extraction into a recipient
DB. Validate the package start against the saved trusted boundary before opening
storage, and repeat that read-only check under the destination writer lease.
Preflight opens storage with `open_locked_preserving_resume`, including unanchored
P2P destinations. Adopt the incoming lower bound only on the final reopen after
overlap/combined-parent validation and durable writes, retaining the same lease.
Unfinished initial restore is fenced by `history-package-init.json` until
`snapshot-boundary.json` is published. The publisher manifest pin authenticates
stored fields; native transaction metadata cannot reproduce the original full
transaction-root validation. Do not claim finality or public historical peer
availability from hashes or controlled-peer tests. Keep raw artifacts outside Git.
