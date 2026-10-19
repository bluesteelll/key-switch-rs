//! The hook watchdog (roadmap item 2.4, KS-08): it finds a keyboard hook
//! that Windows removed and has the main thread reinstall it.
//!
//! Windows calls a low-level keyboard hook by sending a message to the
//! thread that installed it, and removes the hook without a word when a call
//! runs past `LowLevelHooksTimeout` ("on Windows 7 and later, the hook is
//! silently removed", Microsoft Learn, LowLevelKeyboardProc). Remapping then
//! stops while `swch status` still says `running`.
//!
//! The `hook-watchdog` thread registers a message-only window for the raw
//! input of every keyboard (`RIDEV_INPUTSINK`): Windows posts it a
//! `WM_INPUT` per key event, whatever window has the foreground. While keys
//! arrive it compares their count with the keyboard callback's heartbeat
//! (`hook::health`) four times a second ([`decision`]), and when keys arrive
//! and the callback stays silent for more than a second it asks the main
//! thread, through its request queue, to reinstall the keyboard hook: only
//! the thread that installed a hook may install it again (KS-01). It never
//! injects input, which would reset the idle timer and keep the screen on.
//! Raw input counts only while the keyboard hook is installed (ruling I3),
//! so the thread can start before `install` and stop after `uninstall`.
//!
//! # Cost
//!
//! The main thread does nothing per key event for the watchdog but the
//! heartbeat's one atomic increment: raw input goes to this thread, never to
//! the main thread's queue, where every key would add a `WM_INPUT` to
//! dispatch next to the hook calls (cut D1). Per key event Windows builds a
//! `WM_INPUT` for this process and wakes this thread, which reads it with
//! `GetRawInputData` and counts it, on another core (ruling M6). While keys
//! are typed the thread ticks at most four times a second; idle, it waits
//! with no wake-up. A keyboard callback that runs while this thread handles
//! an event is slower, about 18 us at the median on the developer's machine
//! (bench row R14 against R14/settled); whether a key's raw input reaches
//! this thread before or after its own hook call is not documented (cut
//! Q6).
//!
//! # Raw input in this process
//!
//! "Only one window per raw input device class may be registered to receive
//! raw input within a process" (Microsoft Learn, RegisterRawInputDevices):
//! the keyboard's belongs to this window, and a later raw input user in the
//! daemon (the tray of roadmap item 4.6) must share it.
//!
//! # No key data
//!
//! [`raw_key_event`] reads an event's make code and virtual key into a stack
//! local only to tell whether the hook would see it, and returns a `bool`:
//! the watchdog keeps counts and instants, and logs no key.

mod decision;

use std::io;
use std::marker::PhantomData;
use std::mem::offset_of;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::{
    Foundation::{HWND, LPARAM, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM},
    System::Threading::{GetCurrentThreadId, INFINITE},
    UI::{
        Input::{
            GetRawInputBuffer, GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RAWKEYBOARD,
            RID_INPUT, RIM_TYPEKEYBOARD,
        },
        WindowsAndMessaging::*,
    },
};
use windows::core::w;

use crate::hook::health::{HealthState, ReinstallOutcome, ReinstallRequest};
use crate::hook::installer::{self, Installer};
use crate::hook::main_thread::{MainThreadError, MainThreadHandle};
use crate::logging::counters::{Counter, Counters};
use crate::platform::desktop;
use crate::platform::timers::WaitableTimer;
use decision::{TICK, Verdict, Watchdog};

/// Where the watchdog sends its requests: the main thread's request queue
/// in the daemon, a recorder in the tests.
pub(crate) trait RequestReinstall: Send + 'static {
    /// Queues `request` without waiting; the answer comes on the receiver.
    fn request(
        &self,
        request: ReinstallRequest,
    ) -> Result<Receiver<ReinstallOutcome>, MainThreadError>;
}

impl RequestReinstall for MainThreadHandle {
    fn request(
        &self,
        request: ReinstallRequest,
    ) -> Result<Receiver<ReinstallOutcome>, MainThreadError> {
        self.request_reinstall(request)
    }
}

/// The timer the watchdog ticks by: the waitable timer in the daemon, a
/// fake in the tests.
trait TickTimer {
    fn set(&self, after: Duration) -> windows::core::Result<()>;
    fn cancel(&self) -> windows::core::Result<()>;
}

impl TickTimer for WaitableTimer {
    fn set(&self, after: Duration) -> windows::core::Result<()> {
        WaitableTimer::set(self, after)
    }

    fn cancel(&self) -> windows::core::Result<()> {
        WaitableTimer::cancel(self)
    }
}

/// What to do with the tick timer, decided without a Win32 call.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    Cancel,
    Keep,
    Arm { after: Duration, at: Instant },
}

/// The plan for a timer set for `armed` when the decision's next tick is
/// `next`. A timer set for that tick stands: its signal is not a message,
/// so nothing that ran on this thread since can have taken it.
fn plan(armed: Option<Instant>, next: Option<Instant>, now: Instant) -> Plan {
    match next {
        None => Plan::Cancel,
        Some(at) if armed == Some(at) => Plan::Keep,
        Some(at) => Plan::Arm {
            after: at.saturating_duration_since(now),
            at,
        },
    }
}

/// What the loop did in the thread's life: its passes, one per wake, and
/// the raw key events it read, counted or not. The stop line reports them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LoopStats {
    pub(crate) wakes: u64,
    pub(crate) raw_keys: u64,
}

/// What a pass of the loop reads and whom it asks.
struct Context<'a, R> {
    health: &'a HealthState,
    counters: &'a Counters,
    requests: &'a R,
    /// Whether keyboard input is on this thread's desktop now.
    input_here: fn() -> windows::core::Result<bool>,
}

/// The watchdog's state on its thread: the decision, the tick timer, and
/// the reply the main thread owes.
struct Ticker<T> {
    watchdog: Watchdog,
    timer: T,
    /// The tick the timer is set for, until it signals or is cancelled.
    armed: Option<Instant>,
    /// The answer to the outstanding request.
    reply: Option<Receiver<ReinstallOutcome>>,
    /// Whether a raw key event was counted yet, for the one log line.
    raw_input_seen: bool,
    /// The heartbeat at the last tick.
    seen: u64,
    stats: LoopStats,
}

impl<T: TickTimer> Ticker<T> {
    fn new(now: Instant, timer: T, health: &HealthState) -> Self {
        let seen = health.callbacks();
        Self {
            watchdog: Watchdog::new(now, seen),
            timer,
            armed: None,
            reply: None,
            raw_input_seen: false,
            seen,
            stats: LoopStats::default(),
        }
    }

    fn stats(&self) -> LoopStats {
        self.stats
    }

    /// A raw key event that the keyboard hook should see. It counts only
    /// while the keyboard hook is installed (ruling I3): before `install`
    /// and after `uninstall` no callback can answer it.
    fn raw_event(&mut self, now: Instant, health: &HealthState) {
        self.stats.raw_keys += 1;
        if !health.hook_installed() {
            return;
        }
        if !self.raw_input_seen {
            self.raw_input_seen = true;
            log::info!("hook watchdog: raw keyboard input arrives");
        }
        self.watchdog.raw_event(now);
    }

    /// The timer signalled, and the wait took the signal: it is set for
    /// nothing until the next pass.
    fn fired(&mut self) {
        self.armed = None;
    }

    /// When the wait must return without the timer's signal: at the next
    /// tick, while ticking with a timer that could not be set (ruling I1).
    /// The wait's own timeout then brings the tick, on the system clock's
    /// 15.6 ms resolution, and the pass sets the timer again.
    fn wake_by(&self) -> Option<Instant> {
        match self.armed {
            Some(_) => None,
            None => self.watchdog.next_tick(),
        }
    }

    /// One pass after any wake: takes the main thread's answer, ticks if the
    /// tick is due by the clock, so that an early or stale signal changes
    /// nothing, asks for the reinstall a tick decided, and sets the timer for
    /// the decision's next tick, or cancels it (ruling I1).
    fn pass<R: RequestReinstall>(&mut self, now: Instant, context: &Context<'_, R>) {
        self.stats.wakes += 1;
        self.take_answer(now);
        if self.watchdog.next_tick().is_some_and(|due| due <= now) {
            let callbacks = context.health.callbacks();
            if callbacks != self.seen {
                self.seen = callbacks;
                // The hook answers: a failed reinstall before it is no
                // longer the hook's state (ruling M3).
                context.health.hook_answered();
            }
            // The per-event pump reads every queued message before the
            // pass, so no raw input is left behind.
            if let Verdict::Reinstall { request, .. } = self.watchdog.tick(now, callbacks, false) {
                self.ask(now, request, context);
            }
        }
        self.arm(now, context);
    }

    fn take_answer(&mut self, now: Instant) {
        let Some(reply) = &self.reply else {
            return;
        };
        match reply.try_recv() {
            Ok(outcome) => {
                self.reply = None;
                self.watchdog.answered(now, outcome);
            }
            Err(TryRecvError::Empty) => {}
            // The main thread dropped the request unanswered: it is ending.
            Err(TryRecvError::Disconnected) => {
                self.reply = None;
                self.watchdog.not_sent();
            }
        }
    }

    /// Sends `request` if keyboard input is on this desktop, where the hook
    /// would have seen it. On another desktop (a lock, the secure desktop)
    /// the evidence is dropped, and so it is when the check fails, which is
    /// counted (ruling M2).
    fn ask<R: RequestReinstall>(
        &mut self,
        now: Instant,
        request: ReinstallRequest,
        context: &Context<'_, R>,
    ) {
        match (context.input_here)() {
            Ok(true) => match context.requests.request(request) {
                Ok(reply) => {
                    if request.attempt >= 2 {
                        log::warn!(
                            "the keyboard hook still gets no key events after its reinstall; asking again after {} s of silence",
                            request.silent_for.as_secs()
                        );
                    }
                    self.reply = Some(reply);
                }
                Err(_) => self.watchdog.not_sent(),
            },
            Ok(false) => self.watchdog.declined(now),
            Err(_) => {
                context.counters.bump(Counter::WatchdogDesktopCheckFailed);
                self.watchdog.declined(now);
            }
        }
    }

    /// Sets the timer for the decision's next tick, every pass, so that no
    /// signal and no failed set loses a tick (ruling I1). A failed set is
    /// counted and tried again at the next pass, which the wait's timeout
    /// brings at the tick at the latest ([`wake_by`](Self::wake_by)).
    fn arm<R>(&mut self, now: Instant, context: &Context<'_, R>) {
        match plan(self.armed, self.watchdog.next_tick(), now) {
            Plan::Cancel => {
                if self.armed.take().is_some() {
                    // A timer that fails to stop signals once more, and the
                    // pass that follows changes nothing.
                    let _ = self.timer.cancel();
                }
            }
            Plan::Keep => {}
            Plan::Arm { after, at } => {
                // Setting the timer replaces the due time it had.
                self.armed = None;
                match self.timer.set(after) {
                    Ok(()) => {
                        self.armed = Some(at);
                        context.health.set_timer_failing(false);
                    }
                    Err(_) => {
                        context.counters.bump(Counter::WatchdogTimerNotSet);
                        context.health.set_timer_failing(true);
                    }
                }
            }
        }
    }
}

/// A thread message that, in the tests, stands for one raw key event.
#[cfg(test)]
const SYNTHETIC_RAW: u32 = WM_APP + 2;

/// Where the watchdog's raw key events come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    /// The raw input of every keyboard.
    Keyboard,
    /// `SYNTHETIC_RAW` thread messages, in the tests: no raw input
    /// registration, so that no test taps the keyboard of the machine that
    /// runs it, and no desktop check.
    #[cfg(test)]
    Synthetic,
}

impl Source {
    /// The queue bit of the raw key events this source batches: what the
    /// wait leaves out while ticking ([`wake_mask`]).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
        )
    )]
    fn batched_wake(self) -> QUEUE_STATUS_FLAGS {
        match self {
            Source::Keyboard => QS_RAWINPUT,
            #[cfg(test)]
            Source::Synthetic => QS_POSTMESSAGE,
        }
    }

    /// The message of the raw key events this source batches: what the
    /// pump leaves in the queue for the drain ([`pump_ranges`]).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
        )
    )]
    fn batched_message(self) -> u32 {
        match self {
            Source::Keyboard => WM_INPUT,
            #[cfg(test)]
            Source::Synthetic => SYNTHETIC_RAW,
        }
    }

    fn input_here(self) -> fn() -> windows::core::Result<bool> {
        match self {
            Source::Keyboard => desktop::input_is_on_this_desktop,
            #[cfg(test)]
            Source::Synthetic => || Ok(true),
        }
    }

    /// Whether `msg` is a synthetic raw key event of this source.
    fn is_synthetic_raw(self, msg: &MSG) -> bool {
        match self {
            Source::Keyboard => {
                // Only the test source reads `msg`; the daemon's has no
                // synthetic events.
                let _ = msg;
                false
            }
            #[cfg(test)]
            Source::Synthetic => msg.hwnd.is_invalid() && msg.message == SYNTHETIC_RAW,
        }
    }
}

/// The `hook-watchdog` thread. Dropping it stops it, as
/// [`stop`](Self::stop) does.
pub(crate) struct HookWatchdog {
    thread_id: u32,
    join: Option<JoinHandle<()>>,
    /// Receives once the thread has dropped everything it owns, with what
    /// its loop did if the loop ran.
    done: Receiver<Option<LoopStats>>,
}

impl HookWatchdog {
    /// How long [`start`](Self::start) waits for the thread to make its
    /// window, its registration and its timer.
    pub(crate) const START_TIMEOUT: Duration = Duration::from_secs(5);
    /// How long [`stop`](Self::stop) waits for the thread to end.
    pub(crate) const STOP_TIMEOUT: Duration = Duration::from_secs(1);

    /// Starts the thread, which compares `health`'s heartbeat with the raw
    /// keyboard input, sends its requests to `requests` and counts in
    /// `counters`. Returns once the thread has its window, its raw input
    /// registration and its timer; an error if it could not make one.
    pub(crate) fn start(
        health: &'static HealthState,
        requests: impl RequestReinstall,
        counters: &'static Counters,
    ) -> io::Result<Self> {
        Self::spawn(health, requests, counters, Source::Keyboard)
    }

    /// As [`start`](Self::start), with the synthetic raw key events of
    /// [`post_synthetic_raw`](Self::post_synthetic_raw) instead of the
    /// keyboard's.
    #[cfg(test)]
    pub(crate) fn start_synthetic(
        health: &'static HealthState,
        requests: impl RequestReinstall,
        counters: &'static Counters,
    ) -> io::Result<Self> {
        Self::spawn(health, requests, counters, Source::Synthetic)
    }

    fn spawn<R: RequestReinstall>(
        health: &'static HealthState,
        requests: R,
        counters: &'static Counters,
        source: Source,
    ) -> io::Result<Self> {
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (done_tx, done) = mpsc::sync_channel(1);
        let join = thread::Builder::new()
            .name("hook-watchdog".into())
            .spawn(move || {
                let stats = run_watchdog(health, &requests, counters, source, &ready_tx);
                let _ = done_tx.send(stats);
            })?;
        let started = ready
            .recv_timeout(Self::START_TIMEOUT)
            .map_err(|_| io::Error::other("the hook watchdog thread did not start"))?;
        let thread_id = started?;
        Ok(Self {
            thread_id,
            join: Some(join),
            done,
        })
    }

    /// One synthetic raw key event, for a watchdog of
    /// [`start_synthetic`](Self::start_synthetic).
    #[cfg(test)]
    pub(crate) fn post_synthetic_raw(&self) {
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; the message carries no pointer.
        let _ = unsafe { PostThreadMessageW(self.thread_id, SYNTHETIC_RAW, WPARAM(0), LPARAM(0)) };
    }

    /// Stops the thread; true if it ended within [`STOP_TIMEOUT`](Self::STOP_TIMEOUT).
    /// After that it is left to the end of the process.
    pub(crate) fn stop(mut self) -> bool {
        self.stop_thread().is_some()
    }

    /// As [`stop`](Self::stop), with what the thread's loop did: `None` if
    /// the thread did not end in time.
    #[cfg(test)]
    pub(crate) fn stop_counting(mut self) -> Option<LoopStats> {
        self.stop_thread().flatten()
    }

    /// Stops the thread: `None` if it did not end within
    /// [`STOP_TIMEOUT`](Self::STOP_TIMEOUT), else what its loop did, if
    /// the loop ran.
    fn stop_thread(&mut self) -> Option<Option<LoopStats>> {
        let Some(join) = self.join.take() else {
            return Some(None);
        };
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; WM_QUIT carries no pointer. A
        // thread that ended already only makes the call fail, and its done
        // message waits below.
        let _ = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        match self.done.recv_timeout(Self::STOP_TIMEOUT) {
            Ok(stats) => {
                let _ = join.join();
                Some(stats)
            }
            Err(RecvTimeoutError::Disconnected) => {
                let _ = join.join();
                Some(None)
            }
            Err(RecvTimeoutError::Timeout) => None,
        }
    }
}

impl Drop for HookWatchdog {
    fn drop(&mut self) {
        let _ = self.stop_thread();
    }
}

/// The message-only window raw input is delivered to; destroyed when
/// dropped, on the thread that made it.
struct SinkWindow(HWND);

impl SinkWindow {
    fn create() -> windows::core::Result<Self> {
        // SAFETY: "Message" is the system class of message-only windows, so
        // no class of ours is registered; the window name is null and no
        // creation data is passed. HWND_MESSAGE makes a window that no other
        // process enumerates or sends input to.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("Message"),
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
        }?;
        Ok(Self(hwnd))
    }
}

impl Drop for SinkWindow {
    fn drop(&mut self) {
        // SAFETY: `self.0` is the window `create` made on this thread, which
        // nothing else destroys; HWND is not Send, so this drop runs on the
        // creating thread, as DestroyWindow requires.
        if let Err(error) = unsafe { DestroyWindow(self.0) } {
            log::warn!("could not destroy the hook watchdog's window: {error}");
        }
    }
}

/// Says that the watchdog runs, from its start until it is dropped, also
/// on unwind, so that `swch status` never reads `hook ok` without it.
struct Watched<'a>(&'a HealthState);

impl<'a> Watched<'a> {
    fn new(health: &'a HealthState) -> Self {
        health.set_watched(true);
        Self(health)
    }
}

impl Drop for Watched<'_> {
    fn drop(&mut self) {
        self.0.set_watched(false);
    }
}

/// The sink window's raw keyboard registration; removed when dropped,
/// before the window is destroyed. Not `Send`: it belongs to the thread of
/// its window.
struct RawInputRegistration(PhantomData<*const ()>);

impl RawInputRegistration {
    fn register(sink: HWND) -> windows::core::Result<Self> {
        installer::SYSTEM.register_raw_keyboard(sink)?;
        Ok(Self(PhantomData))
    }
}

impl Drop for RawInputRegistration {
    fn drop(&mut self) {
        if let Err(error) = installer::SYSTEM.unregister_raw_keyboard() {
            log::warn!("could not remove the hook watchdog's raw input registration: {error}");
        }
    }
}

/// The watchdog thread: its prologue, its loop and its epilogue, which
/// logs what the loop did. The registration is removed and the window
/// destroyed when they drop, in that order, at the end. `None` if the loop
/// never ran.
fn run_watchdog<R: RequestReinstall>(
    health: &'static HealthState,
    requests: &R,
    counters: &'static Counters,
    source: Source,
    ready: &SyncSender<io::Result<u32>>,
) -> Option<LoopStats> {
    let fail = |what: &str, error: windows::core::Error| {
        let _ = ready.send(Err(io::Error::other(format!("{what}: {error}"))));
        None
    };
    let mut msg = MSG::default();
    // SAFETY: `msg` is a local that outlives the call, which only writes it;
    // the call creates this thread's message queue, as PostThreadMessageW
    // asks of the receiving thread, and removes nothing.
    let _ = unsafe { PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE) };
    // SAFETY: GetCurrentThreadId takes no arguments, touches no memory of
    // ours and cannot fail.
    let thread_id = unsafe { GetCurrentThreadId() };
    let window = match SinkWindow::create() {
        Ok(window) => window,
        Err(error) => return fail("could not create its message-only window", error),
    };
    let _registration = match source {
        Source::Keyboard => match RawInputRegistration::register(window.0) {
            Ok(registration) => Some(registration),
            Err(error) => return fail("could not register for raw keyboard input", error),
        },
        #[cfg(test)]
        Source::Synthetic => None,
    };
    let timer = match WaitableTimer::new() {
        Ok(timer) => timer,
        Err(error) => return fail("could not create its tick timer", error),
    };
    let _watched = Watched::new(health);
    // A starter that gave up waits no more, and nobody would stop this
    // thread.
    if ready.send(Ok(thread_id)).is_err() {
        return None;
    }
    let context = Context {
        health,
        counters,
        requests,
        input_here: source.input_here(),
    };
    let mut ticker = Ticker::new(Instant::now(), timer, health);
    run_loop(&mut ticker, window.0, source, &context, &mut msg);
    let stats = ticker.stats();
    // Once per run, in every build: the release daemon's evidence that the
    // thread wakes once per burst of keys and not per key, and not at
    // idle (manual checks M2 and M9).
    log::info!(
        "hook watchdog stopped after {} wakes for {} raw key events",
        stats.wakes,
        stats.raw_keys
    );
    Some(stats)
}

/// The loop, in the shape of the main loop: wait for the tick timer and the
/// messages at once, take the messages, then one pass, which sets the timer
/// again (ruling I1). Ends at `WM_QUIT`, or when the wait fails.
fn run_loop<R: RequestReinstall>(
    ticker: &mut Ticker<WaitableTimer>,
    window: HWND,
    source: Source,
    context: &Context<'_, R>,
    msg: &mut MSG,
) {
    loop {
        let timeout = ticker.wake_by().map_or(INFINITE, |at| {
            wait_ms(at.saturating_duration_since(Instant::now()))
        });
        // SAFETY: the slice holds the tick timer's handle, which `ticker`
        // keeps open for the whole call; the slice lives until the call
        // returns, which only reads it.
        let woke = unsafe {
            MsgWaitForMultipleObjectsEx(
                Some(&[ticker.timer.handle()]),
                timeout,
                QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            )
        };
        if woke == WAIT_OBJECT_0 {
            ticker.fired();
        } else if woke == WAIT_TIMEOUT {
            // The tick of a timer that could not be set: the pass below.
        } else if woke == WAIT_FAILED {
            log::error!(
                "the hook watchdog's message loop failed ({}) and stopped; a keyboard hook that Windows removes is not reinstalled until the daemon restarts",
                windows::core::Error::from_thread()
            );
            return;
        } else if pump(msg, window, source, ticker, context) {
            return;
        }
        ticker.pass(Instant::now(), context);
    }
}

/// The wait's wake mask: every kind of input, less the batched input
/// `batched` while `leave_out`. While the watchdog ticks, raw input waits in
/// the queue for the tick's drain and wakes nothing (ruling L1); idle, the
/// first raw key event of a burst ends the wait.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
    )
)]
fn wake_mask(batched: QUEUE_STATUS_FLAGS, leave_out: bool) -> QUEUE_STATUS_FLAGS {
    if leave_out {
        QS_ALLINPUT & !batched
    } else {
        QS_ALLINPUT
    }
}

/// The message ranges the pump takes: every message but `skip`, a message
/// above 0 and below 0xFFFF, which stays in the queue for the drain. As
/// Microsoft's batched read does: "only PeekMessage with explicit message
/// range filters ..., skipping WM_INPUT entirely" (Using Raw Input), since
/// "GetMessage removes the current WM_INPUT from the raw input queue"
/// (Raw Input Overview).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
    )
)]
fn pump_ranges(skip: u32) -> [(u32, u32); 2] {
    [(0, skip - 1), (skip + 1, 0xFFFF)]
}

/// `after` in whole milliseconds for a wait, rounded up so that the wait
/// never returns before it, and below `INFINITE`.
fn wait_ms(after: Duration) -> u32 {
    let ms = after.as_nanos().div_ceil(1_000_000);
    u32::try_from(ms).map_or(INFINITE - 1, |ms| ms.min(INFINITE - 1))
}

/// Takes this thread's messages until its queue is empty; true at
/// `WM_QUIT`. A raw key event is counted, then dispatched to the window's
/// procedure, `DefWindowProcW`, which frees it as Windows asks ("The
/// application must call DefWindowProc so the system can perform cleanup",
/// WM_INPUT).
fn pump<R: RequestReinstall>(
    msg: &mut MSG,
    window: HWND,
    source: Source,
    ticker: &mut Ticker<WaitableTimer>,
    context: &Context<'_, R>,
) -> bool {
    // SAFETY: `msg` is the loop's local and outlives every call, which only
    // writes it and removes the message from this thread's own queue.
    while unsafe { PeekMessageW(msg, None, 0, 0, PM_REMOVE) }.as_bool() {
        if msg.message == WM_QUIT {
            return true;
        }
        let raw = if msg.message == WM_INPUT && msg.hwnd == window {
            raw_key_event(msg.lParam, context.counters)
        } else {
            source.is_synthetic_raw(msg)
        };
        if raw {
            ticker.raw_event(Instant::now(), context.health);
        }
        // SAFETY: `msg` was filled by PeekMessageW above and outlives the
        // call, which only reads it; a thread message goes nowhere.
        unsafe { DispatchMessageW(msg) };
    }
    false
}

/// The make code of a keyboard overrun: no key. The `windows` crate has the
/// constant only under its `Win32_Devices_HumanInterfaceDevice` feature.
const KEYBOARD_OVERRUN_MAKE_CODE: u16 = 0xFF;

/// Whether the `WM_INPUT` with `lparam` is a keyboard event that the
/// keyboard hook would see too. The only function that reads key data; it
/// keeps none. A read that fails is counted in `raw_input_unread`.
fn raw_key_event(lparam: LPARAM, counters: &Counters) -> bool {
    let mut raw = RAWINPUT::default();
    let mut size = size_of::<RAWINPUT>() as u32;
    // SAFETY: `lparam` is the HRAWINPUT of a WM_INPUT this thread took and
    // has not dispatched yet, so its data is still there; `raw` is a local
    // RAWINPUT (8-aligned, as the call asks of its buffer) whose size is in
    // `size`, so the call writes within it or fails, and both outlive it.
    let read = unsafe {
        GetRawInputData(
            HRAWINPUT(lparam.0 as *mut std::ffi::c_void),
            RID_INPUT,
            Some(&mut raw as *mut RAWINPUT as *mut std::ffi::c_void),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    if read == u32::MAX {
        counters.bump(Counter::RawInputUnread);
        return false;
    }
    if raw.header.dwType != RIM_TYPEKEYBOARD.0 {
        return false;
    }
    // SAFETY: dwType says that the union holds a RAWKEYBOARD, which the call
    // wrote; it is a plain struct of integers.
    let keyboard = unsafe { raw.data.keyboard };
    qualifies(keyboard.MakeCode, keyboard.VKey)
}

/// How many bytes one buffered read takes at most: room for 64 of the
/// largest records, 76 keyboard records on x64.
const RAW_BUFFER_BYTES: usize = 64 * size_of::<RAWINPUT>();

/// The step from one record of a buffered read to the next: the record's
/// size rounded up to a pointer, winuser.h's `RAWINPUT_ALIGN`, which
/// NEXTRAWINPUTBLOCK applies.
const RAWINPUT_ALIGN: usize = size_of::<usize>();

/// The buffer of a buffered read: plain bytes, always initialised, so that
/// no padding byte of a typed RAWINPUT is ever read; aligned to 8 bytes,
/// which GetRawInputBuffer asks ("aligned on a DWORD boundary", and 8 bytes
/// under WOW64). Made once, on the watchdog thread's stack.
#[repr(C, align(8))]
struct RawBuffer([u8; RAW_BUFFER_BYTES]);

/// What one buffered read held: the key events the keyboard hook would
/// see, and the records that could not be read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BlockCount {
    keys: u64,
    unread: u64,
}

/// The `u32` at `at` in `bytes`, if it is there.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(size_of::<u32>())?;
    Some(u32::from_ne_bytes(bytes.get(at..end)?.try_into().ok()?))
}

/// The `u16` at `at` in `bytes`, if it is there.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(size_of::<u16>())?;
    Some(u16::from_ne_bytes(bytes.get(at..end)?.try_into().ok()?))
}

/// Counts the key events among the `count` records that a buffered read
/// wrote to `bytes`, as [`raw_key_event`] counts one. Like
/// [`raw_key_event`] it reads a record's make code and virtual key only to
/// tell whether the hook would see it, and keeps neither. A record whose
/// header or size does not fit ends the walk, and the records left are
/// unread; so is a keyboard record too short for its key fields.
fn count_raw_keys(bytes: &[u8], count: u32) -> BlockCount {
    let header = size_of::<RAWINPUTHEADER>();
    let body = offset_of!(RAWINPUT, data);
    let mut block = BlockCount::default();
    let mut at = 0_usize;
    for read in 0..count {
        let kind = u32_at(bytes, at + offset_of!(RAWINPUTHEADER, dwType));
        let size = u32_at(bytes, at + offset_of!(RAWINPUTHEADER, dwSize))
            .and_then(|size| usize::try_from(size).ok());
        let (Some(kind), Some(size)) = (kind, size) else {
            block.unread += u64::from(count - read);
            break;
        };
        if size < header || at.saturating_add(size) > bytes.len() {
            block.unread += u64::from(count - read);
            break;
        }
        if kind == RIM_TYPEKEYBOARD.0 {
            if size >= body + size_of::<RAWKEYBOARD>() {
                let make_code = u16_at(bytes, at + body + offset_of!(RAWKEYBOARD, MakeCode));
                let vkey = u16_at(bytes, at + body + offset_of!(RAWKEYBOARD, VKey));
                if let (Some(make_code), Some(vkey)) = (make_code, vkey)
                    && qualifies(make_code, vkey)
                {
                    block.keys += 1;
                }
            } else {
                block.unread += 1;
            }
        }
        at = (at + size).next_multiple_of(RAWINPUT_ALIGN);
    }
    block
}

/// Whether the keyboard's drain reads by `GetRawInputBuffer`. A 32-bit
/// build reads one `WM_INPUT` at a time instead, so that the WOW64 layout
/// of a buffered read never applies (cut L1.3 E11).
const BUFFERED: bool = cfg!(target_pointer_width = "64");

/// How long one drain reads before the loop makes a pass: a time budget,
/// not a count of records, so that a backlog of any size is read in full
/// within a few passes and the ticks stay on time (ruling V3-I1).
const DRAIN_BUDGET: Duration = Duration::from_millis(5);

/// One read of the watchdog thread's raw input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Read {
    /// Nothing was read.
    Empty,
    /// Records were read: the key events the hook would see among them,
    /// and those that could not be read.
    Records(BlockCount),
    /// The read failed: `GetRawInputBuffer` returned `(UINT)-1`.
    Failed,
    /// `WM_QUIT` came instead.
    Quit,
}

/// The reads of a drain: Windows' raw input queue of the watchdog thread
/// in the daemon, a fake in the tests.
trait RawReads {
    /// One buffered read: up to a buffer's worth of records.
    fn buffered(&mut self) -> Read;
    /// One standard read: the next `WM_INPUT`, read and dispatched, or
    /// `WM_QUIT`.
    fn standard(&mut self) -> Read;
    /// Whether the queue still says that it holds raw input.
    fn raw_pending(&mut self) -> bool;
}

/// What one drain read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Drained {
    /// The raw key events the hook would see.
    keys: u64,
    /// The budget ran out with raw input left: the loop drains again before
    /// it waits, and the decision keeps the callbacks' credit for it.
    pending: bool,
    /// The queue says it holds raw input that no read returns: the next
    /// wait leaves raw input out for one tick (ruling V3-M3).
    stuck: bool,
    /// `WM_QUIT` came.
    quit: bool,
}

impl Drained {
    fn add(&mut self, block: BlockCount, counters: &Counters) {
        self.keys += block.keys;
        for _ in 0..block.unread {
            counters.bump(Counter::RawInputUnread);
        }
    }
}

/// Reads raw input until the queue is empty, `WM_QUIT` comes or
/// [`DRAIN_BUDGET`] by `clock` is spent: by buffered reads if `buffered`,
/// else by standard reads. A failed buffered read is counted in
/// `raw_input_unread`, and standard reads take the rest of the drain. When
/// a read finds nothing while the queue still says it holds raw input, one
/// standard read takes it; if that finds nothing either, the drain counts
/// it in `raw_input_unread` and says the queue is stuck, so that the next
/// wait does not return for it at once, for ever (ruling V3-M3).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
    )
)]
fn drain(
    reads: &mut impl RawReads,
    buffered: bool,
    counters: &Counters,
    clock: impl Fn() -> Instant,
) -> Drained {
    let until = clock() + DRAIN_BUDGET;
    let mut buffered = buffered;
    let mut drained = Drained::default();
    loop {
        if clock() >= until {
            drained.pending = true;
            return drained;
        }
        let read = if buffered {
            reads.buffered()
        } else {
            reads.standard()
        };
        match read {
            Read::Records(block) => drained.add(block, counters),
            Read::Failed => {
                counters.bump(Counter::RawInputUnread);
                buffered = false;
            }
            Read::Quit => {
                drained.quit = true;
                return drained;
            }
            Read::Empty => {
                if !reads.raw_pending() {
                    return drained;
                }
                let fallback = if buffered {
                    reads.standard()
                } else {
                    Read::Empty
                };
                match fallback {
                    Read::Records(block) => drained.add(block, counters),
                    Read::Quit => {
                        drained.quit = true;
                        return drained;
                    }
                    Read::Empty | Read::Failed => {
                        if reads.raw_pending() {
                            counters.bump(Counter::RawInputUnread);
                            drained.stuck = true;
                        }
                        return drained;
                    }
                }
            }
        }
    }
}

/// The next wait's wake mask and its timeout in ms, after the drain
/// `drained`: none while raw input is left, so the thread drains again
/// before it waits (ruling V3-I1); the tick the timer could not take
/// (`wake_by`, ruling I1); and one tick without raw input in the mask after
/// a stuck queue (ruling V3-M3).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
    )
)]
fn next_wait(
    batched: QUEUE_STATUS_FLAGS,
    ticking: bool,
    wake_by: Option<Instant>,
    drained: Drained,
    now: Instant,
) -> (QUEUE_STATUS_FLAGS, u32) {
    let mask = wake_mask(batched, ticking || drained.stuck);
    if drained.pending {
        return (mask, 0);
    }
    let back_off = drained.stuck.then(|| now + TICK);
    let until = match (wake_by, back_off) {
        (Some(tick), Some(back_off)) => Some(tick.min(back_off)),
        (tick, back_off) => tick.or(back_off),
    };
    let timeout = until.map_or(INFINITE, |at| wait_ms(at.saturating_duration_since(now)));
    (mask, timeout)
}

/// The raw input reads of the watchdog thread's own queue.
struct KeyboardReads<'a> {
    /// The sink window raw input is delivered to.
    window: HWND,
    buffer: &'a mut RawBuffer,
    msg: &'a mut MSG,
    counters: &'a Counters,
}

impl RawReads for KeyboardReads<'_> {
    fn buffered(&mut self) -> Read {
        let mut size = RAW_BUFFER_BYTES as u32;
        // SAFETY: `self.buffer` is RAW_BUFFER_BYTES of initialised bytes,
        // aligned to 8 as the call asks, alive for the call; `size` says how
        // many bytes it holds, so the call writes within it; the header size
        // is RAWINPUTHEADER's. The call reads the calling thread's queue,
        // which is this thread's.
        let count = unsafe {
            GetRawInputBuffer(
                Some(self.buffer.0.as_mut_ptr().cast::<RAWINPUT>()),
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32,
            )
        };
        match count {
            0 => Read::Empty,
            u32::MAX => Read::Failed,
            count => Read::Records(count_raw_keys(&self.buffer.0, count)),
        }
    }

    fn standard(&mut self) -> Read {
        // SAFETY: `self.msg` outlives the call, which only writes it and
        // removes the message from this thread's own queue; `self.window` is
        // this thread's live sink window.
        let found =
            unsafe { PeekMessageW(self.msg, Some(self.window), WM_INPUT, WM_INPUT, PM_REMOVE) };
        if !found.as_bool() {
            return Read::Empty;
        }
        if self.msg.message == WM_QUIT {
            return Read::Quit;
        }
        let key = raw_key_event(self.msg.lParam, self.counters);
        // SAFETY: `self.msg` was filled by PeekMessageW above and outlives the
        // call, which only reads it; the window's procedure is
        // DefWindowProcW, which frees the raw input, as WM_INPUT asks.
        unsafe { DispatchMessageW(self.msg) };
        Read::Records(BlockCount {
            keys: u64::from(key),
            unread: 0,
        })
    }

    fn raw_pending(&mut self) -> bool {
        // SAFETY: GetQueueStatus takes the flags by value, reads no memory of
        // ours and reports this thread's own queue.
        let status = unsafe { GetQueueStatus(QS_RAWINPUT) };
        // The high word names the kinds of message in the queue now.
        (status >> 16) & QS_RAWINPUT.0 != 0
    }
}

/// Drains the raw keyboard input queued for `window`, this thread's sink
/// window, through `buffer`, made once on the thread's stack.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"
    )
)]
fn drain_keyboard(
    window: HWND,
    buffer: &mut RawBuffer,
    counters: &Counters,
    msg: &mut MSG,
) -> Drained {
    let mut reads = KeyboardReads {
        window,
        buffer,
        msg,
        counters,
    };
    drain(&mut reads, BUFFERED, counters, Instant::now)
}

/// Whether a raw keyboard event with `make_code` and `vkey` reaches the
/// keyboard hook: no overrun, and a virtual key in the hook's documented
/// range 1-254 (KBDLLHOOKSTRUCT). A key of VK 255, which some Fn
/// combinations send, never reaches the hook and must not count as its
/// silence. As Microsoft's sample does: "Ignore key overrun state and keys
/// not mapped to any virtual key code" (RAWKEYBOARD).
fn qualifies(make_code: u16, vkey: u16) -> bool {
    make_code != KEYBOARD_OVERRUN_MAKE_CODE && (1..=254).contains(&vkey)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::E_ACCESSDENIED;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_VOLUME_UP};
    use windows::Win32::UI::Input::RIM_TYPEHID;

    use super::decision::SILENCE;
    use super::*;
    use crate::core::executor::testing::leaked_counters;
    use crate::hook::health::{HealthState, ReinstallOutcome, ReinstallRequest};
    use crate::hook::main_thread::MainThreadError;
    use crate::logging::counters::{Counter, Counters};

    // No test here registers for raw input or installs a hook: the thread
    // tests start the watchdog with synthetic raw events, which skip the
    // registration, and the loop tests drive its passes on this thread
    // with a fake timer and a fake clock.

    /// A requester that records each request with the time it came and
    /// keeps the reply open without answering, as a busy main thread does;
    /// or, made with `answering`, answers each at once.
    struct Recorder {
        sent: mpsc::Sender<(Instant, ReinstallRequest)>,
        waiting: Mutex<Vec<SyncSender<ReinstallOutcome>>>,
        answer: Option<ReinstallOutcome>,
        /// How many of the next requests the queue refuses as full.
        refusals: AtomicU32,
    }

    impl Recorder {
        fn new() -> (Self, Receiver<(Instant, ReinstallRequest)>) {
            Self::answering(None)
        }

        fn answering(
            answer: Option<ReinstallOutcome>,
        ) -> (Self, Receiver<(Instant, ReinstallRequest)>) {
            let (sent, requests) = mpsc::channel();
            let recorder = Self {
                sent,
                waiting: Mutex::new(Vec::new()),
                answer,
                refusals: AtomicU32::new(0),
            };
            (recorder, requests)
        }
    }

    impl RequestReinstall for Recorder {
        fn request(
            &self,
            request: ReinstallRequest,
        ) -> Result<Receiver<ReinstallOutcome>, MainThreadError> {
            if self
                .refusals
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |left| {
                    left.checked_sub(1)
                })
                .is_ok()
            {
                return Err(MainThreadError::Busy);
            }
            let (reply, answer) = mpsc::sync_channel(1);
            if let Some(outcome) = self.answer {
                let _ = reply.try_send(outcome);
            }
            self.waiting
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(reply);
            let _ = self.sent.send((Instant::now(), request));
            Ok(answer)
        }
    }

    /// What the watchdog did to a [`FakeTimer`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TimerCall {
        Set(Duration),
        Cancel,
    }

    /// A tick timer that Windows never sees: it records each call, and
    /// refuses as many sets as asked.
    #[derive(Default)]
    struct FakeTimer {
        calls: RefCell<Vec<TimerCall>>,
        refusals: std::cell::Cell<u32>,
    }

    impl TickTimer for FakeTimer {
        fn set(&self, after: Duration) -> windows::core::Result<()> {
            if self.refusals.get() > 0 {
                self.refusals.set(self.refusals.get() - 1);
                return Err(E_ACCESSDENIED.into());
            }
            self.calls.borrow_mut().push(TimerCall::Set(after));
            Ok(())
        }

        fn cancel(&self) -> windows::core::Result<()> {
            self.calls.borrow_mut().push(TimerCall::Cancel);
            Ok(())
        }
    }

    fn input_here() -> windows::core::Result<bool> {
        Ok(true)
    }

    fn input_unknown() -> windows::core::Result<bool> {
        Err(E_ACCESSDENIED.into())
    }

    fn input_elsewhere() -> windows::core::Result<bool> {
        Ok(false)
    }

    /// The watchdog's loop on this thread, on a fake clock in whole
    /// milliseconds, with a fake timer that signals `early` ms before its
    /// due time, and a wait that times out where the loop's would.
    struct Drive {
        ticker: Ticker<FakeTimer>,
        start: Instant,
        health: &'static HealthState,
        counters: &'static Counters,
        recorder: Recorder,
        sent: Receiver<(Instant, ReinstallRequest)>,
        input_here: fn() -> windows::core::Result<bool>,
        /// When the fake timer is due, in ms.
        due: Option<u64>,
        early: u64,
        /// When the wait times out, in ms.
        timeout: Option<u64>,
        /// When each request was sent, in ms.
        requests: Vec<u64>,
        /// How many passes the loop made.
        passes: u64,
    }

    impl Drive {
        fn new(recorder: (Recorder, Receiver<(Instant, ReinstallRequest)>)) -> Self {
            let start = Instant::now();
            let health: &'static HealthState = Box::leak(Box::new(HealthState::new()));
            health.set_hook_installed(true);
            Self {
                ticker: Ticker::new(start, FakeTimer::default(), health),
                start,
                health,
                counters: leaked_counters(),
                recorder: recorder.0,
                sent: recorder.1,
                input_here,
                due: None,
                early: 0,
                timeout: None,
                requests: Vec::new(),
                passes: 0,
            }
        }

        fn at(&self, ms: u64) -> Instant {
            self.start + Duration::from_millis(ms)
        }

        /// One pass of the loop at `ms`, and what it did to the timer and
        /// the requester.
        fn pass(&mut self, ms: u64) {
            let context = Context {
                health: self.health,
                counters: self.counters,
                requests: &self.recorder,
                input_here: self.input_here,
            };
            self.ticker.pass(self.at(ms), &context);
            self.passes += 1;
            for call in self.ticker.timer.calls.borrow_mut().drain(..) {
                self.due = match call {
                    TimerCall::Set(after) => {
                        Some(ms + u64::try_from(after.as_millis()).expect("a short wait"))
                    }
                    TimerCall::Cancel => None,
                };
            }
            while self.sent.try_recv().is_ok() {
                self.requests.push(ms);
            }
            self.timeout = self.ticker.wake_by().map(|at| {
                let after = at.saturating_duration_since(self.at(ms));
                ms + u64::from(wait_ms(after))
            });
        }

        /// Runs the clock from `from` to `to` ms: at each ms the raw events
        /// `raw` names, each followed by a pass, then the timer's signal or
        /// the wait's timeout, if one is due, followed by a pass.
        fn run(&mut self, from: u64, to: u64, raw: impl Fn(u64) -> bool) {
            for ms in from..=to {
                if raw(ms) {
                    let now = self.at(ms);
                    self.ticker.raw_event(now, self.health);
                    self.pass(ms);
                }
                if let Some(due) = self.due
                    && ms + self.early >= due
                {
                    self.due = None;
                    self.ticker.fired();
                    self.pass(ms);
                } else if self.timeout.is_some_and(|timeout| ms >= timeout) {
                    self.pass(ms);
                }
            }
        }
    }

    /// Raw key events every 30 ms from 0 to 300 ms, so that every tick
    /// after them is reached only through the timer.
    fn a_burst(ms: u64) -> bool {
        ms <= 300 && ms.is_multiple_of(30)
    }

    #[test]
    fn an_early_timer_signal_does_not_lose_the_tick() {
        // Ruling I1: a signal before the tick's due time, here every one, is
        // ignored by the clock, and the pass sets the timer again for the
        // same tick, as the main loop's plan_wake does.
        let mut drive = Drive::new(Recorder::new());
        drive.early = 10;
        drive.run(0, 3000, a_burst);
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            (1000..=1500).contains(&drive.requests[0]),
            "{:?}",
            drive.requests
        );
    }

    #[test]
    fn a_timer_that_refuses_once_is_set_again_at_the_next_pass() {
        // Ruling I1: a failed set is counted, the wait times out at the tick
        // instead, and the pass sets the timer again. The set that fails is
        // the first after the burst, so no raw event brings a pass.
        let mut drive = Drive::new(Recorder::new());
        drive.run(0, 300, a_burst);
        drive.ticker.timer.refusals.set(1);
        drive.run(301, 600, a_burst);
        assert!(
            drive.health.snapshot().timer_failing,
            "the status says the timer fails"
        );
        drive.run(601, 3000, a_burst);
        assert!(
            !drive.health.snapshot().timer_failing,
            "the timer works again"
        );
        assert_eq!(drive.counters.get(Counter::WatchdogTimerNotSet), 1);
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            (1000..=1500).contains(&drive.requests[0]),
            "{:?}",
            drive.requests
        );
    }

    #[test]
    fn a_reply_of_the_main_thread_is_taken_at_the_next_pass() {
        // The main thread answers each request at once with Reinstalled and
        // the callback stays silent: the answer frees the next request,
        // which waits for the doubled window.
        let mut drive = Drive::new(Recorder::answering(Some(ReinstallOutcome::Reinstalled)));
        drive.run(0, 6000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests.len(), 2, "{:?}", drive.requests);
        let gap = drive.requests[1] - drive.requests[0];
        assert!((2000..2600).contains(&gap), "{:?}", drive.requests);
    }

    #[test]
    fn a_failed_reinstall_is_no_longer_reported_once_the_hook_answers() {
        // Ruling M3: the main thread records a failed attempt and keeps the
        // old hook; a tick that sees a callback again clears the report.
        let mut drive = Drive::new(Recorder::new());
        drive.run(0, 300, |ms| ms.is_multiple_of(30));
        drive.health.record_failed_attempt();
        drive.run(301, 600, |ms| ms.is_multiple_of(30));
        assert!(
            drive.health.snapshot().last_attempt_failed,
            "no callback yet"
        );
        drive.health.callback_seen();
        drive.run(601, 800, |ms| ms.is_multiple_of(30));
        assert!(!drive.health.snapshot().last_attempt_failed);
    }

    #[test]
    fn raw_input_before_the_hook_is_installed_is_no_evidence() {
        // Ruling I3: the watchdog starts before the hooks, and counts raw
        // input only while the keyboard hook is installed, so a key typed
        // before `install` draws no reinstall of the new hook.
        let mut drive = Drive::new(Recorder::new());
        drive.health.set_hook_installed(false);
        drive.run(0, 600, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.due, None, "no tick while no hook is installed");
        drive.health.set_hook_installed(true);
        drive.run(601, 4000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            drive.requests[0] >= 610 + 1000,
            "the silence starts at the first raw event counted: {:?}",
            drive.requests
        );
    }

    #[test]
    fn a_failed_desktop_check_declines_and_is_counted() {
        // Ruling M2: a request declined because the desktop check failed
        // leaves a trace in the counters.
        let mut drive = Drive::new(Recorder::new());
        drive.input_here = input_unknown;
        drive.run(0, 3000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests, [] as [u64; 0]);
        assert!(
            drive.counters.get(Counter::WatchdogDesktopCheckFailed) >= 1,
            "the declined decision is counted"
        );
    }

    #[test]
    fn a_request_is_not_sent_while_input_is_on_another_desktop() {
        // The desktop check says the keys go elsewhere (a lock, the secure
        // desktop): the hook could not have seen them, so no request is sent,
        // and a plain "elsewhere" is no failed check.
        let mut drive = Drive::new(Recorder::new());
        drive.input_here = input_elsewhere;
        drive.run(0, 3000, |ms| ms.is_multiple_of(30));
        assert_eq!(
            drive.requests,
            [] as [u64; 0],
            "no request while input is on another desktop"
        );
        assert_eq!(
            drive.counters.get(Counter::WatchdogDesktopCheckFailed),
            0,
            "a desktop that is not ours is not a failed check"
        );
    }

    #[test]
    fn a_request_the_main_threads_queue_refused_is_asked_again() {
        // A full queue sends nothing: the decision is not outstanding, and a
        // later tick asks again. A watchdog that kept waiting for a reply
        // that never comes would never ask again.
        let mut drive = Drive::new(Recorder::new());
        drive.recorder.refusals.store(2, Ordering::Release);
        drive.run(0, 4000, |ms| ms.is_multiple_of(30));
        assert_eq!(
            drive.recorder.refusals.load(Ordering::Acquire),
            0,
            "both refusals were used"
        );
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            drive.requests[0] > 1000,
            "the first decision, at about 1000 ms, was refused: {:?}",
            drive.requests
        );
    }

    #[test]
    fn the_wait_leaves_the_batched_input_out_only_while_ticking() {
        // Ruling L1: idle, the first raw key event of a burst ends the
        // wait; ticking, raw input waits in the queue for the tick's drain,
        // and every other kind of message still ends the wait.
        assert_eq!(wake_mask(QS_RAWINPUT, false), QS_ALLINPUT);
        let ticking = wake_mask(QS_RAWINPUT, true);
        assert_eq!(ticking, QUEUE_STATUS_FLAGS(0xFF));
        assert!(!ticking.contains(QS_RAWINPUT), "{ticking:?}");
        assert!(
            ticking.contains(QS_POSTMESSAGE | QS_SENDMESSAGE | QS_TIMER),
            "{ticking:?}"
        );
        assert_eq!(Source::Keyboard.batched_wake(), QS_RAWINPUT);
        assert_eq!(Source::Synthetic.batched_wake(), QS_POSTMESSAGE);
        assert_eq!(Source::Keyboard.batched_message(), WM_INPUT);
        assert_eq!(Source::Synthetic.batched_message(), SYNTHETIC_RAW);
    }

    #[test]
    fn the_pump_leaves_only_the_batched_message_in_the_queue() {
        // Cut L1.3 E7: a pump that took WM_INPUT would leave the buffered
        // read nothing to read.
        for skip in [WM_INPUT, SYNTHETIC_RAW] {
            let ranges = pump_ranges(skip);
            for message in 0..=0xFFFF_u32 {
                let taken = ranges
                    .iter()
                    .any(|&(low, high)| (low..=high).contains(&message));
                assert_eq!(
                    taken,
                    message != skip,
                    "message {message:#x}, {skip:#x} left in the queue"
                );
            }
        }
    }

    #[test]
    fn the_raw_input_registration_asks_for_no_device_notifications() {
        // Cut L1.3 E10: "Don't use RIDEV_DEVNOTIFY with this pattern", whose
        // WM_INPUT_DEVICE_CHANGE the range-filtered pump cannot take, so the
        // thread would spin. The token is assembled, so that this file does
        // not match itself.
        let text = source("src/hook/installer.rs");
        assert!(
            text.lines().any(|line| line.contains("RIDEV_INPUTSINK")),
            "the registration is a sink"
        );
        let notify = concat!("RIDEV_", "DEVNOTIFY");
        let sites: Vec<&str> = text.lines().filter(|line| line.contains(notify)).collect();
        assert!(sites.is_empty(), "{sites:?}");
    }

    /// The size of a keyboard record of a buffered read.
    const KEYBOARD_RECORD: usize = offset_of!(RAWINPUT, data) + size_of::<RAWKEYBOARD>();

    /// Writes the bytes of `value` at `at`, as Windows lays them out.
    fn put(bytes: &mut [u8], at: usize, value: &[u8]) {
        bytes[at..at + value.len()].copy_from_slice(value);
    }

    /// Writes a record header of `kind` and `size` at `at`.
    fn put_header(bytes: &mut [u8], at: usize, kind: u32, size: u32) {
        put(
            bytes,
            at + offset_of!(RAWINPUTHEADER, dwType),
            &kind.to_ne_bytes(),
        );
        put(
            bytes,
            at + offset_of!(RAWINPUTHEADER, dwSize),
            &size.to_ne_bytes(),
        );
    }

    /// Writes a keyboard record of `make_code` and `vkey` at `at`, and
    /// returns where the next record starts.
    fn put_key(bytes: &mut [u8], at: usize, make_code: u16, vkey: u16) -> usize {
        put_header(bytes, at, RIM_TYPEKEYBOARD.0, KEYBOARD_RECORD as u32);
        let body = at + offset_of!(RAWINPUT, data);
        put(
            bytes,
            body + offset_of!(RAWKEYBOARD, MakeCode),
            &make_code.to_ne_bytes(),
        );
        put(
            bytes,
            body + offset_of!(RAWKEYBOARD, VKey),
            &vkey.to_ne_bytes(),
        );
        (at + KEYBOARD_RECORD).next_multiple_of(size_of::<usize>())
    }

    fn empty_buffer() -> RawBuffer {
        RawBuffer([0; RAW_BUFFER_BYTES])
    }

    #[test]
    fn a_buffered_read_counts_the_key_events_the_hook_sees() {
        // Cut L1.2: the keys of a buffered read count as the standard read
        // counts them (qualifies): no overrun, no VK 0 or 255.
        let mut buffer = empty_buffer();
        let mut at = 0;
        for (make_code, vkey) in [
            (0x1E, VK_A.0),
            (0x1E, VK_A.0),
            (0xFF, VK_A.0),
            (0x1E, 0),
            (0x1E, 255),
            (0, VK_VOLUME_UP.0),
        ] {
            at = put_key(&mut buffer.0, at, make_code, vkey);
        }
        assert_eq!(
            count_raw_keys(&buffer.0, 6),
            BlockCount { keys: 3, unread: 0 }
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn the_walk_steps_by_each_records_size_rounded_up_to_a_pointer() {
        // Cut L1.3 E11: the next record starts at the record's size rounded
        // up to 8 bytes in a 64-bit process (winuser.h's RAWINPUT_ALIGN).
        let mut buffer = empty_buffer();
        assert_eq!(put_key(&mut buffer.0, 0, 0x1E, VK_A.0), 40);
        put_header(&mut buffer.0, 40, RIM_TYPEHID.0, 36);
        assert_eq!(put_key(&mut buffer.0, 80, 0x1E, VK_A.0), 120);
        assert_eq!(
            count_raw_keys(&buffer.0, 3),
            BlockCount { keys: 2, unread: 0 }
        );
    }

    #[test]
    fn a_record_that_does_not_fit_ends_the_walk_as_unread() {
        let mut buffer = empty_buffer();
        let at = put_key(&mut buffer.0, 0, 0x1E, VK_A.0);
        put_header(&mut buffer.0, at, RIM_TYPEKEYBOARD.0, 4000);
        assert_eq!(
            count_raw_keys(&buffer.0, 3),
            BlockCount { keys: 1, unread: 2 }
        );
    }

    #[test]
    fn a_record_shorter_than_its_header_ends_the_walk() {
        // A size of 0 would never move the walk on.
        let mut buffer = empty_buffer();
        let at = put_key(&mut buffer.0, 0, 0x1E, VK_A.0);
        put_header(&mut buffer.0, at, RIM_TYPEKEYBOARD.0, 0);
        put_key(&mut buffer.0, at + 40, 0x1E, VK_A.0);
        assert_eq!(
            count_raw_keys(&buffer.0, 3),
            BlockCount { keys: 1, unread: 2 }
        );
    }

    #[test]
    fn a_keyboard_record_too_short_for_its_keys_is_unread() {
        // A keyboard record that ends before its key fields cannot be
        // read; the walk goes on by its size.
        let mut buffer = empty_buffer();
        let short = size_of::<RAWINPUTHEADER>() + 4;
        put_header(&mut buffer.0, 0, RIM_TYPEKEYBOARD.0, short as u32);
        put_key(
            &mut buffer.0,
            short.next_multiple_of(size_of::<usize>()),
            0x1E,
            VK_A.0,
        );
        assert_eq!(
            count_raw_keys(&buffer.0, 2),
            BlockCount { keys: 1, unread: 1 }
        );
    }

    #[test]
    fn the_buffer_holds_64_records_aligned_for_the_buffered_read() {
        // Cut L1.3 E11: "Pointer should be aligned on a DWORD (32-bit)
        // boundary" (GetRawInputBuffer); 8, for the WOW64 note.
        assert!(align_of::<RawBuffer>() >= 8);
        assert_eq!(size_of::<RawBuffer>(), 64 * size_of::<RAWINPUT>());
        #[cfg(target_pointer_width = "64")]
        assert_eq!(KEYBOARD_RECORD, 40, "a keyboard record on x64");
    }

    /// Raw input reads that Windows never sees: each read takes the next
    /// answer of its list, then `Empty`; the queue says it holds raw input
    /// while `pending`, which a standard read clears if `standard_clears`.
    #[derive(Default)]
    struct FakeReads {
        buffered: VecDeque<Read>,
        standard: VecDeque<Read>,
        pending: bool,
        standard_clears: bool,
        buffered_calls: usize,
        standard_calls: usize,
    }

    impl RawReads for FakeReads {
        fn buffered(&mut self) -> Read {
            self.buffered_calls += 1;
            self.buffered.pop_front().unwrap_or(Read::Empty)
        }

        fn standard(&mut self) -> Read {
            self.standard_calls += 1;
            if self.standard_clears {
                self.pending = false;
            }
            self.standard.pop_front().unwrap_or(Read::Empty)
        }

        fn raw_pending(&mut self) -> bool {
            self.pending
        }
    }

    fn records(keys: u64, unread: u64) -> Read {
        Read::Records(BlockCount { keys, unread })
    }

    /// A clock that stands still.
    fn still() -> impl Fn() -> Instant {
        let now = Instant::now();
        move || now
    }

    /// A clock that moves on 1 ms at each reading.
    fn one_ms_per_reading() -> impl Fn() -> Instant {
        let now = std::cell::Cell::new(Instant::now());
        move || {
            let at = now.get();
            now.set(at + Duration::from_millis(1));
            at
        }
    }

    #[test]
    fn a_drain_reads_until_the_queue_is_empty() {
        // Ruling V3-I1: a drain reads until the queue is empty; its records
        // are counted, and so are those it could not read.
        let counters = leaked_counters();
        let mut reads = FakeReads {
            buffered: [records(3, 0), records(2, 1), records(4, 0)].into(),
            ..FakeReads::default()
        };
        assert_eq!(
            drain(&mut reads, true, counters, still()),
            Drained {
                keys: 9,
                ..Drained::default()
            }
        );
        assert_eq!(reads.buffered_calls, 4, "until a read finds nothing");
        assert_eq!(counters.get(Counter::RawInputUnread), 1);
    }

    #[test]
    fn a_drain_that_spends_its_budget_leaves_the_rest_for_the_next() {
        // Ruling V3-I1: the bound of a drain is a time budget, not a count
        // of records; a drain that spends it says raw input is left, and
        // the next drain goes on where it stopped, so nothing is lost.
        let counters = leaked_counters();
        let mut reads = FakeReads {
            buffered: std::iter::repeat_n(records(76, 0), 100).collect(),
            ..FakeReads::default()
        };
        let clock = one_ms_per_reading();
        let first = drain(&mut reads, true, counters, &clock);
        assert!(first.pending, "{first:?}");
        assert!(first.keys > 0 && first.keys < 7_600, "{first:?}");
        let mut keys = first.keys;
        let mut drains = 1;
        loop {
            let next = drain(&mut reads, true, counters, &clock);
            keys += next.keys;
            drains += 1;
            if !next.pending {
                break;
            }
            assert!(drains < 100, "the drains end");
        }
        assert_eq!(keys, 7_600, "every key event, over {drains} drains");
    }

    #[test]
    fn a_failed_buffered_read_is_counted_and_the_standard_read_takes_the_rest() {
        let counters = leaked_counters();
        let mut reads = FakeReads {
            buffered: [Read::Failed, records(50, 0)].into(),
            standard: [records(1, 0), records(0, 0), records(1, 0)].into(),
            ..FakeReads::default()
        };
        assert_eq!(
            drain(&mut reads, true, counters, still()),
            Drained {
                keys: 2,
                ..Drained::default()
            }
        );
        assert_eq!(
            reads.buffered_calls, 1,
            "no buffered read after a failed one"
        );
        assert_eq!(counters.get(Counter::RawInputUnread), 1);
    }

    #[test]
    fn raw_input_that_no_buffered_read_returns_is_taken_by_one_standard_read() {
        // Ruling V3-M3: a buffered read finds nothing while the queue still
        // says it holds raw input; one standard read takes it.
        let counters = leaked_counters();
        let mut reads = FakeReads {
            standard: [records(1, 0)].into(),
            pending: true,
            standard_clears: true,
            ..FakeReads::default()
        };
        assert_eq!(
            drain(&mut reads, true, counters, still()),
            Drained {
                keys: 1,
                ..Drained::default()
            }
        );
        assert_eq!(reads.standard_calls, 1);
        assert_eq!(counters.get(Counter::RawInputUnread), 0);
    }

    #[test]
    fn raw_input_that_no_read_returns_is_counted_and_the_wait_backs_off_for_a_tick() {
        // Ruling V3-M3: the queue says it holds raw input that neither read
        // returns, which would end every wait with raw input in its mask at
        // once, for ever. The drain counts it once and stops, and the next
        // wait leaves raw input out until a tick has passed: no spin.
        let counters = leaked_counters();
        let mut reads = FakeReads {
            pending: true,
            ..FakeReads::default()
        };
        let drained = drain(&mut reads, true, counters, still());
        assert_eq!(
            drained,
            Drained {
                stuck: true,
                ..Drained::default()
            }
        );
        assert_eq!((reads.buffered_calls, reads.standard_calls), (1, 1));
        assert_eq!(counters.get(Counter::RawInputUnread), 1);
        let now = Instant::now();
        let (mask, timeout) = next_wait(QS_RAWINPUT, false, None, drained, now);
        assert!(!mask.contains(QS_RAWINPUT), "{mask:?}");
        assert_eq!(timeout, 250, "one tick");
    }

    #[test]
    fn a_drain_ends_at_wm_quit() {
        let counters = leaked_counters();
        let mut reads = FakeReads {
            standard: [records(1, 0), Read::Quit, records(1, 0)].into(),
            ..FakeReads::default()
        };
        assert_eq!(
            drain(&mut reads, false, counters, still()),
            Drained {
                keys: 1,
                quit: true,
                ..Drained::default()
            }
        );
    }

    #[test]
    fn the_next_wait_never_waits_while_raw_input_is_left() {
        // Ruling V3-I1: a drain that spent its budget is followed by another
        // before the thread waits; otherwise the wait leaves raw input out
        // while ticking and times out at the tick a timer could not take.
        let now = Instant::now();
        let left = Drained {
            keys: 300,
            pending: true,
            ..Drained::default()
        };
        assert_eq!(next_wait(QS_RAWINPUT, true, None, left, now).1, 0);
        assert_eq!(
            next_wait(QS_RAWINPUT, false, None, Drained::default(), now),
            (QS_ALLINPUT, INFINITE)
        );
        assert_eq!(
            next_wait(QS_RAWINPUT, true, None, Drained::default(), now),
            (QUEUE_STATUS_FLAGS(0xFF), INFINITE)
        );
        assert_eq!(
            next_wait(
                QS_RAWINPUT,
                true,
                Some(now + Duration::from_millis(100)),
                Drained::default(),
                now
            ),
            (QUEUE_STATUS_FLAGS(0xFF), 100)
        );
    }

    #[test]
    fn the_buffered_read_of_a_thread_without_raw_input_reads_nothing() {
        // Critic OQ3, an FFI smoke test of the buffered read's sizes on a
        // thread of its own with a sink window and no raw input
        // registration, so it reads only that thread's empty queue.
        // Microsoft Learn names two outcomes for GetRawInputBuffer: the
        // number of records written, 0 for none, or (UINT)-1 for an error.
        // Either way the drain reads nothing.
        let counters = leaked_counters();
        let (outcome, drained) = thread::spawn(move || {
            let window = SinkWindow::create().expect("a message-only window");
            let mut buffer = empty_buffer();
            let mut msg = MSG::default();
            let mut size = RAW_BUFFER_BYTES as u32;
            // SAFETY: `buffer` is RAW_BUFFER_BYTES of initialised bytes,
            // aligned to 8, alive for the call; `size` says how many bytes it
            // holds, so the call writes within it; the header size is
            // RAWINPUTHEADER's, as the call asks.
            let outcome = unsafe {
                GetRawInputBuffer(
                    Some(buffer.0.as_mut_ptr().cast::<RAWINPUT>()),
                    &mut size,
                    size_of::<RAWINPUTHEADER>() as u32,
                )
            };
            let drained = drain_keyboard(window.0, &mut buffer, counters, &mut msg);
            (outcome, drained)
        })
        .join()
        .expect("the smoke test's thread ends");
        let unread = counters.get(Counter::RawInputUnread);
        match outcome {
            0 => assert_eq!(unread, 0, "no records: nothing was unread"),
            u32::MAX => assert_eq!(unread, 1, "an error: the failed read is counted"),
            records => panic!("{records} records from a queue with no raw input"),
        }
        assert_eq!(drained, Drained::default());
    }

    #[test]
    fn only_keyboard_events_the_hook_can_see_count() {
        assert!(qualifies(0x1E, VK_A.0), "A");
        assert!(!qualifies(0xFF, VK_A.0), "keyboard overrun");
        assert!(!qualifies(0x1E, 0), "no virtual key");
        assert!(!qualifies(0x1E, 255), "VK 255, which the hook never gets");
        assert!(
            qualifies(0, VK_VOLUME_UP.0),
            "a media key without a scan code"
        );
    }

    /// A health state of its own, with the keyboard hook taken as installed.
    fn leaked_health() -> &'static HealthState {
        let health: &'static HealthState = Box::leak(Box::new(HealthState::new()));
        health.set_hook_installed(true);
        health
    }

    #[test]
    fn the_watchdog_thread_starts_and_stops_without_raw_input() {
        let health = leaked_health();
        let (recorder, _requests) = Recorder::new();
        assert!(!health.snapshot().watched);
        let watchdog = HookWatchdog::start_synthetic(health, recorder, leaked_counters())
            .expect("the watchdog thread starts");
        assert!(health.snapshot().watched, "watched while the thread runs");
        let begun = Instant::now();
        assert!(watchdog.stop(), "the thread is joined within 1 s");
        assert!(begun.elapsed() < HookWatchdog::STOP_TIMEOUT);
        assert!(!health.snapshot().watched, "not watched once it stopped");
    }

    /// Counts the windows `EnumThreadWindows` reports.
    unsafe extern "system" fn count_window(_window: HWND, count: LPARAM) -> windows::core::BOOL {
        // SAFETY: `count` is the address of the `usize` that
        // `top_level_windows_of` passes, alive for the whole enumeration.
        unsafe { *(count.0 as *mut usize) += 1 };
        windows::core::BOOL(1)
    }

    /// How many top-level windows the thread `thread_id` has, which only
    /// reads.
    fn top_level_windows_of(thread_id: u32) -> usize {
        let mut count = 0usize;
        // SAFETY: the callback only adds to `count`, a local that outlives
        // the call, through the address passed as its LPARAM.
        let _ = unsafe {
            EnumThreadWindows(
                thread_id,
                Some(count_window),
                LPARAM(&mut count as *mut usize as isize),
            )
        };
        count
    }

    #[test]
    fn the_watchdog_thread_has_no_top_level_window() {
        // Critic OQ3: a message-only window is not top-level, and its thread
        // gets no default IME window, which would be one, so the stop has
        // no IME teardown to wait for.
        let (recorder, _requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(leaked_health(), recorder, leaked_counters())
            .expect("the watchdog thread starts");
        let windows = top_level_windows_of(watchdog.thread_id);
        assert!(watchdog.stop(), "the thread is joined within 1 s");
        assert_eq!(windows, 0);
    }

    #[test]
    fn a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name() {
        // Cut L1.3 E4, a characterisation of Windows rather than of the
        // watchdog: a message already in the queue does not end an
        // MWMO_INPUTAVAILABLE wait whose mask leaves its kind out, and ends
        // one whose mask names it. The batched read of raw input rests on
        // this rule for QS_RAWINPUT; here it is shown for QS_POSTMESSAGE on
        // a thread's own queue, with no hook and no raw input.
        const PROBE: u32 = WM_APP + 0x30;
        let outcome = thread::spawn(|| {
            let mut msg = MSG::default();
            // SAFETY: `msg` is a local that outlives the call, which only
            // writes it; the call makes this thread's queue and removes
            // nothing.
            let _ = unsafe { PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE) };
            // SAFETY: GetCurrentThreadId reads no memory of ours;
            // PostThreadMessageW takes the id and the message by value, and
            // the message carries no pointer.
            unsafe { PostThreadMessageW(GetCurrentThreadId(), PROBE, WPARAM(0), LPARAM(0)) }
                .expect("a thread posts to its own queue");
            let begun = Instant::now();
            // SAFETY: no handles are passed, so the call reads no memory of
            // ours; it waits on this thread's own queue.
            let left_out = unsafe {
                MsgWaitForMultipleObjectsEx(
                    None,
                    100,
                    QS_ALLINPUT & !QS_POSTMESSAGE,
                    MWMO_INPUTAVAILABLE,
                )
            };
            let waited = begun.elapsed();
            let begun = Instant::now();
            // SAFETY: as above.
            let named =
                unsafe { MsgWaitForMultipleObjectsEx(None, 100, QS_ALLINPUT, MWMO_INPUTAVAILABLE) };
            let returned = begun.elapsed();
            let mut removed = 0;
            // SAFETY: `msg` is a local that outlives every call, which only
            // writes it and removes the probe from this thread's own queue.
            while unsafe { PeekMessageW(&mut msg, None, PROBE, PROBE, PM_REMOVE) }.as_bool() {
                removed += 1;
            }
            (left_out, waited, named, returned, removed)
        })
        .join()
        .expect("the probe thread ends");
        let (left_out, waited, named, returned, removed) = outcome;
        assert_eq!(left_out, WAIT_TIMEOUT, "a mask without the message's kind");
        assert!(
            waited >= Duration::from_millis(80),
            "the wait timed out after {waited:?}"
        );
        assert_eq!(named, WAIT_OBJECT_0, "a mask that names the message's kind");
        assert!(
            returned < Duration::from_millis(50),
            "it returned after {returned:?}"
        );
        assert_eq!(removed, 1, "the message stayed queued, once");
    }

    /// Posts a synthetic raw event every 20 ms for 3 s, calling `before`
    /// right before each, and returns the requests with when they came,
    /// from the first raw event.
    fn three_seconds_of_raw_events(
        health: &'static HealthState,
        before: impl Fn(),
    ) -> Vec<(Duration, ReinstallRequest)> {
        let (recorder, requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(health, recorder, leaked_counters())
            .expect("the watchdog thread starts");
        let first = Instant::now();
        while first.elapsed() < Duration::from_secs(3) {
            before();
            watchdog.post_synthetic_raw();
            thread::sleep(Duration::from_millis(20));
        }
        assert!(watchdog.stop(), "the thread is joined within 1 s");
        requests
            .try_iter()
            .map(|(at, request)| (at.saturating_duration_since(first), request))
            .collect()
    }

    #[test]
    fn synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request() {
        // Roadmap item 2.4 below the hook: the thread, its loop and its
        // timer on the real clock. The requester never answers, so the
        // request stays outstanding: exactly one.
        let requests = three_seconds_of_raw_events(leaked_health(), || {});
        assert_eq!(requests.len(), 1, "{requests:?}");
        let (after, request) = requests[0];
        assert!(
            (Duration::from_secs(1)..=Duration::from_secs(2)).contains(&after),
            "requested {after:?} after the first raw event"
        );
        assert_eq!(request.seen, 0);
        assert!(request.silent_for >= SILENCE, "{request:?}");
    }

    #[test]
    fn the_loop_counts_its_wakes_and_raw_key_events() {
        // Cut L1.2 step 6: the loop counts its passes, one per wake, and
        // every raw key event it reads, for the stop line that manual check
        // M9 reads.
        let mut drive = Drive::new(Recorder::new());
        drive.run(0, 3000, a_burst);
        assert_eq!(
            drive.ticker.stats(),
            LoopStats {
                wakes: drive.passes,
                raw_keys: 11
            }
        );
    }

    #[test]
    fn the_watchdog_thread_reports_its_wakes_and_raw_key_events_when_it_stops() {
        let (recorder, _requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(leaked_health(), recorder, leaked_counters())
            .expect("the watchdog thread starts");
        for _ in 0..5 {
            watchdog.post_synthetic_raw();
            thread::sleep(Duration::from_millis(20));
        }
        let stats = watchdog
            .stop_counting()
            .expect("the thread is joined within 1 s");
        assert_eq!(stats.raw_keys, 5, "{stats:?}");
        assert!(stats.wakes >= 1, "{stats:?}");
    }

    #[test]
    fn a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events() {
        // Ruling L1: while the watchdog ticks, raw key events wait in the
        // queue and are read at the ticks, so the thread wakes about four
        // times a second whatever the typing rate. The synthetic source
        // stands for raw input with posted messages (cut L1.2).
        let health = leaked_health();
        let (recorder, _requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(health, recorder, leaked_counters())
            .expect("the watchdog thread starts");
        watchdog.post_synthetic_raw();
        thread::sleep(Duration::from_millis(50));
        let begun = Instant::now();
        let mut posts = 1;
        while begun.elapsed() < Duration::from_secs(1) {
            health.callback_seen();
            watchdog.post_synthetic_raw();
            posts += 1;
            thread::sleep(Duration::from_millis(5));
        }
        let stats = watchdog
            .stop_counting()
            .expect("the thread is joined within 1 s");
        assert_eq!(stats.raw_keys, posts, "every raw key event is read");
        assert!(
            stats.wakes <= 12,
            "a ticking watchdog woke {} times for {posts} raw key events (at most 12)",
            stats.wakes
        );
    }

    #[test]
    fn synthetic_raw_events_with_a_live_heartbeat_send_no_request() {
        let health = leaked_health();
        let requests = three_seconds_of_raw_events(health, || health.callback_seen());
        assert_eq!(requests, []);
    }

    /// The non-test lines of `text`: those before its `#[cfg(test)]` and
    /// `mod tests {` pair, or every line if it has none.
    fn non_test_lines(text: &str) -> Vec<&str> {
        let lines: Vec<&str> = text.lines().collect();
        let end = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len());
        lines[..end].to_vec()
    }

    /// The non-test lines of the first function whose line contains
    /// `signature`, until its braces balance. Panics if there is none, so a
    /// renamed function fails the scan instead of passing it.
    fn fn_lines<'a>(lines: &[&'a str], signature: &str) -> Vec<&'a str> {
        let start = lines
            .iter()
            .position(|line| line.contains(signature))
            .unwrap_or_else(|| panic!("no function `{signature}`"));
        let mut depth = 0_i64;
        for (index, line) in lines.iter().enumerate().skip(start) {
            depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
            if depth == 0 && line.contains('}') {
                return lines[start..=index].to_vec();
            }
        }
        panic!("the function `{signature}` does not end");
    }

    fn source(file: &str) -> String {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(root.join(file)).expect("read a source file")
    }

    #[test]
    fn the_watchdog_injects_no_input() {
        // Roadmap item 2.4's acceptance "ten minutes of idle time produce no
        // injected input": the watchdog and the reinstall name no way to
        // inject input. The tokens are assembled, so this test does not
        // match itself.
        let tokens = [
            concat!("Send", "Input"),
            concat!("keybd", "_event"),
            concat!("mouse", "_event"),
            concat!("KEYBD", "INPUT"),
            concat!("INPUT", "_KEYBOARD"),
        ];
        let mut scanned: Vec<(String, Vec<String>)> = Vec::new();
        for file in [
            "src/hook/watchdog/mod.rs",
            "src/hook/watchdog/decision.rs",
            "src/hook/health.rs",
            "src/hook/installer.rs",
        ] {
            let text = source(file);
            let lines = non_test_lines(&text);
            assert!(lines.len() > 10, "{file} was read");
            scanned.push((
                file.to_owned(),
                lines.iter().map(|line| (*line).to_owned()).collect(),
            ));
        }
        for (file, signature) in [
            ("src/hook/keyboard_hook.rs", "fn reinstall_keyboard_hook("),
            (
                "src/hook/keyboard_hook.rs",
                "fn reinstall_keyboard_hook_with(",
            ),
            ("src/hook/main_thread.rs", "fn reinstall_if_silent("),
            ("src/hook/main_thread.rs", "fn dispatch_sent_messages("),
        ] {
            let text = source(file);
            let lines = non_test_lines(&text);
            scanned.push((
                format!("{file} {signature}"),
                fn_lines(&lines, signature)
                    .iter()
                    .map(|line| (*line).to_owned())
                    .collect(),
            ));
        }
        let mut sites = Vec::new();
        for (what, lines) in &scanned {
            for line in lines {
                if tokens.iter().any(|token| line.contains(token)) {
                    sites.push(format!("{what}: {}", line.trim()));
                }
            }
        }
        assert!(sites.is_empty(), "input injection:\n{}", sites.join("\n"));
    }

    #[test]
    fn no_key_data_leaves_the_raw_input_read() {
        // Roadmap item 2.4: no key data from raw input is kept or logged.
        // The fields that name a key are read only where an event is
        // classified, by the standard read and by the buffered one, and no
        // log line names them.
        let fields = [concat!("V", "Key"), concat!("Make", "Code")];
        let text = source("src/hook/watchdog/mod.rs");
        let lines = non_test_lines(&text);
        let mut allowed = fn_lines(&lines, "fn raw_key_event(");
        allowed.extend(fn_lines(&lines, "fn count_raw_keys("));
        allowed.extend(fn_lines(&lines, "fn qualifies("));
        let mut found = 0;
        let mut sites = Vec::new();
        for line in &lines {
            if fields.iter().any(|field| line.contains(field)) {
                found += 1;
                let logs = line.contains(concat!("log", "::"));
                if logs || !allowed.contains(line) {
                    sites.push(line.trim().to_owned());
                }
            }
        }
        assert!(found > 0, "the raw input read names the key fields");
        assert!(
            sites.is_empty(),
            "key data outside the read:\n{}",
            sites.join("\n")
        );
    }
}
