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
