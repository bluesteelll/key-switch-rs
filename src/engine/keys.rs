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
//! A reset rebuilds the table: from the OS key state when remapping is
//! switched off or on ([`KeyState::resync_from_os`]), and with every key
//! released when keyboard input leaves this desktop
//! ([`KeyState::release_all`], roadmap item 2.3). A swallowed key-down's
//! record survives both, and every foreground change, so the key's
//! auto-repeats and key-up stay swallowed (rule L); only the key's own
//! next key-down or its key-up replaces it.

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
/// [`KeyState::resync_from_os`] leaves them out.
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
    /// The engine never saw this press's key-down, and it suppressed one of
    /// its auto-repeats (roadmap item 2.5). Its key-up is suppressed like a
    /// `Suppressed` one's, unless Windows has the key down, and a reset
    /// keeps it the same way. Otherwise it is `Unknown`: Windows may have
    /// the key down, so the modifier resync reads it, it counts for the menu
    /// mask, and its later repeats are decided by the bindings.
    RepeatSuppressed,
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
    /// key-down was suppressed is left alone as well: Windows never saw it
    /// go down, so its key state says up while the key is held, and
    /// overwriting the flag would also turn the key's next auto-repeat into
    /// a fresh press (KS-14).
    pub(super) fn sync_modifiers(&mut self, env: &dyn Env, skip: Option<VIRTUAL_KEY>) {
        for modifier in MODIFIERS {
            let index = modifier.0 as usize;
            if Some(modifier) == skip || self.key_down_swallowed(index) {
                continue;
            }
            self.active[index] = env.os_key_down(modifier);
        }
    }

    /// Rebuilds the table from the OS key state, after remapping was
    /// switched off or on, or after the keyboard hook was reinstalled
    /// (roadmap item 2.4). Every keyboard code the OS reports held is held
    /// and on record as passed: the engine saw no event while remapping was
    /// off or the hook was gone, Windows saw every key-down, so the key's
    /// auto-repeats and its key-up must reach Windows too. Every other key is up. Its record is
    /// dropped, except a swallowed key-down's or a swallowed auto-repeat's
    /// (roadmap item 2.5): Windows is no witness for a key it never saw go
    /// down (KS-14), so its key-up, if one is still due, stays swallowed,
    /// unless Windows has the key down by then (the witness of
    /// `Engine::on_key_up`).
    ///
    /// Mouse buttons are left out: a seeded button would stay held for
    /// good, and an auto-blocker would then never see its hotkey alone. A
    /// key pressed just before the reset may not show in the OS state yet;
    /// its first auto-repeat then reads as a fresh press.
    pub(super) fn resync_from_os(&mut self, env: &dyn Env) {
        // Every code is below 256, so its slot is its index. 1..=254 is the
        // documented virtual-key range: the OS is not asked about 0 or 255.
        for code in 0..=255u16 {
            let vk = VIRTUAL_KEY(code);
            let index = usize::from(code);
            let held =
                (1..=254).contains(&code) && !MOUSE_BUTTONS.contains(&vk) && env.os_key_down(vk);
            self.active[index] = held;
            if held {
                self.last_down[index] = LastDown::Passed;
            } else if !matches!(
                self.last_down[index],
                LastDown::Suppressed | LastDown::RepeatSuppressed
            ) {
                self.last_down[index] = LastDown::Unknown;
            }
        }
    }

    /// Releases every key and keeps every record, without asking the OS:
    /// keyboard input left this desktop, so any key held may lose its
    /// key-up there (roadmap item 2.3). The next key-down of each key is a
    /// first press, and a key-up that does come later passes or is
    /// swallowed as its key-down was. Calling it twice is calling it once.
    pub(super) fn release_all(&mut self) {
        self.active = [false; 256];
    }

    /// Whether the latest key-down at `index` was suppressed.
    fn key_down_swallowed(&self, index: usize) -> bool {
        self.last_down[index] == LastDown::Suppressed
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

    /// Records that an auto-repeat of `vk`, a key whose key-down the engine
    /// never saw, was suppressed (roadmap item 2.5): its key-up is then
    /// suppressed too, unless Windows has the key down.
    pub(super) fn mark_repeat_suppressed(&mut self, vk: u16) {
        if let Some(index) = slot(vk) {
            self.last_down[index] = LastDown::RepeatSuppressed;
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
    /// record in place.
    pub(super) fn is_blocked(&self, vk: u16) -> bool {
        slot(vk).is_some_and(|index| self.key_down_swallowed(index))
    }

    /// Whether the key-down of `vk` is on record as passed.
    pub(super) fn is_passed(&self, vk: u16) -> bool {
        slot(vk).is_some_and(|index| self.last_down[index] == LastDown::Passed)
    }

    /// Forgets the key-down of `vk` and reports whether it was suppressed,
    /// or an auto-repeat of a press whose key-down the engine never saw was.
    pub(super) fn take_blocked(&mut self, vk: u16) -> bool {
        match slot(vk) {
            Some(index) => matches!(
                std::mem::replace(&mut self.last_down[index], LastDown::Unknown),
                LastDown::Suppressed | LastDown::RepeatSuppressed
            ),
            None => false,
        }
    }

    /// Whether the engine believes the key with this exact code is held.
    pub(super) fn is_active(&self, vk: VIRTUAL_KEY) -> bool {
        slot(vk.0).is_some_and(|index| self.active[index])
    }

    /// Whether Windows has an Alt or Win key down: one is held and its
    /// key-down was not swallowed. The resync keeps every such key in step
    /// with the OS key state, so this asks the OS nothing. A key held with
    /// no record counts: the resync found it down in Windows; so does one
    /// whose swallowed auto-repeat is on record. Read after the
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
    fn resync_from_os_keeps_swallowed_records_frees_their_keys_and_takes_os_keys_as_passed() {
        // Roadmap item 2.3: `swch off` and `swch on` keep a swallowed
        // key-down's record, so its key-up stays swallowed (rule L).
        let mut state = KeyState::new();
        state.press(VK_CAPITAL);
        state.mark_blocked(VK_CAPITAL.0);
        state.press(VK_F13);
        state.mark_passed(VK_F13.0);
        state.mark_blocked(200);

        state.resync_from_os(&Down(&[VK_A]));

        assert!(!state.is_active(VK_CAPITAL), "the OS has CapsLock up");
        assert!(
            state.is_blocked(VK_CAPITAL.0),
            "Windows never saw CapsLock go down: its key-up stays swallowed"
        );
        assert!(state.is_blocked(200), "whether or not the key was held");
        assert!(!state.is_active(VK_F13));
        assert!(
            !state.is_passed(VK_F13.0),
            "a passed key the OS has up has no key-up due"
        );
        assert!(state.is_active(VK_A), "a key the OS has down is held");
        assert!(state.is_passed(VK_A.0), "and on record as passed");
    }

    #[test]
    fn release_all_frees_every_key_and_keeps_every_record() {
        // Roadmap item 2.3: keyboard input left this desktop. The OS is not
        // asked, so the reset does not race the switch to the other desktop.
        let mut state = KeyState::new();
        state.press(VK_CAPITAL);
        state.mark_blocked(VK_CAPITAL.0);
        state.press(VK_LSHIFT);
        state.mark_passed(VK_LSHIFT.0);
        state.press(VIRTUAL_KEY(255));

        state.release_all();
        state.release_all();

        for key in [VK_CAPITAL, VK_LSHIFT, VIRTUAL_KEY(255)] {
            assert!(!state.is_active(key), "{key:?} is released");
        }
        assert!(state.is_blocked(VK_CAPITAL.0), "a swallowed record stays");
        assert!(state.is_passed(VK_LSHIFT.0), "a passed record stays");
        assert!(
            !state.press(VK_CAPITAL),
            "the next key-down is a first press"
        );
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
            "a held key whose key-down was swallowed keeps its key-up swallowed"
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
    fn resync_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons() {
        let mut state = KeyState::new();
        state.press(VK_J);
        state.mark_blocked(VK_J.0);
        state.press(VK_F13);
        state.mark_passed(VK_F13.0);

        state.resync_from_os(&KeysAndButtonsDown);

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
            state.is_blocked(VK_J.0),
            "a swallowed record from before the reset stays"
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
        state.release_all();
        assert!(
            !state.alt_or_win_down_in_windows(),
            "nor does a Win that a reset freed"
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
    fn a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset() {
        // Roadmap item 2.5, cut D10: a swallowed auto-repeat of a press
        // whose key-down the engine never saw keeps its key-up from Windows,
        // across `swch on` and a lock, until the key-up consumes the record.
        let mut state = KeyState::new();
        state.press(VK_LSHIFT);
        state.mark_repeat_suppressed(VK_LSHIFT.0);
        assert!(
            !state.is_blocked(VK_LSHIFT.0),
            "it is not a swallowed key-down"
        );
        assert!(!state.is_passed(VK_LSHIFT.0));

        state.resync_from_os(&Down(&[]));
        state.release_all();
        assert!(!state.is_active(VK_LSHIFT));
        assert!(
            state.take_blocked(VK_LSHIFT.0),
            "the record survives both resets and swallows the key-up"
        );
        assert!(!state.take_blocked(VK_LSHIFT.0), "and is consumed by it");

        state.mark_repeat_suppressed(300);
        assert!(!state.take_blocked(300));
    }

    #[test]
    fn a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask() {
        // Cut D10: Windows may have such a key down, since the engine never
        // saw its key-down, so it is read like a key with no record (KS-14
        // skips only a swallowed key-down) and counts for the menu mask.
        let mut state = KeyState::new();
        state.press(VK_LMENU);
        state.mark_repeat_suppressed(VK_LMENU.0);
        assert!(state.alt_or_win_down_in_windows(), "it counts for the mask");

        state.sync_modifiers(&NothingDown, None);
        assert!(
            !state.is_active(VK_LMENU),
            "the resync reads it: Windows has it up"
        );
        assert!(!state.alt_or_win_down_in_windows());
        assert!(
            state.take_blocked(VK_LMENU.0),
            "the record stays for the key-up"
        );
    }

    #[test]
    fn a_swallowed_modifier_freed_by_a_reset_is_left_alone_by_the_modifier_resync() {
        // Input left this desktop while LShift, swallowed, was held. Its
        // record stays, so the modifier resync never takes its OS state,
        // which says nothing about a key-down Windows did not see (KS-14).
        let mut state = KeyState::new();
        state.press(VK_LSHIFT);
        state.mark_blocked(VK_LSHIFT.0);
        state.release_all();

        state.sync_modifiers(&Down(&[VK_LSHIFT]), None);

        assert!(
            !state.is_active(VK_LSHIFT),
            "the resync skips a modifier whose key-down was swallowed"
        );
        assert!(!state.is_passed(VK_LSHIFT.0));
        assert!(
            state.take_blocked(VK_LSHIFT.0),
            "its key-up is still due to be swallowed"
        );
    }
}
