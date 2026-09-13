//! Archive phases reuse Shinode's header, phase strip, chart, panels and log renderer.
use super::{Instant, Phase, TuiState};
use crate::ui::{archive::State, human_bytes};

impl TuiState {
    pub(super) fn archive_eta(&self) -> String {
        let Some(job) = &self.archive else {
            return "--".into();
        };
        // Network counters include retries; cached archives need no transfer.
        // They cannot predict the remaining download/extraction time reliably.
        if self.current_speed == 0 || job.total == 0 || job.phase == "Downloading / extracting" {
            return "--".into();
        }
        crate::sync::format_eta_seconds(
            job.total.saturating_sub(job.done) as f64 / self.current_speed as f64,
        )
    }

    pub fn begin_history(&mut self, start: u64) {
        let logs = std::mem::take(&mut self.logs);
        let chain = self.config_chain;
        *self = Self::new(start, 0);
        self.config_chain = chain;
        self.logs = logs;
        self.archive_workflow = true;
    }

    pub fn update_archive(&mut self, snapshot: State) {
        let now = Instant::now();
        let counter = if snapshot.phase == "Downloading / extracting" {
            snapshot.downloaded / 1024
        } else {
            snapshot.done
        };
        if self
            .archive
            .as_ref()
            .is_none_or(|old| old.phase != snapshot.phase)
        {
            self.archive_meter = Some((now, now, counter, counter));
            self.speed_history.clear();
            self.current_speed = 0;
            self.avg_speed = 0;
            self.peak_speed = 0;
        }
        if let Some((started, last, base, previous)) = self.archive_meter {
            let elapsed = now.duration_since(last).as_secs_f64();
            if elapsed >= 1.0 {
                self.current_speed = (counter.saturating_sub(previous) as f64 / elapsed) as u64;
                self.avg_speed = (counter.saturating_sub(base) as f64
                    / now.duration_since(started).as_secs_f64())
                    as u64;
                self.peak_speed = self.peak_speed.max(self.current_speed);
                if self.speed_history.len() == 60 {
                    self.speed_history.pop_front();
                }
                self.speed_history.push_back(self.current_speed);
                self.archive_meter = Some((started, now, base, counter));
            }
        }
        self.config_chain = crate::chain::ChainKind::Base;
        self.phase = Phase::Archive;
        self.start_block = 0;
        self.end_block = snapshot.total;
        self.current_block = snapshot.done;
        self.blocks_to_sync = snapshot.total;
        self.inflight = snapshot.workers.len() as u64;
        self.storage_total = Some(snapshot.output as f64 / (1024.0 * 1024.0 * 1024.0));
        self.archive_workflow = true;
        self.progress = if snapshot.total == 0 {
            0.0
        } else {
            (snapshot.done as f64 / snapshot.total as f64).clamp(0.0, 1.0)
        };
        self.logs.clear();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for message in &snapshot.messages {
            self.add_log(tracing::Level::INFO, message.clone(), timestamp);
        }
        self.add_log(
            tracing::Level::INFO,
            format!("{} | {}", snapshot.title, snapshot.detail),
            timestamp,
        );
        self.add_log(
            tracing::Level::INFO,
            format!(
                "Received {} | Extracted {} | Selected {} / {} | Output {} | Elapsed {:.1}s",
                human_bytes(snapshot.downloaded),
                human_bytes(snapshot.extracted),
                human_bytes(snapshot.selected[0]),
                human_bytes(snapshot.selected[1]),
                human_bytes(snapshot.output),
                snapshot.started.elapsed().as_secs_f64()
            ),
            timestamp,
        );
        for (name, status) in &snapshot.workers {
            self.add_log(tracing::Level::INFO, format!("{name}: {status}"), timestamp);
        }
        self.archive = Some(snapshot);
    }
}
