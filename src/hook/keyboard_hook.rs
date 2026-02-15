use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};

use arc_swap::{ArcSwap, Guard};
use windows::{
    core::*,
    Win32::UI::{
        Accessibility::SetWinEventHook,
        WindowsAndMessaging::*,
    },
};

use crate::data::binding::Binding;
use crate::hook::chord_state::ChordState;
use crate::hook::keyboard_hook_callback;
use crate::hook::sequence_state::SequenceState;
use crate::hook::tap_state::TapState;

#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);

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
    /// Bitmap of keys whose key-down was blocked: the matching key-up must also
    /// be swallowed. One bit per virtual-key code (0..256), packed into four
    /// 64-bit atomics. Replaces an earlier single-key field that lost track of
    /// every-but-the-most-recent block.
    pub blocked_keys: [AtomicU64; 4],
    pub active_keys: [AtomicBool; 256],
    /// Pending Tap / Hold / DoubleTap gestures. See [`TapState`].
    pub tap_state: TapState,
    /// Pending simultaneous-chord gestures (`BindingKind::Chord`).
    pub chord_state: ChordState,
    /// Pending leader-sequence gestures (`BindingKind::Sequence`).
    pub sequence_state: SequenceState,
}

impl KeyboardHook {
    /// Constructs an empty hook. Not `const` because `ArcSwap::from_pointee`
    /// requires a heap allocation; use a `LazyLock` for static initialization.
    pub fn new() -> Self {
        Self {
            bindings: ArcSwap::from_pointee(Vec::new()),
            hook_handle: AtomicPtr::new(std::ptr::null_mut()),
            foreground_hook: AtomicPtr::new(std::ptr::null_mut()),
            blocked_keys: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            active_keys: [ATOMIC_FALSE; 256],
            tap_state: TapState::new(),
            chord_state: ChordState::new(),
            sequence_state: SequenceState::new(),
        }
    }

    /// Atomically replace the binding list. Callable any number of times
    /// (this is the hot-reload entry point). The new list is sorted by
    /// key-count descending so more specific combinations are checked first
    /// in the callback.
    pub fn update_bindings(&self, mut bindings: Vec<Binding>) {
        bindings.sort_by(|a, b| b.kind.key_count().cmp(&a.kind.key_count()));
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
        let we_handle = self.foreground_hook.swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !we_handle.is_null() {
            // SAFETY: handle was obtained from SetWinEventHook above.
            unsafe {
                let _ = windows::Win32::UI::Accessibility::UnhookWinEvent(
                    windows::Win32::UI::Accessibility::HWINEVENTHOOK(we_handle),
                );
            }
        }

        let handle = self.hook_handle.swap(std::ptr::null_mut(), Ordering::AcqRel);
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
            Binding::new(
                KeyCombination::new(VK_CAPITAL),
                BindAction::SwitchLanguage,
            ),
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
}
