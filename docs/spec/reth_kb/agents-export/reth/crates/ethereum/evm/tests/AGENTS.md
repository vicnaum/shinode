# tests

## Purpose
Execution/validation tests for `reth-evm-ethereum`: asserts fork-specific execution behaviours and header-field requirements (e.g. Cancun parent beacon root, system contract predeploy interactions, Prague requests hash, etc.).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `execute.rs` - end-to-end execution tests over `BasicBlockExecutor` + `EthEvmConfig`, covering multiple EIPs and fork activation scenarios.
