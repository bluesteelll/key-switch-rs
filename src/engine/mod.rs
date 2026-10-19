//! The decision core: given one keyboard event, the binding list and an
//! [`Env`], decide whether to swallow the event, which actions to run and
//! which timers to start. The engine executes nothing itself.
//!
//! # Architecture
//!
//! ```text
//!   Win32 WH_KEYBOARD_LL               main loop, gesture timer due
//!          |                                       |
//!          v                                       v
//!   hook::keyboard_hook_callback (adapter)   Engine::on_timer(token)
//!     - drops negative codes and the daemon's own injected events
//!     - KBDLLHOOKSTRUCT -> KeyEvent
//!     - Engine::on_key(event, &bindings, &env) -> Decision
//!     - hands Decision::timers to the main thread's gesture timers
//!       (Engine::start_timers), sends the menu mask key if
//!       Decision::mask, runs the fast actions of Decision::fire and
//!       queues the others for the action executor, then returns
//!       LRESULT(1) if Decision::suppress, else CallNextHookEx
//! ```
//!
//! The engine is plain Rust with no Win32 calls, no threads, no locks and
//! no clock of its own. Everything from outside comes in through the
//! [`Env`] trait: the time of the event, the operating system's view of the
//! keys held and the foreground window for `when:` conditions; and the
//! timers it asks for go out through the [`Scheduler`] trait. Because of
//! that, the whole decision path runs under test with a fake `Env` and a
//! [`TimerQueue`] (`testing::Scenario`), from a RON config and a script of
//! key events.
//!
//! What keeps it pure: `#![forbid(unsafe_code)]` below rules out direct
//! FFI, because every Win32 call through the `windows` crate is `unsafe`.
//! It does not rule out safe wrappers that do Win32 work or read the clock,
//! such as `ForegroundContext::capture`, `FastAction::run` or
//! `Instant::now()`. Outside tests the engine calls none of them, and
//! review keeps it that way. The engine uses [`BindAction`] only as data
//! (actions to fire, cloned into pending gestures) and never executes one.
//!
//! ## State
//!
//! [`Engine`] owns all mutable state of the keyboard path, as plain data:
//!
//! - `keys` (`keys::KeyState`): the keys the engine believes are held
//!   (also the auto-repeat detector), and whether each key's latest
//!   key-down was suppressed or passed, so that its auto-repeats and key-up
//!   are treated the same way. A foreground change marks the suppressed
//!   ones as forgotten: their key-ups are no longer due to be suppressed.
//! - `gestures` (`gestures::Gestures`): the pending press of each key, one
//!   machine for its `Tap`, `Hold` and `DoubleTap` (roadmap item 2.2), and
//!   the Holds left to a late timer.
//! - `chords` and `sequences`: pending `Chord` and `Sequence` entries.
//! - `mask_owed`: a `MaskOnly` auto-blocker matched the last key-down, so
//!   the next key event asks for the menu mask key too.
//!
//! [`Engine::reset`] (`swch off`, `swch on`) drops every pending entry and
//! an owed mask, starts a new timer generation, and takes the held keys
//! from the OS key state.
//!
//! The binding list is not engine state. The caller passes a snapshot on
//! every call, already sorted with [`sort_by_specificity`], so a hot reload
//! never has to touch the engine. Pending gestures keep clones of the
//! actions they were armed with and fire those, even after a reload.
//!
//! ## Time
//!
//! Deadlines are `Instant`s derived from [`Env::now`], stored in the pending
//! entries. `Tap`, `DoubleTap`, `Chord` and `Sequence` deadlines are checked
//! lazily: the next event compares its time with the stored deadline. A
//! `Chord` or `Sequence` entry also asks for an expiry timer, which frees it
//! on time and fires nothing (roadmap item 2.2); the lazy check stays the
//! authority, so a late or missing expiry timer changes no outcome.
//! `Hold` must fire while no event arrives, so arming one returns a
//! [`TimerRequest`]; the caller starts a timer and hands the
//! [`TimerToken`] back to [`Engine::on_timer`] when it is due, through a
//! [`Scheduler`] ([`Engine::start_timers`]). Every token carries the number
//! of its arming and the engine's generation, which each reset moves on,
//! and a token whose entry is gone, was re-armed or predates a reset is
//! stale ([`Engine::is_live`]) and does nothing. A late timer from an
//! earlier press therefore cannot fire or delete a newer press (KS-11).
//!
//! A timer can also wake late relative to the key-up. A key-up before a
//! Hold's deadline cancels it; a key-up at or after the deadline does not,
//! because the key was held for the whole term: the entry stays armed and
//! its timer fires it whenever it runs. Only [`Engine::on_timer`] ever fires
//! a Hold. Of the Tap, the Hold and the DoubleTap of one press exactly one
//! fires, whatever their terms (KS-25). A Tap that waits for its DoubleTap
//! window asks for a timer too, at the window's end.
//!
//! ## Threads
//!
//! The engine is single-threaded data and does not lock. The daemon keeps
//! it on its main thread (`hook::main_thread`, roadmap item 2.2), the
//! thread that installs the hooks and pumps their messages, in a
//! thread-local reached only through `with_main`. Every caller runs there:
//! both hook callbacks, the main loop's timers and `swch on` and `swch off`,
//! which call [`Engine::reset`]. `with_main` refuses a nested call on the
//! same thread (the event then passes) instead of panicking.
//!
//! The adapter's [`Env::now`] reads the clock once per event, and the main
//! loop hands a token to [`Engine::on_timer`] only once `Instant::now()` has
//! reached its [`TimerRequest::due`]. The engine compares `Instant`s, so a
//! key-up and the Hold timer of the same press are decided in clock order
//! whichever of the two the loop sees first, which is what makes Tap and
//! Hold with equal terms exclusive.
//!
//! ## Decision rules
//!
//! The rules are those of the hook callback before the engine existed,
//! except where a rule names the audit issue that changed it. On a key-down:
//!
//! 1. The modifier flags are resynced from [`Env::os_key_down`], skipping
//!    the key being delivered and every modifier whose key-down was
//!    suppressed, even one a foreground change forgot: Windows never saw
//!    that key go down, so it stays held for matching and its auto-repeats
//!    stay auto-repeats (KS-14).
//! 2. The key is marked held; if it already was, the event is an auto-repeat.
//! 3. Trigger keys are `KeySpec`s fixed at load time (`data::key_spec`):
//!    `Shift`, `Ctrl`, `Alt` and `Win` accept either key of the pair,
//!    `LShift`, `RWin` and the other sided names only that key. A key of a
//!    combo or sequence step is held if a key it accepts is held; a chord
//!    key is pressed if it accepts the event's code (KS-07).
//! 4. One `Combo` user binding wins the key-down (roadmap item 1.6, KS-13).
//!    A candidate's key just pressed must be one of its trigger keys, a
//!    non-modifier key of the combo or, for a combo made only of modifiers,
//!    any of its keys (KS-04); all of its keys must be held; the modifier
//!    families held (Shift, Ctrl, Alt, Win, either key of a pair counting
//!    as its family) must be exactly the combo's own, unless the binding
//!    sets `allow_extra_modifiers` (owner decision); and its `when:` must
//!    hold, which is asked only once its keys match. The winner has the
//!    most keys; then a binding with a `when:` beats one without; then the
//!    earlier binding in the list, which is file order. Only the winner
//!    acts:
//!    - `Immediate`: it fires on a first key-down and, on an auto-repeat,
//!      only if it sets `repeat` (by default `PressKey`, `Media(VolumeUp)`
//!      and `Media(VolumeDown)`; owner decision, KS-10). The election runs
//!      on every key-down, auto-repeats included, so a winner that does not
//!      repeat still takes its repeats, and a lower binding never fires in
//!      its place.
//!    - `Tap`, `Hold` and `DoubleTap`: every deferred binding on the same
//!      keys (in any order) with the same `when:` (as written) that matches
//!      the press acts with the winner, as one group, so a Tap and a Hold
//!      on one key arm together, as one press that resolves to one of them
//!      (KS-25). Only the first binding of each trigger kind in the group
//!      acts. They ignore repeats and arm under the key just pressed (so
//!      under a key of the combo, whose key-up resolves them). A first
//!      key-down also ends the DoubleTap window of its key: the DoubleTap
//!      fires if the same group, keys and `when:`, wins this key-down
//!      within the window, and the key-down is spent (known issue 74).
//!    - A first key-down of any key fires the Tap of another key that waits
//!      for its DoubleTap window, ahead of the new key, and cancels the
//!      pending gesture of another key whose key-down passed (owner
//!      decision, roadmap item 2.2, known issue 67): a key pressed during it
//!      makes it a shortcut, such as Ctrl+C. A Hold whose term has passed
//!      by the clock stays for its timer, so a late timer changes nothing.
//!      A gesture whose key-down was swallowed stays.
//!
//!    The winner, or any acting member of its group, with `block_default`
//!    asks for suppression; a losing binding never does. One without it
//!    lets the key-down through, and a gesture is armed and fires all the
//!    same (roadmap item 1.3). The loader resolves a missing
//!    `block_default` to `false` only for a deferred combo made only of
//!    modifiers. The other bindings are visited in list order, next to the
//!    winner:
//!    - An auto-blocker (`block_original_combo`) is a `Combo` of the
//!      Windows hotkey that only suppresses or masks: it fires nothing and
//!      is no candidate, so it never keeps a user binding from winning
//!      (KS-18). It matches when the key just pressed is one of the
//!      hotkey's, all of them are held, and every other key held is one
//!      that [`Env::os_key_down`] reports up: Windows would see the hotkey
//!      and nothing else (KS-05), and its `when:`, the `when:` of the
//!      bindings that asked for it OR'ed (KS-35), holds; that is asked only
//!      once the hotkey matches. A `Swallow` blocker (`true`) then asks for
//!      suppression. A `MaskOnly` blocker (experimental) lets the key-down
//!      through and, on a first key-down, asks for the menu mask key on
//!      this event and on the next key event, whatever its key.
//!    - `Sequence` and `Chord` check their `when:` first and ignore
//!      repeats; any key-down that starts, advances or completes one is
//!      suppressed, whichever `Combo` binding wins it.
//! 5. A first key-down is suppressed if anything asked for it, and the key
//!    is then marked blocked; a first key-down that passes is marked
//!    passed. An auto-repeat continues its key-down (KS-06): it is
//!    suppressed if its key is marked blocked, also after a foreground
//!    change, and passed if it is marked passed, whatever the bindings
//!    asked. A repeat without a mark (a modifier the resync found held,
//!    whose key-down the engine never saw) is suppressed if a binding asked
//!    for it; once one passes, the key is marked passed. Actions still fire
//!    on repeats as rule 4 says. A completed chord or sequence marks the
//!    keys whose key-ups it swallows as blocked, but only keys still held
//!    whose key-down did not pass (KS-30, KS-32).
//!
//! On a key-up the key is released and its press resolves. A Hold whose
//! deadline has passed resolves it: the press is left to the Hold's timer,
//! which fires it even if the timer runs after the key-up, and fires no Tap
//! (KS-25). Otherwise the Hold is cancelled, and a Tap whose deadline has
//! not passed fires now, or, if the press opens a DoubleTap window, waits
//! for the window on a timer of its own (roadmap item 2.2, open question
//! 5). A DoubleTap window opens if the key-up comes before it closes. The
//! key-up of a key marked passed always passes. Otherwise it is suppressed
//! if the key was marked blocked or if a Tap or Hold was pending. A
//! foreground change forgets which key-ups were due to be suppressed: such
//! a key-up passes unless a Tap or Hold of its press is pending (roadmap
//! item 2.3).
//!
//! On every key event, once its record is updated, the decision asks for
//! the menu mask key (`Decision::mask`, roadmap item 1.2, KS-12) if the
//! event is suppressed while an Alt or Win key is held whose key-down
//! Windows saw: one on record as passed, or one the resync found held with
//! no record. Windows then has a lone Alt or Win press so far, whose release
//! would activate the menu bar or open the Start menu; the mask key comes
//! between. An Alt or Win whose key-down was swallowed, this event's
//! included, does not count. A `MaskOnly` auto-blocker also asks for the
//! mask on events that pass (rule 4). A timer never asks for the mask.
//!
//! The known defects of these rules are listed in the audit and are fixed in
//! later roadmap items, each with a scenario test: the sequence matching
//! rules (KS-15, KS-16).

#![forbid(unsafe_code)]

mod chords;
mod env;
mod gestures;
mod keys;
mod matching;
mod schedule;
mod sequences;
mod winner;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod scenarios;

#[cfg(test)]
mod properties;

use std::cmp::Reverse;
use std::time::Instant;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;
use crate::data::binding::{Binding, BindingKind, BlockerMode};
use crate::data::trigger::Trigger;

pub use env::Env;
pub use schedule::{Scheduler, TimerQueue};

use chords::{ChordOutcome, Chords};
use env::EnvForeground;
use gestures::Gestures;
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

/// Identifies the pending entry a timer was started for. Opaque: the caller
/// only hands it back to [`Engine::on_timer`] and asks [`Engine::is_live`]
/// about it.
///
/// `generation` counts [`Engine::reset`] calls, `slot` names the table the
/// entry is in, and `seq` is the engine-wide number of the arming, so two
/// armings never share a token, across resets included (KS-11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerToken {
    generation: u32,
    slot: TimerSlot,
    seq: u64,
}

/// The table a timer's entry is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimerSlot {
    /// The gesture pending on the press of this virtual-key code.
    Gesture(u16),
    /// A pending chord's expiry.
    Chord,
    /// A pending sequence's expiry.
    Sequence,
}

/// Where the engine's tokens come from.
#[derive(Default)]
struct Ids {
    /// How many resets there were, wrapping.
    generation: u32,
    /// The `seq` of the latest arming, wrapping; never reset.
    last_seq: u64,
}

impl Ids {
    /// A number no earlier arming has.
    fn next_seq(&mut self) -> u64 {
        self.last_seq = self.last_seq.wrapping_add(1);
        self.last_seq
    }

    /// The timer for the entry armed in `slot` as `seq`, due at `due`.
    fn request(&self, slot: TimerSlot, seq: u64, due: Instant) -> TimerRequest {
        TimerRequest {
            token: TimerToken {
                generation: self.generation,
                slot,
                seq,
            },
            due,
        }
    }
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
    /// Send the menu mask key before the hook returns (roadmap item 1.2,
    /// KS-12), so that Windows does not see a lone Win or Alt press. Set on
    /// a key event that is suppressed while Windows has an Alt or Win key
    /// down whose key-down it saw, and by a `MaskOnly` auto-blocker on the
    /// key-down that completes its hotkey and on the next key event, which
    /// both pass (KS-05, experimental). Always `false` for a timer.
    pub mask: bool,
}

/// Sorts a binding list into the order the engine visits it: more keys
/// first. The sort is stable, so bindings with the same key count keep
/// their config order, and the auto-blockers the loader appends stay after
/// the file's bindings. Which `keys` binding wins a key-down is decided by
/// an explicit rank, not by this order alone; the order breaks ties
/// between bindings of equal rank (file order, roadmap item 1.6).
pub fn sort_by_specificity(bindings: &mut [Binding]) {
    bindings.sort_by_key(|b| Reverse(b.kind.key_count()));
}

/// The decision core. See the [module documentation](self).
pub struct Engine {
    keys: KeyState,
    gestures: Gestures,
    chords: Chords,
    sequences: Sequences,
    /// A `MaskOnly` auto-blocker matched the last key-down: the next key
    /// event asks for the menu mask key too.
    mask_owed: bool,
    ids: Ids,
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
            mask_owed: false,
            ids: Ids::default(),
        }
    }

    /// Decides one keyboard event. `bindings` must be sorted with
    /// [`sort_by_specificity`].
    pub fn on_key(&mut self, event: KeyEvent, bindings: &[Binding], env: &dyn Env) -> Decision {
        // Owed by a `MaskOnly` blocker that matched the previous key-down.
        // This event comes after that key-down reached Windows, whatever
        // the order of the mask sent with it, so its mask lands inside the
        // hotkey.
        let mask_owed = std::mem::take(&mut self.mask_owed);

        // Repair phantom-held modifiers from any earlier focus transition
        // before matching. The key being delivered is skipped: its OS state
        // lags until the hook returns. So is every modifier whose key-down
        // was swallowed: the OS never saw it go down (KS-14).
        self.keys.sync_modifiers(env, Some(event.vk));

        let mut decision = match event.direction {
            KeyDirection::Down => self.on_key_down(event.vk, bindings, env),
            KeyDirection::Up => self.on_key_up(event.vk, env),
        };
        decision.mask |= mask_owed;
        decision
    }

    /// Decides a timer that the caller started for a [`TimerRequest`]. A
    /// token whose entry was cancelled, re-armed or reset fires nothing.
    pub fn on_timer(&mut self, token: TimerToken) -> Decision {
        let mut decision = Decision::default();
        if token.generation != self.ids.generation {
            return decision;
        }
        match token.slot {
            TimerSlot::Gesture(vk) => self.gestures.on_timer(vk, token.seq, &mut decision.fire),
            // An expiry fires nothing: it frees the entry on time.
            TimerSlot::Chord => self.chords.expire(token.seq),
            TimerSlot::Sequence => self.sequences.expire(token.seq),
        }
        decision
    }

    /// Whether an entry still waits for the timer of `token`: one from this
    /// generation, armed as that `seq`. A stale token does nothing in
    /// [`on_timer`](Self::on_timer), so a caller may drop it from its queue.
    pub fn is_live(&self, token: TimerToken) -> bool {
        token.generation == self.ids.generation
            && match token.slot {
                TimerSlot::Gesture(vk) => self.gestures.is_live(vk, token.seq),
                TimerSlot::Chord => self.chords.is_live(token.seq),
                TimerSlot::Sequence => self.sequences.is_live(token.seq),
            }
    }

    /// Hands the timers of a decision to `scheduler`, after dropping every
    /// timer it holds that is no longer live, so that it keeps no stale
    /// timer for long. A timer the scheduler refuses is forgotten
    /// ([`forget_timer`](Self::forget_timer)). Returns how many Holds were
    /// dropped with the refused timers: a refused window or expiry timer
    /// drops nothing.
    pub fn start_timers(
        &mut self,
        timers: Vec<TimerRequest>,
        scheduler: &mut dyn Scheduler,
    ) -> u32 {
        if timers.is_empty() {
            return 0;
        }
        scheduler.retain(&|token| self.is_live(token));
        let mut dropped = 0;
        for request in timers {
            if !scheduler.start(request) && self.forget_timer(request.token) {
                dropped += 1;
            }
        }
        dropped
    }

    /// The caller could not start the timer for `token`. A Hold is dropped:
    /// it never fires, and its key can arm a new Hold on the next press
    /// (KS-33). A Tap that waits for its DoubleTap window is kept: it fires
    /// at the next key-down, or with the next timer handed back after its
    /// window closed. Returns whether a Hold was dropped.
    pub fn forget_timer(&mut self, token: TimerToken) -> bool {
        if token.generation != self.ids.generation {
            return false;
        }
        match token.slot {
            TimerSlot::Gesture(vk) => self.gestures.forget(vk, token.seq),
            // The next key-down drops an expired entry anyway.
            TimerSlot::Chord | TimerSlot::Sequence => false,
        }
    }

    /// The foreground window changed. Key-ups may have been lost while a
    /// higher-integrity window had focus: resync the modifiers from the OS
    /// and forget which key-downs were swallowed, so that no key-up stays
    /// due to be swallowed. Keys whose key-down passed keep that record.
    /// A swallowed key-down stays on record as not seen by Windows, so the
    /// key's auto-repeats stay swallowed.
    ///
    /// The resync skips a modifier whose key-down was swallowed: Windows
    /// never saw it go down, so its key state says nothing (KS-14). Such a
    /// key whose key-up was lost stays held for the engine until its next
    /// press, which reads as an auto-repeat and is swallowed, and its
    /// release (roadmap item 2.3 owns that repair).
    pub fn on_foreground_changed(&mut self, env: &dyn Env) {
        self.keys.sync_modifiers(env, None);
        self.keys.forget_blocked();
    }

    /// Remapping was switched off or on (`swch off`, `swch on`; KS-31).
    ///
    /// Every pending Tap, Hold, DoubleTap, chord and sequence is dropped, so
    /// none completes across the switch, and the reset starts a new
    /// generation: a timer started before it is stale ([`is_live`](Self::is_live))
    /// and fires nothing, also against an entry armed after it (KS-11).
    ///
    /// The held keys are taken from [`Env::os_key_down`], each on record as
    /// passed: while remapping was off the engine saw no event and Windows
    /// saw every key-down, so a key still held keeps its auto-repeats and
    /// its key-up. A key whose key-down the engine swallowed before `off`
    /// reads as up, since Windows never saw it go down.
    ///
    /// Call it only when the switch really changes: a repeated `swch on`
    /// would wipe the records of keys held and swallowed right now, and
    /// their key-ups would reach Windows.
    pub fn reset(&mut self, env: &dyn Env) {
        self.gestures.clear();
        self.chords = Chords::new();
        self.sequences = Sequences::new();
        self.mask_owed = false;
        self.keys.reset_from_os(env);
        self.ids.generation = self.ids.generation.wrapping_add(1);
    }

    fn on_key_down(&mut self, vk: VIRTUAL_KEY, bindings: &[Binding], env: &dyn Env) -> Decision {
        let now = env.now();
        let was_repeat = self.keys.press(vk);
        let foreground = EnvForeground(env);

        // One `keys` binding, or its gesture group, acts on this key-down;
        // the others neither fire, arm nor ask for suppression (roadmap item
        // 1.6, KS-13). Elected on every key-down, auto-repeats included, so
        // a less specific binding never fires in the winner's place.
        let winner = winner::elect(vk, bindings, &self.keys, &foreground);

        let mut decision = Decision::default();
        if !was_repeat {
            // A Tap of another key that waits for its DoubleTap window has
            // completed: it fires now, ahead of this key. A pending gesture
            // of another key whose key-down passed is cancelled, since this
            // key makes it a shortcut (owner decision, roadmap item 2.2).
            self.gestures
                .other_key_down(vk, &self.keys, now, &mut decision.fire);

            // A deferred winner's group arms one press, which resolves to
            // one of its gestures (KS-25). The press also ends this key's
            // DoubleTap window, completing it if the same group wins it.
            let group = winner
                .as_ref()
                .and_then(|winner| winner.deferred_group(vk, bindings, &self.keys));
            let seq = self.ids.next_seq();
            if let Some(due) =
                self.gestures
                    .press(vk.0, group.as_ref(), now, seq, &mut decision.fire)
            {
                decision
                    .timers
                    .push(self.ids.request(TimerSlot::Gesture(vk.0), seq, due));
            }
            if group.is_some_and(|group| group.block) {
                decision.suppress = true;
            }
        }

        for (index, binding) in bindings.iter().enumerate() {
            match &binding.kind {
                BindingKind::Combo(combo) => {
                    if let Some(mode) = binding.auto_blocker {
                        // A blocker only keeps a Windows hotkey from
                        // Windows. It fires nothing and is no candidate for
                        // the election, so a user binding matching the same
                        // key-down fires wherever the blocker sits in the
                        // list (KS-18). Its `when:`, taken from the bindings
                        // that asked for it (KS-35), is asked only once the
                        // hotkey matches: on the hook thread it may capture
                        // the foreground window (KS-08). `Always` asks the
                        // Env nothing.
                        if matching::blocker_matches(combo, vk, &self.keys, |key| {
                            env.os_key_down(key)
                        }) && binding.condition.evaluate(&foreground)
                        {
                            match mode {
                                BlockerMode::Swallow => decision.suppress = true,
                                // The hotkey reaches Windows with the mask
                                // key inside it, now and on the next event
                                // (KS-05, experimental). A repeat changes
                                // nothing Windows has seen.
                                BlockerMode::MaskOnly if !was_repeat => {
                                    decision.mask = true;
                                    self.mask_owed = true;
                                }
                                BlockerMode::MaskOnly => {}
                            }
                        }
                        continue;
                    }
                    if !winner
                        .as_ref()
                        .is_some_and(|w| w.admits(index, binding, vk, &self.keys))
                    {
                        continue;
                    }
                    match binding.trigger {
                        Trigger::Immediate => {
                            if !was_repeat || binding.repeat {
                                decision.fire.push(binding.action.clone());
                            }
                            if binding.block_default {
                                decision.suppress = true;
                            }
                        }
                        // The winner's gesture group acted above, as one
                        // press; its repeats arm nothing.
                        Trigger::Tap(_) | Trigger::Hold(_) | Trigger::DoubleTap(_) => {}
                    }
                }
                BindingKind::Sequence { steps, max_gap } => {
                    // The condition is checked before the feed, which
                    // changes the sequence's state.
                    if was_repeat || !binding.condition.evaluate(&foreground) {
                        continue;
                    }
                    let seq = self.ids.next_seq();
                    match self.sequences.on_key_down(
                        &self.keys,
                        &binding.action,
                        steps,
                        *max_gap,
                        now,
                        seq,
                    ) {
                        SequenceOutcome::NotMatching => {}
                        SequenceOutcome::Advanced { due } => {
                            decision.suppress = true;
                            decision
                                .timers
                                .push(self.ids.request(TimerSlot::Sequence, seq, due));
                        }
                        SequenceOutcome::Completed {
                            action,
                            last_step_keys,
                        } => {
                            decision.suppress = true;
                            decision.fire.push(action);
                            for key in last_step_keys {
                                self.keys.mark_completed(key);
                            }
                        }
                    }
                }
                BindingKind::Chord { keys, window } => {
                    if was_repeat || !binding.condition.evaluate(&foreground) {
                        continue;
                    }
                    let seq = self.ids.next_seq();
                    match self
                        .chords
                        .on_key_down(vk, &binding.action, keys, *window, now, seq)
                    {
                        ChordOutcome::NotInChord => {}
                        ChordOutcome::Pending => decision.suppress = true,
                        ChordOutcome::Started { due } => {
                            decision.suppress = true;
                            decision
                                .timers
                                .push(self.ids.request(TimerSlot::Chord, seq, due));
                        }
                        ChordOutcome::Completed { action, keys } => {
                            decision.suppress = true;
                            decision.fire.push(action);
                            for key in keys {
                                self.keys.mark_completed(key);
                            }
                        }
                    }
                }
            }
        }

        if was_repeat {
            // An auto-repeat continues its key-down: it is suppressed exactly
            // when the key-down was, whatever the bindings asked this time.
            // Otherwise a binding that stops matching mid-hold leaks repeats
            // after a swallowed key-down, and one that starts matching (a
            // modifier released under exact matching) swallows the repeats
            // and the key-up of a key-down the OS saw (KS-06).
            //
            // A key-down that a foreground change forgot still counts as
            // swallowed here: Windows never saw it, so a repeat that reached
            // Windows would be the first key-down it sees, in whatever
            // window took the foreground.
            //
            // The resync can find a modifier held whose key-down the hook
            // never saw, so Windows has it down. A repeat without a record
            // keeps what the bindings asked. If it passes, the press is one
            // Windows saw: record that, so its later repeats and its key-up
            // pass too, and no completed chord swallows that key-up.
            if self.keys.is_blocked(vk.0) {
                decision.suppress = true;
            } else if self.keys.is_passed(vk.0) {
                decision.suppress = false;
            } else if !decision.suppress {
                self.keys.mark_passed(vk.0);
            }
        } else if decision.suppress {
            self.keys.mark_blocked(vk.0);
        } else {
            // This key-down reaches the OS, so its repeats and key-up must
            // too. Replace whatever an earlier press left on record, so this
            // key reads as one the OS saw go down.
            self.keys.mark_passed(vk.0);
        }
        self.mask_if_swallowed_under_alt_or_win(&mut decision);
        decision
    }

    fn on_key_up(&mut self, vk: VIRTUAL_KEY, env: &dyn Env) -> Decision {
        let now = env.now();
        self.keys.release(vk);
        let was_passed = self.keys.is_passed(vk.0);

        // Resolve the press of this key. Its Hold is cancelled if its term
        // has not ended, and otherwise the press is left to the Hold's
        // timer, which fires it (the key-up never does). Else its Tap fires
        // if its term has not ended, unless the press opens a DoubleTap
        // window: then the Tap waits for the window, on a timer of its own.
        let seq = self.ids.next_seq();
        let released = self.gestures.release(vk.0, now, was_passed, seq);
        let mut decision = Decision::default();
        decision.fire.extend(released.fired);
        if let Some(due) = released.window_due {
            decision
                .timers
                .push(self.ids.request(TimerSlot::Gesture(vk.0), seq, due));
        }

        // A key-up whose key-down reached Windows always reaches it too, or
        // the key stays held there. Otherwise it is suppressed if the
        // key-down is on record as suppressed, or if a Tap or Hold of the
        // press was pending: a focus change may have forgotten the record,
        // and the gesture is still evidence of a swallowed press.
        let was_blocked = self.keys.take_blocked(vk.0);
        decision.suppress = was_blocked || (released.gesture_bound && !was_passed);
        self.mask_if_swallowed_under_alt_or_win(&mut decision);
        decision
    }

    /// Asks for the menu mask key if `decision` swallows its event while
    /// Windows has an Alt or Win key down whose key-down it saw (KS-12).
    /// Call it after the event's record update, so that an Alt or Win
    /// key-down swallowed by this very event does not count.
    fn mask_if_swallowed_under_alt_or_win(&self, decision: &mut Decision) {
        if decision.suppress && self.keys.alt_or_win_down_in_windows() {
            decision.mask = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::data::condition::Condition;
    use crate::data::key_combination::KeyCombination;
    use crate::engine::{Scheduler, TimerQueue};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_F13, VK_F14, VK_F15, VK_J, VK_K, VK_LSHIFT, VK_MENU, VK_SHIFT,
    };

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

    /// An [`Env`] stopped at one instant with nothing held, which panics if
    /// the foreground window is asked for.
    struct ForegroundForbiddenEnv(Instant);

    impl Env for ForegroundForbiddenEnv {
        fn now(&self) -> Instant {
            self.0
        }
        fn os_key_down(&self, _vk: VIRTUAL_KEY) -> bool {
            false
        }
        fn foreground_app(&self) -> Option<&str> {
            panic!("the foreground was asked for although no binding with a when: matches this key")
        }
        fn foreground_title(&self) -> Option<&str> {
            panic!("the foreground was asked for although no binding with a when: matches this key")
        }
    }

    #[test]
    fn a_when_condition_is_asked_only_for_a_binding_whose_keys_match() {
        // On the hook thread a `when:` may capture the foreground window
        // (GetForegroundWindow, OpenProcess, GetWindowTextW): a binding on
        // other keys must not pay for it on every key press (KS-08).
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: SwitchLanguage, when: AppEquals("code.exe")),
                (keys: "F14", action: ToggleCapsLock),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();

        let decision = engine.on_key(
            KeyEvent::down(VK_F14),
            &bindings,
            &ForegroundForbiddenEnv(Instant::now()),
        );
        assert_eq!(decision.fire, vec![BindAction::ToggleCapsLock]);
        assert!(decision.suppress, "F14's binding swallows its key-down");
    }

    #[test]
    fn a_blocker_when_condition_is_asked_only_once_its_hotkey_matches() {
        // An auto-blocker takes the `when:` of the bindings that ask for it
        // (KS-35). Asked before its hotkey matches, it would capture the
        // foreground window on the hook thread for every key press (KS-08).
        let mut bindings = vec![
            Binding::new(KeyCombination::new(VK_F14), BindAction::ToggleCapsLock),
            Binding::new_auto_blocker(
                KeyCombination::from_keys([VK_MENU, VK_SHIFT]),
                BlockerMode::Swallow,
            )
            .with_condition(Condition::AppEquals("x.exe".to_owned())),
        ];
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();

        let decision = engine.on_key(
            KeyEvent::down(VK_F14),
            &bindings,
            &ForegroundForbiddenEnv(Instant::now()),
        );
        assert_eq!(decision.fire, vec![BindAction::ToggleCapsLock]);
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

    /// A [`Scheduler`] that refuses every timer, as the main thread's does
    /// when Windows cannot set its timer.
    struct Refusing;

    impl Scheduler for Refusing {
        fn start(&mut self, _request: TimerRequest) -> bool {
            false
        }
        fn retain(&mut self, _live: &dyn Fn(TimerToken) -> bool) {}
    }

    const HOLD_F13: &str =
        r#"( bindings: [ (keys: "F13", action: PressKey("LShift"), trigger: Hold(200)) ] )"#;

    fn hold_f13() -> Vec<Binding> {
        let mut bindings = crate::config::from_ron_str(HOLD_F13).expect("config parses");
        sort_by_specificity(&mut bindings);
        bindings
    }

    #[test]
    fn a_token_from_before_a_reset_is_stale() {
        // KS-11 across `swch off` / `swch on`: the reset starts a new
        // generation, so a timer started before it fires nothing, also
        // against a Hold armed after it.
        let t0 = Instant::now();
        let bindings = hold_f13();
        let mut engine = Engine::new();

        let before = engine
            .on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0))
            .timers[0]
            .token;
        assert!(engine.is_live(before));
        engine.reset(&FixedEnv(t0 + Duration::from_millis(50)));
        assert!(!engine.is_live(before), "the reset makes the token stale");
        assert_eq!(engine.on_timer(before), Decision::default());

        let after = engine
            .on_key(
                KeyEvent::down(VK_F13),
                &bindings,
                &FixedEnv(t0 + Duration::from_millis(100)),
            )
            .timers[0]
            .token;
        assert_ne!(before, after);
        assert!(engine.is_live(after));
        assert!(!engine.is_live(before));
        assert_eq!(
            engine.on_timer(before),
            Decision::default(),
            "the old token does not fire the new Hold"
        );
        assert_eq!(
            engine.on_timer(after).fire,
            [BindAction::PressKey(VK_LSHIFT)]
        );
    }

    #[test]
    fn a_cancelled_hold_leaves_its_token_stale() {
        let t0 = Instant::now();
        let bindings = hold_f13();
        let mut engine = Engine::new();
        let mut queue = TimerQueue::new();

        let mut down = engine.on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0));
        let token = down.timers[0].token;
        assert_eq!(
            engine.start_timers(std::mem::take(&mut down.timers), &mut queue),
            0
        );
        assert!(engine.is_live(token));

        let _ = engine.on_key(
            KeyEvent::up(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(50)),
        );
        assert!(!engine.is_live(token), "the key-up cancelled the Hold");
        assert_eq!(engine.on_timer(token), Decision::default());

        // The next arming prunes the stale timer from the queue.
        let mut again = engine.on_key(
            KeyEvent::down(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(100)),
        );
        engine.start_timers(std::mem::take(&mut again.timers), &mut queue);
        assert_eq!(queue.len(), 1, "only the live timer is queued");
    }

    #[test]
    fn a_timer_the_scheduler_refuses_is_forgotten() {
        // KS-33: a Hold whose timer cannot be set never fires and does not
        // keep its key from arming the next press.
        let t0 = Instant::now();
        let bindings = hold_f13();
        let mut engine = Engine::new();

        let mut down = engine.on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0));
        let token = down.timers[0].token;
        assert_eq!(
            engine.start_timers(std::mem::take(&mut down.timers), &mut Refusing),
            1,
            "one timer was refused"
        );
        assert!(!engine.is_live(token));

        let up = engine.on_key(
            KeyEvent::up(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(300)),
        );
        assert!(up.fire.is_empty(), "the forgotten Hold never fires");
        assert!(up.suppress, "the key-up follows its swallowed key-down");
        assert_eq!(engine.on_timer(token), Decision::default());

        let mut queue = TimerQueue::new();
        let mut again = engine.on_key(
            KeyEvent::down(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(400)),
        );
        let next = again.timers[0].token;
        assert_eq!(
            engine.start_timers(std::mem::take(&mut again.timers), &mut queue),
            0
        );
        assert!(
            engine.is_live(next),
            "the next press arms a Hold of its own"
        );
    }

    #[test]
    fn a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes() {
        // Orchestrator ruling M6: a Tap that waits for its DoubleTap window
        // and whose timer the scheduler refused is not dropped. It fires at
        // the next event, or, as here, at the next timer the caller hands
        // back once its window has closed by that timer's clock.
        let t0 = Instant::now();
        let at = |ms: u64| FixedEnv(t0 + Duration::from_millis(ms));
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: SwitchLanguage, trigger: Tap(200)),
                (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
                (keys: "F14", action: DoNothing, trigger: Hold(500)),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();
        let mut queue = TimerQueue::new();

        let mut hold = engine.on_key(KeyEvent::down(VK_F14), &bindings, &at(0));
        let hold_token = hold.timers[0].token;
        engine.start_timers(std::mem::take(&mut hold.timers), &mut queue);
        let _ = engine.on_key(KeyEvent::down(VK_F13), &bindings, &at(10));
        let mut up = engine.on_key(KeyEvent::up(VK_F13), &bindings, &at(90));
        assert!(up.fire.is_empty(), "the Tap waits for the window");
        assert_eq!(
            engine.start_timers(std::mem::take(&mut up.timers), &mut Refusing),
            0,
            "the window's timer is refused, and no Hold is dropped with it"
        );

        assert_eq!(
            engine.on_timer(hold_token).fire,
            [BindAction::SwitchLanguage, BindAction::DoNothing],
            "the Hold's timer at 500 ms fires the waiting Tap first"
        );
    }

    /// An [`Env`] stopped at one instant with nothing held, which records
    /// every key it is asked about.
    struct CountingEnv {
        now: Instant,
        asked: std::cell::RefCell<Vec<VIRTUAL_KEY>>,
    }

    impl CountingEnv {
        fn asked(&self, vk: VIRTUAL_KEY) -> usize {
            self.asked.borrow().iter().filter(|&&key| key == vk).count()
        }
    }

    impl Env for CountingEnv {
        fn now(&self) -> Instant {
            self.now
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            self.asked.borrow_mut().push(vk);
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
    fn only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key() {
        // Roadmap item 2.3, ruling R1: the witness costs one OS read, and
        // only at a key-up the engine would swallow. A key-up that passes
        // asks nothing.
        let bindings = loaded(
            r#"( bindings: [
                (keys: "F13", action: DoNothing),
                (keys: "F14", action: DoNothing, block_default: false),
            ] )"#,
        );
        let env = CountingEnv {
            now: Instant::now(),
            asked: std::cell::RefCell::new(Vec::new()),
        };
        let mut engine = Engine::new();

        for vk in [VK_F13, VK_F14] {
            let _ = engine.on_key(KeyEvent::down(vk), &bindings, &env);
            let _ = engine.on_key(KeyEvent::up(vk), &bindings, &env);
        }
        assert_eq!(
            env.asked(VK_F13),
            1,
            "a swallowed key-up asks the OS about its key once"
        );
        assert_eq!(env.asked(VK_F14), 0, "a key-up that passes asks nothing");
    }

    #[test]
    fn a_refused_window_timer_drops_no_gesture() {
        // Review m2 of roadmap item 2.2: the count that the adapter bumps
        // `hold_timer_not_started` with is the number of Holds dropped. A
        // Tap that waits for its DoubleTap window is kept when its timer is
        // refused, so it counts nothing.
        let t0 = Instant::now();
        let bindings = loaded(
            r#"( bindings: [
                (keys: "F13", action: SwitchLanguage, trigger: Tap(200)),
                (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
            ] )"#,
        );
        let mut engine = Engine::new();

        let _ = engine.on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0));
        let mut up = engine.on_key(
            KeyEvent::up(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(80)),
        );
        assert_eq!(
            up.timers.len(),
            1,
            "the waiting Tap asks for its window's timer"
        );
        assert_eq!(
            engine.start_timers(std::mem::take(&mut up.timers), &mut Refusing),
            0,
            "a refused window timer drops no gesture"
        );
    }

    fn loaded(ron: &str) -> Vec<Binding> {
        let mut bindings = crate::config::from_ron_str(ron).expect("config parses");
        sort_by_specificity(&mut bindings);
        bindings
    }

    const JK_CHORD: &str =
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#;

    #[test]
    fn a_chord_start_asks_for_a_timer_at_the_end_of_its_window() {
        // Roadmap item 2.2 (D8): a pending chord expires on its own timer.
        let t0 = Instant::now();
        let bindings = loaded(JK_CHORD);
        let mut engine = Engine::new();

        let decision = engine.on_key(KeyEvent::down(VK_J), &bindings, &FixedEnv(t0));
        let [timer] = decision.timers[..] else {
            panic!("one expiry timer, got {:?}", decision.timers)
        };
        assert_eq!(timer.due, t0 + Duration::from_millis(50));
        assert!(engine.is_live(timer.token));
    }

    #[test]
    fn a_chord_expiry_timer_drops_the_pending_chord() {
        let t0 = Instant::now();
        let bindings = loaded(JK_CHORD);
        let mut engine = Engine::new();

        let decision = engine.on_key(KeyEvent::down(VK_J), &bindings, &FixedEnv(t0));
        let [timer] = decision.timers[..] else {
            panic!("one expiry timer, got {:?}", decision.timers)
        };
        assert_eq!(
            engine.on_timer(timer.token),
            Decision::default(),
            "an expiry fires nothing"
        );
        assert!(!engine.is_live(timer.token), "the entry is gone");

        let _ = engine.on_key(
            KeyEvent::up(VK_J),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(55)),
        );
        let k = engine.on_key(
            KeyEvent::down(VK_K),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(60)),
        );
        assert!(k.fire.is_empty(), "K after the window starts a new chord");
        assert_eq!(k.timers.len(), 1, "with a timer of its own");
    }

    #[test]
    fn each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale() {
        let t0 = Instant::now();
        let at = |ms: u64| FixedEnv(t0 + Duration::from_millis(ms));
        let bindings = loaded(
            r#"( bindings: [ (sequence: (steps: ["F13", "F14", "F15"], max_gap_ms: 500), action: SwitchLanguage) ] )"#,
        );
        let mut engine = Engine::new();

        let first = engine.on_key(KeyEvent::down(VK_F13), &bindings, &at(0));
        let [start] = first.timers[..] else {
            panic!("one expiry timer, got {:?}", first.timers)
        };
        assert_eq!(start.due, t0 + Duration::from_millis(500));
        let _ = engine.on_key(KeyEvent::up(VK_F13), &bindings, &at(50));

        let second = engine.on_key(KeyEvent::down(VK_F14), &bindings, &at(100));
        let [advance] = second.timers[..] else {
            panic!("one expiry timer, got {:?}", second.timers)
        };
        assert_eq!(advance.due, t0 + Duration::from_millis(600));
        assert!(!engine.is_live(start.token), "the step took a new number");
        assert!(engine.is_live(advance.token));
        assert_eq!(
            engine.on_timer(start.token),
            Decision::default(),
            "the stale token neither fires nor drops the sequence"
        );
        let _ = engine.on_key(KeyEvent::up(VK_F14), &bindings, &at(150));

        let last = engine.on_key(KeyEvent::down(VK_F15), &bindings, &at(200));
        assert_eq!(last.fire, [BindAction::SwitchLanguage]);
        assert!(
            !engine.is_live(advance.token),
            "a completed sequence is gone"
        );
    }

    #[test]
    fn key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending() {
        // A Tap with `block_default: false` arms under a key-down that
        // passes (roadmap item 1.3; scenario
        // explicit_block_default_false_on_a_tap_passes_every_event_and_still_fires).
        // Windows then has the key down, so the key-up must reach it
        // whatever the gesture does. Here the Tap swallows its key-down and
        // the record is set to passed directly, so the key-up rule is
        // pinned on its own: it follows the record, not the binding.
        let t0 = Instant::now();
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();

        let down = engine.on_key(KeyEvent::down(VK_F13), &bindings, &FixedEnv(t0));
        assert!(down.suppress, "the Tap swallows its key-down");
        engine.keys.mark_passed(VK_F13.0);

        let up = engine.on_key(
            KeyEvent::up(VK_F13),
            &bindings,
            &FixedEnv(t0 + Duration::from_millis(50)),
        );
        assert_eq!(
            up.fire,
            vec![BindAction::SwitchLanguage],
            "the Tap resolves"
        );
        assert!(
            !up.suppress,
            "a key-up whose key-down reached Windows must reach it too"
        );
    }
}
