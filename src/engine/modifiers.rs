//! The modifier codes the engine keeps flags for, and sets of them (roadmap
//! item 2.5).
//!
//! The keyboard hook reports the sided codes of a real keyboard
//! (`VK_LSHIFT`), and another program can inject the generic ones
//! (`VK_SHIFT`), which Windows applies to a side the event does not name.
//! [`ModifierSet`] names any of the eleven codes, one bit each, so that the
//! engine can say which modifiers an event touched and which ones a read of
//! the Windows key state covers, with no list built per event.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// The modifier codes the engine keeps flags for: both sides of Shift, Ctrl,
/// Alt and Win, and the generic Shift, Ctrl and Alt codes an injector can
/// send. Windows has no generic Win code.
pub(super) const MODIFIERS: [VIRTUAL_KEY; 11] = [
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

/// A set of `MODIFIERS` codes, one bit each, in that order (roadmap item
/// 2.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierSet(u16);

impl ModifierSet {
    /// No code.
    pub const EMPTY: ModifierSet = ModifierSet(0);
    /// All eleven codes.
    pub const ALL: ModifierSet = ModifierSet((1 << MODIFIERS.len()) - 1);
    /// The eight sided codes: the left and right Shift, Ctrl, Alt and Win.
    pub const SIDED: ModifierSet = ModifierSet::of(&[
        VK_LSHIFT,
        VK_RSHIFT,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_LMENU,
        VK_RMENU,
        VK_LWIN,
        VK_RWIN,
    ]);
    /// The Alt and Win codes, the generic Alt included: a lone Alt or Win
    /// press that Windows sees activates the menu bar or opens the Start
    /// menu on its release (KS-12).
    pub const MENU: ModifierSet = ModifierSet::of(&[VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN]);

    const SHIFTS: ModifierSet = ModifierSet::of(&[VK_SHIFT, VK_LSHIFT, VK_RSHIFT]);
    const CONTROLS: ModifierSet = ModifierSet::of(&[VK_CONTROL, VK_LCONTROL, VK_RCONTROL]);
    const ALTS: ModifierSet = ModifierSet::of(&[VK_MENU, VK_LMENU, VK_RMENU]);

    /// The set of `codes`; a code that is no modifier adds nothing.
    const fn of(codes: &[VIRTUAL_KEY]) -> ModifierSet {
        let mut bits = 0;
        let mut i = 0;
        while i < codes.len() {
            bits |= ModifierSet::bit(codes[i]);
            i += 1;
        }
        ModifierSet(bits)
    }

    /// The bit of `vk`, its index in [`MODIFIERS`], or 0 if it is no
    /// modifier.
    const fn bit(vk: VIRTUAL_KEY) -> u16 {
        match vk {
            VK_SHIFT => 1 << 0,
            VK_LSHIFT => 1 << 1,
            VK_RSHIFT => 1 << 2,
            VK_CONTROL => 1 << 3,
            VK_LCONTROL => 1 << 4,
            VK_RCONTROL => 1 << 5,
            VK_MENU => 1 << 6,
            VK_LMENU => 1 << 7,
            VK_RMENU => 1 << 8,
            VK_LWIN => 1 << 9,
            VK_RWIN => 1 << 10,
            _ => 0,
        }
    }

    /// The codes whose Windows key state an event of `vk` changes: a sided
    /// Shift, Ctrl or Alt and its family's generic code; a generic code and
    /// both sides of its family; LWin or RWin alone; nothing for any other
    /// key.
    pub fn touched_by(vk: VIRTUAL_KEY) -> ModifierSet {
        let own = ModifierSet(ModifierSet::bit(vk));
        match vk {
            VK_SHIFT => ModifierSet::SHIFTS,
            VK_CONTROL => ModifierSet::CONTROLS,
            VK_MENU => ModifierSet::ALTS,
            VK_LSHIFT | VK_RSHIFT => own.union(ModifierSet(ModifierSet::bit(VK_SHIFT))),
            VK_LCONTROL | VK_RCONTROL => own.union(ModifierSet(ModifierSet::bit(VK_CONTROL))),
            VK_LMENU | VK_RMENU => own.union(ModifierSet(ModifierSet::bit(VK_MENU))),
            _ => own,
        }
    }

    /// Whether `vk` is in the set.
    pub fn contains(self, vk: VIRTUAL_KEY) -> bool {
        self.0 & ModifierSet::bit(vk) != 0
    }

    /// The codes of either set.
    pub fn union(self, other: ModifierSet) -> ModifierSet {
        ModifierSet(self.0 | other.0)
    }

    /// The codes of this set that `other` does not have.
    pub fn without(self, other: ModifierSet) -> ModifierSet {
        ModifierSet(self.0 & !other.0)
    }

    /// Whether the set has no code.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The codes of the set, in `MODIFIERS` order.
    pub fn codes(self) -> impl Iterator<Item = VIRTUAL_KEY> {
        MODIFIERS
            .into_iter()
            .enumerate()
            .filter(move |&(index, _)| self.0 & (1 << index) != 0)
            .map(|(_, vk)| vk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(codes: &[VIRTUAL_KEY]) -> Vec<VIRTUAL_KEY> {
        codes.to_vec()
    }

    #[test]
    fn a_sided_modifier_touches_its_generic_code_and_a_generic_one_both_sides() {
        // Cut D5: an event of a sided Shift, Ctrl or Alt changes the state
        // Windows reports for its generic code too, and an injector's
        // generic code is applied to a side the event does not name.
        let touched = |vk| ModifierSet::touched_by(vk).codes().collect::<Vec<_>>();
        assert_eq!(touched(VK_LSHIFT), set(&[VK_SHIFT, VK_LSHIFT]));
        assert_eq!(touched(VK_RSHIFT), set(&[VK_SHIFT, VK_RSHIFT]));
        assert_eq!(touched(VK_LCONTROL), set(&[VK_CONTROL, VK_LCONTROL]));
        assert_eq!(touched(VK_RCONTROL), set(&[VK_CONTROL, VK_RCONTROL]));
        assert_eq!(touched(VK_LMENU), set(&[VK_MENU, VK_LMENU]));
        assert_eq!(touched(VK_RMENU), set(&[VK_MENU, VK_RMENU]));
        assert_eq!(touched(VK_SHIFT), set(&[VK_SHIFT, VK_LSHIFT, VK_RSHIFT]));
        assert_eq!(
            touched(VK_CONTROL),
            set(&[VK_CONTROL, VK_LCONTROL, VK_RCONTROL])
        );
        assert_eq!(touched(VK_MENU), set(&[VK_MENU, VK_LMENU, VK_RMENU]));
        assert_eq!(touched(VK_LWIN), set(&[VK_LWIN]));
        assert_eq!(touched(VK_RWIN), set(&[VK_RWIN]));
        for vk in [VK_A, VK_CAPITAL, VK_F13, VIRTUAL_KEY(0), VIRTUAL_KEY(300)] {
            assert!(ModifierSet::touched_by(vk).is_empty(), "{vk:?}");
        }

        assert_eq!(ModifierSet::ALL.codes().collect::<Vec<_>>(), MODIFIERS);
        assert_eq!(
            ModifierSet::SIDED.codes().collect::<Vec<_>>(),
            set(&[
                VK_LSHIFT,
                VK_RSHIFT,
                VK_LCONTROL,
                VK_RCONTROL,
                VK_LMENU,
                VK_RMENU,
                VK_LWIN,
                VK_RWIN
            ])
        );
        assert_eq!(
            ModifierSet::MENU.codes().collect::<Vec<_>>(),
            set(&[VK_MENU, VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN])
        );
        for (index, vk) in MODIFIERS.into_iter().enumerate() {
            assert_eq!(ModifierSet::bit(vk), 1 << index, "{vk:?}");
            assert!(ModifierSet::ALL.contains(vk));
        }
        assert!(!ModifierSet::ALL.contains(VK_A));
        assert!(ModifierSet::EMPTY.is_empty());
        assert_eq!(
            ModifierSet::ALL
                .without(ModifierSet::SIDED)
                .codes()
                .collect::<Vec<_>>(),
            set(&[VK_SHIFT, VK_CONTROL, VK_MENU])
        );
    }
}
