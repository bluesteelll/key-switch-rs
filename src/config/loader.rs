//! Disk I/O and conversion from on-disk `RawConfig` (RON) to a `Vec<Binding>`
//! ready for `App`. Default-config generation lives here so first run is
//! zero-touch.

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use std::time::Duration;

use crate::core::windows_actions::{BindAction, MediaKey};
use crate::data::binding::{Binding, BindingKind, BlockerMode};
use crate::data::condition::Condition;
use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::KeySpec;
use crate::data::sequence::{SequenceStep, WindowOp};
use crate::data::trigger::Trigger;
use crate::data::vk_name::parse_vk;

use super::parsing::{parse_combo, parse_key_list, parse_wm_name};
use super::schema::{
    ChordSpec, MediaKeyRef, MessageRef, RawAction, RawBinding, RawBlockOriginalCombo, RawCondition,
    RawConfig, RawStep, RawTrigger, RawWindowKind, SequenceSpec,
};

/// Bundled default `config.ron`. Written to `<exe_dir>\config.ron` when the
/// daemon starts without one.
const DEFAULT_CONFIG: &str = include_str!("default_config.ron");

/// Every timing value of a binding: the Tap, Hold and DoubleTap terms,
/// `max_gap_ms` and `window_ms` (KS-27). `Tap(0)` or `window_ms: 0` would
/// swallow keys and almost never fire; an hour-long `Hold` is a typo.
const TIMING_RANGE_MS: RangeInclusive<u64> = 1..=10_000;

#[derive(Debug)]
pub enum ConfigError {
    /// The config file, or a folder on its path, does not exist.
    Missing {
        path: PathBuf,
    },
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
            ConfigError::Missing { path } => {
                write!(f, "config file {} does not exist", path.display())
            }
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
    /// The first command-line argument, made absolute against the current
    /// directory.
    CommandLine(PathBuf),
    /// `<exe_dir>\config.ron`.
    BesideExe(PathBuf),
}

impl ConfigPath {
    /// The first command-line argument if there is one, else
    /// `<exe_dir>\config.ron`.
    ///
    /// A relative argument is resolved here, once, against the current
    /// directory, so the startup load, the console and the watcher all use
    /// the same absolute path (KS-21).
    pub fn from_arg(arg: Option<OsString>) -> Result<Self, ConfigError> {
        match arg {
            Some(arg) => std::path::absolute(&arg)
                .map(ConfigPath::CommandLine)
                .map_err(|e| {
                    ConfigError::Other(format!("config path {arg:?} cannot be used: {e}"))
                }),
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

/// Loads the bindings at startup.
///
/// A missing `<exe_dir>\config.ron` is first written from the bundled
/// template, so the first run produces a working setup without a
/// hand-written config. A missing command-line path is
/// [`ConfigError::Missing`] and nothing is created for it: a typo must not
/// start the daemon with the defaults (KS-23).
pub fn load(config: &ConfigPath) -> Result<Vec<Binding>, ConfigError> {
    let path = config.path();
    let text = match (config, read_config(path)) {
        (ConfigPath::BesideExe(_), Err(ConfigError::Missing { .. })) => {
            write_default(path)?;
            log::info!("generated default config at {}", path.display());
            read_config(path)?
        }
        (_, read) => {
            let text = read?;
            log::info!("loading config from {}", path.display());
            text
        }
    };
    parse_config(path, &text)
}

/// Loads the bindings again after the file at `path` changed. Never writes:
/// a deleted or renamed file is [`ConfigError::Missing`], and the caller
/// keeps the bindings it has (KS-23).
pub fn reload(path: &Path) -> Result<Vec<Binding>, ConfigError> {
    let text = read_config(path)?;
    log::info!("loading config from {}", path.display());
    parse_config(path, &text)
}

/// The text of the config file. A missing file, or a missing folder on its
/// path, is [`ConfigError::Missing`]. There is no `exists()` check first:
/// the file could come or go between the check and the read.
fn read_config(path: &Path) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|source| match source.kind() {
        io::ErrorKind::NotFound => ConfigError::Missing {
            path: path.to_path_buf(),
        },
        _ => ConfigError::Io {
            path: path.to_path_buf(),
            source,
        },
    })
}

/// [`from_ron_str`] with `path` in a parse error.
fn parse_config(path: &Path, text: &str) -> Result<Vec<Binding>, ConfigError> {
    from_ron_str(text).map_err(|e| match e {
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
/// the auto-blockers for the entries that set `block_original_combo: true`
/// or `MaskOnly`, with the hotkeys read from the registry. Callers can hand
/// the list straight to `KeyboardHook::update_bindings`.
pub(crate) fn from_ron_str(text: &str) -> Result<Vec<Binding>, ConfigError> {
    from_ron_str_with_hotkeys(text, &BindAction::get_system_combination)
}

/// [`from_ron_str`] with the system hotkeys taken from `system_combination`
/// instead of the registry, so a test knows which auto-blockers to expect
/// whatever hotkey the machine has set. Warnings about a config that loads
/// are logged with `log::warn!` on the thread that loads it: the main
/// thread at startup, `config-watcher` on a reload.
pub(crate) fn from_ron_str_with_hotkeys(
    text: &str,
    system_combination: &SystemCombination,
) -> Result<Vec<Binding>, ConfigError> {
    let parsed = parse_ron_str(text, system_combination)?;
    for warning in &parsed.warnings {
        log::warn!("{warning}");
    }
    Ok(parsed.bindings)
}

/// A config that loads: its bindings, as [`from_ron_str_with_hotkeys`]
/// returns them, and the warnings to show about them.
struct ParsedConfig {
    bindings: Vec<Binding>,
    warnings: Vec<String>,
}

/// [`from_ron_str_with_hotkeys`] without logging the warnings, so that a
/// test can read them.
fn parse_ron_str(
    text: &str,
    system_combination: &SystemCombination,
) -> Result<ParsedConfig, ConfigError> {
    // Notepad's "UTF-8 with BOM" and Windows PowerShell 5.1's
    // `-Encoding UTF8` start the file with U+FEFF, which ron 0.10.1 does not
    // skip (KS-24). Only a leading one goes: inside a string it is text.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

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
        log::warn!("config contains no bindings");
    }

    // Every entry converted (an error returned above), so `raw.bindings[k]`
    // and `user_bindings[k]` are the same entry of the file.
    let mut warnings = duplicate_warnings(&raw.bindings, &user_bindings);
    let bindings = expand_with_auto_blockers(user_bindings, system_combination, &mut warnings);
    Ok(ParsedConfig { bindings, warnings })
}

/// One warning for every binding that duplicates an earlier one
/// ([`Binding::duplicates`]), naming the first such earlier binding, with
/// both numbered from 0 and their keys as the file writes them. `raw` and
/// `bindings` hold the file's entries in file order, one binding per entry.
/// The message says which binding repeats which, not which one acts: for
/// `keys` bindings the earlier one does, while two sequences or chords
/// share one pending entry.
fn duplicate_warnings(raw: &[RawBinding], bindings: &[Binding]) -> Vec<String> {
    let mut warnings = Vec::new();
    for (later, binding) in bindings.iter().enumerate() {
        if let Some(earlier) = bindings[..later]
            .iter()
            .position(|other| binding.duplicates(other))
        {
            warnings.push(format!(
                "binding #{later} [{}] duplicates binding #{earlier} [{}]; remove one of them",
                format_raw_binding_keys(&raw[later]),
                format_raw_binding_keys(&raw[earlier]),
            ));
        }
    }
    warnings
}

/// Appends an auto-blocker for the Windows hotkey of every binding that
/// declared `block_original_combo: true` or `MaskOnly`, so that Windows' own
/// behaviour for that hotkey does not run next to the user's remap.
///
/// One blocker per distinct hotkey and mode, in the order the file first
/// asks for them, after all of the file's bindings. `MaskOnly` needs a
/// hotkey made only of modifiers: on another hotkey, such as `` ` ``, it
/// gives a blocker that swallows, as `true` does, and a warning. No blocker
/// for a hotkey whose exact keys a `keys` binding of the file uses, the
/// flagged one included, whatever its trigger or `when:`: that binding takes
/// the hotkey over, and its own `block_default` and `when:` decide whether
/// Windows sees the hotkey (KS-18).
///
/// A blocker acts where the `when:` of a binding that asked for it holds
/// (KS-35): its condition is the conditions of those bindings OR'ed, each
/// once, or `Always` if one of them has none. The bindings' triggers are
/// not inherited: the blocker acts on the hotkey's own press.
fn expand_with_auto_blockers(
    user_bindings: Vec<Binding>,
    system_combination: &SystemCombination,
    warnings: &mut Vec<String>,
) -> Vec<Binding> {
    // What the file asks for, in file order, with the asking binding's
    // condition.
    let mut requests: Vec<(KeyCombination, BlockerMode, &Condition)> = Vec::new();
    for (index, binding) in user_bindings.iter().enumerate() {
        let Some(requested) = binding.block_original_combo else {
            continue;
        };
        let Some(hotkey) = system_combination(&binding.action) else {
            continue;
        };
        let mode = if requested == BlockerMode::MaskOnly && !hotkey.is_modifier_only() {
            warnings.push(format!(
                "binding #{index}: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the {hotkey} hotkey is blocked as with true"
            ));
            BlockerMode::Swallow
        } else {
            requested
        };
        requests.push((hotkey, mode, &binding.condition));
    }

    let mut blockers = Vec::new();
    let mut hotkeys_seen: Vec<&KeyCombination> = Vec::new();
    for (hotkey, _, _) in &requests {
        if hotkeys_seen.contains(&hotkey) {
            continue;
        }
        hotkeys_seen.push(hotkey);
        // Only `keys` bindings hold a set of keys the way a hotkey does;
        // sequences and chords have no `combination()`.
        if let Some(index) = user_bindings
            .iter()
            .position(|b| b.combination() == Some(hotkey))
        {
            log::info!("no auto-blocker for {hotkey}: binding #{index} uses the same keys");
            continue;
        }
        let mut modes: Vec<BlockerMode> = Vec::new();
        for (other, mode, _) in &requests {
            if other == hotkey && !modes.contains(mode) {
                modes.push(*mode);
            }
        }
        for mode in modes {
            let owners = requests
                .iter()
                .filter(|(other, other_mode, _)| other == hotkey && *other_mode == mode)
                .map(|&(_, _, condition)| condition);
            blockers.push(
                Binding::new_auto_blocker(hotkey.clone(), mode)
                    .with_condition(any_of_conditions(owners)),
            );
        }
    }

    let mut result = user_bindings;
    result.extend(blockers);
    result
}

/// The condition that holds where any of `conditions` holds: `Always` if
/// one of them is `Always`, else each distinct condition once, in the
/// order given, as written (`Condition`'s equality), OR'ed if there are
/// several. `Always` for none.
fn any_of_conditions<'a>(conditions: impl IntoIterator<Item = &'a Condition>) -> Condition {
    let mut distinct: Vec<&Condition> = Vec::new();
    for condition in conditions {
        if condition.is_always() {
            return Condition::Always;
        }
        if !distinct.contains(&condition) {
            distinct.push(condition);
        }
    }
    match distinct.as_slice() {
        [] => Condition::Always,
        [one] => (*one).clone(),
        several => Condition::Or(several.iter().map(|&condition| condition.clone()).collect()),
    }
}

/// Writes the bundled template to `path`, creating its folder; only for
/// `<exe_dir>\config.ron` at startup. A file that appeared after the read
/// is left as it is and loaded.
fn write_default(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    let io_error = |source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    };
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => file.write_all(DEFAULT_CONFIG.as_bytes()).map_err(io_error),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(io_error(e)),
    }
}

fn raw_binding_to_binding(raw: &RawBinding, index: usize) -> Result<Binding, String> {
    let keys_display = format_raw_binding_keys(raw);
    let err_prefix = format!("binding #{} [{}]", index, keys_display);

    let kind = raw_binding_to_kind(raw).map_err(|e| format!("{}: {}", err_prefix, e))?;

    let action =
        raw_action_to_bind_action(&raw.action).map_err(|e| format!("{}: {}", err_prefix, e))?;

    let trigger =
        raw_trigger_to_trigger(raw.trigger).map_err(|e| format!("{}: {}", err_prefix, e))?;

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

    // Deferred triggers, sequences and chords never fire on an auto-repeat,
    // so `repeat: true` there would be accepted and ignored. `false` states
    // what they do anyway and loads.
    let honours_repeat = matches!(kind, BindingKind::Combo(_)) && trigger.is_immediate();
    if raw.repeat == Some(true) && !honours_repeat {
        return Err(format!(
            "{}: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger",
            err_prefix
        ));
    }

    let block_default = raw
        .block_default
        .unwrap_or_else(|| Binding::blocks_by_default(&kind, trigger));
    let binding = Binding::with_kind(kind, action)
        .with_block_default(block_default)
        .with_block_original_combo(raw_blocker_to_mode(raw.block_original_combo))
        .with_condition(raw_condition_to_condition(&raw.when))
        .with_trigger(trigger)
        .with_allow_extra_modifiers(raw.allow_extra_modifiers);
    // Unset, `repeat` keeps the default `with_kind` took from the action.
    let binding = match raw.repeat {
        Some(repeat) => binding.with_repeat(repeat),
        None => binding,
    };

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
    let max_gap_ms = check_timing_ms(&format!("max_gap_ms: {}", seq.max_gap_ms), seq.max_gap_ms)?;
    Ok(BindingKind::Sequence {
        steps: parsed,
        max_gap: Duration::from_millis(max_gap_ms),
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
    let window_ms = check_timing_ms(&format!("window_ms: {}", chord.window_ms), chord.window_ms)?;
    Ok(BindingKind::Chord {
        keys: specs,
        window: Duration::from_millis(window_ms),
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

fn raw_blocker_to_mode(raw: RawBlockOriginalCombo) -> Option<BlockerMode> {
    match raw {
        RawBlockOriginalCombo::On => Some(BlockerMode::Swallow),
        RawBlockOriginalCombo::Off => None,
        RawBlockOriginalCombo::MaskOnly => Some(BlockerMode::MaskOnly),
    }
}

fn raw_trigger_to_trigger(raw: RawTrigger) -> Result<Trigger, String> {
    Ok(match raw {
        RawTrigger::Immediate => Trigger::Immediate,
        RawTrigger::Tap(ms) => Trigger::Tap(check_timing_ms(&format!("Tap({ms})"), ms)?),
        RawTrigger::Hold(ms) => Trigger::Hold(check_timing_ms(&format!("Hold({ms})"), ms)?),
        RawTrigger::DoubleTap(ms) => {
            Trigger::DoubleTap(check_timing_ms(&format!("DoubleTap({ms})"), ms)?)
        }
    })
}

/// `Ok(ms)` inside [`TIMING_RANGE_MS`], else "`<written>` is out of range:
/// timings are 1 to 10000 ms", with `written` the value as the config
/// spells it (`Tap(0)`, `max_gap_ms: 0`).
fn check_timing_ms(written: &str, ms: u64) -> Result<u64, String> {
    if TIMING_RANGE_MS.contains(&ms) {
        Ok(ms)
    } else {
        Err(format!(
            "{written} is out of range: timings are {} to {} ms",
            TIMING_RANGE_MS.start(),
            TIMING_RANGE_MS.end()
        ))
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
    use crate::config::testing::{ONE, TempDir};
    use crate::data::key_spec::Modifier;
    use crate::system::registry::hotkey_from_values;
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

    /// The template's commented examples, uncommented and loaded as a user
    /// who removes the `// ` would load them, with no system hotkey.
    fn template_examples() -> Vec<Binding> {
        let ron = template_examples_ron();
        from_ron_str_with_hotkeys(&ron, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("the template's examples do not load: {e}\n{ron}"))
    }

    /// The text of [`template_examples`]: the template's commented examples
    /// with the `// ` removed, as a config of their own.
    fn template_examples_ron() -> String {
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
        format!("( bindings: [\n{}\n] )", examples.join("\n"))
    }

    #[test]
    fn default_config_commented_examples_load() {
        // Users enable the template's examples by removing the `// `, so
        // they must load as written.
        assert!(
            !template_examples().is_empty(),
            "the template's examples are empty"
        );
    }

    #[test]
    fn the_template_lshift_double_tap_example_lets_the_left_shift_through() {
        // Roadmap item 1.3: the example is made only of modifiers, so the
        // left Shift keeps typing capitals.
        let examples = template_examples();
        let example = examples
            .iter()
            .find(|b| {
                b.combination() == Some(&KeyCombination::new(VK_LSHIFT))
                    && b.trigger == Trigger::DoubleTap(250)
            })
            .expect("the template has the LShift DoubleTap example");
        assert!(!example.block_default);
    }

    #[test]
    fn block_default_defaults_to_false_only_for_deferred_bindings_made_only_of_modifiers() {
        let ron_text = r#"( bindings: [
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)),
            (keys: "Win", action: ToggleCapsLock, trigger: Tap(200)),
            (keys: "RShift", action: ToggleCapsLock, trigger: Tap(300)),
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "Shift+CapsLock", action: ToggleCapsLock, trigger: Hold(200)),
            (keys: "LShift", action: ToggleCapsLock),
            (keys: "Alt+Shift", action: SwitchLanguageBackward),
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (chord: (keys: ["F13", "F14"]), action: ToggleCapsLock),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &|_: &BindAction| None).expect("config loads");
        let block_default: Vec<bool> = bindings.iter().map(|b| b.block_default).collect();
        assert_eq!(
            block_default,
            [
                false, false, false, false, true, true, true, true, true, true
            ]
        );
    }

    #[test]
    fn an_explicit_block_default_wins_over_its_default() {
        let ron_text = r#"( bindings: [
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250), block_default: true),
            (keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000), block_default: false),
            (keys: "LShift", action: ToggleCapsLock, block_default: false),
            (keys: "F14", action: SwitchLanguage, trigger: Hold(200), block_default: true),
        ] )"#;
        let bindings =
            from_ron_str_with_hotkeys(ron_text, &|_: &BindAction| None).expect("config loads");
        let block_default: Vec<bool> = bindings.iter().map(|b| b.block_default).collect();
        assert_eq!(block_default, [true, false, false, false, true]);
    }

    #[test]
    fn block_default_takes_some_and_none_like_the_other_optional_fields() {
        let ron_text = r#"( bindings: [
            (keys: "F13", action: DoNothing, block_default: Some(false)),
            (keys: "LShift", action: DoNothing, trigger: DoubleTap(250), block_default: None),
            (keys: "F14", action: DoNothing, block_default: None),
        ] )"#;
        let bindings = from_ron_str_with_hotkeys(ron_text, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("block_default takes Some and None: {e}"));
        let block_default: Vec<bool> = bindings.iter().map(|b| b.block_default).collect();
        assert_eq!(block_default, [false, false, true]);
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
        let bindings = from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).unwrap();
        // 2 entries: the user binding + the auto-blocker added because of
        // block_original_combo. Order: user binding first, then the
        // auto-blocker.
        assert_eq!(bindings.len(), 2);

        let user = bindings.iter().find(|b| !b.is_auto_blocker()).unwrap();
        assert_eq!(
            user.combination().unwrap().keys,
            vec![KeySpec::Key(VK_CAPITAL)]
        );
        assert_eq!(user.action, BindAction::SwitchLanguage);
        assert_eq!(user.block_original_combo, Some(BlockerMode::Swallow));
        assert!(user.block_default); // default

        let auto = bindings.iter().find(|b| b.is_auto_blocker()).unwrap();
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
        assert!(!bindings[0].is_auto_blocker());
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
        assert!(!bindings[0].is_auto_blocker());
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
        let bindings = from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).unwrap();
        // 2 user + 1 auto-blocker = 3, not 4.
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings.iter().filter(|b| b.is_auto_blocker()).count(), 1);
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
        let mut blockers = bindings.iter().filter(|b| b.is_auto_blocker());
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
        bindings.iter().filter(|b| b.is_auto_blocker()).count()
    }

    const ALT_SHIFT: &[VIRTUAL_KEY] = &[VK_MENU, VK_SHIFT];

    /// Stands in for the registry with the `Hotkey` and `Language Hotkey`
    /// strings of `HKCU\Keyboard Layout\Toggle`, `None` for a missing value,
    /// resolved as the daemon resolves them.
    fn toggle_values(
        values: [Option<&'static str>; 2],
    ) -> impl Fn(&BindAction) -> Option<KeyCombination> {
        move |action: &BindAction| {
            action
                .to_system_function()
                .and_then(|function| hotkey_from_values(function, values))
        }
    }

    #[test]
    fn the_registry_values_choose_the_auto_blocker_keys() {
        let alt_shift = KeyCombination::from_keys(ALT_SHIFT.iter().copied());
        let cases = [
            (
                [Some("2"), None],
                KeyCombination::from_keys([VK_CONTROL, VK_SHIFT]),
            ),
            ([None, None], alt_shift.clone()),
            ([Some("1"), Some("3")], alt_shift),
        ];
        for (values, hotkey) in cases {
            let bindings = from_ron_str_with_hotkeys(DEFAULT_CONFIG, &toggle_values(values))
                .unwrap_or_else(|e| panic!("the template loads with {values:?}: {e}"));
            let blocker_keys: Vec<Option<&KeyCombination>> = bindings
                .iter()
                .filter(|b| b.is_auto_blocker())
                .map(Binding::combination)
                .collect();
            assert_eq!(blocker_keys, [Some(&hotkey)], "{values:?}");
        }
    }

    #[test]
    fn a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker() {
        // KS-17: the template's CapsLock binding asks for the blocker of a
        // Windows hotkey that does not exist.
        let not_assigned = toggle_values([Some("3"), Some("3")]);
        let bindings =
            from_ron_str_with_hotkeys(DEFAULT_CONFIG, &not_assigned).expect("the template loads");
        assert_eq!(blockers(&bindings), 0);
        assert_eq!(bindings.len(), 2);

        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly),
        ] )"#;
        let parsed = parse_ron_str(text, &not_assigned)
            .unwrap_or_else(|e| panic!("config does not load: {e}"));
        assert_eq!(blockers(&parsed.bindings), 0);
        assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
    }

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
            !bindings[0].is_auto_blocker(),
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
        assert!(bindings[2].is_auto_blocker(), "the blocker comes last");
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

    // ---- block_original_combo: true | false | MaskOnly (roadmap item 1.2, KS-05) ----

    /// The startup lines of the auto-blockers among `bindings`, in order.
    fn blocker_lines(bindings: &[Binding]) -> Vec<String> {
        bindings
            .iter()
            .map(Binding::to_string)
            .filter(|line| line.starts_with("[AUTO-BLOCK]"))
            .collect()
    }

    #[test]
    fn block_original_combo_takes_true_false_and_mask_only() {
        for (field, blocker) in [
            ("block_original_combo: true,", Some("-> (blocked)")),
            (
                "block_original_combo: MaskOnly,",
                Some("-> (masked, experimental)"),
            ),
            ("block_original_combo: false,", None),
            ("", None),
        ] {
            let text =
                format!(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, {field}) ] )"#);
            let bindings = from_ron_str_with_hotkeys(&text, &language_hotkey(ALT_SHIFT))
                .unwrap_or_else(|e| panic!("{field}: {e}"));
            let lines = blocker_lines(&bindings);
            match blocker {
                Some(tail) => {
                    assert_eq!(lines.len(), 1, "{field}: {lines:?}");
                    assert!(lines[0].ends_with(tail), "{field}: {}", lines[0]);
                }
                None => assert_eq!(lines, Vec::<String>::new(), "{field}"),
            }
        }
    }

    #[test]
    fn a_misspelled_block_original_combo_value_names_the_valid_ones() {
        for value in ["Maskonly", "Some(true)"] {
            let text = format!(
                r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: {value}) ] )"#
            );
            let err = from_ron_str_with_hotkeys(&text, &language_hotkey(ALT_SHIFT))
                .expect_err(&format!("{value}: a misspelled value must be an error"));
            assert!(matches!(err, ConfigError::Parse { .. }), "{value}: {err}");
            assert!(
                err.to_string()
                    .contains("expected one of `true`, `false`, or `MaskOnly` instead"),
                "{value}: {err}"
            );
        }
    }

    #[test]
    fn mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns() {
        // The "`" hotkey (registry value "4") is no modifier chord: letting a
        // lone "`" through would type it and switch the layout.
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly),
        ] )"#;
        let parsed = parse_ron_str(text, &language_hotkey(&[VK_OEM_3]))
            .unwrap_or_else(|e| panic!("config does not load: {e}"));
        assert_eq!(
            parsed.warnings,
            [
                "binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true"
            ]
        );
        let lines = blocker_lines(&parsed.bindings);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].ends_with("-> (blocked)"), "{}", lines[0]);
    }

    #[test]
    fn mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing() {
        // KS-18: the "`" binding takes the hotkey over, so no blocker is
        // added, and a warning about how that blocker would block is noise.
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly),
            (keys: "`", action: DoNothing),
        ] )"#;
        let parsed = parse_ron_str(text, &language_hotkey(&[VK_OEM_3]))
            .unwrap_or_else(|e| panic!("config does not load: {e}"));
        assert_eq!(parsed.warnings, Vec::<String>::new());
        assert_eq!(blockers(&parsed.bindings), 0);
    }

    // ---- The blocker's when: (roadmap item 1.2, KS-35) ----

    /// The mode and condition of every auto-blocker of `text`, loaded with
    /// `Alt+Shift` as the language hotkey.
    fn blockers_of(text: &str) -> Vec<(Option<BlockerMode>, Condition)> {
        from_ron_str_with_hotkeys(text, &language_hotkey(ALT_SHIFT))
            .unwrap_or_else(|e| panic!("config does not load: {e}\n{text}"))
            .into_iter()
            .filter(Binding::is_auto_blocker)
            .map(|b| (b.auto_blocker, b.condition))
            .collect()
    }

    fn app(name: &str) -> Condition {
        Condition::AppEquals(name.to_owned())
    }

    #[test]
    fn the_blocker_takes_the_when_of_the_binding_that_asked_for_it() {
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("x.exe")),
        ] )"#;
        let blockers = blockers_of(text);
        assert_eq!(blockers.len(), 1, "{blockers:?}");
        assert_eq!(blockers[0].1, app("x.exe"));
    }

    #[test]
    fn blockers_of_one_hotkey_combine_the_when_of_their_owners() {
        for (name, text, condition) in [
            (
                "two apps",
                r#"( bindings: [
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("a.exe")),
                    (keys: "F13", action: SwitchLanguageBackward, block_original_combo: true, when: AppEquals("b.exe")),
                ] )"#,
                Condition::Or(vec![app("a.exe"), app("b.exe")]),
            ),
            (
                "an owner without when:",
                r#"( bindings: [
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("a.exe")),
                    (keys: "F13", action: SwitchLanguageBackward, block_original_combo: true),
                ] )"#,
                Condition::Always,
            ),
            (
                "one condition twice",
                r#"( bindings: [
                    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("a.exe")),
                    (keys: "F13", action: SwitchLanguageBackward, block_original_combo: true, when: AppEquals("a.exe")),
                ] )"#,
                app("a.exe"),
            ),
        ] {
            assert_eq!(
                blockers_of(text),
                [(Some(BlockerMode::Swallow), condition)],
                "{name}"
            );
        }
    }

    #[test]
    fn a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when() {
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("a.exe")),
            (keys: "F13", action: SwitchLanguageBackward, block_original_combo: MaskOnly, when: AppEquals("b.exe")),
        ] )"#;
        assert_eq!(
            blockers_of(text),
            [
                (Some(BlockerMode::Swallow), app("a.exe")),
                (Some(BlockerMode::MaskOnly), app("b.exe")),
            ]
        );
    }

    #[test]
    fn true_and_mask_only_on_one_hotkey_give_one_blocker_each() {
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
            (keys: "F13", action: SwitchLanguageBackward, block_original_combo: MaskOnly),
            (keys: "F14", action: SwitchLanguage, block_original_combo: MaskOnly),
        ] )"#;
        let bindings = from_ron_str_with_hotkeys(text, &language_hotkey(ALT_SHIFT))
            .unwrap_or_else(|e| panic!("config does not load: {e}"));
        let lines = blocker_lines(&bindings);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].ends_with("-> (blocked)"), "{}", lines[0]);
        assert!(
            lines[1].ends_with("-> (masked, experimental)"),
            "{}",
            lines[1]
        );
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
    fn a_misspelled_bindings_field_is_an_error_that_names_it() {
        // KS-22: serde used to skip the unknown field and default
        // `bindings` to an empty list.
        let err = from_ron_str(r#"( bindngs: [ ( keys: "F13", action: DoNothing ) ] )"#)
            .expect_err("a misspelled `bindings` must be an error");
        assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
        assert!(err.to_string().contains("named `bindngs`"), "{err}");
    }

    #[test]
    fn unknown_fields_are_errors_that_name_the_field() {
        let in_list = |binding: &str| format!("( bindings: [ {binding} ] )");
        for (field, ron) in [
            (
                "version",
                r#"( bindings: [ ( keys: "F13", action: DoNothing ) ], version: 1 )"#.to_owned(),
            ),
            (
                "tigger",
                in_list(r#"( keys: "CapsLock", action: SwitchLanguage, tigger: Tap(200) )"#),
            ),
            (
                "block_defualt",
                in_list(r#"( keys: "CapsLock", action: SwitchLanguage, block_defualt: false )"#),
            ),
            (
                "whne",
                in_list(
                    r#"( keys: "CapsLock", action: SwitchLanguage, whne: AppEquals("code.exe") )"#,
                ),
            ),
            (
                "allow_extra_modifier",
                in_list(
                    r#"( keys: "CapsLock", action: SwitchLanguage, allow_extra_modifier: true )"#,
                ),
            ),
            (
                "repaet",
                in_list(r#"( keys: "F13", action: PressKey("A"), repaet: true )"#),
            ),
            (
                "maxgap_ms",
                in_list(
                    r#"( sequence: (steps: ["F13", "F14"], maxgap_ms: 300), action: DoNothing )"#,
                ),
            ),
            (
                "windw_ms",
                in_list(r#"( chord: (keys: ["F13", "F14"], windw_ms: 80), action: DoNothing )"#),
            ),
            (
                "cwd",
                in_list(r#"( keys: "F13", action: Launch(exe: "notepad.exe", cwd: "C:/temp") )"#),
            ),
            (
                "wparm",
                in_list(r#"( keys: "F13", action: PostMessage(msg: "WM_CLOSE", wparm: 1) )"#),
            ),
            (
                "argz",
                in_list(
                    r#"( keys: "F13", action: Sequence([ Launch(exe: "notepad.exe", argz: ["x"]) ]) )"#,
                ),
            ),
        ] {
            let err = from_ron_str(&ron)
                .expect_err(&format!("{field}: an unknown field must be an error"));
            assert!(matches!(err, ConfigError::Parse { .. }), "{field}: {err}");
            // With the backticks, so that a valid field in RON's list of
            // expected names cannot satisfy it.
            assert!(
                err.to_string().contains(&format!("named `{field}`")),
                "{field}: {err}"
            );
        }
    }

    #[test]
    fn a_config_without_bindings_is_an_error() {
        let err = from_ron_str("()").expect_err("a config without `bindings` must be an error");
        assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
        assert!(err.to_string().contains("`bindings`"), "{err}");
    }

    #[test]
    fn a_utf8_bom_before_the_config_is_ignored() {
        // KS-24: Notepad's "UTF-8 with BOM" and Windows PowerShell 5.1's
        // `-Encoding UTF8` start the file with U+FEFF.
        let bindings =
            from_ron_str("\u{feff}( bindings: [ ( keys: \"F13\", action: DoNothing ) ] )")
                .unwrap_or_else(|e| panic!("a UTF-8 BOM must be ignored: {e}"));
        assert_eq!(bindings.len(), 1);

        let err = from_ron_str("\u{feff}( bindings: [ ( keys: \"F13\", action: DoNothing ) ]")
            .expect_err("a config without its closing parenthesis must be an error");
        assert!(
            err.to_string().contains("1:51:"),
            "positions count from the first character after the BOM: {err}"
        );
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
        let user = bindings.iter().find(|b| !b.is_auto_blocker()).unwrap();
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

    // ---- timing values (KS-27) ----

    /// `row` as binding #1, after a valid binding #0, so that a message
    /// names the binding by its own number.
    fn after_binding_0(row: &str) -> String {
        format!(r#"( bindings: [ ( keys: "F20", action: DoNothing ), {row} ] )"#)
    }

    #[test]
    fn a_zero_tap_term_is_an_error_that_names_the_binding() {
        let err = from_ron_str(&after_binding_0(
            r#"( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(0) )"#,
        ))
        .expect_err("Tap(0) must be an error");
        match err {
            ConfigError::Bindings(errs) => assert_eq!(
                errs,
                ["binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms"]
            ),
            other => panic!("expected Bindings, got {other:?}"),
        }
    }

    #[test]
    fn timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding() {
        let deferred = |trigger: &str| {
            format!(r#"( keys: "CapsLock", action: SwitchLanguage, trigger: {trigger} )"#)
        };
        let sequence = |gap: &str| {
            format!(
                r#"( sequence: (steps: ["F13", "F14"], max_gap_ms: {gap}), action: DoNothing )"#
            )
        };
        let chord = |window: &str| {
            format!(r#"( chord: (keys: ["F13", "F14"], window_ms: {window}), action: DoNothing )"#)
        };
        for (written, row) in [
            ("Hold(0)", deferred("Hold(0)")),
            ("DoubleTap(0)", deferred("DoubleTap(0)")),
            ("Tap(10001)", deferred("Tap(10001)")),
            ("Hold(3600000)", deferred("Hold(3600000)")),
            ("max_gap_ms: 0", sequence("0")),
            ("max_gap_ms: 10001", sequence("10001")),
            ("window_ms: 0", chord("0")),
            ("window_ms: 10001", chord("10001")),
        ] {
            let err = from_ron_str(&after_binding_0(&row))
                .expect_err(&format!("{written} must be an error"));
            match err {
                ConfigError::Bindings(errs) => {
                    assert_eq!(errs.len(), 1, "{written}: {errs:?}");
                    assert!(errs[0].starts_with("binding #1 "), "{written}: {errs:?}");
                    assert!(errs[0].contains(written), "{written}: {errs:?}");
                    assert!(errs[0].contains("1 to 10000 ms"), "{written}: {errs:?}");
                }
                other => panic!("{written}: expected Bindings, got {other:?}"),
            }
        }
    }

    #[test]
    fn timing_values_at_the_bounds_load() {
        let ron = r#"( bindings: [
            (keys: "F13", action: DoNothing, trigger: Tap(1)),
            (keys: "F14", action: DoNothing, trigger: Hold(10000)),
            (keys: "F15", action: DoNothing, trigger: DoubleTap(10000)),
            (sequence: (steps: ["F16", "F17"], max_gap_ms: 1), action: DoNothing),
            (chord: (keys: ["F18", "F19"], window_ms: 10000), action: DoNothing),
        ] )"#;
        let bindings = from_ron_str(ron).unwrap_or_else(|e| panic!("the bounds must load: {e}"));
        assert_eq!(bindings.len(), 5);
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

    // ---- repeat (roadmap item 1.1, KS-10) ----

    /// The bindings that never fire on an auto-repeat, as (name, keys in
    /// the error prefix, the fields that come before the action).
    const NEVER_REPEATING: [(&str, &str, &str); 5] = [
        ("Tap(200)", "F13", r#"keys: "F13", trigger: Tap(200)"#),
        ("Hold(200)", "F13", r#"keys: "F13", trigger: Hold(200)"#),
        (
            "DoubleTap(250)",
            "F13",
            r#"keys: "F13", trigger: DoubleTap(250)"#,
        ),
        (
            "sequence",
            "Sequence[F13 \u{2192} F14]",
            r#"sequence: (steps: ["F13", "F14"])"#,
        ),
        (
            "chord",
            "Chord[F13+F14]",
            r#"chord: (keys: ["F13", "F14"])"#,
        ),
    ];

    #[test]
    fn repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings() {
        for (name, keys, fields) in NEVER_REPEATING {
            let ron =
                format!(r#"( bindings: [ ( {fields}, action: PressKey("A"), repeat: true ) ] )"#);
            let err =
                from_ron_str(&ron).expect_err(&format!("{name}: repeat: true must be an error"));
            match err {
                ConfigError::Bindings(errs) => assert_eq!(
                    errs,
                    [format!(
                        "binding #0 [{keys}]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger"
                    )],
                    "{name}"
                ),
                other => panic!("expected Bindings error for {name}, got {other:?}"),
            }
        }
    }

    #[test]
    fn repeat_false_loads_on_every_kind_and_trigger() {
        // `false` is what deferred triggers, sequences and chords do anyway.
        for (name, _, fields) in [("Immediate", "F13", r#"keys: "F13""#)]
            .into_iter()
            .chain(NEVER_REPEATING)
        {
            let ron =
                format!(r#"( bindings: [ ( {fields}, action: PressKey("A"), repeat: false ) ] )"#);
            let bindings = from_ron_str(&ron)
                .unwrap_or_else(|e| panic!("{name}: repeat: false must load: {e}"));
            assert_eq!(bindings.len(), 1, "{name}");
        }
    }

    #[test]
    fn repeat_defaults_follow_the_action_and_an_explicit_value_wins() {
        let ron = r#"( bindings: [
            (keys: "F13", action: PressKey("A")),
            (keys: "F14", action: Media(VolumeDown)),
            (keys: "F15", action: Media(VolumeMute)),
            (keys: "F16", action: Launch(exe: "wt.exe")),
            (keys: "F17", action: SwitchLanguage, repeat: true),
            (keys: "F18", action: PressKey("A"), repeat: false),
        ] )"#;
        let bindings = from_ron_str_with_hotkeys(ron, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("config loads: {e}"));
        let repeats: Vec<bool> = bindings.iter().map(|b| b.repeat).collect();
        assert_eq!(repeats, [true, true, false, false, true, false]);
    }

    #[test]
    fn an_unknown_binding_field_lists_repeat_among_the_valid_names() {
        // The README and the book quote this message.
        let err = from_ron_str(
            r#"( bindings: [ ( keys: "CapsLock", action: SwitchLanguage, tigger: Tap(200) ) ] )"#,
        )
        .expect_err("an unknown field must be an error");
        assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
        assert!(
            err.to_string()
                .contains("`trigger`, `allow_extra_modifiers`, or `repeat` instead"),
            "{err}"
        );
    }

    #[test]
    fn display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default() {
        let ron = r#"( bindings: [
            (keys: "F13", action: SwitchLanguage, repeat: true),
            (keys: "F14", action: PressKey("Left"), repeat: false),
            (keys: "F15", action: PressKey("Left")),
            (keys: "F16", action: SwitchLanguage, repeat: false),
            (keys: "F17", action: PressKey("Left"), trigger: Tap(200), repeat: false),
        ] )"#;
        let bindings = from_ron_str_with_hotkeys(ron, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("config loads: {e:?}"));
        let lines: Vec<String> = bindings.iter().map(Binding::to_string).collect();
        assert!(lines[0].ends_with("  [repeat]"), "{}", lines[0]);
        assert!(lines[1].ends_with("  [no repeat]"), "{}", lines[1]);
        for line in &lines[2..] {
            assert!(
                !line.contains("repeat]"),
                "only an override is marked: {line}"
            );
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

    #[test]
    fn a_missing_command_line_config_is_an_error_and_is_not_created() {
        let dir = TempDir::new("missing-cli");
        let path = dir.path().join("my.ron");
        let err = load(&ConfigPath::CommandLine(path.clone()))
            .expect_err("a missing command-line config must be an error");
        assert!(
            !path.exists(),
            "a missing command-line config must not be created"
        );
        assert!(err.to_string().contains("does not exist"), "{err}");
    }

    #[test]
    fn a_missing_folder_of_a_command_line_config_is_not_created() {
        let dir = TempDir::new("missing-folder");
        let path = dir.path().join("nope").join("my.ron");
        load(&ConfigPath::CommandLine(path))
            .expect_err("a command-line config in a missing folder must be an error");
        assert!(
            !dir.path().join("nope").exists(),
            "the folder of a missing command-line config must not be created"
        );
    }

    #[test]
    fn reload_of_a_missing_config_is_an_error_and_writes_nothing() {
        let dir = TempDir::new("missing-reload");
        let path = dir.path().join("config.ron");
        let err = reload(&path).expect_err("reloading a missing config must be an error");
        assert!(
            !path.exists(),
            "reloading a missing config must not create it"
        );
        assert!(err.to_string().contains("does not exist"), "{err}");
    }

    #[test]
    fn reload_reads_a_file_saved_with_a_bom() {
        let dir = TempDir::new("bom");
        let path = dir.path().join("config.ron");
        fs::write(&path, [b"\xEF\xBB\xBF".as_slice(), ONE.as_bytes()].concat())
            .expect("write the config");
        let bindings =
            reload(&path).unwrap_or_else(|e| panic!("a file saved with a BOM must load: {e}"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn a_missing_config_beside_the_exe_is_generated_and_loaded() {
        let dir = TempDir::new("beside-exe");
        let path = dir.path().join("config.ron");
        let bindings = load(&ConfigPath::BesideExe(path.clone()))
            .expect("the template is generated and loads");
        assert!(!bindings.is_empty(), "the template has bindings");
        assert_eq!(
            fs::read_to_string(&path).expect("the template was written"),
            DEFAULT_CONFIG
        );
    }

    // ---- Duplicate bindings (roadmap item 1.6) ----

    /// The warnings of `text`, loaded with no system hotkey.
    fn warnings_of(text: &str) -> Vec<String> {
        parse_ron_str(text, &|_: &BindAction| None)
            .unwrap_or_else(|e| panic!("config does not load: {e}\n{text}"))
            .warnings
    }

    #[test]
    fn a_binding_that_duplicates_an_earlier_one_is_warned_about() {
        // The later CapsLock Tap never acts: the first one of its group
        // takes the press. The file still loads, every binding included.
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "Shift+CapsLock", action: ToggleCapsLock),
            (keys: "capslock", action: ToggleCapsLock, trigger: Tap(300), block_default: false),
        ] )"#;
        let parsed = parse_ron_str(text, &|_: &BindAction| None).expect("config loads");
        assert_eq!(parsed.bindings.len(), 3);
        assert_eq!(
            parsed.warnings,
            ["binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them"]
        );
    }

    #[test]
    fn bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates()
    {
        let text = r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "CapsLock", action: SwitchLanguage, trigger: Hold(200)),
            (keys: "CapsLock", action: SwitchLanguage, trigger: DoubleTap(250)),
            (keys: "CapsLock", action: SwitchLanguage),
            (keys: "CapsLock", action: SwitchLanguage, when: AppEquals("code.exe")),
            (keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true),
            (keys: "LShift+CapsLock", action: SwitchLanguage),
            (keys: "Shift+CapsLock", action: SwitchLanguage),
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (sequence: (steps: ["F14", "F13"]), action: SwitchLanguage),
            (chord: (keys: ["F13", "F14"]), action: SwitchLanguage),
            (chord: (keys: ["F13", "F15"]), action: SwitchLanguage),
        ] )"#;
        assert_eq!(warnings_of(text), Vec::<String>::new());
    }

    #[test]
    fn duplicates_are_found_across_key_order_terms_timings_and_binding_kinds() {
        let text = r#"( bindings: [
            (keys: "Ctrl+Alt+L", action: PressKey("L")),
            (keys: "alt + ctrl + l", action: Launch(exe: "wt.exe")),
            (keys: "F13", action: SwitchLanguage, trigger: Hold(200), when: AppEquals("code.exe")),
            (keys: "F13", action: ToggleCapsLock, trigger: Hold(500), when: AppEquals("code.exe")),
            (sequence: (steps: ["F14", "F15"]), action: SwitchLanguage),
            (sequence: (steps: ["F14", "F15"], max_gap_ms: 900), action: ToggleCapsLock),
            (chord: (keys: ["F16", "F17"]), action: SwitchLanguage),
            (chord: (keys: ["F17", "F16"], window_ms: 80), action: ToggleCapsLock),
            (keys: "Ctrl+Alt+L", action: DoNothing),
        ] )"#;
        assert_eq!(
            warnings_of(text),
            [
                "binding #1 [alt + ctrl + l] duplicates binding #0 [Ctrl+Alt+L]; remove one of them",
                "binding #3 [F13] duplicates binding #2 [F13]; remove one of them",
                "binding #5 [Sequence[F14 \u{2192} F15]] duplicates binding #4 [Sequence[F14 \u{2192} F15]]; remove one of them",
                "binding #7 [Chord[F17+F16]] duplicates binding #6 [Chord[F16+F17]]; remove one of them",
                "binding #8 [Ctrl+Alt+L] duplicates binding #0 [Ctrl+Alt+L]; remove one of them",
            ]
        );
    }

    #[test]
    fn the_template_and_its_examples_have_no_duplicate_bindings() {
        assert_eq!(warnings_of(DEFAULT_CONFIG), Vec::<String>::new());
        assert_eq!(warnings_of(&template_examples_ron()), Vec::<String>::new());
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
