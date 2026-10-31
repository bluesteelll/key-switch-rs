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

use std::fmt;

use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};

// Every container with named fields denies unknown ones: a misspelled field
// would otherwise be skipped and its default used silently (KS-22).

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawConfig {
    /// Top-level list of bindings. Required: a missing or misspelled
    /// `bindings` is a load error (KS-22). An explicit `bindings: []`
    /// loads, with a warning, so every binding can be commented out.
    pub bindings: Vec<RawBinding>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Whether the key events that fire or arm a `keys` binding are kept
    /// from Windows, for every trigger. Missing: `false` for a `Tap`, `Hold`
    /// or `DoubleTap` binding made only of modifiers (`"LShift"`), so that
    /// the modifier keeps working, and `true` for every other binding
    /// (`Binding::blocks_by_default`), which an `Option` is needed to tell
    /// from an explicit value. `sequence` and `chord` bindings ignore it.
    #[serde(default)]
    pub block_default: Option<bool>,

    /// Whether the loader adds an auto-blocker for the Windows hotkey of the
    /// action's system function, and what it does. Missing or `false`: none.
    /// See [`RawBlockOriginalCombo`].
    #[serde(default)]
    pub block_original_combo: RawBlockOriginalCombo,

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

    /// Lets a `keys` binding fire while modifiers it does not name are held
    /// too. Missing or `false`: the modifiers held (Shift, Ctrl, Alt, Win;
    /// either key of a pair) must be exactly the binding's own. The loader
    /// rejects `true` on `sequence` and `chord` bindings.
    #[serde(default)]
    pub allow_extra_modifiers: bool,

    /// Whether a `keys` binding with the `Immediate` trigger fires again on
    /// every auto-repeat of its key. Missing: the action's default
    /// (`BindAction::repeats_by_default`), which an `Option` is needed to
    /// tell from an explicit `false`. The loader rejects `true` on deferred
    /// triggers, sequences and chords, which never fire on an auto-repeat.
    #[serde(default)]
    pub repeat: Option<bool>,
}

/// `block_original_combo: true | false | MaskOnly`, mirrored by
/// `Option<BlockerMode>` on [`crate::data::binding::Binding`]. An enum, not
/// a `bool`, so that `true` and `false` keep loading as they always did:
/// RON reads a variant name as an identifier, and `true` and `false` are
/// identifiers there. The declaration order is the order a misspelled
/// value's error lists them in: ``expected one of `true`, `false`, or
/// `MaskOnly` instead``.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
pub(crate) enum RawBlockOriginalCombo {
    /// An auto-blocker that swallows the hotkey pressed on its own.
    #[serde(rename = "true")]
    On,
    /// No auto-blocker.
    #[default]
    #[serde(rename = "false")]
    Off,
    /// Experimental: an auto-blocker that lets the hotkey through and sends
    /// the menu mask key when it is pressed on its own.
    MaskOnly,
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
#[serde(deny_unknown_fields)]
pub(crate) struct SequenceSpec {
    pub steps: Vec<String>,
    #[serde(default = "default_seq_gap_ms")]
    pub max_gap_ms: u64,
}

/// Simultaneous chord (all keys go down within a tight window).
/// Used as `chord: (keys: ["j", "k"])` at the binding level.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
/// `deny_unknown_fields` applies to the struct variants `PostMessage` and
/// `Launch`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// `Text("Kind regards,")` - types the text in one batch; see
    /// `BindAction::Text`.
    Text(String),
    /// `Paste("Kind regards,")` or `Paste(text: "...", keys: "Shift+Insert",
    /// restore_delay: 800)` - pastes the text through the clipboard; see
    /// [`RawPaste`].
    Paste(RawPaste),
    /// `Media(PlayPause)` / `Media(VolumeUp)` / ...
    Media(MediaKeyRef),
    DoNothing,
}

/// The payload of `Paste(...)`, the action and the macro step: a string,
/// or a struct whose fields are `text` (required), `keys` and
/// `restore_delay`. The loader's `unwrap_variant_newtypes` lets the struct
/// go straight inside `Paste(...)`, so both forms reach this type through
/// one newtype variant; `deserialize_any` tells them apart. `keys` and
/// `restore_delay` take a plain value: `Some(...)` and `None` are not
/// allowed there. The phrase may be a secret, so `Debug` shows only its
/// length and no error text quotes it.
pub(crate) struct RawPaste {
    pub text: String,
    pub keys: Option<String>,
    pub restore_delay: Option<u64>,
}

/// The fields of the named form of `Paste`, in the order ron lists them.
const PASTE_FIELDS: &[&str] = &["text", "keys", "restore_delay"];

impl<'de> Deserialize<'de> for RawPaste {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PasteVisitor)
    }
}

struct PasteVisitor;

impl<'de> Visitor<'de> for PasteVisitor {
    type Value = RawPaste;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(r#"a string, or (text: "...", keys: "...", restore_delay: ...)"#)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<RawPaste, E> {
        self.visit_string(text.to_owned())
    }

    fn visit_string<E: de::Error>(self, text: String) -> Result<RawPaste, E> {
        Ok(RawPaste {
            text,
            keys: None,
            restore_delay: None,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<RawPaste, A::Error> {
        let mut text = None;
        let mut keys = None;
        let mut restore_delay = None;
        while let Some(field) = map.next_key::<PasteField>()? {
            match field {
                PasteField::Text => {
                    if text.is_some() {
                        return Err(de::Error::duplicate_field("text"));
                    }
                    text = Some(map.next_value::<String>()?);
                }
                PasteField::Keys => {
                    if keys.is_some() {
                        return Err(de::Error::duplicate_field("keys"));
                    }
                    keys = Some(map.next_value::<String>()?);
                }
                PasteField::RestoreDelay => {
                    if restore_delay.is_some() {
                        return Err(de::Error::duplicate_field("restore_delay"));
                    }
                    restore_delay = Some(map.next_value::<u64>()?);
                }
            }
        }
        Ok(RawPaste {
            text: text.ok_or_else(|| de::Error::missing_field("text"))?,
            keys,
            restore_delay,
        })
    }
}

/// A field name of the named form of `Paste`.
enum PasteField {
    Text,
    Keys,
    RestoreDelay,
}

impl<'de> Deserialize<'de> for PasteField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_identifier(PasteFieldVisitor)
    }
}

struct PasteFieldVisitor;

impl Visitor<'_> for PasteFieldVisitor {
    type Value = PasteField;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("`text`, `keys` or `restore_delay`")
    }

    fn visit_str<E: de::Error>(self, name: &str) -> Result<PasteField, E> {
        match name {
            "text" => Ok(PasteField::Text),
            "keys" => Ok(PasteField::Keys),
            "restore_delay" => Ok(PasteField::RestoreDelay),
            _ => Err(de::Error::unknown_field(name, PASTE_FIELDS)),
        }
    }
}

impl fmt::Debug for RawPaste {
    /// Written out so that the phrase never reaches a `{:?}`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RawPaste {{ text: <{} chars>, keys: {:?}, restore_delay: {:?} }}",
            self.text.chars().count(),
            self.keys,
            self.restore_delay
        )
    }
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
/// `Window(Restore)`, `Delay(100)`, `Text("hi")`, `Paste("hi")`,
/// `Key("Enter")`, `Combo("Ctrl+S")`, `Launch(...)`, `OpenUrl("...")`,
/// `Media(PlayPause)`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum RawStep {
    Window(RawWindowKind),
    Delay(u64),
    Text(String),
    /// The same two forms as the action; see [`RawPaste`].
    Paste(RawPaste),
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
