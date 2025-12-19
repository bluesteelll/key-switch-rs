use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU16, Ordering};
use windows::{
    core::*,
    Win32::UI::WindowsAndMessaging::*,
};

use crate::binding::Binding;
use crate::hook::keyboard_hook_callback;

const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);

pub struct KeyboardHook {
    pub bindings: Vec<Binding>,
    pub hook_handle: AtomicPtr<std::ffi::c_void>,
    pub blocked_key: AtomicU16,
    pub active_keys: [AtomicBool; 256],
}

impl KeyboardHook {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
            hook_handle: AtomicPtr::new(std::ptr::null_mut()),
            blocked_key: AtomicU16::new(0),
            active_keys: [ATOMIC_FALSE; 256],
        }
    }

    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.push(binding);
        // Sort by key count descending - more specific bindings match first
        self.bindings.sort_by(|a, b| b.combination.keys.len().cmp(&a.combination.keys.len()));
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn install(&mut self) -> Result<()> {
        unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_callback::keyboard_hook_callback),
                None,
                0,
            )?;

            self.hook_handle.store(hook.0, Ordering::Release);
        }

        Ok(())
    }

    pub fn uninstall(&mut self) -> Result<()> {
        let handle = self.hook_handle.swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !handle.is_null() {
            unsafe {
                UnhookWindowsHookEx(HHOOK(handle))?;
            }
        }
        Ok(())
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
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::core::windows_actions::BindAction;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn test_bindings_sorted_by_specificity() {
        let mut hook = KeyboardHook::new();

        hook.add_binding(Binding::new(
            KeyCombination::new(VK_CAPITAL),
            BindAction::SwitchLanguage,
        ));

        hook.add_binding(Binding::new(
            KeyCombination::new(VK_CAPITAL).with(VK_SHIFT),
            BindAction::ToggleCapsLock,
        ));

        hook.add_binding(Binding::new(
            KeyCombination::new(VK_A).with(VK_CONTROL).with(VK_SHIFT),
            BindAction::DoNothing,
        ));

        let bindings = hook.bindings();
        assert_eq!(bindings[0].combination.keys.len(), 3);
        assert_eq!(bindings[1].combination.keys.len(), 2);
        assert_eq!(bindings[2].combination.keys.len(), 1);
    }
}
