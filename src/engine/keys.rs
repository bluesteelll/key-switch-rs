//! Per-key state: which keys the engine believes are held, and which keys'
//! key-ups must be swallowed because their key-down was.
//!
//! Held keys are kept by the code the hook reported, which is sided for
//! modifiers (`VK_LSHIFT`), and the resync also writes the generic modifier
//! codes (`VK_SHIFT`). [`HeldKeys`] answers for a trigger key: `Shift` is
//! held if either Shift slot or the generic one is set, `LShift` only if
//! the left one is.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::env::Env;
use super::matching::HeldKeys;
use crate::data::key_spec::KeySpec;

/// Modifier virtual keys kept in sync with the OS key state. These are the
/// keys most prone to getting stuck after a focus transition into a
/// higher-integrity window (Task Manager / UAC / lock screen) that swallowed
/// the corresponding key-up event.
const MODIFIERS: [VIRTUAL_KEY; 11] = [
    VK_SHIFT,
    VK_LSHIFT,
    VK_RSHIFT,
    VK_CONTROL,
    VK_LCONTROL,
    VK_RCONTROL,
    VK_MENU,
    VK_LMENU,
    VK_RMENU,
    VK_LWIN,
    VK_RWIN,
];

/// Index into the 256-entry tables, or `None` for a code outside them.
fn slot(vk: u16) -> Option<usize> {
    let index = vk as usize;
    (index < 256).then_some(index)
}

/// Held and suppressed keys, one flag per virtual-key code.
pub(super) struct KeyState {
    /// Keys the engine believes are held. Set by key-down, cleared by
    /// key-up, and overwritten from the OS for the modifier codes. Its
    /// previous value is also the auto-repeat detector.
    active: [bool; 256],
    /// Keys whose key-down was suppressed: their key-up must be suppressed
    /// too, so an application never sees a release without a press. Each
    /// key is tracked independently.
    blocked: [bool; 256],
}

impl KeyState {
    pub(super) fn new() -> Self {
        Self {
            active: [false; 256],
            blocked: [false; 256],
        }
    }

    /// Reconciles the modifier flags with the OS key state. `skip` is the
    /// key of the event being delivered: its OS state is not updated until
    /// the hook returns, so it would read as stale.
    pub(super) fn sync_modifiers(&mut self, env: &dyn Env, skip: Option<VIRTUAL_KEY>) {
        for modifier in MODIFIERS {
            if Some(modifier) == skip {
                continue;
            }
            self.active[modifier.0 as usize] = env.os_key_down(modifier);
        }
    }

    /// Records a key-down. Returns true if the key was already held, which
    /// makes this event an auto-repeat.
    pub(super) fn press(&mut self, vk: VIRTUAL_KEY) -> bool {
        match slot(vk.0) {
            Some(index) => std::mem::replace(&mut self.active[index], true),
            None => false,
        }
    }

    /// Records a key-up.
    pub(super) fn release(&mut self, vk: VIRTUAL_KEY) {
        if let Some(index) = slot(vk.0) {
            self.active[index] = false;
        }
    }

    pub(super) fn mark_blocked(&mut self, vk: u16) {
        if let Some(index) = slot(vk) {
            self.blocked[index] = true;
        }
    }

    /// Clears the `vk` flag and reports whether it was set.
    pub(super) fn take_blocked(&mut self, vk: u16) -> bool {
        match slot(vk) {
            Some(index) => std::mem::replace(&mut self.blocked[index], false),
            None => false,
        }
    }

    pub(super) fn clear_all_blocked(&mut self) {
        self.blocked = [false; 256];
    }

    /// Whether the engine believes the key with this exact code is held.
    pub(super) fn is_active(&self, vk: VIRTUAL_KEY) -> bool {
        slot(vk.0).is_some_and(|index| self.active[index])
    }
}

impl HeldKeys for KeyState {
    fn is_down(&self, key: KeySpec) -> bool {
        match key {
            KeySpec::Key(vk) => self.is_active(vk),
            KeySpec::AnySide(m) => {
                self.is_active(m.left())
                    || self.is_active(m.right())
                    || m.generic().is_some_and(|generic| self.is_active(generic))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::data::key_spec::Modifier;

    #[test]
    fn any_side_is_down_for_either_side_or_the_generic_slot() {
        let shift = KeySpec::AnySide(Modifier::Shift);
        let win = KeySpec::AnySide(Modifier::Win);

        let mut state = KeyState::new();
        assert!(!state.is_down(shift));
        state.press(VK_RSHIFT);
        assert!(state.is_down(shift));
        state.release(VK_RSHIFT);
        state.press(VK_LSHIFT);
        assert!(state.is_down(shift));
        state.release(VK_LSHIFT);
        assert!(!state.is_down(shift));

        // The resync writes the generic code from the OS key state.
        state.sync_modifiers(&ShiftDown, None);
        state.release(VK_LSHIFT);
        assert!(state.is_active(VK_SHIFT));
        assert!(state.is_down(shift));

        state.press(VK_RWIN);
        assert!(state.is_down(win));
        assert!(!state.is_down(KeySpec::AnySide(Modifier::Ctrl)));
    }

    #[test]
    fn sided_key_is_down_only_on_its_side() {
        let mut state = KeyState::new();
        state.press(VK_RSHIFT);
        state.press(VK_RWIN);
        assert!(!state.is_down(KeySpec::Key(VK_LSHIFT)));
        assert!(state.is_down(KeySpec::Key(VK_RSHIFT)));
        assert!(!state.is_down(KeySpec::Key(VK_LWIN)));
        assert!(state.is_down(KeySpec::Key(VK_RWIN)));
        assert!(!state.is_down(KeySpec::Key(VK_A)));
    }

    #[test]
    fn second_press_is_a_repeat_until_release() {
        let mut state = KeyState::new();
        assert!(!state.press(VK_F13));
        assert!(state.press(VK_F13));
        state.release(VK_F13);
        assert!(!state.press(VK_F13));
    }

    #[test]
    fn out_of_range_codes_are_ignored() {
        let mut state = KeyState::new();
        assert!(!state.press(VIRTUAL_KEY(300)));
        assert!(!state.press(VIRTUAL_KEY(300)));
        state.release(VIRTUAL_KEY(300));
        state.mark_blocked(300);
        assert!(!state.take_blocked(300));
        assert!(!state.is_active(VIRTUAL_KEY(300)));
        assert!(!state.is_down(KeySpec::Key(VIRTUAL_KEY(300))));
    }

    #[test]
    fn blocked_flags_are_independent() {
        let mut state = KeyState::new();
        state.mark_blocked(20); // VK_CAPITAL
        state.mark_blocked(16); // VK_SHIFT
        state.mark_blocked(200);

        // Releasing VK_CAPITAL must not touch VK_SHIFT or 200.
        assert!(state.take_blocked(20));
        assert!(!state.take_blocked(20)); // second take is a no-op
        assert!(state.take_blocked(16));
        assert!(state.take_blocked(200));
    }

    #[test]
    fn clear_all_resets_every_flag() {
        let mut state = KeyState::new();
        state.mark_blocked(20);
        state.mark_blocked(200);
        state.clear_all_blocked();
        assert!(!state.take_blocked(20));
        assert!(!state.take_blocked(200));
    }

    /// Reports only LShift and the generic Shift as down.
    struct ShiftDown;

    impl Env for ShiftDown {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            vk == VK_LSHIFT || vk == VK_SHIFT
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn sync_overwrites_modifiers_except_the_skipped_key() {
        let mut state = KeyState::new();
        state.press(VK_LCONTROL); // the OS says it is up
        state.press(VK_RMENU); // the event being delivered
        state.press(VK_A); // not a modifier: never synced

        state.sync_modifiers(&ShiftDown, Some(VK_RMENU));

        assert!(state.is_active(VK_LSHIFT));
        assert!(state.is_active(VK_SHIFT));
        assert!(!state.is_active(VK_LCONTROL));
        assert!(state.is_active(VK_RMENU));
        assert!(state.is_active(VK_A));

        state.sync_modifiers(&ShiftDown, None);
        assert!(!state.is_active(VK_RMENU));
    }
}
