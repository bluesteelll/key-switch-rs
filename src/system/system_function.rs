use std::path::PathBuf;

use windows::{
    Win32::{
        Foundation::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::*,
};

use crate::core::constants::injected_sentinel;
use crate::data::key_combination::KeyCombination;
use crate::logging::counters::{self, Counter};
use crate::platform::focus;
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

    /// Posts a "forward" `WM_INPUTLANGCHANGEREQUEST` with lParam 0 through
    /// [`post_to_foreground`]. Runs on the hook thread, so a failed post is
    /// only counted (`post_message_failed`), never logged.
    fn switch_language_forward() {
        post_to_foreground(
            WM_INPUTLANGCHANGEREQUEST,
            INPUTLANGCHANGE_FORWARD as usize,
            0,
        );
    }

    /// As [`switch_language_forward`](Self::switch_language_forward), with
    /// a "backward" request.
    fn switch_language_backward() {
        post_to_foreground(
            WM_INPUTLANGCHANGEREQUEST,
            INPUTLANGCHANGE_BACKWARD as usize,
            0,
        );
    }

    fn lock_workstation() {
        use std::process::Command;
        let exe = system32_path("rundll32.exe");
        if let Err(e) = Command::new(&exe)
            .args(["user32.dll,LockWorkStation"])
            .spawn()
        {
            log::error!("LockWorkStation spawn error ({}): {e}", exe.display());
        }
    }

    fn show_desktop() {
        // SAFETY: FindWindowW + PostMessageW take valid arguments only.
        unsafe {
            let hwnd = match FindWindowW(w!("Shell_TrayWnd"), None) {
                Ok(h) if !h.is_invalid() => h,
                _ => {
                    log::warn!("Shell_TrayWnd not found - cannot show desktop");
                    return;
                }
            };

            if let Err(e) = PostMessageW(Some(hwnd), WM_COMMAND, WPARAM(419), LPARAM(0)) {
                log::error!("Show desktop error: {e:?}");
            }
        }
    }

    fn open_task_manager() {
        use std::process::Command;
        let exe = system32_path("Taskmgr.exe");
        if let Err(e) = Command::new(&exe).spawn() {
            log::error!("Taskmgr spawn error ({}): {e}", exe.display());
        }
    }

    fn toggle_caps_lock() {
        Self::press_key(VK_CAPITAL);
    }

    /// A key press (down + up). It can run on the hook thread, so a short
    /// count is only counted (`key_press_short`), never logged.
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
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            counters::bump(Counter::KeyPressShort);
        }
    }
}

/// The window that a message posted to the foreground window goes to, or
/// `None` if there is no foreground window. Every message goes to
/// `foreground` itself. `is_dialog` and `focus_of` are the reads a choice
/// may need, asked only when it needs them.
pub(crate) fn post_target(
    msg: u32,
    foreground: HWND,
    is_dialog: impl FnOnce(HWND) -> bool,
    focus_of: impl FnOnce(HWND) -> Result<Option<HWND>>,
) -> Option<HWND> {
    if foreground.is_invalid() {
        return None;
    }
    let _ = (msg, is_dialog, focus_of);
    Some(foreground)
}

/// Posts `msg` once to the window [`post_target`] picks from the foreground
/// window, or nothing if there is none: the one post of `SwitchLanguage`,
/// `SwitchLanguageBackward` and `PostMessage`. It runs on the hook thread,
/// so a failed post is only counted (`post_message_failed`), never logged.
pub(crate) fn post_to_foreground(msg: u32, wparam: usize, lparam: isize) {
    // SAFETY: GetForegroundWindow takes no arguments and reads no memory of
    // ours; with no foreground window it returns a null handle.
    let foreground = unsafe { GetForegroundWindow() };
    let Some(target) = post_target(msg, foreground, focus::is_dialog, focus::focus_window_of)
    else {
        return;
    };
    // SAFETY: PostMessageW takes the window, the message and its parameters
    // by value and reads no memory of ours; a handle that names no window,
    // or one that went away since it was read, makes it fail, which is
    // counted.
    let posted = unsafe { PostMessageW(Some(target), msg, WPARAM(wparam), LPARAM(lparam)) };
    counters::bump_if_err(Counter::PostMessageFailed, &posted);
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
        assert!(
            SystemFunction::LockWorkstation
                .get_system_combination()
                .is_some()
        );
        assert!(
            SystemFunction::ShowDesktop
                .get_system_combination()
                .is_some()
        );
        assert!(
            SystemFunction::TaskManager
                .get_system_combination()
                .is_some()
        );
    }

    #[test]
    fn test_capslock_has_no_combination() {
        assert!(
            SystemFunction::ToggleCapsLock
                .get_system_combination()
                .is_none()
        );
    }

    #[test]
    fn system32_path_includes_system32() {
        let p = system32_path("Taskmgr.exe");
        let s = p.to_string_lossy().to_lowercase();
        assert!(s.ends_with("system32\\taskmgr.exe") || s.ends_with("system32/taskmgr.exe"));
    }

    /// A window handle for the selection tests. No Win32 call ever sees it.
    fn window(raw: usize) -> HWND {
        HWND(raw as *mut std::ffi::c_void)
    }

    #[test]
    fn no_foreground_window_gets_nothing_and_nothing_is_asked() {
        let target = post_target(
            WM_INPUTLANGCHANGEREQUEST,
            HWND::default(),
            |_| panic!("the class was asked with no foreground window"),
            |_| panic!("the focus was asked with no foreground window"),
        );
        assert_eq!(target, None);
    }

    #[test]
    fn any_other_message_goes_to_the_foreground_and_nothing_is_asked() {
        for msg in [WM_CLOSE, WM_SYSCOMMAND] {
            let target = post_target(
                msg,
                window(1),
                |_| panic!("the class was asked for message {msg:#06x}"),
                |_| panic!("the focus was asked for message {msg:#06x}"),
            );
            assert_eq!(target, Some(window(1)), "message {msg:#06x}");
        }
    }

    #[test]
    fn a_language_request_outside_a_dialog_goes_to_the_foreground() {
        let target = post_target(
            WM_INPUTLANGCHANGEREQUEST,
            window(1),
            |asked| {
                assert_eq!(asked, window(1));
                false
            },
            |_| panic!("the focus was asked outside a dialog"),
        );
        assert_eq!(target, Some(window(1)));
    }

    #[test]
    fn a_dialog_with_no_focus_window_gets_the_request_itself() {
        let target = post_target(
            WM_INPUTLANGCHANGEREQUEST,
            window(1),
            |_| true,
            |asked| {
                assert_eq!(asked, window(1));
                Ok(None)
            },
        );
        assert_eq!(target, Some(window(1)));
    }

    #[test]
    fn a_dialog_whose_focus_cannot_be_read_gets_the_request_itself() {
        let target = post_target(
            WM_INPUTLANGCHANGEREQUEST,
            window(1),
            |_| true,
            |asked| {
                assert_eq!(asked, window(1));
                Err(Error::from_hresult(HRESULT::from_win32(
                    ERROR_INVALID_THREAD_ID.0,
                )))
            },
        );
        assert_eq!(target, Some(window(1)));
    }

    /// The non-test lines of `file`, before its `#[cfg(test)] mod tests {`.
    fn non_test_lines(file: &str) -> Vec<String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(root.join(file)).expect("read a source file");
        let lines: Vec<&str> = text.lines().collect();
        let end = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len());
        lines[..end].iter().map(|line| (*line).to_owned()).collect()
    }

    /// The text of the first function of `file` whose line contains
    /// `signature`, from that line until its braces balance, outside the
    /// tests module. Panics if there is none, so that a renamed function
    /// fails the scan instead of passing it.
    fn function_text(file: &str, signature: &str) -> String {
        let lines = non_test_lines(file);
        let start = lines
            .iter()
            .position(|line| line.contains(signature))
            .unwrap_or_else(|| panic!("no `{signature}` in {file}"));
        let mut depth = 0_i64;
        for (index, line) in lines[start..].iter().enumerate() {
            depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
            if depth == 0 && line.contains('}') {
                return lines[start..=start + index].join("\n");
            }
        }
        panic!("`{signature}` in {file} does not end before the tests module");
    }

    /// The wiring of the fix (owner report 2026-10-30), which the selection
    /// tests do not see: both language actions and the `PostMessage` action
    /// post through `post_to_foreground`, which hands `post_target` the
    /// real reads and posts once. The `PostMessage` arm of `FastAction::run`
    /// is compiled out under test, so only its source can be checked.
    #[test]
    fn every_language_request_and_every_posted_message_goes_through_post_to_foreground() {
        let post = concat!("PostMessage", "W(");
        let this = "src/system/system_function.rs";
        for (signature, direction) in [
            ("fn switch_language_forward(", "INPUTLANGCHANGE_FORWARD"),
            ("fn switch_language_backward(", "INPUTLANGCHANGE_BACKWARD"),
        ] {
            let body = function_text(this, signature);
            for wired in [
                "post_to_foreground(",
                "WM_INPUTLANGCHANGEREQUEST",
                direction,
            ] {
                assert!(body.contains(wired), "{signature} has no {wired}:\n{body}");
            }
            assert!(!body.contains(post), "{signature} posts by itself:\n{body}");
        }

        let body = function_text(this, "fn post_to_foreground(");
        for wired in ["post_target(", "focus::is_dialog", "focus::focus_window_of"] {
            assert!(
                body.contains(wired),
                "post_to_foreground has no {wired}:\n{body}"
            );
        }
        assert_eq!(
            body.matches(post).count(),
            1,
            "post_to_foreground posts exactly once:\n{body}"
        );

        let actions = "src/core/windows_actions.rs";
        let body = function_text(actions, "fn run(");
        assert!(
            body.contains("system_function::post_to_foreground(msg, wparam, lparam)"),
            "the PostMessage arm of FastAction::run does not post through post_to_foreground:\n{body}"
        );
        let posts: Vec<String> = non_test_lines(actions)
            .into_iter()
            .filter(|line| line.contains(post))
            .collect();
        assert!(posts.is_empty(), "{actions} posts by itself: {posts:?}");
    }
}
