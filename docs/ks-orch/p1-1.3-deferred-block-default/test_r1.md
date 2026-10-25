# Tester: p1-1.3-deferred-block-default / roadmap 1.3 (HEAD 1562acd vs master 1afa720)

Verdict: GREEN. Gate passes, 18 red-first tests are red on master by assertion and green on HEAD, six own mutations were each caught, all four acceptance criteria map to a passing scenario test.

HEAD `1562acd29e6ced26320867397f5c1dba5a81f3cd`, branch `feat/deferred-block-default`, `git status --short` empty before and after.

## Gate (HEAD)
- `cargo fmt --check`: OK (exit 0)
- `cargo clippy --all-targets --locked -- -D warnings`: OK. First run finished in 0.32 s (possibly stale), so I touched `src/lib.rs`, `src/main.rs`, `src/bin/swch.rs` and re-ran: `Checking key-switch-rs ... Finished` exit 0.
- `cargo test --locked --no-fail-fast`: 417 passed, 0 failed, 1 ignored in total.
  - lib: `running 412 tests` / `test result: ok. 411 passed; 0 failed; 1 ignored`
  - swch: `running 0 tests` (no unit tests in the binary, expected)
  - `tests/ipc_roundtrip.rs`: `running 6 tests` / `test result: ok. 6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests` (none exist)
  - Ignored: `data::condition::tests::foreground_context_names_the_focused_notepad` (`manual: start it, then focus a Notepad window within 5 s ...`, pre-existing).
- `cargo build --release --locked`: OK (`Finished release`), not run.

## Red-first replay (master 1afa720 + commit 6950476 test changes)
Method: scratch worktree `wt/master` at `1afa720`, `git diff 5adf19c 6950476` (the declared-red test commit: loader.rs tests, properties.rs, scenarios.rs, testing.rs; no production file) applied with `git apply`, own `CARGO_TARGET_DIR=.../target-wt/master`. It compiled, so every red is by assertion, not compile error. Log: `logs/tester_master_red.log`.
Master + tests: `running 410 tests` / `test result: FAILED. 391 passed; 18 failed; 1 ignored`. (410 = 412 minus the two unit tests for `is_modifier_only` and `blocks_by_default`, which do not exist on master; they are carried by mutations M2/M3 below.)

| Test | Master | HEAD | First failing line on master |
|---|---|---|---|
| loader::block_default_defaults_to_false_only_for_deferred_bindings_made_only_of_modifiers (L1) | red (assert) | ok | loader.rs:719 `left: [true, true, true, true, true, true, true, true, true, true]` |
| loader::block_default_takes_some_and_none_like_the_other_optional_fields (L3) | red | ok | loader.rs:750 parse error `Expected boolean` |
| loader::the_template_lshift_double_tap_example_lets_the_left_shift_through (L4) | red | ok | `assertion failed: !example.block_default` |
| scenarios::explicit_block_default_false_on_a_tap_passes_every_event_and_still_fires (a) | red | ok | `down F13 at +0 ms: expected the event to be passed, it was suppressed` |
| ..._on_a_hold_passes_every_event_and_still_fires (b) | red | ok | same, F13 |
| ..._on_a_double_tap_passes_every_event_and_still_fires (c) | red | ok | same, F13 |
| lshift_double_tap_fires_and_every_lshift_event_passes (d, acceptance 1) | red | ok | `down LShift at +0 ms: expected ... passed` |
| lshift_held_with_its_double_tap_bound_types_a_capital_a (e, acceptance 2) | red | ok | `down LShift at +0 ms` |
| passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too (h) | red | ok | LShift |
| one_long_press_of_a_passing_shift_does_not_fire_its_double_tap (i) | red | ok | LShift |
| modifier_only_hold_and_tap_pass_their_keys_by_default (j) | red | ok | LShift |
| deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default (k) | red | ok | `down LShift at +20 ms` |
| passing_tap_fires_after_a_foreground_change_and_its_key_up_passes (m1) | red | ok | F13 |
| remapping_off_during_a_passing_hold_drops_it_and_nothing_sticks (m2) | red | ok | F13 |
| reload_during_a_passing_double_tap_keeps_each_press_whole (m3) | red | ok | LShift |
| lshift_double_tap_fires_only_for_the_left_shift (flip) | red | ok | `down LShift at +0 ms` |
| a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap (I1-a) | red | ok | LShift |
| a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap (I1-b) | red | ok | `down LCtrl at +0 ms` |

The 18 failing names on master equal the 18 in impl.md and `commit2_red.log`. Guards green on both (as documented): capslock_tap_with_the_default_block_default_still_swallows_capslock (f), explicit_block_default_true_on_a_modifier_only_double_tap_swallows (g), an_explicit_block_default_wins_over_its_default (L2), down_in_windows_check_reports_a_key_windows_does_not_have_down, the D9 scenarios, engine::properties::*. Guards are exercised by the mutations below (f by M6, g by M4, properties by M5).
HEAD run: 411 passed (lib), 0 failed.
Worktrees removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 1562acd [feat/deferred-block-default]`; `wt/` is empty.

## Mutations (scratch worktrees `wt/mut`, `wt/mut2`, `CARGO_TARGET_DIR=.../target-wt/mut`; lib tests, `running 412 tests` each)
| # | File | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | src/engine/mod.rs:466 | DoubleTap gate `if binding.block_default` -> `if true` | lshift_double_tap_fires_and_every_lshift_event_passes (acceptance 1), c | caught: 8 failed incl. both of those, lshift_held_with_its_double_tap_bound_types_a_capital_a, reload_during_a_passing_double_tap_keeps_each_press_whole |
| M2 | src/data/binding.rs:131 | `blocks_by_default` drops the `trigger.is_immediate() ||` clause | blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers, L1 | caught: 4 failed incl. both named plus user_binding_on_the_language_hotkey_fires_in_either_file_order |
| M3 | src/data/key_combination.rs:76 | `is_modifier_only`: `all` -> `any` | is_modifier_only_needs_every_key_to_be_a_modifier, matching::tests::trigger_key_* | caught: 8 failed incl. the unit test, L1, trigger_key_is_a_non_modifier_key_of_the_combo, sided_trigger_needs_its_side |
| M4 | src/config/loader.rs:385 | loader ignores an explicit `block_default`, always uses the default rule | an_explicit_block_default_wins_over_its_default (L2), explicit_block_default_true_on_a_modifier_only_double_tap_swallows (g, acceptance 4) | caught: 18 failed incl. both named |
| M5 | src/engine/mod.rs:579 | key-up rule `was_blocked || (gesture_bound && !was_passed)` -> `was_blocked || gesture_bound` (suppression invariant) | key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending, engine::properties::every_press_is_suppressed_or_passed_as_a_whole | caught: 10 failed incl. both named and all 3 properties |
| M6 | src/data/binding.rs:131 | `blocks_by_default` returns `trigger.is_immediate()` only (every deferred combo defaults to passing) | capslock_tap_with_the_default_block_default_still_swallows_capslock (acceptance 3) | caught: 32 failed incl. the named test |

Each reverted; `git diff --exit-code` clean in the scratch worktree before removal; both worktrees removed. (`git checkout -- .` was refused by the permission layer in this session, so reverts were done by reversing the edit and confirmed with `git diff --exit-code`.) The lane tree was never touched.

## Acceptance (spec + brief; roadmap item 1.3)
| Criterion | Evidence | Status |
|---|---|---|
| Double LShift with DoubleTap(250) fires ToggleCapsLock, every LShift event passes | engine::scenarios::lshift_double_tap_fires_and_every_lshift_event_passes ... ok; also lshift_double_tap_fires_only_for_the_left_shift ... ok (`running 2 tests`, 2 passed); red on master, caught by M1 | verified |
| With LShift held, A passes together with Shift, nothing swallowed | engine::scenarios::lshift_held_with_its_double_tap_bound_types_a_capital_a ... ok (`running 1 test`); red on master, caught by M1 | verified |
| CapsLock Tap with default block_default still swallows CapsLock | engine::scenarios::capslock_tap_with_the_default_block_default_still_swallows_capslock ... ok (`running 1 test`); caught by M6 | verified |
| Explicit block_default: true on a modifier-only DoubleTap swallows | engine::scenarios::explicit_block_default_true_on_a_modifier_only_double_tap_swallows ... ok (`running 1 test`); caught by M4 | verified |
| block_default is Option<bool> in the schema | L3 `block_default_takes_some_and_none_like_the_other_optional_fields` ok (red on master); schema.rs:49-50 | verified |
| Default true with a non-modifier key, false for modifier-only combos | L1, `blocks_by_default_*` unit test, `is_modifier_only_*` unit test; caught by M2, M3, M6 | verified |
| block_default false: every event of the key reaches the OS, gesture still detected, up suppressed iff down | scenarios a, b, c, h, m1-m3 plus `check_press_symmetry` / `check_nothing_stuck` and engine::properties (3, ok); caught by M5 | verified |
| Immediate semantics (default true) unchanged | existing Immediate scenarios and `block_default_false_fires_and_lets_both_edges_through` unchanged and green; M2/M6 break them | verified |
| README, default_config.ron (LShift DoubleTap marked working), book config pages, CHANGELOG | L4 pins the template example (`block_default` false, red on master); README/CHANGELOG/book text not machine-checked here (reviewer scope); `default_config_commented_examples_load` ok in the lib run | template verified; prose not verified by test |
| Real-key behaviour: Shift stays usable, Caps Lock toggles on double LShift, no stuck key (impl.md M1-M3, smoke #6) | no `#[ignore]` test added; needs live hook | manual, not verified |

Manual-only: the real Windows behaviour of `carry_out` injecting ToggleCapsLock before the passed second LShift down (Fact 25), and impl.md M1-M3 / roadmap smoke #6. The engine decision part of all of these is covered by scenarios, so no criterion that a pure function can express lacks a test.

## Failures
None.

## Tests added by the tester (`Tests:` commits)
None. No gap found; no commit made. Lane tree clean.

## Notes
- Safety: no daemon, `swch`, ignored test, hook, `SendInput` or registry write was run. A grep of the `+` lines of `git diff master...HEAD -- src tests` finds no production pipe name, `SendInput`, registry, `Instant::now`, `sleep(` or new `#[ignore]`. All new scenarios use the fake clock (`engine::testing`), so no timing reruns were needed.
- Known issue I1 (a key typed between two passing LShift presses still completes the DoubleTap; quick Ctrl shortcut still fires a passing Ctrl Tap) is pinned by two scenarios as current behaviour, documented in README/CHANGELOG/book. Owner question (interrupt rule: 2.2 acceptance or new item) stays open. It is a design limitation, not a test defect.
- M1 note from the cut (implicit `block_default` false on a modifier-only deferred binding wins over its own `block_original_combo: true` because the dedup drops the blocker) is pinned by scenario k's second config.
- Not covered by me: book/README/CHANGELOG prose and `mdbook build`, left to the reviewer / doc checks.
- Artifacts: `D:/tmp/ks-orch/p1-1.3-deferred-block-default/commit2.patch`, `logs/tester_master_red.log`, `target-wt/` (outside the repo).
