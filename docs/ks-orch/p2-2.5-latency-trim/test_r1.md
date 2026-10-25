# Tester: p2-2.5-latency-trim / roadmap 2.5 (HEAD d853a59 vs master 3cde1d0, P = 62c9fd8)

Verdict: GREEN. The gate, every red-first replay, 60+ mutations and the bench all hold. Two tester `Tests:` commits closed
three mutation gaps (X12, X41b, MY-b). One bench cell is a 9 ns excess on a 100 ns grid (R12m2, below), reported, not a breach
in the unrounded numbers.

HEAD: `d853a597f976cd9195c79852e4f69176262c02cc` on `perf/latency-trim`, `git status --short` empty (0 lines).
The developer's HEAD was `786e61f`; the two commits above it are the tester's (`eb15d53`, `d853a59`), test code only.
`git diff --stat 786e61f..HEAD` touches `src/engine/mod.rs` and `src/hook/main_thread.rs` inside `mod tests` only.

## Gate (HEAD)

- `cargo fmt --check`: OK (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: OK (`Finished dev profile`, no warning), run on `786e61f` and again on `d853a59`.
- `cargo test --locked --no-fail-fast`:
  - on `786e61f` (`D:/tmp/ks-orch/p2-2.5-latency-trim/gate_test.log`):
    - `running 866 tests` / `test result: ok. 859 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 4.05s`
    - `running 0 tests` (swch bin), `running 7 tests` / `test result: ok. 7 passed; 0 failed; 0 ignored` (integration), `running 0 tests` (doc).
  - on `d853a59` (`gate_test_final.log`): `running 869 tests` / `FAILED. 861 passed; 1 failed; 7 ignored`. The one failure is
    `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` ("Delay(50) ended 12.2625ms late in the median"), a real-clock
    test of item 2.1 that this lane does not touch (`git diff 3cde1d0..HEAD --stat -- src/core` is empty). It flaked on master as well:
    8 runs of `cargo test --lib` on master gave 1 failure of the same test ("Delay(20) ended 6.2963ms late in the median"), and 8 runs
    on HEAD gave `862 passed; 0 failed; 7 ignored` every time. Five more filtered runs and three more full lib runs on HEAD were green.
    Counted: **862 passed, 0 failed (flake aside), 7 ignored** (+7 integration passed).
- `cargo build --release --locked`: OK (`Finished release profile`).
- Ignored (7), all `manual:` or `bench:`, none run: `bench::latency_bench` (bench), `core::executor::tests::the_win32_runner_opens_a_url`,
  `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`,
  `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it`, `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`,
  `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. The diff adds and removes no `#[ignore`.
- Safety grep over the added `src` lines: no `key-switch-rs.sock`, no `config.ron`, no registry write, no `SendInput(`, no hook install.
  The added `unsafe` blocks (`tick_count`, bench `GetTickCount`/`GetAsyncKeyState`, test `PeekMessageW`, the extern foreground callback with a null window) carry
  `SAFETY:` comments; no `Ordering::Relaxed`, no `static mut`, no `rg "log::|println!"` hit in `keyboard_hook_callback.rs`.
- Q10: `git diff --stat 3cde1d0 62c9fd8 -- src` lists `src/bench.rs | 119` and `src/platform/timers.rs | 9` only, so P runs master's engine and adapter.

## Red-first replay

Scratch worktree `D:/tmp/ks-orch/p2-2.5-latency-trim/wt/replay` with its own `CARGO_TARGET_DIR`; removed afterwards. All run
`cargo test --locked --lib <path> -- --exact`; every run printed `running 1 test`.

| Test (path as at HEAD) | Red commit | Output on the red commit | Green |
|---|---|---|---|
| `engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows` | `9a9bdcf` (tree = P + test) | `scenario: up LShift at +0 ms: Windows got a lone key-up of LShift`; `FAILED. 0 passed; 1 failed` | `c6b186b`: `ok. 1 passed`; HEAD: `ok. 1 passed`. Master has no such test (`running 0 tests`), so the replay is on `9a9bdcf` as cut.md says |
| `engine::tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` | `b0db237` | ``assertion `left == right` failed: a key event with no modifier held asks Windows about no modifier (roadmap item 2.5)`` `left: 44` `right: 0` | `3f0dc62`: `ok`; HEAD: `ok` |
| `engine::scenarios::a_modifier_that_a_later_hook_swallows_does_not_count_for_matching` (R-A) | `d5458d2` (= `3f0dc62` + test-only hunks, checked with `git diff 3f0dc62 d5458d2 -- src`) | `scenario: down J at +0 ms: expected to fire [DoNothing], fired []` | `e84625e` ok; HEAD ok |
| `engine::scenarios::a_shift_that_a_later_hook_swallows_does_not_make_alt_the_language_hotkey` (R-B) | `d5458d2` | `scenario: down LAlt at +0 ms: expected the event to be passed, it was suppressed` | `e84625e` ok; HEAD ok |
| `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_after_a_note` (R-C) | `d5458d2` | `scenario: down CapsLock at +0 ms: expected to fire [SwitchLanguage], fired [ToggleCapsLock]` | `e84625e` ok; HEAD ok |
| `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_at_the_next_key` (R-D) | `d5458d2` | same as R-C | `e84625e` ok; HEAD ok |
| `engine::scenarios::a_modifier_released_where_the_hook_cannot_see_it_stops_counting_at_the_next_key` (R-E) | `d5458d2` | same as R-C | `e84625e` ok; HEAD ok |
| `engine::tests::a_key_event_asks_windows_once_for_each_modifier_held_for_it_and_never_for_its_own_key_or_a_swallowed_one` (R-F) | `d5458d2` | ``A down with LCtrl held: a key event asks Windows once about each modifier held for it, never about its own key or a swallowed one (roadmap item 2.5)`` `left: 0` `right: 1` | `e84625e` ok; HEAD ok |
| `engine::keys::tests::the_drain_read_clears_a_generic_slot_only_when_its_code_reads_up_and_never_sets_one` (R-G) | `d5458d2` | `Windows has neither Shift down: the drain read clears the generic slot (roadmap item 2.5)` | `e84625e` ok; HEAD ok |
| `engine::scenarios::a_swallowed_key_up_asks_no_mask_for_an_alt_that_windows_has_up` (R-H) | `d5458d2` | `scenario: up F13 at +0 ms: mask true, the rule says false (suppressed true, Alt or Win down in Windows false)` | `e84625e` ok; HEAD ok |
| `hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick` (D12, removed by L3) | `47af1b1` (declared red commit): ``read at the first tick: [(0, 1)]`` `left: Some((0, 1))` `right: Some((250, 1))` | `FAILED. 1 passed; 1 failed` | `b106db9` removes it; the restored 2.4 guard `the_first_raw_key_event_after_idle_wakes_the_watchdog` is `ok` there. Nothing to replay at HEAD |

HEAD quotes for the eight R-tests: each printed `running 1 test` and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 865 filtered out`.
`hook::watchdog::` at HEAD: `running 79 tests` / `ok. 79 passed`. The compile-red tests of commits 4, 5, 7, 9, 10 and 11 are carried by the mutations below.

58 tests are new against master and 1 is gone (`engine::keys::tests::sync_overwrites_modifiers_except_the_skipped_key`, whose function `sync_modifiers` no longer exists);
`cargo test --lib -- --list` on master (808) and HEAD (866) gives the diff, and every added test appears in the mutation table or the acceptance table.

Declared by the developer and confirmed: `47af1b1` fails the gate (one red assertion, quoted above); history is not rewritten.

Worktrees removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs d853a59 [perf/latency-trim]`.

## Mutations

Scratch worktree `wt/mut` (own `CARGO_TARGET_DIR`), one mutation at a time by exact-string replacement, then restored and
`git diff --exit-code` clean after each batch (CLEAN printed). Script `scratchpad/mutate.py`; raw logs `D:/tmp/ks-orch/p2-2.5-latency-trim/mutations_pass1.log`..`pass5.log`, `mutations.log`.
Every run printed `running N tests` with N >= 1 and a `FAILED` line from the named catcher unless marked.

| # | File | Mutation | Named catcher | Result |
|---|---|---|---|---|
| X51 | engine/mod.rs (`on_key_up`) | witness term removed: `decision.suppress = swallow;` | `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` | caught |
| X4 | engine/mod.rs | D10: no `mark_repeat_suppressed` | RF-3 scenario | caught |
| X1 | engine/keys.rs `take_blocked` | ignores `RepeatSuppressed` | `keys::tests::a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset` | caught |
| X2 | keys.rs `key_down_swallowed` | counts `RepeatSuppressed` | `keys::tests::a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask` | caught |
| X3 | engine/mod.rs repeat branch | no `mark_passed` | `tests::a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed` | caught |
| X5 | resync.rs | `hiding: 0` | `resync::tests::a_new_resync_plans_a_full_read_at_its_first_plan` | caught |
| X6 | resync.rs `bump` | no-op | `resync::tests::a_bump_plans_one_full_read`, scenario `a_modifier_pressed_where_the_hook_cannot_see_it_counts_after_a_foreground_change` | caught (both) |
| X7 | resync.rs `plan` | `notes > notes_seen` | `resync::tests::a_changed_note_count_plans_a_full_read_a_smaller_one_included` | caught |
| X8 | resync.rs `plan` | reads fresh codes | `resync::tests::a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next` | caught |
| X9 | resync.rs `plan` | ignores `input_away` | `resync::tests::nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided` | caught |
| X10 | resync.rs `event` | decided does not clear `input_away` | same test and scenario `..._on_the_lock_screen_counts_after_the_first_key_when_no_return_signal_comes` | caught (both) |
| X11 | resync.rs `input_left` | no bump | scenarios `..._lock_screen_counts_after_the_unlock_signal` / `..._first_key_when_no_return_signal_comes` | caught |
| X12 | engine/mod.rs `track_injected` | `resync.event(vk, true)` | listed catcher was a `Resync::event` unit test, which cannot see the call site. **First run: not caught by the whole suite (859 tests).** Closed by tester test `tests::the_daemons_own_injected_events_do_not_bring_keyboard_input_back_to_the_drain` | caught after `eb15d53` |
| X13 | engine/mod.rs `input_returned` | no-op | `tests::every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`, scenario `..._unlock_signal` | caught |
| X14 | engine/mod.rs `on_foreground_changed` | no bump | same test + scenario `..._counts_after_a_foreground_change` | caught |
| X15 | engine/mod.rs reset | `Reloaded` bumps | `tests::a_reload_makes_no_read` | caught |
| X16 | engine/mod.rs reset | no `read_everything` | `tests::swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing` | caught |
| X17 | keys.rs `resync_modifiers` | asks unset generic slot | `tests::a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one`, `keys::tests::the_drain_read_clears_a_generic_slot_only_when_its_code_reads_up_and_never_sets_one` | caught |
| X18 | modifiers.rs `touched_by` | sided code gets no generic bit | `modifiers::tests::a_sided_modifier_touches_its_generic_code_and_a_generic_one_both_sides` | caught |
| X19 | main_thread.rs `drain` | no `resync_if_due` | `main_thread::tests::the_drain_reads_the_modifiers_after_a_note_and_not_again` | caught |
| X19b | main_thread.rs `input_returned` | engine not told | `main_thread::tests::a_return_signal_makes_the_next_drain_read_the_modifiers` | caught |
| X20a | main_thread.rs `with_main` | no note on refusal | `main_thread::tests::a_refused_call_notes_hidden_input` | caught |
| X20b | main_thread.rs | no note on unknown desktop switch | `main_thread::tests::a_desktop_switch_to_an_unknown_destination_notes_hidden_input` | caught |
| X20c | keyboard_hook_callback.rs | foreground callback posts no wake | `keyboard_hook_callback::tests::a_foreground_change_queues_a_wake_for_the_drain` | caught |
| X21 | engine/mod.rs `on_key` | per-event full read back (master's cost) | `tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` + R-F | caught |
| X22 | keys.rs `resync_from_os` | reads generic codes | `keys::tests::a_reset_never_takes_a_generic_modifier_from_the_os` | caught |
| X23a | keys.rs `track_injected` | key-up sets the flag | engine-level `tests::an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held` and scenario I-1 pass under it (the check at the next key repairs it); caught by `keyboard_hook_callback::tests::the_daemons_own_injected_modifiers_pass_and_are_tracked` in the full run | caught (by the callback test only) |
| X23b | keyboard_hook_callback.rs `decide_event` | sentinel event not tracked | `keyboard_hook_callback::tests::the_daemons_own_injected_modifiers_pass_and_are_tracked` | caught |
| X23c | keys.rs `track_injected` | non-modifier tracked | `tests::an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing` | caught |
| X24 | engine/mod.rs reset | `RemappingSwitched`/`HookReinstalled` keep keys | scenarios `a_modifier_pressed_while_remapping_is_off_counts_after_swch_on`, `..._hook_was_gone_counts_after_the_reinstall` | caught |
| X25 | keys.rs check | clears a held generic slot regardless | scenario `another_programs_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up` (G1) | caught |
| X26 | engine/mod.rs `on_key` | check removed | R-A, R-B, R-D, R-E, R-F (7 failures in a full run, incl. the V3-I1 pin) | caught |
| X27 | keys.rs check | visits every slot | R-F, `keys::tests::the_check_reads_only_held_modifiers_skips_its_own_key_and_a_swallowed_one_and_never_sets_a_flag` | caught |
| X28 | keys.rs check | KS-14 skip dropped | scenario `a_swallowed_modifier_stays_held_through_both_reads_and_its_next_press_stays_a_repeat`, C-1 | caught |
| X29 | keys.rs check | generic read replaced by sided | `keys::tests::the_check_asks_a_generic_slot_its_generic_code_and_clears_it_only_when_it_reads_up` | caught |
| X30 | keys.rs check | own key not skipped | R-F, C-1 | caught |
| X31a | engine/mod.rs `on_key_up` | key-up check over `ALL` | R-F | caught |
| X31b | engine/mod.rs `on_key_up` | key-up check at a passing key-up too | R-F | caught |
| X32 | engine/mod.rs `on_key_up` | key-up check removed | `scenarios::a_swallowed_key_up_asks_no_mask_for_an_alt_that_windows_has_up` (R-H) | caught |
| X33 | keys.rs `resync_modifiers` | generic slot never cleared | R-G, scenario R-C | caught |
| X38 / X38b / X38c | callback.rs `late_callback` / `event_age_ms` | `>` instead of `>=` on age; signed rule removed; `>` on run time | `keyboard_hook_callback::tests::lateness_counts_age_and_run_with_a_17_ms_boundary` | caught (3 of 3) |
| X39a | callback.rs | no note when late | `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` | caught |
| X39b | callback.rs | always note | `a_callback_for_a_fresh_event_notes_nothing` | caught |
| X40 | callback.rs | injected stamp counts | `an_injected_events_own_stamp_is_not_its_age` | caught |
| X41a | main_thread.rs E5 | `>` instead of `>=` | `main_thread::tests::an_awake_stretch_of_17_ms_notes_hidden_input` | caught |
| X41b | main_thread.rs `run` | `continue` after the E5 note removed (loop would wait on an unread note) | the listed `the_main_loop_checks_its_awake_stretch_before_every_wait` is a source scan that does not look at `continue`. **First runs: not caught by the whole suite.** Closed by tester test `main_thread::tests::a_long_stretch_sends_the_main_loop_back_to_the_drain_before_the_wait` | caught after `eb15d53` |
| X41c | counters.rs | `HookLate => true` | `logging::counters::tests::hook_late_alone_is_reported_at_info_and_with_another_count_at_warn`, `logging::writer::tests::a_report_whose_only_count_is_hook_late_is_written_at_info` | caught (both) |
| X42 | health.rs | `note_unseen_input` no-op | `health::tests::unseen_input_counts_each_note` | caught |
| X43 | watchdog/decision.rs | `unseen_at_last_tick = 0` | `decision::tests::a_tick_reports_raw_key_events_no_callback_explains` | caught |
| X44a / X44b | watchdog/mod.rs | wake dropped / note dropped | `watchdog::tests::a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` | caught (both) |
| X45 | main_thread.rs | watchdog notes not summed | `main_thread::tests::the_drain_counts_the_watchdogs_notes` | caught |
| X50 | engine/mod.rs `on_key` | check skips `touched_by(LWin)` (D17 restored) | `tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` | caught |

The tester's own mutations (not in the cut's table), each with the test that caught it in a full `cargo test --lib` run:

| # | File | Mutation | Catcher | Result |
|---|---|---|---|---|
| MY-w1 | engine/mod.rs `on_key_up` | witness inverted: `swallow && env.os_key_down(vk)` | the W1 pin `only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` passes (it counts reads, not results); 170 other tests fail, among them the properties `every_press_is_suppressed_or_passed_as_a_whole` and `no_key_sticks_...unnoticed` and the scenario `a_swallowed_key_up_asks_no_mask_for_an_alt_that_windows_has_up` | caught |
| MY-a | keys.rs `alt_or_win_down_in_windows` | drops the KS-14 term | `keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed`, `scenarios::a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed` | caught |
| MY-b | main_thread.rs `MainState::resync_if_due` | `is_enabled` early return removed (drain reads while remapping is off) | **no test in the suite.** Closed by tester test `main_thread::tests::the_drain_reads_nothing_while_remapping_is_off` | caught after `d853a59` |
| MY-c | watchdog/mod.rs | `unseen_at_last_tick() >= 0` (always note) | `watchdog::tests::a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` | caught |
| MY-d | keys.rs `track_injected` | KS-14 skip removed | `tests::an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held` | caught |
| MY-e | keys.rs `resync_modifiers` | KS-14 skip removed from the drain read | `keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed`, `tests::a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one`, S9 | caught |
| MY-f | engine/mod.rs `on_key` | a decided event no longer clears `input_away` | `tests::every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`, the lock-screen scenario | caught |
| MY-g | main_thread.rs `input_returned` | foreground cache not re-read | `main_thread::tests::a_return_signal_reads_the_foreground_window_again` | caught |
| MY-h | keyboard_hook_callback.rs | injected key-up tracked as key-down | `keyboard_hook_callback::tests::the_daemons_own_injected_modifiers_pass_and_are_tracked` | caught |

The `Unnoticed` property (critique v3 M2): X51 fails it (config default, seed 8, as the developer saw); X4 does not (the developer's finding, kept as a
follow-up "property coverage of D10"); MY-w1 fails it. M2 holds with X51 as its named mutation.

## Acceptance

Spec acceptance (`spec.md`) as amended by rulings W1-W4, W2a, V3-I1, L3, L4; the cut's mapping (`cut.md` "Acceptance mapping"). No literal `rg`/`diff` check is named
in the acceptance; the ones the cut and CLAUDE.md imply are run and quoted in the Gate section (Q10 diff, no `log::`/`println!` in the callback file, no `Relaxed`).

| Criterion | Evidence | Status |
|---|---|---|
| R11 within the D14 rule of R12 (superseded by W1) | witness unchanged: `tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` `ok`; bench (H R11 - H R12) p50 = 0.6 us against P 0.6 us (rule: P + 0.2), and H R11 0.8 us against the 5.4 us D14 witness bound | verified |
| A row shows the witness reading after a foreground change | moot under W1 (witness reads at every swallowed key-up, R11) | verified (moot) |
| Every stuck-key and lone-key-up test and property of 2.3 stays green | `engine::scenarios` + `engine::properties`: `running 242 tests ... ok. 242 passed`; `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows` `ok`; 7 properties `ok` (the 6 of 2.3 and the new `Unnoticed` one) | verified |
| W2: R12, R1, R4 show the reads gone; a row for the resync on main | bench below (R12 4.7 -> 0.2 us, R4 4.7 -> 0.2 us, R1 gate, R18 4.5 us); `tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` `ok` | verified |
| W2a: R12m, R12m2 bounded; the trade in one line | bench below; R-F `ok` (zero reads with nothing held, one per held unswallowed modifier) | verified (R12m2 marginal, see bench) |
| Each hiding source triggers a resync (foreground change, lost-input reset, remapping switch, reinstall, refused event, watchdog gap, late callback) | S3-S8 scenarios and `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`, `a_refused_call_notes_hidden_input`, `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`, `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late`, `an_awake_stretch_of_17_ms_notes_hidden_input`, `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread`, `the_drain_counts_the_watchdogs_notes`: all `ok` (19-test run, `19 passed`); each has a mutation above | verified |
| A hidden modifier key-up followed by a binding press heals | R-D, R-E, R-C `ok` (red on `d5458d2`, green on HEAD) | verified |
| No stuck key and no lone key-up across the property modes of 2.3 | the 7 properties `ok`; RF-3 scenario; S9; R-A, R-B | verified |
| W2a pins: AutoHotkey trace, blocker trace, I1 scenario, read count, named mutations | R-A, R-B, R-C/R-D, R-F, with X26, X27, X28, X29 caught | verified |
| W2a: drain read sets and clears sided slots, clears a generic slot only when it reads up | R-G, X33, X17 caught | verified |
| V3-I1: nested decided event keeps master's reading | `tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` `ok`; X50 caught | verified |
| R16 meets a bound | L3 gate: H R16 p50 0.9 us <= H R12 p50 0.2 + 20 us (P R16 52.6 us); p99 reported below | verified |
| No other bench row regresses | bench table below, D10/D14/2.4 rules | verified |
| The 2.4 watchdog acceptance (fake clock) stays green | `hook::watchdog::` `running 79 tests ... 79 passed`; restored expectations `a_batch_read_at_a_tick_counts_all_its_events` `ok`, `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` `ok`, `the_first_raw_key_event_after_idle_wakes_the_watchdog` `ok`, `raw_input_before_the_hook_is_installed_is_read_and_not_counted` `ok` | verified |
| W4: no `[WARN]` from `hook_late` alone; injector stamps | `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` `ok`, `a_report_whose_only_count_is_hook_late_is_written_at_info` `ok`, `an_injected_events_own_stamp_is_not_its_age` `ok`; X41c, X40 caught | verified; M8 manual |
| D10 (lone key-up of a swallowed repeat with no record) | RF-3 scenario red on `9a9bdcf`, green on HEAD | verified |
| D14 leftovers (CHANGELOG, `timers.rs` docs) | Q10 diff shows `src/platform/timers.rs | 9`; `rg "timer thread" CHANGELOG.md` finds nothing per the scout, not re-run | doc, read by reviewer |
| CHANGELOG / README / config pages | not test-covered; `config::loader` README-example tests are part of the 862 | reviewer |
| Manual, not verified: M1-M4, M5-M7 (roadmap checks 44, 39, 47 as written for 2.4), M8 (`hook_late` stays 0 in 5 minutes of typing), M9 (StickyKeys incl. the latched step), M10 (AutoHotkey `LAlt::LCtrl`), M11 (PowerToys LWin to F13) | cut.md "Manual checks"; configs in `D:/tmp/ks-orch/p2-2.5-latency-trim/cfg/` | manual, not verified |

## Bench (tester section 4b, ruling M5 and L3)

Nine release runs in one session, interleaved P1 N1 H1 P2 N2 H2 P3 N3 H3, `KS_BENCH_LOAD=1` on all. P = `62c9fd8` (master plus bench), N = P with
`self.keys.sync_modifiers(env, Some(event.vk));` deleted (uncommitted, own worktree), H = `d853a59` (code identical to `786e61f`).
Each tree built in its own worktree with its own `CARGO_TARGET_DIR`; raw tables and logs in `D:/tmp/ks-orch/p2-2.5-latency-trim/bench/` (`P1.md` .. `H3.md`, `runs.log`, `agg.py`).
CPU load just before each run (%): P1 37, N1 14, H1 1, P2 15, N2 7, H2 2, P3 7, N3 51, H3 13. None reached 79 %, so no re-run rule applies.
Median of the three p50s, largest of the three p99s (us):

| Row | P p50 | N p50 | H p50 | P max p99 | N max p99 | H max p99 |
|---|---|---|---|---|---|---|
| R1 (mean of means) | 0.093 | 0.078 | 0.083 | 0.2 | 0.2 | 0.2 |
| R2 | 0.1 | 0.1 | 0.1 | 0.3 | 0.2 | 0.2 |
| R3 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.2 |
| R4 | 4.7 | 0.2 | 0.2 | 8.3 | 0.9 | 0.9 |
| R5 | 4.8 | 0.3 | 0.3 | 6.9 | 0.6 | 0.5 |
| R6/5 | 274.2 | 344.6 | 279.7 | 837.9 | 611.1 | 969.8 |
| R6/20 | 285.1 | 251.8 | 342.9 | 6282.7 | 840.1 | 498.1 |
| R6/50 | 273.2 | 392.6 | 288.3 | 569.2 | 559.2 | 674.9 |
| R6/200 | 248.9 | 356.7 | 423.5 | 618.8 | 735.9 | 859.0 |
| R7 | 10.4 | 10.3 | 10.2 | 52.5 | 23.1 | 18.6 |
| R8/normal | 6.5 | 6.4 | 7.0 | 211.3 | 237.0 | 219.2 |
| R8/highest | 10.0 | 7.7 | 8.2 | 254.8 | 235.7 | 290.4 |
| R8/normal-rt | 13.5 | 13.4 | 14.2 | 280.8 | 263.9 | 288.6 |
| R8/highest-rt | 17.3 | 14.8 | 15.5 | 350.9 | 311.5 | 376.2 |
| R9 | 4.8 | 0.2 | 0.2 | 9.0 | 1.2 | 0.3 |
| R10 | 5.0 | 0.4 | 0.4 | 8.6 | 0.5 | 0.5 |
| R11 | 5.3 | 0.8 | 0.8 | 8.8 | 1.2 | 1.2 |
| R12 | 4.7 | 0.2 | 0.2 | 6.3 | 0.2 | 0.2 |
| R12m | 4.7 | 0.2 | 0.8 | 10.2 | 0.2 | 1.7 |
| R12m2 | 4.7 | 0.2 | 1.4 | 8.2 | 0.2 | 2.2 |
| R13 | 10.3 | 10.4 | 10.3 | 32.4 | 29.5 | 53.0 |
| R14 | 4.7 | 0.2 | 0.2 | 120.3 | 3.9 | 6.1 |
| R14c | 4.7 | 0.2 | 0.2 | 133.6 | 4.8 | 6.2 |
| R15 | 3.8 | 3.8 | 3.8 | 21.0 | 5.6 | 24.6 |
| R16 | 52.6 | 0.9 | 0.9 | 170.9 | 4.4 | 7.5 |
| R17t | 0.001 | 0.001 | 0.001 | 0.0 | 0.0 | 0.0 |
| R17a | 0.552 | 0.546 | 0.545 | 1.0 | 1.0 | 1.2 |
| R18 | - | - | 4.5 | - | - | 5.8 |

Gates (cut D13 and rulings L3; H runs of R17a p50 = 0.545, R17t = 0.001):

| Row | Rule | Numbers | Result |
|---|---|---|---|
| R12 | H p50 <= N p50 + R17t + 0.1; H p99 <= 1.5 N p99 + 2 | 0.2 <= 0.301; 0.2 <= 2.3 | pass |
| R4 | H p50 <= N p50 + R17t + 0.1 | 0.2 <= 0.301 | pass |
| R1 | Delta = P mean - N mean = 0.015 (>= 0.010); H mean <= N mean + 0.5 Delta | 0.083 <= 0.0855 | pass (margin 2.5 ns; the three H means 0.083, 0.081, 0.084) |
| R12m | H p50 <= N p50 + R17a + R17t + 0.1 | 0.8 <= 0.846 | pass |
| R12m2 | H p50 <= N p50 + 2 R17a + R17t + 0.1 | 1.4 vs 1.391 | **formal excess of 9 ns.** The p50 column has a 100 ns grid (N's 0.2 stands for a mean of 0.19); in means, H R12m2 is 1.369 / 1.354 / 1.577 (loaded run) against N R12 mean 0.19 + 2 x 0.56 + 0.001 + 0.1 = 1.41, and with R17a taken over all nine runs (0.552) the p50 rule gives 1.405. Not a breach at the instrument's resolution; recorded for the reviewer. L4 accepted "two cost 1.1 to 1.4 us" |
| R11 witness term | H (R11 - R12) p50 <= P (R11 - R12) p50 + 0.2 | 0.6 <= 0.8; and 0.8 <= 5.4 (2.3 D14) | pass |
| R16 (gated by L3) | H p50 <= H R12 p50 + 20 | 0.9 <= 20.2; p99 largest of three 7.5 us (H3 only, the run before it at 13 % load; H1 1.4, H2 3.2) | pass |
| R1, R2, R3 (2.2 D10) | H p50 <= 1.25 P + 0.1 | 0.1 <= 0.225 | pass |
| R4, R5 (D10) | R4: H <= 1.25 P + 0.5, p99 <= 1.5 P p99 + 2; R5: H <= P | 0.2, 0.9; 0.3 <= 4.8 | pass |
| R6 (D10) | H p50 <= 1 ms and <= P p50 + 0.5 ms | 280, 343, 288, 424 us; each <= P + 500 us | pass |
| R7 (D10) | H p50 <= P + 20; p99 <= 1.5 P p99 + 50 | 10.2 <= 30.4; 18.6 <= 128.8 | pass |
| R8 | evidence in 2.2 | H within 1 us of P at p50 in all four cells | no regression |
| R9, R10, R13 (2.3) | not above P | R9 0.2, R10 0.4, R13 10.3 vs P 10.3 | pass |
| R14 (2.4) | H p50 <= R14c p50 + 0.3 | 0.2 <= 0.5; H p99 largest 6.1 against control 6.2 | pass |
| R15 | evidence | 3.8 on all trees | - |
| R18 | evidence; p50 <= 8 R17a + 0.5 | 4.5 <= 4.86 (H3: 4.6) | pass |
| R17t | stop rule (c) p50 <= 100 ns | 1 ns | pass |

Trade in one line, from these runs: an ordinary key costs 0.2 us against 4.7 us on master; one held modifier adds about 0.6 us (R12m 0.8) and two about 1.2 us (R12m2 1.4);
the witness at a swallowed key-up is unchanged (0.6 us over an ordinary key); the first key after idle (R16) costs 0.9 us against 52.6 us.
The bench is counted apart, never as a pass.

## Failures

None open. Failures met on the way, all closed:

- F1. X12 (`Engine::track_injected` passing `decided = true`) was not caught by any of the 859 tests; the cut's catcher tests `Resync::event` directly. Closed by `eb15d53`.
- F2. X41b (`continue` removed after the E5 note) was not caught by any test; `the_main_loop_checks_its_awake_stretch_before_every_wait` is a text scan of the call order only. Closed by `eb15d53`.
- F3. MY-b (`MainState::resync_if_due` no longer returns early while remapping is off) was not caught by any test. Closed by `d853a59`.
- F4. `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` flaked in 3 of about 14 full-suite runs on HEAD (twice inside my mutation runs, once in the last gate run) and in 1 of 8 on master. Pre-existing, untouched file, real-clock test of item 2.1; not a lane defect. Recommend the next lane that touches `core/executor` widen or fake it.

## Tests added by the tester (`Tests:` commits on `perf/latency-trim`, test code only, trailer `Co-Authored-By: Claude Sonnet 5.5`)

- `eb15d53` Tests: pin the injected event's resync wiring and the E5 loop-back (roadmap 2.5)
  - `src/engine/mod.rs`: `engine::tests::the_daemons_own_injected_events_do_not_bring_keyboard_input_back_to_the_drain` (red under X12, green on HEAD)
  - `src/hook/main_thread.rs`: `hook::main_thread::tests::a_long_stretch_sends_the_main_loop_back_to_the_drain_before_the_wait` (red under X41b, green on HEAD; a source scan in the style of its neighbour)
- `d853a59` Tests: pin that the drain reads nothing while remapping is off (roadmap 2.5)
  - `src/hook/main_thread.rs`: `hook::main_thread::tests::the_drain_reads_nothing_while_remapping_is_off` (red under MY-b, green on HEAD)

Each was shown red under its mutation and green without it, on the commit that holds it. `fmt` and `clippy -D warnings` pass with them; the lib run is `862 passed; 0 failed; 7 ignored`.

## Notes for the reviewer

- `47af1b1` is red by one assertion (declared; history not rewritten); `b106db9` returns the tree to commit 11's. The merge commit's tree and HEAD are green.
- R16's gate moved from "declared" (2.4 L2, still in `CLAUDE.md:72` and `.claude/agents/tester.md:101`) to gated (L3): the chore after the merge.
- Follow-ups already in `impl.md` stay valid: property coverage of D10 (the `Unnoticed` property passes under X4), Q11, M9's latched step, X23a is caught only by the callback test (the engine-level injected key-up checks pass because the key-down check repairs the flag).
- R1's gate margin is 2.5 ns and R12m2's p50 excess is 9 ns at a 100 ns resolution; both are measurement-limited, not defects.
- No test touches the production pipe name, a real `config.ron`, the registry for writing or live input; no ignored test was run; `swch`, the daemon and `cargo run` were never started.
- Scratch worktrees (`replay`, `mut`, `P`, `N`, `H`, `master`) and `target-wt` removed; `git worktree list` shows only the lane tree; `git status --short` in the lane tree is empty.
