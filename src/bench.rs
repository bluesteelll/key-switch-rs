//! The latency bench of roadmap item 2.2: the cost of the key path, how late
//! a gesture deadline fires, and how fast the message loop answers a sent
//! message, which is how the low-level keyboard hook reaches it. Every later
//! lane runs it on its parent and on its HEAD and compares the two.
//!
//! It is one ignored test, so the default `cargo test` neither runs it nor
//! prints anything. Run it in a release build, alone:
//!
//! ```text
//! KS_BENCH_OUT=<file> KS_BENCH_LABEL=<label> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1
//! ```
//!
//! It appends one Markdown table to `KS_BENCH_OUT`, headed by
//! `KS_BENCH_LABEL`. No test installs a hook here or injects input: the
//! adapter rows call the extern callback the way the dispatcher would, with
//! no hook installed, on bindings whose action is `DoNothing` or a macro that
//! a recording runner takes; the loop rows send `WM_NULL` to a message-only
//! window of this process.
//!
//! Rows (n samples each):
//!
//! - R1: `Engine::on_key` per event of a typing script (letters, Shift,
//!   Space; nothing bound) on the template's bindings with a CapsLock Tap
//!   and Hold, an LShift DoubleTap, a chord and a sequence.
//! - R2: `Engine::on_key` per event of CapsLock Tap(200) + Hold(200) presses
//!   (down, 80 ms, up) and F13 Hold presses.
//! - R3: `Engine::on_timer` for armed Holds.
//! - R4: the adapter's key path, `keyboard_hook_callback` on the
//!   process-wide hook, F13 bound to `DoNothing` and A unbound, down and up,
//!   with the bench thread as the main thread (`hook::main_thread`).
//! - R5: the key-down callback of an F13 Hold(50) press, which sets the main
//!   thread's gesture timer.
//! - R6: how late an F13 Hold(d) fires its macro, for d of 5, 20, 50 and 200
//!   ms: the recording runner's time less the key-down's less d. The bench
//!   thread runs the main loop until a helper ends it at the record.
//! - R7: the round trip of `SendMessageTimeoutW(WM_NULL)` to a window of a
//!   thread that runs the daemon's main loop (`MainThread::run`).
//! - R8, only with `KS_BENCH_LOAD=1`: with a busy thread at normal priority
//!   on every core, the one-way delivery of a sent message to the main
//!   loop's window procedure (the time it enters it less the time before the
//!   send, as a low-level hook call reaches the daemon), with the loop thread
//!   at `THREAD_PRIORITY_NORMAL` and then at `THREAD_PRIORITY_HIGHEST`, and
//!   the round trips of the same sends. It loads every core for a few
//!   seconds, as a build does, which is why it is opt-in.
//! - R9: the key-down callback of F13 bound to `DoNothing` with
//!   `when: AppEquals("no-such.exe")`, which the bench machine's real
//!   foreground window does not satisfy (read only): the cost of a `when:`
//!   on the executable name. Each key-up is called, not timed.
//! - R10: the same with `when: TitleContains("no-such-title")`: the cost
//!   of a `when:` on the title.
//! - R11: the key-up callback of a swallowed F13 press bound to
//!   `DoNothing`; each key-down is called, not timed.
//! - R12: the key-down and key-up callbacks of the unbound A: an ordinary
//!   key.
//! - R13: R7 with the daemon's control window, and so the thread's default
//!   IME window, created on the loop thread before it runs (roadmap item
//!   2.3). It may write a session-notification warning to the log, which
//!   the bench does not show.
//! - R14: steady typing with the hook watchdog ticking (roadmap item 2.4,
//!   ruling L1): the callbacks of R12, one every millisecond, each right
//!   after one synthetic raw key event that the watchdog's thread, started
//!   on the process-wide hook's heartbeat, leaves queued and reads at its
//!   ticks, four times a second, on another core. The callback's own extra
//!   work is the heartbeat's one atomic increment, which R12 pays too.
//! - R14c: R14's control (ruling V3-I2): the same pacing and the same
//!   posted message right before each call, to a thread that never reads
//!   its queue, and no watchdog. R14 less R14c is what the ticking watchdog
//!   costs a callback; R14 less R12 adds the pacing and the post.
//! - R15: what the rejected design would have added per key event on the
//!   main thread (cut D1): `PostMessageW`, `PeekMessageW` and
//!   `DispatchMessageW` of one message to a window of the bench thread
//!   whose procedure calls `GetRawInputData` (on a null handle, which fails
//!   at once) and `DefWindowProcW`. An in-thread lower bound: no wake-up and
//!   no copy of real raw input.
//! - R16: the key-down callback of the first key of a burst, which wakes
//!   the idle watchdog: a raw key event right before the call, so that the
//!   thread's one wake and first pass run next to it, on another core. A
//!   watchdog per sample, started and stopped outside the timing; it takes
//!   about a quarter of a second per sample. A declared row, measured and
//!   reported, not gated (ruling L2): it times the worst order of cut Q6,
//!   a key's raw input before its hook call, and the wake happens once per
//!   burst of keys.

use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::{
    Foundation::*,
    System::{
        LibraryLoader::GetModuleHandleW,
        Threading::{
            GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY,
            THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL,
        },
    },
    UI::{
        Input::{
            GetRawInputData, HRAWINPUT, KeyboardAndMouse::*, RAWINPUT, RAWINPUTHEADER, RID_INPUT,
        },
        WindowsAndMessaging::*,
    },
};
use windows::core::w;

use crate::config::from_ron_str;
use crate::core::executor::testing::{FakeRunner, leaked_counters};
use crate::core::executor::{Executor, QUEUE_CAPACITY};
use crate::core::windows_actions::BindAction;
use crate::data::binding::Binding;
use crate::data::condition::Condition;
use crate::data::key_combination::KeyCombination;
use crate::data::sequence::SequenceStep;
use crate::data::trigger::Trigger;
use crate::data::vk_name::parse_vk;
use crate::engine::{Engine, Env, KeyDirection, KeyEvent, sort_by_specificity};
use crate::hook::control_window::{CONTROL_CLASS, ControlWindow};
use crate::hook::health::{ReinstallOutcome, ReinstallRequest};
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::keyboard_hook_callback::{get_hook, keyboard_hook_callback};
use crate::hook::main_thread::{MainThread, MainThreadError};
use crate::hook::watchdog::{HookWatchdog, RequestReinstall, SYNTHETIC_RAW};

/// One row of the table: its label, what it measures and its samples.
struct Row {
    id: &'static str,
    what: &'static str,
    samples: Vec<Duration>,
}

/// n, mean, p50, p99 and max of `samples`, in microseconds.
fn summary(samples: &[Duration]) -> String {
    let mut sorted = samples.to_vec();
    sorted.sort();
    let n = sorted.len();
    if n == 0 {
        return "| 0 | - | - | - | - |".to_owned();
    }
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    let mean = sorted.iter().map(|d| us(*d)).sum::<f64>() / n as f64;
    let at = |q: f64| sorted[((n as f64 * q).ceil() as usize).clamp(1, n) - 1];
    format!(
        "| {n} | {mean:.3} | {:.3} | {:.3} | {:.3} |",
        us(at(0.50)),
        us(at(0.99)),
        us(sorted[n - 1])
    )
}

/// An [`Env`] for the engine rows: a clock the bench moves and the key
/// state Windows would have, from the events the engine let through.
struct BenchEnv {
    now: std::cell::Cell<Instant>,
    os_down: std::cell::RefCell<[bool; 256]>,
}

impl BenchEnv {
    fn new() -> Self {
        Self {
            now: std::cell::Cell::new(Instant::now()),
            os_down: std::cell::RefCell::new([false; 256]),
        }
    }

    fn advance(&self, ms: u64) {
        self.now.set(self.now.get() + Duration::from_millis(ms));
    }

    /// Records what Windows saw of `event` once the engine decided it.
    fn seen(&self, event: KeyEvent, suppressed: bool) {
        if !suppressed {
            self.os_down.borrow_mut()[usize::from(event.vk.0 & 0xFF)] =
                event.direction == KeyDirection::Down;
        }
    }
}

impl Env for BenchEnv {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
        let down = self.os_down.borrow();
        let at = |key: VIRTUAL_KEY| down[usize::from(key.0 & 0xFF)];
        match vk {
            VK_SHIFT => at(VK_LSHIFT) || at(VK_RSHIFT),
            VK_CONTROL => at(VK_LCONTROL) || at(VK_RCONTROL),
            VK_MENU => at(VK_LMENU) || at(VK_RMENU),
            other => at(other),
        }
    }

    fn foreground_app(&self) -> Option<&str> {
        None
    }

    fn foreground_title(&self) -> Option<&str> {
        None
    }
}

fn bindings(ron: &str) -> Vec<Binding> {
    let mut bindings = from_ron_str(ron).expect("invariant: the bench configs load");
    sort_by_specificity(&mut bindings);
    bindings
}

/// The template's two bindings, without `block_original_combo` (which reads
/// the registry), with the README's CapsLock pair in front, the template's
/// LShift DoubleTap, a chord and a sequence.
const TYPING_CONFIG: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
    (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    (keys: "CapsLock", action: SwitchLanguage),
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
    (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)),
    (chord: (keys: ["F14", "F15"], window_ms: 50), action: SwitchLanguage),
    (sequence: (steps: ["F13", "F16"]), action: SwitchLanguage),
] )"#;

const GESTURE_CONFIG: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
    (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    (keys: "F13", action: DoNothing, trigger: Hold(300)),
] )"#;

/// The events of typing `text` once, with the left Shift for capitals, and
/// the clock step before each.
fn typing(text: &str) -> Vec<(u64, KeyEvent)> {
    let mut events = Vec::new();
    for c in text.chars() {
        let vk = match c {
            ' ' => VK_SPACE,
            c if c.is_ascii_alphabetic() => {
                parse_vk(&c.to_string()).expect("invariant: a letter has a key name")
            }
            _ => continue,
        };
        let shifted = c.is_ascii_uppercase();
        if shifted {
            events.push((40, KeyEvent::down(VK_LSHIFT)));
        }
        events.push((30, KeyEvent::down(vk)));
        events.push((60, KeyEvent::up(vk)));
        if shifted {
            events.push((20, KeyEvent::up(VK_LSHIFT)));
        }
    }
    events
}

/// R1: one engine per round of `per_round` events, each timed alone.
fn r1_typing(rounds: usize, per_round: usize) -> Vec<Duration> {
    let bindings = bindings(TYPING_CONFIG);
    let script = typing("The Quick brown Fox jumps over the Lazy dog and Types On ");
    let mut samples = Vec::with_capacity(rounds * per_round);
    for _ in 0..rounds {
        let env = BenchEnv::new();
        let mut engine = Engine::new();
        for (step, event) in script.iter().cycle().take(per_round) {
            env.advance(*step);
            let begun = Instant::now();
            let decision = engine.on_key(*event, &bindings, &env);
            samples.push(begun.elapsed());
            env.seen(*event, decision.suppress);
            let _ = std::hint::black_box(decision);
        }
    }
    samples
}

/// R2: CapsLock Tap and Hold presses released after 80 ms, and F13 Hold
/// presses released after 100 ms, every key event timed.
fn r2_gestures(events: usize) -> Vec<Duration> {
    let bindings = bindings(GESTURE_CONFIG);
    let env = BenchEnv::new();
    let mut engine = Engine::new();
    let script = [
        (50, KeyEvent::down(VK_CAPITAL)),
        (80, KeyEvent::up(VK_CAPITAL)),
        (50, KeyEvent::down(VK_F13)),
        (100, KeyEvent::up(VK_F13)),
    ];
    let mut samples = Vec::with_capacity(events);
    for (step, event) in script.iter().cycle().take(events) {
        env.advance(*step);
        let begun = Instant::now();
        let decision = engine.on_key(*event, &bindings, &env);
        samples.push(begun.elapsed());
        env.seen(*event, decision.suppress);
        let _ = std::hint::black_box(decision);
    }
    samples
}

/// R3: `on_timer` of each F13 Hold, armed by its key-down and fired at its
/// term, then released.
fn r3_timers(count: usize) -> Vec<Duration> {
    let bindings = bindings(GESTURE_CONFIG);
    let env = BenchEnv::new();
    let mut engine = Engine::new();
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        env.advance(50);
        let down = KeyEvent::down(VK_F13);
        let decision = engine.on_key(down, &bindings, &env);
        env.seen(down, decision.suppress);
        let token = decision.timers[0].token;
        env.advance(300);
        let begun = Instant::now();
        let fired = engine.on_timer(token);
        samples.push(begun.elapsed());
        let _ = std::hint::black_box(fired);
        let up = KeyEvent::up(VK_F13);
        let decision = engine.on_key(up, &bindings, &env);
        env.seen(up, decision.suppress);
    }
    samples
}

/// Calls the extern keyboard callback with one event of `vk`, as the hook
/// dispatcher would.
fn call_keyboard_callback(vk: VIRTUAL_KEY, msg: u32) -> LRESULT {
    let info = KBDLLHOOKSTRUCT {
        vkCode: u32::from(vk.0),
        ..Default::default()
    };
    // SAFETY: code 0 is HC_ACTION, and lparam points to `info`, a
    // KBDLLHOOKSTRUCT on this stack frame that outlives the call: all the
    // callback dereferences. No hook is installed, so the pass path's
    // CallNextHookEx has no next hook procedure to call and delivers the
    // event nowhere.
    unsafe {
        keyboard_hook_callback(
            0,
            WPARAM(msg as usize),
            LPARAM(&info as *const KBDLLHOOKSTRUCT as isize),
        )
    }
}

/// R4: every event of F13 (bound to `DoNothing`) and A (unbound) pressed
/// and released in turn, through the extern callback.
fn r4_adapter(events: usize) -> Vec<Duration> {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_allow_extra_modifiers(true),
    ]);
    let script = [
        (VK_F13, WM_KEYDOWN),
        (VK_F13, WM_KEYUP),
        (VK_A, WM_KEYDOWN),
        (VK_A, WM_KEYUP),
    ];
    let mut samples = Vec::with_capacity(events);
    for (vk, msg) in script.iter().cycle().take(events) {
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(*vk, *msg));
        samples.push(begun.elapsed());
    }
    samples
}

/// R5: the key-down callback of an F13 Hold(50) press, which starts its
/// timer; each press is released at once, so no Hold fires.
fn r5_hold_press(count: usize) -> Vec<Duration> {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_trigger(Trigger::Hold(50))
            .with_allow_extra_modifiers(true),
    ]);
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let begun = Instant::now();
        call_keyboard_callback(VK_F13, WM_KEYDOWN);
        samples.push(begun.elapsed());
        call_keyboard_callback(VK_F13, WM_KEYUP);
    }
    samples
}

/// R9 and R10: the key-down callback of F13 bound to `DoNothing` under
/// `condition`, which does not hold, so the binding's keys match, its
/// `when:` is asked and the key passes; each key-up is called, not timed.
fn r9_conditional_key_down(condition: Condition, count: usize) -> Vec<Duration> {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_condition(condition)
            .with_allow_extra_modifiers(true),
    ]);
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(VK_F13, WM_KEYDOWN));
        samples.push(begun.elapsed());
        call_keyboard_callback(VK_F13, WM_KEYUP);
    }
    samples
}

/// R11: the key-up callback of a swallowed F13 press bound to `DoNothing`;
/// each key-down is called, not timed.
fn r11_swallowed_key_up(count: usize) -> Vec<Duration> {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_allow_extra_modifiers(true),
    ]);
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        call_keyboard_callback(VK_F13, WM_KEYDOWN);
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(VK_F13, WM_KEYUP));
        samples.push(begun.elapsed());
    }
    samples
}

/// R12: every key-down and key-up of the unbound A, with F13 bound to
/// `DoNothing`, through the extern callback: an ordinary key.
fn r12_ordinary_key(events: usize) -> Vec<Duration> {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_allow_extra_modifiers(true),
    ]);
    let mut samples = Vec::with_capacity(events);
    for msg in [WM_KEYDOWN, WM_KEYUP].iter().cycle().take(events) {
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(VK_A, *msg));
        samples.push(begun.elapsed());
    }
    samples
}

/// A requester for the watchdog rows that keeps every request unanswered;
/// with the heartbeat moving at every call, none is expected.
struct Unanswered(Mutex<Vec<mpsc::SyncSender<ReinstallOutcome>>>);

impl RequestReinstall for Unanswered {
    fn request(
        &self,
        _request: ReinstallRequest,
    ) -> Result<mpsc::Receiver<ReinstallOutcome>, MainThreadError> {
        let (reply, answer) = mpsc::sync_channel(1);
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(reply);
        Ok(answer)
    }
}

/// Binds F13 to `DoNothing`, as R12 does, so that the unbound A is an
/// ordinary key.
fn bind_f13_to_do_nothing() {
    get_hook().update_bindings(vec![
        Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
            .with_allow_extra_modifiers(true),
    ]);
}

/// Busy-waits for `wait`, so that the bench thread keeps its core.
fn spin_for(wait: Duration) {
    let begun = Instant::now();
    while begun.elapsed() < wait {
        std::hint::spin_loop();
    }
}

/// `events` timed callbacks of the unbound A, down and up in turn, each
/// `pace` after the previous one began, as steady typing comes, with
/// `before` right before each, outside the timing.
fn paced_calls(events: usize, pace: Duration, before: impl Fn()) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(events);
    let mut previous: Option<Instant> = None;
    for msg in [WM_KEYDOWN, WM_KEYUP].iter().cycle().take(events) {
        if let Some(previous) = previous {
            while previous.elapsed() < pace {
                std::hint::spin_loop();
            }
        }
        before();
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(VK_A, *msg));
        samples.push(begun.elapsed());
        previous = Some(begun);
    }
    samples
}

/// R14: steady typing with the hook watchdog ticking. Its thread watches
/// the process-wide hook's heartbeat with synthetic raw input; the first
/// raw key event of the burst (R16's) and 5 ms let it start ticking, then
/// one raw key event comes right before each paced call, outside the
/// timing. The ticking watchdog leaves them queued and reads them at its
/// ticks. The hook counts as installed for the watchdog while the row
/// runs; no hook is installed.
fn r14_steady_typing_with_the_watchdog(events: usize, pace: Duration) -> Vec<Duration> {
    bind_f13_to_do_nothing();
    let health = get_hook().health();
    health.set_hook_installed(true);
    let watchdog = HookWatchdog::start_synthetic(
        health,
        Unanswered(Mutex::new(Vec::new())),
        leaked_counters(),
    )
    .expect("invariant: the bench's watchdog starts");
    watchdog.post_synthetic_raw();
    spin_for(Duration::from_millis(5));
    let samples = paced_calls(events, pace, || watchdog.post_synthetic_raw());
    // A ticking synthetic watchdog sees its WM_QUIT at its next tick.
    assert!(watchdog.stop(), "the bench's watchdog stops within 1 s");
    health.set_hook_installed(false);
    samples
}

/// A thread with a message queue that it never reads: where R14c posts
/// the raw key events that R14 posts to the watchdog. It ends when
/// dropped, and its queue with it.
struct ParkedQueue {
    thread_id: u32,
    release: Option<mpsc::Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

impl ParkedQueue {
    fn start() -> Self {
        let (ready_tx, ready) = mpsc::channel();
        let (release, parked) = mpsc::channel::<()>();
        let join = thread::spawn(move || {
            let mut msg = MSG::default();
            // SAFETY: `msg` is a local that outlives the call, which only
            // writes it; the call makes this thread's queue, as
            // PostThreadMessageW asks of the receiving thread, and removes
            // nothing.
            let _ = unsafe { PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE) };
            // SAFETY: GetCurrentThreadId reads no memory of ours.
            let _ = ready_tx.send(unsafe { GetCurrentThreadId() });
            // Blocked on the channel, not on its queue, until dropped.
            let _ = parked.recv();
        });
        let thread_id = ready.recv().expect("invariant: the parked thread starts");
        Self {
            thread_id,
            release: Some(release),
            join: Some(join),
        }
    }

    fn post(&self) {
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; the message carries no pointer.
        let _ = unsafe { PostThreadMessageW(self.thread_id, SYNTHETIC_RAW, WPARAM(0), LPARAM(0)) };
    }
}

impl Drop for ParkedQueue {
    fn drop(&mut self) {
        drop(self.release.take());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// R14c: R14's control (ruling V3-I2): the same pacing and the same posted
/// message right before each call, to a thread that never reads its queue,
/// and no watchdog. R14 against R14c is the cost of the ticking watchdog
/// alone.
fn r14c_paced_control(events: usize, pace: Duration) -> Vec<Duration> {
    bind_f13_to_do_nothing();
    let parked = ParkedQueue::start();
    paced_calls(events, pace, || parked.post())
}

/// R16: the key-down callback of the first key of a burst. Each sample
/// starts a watchdog, lets its thread reach its idle wait, posts one raw
/// key event and at once makes the timed call, so that the watchdog's one
/// wake and first pass run next to it, the pessimistic order of cut Q6.
/// The hook counts as installed for the watchdog, as in the daemon, so the
/// first pass counts the event and sets the tick timer. Each key-up is
/// called, not timed; the thread's start and stop are outside the timing.
fn r16_first_key_of_a_burst(count: usize) -> Vec<Duration> {
    bind_f13_to_do_nothing();
    let health = get_hook().health();
    health.set_hook_installed(true);
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let watchdog = HookWatchdog::start_synthetic(
            health,
            Unanswered(Mutex::new(Vec::new())),
            leaked_counters(),
        )
        .expect("invariant: the bench's watchdog starts");
        spin_for(Duration::from_millis(2));
        watchdog.post_synthetic_raw();
        let begun = Instant::now();
        std::hint::black_box(call_keyboard_callback(VK_A, WM_KEYDOWN));
        samples.push(begun.elapsed());
        call_keyboard_callback(VK_A, WM_KEYUP);
        // It ticks now, so it sees its WM_QUIT at its first tick.
        assert!(watchdog.stop(), "the bench's watchdog stops within 1 s");
    }
    health.set_hook_installed(false);
    samples
}

/// The message R15 posts to its window.
const R15_MESSAGE: u32 = WM_APP + 0x15;

/// The class of R15's window, registered for the row only.
const R15_CLASS: windows::core::PCWSTR = w!("ks-bench-r15");

/// R15's window procedure: at `R15_MESSAGE` the work a raw input window
/// does per key event, a `GetRawInputData` (here on a null handle, which
/// fails at once and writes nothing) and the `DefWindowProcW` that frees
/// the input.
unsafe extern "system" fn r15_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == R15_MESSAGE {
        let mut raw = RAWINPUT::default();
        let mut size = size_of::<RAWINPUT>() as u32;
        // SAFETY: a null HRAWINPUT makes the call fail before it touches
        // the buffer; `raw` and `size` are locals that outlive the call and
        // `size` is the size of `raw`.
        let read = unsafe {
            GetRawInputData(
                HRAWINPUT(std::ptr::null_mut()),
                RID_INPUT,
                Some(&mut raw as *mut RAWINPUT as *mut std::ffi::c_void),
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32,
            )
        };
        std::hint::black_box(read);
    }
    // SAFETY: passes the message on with the exact arguments the system
    // gave this window procedure.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// R15: `count` rounds of post, peek and dispatch of `R15_MESSAGE` to a
/// message-only window of the bench thread, each timed whole.
fn r15_raw_input_dispatch(count: usize) -> Vec<Duration> {
    // SAFETY: a null module name asks for the handle of the file that made
    // this process; the call reads no memory of ours.
    let instance = HINSTANCE::from(
        unsafe { GetModuleHandleW(None) }.expect("invariant: the module handle is known"),
    );
    let class = WNDCLASSW {
        lpfnWndProc: Some(r15_proc),
        hInstance: instance,
        lpszClassName: R15_CLASS,
        ..Default::default()
    };
    // SAFETY: `class` is a local that outlives the call, which only reads
    // it; its procedure is a 'static extern "system" fn and its name a
    // static wide literal.
    assert_ne!(
        unsafe { RegisterClassW(&class) },
        0,
        "the R15 class is registered"
    );
    // SAFETY: the class was registered above with this module handle; the
    // window name is null and no creation data is passed; HWND_MESSAGE makes
    // a message-only window that no other process enumerates or sends input
    // to.
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            R15_CLASS,
            None,
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance),
            None,
        )
    }
    .expect("invariant: the R15 window is created");
    let mut msg = MSG::default();
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let begun = Instant::now();
        // SAFETY: `hwnd` is this thread's own window, alive until it is
        // destroyed below; the message carries no pointer. `msg` is a local
        // that outlives the calls, which write and read it and take the
        // message from this thread's own queue.
        unsafe {
            PostMessageW(Some(hwnd), R15_MESSAGE, WPARAM(0), LPARAM(0))
                .expect("invariant: the R15 window takes a posted message");
            while PeekMessageW(&mut msg, Some(hwnd), 0, 0, PM_REMOVE).as_bool() {
                DispatchMessageW(&msg);
            }
        }
        samples.push(begun.elapsed());
    }
    // SAFETY: `hwnd` is this thread's window, destroyed once, here; the
    // class has no window left once it is, so it can be unregistered.
    unsafe {
        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassW(R15_CLASS, Some(instance));
    }
    samples
}

/// R6: how late an F13 Hold(`ms`) fires its macro, `count` times, with
/// `main` the bench thread's main state, whose loop runs until a helper
/// ends it at each record.
fn r6_lateness(main: &MainThread, ms: u64, count: usize) -> Vec<Duration> {
    let (tx, records) = mpsc::channel();
    let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
        .expect("invariant: the bench's executor starts");
    get_hook().attach_executor(Some(executor.handle()));
    get_hook().update_bindings(vec![
        Binding::new(
            KeyCombination::new(VK_F13),
            BindAction::Sequence(Arc::new(vec![SequenceStep::PressKey(VK_F13)])),
        )
        .with_trigger(Trigger::Hold(ms))
        .with_allow_extra_modifiers(true),
    ]);
    let term = Duration::from_millis(ms);
    let handle = main.handle();
    let (fired_tx, fired) = mpsc::channel();
    let waiter = thread::spawn(move || {
        for _ in 0..count {
            let record = records.recv_timeout(Duration::from_secs(2));
            let _ = handle.request_exit();
            let _ = fired_tx.send(record.map(|record| record.at));
        }
    });
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let pressed = Instant::now();
        call_keyboard_callback(VK_F13, WM_KEYDOWN);
        main.run(get_hook())
            .expect("invariant: the main loop ends on WM_QUIT");
        let at = fired
            .recv_timeout(Duration::from_secs(5))
            .expect("the waiter answers")
            .expect("the Hold fires within 2 s");
        call_keyboard_callback(VK_F13, WM_KEYUP);
        samples.push(at.duration_since(pressed).saturating_sub(term));
    }
    waiter.join().expect("the waiter does not panic");
    get_hook().attach_executor(None);
    let _ = executor.stop();
    samples
}

/// A message-only `STATIC` window of the calling thread, as an integer,
/// since an `HWND` is not `Send`.
fn message_only_window() -> isize {
    // SAFETY: the class name is a static wide literal and the window name is
    // null; HWND_MESSAGE makes a message-only window that no other process
    // enumerates or sends input to. No creation data is passed.
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
    .expect("invariant: a message-only STATIC window is created");
    hwnd.0 as isize
}

/// Round trips of `count` `SendMessageTimeoutW(WM_NULL)` to `hwnd`.
fn round_trips(hwnd: isize, count: usize) -> Vec<Duration> {
    let hwnd = HWND(hwnd as *mut std::ffi::c_void);
    (0..count)
        .map(|_| {
            let begun = Instant::now();
            // SAFETY: `hwnd` is a window of the helper thread, alive until
            // that thread gets WM_QUIT after this loop; WM_NULL carries no
            // pointer, and no result pointer is passed.
            let answered = unsafe {
                SendMessageTimeoutW(
                    hwnd,
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG,
                    1000,
                    None,
                )
            };
            let took = begun.elapsed();
            assert_ne!(answered, LRESULT(0), "the loop answered WM_NULL");
            took
        })
        .collect()
}

/// R7: round trips to a window of a helper thread that runs the daemon's
/// main loop.
fn r7_round_trip(count: usize) -> Vec<Duration> {
    let (ready_tx, ready) = mpsc::channel();
    let helper = thread::spawn(move || {
        let main = MainThread::new().expect("invariant: the helper's main thread is set up");
        let hook = KeyboardHook::new();
        let hwnd = message_only_window();
        let _ = ready_tx.send((hwnd, main.handle()));
        main.run(&hook)
            .expect("invariant: the main loop ends on WM_QUIT");
        // SAFETY: `hwnd` is this thread's own window, destroyed once here.
        let _ = unsafe { DestroyWindow(HWND(hwnd as *mut std::ffi::c_void)) };
    });
    let (hwnd, main) = ready
        .recv_timeout(Duration::from_secs(5))
        .expect("the helper made its window");
    let samples = round_trips(hwnd, count);
    main.request_exit().expect("the helper has a message queue");
    helper.join().expect("the helper does not panic");
    samples
}

/// R13: R7 with the daemon's control window, and with it the thread's
/// default IME window, on the loop thread (roadmap item 2.3). The window's
/// class exists once per process, so the row takes the class lock.
fn r13_round_trip_with_the_control_window(count: usize) -> Vec<Duration> {
    let _class = CONTROL_CLASS.lock().unwrap_or_else(PoisonError::into_inner);
    let (ready_tx, ready) = mpsc::channel();
    let helper = thread::spawn(move || {
        let main = MainThread::new().expect("invariant: the helper's main thread is set up");
        let control =
            ControlWindow::create().expect("invariant: the helper's control window is created");
        let hook = KeyboardHook::new();
        let hwnd = message_only_window();
        let _ = ready_tx.send((hwnd, main.handle()));
        main.run(&hook)
            .expect("invariant: the main loop ends on WM_QUIT");
        // SAFETY: `hwnd` is this thread's own window, destroyed once here.
        let _ = unsafe { DestroyWindow(HWND(hwnd as *mut std::ffi::c_void)) };
        drop(control);
    });
    let (hwnd, main) = ready
        .recv_timeout(Duration::from_secs(5))
        .expect("the helper made its windows");
    let samples = round_trips(hwnd, count);
    main.request_exit().expect("the helper has a message queue");
    helper.join().expect("the helper does not panic");
    samples
}

/// The message R8 sends to its probe window.
const PROBE: u32 = WM_APP + 7;

/// The clock R8 measures on: nanoseconds since this instant fit an atomic.
static BASE: LazyLock<Instant> = LazyLock::new(Instant::now);

/// When the probe window's procedure last received `PROBE`, in nanoseconds
/// since [`BASE`]. Stored with Release by the loop thread inside the sent
/// message; loaded with Acquire by the sender after `SendMessageTimeoutW`
/// returned, which happens after the store.
static PROBED_AT: AtomicU64 = AtomicU64::new(0);

fn nanos_since_base() -> u64 {
    u64::try_from(BASE.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

/// The probe window's procedure: stamps `PROBE` on entry. It replaces the
/// `STATIC` class's procedure while the bench sends probes; every other
/// message goes to `DefWindowProcW`, and the original procedure is put back
/// before the window is destroyed.
unsafe extern "system" fn probe_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == PROBE {
        PROBED_AT.store(nanos_since_base(), Ordering::Release);
        return LRESULT(1);
    }
    // SAFETY: passes the message on with the exact arguments the system
    // gave this window procedure.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// A message-only `STATIC` window of the calling thread whose procedure is
/// [`probe_proc`], as an integer, with the procedure it replaced.
fn probe_window() -> (isize, isize) {
    let hwnd = HWND(message_only_window() as *mut std::ffi::c_void);
    // SAFETY: `hwnd` is the calling thread's own window; the new procedure
    // is a 'static extern "system" fn with the WNDPROC signature, and the
    // one it replaces is put back before the window is destroyed.
    let original =
        unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, probe_proc as *const () as isize) };
    assert_ne!(original, 0, "the STATIC procedure is replaced");
    (hwnd.0 as isize, original)
}

/// R8 for one priority of the loop thread: `count` sends, 1 ms apart, of
/// `PROBE` to a probe window of a helper thread that runs the main loop at
/// `priority`. Returns the one-way deliveries and the round trips.
fn r8_delivery(priority: THREAD_PRIORITY, count: usize) -> (Vec<Duration>, Vec<Duration>) {
    let (ready_tx, ready) = mpsc::channel();
    let helper = thread::spawn(move || {
        // SAFETY: the pseudo handle of the calling thread needs no closing;
        // the call changes only this thread's priority.
        unsafe { SetThreadPriority(GetCurrentThread(), priority) }
            .expect("invariant: a thread may set its own priority");
        let main = MainThread::new().expect("invariant: the helper's main thread is set up");
        let hook = KeyboardHook::new();
        let (hwnd, original) = probe_window();
        let _ = ready_tx.send((hwnd, main.handle()));
        main.run(&hook)
            .expect("invariant: the main loop ends on WM_QUIT");
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);
        // SAFETY: `hwnd` is this thread's own window; `original` is the
        // procedure SetWindowLongPtrW returned for it, put back before the
        // window is destroyed, once, here.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original);
            let _ = DestroyWindow(hwnd);
        }
    });
    let (hwnd, main) = ready
        .recv_timeout(Duration::from_secs(5))
        .expect("the helper made its window");
    let hwnd = HWND(hwnd as *mut std::ffi::c_void);
    let mut one_way = Vec::with_capacity(count);
    let mut round_trip = Vec::with_capacity(count);
    for _ in 0..count {
        thread::sleep(Duration::from_millis(1));
        let sent = nanos_since_base();
        let begun = Instant::now();
        // SAFETY: `hwnd` is a window of the helper thread, alive until that
        // thread gets WM_QUIT after this loop; PROBE carries no pointer, and
        // no result pointer is passed.
        let answered = unsafe {
            SendMessageTimeoutW(
                hwnd,
                PROBE,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                1000,
                None,
            )
        };
        round_trip.push(begun.elapsed());
        assert_ne!(answered, LRESULT(0), "the loop answered the probe");
        let received = PROBED_AT.load(Ordering::Acquire);
        one_way.push(Duration::from_nanos(received.saturating_sub(sent)));
    }
    main.request_exit().expect("the helper has a message queue");
    helper.join().expect("the helper does not panic");
    (one_way, round_trip)
}

/// R8: the rows of [`r8_delivery`] at normal and at highest priority, with
/// a busy thread at normal priority on every core meanwhile.
fn r8_under_load(count: usize) -> Vec<Row> {
    let stop = Arc::new(AtomicBool::new(false));
    let cores = thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let busy: Vec<_> = (0..cores)
        .map(|_| {
            let stop = Arc::clone(&stop);
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    std::hint::spin_loop();
                }
            })
        })
        .collect();
    let (normal_one_way, normal_trip) = r8_delivery(THREAD_PRIORITY_NORMAL, count);
    let (highest_one_way, highest_trip) = r8_delivery(THREAD_PRIORITY_HIGHEST, count);
    stop.store(true, Ordering::Release);
    for thread in busy {
        thread.join().expect("a busy thread does not panic");
    }
    vec![
        Row {
            id: "R8/normal",
            what: "one-way delivery under load, loop at NORMAL",
            samples: normal_one_way,
        },
        Row {
            id: "R8/highest",
            what: "one-way delivery under load, loop at HIGHEST",
            samples: highest_one_way,
        },
        Row {
            id: "R8/normal-rt",
            what: "round trip under load, loop at NORMAL",
            samples: normal_trip,
        },
        Row {
            id: "R8/highest-rt",
            what: "round trip under load, loop at HIGHEST",
            samples: highest_trip,
        },
    ]
}

/// Taken by the bench, which drives the process-wide hook, so that it never
/// runs beside a test that does too (it is meant to run alone).
static BENCH: Mutex<()> = Mutex::new(());

#[test]
#[ignore = "bench: KS_BENCH_OUT=<file> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1"]
fn latency_bench() {
    let _alone = BENCH.lock().unwrap_or_else(PoisonError::into_inner);
    let out = std::env::var_os("KS_BENCH_OUT")
        .expect("set KS_BENCH_OUT to the file the bench appends its table to");
    let label = std::env::var("KS_BENCH_LABEL").unwrap_or_else(|_| "unlabelled".to_owned());

    // The bench thread is the main thread for the adapter rows, as the
    // daemon's main thread is for its callbacks.
    let main = MainThread::new().expect("invariant: the bench's main thread is set up");
    let mut rows = vec![
        Row {
            id: "R1",
            what: "Engine::on_key, typing script",
            samples: r1_typing(5, 10_000),
        },
        Row {
            id: "R2",
            what: "Engine::on_key, Tap/Hold and Hold presses",
            samples: r2_gestures(10_000),
        },
        Row {
            id: "R3",
            what: "Engine::on_timer, armed Holds",
            samples: r3_timers(10_000),
        },
        Row {
            id: "R4",
            what: "keyboard_hook_callback, F13 DoNothing + A unbound",
            samples: r4_adapter(10_000),
        },
        Row {
            id: "R5",
            what: "keyboard_hook_callback, F13 Hold(50) key-down",
            samples: r5_hold_press(200),
        },
    ];
    for (ms, id, what) in [
        (5, "R6/5", "Hold(5) lateness"),
        (20, "R6/20", "Hold(20) lateness"),
        (50, "R6/50", "Hold(50) lateness"),
        (200, "R6/200", "Hold(200) lateness"),
    ] {
        rows.push(Row {
            id,
            what,
            samples: r6_lateness(&main, ms, 10),
        });
    }
    rows.push(Row {
        id: "R7",
        what: "SendMessageTimeoutW(WM_NULL) round trip to the loop",
        samples: r7_round_trip(1_000),
    });
    if std::env::var_os("KS_BENCH_LOAD").is_some_and(|load| load == "1") {
        rows.extend(r8_under_load(300));
    }
    // R9 and R10 read the foreground window from the main thread's cache,
    // as the daemon does once its hooks are installed; outside the timing.
    main.refresh_foreground();
    rows.extend([
        Row {
            id: "R9",
            what: "keyboard_hook_callback, F13 key-down, when: AppEquals (no match)",
            samples: r9_conditional_key_down(Condition::AppEquals("no-such.exe".to_owned()), 2_000),
        },
        Row {
            id: "R10",
            what: "keyboard_hook_callback, F13 key-down, when: TitleContains (no match)",
            samples: r9_conditional_key_down(
                Condition::TitleContains("no-such-title".to_owned()),
                2_000,
            ),
        },
        Row {
            id: "R11",
            what: "keyboard_hook_callback, key-up of a swallowed F13",
            samples: r11_swallowed_key_up(2_000),
        },
        Row {
            id: "R12",
            what: "keyboard_hook_callback, unbound A down and up",
            samples: r12_ordinary_key(2_000),
        },
        Row {
            id: "R13",
            what: "SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window",
            samples: r13_round_trip_with_the_control_window(1_000),
        },
        Row {
            id: "R14",
            what: "keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking",
            samples: r14_steady_typing_with_the_watchdog(2_000, Duration::from_millis(1)),
        },
        Row {
            id: "R14c",
            what: "R14 with no watchdog: same pacing, same post to a queue nobody reads",
            samples: r14c_paced_control(2_000, Duration::from_millis(1)),
        },
        Row {
            id: "R15",
            what: "post + peek + dispatch of one message whose procedure reads raw input (rejected design)",
            samples: r15_raw_input_dispatch(2_000),
        },
        Row {
            id: "R16",
            what: "keyboard_hook_callback, A key-down that wakes the idle hook watchdog",
            samples: r16_first_key_of_a_burst(1_000),
        },
    ]);
    get_hook().update_bindings(Vec::new());

    let mut table = format!(
        "\n### {label}\n\n| Row | What | n | mean us | p50 us | p99 us | max us |\n|---|---|---|---|---|---|---|\n"
    );
    for row in &rows {
        let _ = writeln!(
            table,
            "| {} | {} {}",
            row.id,
            row.what,
            summary(&row.samples)
        );
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&out)
        .expect("open KS_BENCH_OUT");
    file.write_all(table.as_bytes())
        .expect("write the table to KS_BENCH_OUT");
}
