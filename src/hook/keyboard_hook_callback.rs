//! The hook adapter: where Win32 keyboard events meet the decision core in
//! [`crate::engine`]. It translates and carries out; it decides nothing.
//!
//! For every `WH_KEYBOARD_LL` event, `keyboard_hook_callback`:
//!
//! 1. passes `code < 0` on untouched, as the hook contract requires;
//! 2. passes every event untouched, without reading it, while remapping is
//!    off (`swch off`, `KeyboardHook::is_enabled`); the hooks stay
//!    installed;
//! 3. passes the daemon's own injected events (the sentinel in
//!    `dwExtraInfo`, see `crate::core::constants`) before any state is
//!    touched, so a synthesized key never feeds back into the engine;
//! 4. translates `WM_KEYDOWN`/`WM_SYSKEYDOWN` and `WM_KEYUP`/`WM_SYSKEYUP`
//!    into an [`engine::KeyEvent`](crate::engine::KeyEvent); any other
//!    message passes;
//! 5. asks [`Engine::on_key`](crate::engine::Engine::on_key) on the main
//!    thread's state (`hook::main_thread::with_main`), with one snapshot of
//!    the binding list and a `HookEnv` that answers the engine's questions
//!    with live Win32 calls, and hands the timers the decision asks for to
//!    the main thread's gesture timers, under the same borrow;
//! 6. carries out the [`Decision`] after the borrow and the snapshot are
//!    released, on this thread: first the menu mask key if
//!    `Decision::mask` asks for it (`constants::MENU_MASK_VK` down and up in
//!    one `SendInput`, tagged with the sentinel; roadmap item 1.2, KS-12),
//!    then its actions in order. A fast action
//!    ([`FastAction`](crate::core::windows_actions::FastAction): `PressKey`, `Media`, `ToggleCapsLock`,
//!    `SwitchLanguage*`, `PostMessage`) runs here, before the hook returns,
//!    so that its output stays in order with the keys typed after it;
//!    `Launch`, `OpenUrl` and `Sequence` are queued for the action executor
//!    (`crate::core::executor`, roadmap item 2.1, KS-08) without waiting;
//! 7. returns `LRESULT(1)` if the decision suppresses the event, and
//!    `CallNextHookEx` otherwise.
//!
//! `foreground_changed` hands foreground switches to
//! [`Engine::on_foreground_changed`](crate::engine::Engine::on_foreground_changed)
//! the same way, and does nothing while remapping is off.
//!
//! # Threads
//!
//! Both callbacks run on the main thread, which installed the hooks, pumps
//! their messages and owns the engine and every gesture timer
//! (`hook::main_thread`, roadmap item 2.2). A gesture that must fire while
//! no event arrives, a `Hold` or a `Tap` that waits for its DoubleTap
//! window, asks for a timer, and so does a pending chord or sequence for its
//! expiry; the main loop takes the timer once it is due by `Instant::now()`
//! and carries out its decision with `carry_out` on the main thread too,
//! with the same split: a fast action runs there, and `Launch`, `OpenUrl`
//! and `Sequence` go to the action executor. No thread is started on the
//! gesture path (KS-33).
//!
//! `swch on` and `swch off` are requests that the main loop serves between
//! callbacks (`MainState::set_enabled`): the switch flips and the engine is
//! reset with no event decided in between. They never install or remove a
//! hook; only the main thread does.
//!
//! The engine reads the clock once per event, here, and the loop takes a
//! timer only once its due time has passed. The engine decides by
//! `Instant`, so a key-up and the Hold timer of its press are decided in
//! clock order, whichever of the two the loop sees first: with equal Tap and
//! Hold terms exactly one of the two fires.
//!
//! # Re-entry
//!
//! The low-level hook is re-entered on the main thread while that thread
//! waits in a message-retrieving call, and while it is inside `SendInput`:
//! the system calls the hook again for the events the call injects
//! (AutoHotkey's hook relies on this). The menu mask key and every key an
//! action sends are injected by `carry_out`, after the main state's borrow
//! and the snapshot are released, and they carry the sentinel, so
//! `decide_event` passes them before it asks for the main state: no event
//! of ours reaches the engine. Microsoft does not document whether another
//! callback, such as `foreground_changed`, can run on this thread while
//! `SendInput` waits; one that does finds the state free and does what any
//! foreground change does.
//!
//! Nothing under the borrow waits for messages or injects input, and
//! `with_main` refuses a nested call on the same thread, so an event that
//! did arrive under the borrow would pass instead of panicking or hanging.
//! That keeps the suppression invariant for a refused key-down (its key-up
//! then passes too), but not for a refused key-up whose key-down was
//! suppressed: the key-up would pass and the engine would keep the key held
//! and blocked, so the key's next press would count as an auto-repeat and
//! its release would be swallowed. No such re-entry exists today. A change
//! that injects input or pumps messages under the borrow (for example a
//! foreground lookup that sends `WM_GETTEXT` to a window of this process)
//! must deal with it first.

use std::cell::OnceCell;
use std::sync::LazyLock;
use std::time::Instant;

use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::HWINEVENTHOOK, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

use crate::core::constants;
use crate::core::executor::TargetWindow;
use crate::core::windows_actions::BindAction;
use crate::data::condition::ForegroundContext;
use crate::engine::{Decision, Env, KeyEvent};
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::main_thread::with_main;
use crate::logging::counters::{self, Counter};

/// Process-wide hook singleton, lazily initialised on first access. `LazyLock`
/// rather than a plain `static` because `KeyboardHook::new()` allocates the
/// `ArcSwap` heap slot for the bindings list and can no longer be `const`.
static KEYBOARD_HOOK: LazyLock<KeyboardHook> = LazyLock::new(KeyboardHook::new);

/// Shared reference to the process-wide hook singleton. Returning `&'static`
/// (not `&'static mut`) keeps re-entrant callback invocations sound: multiple
/// shared references may coexist without violating Rust's aliasing rules. All
/// mutable state in `KeyboardHook` lives behind atomics or an `ArcSwap`; the
/// engine belongs to the main thread (`hook::main_thread`).
pub fn get_hook() -> &'static KeyboardHook {
    &KEYBOARD_HOOK
}

/// The engine's [`Env`], backed by live Win32 calls. One is created on the
/// stack per event, and one per reset in `MainState::set_enabled`. The
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

/// Asks the engine about one hook event, on this thread's main state, and
/// starts the timers the decision asks for. `None` means "pass it, the
/// engine has not seen it": an event the daemon injected itself (checked
/// before any state is touched), a message other than the four key
/// messages, an event while remapping is off, or an event refused because
/// the main state is borrowed or this thread has none (see Re-entry in the
/// module docs).
pub(super) fn decide_event(
    hook: &KeyboardHook,
    wparam: WPARAM,
    vk_code: u32,
    extra_info: usize,
) -> Option<Decision> {
    if constants::is_injected_event(extra_info) {
        return None;
    }
    let event = key_event(wparam, vk_code)?;
    // The callback checked the switch already; this is for the callers that
    // come here directly. It flips only on this thread, between callbacks
    // (`MainState::set_enabled`), so it cannot flip from here to the
    // decision.
    if !hook.is_enabled() {
        return None;
    }
    // One snapshot for the whole decision, even if a reload swaps the list
    // meanwhile. It is dropped on return, before any action runs.
    let bindings = hook.bindings();
    let env = HookEnv::new();
    let (decision, refused) = with_main(|main| {
        let mut decision = main.engine.on_key(event, &bindings, &env);
        let timers = std::mem::take(&mut decision.timers);
        let refused = main.engine.start_timers(timers, &mut main.timers);
        (decision, refused)
    })?;
    // The engine has forgotten a timer the main thread could not set: that
    // Hold never fires (KS-33).
    for _ in 0..refused {
        counters::bump(Counter::HoldTimerNotStarted);
    }
    Some(decision)
}

/// Carries out a decision on the calling thread, outside the main state's
/// borrow: first the menu mask key if it asks for one, then its actions in
/// order (see [`dispatch`]). The mask belongs to the event being decided, so
/// it goes ahead of every key an action sends. Runs in the hook callback and
/// in the main loop's timer drain.
pub(super) fn carry_out(hook: &KeyboardHook, decision: Decision) {
    // Destructured without `..`: a new field of `Decision` does not compile
    // until it is carried out here. `suppress` is the callback's to act on;
    // `timers` were started under the borrow that decided the event or the
    // timer (`decide_event`, the main loop), so none reach this point.
    let Decision {
        suppress: _,
        fire,
        timers: _,
        mask,
    } = decision;
    if mask {
        send_menu_mask();
    }
    for action in fire {
        dispatch(hook, action);
    }
}

/// Runs a fast action here, or queues a slow one for the action executor
/// with the window a macro acts on. Neither waits: a queue that is full or
/// an executor that has stopped is counted by `submit`, and `Closed` means
/// no executor is attached, at startup or exit.
fn dispatch(hook: &KeyboardHook, action: BindAction) {
    match action.as_fast() {
        Some(fast) => fast.run(),
        None => {
            let target = TargetWindow::for_action(&action);
            let _ = hook.submit(action, target);
        }
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

    // Our own work only: from here until the decision is carried out, not
    // the passes above, which do nothing, nor CallNextHookEx (roadmap item
    // 2.1). Two clock reads and one RMW per event.
    let started = Instant::now();
    // SAFETY: for code >= 0 (HC_ACTION) lparam points to a valid
    // KBDLLHOOKSTRUCT supplied by the hook dispatcher for the duration of
    // this call; two fields are copied out of it here.
    let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let suppress = match decide_event(hook, wparam, event.vkCode, event.dwExtraInfo) {
        Some(decision) => {
            let suppress = decision.suppress;
            carry_out(hook, decision);
            suppress
        }
        None => false,
    };
    counters::record_callback(started.elapsed());
    if suppress {
        return LRESULT(1);
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
    let _ = with_main(|main| main.engine.on_foreground_changed(&env));
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, Mutex, PoisonError, mpsc};
    use std::thread;
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
    use crate::hook::main_thread::{MainThread, REFUSED, with_main};

    // No test installs a hook. Every test that reaches the engine makes its
    // thread a main thread (`MainThread::new`), which owns the engine and
    // its gesture timers; no loop runs unless a test says so. The
    // `decide_event` tests use a local `KeyboardHook`, so no action runs
    // (`carry_out` is not called); the `carry_out` tests run only
    // `DoNothing` here and send every other action to an executor whose
    // runner only records, and a fast action is only recorded under test
    // anyway; the extern-callback tests drive the process-wide hook, one at a
    // time (`GLOBAL_HOOK`), and say why that is safe. `HookEnv` only
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
            let _ = with_main(|main| main.set_enabled(get_hook(), true));
        }
    }

    /// Turns remapping on or off on `hook` through this thread's main
    /// state, as the main thread serves `swch on` and `off`, and returns
    /// whether the switch changed.
    fn set_enabled(hook: &KeyboardHook, enabled: bool) -> bool {
        with_main(|main| main.set_enabled(hook, enabled))
            .expect("this thread has a free main state")
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
        let _main = MainThread::new().expect("the main thread is set up");
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
        let _main = MainThread::new().expect("the main thread is set up");
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
        let _main = MainThread::new().expect("the main thread is set up");
        let hook = hook_with(
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        );
        let f13 = u32::from(VK_F13.0);

        assert!(set_enabled(&hook, false));
        assert_eq!(
            decide_event(&hook, message(WM_KEYDOWN), f13, 0),
            None,
            "remapping off: a bound key-down passes without reaching the engine"
        );
        assert_eq!(decide_event(&hook, message(WM_KEYUP), f13, 0), None);

        assert!(set_enabled(&hook, true));
        let pressed = decide_event(&hook, message(WM_KEYDOWN), f13, 0)
            .expect("remapping on: the key-down reaches the engine again");
        assert!(pressed.suppress);
    }

    #[test]
    fn extern_callback_swallows_a_bound_key_only_while_remapping_is_on() {
        // The only test of `LRESULT(1)` iff `Decision::suppress` at the
        // extern level. Its remapping-off asserts hold with or without the
        // callback's gate before it asks for the main state, because
        // `decide_event` checks the switch too;
        // `remapping_off_callbacks_pass_without_asking_for_the_engine` pins
        // that gate (A5-m1). The callback reads the process-wide hook, so
        // this test holds `GLOBAL_HOOK`. `carry_out` runs here, and the
        // binding is `DoNothing` so that it starts and sends nothing. If the
        // user holds Win or Alt meanwhile, the swallowed F13 asks for the
        // menu mask key, which under test is only counted (`send_menu_mask`).
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        let _on_at_end = RemappingOnAtEnd;
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

        assert!(set_enabled(get_hook(), false));
        assert_ne!(
            call_keyboard_callback(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping off: the bound key is passed on"
        );
        assert_ne!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));

        assert!(set_enabled(get_hook(), true));
        assert_eq!(
            call_keyboard_callback(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping on again: the bound key is swallowed"
        );
        assert_eq!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));
    }

    #[test]
    fn the_keyboard_callback_records_how_long_it_ran() {
        // Roadmap item 2.1: the gauge lives in the process-wide counters,
        // which no test takes (no test starts the daemon's writer), so an
        // event of another test can only raise it. The callback reads the
        // process-wide hook, hence `GLOBAL_HOOK`; the binding is
        // `DoNothing`, so nothing is sent.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        let f13 = u32::from(VK_F13.0);

        call_keyboard_callback(f13, WM_KEYDOWN);
        call_keyboard_callback(f13, WM_KEYUP);

        assert!(counters::COUNTERS.longest_callback() > Duration::ZERO);
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
    fn a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued() {
        // Roadmap item 2.2, KS-33: a Hold press sets the main thread's one
        // gesture timer instead of starting a thread, and a cancelled Hold's
        // timer is dropped when the next one is started. The callback reads
        // the process-wide hook, hence `GLOBAL_HOOK`; the action is
        // `DoNothing`, and none fires within the 10 s term.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_trigger(Trigger::Hold(10_000))
                .with_allow_extra_modifiers(true),
        ]);
        let f13 = u32::from(VK_F13.0);

        for _ in 0..1000 {
            assert_eq!(call_keyboard_callback(f13, WM_KEYDOWN), LRESULT(1));
            assert_eq!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));
        }

        let (queued, live) = with_main(|main| {
            let queued = main.timers.queue.len();
            let far = Instant::now() + Duration::from_secs(60);
            let live = std::iter::from_fn(|| main.timers.queue.take_due(far))
                .filter(|timer| main.engine.is_live(timer.token))
                .count();
            (queued, live)
        })
        .expect("the main state is free");
        assert!(queued <= 1, "{queued} timers queued after 1000 presses");
        assert_eq!(live, 0, "no press is pending after its key-up");
    }

    #[test]
    fn remapping_off_callbacks_pass_without_asking_for_the_engine() {
        // A5-m1, the successor of the engine-lock test: while remapping is
        // off both callbacks return before they ask for the main state. The
        // test holds the state borrowed while they run, so a callback that
        // asks is refused, and the refusal is counted. With remapping on the
        // keyboard callback does ask: the control.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        let _on_at_end = RemappingOnAtEnd;
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        assert!(set_enabled(get_hook(), false));
        let f13 = u32::from(VK_F13.0);

        let (down, up, refused) = with_main(|_| {
            let before = REFUSED.with(Cell::get);
            let down = call_keyboard_callback(f13, WM_KEYDOWN);
            let up = call_keyboard_callback(f13, WM_KEYUP);
            // SAFETY: `foreground_changed` reads none of its arguments, so
            // the null handles are never dereferenced or passed on. No hook
            // is installed and no message is pumped here.
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
            (down, up, REFUSED.with(Cell::get) - before)
        })
        .expect("the main state is free");
        assert_ne!(down, LRESULT(1), "remapping off: the bound key-down passes");
        assert_ne!(up, LRESULT(1), "remapping off: its key-up passes");
        assert_eq!(
            refused, 0,
            "with remapping off neither callback asked for the main state"
        );

        assert!(set_enabled(get_hook(), true));
        let refused = with_main(|_| {
            let before = REFUSED.with(Cell::get);
            let down = call_keyboard_callback(f13, WM_KEYDOWN);
            assert_ne!(down, LRESULT(1), "a refused key-down passes");
            REFUSED.with(Cell::get) - before
        })
        .expect("the main state is free");
        assert_eq!(
            refused, 1,
            "with remapping on the keyboard callback asks for the main state"
        );
    }

    #[test]
    fn remapping_off_keyboard_callback_returns_before_it_times_itself() {
        // The callback's own switch check is the fast gate: it returns before
        // the timing work. `decide_event` has a second check, so the refusal
        // count of the test above cannot tell the two apart; the gauge can,
        // because only the code after the first gate records it. Every test
        // that calls the callback holds `GLOBAL_HOOK` and nothing else writes
        // the gauge, so resetting it here and reading it back is race free.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        let _on_at_end = RemappingOnAtEnd;
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        let f13 = u32::from(VK_F13.0);

        assert!(set_enabled(get_hook(), false));
        let _ = counters::COUNTERS.take();
        for _ in 0..100 {
            call_keyboard_callback(f13, WM_KEYDOWN);
            call_keyboard_callback(f13, WM_KEYUP);
        }
        assert_eq!(
            counters::COUNTERS.longest_callback(),
            Duration::ZERO,
            "with remapping off the callback passes the event before it times itself"
        );

        assert!(set_enabled(get_hook(), true));
        for _ in 0..100 {
            call_keyboard_callback(f13, WM_KEYDOWN);
            call_keyboard_callback(f13, WM_KEYUP);
        }
        assert!(
            counters::COUNTERS.longest_callback() > Duration::ZERO,
            "with remapping on the same calls are timed: the control"
        );
    }

    #[test]
    fn a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop() {
        // Roadmap items 2.1 and 2.2: a Hold fires from the main loop's
        // gesture timer, which carries out the decision through the
        // process-wide hook, so the test holds `GLOBAL_HOOK`. Nothing is
        // installed or injected: no hook exists, and the OpenUrl goes to the
        // recording runner. A waiter ends the loop once the runner ran.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let main = MainThread::new().expect("the main thread is set up");
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
        let handle = main.handle();
        let waiter = thread::spawn(move || {
            let record = records.recv_timeout(Duration::from_secs(2));
            let _ = handle.request_exit();
            record
        });

        assert_eq!(call_keyboard_callback(f13, WM_KEYDOWN), LRESULT(1));
        main.run(get_hook()).expect("the loop ends on WM_QUIT");
        let record = waiter
            .join()
            .expect("the waiter does not panic")
            .expect("the Hold fired within 2 s");
        assert_eq!(call_keyboard_callback(f13, WM_KEYUP), LRESULT(1));

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

    /// Roadmap item 2.2, KS-33: no thread is started on the gesture path, so
    /// the thread count does not grow with the presses. The non-test lines
    /// of every file under `src/hook` but the IPC server and the config
    /// watcher (which start their own threads once, at startup), of every
    /// file under `src/engine`, and of the files the fast actions and the
    /// gesture timer run in, name no way to start a thread. The tokens are
    /// assembled, so this test does not match itself.
    #[test]
    fn the_gesture_path_starts_no_thread() {
        let tokens = [
            concat!("thread::", "spawn"),
            concat!("thread::", "Builder"),
            concat!("thread::", "scope"),
            concat!("use std::", "thread"),
        ];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        for folder in ["src/hook", "src/engine"] {
            for entry in std::fs::read_dir(root.join(folder)).expect("list a source folder") {
                let path = entry.expect("a folder entry").path();
                let starts_its_own = ["ipc_server.rs", "config_watcher.rs"]
                    .iter()
                    .any(|name| path.file_name().is_some_and(|file| file == *name));
                if path.extension().is_some_and(|extension| extension == "rs") && !starts_its_own {
                    files.push(path);
                }
            }
        }
        for file in [
            "src/core/windows_actions.rs",
            "src/system/system_function.rs",
            "src/platform/timers.rs",
        ] {
            files.push(root.join(file));
        }
        files.sort();
        assert!(files.len() >= 15, "the scan found the sources: {files:?}");

        let mut sites = Vec::new();
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            for (index, line) in non_test_lines(&text).iter().enumerate() {
                if tokens.iter().any(|token| line.contains(token)) {
                    sites.push(format!("{relative}:{}: {}", index + 1, line.trim()));
                }
            }
        }
        assert!(
            sites.is_empty(),
            "threads started on the gesture path:\n{}",
            sites.join("\n")
        );
    }

    #[test]
    #[ignore = "manual: injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus and check that it passes and nothing is typed and no menu opens"]
    fn the_menu_mask_key_is_inserted_into_the_input_stream() {
        assert_eq!(inject_menu_mask(), 2);
    }
}
