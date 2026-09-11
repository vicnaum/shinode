//! Chain identity and protocol types, adapted from Sieve 914d29f (MIT OR Apache-2.0).

mod base;
mod ethereum;
mod op_stack;

pub use base::BaseChain;
pub use ethereum::EthereumChain;

use alloy_consensus::{BlockBody, TxReceipt};
use alloy_primitives::B256;
use clap::ValueEnum;
use reth_chainspec::{EthChainSpec, Hardforks};
use reth_eth_wire_types::{NetPrimitivesFor, NetworkPrimitives};
use reth_network::config::NetworkConfigBuilder;
use reth_primitives_traits::{Header, NodePrimitives, SignedTransaction};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainKind {
    #[default]
    Mainnet,
    Base,
}

impl ChainKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Mainnet => EthereumChain::NAME,
            Self::Base => BaseChain::NAME,
        }
    }

    #[must_use]
    pub fn chain_id(self) -> u64 {
        match self {
            Self::Mainnet => EthereumChain::chain_spec().chain().id(),
            Self::Base => BaseChain::chain_spec().chain().id(),
        }
    }

    #[must_use]
    pub fn genesis_hash(self) -> B256 {
        match self {
            Self::Mainnet => EthereumChain::chain_spec().genesis_hash(),
            Self::Base => BaseChain::chain_spec().genesis_hash(),
        }
    }
}

/// Compile-time wire types and validation behavior for one chain.
pub trait ChainTypes: Send + Sync + Sized + 'static {
    type SignedTx: SignedTransaction;
    type Receipt: TxReceipt<Log = alloy_primitives::Log> + Unpin + 'static;
    type Primitives: NodePrimitives<
        BlockHeader = Header,
        BlockBody = BlockBody<Self::SignedTx>,
        SignedTx = Self::SignedTx,
        Receipt = Self::Receipt,
    >;
    type Net: NetworkPrimitives<
            BlockHeader = Header,
            BlockBody = BlockBody<Self::SignedTx>,
            Receipt = Self::Receipt,
        > + NetPrimitivesFor<Self::Primitives>;
    type Spec: EthChainSpec + Hardforks + Send + Sync + 'static;

    const NAME: &'static str;
    fn chain_spec() -> Arc<Self::Spec>;
    fn receipts_root(receipts: &[Self::Receipt], header: &Header) -> B256;
    fn withdrawals_valid(header: &Header, body: &BlockBody<Self::SignedTx>) -> bool;
    #[must_use]
    fn configure_network(
        builder: NetworkConfigBuilder<Self::Net>,
        _listen_addr: SocketAddr,
    ) -> NetworkConfigBuilder<Self::Net> {
        builder
    }
}
