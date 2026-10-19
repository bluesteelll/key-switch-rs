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
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::{
    Foundation::{HWND, LPARAM, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM},
    System::Threading::{GetCurrentThreadId, INFINITE},
    UI::{
        Input::{
            GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD,
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
use decision::{Verdict, Watchdog};

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
        }
    }

    /// A raw key event that the keyboard hook should see. It counts only
    /// while the keyboard hook is installed (ruling I3): before `install`
    /// and after `uninstall` no callback can answer it.
    fn raw_event(&mut self, now: Instant, health: &HealthState) {
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
        self.take_answer(now);
        if self.watchdog.next_tick().is_some_and(|due| due <= now) {
            let callbacks = context.health.callbacks();
            if callbacks != self.seen {
                self.seen = callbacks;
                // The hook answers: a failed reinstall before it is no
                // longer the hook's state (ruling M3).
                context.health.hook_answered();
            }
            if let Verdict::Reinstall { request, .. } = self.watchdog.tick(now, callbacks) {
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
    /// Receives once the thread has dropped everything it owns.
    done: Receiver<()>,
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
                run_watchdog(health, &requests, counters, source, &ready_tx);
                let _ = done_tx.send(());
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
        self.stop_thread()
    }

    fn stop_thread(&mut self) -> bool {
        let Some(join) = self.join.take() else {
            return true;
        };
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; WM_QUIT carries no pointer. A
        // thread that ended already only makes the call fail, and its done
        // message waits below.
        let _ = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        match self.done.recv_timeout(Self::STOP_TIMEOUT) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                let _ = join.join();
                true
            }
            Err(RecvTimeoutError::Timeout) => false,
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

/// The watchdog thread: its prologue, its loop and its epilogue. The
/// registration is removed and the window destroyed when they drop, in
/// that order, at the end.
fn run_watchdog<R: RequestReinstall>(
    health: &'static HealthState,
    requests: &R,
    counters: &'static Counters,
    source: Source,
    ready: &SyncSender<io::Result<u32>>,
) {
    let fail = |what: &str, error: windows::core::Error| {
        let _ = ready.send(Err(io::Error::other(format!("{what}: {error}"))));
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
        return;
    }
    let context = Context {
        health,
        counters,
        requests,
        input_here: source.input_here(),
    };
    let mut ticker = Ticker::new(Instant::now(), timer, health);
    run_loop(&mut ticker, window.0, source, &context, &mut msg);
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
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::E_ACCESSDENIED;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_VOLUME_UP};

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
        // The fields that name a key are read only where the event is
        // classified, and no log line names them.
        let fields = [concat!("V", "Key"), concat!("Make", "Code")];
        let text = source("src/hook/watchdog/mod.rs");
        let lines = non_test_lines(&text);
        let mut allowed = fn_lines(&lines, "fn raw_key_event(");
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
