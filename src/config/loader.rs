//! Disk I/O and conversion from on-disk `RawConfig` (RON) to a `Vec<Binding>`
//! ready for `App`. Default-config generation lives here so first run is
//! zero-touch.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use std::time::Duration;

use crate::core::windows_actions::{BindAction, MediaKey};
use crate::data::binding::{Binding, BindingKind};
use crate::data::condition::Condition;
use crate::data::sequence::{SequenceStep, WindowOp};
use crate::data::trigger::Trigger;
use crate::data::vk_name::parse_vk;

use super::parsing::{parse_combo, parse_wm_name};
use super::schema::{
    ChordSpec, MediaKeyRef, MessageRef, RawAction, RawBinding, RawCondition, RawConfig,
    RawStep, RawTrigger, RawWindowKind, SequenceSpec,
};

/// Bundled default `config.ron`. Written to disk the first time the program
/// runs without an existing config.
const DEFAULT_CONFIG: &str = include_str!("default_config.ron");

#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        // Boxed because RON's `SpannedError` carries line/column data that
        // adds up; keep `Result<_, ConfigError>` small (clippy
        // `result_large_err`).
        source: Box<ron::error::SpannedError>,
    },
    /// One or more individual binding entries failed validation. We collect
    /// every failure rather than bail on the first so the user sees the full
    /// list in one pass.
    Bindings(Vec<String>),
    /// Anything else (e.g. `current_exe()` returned a path with no parent).
    Other(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io { path, source } => {
                write!(f, "config I/O error ({}): {}", path.display(), source)
            }
            ConfigError::Parse { path, source } => {
                write!(f, "config parse error ({}): {}", path.display(), source)
            }
            ConfigError::Bindings(errs) => {
                writeln!(f, "config has {} invalid binding(s):", errs.len())?;
                for e in errs {
                    writeln!(f, "  - {}", e)?;
                }
                Ok(())
            }
            ConfigError::Other(s) => write!(f, "config error: {}", s),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io { source, .. } => Some(source),
            ConfigError::Parse { source, .. } => Some(&**source),
            _ => None,
        }
    }
}

/// Path next to the running executable (`<exe_dir>/config.ron`).
pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    let exe = std::env::current_exe().map_err(|e| ConfigError::Io {
        path: PathBuf::from("<current_exe>"),
        source: e,
    })?;
    let dir = exe
        .parent()
        .ok_or_else(|| ConfigError::Other(format!(
            "executable path {:?} has no parent directory",
            exe
        )))?;
    Ok(dir.join("config.ron"))
}

/// Loads bindings from `path`, generating the default file there if it does
/// not yet exist. The "missing → write default" branch is by design: the
/// first run of the binary should produce a working setup without forcing
/// the user to author a config by hand.
pub fn load(path: &Path) -> Result<Vec<Binding>, ConfigError> {
    if !path.exists() {
        write_default(path)?;
        println!("[INFO] generated default config at {}", path.display());
    } else {
        println!("[INFO] loading config from {}", path.display());
    }

    let text = fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    from_ron_str(&text).map_err(|e| match e {
        // Attach the path for I/O-less variants so error messages are useful.
        ConfigError::Parse { source, .. } => ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        },
        other => other,
    })
}

/// Pure (no-I/O) conversion from RON text to `Vec<Binding>`. Kept available
/// to the crate so tests can exercise the schema without touching the disk.
///
/// The returned list is already expanded with auto-blocker bindings for any
/// entries that set `block_original_combo: true`. Callers can hand the list
/// straight to `KeyboardHook::update_bindings`.
pub(crate) fn from_ron_str(text: &str) -> Result<Vec<Binding>, ConfigError> {
    // `IMPLICIT_SOME` lets users write `keys: "CapsLock"` instead of
    // `keys: Some("CapsLock")` for the `Option<String>` fields on
    // `RawBinding`. Standard ergonomic move when an Option is exposed in a
    // human-edited config — no one expects to type `Some(...)` everywhere.
    let options = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

    let raw: RawConfig = options.from_str(text).map_err(|e| ConfigError::Parse {
        path: PathBuf::from("<in-memory>"),
        source: Box::new(e),
    })?;

    let mut user_bindings = Vec::with_capacity(raw.bindings.len());
    let mut errors = Vec::new();

    for (index, rb) in raw.bindings.iter().enumerate() {
        match raw_binding_to_binding(rb, index) {
            Ok(b) => user_bindings.push(b),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        return Err(ConfigError::Bindings(errors));
    }

    if user_bindings.is_empty() {
        eprintln!("[WARN] config contains no bindings");
    }

    Ok(expand_with_auto_blockers(user_bindings))
}

/// Inserts a no-op binding for the OS-default hotkey of any binding that
/// declared `block_original_combo: true`. The auto-blocker silences Windows'
/// own behaviour for that combo so the user's remap is the only thing the
/// foreground app sees.
///
/// Idempotent: if two user bindings demand the same auto-blocker (or if a
/// user binding already covers the system combo), the auto-blocker is added
/// only once.
fn expand_with_auto_blockers(user_bindings: Vec<Binding>) -> Vec<Binding> {
    let mut result: Vec<Binding> = Vec::with_capacity(user_bindings.len());

    for binding in user_bindings {
        if binding.block_original_combo
            && let Some(system_combo) = binding.action.get_system_combination()
        {
            // Only `Combo`-kind bindings can collide with an auto-blocker
            // (sequence/chord don't represent a single hold-this-set combo).
            let already_present = result
                .iter()
                .any(|b| matches!(b.combination(), Some(c) if *c == system_combo));
            if !already_present {
                result.push(Binding::new_auto_blocker(system_combo));
            }
        }
        result.push(binding);
    }

    result
}

fn write_default(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    fs::write(path, DEFAULT_CONFIG).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn raw_binding_to_binding(raw: &RawBinding, index: usize) -> Result<Binding, String> {
    let keys_display = format_raw_binding_keys(raw);
    let err_prefix = format!("binding #{} [{}]", index, keys_display);

    let kind = raw_binding_to_kind(raw)
        .map_err(|e| format!("{}: {}", err_prefix, e))?;

    let action = raw_action_to_bind_action(&raw.action)
        .map_err(|e| format!("{}: {}", err_prefix, e))?;

    let trigger = raw_trigger_to_trigger(raw.trigger);

    // Sequence/Chord have their own temporal semantics; pairing them with
    // a deferred Tap/Hold/DoubleTap trigger would double-up state machines
    // and produce undefined behaviour. Reject at load.
    if !matches!(kind, BindingKind::Combo(_)) && !trigger.is_immediate() {
        return Err(format!(
            "{}: Sequence/Chord cannot use a deferred trigger ({:?}); \
             only `Immediate` (the default) is valid for these kinds",
            err_prefix, trigger
        ));
    }

    let binding = Binding::with_kind(kind, action)
        .with_block_default(raw.block_default)
        .with_block_original_combo(raw.block_original_combo)
        .with_condition(raw_condition_to_condition(&raw.when))
        .with_trigger(trigger);

    Ok(binding)
}

fn raw_binding_to_kind(raw: &RawBinding) -> Result<BindingKind, String> {
    // Exactly one of {keys, sequence, chord} must be set.
    let kind_count = (raw.keys.is_some() as u8)
        + (raw.sequence.is_some() as u8)
        + (raw.chord.is_some() as u8);
    if kind_count == 0 {
        return Err(
            "binding has none of `keys` / `sequence` / `chord` set — must specify exactly one"
                .into(),
        );
    }
    if kind_count > 1 {
        return Err(
            "binding has multiple of `keys` / `sequence` / `chord` set — must be exactly one"
                .into(),
        );
    }

    if let Some(s) = &raw.keys {
        return parse_combo(s).map(BindingKind::Combo);
    }

    if let Some(seq) = &raw.sequence {
        return sequence_spec_to_kind(seq);
    }

    if let Some(chord) = &raw.chord {
        return chord_spec_to_kind(chord);
    }

    unreachable!("kind_count checked above")
}

fn sequence_spec_to_kind(seq: &SequenceSpec) -> Result<BindingKind, String> {
    if seq.steps.len() < 2 {
        return Err(
            "Sequence must have at least 2 steps; for a single combo use plain `keys: \"...\"`"
                .into(),
        );
    }
    let mut parsed = Vec::with_capacity(seq.steps.len());
    for (i, step) in seq.steps.iter().enumerate() {
        let c = parse_combo(step).map_err(|e| format!("Sequence step #{}: {}", i, e))?;
        parsed.push(c);
    }
    Ok(BindingKind::Sequence {
        steps: parsed,
        max_gap: Duration::from_millis(seq.max_gap_ms),
    })
}

fn chord_spec_to_kind(chord: &ChordSpec) -> Result<BindingKind, String> {
    if chord.keys.len() < 2 {
        return Err("Chord must have at least 2 keys; a single key is not a chord".into());
    }
    let mut vks = Vec::with_capacity(chord.keys.len());
    for (i, k) in chord.keys.iter().enumerate() {
        let vk = parse_vk(k)
            .ok_or_else(|| format!("Chord key #{}: unknown key name {:?}", i, k))?;
        if vks.contains(&vk) {
            return Err(format!("Chord key #{}: duplicate {:?}", i, k));
        }
        vks.push(vk);
    }
    Ok(BindingKind::Chord {
        keys: vks,
        window: Duration::from_millis(chord.window_ms),
    })
}

fn format_raw_binding_keys(raw: &RawBinding) -> String {
    if let Some(s) = &raw.keys {
        return s.clone();
    }
    if let Some(seq) = &raw.sequence {
        return format!("Sequence[{}]", seq.steps.join(" → "));
    }
    if let Some(chord) = &raw.chord {
        return format!("Chord[{}]", chord.keys.join("+"));
    }
    "<unset>".into()
}

fn raw_trigger_to_trigger(raw: RawTrigger) -> Trigger {
    match raw {
        RawTrigger::Immediate    => Trigger::Immediate,
        RawTrigger::Tap(ms)      => Trigger::Tap(ms),
        RawTrigger::Hold(ms)     => Trigger::Hold(ms),
        RawTrigger::DoubleTap(ms) => Trigger::DoubleTap(ms),
    }
}

fn raw_condition_to_condition(raw: &RawCondition) -> Condition {
    match raw {
        RawCondition::Always => Condition::Always,
        RawCondition::AppEquals(s) => Condition::AppEquals(s.clone()),
        RawCondition::TitleContains(s) => Condition::TitleContains(s.clone()),
        RawCondition::TitleEquals(s) => Condition::TitleEquals(s.clone()),
        RawCondition::Not(inner) => {
            Condition::Not(Box::new(raw_condition_to_condition(inner)))
        }
        RawCondition::And(parts) => {
            Condition::And(parts.iter().map(raw_condition_to_condition).collect())
        }
        RawCondition::Or(parts) => {
            Condition::Or(parts.iter().map(raw_condition_to_condition).collect())
        }
    }
}

fn raw_action_to_bind_action(raw: &RawAction) -> Result<BindAction, String> {
    Ok(match raw {
        RawAction::SwitchLanguage         => BindAction::SwitchLanguage,
        RawAction::SwitchLanguageBackward => BindAction::SwitchLanguageBackward,
        RawAction::ToggleCapsLock         => BindAction::ToggleCapsLock,
        RawAction::DoNothing              => BindAction::DoNothing,

        RawAction::PressKey(key) => {
            let vk = parse_vk(key)
                .ok_or_else(|| format!("PressKey: unknown key {:?}", key))?;
            BindAction::PressKey(vk)
        }

        RawAction::PostMessage { msg, wparam, lparam } => {
            let code = match msg {
                MessageRef::Code(c) => *c,
                MessageRef::Name(n) => parse_wm_name(n).ok_or_else(|| {
                    format!(
                        "PostMessage: unknown WM_* name {:?} \
                         (use a numeric code if it is not in the built-in table)",
                        n
                    )
                })?,
            };
            // u64 -> usize / i64 -> isize: on Windows we only build for
            // 64-bit targets; widening is a no-op there. The cast is in one
            // place precisely so it can be revisited if we ever ship 32-bit.
            BindAction::PostMessage {
                msg: code,
                wparam: *wparam as usize,
                lparam: *lparam as isize,
            }
        }

        RawAction::Sequence(steps) => {
            // Validate every step up-front so the user sees a clean per-step
            // error before any binding starts firing. Wrap in Arc so cloning
            // the BindAction (frozen binding list, hot-path lookups) is O(1).
            let mut converted = Vec::with_capacity(steps.len());
            for (i, raw_step) in steps.iter().enumerate() {
                converted.push(
                    raw_step_to_step(raw_step)
                        .map_err(|e| format!("step #{}: {}", i, e))?,
                );
            }
            BindAction::Sequence(Arc::new(converted))
        }

        RawAction::Launch { exe, args } => BindAction::Launch {
            exe: exe.clone(),
            args: args.clone(),
        },

        RawAction::OpenUrl(url) => BindAction::OpenUrl(url.clone()),

        RawAction::Media(key) => BindAction::Media(media_key_ref_to_key(*key)),
    })
}

fn media_key_ref_to_key(raw: MediaKeyRef) -> MediaKey {
    match raw {
        MediaKeyRef::PlayPause  => MediaKey::PlayPause,
        MediaKeyRef::Stop       => MediaKey::Stop,
        MediaKeyRef::Next       => MediaKey::Next,
        MediaKeyRef::Previous   => MediaKey::Previous,
        MediaKeyRef::VolumeUp   => MediaKey::VolumeUp,
        MediaKeyRef::VolumeDown => MediaKey::VolumeDown,
        MediaKeyRef::VolumeMute => MediaKey::VolumeMute,
    }
}

fn raw_step_to_step(raw: &RawStep) -> Result<SequenceStep, String> {
    Ok(match raw {
        RawStep::Delay(ms) => SequenceStep::Delay(*ms),

        RawStep::Text(text) => SequenceStep::TypeText(text.clone()),

        RawStep::Key(name) => {
            let vk = parse_vk(name)
                .ok_or_else(|| format!("Key: unknown key {:?}", name))?;
            SequenceStep::PressKey(vk)
        }

        RawStep::Combo(combo) => {
            let kc = parse_combo(combo)
                .map_err(|e| format!("Combo {:?}: {}", combo, e))?;
            if kc.keys.is_empty() {
                return Err(format!("Combo {:?}: empty", combo));
            }
            SequenceStep::PressCombo(kc.keys)
        }

        RawStep::Window(kind) => SequenceStep::Window(match kind {
            RawWindowKind::Minimize => WindowOp::Minimize,
            RawWindowKind::Maximize => WindowOp::Maximize,
            RawWindowKind::Restore  => WindowOp::Restore,
            RawWindowKind::Close    => WindowOp::Close,
        }),

        RawStep::Launch { exe, args } => SequenceStep::Launch {
            exe: exe.clone(),
            args: args.clone(),
        },

        RawStep::OpenUrl(url) => SequenceStep::OpenUrl(url.clone()),

        RawStep::Media(key) => SequenceStep::Media(media_key_ref_to_key(*key)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::WM_CLOSE;

    #[test]
    fn default_config_parses() {
        // Sanity: the bundled default must always be valid and produce at
        // least one binding. Caught at compile time via include_str! +
        // exercised here.
        let bindings = from_ron_str(DEFAULT_CONFIG).expect("default parses");
        assert!(!bindings.is_empty(), "default config must have bindings");
    }

    #[test]
    fn parse_simple_binding() {
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "CapsLock",
                        action: SwitchLanguage,
                        block_original_combo: true,
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        // 2 entries: the user binding + the auto-blocker inserted because of
        // block_original_combo. Order: auto-blocker first, then user.
        assert_eq!(bindings.len(), 2);

        let user = bindings.iter().find(|b| !b.is_auto_blocker).unwrap();
        assert_eq!(user.combination().unwrap().keys, vec![VK_CAPITAL]);
        assert_eq!(user.action, BindAction::SwitchLanguage);
        assert!(user.block_original_combo);
        assert!(user.block_default); // default

        let auto = bindings.iter().find(|b| b.is_auto_blocker).unwrap();
        assert_eq!(auto.action, BindAction::DoNothing);
    }

    #[test]
    fn auto_blocker_not_added_without_flag() {
        // Same binding without block_original_combo: no auto-blocker.
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "CapsLock", action: SwitchLanguage ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert_eq!(bindings.len(), 1);
        assert!(!bindings[0].is_auto_blocker);
    }

    #[test]
    fn auto_blocker_not_added_for_non_system_actions() {
        // PressKey has no underlying system function -> nothing to auto-block.
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "F13",
                        action: PressKey("L"),
                        block_original_combo: true,
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert_eq!(bindings.len(), 1);
        assert!(!bindings[0].is_auto_blocker);
    }

    #[test]
    fn auto_blocker_deduped_when_two_bindings_want_the_same_combo() {
        // Two user bindings, both asking for the same system auto-block.
        // The auto-blocker should be inserted only once.
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "CapsLock",
                        action: SwitchLanguage,
                        block_original_combo: true,
                    ),
                    (
                        keys: "F13",
                        action: SwitchLanguageBackward,
                        block_original_combo: true,
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        // 2 user + 1 auto-blocker = 3, not 4.
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings.iter().filter(|b| b.is_auto_blocker).count(), 1);
    }

    #[test]
    fn parse_press_key_action() {
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "Ctrl+Alt+L",
                        action: PressKey("L"),
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert_eq!(bindings[0].action, BindAction::PressKey(VK_L));
    }

    #[test]
    fn parse_post_message_by_name() {
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "Win+Q",
                        action: PostMessage(msg: "WM_CLOSE"),
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert_eq!(
            bindings[0].action,
            BindAction::PostMessage { msg: WM_CLOSE, wparam: 0, lparam: 0 },
        );
    }

    #[test]
    fn parse_post_message_by_numeric_code() {
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "Win+Q",
                        action: PostMessage(msg: 0x10, wparam: 42, lparam: -1),
                    ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        match bindings[0].action {
            BindAction::PostMessage { msg, wparam, lparam } => {
                assert_eq!(msg, 0x10);
                assert_eq!(wparam, 42);
                assert_eq!(lparam, -1);
            }
            _ => panic!("expected PostMessage"),
        }
    }

    #[test]
    fn parse_multiple_bindings() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "CapsLock",       action: SwitchLanguage ),
                    ( keys: "Shift+CapsLock", action: ToggleCapsLock ),
                    ( keys: "F13",            action: DoNothing, block_default: false ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0].action, BindAction::SwitchLanguage);
        assert_eq!(bindings[1].action, BindAction::ToggleCapsLock);
        assert_eq!(bindings[2].action, BindAction::DoNothing);
        assert!(!bindings[2].block_default);
    }

    #[test]
    fn unknown_key_in_combo_is_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Bogus", action: SwitchLanguage ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        assert!(matches!(err, ConfigError::Bindings(_)));
    }

    #[test]
    fn unknown_action_variant_is_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "CapsLock", action: FlyToMars ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn unknown_press_key_target_is_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+A", action: PressKey("Bogus") ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        if let ConfigError::Bindings(errs) = err {
            assert!(errs[0].contains("Bogus"), "error should mention bogus key: {:?}", errs);
        } else {
            panic!("expected Bindings error, got {:?}", err);
        }
    }

    #[test]
    fn empty_config_is_ok_but_warned() {
        let ron_text = r#"( bindings: [] )"#;
        let bindings = from_ron_str(ron_text).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn all_per_binding_errors_collected() {
        // Two independent failures: report both, not just the first.
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Bogus1", action: SwitchLanguage ),
                    ( keys: "Ctrl+Bogus2", action: SwitchLanguage ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        if let ConfigError::Bindings(errs) = err {
            assert_eq!(errs.len(), 2, "expected both failures: {:?}", errs);
        } else {
            panic!("expected Bindings error, got {:?}", err);
        }
    }

    // ---- sequence tests ----

    #[test]
    fn parse_sequence_all_step_kinds() {
        let ron_text = r#"
            (
                bindings: [
                    (
                        keys: "Ctrl+Alt+N",
                        action: Sequence([
                            Window(Restore),
                            Delay(100),
                            Text("Hello 🚀"),
                            Key("Enter"),
                            Combo("Ctrl+S"),
                            Delay(500),
                            Window(Minimize),
                        ]),
                    ),
                ],
            )
        "#;

        let bindings = from_ron_str(ron_text).unwrap();
        let action = &bindings[0].action;
        let steps = match action {
            BindAction::Sequence(s) => s,
            _ => panic!("expected Sequence, got {:?}", action),
        };

        assert_eq!(steps.len(), 7);

        assert!(matches!(steps[0], SequenceStep::Window(WindowOp::Restore)));
        assert!(matches!(steps[1], SequenceStep::Delay(100)));
        if let SequenceStep::TypeText(t) = &steps[2] {
            assert_eq!(t, "Hello 🚀");
        } else {
            panic!("expected TypeText, got {:?}", steps[2]);
        }
        assert!(matches!(steps[3], SequenceStep::PressKey(vk) if vk == VK_RETURN));
        if let SequenceStep::PressCombo(keys) = &steps[4] {
            assert!(keys.contains(&VK_CONTROL));
            assert!(keys.contains(&VK_S));
        } else {
            panic!("expected PressCombo, got {:?}", steps[4]);
        }
        assert!(matches!(steps[5], SequenceStep::Delay(500)));
        assert!(matches!(steps[6], SequenceStep::Window(WindowOp::Minimize)));
    }

    #[test]
    fn parse_empty_sequence_is_ok() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Alt+E", action: Sequence([]) ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        match &bindings[0].action {
            BindAction::Sequence(s) => assert_eq!(s.len(), 0),
            _ => panic!(),
        }
    }

    #[test]
    fn sequence_unknown_step_variant_is_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Alt+X", action: Sequence([Fly(100)]) ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn sequence_unknown_window_variant_is_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Alt+X", action: Sequence([Window(Explode)]) ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn sequence_unknown_key_in_step_is_bindings_error() {
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "Ctrl+Alt+X", action: Sequence([Key("Bogus")]) ),
                ],
            )
        "#;
        let err = from_ron_str(ron_text).unwrap_err();
        match err {
            ConfigError::Bindings(errs) => {
                assert!(
                    errs[0].contains("Bogus") && errs[0].contains("step #0"),
                    "error should mention step #0 and the bad key: {:?}",
                    errs,
                );
            }
            other => panic!("expected Bindings, got {:?}", other),
        }
    }

    // ---- Launch / OpenUrl / Media tests ----

    #[test]
    fn parse_launch_no_args() {
        let ron = r#"
            ( bindings: [( keys: "Win+T", action: Launch(exe: "wt.exe") )] )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(
            bindings[0].action,
            BindAction::Launch {
                exe: "wt.exe".into(),
                args: vec![],
            }
        );
    }

    #[test]
    fn parse_launch_with_args() {
        let ron = r#"
            (
                bindings: [(
                    keys: "Win+C",
                    action: Launch(exe: "code.exe", args: ["D:\\project", "--new-window"]),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        if let BindAction::Launch { exe, args } = &bindings[0].action {
            assert_eq!(exe, "code.exe");
            assert_eq!(args, &["D:\\project", "--new-window"]);
        } else {
            panic!("expected Launch");
        }
    }

    #[test]
    fn parse_open_url() {
        let ron = r#"
            ( bindings: [( keys: "Win+G", action: OpenUrl("https://github.com") )] )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(
            bindings[0].action,
            BindAction::OpenUrl("https://github.com".into())
        );
    }

    #[test]
    fn parse_media_variants() {
        for (variant, expected) in [
            ("PlayPause",  MediaKey::PlayPause),
            ("Stop",       MediaKey::Stop),
            ("Next",       MediaKey::Next),
            ("Previous",   MediaKey::Previous),
            ("VolumeUp",   MediaKey::VolumeUp),
            ("VolumeDown", MediaKey::VolumeDown),
            ("VolumeMute", MediaKey::VolumeMute),
        ] {
            let ron = format!(
                r#"( bindings: [( keys: "F13", action: Media({}) )] )"#,
                variant
            );
            let bindings = from_ron_str(&ron).unwrap();
            assert_eq!(bindings[0].action, BindAction::Media(expected),
                "variant {} did not round-trip", variant);
        }
    }

    #[test]
    fn parse_sequence_with_new_step_kinds() {
        let ron = r#"
            (
                bindings: [(
                    keys: "Win+Alt+N",
                    action: Sequence([
                        Launch(exe: "notepad.exe"),
                        Delay(200),
                        OpenUrl("https://example.com"),
                        Media(VolumeMute),
                    ]),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        if let BindAction::Sequence(steps) = &bindings[0].action {
            assert_eq!(steps.len(), 4);
            assert!(matches!(&steps[0], SequenceStep::Launch { exe, .. } if exe == "notepad.exe"));
            assert!(matches!(steps[1], SequenceStep::Delay(200)));
            assert!(matches!(&steps[2], SequenceStep::OpenUrl(u) if u == "https://example.com"));
            assert!(matches!(steps[3], SequenceStep::Media(MediaKey::VolumeMute)));
        } else {
            panic!("expected Sequence");
        }
    }

    // ---- Condition (when:) tests ----

    #[test]
    fn parse_when_app_equals() {
        let ron = r#"
            (
                bindings: [(
                    keys: "Pause",
                    action: Media(PlayPause),
                    when: AppEquals("spotify.exe"),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].condition {
            Condition::AppEquals(name) => assert_eq!(name, "spotify.exe"),
            other => panic!("expected AppEquals, got {:?}", other),
        }
    }

    #[test]
    fn parse_when_complex_and_or_not() {
        let ron = r#"
            (
                bindings: [(
                    keys: "Ctrl+Alt+L",
                    action: PressKey("L"),
                    when: And([
                        AppEquals("chrome.exe"),
                        Not(TitleContains("Incognito")),
                    ]),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].condition {
            Condition::And(parts) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(parts[0], Condition::AppEquals(_)));
                assert!(matches!(parts[1], Condition::Not(_)));
            }
            other => panic!("expected And, got {:?}", other),
        }
    }

    #[test]
    fn default_condition_is_always() {
        // No `when:` field → defaults to Always.
        let ron = r#"
            ( bindings: [( keys: "F13", action: DoNothing )] )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert!(bindings[0].condition.is_always());
    }

    // ---- Trigger tests ----

    #[test]
    fn default_trigger_is_immediate() {
        let ron = r#"
            ( bindings: [( keys: "F13", action: DoNothing )] )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(bindings[0].trigger, Trigger::Immediate);
    }

    #[test]
    fn parse_tap_trigger() {
        let ron = r#"
            (
                bindings: [(
                    keys: "CapsLock",
                    action: SwitchLanguage,
                    trigger: Tap(200),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        let user = bindings.iter().find(|b| !b.is_auto_blocker).unwrap();
        assert_eq!(user.trigger, Trigger::Tap(200));
        assert_eq!(user.action, BindAction::SwitchLanguage);
    }

    #[test]
    fn parse_hold_trigger() {
        let ron = r#"
            (
                bindings: [(
                    keys: "CapsLock",
                    action: PressKey("LShift"),
                    trigger: Hold(200),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(bindings[0].trigger, Trigger::Hold(200));
        assert_eq!(bindings[0].action, BindAction::PressKey(VK_LSHIFT));
    }

    #[test]
    fn parse_double_tap_trigger() {
        let ron = r#"
            (
                bindings: [(
                    keys: "LShift",
                    action: ToggleCapsLock,
                    trigger: DoubleTap(250),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(bindings[0].trigger, Trigger::DoubleTap(250));
    }

    #[test]
    fn coexisting_tap_and_hold_on_same_key() {
        // The whole point of B-style triggers: two bindings on the same key
        // with different triggers should both load cleanly.
        let ron = r#"
            (
                bindings: [
                    ( keys: "CapsLock", action: SwitchLanguage,         trigger: Tap(200) ),
                    ( keys: "CapsLock", action: PressKey("LShift"),     trigger: Hold(200) ),
                    ( keys: "CapsLock", action: ToggleCapsLock,         trigger: DoubleTap(250) ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0].trigger, Trigger::Tap(200));
        assert_eq!(bindings[1].trigger, Trigger::Hold(200));
        assert_eq!(bindings[2].trigger, Trigger::DoubleTap(250));
    }

    // ---- Sequence / Chord parsing tests ----

    #[test]
    fn parse_sequence_keys() {
        let ron = r#"
            (
                bindings: [(
                    sequence: (steps: ["g", "g"], max_gap_ms: 500),
                    action: PressKey("Home"),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].kind {
            BindingKind::Sequence { steps, max_gap } => {
                assert_eq!(steps.len(), 2);
                assert_eq!(steps[0].keys, vec![VK_G]);
                assert_eq!(steps[1].keys, vec![VK_G]);
                assert_eq!(max_gap.as_millis(), 500);
            }
            other => panic!("expected Sequence, got {:?}", other),
        }
    }

    #[test]
    fn parse_sequence_with_combos() {
        let ron = r#"
            (
                bindings: [(
                    sequence: (steps: ["Ctrl+X", "Ctrl+S"]),
                    action: PressKey("S"),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].kind {
            BindingKind::Sequence { steps, max_gap } => {
                assert_eq!(steps.len(), 2);
                assert!(steps[0].keys.contains(&VK_CONTROL));
                assert!(steps[0].keys.contains(&VK_X));
                assert!(steps[1].keys.contains(&VK_CONTROL));
                assert!(steps[1].keys.contains(&VK_S));
                assert_eq!(max_gap.as_millis(), 500); // default
            }
            other => panic!("expected Sequence, got {:?}", other),
        }
    }

    #[test]
    fn parse_chord_keys() {
        let ron = r#"
            (
                bindings: [(
                    chord: (keys: ["j", "k"], window_ms: 50),
                    action: PressKey("Escape"),
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].kind {
            BindingKind::Chord { keys, window } => {
                assert_eq!(keys.len(), 2);
                assert!(keys.contains(&VK_J));
                assert!(keys.contains(&VK_K));
                assert_eq!(window.as_millis(), 50);
            }
            other => panic!("expected Chord, got {:?}", other),
        }
    }

    #[test]
    fn parse_chord_default_window() {
        let ron = r#"
            (
                bindings: [(
                    chord: (keys: ["a", "s", "d"]),
                    action: SwitchLanguage,
                )],
            )
        "#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].kind {
            BindingKind::Chord { window, .. } => {
                assert_eq!(window.as_millis(), 50); // default
            }
            _ => panic!(),
        }
    }

    #[test]
    fn sequence_too_short_is_error() {
        let ron = r#"
            (
                bindings: [(
                    sequence: (steps: ["g"]),
                    action: DoNothing,
                )],
            )
        "#;
        let err = from_ron_str(ron).unwrap_err();
        match err {
            ConfigError::Bindings(errs) => {
                assert!(errs[0].contains("at least 2 steps"));
            }
            _ => panic!("expected Bindings error"),
        }
    }

    #[test]
    fn chord_too_short_is_error() {
        let ron = r#"
            (
                bindings: [(
                    chord: (keys: ["j"]),
                    action: DoNothing,
                )],
            )
        "#;
        let err = from_ron_str(ron).unwrap_err();
        match err {
            ConfigError::Bindings(errs) => {
                assert!(errs[0].contains("at least 2 keys"));
            }
            _ => panic!("expected Bindings error"),
        }
    }

    #[test]
    fn sequence_with_deferred_trigger_is_error() {
        let ron = r#"
            (
                bindings: [(
                    sequence: (steps: ["g", "g"]),
                    action: DoNothing,
                    trigger: Hold(200),
                )],
            )
        "#;
        let err = from_ron_str(ron).unwrap_err();
        match err {
            ConfigError::Bindings(errs) => {
                assert!(errs[0].contains("deferred trigger"));
            }
            _ => panic!("expected Bindings error"),
        }
    }

    #[test]
    fn chord_duplicate_key_is_error() {
        let ron = r#"
            (
                bindings: [(
                    chord: (keys: ["j", "j"]),
                    action: DoNothing,
                )],
            )
        "#;
        let err = from_ron_str(ron).unwrap_err();
        match err {
            ConfigError::Bindings(errs) => {
                assert!(errs[0].contains("duplicate"));
            }
            _ => panic!("expected Bindings error"),
        }
    }

    // ---- existing test, kept at the bottom ----

    #[test]
    fn sequence_text_with_emoji_roundtrips() {
        // Surrogate pairs in the source RON must reach SequenceStep::TypeText
        // unchanged.
        let ron_text = r#"
            (
                bindings: [
                    ( keys: "F13", action: Sequence([Text("🚀🌍 ёж")]) ),
                ],
            )
        "#;
        let bindings = from_ron_str(ron_text).unwrap();
        if let BindAction::Sequence(steps) = &bindings[0].action
            && let SequenceStep::TypeText(t) = &steps[0]
        {
            assert_eq!(t, "🚀🌍 ёж");
        } else {
            panic!("expected single TypeText");
        }
    }
}
