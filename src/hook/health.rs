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

    #[test]
    fn the_heartbeat_has_a_cache_line_of_its_own() {
        assert_eq!(std::mem::align_of::<Heartbeat>(), 64);
        assert_eq!(std::mem::size_of::<Heartbeat>(), 64);
    }
}
