# chainspec

## Purpose
`reth-optimism-chainspec` crate: Optimism/Base chain specs, hardfork scheduling, and superchain registry integration.

## Contents (one hop)
### Subdirectories
- [x] `res/` - (skip: data/assets only) genesis JSONs and superchain config tarball.
- [x] `src/` - Chain spec builders, base fee utilities, and superchain helpers.

### Files
- `Cargo.toml` - Manifest for chain spec and superchain config features.
  - **Key items**: features `superchain-configs`, `serde`
