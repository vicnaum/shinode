use super::*;
use crate::ui::tui::{render_ui, TuiState};
use eyre::eyre;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn renders_archive_stages_and_cancellation_at_terminal_sizes() -> Result<()> {
    let progress = Progress::new("Snapshot import", true);
    progress.detail("Blocks 0–49999 → /data/base".into());
    progress.selected(85_140_341, 279_526_316);
    progress.bytes(1024, true);
    progress.bytes(2048, false);
    progress.worker("headers-0-499999", "Downloading", 1024, 4096);
    progress.message("Verified receipts archive");
    for phase in [
        "Downloading / extracting",
        "Validating genesis prefix",
        "Importing",
        "Verifying restored output",
    ] {
        progress.phase(phase, 50_000, "blocks");
        progress.position(25_000);
        for (width, height) in [(120, 40), (140, 30), (100, 36)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height))?;
            let mut state = TuiState::new(0, 0);
            state.update_archive(progress.snapshot());
            terminal.draw(|frame| render_ui(frame, &state))?;
            if width == 120 {
                let text = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .map(ratatui::buffer::Cell::symbol)
                    .collect::<String>();
                ensure!(
                    text.contains(phase) && text.contains("25000 / 50000 blocks"),
                    "missing phase progress"
                );
                ensure!(text.contains("Received"), "missing byte progress");
                ensure!(text.contains("headers-0-499999"), "missing active archive");
            }
        }
    }
    let mut terminal = Terminal::new(TestBackend::new(120, 40))?;
    let mut state = TuiState::new(0, 0);
    state.update_archive(progress.snapshot());
    state.quitting = true;
    terminal.draw(|frame| render_ui(frame, &state))?;
    let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>();
    ensure!(
        text.contains("Quitting... please wait"),
        "missing cancellation status"
    );
    Ok(())
}

#[test]
fn concurrent_workers_keep_transfer_counts_separate_from_verified_progress() -> Result<()> {
    let progress = Progress::new("Snapshot import", true);
    progress.phase("Downloading / extracting", 4, "archives");
    std::thread::scope(|scope| {
        for index in 0..4 {
            let progress = &progress;
            scope.spawn(move || {
                let name = format!("archive-{index}");
                progress.worker(&name, "Downloading", 0, 1000);
                for _ in 0..10 {
                    progress.bytes(100, true);
                }
                progress.bytes(2000, false);
                progress.advance(1);
                progress.worker_done(&name);
                for _ in 0..10 {
                    progress.message("Archive verified");
                }
            });
        }
    });
    let state = progress.state.lock();
    ensure!(
        state.downloaded == 4000 && state.extracted == 8000 && state.done == 4,
        "lost worker counters"
    );
    ensure!(
        state.workers.is_empty() && state.messages.len() == 8,
        "unbounded progress state"
    );
    drop(state);
    progress.clone().cancel();
    ensure!(
        progress.check().is_err(),
        "cancellation did not reach shared worker state"
    );
    Ok(())
}

#[tokio::test]
async fn runner_awaits_worker_cleanup_and_preserves_storage_errors() -> Result<()> {
    struct Lease(Arc<AtomicBool>);
    impl Drop for Lease {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let released = Arc::new(AtomicBool::new(false));
    let worker_released = released.clone();
    let result = run(true, "Test restore", move |progress| {
        let _lease = Lease(worker_released);
        progress.cancel();
        Err(eyre!("storage I/O failed"))
    })
    .await;
    ensure!(
        released.load(Ordering::SeqCst),
        "runner returned before releasing worker lease"
    );
    let error = result.err().ok_or_else(|| eyre!("storage error lost"))?;
    ensure!(
        error.to_string() == "storage I/O failed",
        "cancellation replaced a storage error"
    );
    ensure!(
        run(true, "Cancelled import", |progress| {
            progress.cancel();
            progress.check()
        })
        .await
        .is_err(),
        "cancelled worker reported success"
    );
    Ok(())
}

#[test]
fn archive_handoff_keeps_shared_layout_and_resets_phase_metrics() -> Result<()> {
    let progress = Progress::new("Snapshot import", true);
    progress.message("Archive boundary verified");
    let mut state = TuiState::new(0, 0);
    state.update_archive(progress.snapshot());
    state.current_speed = 123;
    state.begin_history(50_000);
    ensure!(
        state.archive.is_none() && state.archive_workflow && state.current_speed == 0,
        "archive metrics leaked into history"
    );
    ensure!(
        state
            .logs
            .iter()
            .any(|entry| entry.message == "Archive boundary verified"),
        "handoff lost archive completion log"
    );
    for phase in [
        Phase::Startup,
        Phase::Sync,
        Phase::Retry,
        Phase::Compact,
        Phase::Seal,
        Phase::Follow,
    ] {
        state.phase = phase;
        for (width, height) in [(120, 40), (140, 30), (100, 36)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height))?;
            terminal.draw(|frame| render_ui(frame, &state))?;
            if width >= 100 && height >= 36 {
                let text = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .map(ratatui::buffer::Cell::symbol)
                    .collect::<String>();
                ensure!(
                    text.contains("ARCHIVE") && text.contains("HISTORY") && text.contains("FOLLOW"),
                    "workflow phase strip missing"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn archive_eta_uses_phase_units_and_does_not_guess_download_work() -> Result<()> {
    let progress = Progress::new("Snapshot import", true);
    progress.phase("Validating archive prefix", 100, "blocks");
    progress.position(50);
    let mut state = TuiState::new(0, 0);
    state.update_archive(progress.snapshot());
    state.current_speed = 10;
    ensure!(
        state.eta_string() == crate::sync::format_eta_seconds(5.0),
        "archive ETA used P2P counters"
    );
    progress.phase("Downloading / extracting", 3, "archives");
    state.update_archive(progress.snapshot());
    state.current_speed = 1024;
    ensure!(
        state.eta_string() == "--",
        "download ETA mixed archives and bytes"
    );
    Ok(())
}

#[test]
fn archive_uses_original_panels_and_base_only_phase_labels() -> Result<()> {
    let progress = Progress::new("Snapshot import", true);
    progress.phase("Importing", 50000, "blocks");
    progress.position(25000);
    let mut state = TuiState::new(0, 0);
    state.update_archive(progress.snapshot());
    for (width, height) in [(120, 40), (140, 30)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height))?;
        terminal.draw(|frame| render_ui(frame, &state))?;
        let text = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        for original in [
            "NETWORK",
            "QUEUE",
            "STORAGE",
            "DB",
            "Speed (blocks/s)",
            "ARCHIVE",
            "HISTORY",
            "FOLLOW",
        ] {
            ensure!(
                text.contains(original),
                "missing original widget or Base phase: {original}"
            );
        }
        if height >= 36 {
            ensure!(text.contains("Map"), "original map missing");
        }
        ensure!(
            !text.contains("TRANSFER") && !text.contains("ACTIVE FILES"),
            "custom panel layout returned"
        );
    }
    state.begin_history(50000);
    state.phase = Phase::Sync;
    for chain in [
        crate::chain::ChainKind::Mainnet,
        crate::chain::ChainKind::Base,
    ] {
        state.config_chain = chain;
        let mut terminal = Terminal::new(TestBackend::new(120, 40))?;
        terminal.draw(|frame| render_ui(frame, &state))?;
        let text = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        if chain == crate::chain::ChainKind::Base {
            ensure!(
                text.contains("ARCHIVE") && text.contains("HISTORY"),
                "Base phases missing"
            );
        } else {
            ensure!(
                text.contains("STARTUP")
                    && text.contains("SYNC")
                    && !text.contains("ARCHIVE")
                    && !text.contains("HISTORY"),
                "Ethereum phase strip changed"
            );
        }
    }
    Ok(())
}
