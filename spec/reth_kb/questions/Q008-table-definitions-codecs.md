# Q008 - Table Definitions + Codecs Mapping

Status: verified

## Question
How are MDBX tables defined and how do key/value codecs map to stored data?

## Context Packs
- `spec/reth_repo_context/context-storage.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-storage.xml

Question:
How are MDBX tables defined and how do key/value codecs map to stored data?
I want a map of the `tables!` macro definitions, the `Table`/`Encode`/`Decode`/`Compress`/`Decompress` traits, and a few concrete examples of tables (Headers, BlockBodyIndices, Transactions, Accounts/Storage).

Requirements:
1. Cite file paths and type names for every major claim.
2. For each example table, specify Key/Value types and where those types live.
3. Explain how encoding/compression is applied when reading/writing (`DbTx::get`, `DbTxMut::put`).
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Table definition mechanism (files + macro + traits)
3. Example tables (key/value + codec types)
4. Read/write path (encode/compress vs decode/decompress)
5. “Where to look next”
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-storage.xml` pack, here is an analysis of how MDBX tables, keys, and values are defined and processed in Reth.

### 1. Summary
*   **Macro-Driven Definitions**: All database tables are defined centrally using the `tables!` macro, which generates zero-sized marker structs implementing the `Table` trait.
*   **Key vs. Value Codecs**: Keys use the `Encode`/`Decode` traits (preserving order for B-Tree indexing), while Values use `Compress`/`Decompress` (optimizing for storage size).
*   **The `Compact` Trait**: Most table values (e.g., Headers, Transactions) use Reth's custom `Compact` codec, which uses bitfields and variable-length encoding to pack structs tighter than RLP.
*   **DUPSORT Tables**: Specific tables (like `PlainStorageState`) are defined as `DUPSORT`, allowing multiple values per key, sorted by a specific `SubKey`.
*   **Strict Type Safety**: The `Table` trait enforces specific Rust types for `Key`, `Value`, and optional `SubKey` at compile time, preventing type mismatches during database transactions.

---

### 2. Table Definition Mechanism

The database schema is defined in a single file using a declarative macro.

#### A. The `tables!` Macro
Located in **`crates/storage/db-api/src/tables/mod.rs`**, this macro generates the structs and trait implementations.

*   **Syntax**:
    ```rust
    tables! {
        table Name {
            type Key = K;
            type Value = V;
            // Optional:
            type SubKey = S;
        }
    }
    ```
*   **Output**: For each entry, it creates a struct (e.g., `pub struct Headers;`) and implements the `Table` trait for it. It also generates an enum `Tables` containing all table variants for runtime iteration.

#### B. The Core Traits
Located in **`crates/storage/db-api/src/table.rs`**:

1.  **`trait Table`**: Defines the metadata.
    *   `const NAME`: The string name used in MDBX (e.g., "Headers").
    *   `type Key`: Must implement `Encode` + `Decode` + `Ord`.
    *   `type Value`: Must implement `Compress` + `Decompress`.
2.  **`trait DupSort`**: Extends `Table` for tables where a key maps to multiple values.
    *   `type SubKey`: Must implement `Encode` + `Decode`.
3.  **`trait Encode` / `trait Decode`**:
    *   Used for **Keys** and **SubKeys**.
    *   Crucial for preservation of ordering (e.g., BigEndian for integers).
4.  **`trait Compress` / `trait Decompress`**:
    *   Used for **Values**.
    *   Usually delegates to the `Compact` trait (bit packing) or simple serialization.

---

### 3. Example Tables

Here are specific examples found in **`crates/storage/db-api/src/tables/mod.rs`**:

#### A. `Headers` (Standard KV Table)
Stores block headers.
*   **Key**: `BlockNumber` (alias for `u64`).
    *   *Codec*: BigEndian bytes. Implemented via `impl_uints!` macro in `crates/storage/db-api/src/models/mod.rs`.
*   **Value**: `Header` (from `alloy_consensus`).
    *   *Codec*: `Compact` trait. Implemented in `crates/storage/codecs/src/alloy/header.rs`. It uses bitflags to skip empty fields (like `base_fee` in pre-London blocks).

#### B. `BlockBodyIndices` (Indices)
Maps block numbers to transaction ranges.
*   **Key**: `BlockNumber` (`u64`).
*   **Value**: `StoredBlockBodyIndices` (`struct { first_tx_num: u64, tx_count: u64 }`).
    *   *Codec*: `Compact`. Defined in `crates/storage/db-api/src/models/blocks.rs`.

#### C. `Transactions` (Canonical Data)
Stores transaction bodies.
*   **Key**: `TxNumber` (alias for `u64`, an incremental global ID).
*   **Value**: `TransactionSigned` (from `reth_ethereum_primitives`).
    *   *Codec*: `Compact`. Implemented in `crates/storage/codecs/src/alloy/transaction/mod.rs`. This handles EIP-1559 vs Legacy vs Blob formats efficiently.

#### D. `PlainStorageState` (DUPSORT Table)
Stores raw storage slots for accounts.
*   **Key**: `Address` (20 bytes).
*   **Value**: `StorageEntry` (`struct { key: B256, value: U256 }`).
*   **SubKey**: `B256` (The storage slot hash).
    *   *Mechanism*: In MDBX, the key is `Address`. The value stored is `StorageEntry`. MDBX sorts multiple entries for the same `Address` by extracting the `SubKey` (the `key` field of `StorageEntry`).

---

### 4. Read/Write Path

The mapping from Rust types to raw bytes occurs in the Transaction and Cursor implementations.

#### A. Writing (`put` / `upsert`)
Flow: `DbTxMut::put` -> `Encode`/`Compress` -> MDBX.

1.  **Interface**: `tx.put::<tables::Headers>(number, header)` is called.
2.  **Implementation**: In **`crates/storage/db/src/implementation/mdbx/tx.rs`**:
    ```rust
    fn put<T: Table>(...) {
        let key = key.encode();         // Trait Encode (Key)
        let value = value.compress();   // Trait Compress (Value)
        // ... pass raw bytes to MDBX ...
    }
    ```
3.  **Optimization**: `Compress::compress_to_buf` is often used to write directly into a reusable buffer to avoid allocation.

#### B. Reading (`get` / `cursor.next`)
Flow: MDBX -> Raw Bytes -> `Decode`/`Decompress`.

1.  **Interface**: `tx.get::<tables::Headers>(number)` or `cursor.next()`.
2.  **Implementation**: In **`crates/storage/db/src/implementation/mdbx/cursor.rs`**:
    ```rust
    pub fn decode<T>(...) -> PairResult<T> {
        // ...
        let key = T::Key::decode(k)?;       // Trait Decode
        let value = T::Value::decompress(v)?; // Trait Decompress
        Ok((key, value))
    }
    ```
3.  **Decompression**:
    *   If the type implements `Compact`, `decompress` calls `Compact::from_compact`.
    *   Ideally, `from_compact` reads directly from the MDBX pointer (zero-copy) for simple types, but complex types (like Headers) usually require deserialization/copying.

---

### 5. "Where to Look Next"

1.  **`crates/storage/db-api/src/tables/mod.rs`**: The authoritative source of truth for the database schema.
2.  **`crates/storage/codecs/src/lib.rs`**: The definition of the `Compact` trait, which explains the bitfield packing strategy.
3.  **`crates/storage/db/src/implementation/mdbx/cursor.rs`**: To see exactly how `DUPSORT` tables (`DbDupCursorRO`) handle subkeys compared to standard tables.
4.  **`crates/storage/db-api/src/models/accounts.rs`**: To see how composite keys (like `BlockNumberAddress` used in `StorageChangeSets`) are manually encoded to preserve sort order.

### 6. Suggested Follow-up Questions
*   "How does the `Compact` derive macro generate the bitflags for a struct like `Header`?"
*   "How are `IntegerList`s encoded in `AccountsHistory` to store lists of block numbers efficiently?"
*   "What is the difference between `PlainAccountState` and `HashedAccounts`, and why do we need both?"

## Verification
- `tables!` generates zero‑sized marker structs (with `PhantomData`) and implements `Table`.```129:156:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/tables/mod.rs
pub struct $name$(<$($generic $( = $default)?),*>)? {
    _private: std::marker::PhantomData<($($($generic,)*)?)>,
}
impl$(<$($generic),*>)? $crate::table::Table for $name$(<$($generic),*>)? { ... }
```
- The `Table` trait enforces `Key` and `Value` bounds; `Key` requires `Encode`/`Decode`, and `Value` requires `Compress`/`Decompress`.```69:101:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/table.rs
pub trait Key: Encode + Decode + Ord + Clone + Serialize + for<'a> Deserialize<'a> {}
pub trait Value: Compress + Decompress + Serialize {}
...
pub trait Table: Send + Sync + Debug + 'static {
    const NAME: &'static str;
    const DUPSORT: bool;
    type Key: Key;
    type Value: Value;
}
```
- Integer keys use big‑endian encoding via `impl_uints!` (e.g., `u64`).```37:58:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/models/mod.rs
impl Encode for u64 {
    type Encoded = [u8; std::mem::size_of::<u64>()];
    fn encode(self) -> Self::Encoded { self.to_be_bytes() }
}
impl Decode for u64 {
    fn decode(value: &[u8]) -> Result<Self, DatabaseError> {
        Ok(u64::from_be_bytes(value.try_into().map_err(|_| DatabaseError::Decode)?))
    }
}
```
- `Headers`, `BlockBodyIndices`, `Transactions`, and `PlainStorageState` tables define their Key/Value (and SubKey for dupsort).```327:399:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/tables/mod.rs
table Headers<H = Header> {
    type Key = BlockNumber;
    type Value = H;
}
table BlockBodyIndices {
    type Key = BlockNumber;
    type Value = StoredBlockBodyIndices;
}
table Transactions<T = TransactionSigned> {
    type Key = TxNumber;
    type Value = T;
}
table PlainStorageState {
    type Key = Address;
    type Value = StorageEntry;
    type SubKey = B256;
}
```
- `StoredBlockBodyIndices` stores `first_tx_num` and `tx_count` and derives `Compact` (via feature).```17:27:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-models/src/blocks.rs
#[cfg_attr(any(test, feature = "reth-codec"), derive(reth_codecs::Compact))]
pub struct StoredBlockBodyIndices {
    pub first_tx_num: TxNumber,
    pub tx_count: NumTransactions,
}
```
- Many value types use `Compact` for `Compress`/`Decompress` via `impl_compression_for_compact!`, including `Header`, `StorageEntry`, `StoredBlockBodyIndices`, and `TransactionSigned`.```195:237:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/models/mod.rs
impl_compression_for_compact!(
    Header,
    StorageEntry,
    StoredBlockBodyIndices,
    TransactionSigned,
    ...
);
```
- `StorageEntry` carries `(key, value)` and exposes `get_subkey()` returning the storage key (B256).```15:41:/Users/vicnaum/github/stateless-history-node/reth/crates/primitives-traits/src/storage.rs
pub struct StorageEntry {
    pub key: B256,
    pub value: U256,
}
impl ValueWithSubKey for StorageEntry {
    type SubKey = B256;
    fn get_subkey(&self) -> Self::SubKey { self.key }
}
```
- `DbTxMut::put` encodes the key and compresses the value before MDBX write.```372:385:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/implementation/mdbx/tx.rs
let key = key.encode();
let value = value.compress();
...
tx.put(self.get_dbi::<T>()?, key.as_ref(), value, flags)
```
- Cursor decode path uses `Decode` + `Decompress` for `(key, value)` pairs.```64:75:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/implementation/mdbx/cursor.rs
pub fn decode<T>(...) -> PairResult<T>
where T: Table, T::Key: Decode, T::Value: Decompress {
    res.map_err(|e| DatabaseError::Read(e.into()))?.map(decoder::<T>).transpose()
}
```
- `Compact` uses bitflag structs and variable‑length integer encoding to minimize bytes (no compression).```28:34:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/codecs/README.md
The general idea behind [`Compact`] is to minimize the number of bytes ...
... uses a companion bitflag struct ...
```
```149:159:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/codecs/src/lib.rs
let leading = self.leading_zeros() as usize / 8;
buf.put_slice(&self.to_be_bytes()[leading..]);
core::mem::size_of::<$name>() - leading
```

## Corrections / Caveats
- The comparison to “tighter than RLP” is not stated in code; `Compact` is described as minimizing bytes, but RLP is not referenced.
- Not all values necessarily use `Compact`; many do via `impl_compression_for_compact!`, but it’s not universal.
- DUPSORT ordering is governed by MDBX’s duplicate value ordering; Reth exposes `SubKey`/`ValueWithSubKey`, but the precise sorting mechanics aren’t explicitly documented here.

## Actionable Pointers
- Table schema: `reth/crates/storage/db-api/src/tables/mod.rs`.
- Key/value traits and codecs: `reth/crates/storage/db-api/src/table.rs` and `reth/crates/storage/db-api/src/models/mod.rs`.
- Compact codec details: `reth/crates/storage/codecs/README.md` and `reth/crates/storage/codecs/src/lib.rs`.
- Read/write encoding flow: `reth/crates/storage/db/src/implementation/mdbx/tx.rs` and `reth/crates/storage/db/src/implementation/mdbx/cursor.rs`.
