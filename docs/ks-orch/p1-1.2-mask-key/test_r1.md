# Tester: p1-1.2-mask-key / roadmap 1.2 (HEAD 0187cde vs master ece1767)

Verdict: GREEN. Gate passes, all declared red-first tests are red on their red commits (assertion, or compile for H1/H2) and green on HEAD, 14 of 15 own mutations caught by named tests. The one uncaught mutation (M14) is not an acceptance criterion and is reported as a note.

HEAD `0187cdee8bd6e5416befc09b75bef38db9b98f5b` on `feat/mask-key`; `git status --short` empty before and after; `git worktree list` shows only the lane tree; no `Tests:` commit was needed.

## Gate (HEAD)
- `cargo fmt --check`: OK (no output).
- `cargo clippy --all-targets --locked -- -D warnings`: OK. First run `Finished ... in 0.08s` (stale), re-run after `touch src/lib.rs src/main.rs src/bin/swch.rs`: `Checking key-switch-rs ... Finished dev profile ... in 1.39s`, no warnings.
- `cargo test --locked --no-fail-fast` (log `D:/tmp/ks-orch/p1-1.2-mask-key/gate_test_head.log`), exit 0:
  - lib: `running 472 tests` / `test result: ok. 470 passed; 0 failed; 2 ignored`
  - bin key-switch-rs: `running 0 tests` (no tests in the binary, expected)
  - tests/ipc_roundtrip.rs: `running 6 tests` / `test result: ok. 6 passed; 0 failed; 0 ignored`
  - bin swch: `running 0 tests`
  - Totals: 476 passed, 0 failed, 2 ignored.
  - Ignored (manual, not run, not counted as pass): `data::condition::tests::foreground_context_names_the_focused_notepad` (pre-existing); `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (H3: "injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus ...").
- `cargo build --release --locked`: OK (`Finished release profile [optimized] target(s) in 6.36s`).

## Red-first replay
Test commits are the parents of the fix commits in a linear chain on master (`54e0673`'s parent is master `ece1767`; each later red commit sits on the previous green commit). The tests need the new API (`BlockerMode`, `menu_mask_inputs`, ...), so a literal master run of the tests does not compile; replay was done on each red commit, which equals master plus the tests (plus earlier green fixes). Scratch worktree `D:/tmp/ks-orch/p1-1.2-mask-key/wt/red1`, `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.2-mask-key/target-wt/red1`, logs `red1.log`, `red_44adabe.log`, `red_a3e1149.log`, `red_047da43.log`.

| Test | Red commit | Red result | HEAD |
|---|---|---|---|
| S1 win_q_post_message_masks_the_swallowed_q_and_its_release | 54e0673 | red (assert): `down Q at +0 ms: expected the menu mask, got none` | ok |
| S2 alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift | 54e0673 | red (assert): `down LShift ... expected the menu mask, got none` | ok |
| S3 right_alt_and_right_win_mask_a_swallowed_key | 54e0673 | red (assert) `down J ...` | ok |
| S4 swallowed_repeats_and_key_up_while_win_is_held_are_masked | 54e0673 | red (assert) `repeat Q at +500 ms ...` | ok |
| S8 alt_held_before_the_hook_saw_it_masks_a_swallowed_key | 54e0673 | red (assert) | ok |
| S9 win_held_through_remapping_on_masks_a_swallowed_key | 54e0673 | red (assert) | ok |
| S10 win_held_across_a_foreground_change_masks_a_swallowed_key | 54e0673 | red (assert) | ok |
| S11 sequence_and_chord_keys_swallowed_while_win_is_held_are_masked | 54e0673 | red (assert) `down F13 ...` | ok |
| engine::properties::every_press_is_suppressed_or_passed_as_a_whole | 54e0673 | red: `down K at +5019 ms: mask false, the rule says true` | ok |
| engine::properties::nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups | 54e0673 | red: `repeat CapsLock at +1697 ms: mask false, the rule says true` | ok |
| engine::properties::auto_repeats_fire_only_actions_that_repeat | 54e0673 | red: same | ok |
| (guards S5, S6, S7, S12) | 54e0673 | green on red commit as declared (`a_swallowed_event_without_alt_or_win_in_windows_is_not_masked`, `a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed`, `a_passed_event_is_not_masked`, `a_hold_timer_never_masks` all `ok`); each is carried by a mutation below (M13, M1, M11; S12 via the harness assertion, see note under Mutations) | ok |
| engine red run total | 54e0673 | `running 238 tests` ... `test result: FAILED. 227 passed; 11 failed` (exactly S1-S4, S8-S11 + 3 properties) | |
| H1 menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel | 44adabe | compile-red: `error[E0425]: cannot find function menu_mask_inputs` | ok; carried by M4 |
| H2 carry_out_sends_the_menu_mask_only_when_the_decision_asks | 44adabe | compile-red: `cannot find value MASKS_SENT` | ok; carried by M5 |
| H3 the_menu_mask_key_is_inserted_into_the_input_stream | 44adabe | compile-red | ignored (manual), not run |
| L1 block_original_combo_takes_true_false_and_mask_only | a3e1149 | red: `Expected boolean` (MaskOnly unparseable) | ok |
| L2 a_misspelled_block_original_combo_value_names_the_valid_ones | a3e1149 | red | ok |
| L4 mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns | a3e1149 | red `2:78: Expected boolean` | ok |
| L5 true_and_mask_only_on_one_hotkey_give_one_blocker_each | a3e1149 | red | ok |
| S16 mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey | a3e1149 | red (config does not load) | ok |
| S17 mask_only_blocker_lets_ctrl_shift_t_reach_the_application | a3e1149 | red | ok |
| S18 mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held | a3e1149 | red | ok |
| S19 mask_only_blocker_masks_only_the_first_event_after_the_match | a3e1149 | red | ok |
| S20 alt_shift_x_fires_next_to_a_mask_only_blocker | a3e1149 | red | ok |
| S21 remapping_off_drops_an_owed_mask | a3e1149 | red | ok |
| a3e1149 run total | | `running 464 tests` ... `FAILED. 452 passed; 10 failed; 2 ignored` | |
| L7 the_blocker_takes_the_when_of_the_binding_that_asked_for_it | 047da43 | red: `left: Always right: AppEquals("x.exe")` | ok |
| L8 blockers_of_one_hotkey_combine_the_when_of_their_owners | 047da43 | red: `left: [(Some(Swallow), Always)] right: [... Or([AppEquals("a.exe"), AppEquals("b.exe")])]` | ok |
| L9 a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when | 047da43 | red: `left: [(Some(Swallow), Always), (Some(MaskOnly), Always)]` | ok |
| S13 a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps | 047da43 | red: `down LShift at +0 ms: expected the event to be passed, it was suppressed` | ok |
| S14 blockers_act_where_any_owner_when_holds | 047da43 | red: same | ok |
| E1 engine::tests::a_blocker_when_condition_is_asked_only_once_its_hotkey_matches | 047da43 | red: `the foreground was asked for although no binding with a when: matches this key` | ok |
| 047da43 run total | | `running 472 tests` ... `FAILED. 464 passed; 6 failed; 2 ignored` | |

On the L/S16-S21 reds: the failure is "config does not load" (feature absent), a behavioural red from the loader, not a compile error. Worktree removed with `git worktree remove --force`; `git worktree list` shows only the lane tree.

## Mutations
Scratch worktree `D:/tmp/ks-orch/p1-1.2-mask-key/wt/mut` at HEAD, `CARGO_TARGET_DIR=.../target-wt/mut`, script `mut.py`, logs `mut_M*.log`. Each mutation ran the full `cargo test --locked --lib`, was reverted with `git checkout -- .` and `git diff --exit-code` (clean each time). Worktree removed.

| # | File | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | src/engine/keys.rs `alt_or_win_down_in_windows` | count Alt/Win whatever its record (drop `!key_down_swallowed`) | S6 a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed | caught (also keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed + 3 properties) |
| M2 | src/engine/mod.rs blocker branch | drop `if !was_repeat` on MaskOnly | S19 mask_only_blocker_masks_only_the_first_event_after_the_match | caught (only S19 fails: 1 failed) |
| M3 | src/config/loader.rs `any_of_conditions` | single owner's condition replaced by `Always` | L7 the_blocker_takes_the_when_of_the_binding_that_asked_for_it | caught (L7, L8, L9, S13: 4 failed) |
| M4 | keyboard_hook_callback.rs `menu_mask_inputs` | `dwExtraInfo: 0` instead of the sentinel | H1 (compile-red carrier) | caught (H1 only) |
| M5 | keyboard_hook_callback.rs `carry_out` | `if !mask { send_menu_mask() }` | H2 (compile-red carrier) | caught (H2 only) |
| M6 | src/engine/mod.rs `on_key_up` | remove `mask_if_swallowed_under_alt_or_win` on key-up | S4 swallowed_repeats_and_key_up_while_win_is_held_are_masked | caught (13 failed incl. S1, S4) |
| M7 | src/engine/mod.rs `on_key` | drop `decision.mask \|= mask_owed` | S16 | caught (S16, S17, S19: 3 failed) |
| M8 | src/engine/mod.rs blocker branch | MaskOnly also sets `suppress` (v2 swallows the modifier) | S16 | caught (S16, S17, S19, S20: 4 failed) |
| M9 | src/config/loader.rs `raw_blocker_to_mode` | `MaskOnly -> Swallow` | L1 block_original_combo_takes_true_false_and_mask_only | caught (8 failed incl. L1, L4, L5, L9, S16-S20) |
| M10 | src/config/loader.rs fallback | MaskOnly never downgraded on non-modifier-only hotkey | L4 mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns | caught (L4 only) |
| M11 | src/engine/mod.rs `mask_if_swallowed_under_alt_or_win` | mask without `decision.suppress` | S7 a_passed_event_is_not_masked | caught (S7 + 14 more) |
| M12 | src/engine/mod.rs blocker branch | drop `binding.condition.evaluate` (blocker ignores `when:`) | S13 a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps | caught (S13, S14: 2 failed) |
| M13 | src/engine/mod.rs same helper | `mask = suppress` (no Alt/Win check) | S5 a_swallowed_event_without_alt_or_win_in_windows_is_not_masked | caught (S5, S6, ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing, S4 + properties: 7 failed) |
| M14 | keyboard_hook_callback.rs `carry_out` | send the mask after timers and actions instead of first | none expected (ordering) | NOT CAUGHT, `exit 0`, 0 failed. See Notes; not an acceptance criterion |
| M15 | src/engine/mod.rs blocker branch | MaskOnly sets `mask_owed = false` | S16 | caught (S16, S17, S19: 3 failed) |

S12 a_hold_timer_never_masks is a guard through the harness assertion `testing.rs:415` (`advance_ms` asserts `!decision.mask`); not separately mutated (no `on_timer` mask mutation run; named in the developer's report as the killer of that mutation).

## Acceptance
| Criterion (cut / roadmap 1.2) | Evidence | Status |
|---|---|---|
| mask = true exactly when an event is suppressed after Alt or Win reached the OS | true: S1, S2, S3, S4, S8, S9, S10, S11 `ok`; false: S5, S6, S7, S12 `ok`; unit `engine::keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed` `ok`; property oracle (`check_mask_rule`) in the 3 `engine::properties::*` tests `ok`. Red on 54e0673, killed by M1, M6, M11, M13 | verified |
| opt-in v2 passes the modifier key-downs and sets mask on an exact match | S16 (`mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey`), S17 (Ctrl+Shift+T reaches the app), S18 (no mask with another key held), S19, S20, S21; config surface L1, L2, L4, L5 `ok`. Killed by M2, M7, M8, M9, M15 | verified |
| a blocker requested by a binding with `when: AppEquals("x.exe")` is inactive in other apps | S13 `ok` (red on 047da43: `expected the event to be passed, it was suppressed`), S14, L7, L8, L9, E1 `ok`. Killed by M3, M12 | verified |
| mask injected with vk 0xE8 down+up tagged with the sentinel, outside the engine lock, before the hook returns | H1, H2 `ok` (killed by M4, M5); `carry_out` runs after `with_engine` returns and before `LRESULT(1)`/`CallNextHookEx` (callback.rs:339-349, read). Sentinel on all `SendInput` sites (rg output: `dwExtraInfo: injected_sentinel()` at system_function.rs:131,143, windows_actions.rs:200,212, sequence.rs:176,247, callback.rs:234). Actual injection only in H3 `#[ignore = "manual: ..."]` | engine/inputs verified; injection and ordering vs actions: manual, not verified (see M14) |
| README, default_config.ron, book config pages, CHANGELOG updated | `git diff --stat master...HEAD` shows CHANGELOG.md, README.md, src/config/default_config.ron, book/src/config/{matching,example,actions,known-issues,triggers-conditions,errors,README,ron-syntax,structure}.md; loader tests `readme_matching_examples_load` and `the_template_and_its_examples_have_no_duplicate_bindings` are in the 470 passed | verified by presence and loader tests; content not proofread by tester |
| manual checks listed | cut.md / pack M1-M9 (Alt then Shift in Notepad no menu; Win+Q -> PostMessage no Start; v2 Ctrl+Shift+T works and bare Alt+Shift does not switch; others) | manual, not verified |

Manual-only items: M1 (Alt, Shift in Notepad: no menu bar), M2 (Win+Q -> PostMessage(WM_CLOSE): Start stays closed), M3 (Alt+Tab switcher with Shift: no menu bar), M4 (v2 Ctrl+Shift hotkey: Ctrl+Shift+T reopens tab, bare Ctrl+Shift never switches), M5 (v2 Alt+Shift: bare Alt+Shift never switches, Notepad no menu), M6 (`true` + `when: AppEquals("notepad.exe")`), M7 (MaskOnly on a non-modifier-only hotkey prints the `[WARN]`), M8 (H3 with `--ignored`, in a debug build, Notepad focused), M9 (`osk.exe` shows no stuck modifier). Hypothesis that Windows switches language only on a clean Alt+Shift release is unverifiable by agents; it is why v2 is opt-in.

## Safety checks
- No test binds the production pipe: `tests/ipc_roundtrip.rs` uses `unique_pipe_name(..)`; `ipc.rs:319` only formats the production name as a string. No test writes a real `config.ron`.
- Under `cfg(test)` `send_menu_mask` only increments a thread-local counter (callback.rs:268-277), so H2 and the engine-driven callback tests never call `SendInput`. `tests/` contains nothing reaching `carry_out`.
- I did not run the daemon, `swch`, `--ignored` tests, or anything hooking/injecting input.

## Failures
None.

## Tests added by the tester
None (no `Tests:` commit).

## Notes
- M14: moving `send_menu_mask()` after the timers and actions in `carry_out` is not caught by any test. The doc says the mask goes ahead of every key an action sends, a design claim and not an acceptance bullet. Every `BindAction::execute` touches the OS (SendInput, PostMessage, ShellExecute, thread spawn), so a test that observes order would need a new test-only action seam; not added. Reviewer may decide whether to ask for such a seam; manual check M2 (Win+Q -> PostMessage) is the practical coverage.
- No test covers the wiring `decide_event -> carry_out` inside `keyboard_hook_callback` itself (needs the live hook singleton); it was verified by reading lines 339-349 only.
- The Alt+Tab switcher risk (matching.md:109, M3) and the v2 hypothesis stay manual.
- Public API change (`Binding::block_original_combo: Option<BlockerMode>`) breaks the cfgcheck snippet in `book/src/methodology.md:408-409`; listed in the developer's hand-off, outside the tester scope.
- Timing: the gate's engine tests use the fake clock; the only real-clock test target is `tests/ipc_roundtrip.rs` (5.01 s, passed once; not re-run 5 times because this lane did not touch it).
- Logs and scripts: `D:/tmp/ks-orch/p1-1.2-mask-key/{gate_test_head.log,red1.log,red_*.log,mut_M*.log,mut.py}`. Scratch worktrees removed; `target-wt/` contents remain outside the repo.
