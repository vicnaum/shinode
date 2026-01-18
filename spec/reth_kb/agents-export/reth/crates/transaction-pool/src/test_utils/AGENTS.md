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
