//! The calls that put the daemon's hooks and its raw input sink into the
//! system and take them out again (roadmap item 2.4, ruling S1).
//!
//! [`KeyboardHook`](crate::hook::keyboard_hook::KeyboardHook) installs,
//! reinstalls and removes its three hooks through an [`Installer`], and the
//! hook watchdog registers its raw input sink through one. The daemon's is
//! [`SYSTEM`]. Its Win32 calls sit in the `win32` module below, which
//! `#[cfg(not(test))]` keeps out of the test build: under test [`SYSTEM`]
//! panics at every call before anything reaches Windows, so no test can put
//! a global hook or a raw input sink on the machine that runs it, whatever a
//! guard in front of the call does or a mutation removes. A test that
//! exercises an install, a reinstall or a removal hands in a fake that
//! models the handles. `the_hook_installers_are_compiled_out_of_the_tests`
//! pins that the calls appear nowhere else.

use std::ffi::c_void;

use windows::Win32::{Foundation::HWND, UI::Accessibility::WINEVENTPROC};
use windows::core::Result;

/// A hook handle as `KeyboardHook` keeps it: the pointer of an `HHOOK` or of
/// an `HWINEVENTHOOK`.
pub(crate) type RawHandle = *mut c_void;

/// Puts the hooks and the raw input sink into the system and takes them out
/// again. The hooks are installed and removed on the main thread, which
/// pumps their messages (KS-01); the raw input sink on the thread that owns
/// its window.
pub(crate) trait Installer {
    /// Installs a `WH_KEYBOARD_LL` hook that calls `keyboard_hook_callback`,
    /// at the head of the hook chain.
    fn set_keyboard_hook(&self) -> Result<RawHandle>;

    /// Removes the keyboard hook `hook`. An error if Windows has no such
    /// hook, also for one Windows removed itself.
    fn unhook_keyboard(&self, hook: RawHandle) -> Result<()>;

    /// Installs an out-of-context WinEvent hook for `event` alone that skips
    /// this process's own events, or `None` if Windows refused it.
    fn set_win_event_hook(&self, event: u32, callback: WINEVENTPROC) -> Option<RawHandle>;

    /// Removes the WinEvent hook `hook`.
    fn unhook_win_event(&self, hook: RawHandle) -> Result<()>;

    /// Registers `sink` for the raw input of every keyboard, also while
    /// another process has the foreground (`RIDEV_INPUTSINK`).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "first used by Hook: the hook watchdog thread")
    )]
    fn register_raw_keyboard(&self, sink: HWND) -> Result<()>;

    /// Removes this process's raw keyboard registration.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "first used by Hook: the hook watchdog thread")
    )]
    fn unregister_raw_keyboard(&self) -> Result<()>;
}

/// The daemon's installer: the Win32 calls of the `win32` module outside
/// tests, a panic inside them.
pub(crate) struct System;

/// The installer the daemon uses.
pub(crate) const SYSTEM: System = System;

#[cfg(not(test))]
mod win32 {
    use windows::Win32::{
        Foundation::HWND,
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent, WINEVENTPROC},
            Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RIDEV_REMOVE, RegisterRawInputDevices},
            WindowsAndMessaging::{
                HHOOK, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
                WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
            },
        },
    };
    use windows::core::Result;

    use super::{Installer, RawHandle, System};
    use crate::hook::keyboard_hook_callback::keyboard_hook_callback;

    /// The HID usage page of the generic desktop controls and, on it, the
    /// keyboard's usage, which select every keyboard for raw input. The
    /// `windows` crate has them only under its
    /// `Win32_Devices_HumanInterfaceDevice` feature.
    #[expect(dead_code, reason = "first used by Hook: the hook watchdog thread")]
    const GENERIC_DESKTOP: u16 = 0x01;
    #[expect(dead_code, reason = "first used by Hook: the hook watchdog thread")]
    const KEYBOARD: u16 = 0x06;

    impl Installer for System {
        fn set_keyboard_hook(&self) -> Result<RawHandle> {
            // SAFETY: SetWindowsHookExW is the documented installer for
            // WH_KEYBOARD_LL; it gets a 'static extern "system" callback and
            // no module handle, which a low-level hook permits.
            let hook = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), None, 0)
            }?;
            Ok(hook.0)
        }

        fn unhook_keyboard(&self, hook: RawHandle) -> Result<()> {
            // SAFETY: `hook` came from SetWindowsHookExW in this process, and
            // the caller took it out of the field that kept it, so it is
            // removed at most once; a hook that Windows removed itself only
            // makes the call fail.
            unsafe { UnhookWindowsHookEx(HHOOK(hook)) }
        }

        fn set_win_event_hook(&self, event: u32, callback: WINEVENTPROC) -> Option<RawHandle> {
            // SAFETY: WINEVENT_OUTOFCONTEXT delivers the events to the
            // calling thread's message queue, which the main loop pumps, so
            // no module handle is needed; the callback is a 'static extern
            // "system" fn with no captured state.
            let hook = unsafe {
                SetWinEventHook(
                    event,
                    event,
                    None,
                    callback,
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                )
            };
            (!hook.is_invalid()).then_some(hook.0)
        }

        fn unhook_win_event(&self, hook: RawHandle) -> Result<()> {
            // SAFETY: `hook` came from SetWinEventHook in this process, and
            // the caller took it out of the field that kept it, so it is
            // removed at most once.
            unsafe { UnhookWinEvent(HWINEVENTHOOK(hook)) }.ok()
        }

        fn register_raw_keyboard(&self, sink: HWND) -> Result<()> {
            let device = RAWINPUTDEVICE {
                usUsagePage: GENERIC_DESKTOP,
                usUsage: KEYBOARD,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: sink,
            };
            // SAFETY: the slice holds one initialised RAWINPUTDEVICE on this
            // stack frame, alive for the whole call, which only reads it, and
            // the size passed is its size; RIDEV_INPUTSINK needs a target,
            // and `sink` is a live window of the calling thread.
            unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32) }
        }

        fn unregister_raw_keyboard(&self) -> Result<()> {
            // "If RIDEV_REMOVE is set and the hwndTarget member is not set to
            // NULL, then RegisterRawInputDevices function will fail"
            // (RAWINPUTDEVICE).
            let device = RAWINPUTDEVICE {
                usUsagePage: GENERIC_DESKTOP,
                usUsage: KEYBOARD,
                dwFlags: RIDEV_REMOVE,
                hwndTarget: HWND::default(),
            };
            // SAFETY: as for the registration: one initialised record on this
            // stack frame, alive for the call, and its size.
            unsafe { RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32) }
        }
    }
}

/// What every call of the system installer says under test.
#[cfg(test)]
const UNDER_TEST: &str =
    "a test reached the system installer, which no test build calls (ruling S1)";

/// Under test no call reaches Windows (ruling S1): a test that gets here
/// has a defect, and fails at the panic instead of putting a hook on the
/// machine that runs it.
#[cfg(test)]
impl Installer for System {
    fn set_keyboard_hook(&self) -> Result<RawHandle> {
        unreachable!("{UNDER_TEST}")
    }

    fn unhook_keyboard(&self, _hook: RawHandle) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn set_win_event_hook(&self, _event: u32, _callback: WINEVENTPROC) -> Option<RawHandle> {
        unreachable!("{UNDER_TEST}")
    }

    fn unhook_win_event(&self, _hook: RawHandle) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn register_raw_keyboard(&self, _sink: HWND) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn unregister_raw_keyboard(&self) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn the_system_installer_panics_at_every_call_under_test() {
        // Ruling S1: whatever a caller's guard does, the test build's
        // system installer stops the call before Windows sees it.
        let calls: [(&str, &dyn Fn()); 6] = [
            ("set_keyboard_hook", &|| drop(SYSTEM.set_keyboard_hook())),
            ("unhook_keyboard", &|| {
                drop(SYSTEM.unhook_keyboard(std::ptr::null_mut()))
            }),
            ("set_win_event_hook", &|| {
                let _ = SYSTEM.set_win_event_hook(0, None);
            }),
            ("unhook_win_event", &|| {
                drop(SYSTEM.unhook_win_event(std::ptr::null_mut()))
            }),
            ("register_raw_keyboard", &|| {
                drop(SYSTEM.register_raw_keyboard(HWND::default()))
            }),
            ("unregister_raw_keyboard", &|| {
                drop(SYSTEM.unregister_raw_keyboard())
            }),
        ];
        for (name, call) in calls {
            assert!(
                catch_unwind(AssertUnwindSafe(call)).is_err(),
                "{name} returned under test"
            );
        }
    }

    /// The Win32 calls that install or remove a global hook or a raw input
    /// sink. Assembled, so that this file does not match itself.
    const INSTALLERS: [&str; 5] = [
        concat!("SetWindows", "HookExW"),
        concat!("UnhookWindows", "HookEx"),
        concat!("SetWin", "EventHook"),
        concat!("UnhookWin", "Event"),
        concat!("RegisterRaw", "InputDevices"),
    ];

    /// The one place the calls may appear: the block of this module
    /// that opens with this line, right after `#[cfg(not(test))]`.
    const WIN32_BLOCK: &str = "mod win32 {";
    const INSTALLER_FILE: &str = "src/hook/installer.rs";

    /// Every `.rs` file under `dir`, recursively.
    fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("list a source folder") {
            let path = entry.expect("a folder entry").path();
            if path.is_dir() {
                rust_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    /// The index of the line that opens the tests module of a source file
    /// (`#[cfg(test)]` followed by `mod tests {`), or the line count.
    fn tests_start(lines: &[&str]) -> usize {
        lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len())
    }

    /// The code of `line`: without its `//` comment and without the text of
    /// its string literals, so that a message or a doc that names a call
    /// is not taken for the call.
    fn code_of(line: &str) -> String {
        let mut code = String::new();
        let mut in_string = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_string {
                match c {
                    '\\' => {
                        chars.next();
                    }
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '/' if chars.peek() == Some(&'/') => break,
                _ => code.push(c),
            }
        }
        code
    }

    /// The numbers (from 1) of the lines of the block that opens at the
    /// first line equal to `opening` once trimmed, right after a
    /// `#[cfg(not(test))]` line, until its braces balance; `None` if there
    /// is no such block.
    fn cfg_not_test_block(
        lines: &[&str],
        opening: &str,
    ) -> Option<std::ops::RangeInclusive<usize>> {
        let start = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(not(test))]" && pair[1].trim() == opening)?
            + 1;
        let mut depth = 0_i64;
        for (index, line) in lines.iter().enumerate().skip(start) {
            let code = code_of(line);
            depth += code.matches('{').count() as i64 - code.matches('}').count() as i64;
            if depth == 0 {
                return Some(start + 1..=index + 1);
            }
        }
        None
    }

    #[test]
    fn the_hook_installers_are_compiled_out_of_the_tests() {
        // Ruling S1 and S4 of roadmap item 2.4: every call that installs a
        // global hook or a raw input sink sits in the `win32` module of this
        // file, which `#[cfg(not(test))]` keeps out of the test build, so no
        // test can reach one, whatever a guard in front of it does. The
        // integration tests link the library as the daemon does, so none of
        // them may name a call, or the one public way to the hooks,
        // `core::app` (`App::run`).
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rust_files(&root.join("src"), &mut files);
        files.sort();
        assert!(files.len() > 40, "the scan found the sources: {files:?}");

        let mut stray = Vec::new();
        let mut inside = Vec::new();
        let mut block = None;
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            let lines: Vec<&str> = text.lines().collect();
            let allowed = if relative == INSTALLER_FILE {
                block = cfg_not_test_block(&lines, WIN32_BLOCK);
                block.clone()
            } else {
                None
            };
            for (index, line) in lines[..tests_start(&lines)].iter().enumerate() {
                let number = index + 1;
                let code = code_of(line);
                let calls: Vec<&str> = INSTALLERS
                    .into_iter()
                    .filter(|call| code.contains(call))
                    .collect();
                if allowed
                    .as_ref()
                    .is_some_and(|block| block.contains(&number))
                {
                    inside.extend(calls);
                } else if !calls.is_empty() {
                    stray.push(format!("{relative}:{number}: {}", line.trim()));
                }
            }
        }

        let mut integration = Vec::new();
        for entry in std::fs::read_dir(root.join("tests")).expect("list tests") {
            let path = entry.expect("a folder entry").path();
            let text = std::fs::read_to_string(&path).expect("read an integration test");
            for (index, line) in text.lines().enumerate() {
                let code = code_of(line);
                if INSTALLERS.iter().any(|call| code.contains(call)) || code.contains("core::app") {
                    integration.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
                }
            }
        }

        let mut report = String::new();
        if block.is_none() {
            report +=
                &format!("{INSTALLER_FILE} has no `#[cfg(not(test))]` block `{WIN32_BLOCK}`\n");
        }
        for call in INSTALLERS {
            if block.is_some() && !inside.contains(&call) {
                report += &format!(
                    "{call} is not called in the win32 block, so the scan proves nothing about it\n"
                );
            }
        }
        if !stray.is_empty() {
            report += &format!(
                "{} installer calls a test build compiles:\n  {}\n",
                stray.len(),
                stray.join("\n  ")
            );
        }
        if !integration.is_empty() {
            report += &format!(
                "integration tests that reach a hook:\n  {}\n",
                integration.join("\n  ")
            );
        }
        assert!(report.is_empty(), "{report}");
    }
}
