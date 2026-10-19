//! Counts of the diagnostics that must not be logged where they happen.
//!
//! The hook callback, and the actions it runs on its thread, never format or
//! queue a log line for a per-event condition: at the auto-repeat rate that
//! would cost an allocation per key press and fill the log. They bump a
//! counter here instead (one atomic RMW), and the `log-writer` thread
//! reports the counts every minute and at every flush (roadmap item 1.7).
//! The callback also records how long it ran; the longest run since the
//! last report is reported with the counts, and on its own once it reaches
//! [`SLOW_CALLBACK`] (roadmap item 2.1).

use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// How many counters there are.
const SLOTS: usize = 11;

/// A condition that is counted instead of logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    /// `SendInput` inserted fewer than the two events of the menu mask key.
    MenuMaskShort,
    /// `SendInput` inserted fewer than the two events of a `PressKey`,
    /// `Media` or `ToggleCapsLock` key press.
    KeyPressShort,
    /// A Hold whose gesture timer could not be set on the main thread never
    /// fired. A Tap that waits for its DoubleTap window and whose timer could
    /// not be set is not counted: it fires at the next key event. The name
    /// dates from the `hold-timer` threads before roadmap item 2.2, and log
    /// readers know it.
    HoldTimerNotStarted,
    /// A log line was dropped because the log writer's queue was full.
    LogLinesDropped,
    /// A `Launch`, `OpenUrl` or `Sequence` was dropped because the action
    /// executor's queue was full.
    ExecutorJobsDropped,
    /// A `Launch`, `OpenUrl` or `Sequence` fired after the action executor
    /// had stopped on its own, so it did nothing.
    ExecutorUnavailable,
    /// The `PostMessageW` of a `SwitchLanguage`, `SwitchLanguageBackward` or
    /// `PostMessage` action failed.
    PostMessageFailed,
    /// A callback found the main thread's state borrowed and passed its
    /// event without deciding it (`hook::main_thread::with_main`, roadmap
    /// item 2.4). Nothing re-enters the state today, so a count is a defect.
    MainStateRefused,
    /// The hook watchdog could not read raw keyboard input, so it did not
    /// count it (roadmap item 2.4): a `GetRawInputData` or a
    /// `GetRawInputBuffer` that failed, a record of a buffered read that
    /// could not be read, or a queue that reports raw input that no read
    /// returns.
    RawInputUnread,
    /// The hook watchdog could not set its tick timer; its wait then times
    /// out at the tick, and the pass sets the timer again (roadmap item
    /// 2.4).
    WatchdogTimerNotSet,
    /// The hook watchdog could not tell whether keyboard input is on the
    /// daemon's desktop when it decided to ask for a reinstall, so it did
    /// not ask (roadmap item 2.4).
    WatchdogDesktopCheckFailed,
}

impl Counter {
    /// Every counter, in the order of [`Counters::take`] and of the report.
    pub const ALL: [Counter; SLOTS] = [
        Counter::MenuMaskShort,
        Counter::KeyPressShort,
        Counter::HoldTimerNotStarted,
        Counter::LogLinesDropped,
        Counter::ExecutorJobsDropped,
        Counter::ExecutorUnavailable,
        Counter::PostMessageFailed,
        Counter::MainStateRefused,
        Counter::RawInputUnread,
        Counter::WatchdogTimerNotSet,
        Counter::WatchdogDesktopCheckFailed,
    ];
}

impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Counter::MenuMaskShort => "menu_mask_short",
            Counter::KeyPressShort => "key_press_short",
            Counter::HoldTimerNotStarted => "hold_timer_not_started",
            Counter::LogLinesDropped => "log_lines_dropped",
            Counter::ExecutorJobsDropped => "executor_jobs_dropped",
            Counter::ExecutorUnavailable => "executor_unavailable",
            Counter::PostMessageFailed => "post_message_failed",
            Counter::MainStateRefused => "main_state_refused",
            Counter::RawInputUnread => "raw_input_unread",
            Counter::WatchdogTimerNotSet => "watchdog_timer_not_set",
            Counter::WatchdogDesktopCheckFailed => "watchdog_desktop_check_failed",
        })
    }
}

/// A keyboard callback that ran at least this long is reported even when
/// every count is 0: the budget of the hook path (roadmap item 2.1).
/// Normal callbacks take microseconds, so normal typing adds no line.
pub const SLOW_CALLBACK: Duration = Duration::from_millis(5);

/// One count per [`Counter`], and the longest keyboard callback.
pub struct Counters {
    counts: [AtomicU64; SLOTS],
    /// The longest keyboard callback since the last `take`, in nanoseconds.
    longest_callback_ns: AtomicU64,
}

/// What [`Counters::take`] returns, each value since the last `take`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// The counts, in the order of [`Counter::ALL`].
    pub counts: [u64; SLOTS],
    /// The longest keyboard callback.
    pub longest_callback: Duration,
}

impl Counters {
    pub const fn new() -> Self {
        Self {
            counts: [const { AtomicU64::new(0) }; SLOTS],
            longest_callback_ns: AtomicU64::new(0),
        }
    }

    /// The slot of `counter`. Constant indices, so the compiler checks them
    /// and no call on the hook thread can panic on a bound.
    fn slot(&self, counter: Counter) -> &AtomicU64 {
        match counter {
            Counter::MenuMaskShort => &self.counts[0],
            Counter::KeyPressShort => &self.counts[1],
            Counter::HoldTimerNotStarted => &self.counts[2],
            Counter::LogLinesDropped => &self.counts[3],
            Counter::ExecutorJobsDropped => &self.counts[4],
            Counter::ExecutorUnavailable => &self.counts[5],
            Counter::PostMessageFailed => &self.counts[6],
            Counter::MainStateRefused => &self.counts[7],
            Counter::RawInputUnread => &self.counts[8],
            Counter::WatchdogTimerNotSet => &self.counts[9],
            Counter::WatchdogDesktopCheckFailed => &self.counts[10],
        }
    }

    /// Counts one more `counter`.
    pub fn bump(&self, counter: Counter) {
        // AcqRel, the house default for an RMW; the counts publish no other
        // data, and on x86-64 every RMW is a locked instruction anyway.
        self.slot(counter).fetch_add(1, Ordering::AcqRel);
    }

    /// The count of `counter` since the last `take`, without taking it, so
    /// that a test can read one slot while others are bumped.
    #[cfg(test)]
    pub fn get(&self, counter: Counter) -> u64 {
        self.slot(counter).load(Ordering::Acquire)
    }

    /// Counts one more `counter` if `result` is an error, without reading
    /// the error: on the hook thread a failed call is counted, never
    /// logged.
    pub fn bump_if_err<T, E>(&self, counter: Counter, result: &Result<T, E>) {
        if result.is_err() {
            self.bump(counter);
        }
    }

    /// Records that a keyboard callback took `took`, if it is the longest
    /// since the last `take`. One RMW: it runs on the hook thread for
    /// every event.
    pub fn record_callback(&self, took: Duration) {
        let ns = u64::try_from(took.as_nanos()).unwrap_or(u64::MAX);
        // AcqRel, the house default for an RMW; the gauge publishes no
        // other data.
        self.longest_callback_ns.fetch_max(ns, Ordering::AcqRel);
    }

    /// The longest keyboard callback since the last `take`.
    pub fn longest_callback(&self) -> Duration {
        Duration::from_nanos(self.longest_callback_ns.load(Ordering::Acquire))
    }

    /// The counts and the longest callback since the last `take`, each
    /// reset to 0 in the same RMW, so that no bump is lost or counted
    /// twice.
    pub fn take(&self) -> Snapshot {
        Snapshot {
            counts: Counter::ALL.map(|counter| self.slot(counter).swap(0, Ordering::AcqRel)),
            longest_callback: Duration::from_nanos(
                self.longest_callback_ns.swap(0, Ordering::AcqRel),
            ),
        }
    }
}

impl Default for Counters {
    fn default() -> Self {
        Self::new()
    }
}

/// The report of `snapshot` (as [`Counters::take`] returns it), which names
/// every counter once and then the longest callback in microseconds, or
/// `None` while every count is 0 and no callback reached
/// [`SLOW_CALLBACK`].
pub fn summary(snapshot: &Snapshot) -> Option<String> {
    if snapshot.counts.iter().all(|&count| count == 0) && snapshot.longest_callback < SLOW_CALLBACK
    {
        return None;
    }
    let mut text = String::from("counters since the last report:");
    for (counter, count) in Counter::ALL.iter().zip(snapshot.counts) {
        let _ = write!(text, " {counter}={count}");
    }
    let _ = write!(
        text,
        " longest_callback_us={}",
        snapshot.longest_callback.as_micros()
    );
    Some(text)
}

/// The daemon's counters, reported by the log writer.
pub static COUNTERS: Counters = Counters::new();

/// Counts one more `counter` in [`COUNTERS`].
pub fn bump(counter: Counter) {
    COUNTERS.bump(counter);
}

/// Counts one more `counter` in [`COUNTERS`] if `result` is an error.
pub fn bump_if_err<T, E>(counter: Counter, result: &Result<T, E>) {
    COUNTERS.bump_if_err(counter, result);
}

/// Records a keyboard callback's run time in [`COUNTERS`].
pub fn record_callback(took: Duration) {
    COUNTERS.record_callback(took);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn no_summary_while_every_count_is_zero() {
        assert_eq!(
            summary(&Snapshot {
                counts: [0; SLOTS],
                longest_callback: Duration::ZERO
            }),
            None
        );
        assert_eq!(summary(&Counters::new().take()), None);
    }

    #[test]
    fn the_summary_names_every_counter_once_one_is_non_zero() {
        assert_eq!(
            summary(&Snapshot {
                counts: [3, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0],
                longest_callback: Duration::from_micros(42),
            })
            .as_deref(),
            Some(
                "counters since the last report: menu_mask_short=3 key_press_short=0 hold_timer_not_started=1 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=2 post_message_failed=0 main_state_refused=0 raw_input_unread=0 watchdog_timer_not_set=0 watchdog_desktop_check_failed=0 longest_callback_us=42"
            )
        );
    }

    #[test]
    fn take_returns_the_counts_and_resets_them() {
        let counters = Counters::new();
        counters.bump(Counter::KeyPressShort);
        counters.bump(Counter::KeyPressShort);
        counters.bump(Counter::LogLinesDropped);

        assert_eq!(counters.take().counts, [0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            counters.take().counts,
            [0; SLOTS],
            "take resets every count"
        );
    }

    #[test]
    fn the_longest_callback_is_kept_until_taken() {
        let counters = Counters::new();
        counters.record_callback(Duration::from_millis(3));
        counters.record_callback(Duration::from_millis(1));
        assert_eq!(counters.longest_callback(), Duration::from_millis(3));

        assert_eq!(counters.take().longest_callback, Duration::from_millis(3));
        assert_eq!(
            counters.longest_callback(),
            Duration::ZERO,
            "take resets it"
        );
        assert_eq!(counters.take().longest_callback, Duration::ZERO);
    }

    #[test]
    fn a_slow_callback_alone_is_reported_and_a_fast_one_is_not() {
        // Roadmap item 2.1: a callback is normally a few microseconds, so
        // the gauge triggers a report only from the 5 ms budget, and normal
        // typing adds no line per minute.
        let fast = Snapshot {
            counts: [0; SLOTS],
            longest_callback: Duration::from_micros(4_999),
        };
        assert_eq!(summary(&fast), None);

        let slow = Snapshot {
            counts: [0; SLOTS],
            longest_callback: SLOW_CALLBACK,
        };
        assert_eq!(
            summary(&slow).as_deref(),
            Some(
                "counters since the last report: menu_mask_short=0 key_press_short=0 hold_timer_not_started=0 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=0 post_message_failed=0 main_state_refused=0 raw_input_unread=0 watchdog_timer_not_set=0 watchdog_desktop_check_failed=0 longest_callback_us=5000"
            )
        );
    }

    #[test]
    fn bump_if_err_counts_only_an_error() {
        let counters = Counters::new();
        counters.bump_if_err(Counter::PostMessageFailed, &Ok::<(), ()>(()));
        counters.bump_if_err(Counter::PostMessageFailed, &Err::<(), &str>("denied"));

        let counts = counters.take().counts;
        let failed = Counter::ALL
            .iter()
            .zip(counts)
            .find(|(counter, _)| **counter == Counter::PostMessageFailed)
            .map(|(_, count)| count);
        assert_eq!(failed, Some(1));
        assert_eq!(counts.iter().sum::<u64>(), 1, "{counts:?}");
    }

    #[test]
    fn bump_if_err_ignores_an_ok_result() {
        let counters = Counters::new();
        counters.bump_if_err(Counter::PostMessageFailed, &Ok::<(), ()>(()));

        let counts = counters.take().counts;
        assert_eq!(
            counts.iter().sum::<u64>(),
            0,
            "an Ok result counts nothing: {counts:?}"
        );
    }

    #[test]
    fn counter_names_are_snake_case() {
        let names: Vec<String> = Counter::ALL.iter().map(Counter::to_string).collect();
        assert_eq!(
            names,
            [
                "menu_mask_short",
                "key_press_short",
                "hold_timer_not_started",
                "log_lines_dropped",
                "executor_jobs_dropped",
                "executor_unavailable",
                "post_message_failed",
                "main_state_refused",
                "raw_input_unread",
                "watchdog_timer_not_set",
                "watchdog_desktop_check_failed"
            ]
        );
    }
}
