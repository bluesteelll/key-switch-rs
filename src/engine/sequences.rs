//! Pending `BindingKind::Sequence` gestures: a Vim/Emacs-style leader flow.
//! Steps are pressed in order, each step is itself a combo, and no more than
//! `max_gap` may pass between consecutive steps.
//!
//! The engine feeds every non-repeat key-down to [`Sequences::on_key_down`]
//! once per Sequence binding, together with the normalised set of held
//! keys. A pending sequence is identified by its steps, so bindings with the
//! same steps but different actions share one slot (the action of the
//! binding that started it fires).
//!
//! **Suppression** mirrors chords: every key-down that starts or advances a
//! sequence is suppressed, and on completion the key-ups of the last step's
//! keys are suppressed too. If a sequence times out or breaks, the
//! suppressed key-downs are lost; there is no replay. Don't bind sequences
//! to keys used in normal typing.

use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;
use crate::data::key_combination::KeyCombination;

struct PendingSequence {
    steps: Vec<KeyCombination>,
    next_step: usize,
    action: BindAction,
    expires_at: Instant,
}

#[derive(Default)]
pub(super) struct Sequences {
    pending: Vec<PendingSequence>,
}

#[derive(Debug, PartialEq)]
pub(super) enum SequenceOutcome {
    /// This key-down does not match the binding's first (or current) step.
    NotMatching,
    /// The sequence started or advanced: suppress the key-down.
    Advanced,
    /// The sequence completed: suppress the key-down, fire `action` and
    /// suppress the key-ups of the keys in `last_step_keys`.
    Completed {
        action: BindAction,
        last_step_keys: Vec<u16>,
    },
}

impl Sequences {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Feeds one key-down to one Sequence binding. `active_keys` is the
    /// normalised set of held keys after the key-down.
    pub(super) fn on_key_down(
        &mut self,
        active_keys: &[VIRTUAL_KEY],
        action: &BindAction,
        steps: &[KeyCombination],
        max_gap: Duration,
        now: Instant,
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
            if next_step < steps.len() && steps[next_step].matches(active_keys) {
                let entry = &mut self.pending[index];
                entry.next_step += 1;
                entry.expires_at = now + max_gap;

                if entry.next_step == steps.len() {
                    let done = self.pending.remove(index);
                    return SequenceOutcome::Completed {
                        action: done.action,
                        last_step_keys: last_step_keys(steps),
                    };
                }
                return SequenceOutcome::Advanced;
            }
            // Mismatch: abort this pending entry. The code below then checks
            // whether the same key-down starts the sequence afresh.
            self.pending.remove(index);
        }

        // Fresh start: does this key-down match step 0?
        if steps[0].matches(active_keys) {
            // Length-1 sequence (degenerate; the loader requires at least 2
            // steps): fire immediately rather than leave a useless entry.
            if steps.len() == 1 {
                return SequenceOutcome::Completed {
                    action: action.clone(),
                    last_step_keys: last_step_keys(steps),
                };
            }

            self.pending.push(PendingSequence {
                steps: steps.to_vec(),
                next_step: 1,
                action: action.clone(),
                expires_at: now + max_gap,
            });
            return SequenceOutcome::Advanced;
        }

        SequenceOutcome::NotMatching
    }
}

fn last_step_keys(steps: &[KeyCombination]) -> Vec<u16> {
    steps
        .last()
        .map(|step| step.keys.iter().map(|k| k.0).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    const GAP: Duration = Duration::from_millis(500);

    fn combo(keys: &[VIRTUAL_KEY]) -> KeyCombination {
        KeyCombination::from_keys(keys.to_vec())
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn first_step_match_advances() {
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        let outcome =
            state.on_key_down(&[VK_G], &BindAction::DoNothing, &steps, GAP, Instant::now());
        assert_eq!(outcome, SequenceOutcome::Advanced);
    }

    #[test]
    fn second_step_completes_sequence() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        state.on_key_down(&[VK_G], &BindAction::SwitchLanguage, &steps, GAP, t0);
        // The key was released between presses; the second key-down holds
        // [G] again.
        let outcome = state.on_key_down(&[VK_G], &BindAction::DoNothing, &steps, GAP, t0 + ms(100));
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
        state.on_key_down(&[VK_F13], &BindAction::DoNothing, &steps, GAP, t0);
        let outcome = state.on_key_down(&[VK_F14], &BindAction::DoNothing, &steps, GAP, t0 + GAP);
        assert_eq!(outcome, SequenceOutcome::NotMatching);
    }

    #[test]
    fn each_step_restarts_the_gap() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_F13]), combo(&[VK_F14]), combo(&[VK_F15])];
        state.on_key_down(&[VK_F13], &BindAction::DoNothing, &steps, GAP, t0);
        state.on_key_down(&[VK_F14], &BindAction::DoNothing, &steps, GAP, t0 + ms(400));
        let outcome =
            state.on_key_down(&[VK_F15], &BindAction::DoNothing, &steps, GAP, t0 + ms(800));
        assert!(matches!(outcome, SequenceOutcome::Completed { .. }));
    }

    #[test]
    fn mismatched_intermediate_aborts() {
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_H])];
        state.on_key_down(&[VK_G], &BindAction::DoNothing, &steps, GAP, t0);
        // Wrong key: expecting H, got A.
        let outcome = state.on_key_down(&[VK_A], &BindAction::DoNothing, &steps, GAP, t0);
        // No match against this sequence; pending dropped.
        assert_eq!(outcome, SequenceOutcome::NotMatching);
        let outcome = state.on_key_down(&[VK_H], &BindAction::DoNothing, &steps, GAP, t0);
        assert_eq!(outcome, SequenceOutcome::NotMatching);
    }

    #[test]
    fn ctrl_x_ctrl_s_sequence() {
        // Emacs-style "Ctrl+X Ctrl+S" save. Both steps use the modifier,
        // which stays held across the gap.
        let t0 = Instant::now();
        let mut state = Sequences::new();
        let steps = vec![combo(&[VK_CONTROL, VK_X]), combo(&[VK_CONTROL, VK_S])];

        let first = state.on_key_down(&[VK_CONTROL, VK_X], &BindAction::DoNothing, &steps, GAP, t0);
        assert_eq!(first, SequenceOutcome::Advanced);

        let second = state.on_key_down(
            &[VK_CONTROL, VK_S],
            &BindAction::DoNothing,
            &steps,
            GAP,
            t0 + ms(100),
        );
        match second {
            SequenceOutcome::Completed { last_step_keys, .. } => {
                assert!(last_step_keys.contains(&VK_CONTROL.0));
                assert!(last_step_keys.contains(&VK_S.0));
            }
            other => panic!("expected completion, got {other:?}"),
        }
    }
}
