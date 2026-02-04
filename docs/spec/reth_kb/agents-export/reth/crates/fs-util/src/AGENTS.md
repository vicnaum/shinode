# src

## Purpose
Implements `reth-fs-util`: thin wrappers around `std::fs` operations that enrich errors with path context and add common helpers (e.g. JSON read/write, atomic writes, fsync).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `FsPathError` (path-aware error enum) plus wrapper functions like `open()`, `read()`, `write()`, directory ops, metadata/fsync helpers, and JSON helpers.
  - **Key items**: `FsPathError`, `FsPathError::*` constructors, `read_to_string()`, `atomic_write_file()`
