//! Pending `BindingKind::Chord` gestures: "press these keys within `window`
//! of each other to fire the action".
//!
//! The engine feeds every non-repeat key-down to [`Chords::on_key_down`] once
//! per Chord binding. A chord is identified by its sorted key set; a pending
//! entry accumulates the keys seen so far until either the full set is in
//! (fire) or the window, counted from the first key, has passed (the entry
//! is dropped lazily on the next key-down).
//!
//! **Suppression**: every key-down of a key that belongs to a chord binding
//! is suppressed, whether or not the chord completes. When it completes, the
//! key-ups of all chord keys are suppressed too. Keys of a chord that does
//! not complete are lost; there is no replay. Pick chord keys that are not
//! part of normal typing.
//!
//! The key compared is the raw code from the hook, which is sided for
//! modifiers (`VK_LSHIFT`), while `"Shift"` parses to `VK_SHIFT`, so a chord
//! written with an unsided modifier never completes (KS-07, unchanged).

use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;

struct PendingChord {
    /// Canonical (sorted) key list this chord expects.
    chord_keys: Vec<u16>,
    /// Canonical (sorted) keys seen so far in the current window.
    seen_keys: Vec<u16>,
    action: BindAction,
    expires_at: Instant,
}

#[derive(Default)]
pub(super) struct Chords {
    pending: Vec<PendingChord>,
}

/// What the engine should do after consulting the chord table.
#[derive(Debug, PartialEq)]
pub(super) enum ChordOutcome {
    /// The key is not part of this binding's chord.
    NotInChord,
    /// The key is part of a chord that is still incomplete: suppress it.
    Pending,
    /// The chord just completed: suppress the key, fire `action` and
    /// suppress the key-ups of every key in `keys`.
    Completed { action: BindAction, keys: Vec<u16> },
}

impl Chords {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Feeds one key-down to one Chord binding.
    pub(super) fn on_key_down(
        &mut self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        chord_keys: &[VIRTUAL_KEY],
        window: Duration,
        now: Instant,
    ) -> ChordOutcome {
        if !chord_keys.contains(&vk) {
            return ChordOutcome::NotInChord;
        }

        // Canonicalize the chord's key set so two bindings that list the
        // same keys in different orders share the same pending entry.
        let mut canonical: Vec<u16> = chord_keys.iter().map(|k| k.0).collect();
        canonical.sort_unstable();

        // Drop expired entries. Cheap enough to do every event; keeps the
        // list bounded by the number of chords currently in progress.
        self.pending.retain(|p| p.expires_at > now);

        // Look for a pending entry matching this chord shape.
        if let Some(index) = self.pending.iter().position(|p| p.chord_keys == canonical) {
            let entry = &mut self.pending[index];
            if !entry.seen_keys.contains(&vk.0) {
                entry.seen_keys.push(vk.0);
                entry.seen_keys.sort_unstable();
            }

            if entry.seen_keys == entry.chord_keys {
                let done = self.pending.remove(index);
                return ChordOutcome::Completed {
                    action: done.action,
                    keys: done.chord_keys,
                };
            }
            return ChordOutcome::Pending;
        }

        // No pending entry: start a fresh one with this key as the first
        // seen key. Lifetime = the chord window from this moment.
        self.pending.push(PendingChord {
            chord_keys: canonical,
            seen_keys: vec![vk.0],
            action: action.clone(),
            expires_at: now + window,
        });
        ChordOutcome::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    const WINDOW: Duration = Duration::from_millis(50);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn vk_not_in_chord_returns_not_in_chord() {
        let mut state = Chords::new();
        let outcome = state.on_key_down(
            VK_A,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            WINDOW,
            Instant::now(),
        );
        assert_eq!(outcome, ChordOutcome::NotInChord);
    }

    #[test]
    fn first_chord_key_arms_pending() {
        let mut state = Chords::new();
        let outcome = state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            WINDOW,
            Instant::now(),
        );
        assert_eq!(outcome, ChordOutcome::Pending);
    }

    #[test]
    fn second_chord_key_within_window_completes() {
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(VK_J, &BindAction::SwitchLanguage, &[VK_J, VK_K], WINDOW, t0);
        let outcome = state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            WINDOW,
            t0 + ms(49),
        );
        assert_eq!(
            outcome,
            ChordOutcome::Completed {
                // The action stored by the first key fires.
                action: BindAction::SwitchLanguage,
                keys: vec![VK_J.0, VK_K.0],
            }
        );
    }

    #[test]
    fn second_chord_key_after_window_starts_over() {
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(VK_J, &BindAction::DoNothing, &[VK_J, VK_K], WINDOW, t0);
        let outcome = state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &[VK_J, VK_K],
            WINDOW,
            t0 + WINDOW,
        );
        assert_eq!(outcome, ChordOutcome::Pending);
    }

    #[test]
    fn third_chord_key_starts_fresh_after_completion() {
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(VK_J, &BindAction::DoNothing, &[VK_J, VK_K], WINDOW, t0);
        state.on_key_down(VK_K, &BindAction::DoNothing, &[VK_J, VK_K], WINDOW, t0);
        // Both chord keys consumed; a third press of J re-arms a fresh
        // pending entry: chord state must not be sticky.
        let outcome = state.on_key_down(VK_J, &BindAction::DoNothing, &[VK_J, VK_K], WINDOW, t0);
        assert_eq!(outcome, ChordOutcome::Pending);
    }

    #[test]
    fn canonical_order_independent() {
        // A chord listed as [K, J] matches the same pending entry as one
        // listed as [J, K]: order in the binding's keys does not matter.
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(VK_J, &BindAction::DoNothing, &[VK_K, VK_J], WINDOW, t0);
        let outcome = state.on_key_down(VK_K, &BindAction::DoNothing, &[VK_J, VK_K], WINDOW, t0);
        assert!(matches!(outcome, ChordOutcome::Completed { .. }));
    }
}
