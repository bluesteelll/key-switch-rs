use std::fmt;

use super::key_spec::{KeySpec, ModifierSet};

/// The keys of a `keys` binding or of one sequence step, as the config
/// names them: each a [`KeySpec`], without duplicates, in config order. Two
/// combinations are equal when they hold the same keys in any order.
///
/// Matching against the keys held is the engine's job
/// (`engine::matching`).
#[derive(Debug, Clone)]
pub struct KeyCombination {
    pub keys: Vec<KeySpec>,
}

impl PartialEq for KeyCombination {
    fn eq(&self, other: &Self) -> bool {
        if self.keys.len() != other.keys.len() {
            return false;
        }

        for key in &self.keys {
            if !other.keys.contains(key) {
                return false;
            }
        }

        true
    }
}

impl Eq for KeyCombination {}

impl KeyCombination {
    pub fn new(key: impl Into<KeySpec>) -> Self {
        Self {
            keys: vec![key.into()],
        }
    }

    #[allow(dead_code)] // Part of the public builder API; also used by tests.
    pub fn with(mut self, key: impl Into<KeySpec>) -> Self {
        let key = key.into();
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
        self
    }

    pub fn from_keys<K: Into<KeySpec>>(keys: impl IntoIterator<Item = K>) -> Self {
        let mut deduped: Vec<KeySpec> = Vec::new();
        for key in keys {
            let key = key.into();
            if !deduped.contains(&key) {
                deduped.push(key);
            }
        }
        Self { keys: deduped }
    }

    /// The modifier families this combination names, each once whichever
    /// side it names.
    pub fn modifiers(&self) -> ModifierSet {
        self.keys
            .iter()
            .filter_map(|key| key.modifier())
            .fold(ModifierSet::EMPTY, ModifierSet::with)
    }

    /// Whether every key is a modifier (`"LShift"`, `"Alt+Shift"`, `"Win"`),
    /// so that the combination has no payload key. Such a combination arms
    /// on any of its keys, and a deferred binding on it lets its keys reach
    /// Windows unless it sets `block_default`. The parser never yields an
    /// empty combination.
    pub fn is_modifier_only(&self) -> bool {
        self.keys.iter().all(|key| key.modifier().is_some())
    }
}

impl fmt::Display for KeyCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, key) in self.keys.iter().enumerate() {
            if index > 0 {
                f.write_str("+")?;
            }
            write!(f, "{key}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::key_spec::Modifier;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn from_keys_dedups_and_keeps_order() {
        let combo = KeyCombination::from_keys([VK_CONTROL, VK_A, VK_CONTROL, VK_LSHIFT]);
        assert_eq!(
            combo.keys,
            vec![
                KeySpec::AnySide(Modifier::Ctrl),
                KeySpec::Key(VK_A),
                KeySpec::Key(VK_LSHIFT),
            ]
        );
        // "Win" and "LWin" are different keys, so both stay.
        let win = KeySpec::AnySide(Modifier::Win);
        let combo = KeyCombination::new(win).with(VK_LWIN).with(win);
        assert_eq!(combo.keys, vec![win, KeySpec::Key(VK_LWIN)]);
    }

    #[test]
    fn equality_ignores_order() {
        let a = KeyCombination::new(VK_CAPITAL).with(VK_SHIFT);
        let b = KeyCombination::new(VK_SHIFT).with(VK_CAPITAL);
        assert_eq!(a, b);
        assert_ne!(a, KeyCombination::new(VK_CAPITAL).with(VK_LSHIFT));
        assert_ne!(a, KeyCombination::new(VK_CAPITAL));
    }

    #[test]
    fn generic_modifier_codes_become_either_side() {
        let combo = KeyCombination::from_keys([VK_MENU, VK_SHIFT]);
        assert_eq!(
            combo.keys,
            vec![
                KeySpec::AnySide(Modifier::Alt),
                KeySpec::AnySide(Modifier::Shift)
            ]
        );
    }

    #[test]
    fn modifiers_collects_families() {
        let combo = KeyCombination::from_keys([VK_LSHIFT, VK_RSHIFT, VK_CONTROL, VK_J]);
        assert_eq!(
            combo.modifiers(),
            ModifierSet::EMPTY
                .with(Modifier::Shift)
                .with(Modifier::Ctrl)
        );
        let win_e = KeyCombination::new(KeySpec::AnySide(Modifier::Win)).with(VK_E);
        assert_eq!(win_e.modifiers(), ModifierSet::EMPTY.with(Modifier::Win));
        assert_eq!(
            KeyCombination::new(VK_CAPITAL).modifiers(),
            ModifierSet::EMPTY
        );
    }

    #[test]
    fn is_modifier_only_needs_every_key_to_be_a_modifier() {
        let shift = KeySpec::AnySide(Modifier::Shift);
        let alt = KeySpec::AnySide(Modifier::Alt);
        let ctrl = KeySpec::AnySide(Modifier::Ctrl);
        let win = KeySpec::AnySide(Modifier::Win);

        assert!(KeyCombination::new(VK_LSHIFT).is_modifier_only());
        assert!(KeyCombination::new(alt).with(shift).is_modifier_only());
        assert!(KeyCombination::new(win).is_modifier_only());
        assert!(KeyCombination::new(ctrl).with(VK_RWIN).is_modifier_only());

        assert!(
            !KeyCombination::new(shift)
                .with(VK_CAPITAL)
                .is_modifier_only()
        );
        assert!(!KeyCombination::new(VK_CAPITAL).is_modifier_only());
        assert!(
            !KeyCombination::new(VK_LSHIFT)
                .with(VK_F13)
                .is_modifier_only()
        );
    }

    #[test]
    fn display_joins_with_plus() {
        let combo = KeyCombination::new(KeySpec::AnySide(Modifier::Win))
            .with(VK_LSHIFT)
            .with(VK_E);
        assert_eq!(combo.to_string(), "Win+LShift+E");
        assert_eq!(KeyCombination::new(VK_CAPITAL).to_string(), "CapsLock");
    }
}
