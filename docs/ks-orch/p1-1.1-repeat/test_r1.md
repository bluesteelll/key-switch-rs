# Tester: p1-1.1-repeat / roadmap 1.1 (KS-10) (HEAD 10bcf94 = ad96e73 + one Tests: commit; master dbd55ca)

Verdict: GREEN. Gate, red-first replay, 7 own mutations and the acceptance map are clean. One weak pin was found (sequence/chord/Hold guards, see M8 to M10) and closed by a tester commit.

Tree: D:/key-switch-rs/key-switch-rs, branch feat/repeat-flag, clean (`git status --short` empty) at ad96e73 when the gate first ran, and at 10bcf94 now.

## Gate (HEAD ad96e73, re-run on 10bcf94)
- cargo fmt --check: OK (exit 0, both heads)
- cargo clippy --all-targets --locked -- -D warnings: OK. Sources touched first, then `Checking key-switch-rs ... Finished` (6.21 s on ad96e73, 1.76 s on 10bcf94 after touch of src/lib.rs, a real re-check).
- cargo test --locked --no-fail-fast on ad96e73: lib `running 385 tests` -> `384 passed; 0 failed; 1 ignored`; bin/swch `running 0 tests`; tests/ipc_roundtrip `running 6 tests` -> `6 passed`; doc-tests `running 0 tests` (nothing to run, expected).
- cargo test --locked --no-fail-fast on 10bcf94: lib `running 387 tests` -> `386 passed; 0 failed; 1 ignored`; ipc_roundtrip 6 passed.
- Ignored (1, pre-existing): `data::condition::tests::foreground_context_names_the_focused_notepad`, "manual: start it, then focus a Notepad window within 5 s". No `--ignored` run.
- cargo build --release --locked: OK (`Finished release`, not run).
- tests/ipc_roundtrip.rs is not in the delta (`git diff --stat master..HEAD -- tests` empty); the delta adds no SendInput, unsafe, static mut, Mutex or thread::spawn (grep of added src lines: no match). Log: D:/tmp/ks-orch/p1-1.1-repeat/gate_test.log.

## Red-first replay
The red commits 5c43211 and 6a8e20c add tests only (diffstat: loader.rs/scenarios.rs; properties.rs/scenarios.rs/testing.rs), so they are master (dbd55ca) plus the tests, resp. 96528bb plus tests (engine = master). Replayed with `cargo test --locked --lib` in scratch worktrees wt/red1 (5c43211) and wt/red2 (6a8e20c), own CARGO_TARGET_DIR under target-wt. Logs red1.log, red2.log. All failures are assertion panics (code compiled).

5c43211 (parent master): `running 368 tests`, `test result: FAILED. 362 passed; 5 failed; 1 ignored`. Exactly:

| Test | Parent | HEAD | Note |
|---|---|---|---|
| config::loader::tests::repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings | red (assert: got Parse NoSuchStructField `repeat`) | ok | |
| config::loader::tests::repeat_false_loads_on_every_kind_and_trigger | red (Unexpected field `repeat`) | ok | |
| config::loader::tests::an_unknown_binding_field_lists_repeat_among_the_valid_names | red (message lacks `repeat`) | ok | |
| config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default | red (config loads: Parse found "repeat") | ok | |
| engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat | red (config does not load) | ok | acceptance 3 |

6a8e20c (parent 96528bb): `running 385 tests`, `test result: FAILED. 371 passed; 13 failed; 1 ignored`. Exactly the 13 named in impl.md:

| Test | Parent | HEAD | Note |
|---|---|---|---|
| engine::properties::auto_repeats_fire_only_actions_that_repeat | red (`repeat CapsLock at +699 ms: fired [SwitchLanguage] on an auto-repeat`) | ok | |
| engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once | red (`expected to fire [], fired [SwitchLanguage]`) | ok | acceptance 1 |
| engine::scenarios::capslock_held_across_a_focus_change_switches_the_language_once | red (same) | ok | |
| engine::scenarios::capslock_held_through_remapping_off_and_on_switches_the_language_once | red (+533 ms) | ok | |
| engine::scenarios::repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up | red (+533 ms) | ok | |
| engine::scenarios::held_shift_capslock_toggles_capslock_once | red (fired [ToggleCapsLock]) | ok | |
| engine::scenarios::held_launch_and_sequence_hotkeys_fire_once_per_press | red (fired Launch wt.exe) | ok | |
| engine::scenarios::non_repeating_winner_keeps_the_immediate_slot_on_a_repeat | red | ok | |
| engine::scenarios::only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default | red | ok | |
| engine::scenarios::volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not | red (fired [Media(PlayPause)]) | ok | acceptance 4 |
| engine::scenarios::repeat_false_on_press_key_fires_once_per_press | red (fired PressKey(37)) | ok | |
| engine::scenarios::press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding | red (down F13 at +1000 ms fired) | ok | |
| engine::scenarios::reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat | red | ok | |

Green-on-arrival by design (not in either red list, passed on 6a8e20c): press_key_backspace_held_with_ten_repeats_fires_eleven_times, deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat, repeat_check_reports_an_action_fired_on_an_auto_repeat, plus the commit-2 API tests. Their discriminating power is shown under mutation below. Worktrees removed; `git worktree list` shows only the lane tree.

## Mutations (scratch worktree wt/mut, own target dir, each reverted: `git diff --exit-code` clean)
| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| M1 | src/engine/mod.rs Immediate arm | `!was_repeat \|\| binding.repeat` -> `!was_repeat \|\| true` | default_capslock_held_with_ten_repeats_switches_the_language_once | caught (13 failed, that one among them) |
| M2 | same | -> `!was_repeat` (repeat flag ignored, never repeats) | press_key_backspace_held_with_ten_repeats_fires_eleven_times | caught (9 failed incl. it and repeat_true_on_switch_language_fires_on_every_repeat) |
| M3 | same | `immediate_fired = true` moved inside the fire condition (slot not kept on non-firing repeat) | non_repeating_winner_keeps_the_immediate_slot_on_a_repeat | caught (the only failure) |
| M4 | src/engine/mod.rs KS-06 block | blocked repeat: `suppress = true` -> `suppress = !decision.fire.is_empty()` (non-firing repeat leaks) | default_capslock_held_with_ten_repeats_switches_the_language_once (every event suppressed) | caught (25 failed incl. it, every_press_is_suppressed_or_passed_as_a_whole) |
| M5 | src/core/windows_actions.rs | `MediaKey::VolumeMute` default true | core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default | caught (3 failed: that, loader repeat_defaults_follow_the_action_and_an_explicit_value_wins, only_press_key_..._fire_again_on_auto_repeat_by_default) |
| M6 | src/config/loader.rs | `repeat: true` check disabled (`&& index == usize::MAX`) | repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings | caught (the only failure) |
| M7 | src/engine/mod.rs Tap arm | `was_repeat` guard deleted | deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat | caught (4 failed incl. it) |
| M7b | same, DoubleTap arm (lines 448-450) | guard deleted | same test | caught (5 failed incl. it and the property) |
| M8 | same, Sequence arm (464-466) | guard deleted | deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat | NOT caught by it on ad96e73: only `auto_repeats_fire_only_actions_that_repeat` failed. Closed by tester test, see below. On 10bcf94: caught by a_repeat_of_a_sequence_step_does_not_count_as_the_next_step |
| M9 | same, Chord arm (489-491) | guard deleted | same test | NOT caught by it on ad96e73 (only the property). On 10bcf94: caught by repeats_of_both_chord_keys_do_not_complete_the_chord_again |
| M10 | same, Hold arm (434-436) | guard deleted | same test | NOT caught by it (arm_hold is idempotent for a plain hold); caught by pre-existing engine::scenarios::deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed. Left as a note. |

M4-variant note: restricting the Immediate arm's `block_default` suppression with `(!was_repeat \|\| binding.repeat)` is an equivalent mutant (the KS-06 block re-suppresses the repeat); only the harness self-test press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down failed. Not a gap: the invariant is carried by the KS-06 block, which M4 proves is tested.

The property test (seed 0, deterministic) caught M7b, M8, M9, so no mutation was fully uncaught, but the named scenario's claim to pin sequences and chords was weak; hence the tester commit.

## Acceptance
| Criterion | Evidence | Status |
|---|---|---|
| CapsLock down, 10 repeats, up: exactly one SwitchLanguage, every event suppressed | engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once: `running 1 test ... ok` (shipped template via include_str!, check_press_symmetry, check_nothing_stuck, hold_with_repeats expects suppressed on down/repeats/up) | verified |
| PressKey("Backspace") held with 10 repeats fires 11 times | engine::scenarios::press_key_backspace_held_with_ten_repeats_fires_eleven_times ok (expect_total_fired 11); fails under M2 | verified |
| repeat: true on SwitchLanguage restores firing on every repeat | engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat ok (total 11); red on 5c43211 | verified |
| Media(VolumeUp) repeats by default, Media(PlayPause) does not | engine::scenarios::volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not ok (totals 4/4/1); core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default ok; caught M5 | verified |
| Repeat of non-repeating binding stays suppressed (A3) | hold_with_repeats expectations; M4; engine::properties::every_press_is_suppressed_or_passed_as_a_whole ok | verified |
| Deferred, sequence, chord ignore repeats, pinned | deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat (Tap/DoubleTap guards), tester tests for Sequence and Chord, property auto_repeats_fire_only_actions_that_repeat, loader repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings | verified |
| New RawBinding field, deny_unknown_fields stays | an_unknown_binding_field_lists_repeat_among_the_valid_names, unknown_fields_are_errors_that_name_the_field (repaet row), repeat_false_loads_on_every_kind_and_trigger: all ok in the full run | verified |
| README, default_config.ron, book config page, CHANGELOG | default_config_parses, default_config_commented_examples_load, readme_matching_examples_load ok (full run); text content not tested by me, mdBook build and cfgcheck are the developer's (impl.md:60-61) | partly verified (docs reviewed by code-reviewer) |
| Real hook: hold CapsLock gives one language switch, Backspace repeats in an editor, VolumeUp vs PlayPause, Win+Alt+T held opens one window, `[repeat]` marker in startup list, reload error on `Tap(200), repeat: true` keeps old bindings | impl.md manual table (7 rows); no `#[ignore]` test added | manual, not verified |

## Failures
None.

## Tests added by the tester (`Tests:` commit 10bcf94, test code only, src/engine/scenarios.rs, trailer Co-Authored-By: Claude Sonnet 5.5)
- engine::scenarios::a_repeat_of_a_sequence_step_does_not_count_as_the_next_step (sequence ["F13","F13"]; a repeat must not complete it; red under M8, `running 2 tests ... ok` on HEAD)
- engine::scenarios::repeats_of_both_chord_keys_do_not_complete_the_chord_again (chord F13+F14, window 50 ms; repeats of both keys after completion fire nothing; red under M9)
Both use the fake clock only. Red under mutation, green on HEAD (386 passed in the full run).

## Notes
- Open design points for the owner stay as in the pack (Q1: lost key-up after a swallowed key-down reads as a repeat; post-merge book sync). Not test findings.
- No flaky tests observed: three full runs (ad96e73 twice for lib, 10bcf94) gave identical counts. One transient `link.exe` error 1104 in the scratch worktree on a rebuild (file lock, retried, unrelated to the code).
- The harness `check_repeat_fire` inspects only on_key fires (pack fact 15); timer fires on repeat steps are not covered by it. Not a defect for this item.
- Scratch target dirs remain under D:/tmp/ks-orch/p1-1.1-repeat/target-wt (outside the repo); worktrees all removed.
