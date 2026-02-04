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
