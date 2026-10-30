use std::sync::Arc;

use windows::{
    Win32::UI::{Input::KeyboardAndMouse::*, Shell::ShellExecuteW, WindowsAndMessaging::*},
    core::*,
};

use crate::core::constants::injected_sentinel;
use crate::data::key_combination::KeyCombination;
use crate::data::sequence::SequenceStep;
use crate::logging::counters::{self, Counter};
use crate::system::system_function::SystemFunction;

// `Debug` is written out below, so that the text of a `Text` never reaches
// a `{:?}`, an `expect` or a panic message.
#[derive(Clone)]
pub enum BindAction {
    SwitchLanguage,
    SwitchLanguageBackward,
    ToggleCapsLock,
    PressKey(VIRTUAL_KEY),
    PostMessage {
        msg: u32,
        wparam: usize,
        lparam: isize,
    },
    /// Macro-like body: a list of steps run on the action executor
    /// (`core::executor`, roadmap item 2.1). The hook only queues it and
    /// never waits for its delays or its `SendInput`s.
    ///
    /// `Arc` makes cloning the binding (and therefore freezing the binding
    /// list at install) cheap regardless of sequence length.
    Sequence(Arc<Vec<SequenceStep>>),
    /// Spawn a process with `Command::spawn`. The child outlives the daemon
    /// but inherits its console and, in a release build, its administrator
    /// token (KS-09, KS-34; roadmap item 3.3). Use absolute path or rely on
    /// PATH lookup.
    Launch {
        exe: String,
        args: Vec<String>,
    },
    /// Hand a URL / mailto: / file path to the OS default handler via
    /// `ShellExecuteW`. Works for `https://...`, `mailto:...`, document files,
    /// even plain executables (equivalent to "open" verb).
    OpenUrl(String),
    /// Type the text at once, in one `SendInput` batch of `KEYEVENTF_UNICODE`
    /// events, as the macro step `Text` does (`sequence::type_text`), on the
    /// action executor. The `Arc` is built once at load, so a firing only
    /// bumps its count. `Display` and `Debug` show its length only.
    Text(Arc<str>),
    /// Synthesize a media / volume / playback key. Maps to one of the
    /// `VK_MEDIA_*` / `VK_VOLUME_*` virtual keys via `SendInput`.
    Media(MediaKey),
    DoNothing,
}

impl std::fmt::Debug for BindAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Every arm but `Text` writes what `#[derive(Debug)]` wrote. Every
        // arm is spelled out, so a new action does not compile until its
        // form is decided here too.
        match self {
            BindAction::SwitchLanguage => f.write_str("SwitchLanguage"),
            BindAction::SwitchLanguageBackward => f.write_str("SwitchLanguageBackward"),
            BindAction::ToggleCapsLock => f.write_str("ToggleCapsLock"),
            BindAction::PressKey(vk) => f.debug_tuple("PressKey").field(vk).finish(),
            BindAction::PostMessage {
                msg,
                wparam,
                lparam,
            } => f
                .debug_struct("PostMessage")
                .field("msg", msg)
                .field("wparam", wparam)
                .field("lparam", lparam)
                .finish(),
            BindAction::Sequence(steps) => f.debug_tuple("Sequence").field(steps).finish(),
            BindAction::Launch { exe, args } => f
                .debug_struct("Launch")
                .field("exe", exe)
                .field("args", args)
                .finish(),
            BindAction::OpenUrl(url) => f.debug_tuple("OpenUrl").field(url).finish(),
            BindAction::Text(text) => f
                .debug_tuple("Text")
                .field(&format_args!("<{} chars>", text.chars().count()))
                .finish(),
            BindAction::Media(key) => f.debug_tuple("Media").field(key).finish(),
            BindAction::DoNothing => f.write_str("DoNothing"),
        }
    }
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
            MediaKey::PlayPause => VK_MEDIA_PLAY_PAUSE,
            MediaKey::Stop => VK_MEDIA_STOP,
            MediaKey::Next => VK_MEDIA_NEXT_TRACK,
            MediaKey::Previous => VK_MEDIA_PREV_TRACK,
            MediaKey::VolumeUp => VK_VOLUME_UP,
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
                PostMessage {
                    msg: m1,
                    wparam: w1,
                    lparam: l1,
                },
                PostMessage {
                    msg: m2,
                    wparam: w2,
                    lparam: l2,
                },
            ) => m1 == m2 && w1 == w2 && l1 == l2,
            (Sequence(a), Sequence(b)) => Arc::ptr_eq(a, b),
            (Launch { exe: e1, args: a1 }, Launch { exe: e2, args: a2 }) => e1 == e2 && a1 == a2,
            (OpenUrl(a), OpenUrl(b)) => a == b,
            (Text(a), Text(b)) => a == b,
            (Media(a), Media(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for BindAction {}

/// An action that runs on the thread that fires it, the main thread, in the
/// hook callback before the hook returns or in the main loop for a Hold, so
/// that its output stays in order with the keys typed after it (roadmap
/// items 2.1 and 2.2). Each one only posts a message or sends one key
/// press. Every other action goes to the action executor;
/// [`BindAction::as_fast`] decides which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastAction {
    SwitchLanguage,
    SwitchLanguageBackward,
    ToggleCapsLock,
    PressKey(VIRTUAL_KEY),
    PostMessage {
        msg: u32,
        wparam: usize,
        lparam: isize,
    },
    Media(MediaKey),
    DoNothing,
}

impl FastAction {
    /// Runs the action on the calling thread. It can run on the hook
    /// thread at every auto-repeat, so a failure is only counted
    /// (`logging::counters`), never logged.
    ///
    /// Under test it is only recorded, on the calling thread
    /// (`fast_actions_run`): a test that carries out a decision must not
    /// type into, post to or switch the layout of the window that has the
    /// focus on the machine that runs it, whatever its action.
    pub fn run(self) {
        #[cfg(test)]
        FAST_ACTIONS_RUN.with(|ran| ran.borrow_mut().push(self));
        #[cfg(not(test))]
        match self {
            FastAction::SwitchLanguage => SystemFunction::SwitchLanguage.execute(),
            FastAction::SwitchLanguageBackward => SystemFunction::SwitchLanguageBackward.execute(),
            FastAction::ToggleCapsLock => SystemFunction::ToggleCapsLock.execute(),
            FastAction::PressKey(vk) => press_vk(vk),
            FastAction::PostMessage {
                msg,
                wparam,
                lparam,
            } => crate::system::system_function::post_to_foreground(msg, wparam, lparam),
            FastAction::Media(key) => press_vk(key.as_vk()),
            FastAction::DoNothing => {}
        }
    }
}

#[cfg(test)]
thread_local! {
    /// The fast actions `FastAction::run` was asked to run on this thread,
    /// in order.
    static FAST_ACTIONS_RUN: std::cell::RefCell<Vec<FastAction>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Takes the fast actions `FastAction::run` recorded on this thread.
#[cfg(test)]
pub(crate) fn fast_actions_run() -> Vec<FastAction> {
    FAST_ACTIONS_RUN.with(|ran| ran.take())
}

impl BindAction {
    /// The action as a [`FastAction`], to run where it fires, or `None`
    /// for `Launch`, `OpenUrl`, `Text` and `Sequence`, which go to the
    /// action executor. Every arm is spelled out, so a new action does not
    /// compile until its place is decided.
    pub fn as_fast(&self) -> Option<FastAction> {
        match self {
            BindAction::SwitchLanguage => Some(FastAction::SwitchLanguage),
            BindAction::SwitchLanguageBackward => Some(FastAction::SwitchLanguageBackward),
            BindAction::ToggleCapsLock => Some(FastAction::ToggleCapsLock),
            BindAction::PressKey(vk) => Some(FastAction::PressKey(*vk)),
            BindAction::PostMessage {
                msg,
                wparam,
                lparam,
            } => Some(FastAction::PostMessage {
                msg: *msg,
                wparam: *wparam,
                lparam: *lparam,
            }),
            BindAction::Media(key) => Some(FastAction::Media(*key)),
            BindAction::DoNothing => Some(FastAction::DoNothing),
            BindAction::Sequence(_)
            | BindAction::Launch { .. }
            | BindAction::OpenUrl(_)
            | BindAction::Text(_) => None,
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

    /// Whether a binding with this action fires again on every auto-repeat
    /// of its key when the config does not set `repeat:` (owner decision,
    /// roadmap item 1.1, KS-10): only a key press and the two volume keys,
    /// whose effect a held key is meant to repeat. Every arm is spelled out,
    /// so a new action does not compile until its default is decided.
    pub fn repeats_by_default(&self) -> bool {
        match self {
            BindAction::PressKey(_) => true,
            BindAction::Media(key) => match key {
                MediaKey::VolumeUp | MediaKey::VolumeDown => true,
                MediaKey::PlayPause
                | MediaKey::Stop
                | MediaKey::Next
                | MediaKey::Previous
                | MediaKey::VolumeMute => false,
            },
            BindAction::SwitchLanguage
            | BindAction::SwitchLanguageBackward
            | BindAction::ToggleCapsLock
            | BindAction::PostMessage { .. }
            | BindAction::Sequence(_)
            | BindAction::Launch { .. }
            | BindAction::OpenUrl(_)
            | BindAction::Text(_)
            | BindAction::DoNothing => false,
        }
    }
}

/// Synthesize a single key press (down + up), for `PressKey` and
/// `Media(...)`. It runs on the hook thread at every auto-repeat, so a short
/// count is only counted (`key_press_short`), never logged.
// Under test `FastAction::run`, its only caller, only records.
#[cfg_attr(test, allow(dead_code))]
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
        counters::bump(Counter::KeyPressShort);
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
            log::error!("Launch {exe:?}: {e}");
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
        log::error!("OpenUrl {target:?}: ShellExecuteW returned error code {val}");
    }
}

impl std::fmt::Display for BindAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindAction::SwitchLanguage => write!(f, "switch language"),
            BindAction::SwitchLanguageBackward => write!(f, "switch language backward"),
            BindAction::ToggleCapsLock => write!(f, "toggle CapsLock"),
            BindAction::PressKey(vk) => {
                write!(f, "press key {}", crate::data::vk_name::vk_name(*vk))
            }
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
            // The length only: the startup list must not show the phrase.
            BindAction::Text(text) => write!(f, "text ({} chars)", text.chars().count()),
            BindAction::Media(key) => write!(f, "media {:?}", key),
            BindAction::DoNothing => write!(f, "do nothing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_BACK;
    use windows::Win32::UI::WindowsAndMessaging::{WM_CLOSE, WM_NULL};

    #[test]
    fn only_press_key_volume_up_and_volume_down_repeat_by_default() {
        let actions = [
            (BindAction::SwitchLanguage, false),
            (BindAction::SwitchLanguageBackward, false),
            (BindAction::ToggleCapsLock, false),
            (BindAction::PressKey(VK_BACK), true),
            (
                BindAction::PostMessage {
                    msg: WM_CLOSE,
                    wparam: 0,
                    lparam: 0,
                },
                false,
            ),
            (BindAction::Sequence(Arc::new(Vec::new())), false),
            (
                BindAction::Launch {
                    exe: "wt.exe".to_owned(),
                    args: Vec::new(),
                },
                false,
            ),
            (BindAction::OpenUrl("https://example.com".to_owned()), false),
            (BindAction::Text(Arc::from("x")), false),
            (BindAction::Media(MediaKey::PlayPause), false),
            (BindAction::Media(MediaKey::Stop), false),
            (BindAction::Media(MediaKey::Next), false),
            (BindAction::Media(MediaKey::Previous), false),
            (BindAction::Media(MediaKey::VolumeUp), true),
            (BindAction::Media(MediaKey::VolumeDown), true),
            (BindAction::Media(MediaKey::VolumeMute), false),
            (BindAction::DoNothing, false),
        ];
        for (action, expected) in actions {
            assert_eq!(action.repeats_by_default(), expected, "{action:?}");
        }
    }

    #[test]
    fn under_test_a_fast_action_is_only_recorded() {
        // Every test that carries out a decision relies on this. WM_NULL is
        // the action here, so that a run that did reach Windows would post
        // a message that does nothing.
        let null = FastAction::PostMessage {
            msg: WM_NULL,
            wparam: 0,
            lparam: 0,
        };
        assert_eq!(fast_actions_run(), []);
        null.run();
        FastAction::DoNothing.run();
        assert_eq!(fast_actions_run(), [null, FastAction::DoNothing]);
        assert_eq!(fast_actions_run(), [], "taking them empties the record");
    }

    #[test]
    fn only_launch_open_url_text_and_sequence_leave_the_firing_thread() {
        // Roadmap item 2.1: the fast actions stay where they fire, so their
        // output keeps its order with the keys typed after them; the
        // others, `Text` among them, whose batch grows with its text, go to
        // the action executor. Nothing here runs an action.
        let mut actions = vec![
            (BindAction::SwitchLanguage, Some(FastAction::SwitchLanguage)),
            (
                BindAction::SwitchLanguageBackward,
                Some(FastAction::SwitchLanguageBackward),
            ),
            (BindAction::ToggleCapsLock, Some(FastAction::ToggleCapsLock)),
            (
                BindAction::PressKey(VK_BACK),
                Some(FastAction::PressKey(VK_BACK)),
            ),
            (
                BindAction::PostMessage {
                    msg: WM_CLOSE,
                    wparam: 7,
                    lparam: -3,
                },
                Some(FastAction::PostMessage {
                    msg: WM_CLOSE,
                    wparam: 7,
                    lparam: -3,
                }),
            ),
            (BindAction::DoNothing, Some(FastAction::DoNothing)),
            (BindAction::Sequence(Arc::new(Vec::new())), None),
            (
                BindAction::Launch {
                    exe: "wt.exe".to_owned(),
                    args: Vec::new(),
                },
                None,
            ),
            (BindAction::OpenUrl("https://example.com".to_owned()), None),
            (BindAction::Text(Arc::from("x")), None),
        ];
        for key in [
            MediaKey::PlayPause,
            MediaKey::Stop,
            MediaKey::Next,
            MediaKey::Previous,
            MediaKey::VolumeUp,
            MediaKey::VolumeDown,
            MediaKey::VolumeMute,
        ] {
            actions.push((BindAction::Media(key), Some(FastAction::Media(key))));
        }
        for (action, expected) in actions {
            assert_eq!(action.as_fast(), expected, "{action:?}");
        }
    }

    /// Whether `rendered` holds no 3-character run of `secret`.
    fn shows_no_part_of(rendered: &str, secret: &str) -> bool {
        let chars: Vec<char> = secret.chars().collect();
        chars.windows(3).all(|window| {
            let part: String = window.iter().collect();
            !rendered.contains(&part)
        })
    }

    #[test]
    fn a_text_action_shows_its_length_and_never_its_text() {
        // The startup list, a log line, an `expect` or a panic message must
        // not carry a phrase that can be a password.
        let secret = "P4ssw0rd-QZX";
        let action = BindAction::Text(Arc::from(secret));

        assert_eq!(action.to_string(), "text (12 chars)");
        assert_eq!(format!("{action:?}"), "Text(<12 chars>)");
        for rendered in [
            action.to_string(),
            format!("{action:?}"),
            format!("{action:#?}"),
            format!("{:?}", Some(action.clone())),
            format!("{:?}", vec![action.clone()]),
        ] {
            assert!(
                shows_no_part_of(&rendered, secret),
                "a part of the text shows in {rendered}"
            );
        }

        let emoji = BindAction::Text(Arc::from("\u{1F680}\u{1F30D} \u{451}\u{436}"));
        assert_eq!(emoji.to_string(), "text (5 chars)", "characters, not bytes");
    }

    #[test]
    fn a_cloned_text_action_shares_its_text() {
        let action = BindAction::Text(Arc::from("Kind regards,"));
        let cloned = action.clone();
        let (BindAction::Text(original), BindAction::Text(copy)) = (&action, &cloned) else {
            panic!("a clone of a Text is a Text");
        };
        assert!(Arc::ptr_eq(original, copy), "the clone copied the text");
    }

    #[test]
    fn text_actions_are_equal_when_their_texts_are() {
        let a = Arc::<str>::from("a");
        let another_a = Arc::<str>::from("a");
        assert!(!Arc::ptr_eq(&a, &another_a));
        assert_eq!(BindAction::Text(a), BindAction::Text(another_a));
        assert_ne!(
            BindAction::Text(Arc::from("a")),
            BindAction::Text(Arc::from("b"))
        );
        assert_ne!(
            BindAction::Text(Arc::from("a")),
            BindAction::OpenUrl("a".to_owned())
        );
    }

    #[test]
    fn a_text_action_has_no_system_function_or_hotkey() {
        let action = BindAction::Text(Arc::from("x"));
        assert!(action.to_system_function().is_none());
        assert_eq!(action.get_system_combination(), None);
    }

    #[test]
    fn every_other_action_debugs_as_the_derive_did() {
        let cases = [
            (BindAction::SwitchLanguage, "SwitchLanguage".to_owned()),
            (
                BindAction::SwitchLanguageBackward,
                "SwitchLanguageBackward".to_owned(),
            ),
            (BindAction::ToggleCapsLock, "ToggleCapsLock".to_owned()),
            (BindAction::DoNothing, "DoNothing".to_owned()),
            (
                BindAction::PressKey(VK_BACK),
                format!("PressKey({VK_BACK:?})"),
            ),
            (
                BindAction::PostMessage {
                    msg: WM_CLOSE,
                    wparam: 7,
                    lparam: -3,
                },
                "PostMessage { msg: 16, wparam: 7, lparam: -3 }".to_owned(),
            ),
            (
                BindAction::Sequence(Arc::new(Vec::new())),
                "Sequence([])".to_owned(),
            ),
            (
                BindAction::Launch {
                    exe: "wt.exe".to_owned(),
                    args: Vec::new(),
                },
                r#"Launch { exe: "wt.exe", args: [] }"#.to_owned(),
            ),
            (
                BindAction::OpenUrl("u".to_owned()),
                r#"OpenUrl("u")"#.to_owned(),
            ),
            (
                BindAction::Media(MediaKey::PlayPause),
                format!("Media({:?})", MediaKey::PlayPause),
            ),
        ];
        for (action, expected) in cases {
            assert_eq!(format!("{action:?}"), expected);
        }
    }
}
