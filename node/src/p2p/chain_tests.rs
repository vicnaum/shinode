use super::*;
use crate::chain::BaseChain;
use alloy_consensus::{proofs, TxReceipt};
use alloy_primitives::Sealable;
use eyre::ensure;
use op_alloy_consensus::{OpDepositReceipt, OpTxEnvelope, TxDeposit};
use reth_eth_wire_types::{
    BlockBodies, BlockHeaders, Receipts, Receipts69, Receipts70, UnifiedStatus,
};
use reth_optimism_primitives::OpReceipt;
use tokio::sync::mpsc;

fn mock_peer<C: ChainTypes>(
    version: EthVersion,
) -> (NetworkPeer<C>, mpsc::Receiver<PeerRequest<C::Net>>) {
    let peer_id = PeerId::random();
    let (tx, rx) = mpsc::channel(8);
    (
        NetworkPeer {
            peer_id,
            eth_version: version,
            messages: PeerRequestSender::new(peer_id, tx),
            head_number: 0,
            earliest_block: None,
            latest_block: None,
        },
        rx,
    )
}

fn deposit_receipt() -> OpReceipt {
    OpReceipt::Deposit(OpDepositReceipt {
        inner: alloy_consensus::Receipt {
            status: true.into(),
            cumulative_gas_used: 21_000,
            logs: Vec::new(),
        },
        deposit_nonce: Some(7),
        deposit_receipt_version: Some(1),
    })
}

async fn receipt_versions<C: ChainTypes>(receipt: C::Receipt) -> Result<()> {
    for version in [EthVersion::Eth68, EthVersion::Eth69, EthVersion::Eth70] {
        let (peer, mut rx) = mock_peer::<C>(version);
        let expected = vec![receipt.clone()];
        let served = expected.clone();
        let server = tokio::spawn(async move {
            let request = rx.recv().await.ok_or_else(|| eyre!("missing request"))?;
            match request {
                PeerRequest::GetReceipts { response, .. } => {
                    ensure!(version == EthVersion::Eth68, "incorrect legacy request");
                    let _ = response.send(Ok(Receipts(vec![served
                        .into_iter()
                        .map(TxReceipt::into_with_bloom)
                        .collect()])));
                }
                PeerRequest::GetReceipts69 { response, .. } => {
                    ensure!(version == EthVersion::Eth69, "incorrect eth/69 request");
                    let _ = response.send(Ok(Receipts69(vec![served])));
                }
                PeerRequest::GetReceipts70 { request, response } => {
                    ensure!(version == EthVersion::Eth70, "incorrect eth/70 request");
                    ensure!(
                        request.first_block_receipt_index == 0,
                        "nonzero receipt offset"
                    );
                    let _ = response.send(Ok(Receipts70 {
                        last_block_incomplete: false,
                        receipts: vec![served],
                    }));
                }
                _ => return Err(eyre!("unexpected request")),
            }
            Ok(())
        });
        // A short response preserves complete blocks and marks the tail missing.
        let response =
            request_receipts_chunked_partial_with_stats(&peer, &[B256::ZERO, B256::repeat_byte(1)])
                .await?;
        ensure!(
            response.requests == 1,
            "partial response should stop the batch"
        );
        ensure!(
            response.results == vec![Some(expected), None],
            "receipts changed across wire versions"
        );
        server.await??;
    }
    Ok(())
}

#[tokio::test]
async fn base_receipts_preserve_deposit_fields_in_all_versions() -> Result<()> {
    receipt_versions::<BaseChain>(deposit_receipt()).await
}

#[tokio::test]
async fn ethereum_receipts_work_in_all_versions() -> Result<()> {
    receipt_versions::<EthereumChain>(reth_ethereum_primitives::Receipt::default()).await
}

#[tokio::test]
async fn eth70_incomplete_last_block_is_missing() -> Result<()> {
    // Empty, single-incomplete and complete-prefix responses must all be safe to requeue.
    for count in 0..=2 {
        let (peer, mut rx) = mock_peer::<BaseChain>(EthVersion::Eth70);
        let server = tokio::spawn(async move {
            if let Some(PeerRequest::GetReceipts70 { response, .. }) = rx.recv().await {
                let _ = response.send(Ok(Receipts70 {
                    last_block_incomplete: true,
                    receipts: vec![vec![deposit_receipt()]; count],
                }));
            }
        });
        let response =
            request_receipts_chunked_partial_with_stats(&peer, &[B256::ZERO, B256::repeat_byte(1)])
                .await?;
        ensure!(
            response.results[1].is_none(),
            "incomplete tail was accepted"
        );
        ensure!(
            response.results[0].is_some() == (count == 2),
            "complete prefix was lost"
        );
        ensure!(response.requests == 1, "unexpected continuation");
        server.await?;
    }
    Ok(())
}

#[tokio::test]
async fn base_fetch_returns_typed_deposit_payload() -> Result<()> {
    let (peer, mut rx) = mock_peer::<BaseChain>(EthVersion::Eth69);
    let body = BlockBody {
        transactions: vec![OpTxEnvelope::Deposit(TxDeposit::default().seal_slow())],
        ..Default::default()
    };
    let expected = body.clone();
    let mut payload = BlockPayload::<BaseChain> {
        header: Header {
            number: 42,
            ..Default::default()
        },
        body: body.clone(),
        receipts: vec![deposit_receipt()],
    };
    crate::sync::validation::tests::commit_payload(&mut payload);
    let server = tokio::spawn(async move {
        for _ in 0..3 {
            match rx.recv().await {
                Some(PeerRequest::GetBlockHeaders { response, .. }) => {
                    let _ = response.send(Ok(BlockHeaders(vec![payload.header.clone()])));
                }
                Some(PeerRequest::GetBlockBodies { response, .. }) => {
                    let _ = response.send(Ok(BlockBodies(vec![body.clone()])));
                }
                Some(PeerRequest::GetReceipts69 { response, .. }) => {
                    let _ = response.send(Ok(Receipts69(vec![vec![deposit_receipt()]])));
                }
                _ => return Err(eyre!("unexpected request")),
            }
        }
        Ok(())
    });
    let outcome = fetch_payloads_for_peer(&peer, 42..=43).await?;
    ensure!(
        outcome.missing_blocks == vec![43],
        "partial header tail not missing"
    );
    ensure!(outcome.payloads.len() == 1, "expected one payload");
    ensure!(outcome.payloads[0].body == expected, "deposit body changed");
    ensure!(
        outcome.payloads[0].receipts == vec![deposit_receipt()],
        "deposit receipt changed"
    );
    server.await??;
    Ok(())
}

#[test]
fn session_identity_and_advertised_history() -> Result<()> {
    let (peer, _) = mock_peer::<BaseChain>(EthVersion::Eth69);
    let spec = BaseChain::chain_spec();
    let mut info = SessionInfo {
        peer_id: peer.peer_id,
        remote_addr: "127.0.0.1:30303".parse()?,
        client_version: "test".into(),
        capabilities: Arc::new(reth_eth_wire_types::Capabilities::new(Vec::new())),
        status: Arc::new(UnifiedStatus {
            chain: spec.chain(),
            genesis: spec.genesis_hash(),
            earliest_block: Some(100),
            latest_block: Some(200),
            ..Default::default()
        }),
        version: EthVersion::Eth69,
        peer_kind: PeerKind::Static,
    };
    let pool = PeerPool::<BaseChain>::new();
    let accepted = NetworkPeer::<BaseChain>::from_session(&info, peer.messages.clone())
        .ok_or_else(|| eyre!("valid peer rejected"))?;
    pool.add_peer(accepted);
    let snapshot = pool.snapshot();
    ensure!(
        snapshot[0].earliest_block == Some(100) && snapshot[0].latest_block == Some(200),
        "history range lost"
    );
    ensure!(
        snapshot[0].head_number == 0,
        "advertised head was trusted without probing"
    );
    Arc::make_mut(&mut info.status).genesis = EthereumChain::chain_spec().genesis_hash();
    ensure!(
        NetworkPeer::<BaseChain>::from_session(&info, peer.messages.clone()).is_none(),
        "foreign genesis accepted"
    );
    Arc::make_mut(&mut info.status).genesis = spec.genesis_hash();
    Arc::make_mut(&mut info.status).chain = EthereumChain::chain_spec().chain();
    ensure!(
        NetworkPeer::<BaseChain>::from_session(&info, peer.messages).is_none(),
        "foreign chain ID accepted"
    );
    Ok(())
}

#[tokio::test]
async fn head_probes_reject_wrong_headers() -> Result<()> {
    let (peer, mut rx) = mock_peer::<BaseChain>(EthVersion::Eth69);
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            if let Some(PeerRequest::GetBlockHeaders { response, .. }) = rx.recv().await {
                let _ = response.send(Ok(BlockHeaders(vec![Header {
                    number: 999,
                    ..Default::default()
                }])));
            }
        }
    });
    ensure!(
        request_head_number::<BaseChain>(peer.peer_id, B256::ZERO, &peer.messages)
            .await
            .is_err(),
        "wrong head hash accepted"
    );
    let pool = PeerPool::<BaseChain>::new();
    pool.add_peer(peer.clone());
    ensure!(
        re_probe_peer_head(&pool, &peer, 42).await.is_err(),
        "wrong header number accepted"
    );
    ensure!(
        pool.get_peer_head(peer.peer_id) == Some(0),
        "unverified head advanced"
    );
    server.await?;
    Ok(())
}

#[tokio::test]
async fn header_requests_reject_broken_parent_links() -> Result<()> {
    let (peer, mut rx) = mock_peer::<BaseChain>(EthVersion::Eth69);
    let server = tokio::spawn(async move {
        if let Some(PeerRequest::GetBlockHeaders { response, .. }) = rx.recv().await {
            let _ = response.send(Ok(BlockHeaders(vec![
                Header {
                    number: 42,
                    ..Default::default()
                },
                Header {
                    number: 43,
                    parent_hash: B256::ZERO,
                    ..Default::default()
                },
            ])));
        }
    });
    ensure!(
        request_headers_batch(&peer, 42, 2).await.is_err(),
        "disconnected headers accepted"
    );
    server.await?;
    Ok(())
}

async fn verify_live_block<C: ChainTypes>(
    peer: &NetworkPeer<C>,
    require_recent_head: bool,
) -> Result<serde_json::Value> {
    let number = peer.head_number - 2;
    let fetched = fetch_payloads_for_peer(peer, number..=number).await?;
    let block = fetched.payloads.first().ok_or_else(|| {
        eyre!(
            "block {number} missing from {:?} peer (history {:?}..{:?}, requests {:?})",
            peer.eth_version,
            peer.earliest_block,
            peer.latest_block,
            fetched.fetch_stats
        )
    })?;
    ensure!(
        proofs::calculate_transaction_root(&block.body.transactions)
            == block.header.transactions_root,
        "transaction root mismatch"
    );
    ensure!(
        C::receipts_root(&block.receipts, &block.header) == block.header.receipts_root,
        "receipt root mismatch"
    );
    ensure!(
        C::withdrawals_valid(&block.header, &block.body),
        "withdrawals mismatch"
    );
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let head_age_seconds = now.abs_diff(block.header.timestamp);
    if require_recent_head {
        ensure!(head_age_seconds < 300, "peer head is stale");
    }
    Ok(serde_json::json!({
        "chain": C::NAME, "chain_id": C::chain_spec().chain().id(),
        "genesis": C::chain_spec().genesis_hash(), "peer_id": peer.peer_id,
        "eth_version": peer.eth_version.to_string(), "block_number": number,
        "block_hash": block.header.hash_slow(), "observed_at_unix": now,
        "block_timestamp": block.header.timestamp, "head_age_seconds": head_age_seconds,
        "transactions": block.body.transactions.len(), "receipts": block.receipts.len(),
        "logs": block.receipts.iter().map(|r| r.logs().len()).sum::<usize>(),
        "earliest_block": peer.earliest_block, "latest_block": peer.latest_block,
        "transaction_root_verified": true, "receipt_root_verified": true,
        "withdrawals_verified": true,
    }))
}

/// Exercise the production connector and fetch helpers without opening storage.
#[expect(
    clippy::print_stdout,
    reason = "opt-in connectivity diagnostic emits verifiable evidence"
)]
async fn live_connection<C: ChainTypes>(require_recent_head: bool) -> Result<()> {
    let mut failures = std::collections::BTreeSet::new();
    let mut attempted = std::collections::HashSet::new();
    let trusted_peers = match std::env::var_os("SHINODE_PROBE_TRUSTED_PEERS_FILE") {
        Some(path) => std::fs::read_to_string(path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::parse)
            .collect::<Result<Vec<reth_network_peers::TrustedPeer>, _>>()?,
        None => Vec::new(),
    };
    timeout(Duration::from_secs(90), async {
        let session = connect_peers::<C>(None, 0, &trusted_peers).await?;
        loop {
            for peer in session.pool.snapshot() {
                if peer.head_number < 2 || !attempted.insert((peer.peer_id, peer.head_number)) {
                    continue;
                }
                match verify_live_block(&peer, require_recent_head).await {
                    Ok(report) => {
                        println!("{report}");
                        return Ok(());
                    }
                    Err(error) => {
                        failures.insert(error.to_string());
                    }
                }
            }
            sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .map_err(|_| {
        eyre!(
            "{} connectivity timed out after 90s; failures: {failures:?}",
            C::NAME
        )
    })?
}

#[tokio::test]
#[ignore = "live Base connectivity through production P2P; bounded to 90 seconds"]
async fn live_base_connection() -> Result<()> {
    live_connection::<BaseChain>(true).await
}

#[tokio::test]
#[ignore = "live Ethereum connectivity through production P2P; bounded to 90 seconds"]
async fn live_ethereum_connection() -> Result<()> {
    // Ethereum's pre-eth/69 status can advertise an old head. This regression
    // probe verifies a served payload and reports its age, without tip discovery.
    live_connection::<EthereumChain>(false).await
}

async fn omitted_middle_payload_is_not_associated_with_wrong_header<C: ChainTypes + Clone>(
    first: BlockPayload<C>,
) -> Result<()> {
    let mut second = first.clone();
    second.header.number += 1;
    second.header.parent_hash = first.header.hash_slow();
    let mut third = second.clone();
    third.header.number += 1;
    third.header.parent_hash = second.header.hash_slow();
    third.body.transactions.clear();
    third.receipts.clear();
    crate::sync::validation::tests::commit_payload(&mut third);
    let start = first.header.number;
    let (peer, mut rx) = mock_peer::<C>(EthVersion::Eth69);
    let server = tokio::spawn(async move {
        for _ in 0..3 {
            match rx.recv().await {
                Some(PeerRequest::GetBlockHeaders { response, .. }) => {
                    let _ = response.send(Ok(BlockHeaders(vec![
                        first.header.clone(),
                        second.header.clone(),
                        third.header.clone(),
                    ])));
                }
                Some(PeerRequest::GetBlockBodies { response, .. }) => {
                    let _ = response.send(Ok(BlockBodies(vec![
                        first.body.clone(),
                        third.body.clone(),
                    ])));
                }
                Some(PeerRequest::GetReceipts69 { response, .. }) => {
                    let _ = response.send(Ok(Receipts69(vec![
                        first.receipts.clone(),
                        third.receipts.clone(),
                    ])));
                }
                _ => return Err(eyre!("unexpected request")),
            }
        }
        Ok(())
    });
    let fetched = fetch_payloads_for_peer(&peer, start..=start + 2).await?;
    ensure!(
        fetched.payloads.len() == 1 && fetched.payloads[0].header.number == start,
        "misassociated payload was accepted"
    );
    ensure!(
        fetched.missing_blocks == vec![start + 1, start + 2],
        "missing blocks must be retried"
    );
    ensure!(
        fetched.invalid_blocks == vec![start + 1],
        "commitment rejection must be distinguished from absent data"
    );
    server.await??;
    Ok(())
}

#[tokio::test]
async fn base_partial_response_validates_before_accepting_payload() -> Result<()> {
    omitted_middle_payload_is_not_associated_with_wrong_header(
        crate::sync::validation::tests::base_payload(crate::sync::validation::tests::CANYON),
    )
    .await
}

#[tokio::test]
async fn ethereum_partial_response_validates_before_accepting_payload() -> Result<()> {
    omitted_middle_payload_is_not_associated_with_wrong_header(
        crate::sync::validation::tests::ethereum_payload(),
    )
    .await
}

#[tokio::test]
async fn legacy_peer_bloom_is_recomputed_from_logs() -> Result<()> {
    let payload =
        crate::sync::validation::tests::base_payload(crate::sync::validation::tests::CANYON);
    let expected = payload.receipts.clone();
    let number = payload.header.number;
    let (peer, mut rx) = mock_peer::<BaseChain>(EthVersion::Eth68);
    let server = tokio::spawn(async move {
        for _ in 0..3 {
            match rx.recv().await {
                Some(PeerRequest::GetBlockHeaders { response, .. }) => {
                    let _ = response.send(Ok(BlockHeaders(vec![payload.header.clone()])));
                }
                Some(PeerRequest::GetBlockBodies { response, .. }) => {
                    let _ = response.send(Ok(BlockBodies(vec![payload.body.clone()])));
                }
                Some(PeerRequest::GetReceipts { response, .. }) => {
                    let receipts = payload
                        .receipts
                        .iter()
                        .cloned()
                        .map(|r| {
                            r.into_with_bloom_unchecked(alloy_primitives::Bloom::repeat_byte(0xff))
                        })
                        .collect();
                    let _ = response.send(Ok(Receipts(vec![receipts])));
                }
                _ => return Err(eyre!("unexpected request")),
            }
        }
        Ok(())
    });
    let fetched = fetch_payloads_for_peer(&peer, number..=number).await?;
    ensure!(
        fetched.payloads.len() == 1 && fetched.payloads[0].receipts == expected,
        "peer-provided bloom affected receipt validation"
    );
    server.await??;
    Ok(())
}

#[test]
fn serving_range_uses_history_and_verified_head() {
    let (mut peer, _) = mock_peer::<BaseChain>(EthVersion::Eth69);
    peer.earliest_block = Some(100);
    peer.latest_block = Some(200);
    assert_eq!(peer.serving_range(300), 100..=200);
    peer.head_number = 250;
    assert_eq!(peer.serving_range(300), 100..=250);
    assert_eq!(peer.serving_range(150), 100..=150);
    assert!(peer.serving_range(99).is_empty());
}
