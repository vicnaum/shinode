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
