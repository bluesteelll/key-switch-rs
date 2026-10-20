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
//! such as `ForegroundCache::resync`, `FastAction::run` or
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
//!   are treated the same way. Nothing but the key's own next key-down or
//!   its key-up replaces a swallowed key-down's record: not a foreground
//!   change, not a reset (rule L, roadmap item 2.3).
//! - `gestures` (`gestures::Gestures`): the pending press of each key, one
//!   machine for its `Tap`, `Hold` and `DoubleTap` (roadmap item 2.2), and
//!   the Holds left to a late timer.
//! - `chords` and `sequences`: pending `Chord` and `Sequence` entries.
//! - `mask_owed`: a `MaskOnly` auto-blocker matched the last key-down, so
//!   the next key event asks for the menu mask key too.
//! - `resync` (`resync::Resync`): when the main thread reads the modifiers
//!   from the OS between key events (roadmap item 2.5). The engine follows
//!   Shift, Ctrl, Alt and Win from the key events it decides and from the
//!   daemon's own injected ones ([`Engine::track_injected`]); what can hide
//!   a key event from the hook (a foreground change, a reset, input leaving
//!   this desktop and coming back, a note of hidden input from the adapter)
//!   moves a hiding count, and the main thread's drain then reads the
//!   modifiers ([`Engine::resync_if_due`]). No key event reads the modifier
//!   state.
//!
//! [`Engine::reset`] drops every pending entry and an owed mask and starts
//! a new timer generation, for a [`ResetReason`]: `swch off` or `swch on`,
//! which also takes the held keys from the OS key state, a config reload,
//! which leaves the keys as they are, or keyboard input leaving this
//! desktop ([`InputSignal`]), which releases every key held (roadmap item
//! 2.3).
//!
//! The binding list is not engine state. The caller passes a snapshot on
//! every call, already sorted with [`sort_by_specificity`]. Pending
//! gestures keep clones of the actions they were armed with, so a reload
//! resets the engine in the same main-thread step that swaps the list
//! ([`ResetReason::Reloaded`], roadmap item 2.3): no gesture armed with the
//! old list fires after it.
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
//! both hook callbacks, the main loop's timers, and `swch on`, `swch off`,
//! config reloads and the input signals that the main loop serves, which
//! call [`Engine::reset`]. `with_main` refuses a nested call on the same thread
//! (the event then passes) instead of panicking.
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
//! 1. The modifier flags are the engine's own: each key event it decides
//!    sets or clears its key's flag, and so does each injected modifier
//!    event of the daemon. They are read from [`Env::os_key_down`] only
//!    between key events, by the main thread's drain, when something could
//!    have hidden a key event (roadmap item 2.5), skipping every modifier
//!    whose key-down was suppressed: Windows never saw that key go down, so
//!    it stays held for matching and its auto-repeats stay auto-repeats
//!    (KS-14). A generic code (`VK_SHIFT`) is held only between the
//!    key-down and the key-up a program sends under it.
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
//!    for it, and the key is then marked as a swallowed repeat, whose key-up
//!    is suppressed unless Windows has the key down (roadmap item 2.5);
//!    once one passes, the key is marked passed. Actions still fire
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
//! if the key was marked blocked or as a swallowed repeat, or if a Tap or
//! Hold was pending, unless
//! [`Env::os_key_down`] reports the key down before the event: Windows then
//! saw a key-down the hook did not, and the key-up passes so that the key
//! does not stay down there (the witness, roadmap item 2.3). A foreground
//! change or a reset keeps the record, so no key-up reaches Windows alone
//! (rule L, the KS-06 status).
//!
//! On every key event, once its record is updated, the decision asks for
//! the menu mask key (`Decision::mask`, roadmap item 1.2, KS-12) if the
//! event is suppressed while an Alt or Win key is held whose key-down
//! Windows saw: one on record as passed, or one the main thread's read
//! found held with no record. Windows then has a lone Alt or Win press so far, whose release
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
mod modifiers;
mod resync;
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
use std::fmt;
use std::time::Instant;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::core::windows_actions::BindAction;
use crate::data::binding::{Binding, BindingKind, BlockerMode};
use crate::data::trigger::Trigger;

pub use env::Env;
pub use modifiers::ModifierSet;
pub use schedule::{Scheduler, TimerQueue};

use chords::{ChordOutcome, Chords};
use env::EnvForeground;
use gestures::Gestures;
use keys::KeyState;
use resync::Resync;
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

/// Why [`Engine::reset`] starts over (roadmap item 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    /// `swch off` or `swch on` really changed the switch.
    RemappingSwitched,
    /// The config was reloaded: the main thread swapped the binding list
    /// in the same step.
    Reloaded,
    /// Keyboard input left this desktop: the session was locked, or input
    /// switched to another desktop (Ctrl+Alt+Del, a UAC prompt on the
    /// secure desktop). Key-ups may be lost there.
    InputLeft,
    /// The hook watchdog had the keyboard hook reinstalled: Windows had
    /// removed it, so the engine saw no event while Windows saw every one
    /// (roadmap item 2.4, KS-08).
    HookReinstalled,
}

impl fmt::Display for ResetReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ResetReason::RemappingSwitched => "remapping switched",
            ResetReason::Reloaded => "config reloaded",
            ResetReason::InputLeft => "keyboard input left this desktop",
            ResetReason::HookReinstalled => "the keyboard hook was reinstalled",
        })
    }
}

/// A signal about where keyboard input is, noted by the daemon's main
/// thread from its control window (the session notifications) and its
/// desktop-switch hook (roadmap item 2.3).
///
/// Only a signal that input left this desktop resets the engine (ruling
/// R2): once input is on another desktop the hook sees nothing, so nothing
/// a key event creates can come between the departure and its reset, and a
/// second departure signal of one episode finds nothing more to drop. A
/// signal that input came back resets nothing, however late it arrives, so
/// no reset discards a press, a gesture or a macro made after the return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSignal {
    /// `WTS_SESSION_LOCK`.
    SessionLocked,
    /// `WTS_SESSION_UNLOCK`.
    SessionUnlocked,
    /// `EVENT_SYSTEM_DESKTOPSWITCH` while another desktop has input.
    DesktopSwitchedAway,
    /// `EVENT_SYSTEM_DESKTOPSWITCH` while this desktop has input.
    DesktopSwitchedBack,
}

impl InputSignal {
    /// Every signal, in the order the main thread logs them.
    pub const ALL: [InputSignal; 4] = [
        InputSignal::SessionLocked,
        InputSignal::SessionUnlocked,
        InputSignal::DesktopSwitchedAway,
        InputSignal::DesktopSwitchedBack,
    ];

    /// The reset this signal asks for, if any.
    pub fn reset(self) -> Option<ResetReason> {
        match self {
            InputSignal::SessionLocked | InputSignal::DesktopSwitchedAway => {
                Some(ResetReason::InputLeft)
            }
            InputSignal::SessionUnlocked | InputSignal::DesktopSwitchedBack => None,
        }
    }

    /// Whether this signal says that input came back to this desktop.
    pub fn input_returned(self) -> bool {
        matches!(
            self,
            InputSignal::SessionUnlocked | InputSignal::DesktopSwitchedBack
        )
    }
}

impl fmt::Display for InputSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            InputSignal::SessionLocked => "session locked",
            InputSignal::SessionUnlocked => "session unlocked",
            InputSignal::DesktopSwitchedAway => "keyboard input switched to another desktop",
            InputSignal::DesktopSwitchedBack => "keyboard input switched back to this desktop",
        })
    }
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
    /// When the main thread reads the modifiers from Windows between key
    /// events (roadmap item 2.5).
    resync: Resync,
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
            resync: Resync::new(),
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
        // The main thread's next read leaves this event's modifier out: its
        // flag comes from this event, which Windows applies after the hook
        // returns. A decided event also proves that keyboard input is on
        // this desktop (roadmap item 2.5).
        self.resync.event(event.vk, true);

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

    /// The foreground window changed. Key events may have gone past the
    /// hook while a higher-integrity window had focus, so the hiding count
    /// moves, and the main thread's next drain reads the modifiers from the
    /// OS ([`resync_if_due`](Self::resync_if_due), roadmap item 2.5). Every
    /// record stays (roadmap item 2.3): a swallowed key-down keeps its
    /// auto-repeats and its key-up swallowed, so Windows gets no key-up
    /// whose press it never saw (the KS-06 status), and the key-up still
    /// reaches Windows if Windows has the key down by then (the witness at
    /// the key-up).
    ///
    /// The read skips a modifier whose key-down was swallowed: Windows
    /// never saw it go down, so its key state says nothing (KS-14). Such a
    /// key whose key-up was lost stays held for the engine until its next
    /// press, which reads as an auto-repeat and is swallowed, unless
    /// keyboard input left the desktop meanwhile, which releases every key.
    pub fn on_foreground_changed(&mut self) {
        self.resync.bump();
    }

    /// The main thread's read of the modifiers between key events (roadmap
    /// item 2.5), with `notes` the adapter's count of hidden input (a
    /// refused or late callback, a long stretch away from the messages, a
    /// gap the hook watchdog saw). It asks the OS only when something could
    /// have hidden a key event since the last read, and leaves out the
    /// modifiers an event touched since the last drain, whose flags come
    /// from their own events. It skips a modifier whose key-down was
    /// swallowed (KS-14) and changes no record. Returns whether it asked
    /// the OS. Call it between key events, never inside a hook callback.
    pub fn resync_if_due(&mut self, env: &dyn Env, notes: u64) -> bool {
        let which = self.resync.plan(notes);
        if which.is_empty() {
            return false;
        }
        self.keys.resync_modifiers(env, which);
        true
    }

    /// Keyboard input came back to this desktop: the main thread's next
    /// drain reads the modifiers again (roadmap item 2.5). Nothing is reset
    /// (ruling R2 of roadmap item 2.3).
    pub fn input_returned(&mut self) {
        self.resync.input_returned();
    }

    /// One of the daemon's own injected key events (the sentinel), which
    /// the adapter passes undecided (roadmap item 2.5, cut D2). Windows
    /// applies it, so a modifier's flag follows it, and only its flag: no
    /// record changes, nothing fires and no other key is touched. A
    /// modifier whose key-down was swallowed is left alone: the user holds
    /// it, and its auto-repeats must stay auto-repeats (KS-14). It proves
    /// nothing about where keyboard input is.
    pub fn track_injected(&mut self, event: KeyEvent) {
        self.keys.track_injected(event);
        self.resync.event(event.vk, false);
    }

    /// Whether the engine holds the key with this exact code.
    #[cfg(test)]
    pub(crate) fn holds(&self, vk: VIRTUAL_KEY) -> bool {
        self.keys.is_active(vk)
    }

    /// Starts over for `reason` (roadmap item 2.3; KS-31).
    ///
    /// Every pending Tap, Hold, DoubleTap, chord and sequence is dropped, so
    /// none completes across the reset, and the reset starts a new
    /// generation: a timer started before it is stale ([`is_live`](Self::is_live))
    /// and fires nothing, also against an entry armed after it (KS-11).
    ///
    /// The held keys, by reason:
    ///
    /// - [`ResetReason::RemappingSwitched`]: taken from
    ///   [`Env::os_key_down`], each on record as passed. While remapping was
    ///   off the engine saw no event and Windows saw every key-down, so a
    ///   key still held keeps its auto-repeats and its key-up. A key whose
    ///   key-down the engine swallowed reads as up, since Windows never saw
    ///   it go down, and keeps its record: its key-up stays swallowed, or
    ///   reaches Windows if Windows has the key down by then.
    /// - [`ResetReason::HookReinstalled`]: the same, for the same reason:
    ///   while Windows had removed the keyboard hook the engine saw no event
    ///   and Windows saw every one. So a swallowed key released meanwhile
    ///   is up again, and its next press is a press.
    /// - [`ResetReason::Reloaded`]: unchanged. A reload loses no key event,
    ///   and reading the OS would race the key state of a key-down that has
    ///   just passed (the state of the delivered key lags until the hook
    ///   returns): such a key held would read as up and its next repeat as
    ///   a press.
    /// - [`ResetReason::InputLeft`]: every key is released and every record
    ///   kept, and the OS is not asked: while input is on another desktop
    ///   the OS state is no witness, and a reset that ran before the switch
    ///   would read keys that are about to lose their key-ups as held. The
    ///   next key-down of each key is a first press (owner decision on open
    ///   question 9). A second reset of one episode changes nothing more.
    ///   The main thread reads the modifiers again once input is back: at
    ///   a signal that it came back ([`input_returned`](Self::input_returned))
    ///   or at the next decided key event (roadmap item 2.5).
    ///
    /// A reset that reads the OS covers the modifiers the main thread's
    /// drain would read, so that drain reads nothing.
    ///
    /// Call it only when something really changes: a repeated `swch on`
    /// would drop the gestures pending right now.
    pub fn reset(&mut self, env: &dyn Env, reason: ResetReason) {
        self.gestures.clear();
        self.chords = Chords::new();
        self.sequences = Sequences::new();
        self.mask_owed = false;
        match reason {
            ResetReason::RemappingSwitched | ResetReason::HookReinstalled => {
                self.keys.resync_from_os(env);
                self.resync.read_everything();
            }
            // No key event is lost on a reload, and the OS state of a key
            // whose key-down has just passed may still lag behind it.
            ResetReason::Reloaded => {}
            ResetReason::InputLeft => {
                self.keys.release_all();
                self.resync.input_left();
            }
        }
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
                        // hotkey matches: on the hook thread a title
                        // condition reads the foreground window's title
                        // (KS-08). `Always` asks the Env nothing.
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
            // and the key-up of a key-down the OS saw (KS-06). A foreground
            // change keeps the record: Windows never saw a swallowed
            // key-down, so a repeat that reached Windows would be the first
            // key-down it sees, in whatever window took the foreground.
            //
            // The resync can find a modifier held whose key-down the hook
            // never saw, so Windows has it down. A repeat without a record
            // keeps what the bindings asked. If it passes, the press is one
            // Windows saw: record that, so its later repeats and its key-up
            // pass too, and no completed chord swallows that key-up. If it
            // is swallowed, record that too (roadmap item 2.5): the key's
            // release unseen and its next press read as this repeat, so
            // Windows may have it up, and its key-up then asks the witness
            // as a swallowed press's does.
            if self.keys.is_blocked(vk.0) {
                decision.suppress = true;
            } else if self.keys.is_passed(vk.0) {
                decision.suppress = false;
            } else if decision.suppress {
                self.keys.mark_repeat_suppressed(vk.0);
            } else {
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
        // press was pending, which is evidence of a swallowed press even
        // where no record is left. The record is consumed either way.
        let was_blocked = self.keys.take_blocked(vk.0);
        let swallow = was_blocked || (released.gesture_bound && !was_passed);
        // The witness (roadmap item 2.3, ruling R1): Windows has this key
        // down although the engine swallowed its latest key-down, so
        // Windows saw a key-down the hook did not (a higher-integrity
        // window, the secure desktop, a press while remapping was off, an
        // injector). Swallowing the key-up would leave the key down there
        // for good. The hook runs before the key state takes this event, so
        // the read is the state before the key-up. The `&&` keeps every
        // key-up that passes from asking the OS anything. A generic modifier
        // code (an injector's `VK_SHIFT`) reads the generic state, either
        // side, because Windows applies it to a side the event does not
        // name.
        decision.suppress = swallow && !env.os_key_down(vk);
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
        VK_A, VK_CONTROL, VK_F13, VK_F14, VK_F15, VK_J, VK_K, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
        VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_SHIFT,
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
        // On the hook thread a title condition reads the foreground
        // window's title (GetWindowTextW): a binding on other keys must not
        // pay for it on every key press (KS-08).
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
        // (KS-35). Asked before its hotkey matches, a title condition would
        // read the foreground window's title on the hook thread for every
        // key press (KS-08).
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
        engine.reset(
            &FixedEnv(t0 + Duration::from_millis(50)),
            ResetReason::RemappingSwitched,
        );
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

    /// The codes `env` was asked about while `f` ran, in order.
    fn asked_during(env: &CountingEnv, f: impl FnOnce()) -> Vec<VIRTUAL_KEY> {
        let before = env.asked.borrow().len();
        f();
        env.asked.borrow()[before..].to_vec()
    }

    fn counting_env() -> CountingEnv {
        CountingEnv {
            now: Instant::now(),
            asked: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// The eight sided modifier codes, in the order a drain asks them.
    fn sided() -> Vec<VIRTUAL_KEY> {
        ModifierSet::SIDED.codes().collect()
    }

    /// An engine whose start has been read by a drain, so that nothing is
    /// due.
    fn drained_engine(env: &CountingEnv) -> Engine {
        let mut engine = Engine::new();
        assert!(engine.resync_if_due(env, 0), "the start reads (E0)");
        assert!(!engine.resync_if_due(env, 0), "and nothing more is due");
        engine
    }

    #[test]
    fn a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier() {
        // Roadmap item 2.5, ruling W2: the keyboard callback reads no
        // modifier state at an ordinary key, and Windows has everything up.
        let bindings = loaded(r#"( bindings: [ (keys: "F13", action: DoNothing) ] )"#);
        let env = counting_env();
        let mut engine = Engine::new();
        let asked = asked_during(&env, || {
            for event in [
                KeyEvent::down(VK_A),
                KeyEvent::up(VK_A),
                KeyEvent::down(VK_F13),
                KeyEvent::up(VK_F13),
            ] {
                let _ = engine.on_key(event, &bindings, &env);
            }
        });
        let modifiers = asked
            .iter()
            .filter(|&&vk| ModifierSet::ALL.contains(vk))
            .count();
        assert_eq!(
            modifiers, 0,
            "a key event with no modifier held asks Windows about no modifier (roadmap item 2.5)"
        );
    }

    /// An [`Env`] stopped at one instant in which Windows has the left Ctrl,
    /// Shift and Alt down, with their generic codes, and every other key up;
    /// it records every key it is asked about.
    struct ThreeModifiersDown {
        now: Instant,
        asked: std::cell::RefCell<Vec<VIRTUAL_KEY>>,
    }

    impl Env for ThreeModifiersDown {
        fn now(&self) -> Instant {
            self.now
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            self.asked.borrow_mut().push(vk);
            [
                VK_LCONTROL,
                VK_CONTROL,
                VK_LSHIFT,
                VK_SHIFT,
                VK_LMENU,
                VK_MENU,
            ]
            .contains(&vk)
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn a_key_event_asks_windows_once_for_each_modifier_held_for_it_and_never_for_its_own_key_or_a_swallowed_one()
     {
        // Roadmap item 2.5, ruling W2a, cut D16: a key-down asks Windows
        // about each modifier held for the engine, and only a key-up that
        // the engine swallows asks, about each Alt and Win held. The event's
        // own key and a modifier whose key-down was swallowed are never
        // asked (KS-14). Windows has every modifier the script holds down,
        // so nothing is dropped. Both bindings allow extra modifiers, so
        // that LWin and F13 are swallowed with the others held.
        let bindings = loaded(
            r#"( bindings: [
                (keys: "LWin", action: DoNothing, allow_extra_modifiers: true),
                (keys: "F13", action: DoNothing, allow_extra_modifiers: true),
            ] )"#,
        );
        let env = ThreeModifiersDown {
            now: Instant::now(),
            asked: std::cell::RefCell::new(Vec::new()),
        };
        let mut engine = Engine::new();
        let steps = [
            ("A down with nothing held", KeyEvent::down(VK_A), 0, false),
            ("A up with nothing held", KeyEvent::up(VK_A), 0, false),
            ("LCtrl down", KeyEvent::down(VK_LCONTROL), 0, false),
            ("LCtrl repeat", KeyEvent::down(VK_LCONTROL), 0, false),
            ("A down with LCtrl held", KeyEvent::down(VK_A), 1, false),
            ("A up with LCtrl held", KeyEvent::up(VK_A), 0, false),
            (
                "LShift down with LCtrl held",
                KeyEvent::down(VK_LSHIFT),
                1,
                false,
            ),
            (
                "A down with LCtrl and LShift held",
                KeyEvent::down(VK_A),
                2,
                false,
            ),
            (
                "A up with LCtrl and LShift held",
                KeyEvent::up(VK_A),
                0,
                false,
            ),
            ("LWin down, swallowed", KeyEvent::down(VK_LWIN), 2, true),
            (
                "A down with the swallowed LWin held",
                KeyEvent::down(VK_A),
                2,
                false,
            ),
            (
                "A up with the swallowed LWin held",
                KeyEvent::up(VK_A),
                0,
                false,
            ),
            ("LWin up, swallowed", KeyEvent::up(VK_LWIN), 0, true),
            ("LAlt down", KeyEvent::down(VK_LMENU), 2, false),
            ("A down with LAlt held", KeyEvent::down(VK_A), 3, false),
            ("A up with LAlt held, passing", KeyEvent::up(VK_A), 0, false),
            ("F13 down, swallowed", KeyEvent::down(VK_F13), 3, true),
            (
                "F13 up, swallowed, with LAlt held",
                KeyEvent::up(VK_F13),
                1,
                true,
            ),
            ("LAlt up", KeyEvent::up(VK_LMENU), 0, false),
            ("LShift up", KeyEvent::up(VK_LSHIFT), 0, false),
            ("LCtrl up", KeyEvent::up(VK_LCONTROL), 0, false),
        ];
        for (step, event, expected, swallowed) in steps {
            let before = env.asked.borrow().len();
            let decision = engine.on_key(event, &bindings, &env);
            assert_eq!(decision.suppress, swallowed, "{step}: suppressed");
            let asked = env.asked.borrow()[before..]
                .iter()
                .filter(|&&vk| vk != event.vk && ModifierSet::ALL.contains(vk))
                .count();
            assert_eq!(
                asked, expected,
                "{step}: a key event asks Windows once about each modifier held for it, never about its own key or a swallowed one (roadmap item 2.5)"
            );
        }
    }

    #[test]
    fn every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers() {
        // Roadmap item 2.5, cut D4: each source moves the count, and the
        // drain after it asks Windows about the eight sided modifiers once.
        // Only the drain's asks are counted.
        let bindings = loaded(r#"( bindings: [ (keys: "F13", action: DoNothing) ] )"#);
        for source in [
            "a foreground change",
            "input left, then a return signal",
            "input left, then a decided key event",
            "an adapter note",
        ] {
            let env = counting_env();
            let mut engine = drained_engine(&env);
            if source.starts_with("input left") {
                engine.reset(&env, ResetReason::InputLeft);
                assert!(
                    !engine.resync_if_due(&env, 0),
                    "{source}: no read while away"
                );
            }
            let notes = match source {
                "a foreground change" => {
                    engine.on_foreground_changed();
                    0
                }
                "input left, then a return signal" => {
                    engine.input_returned();
                    0
                }
                "input left, then a decided key event" => {
                    let _ = engine.on_key(KeyEvent::down(VK_F13), &bindings, &env);
                    0
                }
                _ => 1,
            };
            let asked = asked_during(&env, || {
                assert!(engine.resync_if_due(&env, notes), "{source}: a read");
            });
            assert_eq!(asked, sided(), "{source}");
            assert!(
                !engine.resync_if_due(&env, notes),
                "{source}: one read, not two"
            );
        }
    }

    #[test]
    fn swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing() {
        // E2: the reset of `swch on` and of a hook reinstall reads every key
        // from Windows, which covers a count that moved before it.
        for reason in [ResetReason::RemappingSwitched, ResetReason::HookReinstalled] {
            let env = counting_env();
            let mut engine = drained_engine(&env);
            engine.on_foreground_changed();
            engine.reset(&env, reason);
            let asked = asked_during(&env, || {
                assert!(!engine.resync_if_due(&env, 0), "{reason}");
            });
            assert_eq!(asked, [], "{reason}");
        }
    }

    #[test]
    fn a_reload_makes_no_read() {
        // A reload hides no key event, and its reset reads no key state.
        let env = counting_env();
        let mut engine = drained_engine(&env);
        let asked = asked_during(&env, || {
            engine.reset(&env, ResetReason::Reloaded);
            assert!(!engine.resync_if_due(&env, 0));
        });
        assert_eq!(asked, []);
    }

    #[test]
    fn a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one() {
        // KS-14: Windows never saw the swallowed LShift go down, so its key
        // state says nothing about it.
        let bindings = loaded(r#"( bindings: [ (keys: "LShift", action: DoNothing) ] )"#);
        let env = counting_env();
        let mut engine = Engine::new();
        let asked = asked_during(&env, || {
            assert!(engine.resync_if_due(&env, 0));
        });
        assert_eq!(asked, sided(), "the start reads the eight sided codes");

        let down = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(down.suppress, "the LShift key-down is swallowed");
        assert!(
            !engine.resync_if_due(&env, 0),
            "nothing is due; the plan forgets that LShift was touched"
        );
        engine.on_foreground_changed();
        let asked = asked_during(&env, || {
            assert!(engine.resync_if_due(&env, 0));
        });
        let expected: Vec<VIRTUAL_KEY> =
            sided().into_iter().filter(|&vk| vk != VK_LSHIFT).collect();
        assert_eq!(asked, expected, "every sided code but the swallowed LShift");
        assert!(engine.keys.is_active(VK_LSHIFT), "it stays held");
    }

    /// An [`Env`] stopped at one instant in which Windows has the right
    /// Shift down, so the generic Shift reads as down too.
    struct RightShiftDown(Instant);

    impl Env for RightShiftDown {
        fn now(&self) -> Instant {
            self.0
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            vk == VK_RSHIFT || vk == VK_SHIFT
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn the_witness_of_a_generic_shift_key_up_reads_the_generic_state() {
        // Critic OQ2 of roadmap item 2.3: a real keyboard reports sided
        // codes, but another program can inject the generic VK_SHIFT, which
        // Windows applies to a side the event does not name. The witness
        // then asks about either side: with nothing down the swallowed
        // key-up stays swallowed; with a Shift down in Windows it passes.
        let bindings = loaded(r#"( bindings: [ (keys: "Shift", action: DoNothing) ] )"#);
        let t0 = Instant::now();

        let mut engine = Engine::new();
        let down = engine.on_key(KeyEvent::down(VK_SHIFT), &bindings, &FixedEnv(t0));
        assert!(down.suppress, "the generic Shift key-down is swallowed");
        let up = engine.on_key(KeyEvent::up(VK_SHIFT), &bindings, &FixedEnv(t0));
        assert!(
            up.suppress,
            "no Shift is down in Windows: the key-up stays swallowed"
        );

        let mut engine = Engine::new();
        let down = engine.on_key(KeyEvent::down(VK_SHIFT), &bindings, &RightShiftDown(t0));
        assert!(down.suppress, "the generic Shift key-down is swallowed");
        let up = engine.on_key(KeyEvent::up(VK_SHIFT), &bindings, &RightShiftDown(t0));
        assert!(
            !up.suppress,
            "a Shift is down in Windows: the generic key-up passes"
        );
    }

    /// An [`Env`] stopped at one instant whose OS key state the test sets;
    /// a generic modifier code reads down while either side is.
    struct OsEnv {
        now: Instant,
        down: std::cell::RefCell<Vec<VIRTUAL_KEY>>,
    }

    impl OsEnv {
        fn new(down: &[VIRTUAL_KEY]) -> Self {
            Self {
                now: Instant::now(),
                down: std::cell::RefCell::new(down.to_vec()),
            }
        }

        /// Windows takes `vk` down.
        fn press(&self, vk: VIRTUAL_KEY) {
            self.down.borrow_mut().push(vk);
        }

        /// Windows takes `vk` up.
        fn release(&self, vk: VIRTUAL_KEY) {
            self.down.borrow_mut().retain(|&key| key != vk);
        }
    }

    impl Env for OsEnv {
        fn now(&self) -> Instant {
            self.now
        }
        fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
            let down = self.down.borrow();
            let either = |left, right| down.contains(&left) || down.contains(&right);
            match vk {
                VK_SHIFT => either(VK_LSHIFT, VK_RSHIFT),
                VK_CONTROL => either(VK_LCONTROL, VK_RCONTROL),
                VK_MENU => either(VK_LMENU, VK_RMENU),
                other => down.contains(&other),
            }
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    /// The `Alt+Shift` auto-blocker that `block_original_combo: true` adds,
    /// built directly: loading it from RON would read the registry.
    fn alt_shift_blocker() -> Vec<Binding> {
        vec![Binding::new_auto_blocker(
            KeyCombination::from_keys([VK_MENU, VK_SHIFT]),
            BlockerMode::Swallow,
        )]
    }

    #[test]
    fn a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed() {
        // Roadmap item 2.5, cut D10: Windows has LShift down from a key-down
        // the engine never saw. The Alt+Shift blocker swallows its first
        // repeat; a repeat that passes while Windows also has LCtrl down
        // makes the press one Windows saw, so a later repeat that the
        // blocker would swallow passes, and so does the key-up (KS-06).
        let bindings = alt_shift_blocker();
        let env = OsEnv::new(&[]);
        let mut engine = Engine::new();
        let alt = engine.on_key(KeyEvent::down(VK_LMENU), &bindings, &env);
        assert!(!alt.suppress, "Alt alone is no hotkey");
        env.press(VK_LMENU);
        env.press(VK_LSHIFT);
        // A foreground change: the main thread's read finds LShift down.
        engine.on_foreground_changed();
        assert!(engine.resync_if_due(&env, 0));

        let first = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(
            first.suppress,
            "the blocker swallows a repeat with no record"
        );
        let ctrl = engine.on_key(KeyEvent::down(VK_LCONTROL), &bindings, &env);
        assert!(!ctrl.suppress);
        env.press(VK_LCONTROL);
        let second = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(
            !second.suppress,
            "with Ctrl down in Windows the blocker does not match"
        );
        let _ = engine.on_key(KeyEvent::up(VK_LCONTROL), &bindings, &env);
        env.release(VK_LCONTROL);
        let third = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(
            !third.suppress,
            "a repeat of a press Windows saw passes whatever the blocker asks"
        );
        let up = engine.on_key(KeyEvent::up(VK_LSHIFT), &bindings, &env);
        assert!(!up.suppress, "and so does its key-up");
    }

    #[test]
    fn an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held() {
        // Roadmap item 2.5, cut D2, KS-14: the user holds the swallowed
        // LShift, so the daemon's own injected LShift down and up leave it
        // held, and its next key-down stays an auto-repeat.
        let bindings = loaded(r#"( bindings: [ (keys: "LShift", action: DoNothing) ] )"#);
        let env = FixedEnv(Instant::now());
        let mut engine = Engine::new();
        let down = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(down.suppress);
        assert_eq!(down.fire, [BindAction::DoNothing]);

        engine.track_injected(KeyEvent::down(VK_LSHIFT));
        engine.track_injected(KeyEvent::up(VK_LSHIFT));
        assert!(engine.holds(VK_LSHIFT), "the user still holds it");

        let repeat = engine.on_key(KeyEvent::down(VK_LSHIFT), &bindings, &env);
        assert!(repeat.suppress, "a repeat of a swallowed press");
        assert!(repeat.fire.is_empty(), "which fires nothing");
    }

    #[test]
    fn an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing() {
        // Cut D2: only modifiers feed matching, so the daemon's injected A
        // (a macro's `Key`, a `TypeText` code unit) is not followed.
        let bindings = loaded(r#"( bindings: [ (keys: "A", action: DoNothing) ] )"#);
        let env = FixedEnv(Instant::now());
        let mut engine = Engine::new();
        engine.track_injected(KeyEvent::down(VK_A));
        assert!(!engine.holds(VK_A));
        let down = engine.on_key(KeyEvent::down(VK_A), &bindings, &env);
        assert_eq!(down.fire, [BindAction::DoNothing], "a first press");
    }

    #[test]
    fn only_a_signal_that_input_left_asks_for_a_reset() {
        // Ruling R2 of roadmap item 2.3: a signal that input came back
        // resets nothing, however late it arrives.
        for signal in InputSignal::ALL {
            assert_eq!(
                signal.reset().is_some(),
                !signal.input_returned(),
                "{signal}"
            );
        }
        assert_eq!(
            InputSignal::SessionLocked.reset(),
            Some(ResetReason::InputLeft)
        );
        assert_eq!(
            InputSignal::DesktopSwitchedAway.reset(),
            Some(ResetReason::InputLeft)
        );
        assert_eq!(InputSignal::SessionUnlocked.reset(), None);
        assert_eq!(InputSignal::DesktopSwitchedBack.reset(), None);
        assert_eq!(
            InputSignal::DesktopSwitchedAway.to_string(),
            "keyboard input switched to another desktop"
        );
        assert_eq!(
            ResetReason::InputLeft.to_string(),
            "keyboard input left this desktop"
        );
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
