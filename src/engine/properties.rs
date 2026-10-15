//! Property tests of the suppression invariant: generated scripts of
//! key-downs, auto-repeats, key-ups, clock steps and focus changes, run
//! through the scenario harness against configs that cover every binding
//! kind and trigger.
//!
//! The scripts come from an in-house SplitMix64 generator with fixed seeds,
//! so every run is the same, and a failure names its config, seed and mode
//! (`Scenario::with_context`) above the transcript. Two properties:
//!
//! - Strict mode, with no focus change and no lost key-up: every press is
//!   suppressed or passed as a whole, its key-down, auto-repeats and key-up
//!   alike (`check_press_symmetry`), and no key is left down in Windows
//!   (`check_nothing_stuck`).
//! - Focus mode, which adds foreground changes and key-ups the hook never
//!   sees: no key is left down in Windows. A focus change makes the engine
//!   let through the key-up of a press whose key-down it swallowed (roadmap
//!   item 2.3), so whole-press symmetry is not checked there.

use super::scenarios::with_alt_shift_blocker;
use super::testing::Scenario;

/// SplitMix64 (Steele, Lea and Flood, 2014): a tiny generator whose output
/// is fixed by its seed, which is all a script generator needs.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. The modulo bias is negligible for an `n` this
    /// small.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

const SEEDS: u64 = 64;
const STEPS: usize = 250;

const KEYS: [&str; 13] = [
    "LShift", "RShift", "LCtrl", "LAlt", "CapsLock", "F13", "F14", "F15", "F16", "J", "K", "Space",
    "A",
];

/// Clock steps: auto-repeat intervals, the repeat delay and both sides of
/// the configs' terms and windows.
const DELAYS_MS: [u64; 13] = [0, 1, 20, 33, 49, 50, 199, 200, 201, 250, 500, 600, 1000];

struct Config {
    name: &'static str,
    /// Put the `Alt+Shift` auto-blocker in front, as the loader does for
    /// `block_original_combo`.
    blocker: bool,
    ron: &'static str,
}

const CONFIGS: [Config; 6] = [
    Config {
        name: "default",
        blocker: true,
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage),
            (keys: "Shift+CapsLock", action: ToggleCapsLock),
            (keys: "Alt+Shift+K", action: ToggleCapsLock),
        ] )"#,
    },
    Config {
        name: "immediate",
        blocker: false,
        ron: r#"( bindings: [
            (keys: "Ctrl+Alt+J", action: ToggleCapsLock),
            (keys: "F13", action: PressKey("Left")),
            (keys: "F14", action: ToggleCapsLock, block_default: false),
            (keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true),
            (keys: "Shift+Space", action: SwitchLanguage),
        ] )"#,
    },
    Config {
        name: "gestures",
        blocker: false,
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(600)),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)),
            (keys: "RShift", action: SwitchLanguage, trigger: Tap(300)),
        ] )"#,
    },
    Config {
        name: "chords",
        blocker: false,
        ron: r#"( bindings: [
            (chord: (keys: ["Shift", "Space"], window_ms: 500), action: SwitchLanguage),
            (chord: (keys: ["J", "K"], window_ms: 50), action: ToggleCapsLock),
            (chord: (keys: ["F13", "F14"], window_ms: 2000), action: SwitchLanguage),
        ] )"#,
    },
    Config {
        name: "sequences",
        blocker: false,
        ron: r#"( bindings: [
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (sequence: (steps: ["F14+F15", "F14+F16"]), action: ToggleCapsLock),
            (sequence: (steps: ["Ctrl+J", "Ctrl+K"]), action: SwitchLanguage),
        ] )"#,
    },
    Config {
        name: "mixed",
        blocker: true,
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage),
            (keys: "Shift+CapsLock", action: ToggleCapsLock),
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(600)),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)),
            (chord: (keys: ["J", "K"], window_ms: 50), action: ToggleCapsLock),
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (sequence: (steps: ["F14+F15", "F14+F16"]), action: ToggleCapsLock),
        ] )"#,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Strict,
    Focus,
}

/// A random key of [`KEYS`] that the script holds (`held == true`) or has
/// up, if there is one.
fn pick(rng: &mut SplitMix64, held: &[bool; KEYS.len()], want_held: bool) -> Option<usize> {
    let candidates: Vec<usize> = (0..KEYS.len()).filter(|&i| held[i] == want_held).collect();
    (!candidates.is_empty()).then(|| candidates[rng.below(candidates.len())])
}

fn press(scenario: Scenario, held: &mut [bool; KEYS.len()], index: usize) -> Scenario {
    held[index] = true;
    scenario.down(KEYS[index])
}

/// Appends one generated step to the script.
fn step(
    scenario: Scenario,
    rng: &mut SplitMix64,
    held: &mut [bool; KEYS.len()],
    mode: Mode,
) -> Scenario {
    let delay = |rng: &mut SplitMix64| DELAYS_MS[rng.below(DELAYS_MS.len())];
    match rng.below(100) {
        0..35 => match pick(rng, held, false) {
            Some(index) => press(scenario, held, index),
            None => {
                let index = pick(rng, held, true).expect("invariant: some key is held");
                scenario.repeat(KEYS[index])
            }
        },
        35..60 => match pick(rng, held, true) {
            Some(index) => scenario.repeat(KEYS[index]),
            None => {
                let index = pick(rng, held, false).expect("invariant: some key is up");
                press(scenario, held, index)
            }
        },
        60..85 => match pick(rng, held, true) {
            Some(index) => {
                held[index] = false;
                if mode == Mode::Focus && rng.below(10) == 0 {
                    scenario.up_unseen(KEYS[index])
                } else {
                    scenario.up(KEYS[index])
                }
            }
            None => {
                let index = pick(rng, held, false).expect("invariant: some key is up");
                press(scenario, held, index)
            }
        },
        85..95 => scenario.advance_ms(delay(rng)),
        95..97 if mode == Mode::Focus => scenario.foreground_changed(),
        95..97 => scenario.advance_ms(delay(rng)),
        _ => scenario.advance_ms_stalled(delay(rng)),
    }
}

/// Runs one generated script: `STEPS` steps, then every held key released
/// and every timer run, with the checks of `mode` on throughout.
fn run(index: usize, config: &Config, seed: u64, mode: Mode) {
    let mut rng = SplitMix64(seed.wrapping_mul(0x100) + index as u64);
    let scenario = if config.blocker {
        Scenario::with_bindings(with_alt_shift_blocker(config.ron))
    } else {
        Scenario::from_ron(config.ron)
    };
    let mut scenario = scenario
        .with_context(format!(
            "config {}, seed {seed}, {mode:?} mode",
            config.name
        ))
        .check_nothing_stuck();
    if mode == Mode::Strict {
        scenario = scenario.check_press_symmetry();
    }

    let mut held = [false; KEYS.len()];
    for _ in 0..STEPS {
        scenario = step(scenario, &mut rng, &mut held, mode);
    }
    for (index, key) in KEYS.iter().enumerate() {
        if held[index] {
            scenario = scenario.up(key);
        }
    }
    scenario.advance_ms(2000).expect_nothing_stuck();
}

fn run_all(mode: Mode) {
    for (index, config) in CONFIGS.iter().enumerate() {
        for seed in 0..SEEDS {
            run(index, config, seed, mode);
        }
    }
}

#[test]
fn every_press_is_suppressed_or_passed_as_a_whole() {
    run_all(Mode::Strict);
}

#[test]
fn nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups() {
    run_all(Mode::Focus);
}
