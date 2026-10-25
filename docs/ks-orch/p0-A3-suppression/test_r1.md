# Tester: p0-A3-suppression / KS-06 rest, KS-14 (HEAD 15e901d + tester commit 5d0fada vs master 82f542f)

Verdict: GREEN. Gate green, all 15 red-first tests red by assertion on the parent and green on HEAD, 12 mutations all caught (one needed a new scenario, committed as 5d0fada).

Tree at start: `git status --short` empty, HEAD `15e901d`. At the end: HEAD `5d0fada`, tree clean, `git worktree list` shows only the lane tree.

## Gate (HEAD 15e901d, then re-run on 5d0fada)
- `cargo fmt --check`: OK
- `cargo clippy --all-targets --locked -- -D warnings`: OK. The first run printed "Finished ... 0.15s" (stale); I touched the sources and re-ran: "Checking key-switch-rs ... Finished ... 1.17s", no warnings. Re-run after 5d0fada also OK.
- `cargo test --locked --no-fail-fast` (15e901d): `running 285 tests` / `test result: ok. 285 passed; 0 failed; 0 ignored`; swch bin `running 0 tests` (no tests there); doc-tests `running 0 tests`. The lib is the only target with tests.
- `cargo test --locked --no-fail-fast` (5d0fada): `running 286 tests` / `ok. 286 passed; 0 failed; 0 ignored`.
- `cargo build --release --locked`: OK (Finished release).
- No `#[ignore]` tests in this lane, 0 ignored.

## Red-first replay
Method: scratch worktrees under `D:/tmp/ks-orch/p0-A3-suppression/wt`, one `CARGO_TARGET_DIR` per commit under `target-wt/`. A first run that shared one target dir between worktrees reused a stale test exe (273 tests at 72ea8d6 and afdb60c, identical to 20d4844). I discarded that run and redid 72ea8d6 and afdb60c with their own target dirs (278 and 282 tests, as expected). Lesson: never share the target dir between worktrees of this package.

Red commits touch test code only, engine at the red commit equals the previous fix. Full lib runs:
- 20d4844 (engine = master): `running 273 tests`, 266 passed, 7 failed.
- 72ea8d6: `running 278 tests`, 274 passed, 4 failed.
- afdb60c: `running 282 tests`, 278 passed, 4 failed.

| Test | Parent | HEAD | Red text at parent |
|---|---|---|---|
| scenarios::alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier | red (assert) @20d4844 | ok | `down X at +0 ms: expected the event to be suppressed, it was passed` |
| scenarios::shift_held_by_a_double_tap_still_counts_for_shift_capslock | red (assert) | ok | `down CapsLock at +50 ms: expected to fire [ToggleCapsLock], fired [SwitchLanguage]` |
| scenarios::one_long_press_of_shift_does_not_fire_its_double_tap | red (assert) | ok | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| scenarios::ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift | red (assert) | ok | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` |
| scenarios::shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press | red (assert) | ok | same text |
| scenarios::suppressed_modifier_stays_an_auto_repeat_across_a_focus_change | red (assert) | ok | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed | red (assert) | ok | panic at keys.rs:364 |
| scenarios::chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed | red (assert) @72ea8d6 | ok | `repeat J at +500 ms: expected the event to be suppressed, it was passed` |
| scenarios::deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed | red (assert) | ok | `repeat CapsLock at +500 ms: ... suppressed, it was passed` |
| scenarios::immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed | red (assert) | ok | `repeat F13 at +500 ms: ... suppressed, it was passed` |
| keys::tests::forget_blocked_keeps_passed_key_downs | red (assert) | ok | `a forgotten key-down was still swallowed, so its repeats are too` |
| scenarios::sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed | red (assert) @afdb60c | ok | `up F14 at +0 ms: VK 0x7D is down in Windows but physically up` |
| scenarios::chord_completion_keeps_the_key_up_of_a_chord_key_whose_down_passed | red (assert) | ok | `up J at +0 ms: VK 0x4A is down in Windows but physically up` |
| scenarios::modifier_held_before_the_engine_saw_it_keeps_passing_once_a_repeat_passed | red (assert) | ok | `repeat LShift at +533 ms: expected the event to be passed, it was suppressed` |
| tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending | red (assert) | ok | `a key-up whose key-down reached Windows must reach it too` (mod.rs:584) |

No compile-red test.

Whole-lane replay: worktree at master `82f542f` with HEAD's `testing.rs`, `scenarios.rs`, `properties.rs` copied in and `mod properties;` added: `running 280 tests`, 266 passed, 14 failed. The 14 are the 12 scenario red-firsts of the table (all but the 3 unit tests) plus both property tests:
- `properties::every_press_is_suppressed_or_passed_as_a_whole`: `repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed` / `context: config default, seed 0, Strict mode`
- `properties::nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups`: `up LCtrl at +4114 ms: VK 0xA2 is down in Windows but physically up` / `context: config gestures, seed 3, Focus mode`

(The HEAD versions of the scenarios, reworked in 15e901d, are therefore red against the master engine.)

On HEAD each of the 15 plus the 2 property tests was run with `-- --exact`: every run printed `running 1 test` / `ok. 1 passed`.

Worktrees (20d4844, 72ea8d6, afdb60c, master, mut) removed; `git worktree list` shows only the lane tree.

## Mutations (scratch worktree at 15e901d, one at a time, reverted by `git checkout -- .`, `git diff --exit-code` clean after each)
| # | File | Mutation | Predicted catcher | Result (failing tests) |
|---|---|---|---|---|
| M1 | keys.rs sync_modifiers | drop `|| self.key_down_swallowed(index)` (D2) | keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed | caught: 9 failed incl. that test, alt_shift_x..., shift_held_after_the_auto_blocker..., properties::every_press_... |
| M2 | keys.rs is_blocked | count only `Suppressed`, not `Forgotten` (I1 b) | keys::tests::forget_blocked_keeps_passed_key_downs | caught: 4 failed (that test + 3 focus-change scenarios) |
| M3 | mod.rs on_key_up | `was_blocked \|\| gesture_bound` (drop `!was_passed`, D4) | engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending | caught: that test only (1 failed) |
| M4 | keys.rs mark_completed | drop the `!= Passed` guard (D5) | sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed | caught: 5 failed incl. both completion scenarios, keys::tests::completion_marks_only_held_keys..., both property tests |
| M5 | mod.rs repeat tail | remove the `is_passed` branch (D3) | scenarios::repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up | caught: 4 failed incl. that test, properties::every_press_... |
| M6 | keys.rs take_blocked | return `!= Passed` instead of `== Suppressed` | keys::tests::forget_blocked_resets_every_flag | caught: 13 failed incl. that test |
| M7 | mod.rs repeat tail | `is_blocked` branch replaced by `false` (a repeat of a suppressed key no longer suppressed) | scenarios::repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired | caught: 12 failed incl. that test, T4/T5, immediate_combo_..._mid_hold |
| M8 | mod.rs first key-down | `mark_blocked` removed | repeat/key-up scenarios | caught: 41 failed incl. hook::keyboard_hook_callback tests |
| M9 | mod.rs on_foreground_changed | `forget_blocked()` removed | scenarios::foreground_change_forgets_pending_key_up_suppression | caught: 8 failed incl. that test |
| M10 | mod.rs on_key_up | `was_passed = false` | the D4 unit test | caught (1 failed, same test as M3) |
| M11 | keys.rs mark_completed | drop `&& self.active[index]` (KS-30) | keys::tests::completion_marks_only_held_keys_whose_key_down_did_not_pass | caught by that unit test only (1 failed); no event-level scenario fails |
| M12 | keys.rs take_blocked | key-up leaves the record `Suppressed` instead of `Unknown` ("suppressed exactly once") | a scenario | NOT caught by any scenario or property; only keys::tests::blocked_flags_are_independent and is_blocked_reads_without_clearing failed. Gap closed below. |

M12 closure: new scenario `engine::scenarios::key_up_of_a_suppressed_press_is_suppressed_only_once` (commit 5d0fada). On HEAD: `running 1 test` / `ok`. Under M12: `FAILED`, `scenario: up LShift at +1000 ms: VK 0xA0 is down in Windows but physically up`. Reverted, clean.

Note on M11: the KS-30 guard is pinned only by a unit test; it is not an acceptance item of this lane, so left as is.

## Acceptance (roadmap Workstream A / brief)
| Criterion | Evidence (HEAD, `--exact`, each `running 1 test` / `ok`) | Status |
|---|---|---|
| A repeat of a suppressed key is suppressed | scenarios::shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press (two repeats), scenarios::repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired, scenarios::immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once, scenarios::ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift; M7 caught by these | verified |
| ...and its key-up is suppressed exactly once | same T5 test (key-up suppressed, next press passes both edges); new scenarios::key_up_of_a_suppressed_press_is_suppressed_only_once (a later key-up passes); M12 caught by it | verified |
| KS-06 default-config route (Alt+Shift blocker, release Alt while holding Shift) leaves nothing stuck | T5 above, with `check_nothing_stuck()` on `with_alt_shift_blocker(DEFAULT_PAIR)`; M1 breaks it | verified |
| Property-style test over generated down/repeat/up sequences | properties::every_press_is_suppressed_or_passed_as_a_whole (Strict, 6 configs x 64 seeds x 250 steps) and properties::nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups (Focus); both ok on HEAD (0.67 s), both red on the master engine (messages above), M1, M4, M5 flip them | verified (uses a fake clock, no real time, so no flake reruns needed) |
| Spec: a passing key-down clears blocked state, key-up passes | scenarios::repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up, sequence_completion_keeps..., chord_completion_keeps..., modifier_held_before_the_engine_saw_it..., engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending; M3, M4, M5, M10 | verified |
| Spec: the resync does not overwrite a suppressed key nor reset repeat detection | keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed, keys::tests::resync_still_repairs_a_modifier_whose_key_down_passed, scenarios::one_long_press_of_shift_does_not_fire_its_double_tap, scenarios::modifier_lost_by_the_os_is_resynced_before_matching; M1 | verified |

Manual only (not verified by me, no `#[ignore]` stubs exist; impl.md M1-M9 smoke list): the real hook / `GetAsyncKeyState` behaviour, M6 layout switch on release, M7 chord J+K -> Notepad, M8 CapsLock Hold(300) -> `wt.exe`, M9 (`swch off`, hold LShift, `swch on`, type `a`, press/release LAlt). All need the live daemon, so I did not run them.

## Failures
None.

## Tests added by the tester
- Committed `5d0fada` "Tests: a suppressed press spends its record, a later key-up of an unseen press passes (KS-06)" on `fix/suppression-symmetry`, test code only (`src/engine/scenarios.rs` +23): `engine::scenarios::key_up_of_a_suppressed_press_is_suppressed_only_once`. Co-Authored-By trailer present.

## Notes for the reviewer
- Safety grep over `src/engine`: no pipe name, no registry write, no `SendInput`, no hook install in tests.
- Engine diff is safe-code only (`#![forbid(unsafe_code)]` remains); scratch `mut.sh`/`mut2.sh` helpers and `target-wt` were deleted, `dev-target/` (developer's) untouched.
- Property generator repeats any held key, scenarios since 15e901d repeat only the last-pressed key (impl.md Deviation 8); M1/M4/M5 show the generator still bites.
- Open points from the pack (RK7 stale non-modifier, `mark_completed` on a resync-introduced `Unknown` modifier, I1(b) first press after a lost key-up swallowed once) are left to roadmap 2.3; I did not test them.
- On Windows git warned about LF/CRLF conversion in the worktree; multi-line string mutations did not apply for that reason, so all mutations are single-line.
