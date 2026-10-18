//! Test-only helpers: a thread-local capture of the records this crate
//! logs, and a console to hand to a writer.
//!
//! No test calls `logging::init` or `logging::attach_file`: the suite must
//! never start the daemon's writer or touch `%LOCALAPPDATA%`.

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, Once, PoisonError};

use super::LOGGER;
use super::sink::{LogLine, TIMESTAMP_LEN};

thread_local! {
    /// The file form of every record logged on this thread inside
    /// [`capture`].
    static CAPTURED: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Runs `f` and returns what it returned, with the file form of every
/// record logged on this thread meanwhile, one string per record.
///
/// The first call installs the daemon's logger for the whole test process,
/// with no writer: records of other threads go nowhere.
pub(crate) fn capture<R>(f: impl FnOnce() -> R) -> (R, Vec<String>) {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(super::MAX_LEVEL);
    });
    assert!(
        std::ptr::addr_eq(log::logger(), &LOGGER),
        "another logger is installed in the test process"
    );

    /// Ends the capture also when `f` panics.
    struct Stop;
    impl Drop for Stop {
        fn drop(&mut self) {
            let _ = CAPTURED.try_with(|captured| captured.borrow_mut().take());
        }
    }

    CAPTURED.with(|captured| *captured.borrow_mut() = Some(Vec::new()));
    let stop = Stop;
    let result = f();
    let lines = CAPTURED
        .with(|captured| captured.borrow_mut().take())
        .unwrap_or_default();
    drop(stop);
    (result, lines)
}

/// Keeps `line` if a capture runs on this thread.
pub(super) fn record(line: &LogLine) {
    let _ = CAPTURED.try_with(|captured| {
        if let Some(lines) = captured.borrow_mut().as_mut() {
            lines.push(line.file_text());
        }
    });
}

/// Whether `line` starts with a timestamp, `YYYY-MM-DDTHH:MM:SS.mmmZ`, and
/// a space.
pub(crate) fn is_timestamped(line: &str) -> bool {
    let Some(stamp) = line.get(..=TIMESTAMP_LEN) else {
        return false;
    };
    stamp
        .bytes()
        .zip("dddd-dd-ddTdd:dd:dd.dddZ ".bytes())
        .all(|(byte, shape)| match shape {
            b'd' => byte.is_ascii_digit(),
            _ => byte == shape,
        })
}

/// A console that keeps what is written to it, shared with its clones.
#[derive(Clone, Default)]
pub(crate) struct Console(Arc<Mutex<Vec<u8>>>);

impl Console {
    /// Everything written so far.
    pub(crate) fn text(&self) -> String {
        let bytes = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timestamp_and_a_space_make_a_line_timestamped() {
        assert!(is_timestamped("2026-10-16T09:41:07.123Z [INFO] x"));
        assert!(!is_timestamped("[INFO] x"));
        assert!(!is_timestamped("2026-10-16T09:41:07.123Z[INFO] x"));
        assert!(!is_timestamped("2026-10-16 09:41:07.123Z [INFO] x"));
        assert!(!is_timestamped("2026-10-16T09:41:07.123Z"));
    }
}
