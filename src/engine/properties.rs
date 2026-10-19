//! Property tests of the suppression invariant: generated scripts of
//! key-downs, auto-repeats, key-ups, clock steps and focus changes, run
//! through the scenario harness against configs that cover every binding
//! kind and trigger. Their `Tap`, `Hold` and `DoubleTap` bindings include
//! ones that swallow their key and ones that let it through
//! (`block_default`, roadmap item 1.3), so a press is checked on both
//! paths. The `winners` config puts bindings on the same key that compete
//! for one press (roadmap item 1.6): a more specific or conditional winner
//! that lets the key through next to a loser that would swallow it, and a
//! deferred winner next to an `Immediate` loser. The `mask_only` config is
//! the `default` one with the experimental `MaskOnly` auto-blocker in place
//! of the one that swallows (roadmap item 1.2, KS-05). The `tap_double_tap`
//! config puts a Tap, a Hold and a DoubleTap on one key, and a Tap and a
//! DoubleTap on a passing modifier, so that a Tap waits for its DoubleTap
//! window and another key-down ends it (roadmap item 2.2).
//!
//! The scripts come from an in-house SplitMix64 generator with fixed seeds,
//! so every run is the same, and a failure names its config, seed and mode
//! (`Scenario::with_context`) above the transcript. Three properties, and in
//! every run but `mask_only`'s the menu mask rule (`check_mask_rule`,
//! KS-12): a key event asks for the mask key exactly when it is suppressed
//! while the modelled OS has Alt down. A `MaskOnly` blocker also masks
//! events it lets through.
//!
//! - Strict mode, with no focus change and no lost key-up: every press is
//!   suppressed or passed as a whole, its key-down, auto-repeats and key-up
//!   alike (`check_press_symmetry`), and no key is left down in Windows
//!   (`check_nothing_stuck`).
//! - Focus mode, which adds foreground changes and key-ups the hook never
//!   sees: no key is left down in Windows. A focus change makes the engine
//!   let through the key-up of a press whose key-down it swallowed (roadmap
//!   item 2.3), so whole-press symmetry is not checked there.
//! - Focus mode again: an auto-repeat fires only an action that repeats
//!   (`check_repeats_fire_only`, KS-10). `PressKey("Left")` is the only
//!   `Immediate` action of the configs that repeats by default; deferred
//!   triggers, chords and sequences fire nothing on a repeat. A `repeat`
//!   step is always an auto-repeat for the engine too: the scripts never
//!   press a key the hook does not see, a held key stays held for the
//!   engine until a key-up it sees, and the modifier resync skips a
//!   swallowed modifier and finds a passed one down in Windows.
//! - Unseen mode, the Focus steps plus key-downs the hook never sees (a
//!   higher-integrity window, the secure desktop; roadmap item 2.3, ruling
//!   R1): no key is left down in Windows. The release of a swallowed key
//!   that was pressed again unseen reaches Windows because Windows has the
//!   key down. Such an Alt or Win does not count for the menu mask (known
//!   issue 78), so the mask rule is not checked, nor is symmetry.

use windows::Win32::UI::Input::KeyboardAndMouse::VK_LEFT;

use super::scenarios::{with_alt_shift_blocker, with_alt_shift_mask_only_blocker};
use super::testing::Scenario;
use crate::core::windows_actions::BindAction;
use crate::data::binding::BlockerMode;

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
    /// Put the `Alt+Shift` auto-blocker that `block_original_combo` adds in
    /// front of the bindings, of this mode (`with_alt_shift_blocker`,
    /// `with_alt_shift_mask_only_blocker`).
    blocker: Option<BlockerMode>,
    ron: &'static str,
}

/// The `default` config's bindings.
const DEFAULT_BINDINGS: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage),
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
    (keys: "Alt+Shift+K", action: ToggleCapsLock),
] )"#;

const CONFIGS: [Config; 9] = [
    Config {
        name: "default",
        blocker: Some(BlockerMode::Swallow),
        ron: DEFAULT_BINDINGS,
    },
    Config {
        name: "immediate",
        blocker: None,
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
        blocker: None,
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(600)),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000), block_default: true),
            (keys: "RShift", action: SwitchLanguage, trigger: Tap(300)),
            (keys: "F15", action: SwitchLanguage, trigger: Tap(200), block_default: false),
        ] )"#,
    },
    Config {
        name: "chords",
        blocker: None,
        ron: r#"( bindings: [
            (chord: (keys: ["Shift", "Space"], window_ms: 500), action: SwitchLanguage),
            (chord: (keys: ["J", "K"], window_ms: 50), action: ToggleCapsLock),
            (chord: (keys: ["F13", "F14"], window_ms: 2000), action: SwitchLanguage),
        ] )"#,
    },
    Config {
        name: "sequences",
        blocker: None,
        ron: r#"( bindings: [
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (sequence: (steps: ["F14+F15", "F14+F16"]), action: ToggleCapsLock),
            (sequence: (steps: ["Ctrl+J", "Ctrl+K"]), action: SwitchLanguage),
        ] )"#,
    },
    Config {
        name: "mixed",
        blocker: Some(BlockerMode::Swallow),
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage),
            (keys: "Shift+CapsLock", action: ToggleCapsLock),
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(600), block_default: true),
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)),
            (chord: (keys: ["J", "K"], window_ms: 50), action: ToggleCapsLock),
            (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage),
            (sequence: (steps: ["F14+F15", "F14+F16"]), action: ToggleCapsLock),
        ] )"#,
    },
    Config {
        name: "winners",
        blocker: None,
        ron: r#"( bindings: [
            (keys: "Shift+CapsLock", action: ToggleCapsLock),
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true),
            (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), allow_extra_modifiers: true),
            (keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true),
            (keys: "Shift+F13", action: ToggleCapsLock, block_default: false),
            (keys: "F14", action: PressKey("Left"), allow_extra_modifiers: true),
            (keys: "Ctrl+F14", action: ToggleCapsLock, trigger: Tap(200), block_default: false),
            (keys: "F15", action: SwitchLanguage),
            (keys: "F15", action: ToggleCapsLock, block_default: false, when: Not(AppEquals("game.exe"))),
        ] )"#,
    },
    // These come last, in the order they were added, so that the scripts
    // of the configs above keep their seeds.
    Config {
        name: "mask_only",
        blocker: Some(BlockerMode::MaskOnly),
        ron: DEFAULT_BINDINGS,
    },
    Config {
        name: "tap_double_tap",
        blocker: None,
        ron: r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(400)),
            (keys: "CapsLock", action: ToggleCapsLock, trigger: DoubleTap(300)),
            (keys: "LShift", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Strict,
    Focus,
    /// The Focus steps, and key-downs the hook does not see (roadmap item
    /// 2.3, ruling R1).
    Unseen,
}

impl Mode {
    /// Whether the script changes the foreground and loses key-ups.
    fn loses_events(self) -> bool {
        matches!(self, Mode::Focus | Mode::Unseen)
    }
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
            Some(index) if mode == Mode::Unseen && rng.below(10) == 0 => {
                held[index] = true;
                scenario.down_unseen(KEYS[index])
            }
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
                if mode.loses_events() && rng.below(10) == 0 {
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
        95..97 if mode.loses_events() => scenario.foreground_changed(),
        95..97 => scenario.advance_ms(delay(rng)),
        _ => scenario.advance_ms_stalled(delay(rng)),
    }
}

/// Runs one generated script: `STEPS` steps, then every held key released
/// and every timer run, with the checks of `mode` on throughout, and the
/// auto-repeat check too with `repeat_check`.
fn run(index: usize, config: &Config, seed: u64, mode: Mode, repeat_check: bool) {
    let mut rng = SplitMix64(seed.wrapping_mul(0x100) + index as u64);
    let scenario = match config.blocker {
        Some(BlockerMode::Swallow) => Scenario::with_bindings(with_alt_shift_blocker(config.ron)),
        Some(BlockerMode::MaskOnly) => {
            Scenario::with_bindings(with_alt_shift_mask_only_blocker(config.ron))
        }
        None => Scenario::from_ron(config.ron),
    };
    let mut scenario = scenario
        .with_context(format!(
            "config {}, seed {seed}, {mode:?} mode",
            config.name
        ))
        .check_nothing_stuck();
    // A `MaskOnly` blocker masks events it lets through, which the rule
    // does not cover. An Alt or Win pressed again where the hook cannot see
    // it, after its swallowed key-down, does not count for the mask
    // (known issue 78), so the rule is not checked with unseen presses.
    if config.blocker != Some(BlockerMode::MaskOnly) && mode != Mode::Unseen {
        scenario = scenario.check_mask_rule();
    }
    if mode == Mode::Strict {
        scenario = scenario.check_press_symmetry();
    }
    if repeat_check {
        scenario = scenario.check_repeats_fire_only(&[BindAction::PressKey(VK_LEFT)]);
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

fn run_all(mode: Mode, repeat_check: bool) {
    for (index, config) in CONFIGS.iter().enumerate() {
        for seed in 0..SEEDS {
            run(index, config, seed, mode, repeat_check);
        }
    }
}

#[test]
fn every_press_is_suppressed_or_passed_as_a_whole() {
    run_all(Mode::Strict, false);
}

#[test]
fn nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups() {
    run_all(Mode::Focus, false);
}

#[test]
fn auto_repeats_fire_only_actions_that_repeat() {
    run_all(Mode::Focus, true);
}

#[test]
fn nothing_is_left_down_in_windows_when_the_hook_misses_key_downs() {
    run_all(Mode::Unseen, false);
}
