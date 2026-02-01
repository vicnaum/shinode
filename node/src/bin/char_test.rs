//! Character width test for peer dot candidates.
//! Run with: cargo run --manifest-path node/Cargo.toml --bin char-test

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::Clear};
use std::io;

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

/// Each candidate: (label, filled_char, empty_char, stale_char)
const CANDIDATES: &[(&str, char, char, char)] = &[
    ("A) ● ○ ⊘  (original)", '\u{25CF}', '\u{25CB}', '\u{2298}'),
    ("B) • ◦ ∘  (bullets)", '\u{2022}', '\u{25E6}', '\u{2218}'),
    ("C) @ o x  (ASCII)", '@', 'o', 'x'),
    ("D) ▮ ▯ ▪  (blocks)", '\u{25AE}', '\u{25AF}', '\u{25AA}'),
    ("E) ⣿ ⡇ ⠶  (braille)", '\u{28FF}', '\u{2847}', '\u{2836}'),
    ("F) ■ □ ▣  (squares)", '\u{25A0}', '\u{25A1}', '\u{25A3}'),
    ("G) ◆ ◇ ◈  (diamonds)", '\u{25C6}', '\u{25C7}', '\u{25C8}'),
    ("H) ★ ☆ ✦  (stars)", '\u{2605}', '\u{2606}', '\u{2726}'),
    ("I) █ ░ ▒  (shading)", '\u{2588}', '\u{2591}', '\u{2592}'),
];

fn render(frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    let bg = Color::Rgb(30, 30, 30);
    let buf = frame.buffer_mut();

    // Fill background
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_style(Style::default().bg(bg));
        }
    }

    let header_style = Style::default().fg(Color::White).bold().bg(bg);
    let label_style = Style::default().fg(Color::Gray).bg(bg);
    let active_style = Style::default().fg(Color::Green).bg(bg);
    let idle_style = Style::default().fg(Color::Green).bg(bg);
    let stale_style = Style::default().fg(Color::DarkGray).bg(bg);
    let pipe_style = Style::default().fg(Color::DarkGray).bg(bg);

    let x0 = area.x + 2;
    let mut y = area.y + 1;

    buf.set_string(x0, y, "PEER DOT CHARACTER TEST — press any key to quit", header_style);
    y += 2;

    // ── Section 1: set_string (ratatui default, 1 cell per char) ──
    buf.set_string(x0, y, "=== set_string (default: 1 cell/char) ===", header_style);
    y += 1;
    for (label, filled, empty, stale) in CANDIDATES {
        if y >= area.y + area.height - 1 { break; }
        buf.set_string(x0, y, *label, label_style);
        let col = x0 + 24;
        let s: String = (0..3).map(|_| *filled)
            .chain((0..4).map(|_| *empty))
            .chain((0..2).map(|_| *stale))
            .collect();
        buf.set_string(col, y, &s, active_style);
        // Color each section properly
        for i in 0..3 { buf[(col + i, y)].set_style(active_style); }
        for i in 3..7 { buf[(col + i, y)].set_style(idle_style); }
        for i in 7..9 { buf[(col + i, y)].set_style(stale_style); }
        buf.set_string(col + 9, y, "| end", pipe_style);
        y += 1;
    }

    y += 1;

    // ── Section 2: manual 2-cell-per-char placement ──
    buf.set_string(x0, y, "=== manual placement (2 cells/char) ===", header_style);
    y += 1;
    for (label, filled, empty, stale) in CANDIDATES {
        if y >= area.y + area.height - 1 { break; }
        buf.set_string(x0, y, *label, label_style);
        let start = x0 + 24;
        let mut col = start;
        // Write each char individually, skip 2 cells per char
        for i in 0..3u16 {
            buf[(col, y)].set_char(*filled).set_style(active_style);
            buf[(col + 1, y)].set_char(' ').set_style(active_style);
            col += 2;
        }
        for i in 0..4u16 {
            buf[(col, y)].set_char(*empty).set_style(idle_style);
            buf[(col + 1, y)].set_char(' ').set_style(idle_style);
            col += 2;
        }
        for i in 0..2u16 {
            buf[(col, y)].set_char(*stale).set_style(stale_style);
            buf[(col + 1, y)].set_char(' ').set_style(stale_style);
            col += 2;
        }
        buf.set_string(col, y, "| end", pipe_style);
        y += 1;
    }

    y += 1;
    if y < area.y + area.height {
        buf.set_string(x0, y, "Good = dots tight with no gaps, '| end' aligned", label_style);
    }
}
