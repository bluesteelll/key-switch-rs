use std::path::PathBuf;

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

use crate::core::constants::injected_sentinel;
use crate::data::key_combination::KeyCombination;
use crate::system::registry;

#[allow(dead_code)] // Variants are part of the public surface; not every binary uses them all.
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

    pub fn get_system_combination(self) -> Option<KeyCombination> {
        registry::get_system_hotkey(self)
    }

    fn switch_language_forward() {
        // SAFETY: GetForegroundWindow + PostMessageW require only a valid HWND,
        // which we validate.
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
        // SAFETY: same as switch_language_forward.
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
        let exe = system32_path("rundll32.exe");
        if let Err(e) = Command::new(&exe).args(["user32.dll,LockWorkStation"]).spawn() {
            eprintln!("LockWorkStation spawn error ({}): {}", exe.display(), e);
        }
    }

    fn show_desktop() {
        // SAFETY: FindWindowW + PostMessageW take valid arguments only.
        unsafe {
            let hwnd = match FindWindowW(w!("Shell_TrayWnd"), None) {
                Ok(h) if !h.is_invalid() => h,
                _ => {
                    eprintln!("[WARN] Shell_TrayWnd not found - cannot show desktop");
                    return;
                }
            };

            if let Err(e) = PostMessageW(Some(hwnd), WM_COMMAND, WPARAM(419), LPARAM(0)) {
                eprintln!("Show desktop error: {:?}", e);
            }
        }
    }

    fn open_task_manager() {
        use std::process::Command;
        let exe = system32_path("Taskmgr.exe");
        if let Err(e) = Command::new(&exe).spawn() {
            eprintln!("Taskmgr spawn error ({}): {}", exe.display(), e);
        }
    }

    fn toggle_caps_lock() {
        Self::press_key(VK_CAPITAL);
    }

    fn press_key(vk: VIRTUAL_KEY) {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: injected_sentinel(),
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
                        dwExtraInfo: injected_sentinel(),
                    },
                },
            },
        ];

        // SAFETY: inputs is a valid stack array; SendInput reads via the size
        // we pass as cbSize.
        let sent = unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32)
        };
        if sent as usize != inputs.len() {
            eprintln!("SendInput dropped events for vk {:?}: sent {} of {}", vk, sent, inputs.len());
        }
    }
}

/// Resolve a path under `%SystemRoot%\System32`, falling back to `C:\Windows\System32`
/// when the environment variable is missing. Path-qualifying these launches
/// prevents PATH-hijack attacks where an attacker plants a same-named binary
/// earlier in the PATH.
fn system32_path(exe_name: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let mut p = PathBuf::from(root);
    p.push("System32");
    p.push(exe_name);
    p
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

    #[test]
    fn system32_path_includes_system32() {
        let p = system32_path("Taskmgr.exe");
        let s = p.to_string_lossy().to_lowercase();
        assert!(s.ends_with("system32\\taskmgr.exe") || s.ends_with("system32/taskmgr.exe"));
    }
}
