//! The panic hook (roadmap item 2.4; roadmap item 1.7, question 4): a
//! panic's message goes through the log writer, so that it reaches
//! `daemon.log`, with a bounded wait.
//!
//! Rust's default hook writes the message to stderr from the panicking
//! thread with no bound: a console in a held QuickEdit selection would stop
//! it there, and the line never reaches the file. This hook builds one line,
//! `[ERROR] thread '<name>' panicked at <file>:<line>:<column>: <message>`,
//! queues it for the writer and waits until it is written, and gives up at
//! [`REPORT_BUDGET`]. It never chains to the default hook, which would write
//! the message a second time, without a bound.
//!
//! # The bound
//!
//! A panic holds its thread in the hook for at most 250 ms, plus the time to
//! build its line. A second panic on the same thread within 1 s of the end of
//! that budget shares it. A panic inside the keyboard callback, an `extern
//! "system"` fn, then aborts the process ("As of 1.81, the non-unwind ABIs
//! (e.g., "C") will now abort on uncaught unwinds", Rust 1.81.0), so the
//! keyboard waits at most about 250 ms for the dying daemon, under the
//! largest `LowLevelHooksTimeout` of 1000 ms. The writer writes the file
//! before the console, so the line is in `daemon.log` even if the console
//! blocks.
//!
//! On the `log-writer` thread itself, or with no writer, the hook writes
//! the line to stderr and does not wait: there is nobody to wait for.

use std::any::Any;
use std::cell::Cell;
use std::fmt::Write as _;
use std::panic::PanicHookInfo;
use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant, SystemTime};

use super::sink::LogLine;
use super::writer::{self, WriterMsg};

/// How long a panic waits for its line to be written.
pub(crate) const REPORT_BUDGET: Duration = Duration::from_millis(250);

/// How long after a budget's end a panic of the same thread shares it.
const EPISODE: Duration = Duration::from_secs(1);

/// The target of the line, which reads as the crate's own.
const TARGET: &str = "key_switch_rs::panic";

/// The name of the writer's thread, which must not wait for itself.
const WRITER_THREAD: &str = "log-writer";

thread_local! {
    /// The end of this thread's last panic budget.
    static BUDGET_END: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Installs the panic hook for the whole process, on the daemon's writer.
/// `logging::init` calls it once it has installed the logger.
pub(crate) fn install() {
    std::panic::set_hook(Box::new(|info| report(info, super::LOGGER.writer.get())));
}

/// Installs the panic hook on the writer that `writer` names, for a test's
/// child process, which must not start the daemon's.
#[cfg(test)]
pub(crate) fn install_with(writer: fn() -> Option<&'static SyncSender<WriterMsg>>) {
    std::panic::set_hook(Box::new(move |info| report(info, writer())));
}

/// The hook: builds the line and hands it to `writer` with this thread's
/// budget, or writes it to stderr where waiting would wait for nothing.
fn report(info: &PanicHookInfo<'_>, writer: Option<&SyncSender<WriterMsg>>) {
    let thread = std::thread::current();
    let name = thread.name();
    let location = info
        .location()
        .map(|location| (location.file(), location.line(), location.column()));
    let text = panic_text(name, location, info.payload());
    let line = LogLine::new(
        SystemTime::now(),
        log::Level::Error,
        TARGET,
        &format_args!("{text}"),
    );
    match writer {
        Some(tx) if name != Some(WRITER_THREAD) => {
            let now = Instant::now();
            let deadline = BUDGET_END
                .try_with(|end| {
                    let deadline = budget_end(now, end.get());
                    end.set(Some(deadline));
                    deadline
                })
                .unwrap_or(now + REPORT_BUDGET);
            let _ = report_on(tx, line, deadline);
        }
        _ => super::write_to_stderr(&line),
    }
}

/// The panic's line: `thread '<name>' panicked at <file>:<line>:<column>:
/// <message>`, as Rust's default hook names a thread without a name
/// (`<unnamed>`); `<unknown>` for no location; the message if the payload
/// is a `&str` or a `String`, else `Box<dyn Any>`. Built with `write!` into
/// a `String`, which cannot panic.
pub(crate) fn panic_text(
    thread: Option<&str>,
    location: Option<(&str, u32, u32)>,
    payload: &(dyn Any + Send),
) -> String {
    let mut text = String::new();
    let _ = write!(
        text,
        "thread '{}' panicked at ",
        thread.unwrap_or("<unnamed>")
    );
    match location {
        Some((file, line, column)) => {
            let _ = write!(text, "{file}:{line}:{column}");
        }
        None => text.push_str("<unknown>"),
    }
    text.push_str(": ");
    text.push_str(message(payload));
    text
}

fn message(payload: &(dyn Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<&str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else {
        "Box<dyn Any>"
    }
}

/// The deadline of a panic at `now` on a thread whose last budget ended at
/// `previous`: that end, if `now` is within [`EPISODE`] of it, else `now`
/// plus [`REPORT_BUDGET`].
pub(crate) fn budget_end(now: Instant, previous: Option<Instant>) -> Instant {
    match previous {
        Some(end) if now < end + EPISODE => end,
        _ => now + REPORT_BUDGET,
    }
}

/// Queues `line` for the writer behind `tx` and waits until it is written,
/// both until `deadline`; true if it was written in time.
pub(crate) fn report_on(tx: &SyncSender<WriterMsg>, line: LogLine, deadline: Instant) -> bool {
    writer::send_by(tx, WriterMsg::Line(line), deadline).is_ok() && writer::flush_by(tx, deadline)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::{Command, Stdio};
    use std::sync::OnceLock;
    use std::sync::mpsc::{SyncSender, sync_channel};
    use std::time::{Duration, Instant, SystemTime};

    use super::*;
    use crate::config::testing::TempDir;
    use crate::logging::counters::Counters;
    use crate::logging::sink::{LogFiles, LogLine};
    use crate::logging::testing::Console;
    use crate::logging::writer::{self, WriterMsg};

    /// A report period no test reaches.
    const NEVER: Duration = Duration::from_secs(3_600);

    /// Counters of their own, so that no writer of these tests takes the
    /// daemon's (ruling M4).
    fn leaked_counters() -> &'static Counters {
        Box::leak(Box::new(Counters::new()))
    }

    fn line(text: &str) -> LogLine {
        LogLine::new(
            SystemTime::now(),
            log::Level::Error,
            "key_switch_rs::panic",
            &format_args!("{text}"),
        )
    }

    #[test]
    fn a_panic_line_names_the_thread_the_place_and_the_message() {
        let place = Some(("src\\main.rs", 3, 7));
        assert_eq!(
            panic_text(Some("main"), place, &"boom"),
            "thread 'main' panicked at src\\main.rs:3:7: boom"
        );
        assert_eq!(
            panic_text(Some("ipc-server"), place, &String::from("no pipe")),
            "thread 'ipc-server' panicked at src\\main.rs:3:7: no pipe"
        );
        assert_eq!(
            panic_text(None, None, &42_u32),
            "thread '<unnamed>' panicked at <unknown>: Box<dyn Any>"
        );
    }

    #[test]
    fn a_nested_panic_on_one_thread_shares_the_budget() {
        // The second panic of a thread within a second of the first one's
        // budget, such as the abort of a panic that cannot unwind out of an
        // extern "system" callback, waits no longer than the first allows.
        let now = Instant::now();
        let first = budget_end(now, None);
        assert_eq!(first, now + REPORT_BUDGET);
        assert_eq!(
            budget_end(now + Duration::from_millis(100), Some(first)),
            first
        );
        assert_eq!(
            budget_end(first + Duration::from_millis(999), Some(first)),
            first,
            "within a second of the budget's end"
        );
        let later = first + Duration::from_secs(1);
        assert_eq!(
            budget_end(later, Some(first)),
            later + REPORT_BUDGET,
            "a later panic has a budget of its own"
        );
    }

    #[test]
    fn a_stuck_writer_holds_a_panic_report_at_most_its_budget() {
        // A queue that is full and never read: a writer stuck at a console
        // write. The report gives up at its deadline.
        let (tx, _unread) = sync_channel(1);
        tx.try_send(WriterMsg::Line(line("before")))
            .expect("the queue takes one line");
        let begun = Instant::now();
        assert!(!report_on(
            &tx,
            line("thread 'x' panicked"),
            begun + REPORT_BUDGET
        ));
        let took = begun.elapsed();
        assert!(
            took <= REPORT_BUDGET + Duration::from_millis(100),
            "the report waited {took:?}"
        );
    }

    #[test]
    fn a_panic_report_reaches_the_log_file_before_it_returns() {
        let temp = TempDir::new("panic-file");
        let console = Console::default();
        let tx = writer::spawn(Box::new(console.clone()), leaked_counters(), NEVER)
            .expect("the writer starts");
        writer::attach_on(
            &tx,
            LogFiles::new(temp.path().to_path_buf()),
            Duration::from_secs(5),
        )
        .expect("the writer takes the files");

        let line = line("thread 'x' panicked at src\\x.rs:1:1: boom");
        assert!(report_on(&tx, line, Instant::now() + REPORT_BUDGET));

        let file = fs::read_to_string(temp.path().join("daemon.log")).expect("daemon.log");
        assert!(file.contains("[ERROR] thread 'x' panicked at "), "{file:?}");
        assert!(
            console.text().starts_with("[ERROR] thread 'x' panicked"),
            "{:?}",
            console.text()
        );
    }

    /// The child's writer, for its panic hook.
    static CHILD_WRITER: OnceLock<SyncSender<WriterMsg>> = OnceLock::new();

    /// The environment variable that makes `panic_child` panic, naming the
    /// folder of its log file.
    const CHILD: &str = "KS_PANIC_CHILD";

    #[test]
    fn panic_child() {
        // Run by the next test in a process of its own: it installs the
        // panic hook on a writer of its own and panics on a spawned thread,
        // an unwinding panic, so nothing aborts. In the suite it does
        // nothing.
        let Some(dir) = std::env::var_os(CHILD) else {
            return;
        };
        let tx = writer::spawn(Box::new(std::io::sink()), leaked_counters(), NEVER)
            .expect("the writer starts");
        writer::attach_on(
            &tx,
            LogFiles::new(std::path::PathBuf::from(dir)),
            Duration::from_secs(5),
        )
        .expect("the writer takes the files");
        let _ = CHILD_WRITER.set(tx);
        install_with(|| CHILD_WRITER.get());
        let panicked = std::thread::spawn(|| panic!("KS panic child")).join();
        assert!(panicked.is_err());
    }

    #[test]
    fn a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook() {
        let temp = TempDir::new("panic-child");
        let mut child = Command::new(std::env::current_exe().expect("the test binary"))
            .args([
                "--exact",
                "logging::panic::tests::panic_child",
                "--test-threads=1",
            ])
            .env(CHILD, temp.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the test binary starts");
        let begun = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().expect("the child can be waited for") {
                break status;
            }
            if begun.elapsed() > Duration::from_secs(10) {
                let _ = child.kill();
                panic!("the child did not end within 10 s");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success(), "the child's test failed: {status}");

        let file = fs::read_to_string(temp.path().join("daemon.log")).expect("daemon.log");
        let lines: Vec<&str> = file
            .lines()
            .filter(|line| line.contains("[ERROR] thread '<unnamed>' panicked at "))
            .collect();
        assert_eq!(lines.len(), 1, "{file:?}");
        assert!(lines[0].contains("panic.rs:"), "{file:?}");
        assert!(lines[0].contains("KS panic child"), "{file:?}");
    }
}
