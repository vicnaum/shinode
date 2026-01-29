# bin

## Purpose
Binary entry points and development/UI demonstration tools. Contains specialized binaries for
testing terminal rendering and previewing the TUI dashboard design, separate from the main
production binary (`stateless-history-node` built from `src/main.rs`).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `color_test.rs` - DOS-style splash screen and terminal color capability demonstration.
  - **Key items**: `main()`, `render()`, `SPLASH_ART`, `CREDIT`, `ART_WIDTH`, `ART_HEIGHT`
- `ui_mock.rs` - Interactive TUI dashboard mock with simulated sync data for all phases.
  - **Key items**: `main()`, `render_ui()`, `MockData`, `Phase`, `render_sync_ui()`, `render_compact_ui()`, `render_follow_ui()`

## Key APIs (no snippets)
- **Binaries**: `color-test`, `ui-mock` (defined in `Cargo.toml`)

## Relationships
- **Used by**: Development and demonstration workflows only; not used by production code.
- **Depends on**: `ratatui`, `crossterm`, `rand` crates only (no sync/storage/p2p dependencies).
- **Related to**: `ui/tui.rs` implements the actual TUI dashboard; these binaries serve as design previews.

## Files (detailed)

### `color_test.rs`
- **Role**: Retro DOS-style splash screen showing "STATELESS HISTORY NODE" ASCII art with DOS blue
  background (`Rgb(0, 0, 170)`) and gold text (`Rgb(255, 255, 85)`). Responsive layout with optional
  double-line border frame (`+84x28` terminal). Press any key to exit.
- **Run**: `cargo run --manifest-path node/Cargo.toml --bin color-test`

### `ui_mock.rs`
- **Role**: Interactive mock of the complete ratatui dashboard with simulated sync data.
  Cycles through all phases: Sync, Retry, Compact, Seal, Follow.
- **Key types**: `MockData` (simulated state), `Phase` enum (Sync/Retry/Compact/Seal/Follow)
- **UI panels**: Header, phase indicator, progress bar, blocks map (braille), speed chart (histogram),
  network panel (peer dots), queue panel, storage panel, DB panel, compaction panel, synced status
  (big ASCII number digits), logs panel.
- **Controls**: `q` = quit, `n` = next phase. Updates at 20 FPS with randomized data.
- **Run**: `cargo run --manifest-path node/Cargo.toml --bin ui-mock`
