//! Per-key state: which keys the engine believes are held, and which keys'
//! key-ups must be swallowed because their key-down was.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::env::Env;

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

/// Bounded buffer of normalized active keys, used per keystroke to evaluate
/// bindings without allocating. 16 simultaneous distinct keys covers any
/// realistic combination; extras past the cap are silently dropped, which is
/// acceptable because bindings are sorted by specificity (longer combos
/// match first) and no real binding uses more than a handful of keys.
pub(super) const ACTIVE_KEYS_CAP: usize = 16;

pub(super) struct ActiveKeys {
    keys: [VIRTUAL_KEY; ACTIVE_KEYS_CAP],
    len: usize,
}

impl ActiveKeys {
    pub(super) fn new() -> Self {
        Self {
            keys: [VIRTUAL_KEY(0); ACTIVE_KEYS_CAP],
            len: 0,
        }
    }

    pub(super) fn push_unique(&mut self, vk: VIRTUAL_KEY) {
        if self.len >= ACTIVE_KEYS_CAP {
            return;
        }
        if self.as_slice().contains(&vk) {
            return;
        }
        self.keys[self.len] = vk;
        self.len += 1;
    }

    pub(super) fn as_slice(&self) -> &[VIRTUAL_KEY] {
        &self.keys[..self.len]
    }
}

/// Folds the sided modifier codes the hook reports into the unsided codes
/// that combos are written with: LShift/RShift to Shift, LCtrl/RCtrl to
/// Ctrl, LAlt/RAlt to Alt, and RWin to LWin (the code `Win` parses to).
fn normalize(vk: VIRTUAL_KEY) -> VIRTUAL_KEY {
    match vk {
        VK_LSHIFT | VK_RSHIFT => VK_SHIFT,
        VK_LCONTROL | VK_RCONTROL => VK_CONTROL,
        VK_LMENU | VK_RMENU => VK_MENU,
        VK_LWIN | VK_RWIN => VK_LWIN,
        other => other,
    }
}

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

    /// The held keys with sided modifiers folded (see [`normalize`]),
    /// deduplicated and capped at [`ACTIVE_KEYS_CAP`], in ascending code
    /// order.
    pub(super) fn active_keys(&self) -> ActiveKeys {
        let mut keys = ActiveKeys::new();
        for (code, &held) in self.active.iter().enumerate() {
            if held {
                keys.push_unique(normalize(VIRTUAL_KEY(code as u16)));
            }
        }
        keys
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

    #[cfg(test)]
    pub(super) fn is_active(&self, vk: VIRTUAL_KEY) -> bool {
        slot(vk.0).is_some_and(|index| self.active[index])
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn active_keys_dedup() {
        let mut buf = ActiveKeys::new();
        buf.push_unique(VK_SHIFT);
        buf.push_unique(VK_CONTROL);
        buf.push_unique(VK_SHIFT);
        assert_eq!(buf.as_slice(), &[VK_SHIFT, VK_CONTROL]);
    }

    #[test]
    fn active_keys_capacity_bound() {
        let mut buf = ActiveKeys::new();
        for i in 0..(ACTIVE_KEYS_CAP as u16 + 4) {
            buf.push_unique(VIRTUAL_KEY(i));
        }
        assert_eq!(buf.as_slice().len(), ACTIVE_KEYS_CAP);
    }

    #[test]
    fn active_keys_fold_sided_modifiers() {
        let mut state = KeyState::new();
        for vk in [VK_LSHIFT, VK_RSHIFT, VK_RCONTROL, VK_LMENU, VK_RWIN, VK_A] {
            state.press(vk);
        }
        // Ascending raw code: A (0x41), RWin (0x5C), then the sided
        // modifiers from 0xA0.
        assert_eq!(
            state.active_keys().as_slice(),
            &[VK_A, VK_LWIN, VK_SHIFT, VK_CONTROL, VK_MENU]
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
        assert!(state.active_keys().as_slice().is_empty());
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
