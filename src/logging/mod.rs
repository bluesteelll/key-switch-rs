//! Diagnostics through the `log` facade, with an in-house sink (roadmap
//! item 1.7, owner decision).
//!
//! [`init`], the first statement of the daemon's `main`, installs the
//! logger and starts the `log-writer` thread, which writes every record to
//! stderr as `[LEVEL] message`. Once the daemon owns its session,
//! [`attach_file`] adds `%LOCALAPPDATA%\key-switch-rs\daemon.log`, where
//! every line reads `2026-10-16T09:41:07.123Z [LEVEL] message` (UTC). At
//! 1 MiB the file rotates into `daemon.1.log` and `daemon.2.log`.
//!
//! A thread that logs only builds its line and queues it without waiting
//! (see `writer`), so logging is safe on the hook thread, where action
//! failures are still reported until roadmap item 2.1. The hook callback
//! itself never logs: its per-event conditions go to [`counters`].
//!
//! A program that never calls [`init`] loses every record, because `log`
//! drops them until a logger is installed: a tool that loads a config to
//! check it calls [`init`] first and [`flush`] before it exits.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::mpsc::SyncSender;
use std::time::{Duration, SystemTime};

pub mod counters;
mod sink;
#[cfg(test)]
pub(crate) mod testing;
mod writer;

use sink::{LogFiles, LogLine};

/// The folder under `%LOCALAPPDATA%` that holds the log files.
pub const LOG_DIR_NAME: &str = "key-switch-rs";
/// The file lines are written to; rotation moves it to `daemon.1.log`.
pub const LOG_FILE_NAME: &str = "daemon.log";
/// The target of an error that ends the daemon: such a record is tagged
/// `[FATAL]`, a level `log` does not have.
pub const FATAL_TARGET: &str = "key_switch_rs::fatal";
/// Records above this level (debug, trace) are not logged.
pub const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Info;

/// How long [`flush`] and [`attach_file`] wait for the writer.
const FLUSH_TIMEOUT: Duration = Duration::from_secs(1);
/// How often the writer reports non-zero [`counters`].
const REPORT_PERIOD: Duration = Duration::from_secs(60);

struct DaemonLogger {
    /// The writer's queue, set once by [`init`]. Empty when the writer
    /// could not be started: records then go to stderr directly.
    writer: OnceLock<SyncSender<writer::WriterMsg>>,
}

static LOGGER: DaemonLogger = DaemonLogger {
    writer: OnceLock::new(),
};

impl log::Log for DaemonLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= MAX_LEVEL
    }

    /// May run on the hook thread: one `String`, a lock-free `OnceLock`
    /// read and a `try_send`, and nothing that can panic.
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = LogLine::new(
            SystemTime::now(),
            record.level(),
            record.target(),
            record.args(),
        );
        #[cfg(test)]
        testing::record(&line);
        let unsent = match self.writer.get() {
            Some(tx) => writer::send_line(tx, line, &counters::COUNTERS).err(),
            None => Some(line),
        };
        if let Some(line) = unsent {
            write_to_stderr(&line);
        }
    }

    fn flush(&self) {
        let _ = flush();
    }
}

/// The fallback with no writer: the console line, written by the calling
/// thread.
#[cfg(not(test))]
fn write_to_stderr(line: &LogLine) {
    use std::io::Write as _;
    let _ = io::stderr().write_all(line.console_text().as_bytes());
}

/// The test suite prints nothing.
#[cfg(test)]
fn write_to_stderr(_line: &LogLine) {}

/// Installs the logger, at level [`MAX_LEVEL`], and starts the `log-writer`
/// thread with the console (stderr) as its only output. Once per process; a
/// later call, or a call after another logger was installed, does nothing.
/// If the thread cannot be started, records go to stderr directly.
pub fn init() {
    let spawned = writer::spawn(Box::new(io::stderr()), &counters::COUNTERS, REPORT_PERIOD);
    if log::set_logger(&LOGGER).is_err() {
        return;
    }
    log::set_max_level(MAX_LEVEL);
    match spawned {
        Ok(tx) => {
            let _ = LOGGER.writer.set(tx);
        }
        Err(error) => {
            log::warn!("could not start the log writer, logging to the console only: {error}")
        }
    }
}

/// From now on, the writer also appends every line to `daemon.log` in
/// `dir`, which it creates on demand, and returns the file's path. A file
/// that cannot be written is reported once on the console, and every later
/// line tries again. Fails only if there is no writer to hand the file to.
pub fn attach_file(dir: &Path) -> io::Result<PathBuf> {
    let files = LogFiles::new(dir.to_path_buf());
    let path = files.path();
    let tx = LOGGER
        .writer
        .get()
        .ok_or_else(|| io::Error::other(writer::SendFailure::Gone.to_string()))?;
    writer::attach_on(tx, files, FLUSH_TIMEOUT)
        .map_err(|failure| io::Error::other(failure.to_string()))?;
    Ok(path)
}

/// Waits at most 1 s until every record logged before the call is written
/// and the counters are reported. `true` if that happened, or if there is
/// no writer, so nothing waits. Never called on the hook callback path.
pub fn flush() -> bool {
    match LOGGER.writer.get() {
        Some(tx) => writer::flush_on(tx, FLUSH_TIMEOUT),
        None => true,
    }
}

/// The folder of the log files for the account whose `%LOCALAPPDATA%` is
/// `local_app_data`.
pub fn log_dir(local_app_data: &Path) -> PathBuf {
    local_app_data.join(LOG_DIR_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::testing::{capture, is_timestamped};

    #[test]
    fn log_macros_reach_the_logger_with_a_timestamp() {
        let ((), lines) = capture(|| log::info!("config reloaded ({} bindings live)", 2));

        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(is_timestamped(&lines[0]), "{lines:?}");
        assert!(
            lines[0].ends_with(" [INFO] config reloaded (2 bindings live)\n"),
            "{lines:?}"
        );
    }

    #[test]
    fn debug_and_trace_records_are_not_logged() {
        let ((), lines) = capture(|| {
            log::debug!("debug");
            log::trace!("trace");
            log::warn!("warn");
        });

        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].ends_with(" [WARN] warn\n"), "{lines:?}");
    }

    #[test]
    fn a_fatal_record_is_tagged_fatal() {
        let ((), lines) =
            capture(|| log::error!(target: FATAL_TARGET, "config file x does not exist"));

        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0].ends_with(" [FATAL] config file x does not exist\n"),
            "{lines:?}"
        );
    }

    #[test]
    fn the_log_folder_is_key_switch_rs_under_local_app_data() {
        assert_eq!(
            log_dir(Path::new(r"C:\Users\u\AppData\Local")),
            Path::new(r"C:\Users\u\AppData\Local\key-switch-rs")
        );
        assert_eq!(LOG_FILE_NAME, "daemon.log");
    }

    #[test]
    fn flush_without_a_writer_returns_true_at_once() {
        let begun = std::time::Instant::now();
        assert!(flush());
        assert!(
            begun.elapsed() < FLUSH_TIMEOUT,
            "flush waited for no writer"
        );
    }

    #[test]
    fn a_file_cannot_be_attached_without_a_writer() {
        let temp = crate::config::testing::TempDir::new("attach-no-writer");
        let dir = temp.path().join("key-switch-rs");

        assert!(attach_file(&dir).is_err());
        assert!(!dir.exists(), "nothing is created without a writer");
    }
}
