# Tester: p0-A4-autoblockers / KS-18 + KS-05 exact part (HEAD 3d65ac9, lane code f5595aa, master 44f79ac)

Verdict: GREEN. Gate green, 18 red-first tests red by assertion on the red commits and green on HEAD, 8 mutations (7 caught by named tests; the 8th, an off-by-one in `all_held`, was not caught until I added a test, shown red under the mutation and green without it). One `Tests:` commit added by the tester (3d65ac9).

Tree: `D:/key-switch-rs/key-switch-rs` on `fix/auto-blockers`. HEAD at start `f5595aa`, clean. HEAD now `3d65ac9` (my one Tests: commit on top), clean; `git worktree list` shows only the lane tree.

## Gate (HEAD 3d65ac9)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (re-run after `touch src/lib.rs src/main.rs src/bin/swch.rs`: "Checking key-switch-rs ... Finished"; not a stale run). Re-run after my commit: OK.
- cargo test --locked --no-fail-fast (HEAD 3d65ac9): lib `running 307 tests` / `test result: ok. 307 passed; 0 failed; 0 ignored`; the two bin targets `running 0 tests` (they have no tests; the lib carries them). Before my commit (f5595aa): `running 306 tests`, 306 passed, 0 failed, 0 ignored. Passed 307, failed 0, ignored 0. No doc-test target runs.
- cargo build --release --locked: OK (`Finished release`). Not run.

## Red-first replay
master (44f79ac) has neither the tests nor the seam (`from_ron_str_with_hotkeys`, `from_ron_with_language_hotkey`), so the new tests cannot compile there (compile-red proves nothing). Real replays use the declared red commits, whose production code is master's behaviour (cae1ec5 = master + behaviour-neutral seam 91937a0 + tests; 8a08253 = bb4dfcd's loader fix + tests). Each ran in its own worktree under `D:/tmp/ks-orch/p0-A4-autoblockers/wt/<sha>` with `CARGO_TARGET_DIR=.../target-wt/<sha>`; full `cargo test --locked --lib`.

cae1ec5: `running 292 tests`, `test result: FAILED. 288 passed; 4 failed`.
8a08253: `running 303 tests`, `test result: FAILED. 289 passed; 14 failed`.
HEAD: 307 passed, 0 failed. Counts equal impl.md:24 (4 and 14).

| Test | Red commit | Red by (assertion text) | HEAD |
|---|---|---|---|
| config::loader::tests::auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order | cae1ec5 | loader.rs:672 `left: 1 right: 0` | ok |
| config::loader::tests::auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys | cae1ec5 | loader.rs:684 `left: 1 right: 0` | ok |
| config::loader::tests::auto_blockers_follow_the_bindings_of_the_file | cae1ec5 | loader.rs:697 `the file's bindings come first` | ok |
| engine::scenarios::user_binding_on_the_language_hotkey_fires_in_either_file_order | cae1ec5 | `down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]` / `context: flagged binding above` | ok |
| shift_pressed_while_alt_and_tab_are_held_reaches_windows (T2) | 8a08253 | `down LShift at +500 ms: expected the event to be passed, it was suppressed` | ok |
| shift_pressed_while_ctrl_and_t_are_held_reaches_windows (T3) | 8a08253 | `down LShift at +0 ms: expected the event to be passed, it was suppressed` | ok |
| alt_shift_pressed_while_a_letter_is_held_reaches_windows (T4) | 8a08253 | same as T3 | ok |
| grave_blocker_acts_only_on_a_lone_grave (T5) | 8a08253 | ``down ` at +0 ms: expected the event to be passed, it was suppressed`` | ok |
| ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing (T6) | 8a08253 | `down LShift at +0 ms: expected to fire [], fired [DoNothing]` | ok |
| shift_pressed_after_tab_was_released_is_still_swallowed (M4 pin) | 8a08253 | same as T6 | ok |
| user_binding_on_alt_shift_fires_with_the_blocker_in_either_order (T7) | 8a08253 | `expected to fire [SwitchLanguageBackward], fired [DoNothing]` / `context: blocker first` | ok |
| sided_user_binding_fires_and_the_blocker_keeps_the_other_sides (T8) | 8a08253 | `down RShift at +0 ms: expected to fire [], fired [DoNothing]` | ok |
| key_whose_release_the_hook_missed_does_not_disable_the_blocker (T9) | 8a08253 | same as T6 | ok |
| key_swallowed_by_a_binding_does_not_disable_the_blocker (T10) | 8a08253 | same as T6 | ok |
| modifier_swallowed_by_a_binding_does_not_disable_the_blocker (M3 pin) | 8a08253 | `expected the event to be suppressed, it was passed` | ok |
| auto_blocker_matches_its_modifiers_exactly (F1 flip) | 8a08253 | same as T6 | ok |
| alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier (F2 flip) | 8a08253 | same as T6 | ok |
| shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press (F3 flip) | 8a08253 | same as T6 | ok |

The failing set on each red commit is exactly the set the developer lists (no extra red, none missing). Tests added with the fix (compile-red, carried by mutations): `blocker_matches_only_what_windows_sees_as_its_hotkey` (M1, M2, M6 below), `blocker_never_asks_windows_about_its_own_keys` (not killed on its own by my mutations; see Notes), `all_held_visits_every_held_code` (M8 area). Pins green throughout: `auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey`, `auto_blocker_keys_come_from_the_hotkey_lookup`.

Worktrees removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 3d65ac9 [fix/auto-blockers]`; `wt/` empty.

## Mutations (scratch worktree wt/mut, own target dir; each reverted with `git -C wt/mut checkout -- .`, `git diff --exit-code` clean)
| # | File | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | src/engine/matching.rs `blocker_matches` | drop `is_trigger_key(combo, pressed)` | blocker_matches_only_what_windows_sees_as_its_hotkey | caught: that test + ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing, shift_pressed_after_tab_was_released_is_still_swallowed, shift_pressed_while_alt_and_tab_are_held_reaches_windows (4 failed) |
| M2 | matching.rs | `\|\| !windows_has_down(vk)` -> `\|\| windows_has_down(vk)` | shift_pressed_while_alt_and_tab_are_held_reaches_windows (T2), T3 | caught: 10 failed incl. T2, T3, grave_blocker_acts_only_on_a_lone_grave, auto_blocker_matches_its_modifiers_exactly, the matching unit test |
| M3 | src/config/loader.rs dedup | scan skips the requesting binding (`&& !b.block_original_combo`) | auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys | caught: that test only (1 failed) |
| M4 | src/engine/mod.rs blocker branch | blocker also sets `immediate_fired = true` (shadows user binding) | user_binding_on_alt_shift_fires_with_the_blocker_in_either_order | caught: that test only (1 failed) |
| M5 | loader.rs | blockers placed before the file's bindings | auto_blockers_follow_the_bindings_of_the_file | caught: that test only (1 failed) |
| M6 | matching.rs | drop `keys_down(combo, held)` | blocker_matches_only_what_windows_sees_as_its_hotkey, T7 | caught: 14 failed incl. both |
| M7 | loader.rs | drop `!hotkeys.contains(&hotkey)` (duplicate blockers) | auto_blocker_deduped_when_two_bindings_want_the_same_combo | caught: that test only (1 failed) |
| M8 | src/engine/keys.rs `all_held` | range `0..256u16` -> `0..255u16` (skips code 255) | all_held_visits_every_held_code | NOT CAUGHT (306 passed). Added test below; then caught |

M8: code 255 (0xFF) can reach the table (`slot` accepts any code < 256), so the iteration bound is a real boundary. I added `engine::keys::tests::all_held_visits_the_highest_code_the_table_holds` (commit 3d65ac9). Under M8: `test engine::keys::tests::all_held_visits_the_highest_code_the_table_holds ... FAILED`, `306 passed; 1 failed`. On HEAD without the mutation: `307 passed; 0 failed`.

## Acceptance (cut.md:265-276, roadmap)
| Criterion | Evidence | Status |
|---|---|---|
| Alt+Shift+Tab passes through when the blocker exists | shift_pressed_while_alt_and_tab_are_held_reaches_windows (Alt, Tab held, then Shift: every event passed, total fired `[]`) ok; alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier ok (third key passes) | verified for the Alt,Tab,Shift order; Alt,Shift,Tab order still swallows the Shift (declared 1.2 residue, mask roadmap 1.2; pinned by shift_pressed_after_tab_was_released_is_still_swallowed). Read the criterion as the cut does. |
| Ctrl+Shift+T passes through | shift_pressed_while_ctrl_and_t_are_held_reaches_windows (both sides) ok; alt_shift_pressed_while_a_letter_is_held_reaches_windows ok | verified (Ctrl,T,Shift order; Ctrl,Shift,T = 1.2 residue) |
| A user binding on Alt+Shift fires with the blocker, either file order | user_binding_on_alt_shift_fires_with_the_blocker_in_either_order (blocker first and last) ok; user_binding_on_the_language_hotkey_fires_in_either_file_order ok; sided_user_binding_fires_and_the_blocker_keeps_the_other_sides ok | verified |
| Tests for loader dedup | auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order, ..._when_the_flagged_binding_uses_the_hotkey_keys, auto_blockers_follow_the_bindings_of_the_file, auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey, auto_blocker_keys_come_from_the_hotkey_lookup, auto_blocker_deduped_when_two_bindings_want_the_same_combo, auto_blocker_not_added_without_flag, auto_blocker_not_added_for_non_system_actions: all ok (19 of 19 in the targeted run, `running 19 tests`) | verified |
| Spec: fires only on exact held set + key belongs to combo | T2-T5, T9, T10, M3 pin, matching unit tests, F1 | verified |
| Spec: auto-blockers distinguishable | `is_auto_blocker` flag, asserted in L3/L4/C1 (loader tests) | verified |

Manual only (no `#[ignore]` stubs were added; impl.md:56-62 M1-M5 are human checks of the live hook, Windows Alt+Tab, Win+L, Ctrl+Shift browser): reported as manual, not verified.

## Failures
None.

## Tests added by the tester
- 3d65ac9 `Tests: all_held visits the highest key code the table holds (KS-05)`: `src/engine/keys.rs` (`#[cfg(test)]` only): `all_held_visits_the_highest_code_the_table_holds`. Co-Authored-By trailer present.

## Notes
- Safety greps: `rg "key-switch-rs.sock|SendInput|RegSetValue"` over `src/engine` and `src/config/loader.rs`: no hit. The new tests use the stand-in hotkey lookup and the fake Env. The pre-existing loader tests `default_config_parses`, `parse_simple_binding`, `auto_blocker_deduped_when_two_bindings_want_the_same_combo` load flagged bindings through `from_ron_str` and so READ `HKCU\Keyboard Layout\Toggle` (read only, allowed); they depend on the machine's hotkey (impl.md open point 1, KS-17 "3" fallback), not a RED.
- `blocker_never_asks_windows_about_its_own_keys` was not the unique killer of any of my eight mutations; the mutations that would affect it (asking about `pressed`) were not tried by me. Its value is for the reviewer.
- No real-clock timing tests in this lane's tests (fake clock), so no flake reruns.
- Critique M1 (`VIRTUAL_KEY(code)` built in the engine, `keys.rs:208`) is left to the reviewer under CLAUDE.md principle 7.
- Pre-existing, not a gate: rustdoc warning `src/data/vk_name.rs:4`. Stale agent rule texts (code-reviewer.md:124, project-analyst.md:60) are an orchestrator item (cut Q6).
