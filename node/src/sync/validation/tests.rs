use super::*;
use crate::chain::{BaseChain, EthereumChain};
use crate::sync::BlockPayload;
use alloy_consensus::SignableTransaction;
use alloy_primitives::{Address, Bytes, Log, Sealable, B256};
use eyre::{ensure, Result};
use op_alloy_consensus::{OpDepositReceipt, OpTxEnvelope, TxDeposit};
use reth_optimism_primitives::OpReceipt;

pub const CANYON: u64 = 1_704_992_401;
pub const ISTHMUS: u64 = 1_746_806_401;

pub fn commit_payload<C: ChainTypes>(payload: &mut BlockPayload<C>) {
    payload.header.transactions_root =
        proofs::calculate_transaction_root(&payload.body.transactions);
    payload.header.ommers_hash = proofs::calculate_ommers_root(&payload.body.ommers);
    payload.header.receipts_root = C::receipts_root(&payload.receipts, &payload.header);
    payload.header.logs_bloom = Bloom::ZERO;
    for receipt in &payload.receipts {
        payload.header.logs_bloom.accrue_logs(receipt.logs());
    }
}

pub fn base_payload(timestamp: u64) -> BlockPayload<BaseChain> {
    let mut payload = BlockPayload {
        header: Header {
            number: 42,
            timestamp,
            ..Default::default()
        },
        body: BlockBody {
            transactions: vec![OpTxEnvelope::Deposit(TxDeposit::default().seal_slow())],
            ..Default::default()
        },
        receipts: vec![OpReceipt::Deposit(OpDepositReceipt {
            inner: alloy_consensus::Receipt {
                status: true.into(),
                cumulative_gas_used: 21_000,
                logs: vec![Log::new_unchecked(
                    Address::repeat_byte(1),
                    vec![B256::repeat_byte(2)],
                    Bytes::from_static(&[3]),
                )],
            },
            deposit_nonce: Some(7),
            deposit_receipt_version: (timestamp >= CANYON).then_some(1),
        })],
    };
    if timestamp >= CANYON {
        payload.body.withdrawals = Some(alloy_eips::eip4895::Withdrawals::default());
        payload.header.withdrawals_root = Some(if timestamp >= ISTHMUS {
            B256::repeat_byte(0x77)
        } else {
            alloy_consensus::constants::EMPTY_ROOT_HASH
        });
    }
    commit_payload(&mut payload);
    payload
}

pub fn ethereum_payload() -> BlockPayload<EthereumChain> {
    let mut payload = BlockPayload {
        header: Header {
            number: 42,
            ..Default::default()
        },
        body: BlockBody {
            transactions: vec![reth_ethereum_primitives::TransactionSigned::Legacy(
                alloy_consensus::TxLegacy::default().into_signed(alloy_primitives::Signature::new(
                    alloy_primitives::U256::from(1),
                    alloy_primitives::U256::from(2),
                    false,
                )),
            )],
            ..Default::default()
        },
        receipts: vec![reth_ethereum_primitives::Receipt::default()],
    };
    commit_payload(&mut payload);
    payload
}

fn reject_corruption<C: ChainTypes + Clone>(valid: BlockPayload<C>) -> Result<()> {
    ensure!(
        validate_payload::<C>(&valid.header, &valid.body, &valid.receipts).is_ok(),
        "valid payload rejected"
    );
    let mut wrong = valid.clone();
    wrong.receipts.clear();
    ensure!(
        validate_payload::<C>(&wrong.header, &wrong.body, &wrong.receipts)
            == Err("transaction/receipt count mismatch"),
        "receipt count not checked"
    );
    for (field, reason) in [
        (0, "transactions root mismatch"),
        (1, "ommers root mismatch"),
        (2, "receipts root mismatch"),
    ] {
        let mut wrong = valid.clone();
        match field {
            0 => wrong.header.transactions_root = B256::repeat_byte(4),
            1 => wrong.body.ommers.push(Header::default()),
            _ => wrong.header.receipts_root = B256::repeat_byte(4),
        }
        ensure!(
            validate_payload::<C>(&wrong.header, &wrong.body, &wrong.receipts) == Err(reason),
            "commitment not checked: {reason}"
        );
    }
    let mut wrong = valid.clone();
    wrong.header.logs_bloom = Bloom::repeat_byte(0xff);
    ensure!(
        validate_payload::<C>(&wrong.header, &wrong.body, &wrong.receipts)
            == Err("logs bloom mismatch"),
        "header bloom not checked"
    );
    let mut wrong = valid;
    wrong.header.withdrawals_root = Some(B256::repeat_byte(4));
    ensure!(
        validate_payload::<C>(&wrong.header, &wrong.body, &wrong.receipts)
            == Err("withdrawals mismatch"),
        "withdrawals not checked"
    );
    Ok(())
}

#[test]
fn ethereum_rejects_malformed_payloads() -> Result<()> {
    reject_corruption(ethereum_payload())
}

#[test]
fn base_rejects_malformed_payloads() -> Result<()> {
    reject_corruption(base_payload(CANYON - 1))
}

#[test]
fn base_fork_boundaries_validate_complete_payloads() -> Result<()> {
    for timestamp in [
        CANYON - 1,
        CANYON,
        CANYON + 1,
        ISTHMUS - 1,
        ISTHMUS,
        ISTHMUS + 1,
    ] {
        let valid = base_payload(timestamp);
        ensure!(
            validate_payload::<BaseChain>(&valid.header, &valid.body, &valid.receipts).is_ok(),
            "valid fork fixture rejected at {timestamp}"
        );
        let mut wrong = valid.clone();
        wrong.body.withdrawals = if timestamp < CANYON {
            Some(alloy_eips::eip4895::Withdrawals::default())
        } else {
            None
        };
        ensure!(
            validate_payload::<BaseChain>(&wrong.header, &wrong.body, &wrong.receipts)
                == Err("withdrawals mismatch"),
            "withdrawals presence not enforced at {timestamp}"
        );
        let mut wrong = valid;
        if timestamp >= ISTHMUS {
            wrong
                .body
                .withdrawals
                .as_mut()
                .ok_or_else(|| eyre::eyre!("missing withdrawals"))?
                .push(alloy_eips::eip4895::Withdrawal::default());
            ensure!(
                validate_payload::<BaseChain>(&wrong.header, &wrong.body, &wrong.receipts)
                    == Err("withdrawals mismatch"),
                "post-Isthmus body must have empty withdrawals"
            );
        }
    }
    // A Canyon deposit receipt version changes the receipt commitment.
    let mut wrong = base_payload(CANYON);
    if let OpReceipt::Deposit(receipt) = &mut wrong.receipts[0] {
        receipt.deposit_receipt_version = None;
    }
    ensure!(
        validate_payload::<BaseChain>(&wrong.header, &wrong.body, &wrong.receipts)
            == Err("receipts root mismatch"),
        "deposit receipt version was ignored"
    );
    Ok(())
}
