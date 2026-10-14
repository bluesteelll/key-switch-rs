//! Pending `Tap`, `Hold` and `DoubleTap` gestures, as plain data.
//!
//! Every entry is keyed by the virtual-key code of the key-down that armed
//! it, and each gesture kind has its own table, so Tap and Hold on one key
//! are armed and resolved independently, as before the engine existed.
//!
//! Time comes in as arguments; nothing here reads a clock or sleeps:
//!
//! - **Tap** stores a deadline. The key-up fires it only before the
//!   deadline. An entry whose deadline has passed counts as absent, which is
//!   what the per-press cleanup timer used to achieve by deleting it.
//! - **DoubleTap** stores a deadline the same way. A second key-down before
//!   it fires the stored action; a later one arms afresh.
//! - **Hold** must fire without an event, so arming it returns a
//!   [`TimerRequest`]. Each armed entry gets a new generation number, and the
//!   timer token carries it: a timer only fires the entry it was started
//!   for. A timer left over from an earlier press of the same key finds a
//!   different generation and does nothing (KS-11).
//!
//!   A key-up before the term cancels the Hold. A key-up at or after the
//!   term does not: the key was held for the whole term, so the Hold stays
//!   armed until its timer fires it, even when the timer runs after the
//!   key-up because its thread woke late. Only the timer ever fires a Hold,
//!   which keeps Hold actions off the hook thread. With equal Tap and Hold
//!   terms exactly one of the two fires (KS-25): the Tap needs a key-up
//!   strictly before the shared deadline, the Hold a key-up at or after it,
//!   or none before the timer.

use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::{TimerRequest, TimerToken};
use crate::core::windows_actions::BindAction;

struct PendingTap {
    vk: u16,
    action: BindAction,
    /// The key-up must arrive strictly before this to fire.
    due: Instant,
}

struct PendingHold {
    vk: u16,
    action: BindAction,
    generation: u64,
    /// The term ends here; a key-up from this moment on no longer cancels.
    due: Instant,
    /// The key went up at or after `due`. The entry only waits for its
    /// timer and is no longer the key's pending Hold: it neither blocks a
    /// new arm nor answers the next key-up.
    released: bool,
}

/// What a key-up did to the Hold pending on its key.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum HoldRelease {
    /// No Hold was pending on the key.
    NotPending,
    /// Released before the term: the Hold is gone and its timer will find
    /// nothing.
    Cancelled,
    /// Released at or after the term: the Hold stays armed for its timer,
    /// which fires it if it has not run yet.
    Overdue,
}

struct PendingDoubleTap {
    vk: u16,
    action: BindAction,
    /// The second key-down must arrive strictly before this to fire.
    due: Instant,
}

#[derive(Default)]
pub(super) struct Gestures {
    taps: Vec<PendingTap>,
    holds: Vec<PendingHold>,
    double_taps: Vec<PendingDoubleTap>,
    /// Generation of the most recently armed Hold.
    last_generation: u64,
}

impl Gestures {
    pub(super) fn new() -> Self {
        Self::default()
    }

    // ---- Tap ----

    /// Arms a tap for `vk`. While a live tap is pending on the key, the
    /// first one wins: a second binding, or a press whose key-up was lost,
    /// does not replace it.
    pub(super) fn arm_tap(
        &mut self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        term: Duration,
        now: Instant,
    ) {
        if let Some(index) = self.taps.iter().position(|p| p.vk == vk.0) {
            if now < self.taps[index].due {
                return;
            }
            self.taps.swap_remove(index);
        }
        self.taps.push(PendingTap {
            vk: vk.0,
            action: action.clone(),
            due: now + term,
        });
    }

    /// Resolves the tap pending on `vk` at its key-up. Returns the action to
    /// fire if the key was released before the deadline; an expired entry is
    /// discarded as if it had never been there.
    pub(super) fn resolve_tap(&mut self, vk: VIRTUAL_KEY, now: Instant) -> Option<BindAction> {
        let index = self.taps.iter().position(|p| p.vk == vk.0)?;
        let tap = self.taps.swap_remove(index);
        (now < tap.due).then_some(tap.action)
    }

    // ---- Hold ----

    /// Arms a hold for `vk` and returns the timer to start, or `None` if a
    /// hold is already pending on the key (the first one wins). A released
    /// entry that still waits for its timer does not count.
    pub(super) fn arm_hold(
        &mut self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        term: Duration,
        now: Instant,
    ) -> Option<TimerRequest> {
        if self.pending_hold_index(vk).is_some() {
            return None;
        }
        self.last_generation = self.last_generation.wrapping_add(1);
        let due = now + term;
        self.holds.push(PendingHold {
            vk: vk.0,
            action: action.clone(),
            generation: self.last_generation,
            due,
            released: false,
        });
        Some(TimerRequest {
            token: TimerToken {
                vk: vk.0,
                generation: self.last_generation,
            },
            due,
        })
    }

    /// Resolves the hold pending on `vk` at its key-up: cancelled before
    /// its term, left armed for its timer from the term on.
    pub(super) fn release_hold(&mut self, vk: VIRTUAL_KEY, now: Instant) -> HoldRelease {
        let Some(index) = self.pending_hold_index(vk) else {
            return HoldRelease::NotPending;
        };
        if now < self.holds[index].due {
            self.holds.swap_remove(index);
            HoldRelease::Cancelled
        } else {
            self.holds[index].released = true;
            HoldRelease::Overdue
        }
    }

    /// Fires the hold that `token` was started for, if it is still pending
    /// or was released after its term.
    pub(super) fn fire_hold(&mut self, token: TimerToken) -> Option<BindAction> {
        let index = self
            .holds
            .iter()
            .position(|p| p.vk == token.vk && p.generation == token.generation)?;
        Some(self.holds.swap_remove(index).action)
    }

    /// Drops the hold that `token` was started for, without firing it,
    /// because the caller could not start its timer. Without this a Hold
    /// released after its term would wait forever for a timer that never
    /// runs.
    pub(super) fn forget_hold(&mut self, token: TimerToken) {
        self.holds
            .retain(|p| !(p.vk == token.vk && p.generation == token.generation));
    }

    /// The key's pending (not yet released) hold.
    fn pending_hold_index(&self, vk: VIRTUAL_KEY) -> Option<usize> {
        self.holds.iter().position(|p| p.vk == vk.0 && !p.released)
    }

    // ---- DoubleTap ----

    /// Handles a key-down for a DoubleTap binding. If a press of `vk` armed
    /// an entry less than its term ago, the entry is consumed and its action
    /// returned. Otherwise a fresh entry is armed with this binding's action.
    pub(super) fn double_tap(
        &mut self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        term: Duration,
        now: Instant,
    ) -> Option<BindAction> {
        if let Some(index) = self.double_taps.iter().position(|p| p.vk == vk.0) {
            let first = self.double_taps.swap_remove(index);
            if now < first.due {
                return Some(first.action);
            }
        }
        self.double_taps.push(PendingDoubleTap {
            vk: vk.0,
            action: action.clone(),
            due: now + term,
        });
        None
    }

    #[cfg(test)]
    fn tap_pending(&self, vk: VIRTUAL_KEY) -> bool {
        self.taps.iter().any(|p| p.vk == vk.0)
    }

    #[cfg(test)]
    fn hold_pending(&self, vk: VIRTUAL_KEY) -> bool {
        self.pending_hold_index(vk).is_some()
    }

    #[cfg(test)]
    fn double_tap_pending(&self, vk: VIRTUAL_KEY) -> bool {
        self.double_taps.iter().any(|p| p.vk == vk.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    const LONG: Duration = Duration::from_secs(10);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn arm_tap_inserts_then_resolve_removes() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        assert!(!state.tap_pending(VK_F13));
        state.arm_tap(VK_F13, &BindAction::DoNothing, LONG, t0);
        assert!(state.tap_pending(VK_F13));
        assert_eq!(
            state.resolve_tap(VK_F13, t0 + ms(10)),
            Some(BindAction::DoNothing)
        );
        assert!(!state.tap_pending(VK_F13));
    }

    #[test]
    fn resolve_tap_with_no_pending_returns_none() {
        let mut state = Gestures::new();
        assert_eq!(state.resolve_tap(VK_F13, Instant::now()), None);
    }

    #[test]
    fn tap_released_at_or_after_its_term_does_not_fire() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.arm_tap(VK_F13, &BindAction::DoNothing, ms(200), t0);
        assert_eq!(state.resolve_tap(VK_F13, t0 + ms(200)), None);
        // The expired entry is gone, not left behind.
        assert!(!state.tap_pending(VK_F13));
    }

    #[test]
    fn expired_tap_is_replaced_by_the_next_arm() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.arm_tap(VK_F13, &BindAction::DoNothing, ms(200), t0);
        // Key-up lost; the next press comes after the term.
        state.arm_tap(VK_F13, &BindAction::SwitchLanguage, ms(200), t0 + ms(300));
        assert_eq!(
            state.resolve_tap(VK_F13, t0 + ms(350)),
            Some(BindAction::SwitchLanguage)
        );
    }

    #[test]
    fn arm_hold_inserts_then_cancel_removes() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let timer = state.arm_hold(VK_F13, &BindAction::DoNothing, LONG, t0);
        assert_eq!(timer.map(|t| t.due), Some(t0 + LONG));
        assert!(state.hold_pending(VK_F13));
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(10)),
            HoldRelease::Cancelled
        );
        assert!(!state.hold_pending(VK_F13));
    }

    #[test]
    fn cancel_hold_with_no_pending_returns_false() {
        let mut state = Gestures::new();
        assert_eq!(
            state.release_hold(VK_F13, Instant::now()),
            HoldRelease::NotPending
        );
    }

    #[test]
    fn hold_timer_fires_its_own_entry_once() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let timer = state
            .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0)
            .unwrap();
        assert_eq!(
            state.fire_hold(timer.token),
            Some(BindAction::SwitchLanguage)
        );
        assert_eq!(state.fire_hold(timer.token), None);
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(250)),
            HoldRelease::NotPending
        );
    }

    #[test]
    fn stale_hold_timer_does_not_fire_a_newer_press() {
        // KS-11: press 1 is released early, press 2 arms a new hold, then
        // press 1's timer wakes up.
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let first = state
            .arm_hold(VK_F13, &BindAction::DoNothing, ms(200), t0)
            .unwrap();
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(80)),
            HoldRelease::Cancelled
        );
        let second = state
            .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0 + ms(150))
            .unwrap();
        assert_ne!(first.token, second.token);

        assert_eq!(state.fire_hold(first.token), None);
        assert!(state.hold_pending(VK_F13));
        assert_eq!(
            state.fire_hold(second.token),
            Some(BindAction::SwitchLanguage)
        );
    }

    #[test]
    fn hold_released_at_or_after_its_due_is_left_to_its_timer() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let timer = state
            .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0)
            .unwrap();
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(200)),
            HoldRelease::Overdue
        );
        // No longer the key's pending hold: a second key-up finds nothing.
        assert!(!state.hold_pending(VK_F13));
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(210)),
            HoldRelease::NotPending
        );
        // Its own timer still fires it, once.
        assert_eq!(
            state.fire_hold(timer.token),
            Some(BindAction::SwitchLanguage)
        );
        assert_eq!(state.fire_hold(timer.token), None);

        // One millisecond before the term the key-up cancels it.
        let timer = state
            .arm_hold(VK_F14, &BindAction::SwitchLanguage, ms(200), t0)
            .unwrap();
        assert_eq!(
            state.release_hold(VK_F14, t0 + ms(199)),
            HoldRelease::Cancelled
        );
        assert_eq!(state.fire_hold(timer.token), None);
    }

    #[test]
    fn hold_left_to_its_timer_does_not_block_a_new_arm() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let first = state
            .arm_hold(VK_F13, &BindAction::DoNothing, ms(200), t0)
            .unwrap();
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(250)),
            HoldRelease::Overdue
        );
        let second = state
            .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0 + ms(260))
            .expect("a released hold must not block the next press");
        assert_ne!(first.token, second.token);

        assert_eq!(state.fire_hold(first.token), Some(BindAction::DoNothing));
        assert!(state.hold_pending(VK_F13));
        assert_eq!(
            state.fire_hold(second.token),
            Some(BindAction::SwitchLanguage)
        );
    }

    #[test]
    fn forgotten_hold_never_fires_and_frees_its_key() {
        // The caller could not start the timer for the first press.
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let timer = state
            .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0)
            .unwrap();
        state.forget_hold(timer.token);
        assert!(!state.hold_pending(VK_F13));
        assert_eq!(state.fire_hold(timer.token), None);
        assert_eq!(
            state.release_hold(VK_F13, t0 + ms(300)),
            HoldRelease::NotPending
        );
        assert!(
            state
                .arm_hold(VK_F13, &BindAction::SwitchLanguage, ms(200), t0 + ms(400))
                .is_some()
        );
    }

    #[test]
    fn double_tap_first_press_arms_second_press_fires() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0),
            None
        );
        assert!(state.double_tap_pending(VK_F13));
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0 + ms(100)),
            Some(BindAction::DoNothing)
        );
        // Pending entry was consumed by second press.
        assert!(!state.double_tap_pending(VK_F13));
    }

    #[test]
    fn double_tap_third_press_arms_fresh_pending() {
        // Tap, tap (consumed), tap: the third is a "first tap" again.
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0);
        state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0 + ms(10));
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0 + ms(20)),
            None
        );
        assert!(state.double_tap_pending(VK_F13));
    }

    #[test]
    fn double_tap_fires_the_first_press_action() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.double_tap(VK_F13, &BindAction::SwitchLanguage, LONG, t0);
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, LONG, t0 + ms(10)),
            Some(BindAction::SwitchLanguage)
        );
    }

    #[test]
    fn double_tap_after_the_term_arms_again() {
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.double_tap(VK_F13, &BindAction::DoNothing, ms(200), t0);
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, ms(200), t0 + ms(200)),
            None
        );
        assert_eq!(
            state.double_tap(VK_F13, &BindAction::DoNothing, ms(200), t0 + ms(300)),
            Some(BindAction::DoNothing)
        );
    }

    #[test]
    fn tap_and_hold_pending_simultaneously_on_same_key() {
        // One key can have both Tap and Hold pending at the same time.
        // Resolution is independent.
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.arm_tap(VK_CAPITAL, &BindAction::DoNothing, LONG, t0);
        state.arm_hold(VK_CAPITAL, &BindAction::DoNothing, LONG, t0);
        assert!(state.tap_pending(VK_CAPITAL));
        assert!(state.hold_pending(VK_CAPITAL));

        // Resolve tap on key-up: only the tap entry should clear.
        state.resolve_tap(VK_CAPITAL, t0 + ms(10));
        assert!(!state.tap_pending(VK_CAPITAL));
        assert!(state.hold_pending(VK_CAPITAL));

        // Then cancel the hold.
        state.release_hold(VK_CAPITAL, t0 + ms(10));
        assert!(!state.hold_pending(VK_CAPITAL));
    }

    #[test]
    fn auto_repeat_keydown_does_not_stack() {
        // A key-down while a live entry is pending must not replace it or
        // start another timer: the first one wins.
        let t0 = Instant::now();
        let mut state = Gestures::new();
        state.arm_tap(VK_F13, &BindAction::DoNothing, LONG, t0);
        state.arm_tap(VK_F13, &BindAction::SwitchLanguage, LONG, t0 + ms(10));
        assert!(
            state
                .arm_hold(VK_F13, &BindAction::DoNothing, LONG, t0)
                .is_some()
        );
        assert!(
            state
                .arm_hold(VK_F13, &BindAction::SwitchLanguage, LONG, t0 + ms(10))
                .is_none()
        );
        assert_eq!(
            state.resolve_tap(VK_F13, t0 + ms(20)),
            Some(BindAction::DoNothing)
        );
        // After resolve, no second entry should remain.
        assert!(!state.tap_pending(VK_F13));
    }
}
