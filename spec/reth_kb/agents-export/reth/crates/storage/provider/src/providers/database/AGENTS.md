# database

## Purpose
Database-backed provider factory and provider implementations.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Provider factory and wiring of database providers.
- **Key items**: `ProviderFactory`, `DatabaseProvider`, `SaveBlocksMode`

### `provider.rs`
- **Role**: Database provider implementations (RO/RW).
- **Key items**: `DatabaseProviderRO`, `DatabaseProviderRW`

### `builder.rs`
- **Role**: Provider factory builder and read-only config.
- **Key items**: `ProviderFactoryBuilder`, `ReadOnlyConfig`

### `metrics.rs`
- **Role**: Metrics for database provider operations.
- **Key items**: provider metrics types

### `chain.rs`
- **Role**: Chain storage adapter traits.
- **Key items**: `ChainStorage`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProviderFactory`, `DatabaseProvider`
