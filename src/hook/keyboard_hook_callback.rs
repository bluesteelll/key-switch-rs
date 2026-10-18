//! The hook adapter: where Win32 keyboard events meet the decision core in
//! [`crate::engine`]. It translates and carries out; it decides nothing.
//!
//! For every `WH_KEYBOARD_LL` event, `keyboard_hook_callback`:
//!
//! 1. passes `code < 0` on untouched, as the hook contract requires;
//! 2. passes every event untouched, without reading it, while remapping is
//!    off (`swch off`, `KeyboardHook::is_enabled`); the hooks stay
//!    installed, and the check is repeated under the engine lock in step 5;
//! 3. passes the daemon's own injected events (the sentinel in
//!    `dwExtraInfo`, see `crate::core::constants`) before any state is
//!    touched, so a synthesized key never feeds back into the engine;
//! 4. translates `WM_KEYDOWN`/`WM_SYSKEYDOWN` and `WM_KEYUP`/`WM_SYSKEYUP`
//!    into an [`engine::KeyEvent`](crate::engine::KeyEvent); any other
//!    message passes;
//! 5. asks [`Engine::on_key`](crate::engine::Engine::on_key) under the
//!    engine lock, if remapping is still on there, with one snapshot of the
//!    binding list and a `HookEnv` that answers the engine's questions with
//!    live Win32 calls;
//! 6. carries out the [`Decision`] after the lock and the snapshot are
//!    released, on this thread: first the menu mask key if
//!    `Decision::mask` asks for it (`constants::MENU_MASK_VK` down and up in
//!    one `SendInput`, tagged with the sentinel; roadmap item 1.2, KS-12),
//!    then the timers it asks for, then its actions in order, as before the
//!    engine existed;
//! 7. returns `LRESULT(1)` if the decision suppresses the event, and
//!    `CallNextHookEx` otherwise.
//!
//! `foreground_changed` hands foreground switches to
//! [`Engine::on_foreground_changed`](crate::engine::Engine::on_foreground_changed)
//! the same way, and does nothing while remapping is off.
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
//! `swch on` and `swch off` take the lock too, on the IPC connection worker
//! that handles them (`KeyboardHook::set_enabled`): the switch flips and the
//! engine is reset under it, and the reset only reads the OS key state.
//! They never install or remove a hook; only the main thread does.
//!
//! The lock (`KeyboardHook::with_engine`) is held only while the engine
//! decides or resets. No action, `SendInput`, thread spawn or binding snapshot is held
//! across it into the carry-out. The engine reads the clock once per event,
//! under the lock, and a timer takes the lock only once its due time has
//! passed, so a key-up and its Hold timer are ordered on one clock: with
//! equal Tap and Hold terms exactly one of the two fires.
//!
//! # Re-entry
//!
//! The low-level hook is re-entered on the main thread while that thread
//! waits in a message-retrieving call, and while it is inside `SendInput`:
//! the system calls the hook again for the events the call injects
//! (AutoHotkey's hook relies on this). The menu mask key and every key an
//! action sends are injected by `carry_out`, after the lock and the snapshot
//! are released, and they carry the sentinel, so `decide_event` passes them
//! before it takes the lock: no event of ours reaches the engine. Microsoft
//! does not document whether another callback, such as
//! `foreground_changed`, can run on this thread while `SendInput` waits; one
//! that does finds the lock free and does what any foreground change does.
//!
//! Nothing under the lock waits for messages or injects input, and
//! `with_engine` refuses a nested call on the same thread, so an event that
//! did arrive under the lock would pass instead of deadlocking. That keeps
//! the suppression invariant for a refused key-down (its key-up then passes
//! too), but not for a refused key-up whose key-down was suppressed: the
//! key-up would pass and the engine would keep the key held and blocked, so
//! the key's next press would count as an auto-repeat and its release would
//! be swallowed. No such re-entry exists today. A change that adds a window
//! to the daemon, injects input under the lock, or pumps messages under it
//! (for example a foreground lookup that sends `WM_GETTEXT` to a window of
//! this process), must deal with it first; roadmap items 2.2 and 3.4 remove
//! the lock.

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
use crate::logging::counters::{self, Counter};

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
/// stack per event, and one per reset in `KeyboardHook::set_enabled`. The
/// foreground window is captured at most once, on the first question about
/// it, so bindings without a `when:` condition never cause a foreground
/// lookup.
pub(super) struct HookEnv {
    foreground: OnceCell<ForegroundContext>,
}

impl HookEnv {
    pub(super) fn new() -> Self {
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
/// state is touched), a message other than the four key messages, an event
/// refused because this thread is already deciding (see Re-entry in the
/// module docs), or an event that finds remapping off under the lock.
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
    // The switch is checked again under the lock, where `set_enabled` flips
    // it and resets the engine. Without this, an event that passed the
    // callback's own check just before `swch off` would be decided after
    // the reset, and a Hold it arms would fire while remapping is off.
    hook.with_engine(|engine| {
        hook.is_enabled()
            .then(|| engine.on_key(event, &bindings, &env))
    })
    .flatten()
}

/// Carries out a decision on the calling thread, outside the engine lock:
/// first the menu mask key if it asks for one, then the timers it asks for,
/// then its actions in order. The mask belongs to the event being decided,
/// so it goes ahead of every key an action sends.
fn carry_out(decision: Decision) {
    // Destructured without `..`: a new field of `Decision` does not compile
    // until it is carried out here. `suppress` is the callback's to act on.
    let Decision {
        suppress: _,
        fire,
        timers,
        mask,
    } = decision;
    if mask {
        send_menu_mask();
    }
    for request in timers {
        start_hold_timer(request);
    }
    for action in &fire {
        action.execute();
    }
}

/// The menu mask key's key-down and key-up (`constants::MENU_MASK_VK`),
/// tagged with the sentinel, so that this hook passes them on without
/// asking the engine.
fn menu_mask_inputs() -> [INPUT; 2] {
    let input = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(constants::MENU_MASK_VK),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: constants::injected_sentinel(),
            },
        },
    };
    [input(KEYBD_EVENT_FLAGS(0)), input(KEYEVENTF_KEYUP)]
}

/// Inserts the menu mask key's key-down and key-up into the input stream,
/// in one call so that nothing comes between them, and returns how many
/// events were inserted.
fn inject_menu_mask() -> u32 {
    let inputs = menu_mask_inputs();
    // SAFETY: `inputs` is an array of two fully initialised INPUT_KEYBOARD
    // records on this stack frame, alive for the whole call, and cbsize is
    // the size of one INPUT, as SendInput requires; it only reads the
    // records of the slice it is given.
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) }
}

/// Sends the menu mask key. `SendInput` inserts fewer events when another
/// thread blocks input, and none when UIPI keeps it from the foreground
/// window without saying so; a short count is counted in
/// `logging::counters` (`menu_mask_short`), which the log writer reports,
/// and the event is decided as before. Nothing is logged here, on the hook
/// thread, per event.
#[cfg(not(test))]
fn send_menu_mask() {
    if inject_menu_mask() != 2 {
        counters::bump(Counter::MenuMaskShort);
    }
}

/// Under test the menu mask key is only counted, on the calling thread: a
/// test that drives `carry_out` must not inject input into the machine that
/// runs it, whatever keys its user holds.
#[cfg(test)]
fn send_menu_mask() {
    MASKS_SENT.with(|sent| sent.set(sent.get() + 1));
}

#[cfg(test)]
thread_local! {
    /// How many menu mask keys `carry_out` asked for on this thread.
    static MASKS_SENT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Starts the timer for a Hold on its own `hold-timer` thread, which runs
/// the Hold action if the engine still wants it.
///
/// If the thread cannot be started, the failure is counted in
/// `logging::counters` (`hold_timer_not_started`), which the log writer
/// reports, and the engine forgets the Hold, which then never fires.
/// Panicking here would abort the daemon from inside the hook callback
/// (KS-33), and firing the Hold later from a key-up would run it on the
/// hook thread.
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
    if spawned.is_err() {
        counters::bump(Counter::HoldTimerNotStarted);
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

    let hook = get_hook();
    if !hook.is_enabled() {
        // SAFETY: passes the event on with the exact arguments the system
        // gave this hook procedure. Remapping is off (`swch off`): the event
        // is not even read, so injected and physical events pass alike.
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // SAFETY: for code >= 0 (HC_ACTION) lparam points to a valid
    // KBDLLHOOKSTRUCT supplied by the hook dispatcher for the duration of
    // this call; two fields are copied out of it here.
    let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if let Some(decision) = decide_event(hook, wparam, event.vkCode, event.dwExtraInfo) {
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
/// higher-integrity window had focus. A modifier whose key-down the engine
/// swallowed is not resynced (Windows never saw it go down, KS-14), so a
/// lost key-up of such a modifier is not repaired here. While remapping is
/// off it does nothing: the reset at `swch on` takes every held key from the
/// OS anyway.
pub(crate) unsafe extern "system" fn foreground_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    let hook = get_hook();
    if !hook.is_enabled() {
        return;
    }
    let env = HookEnv::new();
    // `None` only on re-entry, which nothing triggers today (module docs).
    // The switch is checked again under the lock, as in `decide_event`.
    hook.with_engine(|engine| {
        if hook.is_enabled() {
            engine.on_foreground_changed(&env);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, Mutex, PoisonError, mpsc};
    use std::time::Duration;

    use super::*;
    use crate::core::constants::injected_sentinel;
    use crate::core::executor::testing::{FakeRunner, Record, leaked_counters, next_record};
    use crate::core::executor::{Executor, QUEUE_CAPACITY, Stopped};
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::sequence::SequenceStep;
    use crate::data::trigger::Trigger;

    // No test installs a hook. The `decide_event` tests use a local
    // `KeyboardHook`, so no action runs and no timer starts (`carry_out` is
    // never called); the extern-callback tests drive the process-wide hook,
    // one at a time (`GLOBAL_HOOK`), and say why that is safe. `HookEnv` only
    // reads `GetAsyncKeyState`, the live state of the machine's keyboard, so
    // every F13 binding sets `allow_extra_modifiers`: under exact matching a
    // modifier the user happens to hold while the tests run would otherwise
    // stop it matching.

    /// Taken by every test that drives the process-wide hook through
    /// `get_hook()`. One of them turns remapping off, which would break the
    /// swallow asserts of another running at the same time.
    static GLOBAL_HOOK: Mutex<()> = Mutex::new(());

    /// Turns the process-wide hook's remapping back on when dropped, also on
    /// unwind, so a failed test leaves the next one remapping on.
    struct RemappingOnAtEnd;

    impl Drop for RemappingOnAtEnd {
        fn drop(&mut self) {
            let _ = get_hook().set_enabled(true);
        }
    }

    fn hook_with(binding: Binding) -> KeyboardHook {
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![binding]);
        hook
    }

    fn message(msg: u32) -> WPARAM {
        WPARAM(msg as usize)
    }

    /// Calls the extern keyboard callback with one event of `vk`, as the
    /// hook dispatcher would, and returns what it returns.
    fn call_keyboard_callback(vk: u32, msg: u32) -> LRESULT {
        let info = KBDLLHOOKSTRUCT {
            vkCode: vk,
            ..Default::default()
        };
        // SAFETY: code 0 is HC_ACTION, and lparam points to `info`, a
        // KBDLLHOOKSTRUCT on this stack frame that outlives the call: all
        // the callback dereferences. No hook is installed and this thread is
        // not inside a hook dispatch, so the pass path's CallNextHookEx has
        // no next hook procedure to call; its hhk argument is ignored
        // (Microsoft Learn, CallNextHookEx), and it neither injects input
        // nor delivers the event to any window.
        unsafe {
            keyboard_hook_callback(
                0,
                WPARAM(msg as usize),
                LPARAM(&info as *const KBDLLHOOKSTRUCT as isize),
            )
        }
    }

    #[test]
    fn physical_key_events_reach_the_engine() {
        let hook = hook_with(
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        );
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
                .with_trigger(Trigger::Tap(10_000))
                .with_allow_extra_modifiers(true),
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

    #[test]
    fn disabled_hook_passes_every_event_without_asking_the_engine() {
        let hook = hook_with(
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        );
        let f13 = u32::from(VK_F13.0);

        assert_eq!(hook.set_enabled(false), Some(true));
        assert_eq!(
            decide_event(&hook, message(WM_KEYDOWN), f13, 0),
            None,
            "remapping off: a bound key-down passes without reaching the engine"
        );
        assert_eq!(decide_event(&hook, message(WM_KEYUP), f13, 0), None);

        assert_eq!(hook.set_enabled(true), Some(true));
        let pressed = decide_event(&hook, message(WM_KEYDOWN), f13, 0)
            .expect("remapping on: the key-down reaches the engine again");
        assert!(pressed.suppress);
    }

    #[test]
    fn extern_callback_swallows_a_bound_key_only_while_remapping_is_on() {
        // The only test of `LRESULT(1)` iff `Decision::suppress` at the
        // extern level. Its remapping-off asserts hold with or without the
        // callback's gate before the engine lock, because the check under
        // the lock passes the event too;
        // `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`
        // pins that gate (A5-m1). The callback reads the process-wide hook,
        // so this test holds `GLOBAL_HOOK`. `carry_out` runs here, and the
        // binding is `DoNothing` so that it starts and sends nothing. If the
        // user holds Win or Alt meanwhile, the swallowed F13 asks for the
        // menu mask key, which under test is only counted (`send_menu_mask`).
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let f13 = u32::from(VK_F13.0);
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        assert_eq!(
            call_keyboard_callback(f13, WM_KEYDOWN),
            LRESULT(1),
            "bound F13 down is swallowed"
        );
        assert_eq!(
            call_keyboard_callback(f13, WM_KEYUP),
            LRESULT(1),
            "its key-up is swallowed"
        );
        assert_ne!(
            call_keyboard_callback(u32::from(VK_A.0), WM_KEYDOWN),
            LRESULT(1),
            "an unbound key is passed on"
        );
        assert_ne!(
            call_keyboard_callback(u32::from(VK_A.0), WM_KEYUP),
            LRESULT(1)
        );

        assert_eq!(get_hook().set_enabled(false), Some(true));
        assert_ne!(
            call_keyboard_callback(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping off: the bound key is passed on"
        );
        assert_ne!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));

        assert_eq!(get_hook().set_enabled(true), Some(true));
        assert_eq!(
            call_keyboard_callback(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping on again: the bound key is swallowed"
        );
        assert_eq!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));
    }

    #[test]
    fn remapping_off_callbacks_pass_without_waiting_for_the_engine_lock() {
        // A5-m1: while remapping is off, both callbacks return before they
        // ask for the engine. A holder thread keeps the engine lock while
        // they run, so a callback that asks for it waits until the holder
        // gives up, after 5 s, and its time shows it.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _on_at_end = RemappingOnAtEnd;
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        get_hook()
            .set_enabled(false)
            .expect("not inside with_engine");

        // On a panic below `release_tx` is dropped before `_on_at_end`: the
        // holder's wait ends at once, so the suite does not hang.
        let (locked_tx, locked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let holder = thread::spawn(move || {
            get_hook().with_engine(|_| {
                let _ = locked_tx.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(5));
            })
        });
        locked_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the holder thread took the engine lock");

        let f13 = u32::from(VK_F13.0);
        let keyboard_start = Instant::now();
        let down = call_keyboard_callback(f13, WM_KEYDOWN);
        let up = call_keyboard_callback(f13, WM_KEYUP);
        let keyboard = keyboard_start.elapsed();

        let foreground_start = Instant::now();
        // SAFETY: `foreground_changed` reads none of its arguments, so the
        // null handles are never dereferenced or passed on. No hook is
        // installed and no message is pumped here.
        unsafe {
            foreground_changed(
                HWINEVENTHOOK::default(),
                EVENT_SYSTEM_FOREGROUND,
                HWND::default(),
                0,
                0,
                0,
                0,
            );
        }
        let foreground = foreground_start.elapsed();

        // When a callback did wait, the holder has already given up and
        // dropped its receiver; the asserts below report that, not the send.
        let _ = release_tx.send(());
        holder.join().expect("the holder thread does not panic");

        assert!(
            keyboard < Duration::from_secs(1),
            "with remapping off the keyboard callback waited {keyboard:?} for the engine lock"
        );
        assert!(
            foreground < Duration::from_secs(1),
            "with remapping off the foreground callback waited {foreground:?} for the engine lock"
        );
        assert_ne!(down, LRESULT(1), "remapping off: the bound key-down passes");
        assert_ne!(up, LRESULT(1), "remapping off: its key-up passes");
    }

    #[test]
    fn menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel() {
        let inputs = menu_mask_inputs();
        for (input, flags) in inputs.iter().zip([KEYBD_EVENT_FLAGS(0), KEYEVENTF_KEYUP]) {
            assert_eq!(input.r#type, INPUT_KEYBOARD);
            // SAFETY: every element is built with r#type INPUT_KEYBOARD, so
            // ki is the initialised field of the union.
            let ki = unsafe { input.Anonymous.ki };
            assert_eq!(ki.wVk, VIRTUAL_KEY(0xE8));
            assert_eq!(ki.wScan, 0);
            assert_eq!(ki.dwFlags, flags);
            assert_eq!(ki.time, 0);
            assert_eq!(
                ki.dwExtraInfo,
                injected_sentinel(),
                "the hook must pass the mask key it injects"
            );
        }
    }

    #[test]
    fn carry_out_sends_the_menu_mask_only_when_the_decision_asks() {
        // Under test `send_menu_mask` only counts, on this thread.
        let hook = KeyboardHook::new();
        let before = MASKS_SENT.with(Cell::get);
        carry_out(&hook, Decision::default());
        assert_eq!(MASKS_SENT.with(Cell::get), before, "no mask asked for");
        carry_out(
            &hook,
            Decision {
                mask: true,
                ..Decision::default()
            },
        );
        assert_eq!(MASKS_SENT.with(Cell::get), before + 1, "one mask asked for");
    }

    /// A local hook with an executor whose runner only records, attached.
    fn hook_with_executor(runner: FakeRunner, capacity: usize) -> (KeyboardHook, Executor) {
        let hook = KeyboardHook::new();
        let executor =
            Executor::start(runner, capacity, leaked_counters()).expect("the executor starts");
        hook.attach_executor(Some(executor.handle()));
        (hook, executor)
    }

    #[test]
    fn carry_out_queues_the_slow_actions_in_order_and_runs_none_here() {
        // Only `DoNothing` runs here: a test must not run a real action.
        let (tx, records) = mpsc::channel();
        let (hook, executor) = hook_with_executor(FakeRunner::new(tx), QUEUE_CAPACITY);

        carry_out(
            &hook,
            Decision {
                fire: vec![
                    BindAction::DoNothing,
                    BindAction::Launch {
                        exe: "no-such.exe".to_owned(),
                        args: Vec::new(),
                    },
                    BindAction::OpenUrl("https://example.com".to_owned()),
                    BindAction::Sequence(Arc::new(vec![SequenceStep::PressKey(VK_F13)])),
                ],
                ..Decision::default()
            },
        );

        let ran: Vec<Record> = (0..3).map(|_| next_record(&records)).collect();
        let texts: Vec<&str> = ran.iter().map(Record::text).collect();
        assert_eq!(
            texts,
            ["launch no-such.exe", "open https://example.com", "key F13"]
        );
        for record in &ran {
            assert_eq!(record.thread, executor.thread().id());
            assert_ne!(record.thread, thread::current().id());
        }
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "DoNothing never reaches it");
    }

    /// How long each of 20 `carry_out` calls with one `OpenUrl` took, on
    /// a fresh executor whose runner blocks 500 ms per call.
    fn twenty_carry_out_times() -> Vec<Duration> {
        let (tx, _records) = mpsc::channel();
        let (hook, executor) = hook_with_executor(
            FakeRunner::new(tx).with_block(Duration::from_millis(500)),
            64,
        );
        let times = (0..20)
            .map(|_| {
                let decision = Decision {
                    fire: vec![BindAction::OpenUrl("https://example.com".to_owned())],
                    ..Decision::default()
                };
                let begun = Instant::now();
                carry_out(&hook, decision);
                begun.elapsed()
            })
            .collect();
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
        times
    }

    #[test]
    fn carry_out_returns_in_under_5_ms_while_slow_actions_block() {
        // Roadmap item 2.1: the runner stands in for OpenUrl against a cold
        // browser. Every one of the 20 under 5 ms, one whole-batch retry.
        let limit = Duration::from_millis(5);
        let first = twenty_carry_out_times();
        if first.iter().all(|took| *took < limit) {
            return;
        }
        let second = twenty_carry_out_times();
        assert!(
            second.iter().all(|took| *took < limit),
            "a carry_out took 5 ms or more twice: {first:?}, then {second:?}"
        );
    }

    /// Detaches the executor from the process-wide hook when dropped, also
    /// on unwind, and only then stops it.
    struct DetachAtEnd(Option<Executor>);

    impl Drop for DetachAtEnd {
        fn drop(&mut self) {
            get_hook().attach_executor(None);
            if let Some(executor) = self.0.take() {
                let _ = executor.stop();
            }
        }
    }

    #[test]
    fn a_hold_runs_its_slow_action_on_the_executor() {
        // Roadmap item 2.1: a Hold fires on its `hold-timer` thread, which
        // carries out the decision through the process-wide hook, so the
        // test holds `GLOBAL_HOOK`. Nothing is installed or injected: no
        // hook exists, the mask key is only counted under test, and the
        // OpenUrl goes to the recording runner. The engine fires a Hold
        // from its timer without asking whether the key is still down.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let (tx, records) = mpsc::channel();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        get_hook().attach_executor(Some(executor.handle()));
        let executor = DetachAtEnd(Some(executor));
        get_hook().update_bindings(vec![
            Binding::new(
                KeyCombination::new(VK_F13),
                BindAction::OpenUrl("https://example.com".to_owned()),
            )
            .with_trigger(Trigger::Hold(1))
            .with_allow_extra_modifiers(true),
        ]);
        let f13 = u32::from(VK_F13.0);

        call_keyboard_callback(f13, WM_KEYDOWN);
        let record = next_record(&records);
        call_keyboard_callback(f13, WM_KEYUP);

        assert_eq!(record.text(), "open https://example.com");
        assert_eq!(record.name.as_deref(), Some("action-executor"));
        let thread = executor.0.as_ref().map(|executor| executor.thread().id());
        assert_eq!(Some(record.thread), thread);
    }

    /// The acceptance line of roadmap item 2.1: `rg "\.execute\(\)"
    /// src/hook` finds nothing, so no action runs through the old entry
    /// that ran every action on the hook thread. Every line of every file
    /// under `src/hook` counts, tests included; the call is assembled, so
    /// this test does not match itself.
    #[test]
    fn the_hook_path_names_no_execute_call() {
        let call = concat!(".exe", "cute()");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(root.join("src/hook"))
            .expect("list src/hook")
            .map(|entry| entry.expect("a folder entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect();
        files.sort();
        assert!(files.len() >= 4, "the scan found the sources: {files:?}");

        let mut sites = Vec::new();
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            for (index, line) in text.lines().enumerate() {
                if line.contains(call) {
                    sites.push(format!("{relative}:{}: {}", index + 1, line.trim()));
                }
            }
        }
        assert!(
            sites.is_empty(),
            "execute calls under src/hook:\n{}",
            sites.join("\n")
        );
    }

    #[test]
    #[ignore = "manual: injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus and check that it passes and nothing is typed and no menu opens"]
    fn the_menu_mask_key_is_inserted_into_the_input_stream() {
        assert_eq!(inject_menu_mask(), 2);
    }
}
