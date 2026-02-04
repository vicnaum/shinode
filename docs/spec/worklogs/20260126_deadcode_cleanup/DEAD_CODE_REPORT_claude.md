# Dead Code & Legacy Report for `node/src`

Generated: 2026-01-26

## 1. Dead Code Marked with `#[allow(dead_code)]`

### `metrics.rs`
| Item | Type | Status |
|------|------|--------|
| `lag_to_head()` | function | Unused - has tests but not called anywhere |
| `rate_per_sec()` | function | Unused - has tests but not called anywhere |

### `p2p/mod.rs`
| Item | Type | Status |
|------|------|--------|
| `SelectorPeerId` | type alias | Legacy - not used |
| `PeerSelector` | trait | Legacy - not used |
| `RoundRobinPeerSelector` | struct | Legacy - not used (was likely for test scaffolding) |
| `request_headers_chunked()` | function | Test-only (`#[cfg(test)]`) |
| `request_bodies_chunked_partial()` | function | Test-only wrapper |
| `request_receipts_chunked_partial()` | function | Test-only wrapper |

### `storage/mod.rs`
All types marked `#[allow(dead_code)]` but **are actually used** - these annotations may be overly cautious:
- `StoredTxHashes`, `StoredTransaction`, `StoredTransactions`, `StoredWithdrawal`, `StoredWithdrawals`, `StoredBlockSize`, `StoredReceipts`, `StoredLog`, `StoredLogs`, `LogIndexEntry`, `BlockBundle`, `StoredPeer`

**Note:** `LogIndexEntry` appears to be truly unused (no references outside its definition).

### `storage/sharded/mod.rs`
| Item | Type | Status |
|------|------|--------|
| `SegmentWriter.compression` | field | Reserved but unused in current impl |
| `append_rows_no_commit()` | method | Reserved for future optimization |
| `commit()` | method | Reserved for future use with above |
| `DirtyShardInfo.sorted` | field | Unused |
| `DirtyShardInfo.complete` | field | Unused |
| `set_last_indexed_block()` | method | Only used in tests |

### `sync/historical/db_writer.rs`
| Item | Type | Status |
|------|------|--------|
| `DbWriterMessage::Flush` | enum variant | Never sent - could be removed |

### `sync/historical/stats.rs`
| Item | Type | Status |
|------|------|--------|
| `DbWriteByteTotals::add()` | method | Unused |
| `IngestBenchStats::logs_total()` | method | Unused |

---

## 2. Deleted Files Still Referenced

**`sink.rs`** was deleted (per git status) but is still referenced in:
- `sync/historical/AGENTS.md` - Lists it as a file with `run_probe_sink()`, `ProbeRecord`, etc.

---

## 3. AGENTS.md Files Out of Sync

### `cli/AGENTS.md`
**Non-existent items referenced:**
- `BenchmarkMode` - type doesn't exist
- `DEFAULT_BENCHMARK_TRACE_FILTER` - constant doesn't exist

### `p2p/AGENTS.md`
**Non-existent items referenced:**
- `MultiPeerBlockPayloadSource` - type doesn't exist
- `request_receipt_counts()` - function doesn't exist
- `BlockPayloadSource` - trait doesn't exist

### `sync/AGENTS.md`
**Non-existent items referenced:**
- `BlockPayloadSource` - trait doesn't exist (only `BlockPayload` struct exists)

### `sync/historical/AGENTS.md`
**Extensive outdated content:**
- References deleted `sink.rs` file
- `run_probe_sink()` - doesn't exist
- `fetch_probe_batch()` - doesn't exist
- `FetchProbeOutcome` - doesn't exist
- `ProbeRecord` - doesn't exist
- `ProbeStats` - doesn't exist
- `run_benchmark_probe()` - doesn't exist

**Verdict:** The probe/benchmark functionality appears to have been removed, but AGENTS.md still documents it extensively.

---

## 4. Duplicated Code Patterns

### **High-Impact Duplications:**

#### 1. `PeerHealthSummary` construction (main.rs:654-749)
The exact same struct construction from `dump` appears **3 times**:
- `peer_health_all` (lines ~654-681)
- `top` (lines ~684-713)
- `worst` (lines ~720-749)

**Recommendation:** Implement `From<&PeerHealthDump> for PeerHealthSummary` or a helper function.

#### 2. `bytes_total` calculation (db_writer.rs)
Same calculation of total bytes from `DbWriteByteTotals` repeated at:
- Lines 39-50
- Lines 68-80
- Lines 213-225
- Lines 241-251

**Recommendation:** Add a `DbWriteByteTotals::total()` method.

### **Medium-Impact Duplications:**

#### 3. Test utilities duplicated
- `temp_dir()` function duplicated in: `rpc/mod.rs:636`, `db_writer.rs:516`
- `base_config()` function nearly identical in both test modules

**Recommendation:** Create a shared `#[cfg(test)]` test utilities module.

#### 4. Transaction type matching (process.rs:62-84)
`tx_hash_fast()` and `signing_hash_fast()` have identical match arm structures:
```rust
match tx {
    TransactionSigned::Legacy(signed) => ...
    TransactionSigned::Eip2930(signed) => ...
    TransactionSigned::Eip1559(signed) => ...
    TransactionSigned::Eip4844(signed) => ...
    TransactionSigned::Eip7702(signed) => ...
}
```
**Recommendation:** Consider a macro or generic helper if more tx-type operations are added.

---

## 5. Summary of Recommended Actions

### Remove Dead Code:
1. **`metrics.rs`**: Remove `lag_to_head()` and `rate_per_sec()` (or use them)
2. **`p2p/mod.rs`**: Remove `SelectorPeerId`, `PeerSelector`, `RoundRobinPeerSelector`
3. **`storage/mod.rs`**: Remove `LogIndexEntry` if truly unused
4. **`db_writer.rs`**: Remove `DbWriterMessage::Flush` variant
5. **`stats.rs`**: Remove `DbWriteByteTotals::add()` and `IngestBenchStats::logs_total()`

### Extract Duplicated Code:
1. Create `impl From<&PeerHealthDump> for PeerHealthSummary`
2. Add `DbWriteByteTotals::total() -> u64` method
3. Create shared test utilities module for `temp_dir()` and `base_config()`
