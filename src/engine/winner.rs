//! One winner per key-down among the `keys` bindings (roadmap item 1.6,
//! KS-13).
//!
//! The candidates are the `keys` bindings that are not auto-blockers, whose
//! keys match the press (trigger key, exact modifiers or
//! `allow_extra_modifiers`; [`matching::combo_matches`]) and whose `when:`
//! holds. Precedence is an explicit rank, not the list order: more keys
//! first, then a binding with a `when:` condition over one without, then
//! the earlier binding in the list. The list is sorted stably by key count
//! (`sort_by_specificity`), so "earlier" is "earlier in the file".
//!
//! The winner alone acts on the key-down, unless it has a `Tap`, `Hold` or
//! `DoubleTap` trigger: then every deferred binding on the same keys with
//! the same `when:` that matches the press acts with it, as one gesture
//! group, so a Tap and a Hold on one key still arm together, as one press
//! ([`GroupArming`], roadmap item 2.2). [`GroupArms`] lets only the first
//! binding of each trigger kind in the group act.
//!
//! Auto-blockers, sequences and chords are not candidates. Blockers only
//! suppress, by their own rule; sequences and chords are state machines
//! that must see every key-down.
//!
//! Nothing here keeps state across events, allocates or locks: a [`Winner`]
//! borrows its binding for the one key-down.

use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::keys::KeyState;
use super::matching;
use crate::core::windows_actions::BindAction;
use crate::data::binding::{Binding, BindingKind};
use crate::data::condition::{Condition, ForegroundInfo};
use crate::data::key_combination::KeyCombination;
use crate::data::trigger::Trigger;

/// The `keys` binding that wins a key-down (roadmap item 1.6, KS-13).
pub(super) struct Winner<'a> {
    index: usize,
    binding: &'a Binding,
}

/// The precedence of a `keys` binding: its key count, then whether it has a
/// `when:` condition. `when: Always` written out counts as none.
fn rank(binding: &Binding) -> (usize, bool) {
    (binding.kind.key_count(), !binding.condition.is_always())
}

/// Picks the winner among `bindings` for the key-down of `vk`, with `held`
/// the keys held after it; `None` when no `keys` binding matches.
///
/// The winner is the first candidate in list order whose rank is strictly
/// greater than that of every earlier candidate. A binding that cannot
/// outrank the winner so far is skipped before its keys are compared, and
/// its `when:` is evaluated only once its keys match: the executable comes
/// from the main thread's foreground cache and a title condition reads the
/// window's title (roadmap item 2.3).
pub(super) fn elect<'a>(
    vk: VIRTUAL_KEY,
    bindings: &'a [Binding],
    held: &KeyState,
    foreground: &dyn ForegroundInfo,
) -> Option<Winner<'a>> {
    let mut winner: Option<Winner<'a>> = None;
    for (index, binding) in bindings.iter().enumerate() {
        let BindingKind::Combo(combo) = &binding.kind else {
            continue;
        };
        if binding.is_auto_blocker() {
            continue;
        }
        if winner
            .as_ref()
            .is_some_and(|w| rank(w.binding) >= rank(binding))
        {
            continue;
        }
        if !matching::combo_matches(combo, vk, held, binding.allow_extra_modifiers) {
            continue;
        }
        if !binding.condition.evaluate(foreground) {
            continue;
        }
        winner = Some(Winner { index, binding });
    }
    winner
}

/// What a deferred winner and its group arm on a first key-down: the term
/// and action of the first `Tap`, `Hold` and `DoubleTap` of the group, the
/// group's keys and `when:` (the identity of its DoubleTap window, known
/// issue 74), and whether any of them asks to swallow the key-down.
pub(super) struct GroupArming<'a> {
    pub(super) keys: &'a KeyCombination,
    pub(super) condition: &'a Condition,
    pub(super) tap: Option<(Duration, &'a BindAction)>,
    pub(super) hold: Option<(Duration, &'a BindAction)>,
    pub(super) double_tap: Option<(Duration, &'a BindAction)>,
    /// A member that acts sets `block_default`.
    pub(super) block: bool,
}

impl<'a> Winner<'a> {
    /// The gestures the winner's group arms on the key-down of `vk`, with
    /// `held` the keys held after it; `None` for an `Immediate` winner. The
    /// members are the bindings [`admits`](Self::admits) lets act, from the
    /// winner on in list order, and of each trigger kind only the first
    /// ([`GroupArms`]).
    pub(super) fn deferred_group(
        &self,
        vk: VIRTUAL_KEY,
        bindings: &'a [Binding],
        held: &KeyState,
    ) -> Option<GroupArming<'a>> {
        if self.binding.trigger.is_immediate() {
            return None;
        }
        let mut group = GroupArming {
            keys: self.binding.combination()?,
            condition: &self.binding.condition,
            tap: None,
            hold: None,
            double_tap: None,
            block: false,
        };
        let mut arms = GroupArms::default();
        for (index, binding) in bindings.iter().enumerate().skip(self.index) {
            if binding.is_auto_blocker()
                || !self.admits(index, binding, vk, held)
                || !arms.claim(binding.trigger)
            {
                continue;
            }
            let (slot, term_ms) = match binding.trigger {
                Trigger::Tap(term_ms) => (&mut group.tap, term_ms),
                Trigger::Hold(term_ms) => (&mut group.hold, term_ms),
                Trigger::DoubleTap(term_ms) => (&mut group.double_tap, term_ms),
                Trigger::Immediate => continue,
            };
            *slot = Some((Duration::from_millis(term_ms), &binding.action));
            group.block |= binding.block_default;
        }
        Some(group)
    }

    /// Whether the `keys` user binding at `index` acts on this key-down:
    /// the winner itself, or a member of its gesture group.
    ///
    /// A member has a deferred trigger, as the winner does, the same keys
    /// in any order, the same `when:` as written (already true for the
    /// winner on this event, so not evaluated again), and matches the press
    /// by its own `allow_extra_modifiers`. It has the winner's rank, so it
    /// comes after the winner in the list.
    pub(super) fn admits(
        &self,
        index: usize,
        binding: &Binding,
        vk: VIRTUAL_KEY,
        held: &KeyState,
    ) -> bool {
        if index == self.index {
            return true;
        }
        if self.binding.trigger.is_immediate() || binding.trigger.is_immediate() {
            return false;
        }
        match (binding.combination(), self.binding.combination()) {
            (Some(combo), Some(winner_combo)) => {
                combo == winner_combo
                    && binding.condition == self.binding.condition
                    && matching::combo_matches(combo, vk, held, binding.allow_extra_modifiers)
            }
            _ => false,
        }
    }
}

/// The trigger kinds the winning group has used on this key-down.
#[derive(Default)]
pub(super) struct GroupArms {
    tap: bool,
    hold: bool,
    double_tap: bool,
}

impl GroupArms {
    /// True for `Immediate` and for the first `Tap`, `Hold` and `DoubleTap`
    /// claimed on this key-down; false for a later one of the same kind,
    /// which then neither arms nor asks for suppression. Two `DoubleTap`s
    /// on one key would otherwise share one pending entry, and the second
    /// would fire the first one's action on the first press.
    pub(super) fn claim(&mut self, trigger: Trigger) -> bool {
        let used = match trigger {
            Trigger::Immediate => return true,
            Trigger::Tap(_) => &mut self.tap,
            Trigger::Hold(_) => &mut self.hold,
            Trigger::DoubleTap(_) => &mut self.double_tap,
        };
        !std::mem::replace(used, true)
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::condition::Condition;
    use crate::data::key_combination::KeyCombination;

    #[test]
    fn rank_puts_more_keys_before_a_when_condition() {
        let capslock = Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::SwitchLanguage);
        let conditional = capslock
            .clone()
            .with_condition(Condition::AppEquals("code.exe".to_owned()));
        let written_always = capslock.clone().with_condition(Condition::Always);
        let shift_capslock = Binding::new(
            KeyCombination::from_keys([VK_SHIFT, VK_CAPITAL]),
            BindAction::ToggleCapsLock,
        );

        assert!(rank(&shift_capslock) > rank(&conditional));
        assert!(rank(&conditional) > rank(&capslock));
        assert_eq!(rank(&written_always), rank(&capslock));
    }

    #[test]
    fn group_arms_claims_each_deferred_kind_once() {
        let mut arms = GroupArms::default();
        assert!(arms.claim(Trigger::Tap(200)));
        assert!(arms.claim(Trigger::Hold(200)));
        assert!(arms.claim(Trigger::DoubleTap(250)));
        assert!(!arms.claim(Trigger::Tap(300)));
        assert!(!arms.claim(Trigger::Hold(500)));
        assert!(!arms.claim(Trigger::DoubleTap(250)));
        assert!(arms.claim(Trigger::Immediate));
        assert!(arms.claim(Trigger::Immediate));
    }
}
