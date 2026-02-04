# blobstore

## Purpose
Blob sidecar storage for EIP-4844 transactions (in-memory, disk, or noop), plus tracking helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Blob store trait, errors, size tracking, and implementation re-exports.
- **Key items**: `BlobStore`, `BlobStoreError`, `BlobStoreCleanupStat`, `BlobStoreCanonTracker`
- **Interactions**: Used by pool maintenance to store/cleanup blob sidecars.

### `converter.rs`
- **Role**: Converts blob sidecars to EIP-7594 format with bounded concurrency.
- **Key items**: `BlobSidecarConverter`
- **Knobs / invariants**: Global semaphore caps concurrent conversions.

### `disk.rs`
- **Role**: Disk-backed blob store with caching and deferred deletion.
- **Key items**: `DiskFileBlobStore`, `DiskFileBlobStoreConfig`, `OpenDiskFileBlobStore`
- **Knobs / invariants**: Cleanup required to remove deferred deletions; cache size limits.

### `mem.rs`
- **Role**: In-memory blob store with size tracking.
- **Key items**: `InMemoryBlobStore`

### `noop.rs`
- **Role**: No-op blob store implementation for wiring/testing.
- **Key items**: `NoopBlobStore`

### `tracker.rs`
- **Role**: Tracks blob transactions included in canonical blocks.
- **Key items**: `BlobStoreCanonTracker`, `BlobStoreUpdates`
- **Interactions**: Produces deletions when blocks finalize.

## End-to-end flow (high level)
- Validators extract blob sidecars and insert into a `BlobStore`.
- Pool maintenance uses `BlobStoreCanonTracker` to mark finalized blobs.
- The blob store deletes finalized blobs and optionally cleans up deferred deletions.

## Key APIs (no snippets)
- `BlobStore`, `DiskFileBlobStore`, `InMemoryBlobStore`
- `BlobStoreCanonTracker`, `BlobSidecarConverter`
