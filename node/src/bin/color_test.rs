//! Splash screen mock for Turbo Pascal style intro.
//! Run with: cargo run --manifest-path node/Cargo.toml --bin color-test

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear},
};
use std::io;

// 23 lines of art + line 24 has the credit right-aligned to col 80
const SPLASH_ART: &str = r#" ██████\\████████  ██████\\████████ ████████ ██     | ████████  ██████\  ██████\
██___\██  | ██  | ██__| ██  | ██  | ██__   | ██     | ██__   | ██___\██ ██___\██
██    \   | ██  | ██    ██  | ██  | ██  \  | ██     | ██  \   \██    \ \██    \
\██████\  | ██  | ████████  | ██  | █████  | ██     | █████   _\██████\_\██████\
 \__| ██  | ██  | ██  | ██  | ██  | ██_____| ██_____| ██_____|  \__| ██  \__| ██
██    ██  | ██  | ██  | ██  | ██  | ██     \ ██     \ ██     \\██    ██\██    ██
\██████    \██   \██   \██   \██   \████████\████████\████████ \██████  \██████
        |  \  |  \      \/      \|        \/      \|       \|  \    /  \
        | ██  | ██\██████  ██████\\████████  ██████\ ███████\\██\  /  ██
        | ██__| ██ | ██ | ██___\██  | ██  | ██  | ██ ██__| ██ \██\/  ██
        | ██    ██ | ██  \██    \   | ██  | ██  | ██ ██    ██  \██  ██
        | ████████ | ██  _\██████\  | ██  | ██  | ██ ███████\   \████
        | ██  | ██_| ██_|  \__| ██  | ██  | ██__/ ██ ██  | ██   | ██
        | ██  | ██   ██ \\██    ██  | ██   \██    ██ ██  | ██   | ██
         \██   \██\██████ \██████    \██    \██████ \██   \██    \██
                      |  \  |  \/      \|       \|        \
                     | ██\ | ██  ██████\ ███████\ ████████
                     | ███\| ██ ██  | ██ ██  | ██ ██__
                     | ████\ ██ ██  | ██ ██  | ██ ██  \
                     | ██\██ ██ ██  | ██ ██  | ██ █████
                     | ██ \████ ██__/ ██ ██__/ ██ ██_____
                     | ██  \███\██    ██ ██    ██ ██     \
                      \██   \██ \██████ \███████ \████████"#;

const CREDIT: &str = "2026 by @vicnaum";

const ART_WIDTH: u16 = 80;
const ART_HEIGHT: u16 = 23;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| render(frame))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                break;
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render(frame: &mut Frame) {
    let area = frame.area();

    // Classic DOS palette
    let dos_blue = Color::Rgb(0, 0, 170);
    let dos_gold = Color::Rgb(255, 255, 85);

    // Fill entire screen with blue
    frame.render_widget(Clear, area);
    let bg = Block::default().style(Style::default().bg(dos_blue));
    frame.render_widget(bg, area);

    let buf = frame.buffer_mut();
    let art_style = Style::default().fg(dos_gold).bg(dos_blue);

    let lines: Vec<&str> = SPLASH_ART.lines().collect();

    // Center the logo (art + 1 credit line = 24 lines total)
    let total_height = ART_HEIGHT + 1; // art + credit on last line
    let start_y = area.y + area.height.saturating_sub(total_height) / 2;
    let start_x = area.x + area.width.saturating_sub(ART_WIDTH) / 2;

    let status_msg = "Connecting to Ethereum P2P Network...";
    let status_style = Style::default().fg(Color::LightCyan).bg(dos_blue);

    // Two modes:
    // - Framed: border + padding around art, needs >= 84x28
    // - Framed + status below: needs >= 84x33 (frame + 1 padding + status line + extra)
    let framed = area.width >= ART_WIDTH + 4 && area.height >= total_height + 4;
    let framed_with_status = framed && area.height >= total_height + 4 + 5;

    let (art_x, art_y, frame_bottom_y) = if framed {
        let frame_w = ART_WIDTH + 4;
        let frame_h = total_height + 4;
        let frame_x = area.x + area.width.saturating_sub(frame_w) / 2;
        let frame_y = area.y + area.height.saturating_sub(frame_h) / 2;

        let frame_style = Style::default().fg(dos_gold).bg(dos_blue);

        // Draw double-line border
        let horiz: String = "═".repeat(frame_w as usize - 2);
        buf.set_string(frame_x, frame_y, format!("╔{}╗", horiz), frame_style);
        buf.set_string(frame_x, frame_y + frame_h - 1, format!("╚{}╝", horiz), frame_style);
        for row in 1..frame_h - 1 {
            buf.set_string(frame_x, frame_y + row, "║", frame_style);
            buf.set_string(frame_x + frame_w - 1, frame_y + row, "║", frame_style);
        }

        (frame_x + 2, frame_y + 2, frame_y + frame_h - 1)
    } else {
        (start_x, start_y, 0)
    };

    // Draw art lines
    for (i, line) in lines.iter().enumerate() {
        let y = art_y + i as u16;
        if y < area.y + area.height {
            buf.set_string(art_x, y, *line, art_style);
        }
    }

    // Version — gold, right-aligned on the last art line (above credit)
    let last_art_y = art_y + ART_HEIGHT - 1;
    if last_art_y < area.y + area.height {
        let version = "v0.3";
        let version_x = art_x + ART_WIDTH - version.len() as u16;
        buf.set_string(version_x, last_art_y, version, art_style);
    }

    // Credit line — white, right-aligned on the line after the art
    let credit_y = art_y + ART_HEIGHT;
    if credit_y < area.y + area.height {
        let credit_x = art_x + ART_WIDTH - CREDIT.len() as u16;
        buf.set_string(
            credit_x,
            credit_y,
            CREDIT,
            Style::default().fg(Color::White).bg(dos_blue),
        );
    }

    // Status message placement depends on mode
    if framed_with_status {
        // Big screen: centered below the frame, with 1 line padding
        let status_x = area.x + area.width.saturating_sub(status_msg.len() as u16) / 2;
        let status_y = frame_bottom_y + 2; // 1 line padding below frame
        if status_y < area.y + area.height {
            buf.set_string(status_x, status_y, status_msg, status_style);
        }
    } else {
        // Small screen: left-aligned on the credit line (opposite of credit)
        let status_y = credit_y;
        if status_y < area.y + area.height {
            buf.set_string(art_x, status_y, status_msg, status_style);
        }
    }
}
