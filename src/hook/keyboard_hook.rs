use std::cell::Cell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use arc_swap::{ArcSwap, Guard};
use windows::{
    Win32::UI::{Accessibility::SetWinEventHook, WindowsAndMessaging::*},
    core::*,
};

use crate::data::binding::Binding;
use crate::engine::{self, Engine};
use crate::hook::keyboard_hook_callback;

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
    pub hook_handle: AtomicPtr<std::ffi::c_void>,
    pub foreground_hook: AtomicPtr<std::ffi::c_void>,
    /// The decision core: held and blocked keys and every pending gesture.
    /// Behind a lock until roadmap item 2.2 moves the Hold timers onto the
    /// hook thread, because a `hold-timer` thread must reach it too. Only
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
    /// `f` must only decide: no action, `SendInput` or thread spawn under the
    /// lock. A lock poisoned by an earlier panic is taken over as it is: the
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

    pub fn install(&self) -> Result<()> {
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
        // ate our key-up events. See keyboard_hook_callback::foreground_changed.
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

    /// Idempotent enable: installs the hook if it isn't already. Used by
    /// the IPC `on` command — being already-on is not an error.
    pub fn enable(&self) -> Result<()> {
        if self.is_installed() {
            return Ok(());
        }
        self.install()
    }

    /// Idempotent disable: uninstalls if installed. Used by the IPC `off`
    /// command — being already-off is not an error.
    pub fn disable(&self) -> Result<()> {
        if !self.is_installed() {
            return Ok(());
        }
        self.uninstall()
    }

    pub fn uninstall(&self) -> Result<()> {
        let we_handle = self
            .foreground_hook
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !we_handle.is_null() {
            // SAFETY: handle was obtained from SetWinEventHook above.
            unsafe {
                let _ = windows::Win32::UI::Accessibility::UnhookWinEvent(
                    windows::Win32::UI::Accessibility::HWINEVENTHOOK(we_handle),
                );
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
    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
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
        assert_eq!(single_keys, SINGLE_KEYS);
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
        assert_eq!(snapshot[0].combination().unwrap().keys, vec![VK_F13]);
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
}
