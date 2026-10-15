//! Trigger keys as a config names them: one key on its side, or a modifier
//! on either side.
//!
//! The low-level hook reports modifiers by side (`VK_LSHIFT`, `VK_RWIN`). A
//! config names either side (`"Shift"`, `"Win"`) or one side (`"LShift"`,
//! `"RWin"`), and a `VIRTUAL_KEY` cannot tell the two apart for the Win keys:
//! `"Win"` and `"LWin"` both parse to `VK_LWIN`. A [`KeySpec`] keeps the
//! difference. Every trigger key (the `keys` of a binding, sequence steps and
//! chord keys) is a `KeySpec` fixed at load time, and the engine compares the
//! code of each event against it (KS-07). Keys the daemon sends (`PressKey`,
//! macro steps) stay `VIRTUAL_KEY`.

use std::fmt;

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::vk_name::{normalize_key_name, parse_vk, vk_name};

/// A modifier family. Both keys of a pair count as the same modifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    Shift,
    Ctrl,
    Alt,
    Win,
}

impl Modifier {
    pub const ALL: [Modifier; 4] = [
        Modifier::Shift,
        Modifier::Ctrl,
        Modifier::Alt,
        Modifier::Win,
    ];

    /// The left key of the pair.
    pub fn left(self) -> VIRTUAL_KEY {
        match self {
            Modifier::Shift => VK_LSHIFT,
            Modifier::Ctrl => VK_LCONTROL,
            Modifier::Alt => VK_LMENU,
            Modifier::Win => VK_LWIN,
        }
    }

    /// The right key of the pair.
    pub fn right(self) -> VIRTUAL_KEY {
        match self {
            Modifier::Shift => VK_RSHIFT,
            Modifier::Ctrl => VK_RCONTROL,
            Modifier::Alt => VK_RMENU,
            Modifier::Win => VK_RWIN,
        }
    }

    /// The code Windows uses for "either key of the pair". The Win keys have
    /// none.
    pub fn generic(self) -> Option<VIRTUAL_KEY> {
        match self {
            Modifier::Shift => Some(VK_SHIFT),
            Modifier::Ctrl => Some(VK_CONTROL),
            Modifier::Alt => Some(VK_MENU),
            Modifier::Win => None,
        }
    }

    /// The family of a sided or generic modifier code; `None` for any other
    /// key.
    pub fn of(vk: VIRTUAL_KEY) -> Option<Modifier> {
        Self::ALL
            .into_iter()
            .find(|m| vk == m.left() || vk == m.right() || Some(vk) == m.generic())
    }

    /// The config name that means either key of the pair.
    pub fn name(self) -> &'static str {
        match self {
            Modifier::Shift => "Shift",
            Modifier::Ctrl => "Ctrl",
            Modifier::Alt => "Alt",
            Modifier::Win => "Win",
        }
    }

    /// This family's bit in a [`ModifierSet`].
    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of modifier families: which of Shift, Ctrl, Alt and Win a binding
/// names, or which are held. Sides do not count here; exact modifier
/// matching compares families.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierSet(u8);

impl ModifierSet {
    pub const EMPTY: ModifierSet = ModifierSet(0);

    /// This set with `modifier` added.
    pub fn with(self, modifier: Modifier) -> Self {
        ModifierSet(self.0 | modifier.bit())
    }

    pub fn contains(self, modifier: Modifier) -> bool {
        self.0 & modifier.bit() != 0
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One trigger key of a binding.
///
/// Built from a config name with [`KeySpec::parse`] or from a code with
/// `From<VIRTUAL_KEY>`. The generic codes `VK_SHIFT`, `VK_CONTROL` and
/// `VK_MENU` become `AnySide`; every other code, `VK_LWIN` included, is that
/// one key. "Either Win key" has no code and is written
/// `KeySpec::AnySide(Modifier::Win)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySpec {
    /// Either key of a modifier pair: `"Shift"`, `"Ctrl"`, `"Alt"`, `"Win"`.
    AnySide(Modifier),
    /// This key only, on its side for a modifier: `"LShift"`, `"RWin"`, `"A"`.
    Key(VIRTUAL_KEY),
}

impl KeySpec {
    /// Whether an event with code `vk` is this key. `AnySide` also accepts
    /// the generic code, which only third-party injected input carries.
    pub fn accepts(self, vk: VIRTUAL_KEY) -> bool {
        match self {
            KeySpec::AnySide(m) => vk == m.left() || vk == m.right() || Some(vk) == m.generic(),
            KeySpec::Key(key) => vk == key,
        }
    }

    /// The modifier family of this key, or `None` for a key that is not a
    /// modifier.
    pub fn modifier(self) -> Option<Modifier> {
        match self {
            KeySpec::AnySide(m) => Some(m),
            KeySpec::Key(vk) => Modifier::of(vk),
        }
    }

    /// Whether one key is accepted by both: `Shift` and `LShift`, `Win` and
    /// `RWin`, `A` and `A`.
    pub fn overlaps(self, other: KeySpec) -> bool {
        match (self, other) {
            (KeySpec::AnySide(a), KeySpec::AnySide(b)) => a == b,
            (KeySpec::AnySide(m), KeySpec::Key(vk)) | (KeySpec::Key(vk), KeySpec::AnySide(m)) => {
                KeySpec::AnySide(m).accepts(vk)
            }
            (KeySpec::Key(a), KeySpec::Key(b)) => a == b,
        }
    }

    /// Parses a config key name. `"Win"` and `"Super"` mean either Win key;
    /// every other name is parsed by [`parse_vk`], so `"LWin"` is the left
    /// one. `None` for an unknown name.
    pub fn parse(name: &str) -> Option<KeySpec> {
        match normalize_key_name(name).as_str() {
            "win" | "super" => Some(KeySpec::AnySide(Modifier::Win)),
            _ => parse_vk(name).map(KeySpec::from),
        }
    }
}

impl From<VIRTUAL_KEY> for KeySpec {
    fn from(vk: VIRTUAL_KEY) -> Self {
        match vk {
            VK_SHIFT => KeySpec::AnySide(Modifier::Shift),
            VK_CONTROL => KeySpec::AnySide(Modifier::Ctrl),
            VK_MENU => KeySpec::AnySide(Modifier::Alt),
            other => KeySpec::Key(other),
        }
    }
}

impl fmt::Display for KeySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            KeySpec::AnySide(m) => f.write_str(m.name()),
            // `vk_name` renders VK_LWIN as "Win" for the keys the daemon
            // sends; as a trigger it is the left key only.
            KeySpec::Key(VK_LWIN) => f.write_str("LWin"),
            KeySpec::Key(vk) => f.write_str(&vk_name(vk)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsided_names_parse_to_either_side() {
        for (name, modifier) in [
            ("Shift", Modifier::Shift),
            ("Ctrl", Modifier::Ctrl),
            ("Control", Modifier::Ctrl),
            ("Alt", Modifier::Alt),
            ("Menu", Modifier::Alt),
            ("Win", Modifier::Win),
            ("Super", Modifier::Win),
            ("  wIn ", Modifier::Win),
        ] {
            assert_eq!(
                KeySpec::parse(name),
                Some(KeySpec::AnySide(modifier)),
                "{name:?}"
            );
        }
    }

    #[test]
    fn sided_names_parse_to_one_side() {
        for (name, vk) in [
            ("LShift", VK_LSHIFT),
            ("RShift", VK_RSHIFT),
            ("LCtrl", VK_LCONTROL),
            ("RCtrl", VK_RCONTROL),
            ("LAlt", VK_LMENU),
            ("RAlt", VK_RMENU),
            ("LWin", VK_LWIN),
            ("LeftWin", VK_LWIN),
            ("RWin", VK_RWIN),
            ("RightWin", VK_RWIN),
        ] {
            assert_eq!(KeySpec::parse(name), Some(KeySpec::Key(vk)), "{name:?}");
        }
    }

    #[test]
    fn other_names_parse_to_their_key() {
        assert_eq!(KeySpec::parse("A"), Some(KeySpec::Key(VK_A)));
        assert_eq!(KeySpec::parse("CapsLock"), Some(KeySpec::Key(VK_CAPITAL)));
        assert_eq!(KeySpec::parse("F13"), Some(KeySpec::Key(VK_F13)));
        assert_eq!(KeySpec::parse("Foo"), None);
        assert_eq!(KeySpec::parse(""), None);
    }

    #[test]
    fn any_side_accepts_both_sides_and_the_generic_code() {
        let shift = KeySpec::AnySide(Modifier::Shift);
        assert!(shift.accepts(VK_LSHIFT));
        assert!(shift.accepts(VK_RSHIFT));
        assert!(shift.accepts(VK_SHIFT));
        assert!(!shift.accepts(VK_LCONTROL));
        assert!(!shift.accepts(VK_A));

        let win = KeySpec::AnySide(Modifier::Win);
        assert!(win.accepts(VK_LWIN));
        assert!(win.accepts(VK_RWIN));
        assert!(!win.accepts(VK_LMENU));
    }

    #[test]
    fn sided_key_accepts_only_its_side() {
        let left_shift = KeySpec::Key(VK_LSHIFT);
        assert!(left_shift.accepts(VK_LSHIFT));
        assert!(!left_shift.accepts(VK_RSHIFT));
        assert!(!left_shift.accepts(VK_SHIFT));

        let left_win = KeySpec::Key(VK_LWIN);
        assert!(left_win.accepts(VK_LWIN));
        assert!(!left_win.accepts(VK_RWIN));
    }

    #[test]
    fn win_has_no_generic_code() {
        assert_eq!(Modifier::Win.generic(), None);
        assert_eq!(KeySpec::from(VK_LWIN), KeySpec::Key(VK_LWIN));
        assert_eq!(KeySpec::from(VK_SHIFT), KeySpec::AnySide(Modifier::Shift));
        assert_eq!(KeySpec::from(VK_CONTROL), KeySpec::AnySide(Modifier::Ctrl));
        assert_eq!(KeySpec::from(VK_MENU), KeySpec::AnySide(Modifier::Alt));
    }

    #[test]
    fn modifier_families() {
        assert_eq!(Modifier::of(VK_RCONTROL), Some(Modifier::Ctrl));
        assert_eq!(Modifier::of(VK_MENU), Some(Modifier::Alt));
        assert_eq!(Modifier::of(VK_RWIN), Some(Modifier::Win));
        assert_eq!(Modifier::of(VK_CAPITAL), None);
        assert_eq!(KeySpec::Key(VK_RSHIFT).modifier(), Some(Modifier::Shift));
        assert_eq!(
            KeySpec::AnySide(Modifier::Win).modifier(),
            Some(Modifier::Win)
        );
        assert_eq!(KeySpec::Key(VK_A).modifier(), None);
    }

    #[test]
    fn overlaps_table() {
        let any_shift = KeySpec::AnySide(Modifier::Shift);
        let any_win = KeySpec::AnySide(Modifier::Win);
        assert!(any_shift.overlaps(KeySpec::Key(VK_LSHIFT)));
        assert!(KeySpec::Key(VK_RSHIFT).overlaps(any_shift));
        assert!(any_shift.overlaps(any_shift));
        assert!(any_win.overlaps(KeySpec::Key(VK_RWIN)));
        assert!(KeySpec::Key(VK_A).overlaps(KeySpec::Key(VK_A)));

        assert!(!KeySpec::Key(VK_LSHIFT).overlaps(KeySpec::Key(VK_RSHIFT)));
        assert!(!any_shift.overlaps(KeySpec::AnySide(Modifier::Ctrl)));
        assert!(!any_win.overlaps(KeySpec::Key(VK_LMENU)));
        assert!(!KeySpec::Key(VK_A).overlaps(KeySpec::Key(VK_B)));
    }

    #[test]
    fn modifier_set_holds_families() {
        let set = ModifierSet::EMPTY.with(Modifier::Ctrl).with(Modifier::Win);
        assert!(set.contains(Modifier::Ctrl));
        assert!(set.contains(Modifier::Win));
        assert!(!set.contains(Modifier::Shift));
        assert!(!set.contains(Modifier::Alt));
        assert_eq!(set, set.with(Modifier::Ctrl));
        assert_eq!(
            set,
            ModifierSet::EMPTY.with(Modifier::Win).with(Modifier::Ctrl)
        );
        assert_ne!(set, ModifierSet::EMPTY.with(Modifier::Ctrl));
        assert_eq!(ModifierSet::default(), ModifierSet::EMPTY);
    }

    #[test]
    fn display_names() {
        assert_eq!(KeySpec::AnySide(Modifier::Win).to_string(), "Win");
        assert_eq!(KeySpec::Key(VK_LWIN).to_string(), "LWin");
        assert_eq!(KeySpec::Key(VK_RWIN).to_string(), "RWin");
        assert_eq!(KeySpec::AnySide(Modifier::Shift).to_string(), "Shift");
        assert_eq!(KeySpec::Key(VK_LSHIFT).to_string(), "LShift");
        assert_eq!(KeySpec::Key(VK_CAPITAL).to_string(), "CapsLock");
        assert_eq!(Modifier::Alt.to_string(), "Alt");
    }

    #[test]
    fn display_round_trips_through_parse() {
        for spec in [
            KeySpec::AnySide(Modifier::Win),
            KeySpec::Key(VK_LWIN),
            KeySpec::Key(VK_RWIN),
            KeySpec::AnySide(Modifier::Ctrl),
            KeySpec::Key(VK_RCONTROL),
            KeySpec::Key(VK_F13),
        ] {
            assert_eq!(KeySpec::parse(&spec.to_string()), Some(spec), "{spec}");
        }
    }
}
