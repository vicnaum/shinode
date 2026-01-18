# nat

## Purpose
`reth-net-nat` crate: NAT helpers for reth networking-resolve an external/public IP address via multiple strategies (public IP services, domain, net interface) and optionally poll on an interval.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `NatResolver`, interval-based resolver, and net-interface IP lookup.

### Files
- `Cargo.toml` - crate manifest (reqwest + tokio; optional serde support for `NatResolver`).

## Key APIs (no snippets)
- `NatResolver`
- `ResolveNatInterval`
