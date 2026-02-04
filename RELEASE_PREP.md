# Release Preparation Notes

## Questions (Feb 4, 2025)

How do we prepare a repo for an official release? Currently we have all kinds of stuff there, which is helpful during development, but maybe is already no longer relevant for the release. For example:
- `analysis` dir is not really needed
- RPC-benchmark - I don't even know what this is anymore
- scripts has a script that I don't think we need
- `spec` has a lot of stuff for development but I'm not sure we need this in the official repo - maybe for now we do actually, I'm not sure
- the root folder is a mess - it has a lot of MD files, most of them are different versions of the same thing. this needs to be restructured somehow, and I guess we also need a changelog or something
- readme should be reiterated - it has a lot of old stuff I believe (or maybe just tidy up a bit)
- I don't know if the node itself is okay being in the `node` folder, or it should be directly in root? How does `reth` do it? or `rindexer`? Or other similar Ethereum rust projects?
- I've heard something about adding it to lib.rs or something - what is it and how do I do it?
- we need a gif animation in the readme under the logo - I need to record it
- we probably need a website? shinode.rs domain is free so far - maybe I should buy it before releasing it to the world

---

## Analysis

### 1. Cleanup: Directories to Remove/Move

**Definitely remove from repo (add to `.gitignore`):**
- `analysis/` - development-time exploration
- `rpc-benchmark/` - one-off testing
- `scripts/` (if just personal helper scripts)
- `Opening_storage_debug/` - debugging artifacts
- `data*/` directories - already gitignored but showing
- `logs*/` directories - same
- `repomix-*.xml` files - AI context files
- `reth/`, `rindexer/` - these are submodules/clones, not your code
- `UniswapV3.abi.json`, `rindexer.yaml`, `rindexer_csv/` - development testing
- `.venv/` - Python environment

**Maybe move to a separate repo or archive branch:**
- `spec/` - valuable development history, but clutters release. Could keep as `docs/design/` or move to wiki/separate repo

---

### 2. Root Folder MD Files Consolidation

Current: `ARCHITECTURE.md`, `CLAUDE.md`, `README.md`, `ROADMAP.md`, `SPEC.md`

**Recommended structure:**
```
README.md           # Main entry point, keep lean
CHANGELOG.md        # NEW - version history
LICENSE             # You need this!
CLAUDE.md           # Keep (useful for AI-assisted development)
docs/
  architecture.md   # Move ARCHITECTURE.md here
  roadmap.md        # Move ROADMAP.md here
  specification.md  # Move SPEC.md here (or remove if outdated)
```

---

### 3. Node Location: `node/` vs Root

**Both approaches are valid:**

| Approach | Examples | When to use |
|----------|----------|-------------|
| Code in root | Small single-crate projects | Simple projects, single binary |
| Code in subdir | reth (`crates/`), rindexer (`core/`, `cli/`) | Workspace with multiple crates |

**Recommendation:** Keep `node/` for now, but consider:
- If you want to add a library crate later → workspace with `node/` + `shinode-core/`
- If it stays a single binary → could move to root, but `node/` is fine

---

### 4. The `lib.rs` Question

**What they're suggesting:** Split your code into:
- `lib.rs` - exports the core functionality as a library
- `main.rs` - thin CLI wrapper that uses the library

**Benefits:**
1. Other Rust projects can `use shinode_core::Storage;`
2. Can be published to crates.io
3. Enables embedding in rindexer or other indexers
4. Better testing (libraries are easier to test than binaries)

**How to do it:**

```rust
// node/src/lib.rs (NEW)
pub mod cli;
pub mod p2p;
pub mod rpc;
pub mod storage;
pub mod sync;
pub mod ui;
pub mod logging;

// Re-export key types for easy access
pub use storage::Storage;
pub use sync::SyncStatus;
// etc.
```

```rust
// node/src/main.rs (simplified)
use stateless_history_node::{cli, run};

fn main() -> eyre::Result<()> {
    let config = cli::NodeConfig::from_args();
    run::run_sync(&config)
}
```

```toml
# node/Cargo.toml additions
[lib]
name = "shinode_core"  # or "stateless_history_node"
path = "src/lib.rs"

[[bin]]
name = "shinode"
path = "src/main.rs"
```

---

### 5. README Improvements

Current README is pretty good but could use:
- [ ] **Logo/banner** at top
- [ ] **GIF demo** (you mentioned this)
- [ ] **Badges** (build status, version, license)
- [ ] **Installation section** (cargo install, binary downloads)
- [ ] Trim some verbose sections (deferred compaction is very detailed)

---

### 6. CHANGELOG

**Yes, you need one.** Use [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

## [0.3.0] - 2025-02-XX
### Added
- Sealed shard cache for 10-20x faster startup on HDDs
- `db rebuild-cache` command
- TUI dashboard with speed chart, coverage map

### Changed
- Storage now auto-detects shard size from existing data

### Fixed
- 4.7 minute delay when transitioning to follow mode
```

---

### 7. Website (shinode.rs)

**Yes, grab the domain!** Even a simple landing page helps:
- What it is
- Quick start
- Link to GitHub
- Link to docs

Could be a simple GitHub Pages site or Vocs (like reth uses).

---

### 8. Pre-Release Checklist

```
[x] LICENSE file (MIT OR Apache-2.0) - added LICENSE-MIT and LICENSE-APACHE
[ ] CHANGELOG.md
[x] Clean .gitignore (remove dev artifacts)
[x] Remove/archive dev directories
[ ] Add lib.rs for library usage
[ ] Record demo GIF
[ ] Add badges to README
[ ] Tag version (v0.3.0)
[ ] Consider crates.io publishing
[ ] Buy shinode.rs domain
```

---

## Summary Priorities

1. **Must do:** LICENSE, CHANGELOG, clean up root folder
2. **Should do:** Add `lib.rs`, demo GIF, grab domain
3. **Nice to have:** Website, crates.io publishing, docs site
