//! The hook adapter: where Win32 keyboard events meet the decision core in
//! [`crate::engine`]. It translates and carries out; it decides nothing.
//!
//! For every `WH_KEYBOARD_LL` event, `keyboard_hook_callback`:
//!
//! 1. passes `code < 0` on untouched, as the hook contract requires;
//! 2. passes the daemon's own injected events (the sentinel in
//!    `dwExtraInfo`, see `crate::core::constants`) before any state is
//!    touched, so a synthesized key never feeds back into the engine;
//! 3. translates `WM_KEYDOWN`/`WM_SYSKEYDOWN` and `WM_KEYUP`/`WM_SYSKEYUP`
//!    into an [`engine::KeyEvent`](crate::engine::KeyEvent); any other
//!    message passes;
//! 4. asks [`Engine::on_key`](crate::engine::Engine::on_key) under the
//!    engine lock, with one snapshot of the binding list and a `HookEnv`
//!    that answers the engine's questions with live Win32 calls;
//! 5. carries out the [`Decision`] after the lock and the snapshot are
//!    released: first the timers it asks for, then its actions in order, on
//!    this thread, as before the engine existed (`Decision::mask` is
//!    reserved for roadmap item 1.2 and never set yet);
//! 6. returns `LRESULT(1)` if the decision suppresses the event, and
//!    `CallNextHookEx` otherwise.
//!
//! `foreground_changed` hands foreground switches to
//! [`Engine::on_foreground_changed`](crate::engine::Engine::on_foreground_changed)
//! the same way.
//!
//! # Threads and locking
//!
//! Both callbacks run on the main thread, which installed the hooks and
//! pumps their messages. A `Hold` arms a timer: each [`TimerRequest`] gets
//! its own short-lived `hold-timer` thread, which sleeps until the request is
//! due by `Instant::now()`, asks
//! [`Engine::on_timer`](crate::engine::Engine::on_timer) under the lock and
//! runs the Hold action itself, off the hook thread. The engine fires a Hold
//! only from `on_timer`, so a Hold action never runs on the hook thread.
//! Tap, DoubleTap, Chord and Sequence need no thread: the next event checks
//! their deadlines. Roadmap item 2.2 moves the Hold timers onto the main
//! thread and removes the lock.
//!
//! The lock (`KeyboardHook::with_engine`) is held only while the engine
//! decides. No action, `SendInput`, thread spawn or binding snapshot is held
//! across it into the carry-out. The engine reads the clock once per event,
//! under the lock, and a timer takes the lock only once its due time has
//! passed, so a key-up and its Hold timer are ordered on one clock: with
//! equal Tap and Hold terms exactly one of the two fires.
//!
//! # Re-entry
//!
//! The low-level hook is re-entered on the main thread only while that
//! thread waits in a message-retrieving call. Nothing under the lock does
//! that, and `with_engine` refuses a nested call on the same thread, so such
//! an event would pass instead of deadlocking. That keeps the suppression
//! invariant for a refused key-down (its key-up then passes too), but not
//! for a refused key-up whose key-down was suppressed: the key-up would pass
//! and the engine would keep the key held and blocked, so the key's next
//! press would count as an auto-repeat and its release would be swallowed.
//! No such re-entry exists today. A change that adds a window to the daemon,
//! or anything else that pumps messages under the lock (for example a
//! foreground lookup that sends `WM_GETTEXT` to a window of this process),
//! must deal with it first; roadmap items 2.2 and 3.4 remove the lock.

use std::cell::OnceCell;
use std::sync::LazyLock;
use std::thread;
use std::time::Instant;

use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::HWINEVENTHOOK, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

use crate::core::constants;
use crate::data::condition::ForegroundContext;
use crate::engine::{Decision, Env, KeyEvent, TimerRequest};
use crate::hook::keyboard_hook::KeyboardHook;

/// Process-wide hook singleton, lazily initialised on first access. `LazyLock`
/// rather than a plain `static` because `KeyboardHook::new()` allocates the
/// `ArcSwap` heap slot for the bindings list and can no longer be `const`.
static KEYBOARD_HOOK: LazyLock<KeyboardHook> = LazyLock::new(KeyboardHook::new);

/// Shared reference to the process-wide hook singleton. Returning `&'static`
/// (not `&'static mut`) keeps re-entrant callback invocations sound: multiple
/// shared references may coexist without violating Rust's aliasing rules. All
/// mutable state lives behind atomics, an `ArcSwap` or the engine lock inside
/// `KeyboardHook`.
pub fn get_hook() -> &'static KeyboardHook {
    &KEYBOARD_HOOK
}

/// The engine's [`Env`], backed by live Win32 calls. One is created on the
/// stack per event. The foreground window is captured at most once, on the
/// first question about it, so bindings without a `when:` condition never
/// cause a foreground lookup.
struct HookEnv {
    foreground: OnceCell<ForegroundContext>,
}

impl HookEnv {
    fn new() -> Self {
        Self {
            foreground: OnceCell::new(),
        }
    }

    fn foreground(&self) -> &ForegroundContext {
        self.foreground.get_or_init(ForegroundContext::capture)
    }
}

impl Env for HookEnv {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
        // SAFETY: GetAsyncKeyState takes the key code by value, writes no
        // memory of ours and may be called from any thread; a code outside
        // the virtual-key range only reads as up.
        let state = unsafe { GetAsyncKeyState(i32::from(vk.0)) };
        (state as u16) & 0x8000 != 0
    }

    fn foreground_app(&self) -> Option<&str> {
        self.foreground().app()
    }

    fn foreground_title(&self) -> Option<&str> {
        self.foreground().title()
    }
}

/// Translates a hook message into an engine event, or `None` for anything
/// but the four key messages.
fn key_event(wparam: WPARAM, vk_code: u32) -> Option<KeyEvent> {
    // `vkCode` is documented as 1..=254, so the narrowing keeps the code.
    let vk = VIRTUAL_KEY(vk_code as u16);
    match u32::try_from(wparam.0).ok()? {
        WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyEvent::down(vk)),
        WM_KEYUP | WM_SYSKEYUP => Some(KeyEvent::up(vk)),
        _ => None,
    }
}

/// Asks the engine about one hook event. `None` means "pass it, the engine
/// has not seen it": an event the daemon injected itself (checked before any
/// state is touched), a message other than the four key messages, or an
/// event refused because this thread is already deciding (see Re-entry in
/// the module docs).
fn decide_event(
    hook: &KeyboardHook,
    wparam: WPARAM,
    vk_code: u32,
    extra_info: usize,
) -> Option<Decision> {
    if constants::is_injected_event(extra_info) {
        return None;
    }
    let event = key_event(wparam, vk_code)?;
    // One snapshot for the whole decision, even if a reload swaps the list
    // meanwhile. It is dropped on return, before any action runs.
    let bindings = hook.bindings();
    let env = HookEnv::new();
    hook.with_engine(|engine| engine.on_key(event, &bindings, &env))
}

/// Carries out a decision on the calling thread, outside the engine lock:
/// first the timers it asks for, then its actions in order.
fn carry_out(decision: Decision) {
    let Decision { fire, timers, .. } = decision;
    for request in timers {
        start_hold_timer(request);
    }
    for action in &fire {
        action.execute();
    }
}

/// Starts the timer for a Hold on its own `hold-timer` thread, which runs
/// the Hold action if the engine still wants it.
///
/// If the thread cannot be started, the failure is logged and the engine
/// forgets the Hold, which then never fires. Panicking here would abort the
/// daemon from inside the hook callback (KS-33), and firing the Hold later
/// from a key-up would run it on the hook thread.
fn start_hold_timer(request: TimerRequest) {
    let spawned = thread::Builder::new()
        .name("hold-timer".into())
        .spawn(move || {
            sleep_until(request.due);
            if let Some(decision) = get_hook().with_engine(|engine| engine.on_timer(request.token))
            {
                carry_out(decision);
            }
        });
    if let Err(error) = spawned {
        eprintln!("[WARN] could not start a hold timer, that Hold will not fire: {error}");
        get_hook().with_engine(|engine| engine.forget_timer(request.token));
    }
}

/// Sleeps until `due` by `Instant::now()`, the clock the engine decides
/// with. `thread::sleep` never returns early, and the loop makes sure of it
/// against this clock.
fn sleep_until(due: Instant) {
    loop {
        let now = Instant::now();
        if now >= due {
            return;
        }
        thread::sleep(due - now);
    }
}

pub(crate) unsafe extern "system" fn keyboard_hook_callback(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        // SAFETY: passes the event on with the exact arguments the system
        // gave this hook procedure, as the hook contract requires for a
        // negative code.
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // SAFETY: for code >= 0 (HC_ACTION) lparam points to a valid
    // KBDLLHOOKSTRUCT supplied by the hook dispatcher for the duration of
    // this call; two fields are copied out of it here.
    let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if let Some(decision) = decide_event(get_hook(), wparam, event.vkCode, event.dwExtraInfo) {
        let suppress = decision.suppress;
        carry_out(decision);
        if suppress {
            return LRESULT(1);
        }
    }

    // SAFETY: passes the event on with the exact arguments the system gave
    // this hook procedure.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Foreground-window-change callback installed via `SetWinEventHook`. The
/// engine resyncs the modifiers and forgets which key-ups were due to be
/// swallowed, repairing what the keyboard hook may have missed while a
/// higher-integrity window had focus.
pub(crate) unsafe extern "system" fn foreground_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    let env = HookEnv::new();
    // `None` only on re-entry, which nothing triggers today (module docs).
    get_hook().with_engine(|engine| engine.on_foreground_changed(&env));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::injected_sentinel;
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::trigger::Trigger;

    // These tests drive `decide_event` on a local `KeyboardHook`: no hook is
    // installed, no action runs and no timer starts (`carry_out` is never
    // called). `HookEnv` only reads `GetAsyncKeyState`, and a single-key
    // binding on F13 matches whatever modifiers the machine has held.

    fn hook_with(binding: Binding) -> KeyboardHook {
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![binding]);
        hook
    }

    fn message(msg: u32) -> WPARAM {
        WPARAM(msg as usize)
    }

    #[test]
    fn physical_key_events_reach_the_engine() {
        let hook = hook_with(Binding::new(
            KeyCombination::new(VK_F13),
            BindAction::DoNothing,
        ));
        let f13 = u32::from(VK_F13.0);

        // All four key messages, and each key-up follows a suppressed
        // key-down, so it must reach the engine to be suppressed too.
        for (down, up) in [(WM_KEYDOWN, WM_KEYUP), (WM_SYSKEYDOWN, WM_SYSKEYUP)] {
            let pressed =
                decide_event(&hook, message(down), f13, 0).expect("a key-down reaches the engine");
            assert!(pressed.suppress);
            assert_eq!(pressed.fire, [BindAction::DoNothing]);

            let released =
                decide_event(&hook, message(up), f13, 0).expect("a key-up reaches the engine");
            assert!(released.suppress);
            assert!(released.fire.is_empty());
        }

        let unbound = decide_event(&hook, message(WM_KEYDOWN), u32::from(VK_A.0), 0)
            .expect("an unbound key-down reaches the engine");
        assert!(!unbound.suppress);
        assert!(unbound.fire.is_empty());
    }

    #[test]
    fn injected_and_unknown_events_bypass_the_engine() {
        let hook = hook_with(
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_trigger(Trigger::Tap(10_000)),
        );
        let f13 = u32::from(VK_F13.0);

        assert_eq!(
            decide_event(&hook, message(WM_KEYDOWN), f13, injected_sentinel()),
            None
        );
        // Had the injected key-down reached the engine, F13 would count as
        // held and this physical key-down as an auto-repeat, which a Tap
        // binding skips and passes.
        let pressed = decide_event(&hook, message(WM_KEYDOWN), f13, 0)
            .expect("a physical key-down reaches the engine");
        assert!(pressed.suppress);

        assert_eq!(decide_event(&hook, message(WM_CHAR), f13, 0), None);
    }
}
