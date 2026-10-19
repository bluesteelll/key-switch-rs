//! The keyboard hook's health (roadmap item 2.4, KS-08): the heartbeat that
//! the keyboard callback bumps at every call, which the hook watchdog
//! compares with the raw keyboard input Windows reports, and what the
//! watchdog asks the main thread when the two disagree.

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// The heartbeat, alone on its cache line: the main thread writes it at
/// every key event, and the watchdog thread reads it only at its ticks, so
/// no field another thread writes shares the line the main core keeps.
#[repr(align(64))]
struct Heartbeat(AtomicU64);

/// What the hooks and the watchdog share about the keyboard hook's health:
/// atomics only, a field of `KeyboardHook` (`KeyboardHook::health`).
pub(crate) struct HealthState {
    heartbeat: Heartbeat,
    /// Whether the keyboard hook is installed: set by `KeyboardHook::install`
    /// and cleared by `uninstall`, on the main thread, and read by the
    /// watchdog, which counts raw input only while it is (ruling I3).
    hook_installed: AtomicBool,
}

impl HealthState {
    pub(crate) const fn new() -> Self {
        Self {
            heartbeat: Heartbeat(AtomicU64::new(0)),
            hook_installed: AtomicBool::new(false),
        }
    }

    /// One more call of the keyboard callback. One RMW on the hook path, at
    /// every call with code >= 0, whether remapping is on or off and
    /// whether the event is the daemon's own: raw input reports those too.
    pub(crate) fn callback_seen(&self) {
        // AcqRel, the house default for an RMW; the count publishes no other
        // data, and on x86-64 every RMW is one locked instruction anyway.
        self.heartbeat.0.fetch_add(1, Ordering::AcqRel);
    }

    /// How many calls the keyboard callback has had.
    pub(crate) fn callbacks(&self) -> u64 {
        self.heartbeat.0.load(Ordering::Acquire)
    }

    /// Notes whether the keyboard hook is installed. Main thread, around
    /// its install and its removal; the flag publishes no other data.
    pub(crate) fn set_hook_installed(&self, installed: bool) {
        self.hook_installed.store(installed, Ordering::Release);
    }

    /// Whether the keyboard hook is installed.
    pub(crate) fn hook_installed(&self) -> bool {
        self.hook_installed.load(Ordering::Acquire)
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

/// The hook watchdog's request to the main thread: keys arrived and the
/// keyboard callback stayed silent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReinstallRequest {
    /// The heartbeat the watchdog read when it decided. The main thread
    /// reinstalls only if it has not moved since.
    pub(crate) seen: u64,
    /// How long no callback was seen while keys arrived.
    pub(crate) silent_for: Duration,
    /// Which attempt this is since the last callback seen, from 1.
    pub(crate) attempt: u32,
}

/// What the main thread did with a [`ReinstallRequest`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReinstallOutcome {
    /// The hook answered once the main thread was free, and was kept.
    Alive,
    /// A new hook replaced the old one.
    Reinstalled,
    /// Windows refused the new hook, and the old one was kept.
    Failed,
    /// No keyboard hook is installed, so none was: the daemon is shutting
    /// down, or the session ends.
    NotInstalled,
}

impl fmt::Display for ReinstallOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ReinstallOutcome::Alive => "the hook answered and was kept",
            ReinstallOutcome::Reinstalled => "the hook was reinstalled",
            ReinstallOutcome::Failed => "the new hook was refused; the old one was kept",
            ReinstallOutcome::NotInstalled => "no keyboard hook is installed",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;

    #[test]
    fn the_heartbeat_counts_each_call() {
        let health = HealthState::new();
        assert_eq!(health.callbacks(), 0);
        for _ in 0..3 {
            health.callback_seen();
        }
        assert_eq!(health.callbacks(), 3);
    }

    #[test]
    fn every_reinstall_outcome_has_its_own_text() {
        let texts: Vec<String> = [
            ReinstallOutcome::Alive,
            ReinstallOutcome::Reinstalled,
            ReinstallOutcome::Failed,
            ReinstallOutcome::NotInstalled,
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(
            texts,
            [
                "the hook answered and was kept",
                "the hook was reinstalled",
                "the new hook was refused; the old one was kept",
                "no keyboard hook is installed",
            ]
        );
    }

    /// The health of a watchdog that runs, with nothing else to say.
    fn healthy() -> HookHealth {
        HookHealth {
            watched: true,
            reinstalls: 0,
            last_reinstall: None,
            last_attempt_failed: false,
            timer_failing: false,
        }
    }

    #[test]
    fn the_hook_health_reads_ok_not_watched_or_failed_with_its_reinstalls() {
        // Roadmap item 2.4: `swch status` shows the hook's health after the
        // binding count; `reinstalls: N` is the acceptance's own token.
        let at = UNIX_EPOCH + Duration::from_millis(1_760_866_867_123);
        let table = [
            (healthy(), "hook ok, reinstalls: 0"),
            (
                HookHealth {
                    watched: false,
                    ..healthy()
                },
                "hook not watched, reinstalls: 0",
            ),
            (
                HookHealth {
                    reinstalls: 2,
                    last_reinstall: Some(at),
                    last_attempt_failed: true,
                    ..healthy()
                },
                "hook reinstall failed, reinstalls: 2, last 2025-10-19T09:41:07.123Z",
            ),
            (
                HookHealth {
                    timer_failing: true,
                    ..healthy()
                },
                "hook watchdog timer failing, reinstalls: 0",
            ),
            (
                HookHealth {
                    watched: false,
                    last_attempt_failed: true,
                    timer_failing: true,
                    ..healthy()
                },
                "hook not watched, reinstalls: 0",
            ),
        ];
        for (health, text) in table {
            assert_eq!(health.to_string(), text, "{health:?}");
        }
    }

    #[test]
    fn a_reinstall_is_recorded_with_its_time() {
        let health = HealthState::new();
        health.set_watched(true);
        assert_eq!(health.snapshot(), healthy());
        health.record_reinstall(UNIX_EPOCH + Duration::from_millis(1_760_866_867_123));
        assert_eq!(
            health.snapshot().to_string(),
            "hook ok, reinstalls: 1, last 2025-10-19T09:41:07.123Z"
        );
    }

    #[test]
    fn a_failed_attempt_reads_reinstall_failed_until_the_next_reinstall() {
        let health = HealthState::new();
        health.set_watched(true);
        health.record_failed_attempt();
        assert!(health.snapshot().last_attempt_failed);
        assert_eq!(
            health.snapshot().to_string(),
            "hook reinstall failed, reinstalls: 0"
        );
        health.record_reinstall(UNIX_EPOCH + Duration::from_secs(1));
        assert!(!health.snapshot().last_attempt_failed);
    }

    #[test]
    fn a_failed_attempt_clears_once_the_hook_answers() {
        // Ruling M3: the old hook kept after a failed attempt can be alive;
        // once it answers, the status no longer says it failed.
        let health = HealthState::new();
        health.set_watched(true);
        health.record_failed_attempt();
        health.hook_answered();
        assert_eq!(health.snapshot(), healthy());
    }

    #[test]
    fn a_tick_timer_that_cannot_be_set_is_not_hook_ok() {
        // Ruling I1: while the watchdog's timer keeps failing, the status
        // does not say `hook ok`.
        let health = HealthState::new();
        health.set_watched(true);
        health.set_timer_failing(true);
        assert_eq!(
            health.snapshot().to_string(),
            "hook watchdog timer failing, reinstalls: 0"
        );
        health.set_timer_failing(false);
        assert_eq!(health.snapshot(), healthy());
    }

    #[test]
    fn the_heartbeat_has_a_cache_line_of_its_own() {
        assert_eq!(std::mem::align_of::<Heartbeat>(), 64);
        assert_eq!(std::mem::size_of::<Heartbeat>(), 64);
    }
}
