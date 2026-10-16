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
/// window without saying so; a short count is logged and the event is
/// decided as before.
#[cfg(not(test))]
fn send_menu_mask() {
    let sent = inject_menu_mask();
    if sent != 2 {
        eprintln!("[WARN] SendInput inserted {sent} of 2 menu mask key events");
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

    use super::*;
    use crate::core::constants::injected_sentinel;
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::trigger::Trigger;

    // No test installs a hook. The `decide_event` tests use a local
    // `KeyboardHook`, so no action runs and no timer starts (`carry_out` is
    // never called); the extern-callback test drives the process-wide hook
    // and says why that is safe. `HookEnv` only reads `GetAsyncKeyState`, the
    // live state of the machine's keyboard, so every F13 binding sets
    // `allow_extra_modifiers`: under exact matching a modifier the user
    // happens to hold while the tests run would otherwise stop it matching.

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
        // extern level, and of the gate before the engine lock. The callback
        // reads the process-wide hook, so this is the only test that may
        // touch `get_hook()`. `carry_out` runs here, and the binding is
        // `DoNothing` so that it starts and sends nothing. If the user holds
        // Win or Alt meanwhile, the swallowed F13 asks for the menu mask key,
        // which under test is only counted (`send_menu_mask`).
        let f13 = u32::from(VK_F13.0);
        get_hook().update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_allow_extra_modifiers(true),
        ]);
        let call = |vk: u32, msg: u32| {
            let info = KBDLLHOOKSTRUCT {
                vkCode: vk,
                ..Default::default()
            };
            // SAFETY: code 0 is HC_ACTION, and lparam points to `info`, a
            // KBDLLHOOKSTRUCT on this stack frame that outlives the call:
            // all the callback dereferences. No hook is installed and this
            // thread is not inside a hook dispatch, so the pass path's
            // CallNextHookEx has no next hook procedure to call; its hhk
            // argument is ignored (Microsoft Learn, CallNextHookEx), and it
            // neither injects input nor delivers the event to any window.
            unsafe {
                keyboard_hook_callback(
                    0,
                    WPARAM(msg as usize),
                    LPARAM(&info as *const KBDLLHOOKSTRUCT as isize),
                )
            }
        };
        assert_eq!(
            call(f13, WM_KEYDOWN),
            LRESULT(1),
            "bound F13 down is swallowed"
        );
        assert_eq!(call(f13, WM_KEYUP), LRESULT(1), "its key-up is swallowed");
        assert_ne!(
            call(u32::from(VK_A.0), WM_KEYDOWN),
            LRESULT(1),
            "an unbound key is passed on"
        );
        assert_ne!(call(u32::from(VK_A.0), WM_KEYUP), LRESULT(1));

        assert_eq!(get_hook().set_enabled(false), Some(true));
        assert_ne!(
            call(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping off: the bound key is passed on"
        );
        assert_ne!(call(f13, WM_KEYUP), LRESULT(1));

        assert_eq!(get_hook().set_enabled(true), Some(true));
        assert_eq!(
            call(f13, WM_KEYDOWN),
            LRESULT(1),
            "remapping on again: the bound key is swallowed"
        );
        assert_eq!(call(f13, WM_KEYUP), LRESULT(1));
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
        let before = MASKS_SENT.with(Cell::get);
        carry_out(Decision::default());
        assert_eq!(MASKS_SENT.with(Cell::get), before, "no mask asked for");
        carry_out(Decision {
            mask: true,
            ..Decision::default()
        });
        assert_eq!(MASKS_SENT.with(Cell::get), before + 1, "one mask asked for");
    }

    #[test]
    #[ignore = "manual: injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus and check that it passes and nothing is typed and no menu opens"]
    fn the_menu_mask_key_is_inserted_into_the_input_stream() {
        assert_eq!(inject_menu_mask(), 2);
    }
}
