//! JSON log writer and tracing layer for structured logging.

use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use parking_lot::Mutex;
use std::{
    hash::{Hash, Hasher},
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError, SyncSender, TrySendError},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tracing::Event;
use tracing_subscriber::Layer;

/// Buffer size for JSON log channel.
pub const LOG_BUFFER: usize = 10_000;

/// Flush after this many records (count-based).
const FLUSH_COUNT: usize = 4096;

/// Flush after this duration (time-based, for timely log visibility during debugging).
const FLUSH_INTERVAL: Duration = Duration::from_secs(2);

/// Timeout for recv_timeout to allow periodic flushing even without new records.
const RECV_TIMEOUT: Duration = Duration::from_millis(500);

/// A single log record serialized to JSON.
#[derive(Debug, Serialize)]
pub struct LogRecord {
    pub t_ms: u64,
    pub level: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: Option<String>,
    pub fields: JsonMap<String, JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

/// Visitor that collects tracing fields into a JSON map.
#[derive(Default)]
pub struct JsonLogVisitor {
    pub fields: JsonMap<String, JsonValue>,
}

impl tracing::field::Visit for JsonLogVisitor {
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Number(value.into()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), JsonValue::Number(value.into()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let number = serde_json::Number::from_f64(value)
            .map_or_else(|| JsonValue::String(value.to_string()), JsonValue::Number);
        self.fields.insert(field.name().to_string(), number);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(
            field.name().to_string(),
            JsonValue::String(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            JsonValue::String(format!("{value:?}")),
        );
    }
}

/// Window within which identical consecutive records are deduplicated.
const DEDUP_WINDOW: Duration = Duration::from_secs(1);

/// Hash a record by its message and fields (ignoring t_ms, level, target, file, line).
fn hash_record(record: &LogRecord) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    record.message.hash(&mut hasher);
    // Hash fields deterministically by sorting keys
    let mut keys: Vec<&String> = record.fields.keys().collect();
    keys.sort();
    for key in keys {
        key.hash(&mut hasher);
        if let Some(val) = record.fields.get(key) {
            // Use the JSON string representation for hashing
            val.to_string().hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Async JSON log writer that writes records to a file in a background thread.
#[derive(Debug)]
pub struct JsonLogWriter {
    started_at: Instant,
    sender: Mutex<Option<SyncSender<LogRecord>>>,
    handle: Mutex<Option<JoinHandle<eyre::Result<()>>>>,
    dropped_events: AtomicU64,
    total_events: AtomicU64,
}

impl JsonLogWriter {
    /// Create a new JSON log writer that writes to the specified path.
    pub fn new(path: &std::path::Path, capacity: usize) -> eyre::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        let (tx, rx) = mpsc::sync_channel::<LogRecord>(capacity);
        let handle = std::thread::spawn(move || -> eyre::Result<()> {
            let mut since_flush = 0usize;
            let mut last_flush = Instant::now();

            // Dedup state: collapse consecutive identical records within DEDUP_WINDOW
            let mut prev_key: Option<u64> = None;
            let mut prev_record: Option<LogRecord> = None;
            let mut dup_count: u64 = 0;
            let mut dup_since: Instant = Instant::now();

            /// Flush a buffered dedup record to the writer.
            fn flush_prev(
                writer: &mut BufWriter<std::fs::File>,
                prev: &mut Option<LogRecord>,
                dup_count: &mut u64,
                since_flush: &mut usize,
            ) -> eyre::Result<()> {
                if let Some(mut rec) = prev.take() {
                    if *dup_count > 0 {
                        rec.count = Some(*dup_count + 1);
                    }
                    serde_json::to_writer(&mut *writer, &rec)?;
                    writer.write_all(b"\n")?;
                    *since_flush = since_flush.saturating_add(1);
                    *dup_count = 0;
                }
                Ok(())
            }

            loop {
                // Use recv_timeout to allow periodic flushing even without new records
                match rx.recv_timeout(RECV_TIMEOUT) {
                    Ok(record) => {
                        let key = hash_record(&record);
                        if prev_key == Some(key) && dup_since.elapsed() < DEDUP_WINDOW {
                            dup_count += 1;
                        } else {
                            // Flush previous record
                            flush_prev(&mut writer, &mut prev_record, &mut dup_count, &mut since_flush)?;
                            prev_key = Some(key);
                            prev_record = Some(record);
                            dup_count = 0;
                            dup_since = Instant::now();
                        }

                        // Flush on count threshold or time interval
                        let elapsed = last_flush.elapsed();
                        if since_flush >= FLUSH_COUNT || elapsed >= FLUSH_INTERVAL {
                            // Also flush pending dedup record on time threshold
                            flush_prev(&mut writer, &mut prev_record, &mut dup_count, &mut since_flush)?;
                            prev_key = None;
                            writer.flush()?;
                            since_flush = 0;
                            last_flush = Instant::now();
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // Flush any pending dedup record on timeout
                        flush_prev(&mut writer, &mut prev_record, &mut dup_count, &mut since_flush)?;
                        prev_key = None;
                        if since_flush > 0 {
                            writer.flush()?;
                            since_flush = 0;
                            last_flush = Instant::now();
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // Flush pending dedup record and exit
                        flush_prev(&mut writer, &mut prev_record, &mut dup_count, &mut since_flush)?;
                        break;
                    }
                }
            }

            writer.flush()?;
            Ok(())
        });

        Ok(Self {
            started_at: Instant::now(),
            sender: Mutex::new(Some(tx)),
            handle: Mutex::new(Some(handle)),
            dropped_events: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
        })
    }

    /// Record a log entry, dropping it if the channel is full.
    pub fn record(&self, record: LogRecord) {
        let sender = self.sender.lock().as_ref().cloned();
        if let Some(sender) = sender {
            match sender.try_send(record) {
                Ok(()) => {
                    self.total_events.fetch_add(1, Ordering::SeqCst);
                }
                Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                    self.dropped_events.fetch_add(1, Ordering::SeqCst);
                }
            }
        } else {
            self.dropped_events.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Finish writing and flush all pending records.
    pub fn finish(&self) -> eyre::Result<()> {
        let sender = self.sender.lock().take();
        drop(sender);
        let handle = self.handle.lock().take();
        if let Some(handle) = handle {
            match handle.join() {
                Ok(res) => res?,
                Err(_) => return Err(eyre::eyre!("json log writer thread panicked")),
            }
        }
        Ok(())
    }

    /// Get the number of dropped events.
    #[expect(dead_code, reason = "API for diagnostics, used in tests")]
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::SeqCst)
    }

    /// Get the total number of events written.
    #[expect(dead_code, reason = "API for diagnostics, used in tests")]
    pub fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::SeqCst)
    }

    /// Get the elapsed time since the writer was created.
    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
}

/// Filter mode for JSON log layers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsonLogFilter {
    /// Accept all events.
    All,
    /// Accept only events where message == "resources".
    ResourcesOnly,
    /// Accept all events except where message == "resources".
    ExcludeResources,
}

/// A tracing layer that writes events as JSON to a file.
#[derive(Clone)]
pub struct JsonLogLayer {
    writer: std::sync::Arc<JsonLogWriter>,
    filter: JsonLogFilter,
}

impl JsonLogLayer {
    /// Create a new JSON log layer that accepts all events.
    #[expect(dead_code, reason = "convenience constructor for unfiltered logging")]
    pub const fn new(writer: std::sync::Arc<JsonLogWriter>) -> Self {
        Self {
            writer,
            filter: JsonLogFilter::All,
        }
    }

    /// Create a new JSON log layer with a specific filter.
    pub const fn with_filter(writer: std::sync::Arc<JsonLogWriter>, filter: JsonLogFilter) -> Self {
        Self { writer, filter }
    }
}

impl<S> Layer<S> for JsonLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let meta = event.metadata();
        let mut visitor = JsonLogVisitor::default();
        event.record(&mut visitor);
        let mut fields = visitor.fields;
        let message = match fields.remove("message") {
            Some(JsonValue::String(value)) => Some(value),
            Some(value) => Some(value.to_string()),
            None => None,
        };

        // Apply filter based on message content
        let is_resources = message.as_deref() == Some("resources");
        match self.filter {
            JsonLogFilter::ResourcesOnly if !is_resources => return,
            JsonLogFilter::ExcludeResources if is_resources => return,
            JsonLogFilter::All | JsonLogFilter::ResourcesOnly | JsonLogFilter::ExcludeResources => {
            }
        }

        let record = LogRecord {
            t_ms: self.writer.elapsed().as_millis() as u64,
            level: meta.level().as_str().to_string(),
            target: meta.target().to_string(),
            file: meta.file().map(ToString::to_string),
            line: meta.line(),
            message,
            fields,
            count: None,
        };
        self.writer.record(record);
    }
}

// ============================================================================
// TUI Log Capture
// ============================================================================

use std::collections::VecDeque;
use std::sync::Arc;

/// Maximum number of log entries to keep in the TUI buffer.
const TUI_LOG_BUFFER_SIZE: usize = 100;

/// A single log entry for TUI display.
#[derive(Clone, Debug)]
pub struct TuiLogEntry {
    pub level: tracing::Level,
    pub message: String,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
}

/// Shared buffer for TUI log capture.
#[derive(Debug, Default)]
pub struct TuiLogBuffer {
    entries: Mutex<VecDeque<TuiLogEntry>>,
}

impl TuiLogBuffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(TUI_LOG_BUFFER_SIZE)),
        }
    }

    /// Add a log entry, removing oldest if at capacity.
    pub fn push(&self, entry: TuiLogEntry) {
        let mut entries = self.entries.lock();
        if entries.len() >= TUI_LOG_BUFFER_SIZE {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Drain all entries from the buffer (returns and clears).
    pub fn drain(&self) -> Vec<TuiLogEntry> {
        let mut entries = self.entries.lock();
        entries.drain(..).collect()
    }
}

/// A tracing layer that captures logs to a shared TUI buffer.
#[derive(Clone)]
pub struct TuiLogLayer {
    buffer: Arc<TuiLogBuffer>,
    /// Minimum level to capture (based on verbosity).
    min_level: tracing::Level,
    /// Whether to display WARN-level events (requires `-v`).
    show_warn: bool,
}

impl TuiLogLayer {
    /// Create a new TUI log layer with the given buffer and minimum level.
    pub const fn new(buffer: Arc<TuiLogBuffer>, min_level: tracing::Level, show_warn: bool) -> Self {
        Self { buffer, min_level, show_warn }
    }
}

/// Format a structured field value for TUI display, truncating long hex strings and values.
fn format_tui_field_value(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => {
            // Truncate long hex strings (peer IDs, hashes): "0x1234…cdef"
            if s.starts_with("0x") && s.len() > 18 {
                format!("{}…{}", &s[..6], &s[s.len() - 4..])
            } else if s.len() > 40 {
                format!("{}…", &s[..37])
            } else {
                s.clone()
            }
        }
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        _ => value.to_string(),
    }
}

impl<S> Layer<S> for TuiLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let meta = event.metadata();
        let level = *meta.level();

        // Skip events below minimum level
        if level > self.min_level {
            return;
        }

        // Skip WARN unless verbosity >= 1 (-v)
        if level == tracing::Level::WARN && !self.show_warn {
            return;
        }

        let mut visitor = JsonLogVisitor::default();
        event.record(&mut visitor);

        // Build message from visitor fields
        let base_message = visitor
            .fields
            .get("message")
            .and_then(|v| v.as_str())
            .map_or_else(
                || {
                    // Fallback: use target and first field
                    let target = meta.target();
                    let short_target = target.rsplit("::").next().unwrap_or(target);
                    if let Some((key, value)) = visitor.fields.iter().next() {
                        format!("{short_target}: {key}={value}")
                    } else {
                        short_target.to_string()
                    }
                },
                ToString::to_string,
            );

        // Collect non-message structured fields as "key=value" pairs
        let mut extras = String::new();
        for (key, value) in &visitor.fields {
            if key == "message" {
                continue;
            }
            if !extras.is_empty() {
                extras.push_str(", ");
            }
            extras.push_str(key);
            extras.push('=');
            extras.push_str(&format_tui_field_value(value));
        }

        let message = if extras.is_empty() {
            base_message
        } else {
            format!("{base_message} \u{2502} {extras}")
        };

        // Use system time for absolute timestamps
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.buffer.push(TuiLogEntry { level, message, timestamp_ms });
    }
}
