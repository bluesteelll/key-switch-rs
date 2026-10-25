# Tester: p1-1.6-one-winner / roadmap 1.6 (KS-13) (HEAD f7d54ef vs master 9357cfa)

Verdict: GREEN. Gate green; 12 engine and 2 loader red-first tests red by assertion on pre-fix code and green on HEAD; 11 mutations all caught; all acceptance criteria mapped to tests that run green.

HEAD `f7d54efc1439f577d6a2048b7dca15f3c2d80642` on `feat/one-winner`; `git status --short` empty before and after all work.

## Gate (HEAD)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK. The first run said "Finished in 0.14s" (looked stale); I touched the sources and re-ran: `Checking key-switch-rs ... Finished`, clean.
- cargo test --locked --no-fail-fast (log `test_full.log`), exit 0:
  - lib: `running 438 tests` / `test result: ok. 437 passed; 0 failed; 1 ignored`
  - swch bin: `running 0 tests` (no tests in that target, pre-existing)
  - tests/ipc_roundtrip.rs: `running 6 tests` / `test result: ok. 6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests` (none)
  - total 443 passed, 0 failed, 1 ignored. The ignored test is pre-existing: `data::condition::tests::foreground_context_names_the_focused_notepad` ("manual: start it, then focus a Notepad window within 5 s ..."). The lane adds no ignored test. `--ignored` never run.
- cargo build --release --locked: OK (`Finished release`)

## Red-first replay
Scratch worktrees under `D:/tmp/ks-orch/p1-1.6-one-winner/wt`, own `CARGO_TARGET_DIR` under `target-wt/<name>`.

Engine set: worktree at `ede8472` (master code `9357cfa` plus the red-first test commit), `cargo test --locked --lib engine::` gave `running 223 tests` / `test result: FAILED. 211 passed; 12 failed` (log `tester_red1.log`). All 12 fail by assertion, none by compile error.

| Test | pre-fix code | HEAD | Red line |
|---|---|---|---|
| scenarios::shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers (W1) | red (assert) | ok | up CapsLock at +70 ms: expected to fire [], fired [SwitchLanguage] |
| scenarios::shift_capslock_held_past_the_term_fires_no_capslock_hold (W2) | red | ok | advance 300 ms: expected to fire [], fired [PressKey(160)] |
| scenarios::a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order (W3) | red | ok | down Pause: expected [Media(PlayPause)], fired [SwitchLanguage] |
| scenarios::a_more_specific_winner_with_block_default_false_lets_the_key_through (W4) | red | ok | down F13: expected passed, it was suppressed |
| scenarios::a_conditional_winner_with_block_default_false_lets_the_key_through (W5) | red | ok | down F14: expected passed, it was suppressed |
| scenarios::a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing (W6) | red | ok | down F22: expected passed, it was suppressed |
| scenarios::a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding (W7) | red | ok | down CapsLock: expected [], fired [ToggleCapsLock] |
| scenarios::a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together (W8) | red | ok | advance 300 ms: expected [], fired [PressKey(160)] |
| scenarios::two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press (W9) | red | ok | down F13: expected [], fired [ToggleCapsLock] |
| scenarios::a_more_specific_tap_keeps_a_less_specific_hold_from_arming (W10) | red | ok | advance 300 ms: expected [], fired [SwitchLanguage] |
| scenarios::tap_and_hold_on_capslock_still_arm_together_as_one_group (W14) | red | ok | down CapsLock: expected [], fired [ToggleCapsLock] |
| engine::tests::a_when_condition_is_asked_only_for_a_binding_whose_keys_match (W16) | red | ok | panic at mod.rs:620 (foreground asked for a non-matching binding) |

Loader set: worktree at `69a8406`, `cargo test --locked --lib config::loader::tests::` gave `running 80 tests` / `test result: FAILED. 78 passed; 2 failed` (log `tester_red4.log`):

| Test | 69a8406 | HEAD | Red line |
|---|---|---|---|
| config::loader::tests::a_binding_that_duplicates_an_earlier_one_is_warned_about (L1) | red (assert) | ok | left: [] right: ["binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them"] |
| config::loader::tests::duplicates_are_found_across_key_order_terms_timings_and_binding_kinds (L3) | red | ok | left: [] right: 5 warnings |

Test added in the later commit 006ff5e (not named red-first, replayed by me): worktree at master `9357cfa` with `src/engine/scenarios.rs` copied from HEAD, `repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` gave `running 1 test` / `FAILED. 0 passed; 1 failed` at scenarios.rs:3860 `down J at +0 ms: expected the event to be passed, it was suppressed`. Green on HEAD (full run).

Green guards (green on both trees by design: W11, W12, W13, L2, L4, the 3 engine::properties tests): green on HEAD. Properties re-run: `running 3 tests` / 3 ok.

HEAD runs by filter (each with `running N`, N >= 1, all ok): W1, W2, W3, W4+W5 (`running 2 tests`), W6+W10 (`running 2 tests`), W14, L1, L3, L4, `tap_and_hold_with_equal_terms_fire_exactly_one`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`.

All scratch worktrees removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs f7d54ef [feat/one-winner]`.

## Mutations
Scratch worktree wt/mut at HEAD, one fault at a time. Each reverted with `git checkout -- .` and `git diff --exit-code` clean. Logs `tester_mut_M*.log`.

| # | File | Mutation | Test named before running | Result |
|---|---|---|---|---|
| M1 | engine/winner.rs `rank` | condition bit dropped: `(key_count, false)` | a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order | caught; 4 failed: that, W5, W7, winner::tests::rank_puts_more_keys_before_a_when_condition |
| M2 | engine/winner.rs `elect` | `>=` to `>` (a later equal rank replaces the winner) | non_repeating_winner_keeps_the_immediate_slot_on_a_repeat | caught; 2 failed: that and tap_and_hold_on_capslock_still_arm_together_as_one_group |
| M3 | engine/winner.rs `admits` | group ignores `when:` equality | a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together | caught; 1 failed, that one |
| M4 | engine/winner.rs `claim` | always returns true | two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press | caught; that and winner::tests::group_arms_claims_each_deferred_kind_once |
| M5 | engine/winner.rs `elect` | auto-blocker not skipped | a_passing_user_binding_does_not_stop_the_auto_blocker | caught; that and user_binding_on_alt_shift_fires_with_the_blocker_in_either_order |
| M6 | data/binding.rs `duplicates` | allow_extra_modifiers not compared | bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates | caught; that and data::binding::tests::duplicates_needs_the_same_kind_keys_trigger_kind_when_and_flag |
| M7 | engine/winner.rs `admits` | Immediate bindings admitted to the group | tap_and_hold_on_capslock_still_arm_together_as_one_group | caught; that and non_repeating_winner_keeps_the_immediate_slot_on_a_repeat |
| M8 | engine/winner.rs `admits` | member's own `combo_matches` dropped | a_group_member_arms_only_if_it_matches_the_press_itself | caught; 1 failed, that one |
| M10 | engine/winner.rs `elect` | `when:` never evaluated | a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order | caught; 8 failed incl. W3, W5, W7, W8, a_binding_whose_condition_holds_wins_on_a_shared_key |
| M11 | engine/mod.rs act loop | Immediate ignores `binding.repeat` | press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed | caught; 10 failed incl. repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down |
| M12 | engine/mod.rs act loop | `admits` replaced by `.is_some()` (no one-winner) | shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers | caught; 33 failed incl. W1, W2, W3, W4, W5, W6, W7, W8, W10, W14, W16 |

M9 was an edit whose pattern did not match; no code changed, and I replaced it with M10. For M7 I first guessed a different test and it was caught by a superset that includes the named one.

## Acceptance (roadmap 1.6 / spec)
| Criterion | Evidence | Status |
|---|---|---|
| Shift+CapsLock (Immediate) with CapsLock (Tap, allow_extra_modifiers: true) fires only ToggleCapsLock | scenarios::shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers ok; shift_capslock_held_past_the_term_fires_no_capslock_hold ok | verified |
| Conditional Pause wins over unconditional in either file order | scenarios::a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order ok | verified |
| Winner with block_default: false lets the key through although a loser has true | a_more_specific_winner_with_block_default_false_lets_the_key_through, a_conditional_winner_with_block_default_false_lets_the_key_through (`running 2 tests`, 2 ok); a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing ok | verified |
| Tap(200)+Hold(200) on CapsLock still arm together as one group | tap_and_hold_on_capslock_still_arm_together_as_one_group ok (Tap fires SwitchLanguage, Hold fires PressKey(LShift)); tap_and_hold_with_equal_terms_fire_exactly_one ok; equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one ok | verified |
| Duplicates produce a warning in a loader test | config::loader::tests::a_binding_that_duplicates_an_earlier_one_is_warned_about ok; duplicates_are_found_across_key_order_terms_timings_and_binding_kinds ok; negatives L2 and L4 ok | verified |
| Auto-blockers keep their own rule | a_passing_user_binding_does_not_stop_the_auto_blocker ok (M5 catches) | verified |
| Suppression symmetry | check_press_symmetry in W1-W14; engine::properties 3 ok with the new `winners` config | verified |
| Update README and CHANGELOG | `git diff master..HEAD --stat`: README.md and CHANGELOG.md changed (+126/-14 and +60); README RON block under `### Combo syntax` is loaded by readme_matching_examples_load (ok in the full run) | present; prose not proofread |

Manual, not verified (no `#[ignore]` added; human checks M1-M5 in `cut.md:408-418`): live-hook behaviour of Shift+CapsLock next to a CapsLock Tap, a conditional binding in a real foreground window, and the stderr `[WARN] ... duplicates ...` line when the daemon loads a real config. The `eprintln!` itself has no capture test; the warning list is tested through `parse_ron_str`.

## Failures
None.

## Tests added by the tester (`Tests:` commits)
None. No gap found: every mutation was caught.

## Notes
- Engine tests use the fake clock. The one `Instant::now()` in the diff builds the fake `ForegroundForbiddenEnv` of the W16 unit test. No test in the diff touches the production pipe name, a real `config.ron`, registry writes or input injection (a search of added lines for `key-switch-rs.sock`, `config.ron`, `SendInput`, `sleep`: only the `default_config.ron` diff header matched).
- Expected stderr noise `[WARN] ... duplicates ...` in W9 and the immediate-slot repeat scenario (duplicates on purpose).
- audit.md:36 and roadmap.md:617 still read Open for KS-13; the developer deferred this to the Phase-1 book sync.
- Logs in `D:/tmp/ks-orch/p1-1.6-one-winner/`: `test_full.log`, `tester_red1.log`, `tester_red4.log`, `tester_m1.log`, `tester_mut_M*.log`.
