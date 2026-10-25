# Tester r2: p0-A2-matching / exact-modifier + trigger-key lane (c8ca4a5 vs previous c609a69 + 965ac4d)

Verdict: GREEN: I1 red-first tests are red by assertion on c59a110 (= 965ac4d + tests) and green on HEAD; 8 own mutations, all caught; gate clean.

HEAD `c8ca4a59679d351299607fc6dda1a8a53de30511` on `fix/matching`; `git status --short` empty before and after; `git worktree list` shows only the lane tree. No tests added, no commit made by the tester (no gap found).

## Gate (HEAD)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (sources touched first; output `Checking key-switch-rs ... Finished`, no warnings)
- cargo test --locked --no-fail-fast: lib `running 263 tests` / `test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`; two bin targets `running 0 tests` (no tests in the bins; not a filter). Totals: 263 passed, 0 failed, 0 ignored.
- cargo build --release --locked: OK (`Finished release profile`); not run.

## Red-first replay (CONFIRMED item I1)
Parent = c59a110 (tests only on top of 965ac4d, so identical production code to c609a69 + 965ac4d), scratch worktree `t2/wt/parent`, `CARGO_TARGET_DIR=t2/target-wt/parent`, `--lib engine::scenarios::<name> -- --exact` (each `running 1 test`).

| Test | Parent | HEAD | Note |
|---|---|---|---|
| immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed | red (assert) `repeat F13 at +500 ms: expected the event to be suppressed, it was passed` | ok (1 passed, 262 filtered out) | triage p1 |
| capslock_held_across_a_focus_change_does_not_reach_windows | red (assert) `repeat CapsLock at +500 ms: ... it was passed` | ok | triage p2, default config |
| win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed | red (assert) `repeat T at +500 ms: ... it was passed` | ok | triage p5, roadmap 2.3 sentence |
| press_after_a_lost_key_up_and_a_focus_change_follows_its_binding | red (assert) `down F13 at +1000 ms: ... it was passed` | ok | reviewer scenario 2 |
| repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change (guard) | ok (green, as designed) | ok | pins Passed survives focus change; carried by M1, M2, M8 |
| keys::tests::clear_all_keeps_passed_key_downs, ::latest_key_down_replaces_the_record_and_take_forgets_it | compile-red by construction (new `mark_passed`/`is_passed`, not in c59a110) | ok (`engine::keys::tests`: running 10 tests, 10 passed) | carried by M7, M8 |

Worktree removed, pruned.

## Mutations (scratch worktree `t2/wt/mut` at HEAD, own target dir; each reverted, `git diff --exit-code` clean; worktree removed)
Not repeats of the developer's M-a/M-b (M8 is a variant of M-a, M4 a variant of M-b with a different shape).

| # | File | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | mod.rs repeat tail | `else if self.keys.is_passed(vk.0)` -> `else if false` | guard `repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change` | caught: that + `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (2 failed) |
| M2 | mod.rs first key-down tail | `mark_passed(vk.0)` removed | guard (record becomes Unknown, binding decides) | caught: guard, `repeat_of_a_key_whose_down_passed_...key_up`, `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (3 failed) |
| M3 | keys.rs `clear_all_blocked` | no-op (`if false`) | `foreground_change_forgets_pending_key_up_suppression` | caught: that + `clear_all_resets_every_flag`, `clear_all_keeps_passed_key_downs`, three focus-change scenarios (6 failed) |
| M4 | mod.rs repeat tail | `else if is_passed` -> `else` (Unknown repeat treated as passed) | the four red-first scenarios | caught: `capslock_held_...`, `immediate_key_held_...`, `win_alt_t_held_...`, `press_after_a_lost_key_up_...` (4 failed) |
| M5 | mod.rs combo_matches call | deferred Tap ignores exact modifiers (`allow_extra_modifiers \|\| Tap`) | r1 tester test `deferred_binding_does_not_arm_while_shift_is_held` | caught: exactly that test (1 failed) |
| M6 | same | deferred Tap ignores `allow_extra_modifiers: true` | r1 tester test `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers` | caught: exactly that test (1 failed) |
| M7 | keys.rs `take_blocked` | a Passed record survives a key-up | unit `latest_key_down_replaces_the_record_and_take_forgets_it` | caught: that test (1 failed); no scenario catches it (next press re-marks), unit test is the sole guard |
| M8 | keys.rs `clear_all_blocked` | also forgets `Passed` | `clear_all_keeps_passed_key_downs` + guard | caught: both (2 failed) |

First attempts at M5-M7 did not apply (multi-line match, then a borrow error in M7); the reported results are from the reapplied, compiled mutations. Unapplied runs were unmutated and green (263 passed).

## Acceptance (brief list; unchanged since r1, delta re-checked on HEAD by the full run)
| Criterion | Evidence (HEAD, all within the 263 ok) | Status |
|---|---|---|
| CapsLock held, `a` passes, nothing fires | `key_pressed_while_an_immediate_binding_key_is_held_passes`, `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` | verified |
| CapsLock binding not fired with Shift held; fires with allow_extra_modifiers | `capslock_binding_does_not_fire_while_shift_is_held`, `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held`, deferred pair (M5/M6 prove they can fail) | verified |
| Ctrl+J not on Ctrl+Shift+J | `ctrl_j_does_not_fire_on_ctrl_shift_j` | verified |
| LShift/RWin side-specific, Shift both | `lshift_in_keys_fires_only_for_the_left_shift`, `rwin_and_lwin_in_keys_fire_only_for_their_side`, `unsided_shift_and_win_in_keys_fire_for_either_side` | verified |
| Shift in a chord fires | `unsided_shift_in_a_chord_fires_for_either_side`, `sided_shift_in_a_chord_is_only_its_side` | verified |
| Shift+CapsLock fires ToggleCapsLock | `shift_capslock_fires_toggle_capslock_with_either_shift` | verified |
| KS-06 repeat rule incl. focus change (lane addition, I1) | red-first table above | verified |

## Failures
None.

## Tests added by the tester
None.

## Notes
- Not verified by the tester: I1(c) (KS-29 lost key-up + combo, DEFERRED to roadmap 2.3), M1 pending Tap/Hold modifier key-up (KS-14, docs narrowed only), deferred-binding repeats across a focus change (pack fact 20, no scenario pins it). These are accepted open points in fix_r1.md/triage_r1.md, not in this round's CONFIRMED set.
- Behaviour change vs master (by design, documented): after a focus change the key-up of a key whose repeats an Immediate binding swallowed passes. Probes p1/p5 asserting master's key-up fail only at that step.
- Scratch: `D:/tmp/ks-orch/p0-A2-matching/t2/` (target dirs, mut.py). Nothing in the lane tree.
