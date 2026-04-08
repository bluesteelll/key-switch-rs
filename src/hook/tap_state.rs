//! Deferred-gesture state for `Trigger::Tap`, `Trigger::Hold`, `Trigger::DoubleTap`.
//!
//! Each gesture has its own `HashMap` of pending entries keyed by VK. The
//! resolution protocol is Mutex-serialized: whichever side (timer thread or
//! key-event handler) acquires the lock first and successfully `remove()`s
//! the entry gets to fire its action. The other side observes `None` and
//! no-ops. No atomic gate needed — `Mutex` provides the linearization.
//!
//! All three maps are owned by `Arc<Mutex<...>>` so spawned timer threads
//! can share them without needing a `'static` reference into `KeyboardHook`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;

#[derive(Default)]
pub struct TapState {
    /// `Tap` triggers: action stored here until either key-up arrives in
    /// time (→ fire) or the timer GC's the entry (→ tap window expired,
    /// no fire — this was actually a hold).
    pending_taps: Arc<Mutex<HashMap<u16, BindAction>>>,
    /// `Hold` triggers: action stored here until either the timer fires
    /// (→ key held long enough) or key-up cancels (→ user released too
    /// early).
    pending_holds: Arc<Mutex<HashMap<u16, BindAction>>>,
    /// `DoubleTap` triggers: first key-down arms an entry, second key-down
    /// within term consumes it (→ fire). Otherwise the timer GC's it.
    pending_double_taps: Arc<Mutex<HashMap<u16, BindAction>>>,
}

impl TapState {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- Tap ----

    /// Arm a tap gesture. The action will fire if `resolve_tap_on_keyup` is
    /// called with this VK before `term_ms` elapses; otherwise the timer
    /// GC's the entry silently.
    pub fn arm_tap(&self, vk: VIRTUAL_KEY, action: BindAction, term_ms: u64) {
        {
            let mut map = self.pending_taps.lock().expect("tap mutex poisoned");
            // Auto-repeat protection: don't stack pendings on the same key
            // if user holds it down and OS re-fires key-down. First wins.
            if map.contains_key(&vk.0) {
                return;
            }
            map.insert(vk.0, action);
        }

        // Timer thread is purely GC — if the window expires without a
        // matching key-up, drop the entry so the next press starts fresh.
        let map_arc = Arc::clone(&self.pending_taps);
        let vk_u16 = vk.0;
        thread::Builder::new()
            .name("tap-gc".into())
            .spawn(move || {
                thread::sleep(Duration::from_millis(term_ms));
                map_arc.lock().expect("tap mutex poisoned").remove(&vk_u16);
            })
            .expect("spawn tap-gc thread");
    }

    /// Called on key-up. Returns `true` if there was a pending tap (and the
    /// caller should suppress the key-up itself); the tap action, if any,
    /// is executed inline before returning. `false` means this key wasn't
    /// part of any pending tap gesture.
    pub fn resolve_tap_on_keyup(&self, vk: VIRTUAL_KEY) -> bool {
        let action = self
            .pending_taps
            .lock()
            .expect("tap mutex poisoned")
            .remove(&vk.0);

        match action {
            Some(action) => {
                action.execute();
                true
            }
            None => false,
        }
    }

    // ---- Hold ----

    /// Arm a hold gesture. The action fires from the timer thread after
    /// `term_ms` unless `cancel_hold_on_keyup` removes the entry first.
    pub fn arm_hold(&self, vk: VIRTUAL_KEY, action: BindAction, term_ms: u64) {
        {
            let mut map = self.pending_holds.lock().expect("hold mutex poisoned");
            if map.contains_key(&vk.0) {
                return;
            }
            map.insert(vk.0, action);
        }

        let map_arc = Arc::clone(&self.pending_holds);
        let vk_u16 = vk.0;
        thread::Builder::new()
            .name("hold-timer".into())
            .spawn(move || {
                thread::sleep(Duration::from_millis(term_ms));
                // Lock + remove + fire under the same critical section so
                // we don't race with cancel_hold_on_keyup.
                let action = map_arc
                    .lock()
                    .expect("hold mutex poisoned")
                    .remove(&vk_u16);
                if let Some(action) = action {
                    action.execute();
                }
            })
            .expect("spawn hold-timer thread");
    }

    /// Called on key-up. Returns `true` if there was a pending hold (caller
    /// suppresses the key-up); the gesture is simply cancelled — nothing
    /// fires because the key was released before the hold window elapsed.
    pub fn cancel_hold_on_keyup(&self, vk: VIRTUAL_KEY) -> bool {
        self.pending_holds
            .lock()
            .expect("hold mutex poisoned")
            .remove(&vk.0)
            .is_some()
    }

    // ---- DoubleTap ----

    /// Called on key-down. If there is a pending double-tap entry for this
    /// VK (i.e. the user pressed this key recently enough), the entry is
    /// consumed and the action fires. Otherwise a fresh entry is armed and
    /// the timer will GC it after `term_ms`. Returns `true` either way —
    /// the caller should suppress the key event from the foreground.
    pub fn handle_double_tap(&self, vk: VIRTUAL_KEY, action: BindAction, term_ms: u64) -> bool {
        {
            let mut map = self
                .pending_double_taps
                .lock()
                .expect("double-tap mutex poisoned");
            if let Some(existing_action) = map.remove(&vk.0) {
                // Second press inside window → fire double action.
                existing_action.execute();
                return true;
            }
            // First press: arm.
            map.insert(vk.0, action);
        }

        // GC thread — drop the entry if no second press arrives in time.
        let map_arc = Arc::clone(&self.pending_double_taps);
        let vk_u16 = vk.0;
        thread::Builder::new()
            .name("double-tap-gc".into())
            .spawn(move || {
                thread::sleep(Duration::from_millis(term_ms));
                map_arc
                    .lock()
                    .expect("double-tap mutex poisoned")
                    .remove(&vk_u16);
            })
            .expect("spawn double-tap-gc thread");

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    fn tap_pending(state: &TapState, vk: VIRTUAL_KEY) -> bool {
        state
            .pending_taps
            .lock()
            .unwrap()
            .contains_key(&vk.0)
    }
    fn hold_pending(state: &TapState, vk: VIRTUAL_KEY) -> bool {
        state
            .pending_holds
            .lock()
            .unwrap()
            .contains_key(&vk.0)
    }
    fn double_tap_pending(state: &TapState, vk: VIRTUAL_KEY) -> bool {
        state
            .pending_double_taps
            .lock()
            .unwrap()
            .contains_key(&vk.0)
    }

    #[test]
    fn arm_tap_inserts_then_resolve_removes() {
        let state = TapState::new();
        assert!(!tap_pending(&state, VK_F13));
        state.arm_tap(VK_F13, BindAction::DoNothing, 10_000);
        assert!(tap_pending(&state, VK_F13));
        assert!(state.resolve_tap_on_keyup(VK_F13));
        assert!(!tap_pending(&state, VK_F13));
    }

    #[test]
    fn resolve_tap_with_no_pending_returns_false() {
        let state = TapState::new();
        assert!(!state.resolve_tap_on_keyup(VK_F13));
    }

    #[test]
    fn arm_hold_inserts_then_cancel_removes() {
        let state = TapState::new();
        state.arm_hold(VK_F13, BindAction::DoNothing, 10_000);
        assert!(hold_pending(&state, VK_F13));
        assert!(state.cancel_hold_on_keyup(VK_F13));
        assert!(!hold_pending(&state, VK_F13));
    }

    #[test]
    fn cancel_hold_with_no_pending_returns_false() {
        let state = TapState::new();
        assert!(!state.cancel_hold_on_keyup(VK_F13));
    }

    #[test]
    fn double_tap_first_press_arms_second_press_fires() {
        let state = TapState::new();
        assert!(state.handle_double_tap(VK_F13, BindAction::DoNothing, 10_000));
        assert!(double_tap_pending(&state, VK_F13));
        assert!(state.handle_double_tap(VK_F13, BindAction::DoNothing, 10_000));
        // Pending entry was consumed by second press.
        assert!(!double_tap_pending(&state, VK_F13));
    }

    #[test]
    fn double_tap_third_press_arms_fresh_pending() {
        // Tap, tap (consumed), tap → third is a "first tap" again.
        let state = TapState::new();
        state.handle_double_tap(VK_F13, BindAction::DoNothing, 10_000);
        state.handle_double_tap(VK_F13, BindAction::DoNothing, 10_000);
        state.handle_double_tap(VK_F13, BindAction::DoNothing, 10_000);
        assert!(double_tap_pending(&state, VK_F13));
    }

    #[test]
    fn tap_and_hold_pending_simultaneously_on_same_key() {
        // The whole point of B-style: one key can have both Tap and Hold
        // pending at the same time. Resolution is independent.
        let state = TapState::new();
        state.arm_tap(VK_CAPITAL, BindAction::DoNothing, 10_000);
        state.arm_hold(VK_CAPITAL, BindAction::DoNothing, 10_000);
        assert!(tap_pending(&state, VK_CAPITAL));
        assert!(hold_pending(&state, VK_CAPITAL));

        // Resolve tap on key-up — only the tap entry should clear.
        state.resolve_tap_on_keyup(VK_CAPITAL);
        assert!(!tap_pending(&state, VK_CAPITAL));
        assert!(hold_pending(&state, VK_CAPITAL));

        // Then cancel the hold.
        state.cancel_hold_on_keyup(VK_CAPITAL);
        assert!(!hold_pending(&state, VK_CAPITAL));
    }

    #[test]
    fn auto_repeat_keydown_does_not_stack() {
        // Holding the key down causes OS to fire repeated key-downs.
        // arm_tap must NOT replace the existing entry or spawn extra timer
        // threads — first one wins.
        let state = TapState::new();
        state.arm_tap(VK_F13, BindAction::DoNothing, 10_000);
        state.arm_tap(VK_F13, BindAction::SwitchLanguage, 10_000); // would-be replacement
        assert!(tap_pending(&state, VK_F13));
        // After resolve, no second entry should remain.
        state.resolve_tap_on_keyup(VK_F13);
        assert!(!tap_pending(&state, VK_F13));
    }
}
