use super::manifest::{chunk_path, Chunk, Plan};
use crate::{
    chain::{BaseChain, ChainKind},
    sync::{validation::validate_payload, BlockPayload},
};
use alloy_consensus::{proofs, BlockBody, Header, TxReceipt};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result, WrapErr};
use op_alloy_consensus::{OpReceipt, OpTxEnvelope};
use reth_codecs::Compact;
use reth_nippy_jar::{NippyJar, NippyJarCursor};
use reth_static_file_types::{SegmentHeader, StaticFileSegment};
use std::path::Path;

fn load(root: &Path, chunk: &Chunk, kind: StaticFileSegment) -> Result<NippyJar<SegmentHeader>> {
    let file = chunk
        .files
        .iter()
        .find(|file| Path::new(&file.path).extension().is_none())
        .ok_or_else(|| eyre!("missing static file"))?;
    let jar = NippyJar::<SegmentHeader>::load(&chunk_path(root, chunk).join(&file.path))?;
    ensure!(
        jar.user_header().segment() == kind && jar.columns() == kind.columns(),
        "static-file segment/column mismatch"
    );
    ensure!(
        jar.user_header().block_start() == Some(chunk.start),
        "static-file start mismatch"
    );
    ensure!(
        jar.user_header().expected_block_start() == chunk.start
            && jar.user_header().expected_block_end() == chunk.end,
        "static-file expected range differs from manifest"
    );
    ensure!(
        jar.user_header()
            .block_end()
            .is_some_and(|end| end >= chunk.start && end <= chunk.end),
        "static-file end mismatch"
    );
    Ok(jar)
}

fn compact<T: Compact>(bytes: &[u8]) -> Result<T> {
    // Upstream Compact exposes infallible decoders that can panic on an unknown
    // format. Convert that into an import error before any output is committed.
    // Compressed branches return the original input in their remainder slot.
    std::panic::catch_unwind(|| T::from_compact(bytes, bytes.len()).0)
        .map_err(|_| eyre!("unsupported or corrupt snapshot Compact encoding"))
}

pub(super) fn reconstruct(
    header: Header,
    max_transactions: u32,
    mut next: impl FnMut() -> Result<(OpTxEnvelope, OpReceipt)>,
) -> Result<BlockPayload<BaseChain>> {
    ensure!(
        header.withdrawals_root.is_none(),
        "snapshot import supports pre-Canyon blocks only; later-fork bodies are not verified"
    );
    let mut body = BlockBody::<OpTxEnvelope>::default();
    ensure!(
        proofs::calculate_ommers_root(&body.ommers) == header.ommers_hash,
        "missing ommers"
    );
    let mut receipts = Vec::new();
    let mut gas = 0;
    while gas != header.gas_used
        || proofs::calculate_transaction_root(&body.transactions) != header.transactions_root
    {
        ensure!(
            receipts.len() < max_transactions as usize,
            "per-block transaction limit reached"
        );
        let (tx, receipt) = next()?;
        ensure!(
            u8::from(tx.tx_type()) == u8::from(receipt.tx_type()),
            "transaction/receipt type mismatch"
        );
        let cumulative = receipt.cumulative_gas_used();
        ensure!(
            cumulative >= gas && cumulative <= header.gas_used,
            "receipt gas outside block"
        );
        gas = cumulative;
        body.transactions.push(tx);
        receipts.push(receipt);
    }
    validate_payload::<BaseChain>(&header, &body, &receipts).map_err(|reason| eyre!(reason))?;
    Ok(BlockPayload {
        header,
        body,
        receipts,
    })
}

#[derive(Default)]
struct Position {
    previous: Option<B256>,
    block: u64,
    transaction: u64,
}

pub fn scan(
    root: &Path,
    plan: &Plan,
    max_transactions: u32,
    mut consume: impl FnMut(BlockPayload<BaseChain>) -> Result<()>,
) -> Result<()> {
    let mut position = Position::default();
    for chunk in plan.chunks.iter().filter(|c| c.component == "headers") {
        scan_chunk(
            root,
            plan,
            chunk,
            max_transactions,
            &mut position,
            &mut consume,
        )?;
    }
    ensure!(
        position.block == plan.requested_range[1] + 1,
        "snapshot did not cover requested end"
    );
    Ok(())
}

fn scan_chunk(
    root: &Path,
    plan: &Plan,
    header_chunk: &Chunk,
    max_transactions: u32,
    position: &mut Position,
    consume: &mut impl FnMut(BlockPayload<BaseChain>) -> Result<()>,
) -> Result<()> {
    let find = |name: &str| {
        plan.chunks
            .iter()
            .find(|c| {
                c.component == name && c.start == header_chunk.start && c.end == header_chunk.end
            })
            .ok_or_else(|| eyre!("missing aligned {name} chunk"))
    };
    let headers = load(root, header_chunk, StaticFileSegment::Headers)?;
    let transactions = load(root, find("transactions")?, StaticFileSegment::Transactions)?;
    let receipts = load(root, find("receipts")?, StaticFileSegment::Receipts)?;
    let end = header_chunk.end.min(plan.snapshot_height);
    ensure!(
        headers.rows() as u64 == end - header_chunk.start + 1
            && headers.user_header().block_end() == Some(end),
        "header chunk is truncated"
    );
    ensure!(
        position.block == header_chunk.start,
        "gap between header chunks"
    );
    ensure!(
        transactions.rows() == receipts.rows()
            && transactions.user_header().tx_range() == receipts.user_header().tx_range(),
        "transaction/receipt rows differ"
    );
    ensure!(
        transactions.user_header().tx_len().unwrap_or(0) == transactions.rows() as u64,
        "transaction range does not match row count"
    );
    ensure!(
        transactions.user_header().block_end() == Some(end)
            && receipts.user_header().block_end() == Some(end),
        "incomplete transaction/receipt block range"
    );
    ensure!(
        transactions.rows() == 0
            || transactions.user_header().tx_start() == Some(position.transaction),
        "transaction-number discontinuity"
    );
    let first_transaction = position.transaction;
    let mut h = NippyJarCursor::new(&headers)?;
    let mut t = NippyJarCursor::new(&transactions)?;
    let mut r = NippyJarCursor::new(&receipts)?;
    while position.block <= end.min(plan.requested_range[1]) {
        let row = h.next_row()?.ok_or_else(|| eyre!("missing header row"))?;
        ensure!(row.len() == 3, "wrong header columns");
        let header: Header = compact(row[0])?;
        let hash = header.hash_slow();
        ensure!(
            header.number == position.block && row[2] == hash.as_slice(),
            "header number/hash mismatch"
        );
        if let Some(parent) = position.previous {
            ensure!(header.parent_hash == parent, "broken parent continuity");
        } else {
            ensure!(
                hash == ChainKind::Base.genesis_hash(),
                "snapshot genesis mismatch"
            );
        }
        let payload = reconstruct(header, max_transactions, || {
            let row = t
                .next_row()?
                .ok_or_else(|| eyre!("missing transaction row"))?;
            let tx = compact(
                row.first()
                    .ok_or_else(|| eyre!("missing transaction column"))?,
            )?;
            let row = r.next_row()?.ok_or_else(|| eyre!("missing receipt row"))?;
            let receipt = compact(row.first().ok_or_else(|| eyre!("missing receipt column"))?)?;
            position.transaction += 1;
            Ok((tx, receipt))
        })
        .wrap_err_with(|| format!("reconstructing block {}", position.block))?;
        consume(payload)?;
        position.previous = Some(hash);
        position.block += 1;
    }
    if position.block == end + 1 {
        ensure!(
            position.transaction - first_transaction == transactions.rows() as u64,
            "unassigned transactions at archive boundary"
        );
    }
    Ok(())
}
