// Adapted from Sieve 914d29f (MIT OR Apache-2.0).
//! Base mainnet chain wiring (OP-stack L2, chain id 8453).
//!
//! Base left the OP superchain registry in April 2026 and now runs its own
//! client (base-reth-node). Three things differ from the pinned reth's
//! built-in `BASE_MAINNET`:
//!
//! 1. **Hardforks**: two post-Jovian forks are live — Azul (2026-05-28,
//!    which also activates Ethereum Osaka) and Beryl (2026-06-25). Without
//!    them the EIP-2124 fork-id would be stale and current peers would
//!    reject the handshake. The expected head fork-id is `0xea0159eb/0`
//!    (verified by a unit test below).
//! 2. **Discovery**: Base EL nodes run discv5 with a custom packet
//!    protocol id `basev0` (default is `discv5`), hard-partitioning their
//!    DHT — a stock discv5 node cannot even decode Base discovery packets.
//!    discv4 is disabled on Base nodes.
//! 3. **Types**: every Base block carries a type-0x7E deposit transaction,
//!    so bodies/receipts use the OP envelope types.
//!
//! Verified through Beryl on 2026-09-11; see docs/base-protocol.md.
//! The pinned OP types do not support type 0x79 (Zenith in the
//! reviewed official Base source). Recheck runtime upgrades before release.

use super::{op_stack, ChainTypes};
use alloy_primitives::B256;
use op_alloy_consensus::{OpPooledTransaction, OpReceipt, OpTxEnvelope};
use reth_chainspec::{EthereumHardfork, ForkCondition, Hardfork};
use reth_eth_wire_types::BasicNetworkPrimitives;
use reth_network::config::NetworkConfigBuilder;
use reth_network_peers::NodeRecord;
use reth_optimism_chainspec::{OpChainSpec, BASE_MAINNET};
use reth_optimism_primitives::OpPrimitives;
use reth_primitives_traits::Header;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};

/// Azul activation timestamp on Base mainnet (2026-05-28 18:00:00 UTC).
/// Also activates Ethereum Osaka at the execution layer.
const AZUL_TIMESTAMP: u64 = 1_779_991_200;

/// Beryl activation timestamp on Base mainnet (2026-06-25 18:00:00 UTC).
const BERYL_TIMESTAMP: u64 = 1_782_410_400;

/// discv5 packet protocol id used by Base EL nodes since Azul.
const BASE_V0_PROTOCOL_ID: [u8; 6] = *b"basev0";

/// Base mainnet EL devp2p bootnodes (5 hosts, each on the legacy discv5
/// port 30301 and the newer 9200). From base-reth-node's chain config.
const BASE_BOOTNODES: [&str; 10] = [
    "enode://87a32fd13bd596b2ffca97020e31aef4ddcc1bbd4b95bb633d16c1329f654f34049ed240a36b449fda5e5225d70fe40bc667f53c304b71f8e68fc9d448690b51@3.231.138.188:30301",
    "enode://87a32fd13bd596b2ffca97020e31aef4ddcc1bbd4b95bb633d16c1329f654f34049ed240a36b449fda5e5225d70fe40bc667f53c304b71f8e68fc9d448690b51@3.231.138.188:9200",
    "enode://ca21ea8f176adb2e229ce2d700830c844af0ea941a1d8152a9513b966fe525e809c3a6c73a2c18a12b74ed6ec4380edf91662778fe0b79f6a591236e49e176f9@184.72.129.189:30301",
    "enode://ca21ea8f176adb2e229ce2d700830c844af0ea941a1d8152a9513b966fe525e809c3a6c73a2c18a12b74ed6ec4380edf91662778fe0b79f6a591236e49e176f9@184.72.129.189:9200",
    "enode://acf4507a211ba7c1e52cdf4eef62cdc3c32e7c9c47998954f7ba024026f9a6b2150cd3f0b734d9c78e507ab70d59ba61dfe5c45e1078c7ad0775fb251d7735a2@3.220.145.177:30301",
    "enode://acf4507a211ba7c1e52cdf4eef62cdc3c32e7c9c47998954f7ba024026f9a6b2150cd3f0b734d9c78e507ab70d59ba61dfe5c45e1078c7ad0775fb251d7735a2@3.220.145.177:9200",
    "enode://8a5a5006159bf079d06a04e5eceab2a1ce6e0f721875b2a9c96905336219dbe14203d38f70f3754686a6324f786c2f9852d8c0dd3adac2d080f4db35efc678c5@3.231.11.52:30301",
    "enode://8a5a5006159bf079d06a04e5eceab2a1ce6e0f721875b2a9c96905336219dbe14203d38f70f3754686a6324f786c2f9852d8c0dd3adac2d080f4db35efc678c5@3.231.11.52:9200",
    "enode://cdadbe835308ad3557f9a1de8db411da1a260a98f8421d62da90e71da66e55e98aaa8e90aa7ce01b408a54e4bd2253d701218081ded3dbe5efbbc7b41d7cef79@54.198.153.150:30301",
    "enode://cdadbe835308ad3557f9a1de8db411da1a260a98f8421d62da90e71da66e55e98aaa8e90aa7ce01b408a54e4bd2253d701218081ded3dbe5efbbc7b41d7cef79@54.198.153.150:9200",
];

/// Base-specific hardforks not present in the pinned reth's fork list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BaseHardfork {
    /// 2026-05-28: Osaka EVM features, eth/69, tx gas cap.
    Azul,
    /// 2026-06-25: B20 native token precompiles.
    Beryl,
}

impl Hardfork for BaseHardfork {
    fn name(&self) -> &'static str {
        match self {
            Self::Azul => "Azul",
            Self::Beryl => "Beryl",
        }
    }
}

/// `BASE_MAINNET` extended with the post-Jovian hardforks Base activated
/// after leaving the superchain registry.
static BASE_MAINNET_CURRENT: LazyLock<Arc<OpChainSpec>> = LazyLock::new(|| {
    let mut spec: OpChainSpec = (**BASE_MAINNET).clone();
    spec.inner.hardforks.insert(
        EthereumHardfork::Osaka,
        ForkCondition::Timestamp(AZUL_TIMESTAMP),
    );
    spec.inner
        .hardforks
        .insert(BaseHardfork::Azul, ForkCondition::Timestamp(AZUL_TIMESTAMP));
    spec.inner.hardforks.insert(
        BaseHardfork::Beryl,
        ForkCondition::Timestamp(BERYL_TIMESTAMP),
    );
    Arc::new(spec)
});

/// Parse the Base bootnode list.
fn base_boot_nodes() -> Vec<NodeRecord> {
    BASE_BOOTNODES
        .iter()
        .filter_map(|record| record.parse().ok())
        .collect()
}

/// Base mainnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseChain;

impl ChainTypes for BaseChain {
    type SignedTx = OpTxEnvelope;
    type Receipt = OpReceipt;
    type Primitives = OpPrimitives;
    type Net = BasicNetworkPrimitives<OpPrimitives, OpPooledTransaction>;
    type Spec = OpChainSpec;

    const NAME: &'static str = "base";

    fn chain_spec() -> Arc<OpChainSpec> {
        Arc::clone(&BASE_MAINNET_CURRENT)
    }

    fn receipts_root(receipts: &[OpReceipt], header: &Header) -> B256 {
        op_stack::receipts_root(Self::chain_spec().as_ref(), receipts, header)
    }

    fn withdrawals_valid(header: &Header, body: &alloy_consensus::BlockBody<OpTxEnvelope>) -> bool {
        op_stack::withdrawals_valid(Self::chain_spec().as_ref(), header, body)
    }

    /// Base discovery: discv4 off, discv5 with the `basev0` packet
    /// protocol id, bootstrapped from the Base bootnodes. The RLPx-level
    /// boot nodes are replaced with the Base list too (the chainspec's
    /// built-in list is the stale OP superchain set).
    fn configure_network(
        builder: NetworkConfigBuilder<Self::Net>,
        listen_addr: SocketAddr,
    ) -> NetworkConfigBuilder<Self::Net> {
        let mut discv5_config = discv5::ConfigBuilder::new(discv5::ListenConfig::from_ip(
            std::net::Ipv4Addr::UNSPECIFIED.into(),
            listen_addr.port(),
        ));
        discv5_config.protocol_identity(discv5::ProtocolIdentity {
            protocol_id: BASE_V0_PROTOCOL_ID,
            ..Default::default()
        });

        let discv5_builder = reth_discv5::Config::builder(listen_addr)
            .discv5_config(discv5_config.build())
            .add_unsigned_boot_nodes(base_boot_nodes());

        builder
            .boot_nodes(base_boot_nodes())
            .disable_discv4_discovery()
            .discovery_v5(discv5_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::b256;
    use reth_chainspec::EthChainSpec;

    #[test]
    fn genesis_hash_matches_base_mainnet() {
        assert_eq!(
            BaseChain::chain_spec().genesis_hash(),
            b256!("f712aa9241cc24369b143cf6dce85f0902a9731e70d66818a3a5845b296c73dd")
        );
    }

    #[test]
    fn latest_fork_id_matches_beryl() {
        // Expected head fork-id after Beryl, verified against
        // base-reth-node's own fork-id test vectors + EIP-2124 CRC chain.
        let fork_id = BaseChain::chain_spec().latest_fork_id();
        assert_eq!(fork_id.hash.0, [0xea, 0x01, 0x59, 0xeb]);
        assert_eq!(fork_id.next, 0);
    }

    #[test]
    fn all_bootnodes_parse() {
        assert_eq!(base_boot_nodes().len(), BASE_BOOTNODES.len());
    }

    /// Canyon activation on Base mainnet (Shanghai / withdrawals field).
    const CANYON_TIMESTAMP: u64 = 1_704_992_401;
    /// Isthmus activation on Base mainnet (withdrawals_root repurposed).
    const ISTHMUS_TIMESTAMP: u64 = 1_746_806_401;

    fn header_at(timestamp: u64, withdrawals_root: Option<alloy_primitives::B256>) -> Header {
        Header {
            timestamp,
            withdrawals_root,
            ..Default::default()
        }
    }

    fn empty_withdrawals_body() -> alloy_consensus::BlockBody<OpTxEnvelope> {
        alloy_consensus::BlockBody {
            withdrawals: Some(alloy_eips::eip4895::Withdrawals::default()),
            ..Default::default()
        }
    }

    #[test]
    fn withdrawals_pre_canyon_none_on_both_sides() {
        let ts = CANYON_TIMESTAMP - 1;
        let body_absent: alloy_consensus::BlockBody<OpTxEnvelope> =
            alloy_consensus::BlockBody::default();

        assert!(BaseChain::withdrawals_valid(
            &header_at(ts, None),
            &body_absent
        ));
        // Header claims a root the fork doesn't have yet — reject.
        let forged = header_at(ts, Some(alloy_primitives::B256::repeat_byte(0x42)));
        assert!(!BaseChain::withdrawals_valid(&forged, &body_absent));
        // Body carries withdrawals without a header root — reject.
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, None),
            &empty_withdrawals_body()
        ));
        // Paired Some/Some before Canyon must also be rejected, even when
        // the roots match.
        let empty_root = alloy_consensus::constants::EMPTY_ROOT_HASH;
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, Some(empty_root)),
            &empty_withdrawals_body()
        ));
    }

    #[test]
    fn withdrawals_canyon_to_isthmus_roots_must_match() {
        let ts = ISTHMUS_TIMESTAMP - 1;
        let empty_root = alloy_consensus::constants::EMPTY_ROOT_HASH;

        assert!(BaseChain::withdrawals_valid(
            &header_at(ts, Some(empty_root)),
            &empty_withdrawals_body()
        ));
        // Forged header root pre-Isthmus must be rejected.
        let forged = header_at(ts, Some(alloy_primitives::B256::repeat_byte(0x42)));
        assert!(!BaseChain::withdrawals_valid(
            &forged,
            &empty_withdrawals_body()
        ));
        // Missing body withdrawals against a header root — reject.
        let body_absent: alloy_consensus::BlockBody<OpTxEnvelope> =
            alloy_consensus::BlockBody::default();
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, Some(empty_root)),
            &body_absent
        ));
        // Paired None/None after Canyon must be rejected.
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, None),
            &body_absent
        ));
    }

    #[test]
    fn withdrawals_post_isthmus_storage_root_and_empty_body() {
        let ts = ISTHMUS_TIMESTAMP;
        let storage_root = alloy_primitives::B256::repeat_byte(0x42);

        // Any header root is accepted (it commits to predeploy storage),
        // as long as the body withdrawals list is present and empty.
        assert!(BaseChain::withdrawals_valid(
            &header_at(ts, Some(storage_root)),
            &empty_withdrawals_body()
        ));

        // Header root missing post-Isthmus — reject.
        let body_absent: alloy_consensus::BlockBody<OpTxEnvelope> =
            alloy_consensus::BlockBody::default();
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, None),
            &empty_withdrawals_body()
        ));
        // Body withdrawals absent post-Isthmus — reject.
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, Some(storage_root)),
            &body_absent
        ));
        // Non-empty body withdrawals — reject.
        let nonempty = alloy_consensus::BlockBody::<OpTxEnvelope> {
            withdrawals: Some(alloy_eips::eip4895::Withdrawals::new(vec![
                alloy_eips::eip4895::Withdrawal::default(),
            ])),
            ..Default::default()
        };
        assert!(!BaseChain::withdrawals_valid(
            &header_at(ts, Some(storage_root)),
            &nonempty
        ));
    }

    #[test]
    fn azul_and_beryl_are_scheduled() {
        let spec = BaseChain::chain_spec();
        assert!(spec
            .inner
            .hardforks
            .fork(BaseHardfork::Azul)
            .active_at_timestamp(AZUL_TIMESTAMP));
        assert!(spec
            .inner
            .hardforks
            .fork(BaseHardfork::Beryl)
            .active_at_timestamp(BERYL_TIMESTAMP));
        assert!(!spec
            .inner
            .hardforks
            .fork(BaseHardfork::Beryl)
            .active_at_timestamp(BERYL_TIMESTAMP - 1));
    }
}
