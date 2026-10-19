//! Pending `BindingKind::Chord` gestures: "press these keys within `window`
//! of each other to fire the action".
//!
//! The engine feeds every non-repeat key-down to [`Chords::on_key_down`] once
//! per Chord binding. A chord is identified by its set of keys; a pending
//! entry accumulates the keys pressed so far until every chord key has been
//! pressed (fire) or the window, counted from the first key, has passed. A
//! new entry asks for a timer at the window's end, which drops it on time
//! (roadmap item 2.2); the next key-down also drops an expired entry, so a
//! late or missing timer changes no outcome.
//!
//! **Suppression**: every key-down of a key that belongs to a chord binding
//! is suppressed, whether or not the chord completes. When it completes, the
//! key-ups of the chord keys still held are suppressed too, except a key
//! whose key-down reached the OS (pressed again while the binding's `when:`
//! did not hold); the engine checks (KS-30). Keys of a chord that does
//! not complete are lost; there is no replay. Pick chord keys that are not
//! part of normal typing.
//!
//! Chord keys are [`KeySpec`]s and the event carries the code the hook
//! reported, sided for modifiers: `"Shift"` is pressed by either Shift key,
//! `"LShift"` only by the left one (KS-07). The loader rejects chord keys
//! that overlap, so one press never counts for two chord keys.

use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;
use crate::data::key_spec::KeySpec;

struct PendingChord {
    /// The binding's chord keys, compared as a set.
    chord_keys: Vec<KeySpec>,
    /// Codes pressed so far in the current window, in press order, without
    /// duplicates. Every one was suppressed as part of this chord.
    seen: Vec<VIRTUAL_KEY>,
    action: BindAction,
    expires_at: Instant,
    /// The engine's number of the arming, which its expiry timer carries.
    seq: u64,
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
    /// The key starts a chord: suppress it, and start its expiry timer, due
    /// at the end of the window.
    Started { due: Instant },
    /// The chord just completed: suppress the key, fire `action` and
    /// suppress the key-ups of the keys in `keys`, the codes pressed for it,
    /// that are still held and whose key-down did not pass.
    Completed { action: BindAction, keys: Vec<u16> },
}

impl Chords {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Feeds one key-down to one Chord binding. `seq` is the engine's number
    /// for an entry this key-down starts.
    pub(super) fn on_key_down(
        &mut self,
        vk: VIRTUAL_KEY,
        action: &BindAction,
        chord_keys: &[KeySpec],
        window: Duration,
        now: Instant,
        seq: u64,
    ) -> ChordOutcome {
        if !chord_keys.iter().any(|key| key.accepts(vk)) {
            return ChordOutcome::NotInChord;
        }

        // Drop expired entries. Cheap enough to do every event; keeps the
        // list bounded by the number of chords currently in progress.
        self.pending.retain(|p| p.expires_at > now);

        // Look for a pending entry for the same key set, so two bindings
        // that list the same keys in different orders share it.
        if let Some(index) = self
            .pending
            .iter()
            .position(|p| same_keys(&p.chord_keys, chord_keys))
        {
            let entry = &mut self.pending[index];
            if !entry.seen.contains(&vk) {
                entry.seen.push(vk);
            }

            let complete = entry
                .chord_keys
                .iter()
                .all(|key| entry.seen.iter().any(|&code| key.accepts(code)));
            if complete {
                let done = self.pending.remove(index);
                return ChordOutcome::Completed {
                    action: done.action,
                    keys: done.seen.iter().map(|code| code.0).collect(),
                };
            }
            return ChordOutcome::Pending;
        }

        // No pending entry: start a fresh one with this key as the first
        // seen key. Lifetime = the chord window from this moment.
        let due = now + window;
        self.pending.push(PendingChord {
            chord_keys: chord_keys.to_vec(),
            seen: vec![vk],
            action: action.clone(),
            expires_at: due,
            seq,
        });
        ChordOutcome::Started { due }
    }

    /// The expiry timer of the entry armed as `seq` is due: the entry goes.
    pub(super) fn expire(&mut self, seq: u64) {
        self.pending.retain(|p| p.seq != seq);
    }

    /// Whether the entry armed as `seq` still waits for its expiry timer.
    pub(super) fn is_live(&self, seq: u64) -> bool {
        self.pending.iter().any(|p| p.seq == seq)
    }
}

/// Whether two chord key lists hold the same keys in any order. The loader
/// rejects duplicate keys, so equal length and containment suffice.
fn same_keys(a: &[KeySpec], b: &[KeySpec]) -> bool {
    a.len() == b.len() && a.iter().all(|key| b.contains(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    const WINDOW: Duration = Duration::from_millis(50);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn specs(keys: &[VIRTUAL_KEY]) -> Vec<KeySpec> {
        keys.iter().copied().map(KeySpec::from).collect()
    }

    #[test]
    fn vk_not_in_chord_returns_not_in_chord() {
        let mut state = Chords::new();
        let outcome = state.on_key_down(
            VK_A,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            Instant::now(),
            1,
        );
        assert_eq!(outcome, ChordOutcome::NotInChord);
    }

    #[test]
    fn first_chord_key_arms_pending() {
        let mut state = Chords::new();
        let now = Instant::now();
        let outcome = state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            now,
            2,
        );
        assert_eq!(outcome, ChordOutcome::Started { due: now + WINDOW });
        assert!(state.is_live(2), "it waits for its expiry timer");
        state.expire(2);
        assert!(!state.is_live(2), "the timer drops it");
    }

    #[test]
    fn second_chord_key_within_window_completes() {
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(
            VK_J,
            &BindAction::SwitchLanguage,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            3,
        );
        let outcome = state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0 + ms(49),
            4,
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
        state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            5,
        );
        let outcome = state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0 + WINDOW,
            6,
        );
        assert_eq!(
            outcome,
            ChordOutcome::Started {
                due: t0 + WINDOW + WINDOW
            },
            "a new entry with a window of its own"
        );
    }

    #[test]
    fn third_chord_key_starts_fresh_after_completion() {
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            7,
        );
        state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            8,
        );
        // Both chord keys consumed; a third press of J re-arms a fresh
        // pending entry: chord state must not be sticky.
        let outcome = state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            9,
        );
        assert_eq!(outcome, ChordOutcome::Started { due: t0 + WINDOW });
    }

    #[test]
    fn canonical_order_independent() {
        // A chord listed as [K, J] matches the same pending entry as one
        // listed as [J, K]: order in the binding's keys does not matter.
        let t0 = Instant::now();
        let mut state = Chords::new();
        state.on_key_down(
            VK_J,
            &BindAction::DoNothing,
            &specs(&[VK_K, VK_J]),
            WINDOW,
            t0,
            10,
        );
        let outcome = state.on_key_down(
            VK_K,
            &BindAction::DoNothing,
            &specs(&[VK_J, VK_K]),
            WINDOW,
            t0,
            11,
        );
        assert!(matches!(outcome, ChordOutcome::Completed { .. }));
    }

    #[test]
    fn unsided_shift_chord_completes_with_either_shift() {
        let keys = specs(&[VK_SHIFT, VK_SPACE]);
        for shift in [VK_LSHIFT, VK_RSHIFT] {
            let t0 = Instant::now();
            let mut state = Chords::new();
            let first = state.on_key_down(shift, &BindAction::DoNothing, &keys, WINDOW, t0, 12);
            assert_eq!(first, ChordOutcome::Started { due: t0 + WINDOW });
            let second = state.on_key_down(VK_SPACE, &BindAction::DoNothing, &keys, WINDOW, t0, 13);
            assert!(
                matches!(second, ChordOutcome::Completed { .. }),
                "{shift:?}"
            );
        }

        let mut state = Chords::new();
        let left_only = specs(&[VK_LSHIFT, VK_SPACE]);
        let outcome = state.on_key_down(
            VK_RSHIFT,
            &BindAction::DoNothing,
            &left_only,
            WINDOW,
            Instant::now(),
            14,
        );
        assert_eq!(outcome, ChordOutcome::NotInChord);
    }

    #[test]
    fn completed_reports_the_pressed_codes() {
        let t0 = Instant::now();
        let keys = specs(&[VK_SHIFT, VK_SPACE]);
        let mut state = Chords::new();
        state.on_key_down(
            VK_RSHIFT,
            &BindAction::SwitchLanguage,
            &keys,
            WINDOW,
            t0,
            15,
        );
        let outcome = state.on_key_down(VK_SPACE, &BindAction::DoNothing, &keys, WINDOW, t0, 16);
        assert_eq!(
            outcome,
            ChordOutcome::Completed {
                action: BindAction::SwitchLanguage,
                keys: vec![VK_RSHIFT.0, VK_SPACE.0],
            }
        );
    }
}
