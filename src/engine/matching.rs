//! How a binding's keys are compared with the keys held.
//!
//! Trigger keys are [`KeySpec`]s fixed at load time: `Shift` accepts either
//! Shift key, `LShift` only the left one. The engine knows the held keys by
//! the codes the hook reported, which are sided for modifiers, plus the
//! generic modifier codes its resync reads from the OS. [`HeldKeys`] answers
//! for one `KeySpec` with a few table lookups: no list of held keys is built
//! per event.
//!
//! A `keys` binding is decided on the key-down of one of its own keys only,
//! its trigger keys (KS-04): a key held from before, such as CapsLock while
//! the user types, never makes a later key-down fire or arm it.

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::KeySpec;

/// The keys the engine believes are held, asked one [`KeySpec`] at a time.
pub(super) trait HeldKeys {
    /// Whether a key that `key` accepts is held.
    fn is_down(&self, key: KeySpec) -> bool;
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
    combo.keys.iter().any(|key| {
        key.accepts(pressed)
            && (key.modifier().is_none() || combo.keys.iter().all(|k| k.modifier().is_some()))
    })
}

/// Whether the key-down of `pressed`, with `held` the keys held after it,
/// fires or arms `combo`.
pub(super) fn combo_matches<H: HeldKeys + ?Sized>(
    combo: &KeyCombination,
    pressed: VIRTUAL_KEY,
    held: &H,
) -> bool {
    is_trigger_key(combo, pressed) && keys_down(combo, held)
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
        assert!(combo_matches(&ctrl_j, VK_J, &all_held));
        assert!(!combo_matches(&ctrl_j, VK_LCONTROL, &all_held));
        assert!(!combo_matches(&ctrl_j, VK_J, &held(&[VK_J])));
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
}
