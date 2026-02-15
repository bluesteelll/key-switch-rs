use std::sync::Arc;

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::{
            Input::KeyboardAndMouse::*,
            Shell::ShellExecuteW,
            WindowsAndMessaging::*,
        },
    },
};

use crate::core::constants::injected_sentinel;
use crate::data::key_combination::KeyCombination;
use crate::data::sequence::{spawn_sequence, SequenceStep};
use crate::system::system_function::SystemFunction;

#[allow(dead_code)] // Variants are part of the public surface; not every binary uses them all.
#[derive(Debug, Clone)]
pub enum BindAction {
    SwitchLanguage,
    SwitchLanguageBackward,
    ToggleCapsLock,
    PressKey(VIRTUAL_KEY),
    PostMessage { msg: u32, wparam: usize, lparam: isize },
    /// Macro-like body: list of steps executed in a spawned worker thread.
    /// The hook callback must return fast, so it only kicks off the thread —
    /// it never blocks waiting for delays or `SendInput` to drain.
    ///
    /// `Arc` makes cloning the binding (and therefore freezing the binding
    /// list at install) cheap regardless of sequence length.
    Sequence(Arc<Vec<SequenceStep>>),
    /// Spawn a new process. Detached: child outlives the daemon, no console
    /// window is attached to ours. Use absolute path or rely on PATH lookup.
    Launch { exe: String, args: Vec<String> },
    /// Hand a URL / mailto: / file path to the OS default handler via
    /// `ShellExecuteW`. Works for `https://...`, `mailto:...`, document files,
    /// even plain executables (equivalent to "open" verb).
    OpenUrl(String),
    /// Synthesize a media / volume / playback key. Maps to one of the
    /// `VK_MEDIA_*` / `VK_VOLUME_*` virtual keys via `SendInput`.
    Media(MediaKey),
    DoNothing,
}

/// Media-control keys synthesized via `SendInput`. These map 1:1 to the
/// virtual keys most physical multimedia keyboards already emit, so binding
/// a hardware media key is the same code path as binding e.g. F13 to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKey {
    PlayPause,
    Stop,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    VolumeMute,
}

impl MediaKey {
    pub fn as_vk(self) -> VIRTUAL_KEY {
        match self {
            MediaKey::PlayPause  => VK_MEDIA_PLAY_PAUSE,
            MediaKey::Stop       => VK_MEDIA_STOP,
            MediaKey::Next       => VK_MEDIA_NEXT_TRACK,
            MediaKey::Previous   => VK_MEDIA_PREV_TRACK,
            MediaKey::VolumeUp   => VK_VOLUME_UP,
            MediaKey::VolumeDown => VK_VOLUME_DOWN,
            MediaKey::VolumeMute => VK_VOLUME_MUTE,
        }
    }
}

// Hand-rolled `PartialEq` because the inner `Arc<Vec<…>>` makes the default
// derive force structural equality on the Vec; we want pointer equality (two
// bindings with the exact same `Sequence` Arc compare equal, distinct Arcs
// compare unequal) which is the only case the rest of the program cares
// about (auto-blocker dedup).
impl PartialEq for BindAction {
    fn eq(&self, other: &Self) -> bool {
        use BindAction::*;
        match (self, other) {
            (SwitchLanguage, SwitchLanguage)
            | (SwitchLanguageBackward, SwitchLanguageBackward)
            | (ToggleCapsLock, ToggleCapsLock)
            | (DoNothing, DoNothing) => true,
            (PressKey(a), PressKey(b)) => a == b,
            (
                PostMessage { msg: m1, wparam: w1, lparam: l1 },
                PostMessage { msg: m2, wparam: w2, lparam: l2 },
            ) => m1 == m2 && w1 == w2 && l1 == l2,
            (Sequence(a), Sequence(b)) => Arc::ptr_eq(a, b),
            (Launch { exe: e1, args: a1 }, Launch { exe: e2, args: a2 }) => e1 == e2 && a1 == a2,
            (OpenUrl(a), OpenUrl(b)) => a == b,
            (Media(a), Media(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for BindAction {}

impl BindAction {
    pub fn execute(&self) {
        match self {
            BindAction::SwitchLanguage
            | BindAction::SwitchLanguageBackward
            | BindAction::ToggleCapsLock => {
                if let Some(sys_func) = self.to_system_function() {
                    sys_func.execute();
                }
            }
            BindAction::PressKey(vk) => press_vk(*vk),
            BindAction::PostMessage { msg, wparam, lparam } => {
                post_message_to_foreground(*msg, *wparam, *lparam)
            }
            BindAction::Sequence(steps) => {
                // Cheap Arc clone — the worker thread holds its own handle.
                spawn_sequence(Arc::clone(steps));
            }
            BindAction::Launch { exe, args } => launch_process(exe, args),
            BindAction::OpenUrl(url) => shell_open(url),
            BindAction::Media(key) => press_vk(key.as_vk()),
            BindAction::DoNothing => {}
        }
    }

    pub fn to_system_function(&self) -> Option<SystemFunction> {
        match self {
            BindAction::SwitchLanguage => Some(SystemFunction::SwitchLanguage),
            BindAction::SwitchLanguageBackward => Some(SystemFunction::SwitchLanguageBackward),
            BindAction::ToggleCapsLock => Some(SystemFunction::ToggleCapsLock),
            _ => None,
        }
    }

    /// The system-default hotkey for this action's underlying OS function, if
    /// any. Used to derive auto-blockers when `block_original_combo` is set.
    pub fn get_system_combination(&self) -> Option<KeyCombination> {
        self.to_system_function()
            .and_then(|sf| sf.get_system_combination())
    }
}

/// Synthesize a single key press (down + up). Shared between `PressKey`,
/// `Media(...)`, and the sequence `Key(...)` step.
pub(crate) fn press_vk(vk: VIRTUAL_KEY) {
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
        eprintln!(
            "SendInput dropped events for vk {:?}: sent {} of {}",
            vk,
            sent,
            inputs.len()
        );
    }
}

fn post_message_to_foreground(msg: u32, wparam: usize, lparam: isize) {
    // SAFETY: GetForegroundWindow / PostMessageW have no preconditions on
    // their inputs beyond what we already validate (HWND non-invalid).
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return;
        }

        if let Err(e) = PostMessageW(Some(hwnd), msg, WPARAM(wparam), LPARAM(lparam)) {
            eprintln!("Post message error {:#X}: {:?}", msg, e);
        }
    }
}

pub(crate) fn launch_process(exe: &str, args: &[String]) {
    // `Command::spawn()` on Windows produces a child detached from this
    // process's lifetime — closing the daemon does not kill launched apps.
    // We deliberately do not pipe stdin/stdout/stderr; child runs with
    // inherited handles, which for a Windows console daemon means it
    // attaches to our console if it's a CLI tool (rare for hotkey targets).
    match std::process::Command::new(exe).args(args).spawn() {
        Ok(_child) => {
            // We drop the Child handle; the OS keeps the process alive.
        }
        Err(e) => {
            eprintln!("[ERROR] Launch {:?}: {}", exe, e);
        }
    }
}

pub(crate) fn shell_open(target: &str) {
    let target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: target_wide is null-terminated; the "open" verb literal lives
    // for the call duration. ShellExecuteW tolerates invalid URLs by
    // returning an HINSTANCE <= 32 — see MSDN return-value table.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(target_wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns the new instance's HINSTANCE on success.
    // Per MSDN, values <= 32 are error codes (cast through usize because the
    // HINSTANCE-as-pointer convention is tagged-pointer-style on Windows).
    let val = result.0 as usize;
    if val <= 32 {
        eprintln!(
            "[ERROR] OpenUrl {:?}: ShellExecuteW returned error code {}",
            target, val
        );
    }
}

impl std::fmt::Display for BindAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindAction::SwitchLanguage => write!(f, "switch language"),
            BindAction::SwitchLanguageBackward => write!(f, "switch language backward"),
            BindAction::ToggleCapsLock => write!(f, "toggle CapsLock"),
            BindAction::PressKey(vk) => write!(f, "press key {}", crate::data::vk_name::vk_name(*vk)),
            BindAction::PostMessage { msg, .. } => write!(f, "post message {:#X}", msg),
            BindAction::Sequence(steps) => write!(f, "sequence ({} steps)", steps.len()),
            BindAction::Launch { exe, args } => {
                if args.is_empty() {
                    write!(f, "launch {}", exe)
                } else {
                    write!(f, "launch {} {}", exe, args.join(" "))
                }
            }
            BindAction::OpenUrl(url) => write!(f, "open {}", url),
            BindAction::Media(key) => write!(f, "media {:?}", key),
            BindAction::DoNothing => write!(f, "do nothing"),
        }
    }
}
