# src

## Purpose
Implements `reth-net-banlist`: simple ban/allow-list utilities for networking-maintains banned peer IDs and IPs (optionally with expiry) and provides an `IpFilter` to restrict communication to allowed CIDR ranges.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `BanList` (ban/unban, expiry eviction, global-IP gating) and `IpFilter` (CIDR allow-list parsing and checks).
  - **Key items**: `BanList`, `BanList::ban_ip_with()`, `evict()`, `is_banned()`, `IpFilter`, `IpFilter::from_cidr_string()`
