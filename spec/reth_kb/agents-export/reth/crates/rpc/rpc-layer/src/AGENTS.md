# src

## Purpose
HTTP middleware layers for RPC auth, compression, and JWT validation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public exports for RPC layers.
- **Key items**: `AuthLayer`, `AuthClientLayer`, `CompressionLayer`, `JwtAuthValidator`, `AuthValidator`

### `auth_layer.rs`
- **Role**: Server-side authorization layer that validates JWT headers.
- **Key items**: `AuthLayer`, `AuthService`, `ResponseFuture`
- **Interactions**: Delegates validation to `AuthValidator`.

### `auth_client_layer.rs`
- **Role**: Client-side layer that injects JWT bearer tokens.
- **Key items**: `AuthClientLayer`, `AuthClientService`, `secret_to_bearer_header()`

### `compression_layer.rs`
- **Role**: Response compression layer for RPC HTTP responses.
- **Key items**: `CompressionLayer`, `CompressionService`

### `jwt_validator.rs`
- **Role**: JWT validation implementation for server auth.
- **Key items**: `JwtAuthValidator`, `validate()`, `get_bearer()`

## End-to-end flow (high level)
- Server wraps RPC HTTP with `AuthLayer` using `JwtAuthValidator`.
- Clients can use `AuthClientLayer` to add JWT headers.
- `CompressionLayer` compresses responses based on `Accept-Encoding`.

## Key APIs (no snippets)
- `AuthLayer`, `AuthClientLayer`, `JwtAuthValidator`, `CompressionLayer`
