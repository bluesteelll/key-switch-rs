//! The `log-writer` thread: the one owner of the console output and of the
//! log files.
//!
//! Every thread that logs only builds its [`LogLine`] and queues it with
//! `try_send`, which never waits: no file or console I/O happens there, and
//! the only lock is the channel's own, held for O(1) list work when it wakes
//! the parked writer. A full queue drops the line and counts it. The writer
//! uses no Win32 UI API and needs no message loop.

use std::io::{self, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use super::counters::{self, Counter, Counters};
use super::sink::{LogFiles, LogLine};

/// How many messages wait for the writer before new lines are dropped.
pub(crate) const QUEUE_CAPACITY: usize = 1024;

/// How often a send is retried while the queue is full, for the messages
/// that must not be dropped.
const RETRY_EVERY: Duration = Duration::from_millis(1);

/// The writer's own records name the crate as their target.
const TARGET: &str = "key_switch_rs::logging";

pub(crate) enum WriterMsg {
    /// A line for the console and, once attached, the files.
    Line(LogLine),
    /// From now on, lines also go into these files.
    Attach(LogFiles),
    /// Report the counters, then acknowledge: every earlier message has
    /// been written.
    Flush(SyncSender<()>),
}

/// Starts the `log-writer` thread, which writes to `console` and reports
/// `counters` every `report_every`, and returns its queue. The thread ends
/// once every sender is gone.
pub(crate) fn spawn(
    console: Box<dyn Write + Send>,
    counters: &'static Counters,
    report_every: Duration,
) -> io::Result<SyncSender<WriterMsg>> {
    let (tx, rx) = sync_channel(QUEUE_CAPACITY);
    let writer = Writer {
        console,
        files: None,
        failing: false,
        counters,
    };
    thread::Builder::new()
        .name("log-writer".into())
        .spawn(move || writer.run(&rx, report_every))?;
    Ok(tx)
}

/// Queues `line` without waiting. A full queue drops it and counts it in
/// `counters`; a writer that is gone gives the line back, for the caller to
/// write to the console itself.
pub(crate) fn send_line(
    tx: &SyncSender<WriterMsg>,
    line: LogLine,
    counters: &Counters,
) -> Result<(), LogLine> {
    match tx.try_send(WriterMsg::Line(line)) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => {
            counters.bump(Counter::LogLinesDropped);
            Ok(())
        }
        Err(TrySendError::Disconnected(WriterMsg::Line(line))) => Err(line),
        // `try_send` gives back the message it was given.
        Err(TrySendError::Disconnected(_)) => Ok(()),
    }
}

/// Why a message that must arrive did not.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SendFailure {
    /// The queue stayed full until the deadline.
    TimedOut,
    /// The writer is gone.
    Gone,
}

impl std::fmt::Display for SendFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SendFailure::TimedOut => "the log writer's queue stayed full",
            SendFailure::Gone => "the log writer is not running",
        })
    }
}

/// Queues `msg`, retrying every millisecond while the queue is full, until
/// `deadline`.
fn send_by(
    tx: &SyncSender<WriterMsg>,
    mut msg: WriterMsg,
    deadline: Instant,
) -> Result<(), SendFailure> {
    loop {
        match tx.try_send(msg) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Disconnected(_)) => return Err(SendFailure::Gone),
            Err(TrySendError::Full(back)) => {
                if Instant::now() >= deadline {
                    return Err(SendFailure::TimedOut);
                }
                msg = back;
                thread::sleep(RETRY_EVERY);
            }
        }
    }
}

/// Hands `files` to the writer, waiting at most `timeout` for room in the
/// queue.
pub(crate) fn attach_on(
    tx: &SyncSender<WriterMsg>,
    files: LogFiles,
    timeout: Duration,
) -> Result<(), SendFailure> {
    send_by(tx, WriterMsg::Attach(files), Instant::now() + timeout)
}

/// Waits at most `timeout` until every message queued before this call is
/// written and the counters are reported. `true` if that happened in time,
/// or if there is no writer left to wait for.
pub(crate) fn flush_on(tx: &SyncSender<WriterMsg>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let (ack_tx, ack_rx) = sync_channel(1);
    match send_by(tx, WriterMsg::Flush(ack_tx), deadline) {
        Ok(()) => ack_rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .is_ok(),
        Err(SendFailure::Gone) => true,
        Err(SendFailure::TimedOut) => false,
    }
}

struct Writer {
    console: Box<dyn Write + Send>,
    files: Option<LogFiles>,
    /// Whether the last line for the files failed, so that a streak of
    /// failures is reported once.
    failing: bool,
    counters: &'static Counters,
}

impl Writer {
    fn run(mut self, rx: &Receiver<WriterMsg>, report_every: Duration) {
        let mut next_report = Instant::now() + report_every;
        loop {
            let now = Instant::now();
            if now >= next_report {
                self.report_counters();
                next_report = now + report_every;
            }
            match rx.recv_timeout(next_report.saturating_duration_since(now)) {
                Ok(WriterMsg::Line(line)) => self.write(&line),
                Ok(WriterMsg::Attach(files)) => {
                    self.files = Some(files);
                    self.failing = false;
                }
                Ok(WriterMsg::Flush(ack)) => {
                    self.report_counters();
                    let _ = self.console.flush();
                    let _ = ack.send(());
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    self.report_counters();
                    return;
                }
            }
        }
    }

    /// Writes `line` to the files first, then to the console: a console
    /// that blocks (a held selection in a conhost window) then holds back
    /// the next lines, never this one's file write. A notice about the
    /// files follows the line on the console.
    fn write(&mut self, line: &LogLine) {
        let notice = self.write_files(line);
        write_console(self.console.as_mut(), line);
        if let Some(notice) = notice {
            write_console(self.console.as_mut(), &notice);
        }
    }

    /// Writes `line` to the files, if attached, and returns the notice for
    /// the console when a streak of failures starts or ends. The first
    /// failure of a streak is reported on the console only, since the file
    /// is what fails; the end of a streak goes into the file too.
    fn write_files(&mut self, line: &LogLine) -> Option<LogLine> {
        let files = self.files.as_ref()?;
        match files.write_line(&line.file_text()) {
            Ok(()) if self.failing => {
                self.failing = false;
                let notice = writer_line(
                    log::Level::Info,
                    format_args!("log file {} works again", files.path().display()),
                );
                // A failure here shows with the next line.
                let _ = files.write_line(&notice.file_text());
                Some(notice)
            }
            Ok(()) => None,
            Err(error) if !self.failing => {
                self.failing = true;
                Some(writer_line(
                    log::Level::Warn,
                    format_args!("log file {}: {error}", files.path().display()),
                ))
            }
            Err(_) => None,
        }
    }

    /// One line with the counts since the last report, if any is non-zero.
    fn report_counters(&mut self) {
        if let Some(summary) = counters::summary(&self.counters.take()) {
            let report = writer_line(log::Level::Warn, format_args!("{summary}"));
            self.write(&report);
        }
    }
}

fn write_console(console: &mut dyn Write, line: &LogLine) {
    // With no console attached the write fails, and there is nobody to
    // tell.
    let _ = console.write_all(line.console_text().as_bytes());
}

/// A record of the writer itself, stamped now.
fn writer_line(level: log::Level, args: std::fmt::Arguments<'_>) -> LogLine {
    LogLine::new(SystemTime::now(), level, TARGET, &args)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::config::testing::TempDir;
    use crate::logging::testing::{Console, is_timestamped};

    const WAIT: Duration = Duration::from_secs(5);
    /// A report period no test reaches.
    const NEVER: Duration = Duration::from_secs(3_600);

    fn leaked_counters() -> &'static Counters {
        Box::leak(Box::new(Counters::new()))
    }

    fn info(message: &str) -> LogLine {
        LogLine::new(
            SystemTime::now(),
            log::Level::Info,
            "key_switch_rs",
            &format_args!("{message}"),
        )
    }

    fn start(
        console: &Console,
        counters: &'static Counters,
        every: Duration,
    ) -> SyncSender<WriterMsg> {
        spawn(Box::new(console.clone()), counters, every).expect("the writer starts")
    }

    fn queue(tx: &SyncSender<WriterMsg>, line: LogLine) {
        assert!(
            send_line(tx, line, leaked_counters()).is_ok(),
            "the writer takes the line"
        );
    }

    #[test]
    fn the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console() {
        let temp = TempDir::new("writer-file");
        let console = Console::default();
        let tx = start(&console, leaked_counters(), NEVER);
        attach_on(&tx, LogFiles::new(temp.path().to_path_buf()), WAIT).expect("attached");

        queue(&tx, info("config reloaded (2 bindings live)"));
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        assert_eq!(console.text(), "[INFO] config reloaded (2 bindings live)\n");
        let file = fs::read_to_string(temp.path().join("daemon.log")).expect("daemon.log");
        let lines: Vec<&str> = file.lines().collect();
        assert_eq!(lines.len(), 1, "{file:?}");
        assert!(is_timestamped(lines[0]), "{file:?}");
        assert!(
            lines[0].ends_with(" [INFO] config reloaded (2 bindings live)"),
            "{file:?}"
        );
    }

    #[test]
    fn lines_before_the_file_is_attached_reach_the_console_only() {
        let temp = TempDir::new("writer-before");
        let console = Console::default();
        let tx = start(&console, leaked_counters(), NEVER);

        queue(&tx, info("before"));
        attach_on(&tx, LogFiles::new(temp.path().to_path_buf()), WAIT).expect("attached");
        queue(&tx, info("after"));
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        assert_eq!(console.text(), "[INFO] before\n[INFO] after\n");
        let file = fs::read_to_string(temp.path().join("daemon.log")).expect("daemon.log");
        assert!(!file.contains("before"), "{file:?}");
        assert!(file.ends_with(" [INFO] after\n"), "{file:?}");
    }

    #[test]
    fn a_flush_returns_once_earlier_lines_are_written() {
        let console = Console::default();
        let tx = start(&console, leaked_counters(), NEVER);
        for n in 0..100 {
            queue(&tx, info(&format!("line {n}")));
        }

        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        let text = console.text();
        assert_eq!(text.lines().count(), 100, "{text}");
        assert!(text.ends_with("[INFO] line 99\n"), "{text}");
    }

    #[test]
    fn a_flush_reports_non_zero_counters_before_it_returns() {
        let console = Console::default();
        let counters = leaked_counters();
        let tx = start(&console, counters, NEVER);
        counters.bump(Counter::KeyPressShort);
        counters.bump(Counter::KeyPressShort);

        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        assert_eq!(
            console.text(),
            "[WARN] counters since the last report: menu_mask_short=0 key_press_short=2 hold_timer_not_started=0 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=0\n"
        );
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");
        assert_eq!(
            console.text().lines().count(),
            1,
            "counts that are all 0 again are not reported"
        );
    }

    #[test]
    fn a_flush_reports_a_slow_callback() {
        let console = Console::default();
        let counters = leaked_counters();
        let tx = start(&console, counters, NEVER);
        counters.record_callback(Duration::from_millis(7));

        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        assert_eq!(
            console.text(),
            "[WARN] counters since the last report: menu_mask_short=0 key_press_short=0 hold_timer_not_started=0 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=0 longest_callback_us=7000\n"
        );
    }

    #[test]
    fn the_writer_reports_counters_every_period() {
        let console = Console::default();
        let counters = leaked_counters();
        let _tx = start(&console, counters, Duration::from_millis(20));
        counters.bump(Counter::HoldTimerNotStarted);

        let begun = Instant::now();
        while !console.text().contains("hold_timer_not_started=1") {
            assert!(
                begun.elapsed() < WAIT,
                "no counter report within 5 s: {:?}",
                console.text()
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            console
                .text()
                .starts_with("[WARN] counters since the last report: "),
            "{:?}",
            console.text()
        );
    }

    #[test]
    fn a_failing_log_file_is_reported_once_on_the_console() {
        let temp = TempDir::new("writer-failing");
        // A file where the folder should be: no daemon.log can be created.
        let not_a_folder = temp.path().join("key-switch-rs");
        fs::write(&not_a_folder, "").expect("write the blocking file");
        let console = Console::default();
        let tx = start(&console, leaked_counters(), NEVER);
        attach_on(&tx, LogFiles::new(not_a_folder.clone()), WAIT).expect("attached");

        queue(&tx, info("one"));
        queue(&tx, info("two"));
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        let text = console.text();
        let warning = format!(
            "[WARN] log file {}: could not write: ",
            not_a_folder.join("daemon.log").display()
        );
        assert_eq!(text.matches(&warning).count(), 1, "{text}");
        assert!(
            text.starts_with(&format!("[INFO] one\n{warning}")),
            "the warning follows the first line that failed: {text}"
        );
        assert!(text.ends_with(")\n[INFO] two\n"), "{text}");
    }

    #[test]
    fn a_log_file_that_works_again_is_reported_in_the_file_and_on_the_console() {
        let temp = TempDir::new("writer-again");
        let dir = temp.path().join("key-switch-rs");
        fs::write(&dir, "").expect("write the blocking file");
        let console = Console::default();
        let tx = start(&console, leaked_counters(), NEVER);
        attach_on(&tx, LogFiles::new(dir.clone()), WAIT).expect("attached");
        queue(&tx, info("lost"));
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        fs::remove_file(&dir).expect("remove the blocking file");
        queue(&tx, info("kept"));
        assert!(flush_on(&tx, WAIT), "the flush is acknowledged");

        let notice = format!(
            "[INFO] log file {} works again\n",
            dir.join("daemon.log").display()
        );
        let text = console.text();
        assert!(text.ends_with(&format!("[INFO] kept\n{notice}")), "{text}");
        let file = fs::read_to_string(dir.join("daemon.log")).expect("daemon.log");
        let lines: Vec<&str> = file.lines().collect();
        assert_eq!(lines.len(), 2, "{file:?}");
        assert!(lines[0].ends_with(" [INFO] kept"), "{file:?}");
        assert!(
            lines[1].ends_with(&format!(" {}", notice.trim_end())),
            "{file:?}"
        );
    }

    #[test]
    fn a_full_queue_drops_the_line_and_counts_it() {
        let counters = Counters::new();
        let (tx, rx) = sync_channel(1);

        assert!(send_line(&tx, info("kept"), &counters).is_ok());
        assert!(send_line(&tx, info("dropped"), &counters).is_ok());

        assert_eq!(counters.take(), [0, 0, 0, 1, 0, 0], "one line dropped");
        let Ok(WriterMsg::Line(line)) = rx.try_recv() else {
            panic!("the first line waits in the queue");
        };
        assert_eq!(line.console_text(), "[INFO] kept\n");
        assert!(rx.try_recv().is_err(), "the second line was dropped");
    }

    #[test]
    fn a_line_for_a_writer_that_is_gone_comes_back() {
        let counters = Counters::new();
        let (tx, rx) = sync_channel(1);
        drop(rx);

        let Err(line) = send_line(&tx, info("back"), &counters) else {
            panic!("a line for a writer that is gone must come back");
        };
        assert_eq!(line.console_text(), "[INFO] back\n");
        assert_eq!(
            counters.take(),
            [0; Counter::ALL.len()],
            "nothing was dropped"
        );
        assert!(flush_on(&tx, WAIT), "no writer, nothing to wait for");
    }
}
