//! State machine for `BindingKind::Chord` — "press these keys within `window`
//! of each other to fire the action".
//!
//! Hot path calls `handle_keydown` once per Chord-kind binding per key-down
//! event. The chord is identified by its sorted key set; a pending entry
//! accumulates seen keys until either the full set is in (fire action) or
//! the window expires (drop entry).
//!
//! Mutex serializes access; no atomic gates needed (single critical section
//! per event).
//!
//! **Suppression**: every key-down event for a key that is part of a chord
//! binding suppresses the foreground from seeing it. When the chord
//! completes, the key-up of every chord key is also suppressed (via the
//! existing `blocked_keys` bitmap). If the chord fails to complete within
//! the window, the suppressed key-downs are simply lost — there is no
//! replay. Pick chord keys that are not part of normal typing.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;

struct PendingChord {
    /// Canonical (sorted) VK list this chord expects.
    chord_keys: Vec<u16>,
    /// Canonical (sorted) VKs seen so far in the current window.
    seen_keys: Vec<u16>,
    action: BindAction,
    expires_at: Instant,
}

#[derive(Default)]
pub struct ChordState {
    pending: Arc<Mutex<Vec<PendingChord>>>,
}

/// What the hot path should do after consulting the chord state machine.
pub enum ChordOutcome {
    /// VK is not part of this binding's chord — caller proceeds with
    /// normal handling.
    NotInChord,
    /// VK is part of a pending chord; caller must suppress the key-down.
    /// If `Some`, the chord just completed: caller must mark every listed
    /// VK as blocked so the matching key-ups get suppressed too.
    Suppress { completed_keys: Option<Vec<u16>> },
}

impl ChordState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Per-binding feed: called once for each Chord-kind binding on every
    /// raw key-down. Returns whether the key-down should be suppressed and
    /// the list of keys whose key-up must also be suppressed if the chord
    /// just completed.
    pub fn handle_keydown(
        &self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        chord_keys: &[VIRTUAL_KEY],
        window: Duration,
    ) -> ChordOutcome {
        // Canonicalize the chord's key set so two bindings that list the
        // same keys in different orders share the same pending entry.
        let mut canonical: Vec<u16> = chord_keys.iter().map(|k| k.0).collect();
        canonical.sort_unstable();

        if !canonical.contains(&vk.0) {
            return ChordOutcome::NotInChord;
        }

        let now = Instant::now();
        let mut pending = self.pending.lock().expect("chord mutex poisoned");

        // Drop expired entries. Cheap enough to do every event; keeps the
        // vec bounded by the number of currently-in-progress chords (≤ user
        // fingers).
        pending.retain(|p| p.expires_at > now);

        // Look for a pending entry matching this chord shape.
        if let Some(idx) = pending.iter().position(|p| p.chord_keys == canonical) {
            let entry = &mut pending[idx];
            if !entry.seen_keys.contains(&vk.0) {
                entry.seen_keys.push(vk.0);
                entry.seen_keys.sort_unstable();
            }

            // Completed?
            if entry.seen_keys == entry.chord_keys {
                let fired_action = entry.action.clone();
                let fired_keys = entry.chord_keys.clone();
                pending.remove(idx);
                drop(pending); // release lock before executing user action

                fired_action.execute();
                return ChordOutcome::Suppress {
                    completed_keys: Some(fired_keys),
                };
            }

            return ChordOutcome::Suppress { completed_keys: None };
        }

        // No pending entry — start a fresh one with this VK as the first
        // seen key. Lifetime = the chord window from this moment.
        pending.push(PendingChord {
            chord_keys: canonical,
            seen_keys: vec![vk.0],
            action: action.clone(),
            expires_at: now + window,
        });

        ChordOutcome::Suppress { completed_keys: None }
    }
}

/// Apply the "completed chord" side effect: mark all chord keys in the
/// blocked-keys bitmap so the corresponding key-ups also get suppressed.
/// Kept separate from `ChordOutcome` so the hot path stays the source of
/// truth for the bitmap layout.
pub fn mark_completed_keys_blocked(blocked: &[std::sync::atomic::AtomicU64; 4], keys: &[u16]) {
    for k in keys {
        let idx = (*k as usize) / 64;
        let bit = 1u64 << ((*k as usize) % 64);
        if idx < blocked.len() {
            blocked[idx].fetch_or(bit, Ordering::AcqRel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn vk_not_in_chord_returns_not_in_chord() {
        let state = ChordState::new();
        let outcome = state.handle_keydown(
            VK_A,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            Duration::from_millis(50),
        );
        assert!(matches!(outcome, ChordOutcome::NotInChord));
    }

    #[test]
    fn first_chord_key_arms_pending() {
        let state = ChordState::new();
        let outcome = state.handle_keydown(
            VK_J,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            Duration::from_millis(10_000),
        );
        match outcome {
            ChordOutcome::Suppress { completed_keys: None } => {}
            other => panic!("expected Suppress without completion, got {:?}",
                match other { ChordOutcome::NotInChord => "NotInChord",
                              ChordOutcome::Suppress { completed_keys } => {
                                  if completed_keys.is_some() { "Completed" } else { "Suppress" }
                              }}),
        }
    }

    #[test]
    fn second_chord_key_within_window_completes() {
        let state = ChordState::new();
        state.handle_keydown(VK_J, &BindAction::DoNothing, &[VK_J, VK_K],
                             Duration::from_millis(10_000));
        let outcome = state.handle_keydown(
            VK_K,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            Duration::from_millis(10_000),
        );
        match outcome {
            ChordOutcome::Suppress { completed_keys: Some(keys) } => {
                assert_eq!(keys, vec![VK_J.0, VK_K.0]);
            }
            _ => panic!("expected completion"),
        }
    }

    #[test]
    fn third_chord_key_starts_fresh_after_completion() {
        let state = ChordState::new();
        state.handle_keydown(VK_J, &BindAction::DoNothing, &[VK_J, VK_K],
                             Duration::from_millis(10_000));
        state.handle_keydown(VK_K, &BindAction::DoNothing, &[VK_J, VK_K],
                             Duration::from_millis(10_000));
        // Both chord keys consumed; a third press of J should re-arm a
        // fresh pending entry — chord state must not be sticky.
        let outcome = state.handle_keydown(
            VK_J,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            Duration::from_millis(10_000),
        );
        assert!(matches!(outcome, ChordOutcome::Suppress { completed_keys: None }));
    }

    #[test]
    fn canonical_order_independent() {
        // Chord listed as [K, J] should match the same pending entry as one
        // listed as [J, K] — order in the binding's `keys` shouldn't matter.
        let state = ChordState::new();
        state.handle_keydown(VK_J, &BindAction::DoNothing, &[VK_K, VK_J],
                             Duration::from_millis(10_000));
        let outcome = state.handle_keydown(
            VK_K,
            &BindAction::DoNothing,
            &[VK_J, VK_K], // reversed order
            Duration::from_millis(10_000),
        );
        assert!(matches!(
            outcome,
            ChordOutcome::Suppress { completed_keys: Some(_) }
        ));
    }
}
