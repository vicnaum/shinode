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
