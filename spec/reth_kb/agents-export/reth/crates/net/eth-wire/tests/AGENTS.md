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
