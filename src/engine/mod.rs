//! The decision core: given one keyboard event, the binding list and an
//! [`Env`], decide whether to swallow the event, which actions to run and
//! which timers to start. The engine executes nothing itself.
//!
//! # Architecture
//!
//! ```text
//!   Win32 WH_KEYBOARD_LL                          hook timer threads
//!          |                                              |
//!          v                                              v
//!   hook::keyboard_hook_callback (adapter) ---- Engine::on_timer(token)
//!     - drops negative codes and the daemon's own injected events
//!     - KBDLLHOOKSTRUCT -> KeyEvent
//!     - Engine::on_key(event, &bindings, &env) -> Decision
//!     - starts Decision::timers, runs Decision::fire, then returns
//!       LRESULT(1) if Decision::suppress, else CallNextHookEx
//! ```
//!
//! The engine is plain Rust with no Win32 calls, no threads, no locks and
//! no clock of its own. Everything from outside comes in through the
//! [`Env`] trait: the time of the event, the operating system's view of the
//! modifier keys and the foreground window for `when:` conditions. Because
//! of that, the whole decision path runs under test with a fake `Env`
//! (`testing::Scenario`), from a RON config and a script of key events.
//!
//! What keeps it pure: `#![forbid(unsafe_code)]` below rules out direct
//! FFI, because every Win32 call through the `windows` crate is `unsafe`.
//! It does not rule out safe wrappers that do Win32 work or read the clock,
//! such as `ForegroundContext::capture`, `BindAction::execute` or
//! `Instant::now()`. Outside tests the engine calls none of them, and
//! review keeps it that way. The engine uses [`BindAction`] only as data
//! (actions to fire, cloned into pending gestures) and never executes one.
//!
//! ## State
//!
//! [`Engine`] owns all mutable state of the keyboard path, as plain data:
//!
//! - `keys` (`keys::KeyState`): the keys the engine believes are held
//!   (also the auto-repeat detector), and the keys whose key-down was
//!   suppressed, so that their key-up is suppressed as well.
//! - `gestures` (`gestures::Gestures`): pending `Tap`, `Hold` and
//!   `DoubleTap` entries, keyed by virtual-key code.
//! - `chords` and `sequences`: pending `Chord` and `Sequence` entries.
//!
//! The binding list is not engine state. The caller passes a snapshot on
//! every call, already sorted with [`sort_by_specificity`], so a hot reload
//! never has to touch the engine. Pending gestures keep clones of the
//! actions they were armed with and fire those, even after a reload.
//!
//! ## Time
//!
//! Deadlines are `Instant`s derived from [`Env::now`], stored in the pending
//! entries. `Tap`, `DoubleTap`, `Chord` and `Sequence` entries expire
//! lazily: the next event compares its time with the stored deadline.
//! `Hold` must fire while no event arrives, so arming one returns a
//! [`TimerRequest`]; the caller starts a timer and hands the
//! [`TimerToken`] back to [`Engine::on_timer`] when it is due. Every token
//! carries the generation of the entry it was armed for, and a token whose
//! entry is gone or was re-armed does nothing. A late timer from an earlier
//! press therefore cannot fire or delete a newer press (KS-11).
//!
//! A timer can also wake late relative to the key-up. A key-up before a
//! Hold's deadline cancels it; a key-up at or after the deadline does not,
//! because the key was held for the whole term: the entry stays armed and
//! its timer fires it whenever it runs. Only [`Engine::on_timer`] ever fires
//! a Hold, so a caller that runs timer decisions off the hook thread keeps
//! every Hold action off it. With equal Tap and Hold terms on one key
//! exactly one of the two fires (KS-25).
//!
//! ## Threads and locking
//!
//! The engine is single-threaded data and does not lock. For now the hook
//! adapter keeps it behind a `Mutex` in `KeyboardHook`, reached only through
//! `KeyboardHook::with_engine`, because `Hold` timers still run on their own
//! short-lived `hold-timer` threads and call [`Engine::on_timer`] there; the
//! Hold action then runs on that timer thread, as before. The adapter holds
//! the lock only while the engine decides and runs every action after
//! releasing it. `with_engine` refuses a nested call on the same thread
//! (the event then passes) instead of deadlocking on the lock.
//!
//! The adapter's [`Env::now`] reads the clock inside the lock, and a timer
//! thread waits until `Instant::now()` has reached its [`TimerRequest::due`]
//! before it locks. A key-up and the Hold timer of the same press are
//! therefore decided in clock order, which is what makes Tap and Hold with
//! equal terms exclusive. Moving the timers onto the hook thread is roadmap
//! item 2.2.
//!
//! ## Decision rules
//!
//! The rules are those of the hook callback before the engine existed,
//! except where a rule names the audit issue that changed it. On a key-down:
//!
//! 1. The modifier flags are resynced from [`Env::os_key_down`], skipping
//!    the key being delivered.
//! 2. The key is marked held; if it already was, the event is an auto-repeat.
//! 3. Trigger keys are `KeySpec`s fixed at load time (`data::key_spec`):
//!    `Shift`, `Ctrl`, `Alt` and `Win` accept either key of the pair,
//!    `LShift`, `RWin` and the other sided names only that key. A key of a
//!    combo or sequence step is held if a key it accepts is held; a chord
//!    key is pressed if it accepts the event's code (KS-07).
//! 4. Every binding is visited in specificity order. Its `when:` condition is
//!    checked first, then its kind:
//!    - `Combo`: all of its keys must be held. `Immediate` fires only for the
//!      first matching Immediate binding, on every key-down including
//!      repeats, and suppresses if `block_default` is set. `Tap`, `Hold` and
//!      `DoubleTap` ignore repeats, arm under the key just pressed and always
//!      suppress.
//!    - `Sequence` and `Chord` ignore repeats; any key-down that starts,
//!      advances or completes one is suppressed.
//! 5. If anything asked for suppression, the key is marked blocked.
//!
//! On a key-up the key is released, a pending Tap fires if its deadline has
//! not passed, and a pending Hold is cancelled if its deadline has not
//! passed. A Hold whose deadline has passed is left armed for its timer,
//! which fires it even if the timer runs after the key-up (KS-25). The
//! key-up is suppressed if a Tap or Hold was pending or if the key was
//! marked blocked.
//!
//! The known defects of these rules are listed in the audit and are fixed in
//! later roadmap items, each with a scenario test: superset matching (KS-04),
//! leaking repeats (KS-06), repeat-firing Immediate actions (KS-10),
//! specificity across trigger kinds (KS-13), the modifier resync (KS-14) and
//! the sequence matching rules (KS-15, KS-16).

#![forbid(unsafe_code)]

mod chords;
mod env;
mod gestures;
mod keys;
mod matching;
mod sequences;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod scenarios;

use std::cmp::Reverse;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;
use crate::data::binding::{Binding, BindingKind};
use crate::data::trigger::Trigger;

pub use env::Env;

use chords::{ChordOutcome, Chords};
use env::EnvForeground;
use gestures::{Gestures, HoldRelease};
use keys::KeyState;
use sequences::{SequenceOutcome, Sequences};

/// Whether a key went down (including auto-repeats) or up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyDirection {
    Down,
    Up,
}

/// One keyboard event as the engine sees it: the virtual-key code the hook
/// reported (sided for modifiers, e.g. `VK_LSHIFT`) and its direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub vk: VIRTUAL_KEY,
    pub direction: KeyDirection,
}

impl KeyEvent {
    pub fn down(vk: VIRTUAL_KEY) -> Self {
        Self {
            vk,
            direction: KeyDirection::Down,
        }
    }

    pub fn up(vk: VIRTUAL_KEY) -> Self {
        Self {
            vk,
            direction: KeyDirection::Up,
        }
    }
}

/// Identifies the pending gesture a timer was started for. Opaque: the
/// caller only hands it back to [`Engine::on_timer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerToken {
    vk: u16,
    generation: u64,
}

/// A timer the caller must start: call [`Engine::on_timer`] with `token`
/// once `due` is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerRequest {
    pub token: TimerToken,
    pub due: Instant,
}

/// What to do about one event or timer. The engine has already updated its
/// own state; carrying out the rest is the caller's job.
#[derive(Debug, Default, PartialEq)]
#[must_use = "a Decision does nothing until the caller carries it out"]
pub struct Decision {
    /// Swallow the event (`true`) or pass it on to the next hook (`false`).
    /// Always `false` for a timer.
    pub suppress: bool,
    /// Actions to run, in order.
    pub fire: Vec<BindAction>,
    /// Timers to start.
    pub timers: Vec<TimerRequest>,
    /// Reserved for the menu mask key (roadmap item 1.2): send a mask key
    /// before the event so the OS does not see a lone Win or Alt tap. The
    /// engine does not set it yet.
    pub mask: bool,
}

/// Sorts a binding list into the order the engine visits it: more keys
/// first. The sort is stable, so bindings with the same key count keep
/// their config order (two bindings on one key with different `when:`
/// conditions rely on this).
pub fn sort_by_specificity(bindings: &mut [Binding]) {
    bindings.sort_by_key(|b| Reverse(b.kind.key_count()));
}

/// The decision core. See the [module documentation](self).
pub struct Engine {
    keys: KeyState,
    gestures: Gestures,
    chords: Chords,
    sequences: Sequences,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            keys: KeyState::new(),
            gestures: Gestures::new(),
            chords: Chords::new(),
            sequences: Sequences::new(),
        }
    }

    /// Decides one keyboard event. `bindings` must be sorted with
    /// [`sort_by_specificity`].
    pub fn on_key(&mut self, event: KeyEvent, bindings: &[Binding], env: &dyn Env) -> Decision {
        // Repair phantom-held modifiers from any earlier focus transition
        // before matching. The key being delivered is skipped: its OS state
        // lags until the hook returns.
        self.keys.sync_modifiers(env, Some(event.vk));

        match event.direction {
            KeyDirection::Down => self.on_key_down(event.vk, bindings, env),
            KeyDirection::Up => self.on_key_up(event.vk, env),
        }
    }

    /// Decides a timer that the caller started for a [`TimerRequest`]. A
    /// token whose gesture was cancelled or re-armed fires nothing.
    pub fn on_timer(&mut self, token: TimerToken) -> Decision {
        let mut decision = Decision::default();
        if let Some(action) = self.gestures.fire_hold(token) {
            decision.fire.push(action);
        }
        decision
    }

    /// The caller could not start the timer for `token` (for example, a
    /// thread could not be spawned). Its Hold is dropped: it never fires,
    /// and its key can arm a new Hold on the next press.
    pub fn forget_timer(&mut self, token: TimerToken) {
        self.gestures.forget_hold(token);
    }

    /// The foreground window changed. Key-ups may have been lost while a
    /// higher-integrity window had focus: resync every modifier from the OS
    /// and forget which key-ups were due to be swallowed.
    pub fn on_foreground_changed(&mut self, env: &dyn Env) {
        self.keys.sync_modifiers(env, None);
        self.keys.clear_all_blocked();
    }

    fn on_key_down(&mut self, vk: VIRTUAL_KEY, bindings: &[Binding], env: &dyn Env) -> Decision {
        let now = env.now();
        let was_repeat = self.keys.press(vk);
        let foreground = EnvForeground(env);

        let mut decision = Decision::default();
        let mut immediate_fired = false;

        for binding in bindings {
            // `Always` returns true without asking the Env, so a config
            // without `when:` never looks at the foreground window.
            if !binding.condition.evaluate(&foreground) {
                continue;
            }

            match &binding.kind {
                BindingKind::Combo(combo) => {
                    if !matching::keys_down(combo, &self.keys) {
                        continue;
                    }
                    match binding.trigger {
                        Trigger::Immediate => {
                            if !immediate_fired {
                                decision.fire.push(binding.action.clone());
                                immediate_fired = true;
                            }
                            if binding.block_default {
                                decision.suppress = true;
                            }
                        }
                        Trigger::Tap(term_ms) => {
                            if was_repeat {
                                continue;
                            }
                            self.gestures.arm_tap(
                                vk,
                                &binding.action,
                                Duration::from_millis(term_ms),
                                now,
                            );
                            decision.suppress = true;
                        }
                        Trigger::Hold(term_ms) => {
                            if was_repeat {
                                continue;
                            }
                            if let Some(timer) = self.gestures.arm_hold(
                                vk,
                                &binding.action,
                                Duration::from_millis(term_ms),
                                now,
                            ) {
                                decision.timers.push(timer);
                            }
                            decision.suppress = true;
                        }
                        Trigger::DoubleTap(term_ms) => {
                            if was_repeat {
                                continue;
                            }
                            if let Some(action) = self.gestures.double_tap(
                                vk,
                                &binding.action,
                                Duration::from_millis(term_ms),
                                now,
                            ) {
                                decision.fire.push(action);
                            }
                            decision.suppress = true;
                        }
                    }
                }
                BindingKind::Sequence { steps, max_gap } => {
                    if was_repeat {
                        continue;
                    }
                    match self.sequences.on_key_down(
                        &self.keys,
                        &binding.action,
                        steps,
                        *max_gap,
                        now,
                    ) {
                        SequenceOutcome::NotMatching => {}
                        SequenceOutcome::Advanced => decision.suppress = true,
                        SequenceOutcome::Completed {
                            action,
                            last_step_keys,
                        } => {
                            decision.suppress = true;
                            decision.fire.push(action);
                            for key in last_step_keys {
                                self.keys.mark_blocked(key);
                            }
                        }
                    }
                }
                BindingKind::Chord { keys, window } => {
                    if was_repeat {
                        continue;
                    }
                    match self
                        .chords
                        .on_key_down(vk, &binding.action, keys, *window, now)
                    {
                        ChordOutcome::NotInChord => {}
                        ChordOutcome::Pending => decision.suppress = true,
                        ChordOutcome::Completed { action, keys } => {
                            decision.suppress = true;
                            decision.fire.push(action);
                            for key in keys {
                                self.keys.mark_blocked(key);
                            }
                        }
                    }
                }
            }
        }

        if decision.suppress {
            self.keys.mark_blocked(vk.0);
        }
        decision
    }

    fn on_key_up(&mut self, vk: VIRTUAL_KEY, env: &dyn Env) -> Decision {
        let now = env.now();
        self.keys.release(vk);

        // Resolve every deferred gesture pending for this key. A Tap fires
        // if its term has not ended. A Hold is cancelled if its term has
        // not ended, and otherwise left to its timer, which fires it (the
        // key-up never does). Both can be pending at once on the same key:
        // they were armed independently and resolve independently.
        let tap = self.gestures.resolve_tap(vk, now);
        let tap_fired = tap.is_some();
        let hold = self.gestures.release_hold(vk, now);

        let mut decision = Decision::default();
        decision.fire.extend(tap);

        if tap_fired || hold != HoldRelease::NotPending {
            // Gesture-bound key: suppress the key-up too, and clear the
            // blocked flag its key-down set so it does not leak.
            self.keys.take_blocked(vk.0);
            decision.suppress = true;
        } else {
            decision.suppress = self.keys.take_blocked(vk.0);
        }
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_F13;

    /// An [`Env`] stopped at one instant with nothing held and no
    /// foreground window: enough to arm a Hold and read its timer token.
    struct FixedEnv(Instant);

    impl Env for FixedEnv {
        fn now(&self) -> Instant {
            self.0
        }
        fn os_key_down(&self, _vk: VIRTUAL_KEY) -> bool {
            false
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn forget_timer_makes_a_late_timer_fire_nothing() {
        // KS-33: the adapter calls `forget_timer` when it cannot spawn a
        // Hold's timer thread.
        let t0 = Instant::now();
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [ (keys: "F13", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();

        let decision = engine.on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0));
        let token = decision.timers[0].token;
        engine.forget_timer(token);

        assert_eq!(
            engine.on_timer(token),
            Decision::default(),
            "a timer the caller could not start but that wakes anyway must not fire the forgotten Hold"
        );
    }
}
