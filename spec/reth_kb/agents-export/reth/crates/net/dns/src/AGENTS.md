# src

## Purpose
Implements EIP-1459 "Node Discovery via DNS" for reth: fetches and verifies DNS ENR trees (`enrtree-root`, branches, links, and ENR leaves), rate-limits TXT lookups, incrementally syncs trees, and emits discovered peers as `NodeRecord` + optional `ForkId`.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `config.rs`
- **Role**: Configuration for `DnsDiscoveryService` runtime behavior (timeouts, rate limits, cache sizing, periodic recheck interval, and optional bootstrap networks).
- **Key items**: `DnsDiscoveryConfig`, `lookup_timeout`, `max_requests_per_sec`, `recheck_interval`, `dns_record_cache_limit`, `bootstrap_dns_networks`
- **Knobs / invariants**: DNS TXT lookups are rate-limited; cache limit is enforced by an LRU keyed by record hash/domain.

### `error.rs`
- **Role**: Error types for parsing DNS tree entries and for lookup failures (timeouts, missing entries, root verification).
- **Key items**: `ParseDnsEntryError`, `LookupError`, `ParseEntryResult`, `LookupResult`

### `lib.rs`
- **Role**: Main service/handle: drives tree syncing, maintains per-tree state, executes lookups through `QueryPool`, caches resolved records, and notifies subscribers with `DnsNodeRecordUpdate`.
- **Key items**: `DnsDiscoveryService`, `DnsDiscoveryHandle`, `DnsNodeRecordUpdate`, `DnsDiscoveryEvent`, `sync_tree()`/`sync_tree_with_link()`, `node_record_stream()`, `convert_enr_node_record()`
- **Interactions**: Uses `QueryPool` (`query.rs`) for rate-limited DNS queries; uses `SyncTree` (`sync.rs`) to produce sync actions; parses DNS TXT records into `DnsEntry` (`tree.rs`).
- **Knobs / invariants**: Root entries are verified against the link's public key before syncing children; node records are only emitted when ENR has required socket fields.

### `query.rs`
- **Role**: Aggregates and drives DNS queries to completion with rate limiting and lookup timeouts (root lookups vs entry lookups).
- **Key items**: `QueryPool`, `QueryOutcome`, `ResolveRootResult`, `ResolveEntryResult`, `resolve_root()`, `resolve_entry()`
- **Interactions**: Called by `DnsDiscoveryService` to schedule and poll DNS lookups; delegates I/O to `Resolver`.

### `resolver.rs`
- **Role**: DNS lookup abstraction: `Resolver` trait for TXT queries, plus concrete implementations for real DNS (`DnsResolver`) and in-memory testing (`MapResolver`).
- **Key items**: `Resolver`, `DnsResolver`, `TokioResolver`, `MapResolver`

### `sync.rs`
- **Role**: Tree-sync state machine: maintains which roots/branches/links/ENRs remain to be fetched and emits `SyncAction`s that drive the next DNS lookups.
- **Key items**: `SyncTree`, `SyncAction`, `ResolveKind`, `SyncTree::poll()`, `SyncTree::update_root()`
- **Knobs / invariants**: Periodically triggers root refresh after `recheck_interval`; resync strategy depends on which root hashes changed (ENR root vs link root).

### `tree.rs`
- **Role**: EIP-1459 DNS record structure parsing and verification: represents and parses root/link/branch/node entries and verifies signed roots.
- **Key items**: `DnsEntry`, `TreeRootEntry`, `BranchEntry`, `LinkEntry`, `NodeEntry`, `TreeRootEntry::verify()`
- **Knobs / invariants**: Root signatures are verified over the unsigned root content; branch child hashes must meet size/format constraints.

## End-to-end flow (high level)
- Create a `DnsResolver` (system DNS config) or another `Resolver` implementation.
- Construct `DnsDiscoveryService::new_pair(resolver, DnsDiscoveryConfig)` or `DnsDiscoveryService::new(...)`.
- Start syncing one or more `enrtree://...@domain` links via `DnsDiscoveryHandle::sync_tree()` / `sync_tree_with_link()`.
- Service schedules a root TXT lookup and verifies the root's signature before accepting it.
- `SyncTree` emits `SyncAction`s to fetch link roots and ENR roots, expanding branches into child hashes.
- `QueryPool` executes TXT queries with request rate limiting and per-lookup timeouts.
- Resolved ENR leaf records are converted into `DnsNodeRecordUpdate` (node `NodeRecord` + optional `ForkId`) and broadcast to subscribers.
- Periodically rechecks roots and re-syncs subtrees if the sequence number/root hashes changed.

## Key APIs (no snippets)
- **Service**: `DnsDiscoveryService`, `DnsDiscoveryHandle`, `DnsNodeRecordUpdate`
- **Tree model**: `DnsEntry`, `TreeRootEntry`, `LinkEntry`, `BranchEntry`
- **I/O**: `Resolver`, `DnsResolver`
