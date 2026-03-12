//! Sequence steps: the macro-like body of a `BindAction::Sequence`.
//!
//! A sequence runs in its own spawned thread because a low-level keyboard
//! hook callback must return *fast* — blocking it for any meaningful sleep
//! would freeze input system-wide. The hook only enqueues the work and lets
//! the worker thread do the typing/delays/window ops.
//!
//! The target window (`GetForegroundWindow()` at the moment the combo
//! matched) is captured once and reused for every window-related step, so
//! "restore this window … type … minimize this window" operates on the same
//! HWND throughout — not on whatever happened to be focused later.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use windows::Win32::{
    Foundation::*,
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::*,
    },
};

use crate::core::constants::injected_sentinel;
use crate::core::windows_actions::{launch_process, shell_open, MediaKey};
use crate::data::vk_name::vk_name;

#[derive(Debug, Clone)]
pub enum SequenceStep {
    /// Type a Unicode string via `SendInput` with `KEYEVENTF_UNICODE`. This
    /// is layout-independent — characters appear regardless of which IME or
    /// keyboard layout the foreground window is using.
    TypeText(String),

    /// Synthesize a single virtual-key press (down then up). No modifiers.
    PressKey(VIRTUAL_KEY),

    /// Synthesize a chord: modifiers down, key down, key up, modifiers up.
    /// The split between modifiers and "the actual key" is automatic — every
    /// known modifier vkey (Shift/Ctrl/Alt/Win, sided variants) is held
    /// during the press, everything else is the chord's payload key(s).
    PressCombo(Vec<VIRTUAL_KEY>),

    /// Sleep for `ms` milliseconds before the next step. Lives in the worker
    /// thread, never in the hook callback.
    Delay(u64),

    /// Apply a `ShowWindow`/`PostMessage` operation to the captured target
    /// window.
    Window(WindowOp),

    /// Spawn a process. Same as the top-level `Launch` action.
    Launch { exe: String, args: Vec<String> },

    /// Hand a URL / mailto: / file path to the OS default handler. Same as
    /// the top-level `OpenUrl` action.
    OpenUrl(String),

    /// Synthesize a media / volume key. Same as the top-level `Media` action.
    Media(MediaKey),
}

#[derive(Debug, Clone, Copy)]
pub enum WindowOp {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl std::fmt::Display for SequenceStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequenceStep::TypeText(s) => {
                // Truncate long text in the startup banner so the line stays readable.
                const MAX: usize = 32;
                if s.chars().count() > MAX {
                    let head: String = s.chars().take(MAX).collect();
                    write!(f, "text {:?}…", head)
                } else {
                    write!(f, "text {:?}", s)
                }
            }
            SequenceStep::PressKey(vk) => write!(f, "key {}", vk_name(*vk)),
            SequenceStep::PressCombo(keys) => {
                let names: Vec<String> = keys.iter().map(|k| vk_name(*k)).collect();
                write!(f, "combo {}", names.join("+"))
            }
            SequenceStep::Delay(ms) => write!(f, "delay {}ms", ms),
            SequenceStep::Window(op) => write!(f, "window {:?}", op),
            SequenceStep::Launch { exe, args } => {
                if args.is_empty() {
                    write!(f, "launch {}", exe)
                } else {
                    write!(f, "launch {} {}", exe, args.join(" "))
                }
            }
            SequenceStep::OpenUrl(url) => write!(f, "open {}", url),
            SequenceStep::Media(key) => write!(f, "media {:?}", key),
        }
    }
}

/// Spawns a worker thread that executes the given steps in order against
/// the foreground window captured *now*. Returns immediately — the caller
/// (a hook callback) must not block.
///
/// `steps` is shared with the worker via `Arc` (no deep clone). `HWND` is
/// `*mut c_void` and therefore not `Send`, so we ferry it across the thread
/// boundary as an `isize` and rebuild it on arrival — equivalent to passing
/// a window handle through any other integer-bearing channel (PostMessage's
/// thread-message queue, registry, etc.).
pub fn spawn_sequence(steps: Arc<Vec<SequenceStep>>) {
    // SAFETY: GetForegroundWindow has no preconditions; an invalid/null HWND
    // is handled by per-step checks in `apply_window_op`.
    let target_raw: isize = unsafe { GetForegroundWindow().0 as isize };

    thread::spawn(move || {
        let target = HWND(target_raw as *mut std::ffi::c_void);
        for step in steps.iter() {
            execute_step(step, target);
        }
    });
}

fn execute_step(step: &SequenceStep, target: HWND) {
    match step {
        SequenceStep::TypeText(s) => type_text(s),
        SequenceStep::PressKey(vk) => press_key(*vk),
        SequenceStep::PressCombo(keys) => press_combo(keys),
        SequenceStep::Delay(ms) => thread::sleep(Duration::from_millis(*ms)),
        SequenceStep::Window(op) => apply_window_op(*op, target),
        SequenceStep::Launch { exe, args } => launch_process(exe, args),
        SequenceStep::OpenUrl(url) => shell_open(url),
        SequenceStep::Media(key) => press_key(key.as_vk()),
    }
}

/// Emits each UTF-16 code unit as a Unicode key-down/key-up pair. Surrogate
/// pairs (anything outside the BMP — most emoji, less common CJK) are sent
/// as two separate code units; Windows recombines them into a single
/// character when delivering to the target application.
fn type_text(s: &str) {
    // Two INPUTs per code unit (down + up). UTF-16 length is the right
    // capacity bound because that's the unit SendInput's wScan field takes.
    let cap = s.encode_utf16().count().saturating_mul(2);
    if cap == 0 {
        return;
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(cap);
    for code in s.encode_utf16() {
        inputs.push(unicode_input(code, KEYBD_EVENT_FLAGS(0)));
        inputs.push(unicode_input(code, KEYEVENTF_KEYUP));
    }

    // SAFETY: inputs is a contiguous stack-equivalent buffer; SendInput reads
    // exactly `cbSize` bytes per entry.
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if (sent as usize) != inputs.len() {
        eprintln!(
            "[WARN] type_text: SendInput sent {} of {} inputs",
            sent,
            inputs.len()
        );
    }
}

fn unicode_input(scan: u16, extra_flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0), // 0 + KEYEVENTF_UNICODE → wScan is the codepoint
                wScan: scan,
                dwFlags: KEYEVENTF_UNICODE | extra_flags,
                time: 0,
                dwExtraInfo: injected_sentinel(),
            },
        },
    }
}

fn press_key(vk: VIRTUAL_KEY) {
    let inputs = [
        vk_input(vk, KEYBD_EVENT_FLAGS(0)),
        vk_input(vk, KEYEVENTF_KEYUP),
    ];
    // SAFETY: see type_text.
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if (sent as usize) != inputs.len() {
        eprintln!(
            "[WARN] press_key: SendInput sent {} of {} for {:?}",
            sent,
            inputs.len(),
            vk
        );
    }
}

/// Press a chord: modifiers go down first, payload key(s) tap, modifiers
/// release last (reverse order). This matches the standard "hold Shift,
/// press A, release Shift" sequence Windows expects from human input.
fn press_combo(keys: &[VIRTUAL_KEY]) {
    if keys.is_empty() {
        return;
    }

    let (mods, payload): (Vec<VIRTUAL_KEY>, Vec<VIRTUAL_KEY>) =
        keys.iter().copied().partition(|vk| is_modifier(*vk));

    // 2 INPUTs per key for the modifiers (down + up) and 2 per payload key.
    let mut inputs: Vec<INPUT> = Vec::with_capacity((mods.len() + payload.len()) * 2);

    for vk in &mods {
        inputs.push(vk_input(*vk, KEYBD_EVENT_FLAGS(0)));
    }
    for vk in &payload {
        inputs.push(vk_input(*vk, KEYBD_EVENT_FLAGS(0)));
    }
    // Reverse-order release.
    for vk in payload.iter().rev() {
        inputs.push(vk_input(*vk, KEYEVENTF_KEYUP));
    }
    for vk in mods.iter().rev() {
        inputs.push(vk_input(*vk, KEYEVENTF_KEYUP));
    }

    // SAFETY: see type_text.
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if (sent as usize) != inputs.len() {
        eprintln!(
            "[WARN] press_combo: SendInput sent {} of {} inputs",
            sent,
            inputs.len()
        );
    }
}

fn vk_input(vk: VIRTUAL_KEY, extra_flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: extra_flags,
                time: 0,
                dwExtraInfo: injected_sentinel(),
            },
        },
    }
}

fn is_modifier(vk: VIRTUAL_KEY) -> bool {
    matches!(
        vk,
        VK_SHIFT    | VK_LSHIFT   | VK_RSHIFT
        | VK_CONTROL  | VK_LCONTROL | VK_RCONTROL
        | VK_MENU     | VK_LMENU    | VK_RMENU
        | VK_LWIN     | VK_RWIN
    )
}

fn apply_window_op(op: WindowOp, target: HWND) {
    if target.is_invalid() {
        eprintln!("[WARN] window step skipped: no foreground HWND captured");
        return;
    }

    // SAFETY: target is a non-invalid HWND obtained from GetForegroundWindow
    // at sequence start; ShowWindow / PostMessageW both tolerate a stale
    // HWND by returning an error / FALSE without UB.
    unsafe {
        match op {
            WindowOp::Minimize => {
                let _ = ShowWindow(target, SW_MINIMIZE);
            }
            WindowOp::Maximize => {
                let _ = ShowWindow(target, SW_MAXIMIZE);
            }
            WindowOp::Restore => {
                let _ = ShowWindow(target, SW_RESTORE);
            }
            WindowOp::Close => {
                if let Err(e) = PostMessageW(Some(target), WM_CLOSE, WPARAM(0), LPARAM(0)) {
                    eprintln!("[WARN] window close PostMessage failed: {:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_modifier_classifies_correctly() {
        for vk in [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN, VK_LSHIFT] {
            assert!(is_modifier(vk), "{:?} should be a modifier", vk);
        }
        for vk in [VK_A, VK_Z, VK_F1, VK_CAPITAL, VK_ESCAPE] {
            assert!(!is_modifier(vk), "{:?} should not be a modifier", vk);
        }
    }

    #[test]
    fn display_truncates_long_text() {
        let long: String = "a".repeat(200);
        let s = SequenceStep::TypeText(long);
        let rendered = format!("{}", s);
        // Format: `text "aaa...aaa"…` — quote first, ellipsis after.
        assert!(rendered.ends_with("\"…"), "expected ellipsis after quote, got {}", rendered);
        assert!(rendered.len() < 100, "rendered should be truncated, got {} bytes", rendered.len());
    }

    #[test]
    fn display_short_text_unchanged() {
        let s = SequenceStep::TypeText("hi".into());
        assert_eq!(format!("{}", s), "text \"hi\"");
    }

    #[test]
    fn display_combo_uses_plus() {
        let s = SequenceStep::PressCombo(vec![VK_CONTROL, VK_S]);
        assert_eq!(format!("{}", s), "combo Ctrl+S");
    }

    #[test]
    fn display_delay_includes_ms() {
        let s = SequenceStep::Delay(250);
        assert_eq!(format!("{}", s), "delay 250ms");
    }
}
