# transactions

## Purpose
Transaction gossip and fetch logic for the network: manages announcements and full broadcasts, fetches missing pooled transactions, applies policies and limits, and coordinates with the transaction pool.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core transactions manager task and handle API: routes network events, tracks peers/seen hashes, coordinates pool imports, fetches missing txs, and drives propagation.
- **Key items**: `TransactionsManager`, `TransactionsHandle`, `PeerMetadata`, `NetworkTransactionEvent`, `PendingPoolImportsInfo`, `TransactionsCommand`, `PoolImportFuture`
- **Interactions**: Uses `TransactionFetcher` (`fetcher.rs`), `NetworkPolicies` (`policy.rs`), and config/constants; integrates with `NetworkHandle` and `TransactionPool`.
- **Knobs / invariants**: Limits pending pool imports, tracks bad imports, and respects propagation/ingress policies; disables propagation during sync.

### `config.rs`
- **Role**: Configuration surface and policy traits for transaction propagation and announcement filtering.
- **Key items**: `TransactionsManagerConfig`, `TransactionFetcherConfig`, `TransactionPropagationMode`, `TransactionPropagationKind`, `TransactionIngressPolicy`, `AnnouncementFilteringPolicy`, `AnnouncementAcceptance`
- **Interactions**: `TransactionsManager` consumes these configs; policies control which peers are allowed to ingest or receive txs.
- **Knobs / invariants**: Propagation modes (`Sqrt`, `All`, `Max`) and ingress policies (`All`, `Trusted`, `None`).

### `constants.rs`
- **Role**: Spec-derived and default bounds for transaction broadcasts, requests, retries, and fetcher concurrency/budgets.
- **Key items**: `SOFT_LIMIT_COUNT_HASHES_IN_NEW_POOLED_TRANSACTIONS_BROADCAST_MESSAGE`, `SOFT_LIMIT_COUNT_HASHES_IN_GET_POOLED_TRANSACTIONS_REQUEST`, `SOFT_LIMIT_BYTE_SIZE_POOLED_TRANSACTIONS_RESPONSE`, `tx_manager::*`, `tx_fetcher::*`
- **Knobs / invariants**: Default limits for peers, inflight requests, pending fetch capacity, and request packing sizes.

### `fetcher.rs`
- **Role**: Transaction fetcher: deduplicates announced hashes, packs `GetPooledTransactions` requests, retries missing hashes, and tracks inflight/pending fetch state per peer.
- **Key items**: `TransactionFetcher`, `TxFetchMetadata`, `FetchEvent`, `GetPooledTxRequest`, `GetPooledTxResponse`, `GetPooledTxRequestFut`, `UnverifiedPooledTransactions`, `VerifiedPooledTransactions`, `VerificationOutcome`, `TransactionFetcherInfo`
- **Interactions**: Used by `TransactionsManager` to request missing transactions and update metrics; uses cache structures for inflight/pending hashes and fallback peers.
- **Knobs / invariants**: Enforces per-peer inflight limits and request size/byte constraints based on `EthVersion`; retries bounded by `DEFAULT_MAX_RETRIES`.

### `policy.rs`
- **Role**: Bundles transaction propagation and announcement filtering policies into a single container.
- **Key items**: `NetworkPolicies`, `with_propagation()`, `with_announcement()`, `propagation_policy()`, `announcement_filter()`
- **Interactions**: Passed into `TransactionsManager::with_policy()` to control gossip behavior.

## End-to-end flow (high level)
- `TransactionsManager` subscribes to network events and pool pending transactions.
- Incoming announcements are filtered and deduplicated; unknown hashes are handed to `TransactionFetcher`.
- `TransactionFetcher` builds `GetPooledTransactions` requests within size limits and assigns them to idle peers, retrying via fallback peers on partial/failed responses.
- Received transactions are validated/imported into the pool with bounded concurrency; bad imports are cached and peers penalized.
- Propagation policy determines which peers receive full transactions vs hashes, and ingress policy controls which peers are accepted.

## Key APIs (no snippets)
- `TransactionsManager`, `TransactionsHandle`
- `TransactionFetcher`, `TransactionFetcherConfig`
- `TransactionPropagationMode`, `TransactionIngressPolicy`, `NetworkPolicies`
