# banlist

## Purpose
`reth-net-banlist` crate: supports banning peers/IPs (with optional expiry) and filtering allowed IP ranges (CIDR) for network communications.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `BanList` and `IpFilter` implementations.

### Files
- `Cargo.toml` - crate manifest (uses `ipnet` and `alloy-primitives` for peer IDs).

## Key APIs (no snippets)
- `BanList`
- `IpFilter`
