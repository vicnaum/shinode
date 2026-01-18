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
