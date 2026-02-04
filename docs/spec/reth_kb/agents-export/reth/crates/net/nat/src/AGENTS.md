# src

## Purpose
Implements `reth-net-nat`: helpers for NAT/external address resolution (public IP lookup, UPnP placeholder, domain-to-IP resolution, and container network-interface IP lookup), including a periodic resolver.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines `NatResolver` (resolver strategy enum), parsing/display, and async helpers to resolve external IPs (via public IP services, DNS, or net-if), plus `ResolveNatInterval` for periodic resolution.
- **Key items**: `NatResolver`, `external_ip()`, `external_addr_with()`, `ResolveNatInterval`
- **Interactions**: uses `reqwest` to query public IP services; uses `tokio::time::Interval` to schedule polling.

#### `net_if.rs`
- **Role**: OS interface IP lookup (useful in docker bridge networks).
- **Key items**: `DEFAULT_NET_IF_NAME`, `resolve_net_if_ip()`, `NetInterfaceError`

## Key APIs (no snippets)
- `NatResolver`
- `ResolveNatInterval`
