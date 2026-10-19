//! Pending `BindingKind::Sequence` gestures: a Vim/Emacs-style leader flow.
//! Steps are pressed in order, each step is itself a combo, and no more than
//! `max_gap` may pass between consecutive steps.
//!
//! The engine feeds every non-repeat key-down to [`Sequences::on_key_down`]
//! once per Sequence binding, together with the held keys. A step matches
//! when all of its keys are held, each on its side (`"Shift"` either Shift
//! key, `"LShift"` the left one). A pending sequence is identified by its
//! steps, so bindings with the same steps but different actions share one
//! slot (the action of the binding that started it fires).
//!
//! Every start and every advance asks for an expiry timer at the new
//! deadline, with a new number, so the previous step's timer goes stale;
//! the timer drops the entry on time (roadmap item 2.2), and the next
//! key-down also drops an expired one, so a late or missing timer changes
//! no outcome.
//!
//! **Suppression** mirrors chords: every key-down that starts or advances a
//! sequence is suppressed, and on completion the key-ups of the last step's
//! non-modifier keys are suppressed too, for the keys still held whose
//! key-down did not reach the OS (the engine checks; KS-32). If a sequence
//! times out or breaks, the suppressed key-downs are lost; there is no
//! replay. Don't bind sequences to keys used in normal typing.

use std::time::{Duration, Instant};

use super::matching::{HeldKeys, keys_down};
use crate::core::windows_actions::BindAction;
use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::KeySpec;

struct PendingSequence {
    steps: Vec<KeyCombination>,
    next_step: usize,
    action: BindAction,
    expires_at: Instant,
    /// The engine's number of the latest start or advance, which its
    /// expiry timer carries.
    seq: u64,
}

#[derive(Default)]
pub(super) struct Sequences {
    pending: Vec<PendingSequence>,
}

#[derive(Debug, PartialEq)]
pub(super) enum SequenceOutcome {
    /// This key-down does not match the binding's first (or current) step.
    NotMatching,
    /// The sequence started or advanced: suppress the key-down, and start
    /// its expiry timer, due at the new deadline.
    Advanced { due: Instant },
    /// The sequence completed: suppress the key-down, fire `action` and
    /// suppress the key-ups of the keys in `last_step_keys` that are held
    /// and whose key-down did not pass.
    Completed {
        action: BindAction,
        last_step_keys: Vec<u16>,
    },
}

impl Sequences {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Feeds one key-down to one Sequence binding. `held` are the keys held
    /// after the key-down; `seq` is the engine's number for a start or an
    /// advance.
    pub(super) fn on_key_down<H: HeldKeys + ?Sized>(
        &mut self,
        held: &H,
        action: &BindAction,
        steps: &[KeyCombination],
        max_gap: Duration,
        now: Instant,
        seq: u64,
    ) -> SequenceOutcome {
        if steps.is_empty() {
            return SequenceOutcome::NotMatching;
        }

        // Drop expired entries first: keeps the list short and lets a fresh
        // start match cleanly even if a stale pending entry was hanging.
        self.pending.retain(|p| p.expires_at > now);

        // Look for an existing pending entry for this sequence shape.
        if let Some(index) = self.pending.iter().position(|p| p.steps == steps) {
            let next_step = self.pending[index].next_step;
            if next_step < steps.len() && keys_down(&steps[next_step], held) {
                let entry = &mut self.pending[index];
                entry.next_step += 1;
                entry.expires_at = now + max_gap;
                entry.seq = seq;

                if entry.next_step == steps.len() {
                    let done = self.pending.remove(index);
                    return SequenceOutcome::Completed {
                        action: done.action,
                        last_step_keys: last_step_keys(steps),
                    };
                }
                return SequenceOutcome::Advanced {
                    due: entry.expires_at,
                };
            }
            // Mismatch: abort this pending entry. The code below then checks
            // whether the same key-down starts the sequence afresh.
            self.pending.remove(index);
        }

        // Fresh start: does this key-down match step 0?
        if keys_down(&steps[0], held) {
            // Length-1 sequence (degenerate; the loader requires at least 2
            // steps): fire immediately rather than leave a useless entry.
            if steps.len() == 1 {
                return SequenceOutcome::Completed {
                    action: action.clone(),
                    last_step_keys: last_step_keys(steps),
                };
            }

            let due = now + max_gap;
            self.pending.push(PendingSequence {
                steps: steps.to_vec(),
                next_step: 1,
                action: action.clone(),
                expires_at: due,
                seq,
            });
            return SequenceOutcome::Advanced { due };
        }

        SequenceOutcome::NotMatching
    }

    /// The expiry timer of the start or advance numbered `seq` is due: the
    /// entry goes.
    pub(super) fn expire(&mut self, seq: u64) {
        self.pending.retain(|p| p.seq != seq);
    }

    /// Whether an entry still waits for the expiry timer numbered `seq`.
    pub(super) fn is_live(&self, seq: u64) -> bool {
        self.pending.iter().any(|p| p.seq == seq)
    }
}

/// The keys whose key-ups a completed sequence swallows: the codes of the
/// last step's non-modifier keys. A modifier in that step reached the OS on
/// its key-down, unless it was the completing key, which the engine marks
/// with the event itself; marking it here would swallow a key-up whose
/// key-down the OS saw. The engine also skips a non-modifier key whose
/// key-down passed, such as F14 pressed first for `["F14+F15", "F14+F16"]`.
fn last_step_keys(steps: &[KeyCombination]) -> Vec<u16> {
    steps
        .last()
        .map(|step| {
            step.keys
                .iter()
                .filter(|key| key.modifier().is_none())
                .filter_map(|key| match key {
                    KeySpec::Key(vk) => Some(vk.0),
                    KeySpec::AnySide(_) => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::keys::KeyState;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    const GAP: Duration = Duration::from_millis(500);

    fn combo(keys: &[VIRTUAL_KEY]) -> KeyCombination {
        KeyCombination::from_keys(keys.to_vec())
    }

    /// A `KeyState` with `keys` pressed: the engine's own held-key table.
    fn held(keys: &[VIRTUAL_KEY]) -> KeyState {
        let mut state = KeyState::new();
        for &key in keys {
            state.press(key);
        }
        state
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn first_step_match_advances() {
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        let outcome = state.on_key_down(
            &held(&[VK_G]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            Instant::now(),
            1,
        );
        assert!(matches!(outcome, SequenceOutcome::Advanced { .. }));
    }

    #[test]
    fn second_step_completes_sequence() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        state.on_key_down(
            &held(&[VK_G]),
            &BindAction::SwitchLanguage,
            &steps,
            GAP,
            t0,
            2,
        );
        // The key was released between presses; the second key-down holds
        // [G] again.
        let outcome = state.on_key_down(
            &held(&[VK_G]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + ms(100),
            3,
        );
        assert_eq!(
            outcome,
            SequenceOutcome::Completed {
                action: BindAction::SwitchLanguage,
                last_step_keys: vec![VK_G.0],
            }
        );
    }

    #[test]
    fn step_after_max_gap_does_not_complete() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_F13]), combo(&[VK_F14])];
        state.on_key_down(&held(&[VK_F13]), &BindAction::DoNothing, &steps, GAP, t0, 4);
        let outcome = state.on_key_down(
            &held(&[VK_F14]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + GAP,
            5,
        );
        assert_eq!(outcome, SequenceOutcome::NotMatching);
    }

    #[test]
    fn each_step_restarts_the_gap() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_F13]), combo(&[VK_F14]), combo(&[VK_F15])];
        state.on_key_down(&held(&[VK_F13]), &BindAction::DoNothing, &steps, GAP, t0, 6);
        state.on_key_down(
            &held(&[VK_F14]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + ms(400),
            7,
        );
        let outcome = state.on_key_down(
            &held(&[VK_F15]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + ms(800),
            8,
        );
        assert!(matches!(outcome, SequenceOutcome::Completed { .. }));
    }

    #[test]
    fn mismatched_intermediate_aborts() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_H])];
        state.on_key_down(&held(&[VK_G]), &BindAction::DoNothing, &steps, GAP, t0, 9);
        // Wrong key: expecting H, got A.
        let outcome =
            state.on_key_down(&held(&[VK_A]), &BindAction::DoNothing, &steps, GAP, t0, 10);
        // No match against this sequence; pending dropped.
        assert_eq!(outcome, SequenceOutcome::NotMatching);
        let outcome =
            state.on_key_down(&held(&[VK_H]), &BindAction::DoNothing, &steps, GAP, t0, 11);
        assert_eq!(outcome, SequenceOutcome::NotMatching);
    }

    #[test]
    fn ctrl_x_ctrl_s_sequence() {
        // Emacs-style "Ctrl+X Ctrl+S" save. Both steps use the modifier,
        // which stays held across the gap.
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_CONTROL, VK_X]), combo(&[VK_CONTROL, VK_S])];

        let first = state.on_key_down(
            &held(&[VK_LCONTROL, VK_X]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0,
            12,
        );
        assert!(matches!(first, SequenceOutcome::Advanced { .. }));

        let second = state.on_key_down(
            &held(&[VK_LCONTROL, VK_S]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + ms(100),
            13,
        );
        match second {
            SequenceOutcome::Completed { last_step_keys, .. } => {
                // Ctrl's key-down reached the OS, so its key-up must too:
                // only S is swallowed.
                assert_eq!(last_step_keys, vec![VK_S.0]);
            }
            other => panic!("expected completion, got {other:?}"),
        }
    }

    #[test]
    fn sided_step_needs_its_side() {
        let t0 = Instant::now();
        let steps = vec![combo(&[VK_F13]), combo(&[VK_LSHIFT, VK_F14])];

        let mut state = Sequences::new();
        state.on_key_down(
            &held(&[VK_F13]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0,
            14,
        );
        let outcome = state.on_key_down(
            &held(&[VK_RSHIFT, VK_F14]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0,
            15,
        );
        assert_eq!(outcome, SequenceOutcome::NotMatching);

        let mut state = Sequences::new();
        state.on_key_down(
            &held(&[VK_F13]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0,
            16,
        );
        let outcome = state.on_key_down(
            &held(&[VK_LSHIFT, VK_F14]),
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0,
            17,
        );
        assert_eq!(
            outcome,
            SequenceOutcome::Completed {
                action: BindAction::DoNothing,
                last_step_keys: vec![VK_F14.0],
            }
        );
    }
}
