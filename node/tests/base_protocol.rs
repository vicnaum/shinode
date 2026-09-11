//! Phase-1 compatibility checks; these do not enable Base in the node runtime.
//! Network setup adapts Sieve 914d29f (MIT OR Apache-2.0).

use alloy_consensus::{proofs, Receipt, TxReceipt};
use alloy_eips::eip2718::Typed2718;
use alloy_primitives::{Address, Bytes, Log, B256};
use eyre::{ensure, eyre, Result};
use futures::StreamExt;
use op_alloy_consensus::{OpDepositReceipt, OpPooledTransaction, OpReceipt, OpTxType};
use reth_chainspec::{EthChainSpec, EthereumHardfork, ForkCondition, Hardfork};
use reth_eth_wire::EthVersion;
use reth_eth_wire_types::{
    BasicNetworkPrimitives, BlockHashOrNumber, GetBlockBodies, GetBlockHeaders, GetReceipts,
    GetReceipts70, HeadersDirection,
};
use reth_network::{config::NetworkConfigBuilder, import::ProofOfStakeBlockImport};
use reth_network_api::{
    NetworkEvent, NetworkEventListenerProvider, PeerRequest, PeerRequestSender,
};
use reth_network_peers::NodeRecord;
use reth_optimism_chainspec::{OpChainSpec, BASE_MAINNET};
use reth_optimism_primitives::OpPrimitives;
use reth_primitives_traits::{serde_bincode_compat::SerdeBincodeCompat, SealedHeader};
use serde::Deserialize;
use std::{collections::BTreeSet, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::oneshot, time::timeout};

type BaseNet = BasicNetworkPrimitives<OpPrimitives, OpPooledTransaction>;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct ProtocolFixture {
    source_revision: String,
    chain_id: u64,
    genesis_hash: B256,
    azul_timestamp: u64,
    beryl_timestamp: u64,
    fork_hash: [u8; 4],
    discovery_protocol: String,
    bootnodes: Vec<String>,
}

fn fixture() -> Result<ProtocolFixture> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/base-mainnet-protocol.json"
    ))?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BaseFork {
    Azul,
    Beryl,
}

impl Hardfork for BaseFork {
    fn name(&self) -> &'static str {
        match self {
            Self::Azul => "Azul",
            Self::Beryl => "Beryl",
        }
    }
}

fn base_spec(f: &ProtocolFixture) -> Arc<OpChainSpec> {
    let mut spec = (**BASE_MAINNET).clone();
    spec.inner.hardforks.insert(
        EthereumHardfork::Osaka,
        ForkCondition::Timestamp(f.azul_timestamp),
    );
    spec.inner
        .hardforks
        .insert(BaseFork::Azul, ForkCondition::Timestamp(f.azul_timestamp));
    spec.inner
        .hardforks
        .insert(BaseFork::Beryl, ForkCondition::Timestamp(f.beryl_timestamp));
    Arc::new(spec)
}

#[test]
fn pinned_spec_matches_base_beryl() -> Result<()> {
    let f = fixture()?;
    let spec = base_spec(&f);
    ensure!(spec.chain().id() == f.chain_id, "chain ID mismatch");
    ensure!(spec.genesis_hash() == f.genesis_hash, "genesis mismatch");
    ensure!(
        spec.latest_fork_id().hash.0 == f.fork_hash,
        "Beryl fork hash mismatch"
    );
    ensure!(spec.latest_fork_id().next == 0, "unexpected scheduled fork");
    ensure!(f.discovery_protocol == "basev0", "wrong discovery protocol");
    ensure!(f.bootnodes.len() == 10, "incomplete bootnode fixture");
    for node in &f.bootnodes {
        node.parse::<NodeRecord>()?;
    }
    Ok(())
}

#[test]
fn op_receipts_preserve_deposit_fields_through_bincode() -> Result<()> {
    let inner = Receipt {
        status: true.into(),
        cumulative_gas_used: 42_000,
        logs: vec![Log::new_unchecked(
            Address::repeat_byte(0x11),
            vec![B256::repeat_byte(0x22)],
            Bytes::from_static(&[0x33, 0x44]),
        )],
    };
    // Exercise absent fields, Regolith nonce, and Canyon nonce + version.
    for (nonce, version) in [(None, None), (Some(7), None), (Some(7), Some(1))] {
        let receipts = vec![
            OpReceipt::Deposit(OpDepositReceipt {
                inner: inner.clone(),
                deposit_nonce: nonce,
                deposit_receipt_version: version,
            }),
            OpReceipt::Eip1559(inner.clone()),
        ];
        let encoded = bincode::serialize(
            &receipts
                .iter()
                .map(SerdeBincodeCompat::as_repr)
                .collect::<Vec<_>>(),
        )?;
        let repr: Vec<<OpReceipt as SerdeBincodeCompat>::BincodeRepr<'_>> =
            bincode::deserialize(&encoded)?;
        let decoded: Vec<_> = repr.into_iter().map(OpReceipt::from_repr).collect();
        ensure!(
            decoded == receipts,
            "receipt round-trip lost data: nonce={nonce:?}, version={version:?}"
        );
    }
    Ok(())
}

#[test]
fn supported_transaction_types_are_explicit() {
    for ty in [0u8, 1, 2, 4, 0x7e] {
        assert!(OpTxType::try_from(ty).is_ok());
    }
    // Current Base source gates 0x79 behind Zenith. Never silently decode it
    // as an ordinary transaction with this pre-Zenith dependency set.
    assert!(OpTxType::try_from(0x79u8).is_err());
}

async fn receipts(
    messages: &PeerRequestSender<PeerRequest<BaseNet>>,
    version: EthVersion,
    hash: B256,
) -> Result<Vec<OpReceipt>> {
    let blocks = match version {
        EthVersion::Eth70 => {
            let (tx, rx) = oneshot::channel();
            messages
                .try_send(PeerRequest::GetReceipts70 {
                    request: GetReceipts70 {
                        first_block_receipt_index: 0,
                        block_hashes: vec![hash],
                    },
                    response: tx,
                })
                .map_err(|e| eyre!("send receipts: {e:?}"))?;
            let response = timeout(REQUEST_TIMEOUT, rx).await???;
            ensure!(
                !response.last_block_incomplete,
                "incomplete eth/70 block receipts"
            );
            response.receipts
        }
        EthVersion::Eth69 => {
            let (tx, rx) = oneshot::channel();
            messages
                .try_send(PeerRequest::GetReceipts69 {
                    request: GetReceipts(vec![hash]),
                    response: tx,
                })
                .map_err(|e| eyre!("send receipts: {e:?}"))?;
            timeout(REQUEST_TIMEOUT, rx).await???.0
        }
        _ => {
            let (tx, rx) = oneshot::channel();
            messages
                .try_send(PeerRequest::GetReceipts {
                    request: GetReceipts(vec![hash]),
                    response: tx,
                })
                .map_err(|e| eyre!("send receipts: {e:?}"))?;
            timeout(REQUEST_TIMEOUT, rx)
                .await???
                .0
                .into_iter()
                .map(|block| block.into_iter().map(|r| r.receipt).collect())
                .collect()
        }
    };
    ensure!(blocks.len() == 1, "expected one receipt block");
    blocks
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("missing receipts"))
}

async fn probe_block(
    messages: &PeerRequestSender<PeerRequest<BaseNet>>,
    version: EthVersion,
    hash: B256,
    spec: &OpChainSpec,
) -> Result<serde_json::Value> {
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders {
            request: GetBlockHeaders {
                start_block: BlockHashOrNumber::Hash(hash),
                limit: 1,
                skip: 0,
                direction: HeadersDirection::Rising,
            },
            response: tx,
        })
        .map_err(|e| eyre!("send header: {e:?}"))?;
    let headers = timeout(REQUEST_TIMEOUT, rx).await???.0;
    ensure!(headers.len() == 1, "expected one header");
    let header = SealedHeader::seal_slow(
        headers
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("missing header"))?,
    );
    ensure!(header.hash() == hash, "peer head hash mismatch");
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockBodies {
            request: GetBlockBodies(vec![hash]),
            response: tx,
        })
        .map_err(|e| eyre!("send body: {e:?}"))?;
    let bodies = timeout(REQUEST_TIMEOUT, rx).await???.0;
    ensure!(bodies.len() == 1, "expected one body");
    let body = bodies
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("missing body"))?;
    let receipts = receipts(messages, version, hash).await?;
    ensure!(
        body.transactions.len() == receipts.len(),
        "transaction/receipt count mismatch"
    );
    ensure!(
        proofs::calculate_transaction_root(&body.transactions) == header.transactions_root,
        "transaction root mismatch"
    );
    ensure!(
        reth_optimism_consensus::calculate_receipt_root_no_memo_optimism(
            &receipts,
            spec,
            header.timestamp
        ) == header.receipts_root,
        "receipt root mismatch"
    );
    let types: BTreeSet<_> = body.transactions.iter().map(Typed2718::ty).collect();
    ensure!(types.contains(&0x7e), "expected a Base deposit transaction");
    Ok(serde_json::json!({
        "block_number": header.number, "block_hash": hash,
        "block_timestamp": header.timestamp, "transactions": body.transactions.len(),
        "transaction_types": types, "receipts": receipts.len(),
        "logs": receipts.iter().map(|r| r.logs().len()).sum::<usize>(),
        "transaction_root_verified": true, "receipt_root_verified": true,
    }))
}

/// Explicitly opt in; no networking in normal cargo test runs.
#[tokio::test]
#[ignore = "live Base P2P probe; set SHINODE_BASE_PROBE_OUTPUT to save evidence"]
async fn live_base_protocol_probe() -> Result<()> {
    let f = fixture()?;
    let spec = base_spec(&f);
    let listen: SocketAddr = "0.0.0.0:0".parse()?;
    let mut discovery = discv5::ConfigBuilder::new(discv5::ListenConfig::from_ip(listen.ip(), 0));
    discovery.protocol_identity(discv5::ProtocolIdentity {
        protocol_id: f.discovery_protocol.as_bytes().try_into()?,
        ..Default::default()
    });
    let bootnodes = f
        .bootnodes
        .iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<NodeRecord>, _>>()?;
    let config = NetworkConfigBuilder::<BaseNet>::new(reth_network::config::rng_secret_key())
        .listener_addr(listen)
        .discovery_addr(listen)
        .boot_nodes(bootnodes.clone())
        .disable_discv4_discovery()
        .discovery_v5(
            reth_discv5::Config::builder(listen)
                .discv5_config(discovery.build())
                .add_unsigned_boot_nodes(bootnodes),
        )
        .disable_tx_gossip(true)
        .block_import(Box::new(ProofOfStakeBlockImport::default()))
        .build(reth_storage_api::noop::NoopProvider::<
            OpChainSpec,
            OpPrimitives,
        >::new(Arc::clone(&spec)));
    let handle = config.start_network().await?;
    let mut events = handle.event_listener();
    let mut failures = BTreeSet::new();
    let report = timeout(Duration::from_secs(90), async {
        while let Some(event) = events.next().await {
            if let NetworkEvent::ActivePeerSession { info, messages } = event {
                ensure!(info.status.genesis == f.genesis_hash, "wrong peer genesis");
                ensure!(info.status.chain.id() == f.chain_id, "wrong peer chain ID");
                ensure!(
                    info.status.forkid == spec.latest_fork_id(),
                    "peer has a different active fork; recheck protocol support"
                );
                match probe_block(&messages, info.version, info.status.blockhash, &spec).await {
                    Ok(mut report) => {
                        let observed_at = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs();
                        ensure!(
                            report["block_timestamp"]
                                .as_u64()
                                .is_some_and(|ts| observed_at.abs_diff(ts) <= 300),
                            "peer head is not recent"
                        );
                        ensure!(
                            report["block_timestamp"]
                                .as_u64()
                                .is_some_and(|ts| ts >= f.beryl_timestamp),
                            "peer predates Beryl"
                        );
                        report["peer_id"] = serde_json::json!(info.peer_id);
                        report["eth_version"] = serde_json::json!(info.version.to_string());
                        report["fork_id"] = serde_json::json!(info.status.forkid);
                        report["source_revision"] = serde_json::json!(f.source_revision);
                        report["observed_at_unix"] = serde_json::json!(observed_at);
                        return Ok(report);
                    }
                    Err(error) => {
                        failures.insert(error.to_string());
                    }
                }
            }
        }
        Err(eyre!("network event stream ended"))
    })
    .await
    .map_err(|_| eyre!("no serving Base peer within 90s; failures: {failures:?}"))??;
    if let Some(path) = std::env::var_os("SHINODE_BASE_PROBE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report)?)?;
    }
    Ok(())
}
