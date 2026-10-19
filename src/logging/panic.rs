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
