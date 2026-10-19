//! The keyboard hook's health (roadmap item 2.4, KS-08): the heartbeat that
//! the keyboard callback bumps at every call, which the hook watchdog
//! compares with the raw keyboard input Windows reports.

use std::sync::atomic::{AtomicU64, Ordering};

/// The heartbeat, alone on its cache line: the main thread writes it at
/// every key event, and the watchdog thread reads it only at its ticks, so
/// no field another thread writes shares the line the main core keeps.
#[repr(align(64))]
struct Heartbeat(AtomicU64);

/// What the hooks and the watchdog share about the keyboard hook's health:
/// atomics only, a field of `KeyboardHook` (`KeyboardHook::health`).
pub(crate) struct HealthState {
    heartbeat: Heartbeat,
}

impl HealthState {
    pub(crate) const fn new() -> Self {
        Self {
            heartbeat: Heartbeat(AtomicU64::new(0)),
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
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "first used by Hook: reinstall the keyboard hook on the main thread"
        )
    )]
    pub(crate) fn callbacks(&self) -> u64 {
        self.heartbeat.0.load(Ordering::Acquire)
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
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
    fn the_heartbeat_has_a_cache_line_of_its_own() {
        assert_eq!(std::mem::align_of::<Heartbeat>(), 64);
        assert_eq!(std::mem::size_of::<Heartbeat>(), 64);
    }
}
