//! The action executor (roadmap item 2.1, KS-08): one `action-executor`
//! thread that runs the slow actions, `Launch`, `OpenUrl` and `Sequence`,
//! so that the keyboard hook never waits for them.
//!
//! The thread that fires such an action (the hook thread, or a Hold's
//! `hold-timer` thread) hands it to [`ExecutorHandle::submit`], which moves
//! it into a bounded queue without waiting and posts a wake-up to the
//! executor; a full queue drops the job and counts it
//! (`executor_jobs_dropped`). The executor initialises COM as a
//! single-threaded apartment, as `ShellExecuteW` asks, and waits for its
//! messages and its `Delay` timer at once, so it pumps its queue as an STA
//! must.
//!
//! Macros run side by side without a thread each: a `Sequence` is a run
//! (`runs`) that goes up to its next `Delay`, stores the deadline and
//! yields, and one high-resolution waitable timer
//! ([`WaitableTimer`]) wakes the loop for the nearest deadline, within about
//! a millisecond. A `Delay` holds up no other job. Only a step that blocks
//! inside Windows (a `Launch`, an `OpenUrl`, a dialog `ShellExecuteW` shows)
//! holds up the other jobs and runs while it runs.
//!
//! [`ExecutorHandle::cancel_pending`] starts a new epoch (`swch off` or
//! `on`, a reload, exit): every queued job and every run of the old epoch
//! is dropped at the executor's next wake, and no further step of such a
//! run starts; a step already running finishes first.
//!
//! The executor never touches a hook or the engine. What it runs goes
//! through a [`Runner`]: the daemon's [`Win32Runner`], or a fake one in the
//! tests, so that no test runs a real action.

mod runs;
#[cfg(test)]
pub(crate) mod testing;

use std::io;
use std::marker::PhantomData;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError};
use std::thread::{self, JoinHandle, Thread};
use std::time::{Duration, Instant};

use windows::Win32::{
    Foundation::*,
    System::{
        Com::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize},
        Threading::{GetCurrentThreadId, INFINITE},
    },
    UI::WindowsAndMessaging::*,
};

use crate::core::windows_actions::{BindAction, launch_process, shell_open};
use crate::data::sequence::{self, SequenceStep};
use crate::logging::counters::{Counter, Counters};
use crate::platform::timers::WaitableTimer;
use runs::{Calls, Ctx, Job, Next, Runs};

/// How many slow actions wait for the executor before the next one is
/// dropped: a second of a `repeat: true` macro at the auto-repeat rate.
pub const QUEUE_CAPACITY: usize = 32;
/// How long [`Executor::start`] waits for the thread to create its queue.
pub const START_TIMEOUT: Duration = Duration::from_secs(5);
/// How long [`Executor::stop`] waits for a running step to finish.
pub const STOP_TIMEOUT: Duration = Duration::from_secs(2);
/// The thread message that wakes the executor: a job was queued, or the
/// epoch changed.
const WAKE: u32 = WM_APP;

/// The window a macro's `Window(...)` steps act on: the foreground window
/// when the macro fired. An integer, because `HWND` is not `Send`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetWindow(isize);

impl TargetWindow {
    /// No window. What a `Launch` or an `OpenUrl` carries, which never
    /// reads it; a `Window(...)` step skips it.
    pub const NONE: Self = Self(0);

    /// The foreground window now, or `NONE` if there is none.
    pub fn foreground() -> Self {
        // SAFETY: GetForegroundWindow takes no arguments and reads no
        // memory of ours; a null window becomes NONE.
        let hwnd = unsafe { GetForegroundWindow() };
        Self(hwnd.0 as isize)
    }

    /// The target to queue with `action`: the foreground window for a
    /// `Sequence`, the only action with `Window(...)` steps, and `NONE`
    /// otherwise, so that a `Launch` or an `OpenUrl` adds no Win32 call to
    /// the thread that fires it.
    pub fn for_action(action: &BindAction) -> Self {
        if let BindAction::Sequence(_) = action {
            Self::foreground()
        } else {
            Self::NONE
        }
    }

    fn hwnd(self) -> HWND {
        HWND(self.0 as *mut std::ffi::c_void)
    }

    #[cfg(test)]
    pub(crate) const fn from_raw(raw: isize) -> Self {
        Self(raw)
    }
}

/// Runs what the executor hands out, on the executor thread. The executor
/// keeps the iteration, the `Delay` steps, their deadlines and the epochs
/// itself. Every call is made under `catch_unwind`, so an implementation
/// must stay usable after one of its calls panicked.
pub trait Runner: Send + 'static {
    /// Runs a job that is not a `Sequence`: a `Launch` or an `OpenUrl`.
    fn action(&mut self, action: &BindAction);
    /// Runs one step of a macro: any step but a `Delay`.
    fn step(&mut self, step: &SequenceStep, target: TargetWindow);
}

/// The daemon's runner, which runs the actions and steps for real.
/// Stateless, so a panic in one call leaves nothing behind.
pub struct Win32Runner;

impl Runner for Win32Runner {
    fn action(&mut self, action: &BindAction) {
        // Every arm is spelled out, so a new action does not compile until
        // its place is decided here too.
        match action {
            BindAction::Launch { exe, args } => launch_process(exe, args),
            BindAction::OpenUrl(url) => shell_open(url),
            // The executor walks a macro step by step and never hands one
            // to `action`.
            BindAction::Sequence(_) => {}
            BindAction::SwitchLanguage
            | BindAction::SwitchLanguageBackward
            | BindAction::ToggleCapsLock
            | BindAction::PressKey(_)
            | BindAction::PostMessage { .. }
            | BindAction::Media(_)
            | BindAction::DoNothing => {
                if let Some(fast) = action.as_fast() {
                    fast.run();
                }
            }
        }
    }

    fn step(&mut self, step: &SequenceStep, target: TargetWindow) {
        sequence::run_step(step, target.hwnd());
    }
}

/// What became of a submitted action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submitted {
    /// It waits in the queue.
    Queued,
    /// The queue was full; counted in `executor_jobs_dropped`.
    Dropped,
    /// The executor had stopped on its own; counted in
    /// `executor_unavailable`.
    Unavailable,
    /// The executor was asked to stop, or none is attached; not counted.
    Closed,
}

/// How [`Executor::stop`] ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// The thread ended and was joined.
    Joined,
    /// A step still ran after [`STOP_TIMEOUT`]; the thread was left to the
    /// end of the process.
    TimedOut,
}

/// What the handles and the executor thread share. None of the fields
/// publishes other data: a job travels through the channel.
struct Shared {
    /// Moved on by `cancel_pending` (`fetch_add`, AcqRel) and read with
    /// Acquire; work fired in an older epoch never runs.
    epoch: AtomicU64,
    /// Set once by `Executor::stop` (Release), read with Acquire.
    stopping: AtomicBool,
    /// How many macros run, stored by the executor after each poll
    /// (Release), read with Acquire.
    active_runs: AtomicUsize,
}

impl Shared {
    fn stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }

    /// Whether work fired in `epoch` may still run.
    fn live(&self, epoch: u64) -> bool {
        !self.stopping() && self.epoch.load(Ordering::Acquire) == epoch
    }
}

/// The submitting side of an executor, for every thread that fires or
/// cancels actions. No method waits.
pub struct ExecutorHandle {
    jobs: SyncSender<Job>,
    thread_id: u32,
    shared: Arc<Shared>,
    counters: &'static Counters,
}

impl ExecutorHandle {
    /// Queues `action` with the window its `Window(...)` steps act on and
    /// wakes the executor. It runs on the hook thread, so it never waits,
    /// allocates or logs: the bounded queue's slots exist already, and a
    /// full queue or a dead executor is only counted.
    pub fn submit(&self, action: BindAction, target: TargetWindow) -> Submitted {
        let epoch = self.shared.epoch.load(Ordering::Acquire);
        match self.jobs.try_send(Job {
            action,
            target,
            epoch,
        }) {
            Ok(()) => {
                self.wake();
                Submitted::Queued
            }
            Err(TrySendError::Full(_)) => {
                self.counters.bump(Counter::ExecutorJobsDropped);
                Submitted::Dropped
            }
            Err(TrySendError::Disconnected(_)) if self.shared.stopping() => Submitted::Closed,
            Err(TrySendError::Disconnected(_)) => {
                self.counters.bump(Counter::ExecutorUnavailable);
                Submitted::Unavailable
            }
        }
    }

    /// Starts a new epoch: every job queued and every macro running is
    /// dropped at the executor's next wake, which this posts, and no
    /// further step of such a macro starts. A step already running
    /// finishes first.
    pub fn cancel_pending(&self) {
        self.shared.epoch.fetch_add(1, Ordering::AcqRel);
        self.wake();
    }

    /// How many macros run, as of the executor's last poll.
    pub fn active_runs(&self) -> usize {
        self.shared.active_runs.load(Ordering::Acquire)
    }

    /// Posted after the job or the epoch it announces, so that a drain
    /// that missed them is followed by this wake. A lost or failed wake
    /// loses no job: it waits in the queue until the next one.
    fn wake(&self) {
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value, reads no memory of ours and returns without waiting; WAKE
        // carries no pointer. An id that is no longer valid only makes the
        // call fail.
        let _ = unsafe { PostThreadMessageW(self.thread_id, WAKE, WPARAM(0), LPARAM(0)) };
    }
}

/// The `action-executor` thread. Dropping it stops it, as
/// [`stop`](Self::stop) does.
pub struct Executor {
    handle: Arc<ExecutorHandle>,
    thread: Thread,
    join: Option<JoinHandle<()>>,
    /// Receives once the thread has dropped everything it owns.
    done: Receiver<()>,
}

impl Executor {
    /// Starts the thread with `runner` and a queue of `capacity` jobs (at
    /// least one), counting in `counters`. Returns once the thread has made
    /// its `Delay` timer and its message queue, so that no wake is posted
    /// before the queue exists; an error if it could not make either.
    pub fn start<R: Runner>(
        runner: R,
        capacity: usize,
        counters: &'static Counters,
    ) -> io::Result<Self> {
        let (jobs, queue) = mpsc::sync_channel(capacity.max(1));
        let shared = Arc::new(Shared {
            epoch: AtomicU64::new(0),
            stopping: AtomicBool::new(false),
            active_runs: AtomicUsize::new(0),
        });
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (done_tx, done) = mpsc::sync_channel(1);
        let thread_shared = Arc::clone(&shared);
        let join = thread::Builder::new()
            .name("action-executor".into())
            .spawn(move || {
                run_executor(runner, queue, &thread_shared, counters, &ready_tx);
                let _ = done_tx.send(());
            })?;
        let started = ready.recv_timeout(START_TIMEOUT).map_err(|_| {
            io::Error::other("the action executor thread did not create its message queue")
        })?;
        let thread_id = started?;
        Ok(Self {
            handle: Arc::new(ExecutorHandle {
                jobs,
                thread_id,
                shared,
                counters,
            }),
            thread: join.thread().clone(),
            join: Some(join),
            done,
        })
    }

    /// A handle to submit and cancel with, for any thread.
    pub fn handle(&self) -> Arc<ExecutorHandle> {
        Arc::clone(&self.handle)
    }

    /// The executor thread, so that a test can check where a job ran.
    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    /// Whether the thread has ended: after a stop, or on its own.
    pub fn is_finished(&self) -> bool {
        self.join.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Stops the executor: no queued job and no further step runs, and the
    /// thread is joined once the step running now, if any, has finished;
    /// after [`STOP_TIMEOUT`] the thread is left to the end of the process.
    pub fn stop(mut self) -> Stopped {
        self.stop_thread()
    }

    fn stop_thread(&mut self) -> Stopped {
        let Some(join) = self.join.take() else {
            return Stopped::Joined;
        };
        self.handle.shared.stopping.store(true, Ordering::Release);
        self.handle.cancel_pending();
        match self.done.recv_timeout(STOP_TIMEOUT) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                // The thread catches the panics of its loop; one of its
                // epilogue has nobody to report to.
                let _ = join.join();
                Stopped::Joined
            }
            Err(RecvTimeoutError::Timeout) => Stopped::TimedOut,
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        let _ = self.stop_thread();
    }
}

/// This thread's COM apartment, from [`enter_sta`](Self::enter_sta) until
/// dropped. Not `Send`: `CoUninitialize` must run on the thread that
/// initialised COM.
struct Apartment {
    initialized: bool,
    _thread_bound: PhantomData<*const ()>,
}

impl Apartment {
    /// Initialises COM on this thread as a single-threaded apartment, as
    /// `ShellExecuteW` asks ("Some Shell extensions require the COM
    /// single-threaded apartment"). On failure the thread runs without COM,
    /// as every action did before the executor existed.
    fn enter_sta() -> Self {
        // SAFETY: CoInitializeEx takes no pointer of ours (no reserved
        // argument) and only sets up COM for the calling thread; `Drop`
        // balances every successful call, S_FALSE included, on this same
        // thread, since an Apartment never leaves it.
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        if hr.is_err() {
            log::warn!(
                "could not initialise COM on the action executor ({hr}); OpenUrl runs without it"
            );
        }
        Self {
            initialized: hr.is_ok(),
            _thread_bound: PhantomData,
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: balances the successful CoInitializeEx of `enter_sta`
            // on the thread that made it; no COM object of ours outlives
            // this guard.
            unsafe { CoUninitialize() };
        }
    }
}

/// The timer that wakes the loop for the nearest `Delay` deadline: a
/// high-resolution waitable timer, set, cancelled and waited on by the
/// executor thread, which owns it. `Instant` decides when a run goes on;
/// the timer is only the wake-up, so an early, late or stale signal only
/// makes the loop poll and set it again.
struct DelayTimer {
    timer: WaitableTimer,
    /// The deadline the timer is set for, until it signals or is
    /// cancelled.
    armed: Option<Instant>,
}

/// What [`DelayTimer::arm`] did.
enum Arming {
    /// No run waits for a deadline, and the timer is not set.
    Idle,
    /// The timer is set for the nearest deadline.
    Waiting,
    /// The nearest deadline has passed: poll again without waiting.
    Due,
    /// The timer could not be set.
    Failed(windows::core::Error),
}

/// What to do with the timer, decided without a Win32 call.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    Cancel,
    Due,
    Keep,
    Arm { after: Duration, at: Instant },
}

/// The plan for a timer set for `armed`, when the runs next wait for
/// `next`. A timer set for the nearest deadline stands: its signal is not a
/// message, so no code that ran on this thread since (a runner call, a
/// dispatched message) can have taken it.
fn plan(armed: Option<Instant>, next: Next, now: Instant) -> Plan {
    match next {
        Next::Idle => Plan::Cancel,
        Next::At(at) if at <= now => Plan::Due,
        Next::At(at) if armed == Some(at) => Plan::Keep,
        Next::At(at) => Plan::Arm {
            after: at.duration_since(now),
            at,
        },
    }
}

impl DelayTimer {
    fn new(timer: WaitableTimer) -> Self {
        Self { timer, armed: None }
    }

    fn arm(&mut self, next: Next, now: Instant) -> Arming {
        match plan(self.armed, next, now) {
            Plan::Cancel => {
                self.cancel();
                Arming::Idle
            }
            Plan::Due => {
                self.cancel();
                Arming::Due
            }
            Plan::Keep => Arming::Waiting,
            Plan::Arm { after, at } => {
                // Setting the timer replaces the due time it had.
                self.armed = None;
                if let Err(error) = self.timer.set(after) {
                    return Arming::Failed(error);
                }
                self.armed = Some(at);
                Arming::Waiting
            }
        }
    }

    /// The timer signalled and the wait took the signal: it is set for
    /// nothing until the next `arm`.
    fn fired(&mut self) {
        self.armed = None;
    }

    fn cancel(&mut self) {
        if self.armed.take().is_some() {
            // A timer that fails to stop signals once more, and the loop
            // takes that for a wake.
            let _ = self.timer.cancel();
        }
    }

    /// The handle the loop waits on.
    fn handle(&self) -> HANDLE {
        self.timer.handle()
    }
}

/// Why the loop ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// A stop was asked for, or every handle is gone.
    Stopped,
    /// It ended on its own; it has logged why.
    Died,
}

/// The executor thread: its prologue, its loop and its epilogue.
fn run_executor<R: Runner>(
    mut runner: R,
    queue: Receiver<Job>,
    shared: &Shared,
    counters: &Counters,
    ready: &SyncSender<io::Result<u32>>,
) {
    let apartment = Apartment::enter_sta();
    let mut timer = match WaitableTimer::new() {
        Ok(timer) => DelayTimer::new(timer),
        Err(error) => {
            let _ = ready.send(Err(io::Error::other(format!(
                "could not create the Delay timer: {error}"
            ))));
            return;
        }
    };
    let mut msg = MSG::default();
    // SAFETY: `msg` is a local that outlives the call, which only writes
    // it; the call creates this thread's message queue, as
    // PostThreadMessageW asks of the receiving thread, and removes nothing.
    let _ = unsafe { PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE) };
    // SAFETY: GetCurrentThreadId takes no arguments, touches no memory of
    // ours and cannot fail.
    let thread_id = unsafe { GetCurrentThreadId() };
    // A starter that gave up has dropped the queue's sender, so the loop
    // ends at its first drain.
    let _ = ready.send(Ok(thread_id));

    // The queue, the timer and the runs are owned out here, outside the
    // closure that can unwind, so that this epilogue also runs after a
    // panic of the loop itself.
    let mut runs = Runs::new();
    let end = panic::catch_unwind(AssertUnwindSafe(|| {
        run_loop(&mut runner, &queue, shared, &mut runs, &mut timer)
    }))
    .unwrap_or_else(|_| {
        log::error!(
            "the action executor panicked and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts"
        );
        End::Died
    });

    // Closing the timer's only handle destroys it: it signals nothing more.
    drop(timer);
    shared.active_runs.store(0, Ordering::Release);
    drop(runs);
    if end == End::Died {
        // The jobs still queued would vanish with the queue: they count as
        // every later submit does.
        while queue.try_recv().is_ok() {
            counters.bump(Counter::ExecutorUnavailable);
        }
    }
    drop(queue);
    drop(runner);
    drop(apartment);
}

fn report_panics(calls: Calls) {
    for _ in 0..calls.panics {
        log::error!("an action panicked on the action executor; it keeps running");
    }
}

/// The loop: drain the jobs and advance the due runs until a whole pass
/// made no runner call, then set the timer and wait for it and for the
/// thread's messages at once.
///
/// A runner call can pump this thread's queue (a `ShellExecuteW` dialog, a
/// DDE or COM wait) and eat the `WAKE` of a job queued meanwhile, so the
/// last thing before the wait is a drain that found the queue empty, with
/// no runner call after it. Such a pump cannot take the timer's signal,
/// which is not a message.
fn run_loop(
    runner: &mut dyn Runner,
    queue: &Receiver<Job>,
    shared: &Shared,
    runs: &mut Runs,
    timer: &mut DelayTimer,
) -> End {
    let live = |epoch: u64| shared.live(epoch);
    let now = Instant::now;
    let mut ctx = Ctx {
        runner,
        live: &live,
        now: &now,
    };
    let mut msg = MSG::default();
    loop {
        let next = loop {
            let mut calls = Calls::default();
            // Jobs, oldest first, each followed by a poll, so that a due
            // run is late by one job at most.
            loop {
                if shared.stopping() {
                    return End::Stopped;
                }
                match queue.try_recv() {
                    Ok(job) => {
                        calls += runs.start(job, &mut ctx);
                        calls += runs.poll(&mut ctx).calls;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return End::Stopped,
                }
            }

            let polled = runs.poll(&mut ctx);
            calls += polled.calls;
            report_panics(calls);
            shared.active_runs.store(runs.len(), Ordering::Release);
            if shared.stopping() {
                return End::Stopped;
            }
            if calls.made == 0 {
                break polled.next;
            }
        };

        match timer.arm(next, Instant::now()) {
            Arming::Due => continue,
            Arming::Failed(error) => {
                let stopped = runs.drop_timed();
                shared.active_runs.store(runs.len(), Ordering::Release);
                log::error!(
                    "could not start the Delay timer ({error}); {stopped} macros waiting in a Delay stop"
                );
            }
            Arming::Idle | Arming::Waiting => {}
        }

        // SAFETY: the slice holds the Delay timer's handle, which `timer`
        // keeps open across the call, and lives until the call returns; the
        // call only reads it.
        let woke = unsafe {
            MsgWaitForMultipleObjectsEx(
                Some(&[timer.handle()]),
                INFINITE,
                QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            )
        };
        if woke == WAIT_OBJECT_0 {
            timer.fired();
            continue;
        }
        if woke == WAIT_FAILED {
            log::error!(
                "the action executor's message loop failed ({}) and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts",
                windows::core::Error::from_thread()
            );
            return End::Died;
        }
        // A message waits. MWMO_INPUTAVAILABLE also counts one queued before
        // the wait began, such as the messages behind a WAKE that `pump`
        // left for after the drain.
        if let Some(end) = pump(&mut msg) {
            return end;
        }
    }
}

/// Takes this thread's messages until its queue is empty or a `WAKE` comes,
/// which goes back to the drain at once. `Some(End::Died)` on `WM_QUIT`.
fn pump(msg: &mut MSG) -> Option<End> {
    // SAFETY: `msg` is the loop's local and outlives every call, which only
    // writes it and removes the message from this thread's own queue.
    while unsafe { PeekMessageW(msg, None, 0, 0, PM_REMOVE) }.as_bool() {
        if msg.message == WM_QUIT {
            // Nothing of ours posts WM_QUIT; foreign code on this thread
            // asked it to end its loop.
            log::error!(
                "the action executor received WM_QUIT and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts"
            );
            return Some(End::Died);
        }
        if msg.hwnd.is_invalid() && msg.message == WAKE {
            return None;
        }
        // COM's hidden windows, or a TimerProc's WM_TIMER.
        // SAFETY: `msg` was filled by PeekMessageW above and outlives both
        // calls, which only read it.
        unsafe {
            let _ = TranslateMessage(msg);
            DispatchMessageW(msg);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::System::Com::{
        APTTYPE, APTTYPE_MAINSTA, APTTYPE_STA, APTTYPEQUALIFIER, CoGetApartmentType,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::runs::{Ctx, Job, Next, Runs};
    use super::testing::{
        FakeRunner, Ran, Record, WAIT, leaked_counters, next_record as next, wait_until,
    };
    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::sequence::SequenceStep;
    use crate::logging::counters::{Counter, Counters};
    use crate::platform::timers::{LONGEST, due_time};

    fn sequence(steps: Vec<SequenceStep>) -> BindAction {
        BindAction::Sequence(Arc::new(steps))
    }

    fn key(vk: VIRTUAL_KEY) -> SequenceStep {
        SequenceStep::PressKey(vk)
    }

    fn delay(ms: u64) -> SequenceStep {
        SequenceStep::Delay(ms)
    }

    fn open(url: &str) -> BindAction {
        BindAction::OpenUrl(url.to_owned())
    }

    /// A real executor thread with `runner` and counters of its own.
    fn start(runner: FakeRunner, capacity: usize) -> (Executor, &'static Counters) {
        let counters = leaked_counters();
        let executor = Executor::start(runner, capacity, counters).expect("the executor starts");
        (executor, counters)
    }

    /// The counts since the last call, each with its counter.
    fn take_counts(counters: &Counters) -> Vec<(Counter, u64)> {
        Counter::ALL
            .into_iter()
            .zip(counters.take().counts)
            .collect()
    }

    fn count_of(counts: &[(Counter, u64)], counter: Counter) -> u64 {
        counts
            .iter()
            .find(|(each, _)| *each == counter)
            .map_or(0, |(_, count)| *count)
    }

    #[test]
    fn a_hundred_sequence_runs_use_the_one_executor_thread() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();

        for _ in 0..100 {
            assert_eq!(
                handle.submit(sequence(vec![key(VK_F13)]), TargetWindow::NONE),
                Submitted::Queued
            );
            let record = next(&records);
            assert_eq!(record.text(), "key F13");
            assert_eq!(record.thread, executor.thread().id());
            assert_ne!(record.thread, thread::current().id());
            assert_eq!(record.name.as_deref(), Some("action-executor"));
        }
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn delay_10000_stops_within_100_ms_of_an_epoch_change() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![delay(10_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        let t0 = Instant::now();
        handle.cancel_pending();
        wait_until("the cancelled macro is gone", || handle.active_runs() == 0);
        let stopped = t0.elapsed();
        assert!(
            stopped < Duration::from_millis(100),
            "the Delay stopped {stopped:?} after the epoch change"
        );

        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15", "F14 never runs");
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err());
    }

    #[test]
    fn a_delay_5000_macro_holds_up_no_other_job() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(5_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        let t0 = Instant::now();
        handle.submit(open("https://example.com"), TargetWindow::NONE);
        let opened = next(&records);
        assert_eq!(opened.text(), "open https://example.com");
        let waited = opened.at.duration_since(t0);
        assert!(
            waited < Duration::from_millis(100),
            "the OpenUrl waited {waited:?} behind the Delay"
        );

        handle.cancel_pending();
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "F14 never runs");
    }

    #[test]
    fn two_macros_fired_together_both_run_in_their_own_order() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(300), key(VK_F14)]),
            TargetWindow::NONE,
        );
        handle.submit(
            sequence(vec![key(VK_F15), delay(300), key(VK_F16)]),
            TargetWindow::NONE,
        );

        let texts: Vec<String> = (0..4).map(|_| next(&records).text().to_owned()).collect();
        assert_eq!(texts, ["key F13", "key F15", "key F14", "key F16"]);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_cancelled_waiting_run_never_resumes() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(150), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        handle.cancel_pending();
        thread::sleep(Duration::from_millis(400));
        assert!(records.try_recv().is_err(), "F14 must not run");

        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_delay_resumes_its_run_after_its_deadline() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        executor.handle().submit(
            sequence(vec![key(VK_F13), delay(50), key(VK_F14)]),
            TargetWindow::NONE,
        );

        let first = next(&records);
        let second = next(&records);
        assert_eq!(second.text(), "key F14");
        let between = second.at.duration_since(first.at);
        assert!(between >= Duration::from_millis(50), "{between:?}");
        assert!(between < Duration::from_secs(1), "{between:?}");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    /// How late each `Delay(ms)` of a `[Key(F13), Delay(ms)] x count` macro
    /// ended on a real executor thread: the time between two keys less
    /// `ms`. Fails if a `Delay` ended early.
    fn delay_lateness(ms: u64, count: usize) -> Vec<Duration> {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let steps = (0..count)
            .flat_map(|_| [key(VK_F13), delay(ms)])
            .chain([key(VK_F13)])
            .collect();
        executor
            .handle()
            .submit(sequence(steps), TargetWindow::NONE);
        let keys: Vec<Instant> = (0..=count).map(|_| next(&records).at).collect();
        assert_eq!(executor.stop(), Stopped::Joined);

        let wanted = Duration::from_millis(ms);
        keys.windows(2)
            .map(|pair| {
                let gap = pair[1].duration_since(pair[0]);
                assert!(gap >= wanted, "a Delay({ms}) ended after {gap:?}");
                gap - wanted
            })
            .collect()
    }

    #[test]
    fn a_delay_ends_within_a_few_ms_of_its_deadline() {
        // A Delay used to be a high-resolution sleep, about 1 ms late. A
        // wake on the system clock tick (about 15.6 ms) is about 10 ms late
        // on average, and a thread timer is never shorter than 10 ms, so a
        // Delay(5) on one is at least 5 ms late. 5 ms is a third of a tick
        // and well above the lateness of a high-resolution timer here
        // (under 1 ms); the median ignores the few gaps in which a loaded
        // machine did not run the executor thread at once.
        let limit = Duration::from_millis(5);
        for (ms, count) in [(5, 20), (20, 20), (50, 10)] {
            let mut late = delay_lateness(ms, count);
            late.sort();
            let median = late[late.len() / 2];
            assert!(
                median < limit,
                "Delay({ms}) ended {median:?} late in the median; sorted: {late:?}"
            );
        }
    }

    #[test]
    fn a_delay_of_u64_max_waits_until_cancelled() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![delay(u64::MAX), key(VK_F14)]),
            TargetWindow::NONE,
        );
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        thread::sleep(Duration::from_millis(50));
        assert!(records.try_recv().is_err(), "nothing runs");
        assert!(!executor.is_finished(), "the executor still runs");
        assert_eq!(handle.active_runs(), 1);

        handle.cancel_pending();
        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms() {
        let (tx, records) = mpsc::channel();
        let mut runner = FakeRunner::new(tx);
        let base = Instant::now();
        let mut runs = Runs::new();
        let live = |_: u64| true;
        let now = || base;
        let mut ctx = Ctx {
            runner: &mut runner,
            live: &live,
            now: &now,
        };
        runs.start(
            Job {
                action: sequence(vec![delay(u64::MAX), key(VK_F14)]),
                target: TargetWindow::NONE,
                epoch: 0,
            },
            &mut ctx,
        );
        let polled = runs.poll(&mut ctx);

        match polled.next {
            Next::Idle => {}
            Next::At(_) => {
                let Plan::Arm { after, .. } = plan(None, polled.next, base) else {
                    panic!("a deadline in the future sets the timer");
                };
                assert_eq!(
                    due_time(after),
                    due_time(LONGEST),
                    "the timer is set for at most LONGEST"
                );
                assert!(
                    LONGEST < Duration::from_millis(u64::from(u32::MAX)),
                    "no timer longer than u32::MAX - 1 ms"
                );
            }
        }

        let a_day_later = base + Duration::from_secs(86_400);
        let later = || a_day_later;
        let mut ctx = Ctx {
            runner: &mut runner,
            live: &live,
            now: &later,
        };
        let polled = runs.poll(&mut ctx);
        assert_eq!(polled.calls.made, 0);
        assert!(records.try_recv().is_err(), "nothing runs a day later");
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn the_delay_timer_is_set_again_only_for_another_deadline() {
        // The timer's signal is not a message: code that ran on the executor
        // thread since it was set (a runner call, a dispatched message)
        // cannot have taken it, so a timer set for the nearest deadline
        // stands.
        let now = Instant::now();
        let at = now + Duration::from_millis(100);
        let sooner = now + Duration::from_millis(50);

        assert_eq!(plan(None, Next::Idle, now), Plan::Cancel);
        assert_eq!(plan(Some(at), Next::Idle, now), Plan::Cancel);
        assert_eq!(plan(Some(at), Next::At(now), now), Plan::Due);
        assert_eq!(
            plan(None, Next::At(at), now),
            Plan::Arm {
                after: Duration::from_millis(100),
                at
            }
        );
        assert_eq!(
            plan(Some(at), Next::At(at), now),
            Plan::Keep,
            "set for this deadline: the timer stands"
        );
        assert_eq!(
            plan(Some(at), Next::At(sooner), now),
            Plan::Arm {
                after: Duration::from_millis(50),
                at: sooner
            }
        );
    }

    /// The enqueue times of 20 `OpenUrl` jobs on a fresh executor whose
    /// runner blocks 500 ms per call. The executor is stopped before this
    /// returns, after at most one call.
    fn twenty_enqueue_times() -> Vec<Duration> {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_block(Duration::from_millis(500)),
            64,
        );
        let handle = executor.handle();
        let times = (0..20)
            .map(|_| {
                let begun = Instant::now();
                let submitted = handle.submit(open("https://example.com"), TargetWindow::NONE);
                let took = begun.elapsed();
                assert_eq!(submitted, Submitted::Queued);
                took
            })
            .collect();
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_iter().count() <= 1, "the stop drops the queue");
        times
    }

    #[test]
    fn twenty_slow_jobs_each_enqueue_in_under_5_ms() {
        // The rule of the lane: every one of the 20 under 5 ms, and one
        // retry of the whole batch for a scheduler hiccup.
        let limit = Duration::from_millis(5);
        let first = twenty_enqueue_times();
        if first.iter().all(|took| *took < limit) {
            return;
        }
        let second = twenty_enqueue_times();
        assert!(
            second.iter().all(|took| *took < limit),
            "an enqueue took 5 ms or more twice: {first:?}, then {second:?}"
        );
    }

    #[test]
    fn a_job_queued_while_a_step_pumps_messages_runs_without_another_wake() {
        // A step resumed after a Delay pumps the thread's
        // queue (a ShellExecuteW dialog, a DDE wait) and eats the WAKE of a
        // job queued meanwhile; the job must run once the step returns,
        // with no further wake.
        let (tx, records) = mpsc::channel();
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let (executor, _) = start(
            FakeRunner::new(tx)
                .with_gate(started_tx, release_rx)
                .with_eat_wakes(),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();
        handle.submit(sequence(vec![delay(30), key(VK_F13)]), TargetWindow::NONE);
        started
            .recv_timeout(WAIT)
            .expect("the step after the Delay runs");
        assert_eq!(next(&records).text(), "key F13");

        assert_eq!(
            handle.submit(open("B"), TargetWindow::NONE),
            Submitted::Queued
        );
        drop(release);

        let ran = records.recv_timeout(Duration::from_secs(1));
        assert_eq!(
            ran.as_ref().map(Record::text).ok(),
            Some("open B"),
            "the job queued while a step pumped messages did not run within 1 s"
        );
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_full_queue_drops_the_job_and_counts_it() {
        let (tx, records) = mpsc::channel();
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let (executor, counters) = start(FakeRunner::new(tx).with_gate(started_tx, release_rx), 1);
        let handle = executor.handle();

        assert_eq!(
            handle.submit(open("A"), TargetWindow::NONE),
            Submitted::Queued
        );
        started.recv_timeout(WAIT).expect("A runs");
        assert_eq!(
            handle.submit(open("B"), TargetWindow::NONE),
            Submitted::Queued
        );
        assert_eq!(
            handle.submit(open("C"), TargetWindow::NONE),
            Submitted::Dropped
        );
        let counts = take_counts(counters);
        assert_eq!(
            count_of(&counts, Counter::ExecutorJobsDropped),
            1,
            "{counts:?}"
        );

        drop(release);
        assert_eq!(next(&records).text(), "open A");
        assert_eq!(next(&records).text(), "open B");
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "C never runs");
    }

    #[test]
    fn a_job_queued_before_an_epoch_change_does_not_run() {
        let (tx, records) = mpsc::channel();
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let (executor, _) = start(
            FakeRunner::new(tx).with_gate(started_tx, release_rx),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("A"), TargetWindow::NONE);
        started.recv_timeout(WAIT).expect("A runs");
        handle.submit(open("B"), TargetWindow::NONE);
        handle.cancel_pending();
        drop(release);
        handle.submit(open("C"), TargetWindow::NONE);

        assert_eq!(next(&records).text(), "open A");
        assert_eq!(next(&records).text(), "open C", "B never runs");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn jobs_run_in_the_order_they_were_queued_with_their_target() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        let target = TargetWindow::from_raw(0x1234);

        handle.submit(
            BindAction::Launch {
                exe: "no-such.exe".to_owned(),
                args: Vec::new(),
            },
            TargetWindow::NONE,
        );
        handle.submit(open("https://example.com"), TargetWindow::NONE);
        handle.submit(sequence(vec![key(VK_F13), key(VK_F14)]), target);

        let ran: Vec<Ran> = (0..4).map(|_| next(&records).ran).collect();
        assert_eq!(
            ran,
            [
                Ran::Action("launch no-such.exe".to_owned()),
                Ran::Action("open https://example.com".to_owned()),
                Ran::Step("key F13".to_owned(), target),
                Ran::Step("key F14".to_owned(), target),
            ]
        );
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn stop_interrupts_a_delay_drops_the_queue_and_joins() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_block(Duration::from_millis(100)),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(10_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);
        handle.submit(open("A"), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "open A");
        for url in ["B", "C", "D"] {
            handle.submit(open(url), TargetWindow::NONE);
        }

        let begun = Instant::now();
        assert_eq!(executor.stop(), Stopped::Joined);
        let took = begun.elapsed();
        assert!(took < Duration::from_secs(1), "the stop took {took:?}");
        assert!(records.try_recv().is_err(), "nothing after A runs");
    }

    #[test]
    fn a_submit_after_stop_is_closed_and_not_counted() {
        let (tx, _records) = mpsc::channel();
        let (executor, counters) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();

        assert_eq!(executor.stop(), Stopped::Joined);

        assert_eq!(
            handle.submit(open("A"), TargetWindow::NONE),
            Submitted::Closed
        );
        let counts = take_counts(counters);
        assert!(counts.iter().all(|(_, count)| *count == 0), "{counts:?}");
    }

    #[test]
    fn an_executor_ended_by_wm_quit_counts_every_later_submit() {
        let (tx, records) = mpsc::channel();
        let (executor, counters) = start(
            FakeRunner::new(tx).with_quit_on("open quit"),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("quit"), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "open quit");
        wait_until("the executor ends on WM_QUIT", || executor.is_finished());

        assert_eq!(
            handle.submit(open("later"), TargetWindow::NONE),
            Submitted::Unavailable
        );
        let counts = take_counts(counters);
        assert_eq!(
            count_of(&counts, Counter::ExecutorUnavailable),
            1,
            "{counts:?}"
        );
        assert_eq!(
            count_of(&counts, Counter::ExecutorJobsDropped),
            0,
            "{counts:?}"
        );

        assert_eq!(executor.stop(), Stopped::Joined);
        assert_eq!(
            handle.submit(open("after stop"), TargetWindow::NONE),
            Submitted::Closed
        );
        let counts = take_counts(counters);
        assert!(counts.iter().all(|(_, count)| *count == 0), "{counts:?}");
        assert!(records.try_recv().is_err());
    }

    /// Jobs whose wake was lost (the case the death epilogue exists for):
    /// on an executor that ran one job, `lost` are queued without the wake
    /// that `submit` posts, and then WM_QUIT ends it. Returns the counts it
    /// left, or `None` if one of `lost` ran.
    fn die_with_lost_jobs(lost: &[&str]) -> Option<Vec<(Counter, u64)>> {
        use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

        let (tx, records) = mpsc::channel();
        let (executor, counters) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();

        handle.submit(open("first"), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "open first");
        // The executor goes back to its wait within microseconds of the
        // call; this pause makes sure that it has, so that the jobs below
        // are not drained and run by a pass that is still going, unless the
        // machine did not run the thread for all of it.
        thread::sleep(Duration::from_millis(200));

        let epoch = handle.shared.epoch.load(Ordering::Acquire);
        for name in lost {
            handle
                .jobs
                .try_send(Job {
                    action: open(name),
                    target: TargetWindow::NONE,
                    epoch,
                })
                .expect("the queue has room");
        }
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; WM_QUIT carries no pointer, and
        // the thread is this test's own executor.
        unsafe { PostThreadMessageW(handle.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) }
            .expect("the executor thread has a message queue");
        wait_until("the executor ends on WM_QUIT", || executor.is_finished());

        let counts = take_counts(counters);
        let ran = records.try_recv().is_ok();
        assert_eq!(executor.stop(), Stopped::Joined);
        (!ran).then_some(counts)
    }

    /// The counts left by `lost`, with one retry if one of them ran: a
    /// pass still going after the first job runs it, a defect of the death
    /// path too, and only the defect does so twice.
    fn counts_left_by_lost_jobs(lost: &[&str]) -> Vec<(Counter, u64)> {
        die_with_lost_jobs(lost)
            .or_else(|| die_with_lost_jobs(lost))
            .expect("a dead executor runs nothing more: a lost job ran twice")
    }

    #[test]
    fn a_job_still_queued_when_the_executor_dies_is_counted() {
        let counts = counts_left_by_lost_jobs(&["lost"]);
        assert_eq!(
            count_of(&counts, Counter::ExecutorUnavailable),
            1,
            "the job left in the queue is counted when the executor dies: {counts:?}"
        );
    }

    #[test]
    fn every_job_still_queued_when_the_executor_dies_is_counted() {
        let counts = counts_left_by_lost_jobs(&["lost one", "lost two", "lost three"]);
        assert_eq!(
            count_of(&counts, Counter::ExecutorUnavailable),
            3,
            "each of the three jobs left in the queue is counted once: {counts:?}"
        );
    }

    #[test]
    fn launch_and_open_url_carry_no_window_and_ask_for_none() {
        // Only a Sequence reads the foreground window; for the others the
        // hook thread makes no Win32 call and queues NONE. Where no window
        // has the foreground at all, NONE and the foreground are the same
        // and this checks nothing, so it runs where a desktop exists.
        let launch = BindAction::Launch {
            exe: "tool.exe".to_owned(),
            args: Vec::new(),
        };
        assert_eq!(
            TargetWindow::for_action(&launch),
            TargetWindow::NONE,
            "a Launch never reads its window"
        );
        assert_eq!(
            TargetWindow::for_action(&open("https://example.com")),
            TargetWindow::NONE,
            "an OpenUrl never reads its window"
        );
    }

    #[test]
    fn a_panicking_job_leaves_the_executor_running() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_panic_on("open boom"),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("boom"), TargetWindow::NONE);
        handle.submit(open("next"), TargetWindow::NONE);

        assert_eq!(next(&records).text(), "open next");
        assert!(!executor.is_finished());
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    /// The apartment of the calling thread, or `None` without COM.
    fn apartment_type() -> Option<APTTYPE> {
        let mut kind = APTTYPE::default();
        let mut qualifier = APTTYPEQUALIFIER::default();
        // SAFETY: both pointers are to locals of this frame that outlive
        // the call, which only writes them; it changes no COM state.
        unsafe { CoGetApartmentType(&mut kind, &mut qualifier) }
            .ok()
            .map(|()| kind)
    }

    /// Whether `kind` is a single-threaded apartment. The first thread to
    /// enter an STA while no other STA thread is alive becomes the process's
    /// main STA and reads as `APTTYPE_MAINSTA`, so which of the two a thread
    /// gets depends on the other tests running at the same time.
    fn is_sta(kind: Option<APTTYPE>) -> bool {
        kind == Some(APTTYPE_STA) || kind == Some(APTTYPE_MAINSTA)
    }

    /// Reports the apartment type inside every `action`.
    struct ApartmentProbe(mpsc::Sender<Option<APTTYPE>>);

    impl Runner for ApartmentProbe {
        fn action(&mut self, _action: &BindAction) {
            let _ = self.0.send(apartment_type());
        }

        fn step(&mut self, _step: &SequenceStep, _target: TargetWindow) {}
    }

    #[test]
    fn the_executor_runs_jobs_in_a_single_threaded_apartment() {
        let (tx, kinds) = mpsc::channel();
        let executor = Executor::start(ApartmentProbe(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");

        executor
            .handle()
            .submit(open("https://example.com"), TargetWindow::NONE);

        let kind = kinds.recv_timeout(WAIT).expect("the probe runs");
        assert!(is_sta(kind), "the job ran in {kind:?}, not an STA");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn the_apartment_guard_uninitialises_com_on_drop() {
        let (inside, after) = thread::spawn(|| {
            let apartment = Apartment::enter_sta();
            let inside = apartment_type();
            drop(apartment);
            (inside, apartment_type())
        })
        .join()
        .expect("the scratch thread does not panic");

        assert!(is_sta(inside), "the guard entered {inside:?}, not an STA");
        assert!(
            !is_sta(after),
            "after the guard the thread has no apartment of its own (or the implicit MTA), not {after:?}"
        );
    }

    /// Roadmap item 2.1: a macro's `Delay` is a deadline that the loop's
    /// timer resumes, so neither the executor nor the steps it runs sleep
    /// or block in a wait of their own. The one wait is the loop's, for its
    /// messages and its Delay timer at once, so that a message never waits
    /// for the timer nor the timer for a message. The non-test lines of the
    /// three files count; the names are assembled, so this test does not
    /// match itself.
    #[test]
    fn macros_never_sleep_or_block_in_a_wait() {
        let the_wait = concat!("MsgWaitFor", "MultipleObjectsEx(");
        let banned = [
            concat!("thread::", "sleep"),
            concat!("MsgWaitFor", "MultipleObjects"),
            concat!("WaitFor", "SingleObject"),
            concat!("GetMess", "ageW"),
        ];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut sites = Vec::new();
        for file in [
            "src/core/executor/mod.rs",
            "src/core/executor/runs.rs",
            "src/data/sequence.rs",
        ] {
            let text = std::fs::read_to_string(root.join(file)).expect("read a source file");
            let lines: Vec<&str> = text.lines().collect();
            let tests_at = lines
                .windows(2)
                .position(|pair| {
                    pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {"
                })
                .unwrap_or_else(|| panic!("{file} has a tests module"));
            for (index, line) in lines[..tests_at].iter().enumerate() {
                if banned.iter().any(|name| line.contains(name)) {
                    sites.push(format!("{file}:{}: {}", index + 1, line.trim()));
                }
            }
        }
        let (waits, others): (Vec<String>, Vec<String>) = sites.into_iter().partition(|site| {
            site.starts_with("src/core/executor/mod.rs:") && site.contains(the_wait)
        });
        assert!(
            others.is_empty(),
            "sleeps or blocking waits:\n{}",
            others.join("\n")
        );
        assert_eq!(
            waits.len(),
            1,
            "the executor's loop waits in exactly one place: {waits:?}"
        );
    }

    #[test]
    #[ignore = "manual: opens https://example.com in the default browser through Win32Runner on a real executor thread; check that the page opens and the test passes"]
    fn the_win32_runner_opens_a_url() {
        let executor = Executor::start(Win32Runner, QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        assert_eq!(
            executor
                .handle()
                .submit(open("https://example.com"), TargetWindow::NONE),
            Submitted::Queued
        );
        thread::sleep(Duration::from_secs(3));
        assert_eq!(executor.stop(), Stopped::Joined);
    }
}
