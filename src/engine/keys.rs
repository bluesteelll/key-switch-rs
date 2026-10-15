//! Per-key state: which keys the engine believes are held, and whether each
//! key's latest key-down was swallowed or passed, which its auto-repeats and
//! its key-up follow.
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
/// the corresponding key-up event. A modifier whose key-down the engine
/// swallowed is left out of the sync: Windows never saw it go down (KS-14).
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

/// What became of a key's latest key-down, as far as the engine knows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LastDown {
    /// Nothing on record: the key is up, or the engine never saw its
    /// key-down (a modifier the resync found held).
    Unknown,
    /// The key-down reached the OS.
    Passed,
    /// The key-down was suppressed.
    Suppressed,
    /// The key-down was suppressed, then a foreground change forgot it: its
    /// key-up is no longer due to be suppressed. Windows still never saw the
    /// key go down, so its auto-repeats stay suppressed and the resync
    /// leaves it alone.
    Forgotten,
}

/// Held and suppressed keys, one entry per virtual-key code.
pub(super) struct KeyState {
    /// Keys the engine believes are held. Set by key-down, cleared by
    /// key-up, and overwritten from the OS for the modifier codes whose
    /// key-down the engine did not swallow. Its previous value is also the
    /// auto-repeat detector.
    active: [bool; 256],
    /// What became of each key's latest key-down. A suppressed key-down
    /// makes its auto-repeats and its key-up suppressed too, so an
    /// application never sees a repeat or a release without a press; a
    /// passed one makes them pass. Each key is tracked independently.
    last_down: [LastDown; 256],
}

impl KeyState {
    pub(super) fn new() -> Self {
        Self {
            active: [false; 256],
            last_down: [LastDown::Unknown; 256],
        }
    }

    /// Reconciles the modifier flags with the OS key state. `skip` is the
    /// key of the event being delivered: its OS state is not updated until
    /// the hook returns, so it would read as stale. A modifier whose
    /// key-down was suppressed, even one a foreground change forgot, is left
    /// alone as well: Windows never saw it go down, so its key state says up
    /// while the key is held, and overwriting the flag would also turn the
    /// key's next auto-repeat into a fresh press (KS-14).
    pub(super) fn sync_modifiers(&mut self, env: &dyn Env, skip: Option<VIRTUAL_KEY>) {
        for modifier in MODIFIERS {
            let index = modifier.0 as usize;
            if Some(modifier) == skip || self.key_down_swallowed(index) {
                continue;
            }
            self.active[index] = env.os_key_down(modifier);
        }
    }

    /// Whether the latest key-down at `index` was suppressed, forgotten by a
    /// foreground change or not.
    fn key_down_swallowed(&self, index: usize) -> bool {
        matches!(
            self.last_down[index],
            LastDown::Suppressed | LastDown::Forgotten
        )
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

    /// Records that the key-down of `vk` was suppressed.
    pub(super) fn mark_blocked(&mut self, vk: u16) {
        if let Some(index) = slot(vk) {
            self.last_down[index] = LastDown::Suppressed;
        }
    }

    /// Records that the key-down of `vk` reached the OS.
    pub(super) fn mark_passed(&mut self, vk: u16) {
        if let Some(index) = slot(vk) {
            self.last_down[index] = LastDown::Passed;
        }
    }

    /// Whether the key-down of `vk` is on record as suppressed, leaving the
    /// record in place. A key-down that a foreground change forgot counts
    /// too: Windows never saw it, so the key's auto-repeats must not reach
    /// Windows either.
    pub(super) fn is_blocked(&self, vk: u16) -> bool {
        slot(vk).is_some_and(|index| self.key_down_swallowed(index))
    }

    /// Whether the key-down of `vk` is on record as passed.
    pub(super) fn is_passed(&self, vk: u16) -> bool {
        slot(vk).is_some_and(|index| self.last_down[index] == LastDown::Passed)
    }

    /// Forgets the key-down of `vk` and reports whether it was suppressed.
    pub(super) fn take_blocked(&mut self, vk: u16) -> bool {
        match slot(vk) {
            Some(index) => {
                std::mem::replace(&mut self.last_down[index], LastDown::Unknown)
                    == LastDown::Suppressed
            }
            None => false,
        }
    }

    /// Forgets every suppressed key-down, so that no key-up stays due to be
    /// suppressed. The record still says the key-down never reached the OS,
    /// so the key's auto-repeats stay suppressed. A passed key-down stays on
    /// record: the OS saw it, so its repeats and key-up must still pass.
    pub(super) fn forget_blocked(&mut self) {
        for record in &mut self.last_down {
            if *record == LastDown::Suppressed {
                *record = LastDown::Forgotten;
            }
        }
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
    fn is_blocked_reads_without_clearing() {
        let mut state = KeyState::new();
        assert!(!state.is_blocked(20));
        state.mark_blocked(20);
        assert!(state.is_blocked(20));
        assert!(state.is_blocked(20));
        assert!(state.take_blocked(20));
        assert!(!state.is_blocked(20));
        assert!(!state.is_blocked(300));
    }

    #[test]
    fn forget_blocked_resets_every_flag() {
        let mut state = KeyState::new();
        state.mark_blocked(20);
        state.mark_blocked(200);
        state.forget_blocked();
        assert!(!state.take_blocked(20));
        assert!(!state.take_blocked(200));
    }

    #[test]
    fn forget_blocked_keeps_passed_key_downs() {
        let mut state = KeyState::new();
        state.mark_blocked(20);
        state.mark_passed(65);
        state.forget_blocked();
        assert!(
            state.is_blocked(20),
            "a forgotten key-down was still swallowed, so its repeats are too"
        );
        assert!(!state.is_passed(20));
        assert!(state.is_passed(65));
    }

    #[test]
    fn latest_key_down_replaces_the_record_and_take_forgets_it() {
        let mut state = KeyState::new();
        assert!(!state.is_passed(20));
        state.mark_blocked(20);
        state.mark_passed(20);
        assert!(state.is_passed(20));
        assert!(!state.is_blocked(20));
        state.mark_blocked(20);
        assert!(state.is_blocked(20));
        assert!(!state.is_passed(20));

        state.mark_passed(20);
        assert!(!state.take_blocked(20));
        assert!(!state.is_passed(20));

        state.mark_passed(300);
        assert!(!state.is_passed(300));
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

    /// Reports every key as up.
    struct NothingDown;

    impl Env for NothingDown {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn os_key_down(&self, _vk: VIRTUAL_KEY) -> bool {
            false
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn resync_leaves_a_modifier_whose_key_down_was_suppressed() {
        // KS-14: Windows never saw the key-down, so its key state says up.
        let mut state = KeyState::new();
        state.press(VK_LSHIFT);
        state.mark_blocked(VK_LSHIFT.0);

        state.sync_modifiers(&NothingDown, Some(VK_A));

        assert!(
            state.is_active(VK_LSHIFT),
            "a modifier whose key-down was swallowed stays held through the resync"
        );
        assert!(
            state.press(VK_LSHIFT),
            "its next key-down is still an auto-repeat"
        );
    }

    #[test]
    fn resync_still_repairs_a_modifier_whose_key_down_passed() {
        // Its key-up went to a window the hook does not see.
        let mut state = KeyState::new();
        state.press(VK_LCONTROL);
        state.mark_passed(VK_LCONTROL.0);

        state.sync_modifiers(&NothingDown, Some(VK_A));

        assert!(!state.is_active(VK_LCONTROL));
    }

    #[test]
    fn forgotten_modifier_is_left_alone_by_the_resync() {
        let mut state = KeyState::new();
        state.press(VK_LSHIFT);
        state.mark_blocked(VK_LSHIFT.0);
        state.forget_blocked();

        state.sync_modifiers(&NothingDown, None);

        assert!(
            state.is_active(VK_LSHIFT),
            "a focus change does not make Windows see the key-down"
        );
        assert!(!state.is_passed(VK_LSHIFT.0));
        assert!(
            !state.take_blocked(VK_LSHIFT.0),
            "its key-up is no longer due to be suppressed"
        );
    }
}
