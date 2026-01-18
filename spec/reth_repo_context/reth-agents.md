This file is a merged representation of a subset of the codebase, containing specifically included files, combined into a single document by Repomix.

# File Summary

## Purpose
This file contains a packed representation of a subset of the repository's contents that is considered the most important context.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.

## File Format
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Repository files (if enabled)
5. Multiple file entries, each consisting of:
  a. A header with the file path (## File: path/to/file)
  b. The full contents of the file in a code block

## Usage Guidelines
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.

## Notes
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Only files matching these patterns are included: **/AGENTS.md
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded
- Files are sorted by Git change count (files with more changes are at the bottom)

# Directory Structure
```
crates/
  chain-state/
    benches/
      AGENTS.md
    src/
      AGENTS.md
    AGENTS.md
  chainspec/
    src/
      AGENTS.md
    AGENTS.md
  cli/
    cli/
      src/
        AGENTS.md
      AGENTS.md
    commands/
      src/
        db/
          AGENTS.md
        init_state/
          AGENTS.md
        p2p/
          AGENTS.md
        stage/
          dump/
            AGENTS.md
          AGENTS.md
        test_vectors/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    runner/
      src/
        AGENTS.md
      AGENTS.md
    util/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  config/
    src/
      AGENTS.md
    AGENTS.md
  consensus/
    common/
      src/
        AGENTS.md
      AGENTS.md
    consensus/
      src/
        AGENTS.md
      AGENTS.md
    debug-client/
      src/
        providers/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  e2e-test-utils/
    src/
      testsuite/
        actions/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    tests/
      e2e-testsuite/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  engine/
    invalid-block-hooks/
      src/
        AGENTS.md
      AGENTS.md
    local/
      src/
        AGENTS.md
      AGENTS.md
    primitives/
      src/
        AGENTS.md
      AGENTS.md
    service/
      src/
        AGENTS.md
      AGENTS.md
    tree/
      benches/
        AGENTS.md
      docs/
        AGENTS.md
      src/
        tree/
          payload_processor/
            AGENTS.md
          AGENTS.md
        AGENTS.md
      tests/
        e2e-testsuite/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    util/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  era/
    src/
      common/
        AGENTS.md
      e2s/
        AGENTS.md
      era/
        types/
          AGENTS.md
        AGENTS.md
      era1/
        types/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    tests/
      it/
        era/
          AGENTS.md
        era1/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  era-downloader/
    src/
      AGENTS.md
    tests/
      it/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  era-utils/
    src/
      AGENTS.md
    tests/
      it/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  errors/
    src/
      AGENTS.md
    AGENTS.md
  ethereum/
    cli/
      src/
        AGENTS.md
      AGENTS.md
    consensus/
      src/
        AGENTS.md
      AGENTS.md
    engine-primitives/
      src/
        AGENTS.md
      AGENTS.md
    evm/
      src/
        AGENTS.md
      tests/
        AGENTS.md
      AGENTS.md
    hardforks/
      src/
        hardforks/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    node/
      src/
        AGENTS.md
      tests/
        e2e/
          AGENTS.md
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    payload/
      src/
        AGENTS.md
      AGENTS.md
    primitives/
      src/
        AGENTS.md
      AGENTS.md
    reth/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  etl/
    src/
      AGENTS.md
    AGENTS.md
  evm/
    evm/
      src/
        AGENTS.md
      AGENTS.md
    execution-errors/
      src/
        AGENTS.md
      AGENTS.md
    execution-types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  exex/
    exex/
      src/
        backfill/
          AGENTS.md
        wal/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    test-utils/
      src/
        AGENTS.md
      AGENTS.md
    types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  fs-util/
    src/
      AGENTS.md
    AGENTS.md
  metrics/
    src/
      common/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  net/
    banlist/
      src/
        AGENTS.md
      AGENTS.md
    discv4/
      src/
        AGENTS.md
      AGENTS.md
    discv5/
      src/
        AGENTS.md
      AGENTS.md
    dns/
      src/
        AGENTS.md
      AGENTS.md
    downloaders/
      src/
        bodies/
          AGENTS.md
        headers/
          AGENTS.md
        test_utils/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    ecies/
      src/
        AGENTS.md
      AGENTS.md
    eth-wire/
      src/
        errors/
          AGENTS.md
        AGENTS.md
      tests/
        AGENTS.md
      AGENTS.md
    eth-wire-types/
      src/
        AGENTS.md
      AGENTS.md
    nat/
      src/
        AGENTS.md
      AGENTS.md
    network/
      benches/
        AGENTS.md
      src/
        fetch/
          AGENTS.md
        session/
          AGENTS.md
        test_utils/
          AGENTS.md
        transactions/
          AGENTS.md
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    network-api/
      src/
        test_utils/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    network-types/
      src/
        peers/
          AGENTS.md
        session/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    p2p/
      src/
        bodies/
          AGENTS.md
        headers/
          AGENTS.md
        snap/
          AGENTS.md
        test_utils/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    peers/
      src/
        bootnodes/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  node/
    api/
      src/
        AGENTS.md
      AGENTS.md
    builder/
      src/
        builder/
          AGENTS.md
        components/
          AGENTS.md
        launch/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    core/
      src/
        args/
          AGENTS.md
        cli/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    ethstats/
      src/
        AGENTS.md
      AGENTS.md
    events/
      src/
        AGENTS.md
      AGENTS.md
    metrics/
      src/
        AGENTS.md
      AGENTS.md
    types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  optimism/
    bin/
      src/
        AGENTS.md
      AGENTS.md
    chainspec/
      src/
        superchain/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    cli/
      src/
        commands/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    consensus/
      src/
        validation/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    evm/
      src/
        AGENTS.md
      AGENTS.md
    flashblocks/
      src/
        ws/
          AGENTS.md
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    hardforks/
      src/
        AGENTS.md
      AGENTS.md
    node/
      src/
        AGENTS.md
      tests/
        e2e-testsuite/
          AGENTS.md
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    payload/
      src/
        AGENTS.md
      AGENTS.md
    primitives/
      src/
        transaction/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    reth/
      src/
        AGENTS.md
      AGENTS.md
    rpc/
      src/
        eth/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    storage/
      src/
        AGENTS.md
      AGENTS.md
    txpool/
      src/
        supervisor/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  payload/
    basic/
      src/
        AGENTS.md
      AGENTS.md
    builder/
      src/
        AGENTS.md
      AGENTS.md
    builder-primitives/
      src/
        AGENTS.md
      AGENTS.md
    primitives/
      src/
        AGENTS.md
      AGENTS.md
    util/
      src/
        AGENTS.md
      AGENTS.md
    validator/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  primitives/
    benches/
      AGENTS.md
    src/
      transaction/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  primitives-traits/
    src/
      block/
        AGENTS.md
      constants/
        AGENTS.md
      header/
        AGENTS.md
      transaction/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  prune/
    db/
      src/
        AGENTS.md
      AGENTS.md
    prune/
      src/
        segments/
          user/
            AGENTS.md
          AGENTS.md
        AGENTS.md
      AGENTS.md
    types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  ress/
    protocol/
      src/
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    provider/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  revm/
    src/
      AGENTS.md
    AGENTS.md
  rpc/
    ipc/
      src/
        client/
          AGENTS.md
        server/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc/
      src/
        eth/
          helpers/
            AGENTS.md
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-api/
      src/
        AGENTS.md
      AGENTS.md
    rpc-builder/
      src/
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-convert/
      src/
        AGENTS.md
      AGENTS.md
    rpc-e2e-tests/
      src/
        AGENTS.md
      tests/
        e2e-testsuite/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-engine-api/
      src/
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-eth-api/
      src/
        helpers/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-eth-types/
      src/
        builder/
          AGENTS.md
        cache/
          AGENTS.md
        error/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-layer/
      src/
        AGENTS.md
      AGENTS.md
    rpc-server-types/
      src/
        AGENTS.md
      AGENTS.md
    rpc-testing-util/
      src/
        AGENTS.md
      tests/
        it/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  stages/
    api/
      src/
        metrics/
          AGENTS.md
        pipeline/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    stages/
      benches/
        setup/
          AGENTS.md
        AGENTS.md
      src/
        stages/
          AGENTS.md
        test_utils/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  stateless/
    src/
      AGENTS.md
    AGENTS.md
  static-file/
    static-file/
      src/
        segments/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    types/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  storage/
    codecs/
      derive/
        src/
          compact/
            AGENTS.md
          AGENTS.md
        AGENTS.md
      src/
        alloy/
          transaction/
            AGENTS.md
          AGENTS.md
        AGENTS.md
      testdata/
        AGENTS.md
      AGENTS.md
    db/
      benches/
        AGENTS.md
      src/
        implementation/
          mdbx/
            AGENTS.md
          AGENTS.md
        static_file/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    db-api/
      src/
        models/
          AGENTS.md
        tables/
          codecs/
            fuzz/
              AGENTS.md
            AGENTS.md
          AGENTS.md
        AGENTS.md
      AGENTS.md
    db-common/
      src/
        db_tool/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    db-models/
      src/
        AGENTS.md
      AGENTS.md
    errors/
      src/
        AGENTS.md
      AGENTS.md
    libmdbx-rs/
      benches/
        AGENTS.md
      mdbx-sys/
        libmdbx/
          cmake/
            AGENTS.md
          man1/
            AGENTS.md
          AGENTS.md
        src/
          AGENTS.md
        AGENTS.md
      src/
        AGENTS.md
      tests/
        AGENTS.md
      AGENTS.md
    nippy-jar/
      src/
        compression/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    provider/
      src/
        changesets_utils/
          AGENTS.md
        providers/
          database/
            AGENTS.md
          rocksdb/
            AGENTS.md
          state/
            AGENTS.md
          static_file/
            AGENTS.md
          AGENTS.md
        test_utils/
          AGENTS.md
        traits/
          AGENTS.md
        writer/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    rpc-provider/
      src/
        AGENTS.md
      AGENTS.md
    storage-api/
      src/
        AGENTS.md
      AGENTS.md
    zstd-compressors/
      src/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  tasks/
    src/
      AGENTS.md
    AGENTS.md
  tokio-util/
    src/
      AGENTS.md
    AGENTS.md
  tracing/
    src/
      AGENTS.md
    AGENTS.md
  tracing-otlp/
    src/
      AGENTS.md
    AGENTS.md
  transaction-pool/
    benches/
      AGENTS.md
    src/
      blobstore/
        AGENTS.md
      pool/
        AGENTS.md
      test_utils/
        AGENTS.md
      validate/
        AGENTS.md
      AGENTS.md
    tests/
      it/
        AGENTS.md
      AGENTS.md
    AGENTS.md
  trie/
    common/
      benches/
        AGENTS.md
      src/
        hash_builder/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    db/
      src/
        AGENTS.md
      tests/
        AGENTS.md
      AGENTS.md
    parallel/
      benches/
        AGENTS.md
      src/
        AGENTS.md
      AGENTS.md
    sparse/
      benches/
        AGENTS.md
      src/
        AGENTS.md
      AGENTS.md
    sparse-parallel/
      src/
        AGENTS.md
      AGENTS.md
    trie/
      benches/
        AGENTS.md
      src/
        hashed_cursor/
          AGENTS.md
        proof/
          AGENTS.md
        proof_v2/
          AGENTS.md
        trie_cursor/
          AGENTS.md
        AGENTS.md
      AGENTS.md
    AGENTS.md
  AGENTS.md
```

# Files

## File: crates/chain-state/benches/AGENTS.md
```markdown
# benches

## Purpose
Benchmarks for `reth-chain-state`, focused on performance of canonical-chain helpers over in-memory overlays.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `canonical_hashes_range.rs` - Criterion benchmark for `MemoryOverlayStateProviderRef::canonical_hashes_range()` across different range sizes and scenarios.

## Key APIs (no snippets)
- **Bench entrypoint**: `canonical_hashes_range.rs`

## Relationships
- **Declared by**: `reth/crates/chain-state/Cargo.toml` (`[[bench]] canonical_hashes_range`, requires `test-utils`).
```

## File: crates/chain-state/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-chain-state` core types: in-memory canonical chain tracking, chain-info (head/safe/finalized) tracking, canonical-state notifications, deferred trie data helpers, and state-provider overlays that combine persisted state with unpersisted in-memory blocks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `chain_info.rs` - `ChainInfoTracker`: tracks canonical head + safe/finalized/persisted headers and forkchoice timing via `watch` channels.
- `deferred_trie.rs` - `DeferredTrieData` + `ComputedTrieData`: lazy/async trie-data handle with deadlock-avoiding synchronous fallback and anchored overlay reuse.
- `in_memory.rs` - `CanonicalInMemoryState` internals for tracking unpersisted canonical blocks, pending block state, and emitting canon-state notifications.
- `lib.rs` - module wiring and public re-exports.
- `memory_overlay.rs` - `MemoryOverlayStateProvider{,Ref}`: state provider overlaying in-memory executed blocks over a historical provider (incl. roots/proofs APIs).
- `noop.rs` - no-op/placeholder pieces used where a chain-state implementation is required but functionality is intentionally disabled.
- `notifications.rs` - `CanonStateNotification` + subscription/stream types (`CanonStateSubscriptions`, `ForkChoiceSubscriptions`, `WatchValueStream`, etc.).
- `test_utils.rs` - test helpers (compiled in `test-utils` / tests).

## Key APIs (no snippets)
- **Types**: `CanonicalInMemoryState`, `ChainInfoTracker`, `CanonStateNotification`, `CanonStateNotificationStream`, `DeferredTrieData`, `ComputedTrieData`, `AnchoredTrieInput`, `MemoryOverlayStateProvider`, `MemoryOverlayStateProviderRef`
- **Traits**: `CanonStateSubscriptions`, `ForkChoiceSubscriptions`
- **Methods**: `ChainInfoTracker::set_canonical_head()`, `ChainInfoTracker::set_safe()`, `ChainInfoTracker::set_finalized()`, `DeferredTrieData::wait_cloned()`, `MemoryOverlayStateProviderRef::canonical_hashes_range()`

## Relationships
- **Used by**: engine-tree and other subsystems that need fast access to "recent canonical blocks not yet persisted" and/or need to emit canonical-chain change notifications.
- **Depends on**: `reth-storage-api` (provider traits), `reth-trie` (hashed state/trie inputs), `tokio` (`watch`/`broadcast` for subscriptions), `reth-execution-types` (canonical chain segments/receipts).

## Notes
- `DeferredTrieData` is designed to avoid deadlocks by not blocking on async completion: it tries to lock and can compute synchronously from stored inputs if needed.
```

## File: crates/chain-state/AGENTS.md
```markdown
# chain-state

## Purpose
`reth-chain-state` crate: provides chain-state tracking primitives for reth, especially around the canonical chain's "recent, not-yet-persisted" blocks, forkchoice-related heads (head/safe/finalized), and notification streams for consumers like the engine and RPC.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for canonical-chain helpers over in-memory overlays.
- [x] `src/` - Canonical in-memory state tracking, chain-info tracking, notifications, and overlay state providers.

### Files
- `Cargo.toml` - crate manifest (features like `serde` and `test-utils`, bench registration).

## Key APIs (no snippets)
- **Types**: `CanonicalInMemoryState`, `ChainInfoTracker`, `CanonStateNotification`, `MemoryOverlayStateProvider`, `DeferredTrieData`
- **Streams/subscriptions**: canonical-state notifications (`CanonStateNotificationStream`) and forkchoice head subscriptions (`ForkChoiceSubscriptions`).

## Relationships
- **Used by**: engine components that need quick access to recent canonical data and to publish chain-change notifications.
```

## File: crates/chainspec/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-chainspec` core APIs for describing an Ethereum-like network: chain identity, hardfork schedule, genesis/header construction, and chain-spec access traits.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `api.rs` - `EthChainSpec` trait: chain-spec queries used across the codebase (fork activation, params lookups, etc.).
- `constants.rs` - chain-specific constants and convenience re-exports.
- `info.rs` - `ChainInfo` type (network identity/metadata used by the node).
- `lib.rs` - crate entrypoint; re-exports `ChainSpec`, `ChainSpecBuilder`, fork types, and built-in chain presets (e.g. `MAINNET`, `SEPOLIA`, `HOLESKY`).
- `spec.rs` - `ChainSpec` / `ChainSpecBuilder` implementations and helpers (genesis header creation, base-fee params, deposit contract metadata, etc.).

## Key APIs (no snippets)
- **Traits**: `EthChainSpec`, `ChainSpecProvider`
- **Types**: `ChainSpec`, `ChainSpecBuilder`, `ChainInfo`, `BaseFeeParams`, `BaseFeeParamsKind`, `ForkBaseFeeParams`, `DepositContract`
- **Constants/presets**: `MAINNET`, `SEPOLIA`, `HOLESKY`, `HOODI`, `DEV`
- **Helpers**: `make_genesis_header()`, `create_chain_config()`, `mainnet_chain_config()`

## Relationships
- **Used by**: most subsystems that need fork activation checks and chain parameters (consensus, engine, EVM config, network handshake/status, etc.).
- **Depends on**: `reth-ethereum-forks` (hardfork types/schedule), `alloy-genesis` + `res/genesis/*.json` (genesis inputs), `reth-network-peers` (peer/network integration helpers).
```

## File: crates/chainspec/AGENTS.md
```markdown
# chainspec

## Purpose
`reth-chainspec` crate: defines the spec of an Ethereum network (chain identity, genesis, and hardfork schedule) and exposes shared chain-spec query traits used across reth.

## Contents (one hop)
### Subdirectories
- [x] `src/` - ChainSpec types/builders, chain presets, and `EthChainSpec` trait.
- [x] `res/` - (skip: static assets) Genesis JSON inputs used to build built-in chain specs.

### Files
- `Cargo.toml` - crate manifest (features like `std`, `arbitrary`, `test-utils`).

## Key APIs (no snippets)
- **Traits**: `EthChainSpec`, `ChainSpecProvider`
- **Types**: `ChainSpec`, `ChainSpecBuilder`, `ChainInfo`
- **Presets**: `MAINNET`, `SEPOLIA`, `HOLESKY`, `HOODI`, `DEV`

## Relationships
- **Used by**: consensus/validation, engine, EVM execution configuration, network status/handshakes, and RPC where fork/chain params are required.
```

## File: crates/cli/cli/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-cli`: a small abstraction layer for reth-based node CLIs, providing shared traits for parsing args, selecting chainspecs, and running commands on a standard runner.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `chainspec.rs` - `ChainSpecParser` trait + clap `TypedValueParser` adapter; helper `parse_genesis()` (inline JSON or load-from-disk).
- `lib.rs` - `RethCli` trait (name/version/client version + helpers for arg parsing and command execution); exports `chainspec` module.

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`
- **Helpers**: `parse_genesis()`

## Relationships
- **Depends on**: `clap` (argument parsing), `reth-cli-runner` (tokio runner), `reth-db::ClientVersion`, `alloy-genesis` (genesis representation).
```

## File: crates/cli/cli/AGENTS.md
```markdown
# cli

## Purpose
`reth-cli` crate: shared CLI abstractions for reth-based nodes (argument parsing + command execution wiring), intended to be embedded by concrete binaries like `reth` / `op-reth`.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `RethCli` and chainspec parsing helpers.

### Files
- `Cargo.toml` - crate manifest.

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`
```

## File: crates/cli/commands/src/db/AGENTS.md
```markdown
# db

## Purpose
Implements `reth db`: CLI database/static-file inspection and maintenance commands (stats/list/get/clear/diff/checksum) plus heavier repair/diagnostics tooling (trie verification/repair) and a small TUI for browsing table entries.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Entry point for the `reth db` command group: defines the top-level clap command, wires `EnvironmentArgs`/provider access, and dispatches to subcommands.
- **Key items**: `Command<C>`, `Subcommands`, `list::Command`, `stats::Command`, `get::Command`, `diff::Command`, `clear::Command`
- **Interactions**: constructs/uses `reth_db_common::DbTool` and forwards `tool`/`data_dir`/`task_executor` into subcommand executors.
- **Knobs / invariants**: some subcommands warn/assume the node is not running (long read transactions; safe use of RW operations).

#### `stats.rs`
- **Role**: Implements `reth db stats`: prints sizes/entry counts for MDBX tables and static-file segments, and can optionally checksum all tables.
- **Key items**: `stats::Command`, `db_stats_table()`, `static_files_stats_table()`, `checksum_report()`, `ChecksumViewer`
- **Interactions**: reads MDBX stats via `db_stat`, iterates static files via `iter_static_files`, and reuses `checksum::ChecksumViewer` for checksumming.
- **Knobs / invariants**: `--checksum` is slow (full DB traversal); `--skip-consistency-checks`, `--detailed-sizes`, `--detailed-segments`.

#### `list.rs`
- **Role**: Implements `reth db list`: paginated table browsing with filters, supporting JSON output or an interactive terminal UI.
- **Key items**: `list::Command`, `list_filter()`, `ListTableViewer`, `DbListTUI`
- **Interactions**: uses `reth_db_common::DbTool::list()` and `tui::DbListTUI` for interactive paging; disables long-read transaction safety for long sessions.
- **Knobs / invariants**: `--skip`, `--len`, `--reverse`, `--search`, `--min-*-size`, `--count`, `--json`, `--raw`.

#### `tui.rs`
- **Role**: Terminal UI used by `db list` to navigate table rows and inspect key/value payloads.
- **Key items**: `DbListTUI<F, T>`, `ViewMode`, `Entries<T>`, `event_loop()`, `handle_event()`, `CMDS`
- **Interactions**: calls a provided page fetcher \(closure\) to load pages; integrates `crossterm` input with `ratatui` rendering.
- **Knobs / invariants**: assumes raw-mode terminal operation; keybindings for paging/selection and "go to page".

#### `get.rs`
- **Role**: Implements `reth db get`: retrieves a single entry (or a range) from an MDBX table, or retrieves a record from a static-file segment (with optional raw bytes output).
- **Key items**: `get::Command`, `Subcommand`, `GetValueViewer`, `table_key()`, `maybe_json_value_parser`
- **Interactions**: uses `Tables::view()` to decode MDBX values; special-cases `StaticFileSegment::AccountChangeSets` via provider helpers.
- **Knobs / invariants**: JSON parsing for keys/subkeys; `--raw` prints bytes for MDBX/static-file reads.

#### `checksum.rs`
- **Role**: Implements `reth db checksum`: computes a stable hash over table key/value bytes for auditing or cross-node comparisons (optionally ranged/limited).
- **Key items**: `checksum::Command`, `ChecksumViewer`, `ChecksumViewer::new()`
- **Interactions**: uses a raw cursor walk and feeds bytes into a seeded hasher; reuses key parsing utilities from `get.rs`.
- **Knobs / invariants**: expects the node to be stopped (long-lived reads); supports `--start-key`, `--end-key`, `--limit`.

#### `diff.rs`
- **Role**: Implements `reth db diff`: compares a primary DB with a secondary DB (read-only) and writes discrepancy reports per table.
- **Key items**: `diff::Command`, `find_diffs()`, `find_diffs_advanced()`, `tables_to_generic!`
- **Interactions**: opens a second DB via `open_db_read_only`, walks tables with cursors, and emits text reports into an output directory.
- **Knobs / invariants**: expects the node to be stopped; supports diffing all tables or a selected table.

#### `clear.rs`
- **Role**: Implements `reth db clear`: deletes all entries for a selected MDBX table, or deletes static-file jars for a given segment.
- **Key items**: `clear::Command`, `Subcommands`, `ClearViewer`
- **Interactions**: clears MDBX tables via a RW transaction and commit; deletes static-file jars by iterating `iter_static_files`.
- **Knobs / invariants**: destructive; static-file deletion operates at the jar level by segment.

#### `static_file_header.rs`
- **Role**: Implements `reth db static-file-header`: prints the user header for a static file segment (by block/segment or by path) for debugging static-file layout/ranges.
- **Key items**: `static_file_header::Command`, `Source`, `StaticFileProviderFactory::get_segment_provider*`
- **Interactions**: performs an optional consistency check before selecting a segment provider and printing header metadata.
- **Knobs / invariants**: `Source::Block` vs `Source::Path` selection.

#### `settings.rs`
- **Role**: Implements `reth db settings`: reads or updates persisted `StorageSettings` (what lives in static files vs MDBX vs rocksdb, etc.).
- **Key items**: `settings::Command`, `Subcommands`, `SetCommand`, `StorageSettings`
- **Interactions**: reads settings via `MetadataProvider` and writes via `MetadataWriter`/`write_storage_settings()`.
- **Knobs / invariants**: distinguishes RO vs RW access needs (`Command::access_rights()`); setters are per-field toggles.

#### `repair_trie.rs`
- **Role**: Implements `reth db repair-trie`: verifies trie consistency against hashed state and (optionally) repairs DB trie tables when inconsistencies are found.
- **Key items**: `repair_trie::Command`, `Verifier`, `Output`, `verify_checkpoints()`, `verify_and_repair()`
- **Interactions**: checks stage checkpoints to ensure Merkle sync is complete; reads/writes `AccountsTrie`/`StoragesTrie` via cursors; can expose Prometheus metrics.
- **Knobs / invariants**: `--dry-run` avoids mutations; requires a consistent stage state (no in-progress MerkleExecute).

#### `account_storage.rs`
- **Role**: Implements `reth db account-storage`: estimates storage footprint for an account by scanning `PlainStorageState` dupsort entries.
- **Key items**: `account_storage::Command`, `PlainStorageState`, `keccak256`, `LOG_INTERVAL`
- **Interactions**: iterates dupsort values and uses `reth_codecs::Compact` to estimate encoded size; prints plain vs hashed size estimates.
- **Knobs / invariants**: logs progress periodically; operates as a full scan over the address's storage entries.

## Key APIs (no snippets)
- **Top-level**: `db::Command<C>`, `db::Subcommands`
- **Subcommands**: `stats::Command`, `list::Command`, `get::Command`, `diff::Command`, `checksum::Command`, `clear::Command`, `repair_trie::Command`, `static_file_header::Command`, `settings::Command`, `account_storage::Command`
- **TUI**: `DbListTUI<F, T>`, `ViewMode`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` (registers `db` under the main CLI command set).
- **Depends on**: `reth-db*` + `reth-db-common` (tooling/cursors), `reth-provider` (provider factories/static files/metadata), `reth-trie*` (verification/repair), and `ratatui`/`crossterm` (interactive TUI).

## End-to-end flow (high level)
- Parse `reth db ...` CLI args into `db::Command<C>` + `Subcommands`.
- Initialize environment/provider access according to command needs (RO vs RW).
- Build `DbTool` and dispatch to the selected subcommand.
- Subcommand runs a DB/static-file read/write routine (cursor walk, table stats, static-file provider lookup, etc.).
- Print results (JSON/text/tables) or run `DbListTUI` for interactive navigation.
```

## File: crates/cli/commands/src/init_state/AGENTS.md
```markdown
# init_state

## Purpose
Implements `reth init-state`: initializes a data directory from a JSONL state dump, with an optional `--without-evm` mode that creates a dummy EVM chain up to a provided header before importing state.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command definition and execution logic for importing the JSONL state dump into the database/static files.
  - **Key items**: `InitStateCommand<C>`, `init_from_state_dump()`, `EnvironmentArgs<C>`
- `without_evm.rs` - helpers for `--without-evm`: read an RLP header from file and append dummy blocks/static-file ranges before inserting the first "real" block.
  - **Key items**: `read_header_from_file()`, `setup_without_evm()`, `append_dummy_chain()`, `append_first_block()`

## Key APIs (no snippets)
- **CLI type**: `InitStateCommand<C>`
- **Helpers**: `setup_without_evm()`, `read_header_from_file()`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` via the `init_state` module.
- **Depends on**: `reth-db-common::init` (state dump importer), provider/static-file writer APIs (`reth-provider`, `reth-static-file-types`) to materialize state and required metadata.
```

## File: crates/cli/commands/src/p2p/AGENTS.md
```markdown
# p2p

## Purpose
Implements `reth p2p`: a CLI debugging tool for networking/P2P operations (fetching headers/bodies via the fetch client, simple RLPx handshakes, and running a discovery-only bootnode).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command definitions and dispatch for `reth p2p` subcommands (header/body download + RLPx + bootnode).
  - **Key items**: `Command<C>`, `Subcommands<C>`, `DownloadArgs<C>`, `DownloadArgs::launch_network()`, `DownloadArgs::backoff()`
- `bootnode.rs` - standalone bootnode: runs discv4 (and optional discv5) for discovery-only operation.
  - **Key items**: `bootnode::Command`, `Discv4`, `Discv5`, `NatResolver`
- `rlpx.rs` - RLPx utilities (currently a ping/handshake flow to print the peer's `HelloMessage`).
  - **Key items**: `rlpx::Command`, `ECIESStream`, `UnauthedP2PStream`, `HelloMessage`

## Key APIs (no snippets)
- **CLI types**: `Command<C>`, `Subcommands<C>`, `DownloadArgs<C>`, `bootnode::Command`, `rlpx::Command`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` as part of the CLI command suite.
- **Depends on**: `reth-network` + `reth-network-p2p` (network startup + fetch client), `reth-eth-wire` (RLPx handshake messages), `reth-discv4`/`reth-discv5` (discovery services).
```

## File: crates/cli/commands/src/stage/dump/AGENTS.md
```markdown
# dump

## Purpose
Implements `reth stage dump`: utilities to export ("dump") selected pipeline stage data for a given block range into a new database for debugging and repro workflows.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `execution.rs` - dump logic for the Execution stage.
  - **Key items**: `dump_execution_stage()`
- `hashing_account.rs` - dump logic for the AccountHashing stage.
  - **Key items**: `dump_hashing_account_stage()`
- `hashing_storage.rs` - dump logic for the StorageHashing stage.
  - **Key items**: `dump_hashing_storage_stage()`
- `merkle.rs` - dump logic for the Merkle stage.
  - **Key items**: `dump_merkle_stage()`
- `mod.rs` - clap command definitions + dispatch across dumpable stages; shared setup for the output DB and initial state.
  - **Key items**: `Command<C>`, `Stages`, `StageCommand`, `setup()`

## Key APIs (no snippets)
- **CLI types**: `Command<C>`, `Stages`, `StageCommand`
- **Stage dump entrypoints**: `dump_execution_stage()`, `dump_hashing_storage_stage()`, `dump_hashing_account_stage()`, `dump_merkle_stage()`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/stage/mod.rs` (`reth stage dump` subcommand).
- **Depends on**: DB tooling (`reth-db*`, `reth-db-common`) and stage implementations (`reth-stages*`) to read input state and write a new output DB.
```

## File: crates/cli/commands/src/stage/AGENTS.md
```markdown
# stage

## Purpose
Implements `reth stage` CLI tooling for pipeline/stage debugging: run a single stage over a block range, unwind stages/pipeline state, dump stage data into a new DB, or drop stage-related data from the DB/static files.

## Contents (one hop)
### Subdirectories
- [x] `dump/` - Stage "dump" utilities that export stage data for a range into a new database.

### Files
- `mod.rs` - `reth stage` command dispatcher and chain-spec plumbing.
  - **Key items**: `Command<C>`, `Subcommands<C>`
- `run.rs` - run a single stage over a range (optionally with metrics and network-backed downloaders for headers/bodies stages).
  - **Key items**: `Command<C>`, `StageEnum`, `ExecutionStageThresholds`, `HeaderStage`, `BodyStage`
- `drop.rs` - "drop stage" tooling: clears stage tables/checkpoints and coordinates pruning of static-file segments when applicable.
  - **Key items**: `Command<C>`, `StageEnum`, `reset_stage_checkpoint()`, `reset_prune_checkpoint()`, `StaticFileSegment`
- `unwind.rs` - pipeline unwind command (supports an "offline" unwind mode that avoids headers/bodies/senders).
  - **Key items**: `Command<C>`, `Subcommands`, `Pipeline`, `DefaultStages`, `OfflineStages`

## Key APIs (no snippets)
- **CLI types**: `stage::Command<C>`, `stage::Subcommands<C>`
- **Stage commands**: `run::Command<C>`, `drop::Command<C>`, `unwind::Command<C>`, `dump::Command<C>`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` via the `stage` module (as part of the `reth` CLI command suite).
- **Depends on**: `reth-stages` (stage implementations/pipeline), `reth-db*` + `reth-db-common` (DB operations/checkpoints), `reth-static-file` (static file producer) and network/downloaders for online stages.
```

## File: crates/cli/commands/src/test_vectors/AGENTS.md
```markdown
# test_vectors

## Purpose
Implements `reth test-vectors`: CLI tooling to generate and validate small "micro" test vectors for DB tables and `Compact`-encoded types (primarily for codec/DB compatibility checks).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command and subcommand dispatch (`Tables` vs `Compact`).
  - **Key items**: `Command`, `Subcommands`
- `tables.rs` - generates JSON vectors for selected DB tables (including dupsort tables), using proptest-based generation and stable ordering.
  - **Key items**: `generate_vectors()`, `generate_table_vector()`, `generate_dupsort_vector()`, `save_to_file()`, `VECTORS_FOLDER`
- `compact.rs` - generates/reads vectors for many `Compact` types (codecs), using proptest/arbitrary and a macro-driven registry.
  - **Key items**: `compact_types!`, `GENERATE_VECTORS`, `READ_VECTORS`, `VECTORS_FOLDER`, `VECTOR_SIZE`

## Key APIs (no snippets)
- **CLI types**: `Command`, `Subcommands`
- **Generators**: `tables::generate_vectors()`, `compact::generate_vectors()`, `compact::read_vectors()`

## Relationships
- **Feature-gated by**: `reth-cli-commands` `arbitrary` feature (enables the `test_vectors` module).
- **Writes to**: `testdata/micro/*` folders (generated vector artifacts; not part of normal node operation).
```

## File: crates/cli/commands/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-cli-commands`: a shared collection of concrete `reth ...` CLI subcommands (node launch, import/export, DB tools, stage/pipeline tooling, network/P2P debug, etc.) plus common environment/bootstrap helpers used by reth-based binaries.

## Contents (one hop)
### Subdirectories
- [x] `db/` - `reth db` command suite: DB/static-file inspection, diff/checksum/clear, trie verification/repair, and a TUI for browsing tables.
- [x] `init_state/` - `reth init-state`: initialize a datadir from a state dump (including `--without-evm` helper path).
- [x] `p2p/` - `reth p2p`: P2P/network debugging commands (fetching, RLPx, bootnode/discovery).
- [x] `stage/` - `reth stage`: stage/pipeline debugging and maintenance (run/unwind/drop/dump).
- [x] `test_vectors/` - (feature-gated) `reth test-vectors`: generate micro vectors for DB tables and codec/Compact types.

### Files
- `common.rs` - shared CLI environment/bootstrap used by multiple commands: config+datadir resolution, DB/static-file opening, consistency checks/healing, and common CLI node type bounds.
  - **Key items**: `EnvironmentArgs<C>`, `Environment<N>`, `AccessRights`, `CliNodeTypes`
- `config_cmd.rs` - `reth config`: prints either a specified config file or the default config as TOML.
  - **Key items**: `config_cmd::Command`, `Command::execute()`
- `download.rs` - `reth download`: snapshot download + streaming extraction (with configurable default snapshot sources).
  - **Key items**: `DownloadCommand<C>`, `DownloadDefaults`, `stream_and_extract()`, `DownloadProgress`
- `dump_genesis.rs` - `reth dump-genesis`: prints the selected chain's genesis JSON.
  - **Key items**: `DumpGenesisCommand<C>`
- `export_era.rs` - `reth export-era`: exports blocks from the local DB into ERA1 files.
  - **Key items**: `ExportEraCommand<C>`, `ExportArgs`, `ERA1_EXPORT_FOLDER_NAME`
- `import.rs` - `reth import`: imports RLP-encoded blocks from one or more files, running the pipeline after file-backed header/body ingestion.
  - **Key items**: `ImportCommand<C>`, `ImportConfig`, `import_blocks_from_file()`, `build_import_pipeline`
- `import_core.rs` - core import logic without clap dependencies (shared by CLI and tests).
  - **Key items**: `ImportConfig`, `ImportResult`, `import_blocks_from_file()`, `build_import_pipeline_impl()`
- `import_era.rs` - `reth import-era`: imports ERA1 blocks from a local directory or remote host (HTTP), starting from the current static-file tip.
  - **Key items**: `ImportEraCommand<C>`, `ImportArgs`, `EraStream`, `EraStreamConfig`
- `init_cmd.rs` - `reth init`: initializes the database with the genesis block.
  - **Key items**: `InitCommand<C>`
- `launcher.rs` - node launch abstraction: trait-based launcher interface and a closure adapter for backward-compatible wiring.
  - **Key items**: `Launcher<C, Ext>`, `FnLauncher<F>`
- `lib.rs` - crate module wiring and re-exports.
  - **Key items**: module list; `pub use node::NodeCommand`
- `node.rs` - `reth node`: main node command (argument surface for network/rpc/txpool/pruning/etc.) and the generic launch entrypoint.
  - **Key items**: `NodeCommand<C, Ext>`, `NodeCommand::execute()`, `NoArgs`
- `prune.rs` - `reth prune`: runs configured pruning without enforced limits (after copying eligible DB data into static files).
  - **Key items**: `PruneCommand<C>`, `PrunerBuilder`, `StaticFileProducer`
- `re_execute.rs` - `reth re-execute`: parallel re-execution of historical blocks to validate sync correctness.
  - **Key items**: `re_execute::Command<C>`, `CancellationToken`, `JoinSet`, `Executor`

## Key APIs (no snippets)
- **Environment/bootstrap**: `EnvironmentArgs<C>`, `Environment<N>`, `AccessRights`
- **Node launch**: `NodeCommand<C, Ext>`, `Launcher<C, Ext>`, `FnLauncher<F>`
- **Data movement**: `ImportCommand<C>`, `ImportEraCommand<C>`, `ExportEraCommand<C>`, `DownloadCommand<C>`
- **Maintenance/debug**: `db::Command<C>`, `stage::Command<C>`, `PruneCommand<C>`, `re_execute::Command<C>`

## Relationships
- **Used by**: reth-based CLI binaries (e.g. `reth-ethereum-cli` and downstream CLIs) that want a standard set of subcommands.
- **Depends on**: `reth-cli` (chainspec parsing), `reth-cli-runner`/`reth-cli-util` (runner and helpers), and many core reth subsystems (DB/provider/static files/stages/network/evm).

## Notes
- `test_vectors` is behind the crate `arbitrary` feature; the directory exists in-tree but is not always compiled.
```

## File: crates/cli/commands/AGENTS.md
```markdown
# commands

## Purpose
`reth-cli-commands` crate: provides the concrete `reth ...` command implementations (node launch, DB tooling, stage/pipeline debugging, import/export, etc.) that are shared by reth-based binaries.

## Contents (one hop)
### Subdirectories
- [x] `src/` - command implementations and shared environment/bootstrap helpers.

### Files
- `Cargo.toml` - crate manifest for `reth-cli-commands` (feature flags and broad subsystem dependencies).
  - **Key items**: `features.arbitrary`, `features.edge`

## Key APIs (no snippets)
- **Command entrypoints**: `NodeCommand`, `db::Command`, `stage::Command`, `ImportCommand`, `ImportEraCommand`, `ExportEraCommand`, `DownloadCommand`

## Relationships
- **Used by**: higher-level CLI crates/binaries that want the standard reth command set.
- **Depends on**: most core reth subsystems (DB/provider/static files/stages/network/evm) to implement operational commands.
```

## File: crates/cli/runner/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-cli-runner`: a tokio-based runner for executing reth CLI commands with task management and graceful shutdown on `SIGINT`/`SIGTERM`.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `CliRunner` + `CliContext` and helpers for running async / blocking commands until completion or ctrl-c, with `TaskManager`-backed shutdown.

## Key APIs (no snippets)
- **Types**: `CliRunner`, `CliContext`, `CliRunnerConfig`
- **Functions**: `tokio_runtime()`
- **Methods**: `CliRunner::try_default_runtime()`, `CliRunner::run_command_until_exit()`, `CliRunner::run_blocking_command_until_exit()`

## Relationships
- **Depends on**: `reth-tasks` (task manager/executor), `tokio` (runtime + signals).
```

## File: crates/cli/runner/AGENTS.md
```markdown
# runner

## Purpose
`reth-cli-runner` crate: provides a tokio-based runner for reth CLI commands, integrating `reth-tasks` for task lifecycle management and graceful shutdown on exit signals.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `CliRunner` implementation and helpers.

### Files
- `Cargo.toml` - crate manifest.

## Key APIs (no snippets)
- **Types**: `CliRunner`, `CliContext`, `CliRunnerConfig`
- **Functions**: `tokio_runtime()`

## Relationships
- **Used by**: higher-level CLI crates to execute commands consistently (async or blocking) and to ensure spawned tasks are shut down cleanly.
```

## File: crates/cli/util/src/AGENTS.md
```markdown
# src

## Purpose
Implementation of `reth-cli-util`: small reusable helpers for CLI wiring (allocators, cancellation handling, secret-key loading, and common value parsers).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `allocator.rs` - allocator selection/wrappers for CLI builds (jemalloc/snmalloc/tracy).
- `cancellation.rs` - cancellation primitives/helpers for long-running CLI operations.
- `lib.rs` - module wiring and public re-exports.
- `load_secret_key.rs` - load/parse a secp256k1 secret key from file/hex for CLI usage.
- `parsers.rs` - parsing helpers used as clap value parsers (durations, socket addresses, ether values, block hash-or-number, JSON file reading).
- `sigsegv_handler.rs` - unix-only signal handler hook (or no-op fallback) for better backtraces on crashes/stack overflows.

## Key APIs (no snippets)
- **Parsing**: `parse_socket_address()`, `parse_duration_from_secs_or_ms()`, `parse_ether_value()`, `hash_or_num_value_parser()`
- **Key handling**: `get_secret_key()`, `parse_secret_key_from_hex()`

## Relationships
- **Used by**: CLI crates (`reth-cli`, `reth-cli-commands`) for consistent argument parsing and runtime setup.
```

## File: crates/cli/util/AGENTS.md
```markdown
# util

## Purpose
`reth-cli-util` crate: shared utilities used across reth's CLI crates (argument/value parsing, allocator configuration, cancellation helpers, and key loading).

## Contents (one hop)
### Subdirectories
- [x] `src/` - implementation of parsing + runtime/allocator helpers.

### Files
- `Cargo.toml` - crate manifest (allocator-related feature flags).

## Key APIs (no snippets)
- **Re-exports**: `parse_socket_address()`, `parse_duration_from_secs_or_ms()`, `parse_ether_value()`, `hash_or_num_value_parser()`, `get_secret_key()`
```

## File: crates/cli/AGENTS.md
```markdown
# cli

## Purpose
CLI subsystem crates for reth: shared abstractions for building CLIs (`reth-cli`), the concrete command suite (`reth-cli-commands`), a tokio-based execution runner (`reth-cli-runner`), and small shared parsing/runtime utilities (`reth-cli-util`).

## Contents (one hop)
### Subdirectories
- [x] `cli/` - `reth-cli`: shared CLI traits and chainspec parsing helpers used by reth-based CLIs.
- [x] `commands/` - `reth-cli-commands`: implementations of `reth ...` subcommands (node/db/stage/import/export/etc.).
- [x] `runner/` - `reth-cli-runner`: tokio runtime + graceful shutdown wrapper for CLI command execution.
- [x] `util/` - `reth-cli-util`: reusable CLI helpers (value parsers, allocator/cancellation helpers, secret key loading).

### Files
- (none)

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`, `Launcher`
- **Runners**: `CliRunner`, `CliContext`
- **Command surfaces**: `NodeCommand`, `db::Command`, `stage::Command`

## Relationships
- **Used by**: top-level reth binaries/CLIs (e.g. `reth`, `op-reth`) to share consistent argument parsing and command execution patterns.
- **Connects**: CLI argument parsing (`clap`) <-> node configuration (`reth-node-core`/`reth-node-builder`) <-> runtime/task management (`reth-tasks`).
```

## File: crates/config/src/AGENTS.md
```markdown
# src

## Purpose
Defines `reth-config` core configuration types (pipeline stages, pruning, peers/sessions, static files) and, when the `serde` feature is enabled, TOML load/save helpers for node/CLI configuration files.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires the `config` module and re-exports the primary configuration types.
- **Key items**: `config` module, `Config`, `PruneConfig`, `BodiesConfig`
- **Interactions**: re-export surface consumed by `reth-node-core`/CLI crates so they don't import deep module paths.
- **Knobs / invariants**: none (pure wiring/re-exports).

#### `config.rs`
- **Role**: The actual configuration schema for reth: a top-level `Config` object with nested per-subsystem settings (stages, pruning, peers/sessions, static files) and many stage-specific config structs. Under `serde`, includes TOML read/write helpers and duration parsing.
- **Key items**: `Config`, `StageConfig`, `EraConfig`, `HeadersConfig`, `BodiesConfig`, `ExecutionConfig`, `EtlConfig`, `PruneConfig`, `StaticFilesConfig`, `DEFAULT_BLOCK_INTERVAL`
- **Interactions**: converts/feeds stage-related thresholds into pipeline code (e.g. `ExecutionConfig` -> `ExecutionStageThresholds`); composes external config types like `PeersConfig`, `SessionsConfig`, `PruneModes`, `StaticFileMap`.
- **Knobs / invariants**: `Config::from_path()` creates a default file if missing (TOML, `serde` feature); `Config::save()` enforces `.toml` extension.

## Key APIs (no snippets)
- **Top-level**: `Config`
- **Stage grouping**: `StageConfig`
- **Stage configs**: `EraConfig`, `HeadersConfig`, `BodiesConfig`, `SenderRecoveryConfig`, `ExecutionConfig`, `PruneStageConfig`, `HashingConfig`, `MerkleConfig`, `TransactionLookupConfig`, `IndexHistoryConfig`, `EtlConfig`
- **Pruning/static files**: `PruneConfig`, `StaticFilesConfig`, `BlocksPerFileConfig`

## Relationships
- **Used by**: node configuration plumbing (node/CLI loads config and passes stage/prune/static-file knobs into pipeline and storage).
- **Depends on**: `reth-*-types` crates for value-object config types (`reth-network-types`, `reth-stages-types`, `reth-prune-types`, `reth-static-file-types`).

## End-to-end flow (high level)
- Construct `Config` (default or from TOML when `serde` is enabled).
- Read nested knobs for stages/pruning/peers/sessions/static files.
- Convert specific config parts into execution thresholds or stage configs (e.g. `ExecutionConfig` -> `ExecutionStageThresholds`).
- Downstream subsystems consume those settings to configure pipeline behavior, pruning strategy, and storage layout.
```

## File: crates/config/AGENTS.md
```markdown
# config

## Purpose
`reth-config` crate: standalone configuration schema for reth (pipeline stage knobs, pruning modes, peer/session settings, static-file layout) with optional TOML serialization/deserialization via the `serde` feature.

## Contents (one hop)
### Subdirectories
- [x] `src/` - configuration structs and (feature-gated) TOML load/save helpers.

### Files
- `Cargo.toml` - crate manifest (optional `serde`/`toml` support).
  - **Key items**: `features.serde`

## Key APIs (no snippets)
- **Types**: `Config`, `StageConfig`, `PruneConfig`, `StaticFilesConfig`

## Relationships
- **Used by**: node/CLI configuration loading (feeds settings into storage, pipeline stages, and pruning/static-file behavior).
```

## File: crates/consensus/common/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-consensus-common`: reusable, chain-spec-aware block/header/body validation helpers (pre-execution / "stateless" checks) that are shared across consensus implementations.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint exposing the `validation` module.
- **Key items**: `validation` module
- **Interactions**: consumed by consensus/validation callers that want shared helper functions.
- **Knobs / invariants**: `no_std` when `std` feature is disabled.

#### `validation.rs`
- **Role**: Collection of validation routines that check header/body invariants and fork-activated fields without requiring state execution (e.g. gas limits, roots, withdrawals, blob gas, block size limits).
- **Key items**: `validate_header_gas()`, `validate_header_base_fee()`, `validate_body_against_header()`, `validate_block_pre_execution()`, `post_merge_hardfork_fields()`, `validate_4844_header_standalone()`, `MAX_RLP_BLOCK_SIZE`
- **Interactions**: relies on `reth_chainspec::EthereumHardforks` to gate validations by fork activation; returns `reth_consensus::ConsensusError` to unify error reporting.
- **Knobs / invariants**: enforces fork-specific requirements (Shanghai withdrawals, Cancun blob gas fields, Osaka block size / tx gas-limit constraints) based on the provided chainspec and header timestamps/numbers.

## Key APIs (no snippets)
- **Constants**: `MAX_RLP_BLOCK_SIZE`
- **Header validation**: `validate_header_gas()`, `validate_header_base_fee()`, `validate_header_extra_data()`, `validate_against_parent_hash_number()`
- **Body/block validation**: `validate_body_against_header()`, `validate_block_pre_execution()`, `post_merge_hardfork_fields()`
- **Fork-specific**: `validate_shanghai_withdrawals()`, `validate_cancun_gas()`, `validate_4844_header_standalone()`

## Relationships
- **Used by**: consensus engines and other components that need consistent "pre-exec" validation behavior.
- **Depends on**: `reth-consensus` for shared error types, `reth-chainspec` for fork activation checks, and primitives traits for block/header/body abstractions.

## End-to-end flow (high level)
- Take a sealed header/block + chainspec.
- Apply basic invariants (gas limit bounds, required header fields).
- Apply fork-activated rules (withdrawals, blob gas, size limits) based on block number/timestamp.
- Compare body-derived roots/hashes to header roots/hashes.
- Return a `ConsensusError` describing the first violated invariant.
```

## File: crates/consensus/common/AGENTS.md
```markdown
# common

## Purpose
`reth-consensus-common` crate: shared, chain-spec-aware validation helpers for Ethereum blocks/headers/bodies (pre-execution / stateless checks) used by consensus implementations.

## Contents (one hop)
### Subdirectories
- [x] `src/` - validation helper functions and constants.

### Files
- `Cargo.toml` - crate manifest (std/no-std support and dependency wiring).
  - **Key items**: `features.std`

## Key APIs (no snippets)
- **Validation helpers**: `validate_block_pre_execution()`, `validate_body_against_header()`, `validate_header_gas()`, `post_merge_hardfork_fields()`

## Relationships
- **Used by**: consensus engines and validation call sites that want common Ethereum consensus checks.
```

## File: crates/consensus/consensus/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-consensus`: core consensus/validation traits and shared error types for validating headers and blocks (pre- and post-execution), plus a no-op and test consensus implementation.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines the primary consensus traits (`HeaderValidator`, `Consensus`, `FullConsensus`) and the shared error model (`ConsensusError`, related helpers) used across reth's validation pipeline.
- **Key items**: `HeaderValidator`, `Consensus`, `FullConsensus`, `ConsensusError`, `HeaderConsensusError`
- **Interactions**: implemented by concrete consensus engines; consumed by sync/pipeline code that needs uniform validation hooks.
- **Knobs / invariants**: provides default `HeaderValidator::validate_header_range()` behavior (standalone validation + parent-chain validation in ascending order).

#### `noop.rs`
- **Role**: A "do nothing" consensus engine for testing and special cases where validation is intentionally bypassed.
- **Key items**: `NoopConsensus`, `NoopConsensus::arc()`
- **Interactions**: implements `HeaderValidator`, `Consensus`, and `FullConsensus` by always returning `Ok(())`.
- **Knobs / invariants**: explicitly not suitable for production use (no security/consensus guarantees).

#### `test_utils.rs`
- **Role**: Test helper consensus implementation with toggles to force validation failures in different phases.
- **Key items**: `TestConsensus`, `set_fail_validation()`, `set_fail_body_against_header()`
- **Interactions**: used in tests to exercise networking/sync behavior when blocks/headers are rejected.
- **Knobs / invariants**: separate flags for "header/overall" validation vs "body-against-header" failures.

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`, `HeaderConsensusError`
- **Implementations**: `NoopConsensus`, `TestConsensus`

## Relationships
- **Used by**: pipeline stages, sync/networking components, and execution validation flows that need to validate blocks/headers consistently.
- **Depends on**: primitives and execution result types (`reth-primitives-traits`, `reth-execution-types`) for block/header representations and post-execution outcome validation.

## End-to-end flow (high level)
- A component chooses/constructs a concrete consensus engine implementing `HeaderValidator` (+ optionally `Consensus`/`FullConsensus`).
- Sync/pipeline validates standalone headers, then headers against parents, and bodies against headers.
- After execution, `FullConsensus::validate_block_post_execution()` can validate receipts/state roots/etc. using the execution result.
- Errors are normalized into `ConsensusError` (and `HeaderConsensusError` for range validation context).
```

## File: crates/consensus/consensus/AGENTS.md
```markdown
# consensus

## Purpose
`reth-consensus` crate: defines the consensus/validation trait surface and shared error types for reth, including optional post-execution validation (`FullConsensus`) and helper implementations for testing (`NoopConsensus`, `TestConsensus`).

## Contents (one hop)
### Subdirectories
- [x] `src/` - consensus traits, errors, and built-in no-op/test implementations.

### Files
- `Cargo.toml` - crate manifest (std/no-std support and `test-utils` feature).
  - **Key items**: `features.std`, `features.test-utils`

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`
- **Implementations**: `NoopConsensus`

## Relationships
- **Used by**: block processing components that need an abstract validation interface and unified consensus error reporting.
```

## File: crates/consensus/debug-client/src/providers/AGENTS.md
```markdown
# providers

## Purpose
Implements concrete `BlockProvider` backends for the debug consensus client: fetching blocks either via an RPC endpoint (subscription/polling) or via the Etherscan HTTP API.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Provider module wiring and re-exports.
- **Key items**: `EtherscanBlockProvider`, `RpcBlockProvider`
- **Interactions**: re-exported by the crate root for easy consumption.
- **Knobs / invariants**: none.

#### `rpc.rs`
- **Role**: RPC-backed block provider that subscribes to new full blocks (websocket `eth_subscribe` when possible, otherwise polling) and supports historical `get_block` queries.
- **Key items**: `RpcBlockProvider<N, PrimitiveBlock>`, `RpcBlockProvider::new()`, `subscribe_blocks()`, `get_block()`, `full_block_stream()`
- **Interactions**: uses `alloy-provider`/`ProviderBuilder` with websocket config; converts `N::BlockResponse` into a primitives block via a caller-supplied converter.
- **Knobs / invariants**: configures large WS frame/message sizes for "big blocks"; re-establishes subscriptions when streams terminate.

#### `etherscan.rs`
- **Role**: Etherscan-backed block provider that polls the Etherscan "proxy" API for latest blocks and supports fetching blocks by number/tag.
- **Key items**: `EtherscanBlockProvider<RpcBlock, PrimitiveBlock>`, `load_block()`, `subscribe_blocks()`, `with_interval()`
- **Interactions**: issues HTTP GET requests with module/action parameters, parses `alloy_json_rpc::Response`, and converts the decoded RPC block into a primitives block via a caller-supplied converter.
- **Knobs / invariants**: polling interval (default ~3s); handles `chainid` parameter inclusion depending on the base URL format.

## Key APIs (no snippets)
- **RPC provider**: `RpcBlockProvider`
- **Etherscan provider**: `EtherscanBlockProvider`

## Relationships
- **Used by**: `DebugConsensusClient` as a source of new blocks and historical block lookups.
- **Depends on**: `alloy-provider` (RPC transport/subscriptions) and `reqwest` + `alloy-json-rpc` (Etherscan HTTP polling/parsing).
```

## File: crates/consensus/debug-client/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-consensus-debug-client`: a small "debug consensus" worker that drives an execution client by sending `newPayload` and `forkchoiceUpdated` messages using blocks fetched from an external source (RPC or Etherscan), without running a real consensus node.

## Contents (one hop)
### Subdirectories
- [x] `providers/` - Block provider backends (RPC subscription/polling, Etherscan HTTP polling).

### Files
- `lib.rs` - crate module wiring and public re-exports.
  - **Key items**: `DebugConsensusClient`, `BlockProvider`, `EtherscanBlockProvider`, `RpcBlockProvider`
- `client.rs` - core debug-consensus logic: block provider trait and the worker that converts blocks into execution payloads and emits engine API messages.
  - **Key items**: `BlockProvider`, `DebugConsensusClient<P, T>`, `DebugConsensusClient::run()`, `get_or_fetch_previous_block()`

## Key APIs (no snippets)
- **Block source abstraction**: `BlockProvider`
- **Worker**: `DebugConsensusClient`

## Relationships
- **Used by**: tooling/tests that want to validate/experiment with the engine API/execution client behavior without a beacon/consensus client.
- **Depends on**: `reth-node-api` engine API handle/types and `alloy-rpc-types-engine` forkchoice types; provider backends depend on `alloy-provider`/RPC and/or Etherscan HTTP APIs.
```

## File: crates/consensus/debug-client/AGENTS.md
```markdown
# debug-client

## Purpose
`reth-consensus-debug-client` crate: debug-only "consensus driver" that sources recent blocks from an external provider (RPC or Etherscan) and feeds them into the engine API (`newPayload` + `forkchoiceUpdated`) to exercise an execution client without a real consensus node.

## Contents (one hop)
### Subdirectories
- [x] `src/` - debug consensus worker and block-provider backends.

### Files
- `Cargo.toml` - crate manifest (RPC/Etherscan client dependencies and engine API types).
  - **Key items**: `alloy-provider (ws)`, `reqwest (rustls-tls)`

## Key APIs (no snippets)
- **Worker**: `DebugConsensusClient`
- **Sources**: `BlockProvider`, `RpcBlockProvider`, `EtherscanBlockProvider`

## Relationships
- **Connects**: external block sources <-> engine API handle (`reth-node-api`) for driving payload + forkchoice messages.
```

## File: crates/consensus/AGENTS.md
```markdown
# consensus

## Purpose
Consensus/validation subsystem crates: core consensus trait surface and errors, shared Ethereum validation helpers, and debug tooling for driving the engine API from external block sources.

## Contents (one hop)
### Subdirectories
- [x] `common/` - `reth-consensus-common`: shared pre-execution validation helpers (header/body/fork checks).
- [x] `consensus/` - `reth-consensus`: consensus traits (`HeaderValidator`/`Consensus`/`FullConsensus`) and shared `ConsensusError` model (plus noop/test implementations).
- [x] `debug-client/` - `reth-consensus-debug-client`: debug worker that feeds `newPayload`/`forkchoiceUpdated` to an execution client using blocks from RPC/Etherscan.

### Files
- (none)

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`
- **Helpers**: `reth-consensus-common::validation::*`
- **Debug tooling**: `DebugConsensusClient`, `BlockProvider`

## Relationships
- **Used by**: sync/pipeline and engine/execution flows for validating headers/blocks (and, in debug scenarios, for simulating consensus inputs).
```

## File: crates/e2e-test-utils/src/testsuite/actions/AGENTS.md
```markdown
# actions

## Purpose
Action library for the e2e testing framework: composable async steps that operate on a `testsuite::Environment` to drive nodes via RPC/Engine API, produce blocks, create forks/reorgs, and assert expected behavior.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Defines the `Action` trait and core action containers (`ActionBox`, `Sequence`), re-exports all concrete actions, and provides common helpers for validating forkchoice-update results.
- **Key items**: `Action<I>`, `ActionBox<I>`, `Sequence<I>`, `MakeCanonical`, `CaptureBlock`, `validate_fcu_response()`, `expect_fcu_valid()`
- **Interactions**: actions operate on `testsuite::Environment` and often call `EngineApiClient` for FCU/newPayload flows.
- **Knobs / invariants**: requires `EngineTypes` (and sometimes `PayloadTypes`) bounds for actions that build payload attributes or convert payload envelopes.

#### `produce_blocks.rs`
- **Role**: Block production and propagation actions: generate payload attributes, request payload building, broadcast payloads/FCUs, and assert acceptance/canonicalization.
- **Key items**: `AssertMineBlock`, `ProduceBlocks`, `ProduceBlocksLocally`, `ProduceInvalidBlocks`, `GeneratePayloadAttributes`, `GenerateNextPayload`, `BroadcastLatestForkchoice`, `BroadcastNextNewPayload`
- **Interactions**: uses both `EthApiClient` (to read latest blocks) and `EngineApiClient` (to send FCU/getPayload/newPayload); updates per-node `Environment` state (block info, payload IDs, tags).
- **Knobs / invariants**: some actions fall back between Engine API versions (v2 <-> v3) depending on fork support; relies on environment-managed timestamp increments and tag registry.

#### `engine_api.rs`
- **Role**: Engine API-specific actions for explicitly sending `newPayload` to nodes (including ordering/buffering scenarios) and asserting expected payload status outcomes.
- **Key items**: `SendNewPayload`, `SendNewPayloads`, `ExpectedPayloadStatus`
- **Interactions**: fetches blocks from a "source" node via `EthApiClient`, converts to `ExecutionPayloadV3`, and calls `EngineApiClient::new_payload_v3`.
- **Knobs / invariants**: expected-status matching supports "valid", "invalid", and "syncing/accepted" (buffered) scenarios.

#### `custom_fcu.rs`
- **Role**: Custom forkchoice-update actions to craft FCUs with explicit head/safe/finalized references (by hash/tag/latest) and validate response status.
- **Key items**: `BlockReference`, `SendForkchoiceUpdate`, `FinalizeBlock`, `resolve_block_reference()`
- **Interactions**: resolves tags via `Environment`'s registry, sends FCUs via `EngineApiClient::fork_choice_updated_v3`.
- **Knobs / invariants**: optional expected status for response validation; optional target node (defaults to active node).

#### `fork.rs`
- **Role**: Fork creation and validation actions: choose a fork base (block number or tag), update environment forkchoice state, produce blocks on the fork, and verify chain ancestry expectations.
- **Key items**: `ForkBase`, `CreateFork`, `SetForkBase`, `SetForkBaseFromBlockInfo`, `ValidateFork`
- **Interactions**: uses `EthApiClient` to locate fork base blocks and updates `Environment`'s per-node fork tracking (`current_fork_base`, forkchoice state).
- **Knobs / invariants**: fork base is stored for later validation; actions assume the environment's active node is the target for fork operations unless overridden upstream.

#### `reorg.rs`
- **Role**: Reorg actions: set a reorg target (typically via a previously captured/tagged block) and broadcast forkchoice to make a different head canonical.
- **Key items**: `ReorgTarget`, `ReorgTo`, `SetReorgTarget`
- **Interactions**: composes smaller actions (e.g. `BroadcastLatestForkchoice`) via `Sequence`.
- **Knobs / invariants**: "direct hash" reorgs are explicitly rejected; requires tagging/capturing the target block first.

#### `node_ops.rs`
- **Role**: Multi-node coordination actions: select active node, compare tips, tag blocks from specific nodes, wait for sync, and assert per-node chain state.
- **Key items**: `SelectActiveNode`, `CompareNodeChainTips`, `CaptureBlockOnNode`, `ValidateBlockTag`, `WaitForSync`
- **Interactions**: uses `EthApiClient` queries across nodes and updates the environment's active-node index and tag registry.
- **Knobs / invariants**: retry/timeout driven polling for "wait for sync" style actions.

## Key APIs (no snippets)
- **Action surface**: `Action<I>`, `ActionBox<I>`, `Sequence<I>`
- **Core orchestration**: `MakeCanonical`, `CaptureBlock`
- **Engine API actions**: `SendNewPayload`, `SendForkchoiceUpdate`, `FinalizeBlock`
- **Chain topology**: `CreateFork`, `ReorgTo`, `WaitForSync`

## Relationships
- **Used by**: `reth/crates/e2e-test-utils/src/testsuite/*` and workspace e2e test binaries (`tests/e2e-testsuite/*`).
- **Depends on**: `reth-rpc-api` clients (`EthApiClient`, `EngineApiClient`) plus engine/eth RPC types (`alloy_rpc_types_*`).

## End-to-end flow (high level)
- Build an e2e `Environment` with one or more nodes and RPC/Engine API clients.
- Execute one action (or a `Sequence`) at a time against the environment.
- Actions drive nodes via Engine API (FCU/newPayload/getPayload) and/or regular RPC reads, updating environment state (tips, tags, payload IDs).
- Assertions are embedded in actions (status checks, ancestry checks, expected equality/difference of tips).
```

## File: crates/e2e-test-utils/src/testsuite/AGENTS.md
```markdown
# testsuite

## Purpose
E2E test framework for reth: defines the multi-node `Environment` model, RPC/Engine API client wrappers, and setup/build utilities; test logic is expressed as composable `Action`s (produce blocks, forks/reorgs, sync assertions, engine API scenarios).

## Contents (one hop)
### Subdirectories
- [x] `actions/` - Action library for driving nodes and asserting behavior (block production, FCU/newPayload, forks/reorgs, sync ops).
- [x] `assets/` - (skip: static assets) Test fixtures (e.g. `genesis.json`) used by setup/import helpers.

### Files
- `mod.rs` - framework core types: `Environment`, `NodeClient`, `NodeState`, block registry/tagging, and shared state tracking for Engine API testing.
  - **Key items**: `Environment<I>`, `NodeClient<Payload>`, `NodeState<I>`, `BlockInfo`
- `setup.rs` - setup helper that creates node networks (optionally importing an RLP chain), waits for readiness, and initializes environment state.
  - **Key items**: `Setup<I>`, `Setup::apply()`, `Setup::apply_with_import()`, `NetworkSetup`
- `README.md` - documentation and usage guide for the e2e framework and action catalog.
  - **Key items**: `binary(e2e_testsuite)` nextest conventions, action categories

## Key APIs (no snippets)
- **Framework state**: `Environment<I>`, `NodeClient<I>`, `NodeState<I>`, `BlockInfo`
- **Setup**: `Setup<I>`
- **Actions**: `actions::Action<I>`, `actions::Sequence<I>`

## Relationships
- **Used by**: crate-level e2e test binaries under `tests/e2e-testsuite/` across the workspace.
- **Connects**: node instances <-> JSON-RPC + Engine API clients <-> high-level `Action` sequences that mutate/inspect environment state.
```

## File: crates/e2e-test-utils/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-e2e-test-utils`: utilities for spinning up one or more in-process test nodes, wiring RPC/Engine API clients, producing payloads/blocks, and running higher-level e2e "testsuite" action sequences.

## Contents (one hop)
### Subdirectories
- [x] `testsuite/` - e2e framework (environment model, setup logic, and action catalog).

### Files
- `lib.rs` - crate entrypoint: re-exports major helpers and defines setup functions/type aliases used across tests.
  - **Key items**: `setup()`, `setup_engine()`, `setup_engine_with_connection()`, `E2ETestSetupBuilder`, `NodeBuilderHelper`, `NodeHelperType`
- `node.rs` - per-node helper wrapper for driving a running `FullNode`: payload building/submission, forkchoice updates, canonical-state assertions, and network/RPC contexts.
  - **Key items**: `NodeTestContext`, `advance()`, `advance_block()`, `build_and_submit_payload()`, `update_forkchoice()`
- `payload.rs` - payload builder helper that subscribes to payload events, generates attributes, and waits for built payloads.
  - **Key items**: `PayloadTestContext<T>`, `new_payload()`, `expect_attr_event()`, `wait_for_built_payload()`
- `network.rs` - network helper for peer operations and session establishment assertions.
  - **Key items**: `NetworkTestContext<Network>`, `add_peer()`, `next_session_established()`, `record()`
- `rpc.rs` - RPC helper for injecting txs and reading raw transactions via debug API.
  - **Key items**: `RpcTestContext`, `inject_tx()`, `envelope_by_hash()`
- `setup_builder.rs` - builder for launching configurable test node networks (node config/tree config modifiers; optional interconnection).
  - **Key items**: `E2ETestSetupBuilder<N, F>`, `with_tree_config_modifier()`, `with_node_config_modifier()`, `build()`
- `setup_import.rs` - helpers to import an RLP chain into per-node temporary datadirs before launching nodes (Ethereum-only today).
  - **Key items**: `ChainImportResult`, `setup_engine_with_chain_import()`, `load_forkchoice_state()`
- `test_rlp_utils.rs` - utilities to generate simple test blocks and write them to an RLP file; helpers for generating FCU JSON payloads.
  - **Key items**: `generate_test_blocks()`, `write_blocks_to_rlp()`, `create_fcu_json()`
- `transaction.rs` - transaction builder/signing helpers for tests (transfers, deployments, blob txs, EIP-7702 set-code, etc.).
  - **Key items**: `TransactionTestContext`, `transfer_tx_bytes()`, `deploy_tx_bytes()`, `tx_with_blobs_bytes()`
- `wallet.rs` - deterministic wallet generator from a test mnemonic (derivation-path based).
  - **Key items**: `Wallet`, `wallet_gen()`, `with_chain_id()`

## Key APIs (no snippets)
- **Setup**: `E2ETestSetupBuilder`, `testsuite::Setup`, `setup_engine_with_connection()`
- **Node wrapper**: `NodeTestContext`
- **Testsuite**: `testsuite::Environment`, `testsuite::actions::Action`

## Relationships
- **Used by**: workspace crates that define `tests/e2e-testsuite/*` binaries and want consistent node setup + action-based test flows.
- **Depends on**: node builder/test utils, payload builder test utils, RPC builder/client types, and engine/local components to run in-process nodes.
```

## File: crates/e2e-test-utils/tests/e2e-testsuite/AGENTS.md
```markdown
# e2e-testsuite

## Purpose
Crate-local e2e test binary for `reth-e2e-test-utils`: example/validation tests that exercise the framework (setup, import-based setup, and action execution).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - example `#[tokio::test]` cases using `testsuite::Setup`/`TestBuilder` and action sequences (produce blocks, make canonical, forks/reorgs, import-from-RLP).
  - **Key items**: `test_apply_with_import`, `TestBuilder::with_action(...)`, `Setup::apply_with_import()`

## Key APIs (no snippets)
- **Framework usage**: `testsuite::Setup`, `testsuite::Environment`, `testsuite::actions::*`

## Relationships
- **Registered by**: `reth/crates/e2e-test-utils/Cargo.toml` as `[[test]] name = "e2e_testsuite"`.
```

## File: crates/e2e-test-utils/tests/AGENTS.md
```markdown
# tests

## Purpose
Test binary entrypoints for the e2e framework crate.

## Contents (one hop)
### Subdirectories
- [x] `e2e-testsuite/` - `e2e_testsuite` test binary using the framework (examples and regression tests).

### Files
- (none)

## Relationships
- **Wired by**: `reth/crates/e2e-test-utils/Cargo.toml` (`[[test]] e2e_testsuite`).
```

## File: crates/e2e-test-utils/AGENTS.md
```markdown
# e2e-test-utils

## Purpose
`reth-e2e-test-utils` crate: shared end-to-end testing utilities for reth, including multi-node in-process setup helpers, RPC/Engine API clients, an action-based testsuite framework, and an example `e2e_testsuite` test binary.

## Contents (one hop)
### Subdirectories
- [x] `src/` - framework and helpers for launching nodes, driving payloads/blocks, and running action sequences.
- [x] `tests/` - `e2e_testsuite` test binary entrypoint using the framework.

### Files
- `Cargo.toml` - crate manifest and test-binary registration.
  - **Key items**: `[[test]] e2e_testsuite`

## Key APIs (no snippets)
- **Framework**: `testsuite::Environment`, `testsuite::Setup`, `testsuite::actions::*`
- **Builder**: `E2ETestSetupBuilder`

## Relationships
- **Used by**: other crates' `tests/e2e-testsuite/*` binaries across the workspace (shared e2e patterns + helpers).
```

## File: crates/engine/invalid-block-hooks/src/AGENTS.md
```markdown
# src

## Purpose
Provides concrete `InvalidBlockHook` implementations for debugging invalid blocks, including witness generation and optional comparison against a "healthy" node.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring; exports `InvalidBlockWitnessHook`.
- `witness.rs` - `InvalidBlockWitnessHook`: re-executes invalid blocks, generates `ExecutionWitness`, writes JSON artifacts/diffs, and can fetch/compare a witness via `DebugApiClient`.

## Key APIs (no snippets)
- **Types**: `InvalidBlockWitnessHook<P, E>`
- **Implements**: `reth-engine-primitives::InvalidBlockHook`

## Relationships
- **Used by**: engine payload validation flows that want to capture diagnostics on invalid blocks (plugged in via `InvalidBlockHook`).
- **Depends on**: `reth-evm`/`reth-revm` (re-execution), `reth-trie` (hashed state + trie updates), `reth-rpc-api` debug client (optional healthy-node comparison).

## Notes
- This module is intentionally I/O heavy (writes artifacts to disk) and should be configured/used accordingly in production vs debugging contexts.
```

## File: crates/engine/invalid-block-hooks/AGENTS.md
```markdown
# invalid-block-hooks

## Purpose
`reth-invalid-block-hooks` crate root: debugging hooks for invalid-block handling (e.g., witness generation) that can be plugged into engine validation.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `InvalidBlockWitnessHook` implementation and crate wiring.

### Files
- `Cargo.toml` - crate manifest for `reth-invalid-block-hooks`.

## Key APIs (no snippets)
- **Exports**: `InvalidBlockWitnessHook`
```

## File: crates/engine/local/src/AGENTS.md
```markdown
# src

## Purpose
Implements the "local engine" components used to drive a dev chain without an external consensus client: a local miner loop and payload-attribute builders for producing Engine API traffic.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring; exports `LocalMiner`, `MiningMode`, and `LocalPayloadAttributesBuilder`.
- `miner.rs` - `LocalMiner` and `MiningMode`: drive payload building (instant or interval) and send FCU/newPayload to the engine.
- `payload.rs` - `LocalPayloadAttributesBuilder`: builds payload attributes (timestamp, withdrawals, etc.) for local payload jobs (Ethereum; optional OP support).

## Key APIs (no snippets)
- **Types**: `LocalMiner`, `MiningMode`, `LocalPayloadAttributesBuilder`
- **Methods**: `LocalMiner::new()`, `LocalMiner::run()`

## Relationships
- **Uses**: `reth-engine-primitives::ConsensusEngineHandle` to send `fork_choice_updated` / `new_payload` messages to the engine, and `reth-payload-builder::PayloadBuilderHandle` to resolve payload jobs.
- **Intended for**: local dev/test setups where the node itself generates Engine API inputs.
```

## File: crates/engine/local/AGENTS.md
```markdown
# local

## Purpose
`reth-engine-local` crate root: local mining/dev-chain utilities that generate Engine API messages (FCU/newPayload) without an external CL.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `LocalMiner`/`MiningMode` loop and payload-attribute builders.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-local` (optional `op` feature for OP payload attributes).

## Key APIs (no snippets)
- **Exports**: `LocalMiner`, `MiningMode`, `LocalPayloadAttributesBuilder`
```

## File: crates/engine/primitives/src/AGENTS.md
```markdown
# src

## Purpose
Defines the shared Engine API surface types for reth's engine subsystem: request/response message wrappers, forkchoice tracking, engine events, config knobs, and extension hooks used by the engine-tree and engine service.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `config.rs` - `TreeConfig` and related constants for engine-tree tuning (buffers, caching, workers, proof options).
- `error.rs` - Engine API-facing error types (`BeaconOnNewPayloadError`, `BeaconForkChoiceUpdateError`).
- `event.rs` - `ConsensusEngineEvent`: events emitted by the consensus/engine layer (forkchoice, block added/received, invalid block).
- `forkchoice.rs` - `ForkchoiceStateTracker` + `ForkchoiceStatus` and helpers for tracking latest/valid/syncing forkchoice states.
- `invalid_block_hook.rs` - `InvalidBlockHook` trait and helpers (`NoopInvalidBlockHook`, `InvalidBlockHooks`).
- `lib.rs` - crate module wiring and core traits (`EngineTypes`, `EngineApiValidator`, `PayloadValidator`) + re-exports.
- `message.rs` - `BeaconEngineMessage`, `ConsensusEngineHandle`, `OnForkChoiceUpdated` future used to handle/await FCU outcomes.

## Key APIs (no snippets)
- **Traits**: `EngineTypes`, `EngineApiValidator`, `PayloadValidator`, `InvalidBlockHook`
- **Types**: `BeaconEngineMessage<T>`, `ConsensusEngineHandle<T>`, `ConsensusEngineEvent<N>`, `ForkchoiceStateTracker`, `ForkchoiceStatus`, `TreeConfig`
- **Helpers**: `InvalidBlockHooks`, `NoopInvalidBlockHook`, `ForkchoiceStateHash`

## Relationships
- **Used by**: `reth-engine-tree` (engine handler/event plumbing + config + forkchoice tracking), `reth-engine-service` (service stream emits `ConsensusEngineEvent`), and Engine API RPC wiring (via `ConsensusEngineHandle` / `BeaconEngineMessage` stream).
- **Depends on**: `reth-payload-primitives` (payload + versioned Engine API types), `reth-primitives-traits` (block/header abstractions), `alloy-rpc-types-engine` (Engine API data types).

## Notes
- `TreeConfig` lives here (engine primitives) even though it configures engine-tree; it's treated as part of the public engine surface.
```

## File: crates/engine/primitives/AGENTS.md
```markdown
# primitives

## Purpose
`reth-engine-primitives` crate root: shared types and traits that define reth's Engine API message/event surface, forkchoice tracking, engine-tree configuration, and extension hooks.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Engine API messages (`BeaconEngineMessage`), handles/futures, events, forkchoice tracking, `TreeConfig`, and invalid-block hook traits.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-primitives` (features and dependencies).

## Key APIs (no snippets)
- **Exports**: `BeaconEngineMessage`, `ConsensusEngineHandle`, `ConsensusEngineEvent`, `ForkchoiceStateTracker`, `ForkchoiceStatus`, `TreeConfig`, `EngineTypes`

## Relationships
- **Used by**: engine crates under `reth/crates/engine/` (especially `tree/` and `service/`).
```

## File: crates/engine/service/src/AGENTS.md
```markdown
# src

## Purpose
Implements the `reth-engine-service` runtime wrapper that wires `reth-engine-tree` into a pollable service (stream), connecting consensus messages, on-demand downloads, pipeline backfill, and persistence.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - crate entrypoint; re-exports the `service` module.
- `service.rs` - `EngineService` definition and constructor that wires `EngineHandler` + `ChainOrchestrator` + `PipelineSync` + persistence.

## Key APIs (no snippets)
- **Types**: `EngineService<N, Client>`, `EngineMessageStream<T>`
- **Re-exports**: `ChainOrchestrator`, `ChainEvent`, `EngineApiEvent` (from `reth-engine-tree`)

## Relationships
- **Depends on**: `reth-engine-tree` (orchestrator/handler/tree/persistence), `reth-network-p2p` (block client), `reth-provider` (provider factory), `reth-payload-builder` (payload jobs), `reth-consensus` (validation), `reth-engine-primitives` (events/messages).
- **Used by**: node wiring (engine task/service that drives the chain forward from Engine API messages).
```

## File: crates/engine/service/AGENTS.md
```markdown
# service

## Purpose
`reth-engine-service` crate root: provides a top-level "engine service" abstraction that composes the engine-tree components into a stream/service suitable for integration into the node runtime.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EngineService` implementation and crate wiring.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-service`.

## Key APIs (no snippets)
- **Exports**: `EngineService`, `EngineMessageStream`

## Relationships
- **Builds on**: `reth-engine-tree` (core engine logic) and `reth-engine-primitives` (events/messages/config surface).
```

## File: crates/engine/tree/benches/AGENTS.md
```markdown
# benches

## Purpose
Benchmarks for `reth-engine-tree` performance-critical components (e.g., channel throughput and state root task behavior).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `channel_perf.rs` - benchmark focused on channel/message throughput used in engine-tree workflows.
- `state_root_task.rs` - benchmark focused on the state root computation task pipeline.

## Key APIs (no snippets)
- **Bench entrypoints**: `channel_perf.rs`, `state_root_task.rs`

## Relationships
- **Declared by**: `reth/crates/engine/tree/Cargo.toml` (`[[bench]] channel_perf`, `[[bench]] state_root_task`).
```

## File: crates/engine/tree/docs/AGENTS.md
```markdown
# docs

## Purpose
Design documentation and diagrams for `reth-engine-tree`, including Mermaid sources and rendered images.

## Contents (one hop)
### Subdirectories
- [x] `mermaid/` - (skip: Mermaid `.mmd` sources + rendered `.png` diagrams)

### Files
- `root.md` - high-level documentation entrypoint for engine-tree internals and design notes.

## Key APIs (no snippets)
- **Docs entrypoints**: `root.md` (narrative docs), `mermaid/*.mmd` (diagram sources)

## Relationships
- **Describes**: `reth/crates/engine/tree/src/` modules and their interactions (handler/orchestrator/backfill/persistence/tree).
```

## File: crates/engine/tree/src/tree/payload_processor/AGENTS.md
```markdown
# payload_processor

## Purpose
Implements the engine-tree "payload processing" pipeline: prewarm execution/state caches and compute state roots + trie updates for a block using parallel tasks (prewarm -> multiproof -> sparse trie).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `bal.rs` - EIP-7928 Block Access List helpers (slot iteration, BAL -> `HashedPostState`).
- `configured_sparse_trie.rs` - runtime switch between serial and parallel sparse trie implementations.
- `executor.rs` - `WorkloadExecutor` wrapper for spawning blocking tasks (reuses/creates a Tokio runtime).
- `mod.rs` - main entrypoint; wires tasks together and defines handles/types/constants.
- `multiproof.rs` - multiproof task: turns state updates/BAL into ordered `SparseTrieUpdate`s + proofs (with metrics).
- `prewarm.rs` - cache prewarming task (tx-parallel execution or BAL-driven prefetch) and state-update emission.
- `sparse_trie.rs` - sparse trie task: applies updates and computes final state root + `TrieUpdates`.

## Key APIs (no snippets)
- **Types**: `PayloadProcessor<Evm>`, `PayloadHandle<Tx, Err, R>`, `ExecutionEnv<Evm>`, `WorkloadExecutor`, `SparseTrieUpdate`, `StateRootComputeOutcome`
- **Functions**: `PayloadProcessor::spawn()`, `PayloadProcessor::spawn_cache_exclusive()`, `PayloadHandle::state_root()`, `PayloadHandle::state_hook()`, `update_sparse_trie()`, `bal_to_hashed_post_state()`, `total_slots()`
- **Constants**: `PARALLEL_SPARSE_TRIE_PARALLELISM_THRESHOLDS`, `SPARSE_TRIE_MAX_NODES_SHRINK_CAPACITY`, `SPARSE_TRIE_MAX_VALUES_SHRINK_CAPACITY`

## Relationships
- **Used by**: `reth/crates/engine/tree/src/tree/payload_validator.rs` (`BasicEngineValidator` owns a `PayloadProcessor` for state root / trie-update computation during block validation).
- **Internal flow**: `PrewarmCacheTask` warms caches and emits state updates -> `MultiProofTask` produces `SparseTrieUpdate` proofs in order -> `SparseTrieTask` applies proofs/updates and produces `(state_root, trie_updates)`.
- **Depends on**: `reth_trie_parallel` (proof workers), `reth_trie_sparse{,_parallel}` (sparse trie), `reth_evm`/`revm` (transaction execution for prewarming).

## Notes
- `prewarm.rs` includes special handling for "system" tx types (> 4) to broadcast the first tx to all workers (useful for L2s that embed metadata in the first transaction).
```

## File: crates/engine/tree/src/tree/AGENTS.md
```markdown
# tree

## Purpose
Implements the in-memory "engine tree" that executes/validates incoming payloads and forkchoice updates, tracks canonical vs fork blocks, and coordinates background persistence while staying responsive to Engine API traffic.

## Contents (one hop)
### Subdirectories
- [x] `payload_processor/` - Prewarming + multiproof + sparse-trie pipeline used to compute state roots and trie updates for executed payloads.

### Files
- `block_buffer.rs` - `BlockBuffer`: bounded FIFO buffer for unconnected blocks (missing parent) with child tracking and eviction.
- `cached_state.rs` - `CachedStateProvider` + `ExecutionCache`: cross-block caches for accounts/storage/bytecode and helpers for proofs/state root.
- `error.rs` - internal error types for block/payload insertion and persistence advancement.
- `instrumented_state.rs` - `InstrumentedStateProvider`: wraps a provider and records per-call + lifetime latency metrics.
- `invalid_headers.rs` - `InvalidHeaderCache`: LRU cache mapping invalid blocks/ancestors with hit-based eviction and metrics.
- `metrics.rs` - `EngineApiMetrics` and related metric groups for execution, validation, persistence, and reorg observability.
- `mod.rs` - module root: defines `EngineApiTreeHandler`/`EngineApiTreeState`/`StateProviderBuilder` and re-exports key tree APIs.
- `payload_validator.rs` - `BasicEngineValidator` and `EngineValidator`: reusable payload/block validation + execution + state root computation.
- `persistence_state.rs` - `PersistenceState`: tracks in-flight persistence tasks and last persisted block.
- `precompile_cache.rs` - `PrecompileCacheMap` and `CachedPrecompile`: per-precompile input->output caching with metrics.
- `state.rs` - `TreeState`: stores executed blocks connected to canonical chain, fork structure, and canonical head tracking.
- `tests.rs` - unit tests for tree internals.
- `trie_updates.rs` - trie update helpers used during state root computation / comparison.

## Key APIs (no snippets)
- **Types**: `EngineApiTreeHandler`, `EngineApiTreeState`, `TreeState`, `BlockBuffer`, `InvalidHeaderCache`, `PersistenceState`, `StateProviderBuilder`
- **Traits**: `EngineValidator` (engine-tree validation interface), `PayloadValidator` (from `reth-engine-primitives`, implemented by chain-specific validators)
- **Functions/Methods**: `EngineApiTreeHandler::new()`, `EngineApiTreeHandler::spawn_new()`, `TreeState::insert_executed()`, `TreeState::prune_finalized_sidechains()`, `BlockBuffer::insert_block()`, `BlockBuffer::remove_block_with_children()`

## Relationships
- **Used by**: `reth/crates/engine/tree/src/engine.rs` (`EngineApiRequestHandler` forwards events into `EngineApiTreeHandler`), `reth/crates/engine/service/src/service.rs` (spawns the tree handler thread and wires it into `EngineService`).
- **Depends on**: `reth-provider` (state/db access), `reth-consensus` (header/body rules), `reth-evm`/`revm` (execution), `reth-trie*` crates (proofs/state-root), `reth-engine-primitives` (Engine API message/event/config types).
- **Data/control flow**:
  - Engine API messages arrive -> validated/executed -> tree state updated (canonical/fork) -> persistence actions scheduled -> metrics/events emitted back to orchestrator.

## Notes
- The design biases toward keeping Engine API handling off the DB write path; persistence is tracked via `PersistenceState` and executed asynchronously via `persistence::PersistenceHandle`.
```

## File: crates/engine/tree/src/AGENTS.md
```markdown
# src

## Purpose
Core implementation of the `reth-engine-tree` crate: orchestrates live Engine API handling, backfill (pipeline) sync, on-demand block downloads, in-memory chain tracking, and background persistence.

## Contents (one hop)
### Subdirectories
- [x] `tree/` - Engine API execution/validation core: in-memory `TreeState`, payload validation, state-root pipeline, buffering, caching, and engine metrics.

### Files
- `backfill.rs` - backfill sync types (`BackfillSyncState`, `BackfillAction`, `BackfillEvent`) and `PipelineSync` wrapper for running the stages pipeline.
- `chain.rs` - `ChainOrchestrator` state machine that coordinates a `ChainHandler` with a `BackfillSync` implementation.
- `download.rs` - `BlockDownloader` trait and `BasicBlockDownloader` implementation for fetching full blocks/ranges on demand.
- `engine.rs` - Engine API-facing handler types: `EngineHandler`, `EngineRequestHandler`, `EngineApiRequestHandler`, request/event enums, and download requests.
- `lib.rs` - crate entrypoint and module wiring; documents design goals (in-memory critical path, persistence off-thread).
- `metrics.rs` - metrics structs for downloader and persistence services.
- `persistence.rs` - persistence service (`PersistenceService`, `PersistenceAction`, `PersistenceHandle`) that writes blocks/removals to DB/static files and triggers pruning.
- `test_utils.rs` - helpers for engine-tree tests (test pipeline builder, test header insertion).

## Key APIs (no snippets)
- **Types**: `ChainOrchestrator`, `EngineHandler`, `EngineApiRequestHandler`, `PipelineSync`, `BasicBlockDownloader`, `PersistenceService`, `PersistenceHandle`
- **Traits**: `ChainHandler`, `BackfillSync`, `BlockDownloader`, `EngineRequestHandler`
- **Enums**: `BackfillAction`, `BackfillEvent`, `BackfillSyncState`, `DownloadRequest`, `DownloadOutcome`, `EngineApiKind`

## Relationships
- **Used by**: `reth/crates/engine/service/src/service.rs` (wraps the engine-tree orchestrator as a stream service).
- **Depends on**: `reth-engine-primitives` (Engine API message/event/config), `reth-stages-api` (pipeline targets/control flow), `reth-network-p2p` (block client), `reth-provider` (DB/provider factories), `reth-prune` (pruning during persistence).
- **Data/control flow**:
  - Incoming Engine API messages -> `engine.rs` handler -> `tree/` processing
  - Missing blocks -> `download.rs` downloader -> blocks returned to handler
  - Large gaps -> `backfill.rs` pipeline sync -> `chain.rs` orchestrator coordinates exclusive DB access
  - Persisted outputs -> `persistence.rs` background thread -> pruning + sync height metrics

## Notes
- The `chain.rs` orchestrator documents DB write-lock invariants: backfill/persistence must not deadlock with live handler writes.
```

## File: crates/engine/tree/tests/e2e-testsuite/AGENTS.md
```markdown
# e2e-testsuite

## Purpose
End-to-end integration tests for `reth-engine-tree`, exercising Engine API handling and chain progression behaviors (e.g., forkchoice/finalization edge cases).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `fcu_finalized_blocks.rs` - e2e coverage for forkchoiceUpdated interactions involving finalized blocks.
- `main.rs` - test harness entrypoint wiring the e2e test suite.

## Key APIs (no snippets)
- **Files-as-entrypoints**: `main.rs` (suite runner), `fcu_finalized_blocks.rs` (scenario coverage)

## Relationships
- **Used by**: `reth/crates/engine/tree/Cargo.toml` declares this as an integration test target (`[[test]] name = "e2e_testsuite"`).

## Notes
- These tests are intended to validate cross-component behavior (tree + engine handler + persistence/backfill coordination), not individual unit invariants.
```

## File: crates/engine/tree/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration test area for the `reth-engine-tree` crate.

## Contents (one hop)
### Subdirectories
- [x] `e2e-testsuite/` - End-to-end Engine API test suite (forkchoice/newPayload scenarios, finalized-block edge cases).

### Files
- (none)

## Key APIs (no snippets)
- **Test targets**: `e2e-testsuite/main.rs` (suite entrypoint)

## Relationships
- **Related crate**: `reth/crates/engine/tree/Cargo.toml` registers `tests/e2e-testsuite/main.rs` as an integration test target.
```

## File: crates/engine/tree/AGENTS.md
```markdown
# tree

## Purpose
`reth-engine-tree` crate root: packages the engine-tree implementation that keeps the node in sync during live operation (Engine API), can trigger backfill pipeline runs for large gaps, and persists executed data off the critical path.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - benchmarks for engine-tree hot paths (channels, state-root task).
- [x] `docs/` - design docs and diagrams (Mermaid sources + rendered images).
- [x] `src/` - implementation of engine-tree (handlers, orchestrator, backfill, download, persistence, in-memory tree).
- [x] `tests/` - integration/e2e tests for engine-tree behavior.
- [x] `test-data/` - (skip: test fixtures only; contains `.rlp` sample payloads)

### Files
- `Cargo.toml` - crate manifest for `reth-engine-tree` (features, deps, benches/tests registration).

## Key APIs (no snippets)
- **Primary modules**: `src/lib.rs` (overview), `src/engine.rs` (Engine API handler), `src/tree/` (tree core), `src/persistence.rs` (background persistence), `src/backfill.rs` (pipeline sync)

## Relationships
- **Used by**: `reth-engine-service` (wires engine-tree into a pollable service for the node).
- **Depends on**: `reth-engine-primitives` (message/event/config types), `reth-provider` (DB/provider), `reth-network-p2p` (block downloads), `reth-stages-api` (pipeline control flow).
```

## File: crates/engine/util/src/AGENTS.md
```markdown
# src

## Purpose
Stream utilities for consuming a `BeaconEngineMessage` stream: wrappers for skipping messages, storing Engine API calls to disk, and simulating reorg behavior for testing/experimentation.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `engine_store.rs` - on-disk message store + `EngineStoreStream` wrapper that persists FCU/newPayload calls as JSON.
- `lib.rs` - `EngineMessageStreamExt` extension trait (combinators: skip/store/reorg).
- `reorg.rs` - `EngineReorg` stream wrapper that injects reorg sequences at a configured frequency/depth.
- `skip_fcu.rs` - `EngineSkipFcu` wrapper that skips a number of forkchoiceUpdated messages (responds with syncing).
- `skip_new_payload.rs` - `EngineSkipNewPayload` wrapper that skips a number of newPayload messages (responds with syncing).

## Key APIs (no snippets)
- **Traits**: `EngineMessageStreamExt` (extension methods for `Stream<Item = BeaconEngineMessage<_>>`)
- **Types**: `EngineMessageStore`, `EngineStoreStream`, `StoredEngineApiMessage`, `EngineReorg`, `EngineSkipFcu`, `EngineSkipNewPayload`

## Relationships
- **Depends on**: `reth-engine-primitives` (message types), `reth-engine-tree` (validator used by reorg simulation), `reth-fs-util` (filesystem helpers), `serde` (JSON serialization).
- **Used by**: tooling/tests/bench scenarios that want deterministic Engine API input playback or fault injection.

## Notes
- The "skip" wrappers send immediate SYNCING-style responses via the oneshot channels, so the upstream caller doesn't hang when messages are intentionally dropped.
```

## File: crates/engine/util/AGENTS.md
```markdown
# util

## Purpose
`reth-engine-util` crate root: convenience utilities around Engine API message streams (store/replay/skip/reorg simulation).

## Contents (one hop)
### Subdirectories
- [x] `src/` - stream wrappers and the `EngineMessageStreamExt` extension trait.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-util`.

## Key APIs (no snippets)
- **Exports**: `EngineMessageStreamExt` (and wrappers reachable via its combinators).
```

## File: crates/engine/AGENTS.md
```markdown
# engine

## Purpose
Top-level engine subsystem directory: contains the crates that implement reth's Engine API handling, live-sync "engine tree", orchestration/persistence wiring, and supporting utilities for testing/debugging.

## Contents (one hop)
### Subdirectories
- [x] `invalid-block-hooks/` - Debugging hooks for invalid blocks (e.g., witness generation via `InvalidBlockHook`).
- [x] `local/` - Local/dev-chain components that generate Engine API traffic (`LocalMiner`, payload attributes builder).
- [x] `primitives/` - Shared Engine API types/traits/events/config (`BeaconEngineMessage`, `ConsensusEngineEvent`, `TreeConfig`).
- [x] `service/` - `EngineService` wiring that composes engine-tree + downloader + pipeline backfill into a pollable service.
- [x] `tree/` - Core `reth-engine-tree` implementation (Engine API request handling, in-memory tree state, backfill, downloads, persistence).
- [x] `util/` - Stream utilities around engine message streams (store/replay/skip/reorg simulation).

### Files
- (none)

## Key APIs (no snippets)
- **Main building blocks**:
  - `tree/` - `EngineApiTreeHandler`, `BasicEngineValidator`, `ChainOrchestrator`, `PipelineSync`, `PersistenceHandle`
  - `service/` - `EngineService`
  - `primitives/` - `BeaconEngineMessage`, `ConsensusEngineHandle`, `ConsensusEngineEvent`, `TreeConfig`

## Relationships
- **Integrated by**: node launch/wiring layers outside this subtree (Engine API RPC ingress -> engine message stream -> engine service/tree).
- **Design intent**: keep Engine API responsiveness high by minimizing DB writes on the critical path and delegating persistence/pruning to background workers.
```

## File: crates/era/src/common/AGENTS.md
```markdown
# common

## Purpose
Shared utilities for ERA/E2Store formats: common decoding helpers and generic file-format traits used by both `.era` and `.era1` implementations.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for shared utilities.
- **Key items**: `decode`, `file_ops`
- **Interactions**: re-exported/used by `era`, `era1`, and `e2s` layers.
- **Knobs / invariants**: none.

#### `decode.rs`
- **Role**: Defines a small extension trait for decoding typed values from compressed RLP payloads.
- **Key items**: `DecodeCompressedRlp`, `DecodeCompressedRlp::decode<T>()`
- **Interactions**: used by compressed execution/consensus payload wrappers to expose "decompress + RLP decode" as a single call.
- **Knobs / invariants**: error type is normalized to `E2sError`.

#### `file_ops.rs`
- **Role**: Generic era file format traits and naming helpers shared across file types; abstracts "reader/writer" patterns over `Read+Seek` / `Write`.
- **Key items**: `EraFileFormat`, `EraFileId`, `StreamReader`, `FileReader`, `StreamWriter`, `FileWriter`, `EraFileType`, `format_hash()`
- **Interactions**: implemented by `EraFile`/`Era1File` and their readers/writers; provides standardized filename formatting for `.era`/`.era1`.
- **Knobs / invariants**: `EraFileId::ITEMS_PER_ERA` and `era_count()` enforce the "max 8192 items per file" expectation; `EraFileType` determines extension and naming format.

## Key APIs (no snippets)
- **Format traits**: `EraFileFormat`, `EraFileId`
- **I/O traits**: `StreamReader`, `FileReader`, `StreamWriter`, `FileWriter`
- **File typing/naming**: `EraFileType`, `EraFileId::to_file_name()`, `EraFileType::format_filename()`
- **Decode helper**: `DecodeCompressedRlp`

## Relationships
- **Used by**: `e2s` (core TLV records), `era` (consensus/CL), and `era1` (execution/EL) modules to share I/O abstractions and naming conventions.
```

## File: crates/era/src/e2s/AGENTS.md
```markdown
# e2s

## Purpose
Core e2store (`.e2s`) primitives: TLV record encoding/decoding, buffered readers/writers, and shared error types used as the foundation for `.era` and `.era1` file formats.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for the core e2store building blocks.
- **Key items**: `error`, `file`, `types`
- **Interactions**: consumed by `era`/`era1` readers and writers as the underlying record format.
- **Knobs / invariants**: none.

#### `error.rs`
- **Role**: Unified error enum for e2store operations (I/O, SSZ record parsing, snappy compression/decompression, RLP).
- **Key items**: `E2sError`
- **Interactions**: used across all modules to normalize failures while reading/writing and compressing/decompressing entries.
- **Knobs / invariants**: distinguishes "format invariant" errors like `ReservedNotZero`.

#### `types.rs`
- **Role**: Defines the e2store TLV record layout (`Header` + `Entry`), special record IDs (version/index), and an `IndexEntry` helper for serializing offset indices.
- **Key items**: `Header`, `Entry`, `Version`, `IndexEntry`, `VERSION`, `SLOT_INDEX`
- **Interactions**: `Entry::read()` / `Entry::write()` are used by `E2StoreReader`/`E2StoreWriter`; `IndexEntry` is implemented by `SlotIndex`/`BlockIndex` for `.era`/`.era1`.
- **Knobs / invariants**: header `reserved` must be zero; `IndexEntry::from_entry()` validates size and count layout.

#### `file.rs`
- **Role**: Buffered file reader/writer for sequences of e2store `Entry` records, including version handling.
- **Key items**: `E2StoreReader<R>`, `E2StoreWriter<W>`, `read_version()`, `read_next_entry()`, `write_version()`, `write_entry()`
- **Interactions**: `EraReader`/`EraWriter` and `Era1Reader`/`Era1Writer` build on top of this abstraction.
- **Knobs / invariants**: writer automatically inserts the version record before the first non-version entry if not written explicitly.

## Key APIs (no snippets)
- **Records**: `Header`, `Entry`, `Version`
- **I/O**: `E2StoreReader`, `E2StoreWriter`
- **Errors**: `E2sError`
- **Index helper**: `IndexEntry`

## Relationships
- **Used by**: `.era` and `.era1` implementations as the low-level record format and I/O layer.
```

## File: crates/era/src/era/types/AGENTS.md
```markdown
# types

## Purpose
Data types for `.era` (consensus-layer) files: compressed beacon block/state record wrappers and the `.era` content grouping/index/id types used by readers/writers.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for `.era` types.
- **Key items**: `consensus`, `group`
- **Interactions**: imported by `era::file` for assembling/parsing complete `.era` files.
- **Knobs / invariants**: none.

#### `consensus.rs`
- **Role**: Snappy-compressed SSZ payload wrappers for post-merge consensus data (signed beacon blocks and beacon states), with bounded decompression for safety.
- **Key items**: `CompressedSignedBeaconBlock`, `CompressedBeaconState`, `COMPRESSED_SIGNED_BEACON_BLOCK`, `COMPRESSED_BEACON_STATE`, `decompress()`, `from_ssz()`
- **Interactions**: converted to/from `e2s::types::Entry` when reading/writing `.era` files; external SSZ decoding into concrete beacon types is intentionally out-of-scope (caller responsibility).
- **Knobs / invariants**: bounded decompression limits (e.g. max decompressed beacon state size) to avoid pathological inputs.

#### `group.rs`
- **Role**: Defines `.era` file content grouping and identifiers: `EraGroup` (blocks + era-state + indices + extra entries), `SlotIndex` (offset index), and `EraId` (file naming metadata).
- **Key items**: `EraGroup`, `SlotIndex`, `EraId`, `SLOTS_PER_HISTORICAL_ROOT`
- **Interactions**: `SlotIndex` implements `IndexEntry` for e2store; `EraId` implements `EraFileId` for standardized naming.
- **Knobs / invariants**: genesis era is represented by absence of `slot_index` and empty block list; `slot_range()` drives ID construction.

## Key APIs (no snippets)
- **Compressed consensus payloads**: `CompressedSignedBeaconBlock`, `CompressedBeaconState`
- **Grouping/indexing**: `EraGroup`, `SlotIndex`
- **File identity**: `EraId`

## Relationships
- **Used by**: `era::file` to parse/assemble `.era` streams and to enforce structure ordering (blocks -> state -> indices).
```

## File: crates/era/src/era/AGENTS.md
```markdown
# era

## Purpose
Implements `.era` (consensus-layer) file support: reading/writing the era file structure on top of e2store records, and assembling the parsed stream into an `EraFile` (blocks + era-state + indices).

## Contents (one hop)
### Subdirectories
- [x] `types/` - compressed consensus payload wrappers and `.era` group/index/id types.

### Files
- `mod.rs` - module wiring for `.era` primitives.
  - **Key items**: `file`, `types`
- `file.rs` - core `.era` file model plus streaming reader/writer implementations built on `E2StoreReader`/`E2StoreWriter`.
  - **Key items**: `EraFile`, `EraReader<R>`, `EraWriter<W>`, `BeaconBlockIterator<R>`, `EraReader::read_and_assemble()`

## Key APIs (no snippets)
- **Models**: `EraFile`
- **I/O**: `EraReader`, `EraWriter`, `BeaconBlockIterator`

## Relationships
- **Builds on**: `e2s` record format and `common::file_ops` trait layer.
- **Produces/consumes**: `types::EraGroup` and compressed consensus payload entries.
```

## File: crates/era/src/era1/types/AGENTS.md
```markdown
# types

## Purpose
Data types for `.era1` (execution-layer) files: compressed execution block components and the `.era1` grouping/index/id structures used by readers/writers.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for `.era1` types.
- **Key items**: `execution`, `group`
- **Interactions**: imported by `era1::file` for assembling/parsing complete `.era1` files.
- **Knobs / invariants**: none.

#### `execution.rs`
- **Role**: Execution-layer record wrappers for `.era1`: snappy-framed RLP codecs for block headers/bodies/receipts, plus total difficulty and accumulator records.
- **Key items**: `CompressedHeader`, `CompressedBody`, `CompressedReceipts`, `TotalDifficulty`, `Accumulator`, `BlockTuple`, `SnappyRlpCodec<T>`, `MAX_BLOCKS_PER_ERA1`
- **Interactions**: implements `DecodeCompressedRlp` for compressed types; converts to/from `e2s::types::Entry` using record IDs like `COMPRESSED_HEADER`/`COMPRESSED_BODY`/`COMPRESSED_RECEIPTS`.
- **Knobs / invariants**: `.era1` size/structure constraints depend on accumulator/index; `MAX_BLOCKS_PER_ERA1` (8192) caps per-file block tuples.

#### `group.rs`
- **Role**: Defines `.era1` content grouping and identifiers: `Era1Group` (block tuples + accumulator + block index + extra entries), `BlockIndex` (offset index), and `Era1Id` (file naming metadata).
- **Key items**: `Era1Group`, `BlockIndex`, `Era1Id`, `BLOCK_INDEX`
- **Interactions**: `BlockIndex` implements `IndexEntry`; `Era1Id` implements `EraFileId` for standardized naming.
- **Knobs / invariants**: block index offsets length must match block count; file naming is derived from start block and (optional) hash.

## Key APIs (no snippets)
- **Compressed execution payloads**: `CompressedHeader`, `CompressedBody`, `CompressedReceipts`
- **Block tuple/metadata**: `BlockTuple`, `TotalDifficulty`, `Accumulator`
- **Grouping/indexing**: `Era1Group`, `BlockIndex`
- **File identity**: `Era1Id`

## Relationships
- **Used by**: `era1::file` to parse/assemble `.era1` streams and to enforce structure ordering (block tuples -> extras -> accumulator -> block index).
```

## File: crates/era/src/era1/AGENTS.md
```markdown
# era1

## Purpose
Implements `.era1` (execution-layer) file support: reading/writing the era1 file structure on top of e2store records, and assembling the parsed stream into an `Era1File` (block tuples + accumulator + block index).

## Contents (one hop)
### Subdirectories
- [x] `types/` - compressed execution payload wrappers and `.era1` group/index/id types.

### Files
- `mod.rs` - module wiring for `.era1` primitives.
  - **Key items**: `file`, `types`
- `file.rs` - core `.era1` file model plus streaming reader/writer implementations built on `E2StoreReader`/`E2StoreWriter`.
  - **Key items**: `Era1File`, `Era1Reader<R>`, `Era1Writer<W>`, `BlockTupleIterator<R>`, `Era1File::get_block_by_number()`

## Key APIs (no snippets)
- **Models**: `Era1File`
- **I/O**: `Era1Reader`, `Era1Writer`, `BlockTupleIterator`

## Relationships
- **Builds on**: `e2s` record format and `common::file_ops` trait layer.
- **Produces/consumes**: `types::Era1Group` and compressed execution payload entries.
```

## File: crates/era/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-era` core logic for e2store-based history files: the shared `.e2s` record layer, `.era` (consensus/CL) and `.era1` (execution/EL) file models with streaming readers/writers, and helper utilities for compression/decompression and naming.

## Contents (one hop)
### Subdirectories
- [x] `common/` - shared decoding helpers and generic file-format traits/naming utilities.
- [x] `e2s/` - core e2store TLV records, readers/writers, and error types (foundation for `.era`/`.era1`).
- [x] `era/` - `.era` consensus-layer file support (beacon block/state compression wrappers, group/index/id types, reader/writer).
- [x] `era1/` - `.era1` execution-layer file support (compressed header/body/receipts, accumulator/index, reader/writer).

### Files
- `lib.rs` - crate root documentation and module wiring.
  - **Key items**: `common`, `e2s`, `era`, `era1`
- `test_utils.rs` - test-only helpers for constructing sample compressed blocks/states and receipts for unit/integration tests.
  - **Key items**: `create_header()`, `create_sample_block()`, `create_test_block_with_compressed_data()`, `create_beacon_block()`, `create_beacon_state()`

## Key APIs (no snippets)
- **E2Store**: `E2StoreReader`, `E2StoreWriter`, `Entry`, `E2sError`
- **Era (CL)**: `EraFile`, `EraReader`, `EraWriter`, `CompressedSignedBeaconBlock`, `CompressedBeaconState`
- **Era1 (EL)**: `Era1File`, `Era1Reader`, `Era1Writer`, `BlockTuple`, `CompressedHeader`, `CompressedBody`, `CompressedReceipts`

## Relationships
- **Used by**: downloader/utility crates that fetch and process era/era1 history files for history expiry workflows.
```

## File: crates/era/tests/it/era/AGENTS.md
```markdown
# era

## Purpose
Integration tests for `.era` (consensus-layer) files: validate genesis handling and roundtrip read/decode/write behavior against downloaded fixtures.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - test module wiring.
  - **Key items**: `genesis`, `roundtrip`
- `genesis.rs` - tests around genesis-era `.era` file structure/edge cases.
  - **Key items**: (see tests in file)
- `roundtrip.rs` - roundtrip tests for `.era` files (read -> decode/decompress -> re-encode -> write -> re-read and compare).
  - **Key items**: `EraReader`, `EraWriter`, `EraFile`

## Relationships
- **Driven by**: `tests/it/main.rs` helper that downloads fixtures into a temp dir and opens them via `EraReader`.
```

## File: crates/era/tests/it/era1/AGENTS.md
```markdown
# era1

## Purpose
Integration tests for `.era1` (execution-layer) files: validate genesis handling and full roundtrip read/decode/write behavior against downloaded fixtures (mainnet/sepolia samples).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - test module wiring.
  - **Key items**: `genesis`, `roundtrip`
- `genesis.rs` - tests around genesis-era `.era1` file structure/edge cases.
  - **Key items**: (see tests in file)
- `roundtrip.rs` - roundtrip tests for `.era1` files (read -> decompress/decode -> re-encode/recompress -> write -> re-read and compare).
  - **Key items**: `Era1Reader`, `Era1Writer`, `BlockTuple`, `CompressedHeader`, `CompressedBody`, `CompressedReceipts`

## Relationships
- **Driven by**: `tests/it/main.rs` helper that downloads fixtures into a temp dir and opens them via `Era1Reader`.
```

## File: crates/era/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration test harness for `reth-era`: downloads a small curated set of `.era`/`.era1` files from public hosts and validates that the library can read, decompress/decode, and roundtrip them correctly.

## Contents (one hop)
### Subdirectories
- [x] `era/` - `.era` file integration tests (genesis + roundtrip).
- [x] `era1/` - `.era1` file integration tests (genesis + roundtrip).

### Files
- `main.rs` - shared test harness: fixture URL lists, caching downloader, and helpers for opening files with `EraReader`/`Era1Reader`.
  - **Key items**: `EraTestDownloader`, `EraFileType`, `EraReader`, `Era1Reader`

## Relationships
- **Uses**: `reth-era-downloader` to fetch file lists and download specific files to a temp dir for testing.
```

## File: crates/era/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for `reth-era` (download fixtures and verify parsing/roundtrips).

## Contents (one hop)
### Subdirectories
- [x] `it/` - integration test harness + `.era`/`.era1` roundtrip suites.

### Files
- (none)
```

## File: crates/era/AGENTS.md
```markdown
# era

## Purpose
`reth-era` crate: core logic for e2store-based history files, implementing `.era` (consensus-layer) and `.era1` (execution-layer) formats with streaming readers/writers, compression wrappers, indices, and standardized file naming for history expiry workflows.

## Contents (one hop)
### Subdirectories
- [x] `src/` - e2store primitives plus `.era`/`.era1` models, types, and readers/writers.
- [x] `tests/` - integration tests that download sample `.era`/`.era1` files and verify parsing/roundtrips.

### Files
- `Cargo.toml` - crate manifest (SSZ + snappy + RLP dependencies; test-time downloader wiring).
  - **Key items**: `description = "e2store and era1 files core logic"`

## Key APIs (no snippets)
- **E2Store**: `E2StoreReader`, `E2StoreWriter`, `Entry`, `E2sError`
- **Era / Era1**: `EraFile`/`EraReader`/`EraWriter`, `Era1File`/`Era1Reader`/`Era1Writer`

## Relationships
- **Used by**: ERA download/import/export tooling to store and distribute pruned history data.
```

## File: crates/era-downloader/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-era-downloader`: an async interface for discovering and fetching `.era`/`.era1` history files from remote hosts (or local directories), with streaming/concurrency controls and optional checksum verification (for `.era1`).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires modules and exposes the public API surface for clients/streams.
- **Key items**: `EraClient`, `EraStream`, `EraStreamConfig`, `EraMeta`, `read_dir()`, `BLOCKS_PER_FILE`
- **Interactions**: re-exports `HttpClient` trait to allow custom HTTP backends (used heavily in tests with a stub client).
- **Knobs / invariants**: `BLOCKS_PER_FILE` (8192) is used to map block numbers to era1 file indices.

#### `client.rs`
- **Role**: Remote download client: fetches an index page and (for `.era1`) `checksums.txt`, extracts ordered filenames, downloads files to disk, and verifies checksums when applicable.
- **Key items**: `HttpClient`, `EraClient<Http>`, `download_to_file()`, `fetch_file_list()`, `number_to_file_name()`, `verify_checksum()`, `recover_index()`, `delete_outside_range()`
- **Interactions**: uses `reth_era::common::file_ops::EraFileType` to distinguish `.era` vs `.era1`; streams bytes via `HttpClient` and writes to `tokio::fs::File`.
- **Knobs / invariants**: checksum verification is enforced for `.era1` (SHA-256); `.era` hosts may not provide checksums so verification is skipped.

#### `stream.rs`
- **Role**: Asynchronous stream abstraction that schedules and runs downloads concurrently while enforcing a cap on concurrent downloads and on-disk file count.
- **Key items**: `EraStream<Http>`, `EraStreamConfig`, `EraMeta`, `EraRemoteMeta`, `EraStreamConfig::start_from()`
- **Interactions**: coordinates with `EraClient` for URL discovery and download; returns temporary local files whose lifecycle is controlled via `EraMeta::mark_as_processed()`.
- **Knobs / invariants**: `max_files` limits how many downloaded files may exist simultaneously; `max_concurrent_downloads` controls parallelism; internal state machine advances through "fetch list -> delete outside range -> recover index -> schedule URLs".

#### `fs.rs`
- **Role**: Local-directory adapter: reads `.era1` files from disk in index order starting at a computed start point, validating against `checksums.txt`.
- **Key items**: `read_dir()`, `EraLocalMeta`, `EraMeta::mark_as_processed()` (no-op)
- **Interactions**: uses `sha2` to compute file checksums and aligns them with `checksums.txt` lines by index.
- **Knobs / invariants**: requires `checksums.txt` to exist and have at least as many entries as files in range; start position is derived from `start_from / BLOCKS_PER_FILE`.

## Key APIs (no snippets)
- **HTTP abstraction**: `HttpClient`
- **Remote client**: `EraClient`
- **Streaming download**: `EraStream`, `EraStreamConfig`, `EraMeta`
- **Local ingestion**: `read_dir()`

## Relationships
- **Used by**: ERA import tooling (`reth-cli-commands` `import-era`, and other utilities) that want to stream history files into storage/pipeline ingestion.
```

## File: crates/era-downloader/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for `reth-era-downloader`: validate file list parsing, URL selection, download streaming behavior, checksum enforcement (ERA1), and local-directory streaming.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `main.rs`
- **Role**: Test harness that bundles all integration tests into a single binary and provides a `StubClient` implementing `HttpClient` using embedded fixtures from `tests/res/`.
- **Key items**: `StubClient`, `ERA1_*`/`ERA_*` fixture constants, module wiring (`checksums`, `download`, `fs`, `list`, `stream`)
- **Interactions**: `StubClient::get()` maps known URLs to fixture bytes (HTML index pages, checksums.txt, and small `.era`/`.era1` sample files).

#### `list.rs`
- **Role**: Tests index-page parsing and ordered filename lookup (`number_to_file_name`) after `fetch_file_list()`.
- **Key items**: `EraClient::fetch_file_list()`, `EraClient::number_to_file_name()`

#### `download.rs`
- **Role**: Tests URL construction and download-to-disk behavior for both `.era1` and `.era` after fetching the file list.
- **Key items**: `EraClient::url()`, `EraClient::download_to_file()`, `EraClient::files_count()`

#### `stream.rs`
- **Role**: Tests `EraStream` scheduling and error behavior (including missing download directory), asserting that downloaded paths match expected filenames.
- **Key items**: `EraStream`, `EraStreamConfig`, `EraStreamConfig::with_max_files()`, `EraStreamConfig::with_max_concurrent_downloads()`

#### `checksums.rs`
- **Role**: Tests that invalid ERA1 checksums are detected and surfaced as errors (using a `FailingClient` with intentionally wrong `checksums.txt`).
- **Key items**: `FailingClient`, checksum mismatch assertions

#### `fs.rs`
- **Role**: Tests `read_dir()` behavior for local directories (valid checksums, invalid checksums, partial failures, and missing `checksums.txt`).
- **Key items**: `read_dir()`, checksum mismatch error strings

## Relationships
- **Uses fixtures from**: `tests/res/` (embedded via `include_bytes!` in `main.rs`).
```

## File: crates/era-downloader/tests/AGENTS.md
```markdown
# tests

## Purpose
Test suites and fixtures for `reth-era-downloader`.

## Contents (one hop)
### Subdirectories
- [x] `it/` - integration tests (using a stub HTTP client and embedded fixtures).
- [x] `res/` - (skip: test fixtures) HTML index pages, checksums, and small sample `.era`/`.era1` files embedded by tests.

### Files
- (none)
```

## File: crates/era-downloader/AGENTS.md
```markdown
# era-downloader

## Purpose
`reth-era-downloader` crate: provides an async streaming interface for discovering and downloading ERA history files (`.era1` and `.era`) from remote hosts (or reading `.era1` from local dirs), with configurable concurrency and on-disk retention limits.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EraClient` (index/checksum + downloads), `EraStream` (scheduling), and local `read_dir()` ingestion.
- [x] `tests/` - integration tests plus embedded fixtures for stubbed HTTP/local-dir scenarios.

### Files
- `Cargo.toml` - crate manifest (reqwest streaming + sha2 checksum verification for `.era1`).
  - **Key items**: `description = "An asynchronous stream interface for downloading ERA1 files"`

## Key APIs (no snippets)
- **Remote**: `EraClient`, `EraStream`, `EraStreamConfig`, `EraMeta`
- **Local**: `read_dir()`

## Relationships
- **Used by**: import tooling (e.g. CLI `import-era`) that streams history files for ingestion into storage/pipeline.
```

## File: crates/era-utils/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-era-utils`: higher-level utilities for importing `.era1` history into reth storage (via `storage-api`/providers/static files) and exporting stored block history back into `.era1` files.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires modules and re-exports the import/export helper APIs.
- **Key items**: `ExportConfig`, `export()`, `import()`, `process()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Interactions**: re-exports are consumed by CLI import/export paths and tests.
- **Knobs / invariants**: none (mostly surface consolidation).

#### `export.rs`
- **Role**: Exports block history from a provider into `.era1` files on disk using `Era1Writer`, chunking by configurable "blocks per file" and building accumulator + block index records.
- **Key items**: `ExportConfig`, `ExportConfig::validate()`, `export()`, `determine_export_range()`, `compress_block_data()`, `MAX_BLOCKS_PER_ERA1`
- **Interactions**: reads headers/bodies/receipts via `reth_storage_api` provider traits; uses `reth_era` compressed record types and `Era1Id` naming.
- **Knobs / invariants**: `max_blocks_per_file` must be \(1..=8192\); filename includes optional era count when using custom blocks-per-file.

#### `history.rs`
- **Role**: Imports history from downloaded `.era1` files into storage: opens/iterates the era1 reader, decodes headers/bodies, appends into static files + DB tables, and updates stage checkpoints/index tables.
- **Key items**: `import()`, `process()`, `ProcessIter`, `open()`, `decode()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Interactions**: consumes an `EraMeta` stream (from `reth_era_downloader`); appends headers into `StaticFileSegment::Headers` and bodies into DB; uses an ETL `Collector` to build hash->number indexes.
- **Knobs / invariants**: import updates stage checkpoints for `Headers`/`Bodies` to keep pipeline stages consistent with imported data.

## Key APIs (no snippets)
- **Import**: `import()`, `process()`, `process_iter()`, `ProcessIter`
- **Export**: `export()`, `ExportConfig`

## Relationships
- **Builds on**: `reth-era` (file formats), `reth-era-downloader` (file streaming), and `reth-storage-api`/providers (persisting history into reth storage).
```

## File: crates/era-utils/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for `reth-era-utils`: validate importing `.era1` history into a fresh provider factory and exporting history back into `.era1` files (including roundtrip expectations and file naming/chunking).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test harness: defines an `HttpClient` wrapper that fakes the remote index to expose only a single known era1 file while delegating actual downloads to a real `reqwest::Client`.
  - **Key items**: `ClientWithFakeIndex`, `ITHACA_ERA_INDEX_URL`
- `genesis.rs` - verifies exporting when the DB contains only genesis (exports a single `.era1` file and asserts basic properties).
  - **Key items**: `test_export_with_genesis_only`, `ExportConfig`, `export()`
- `history.rs` - imports from a remote era1 stream into a fresh state and tests export-after-import roundtrips (chunking and filename format assertions).
  - **Key items**: `test_history_imports_from_fresh_state_successfully`, `test_roundtrip_export_after_import`, `import()`, `export()`

## Relationships
- **Exercises**: `reth_era_downloader::EraStream` -> `reth_era_utils::import()` -> provider/static-files; then `reth_era_utils::export()` -> `.era1` files on disk.
```

## File: crates/era-utils/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for `reth-era-utils`.

## Contents (one hop)
### Subdirectories
- [x] `it/` - integration tests (import, export, and roundtrip validation).

### Files
- (none)
```

## File: crates/era-utils/AGENTS.md
```markdown
# era-utils

## Purpose
`reth-era-utils` crate: utilities to import block history from downloaded `.era1` files into reth storage (static files + DB + stage checkpoints) and to export stored history back into `.era1` files (with configurable chunking and naming).

## Contents (one hop)
### Subdirectories
- [x] `src/` - import/export logic on top of `reth-era` and `reth-storage-api` providers.
- [x] `tests/` - integration tests for genesis-only export and import->export roundtrips.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-era`, `reth-era-downloader`, storage/provider APIs).
  - **Key items**: `description = "Utilities to store and fetch history data with storage-api"`

## Key APIs (no snippets)
- **Import**: `import()`, `process()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Export**: `export()`, `ExportConfig`

## Relationships
- **Used by**: CLI era import/export commands and other tooling that integrates ERA history files with reth's storage layout.
```

## File: crates/errors/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-errors`: a small "umbrella" error crate that unifies common reth error types into a single `RethError` and re-exports frequently used error/result aliases.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports `RethError`/`RethResult` and common error types from consensus, execution, and storage crates.
- **Key items**: `RethError`, `RethResult`, `ConsensusError`, `BlockExecutionError`, `ProviderError`, `DatabaseError`
- **Interactions**: used by higher-level crates to depend on a single error type instead of wiring multiple subsystem errors.
- **Knobs / invariants**: `#![no_std]` + `alloc` for boxed errors.

#### `error.rs`
- **Role**: Defines `RethError` enum variants for core subsystems and an "Other" escape hatch, plus constructors and size assertions.
- **Key items**: `RethError`, `RethError::other()`, `RethError::msg()`, `RethResult<T>`
- **Interactions**: wraps `BlockExecutionError`, `ConsensusError`, `DatabaseError`, `ProviderError`.
- **Knobs / invariants**: size assertions on x86_64 ensure common error types don't grow unexpectedly.

## Key APIs (no snippets)
- **Error type**: `RethError`
- **Result alias**: `RethResult<T>`

## Relationships
- **Depends on**: `reth-consensus`, `reth-execution-errors`, `reth-storage-errors` (re-exported/wrapped).
```

## File: crates/errors/AGENTS.md
```markdown
# errors

## Purpose
`reth-errors` crate: high-level error types and re-exports for reth, consolidating consensus/execution/storage failures into a single `RethError` for ergonomic error handling across the codebase.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `RethError` definition and re-export surface.

### Files
- `Cargo.toml` - crate manifest (ties together consensus/execution/storage error crates).
  - **Key items**: dependencies on `reth-consensus`, `reth-execution-errors`, `reth-storage-errors`

## Key APIs (no snippets)
- **Type**: `RethError`
- **Alias**: `RethResult<T>`
```

## File: crates/ethereum/cli/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum-cli`: the Ethereum-flavoured `reth` CLI surface, including the top-level `Cli` parser/runner, chain-spec parsing for Ethereum networks, and the glue that executes `reth-cli-commands` using Ethereum node components.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Module wiring + public re-exports.
- **Key items**: `CliApp`, `ExtendedCommand`, `Cli`, `Commands`, `NoSubCmd`
- **Interactions**: consumers generally use `Cli::parse_args()` / `Cli::run()` from `interface`.

#### `interface.rs`
- **Role**: Defines the main clap `Cli` type and helpers to run commands with tracing/metrics initialized, plus methods to run with custom runners and/or component builders.
- **Key items**: `Cli<...>`, `Cli::run()`, `Cli::run_with_components()`, `Cli::with_runner()`, `Cli::with_runner_and_components()`, `Cli::init_tracing()`
- **Interactions**: dispatches to `reth_cli_commands` command implementations; uses `reth_node_builder` to launch Ethereum node types.

#### `app.rs`
- **Role**: Defines `CliApp` wrapper (parsed CLI + optional runner/tracing layers) and the shared `run_commands_with()` dispatch implementation used by both `CliApp` and `Cli` methods.
- **Key items**: `CliApp`, `CliApp::run()`, `CliApp::run_with_components()`, `run_commands_with()`, `ExtendedCommand`
- **Interactions**: installs Prometheus recorder; validates RPC module selections via `RpcModuleValidator`; executes each `Commands` variant on the configured `CliRunner`.

#### `chainspec.rs`
- **Role**: Ethereum chain spec parser for clap args: maps known chain names to embedded specs (mainnet/sepolia/holesky/hoodi/dev) or parses a genesis JSON (file path or inline JSON).
- **Key items**: `SUPPORTED_CHAINS`, `chain_value_parser()`, `EthereumChainSpecParser`
- **Interactions**: used by `reth_cli_commands::NodeCommand` parsing and other CLI flows that need `ChainSpecParser`.

## Key APIs (no snippets)
- **CLI**: `Cli`, `Commands`
- **Runner wrapper**: `CliApp`, `run_commands_with()`
- **Chain parsing**: `EthereumChainSpecParser`, `chain_value_parser()`

## Relationships
- **Builds on**: generic CLI crates (`reth-cli`, `reth-cli-commands`, `reth-cli-runner`) plus Ethereum node wiring (`reth-node-ethereum`).
```

## File: crates/ethereum/cli/AGENTS.md
```markdown
# cli

## Purpose
`reth-ethereum-cli` crate: Ethereum-specific CLI entrypoint and execution glue that wires `reth-cli-commands` to Ethereum node components (EVM config + beacon consensus) and provides Ethereum chain spec parsing.

## Contents (one hop)
### Subdirectories
- [x] `src/` - clap `Cli` type, command dispatch, and Ethereum chain spec parser.

### Files
- `Cargo.toml` - crate manifest (depends on core CLI crates + Ethereum node wiring).
  - **Key items**: feature flags for OTLP/tracy/jemalloc and CLI "dev" mode.

## Key APIs (no snippets)
- `Cli`, `CliApp`
- `EthereumChainSpecParser`
```

## File: crates/ethereum/consensus/src/AGENTS.md
```markdown
# src

## Purpose
Implements Ethereum beacon-chain style consensus validation (`reth-ethereum-consensus`): header/body pre-execution checks and post-execution checks aligned with Ethereum hardfork rules (Merge/Shanghai/Cancun/Prague, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines `EthBeaconConsensus` and implements `reth_consensus::{Consensus, HeaderValidator, FullConsensus}` traits for Ethereum chainspecs.
- **Key items**: `EthBeaconConsensus`, `EthBeaconConsensus::new()`, `HeaderValidator::validate_header()`, `HeaderValidator::validate_header_against_parent()`
- **Interactions**: delegates to `reth_consensus_common::validation::*` helpers; uses chainspec hardfork queries to gate EIP-specific header fields.
- **Knobs / invariants**: `max_extra_data_size` (defaults to `MAXIMUM_EXTRA_DATA_SIZE`); enforces Merge-specific header constraints (difficulty/nonce/ommers).

#### `validation.rs`
- **Role**: Implements post-execution validation: receipts root + logs bloom correctness, gas used correctness, and Prague requests hash validation.
- **Key items**: `validate_block_post_execution()`
- **Interactions**: consumes execution receipts + `Requests` and compares against header fields; returns `ConsensusError` variants with `GotExpected` diffs.

## Key APIs (no snippets)
- `EthBeaconConsensus`
- `validate_block_post_execution()`

## Relationships
- **Builds on**: `reth-consensus-common` (pre-exec/header helpers) and `reth-consensus` traits.
```

## File: crates/ethereum/consensus/AGENTS.md
```markdown
# consensus

## Purpose
`reth-ethereum-consensus` crate: Ethereum beacon consensus validator implementation (header/body pre-execution validation + post-execution receipt/gas/requests validation), parameterized by an Ethereum chainspec/hardfork schedule.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthBeaconConsensus` and post-execution validation helpers.

### Files
- `Cargo.toml` - crate manifest (depends on consensus traits, chainspec/hardforks, execution types).

## Key APIs (no snippets)
- `EthBeaconConsensus`
- `validate_block_post_execution()`
```

## File: crates/ethereum/engine-primitives/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum-engine-primitives`: Ethereum-specific Engine API primitive types and conversions, especially around payload building/output (`EthBuiltPayload`) and mapping sealed blocks into Engine API execution payload envelopes (V1-V5, including Cancun/Prague-era fields).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `EthEngineTypes`/`EthPayloadTypes` (bindings between reth payload types and Engine API types) and re-exports payload helpers and conversion errors.
- **Key items**: `EthEngineTypes`, `EthPayloadTypes`, `payload_id()`, `EthBuiltPayload`, `EthPayloadBuilderAttributes`, `BlobSidecars`, `EthPayloadAttributes`
- **Interactions**: implements `reth_payload_primitives::PayloadTypes` and `reth_engine_primitives::EngineTypes` for Ethereum nodes; converts `SealedBlock` -> `ExecutionData` (`ExecutionPayload` + sidecar).

#### `payload.rs`
- **Role**: Ethereum payload building and Engine API envelope conversion: holds `EthBuiltPayload` (sealed block + fees + sidecars + optional requests) and converts it into `ExecutionPayloadV1` / envelope V2-V5 with correct blobs bundle shape.
- **Key items**: `EthBuiltPayload`, `EthBuiltPayload::try_into_v3()`, `try_into_v4()`, `try_into_v5()`, `BlobSidecars`
- **Interactions**: uses `alloy_rpc_types_engine` envelope types and `alloy_eips` sidecar/request types; enforces "expected sidecar variant" rules across fork versions.

#### `error.rs`
- **Role**: Conversion error types for built-payload -> envelope conversions.
- **Key items**: `BuiltPayloadConversionError`

## Key APIs (no snippets)
- `EthEngineTypes`, `EthPayloadTypes`
- `EthBuiltPayload`, `BlobSidecars`
- `BuiltPayloadConversionError`

## Relationships
- **Used by**: Ethereum engine/payload builders and Engine API server logic that needs to serve `engine_getPayload*` responses for Ethereum forks.
```

## File: crates/ethereum/engine-primitives/AGENTS.md
```markdown
# engine-primitives

## Purpose
`reth-ethereum-engine-primitives` crate: Ethereum-specific Engine API payload types and conversions (including envelope V1-V5), bridging reth blocks/payload builders to `alloy_rpc_types_engine` representations.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthEngineTypes`/`EthBuiltPayload` and conversion helpers.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-engine-primitives`, `reth-payload-primitives`, and alloy Engine API types).

## Key APIs (no snippets)
- `EthEngineTypes`, `EthPayloadTypes`
- `EthBuiltPayload`, `BlobSidecars`
```

## File: crates/ethereum/evm/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-evm-ethereum`: Ethereum EVM configuration for reth, including revm spec selection, EVM environment/context derivation for block execution and Engine API payload execution, block assembly (header/body roots and fork-specific fields), and receipt building over reth primitive types.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `EthEvmConfig` (executor factory + block assembler), implements `ConfigureEvm` (and `ConfigureEngineEvm` for Engine API `ExecutionData`), and re-exports helper pieces.
- **Key items**: `EthEvmConfig`, `EthEvmConfig::mainnet()`, `EthEvmConfig::ethereum()`, `ConfigureEvm` impl, `ConfigureEngineEvm<ExecutionData>` impl
- **Interactions**: connects `reth-evm` traits to `alloy_evm`'s Ethereum execution context (`EthBlockExecutionCtx`) and `revm::SpecId` hardfork selection.
- **Knobs / invariants**: this crate does not enforce revm feature flags (noted in docs); fork selection uses timestamp/block-number + chainspec hardfork schedule.

#### `config.rs`
- **Role**: Re-exports revm spec selection helpers from `alloy_evm`.
- **Key items**: `revm_spec`, `revm_spec_by_timestamp_and_block_number`

#### `build.rs`
- **Role**: Ethereum block assembler: constructs an `alloy_consensus::Block` (header + body) from execution outputs, computing tx/receipt roots, bloom, withdrawals root, and fork-gated Cancun/Prague fields.
- **Key items**: `EthBlockAssembler`, `BlockAssembler::assemble_block()`
- **Interactions**: uses chainspec to decide withdrawals/request hash/Cancun blob fields; consumes `BlockExecutionResult` (receipts/requests/gas/blob gas) from execution.

#### `receipt.rs`
- **Role**: Receipt builder mapping EVM execution results to reth primitive receipts.
- **Key items**: `RethReceiptBuilder`
- **Interactions**: implements `alloy_evm::eth::receipt_builder::ReceiptBuilder` for `TransactionSigned` -> `Receipt`.

#### `test_utils.rs` (feature `test-utils`)
- **Role**: Test helpers for mocking execution: `MockEvmConfig`/`MockExecutor` return predetermined `ExecutionOutcome` results and can be used anywhere a `ConfigureEvm`/`BlockExecutorFactory` is required.
- **Key items**: `MockEvmConfig`, `MockExecutorProvider`, `MockEvmConfig::extend()`

## Key APIs (no snippets)
- `EthEvmConfig`
- `EthBlockAssembler`
- `RethReceiptBuilder`

## Relationships
- **Used by**: Ethereum node wiring (`reth-node-ethereum`) and any executor/payload builder needing Ethereum-specific EVM env derivation and block assembly.
```

## File: crates/ethereum/evm/tests/AGENTS.md
```markdown
# tests

## Purpose
Execution/validation tests for `reth-evm-ethereum`: asserts fork-specific execution behaviours and header-field requirements (e.g. Cancun parent beacon root, system contract predeploy interactions, Prague requests hash, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `execute.rs` - end-to-end execution tests over `BasicBlockExecutor` + `EthEvmConfig`, covering multiple EIPs and fork activation scenarios.
```

## File: crates/ethereum/evm/AGENTS.md
```markdown
# evm

## Purpose
`reth-evm-ethereum` crate: Ethereum-specific EVM configuration for reth, bridging `reth-evm` execution traits to `alloy_evm`/`revm` Ethereum execution contexts and hardfork specs, plus block assembly and receipt building.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthEvmConfig`, block assembler, receipt builder, revm spec helpers, and test-only mocks.
- [x] `tests/` - integration-style execution tests for fork-specific behaviours.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-evm`, `revm`, `reth-chainspec`, and Ethereum forks/primitives).

## Key APIs (no snippets)
- `EthEvmConfig`
- `EthBlockAssembler`
- `RethReceiptBuilder`
```

## File: crates/ethereum/hardforks/src/hardforks/AGENTS.md
```markdown
# hardforks

## Purpose
Hardfork schedule containers and traits for `reth-ethereum-forks`: defines how to represent an ordered set of hardforks, query activation conditions, and derive fork-id/filter information.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - core hardfork set API: `Hardforks` trait and `ChainHardforks` ordered list with insertion/ordering helpers.
  - **Key items**: `Hardforks` (trait), `ChainHardforks`, `ChainHardforks::new()`, `insert()`, `forks_iter()`, `fork_id()`, `fork_filter()`
- `dev.rs` - dev-mode hardfork schedule: activates essentially all Ethereum hardforks at block/timestamp 0 (with Merge TTD = 0).
  - **Key items**: `DEV_HARDFORKS`
```

## File: crates/ethereum/hardforks/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum-forks`: Ethereum fork/hardfork types and utilities used across reth, including ordered hardfork schedules (`ChainHardforks`), fork-id/filter helpers, and human-readable hardfork schedule formatting.

## Contents (one hop)
### Subdirectories
- [x] `hardforks/` - `Hardforks` trait + `ChainHardforks` ordered list and dev-mode hardfork schedule.

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports core hardfork/forkid types from `alloy-hardforks` and `alloy-eip2124`, and exposes local helpers (`DisplayHardforks`, `ChainHardforks`, etc.).
- **Key items**: re-exports `ForkId` types, `DisplayHardforks`, `DEV_HARDFORKS` (via module re-export), and fork condition/types.
- **Interactions**: consumed by chainspec and execution components to gate behaviour by fork activation.

#### `display.rs`
- **Role**: Pretty-print utilities for a hardfork schedule: formats pre-merge / merge / post-merge forks with activation conditions and optional metadata.
- **Key items**: `DisplayHardforks`, `DisplayHardforks::new()`, `DisplayHardforks::with_meta()`

## Key APIs (no snippets)
- `Hardforks` (trait), `ChainHardforks`
- `DisplayHardforks`
- `DEV_HARDFORKS`

## Relationships
- **Builds on**: `alloy-hardforks` and EIP-2124 fork-id primitives.
```

## File: crates/ethereum/hardforks/AGENTS.md
```markdown
# hardforks

## Purpose
`reth-ethereum-forks` crate: Ethereum fork/hardfork types and utilities used across reth (fork-id types, fork activation conditions, ordered hardfork schedules, and display helpers).

## Contents (one hop)
### Subdirectories
- [x] `src/` - fork types + schedules (`ChainHardforks`) and display helpers.

### Files
- `Cargo.toml` - crate manifest (re-exports alloy hardfork + EIP-2124 forkid types; optional `arbitrary`/`serde`).

## Key APIs (no snippets)
- `ChainHardforks`, `Hardforks`
- `DisplayHardforks`
- `DEV_HARDFORKS`
```

## File: crates/ethereum/node/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-node-ethereum`: Ethereum-flavoured node wiring for reth (node type configuration, component builders for pool/network/executor/consensus, payload builder wiring, RPC add-ons, and Engine API payload validation).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: exposes Ethereum node building blocks and re-exports common types (`EthEngineTypes`, `EthEvmConfig`, consensus module, node config, payload builder wiring, engine validator).
- **Key items**: `EthEngineTypes`, `EthEvmConfig`, module `node`, module `payload`, `EthereumEngineValidator`
- **Interactions**: imported by `reth-ethereum-cli`/`reth-node-builder` to instantiate a standard Ethereum node.

#### `node.rs`
- **Role**: Core Ethereum node integration with `reth-node-builder`: defines `EthereumNode` (implements `NodeTypes`), provides `components()` builder preset, and implements builders for executor, txpool, networking, consensus, and engine payload validation; wires RPC add-ons (`EthereumAddOns`) including module selection and middleware hooks.
- **Key items**: `EthereumNode`, `EthereumNode::components()`, `EthereumNode::provider_factory_builder()`, `EthereumAddOns`, `EthereumEthApiBuilder`, `EthereumExecutorBuilder`, `EthereumPoolBuilder`, `EthereumNetworkBuilder`, `EthereumConsensusBuilder`, `EthereumEngineValidatorBuilder`
- **Interactions**: uses `EthBeaconConsensus` + `EthEvmConfig`; builds `EthTransactionPool` with blob store/cache sizing and KZG init; launches networking via `NetworkHandle`; configures RPC modules via `reth_rpc_builder` and per-module wiring.

#### `payload.rs`
- **Role**: Payload component wiring for the Ethereum node: adapts `reth-ethereum-payload-builder` into `reth-node-builder`'s `PayloadBuilderBuilder` interface and maps node config -> `EthereumBuilderConfig`.
- **Key items**: `EthereumPayloadBuilder` (builder), `PayloadBuilderBuilder` impl
- **Interactions**: feeds gas limit / max blobs / extra data from `PayloadBuilderConfig` into the payload builder crate.

#### `engine.rs`
- **Role**: Engine API validation for Ethereum: wraps `EthereumExecutionPayloadValidator` and implements `PayloadValidator` + `EngineApiValidator` to enforce version-specific Engine API object rules (including Cancun/Prague execution requests).
- **Key items**: `EthereumEngineValidator`, `PayloadValidator` impl, `EngineApiValidator` impl
- **Interactions**: used by Engine API server wiring to validate newPayload/forkchoice messages and payload attributes based on `EngineApiMessageVersion`.

#### `evm.rs`
- **Role**: Re-exports Ethereum EVM types used by the node.
- **Key items**: `EthEvmConfig`, `EthEvm`

## Key APIs (no snippets)
- `EthereumNode`
- `EthereumAddOns`
- `EthereumEngineValidator`

## Relationships
- **Builds on**: Ethereum consensus (`reth-ethereum-consensus`), Ethereum EVM config (`reth-evm-ethereum`), payload builder (`reth-ethereum-payload-builder`), and the generic node builder (`reth-node-builder`).
```

## File: crates/ethereum/node/tests/e2e/AGENTS.md
```markdown
# e2e

## Purpose
End-to-end tests for `reth-node-ethereum` using `reth-e2e-test-utils`: spin up real in-process nodes (often with Engine API) and validate Ethereum-specific behaviours across blobs, RPC, P2P sync, reorgs, txpool maintenance, and debug tracing.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test module root that composes the `e2e` test suite.
- `utils.rs` - shared helpers (payload attributes builder, random-tx chain advancement, and related utility helpers).
- `eth.rs` - "node can run" tests: basic tx inclusion, Engine API over IPC variants, graceful engine shutdown persistence, and other core ETH-node flows.
- `blobs.rs` - blob-transaction lifecycle tests (pool injection, payload building/submission, sidecar presence/conversion, and reorg behaviour restoring txs).
- `custom_genesis.rs` - tests for custom genesis parameters (genesis block number boundaries and stage checkpoint initialization).
- `dev.rs` - dev-mode node tests (dev chain config, debug capabilities, payload attribute mutation).
- `p2p.rs` - P2P sync/reorg end-to-end tests (gossip sync, explicit sync-to-head, long reorg and backfill-related reorg scenarios).
- `pool.rs` - txpool maintenance behaviour tests (stale eviction and handling canonical reorg notifications).
- `prestate.rs` - debug tracing regression test: `debug_traceCall` prestate is compared against a captured Geth snapshot.
- `rpc.rs` - RPC surface end-to-end tests (fee history correctness and Flashbots builder submission validation endpoints).

## Relationships
- **Uses**: `reth-e2e-test-utils` to provision nodes and helpers, plus `alloy` RPC providers to drive RPC/Engine API calls.
```

## File: crates/ethereum/node/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for `reth-node-ethereum` focused on node builder wiring, add-ons, and extension points (RPC hooks and ExEx installation).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test module root that composes the `it` test suite.
- `builder.rs` - `NodeBuilder` setup tests: basic configuration, callbacks (`on_component_initialized`, `on_node_started`, `on_rpc_started`), launcher usage, and custom tokio runtime wiring for add-ons.
- `exex.rs` - ExEx integration test: installs a dummy execution extension via `install_exex()` and checks launch wiring.
- `testing.rs` - E2E-style test for the `testing_` RPC namespace: wires only `RethRpcModule::Testing` and validates `testing_buildBlockV1` works end-to-end.
```

## File: crates/ethereum/node/tests/AGENTS.md
```markdown
# tests

## Purpose
Test suites for `reth-node-ethereum`, including end-to-end node/rpc/p2p behaviours and integration tests around node builder wiring.

## Contents (one hop)
### Subdirectories
- [x] `e2e/` - end-to-end tests that spin up nodes and validate Ethereum-specific behaviours (blobs, RPC, P2P sync/reorgs, debug tracing).
- [x] `it/` - integration tests for `NodeBuilder` wiring, RPC add-on configuration, and ExEx installation.
- [x] `assets/` - (skip: test fixtures) genesis JSON used by node tests.

### Files
- (none)
```

## File: crates/ethereum/node/AGENTS.md
```markdown
# node

## Purpose
`reth-node-ethereum` crate: Ethereum node wiring for reth-provides the canonical `EthereumNode` type configuration, component builders (executor/pool/network/consensus), payload builder integration, Engine API validators, and RPC add-on wiring for a full Ethereum node.

## Contents (one hop)
### Subdirectories
- [x] `src/` - node types/builders, Engine API validator, payload builder wiring, and EVM re-exports.
- [x] `tests/` - end-to-end and integration tests for Ethereum node wiring and behaviours.

### Files
- `Cargo.toml` - crate manifest (pulls together Ethereum consensus/EVM/payload/engine primitives and node-builder/RPC subsystems; enables required `revm` features for Ethereum).

## Key APIs (no snippets)
- `EthereumNode`
- `EthereumAddOns`
- `EthereumEngineValidator`
```

## File: crates/ethereum/payload/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum-payload-builder`: a basic Ethereum payload builder that selects transactions from the txpool, executes them with an Ethereum EVM config, and produces `EthBuiltPayload` suitable for serving via the Engine API (including fork-specific blob/request limits and well-formedness checks).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Main payload builder implementation: `EthereumPayloadBuilder` implements `reth_basic_payload_builder::PayloadBuilder` and provides `default_ethereum_payload()` to build a payload from best txs, tracking gas, RLP size limits, blob limits, and fees.
- **Key items**: `EthereumPayloadBuilder`, `default_ethereum_payload()`, `is_better_payload` usage, `BlobSidecars` handling, error mapping to `PayloadBuilderError`
- **Interactions**: pulls best transactions from `reth_transaction_pool`; executes via `reth_evm` block builder (`EthEvmConfig` by default); uses `reth_storage_api` state provider to build against the parent state.
- **Knobs / invariants**: enforces `MAX_RLP_BLOCK_SIZE` under Osaka; applies protocol and user-configured per-block blob limits (EIP-7872).

#### `config.rs`
- **Role**: Builder configuration knobs and helpers.
- **Key items**: `EthereumBuilderConfig`, `calculate_block_gas_limit()`
- **Interactions**: gas limit target is clamped to Ethereum's allowed delta (`GAS_LIMIT_BOUND_DIVISOR`) vs parent gas limit; carries extra-data and optional max-blobs-per-block.

#### `validator.rs`
- **Role**: Execution payload "well-formedness" validator for Engine API ingestion: converts `ExecutionData` into a sealed block and checks fork-specific required fields (Shanghai/Cancun/Prague).
- **Key items**: `EthereumExecutionPayloadValidator`, `ensure_well_formed_payload()`
- **Interactions**: delegates to `reth_payload_validator::{shanghai,cancun,prague}`; verifies payload hash matches derived block hash and checks sidecar fields match transactions.

## Key APIs (no snippets)
- `EthereumPayloadBuilder`
- `EthereumBuilderConfig`
- `EthereumExecutionPayloadValidator`

## Relationships
- **Used by**: Ethereum engine/payload service wiring to produce and validate payloads for `engine_forkchoiceUpdated` / `engine_getPayload*` flows.
```

## File: crates/ethereum/payload/AGENTS.md
```markdown
# payload

## Purpose
`reth-ethereum-payload-builder` crate: Ethereum payload building and validation on top of the txpool API and Ethereum EVM config, producing `EthBuiltPayload` and enforcing fork-specific layout rules for Engine API payloads.

## Contents (one hop)
### Subdirectories
- [x] `src/` - payload builder, configuration, and execution payload validator.

### Files
- `Cargo.toml` - crate manifest (ties together txpool, payload-builder traits, Ethereum EVM config, and payload validators).

## Key APIs (no snippets)
- `EthereumPayloadBuilder`
- `EthereumBuilderConfig`
- `EthereumExecutionPayloadValidator`
```

## File: crates/ethereum/primitives/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum-primitives`: Ethereum-specific primitive types for reth (block/tx/receipt aliases, `NodePrimitives` mapping, and receipt encoding/decoding utilities compatible with typed transactions and multiple serialization targets).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines Ethereum type aliases (transactions/blocks) based on `alloy-consensus` and provides `EthPrimitives` as the `NodePrimitives` implementation for Ethereum.
- **Key items**: `Transaction`, `TransactionSigned`, `PooledTransactionVariant`, `Block`, `BlockBody`, `EthPrimitives`
- **Interactions**: used pervasively throughout Ethereum node components (EVM, engine, payload, RPC) as the canonical primitive set.

#### `receipt.rs`
- **Role**: Ethereum receipt type and encoding/decoding support: typed receipt envelopes (EIP-2718), bloom handling, RLP encoding helpers, and receipt-root calculation helpers.
- **Key items**: `Receipt`/`EthereumReceipt`, `TxTy` (trait alias), `calculate_receipt_root_no_memo()`, EIP-2718 encode/decode impls
- **Interactions**: integrates with `alloy_consensus` receipt traits and reth trie-root helpers (`ordered_trie_root_with_encoder`); optionally supports RPC log types (`rpc` feature).

#### `transaction.rs` (test-only)
- **Role**: Legacy `Transaction`/`TransactionSigned` implementation kept for consistency tests against the newer `alloy-consensus` transaction envelope types.
- **Key items**: `Transaction` enum (legacy/EIP-2930/EIP-1559/EIP-4844/EIP-7702), trait impls for encoding/signing and size.
- **Interactions**: compiled under `#[cfg(test)]` (and some feature-gated codec tests).

## Key APIs (no snippets)
- `EthPrimitives`
- `TransactionSigned`, `Block`, `Receipt`

## Relationships
- **Builds on**: `alloy-consensus`/`alloy-eips` for typed transactions/receipts; implements reth's `NodePrimitives` for Ethereum.
```

## File: crates/ethereum/primitives/AGENTS.md
```markdown
# primitives

## Purpose
`reth-ethereum-primitives` crate: Ethereum primitive types for reth (blocks, typed transactions, receipts) and the `EthPrimitives` `NodePrimitives` mapping used across Ethereum node components.

## Contents (one hop)
### Subdirectories
- [x] `src/` - type aliases + receipt/tx helpers.

### Files
- `Cargo.toml` - crate manifest (feature flags for `serde`, `arbitrary`, and optional codec/rpc integrations).

## Key APIs (no snippets)
- `EthPrimitives`
- `TransactionSigned`, `Block`, `Receipt`
```

## File: crates/ethereum/reth/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-ethereum`: a feature-gated "meta crate" that re-exports commonly used Ethereum-flavoured reth types and subsystems behind a single dependency, for convenience in downstream crates/apps.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - feature-gated re-exports for primitives, chainspec, CLI, consensus, EVM, network, provider/storage, node/engine, trie, and RPC modules.
  - **Key items**: modules `primitives`, `chainspec`, `cli` (feature), `consensus` (feature), `evm` (feature), `network` (feature), `provider`/`storage` (feature), `node`/`engine` (feature), `trie` (feature), `rpc` (feature)
```

## File: crates/ethereum/reth/AGENTS.md
```markdown
# reth

## Purpose
`reth-ethereum` crate: feature-gated meta crate that bundles and re-exports Ethereum-specific reth primitives and (optionally) node subsystems (consensus/evm/network/provider/rpc/etc.) behind a single dependency.

## Contents (one hop)
### Subdirectories
- [x] `src/` - re-export surface.

### Files
- `Cargo.toml` - crate manifest with feature flags (`full`, `node`, `rpc`, `consensus`, `evm`, `provider`, `network`, `cli`, etc.) controlling which subsystems are re-exported.

## Key APIs (no snippets)
- Re-export modules: `primitives`, `chainspec`, `cli`, `consensus`, `evm`, `network`, `provider`, `storage`, `node`, `engine`, `trie`, `rpc`
```

## File: crates/ethereum/AGENTS.md
```markdown
# ethereum

## Purpose
Ethereum-specific crates for reth: Ethereum node wiring, EVM configuration, consensus validation, payload handling, primitives/types, hardfork schedules, and CLI integration for running a canonical Ethereum reth node.

## Contents (one hop)
### Subdirectories
- [x] `cli/` - Ethereum `reth` CLI surface: clap `Cli` type, Ethereum chain spec parsing, tracing/metrics init, and command dispatch using Ethereum node components.
- [x] `consensus/` - Ethereum beacon consensus validator: hardfork-aware header/body checks and post-execution validation (receipts root/bloom, gas used, Prague requests hash).
- [x] `engine-primitives/` - Ethereum Engine API primitives: `EthEngineTypes`/`EthBuiltPayload` and conversions to execution payload envelopes (V1-V5), including blob sidecars and requests handling.
- [x] `evm/` - Ethereum EVM config: `EthEvmConfig` for env derivation and execution, block assembly (roots + fork-gated fields), receipt building, and fork-behaviour tests.
- [x] `hardforks/` - Ethereum fork types/schedules: `ChainHardforks` + `Hardforks` trait, fork-id/filter utilities, dev hardfork schedule, and pretty-printing of fork activation timelines.
- [x] `node/` - Ethereum node wiring: `EthereumNode` type config, component builders (pool/network/executor/consensus), payload builder + Engine API validators, and RPC add-ons for a full Ethereum node.
- [x] `payload/` - Ethereum payload builder: selects best txs from the pool, executes with Ethereum EVM config, enforces gas/RLP/blob limits, produces `EthBuiltPayload`, and validates well-formed Engine API payloads.
- [x] `primitives/` - Ethereum primitives: canonical block/tx/receipt types (`EthPrimitives`) based on alloy envelopes, plus receipt encoding/decoding and receipt-root helpers.
- [x] `reth/` - Feature-gated meta crate: re-exports Ethereum primitives and (optionally) consensus/EVM/node/network/provider/RPC subsystems behind a single dependency.

### Files
- (none)

## Notes
- This directory is an "umbrella" group; details live in each child crate's `AGENTS.md`.
```

## File: crates/etl/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-etl`: a lightweight ETL (extract/transform/load) collector that buffers unsorted key/value pairs, periodically sorts + spills them to temporary files, and later provides a merged sorted iterator (useful for efficient sorted DB inserts and memory management).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - defines the `Collector` type, flush-to-disk logic, and `EtlIter` for merged sorted iteration across ETL files.
  - **Key items**: `Collector`, `Collector::insert()`, `Collector::iter()`, `Collector::clear()`, `EtlIter`

## Relationships
- **Used by**: import/indexing flows that need sorted key order when writing to DB tables (e.g., building hash->number indexes).
```

## File: crates/etl/AGENTS.md
```markdown
# etl

## Purpose
`reth-etl` crate: ETL data collector that sorts/spills key/value pairs to temp files and provides merged sorted iteration for downstream loading (commonly used to optimize DB insert patterns).

## Contents (one hop)
### Subdirectories
- [x] `src/` - `Collector` and merged iterator implementation.

### Files
- `Cargo.toml` - crate manifest (uses `tempfile`, `rayon`, and `reth-db-api` table encoding/compression traits).

## Key APIs (no snippets)
- `Collector`
- `EtlIter`
```

## File: crates/evm/evm/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-evm`: core EVM abstraction layer for reth. Defines the `ConfigureEvm` trait (EVM env/context derivation + executor factory + block assembler), execution traits/utilities for executing blocks and building new blocks, and optional helpers for Engine API payload execution and metrics.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `ConfigureEvm` (the primary EVM configuration trait) and re-exports key building blocks from `alloy_evm` plus local modules (`execute`, `engine`, etc.).
- **Key items**: `ConfigureEvm`, `NextBlockEnvAttributes` (via re-exports), `EvmEnvFor`/type helpers (via `aliases`), `ConfigureEngineEvm` (feature `std`)
- **Interactions**: consumed by node wiring and payload builders to execute blocks and/or build new blocks with chain-specific configs (e.g., Ethereum vs OP).

#### `aliases.rs`
- **Role**: Type aliases and helper trait bounds for working with `ConfigureEvm`-based stacks.
- **Key items**: `EvmFactoryFor`, `SpecFor`, `BlockEnvFor`, `EvmFor`, `EvmEnvFor`, `ExecutionCtxFor`, `InspectorFor`

#### `execute.rs`
- **Role**: Execution/building traits and helpers: defines the `Executor` trait (execute one/batch blocks against `State<DB>`), block assembly input types, and exports execution/output/error types.
- **Key items**: `Executor`, `BlockAssemblerInput`, `BlockAssembler`, `BlockBuilder`, `BasicBlockExecutor`, `BasicBlockBuilder`, `BlockExecutionOutput`, `ExecutionOutcome`
- **Interactions**: bridges `reth_execution_errors` + `reth_execution_types` with `alloy_evm` execution primitives; integrates trie update types (`TrieUpdates`, `HashedPostState`) and provider state access.

#### `engine.rs` (feature `std`)
- **Role**: Engine API payload execution helpers: trait extension for `ConfigureEvm` to derive EVM env/context/tx-iterator from an Engine API payload (and parallelize decode/recovery work).
- **Key items**: `ConfigureEngineEvm`, `ExecutableTxTuple`, `ExecutableTxIterator`

#### `either.rs`
- **Role**: `Either<A,B>` adapter that implements `Executor` by delegating to one of two executor implementations.
- **Key items**: `Executor` impl for `futures_util::future::Either`

#### `noop.rs`
- **Role**: No-op EVM config wrapper for satisfying `ConfigureEvm` bounds in tests/type-level plumbing (panics if invoked).
- **Key items**: `NoopEvmConfig`

#### `metrics.rs` (feature `metrics`)
- **Role**: Execution metrics helpers (gas processed, timings, state load/update histograms).
- **Key items**: `ExecutorMetrics`, `ExecutorMetrics::metered_one()`

#### `test_utils.rs` (tests / feature `test-utils`)
- **Role**: Small helpers for testing `BasicBlockExecutor` state access.
- **Key items**: `BasicBlockExecutor::with_state()`, `with_state_mut()`

## Key APIs (no snippets)
- `ConfigureEvm`
- `Executor` / `BasicBlockExecutor`
- `BlockAssemblerInput`
- `ConfigureEngineEvm` (std)

## Relationships
- **Builds on**: `alloy_evm`/`revm` for execution primitives, and reth's execution error/result crates for error typing and execution outcomes.
```

## File: crates/evm/evm/AGENTS.md
```markdown
# evm

## Purpose
`reth-evm` crate: core EVM abstraction for reth-defines `ConfigureEvm` and execution/building traits used by nodes and payload builders to execute blocks and build new blocks, with optional Engine API helpers and metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `ConfigureEvm`, execution/building helpers, engine extensions, and metrics.

### Files
- `Cargo.toml` - crate manifest (depends on `revm` + `alloy_evm` and reth execution error/type crates; optional metrics support).

## Key APIs (no snippets)
- `ConfigureEvm`
- `Executor`
- `ConfigureEngineEvm` (feature `std`)
```

## File: crates/evm/execution-errors/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-execution-errors`: shared error types used by block execution and state/trie computations (including state root/proof/trie witness error families), and re-exports common EVM execution/validation errors.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports `alloy_evm::block` execution/validation error types and exposes the trie/state-root related error module.
- **Key items**: `BlockExecutionError`, `BlockValidationError`, `InternalBlockExecutionError`

#### `trie.rs`
- **Role**: Error types for state root, storage root, state proofs, sparse trie operations, and trie witness building.
- **Key items**: `StateRootError`, `StorageRootError`, `StateProofError`, `SparseStateTrieError`/`SparseStateTrieErrorKind`, `SparseTrieError`/`SparseTrieErrorKind`, `TrieWitnessError`
- **Interactions**: bridges errors into `reth_storage_errors::{DatabaseError, ProviderError}` where applicable.

## Key APIs (no snippets)
- Error families: `StateRootError`, `TrieWitnessError`, `SparseStateTrieError`
```

## File: crates/evm/execution-errors/AGENTS.md
```markdown
# execution-errors

## Purpose
`reth-execution-errors` crate: execution-related error types used across reth's execution pipeline, including EVM block execution/validation errors and trie/state-root/proof/witness error families.

## Contents (one hop)
### Subdirectories
- [x] `src/` - error definitions and re-exports.

### Files
- `Cargo.toml` - crate manifest (depends on `alloy-evm`, `reth-storage-errors`, and trie utilities).

## Key APIs (no snippets)
- `BlockExecutionError`, `BlockValidationError`
- `StateRootError`, `TrieWitnessError`
```

## File: crates/evm/execution-types/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-execution-types`: shared data types for EVM/block execution outputs in reth, including per-block execution results with bundle state, aggregated multi-block execution outcomes, and `Chain` containers that pair blocks with their derived execution/trie state.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports the execution result/output/outcome types and the `Chain` container.
- **Key items**: modules `execute`, `execution_outcome`, `chain`, optional `serde_bincode_compat`

#### `execute.rs`
- **Role**: Defines `BlockExecutionOutput<T>` (execution result + bundle state) and re-exports `alloy_evm::block::BlockExecutionResult`.
- **Key items**: `BlockExecutionResult`, `BlockExecutionOutput`

#### `execution_outcome.rs`
- **Role**: Aggregated execution outcome representation over one or more blocks: bundle state (with reverts), per-block receipts, and per-block EIP-7685 requests, plus helpers for querying and reverting/splitting outcomes.
- **Key items**: `ExecutionOutcome`, `ExecutionOutcome::single()`, `ExecutionOutcome::from_blocks()`, `ExecutionOutcome::revert_to()`, `ExecutionOutcome::hash_state_slow()`
- **Interactions**: uses `revm::database::BundleState` and trie helpers (`HashedPostState`) to represent post-state and compute hashed representations.

#### `chain.rs`
- **Role**: `Chain` container: holds an ordered map of recovered blocks plus their execution outcome and per-block trie updates/hashed post-state, used in fork-tree / blockchain-tree contexts.
- **Key items**: `Chain`, `Chain::new()`, `Chain::from_block()`, `execution_outcome_at_block()`, iterators over blocks/receipts, helpers to find tx+receipt metadata, and fork-block metadata.

## Key APIs (no snippets)
- `BlockExecutionResult`, `BlockExecutionOutput`
- `ExecutionOutcome`
- `Chain`
```

## File: crates/evm/execution-types/AGENTS.md
```markdown
# execution-types

## Purpose
`reth-execution-types` crate: shared types for representing EVM block execution results and outcomes (per-block `BlockExecutionResult`/`BlockExecutionOutput`, multi-block `ExecutionOutcome`, and `Chain` containers used by fork/tree logic).

## Contents (one hop)
### Subdirectories
- [x] `src/` - execution output/outcome types and the `Chain` container.

### Files
- `Cargo.toml` - crate manifest (ties together `revm` bundle state, trie helpers, and primitive types; optional serde/bincode compatibility).

## Key APIs (no snippets)
- `BlockExecutionResult`, `BlockExecutionOutput`
- `ExecutionOutcome`
- `Chain`
```

## File: crates/evm/AGENTS.md
```markdown
# evm

## Purpose
EVM/execution subsystem crates: core EVM execution traits and adapters (`reth-evm`), execution error types (`reth-execution-errors`), and execution outcome/result types (`reth-execution-types`).

## Contents (one hop)
### Subdirectories
- [x] `evm/` - Core EVM abstraction: `ConfigureEvm` + execution/building traits for executing blocks and building new blocks, with optional Engine API helpers and metrics.
- [x] `execution-errors/` - Execution error types: re-exports block execution/validation errors and defines state root/proof/sparse trie/trie witness error families.
- [x] `execution-types/` - Execution result types: `BlockExecutionResult`/`BlockExecutionOutput`, multi-block `ExecutionOutcome`, and `Chain` containers pairing blocks with derived execution/trie state.

### Files
- (none)
```

## File: crates/exex/exex/src/backfill/AGENTS.md
```markdown
# backfill

## Purpose
Backfill utilities for ExEx notifications: execute historical block ranges (single-block or batched) using an `EvmConfig` + provider, producing `Chain`/execution outputs to "catch up" an ExEx to a target head.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - module wiring and public re-exports for backfill jobs and factories.
  - **Key items**: `BackfillJobFactory`, `BackfillJob`, `SingleBlockBackfillJob`, `StreamBackfillJob`
- `factory.rs` - backfill job factory that captures `evm_config`, `provider`, prune modes, thresholds, and stream parallelism.
  - **Key items**: `BackfillJobFactory::new()`, `backfill()`, `new_from_components()`
- `job.rs` - core backfill iterators: `BackfillJob` (batched execution yielding `Chain`) and `SingleBlockBackfillJob` (yields `(RecoveredBlock, BlockExecutionOutput)`), with threshold-driven batching.
  - **Key items**: `BackfillJob`, `BackfillJob::into_stream()`, `SingleBlockBackfillJob`, `execute_block()`
- `stream.rs` - async stream adapter that executes backfill jobs in parallel using `spawn_blocking`, while yielding results in-order.
  - **Key items**: `StreamBackfillJob`, `with_parallelism()`, `with_batch_size()`
- `test_utils.rs` (tests) - helpers for constructing test chains/specs and executing blocks into a test provider factory.
```

## File: crates/exex/exex/src/wal/AGENTS.md
```markdown
# wal

## Purpose
Write-ahead log (WAL) for ExEx notifications: persists notifications to disk (MessagePack) and maintains an in-memory block cache for efficient lookup and finalization to bound growth.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - `Wal` and `WalHandle` facade: commit/finalize/iterate notifications and cached lookup by committed block hash.
  - **Key items**: `Wal`, `Wal::commit()`, `Wal::finalize()`, `Wal::iter_notifications()`, `WalHandle`
- `storage.rs` - file-backed storage: each notification is a `{id}.wal` file encoded via `rmp-serde` using `serde_bincode_compat` wrapper types.
  - **Key items**: `Storage`, `files_range()`, `read_notification()`, `write_notification()`, `remove_notifications()`
- `cache.rs` - in-memory cache of notification max blocks and committed block hash->(file id, metadata) mapping to avoid scanning storage on lookups/finalization.
  - **Key items**: `BlockCache`, `insert_notification_blocks_with_file_id()`, `remove_before()`
- `metrics.rs` - WAL metrics (bytes, notification counts, committed block heights).
  - **Key items**: `Metrics`
- `error.rs` - WAL error types and result alias.
  - **Key items**: `WalError`, `WalResult<T>`
```

## File: crates/exex/exex/src/AGENTS.md
```markdown
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
```

## File: crates/exex/exex/AGENTS.md
```markdown
# exex

## Purpose
`reth-exex` crate: execution extensions runtime for reth-hosts long-running ExEx tasks that derive state from canonical execution notifications, report pruning-safe progress, supports catch-up via backfill, and persists notifications with a write-ahead log (WAL).

## Contents (one hop)
### Subdirectories
- [x] `src/` - ExEx manager, context APIs, notification streams/backfill, and WAL implementation.
- [x] `test-data/` - (skip: test fixtures) serialized `.wal` files used to validate WAL decoding/compatibility.

### Files
- `Cargo.toml` - crate manifest (integrates node components, provider/evm APIs, pruning types, tasks/tracing/metrics).
  - **Key items**: `description = "Execution extensions for Reth"`

## Key APIs (no snippets)
- `ExExContext`, `ExExEvent`
- `ExExNotifications`
- `ExExManager`
- `Wal`
```

## File: crates/exex/test-utils/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-exex-test-utils`: utilities for testing ExEx integrations, including a lightweight `TestNode` configuration, helpers to construct `ExExContext` instances, and harness types to drive notifications/events during tests.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - test node and harness wiring: provides `TestNode`, pool/executor/consensus builders, `TestExExContext`/`TestExExHandle`, and helpers to create an ExEx context with an ephemeral provider factory + WAL.
  - **Key items**: `TestNode`, `TestPoolBuilder`, `TestExecutorBuilder`, `TestConsensusBuilder`, `TestExExHandle`, `test_exex_context_with_chain_spec()`
```

## File: crates/exex/test-utils/AGENTS.md
```markdown
# test-utils

## Purpose
`reth-exex-test-utils` crate: helpers for writing ExEx tests-builds a minimal test node configuration and provides utilities to create and drive `ExExContext` instances with in-memory/on-disk test infrastructure.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `TestNode` + ExEx context harness utilities.

### Files
- `Cargo.toml` - crate manifest (pulls in test-utils feature sets across provider/db/network/node-builder and Ethereum node wiring).

## Key APIs (no snippets)
- `test_exex_context_with_chain_spec()`
- `TestExExHandle`
```

## File: crates/exex/types/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-exex-types`: shared, dependency-light types for ExEx integration-notifications sent to ExExes (`ExExNotification`), ExEx progress/head markers, and the aggregated "finished height" signal used for pruning decisions.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring + exports; optional `serde_bincode_compat` module for always-serializing wrapper types.
  - **Key items**: `ExExNotification`, `ExExHead`, `FinishedExExHeight`
- `notification.rs` - ExEx notification enum (commit/reorg/revert) wrapping `Arc<Chain<N>>`, helpers to access committed/reverted chains and invert notifications; conversion from `CanonStateNotification`.
  - **Key items**: `ExExNotification`, `committed_chain()`, `reverted_chain()`, `into_inverted()`
- `head.rs` - ExEx head marker (highest processed host block).
  - **Key items**: `ExExHead`
- `finished_height.rs` - aggregated finished height across all ExExes (NoExExs/NotReady/Height).
  - **Key items**: `FinishedExExHeight`
```

## File: crates/exex/types/AGENTS.md
```markdown
# types

## Purpose
`reth-exex-types` crate: shared types for ExEx usage (notifications, ExEx head/progress, and aggregated finished height for pruning), with optional serde + bincode-compat wrappers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `ExExNotification`, `ExExHead`, `FinishedExExHeight`.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-chain-state` and `reth-execution-types` for `CanonStateNotification`/`Chain` integration).

## Key APIs (no snippets)
- `ExExNotification`
- `ExExHead`
- `FinishedExExHeight`
```

## File: crates/exex/AGENTS.md
```markdown
# exex

## Purpose
Execution Extensions ("ExEx") subsystem crates: core ExEx runtime/wiring (`reth-exex`), shared ExEx types (`reth-exex-types`), and testing utilities (`reth-exex-test-utils`).

## Contents (one hop)
### Subdirectories
- [x] `exex/` - Core ExEx runtime: ExEx manager + contexts, notification streams with backfill, pruning progress events (`FinishedHeight`), and notification WAL persistence.
- [x] `types/` - Shared ExEx types: `ExExNotification` (commit/reorg/revert), `ExExHead`, and aggregated `FinishedExExHeight` for pruning safety decisions.
- [x] `test-utils/` - ExEx testing helpers: minimal test node wiring and harness utilities to construct and drive `ExExContext` instances in tests.

### Files
- (none)
```

## File: crates/fs-util/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-fs-util`: thin wrappers around `std::fs` operations that enrich errors with path context and add common helpers (e.g. JSON read/write, atomic writes, fsync).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `FsPathError` (path-aware error enum) plus wrapper functions like `open()`, `read()`, `write()`, directory ops, metadata/fsync helpers, and JSON helpers.
  - **Key items**: `FsPathError`, `FsPathError::*` constructors, `read_to_string()`, `atomic_write_file()`
```

## File: crates/fs-util/AGENTS.md
```markdown
# fs-util

## Purpose
`reth-fs-util` crate: filesystem utility wrappers that attach path context to errors and provide shared helpers (including JSON read/write and atomic file writes).

## Contents (one hop)
### Subdirectories
- [x] `src/` - `FsPathError` and fs wrapper helpers.

### Files
- `Cargo.toml` - crate manifest (serde + serde_json for JSON helpers).

## Key APIs (no snippets)
- `FsPathError`
```

## File: crates/metrics/src/common/AGENTS.md
```markdown
# common

## Purpose
Common metric utilities for `reth-metrics`, primarily metered wrappers around tokio mpsc channels to expose counters for sends/receives and errors.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - exports common utilities (currently the `mpsc` module).
- `mpsc.rs` - metered tokio mpsc wrappers (`MeteredSender`/`MeteredReceiver` and unbounded variants), plus helpers to construct metered channels.
  - **Key items**: `metered_channel()`, `metered_unbounded_channel()`, `MeteredSender`, `MeteredReceiver`, `UnboundedMeteredSender`, `UnboundedMeteredReceiver`
```

## File: crates/metrics/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-metrics`: small utilities for metrics usage across reth, including the `Metrics` derive macro re-export and (feature-gated) common helpers like metered tokio channels.

## Contents (one hop)
### Subdirectories
- [x] `common/` - metered tokio mpsc channel wrappers (feature `common`).

### Files
- `lib.rs` - re-exports `metrics_derive::Metrics` and the core `metrics` crate; conditionally exposes `common` helpers.
  - **Key items**: `Metrics` derive macro, `metrics` re-export
```

## File: crates/metrics/AGENTS.md
```markdown
# metrics

## Purpose
`reth-metrics` crate: metrics utilities used across reth-re-exports the `Metrics` derive macro and provides optional common helpers (e.g., metered tokio mpsc channels).

## Contents (one hop)
### Subdirectories
- [x] `src/` - `Metrics` re-export and common utilities.

### Files
- `Cargo.toml` - crate manifest (feature `common` pulls in tokio/futures utilities for metered channels).

## Key APIs (no snippets)
- `Metrics` (derive macro re-export)
```

## File: crates/net/banlist/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-net-banlist`: simple ban/allow-list utilities for networking-maintains banned peer IDs and IPs (optionally with expiry) and provides an `IpFilter` to restrict communication to allowed CIDR ranges.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `BanList` (ban/unban, expiry eviction, global-IP gating) and `IpFilter` (CIDR allow-list parsing and checks).
  - **Key items**: `BanList`, `BanList::ban_ip_with()`, `evict()`, `is_banned()`, `IpFilter`, `IpFilter::from_cidr_string()`
```

## File: crates/net/banlist/AGENTS.md
```markdown
# banlist

## Purpose
`reth-net-banlist` crate: supports banning peers/IPs (with optional expiry) and filtering allowed IP ranges (CIDR) for network communications.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `BanList` and `IpFilter` implementations.

### Files
- `Cargo.toml` - crate manifest (uses `ipnet` and `alloy-primitives` for peer IDs).

## Key APIs (no snippets)
- `BanList`
- `IpFilter`
```

## File: crates/net/discv4/src/AGENTS.md
```markdown
# src

## Purpose
Implements Discovery v4 (devp2p discv4) for reth: maintains a Kademlia-like routing table over UDP, performs PING/PONG bonding and `FINDNODE` lookups, and can infer an external/public IP in NAT environments (with optional periodic re-resolution).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Main protocol/service implementation: `Discv4` frontend + `Discv4Service` driving UDP I/O, routing table maintenance, lookups, bonding, and update/event streams.
- **Key items**: `Discv4`, `Discv4Service`, `DiscoveryUpdate` (routing-table change notifications), constants like `DEFAULT_DISCOVERY_ADDRESS`
- **Interactions**: Uses `reth-net-nat` for public IP resolution; uses `reth-network-peers::NodeRecord` as peer address primitive.

#### `config.rs`
- **Role**: Configuration surface for discv4 performance and behavior (timeouts, buffers, bootstrap nodes, EIP-868 behavior, NAT resolver + interval).
- **Key items**: `Discv4Config`, `Discv4ConfigBuilder`, `resolve_external_ip_interval()` (builds `ResolveNatInterval`)

#### `proto.rs`
- **Role**: Wire protocol types and encode/decode logic for UDP packets (message IDs, RLP payload encoding, secp256k1 recoverable signatures, packet hash verification).
- **Key items**: `Message`, `MessageId`, `Packet`, `Ping`/`Pong`, `FindNode`, `Neighbours`, `EnrRequest`/`EnrResponse`, `NodeEndpoint`

#### `node.rs`
- **Role**: Adapters for using `PeerId`/node IDs in kbucket tables (compute Kademlia key via keccak hash).
- **Key items**: `kad_key()`, `NodeKey`

#### `table.rs`
- **Role**: Small helper tables for tracking per-peer state needed by the service (e.g. last PONG per node/IP).
- **Key items**: `PongTable`

#### `error.rs`
- **Role**: Error types for decoding packets and interacting with the service via channels.
- **Key items**: `DecodePacketError`, `Discv4Error`

#### `test_utils.rs`
- **Role**: Feature-gated testing utilities, including a mock discovery peer and helpers for constructing test `Discv4` instances.
- **Key items**: `MockDiscovery`, `MockCommand`, `MockEvent`, `create_discv4()`

## Key APIs (no snippets)
- `Discv4`, `Discv4Service`
- `Discv4Config`, `Discv4ConfigBuilder`
- `Message`/`Packet` (protocol encoding/decoding)
```

## File: crates/net/discv4/AGENTS.md
```markdown
# discv4

## Purpose
`reth-discv4` crate: Discovery v4 implementation for Ethereum devp2p networking-UDP-based peer discovery with a Kademlia-like routing table, node bonding via PING/PONG, recursive `FINDNODE` lookups, and optional external/public IP detection for NAT scenarios.

## Contents (one hop)
### Subdirectories
- [x] `src/` - core discv4 protocol, config, wire encoding/decoding, and test utilities.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-network-peers`, `reth-net-nat`, `discv5` kbucket types; optional serde and test-utils feature).
- `README.md` - crate overview/documentation.

## Key APIs (no snippets)
- `Discv4`, `Discv4Service`
- `Discv4Config`, `Discv4ConfigBuilder`
```

## File: crates/net/discv5/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-discv5`: a reth-oriented wrapper around `sigp/discv5` that bootstraps discovery v5, builds/updates the local ENR (including EIP-868 fork IDs), filters discovered peers, and exposes discovered peers as `NodeRecord` + optional `ForkId` for upstream networking.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `config.rs`
- **Role**: Configuration surface for starting discv5: wraps `discv5::Config`, manages boot nodes (signed ENRs or unsigned "enode" multiaddrs), sets fork kv-pairs and additional ENR kv-pairs, and defines lookup/bootstrapping intervals.
- **Key items**: `Config`, `ConfigBuilder`, `BootNode`, `DEFAULT_DISCOVERY_V5_LISTEN_CONFIG`, `DEFAULT_SECONDS_LOOKUP_INTERVAL`, `amend_listen_config_wrt_rlpx()`
- **Interactions**: Feeds `Config` into `Discv5::start()` / `build_local_enr()` in `lib.rs`.
- **Knobs / invariants**: ENR is limited to one IP per IP-version; RLPx IP can overwrite discv5 listen IP of same version.

### `enr.rs`
- **Role**: Bridges identity types between discv5 and reth's v4-style peer IDs: converts ENRs/peer IDs between discv5 node IDs, libp2p peer IDs, and `reth-network-peers::PeerId`.
- **Key items**: `enr_to_discv4_id()`, `discv4_id_to_discv5_id()`, `discv4_id_to_multiaddr_id()`, `EnrCombinedKeyWrapper`
- **Interactions**: Used by `lib.rs` for banning (`discv4_id_to_discv5_id`) and for producing `NodeRecord`s from ENRs (`enr_to_discv4_id`).

### `error.rs`
- **Role**: Error taxonomy for operations around the underlying `discv5::Discv5` node (initialization, listen config mismatches, fork ID decoding, and node/peer compatibility).
- **Key items**: `Error` (variants: `AddNodeFailed`, `IncompatibleKeyType`, `ForkMissing`, `ListenConfigMisconfigured`, `Discv5Error`)

### `filter.rs`
- **Role**: Filtering predicates applied to discovered peers' ENRs (e.g. disallowing specific network stack keys such as `eth2`) to decide whether to pass peers to RLPx or drop them.
- **Key items**: `FilterOutcome`, `MustNotIncludeKeys`, `MustIncludeKey`, `MustNotIncludeKeys::filter()`, `add_disallowed_keys()`
- **Interactions**: Called by `Discv5::filter_discovered_peer()` in `lib.rs` during event processing.

### `lib.rs`
- **Role**: Main wrapper and orchestration: starts discv5, constructs local ENR (fork kv-pair + sockets), bootstraps with configured peers, spawns background kbucket maintenance, processes discv5 events into `DiscoveredPeer` outputs, and tracks discovery metrics.
- **Key items**: `Discv5`, `DiscoveredPeer`, `Discv5::start()`, `on_discv5_update()`, `on_discovered_peer()`, `build_local_enr()`, `bootstrap()`, `spawn_populate_kbuckets_bg()`, `try_into_reachable()`, `get_fork_id()`
- **Interactions**: Consumes `Config` from `config.rs`; uses `MustNotIncludeKeys` from `filter.rs`; uses ENR conversion helpers from `enr.rs`; updates `Discv5Metrics`.
- **Knobs / invariants**: Handles `UnverifiableEnr` events by deriving a reachable `NodeRecord` from the sender socket (compatibility with peers advertising unreachable ENR sockets).

### `metrics.rs`
- **Role**: Metrics collection for discovery: tracks sessions, kbucket counts/insertions, unverifiable ENRs, and frequencies of advertised network stack IDs.
- **Key items**: `Discv5Metrics`, `DiscoveredPeersMetrics`, `AdvertisedChainMetrics`, `increment_once_by_network_type()`

### `network_stack_id.rs`
- **Role**: Defines ENR kv-pair keys used to label which network stack a node belongs to (Ethereum EL/CL, Optimism EL/CL) and selects keys from a chain spec.
- **Key items**: `NetworkStackId::{ETH, ETH2, OPEL, OPSTACK}`, `NetworkStackId::id()`

## End-to-end flow (high level)
- Build a `Config` via `Config::builder(rlpx_tcp_socket)` and set discovery listen config, boot nodes, fork kv-pair, and lookup intervals.
- `Discv5::start()` derives the local ENR + a backwards-compatible `NodeRecord` with `build_local_enr()`.
- Start underlying `discv5::Discv5`, obtain its event stream, and add boot nodes (direct ENR insert or ENR request for unsigned "enode" multiaddrs).
- Spawn background lookup tasks to populate kbuckets over time (`spawn_populate_kbuckets_bg`).
- Consume `discv5::Event`s and pass them through `on_discv5_update()` / `on_discovered_peer()`.
- For discovered peers, attempt to produce a reachable `NodeRecord`, apply `MustNotIncludeKeys` filtering, and extract an optional `ForkId`.
- Emit `DiscoveredPeer { node_record, fork_id }` to upstream networking and record metrics throughout.

## Key APIs (no snippets)
- **Types**: `Discv5`, `DiscoveredPeer`, `Config`, `ConfigBuilder`, `BootNode`
- **Functions**: `build_local_enr()`, `bootstrap()`, `spawn_populate_kbuckets_bg()`
- **Filtering**: `MustNotIncludeKeys`, `FilterOutcome`
```

## File: crates/net/discv5/AGENTS.md
```markdown
# discv5

## Purpose
`reth-discv5` crate: discovery v5 integration for reth, wrapping `sigp/discv5` with reth-friendly configuration, ENR construction (fork/network-stack keys), discovered-peer filtering, and metrics to feed upstream networking with reachable `NodeRecord`s.

## Contents (one hop)
### Subdirectories
- [x] `src/` - discv5 wrapper, config, ENR conversions, filters, and metrics.

### Files
- `Cargo.toml` - crate manifest.
  - **Key items**: `reth-network-peers` (secp256k1), `discv5` (libp2p), `reth-metrics`, `reth-chainspec`
- `README.md` - crate documentation/overview.
  - **Key items**: n/a

## Key APIs (no snippets)
- `Discv5`
- `Config` / `ConfigBuilder`
- `DiscoveredPeer`

## Relationships
- **Depends on**: `reth-network-peers` (peer IDs + `NodeRecord`), `reth-ethereum-forks` (fork IDs), `reth-chainspec` (network selection), `reth-metrics` (metrics plumbing), `discv5` (protocol implementation).
- **Used by**: higher-level network/discovery orchestration in `reth/crates/net/network*` (via the `Discv5` wrapper and its event processing).
```

## File: crates/net/dns/src/AGENTS.md
```markdown
# src

## Purpose
Implements EIP-1459 "Node Discovery via DNS" for reth: fetches and verifies DNS ENR trees (`enrtree-root`, branches, links, and ENR leaves), rate-limits TXT lookups, incrementally syncs trees, and emits discovered peers as `NodeRecord` + optional `ForkId`.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `config.rs`
- **Role**: Configuration for `DnsDiscoveryService` runtime behavior (timeouts, rate limits, cache sizing, periodic recheck interval, and optional bootstrap networks).
- **Key items**: `DnsDiscoveryConfig`, `lookup_timeout`, `max_requests_per_sec`, `recheck_interval`, `dns_record_cache_limit`, `bootstrap_dns_networks`
- **Knobs / invariants**: DNS TXT lookups are rate-limited; cache limit is enforced by an LRU keyed by record hash/domain.

### `error.rs`
- **Role**: Error types for parsing DNS tree entries and for lookup failures (timeouts, missing entries, root verification).
- **Key items**: `ParseDnsEntryError`, `LookupError`, `ParseEntryResult`, `LookupResult`

### `lib.rs`
- **Role**: Main service/handle: drives tree syncing, maintains per-tree state, executes lookups through `QueryPool`, caches resolved records, and notifies subscribers with `DnsNodeRecordUpdate`.
- **Key items**: `DnsDiscoveryService`, `DnsDiscoveryHandle`, `DnsNodeRecordUpdate`, `DnsDiscoveryEvent`, `sync_tree()`/`sync_tree_with_link()`, `node_record_stream()`, `convert_enr_node_record()`
- **Interactions**: Uses `QueryPool` (`query.rs`) for rate-limited DNS queries; uses `SyncTree` (`sync.rs`) to produce sync actions; parses DNS TXT records into `DnsEntry` (`tree.rs`).
- **Knobs / invariants**: Root entries are verified against the link's public key before syncing children; node records are only emitted when ENR has required socket fields.

### `query.rs`
- **Role**: Aggregates and drives DNS queries to completion with rate limiting and lookup timeouts (root lookups vs entry lookups).
- **Key items**: `QueryPool`, `QueryOutcome`, `ResolveRootResult`, `ResolveEntryResult`, `resolve_root()`, `resolve_entry()`
- **Interactions**: Called by `DnsDiscoveryService` to schedule and poll DNS lookups; delegates I/O to `Resolver`.

### `resolver.rs`
- **Role**: DNS lookup abstraction: `Resolver` trait for TXT queries, plus concrete implementations for real DNS (`DnsResolver`) and in-memory testing (`MapResolver`).
- **Key items**: `Resolver`, `DnsResolver`, `TokioResolver`, `MapResolver`

### `sync.rs`
- **Role**: Tree-sync state machine: maintains which roots/branches/links/ENRs remain to be fetched and emits `SyncAction`s that drive the next DNS lookups.
- **Key items**: `SyncTree`, `SyncAction`, `ResolveKind`, `SyncTree::poll()`, `SyncTree::update_root()`
- **Knobs / invariants**: Periodically triggers root refresh after `recheck_interval`; resync strategy depends on which root hashes changed (ENR root vs link root).

### `tree.rs`
- **Role**: EIP-1459 DNS record structure parsing and verification: represents and parses root/link/branch/node entries and verifies signed roots.
- **Key items**: `DnsEntry`, `TreeRootEntry`, `BranchEntry`, `LinkEntry`, `NodeEntry`, `TreeRootEntry::verify()`
- **Knobs / invariants**: Root signatures are verified over the unsigned root content; branch child hashes must meet size/format constraints.

## End-to-end flow (high level)
- Create a `DnsResolver` (system DNS config) or another `Resolver` implementation.
- Construct `DnsDiscoveryService::new_pair(resolver, DnsDiscoveryConfig)` or `DnsDiscoveryService::new(...)`.
- Start syncing one or more `enrtree://...@domain` links via `DnsDiscoveryHandle::sync_tree()` / `sync_tree_with_link()`.
- Service schedules a root TXT lookup and verifies the root's signature before accepting it.
- `SyncTree` emits `SyncAction`s to fetch link roots and ENR roots, expanding branches into child hashes.
- `QueryPool` executes TXT queries with request rate limiting and per-lookup timeouts.
- Resolved ENR leaf records are converted into `DnsNodeRecordUpdate` (node `NodeRecord` + optional `ForkId`) and broadcast to subscribers.
- Periodically rechecks roots and re-syncs subtrees if the sequence number/root hashes changed.

## Key APIs (no snippets)
- **Service**: `DnsDiscoveryService`, `DnsDiscoveryHandle`, `DnsNodeRecordUpdate`
- **Tree model**: `DnsEntry`, `TreeRootEntry`, `LinkEntry`, `BranchEntry`
- **I/O**: `Resolver`, `DnsResolver`
```

## File: crates/net/dns/AGENTS.md
```markdown
# dns

## Purpose
`reth-dns-discovery` crate: EIP-1459 DNS-based node discovery-resolves and verifies ENR tree records via DNS TXT lookups, rate-limits queries, incrementally syncs trees, and emits discovered peers as `NodeRecord` updates.

## Contents (one hop)
### Subdirectories
- [x] `src/` - DNS tree parsing/verification, query pool + resolver abstraction, and discovery service.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support).
  - **Key items**: feature `serde`; deps `hickory-resolver`, `reth-network-peers`, `reth-tokio-util` (ratelimit)

## Key APIs (no snippets)
- `DnsDiscoveryService`, `DnsDiscoveryHandle`
- `DnsNodeRecordUpdate`
- `DnsDiscoveryConfig`
```

## File: crates/net/downloaders/src/bodies/AGENTS.md
```markdown
# bodies

## Purpose
Implements block body downloader algorithms for `reth-downloaders`: a concurrent, range-based bodies downloader (`BodiesDownloader`) plus helpers for scheduling/validating requests, buffering/reordering responses, and optionally driving the downloader on a separate task.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for body downloaders: exports the main concurrent downloader, a noop implementation, a task-driven wrapper, and optional test helpers; keeps request queue/future internal.
- **Key items**: `bodies`, `noop`, `task`, `queue` (private), `request` (private), `test_utils` (cfg)

### `bodies.rs`
- **Role**: Main concurrent bodies downloader implementation: batches header-derived requests, issues multiple in-flight peer requests, buffers/reorders responses to emit bodies in block-number order, and applies memory/backpressure limits while streaming results.
- **Key items**: `BodiesDownloader`, `BodiesDownloaderBuilder`, `set_download_range()`, `into_task()`, `into_task_with()`
- **Interactions**: Uses `BodiesRequestQueue` (`queue.rs`) + `BodiesRequestFuture` (`request.rs`) for request concurrency; records `BodyDownloaderMetrics` (`crate::metrics`); can be driven as a `TaskDownloader` (`task.rs`).
- **Knobs / invariants**: Respects `request_limit`, `stream_batch_size`, `concurrent_requests_range`, `max_buffered_blocks_size_bytes`; emits in-order batches even if requests complete out-of-order; clears state on fatal errors and on non-consecutive range changes.

### `request.rs`
- **Role**: Request/validation future for a single header batch: repeatedly requests missing bodies until fulfilled, filters empty headers, validates bodies against consensus, and penalizes peers on malformed responses before retrying.
- **Key items**: `BodiesRequestFuture`, `with_headers()`, `on_block_response()`, `try_buffer_blocks()`, `submit_request()`
- **Interactions**: Driven by `BodiesRequestQueue` (`queue.rs`); uses `BodiesClient` for network requests and `Consensus::validate_block_pre_execution`; updates `BodyDownloaderMetrics` + per-response `ResponseMetrics`.
- **Knobs / invariants**: Assumes peers return bodies in request order; empty/overlong responses and validation failures trigger error accounting + peer penalty + high-priority retry.

### `queue.rs`
- **Role**: In-flight request queue wrapper over `FuturesUnordered` that tracks the last requested block number and exposes a `Stream` of request results.
- **Key items**: `BodiesRequestQueue`, `push_new_request()`, `last_requested_block_number`, `clear()`
- **Interactions**: Owned/polled by `BodiesDownloader` (`bodies.rs`); carries `BodyDownloaderMetrics` for queue-level accounting.

### `task.rs`
- **Role**: Task wrapper that runs any `BodyDownloader` on a spawned task and forwards results over channels, allowing download ranges to be updated asynchronously.
- **Key items**: `TaskDownloader`, `BODIES_TASK_BUFFER_SIZE`, `spawn()`, `spawn_with()`
- **Interactions**: `BodiesDownloader::into_task*()` returns this wrapper; uses `reth_tasks::TaskSpawner` + Tokio channels/streams to bridge caller <-> background downloader.

### `noop.rs`
- **Role**: No-op `BodyDownloader` implementation used for unwind-only pipelines or configurations that must satisfy a downloader interface without doing work.
- **Key items**: `NoopBodiesDownloader`

### `test_utils.rs`
- **Role**: Test-only helpers for composing expected body responses and inserting headers into a test provider/static-file writer.
- **Key items**: `zip_blocks()`, `create_raw_bodies()`, `insert_headers()`
- **Interactions**: Used by tests in `bodies.rs`, `request.rs`, `task.rs` and crate-level test utilities.

## End-to-end flow (high level)
- Construct a downloader via `BodiesDownloaderBuilder` (often from `BodiesConfig`) with a `BodiesClient`, `Consensus`, and `HeaderProvider`.
- Set an inclusive block range with `set_download_range()`, which may append consecutive ranges or reset internal state for non-consecutive changes.
- Query headers from the provider and form variable-length batches based on non-empty count and `stream_batch_size`.
- For each header batch, push a `BodiesRequestFuture` into `BodiesRequestQueue`, keeping multiple requests in flight up to a peer-aware concurrency limit.
- Each `BodiesRequestFuture` requests bodies by header hashes, filters empty headers, validates bodies via `Consensus`, and retries/penalizes peers on bad responses.
- Completed responses are buffered and reordered so the stream only yields bodies in block-number order.
- Once enough contiguous bodies are queued, yield a batch (up to `stream_batch_size`) and apply backpressure using queued-count and buffered-bytes limits.
- Optionally, drive the downloader on a background task via `TaskDownloader`, sending range updates over a channel and receiving streamed results.

## Key APIs (no snippets)
- **Downloaders**: `BodiesDownloader`, `BodiesDownloaderBuilder`, `TaskDownloader`, `NoopBodiesDownloader`
- **Internal plumbing**: `BodiesRequestFuture`, `BodiesRequestQueue`
```

## File: crates/net/downloaders/src/headers/AGENTS.md
```markdown
# headers

## Purpose
Implements block header downloader algorithms for `reth-downloaders`: a reverse (tip-to-head) concurrent headers downloader, plus a task-driven wrapper and a noop implementation for pipeline wiring.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for header downloaders: exports the reverse downloader, a noop implementation, a task-driven wrapper, and optional test helpers.
- **Key items**: `reverse_headers`, `noop`, `task`, `test_utils` (cfg)

### `reverse_headers.rs`
- **Role**: Main concurrent reverse headers downloader that fills the gap between local head and sync target by issuing multiple in-flight requests, buffering out-of-order responses, and validating headers before emitting sorted batches.
- **Key items**: `ReverseHeadersDownloader`, `ReverseHeadersDownloaderBuilder`, `SyncTargetBlock`, `REQUESTS_PER_PEER_MULTIPLIER`, `set_batch_size()`, `update_local_head()`, `update_sync_target()`
- **Interactions**: Uses `HeaderValidator` to validate parent links; requests via `HeadersClient`; records `HeaderDownloaderMetrics`; can be driven as a `TaskDownloader` (`task.rs`).
- **Knobs / invariants**: Downloads in reverse (falling block numbers); uses `request_limit`, `stream_batch_size`, `min/max_concurrent_requests`, `max_buffered_responses`; retries and penalizes peers on invalid responses; clears/reset state on detached head or target changes.

### `task.rs`
- **Role**: Task wrapper that runs any `HeaderDownloader` on a spawned task and forwards results over channels, allowing sync target/head updates and batch-size changes asynchronously.
- **Key items**: `TaskDownloader`, `HEADERS_TASK_BUFFER_SIZE`, `spawn()`, `spawn_with()`, `DownloaderUpdates`
- **Interactions**: `ReverseHeadersDownloader::into_task*()` returns this wrapper; forwards update commands to the underlying downloader and streams validated batches.

### `noop.rs`
- **Role**: No-op `HeaderDownloader` implementation used for unwind-only pipelines or configurations that must satisfy a downloader interface without doing work.
- **Key items**: `NoopHeaderDownloader`

### `test_utils.rs`
- **Role**: Small test helper to build a child header from a parent.
- **Key items**: `child_header()`

## End-to-end flow (high level)
- Construct a `ReverseHeadersDownloader` via `builder()` or `ReverseHeadersDownloaderBuilder::new(HeadersConfig)`.
- Set boundaries by calling `update_local_head()` and `update_sync_target()` (or via task wrapper updates).
- The downloader requests the sync target header first, validates it, and initializes request trackers.
- It issues reverse `GetBlockHeaders` requests concurrently, keeping per-peer request capacity based on connected peers.
- Responses are sorted, validated against parent links and consensus rules, and buffered if they arrive out of order.
- Validated headers are queued in descending order and emitted in batches (`stream_batch_size`) as a `Stream`.
- On bad responses, the peer is penalized and the request is retried at higher priority; on detached heads, state is reset to await a new target.

## Key APIs (no snippets)
- **Downloaders**: `ReverseHeadersDownloader`, `ReverseHeadersDownloaderBuilder`, `TaskDownloader`, `NoopHeaderDownloader`
- **Sync target tracking**: `SyncTargetBlock`
```

## File: crates/net/downloaders/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Test-only helpers for `reth-downloaders`: generators for mock headers/bodies (including optional file-backed fixtures) and a `BodiesClient` stub for downloader tests.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test utilities entrypoint: exposes `TestBodiesClient`, provides body/header generators, and (when `file-client` is enabled) writes generated bodies into a temporary file using the downloader file codec.
- **Key items**: `TestBodiesClient`, `TEST_SCOPE`, `generate_bodies()`, `generate_bodies_file()`
- **Interactions**: Uses `bodies::test_utils::create_raw_bodies()` and `file_codec::BlockFileCodec` when `file-client` is enabled.
- **Knobs / invariants**: `generate_bodies_file()` is feature-gated (`file-client`) and rewinds the temp file before returning.

### `bodies_client.rs`
- **Role**: In-memory `BodiesClient` implementation for tests, supporting delayed responses, max batch sizing, and optional empty responses to exercise downloader behavior.
- **Key items**: `TestBodiesClient`, `with_bodies()`, `with_should_delay()`, `with_empty_responses()`, `with_max_batch_size()`, `times_requested()`, `should_respond_empty()`
- **Interactions**: Implements `DownloadClient` and `BodiesClient` for use by `bodies` and `request` tests.
- **Knobs / invariants**: `empty_response_mod` triggers empty responses every N requests; `times_requested` counts requests for assertions.

## End-to-end flow (high level)
- Use `generate_bodies()` to create a header list and body map for a block range.
- Build a `TestBodiesClient` with `with_bodies()` and optional behaviors (delay, max batch, empty responses).
- Pass the client into `BodiesDownloader`/`BodiesRequestFuture` tests to assert ordering and retry behavior.
- If file-client tests are enabled, call `generate_bodies_file()` to create a temp file encoded with `BlockFileCodec`.

## Key APIs (no snippets)
- `TestBodiesClient`
- `generate_bodies()`, `generate_bodies_file()`
```

## File: crates/net/downloaders/src/AGENTS.md
```markdown
# src

## Purpose
Core implementation of `reth-downloaders`: bodies/headers downloaders, file-based block/receipt clients for import workflows, shared metrics, and test utilities.

## Contents (one hop)
### Subdirectories
- [x] `bodies/` - Concurrent block-body downloader with request queue/futures, buffering/reordering, task wrapper, and noop/test helpers (`BodiesDownloader`, `TaskDownloader`).
- [x] `headers/` - Reverse (tip-to-head) header downloader with concurrent requests, validation/buffering, task wrapper, and noop/test helpers (`ReverseHeadersDownloader`).
- [x] `test_utils/` - Test helpers and in-memory `BodiesClient` for downloader tests; optional file-backed body fixtures.

### Files
- `lib.rs` - Crate entrypoint wiring modules and feature flags; re-exports file-client helpers when enabled.
  - **Key items**: feature flags `test-utils`, `file-client`; modules `bodies`, `headers`, `metrics`, `file_client`, `receipt_file_client`, `file_codec`, `test_utils`; re-exports `DecodedFileChunk`, `FileClientError`
- `metrics.rs` - Metrics types for body and header downloaders plus per-response gauges.
  - **Key items**: `BodyDownloaderMetrics`, `HeaderDownloaderMetrics`, `ResponseMetrics`, `increment_errors()`
  - **Interactions**: Used by `bodies/` and `headers/` to account for in-flight requests, buffering, and error categories.
- `file_client.rs` - File-based block client and chunked reader for RLP-encoded block files, with consensus validation and gzip support.
  - **Key items**: `FileClient`, `FileClientError`, `ChunkedFileReader`, `DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE`, `FromReader`, `DecodedFileChunk`
  - **Interactions**: Uses `BlockFileCodec` (`file_codec.rs`) for RLP framing; implements `HeadersClient`/`BodiesClient` for downloaders; bridges to `FromReceiptReader` (`receipt_file_client.rs`) for receipt imports.
- `file_codec.rs` - RLP codec for block files used by `FileClient` and test utilities.
  - **Key items**: `BlockFileCodec`
  - **Knobs / invariants**: Requires `FramedRead::with_capacity` to cover a full block payload to avoid `InputTooShort`.
- `receipt_file_client.rs` - File client for sequential receipt streams with block-number association, used by chunked import flows.
  - **Key items**: `ReceiptDecoder`, `ReceiptFileClient`, `FromReceiptReader`, `ReceiptWithBlockNumber`
  - **Interactions**: Consumed via `ChunkedFileReader::next_receipts_chunk()` in file import commands.

## Key APIs (no snippets)
- **Downloaders**: `BodiesDownloaderBuilder`, `ReverseHeadersDownloaderBuilder`
- **File import**: `FileClient`, `ChunkedFileReader`, `ReceiptFileClient`, `DecodedFileChunk`
- **Metrics**: `BodyDownloaderMetrics`, `HeaderDownloaderMetrics`, `ResponseMetrics`

## Relationships
- **Depends on**: `reth-network-p2p` (download traits/clients), `reth-consensus` (header/body validation), `reth-metrics` (metrics), `tokio`/`tokio-util` (async I/O + framing).
- **Used by**: `reth-stages` headers/bodies stages, `reth-node` builder setup, CLI import commands (`reth/crates/cli/commands/src/import_core.rs`, `reth/crates/cli/commands/src/stage/run.rs`), Optimism CLI import tools (`reth/crates/optimism/cli/src/commands/import*.rs`).
```

## File: crates/net/downloaders/AGENTS.md
```markdown
# downloaders

## Purpose
`reth-downloaders` crate: implements block body and header downloaders (network-based) plus optional file-based import clients and shared downloader metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core downloaders, file clients/codecs, metrics, and test utilities.

### Files
- `Cargo.toml` - Crate manifest and feature flags for file-based clients and test utilities.
  - **Key items**: features `file-client`, `test-utils`; deps `reth-network-p2p`, `reth-consensus`, `reth-config`, `reth-tasks`, `tokio`, `async-compression`

## Key APIs (no snippets)
- `BodiesDownloader` / `BodiesDownloaderBuilder`
- `ReverseHeadersDownloader` / `ReverseHeadersDownloaderBuilder`
- `FileClient`, `ChunkedFileReader`, `ReceiptFileClient`

## Relationships
- **Depends on**: `reth-network-p2p` for download traits and network clients; `reth-consensus` for validation; `reth-storage-api` for header access; `reth-metrics` for instrumentation.
- **Used by**: `reth-stages` and node builder/CLI import flows to drive header/body sync and file imports.
```

## File: crates/net/ecies/src/AGENTS.md
```markdown
# src

## Purpose
Implements the RLPx ECIES framed transport used by Ethereum's devp2p stack: performs the AUTH/ACK handshake, derives symmetric keys, encrypts/decrypts framed messages (header/body), and exposes Tokio `codec` + `Stream`/`Sink` wrappers for use over async I/O.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `algorithm.rs`
- **Role**: Core ECIES handshake + frame crypto: parses and validates encrypted messages, derives symmetric keys, and reads/writes AUTH, ACK, header, and body frames.
- **Key items**: `ECIES`, `EncryptedMessage`, `RLPxSymmetricKeys`, `new_client()`, `new_server()`, `write_auth()`/`read_auth()`, `write_ack()`/`read_ack()`, `write_header()`/`read_header()`, `write_body()`/`read_body()`
- **Interactions**: Used by `ECIESCodec` (`codec.rs`) as the stateful cryptographic engine for framing.
- **Knobs / invariants**: Message integrity is enforced via tag checks (HMAC-SHA256 for ECIES messages; Ethereum MAC for RLPx frames); errors map to `ECIESErrorImpl`.

### `codec.rs`
- **Role**: Tokio `Decoder`/`Encoder` implementation for the ECIES protocol: drives the handshake state machine and emits/accepts high-level ingress/egress values.
- **Key items**: `ECIESCodec`, `ECIESState` (`Auth`, `Ack`, `InitialHeader`, `Header`, `Body`), `MAX_INITIAL_HANDSHAKE_SIZE`
- **Interactions**: Wraps `ECIES` (`algorithm.rs`); produces `IngressECIESValue` and consumes `EgressECIESValue` from `lib.rs`.
- **Knobs / invariants**: Initial post-handshake header is bounded (`MAX_INITIAL_HANDSHAKE_SIZE`) to limit oversized initial payloads.

### `error.rs`
- **Role**: Error types for ECIES handshake and framing failures (cryptographic tag checks, malformed frames, IO/codec boundary conditions, and handshake mismatches).
- **Key items**: `ECIESError`, `ECIESErrorImpl`, `TagCheck*Failed`, `InvalidHandshake`, `UnreadableStream`, `StreamTimeout`

### `lib.rs`
- **Role**: Crate entrypoint and shared enums for message exchange at the codec boundary.
- **Key items**: `EgressECIESValue` (`Auth`, `Ack`, `Message`), `IngressECIESValue` (`AuthReceive`, `Ack`, `Message`), re-exports `ECIESError`/`ECIESErrorImpl`

### `mac.rs`
- **Role**: Ethereum's nonstandard RLPx MAC construction (AES-256 + Keccak-256) used to authenticate framed headers and bodies.
- **Key items**: `MAC`, `update()`, `update_header()`, `update_body()`, `digest()`
- **Knobs / invariants**: Designed specifically for the RLPx framing rules (128-bit digest slices, AES-ECB-like block usage).

### `stream.rs`
- **Role**: High-level async wrapper over `AsyncRead`/`AsyncWrite`: performs the ECIES handshake (client/server) and exposes encrypted message exchange as a `Stream`/`Sink`.
- **Key items**: `ECIESStream`, `connect()` / `connect_with_timeout()`, `incoming()`, `HANDSHAKE_TIMEOUT`
- **Interactions**: Wraps `Framed<Io, ECIESCodec>`; uses `IngressECIESValue`/`EgressECIESValue` to coordinate handshake.
- **Knobs / invariants**: Enforces handshake timeouts and maps "remote closed before readable" into `UnreadableStream`.

### `util.rs`
- **Role**: Small cryptographic helpers used by the ECIES implementation.
- **Key items**: `sha256()`, `hmac_sha256()`

## End-to-end flow (high level)
- Client creates `ECIESStream::connect(...)` with its secret key and the remote peer ID.
- `ECIESCodec` begins in `Auth` state and sends `EgressECIESValue::Auth`; server reads it as `IngressECIESValue::AuthReceive(peer_id)`.
- Server responds with `EgressECIESValue::Ack`; client validates it as `IngressECIESValue::Ack`.
- After handshake, both sides derive symmetric keys and transition to reading/writing framed messages (`Header`/`Body`).
- Each outbound message is encoded as header + body with integrity tags and encrypted payload; inbound frames are verified and decrypted before yielding bytes.

## Key APIs (no snippets)
- **Types**: `ECIES`, `ECIESCodec`, `ECIESStream`, `MAC`
- **Enums**: `ECIESState`, `EgressECIESValue`, `IngressECIESValue`
```

## File: crates/net/ecies/AGENTS.md
```markdown
# ecies

## Purpose
`reth-ecies` crate: RLPx ECIES framed transport implementation (Ethereum devp2p) providing handshake (AUTH/ACK), encrypted framing (header/body), and Tokio-friendly codecs/streams for use over TCP transports.

## Contents (one hop)
### Subdirectories
- [x] `src/` - core ECIES crypto + framing, Tokio codec, and async stream wrapper.

### Files
- `Cargo.toml` - crate manifest.
  - **Key items**: deps `tokio-util` (codec), `secp256k1`, `aes`/`ctr`, `sha2`/`hmac`, `reth-network-peers`

## Key APIs (no snippets)
- `ECIESStream`
- `ECIESCodec`
- `ECIES` (handshake/framing engine)
```

## File: crates/net/eth-wire/src/errors/AGENTS.md
```markdown
# errors

## Purpose
Error types for the `reth-eth-wire` streams and handshakes: separates `p2p`-level framing/handshake failures from `eth` subprotocol errors and provides helpers to surface disconnect reasons and IO causes.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that re-exports the `eth` and `p2p` error types under a single `errors` namespace.
- **Key items**: `pub use eth::*`, `pub use p2p::*`

### `p2p.rs`
- **Role**: Error taxonomy for `P2PStream` (hello handshake, snappy compression, ping/pong liveness, disconnect handling, capability negotiation).
- **Key items**: `P2PStreamError`, `P2PHandshakeError`, `PingerError`, `CapabilityNotShared`, `MismatchedProtocolVersion`, `Disconnected`, `UnknownReservedMessageId`
- **Interactions**: Produced by `UnauthedP2PStream::handshake()` / `P2PStream` read/write paths and propagated upward into `EthStreamError`.

### `eth.rs`
- **Role**: Error taxonomy for `EthStream` and eth subprotocol handshake (status validation, fork validation, message decoding, size limits, unsupported IDs).
- **Key items**: `EthStreamError`, `EthHandshakeError`, `MessageTooBig`, `InvalidMessage`, `UnsupportedMessage`, `TransactionHashesInvalidLenOfFields`
- **Interactions**: Returned by `UnauthedEthStream::handshake()` and `EthStreamInner::{decode_message, encode_message}`.

## Key APIs (no snippets)
- `P2PStreamError`, `P2PHandshakeError`
- `EthStreamError`, `EthHandshakeError`
```

## File: crates/net/eth-wire/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-eth-wire`: stream/handshake machinery for RLPx `p2p` + `eth` protocols on top of an encrypted transport (ECIES), including hello/capability negotiation, message-id multiplexing, ping/pong liveness, eth status handshake, and optional combined eth+snap stream handling.

## Contents (one hop)
### Subdirectories
- [x] `errors/` - error types for `P2PStream` and `EthStream` (handshake, decoding, disconnects).

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint: wires modules and re-exports the primary stream types, handshake helpers, and `reth-eth-wire-types` payloads.
- **Key items**: `P2PStream`, `UnauthedP2PStream`, `EthStream`, `UnauthedEthStream`, `HelloMessage*`, `RlpxProtocolMultiplexer`, `Capability`, `ProtocolVersion`

### `protocol.rs`
- **Role**: Defines `Protocol` = a capability plus its reserved message-ID count (needed to perform RLPx message-id multiplexing deterministically).
- **Key items**: `Protocol`, `Protocol::eth()`, `ProtoVersion`, `messages()`
- **Interactions**: Used by hello negotiation (`hello.rs`) and shared capability computation (`capability.rs`).

### `hello.rs`
- **Role**: Hello message types and builders for the `p2p` handshake; tracks both the raw `HelloMessage` and an enriched `HelloMessageWithProtocols` that includes per-capability message counts.
- **Key items**: `HelloMessage`, `HelloMessageWithProtocols`, `HelloMessageBuilder`, `DEFAULT_TCP_PORT`
- **Knobs / invariants**: Capability set defaults to the node's supported `EthVersion::ALL_VERSIONS` and `ProtocolVersion::V5` unless overridden.

### `capability.rs`
- **Role**: Shared capability negotiation and message-id offset assignment for multiplexing (selects highest version per capability name and assigns offsets beyond the reserved `p2p` range).
- **Key items**: `SharedCapability`, `SharedCapabilities`, `SharedCapabilityError`, `UnsupportedCapabilityError`, `find_by_offset()`, `eth_version()`
- **Knobs / invariants**: Offsets must be `> MAX_RESERVED_MESSAGE_ID`; shared capabilities are ordered alphabetically by name.

### `p2pstream.rs`
- **Role**: Implements the RLPx `p2p` stream: performs hello handshake, applies Snappy compression after handshake, demuxes reserved `p2p` message IDs vs subprotocol payloads, and runs ping/pong liveness via `Pinger`.
- **Key items**: `UnauthedP2PStream`, `P2PStream`, `P2PMessage`, `P2PMessageID`, `DisconnectP2P`, `HANDSHAKE_TIMEOUT`, `MAX_RESERVED_MESSAGE_ID`
- **Interactions**: Produces/consumes multiplexed bytes for subprotocol streams and feeds the multiplexer (`multiplex.rs`).
- **Knobs / invariants**: Enforces EIP-706 payload limits and buffer-capacity backpressure (`MAX_P2P_CAPACITY`).

### `pinger.rs`
- **Role**: Internal ping state machine used by `P2PStream` to schedule pings, detect timeouts, and handle late pongs.
- **Key items**: `Pinger`, `PingState`, `PingerEvent`, `on_pong()`, `poll_ping()`

### `handshake.rs`
- **Role**: Eth subprotocol handshake abstraction: defines a pluggable `EthRlpxHandshake` trait and provides the default Ethereum eth-status handshake implementation with fork/genesis/version validation.
- **Key items**: `EthRlpxHandshake`, `UnauthEth`, `EthHandshake`, `EthereumEthHandshake::eth_handshake()`
- **Knobs / invariants**: Validates genesis/chain/version match, fork filter acceptance, message size limits, and eth/69 history range sanity.

### `ethstream.rs`
- **Role**: Typed eth protocol stream built on top of a byte stream: handles `Status` handshake gating and version-aware encode/decode of `EthMessage` payloads.
- **Key items**: `UnauthedEthStream`, `EthStream`, `EthStreamInner`, `MAX_MESSAGE_SIZE`, `start_send_broadcast()`, `start_send_raw()`
- **Interactions**: Uses `ProtocolMessage`/`EthMessage` from `reth-eth-wire-types`; used as primary protocol stream in the multiplexer.

### `eth_snap_stream.rs`
- **Role**: Combined eth + snap message stream over a single connection: decodes/encodes either `EthMessage` or `SnapProtocolMessage` based on message IDs while keeping eth version constraints.
- **Key items**: `EthSnapStream`, `EthSnapMessage`, `EthSnapStreamError`, `decode_message()`, `encode_eth_message()`, `encode_snap_message()`

### `multiplex.rs`
- **Role**: RLPx subprotocol multiplexer: installs multiple subprotocol handlers over a single `P2PStream`, routes messages by capability offsets, and provides a "satellite stream" pattern (primary protocol + dependent satellites like snap).
- **Key items**: `RlpxProtocolMultiplexer`, `ProtocolProxy`, `ProtocolConnection`, `RlpxSatelliteStream`, `install_protocol()`, `into_eth_satellite_stream()`
- **Interactions**: Consumes `SharedCapabilities` from `P2PStream` and drives the primary stream handshake before returning a running satellite stream.

### `disconnect.rs`
- **Role**: Abstraction that lets higher-level streams request a disconnect on the underlying transport (with a `DisconnectReason` when supported).
- **Key items**: `CanDisconnect`

### `test_utils.rs`
- **Role**: Test-only utilities for constructing hello/status fixtures and a passthrough `P2PStream`, plus a small synthetic "test" subprotocol for multiplexer tests.
- **Key items**: `eth_hello()`, `eth_handshake()`, `connect_passthrough()`, `proto::TestProtoMessage`, `proto::TestProtoMessageId`

## Key APIs (no snippets)
- **Streams**: `UnauthedP2PStream` -> `P2PStream`; `UnauthedEthStream` -> `EthStream`
- **Multiplexing**: `RlpxProtocolMultiplexer`, `SharedCapabilities`
- **Handshake**: `EthHandshake` / `EthRlpxHandshake`, `HelloMessageWithProtocols`
```

## File: crates/net/eth-wire/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration and fuzz-style tests for `reth-eth-wire`: validate that real-world captured network payloads decode correctly and that key wire types round-trip encode/decode across a range of inputs (including blob tx variants).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `fuzz_roundtrip.rs` - fuzz/roundtrip encoding tests for many RLP-encodable types used by `eth-wire` and `eth-wire-types`.
  - **Key items**: `roundtrip_encoding()`, `roundtrip_fuzz()`, `fuzz_type_and_name!`, `fuzz_rlp` module
- `new_block.rs` - decoding tests for `NewBlock` payloads from `testdata/` captures (including BSC samples).
  - **Key items**: `decode_new_block_network*`, `testdata/new_block_network_rlp`, `testdata/bsc_new_block_network_*`
- `new_pooled_transactions.rs` - decoding test for `NewPooledTransactionHashes66` from captured network payload.
  - **Key items**: `decode_new_pooled_transaction_hashes_network`, `testdata/new_pooled_transactions_network_rlp`
- `pooled_transactions.rs` - decoding and roundtrip tests for pooled transactions payloads (including blob tx request pairs and RPC-style blob tx encoding).
  - **Key items**: `roundtrip_pooled_transactions`, `decode_request_pair_pooled_blob_transactions`, `PooledTransaction::decode_2718`, `EthVersion::Eth68`
```

## File: crates/net/eth-wire/AGENTS.md
```markdown
# eth-wire

## Purpose
`reth-eth-wire` crate: implements RLPx `p2p` + `eth` wire protocol stream plumbing-hello/capability negotiation, message-id multiplexing, ping/pong liveness, eth status handshake, and typed `EthStream`/`P2PStream` adapters over encrypted transports (ECIES).

## Contents (one hop)
### Subdirectories
- [x] `src/` - core stream/handshake/multiplexing implementations and error types.
- [x] `tests/` - integration tests and fuzz/roundtrip tests for on-wire payloads.
- [x] `testdata/` - (skip: test fixtures) captured/serialized network payload hex used by tests.

### Files
- `Cargo.toml` - crate manifest and feature flags.
  - **Key items**: features `serde`, `arbitrary`; deps `reth-ecies`, `reth-eth-wire-types`, `snap` (snappy), `tokio-util` (codec)

## Key APIs (no snippets)
- `UnauthedP2PStream` / `P2PStream`
- `UnauthedEthStream` / `EthStream`
- `RlpxProtocolMultiplexer`
```

## File: crates/net/eth-wire-types/src/AGENTS.md
```markdown
# src

## Purpose
Defines the data model for the devp2p `eth` wire protocol (eth/66-eth/70) and related sub-protocol payloads: message IDs, request/response wrappers, status handshakes, block/tx/state/receipt message types, broadcast announcements, and SNAP message structures, all with version-aware decoding/encoding.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `blocks.rs`
- **Role**: Request/response payloads for block headers and bodies (`GetBlockHeaders`, `GetBlockBodies`) and their responses.
- **Key items**: `GetBlockHeaders`, `BlockHeaders`, `GetBlockBodies`, `BlockBodies`, `BlockHashOrNumber`, `HeadersDirection`

### `broadcast.rs`
- **Role**: Broadcast payloads for new blocks and transactions: block hash announcements, full block announcements, transaction broadcasts, pooled-transaction hash announcements, and related validation metadata.
- **Key items**: `NewBlockHashes`, `BlockHashNumber`, `NewBlock`, `Transactions`, `SharedTransactions`, `NewPooledTransactionHashes`, `NewPooledTransactionHashes66`, `NewPooledTransactionHashes68`, `ValidAnnouncementData`, `BlockRangeUpdate`
- **Knobs / invariants**: Broadcast formats vary by `EthVersion` (e.g. pooled tx hash announcement shapes across eth/66 vs eth/68+).

### `capability.rs`
- **Role**: Capability negotiation helpers and raw capability framing (`p2p` capability lists and raw payload forwarding).
- **Key items**: `Capability`, `Capabilities`, `RawCapabilityMessage`, `Capability::eth_*()`, `Capabilities::supports_eth_*()`

### `disconnect_reason.rs`
- **Role**: `p2p` disconnect reason codes with RLP encoding/decoding and unknown-reason handling.
- **Key items**: `DisconnectReason`, `UnknownDisconnectReason`

### `header.rs`
- **Role**: Header-request direction abstraction, bridging the legacy `reverse` flag to an explicit direction.
- **Key items**: `HeadersDirection`, `HeadersDirection::new()`, `is_rising()`, `is_falling()`

### `lib.rs`
- **Role**: Crate entrypoint: re-exports all message payload modules and key shared types under a stable public surface (including no-std support).
- **Key items**: `Status`/`UnifiedStatus`, `EthVersion`, `ProtocolMessage`, `EthMessage`, `EthMessageID`, `Capability`, `SnapProtocolMessage`

### `message.rs`
- **Role**: Central protocol message model: message IDs, version-aware decoding/encoding into strongly typed `EthMessage` variants, request/response pairing, and broadcast message wrappers.
- **Key items**: `ProtocolMessage`, `EthMessage`, `EthMessageID`, `RequestPair<T>`, `MessageError`, `MAX_MESSAGE_SIZE`, `ProtocolBroadcastMessage`, `EthBroadcastMessage`
- **Knobs / invariants**: Decoding is gated by `EthVersion` (e.g. `Status` legacy vs eth/69, receipt shapes for eth/69/70, removal of `GetNodeData` in eth/67).

### `primitives.rs`
- **Role**: Abstractions over "network primitives" used in messages (block/header/body, broadcasted vs pooled tx formats, receipt types, new-block payloads), with a standard Ethereum instantiation.
- **Key items**: `NetworkPrimitives`, `NetPrimitivesFor`, `BasicNetworkPrimitives`, `EthNetworkPrimitives`

### `receipts.rs`
- **Role**: Receipt request/response payloads across protocol versions (legacy bloom-bearing receipts vs eth/69+ bloomless receipts; eth/70 partial receipts support).
- **Key items**: `GetReceipts`, `GetReceipts70`, `Receipts`, `Receipts69`, `Receipts70`, `into_with_bloom()`, `last_block_incomplete`

### `snap.rs`
- **Role**: SNAP protocol message payloads (snap/1) for state snapshot exchange: account ranges, storage ranges, bytecodes, and trie nodes.
- **Key items**: `SnapMessageId`, `SnapProtocolMessage`, `GetAccountRangeMessage`, `AccountRangeMessage`, `GetStorageRangesMessage`, `StorageRangesMessage`, `GetByteCodesMessage`, `ByteCodesMessage`, `GetTrieNodesMessage`, `TrieNodesMessage`

### `state.rs`
- **Role**: Legacy eth message payloads for node/state data retrieval (removed in eth/67).
- **Key items**: `GetNodeData`, `NodeData`

### `status.rs`
- **Role**: Status handshake payloads across protocol versions, including a unified superset type and builders to produce the correct on-wire status message by version.
- **Key items**: `UnifiedStatus`, `StatusBuilder`, `Status`, `StatusEth69`, `StatusMessage`, `UnifiedStatus::into_message()`, `spec_builder()`
- **Knobs / invariants**: eth/66-68 uses total difficulty; eth/69 uses earliest/latest history range instead.

### `transactions.rs`
- **Role**: Pooled transaction request/response payloads and helpers.
- **Key items**: `GetPooledTransactions`, `PooledTransactions`, `PooledTransactions::hashes()`

### `version.rs`
- **Role**: Protocol version enums and parsing/encoding helpers for eth and base `p2p` protocol versions.
- **Key items**: `EthVersion`, `ProtocolVersion`, `ParseVersionError`, `EthVersion::ALL_VERSIONS`, `EthVersion::LATEST`

## End-to-end flow (high level)
- During handshake, peers exchange `StatusMessage` (legacy `Status` vs `StatusEth69`) based on negotiated `EthVersion`.
- Wire messages are received as `(EthMessageID, rlp-bytes)` and decoded into `EthMessage` via `ProtocolMessage::decode_message(version, ...)`.
- Requests are wrapped in `RequestPair<T>` to correlate responses with request IDs (e.g. headers/bodies/receipts/pooled transactions).
- Broadcast announcements use dedicated payload types (e.g. `NewBlockHashes`, `NewPooledTransactionHashes*`) and are gated by version where formats differ.
- Optional SNAP messages (`SnapProtocolMessage`) are encoded/decoded independently for state snapshot sync on top of RLPx.

## Key APIs (no snippets)
- **Core**: `EthMessage`, `EthMessageID`, `ProtocolMessage`, `RequestPair<T>`, `EthVersion`
- **Handshake**: `UnifiedStatus`, `StatusMessage`
- **Broadcast**: `NewBlockHashes`, `NewPooledTransactionHashes*`
```

## File: crates/net/eth-wire-types/AGENTS.md
```markdown
# eth-wire-types

## Purpose
`reth-eth-wire-types` crate: strongly typed payloads and helpers for the devp2p `eth` wire protocol (eth/66-eth/70) plus related message families (broadcasts, capabilities, disconnect reasons, and SNAP), with version-aware decoding/encoding and pluggable "network primitives" abstractions.

## Contents (one hop)
### Subdirectories
- [x] `src/` - message payload types, IDs, and versioned encoding/decoding for `eth`/SNAP.

### Files
- `Cargo.toml` - crate manifest and feature flags.
  - **Key items**: features `std` (default), `serde`, `arbitrary`; deps `reth-ethereum-primitives`, `reth-primitives-traits`, `alloy-rlp`, `alloy-consensus`, `alloy-eips`

## Key APIs (no snippets)
- `EthMessage`, `EthMessageID`, `ProtocolMessage`
- `EthVersion`, `ProtocolVersion`
- `NetworkPrimitives` / `EthNetworkPrimitives`
```

## File: crates/net/nat/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-net-nat`: helpers for NAT/external address resolution (public IP lookup, UPnP placeholder, domain-to-IP resolution, and container network-interface IP lookup), including a periodic resolver.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines `NatResolver` (resolver strategy enum), parsing/display, and async helpers to resolve external IPs (via public IP services, DNS, or net-if), plus `ResolveNatInterval` for periodic resolution.
- **Key items**: `NatResolver`, `external_ip()`, `external_addr_with()`, `ResolveNatInterval`
- **Interactions**: uses `reqwest` to query public IP services; uses `tokio::time::Interval` to schedule polling.

#### `net_if.rs`
- **Role**: OS interface IP lookup (useful in docker bridge networks).
- **Key items**: `DEFAULT_NET_IF_NAME`, `resolve_net_if_ip()`, `NetInterfaceError`

## Key APIs (no snippets)
- `NatResolver`
- `ResolveNatInterval`
```

## File: crates/net/nat/AGENTS.md
```markdown
# nat

## Purpose
`reth-net-nat` crate: NAT helpers for reth networking-resolve an external/public IP address via multiple strategies (public IP services, domain, net interface) and optionally poll on an interval.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `NatResolver`, interval-based resolver, and net-interface IP lookup.

### Files
- `Cargo.toml` - crate manifest (reqwest + tokio; optional serde support for `NatResolver`).

## Key APIs (no snippets)
- `NatResolver`
- `ResolveNatInterval`
```

## File: crates/net/network/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for network transaction propagation and fetch behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `broadcast.rs`
- **Role**: Benchmarks broadcast ingress by sending transactions between two testnet peers and measuring receive throughput.
- **Key items**: `broadcast_ingress_bench()`, `Testnet::create_with()`, `TransactionGenerator`, `send_transactions()`
- **Interactions**: Uses `Testnet` from `test_utils` and transaction pool generators to simulate broadcasts.

### `tx_manager_hash_fetching.rs`
- **Role**: Benchmarks transaction hash fetching and pending-hash processing at various peer counts.
- **Key items**: `benchmark_fetch_pending_hashes()`, `fetch_pending_hashes()`, `tx_fetch_bench()`, `TransactionFetcher`, `TransactionsManagerConfig`
- **Interactions**: Uses `TransactionFetcher` and `Testnet` harness; leverages helpers from `test_utils::transactions`.

## End-to-end flow (high level)
- Build a `Testnet` with mock providers and transaction pools.
- For broadcast bench, send transactions between peers and measure receive loop throughput.
- For fetch benches, populate the fetcher with pending hashes and measure time to schedule requests.

## Key APIs (no snippets)
- `broadcast_ingress_bench()`
- `benchmark_fetch_pending_hashes()`, `tx_fetch_bench()`
```

## File: crates/net/network/src/fetch/AGENTS.md
```markdown
# fetch

## Purpose
Implements network fetch plumbing: a stateful request coordinator (`StateFetcher`) that selects peers and tracks in-flight eth requests, plus a `FetchClient` that exposes `HeadersClient`/`BodiesClient` APIs backed by the coordinator.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core fetcher state machine that queues header/body requests, selects the best peer, dispatches `GetBlockHeaders`/`GetBlockBodies`, and processes responses with reputation outcomes.
- **Key items**: `StateFetcher`, `DownloadRequest`, `FetchAction`, `BlockResponseOutcome`, `Peer`, `PeerState`, `BestPeerRequirements`
- **Interactions**: Emits `BlockRequest` messages to sessions (`crate::message`); uses `PeersHandle` for reputation changes; relies on `EthResponseValidator` to detect bad responses and returns `ReputationChangeKind`.
- **Knobs / invariants**: Prioritizes low-latency and range-capable peers; tracks per-peer range info and response quality; re-queues high-priority requests ahead of normal queue.

### `client.rs`
- **Role**: Front-end `FetchClient` implementing `HeadersClient` and `BodiesClient` by sending requests over a channel to the `StateFetcher` and flattening responses.
- **Key items**: `FetchClient`, `HeadersClientFuture`, `get_headers_with_priority()`, `get_block_bodies_with_priority_and_range_hint()`
- **Interactions**: Uses `FlattenedResponse` for oneshot response handling; delegates reputation changes to `PeersHandle`; exposes `DownloadClient` and `BlockClient` for downloader integration.

## End-to-end flow (high level)
- Construct a `StateFetcher` with a `PeersHandle` and active-peer counter, then create a `FetchClient` via `StateFetcher::client()`.
- Call `FetchClient` methods to queue `DownloadRequest`s for headers or bodies; responses are delivered through oneshot channels.
- `StateFetcher::poll()` drains queued requests, selects the next best idle peer (range/latency aware), and emits a `FetchAction::BlockRequest`.
- When responses arrive, `StateFetcher` validates for likely bad responses, updates per-peer state, and optionally returns `BlockResponseOutcome::BadResponse`.
- Successful peers can receive immediate follow-up requests; failed peers are penalized via `ReputationChangeKind`.

## Key APIs (no snippets)
- `StateFetcher`
- `FetchClient`
- `DownloadRequest`, `FetchAction`, `BlockResponseOutcome`
```

## File: crates/net/network/src/session/AGENTS.md
```markdown
# session

## Purpose
Session lifecycle management for the network: authenticates peers via RLPx/ECIES, maintains pending and active sessions, dispatches/receives protocol messages, enforces timeouts, and tracks per-peer block range announcements (eth/69).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Session manager and handshake orchestration: accepts inbound/outbound connections, performs RLPx + eth handshakes, transitions pending -> active sessions, and emits `SessionEvent`s for the rest of the network stack.
- **Key items**: `SessionManager`, `SessionId`, `SessionEvent`, `PendingSessionHandshakeError`, `ExceedsSessionLimit`, `pending_session_with_timeout()`, `start_pending_incoming_session()`, `start_pending_outbound_session()`, `authenticate_stream()`
- **Interactions**: Spawns `ActiveSession` tasks (`active.rs`), wraps connections using `EthRlpxConnection` (`conn.rs`), uses `SessionCounter` (`counter.rs`) and handle/events (`handle.rs`), propagates `BlockRangeInfo` (`types.rs`) for eth/69 range updates.
- **Knobs / invariants**: Enforces session limits, handshake timeouts, and protocol breach timeouts; optionally negotiates extra RLPx subprotocols; caps concurrent graceful disconnects.

### `active.rs`
- **Role**: Active session state machine that drives a single peer connection: handles inbound messages, internal requests, outgoing responses/broadcasts, and disconnect/error paths with backpressure and timeout logic.
- **Key items**: `ActiveSession`, `QueuedOutgoingMessages`, `OutgoingMessage`, `RANGE_UPDATE_INTERVAL`, `MAX_QUEUED_OUTGOING_RESPONSES`, `InflightRequest`, `ReceivedRequest`, `calculate_new_timeout()`
- **Interactions**: Sends `ActiveSessionMessage` back to `SessionManager`; consumes `PeerRequest`/`PeerMessage`; uses `EthRlpxConnection` to read/write `EthMessage`.
- **Knobs / invariants**: Updates internal request timeout via RTT sampling; triggers protocol-breach errors after prolonged timeouts; throttles reads when queued responses exceed limits; emits `BlockRangeUpdate` periodically for eth/69+.

### `conn.rs`
- **Role**: Connection wrapper that abstracts ETH-only vs ETH+satellite (multiplexed) streams and exposes a uniform `Stream`/`Sink` interface.
- **Key items**: `EthRlpxConnection`, `EthPeerConnection`, `EthSatelliteConnection`, `start_send_broadcast()`, `start_send_raw()`
- **Interactions**: Used by `SessionManager` and `ActiveSession` to send/receive `EthMessage` across ECIES/P2P streams.
- **Knobs / invariants**: Boxes underlying streams to keep type size manageable.

### `counter.rs`
- **Role**: Tracks pending/active inbound/outbound session counts and enforces configured limits.
- **Key items**: `SessionCounter`, `ensure_pending_inbound()`, `ensure_pending_outbound()`, `inc_active()`, `dec_active()`
- **Interactions**: Used by `SessionManager` when accepting/dialing sessions.

### `handle.rs`
- **Role**: Session handle types and command/event enums for pending and active session coordination.
- **Key items**: `PendingSessionHandle`, `ActiveSessionHandle`, `PendingSessionEvent`, `SessionCommand`, `ActiveSessionMessage`
- **Interactions**: `ActiveSessionHandle::disconnect()` sends commands to the session task; `peer_info()` builds `PeerInfo`.

### `types.rs`
- **Role**: Shared range information for eth/69 block-range announcements.
- **Key items**: `BlockRangeInfo`, `BlockRangeInfo::update()`, `BlockRangeInfo::to_message()`
- **Interactions**: Used by `SessionManager` and `ActiveSession` to advertise and track range updates.

## End-to-end flow (high level)
- `SessionManager` accepts inbound TCP connections or dials outbound peers, enforcing session limits via `SessionCounter`.
- Pending sessions perform ECIES auth, hello exchange, and eth status handshake; extra subprotocols are negotiated as needed.
- On success, the manager creates an `EthRlpxConnection`, spawns an `ActiveSession`, and emits `SessionEstablished`.
- `ActiveSession` multiplexes manager commands, internal requests, and wire messages, buffering outbound traffic with backpressure.
- Incoming requests from the peer are forwarded as `PeerMessage` events and resolved via response channels.
- Timeouts and protocol breaches trigger disconnects; graceful disconnects are capped by `DisconnectionsCounter`.
- For eth/69+, range updates are periodically sent and tracked via `BlockRangeInfo`.

## Key APIs (no snippets)
- `SessionManager`, `SessionEvent`, `SessionCommand`
- `ActiveSession`, `ActiveSessionHandle`, `PendingSessionHandle`
- `EthRlpxConnection`, `BlockRangeInfo`
```

## File: crates/net/network/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Network test helpers: utilities for free ports/peer IDs, an in-process testnet harness, and helpers for transaction manager tests.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test-utils module wiring and re-exports for network tests.
- **Key items**: `Testnet`, `Peer`, `PeerConfig`, `PeerHandle`, `NetworkEventStream`, `unused_tcp_udp()`, `new_tx_manager()`

### `init.rs`
- **Role**: Low-level helpers to obtain unused TCP/UDP ports/addresses and convert ENR to `PeerId`.
- **Key items**: `enr_to_peer_id()`, `unused_port()`, `unused_tcp_addr()`, `unused_udp_addr()`, `unused_tcp_and_udp_port()`, `unused_tcp_udp()`
- **Knobs / invariants**: Port helpers do not reserve ports; they only check availability at call time.

### `testnet.rs`
- **Role**: Test network harness that spins up multiple `NetworkManager` peers with optional transaction pools and request handlers.
- **Key items**: `Testnet`, `TestnetHandle`, `Peer`, `PeerConfig`, `PeerHandle`, `NetworkEventStream`
- **Interactions**: Builds `NetworkManager`, `TransactionsManager`, and `EthRequestHandler`; uses event streams to await session establishment.
- **Knobs / invariants**: Convenience builders for eth pool configuration and transaction propagation policy.

### `transactions.rs`
- **Role**: Transaction-manager test helpers for constructing a manager, inserting hashes into fetch state, and creating mock session metadata.
- **Key items**: `new_tx_manager()`, `buffer_hash_to_tx_fetcher()`, `new_mock_session()`
- **Interactions**: Works with `TransactionFetcher`, `TransactionsManager`, and `PeerMetadata` to simulate network behavior.

## End-to-end flow (high level)
- Use `PeerConfig`/`Testnet::create*` to spin up a multi-peer test network with optional pools.
- Call `TestnetHandle::connect_peers()` and await events via `NetworkEventStream`.
- For transaction-specific tests, build a `TransactionsManager` via `new_tx_manager()` and simulate inflight hashes with `buffer_hash_to_tx_fetcher()`.

## Key APIs (no snippets)
- `Testnet`, `Peer`, `PeerHandle`, `NetworkEventStream`
- `new_tx_manager()`, `new_mock_session()`
- `unused_tcp_udp()`, `enr_to_peer_id()`
```

## File: crates/net/network/src/transactions/AGENTS.md
```markdown
# transactions

## Purpose
Transaction gossip and fetch logic for the network: manages announcements and full broadcasts, fetches missing pooled transactions, applies policies and limits, and coordinates with the transaction pool.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core transactions manager task and handle API: routes network events, tracks peers/seen hashes, coordinates pool imports, fetches missing txs, and drives propagation.
- **Key items**: `TransactionsManager`, `TransactionsHandle`, `PeerMetadata`, `NetworkTransactionEvent`, `PendingPoolImportsInfo`, `TransactionsCommand`, `PoolImportFuture`
- **Interactions**: Uses `TransactionFetcher` (`fetcher.rs`), `NetworkPolicies` (`policy.rs`), and config/constants; integrates with `NetworkHandle` and `TransactionPool`.
- **Knobs / invariants**: Limits pending pool imports, tracks bad imports, and respects propagation/ingress policies; disables propagation during sync.

### `config.rs`
- **Role**: Configuration surface and policy traits for transaction propagation and announcement filtering.
- **Key items**: `TransactionsManagerConfig`, `TransactionFetcherConfig`, `TransactionPropagationMode`, `TransactionPropagationKind`, `TransactionIngressPolicy`, `AnnouncementFilteringPolicy`, `AnnouncementAcceptance`
- **Interactions**: `TransactionsManager` consumes these configs; policies control which peers are allowed to ingest or receive txs.
- **Knobs / invariants**: Propagation modes (`Sqrt`, `All`, `Max`) and ingress policies (`All`, `Trusted`, `None`).

### `constants.rs`
- **Role**: Spec-derived and default bounds for transaction broadcasts, requests, retries, and fetcher concurrency/budgets.
- **Key items**: `SOFT_LIMIT_COUNT_HASHES_IN_NEW_POOLED_TRANSACTIONS_BROADCAST_MESSAGE`, `SOFT_LIMIT_COUNT_HASHES_IN_GET_POOLED_TRANSACTIONS_REQUEST`, `SOFT_LIMIT_BYTE_SIZE_POOLED_TRANSACTIONS_RESPONSE`, `tx_manager::*`, `tx_fetcher::*`
- **Knobs / invariants**: Default limits for peers, inflight requests, pending fetch capacity, and request packing sizes.

### `fetcher.rs`
- **Role**: Transaction fetcher: deduplicates announced hashes, packs `GetPooledTransactions` requests, retries missing hashes, and tracks inflight/pending fetch state per peer.
- **Key items**: `TransactionFetcher`, `TxFetchMetadata`, `FetchEvent`, `GetPooledTxRequest`, `GetPooledTxResponse`, `GetPooledTxRequestFut`, `UnverifiedPooledTransactions`, `VerifiedPooledTransactions`, `VerificationOutcome`, `TransactionFetcherInfo`
- **Interactions**: Used by `TransactionsManager` to request missing transactions and update metrics; uses cache structures for inflight/pending hashes and fallback peers.
- **Knobs / invariants**: Enforces per-peer inflight limits and request size/byte constraints based on `EthVersion`; retries bounded by `DEFAULT_MAX_RETRIES`.

### `policy.rs`
- **Role**: Bundles transaction propagation and announcement filtering policies into a single container.
- **Key items**: `NetworkPolicies`, `with_propagation()`, `with_announcement()`, `propagation_policy()`, `announcement_filter()`
- **Interactions**: Passed into `TransactionsManager::with_policy()` to control gossip behavior.

## End-to-end flow (high level)
- `TransactionsManager` subscribes to network events and pool pending transactions.
- Incoming announcements are filtered and deduplicated; unknown hashes are handed to `TransactionFetcher`.
- `TransactionFetcher` builds `GetPooledTransactions` requests within size limits and assigns them to idle peers, retrying via fallback peers on partial/failed responses.
- Received transactions are validated/imported into the pool with bounded concurrency; bad imports are cached and peers penalized.
- Propagation policy determines which peers receive full transactions vs hashes, and ingress policy controls which peers are accepted.

## Key APIs (no snippets)
- `TransactionsManager`, `TransactionsHandle`
- `TransactionFetcher`, `TransactionFetcherConfig`
- `TransactionPropagationMode`, `TransactionIngressPolicy`, `NetworkPolicies`
```

## File: crates/net/network/src/AGENTS.md
```markdown
# src

## Purpose
Implements the `reth-network` crate's core P2P stack: discovery, peer/session management, network state, request routing, transaction gossip, and the public network handle API.

## Contents (one hop)
### Subdirectories
- [x] `fetch/` - Request coordination and `FetchClient` for headers/bodies downloads.
- [x] `session/` - RLPx/ETH session lifecycle, handshake, and active session state machines.
- [x] `test_utils/` - Testnet harness and helpers for network/tx tests.
- [x] `transactions/` - Transaction gossip, fetcher, policies, and pool integration.

### Files
- `budget.rs` - Polling budgets and macros for draining nested streams with bounded work.
  - **Key items**: `DEFAULT_BUDGET_TRY_DRAIN_STREAM`, `DEFAULT_BUDGET_TRY_DRAIN_SWARM`, `DEFAULT_BUDGET_TRY_DRAIN_NETWORK_HANDLE_CHANNEL`, `poll_nested_stream_with_budget!`, `metered_poll_nested_stream_with_budget!`
- `builder.rs` - Network builder wiring for request handlers and transactions manager.
  - **Key items**: `NetworkBuilder`, `ETH_REQUEST_CHANNEL_CAPACITY`, `request_handler()`, `transactions_with_policy()`, `transactions_with_policies()`
- `cache.rs` - Lightweight LRU cache wrappers used across network subsystems.
  - **Key items**: `LruCache`, `LruMap`
- `config.rs` - Network configuration and builder API for discovery, sessions, and boot nodes.
  - **Key items**: `NetworkConfig`, `NetworkConfigBuilder`, `rng_secret_key()`, `NetworkMode`, `start_network()`
- `discovery.rs` - Discovery orchestration for discv4/discv5/DNS and event fan-out.
  - **Key items**: `Discovery`, `DEFAULT_MAX_CAPACITY_DISCOVERED_PEERS_CACHE`, `add_listener()`, `update_fork_id()`, `ban()`
  - **Interactions**: Drives `reth-discv4`, `reth-discv5`, and DNS discovery streams into `DiscoveryEvent`s.
- `error.rs` - Network error taxonomy and session error classification/backoff rules.
  - **Key items**: `NetworkError`, `ServiceKind`, `SessionError`, `from_io_error()`
- `eth_requests.rs` - ETH request handler for headers/bodies/receipts with size limits.
  - **Key items**: `EthRequestHandler`, `IncomingEthRequest`, `MAX_HEADERS_SERVE`, `MAX_BODIES_SERVE`, `SOFT_RESPONSE_LIMIT`
  - **Interactions**: Uses `BlockReader`/`HeaderProvider` to serve requests and records `EthRequestHandlerMetrics`.
- `flattened_response.rs` - Helper to flatten oneshot receiver errors for async responses.
  - **Key items**: `FlattenedResponse`
- `import.rs` - Block import abstraction for `NewBlock`/`NewBlockHashes` announcements.
  - **Key items**: `BlockImport`, `NewBlockEvent`, `BlockImportEvent`, `BlockValidation`, `ProofOfStakeBlockImport`
- `lib.rs` - Crate entrypoint and public re-exports for networking types and handles.
  - **Key items**: module exports, `NetworkManager`, `NetworkHandle`, `NetworkConfig`, `FetchClient`
- `listener.rs` - TCP connection listener and event type used by the swarm.
  - **Key items**: `ConnectionListener`, `ListenerEvent`
- `manager.rs` - Top-level network event loop that ties together swarm, handlers, and discovery.
  - **Key items**: `NetworkManager`, `with_transactions()`, `set_eth_request_handler()`, `add_rlpx_sub_protocol()`
  - **Interactions**: Drives `Swarm` and routes `NetworkHandle` commands to sessions, tx manager, and request handler.
- `message.rs` - Internal message wrappers and response plumbing for ETH requests.
  - **Key items**: `PeerMessage`, `BlockRequest`, `PeerResponse`, `PeerResponseResult`, `NewBlockMessage`
- `metrics.rs` - Metrics types for network, sessions, transactions, fetcher, and request handling.
  - **Key items**: `NetworkMetrics`, `SessionManagerMetrics`, `TransactionsManagerMetrics`, `TransactionFetcherMetrics`, `EthRequestHandlerMetrics`, `TxTypesCounter`
- `network.rs` - Shareable network handle API and protocol registry hooks.
  - **Key items**: `NetworkHandle`, `NetworkProtocols`, `send_request()`, `announce_block()`, `shutdown()`
- `peers.rs` - Peer manager with reputation/backoff, ban lists, and trusted peer resolution.
  - **Key items**: `PeersManager`, `PeerAction`, `ConnectionInfo`, `InboundConnectionError`
- `protocol.rs` - Interfaces for custom RLPx subprotocols and connection handlers.
  - **Key items**: `ProtocolHandler`, `ConnectionHandler`, `RlpxSubProtocol`, `RlpxSubProtocols`, `OnNotSupported`
- `required_block_filter.rs` - Optional filter task to ban peers missing required block hashes.
  - **Key items**: `RequiredBlockFilter`
- `state.rs` - Network state machine coordinating discovery, peers, and fetcher actions.
  - **Key items**: `NetworkState`, `StateAction`, `BlockNumReader`
- `swarm.rs` - Connectivity layer combining listener, session manager, and network state.
  - **Key items**: `Swarm`, `SwarmEvent`, `NetworkConnectionState`
- `trusted_peers_resolver.rs` - Periodic DNS resolution for trusted peers.
  - **Key items**: `TrustedPeersResolver`

## Key APIs (no snippets)
- `NetworkManager`, `NetworkHandle`, `NetworkConfig`
- `PeersManager`, `SessionManager`
- `Discovery`, `FetchClient`, `EthRequestHandler`

## Relationships
- **Depends on**: `reth-eth-wire` (protocols/handshakes), `reth-discv4`/`reth-discv5`/`reth-dns-discovery` (discovery), `reth-network-p2p` (download traits), `reth-network-types` (peer/session config), `reth-transaction-pool` (tx propagation).
- **Used by**: Node builder and CLI commands to configure and run the network stack; tests in `tests/it` and benches in `benches`.
```

## File: crates/net/network/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for network behavior: peer connections, request/response flows, session negotiation, startup/discovery edge cases, and transaction gossip/fetching.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness that wires all integration test modules.
- **Key items**: module list for `connect`, `requests`, `session`, `startup`, `transaction_hash_fetching`, `txgossip`, `multiplex`, `big_pooled_txs_req`

### `big_pooled_txs_req.rs`
- **Role**: Validates large `GetPooledTransactions` request handling by fetching ~2000 txs from a peer and verifying returned hashes.
- **Key items**: `test_large_tx_req`, `GetPooledTransactions`, `PooledTransactions`, `Testnet`
- **Interactions**: Uses `TransactionsManager` + request handlers on a two-peer testnet.

### `connect.rs`
- **Role**: Connection and peer-management integration tests (establishment, already-connected handling, peer lookups, and discovery/listener edge cases).
- **Key items**: `test_establish_connections`, `test_already_connected`, `test_get_peer`, `NetworkConfigBuilder`, `Discv4Config`
- **Interactions**: Uses `Testnet`, `NetworkManager`, `NetworkEventStream`, and `PeersConfig` to validate connection lifecycle.

### `multiplex.rs`
- **Role**: RLPx subprotocol multiplexing tests using a custom ping/pong protocol handler.
- **Key items**: `PingPongProtoMessage`, `PingPongProtoHandler`, `ProtocolHandler`, `ConnectionHandler`
- **Interactions**: Exercises `ProtocolConnection` and capability negotiation alongside the core network.

### `requests.rs`
- **Role**: Verifies header/body request flows via `FetchClient` and network request handlers.
- **Key items**: `test_get_body`, `test_get_body_range`, `test_get_header`, `test_get_header_range`
- **Interactions**: Uses `HeadersClient`, `BodiesClient`, `Testnet`, and `MockEthProvider`.

### `session.rs`
- **Role**: Session negotiation tests for eth protocol version selection and capability mismatches.
- **Key items**: `test_session_established_with_highest_version`, `test_capability_version_mismatch`, `test_eth69_peers_can_connect`
- **Interactions**: Uses `PeerConfig::with_protocols` and `NetworkEventStream`.

### `startup.rs`
- **Role**: Startup/config edge-case tests (addr-in-use errors, discovery conflicts, NAT address handling).
- **Key items**: `test_listener_addr_in_use`, `test_discovery_addr_in_use`, `test_discv5_and_discv4_same_socket_fails`
- **Interactions**: Validates `NetworkConfigBuilder`, `Discovery`, and `NetworkManager` error paths.

### `transaction_hash_fetching.rs`
- **Role**: End-to-end tx hash fetching test across peers (ignored by default).
- **Key items**: `transaction_hash_fetching` (#[ignore]), `TransactionsManagerConfig`, `Testnet`
- **Interactions**: Uses pools and pending transaction listeners to confirm fetch completion.

### `txgossip.rs`
- **Role**: Transaction gossip and policy tests (propagation modes and ingress policies).
- **Key items**: `test_tx_gossip`, `test_tx_propagation_policy_trusted_only`, `test_tx_ingress_policy_trusted_only`
- **Interactions**: Exercises `TransactionsManagerConfig`, `TransactionPropagationKind`, `TransactionIngressPolicy`, and peer trust changes.

## End-to-end flow (high level)
- Build a `Testnet` with mock providers and pools, then connect peers.
- Drive requests or broadcasts through network handles and wait for events.
- Assert negotiation behavior, error handling, or policy effects based on the scenario.

## Key APIs (no snippets)
- `Testnet`, `NetworkEventStream`
- `FetchClient`, `HeadersClient`, `BodiesClient`
- `TransactionsManagerConfig`, `TransactionPropagationKind`, `TransactionIngressPolicy`
```

## File: crates/net/network/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration test suites for the network crate.

## Contents (one hop)
### Subdirectories
- [x] `it/` - Integration tests covering connections, requests, sessions, startup config, and tx gossip/fetching.

### Files
- (none)
```

## File: crates/net/network/AGENTS.md
```markdown
# network

## Purpose
`reth-network` crate: full P2P networking stack for Ethereum (discovery, sessions, peer management, request routing, transaction gossip, and network APIs).

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Criterion benchmarks for broadcast ingress and transaction hash fetching.
- [x] `docs/` - (skip: diagrams/docs only) mermaid diagrams for fetch client, swarm, and network manager.
- [x] `src/` - Core networking implementation: discovery, sessions, peers, state, request handlers, and transactions.
- [x] `tests/` - Integration tests for connections, requests, sessions, and gossip policies.

### Files
- `Cargo.toml` - Crate manifest with networking, discovery, and transaction-pool dependencies plus test-utils features.
  - **Key items**: features `serde`, `test-utils`; deps `reth-network-p2p`, `reth-discv4`, `reth-discv5`, `reth-dns-discovery`, `reth-eth-wire`, `reth-transaction-pool`
- `README.md` - Short crate-level description.
  - **Key items**: `RETH network implementation`

## Key APIs (no snippets)
- `NetworkManager`, `NetworkHandle`
- `NetworkConfig`, `NetworkBuilder`
- `PeersManager`, `SessionManager`
```

## File: crates/net/network-api/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Test/integration helpers for `reth-network-api`: provides a lightweight handle interface for interacting with a peer manager in tests (add/remove peers, reputation changes, and peer queries).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that exposes the peers-manager testing API types.
- **Key items**: `PeersHandle`, `PeersHandleProvider`, `PeerCommand`

### `peers_manager.rs`
- **Role**: Defines `PeersHandle` and command types used to drive `PeersManager` behavior from tests (manual peer set edits and lookups).
- **Key items**: `PeersHandleProvider`, `PeersHandle`, `PeerCommand`, `PeersHandle::add_peer()`, `remove_peer()`, `reputation_change()`, `peer_by_id()`, `all_peers()`
```

## File: crates/net/network-api/src/AGENTS.md
```markdown
# src

## Purpose
Defines the public "network surface" for reth: traits and common types that higher-level components use to interact with the networking subsystem (peer management, event streams, block download clients, discovery events), plus a noop implementation and testing helpers.

## Contents (one hop)
### Subdirectories
- [x] `test_utils/` - testing helpers for interacting with peers management in integration tests.

## Files (detailed)

### `lib.rs`
- **Role**: Main API surface: defines core network traits (info, peer management, event listeners, downloaders) and shared structs/enums for peer/session/network status.
- **Key items**: `FullNetwork`, `NetworkInfo`, `PeersInfo`, `Peers`, `PeerInfo`, `Direction`, `NetworkStatus`, `PeerId`
- **Interactions**: Implemented by `reth-network`'s concrete network manager; consumed by node wiring, RPC, and sync/pipeline orchestration.

### `downloaders.rs`
- **Role**: Trait for obtaining a `BlockClient` used to fetch blocks from peers (abstracts downloader/client wiring).
- **Key items**: `BlockDownloaderProvider`, `fetch_client()`

### `events.rs`
- **Role**: Event types and listener traits for peer lifecycle, discovery, and per-peer request routing (request/response channels for headers/bodies/txs/receipts).
- **Key items**: `PeerEvent`, `SessionInfo`, `NetworkEvent`, `PeerEventStream`, `NetworkEventListenerProvider`, `DiscoveryEvent`, `DiscoveredEvent`, `PeerRequest`
- **Knobs / invariants**: Many request variants are version-sensitive at the wire level (e.g. receipts eth/69 vs eth/70), but are exposed here as typed request/response channels.

### `error.rs`
- **Role**: Common error type for network API calls (primarily channel closure/oneshot failures).
- **Key items**: `NetworkError::ChannelClosed`

### `noop.rs`
- **Role**: `NoopNetwork` implementation that satisfies the main network traits while doing nothing, useful for tests and wiring generic components.
- **Key items**: `NoopNetwork`, `with_chain_id()`, impls for `NetworkInfo`, `Peers`, `BlockDownloaderProvider`, `NetworkEventListenerProvider`

## Key APIs (no snippets)
- **Traits**: `FullNetwork`, `NetworkInfo`, `Peers`, `NetworkEventListenerProvider`, `BlockDownloaderProvider`
- **Types**: `PeerInfo`, `NetworkStatus`, `PeerEvent` / `NetworkEvent`
```

## File: crates/net/network-api/AGENTS.md
```markdown
# network-api

## Purpose
`reth-network-api` crate: interfaces and common types that decouple reth's networking implementation from its consumers (node wiring, RPC, sync), including peer management traits, event stream types, downloader/client providers, and a noop implementation.

## Contents (one hop)
### Subdirectories
- [x] `src/` - network traits, events, error types, noop impl, and test utilities.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support for public types).
  - **Key items**: feature `serde`; deps `reth-network-types`, `reth-network-p2p`, `reth-eth-wire-types`

## Key APIs (no snippets)
- `FullNetwork`, `NetworkInfo`, `Peers`
- `NetworkEventListenerProvider`, `PeerEventStream`
- `BlockDownloaderProvider`
```

## File: crates/net/network-types/src/peers/AGENTS.md
```markdown
# peers

## Purpose
Peer-related shared types and configuration: describes peer identity/addressing, connection state, reputation scoring and outcomes, and configuration knobs for peering/backoff/ban lists.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Peer model root: defines `Peer` and re-exports peer config/state/reputation/kind/address types.
- **Key items**: `Peer`, `apply_reputation()`, `ReputationChangeOutcome`, `ConnectionsConfig`, `PeersConfig`

### `addr.rs`
- **Role**: Socket addressing for a peer (TCP + optional UDP) used by dialing and discovery integration.
- **Key items**: `PeerAddr`, `PeerAddr::tcp()`, `udp()`, `new_with_ports()`

### `kind.rs`
- **Role**: Classification of peers by trust/source (basic/static/trusted) with convenience predicates.
- **Key items**: `PeerKind`, `is_trusted()`, `is_static()`, `is_basic()`

### `state.rs`
- **Role**: Tracks current connection lifecycle state to a peer (idle/connected/pending/disconnecting).
- **Key items**: `PeerConnectionState`, `disconnect()`, `is_connected()`, `is_pending_out()`

### `reputation.rs`
- **Role**: Reputation scoring constants, change kinds/weights, and computed outcomes (ban/unban/disconnect) used to police peers and backoff decisions.
- **Key items**: `Reputation`, `ReputationChangeKind`, `ReputationChangeWeights`, `ReputationChangeOutcome`, `BANNED_REPUTATION`, `FAILED_TO_CONNECT_REPUTATION_CHANGE`, `MAX_TRUSTED_PEER_REPUTATION_CHANGE`
- **Knobs / invariants**: Banning threshold and per-kind weights are configurable; `BadProtocol` is maximal penalty.

### `config.rs`
- **Role**: Peering configuration for `PeersManager`: connection limits, backoff durations, ban durations/lists, trusted peers, and IP filtering.
- **Key items**: `PeersConfig`, `ConnectionsConfig`, `PeerBackoffDurations`, `DEFAULT_MAX_COUNT_PEERS_OUTBOUND`, `DEFAULT_MAX_COUNT_PEERS_INBOUND`, `DEFAULT_MAX_COUNT_CONCURRENT_OUTBOUND_DIALS`, `INBOUND_IP_THROTTLE_DURATION`
- **Knobs / invariants**: Backoff duration scales with counter and is capped by `max`; trusted peers are exempt from some drop/backoff behavior.

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`, `PeerConnectionState`
- `PeersConfig`, `ConnectionsConfig`, `PeerBackoffDurations`
- `ReputationChangeKind`, `ReputationChangeWeights`
```

## File: crates/net/network-types/src/session/AGENTS.md
```markdown
# session

## Purpose
Shared configuration types for peer session management: timeouts and buffering limits that govern how session tasks communicate with the session manager and how long the node waits before treating peers as timed out or in protocol breach.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that exports the main session config types.
- **Key items**: `SessionsConfig`, `SessionLimits`

### `config.rs`
- **Role**: Session manager configuration: request timeouts, protocol breach window, pending session timeout, per-session command buffer sizing, and scalable event buffering.
- **Key items**: `SessionsConfig`, `SessionLimits`, `INITIAL_REQUEST_TIMEOUT`, `PROTOCOL_BREACH_REQUEST_TIMEOUT`, `PENDING_SESSION_TIMEOUT`, `with_upscaled_event_buffer()`
- **Knobs / invariants**: Two-stage timeout model: internal request timeout vs a longer "protocol breach" ceiling after which a non-responsive peer is considered violating.
```

## File: crates/net/network-types/src/AGENTS.md
```markdown
# src

## Purpose
Shared "plumbing" types used across reth networking: peer model/state/configuration (addresses, kinds, reputation, backoff, bans) and session manager configuration/limits.

## Contents (one hop)
### Subdirectories
- [x] `peers/` - peer address/kind/state, reputation model, and peering configuration.
- [x] `session/` - session manager configuration: timeouts, buffers, and limits.

### Files
- `lib.rs` - crate entrypoint: exports the common peer/session types and config from submodules.
  - **Key items**: `Peer`, `PeerAddr`, `PeerKind`, `Reputation`, `PeersConfig`, `SessionsConfig`, `BackoffKind`
- `backoff.rs` - backoff severity classification used by peering logic.
  - **Key items**: `BackoffKind::{Low,Medium,High}`, `BackoffKind::is_severe()`

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`, `PeerConnectionState`
- `PeersConfig`, `SessionsConfig`
- `ReputationChangeKind`, `BackoffKind`
```

## File: crates/net/network-types/AGENTS.md
```markdown
# network-types

## Purpose
`reth-network-types` crate: commonly used networking types shared across reth (peer model and configuration, reputation/backoff, session limits/timeouts), intended to keep higher-level networking components decoupled from implementation details.

## Contents (one hop)
### Subdirectories
- [x] `src/` - peer/session/backoff types and configuration.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support for config types).
  - **Key items**: feature `serde`; deps `reth-network-peers`, `reth-net-banlist`, `alloy-eip2124`

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`
- `PeersConfig`, `ConnectionsConfig`
- `SessionsConfig`, `SessionLimits`
- `Reputation`, `ReputationChangeKind`, `BackoffKind`
```

## File: crates/net/p2p/src/bodies/AGENTS.md
```markdown
# bodies

## Purpose
Shared traits and types for block body downloads in the p2p layer, including client abstractions, downloader interface, and response wrapper.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for body download traits and response types.
- **Key items**: `client`, `downloader`, `response`

### `client.rs`
- **Role**: Defines the `BodiesClient` trait and helpers for fetching bodies with optional priority/range hints.
- **Key items**: `BodiesClient`, `BodiesFut`, `SingleBodyRequest`, `get_block_bodies_with_priority_and_range_hint()`
- **Interactions**: Extends `DownloadClient`; used by downloaders in `reth-downloaders` and full-block helpers.

### `downloader.rs`
- **Role**: Downloader trait for streaming body batches derived from headers.
- **Key items**: `BodyDownloader`, `BodyDownloaderResult`, `set_download_range()`

### `response.rs`
- **Role**: Wrapper for body responses, distinguishing empty vs full blocks with helpers.
- **Key items**: `BlockResponse`, `block_number()`, `difficulty()`, `into_body()`, `body()`
- **Interactions**: Implements `InMemorySize` for buffering/memory accounting.

## End-to-end flow (high level)
- A `BodiesClient` issues `GetBlockBodies` requests with optional priority and range hints.
- A `BodyDownloader` consumes headers, issues body requests via the client, and yields `BlockResponse` batches.

## Key APIs (no snippets)
- `BodiesClient`, `BodyDownloader`, `BlockResponse`
```

## File: crates/net/p2p/src/headers/AGENTS.md
```markdown
# headers

## Purpose
Shared header download abstractions for p2p: request types, client trait, downloader interface, and validation/error helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for header client, downloader, and error types.
- **Key items**: `client`, `downloader`, `error`

### `client.rs`
- **Role**: Defines `HeadersClient` and request types for fetching block headers with priority.
- **Key items**: `HeadersRequest`, `HeadersClient`, `HeadersFut`, `SingleHeaderRequest`, `HeadersDirection`

### `downloader.rs`
- **Role**: Header downloader trait and sync target helpers, plus validation utility.
- **Key items**: `HeaderDownloader`, `SyncTarget`, `HeaderSyncGap`, `validate_header_download()`
- **Interactions**: Uses `HeaderValidator` to validate parent linkage and standalone header rules.

### `error.rs`
- **Role**: Downloader error type for header validation failures (e.g., detached head).
- **Key items**: `HeadersDownloaderError`, `HeadersDownloaderResult`

## End-to-end flow (high level)
- A `HeadersClient` issues `HeadersRequest` with a direction and limit.
- A `HeaderDownloader` streams validated `SealedHeader` batches toward a `SyncTarget`.

## Key APIs (no snippets)
- `HeadersClient`, `HeaderDownloader`, `HeadersRequest`, `SyncTarget`
```

## File: crates/net/p2p/src/snap/AGENTS.md
```markdown
# snap

## Purpose
Traits for SNAP sync requests and response types in the p2p layer.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for SNAP client traits.
- **Key items**: `client`

### `client.rs`
- **Role**: Defines `SnapClient` and response variants for SNAP requests with priority support.
- **Key items**: `SnapClient`, `SnapResponse`, `get_account_range*()`, `get_storage_ranges*()`, `get_byte_codes*()`, `get_trie_nodes*()`

## End-to-end flow (high level)
- A `SnapClient` issues SNAP requests (accounts, storage, bytecodes, trie nodes) with optional priority.
- The client returns a `SnapResponse` variant for the requested data type.

## Key APIs (no snippets)
- `SnapClient`, `SnapResponse`
```

## File: crates/net/p2p/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Test helpers for p2p interfaces: mock clients for headers/bodies and a full-block in-memory client.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for test helpers.
- **Key items**: `bodies`, `headers`, `full_block`

### `bodies.rs`
- **Role**: Simple bodies client that returns responder-provided results.
- **Key items**: `TestBodiesClient`
- **Interactions**: Implements `DownloadClient` and `BodiesClient` for unit tests.

### `headers.rs`
- **Role**: Test header downloader and mock header client with queued responses and error injection.
- **Key items**: `TestHeaderDownloader`, `TestHeadersClient`
- **Interactions**: Implements `HeaderDownloader`, `HeadersClient`, and uses `RequestError`/`DownloadError` for failure paths.

### `full_block.rs`
- **Role**: In-memory full-block client storing headers and bodies with a soft body response limit.
- **Key items**: `TestFullBlockClient`
- **Interactions**: Implements `HeadersClient`, `BodiesClient`, and `BlockClient`.

## End-to-end flow (high level)
- Configure a test client (headers/bodies/full-block) with in-memory data or custom responder.
- Use the client in downloader or full-block tests to validate request/response behavior.

## Key APIs (no snippets)
- `TestBodiesClient`, `TestHeadersClient`, `TestHeaderDownloader`, `TestFullBlockClient`
```

## File: crates/net/p2p/src/AGENTS.md
```markdown
# src

## Purpose
P2P protocol abstractions for downloaders: request clients, error types, full-block fetch helpers, SNAP traits, and sync state interfaces.

## Contents (one hop)
### Subdirectories
- [x] `bodies/` - Body download client traits, downloader interface, and response wrapper.
- [x] `headers/` - Header request/client traits, downloader interface, and validation helpers.
- [x] `snap/` - SNAP sync client trait and response variants.
- [x] `test_utils/` - Mock clients and downloaders for tests.

### Files
- `download.rs` - Base trait for download clients used across bodies/headers/SNAP.
  - **Key items**: `DownloadClient`
- `either.rs` - `Either` adapter implementing `DownloadClient`, `BodiesClient`, and `HeadersClient` for two backends.
  - **Key items**: `Either` impls for `DownloadClient`/`BodiesClient`/`HeadersClient`
- `error.rs` - Common request/response error types and validation helpers.
  - **Key items**: `RequestError`, `DownloadError`, `EthResponseValidator`, `RequestResult`, `PeerRequestResult`
- `full_block.rs` - Full-block fetcher built from header/body clients with validation.
  - **Key items**: `FullBlockClient`, `FetchFullBlockFuture`, `FetchFullBlockRangeFuture`, `NoopFullBlockClient`
- `lib.rs` - Crate entrypoint and public re-exports.
  - **Key items**: modules `bodies`, `headers`, `snap`, `sync`, `full_block`; trait `BlockClient`
- `priority.rs` - Request priority enum used by clients and downloaders.
  - **Key items**: `Priority`
- `sync.rs` - Sync state traits and a noop updater.
  - **Key items**: `SyncStateProvider`, `NetworkSyncUpdater`, `SyncState`, `NoopSyncStateUpdater`

## Key APIs (no snippets)
- `BodiesClient`, `HeadersClient`, `SnapClient`
- `DownloadClient`, `RequestError`, `DownloadError`
- `FullBlockClient`, `BlockClient`, `NetworkSyncUpdater`
```

## File: crates/net/p2p/AGENTS.md
```markdown
# p2p

## Purpose
`reth-network-p2p` crate: shared traits and types for p2p download clients, errors, and sync state interfaces.

## Contents (one hop)
### Subdirectories
- [x] `src/` - P2P client traits, error types, full-block helpers, SNAP interfaces, and test utils.

### Files
- `Cargo.toml` - Crate manifest defining p2p trait dependencies and feature flags.
  - **Key items**: features `test-utils`, `std`; deps `reth-consensus`, `reth-network-types`, `reth-eth-wire-types`

## Key APIs (no snippets)
- `DownloadClient`, `BodiesClient`, `HeadersClient`, `SnapClient`
- `FullBlockClient`, `BlockClient`, `NetworkSyncUpdater`
```

## File: crates/net/peers/src/bootnodes/AGENTS.md
```markdown
# bootnodes

## Purpose
Bootnode lists and helpers for constructing initial peer sets for different networks (Ethereum mainnet/testnets and OP-stack networks).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - exposes bootnode lists and helpers that parse them into `NodeRecord` values.
  - **Key items**: `mainnet_nodes()`, `sepolia_nodes()`, `holesky_nodes()`, `hoodi_nodes()`, `op_nodes()`, `op_testnet_nodes()`, `parse_nodes()`
- `ethereum.rs` - Ethereum Foundation bootnode lists for mainnet and testnets.
  - **Key items**: `MAINNET_BOOTNODES`, `SEPOLIA_BOOTNODES`, `HOLESKY_BOOTNODES`, `HOODI_BOOTNODES`
- `optimism.rs` - OP-stack bootnode lists for mainnet/testnet (including Base endpoints).
  - **Key items**: `OP_BOOTNODES`, `OP_TESTNET_BOOTNODES`
```

## File: crates/net/peers/src/AGENTS.md
```markdown
# src

## Purpose
Implements `reth-network-peers`: peer identity and node record utilities for networking and RPC configuration, including parsing/displaying enode URLs, bootnode lists, and "trusted peer" records that can resolve DNS names to IPs.

## Contents (one hop)
### Subdirectories
- [x] `bootnodes/` - bootnode lists for Ethereum networks and OP-stack networks.

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `PeerId` alias, node-record types (`NodeRecord`, `TrustedPeer`, `AnyNode`), and helpers to convert between peer IDs and secp256k1 public keys (feature-gated).
- **Key items**: `PeerId`, `AnyNode`, `pk2id()` / `id2pk()` (feature `secp256k1`), `WithPeerId<T>`
- **Interactions**: `AnyNode` is used when accepting "any peer identifier" inputs (e.g. trusted peers via RPC).

#### `node_record.rs`
- **Role**: `NodeRecord` type (enode URL form) used by discovery v4 and related tooling: stores IP + tcp/udp ports + peer ID, supports parsing/display, and IPv4-mapped IPv6 normalization.
- **Key items**: `NodeRecord`, `NodeRecord::new()`, `new_with_ports()`, `tcp_addr()`, `udp_addr()`, `NodeRecordParseError`

#### `trusted_peer.rs`
- **Role**: `TrustedPeer` type for persistent/trusted peer configuration: supports domain names and can resolve them to IP addresses to produce a `NodeRecord`.
- **Key items**: `TrustedPeer`, `resolve_blocking()` (std/test), `resolve()` (feature `net`)

## Key APIs (no snippets)
- `PeerId`, `NodeRecord`, `TrustedPeer`, `AnyNode`
```

## File: crates/net/peers/AGENTS.md
```markdown
# peers

## Purpose
`reth-network-peers` crate: peer/node record types and utilities, including bootnode lists, `NodeRecord` (enode) parsing/display, `TrustedPeer` with optional DNS resolution, and flexible `AnyNode` parsing for APIs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - peer ID/node record/trusted peer types plus bootnode lists.

### Files
- `Cargo.toml` - crate manifest (feature-gated secp256k1 + async DNS lookup support).

## Key APIs (no snippets)
- `NodeRecord`
- `TrustedPeer`
- `AnyNode`
```

## File: crates/net/AGENTS.md
```markdown
# net

## Purpose
Networking subsystem crates for reth: peer discovery (discv4/discv5, DNS), cryptography (ECIES), wire protocols (eth-wire and types), peer management, P2P sync/downloaders, the network manager/swarm implementation, and supporting types/APIs.

## Contents (one hop)
### Subdirectories
- [x] `banlist/` - Peer/IP banning utilities: `BanList` with optional expiry and `IpFilter` for CIDR allow-lists.
- [x] `discv4/` - Discovery v4: UDP-based peer discovery with kbucket routing table, bonding/lookups, and NAT external IP detection (`Discv4`/`Discv4Service`).
- [x] `discv5/` - Discovery v5 wrapper: starts `sigp/discv5`, builds local ENR (fork kv-pairs), filters discovered peers, and emits reachable `NodeRecord`s (`Discv5`, `Config`).
- [x] `dns/` - EIP-1459 DNS discovery: verify ENR trees, rate-limit TXT lookups, and stream discovered `NodeRecord`s (`DnsDiscoveryService`, `DnsNodeRecordUpdate`).
- [x] `downloaders/` - Header/body downloaders plus file import clients and metrics (`ReverseHeadersDownloader`, `BodiesDownloader`, `FileClient`, `ChunkedFileReader`).
- [x] `ecies/` - RLPx ECIES framed transport: AUTH/ACK handshake, encrypted header/body framing, and Tokio codec/stream wrappers (`ECIESStream`, `ECIESCodec`).
- [x] `eth-wire/` - RLPx `p2p` + `eth` stream plumbing: hello/capability negotiation, message multiplexing, ping/pong, and typed `EthStream`/`P2PStream` over ECIES (`RlpxProtocolMultiplexer`).
- [x] `eth-wire-types/` - Devp2p `eth`/SNAP payload types and versioned decoding/encoding (eth/66-70): messages, IDs, status, requests/responses, broadcasts (`EthMessage`, `EthVersion`).
- [x] `nat/` - NAT helpers: resolve external/public IP via public IP services, domain/net-if, and interval-based polling (`NatResolver`, `ResolveNatInterval`).
- [x] `network/` - Full network stack: discovery, sessions, peers, request routing, tx gossip, and network handle APIs (`NetworkManager`, `NetworkHandle`).
- [x] `network-api/` - Network interfaces/types: peer management + event streams + downloader providers and a noop impl (`FullNetwork`, `Peers`, `NetworkEvent`).
- [x] `network-types/` - Shared networking config/types: peers (addr/kind/state/reputation/backoff) and session limits/timeouts (`PeersConfig`, `ReputationChangeKind`, `SessionsConfig`).
- [x] `p2p/` - Shared p2p traits/types for download clients, errors, and sync state (`DownloadClient`, `HeadersClient`, `BodiesClient`).
- [x] `peers/` - Peer record utilities: `PeerId`/`NodeRecord`/`TrustedPeer`/`AnyNode` parsing and bootnode lists for Ethereum and OP-stack networks.

### Files
- (none)
```

## File: crates/node/api/src/AGENTS.md
```markdown
# src

## Purpose
Node API traits and configuration interfaces for assembling a full reth node with pluggable components and add-ons.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint that re-exports engine/payload/payload-builder/EVM abstractions and node types.
  - **Key items**: re-exports `reth_engine_primitives`, `reth_payload_primitives`, `reth_payload_builder_primitives`, `ConfigureEvm`, `FullProvider`
- `node.rs` - Core node configuration traits and add-on interface.
  - **Key items**: `FullNodeTypes`, `FullNodeTypesAdapter`, `FullNodeComponents`, `PayloadBuilderFor`, `NodeAddOns`, `AddOnsContext`

## Key APIs (no snippets)
- `FullNodeTypes`, `FullNodeComponents`
- `NodeAddOns`, `AddOnsContext`
```

## File: crates/node/api/AGENTS.md
```markdown
# api

## Purpose
`reth-node-api` crate: shared traits and abstractions for configuring node components, payload builders, and add-on services.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Node API traits and re-exports for engine/payload/EVM primitives.

### Files
- `Cargo.toml` - Manifest defining dependencies for node component traits and add-ons.
  - **Key items**: deps `reth-node-types`, `reth-network-api`, `reth-transaction-pool`, `reth-engine-primitives`
```

## File: crates/node/builder/src/builder/AGENTS.md
```markdown
# builder

## Purpose
Node builder state machine: types and helpers that progressively configure node types, components, add-ons, and launch context.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Main `NodeBuilder` implementation and builder flow utilities.
  - **Key items**: `NodeBuilder`, `WithLaunchContext`, `BuilderContext`, `RethFullAdapter`
- `states.rs` - Builder state types and adapters for configured types/components.
  - **Key items**: `NodeBuilderWithTypes`, `NodeTypesAdapter`, `NodeAdapter`, `NodeBuilderWithComponents`
- `add_ons.rs` - Container for node add-ons, hooks, and execution extensions.
  - **Key items**: `AddOns`

## Key APIs (no snippets)
- `NodeBuilder`
- `NodeBuilderWithComponents`
```

## File: crates/node/builder/src/components/AGENTS.md
```markdown
# components

## Purpose
Component builder traits and defaults for assembling node subsystems (pool, network, payload service, consensus, EVM).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and shared `NodeComponents` trait and `Components` container.
  - **Key items**: `NodeComponents`, `Components`
- `builder.rs` - Generic `ComponentsBuilder` with default/noop component builders.
  - **Key items**: `ComponentsBuilder`, `NodeComponentsBuilder`, `NoopTransactionPoolBuilder`, `NoopNetworkBuilder`, `NoopConsensusBuilder`, `NoopPayloadBuilder`
- `consensus.rs` - Consensus builder trait.
  - **Key items**: `ConsensusBuilder`
- `execute.rs` - EVM/executor builder trait.
  - **Key items**: `ExecutorBuilder`
- `network.rs` - Network builder trait.
  - **Key items**: `NetworkBuilder`
- `payload.rs` - Payload service builder traits and basic/noop implementations.
  - **Key items**: `PayloadServiceBuilder`, `PayloadBuilderBuilder`, `BasicPayloadServiceBuilder`, `NoopPayloadServiceBuilder`
- `pool.rs` - Transaction pool builder trait and helper builders.
  - **Key items**: `PoolBuilder`, `PoolBuilderConfigOverrides`, `TxPoolBuilder`

## Key APIs (no snippets)
- `NodeComponents`, `ComponentsBuilder`
- `PoolBuilder`, `NetworkBuilder`, `PayloadServiceBuilder`
```

## File: crates/node/builder/src/launch/AGENTS.md
```markdown
# launch

## Purpose
Node launch orchestration: type-state launch context, engine/debug launchers, execution extensions, and invalid-block hooks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Launch trait and module wiring.
  - **Key items**: `LaunchNode`
- `common.rs` - Type-state launch context with attachments for configs, providers, and components.
  - **Key items**: `LaunchContext`, `LaunchContextWith`, `Attached`, `WithConfigs`, `WithComponents`
- `engine.rs` - Main engine node launcher integrating pipeline, network, and engine APIs.
  - **Key items**: `EngineNodeLauncher`
- `debug.rs` - Debug launcher that wraps a launcher to add RPC/Etherscan consensus clients.
  - **Key items**: `DebugNode`, `DebugNodeLauncher`, `DebugNodeLauncherFuture`
- `exex.rs` - Execution extension launcher that wires ExEx manager and WAL.
  - **Key items**: `ExExLauncher`
- `invalid_block_hook.rs` - Invalid block hook creation helpers.
  - **Key items**: `InvalidBlockHookExt`, `create_invalid_block_hook()`

## Key APIs (no snippets)
- `LaunchContext`, `LaunchNode`
- `EngineNodeLauncher`, `DebugNodeLauncher`, `ExExLauncher`
```

## File: crates/node/builder/src/AGENTS.md
```markdown
# src

## Purpose
Node builder implementation: type-safe component assembly, launch orchestration, RPC wiring, hooks, and execution extensions.

## Contents (one hop)
### Subdirectories
- [x] `builder/` - Node builder state machine and add-on containers.
- [x] `components/` - Component builder traits for pool/network/payload/consensus/EVM.
- [x] `launch/` - Launch contexts and engine/debug/ExEx launchers.

### Files
- `lib.rs` - Crate entrypoint and re-exports for builder, components, launch, and RPC helpers.
  - **Key items**: `NodeBuilder`, `EngineNodeLauncher`, `DebugNodeLauncher`, `NodeHandle`
- `node.rs` - Node trait presets and `FullNode` wrapper with RPC handles.
  - **Key items**: `Node`, `AnyNode`, `FullNode`, `ComponentsFor`
- `rpc.rs` - RPC add-ons, hooks, and builders for Engine API and Eth API.
  - **Key items**: `RethRpcServerHandles`, `RpcHooks`, `RpcRegistry`, `RethRpcAddOns`
- `setup.rs` - Pipeline and downloader setup helpers.
  - **Key items**: `build_pipeline()`, `build_networked_pipeline()`
- `hooks.rs` - Hook containers for component initialization and node start.
  - **Key items**: `NodeHooks`, `OnComponentInitializedHook`, `OnNodeStartedHook`
- `handle.rs` - `NodeHandle` wrapper with node components and exit future.
  - **Key items**: `NodeHandle`
- `exex.rs` - Traits for launching execution extensions (ExEx).
  - **Key items**: `LaunchExEx`, `BoxedLaunchExEx`, `BoxExEx`
- `engine_api_ext.rs` - Wrapper to capture built Engine API instances.
  - **Key items**: `EngineApiExt`
- `aliases.rs` - Trait alias for block reader bounds.
  - **Key items**: `BlockReaderFor`

## Key APIs (no snippets)
- `NodeBuilder`, `NodeHandle`, `FullNode`
- `RethRpcAddOns`, `RpcRegistry`
```

## File: crates/node/builder/AGENTS.md
```markdown
# builder

## Purpose
`reth-node-builder` crate: declarative, type-safe node builder with component wiring, launch orchestration, and RPC add-ons.

## Contents (one hop)
### Subdirectories
- [x] `docs/` - (skip: diagrams/docs only) mermaid diagram for builder flow.
- [x] `src/` - Builder core, component traits, launchers, and RPC wiring.

### Files
- `Cargo.toml` - Manifest with node builder dependencies and feature flags.
  - **Key items**: features `js-tracer`, `test-utils`, `op`; deps `reth-network`, `reth-rpc`, `reth-engine-tree`
- `README.md` - Short crate description and example pointer.

## Key APIs (no snippets)
- `NodeBuilder`, `NodeComponentsBuilder`
- `EngineNodeLauncher`, `DebugNodeLauncher`
```

## File: crates/node/core/src/args/AGENTS.md
```markdown
# args

## Purpose
CLI argument structs and parsing helpers for configuring node subsystems (network, RPC, engine, DB, pruning, tracing, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and re-exports for CLI argument groups.
  - **Key items**: `NetworkArgs`, `RpcServerArgs`, `TxPoolArgs`, `PayloadBuilderArgs`, `EngineArgs`, `PruningArgs`
- `benchmark_args.rs` - Benchmark CLI arguments.
  - **Key items**: `BenchmarkArgs`
- `database.rs` - Database CLI configuration and size parsing.
  - **Key items**: `DatabaseArgs`, `ByteSize`
- `datadir_args.rs` - Data directory CLI arguments.
  - **Key items**: `DatadirArgs`
- `debug.rs` - Debug CLI options and invalid-block hook selection.
  - **Key items**: `DebugArgs`, `InvalidBlockSelection`, `InvalidBlockHookType`
- `dev.rs` - Dev-mode CLI configuration.
  - **Key items**: `DevArgs`
- `engine.rs` - Engine CLI configuration.
  - **Key items**: `EngineArgs`, `DefaultEngineValues`
- `era.rs` - ERA import CLI arguments and source selection.
  - **Key items**: `EraArgs`, `EraSourceArgs`, `DefaultEraHost`
- `error.rs` - Parsing errors for receipt log prune configs.
  - **Key items**: `ReceiptsLogError`
- `gas_price_oracle.rs` - Gas price oracle CLI arguments.
  - **Key items**: `GasPriceOracleArgs`
- `log.rs` - Logging CLI configuration.
  - **Key items**: `LogArgs`, `ColorMode`, `Verbosity`
- `metric.rs` - Metrics CLI configuration.
  - **Key items**: `MetricArgs`
- `network.rs` - Network and discovery CLI configuration.
  - **Key items**: `NetworkArgs`, `DiscoveryArgs`
- `payload_builder.rs` - Payload builder CLI configuration.
  - **Key items**: `PayloadBuilderArgs`, `DefaultPayloadBuilderValues`
- `pruning.rs` - Pruning CLI configuration.
  - **Key items**: `PruningArgs`
- `ress_args.rs` - RESS subprotocol CLI configuration.
  - **Key items**: `RessArgs`
- `rpc_server.rs` - RPC server CLI configuration.
  - **Key items**: `RpcServerArgs`, `DefaultRpcServerArgs`
- `rpc_state_cache.rs` - RPC state cache CLI configuration.
  - **Key items**: `RpcStateCacheArgs`
- `stage.rs` - Stage selection CLI enum.
  - **Key items**: `StageEnum`
- `static_files.rs` - Static file CLI configuration and defaults.
  - **Key items**: `StaticFilesArgs`, `MINIMAL_BLOCKS_PER_FILE`
- `trace.rs` - Tracing/OTLP CLI configuration.
  - **Key items**: `TraceArgs`, `OtlpInitStatus`, `OtlpLogsStatus`
- `txpool.rs` - Transaction pool CLI configuration.
  - **Key items**: `TxPoolArgs`, `DefaultTxPoolValues`
- `types.rs` - CLI parsing helper types (zero-as-none, max).
  - **Key items**: `ZeroAsNoneU64`, `ZeroAsNoneU32`, `MaxU64`, `MaxU32`, `MaxOr`

## Key APIs (no snippets)
- `NetworkArgs`, `RpcServerArgs`, `TxPoolArgs`, `PayloadBuilderArgs`
```

## File: crates/node/core/src/cli/AGENTS.md
```markdown
# cli

## Purpose
Core CLI configuration traits for node components (payload builder, network, tx pool).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring for CLI config traits.
  - **Key items**: `config`
- `config.rs` - Traits for component configuration used by CLI args.
  - **Key items**: `PayloadBuilderConfig`, `RethNetworkConfig`, `RethTransactionPoolConfig`

## Key APIs (no snippets)
- `PayloadBuilderConfig`, `RethNetworkConfig`, `RethTransactionPoolConfig`
```

## File: crates/node/core/src/AGENTS.md
```markdown
# src

## Purpose
Core node utilities and configuration: CLI argument types, config helpers, directory paths, version metadata, and startup/shutdown utilities.

## Contents (one hop)
### Subdirectories
- [x] `args/` - CLI argument groups for node subsystems.
- [x] `cli/` - CLI configuration traits for payload builder, network, and txpool.

### Files
- `lib.rs` - Crate entrypoint and module exports; re-exports primitives and RPC compat helpers.
  - **Key items**: modules `args`, `cli`, `dirs`, `node_config`, `utils`, `version`
- `node_config.rs` - `NodeConfig` type and helpers for building full node configuration.
  - **Key items**: `NodeConfig`, `DEFAULT_CROSS_BLOCK_CACHE_SIZE_MB`
- `dirs.rs` - Data directory and XDG path helpers.
  - **Key items**: `data_dir()`, `PlatformPath`, `ChainPath`, `MaybePlatformPath`
- `exit.rs` - Future for waiting on node shutdown conditions.
  - **Key items**: `NodeExitFuture`
- `utils.rs` - Startup helpers (path parsing, JWT secret handling) and single header/body fetch.
  - **Key items**: `parse_path()`, `get_or_create_jwt_secret_from_path()`, `get_single_header()`, `get_single_body()`
- `version.rs` - Version metadata constants and helpers.
  - **Key items**: `RethCliVersionConsts`, `version_metadata()`, `default_extra_data()`

## Key APIs (no snippets)
- `NodeConfig`
- `PlatformPath`, `ChainPath`
- `RethCliVersionConsts`
```

## File: crates/node/core/AGENTS.md
```markdown
# core

## Purpose
`reth-node-core` crate: core node configuration, CLI args, directory management, and version/build metadata utilities.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Node config types, CLI args, and core utilities.

### Files
- `Cargo.toml` - Manifest with core node dependencies and build features.
  - **Key items**: deps `reth-network`, `reth-transaction-pool`, `reth-config`; features `otlp`, `tracy`, log level gates
- `build.rs` - Build-time version metadata generation using vergen.
  - **Key items**: `VERGEN_*` envs, `RETH_SHORT_VERSION`, `RETH_LONG_VERSION_*`

## Key APIs (no snippets)
- `NodeConfig`
```

## File: crates/node/ethstats/src/AGENTS.md
```markdown
# src

## Purpose
EthStats client implementation: websocket connection management, protocol message types, and the main reporting service.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and public re-exports for EthStats service and message types.
  - **Key items**: `ethstats`, `events`
- `ethstats.rs` - Main EthStats service: connects, authenticates, reports stats, and streams block/tx updates.
  - **Key items**: `EthStatsService`, `HISTORY_UPDATE_RANGE`, `REPORT_INTERVAL`, `connect()`, `login()`
- `connection.rs` - WebSocket connection wrapper with JSON read/write helpers.
  - **Key items**: `ConnWrapper`, `WsStream`, `read_json()`, `write_json()`
- `credentials.rs` - Parses and stores EthStats connection credentials.
  - **Key items**: `EthstatsCredentials`
- `events.rs` - EthStats protocol message structures for node/block/pending/stats updates.
  - **Key items**: `NodeInfo`, `AuthMsg`, `BlockMsg`, `HistoryMsg`, `PendingMsg`, `StatsMsg`, `LatencyMsg`
- `error.rs` - Connection and service error types.
  - **Key items**: `ConnectionError`, `EthStatsError`

## Key APIs (no snippets)
- `EthStatsService`
- `EthStatsError`
```

## File: crates/node/ethstats/AGENTS.md
```markdown
# ethstats

## Purpose
`reth-node-ethstats` crate: EthStats client service for reporting node and chain stats over WebSocket.

## Contents (one hop)
### Subdirectories
- [x] `src/` - EthStats service, protocol message types, and websocket helpers.

### Files
- `Cargo.toml` - Manifest for EthStats networking and serialization dependencies.
  - **Key items**: deps `reth-network-api`, `reth-chain-state`, `tokio-tungstenite`, `serde`
```

## File: crates/node/events/src/AGENTS.md
```markdown
# src

## Purpose
Node event handlers for logging and health monitoring, including consensus-layer health checks and pipeline progress reporting.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring for node and consensus-layer event handlers.
  - **Key items**: `cl`, `node`
- `node.rs` - Processes pipeline/network/engine events and emits periodic status logs.
  - **Key items**: `NodeState`, `INFO_MESSAGE_INTERVAL`, event handlers for `PipelineEvent` and `ConsensusEngineEvent`
- `cl.rs` - Consensus Layer health event stream with periodic checks.
  - **Key items**: `ConsensusLayerHealthEvents`, `ConsensusLayerHealthEvent`, `CHECK_INTERVAL`

## Key APIs (no snippets)
- `ConsensusLayerHealthEvents`
- `NodeState` (internal event handler)
```

## File: crates/node/events/AGENTS.md
```markdown
# events

## Purpose
`reth-node-events` crate: event streams and handlers for node lifecycle reporting and consensus-layer health monitoring.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Node event processing and CL health streams.

### Files
- `Cargo.toml` - Manifest for node event handling dependencies.
  - **Key items**: deps `reth-stages`, `reth-network-api`, `reth-engine-primitives`, `reth-storage-api`
```

## File: crates/node/metrics/src/AGENTS.md
```markdown
# src

## Purpose
Node metrics utilities: Prometheus recorder setup, metrics server, and metric hooks for chain/version info and system stats.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and re-exports of Prometheus/process metrics crates.
  - **Key items**: `chain`, `hooks`, `recorder`, `server`, `version`
- `chain.rs` - Exposes chain-spec info as Prometheus metrics.
  - **Key items**: `ChainSpecInfo`, `register_chain_spec_metrics()`
- `hooks.rs` - Hook registry for periodic metric collectors (process, memory, IO).
  - **Key items**: `Hook`, `HooksBuilder`, `Hooks`
- `recorder.rs` - Installs Prometheus recorder and spawns upkeep for the global metrics recorder.
  - **Key items**: `PrometheusRecorder`, `install_prometheus_recorder()`, `spawn_upkeep()`
- `server.rs` - Metrics HTTP server and optional push gateway task.
  - **Key items**: `MetricServer`, `MetricServerConfig`, `serve()`
- `version.rs` - Exposes build/version info as Prometheus metrics.
  - **Key items**: `VersionInfo`, `register_version_metrics()`

## Key APIs (no snippets)
- `MetricServer`, `MetricServerConfig`
- `PrometheusRecorder`
```

## File: crates/node/metrics/AGENTS.md
```markdown
# metrics

## Purpose
`reth-node-metrics` crate: Prometheus recorder utilities and a metrics HTTP server for node metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Metrics recorder setup, hooks, and HTTP server.

### Files
- `Cargo.toml` - Manifest for metrics recorder/server and optional jemalloc/profiling features.
  - **Key items**: features `jemalloc`, `jemalloc-prof`; deps `metrics-exporter-prometheus`, `jsonrpsee-server`
```

## File: crates/node/types/src/AGENTS.md
```markdown
# src

## Purpose
Defines core `NodeTypes` traits and adapters used to describe the primitive types of a reth node.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Core `NodeTypes` traits and adapters plus helper type aliases.
  - **Key items**: `NodeTypes`, `NodeTypesWithDB`, `NodeTypesWithDBAdapter`, `AnyNodeTypes`, `AnyNodeTypesWithEngine`, `BlockTy`, `TxTy`

## Key APIs (no snippets)
- `NodeTypes`, `NodeTypesWithDB`
- `AnyNodeTypes`, `AnyNodeTypesWithEngine`
```

## File: crates/node/types/AGENTS.md
```markdown
# types

## Purpose
`reth-node-types` crate: core type traits and adapters for configuring node primitives.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `NodeTypes` traits, adapters, and helper type aliases.

### Files
- `Cargo.toml` - Manifest for node types and primitives dependencies.
  - **Key items**: deps `reth-primitives-traits`, `reth-engine-primitives`, `reth-chainspec`; feature `std`
```

## File: crates/node/AGENTS.md
```markdown
# node

## Purpose
Node-level crates: builder and core configuration utilities, node APIs/types, metrics/events, and optional EthStats reporting.

## Contents (one hop)
### Subdirectories
- [x] `api/` - Node component traits and add-on interfaces (`FullNodeComponents`, `NodeAddOns`).
- [x] `builder/` - Type-safe node builder, launchers, and RPC wiring (`NodeBuilder`, `EngineNodeLauncher`).
- [x] `core/` - Core node config, CLI args, paths, and version metadata (`NodeConfig`).
- [x] `ethstats/` - EthStats client service and protocol message types (`EthStatsService`).
- [x] `events/` - Node event handlers and CL health event stream (`ConsensusLayerHealthEvents`).
- [x] `metrics/` - Prometheus recorder and metrics server utilities (`MetricServer`).
- [x] `types/` - Node type traits and adapters (`NodeTypes`).

### Files
- (none)

## Notes
- This directory aggregates node-focused crates; details are in each subcrate's `AGENTS.md`.
```

## File: crates/optimism/bin/src/AGENTS.md
```markdown
# src

## Purpose
op-reth binary entrypoint and re-export surface for OP crates.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Re-exports op-reth CLI and subsystem crates.
  - **Key items**: modules `cli`, `chainspec`, `consensus`, `evm`, `node`, `rpc`
- `main.rs` - Binary entrypoint that parses CLI and launches node.
  - **Key items**: `Cli::run()`, `OpNode::new()`

## Key APIs (no snippets)
- `op-reth` binary entrypoint
```

## File: crates/optimism/bin/AGENTS.md
```markdown
# bin

## Purpose
`op-reth` binary crate: CLI entrypoint and feature wiring for op-reth.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Binary entrypoint and re-exports.

### Files
- `Cargo.toml` - Manifest with op-reth binary dependencies and features.
  - **Key items**: features `jemalloc`, `js-tracer`, `keccak-cache-global`, `asm-keccak`
```

## File: crates/optimism/chainspec/src/superchain/AGENTS.md
```markdown
# superchain

## Purpose
Superchain registry support: generate chain specs from bundled superchain configs and genesis files.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring for superchain registry support.
  - **Key items**: re-exports from `chain_specs`
- `chain_specs.rs` - Generated list of superchain (name, environment) pairs.
  - **Key items**: `create_superchain_specs!` invocation
- `configs.rs` - Reads and decompresses genesis/config data from bundled tar archive.
  - **Key items**: `read_superchain_genesis()`
- `chain_metadata.rs` - Metadata structs and conversion to genesis chain config.
  - **Key items**: `ChainMetadata`, `to_genesis_chain_config()`
- `chain_spec_macro.rs` - Macros to create chain specs and superchain enums.
  - **Key items**: `create_chain_spec!`, `create_superchain_specs!`

## Key APIs (no snippets)
- `Superchain` (generated enum), `read_superchain_genesis()`
```

## File: crates/optimism/chainspec/src/AGENTS.md
```markdown
# src

## Purpose
Optimism chain specs and base fee utilities, including Base/OP mainnet & testnet specs.

## Contents (one hop)
### Subdirectories
- [x] `superchain/` - Superchain registry-derived chain specs (feature-gated).

### Files
- `lib.rs` - Crate entrypoint and `OpChainSpec` builder/types.
  - **Key items**: `OpChainSpecBuilder`, `OpChainSpec`, `BASE_MAINNET`, `OP_MAINNET`, `OP_SEPOLIA`, `BASE_SEPOLIA`, `OP_DEV`
- `constants.rs` - OP/Base chain constants and dev L1 info tx.
  - **Key items**: `BASE_MAINNET_MAX_GAS_LIMIT`, `BASE_SEPOLIA_MAX_GAS_LIMIT`, `TX_SET_L1_BLOCK_OP_MAINNET_BLOCK_124665056`
- `base.rs` - Base mainnet chain spec.
  - **Key items**: `BASE_MAINNET`
- `base_sepolia.rs` - Base Sepolia chain spec.
  - **Key items**: `BASE_SEPOLIA`
- `op.rs` - Optimism mainnet chain spec.
  - **Key items**: `OP_MAINNET`
- `op_sepolia.rs` - Optimism Sepolia chain spec.
  - **Key items**: `OP_SEPOLIA`
- `dev.rs` - Dev chain spec with prefunded accounts.
  - **Key items**: `OP_DEV`
- `basefee.rs` - Base fee calculation helpers for Holocene/Jovian.
  - **Key items**: `decode_holocene_base_fee()`, `compute_jovian_base_fee()`

## Key APIs (no snippets)
- `OpChainSpec`, `OpChainSpecBuilder`
```

## File: crates/optimism/chainspec/AGENTS.md
```markdown
# chainspec

## Purpose
`reth-optimism-chainspec` crate: Optimism/Base chain specs, hardfork scheduling, and superchain registry integration.

## Contents (one hop)
### Subdirectories
- [x] `res/` - (skip: data/assets only) genesis JSONs and superchain config tarball.
- [x] `src/` - Chain spec builders, base fee utilities, and superchain helpers.

### Files
- `Cargo.toml` - Manifest for chain spec and superchain config features.
  - **Key items**: features `superchain-configs`, `serde`
```

## File: crates/optimism/cli/src/commands/AGENTS.md
```markdown
# commands

## Purpose
Optimism CLI commands for node launch, imports, init-state, and test vectors.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - CLI command enum wiring for op-reth.
  - **Key items**: `Commands`
- `import.rs` - Import pre-Bedrock OP blocks from file.
  - **Key items**: `ImportOpCommand`
- `import_receipts.rs` - Import OP receipts from file into static files.
  - **Key items**: `ImportReceiptsOpCommand`, `import_receipts_from_reader()`
- `init_state.rs` - Initialize state from dump with optional OVM scaffolding.
  - **Key items**: `InitStateCommandOp`
- `test_vectors.rs` - (feature `dev`) Generate OP test vectors.
  - **Key items**: `Command`, `Subcommands`

## Key APIs (no snippets)
- `Commands`, `ImportOpCommand`, `ImportReceiptsOpCommand`
```

## File: crates/optimism/cli/src/AGENTS.md
```markdown
# src

## Purpose
Optimism CLI wiring: command parsing, chain spec selection, and import codecs.

## Contents (one hop)
### Subdirectories
- [x] `commands/` - CLI subcommands for node, imports, init-state, and test vectors.

### Files
- `lib.rs` - CLI entrypoint and `Cli` struct for op-reth.
  - **Key items**: `Cli`, `Cli::run()`, re-exports `CliApp`, `ImportOpCommand`
- `app.rs` - CLI runner wrapper that initializes tracing and executes commands.
  - **Key items**: `CliApp`
- `chainspec.rs` - Optimism chain spec parser and value parser.
  - **Key items**: `OpChainSpecParser`, `chain_value_parser()`
- `ovm_file_codec.rs` - OVM block/transaction file codec for pre-bedrock imports.
  - **Key items**: `OvmBlockFileCodec`, `OvmBlock`, `OvmTransactionSigned`
- `receipt_file_codec.rs` - Receipt file codec for op-geth export format.
  - **Key items**: `OpGethReceiptFileCodec`, `OpGethReceipt`

## Key APIs (no snippets)
- `Cli`, `CliApp`
- `OpChainSpecParser`
```

## File: crates/optimism/cli/AGENTS.md
```markdown
# cli

## Purpose
`reth-optimism-cli` crate: op-reth command-line interface with custom commands and import codecs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - CLI parsing, commands, and file codecs.

### Files
- `Cargo.toml` - Manifest for op-reth CLI dependencies and features.
  - **Key items**: features `dev`, `otlp`, `jemalloc`; deps `reth-cli-commands`, `reth-optimism-node`
```

## File: crates/optimism/consensus/src/validation/AGENTS.md
```markdown
# validation

## Purpose
Optimism-specific consensus validation helpers for hardforks and post-execution checks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Shared validation helpers and receipt/root verification.
  - **Key items**: `validate_body_against_header_op()`, `validate_block_post_execution()`
- `canyon.rs` - Canyon hardfork checks for empty withdrawals.
  - **Key items**: `ensure_empty_withdrawals_root()`, `ensure_empty_shanghai_withdrawals()`
- `isthmus.rs` - Isthmus hardfork checks for L2 withdrawals storage root.
  - **Key items**: `ensure_withdrawals_storage_root_is_some()`, `verify_withdrawals_root()`

## Key APIs (no snippets)
- `validate_block_post_execution`
```

## File: crates/optimism/consensus/src/AGENTS.md
```markdown
# src

## Purpose
Optimism consensus implementation with OP hardfork-specific validation and receipt root handling.

## Contents (one hop)
### Subdirectories
- [x] `validation/` - Hardfork-specific validation helpers and post-exec checks.

### Files
- `lib.rs` - Main Optimism consensus type and validation wiring.
  - **Key items**: `OpBeaconConsensus`, `calculate_receipt_root_no_memo_optimism`
- `error.rs` - Optimism consensus errors.
  - **Key items**: `OpConsensusError`
- `proof.rs` - Receipt root calculation helpers with Regolith/Canyon handling.
  - **Key items**: `calculate_receipt_root_optimism()`, `calculate_receipt_root_no_memo_optimism()`

## Key APIs (no snippets)
- `OpBeaconConsensus`, `OpConsensusError`
```

## File: crates/optimism/consensus/AGENTS.md
```markdown
# consensus

## Purpose
`reth-optimism-consensus` crate: OP consensus rules, hardfork validation, and receipt root handling.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Consensus implementation and validation helpers.

### Files
- `Cargo.toml` - Manifest for OP consensus dependencies.
  - **Key items**: deps `reth-consensus`, `reth-optimism-forks`, `reth-optimism-primitives`; feature `std`
```

## File: crates/optimism/evm/src/AGENTS.md
```markdown
# src

## Purpose
Optimism EVM configuration and block execution helpers, including L1 info parsing and OP block assembly.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and `OpEvmConfig` implementation.
  - **Key items**: `OpEvmConfig`, `OpBlockAssembler`, `OpRethReceiptBuilder`, `OpBlockExecutionError`
- `config.rs` - OP next-block env attributes and revm spec helpers.
  - **Key items**: `OpNextBlockEnvAttributes`, `revm_spec`, `revm_spec_by_timestamp_after_bedrock`
- `execute.rs` - Executor helpers and backwards-compatible provider alias.
  - **Key items**: `OpExecutorProvider`
- `l1.rs` - L1 block info parsing for Bedrock/Ecotone/Isthmus/Jovian.
  - **Key items**: `extract_l1_info()`, `parse_l1_info_tx_ecotone()`, `parse_l1_info_tx_isthmus()`
- `receipts.rs` - Receipt builder for OP transactions.
  - **Key items**: `OpRethReceiptBuilder`
- `error.rs` - OP execution error types.
  - **Key items**: `L1BlockInfoError`, `OpBlockExecutionError`
- `build.rs` - OP block assembler implementation.
  - **Key items**: `OpBlockAssembler`

## Key APIs (no snippets)
- `OpEvmConfig`, `OpBlockAssembler`
- `OpRethReceiptBuilder`, `OpNextBlockEnvAttributes`
```

## File: crates/optimism/evm/AGENTS.md
```markdown
# evm

## Purpose
`reth-optimism-evm` crate: OP-specific EVM configuration, execution helpers, and block assembly.

## Contents (one hop)
### Subdirectories
- [x] `src/` - EVM config, L1 info parsing, receipt building, and block assembly.

### Files
- `Cargo.toml` - Manifest for OP EVM and execution dependencies.
  - **Key items**: deps `reth-evm`, `op-alloy-evm`, `reth-optimism-consensus`; features `portable`, `rpc`
```

## File: crates/optimism/flashblocks/src/ws/AGENTS.md
```markdown
# ws

## Purpose
WebSocket client utilities for streaming flashblocks and decoding payloads.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for websocket stream and decoder types.
- **Key items**: `WsFlashBlockStream`, `WsConnect`, `FlashBlockDecoder`
- **Interactions**: Re-exports `decoding` and `stream` modules.

### `decoding.rs`
- **Role**: Decoder trait and helpers to parse flashblock payloads from bytes.
- **Key items**: `FlashBlockDecoder`, `decode_flashblock()`, `try_parse_message()`
- **Interactions**: Used by `WsFlashBlockStream` to decode incoming frames.
- **Knobs / invariants**: Accepts JSON payloads directly or brotli-decompresses non-JSON frames.

### `stream.rs`
- **Role**: Stateful websocket stream that connects, decodes messages, and handles ping/pong.
- **Key items**: `WsFlashBlockStream`, `WsConnect`, `WsConnector`, `State`
- **Interactions**: Uses `FlashBlockDecoder` for payload decoding and `tokio_tungstenite` for IO.
- **Knobs / invariants**: Retries connections indefinitely; responds to ping with pong.

## End-to-end flow (high level)
- Create `WsFlashBlockStream` with a websocket URL and optional decoder.
- Connect via `WsConnector` (or custom `WsConnect`) and split into sink/stream.
- Decode incoming frames into `FlashBlock` values.
- Handle ping/pong and reconnect on connection errors.

## Key APIs (no snippets)
- `WsFlashBlockStream`, `WsConnect`, `FlashBlockDecoder`
```

## File: crates/optimism/flashblocks/src/AGENTS.md
```markdown
# src

## Purpose
Flashblocks integration for OP: stream and decode flashblocks, assemble pending blocks, and optionally drive consensus updates.

## Contents (one hop)
### Subdirectories
- [x] `ws/` - Websocket stream and decoder for flashblock payloads.

### Files
- `lib.rs` - Crate wiring, re-exports, and channel type aliases.
  - **Key items**: `FlashblocksListeners`, `PendingBlockRx`, `FlashBlockCompleteSequenceRx`, `FlashBlockRx`
- `consensus.rs` - Consensus client that submits flashblock-derived payloads to the engine.
  - **Key items**: `FlashBlockConsensusClient`, `submit_new_payload()`, `submit_forkchoice_update()`
- `payload.rs` - Flashblock payload types and pending block wrapper.
  - **Key items**: `FlashBlock`, `PendingFlashBlock`
- `sequence.rs` - Sequence tracking and validation for flashblocks.
  - **Key items**: `FlashBlockPendingSequence`, `FlashBlockCompleteSequence`, `SequenceExecutionOutcome`
- `service.rs` - Main service loop for processing flashblocks and building pending blocks.
  - **Key items**: `FlashBlockService`, `FlashBlockBuildInfo`, `run()`
- `worker.rs` - Flashblock block builder and execution worker.
  - **Key items**: `FlashBlockBuilder`, `BuildArgs`, `execute()`
- `cache.rs` - Sequence cache manager and build selection logic.
  - **Key items**: `SequenceManager`, `FLASHBLOCK_BLOCK_TIME`, `CACHE_SIZE`
- `test_utils.rs` - Test-only helpers for building flashblocks.
  - **Key items**: `TestFlashBlockFactory`, `TestFlashBlockBuilder`

## Key APIs (no snippets)
- `FlashBlockService`, `FlashBlockConsensusClient`
- `FlashblocksListeners`, `PendingFlashBlock`
- `WsFlashBlockStream`
```

## File: crates/optimism/flashblocks/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for flashblock websocket streaming.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness for integration tests.
- **Key items**: module `stream`
- **Interactions**: Aggregates websocket stream tests.

### `stream.rs`
- **Role**: Verifies that websocket streaming yields decodable flashblocks from a remote source.
- **Key items**: `test_streaming_flashblocks_from_remote_source_is_successful`
- **Interactions**: Uses `WsFlashBlockStream` to fetch and decode items.

## End-to-end flow (high level)
- Construct a `WsFlashBlockStream` targeting a flashblocks endpoint.
- Consume a small number of items and assert they decode successfully.

## Key APIs (no snippets)
- `WsFlashBlockStream`
```

## File: crates/optimism/flashblocks/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for the flashblocks streaming client.

## Contents (one hop)
### Subdirectories
- [x] `it/` - Websocket streaming integration tests.

### Files
- (none)
```

## File: crates/optimism/flashblocks/AGENTS.md
```markdown
# flashblocks

## Purpose
`reth-optimism-flashblocks` crate: flashblocks stream decoding, pending block assembly, and optional consensus submission.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Flashblocks service, sequence management, builders, and websocket client.
- [x] `tests/` - Integration tests for websocket streaming.

### Files
- `Cargo.toml` - Manifest for flashblocks dependencies (reth execution, websocket IO, codecs).
  - **Key items**: deps `reth-evm`, `reth-optimism-payload-builder`, `tokio-tungstenite`
```

## File: crates/optimism/hardforks/src/AGENTS.md
```markdown
# src

## Purpose
Defines Optimism/Base hardfork schedules and re-exports OP hardfork types.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Hardfork schedules for OP/Base chains and dev presets.
  - **Key items**: `DEV_HARDFORKS`, `OP_MAINNET_HARDFORKS`, `OP_SEPOLIA_HARDFORKS`, `BASE_MAINNET_HARDFORKS`, `BASE_SEPOLIA_HARDFORKS`
  - **Interactions**: Re-exports `OpHardfork`, `OpHardforks` from `alloy-op-hardforks`.

## Key APIs (no snippets)
- `OpHardfork`, `OpHardforks`
```

## File: crates/optimism/hardforks/AGENTS.md
```markdown
# hardforks

## Purpose
`reth-optimism-forks` crate: Optimism/Base hardfork schedules and OP hardfork type re-exports.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Hardfork schedules for dev, OP, and Base chains.

### Files
- `Cargo.toml` - Manifest for OP hardfork schedule dependencies and features.
  - **Key items**: deps `alloy-op-hardforks`, `reth-ethereum-forks`; features `serde`, `std`
```

## File: crates/optimism/node/src/AGENTS.md
```markdown
# src

## Purpose
Optimism node configuration, components, and engine/RPC wiring.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and re-exports for OP node components.
  - **Key items**: `OpEngineTypes`, `OpEngineApiBuilder`, `OpPayloadBuilder`, `OpStorage`
- `args.rs` - Rollup CLI arguments for OP nodes.
  - **Key items**: `RollupArgs`
- `engine.rs` - Engine types and validator for OP engine API.
  - **Key items**: `OpEngineTypes`, `OpEngineValidator`
- `node.rs` - Optimism node type and component builder wiring.
  - **Key items**: `OpNode`, `OpNodeComponentBuilder`, `OpAddOnsBuilder`
- `rpc.rs` - RPC engine API builder for OP.
  - **Key items**: `OpEngineApiBuilder`
- `utils.rs` - Test utilities for OP node setup and payload attributes.
  - **Key items**: `setup()`, `advance_chain()`, `optimism_payload_attributes()`
- `version.rs` - OP client name constant.
  - **Key items**: `OP_NAME_CLIENT`

## Key APIs (no snippets)
- `OpNode`, `OpEngineTypes`, `OpEngineValidator`
```

## File: crates/optimism/node/tests/e2e-testsuite/AGENTS.md
```markdown
# e2e-testsuite

## Purpose
End-to-end test suite for OP node behavior, sync, and payload handling.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness wiring the e2e test suite modules.
- **Key items**: modules `p2p`, `testsuite`
- **Interactions**: Includes the sync and testsuite modules under the e2e harness.

### `p2p.rs`
- **Role**: Multi-node e2e sync test that exercises optimistic sync, reorgs, and live sync.
- **Key items**: `can_sync`
- **Interactions**: Uses the e2e test utils to configure peers and advance the chain.

### `testsuite.rs`
- **Role**: Testsuite integration that verifies mined block attributes with OP-specific checks.
- **Key items**: `test_testsuite_op_assert_mine_block*`
- **Interactions**: Uses `reth_e2e_test_utils` assertions on OP payload attributes.

## End-to-end flow (high level)
- Wire the e2e modules via `main.rs`.
- Spin up OP nodes and advance a canonical chain in `p2p` tests.
- Run testsuite assertions against mined payload attributes in `testsuite`.

## Key APIs (no snippets)
- E2E tests using `reth_e2e_test_utils`
```

## File: crates/optimism/node/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for OP node builder, custom genesis, tx priority, and RPC behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness wiring the integration test modules.
- **Key items**: modules `builder`, `priority`, `rpc`, `custom_genesis`
- **Interactions**: Aggregates the module tests under `tests/it`.

### `builder.rs`
- **Role**: Verifies OP node builder setup and custom precompile wiring.
- **Key items**: `test_basic_setup`, `test_setup_custom_precompiles`
- **Interactions**: Exercises node builder configuration and EVM factory customization.

### `custom_genesis.rs`
- **Role**: Validates custom genesis block number support.
- **Key items**: `test_op_node_custom_genesis_number`
- **Interactions**: Asserts stage checkpoints and provider queries after init.

### `priority.rs`
- **Role**: Tests custom transaction priority integration with payload builder.
- **Key items**: `CustomTxPriority`, `test_custom_block_priority_config`
- **Interactions**: Influences payload assembly ordering.

### `rpc.rs`
- **Role**: Checks admin RPC external IP reporting.
- **Key items**: `test_admin_external_ip`
- **Interactions**: Verifies network config propagation to RPC.

## End-to-end flow (high level)
- Build node configurations via the builder tests.
- Initialize custom genesis and validate checkpoints.
- Exercise custom transaction priority in payload builder tests.
- Validate RPC admin info reflects network configuration.

## Key APIs (no snippets)
- `OpNode` builder tests
```

## File: crates/optimism/node/tests/AGENTS.md
```markdown
# tests

## Purpose
Test suites for OP node behavior, including e2e sync and integration tests.

## Contents (one hop)
### Subdirectories
- [x] `assets/` - (skip: data/assets only) genesis fixtures.
- [x] `e2e-testsuite/` - E2E sync and testsuite-based assertions.
- [x] `it/` - Integration tests for builder, RPC, and custom genesis behavior.

### Files
- (none)
```

## File: crates/optimism/node/AGENTS.md
```markdown
# node

## Purpose
`reth-optimism-node` crate: OP node type definitions, component wiring, and engine/RPC integration.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP node config, engine types, and component builders.
- [x] `tests/` - E2E and integration tests for OP node behavior.

### Files
- `Cargo.toml` - Manifest for OP node dependencies and features.
  - **Key items**: features `js-tracer`, `test-utils`, `reth-codec`; deps `reth-node-builder`, `reth-optimism-rpc`
```

## File: crates/optimism/payload/src/AGENTS.md
```markdown
# src

## Purpose
Optimism payload builder implementation, payload types, and validation utilities.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and `OpPayloadTypes` aggregation.
  - **Key items**: `OpPayloadTypes`, `OpPayloadBuilder`, `OpExecutionPayloadValidator`
- `builder.rs` - Optimism payload builder logic.
  - **Key items**: `OpPayloadBuilder`, `build_payload()`
- `config.rs` - Builder configuration for data availability and gas limits.
  - **Key items**: `OpBuilderConfig`, `OpDAConfig`, `OpGasLimitConfig`
- `error.rs` - Optimism payload builder errors.
  - **Key items**: `OpPayloadBuilderError`
- `payload.rs` - Payload attribute types, payload ID derivation, and built payload wrapper.
  - **Key items**: `OpPayloadBuilderAttributes`, `OpBuiltPayload`, `payload_id_optimism()`
- `traits.rs` - Helper traits for OP payload primitives and attributes.
  - **Key items**: `OpPayloadPrimitives`, `OpAttributes`
- `validator.rs` - Execution payload validator for OP hardfork rules.
  - **Key items**: `OpExecutionPayloadValidator`, `ensure_well_formed_payload()`

## Key APIs (no snippets)
- `OpPayloadBuilder`, `OpPayloadTypes`
- `OpPayloadBuilderAttributes`, `OpBuiltPayload`
```

## File: crates/optimism/payload/AGENTS.md
```markdown
# payload

## Purpose
`reth-optimism-payload-builder` crate: builds Optimism payloads and validates OP execution payloads.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Payload builder implementation, types, and validators.

### Files
- `Cargo.toml` - Manifest for OP payload builder dependencies.
  - **Key items**: deps `reth-payload-builder`, `reth-optimism-evm`, `reth-optimism-txpool`
```

## File: crates/optimism/primitives/src/transaction/AGENTS.md
```markdown
# transaction

## Purpose
Optimism transaction type definitions and test-only legacy compatibility helpers.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and type aliases for Optimism transactions.
  - **Key items**: `OpTransactionSigned`, re-exports `OpTransaction`, `OpTypedTransaction`, `OpTxType`
- `tx_type.rs` - Tests for compact encoding of OP transaction types.
  - **Key items**: `OpTxType` compact encoding/decoding tests
- `signed.rs` - Legacy signed transaction type kept for consistency tests.
  - **Key items**: `OpTransactionSigned` (legacy test-only)

## Key APIs (no snippets)
- `OpTransactionSigned`, `OpTxType`
```

## File: crates/optimism/primitives/src/AGENTS.md
```markdown
# src

## Purpose
Optimism primitive types: OP block/receipt types, bedrock constants, and transaction wrappers.

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - OP transaction types and test-only legacy helpers.

### Files
- `lib.rs` - Crate entrypoint defining `OpPrimitives` and type aliases.
  - **Key items**: `OpPrimitives`, `OpBlock`, `OpBlockBody`, `OpTransactionSigned`, `OpReceipt`
- `bedrock.rs` - Bedrock and replayed-transaction constants.
  - **Key items**: `BEDROCK_HEADER`, `BEDROCK_HEADER_HASH`, `BLOCK_NUMS_REPLAYED_TX`, `is_dup_tx()`
- `receipt.rs` - Receipt utilities and deposit receipt helpers.
  - **Key items**: `DepositReceipt`, `OpReceipt` serde compat (feature-gated)

## Key APIs (no snippets)
- `OpPrimitives`, `OpBlock`, `OpTransactionSigned`, `OpReceipt`
```

## File: crates/optimism/primitives/AGENTS.md
```markdown
# primitives

## Purpose
`reth-optimism-primitives` crate: OP-specific primitive types, receipts, and transaction wrappers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP primitives, bedrock constants, receipts, and transaction types.

### Files
- `Cargo.toml` - Manifest for OP primitives and optional serde/codec features.
  - **Key items**: features `serde`, `reth-codec`, `serde-bincode-compat`, `alloy-compat`
```

## File: crates/optimism/reth/src/AGENTS.md
```markdown
# src

## Purpose
Optimism meta-crate that re-exports commonly used reth and OP types behind feature flags.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Feature-gated re-exports for primitives, chainspec, node, RPC, provider, and network.
  - **Key items**: modules `cli`, `consensus`, `chainspec`, `evm`, `node`, `rpc`, `network`, `provider`

## Key APIs (no snippets)
- Re-exports of `reth_optimism_primitives` and related subsystems
```

## File: crates/optimism/reth/AGENTS.md
```markdown
# reth

## Purpose
`reth-op` meta crate: feature-gated re-exports for OP reth stacks (consensus, node, RPC, storage, etc.).

## Contents (one hop)
### Subdirectories
- [x] `src/` - Feature-gated re-export surface.

### Files
- `Cargo.toml` - Manifest defining feature flags for OP reth re-exports.
  - **Key items**: features `full`, `node`, `rpc`, `consensus`, `evm`, `provider`, `cli`
```

## File: crates/optimism/rpc/src/eth/AGENTS.md
```markdown
# eth

## Purpose
OP-specific `eth_` RPC implementation built on reth's Eth API, adding sequencer forwarding, flashblocks support, and OP receipt/transaction conversions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core OP Eth API wrapper and builder integrating reth's `EthApiInner` with OP extensions.
- **Key items**: `OpEthApi`, `OpEthApiInner`, `OpEthApiBuilder`, `OpRpcConvert`, `flashblock_receipts_stream()`
- **Interactions**: Wraps `reth_rpc::eth::core::EthApiInner`, uses `SequencerClient`, `FlashblocksListeners`, `OpReceiptConverter`, `OpTxInfoMapper`.
- **Knobs / invariants**: `min_suggested_priority_fee`; `flashblock_consensus` requires `flashblocks_url`; `MAX_FLASHBLOCK_WAIT_DURATION`.

### `block.rs`
- **Role**: Implements block-loading traits for `OpEthApi`.
- **Key items**: `EthBlocks`, `LoadBlock`
- **Interactions**: Delegates to the inner Eth API.

### `call.rs`
- **Role**: Implements call and estimate helpers for OP Eth API.
- **Key items**: `EthCall`, `EstimateCall`, `call_gas_limit()`, `max_simulate_blocks()`, `evm_memory_limit()`
- **Interactions**: Reads limits from the inner Eth API configuration.

### `ext.rs`
- **Role**: L2 `eth_` extension that validates conditional transactions and routes to sequencer or pool.
- **Key items**: `OpEthExtApi`, `send_raw_transaction_conditional()`, `validate_known_accounts()`, `TxConditionalErr`
- **Interactions**: Uses `SequencerClient`, `TransactionPool`, and state providers for validation.
- **Knobs / invariants**: `MAX_CONDITIONAL_EXECUTION_COST`, `MAX_CONCURRENT_CONDITIONAL_VALIDATIONS`, checks latest header bounds.

### `pending_block.rs`
- **Role**: Pending block resolution for OP, including flashblocks-backed pending data.
- **Key items**: `LoadPendingBlock`, `local_pending_state()`, `local_pending_block()`
- **Interactions**: Integrates `PendingEnvBuilder` and flashblock receivers.

### `receipt.rs`
- **Role**: Converts receipts into OP-specific types with L1 fee fields and hardfork metadata.
- **Key items**: `OpReceiptConverter`, `OpReceiptFieldsBuilder`, `OpReceiptBuilder`
- **Interactions**: Uses `reth_optimism_evm::extract_l1_info` and chain spec hardfork checks.
- **Knobs / invariants**: Clears per-tx L1 cost cache; special-cases genesis with missing L1 info.

### `transaction.rs`
- **Role**: Transaction handling with sequencer forwarding, sync receipt waits, and OP tx info mapping.
- **Key items**: `OpTxInfoMapper`, `send_transaction()`, `send_raw_transaction_sync()`, `transaction_receipt()`
- **Interactions**: Consults flashblocks for receipt confirmation and falls back to canonical chain/pool.

## End-to-end flow (high level)
- Build `OpEthApi` via `OpEthApiBuilder`, optionally wiring a `SequencerClient` and flashblocks.
- Serve `eth_` calls using reth Eth helpers with OP-specific conversions and overrides.
- Forward raw transactions to the sequencer when configured, retaining them in the local pool.
- Resolve pending blocks/receipts using flashblocks when available, otherwise use canonical data.
- Convert receipts with OP L1 fee and hardfork-specific fields.
- Handle conditional transactions in `OpEthExtApi` with state validation and routing.

## Key APIs (no snippets)
- `OpEthApi`, `OpEthApiBuilder`, `OpRpcConvert`
- `OpReceiptConverter`, `OpReceiptBuilder`, `OpTxInfoMapper`
- `OpEthExtApi`
```

## File: crates/optimism/rpc/src/AGENTS.md
```markdown
# src

## Purpose
OP-specific RPC server implementations and helpers: engine API, eth API wrapper, sequencer client, historical forwarding, and debug witness support.

## Contents (one hop)
### Subdirectories
- [x] `eth/` - OP `eth_` API wrapper with sequencer forwarding, flashblocks, and OP receipt/tx conversions.

### Files
- `lib.rs` - Module wiring and public re-exports for OP RPC types.
  - **Key items**: `OpEthApi`, `OpEthApiBuilder`, `OpReceiptBuilder`, `OpEngineApi`, `OP_ENGINE_CAPABILITIES`
- `engine.rs` - OP Engine API trait and server implementation over `EngineApi`.
  - **Key items**: `OpEngineApi`, `OP_ENGINE_CAPABILITIES`, `OP_STACK_SUPPORT`, `signal_superchain_v1`
- `error.rs` - OP-specific RPC error types and conversions.
  - **Key items**: `OpEthApiError`, `OpInvalidTransactionError`, `TxConditionalErr`, `SequencerClientError`
- `historical.rs` - Historical RPC forwarding layer for pre-bedrock data.
  - **Key items**: `HistoricalRpcClient`, `HistoricalRpc`, `HistoricalRpcService`
- `metrics.rs` - Sequencer client latency metrics.
  - **Key items**: `SequencerMetrics`
- `miner.rs` - Miner extension API for DA/gas limit controls.
  - **Key items**: `OpMinerExtApi`, `OpMinerMetrics`, `set_max_da_size()`, `set_gas_limit()`
- `sequencer.rs` - Sequencer client and request forwarding helpers.
  - **Key items**: `SequencerClient`, `forward_raw_transaction()`, `forward_raw_transaction_conditional()`
- `witness.rs` - Debug witness API backed by OP payload builder.
  - **Key items**: `OpDebugWitnessApi`, `execute_payload()`

## Key APIs (no snippets)
- `OpEthApi`, `OpEthApiBuilder`, `OpEngineApi`
- `SequencerClient`, `HistoricalRpcClient`
- `OpMinerExtApi`, `OpDebugWitnessApi`
```

## File: crates/optimism/rpc/AGENTS.md
```markdown
# rpc

## Purpose
`reth-optimism-rpc` crate: OP-specific RPC implementations for engine, eth, miner, sequencer, and debug/witness endpoints.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP RPC servers, errors, sequencer client, historical forwarding, and eth wrappers.

### Files
- `Cargo.toml` - Manifest for OP RPC dependencies and client feature.
  - **Key items**: feature `client`; deps `reth-rpc-eth-api`, `reth-rpc-engine-api`, `reth-optimism-flashblocks`, `op-alloy-rpc-types-engine`
```

## File: crates/optimism/storage/src/AGENTS.md
```markdown
# src

## Purpose
Optimism storage type alias built on `EmptyBodyStorage`.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and tests for prune codec compatibility.
  - **Key items**: `OpStorage`
- `chain.rs` - Type alias for Optimism storage implementation.
  - **Key items**: `OpStorage`

## Key APIs (no snippets)
- `OpStorage`
```

## File: crates/optimism/storage/AGENTS.md
```markdown
# storage

## Purpose
`reth-optimism-storage` crate: Optimism storage alias types built on `EmptyBodyStorage`.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Storage alias and codec compatibility tests.

### Files
- `Cargo.toml` - Manifest for storage and primitives dependencies.
  - **Key items**: deps `reth-storage-api`, `reth-optimism-primitives`; feature `std`
```

## File: crates/optimism/txpool/src/supervisor/AGENTS.md
```markdown
# supervisor

## Purpose
Interop supervisor client and helpers for validating cross-chain transactions in the OP txpool.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for supervisor client and interop types.
- **Key items**: `SupervisorClient`, `SupervisorClientBuilder`, `InteropTxValidatorError`, `ExecutingDescriptor`
- **Interactions**: Re-exports op-alloy interop primitives and internal helpers.

### `access_list.rs`
- **Role**: Parses access list items to extract cross-L2 inbox entries.
- **Key items**: `parse_access_list_items_to_inbox_entries()`
- **Knobs / invariants**: Only access list items targeting `CROSS_L2_INBOX_ADDRESS` are considered.

### `client.rs`
- **Role**: Supervisor RPC client for validating cross-chain transaction access lists.
- **Key items**: `SupervisorClient`, `SupervisorClientBuilder`, `CheckAccessListRequest`, `DEFAULT_SUPERVISOR_URL`
- **Interactions**: Uses `SupervisorMetrics` and `ExecutingDescriptor` for requests; emits `InvalidCrossTx` results.
- **Knobs / invariants**: Configurable timeout and minimum safety level; supports concurrent revalidation streams.

### `errors.rs`
- **Role**: Error mapping for supervisor validation failures.
- **Key items**: `InteropTxValidatorError`, `from_json_rpc()`
- **Interactions**: Converts supervisor JSON-RPC error codes into `SuperchainDAError` variants.

### `message.rs`
- **Role**: Defines the executing descriptor payload for supervisor checks.
- **Key items**: `ExecutingDescriptor`
- **Knobs / invariants**: Includes timestamp and optional timeout for validity windows.

### `metrics.rs`
- **Role**: Metrics for supervisor query latency and error categories.
- **Key items**: `SupervisorMetrics`, `record_supervisor_query()`, `increment_metrics_for_error()`
- **Interactions**: Updates counters based on `SuperchainDAError`.

## End-to-end flow (high level)
- Parse access list entries to detect cross-chain inbox references.
- Build a `SupervisorClient` with timeout and safety-level settings.
- Issue `supervisor_checkAccessList` requests with an `ExecutingDescriptor`.
- Map response errors into `InteropTxValidatorError` and update metrics.
- Use revalidation streams to periodically re-check interop transactions.

## Key APIs (no snippets)
- `SupervisorClient`, `SupervisorClientBuilder`
- `InteropTxValidatorError`, `ExecutingDescriptor`
```

## File: crates/optimism/txpool/src/AGENTS.md
```markdown
# src

## Purpose
OP transaction pool extensions: pooled transaction wrapper, OP-specific validation, and maintenance for conditional/interop txs.

## Contents (one hop)
### Subdirectories
- [x] `supervisor/` - Supervisor client and helpers for interop transaction validation.

### Files
- `lib.rs` - Crate wiring and public exports for OP txpool.
  - **Key items**: `OpTransactionPool`, `OpTransactionValidator`, `OpPooledTransaction`, `InvalidCrossTx`
- `transaction.rs` - OP pooled transaction wrapper with conditional/interop metadata and DA size helpers.
  - **Key items**: `OpPooledTransaction`, `OpPooledTx`, `estimated_compressed_size()`, `encoded_2718()`
- `validator.rs` - OP transaction validator with L1 gas checks and interop validation.
  - **Key items**: `OpL1BlockInfo`, `OpTransactionValidator`, `update_l1_block_info()`, `validate_one_with_state()`
- `conditional.rs` - Trait helpers for attaching and checking transaction conditionals.
  - **Key items**: `MaybeConditionalTransaction`, `has_exceeded_block_attributes()`
- `interop.rs` - Trait helpers and utilities for interop deadlines.
  - **Key items**: `MaybeInteropTransaction`, `is_valid_interop()`, `is_stale_interop()`
- `estimated_da_size.rs` - Data availability sizing trait.
  - **Key items**: `DataAvailabilitySized`
- `maintain.rs` - Pool maintenance loops for conditional and interop transactions.
  - **Key items**: `maintain_transaction_pool_conditional_future()`, `maintain_transaction_pool_interop_future()`
- `error.rs` - Cross-chain validation errors for the pool.
  - **Key items**: `InvalidCrossTx`

## Key APIs (no snippets)
- `OpTransactionPool`, `OpTransactionValidator`
- `OpPooledTransaction`, `OpPooledTx`
- `SupervisorClient` (from `supervisor/`)
```

## File: crates/optimism/txpool/AGENTS.md
```markdown
# txpool

## Purpose
`reth-optimism-txpool` crate: OP transaction pool types, validation, and maintenance for conditional and interop transactions.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP pooled transactions, validators, supervisor client, and maintenance loops.

### Files
- `Cargo.toml` - Manifest for OP txpool dependencies and metrics.
  - **Key items**: deps `reth-transaction-pool`, `reth-optimism-evm`, `op-alloy-consensus`
```

## File: crates/optimism/AGENTS.md
```markdown
# optimism

## Purpose
Optimism-specific crates for op-reth: chain specs, consensus/EVM, node wiring, RPC, and supporting tooling.

## Contents (one hop)
### Subdirectories
- [x] `bin/` - op-reth binary entrypoint and feature wiring.
- [x] `chainspec/` - OP/Base chain specs and superchain registry helpers.
- [x] `cli/` - op-reth CLI with OP import codecs and commands.
- [x] `consensus/` - OP consensus rules and receipt root handling.
- [x] `evm/` - OP EVM config, L1 info parsing, and block assembly.
- [x] `flashblocks/` - Flashblocks streaming, pending block assembly, and consensus updates.
- [x] `hardforks/` - OP/Base hardfork schedule definitions.
- [x] `node/` - OP node types, builders, engine/RPC wiring, and tests.
- [x] `payload/` - OP payload builder and validator.
- [x] `primitives/` - OP primitive types, receipts, and transactions.
- [x] `reth/` - OP meta-crate re-exports.
- [x] `rpc/` - OP RPC implementations for engine/eth/sequencer endpoints.
- [x] `storage/` - OP storage alias types.
- [x] `txpool/` - OP transaction pool types, validation, and maintenance.

### Files
- (none)
```

## File: crates/payload/basic/src/AGENTS.md
```markdown
# src

## Purpose
Basic payload builder implementation that periodically builds payloads from the txpool with concurrency limits and cache reuse.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Core job generator and payload builder trait definitions plus job execution logic.
- **Key items**: `BasicPayloadJobGenerator`, `BasicPayloadJobGeneratorConfig`, `BasicPayloadJob`, `PayloadBuilder`, `PayloadConfig`, `BuildOutcome`
- **Interactions**: Uses `TaskSpawner` for build tasks, `CachedReads` for state reuse, and `CanonStateNotification` to seed pre-cached state.
- **Knobs / invariants**: `max_payload_tasks`, `interval`, `deadline`; `BuildOutcome::Freeze` stops further building.

### `better_payload_emitter.rs`
- **Role**: Wraps a `PayloadBuilder` to broadcast better/frozen payloads to subscribers.
- **Key items**: `BetterPayloadEmitter`
- **Interactions**: Emits payloads on a `tokio::sync::broadcast` channel.

### `metrics.rs`
- **Role**: Metrics for payload build attempts and empty responses.
- **Key items**: `PayloadBuilderMetrics`, `inc_requested_empty_payload()`, `inc_failed_payload_builds()`

### `stack.rs`
- **Role**: Builder composition utilities for chaining multiple builders.
- **Key items**: `PayloadBuilderStack`, `Either`
- **Interactions**: Implements `PayloadBuilderAttributes`/`BuiltPayload` for `Either` to allow fallback chains.

## End-to-end flow (high level)
- Configure a `BasicPayloadJobGenerator` with a `PayloadBuilder` and limits.
- Spawn `BasicPayloadJob` instances that periodically attempt payload builds.
- Use cached reads and best-payload tracking to avoid redundant work.
- Optionally wrap builders with `BetterPayloadEmitter` or `PayloadBuilderStack` for emission/fallback.

## Key APIs (no snippets)
- `BasicPayloadJobGenerator`, `BasicPayloadJob`
- `PayloadBuilder`, `BuildOutcome`, `PayloadConfig`
- `BetterPayloadEmitter`, `PayloadBuilderStack`
```

## File: crates/payload/basic/AGENTS.md
```markdown
# basic

## Purpose
`reth-basic-payload-builder` crate: basic payload job generator and builder utilities built on txpool APIs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Basic payload builder jobs, metrics, and builder wrappers.

### Files
- `Cargo.toml` - Manifest for basic payload builder dependencies.
  - **Key items**: deps `reth-payload-builder`, `reth-payload-primitives`, `reth-tasks`
```

## File: crates/payload/builder/src/AGENTS.md
```markdown
# src

## Purpose
Payload builder service abstractions: job traits, service orchestration, and handle APIs for engine integration.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate wiring and re-exports for payload builder service types and Ethereum payload primitives.
- **Key items**: `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadJob`, `PayloadJobGenerator`, `PayloadId`
- **Interactions**: Re-exports `EthBuiltPayload`/`EthPayloadBuilderAttributes` for convenience.

### `service.rs`
- **Role**: Core service that manages payload jobs and handles engine-facing commands.
- **Key items**: `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadStore`, `PayloadServiceCommand`
- **Interactions**: Drives `PayloadJob` futures and emits `PayloadEvents`.
- **Knobs / invariants**: Tracks active jobs, best/resolved payloads, and handles resolve semantics.

### `traits.rs`
- **Role**: Trait definitions for payload jobs and job generators.
- **Key items**: `PayloadJob`, `PayloadJobGenerator`, `KeepPayloadJobAlive`
- **Knobs / invariants**: `resolve_kind()` must return quickly; jobs must be cancel safe.

### `metrics.rs`
- **Role**: Metrics for payload builder service activity and revenues.
- **Key items**: `PayloadBuilderServiceMetrics`

### `noop.rs`
- **Role**: No-op payload builder service for configurations that disable payload building.
- **Key items**: `NoopPayloadBuilderService`, `PayloadBuilderHandle::noop()`

### `test_utils.rs`
- **Role**: Test helpers for creating and spawning a payload builder service.
- **Key items**: `test_payload_service()`, `spawn_test_payload_service()`, `TestPayloadJobGenerator`

## End-to-end flow (high level)
- Engine requests trigger `PayloadBuilderHandle` commands to the service.
- `PayloadBuilderService` creates a `PayloadJob` via the configured generator.
- The job is polled to produce better payloads until resolved or cancelled.
- Clients query best payloads or resolve with `PayloadKind` semantics.

## Key APIs (no snippets)
- `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadStore`
- `PayloadJob`, `PayloadJobGenerator`, `KeepPayloadJobAlive`
```

## File: crates/payload/builder/AGENTS.md
```markdown
# builder

## Purpose
`reth-payload-builder` crate: payload builder service traits, orchestration, and engine-facing APIs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Payload builder service, job traits, and utilities.

### Files
- `Cargo.toml` - Manifest for payload builder core dependencies and test-utils feature.
  - **Key items**: feature `test-utils`; deps `reth-payload-primitives`, `reth-chain-state`
```

## File: crates/payload/builder-primitives/src/AGENTS.md
```markdown
# src

## Purpose
Payload builder event primitives: broadcast event types and stream adapters.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports for payload events.
- **Key items**: `Events`, `PayloadEvents`, `PayloadBuilderError`
- **Interactions**: Re-exports definitions from `events.rs`.

### `events.rs`
- **Role**: Event enum and stream adapters for payload attribute and payload outputs.
- **Key items**: `Events`, `PayloadEvents`, `BuiltPayloadStream`, `PayloadAttributeStream`
- **Interactions**: Wraps `tokio::sync::broadcast` into `tokio_stream` adapters.

## End-to-end flow (high level)
- Payload builder service broadcasts `Events` through a channel.
- `PayloadEvents` wraps the receiver and offers typed stream adapters.
- Consumers subscribe to payload attributes or built payloads as streams.

## Key APIs (no snippets)
- `Events`, `PayloadEvents`
```

## File: crates/payload/builder-primitives/AGENTS.md
```markdown
# builder-primitives

## Purpose
`reth-payload-builder-primitives` crate: event types and stream adapters for payload builder services.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Payload event enums and stream wrappers.

### Files
- `Cargo.toml` - Manifest for payload builder primitives dependencies.
  - **Key items**: deps `reth-payload-primitives`, `tokio-stream`
```

## File: crates/payload/primitives/src/AGENTS.md
```markdown
# src

## Purpose
Payload primitives and validation helpers for execution payloads, attributes, and builder errors.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint defining `PayloadTypes` and validation helpers across engine versions.
- **Key items**: `PayloadTypes`, `validate_payload_timestamp()`, `validate_withdrawals_presence()`, `validate_parent_beacon_block_root_presence()`, `validate_version_specific_fields()`
- **Interactions**: Re-exports `PayloadBuilderError` and validation error types from `error.rs`.

### `payload.rs`
- **Role**: Execution payload trait abstraction and wrapper for payloads vs attributes.
- **Key items**: `ExecutionPayload`, `PayloadOrAttributes`
- **Interactions**: Implements `ExecutionPayload` for `ExecutionData` and OP execution data (feature-gated).

### `traits.rs`
- **Role**: Core payload traits for built payloads and attribute types.
- **Key items**: `BuiltPayload`, `BuiltPayloadExecutedBlock`, `PayloadBuilderAttributes`, `PayloadAttributes`, `PayloadAttributesBuilder`, `BuildNextEnv`
- **Interactions**: Bridges execution results and trie updates into `BuiltPayloadExecutedBlock`.

### `error.rs`
- **Role**: Error types for payload building and validation.
- **Key items**: `PayloadBuilderError`, `EngineObjectValidationError`, `VersionSpecificValidationError`, `NewPayloadError`

## End-to-end flow (high level)
- Define payload types via `PayloadTypes` and related trait implementations.
- Validate payload/attribute fields against engine version rules.
- Use payload traits to expose built payload data and execution outcomes.

## Key APIs (no snippets)
- `PayloadTypes`, `ExecutionPayload`, `BuiltPayload`
- `PayloadBuilderError`, `EngineObjectValidationError`
```

## File: crates/payload/primitives/AGENTS.md
```markdown
# primitives

## Purpose
`reth-payload-primitives` crate: payload trait definitions, validation helpers, and payload error types.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Payload traits, validation utilities, and errors.

### Files
- `Cargo.toml` - Manifest for payload primitive dependencies and `op` feature.
  - **Key items**: features `std`, `op`; deps `alloy-rpc-types-engine`, `reth-execution-types`
```

## File: crates/payload/util/src/AGENTS.md
```markdown
# src

## Purpose
Utilities for selecting and composing transaction iterators during payload building.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and re-exports for payload transaction iterators.
- **Key items**: `PayloadTransactions`, `BestPayloadTransactions`, `PayloadTransactionsChain`

### `traits.rs`
- **Role**: Core iterator trait and adapters for txpool best transactions.
- **Key items**: `PayloadTransactions`, `NoopPayloadTransactions`, `BestPayloadTransactions`
- **Interactions**: Adapts `ValidPoolTransaction` iterators from the txpool.
- **Knobs / invariants**: `mark_invalid()` must prevent dependent txs from reappearing.

### `transaction.rs`
- **Role**: Fixed and chained transaction iterators with gas budgeting.
- **Key items**: `PayloadTransactionsFixed`, `PayloadTransactionsChain`
- **Knobs / invariants**: `PayloadTransactionsChain` enforces per-segment gas limits and propagates invalidation.

## End-to-end flow (high level)
- Build a `PayloadTransactions` iterator from txpool best transactions.
- Optionally prepend fixed transactions (e.g., sequencer-specified).
- Chain iterators with gas budgets to control block composition order.

## Key APIs (no snippets)
- `PayloadTransactions`, `BestPayloadTransactions`
- `PayloadTransactionsFixed`, `PayloadTransactionsChain`
```

## File: crates/payload/util/AGENTS.md
```markdown
# util

## Purpose
`reth-payload-util` crate: transaction selection helpers for payload building.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Payload transaction iterators and adapters.

### Files
- `Cargo.toml` - Manifest for payload utility dependencies.
  - **Key items**: deps `reth-transaction-pool`, `alloy-consensus`
```

## File: crates/payload/validator/src/AGENTS.md
```markdown
# src

## Purpose
Payload validation helpers for Shanghai, Cancun, and Prague protocol rules.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring for payload validation rules.
- **Key items**: modules `shanghai`, `cancun`, `prague`

### `shanghai.rs`
- **Role**: Validation for withdrawals field based on Shanghai activation.
- **Key items**: `ensure_well_formed_fields()`
- **Knobs / invariants**: Withdrawals must be present post-Shanghai and absent pre-Shanghai.

### `cancun.rs`
- **Role**: Validation for Cancun header fields, sidecar fields, and blob tx hashes.
- **Key items**: `ensure_well_formed_fields()`, `ensure_well_formed_header_and_sidecar_fields()`, `ensure_matching_blob_versioned_hashes()`
- **Knobs / invariants**: Blob gas fields and versioned hashes must match Cancun activation.

### `prague.rs`
- **Role**: Validation for Prague sidecar fields and EIP-7702 tx presence.
- **Key items**: `ensure_well_formed_fields()`, `ensure_well_formed_sidecar_fields()`
- **Knobs / invariants**: Prague request fields must be absent before Prague.

## End-to-end flow (high level)
- Determine hardfork activation for the payload timestamp.
- Apply Shanghai/Cancun/Prague checks to block body and sidecar fields.
- Reject payloads with unsupported fields for the active fork.

## Key APIs (no snippets)
- `ensure_well_formed_fields` (Shanghai/Cancun/Prague)
```

## File: crates/payload/validator/AGENTS.md
```markdown
# validator

## Purpose
`reth-payload-validator` crate: fork-specific payload validation helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Shanghai/Cancun/Prague validation rules.

### Files
- `Cargo.toml` - Manifest for payload validation dependencies and std feature.
  - **Key items**: feature `std`; deps `alloy-rpc-types-engine`
```

## File: crates/payload/AGENTS.md
```markdown
# payload

## Purpose
Payload builder subsystem crates: core service traits, primitives, validators, and basic utilities/builders.

## Contents (one hop)
### Subdirectories
- [x] `basic/` - Basic payload job generator and builder wrappers.
- [x] `builder/` - Payload builder service and job orchestration traits.
- [x] `builder-primitives/` - Payload builder event types and streams.
- [x] `primitives/` - Payload traits, validation helpers, and error types.
- [x] `util/` - Transaction iterator utilities for payload building.
- [x] `validator/` - Fork-specific payload validation checks.

### Files
- (none)
```

## File: crates/primitives/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for transaction signature recovery and blob validation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `recover_ecdsa_crit.rs`
- **Role**: Benchmarks ECDSA signer recovery for a sample transaction.
- **Key items**: `criterion_benchmark()`

### `validate_blob_tx.rs`
- **Role**: Benchmarks EIP-4844 blob validation with varying blob counts.
- **Key items**: `blob_validation()`, `validate_blob_tx()`
- **Knobs / invariants**: Runs across `MAX_BLOBS_PER_BLOCK_DENCUN` and KZG settings.

## End-to-end flow (high level)
- Construct sample transactions and sidecars for benchmarks.
- Run criterion benchmarks for signer recovery and blob validation.

## Key APIs (no snippets)
- Criterion benchmarks for primitives
```

## File: crates/primitives/src/transaction/AGENTS.md
```markdown
# transaction

## Purpose
Transaction type definitions, signature recovery helpers, and pooled transaction aliases.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and public re-exports for transaction types and errors.
- **Key items**: `Transaction`, `TransactionSigned`, `InvalidTransactionError`, `recover_signer`, `TxType`
- **Interactions**: Re-exports helpers from `signature`, `util`, and `tx_type`.

### `pooled.rs`
- **Role**: Deprecated pooled transaction alias for recovered pooled txs.
- **Key items**: `PooledTransactionsElementEcRecovered`

### `signature.rs`
- **Role**: Signature recovery helpers for transactions.
- **Key items**: `recover_signer`, `recover_signer_unchecked`

### `tx_type.rs`
- **Role**: Transaction type enum re-export and codec notes.
- **Key items**: `TxType`
- **Knobs / invariants**: Tx type encoding is limited to 2 bits for compact codecs.

### `util.rs`
- **Role**: Signature utility re-exports.
- **Key items**: `crypto` re-exports

## End-to-end flow (high level)
- Use `TransactionSigned`/`TxType` to represent EIP-2718 transactions.
- Recover signers via `recover_signer` helpers.
- Map pooled transaction aliases where needed for compatibility.

## Key APIs (no snippets)
- `Transaction`, `TransactionSigned`, `TxType`
- `recover_signer`, `recover_signer_unchecked`
```

## File: crates/primitives/src/AGENTS.md
```markdown
# src

## Purpose
Core primitive type aliases and re-exports used throughout reth (blocks, transactions, receipts).

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - Transaction types, signature helpers, and pooled aliases.

### Files
- `lib.rs` - Crate entrypoint and re-exports for primitives and fork definitions.
  - **Key items**: `Block`, `Receipt`, `Transaction`, `TxType`, `static_file`, `EthPrimitives`
- `block.rs` - Ethereum block and body type aliases.
  - **Key items**: `Block`, `BlockBody`, `SealedBlock`
- `receipt.rs` - Receipt type alias and helpers.
  - **Key items**: `Receipt`, `gas_spent_by_transactions`

## Key APIs (no snippets)
- `Block`, `BlockBody`, `SealedBlock`
- `Receipt`, `Transaction`, `TxType`
```

## File: crates/primitives/AGENTS.md
```markdown
# primitives

## Purpose
`reth-primitives` crate: common block, transaction, and receipt types plus benchmarks.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for signature recovery and blob validation.
- [x] `src/` - Primitive type aliases and transaction helpers.

### Files
- `Cargo.toml` - Manifest for primitives dependencies and feature flags.
  - **Key items**: features `reth-codec`, `c-kzg`, `arbitrary`, `serde-bincode-compat`
```

## File: crates/primitives-traits/src/block/AGENTS.md
```markdown
# block

## Purpose
Block trait abstractions and wrappers for sealing and sender recovery.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core block traits and re-exports for sealed/recovered blocks.
- **Key items**: `Block`, `FullBlock`, `BlockTx`, `SealedBlock`, `RecoveredBlock`
- **Interactions**: Uses `BlockBody` and `BlockHeader` traits from sibling modules.

### `body.rs`
- **Role**: Block body trait abstraction with transaction/ommers/withdrawals helpers.
- **Key items**: `BlockBody`, `FullBlockBody`, `BodyTx`, `BodyOmmer`
- **Interactions**: Uses transaction recovery helpers to compute sender lists.

### `header.rs`
- **Role**: Block header trait abstraction and aliases.
- **Key items**: `BlockHeader`, `FullBlockHeader`, `AlloyBlockHeader`

### `sealed.rs`
- **Role**: Sealed block wrapper that caches block hash and provides recovery helpers.
- **Key items**: `SealedBlock`, `seal_slow()`, `try_recover()`, `try_with_senders()`
- **Interactions**: Converts to `RecoveredBlock` and uses `SealedHeader`.

### `recovered.rs`
- **Role**: Recovered block wrapper with cached sender addresses.
- **Key items**: `RecoveredBlock`, `IndexedTx`
- **Interactions**: Upgrades from `SealedBlock`, preserves sender list.
- **Knobs / invariants**: Recovery can be checked vs unchecked (EIP-2 low-s).

### `error.rs`
- **Role**: Error types for block recovery.
- **Key items**: `BlockRecoveryError`, `SealedBlockRecoveryError`

## End-to-end flow (high level)
- Start with a `Block` and optionally seal it to get `SealedBlock`.
- Recover transaction senders to produce `RecoveredBlock`.
- Use recovery errors to inspect invalid blocks.

## Key APIs (no snippets)
- `Block`, `BlockBody`, `BlockHeader`
- `SealedBlock`, `RecoveredBlock`
```

## File: crates/primitives-traits/src/constants/AGENTS.md
```markdown
# constants

## Purpose
Protocol constants and gas unit helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Protocol constants and re-exports for gas units.
- **Key items**: `RETH_CLIENT_VERSION`, `MINIMUM_GAS_LIMIT`, `MAXIMUM_GAS_LIMIT_BLOCK`, `GAS_LIMIT_BOUND_DIVISOR`
- **Interactions**: Re-exports `KILOGAS`, `MEGAGAS`, `GIGAGAS`.

### `gas_units.rs`
- **Role**: Gas unit constants and formatting helpers.
- **Key items**: `KILOGAS`, `MEGAGAS`, `GIGAGAS`, `format_gas()`, `format_gas_throughput()`

## End-to-end flow (high level)
- Use gas unit constants for display or calculations.
- Format gas values and throughput strings for logging.

## Key APIs (no snippets)
- `format_gas()`, `format_gas_throughput()`
```

## File: crates/primitives-traits/src/header/AGENTS.md
```markdown
# header

## Purpose
Header wrapper types, mutable header helpers, and header test utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for header types.
- **Key items**: `SealedHeader`, `SealedHeaderFor`, `HeaderMut`
- **Interactions**: Exposes `test_utils` under test/features.

### `sealed.rs`
- **Role**: Sealed header wrapper with lazy hash caching.
- **Key items**: `SealedHeader`, `SealedHeaderFor`, `hash()`, `seal_slow()`
- **Knobs / invariants**: Hash computed lazily; equality/hash are based on cached hash.

### `header_mut.rs`
- **Role**: Mutable header trait for testing and mocks.
- **Key items**: `HeaderMut`, `set_parent_hash()`, `set_timestamp()`

### `test_utils.rs`
- **Role**: Header test helpers and proptest strategies.
- **Key items**: `generate_valid_header()`, `valid_header_strategy()`, `TestHeader`

## End-to-end flow (high level)
- Wrap headers in `SealedHeader` to cache hashes.
- Use `HeaderMut` in tests to tweak header fields.
- Generate fork-valid headers for testing via `test_utils`.

## Key APIs (no snippets)
- `SealedHeader`, `HeaderMut`
```

## File: crates/primitives-traits/src/transaction/AGENTS.md
```markdown
# transaction

## Purpose
Transaction trait abstractions, signature recovery, and validation errors.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Transaction trait entrypoint and module wiring.
- **Key items**: `Transaction`, `FullTransaction`, `SignerRecoverable`, `TransactionMeta`
- **Interactions**: Exposes `signed`, `signature`, `recover`, and error helpers.

### `signed.rs`
- **Role**: Signed transaction trait with recovery helpers.
- **Key items**: `SignedTransaction`, `FullSignedTx`, `RecoveryError`
- **Knobs / invariants**: Checked vs unchecked recovery for pre-EIP-2 transactions.

### `signature.rs`
- **Role**: Signature type re-export and tests.
- **Key items**: `Signature`

### `recover.rs`
- **Role**: Batch recovery helpers with optional rayon support.
- **Key items**: `recover_signers()`, `recover_signers_unchecked()`
- **Knobs / invariants**: Uses rayon when `rayon` feature is enabled.

### `execute.rs`
- **Role**: Trait for filling EVM transaction environment.
- **Key items**: `FillTxEnv`

### `error.rs`
- **Role**: Transaction validation and conversion errors.
- **Key items**: `InvalidTransactionError`, `TransactionConversionError`, `TryFromRecoveredTransactionError`

### `access_list.rs`
- **Role**: Test-only compatibility helpers for access list codecs.
- **Key items**: `RethAccessList`, `RethAccessListItem` (tests)

## End-to-end flow (high level)
- Represent transactions via `Transaction` and `SignedTransaction` traits.
- Recover signers with checked/unchecked helpers.
- Report validation and conversion errors via shared error enums.

## Key APIs (no snippets)
- `Transaction`, `SignedTransaction`, `FillTxEnv`
- `InvalidTransactionError`
```

## File: crates/primitives-traits/src/AGENTS.md
```markdown
# src

## Purpose
Foundational traits and helpers for reth primitives: blocks, headers, transactions, receipts, and supporting utilities.

## Contents (one hop)
### Subdirectories
- [x] `block/` - Block traits plus sealed/recovered wrappers and recovery errors.
- [x] `constants/` - Protocol constants and gas unit helpers.
- [x] `header/` - Sealed header wrapper and mutable/test header helpers.
- [x] `transaction/` - Transaction and signed-transaction traits with recovery helpers.

### Files
- `lib.rs` - Crate entrypoint, feature flags, and re-exports of core traits/types.
  - **Key items**: `MaybeSerde`, `MaybeCompact`, `MaybeSerdeBincodeCompat`, `NodePrimitives`
- `account.rs` - Account and bytecode primitives with codec helpers.
  - **Key items**: `Account`, `Bytecode`, `compact_ids`
- `crypto.rs` - Crypto helper re-exports.
  - **Key items**: `crypto` re-exports
- `error.rs` - Generic mismatch error types.
  - **Key items**: `GotExpected`, `GotExpectedBoxed`
- `extended.rs` - Wrapper enum for extending built-in transaction/receipt types.
  - **Key items**: `Extended`
- `log.rs` - (tests) codec compatibility helpers for logs.
  - **Key items**: test-only `Log` wrapper
- `node.rs` - Node primitives trait and type aliases.
  - **Key items**: `NodePrimitives`, `HeaderTy`, `BlockTy`, `TxTy`
- `proofs.rs` - Merkle root calculation helpers.
  - **Key items**: `calculate_transaction_root`, `calculate_receipt_root`, `calculate_withdrawals_root`
- `receipt.rs` - Receipt trait abstraction and gas accounting helper.
  - **Key items**: `Receipt`, `FullReceipt`, `gas_spent_by_transactions()`
- `serde_bincode_compat.rs` - Bincode-compatible serde helpers for consensus types.
  - **Key items**: `SerdeBincodeCompat`, `BincodeReprFor`, `RlpBincode`
- `size.rs` - Heuristic in-memory size trait and implementations.
  - **Key items**: `InMemorySize`
- `storage.rs` - Storage entry types and subkey extraction.
  - **Key items**: `StorageEntry`, `ValueWithSubKey`
- `sync.rs` - Sync primitives re-exports for std/no-std.
  - **Key items**: `LazyLock`, `OnceLock`
- `withdrawal.rs` - (tests) withdrawal codec compatibility helpers.
  - **Key items**: test-only `RethWithdrawal` wrapper

## Key APIs (no snippets)
- `NodePrimitives`, `Block`, `BlockBody`, `SignedTransaction`
- `SealedBlock`, `SealedHeader`, `RecoveredBlock`
- `InMemorySize`, `SerdeBincodeCompat`
```

## File: crates/primitives-traits/AGENTS.md
```markdown
# primitives-traits

## Purpose
`reth-primitives-traits` crate: foundational traits and utilities for blocks, headers, transactions, receipts, and serialization helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core traits, wrappers, and utility modules for primitives.

### Files
- `Cargo.toml` - Manifest with feature flags for serde, codecs, op, and arbitrary support.
  - **Key items**: features `serde`, `reth-codec`, `op`, `arbitrary`, `rpc-compat`
```

## File: crates/prune/db/src/AGENTS.md
```markdown
# src

## Purpose
Database integration layer for prune (placeholder module).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate-level documentation stub for `reth-prune-db`.

## Key APIs (no snippets)
- (none)
```

## File: crates/prune/db/AGENTS.md
```markdown
# db

## Purpose
`reth-prune-db` crate: database integration for prune implementation.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Minimal crate entrypoint documentation.

### Files
- `Cargo.toml` - Manifest for prune DB integration crate.
```

## File: crates/prune/prune/src/segments/user/AGENTS.md
```markdown
# user

## Purpose
User-configured prune segments for receipts, history, senders, and transaction lookup.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for user prune segments.
- **Key items**: `AccountHistory`, `StorageHistory`, `Receipts`, `ReceiptsByLogs`, `SenderRecovery`, `TransactionLookup`, `Bodies`

### `account_history.rs`
- **Role**: Prunes account history change sets and history indices.
- **Key items**: `AccountHistory`, `ACCOUNT_HISTORY_TABLES_TO_PRUNE`
- **Interactions**: Uses `prune_history_indices` to prune `AccountsHistory` shards.

### `storage_history.rs`
- **Role**: Prunes storage change sets and history indices.
- **Key items**: `StorageHistory`, `STORAGE_HISTORY_TABLES_TO_PRUNE`
- **Interactions**: Uses `prune_history_indices` on `StoragesHistory` shards.

### `history.rs`
- **Role**: Shared helpers for pruning history index shards.
- **Key items**: `PrunedIndices`, `prune_history_indices()`
- **Interactions**: Used by account/storage history segments.

### `bodies.rs`
- **Role**: Prunes transaction bodies stored in static files.
- **Key items**: `Bodies`
- **Interactions**: Delegates to `segments::prune_static_files`.

### `receipts.rs`
- **Role**: User-configured receipts pruning segment wrapper.
- **Key items**: `Receipts`
- **Interactions**: Delegates to `segments::receipts::{prune, save_checkpoint}`.

### `receipts_by_logs.rs`
- **Role**: Prunes receipts while retaining logs from configured addresses.
- **Key items**: `ReceiptsByLogs`
- **Interactions**: Uses `ReceiptsLogPruneConfig` to build block ranges and filters.
- **Knobs / invariants**: Applies `MINIMUM_PRUNING_DISTANCE` to ensure safe retention window.

### `sender_recovery.rs`
- **Role**: Prunes transaction sender recovery table entries.
- **Key items**: `SenderRecovery`

### `transaction_lookup.rs`
- **Role**: Prunes transaction hash lookup table with static-file-aware checkpoints.
- **Key items**: `TransactionLookup`
- **Interactions**: Computes tx hashes in parallel via rayon before pruning.

### `merkle_change_sets.rs`
- **Role**: Prunes trie change set tables for accounts and storages.
- **Key items**: `MerkleChangeSets`
- **Interactions**: Requires `StageId::MerkleChangeSets` before pruning.

## Key APIs (no snippets)
- `AccountHistory`, `StorageHistory`, `Receipts`, `ReceiptsByLogs`
- `SenderRecovery`, `TransactionLookup`, `Bodies`
```

## File: crates/prune/prune/src/segments/AGENTS.md
```markdown
# segments

## Purpose
Prune segment trait definitions and shared helpers for segment execution.

## Contents (one hop)
### Subdirectories
- [x] `user/` - User-configured prune segments for history, receipts, and tx data.

## Files (detailed)

### `mod.rs`
- **Role**: Segment trait, shared prune helpers, and public re-exports.
- **Key items**: `Segment`, `PruneInput`, `prune_static_files()`
- **Interactions**: Calls `Segment::prune` and handles static file pruning for receipts/bodies.

### `receipts.rs`
- **Role**: Shared receipts pruning logic and checkpoint saving.
- **Key items**: `prune()`, `save_checkpoint()`
- **Interactions**: Handles DB vs static-file receipts destinations.

### `set.rs`
- **Role**: Segment collection builder based on prune modes.
- **Key items**: `SegmentSet`, `from_components()`
- **Interactions**: Builds segment list in pruning order from `PruneModes`.

## Key APIs (no snippets)
- `Segment`, `PruneInput`, `SegmentSet`
```

## File: crates/prune/prune/src/AGENTS.md
```markdown
# src

## Purpose
Core prune engine: pruner runtime, segment wiring, and DB pruning helpers.

## Contents (one hop)
### Subdirectories
- [x] `segments/` - Segment trait definitions and user/static prune segments.

### Files
- `lib.rs` - Crate wiring and re-exports for the prune engine.
  - **Key items**: `Pruner`, `PrunerBuilder`, `PruneLimiter`, `PrunerError`
- `builder.rs` - Builder for configuring and constructing a `Pruner`.
  - **Key items**: `PrunerBuilder`, `build()`, `build_with_provider_factory()`
  - **Knobs / invariants**: `block_interval`, `delete_limit`, `timeout`, `segments`
- `pruner.rs` - Main pruner runtime, run loop, and events.
  - **Key items**: `Pruner`, `run_with_provider()`, `PrunerResult`
  - **Interactions**: Emits `PrunerEvent`, uses `PruneLimiter` and per-segment pruning.
- `limiter.rs` - Per-run limits for pruning by time or deleted entries.
  - **Key items**: `PruneLimiter`, `progress()`, `interrupt_reason()`
- `error.rs` - Error types for prune execution.
  - **Key items**: `PrunerError`
- `db_ext.rs` - DB transaction pruning extensions and helpers.
  - **Key items**: `DbTxPruneExt`, `prune_table_with_range()`, `prune_dupsort_table_with_range()`
- `metrics.rs` - Metrics for prune duration and per-segment progress.
  - **Key items**: `Metrics`, `PrunerSegmentMetrics`

## Key APIs (no snippets)
- `Pruner`, `PrunerBuilder`, `PruneLimiter`
- `Segment`, `PrunerEvent`
```

## File: crates/prune/prune/AGENTS.md
```markdown
# prune

## Purpose
`reth-prune` crate: pruning runtime, segment definitions, and DB prune helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Pruner runtime, segment wiring, and DB pruning helpers.

### Files
- `Cargo.toml` - Manifest for prune implementation dependencies.
  - **Key items**: deps `reth-prune-types`, `reth-provider`, `reth-stages-types`
```

## File: crates/prune/types/src/AGENTS.md
```markdown
# src

## Purpose
Common prune configuration and runtime types: segments, modes, checkpoints, and events.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and re-exports for prune types and helpers.
  - **Key items**: `PruneModes`, `PruneMode`, `PruneSegment`, `PrunerOutput`, `ReceiptsLogPruneConfig`
- `segment.rs` - Prune segment identifiers and purpose enums.
  - **Key items**: `PruneSegment`, `PrunePurpose`, `PruneSegmentError`
  - **Knobs / invariants**: Segment enum order is stable for DB encoding.
- `mode.rs` - Prune mode semantics and target block computation.
  - **Key items**: `PruneMode`, `prune_target_block()`, `should_prune()`
- `target.rs` - Top-level prune configuration and minimum distance rules.
  - **Key items**: `PruneModes`, `MINIMUM_PRUNING_DISTANCE`, `UnwindTargetPrunedError`
- `pruner.rs` - Runtime output and progress types for pruner runs.
  - **Key items**: `PrunerOutput`, `SegmentOutput`, `PruneProgress`, `PruneInterruptReason`
- `checkpoint.rs` - Persisted pruning checkpoint structure.
  - **Key items**: `PruneCheckpoint`
- `event.rs` - Pruner lifecycle events.
  - **Key items**: `PrunerEvent`

## Key APIs (no snippets)
- `PruneModes`, `PruneMode`, `PruneSegment`
- `PruneCheckpoint`, `PrunerOutput`, `PrunerEvent`
```

## File: crates/prune/types/AGENTS.md
```markdown
# types

## Purpose
`reth-prune-types` crate: shared types for prune configuration, progress, and checkpoints.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Prune modes, segments, checkpoints, and events.

### Files
- `Cargo.toml` - Manifest for prune type dependencies and serde/arbitrary features.
  - **Key items**: features `serde`, `reth-codec`, `test-utils`, `arbitrary`
```

## File: crates/prune/AGENTS.md
```markdown
# prune

## Purpose
Pruning subsystem crates: engine, shared types, and DB integration.

## Contents (one hop)
### Subdirectories
- [x] `db/` - Database integration stub for pruning.
- [x] `prune/` - Core prune engine, segments, and runtime helpers.
- [x] `types/` - Shared prune configuration, checkpoints, and events.

### Files
- (none)
```

## File: crates/ress/protocol/src/AGENTS.md
```markdown
# src

## Purpose
RESS protocol definitions, message codecs, and network handlers.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and module wiring for RESS protocol.
  - **Key items**: `NodeType`, `RessProtocolMessage`, `RessProtocolProvider`, `RessProtocolConnection`
- `types.rs` - Node type enum and handshake compatibility checks.
  - **Key items**: `NodeType`, `is_valid_connection()`
- `message.rs` - RLP message definitions and encoding helpers.
  - **Key items**: `RessProtocolMessage`, `RessMessage`, `RessMessageID`, `GetHeaders`
- `provider.rs` - Provider trait for serving headers, bodies, bytecode, and witnesses.
  - **Key items**: `RessProtocolProvider`
  - **Knobs / invariants**: Respects `MAX_HEADERS_SERVE`, `MAX_BODIES_SERVE`, `SOFT_RESPONSE_LIMIT`.
- `handlers.rs` - Protocol handler for connection lifecycle and max-connection limits.
  - **Key items**: `RessProtocolHandler`, `ProtocolEvent`, `ProtocolState`
- `connection.rs` - Connection state machine for request/response handling.
  - **Key items**: `RessProtocolConnection`, `RessPeerRequest`
  - **Interactions**: Routes `GetHeaders`, `GetBlockBodies`, `GetBytecode`, `GetWitness` messages.
- `test_utils.rs` - Mock and noop provider implementations for tests.
  - **Key items**: `MockRessProtocolProvider`, `NoopRessProtocolProvider`

## Key APIs (no snippets)
- `RessProtocolMessage`, `RessProtocolHandler`, `RessProtocolProvider`
- `NodeType`
```

## File: crates/ress/protocol/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for RESS protocol behavior and message flow.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test harness module wiring.
- **Key items**: module `e2e`

### `e2e.rs`
- **Role**: End-to-end protocol tests using the network test harness.
- **Key items**: `disconnect_on_stateful_pair`, `message_exchange`, `witness_fetching_does_not_block`
- **Interactions**: Uses `RessProtocolHandler` and mock providers to validate message flow.

## Key APIs (no snippets)
- `RessProtocolHandler`, `ProtocolEvent`
```

## File: crates/ress/protocol/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for RESS protocol networking and message flow.

## Contents (one hop)
### Subdirectories
- [x] `it/` - E2E protocol tests and message exchange scenarios.

### Files
- (none)
```

## File: crates/ress/protocol/AGENTS.md
```markdown
# protocol

## Purpose
`reth-ress-protocol` crate: RESS protocol messages, handlers, and provider traits for stateless sync.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Protocol messages, handlers, connection logic, and provider trait.
- [x] `tests/` - Integration tests for protocol behavior.

### Files
- `Cargo.toml` - Manifest for RESS protocol dependencies and features.
  - **Key items**: features `test-utils`, `arbitrary`
- `README.md` - Protocol overview, message formats, and usage notes.
```

## File: crates/ress/provider/src/AGENTS.md
```markdown
# src

## Purpose
Reth-backed provider implementation for the RESS protocol.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Implements `RessProtocolProvider` using reth storage, EVM execution, and witness caching.
  - **Key items**: `RethRessProtocolProvider`, `generate_witness()`, `block_by_hash()`
  - **Knobs / invariants**: `max_witness_window`, `witness_max_parallel`, LRU witness cache.
- `pending_state.rs` - Tracks executed and invalid blocks for witness construction.
  - **Key items**: `PendingState`, `maintain_pending_state()`
  - **Interactions**: Consumes `ConsensusEngineEvent` stream to keep pending state updated.
- `recorder.rs` - EVM database wrapper that records accessed state during execution.
  - **Key items**: `StateWitnessRecorderDatabase`
  - **Interactions**: Collects hashed account/storage accesses for witness generation.

## Key APIs (no snippets)
- `RethRessProtocolProvider`
- `PendingState`, `maintain_pending_state`
```

## File: crates/ress/provider/AGENTS.md
```markdown
# provider

## Purpose
`reth-ress-provider` crate: reth-backed provider for serving RESS protocol requests.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RESS provider implementation and pending-state tracking.

### Files
- `Cargo.toml` - Manifest for RESS provider dependencies.
  - **Key items**: deps `reth-ress-protocol`, `reth-revm`, `reth-trie`, `reth-storage-api`
```

## File: crates/ress/AGENTS.md
```markdown
# ress

## Purpose
RESS (stateless) protocol crates: protocol definitions and reth-backed provider implementation.

## Contents (one hop)
### Subdirectories
- [x] `protocol/` - RESS messages, handlers, and provider trait.
- [x] `provider/` - Reth-backed provider for RESS requests and witness generation.

### Files
- (none)
```

## File: crates/revm/src/AGENTS.md
```markdown
# src

## Purpose
Reth-specific adapters and helpers around the `revm` EVM, including storage-backed databases, cached reads, cancellation flags, and witness recording.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint, module wiring, and re-exports for revm integration.
- **Key items**: `cached`, `cancelled`, `database`, `witness` (feature), `test_utils`
- **Interactions**: Re-exports `revm` types and local helpers.

### `database.rs`
- **Role**: Glue layer that adapts reth state providers to `revm::Database`/`DatabaseRef`.
- **Key items**: `EvmStateProvider`, `StateProviderDatabase`
- **Interactions**: Uses `AccountReader`, `BlockHashReader`, `BytecodeReader`, `StateProvider`.
- **Knobs / invariants**: Missing data returns defaults (e.g., empty bytecode, zero hash).

### `cached.rs`
- **Role**: Read-through caching wrappers for repeated payload building.
- **Key items**: `CachedReads`, `CachedReadsDbMut`, `CachedReadsDBRef`, `CachedAccount`
- **Interactions**: Wraps a `DatabaseRef` to cache accounts, storage, bytecode, and block hashes.
- **Knobs / invariants**: `CachedReads` must outlive wrappers; cache assumes same base state.

### `cancelled.rs`
- **Role**: Cancellation markers for long-running execution/payload jobs.
- **Key items**: `CancelOnDrop`, `ManualCancel`
- **Knobs / invariants**: `CancelOnDrop` sets flag on drop; `ManualCancel` sets only on `cancel()`.

### `witness.rs`
- **Role**: Records accessed state during execution for witness generation (feature `witness`).
- **Key items**: `ExecutionWitnessRecord`, `record_executed_state()`, `from_executed_state()`
- **Interactions**: Reads from `revm::database::State` and builds `HashedPostState`.

### `test_utils.rs`
- **Role**: In-memory state provider for tests and fixtures.
- **Key items**: `StateProviderTest`
- **Interactions**: Implements `StateProvider` + proof/root traits with stubs.
- **Knobs / invariants**: Proof/root methods are `unimplemented!` for tests only.

## End-to-end flow (high level)
- Wrap a reth `StateProvider` in `StateProviderDatabase` to satisfy `revm::Database`.
- Optionally layer `CachedReads` over the database for repeated payload building.
- Use `CancelOnDrop`/`ManualCancel` to abort long-running jobs safely.
- Record execution access patterns via `ExecutionWitnessRecord` when witness generation is enabled.
- Use `StateProviderTest` for unit tests without full storage backends.

## Key APIs (no snippets)
- `StateProviderDatabase`, `EvmStateProvider`
- `CachedReads`, `CachedReadsDbMut`, `CachedReadsDBRef`
- `CancelOnDrop`, `ManualCancel`, `ExecutionWitnessRecord`
```

## File: crates/revm/AGENTS.md
```markdown
# revm

## Purpose
`reth-revm` crate: reth-specific adapters and helpers around `revm` execution.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Database adapters, cached reads, cancellation markers, and witness helpers.

### Files
- `Cargo.toml` - Manifest for revm utilities and feature flags.
  - **Key items**: features `witness`, `test-utils`, `serde`, `optional-checks`, `memory_limit`
```

## File: crates/rpc/ipc/src/client/AGENTS.md
```markdown
# client

## Purpose
IPC client transport for `jsonrpsee`, providing sender/receiver adapters and a client builder.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Implements the IPC transport sender/receiver and `IpcClientBuilder` wrapper around `jsonrpsee` client creation.
- **Key items**: `Sender`, `Receiver`, `IpcTransportClientBuilder`, `IpcClientBuilder`, `IpcError`
- **Interactions**: Uses `StreamCodec` for framing, `interprocess` sockets for connection, and `jsonrpsee` client traits.
- **Knobs / invariants**: `request_timeout` default is 60s; `send_ping` is not supported.

## End-to-end flow (high level)
- Build an IPC transport pair with `IpcTransportClientBuilder`.
- Connect via `LocalSocketStream` and wrap the receiver with `StreamCodec`.
- Use `IpcClientBuilder` to construct a `jsonrpsee` `Client`.
- Send requests through `Sender` and receive responses with `Receiver`.

## Key APIs (no snippets)
- `IpcClientBuilder`, `IpcTransportClientBuilder`
- `Sender`, `Receiver`, `IpcError`
```

## File: crates/rpc/ipc/src/server/AGENTS.md
```markdown
# server

## Purpose
IPC server implementation for JSON-RPC: connection handling, request processing, and middleware wiring.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Main IPC server implementation with settings, middleware builders, and connection lifecycle orchestration.
- **Key items**: `IpcServer`, `Settings`, `Builder`, `RpcServiceBuilder`, `TowerServiceNoHttp`, `ServiceData`
- **Interactions**: Uses `IpcConnDriver` from `connection` and `RpcService` from `rpc_service` to execute methods.
- **Knobs / invariants**: Limits for request/response size, connections, subscriptions, and message buffer capacity; optional socket permissions.

### `connection.rs`
- **Role**: IPC connection stream/sink adapter and driver that multiplexes requests and responses.
- **Key items**: `IpcConn`, `IpcConnDriver`, `JsonRpcStream`
- **Interactions**: Wraps `StreamCodec` framing and drives a `tower::Service` for RPC handling.
- **Knobs / invariants**: Flushes on readiness to avoid buffering stalls; uses a small budget to yield.

### `ipc.rs`
- **Role**: Request parsing and execution helpers for single and batch JSON-RPC calls.
- **Key items**: `process_single_request`, `process_batch_request`, `call_with_service`, `execute_call_with_tracing`
- **Interactions**: Calls `RpcServiceT` methods and builds JSON-RPC responses.
- **Knobs / invariants**: Enforces max request size; batches are assembled into a single response message.

### `rpc_service.rs`
- **Role**: Middleware wrapper implementing `RpcServiceT` to route calls and subscriptions.
- **Key items**: `RpcService`, `RpcServiceCfg`, `call`, `batch`
- **Interactions**: Uses `Methods` registry, `MethodSink` for subscriptions, and `BoundedSubscriptions` limits.
- **Knobs / invariants**: Rejects subscriptions when limits are exceeded; enforces response size limits.

## End-to-end flow (high level)
- Build an `IpcServer` with `Builder` and configure limits/middleware.
- Accept socket connections and wrap them with `StreamCodec` framing.
- Spawn an `IpcConnDriver` task to read requests and write responses.
- Parse single or batch requests and dispatch to `RpcService`.
- Enforce subscription and payload size limits, then send responses back.

## Key APIs (no snippets)
- `IpcServer`, `Builder`, `Settings`
- `IpcConn`, `IpcConnDriver`
- `RpcService`, `RpcServiceCfg`
```

## File: crates/rpc/ipc/src/AGENTS.md
```markdown
# src

## Purpose
IPC transport and server/client implementations for JSON-RPC over local sockets.

## Contents (one hop)
### Subdirectories
- [x] `client/` - IPC client transport wrappers and builders.
- [x] `server/` - IPC server, connection handling, and request processing.

### Files
- `lib.rs` - Crate entrypoint with module wiring and feature flags.
  - **Key items**: `client`, `server`, `stream_codec`
- `stream_codec.rs` - Streaming JSON codec for IPC/tcp transports.
  - **Key items**: `StreamCodec`, `Separator`, `stream_incoming()`
  - **Interactions**: Used by both client and server framing paths.

## Key APIs (no snippets)
- `StreamCodec`, `Separator`
```

## File: crates/rpc/ipc/AGENTS.md
```markdown
# ipc

## Purpose
`reth-ipc` crate: IPC transport support for `jsonrpsee` clients and servers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - IPC client/server implementations and stream codec.

### Files
- `Cargo.toml` - Manifest for IPC transport dependencies.
  - **Key items**: `jsonrpsee` (client/server), `interprocess`, `tokio`, `tokio-util`
- `README.md` - Crate overview and JSON-RPC IPC description.
  - **Key items**: `jsonrpsee` IPC transport
```

## File: crates/rpc/rpc/src/eth/helpers/AGENTS.md
```markdown
# helpers

## Purpose
Helper implementations and utilities for the `eth` namespace: trait adapters, signer helpers, pending block support, and sync utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for `eth` helper implementations and re-exports.
- **Key items**: `signer`, `sync_listener`, `types`
- **Interactions**: Re-exports `SyncListener` for `eth` APIs.

### `block.rs`
- **Role**: Implements block-related helper traits for `EthApi`.
- **Key items**: `EthBlocks`, `LoadBlock`, `EthApi`
- **Interactions**: Uses `RpcConvert` and `RpcNodeCore` trait bounds.

### `call.rs`
- **Role**: Implements call/estimate helpers for EVM execution endpoints.
- **Key items**: `EthCall`, `Call`, `EstimateCall`, `call_gas_limit()`, `max_simulate_blocks()`
- **Interactions**: Delegates limits to `EthApi` inner config.
- **Knobs / invariants**: Exposes gas cap, simulate block cap, and EVM memory limits.

### `fees.rs`
- **Role**: Fee history and gas oracle helpers for `EthApi`.
- **Key items**: `EthFees`, `LoadFee`, `GasPriceOracle`, `FeeHistoryCache`
- **Interactions**: Reads provider headers and cached fee history.

### `pending_block.rs`
- **Role**: Pending-block helpers and access to pending block state.
- **Key items**: `LoadPendingBlock`, `PendingEnvBuilder`, `PendingBlock`, `PendingBlockKind`
- **Interactions**: Accesses `EthApi` pending block mutex and env builder.

### `receipt.rs`
- **Role**: Receipt loading adapter for `EthApi`.
- **Key items**: `LoadReceipt`, `EthApi`
- **Interactions**: Ties `RpcConvert` to receipt conversion.

### `signer.rs`
- **Role**: Developer signer implementation for `eth_sign`/tx signing.
- **Key items**: `DevSigner`, `random_signers()`, `from_mnemonic()`, `sign_transaction()`, `sign_typed_data()`
- **Interactions**: Implements `EthSigner` over local private keys.
- **Knobs / invariants**: Mnemonic derivation uses `m/44'/60'/0'/0/{index}` path.

### `spec.rs`
- **Role**: Spec helper trait implementation for `EthApi`.
- **Key items**: `EthApiSpec`, `starting_block()`
- **Interactions**: Uses `RpcNodeCore` and `RpcConvert` bounds.

### `state.rs`
- **Role**: State access helpers for account/storage/proof queries.
- **Key items**: `EthState`, `LoadState`, `max_proof_window()`
- **Interactions**: Requires `LoadPendingBlock` for consistent state reads.
- **Knobs / invariants**: Proof window limit is configured via `eth_proof_window`.

### `sync_listener.rs`
- **Role**: Utility future that waits for sync completion.
- **Key items**: `SyncListener`, `new()`, `Future` impl
- **Interactions**: Polls `NetworkInfo::is_syncing` on a tick stream.
- **Knobs / invariants**: Completes immediately if not syncing.

### `trace.rs`
- **Role**: Trace helper trait implementation for `EthApi`.
- **Key items**: `Trace`, `EthApi`
- **Interactions**: Binds `RpcConvert` and EVM error conversions.

### `transaction.rs`
- **Role**: Transaction helpers: sending raw txs, signer access, pool integration.
- **Key items**: `EthTransactions`, `LoadTransaction`, `send_transaction()`, `send_raw_transaction_sync_timeout()`, `signers()`
- **Interactions**: Uses `TransactionPool`, blob sidecar conversion, and optional raw-tx forwarder.
- **Knobs / invariants**: Converts legacy blob sidecars to EIP-7594 when Osaka is active.

### `types.rs`
- **Role**: Ethereum-specific RPC converter type alias.
- **Key items**: `EthRpcConverter`
- **Interactions**: Combines `RpcConverter` with `EthEvmConfig` and receipt converter.

## End-to-end flow (high level)
- `EthApi` builds with an `EthRpcConverter` for Ethereum types.
- Helper traits (`EthBlocks`, `EthCall`, `EthFees`, `EthState`, `Trace`) are implemented for `EthApi`.
- Pending-block helpers provide access to local pending execution context.
- Transaction helpers validate/forward raw transactions and integrate with the pool.
- Fee/history helpers read cached fee data and gas oracle configuration.
- `SyncListener` exposes an awaitable sync-completion future for clients.

## Key APIs (no snippets)
- `EthRpcConverter`, `DevSigner`, `SyncListener`
- `EthBlocks`, `EthCall`, `EthFees`, `EthState`, `EthTransactions`
```

## File: crates/rpc/rpc/src/eth/AGENTS.md
```markdown
# eth

## Purpose
Implementation of the `eth` namespace: core API type, builder/config, filters, pubsub, bundles, and simulation helpers.

## Contents (one hop)
### Subdirectories
- [x] `helpers/` - Trait adapters, signer helpers, pending block support, and sync utilities.

### Files
- `mod.rs` - Module wiring and re-exports for `eth` handlers.
  - **Key items**: `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`, `EthBundle`
- `builder.rs` - Configurable builder for constructing `EthApi` instances.
  - **Key items**: `EthApiBuilder`, `GasCap`, `FeeHistoryCacheConfig`, `EthStateCacheConfig`, `PendingBlockKind`
  - **Interactions**: Wires task spawners, caches, and limits into `EthApiInner`.
- `core.rs` - Core `EthApi` implementation and shared types.
  - **Key items**: `EthApi`, `EthApiInner`, `EthApiFor`, `EthApiBuilderFor`, `EthRpcConverterFor`
  - **Interactions**: Centralizes provider/pool/network access and EVM execution helpers.
- `filter.rs` - `eth_getLogs` and filter lifecycle management.
  - **Key items**: `EthFilter`, `ActiveFilters`, `EthFilterError`, `QueryLimits`
  - **Interactions**: Reads headers/receipts, uses pool streams, and evicts stale filters.
- `pubsub.rs` - `eth_subscribe` implementation for headers/logs/txs/syncing.
  - **Key items**: `EthPubSub`, `SubscriptionSerializeError`
  - **Interactions**: Consumes canonical state broadcasts and pool events.
- `bundle.rs` - MEV bundle simulation for `eth_callBundle`.
  - **Key items**: `EthBundle`, `EthBundleError`, `call_bundle()`
  - **Interactions**: Executes bundles via EVM with state at specified block.
- `sim_bundle.rs` - MEV bundle simulation for `mev_simBundle`.
  - **Key items**: `EthSimBundle`, `FlattenedBundleItem`, `EthSimBundleError`
  - **Interactions**: Flattens nested bundles and simulates with EVM.

## Key APIs (no snippets)
- `EthApi`, `EthApiBuilder`
- `EthFilter`, `EthPubSub`
- `EthBundle`, `EthSimBundle`
```

## File: crates/rpc/rpc/src/AGENTS.md
```markdown
# src

## Purpose
Concrete RPC handlers for reth namespaces (`eth`, `admin`, `debug`, `trace`, `txpool`, etc.).

## Contents (one hop)
### Subdirectories
- [x] `eth/` - `eth_` namespace core, filters, pubsub, and bundle simulation.

### Files
- `lib.rs` - Crate entrypoint and RPC handler re-exports.
  - **Key items**: `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`, `AdminApi`, `DebugApi`, `TraceApi`
- `admin.rs` - `admin_` namespace handlers (peers, node info, txpool maintenance).
  - **Key items**: `AdminApi`, `add_peer()`, `peers()`, `node_info()`, `clear_txpool()`
- `aliases.rs` - Type aliases for dynamic RPC converter usage.
  - **Key items**: `DynRpcConverter`
- `debug.rs` - `debug_` namespace handlers and tracing helpers.
  - **Key items**: `DebugApi`, `debug_trace_block()`, `debug_trace_raw_block()`, `BadBlockStore`
- `engine.rs` - `engine_`-compatible `eth_` subset wrapper.
  - **Key items**: `EngineEthApi`, `syncing()`, `block_by_number()`, `logs()`
- `miner.rs` - `miner_` namespace stub implementation.
  - **Key items**: `MinerApi`, `set_extra()`, `set_gas_price()`, `set_gas_limit()`
- `net.rs` - `net_` namespace implementation.
  - **Key items**: `NetApi`, `version()`, `peer_count()`, `is_listening()`
- `otterscan.rs` - Otterscan and Erigon compatibility endpoints.
  - **Key items**: `OtterscanApi`, `get_block_details()`, `get_internal_operations()`, `trace_transaction()`
- `reth.rs` - `reth_` namespace for prototype endpoints and subscriptions.
  - **Key items**: `RethApi`, `balance_changes_in_block()`, `reth_subscribe_chain_notifications()`
- `rpc.rs` - `rpc_` namespace implementation (`rpc_modules`).
  - **Key items**: `RPCApi`, `rpc_modules()`
- `testing.rs` - `testing_` namespace block builder (non-production).
  - **Key items**: `TestingApi`, `build_block_v1()`, `with_skip_invalid_transactions()`
- `trace.rs` - Parity-style `trace_` namespace implementation.
  - **Key items**: `TraceApi`, `trace_call()`, `trace_raw_transaction()`, `trace_call_many()`
- `txpool.rs` - `txpool_` namespace implementation.
  - **Key items**: `TxPoolApi`, `txpool_status()`, `txpool_inspect()`, `txpool_content()`
- `validation.rs` - Builder validation namespace for payload/bid verification.
  - **Key items**: `ValidationApi`, `ValidationApiConfig`, `ValidationApiError`, `validate_message_against_block()`
- `web3.rs` - `web3_` namespace handlers.
  - **Key items**: `Web3Api`, `client_version()`, `sha3()`

## Key APIs (no snippets)
- `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`
- `AdminApi`, `DebugApi`, `TraceApi`, `TxPoolApi`
```

## File: crates/rpc/rpc/AGENTS.md
```markdown
# rpc

## Purpose
`reth-rpc` crate: concrete RPC server implementations for all namespaces over JSON-RPC.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC handlers for `eth`, `debug`, `trace`, `admin`, `net`, `txpool`, etc.

### Files
- `Cargo.toml` - Manifest for RPC implementations and feature flags.
  - **Key items**: feature `js-tracer`, dependencies on `reth-rpc-eth-api`, `jsonrpsee`
```

## File: crates/rpc/rpc-api/src/AGENTS.md
```markdown
# src

## Purpose
RPC interface definitions for all namespaces (traits and JSON-RPC method signatures).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports of all server/client traits.
- **Key items**: `servers`, `clients`, `TestingBuildBlockRequestV1`, `TESTING_BUILD_BLOCK_V1`
- **Interactions**: Re-exports `reth_rpc_eth_api` server/client traits.

### `admin.rs`
- **Role**: Admin namespace trait definitions.
- **Key items**: `AdminApi`, `add_peer()`, `remove_peer()`, `peers()`, `node_info()`

### `anvil.rs`
- **Role**: Anvil-compatible namespace trait definitions.
- **Key items**: `AnvilApi`, `anvil_impersonate_account()`, `anvil_mine()`, `anvil_reset()`, `anvil_dump_state()`

### `debug.rs`
- **Role**: Debug namespace trait definitions for raw block/tx access and tracing.
- **Key items**: `DebugApi`, `debug_trace_block()`, `debug_trace_transaction()`, `debug_execution_witness()`

### `engine.rs`
- **Role**: Engine API trait definitions and module adapter for consensus clients.
- **Key items**: `EngineApi`, `EngineEthApi`, `IntoEngineApiRpcModule`, `new_payload_v3()`, `fork_choice_updated_v3()`

### `hardhat.rs`
- **Role**: Hardhat-compatible namespace trait definitions.
- **Key items**: `HardhatApi`, `hardhat_impersonate_account()`, `hardhat_set_balance()`, `hardhat_reset()`

### `mev.rs`
- **Role**: MEV namespace trait definitions for bundle submission/simulation.
- **Key items**: `MevSimApi`, `MevFullApi`, `sim_bundle()`, `send_bundle()`

### `miner.rs`
- **Role**: Miner namespace trait definitions.
- **Key items**: `MinerApi`, `set_extra()`, `set_gas_price()`, `set_gas_limit()`

### `net.rs`
- **Role**: Net namespace trait definitions.
- **Key items**: `NetApi`, `version()`, `peer_count()`, `is_listening()`

### `otterscan.rs`
- **Role**: Otterscan-compatible namespace trait definitions.
- **Key items**: `Otterscan`, `get_header_by_number()`, `get_block_details()`, `trace_transaction()`

### `reth.rs`
- **Role**: Reth-specific namespace trait definitions.
- **Key items**: `RethApi`, `reth_get_balance_changes_in_block()`, `reth_subscribe_chain_notifications()`

### `rpc.rs`
- **Role**: RPC namespace trait definitions.
- **Key items**: `RpcApi`, `rpc_modules()`

### `testing.rs`
- **Role**: Testing namespace trait definitions for block-building RPC.
- **Key items**: `TestingApi`, `TestingBuildBlockRequestV1`, `TESTING_BUILD_BLOCK_V1`, `build_block_v1()`

### `trace.rs`
- **Role**: Trace namespace trait definitions for parity-style tracing.
- **Key items**: `TraceApi`, `trace_call()`, `trace_raw_transaction()`, `trace_filter()`

### `txpool.rs`
- **Role**: Txpool namespace trait definitions.
- **Key items**: `TxPoolApi`, `txpool_status()`, `txpool_inspect()`, `txpool_content()`

### `validation.rs`
- **Role**: Block submission validation namespace trait definitions.
- **Key items**: `BlockSubmissionValidationApi`, `validate_builder_submission_v1()`, `validate_builder_submission_v5()`

### `web3.rs`
- **Role**: Web3 namespace trait definitions.
- **Key items**: `Web3Api`, `client_version()`, `sha3()`

## End-to-end flow (high level)
- Namespace trait definitions are declared with `jsonrpsee` macros.
- The `servers` module re-exports all server-side traits for implementation.
- The optional `clients` module re-exports client-side traits when enabled.
- Downstream crates implement these traits to provide concrete RPC handlers.

## Key APIs (no snippets)
- `AdminApi`, `EngineApi`, `EthApiServer`, `TraceApi`
- `MevSimApi`, `TestingApi`, `BlockSubmissionValidationApi`
```

## File: crates/rpc/rpc-api/AGENTS.md
```markdown
# rpc-api

## Purpose
`reth-rpc-api` crate: JSON-RPC interface definitions and trait exports for all namespaces.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC trait definitions for admin, engine, eth, trace, etc.

### Files
- `Cargo.toml` - Manifest for RPC interface traits and client feature.
  - **Key items**: feature `client`, `jsonrpsee` macros, `reth-rpc-eth-api`
```

## File: crates/rpc/rpc-builder/src/AGENTS.md
```markdown
# src

## Purpose
RPC server configuration and builder utilities: module selection, server setup, middleware, and metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Main RPC builder API for assembling modules and starting servers.
- **Key items**: `RpcModuleBuilder`, `RpcModuleConfig`, `RpcServerConfig`, `TransportRpcModuleConfig`, `TransportRpcModules`
- **Interactions**: Wires `reth-rpc` handlers with `jsonrpsee` servers and transport configs.

### `auth.rs`
- **Role**: Auth (engine API) server configuration and startup helpers.
- **Key items**: `AuthServerConfig`, `AuthServerConfigBuilder`, `AuthRpcModule`, `AuthServerHandle`
- **Interactions**: Applies JWT auth middleware and optional IPC server startup.

### `config.rs`
- **Role**: RPC server config trait for CLI integration.
- **Key items**: `RethRpcServerConfig`, `transport_rpc_module_config()`, `rpc_server_config()`, `auth_server_config()`
- **Interactions**: Builds module selections and server configs from CLI args.

### `cors.rs`
- **Role**: CORS parsing and layer creation.
- **Key items**: `CorsDomainError`, `create_cors_layer()`
- **Knobs / invariants**: Disallows wildcard mixed with explicit domains.

### `error.rs`
- **Role**: RPC server error types and conflict detection.
- **Key items**: `RpcError`, `ServerKind`, `ConflictingModules`, `WsHttpSamePortError`
- **Knobs / invariants**: Distinguishes address-in-use vs generic start failures.

### `eth.rs`
- **Role**: Bundles `eth` core, filter, and pubsub handlers.
- **Key items**: `EthHandlers`, `bootstrap()`
- **Interactions**: Creates `EthFilter` and `EthPubSub` alongside main `EthApi`.

### `metrics.rs`
- **Role**: Metrics middleware for RPC requests and connections.
- **Key items**: `RpcRequestMetrics`, `RpcRequestMetricsService`, `MeteredRequestFuture`, `MeteredBatchRequestsFuture`
- **Interactions**: Wraps `RpcServiceT` to record method timings and counts.

### `middleware.rs`
- **Role**: Trait alias for supported RPC middleware layers.
- **Key items**: `RethRpcMiddleware`

### `rate_limiter.rs`
- **Role**: Rate limiting middleware for expensive methods.
- **Key items**: `RpcRequestRateLimiter`, `RpcRequestRateLimitingService`, `RateLimitingRequestFuture`
- **Knobs / invariants**: Applies semaphore limits to `trace_` and `debug_` calls.

## End-to-end flow (high level)
- Build RPC handlers via `RpcModuleBuilder` with provider/pool/network/consensus.
- Configure module selection per transport in `TransportRpcModuleConfig`.
- Start HTTP/WS/IPC servers via `RpcServerConfig` or auth server via `AuthServerConfig`.
- Apply middleware layers (auth, CORS, rate limiting, metrics).
- Serve `jsonrpsee` modules with the configured API set.

## Key APIs (no snippets)
- `RpcModuleBuilder`, `RpcServerConfig`, `TransportRpcModules`
- `AuthServerConfig`, `AuthRpcModule`
- `RpcRequestMetrics`, `RpcRequestRateLimiter`
```

## File: crates/rpc/rpc-builder/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for RPC builder configuration, auth server, middleware, and transport startup.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for integration tests.
- **Key items**: `auth`, `http`, `middleware`, `serde`, `startup`, `utils`

### `auth.rs`
- **Role**: Auth server integration tests for engine API endpoints.
- **Key items**: `test_basic_engine_calls()`, `test_auth_endpoints_http()`, `test_auth_endpoints_ws()`

### `http.rs`
- **Role**: HTTP/WS RPC endpoint compatibility tests.
- **Key items**: `RawRpcParamsBuilder`, `test_rpc_call_ok()`, `test_filter_calls()`, `test_basic_eth_calls()`

### `middleware.rs`
- **Role**: Custom RPC middleware integration test.
- **Key items**: `MyMiddlewareLayer`, `MyMiddlewareService`, `test_rpc_middleware()`

### `serde.rs`
- **Role**: Serde and raw-params handling tests.
- **Key items**: `RawRpcParams`, `test_eth_balance_serde()`

### `startup.rs`
- **Role**: Server startup and port conflict tests.
- **Key items**: `test_http_addr_in_use()`, `test_ws_addr_in_use()`, `test_launch_same_port_different_modules()`

### `utils.rs`
- **Role**: Test helpers to launch RPC servers and build test modules.
- **Key items**: `test_address()`, `launch_auth()`, `launch_http()`, `launch_ws()`, `test_rpc_builder()`

## End-to-end flow (high level)
- Build test RPC modules with `test_rpc_builder`.
- Launch HTTP/WS/auth servers with helper utilities.
- Exercise core namespaces and engine API endpoints.
- Validate middleware hooks, serde behavior, and startup error handling.

## Key APIs (no snippets)
- `launch_auth()`, `launch_http()`, `launch_ws()`
- `test_basic_engine_calls()`, `test_rpc_middleware()`
```

## File: crates/rpc/rpc-builder/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for RPC builder and server setup.

## Contents (one hop)
### Subdirectories
- [x] `it/` - RPC builder/auth/transport integration tests.

### Files
- (none)
```

## File: crates/rpc/rpc-builder/AGENTS.md
```markdown
# rpc-builder

## Purpose
`reth-rpc-builder` crate: configure RPC modules and start HTTP/WS/IPC/auth servers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC builders, configs, middleware, and metrics.
- [x] `tests/` - Integration tests for RPC server startup and middleware.

### Files
- `Cargo.toml` - Manifest for RPC builder utilities and server dependencies.
  - **Key items**: `reth-ipc`, `jsonrpsee`, `reth-rpc`, `reth-rpc-layer`
```

## File: crates/rpc/rpc-convert/src/AGENTS.md
```markdown
# src

## Purpose
Compatibility and conversion helpers between reth primitives and RPC types.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports for conversion traits.
- **Key items**: `TryFromBlockResponse`, `TryFromReceiptResponse`, `RpcConvert`, `RpcConverter`
- **Interactions**: Re-exports call fee helpers from `alloy_evm::rpc`.

### `block.rs`
- **Role**: Block response conversion trait definitions.
- **Key items**: `TryFromBlockResponse`, `from_block_response()`

### `receipt.rs`
- **Role**: Receipt response conversion trait definitions.
- **Key items**: `TryFromReceiptResponse`, `from_receipt_response()`

### `rpc.rs`
- **Role**: RPC type adapters and signable request abstraction.
- **Key items**: `RpcTypes`, `RpcTransaction`, `RpcReceipt`, `RpcHeader`, `RpcTxReq`, `SignableTxRequest`
- **Knobs / invariants**: `SignableTxRequest` validates typed tx construction.

### `transaction.rs`
- **Role**: Core conversion logic between consensus and RPC transaction types.
- **Key items**: `RpcConvert`, `ConvertReceiptInput`, `ReceiptConverter`, `HeaderConverter`, `FromConsensusHeader`
- **Interactions**: Bridges EVM env (`TxEnvFor`) and RPC request conversions.

## End-to-end flow (high level)
- Use `RpcTypes` to bind a network's RPC response types.
- Convert blocks/receipts with `TryFromBlockResponse` and `TryFromReceiptResponse`.
- Convert transaction requests and responses via `RpcConvert`/`RpcConverter`.
- Build EVM environments from RPC requests using `SignableTxRequest` helpers.

## Key APIs (no snippets)
- `RpcConvert`, `RpcConverter`, `RpcTypes`
- `TryFromBlockResponse`, `TryFromReceiptResponse`
```

## File: crates/rpc/rpc-convert/AGENTS.md
```markdown
# rpc-convert

## Purpose
`reth-rpc-convert` crate: conversion layer between reth primitives and RPC types.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Conversion traits and RPC type adapters.

### Files
- `Cargo.toml` - Manifest for RPC conversion utilities and optional OP support.
  - **Key items**: feature `op`, `reth-evm`, `alloy-rpc-types-eth`
```

## File: crates/rpc/rpc-e2e-tests/src/AGENTS.md
```markdown
# src

## Purpose
End-to-end RPC compatibility testing utilities and actions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and module wiring.
- **Key items**: `rpc_compat`

### `rpc_compat.rs`
- **Role**: Actions for running execution-apis RPC compatibility tests.
- **Key items**: `RpcTestCase`, `RunRpcCompatTests`, `InitializeFromExecutionApis`, `parse_io_file()`, `compare_json_values()`
- **Interactions**: Integrates with `reth-e2e-test-utils` action framework.

## End-to-end flow (high level)
- Load execution-apis test data from `.io` files.
- Convert test cases into `RpcTestCase` structs.
- Execute JSON-RPC requests against node clients.
- Compare actual responses against expected fixtures.

## Key APIs (no snippets)
- `RunRpcCompatTests`, `RpcTestCase`
```

## File: crates/rpc/rpc-e2e-tests/tests/e2e-testsuite/AGENTS.md
```markdown
# e2e-testsuite

## Purpose
Execution-apis RPC compatibility test suite entrypoints.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Runs local and external execution-apis compatibility tests.
- **Key items**: `test_local_rpc_tests_compat()`, `test_execution_apis_compat()`, `RunRpcCompatTests`
- **Interactions**: Uses `InitializeFromExecutionApis` and e2e test framework actions.

## End-to-end flow (high level)
- Load test data paths (repo-local or env-based).
- Initialize chain state from RLP + forkchoice.
- Run compatibility test actions across discovered methods.

## Key APIs (no snippets)
- `RunRpcCompatTests`, `InitializeFromExecutionApis`
```

## File: crates/rpc/rpc-e2e-tests/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration test suite for RPC compatibility.

## Contents (one hop)
### Subdirectories
- [x] `e2e-testsuite/` - Execution-apis compatibility test entrypoints.

### Files
- (none)
```

## File: crates/rpc/rpc-e2e-tests/AGENTS.md
```markdown
# rpc-e2e-tests

## Purpose
`reth-rpc-e2e-tests` crate: end-to-end RPC compatibility tests (execution-apis).

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC compatibility test actions.
- [x] `tests/` - Test suite entrypoints for compatibility runs.
- [x] `testdata/` - (skip: execution-apis fixtures and JSON/IO test data).

### Files
- `Cargo.toml` - Manifest for RPC e2e testing dependencies.
  - **Key items**: `reth-e2e-test-utils`, `reth-rpc-api` (client)
- `README.md` - Usage and architecture for compatibility testing.
  - **Key items**: `RunRpcCompatTests`, execution-apis `.io` format
```

## File: crates/rpc/rpc-engine-api/src/AGENTS.md
```markdown
# src

## Purpose
Engine API implementation for consensus-layer interaction (payloads, forkchoice, blobs).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports for engine API implementation.
- **Key items**: `EngineApi`, `EngineApiSender`, `EngineCapabilities`, `EngineApiError`

### `capabilities.rs`
- **Role**: Supported engine capabilities list and management.
- **Key items**: `CAPABILITIES`, `EngineCapabilities`, `list()`, `add_capability()`, `remove_capability()`

### `engine_api.rs`
- **Role**: Core Engine API handler implementation and methods.
- **Key items**: `EngineApi`, `EngineApiSender`, `new_payload_v1()`, `fork_choice_updated_v3()`, `get_payload_v5()`
- **Knobs / invariants**: Enforces payload/body/blob request limits.

### `error.rs`
- **Role**: Engine API error types and RPC error mapping.
- **Key items**: `EngineApiError`, `EngineApiResult`, `INVALID_PAYLOAD_ATTRIBUTES`, `REQUEST_TOO_LARGE_CODE`

### `metrics.rs`
- **Role**: Metrics for engine API latency and blob request counts.
- **Key items**: `EngineApiMetrics`, `EngineApiLatencyMetrics`, `BlobMetrics`

## End-to-end flow (high level)
- Construct `EngineApi` with provider, payload store, and consensus handle.
- Validate payloads and forkchoice updates per engine spec versions.
- Serve payload retrieval and blob queries with size limits.
- Emit metrics for latency and blob cache hits/misses.

## Key APIs (no snippets)
- `EngineApi`, `EngineApiError`, `EngineCapabilities`
```

## File: crates/rpc/rpc-engine-api/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for engine payload encoding and validation helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for payload tests.
- **Key items**: `payload`

### `payload.rs`
- **Role**: Payload conversion and validation test cases.
- **Key items**: `payload_body_roundtrip()`, `payload_validation_conversion()`, `ExecutionPayloadV1`

## End-to-end flow (high level)
- Generate random blocks and payload bodies.
- Verify payload body transaction decoding roundtrips.
- Validate payload conversion error cases.

## Key APIs (no snippets)
- `ExecutionPayloadV1`, `ExecutionPayloadBodyV1`
```

## File: crates/rpc/rpc-engine-api/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for engine API helpers.

## Contents (one hop)
### Subdirectories
- [x] `it/` - Payload encoding/validation tests.

### Files
- (none)
```

## File: crates/rpc/rpc-engine-api/AGENTS.md
```markdown
# rpc-engine-api

## Purpose
`reth-rpc-engine-api` crate: implementation of the Engine API for consensus clients.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Engine API implementation, errors, and metrics.
- [x] `tests/` - Integration tests for payload conversion.

### Files
- `Cargo.toml` - Manifest for engine API implementation and metrics.
  - **Key items**: `reth-payload-builder`, `reth-rpc-api`, `reth-engine-primitives`
```

## File: crates/rpc/rpc-eth-api/src/helpers/AGENTS.md
```markdown
# helpers

## Purpose
Trait definitions and helper utilities for `eth_` RPC handling (loaders, execution helpers, config).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for `eth` helper traits.
- **Key items**: `EthBlocks`, `EthTransactions`, `EthFees`, `EthState`, `EthCall`, `TraceExt`

### `block.rs`
- **Role**: Block loading traits and helpers.
- **Key items**: `EthBlocks`, `LoadBlock`

### `blocking_task.rs`
- **Role**: Spawn-blocking helper trait for offloading heavy RPC work.
- **Key items**: `SpawnBlocking`

### `call.rs`
- **Role**: Call execution traits and EVM call helpers.
- **Key items**: `EthCall`, `Call`

### `config.rs`
- **Role**: `eth_config` endpoint and fork/precompile config builder.
- **Key items**: `EthConfigApi`, `EthConfigHandler`, `config()`

### `estimate.rs`
- **Role**: Gas estimation logic and helper trait.
- **Key items**: `EstimateCall`, `estimate_gas_with()`
- **Knobs / invariants**: Disables EIP-3607/basefee for estimate; uses binary search.

### `fee.rs`
- **Role**: Fee history and gas oracle helpers.
- **Key items**: `EthFees`, `LoadFee`, `fee_history()`, `gas_price()`

### `pending_block.rs`
- **Role**: Pending block access and environment builders.
- **Key items**: `LoadPendingBlock`, `PendingEnvBuilder`, `BuildPendingEnv`

### `receipt.rs`
- **Role**: Receipt loading trait for `eth_` endpoints.
- **Key items**: `LoadReceipt`

### `signer.rs`
- **Role**: Signing abstraction for transaction requests.
- **Key items**: `EthSigner`

### `spec.rs`
- **Role**: Chain spec accessor trait for `eth_` requests.
- **Key items**: `EthApiSpec`

### `state.rs`
- **Role**: State access traits for account/storage/proofs.
- **Key items**: `EthState`, `LoadState`

### `trace.rs`
- **Role**: Tracing trait for transaction/block traces.
- **Key items**: `Trace`

### `transaction.rs`
- **Role**: Transaction submission and lookup traits.
- **Key items**: `EthTransactions`, `LoadTransaction`

## End-to-end flow (high level)
- Implement `Load*` traits to provide database access primitives.
- Compose `Eth*` traits to expose RPC behaviors by namespace.
- Use `SpawnBlocking` for CPU or blocking IO operations.
- Extend with `TraceExt` for tracing-specific flows.

## Key APIs (no snippets)
- `EthBlocks`, `EthTransactions`, `EthFees`, `EthState`, `EthCall`
- `LoadBlock`, `LoadTransaction`, `LoadState`, `LoadFee`
```

## File: crates/rpc/rpc-eth-api/src/AGENTS.md
```markdown
# src

## Purpose
Shared `eth_` namespace API traits, type aliases, and server/client interfaces.

## Contents (one hop)
### Subdirectories
- [x] `helpers/` - Helper traits for loading data, calls, fees, state, and tracing.

### Files
- `lib.rs` - Module wiring and re-exports for `eth_` APIs.
  - **Key items**: `EthApiServer`, `EthFilterApiServer`, `EthPubSubApiServer`, `RpcNodeCore`
- `bundle.rs` - Bundle RPC trait definitions.
  - **Key items**: `EthCallBundleApi`, `EthBundleApi`
- `core.rs` - Core `eth_` API server trait definitions.
  - **Key items**: `EthApiServer`, `FullEthApiServer`
- `ext.rs` - L2 `eth_` extension traits.
  - **Key items**: `L2EthApiExt`
- `filter.rs` - Filter RPC trait definitions and query limits.
  - **Key items**: `EthFilterApi`, `EngineEthFilter`, `QueryLimits`
- `node.rs` - RPC node core abstraction traits.
  - **Key items**: `RpcNodeCore`, `RpcNodeCoreExt`
- `pubsub.rs` - PubSub RPC trait definitions.
  - **Key items**: `EthPubSubApi`
- `types.rs` - Core RPC type aliases and trait bounds.
  - **Key items**: `EthApiTypes`, `FullEthApiTypes`, `RpcBlock`, `RpcTransaction`

## Key APIs (no snippets)
- `EthApiServer`, `EthFilterApiServer`, `EthPubSubApi`
- `RpcNodeCore`, `EthApiTypes`
```

## File: crates/rpc/rpc-eth-api/AGENTS.md
```markdown
# rpc-eth-api

## Purpose
`reth-rpc-eth-api` crate: shared `eth_` namespace traits, helpers, and RPC type aliases.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core `eth_` API traits, helpers, and types.

### Files
- `Cargo.toml` - Manifest for `eth` RPC API traits and features.
  - **Key items**: features `js-tracer`, `client`, `op`
```

## File: crates/rpc/rpc-eth-types/src/builder/AGENTS.md
```markdown
# builder

## Purpose
Configuration types for `eth` namespace RPC behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for builder/config types.
- **Key items**: `config`

### `config.rs`
- **Role**: Defines `eth` RPC configuration structures and defaults.
- **Key items**: `EthConfig`, `EthFilterConfig`, `PendingBlockKind`, `DEFAULT_STALE_FILTER_TTL`
- **Knobs / invariants**: Gas caps, proof window, tracing limits, cache sizes, and pending block behavior.

## End-to-end flow (high level)
- Build `EthConfig` with defaults or CLI overrides.
- Derive `EthFilterConfig` and pending-block behavior from config.
- Feed config into RPC handlers and caches.

## Key APIs (no snippets)
- `EthConfig`, `EthFilterConfig`, `PendingBlockKind`
```

## File: crates/rpc/rpc-eth-types/src/cache/AGENTS.md
```markdown
# cache

## Purpose
Async caching layer for `eth` RPC data (blocks, receipts, headers).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Cache service frontend and task wiring.
- **Key items**: `EthStateCache`, `EthStateCacheConfig`, `CacheAction`
- **Interactions**: Spawns cache service tasks and handles requests via channels.

### `config.rs`
- **Role**: Cache size and concurrency configuration.
- **Key items**: `EthStateCacheConfig`, `max_blocks`, `max_receipts`, `max_headers`

### `db.rs`
- **Role**: Helper types for state provider trait objects.
- **Key items**: `StateCacheDb`, `StateProviderTraitObjWrapper`
- **Knobs / invariants**: Wraps trait objects to avoid HRTB lifetime issues.

### `metrics.rs`
- **Role**: Metrics for cache hit/miss and memory usage.
- **Key items**: `CacheMetrics`

### `multi_consumer.rs`
- **Role**: LRU cache with queued consumers on miss.
- **Key items**: `MultiConsumerLruCache`, `queue()`, `insert()`, `update_cached_metrics()`

## End-to-end flow (high level)
- Create `EthStateCache` with size and concurrency limits.
- Cache service fetches blocks/receipts/headers from provider on misses.
- LRU and queued consumers coordinate concurrent requests.
- Metrics report cache hits/misses and memory usage.

## Key APIs (no snippets)
- `EthStateCache`, `EthStateCacheConfig`, `MultiConsumerLruCache`
```

## File: crates/rpc/rpc-eth-types/src/error/AGENTS.md
```markdown
# error

## Purpose
Error types and conversion helpers for `eth` RPC.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core `eth` RPC error types and mapping helpers.
- **Key items**: `EthApiError`, `EthResult`, `RpcInvalidTransactionError`, `RpcPoolError`, `SignError`

### `api.rs`
- **Role**: Error conversion traits for wrapping core errors.
- **Key items**: `FromEthApiError`, `IntoEthApiError`, `AsEthApiError`, `FromEvmError`, `FromRevert`

## End-to-end flow (high level)
- Convert lower-level errors into `EthApiError`.
- Map invalid transactions to specific RPC error codes.
- Expose conversion traits for EVM and provider errors.

## Key APIs (no snippets)
- `EthApiError`, `RpcInvalidTransactionError`, `FromEvmError`
```

## File: crates/rpc/rpc-eth-types/src/AGENTS.md
```markdown
# src

## Purpose
Types, configs, caches, and helpers for implementing `eth` RPC server behavior.

## Contents (one hop)
### Subdirectories
- [x] `builder/` - `eth` namespace configuration types.
- [x] `cache/` - Async caching layer for blocks/receipts/headers.
- [x] `error/` - `eth` RPC error types and conversion helpers.

### Files
- `lib.rs` - Module wiring and public re-exports.
  - **Key items**: `EthConfig`, `EthStateCache`, `EthApiError`, `FeeHistoryCache`, `GasPriceOracle`
- `block.rs` - Block + receipts helper types and conversions.
  - **Key items**: `BlockAndReceipts`, `convert_transaction_receipt()`
- `fee_history.rs` - Fee history cache and entry types.
  - **Key items**: `FeeHistoryCache`, `FeeHistoryCacheConfig`, `FeeHistoryEntry`
- `gas_oracle.rs` - Gas price oracle implementation and config.
  - **Key items**: `GasPriceOracle`, `GasPriceOracleConfig`, `GasPriceOracleResult`, `GasCap`
- `id_provider.rs` - Subscription ID provider for `eth` RPC.
  - **Key items**: `EthSubscriptionIdProvider`
- `logs_utils.rs` - Log filtering helpers for `eth_getLogs`.
  - **Key items**: `ProviderOrBlock`, `matching_block_logs_with_tx_hashes()`, `FilterBlockRangeError`
- `pending_block.rs` - Pending block structures and helpers.
  - **Key items**: `PendingBlockEnv`, `PendingBlockEnvOrigin`, `PendingBlock`, `PendingBlockAndReceipts`
- `receipt.rs` - Receipt conversion helpers.
  - **Key items**: `EthReceiptConverter`
- `simulate.rs` - Simulation error types for `eth_simulate`-style calls.
  - **Key items**: `EthSimulateError`
- `transaction.rs` - Transaction source and conversion helpers.
  - **Key items**: `TransactionSource`
- `tx_forward.rs` - Raw transaction forwarder configuration.
  - **Key items**: `ForwardConfig`
- `utils.rs` - Misc utilities used across eth RPC helpers.
  - **Key items**: `checked_blob_gas_used_ratio()`, `calculate_gas_used_and_next_log_index()`

## Key APIs (no snippets)
- `EthConfig`, `EthStateCache`, `EthApiError`
- `FeeHistoryCache`, `GasPriceOracle`, `PendingBlock`
```

## File: crates/rpc/rpc-eth-types/AGENTS.md
```markdown
# rpc-eth-types

## Purpose
`reth-rpc-eth-types` crate: shared types, caches, and config for `eth` RPC implementation.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `eth` RPC types, cache, config, and errors.

### Files
- `Cargo.toml` - Manifest for eth RPC support types and features.
  - **Key items**: feature `js-tracer`, dependencies on `reth-evm`, `reth-rpc-convert`
```

## File: crates/rpc/rpc-layer/src/AGENTS.md
```markdown
# src

## Purpose
HTTP middleware layers for RPC auth, compression, and JWT validation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public exports for RPC layers.
- **Key items**: `AuthLayer`, `AuthClientLayer`, `CompressionLayer`, `JwtAuthValidator`, `AuthValidator`

### `auth_layer.rs`
- **Role**: Server-side authorization layer that validates JWT headers.
- **Key items**: `AuthLayer`, `AuthService`, `ResponseFuture`
- **Interactions**: Delegates validation to `AuthValidator`.

### `auth_client_layer.rs`
- **Role**: Client-side layer that injects JWT bearer tokens.
- **Key items**: `AuthClientLayer`, `AuthClientService`, `secret_to_bearer_header()`

### `compression_layer.rs`
- **Role**: Response compression layer for RPC HTTP responses.
- **Key items**: `CompressionLayer`, `CompressionService`

### `jwt_validator.rs`
- **Role**: JWT validation implementation for server auth.
- **Key items**: `JwtAuthValidator`, `validate()`, `get_bearer()`

## End-to-end flow (high level)
- Server wraps RPC HTTP with `AuthLayer` using `JwtAuthValidator`.
- Clients can use `AuthClientLayer` to add JWT headers.
- `CompressionLayer` compresses responses based on `Accept-Encoding`.

## Key APIs (no snippets)
- `AuthLayer`, `AuthClientLayer`, `JwtAuthValidator`, `CompressionLayer`
```

## File: crates/rpc/rpc-layer/AGENTS.md
```markdown
# rpc-layer

## Purpose
`reth-rpc-layer` crate: middleware layers for RPC auth and compression.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Auth, JWT, and compression layers.

### Files
- `Cargo.toml` - Manifest for RPC middleware layers.
  - **Key items**: `tower-http`, `jsonrpsee-http-client`
```

## File: crates/rpc/rpc-server-types/src/AGENTS.md
```markdown
# src

## Purpose
Shared RPC server constants, module selection, and error helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports.
- **Key items**: `RethRpcModule`, `RpcModuleSelection`, `ToRpcResult`

### `constants.rs`
- **Role**: Default constants for RPC servers and gas/trace limits.
- **Key items**: `DEFAULT_HTTP_RPC_PORT`, `DEFAULT_MAX_BLOCKS_PER_FILTER`, `DEFAULT_PROOF_PERMITS`, `gas_oracle::*`

### `module.rs`
- **Role**: RPC module enums and selection utilities.
- **Key items**: `RethRpcModule`, `RpcModuleSelection`, `RpcModuleValidator`
- **Knobs / invariants**: Supports `All`, `Standard`, or explicit selections.

### `result.rs`
- **Role**: RPC error conversion helpers and macros.
- **Key items**: `ToRpcResult`, `impl_to_rpc_result!`, `rpc_err()`, `invalid_params_rpc_err()`

## End-to-end flow (high level)
- Use constants to configure RPC servers and limits.
- Select modules via `RpcModuleSelection` and validators.
- Map internal errors to JSON-RPC error objects with `ToRpcResult`.

## Key APIs (no snippets)
- `RethRpcModule`, `RpcModuleSelection`, `ToRpcResult`
```

## File: crates/rpc/rpc-server-types/AGENTS.md
```markdown
# rpc-server-types

## Purpose
`reth-rpc-server-types` crate: shared constants, module selection, and error helpers for RPC servers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Constants, module selection, and RPC error helpers.

### Files
- `Cargo.toml` - Manifest for RPC server shared types.
  - **Key items**: `jsonrpsee-core`, `alloy-rpc-types-engine`
```

## File: crates/rpc/rpc-testing-util/src/AGENTS.md
```markdown
# src

## Purpose
Testing utilities and extension traits for RPC debug and trace clients.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring for RPC testing utilities.
- **Key items**: `debug`, `trace`, `utils`

### `debug.rs`
- **Role**: Debug API test helpers and stream utilities.
- **Key items**: `DebugApiExt`, `TraceTransactionResult`, `DebugTraceBlockResult`, `JsTracerBuilder`
- **Interactions**: Uses JS tracer templates from assets.

### `trace.rs`
- **Role**: Trace API stream helpers and result types.
- **Key items**: `TraceApiExt`, `TraceBlockResult`, `TraceCallStream`, `TraceFilterStream`

### `utils.rs`
- **Role**: Test utility helpers for RPC endpoint discovery.
- **Key items**: `parse_env_url()`

## End-to-end flow (high level)
- Build HTTP clients from env URLs.
- Use `DebugApiExt` and `TraceApiExt` to run RPC traces.
- Stream results and capture errors with typed wrappers.

## Key APIs (no snippets)
- `DebugApiExt`, `TraceApiExt`, `JsTracerBuilder`
```

## File: crates/rpc/rpc-testing-util/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for trace/debug RPC helper utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for trace tests.
- **Key items**: `trace`

### `trace.rs`
- **Role**: Local-node trace/debug integration tests using helpers.
- **Key items**: `trace_many_blocks()`, `trace_filters()`, `debug_trace_block_entire_chain()`
- **Knobs / invariants**: Tests are no-ops unless `RETH_RPC_TEST_NODE_URL` is set.

## End-to-end flow (high level)
- Build RPC client from environment variable.
- Run trace/debug streams against a live node.
- Print or inspect streamed results for errors.

## Key APIs (no snippets)
- `TraceApiExt`, `DebugApiExt`, `parse_env_url()`
```

## File: crates/rpc/rpc-testing-util/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for RPC testing utilities.

## Contents (one hop)
### Subdirectories
- [x] `it/` - Trace/debug helper integration tests.

### Files
- (none)
```

## File: crates/rpc/rpc-testing-util/AGENTS.md
```markdown
# rpc-testing-util

## Purpose
`reth-rpc-api-testing-util` crate: helpers for testing debug/trace RPC APIs.

## Contents (one hop)
### Subdirectories
- [x] `assets/` - (skip: JS tracer templates for debug/trace tests).
- [x] `src/` - RPC testing helpers and extension traits.
- [x] `tests/` - Integration tests for helper utilities.

### Files
- `Cargo.toml` - Manifest for RPC testing utilities.
  - **Key items**: `reth-rpc-api` (client), `jsonrpsee` client
```

## File: crates/rpc/AGENTS.md
```markdown
# rpc

## Purpose
RPC subsystem crates: interfaces, implementations, builders, and supporting types.

## Contents (one hop)
### Subdirectories
- [x] `ipc/` - IPC transport client/server for JSON-RPC.
- [x] `rpc/` - Concrete RPC handler implementations for all namespaces.
- [x] `rpc-api/` - JSON-RPC trait definitions for all namespaces.
- [x] `rpc-builder/` - RPC module assembly and server configuration helpers.
- [x] `rpc-convert/` - Conversion layer between primitives and RPC types.
- [x] `rpc-e2e-tests/` - End-to-end RPC compatibility tests (execution-apis).
- [x] `rpc-engine-api/` - Engine API implementation for consensus clients.
- [x] `rpc-eth-api/` - Shared `eth_` namespace traits and helpers.
- [x] `rpc-eth-types/` - `eth` RPC types, config, caches, and errors.
- [x] `rpc-layer/` - RPC middleware layers (auth, compression).
- [x] `rpc-server-types/` - Shared server constants and module selection.
- [x] `rpc-testing-util/` - Helpers for testing debug/trace RPC APIs.

### Files
- (none)
```

## File: crates/stages/api/src/metrics/AGENTS.md
```markdown
# metrics

## Purpose
Sync pipeline metric events and listeners that update per-stage gauges.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and exports for metric events and listener.
- **Key items**: `MetricEvent`, `MetricEventsSender`, `MetricsListener`

### `listener.rs`
- **Role**: Event listener that updates metrics on incoming `MetricEvent`s.
- **Key items**: `MetricEvent`, `MetricsListener`
- **Interactions**: Updates `SyncMetrics` and per-stage gauges.

### `sync_metrics.rs`
- **Role**: Metrics definitions for stage progress.
- **Key items**: `SyncMetrics`, `StageMetrics`
- **Knobs / invariants**: Metrics scoped under `sync`.

## End-to-end flow (high level)
- Stage/pipeline code sends `MetricEvent` over an unbounded channel.
- `MetricsListener` polls the channel and updates per-stage metrics.
- `SyncMetrics` lazily creates per-stage `StageMetrics` gauges.

## Key APIs (no snippets)
- `MetricEvent`, `MetricEventsSender`, `MetricsListener`
- `SyncMetrics`, `StageMetrics`
```

## File: crates/stages/api/src/pipeline/AGENTS.md
```markdown
# pipeline

## Purpose
Staged-sync pipeline orchestration: stage execution loop, control flow, events, and stage set builders.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Pipeline core implementation and run loop.
- **Key items**: `Pipeline`, `PipelineFut`, `PipelineWithResult`, `PipelineEvent`
- **Interactions**: Drives `Stage` execution and emits pipeline events/metrics.
- **Knobs / invariants**: `fail_on_unwind` stops on unexpected unwind; honors `max_block`.

### `builder.rs`
- **Role**: Builder for assembling pipeline stages and settings.
- **Key items**: `PipelineBuilder`, `add_stage()`, `add_stages()`, `with_max_block()`

### `ctrl.rs`
- **Role**: Control flow enum for pipeline outcomes.
- **Key items**: `ControlFlow`

### `event.rs`
- **Role**: Event types emitted during pipeline execution.
- **Key items**: `PipelineEvent`, `PipelineStagesProgress`

### `progress.rs`
- **Role**: Tracks minimum/maximum block progress across stages.
- **Key items**: `PipelineProgress`, `next_ctrl()`

### `set.rs`
- **Role**: Stage set abstraction and ordering helpers.
- **Key items**: `StageSet`, `StageSetBuilder`
- **Knobs / invariants**: Maintains stage ordering and enable/disable flags.

## End-to-end flow (high level)
- Build a `Pipeline` with `PipelineBuilder` and `StageSet` helpers.
- Run `Pipeline::run`/`run_as_fut` to execute stages sequentially.
- Each stage emits `PipelineEvent` and updates `PipelineProgress`.
- Control flow (`Continue`/`NoProgress`/`Unwind`) guides the loop behavior.

## Key APIs (no snippets)
- `Pipeline`, `PipelineBuilder`, `StageSetBuilder`
- `PipelineEvent`, `ControlFlow`, `PipelineProgress`
```

## File: crates/stages/api/src/AGENTS.md
```markdown
# src

## Purpose
Stage API definitions: stage trait and inputs/outputs, pipeline orchestration, metrics, and errors.

## Contents (one hop)
### Subdirectories
- [x] `metrics/` - Metric events and listener for stage progress gauges.
- [x] `pipeline/` - Pipeline orchestration, control flow, and stage set builders.

### Files
- `lib.rs` - Module wiring and public re-exports for stages and pipeline.
  - **Key items**: `Stage`, `Pipeline`, `StageId`, `StageCheckpoint`
- `error.rs` - Stage/pipeline error types and helpers.
  - **Key items**: `StageError`, `PipelineError`, `BlockErrorKind`
- `stage.rs` - Stage trait definitions and execution/unwind inputs.
  - **Key items**: `ExecInput`, `ExecOutput`, `UnwindInput`, `UnwindOutput`, `Stage`, `StageExt`
  - **Knobs / invariants**: Range helpers enforce transaction/blocks thresholds.
- `util.rs` - Small option helpers for min/max aggregation.
  - **Key items**: `opt::max()`, `opt::min()`
- `test_utils.rs` - Test stage implementation used by stage tests.
  - **Key items**: `TestStage`

## Key APIs (no snippets)
- `Stage`, `ExecInput`, `ExecOutput`, `UnwindInput`, `UnwindOutput`
- `StageError`, `PipelineError`, `BlockErrorKind`
```

## File: crates/stages/api/AGENTS.md
```markdown
# api

## Purpose
`reth-stages-api` crate: pipeline and stage interfaces, errors, and metrics for staged sync.

## Contents (one hop)
### Subdirectories
- [x] `docs/` - (skip: mermaid diagrams and docs assets).
- [x] `src/` - Stage traits, pipeline orchestration, metrics, and errors.

### Files
- `Cargo.toml` - Manifest for stages API and testing features.
  - **Key items**: feature `test-utils`
```

## File: crates/stages/stages/benches/setup/AGENTS.md
```markdown
# setup

## Purpose
Benchmark setup utilities for stages: test data generation and stage range helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Benchmark setup helpers for stage execution and unwind flows.
- **Key items**: `StageRange`, `stage_unwind()`, `unwind_hashes()`, `txs_testdata()`
- **Interactions**: Uses `AccountHashingStage`/`StorageHashingStage` for setup.

### `account_hashing.rs`
- **Role**: Prepares account-hashing benchmark data and stage ranges.
- **Key items**: `prepare_account_hashing()`, `generate_testdata_db()`
- **Knobs / invariants**: Optional external DB via `ACCOUNT_HASHING_DB` env var.

### `constants.rs`
- **Role**: Benchmark environment constants.
- **Key items**: `ACCOUNT_HASHING_DB`

## End-to-end flow (high level)
- Detect whether an external DB is provided via `ACCOUNT_HASHING_DB`.
- If missing, generate local test data and stage range.
- Provide helpers to reset/unwind stage state before benchmarking.

## Key APIs (no snippets)
- `prepare_account_hashing()`, `txs_testdata()`, `stage_unwind()`
```

## File: crates/stages/stages/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for core sync stages and their performance profiles.

## Contents (one hop)
### Subdirectories
- [x] `setup/` - Benchmark data generation and stage range helpers.

### Files
- `README.md` - Benchmark usage and external DB instructions.
  - **Key items**: `ACCOUNT_HASHING_DB`, `cargo bench --package reth-stages --bench criterion`
- `criterion.rs` - Criterion benchmark runner for multiple stages.
  - **Key items**: `transaction_lookup()`, `account_hashing()`, `senders()`, `merkle()`
  - **Interactions**: Uses `setup` helpers to build test DBs and unwind stages.

## Key APIs (no snippets)
- `run_benches()`, `measure_stage()`
```

## File: crates/stages/stages/src/stages/AGENTS.md
```markdown
# stages

## Purpose
Concrete stage implementations used by the sync pipeline: download, execution, hashing, indexing, pruning, and finish steps.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `headers.rs`
- **Role**: Downloads headers (reverse) and writes them into static files and header indices.
- **Key items**: `HeaderStage`, `write_headers()`, `HeaderSyncGap`
- **Interactions**: Uses `HeaderDownloader` and ETL collectors for hash/headers.
- **Knobs / invariants**: Uses ETL file size; checkpoint updates after gap filled.

### `bodies.rs`
- **Role**: Downloads block bodies and writes ommers, bodies, transactions, and tx blocks.
- **Key items**: `BodyStage`, `ensure_consistency()`
- **Interactions**: Depends on headers; writes to DB/static files via `BlockWriter`.
- **Knobs / invariants**: Requires downloader buffer; handles empty-block shortcut rules.

### `sender_recovery.rs`
- **Role**: Recovers transaction senders and persists them to DB or static files.
- **Key items**: `SenderRecoveryStage`, `BATCH_SIZE`, `WORKER_CHUNK_SIZE`
- **Interactions**: Reads transactions and block body indices; writes senders via `EitherWriter`.
- **Knobs / invariants**: `commit_threshold` controls batch size and checkpoint cadence.

### `tx_lookup.rs`
- **Role**: Builds transaction hash -> transaction number index.
- **Key items**: `TransactionLookupStage`
- **Interactions**: Uses ETL collector; supports RocksDB batch when enabled.
- **Knobs / invariants**: `chunk_size` controls ETL batching; respects prune mode.

### `execution.rs`
- **Role**: Executes blocks, writes state changes, receipts, and changesets.
- **Key items**: `ExecutionStage`, `calculate_gas_used_from_headers()`
- **Interactions**: Uses `ConfigureEvm`, `FullConsensus`, and `StateProviderDatabase`.
- **Knobs / invariants**: `ExecutionStageThresholds` and `external_clean_threshold` tune batching.

### `hashing_account.rs`
- **Role**: Hashes plain account state into `HashedAccounts`.
- **Key items**: `AccountHashingStage`, `SeedOpts`
- **Interactions**: Reads `PlainAccountState` and change sets; uses ETL collector.
- **Knobs / invariants**: `clean_threshold` decides full vs incremental hashing.

### `hashing_storage.rs`
- **Role**: Hashes plain storage state into `HashedStorages`.
- **Key items**: `StorageHashingStage`, `HASHED_ZERO_ADDRESS`
- **Interactions**: Reads `PlainStorageState` and storage changesets.
- **Knobs / invariants**: `clean_threshold` decides full vs incremental hashing.

### `merkle.rs`
- **Role**: Computes state roots and intermediate trie hashes from hashed state.
- **Key items**: `MerkleStage`, `MERKLE_STAGE_DEFAULT_REBUILD_THRESHOLD`, `MERKLE_STAGE_DEFAULT_INCREMENTAL_THRESHOLD`
- **Interactions**: Uses `MerkleCheckpoint` and trie DB updates.
- **Knobs / invariants**: Execution vs unwind modes must be ordered correctly.

### `merkle_changesets.rs`
- **Role**: Maintains trie changesets from finalized block to latest block.
- **Key items**: `MerkleChangeSets`
- **Interactions**: Computes `TrieUpdates` and validates state roots.
- **Knobs / invariants**: Retains changesets based on finalized block or retention window.

### `index_account_history.rs`
- **Role**: Builds `AccountsHistory` shard indices from account change sets.
- **Key items**: `IndexAccountHistoryStage`
- **Interactions**: Uses ETL and `collect_account_history_indices()`.
- **Knobs / invariants**: `commit_threshold` and `prune_mode` control range and pruning.

### `index_storage_history.rs`
- **Role**: Builds `StoragesHistory` shard indices from storage change sets.
- **Key items**: `IndexStorageHistoryStage`
- **Interactions**: Uses `collect_history_indices()` and storage sharded keys.
- **Knobs / invariants**: `commit_threshold` and `prune_mode` control range and pruning.

### `prune.rs`
- **Role**: Runs pruning segments for configured prune modes.
- **Key items**: `PruneStage`, `PruneSenderRecoveryStage`
- **Interactions**: Uses `PrunerBuilder` and prune checkpoints.
- **Knobs / invariants**: `commit_threshold` caps deletions before commit.

### `finish.rs`
- **Role**: Marks the pipeline as fully synced at the target block.
- **Key items**: `FinishStage`

### `era.rs`
- **Role**: Imports pre-merge history from ERA1 files or URLs into storage.
- **Key items**: `EraStage`, `EraImportSource`
- **Interactions**: Streams `Era1Reader` items and builds header indices.
- **Knobs / invariants**: Optional import source; uses ETL for hash indexing.

### `utils.rs`
- **Role**: Shared helpers for history index collection/loading.
- **Key items**: `collect_history_indices()`, `collect_account_history_indices()`, `load_history_indices()`
- **Knobs / invariants**: `DEFAULT_CACHE_THRESHOLD` controls cache flush cadence.

## End-to-end flow (high level)
- Optionally import pre-merge headers/bodies with `EraStage`.
- Download headers (`HeaderStage`) and bodies (`BodyStage`) from the network.
- Recover senders (`SenderRecoveryStage`) and execute blocks (`ExecutionStage`).
- Hash accounts and storage, then compute state roots (`MerkleStage`).
- Build transaction lookup and history indices (`TransactionLookupStage`, `Index*HistoryStage`).
- Prune configured segments (`PruneStage`/`PruneSenderRecoveryStage`).
- Mark completion with `FinishStage`.

## Key APIs (no snippets)
- `HeaderStage`, `BodyStage`, `ExecutionStage`, `MerkleStage`
- `SenderRecoveryStage`, `TransactionLookupStage`, `PruneStage`
- `AccountHashingStage`, `StorageHashingStage`, `IndexAccountHistoryStage`
```

## File: crates/stages/stages/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Test helpers for stage implementations: test DB harness, stage runners, and macros.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and exports for stage test helpers.
- **Key items**: `TestStageDB`, `TestStages`, `StageTestRunner`, `TEST_STAGE_ID`

### `macros.rs`
- **Role**: Shared test suite macros for stage execute/unwind flows.
- **Key items**: `stage_test_suite!`, `stage_test_suite_ext!`

### `runner.rs`
- **Role**: Generic test runner traits for stage execution and unwind checks.
- **Key items**: `StageTestRunner`, `ExecuteStageTestRunner`, `UnwindStageTestRunner`, `TestRunnerError`
- **Interactions**: Spawns async tasks that call `Stage::execute`/`Stage::unwind`.

### `set.rs`
- **Role**: StageSet implementation backed by `TestStage` outputs.
- **Key items**: `TestStages`

### `test_db.rs`
- **Role**: Test database harness with helpers for inserting headers/blocks and querying tables.
- **Key items**: `TestStageDB`, `StorageKind`, `insert_headers()`, `insert_blocks()`
- **Knobs / invariants**: Uses temp MDBX/static files and optional RocksDB for tests.

## End-to-end flow (high level)
- Build a `TestStageDB` with temp static files and DB env.
- Implement `StageTestRunner` to seed data and validate results.
- Use macros to exercise execute/unwind flows consistently across stages.

## Key APIs (no snippets)
- `TestStageDB`, `StageTestRunner`, `ExecuteStageTestRunner`
- `stage_test_suite!`, `TestStages`
```

## File: crates/stages/stages/src/AGENTS.md
```markdown
# src

## Purpose
Stage implementations and built-in stage sets used by the sync pipeline.

## Contents (one hop)
### Subdirectories
- [x] `stages/` - Concrete stage implementations (download, execute, hash, prune, finish).
- [x] `test_utils/` - Stage test harness, runners, and macros.

### Files
- `lib.rs` - Crate entrypoint and re-exports for pipeline and stages.
  - **Key items**: `Pipeline`, `Stage`, `StageSet`, `stages`, `sets`
- `prelude.rs` - Re-exports of common stage set types.
  - **Key items**: `DefaultStages`, `OnlineStages`, `OfflineStages`
- `sets.rs` - Built-in stage sets and wiring for default sync order.
  - **Key items**: `DefaultStages`, `OnlineStages`, `OfflineStages`, `HashingStages`
  - **Interactions**: Constructs `StageSetBuilder` with downloader, consensus, and config inputs.

## Key APIs (no snippets)
- `DefaultStages`, `OnlineStages`, `OfflineStages`
- `Pipeline`, `StageSet`
```

## File: crates/stages/stages/AGENTS.md
```markdown
# stages

## Purpose
`reth-stages` crate: concrete stage implementations and built-in stage sets for sync pipelines.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Criterion benchmarks and setup helpers for stages.
- [x] `src/` - Stage implementations, stage sets, and test utilities.

### Files
- `Cargo.toml` - Manifest for stage implementations and optional test utilities.
  - **Key items**: features `test-utils`, `rocksdb`
```

## File: crates/stages/types/src/AGENTS.md
```markdown
# src

## Purpose
Common stage and pipeline types: stage identifiers, checkpoints, and execution thresholds.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports for stage IDs, checkpoints, and execution controls.
- **Key items**: `StageId`, `StageCheckpoint`, `ExecutionStageThresholds`, `PipelineTarget`
- **Interactions**: Re-exports checkpoint structs from `checkpoints.rs`.

### `id.rs`
- **Role**: Stage ID registry and helpers.
- **Key items**: `StageId`, `StageId::ALL`, `StageId::STATE_REQUIRED`, `get_pre_encoded()`
- **Knobs / invariants**: Stage IDs are stable and used as DB keys.

### `checkpoints.rs`
- **Role**: Checkpoint structures for stages and merkle progress.
- **Key items**: `StageCheckpoint`, `EntitiesCheckpoint`, `MerkleCheckpoint`, `StorageRootMerkleCheckpoint`
- **Knobs / invariants**: Compact encoding for checkpoints; optional inner progress for merkle.

### `execution.rs`
- **Role**: Execution batching thresholds for the execution stage.
- **Key items**: `ExecutionStageThresholds`, `is_end_of_batch()`
- **Knobs / invariants**: Defaults enforce max blocks, changes, gas, and duration.

## End-to-end flow (high level)
- Pipeline code uses `StageId` to identify stages and persist checkpoints.
- Stages write `StageCheckpoint` with optional entity progress.
- Merkle stages persist `MerkleCheckpoint`/`StorageRootMerkleCheckpoint` for resumable hashing.
- Execution uses `ExecutionStageThresholds` to decide when to commit.

## Key APIs (no snippets)
- `StageId`, `StageCheckpoint`, `EntitiesCheckpoint`
- `MerkleCheckpoint`, `StorageRootMerkleCheckpoint`
- `ExecutionStageThresholds`, `PipelineTarget`
```

## File: crates/stages/types/AGENTS.md
```markdown
# types

## Purpose
`reth-stages-types` crate: shared stage IDs, checkpoints, and execution threshold types.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Stage identifiers, checkpoints, and execution batching thresholds.

### Files
- `Cargo.toml` - Manifest for stage-type dependencies and feature flags.
  - **Key items**: features `reth-codec`, `serde`, `arbitrary`, `test-utils`
```

## File: crates/stages/AGENTS.md
```markdown
# stages

## Purpose
Staged sync subsystem crates: APIs, shared types, and concrete stage implementations.

## Contents (one hop)
### Subdirectories
- [x] `api/` - Stage/pipeline interfaces, errors, metrics, and orchestration.
- [x] `stages/` - Concrete stage implementations and built-in stage sets.
- [x] `types/` - Shared stage IDs, checkpoints, and execution thresholds.

### Files
- (none)
```

## File: crates/stateless/src/AGENTS.md
```markdown
# src

## Purpose
Stateless execution and validation helpers that verify blocks using witness data instead of a full database.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint, public exports, and `StatelessInput` serialization helper.
- **Key items**: `StatelessInput`, `ExecutionWitness`, `stateless_validation()`, `stateless_validation_with_trie()`
- **Interactions**: Re-exports `StatelessTrie` and `UncompressedPublicKey`.

### `recover_block.rs`
- **Role**: Signature verification and sender recovery using provided public keys.
- **Key items**: `UncompressedPublicKey`, `recover_block_with_public_keys()`
- **Knobs / invariants**: Enforces Homestead signature normalization; requires matching tx/key counts.

### `trie.rs`
- **Role**: Stateless trie abstraction and sparse trie implementation backed by witness data.
- **Key items**: `StatelessTrie`, `StatelessSparseTrie`, `verify_execution_witness()`, `calculate_state_root()`
- **Interactions**: Uses `SparseStateTrie` and validates witness completeness for account/storage lookups.
- **Knobs / invariants**: Errors on incomplete witnesses; pre-state root must match.

### `validation.rs`
- **Role**: Stateless validation pipeline orchestration and error types.
- **Key items**: `StatelessValidationError`, `stateless_validation()`, `stateless_validation_with_trie()`
- **Interactions**: Executes blocks via `reth-evm`, validates consensus, and checks state roots.
- **Knobs / invariants**: Enforces `BLOCKHASH` ancestor limit (256) and contiguous ancestor headers.

### `witness_db.rs`
- **Role**: `revm::Database` implementation backed by stateless trie + witness bytecode/hashes.
- **Key items**: `WitnessDatabase`
- **Knobs / invariants**: Missing bytecode/hash entries produce errors; assumes contiguous ancestor hashes.

## End-to-end flow (high level)
- Deserialize `StatelessInput` containing block, witness, and chain config.
- Recover senders using `recover_block_with_public_keys`.
- Build a `StatelessTrie` from witness data and verify pre-state root.
- Initialize `WitnessDatabase` with trie, bytecode map, and ancestor hashes.
- Execute the block with `reth-evm` and validate consensus/post-exec checks.
- Recompute the post-state root and compare to the block header.

## Key APIs (no snippets)
- `stateless_validation()`, `stateless_validation_with_trie()`
- `StatelessTrie`, `StatelessSparseTrie`, `WitnessDatabase`
- `StatelessValidationError`, `UncompressedPublicKey`
```

## File: crates/stateless/AGENTS.md
```markdown
# stateless

## Purpose
`reth-stateless` crate: stateless block execution and validation using witness data.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Stateless validation pipeline, trie utilities, and witness-backed DB.

### Files
- `Cargo.toml` - Manifest for stateless validation dependencies and crypto backends.
  - **Key items**: features `k256`, `secp256k1`
```

## File: crates/static-file/static-file/src/segments/AGENTS.md
```markdown
# segments

## Purpose
Static file segment implementations for copying data from the database.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Segment trait definition and module wiring.
- **Key items**: `Segment`, `StaticFileSegment`

### `receipts.rs`
- **Role**: Segment that copies receipts into static files.
- **Key items**: `Receipts`
- **Interactions**: Uses `StaticFileProviderFactory` and receipt cursors to append data.

## Key APIs (no snippets)
- `Segment`, `Receipts`
```

## File: crates/static-file/static-file/src/AGENTS.md
```markdown
# src

## Purpose
Static file producer implementation for moving finalized data into static files.

## Contents (one hop)
### Subdirectories
- [x] `segments/` - Segment implementations for static file copying.

### Files
- `lib.rs` - Crate wiring and re-exports for static file producer types.
  - **Key items**: `StaticFileProducer`, `StaticFileProducerInner`
- `static_file_producer.rs` - Producer logic for selecting targets and copying data.
  - **Key items**: `StaticFileProducer`, `StaticFileProducerInner`, `StaticFileProducerResult`
  - **Interactions**: Emits `StaticFileProducerEvent` and updates static file indexes.

## Key APIs (no snippets)
- `StaticFileProducer`, `StaticFileProducerInner`
```

## File: crates/static-file/static-file/AGENTS.md
```markdown
# static-file

## Purpose
`reth-static-file` crate: static file producer implementation for finalized data.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Producer logic and segment implementations.

### Files
- `Cargo.toml` - Manifest for static file producer dependencies.
  - **Key items**: deps `reth-static-file-types`, `reth-provider`, `reth-prune-types`
- `README.md` - Static file overview and architecture diagrams.
```

## File: crates/static-file/types/src/AGENTS.md
```markdown
# src

## Purpose
Static file type definitions: segments, headers, compression, and producer events.

## Contents (one hop)
### Subdirectories
- [x] `snapshots/` - (skip: test snapshot fixtures).

### Files
- `lib.rs` - Crate entrypoint and shared static-file types.
  - **Key items**: `StaticFileSegment`, `SegmentHeader`, `StaticFileTargets`, `HighestStaticFiles`, `DEFAULT_BLOCKS_PER_STATIC_FILE`
- `segment.rs` - Static file segment enum and segment header/offset types.
  - **Key items**: `StaticFileSegment`, `SegmentHeader`, `SegmentRangeInclusive`, `SegmentConfig`
  - **Knobs / invariants**: Segment order is significant; filename format must stay in sync.
- `event.rs` - Producer lifecycle events.
  - **Key items**: `StaticFileProducerEvent`
- `compression.rs` - Compression type enum for static files.
  - **Key items**: `Compression`

## Key APIs (no snippets)
- `StaticFileSegment`, `SegmentHeader`, `SegmentRangeInclusive`
- `StaticFileTargets`, `StaticFileProducerEvent`
```

## File: crates/static-file/types/AGENTS.md
```markdown
# types

## Purpose
`reth-static-file-types` crate: shared types for static file segments and producer events.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Segment definitions, headers, compression, and events.

### Files
- `Cargo.toml` - Manifest for static file type dependencies and features.
  - **Key items**: feature `clap`, `std`
```

## File: crates/static-file/AGENTS.md
```markdown
# static-file

## Purpose
Static file subsystem: producer implementation and shared static-file types.

## Contents (one hop)
### Subdirectories
- [x] `static-file/` - Static file producer and segment copying logic.
- [x] `types/` - Segment and event type definitions for static files.

### Files
- (none)
```

## File: crates/storage/codecs/derive/src/compact/AGENTS.md
```markdown
# compact

## Purpose
Code generation helpers for the `Compact` derive macro, including field analysis and bitflag
struct generation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Entry point for `Compact` derive codegen, wiring helpers and field analysis.
- **Key items**: `derive()`, `StructFieldDescriptor`, `FieldTypes`, `get_fields()`,
  `get_bit_size()`
- **Interactions**: Delegates to `flags`, `structs`, `enums`, and `generator`.

### `enums.rs`
- **Role**: Codegen helpers for enum Compact implementations.
- **Key items**: enum variant handling, `generate_from_to` helpers (enum paths)
- **Interactions**: Used by `mod.rs` during derive expansion for enums.

### `flags.rs`
- **Role**: Builds bitflag structs and metadata for compacted fields.
- **Key items**: `generate_flag_struct()`, `bitflag_encoded_bytes()`, `bitflag_unused_bits()`
- **Interactions**: Uses `modular_bitfield` and `get_bit_size()` for flag sizing.
- **Knobs / invariants**: Pads flag structs to full bytes and tracks unused bits for extensibility.

### `generator.rs`
- **Role**: Emits `to_compact`/`from_compact` implementations and optional zstd wrapping.
- **Key items**: `generate_from_to()`, zstd handling, field encode/decode paths
- **Interactions**: Consumes field descriptors and flags generated in `mod.rs`.

### `structs.rs`
- **Role**: Codegen helpers for struct Compact implementations.
- **Key items**: struct field ordering, special-casing `Option`/`Vec` fields, flag extraction
- **Interactions**: Used by `generator.rs` when emitting struct encode/decode logic.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StructFieldDescriptor`, `FieldTypes`
- **Modules / Packages**: `flags`, `generator`, `structs`, `enums`
- **Functions**: `derive()`, `get_fields()`, `get_bit_size()`, `generate_flag_struct()`

## Relationships
- **Depends on**: `syn`/`quote` for AST parsing and code generation.
- **Data/control flow**: field descriptors feed flag generation, then encoder/decoder codegen.

## End-to-end flow (high level)
- Parse derive input into field descriptors.
- Generate flag structs and metadata for compacted fields.
- Emit `to_compact` and `from_compact` implementations (optionally with zstd wrapping).
```

## File: crates/storage/codecs/derive/src/AGENTS.md
```markdown
# src

## Purpose
Proc-macro implementation for deriving `Compact` codecs and generating arbitrary/roundtrip tests.

## Contents (one hop)
### Subdirectories
- [x] `compact/` - `Compact` derive codegen helpers and flag struct generation.

### Files
- `arbitrary.rs` - Generates optional proptest roundtrip tests for `Compact`/RLP encodings.
  - **Key items**: `maybe_generate_tests()`, proptest config handling, `add_arbitrary_tests`
- `lib.rs` - Proc-macro entrypoints for `Compact`, `CompactZstd`, and test generation helpers.
  - **Key items**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ZstdConfig`
- **Modules / Packages**: `compact`, `arbitrary`
- **Functions**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Relationships
- **Depends on**: `syn` and `quote` for macro parsing and code generation.
- **Data/control flow**: `Compact` derive dispatches into `compact` helpers; test generation uses
  `arbitrary` to emit proptest modules.
```

## File: crates/storage/codecs/derive/AGENTS.md
```markdown
# derive

## Purpose
`reth-codecs-derive` proc-macro crate for deriving `Compact` codecs and optional zstd/test helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Proc-macro entrypoints and codegen helpers.

### Files
- `Cargo.toml` - Proc-macro crate manifest.
  - **Key items**: `proc-macro = true`, deps `syn`, `quote`, `proc-macro2`

## Key APIs (no snippets)
- **Modules / Packages**: `compact`, `arbitrary`
- **Functions**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Relationships
- **Depends on**: `syn` and `quote` for parsing and code generation.
```

## File: crates/storage/codecs/src/alloy/transaction/AGENTS.md
```markdown
# transaction

## Purpose
Compact codec implementations for Ethereum and Optimism transaction types and envelopes.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Implements `Compact` for `EthereumTypedTransaction` and wires transaction modules.
- **Key items**: `EthereumTypedTransaction`, `CompactEnvelope`, `Envelope`, `FromTxCompact`,
  `ToTxCompact`
- **Interactions**: Delegates to per-tx-type modules and `ethereum.rs` helpers.

### `ethereum.rs`
- **Role**: Compact serialization for transaction envelopes with signature handling and optional
  zstd compression.
- **Key items**: `ToTxCompact`, `FromTxCompact`, `Envelope`, `CompactEnvelope`
- **Interactions**: Uses `TxType` for type discrimination and `Signature` for rehydration.
- **Knobs / invariants**: Flag byte packs signature/type/compression bits; compresses inputs >= 32B.

### `eip1559.rs`
- **Role**: Compact bridge struct for `TxEip1559`.
- **Key items**: `TxEip1559` fields (fees, access list, input), `Compact` impl

### `eip2930.rs`
- **Role**: Compact bridge struct for `TxEip2930`.
- **Key items**: `TxEip2930`, `AccessList`, `Compact` impl

### `eip4844.rs`
- **Role**: Compact bridge struct for `TxEip4844` blob transactions.
- **Key items**: `TxEip4844`, `blob_versioned_hashes`, `max_fee_per_blob_gas`, placeholder bit
- **Knobs / invariants**: Placeholder field preserves bitflag layout for compatibility.

### `eip7702.rs`
- **Role**: Compact bridge struct for `TxEip7702` set-code transactions.
- **Key items**: `TxEip7702`, `authorization_list`, `Compact` impl

### `legacy.rs`
- **Role**: Compact bridge struct for legacy transactions.
- **Key items**: `TxLegacy`, `gas_price`, `chain_id`, `Compact` impl

### `txtype.rs`
- **Role**: Compact mapping for `TxType` identifiers with extended type support.
- **Key items**: `TxType::to_compact()`, `TxType::from_compact()`,
  `COMPACT_EXTENDED_IDENTIFIER_FLAG`

### `optimism.rs`
- **Role**: Compact support for Optimism deposit transactions and typed envelopes.
- **Key items**: `TxDeposit`, `OpTxType`, `OpTypedTransaction`, `CompactEnvelope` impls
- **Interactions**: Extends tx-type encoding to include deposit identifiers.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `CompactEnvelope`, `Envelope`, `TxEip1559`, `TxEip2930`,
  `TxEip4844`, `TxEip7702`, `TxLegacy`, `TxDeposit`
- **Modules / Packages**: `ethereum`, `txtype`
- **Functions**: `to_compact()`, `from_compact()`, `to_tx_compact()`, `from_tx_compact()`

## Relationships
- **Depends on**: `alloy-consensus` transaction types and `alloy-eips` access lists.
- **Depends on**: `reth-zstd-compressors` for optional envelope compression.
- **Data/control flow**: tx type identifiers drive dispatch into per-type compact codecs, with
  envelope helpers adding signature and compression metadata.

## End-to-end flow (high level)
- Encode tx type identifier and signature metadata.
- Serialize the concrete transaction payload with Compact.
- Optionally compress payload for large inputs.
- Decode by reading flags, inflating payload, and constructing typed transactions.
```

## File: crates/storage/codecs/src/alloy/AGENTS.md
```markdown
# alloy

## Purpose
Compact codec implementations for alloy consensus/primitives types used in storage encoding.

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - Compact codecs for Ethereum and Optimism transaction types and envelopes.

### Files
- `access_list.rs` - Compact encoding for EIP-2930 access lists.
  - **Key items**: `AccessList`, `AccessListItem`, `Compact` impls
- `authorization_list.rs` - Compact encoding for EIP-7702 authorizations.
  - **Key items**: `Authorization`, `SignedAuthorization`, `Compact` impls
- `genesis_account.rs` - Compact bridges for genesis accounts and storage entries.
  - **Key items**: `GenesisAccount`, `GenesisAccountRef`, `StorageEntries`, `StorageEntry`
- `header.rs` - Compact bridge for block headers with extension struct.
  - **Key items**: `Header`, `HeaderExt`, `Compact` impl for `alloy_consensus::Header`
- `log.rs` - Compact codecs for log and log data types.
  - **Key items**: `Log`, `LogData`, `Compact` impls
- `optimism.rs` - Compact encoding for Optimism receipts using zstd helpers.
  - **Key items**: `CompactOpReceipt`, `OpReceipt`, `CompactZstd`
- `signature.rs` - Compact encoding for ECDSA signatures.
  - **Key items**: `Signature`, `Compact` impl
- `trie.rs` - Compact encoding for trie builder values and branch nodes.
  - **Key items**: `HashBuilderValue`, `BranchNodeCompact`, `TrieMask`
- `txkind.rs` - Compact encoding for transaction kind (create/call).
  - **Key items**: `TxKind`, `TX_KIND_TYPE_CREATE`, `TX_KIND_TYPE_CALL`
- `withdrawal.rs` - Compact encoding for withdrawals and withdrawal lists.
  - **Key items**: `Withdrawal`, `Withdrawals`, `Compact` impls
- `mod.rs` - Module wiring and conditional exports for alloy codecs.
  - **Key items**: `cond_mod!`, `transaction`, `optimism`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Header`, `GenesisAccount`, `Withdrawal`, `CompactOpReceipt`
- **Modules / Packages**: `transaction`, `optimism`
- **Functions**: `to_compact()`, `from_compact()`

## Relationships
- **Depends on**: `alloy-consensus`, `alloy-eips`, `alloy-genesis`, and `alloy-trie`.
- **Data/control flow**: Compact implementations bridge alloy types into storage-optimized
  encodings, reused by database codecs and transaction envelopes.
```

## File: crates/storage/codecs/src/AGENTS.md
```markdown
# src

## Purpose
Core Compact codec trait and utilities plus alloy-specific codec implementations.

## Contents (one hop)
### Subdirectories
- [x] `alloy/` - Compact codecs for alloy consensus/primitives types and transactions.

### Files
- `lib.rs` - Defines the `Compact` trait and core encoding/decoding helpers.
  - **Key items**: `Compact`, `CompactPlaceholder`, `encode_varuint()`, `decode_varuint()`,
    `specialized_to_compact()`
- `private.rs` - Internal re-exports for derive macro expansion.
  - **Key items**: `modular_bitfield`, `bytes::Buf`
- `test_utils.rs` - Test helpers and macros for backwards compatibility checks.
  - **Key items**: `validate_bitflag_backwards_compat!`, `UnusedBits`, `test_decode()`
- `txtype.rs` - Transaction type identifier constants for compact encoding.
  - **Key items**: `COMPACT_IDENTIFIER_LEGACY`, `COMPACT_IDENTIFIER_EIP2930`,
    `COMPACT_EXTENDED_IDENTIFIER_FLAG`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Compact`
- **Modules / Packages**: `alloy`, `txtype`
- **Functions**: `to_compact()`, `from_compact()`, `decode_varuint()`

## Relationships
- **Depends on**: `reth-codecs-derive` for generated `Compact` implementations.
- **Data/control flow**: `Compact` trait is implemented for core primitives and alloy types,
  enabling storage serialization across crates.
```

## File: crates/storage/codecs/testdata/AGENTS.md
```markdown
# testdata

## Purpose
JSON fixtures for compact codec tests.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `access_list_compact.json`
- **Role**: Fixture data for access list compact encoding tests.

### `log_compact.json`
- **Role**: Fixture data for log compact encoding tests.
```

## File: crates/storage/codecs/AGENTS.md
```markdown
# codecs

## Purpose
`reth-codecs` crate: Compact codec trait, derive macros, and alloy type implementations for
storage serialization.

## Contents (one hop)
### Subdirectories
- [x] `derive/` - Proc-macro derive helpers for Compact and zstd wrappers.
- [x] `src/` - Compact trait, core utilities, and alloy codecs.
- [x] `testdata/` - (skip: JSON fixtures for compact codec tests)

### Files
- `Cargo.toml` - Crate manifest for Compact codecs and feature flags.
  - **Key items**: features `alloy`, `op`, `serde`, `test-utils`; deps `reth-codecs-derive`,
    optional `reth-zstd-compressors`
- `README.md` - Overview of Compact codec design and derive macro usage.
  - **Key items**: `Compact`, `#[reth_codec]`, `#[add_arbitrary_tests]`, backwards compatibility

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Compact`, `CompactPlaceholder`
- **Modules / Packages**: `derive`, `alloy`
- **Functions**: `to_compact()`, `from_compact()`, `decode_varuint()`

## Relationships
- **Depends on**: `reth-codecs-derive` for proc-macro codegen.
- **Depends on**: alloy types for consensus, transactions, and trie codecs.
```

## File: crates/storage/db/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for MDBX table operations and serialization paths.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `criterion.rs`
- **Role**: Main benchmark harness for table operations and serialization.
- **Key items**: `db()`, `serialization()`, `measure_table_db()`, `measure_table_serialization()`
- **Interactions**: Uses `utils.rs` helpers to load test vectors and open temp DBs.

### `get.rs`
- **Role**: Benchmarks point lookup performance for MDBX tables.
- **Key items**: `bench_get`, `DbCursorRO`, table-specific loops

### `hash_keys.rs`
- **Role**: Benchmarks hashed key generation for table workloads.
- **Key items**: hashing helpers, dataset generation

### `put.rs`
- **Role**: Benchmarks insert/write performance for MDBX tables.
- **Key items**: `DbTxMut`, `cursor_write`, insert loops

### `utils.rs`
- **Role**: Benchmark data generation and table vector loading.
- **Key items**: `load_vectors()`, test data builders, `BENCH_DB_PATH`

### `README.md`
- **Role**: Notes on running DB benchmarks and codec benchmarks.
- **Key items**: `cargo bench --features bench`

## End-to-end flow (high level)
- Generate test vectors and open a temp database.
- Run serialization and CRUD benchmarks across core tables.
- Record timing per table and operation.
```

## File: crates/storage/db/src/implementation/mdbx/AGENTS.md
```markdown
# mdbx

## Purpose
MDBX-backed implementations of database cursors and transactions for `reth-db`.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: MDBX environment wrapper and configuration for database open/init.
- **Key items**: `DatabaseEnvKind`, `DatabaseArguments`, size constants (`KILOBYTE`, `GIGABYTE`)
- **Interactions**: Wires `cursor` and `tx` modules and exports MDBX env helpers.
- **Knobs / invariants**: `DEFAULT_MAX_READERS`, sync mode, geometry sizing.

### `cursor.rs`
- **Role**: Cursor wrappers implementing `DbCursorRO/RW` and dup-sort cursors.
- **Key items**: `Cursor`, `CursorRO`, `CursorRW`, `decode()`, `compress_to_buf_or_ref!`
- **Interactions**: Uses table `Encode/Decode` and metrics hooks for operations.

### `tx.rs`
- **Role**: Transaction wrapper for MDBX read/write transactions.
- **Key items**: `Tx`, `get_dbi()`, `new_cursor()`, `commit()`, metrics handler types
- **Interactions**: Produces MDBX cursors and records transaction metrics.
- **Knobs / invariants**: Long transaction logging threshold and commit latency recording.

### `utils.rs`
- **Role**: Decode helpers for key/value pairs and values.
- **Key items**: `decoder()`, `decode_value()`, `decode_one()`
- **Interactions**: Shared by cursor implementations for decoding.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseArguments`, `DatabaseEnvKind`, `Tx`, `Cursor`
- **Modules / Packages**: `cursor`, `tx`
- **Functions**: `decoder()`, `get_dbi()`, `new_cursor()`

## Relationships
- **Depends on**: `reth_libmdbx` for MDBX bindings and environment types.
- **Depends on**: `reth-db-api` for cursor/transaction traits and table codecs.
- **Data/control flow**: MDBX cursors decode table entries and surface them via `DbCursor` traits.

## End-to-end flow (high level)
- Open MDBX environment and configure geometry/sync options.
- Create transactions and cursors for tables.
- Encode/decode entries and record metrics for operations.
```

## File: crates/storage/db/src/implementation/AGENTS.md
```markdown
# implementation

## Purpose
Backend-specific database implementations (currently MDBX).

## Contents (one hop)
### Subdirectories
- [x] `mdbx/` - MDBX-backed cursor and transaction implementations.

### Files
- `mod.rs` - Feature-gated module selection for DB backends.
  - **Key items**: `mdbx` (feature gated)

## Key APIs (no snippets)
- **Modules / Packages**: `mdbx`

## Relationships
- **Depends on**: `reth-libmdbx` when the `mdbx` feature is enabled.
```

## File: crates/storage/db/src/static_file/AGENTS.md
```markdown
# static_file

## Purpose
Static file segment access helpers built on top of nippy-jar archives.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Static file utilities and directory scanning for segment jars.
- **Key items**: `iter_static_files()`, `StaticFileCursor`, `StaticFileMask`
- **Interactions**: Uses `NippyJar` and `StaticFileSegment` headers to enumerate ranges.

### `cursor.rs`
- **Role**: Cursor wrapper for reading static file rows and column subsets.
- **Key items**: `StaticFileCursor`, `KeyOrNumber`, `get_one()`, `get_two()`, `get_three()`
- **Interactions**: Uses column selector traits from `mask.rs` and `NippyJarCursor`.

### `mask.rs`
- **Role**: Column selector traits and macro for static file masks.
- **Key items**: `ColumnSelectorOne`, `ColumnSelectorTwo`, `ColumnSelectorThree`,
  `add_static_file_mask!`

### `masks.rs`
- **Role**: Predefined masks for common static file segments.
- **Key items**: `HeaderMask`, `TotalDifficultyMask`, `BlockHashMask`, `ReceiptMask`,
  `TransactionMask`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StaticFileCursor`, `KeyOrNumber`
- **Modules / Packages**: `mask`, `masks`
- **Functions**: `iter_static_files()`, `get_one()`, `get_two()`

## Relationships
- **Depends on**: `reth_nippy_jar` for static file storage and cursoring.
- **Data/control flow**: column masks select subsets of row data for typed decoding.

## End-to-end flow (high level)
- Scan static file directory for segment jars.
- Create `StaticFileCursor` over a jar segment.
- Use mask selectors to decode one or more columns.
```

## File: crates/storage/db/src/AGENTS.md
```markdown
# src

## Purpose
MDBX-backed database implementation for reth, including environment management, metrics, and
static file helpers.

## Contents (one hop)
### Subdirectories
- [x] `implementation/` - Backend-specific implementations (MDBX).
- [x] `static_file/` - Static file cursor and mask helpers.

### Files
- `lib.rs` - Crate entrypoint wiring MDBX implementation and test utilities.
  - **Key items**: `DatabaseEnv`, `DatabaseEnvKind`, `create_db()`, `init_db()`, `open_db()`
- `lockfile.rs` - Storage lockfile helpers.
  - **Key items**: `StorageLock`
- `mdbx.rs` - Convenience helpers for creating/opening MDBX environments.
  - **Key items**: `create_db()`, `init_db_for()`, `open_db_read_only()`
- `metrics.rs` - Database operation and transaction metrics.
  - **Key items**: `DatabaseEnvMetrics`, `TransactionMode`, `TransactionOutcome`
- `utils.rs` - Environment helper utilities.
  - **Key items**: `default_page_size()`, `is_database_empty()`
- `version.rs` - Database version file management.
  - **Key items**: `DB_VERSION`, `check_db_version_file()`, `create_db_version_file()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseEnv`, `DatabaseArguments`, `DatabaseEnvMetrics`
- **Modules / Packages**: `mdbx`, `static_file`
- **Functions**: `create_db()`, `init_db()`, `open_db()`, `is_database_empty()`

## Relationships
- **Depends on**: `reth-db-api` for database traits and table sets.
- **Depends on**: `reth-libmdbx` for MDBX bindings (feature gated).
- **Data/control flow**: MDBX environment -> transactions -> cursors -> table encode/decode.
```

## File: crates/storage/db/AGENTS.md
```markdown
# db

## Purpose
`reth-db` crate: MDBX-backed implementation of the database abstraction layer with static file
helpers and benchmarks.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Database and serialization benchmarks.
- [x] `src/` - MDBX environment, cursor/tx implementation, metrics, and static file helpers.

### Files
- `Cargo.toml` - Crate manifest for MDBX-backed DB implementation.
  - **Key items**: feature `mdbx`, deps `reth-db-api`, `reth-libmdbx`, `reth-storage-errors`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseEnv`, `DatabaseArguments`, `DbCursorRO`, `DbTx`
- **Modules / Packages**: `mdbx`, `static_file`
- **Functions**: `create_db()`, `init_db()`, `open_db_read_only()`

## Relationships
- **Depends on**: `reth-db-api` for database traits and tables.
- **Depends on**: `reth-libmdbx` for MDBX environment and cursor bindings.
```

## File: crates/storage/db-api/src/models/AGENTS.md
```markdown
# models

## Purpose
Database model types and encoding helpers used by table definitions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Model re-exports and shared `Encode/Decode/Compress/Decompress` impls.
- **Key items**: primitive encode/decode impls, `impl_compression_for_compact!`
- **Interactions**: Bridges model types with `reth_db_api::table` codecs.

### `accounts.rs`
- **Role**: Account-related composite key types.
- **Key items**: `BlockNumberAddress`, `BlockNumberHashedAddress`, `AddressStorageKey`,
  `BlockNumberAddressRange`

### `blocks.rs`
- **Role**: Block-related model types.
- **Key items**: `StoredBlockOmmers`, `HeaderHash`

### `integer_list.rs`
- **Role**: Compressed integer list backed by Roaring bitmaps.
- **Key items**: `IntegerList`, `IntegerListError`
- **Interactions**: Implements `Compress`/`Decompress` and serde.

### `metadata.rs`
- **Role**: Storage configuration metadata models.
- **Key items**: `StorageSettings`
- **Knobs / invariants**: flags for static files and RocksDB usage.

### `sharded_key.rs`
- **Role**: Generic sharded key helper for large datasets.
- **Key items**: `ShardedKey`, `NUM_OF_INDICES_IN_SHARD`

### `storage_sharded_key.rs`
- **Role**: Sharded key for storage (address + slot + block).
- **Key items**: `StorageShardedKey`, `NUM_OF_INDICES_IN_SHARD`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockNumberAddress`, `IntegerList`, `StorageSettings`,
  `ShardedKey`
- **Modules / Packages**: `accounts`, `blocks`, `integer_list`

## Relationships
- **Used by**: `tables` definitions and table codecs.
- **Depends on**: `reth_codecs` for compact encoding and `reth_primitives_traits` types.
```

## File: crates/storage/db-api/src/tables/codecs/fuzz/AGENTS.md
```markdown
# fuzz

## Purpose
Fuzzing harnesses and curated inputs for table encode/decode round-trips.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Fuzz target macro definitions for encode/decode and compress/decompress.
- **Key items**: `impl_fuzzer_with_input!`, `impl_fuzzer_key!`, `impl_fuzzer_value_with_input!`

### `inputs.rs`
- **Role**: Curated fuzzer input types.
- **Key items**: `IntegerListInput`

## Key APIs (no snippets)
- **Modules / Packages**: `inputs`
```

## File: crates/storage/db-api/src/tables/codecs/AGENTS.md
```markdown
# codecs

## Purpose
Integrations between table codecs and encode/decode traits.

## Contents (one hop)
### Subdirectories
- [x] `fuzz/` - Fuzzing targets and curated inputs.

### Files
- `mod.rs` - Codec integration module entrypoint.
  - **Key items**: `fuzz` module

## Relationships
- **Used by**: `tables` to validate encode/decode implementations.
```

## File: crates/storage/db-api/src/tables/AGENTS.md
```markdown
# tables

## Purpose
Defines database tables, table metadata, and raw table access types.

## Contents (one hop)
### Subdirectories
- [x] `codecs/` - Codec integration and fuzz targets.

### Files
- `mod.rs` - Table definitions and table set utilities.
  - **Key items**: `TableType`, `TableViewer`, `TableSet`, `tables!` macro, `Tables` enum
- `raw.rs` - Raw table wrappers for delayed encoding/decoding.
  - **Key items**: `RawTable`, `RawDupSort`, `RawKey`, `RawValue`, `TableRawRow`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Tables`, `RawTable`, `RawKey`, `RawValue`
- **Modules / Packages**: `codecs`

## Relationships
- **Depends on**: `models` for table value types.
- **Used by**: DB implementations and providers to select tables and iterate.
```

## File: crates/storage/db-api/src/AGENTS.md
```markdown
# src

## Purpose
Database abstraction layer: traits for transactions, cursors, tables, and table models.

## Contents (one hop)
### Subdirectories
- [x] `models/` - Database model types and key/value encodings.
- [x] `tables/` - Table definitions and raw table wrappers.

### Files
- `lib.rs` - Crate entrypoint wiring modules and re-exports.
  - **Key items**: `Database`, `DbTx`, `DbCursorRO`, `Tables`, `DbTxUnwindExt`
- `common.rs` - Shared result types for cursor/iterator operations.
  - **Key items**: `PairResult`, `IterPairResult`, `ValueOnlyResult`
- `cursor.rs` - Cursor traits and walker iterators.
  - **Key items**: `DbCursorRO`, `DbCursorRW`, `DbDupCursorRO`, `Walker`, `RangeWalker`
- `database.rs` - Core `Database` trait.
  - **Key items**: `Database::tx()`, `Database::tx_mut()`, `view()`, `update()`
- `database_metrics.rs` - Metrics reporting hooks for database backends.
  - **Key items**: `DatabaseMetrics`, `report_metrics()`
- `mock.rs` - Mock database and cursor implementations for tests.
  - **Key items**: `DatabaseMock`, `TxMock`, `CursorMock`
- `table.rs` - Table and codec traits.
  - **Key items**: `Table`, `DupSort`, `Encode`, `Decode`, `Compress`, `Decompress`
- `transaction.rs` - Transaction traits and cursor type aliases.
  - **Key items**: `DbTx`, `DbTxMut`, `CursorTy`, `DupCursorTy`
- `unwind.rs` - Transaction unwind helpers.
  - **Key items**: `DbTxUnwindExt`
- `scale.rs` - SCALE codec integration for select types.
  - **Key items**: `ScaleValue`
- `utils.rs` - Arbitrary helpers for fixed-size types.
  - **Key items**: `impl_fixed_arbitrary!`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Database`, `DbTx`, `DbTxMut`, `DbCursorRO`, `Table`
- **Modules / Packages**: `cursor`, `table`, `tables`, `models`

## Relationships
- **Used by**: `reth-db` and storage providers to implement database backends.
```

## File: crates/storage/db-api/AGENTS.md
```markdown
# db-api

## Purpose
Database abstraction traits, table definitions, and codec integrations for reth storage.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core traits, models, table definitions, and utilities.

### Files
- `Cargo.toml` - Crate manifest for DB API.
  - **Key items**: features for codecs, fuzzing, and table definitions.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Database`, `DbTx`, `DbCursorRO`, `Table`, `Tables`
- **Modules / Packages**: `models`, `tables`

## Relationships
- **Used by**: `reth-db` and provider crates to implement storage backends.
```

## File: crates/storage/db-common/src/db_tool/AGENTS.md
```markdown
# db_tool

## Purpose
Helper utilities for inspecting and operating on database tables from tooling/CLI.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: DB helper tool and filters for listing/querying table data.
- **Key items**: `DbTool`, `ListFilter`, `list()`, `get()`, `drop_table()`
- **Interactions**: Uses `RawTable` and cursor walkers to scan tables.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DbTool`, `ListFilter`
- **Functions**: `list()`, `get()`, `drop()`
```

## File: crates/storage/db-common/src/AGENTS.md
```markdown
# src

## Purpose
Shared storage initialization and DB tooling utilities.

## Contents (one hop)
### Subdirectories
- [x] `db_tool/` - CLI/tooling helpers for table inspection.

### Files
- `lib.rs` - Crate entrypoint.
  - **Key items**: `init`, `db_tool` exports
- `init.rs` - Genesis and storage initialization routines.
  - **Key items**: `init_genesis()`, `init_genesis_with_settings()`, `InitStorageError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `InitStorageError`, `StorageSettings`
- **Modules / Packages**: `init`, `db_tool`

## Relationships
- **Depends on**: `reth_provider` and `reth_db_api` for storage read/write.
```

## File: crates/storage/db-common/AGENTS.md
```markdown
# db-common

## Purpose
Shared DB initialization utilities and tooling helpers for reth storage.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Genesis init and DB tooling utilities.

### Files
- `Cargo.toml` - Crate manifest for common DB utilities.
  - **Key items**: deps on `reth-provider`, `reth-db-api`, `reth-trie`

## Key APIs (no snippets)
- **Modules / Packages**: `init`, `db_tool`
```

## File: crates/storage/db-models/src/AGENTS.md
```markdown
# src

## Purpose
Storage model types shared by database layers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and module re-exports.
- **Key items**: `AccountBeforeTx`, `StoredBlockBodyIndices`, `ClientVersion`

### `accounts.rs`
- **Role**: Account pre-transaction models.
- **Key items**: `AccountBeforeTx`

### `blocks.rs`
- **Role**: Block-related storage models.
- **Key items**: `StoredBlockBodyIndices`, `StoredBlockWithdrawals`,
  `StaticFileBlockWithdrawals`

### `client_version.rs`
- **Role**: Client version model used by storage metadata.
- **Key items**: `ClientVersion`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `AccountBeforeTx`, `StoredBlockBodyIndices`, `ClientVersion`

## Relationships
- **Used by**: `db-api` model definitions and table value types.
```

## File: crates/storage/db-models/AGENTS.md
```markdown
# db-models

## Purpose
Shared storage model structs for DB tables and metadata.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Account, block, and client version models.

### Files
- `Cargo.toml` - Crate manifest for storage models.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `AccountBeforeTx`, `StoredBlockBodyIndices`, `ClientVersion`
```

## File: crates/storage/errors/src/AGENTS.md
```markdown
# src

## Purpose
Shared storage error types for database, provider, and lockfile layers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: `db`, `lockfile`, `provider`, `any`

### `db.rs`
- **Role**: Database error types and log level helpers.
- **Key items**: `DatabaseError`, `DatabaseErrorInfo`, `DatabaseWriteError`, `LogLevel`

### `lockfile.rs`
- **Role**: Storage lockfile error definitions.
- **Key items**: `StorageLockError`

### `provider.rs`
- **Role**: Provider-layer error types and helpers.
- **Key items**: `ProviderError`, `ProviderResult`

### `any.rs`
- **Role**: Cloneable error wrapper for arbitrary errors.
- **Key items**: `AnyError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseError`, `ProviderError`, `StorageLockError`
```

## File: crates/storage/errors/AGENTS.md
```markdown
# errors

## Purpose
Shared error types used across storage and provider crates.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Error definitions and wrappers.

### Files
- `Cargo.toml` - Crate manifest for storage errors.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseError`, `ProviderError`, `StorageLockError`
```

## File: crates/storage/libmdbx-rs/benches/AGENTS.md
```markdown
# benches

## Purpose
Benchmarks for MDBX cursor and transaction performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `cursor.rs`
- **Role**: Cursor performance benchmarks.
- **Key items**: benchmark groups for read/write cursor ops

### `transaction.rs`
- **Role**: Transaction-level performance benchmarks.
- **Key items**: begin/commit/put loops

### `utils.rs`
- **Role**: Benchmark helpers and dataset generation.
- **Key items**: env setup and data generators
```

## File: crates/storage/libmdbx-rs/mdbx-sys/libmdbx/cmake/AGENTS.md
```markdown
# cmake

## Purpose
Vendor CMake helper modules for building libmdbx.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `compiler.cmake`
- **Role**: Compiler feature and flag detection.

### `profile.cmake`
- **Role**: Build profiles and optimization settings.

### `utils.cmake`
- **Role**: CMake helper utilities.
```

## File: crates/storage/libmdbx-rs/mdbx-sys/libmdbx/man1/AGENTS.md
```markdown
# man1

## Purpose
Manual pages for libmdbx command-line tools.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mdbx_chk.1`
- **Role**: `mdbx_chk` man page.

### `mdbx_copy.1`
- **Role**: `mdbx_copy` man page.

### `mdbx_drop.1`
- **Role**: `mdbx_drop` man page.

### `mdbx_dump.1`
- **Role**: `mdbx_dump` man page.

### `mdbx_load.1`
- **Role**: `mdbx_load` man page.

### `mdbx_stat.1`
- **Role**: `mdbx_stat` man page.
```

## File: crates/storage/libmdbx-rs/mdbx-sys/libmdbx/AGENTS.md
```markdown
# libmdbx

## Purpose
Vendored libmdbx C/C++ source and build files.

## Contents (one hop)
### Subdirectories
- [x] `cmake/` - CMake helper modules.
- [x] `man1/` - Command-line tool man pages.

### Files
- `mdbx.c`, `mdbx.c++` - Core libmdbx implementation.
- `mdbx.h`, `mdbx.h++` - Public C/C++ headers.
- `CMakeLists.txt`, `Makefile`, `GNUmakefile` - Build scripts.
- `config.h.in` - Build configuration template.
- `VERSION.json` - Upstream version metadata.
- `LICENSE`, `NOTICE` - Licensing and notices.
- `mdbx_chk.c`, `mdbx_copy.c`, `mdbx_drop.c`, `mdbx_dump.c`, `mdbx_load.c`, `mdbx_stat.c` - CLI tools.
- `ntdll.def` - Windows export definitions.

## Relationships
- **Used by**: `mdbx-sys` build to compile the native library.
```

## File: crates/storage/libmdbx-rs/mdbx-sys/src/AGENTS.md
```markdown
# src

## Purpose
Generated Rust FFI bindings for libmdbx.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: FFI bindings entrypoint including generated `bindings.rs`.
- **Key items**: `include!(.../bindings.rs)`
```

## File: crates/storage/libmdbx-rs/mdbx-sys/AGENTS.md
```markdown
# mdbx-sys

## Purpose
FFI layer and build glue for vendored libmdbx.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Generated Rust bindings.
- [x] `libmdbx/` - Vendored libmdbx sources.

### Files
- `build.rs` - Build script to compile and bind libmdbx.
- `Cargo.toml` - Crate manifest for MDBX FFI bindings.

## Key APIs (no snippets)
- **Modules / Packages**: `libmdbx` (vendor), `bindings`
```

## File: crates/storage/libmdbx-rs/src/AGENTS.md
```markdown
# src

## Purpose
Rust wrapper around libmdbx with safe environment, transaction, and cursor APIs.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: `Environment`, `Transaction`, `Cursor`, `Database`, `Error`

### `environment.rs`
- **Role**: MDBX environment builder and runtime handles.
- **Key items**: `Environment`, `EnvironmentBuilder`, `Geometry`, `Stat`, `Info`

### `transaction.rs`
- **Role**: Transaction wrappers for RO/RW operations.
- **Key items**: `Transaction`, `TransactionKind`, `RO`, `RW`, `CommitLatency`

### `cursor.rs`
- **Role**: Cursor wrappers and iterators.
- **Key items**: `Cursor`, `Iter`, `IterDup`

### `database.rs`
- **Role**: Database handle wrappers and DBI helpers.
- **Key items**: `Database`, DBI helpers

### `flags.rs`
- **Role**: MDBX flag types and bitflags.
- **Key items**: env/txn/db flags

### `codec.rs`
- **Role**: Encoding helpers for MDBX values.
- **Key items**: codec helpers and traits

### `error.rs`
- **Role**: Error types for MDBX wrapper.
- **Key items**: `Error`, `Result`

### `txn_manager.rs`
- **Role**: Transaction manager utilities.
- **Key items**: read-transaction tracking

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Environment`, `Transaction`, `Cursor`, `Database`

## Relationships
- **Depends on**: `mdbx-sys` for FFI bindings.
```

## File: crates/storage/libmdbx-rs/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration tests for MDBX environment, cursor, and transaction APIs.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `cursor.rs`
- **Role**: Cursor behavior tests.
- **Key items**: iteration and seek correctness

### `environment.rs`
- **Role**: Environment builder and configuration tests.
- **Key items**: open/close behaviors

### `transaction.rs`
- **Role**: Transaction lifecycle tests.
- **Key items**: commit/abort semantics
```

## File: crates/storage/libmdbx-rs/AGENTS.md
```markdown
# libmdbx-rs

## Purpose
Rust wrapper and FFI bindings for the libmdbx key-value store.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Safe Rust wrappers for MDBX.
- [x] `mdbx-sys/` - FFI bindings and vendored libmdbx sources.
- [x] `benches/` - Performance benchmarks.
- [x] `tests/` - Integration tests.

### Files
- `Cargo.toml` - Crate manifest for Rust MDBX wrapper.
- `README.md` - Crate overview and usage notes.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Environment`, `Transaction`, `Cursor`, `Database`
```

## File: crates/storage/nippy-jar/src/compression/AGENTS.md
```markdown
# compression

## Purpose
Compression backends and traits for NippyJar columnar storage.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Compression trait and enum wrapper.
- **Key items**: `Compression`, `Compressors`

### `lz4.rs`
- **Role**: LZ4 compression backend.
- **Key items**: `Lz4`

### `zstd.rs`
- **Role**: Zstd compression backend and dictionary support.
- **Key items**: `Zstd`, `Decompressor`, `DecoderDictionary`
```

## File: crates/storage/nippy-jar/src/AGENTS.md
```markdown
# src

## Purpose
Immutable columnar storage format (NippyJar) with cursor, writer, and consistency checks.

## Contents (one hop)
### Subdirectories
- [x] `compression/` - Compression backends and traits.

### Files
- `lib.rs` - Core NippyJar type and helpers.
  - **Key items**: `NippyJar`, `NippyJarHeader`, `Compressors`, `NippyJarError`
- `cursor.rs` - Read cursor for NippyJar rows.
  - **Key items**: `NippyJarCursor`
- `writer.rs` - Builder/writer for NippyJar files.
  - **Key items**: `NippyJarWriter`
- `consistency.rs` - Consistency checker for jar contents.
  - **Key items**: `NippyJarChecker`
- `error.rs` - NippyJar error types.
  - **Key items**: `NippyJarError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `NippyJar`, `NippyJarCursor`, `NippyJarWriter`

## Relationships
- **Used by**: static file providers and storage segments.
```

## File: crates/storage/nippy-jar/AGENTS.md
```markdown
# nippy-jar

## Purpose
Immutable columnar storage format used for static file segments.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core NippyJar types, cursor, writer, compression.

### Files
- `Cargo.toml` - Crate manifest for NippyJar.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `NippyJar`, `NippyJarWriter`, `NippyJarCursor`
```

## File: crates/storage/provider/src/changesets_utils/AGENTS.md
```markdown
# changesets_utils

## Purpose
Utilities for changeset table handling and state reverts.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module entrypoint and re-exports.
- **Key items**: `StorageRevertsIter`

### `state_reverts.rs`
- **Role**: Iterator to merge storage reverts and wiped entries.
- **Key items**: `StorageRevertsIter`
```

## File: crates/storage/provider/src/providers/database/AGENTS.md
```markdown
# database

## Purpose
Database-backed provider factory and provider implementations.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Provider factory and wiring of database providers.
- **Key items**: `ProviderFactory`, `DatabaseProvider`, `SaveBlocksMode`

### `provider.rs`
- **Role**: Database provider implementations (RO/RW).
- **Key items**: `DatabaseProviderRO`, `DatabaseProviderRW`

### `builder.rs`
- **Role**: Provider factory builder and read-only config.
- **Key items**: `ProviderFactoryBuilder`, `ReadOnlyConfig`

### `metrics.rs`
- **Role**: Metrics for database provider operations.
- **Key items**: provider metrics types

### `chain.rs`
- **Role**: Chain storage adapter traits.
- **Key items**: `ChainStorage`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProviderFactory`, `DatabaseProvider`
```

## File: crates/storage/provider/src/providers/rocksdb/AGENTS.md
```markdown
# rocksdb

## Purpose
RocksDB-backed history storage provider and helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: RocksDB provider module entrypoint and exports.
- **Key items**: `RocksDBProvider`, `RocksDBBuilder`, `RocksTx`

### `provider.rs`
- **Role**: RocksDB provider implementation.
- **Key items**: read/write APIs for history storage

### `metrics.rs`
- **Role**: RocksDB metrics helpers.
- **Key items**: metrics structs and recording

### `invariants.rs`
- **Role**: RocksDB invariants and validation helpers.
- **Key items**: invariant checks
```

## File: crates/storage/provider/src/providers/state/AGENTS.md
```markdown
# state

## Purpose
State provider implementations for latest, historical, and overlay views.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: State provider module entrypoint and exports.
- **Key items**: `HistoricalStateProvider`, `LatestStateProvider`, `OverlayStateProvider`

### `historical.rs`
- **Role**: Historical state provider implementation and helpers.
- **Key items**: `HistoryInfo`, `LowestAvailableBlocks`

### `latest.rs`
- **Role**: Latest state provider implementation.
- **Key items**: `LatestStateProvider`

### `overlay.rs`
- **Role**: Overlay state provider for temporary state.
- **Key items**: `OverlayStateProvider`, `OverlayStateProviderFactory`
```

## File: crates/storage/provider/src/providers/static_file/AGENTS.md
```markdown
# static_file

## Purpose
Static file provider, writer, and manager for immutable segment storage.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Static file provider entrypoint and exports.
- **Key items**: `StaticFileProvider`, `StaticFileWriter`, `StaticFileAccess`

### `jar.rs`
- **Role**: Jar-level helpers for static files.
- **Key items**: jar reader/writer helpers

### `manager.rs`
- **Role**: Static file manager for segment lifecycle.
- **Key items**: manager struct and segment registry

### `metrics.rs`
- **Role**: Metrics for static file operations.
- **Key items**: metric recording helpers

### `writer.rs`
- **Role**: Static file writer implementation.
- **Key items**: `StaticFileWriter`
```

## File: crates/storage/provider/src/providers/AGENTS.md
```markdown
# providers

## Purpose
Concrete provider implementations backed by database, static files, or RocksDB.

## Contents (one hop)
### Subdirectories
- [x] `database/` - Database-backed provider factory and DB providers.
- [x] `rocksdb/` - RocksDB-backed history providers.
- [x] `state/` - Latest/historical/overlay state providers.
- [x] `static_file/` - Static file provider and writer.

### Files
- `mod.rs` - Provider module wiring and exports.
  - **Key items**: `BlockchainProvider`, `ConsistentProvider`, `StaticFileProvider`
- `blockchain_provider.rs` - High-level blockchain provider implementation.
- `consistent.rs` - Consistent provider wrapper.
- `consistent_view.rs` - Consistent DB view utilities.
  - **Key items**: `ConsistentDbView`, `ConsistentViewError`
- `rocksdb_stub.rs` - Stub module for non-RocksDB builds.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockchainProvider`, `ProviderFactory`
```

## File: crates/storage/provider/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Test helpers and mock providers for storage.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test utilities entrypoint and exports.

### `blocks.rs`
- **Role**: Block test helpers and fixtures.

### `mock.rs`
- **Role**: Mock provider implementations.

### `noop.rs`
- **Role**: No-op provider helpers.
```

## File: crates/storage/provider/src/traits/AGENTS.md
```markdown
# traits

## Purpose
Provider trait definitions and factory abstractions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Trait re-exports and module wiring.
- **Key items**: `FullProvider`, `StaticFileProviderFactory`, `RocksDBProviderFactory`

### `full.rs`
- **Role**: Full provider trait combining common provider capabilities.
- **Key items**: `FullProvider`

### `rocksdb_provider.rs`
- **Role**: RocksDB provider factory trait.
- **Key items**: `RocksDBProviderFactory`

### `static_file_provider.rs`
- **Role**: Static file provider factory trait.
- **Key items**: `StaticFileProviderFactory`
```

## File: crates/storage/provider/src/writer/AGENTS.md
```markdown
# writer

## Purpose
Provider write-path tests and helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test module for write-path behaviors.
- **Key items**: write-state and revert tests
```

## File: crates/storage/provider/src/AGENTS.md
```markdown
# src

## Purpose
Provider traits and implementations for accessing chain state, history, and static files.

## Contents (one hop)
### Subdirectories
- [x] `changesets_utils/` - Changeset utilities and iterators.
- [x] `providers/` - Database, static file, state, and RocksDB providers.
- [x] `test_utils/` - Test helpers and mocks.
- [x] `traits/` - Provider trait definitions and factories.
- [x] `writer/` - Write-path tests.

### Files
- `lib.rs` - Crate entrypoint and re-exports.
  - **Key items**: provider exports, `to_range()`
- `changeset_walker.rs` - Changeset iteration over static files.
  - **Key items**: `StaticFileAccountChangesetWalker`
- `either_writer.rs` - Abstraction over writers with different destinations.
  - **Key items**: `EitherWriter` and destination types

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProviderFactory`, `DatabaseProvider`, `StaticFileWriter`
```

## File: crates/storage/provider/AGENTS.md
```markdown
# provider

## Purpose
Provider abstraction and implementations for database, static file, and history storage access.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Provider traits, implementations, and helpers.

### Files
- `Cargo.toml` - Crate manifest for storage provider.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProviderFactory`, `DatabaseProvider`, `StaticFileProvider`
```

## File: crates/storage/rpc-provider/src/AGENTS.md
```markdown
# src

## Purpose
RPC-based provider implementing storage traits via remote node calls.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: RPC blockchain provider implementation.
- **Key items**: `RpcBlockchainProvider`, `RpcBlockchainProviderConfig`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `RpcBlockchainProvider`, `RpcBlockchainProviderConfig`
```

## File: crates/storage/rpc-provider/AGENTS.md
```markdown
# rpc-provider

## Purpose
RPC-backed implementation of storage provider traits.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC provider implementation.

### Files
- `Cargo.toml` - Crate manifest for RPC provider.
- `README.md` - Usage overview.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `RpcBlockchainProvider`
```

## File: crates/storage/storage-api/src/AGENTS.md
```markdown
# src

## Purpose
Storage access traits and helper types for blocks, state, receipts, and metadata.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: module re-exports and `StorageSettings` gating

### `account.rs`
- **Role**: Account and changeset reader traits.
- **Key items**: `AccountReader`, `AccountExtReader`, `ChangeSetReader`

### `block.rs`
- **Role**: Block reader trait and `BlockSource` enum.
- **Key items**: `BlockReader`, `BlockSource`, `ProviderBlock`

### `block_hash.rs`
- **Role**: Block hash reader trait.
- **Key items**: `BlockHashReader`

### `block_id.rs`
- **Role**: Block id/number conversion traits.
- **Key items**: `BlockNumReader`, `BlockIdReader`

### `block_indices.rs`
- **Role**: Block body index readers.
- **Key items**: `BlockBodyIndicesProvider`

### `block_writer.rs`
- **Role**: Block and execution write traits.
- **Key items**: `BlockWriter`, `BlockExecutionWriter`

### `chain.rs`
- **Role**: Chain storage reader/writer traits and Eth storage impls.
- **Key items**: `BlockBodyReader`, `BlockBodyWriter`, `ChainStorageReader`, `ChainStorageWriter`,
  `EthStorage`

### `chain_info.rs`
- **Role**: Canon chain tracking interface.
- **Key items**: `CanonChainTracker`

### `database_provider.rs`
- **Role**: DB provider trait and helpers for table access.
- **Key items**: `DBProvider`

### `full.rs`
- **Role**: Full RPC provider trait alias.
- **Key items**: `FullRpcProvider`

### `hashing.rs`
- **Role**: Hashing writer trait for account/storage hashing tables.
- **Key items**: `HashingWriter`

### `header.rs`
- **Role**: Header reader trait.
- **Key items**: `HeaderProvider`, `ProviderHeader`

### `header_sync_gap.rs`
- **Role**: Sync gap provider for headers.
- **Key items**: `HeaderSyncGapProvider`

### `history.rs`
- **Role**: History index writer trait.
- **Key items**: `HistoryWriter`

### `macros.rs`
- **Role**: Macro helpers for provider trait delegation.
- **Key items**: `delegate_impls_to_as_ref!`, `delegate_provider_impls!`

### `metadata.rs`
- **Role**: Metadata and storage settings providers.
- **Key items**: `MetadataProvider`, `MetadataWriter`, `StorageSettingsCache`

### `noop.rs`
- **Role**: No-op provider implementations for tests.
- **Key items**: `NoopProvider`

### `primitives.rs`
- **Role**: Node primitives provider trait.
- **Key items**: `NodePrimitivesProvider`

### `prune_checkpoint.rs`
- **Role**: Prune checkpoint read/write traits.
- **Key items**: `PruneCheckpointReader`, `PruneCheckpointWriter`

### `receipts.rs`
- **Role**: Receipt reader traits and id extensions.
- **Key items**: `ReceiptProvider`, `ReceiptProviderIdExt`, `ProviderReceipt`

### `stage_checkpoint.rs`
- **Role**: Stage checkpoint read/write traits.
- **Key items**: `StageCheckpointReader`, `StageCheckpointWriter`

### `state.rs`
- **Role**: State provider and state-reading traits.
- **Key items**: `StateProvider`, `StateReader`, `StateProviderBox`, `HashedPostStateProvider`,
  `BytecodeReader`, `TryIntoHistoricalStateProvider`

### `state_writer.rs`
- **Role**: State write traits and configuration.
- **Key items**: `StateWriter`, `StateWriteConfig`

### `stats.rs`
- **Role**: Provider statistics trait.
- **Key items**: `StatsReader`

### `storage.rs`
- **Role**: Storage readers and changeset readers.
- **Key items**: `StorageReader`, `StorageChangeSetReader`

### `transactions.rs`
- **Role**: Transaction provider traits.
- **Key items**: `TransactionsProvider`, `TransactionsProviderExt`, `TransactionVariant`

### `trie.rs`
- **Role**: Trie-related read/proof and write traits.
- **Key items**: `StateRootProvider`, `StorageRootProvider`, `StateProofProvider`, `TrieWriter`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockReader`, `StateProvider`, `ReceiptProvider`, `StateWriter`
```

## File: crates/storage/storage-api/AGENTS.md
```markdown
# storage-api

## Purpose
Storage access traits and helper types shared across storage providers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Storage trait definitions and helpers.

### Files
- `Cargo.toml` - Crate manifest for storage API.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockReader`, `StateProvider`, `ReceiptProvider`
```

## File: crates/storage/zstd-compressors/src/AGENTS.md
```markdown
# src

## Purpose
Zstd compressors and decompressors with shared dictionaries for receipts/transactions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Dictionary-backed compressors/decompressors and helpers.
- **Key items**: `RECEIPT_DICTIONARY`, `TRANSACTION_DICTIONARY`,
  `create_tx_compressor()`, `create_receipt_compressor()`, `ReusableDecompressor`
```

## File: crates/storage/zstd-compressors/AGENTS.md
```markdown
# zstd-compressors

## Purpose
Shared Zstd dictionaries and compressor helpers for storage types.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Compressor helpers and reusable decompressors.

### Files
- `Cargo.toml` - Crate manifest for zstd compressors.
- `receipt_dictionary.bin` - Zstd dictionary for receipts.
- `transaction_dictionary.bin` - Zstd dictionary for transactions.
```

## File: crates/storage/AGENTS.md
```markdown
# storage

## Purpose
Storage crates for database access, providers, APIs, and static file formats.

## Contents (one hop)
### Subdirectories
- [x] `codecs/` - Compact codecs and derive helpers.
- [x] `db/` - MDBX-backed DB implementation.
- [x] `db-api/` - Database abstraction traits and table definitions.
- [x] `db-common/` - Shared DB init and tooling helpers.
- [x] `db-models/` - Storage model structs.
- [x] `errors/` - Shared storage error types.
- [x] `libmdbx-rs/` - Rust MDBX wrapper and FFI.
- [x] `nippy-jar/` - Immutable columnar storage format.
- [x] `provider/` - Provider traits and implementations.
- [x] `rpc-provider/` - RPC-backed provider implementation.
- [x] `storage-api/` - Storage access trait definitions.
- [x] `zstd-compressors/` - Zstd dictionary compressors.

## Relationships
- **Used by**: higher-level chain, sync, and RPC components.
```

## File: crates/tasks/src/AGENTS.md
```markdown
# src

## Purpose
Task management utilities: task spawners, manager, shutdown signals, and metrics.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Core task manager and spawner traits plus executor helpers.
  - **Key items**: `TaskSpawner`, `TokioTaskExecutor`, `TaskManager`, `TaskExecutor`
- `metrics.rs` - Task executor metrics counters and helpers.
  - **Key items**: `TaskExecutorMetrics`, `IncCounterOnDrop`
- `shutdown.rs` - Shutdown signaling primitives for graceful shutdown.
  - **Key items**: `GracefulShutdown`, `GracefulShutdownGuard`, `Shutdown`, `Signal`, `signal()`
- `pool.rs` - Rayon-based blocking task pool (feature `rayon`).
  - **Key items**: `BlockingTaskPool`, `BlockingTaskGuard`, `BlockingTaskHandle`

## Key APIs (no snippets)
- `TaskManager`, `TaskSpawner`, `TaskExecutor`
- `Shutdown`, `GracefulShutdown`
```

## File: crates/tasks/AGENTS.md
```markdown
# tasks

## Purpose
`reth-tasks` crate: task management, executors, and shutdown signaling.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Task spawners, manager, metrics, and shutdown helpers.

### Files
- `Cargo.toml` - Manifest for task utilities and optional rayon pool.
  - **Key items**: feature `rayon`
```

## File: crates/tokio-util/src/AGENTS.md
```markdown
# src

## Purpose
Tokio utilities for event broadcasting and rate limiting.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and public exports.
  - **Key items**: `EventSender`, `EventStream`, `ratelimit` (feature-gated)
- `event_sender.rs` - Broadcast event sender with listener creation.
  - **Key items**: `EventSender`, `notify()`, `new_listener()`
- `event_stream.rs` - Stream wrapper that skips broadcast lag errors.
  - **Key items**: `EventStream`
- `ratelimit.rs` - Simple rate limiter for async workflows (feature `time`).
  - **Key items**: `RateLimit`, `Rate`, `poll_ready()`, `tick()`

## Key APIs (no snippets)
- `EventSender`, `EventStream`
- `RateLimit`, `Rate`
```

## File: crates/tokio-util/AGENTS.md
```markdown
# tokio-util

## Purpose
`reth-tokio-util` crate: event broadcasting streams and optional rate limiting utilities.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Event sender/stream wrappers and rate limiter.

### Files
- `Cargo.toml` - Manifest for tokio utility dependencies and `time` feature.
  - **Key items**: feature `time`
```

## File: crates/tracing/src/AGENTS.md
```markdown
# src

## Purpose
Tracing configuration helpers and layer builders for reth logging.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Main tracer configuration API and re-exports.
  - **Key items**: `RethTracer`, `LayerInfo`, `Tracer`, `LogFormat`
- `layers.rs` - Layer builders for stdout, file, journald, samply, and OTLP.
  - **Key items**: `Layers`, `FileInfo`, `FileWorkerGuard`
  - **Knobs / invariants**: Default filter directives suppress noisy deps.
- `formatter.rs` - Log format enum and layer builder for JSON/logfmt/terminal.
  - **Key items**: `LogFormat`, `apply()`
- `throttle.rs` - Throttling macro for rate-limited logging.
  - **Key items**: `throttle!`, `should_run()`
- `test_tracer.rs` - Minimal tracer for tests.
  - **Key items**: `TestTracer`

## Key APIs (no snippets)
- `RethTracer`, `LayerInfo`, `Layers`
- `LogFormat`, `throttle!`
```

## File: crates/tracing/AGENTS.md
```markdown
# tracing

## Purpose
`reth-tracing` crate: logging/tracing configuration, formatters, and layer builders.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Tracer config, layer builders, formatter, and throttle macro.

### Files
- `Cargo.toml` - Manifest for tracing dependencies and optional OTLP/Tracy support.
  - **Key items**: features `otlp`, `otlp-logs`, `tracy`
```

## File: crates/tracing-otlp/src/AGENTS.md
```markdown
# src

## Purpose
OpenTelemetry OTLP tracing and logging layers for reth.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Builds OTLP tracing/logging layers and configuration types.
  - **Key items**: `span_layer()`, `log_layer()`, `OtlpConfig`, `OtlpLogsConfig`, `OtlpProtocol`
  - **Knobs / invariants**: `sample_ratio` must be in `[0.0, 1.0]`; endpoint protocol `http`/`grpc`.

## Key APIs (no snippets)
- `span_layer()`, `log_layer()`
- `OtlpConfig`, `OtlpLogsConfig`
```

## File: crates/tracing-otlp/AGENTS.md
```markdown
# tracing-otlp

## Purpose
`reth-tracing-otlp` crate: OpenTelemetry OTLP tracing/logging layer helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OTLP layer builders and config types.

### Files
- `Cargo.toml` - Manifest for OTLP dependencies and feature flags.
  - **Key items**: features `otlp`, `otlp-logs`
```

## File: crates/transaction-pool/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for transaction pool insertion, ordering, eviction, and updates.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `canonical_state_change.rs`
- **Role**: Benchmarks pool updates on canonical state changes.
- **Key items**: `canonical_state_change_bench()`, `fill_pool()`

### `insertion.rs`
- **Role**: Benchmarks individual and batch transaction insertion.
- **Key items**: `txpool_insertion()`, `txpool_batch_insertion()`

### `priority.rs`
- **Role**: Benchmarks blob priority and fee delta calculations.
- **Key items**: `blob_tx_priority()`, `fee_delta()`

### `reorder.rs`
- **Role**: Benchmarks reordering strategies for base-fee changes.
- **Key items**: `BenchTxPool`, `txpool_reordering_bench()`

### `truncate.rs`
- **Role**: Benchmarks subpool truncation behavior.
- **Key items**: `truncate_pending()`, `truncate_blob()`, `benchmark_pools()`

## End-to-end flow (high level)
- Generate mock transaction sets and configure pool limits.
- Run Criterion benchmarks for insertion, ordering, and eviction paths.

## Key APIs (no snippets)
- `canonical_state_change_bench()`, `txpool_insertion()`, `txpool_reordering()`
```

## File: crates/transaction-pool/src/blobstore/AGENTS.md
```markdown
# blobstore

## Purpose
Blob sidecar storage for EIP-4844 transactions (in-memory, disk, or noop), plus tracking helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Blob store trait, errors, size tracking, and implementation re-exports.
- **Key items**: `BlobStore`, `BlobStoreError`, `BlobStoreCleanupStat`, `BlobStoreCanonTracker`
- **Interactions**: Used by pool maintenance to store/cleanup blob sidecars.

### `converter.rs`
- **Role**: Converts blob sidecars to EIP-7594 format with bounded concurrency.
- **Key items**: `BlobSidecarConverter`
- **Knobs / invariants**: Global semaphore caps concurrent conversions.

### `disk.rs`
- **Role**: Disk-backed blob store with caching and deferred deletion.
- **Key items**: `DiskFileBlobStore`, `DiskFileBlobStoreConfig`, `OpenDiskFileBlobStore`
- **Knobs / invariants**: Cleanup required to remove deferred deletions; cache size limits.

### `mem.rs`
- **Role**: In-memory blob store with size tracking.
- **Key items**: `InMemoryBlobStore`

### `noop.rs`
- **Role**: No-op blob store implementation for wiring/testing.
- **Key items**: `NoopBlobStore`

### `tracker.rs`
- **Role**: Tracks blob transactions included in canonical blocks.
- **Key items**: `BlobStoreCanonTracker`, `BlobStoreUpdates`
- **Interactions**: Produces deletions when blocks finalize.

## End-to-end flow (high level)
- Validators extract blob sidecars and insert into a `BlobStore`.
- Pool maintenance uses `BlobStoreCanonTracker` to mark finalized blobs.
- The blob store deletes finalized blobs and optionally cleans up deferred deletions.

## Key APIs (no snippets)
- `BlobStore`, `DiskFileBlobStore`, `InMemoryBlobStore`
- `BlobStoreCanonTracker`, `BlobSidecarConverter`
```

## File: crates/transaction-pool/src/pool/AGENTS.md
```markdown
# pool

## Purpose
Internal transaction pool implementation: subpools, ordering, event listeners, and update logic.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Pool internals entrypoint, `PoolInner` implementation, and re-exports.
- **Key items**: `PoolInner`, `PENDING_TX_LISTENER_BUFFER_SIZE`, `NEW_TX_LISTENER_BUFFER_SIZE`
- **Interactions**: Wraps `TxPool`, manages listeners and blob store access.

### `txpool.rs`
- **Role**: Core pool data structure with pending/queued/basefee/blob subpools.
- **Key items**: `TxPool`, `AllTransactions`, `PendingFees`
- **Knobs / invariants**: Enforces subpool limits and ordering; uses base fee and blob fee.

### `state.rs`
- **Role**: Transaction state flags and subpool classification.
- **Key items**: `TxState`, `SubPool`, `determine_queued_reason()`
- **Knobs / invariants**: Bitflag thresholds map to subpool selection.

### `pending.rs`
- **Role**: Pending subpool storing gapless, executable transactions.
- **Key items**: `PendingPool`, `PendingTransaction`
- **Interactions**: Feeds `BestTransactions` iterator for block production.

### `parked.rs`
- **Role**: Parked subpools for queued/basefee transactions.
- **Key items**: `ParkedPool`, `ParkedOrd`, `BasefeeOrd`, `QueuedOrd`
- **Knobs / invariants**: Enforces sender slot limits and truncation rules.

### `best.rs`
- **Role**: Iterators for best-transaction selection with fee constraints.
- **Key items**: `BestTransactions`, `BestTransactionsWithFees`

### `blob.rs`
- **Role**: Blob transaction subpool and prioritization helpers.
- **Key items**: `BlobTransactions`, `BlobOrd`, `blob_tx_priority()`, `fee_delta()`
- **Knobs / invariants**: Blob transactions must be gapless.

### `events.rs`
- **Role**: Pool event types for transaction lifecycle changes.
- **Key items**: `FullTransactionEvent`, `TransactionEvent`, `NewTransactionEvent`

### `listener.rs`
- **Role**: Event broadcast streams for per-tx and all-tx listeners.
- **Key items**: `TransactionEvents`, `AllTransactionsEvents`, `PoolEventBroadcast`
- **Knobs / invariants**: Event channel buffer sizes and final event termination.

### `size.rs`
- **Role**: Size accounting utility for pool memory tracking.
- **Key items**: `SizeTracker`

### `update.rs`
- **Role**: Update bookkeeping for pool promotions/demotions.
- **Key items**: `PoolUpdate`, `Destination`, `UpdateOutcome`

## End-to-end flow (high level)
- `PoolInner` validates and inserts transactions into `TxPool`.
- `TxPool` classifies transactions into subpools using `TxState` and fees.
- `PendingPool` yields best transactions via `BestTransactions`.
- Updates move transactions across subpools and emit events to listeners.

## Key APIs (no snippets)
- `PoolInner`, `TxPool`, `PendingPool`, `ParkedPool`
- `BestTransactions`, `BlobTransactions`
- `TransactionEvents`, `FullTransactionEvent`
```

## File: crates/transaction-pool/src/test_utils/AGENTS.md
```markdown
# test_utils

## Purpose
Testing helpers: mock transactions, validators, pools, and generators.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test pool builder and module wiring.
- **Key items**: `TestPool`, `TestPoolBuilder`, `testing_pool()`

### `mock.rs`
- **Role**: Mock transaction types, factories, and ordering for pool tests.
- **Key items**: `MockTransaction`, `MockTransactionFactory`, `MockOrdering`
- **Interactions**: Used by benches and integration tests.

### `okvalidator.rs`
- **Role**: Validator that always returns transactions as valid.
- **Key items**: `OkValidator`

### `pool.rs`
- **Role**: Mock pool wrapper and simulator for scenario testing.
- **Key items**: `MockPool`, `MockTransactionSimulator`

### `tx_gen.rs`
- **Role**: Random transaction generator and builder helpers.
- **Key items**: `TransactionGenerator`, `TransactionBuilder`

## End-to-end flow (high level)
- Build a `TestPool` with `TestPoolBuilder` and in-memory blob store.
- Generate mock transactions and feed them into the pool.
- Use `OkValidator` or mock validators to control validation outcomes.

## Key APIs (no snippets)
- `TestPoolBuilder`, `MockTransaction`, `OkValidator`
- `TransactionGenerator`, `MockTransactionSimulator`
```

## File: crates/transaction-pool/src/validate/AGENTS.md
```markdown
# validate

## Purpose
Transaction validation logic and task executors for pool admission.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Validation outcome types and validator traits.
- **Key items**: `TransactionValidationOutcome`, `ValidTransaction`, `TransactionValidator`
- **Interactions**: Used by pool insertion to gate transactions.

### `constants.rs`
- **Role**: Validation size limits for inputs/bytecode.
- **Key items**: `DEFAULT_MAX_TX_INPUT_BYTES`, `MAX_CODE_BYTE_SIZE`, `MAX_INIT_CODE_BYTE_SIZE`

### `eth.rs`
- **Role**: Ethereum-specific transaction validator with fork tracking and fee checks.
- **Key items**: `EthTransactionValidator`, `EthTransactionValidatorBuilder`, `ForkTracker`
- **Knobs / invariants**: Enforces max size, gas limit, and fork-activated tx types.

### `task.rs`
- **Role**: Async validation task runner and executor wrapper.
- **Key items**: `ValidationTask`, `ValidationJobSender`, `TransactionValidationTaskExecutor`
- **Interactions**: Spawns validation futures via task spawner.

## End-to-end flow (high level)
- Pool submits transactions to a `TransactionValidator`.
- `TransactionValidationTaskExecutor` dispatches validation jobs to `ValidationTask`.
- Outcomes are returned as `TransactionValidationOutcome` for pool insertion.

## Key APIs (no snippets)
- `TransactionValidator`, `TransactionValidationOutcome`
- `EthTransactionValidator`, `TransactionValidationTaskExecutor`
```

## File: crates/transaction-pool/src/AGENTS.md
```markdown
# src

## Purpose
Transaction pool core implementation: configuration, traits, ordering, validation, and maintenance.

## Contents (one hop)
### Subdirectories
- [x] `blobstore/` - Blob sidecar storage implementations and tracking.
- [x] `pool/` - Internal subpool logic, events, and update helpers.
- [x] `test_utils/` - Mock pools and transaction generators for tests.
- [x] `validate/` - Transaction validation APIs and executors.

### Files
- `lib.rs` - Crate entrypoint and public pool type exports.
  - **Key items**: `Pool`, `EthTransactionPool`
- `batcher.rs` - Batch insertion processor to reduce lock contention.
  - **Key items**: `BatchTxRequest`, `BatchTxProcessor`
- `config.rs` - Pool limits and fee configuration.
  - **Key items**: `PoolConfig`, `SubPoolLimit`, `PriceBumpConfig`, `LocalTransactionConfig`
- `error.rs` - Pool error types and validation error variants.
  - **Key items**: `PoolError`, `PoolErrorKind`, `InvalidPoolTransactionError`
- `identifier.rs` - Sender/transaction identifiers for indexing.
  - **Key items**: `SenderIdentifiers`, `SenderId`, `TransactionId`
- `maintain.rs` - Pool maintenance loop for canonical state updates and backups.
  - **Key items**: `MaintainPoolConfig`, `maintain_transaction_pool_future()`, `TxBackup`
- `metrics.rs` - Pool metrics structs for pool/validation/maintenance.
  - **Key items**: `TxPoolMetrics`, `MaintainPoolMetrics`, `TxPoolValidatorMetrics`
- `noop.rs` - No-op pool implementation for wiring/testing.
  - **Key items**: `NoopTransactionPool`, `NoopInsertError`
- `ordering.rs` - Transaction ordering traits and default ordering.
  - **Key items**: `TransactionOrdering`, `Priority`, `CoinbaseTipOrdering`
- `traits.rs` - Core pool and transaction trait definitions.
  - **Key items**: `TransactionPool`, `PoolTransaction`, `EthPoolTransaction`

## Key APIs (no snippets)
- `Pool`, `TransactionPool`, `PoolTransaction`
- `TransactionOrdering`, `PoolConfig`, `PoolError`
```

## File: crates/transaction-pool/tests/it/AGENTS.md
```markdown
# it

## Purpose
Integration tests for transaction pool behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring and feature gating.
- **Key items**: `blobs`, `evict`, `listeners`, `pending`, `best`

### `best.rs`
- **Role**: Sanity checks for best-transaction iteration.
- **Key items**: `test_best_transactions()`

### `blobs.rs`
- **Role**: Blob transaction exclusivity tests.
- **Key items**: `blobs_exclusive()`

### `evict.rs`
- **Role**: Pool eviction behavior under size limits.
- **Key items**: `only_blobs_eviction()`, `mixed_eviction()`

### `listeners.rs`
- **Role**: Listener and event stream behavior tests.
- **Key items**: `txpool_listener_by_hash()`, `txpool_listener_replace_event()`

### `pending.rs`
- **Role**: Pending transaction stream tests.
- **Key items**: `txpool_new_pending_txs()`

## End-to-end flow (high level)
- Insert mock transactions and assert subpool transitions.
- Verify listener event streams for pending/queued/replaced/discarded events.
- Validate eviction behavior with varied pool limits.

## Key APIs (no snippets)
- `TransactionPool`, `pending_transactions_listener()`
```

## File: crates/transaction-pool/tests/AGENTS.md
```markdown
# tests

## Purpose
Transaction pool integration tests.

## Contents (one hop)
### Subdirectories
- [x] `it/` - Integration tests for pool behavior and listeners.

### Files
- (none)
```

## File: crates/transaction-pool/AGENTS.md
```markdown
# transaction-pool

## Purpose
`reth-transaction-pool` crate: mempool implementation, validation, ordering, and maintenance.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Transaction pool performance benchmarks.
- [x] `docs/` - (skip: mermaid diagram assets).
- [x] `src/` - Pool implementation, validation, and maintenance.
- [x] `test_data/` - (skip: test fixture JSON data).
- [x] `tests/` - Integration tests for pool behavior.

### Files
- `Cargo.toml` - Manifest with pool features, benches, and test utils.
  - **Key items**: features `serde`, `test-utils`, `arbitrary`
```

## File: crates/trie/common/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for trie common components (hashed state and prefix set performance).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `hashed_state.rs`
- **Role**: Benchmarks parallel construction of `HashedPostState` with alternative iterator strategies.
- **Key items**: `generate_test_data()`, `keccak256_u64()`, `from_par_iter_fold_reduce()`, `from_par_iter_collect_twice()`, `bench_from_parallel_iterator()`
- **Interactions**: Exercises `HashedPostState::from_par_iter` under the `rayon` feature.

### `prefix_set.rs`
- **Role**: Benchmarks prefix-set lookup implementations across varying set sizes.
- **Key items**: `PrefixSetMutAbstraction`, `PrefixSetAbstraction`, `prefix_set_lookups()`, `generate_test_data()`, `prefix_set_bench()`
- **Interactions**: Compares BTreeSet-backed and Vec/cursor-backed prefix set variants.

## End-to-end flow (high level)
- Generate randomized test data at multiple sizes.
- Run Criterion benchmarks for each implementation.
- Compare throughput across strategies for hashed state building and prefix lookups.
```

## File: crates/trie/common/src/hash_builder/AGENTS.md
```markdown
# hash_builder

## Purpose
Persistable hash-builder utilities that capture and restore trie hash builder state for reuse.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for hash builder helpers.
- **Key items**: `HashBuilderState`, `alloy_trie::hash_builder::*`
- **Interactions**: Re-exported by `src/lib.rs` for crate consumers.

### `state.rs`
- **Role**: Serializable snapshot of `HashBuilder` internals for storing/resuming trie computation.
- **Key items**: `HashBuilderState`, `From<HashBuilderState> for HashBuilder`, `From<HashBuilder> for HashBuilderState`, `Compact` impl
- **Interactions**: Uses `TrieMask`, `RlpNode`, `HashBuilderValue` and `reth_codecs` for compact encoding.
- **Knobs / invariants**: Preserves stack/mask ordering; `stored_in_database` flag is round-tripped in compact form.

## End-to-end flow (high level)
- Build a `HashBuilder` while computing trie updates.
- Convert to `HashBuilderState` to snapshot the builder state.
- Encode/decode via `reth_codecs::Compact` when persisting or loading.
- Rehydrate into `HashBuilder` to resume hashing or proof generation.
```

## File: crates/trie/common/src/AGENTS.md
```markdown
# src

## Purpose
Shared trie data structures and helpers: hashed state overlays, prefix sets, proofs, update buffers, and root computation utilities.

## Contents (one hop)
### Subdirectories
- [x] `hash_builder/` - Hash builder state snapshots and compact encoding helpers.

### Files
- `account.rs` - Re-exports the trie account type and validates conversions from genesis/account inputs.
  - **Key items**: `TrieAccount`, `GenesisAccount`, `Account`
  - **Interactions**: Tests call `root::storage_root_unhashed` for storage-root derivation.
- `added_removed_keys.rs` - Tracks added/removed keys across account and storage tries.
  - **Key items**: `MultiAddedRemovedKeys`, `update_with_state()`, `touch_accounts()`, `AddedRemovedKeys`
  - **Interactions**: Consumes `HashedPostState` updates to update removal sets.
- `constants.rs` - Defines trie size constants for RLP encoding.
  - **Key items**: `TRIE_ACCOUNT_RLP_MAX_SIZE`, `account_rlp_max_size()`
- `hashed_state.rs` - In-memory hashed state overlays, sorted forms, and chunking utilities.
  - **Key items**: `HashedPostState`, `HashedStorage`, `HashedPostStateSorted`, `HashedStorageSorted`, `ChunkedHashedPostState`
  - **Interactions**: Builds `TriePrefixSetsMut`, consumes `MultiProofTargets`, and uses `MultiAddedRemovedKeys`.
  - **Knobs / invariants**: `wiped` flag semantics; chunking preserves wipe -> storage updates -> account ordering.
- `input.rs` - Aggregates trie updates, hashed state, and prefix sets into computation inputs.
  - **Key items**: `TrieInput`, `TrieInputSorted`, `from_state()`, `from_blocks()`, `append_cached_ref()`
  - **Interactions**: Composes `TrieUpdates`, `HashedPostState`, and `TriePrefixSetsMut`.
- `key.rs` - Key hashing trait and Keccak implementation.
  - **Key items**: `KeyHasher`, `KeccakKeyHasher`, `hash_key()`
- `lib.rs` - Crate entrypoint with module wiring and re-exports.
  - **Key items**: `TrieInput`, `TrieAccount`, `KeccakKeyHasher`, `BranchNodeCompact`, `HashBuilder`, `TrieMask`
  - **Interactions**: Re-exports alloy-trie primitives for downstream crates.
- `nibbles.rs` - Stored nibble wrappers and compact encoding helpers.
  - **Key items**: `Nibbles`, `StoredNibbles`, `StoredNibblesSubKey`, `Compact` impls
- `prefix_set.rs` - Mutable and immutable prefix-set containers for trie invalidation.
  - **Key items**: `TriePrefixSetsMut`, `TriePrefixSets`, `PrefixSetMut`, `PrefixSet`, `freeze()`
  - **Knobs / invariants**: `PrefixSet::contains()` maintains a cursor for sequential lookups; `all` flag overrides keys.
- `proofs.rs` - Merkle proof and multiproof structures with verification helpers.
  - **Key items**: `MultiProofTargets`, `MultiProof`, `DecodedMultiProof`, `AccountProof`, `StorageProof`, `StorageMultiProof`
  - **Interactions**: Uses `BranchNodeMasksMap`, `TrieAccount`, and `verify_proof` from `alloy_trie`.
- `root.rs` - Re-exports root computation helpers from `alloy_trie`.
  - **Key items**: `state_root`, `state_root_unhashed`, `state_root_unsorted`, `storage_root`, `storage_root_unhashed`, `storage_root_unsorted`
- `storage.rs` - Storage trie table entries and changeset value wrappers.
  - **Key items**: `StorageTrieEntry`, `TrieChangeSetsEntry`, `StoredNibblesSubKey`, `BranchNodeCompact`
  - **Interactions**: Implements `ValueWithSubKey` for DB changeset usage.
- `subnode.rs` - Stored subnode representation for trie walker state.
  - **Key items**: `StoredSubNode`, `key`, `nibble`, `node`
  - **Interactions**: Uses `BranchNodeCompact` for persisted node snapshots.
- `trie.rs` - Sparse trie mask types and proof node descriptors.
  - **Key items**: `BranchNodeMasks`, `BranchNodeMasksMap`, `ProofTrieNode`
- `updates.rs` - Trie update buffers and sorted variants, including batch merge utilities.
  - **Key items**: `TrieUpdates`, `StorageTrieUpdates`, `TrieUpdatesSorted`, `StorageTrieUpdatesSorted`, `merge_batch()`, `finalize()`
  - **Interactions**: Consumes `HashBuilder` output and compacts changes for storage tries.
  - **Knobs / invariants**: `is_deleted` wipes prior storage nodes; empty-nibble paths are filtered.
- `utils.rs` - Sorted merge helpers used by hashed state and update buffers.
  - **Key items**: `kway_merge_sorted()`, `extend_sorted_vec()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `HashedPostState`, `TrieUpdates`, `MultiProof`, `TrieInput`, `TriePrefixSetsMut` - core data carriers for trie computation and proofs.
- **Modules / Packages**: `hash_builder`, `prefix_set`, `updates` - state snapshots, invalidation sets, and update buffers.
- **Functions**: `state_root()`, `storage_root_unhashed()`, `kway_merge_sorted()`, `extend_sorted_vec()` - root helpers and sorted-merge utilities.

## Relationships
- **Depends on**: `alloy-trie`, `alloy-primitives`, `reth-primitives-traits` - trie node encoding, hash types, and account traits.
- **Data/control flow**: `HashedPostState` + `TrieUpdates` + `TriePrefixSetsMut` compose a `TrieInput`.
- **Data/control flow**: `TrieInput`/`TrieInputSorted` feed root/proof computation in higher-level trie crates.
- **Data/control flow**: `MultiProofTargets` -> `MultiProof`/`AccountProof` -> `verify()` for proof validation.
- **Data/control flow**: `HashBuilder`/`HashBuilderState` snapshots feed `TrieUpdates::finalize()`.
```

## File: crates/trie/common/AGENTS.md
```markdown
# common

## Purpose
`reth-trie-common` crate with shared trie types, hashing utilities, proofs, and update buffers used across trie implementations.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Criterion benchmarks for hashed state construction and prefix set lookups.
- [x] `src/` - Core trie common types: hashed state overlays, prefix sets, proofs, updates, and root helpers.

### Files
- `Cargo.toml` - Crate manifest defining features, dependencies, and bench setup.
  - **Key items**: features `serde`, `serde-bincode-compat`, `eip1186`, `reth-codec`, `rayon`, `test-utils`; benches `prefix_set`, `hashed_state`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `TrieInput`, `HashedPostState`, `TrieUpdates`, `MultiProof`, `PrefixSetMut` - shared primitives for trie computation and proof handling.
- **Modules / Packages**: `hash_builder`, `updates`, `proofs` - state snapshots, update buffers, and proof helpers.
- **Functions**: `state_root()`, `storage_root()` - root helpers re-exported from `alloy_trie`.

## Relationships
- **Depends on**: `alloy-trie`, `alloy-primitives`, `alloy-rlp` - trie node/proof encoding and hash types.
- **Depends on**: `reth-primitives-traits`, `revm-database` - account types and bundle state hashing inputs.
- **Data/control flow**: `TrieInput` aggregates `HashedPostState`, `TrieUpdates`, and prefix sets for higher-level trie computations.
```

## File: crates/trie/db/src/AGENTS.md
```markdown
# src

## Purpose
Database-backed trie integrations: cursor factories, prefix set loading, root computation, proofs, witnesses, and changeset caching.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `changesets.rs`
- **Role**: Computes trie changesets/updates for blocks and caches them for reorg-safe lookups.
- **Key items**: `compute_block_trie_changesets()`, `compute_block_trie_updates()`, `ChangesetCache`, `get_or_compute_range()`, `evict()`
- **Interactions**: Uses `DatabaseTrieCursorFactory`, `InMemoryTrieCursorFactory`, and `StateRoot::overlay_root_from_nodes_with_updates`.
- **Knobs / invariants**: Cache eviction is explicit; range accumulation is newest-to-oldest so older values win.

### `hashed_cursor.rs`
- **Role**: Database cursor wrappers implementing hashed account/storage cursor traits.
- **Key items**: `DatabaseHashedCursorFactory`, `DatabaseHashedAccountCursor`, `DatabaseHashedStorageCursor`, `is_storage_empty()`, `set_hashed_address()`
- **Interactions**: Reads `HashedAccounts`/`HashedStorages` tables via `reth_db_api` cursors.

### `lib.rs`
- **Role**: Crate entrypoint wiring and re-exports for DB integration traits and cursor factories.
- **Key items**: `DatabaseStateRoot`, `DatabaseHashedPostState`, `DatabaseStorageRoot`, `DatabaseProof`, `DatabaseTrieCursorFactory`, `DatabaseTrieWitness`

### `prefix_set.rs`
- **Role**: Loads account/storage prefix sets from changesets over block ranges.
- **Key items**: `PrefixSetLoader`, `load()`, `load_prefix_sets_with_provider()`, `TriePrefixSets`, `PrefixSetMut`
- **Interactions**: Reads `AccountChangeSets`/`StorageChangeSets` and hashes keys via `KeyHasher`.

### `proof.rs`
- **Role**: Database-specific proof builders that overlay trie input on top of DB state.
- **Key items**: `DatabaseProof`, `DatabaseStorageProof`, `overlay_account_proof()`, `overlay_multiproof()`, `overlay_storage_proof()`, `overlay_storage_multiproof()`
- **Interactions**: Uses `Proof`, `StorageProof`, and `InMemoryTrieCursorFactory` with `TrieInput`.

### `state.rs`
- **Role**: Database-backed state root computation and hashed post-state reverts ingestion.
- **Key items**: `DatabaseStateRoot`, `DatabaseHashedPostState`, `incremental_root()`, `overlay_root()`, `overlay_root_from_nodes_with_updates()`, `from_reverts()`
- **Interactions**: Uses `load_prefix_sets_with_provider`, `HashedPostStateSorted`, and `TrieInputSorted`.
- **Knobs / invariants**: `from_reverts` keeps first occurrence per key and respects range bounds.

### `storage.rs`
- **Role**: Database-backed storage root computation and hashed storage reverts.
- **Key items**: `DatabaseStorageRoot`, `DatabaseHashedStorage`, `overlay_root()`, `from_reverts()`
- **Interactions**: Uses `StorageRoot` with DB cursor factories and `StorageChangeSets` tables.

### `trie_cursor.rs`
- **Role**: Database trie cursor factories for account/storage tries, plus sorted update writes.
- **Key items**: `DatabaseTrieCursorFactory`, `DatabaseAccountTrieCursor`, `DatabaseStorageTrieCursor`, `write_storage_trie_updates_sorted()`
- **Interactions**: Uses `StorageTrieEntry`, `StoredNibbles`, and `StoredNibblesSubKey` for table IO.
- **Knobs / invariants**: `write_storage_trie_updates_sorted` deletes duplicates when `is_deleted` and skips empty nibbles.

### `witness.rs`
- **Role**: Database-specific trie witness generation for a target state overlay.
- **Key items**: `DatabaseTrieWitness`, `overlay_witness()`, `TrieWitness`
- **Interactions**: Builds overlay factories with `TrieInput` and `HashedPostStateCursorFactory`.

## End-to-end flow (high level)
- Load prefix sets from changesets for a block range.
- Build `HashedPostStateSorted` from reverts via `DatabaseHashedPostState`.
- Instantiate `StateRoot`/`StorageRoot` with DB cursor factories and compute roots or updates.
- Overlay cached nodes with `TrieInputSorted` for incremental roots or proof generation.
- Produce account/storage proofs and multiproofs from DB-backed cursors.
- Generate witness maps for target hashed state overlays.
- Cache per-block trie changesets for reorg-safe lookups and explicit eviction.
```

## File: crates/trie/db/tests/AGENTS.md
```markdown
# tests

## Purpose
Integration and property tests for database-backed trie cursors, roots, proofs, walkers, and witnesses.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `fuzz_in_memory_nodes.rs`
- **Role**: Property tests comparing in-memory overlay roots against expected roots for accounts and storage.
- **Key items**: `fuzz_in_memory_account_nodes`, `fuzz_in_memory_storage_nodes`, `InMemoryTrieCursorFactory`, `StateRoot::from_tx`, `StorageRoot::from_tx_hashed`
- **Interactions**: Extends `TrieUpdates` with in-memory nodes and validates expected roots.

### `post_state.rs`
- **Role**: Validates hashed cursor ordering and precedence between DB and post-state overlays.
- **Key items**: `assert_account_cursor_order`, `assert_storage_cursor_order`, `storage_is_empty`, `storage_cursor_correct_order`, `zero_value_storage_entries_are_discarded`, `wiped_storage_is_discarded`
- **Interactions**: Uses `HashedPostStateCursorFactory` with `DatabaseHashedCursorFactory`.

### `proof.rs`
- **Role**: Verifies account and storage proofs against known genesis/testspec fixtures.
- **Key items**: `testspec_proofs`, `testspec_empty_storage_proof`, `mainnet_genesis_account_proof`, `holesky_deposit_contract_proof`, `DatabaseProof`
- **Interactions**: Compares computed proofs to expected byte sequences and calls `verify()`.

### `trie.rs`
- **Role**: End-to-end root computation tests (full, incremental, extension-node scenarios).
- **Key items**: `incremental_vs_full_root`, `arbitrary_state_root`, `arbitrary_state_root_with_progress`, `account_and_storage_trie`, `storage_root_regression`, `fuzz_state_root_incremental`
- **Interactions**: Exercises `StateRoot`, `StorageRoot`, `TrieUpdates`, and `TriePrefixSets`.

### `walker.rs`
- **Role**: Tests trie walker traversal ordering and prefix-set behavior for account/storage tries.
- **Key items**: `walk_nodes_with_common_prefix`, `cursor_rootnode_with_changesets`, `TrieWalker`, `PrefixSetMut`
- **Interactions**: Uses `DatabaseAccountTrieCursor` and `DatabaseStorageTrieCursor`.

### `witness.rs`
- **Role**: Validates trie witness generation for empty roots and destroyed storage.
- **Key items**: `includes_empty_node_preimage`, `includes_nodes_for_destroyed_storage_nodes`, `correctly_decodes_branch_node_values`, `DatabaseTrieWitness`
- **Interactions**: Cross-checks witness contents against `Proof` multiproofs.

## End-to-end flow (high level)
- Build test databases/providers and seed hashed accounts/storage.
- Compute roots/updates with prefix sets and compare against expected roots.
- Generate proofs and witnesses from DB-backed factories.
- Assert ordering, deletion, and witness completeness invariants.
- Use proptest fuzzing to validate cursor/overlay behavior.
```

## File: crates/trie/db/AGENTS.md
```markdown
# db

## Purpose
`reth-trie-db` crate: database-backed trie integration for roots, proofs, witnesses, and changeset caching.

## Contents (one hop)
### Subdirectories
- [x] `src/` - DB cursor factories, prefix set loading, roots/proofs/witnesses, and changeset cache.
- [x] `tests/` - Integration and fuzz tests for cursors, roots, proofs, walkers, and witnesses.

### Files
- `Cargo.toml` - Manifest for DB integration dependencies and optional features.
  - **Key items**: features `metrics`, `serde`, `test-utils`; deps `reth-trie`, `reth-trie-common`, `reth-db-api`, `reth-storage-api`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseStateRoot`, `DatabaseStorageRoot`, `DatabaseProof`, `DatabaseTrieWitness` - DB adapters for root/proof computation.
- **Modules / Packages**: `changesets`, `trie_cursor`, `hashed_cursor` - cache, trie cursors, and hashed state access.
- **Functions**: `compute_block_trie_changesets()`, `compute_block_trie_updates()` - changeset calculation utilities.

## Relationships
- **Depends on**: `reth-trie`, `reth-trie-common`, `reth-db-api`, `reth-storage-api` - trie logic and DB access layers.
- **Data/control flow**: changesets -> prefix sets -> incremental roots/proofs/witnesses using DB cursor factories.
```

## File: crates/trie/parallel/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks comparing single-threaded and parallel state root computation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `root.rs`
- **Role**: Benchmarks state root calculation on synthetic data sets with varying sizes.
- **Key items**: `calculate_state_root()`, `generate_test_data()`, `ParallelStateRoot`, `StateRoot::root()`, `TrieInput::from_state()`
- **Interactions**: Compares sequential `StateRoot` vs `ParallelStateRoot` over overlay providers.

## End-to-end flow (high level)
- Generate randomized hashed account/storage state for a target size.
- Seed DB state and write trie updates.
- Benchmark sequential state root calculation.
- Benchmark parallel state root calculation using `ParallelStateRoot`.
```

## File: crates/trie/parallel/src/AGENTS.md
```markdown
# src

## Purpose
Parallel state root and proof computation with worker pools, metrics, and stats tracking.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint exposing parallel root/proof modules and helpers.
- **Key items**: `StorageRootTargets`, `stats`, `root`, `proof`, `proof_task`, `metrics` (feature)
- **Interactions**: Gated exports for metrics and proof-task metrics.

### `metrics.rs`
- **Role**: Metrics for parallel state root computation.
- **Key items**: `ParallelStateRootMetrics`, `ParallelTrieMetrics`, `record_state_trie()`
- **Interactions**: Wraps `TrieRootMetrics` and consumes `ParallelTrieStats`.

### `proof_task_metrics.rs`
- **Role**: Metrics for proof-task workers and cursor operations.
- **Key items**: `ProofTaskTrieMetrics`, `ProofTaskCursorMetrics`, `ProofTaskCursorMetricsCache`, `record()`, `record_spans()`
- **Interactions**: Uses `TrieCursorMetrics` and `HashedCursorMetrics` caches from `reth_trie`.

### `proof_task.rs`
- **Role**: Worker-pool execution for parallel multiproof and storage proof computation.
- **Key items**: `ProofWorkerHandle`, `ProofResultMessage`, `ProofResultContext`, `StorageProofInput`, `AccountMultiproofInput`, `StorageProofWorker`, `AccountProofWorker`
- **Interactions**: Uses `TrieWalker`, `HashBuilder`, blinded node providers, and `MultiAddedRemovedKeys` while coordinating storage proofs.
- **Knobs / invariants**: `v2_proofs_enabled`, worker counts, and `collect_branch_node_masks` shape results; workers own dedicated DB transactions.

### `proof.rs`
- **Role**: High-level parallel proof orchestrator built on worker pools.
- **Key items**: `ParallelProof`, `decoded_multiproof()`, `storage_proof()`, `extend_prefix_sets_with_targets()`, `with_v2_proofs_enabled()`
- **Interactions**: Dispatches tasks via `ProofWorkerHandle` and aggregates `DecodedMultiProof` results.

### `root.rs`
- **Role**: Parallel incremental state root calculator with optional trie updates.
- **Key items**: `ParallelStateRoot`, `incremental_root()`, `incremental_root_with_updates()`, `ParallelStateRootError`, `get_runtime_handle()`
- **Interactions**: Spawns blocking storage-root tasks and walks the account trie to build the root.
- **Knobs / invariants**: `retain_updates` toggles update collection; errors if storage root returns progress.

### `stats.rs`
- **Role**: Statistics tracking for parallel trie computations.
- **Key items**: `ParallelTrieStats`, `ParallelTrieTracker`, `inc_branch()`, `inc_leaf()`, `inc_missed_leaves()`, `finish()`
- **Interactions**: Wraps `TrieTracker` stats and optionally collects cursor metrics.

### `storage_root_targets.rs`
- **Role**: Aggregates storage root targets and prefix sets for parallel processing.
- **Key items**: `StorageRootTargets`, `new()`, `count()`, `IntoParallelIterator` impl
- **Interactions**: Combines account prefix sets with storage prefix sets and drives parallel iteration.

## End-to-end flow (high level)
- Build prefix sets and `StorageRootTargets` from state changes.
- Spawn worker pools for account and storage proofs via `ProofWorkerHandle`.
- Dispatch storage proof/root tasks in parallel, tracking availability and metrics.
- Walk the account trie and integrate storage roots into `HashBuilder`.
- Produce parallel roots or multiproofs, recording stats and optional metrics.
```

## File: crates/trie/parallel/AGENTS.md
```markdown
# parallel

## Purpose
`reth-trie-parallel` crate: parallel state root and proof computation using worker pools.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks comparing sequential and parallel root computation.
- [x] `src/` - Parallel root/proof implementations, worker pools, stats, and metrics.

### Files
- `Cargo.toml` - Manifest for parallel trie computation and optional metrics.
  - **Key items**: features `metrics`, `test-utils`; deps `reth-trie`, `reth-trie-common`, `reth-trie-sparse`, `rayon`, `tokio`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelStateRoot`, `ParallelProof`, `ProofWorkerHandle`, `StorageRootTargets`
- **Modules / Packages**: `root`, `proof`, `proof_task`, `stats`
- **Functions**: `incremental_root()`, `decoded_multiproof()` - parallel root/proof entrypoints.

## Relationships
- **Depends on**: `reth-trie` and `reth-trie-sparse` for trie traversal, proofs, and sparse node access.
- **Data/control flow**: storage roots computed in parallel feed account trie traversal to finalize state root.
```

## File: crates/trie/sparse/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for sparse trie operations and root computation performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `rlp_node.rs`
- **Role**: Benchmarks `SerialSparseTrie::update_rlp_node_level` across trie depths and update
  percentages after initial population.
- **Key items**: `update_rlp_node_level()`, `Criterion`, `TestRunner`, `DefaultTrieNodeProvider`,
  `SerialSparseTrie`
- **Interactions**: Uses `SerialSparseTrie` update and root APIs from the sparse trie crate.
- **Knobs / invariants**: Fixed dataset size (100_000), sampled update percentages, per-depth
  iteration.

### `root.rs`
- **Role**: Compares root computation using `HashBuilder` versus `SparseTrie`, and measures
  repeated update scenarios.
- **Key items**: `calculate_root_from_leaves()`, `calculate_root_from_leaves_repeated()`,
  `generate_test_data()`, `SparseTrie::<SerialSparseTrie>`, `HashBuilder`, `TrieNodeIter`
- **Interactions**: Uses `reth_trie` walkers and cursors for the hash-builder baseline.
- **Knobs / invariants**: Dataset sizes and update sizes vary; some runs are skipped under
  `codspeed` to avoid long benchmarks.

## End-to-end flow (high level)
- Generate randomized account/storage inputs.
- Populate sparse trie or hash builder and compute initial roots.
- Apply batches of updates and recompute roots.
- Record timings for each configuration.
```

## File: crates/trie/sparse/src/AGENTS.md
```markdown
# src

## Purpose
Core sparse trie implementation and state-trie wrapper used for lazy node reveal, update tracking,
and root computation. Defines provider traits, sparse trie data structures, and optional metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint that wires modules, re-exports main types, and exposes sparse trie
  error types.
- **Key items**: `state`, `trie`, `traits`, `provider`, `errors`, `SparseTrieErrorKind`
- **Interactions**: Re-exports public APIs defined in `state.rs`, `trie.rs`, and `traits.rs`.

### `metrics.rs`
- **Role**: Metrics collection for sparse state trie multiproof reveals.
- **Key items**: `SparseStateTrieMetrics`, `SparseStateTrieInnerMetrics`, `record()`,
  `increment_*_nodes()`
- **Interactions**: Used by `SparseStateTrie` when the `metrics` feature is enabled.
- **Knobs / invariants**: Counters are accumulated and recorded into histograms via `record()`.

### `provider.rs`
- **Role**: Traits and default implementations for retrieving blinded trie nodes on demand.
- **Key items**: `TrieNodeProviderFactory`, `TrieNodeProvider`, `RevealedNode`,
  `DefaultTrieNodeProviderFactory`, `DefaultTrieNodeProvider`, `pad_path_to_key()`
- **Interactions**: Providers are used by `SparseTrieInterface` operations in `trie.rs` and
  reveal helpers in `state.rs`.

### `state.rs`
- **Role**: Sparse state trie that combines account and storage sparse tries with multiproof and
  witness reveal helpers.
- **Key items**: `SparseStateTrie`, `ClearedSparseStateTrie`, `reveal_multiproof()`,
  `reveal_decoded_multiproof()`, `reveal_witness()`, `root()`, `root_with_updates()`,
  `update_account_leaf()`, `update_storage_leaf()`, `update_account()`, `update_account_storage_root()`,
  `wipe_storage()`, `storage_root()`
- **Interactions**: Uses `SparseTrie` from `trie.rs`, `TrieNodeProviderFactory` from `provider.rs`,
  and `TrieAccount`/`MultiProof` types from `reth-trie-common`.
- **Knobs / invariants**: `retain_updates` controls update tracking; storage tries and revealed
  path sets are pooled for allocation reuse.

### `traits.rs`
- **Role**: Defines the sparse trie interface and shared update/result types.
- **Key items**: `SparseTrieInterface`, `SparseTrieUpdates`, `LeafLookup`, `LeafLookupError`
- **Interactions**: Implemented by `SerialSparseTrie` in `trie.rs` and used by `SparseTrie` wrapper.

### `trie.rs`
- **Role**: Core sparse trie implementation, node types, and RLP/hash update logic.
- **Key items**: `SparseTrie`, `SerialSparseTrie`, `SparseNode`, `SparseNodeType`,
  `RlpNodeBuffers`, `root_with_updates()`, `update_leaf()`, `remove_leaf()`, `find_leaf()`,
  `update_rlp_node_level()`, `rlp_node()`
- **Interactions**: Pulls blinded nodes via `TrieNodeProvider`, uses `PrefixSet` and
  `BranchNodeMasks` from `reth-trie-common`, emits `SparseTrieUpdates`.
- **Knobs / invariants**: Root node is always present; leaf values are stored in a separate map;
  `SPARSE_TRIE_SUBTRIE_HASHES_LEVEL` controls subtrie hash refresh depth.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `SparseTrie`, `SerialSparseTrie`, `SparseStateTrie`,
  `SparseTrieInterface`, `SparseTrieUpdates`, `TrieNodeProvider`, `TrieNodeProviderFactory`,
  `RevealedNode`
- **Modules / Packages**: `state`, `trie`, `traits`, `provider`, `metrics`
- **Functions**: `reveal_multiproof()`, `reveal_witness()`, `root_with_updates()`,
  `update_leaf()`, `remove_leaf()`, `update_rlp_node_level()`, `pad_path_to_key()`

## Relationships
- **Depends on**: `reth-trie-common` for node types, masks, and proof nodes.
- **Depends on**: `alloy-trie` and `alloy-rlp` for decoding and RLP encoding.
- **Depends on**: `reth-execution-errors` for sparse trie error types.
- **Data/control flow**: `MultiProof`/witness data is decoded into `ProofTrieNode`s and revealed
  into `SparseTrie` instances.
- **Data/control flow**: Leaf updates/removals use `TrieNodeProvider` to fetch blinded nodes
  and update `PrefixSet` for dirty hashing.
- **Data/control flow**: `SerialSparseTrie` recomputes RLP nodes and hashes, optionally capturing
  `SparseTrieUpdates` for DB persistence.
- **Data/control flow**: `SparseStateTrie` merges account and storage trie roots into account RLP,
  updates account leaves, and returns root and updates.

## End-to-end flow (high level)
- Initialize `SparseStateTrie` or `SparseTrie` in blind or revealed mode.
- Reveal account/storage paths using multiproofs or witness data.
- Apply account and storage leaf updates or removals, resolving blinded nodes via providers.
- Recompute subtrie hashes and root RLP nodes, optionally collecting update sets.
- Return root hashes (and updates) for persistence; clear or reuse tries for the next run.
```

## File: crates/trie/sparse/AGENTS.md
```markdown
# sparse

## Purpose
`reth-trie-sparse` crate: sparse MPT implementation with lazy node reveal, account/storage state
trie wrapper, update tracking, and optional metrics.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for RLP node updates and root calculation.
- [x] `src/` - Core sparse trie types, state trie wrapper, providers, and metrics.

### Files
- `Cargo.toml` - Crate manifest for sparse trie implementation and feature flags.
  - **Key items**: features `std`, `metrics`, `test-utils`, `arbitrary`; benches `root`, `rlp_node`;
    deps `reth-trie-common`, `alloy-trie`, `alloy-rlp`, `smallvec`, optional `rayon`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `SparseTrie`, `SerialSparseTrie`, `SparseStateTrie`,
  `SparseTrieInterface`, `TrieNodeProvider`, `TrieNodeProviderFactory`, `SparseTrieUpdates`
- **Modules / Packages**: `state`, `trie`, `traits`, `provider`
- **Functions**: `reveal_multiproof()`, `reveal_witness()`, `root_with_updates()`,
  `update_leaf()`, `remove_leaf()`

## Relationships
- **Depends on**: `reth-trie-common`, `alloy-trie`, `alloy-rlp` for trie node encoding/decoding.
- **Depends on**: `reth-execution-errors` for sparse trie error types.
- **Data/control flow**: multiproof and witness data is decoded, revealed into sparse tries, and
  updated leaves yield new roots and update sets for persistence.
```

## File: crates/trie/sparse-parallel/src/AGENTS.md
```markdown
# src

## Purpose
Parallel sparse trie implementation that splits the trie into upper and lower subtries to
parallelize reveals and hash updates, with update tracking and optional metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint that re-exports the parallel sparse trie and wires internal modules.
- **Key items**: `trie`, `lower`, `metrics` (feature gated)
- **Interactions**: Re-exports `ParallelSparseTrie` and related types from `trie.rs`.

### `lower.rs`
- **Role**: Manages the lower subtrie lifecycle (blind vs revealed) and allocation reuse.
- **Key items**: `LowerSparseSubtrie`, `reveal()`, `clear()`, `take_revealed()`,
  `take_revealed_if()`, `shrink_nodes_to()`, `shrink_values_to()`
- **Interactions**: Used by `ParallelSparseTrie` to store and recycle lower subtries.
- **Knobs / invariants**: `reveal()` may shorten the stored root path; `clear()` preserves a
  cleared subtrie for reuse.

### `metrics.rs`
- **Role**: Metrics collection for parallel sparse trie hash updates.
- **Key items**: `ParallelSparseTrieMetrics`, `subtries_updated`, `subtrie_hash_update_latency`,
  `subtrie_upper_hash_latency`
- **Interactions**: Used by `ParallelSparseTrie` when the `metrics` feature is enabled.

### `trie.rs`
- **Role**: Core parallel sparse trie implementation, including subtrie partitioning, leaf
  updates, parallel reveal and hashing, and update tracking.
- **Key items**: `ParallelSparseTrie`, `ParallelismThresholds`, `UPPER_TRIE_MAX_DEPTH`,
  `NUM_LOWER_SUBTRIES`, `SparseSubtrie`, `SparseSubtrieType`, `LeafUpdateStep`,
  `SparseSubtrieBuffers`, `SparseTrieUpdatesAction`, `RlpNodePathStackItem`
- **Interactions**: Uses `SparseNode`/`SparseTrieInterface` from `reth-trie-sparse`,
  `PrefixSet` and `BranchNodeMasks` from `reth-trie-common`, and `TrieNodeProvider`
  for revealing blinded nodes.
- **Knobs / invariants**: `ParallelismThresholds` gates parallel execution; paths shorter than
  `UPPER_TRIE_MAX_DEPTH` live in the upper subtrie; prefix sets drive incremental hashing.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelSparseTrie`, `ParallelismThresholds`,
  `SparseSubtrie`, `SparseSubtrieType`, `LowerSparseSubtrie`
- **Modules / Packages**: `trie`, `lower`, `metrics`
- **Functions**: `with_parallelism_thresholds()`, `reveal_nodes()`, `update_leaf()`,
  `remove_leaf()`, `root()`

## Relationships
- **Depends on**: `reth-trie-sparse` for sparse node types and shared interfaces.
- **Depends on**: `reth-trie-common` for masks, RLP nodes, and prefix sets.
- **Depends on**: `rayon` (optional) to parallelize reveal and hash updates under `std`.
- **Data/control flow**: proof nodes are partitioned into upper/lower subtries and revealed,
  then updates are applied to subtrie maps and prefix sets.
- **Data/control flow**: hash updates run bottom-up on lower subtries (optionally in parallel),
  then the upper subtrie hash is updated to produce the final root.
- **Data/control flow**: update actions emitted by subtries are folded into `SparseTrieUpdates`.

## End-to-end flow (high level)
- Initialize `ParallelSparseTrie` (empty or from a root node) and configure thresholds.
- Reveal proof nodes, updating branch masks and routing nodes to upper/lower subtries.
- Apply leaf updates or removals, revealing blinded nodes via providers as needed.
- Recompute lower subtrie hashes (parallel when thresholds are met), then update upper hashes.
- Return the root hash and optional update sets for persistence.
```

## File: crates/trie/sparse-parallel/AGENTS.md
```markdown
# sparse-parallel

## Purpose
`reth-trie-sparse-parallel` crate: parallel sparse MPT implementation that splits the trie into
upper and lower subtries for concurrent reveal and hash updates.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Parallel sparse trie implementation, lower subtrie management, and metrics.

### Files
- `Cargo.toml` - Crate manifest for parallel sparse trie implementation and features.
  - **Key items**: features `std`, `metrics`; deps `reth-trie-sparse`, `reth-trie-common`,
    optional `rayon`, `metrics`/`reth-metrics`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelSparseTrie`, `ParallelismThresholds`,
  `LowerSparseSubtrie`, `SparseSubtrie`
- **Modules / Packages**: `trie`, `lower`, `metrics`
- **Functions**: `with_parallelism_thresholds()`, `reveal_nodes()`, `root()`

## Relationships
- **Depends on**: `reth-trie-sparse` for sparse node types and common interfaces.
- **Depends on**: `reth-trie-common` for masks, prefix sets, and RLP utilities.
- **Data/control flow**: lower subtrie hash updates feed upper subtrie hashing to produce the
  final root; optional metrics record update counts and latencies.
```

## File: crates/trie/trie/benches/AGENTS.md
```markdown
# benches

## Purpose
Criterion benchmarks for trie hashing, proof generation, and root calculation performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `hash_post_state.rs`
- **Role**: Benchmarks sequential versus parallel hashing of `HashedPostState` from bundle state.
- **Key items**: `hash_post_state()`, `from_bundle_state_seq()`, `generate_test_data()`,
  `HashedPostState::from_bundle_state()`
- **Interactions**: Uses `revm_database::BundleBuilder` to generate bundle state inputs.
- **Knobs / invariants**: Dataset sizes scale from 100 to 10_000; codspeed skips larger sizes.

### `proof_v2.rs`
- **Role**: Benchmarks legacy storage multiproof generation versus proof_v2 implementation.
- **Key items**: `bench_proof_algos()`, `generate_test_data()`, `StorageProof`,
  `StorageProofCalculator`
- **Interactions**: Uses mock cursor factories built from `HashedPostState`.
- **Knobs / invariants**: Dataset size and number of targets vary per benchmark case.

### `trie_root.rs`
- **Role**: Benchmarks receipt root calculation using `triehash::ordered_trie_root` vs
  `HashBuilder`.
- **Key items**: `trie_root_benchmark()`, `trie_hash_ordered_trie_root()`, `hash_builder_root()`
- **Interactions**: Encodes receipts via `Encodable2718` before hashing.

## End-to-end flow (high level)
- Generate randomized dataset inputs.
- Run alternative implementations for each operation (hashing, proof, root).
- Record timing across dataset sizes and target counts.
```

## File: crates/trie/trie/src/hashed_cursor/AGENTS.md
```markdown
# hashed_cursor

## Purpose
Hashed state cursor traits and overlay implementations used to iterate hashed accounts and
storages, plus noop/mock cursors and metrics instrumentation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Defines core hashed cursor traits and re-exports implementations and metrics helpers.
- **Key items**: `HashedCursorFactory`, `HashedCursor`, `HashedStorageCursor`,
  `HashedCursorMetricsCache`, `InstrumentedHashedCursor`
- **Interactions**: Implemented by `post_state.rs`, `noop.rs`, and `mock.rs`.

### `post_state.rs`
- **Role**: Overlay cursors that merge database state with in-memory post-state updates.
- **Key items**: `HashedPostStateCursorFactory`, `HashedPostStateCursor`,
  `HashedPostStateCursorValue`, `new_account()`, `new_storage()`
- **Interactions**: Uses `ForwardInMemoryCursor` to merge sorted updates with DB cursors.
- **Knobs / invariants**: Post-state entries take precedence; deletions use `None` or `U256::ZERO`;
  storage cursors must `set_hashed_address()` before seeking.

### `noop.rs`
- **Role**: Noop hashed cursor implementations for empty datasets or tests.
- **Key items**: `NoopHashedCursorFactory`, `NoopHashedCursor`, `is_storage_empty()`
- **Interactions**: Satisfies `HashedCursorFactory`/`HashedStorageCursor` traits.

### `metrics.rs`
- **Role**: Metrics wrappers and caches for hashed cursor operations.
- **Key items**: `HashedCursorMetrics`, `HashedCursorMetricsCache`, `InstrumentedHashedCursor`
- **Interactions**: Wraps a `HashedCursor` to record seek/next timings and counts.

### `mock.rs`
- **Role**: Mock hashed cursor factory and cursors for tests with visit tracking.
- **Key items**: `MockHashedCursorFactory`, `MockHashedCursor`, `MockHashedCursorType`,
  `visited_account_keys()`, `visited_storage_keys()`
- **Interactions**: Builds cursors from `HashedPostState` and records key visits for assertions.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `HashedCursorFactory`, `HashedCursor`, `HashedStorageCursor`,
  `HashedPostStateCursor`, `InstrumentedHashedCursor`
- **Modules / Packages**: `post_state`, `noop`, `metrics`, `mock`
- **Functions**: `hashed_account_cursor()`, `hashed_storage_cursor()`, `set_hashed_address()`

## Relationships
- **Depends on**: `reth-trie-common::HashedPostStateSorted` for post-state overlays.
- **Data/control flow**: post-state overlays merge in-memory updates with DB cursors, yielding a
  single ordered stream of hashed entries.
- **Data/control flow**: instrumentation wrappers record operation counts/latency for metrics.

## End-to-end flow (high level)
- Create a cursor factory (DB-backed, overlay, or noop).
- Iterate hashed accounts and storage entries with `seek`/`next`.
- For overlays, in-memory updates override DB entries and deletions are filtered out.
- Optional metrics wrappers record operation counts and timing.
```

## File: crates/trie/trie/src/proof/AGENTS.md
```markdown
# proof

## Purpose
Merkle proof generation for account and storage tries using trie walkers, hashed cursors, and
hash builder retention.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Proof generators for accounts and storage, producing multiproofs and account proofs.
- **Key items**: `Proof`, `StorageProof`, `account_proof()`, `multiproof()`, `storage_proof()`,
  `storage_multiproof()`, `with_branch_node_masks()`
- **Interactions**: Uses `TrieWalker`, `TrieNodeIter`, and `HashBuilder` to rebuild roots and
  retain proof nodes.
- **Knobs / invariants**: `collect_branch_node_masks` toggles branch mask capture; prefix sets are
  extended with proof targets.

### `trie_node.rs`
- **Role**: Blinded node providers that materialize missing trie nodes by issuing proofs.
- **Key items**: `ProofTrieNodeProviderFactory`, `ProofBlindedAccountProvider`,
  `ProofBlindedStorageProvider`
- **Interactions**: Uses `Proof`/`StorageProof` to fetch nodes and emits `RevealedNode` with masks.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Proof`, `StorageProof`, `ProofTrieNodeProviderFactory`
- **Modules / Packages**: `trie_node`
- **Functions**: `account_proof()`, `multiproof()`, `storage_proof()`, `storage_multiproof()`

## Relationships
- **Depends on**: `TrieCursorFactory` and `HashedCursorFactory` for trie traversal and leaf access.
- **Depends on**: `reth-trie-common` proof types (`MultiProof`, `StorageMultiProof`).
- **Data/control flow**: proof targets expand prefix sets, walker iterates trie nodes, hash builder
  retains proof nodes, and storage proofs are embedded into account proofs.

## End-to-end flow (high level)
- Build a `Proof` with trie and hashed cursor factories.
- Extend prefix sets with targets and walk the trie.
- Use `HashBuilder` retainer to capture proof nodes while recomputing roots.
- Emit account/storage proofs; optionally fetch blinded nodes via proof-based providers.
```

## File: crates/trie/trie/src/proof_v2/AGENTS.md
```markdown
# proof_v2

## Purpose
Leaf-only proof calculator that generates lexicographically ordered proof nodes with cursor
reuse and deferred value encoding.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core proof calculator that builds proofs from leaf data and cached branch nodes.
- **Key items**: `ProofCalculator`, `StorageProofCalculator`, `storage_proof()`,
  `TargetsCursor`, `PATH_ALL_ZEROS`
- **Interactions**: Uses `TrieCursor` for branch nodes and `HashedCursor` for leaf data, emitting
  `ProofTrieNode`s in depth-first order.
- **Knobs / invariants**: Targets must be sorted; calculator resets after each call and reuses
  internal buffers.

### `node.rs`
- **Role**: Internal branch/child node representations and RLP conversion helpers.
- **Key items**: `ProofTrieBranchChild`, `ProofTrieBranch`, `trim_nibbles_prefix()`
- **Interactions**: Converts deferred leaf encoders into RLP nodes and proof nodes.

### `target.rs`
- **Role**: Proof target representation and sub-trie grouping helpers.
- **Key items**: `Target`, `SubTrieTargets`, `iter_sub_trie_targets()`
- **Interactions**: Chunks targets by sub-trie prefix to drive proof calculation.

### `value.rs`
- **Role**: Deferred leaf value encoding for storage slots and accounts.
- **Key items**: `LeafValueEncoder`, `DeferredValueEncoder`, `StorageValueEncoder`,
  `SyncAccountValueEncoder`
- **Interactions**: Integrates with `ProofCalculator` to encode values late, allowing storage
  root computation at encode time.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProofCalculator`, `StorageProofCalculator`, `Target`,
  `LeafValueEncoder`, `DeferredValueEncoder`
- **Modules / Packages**: `node`, `target`, `value`
- **Functions**: `storage_proof()`, `iter_sub_trie_targets()`, `deferred_encoder()`

## Relationships
- **Depends on**: `TrieCursor` and `HashedCursor` for branch/leaf data.
- **Depends on**: `reth-trie-common` proof node types and masks.
- **Data/control flow**: targets are grouped by sub-trie, leaves are encoded lazily, branch nodes
  are assembled and retained into proof output.

## End-to-end flow (high level)
- Build a `ProofCalculator` with trie and hashed cursors.
- Sort and chunk targets by sub-trie prefix.
- Walk leaf data, building branch stacks and encoding deferred leaf values.
- Emit proof nodes in lexicographic order and reset internal buffers for reuse.
```

## File: crates/trie/trie/src/trie_cursor/AGENTS.md
```markdown
# trie_cursor

## Purpose
Trie cursor traits and implementations for iterating account and storage tries, including
in-memory overlays, depth-first traversal, and metrics instrumentation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Defines core trie cursor traits and exports cursor helpers.
- **Key items**: `TrieCursorFactory`, `TrieCursor`, `TrieStorageCursor`, `TrieCursorIter`,
  `DepthFirstTrieIterator`, `CursorSubNode`
- **Interactions**: Implemented by `in_memory.rs`, `noop.rs`, and `mock.rs`.

### `depth_first.rs`
- **Role**: Depth-first post-order iterator over trie nodes.
- **Key items**: `DepthFirstTrieIterator`, `cmp()`
- **Interactions**: Used by verification routines to compare branch nodes in depth-first order.

### `in_memory.rs`
- **Role**: Overlay cursors that merge trie updates with database cursors.
- **Key items**: `InMemoryTrieCursorFactory`, `InMemoryTrieCursor`, `new_account()`, `new_storage()`
- **Interactions**: Uses `ForwardInMemoryCursor` and `TrieUpdatesSorted` to override DB nodes.
- **Knobs / invariants**: Overlay entries take precedence; wiped storage bypasses DB cursor.

### `metrics.rs`
- **Role**: Metrics wrappers and caches for trie cursor operations.
- **Key items**: `TrieCursorMetrics`, `TrieCursorMetricsCache`, `InstrumentedTrieCursor`
- **Interactions**: Wraps `TrieCursor`/`TrieStorageCursor` to record seek/next timings and counts.

### `mock.rs`
- **Role**: Mock trie cursor factory and cursors for tests with visit tracking.
- **Key items**: `MockTrieCursorFactory`, `MockTrieCursor`, `visited_account_keys()`,
  `visited_storage_keys()`
- **Interactions**: Builds cursors from `TrieUpdates` to drive tests.

### `noop.rs`
- **Role**: Noop trie cursor implementations for empty datasets or tests.
- **Key items**: `NoopTrieCursorFactory`, `NoopAccountTrieCursor`, `NoopStorageTrieCursor`
- **Interactions**: Satisfies `TrieCursorFactory`/`TrieStorageCursor` traits.

### `subnode.rs`
- **Role**: Subtrie cursor node representation with mask and hash accessors.
- **Key items**: `CursorSubNode`, `SubNodePosition`, `full_key_is_only_nonremoved_child()`,
  `hash_flag()`, `maybe_hash()`
- **Interactions**: Used by `TrieWalker` and progress checkpoints.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `TrieCursorFactory`, `TrieCursor`, `TrieStorageCursor`,
  `TrieCursorIter`, `DepthFirstTrieIterator`, `CursorSubNode`
- **Modules / Packages**: `in_memory`, `depth_first`, `metrics`, `subnode`
- **Functions**: `seek()`, `seek_exact()`, `next()`, `set_hashed_address()`

## Relationships
- **Depends on**: `reth-trie-common` for `BranchNodeCompact` and `Nibbles`.
- **Data/control flow**: in-memory overlays merge updates with DB cursor output, yielding a
  unified ordered stream of nodes.
- **Data/control flow**: depth-first iterators and subnode helpers feed root verification and
  trie walking logic.

## End-to-end flow (high level)
- Create a cursor factory (DB-backed, in-memory overlay, or noop).
- Use cursors to seek/iterate nodes in lexicographic order.
- Optionally wrap cursors with metrics instrumentation.
- Feed cursor output into walkers, proof generators, or verification routines.
```

## File: crates/trie/trie/src/AGENTS.md
```markdown
# src

## Purpose
Merkle Patricia Trie core: root calculation, cursors and walkers, proof/witness generation, and
verification utilities.

## Contents (one hop)
### Subdirectories
- [x] `hashed_cursor/` - Hashed state cursor traits, overlays, metrics, and mocks.
- [x] `proof/` - Proof generators and proof-based node providers.
- [x] `proof_v2/` - Leaf-only proof calculator with deferred value encoding.
- [x] `trie_cursor/` - Trie cursor traits and implementations (overlay, depth-first, metrics).

### Files
- `changesets.rs` - Computes trie changesets (old node values) from trie updates.
  - **Key items**: `compute_trie_changesets()`, `compute_storage_changesets()`,
    `compute_wiped_storage_changesets()`, `storage_trie_wiped_changeset_iter()`
- `forward_cursor.rs` - Forward-only in-memory cursor over sorted entries.
  - **Key items**: `ForwardInMemoryCursor`, `seek()`, `first_after()`, `BINARY_SEARCH_THRESHOLD`
- `lib.rs` - Module wiring and re-exports for trie components and progress types.
  - **Key items**: `StateRoot`, `StorageRoot`, `TrieType`, `StateRootProgress`, `StorageRootProgress`
- `metrics.rs` - Metrics for root computation, walkers, and node iterators.
  - **Key items**: `StateRootMetrics`, `TrieRootMetrics`, `WalkerMetrics`, `TrieNodeIterMetrics`
- `mock.rs` - Shared mock helpers for cursor tests.
  - **Key items**: `KeyVisit`, `KeyVisitType`
- `node_iter.rs` - Iterator that merges trie walkers with hashed cursors for hash building.
  - **Key items**: `TrieNodeIter`, `TrieElement`, `TrieBranchNode`, `try_next()`
- `progress.rs` - Intermediate checkpoint types for resumable root computation.
  - **Key items**: `StateRootProgress`, `StorageRootProgress`, `IntermediateStateRootState`,
    `IntermediateRootState`
- `stats.rs` - Root calculation stats and tracker.
  - **Key items**: `TrieStats`, `TrieTracker`
- `test_utils.rs` - Test helpers for computing trie roots via `triehash`.
  - **Key items**: `state_root()`, `storage_root()`, `state_root_prehashed()`,
    `storage_root_prehashed()`
- `trie.rs` - State and storage root computation pipelines with thresholds and updates.
  - **Key items**: `StateRoot`, `StorageRoot`, `TrieType`, `root_with_updates()`,
    `root_with_progress()`, `with_threshold()`
- `verify.rs` - Verifies trie tables against hashed state tables, reporting inconsistencies.
  - **Key items**: `Verifier`, `Output`, `StateRootBranchNodesIter`
- `walker.rs` - Trie traversal with prefix-set skipping and removal tracking.
  - **Key items**: `TrieWalker`, `next_unprocessed_key()`, `with_deletions_retained()`
- `witness.rs` - Builds state transition witnesses using multiproofs and sparse trie updates.
  - **Key items**: `TrieWitness`, `WitnessTrieNodeProviderFactory`, `compute()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StateRoot`, `StorageRoot`, `TrieWalker`, `TrieNodeIter`,
  `TrieCursorFactory`, `HashedCursorFactory`, `Verifier`, `TrieWitness`
- **Modules / Packages**: `changesets`, `proof`, `proof_v2`, `trie_cursor`, `hashed_cursor`
- **Functions**: `compute_trie_changesets()`, `root_with_updates()`, `multiproof()`,
  `storage_proof()`

## Relationships
- **Depends on**: `reth-trie-common` for hash builder, node types, and prefix sets.
- **Depends on**: `reth-trie-sparse` for sparse trie interfaces used in witness generation.
- **Data/control flow**: trie cursors and walkers feed `TrieNodeIter`, which drives `HashBuilder`
  for root computation and updates.
- **Data/control flow**: proofs and witnesses reuse cursor factories to fetch nodes and build
  proof node collections.
- **Data/control flow**: verification recomputes branch nodes from hashed state and compares
  against trie tables.
```

## File: crates/trie/trie/AGENTS.md
```markdown
# trie

## Purpose
`reth-trie` crate: Merkle Patricia Trie implementation with root computation, cursors, proofs,
witness generation, and verification tooling.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for hashing, proof generation, and trie roots.
- [x] `src/` - Core trie logic: root calculation, cursors, proofs, witnesses, verification.
- [x] `testdata/` - (skip: static JSON fixtures for proof tests)

### Files
- `Cargo.toml` - Crate manifest for trie implementation and feature flags.
  - **Key items**: features `metrics`, `serde`, `test-utils`; benches `hash_post_state`,
    `trie_root`, `proof_v2`; deps `reth-trie-common`, `reth-trie-sparse`, `alloy-trie`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StateRoot`, `StorageRoot`, `TrieWalker`, `TrieNodeIter`,
  `TrieWitness`, `Proof`, `ProofCalculator`
- **Modules / Packages**: `trie_cursor`, `hashed_cursor`, `proof`, `proof_v2`, `witness`
- **Functions**: `root_with_updates()`, `multiproof()`, `storage_proof()`

## Relationships
- **Depends on**: `reth-trie-common` for hash builder, nodes, and prefix sets.
- **Depends on**: `reth-trie-sparse` for sparse trie integration in witness generation.
- **Depends on**: `alloy-trie`/`alloy-rlp` for trie node encoding and proofs.
- **Data/control flow**: cursors and walkers feed node iterators, which drive hash builders to
  compute roots and updates.
```

## File: crates/trie/AGENTS.md
```markdown
# trie

## Purpose
Trie subsystem crates: common data types, database-backed access, sparse trie variants, and parallelized root/proof computation.

## Contents (one hop)
### Subdirectories
- [x] `common/` - Shared trie types: hashed state overlays, proofs, updates, prefix sets, and root helpers.
- [x] `db/` - Database-backed trie integration: cursor factories, roots/proofs, and changeset caching.
- [x] `parallel/` - Parallel state root/proof computation with worker pools and metrics.
- [x] `sparse/` - Sparse MPT implementation with lazy reveal, state trie wrapper, and update tracking.
- [x] `sparse-parallel/` - Parallel sparse MPT using upper/lower subtries and update thresholds.
- [x] `trie/` - Merkle trie core: root computation, cursors, proofs, witnesses, and verification.

### Files
- (none)

## Key APIs (no snippets)
- **Modules / Packages**: `common`, `db`, `sparse`, `parallel`, `trie` - coordinated crates for trie storage, computation, and proofs.

## Relationships
- **Depends on**: `common` types are shared by the other trie crates in this subtree.
```

## File: crates/AGENTS.md
```markdown
# crates

## Purpose
Reth's Rust workspace crates: domain-grouped libraries and binaries that make up the node (networking, engine, storage, RPC, sync stages, primitives, etc.).

## Contents (one hop)
### Subdirectories
- [x] `chain-state/` - Canonical-chain in-memory state tracking, forkchoice head tracking (head/safe/finalized), and notification streams for consumers like engine/RPC.
- [x] `chainspec/` - Ethereum network spec definitions: genesis inputs, hardfork schedule, and chain-spec query traits/presets used across reth.
- [x] `cli/` - CLI subsystem crates: shared CLI traits/parsers, a tokio runner, reusable parsing/runtime utilities, and the concrete `reth ...` command suite (node/db/stage/import/export/etc.).
- [x] `config/` - Reth configuration schema (`Config`): stage/pipeline knobs, pruning/static-file settings, and peer/session config (with optional TOML load/save via `serde` feature).
- [x] `consensus/` - Consensus/validation crates: core consensus traits+errors, shared Ethereum block/header validation helpers, and a debug client for driving the engine API from external block sources.
- [x] `e2e-test-utils/` - E2E testing framework and helpers: spin up in-process nodes, drive them via RPC/Engine API, and express tests as composable "actions" (produce blocks, forks/reorgs, sync assertions).
- [x] `engine/` - Engine subsystem crates: engine-tree, engine service wiring, engine API primitives/config, and testing/debug utilities.
- [x] `era/` - E2Store-based history file formats: core read/write and compression logic for `.era` (CL) and `.era1` (EL) files, including indices and standardized naming.
- [x] `era-downloader/` - Async ERA file downloader/streamer: fetches index/checksums (for `.era1`), downloads files with concurrency + retention limits, and supports local `.era1` directory streaming with checksum validation.
- [x] `era-utils/` - ERA storage utilities: import `.era1` history into reth storage (static files/DB/checkpoints) and export stored history back into `.era1` files with configurable chunking/naming.
- [x] `errors/` - High-level error umbrella crate: `RethError` and common error/result re-exports across consensus/execution/storage subsystems.
- [x] `ethereum/` - Ethereum-specific node stack: Ethereum CLI, consensus validation, Engine API primitives, EVM config, fork schedules, payload builder/validator, Ethereum node wiring (components/RPC add-ons), and canonical primitive types.
- [x] `etl/` - ETL collector: buffers unsorted key/value pairs, sorts + spills to temp files, and iterates a merged sorted stream (often used for efficient sorted DB inserts / indexing).
- [x] `evm/` - EVM/execution subsystem: core `reth-evm` execution traits/config, shared execution error types, and shared execution outcome/result types (`ExecutionOutcome`, `Chain`, etc.).
- [x] `exex/` - Execution extensions subsystem: ExEx manager/runtime, shared ExEx notification/progress types, and test utilities; includes notification backfill + WAL persistence to support recovery and pruning-safe progress.
- [x] `fs-util/` - Path-aware filesystem utilities: wraps `std::fs` with richer errors and provides common helpers like atomic writes and JSON read/write.
- [x] `metrics/` - Metrics utilities: re-exports the `Metrics` derive macro and provides common helpers like metered tokio mpsc channels (feature-gated).
- [x] `net/` - Networking stack: discovery, peer/session management, wire protocols, downloaders, network manager, and shared networking types/APIs.
- [x] `node/` - Node-layer crates: builder/launch, core config/CLI utils, metrics/events, EthStats, and node API/types.
- [x] `optimism/` - OP-specific crates: chain specs, consensus/EVM, node wiring, RPC, flashblocks, and txpool support.
- [x] `payload/` - Payload builder subsystem: jobs/services, primitives, validators, and utilities.
- [x] `primitives/` - Common block/tx/receipt types, transaction helpers, and primitives benchmarks.
- [x] `primitives-traits/` - Core traits/utilities for blocks, headers, txs, receipts, and serialization helpers.
- [x] `prune/` - Pruning engine, segment definitions, and shared prune configuration types.
- [x] `ress/` - RESS protocol and provider crates for stateless witness/bytecode fetching.
- [x] `revm/` - Reth-specific revm adapters, cached reads, and witness helpers.
- [x] `rpc/` - RPC subsystem crates: interfaces, handlers, builders, and supporting types/tests.
- [x] `stages/` - Staged sync APIs/types and concrete stage implementations with benchmarks.
- [x] `stateless/` - Stateless validation pipeline, trie helpers, and witness-backed DB.
- [x] `static-file/` - Static file producer and shared segment/type definitions.
- [x] `storage/` - Storage crates for DB access, providers, codecs, and static files.
- [x] `tasks/` - Task management, executors, shutdown signaling, and metrics.
- [x] `tokio-util/` - Tokio event broadcasting utilities and optional rate limiter.
- [x] `tracing/` - Tracing configuration, formatter, layer builders, and throttle macro.
- [x] `tracing-otlp/` - OTLP tracing/logging layer builders and config types.
- [x] `transaction-pool/` - Transaction pool implementation with validation, subpools, and metrics.
- [x] `trie/` - Trie subsystem crates: common types, DB integration, sparse/parallel, and core roots/proofs.

### Files
- (none)

## Notes
- This `AGENTS.md` is intentionally high-level; details are pushed down into each subdirectory's own `AGENTS.md`.
```
