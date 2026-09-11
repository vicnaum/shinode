# Base protocol compatibility — phase 1

Verified on 2026-09-11. The initial port targets **Base mainnet through Beryl**.
Shinode's existing Reth revision and OP Alloy 0.23.1 successfully decoded and
verified a recent Base block over P2P. No Reth upgrade is needed for this target.
Base selection and ingestion are not enabled in the production binary yet.

## Pinned sources and protocol

Official Base source revision:
[`d5eed12dea87bff8688fbf0e1b789aa3f93f78a2`](https://github.com/base/base/tree/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2).
Sieve adaptation reference: `914d29fccf55c40a7bbc03ba2db122f7a1ae24c7`.

| Property | Verified value |
| --- | --- |
| Chain ID | 8453 (`0x2105`) |
| Genesis | `0xf712aa9241cc24369b143cf6dce85f0902a9731e70d66818a3a5845b296c73dd` |
| Azul / Ethereum Osaka activation | `1779991200` — 2026-05-28 18:00 UTC |
| Beryl activation | `1782410400` — 2026-06-25 18:00 UTC |
| Observed peer fork ID | `0xea0159eb`, next `0` |
| Discovery | discv5 with packet protocol ID `basev0`; discv4 disabled in the probe |
| Execution bootstrap records | 10 records, five node identities on ports 30301 and 9200 |
| Decoded transaction types | `0x00`, `0x01`, `0x02`, `0x04`, `0x7e` |

The genesis, timestamps, and execution bootnodes come from the pinned
[chain configuration](https://github.com/base/base/blob/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2/crates/common/chains/src/config.rs).
The `basev0` setting is defined in the pinned
[execution network configuration](https://github.com/base/base/blob/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2/crates/execution/node/src/node.rs).
The extracted bootstrap records are in
[`base-mainnet-protocol.json`](../node/tests/fixtures/base-mainnet-protocol.json).
Osaka and Azul share an activation timestamp; the fork-ID regression test ensures
the composed Reth spec produces the observed Beryl fork ID.

## Cobalt, Denim, and Zenith boundary

Sieve's comment associating the new transaction type `0x79` with Cobalt is stale.
The pinned Base source identifies `0x79` as EIP-8130 and gates RPC submission
behind **Zenith**, as shown in its
[transaction type definitions](https://github.com/base/base/blob/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2/crates/common/consensus/src/transaction/tx_type.rs).
The pinned mainnet configuration has no static Cobalt or Denim timestamp.

An absent timestamp is not proof that an upgrade cannot activate: Base supports
runtime schedule overrides from an L1 contract through its
[upgrade signal reader](https://github.com/base/base/blob/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2/crates/utilities/upgrade-signal/src/contract.rs).
The live probe confirmed the Beryl fork ID and successfully verified a recent
block, independently matching its hash against the public Base RPC. This is
evidence for the observed Beryl protocol, not support for future runtime upgrades
or proof that every peer has the same configuration.

The port must explicitly reject unsupported transaction types and recheck the
active fork before release. Cobalt/Denim/Zenith support is outside the verified
target; if the active protocol changes, revisit dependencies and fork handling
before advertising current-head compatibility. The offline test explicitly
confirms that this OP Alloy version rejects `0x79`.

## Dependency and storage result

All Reth dependencies remain pinned to
`a74cb9cbc38a5590b2596c67e64a3c7642024aca`. The additions provide OP chainspec,
primitives, fork/receipt validation, typed noop storage, and Base discovery.
`Cargo.lock` resolves `op-alloy-consensus` to 0.23.1.

`reth-optimism-primitives` enables the OP implementation of
`SerdeBincodeCompat`; its `serde-bincode-compat` feature and OP Alloy's matching
feature expose a representation that works with Shinode's existing bincode 1
serializer. Tests round-trip a mixed ordinary/deposit receipt vector with logs
and all three deposit field combinations: no fields, nonce only, nonce plus
receipt version. No production storage format is changed in phase 1.

## Live evidence

At **2026-09-11 14:55:00 UTC**, the probe completed an **eth/69** session and
fetched block **51,174,575**:

`0xb012f7eec1b586865500b42d70c4922d6e3468092e3ca0965758f0b4a8be51a1`

It decoded **395 transactions, 395 receipts, and 773 logs**, checked the
requested header hash, recomputed the transaction root and OP receipt root, and
required the block to be within five minutes of local time. The observed block
was three seconds old. Public RPC returned the same block hash, number,
timestamp, and transaction count. See the retained
[evidence JSON](base-protocol-evidence.json), including peer ID and reference roots.

This probe does not validate L1 finality, all historical forks, or every receipt
protocol variant against live peers; those checks remain in later phases.

## Reproduce and validation results

```sh
cargo test --test base_protocol --locked
SHINODE_BASE_PROBE_OUTPUT=/tmp/shinode-base-p2p-evidence.json \
  cargo test --test base_protocol --locked live_base_protocol_probe -- --ignored --nocapture
```

The network test uses ephemeral ports, writes no node database, stops after at
most 90 seconds of peer probing, and is ignored in normal test runs. It writes
the report only when `SHINODE_BASE_PROBE_OUTPUT` is set; the report itself is not
a continuously updated compatibility guarantee.

Validation on this branch:

- `cargo check --workspace`: passed.
- `cargo check --workspace --no-default-features --locked --offline`: passed.
- `cargo test --workspace --locked --offline`: 47 passed, 3 ignored (the new
  live probe plus two pre-existing integration placeholders).
- Explicit live probe: 1 passed; evidence above.
- `cargo clippy --test base_protocol --locked --offline -- -D warnings`: passed.

Chain selection and production network/storage integration remain follow-up work.
