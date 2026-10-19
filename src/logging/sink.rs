//! A log line and the rotating files it is written to.
//!
//! Lines are built on the thread that logs (one `String`); the `log-writer`
//! thread expands them into their file and console forms and owns the files.

use std::fmt::{self, Write as _};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{FATAL_TARGET, LOG_FILE_NAME};

/// The length of a timestamp, `YYYY-MM-DDTHH:MM:SS.mmmZ`, before year 10000.
#[cfg(test)]
pub(crate) const TIMESTAMP_LEN: usize = 24;

/// The size `daemon.log` may reach before it rotates: 1 MiB, the "1 MB" of
/// roadmap item 1.7.
pub(crate) const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// How many files a rotation keeps: `daemon.log`, `daemon.1.log` and
/// `daemon.2.log`.
pub(crate) const KEPT_FILES: usize = 3;

/// The crate's own records name no target in their line.
const OWN_TARGET: &str = "key_switch_rs";

/// One record: `{ts} [{TAG}] {target: }{message}`, with the message's
/// trailing line breaks trimmed.
pub(crate) struct LogLine {
    text: String,
    /// Where the console form starts: after the timestamp and its space.
    console_from: usize,
    /// Where the record's own text starts: after `[{TAG}] `. The text
    /// before it is what every further line of the record repeats in the
    /// file.
    body_from: usize,
}

impl LogLine {
    /// Builds the line on the calling thread. Never panics: a `Display`
    /// that fails leaves the message cut short, because `write!` into a
    /// `String` only passes the error on, where `format!` would panic.
    pub(crate) fn new(
        now: SystemTime,
        level: log::Level,
        target: &str,
        args: &fmt::Arguments<'_>,
    ) -> Self {
        let mut text = String::new();
        write_timestamp(&mut text, now);
        text.push(' ');
        let console_from = text.len();
        let _ = write!(text, "[{}] ", tag(level, target));
        let body_from = text.len();
        if !is_own_target(target) {
            let _ = write!(text, "{target}: ");
        }
        let _ = text.write_fmt(*args);
        let kept = text.trim_end_matches(['\r', '\n']).len();
        text.truncate(kept);
        Self {
            text,
            console_from,
            body_from,
        }
    }

    /// The line as `daemon.log` gets it: every physical line of the record
    /// starts with the record's timestamp and tag, so that a reader that
    /// goes line by line never meets a line without them. Ends with one
    /// `\n`.
    pub(crate) fn file_text(&self) -> String {
        let repeated = self.text.get(..self.body_from).unwrap_or_default();
        let mut file = String::with_capacity(self.text.len() + 1);
        for (index, line) in self.text.lines().enumerate() {
            if index > 0 {
                file.push_str(repeated);
            }
            file.push_str(line);
            file.push('\n');
        }
        file
    }

    /// The line as the console gets it: without the timestamp, the tag on
    /// the first line only, so that every documented console line stays as
    /// it was. Ends with one `\n`.
    pub(crate) fn console_text(&self) -> String {
        let mut console = self
            .text
            .get(self.console_from..)
            .unwrap_or(&self.text)
            .to_owned();
        console.push('\n');
        console
    }
}

/// `FATAL` for an error with [`FATAL_TARGET`], which `log` has no level for,
/// else the level.
fn tag(level: log::Level, target: &str) -> &'static str {
    if level == log::Level::Error && target == FATAL_TARGET {
        "FATAL"
    } else {
        level.as_str()
    }
}

/// Whether `target` is this crate or one of its modules.
fn is_own_target(target: &str) -> bool {
    target
        .strip_prefix(OWN_TARGET)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
}

/// `now` in UTC as ISO 8601 with milliseconds: `2026-10-16T09:41:07.123Z`.
/// A time before 1970 is written as the epoch.
pub(crate) fn write_timestamp(out: &mut String, now: SystemTime) {
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    let seconds = since_epoch.as_secs();
    let (year, month, day) = civil_from_days(seconds / 86_400);
    let of_day = seconds % 86_400;
    let _ = write!(
        out,
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60,
        since_epoch.subsec_millis()
    );
}

/// The proleptic Gregorian date of `days` after 1970-01-01, as
/// `(year, month, day)`: Howard Hinnant's `civil_from_days`, for days that
/// are never negative.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    // Days since 0000-03-01, which starts a 400-year era with the leap day
    // at the end of its years.
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    // Months counted from March.
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = era * 400 + year_of_era + u64::from(month <= 2);
    (year, month, day)
}

/// Why a line did not go into the log files as it should.
#[derive(Debug)]
pub(crate) enum WriteError {
    /// The files could not be rotated; the line went into `daemon.log`,
    /// which grows on until a rotation succeeds.
    Rotate(io::Error),
    /// The line is lost for the file.
    Write(io::Error),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Rotate(error) => write!(
                f,
                "could not rotate, so {LOG_FILE_NAME} grows past its limit: {error}"
            ),
            WriteError::Write(error) => write!(f, "could not write: {error}"),
        }
    }
}

/// `daemon.log` and its rotated files in one folder. No handle is kept
/// between lines: the user can open, move or delete the files at any time,
/// and a deleted file or folder comes back with the next line.
pub(crate) struct LogFiles {
    dir: PathBuf,
    limit: u64,
}

impl LogFiles {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            limit: MAX_FILE_BYTES,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_limit(dir: PathBuf, limit: u64) -> Self {
        Self { dir, limit }
    }

    /// `daemon.log`, the file lines are written to.
    pub(crate) fn path(&self) -> PathBuf {
        self.file(0)
    }

    /// `daemon.log` for 0, else `daemon.{index}.log`.
    fn file(&self, index: usize) -> PathBuf {
        if index == 0 {
            self.dir.join(LOG_FILE_NAME)
        } else {
            self.dir.join(format!("daemon.{index}.log"))
        }
    }

    /// Appends `text` to `daemon.log`. A file that already holds something
    /// and would pass the limit with `text` is rotated first, so only a
    /// single line longer than the limit makes a file larger than it.
    pub(crate) fn write_line(&self, text: &str) -> Result<(), WriteError> {
        let mut file = self.open().map_err(WriteError::Write)?;
        let size = file.metadata().map_err(WriteError::Write)?.len();
        let mut rotated = Ok(());
        if size > 0 && size.saturating_add(text.len() as u64) > self.limit {
            drop(file);
            rotated = self.rotate();
            file = self.open().map_err(WriteError::Write)?;
        }
        file.write_all(text.as_bytes()).map_err(WriteError::Write)?;
        rotated.map_err(WriteError::Rotate)
    }

    /// Opens `daemon.log` for appending, creating it and, on demand, its
    /// folder.
    fn open(&self) -> io::Result<File> {
        let path = self.path();
        let open = || OpenOptions::new().create(true).append(true).open(&path);
        match open() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(&self.dir)?;
                open()
            }
            opened => opened,
        }
    }

    /// Moves every file one place down, `daemon.log` to `daemon.1.log`, and
    /// drops the oldest. Stops at the first rename that fails, so that a
    /// file another program holds never makes a later rename replace a
    /// file whose content has not moved on. A file that does not exist is
    /// skipped.
    fn rotate(&self) -> io::Result<()> {
        for index in (1..KEPT_FILES).rev() {
            if let Err(error) = fs::rename(self.file(index - 1), self.file(index))
                && error.kind() != io::ErrorKind::NotFound
            {
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::Path;

    use super::*;
    use crate::config::testing::TempDir;

    /// `FILE_SHARE_READ` only: no delete sharing, so a rename of the file,
    /// or onto it, fails while it is open.
    const FILE_SHARE_READ: u32 = 0x1;

    fn at(ms: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(ms)
    }

    fn timestamp(now: SystemTime) -> String {
        let mut text = String::new();
        write_timestamp(&mut text, now);
        text
    }

    /// A record logged at 2026-10-16T09:41:07.123Z.
    fn record(level: log::Level, target: &str, args: fmt::Arguments<'_>) -> LogLine {
        LogLine::new(at(1_792_143_667_123), level, target, &args)
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .expect("list the log folder")
            .map(|entry| {
                entry
                    .expect("a folder entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn timestamps_are_iso_8601_utc_with_milliseconds() {
        for (ms, expected) in [
            (0, "1970-01-01T00:00:00.000Z"),
            (951_868_799_999, "2000-02-29T23:59:59.999Z"),
            (1_792_143_667_123, "2026-10-16T09:41:07.123Z"),
            (4_107_542_399_999, "2100-02-28T23:59:59.999Z"),
            (4_107_542_400_000, "2100-03-01T00:00:00.000Z"),
        ] {
            let written = timestamp(at(ms));
            assert_eq!(written, expected, "{ms} ms after the epoch");
            assert_eq!(written.len(), TIMESTAMP_LEN);
        }
    }

    #[test]
    fn a_time_before_1970_is_written_as_the_epoch() {
        let before = UNIX_EPOCH - Duration::from_secs(86_400);
        assert_eq!(timestamp(before), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn a_line_is_the_timestamp_the_level_and_the_message() {
        let line = record(
            log::Level::Info,
            "key_switch_rs::hook::config_watcher",
            format_args!("config reloaded ({} bindings live)", 2),
        );
        assert_eq!(
            line.file_text(),
            "2026-10-16T09:41:07.123Z [INFO] config reloaded (2 bindings live)\n"
        );
        assert_eq!(
            line.console_text(),
            "[INFO] config reloaded (2 bindings live)\n"
        );
    }

    #[test]
    fn an_error_with_the_fatal_target_is_tagged_fatal() {
        fn console(level: log::Level, target: &str) -> String {
            record(level, target, format_args!("x")).console_text()
        }
        assert_eq!(console(log::Level::Error, FATAL_TARGET), "[FATAL] x\n");
        assert_eq!(console(log::Level::Error, "key_switch_rs"), "[ERROR] x\n");
        assert_eq!(console(log::Level::Info, FATAL_TARGET), "[INFO] x\n");
        assert_eq!(console(log::Level::Warn, "key_switch_rs"), "[WARN] x\n");
    }

    #[test]
    fn a_record_of_another_crate_names_its_target() {
        let line = record(
            log::Level::Warn,
            "notify::windows",
            format_args!("watch lost"),
        );
        assert_eq!(line.console_text(), "[WARN] notify::windows: watch lost\n");
        assert_eq!(
            line.file_text(),
            "2026-10-16T09:41:07.123Z [WARN] notify::windows: watch lost\n"
        );
        let similar = record(log::Level::Warn, "key_switch_rs_extra", format_args!("y"));
        assert_eq!(similar.console_text(), "[WARN] key_switch_rs_extra: y\n");
    }

    #[test]
    fn every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file() {
        // The shape of a reload error: a header, one line per invalid
        // binding, and the trailing line break of `ConfigError::Bindings`.
        let line = record(
            log::Level::Error,
            "key_switch_rs::hook::config_watcher",
            format_args!(
                "config reload failed:\n{}",
                "config has 1 invalid binding(s):\n  - binding #0 [F13]: bad\n"
            ),
        );
        assert_eq!(
            line.file_text(),
            "2026-10-16T09:41:07.123Z [ERROR] config reload failed:\n\
             2026-10-16T09:41:07.123Z [ERROR] config has 1 invalid binding(s):\n\
             2026-10-16T09:41:07.123Z [ERROR]   - binding #0 [F13]: bad\n"
        );
        assert_eq!(
            line.console_text(),
            "[ERROR] config reload failed:\n\
             config has 1 invalid binding(s):\n  - binding #0 [F13]: bad\n",
            "the console keeps the tag on the first line only, and no blank line"
        );

        let other = record(
            log::Level::Warn,
            "notify::windows",
            format_args!("one\r\ntwo\n\n"),
        );
        assert_eq!(
            other.file_text(),
            "2026-10-16T09:41:07.123Z [WARN] notify::windows: one\n\
             2026-10-16T09:41:07.123Z [WARN] two\n",
            "the target goes on the first line only, and CR LF is one line break"
        );
        assert_eq!(other.console_text(), "[WARN] notify::windows: one\r\ntwo\n");
    }

    #[test]
    fn the_first_line_creates_the_folder_and_daemon_log() {
        let temp = TempDir::new("log-create");
        let dir = temp.path().join("Local").join("key-switch-rs");
        let files = LogFiles::new(dir.clone());

        files.write_line("one\n").expect("the line is written");

        assert_eq!(files.path(), dir.join("daemon.log"));
        assert_eq!(read(&files.path()), "one\n");
    }

    #[test]
    fn lines_are_appended_to_an_existing_daemon_log() {
        let temp = TempDir::new("log-append");
        fs::write(temp.path().join("daemon.log"), "old\n").expect("write daemon.log");
        let files = LogFiles::new(temp.path().to_path_buf());

        files.write_line("one\n").expect("the line is written");
        files.write_line("two\n").expect("the line is written");

        assert_eq!(read(&files.path()), "old\none\ntwo\n");
    }

    #[test]
    fn a_line_that_would_pass_the_limit_rotates_first() {
        let temp = TempDir::new("log-rotate");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 100);
        let thirty = format!("{}\n", "a".repeat(29));
        for _ in 0..3 {
            files.write_line(&thirty).expect("the line is written");
        }

        let next = format!("{}\n", "b".repeat(29));
        files.write_line(&next).expect("the line is written");

        assert_eq!(read(&temp.path().join("daemon.1.log")), thirty.repeat(3));
        assert_eq!(read(&files.path()), next);
    }

    #[test]
    fn a_line_that_reaches_the_limit_exactly_does_not_rotate() {
        let temp = TempDir::new("log-exact");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 100);
        let fifty = format!("{}\n", "a".repeat(49));

        files.write_line(&fifty).expect("the line is written");
        files.write_line(&fifty).expect("the line is written");

        assert_eq!(read(&files.path()), fifty.repeat(2));
        assert_eq!(names_in(temp.path()), ["daemon.log"]);
    }

    #[test]
    fn rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest() {
        let temp = TempDir::new("log-keep");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 10);
        // Each batch fills a file, so the next one rotates.
        let batches = [
            "batch 1\n",
            "batch 2\n",
            "batch 3\n",
            "batch 4\n",
            "batch 5\n",
        ];
        for batch in batches {
            files.write_line(batch).expect("the line is written");
        }

        assert_eq!(
            names_in(temp.path()),
            ["daemon.1.log", "daemon.2.log", "daemon.log"]
        );
        assert_eq!(read(&temp.path().join("daemon.2.log")), "batch 3\n");
        assert_eq!(read(&temp.path().join("daemon.1.log")), "batch 4\n");
        assert_eq!(read(&files.path()), "batch 5\n");
    }

    #[test]
    fn a_line_longer_than_the_limit_goes_into_an_empty_file() {
        let temp = TempDir::new("log-long");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 10);
        let long = format!("{}\n", "x".repeat(40));

        files.write_line("short\n").expect("the line is written");
        files.write_line(&long).expect("the line is written");
        files.write_line("after\n").expect("the line is written");

        assert_eq!(read(&temp.path().join("daemon.2.log")), "short\n");
        assert_eq!(read(&temp.path().join("daemon.1.log")), long);
        assert_eq!(read(&files.path()), "after\n");
    }

    #[test]
    fn a_line_longer_than_the_limit_does_not_rotate_an_empty_daemon_log() {
        let temp = TempDir::new("log-long-empty");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 10);
        let long = format!("{}\n", "x".repeat(40));
        fs::write(temp.path().join("daemon.1.log"), "keep\n").expect("write daemon.1.log");

        files.write_line(&long).expect("the line is written");

        assert_eq!(
            read(&files.path()),
            long,
            "the long line goes into the empty daemon.log"
        );
        assert_eq!(
            read(&temp.path().join("daemon.1.log")),
            "keep\n",
            "an empty daemon.log is not pushed down over daemon.1.log"
        );
        assert_eq!(
            names_in(temp.path()),
            ["daemon.1.log", "daemon.log"],
            "nothing is rotated into daemon.2.log"
        );
    }

    #[test]
    fn daemon_log_rotates_at_one_mebibyte() {
        let temp = TempDir::new("log-mebibyte");
        let files = LogFiles::new(temp.path().to_path_buf());
        File::create(files.path())
            .and_then(|file| file.set_len(MAX_FILE_BYTES - 10))
            .expect("pre-size daemon.log");

        files
            .write_line("123456789\n")
            .expect("the line is written");
        assert_eq!(
            fs::metadata(files.path()).expect("daemon.log").len(),
            MAX_FILE_BYTES,
            "a line that fills daemon.log to 1 MiB exactly does not rotate it"
        );
        assert_eq!(names_in(temp.path()), ["daemon.log"]);

        files.write_line("\n").expect("the line is written");
        assert_eq!(
            fs::metadata(temp.path().join("daemon.1.log"))
                .expect("daemon.1.log")
                .len(),
            MAX_FILE_BYTES
        );
        assert_eq!(read(&files.path()), "\n");
    }

    #[test]
    fn the_limit_is_one_mebibyte_and_three_files_are_kept() {
        assert_eq!(MAX_FILE_BYTES, 1_048_576);
        assert_eq!(KEPT_FILES, 3);
        assert_eq!(LOG_FILE_NAME, "daemon.log");
    }

    #[test]
    fn a_rotation_that_cannot_rename_keeps_writing_to_daemon_log() {
        let temp = TempDir::new("log-held-1");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 10);
        files.write_line("first 1\n").expect("the line is written");
        files.write_line("second 1\n").expect("the line is written");
        let blocker = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(temp.path().join("daemon.1.log"))
            .expect("hold daemon.1.log");

        let result = files.write_line("third 1\n");

        assert!(
            matches!(result, Err(WriteError::Rotate(_))),
            "a held daemon.1.log stops the rotation: {result:?}"
        );
        assert_eq!(read(&files.path()), "second 1\nthird 1\n");
        assert_eq!(read(&temp.path().join("daemon.1.log")), "first 1\n");

        drop(blocker);
        files
            .write_line("fourth 1\n")
            .expect("the rotation works again");
        assert_eq!(read(&temp.path().join("daemon.2.log")), "first 1\n");
        assert_eq!(
            read(&temp.path().join("daemon.1.log")),
            "second 1\nthird 1\n"
        );
        assert_eq!(read(&files.path()), "fourth 1\n");
    }

    #[test]
    fn a_rotation_stops_at_the_first_rename_that_fails() {
        let temp = TempDir::new("log-held-2");
        let files = LogFiles::with_limit(temp.path().to_path_buf(), 10);
        for text in ["first 1\n", "second 1\n", "third 1\n"] {
            files.write_line(text).expect("the line is written");
        }
        let blocker = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(temp.path().join("daemon.2.log"))
            .expect("hold daemon.2.log");

        let result = files.write_line("fourth 1\n");

        assert!(
            matches!(result, Err(WriteError::Rotate(_))),
            "a held daemon.2.log stops the rotation: {result:?}"
        );
        assert_eq!(
            read(&temp.path().join("daemon.1.log")),
            "second 1\n",
            "daemon.1.log keeps its content when daemon.2.log cannot be replaced"
        );
        assert_eq!(read(&temp.path().join("daemon.2.log")), "first 1\n");
        assert_eq!(read(&files.path()), "third 1\nfourth 1\n");
        drop(blocker);
    }

    #[test]
    fn a_deleted_folder_is_created_again_on_the_next_line() {
        let temp = TempDir::new("log-deleted");
        let dir = temp.path().join("key-switch-rs");
        let files = LogFiles::new(dir.clone());
        files.write_line("one\n").expect("the line is written");

        fs::remove_dir_all(&dir).expect("delete the log folder");
        files.write_line("two\n").expect("the line is written");

        assert_eq!(read(&files.path()), "two\n");
    }
}
