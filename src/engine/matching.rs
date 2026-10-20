//! How a binding's keys are compared with the keys held.
//!
//! Trigger keys are [`KeySpec`]s fixed at load time: `Shift` accepts either
//! Shift key, `LShift` only the left one. The engine knows the held keys by
//! the codes of their events, which are sided for a keyboard's modifiers,
//! plus the generic modifier codes another program sends, held from that
//! code's key-down to its key-up (roadmap item 2.5). [`HeldKeys`] answers
//! for one `KeySpec` with a few table lookups: no list of held keys is built
//! per event.
//!
//! A `keys` binding is decided on the key-down of one of its own keys only,
//! its trigger keys (KS-04): a key held from before, such as CapsLock while
//! the user types, never makes a later key-down fire or arm it. Its
//! modifiers match exactly by family (owner decision): the families held
//! must be the ones it names, unless it sets `allow_extra_modifiers`.
//!
//! An auto-blocker (`block_original_combo`) exists to keep a Windows hotkey
//! from Windows, so it matches by what Windows would see (KS-05): the key
//! just pressed is one of the hotkey's, all of them are held, and every
//! other key held is one Windows does not have down, because a binding
//! swallowed its key-down or the hook missed its key-up. Only for those
//! other keys does [`blocker_matches`] ask about Windows' key state. By
//! then the key-down's check has dropped every modifier held for the
//! engine that Windows has up (roadmap item 2.5), so a modifier that the
//! hook saw released, or that a later hook swallowed, no longer counts
//! here or for any binding.

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::{KeySpec, Modifier, ModifierSet};

/// The keys the engine believes are held, asked one [`KeySpec`] at a time.
pub(super) trait HeldKeys {
    /// Whether a key that `key` accepts is held.
    fn is_down(&self, key: KeySpec) -> bool;

    /// Whether `accept` holds for the code of every key held, the generic
    /// modifier codes included; true when nothing is held.
    fn all_held(&self, accept: impl FnMut(VIRTUAL_KEY) -> bool) -> bool;

    /// The modifier families held: a family counts if either of its keys is
    /// held.
    fn modifiers(&self) -> ModifierSet {
        Modifier::ALL
            .into_iter()
            .filter(|&m| self.is_down(KeySpec::AnySide(m)))
            .fold(ModifierSet::EMPTY, ModifierSet::with)
    }
}

/// Whether every key of `combo` is held, each on its side.
pub(super) fn keys_down<H: HeldKeys + ?Sized>(combo: &KeyCombination, held: &H) -> bool {
    combo.keys.iter().all(|&key| held.is_down(key))
}

/// Whether a key-down of `pressed` can fire or arm `combo`: `pressed` is
/// one of its non-modifier keys or, for a combo made only of modifiers
/// (`"Alt+Shift"`, `"LShift"`), any of its keys. The modifiers of a combo
/// with a payload key never trigger it, so Shift pressed while CapsLock is
/// held does not fire `"Shift+CapsLock"`.
pub(super) fn is_trigger_key(combo: &KeyCombination, pressed: VIRTUAL_KEY) -> bool {
    combo
        .keys
        .iter()
        .any(|key| key.accepts(pressed) && (key.modifier().is_none() || combo.is_modifier_only()))
}

/// Whether the key-down of `pressed`, with `held` the keys held after it,
/// fires or arms `combo`: `pressed` is a trigger key, every key of the
/// combo is held on its side, and the modifier families held are exactly
/// the combo's, or include them with `allow_extra_modifiers`. Other
/// non-modifier keys held at the same time never block a match.
pub(super) fn combo_matches<H: HeldKeys + ?Sized>(
    combo: &KeyCombination,
    pressed: VIRTUAL_KEY,
    held: &H,
    allow_extra_modifiers: bool,
) -> bool {
    is_trigger_key(combo, pressed)
        && keys_down(combo, held)
        && (allow_extra_modifiers || held.modifiers() == combo.modifiers())
}

/// Whether the key-down of `pressed`, with `held` the keys held after it,
/// shows Windows its hotkey `combo` and nothing else, so that the
/// auto-blocker for it acts (swallows the key or asks for the menu mask
/// key, by its mode): `pressed` is a trigger key, every key of the hotkey
/// is held, and every other key held is one that `windows_has_down` reports
/// up.
///
/// `windows_has_down` is asked only about held keys that no key of `combo`
/// accepts, so never about `pressed`, whose state Windows updates only
/// after the hook returns. A generic modifier code that a program sent
/// counts as one of the hotkey's keys only through an `AnySide` key, so a
/// hotkey names its modifiers `AnySide`, as the registry lookup does.
pub(super) fn blocker_matches<H: HeldKeys + ?Sized>(
    combo: &KeyCombination,
    pressed: VIRTUAL_KEY,
    held: &H,
    windows_has_down: impl Fn(VIRTUAL_KEY) -> bool,
) -> bool {
    is_trigger_key(combo, pressed)
        && keys_down(combo, held)
        && held.all_held(|vk| combo.keys.iter().any(|key| key.accepts(vk)) || !windows_has_down(vk))
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;
    use crate::data::key_spec::Modifier;
    use crate::engine::keys::KeyState;

    /// A `KeyState` with `keys` pressed, so the production `HeldKeys` is the
    /// one under test.
    fn held(keys: &[VIRTUAL_KEY]) -> KeyState {
        let mut state = KeyState::new();
        for &key in keys {
            state.press(key);
        }
        state
    }

    #[test]
    fn keys_down_needs_every_key_on_its_side() {
        let left_shift_a = KeyCombination::new(VK_LSHIFT).with(VK_A);
        assert!(keys_down(&left_shift_a, &held(&[VK_LSHIFT, VK_A])));
        assert!(keys_down(
            &left_shift_a,
            &held(&[VK_LSHIFT, VK_RSHIFT, VK_A])
        ));
        assert!(!keys_down(&left_shift_a, &held(&[VK_RSHIFT, VK_A])));
        assert!(!keys_down(&left_shift_a, &held(&[VK_LSHIFT])));

        let shift_a = KeyCombination::new(KeySpec::AnySide(Modifier::Shift)).with(VK_A);
        assert!(keys_down(&shift_a, &held(&[VK_LSHIFT, VK_A])));
        assert!(keys_down(&shift_a, &held(&[VK_RSHIFT, VK_A])));
        assert!(!keys_down(&shift_a, &held(&[VK_A])));

        let win_e = KeyCombination::new(KeySpec::AnySide(Modifier::Win)).with(VK_E);
        assert!(keys_down(&win_e, &held(&[VK_RWIN, VK_E])));
        let lwin_e = KeyCombination::new(VK_LWIN).with(VK_E);
        assert!(!keys_down(&lwin_e, &held(&[VK_RWIN, VK_E])));
    }

    #[test]
    fn trigger_key_is_a_non_modifier_key_of_the_combo() {
        let ctrl_j = KeyCombination::new(KeySpec::AnySide(Modifier::Ctrl)).with(VK_J);
        assert!(is_trigger_key(&ctrl_j, VK_J));
        assert!(!is_trigger_key(&ctrl_j, VK_LCONTROL));
        assert!(!is_trigger_key(&ctrl_j, VK_RCONTROL));
        assert!(!is_trigger_key(&ctrl_j, VK_A));

        // Held keys alone do not match: the key just pressed must trigger.
        let all_held = held(&[VK_LCONTROL, VK_J]);
        assert!(combo_matches(&ctrl_j, VK_J, &all_held, false));
        assert!(!combo_matches(&ctrl_j, VK_LCONTROL, &all_held, false));
        assert!(!combo_matches(&ctrl_j, VK_J, &held(&[VK_J]), false));
    }

    #[test]
    fn modifier_only_combo_triggers_on_any_of_its_keys() {
        let alt_shift = KeyCombination::new(KeySpec::AnySide(Modifier::Alt))
            .with(KeySpec::AnySide(Modifier::Shift));
        assert!(is_trigger_key(&alt_shift, VK_LMENU));
        assert!(is_trigger_key(&alt_shift, VK_RSHIFT));
        assert!(!is_trigger_key(&alt_shift, VK_A));
        assert!(!is_trigger_key(&alt_shift, VK_LCONTROL));
    }

    #[test]
    fn sided_trigger_needs_its_side() {
        let left_shift = KeyCombination::new(VK_LSHIFT);
        assert!(is_trigger_key(&left_shift, VK_LSHIFT));
        assert!(!is_trigger_key(&left_shift, VK_RSHIFT));

        let rwin_e = KeyCombination::new(VK_RWIN).with(VK_E);
        assert!(is_trigger_key(&rwin_e, VK_E));
        assert!(!is_trigger_key(&rwin_e, VK_RWIN));
    }

    #[test]
    fn exact_modifiers_reject_an_extra_family() {
        let ctrl_j = KeyCombination::new(KeySpec::AnySide(Modifier::Ctrl)).with(VK_J);
        let extra_shift = held(&[VK_LCONTROL, VK_LSHIFT, VK_J]);
        assert!(!combo_matches(&ctrl_j, VK_J, &extra_shift, false));
        let extra_win = held(&[VK_RCONTROL, VK_RWIN, VK_J]);
        assert!(!combo_matches(&ctrl_j, VK_J, &extra_win, false));

        let capslock = KeyCombination::new(VK_CAPITAL);
        assert!(combo_matches(
            &capslock,
            VK_CAPITAL,
            &held(&[VK_CAPITAL]),
            false
        ));
        let with_shift = held(&[VK_RSHIFT, VK_CAPITAL]);
        assert!(!combo_matches(&capslock, VK_CAPITAL, &with_shift, false));
        // A non-modifier key held as well is not an extra modifier.
        let with_a = held(&[VK_A, VK_CAPITAL]);
        assert!(combo_matches(&capslock, VK_CAPITAL, &with_a, false));
    }

    #[test]
    fn either_side_of_a_family_counts_once() {
        let left_shift_f13 = KeyCombination::new(VK_LSHIFT).with(VK_F13);
        let both_shifts = held(&[VK_LSHIFT, VK_RSHIFT, VK_F13]);
        assert!(combo_matches(&left_shift_f13, VK_F13, &both_shifts, false));

        let shift_f13 = KeyCombination::new(KeySpec::AnySide(Modifier::Shift)).with(VK_F13);
        let right_shift = held(&[VK_RSHIFT, VK_F13]);
        assert!(combo_matches(&shift_f13, VK_F13, &right_shift, false));

        // The generic slot of a program's injected code counts for its
        // family too.
        let generic_shift = held(&[VK_SHIFT, VK_F13]);
        let f13 = KeyCombination::new(VK_F13);
        assert!(!combo_matches(&f13, VK_F13, &generic_shift, false));
    }

    #[test]
    fn allow_extra_modifiers_accepts_a_superset() {
        let ctrl_j = KeyCombination::new(KeySpec::AnySide(Modifier::Ctrl)).with(VK_J);
        let extras = held(&[VK_LCONTROL, VK_LSHIFT, VK_LMENU, VK_J]);
        assert!(combo_matches(&ctrl_j, VK_J, &extras, true));
        // Its own keys must still be held, on their side.
        let no_ctrl = held(&[VK_LSHIFT, VK_J]);
        assert!(!combo_matches(&ctrl_j, VK_J, &no_ctrl, true));
        let left_shift_f13 = KeyCombination::new(VK_LSHIFT).with(VK_F13);
        let right_shift = held(&[VK_RSHIFT, VK_F13]);
        assert!(!combo_matches(&left_shift_f13, VK_F13, &right_shift, true));
    }

    #[test]
    fn modifier_only_combo_is_exact_too() {
        let alt_shift = KeyCombination::new(KeySpec::AnySide(Modifier::Alt))
            .with(KeySpec::AnySide(Modifier::Shift));
        let alt_shift_held = held(&[VK_LMENU, VK_LSHIFT]);
        assert!(combo_matches(&alt_shift, VK_LSHIFT, &alt_shift_held, false));
        let with_ctrl = held(&[VK_LCONTROL, VK_LMENU, VK_LSHIFT]);
        assert!(!combo_matches(&alt_shift, VK_LSHIFT, &with_ctrl, false));
    }

    #[test]
    fn blocker_matches_only_what_windows_sees_as_its_hotkey() {
        let alt_shift = KeyCombination::new(KeySpec::AnySide(Modifier::Alt))
            .with(KeySpec::AnySide(Modifier::Shift));
        let all_down = |_: VIRTUAL_KEY| true;
        let up = |key: VIRTUAL_KEY| move |vk: VIRTUAL_KEY| vk != key;

        let lone = held(&[VK_LMENU, VK_LSHIFT]);
        assert!(blocker_matches(&alt_shift, VK_LSHIFT, &lone, all_down));
        let right = held(&[VK_RMENU, VK_RSHIFT]);
        assert!(blocker_matches(&alt_shift, VK_RMENU, &right, all_down));

        // A key held besides the hotkey counts only if Windows has it down.
        let with_tab = held(&[VK_LMENU, VK_LSHIFT, VK_TAB]);
        assert!(blocker_matches(
            &alt_shift,
            VK_LSHIFT,
            &with_tab,
            up(VK_TAB)
        ));
        assert!(!blocker_matches(&alt_shift, VK_LSHIFT, &with_tab, all_down));
        let with_ctrl = held(&[VK_LCONTROL, VK_LMENU, VK_LSHIFT]);
        assert!(
            blocker_matches(&alt_shift, VK_LSHIFT, &with_ctrl, up(VK_LCONTROL)),
            "a modifier a binding swallowed is not down in Windows"
        );
        assert!(!blocker_matches(
            &alt_shift, VK_LSHIFT, &with_ctrl, all_down
        ));

        // The key pressed must be the hotkey's, and all of it held.
        let with_a = held(&[VK_LMENU, VK_LSHIFT, VK_A]);
        assert!(!blocker_matches(&alt_shift, VK_A, &with_a, up(VK_A)));
        let alt_only = held(&[VK_LMENU]);
        assert!(!blocker_matches(&alt_shift, VK_LMENU, &alt_only, all_down));

        let grave = KeyCombination::new(VK_OEM_3);
        assert!(blocker_matches(
            &grave,
            VK_OEM_3,
            &held(&[VK_OEM_3]),
            all_down
        ));
        assert!(!blocker_matches(
            &grave,
            VK_OEM_3,
            &held(&[VK_A, VK_OEM_3]),
            all_down
        ));
        assert!(!blocker_matches(
            &grave,
            VK_OEM_3,
            &held(&[VK_LSHIFT, VK_OEM_3]),
            all_down
        ));
    }

    #[test]
    fn blocker_never_asks_windows_about_its_own_keys() {
        let alt_shift = KeyCombination::new(KeySpec::AnySide(Modifier::Alt))
            .with(KeySpec::AnySide(Modifier::Shift));
        // `VK_MENU` is the generic slot of a program's injected Alt.
        let hotkey_held = held(&[VK_LMENU, VK_MENU, VK_LSHIFT]);
        assert!(blocker_matches(
            &alt_shift,
            VK_LSHIFT,
            &hotkey_held,
            |vk: VIRTUAL_KEY| -> bool { panic!("asked Windows about {vk:?}") }
        ));
    }
}
