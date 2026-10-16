//! Per-key state: which keys the engine believes are held, and whether each
//! key's latest key-down was swallowed or passed, which its auto-repeats and
//! its key-up follow.
//!
//! Held keys are kept by the code the hook reported, which is sided for
//! modifiers (`VK_LSHIFT`), and the resync also writes the generic modifier
//! codes (`VK_SHIFT`). [`HeldKeys`] answers for a trigger key: `Shift` is
//! held if either Shift slot or the generic one is set, `LShift` only if
//! the left one is. For an auto-blocker it also visits every code held.
//!
//! A reset (`swch off`, `swch on`) rebuilds the table from the OS key state:
//! [`KeyState::reset_from_os`].

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

/// The Alt and Win codes, the generic `VK_MENU` slot the resync writes
/// included. A lone Win or Alt press that Windows sees opens the Start menu
/// or activates the menu bar on its release (KS-12).
const MENU_KEYS: [VIRTUAL_KEY; 5] = [VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN];

/// Mouse buttons. They have virtual-key codes, and the OS key state reports
/// them, but the keyboard hook never sees them, so
/// [`KeyState::reset_from_os`] leaves them out.
const MOUSE_BUTTONS: [VIRTUAL_KEY; 5] =
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2];

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

    /// Starts over from the OS key state, after remapping was switched off
    /// or on. Every keyboard code the OS reports held is held and on record
    /// as passed: the engine saw no event while remapping was off, Windows
    /// saw every key-down, so the key's auto-repeats and its key-up must
    /// reach Windows too. Every other key is up with nothing on record.
    ///
    /// Mouse buttons are left out: a seeded button would stay held for
    /// good, and an auto-blocker would then never see its hotkey alone. A
    /// key pressed just before the reset may not show in the OS state yet;
    /// its first auto-repeat then reads as a fresh press.
    pub(super) fn reset_from_os(&mut self, env: &dyn Env) {
        *self = Self::new();
        // 1..=254 is the documented virtual-key range, and every code is
        // below 256, so its slot is its index.
        for code in 1..=254u16 {
            let vk = VIRTUAL_KEY(code);
            if MOUSE_BUTTONS.contains(&vk) || !env.os_key_down(vk) {
                continue;
            }
            let index = usize::from(code);
            self.active[index] = true;
            self.last_down[index] = LastDown::Passed;
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

    /// Records that a completed chord or sequence swallows the key-up of
    /// `vk`, if the key is held and its key-down did not reach the OS. A key
    /// already released has no key-up left to swallow (KS-30), and one whose
    /// key-down passed must keep its key-up (KS-32).
    pub(super) fn mark_completed(&mut self, vk: u16) {
        if let Some(index) = slot(vk)
            && self.active[index]
            && self.last_down[index] != LastDown::Passed
        {
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

    /// Whether Windows has an Alt or Win key down: one is held and its
    /// key-down was not swallowed. The resync keeps every such key in step
    /// with the OS key state, so this asks the OS nothing. A key held with
    /// no record counts: the resync found it down in Windows. Read after the
    /// event's own record update, an Alt or Win key-down being swallowed
    /// right now does not count.
    pub(super) fn alt_or_win_down_in_windows(&self) -> bool {
        MENU_KEYS.iter().any(|key| {
            let index = usize::from(key.0);
            self.active[index] && !self.key_down_swallowed(index)
        })
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

    fn all_held(&self, mut accept: impl FnMut(VIRTUAL_KEY) -> bool) -> bool {
        // A slot's index is the 8-bit code of its key (`slot`), so every
        // code built here is one the hook or the resync stored, below 256.
        (0..256u16)
            .filter(|&code| self.active[usize::from(code)])
            .all(|code| accept(VIRTUAL_KEY(code)))
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
    fn all_held_visits_every_held_code() {
        let mut state = KeyState::new();
        assert!(state.all_held(|_| false), "nothing is held");

        state.press(VK_LSHIFT);
        state.press(VK_A);
        // The resync writes the generic code from the OS key state.
        state.sync_modifiers(&ShiftDown, None);
        assert!(state.all_held(|vk| [VK_LSHIFT, VK_SHIFT, VK_A].contains(&vk)));
        assert!(!state.all_held(|vk| vk != VK_A));
        assert!(
            !state.all_held(|vk| vk != VK_SHIFT),
            "the generic slot is visited too"
        );
    }

    #[test]
    fn all_held_visits_the_highest_code_the_table_holds() {
        let mut state = KeyState::new();
        state.press(VIRTUAL_KEY(255));
        assert!(
            !state.all_held(|vk| vk != VIRTUAL_KEY(255)),
            "the last slot (code 255) is visited by all_held"
        );
        assert!(
            state.all_held(|vk| vk == VIRTUAL_KEY(255)),
            "code 255 is the only key held"
        );
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
    fn completion_marks_only_held_keys_whose_key_down_did_not_pass() {
        let mut state = KeyState::new();
        state.press(VK_F14);
        state.mark_passed(VK_F14.0);
        state.press(VK_J);
        state.mark_blocked(VK_J.0);
        state.forget_blocked();

        state.mark_completed(VK_F14.0);
        state.mark_completed(VK_J.0);
        state.mark_completed(VK_K.0);
        state.mark_completed(300);

        assert!(
            state.is_passed(VK_F14.0),
            "a held key whose key-down passed keeps its key-up"
        );
        assert!(
            state.take_blocked(VK_J.0),
            "a held key whose swallowed key-down a focus change forgot has its key-up swallowed again"
        );
        assert!(
            !state.is_blocked(VK_K.0),
            "a key already released is not marked"
        );
        assert!(!state.is_blocked(300));
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

    /// Reports A, LShift with the generic Shift, and two mouse buttons as
    /// down.
    struct KeysAndButtonsDown;

    impl Env for KeysAndButtonsDown {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            [VK_A, VK_LSHIFT, VK_SHIFT, VK_LBUTTON, VK_XBUTTON2].contains(&vk)
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons() {
        let mut state = KeyState::new();
        state.press(VK_J);
        state.mark_blocked(VK_J.0);
        state.press(VK_F13);
        state.mark_passed(VK_F13.0);

        state.reset_from_os(&KeysAndButtonsDown);

        for key in [VK_A, VK_LSHIFT, VK_SHIFT] {
            assert!(state.is_active(key), "{key:?} is held in Windows");
            assert!(
                state.is_passed(key.0),
                "{key:?}: Windows saw its key-down, so its repeats and key-up pass"
            );
        }
        for button in [VK_LBUTTON, VK_XBUTTON2] {
            assert!(
                !state.is_active(button),
                "{button:?}: the keyboard hook never reports a mouse button's release"
            );
        }
        assert!(!state.is_active(VK_J));
        assert!(
            !state.is_blocked(VK_J.0),
            "a record from before the reset is gone"
        );
        assert!(!state.is_active(VK_F13));
        assert!(!state.is_passed(VK_F13.0));
        assert!(
            state.press(VK_A),
            "the next key-down of a key held through the reset is an auto-repeat"
        );
    }

    /// Reports exactly the listed codes as down.
    struct Down(&'static [VIRTUAL_KEY]);

    impl Env for Down {
        fn now(&self) -> Instant {
            Instant::now()
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            self.0.contains(&vk)
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed() {
        let mut state = KeyState::new();
        assert!(!state.alt_or_win_down_in_windows(), "nothing is held");

        state.press(VK_LSHIFT);
        state.mark_passed(VK_LSHIFT.0);
        assert!(
            !state.alt_or_win_down_in_windows(),
            "Shift is neither Alt nor Win"
        );

        state.press(VK_LMENU);
        state.mark_passed(VK_LMENU.0);
        assert!(state.alt_or_win_down_in_windows(), "a passed Alt counts");
        state.release(VK_LMENU);
        state.take_blocked(VK_LMENU.0);
        assert!(
            !state.alt_or_win_down_in_windows(),
            "a released Alt does not"
        );

        state.press(VK_LWIN);
        state.mark_blocked(VK_LWIN.0);
        assert!(
            !state.alt_or_win_down_in_windows(),
            "a Win whose key-down was swallowed does not count"
        );
        state.forget_blocked();
        assert!(
            !state.alt_or_win_down_in_windows(),
            "nor does one a foreground change forgot"
        );

        // The resync skips the swallowed LWin and writes the rest.
        state.sync_modifiers(&Down(&[VK_RWIN]), None);
        assert!(
            state.alt_or_win_down_in_windows(),
            "a Win the resync found held with no record counts"
        );
        state.sync_modifiers(&Down(&[VK_MENU]), None);
        assert!(
            state.alt_or_win_down_in_windows(),
            "the generic Alt slot the resync writes counts"
        );
        state.sync_modifiers(&Down(&[]), None);
        assert!(!state.alt_or_win_down_in_windows());
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
