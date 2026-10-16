//! Disk I/O and conversion from on-disk `RawConfig` (RON) to a `Vec<Binding>`
//! ready for `App`. Default-config generation lives here so first run is
//! zero-touch.

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use std::time::Duration;

use crate::core::windows_actions::{BindAction, MediaKey};
use crate::data::binding::{Binding, BindingKind};
use crate::data::condition::Condition;
use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::KeySpec;
use crate::data::sequence::{SequenceStep, WindowOp};
use crate::data::trigger::Trigger;
use crate::data::vk_name::parse_vk;

use super::parsing::{parse_combo, parse_key_list, parse_wm_name};
use super::schema::{
    ChordSpec, MediaKeyRef, MessageRef, RawAction, RawBinding, RawCondition, RawConfig, RawStep,
    RawTrigger, RawWindowKind, SequenceSpec,
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
    let dir = exe.parent().ok_or_else(|| {
        ConfigError::Other(format!("executable path {:?} has no parent directory", exe))
    })?;
    Ok(dir.join("config.ron"))
}

/// Where the config file is, decided once at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigPath {
    /// The first command-line argument.
    CommandLine(PathBuf),
    /// `<exe_dir>\config.ron`.
    BesideExe(PathBuf),
}

impl ConfigPath {
    /// The first command-line argument if there is one, else
    /// `<exe_dir>\config.ron`.
    pub fn from_arg(arg: Option<OsString>) -> Result<Self, ConfigError> {
        match arg {
            Some(arg) => Ok(ConfigPath::CommandLine(PathBuf::from(arg))),
            None => default_config_path().map(ConfigPath::BesideExe),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ConfigPath::CommandLine(path) | ConfigPath::BesideExe(path) => path,
        }
    }

    pub fn into_path(self) -> PathBuf {
        match self {
            ConfigPath::CommandLine(path) | ConfigPath::BesideExe(path) => path,
        }
    }
}

/// Loads the bindings at startup, generating the default file if it does
/// not yet exist.
pub fn load(config: &ConfigPath) -> Result<Vec<Binding>, ConfigError> {
    load_or_generate(config.path())
}

/// Loads the bindings again after the file at `path` changed. For now it
/// also writes the default file to a missing `path`, like a first start
/// (KS-23).
pub fn reload(path: &Path) -> Result<Vec<Binding>, ConfigError> {
    load_or_generate(path)
}

/// Loads bindings from `path`, generating the default file there if it does
/// not yet exist. The "missing → write default" branch is by design: the
/// first run of the binary should produce a working setup without forcing
/// the user to author a config by hand.
fn load_or_generate(path: &Path) -> Result<Vec<Binding>, ConfigError> {
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

/// The Windows hotkey of an action's system function, if it has one. The
/// loader asks it for every binding that sets `block_original_combo`.
pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>;

/// Pure (no-I/O) conversion from RON text to `Vec<Binding>`. Kept available
/// to the crate so tests can exercise the schema without touching the disk.
///
/// The returned list holds the file's bindings in file order, followed by
/// the auto-blockers for the entries that set `block_original_combo: true`,
/// with the hotkeys read from the registry. Callers can hand the list
/// straight to `KeyboardHook::update_bindings`.
pub(crate) fn from_ron_str(text: &str) -> Result<Vec<Binding>, ConfigError> {
    from_ron_str_with_hotkeys(text, &BindAction::get_system_combination)
}

/// [`from_ron_str`] with the system hotkeys taken from `system_combination`
/// instead of the registry, so a test knows which auto-blockers to expect
/// whatever hotkey the machine has set.
pub(crate) fn from_ron_str_with_hotkeys(
    text: &str,
    system_combination: &SystemCombination,
) -> Result<Vec<Binding>, ConfigError> {
    // `IMPLICIT_SOME` lets users write `keys: "CapsLock"` instead of
    // `keys: Some("CapsLock")` for the `Option<String>` fields on
    // `RawBinding`. Standard ergonomic move when an Option is exposed in a
    // human-edited config — no one expects to type `Some(...)` everywhere.
    let options =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

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

    Ok(expand_with_auto_blockers(user_bindings, system_combination))
}

/// Appends an auto-blocker for the Windows hotkey of every binding that
/// declared `block_original_combo: true`, so that Windows' own behaviour for
/// that hotkey does not run next to the user's remap.
///
/// One blocker per distinct hotkey, after all of the file's bindings. None
/// for a hotkey whose exact keys a `keys` binding of the file uses, the
/// flagged one included, whatever its trigger or `when:`: that binding takes
/// the hotkey over, and its own `block_default` and `when:` decide whether
/// Windows sees the hotkey (KS-18).
fn expand_with_auto_blockers(
    user_bindings: Vec<Binding>,
    system_combination: &SystemCombination,
) -> Vec<Binding> {
    let mut hotkeys: Vec<KeyCombination> = Vec::new();
    for binding in &user_bindings {
        if binding.block_original_combo
            && let Some(hotkey) = system_combination(&binding.action)
            && !hotkeys.contains(&hotkey)
        {
            hotkeys.push(hotkey);
        }
    }

    let mut blockers = Vec::with_capacity(hotkeys.len());
    for hotkey in hotkeys {
        // Only `keys` bindings hold a set of keys the way a hotkey does;
        // sequences and chords have no `combination()`.
        match user_bindings
            .iter()
            .position(|b| b.combination() == Some(&hotkey))
        {
            Some(index) => {
                println!("[INFO] no auto-blocker for {hotkey}: binding #{index} uses the same keys")
            }
            None => blockers.push(Binding::new_auto_blocker(hotkey)),
        }
    }

    let mut result = user_bindings;
    result.extend(blockers);
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

    let kind = raw_binding_to_kind(raw).map_err(|e| format!("{}: {}", err_prefix, e))?;

    let action =
        raw_action_to_bind_action(&raw.action).map_err(|e| format!("{}: {}", err_prefix, e))?;

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

    // Sequence steps and chords have no exact-modifier rule to relax yet
    // (roadmap 3.2); an accepted but ignored flag would change meaning
    // silently once they get one.
    if !matches!(kind, BindingKind::Combo(_)) && raw.allow_extra_modifiers {
        return Err(format!(
            "{}: `allow_extra_modifiers` applies only to `keys` bindings",
            err_prefix
        ));
    }

    let binding = Binding::with_kind(kind, action)
        .with_block_default(raw.block_default)
        .with_block_original_combo(raw.block_original_combo)
        .with_condition(raw_condition_to_condition(&raw.when))
        .with_trigger(trigger)
        .with_allow_extra_modifiers(raw.allow_extra_modifiers);

    Ok(binding)
}

fn raw_binding_to_kind(raw: &RawBinding) -> Result<BindingKind, String> {
    // Exactly one of {keys, sequence, chord} must be set.
    let kind_count =
        (raw.keys.is_some() as u8) + (raw.sequence.is_some() as u8) + (raw.chord.is_some() as u8);
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
    let mut specs: Vec<KeySpec> = Vec::with_capacity(chord.keys.len());
    for (i, k) in chord.keys.iter().enumerate() {
        let spec = KeySpec::parse(k)
            .ok_or_else(|| format!("Chord key #{}: unknown key name {:?}", i, k))?;
        // Overlapping keys (`Shift` and `LShift`) would both count one
        // physical key, so a single press could complete the chord.
        if let Some(j) = specs.iter().position(|s| s.overlaps(spec)) {
            return Err(format!(
                "Chord key #{}: duplicate {:?} (overlaps key #{})",
                i, k, j
            ));
        }
        specs.push(spec);
    }
    Ok(BindingKind::Chord {
        keys: specs,
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
        RawTrigger::Immediate => Trigger::Immediate,
        RawTrigger::Tap(ms) => Trigger::Tap(ms),
        RawTrigger::Hold(ms) => Trigger::Hold(ms),
        RawTrigger::DoubleTap(ms) => Trigger::DoubleTap(ms),
    }
}

fn raw_condition_to_condition(raw: &RawCondition) -> Condition {
    match raw {
        RawCondition::Always => Condition::Always,
        RawCondition::AppEquals(s) => Condition::AppEquals(s.clone()),
        RawCondition::TitleContains(s) => Condition::TitleContains(s.clone()),
        RawCondition::TitleEquals(s) => Condition::TitleEquals(s.clone()),
        RawCondition::Not(inner) => Condition::Not(Box::new(raw_condition_to_condition(inner))),
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
        RawAction::SwitchLanguage => BindAction::SwitchLanguage,
        RawAction::SwitchLanguageBackward => BindAction::SwitchLanguageBackward,
        RawAction::ToggleCapsLock => BindAction::ToggleCapsLock,
        RawAction::DoNothing => BindAction::DoNothing,

        RawAction::PressKey(key) => {
            let vk = parse_vk(key).ok_or_else(|| format!("PressKey: unknown key {:?}", key))?;
            BindAction::PressKey(vk)
        }

        RawAction::PostMessage {
            msg,
            wparam,
            lparam,
        } => {
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
                converted
                    .push(raw_step_to_step(raw_step).map_err(|e| format!("step #{}: {}", i, e))?);
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
        MediaKeyRef::PlayPause => MediaKey::PlayPause,
        MediaKeyRef::Stop => MediaKey::Stop,
        MediaKeyRef::Next => MediaKey::Next,
        MediaKeyRef::Previous => MediaKey::Previous,
        MediaKeyRef::VolumeUp => MediaKey::VolumeUp,
        MediaKeyRef::VolumeDown => MediaKey::VolumeDown,
        MediaKeyRef::VolumeMute => MediaKey::VolumeMute,
    }
}

fn raw_step_to_step(raw: &RawStep) -> Result<SequenceStep, String> {
    Ok(match raw {
        RawStep::Delay(ms) => SequenceStep::Delay(*ms),

        RawStep::Text(text) => SequenceStep::TypeText(text.clone()),

        RawStep::Key(name) => {
            let vk = parse_vk(name).ok_or_else(|| format!("Key: unknown key {:?}", name))?;
            SequenceStep::PressKey(vk)
        }

        RawStep::Combo(combo) => {
            let keys = parse_key_list(combo).map_err(|e| format!("Combo {:?}: {}", combo, e))?;
            if keys.is_empty() {
                return Err(format!("Combo {:?}: empty", combo));
            }
            SequenceStep::PressCombo(keys)
        }

        RawStep::Window(kind) => SequenceStep::Window(match kind {
            RawWindowKind::Minimize => WindowOp::Minimize,
            RawWindowKind::Maximize => WindowOp::Maximize,
            RawWindowKind::Restore => WindowOp::Restore,
            RawWindowKind::Close => WindowOp::Close,
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
    use crate::data::key_spec::Modifier;
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
    fn default_config_commented_examples_load() {
        // Users enable the template's examples by removing the `// `, so
        // they must load as written.
        let mut lines = DEFAULT_CONFIG.lines();
        lines
            .by_ref()
            .find(|line| line.contains("// ---- examples (uncomment to enable) ----"))
            .expect("the template has an examples section");
        let examples: Vec<&str> = lines
            .take_while(|line| line.trim() != "],")
            .map(|line| match line.trim_start() {
                "//" => "",
                line => line.strip_prefix("// ").unwrap_or(line),
            })
            .collect();
        let ron = format!("( bindings: [\n{}\n] )", examples.join("\n"));
        let bindings = from_ron_str_with_hotkeys(&ron, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("the template's examples do not load: {e}\n{ron}"));
        assert!(!bindings.is_empty(), "the template's examples are empty");
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
        // 2 entries: the user binding + the auto-blocker added because of
        // block_original_combo. Order: user binding first, then the
        // auto-blocker.
        assert_eq!(bindings.len(), 2);

        let user = bindings.iter().find(|b| !b.is_auto_blocker).unwrap();
        assert_eq!(
            user.combination().unwrap().keys,
            vec![KeySpec::Key(VK_CAPITAL)]
        );
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
    fn auto_blocker_keys_come_from_the_hotkey_lookup() {
        let ron_text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
            (keys: "F13", action: ToggleCapsLock, block_original_combo: true),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &language_hotkey(&[VK_CONTROL, VK_SHIFT]))
                .expect("config loads");

        assert_eq!(bindings.len(), 3);
        let mut blockers = bindings.iter().filter(|b| b.is_auto_blocker);
        let blocker = blockers.next().expect("SwitchLanguage gets a blocker");
        assert_eq!(
            blocker.combination(),
            Some(&KeyCombination::from_keys([VK_CONTROL, VK_SHIFT]))
        );
        assert!(
            blockers.next().is_none(),
            "ToggleCapsLock has no hotkey, so no blocker"
        );
    }

    /// Stands in for the registry: `keys` is the Windows language hotkey,
    /// and no other action has a hotkey.
    fn language_hotkey(
        keys: &'static [VIRTUAL_KEY],
    ) -> impl Fn(&BindAction) -> Option<KeyCombination> {
        let hotkey = KeyCombination::from_keys(keys.iter().copied());
        move |action: &BindAction| match action {
            BindAction::SwitchLanguage | BindAction::SwitchLanguageBackward => Some(hotkey.clone()),
            _ => None,
        }
    }

    /// How many auto-blockers the loader added.
    fn blockers(bindings: &[Binding]) -> usize {
        bindings.iter().filter(|b| b.is_auto_blocker).count()
    }

    const ALT_SHIFT: &[VIRTUAL_KEY] = &[VK_MENU, VK_SHIFT];

    #[test]
    fn auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order() {
        // KS-18: the dedup looked only at the bindings above the flagged one.
        let configs = [
            (
                "flagged binding above",
                r#"( bindings: [
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
                    (keys: "Alt+Shift", action: SwitchLanguageBackward),
                ] )"#,
            ),
            (
                "flagged binding below",
                r#"( bindings: [
                    (keys: "Alt+Shift", action: SwitchLanguageBackward),
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
                ] )"#,
            ),
            (
                "conditional deferred binding below",
                r#"( bindings: [
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
                    (
                        keys: "Shift+Alt",
                        action: SwitchLanguageBackward,
                        trigger: Tap(200),
                        when: AppEquals("code.exe"),
                    ),
                ] )"#,
            ),
        ];
        for (name, ron_text) in configs {
            let bindings = from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(blockers(&bindings), 0, "{name}");
            assert_eq!(bindings.len(), 2, "{name}");
        }
    }

    #[test]
    fn auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys() {
        let ron_text = r#"( bindings: [
            (keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).expect("config loads");
        assert_eq!(blockers(&bindings), 0);
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn auto_blockers_follow_the_bindings_of_the_file() {
        let ron_text = r#"( bindings: [
            (keys: "F13", action: SwitchLanguage, block_original_combo: true),
            (keys: "F14", action: DoNothing),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).expect("config loads");
        assert_eq!(bindings.len(), 3);
        assert!(
            !bindings[0].is_auto_blocker,
            "the file's bindings come first"
        );
        assert_eq!(
            bindings[0].combination().expect("a keys binding").keys,
            vec![KeySpec::Key(VK_F13)]
        );
        assert_eq!(
            bindings[1].combination().expect("a keys binding").keys,
            vec![KeySpec::Key(VK_F14)]
        );
        assert!(bindings[2].is_auto_blocker, "the blocker comes last");
        assert_eq!(
            bindings[2].combination(),
            Some(&KeyCombination::from_keys(ALT_SHIFT.iter().copied()))
        );
    }

    #[test]
    fn auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey() {
        // Only the same keys take the hotkey over: a sided or larger combo,
        // a chord or a sequence leaves the other presses of the hotkey to
        // the blocker.
        let ron_text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
            (keys: "LAlt+LShift", action: SwitchLanguageBackward),
            (keys: "Ctrl+Alt+Shift", action: SwitchLanguageBackward),
            (chord: (keys: ["Alt", "Shift"]), action: SwitchLanguageBackward),
            (sequence: (steps: ["Alt+Shift", "F13"]), action: SwitchLanguageBackward),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).expect("config loads");
        assert_eq!(blockers(&bindings), 1);
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
            BindAction::PostMessage {
                msg: WM_CLOSE,
                wparam: 0,
                lparam: 0
            },
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
            BindAction::PostMessage {
                msg,
                wparam,
                lparam,
            } => {
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
            assert!(
                errs[0].contains("Bogus"),
                "error should mention bogus key: {:?}",
                errs
            );
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
            ("PlayPause", MediaKey::PlayPause),
            ("Stop", MediaKey::Stop),
            ("Next", MediaKey::Next),
            ("Previous", MediaKey::Previous),
            ("VolumeUp", MediaKey::VolumeUp),
            ("VolumeDown", MediaKey::VolumeDown),
            ("VolumeMute", MediaKey::VolumeMute),
        ] {
            let ron = format!(
                r#"( bindings: [( keys: "F13", action: Media({}) )] )"#,
                variant
            );
            let bindings = from_ron_str(&ron).unwrap();
            assert_eq!(
                bindings[0].action,
                BindAction::Media(expected),
                "variant {} did not round-trip",
                variant
            );
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
            assert!(matches!(
                steps[3],
                SequenceStep::Media(MediaKey::VolumeMute)
            ));
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
                assert_eq!(steps[0].keys, vec![KeySpec::Key(VK_G)]);
                assert_eq!(steps[1].keys, vec![KeySpec::Key(VK_G)]);
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
                let ctrl = KeySpec::AnySide(Modifier::Ctrl);
                assert!(steps[0].keys.contains(&ctrl));
                assert!(steps[0].keys.contains(&KeySpec::Key(VK_X)));
                assert!(steps[1].keys.contains(&ctrl));
                assert!(steps[1].keys.contains(&KeySpec::Key(VK_S)));
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
                assert!(keys.contains(&KeySpec::Key(VK_J)));
                assert!(keys.contains(&KeySpec::Key(VK_K)));
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

    #[test]
    fn chord_with_an_unsided_modifier_parses_to_either_side() {
        let ron = r#"( bindings: [ ( chord: (keys: ["Shift", "Space"]), action: DoNothing ) ] )"#;
        let bindings = from_ron_str(ron).unwrap();
        match &bindings[0].kind {
            BindingKind::Chord { keys, .. } => assert_eq!(
                keys,
                &vec![KeySpec::AnySide(Modifier::Shift), KeySpec::Key(VK_SPACE)]
            ),
            other => panic!("expected Chord, got {:?}", other),
        }
    }

    #[test]
    fn chord_keys_that_overlap_are_rejected() {
        // One physical key would count for both chord keys (KS-07).
        for keys in [
            r#"["Shift", "LShift"]"#,
            r#"["Win", "RWin"]"#,
            r#"["Ctrl", "RCtrl"]"#,
        ] {
            let ron = format!(r#"( bindings: [ ( chord: (keys: {keys}), action: DoNothing ) ] )"#);
            let err = from_ron_str(&ron).expect_err("overlapping chord keys must be rejected");
            match err {
                ConfigError::Bindings(errs) => {
                    assert!(
                        errs[0].contains("duplicate"),
                        "error for {keys} should say duplicate: {errs:?}"
                    );
                }
                other => panic!("expected Bindings error for {keys}, got {other:?}"),
            }
        }
    }

    #[test]
    fn allow_extra_modifiers_defaults_to_false() {
        let bindings =
            from_ron_str(r#"( bindings: [ ( keys: "CapsLock", action: SwitchLanguage ) ] )"#)
                .unwrap();
        assert!(!bindings[0].allow_extra_modifiers);
    }

    #[test]
    fn allow_extra_modifiers_true_is_parsed() {
        let ron = r#"( bindings: [
            ( keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true ),
            ( keys: "F13", action: DoNothing, allow_extra_modifiers: false ),
        ] )"#;
        let bindings = from_ron_str(ron).unwrap();
        assert!(bindings[0].allow_extra_modifiers);
        assert!(!bindings[1].allow_extra_modifiers);
    }

    /// The README's combo-syntax section, read at compile time so that the
    /// published examples are the ones under test.
    const README: &str = include_str!("../../README.md");

    #[test]
    fn readme_matching_examples_load() {
        let start = README
            .find("### Combo syntax")
            .expect("README has a combo syntax section");
        let section = &README[start..];
        let end = section.find("\n### ").unwrap_or(section.len());
        let section = &section[..end];

        let mut blocks = 0;
        for block in section.split("```ron").skip(1) {
            let body = block
                .split("```")
                .next()
                .expect("invariant: split yields at least one piece");
            let ron = format!("( bindings: [ {body} ] )");
            let bindings = from_ron_str(&ron)
                .unwrap_or_else(|e| panic!("README example does not load: {e}\n{body}"));
            assert!(!bindings.is_empty(), "empty README example:\n{body}");
            blocks += 1;
        }
        assert!(
            blocks > 0,
            "no RON example in the README's combo syntax section"
        );
    }

    #[test]
    fn allow_extra_modifiers_is_only_for_keys_bindings() {
        for kind in [
            r#"sequence: (steps: ["F13", "F14"])"#,
            r#"chord: (keys: ["F13", "F14"])"#,
        ] {
            let ron = format!(
                r#"( bindings: [ ( {kind}, action: DoNothing, allow_extra_modifiers: true ) ] )"#
            );
            let err = from_ron_str(&ron)
                .expect_err("allow_extra_modifiers on a sequence/chord must be rejected");
            match err {
                ConfigError::Bindings(errs) => assert!(
                    errs[0].contains("allow_extra_modifiers"),
                    "error for {kind} should name the field: {errs:?}"
                ),
                other => panic!("expected Bindings error for {kind}, got {other:?}"),
            }
        }
    }

    // ---- the config path and the file ----

    #[test]
    fn a_relative_config_argument_is_made_absolute() {
        // KS-21: the watcher compares absolute event paths, so a relative
        // argument must not reach it as given.
        let cwd = std::env::current_dir().expect("the test has a current folder");
        for arg in ["my.ron", r".\sub\..\my.ron"] {
            let config = ConfigPath::from_arg(Some(arg.into())).expect("a relative path is valid");
            assert!(matches!(config, ConfigPath::CommandLine(_)), "{arg}");
            assert_eq!(config.path(), cwd.join("my.ron"), "{arg}");
        }
    }

    #[test]
    fn an_empty_config_argument_is_an_error() {
        ConfigPath::from_arg(Some(OsString::new()))
            .expect_err("an empty config path must be an error");
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
