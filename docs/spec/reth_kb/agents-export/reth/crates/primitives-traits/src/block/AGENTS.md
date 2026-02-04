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
