//! The daemon's main thread (roadmap item 2.2, KS-11, KS-25, KS-33): the
//! thread that installs the hooks and pumps their messages also owns the
//! engine and every gesture timer, and serves `swch on` and `swch off` and
//! config reloads (roadmap item 2.3).
//!
//! # State
//!
//! The engine, the gesture timers and the foreground cache of `when:`
//! (`platform::foreground`, roadmap item 2.3) live in this thread's
//! [`MainState`], a thread-local that only [`with_main`] reaches. Every caller of the engine
//! runs on this thread: the keyboard and foreground hook callbacks, which
//! Windows dispatches inside this thread's message calls, and the loop's
//! timer drain and requests. So no lock guards the engine, and none can be
//! held across a callback or inverted by a thread priority. `with_main`
//! borrows the state with `try_borrow_mut`: a callback that runs while the
//! state is borrowed is refused and its event passes, so a re-entered
//! callback neither panics nor hangs the keyboard. Under the borrow only
//! the engine decides, the gesture timer is set (`SetWaitableTimer`), the
//! foreground cache reads a process name (in the foreground callback) and
//! a title condition reads a title of another process's window; no action
//! runs, nothing is injected or submitted and no message is pumped or sent.
//!
//! # The loop
//!
//! [`MainThread::run`] waits for its gesture timer and its messages at once,
//! in the shape of the action executor's loop (roadmap item 2.1):
//!
//! 1. Drain, until a whole pass finds nothing: the requests (`swch on` and
//!    `off`, a config reload), then the input signals noted since the last
//!    pass (keyboard input left this desktop or came back), then every
//!    gesture timer whose due time has passed by the clock, each decided
//!    under the borrow and carried out after it.
//! 2. Plan: drop the timers no entry waits for, then set the wake for the
//!    earliest one left unless it is set for it already, or cancel it.
//! 3. Wait: `MsgWaitForMultipleObjectsEx` on the gesture timer, for any
//!    message (`QS_ALLINPUT`, which has `QS_SENDMESSAGE`), also one already
//!    seen but not removed (`MWMO_INPUTAVAILABLE`).
//! 4. Pump: `PeekMessageW` until the queue is empty, a `MAIN_WAKE` comes (a
//!    request or a noted input signal waits: back to 1) or `WM_QUIT` (the
//!    loop ends).
//!
//! The hook callbacks run inside those `PeekMessageW` calls. Windows calls a
//! low-level hook by sending a message to the thread that installed it, a
//! sent message wakes the wait, and `PeekMessageW` dispatches sent messages
//! before it looks at posted ones, so nothing stands between a key event and
//! its callback. A callback that asks for a timer sets the wake itself
//! ([`GestureTimers`] as a [`Scheduler`]), so the wait always holds the
//! earliest deadline, whatever ran inside it. The clock decides: an early,
//! late or stale signal costs one drain.
//!
//! The gesture timer is a high-resolution waitable timer
//! (`platform::timers`), so a deadline is met within about a millisecond,
//! not on the 15.6 ms tick of the system clock.

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::time::{Duration, Instant};

use windows::Win32::{
    Foundation::{E_UNEXPECTED, HWND, LPARAM, WAIT_FAILED, WAIT_OBJECT_0, WPARAM},
    System::Threading::{GetCurrentThreadId, INFINITE},
    UI::WindowsAndMessaging::*,
};
use windows::core::{Error, HRESULT};

use crate::data::binding::Binding;
use crate::engine::{
    Decision, Engine, InputSignal, ResetReason, Scheduler, TimerQueue, TimerRequest, TimerToken,
};
use crate::hook::config_watcher::ApplyReload;
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::keyboard_hook_callback::{HookEnv, carry_out};
use crate::logging::counters::{self, Counter};
use crate::platform::foreground::ForegroundCache;
use crate::platform::timers::WaitableTimer;

/// The thread message that wakes the loop for a request.
pub(crate) const MAIN_WAKE: u32 = WM_APP + 1;

/// How long `swch on` and `off` wait for the main thread's answer. Below the
/// `swch` client's 5 s reply deadline, so the client gets this reason.
pub(crate) const REPLY_TIMEOUT: Duration = Duration::from_secs(2);

/// How many requests wait for the main thread before the next is refused.
const REQUEST_CAPACITY: usize = 8;

thread_local! {
    /// The main state of this thread, if it is the daemon's main thread.
    static MAIN: RefCell<Option<MainState>> = const { RefCell::new(None) };

    /// The input signals noted on this thread since the last drain
    /// (roadmap item 2.3). A `Cell` of plain data: the control window's
    /// procedure and the desktop-switch callback note one, which needs no
    /// borrow, whatever message call they run in.
    static PENDING_SIGNALS: Cell<PendingSignals> = const { Cell::new(PendingSignals::NONE) };
}

/// The input signals that wait for the drain (roadmap item 2.3): one flag
/// per [`InputSignal`], so that a signal noted twice before a drain is
/// logged and acted on once, and the error of a desktop switch whose
/// destination could not be told.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingSignals {
    /// Indexed by the position of the signal in [`InputSignal::ALL`].
    seen: [bool; 4],
    unknown: Option<HRESULT>,
}

impl PendingSignals {
    const NONE: PendingSignals = PendingSignals {
        seen: [false; 4],
        unknown: None,
    };

    /// The signals noted, in [`InputSignal::ALL`] order.
    pub(crate) fn signals(self) -> impl Iterator<Item = InputSignal> {
        InputSignal::ALL
            .into_iter()
            .zip(self.seen)
            .filter_map(|(signal, seen)| seen.then_some(signal))
    }

    /// The error of a desktop switch whose destination could not be told.
    pub(crate) fn unknown(self) -> Option<HRESULT> {
        self.unknown
    }
}

/// Notes `signal` for the next drain of this thread's main loop and wakes
/// the loop. Main thread only: called from the control window's procedure
/// and the desktop-switch callback, which may run inside any message call,
/// so it only sets a flag and posts a wake, never resets anything here.
pub(crate) fn note_input_signal(signal: InputSignal) {
    // `ALL` lists every signal, so the position is always found; nothing
    // here may panic inside a window procedure.
    let Some(index) = InputSignal::ALL.iter().position(|&known| known == signal) else {
        return;
    };
    let _ = PENDING_SIGNALS.try_with(|pending| {
        let mut noted = pending.get();
        noted.seen[index] = true;
        pending.set(noted);
    });
    wake_this_thread();
}

/// Notes a desktop switch whose destination Windows would not tell
/// (`error`), for the next drain, which logs it and resets nothing.
pub(crate) fn note_unknown_desktop_switch(error: HRESULT) {
    let _ = PENDING_SIGNALS.try_with(|pending| {
        let mut noted = pending.get();
        noted.unknown = Some(error);
        pending.set(noted);
    });
    wake_this_thread();
}

/// The signals noted since the last call, if any, which it clears.
pub(crate) fn take_pending_signals() -> Option<PendingSignals> {
    PENDING_SIGNALS
        .try_with(|pending| pending.replace(PendingSignals::NONE))
        .ok()
        .filter(|noted| *noted != PendingSignals::NONE)
}

/// Posts `MAIN_WAKE` to the calling thread, so that a wait that a noted
/// signal came in returns to the drain. A post that fails loses no signal:
/// the drain also runs when `pump` empties the queue.
fn wake_this_thread() {
    // SAFETY: GetCurrentThreadId takes no arguments; PostThreadMessageW
    // takes the thread id and the message by value, reads no memory of ours
    // and returns without waiting; MAIN_WAKE carries no pointer.
    let _ = unsafe { PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE, WPARAM(0), LPARAM(0)) };
}

#[cfg(test)]
thread_local! {
    /// How many calls `with_main` refused on this thread because the state
    /// was borrowed.
    pub(crate) static REFUSED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// What the main thread owns: the engine and its gesture timers.
pub(crate) struct MainState {
    pub(super) engine: Engine,
    pub(super) timers: GestureTimers,
    /// The foreground window's executable name for `when:`, filled from the
    /// foreground hook and never in the keyboard callback (roadmap item 2.3).
    pub(super) foreground: ForegroundCache,
}

/// Runs `f` on this thread's main state and returns what it returns.
///
/// `None`, without running `f`, if this thread has no main state, or if the
/// state is borrowed already: a callback re-entered under the borrow then
/// passes its event, and nothing panics or waits. `f` must only decide,
/// reset or set the gesture timer (see the module docs).
pub(crate) fn with_main<R>(f: impl FnOnce(&mut MainState) -> R) -> Option<R> {
    MAIN.try_with(|main| {
        let Ok(mut state) = main.try_borrow_mut() else {
            #[cfg(test)]
            REFUSED.with(|refused| refused.set(refused.get() + 1));
            return None;
        };
        state.as_mut().map(f)
    })
    .ok()
    .flatten()
}

impl MainState {
    /// Turns remapping on or off (`swch on`, `swch off`; KS-31) and returns
    /// whether the switch changed. On the main thread, between callbacks:
    /// an event is decided either before the flip with the old state or
    /// after it with the reset one. A real change resets the transient
    /// state ([`reset_transient_state`](Self::reset_transient_state)). A
    /// change that is no change resets nothing: it would drop the gestures
    /// pending right now.
    pub(crate) fn set_enabled(&mut self, hook: &KeyboardHook, enabled: bool) -> bool {
        if !hook.switch_enabled(enabled, self) {
            return false;
        }
        self.reset_transient_state(hook, ResetReason::RemappingSwitched);
        true
    }

    /// Drops every pending gesture, chord, sequence and owed mask, starts a
    /// new timer generation and drops the timers it made stale, takes the
    /// held keys as `reason` says ([`Engine::reset`]), and stops every macro
    /// running and every slow action still queued (roadmap items 2.1 and
    /// 2.3). On the main thread, between two events.
    pub(crate) fn reset_transient_state(&mut self, hook: &KeyboardHook, reason: ResetReason) {
        self.reset_engine(reason);
        hook.cancel_actions();
    }

    /// Makes `bindings` the live list and resets the engine with it, on the
    /// main thread, between two events (roadmap item 2.3, KS-31): no key
    /// event is decided with the new list and a gesture of the old one.
    /// `update_bindings` sorts the list, stores it and stops the macros of
    /// the old list, so the reset here only resets the engine.
    pub(crate) fn reload(&mut self, hook: &KeyboardHook, bindings: Vec<Binding>) {
        hook.update_bindings(bindings);
        self.reset_engine(ResetReason::Reloaded);
    }

    /// The engine half of [`reset_transient_state`](Self::reset_transient_state).
    fn reset_engine(&mut self, reason: ResetReason) {
        // The foreground hook reports nothing while remapping is off, so a
        // window that took the foreground meanwhile is read here.
        if reason == ResetReason::RemappingSwitched {
            self.foreground.resync();
        }
        self.engine.reset(&HookEnv::new(&self.foreground), reason);
        let engine = &self.engine;
        self.timers.retain(&|token| engine.is_live(token));
    }

    /// `window` took the foreground (`EVENT_SYSTEM_FOREGROUND`): the cache
    /// reads its executable once, here and not in the keyboard callback,
    /// and the engine resyncs the modifiers.
    pub(crate) fn foreground_changed(&mut self, window: HWND) {
        self.foreground.window_changed(window);
        let env = HookEnv::new(&self.foreground);
        self.engine.on_foreground_changed(&env);
    }

    /// Keyboard input came back to this desktop: the window that has the
    /// foreground now is read again, since the foreground hook may have
    /// reported nothing meanwhile. Nothing is reset (ruling R2).
    pub(crate) fn input_returned(&mut self) {
        self.foreground.resync();
    }
}

/// The main thread's gesture timers: the queue of every timer the engine
/// asked for, and one waitable timer set for the earliest.
pub(crate) struct GestureTimers {
    pub(super) queue: TimerQueue,
    wake: WaitableTimer,
    /// The due time the waitable timer is set for, until it signals or is
    /// cancelled.
    armed: Option<Instant>,
}

/// What [`GestureTimers::plan_wake`] did.
enum Wake {
    /// The wake is set for the earliest timer, or no timer is queued.
    Wait,
    /// The earliest timer is due: drain without waiting.
    Due,
    /// The waitable timer could not be set: every queued timer was
    /// forgotten, and `dropped` Holds with them.
    Failed { error: Error, dropped: usize },
}

/// What to do with the gesture timer, decided without a Win32 call.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    Cancel,
    Due,
    Keep,
    Arm { after: Duration, at: Instant },
}

/// The plan for a timer set for `armed` when the earliest queued deadline is
/// `next`. A timer set for that deadline stands: its signal is not a
/// message, so nothing that ran on this thread since can have taken it.
fn plan(armed: Option<Instant>, next: Option<Instant>, now: Instant) -> Plan {
    match next {
        None => Plan::Cancel,
        Some(at) if at <= now => Plan::Due,
        Some(at) if armed == Some(at) => Plan::Keep,
        Some(at) => Plan::Arm {
            after: at.duration_since(now),
            at,
        },
    }
}

impl GestureTimers {
    pub(crate) fn new() -> windows::core::Result<Self> {
        Ok(Self {
            queue: TimerQueue::new(),
            wake: WaitableTimer::new()?,
            armed: None,
        })
    }

    /// The wait took the timer's signal: it is set for nothing now.
    fn fired(&mut self) {
        self.armed = None;
    }

    fn cancel(&mut self) {
        if self.armed.take().is_some() {
            // A timer that fails to stop signals once more, and the loop
            // takes that for a wake.
            let _ = self.wake.cancel();
        }
    }

    /// Step 2 of the loop: drops the timers no entry of `engine` waits for
    /// and sets the wake for the earliest one left. If the waitable timer
    /// cannot be set, every queued timer is forgotten, so that no Hold waits
    /// for a wake that never comes.
    fn plan_wake(&mut self, engine: &mut Engine, now: Instant) -> Wake {
        let live = &*engine;
        self.queue.retain(&|token| live.is_live(token));
        match plan(self.armed, self.queue.next_due(), now) {
            Plan::Cancel => {
                self.cancel();
                Wake::Wait
            }
            Plan::Due => Wake::Due,
            Plan::Keep => Wake::Wait,
            Plan::Arm { after, at } => {
                // Setting the timer replaces the due time it had.
                self.armed = None;
                match self.wake.set(after) {
                    Ok(()) => {
                        self.armed = Some(at);
                        Wake::Wait
                    }
                    Err(error) => {
                        let lost = RefCell::new(Vec::new());
                        self.queue.retain(&|token| {
                            lost.borrow_mut().push(token);
                            false
                        });
                        let dropped = lost
                            .into_inner()
                            .into_iter()
                            .filter(|token| engine.forget_timer(*token))
                            .count();
                        Wake::Failed { error, dropped }
                    }
                }
            }
        }
    }
}

impl Scheduler for GestureTimers {
    /// Queues `request` and, if it is due before the deadline the wake is set
    /// for, sets the wake for it. Runs in the hook callback, under the
    /// borrow: one `SetWaitableTimer` at most, no allocation while the
    /// queue has room. False, with `request` taken out again, if the timer
    /// could not be set.
    fn start(&mut self, request: TimerRequest) -> bool {
        self.queue.start(request);
        if self.armed.is_none_or(|armed| request.due < armed) {
            let after = request.due.saturating_duration_since(Instant::now());
            if self.wake.set(after).is_err() {
                let token = request.token;
                self.queue.retain(&|queued| queued != token);
                // A failed set leaves the timer set for no deadline the loop
                // can name, so the next plan sets it again for the earliest
                // one left instead of keeping it.
                self.armed = None;
                return false;
            }
            self.armed = Some(request.due);
        }
        true
    }

    fn retain(&mut self, live: &dyn Fn(TimerToken) -> bool) {
        self.queue.retain(live);
    }
}

/// A request to the main thread.
enum Request {
    /// `swch on` or `swch off`; the answer is whether the switch changed.
    SetEnabled {
        enabled: bool,
        reply: SyncSender<bool>,
    },
    /// A config reload from the watcher thread: the new list, swapped in
    /// together with an engine reset (roadmap item 2.3).
    Reload {
        bindings: Vec<Binding>,
        reply: SyncSender<()>,
    },
}

/// Why a request to the main thread got no answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MainThreadError {
    /// The main thread did not answer within [`REPLY_TIMEOUT`]. The request
    /// stays queued and is carried out when the main thread reaches it.
    NoAnswer,
    /// The main loop has ended, or is ending.
    ShuttingDown,
    /// Too many requests wait already.
    Busy,
}

impl std::fmt::Display for MainThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MainThreadError::NoAnswer => "the daemon's main thread did not answer within 2 s",
            MainThreadError::ShuttingDown => "the daemon is shutting down",
            MainThreadError::Busy => "the daemon is busy; try again",
        })
    }
}

impl std::error::Error for MainThreadError {}

impl ApplyReload for MainThreadHandle {
    fn apply(&self, bindings: Vec<Binding>) -> Result<(), MainThreadError> {
        self.reload(bindings)
    }
}

/// How another thread reaches the main thread: the IPC connection workers,
/// and a test's helper thread.
#[derive(Clone)]
pub(crate) struct MainThreadHandle {
    thread_id: u32,
    requests: SyncSender<Request>,
}

impl MainThreadHandle {
    /// Asks the main thread to turn remapping on or off and waits up to
    /// [`REPLY_TIMEOUT`] for its answer: whether the switch changed.
    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<bool, MainThreadError> {
        let (reply, answer) = mpsc::sync_channel(1);
        match self
            .requests
            .try_send(Request::SetEnabled { enabled, reply })
        {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(MainThreadError::Busy),
            Err(TrySendError::Disconnected(_)) => return Err(MainThreadError::ShuttingDown),
        }
        self.wake();
        answer
            .recv_timeout(REPLY_TIMEOUT)
            .map_err(|error| match error {
                RecvTimeoutError::Timeout => MainThreadError::NoAnswer,
                RecvTimeoutError::Disconnected => MainThreadError::ShuttingDown,
            })
    }

    /// Asks the main thread to make `bindings` the live list, with the
    /// engine reset that goes with it (`MainState::reload`), and waits up
    /// to [`REPLY_TIMEOUT`] for it. On `NoAnswer` the reload stays queued
    /// and goes live when the main thread reaches it.
    pub(crate) fn reload(&self, bindings: Vec<Binding>) -> Result<(), MainThreadError> {
        let (reply, answer) = mpsc::sync_channel(1);
        match self.requests.try_send(Request::Reload { bindings, reply }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(MainThreadError::Busy),
            Err(TrySendError::Disconnected(_)) => return Err(MainThreadError::ShuttingDown),
        }
        self.wake();
        answer
            .recv_timeout(REPLY_TIMEOUT)
            .map_err(|error| match error {
                RecvTimeoutError::Timeout => MainThreadError::NoAnswer,
                RecvTimeoutError::Disconnected => MainThreadError::ShuttingDown,
            })
    }

    /// Ends the main loop: posts `WM_QUIT` to the main thread, which then
    /// removes the hooks itself.
    pub(crate) fn request_exit(&self) -> windows::core::Result<()> {
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; WM_QUIT carries no pointer in
        // its parameters. A thread id that is no longer valid only makes the
        // call fail, and that error is returned.
        unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) }
    }

    /// Posted after the request it announces, so that a drain that missed
    /// it is followed by this wake. A lost wake loses no request: it waits
    /// in the queue until the loop's next wake of any kind.
    fn wake(&self) {
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value, reads no memory of ours and returns without waiting;
        // MAIN_WAKE carries no pointer. An id that is no longer valid only
        // makes the call fail.
        let _ = unsafe { PostThreadMessageW(self.thread_id, MAIN_WAKE, WPARAM(0), LPARAM(0)) };
    }

    /// A handle with no main thread behind it, as one whose main thread has
    /// ended: every request fails at once.
    #[cfg(test)]
    pub(crate) fn without_a_thread() -> Self {
        let (requests, _) = mpsc::sync_channel(1);
        Self {
            thread_id: 0,
            requests,
        }
    }
}

/// The calling thread as the daemon's main thread: its [`MainState`] and its
/// requests. Dropping it removes the state, after which every callback on
/// this thread passes its event, and closes the request queue, after which
/// every request fails at once. Not `Send`: the state belongs to the thread
/// that made it.
pub(crate) struct MainThread {
    requests: Receiver<Request>,
    handle: MainThreadHandle,
    _thread_bound: PhantomData<*const ()>,
}

impl MainThread {
    /// Makes the calling thread the main thread: creates its message queue,
    /// its gesture timer and its engine. An error if the timer cannot be
    /// made, or if this thread is a main thread already.
    pub(crate) fn new() -> windows::core::Result<Self> {
        let timers = GestureTimers::new()?;
        if !timers.wake.is_high_resolution() {
            log::warn!(
                "gesture timers wake on the 15.6 ms system tick: no high-resolution timer before Windows 10 version 1803"
            );
        }
        let state = MainState {
            engine: Engine::new(),
            timers,
            foreground: ForegroundCache::new(),
        };
        let installed = MAIN.with(|main| match main.try_borrow_mut() {
            Ok(mut slot) if slot.is_none() => {
                *slot = Some(state);
                true
            }
            _ => false,
        });
        if !installed {
            return Err(Error::new(
                E_UNEXPECTED,
                "this thread is the daemon's main thread already",
            ));
        }
        // A signal noted on this thread before it was a main thread is no
        // signal for this loop.
        let _ = take_pending_signals();
        let mut msg = MSG::default();
        // SAFETY: `msg` is a local that outlives the call, which only writes
        // it; the call creates this thread's message queue, as
        // PostThreadMessageW asks of the receiving thread, and removes
        // nothing.
        let _ = unsafe { PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE) };
        // SAFETY: GetCurrentThreadId takes no arguments, touches no memory of
        // ours and cannot fail.
        let thread_id = unsafe { GetCurrentThreadId() };
        let (requests_tx, requests) = mpsc::sync_channel(REQUEST_CAPACITY);
        Ok(Self {
            requests,
            handle: MainThreadHandle {
                thread_id,
                requests: requests_tx,
            },
            _thread_bound: PhantomData,
        })
    }

    /// A handle for the threads that send requests.
    pub(crate) fn handle(&self) -> MainThreadHandle {
        self.handle.clone()
    }

    /// Reads the foreground window into the cache (roadmap item 2.3).
    /// `App::run` calls it right after the hooks are installed: a window
    /// change from then on is a foreground event, so no change falls
    /// between this read and the hook.
    pub(crate) fn refresh_foreground(&self) {
        let _ = with_main(|main| main.foreground.resync());
    }

    /// Runs the main loop (see the module docs) until `WM_QUIT`, carrying
    /// out every decision on `hook`. An error if the wait fails; the caller
    /// then removes the hooks.
    pub(crate) fn run(&self, hook: &KeyboardHook) -> windows::core::Result<()> {
        let Some(timer) = with_main(|main| main.timers.wake.handle()) else {
            return Err(Error::new(
                E_UNEXPECTED,
                "the main loop runs without its main state",
            ));
        };
        let mut msg = MSG::default();
        loop {
            self.drain(hook);
            match with_main(|main| main.timers.plan_wake(&mut main.engine, Instant::now())) {
                Some(Wake::Due) => continue,
                Some(Wake::Failed { error, dropped }) => log::error!(
                    "could not set the gesture timer ({error}); {dropped} pending Holds dropped"
                ),
                Some(Wake::Wait) | None => {}
            }
            // SAFETY: the slice holds the gesture timer's handle, which the
            // WaitableTimer in this thread's main state keeps open: only
            // dropping `self` removes that state, and `self` is borrowed for
            // the whole call. The slice lives until the call returns, which
            // only reads it.
            let woke = unsafe {
                MsgWaitForMultipleObjectsEx(
                    Some(&[timer]),
                    INFINITE,
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                )
            };
            if woke == WAIT_OBJECT_0 {
                with_main(|main| main.timers.fired());
                continue;
            }
            if woke == WAIT_FAILED {
                return Err(Error::from_thread());
            }
            // A message waits. MWMO_INPUTAVAILABLE also counts one queued
            // before the wait began, such as the messages behind a MAIN_WAKE
            // that `pump` left for after the drain.
            if pump(&mut msg) {
                return Ok(());
            }
        }
    }

    /// Step 1 of the loop: serves the requests, answers the noted input
    /// signals and carries out the due timers until a whole pass finds none
    /// of them.
    fn drain(&self, hook: &KeyboardHook) {
        loop {
            let mut busy = false;
            while let Ok(request) = self.requests.try_recv() {
                serve(hook, request);
                busy = true;
            }
            if let Some(signals) = take_pending_signals() {
                answer_input_signals(hook, signals);
                busy = true;
            }
            while let Some(decision) = with_main(take_due_timer).flatten() {
                carry_out(hook, decision);
                busy = true;
            }
            if !busy {
                return;
            }
        }
    }
}

impl Drop for MainThread {
    fn drop(&mut self) {
        // In a thread's end the slot may be gone already; and a drop inside
        // `with_main` leaves the state to the thread's end.
        let _ = MAIN.try_with(|main| {
            if let Ok(mut slot) = main.try_borrow_mut() {
                slot.take();
            }
        });
        let _ = take_pending_signals();
    }
}

/// Step 1 of the loop, for the input signals noted since the last drain
/// (roadmap item 2.3, ruling R2). While remapping is off they are dropped:
/// the reset of `swch on` reads the keys and the foreground anyway.
/// Otherwise each is logged once, in [`InputSignal::ALL`] order; a signal
/// that keyboard input left this desktop resets the transient state once,
/// however many such signals there are; a signal that input came back
/// reads the foreground window again and resets nothing, however late it
/// comes. The logging and the reset run outside the main state's borrow
/// and outside every callback, between two events.
fn answer_input_signals(hook: &KeyboardHook, signals: PendingSignals) {
    if !hook.is_enabled() {
        return;
    }
    for signal in signals.signals() {
        log::info!("{signal}");
    }
    if let Some(error) = signals.unknown() {
        log::warn!(
            "keyboard input switched desktops, but the daemon could not tell where it went ({}); nothing was reset",
            Error::from(error)
        );
    }
    if let Some(reason) = signals.signals().find_map(InputSignal::reset)
        && with_main(|main| main.reset_transient_state(hook, reason)).is_some()
    {
        log::info!("state reset: {reason}");
    }
    if signals.signals().any(InputSignal::input_returned) {
        let _ = with_main(MainState::input_returned);
    }
}

/// Decides the earliest timer that is due by the clock, if any, and starts
/// the timers its decision asks for.
fn take_due_timer(main: &mut MainState) -> Option<Decision> {
    let request = main.timers.queue.take_due(Instant::now())?;
    let mut decision = main.engine.on_timer(request.token);
    let timers = std::mem::take(&mut decision.timers);
    for _ in 0..main.engine.start_timers(timers, &mut main.timers) {
        counters::bump(Counter::HoldTimerNotStarted);
    }
    Some(decision)
}

/// Carries out one request. The answer never blocks: a requester that gave
/// up has dropped its receiver, and its request still took effect.
fn serve(hook: &KeyboardHook, request: Request) {
    match request {
        Request::SetEnabled { enabled, reply } => {
            // `None` only if the loop itself ran inside `with_main`, which it
            // never does; the requester then hears that the daemon stops.
            if let Some(changed) = with_main(|main| main.set_enabled(hook, enabled)) {
                let _ = reply.try_send(changed);
            }
        }
        Request::Reload { bindings, reply } => {
            let count = bindings.len();
            if with_main(|main| main.reload(hook, bindings)).is_some()
                && reply.try_send(()).is_err()
            {
                // The watcher stopped waiting after 2 s and logged that the
                // bindings go live later; this is when (critic OQ4).
                log::info!("config reload carried out late: {count} bindings live");
            }
        }
    }
}

/// Step 4 of the loop: takes this thread's messages until its queue is
/// empty or a `MAIN_WAKE` comes, which goes back to the drain at once. True
/// on `WM_QUIT`.
fn pump(msg: &mut MSG) -> bool {
    // SAFETY: `msg` is the loop's local and outlives every call, which only
    // writes it and removes the message from this thread's own queue. The
    // hook callbacks run inside this call.
    while unsafe { PeekMessageW(msg, None, 0, 0, PM_REMOVE) }.as_bool() {
        if msg.message == WM_QUIT {
            return true;
        }
        if msg.hwnd.is_invalid() && msg.message == MAIN_WAKE {
            return false;
        }
        // SAFETY: `msg` was filled by PeekMessageW above and outlives both
        // calls, which only read it.
        unsafe {
            let _ = TranslateMessage(msg);
            DispatchMessageW(msg);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::{Foundation::*, UI::Input::KeyboardAndMouse::*};
    use windows::core::w;

    use super::*;
    use crate::core::executor::testing::{
        FakeRunner, Record, leaked_counters, next_record, wait_until,
    };
    use crate::core::executor::{Executor, QUEUE_CAPACITY, Stopped, Submitted, TargetWindow};
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::sequence::SequenceStep;
    use crate::data::trigger::Trigger;
    use crate::engine::{Engine, Env, InputSignal, KeyEvent, sort_by_specificity};
    use crate::hook::keyboard_hook::KeyboardHook;
    use crate::hook::keyboard_hook_callback::decide_event;

    // No test installs a hook or injects input. The loop tests run
    // `MainThread::run` on a thread of their own with a local hook, send
    // only `WM_NULL` and `WM_CLOSE` to a message-only window of that thread,
    // and every action is `DoNothing` or a macro that a recording runner
    // takes; a fast action is only recorded under test anyway.

    /// An [`Env`] stopped at one instant with nothing held and no
    /// foreground window.
    struct QuietEnv(Instant);

    impl Env for QuietEnv {
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

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// A local hook with `binding` on F13, which allows extra modifiers: the
    /// adapter reads the live key state of this machine.
    fn hook_with(binding: Binding) -> KeyboardHook {
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![binding.with_allow_extra_modifiers(true)]);
        hook
    }

    fn f13_hold(term_ms: u64, action: BindAction) -> Binding {
        Binding::new(KeyCombination::new(VK_F13), action).with_trigger(Trigger::Hold(term_ms))
    }

    fn key_macro() -> BindAction {
        BindAction::Sequence(Arc::new(vec![SequenceStep::PressKey(VK_F13)]))
    }

    fn f13(msg: u32) -> (WPARAM, u32) {
        (WPARAM(msg as usize), u32::from(VK_F13.0))
    }

    /// A message-only `STATIC` window of the calling thread, as an integer,
    /// since an `HWND` is not `Send`.
    fn message_only_window() -> isize {
        // SAFETY: the class name is a static wide literal and the window
        // name is null; HWND_MESSAGE makes a message-only window, which no
        // other process enumerates or sends input to. No creation data is
        // passed.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                None,
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        }
        .expect("a message-only STATIC window is created");
        hwnd.0 as isize
    }

    fn hwnd(raw: isize) -> HWND {
        HWND(raw as *mut std::ffi::c_void)
    }

    #[test]
    fn with_main_refuses_reentry_on_the_same_thread() {
        // On its own thread, so that a broken refusal fails the test
        // instead of the suite.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _main = MainThread::new().expect("the main thread is set up");
            let before = REFUSED.with(Cell::get);
            let nested = with_main(|_| with_main(|_| ()));
            let _ = tx.send((nested, REFUSED.with(Cell::get) - before));
        });
        let (nested, refused) = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("re-entry panicked or hung");
        assert_eq!(nested, Some(None));
        assert_eq!(refused, 1, "the refusal is counted");
    }

    #[test]
    fn with_main_works_again_after_a_call() {
        let _main = MainThread::new().expect("the main thread is set up");
        assert_eq!(with_main(|_| 1), Some(1));
        assert_eq!(with_main(|_| 2), Some(2));
    }

    #[test]
    fn with_main_is_none_without_a_main_state() {
        assert_eq!(with_main(|_| ()), None);
        let main = MainThread::new().expect("the main thread is set up");
        assert_eq!(with_main(|_| ()), Some(()));
        drop(main);
        assert_eq!(with_main(|_| ()), None, "dropping it clears the state");
    }

    #[test]
    fn a_second_main_thread_on_one_thread_is_refused() {
        let _main = MainThread::new().expect("the main thread is set up");
        assert!(MainThread::new().is_err());
        assert_eq!(with_main(|_| ()), Some(()), "the first one stays");
    }

    #[test]
    fn plan_sets_the_timer_only_for_another_deadline() {
        // The gesture timer's signal is not a message, so nothing the loop
        // dispatched since it was set can have taken it: a timer set for the
        // earliest deadline stands.
        let now = Instant::now();
        let at = now + ms(100);
        let sooner = now + ms(50);

        assert_eq!(plan(None, None, now), Plan::Cancel);
        assert_eq!(plan(Some(at), None, now), Plan::Cancel);
        assert_eq!(plan(Some(at), Some(now), now), Plan::Due);
        assert_eq!(plan(None, Some(at), now), Plan::Arm { after: ms(100), at });
        assert_eq!(
            plan(Some(at), Some(at), now),
            Plan::Keep,
            "set for this deadline: the timer stands"
        );
        assert_eq!(
            plan(Some(at), Some(sooner), now),
            Plan::Arm {
                after: ms(50),
                at: sooner
            }
        );
    }

    #[test]
    fn a_timer_started_for_an_earlier_deadline_resets_the_wake() {
        // The wake is set inside `start`, which a hook callback calls, so the
        // loop's wait always holds the earliest deadline, whatever ran
        // inside the wait.
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: DoNothing, trigger: Hold(100)),
                (keys: "F14", action: DoNothing, trigger: Hold(50)),
                (keys: "F15", action: DoNothing, trigger: Hold(200)),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();
        let mut timers = GestureTimers::new().expect("the gesture timer is made");
        let env = QuietEnv(Instant::now());
        let mut press = |vk: VIRTUAL_KEY, timers: &mut GestureTimers| {
            let mut decision = engine.on_key(KeyEvent::down(vk), &bindings, &env);
            let due = decision.timers[0].due;
            assert_eq!(
                engine.start_timers(std::mem::take(&mut decision.timers), timers),
                0
            );
            due
        };

        assert_eq!(timers.armed, None);
        let first = press(VK_F13, &mut timers);
        assert_eq!(timers.armed, Some(first));
        let earlier = press(VK_F14, &mut timers);
        assert_eq!(timers.armed, Some(earlier), "an earlier deadline resets it");
        press(VK_F15, &mut timers);
        assert_eq!(timers.armed, Some(earlier), "a later one leaves it");
        assert_eq!(timers.queue.len(), 3);
    }

    #[test]
    fn a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop() {
        // Review OQ3 of roadmap item 2.2: after a failed set the waitable
        // timer is set for no deadline the loop can name. If `armed` still
        // named the old one, the plan would keep it, and the loop could wait
        // for a wake that never comes.
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: DoNothing, trigger: Hold(10000)),
                (keys: "F14", action: DoNothing, trigger: Hold(5000)),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();
        let mut timers = GestureTimers::new().expect("the gesture timer is made");
        let env = QuietEnv(Instant::now());

        let mut f13 = engine.on_key(KeyEvent::down(VK_F13), &bindings, &env);
        let f13_due = f13.timers[0].due;
        assert_eq!(
            engine.start_timers(std::mem::take(&mut f13.timers), &mut timers),
            0
        );
        assert_eq!(timers.armed, Some(f13_due));

        timers.wake = WaitableTimer::not_a_timer();
        assert!(
            WaitableTimer::not_a_timer().set(ms(10)).is_err(),
            "precondition: Windows refuses to set an event as a timer"
        );
        let mut f14 = engine.on_key(KeyEvent::down(VK_F14), &bindings, &env);
        assert_eq!(
            engine.start_timers(std::mem::take(&mut f14.timers), &mut timers),
            1,
            "the earlier deadline could not be set"
        );
        assert_eq!(
            timers.armed, None,
            "a failed set leaves the timer set for no deadline the loop knows"
        );

        timers.wake = WaitableTimer::new().expect("the gesture timer is made");
        assert!(matches!(
            timers.plan_wake(&mut engine, Instant::now()),
            Wake::Wait
        ));
        assert_eq!(
            timers.armed,
            Some(f13_due),
            "the loop sets the wake again for the Hold that is left"
        );
    }

    #[test]
    fn a_wake_that_cannot_be_set_counts_only_the_holds_it_drops() {
        // Review m2 of roadmap item 2.2: a wake that cannot be set forgets
        // every queued timer, but only a Hold is dropped with it. The Tap
        // that waits for its DoubleTap window stays and fires at the next
        // key-down; a chord's expiry drops nothing.
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: DoNothing, trigger: Hold(10000)),
                (keys: "F14", action: SwitchLanguage, trigger: Tap(200)),
                (keys: "F14", action: ToggleCapsLock, trigger: DoubleTap(6000)),
                (chord: (keys: ["J", "K"], window_ms: 5000), action: ToggleCapsLock),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();
        let mut timers = GestureTimers::new().expect("the gesture timer is made");
        let env = QuietEnv(Instant::now());
        // J before F14: a first key-down fires a Tap that waits for its
        // window, so the window must open last.
        for event in [
            KeyEvent::down(VK_F13),
            KeyEvent::down(VK_J),
            KeyEvent::down(VK_F14),
            KeyEvent::up(VK_F14),
        ] {
            let mut decision = engine.on_key(event, &bindings, &env);
            assert_eq!(
                engine.start_timers(std::mem::take(&mut decision.timers), &mut timers),
                0
            );
        }
        assert_eq!(timers.queue.len(), 3, "a Hold, a window and a chord expiry");

        timers.wake = WaitableTimer::not_a_timer();
        timers.armed = None;
        let Wake::Failed { dropped, .. } = timers.plan_wake(&mut engine, Instant::now()) else {
            panic!("a wake that cannot be set fails");
        };
        assert_eq!(dropped, 1, "only the Hold is dropped");

        let decision = engine.on_key(KeyEvent::down(VK_F15), &bindings, &env);
        assert_eq!(
            decision.fire,
            [BindAction::SwitchLanguage],
            "the waiting Tap fires at the next key-down"
        );
    }

    #[test]
    fn a_message_posted_behind_a_wake_is_dispatched_without_more_input() {
        // Technical debt m-1 of roadmap item 2.1, for the main loop: `pump`
        // goes back to the drain at a MAIN_WAKE and leaves the messages
        // behind it, which are then no longer new input. Without
        // MWMO_INPUTAVAILABLE the wait would block on them until more input
        // came; here nothing else comes until the watchdog's WM_QUIT.
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let hook = KeyboardHook::new();
            let window = message_only_window();
            let handle = main.handle();
            // SAFETY: PostThreadMessageW and PostMessageW take their
            // arguments by value and read no memory of ours; both post to
            // this thread's own queue, a MAIN_WAKE first and the window's
            // WM_CLOSE behind it, and neither message carries a pointer.
            unsafe {
                PostThreadMessageW(handle.thread_id, MAIN_WAKE, WPARAM(0), LPARAM(0))
                    .expect("this thread has a message queue");
                PostMessageW(Some(hwnd(window)), WM_CLOSE, WPARAM(0), LPARAM(0))
                    .expect("the window takes a posted message");
            }
            let _ = ready_tx.send((window, handle));
            main.run(&hook)
        });
        let (window, handle) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");

        let begun = Instant::now();
        let closed = loop {
            // SAFETY: IsWindow only reads the handle's value; a window that
            // was destroyed reads as no window.
            if !unsafe { IsWindow(Some(hwnd(window))) }.as_bool() {
                break true;
            }
            if begun.elapsed() > Duration::from_secs(1) {
                break false;
            }
            thread::sleep(ms(1));
        };
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        assert!(
            closed,
            "the WM_CLOSE posted behind a MAIN_WAKE was not dispatched within 1 s"
        );
    }

    #[test]
    fn a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer() {
        // A low-level hook call is a message sent to the thread that
        // installed the hook. A wait blind to sent messages would leave each
        // one to the 1 s timeout below.
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let hook = hook_with(f13_hold(10_000, BindAction::DoNothing));
            let window = message_only_window();
            let (wparam, vk) = f13(WM_KEYDOWN);
            let armed = decide_event(&hook, wparam, vk, 0).is_some();
            let queued = with_main(|main| main.timers.queue.len());
            let _ = ready_tx.send((window, main.handle(), armed, queued));
            main.run(&hook)
        });
        let (window, handle, armed, queued) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert!(armed, "the F13 key-down reached the engine");
        assert_eq!(queued, Some(1), "a Hold timer 10 s out is queued");

        let mut trips = Vec::new();
        let mut failed = 0;
        for _ in 0..100 {
            let begun = Instant::now();
            // SAFETY: `window` belongs to the loop thread, which keeps it
            // until it ends after the WM_QUIT below; WM_NULL carries no
            // pointer and no result pointer is passed.
            let answered = unsafe {
                SendMessageTimeoutW(
                    hwnd(window),
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG,
                    1000,
                    None,
                )
            };
            trips.push(begun.elapsed());
            if answered == LRESULT(0) {
                failed += 1;
            }
        }
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");

        assert_eq!(failed, 0, "every sent message is answered: {trips:?}");
        trips.sort();
        let median = trips[trips.len() / 2];
        assert!(
            median < ms(2),
            "a sent message took {median:?} in the median; sorted: {trips:?}"
        );
    }

    #[test]
    fn a_200_ms_hold_fires_within_a_few_ms_of_its_deadline() {
        // A wake on the system clock tick (about 15.6 ms) is about 7.8 ms
        // late on average, and a high-resolution timer under 1 ms. 5 ms sits
        // between the two; the median of five ignores one press in which a
        // loaded machine did not run the thread at once. The key-down's time
        // is read before the engine reads its own, so no lateness is
        // negative.
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        let hook = hook_with(f13_hold(200, key_macro()));
        hook.attach_executor(Some(executor.handle()));
        let main = MainThread::new().expect("the main thread is set up");
        let handle = main.handle();
        let (fired_tx, fired) = mpsc::channel();
        let waiter = thread::spawn(move || {
            for _ in 0..5 {
                let record = records.recv_timeout(Duration::from_secs(2));
                let _ = handle.request_exit();
                let _ = fired_tx.send(record.map(|record| record.at));
            }
        });

        let mut late = Vec::new();
        for _ in 0..5 {
            let pressed = Instant::now();
            let (wparam, vk) = f13(WM_KEYDOWN);
            assert!(decide_event(&hook, wparam, vk, 0).is_some());
            main.run(&hook).expect("the loop ends on WM_QUIT");
            let at = fired
                .recv_timeout(Duration::from_secs(5))
                .expect("the waiter answers")
                .expect("the Hold fired within 2 s");
            let (wparam, vk) = f13(WM_KEYUP);
            assert!(decide_event(&hook, wparam, vk, 0).is_some());
            let held = at.duration_since(pressed);
            assert!(
                held >= ms(200),
                "the Hold fired early, {held:?} after its key-down"
            );
            late.push(held - ms(200));
        }
        waiter.join().expect("the waiter does not panic");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);

        late.sort();
        let median = late[late.len() / 2];
        assert!(
            median < ms(5),
            "a 200 ms Hold fired {median:?} late in the median; sorted: {late:?}"
        );
    }

    #[test]
    fn remapping_switches_through_the_main_thread_and_drops_pending_timers() {
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        // Leaked, so that the loop thread and this one share it, as the
        // daemon's threads share the process-wide hook.
        let hook: &'static KeyboardHook =
            Box::leak(Box::new(hook_with(f13_hold(300, key_macro()))));
        hook.attach_executor(Some(executor.handle()));
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let (wparam, vk) = f13(WM_KEYDOWN);
            let armed = decide_event(hook, wparam, vk, 0).is_some();
            let _ = ready_tx.send((main.handle(), armed));
            main.run(hook)
        });
        let (handle, armed) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert!(armed, "a 300 ms Hold is armed");

        assert_eq!(handle.set_enabled(false), Ok(true));
        assert!(!hook.is_enabled());
        assert_eq!(handle.set_enabled(false), Ok(false), "already off");
        assert!(
            records.recv_timeout(ms(500)).is_err(),
            "a Hold armed before remapping went off never fires"
        );
        assert_eq!(handle.set_enabled(true), Ok(true));
        assert!(hook.is_enabled());

        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing() {
        // Roadmap item 2.3's acceptance (KS-31): the swap and the reset are
        // one step on the main thread, between two events, so the key-up
        // that follows a reload finds no Tap of the old bindings to fire.
        let _main = MainThread::new().expect("the main thread is set up");
        let hook = hook_with(
            Binding::new(KeyCombination::new(VK_F13), BindAction::SwitchLanguage)
                .with_trigger(Trigger::Tap(10_000)),
        );
        let (wparam, vk) = f13(WM_KEYDOWN);
        let down = decide_event(&hook, wparam, vk, 0).expect("the key-down reaches the engine");
        assert!(down.suppress, "the Tap swallows its key-down");

        with_main(|main| {
            main.reload(
                &hook,
                vec![
                    Binding::new(KeyCombination::new(VK_F13), BindAction::ToggleCapsLock)
                        .with_trigger(Trigger::Tap(10_000))
                        .with_allow_extra_modifiers(true),
                ],
            );
        })
        .expect("the main state is free");
        assert_eq!(
            hook.bindings()[0].action,
            BindAction::ToggleCapsLock,
            "the new bindings are live"
        );

        let (wparam, vk) = f13(WM_KEYUP);
        let up = decide_event(&hook, wparam, vk, 0).expect("the key-up reaches the engine");
        assert!(up.fire.is_empty(), "the old Tap fired: {:?}", up.fire);
        assert!(up.suppress, "the key-up follows its swallowed key-down");
    }

    #[test]
    fn a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold() {
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        // Leaked, so that the loop thread and this one share it, as the
        // daemon's threads share the process-wide hook.
        let hook: &'static KeyboardHook =
            Box::leak(Box::new(hook_with(f13_hold(300, key_macro()))));
        hook.attach_executor(Some(executor.handle()));
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let (wparam, vk) = f13(WM_KEYDOWN);
            let armed = decide_event(hook, wparam, vk, 0).is_some();
            let _ = ready_tx.send((main.handle(), armed));
            main.run(hook)
        });
        let (handle, armed) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert!(armed, "a 300 ms Hold is armed");

        assert_eq!(
            handle.reload(vec![f13_hold(300, BindAction::DoNothing)]),
            Ok(())
        );
        assert_eq!(
            hook.bindings()[0].action,
            BindAction::DoNothing,
            "the new bindings are live once the main thread answers"
        );
        assert!(
            records.recv_timeout(ms(500)).is_err(),
            "a Hold armed before the reload never fires"
        );

        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands() {
        // Critic OQ4: a reload that the main thread reaches after the
        // watcher's 2 s wait still goes live, and says so.
        let _main = MainThread::new().expect("the main thread is set up");
        let hook = KeyboardHook::new();
        let (reply, answer) = mpsc::sync_channel(1);
        drop(answer);
        let ((), lines) = crate::logging::testing::capture(|| {
            serve(
                &hook,
                Request::Reload {
                    bindings: vec![f13_hold(300, BindAction::DoNothing)],
                    reply,
                },
            );
        });
        assert_eq!(hook.bindings().len(), 1, "the reload went live");
        assert!(
            lines
                .iter()
                .any(|line| line.contains("config reload carried out late: 1 bindings live")),
            "{lines:?}"
        );
    }

    /// `[F15, Delay(10000), F16]`: a macro that runs until it is stopped.
    fn long_macro() -> BindAction {
        BindAction::Sequence(Arc::new(vec![
            SequenceStep::PressKey(VK_F15),
            SequenceStep::Delay(10_000),
            SequenceStep::PressKey(VK_F16),
        ]))
    }

    /// The bindings of the lock tests: an F13 Hold(300) of `key_macro()`
    /// and an F14 `SwitchLanguage` that swallows F14 and does not repeat.
    fn lock_test_hook() -> &'static KeyboardHook {
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![
            f13_hold(300, key_macro()).with_allow_extra_modifiers(true),
            Binding::new(KeyCombination::new(VK_F14), BindAction::SwitchLanguage)
                .with_allow_extra_modifiers(true),
        ]);
        Box::leak(Box::new(hook))
    }

    #[test]
    fn a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key() {
        // Roadmap item 2.3's acceptance "nothing is stuck after Win+L",
        // through the request path and without a session change: the lock
        // signal, as the control window notes it, is drained by the loop,
        // which drops the armed Hold, stops the running macro and releases
        // F14, swallowed and held when the session locked.
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        let runs = executor.handle();
        let hook = lock_test_hook();
        hook.attach_executor(Some(executor.handle()));
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let pressed = [
                f13(WM_KEYDOWN),
                (WPARAM(WM_KEYDOWN as usize), u32::from(VK_F14.0)),
            ]
            .map(|(wparam, vk)| decide_event(hook, wparam, vk, 0));
            assert!(
                pressed.iter().all(Option::is_some),
                "both key-downs reach the engine"
            );
            assert_eq!(
                hook.submit(long_macro(), TargetWindow::NONE),
                Submitted::Queued
            );
            let _ = ready_tx.send(main.handle());
            wait_until("the macro waits in its Delay", || runs.active_runs() == 1);
            note_input_signal(InputSignal::SessionLocked);
            main.run(hook).expect("the loop ends on WM_QUIT");
            let down = decide_event(hook, WPARAM(WM_KEYDOWN as usize), u32::from(VK_F14.0), 0)
                .expect("the key-down reaches the engine");
            let up = decide_event(hook, WPARAM(WM_KEYUP as usize), u32::from(VK_F14.0), 0)
                .expect("the key-up reaches the engine");
            (down, up)
        });
        let handle = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert_eq!(next_record(&records).text(), "key F15", "the macro runs");

        let begun = Instant::now();
        wait_until("the macro is gone", || executor.handle().active_runs() == 0);
        let stopped = begun.elapsed();
        assert!(
            records.recv_timeout(ms(500)).is_err(),
            "neither the Hold armed before the lock nor the macro's F16 runs"
        );
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        let (down, up) = looper.join().expect("the loop thread does not panic");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);

        assert!(
            stopped < ms(100),
            "the macro stopped {stopped:?} after the lock"
        );
        assert_eq!(
            down.fire,
            [BindAction::SwitchLanguage],
            "the first F14 after the lock is a press, not a repeat (owner decision, open question 9)"
        );
        assert!(down.suppress);
        assert!(up.suppress, "its key-up follows its swallowed key-down");
    }

    #[test]
    fn a_late_return_signal_keeps_a_running_macro_and_an_armed_hold() {
        // Ruling R2 at the adapter: signals that input came back reset
        // nothing, so a Hold armed and a macro started after the return
        // survive them.
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        let runs = executor.handle();
        let hook = lock_test_hook();
        hook.attach_executor(Some(executor.handle()));
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            note_input_signal(InputSignal::SessionLocked);
            main.drain(hook);
            let (wparam, vk) = f13(WM_KEYDOWN);
            assert!(decide_event(hook, wparam, vk, 0).is_some(), "the Hold arms");
            assert_eq!(
                hook.submit(long_macro(), TargetWindow::NONE),
                Submitted::Queued
            );
            wait_until("the macro waits in its Delay", || runs.active_runs() == 1);
            note_input_signal(InputSignal::SessionUnlocked);
            note_input_signal(InputSignal::DesktopSwitchedBack);
            let _ = ready_tx.send(main.handle());
            main.run(hook)
        });
        let handle = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert_eq!(next_record(&records).text(), "key F15", "the macro runs");
        thread::sleep(ms(100));
        let running = executor.handle().active_runs();
        let held = records.recv_timeout(ms(500));
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);

        assert_eq!(running, 1, "the macro started after the return still runs");
        assert_eq!(
            held.expect("the Hold armed after the return fires").text(),
            "key F13"
        );
    }

    #[test]
    fn the_drain_logs_one_line_per_input_signal() {
        // Critique M1 and ruling R2: one timestamped line per signal, in
        // `InputSignal::ALL` order within one drain, and one reset however
        // many departure signals the drain finds.
        let main = MainThread::new().expect("the main thread is set up");
        let hook = KeyboardHook::new();
        let ((), lines) = crate::logging::testing::capture(|| {
            for signal in InputSignal::ALL {
                note_input_signal(signal);
            }
            note_unknown_desktop_switch(E_ACCESSDENIED);
            main.drain(&hook);
        });
        let expected = [
            "[INFO] session locked",
            "[INFO] session unlocked",
            "[INFO] keyboard input switched to another desktop",
            "[INFO] keyboard input switched back to this desktop",
            "[WARN] keyboard input switched desktops, but the daemon could not tell where it went",
            "[INFO] state reset: keyboard input left this desktop",
        ];
        for text in expected {
            assert_eq!(
                lines.iter().filter(|line| line.contains(text)).count(),
                1,
                "{text:?} once in {lines:?}"
            );
        }
        assert_eq!(take_pending_signals(), None, "the drain took them");
    }

    #[test]
    fn a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain() {
        // Ruling R2 and the drain's own gate: signals still pending when
        // `swch off` lands are dropped, not logged and not answered, since
        // the reset of `swch on` reads the keys again anyway.
        let main = MainThread::new().expect("the main thread is set up");
        let hook = KeyboardHook::new();
        note_input_signal(InputSignal::SessionLocked);
        assert_eq!(
            with_main(|state| state.set_enabled(&hook, false)),
            Some(true),
            "remapping goes off"
        );
        let ((), lines) = crate::logging::testing::capture(|| main.drain(&hook));
        assert!(
            lines
                .iter()
                .all(|line| !line.contains("session locked") && !line.contains("state reset")),
            "a drain with remapping off logs and resets nothing: {lines:?}"
        );
        assert_eq!(take_pending_signals(), None, "the drain still took them");
    }

    /// A [`WindowQuery`](crate::platform::foreground::WindowQuery) whose
    /// foreground window the test moves; window `n` belongs to process `n`
    /// and process `n` runs `p<n>.exe`.
    struct MovingQuery(std::rc::Rc<Cell<usize>>);

    impl crate::platform::foreground::WindowQuery for MovingQuery {
        fn foreground(&self) -> HWND {
            HWND(self.0.get() as *mut std::ffi::c_void)
        }
        fn owner(&self, window: HWND) -> u32 {
            window.0 as usize as u32
        }
        fn image_name(&self, pid: u32) -> Option<String> {
            Some(format!("p{pid}.exe"))
        }
        fn title(&self, _window: HWND) -> Option<String> {
            None
        }
    }

    #[test]
    fn a_return_signal_reads_the_foreground_window_again() {
        // Roadmap 2.3, ruling M3: the foreground hook reports nothing while
        // input is on another desktop, so the drain of a return signal
        // reads the window that has the foreground now.
        for signal in [
            InputSignal::SessionUnlocked,
            InputSignal::DesktopSwitchedBack,
        ] {
            let main = MainThread::new().expect("the main thread is set up");
            let hook = KeyboardHook::new();
            let window = std::rc::Rc::new(Cell::new(0x1000));
            with_main(|state| {
                state.foreground =
                    ForegroundCache::with_query(Box::new(MovingQuery(window.clone())));
                state.foreground_changed(HWND(0x1000 as *mut std::ffi::c_void));
            })
            .expect("the main state is free");
            window.set(0x2000);
            assert_eq!(
                with_main(|state| state.foreground.app().map(str::to_owned)),
                Some(Some("p4096.exe".to_owned())),
                "the cache still names the window of before the {signal}"
            );

            note_input_signal(signal);
            main.drain(&hook);

            assert_eq!(
                with_main(|state| state.foreground.app().map(str::to_owned)),
                Some(Some("p8192.exe".to_owned())),
                "the drain of {signal} reads the foreground window again"
            );
        }
    }

    #[test]
    fn a_switch_request_to_an_ended_main_thread_fails_at_once() {
        let handle = thread::spawn(|| {
            MainThread::new()
                .expect("the main thread is set up")
                .handle()
        })
        .join()
        .expect("the scratch thread does not panic");

        let begun = Instant::now();
        assert_eq!(
            handle.set_enabled(false),
            Err(MainThreadError::ShuttingDown)
        );
        let took = begun.elapsed();
        assert!(took < ms(100), "the request waited {took:?}");
    }
}
