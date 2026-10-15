use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use arc_swap::{ArcSwap, Guard};
use windows::{
    Win32::UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::*,
    },
    core::*,
};

use crate::data::binding::Binding;
use crate::engine::{self, Engine};
use crate::hook::keyboard_hook_callback::{self, HookEnv};

thread_local! {
    /// Set while this thread runs a closure inside
    /// [`KeyboardHook::with_engine`]. Per thread, so it needs no atomics.
    static DECIDING: Cell<bool> = const { Cell::new(false) };
}

/// Marks this thread as deciding until dropped, also on unwind.
struct DecidingGuard;

impl DecidingGuard {
    fn enter() -> Self {
        DECIDING.with(|deciding| deciding.set(true));
        Self
    }
}

impl Drop for DecidingGuard {
    fn drop(&mut self) {
        DECIDING.with(|deciding| deciding.set(false));
    }
}

pub struct KeyboardHook {
    /// Bindings list. Hot path reads via `ArcSwap::load()` which is
    /// effectively lock-free (no atomic CAS on the read side). Hot-reload
    /// updates the list with `update_bindings`, which atomically swaps the
    /// inner `Arc`. Any sequence currently executing keeps its own `Arc`
    /// reference until it finishes, so a swap mid-sequence never breaks an
    /// in-flight macro.
    bindings: ArcSwap<Vec<Binding>>,
    hook_handle: AtomicPtr<std::ffi::c_void>,
    foreground_hook: AtomicPtr<std::ffi::c_void>,
    /// Whether remapping is on (`swch on`, `swch off`). While it is off the
    /// hooks stay installed and their callbacks pass every event untouched.
    ///
    /// Written only by [`set_enabled`](Self::set_enabled), with a
    /// `swap(AcqRel)` under the engine lock; read with `load(Acquire)` by
    /// the callbacks, once before the lock and once under it. The flag
    /// publishes no other data, since the engine state is behind the lock,
    /// so a stale read before the lock only lets one event reach the lock,
    /// where the check under it decides.
    enabled: AtomicBool,
    /// The decision core: held and blocked keys and every pending gesture.
    /// Behind a lock until roadmap item 2.2 moves the Hold timers onto the
    /// hook thread, because a `hold-timer` thread must reach it too, and so
    /// must the IPC worker that resets it in
    /// [`set_enabled`](Self::set_enabled). Only
    /// [`with_engine`](Self::with_engine) touches it.
    engine: Mutex<Engine>,
}

impl KeyboardHook {
    /// Constructs an empty hook. Not `const` because `ArcSwap::from_pointee`
    /// requires a heap allocation; use a `LazyLock` for static initialization.
    pub fn new() -> Self {
        Self {
            bindings: ArcSwap::from_pointee(Vec::new()),
            hook_handle: AtomicPtr::new(std::ptr::null_mut()),
            foreground_hook: AtomicPtr::new(std::ptr::null_mut()),
            enabled: AtomicBool::new(true),
            engine: Mutex::new(Engine::new()),
        }
    }

    /// Runs `f` on the engine under its lock and returns what `f` returns.
    ///
    /// Returns `None`, without locking, when this thread is already inside
    /// `with_engine`. `std::sync::Mutex` may deadlock or panic on such a
    /// re-entrant lock, and the hook callback is re-entered on its own
    /// thread if anything under the lock pumps messages; the caller then
    /// passes the event instead of hanging the keyboard. Nothing under the
    /// lock does that today (see the adapter's module docs).
    ///
    /// `f` must only decide or reset: no action, `SendInput` or thread spawn
    /// under the lock. A lock poisoned by an earlier panic is taken over as it is: the
    /// engine's state is plain data, and a panic in the hook callback cannot
    /// be unwound into a usable state anyway.
    pub(crate) fn with_engine<R>(&self, f: impl FnOnce(&mut Engine) -> R) -> Option<R> {
        if DECIDING.with(Cell::get) {
            return None;
        }
        let mut engine = self.engine.lock().unwrap_or_else(PoisonError::into_inner);
        // Declared after the lock guard, so it is dropped first: the flag is
        // cleared before the lock is released.
        let _deciding = DecidingGuard::enter();
        Some(f(&mut engine))
    }

    /// Atomically replace the binding list. Callable any number of times
    /// (this is the hot-reload entry point). The new list is sorted by
    /// key-count descending so more specific combinations are checked first
    /// in the callback.
    pub fn update_bindings(&self, mut bindings: Vec<Binding>) {
        // A stable sort, so bindings with the same key count keep their
        // config order.
        engine::sort_by_specificity(&mut bindings);
        self.bindings.store(Arc::new(bindings));
    }

    /// Snapshot of the current binding list. The returned `Guard` derefs to
    /// `Arc<Vec<Binding>>` — callers can `.iter()` directly. The snapshot is
    /// stable for the guard's lifetime even if a reload happens concurrently.
    pub fn bindings(&self) -> Guard<Arc<Vec<Binding>>> {
        self.bindings.load()
    }

    /// Whether remapping is on. See [`set_enabled`](Self::set_enabled).
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Turns remapping on or off (`swch on`, `swch off`). Returns
    /// `Some(true)` if the switch changed, `Some(false)` if it was already in
    /// that position, and `None`, changing nothing, if this thread is already
    /// inside [`with_engine`](Self::with_engine).
    ///
    /// The hooks stay installed either way; only the main thread installs
    /// and removes them (KS-01). A real change resets the engine
    /// ([`Engine::reset`], KS-31), and the flag flips under the same engine
    /// lock, so an event decided under the lock sees either the old position
    /// with the old state or the new position after the reset. A change that
    /// is no change resets nothing: it would wipe the records of keys held
    /// and swallowed right now.
    ///
    /// Runs on the calling thread, an IPC connection worker in the daemon.
    /// Under the lock the reset only reads the OS key state, which may be
    /// read from any thread. Roadmap item 2.2 removes the engine lock; the
    /// flip and the reset must then move to the thread that owns the engine.
    pub fn set_enabled(&self, enabled: bool) -> Option<bool> {
        let env = HookEnv::new();
        self.with_engine(|engine| {
            if self.enabled.swap(enabled, Ordering::AcqRel) == enabled {
                return false;
            }
            engine.reset(&env);
            true
        })
    }

    /// Installs the keyboard hook and the foreground hook.
    ///
    /// Main thread only: the thread that pumps `GetMessageW`, because both
    /// hooks belong to the thread that installs them and are called through
    /// its message loop (KS-01).
    pub(crate) fn install(&self) -> Result<()> {
        if !self.hook_handle.load(Ordering::Acquire).is_null() {
            return Err(Error::new(
                windows::Win32::Foundation::E_UNEXPECTED,
                "keyboard hook is already installed",
            ));
        }

        // SAFETY: SetWindowsHookExW is the documented installer for WH_KEYBOARD_LL;
        // we pass a 'static extern "system" callback and no module handle (NULL is
        // permitted for low-level hooks per MSDN).
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_callback::keyboard_hook_callback),
                None,
                0,
            )?
        };
        self.hook_handle.store(hook.0, Ordering::Release);

        // Foreground-change hook re-syncs modifier state after focus returns from
        // a higher-integrity window (Task Manager, UAC, lock screen, etc.) that
        // ate our key-up events, except for modifiers whose key-down the engine
        // swallowed. See keyboard_hook_callback::foreground_changed.
        //
        // SAFETY: WINEVENT_OUTOFCONTEXT delivers events to this thread's message
        // queue, which the App message loop pumps. The callback is a 'static
        // extern "system" fn with no captured state.
        let we_hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(keyboard_hook_callback::foreground_changed),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if we_hook.is_invalid() {
            // The keyboard hook is already installed; tear it down to keep state
            // consistent with the failure being reported.
            let _ = self.uninstall();
            return Err(Error::new(
                windows::Win32::Foundation::E_FAIL,
                "SetWinEventHook failed for EVENT_SYSTEM_FOREGROUND",
            ));
        }
        self.foreground_hook.store(we_hook.0, Ordering::Release);

        Ok(())
    }

    /// True if the low-level keyboard hook is currently installed.
    pub fn is_installed(&self) -> bool {
        !self.hook_handle.load(Ordering::Acquire).is_null()
    }

    /// Removes both hooks. A foreground hook that cannot be removed is
    /// logged; the keyboard hook's failure is returned.
    ///
    /// Main thread only, like [`install`](Self::install):
    /// `UnhookWinEvent` fails on any other thread.
    pub(crate) fn uninstall(&self) -> Result<()> {
        let we_handle = self
            .foreground_hook
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !we_handle.is_null() {
            // SAFETY: the handle came from SetWinEventHook in `install`, and
            // the swap above took it out of the field, so it is unhooked at
            // most once.
            let removed = unsafe { UnhookWinEvent(HWINEVENTHOOK(we_handle)) };
            if let Err(error) = removed.ok() {
                eprintln!("[WARN] could not remove the foreground hook: {error}");
            }
        }

        let handle = self
            .hook_handle
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !handle.is_null() {
            // SAFETY: handle was obtained from SetWindowsHookExW above.
            unsafe {
                UnhookWindowsHookEx(HHOOK(handle))?;
            }
        }
        Ok(())
    }
}

impl Default for KeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.uninstall();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::key_spec::KeySpec;
    use crate::data::trigger::Trigger;
    use crate::engine::{Decision, KeyEvent, TimerToken};
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    fn three_bindings() -> Vec<Binding> {
        vec![
            Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::SwitchLanguage),
            Binding::new(
                KeyCombination::new(VK_CAPITAL).with(VK_SHIFT),
                BindAction::ToggleCapsLock,
            ),
            Binding::new(
                KeyCombination::new(VK_A).with(VK_CONTROL).with(VK_SHIFT),
                BindAction::DoNothing,
            ),
        ]
    }

    #[test]
    fn bindings_sorted_by_specificity() {
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());

        let bindings = hook.bindings();
        assert_eq!(bindings[0].kind.key_count(), 3);
        assert_eq!(bindings[1].kind.key_count(), 2);
        assert_eq!(bindings[2].kind.key_count(), 1);
    }

    /// 30 distinct single keys, in the order a config would list them.
    const SINGLE_KEYS: [VIRTUAL_KEY; 30] = [
        VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
        VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21, VK_F22, VK_F23,
        VK_F24, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5,
    ];

    #[test]
    fn equal_specificity_keeps_config_order() {
        // The sort must be stable: among bindings with the same key count,
        // the one listed first in the config is checked first (two bindings
        // on one key with different `when` conditions rely on this).
        //
        // The list must be longer than 20 and not already sorted: for 20 or
        // fewer elements `sort_unstable*` falls back to insertion sort, which
        // happens to be stable, so a short list cannot tell the two apart.
        // A 2-key binding in the middle forces the sort to move elements.
        // With `sort_unstable_by_key` in `update_bindings` this test fails.
        let mut config: Vec<Binding> = SINGLE_KEYS
            .iter()
            .map(|&vk| Binding::new(KeyCombination::new(vk), BindAction::DoNothing))
            .collect();
        config.insert(
            15,
            Binding::new(
                KeyCombination::new(VK_A).with(VK_CONTROL),
                BindAction::SwitchLanguage,
            ),
        );
        assert!(config.len() > 20);

        let hook = KeyboardHook::new();
        hook.update_bindings(config);

        let bindings = hook.bindings();
        assert_eq!(bindings[0].kind.key_count(), 2);
        let single_keys: Vec<_> = bindings[1..]
            .iter()
            .map(|b| b.combination().unwrap().keys[0])
            .collect();
        assert_eq!(single_keys, SINGLE_KEYS.map(KeySpec::from));
    }

    #[test]
    fn update_bindings_replaces_list() {
        // Sanity check that ArcSwap actually swaps — a second update with
        // fewer bindings must shrink the observable list, and the new sort
        // must be applied (not just appended).
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());
        assert_eq!(hook.bindings().len(), 3);

        hook.update_bindings(vec![Binding::new(
            KeyCombination::new(VK_F13),
            BindAction::SwitchLanguage,
        )]);
        let snapshot = hook.bindings();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(
            snapshot[0].combination().unwrap().keys,
            vec![KeySpec::from(VK_F13)]
        );
    }

    #[test]
    fn update_bindings_can_clear_to_empty() {
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());
        hook.update_bindings(vec![]);
        assert_eq!(hook.bindings().len(), 0);
    }

    #[test]
    fn initial_bindings_are_empty() {
        let hook = KeyboardHook::new();
        assert_eq!(hook.bindings().len(), 0);
    }

    #[test]
    fn with_engine_refuses_reentry_on_the_same_thread() {
        // On its own thread, so a broken guard fails the test after the
        // timeout instead of hanging the suite.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let hook = KeyboardHook::new();
            let nested = hook.with_engine(|_| hook.with_engine(|_| ()));
            let _ = tx.send(nested);
        });
        let nested = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("re-entry deadlocked");
        assert_eq!(nested, Some(None));
    }

    #[test]
    fn with_engine_works_again_after_a_call() {
        let hook = KeyboardHook::new();
        assert_eq!(hook.with_engine(|_| 1), Some(1));
        assert_eq!(hook.with_engine(|_| 2), Some(2));
    }

    #[test]
    fn toggling_reports_changes_and_never_installs_a_hook() {
        let hook = KeyboardHook::new();
        assert!(hook.is_enabled(), "remapping starts on");
        assert_eq!(hook.set_enabled(true), Some(false), "already on");
        assert!(!hook.is_installed());
        assert_eq!(hook.set_enabled(false), Some(true));
        assert!(!hook.is_enabled());
        assert!(!hook.is_installed());
        assert_eq!(hook.set_enabled(false), Some(false), "already off");
        assert!(!hook.is_installed());
        assert_eq!(hook.set_enabled(true), Some(true));
        assert!(hook.is_enabled());
        assert!(!hook.is_installed(), "a toggle never installs a hook");
    }

    /// An [`engine::Env`] stopped at one instant with nothing held and no
    /// foreground window.
    struct QuietEnv(Instant);

    impl engine::Env for QuietEnv {
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

    #[test]
    fn turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it() {
        // KS-31. No timer thread is started: the test hands the tokens back
        // to the engine itself, so nothing fires outside the assertions.
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_trigger(Trigger::Hold(200))
                .with_allow_extra_modifiers(true),
        ]);
        let t0 = Instant::now();
        let key = |event: KeyEvent, at: Duration| {
            hook.with_engine(|engine| engine.on_key(event, &hook.bindings(), &QuietEnv(t0 + at)))
                .expect("not inside with_engine")
        };
        let timer = |token: TimerToken| {
            hook.with_engine(|engine| engine.on_timer(token))
                .expect("not inside with_engine")
        };

        let first = key(KeyEvent::down(VK_F13), Duration::ZERO).timers[0].token;
        assert_eq!(hook.set_enabled(true), Some(false));
        assert_eq!(
            timer(first).fire,
            [BindAction::DoNothing],
            "a toggle that changes nothing keeps the pending Hold"
        );

        let _ = key(KeyEvent::up(VK_F13), Duration::from_millis(300));
        let second = key(KeyEvent::down(VK_F13), Duration::from_millis(400)).timers[0].token;
        assert_eq!(hook.set_enabled(false), Some(true));
        assert_eq!(
            timer(second),
            Decision::default(),
            "a Hold armed before remapping went off must not fire"
        );
    }

    #[test]
    fn set_enabled_refuses_a_nested_call() {
        // On its own thread, so a broken guard fails the test after the
        // timeout instead of hanging the suite.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let hook = KeyboardHook::new();
            let nested = hook.with_engine(|_| hook.set_enabled(false));
            let _ = tx.send((nested, hook.is_enabled()));
        });
        let (nested, enabled) = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("set_enabled deadlocked inside with_engine");
        assert_eq!(nested, Some(None));
        assert!(enabled, "a refused call leaves remapping on");
    }
}
