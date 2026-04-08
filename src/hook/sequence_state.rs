//! State machine for `BindingKind::Sequence` — Vim/Emacs-style leader-key
//! flow. Steps are pressed in order, each step is itself a combo, and no
//! more than `max_gap` may elapse between consecutive steps.
//!
//! Hot path calls `handle_keydown` once per Sequence-kind binding per
//! non-auto-repeat key-down event. The state machine identifies a pending
//! sequence by its steps shape; multiple sequences with the same shape but
//! different actions share the same pending slot (first binding's action
//! wins on completion).
//!
//! **Suppression rules** mirror chord_state: every key-down that advances
//! a sequence is suppressed; the key-up of the final completing key is
//! also suppressed (via the blocked-keys bitmap). If a sequence times out
//! mid-flight, the suppressed key-downs are lost — no replay. Don't bind
//! sequences to letters that appear in normal typing.

use std::sync::{Arc, Mutex};
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
pub struct SequenceState {
    pending: Arc<Mutex<Vec<PendingSequence>>>,
}

pub enum SequenceOutcome {
    /// This key-down does not match the binding's first (or current) step.
    NotMatching,
    /// Sequence advanced; key-down should be suppressed.
    Advanced,
    /// Sequence completed; action has been executed. The caller should
    /// suppress this key-down AND mark the listed VKs in `blocked_keys` so
    /// their key-ups are suppressed too.
    Completed { last_step_keys: Vec<u16> },
}

impl SequenceState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one Sequence binding's spec against the current input state.
    /// Called once per binding per (non-auto-repeat) key-down event.
    pub fn handle_keydown(
        &self,
        active_keys: &[VIRTUAL_KEY],
        action: &BindAction,
        steps: &[KeyCombination],
        max_gap: Duration,
    ) -> SequenceOutcome {
        if steps.is_empty() {
            return SequenceOutcome::NotMatching;
        }

        let now = Instant::now();
        let mut pending = self.pending.lock().expect("sequence mutex poisoned");

        // Drop expired entries first — keeps the vec short and lets a fresh
        // start match cleanly even if a stale pending entry was hanging.
        pending.retain(|p| p.expires_at > now);

        // Look for an existing pending entry for this sequence shape.
        if let Some(idx) = pending.iter().position(|p| p.steps == steps) {
            let next_step_idx = pending[idx].next_step;
            if next_step_idx < steps.len() && steps[next_step_idx].matches(active_keys) {
                pending[idx].next_step += 1;
                pending[idx].expires_at = now + max_gap;

                if pending[idx].next_step == steps.len() {
                    let act = pending[idx].action.clone();
                    let last_keys: Vec<u16> =
                        steps[steps.len() - 1].keys.iter().map(|k| k.0).collect();
                    pending.remove(idx);
                    drop(pending);
                    act.execute();
                    return SequenceOutcome::Completed { last_step_keys: last_keys };
                }
                return SequenceOutcome::Advanced;
            }
            // Mismatch — abort this pending. We do NOT immediately try to
            // restart at step 0: that would silently classify a wrong key
            // as the start of a fresh attempt, which is more confusing
            // than just resetting and letting the user try again.
            pending.remove(idx);
        }

        // Fresh start: does this key-down match step 0?
        if steps[0].matches(active_keys) {
            // Length-1 sequence (degenerate; schema enforces ≥ 2 anyway):
            // fire immediately rather than hanging a useless pending entry.
            if steps.len() == 1 {
                let act = action.clone();
                let last_keys: Vec<u16> = steps[0].keys.iter().map(|k| k.0).collect();
                drop(pending);
                act.execute();
                return SequenceOutcome::Completed { last_step_keys: last_keys };
            }

            pending.push(PendingSequence {
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

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    fn combo(keys: &[VIRTUAL_KEY]) -> KeyCombination {
        KeyCombination::from_keys(keys.to_vec())
    }

    #[test]
    fn first_step_match_advances() {
        let state = SequenceState::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        let outcome = state.handle_keydown(
            &[VK_G],
            &BindAction::DoNothing,
            &steps,
            Duration::from_millis(10_000),
        );
        assert!(matches!(outcome, SequenceOutcome::Advanced));
    }

    #[test]
    fn second_step_completes_sequence() {
        let state = SequenceState::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_G])];
        state.handle_keydown(&[VK_G], &BindAction::DoNothing, &steps,
                              Duration::from_millis(10_000));
        // simulating release between presses — active_keys empty in real
        // flow, but second key-down delivers active=[G] again.
        let outcome = state.handle_keydown(
            &[VK_G],
            &BindAction::DoNothing,
            &steps,
            Duration::from_millis(10_000),
        );
        match outcome {
            SequenceOutcome::Completed { last_step_keys } => {
                assert_eq!(last_step_keys, vec![VK_G.0]);
            }
            _ => panic!("expected completion"),
        }
    }

    #[test]
    fn mismatched_intermediate_aborts() {
        let state = SequenceState::new();
        let steps = vec![combo(&[VK_G]), combo(&[VK_H])];
        state.handle_keydown(&[VK_G], &BindAction::DoNothing, &steps,
                              Duration::from_millis(10_000));
        // Wrong key: expecting H, got A.
        let outcome = state.handle_keydown(
            &[VK_A],
            &BindAction::DoNothing,
            &steps,
            Duration::from_millis(10_000),
        );
        // No match against this sequence; pending dropped.
        assert!(matches!(outcome, SequenceOutcome::NotMatching));
    }

    #[test]
    fn ctrl_x_ctrl_s_sequence() {
        // Emacs-style "Ctrl+X Ctrl+S" save. Both steps use modifier; modifier
        // stays held across the gap.
        let state = SequenceState::new();
        let steps = vec![
            KeyCombination::from_keys(vec![VK_CONTROL, VK_X]),
            KeyCombination::from_keys(vec![VK_CONTROL, VK_S]),
        ];

        // First step: active=[Ctrl, X]
        let o1 = state.handle_keydown(&[VK_CONTROL, VK_X], &BindAction::DoNothing,
                                       &steps, Duration::from_millis(10_000));
        assert!(matches!(o1, SequenceOutcome::Advanced));

        // Second step: active=[Ctrl, S]
        let o2 = state.handle_keydown(&[VK_CONTROL, VK_S], &BindAction::DoNothing,
                                       &steps, Duration::from_millis(10_000));
        match o2 {
            SequenceOutcome::Completed { last_step_keys } => {
                assert!(last_step_keys.contains(&VK_CONTROL.0));
                assert!(last_step_keys.contains(&VK_S.0));
            }
            _ => panic!("expected completion"),
        }
    }
}
