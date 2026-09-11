// Adapted from Sieve 914d29f (MIT OR Apache-2.0).
//! Shared OP-stack chain behavior.
//!
//! Base and OP mainnet differ in chain spec (genesis, fork schedule,
//! discovery) but share the OP execution semantics that matter to Sieve:
//! how deposit receipts are hashed into the receipts root, and how the
//! header's withdrawals commitment is validated across Canyon/Isthmus.
//! One implementation here keeps the two chains from drifting apart.

use alloy_consensus::BlockBody;
use alloy_primitives::B256;
use op_alloy_consensus::{OpReceipt, OpTxEnvelope};
use reth_optimism_chainspec::OpChainSpec;
use reth_optimism_forks::OpHardforks;
use reth_primitives_traits::Header;

/// Compute the receipts root for an OP-stack block, recomputing blooms
/// from logs (never trusting the peer). Deposit receipts hash differently
/// depending on Canyon activation, hence the spec + timestamp.
pub(super) fn receipts_root(spec: &OpChainSpec, receipts: &[OpReceipt], header: &Header) -> B256 {
    reth_optimism_consensus::calculate_receipt_root_no_memo_optimism(
        receipts,
        spec,
        header.timestamp,
    )
}

/// Fork-aware OP withdrawals validation, mirroring reth's OP consensus
/// checks plus field-presence rules:
///
/// - pre-Canyon: neither header nor body may carry withdrawals;
/// - Canyon→Isthmus: both must be present and the header root must
///   match the body root (both are the empty-withdrawals root on OP);
/// - post-Isthmus: both must be present; the header root is repurposed
///   as the `L2ToL1MessagePasser` predeploy storage root (not
///   recomputable from the body), and the body root must be the empty
///   root.
pub(super) fn withdrawals_valid(
    spec: &impl OpHardforks,
    header: &Header,
    body: &BlockBody<OpTxEnvelope>,
) -> bool {
    let canyon = spec.is_canyon_active_at_timestamp(header.timestamp);
    match (header.withdrawals_root, body.calculate_withdrawals_root()) {
        (Some(header_root), Some(body_root)) => {
            canyon
                && if spec.is_isthmus_active_at_timestamp(header.timestamp) {
                    body_root == alloy_consensus::constants::EMPTY_ROOT_HASH
                } else {
                    body_root == header_root
                }
        }
        (None, None) => !canyon,
        _ => false,
    }
}
