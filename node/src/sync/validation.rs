//! Verify payload commitments, independently of canonicality or L1 finality.

use crate::chain::ChainTypes;
use alloy_consensus::{proofs, BlockBody, TxReceipt};
use alloy_primitives::Bloom;
use reth_primitives_traits::Header;

pub fn validate_payload<C: ChainTypes>(
    header: &Header,
    body: &BlockBody<C::SignedTx>,
    receipts: &[C::Receipt],
) -> Result<(), &'static str> {
    if body.transactions.len() != receipts.len() {
        return Err("transaction/receipt count mismatch");
    }
    if proofs::calculate_transaction_root(&body.transactions) != header.transactions_root {
        return Err("transactions root mismatch");
    }
    if proofs::calculate_ommers_root(&body.ommers) != header.ommers_hash {
        return Err("ommers root mismatch");
    }
    if !C::withdrawals_valid(header, body) {
        return Err("withdrawals mismatch");
    }
    // Chain implementations recompute receipt blooms from logs before hashing.
    if C::receipts_root(receipts, header) != header.receipts_root {
        return Err("receipts root mismatch");
    }
    let mut bloom = Bloom::ZERO;
    for receipt in receipts {
        bloom.accrue_logs(receipt.logs());
    }
    if bloom != header.logs_bloom {
        return Err("logs bloom mismatch");
    }
    Ok(())
}

#[cfg(test)]
pub mod tests;
