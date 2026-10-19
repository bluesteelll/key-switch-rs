//! Pending `Tap`, `Hold` and `DoubleTap` gestures, as plain data: one
//! machine per press (roadmap item 2.2, KS-25).
//!
//! A first key-down that a gesture group wins (`winner::GroupArming`) arms
//! one press under the key just pressed, with every gesture of the group:
//! a Tap deadline, a Hold deadline, for which the engine asks for a timer,
//! and a DoubleTap window. The press resolves to one of them, never two:
//!
//! - Its **Hold**, when the Hold's timer fires while the key is held, or
//!   when the key-up came at or after the Hold's term: the press is then
//!   left to its timer as a due Hold, which fires even if the timer runs
//!   after the key-up. Only a timer ever fires a Hold. A press that its Hold
//!   resolved fires no Tap and opens no DoubleTap window.
//! - Otherwise, at its key-up, its **Tap** if the key-up came before the
//!   Tap's term, and its **DoubleTap** window opens if the key-up came
//!   before the window closed. With both, the Tap waits for the window
//!   (open question 5): a second press of the key that the same group wins
//!   within the window fires only the DoubleTap, and otherwise the Tap fires
//!   when the window closes, on its own timer, or at once when a key goes
//!   down first. Without a DoubleTap the Tap fires at the key-up, as before.
//!
//! A DoubleTap window belongs to its group, the keys and `when:` of the
//! bindings that armed it (known issue 74): a press of the key that another
//! group wins ends the window instead of completing it, and opens its own.
//! Identity by keys and `when:`, not by a binding's place in the list,
//! survives a reload.
//!
//! A first key-down of another key cancels a pending press whose key-down
//! passed (owner decision, known issue 67), held or waiting for its second
//! press, unless its Hold's term has passed by the clock; a press whose
//! key-down was swallowed stays. A waiting Tap fires instead, ahead of the
//! new key. Auto-repeats and key-ups of other keys change nothing.
//!
//! Time comes in as arguments; nothing here reads a clock or sleeps. Every
//! press keeps the engine's number of its arming (`seq`), which its timer
//! token carries, so a timer only acts on the press it was started for
//! (KS-11). A press that goes on to wait for its DoubleTap window takes a
//! new number: the timer of its cancelled Hold then finds nothing.

use std::time::Instant;

use super::keys::KeyState;
use super::winner::GroupArming;
use crate::core::windows_actions::BindAction;
use crate::data::condition::Condition;
use crate::data::key_combination::KeyCombination;

/// The keys and `when:` of the group that opened a DoubleTap window: what
/// the group of a second press must have to complete it. `KeyCombination`
/// compares its keys in any order.
#[derive(PartialEq)]
struct GroupKey {
    keys: KeyCombination,
    condition: Condition,
}

/// The gesture machine of one press of one key.
struct Press {
    vk: u16,
    /// The engine's number of the arming, which this press's timer token
    /// carries.
    seq: u64,
    phase: Phase,
}

enum Phase {
    /// The key is held. Each gesture with its deadline and the action it
    /// fires; the DoubleTap with its group.
    Held {
        tap: Option<(Instant, BindAction)>,
        hold: Option<(Instant, BindAction)>,
        double: Option<(Instant, BindAction, GroupKey)>,
    },
    /// The key went up in time for a DoubleTap: a second press won by
    /// `group` before `due` fires `double`. `deferred_tap` is the press's
    /// Tap, which then waits for the window and has a timer at `due`.
    /// `passed` is whether the press's key-down reached Windows.
    AwaitSecond {
        due: Instant,
        double: BindAction,
        group: GroupKey,
        deferred_tap: Option<BindAction>,
        passed: bool,
    },
}

impl Phase {
    /// When the timer this phase waits for is due, if it waits for one: the
    /// Hold of a held press, the Tap of a press that waits for its window.
    fn timer_due(&self) -> Option<Instant> {
        match self {
            Phase::Held {
                hold: Some((due, _)),
                ..
            }
            | Phase::AwaitSecond {
                due,
                deferred_tap: Some(_),
                ..
            } => Some(*due),
            Phase::Held { .. } | Phase::AwaitSecond { .. } => None,
        }
    }

    /// What its timer fires; `None` if it waits for no timer.
    fn into_timer_action(self) -> Option<BindAction> {
        match self {
            Phase::Held {
                hold: Some((_, action)),
                ..
            }
            | Phase::AwaitSecond {
                deferred_tap: Some(action),
                ..
            } => Some(action),
            Phase::Held { .. } | Phase::AwaitSecond { .. } => None,
        }
    }
}

/// A press resolved as a Hold whose timer has not run yet. It is no longer
/// its key's press: it neither blocks the next press nor answers its key-up.
struct DueHold {
    vk: u16,
    seq: u64,
    due: Instant,
    action: BindAction,
}

/// What a key-up did ([`Gestures::release`]).
#[derive(Debug, Default, PartialEq)]
pub(super) struct Released {
    /// The Tap to fire now.
    pub(super) fired: Option<BindAction>,
    /// A Tap fired or now waits for its window, or the press had a Hold:
    /// the gesture is evidence of a swallowed key-down.
    pub(super) gesture_bound: bool,
    /// The press waits for its DoubleTap window with its Tap: start a timer
    /// due then, with the `seq` the key-up passed.
    pub(super) window_due: Option<Instant>,
}

#[derive(Default)]
pub(super) struct Gestures {
    /// At most one per key.
    presses: Vec<Press>,
    due_holds: Vec<DueHold>,
}

impl Gestures {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// A first key-down of `vk`, at `now`, which `group` wins if it is a
    /// gesture group, as the engine's arming `seq`. Returns the due time of
    /// the Hold to start a timer for. Actions to fire go to `fire`.
    ///
    /// A press of `vk` that waits for its DoubleTap window ends here: with
    /// a DoubleTap in `group`, the same keys and `when:` and the window
    /// still open, its DoubleTap fires and this press is spent, arming
    /// nothing; otherwise its waiting Tap, if any, fires first. A held press
    /// of `vk` left over from a key-up that was never seen is replaced; its
    /// Hold, if due by the clock, stays for its timer.
    pub(super) fn press(
        &mut self,
        vk: u16,
        group: Option<&GroupArming>,
        now: Instant,
        seq: u64,
        fire: &mut Vec<BindAction>,
    ) -> Option<Instant> {
        if let Some(index) = self.presses.iter().position(|p| p.vk == vk) {
            let earlier = self.presses.remove(index);
            match earlier.phase {
                Phase::AwaitSecond {
                    due,
                    double,
                    group: window,
                    deferred_tap,
                    ..
                } => {
                    let completes = now < due
                        && group.is_some_and(|group| {
                            group.double_tap.is_some()
                                && *group.keys == window.keys
                                && *group.condition == window.condition
                        });
                    if completes {
                        fire.push(double);
                        return None;
                    }
                    fire.extend(deferred_tap);
                }
                Phase::Held { hold, .. } => {
                    if let Some((due, action)) = hold
                        && now >= due
                    {
                        self.due_holds.push(DueHold {
                            vk,
                            seq: earlier.seq,
                            due,
                            action,
                        });
                    }
                }
            }
        }

        let group = group?;
        let tap = group.tap.map(|(term, action)| (now + term, action.clone()));
        let hold = group
            .hold
            .map(|(term, action)| (now + term, action.clone()));
        let double = group.double_tap.map(|(term, action)| {
            let key = GroupKey {
                keys: group.keys.clone(),
                condition: group.condition.clone(),
            };
            (now + term, action.clone(), key)
        });
        let hold_due = hold.as_ref().map(|(due, _)| *due);
        self.presses.push(Press {
            vk,
            seq,
            phase: Phase::Held { tap, hold, double },
        });
        hold_due
    }

    /// The key-up of `vk` at `now`; `passed` is whether its key-down
    /// reached Windows, and `seq` the engine's number for a press that goes
    /// on to wait for its DoubleTap window.
    pub(super) fn release(&mut self, vk: u16, now: Instant, passed: bool, seq: u64) -> Released {
        let mut released = Released::default();
        let Some(index) = self
            .presses
            .iter()
            .position(|p| p.vk == vk && matches!(p.phase, Phase::Held { .. }))
        else {
            return released;
        };
        let press = self.presses.remove(index);
        let Phase::Held { tap, hold, double } = press.phase else {
            return released;
        };

        if let Some((due, action)) = hold {
            released.gesture_bound = true;
            if now >= due {
                // Held for the whole term: the Hold resolves the press, and
                // its timer fires it whenever it runs.
                self.due_holds.push(DueHold {
                    vk,
                    seq: press.seq,
                    due,
                    action,
                });
                return released;
            }
        }
        let tap = tap.filter(|(due, _)| now < *due).map(|(_, action)| action);
        released.gesture_bound |= tap.is_some();
        match (tap, double.filter(|(due, ..)| now < *due)) {
            (tap, Some((due, double, group))) => {
                released.window_due = tap.as_ref().map(|_| due);
                self.presses.push(Press {
                    vk,
                    seq,
                    phase: Phase::AwaitSecond {
                        due,
                        double,
                        group,
                        deferred_tap: tap,
                        passed,
                    },
                });
            }
            (Some(tap), None) => released.fired = Some(tap),
            (None, None) => {}
        }
        released
    }

    /// A first key-down of `vk` at `now`, for the presses of the other
    /// keys (`keys` is the engine's record of them), as the owner decided
    /// (roadmap item 2.2, open question 10; AutoHotkey does the same):
    ///
    /// - A Tap that waits for its DoubleTap window has completed, so it
    ///   fires now, ahead of the new key, and its window closes.
    /// - A pending gesture whose key passed, held or waiting for a second
    ///   press, is cancelled: a key pressed during it makes it a shortcut,
    ///   such as Ctrl+C or two capitals with Shift. A held press whose Hold
    ///   term has passed is not pending any more: it stays for its timer.
    /// - A gesture whose key was swallowed stays, as before.
    pub(super) fn other_key_down(
        &mut self,
        vk: u16,
        keys: &KeyState,
        now: Instant,
        fire: &mut Vec<BindAction>,
    ) {
        let mut index = 0;
        while index < self.presses.len() {
            let press = &self.presses[index];
            let cancel = press.vk != vk
                && match &press.phase {
                    Phase::AwaitSecond {
                        deferred_tap: Some(_),
                        ..
                    } => true,
                    Phase::AwaitSecond { passed, .. } => *passed,
                    // A Hold whose term has passed has resolved the press by
                    // the clock, whether or not its timer has run yet, so a
                    // late timer cannot change the outcome (orchestrator
                    // ruling I1): it stays for its timer, as a due Hold does.
                    Phase::Held { hold, .. } => {
                        keys.is_passed(press.vk)
                            && !hold.as_ref().is_some_and(|(due, _)| now >= *due)
                    }
                };
            if !cancel {
                index += 1;
                continue;
            }
            if let Phase::AwaitSecond {
                deferred_tap: Some(tap),
                ..
            } = self.presses.remove(index).phase
            {
                fire.push(tap);
            }
        }
    }

    /// The timer of `vk` armed as `seq` is due: fires the Hold or the
    /// waiting Tap it was started for, if that still waits for it. Waiting
    /// Taps whose windows closed by then fire first, so that one whose own
    /// timer could not be started is not lost.
    pub(super) fn on_timer(&mut self, vk: u16, seq: u64, fire: &mut Vec<BindAction>) {
        let fired = if let Some(index) = self
            .due_holds
            .iter()
            .position(|h| h.vk == vk && h.seq == seq)
        {
            let hold = self.due_holds.remove(index);
            Some((hold.due, hold.action))
        } else {
            self.presses
                .iter()
                .position(|p| p.vk == vk && p.seq == seq && p.phase.timer_due().is_some())
                .and_then(|index| {
                    let press = self.presses.remove(index);
                    let due = press.phase.timer_due()?;
                    Some((due, press.phase.into_timer_action()?))
                })
        };
        let Some((due, action)) = fired else {
            return;
        };
        self.fire_closed_windows(due, fire);
        fire.push(action);
    }

    /// Fires every Tap that waits for a DoubleTap window closed by `now`.
    fn fire_closed_windows(&mut self, now: Instant, fire: &mut Vec<BindAction>) {
        let mut index = 0;
        while index < self.presses.len() {
            let closed = matches!(
                self.presses[index].phase,
                Phase::AwaitSecond {
                    due,
                    deferred_tap: Some(_),
                    ..
                } if due <= now
            );
            if !closed {
                index += 1;
                continue;
            }
            if let Phase::AwaitSecond {
                deferred_tap: Some(tap),
                ..
            } = self.presses.remove(index).phase
            {
                fire.push(tap);
            }
        }
    }

    /// Whether the press or due Hold of `vk` armed as `seq` waits for a
    /// timer.
    pub(super) fn is_live(&self, vk: u16, seq: u64) -> bool {
        self.due_holds.iter().any(|h| h.vk == vk && h.seq == seq)
            || self
                .presses
                .iter()
                .any(|p| p.vk == vk && p.seq == seq && p.phase.timer_due().is_some())
    }

    /// The timer of `vk` armed as `seq` could not be started. A Hold, held
    /// or due, is dropped: it never fires, and its key arms afresh on the
    /// next press (KS-33). A Tap that waits for its DoubleTap window stays:
    /// it fires at the next key-down, or at the next timer due after its
    /// window closed (orchestrator ruling M6).
    pub(super) fn forget(&mut self, vk: u16, seq: u64) {
        self.due_holds.retain(|h| !(h.vk == vk && h.seq == seq));
        let Some(index) = self.presses.iter().position(|p| p.vk == vk && p.seq == seq) else {
            return;
        };
        if let Phase::Held { tap, hold, double } = &mut self.presses[index].phase {
            *hold = None;
            if tap.is_none() && double.is_none() {
                self.presses.remove(index);
            }
        }
    }

    /// Drops every pending press and due Hold. The engine starts a new
    /// generation with it, so a timer started before the clear matches
    /// nothing armed after it (KS-11).
    pub(super) fn clear(&mut self) {
        self.presses.clear();
        self.due_holds.clear();
    }

    /// Whether the press of `vk` is held, and with a Tap, a Hold and a
    /// DoubleTap pending.
    #[cfg(test)]
    fn held(&self, vk: u16) -> Option<(bool, bool, bool)> {
        self.presses
            .iter()
            .find(|p| p.vk == vk)
            .and_then(|p| match &p.phase {
                Phase::Held { tap, hold, double } => {
                    Some((tap.is_some(), hold.is_some(), double.is_some()))
                }
                Phase::AwaitSecond { .. } => None,
            })
    }

    /// Whether the press of `vk` waits for its DoubleTap window, and with
    /// its Tap.
    #[cfg(test)]
    fn awaiting(&self, vk: u16) -> Option<bool> {
        self.presses
            .iter()
            .find(|p| p.vk == vk)
            .and_then(|p| match &p.phase {
                Phase::AwaitSecond { deferred_tap, .. } => Some(deferred_tap.is_some()),
                Phase::Held { .. } => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    const LONG: Duration = Duration::from_secs(10);
    const F13: u16 = VK_F13.0;
    const F14: u16 = VK_F14.0;

    /// The keys, `when:` and action the groups of these tests borrow.
    struct Fixture {
        keys: KeyCombination,
        condition: Condition,
        action: BindAction,
    }

    fn fixture() -> Fixture {
        Fixture {
            keys: KeyCombination::new(VK_F13),
            condition: Condition::Always,
            action: BindAction::SwitchLanguage,
        }
    }

    /// A group with the given gestures, each firing `action`.
    fn group<'a>(
        keys: &'a KeyCombination,
        condition: &'a Condition,
        terms: [Option<Duration>; 3],
        action: &'a BindAction,
    ) -> GroupArming<'a> {
        let [tap, hold, double_tap] = terms;
        GroupArming {
            keys,
            condition,
            tap: tap.map(|term| (term, action)),
            hold: hold.map(|term| (term, action)),
            double_tap: double_tap.map(|term| (term, action)),
            block: true,
        }
    }

    #[test]
    fn a_tap_alone_fires_at_a_key_up_before_its_term() {
        let f = fixture();
        let tap = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, None],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        assert_eq!(state.press(F13, Some(&tap), t0, 1, &mut fire), None);
        assert_eq!(state.held(F13), Some((true, false, false)));
        let released = state.release(F13, t0 + ms(199), false, 2);
        assert_eq!(released.fired, Some(BindAction::SwitchLanguage));
        assert!(released.gesture_bound);
        assert_eq!(released.window_due, None);
        assert_eq!(state.held(F13), None, "the press is resolved");

        state.press(F13, Some(&tap), t0 + ms(300), 3, &mut fire);
        let late = state.release(F13, t0 + ms(500), false, 4);
        assert_eq!(late.fired, None, "released at its term: no Tap");
        assert!(!late.gesture_bound);
        assert!(fire.is_empty());
    }

    #[test]
    fn a_hold_is_cancelled_before_its_term_and_left_to_its_timer_after_it() {
        let f = fixture();
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(200)), None],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        assert_eq!(
            state.press(F13, Some(&hold), t0, 1, &mut fire),
            Some(t0 + ms(200))
        );
        assert!(state.is_live(F13, 1));
        let early = state.release(F13, t0 + ms(199), false, 2);
        assert!(
            early.gesture_bound,
            "a cancelled Hold still binds the key-up"
        );
        assert!(!state.is_live(F13, 1), "cancelled");
        state.on_timer(F13, 1, &mut fire);
        assert!(fire.is_empty(), "its timer finds nothing");

        state.press(F13, Some(&hold), t0 + ms(300), 3, &mut fire);
        let late = state.release(F13, t0 + ms(500), false, 4);
        assert!(late.gesture_bound);
        assert_eq!(late.fired, None, "a key-up never fires a Hold");
        assert!(state.is_live(F13, 3), "left to its timer");
        state.on_timer(F13, 3, &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
        assert!(!state.is_live(F13, 3), "once");
    }

    #[test]
    fn a_due_hold_does_not_block_the_next_press() {
        let f = fixture();
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(200)), None],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&hold), t0, 1, &mut fire);
        state.release(F13, t0 + ms(250), false, 2);
        assert_eq!(
            state.press(F13, Some(&hold), t0 + ms(260), 3, &mut fire),
            Some(t0 + ms(460))
        );
        state.on_timer(F13, 1, &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage], "the first press's Hold");
        assert!(state.is_live(F13, 3), "the second press waits for its own");
    }

    #[test]
    fn a_stale_timer_does_not_fire_a_newer_press() {
        // KS-11: press 1 is released early, press 2 arms a new Hold, then
        // press 1's timer wakes up.
        let f = fixture();
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(200)), None],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&hold), t0, 1, &mut fire);
        state.release(F13, t0 + ms(80), false, 2);
        state.press(F13, Some(&hold), t0 + ms(150), 3, &mut fire);
        state.on_timer(F13, 1, &mut fire);
        assert!(fire.is_empty());
        assert_eq!(state.held(F13), Some((false, true, false)));
        state.on_timer(F13, 3, &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
    }

    #[test]
    fn a_tap_and_a_hold_of_one_press_fire_exactly_one() {
        // KS-25 for every pair of terms: the Hold resolves the press once
        // its term has passed, whatever the Tap's term.
        let f = fixture();
        let t0 = Instant::now();
        for (tap, hold, release, tapped, held) in [
            (200, 200, 199, true, false),
            (200, 200, 200, false, true),
            (300, 200, 250, false, true),
            (100, 200, 150, false, false),
            (300, 200, 150, true, false),
        ] {
            let both = group(
                &f.keys,
                &f.condition,
                [Some(ms(tap)), Some(ms(hold)), None],
                &f.action,
            );
            let mut state = Gestures::new();
            let mut fire = Vec::new();
            state.press(F13, Some(&both), t0, 1, &mut fire);
            let released = state.release(F13, t0 + ms(release), false, 2);
            let case = format!("Tap({tap}) + Hold({hold}) released at {release}");
            assert_eq!(released.fired.is_some(), tapped, "{case}");
            assert_eq!(state.is_live(F13, 1), held, "{case}");
        }
    }

    #[test]
    fn a_double_tap_fires_on_a_second_press_of_its_group_within_its_window() {
        let f = fixture();
        let double = group(
            &f.keys,
            &f.condition,
            [None, None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&double), t0, 1, &mut fire);
        let released = state.release(F13, t0 + ms(50), false, 2);
        assert_eq!(released, Released::default(), "no Tap, no timer");
        assert_eq!(state.awaiting(F13), Some(false));
        assert_eq!(
            state.press(F13, Some(&double), t0 + ms(299), 3, &mut fire),
            None
        );
        assert_eq!(fire, [BindAction::SwitchLanguage]);
        assert_eq!(state.held(F13), None, "the second press is spent");
        assert_eq!(state.awaiting(F13), None);

        // After the window a second press is a first press again.
        fire.clear();
        state.press(F13, Some(&double), t0 + ms(400), 4, &mut fire);
        state.release(F13, t0 + ms(450), false, 5);
        state.press(F13, Some(&double), t0 + ms(700), 6, &mut fire);
        assert!(fire.is_empty());
        assert_eq!(state.held(F13), Some((false, false, true)));
    }

    #[test]
    fn a_tap_with_a_double_tap_waits_for_the_window_and_its_timer_fires_it() {
        let f = fixture();
        let both = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&both), t0, 1, &mut fire);
        let released = state.release(F13, t0 + ms(80), false, 2);
        assert_eq!(released.fired, None, "the Tap waits");
        assert!(released.gesture_bound);
        assert_eq!(released.window_due, Some(t0 + ms(300)));
        assert_eq!(state.awaiting(F13), Some(true));
        assert!(state.is_live(F13, 2), "the window's timer");
        assert!(!state.is_live(F13, 1), "the press took a new number");

        state.on_timer(F13, 2, &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
        assert_eq!(state.awaiting(F13), None);
    }

    #[test]
    fn a_window_closes_for_a_press_another_group_wins() {
        // Known issue 74.
        let caps = KeyCombination::new(VK_CAPITAL);
        let shift_caps = KeyCombination::from_keys([VK_SHIFT, VK_CAPITAL]);
        let always = Condition::Always;
        let first = BindAction::SwitchLanguage;
        let second = BindAction::ToggleCapsLock;
        let plain = group(&caps, &always, [None, None, Some(ms(300))], &first);
        let shifted = group(&shift_caps, &always, [None, None, Some(ms(300))], &second);
        let t0 = Instant::now();
        let caps_vk = VK_CAPITAL.0;
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(caps_vk, Some(&plain), t0, 1, &mut fire);
        state.release(caps_vk, t0 + ms(40), false, 2);
        state.press(caps_vk, Some(&shifted), t0 + ms(80), 3, &mut fire);
        assert!(fire.is_empty(), "another group's press ends the window");
        state.release(caps_vk, t0 + ms(120), false, 4);
        state.press(caps_vk, Some(&shifted), t0 + ms(160), 5, &mut fire);
        assert_eq!(fire, [BindAction::ToggleCapsLock]);
    }

    #[test]
    fn a_press_won_by_no_group_fires_the_waiting_tap_and_closes_the_window() {
        let f = fixture();
        let both = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&both), t0, 1, &mut fire);
        state.release(F13, t0 + ms(80), false, 2);
        assert_eq!(state.press(F13, None, t0 + ms(100), 3, &mut fire), None);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
        assert_eq!(state.awaiting(F13), None);
        assert_eq!(state.held(F13), None);
    }

    #[test]
    fn another_key_down_fires_a_waiting_tap() {
        let f = fixture();
        let both = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let keys = KeyState::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&both), t0, 1, &mut fire);
        state.release(F13, t0 + ms(80), false, 2);
        state.other_key_down(F13, &keys, t0 + ms(90), &mut fire);
        assert!(fire.is_empty(), "its own key is not another key");
        state.other_key_down(F14, &keys, t0 + ms(100), &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
        assert_eq!(state.awaiting(F13), None);
    }

    #[test]
    fn another_key_down_cancels_a_passing_press_but_not_a_swallowed_one_or_a_due_hold() {
        // Owner decision (roadmap item 2.2, open question 10), and ruling
        // I1 for a Hold whose term has passed by the clock.
        let f = fixture();
        let tap_hold = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), Some(ms(300)), None],
            &f.action,
        );
        let double = group(
            &f.keys,
            &f.condition,
            [None, None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut passed = KeyState::new();
        passed.press(VK_F13);
        passed.mark_passed(F13);
        let mut swallowed = KeyState::new();
        swallowed.press(VK_F13);
        swallowed.mark_blocked(F13);
        let mut fire = Vec::new();

        let mut state = Gestures::new();
        state.press(F13, Some(&tap_hold), t0, 1, &mut fire);
        state.other_key_down(F14, &swallowed, t0 + ms(50), &mut fire);
        assert_eq!(
            state.held(F13),
            Some((true, true, false)),
            "swallowed: kept"
        );
        state.other_key_down(F14, &passed, t0 + ms(50), &mut fire);
        assert_eq!(state.held(F13), None, "passed: cancelled");
        assert!(!state.is_live(F13, 1), "and its Hold's timer is stale");

        let mut state = Gestures::new();
        state.press(F13, Some(&tap_hold), t0, 1, &mut fire);
        state.other_key_down(F14, &passed, t0 + ms(300), &mut fire);
        assert!(
            state.is_live(F13, 1),
            "a Hold due by the clock stays for its late timer"
        );

        let mut state = Gestures::new();
        state.press(F13, Some(&double), t0, 1, &mut fire);
        state.release(F13, t0 + ms(50), false, 2);
        state.other_key_down(F14, &KeyState::new(), t0 + ms(60), &mut fire);
        assert_eq!(state.awaiting(F13), Some(false), "swallowed window: kept");
        let mut state = Gestures::new();
        state.press(F13, Some(&double), t0, 1, &mut fire);
        state.release(F13, t0 + ms(50), true, 2);
        state.other_key_down(F14, &KeyState::new(), t0 + ms(60), &mut fire);
        assert_eq!(state.awaiting(F13), None, "passing window: cancelled");
        assert!(fire.is_empty(), "nothing fires");
    }

    #[test]
    fn a_forgotten_hold_never_fires_and_a_forgotten_window_keeps_its_tap() {
        let f = fixture();
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(200)), None],
            &f.action,
        );
        let both = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, Some(ms(300))],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&hold), t0, 1, &mut fire);
        state.forget(F13, 1);
        assert_eq!(state.held(F13), None, "a press with nothing left is gone");
        let released = state.release(F13, t0 + ms(300), false, 2);
        assert_eq!(released, Released::default());
        state.on_timer(F13, 1, &mut fire);
        assert!(fire.is_empty());

        state.press(F13, Some(&both), t0 + ms(400), 3, &mut fire);
        state.release(F13, t0 + ms(450), false, 4);
        state.forget(F13, 4);
        assert_eq!(state.awaiting(F13), Some(true), "the waiting Tap stays");
        state.other_key_down(F14, &KeyState::new(), t0 + ms(800), &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage]);
    }

    #[test]
    fn a_timer_fires_the_taps_whose_windows_closed_before_it() {
        let f = fixture();
        let both = group(
            &f.keys,
            &f.condition,
            [Some(ms(200)), None, Some(ms(300))],
            &f.action,
        );
        let hold_action = BindAction::DoNothing;
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(500)), None],
            &hold_action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F14, Some(&hold), t0, 1, &mut fire);
        state.press(F13, Some(&both), t0 + ms(10), 2, &mut fire);
        state.release(F13, t0 + ms(90), false, 3);
        state.forget(F13, 3);
        state.on_timer(F14, 1, &mut fire);
        assert_eq!(fire, [BindAction::SwitchLanguage, BindAction::DoNothing]);
    }

    #[test]
    fn a_held_press_left_by_a_lost_key_up_is_replaced() {
        let f = fixture();
        let hold = group(
            &f.keys,
            &f.condition,
            [None, Some(ms(100)), None],
            &f.action,
        );
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F13, Some(&hold), t0, 1, &mut fire);
        assert_eq!(
            state.press(F13, Some(&hold), t0 + ms(50), 2, &mut fire),
            Some(t0 + ms(150)),
            "before the term: the stale Hold is dropped"
        );
        assert!(!state.is_live(F13, 1));
        assert_eq!(
            state.press(F13, Some(&hold), t0 + ms(300), 3, &mut fire),
            Some(t0 + ms(400))
        );
        assert!(
            state.is_live(F13, 2),
            "due by the clock: the stale Hold stays for its timer"
        );
        assert!(state.is_live(F13, 3));
        assert!(fire.is_empty());
    }

    #[test]
    fn clear_drops_every_press_and_due_hold() {
        let f = fixture();
        let all = group(
            &f.keys,
            &f.condition,
            [Some(LONG), Some(LONG), Some(LONG)],
            &f.action,
        );
        let hold = group(&f.keys, &f.condition, [None, Some(ms(10)), None], &f.action);
        let t0 = Instant::now();
        let mut state = Gestures::new();
        let mut fire = Vec::new();

        state.press(F14, Some(&hold), t0, 1, &mut fire);
        state.release(F14, t0 + ms(20), false, 2);
        state.press(F13, Some(&all), t0, 3, &mut fire);
        assert!(state.is_live(F14, 1), "a due Hold");
        state.clear();
        assert_eq!(state.held(F13), None);
        assert!(!state.is_live(F14, 1));
        assert!(!state.is_live(F13, 3));
    }
}
