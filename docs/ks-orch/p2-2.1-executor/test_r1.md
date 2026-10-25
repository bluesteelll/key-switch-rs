# Tester: p2-2.1-executor / roadmap 2.1 Action executor, KS-08 (round 1)

HEAD under test: `6a1db71` (developer). Tester added 3 `Tests:` commits on top; final HEAD `a0a8b82`, tree clean (`git status --porcelain` empty), `git worktree list` shows only the lane tree.
Parent for red-first replays: each test's own red commit (see table); master = `7d757a1`.

Verdict: RED: two executor COM tests fail whenever they do not run next to another live STA thread (F1); three mutations left the suite green and needed new tests (added, F2).

## Gate (HEAD `a0a8b82`; `6a1db71` gave the same result before the added tests)
- `cargo fmt --check`: OK
- `cargo clippy --all-targets --locked -- -D warnings`: OK (re-run after `touch` of the three crate roots: "Checking key-switch-rs ... Finished")
- `cargo test --locked --no-fail-fast` (default threads): 586 lib passed + 6 `ipc_roundtrip` passed = 592 passed, 0 failed, 5 ignored.
  - `running 591 tests` / `test result: ok. 586 passed; 0 failed; 5 ignored`
  - `running 6 tests` / `test result: ok. 6 passed; 0 failed; 0 ignored` (tests/ipc_roundtrip.rs)
  - swch and doc-tests: `running 0 tests` (no tests exist there; not a filter miss)
  - At `6a1db71`: `running 587 tests` / `582 passed; 0 failed; 5 ignored`.
- Ignored (all manual, none run): `core::executor::tests::the_win32_runner_opens_a_url` (new, M9), `data::condition::tests::foreground_context_names_the_focused_notepad`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`.
- `cargo build --release --locked`: OK (not run).
- NOT green outside the default run: `cargo test --locked --lib -- --test-threads=1` -> `running 591 tests` / `584 passed; 2 failed` and `--test-threads=2` fails in 2 of 3 runs (F1).
- Timing repeat, `cargo test --locked --lib -- core::executor hook::keyboard_hook` x5 on `a0a8b82`: `59 passed; 0 failed; 2 ignored` five times (0.51 s each). Same x5 on `6a1db71`: `56 passed` five times.

## Red-first replay
Method: scratch worktree `D:/tmp/ks-orch/p2-2.1-executor/wt/red`, own `CARGO_TARGET_DIR=.../target-wt/red`, one `git checkout --detach <sha>` per test, `--lib <path> -- --exact`. Every run printed `running 1 test`. Outputs: `D:/tmp/ks-orch/p2-2.1-executor/t1/red_<sha>.txt`. Worktree removed with `git worktree remove --force`, `git worktree prune`.

| Test | Red commit | Parent result | HEAD result | Note |
|---|---|---|---|---|
| `config::loader::tests::mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing` | `fbd6b59` | red (assert): `left: ["binding #0: block_original_combo: MaskOnly needs a Windows hotkey ... blocked as with true"]`, `right: []` | `1 passed` | L3; also red on master (no seam needed) |
| `config::loader::tests::an_empty_config_warning_is_returned_not_logged` | `47f2b88` | red (assert): `left: []`, `right: ["config contains no bindings"]` | `1 passed` | L5 |
| `logging::tests::a_second_init_starts_no_writer` | `7827ec8` | red (assert): `a second init must not start a writer` (`logging/mod.rs:238`) | `1 passed` | L8; seam `start_logging` from `89fcf5e` |
| `core::executor::tests::a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` | `a364b77` | red (assert): `left: None right: Some("open B")`, "did not run within 1 s" | `1 passed` | V2-I1 |
| `data::sequence::tests::a_window_step_never_waits_for_its_window` | `fd5993e` | red (assert): lists `sequence.rs:277/280/283 ShowWindow(target, SW_*)` | `1 passed` | X3 |
| `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` | `fcce19b` | red (assert): `keyboard_hook_callback.rs:219: action.execute();` | `1 passed` | X5 |
| `core::executor::tests::macros_never_sleep_or_block_in_a_wait` | `ba3122f` | red (assert): `src/data/sequence.rs:133: SequenceStep::Delay(ms) => thread::sleep(...)` | `1 passed` | X6 |
| `logging::tests::the_functions_on_the_hook_path_log_nothing` | `37b1847` | red (assert): `windows_actions.rs:278`, `system_function.rs:58`, `:77` | `1 passed` | X11 |
| all `core::executor::tests` + `runs::tests` | `cf3c9f6` | compile-red: E0433 `TargetWindow` x34, `Stopped` x17, E0425 `QUEUE_CAPACITY` x15, ... ; `cargo build` green | `core::executor`: `running 34 tests` (33 passed, 1 ignored) at `a0a8b82` | X1; carried by mutations M1-M12, M23-M27 |
| X7 set (`only_launch_open_url_and_sequence_leave_the_firing_thread`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`, `a_hold_runs_its_slow_action_on_the_executor`, `swch_off_stops_a_running_macro_within_100_ms`, `a_reload_stops_a_running_macro_within_100_ms`, `a_repeated_swch_on_keeps_the_running_macro`, `a_slow_action_without_an_executor_is_closed`) | `6edfa0f` | compile-red: E0433 `FastAction` x7, E0599 `attach_executor` x6, E0061 `carry_out` x4, E0599 `submit` x3 (22 errors); `cargo build` green | `hook::keyboard_hook::tests` `running 14`, all ok; `hook::keyboard_hook_callback::tests` `running 13`, 12 ok 1 ignored; `core::windows_actions` `running 2`, ok | carried by M5, M6, M7, M13, M19, M20 |
| X9 set (`the_longest_callback_is_kept_until_taken`, `a_slow_callback_alone_is_reported_and_a_fast_one_is_not`, `a_flush_reports_a_slow_callback`, `the_keyboard_callback_records_how_long_it_ran`) | `0ff3ce2` | compile-red: E0599 `longest_callback` x3, `record_callback` x3, E0609 x2, E0422 `Snapshot` x2, E0425 `SLOW_CALLBACK` | `logging::counters` `running 7` ok; `logging::writer` `running 10` ok | carried by M14, M15, M16 |

Each HEAD result was run singly with `--exact`, in default-thread mode, and printed `running 1 test` / `test result: ok. 1 passed`.

## Mutations (scratch worktree `wt/mut`, own target dir; reverted with `git checkout -- .`, `git diff --exit-code` exit 0 after each; worktree removed)
Test named before the run. Run logs: `t1/mut_a.txt` .. `mut_g.txt`.

| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| M1 | executor/runs.rs `poll` | `now >= deadline` -> `>` | `runs::tests::a_run_goes_up_to_its_first_delay_and_resumes_at_its_deadline` | caught (3 of 8 runs tests red) |
| M2 | executor/mod.rs `submit` | Disconnected-while-stopping no longer `Closed` | `a_submit_after_stop_is_closed_and_not_counted` | caught |
| M3 | executor/mod.rs `run_loop` | break after first pass (drop re-drain after a runner call) | `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` | caught |
| M4 | executor/mod.rs `Shared::live` | ignore the epoch | `delay_10000_stops_within_100_ms_of_an_epoch_change`, `a_job_queued_before_an_epoch_change_does_not_run` | caught (both) |
| M5 | keyboard_hook.rs `set_enabled` | no `cancel_actions` | `swch_off_stops_a_running_macro_within_100_ms` | caught |
| M6 | keyboard_hook.rs `update_bindings` | no `cancel_actions` | `a_reload_stops_a_running_macro_within_100_ms` | caught |
| M7 | windows_actions.rs `as_fast` | `OpenUrl` becomes a fast action | `only_launch_open_url_and_sequence_leave_the_firing_thread`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here` | caught (both) |
| M8 | executor/mod.rs `plan` | drop `!code_ran` | `the_delay_timer_is_kept_only_while_nothing_ran` | caught |
| M9 | executor/mod.rs `submit` | `Full` no longer bumps the counter | `a_full_queue_drops_the_job_and_counts_it` | caught |
| M10 | executor/mod.rs `timer_ms` | `div_ceil` -> floor | `timer_ms_rounds_up_and_clamps` | caught |
| M11 | executor/runs.rs `advance` | a panicking step no longer ends its run | `runs::tests::a_panicking_step_ends_only_its_macro` | caught |
| M12 | executor/runs.rs `poll` | drop `retain(live)` | `delay_10000_stops_within_100_ms_of_an_epoch_change` (+ swch/reload tests) | caught (my first filter named the wrong tests; full lib run: 3 red) |
| M13 | keyboard_hook.rs `set_enabled` | cancel also on `Some(false)` (repeated on/off) | `a_repeated_swch_on_keeps_the_running_macro` | caught |
| M14 | keyboard_hook_callback.rs | remove `counters::record_callback` | `the_keyboard_callback_records_how_long_it_ran` | caught |
| M15 | counters.rs `record_callback` | `fetch_max` -> `store` | `the_longest_callback_is_kept_until_taken` | caught |
| M16 | counters.rs `summary` | `<` -> `<=` at `SLOW_CALLBACK` | `a_slow_callback_alone_is_reported_and_a_fast_one_is_not` | caught |
| M17 | executor/mod.rs `Apartment::drop` | never `CoUninitialize` | `the_apartment_guard_uninitialises_com_on_drop` | red at `6a1db71`, but it is red WITHOUT the mutation when run alone (F1); proven right-reason only with the proposed F1 fix in scratch ("after the guard the thread has no apartment of its own") |
| M18 | executor/mod.rs `run_executor` | Died epilogue no longer drains and counts queued jobs | none existed | **NOT CAUGHT at `6a1db71`**; new test `a_job_still_queued_when_the_executor_dies_is_counted` (commit `c94acd5`) is red under M18 (`FAILED`, 0.20 s), green without |
| M19 | keyboard_hook.rs `submit` | no executor -> `Unavailable` instead of `Closed` | `a_slow_action_without_an_executor_is_closed` | caught |
| M20 | keyboard_hook_callback.rs `start_hold_timer` | Hold's decision not carried out | `a_hold_runs_its_slow_action_on_the_executor` | caught |
| M21 | counters.rs `bump_if_err` | `is_err` -> `is_ok` | `bump_if_err_counts_only_an_error` | **NOT CAUGHT** (the test sums one Ok and one Err, which an inverted check also satisfies); new test `bump_if_err_ignores_an_ok_result` (commit `2e51c2a`) red under M21, green without |
| M22 | executor/mod.rs `for_action` | `Launch` also reads the foreground window | none existed | **NOT CAUGHT**; new test `launch_and_open_url_carry_no_window_and_ask_for_none` (`2e51c2a`) red under M22, green without (checks something only where a foreground window exists; the owner's desktop has one) |
| M23 | executor/mod.rs `submit` | Disconnected no longer bumps `ExecutorUnavailable` | `an_executor_ended_by_wm_quit_counts_every_later_submit` | caught |
| M24 | executor/mod.rs `submit` | job always carries epoch 0 | `core::executor::tests` | caught (4 red: `delay_10000...`, `a_job_queued_before_an_epoch_change_does_not_run`, `a_delay_of_u64_max_waits_until_cancelled`, `a_cancelled_waiting_run_never_resumes`) |
| M25 | executor/mod.rs `stop_thread` | `stopping` never set | `stop_interrupts_a_delay_drops_the_queue_and_joins`, `a_submit_after_stop_is_closed_and_not_counted` | caught (both) |
| M26 | executor/runs.rs `next` | nearest deadline -> latest (`min` -> `max`) | none existed (all runs tests use equal delays) | **NOT CAUGHT** (whole `core::executor`: 32 passed); new pure test `runs::tests::the_nearest_deadline_of_several_waiting_runs_is_next` (commit `a0a8b82`) red under M26, green without |
| M27 | executor/mod.rs `submit` | `Full` blocks on `send` instead of dropping | `a_full_queue_drops_the_job_and_counts_it` | caught (red after 10 s: the blocking `send` waits on the gate) |

The first M15, M10 and M21/M22/M18 attempts were redone: M15 (`store` with `AcqRel`) and the first M10 (`.div`) did not compile and are not counted; M18 and M21/M22 re-runs at the first filters printed `running 0 tests` after I had pointed the scratch tree at the wrong commit, were noticed and redone at `2e51c2a`/`a0a8b82` with `running 1 test`.
Test code only was changed in the lane tree; no production file was touched.

## Acceptance
Spec `spec.md` and `rulings.md`; each test run at `a0a8b82` singly with `--exact`, `running 1 test`, `1 passed`.

| Criterion | Evidence | Status |
|---|---|---|
| `rg "\.execute\(\)" src/hook` finds nothing | Grep tool (no `rg` binary in this shell): "No matches found"; test `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` ok | verified |
| 100 Sequence runs create no threads | `core::executor::tests::a_hundred_sequence_runs_use_the_one_executor_thread` ok; non-test thread spawns under `src/core`, `src/data`: only `src/core/executor/mod.rs:290` (`thread::Builder` `action-executor`), test spawn at `mod.rs:1363`; `spawn_sequence` has no match in `src` | verified |
| `Delay(10000)` stops within 100 ms of an epoch change | `core::executor::tests::delay_10000_stops_within_100_ms_of_an_epoch_change` ok (0.01 s); `hook::keyboard_hook::tests::swch_off_stops_a_running_macro_within_100_ms` ok; `a_reload_stops_a_running_macro_within_100_ms` ok; M4, M5, M6, M12, M24 each turn them red | verified |
| 20 slow jobs, each enqueue under 5 ms | `core::executor::tests::twenty_slow_jobs_each_enqueue_in_under_5_ms` ok (0.50 s); `hook::keyboard_hook_callback::tests::carry_out_returns_in_under_5_ms_while_slow_actions_block` ok; five repeats clean; the cold-browser run itself is manual (M-list in `cut.md:476-490`) | verified (cold browser: manual, not verified) |
| A full queue drops the job and bumps the counter | `core::executor::tests::a_full_queue_drops_the_job_and_counts_it` ok; M9, M27 red | verified |
| R1: a `Delay` holds up no other job; macros interleave in order; cancelled waiting run never resumes; `Delay(u64::MAX)` | `a_delay_5000_macro_holds_up_no_other_job`, `two_macros_fired_together_both_run_in_their_own_order`, `a_cancelled_waiting_run_never_resumes` (both modules), `a_delay_of_u64_max_waits_until_cancelled`, `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms`, `timer_ms_rounds_up_and_clamps` all ok; nearest-deadline selection added (M26) | verified |
| No sleep or blocking wait in macro code | `macros_never_sleep_or_block_in_a_wait` ok (red at `ba3122f`) | verified |
| R2 `Window(...)` never waits for its window | `data::sequence::tests::a_window_step_never_waits_for_its_window` ok (red at `fd5993e`); no `ShowWindow(` in `src` | verified (a hung-window case: manual M11) |
| R3 executor death counted; closed during stop silent | `an_executor_ended_by_wm_quit_counts_every_later_submit`, `a_submit_after_stop_is_closed_and_not_counted` ok; queued-at-death drain now pinned by `a_job_still_queued_when_the_executor_dies_is_counted` (new) | verified |
| V2-I1 drain before every wait | `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` ok (red at `a364b77`, M3) | verified |
| V2-M3 timer kept only while nothing ran | `the_delay_timer_is_kept_only_while_nothing_ran` ok (M8) | verified |
| STA COM, balanced uninit | `the_executor_runs_jobs_in_a_single_threaded_apartment`, `the_apartment_guard_uninitialises_com_on_drop`: ok in the default full run, **FAIL when run alone or with `--test-threads=1`** (F1) | not reliably verified: RED |
| Fast actions stay in the callback, slow ones queue | `only_launch_open_url_and_sequence_leave_the_firing_thread`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here` ok (M7) | verified |
| M5 Hold follows the same split | `a_hold_runs_its_slow_action_on_the_executor` ok (M20); no hook installed, no input injected | verified |
| Longest callback recorded and reported | `the_longest_callback_is_kept_until_taken`, `a_slow_callback_alone_is_reported_and_a_fast_one_is_not`, `a_flush_reports_a_slow_callback`, `the_keyboard_callback_records_how_long_it_ran` ok (M14-M16) | verified |
| Hook path logs nothing (D11); only the banner prints | `logging::tests::the_functions_on_the_hook_path_log_nothing`, `only_the_startup_banner_prints_outside_src_bin` ok | verified |
| Sentinel on every `SendInput` | `rg "SendInput\("` equivalent via Grep: 6 sites in `src`, the same 6 as on master (`windows_actions.rs`, `sequence.rs` x3, `keyboard_hook_callback.rs`, `system_function.rs`); lane added none | verified |
| Leftovers L3, L5, L8 | the three red-first tests above | verified |
| Leftovers L1, L2, L10, L11 (doc, comment) | no harness | manual / review, not verified |
| Executor started before the hooks, stopped after they are removed, joined (bounded, 2 s) before `logging::flush` | `core/app.rs:199-209`, `:266-267`; no harness (KS-38) | manual, not verified |
| Cold-browser OpenUrl, Window(Minimize) vs a hung window, 20 x `[Key, Delay(50)]` timing, M12/M13 | `#[ignore = "manual: ..."]` or human lines in `cut.md`/`impl.md` | manual, not verified |

Safety review of the diff: no test binds `key-switch-rs.sock`, writes a real `config.ron`, installs a hook, calls `SendInput`, `ShellExecuteW`, or writes the registry (`git diff 7d757a1..HEAD -- src` grep of added lines: only comments). The one new ignored test is manual M9. `mdbook build` and link check were not re-run (not in this brief).

## Failures
### F1. COM tests depend on another STA thread being alive (RED)
- Tests: `core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment` (`mod.rs:1357`) and `core::executor::tests::the_apartment_guard_uninitialises_com_on_drop` (`mod.rs:1372`).
- Expected `Some(APTTYPE_STA)` (0); received `Some(APTTYPE(3))` = `APTTYPE_MAINSTA`.
- Reproduction (HEAD `6a1db71` and `a0a8b82`): `cargo test --locked --lib core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment -- --exact` fails 3 of 3; the guard test fails the same way alone; `cargo test --locked --lib core::executor -- --test-threads=1` -> `31 passed; 2 failed`; `cargo test --locked --lib -- --test-threads=1` -> `584 passed; 2 failed; 5 ignored`; `--test-threads=2` failed 2 of 3 runs. They pass in the default parallel run only because some other test's executor thread already holds the process's "main STA" while these two run.
- Cause: a thread that initialises an STA when the process has no live STA becomes the main STA, which `CoGetApartmentType` reports as `APTTYPE_MAINSTA`. This is a test defect; the production `Apartment` is correct (MAINSTA is an STA).
- Consequence: M17 (never uninitialise) was "caught" at `6a1db71` only by this unrelated failure when run alone.
- Verified fix in scratch (not committed; existing assertions are the developer's): accept `STA` or `MAINSTA` for `inside`/`kind` and require `!is_sta(after)`:
  `fn is_sta(kind: Option<APTTYPE>) -> bool { kind == Some(APTTYPE_STA) || kind == Some(APTTYPE_MAINSTA) }`, `use ...::APTTYPE_MAINSTA`. With it `core::executor -- --test-threads=1` gave `33 passed; 0 failed; 1 ignored`, and M17 then fails the guard test with `after the guard the thread has no apartment of its own` (right reason).

### F2. Three behaviours had no failing test (fixed by tester commits)
M18 (Died-epilogue drain of queued jobs), M21 (`bump_if_err` inverted), M22 (`for_action` reading the foreground for Launch), M26 (timer armed for the latest deadline) passed the whole suite at `6a1db71`. Tests now exist (below); each is red under its mutation and green on HEAD.

## Tests added by the tester (`Tests:` commits on `feat/action-executor`, test code only, trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`)
- `c94acd5` Tests: a job still queued when the executor dies is counted: `src/core/executor/mod.rs` `core::executor::tests::a_job_still_queued_when_the_executor_dies_is_counted` (queues a job without the wake, posts `WM_QUIT` to the executor thread; it contains a 200 ms `thread::sleep` that only waits for the executor to return to `GetMessageW`; a slower machine can only make it fail towards "job ran", never pass vacuously; 3 of 3 runs plus the x5 repeats green).
- `2e51c2a` Tests: `core::executor::tests::launch_and_open_url_carry_no_window_and_ask_for_none` and `logging::counters::tests::bump_if_err_ignores_an_ok_result`.
- `a0a8b82` Tests: `core::executor::runs::tests::the_nearest_deadline_of_several_waiting_runs_is_next`.
Each passed fmt and clippy before commit; the full suite on `a0a8b82` is the gate result above.

## Notes
- `rg` is not installed in the Bash tool; literal checks were run with the Grep tool (ripgrep) instead, same regex.
- `the_keyboard_callback_records_how_long_it_ran` asserts on the process-wide `COUNTERS` (pack fact 53): not flaky in 5 repeats, but it shares state with any other test that runs the callback.
- Residual race R-7, the unsplit `hold-timer` `thread::sleep`, the cold-browser run and `App::run` ordering stay manual/review items as declared in `cut.md`/`rulings.md`.
- Scratch dirs: `D:/tmp/ks-orch/p2-2.1-executor/{wt,target-wt,t1}` (worktrees removed, target and logs kept for the reviewer).
