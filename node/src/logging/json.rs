//! JSON log writer and tracing layer for structured logging.

use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use parking_lot::Mutex;
use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    thread::JoinHandle,
    time::Instant,
};
use tracing::Event;
use tracing_subscriber::Layer;

/// Buffer size for JSON log channel.
pub const LOG_BUFFER: usize = 10_000;

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
    pub fn new(path: PathBuf, capacity: usize) -> eyre::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let (tx, rx) = mpsc::sync_channel::<LogRecord>(capacity);
        let handle = std::thread::spawn(move || -> eyre::Result<()> {
            let mut since_flush = 0usize;
            for record in rx {
                serde_json::to_writer(&mut writer, &record)?;
                writer.write_all(b"\n")?;
                since_flush = since_flush.saturating_add(1);
                if since_flush >= 4096 {
                    writer.flush()?;
                    since_flush = 0;
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
    pub fn new(writer: std::sync::Arc<JsonLogWriter>) -> Self {
        Self {
            writer,
            filter: JsonLogFilter::All,
        }
    }

    /// Create a new JSON log layer with a specific filter.
    pub fn with_filter(writer: std::sync::Arc<JsonLogWriter>, filter: JsonLogFilter) -> Self {
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
        };
        self.writer.record(record);
    }
}
