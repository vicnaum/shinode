# bin

## Purpose

Demo and testing binaries for TUI development. These standalone executables allow iterating on the ratatui dashboard design, validating Unicode character rendering, and testing terminal color schemes without running the full node.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `ui_mock.rs` - Interactive TUI dashboard mock with simulated sync data for all phases.
- `color_test.rs` - DOS-style splash screen with animated status dots and terminal color demonstration.
- `char_test.rs` - Unicode character width testing for peer dot rendering across terminal emulators.

## Files (detailed)

### `ui_mock.rs`
- **Role**: Interactive mock of the complete ratatui dashboard with simulated sync data (~1576 lines). Cycles through all phases: Sync, Retry, Compact, Seal, Follow.
- **Key items**: `Phase` (Sync, Retry, Compact, Seal, Follow), `MockData`, `main()`, `render_ui()`, `render_sync_ui()`, `render_compact_ui()`, `render_follow_ui()`, `render_blocks_map()`, `render_shards_map()`, `render_speed_chart()`, `render_synced_status()`, `render_big_number()`
- **UI panels**: Header, phase indicator, progress bar, blocks map (braille), shards map, speed chart (histogram), network panel (peer dots), queue panel, storage panel, DB panel, compaction panel, synced status (big ASCII number digits), logs panel.
- **Controls**: `q` = quit, `n` = next phase. Updates at 20 FPS (`tick_rate` = 50ms) with randomized data.
- **Interactions**: Standalone; no imports from parent crate. Uses `ratatui`, `crossterm`, `rand`.
- **Run**: `cargo run --manifest-path node/Cargo.toml --bin ui-mock`

### `color_test.rs`
- **Role**: DOS-style splash screen showing "SHINODE" ASCII logo with animated "Connecting to Ethereum P2P Network..." status dots (~199 lines). Uses DOS blue background (`Rgb(0, 0, 170)`) and gold text (`Rgb(255, 255, 85)`). Includes Japanese mountain silhouette and grass/title overlays inside a double-line border frame.
- **Key items**: `main()`, `render()`, `SPLASH_LOGO` (8-line ASCII art), `SPLASH_JAPANESE` (4-line mountain silhouette), `SPLASH_GRASS` (5-line grass + title), `CREDIT`, `VERSION`, `DOS_WIDTH`, `DOS_HEIGHT`
- **Interactions**: Standalone. Press any key to exit. Animated at 100ms poll interval.
- **Knobs / invariants**:
  - `DOS_WIDTH` = 80, `DOS_HEIGHT` = 25 (standard DOS text mode)
  - Centered on screen with black surround, blue DOS window, gold text
- **Run**: `cargo run --manifest-path node/Cargo.toml --bin color-test`

### `char_test.rs`
- **Role**: Unicode character width testing for peer dot rendering (~124 lines). Validates which characters render consistently across terminal emulators by displaying them in two rendering modes side-by-side.
- **Key items**: `main()`, `render()`, `CANDIDATES` (9 character sets: original circles, bullets, ASCII, blocks, braille, squares, diamonds, stars, shading)
- **Rendering modes**: Default 1-cell-per-char (`set_string`) and manual 2-cell-per-char placement. A correctly rendering character set has tight dots with no gaps and aligned "| end" markers.
- **Interactions**: Standalone. Press any key to exit.
- **Knobs / invariants**:
  - Dark background (`Rgb(30, 30, 30)`) for contrast
  - Active/idle in green, stale in dark gray
- **Run**: `cargo run --manifest-path node/Cargo.toml --bin char-test`

## Key APIs (no snippets)

No public APIs. These are standalone binaries:
- **Binaries**: `ui-mock`, `color-test`, `char-test` (defined in `Cargo.toml`)

## Relationships

- **Depends on**: `ratatui`, `crossterm`, `rand` (external only; no parent crate imports)
- **Used by**: Developers for manual TUI design iteration and terminal compatibility testing
- **Related to**: `ui/tui.rs` implements the actual TUI dashboard; these binaries serve as design previews
- **Data/control flow**: Self-contained; simulate data internally

## End-to-end flow (high level)

1. Developer runs one of the 3 binaries to test a specific TUI aspect
2. `ui_mock`: Simulates realistic node metrics at 20 FPS; press 'n' to cycle through Sync/Retry/Compact/Seal/Follow layouts
3. `color_test`: Renders DOS-style splash with blue/gold palette and animated connection dots; verifies border drawing and centering
4. `char_test`: Displays 9 candidate character sets in two rendering modes; developer identifies which renders with correct spacing
5. Findings inform character/color choices in production `ui/tui.rs`

## Notes

