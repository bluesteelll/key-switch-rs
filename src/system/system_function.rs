use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::*,
        },
    },
};

use crate::data::key_combination::KeyCombination;
use crate::system::registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemFunction {
    SwitchLanguage,
    SwitchLanguageBackward,
    LockWorkstation,
    ShowDesktop,
    TaskManager,
    ToggleCapsLock,
}

impl SystemFunction {
    pub fn execute(&self) {
        match self {
            SystemFunction::SwitchLanguage => Self::switch_language_forward(),
            SystemFunction::SwitchLanguageBackward => Self::switch_language_backward(),
            SystemFunction::LockWorkstation => Self::lock_workstation(),
            SystemFunction::ShowDesktop => Self::show_desktop(),
            SystemFunction::TaskManager => Self::open_task_manager(),
            SystemFunction::ToggleCapsLock => Self::toggle_caps_lock(),
        }
    }

    pub fn get_system_combinations(&self) -> Vec<KeyCombination> {
        registry::get_system_hotkey(*self)
    }

    pub fn get_system_combination(&self) -> Option<KeyCombination> {
        self.get_system_combinations().into_iter().next()
    }

    fn switch_language_forward() {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                return;
            }

            if let Err(e) = PostMessageW(
                Some(hwnd),
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(INPUTLANGCHANGE_FORWARD as usize),
                LPARAM(0),
            ) {
                eprintln!("Language switch error: {:?}", e);
            }
        }
    }

    fn switch_language_backward() {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                return;
            }

            if let Err(e) = PostMessageW(
                Some(hwnd),
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(INPUTLANGCHANGE_BACKWARD as usize),
                LPARAM(0),
            ) {
                eprintln!("Language switch backward error: {:?}", e);
            }
        }
    }

    fn lock_workstation() {
        use std::process::Command;
        let _ = Command::new("rundll32.exe")
            .args(&["user32.dll,LockWorkStation"])
            .spawn();
    }

    fn show_desktop() {
        unsafe {
            if let Ok(hwnd) = FindWindowW(w!("Shell_TrayWnd"), None) {
                if !hwnd.is_invalid() {
                    if let Err(e) = PostMessageW(
                        Some(hwnd),
                        WM_COMMAND,
                        WPARAM(419),
                        LPARAM(0),
                    ) {
                        eprintln!("Show desktop error: {:?}", e);
                    }
                }
            }
        }
    }

    fn open_task_manager() {
        use std::process::Command;
        let _ = Command::new("taskmgr.exe").spawn();
    }

    fn toggle_caps_lock() {
        Self::press_key(VK_CAPITAL);
    }

    fn press_key(vk: VIRTUAL_KEY) {
        const INJECTED_EXTRA_INFO: usize = 0xDEADBEEF;

        let mut inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: INJECTED_EXTRA_INFO,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: INJECTED_EXTRA_INFO,
                    },
                },
            },
        ];

        unsafe {
            SendInput(&mut inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardcoded_combinations() {
        assert!(SystemFunction::LockWorkstation.get_system_combination().is_some());
        assert!(SystemFunction::ShowDesktop.get_system_combination().is_some());
        assert!(SystemFunction::TaskManager.get_system_combination().is_some());
    }

    #[test]
    fn test_capslock_has_no_combination() {
        assert!(SystemFunction::ToggleCapsLock.get_system_combination().is_none());
    }
}
