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
//! (see `writer`). The hook path never logs: the hook callback and the
//! fast actions it runs count their per-event conditions in [`counters`].
//! The failures of `Launch`, `OpenUrl` and macro steps are logged on the
//! `action-executor` thread (roadmap item 2.1).
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
pub(crate) mod panic;
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
/// thread with the console (stderr) as its only output, and installs the
/// panic hook, which logs a panic through that writer (`panic`). Once per
/// process; a later call, or a call after another logger was installed,
/// does nothing. If the thread cannot be started, records go to stderr
/// directly.
pub fn init() {
    let Some(spawned) = start_logging(
        || log::set_logger(&LOGGER).is_ok(),
        || writer::spawn(Box::new(io::stderr()), &counters::COUNTERS, REPORT_PERIOD),
    ) else {
        return;
    };
    log::set_max_level(MAX_LEVEL);
    // A panic's message goes through the writer from now on, with a
    // bounded wait (roadmap item 2.4).
    panic::install();
    match spawned {
        Ok(tx) => {
            let _ = LOGGER.writer.set(tx);
        }
        Err(error) => {
            log::warn!("could not start the log writer, logging to the console only: {error}")
        }
    }
}

/// The order of [`init`]: installs the logger with `install`, and only if
/// that worked starts the writer with `start_writer` and returns what it
/// returned. `None` means the writer was never started: a second `init`,
/// or one after another logger was installed, starts no thread that would
/// report the counters once its queue is dropped.
fn start_logging<W>(
    install: impl FnOnce() -> bool,
    start_writer: impl FnOnce() -> io::Result<W>,
) -> Option<io::Result<W>> {
    if !install() {
        return None;
    }
    Some(start_writer())
}

/// From now on, the writer also appends every line to `daemon.log` in
/// `dir`, which it creates on demand, and returns the file's path. A file
/// that cannot be written is reported once on the console, and every later
/// line tries again. Fails if there is no writer to hand the file to, or
/// if the writer's queue stays full for 1 s.
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

/// `at` in UTC, as the log writes it: `2026-10-16T09:41:07.123Z`.
pub(crate) fn utc_timestamp(at: SystemTime) -> String {
    let mut text = String::new();
    sink::write_timestamp(&mut text, at);
    text
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
    fn a_second_init_starts_no_writer() {
        // The 1.7 review (m1): a second `init` finds the logger installed,
        // and a writer it started first would only run until its queue is
        // dropped. No test calls `init` itself.
        let started = std::cell::Cell::new(false);
        assert!(
            start_logging(
                || false,
                || {
                    started.set(true);
                    Ok(())
                }
            )
            .is_none()
        );
        assert!(!started.get(), "a second init must not start a writer");

        assert!(matches!(start_logging(|| true, || Ok(7)), Some(Ok(7))));
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

    /// Every `.rs` file under `dir`, recursively, except under `skip`.
    fn rust_files(dir: &Path, skip: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("list a source folder") {
            let path = entry.expect("a folder entry").path();
            if path == skip {
                continue;
            }
            if path.is_dir() {
                rust_files(&path, skip, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    /// The index of the line that opens the tests module of a source file
    /// (`#[cfg(test)]` followed by `mod tests {`), or the line count.
    fn tests_start(lines: &[&str]) -> usize {
        lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len())
    }

    /// The numbers (from 1) of the lines of the first function whose line
    /// contains `signature` before the tests module of `text`, from that
    /// line until its braces balance. Panics if there is none, so that a
    /// renamed function fails a scan loudly instead of passing it.
    fn fn_lines(text: &str, signature: &str) -> std::ops::RangeInclusive<usize> {
        let lines: Vec<&str> = text.lines().collect();
        let mut first = None;
        let mut depth = 0_i64;
        for (index, line) in lines[..tests_start(&lines)].iter().enumerate() {
            if first.is_none() && line.contains(signature) {
                first = Some(index + 1);
            }
            let Some(start) = first else {
                continue;
            };
            depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
            if depth == 0 && line.contains('}') {
                return start..=index + 1;
            }
        }
        panic!("no complete function `{signature}` before the tests module");
    }

    /// Whether `line` names the `log` crate's path, as a macro call or an
    /// import does, and not only a path that ends in `log`.
    fn names_the_log_crate(line: &str) -> bool {
        let path = concat!("log", "::");
        line.match_indices(path).any(|(at, _)| {
            !line[..at]
                .chars()
                .next_back()
                .is_some_and(|before| before.is_alphanumeric() || before == '_')
        })
    }

    /// The acceptance line of roadmap item 1.7: the only print macros
    /// under `src`, `src/bin` aside, are the startup banner's. And the hook
    /// callback module logs nothing outside its tests: its per-event
    /// conditions go to the counters. The macro names are assembled, so
    /// this file does not match itself.
    #[test]
    fn only_the_startup_banner_prints_outside_src_bin() {
        let print = concat!("print", "ln!");
        let eprint = concat!("eprint", "ln!");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let src = root.join("src");
        let mut files = Vec::new();
        rust_files(&src, &src.join("bin"), &mut files);
        files.sort();
        assert!(files.len() > 10, "the scan found the sources: {files:?}");

        let mut prints = Vec::new();
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            let banner =
                (relative == "src/core/app.rs").then(|| fn_lines(&text, "fn print_welcome("));
            for (index, line) in text.lines().enumerate() {
                let number = index + 1;
                let allowed = banner.as_ref().is_some_and(|lines| lines.contains(&number))
                    && !line.contains(eprint);
                if line.contains(print) && !allowed {
                    prints.push(format!("{relative}:{number}: {}", line.trim()));
                }
            }
        }

        let callback = "src/hook/keyboard_hook_callback.rs";
        let text = std::fs::read_to_string(root.join(callback)).expect("read the hook callback");
        let lines: Vec<&str> = text.lines().collect();
        let tests_at = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .expect("the hook callback has a tests module");
        let hook_logs: Vec<String> = lines[..tests_at]
            .iter()
            .enumerate()
            .filter(|(_, line)| names_the_log_crate(line))
            .map(|(index, line)| format!("{callback}:{}: {}", index + 1, line.trim()))
            .collect();

        let mut report = String::new();
        if !prints.is_empty() {
            report += &format!("{} print sites outside the startup banner:\n", prints.len());
            for site in &prints {
                report += &format!("  {site}\n");
            }
        }
        if !hook_logs.is_empty() {
            report += &format!(
                "{} log calls in {callback} outside its tests:\n",
                hook_logs.len()
            );
            for site in &hook_logs {
                report += &format!("  {site}\n");
            }
        }
        assert!(report.is_empty(), "{report}");
    }

    /// Roadmap item 2.1 ("the hook path never calls `log::`"): the
    /// functions that run in the hook callback (or in the main loop's timer
    /// drain, which carries out the same way) outside
    /// `keyboard_hook_callback.rs` name no `log::`. That file, with
    /// `carry_out` and `dispatch`, is scanned whole by
    /// `only_the_startup_banner_prints_outside_src_bin`. `None` scans the
    /// whole non-test part of the file.
    #[test]
    fn the_functions_on_the_hook_path_log_nothing() {
        let hook_path: &[(&str, Option<&str>)] = &[
            // Picks where every action fired on the hook thread runs.
            ("src/core/windows_actions.rs", Some("fn as_fast(")),
            // `FastAction::run`: runs the fast actions where they fire.
            ("src/core/windows_actions.rs", Some("fn run(")),
            // What `run` calls for `PressKey` and `Media`, and for
            // `PostMessage`.
            ("src/core/windows_actions.rs", Some("fn press_vk(")),
            (
                "src/core/windows_actions.rs",
                Some("fn post_message_to_foreground("),
            ),
            // What `run` reaches for `SwitchLanguage`,
            // `SwitchLanguageBackward` and `ToggleCapsLock`.
            ("src/system/system_function.rs", Some("fn execute(")),
            (
                "src/system/system_function.rs",
                Some("fn switch_language_forward("),
            ),
            (
                "src/system/system_function.rs",
                Some("fn switch_language_backward("),
            ),
            (
                "src/system/system_function.rs",
                Some("fn toggle_caps_lock("),
            ),
            ("src/system/system_function.rs", Some("fn press_key(")),
            // Every slow fire: the target, the queue and the wake.
            ("src/core/executor/mod.rs", Some("fn for_action(")),
            ("src/core/executor/mod.rs", Some("fn foreground(")),
            ("src/core/executor/mod.rs", Some("fn submit(")),
            ("src/core/executor/mod.rs", Some("fn wake(")),
            // The cancel that `MainState::set_enabled` and
            // `update_bindings` make.
            ("src/core/executor/mod.rs", Some("fn cancel_pending(")),
            // The hook's entry to the executor, and its cancel.
            ("src/hook/keyboard_hook.rs", Some("fn submit(")),
            ("src/hook/keyboard_hook.rs", Some("fn cancel_actions(")),
            // The callback's way to the engine and the gesture timer it
            // sets (roadmap item 2.2): `with_main`, then the scheduler's
            // `start` and `retain` (`GestureTimers`; the file's startup
            // function is `MainThread::new`, so its one `fn start(` is that).
            ("src/hook/main_thread.rs", Some("fn with_main<")),
            ("src/hook/main_thread.rs", Some("fn start(")),
            ("src/hook/main_thread.rs", Some("fn retain(")),
            // What a `when:` reads in the callback (roadmap item 2.3): the
            // cached executable name and the live title.
            ("src/platform/foreground.rs", Some("fn app(")),
            ("src/platform/foreground.rs", Some("fn read_title(")),
            ("src/platform/foreground.rs", Some("fn window_title(")),
            // What the desktop-switch callback asks (roadmap item 2.3).
            ("src/platform/desktop.rs", None),
            // `bump` and `record_callback` run on the hook thread for every
            // event that needs them.
            ("src/logging/counters.rs", None),
            // The heartbeat of every keyboard callback (roadmap item 2.4).
            ("src/hook/health.rs", Some("fn callback_seen(")),
        ];
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut sites = Vec::new();
        for (file, signature) in hook_path {
            let text = std::fs::read_to_string(root.join(file)).expect("read a source file");
            let lines: Vec<&str> = text.lines().collect();
            let scanned = match signature {
                Some(signature) => fn_lines(&text, signature),
                None => 1..=tests_start(&lines),
            };
            for number in scanned {
                let line = lines[number - 1];
                if names_the_log_crate(line) {
                    sites.push(format!(
                        "{file}:{number}: {} in {}",
                        line.trim(),
                        signature.unwrap_or("the non-test part")
                    ));
                }
            }
        }
        assert!(
            sites.is_empty(),
            "{} log calls on the hook path:\n  {}",
            sites.len(),
            sites.join("\n  ")
        );
    }

    #[test]
    fn a_path_that_only_ends_in_log_does_not_count_as_the_log_crate() {
        let call = concat!("log", "::warn!(\"x\")");
        assert!(names_the_log_crate(call));
        assert!(names_the_log_crate(&format!("    {call}")));
        assert!(names_the_log_crate(concat!("use ", "log", "::Level;")));
        assert!(!names_the_log_crate("crate::logging::counters::bump(c)"));
        assert!(!names_the_log_crate(concat!("sys", "log", "::send()")));
        assert!(!names_the_log_crate(concat!("my_", "log", "::send()")));
    }
}
