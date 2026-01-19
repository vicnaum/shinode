//! P2P subsystem.

use crate::sync::{BlockPayload, BlockPayloadSource};
use alloy_primitives::B256;
use async_trait::async_trait;
use eyre::{eyre, Result, WrapErr};
use futures::StreamExt;
use reth_chainspec::MAINNET;
use reth_eth_wire::{EthNetworkPrimitives, EthVersion};
use reth_eth_wire_types::{
    BlockHashOrNumber, GetBlockBodies, GetBlockHeaders, GetReceipts, GetReceipts70,
    HeadersDirection,
};
use reth_network::config::{rng_secret_key, NetworkConfigBuilder};
use reth_network::import::ProofOfStakeBlockImport;
use reth_network::NetworkHandle;
use reth_network_api::{NetworkEvent, NetworkEventListenerProvider, PeerId, PeerRequest, PeerRequestSender};
use reth_primitives_traits::{Header, SealedHeader};
use reth_ethereum_primitives::Receipt;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

/// Identifier for a peer in the selection pool.
#[allow(dead_code)]
pub type SelectorPeerId = String;

/// Peer selector abstraction.
#[allow(dead_code)]
pub trait PeerSelector: Send + Sync {
    fn next_peer(&mut self) -> Option<SelectorPeerId>;
}

/// Round-robin peer selector for test scaffolding.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RoundRobinPeerSelector {
    peers: Vec<SelectorPeerId>,
    next_index: usize,
}

#[allow(dead_code)]
impl RoundRobinPeerSelector {
    pub fn new(peers: Vec<SelectorPeerId>) -> Self {
        Self { peers, next_index: 0 }
    }
}

impl PeerSelector for RoundRobinPeerSelector {
    fn next_peer(&mut self) -> Option<SelectorPeerId> {
        if self.peers.is_empty() {
            return None;
        }
        let peer = self.peers[self.next_index].clone();
        self.next_index = (self.next_index + 1) % self.peers.len();
        Some(peer)
    }
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const REQUEST_CHUNK_SIZE: usize = 25;

/// Active peer session information used for requests.
#[derive(Clone, Debug)]
pub struct NetworkPeer {
    pub peer_id: PeerId,
    pub eth_version: EthVersion,
    pub messages: PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
    pub head_number: u64,
}

/// Wrapper that keeps the network handle alive.
#[derive(Debug)]
pub struct NetworkSession {
    pub _handle: NetworkHandle<EthNetworkPrimitives>,
    pub peer: NetworkPeer,
}

/// Start the devp2p network and wait for the first compatible peer.
pub async fn connect_mainnet_peer() -> Result<NetworkSession> {
    let secret_key = rng_secret_key();
    let net_config = NetworkConfigBuilder::<EthNetworkPrimitives>::new(secret_key)
        .mainnet_boot_nodes()
        .with_unused_ports()
        .disable_tx_gossip(true)
        .block_import(Box::new(ProofOfStakeBlockImport::default()))
        .build_with_noop_provider(MAINNET.clone());

    let handle = net_config
        .start_network()
        .await
        .wrap_err("failed to start p2p network")?;
    let peer = wait_for_peer(&handle).await?;

    Ok(NetworkSession { _handle: handle, peer })
}

/// P2P-backed block payload source.
#[derive(Clone, Debug)]
pub struct NetworkBlockPayloadSource {
    peer: NetworkPeer,
}

impl NetworkBlockPayloadSource {
    pub fn new(peer: NetworkPeer) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl BlockPayloadSource for NetworkBlockPayloadSource {
    async fn head(&self) -> Result<u64> {
        Ok(self.peer.head_number)
    }

    async fn blocks_by_number(&self, range: std::ops::RangeInclusive<u64>) -> Result<Vec<BlockPayload>> {
        let start = *range.start();
        let end = *range.end();
        let count = (end - start + 1) as usize;

        let headers = request_headers(&self.peer, start, count).await?;
        if headers.len() != count {
            return Err(eyre!(
                "header count mismatch: expected {}, got {}",
                count,
                headers.len()
            ));
        }

        let mut header_stubs = Vec::with_capacity(count);
        let mut hashes = Vec::with_capacity(count);
        for header in headers {
            let hash = SealedHeader::seal_slow(header.clone()).hash();
            header_stubs.push(crate::chain::HeaderStub {
                number: header.number,
                hash,
                parent_hash: header.parent_hash,
            });
            hashes.push(hash);
        }

        let bodies = request_bodies_chunked(&self.peer, &hashes).await?;
        let receipts = request_receipts_chunked(&self.peer, &hashes).await?;

        let mut payloads = Vec::with_capacity(count);
        for ((header, body), receipts) in header_stubs
            .into_iter()
            .zip(bodies.into_iter())
            .zip(receipts.into_iter())
        {
            let tx_hashes = body
                .transactions
                .iter()
                .map(|tx| *tx.hash())
                .collect::<Vec<_>>();
            if tx_hashes.len() != receipts.len() {
                return Err(eyre!(
                    "tx/receipt mismatch for block {}: {} txs vs {} receipts",
                    header.number,
                    tx_hashes.len(),
                    receipts.len()
                ));
            }
            payloads.push(BlockPayload {
                header,
                tx_hashes,
                receipts,
            });
        }

        Ok(payloads)
    }
}

async fn wait_for_peer(handle: &NetworkHandle<EthNetworkPrimitives>) -> Result<NetworkPeer> {
    let mut events = handle.event_listener();
    while let Some(event) = events.next().await {
        if let NetworkEvent::ActivePeerSession { info, messages } = event {
            if info.status.genesis != MAINNET.genesis_hash() {
                continue;
            }
            let head_number =
                request_head_number(info.peer_id, info.status.blockhash, &messages).await?;
            return Ok(NetworkPeer {
                peer_id: info.peer_id,
                eth_version: info.version,
                messages,
                head_number,
            });
        }
    }
    Err(eyre!("no active peer sessions available"))
}

async fn request_head_number(
    peer_id: PeerId,
    head_hash: B256,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<u64> {
    let headers = request_headers_by_hash(peer_id, head_hash, messages).await?;
    let header = headers
        .get(0)
        .ok_or_else(|| eyre!("empty header response for head"))?;
    Ok(header.number)
}

async fn request_headers(
    peer: &NetworkPeer,
    start_block: u64,
    limit: usize,
) -> Result<Vec<Header>> {
    request_headers_by_number(peer.peer_id, start_block, limit, &peer.messages)
        .await
}

async fn request_headers_by_number(
    peer_id: PeerId,
    start_block: u64,
    limit: usize,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<Vec<Header>> {
    let request = GetBlockHeaders {
        start_block: BlockHashOrNumber::Number(start_block),
        limit: limit as u64,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| eyre!("failed to send header request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("header request to {peer_id:?} timed out"))??;
    let headers = response.map_err(|err| eyre!("header response error from {peer_id:?}: {err:?}"))?;
    Ok(headers.0)
}

async fn request_headers_by_hash(
    peer_id: PeerId,
    hash: B256,
    messages: &PeerRequestSender<PeerRequest<EthNetworkPrimitives>>,
) -> Result<Vec<Header>> {
    let request = GetBlockHeaders {
        start_block: BlockHashOrNumber::Hash(hash),
        limit: 1,
        skip: 0,
        direction: HeadersDirection::Rising,
    };
    let (tx, rx) = oneshot::channel();
    messages
        .try_send(PeerRequest::GetBlockHeaders { request, response: tx })
        .map_err(|err| eyre!("failed to send header request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("header request to {peer_id:?} timed out"))??;
    let headers = response.map_err(|err| eyre!("header response error from {peer_id:?}: {err:?}"))?;
    Ok(headers.0)
}

async fn request_bodies(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<reth_ethereum_primitives::BlockBody>> {
    let request = GetBlockBodies::from(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetBlockBodies { request, response: tx })
        .map_err(|err| eyre!("failed to send body request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("body request to {:?} timed out", peer.peer_id))??;
    let bodies = response.map_err(|err| eyre!("body response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(bodies.0)
}

async fn request_bodies_chunked(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<reth_ethereum_primitives::BlockBody>> {
    let mut bodies = Vec::with_capacity(hashes.len());
    for chunk in hashes.chunks(REQUEST_CHUNK_SIZE) {
        let chunk_bodies = request_bodies(peer, chunk).await?;
        if chunk_bodies.len() != chunk.len() {
            return Err(eyre!(
                "body count mismatch: expected {}, got {}",
                chunk.len(),
                chunk_bodies.len()
            ));
        }
        bodies.extend(chunk_bodies);
    }
    Ok(bodies)
}

async fn request_receipts(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    match peer.eth_version {
        EthVersion::Eth70 => request_receipts70(peer, hashes).await,
        EthVersion::Eth69 => request_receipts69(peer, hashes).await,
        _ => request_receipts_legacy(peer, hashes).await,
    }
}

async fn request_receipts_chunked(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    if hashes.is_empty() {
        return Ok(Vec::new());
    }
    let mut results: Vec<Option<Vec<Receipt>>> = vec![None; hashes.len()];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((0usize, hashes.len()));

    while let Some((start, end)) = queue.pop_front() {
        let slice = &hashes[start..end];
        let receipts = request_receipts(peer, slice).await?;
        if receipts.len() == slice.len() {
            for (offset, receipts) in receipts.into_iter().enumerate() {
                results[start + offset] = Some(receipts);
            }
            continue;
        }

        if slice.len() == 1 {
            return Err(eyre!(
                "receipt count mismatch: expected 1, got {}",
                receipts.len()
            ));
        }

        let mid = start + slice.len() / 2;
        queue.push_back((start, mid));
        queue.push_back((mid, end));
    }

    let mut output = Vec::with_capacity(hashes.len());
    for receipts in results {
        output.push(
            receipts.ok_or_else(|| eyre!("missing receipts for requested hash"))?,
        );
    }
    Ok(output)
}

async fn request_receipts_legacy(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0
        .into_iter()
        .map(|block| block.into_iter().map(|r| r.receipt).collect())
        .collect())
}

async fn request_receipts69(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts(hashes.to_vec());
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts69 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts69 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts69 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts69 response error from {:?}: {err:?}", peer.peer_id))?;
    Ok(receipts.0)
}

async fn request_receipts70(
    peer: &NetworkPeer,
    hashes: &[B256],
) -> Result<Vec<Vec<Receipt>>> {
    let request = GetReceipts70 {
        first_block_receipt_index: 0,
        block_hashes: hashes.to_vec(),
    };
    let (tx, rx) = oneshot::channel();
    peer.messages
        .try_send(PeerRequest::GetReceipts70 { request, response: tx })
        .map_err(|err| eyre!("failed to send receipts70 request: {err:?}"))?;
    let response = timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| eyre!("receipts70 request to {:?} timed out", peer.peer_id))??;
    let receipts = response.map_err(|err| eyre!("receipts70 response error from {:?}: {err:?}", peer.peer_id))?;
    if receipts.last_block_incomplete {
        return Err(eyre!("partial receipts in eth/70 response"));
    }
    Ok(receipts.receipts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_cycles_peers() {
        let mut selector = RoundRobinPeerSelector::new(vec![
            "peer-a".to_string(),
            "peer-b".to_string(),
            "peer-c".to_string(),
        ]);
        let selected = vec![
            selector.next_peer(),
            selector.next_peer(),
            selector.next_peer(),
            selector.next_peer(),
        ];
        assert_eq!(
            selected,
            vec![
                Some("peer-a".to_string()),
                Some("peer-b".to_string()),
                Some("peer-c".to_string()),
                Some("peer-a".to_string()),
            ]
        );
    }
}
