// Adapted from Sieve 914d29f (MIT OR Apache-2.0).
//! Ethereum mainnet chain wiring.

use super::ChainTypes;
use alloy_consensus::BlockBody;
use alloy_primitives::B256;
use reth_chainspec::{ChainSpec, EthereumHardforks, MAINNET};
use reth_eth_wire::EthNetworkPrimitives;
use reth_ethereum_primitives::{EthPrimitives, Receipt, TransactionSigned};
use reth_primitives_traits::Header;
use std::sync::Arc;

/// Ethereum mainnet.
#[derive(Debug, Clone, Copy)]
pub struct EthereumChain;

impl ChainTypes for EthereumChain {
    type SignedTx = TransactionSigned;
    type Receipt = Receipt;
    type Primitives = EthPrimitives;
    type Net = EthNetworkPrimitives;
    type Spec = ChainSpec;

    const NAME: &'static str = "mainnet";

    fn chain_spec() -> Arc<ChainSpec> {
        MAINNET.clone()
    }

    fn receipts_root(receipts: &[Receipt], _header: &Header) -> B256 {
        Receipt::calculate_receipt_root_no_memo(receipts)
    }

    /// Fork-aware withdrawals validation: the field must be absent before
    /// Shanghai, present from Shanghai on, and the roots must match.
    fn withdrawals_valid(header: &Header, body: &BlockBody<TransactionSigned>) -> bool {
        let shanghai = Self::chain_spec().is_shanghai_active_at_timestamp(header.timestamp);
        match (header.withdrawals_root, body.calculate_withdrawals_root()) {
            (Some(header_root), Some(body_root)) => shanghai && header_root == body_root,
            (None, None) => !shanghai,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;

    /// Shanghai activation on Ethereum mainnet.
    const SHANGHAI_TIMESTAMP: u64 = 1_681_338_455;

    fn header_at(timestamp: u64, withdrawals_root: Option<B256>) -> Header {
        Header {
            timestamp,
            withdrawals_root,
            ..Default::default()
        }
    }

    fn body_with_empty_withdrawals() -> BlockBody<TransactionSigned> {
        BlockBody {
            withdrawals: Some(alloy_eips::eip4895::Withdrawals::default()),
            ..Default::default()
        }
    }

    #[test]
    fn withdrawals_pre_shanghai_requires_none() {
        let ts = SHANGHAI_TIMESTAMP - 1;
        let absent: BlockBody<TransactionSigned> = BlockBody::default();

        assert!(EthereumChain::withdrawals_valid(
            &header_at(ts, None),
            &absent
        ));
        // Paired Some/Some before the fork must be rejected.
        let empty_root = alloy_consensus::constants::EMPTY_ROOT_HASH;
        assert!(!EthereumChain::withdrawals_valid(
            &header_at(ts, Some(empty_root)),
            &body_with_empty_withdrawals()
        ));
    }

    #[test]
    fn withdrawals_post_shanghai_requires_matching_roots() {
        let ts = SHANGHAI_TIMESTAMP;
        let empty_root = alloy_consensus::constants::EMPTY_ROOT_HASH;

        assert!(EthereumChain::withdrawals_valid(
            &header_at(ts, Some(empty_root)),
            &body_with_empty_withdrawals()
        ));
        // Paired None/None after the fork must be rejected.
        let absent: BlockBody<TransactionSigned> = BlockBody::default();
        assert!(!EthereumChain::withdrawals_valid(
            &header_at(ts, None),
            &absent
        ));
        // Mismatched roots rejected.
        assert!(!EthereumChain::withdrawals_valid(
            &header_at(ts, Some(B256::repeat_byte(0x42))),
            &body_with_empty_withdrawals()
        ));
    }
}
