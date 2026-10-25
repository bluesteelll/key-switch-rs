# Tester: p0-A1-engine / Workstream A "Engine" (HEAD d3131fc vs master 73bbf40)

Verdict: GREEN. Gate clean, every red-first red on its parent and green on HEAD, 21 own mutations (19 caught; 2 equivalent mutants, 0 left uncaught after I added 4 tests). Four tester tests are added UNCOMMITTED and must be kept (they close real gaps, see Mutations).

HEAD `d3131fc7158977bbba6f81dfd54e1dd293e5b780`, branch `refactor/engine`, `git status --short` empty before I started. `master` = `73bbf404ec50003c88f2dbe353f429cff6e43f1f`.

## Gate (HEAD, before my additions)
- `cargo fmt --check`: OK (exit 0)
- `cargo clippy --all-targets --locked -- -D warnings`: OK. The first run said `Finished ... in 0.16s` (stale-looking); I touched `src/lib.rs`, `src/main.rs`, `src/engine/*.rs`, `src/hook/*.rs`, `src/data/*.rs` and re-ran: `Checking key-switch-rs ... Finished ... in 1.22s`, exit 0.
- `cargo test --locked --no-fail-fast`:
  - `Running unittests src\lib.rs` -> `running 189 tests` -> `test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - `unittests src\bin\swch.rs` -> `running 0 tests` (no tests in that binary; not a filter problem)
  - `Doc-tests key_switch_rs` -> `running 0 tests` (no doc tests)
  - Passed 189, failed 0, ignored 0. `rg '#\[ignore' src` = no hits, so there is no ignored test at all (no `manual:` tests exist; manual checks are listed below).
- `cargo build --release --locked`: OK (`Finished release profile [optimized] target(s) in 8.58s`). Daemon never run.
- After adding my 4 tests (uncommitted): `cargo fmt --check` OK, clippy `-D warnings` OK, `running 193 tests ... 193 passed; 0 failed; 0 ignored`.
- Flake reruns: `cargo test --locked --lib hook::` x5 -> `running 10 tests ... 10 passed` each time; `engine::` x5 -> `78 passed` each time (fake clock; only the `with_engine` re-entry test touches a real 5 s timeout and does not wait unless the guard is broken).

## Red-first replay
Scratch worktree `D:/tmp/ks-orch/p0-A1-engine/wt/replay`, `CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-A1-engine/target-wt/replay`.

| Test | Parent / commit | HEAD | Note |
|---|---|---|---|
| `engine::scenarios::hold_released_after_its_term_fires_when_its_late_timer_wakes` | `eef8629`: `running 1 test`, FAILED, `scenario: advance 10 ms to +260 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []` (scenarios.rs:407), `test result: FAILED. 0 passed; 1 failed` | `running 1 test` ... ok, `1 passed` | assertion-red, KS-25 |
| `engine::scenarios::equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` | `eef8629`: FAILED `advance 50 ms to +250 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []` | ok | assertion-red |
| `engine::scenarios::hold_left_to_a_late_timer_does_not_block_the_next_press` | `eef8629`: FAILED `advance 10 ms to +270 ms: expected to fire ..., fired []` | ok | assertion-red |
| `engine::scenarios::hold_released_before_its_term_with_a_late_timer_is_cancelled` | `eef8629`: ok | ok | declared guard, green on both as declared |
| `engine::gestures::tests::hold_released_at_or_after_its_due_is_left_to_its_timer`, `hold_left_to_its_timer_does_not_block_a_new_arm` | do not exist at `eef8629` (added in `a50e8d7`) | ok (`engine::gestures::` `running 17 tests ... 17 passed`) | carried by mutations M11 and M2 (both caught) |
| `engine::gestures::tests::forgotten_hold_never_fires_and_frees_its_key`, `engine::scenarios::hold_whose_timer_cannot_start_never_fires` | `a50e8d7` + scenario/harness hunks of `255c9a0`: `error[E0599]: no method named \`forget_timer\` found for struct \`Engine\`` (testing.rs:258) | ok | compile-red. Carried by M4: with `Engine::forget_timer` a no-op, NEITHER of these two tests failed (189 passed), see Mutations; my `engine_forget_timer_makes_a_late_timer_fire_nothing` closes it |
| `hook::keyboard_hook_callback::tests::{physical_key_events_reach_the_engine, injected_and_unknown_events_bypass_the_engine}`, `hook::keyboard_hook::tests::{with_engine_refuses_reentry_on_the_same_thread, with_engine_works_again_after_a_call}` | `9e9057e`: `error[E0599]: no method named \`with_engine\` found for struct \`KeyboardHook\`` (keyboard_hook.rs:312, 324, 325), `error[E0425]: cannot find function \`decide_event\`` (keyboard_hook_callback.rs:403) | ok (`hook::` `10 passed`) | compile-red. Carried by M5, M14, M20 (all caught) |
| every `engine::` test on `master` | `73bbf40`: `cargo test engine::` -> `running 0 tests` (module does not exist; vacuous, so no behavioural comparison). `hook::` on master: `running 26 tests ... 26 passed` | n/a | not usable as red evidence |
| KS-11 on master (cut.md optional) | scratch test in `src/hook/tap_state.rs` on `73bbf40` (`arm_tap(F13, DoNothing, 100)`, sleep 40, resolve, sleep 30, `arm_tap`, sleep 60, `assert!(resolve_tap_on_keyup, "KS-11")`): `panicked at src\hook\tap_state.rs:284:9: KS-11`, `test result: FAILED. 0 passed; 1 failed` | HEAD counterparts `engine::scenarios::two_quick_taps_fire_twice`, `stale_hold_timer_does_not_fire_the_next_press_early`, `four_quick_presses_double_tap_twice`, `engine::gestures::tests::stale_hold_timer_does_not_fire_a_newer_press` all ok; M2 and M21 (generation token removed) turn them red | bug is real on master, fixed by design on HEAD |

Worktrees removed. `git worktree list` afterwards: only `D:/key-switch-rs/key-switch-rs d3131fc [refactor/engine]`. Scratch `target-wt` deleted.

## Mutations
Scratch worktree `wt/mut` at HEAD (own `CARGO_TARGET_DIR`), one at a time, full `cargo test --locked --lib` each. The expected catching test was fixed in the script BEFORE running (`D:/tmp/ks-orch/p0-A1-engine/mut.py`).

| # | File | Mutation | Expected catcher | Result |
|---|---|---|---|---|
| M1 | engine/mod.rs `on_key_up` | key-up of a blocked key no longer suppressed | `immediate_single_key_fires_on_down_and_swallows_both_edges` | caught (+15 more, 16 failed incl. `hook::...::physical_key_events_reach_the_engine`) |
| M2 | engine/gestures.rs `arm_hold` | generation never incremented (KS-11) | `stale_hold_timer_does_not_fire_the_next_press_early`, `gestures::...::stale_hold_timer_does_not_fire_a_newer_press` | caught (3 failed, also `hold_left_to_its_timer_does_not_block_a_new_arm`) |
| M3 | engine/mod.rs `on_key` | modifier resync dropped | `modifier_lost_by_the_os_is_resynced_before_matching` | caught (1 failed) |
| M4 | engine/mod.rs `Engine::forget_timer` | body replaced by no-op (KS-33 fix) | `hold_whose_timer_cannot_start_never_fires` | NOT CAUGHT by existing tests (189 passed). Caught by added `engine::tester_tests::engine_forget_timer_makes_a_late_timer_fire_nothing` |
| M5 | hook/keyboard_hook_callback.rs `decide_event` | sentinel check removed | `injected_and_unknown_events_bypass_the_engine` | caught (1 failed) |
| M6 | engine/mod.rs Immediate | `block_default` ignored (always suppress) | `block_default_false_fires_and_lets_both_edges_through` | caught (1 failed) |
| M7 | engine/mod.rs `on_foreground_changed` | blocked flags not cleared | `foreground_change_forgets_pending_key_up_suppression` | caught (1 failed) |
| M8 | engine/gestures.rs `resolve_tap` | `now < due` -> `now <= due` | a tap-boundary test | caught: `gestures::tests::tap_released_at_or_after_its_term_does_not_fire`, `tap_held_past_its_term_fires_nothing_and_swallows_the_release`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` |
| M9 | engine/mod.rs | `when:` condition check removed | `when_*` scenarios | caught (3 failed) |
| M10 | engine/gestures.rs `double_tap` | term never expires | `double_tap_second_press_after_the_term_arms_again` | caught (+ `gestures::tests::double_tap_after_the_term_arms_again`) |
| M11 | engine/gestures.rs `release_hold` | always cancel (KS-25 fix reverted) | `hold_released_after_its_term_fires_when_its_late_timer_wakes` | caught (5 failed) |
| M12 | engine/chords.rs | expiry window off | `chord_does_not_fire_once_its_window_has_passed` | caught (+ `chords::tests::second_chord_key_after_window_starts_over`) |
| M13 | engine/sequences.rs | expiry off | `sequence_times_out_after_max_gap` | caught (+ `sequences::tests::step_after_max_gap_does_not_complete`) |
| M14 | hook/keyboard_hook.rs `with_engine` | re-entry guard removed | `with_engine_refuses_reentry_on_the_same_thread` | caught (1 failed, 5.01 s timeout as designed) |
| M15 | engine/mod.rs Tap | `if was_repeat { continue }` removed | any Tap + auto-repeat test | NOT CAUGHT by existing tests (189 passed). Caught by added `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` (an overdue Tap re-armed by a repeat then fired on release) |
| M16 | engine/mod.rs Chord | completion no longer re-marks the chord keys blocked | chord key-up test | NOT CAUGHT by existing tests. Only distinguishable after a focus change between the chord keys. Caught by added `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` |
| M17 | hook/keyboard_hook_callback.rs extern fn | `return LRESULT(1)` on suppress removed | (no test expected: extern fn untested) | NOT CAUGHT by existing tests. Caught by added `hook::keyboard_hook_callback::tests::tester_extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` |
| M18 | engine/mod.rs Sequence | last-step keys not re-marked blocked | `sequence_with_a_modifier_held_through_both_steps` | not caught; EQUIVALENT in reachable flows: the completing key is marked by `decision.suppress`; generic modifier codes (0x11) never equal a sided physical code (0xA2); a multi-key non-modifier step cannot complete because the first key of the step aborts the pending sequence. No test added |
| M19 | engine/mod.rs `on_key_up` | `tap_fired \|\| hold != NotPending` -> `tap_fired` | `tap_held_past_its_term_fires_nothing_and_swallows_the_release` | not caught; EQUIVALENT: the `else` branch's `take_blocked` returns true for the same key (suppressed at down). No test added |
| M20 | hook/keyboard_hook_callback.rs `key_event` | `WM_SYSKEYUP` unmapped | `physical_key_events_reach_the_engine` | caught (1 failed) |
| M21 | engine/gestures.rs `fire_hold` | matches `vk` only, not generation | `stale_hold_timer_does_not_fire_a_newer_press` | caught (also `engine::scenarios::stale_hold_timer_does_not_fire_the_next_press_early`) |

Reverted after each (`git checkout -- .`, then the tester patch re-applied and the diff compared with the patch file; no mismatch printed). Worktree removed.

## Acceptance
Brief acceptance plus the roadmap Workstream A lines.

| Criterion | Evidence | Status |
|---|---|---|
| quality gate passes | fmt OK, clippy `-D warnings` OK, `189 passed; 0 failed; 0 ignored` (193 with tester tests), release build OK | verified |
| `rg 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::' src/engine` finds nothing outside comments | run: one hit, `src/engine\mod.rs:73:` is a `//!` line ("adapter keeps it behind a `Mutex` in `KeyboardHook`..."). Stricter awk check from the pack (excludes comments, `testing.rs`, `scenarios.rs`, stops at `mod tests {`) over `GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::\|Instant::now\|ForegroundContext\|\.execute\(\|unsafe` -> no output at HEAD. With my uncommitted tests, `mod tester_tests` (not named `mod tests {`) shows `mod.rs:469: let t0 = Instant::now();`, a test line. Only other production-adjacent `Instant::now()` is `testing.rs:51` (fake clock seed, test code) | verified (grep literally yields one comment hit) |
| engine executes nothing | `rg '\.execute\(' src/engine` -> no hits (exit 1). Callers of `.execute(` in `src/hook`: only `keyboard_hook_callback.rs:175` (`carry_out`) | verified |
| `#![forbid(unsafe_code)]` in engine | `src/engine/mod.rs:122`; compiles | verified (probe reported by developer, not re-run) |
| the hook callback calls the engine for every key event | `hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine` (all four messages, down/up, bound and unbound), `injected_and_unknown_events_bypass_the_engine`; mutations M5 and M20 caught; added `tester_extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` drives the real extern fn on the global hook with a `DoNothing` binding | verified |
| no decision logic left in `src/hook` | raw `rg` of `Trigger::\|BindingKind::\|\.matches\(\|was_repeat\|block_default\|sync_modifiers\|mark_blocked\|take_blocked\|ActiveKeys\|TapState\|ChordState\|SequenceState` over `src/hook`: one hit, `keyboard_hook_callback.rs:318` `Trigger::Tap(10_000)` inside `mod tests`. Same check excluding tests: no output (exit 1). `ls src/hook` = `config_watcher.rs ipc_server.rs keyboard_hook.rs keyboard_hook_callback.rs mod.rs` (no `tap_state`/`chord_state`/`sequence_state`) | verified |
| scenario: Immediate combo | `immediate_single_key_fires_on_down_and_swallows_both_edges`, `immediate_combo_fires_on_its_last_key_and_lets_the_modifiers_through`, `right_hand_modifiers_match_unsided_combo_keys`, `combo_does_not_fire_without_all_of_its_keys`, `more_specific_immediate_binding_wins`, `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed`, `unbound_key_passes_through`; M1, M3, M6 caught | verified |
| scenario: Tap | `tap_fires_on_a_release_within_its_term`, `tap_held_past_its_term_fires_nothing_and_swallows_the_release`, `two_quick_taps_fire_twice`; M8 caught | verified |
| scenario: Hold | `hold_fires_once_its_term_passes_while_held`, `hold_released_early_is_cancelled`, `stale_hold_timer_does_not_fire_the_next_press_early`, `tap_and_hold_with_equal_terms_fire_exactly_one`, + the 3 KS-25 tests, `hold_whose_timer_cannot_start_never_fires`; M2, M11, M21 caught | verified |
| scenario: DoubleTap | `double_tap_fires_on_the_second_press_within_its_term`, `double_tap_second_press_after_the_term_arms_again`, `four_quick_presses_double_tap_twice`; M10 caught | verified |
| scenario: Sequence | `sequence_fires_when_its_steps_follow_within_max_gap`, `sequence_times_out_after_max_gap`, `sequence_broken_by_another_key_does_not_fire`, `sequence_with_a_modifier_held_through_both_steps`; M13 caught | verified |
| scenario: Chord | `chord_fires_when_all_keys_go_down_within_its_window`, `chord_keys_can_go_down_in_any_order`, `chord_does_not_fire_once_its_window_has_passed`, `key_outside_the_chord_passes`; M12 caught | verified |
| scenario: when-condition | `when_app_equals_matches_the_foreground_app_case_insensitively`, `when_title_contains_and_not_conditions`, `first_binding_whose_condition_holds_wins_on_a_shared_key`; M9 caught | verified |
| scenario: block_default false | `block_default_false_fires_and_lets_both_edges_through`; M6 caught | verified |
| architecture documented in module docs | `//!` in `src/engine/mod.rs:1-120`, `env.rs`, `gestures.rs:1-27`, `testing.rs:1-25`, `keyboard_hook_callback.rs:1-60` present (read the headings; not graded for prose quality, the reviewer does that) | verified (existence) |
| behaviour unchanged except design-made fixes, each in CHANGELOG Fixed with a test | CHANGELOG `### Fixed`: KS-11 (tests: `two_quick_taps_fire_twice`, `stale_hold_timer_does_not_fire_the_next_press_early`, `four_quick_presses_double_tap_twice`, `gestures::...::stale_hold_timer_does_not_fire_a_newer_press`), KS-25 equal terms (3 red-first tests above), KS-33 partly (`hold_whose_timer_cannot_start_never_fires`, `forgotten_hold_never_fires_and_frees_its_key`, plus my `engine_forget_timer_makes_a_late_timer_fire_nothing`). KS-11 reproduced red on master (see replay). Blank line at CHANGELOG.md:68 inside the Fixed list is cosmetic | verified |
| whole existing suite keeps passing | master hook tests (26) map to same names or the renamed ones listed in the pack (Fact 39); full suite 189 ok | verified (name mapping taken from pack, `--list` of HEAD matches count) |
| "same threads, same timing" (Decision executed as today) | `carry_out`/`start_hold_timer` have no unit test (real threads, `SendInput`) | manual, not verified |
| M1 default config CapsLock / Shift+CapsLock, M2 Tap(200)+Hold(200) 20 taps + 3 holds, M3 DoubleTap(200), M4 chord J+K / sequence F13,F14, M5 `TitleContains("Notepad")`, M6 Hold(300) Launch notepad no stall, M7 thread count (no `tap-gc`), M8 `PressKey("Left")` F13 held (sentinel feedback loop), M9 Hold(300) Launch under CPU load x10 | all need the live hook / real input; no `#[ignore = "manual: ..."]` test exists for any of them (the project has zero `#[ignore]`); they live as the manual list in cut.md | manual, not verified |

## Failures
None on HEAD. Gaps found and closed with added tests: M4, M15, M16, M17.

## Tests added by the tester (uncommitted, in the lane tree)
Patch: `D:/tmp/ks-orch/p0-A1-engine/tester_tests.patch` (`git diff`: `src/engine/mod.rs` +~60 lines, `src/hook/keyboard_hook_callback.rs` +~38 lines; nothing else touched, `git status --short` shows exactly these two files modified).
- `src/engine/mod.rs`, new `#[cfg(test)] mod tester_tests`:
  - `engine_forget_timer_makes_a_late_timer_fire_nothing` (M4; own `Env` with a fixed clock; `Engine::on_timer` after `forget_timer` must return `Decision::default()`)
  - `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` (M15; deliberately does not assert suppression of the repeat, see Notes)
  - `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (M16)
- `src/hook/keyboard_hook_callback.rs` tests: `tester_extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` (M17). It calls `keyboard_hook_callback` on the process-global hook with a `DoNothing` binding on F13; no hook is installed, no input injected (`CallNextHookEx(None, ..)` finds no chain), no action beyond `DoNothing` runs. It changes the global hook's bindings in the test process; no other test uses `get_hook()` (`rg get_hook` shows only `app.rs:88` and the callback file).
Each is red under its mutation and green on HEAD (quoted above). The developer/orchestrator should merge them into the regular test modules (rename without the `tester` prefix if preferred).

## Notes
- Safety: no daemon, no `swch`, no ignored tests, no hook, no `SendInput`. The one real-OS read in tests is `GetAsyncKeyState` through `HookEnv` in the two `decide_event` tests (read-only; they also match the F13 binding regardless of held modifiers, so a human typing during the run cannot flip them except for a held F13). No test binds the pipe `key-switch-rs.sock` or writes a `config.ron` (`rg` over `src` for the pipe name: only `ipc.rs` constants).
- Behaviour observation, not a regression (ported from master, KS-06, out of scope per cut.md): for Tap/Hold/DoubleTap an auto-repeat key-down hits `continue` and the binding contributes no `suppress`, so the repeat of a gesture key reaches the OS while the first press and the key-up are swallowed. My Tap-repeat test avoids pinning this.
- `Decision::mask` is never set (as documented, roadmap 1.2).
- Pre-existing items the reviewer may weigh: `CLAUDE.md:21-24,65` still describe the pre-engine layout and forbid a hook-wide `Mutex`, while the spec sanctions `Mutex<Engine>` in the adapter (open point in the pack); `eprintln!` in `start_hold_timer` instead of `log`; rustdoc warning `src/data/vk_name.rs:4` pre-existing.
- A `running 0 tests` appears for `swch` unittests and doc-tests; neither target has tests, they are not filtered-out targets.
