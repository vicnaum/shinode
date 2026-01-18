# db_tool

## Purpose
Helper utilities for inspecting and operating on database tables from tooling/CLI.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: DB helper tool and filters for listing/querying table data.
- **Key items**: `DbTool`, `ListFilter`, `list()`, `get()`, `drop_table()`
- **Interactions**: Uses `RawTable` and cursor walkers to scan tables.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DbTool`, `ListFilter`
- **Functions**: `list()`, `get()`, `drop()`
