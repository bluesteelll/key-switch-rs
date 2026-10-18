//! Counts of the diagnostics that must not be logged where they happen.
//!
//! The hook callback, and the actions it runs on its thread, never format or
//! queue a log line for a per-event condition: at the auto-repeat rate that
//! would cost an allocation per key press and fill the log. They bump a
//! counter here instead (one atomic RMW), and the `log-writer` thread
//! reports the counts every minute and at every flush (roadmap item 1.7).

use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicU64, Ordering};

/// How many counters there are.
const SLOTS: usize = 6;

/// A condition that is counted instead of logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    /// `SendInput` inserted fewer than the two events of the menu mask key.
    MenuMaskShort,
    /// `SendInput` inserted fewer than the two events of a `PressKey`,
    /// `Media` or `ToggleCapsLock` key press.
    KeyPressShort,
    /// A `hold-timer` thread could not be started, so that Hold never fired.
    HoldTimerNotStarted,
    /// A log line was dropped because the log writer's queue was full.
    LogLinesDropped,
    /// A `Launch`, `OpenUrl` or `Sequence` was dropped because the action
    /// executor's queue was full.
    ExecutorJobsDropped,
    /// A `Launch`, `OpenUrl` or `Sequence` fired after the action executor
    /// had stopped on its own, so it did nothing.
    ExecutorUnavailable,
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
        })
    }
}

/// One count per [`Counter`].
pub struct Counters([AtomicU64; SLOTS]);

impl Counters {
    pub const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; SLOTS])
    }

    /// The slot of `counter`. Constant indices, so the compiler checks them
    /// and no call on the hook thread can panic on a bound.
    fn slot(&self, counter: Counter) -> &AtomicU64 {
        match counter {
            Counter::MenuMaskShort => &self.0[0],
            Counter::KeyPressShort => &self.0[1],
            Counter::HoldTimerNotStarted => &self.0[2],
            Counter::LogLinesDropped => &self.0[3],
            Counter::ExecutorJobsDropped => &self.0[4],
            Counter::ExecutorUnavailable => &self.0[5],
        }
    }

    /// Counts one more `counter`.
    pub fn bump(&self, counter: Counter) {
        // AcqRel, the house default for an RMW; the counts publish no other
        // data, and on x86-64 every RMW is a locked instruction anyway.
        self.slot(counter).fetch_add(1, Ordering::AcqRel);
    }

    /// The counts since the last `take`, in the order of [`Counter::ALL`],
    /// each reset to 0 in the same RMW, so that no bump is lost or counted
    /// twice.
    pub fn take(&self) -> [u64; SLOTS] {
        Counter::ALL.map(|counter| self.slot(counter).swap(0, Ordering::AcqRel))
    }
}

impl Default for Counters {
    fn default() -> Self {
        Self::new()
    }
}

/// The report of `counts` (as [`Counters::take`] returns them), which names
/// every counter once, or `None` while every count is 0.
pub fn summary(counts: &[u64; SLOTS]) -> Option<String> {
    if counts.iter().all(|&count| count == 0) {
        return None;
    }
    let mut text = String::from("counters since the last report:");
    for (counter, count) in Counter::ALL.iter().zip(counts) {
        let _ = write!(text, " {counter}={count}");
    }
    Some(text)
}

/// The daemon's counters, reported by the log writer.
pub static COUNTERS: Counters = Counters::new();

/// Counts one more `counter` in [`COUNTERS`].
pub fn bump(counter: Counter) {
    COUNTERS.bump(counter);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn no_summary_while_every_count_is_zero() {
        assert_eq!(summary(&[0; SLOTS]), None);
        assert_eq!(summary(&Counters::new().take()), None);
    }

    #[test]
    fn the_summary_names_every_counter_once_one_is_non_zero() {
        assert_eq!(
            summary(&[3, 0, 1, 0, 0, 2]).as_deref(),
            Some(
                "counters since the last report: menu_mask_short=3 key_press_short=0 hold_timer_not_started=1 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=2"
            )
        );
    }

    #[test]
    fn take_returns_the_counts_and_resets_them() {
        let counters = Counters::new();
        counters.bump(Counter::KeyPressShort);
        counters.bump(Counter::KeyPressShort);
        counters.bump(Counter::LogLinesDropped);

        assert_eq!(counters.take(), [0, 2, 0, 1, 0, 0]);
        assert_eq!(counters.take(), [0; SLOTS], "take resets every count");
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
                "counters since the last report: menu_mask_short=0 key_press_short=0 hold_timer_not_started=0 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=0 longest_callback_us=5000"
            )
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
                "executor_unavailable"
            ]
        );
    }
}
