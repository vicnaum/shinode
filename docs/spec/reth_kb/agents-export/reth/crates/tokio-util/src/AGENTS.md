# src

## Purpose
Tokio utilities for event broadcasting and rate limiting.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and public exports.
  - **Key items**: `EventSender`, `EventStream`, `ratelimit` (feature-gated)
- `event_sender.rs` - Broadcast event sender with listener creation.
  - **Key items**: `EventSender`, `notify()`, `new_listener()`
- `event_stream.rs` - Stream wrapper that skips broadcast lag errors.
  - **Key items**: `EventStream`
- `ratelimit.rs` - Simple rate limiter for async workflows (feature `time`).
  - **Key items**: `RateLimit`, `Rate`, `poll_ready()`, `tick()`

## Key APIs (no snippets)
- `EventSender`, `EventStream`
- `RateLimit`, `Rate`
