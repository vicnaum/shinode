//! Splash screen mock for DOS-style intro with animated dots.
//! Run with: cargo run --manifest-path node/Cargo.toml --bin color-test

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::Clear,
};
use std::io;
use std::time::Duration;

const SPLASH_LOGO: &str = r#"   ▄████████    ▄█    █▄     ▄█  ███▄▄▄▄    ▄██████▄  ████████▄     ▄████████
  ███    ███   ███    ███   ███  ███▀▀▀██▄ ███    ███ ███   ▀███   ███    ███
  ███    █▀    ███    ███   ███▌ ███   ███ ███    ███ ███    ███   ███    █▀
  ███         ▄███▄▄▄▄███▄▄ ███▌ ███   ███ ███    ███ ███    ███  ▄███▄▄▄
▀███████████ ▀▀███▀▀▀▀███▀  ███▌ ███   ███ ███    ███ ███    ███ ▀▀███▀▀▀
         ███   ███    ███   ███  ███   ███ ███    ███ ███    ███   ███    █▄
   ▄█    ███   ███    ███   ███  ███   ███ ███    ███ ███   ▄███   ███    ███
 ▄████████▀    ███    █▀    █▀    ▀█   █▀   ▀██████▀  ████████▀    ██████████"#;

// Japanese mountain silhouette (right side, rendered first so grass can overlay)
const SPLASH_JAPANESE: [&str; 4] = [
    "                                                                \u{259D}\u{2584}     \u{2596} \u{2584}\u{2584}\u{259E}\u{259D}",
    "                                                               \u{259D}\u{2584}  \u{2596}  \u{2597}\u{2598}\u{2584}\u{2584}\u{2584}\u{2584}\u{2596}",
    "                                                                 \u{2584}\u{259E} \u{2597}\u{259E}\u{2598}   \u{2590}",
    "                                                               \u{2580}\u{2580}  \u{2580}\u{2598}    \u{2584}\u{2598}",
];

// Grass + title (left/center side, rendered on top of japanese)
const SPLASH_GRASS: [&str; 5] = [
    r"       \/|\/               STATELESS HISTORY NODE",
    r"       \\|//",
    r"        \|/",
    r"        \|/   |",
    r"     |   |   \|/",
];

const CREDIT: &str = "2026 by @vicnaum";
const VERSION: &str = "v0.3";

// Standard DOS text mode dimensions
const DOS_WIDTH: u16 = 80;
const DOS_HEIGHT: u16 = 25;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut animation_frame: u64 = 0;

    loop {
        terminal.draw(|frame| render(frame, animation_frame))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    break;
                }
            }
        }
        animation_frame += 1;
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render(frame: &mut Frame, animation_frame: u64) {
    let area = frame.area();

    // Classic DOS palette
    let dos_blue = Color::Rgb(0, 0, 170);
    let dos_gold = Color::Rgb(255, 255, 85);

    // Fill entire terminal with black
    frame.render_widget(Clear, area);
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            frame.buffer_mut()[(x, y)].set_bg(Color::Black);
        }
    }

    // Center the 80x25 DOS window on screen
    let dos_x = area.x + area.width.saturating_sub(DOS_WIDTH) / 2;
    let dos_y = area.y + area.height.saturating_sub(DOS_HEIGHT) / 2;

    let buf = frame.buffer_mut();

    // Fill DOS window area with blue
    for row in 0..DOS_HEIGHT.min(area.height) {
        let y = dos_y + row;
        if y >= area.y + area.height {
            break;
        }
        for col in 0..DOS_WIDTH.min(area.width) {
            let x = dos_x + col;
            if x >= area.x + area.width {
                break;
            }
            buf[(x, y)].set_bg(dos_blue);
        }
    }

    let art_style = Style::default().fg(dos_gold).bg(dos_blue);
    let frame_style = Style::default().fg(dos_gold).bg(dos_blue);

    // Draw double-line border at the DOS window edges
    let horiz: String = "═".repeat(DOS_WIDTH as usize - 2);
    buf.set_string(dos_x, dos_y, format!("╔{}╗", horiz), frame_style);
    buf.set_string(
        dos_x,
        dos_y + DOS_HEIGHT - 1,
        format!("╚{}╝", horiz),
        frame_style,
    );
    for row in 1..DOS_HEIGHT - 1 {
        let y = dos_y + row;
        if y < area.y + area.height {
            buf.set_string(dos_x, y, "║", frame_style);
            buf.set_string(dos_x + DOS_WIDTH - 1, y, "║", frame_style);
        }
    }

    let art_x = dos_x + 1;
    let art_y = dos_y + 1;

    // Draw logo (8 lines)
    for (i, line) in SPLASH_LOGO.lines().enumerate() {
        let y = art_y + i as u16;
        if y < area.y + area.height {
            buf.set_string(art_x, y, line, art_style);
        }
    }

    // Japanese mountain silhouette — 1 line gap below logo, stays fixed
    let japanese_y = art_y + 8 + 1;
    for (i, line) in SPLASH_JAPANESE.iter().enumerate() {
        let y = japanese_y + i as u16;
        if y < area.y + area.height {
            buf.set_string(art_x, y, *line, art_style);
        }
    }

    // Grass + title — 3 lines below japanese start (overlaps rows, left side only)
    let grass_y = japanese_y + 3;
    for (i, line) in SPLASH_GRASS.iter().enumerate() {
        let y = grass_y + i as u16;
        if y < area.y + area.height {
            buf.set_string(art_x, y, *line, art_style);
        }
    }

    // Grass-integrated separator right after grass
    let sep_y = grass_y + SPLASH_GRASS.len() as u16;
    if sep_y < area.y + area.height {
        let fill = "─".repeat(DOS_WIDTH as usize - 2 - 16);
        let separator = format!("║────\\|/──|───\\|/{}║", fill);
        buf.set_string(dos_x, sep_y, &separator, frame_style);
    }

    // Status message with animated dots — cyan, centered below separator
    let base_status = "Connecting to Ethereum P2P Network";
    let max_dots: usize = 3;
    let dot_count = (animation_frame / 5) % (max_dots as u64 + 1);
    let dots = ".".repeat(dot_count as usize);
    let pad = " ".repeat(max_dots - dot_count as usize);
    let status_msg = format!("{}{}{}", base_status, dots, pad);
    let status_style = Style::default().fg(Color::LightCyan).bg(dos_blue);

    let status_y = sep_y + 3; // 2 blank lines below separator
    if status_y < area.y + area.height {
        let status_x = dos_x + (DOS_WIDTH.saturating_sub(status_msg.len() as u16)) / 2;
        buf.set_string(status_x, status_y, &status_msg, status_style);
    }

    // Version — gold, right-aligned
    let ver_y = dos_y + DOS_HEIGHT - 3;
    if ver_y < area.y + area.height {
        let ver_x = dos_x + DOS_WIDTH - 1 - VERSION.len() as u16 - 1;
        buf.set_string(ver_x, ver_y, VERSION, art_style);
    }

    // Credit — white, right-aligned
    let credit_y = dos_y + DOS_HEIGHT - 2;
    if credit_y < area.y + area.height {
        let credit_x = dos_x + DOS_WIDTH - 1 - CREDIT.len() as u16 - 1;
        buf.set_string(
            credit_x,
            credit_y,
            CREDIT,
            Style::default().fg(Color::White).bg(dos_blue),
        );
    }
}
