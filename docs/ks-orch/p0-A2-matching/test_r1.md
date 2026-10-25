# Tester: p0-A2-matching / A2 matching (HEAD c609a69 + tester commit 965ac4d, vs master e67e5cb)

Verdict: GREEN. Gate green, all 22 scenario red-firsts plus the 2 loader red-firsts are red by assertion on their pre-fix commits and green on HEAD, 14 own mutations all caught, every acceptance line mapped to a run test. One test-only commit added (965ac4d). One deviation from the brief's wording is reported for the orchestrator (see Notes).

Tree: `fix/matching`, `git status --short` empty at read time (HEAD c609a69); after my commit HEAD = 965ac4d, tree clean, `git worktree list` shows only the lane tree.

## Gate (HEAD c609a69; re-run on 965ac4d, same result)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (sources touched first; "Checking key-switch-rs ... Finished")
- cargo test --locked --no-fail-fast, on c609a69:
  - `running 254 tests` / `test result: ok. 254 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (lib)
  - `running 0 tests` swch bin, `running 0 tests` doc-tests (both legitimately empty; the daemon bin has `test = false`)
  - on 965ac4d: `running 256 tests` / `test result: ok. 256 passed; 0 failed; 0 ignored` (+2 new scenarios)
- cargo build --release --locked: OK ("Finished `release` profile"), not run.
- Ignored tests: 0 (`grep -rn "#\[ignore" src` = 0). No real-clock decision tests (all scenarios use the fake clock), so no flake reruns needed.
- Safety greps on `git diff master..HEAD -- src`: no `key-switch-rs.sock`, `SetWindowsHookEx`, `RegSetValue`, `ShellExecute`; the only `SendInput` hit is a doc comment in `vk_name.rs`. Hook/registry diffs are test-only or combo construction (read-only).

## Red-first replay
Method: each red commit checked out in a scratch worktree (`D:/tmp/ks-orch/p0-A2-matching/wt/red`, own `CARGO_TARGET_DIR=.../target-wt/red`), full `cargo test --locked --lib`, failures listed and panic text compared with impl.md. Outputs: `D:/tmp/ks-orch/p0-A2-matching/out/red_<sha>.txt`. Then HEAD run with `--exact` of all 29 names: `running 29 tests ... test result: ok. 29 passed; 0 failed`.

Additionally every scenario red-first was run against master `e67e5cb` (scratch worktree, own target dir) with HEAD's `src/engine/scenarios.rs` copied in (it compiles on master since it only uses RON and the harness): `running 219 tests`, `test result: FAILED. 198 passed; 21 failed` = the 20 scenario red-firsts + `either_shift_counts_as_the_same_modifier` (guard that needs sided LShift; red on master by `down F13 ... expected the event to be suppressed, it was passed`). Output: `out/master.txt`.

| Test | Pre-fix commit | Red text (assertion) | HEAD |
|---|---|---|---|
| lshift_in_keys_fires_only_for_the_left_shift | 1bc3b0e | `down F13 at +0 ms: expected the event to be suppressed, it was passed` | ok |
| rwin_and_lwin_in_keys_fire_only_for_their_side | 1bc3b0e | same text | ok |
| lshift_double_tap_fires_only_for_the_left_shift | 1bc3b0e | `down LShift at +0 ms: ... suppressed, it was passed` | ok |
| unsided_shift_in_a_chord_fires_for_either_side | 1bc3b0e | `down LShift at +0 ms: ... suppressed, it was passed` | ok |
| sided_shift_in_a_sequence_step_matches_only_its_side | 1bc3b0e | `down F14 at +0 ms: ... suppressed, it was passed` | ok |
| win_in_the_last_sequence_step_does_not_swallow_its_key_up | 1bc3b0e | `up Win at +0 ms: expected the event to be passed, it was suppressed` | ok |
| config::loader::tests::chord_keys_that_overlap_are_rejected | 1bc3b0e | panic `overlapping chord keys must be rejected: [Binding { kind: Chord ...` | ok |
| key_pressed_while_an_immediate_binding_key_is_held_passes | 46424a1 | `down A at +30 ms: expected ... passed, it was suppressed` | ok |
| key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing | 46424a1 | `down A at +50 ms: ... passed, it was suppressed` | ok |
| modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock | 46424a1 | `down LShift at +0 ms: ... passed, it was suppressed` | ok |
| modifier_pressed_last_does_not_fire_a_combo | 46424a1 | `down LCtrl at +0 ms: ... passed, it was suppressed` | ok |
| deferred_combo_arms_only_on_its_trigger_key | 46424a1 | `down LShift at +350 ms: ... passed, it was suppressed` | ok |
| capslock_binding_does_not_fire_while_shift_is_held | 7b65609 | `down CapsLock at +0 ms: ... passed, it was suppressed` | ok |
| ctrl_j_does_not_fire_on_ctrl_shift_j | 7b65609 | `down J at +0 ms: ... passed, it was suppressed` | ok |
| auto_blocker_matches_its_modifiers_exactly | 7b65609 | `down LShift at +0 ms: ... passed, it was suppressed` | ok |
| config::loader::tests::allow_extra_modifiers_is_only_for_keys_bindings | 7b65609 | panic `allow_extra_modifiers on a sequence/chord must be rejected: [Binding { kind: Sequence ...` | ok |
| repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up | f4b2582 | `repeat CapsLock at +533 ms: ... passed, it was suppressed` | ok |
| double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too | f4b2582 | `repeat LShift at +500 ms: ... suppressed, it was passed` | ok |
| chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too | f4b2582 | `repeat LShift at +500 ms: ... suppressed, it was passed` | ok |
| repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired | f4b2582 | `repeat CapsLock at +500 ms: ... suppressed, it was passed` | ok |
| immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once | f4b2582 | `repeat L at +500 ms: ... suppressed, it was passed` | ok |
| shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck | faf3be1 | `up LShift at +533 ms: ... passed, it was suppressed` | ok |

Per-commit failure counts: 1bc3b0e 195 passed / 7 failed (202 tests); 46424a1 221/5 (226); 7b65609 233/4 (237); f4b2582 247/5 (252); faf3be1 247/6 (253). The failing sets are exactly the tests above, nothing else; guards (T3, T6, T16, T18, T19, T22) stayed green on the red commits. No compile-red. The two loader tests exist only in `loader.rs` (not copyable onto master), so their master-equivalent red is the red-commit run above.

Worktrees removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 965ac4d [fix/matching]`.

## Mutations (scratch worktree `wt/mut` at c609a69, own target dir; each reverted, `git diff --exit-code` clean, worktree removed)
Prediction named before running; actual failing set in the last column (full output `out/mut_all.txt`, `out/mut2_all.txt`).

| # | File | Mutation | Predicted catcher | Result (tests that failed) |
|---|---|---|---|---|
| M1 | engine/matching.rs `combo_matches` | exact clause `allow_extra_modifiers \|\| ...` -> `true \|\| ...` | ctrl_j_does_not_fire_on_ctrl_shift_j | caught: ctrl_j_..., capslock_binding_does_not_fire_while_shift_is_held, auto_blocker_matches_its_modifiers_exactly, repeat_of_a_key_whose_down_passed_..., 3 matching unit tests |
| M2 | engine/matching.rs `is_trigger_key` | drop `key.modifier().is_none()` (modifier counts as trigger) | modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock | caught: that, modifier_pressed_last_does_not_fire_a_combo, deferred_combo_arms_only_on_its_trigger_key, 2 matching unit tests |
| M3 | engine/mod.rs:434 | repeat no longer takes `is_blocked` | repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired | caught: that + double_tap_lshift_held_..., chord_keys_held_..., immediate_combo_that_stops_matching_..., repeat_of_a_key_whose_down_passed_... |
| M4 | data/key_spec.rs `accepts` and `Modifier::of` | `AnySide` no longer accepts the right key | rwin_and_lwin_in_keys_fire_only_for_their_side | caught: 11 failures incl. rwin_and_lwin_..., unsided_shift_in_a_chord_fires_for_either_side, chord_keys_that_overlap_are_rejected |
| M5 | config/loader.rs:244 | `with_allow_extra_modifiers(false)` (flag ignored) | allow_extra_modifiers_lets_capslock_fire_while_shift_is_held | caught: that, allow_extra_modifiers_lets_ctrl_j_..., allow_extra_modifiers_true_is_parsed |
| M6 | engine/mod.rs:442 | passing key-down no longer clears its key-up block | shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck | caught: exactly that test (the only one) |
| M7 | config/loader.rs chord check | overlap check removed | chord_keys_that_overlap_are_rejected | caught: that + chord_duplicate_key_is_error |
| M8 | engine/matching.rs `keys_down` | `all` -> `any` | keys_down_needs_every_key_on_its_side | caught: that + 6 (lshift_in_keys_..., rwin_and_lwin_..., sequence_with_a_modifier_held_through_both_steps, sided_shift_in_a_sequence_step_..., sequences::sided_step_needs_its_side, matching unit) |
| M9 | data/binding.rs:102 | auto-blocker built with `allow_extra_modifiers: true` | auto_blocker_matches_its_modifiers_exactly | caught: exactly that test |
| M10 | engine/keys.rs `impl HeldKeys` | `AnySide` ignores the right key | unsided_shift_and_win_in_keys_fire_for_either_side | caught: that, rwin_and_lwin_..., 4 keys/matching unit tests |
| M11 | engine/matching.rs `is_trigger_key` | `key.accepts(pressed)` -> `true` (any pressed key triggers) | key_pressed_while_an_immediate_binding_key_is_held_passes | caught: that, key_pressed_while_a_tap_hold_key_..., modifier_pressed_*, deferred_combo_arms_only_on_its_trigger_key, 3 matching unit tests |
| M12 | config/loader.rs:229 | non-Combo `allow_extra_modifiers` no longer rejected | allow_extra_modifiers_is_only_for_keys_bindings | caught: exactly that test |
| M14 | data/key_combination.rs `modifiers()` | always EMPTY | ctrl_j_does_not_fire_on_ctrl_shift_j | caught: 19 failures incl. ctrl_j_..., shift_capslock_fires_toggle_capslock_with_either_shift, auto_blocker_... |
| M16 | engine/mod.rs:434 | repeat rule `=` -> `\|=` (a binding can still request suppress on a repeat) | repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up | caught: exactly that test (I2) |

New-test mutations (tests added in 965ac4d, copied into the scratch worktree): with M1 `deferred_binding_does_not_arm_while_shift_is_held` FAILED (`running 2 tests`, 1 failed); with M5 `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers` FAILED; unmutated both ok.

## Acceptance (brief)
All named tests are in the 29-name `--exact` HEAD run (`running 29 tests ... 29 passed`) and in the full 254/256 run.

| Criterion | Evidence | Status |
|---|---|---|
| With CapsLock held, `a` passes and nothing fires | `key_pressed_while_an_immediate_binding_key_is_held_passes`, `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` (red on 46424a1, M11 catches) | verified |
| CapsLock binding does NOT fire while Shift is held | `capslock_binding_does_not_fire_while_shift_is_held` (red on 7b65609, M1/M14 catch); deferred variant `deferred_binding_does_not_arm_while_shift_is_held` (tester) | verified |
| ... and does with `allow_extra_modifiers: true` | `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held`, `config::loader::tests::allow_extra_modifiers_true_is_parsed`, `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers` (tester); M5 catches | verified |
| Ctrl+J does not fire on Ctrl+Shift+J | `ctrl_j_does_not_fire_on_ctrl_shift_j` (+ guard `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j`) | verified |
| LShift and RWin in keys fire only for that side | `lshift_in_keys_fires_only_for_the_left_shift`, `rwin_and_lwin_in_keys_fire_only_for_their_side`, `lshift_double_tap_fires_only_for_the_left_shift` | verified |
| Shift fires for both sides | `unsided_shift_and_win_in_keys_fire_for_either_side`, `either_shift_counts_as_the_same_modifier` | verified |
| Shift in a chord fires | `unsided_shift_in_a_chord_fires_for_either_side` (both sides), `sided_shift_in_a_chord_is_only_its_side`, `engine::chords::tests::unsided_shift_chord_completes_with_either_shift` | verified |
| Shift+CapsLock fires ToggleCapsLock | `shift_capslock_fires_toggle_capslock_with_either_shift`, `more_specific_immediate_binding_wins` | verified |
| Trigger rule for deferred arms keyed on the key | `deferred_combo_arms_only_on_its_trigger_key`, `key_pressed_while_a_tap_hold_key_...` | verified |
| README combo syntax / sided modifiers, default_config.ron comments, CHANGELOG "Breaking:" | `config::loader::tests::readme_matching_examples_load` and `default_config_parses` ok; grep: CHANGELOG.md:55 and :66 `**Breaking:**` (sided names; exact modifiers), README.md:175-230 section + :420 flag row, default_config.ron:58, :126 | verified (prose not machine-checked beyond the RON examples) |
| Real Windows behaviour (live hook, Windows keeps the key-up, AltGr, Start menu, stuck keys) | impl.md manual checks M1-M10 (release build, `swch open`, `osk.exe`); no `#[ignore]` tests exist for them | manual, not verified |

Criteria only checkable manually: impl.md M1-M10 (CapsLock+typing within 0.4 s, Shift+CapsLock per Shift, Ctrl+J vs Ctrl+Shift+J in a browser, LShift DoubleTap/stuck-Shift, RWin+E, chord Shift+Space, AltGr+Alt+X, Hold/Tap CapsLock stuck key in osk.exe, Alt+Shift language hotkey). None can be expressed as a pure decision test beyond what scenarios already cover; no pure-function gap found.

## Failures
None.

## Tests added by the tester (committed on fix/matching)
- Commit `965ac4d` "Tests: deferred bindings follow the exact-modifier rule and allow_extra_modifiers" (touches only `src/engine/scenarios.rs`, +44 lines, Co-Authored-By trailer): `deferred_binding_does_not_arm_while_shift_is_held` and `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers`. Gap closed: exact-modifier rule and the flag were pinned for Immediate bindings only; both tests shown red under M1 / M5 and green on HEAD.
- Considered and dropped: a Shift+CapsLock scenario next to the Alt+Shift / Ctrl+Shift auto-blockers; no single mutation made it fail (a test that cannot fail), so it is not committed.

## Notes (for the code-reviewer and orchestrator)
- Deviation from the brief wording, already in cut.md Q1/D2 and critique: the brief says a combo fires only when the pressed key "belongs to its combo"; the implementation is stricter: with a non-modifier key in the combo, only that key triggers (`Shift` pressed while CapsLock is held does not fire `Shift+CapsLock`; `Ctrl` pressed after `J` does not fire `Ctrl+J`). Modifier-only combos trigger on any of their keys. Pinned by `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock` and `modifier_pressed_last_does_not_fire_a_combo`. Needs the owner's confirmation if they meant the literal reading.
- Known scope limits (documented by impl.md, not defects of this lane): repeats still fire Immediate actions (KS-10); a swallowed modifier erased by the resync counts as not held for exact matching (KS-14); sequence steps and chords do not use the exact-modifier rule (3.2); book sync and CLAUDE.md/R8 text updates are not done.
- Pre-existing rustdoc warning `src/data/vk_name.rs:4` untouched.
- My commit is on `fix/matching`; the lane tree is clean; scratch dirs live only under `D:/tmp/ks-orch/p0-A2-matching/` (`out/`, `target-wt/`, `mut.sh`, `mut2.sh`, `newtests.txt`).
