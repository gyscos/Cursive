//! Logging utilities

use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Saves all log records in a global deque.
///
/// Uses a `DebugView` to access it.
#[cfg(not(feature = "configurable-logger"))]
pub struct CursiveLogger;

/// Saves all log records in a global deque.
///
/// Uses a `DebugView` to access it.
#[cfg(feature = "configurable-logger")]
pub struct CursiveLogger {
    level: log::Level,
}

#[cfg(not(feature = "configurable-logger"))]
impl Default for CursiveLogger {
    fn default() -> CursiveLogger {
        CursiveLogger
    }
}

#[cfg(feature = "configurable-logger")]
impl Default for CursiveLogger {
    fn default() -> CursiveLogger {
        CursiveLogger {
            level: log::Level::Debug,
        }
    }
}

#[cfg(not(feature = "configurable-logger"))]
static LOGGER: CursiveLogger = CursiveLogger;

#[cfg(feature = "configurable-logger")]
static LOGGER: CursiveLogger = CursiveLogger {
    level: log::Level::Debug,
};

/// A log record.
pub struct Record {
    /// Log level used for this record
    pub level: log::Level,
    /// Time this message was logged
    pub time: chrono::DateTime<chrono::Utc>,
    /// Message content
    pub message: String,
}

lazy_static! {
    /// Circular buffer for logs. Use it to implement [`DebugView`].
    ///
    /// [`DebugView`]: ../views/struct.DebugView.html
    pub static ref LOGS: Mutex<VecDeque<Record>> =
        Mutex::new(VecDeque::new());
}

/// Log a record in cursive's log queue.
pub fn log(record: &log::Record<'_>) {
    let mut logs = LOGS.lock().unwrap();
    // TODO: customize the format? Use colors? Save more info?
    if logs.len() == logs.capacity() {
        logs.pop_front();
    }
    logs.push_back(Record {
        level: record.level(),
        message: format!("{}", record.args()),
        time: chrono::Utc::now(),
    });
}

#[cfg(not(feature = "configurable-logger"))]
impl log::Log for CursiveLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        log(record);
    }

    fn flush(&self) {}
}

#[cfg(feature = "configurable-logger")]
impl log::Log for CursiveLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record<'_>) {
        log(record);
    }

    fn flush(&self) {}
}

/// Initialize the Cursive logger.
///
/// Make sure this is the only logger your are using.
///
/// Use a [`DebugView`](crate::views::DebugView) to see the logs, or use
/// [`Cursive::toggle_debug_console()`](crate::Cursive::toggle_debug_console()).
pub fn init() {
    // TODO: Configure the deque size?
    reserve_logs(1_000);

    // This will panic if `set_logger` was already called.
    log::set_logger(&LOGGER).unwrap();

    // TODO: read the level from env variable? From argument?
    log::set_max_level(log::LevelFilter::Trace);
}

/// Builder helper type for building a logger
///
/// Use Builder::default() to build a default-configured Builder object, which
///
/// * Logs with "debug" level
#[cfg(feature = "configurable-logger")]
pub struct Builder {
    level: log::Level,
}

#[cfg(feature = "configurable-logger")]
impl Default for Builder {
    fn default() -> Builder {
        Builder {
            level: log::Level::Debug,
        }
    }
}

#[cfg(feature = "configurable-logger")]
impl Builder {
    /// initialize the logger
    pub fn init(self) {
        unimplemented!()
    }

    /// Set the level to log with
    pub fn level(mut self, level: log::Level) -> Self {
        self.level = level;
        self
    }
}

/// Return a logger that stores records in cursive's log queue.
///
/// These logs can then be read by a [`DebugView`](crate::views::DebugView).
///
/// An easier alternative might be to use [`init()`].
pub fn get_logger() -> CursiveLogger {
    reserve_logs(1_000);
    CursiveLogger::default()
}

/// Adds `n` more entries to cursive's log queue.
///
/// Most of the time you don't need to use this directly.
///
/// You should call this if you're not using `init()` nor `get_logger()`.
pub fn reserve_logs(n: usize) {
    LOGS.lock().unwrap().reserve(n);
}
