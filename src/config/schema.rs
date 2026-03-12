//! Serde-facing schema for `config.ron`.
//!
//! The on-disk shape is intentionally close to — but not identical to —
//! the runtime `BindAction`/`SequenceStep`/`Binding` types. The gap is the
//! VK-bearing fields: at runtime we hold `VIRTUAL_KEY` / `Vec<VIRTUAL_KEY>`,
//! but those come from the `windows` crate and have no `Deserialize` impl.
//! The user-facing form keeps them as human-readable strings (`"Enter"`,
//! `"Ctrl+S"`); the loader parses them into VKs after deserialization.
//!
//! Everything else (action variants, sequence steps, the `Window(...)`
//! sub-enum) deserializes directly into Rust enum literals — no tag fields,
//! no `untagged` discrimination, no `rename_all`. RON's grammar already
//! understands `Variant`, `Variant(arg)`, and `Variant { field: ... }`,
//! which is the whole point of choosing it over TOML.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct RawConfig {
    /// Top-level list of bindings. Empty / missing is OK (loader warns but
    /// doesn't fail), so iterating on the file never crashes the daemon.
    #[serde(default)]
    pub bindings: Vec<RawBinding>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawBinding {
    /// Combo-style binding. Mutually exclusive with `sequence` and `chord`.
    /// The loader rejects bindings that set zero or more than one of these.
    #[serde(default)]
    pub keys: Option<String>,
    #[serde(default)]
    pub sequence: Option<SequenceSpec>,
    #[serde(default)]
    pub chord: Option<ChordSpec>,
    pub action: RawAction,

    #[serde(default = "default_true")]
    pub block_default: bool,

    #[serde(default)]
    pub block_original_combo: bool,

    /// Foreground-window guard. Missing or `Always` means the binding fires
    /// unconditionally. Anything else gates the binding on focused app /
    /// title — see [`RawCondition`].
    #[serde(default)]
    pub when: RawCondition,

    /// Resolution mode for *when* the action fires given the combo just
    /// triggered. Missing or `Immediate` means the existing "fire on
    /// key-down" behaviour; `Tap`/`Hold`/`DoubleTap` defer to the
    /// gesture state machine. See [`RawTrigger`].
    #[serde(default)]
    pub trigger: RawTrigger,
}

/// On-disk mirror of [`crate::data::trigger::Trigger`]. The tuple argument
/// for the deferred variants is the term in milliseconds — RON literal form
/// `Tap(200)`, `Hold(200)`, `DoubleTap(250)`.
#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub(crate) enum RawTrigger {
    #[default]
    Immediate,
    Tap(u64),
    Hold(u64),
    DoubleTap(u64),
}

/// Vim/Emacs-style leader sequence. Used as `sequence: (steps: ["g", "g"])`
/// at the binding level (RON struct literal).
#[derive(Debug, Deserialize)]
pub(crate) struct SequenceSpec {
    pub steps: Vec<String>,
    #[serde(default = "default_seq_gap_ms")]
    pub max_gap_ms: u64,
}

/// Simultaneous chord (all keys go down within a tight window).
/// Used as `chord: (keys: ["j", "k"])` at the binding level.
#[derive(Debug, Deserialize)]
pub(crate) struct ChordSpec {
    pub keys: Vec<String>,
    #[serde(default = "default_chord_window_ms")]
    pub window_ms: u64,
}

fn default_seq_gap_ms() -> u64 {
    500
}

fn default_chord_window_ms() -> u64 {
    50
}

/// Predicate against the focused window's exe name / title. On-disk mirror
/// of [`crate::data::condition::Condition`].
///
/// RON syntax:
/// ```ron
/// when: AppEquals("code.exe"),
/// when: TitleContains("Spotify"),
/// when: Not(AppEquals("chrome.exe")),
/// when: And([AppEquals("code.exe"), Not(TitleContains("Settings"))]),
/// when: Or([AppEquals("chrome.exe"), AppEquals("firefox.exe")]),
/// ```
#[derive(Debug, Deserialize, Default)]
pub(crate) enum RawCondition {
    #[default]
    Always,
    AppEquals(String),
    TitleContains(String),
    TitleEquals(String),
    Not(Box<RawCondition>),
    And(Vec<RawCondition>),
    Or(Vec<RawCondition>),
}

/// `BindAction` mirror with VK fields kept as strings until the loader runs.
#[derive(Debug, Deserialize)]
pub(crate) enum RawAction {
    SwitchLanguage,
    SwitchLanguageBackward,
    ToggleCapsLock,
    /// `PressKey("L")` — single key by human name.
    PressKey(String),
    /// `PostMessage(msg: "WM_CLOSE", wparam: 0, lparam: 0)`. `wparam` /
    /// `lparam` default to 0.
    PostMessage {
        msg: MessageRef,
        #[serde(default)]
        wparam: u64,
        #[serde(default)]
        lparam: i64,
    },
    /// `Sequence([Window(Restore), Delay(100), Text("hi"), ...])`.
    Sequence(Vec<RawStep>),
    /// `Launch(exe: "notepad.exe")` or `Launch(exe: "code.exe", args: ["D:\\"])`.
    Launch {
        exe: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// `OpenUrl("https://...")` / `OpenUrl("mailto:foo@bar")` /
    /// `OpenUrl("C:\\path\\to\\file.txt")` — anything `ShellExecuteW` knows.
    OpenUrl(String),
    /// `Media(PlayPause)` / `Media(VolumeUp)` / ...
    Media(MediaKeyRef),
    DoNothing,
}

/// On-disk representation of the runtime `MediaKey` enum. Same variants;
/// kept separate so the runtime enum can live in `core::windows_actions`
/// without depending on serde.
#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) enum MediaKeyRef {
    PlayPause,
    Stop,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    VolumeMute,
}

/// Either a symbolic `"WM_CLOSE"` (string) or a numeric `0x10` (integer).
/// `untagged` because RON's grammar lets us put either kind of literal
/// directly in the field — no `Name(...)` / `Code(...)` wrapper required.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum MessageRef {
    Code(u32),
    Name(String),
}

/// One step inside a `Sequence(...)`. Direct enum-variant syntax in RON:
/// `Window(Restore)`, `Delay(100)`, `Text("hi")`, `Key("Enter")`,
/// `Combo("Ctrl+S")`, `Launch(...)`, `OpenUrl("...")`, `Media(PlayPause)`.
#[derive(Debug, Deserialize)]
pub(crate) enum RawStep {
    Window(RawWindowKind),
    Delay(u64),
    Text(String),
    Key(String),
    Combo(String),
    Launch {
        exe: String,
        #[serde(default)]
        args: Vec<String>,
    },
    OpenUrl(String),
    Media(MediaKeyRef),
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) enum RawWindowKind {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn default_true() -> bool {
    true
}
