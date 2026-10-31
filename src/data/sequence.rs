//! Sequence steps: the macro-like body of a `BindAction::Sequence`.
//!
//! A sequence runs on the action executor (`core::executor`, roadmap item
//! 2.1), because a low-level keyboard hook callback must return *fast* —
//! blocking it for any meaningful time would freeze input system-wide. The
//! hook only queues the macro; the executor walks its steps, keeps each
//! `Delay` as a deadline on its own timer, and runs every other step
//! through `run_step`.
//!
//! The target window (`GetForegroundWindow()` at the moment the combo
//! matched) is captured once and reused for every window-related step, so
//! "restore this window … type … minimize this window" operates on the same
//! HWND throughout — not on whatever happened to be focused later.

use std::sync::Arc;

use windows::Win32::{
    Foundation::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

use crate::core::constants::injected_sentinel;
use crate::core::windows_actions::{MediaKey, launch_process, shell_open};
use crate::data::paste::PasteSpec;
use crate::data::vk_name::vk_name;

#[derive(Debug, Clone)]
pub enum SequenceStep {
    /// Type a Unicode string via `SendInput` with `KEYEVENTF_UNICODE`. This
    /// is layout-independent — characters appear regardless of which IME or
    /// keyboard layout the foreground window is using.
    TypeText(String),

    /// Put the phrase on the clipboard, send the paste keystroke, and put
    /// the clipboard back later, as the `Paste` action does. The action
    /// executor runs it itself, in its paste session, and never hands it to
    /// `run_step`. `Display` and `Debug` show its length and keys only.
    Paste(Arc<PasteSpec>),

    /// Synthesize a single virtual-key press (down then up). No modifiers.
    PressKey(VIRTUAL_KEY),

    /// Synthesize a chord: modifiers down, key down, key up, modifiers up.
    /// The split between modifiers and "the actual key" is automatic — every
    /// known modifier vkey (Shift/Ctrl/Alt/Win, sided variants) is held
    /// during the press, everything else is the chord's payload key(s).
    PressCombo(Vec<VIRTUAL_KEY>),

    /// Wait `ms` milliseconds before the next step. The action executor
    /// keeps it as a deadline on its own timer, so other actions and macros
    /// go on meanwhile, and `swch off`, a reload or exit cut it short.
    Delay(u64),

    /// Minimise, maximise or restore the captured target window with
    /// `ShowWindowAsync`, or close it with a posted `WM_CLOSE`. Neither
    /// waits for the window: it changes when its own thread handles the
    /// request, so a step that depends on the change needs a `Delay`
    /// before it.
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
            SequenceStep::Paste(spec) => write!(f, "{spec}"),
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

/// Runs one step against `target`, on the calling thread.
pub(crate) fn run_step(step: &SequenceStep, target: HWND) {
    match step {
        SequenceStep::TypeText(s) => type_text(s),
        // The executor runs a Paste step itself, in its paste session
        // (`runs::Run::advance`), and never hands one here.
        SequenceStep::Paste(_) => {}
        SequenceStep::PressKey(vk) => press_key(*vk),
        SequenceStep::PressCombo(keys) => {
            press_combo(keys);
        }
        // The executor keeps a Delay itself and never hands one here.
        SequenceStep::Delay(_) => {}
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
///
/// The one builder of both the macro step `Text` and the `Text` action
/// (`Win32Runner::action`): the whole text goes out in one `SendInput`, so
/// no other input lands inside it.
pub(crate) fn type_text(s: &str) {
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
        log::warn!(
            "type_text: SendInput sent {sent} of {} inputs",
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
        // No key named: the log never records which keys were sent.
        log::warn!("press_key: SendInput sent {sent} of {}", inputs.len());
    }
}

/// How many events of a [`press_combo`] batch `SendInput` inserted, of how
/// many.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboSent {
    pub sent: u32,
    pub of: u32,
}

/// Press a chord: modifiers go down first, payload key(s) tap, modifiers
/// release last (reverse order). This matches the standard "hold Shift,
/// press A, release Shift" sequence Windows expects from human input.
///
/// The macro step `Combo` and the `Paste` keystroke (`executor::paste`)
/// send it, in one `SendInput`, each event with the sentinel.
pub(crate) fn press_combo(keys: &[VIRTUAL_KEY]) -> ComboSent {
    if keys.is_empty() {
        return ComboSent { sent: 0, of: 0 };
    }

    let inputs: Vec<INPUT> = combo_events(keys)
        .into_iter()
        .map(|(vk, flags)| vk_input(vk, flags))
        .collect();

    let sent = send(&inputs);
    if (sent as usize) != inputs.len() {
        log::warn!(
            "press_combo: SendInput sent {sent} of {} inputs",
            inputs.len()
        );
    }
    ComboSent {
        sent,
        of: u32::try_from(inputs.len()).unwrap_or(u32::MAX),
    }
}

/// The events of a `press_combo` batch of `keys`, in the order it sends
/// them: the modifiers down, the payload keys down, the payload keys up in
/// reverse order, the modifiers up in reverse order. One list, so that the
/// batch and [`unreleased_keys`] cannot disagree on it.
fn combo_events(keys: &[VIRTUAL_KEY]) -> Vec<(VIRTUAL_KEY, KEYBD_EVENT_FLAGS)> {
    let (mods, payload): (Vec<VIRTUAL_KEY>, Vec<VIRTUAL_KEY>) =
        keys.iter().copied().partition(|vk| is_modifier(*vk));
    let down = KEYBD_EVENT_FLAGS(0);
    mods.iter()
        .chain(&payload)
        .map(|&vk| (vk, down))
        .chain(
            payload
                .iter()
                .rev()
                .chain(mods.iter().rev())
                .map(|&vk| (vk, KEYEVENTF_KEYUP)),
        )
        .collect()
}

/// The key-ups a `press_combo` batch of `keys` still owes when `SendInput`
/// inserted only its first `sent` events: those of the keys whose key-down
/// went out and whose key-up did not, in the order the batch releases them,
/// so that no key stays down (research critique M1).
pub(crate) fn unreleased_keys(keys: &[VIRTUAL_KEY], sent: u32) -> Vec<VIRTUAL_KEY> {
    let events = combo_events(keys);
    let sent = usize::try_from(sent).map_or(events.len(), |sent| sent.min(events.len()));
    let (done, left) = events.split_at(sent);
    left.iter()
        .filter(|(vk, flags)| {
            *flags == KEYEVENTF_KEYUP
                && done
                    .iter()
                    .any(|(went, went_flags)| went == vk && *went_flags != KEYEVENTF_KEYUP)
        })
        .map(|(vk, _)| *vk)
        .collect()
}

/// Sends the key-ups [`unreleased_keys`] names, in one `SendInput`, each
/// with the sentinel, and returns how many went out.
// Its only caller, the daemon's desk (`executor::paste`), is compiled out
// under test.
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn release_unreleased(keys: &[VIRTUAL_KEY], sent: u32) -> u32 {
    let inputs: Vec<INPUT> = unreleased_keys(keys, sent)
        .into_iter()
        .map(|vk| vk_input(vk, KEYEVENTF_KEYUP))
        .collect();
    if inputs.is_empty() {
        return 0;
    }
    send(&inputs)
}

/// Sends `inputs` in one `SendInput` and returns how many it inserted.
fn send(inputs: &[INPUT]) -> u32 {
    // SAFETY: `inputs` is a live slice of initialised INPUT records that
    // outlives the call, which only reads it, and cbSize is the size of one
    // record, so SendInput reads exactly the slice; every record carries the
    // sentinel (`vk_input`).
    unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) }
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

/// Whether `vk` is a Shift, Ctrl, Alt or Win code, generic or sided: what
/// `press_combo` holds down around the other keys.
pub(crate) fn is_modifier(vk: VIRTUAL_KEY) -> bool {
    matches!(
        vk,
        VK_SHIFT
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_CONTROL
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_MENU
            | VK_LMENU
            | VK_RMENU
            | VK_LWIN
            | VK_RWIN
    )
}

fn apply_window_op(op: WindowOp, target: HWND) {
    if target.is_invalid() {
        log::warn!("window step skipped: no foreground HWND captured");
        return;
    }

    // Nothing here waits for the target window, which may be Not
    // Responding: ShowWindowAsync "posts a show-window event to
    // the message queue of the given window", and WM_CLOSE is posted.
    let show = match op {
        WindowOp::Minimize => SW_MINIMIZE,
        WindowOp::Maximize => SW_MAXIMIZE,
        WindowOp::Restore => SW_RESTORE,
        WindowOp::Close => {
            // SAFETY: target is a non-invalid HWND obtained from
            // GetForegroundWindow when the macro fired; PostMessageW takes
            // it and the message by value, reads no memory of ours, and
            // fails without UB on a stale HWND.
            let posted = unsafe { PostMessageW(Some(target), WM_CLOSE, WPARAM(0), LPARAM(0)) };
            if let Err(e) = posted {
                log::warn!("window close PostMessage failed: {e:?}");
            }
            return;
        }
    };
    // SAFETY: target is a non-invalid HWND obtained from GetForegroundWindow
    // when the macro fired; ShowWindowAsync takes it by value, reads no
    // memory of ours, only posts to the window's thread and returns FALSE
    // without UB on a stale HWND.
    let started = unsafe { ShowWindowAsync(target, show) };
    if !started.as_bool() {
        log::warn!("window step failed: ShowWindowAsync did not start {op:?}");
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
        assert!(
            rendered.ends_with("\"…"),
            "expected ellipsis after quote, got {}",
            rendered
        );
        assert!(
            rendered.len() < 100,
            "rendered should be truncated, got {} bytes",
            rendered.len()
        );
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

    #[test]
    fn the_key_ups_left_after_a_cut_short_combo() {
        // The batch of [Ctrl, V]: Ctrl down, V down, V up, Ctrl up.
        let ctrl_v = [VK_CONTROL, VK_V];
        let left: Vec<Vec<VIRTUAL_KEY>> =
            (0..=4).map(|sent| unreleased_keys(&ctrl_v, sent)).collect();
        assert_eq!(
            left,
            [
                vec![],
                vec![VK_CONTROL],
                vec![VK_V, VK_CONTROL],
                vec![VK_CONTROL],
                vec![]
            ]
        );
        let shift_insert = [VK_SHIFT, VK_INSERT];
        let left: Vec<Vec<VIRTUAL_KEY>> = (0..=4)
            .map(|sent| unreleased_keys(&shift_insert, sent))
            .collect();
        assert_eq!(
            left,
            [
                vec![],
                vec![VK_SHIFT],
                vec![VK_INSERT, VK_SHIFT],
                vec![VK_SHIFT],
                vec![]
            ]
        );
        // Ctrl down, Shift down, V down, V up, Shift up, Ctrl up.
        let ctrl_shift_v = [VK_CONTROL, VK_SHIFT, VK_V];
        assert_eq!(unreleased_keys(&ctrl_shift_v, 2), [VK_SHIFT, VK_CONTROL]);
        assert_eq!(unreleased_keys(&ctrl_shift_v, 4), [VK_SHIFT, VK_CONTROL]);
        assert_eq!(unreleased_keys(&ctrl_shift_v, 5), [VK_CONTROL]);
        assert_eq!(unreleased_keys(&ctrl_shift_v, 6), []);
        assert_eq!(
            unreleased_keys(&ctrl_shift_v, 99),
            [],
            "more than the batch"
        );
    }

    #[test]
    fn a_paste_step_shows_its_length_and_keys_only() {
        let secret = "P4ssw0rd-QZX";
        let spec = PasteSpec::new(secret, vec![VK_SHIFT, VK_INSERT], 800).expect("a valid spec");
        let step = SequenceStep::Paste(Arc::new(spec));

        assert_eq!(step.to_string(), "paste (12 chars, Shift+Insert)");
        assert_eq!(
            format!("{step:?}"),
            "Paste(PasteSpec { chars: 12, keys: Shift+Insert, restore_delay: 800ms })"
        );
        let chars: Vec<char> = secret.chars().collect();
        for rendered in [step.to_string(), format!("{step:?}"), format!("{step:#?}")] {
            for window in chars.windows(3) {
                let part: String = window.iter().collect();
                assert!(!rendered.contains(&part), "{part:?} shows in {rendered}");
            }
        }
    }

    /// The `Paste` keystroke goes out through `press_combo`, so the lane
    /// adds no place that builds or sends key events (cut D15): the code
    /// lines that call `SendInput`, build a `KEYBDINPUT` or read the
    /// sentinel are exactly these, per file, outside the tests modules, the
    /// bench and the test-only helpers. The input-module lane of roadmap
    /// item 3.1 replaces this pin by its scan. The tokens are assembled, so
    /// this file does not match itself.
    #[test]
    fn the_paste_adds_no_injection_site() {
        use crate::scan::{code_of, production_lines, rust_files};

        let tokens: [(&str, &[(&str, usize)]); 3] = [
            (
                concat!("Send", "Input("),
                &[
                    ("src/core/windows_actions.rs", 1),
                    ("src/data/sequence.rs", 3),
                    ("src/hook/keyboard_hook_callback.rs", 1),
                    ("src/system/system_function.rs", 1),
                ],
            ),
            (
                concat!("KEYBD", "INPUT {"),
                &[
                    ("src/core/windows_actions.rs", 2),
                    ("src/data/sequence.rs", 2),
                    ("src/hook/keyboard_hook_callback.rs", 1),
                    ("src/system/system_function.rs", 2),
                ],
            ),
            (
                concat!("injected_", "sentinel()"),
                &[
                    ("src/core/constants.rs", 2),
                    ("src/core/windows_actions.rs", 2),
                    ("src/data/sequence.rs", 2),
                    ("src/hook/keyboard_hook_callback.rs", 1),
                    ("src/system/system_function.rs", 2),
                ],
            ),
        ];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rust_files(&root.join("src"), &mut files);
        files.sort();
        assert!(files.len() > 40, "the scan found the sources: {files:?}");

        let mut report = String::new();
        for (token, expected) in tokens {
            let mut found: Vec<(String, usize)> = Vec::new();
            for file in &files {
                let relative = file
                    .strip_prefix(root)
                    .expect("a file under the manifest folder")
                    .to_string_lossy()
                    .replace('\\', "/");
                if relative == "src/bench.rs"
                    || relative == "src/scan.rs"
                    || relative.ends_with("/testing.rs")
                {
                    continue;
                }
                let text = std::fs::read_to_string(file).expect("read a source file");
                let lines: Vec<&str> = text.lines().collect();
                let count = production_lines(&lines)
                    .iter()
                    .filter(|line| code_of(line).contains(token))
                    .count();
                if count > 0 {
                    found.push((relative, count));
                }
            }
            let expected: Vec<(String, usize)> = expected
                .iter()
                .map(|(file, count)| ((*file).to_owned(), *count))
                .collect();
            if found != expected {
                report += &format!("{token}: found {found:?}, expected {expected:?}\n");
            }
        }
        assert!(report.is_empty(), "{report}");
    }

    /// Roadmap item 2.1: `ShowWindow` waits until the target window's thread has
    /// handled the change, so on a window that is Not Responding it would
    /// hold up the one executor thread. The name is assembled, so the scan
    /// does not match itself.
    #[test]
    fn a_window_step_never_waits_for_its_window() {
        let blocking = concat!("ShowWindow", "(");
        let file = "src/data/sequence.rs";
        let text =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file))
                .expect("read the sequence module");
        let lines: Vec<&str> = text.lines().collect();
        let tests_at = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .expect("the sequence module has a tests module");
        let sites: Vec<String> = lines[..tests_at]
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(blocking))
            .map(|(index, line)| format!("{file}:{}: {}", index + 1, line.trim()))
            .collect();
        assert!(
            sites.is_empty(),
            "synchronous ShowWindow calls:\n{}",
            sites.join("\n")
        );
    }
}
