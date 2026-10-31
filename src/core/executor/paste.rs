//! The `Paste` action and macro step on the action executor (owner request
//! 2026-10-30, roadmap new feature 9).
//!
//! What a paste needs besides the clipboard (`platform::clipboard`) goes
//! through a [`Desk`]: the foreground window, whether a window answers,
//! whether a modifier key is down, and the paste keystroke. The daemon's is
//! the executor's `Win32Desk`, whose Win32 calls sit in the `win32` module
//! below, which `#[cfg(not(test))]` keeps out of the test build: under test
//! it panics at every call, so no test sends a key or probes a window of the
//! machine that runs it. The tests hand in a fake desk instead.

use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::TargetWindow;
use crate::data::sequence::ComboSent;
use crate::platform::clipboard::Clipboard;

/// What a paste reads from and sends to the desktop, besides the
/// clipboard. Every method runs on the action executor.
pub trait Desk {
    /// The foreground window now, or `NONE`.
    fn foreground(&self) -> TargetWindow;
    /// Whether the thread of `window` answers a `WM_NULL` within `within`.
    fn responds(&self, window: isize, within: Duration) -> Probe;
    /// Whether Windows has a Shift, Ctrl, Alt or Win key down, either one
    /// of each pair.
    fn modifier_down(&self) -> bool;
    /// Sends the paste keystroke `keys` in one batch (`press_combo`), each
    /// event with the sentinel.
    fn press(&self, keys: &[VIRTUAL_KEY]) -> ComboSent;
    /// Sends the key-ups a batch of `keys` cut short after `sent` events
    /// still owes, and returns how many went out.
    fn release_after(&self, keys: &[VIRTUAL_KEY], sent: u32) -> u32;
}

/// What a [`Desk::responds`] probe found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// The window's thread answered at once.
    Answered,
    /// It answered, but only after a noticeable wait: it was busy, and the
    /// input queued for it is handled only now (cut critique M1).
    Late,
    /// It did not answer in time, or it looks hung.
    NoAnswer,
    /// The window no longer exists (cut critique I1): nothing will read
    /// the clipboard for it.
    Gone,
}

/// The clipboard and the desk a paste works through.
pub struct Ports<'a> {
    pub clipboard: &'a dyn Clipboard,
    pub desk: &'a dyn Desk,
}

/// The daemon's desk: the Win32 calls of the `win32` module outside tests,
/// a panic inside them. Crate-private, so that no doctest or integration
/// test can reach a keystroke through it (cut critique M6).
pub(crate) struct Win32Desk;

/// The desk the daemon uses.
pub(crate) const WIN32_DESK: Win32Desk = Win32Desk;

#[cfg(not(test))]
mod win32 {
    use std::time::{Duration, Instant};

    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::{
                IsWindow, SMTO_ABORTIFHUNG, SMTO_BLOCK, SendMessageTimeoutW, WM_NULL,
            },
        },
    };

    use super::{Desk, Probe, Win32Desk};
    use crate::core::executor::TargetWindow;
    use crate::data::sequence::{self, ComboSent};

    /// The modifier keys whose state the wait reads: both keys of each
    /// pair, as Windows reports them.
    const SIDED_MODIFIERS: [VIRTUAL_KEY; 8] = [
        VK_LSHIFT,
        VK_RSHIFT,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_LMENU,
        VK_RMENU,
        VK_LWIN,
        VK_RWIN,
    ];

    /// An answer that took at least this long counts as a busy thread's: a
    /// responsive one answers a `WM_NULL` in well under a millisecond.
    const LATE_ANSWER: Duration = Duration::from_millis(5);

    fn exists(window: HWND) -> bool {
        // SAFETY: IsWindow takes the handle by value and reads no memory of
        // ours; a handle that names no window gives FALSE.
        unsafe { IsWindow(Some(window)) }.as_bool()
    }

    impl Desk for Win32Desk {
        fn foreground(&self) -> TargetWindow {
            TargetWindow::foreground()
        }

        fn responds(&self, window: isize, within: Duration) -> Probe {
            let window = HWND(window as *mut core::ffi::c_void);
            if !exists(window) {
                return Probe::Gone;
            }
            let timeout = u32::try_from(within.as_millis()).unwrap_or(u32::MAX);
            let begun = Instant::now();
            // SAFETY: WM_NULL carries no pointer and no result is asked for,
            // so the call reads and writes no memory of ours; SMTO_BLOCK
            // keeps this thread from handling other sent messages while it
            // waits, SMTO_ABORTIFHUNG returns at once for a thread Windows
            // takes for hung, and a stale handle only makes it fail.
            let answered = unsafe {
                SendMessageTimeoutW(
                    window,
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG | SMTO_BLOCK,
                    timeout,
                    None,
                )
            };
            if answered.0 == 0 {
                // "If the function fails or times out, the return value is
                // 0": a window that went away meanwhile is gone, not hung.
                return if exists(window) {
                    Probe::NoAnswer
                } else {
                    Probe::Gone
                };
            }
            if begun.elapsed() >= LATE_ANSWER {
                Probe::Late
            } else {
                Probe::Answered
            }
        }

        fn modifier_down(&self) -> bool {
            SIDED_MODIFIERS.iter().any(|&key| {
                // SAFETY: GetAsyncKeyState takes the key code by value,
                // writes no memory of ours and may be called from any
                // thread; a failure (another desktop, UIPI) reads as up.
                let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
                (state as u16) & 0x8000 != 0
            })
        }

        fn press(&self, keys: &[VIRTUAL_KEY]) -> ComboSent {
            sequence::press_combo(keys)
        }

        fn release_after(&self, keys: &[VIRTUAL_KEY], sent: u32) -> u32 {
            sequence::release_unreleased(keys, sent)
        }
    }
}

/// What every call of the daemon's desk says under test.
#[cfg(test)]
const UNDER_TEST: &str =
    "a test reached the daemon's desk, which no test build calls (lane feat-clipboard-paste)";

/// Under test no call reaches Windows: a test that gets here has a defect,
/// and fails at the panic instead of pressing a key on the machine that
/// runs it.
#[cfg(test)]
impl Desk for Win32Desk {
    fn foreground(&self) -> TargetWindow {
        unreachable!("{UNDER_TEST}")
    }

    fn responds(&self, _window: isize, _within: Duration) -> Probe {
        unreachable!("{UNDER_TEST}")
    }

    fn modifier_down(&self) -> bool {
        unreachable!("{UNDER_TEST}")
    }

    fn press(&self, _keys: &[VIRTUAL_KEY]) -> ComboSent {
        unreachable!("{UNDER_TEST}")
    }

    fn release_after(&self, _keys: &[VIRTUAL_KEY], _sent: u32) -> u32 {
        unreachable!("{UNDER_TEST}")
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;

    #[test]
    fn the_win32_desk_panics_at_every_call_under_test() {
        let calls: [(&str, &dyn Fn()); 5] = [
            ("foreground", &|| {
                let _ = WIN32_DESK.foreground();
            }),
            ("responds", &|| {
                let _ = WIN32_DESK.responds(0x1234, Duration::from_millis(1));
            }),
            ("modifier_down", &|| {
                let _ = WIN32_DESK.modifier_down();
            }),
            ("press", &|| {
                let _ = WIN32_DESK.press(&[VK_CONTROL, VK_V]);
            }),
            ("release_after", &|| {
                let _ = WIN32_DESK.release_after(&[VK_CONTROL, VK_V], 1);
            }),
        ];
        for (name, call) in calls {
            assert!(
                catch_unwind(AssertUnwindSafe(call)).is_err(),
                "{name} returned under test"
            );
        }
    }
}
