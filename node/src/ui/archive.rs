//! Fullscreen progress for bounded archive jobs; independent of P2P sync statistics.
use super::human_bytes;
use super::tui::{Phase, TuiController};
use eyre::{ensure, Result};
use parking_lot::Mutex;
use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, IsTerminal},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct Progress {
    state: Arc<Mutex<State>>,
    cancelled: Arc<AtomicBool>,
    fullscreen: bool,
}

#[derive(Clone)]
pub struct State {
    pub(crate) title: String,
    pub(crate) phase: String,
    pub(crate) detail: String,
    pub(crate) done: u64,
    pub(crate) total: u64,
    pub(crate) unit: &'static str,
    pub(crate) downloaded: u64,
    pub(crate) extracted: u64,
    pub(crate) selected: [u64; 2],
    pub(crate) output: u64,
    pub(crate) workers: BTreeMap<String, String>,
    pub(crate) messages: VecDeque<String>,
    pub(crate) started: Instant,
    pub(crate) activity: String,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new("Archive job", false)
    }
}

impl Progress {
    fn new(title: &str, fullscreen: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                title: title.into(),
                phase: "Inspecting".into(),
                detail: String::new(),
                done: 0,
                total: 0,
                unit: "items",
                downloaded: 0,
                extracted: 0,
                selected: [0, 0],
                output: 0,
                workers: BTreeMap::new(),
                messages: VecDeque::new(),
                started: Instant::now(),
                activity: String::new(),
            })),
            cancelled: Arc::new(AtomicBool::new(false)),
            fullscreen,
        }
    }

    pub fn check(&self) -> Result<()> {
        ensure!(
            !self.cancelled.load(Ordering::Relaxed),
            "archive job cancelled; repeat the command to resume"
        );
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn phase(&self, name: &str, total: u64, unit: &'static str) {
        let mut state = self.state.lock();
        state.phase = name.into();
        state.total = total;
        state.unit = unit;
        state.done = 0;
        state.activity.clear();
    }

    pub fn activity(&self, text: &str) {
        self.state.lock().activity = text.into();
    }
    pub fn position(&self, done: u64) {
        self.state.lock().done = done;
    }
    pub fn advance(&self, count: u64) {
        let mut state = self.state.lock();
        state.done = state.done.saturating_add(count);
    }
    pub fn detail(&self, detail: String) {
        self.state.lock().detail = detail;
    }
    pub fn selected(&self, compressed: u64, extracted: u64) {
        self.state.lock().selected = [compressed, extracted];
    }
    pub fn output(&self, bytes: u64) {
        self.state.lock().output = bytes;
    }
    pub fn bytes(&self, count: u64, network: bool) {
        let mut state = self.state.lock();
        if network {
            state.downloaded = state.downloaded.saturating_add(count);
        } else {
            state.extracted = state.extracted.saturating_add(count);
        }
    }
    pub fn worker(&self, name: &str, status: &str, done: u64, total: u64) {
        let mut state = self.state.lock();
        // Only active workers are retained; completion goes into the bounded log.
        state.workers.insert(
            name.into(),
            format!("{status}: {} / {}", human_bytes(done), human_bytes(total)),
        );
    }
    pub fn worker_done(&self, name: &str) {
        self.state.lock().workers.remove(name);
    }
    pub fn message(&self, message: &str) {
        if !self.fullscreen {
            eprintln!("{message}");
        }
        let mut state = self.state.lock();
        if state.messages.len() == 8 {
            state.messages.pop_front();
        }
        state.messages.push_back(message.into());
    }
}

/// Keep the worker alive until it releases its storage leases, even after a UI error.
pub async fn run(
    no_tui: bool,
    title: &str,
    job: impl FnOnce(Progress) -> Result<()> + Send + 'static,
) -> Result<()> {
    run_with_handoff(no_tui, title, false, job).await.map(drop)
}

pub async fn run_with_handoff(
    no_tui: bool,
    title: &str,
    handoff: bool,
    job: impl FnOnce(Progress) -> Result<()> + Send + 'static,
) -> Result<Option<TuiController>> {
    let fullscreen = !no_tui
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && io::stderr().is_terminal();
    let mut screen = if fullscreen {
        Some(TuiController::new(0, 0)?)
    } else {
        None
    };
    let progress = Progress::new(title, fullscreen);
    let worker_progress = progress.clone();
    let mut worker = tokio::task::spawn_blocking(move || job(worker_progress));
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let mut display_error = None;
    let signal = tokio::signal::ctrl_c();
    tokio::pin!(signal);
    let mut signal_received = false;
    let result = loop {
        tokio::select! {
            result = &mut worker => break result,
            _ = &mut signal, if !signal_received => {
                signal_received = true;
                progress.cancel();
            }
            _ = interval.tick() => {
                if let Some(ui) = &mut screen {
                    if let Err(error) = tick(ui, &progress) {
                        progress.cancel();
                        display_error = Some(error);
                        screen = None;
                    }
                }
            }
        }
    };
    let retained = if handoff
        && result.as_ref().is_ok_and(std::result::Result::is_ok)
        && progress.check().is_ok()
        && display_error.is_none()
    {
        screen.take()
    } else {
        None
    };
    drop(screen);
    if fullscreen && retained.is_none() {
        for message in &progress.state.lock().messages {
            eprintln!("{message}");
        }
    }
    // Genuine worker/storage errors take precedence over a display failure.
    result??;
    if let Some(error) = display_error {
        return Err(error.into());
    }
    progress.check()?;
    if retained.is_none() {
        eprintln!("{title} complete");
    }
    Ok(retained)
}

fn tick(ui: &mut TuiController, progress: &Progress) -> io::Result<()> {
    if ui.poll_quit()? {
        progress.cancel();
    }
    ui.state.update_archive(progress.snapshot());
    ui.state.phase = Phase::Archive;
    ui.state.quitting = progress.cancelled.load(Ordering::Relaxed);
    ui.draw()
}

impl Progress {
    pub fn snapshot(&self) -> State {
        self.state.lock().clone()
    }
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
