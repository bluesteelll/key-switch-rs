# Tester: p2-2.4-watchdog / roadmap 2.4 Hook watchdog, KS-08 (HEAD 140b831 vs master fa20b30; developer's HEAD 4a90d44)

Verdict: RED: R14 (key path with the watchdog running) misses cut D11's bound by a wide margin (HEAD median p50 22.7 us against R12 4.7 us, bound R12 + 0.3 us; largest p99 65.9 us against bound 9.0 us), and impl.md open point 1 asks for a ruling that is not in rulings.md. Everything else is green: gate, red-first, 20 mutations (6 uncaught at first, each closed by a `Tests:` commit and re-proved), R1-R13 bench rows within their bounds.

HEAD is 140b831 = developer's 4a90d44 + 4 tester `Tests:` commits (test code only). Tree clean, `git worktree list` shows only the lane tree.

## Gate (HEAD 140b831)
- cargo fmt --check: OK (exit 0).
- cargo clippy --all-targets --locked -- -D warnings: OK (after `touch src/lib.rs src/main.rs src/bin/swch.rs`: "Checking key-switch-rs ... Finished `dev` profile"; exit 0).
- cargo test --locked --no-fail-fast (log `D:/tmp/ks-orch/p2-2.4-watchdog/gate_test_final.log`):
  - lib: `running 759 tests` / `test result: ok. 752 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 3.55s`
  - swch bin: `running 0 tests` (no tests there; normal)
  - `tests/ipc_roundtrip.rs`: `running 7 tests` / `test result: ok. 7 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests`
  - On 4a90d44 (before my tests, `gate_test.log`): `running 753 tests`, 746 passed, 7 ignored; ipc_roundtrip 7 passed.
- cargo build --release --locked: OK (`Finished release profile`).
- Ignored (7, not counted as passes): `bench::latency_bench` (bench: run in the bench section), and 6 `manual:` tests: `core::executor::tests::the_win32_runner_opens_a_url`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it`, `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. None were run (`--ignored` was used only for the bench, by its documented command).
- Flake check: `cargo test --locked --lib -- hook::watchdog::tests logging::panic` x5 (the real-clock `synthetic_raw_events_*` and the child-process panic test): 5 times `21 passed; 0 failed`, 3.01 s each.
- Safety audit of the diff: `rg "SendInput|keybd_event|mouse_event|KEYBDINPUT|INPUT_KEYBOARD" src/hook/watchdog src/hook/health.rs src/hook/installer.rs src/hook/debug_hold.rs`: no match (rc 1). `rg "SetWindowsHookExW\(|UnhookWindowsHookEx\(|SetWinEventHook\(|UnhookWinEvent\(|RegisterRawInputDevices\(" src tests`: six hits, all in `src/hook/installer.rs:91,101,110,127,141,156` (the `#[cfg(not(test))] mod win32`). No `Relaxed|SeqCst|static mut` in the new modules. The diff adds no non-ASCII, no print macro, no `config.ron` or `PIPE_NAME` use (`git diff master..HEAD`); the production pipe name occurs only in pre-existing message-format tests (`src/ipc.rs:319`, `src/core/app.rs:494,509`), which bind nothing. Every new `unsafe {` has a SAFETY comment (script over the 10 touched files; the only unmatched lines are `unsafe extern "system" fn` declarations).

## Red-first replay
Method: scratch worktree `D:/tmp/ks-orch/p2-2.4-watchdog/wt/parent`, `CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.4-watchdog/target-wt/parent`, checked out at each red commit (the green commit's parent). All red commits except the first fail to compile on their parent because the API under test does not exist (the cut and pack declare it); those are `compile-red` and rest on the mutations below. HEAD column: `... ok` line in `gate_test_final.log` for every named test (57 names checked by script, 0 missing; 16 decision rows all ok).

| Test(s) | Parent | HEAD | Note |
|---|---|---|---|
| `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests` | red (assertion) at 29060a1: `running 1 test` ... `panicked at src\hook\installer.rs:181:9: src/hook/installer.rs has no #[cfg(not(test))] block mod win32 { / 7 installer calls a test build compiles: src/hook/keyboard_hook.rs:153: SetWindowsHookExW( ...` / `test result: FAILED. 0 passed; 1 failed` | ok | S4; the only behaviour-red |
| `main_thread::a_refused_reentry_is_counted_in_main_state_refused`, `counters::counter_names_are_snake_case` | compile-red 62d79da: `E0599 no method named get ... Counters`, `no variant ... MainStateRefused` | ok | not mutated by me: see the Notes |
| `keyboard_hook_callback::the_keyboard_callback_counts_every_call_also_while_remapping_is_off` | compile-red 6c18929: `E0599 no method named health ... &'static KeyboardHook` | ok | mutation M13 |
| 4 reinstall scenarios in `engine::scenarios` | compile-red bdf1480: `E0599 no method named hook_reinstalled ... Scenario` (x4) | ok | developer's `HookReinstalled => {}` mutation (not repeated by me) |
| 5 `keyboard_hook::tests` + 7 `main_thread::tests` reinstall tests | compile-red a8aa0f7: `E0425 reinstall_if_silent` (x4), `E0432 ReinstallOutcome, ReinstallRequest / OldHook, Reinstalled`, `E0599 reinstall_keyboard_hook_with / request_reinstall` | ok | mutations M3, M4, M21, M22 |
| 15 `watchdog::decision::tests` rows | compile-red 3505484: `E0425 Watchdog, SILENCE, TICK, MAX_BACKOFF_SHIFT`, `E0433 Verdict` | ok (16 rows with mine) | mutations M1, M2, M14 |
| `watchdog::tests` (thread, loop, scans) + `core::app` order scan | compile-red 87ea196: `E0405 RequestReinstall, TickTimer`, `E0425 qualifies`, `E0422 Context`, `E0433 HookWatchdog, Ticker` | ok | mutations M5-M7, M12 |
| `health::tests`, `ipc_server::tests::dispatch_status_appends_the_hook_health`, `ipc_roundtrip` | compile-red 2c624d1: `E0432 HookHealth` (lib and `ipc_roundtrip`), `E0560 DaemonStatus has no field hook`, `E0599 snapshot/record_reinstall/record_failed_attempt/hook_answered` | ok | mutations M10, M18, M19 |
| 5 `logging::panic::tests` | compile-red 96a43dc: `E0425 panic_text, budget_end, report_on, install_with, REPORT_BUDGET` | ok | mutation M17 |
| `debug_hold::tests` (2), `the_debug_hold_is_compiled_only_into_debug_builds` | compile-red 58c4370: `E0425 parse, should_hold` | ok | mutations M15, M16 |

Worktree `wt/parent` removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs`.

## Mutations
Scratch worktree `wt/mut` (detached, its own `CARGO_TARGET_DIR`), one mutation at a time, reverted with `git checkout -- .`, each followed by `git diff --exit-code` clean (my script printed `REVERTED diff rc 0` every time); worktree removed. The expected catcher was named before running. "NOT CAUGHT" rows were closed with a `Tests:` commit and re-run on that commit.

| # | File | Mutation | Expected catcher | Result |
|---|---|---|---|---|
| M1 | watchdog/decision.rs | `now >= quiet_since + window` -> `>` | decision rows at the boundary | caught: `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops` (`[1250, 3500]`), `raw_events_before_an_answer_are_no_evidence_after_it` |
| M2 | decision.rs | `MIN_EVIDENCE` 2 -> 1 | `a_single_raw_event_without_a_callback_asks_for_nothing` | caught (decision.rs:502) |
| M3 | main_thread.rs | alive check `callbacks() != request.seen` disabled | `a_hook_that_answered_is_kept_without_a_replace` | caught (also `a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing`; the SYSTEM installer panicked under test, so no real hook was touched) |
| M4 | main_thread.rs | engine reset after a reinstall skipped (`with_main(hook_reinstalled)` replaced by a constant) | `a_reinstall_resets_the_engine_and_keeps_the_macros` | caught (main_thread.rs:2036) |
| M5 | watchdog/mod.rs | desktop check `Ok(true)` -> `Ok(_)` (send even when input is on another desktop) | a Ticker-level test | NOT CAUGHT on 4a90d44 (28 passed). Closed by `065c824` `a_request_is_not_sent_while_input_is_on_another_desktop`; re-run: caught (`mod.rs:1047 no request while input is on another desktop`) |
| M6 | watchdog/mod.rs | `health.hook_answered()` removed (ruling M3) | `a_failed_reinstall_is_no_longer_reported_once_the_hook_answers` | caught (mod.rs:986) |
| M7 | watchdog/mod.rs | refused request (`Err` of `request`) no longer calls `not_sent`; the watchdog stays outstanding forever | Ticker-level test | NOT CAUGHT on 4a90d44 (28 passed). Closed by `065c824` `a_request_the_main_threads_queue_refused_is_asked_again`; re-run: caught (`both refusals were used`) |
| M8 | keyboard_hook.rs | `install_with` no longer sets `hook_installed` (the I3 gate's input) | any | NOT CAUGHT on 4a90d44 (530 passed of 533 in hook/logging/engine/core). Closed by `ec25787` `the_install_marks_the_hook_installed_and_the_removal_clears_it`; re-run: caught (keyboard_hook.rs:794) |
| M9 | keyboard_hook.rs | `uninstall_with` no longer clears `hook_installed` | any | NOT CAUGHT; same commit; re-run: caught (keyboard_hook.rs:799) |
| M10 | ipc_server.rs | status reply drops the `; {hook}` text | `dispatch_status_appends_the_hook_health` | caught (also `dispatch_switches_remapping_and_status_reports_the_flag`) |
| M12 | core/app.rs | watchdog stopped before `hook.uninstall()` (I3 order) | `the_hook_watchdog_starts_before_the_hooks_and_stops_after_them` | caught (`assertion failed: stop > position("hook.uninstall()?")`) |
| M13 | keyboard_hook_callback.rs | `callback_seen()` removed from the callback | `the_keyboard_callback_counts_every_call_also_while_remapping_is_off` | caught (callback.rs:891) |
| M14 | decision.rs | a `Failed` answer no longer counts as an attempt (no backoff after a refused reinstall) | a decision row | NOT CAUGHT on 4a90d44 (30 passed). Closed by `7f10ecc` `failed_reinstalls_are_spaced_out_like_reinstalls`; re-run: caught (16 requests every 1250 ms instead of 4) |
| M15 | debug_hold.rs | `should_hold` also holds on key-up | `only_a_key_down_of_the_armed_key_holds` | caught (debug_hold.rs:97) |
| M16 | keyboard_hook_callback.rs | `#[cfg(debug_assertions)]` removed from the debug-hold call | `the_debug_hold_is_compiled_only_into_debug_builds` | caught (callback.rs:1427) |
| M17 | logging/panic.rs | `budget_end`: `now < end + EPISODE` -> `now > end + EPISODE` | `a_nested_panic_on_one_thread_shares_the_budget` | caught (panic.rs:210) |
| M18 | health.rs | Display: timer-failing outranks reinstall-failed | any | NOT CAUGHT on 4a90d44 (8 passed). Closed by `140b831` `the_hook_health_names_the_first_problem_in_its_precedence`; re-run: caught (`a failed reinstall outranks a failing timer`) |
| M19 | health.rs | `record_reinstall` no longer clears `last_attempt_failed` | `a_failed_attempt_reads_reinstall_failed_until_the_next_reinstall` | caught (health.rs:362) |
| M21 | main_thread.rs | failed reinstall does not `record_failed_attempt` | `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing` | caught (main_thread.rs:2073) |
| M22 | main_thread.rs | reinstall not recorded (`record_reinstall` removed) | `a_reinstall_resets_the_engine_and_keeps_the_macros` | caught (`swch status counts it`, main_thread.rs:2030) |

(Numbering has gaps: M11, M20 were dropped before running; 20 mutations ran, 6 rows were NOT CAUGHT at first: M5, M7, M8, M9, M14, M18.) Not repeated by me, taken from the developer (`impl.md`): the S3 mutations (unhook first, handle nulled before install, no null guard, old hook left installed), `HookReinstalled => {}`, arming only after a tick, no timeout fallback, the heartbeat after the gate, the `MainStateRefused` bump. I read the S3 tests (the `FakeInstaller` records `keyboard_counts`, asserted `[2, 1]`) but did not re-mutate them.

## Tests added by the tester (test code only, committed on feat/hook-watchdog)
- `065c824` Tests: the watchdog sends no request off its desktop and asks again after a refusal: `src/hook/watchdog/mod.rs`: `a_request_is_not_sent_while_input_is_on_another_desktop`, `a_request_the_main_threads_queue_refused_is_asked_again` (the `Recorder` helper got a `refusals` counter).
- `ec25787` Tests: the install and the removal set the hook-installed flag the watchdog reads: `src/hook/keyboard_hook.rs`: `the_install_marks_the_hook_installed_and_the_removal_clears_it`, `a_refused_install_leaves_the_hook_unmarked`.
- `7f10ecc` Tests: a failed reinstall doubles the next silence like a reinstall does: `src/hook/watchdog/decision.rs`: `failed_reinstalls_are_spaced_out_like_reinstalls`.
- `140b831` Tests: the hook health names its first problem in the documented precedence: `src/hook/health.rs`: `the_hook_health_names_the_first_problem_in_its_precedence`.

All four end with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`; fmt and clippy clean; the full gate above is on 140b831.

## Latency bench (tester.md 4b)
Method: scratch worktrees at fa20b30 (master) and 140b831 (HEAD), own `CARGO_TARGET_DIR` each (`target-wt/bench-master`, `target-wt/bench-head`), `KS_BENCH_OUT=<file> KS_BENCH_LABEL=<..> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`, 3 runs each, interleaved (master-1, head-1, master-2, ...). All six: `running 1 test`, `test result: ok. 1 passed`. No `KS_BENCH_LOAD` (the lane does not touch the main loop, thread priority or the executor pump; R8 not taken). Raw tables: `D:/tmp/ks-orch/p2-2.4-watchdog/bench-tester-{master,head}-{1,2,3}.md`. Median of the three p50s, largest p99, in us. I did not check the session type (the developer's runs 1-3 were under Remote Desktop); my R12 p50 4.7 us and R11 5.3 us match the 2.3 witness (5.4 us), so these runs are quiet.

| Row | master p50 / max p99 | HEAD p50 / max p99 | Bound (D10 / D14 / D11) | Result |
|---|---|---|---|---|
| R1 | 0.1 / 0.2 | 0.1 / 0.2 | p50 <= 1.25 m + 0.1 = 0.225 | ok |
| R2 | 0.2 / 0.3 | 0.2 / 0.3 | <= 0.350 | ok |
| R3 | 0.1 / 0.1 | 0.1 / 0.1 | <= 0.225 | ok |
| R4 | 4.8 / 8.8 | 4.8 / 6.7 | p50 <= 6.5, p99 <= 15.2 | ok |
| R5 | 4.8 / 8.5 | 4.8 / 6.2 | p50 <= master | ok |
| R6/5, /20, /50, /200 | 310 / 345 / 352 / 366 | 266 / 308 / 281 / 309 | p50 <= 1 ms and <= m + 0.5 ms | ok (all four) |
| R7 | 10.5 / 28.7 | 10.7 / 30.8 | p50 <= m + 20, p99 <= 1.5 m + 50 | ok |
| R9 | 4.8 / 6.6 | 4.7 / 6.4 | p50 <= 1.05 m + 0.1 = 5.14, p99 <= 11.9 | ok |
| R10 | 5.0 / 6.7 | 5.0 / 11.3 | <= 5.35, p99 <= 12.05 | ok |
| R11 | 5.3 / 28.6 | 5.3 / 9.1 | <= 5.665, p99 <= 44.9 (witness 5.4) | ok |
| R12 | 4.7 / 9.5 | 4.7 / 7.0 | <= 5.035, p99 <= 16.25 | ok |
| R13 | 10.5 / 23.9 | 10.7 / 39.5 | <= HEAD R7 + 20 | ok |
| **R14** | - | **22.7 / 65.9** (p50s 22.7, 21.5, 23.1; p99s 62.2, 53.7, 65.9) | **p50 <= HEAD R12 + 0.3 = 5.00; p99 <= R12 p99 + 2 = 9.00** | **BREACH, about 4.5x at p50** |
| R14/settled | - | 5.0 / 153.5 (p50s 5.0, 4.9, 5.0) | evidence row, no bound | p50 within 0.3 us of R12; p99 noisy (73.7, 104.6, 153.5) |
| R15 | - | 3.9 / 6.4 | evidence (the rejected design) | 3.9 us per key on main |

Reading: the heartbeat's own cost is invisible (R4, R9-R12 equal master within noise). R14 times the callback while the watchdog thread handles a raw event posted just before the call; it costs about 18 us more at the median. R14/settled (the watchdog finished before the call) shows the gap is the two threads colliding, not the heartbeat. The cut's "justified extra work" for R14 is "the watchdog thread on another core" under a 300 ns bound; the lane exceeds that bound by 18 us and the cut does not justify it. impl.md open point 1 offers (a) accept, on the argument that real WM_INPUT follows the hook call of the same key and the next key is milliseconds away, (b) batch raw input at ticks, (c) raw input on main (R15, 3.9 us per key on main). rulings.md has no ruling on it (its Latency section only asks the results analyst to state the R14 cost). Per tester rule a breach the cut does not justify is RED. If the orchestrator or owner rules (a), this is the only red and it reduces to a documented, accepted R14.

## Acceptance
| Criterion (spec / roadmap) | Evidence (all `ok` on HEAD unless noted) | Status |
|---|---|---|
| Silent callback > 1 s gives exactly one reinstall | `watchdog::decision::tests::key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall`; `a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more`; thread level `watchdog::tests::synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`; loop level `an_early_timer_signal_does_not_lose_the_tick`, `a_timer_that_refuses_once_is_set_again_at_the_next_pass`. Mutations M1, M2 caught | verified |
| A callback that keeps up gives none | `a_callback_that_keeps_up_asks_for_nothing`, `raw_events_of_callbacks_seen_at_the_last_tick_are_explained`, `evidence_is_checked_before_this_intervals_raw_events_are_added`, `synthetic_raw_events_with_a_live_heartbeat_send_no_request` | verified |
| Idle gives none | `idle_asks_for_nothing_and_stops_ticking` (600 s fake clock, one idle verdict, no later tick), `a_single_raw_event_without_a_callback_asks_for_nothing`, `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops` | verified |
| A busy main thread gives no reinstall loop | `a_busy_main_thread_gives_no_reinstall_loop`, `reinstalls_that_bring_no_callback_back_are_spaced_out` (8 requests over 250 s), `failed_reinstalls_are_spaced_out_like_reinstalls` (mine), `main_thread::tests::a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing`, `a_hook_that_answered_is_kept_without_a_replace` | verified |
| Debug hold 1.5 s gives `reinstalls: 1` within 2 s | the live path is manual M1 (cut.md "Manual checks"); below the hook: `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`, `a_reinstall_resets_the_engine_and_keeps_the_macros`, `the_debug_hold_is_compiled_only_into_debug_builds` | manual, not verified (needs the live daemon; that Windows removes a hook after a 1.5 s callback is an unverified OS premise, Q5-Q8) |
| 10 min idle: no injected input | scan `watchdog::tests::the_watchdog_injects_no_input` ok; literal grep above (no match, rc 1); `idle_asks_for_nothing_and_stops_ticking`; manual M2 for the live ten minutes | scan + grep verified; live part manual, not verified |
| `swch status` shows HookHealth; ipc_roundtrip covers it | `tests/ipc_roundtrip.rs::status_reports_the_hook_health_over_the_pipe` and `::status_off_on_status_round_trip_without_a_hook` (7 passed), `ipc_server::tests::dispatch_status_appends_the_hook_health`, `health::tests::*` (9 incl. mine); mutations M10, M18, M19 | verified |
| Latency: no unjustified regression on the key path with the watchdog active | bench table above | R1-R13 verified; **R14 BREACH, open for a ruling** |
| Reinstall never double-installs, never leaves none (S3) | `keyboard_hook::tests::{a_reinstall_installs_the_new_hook_before_it_removes_the_old_one, a_failed_reinstall_keeps_the_old_hook, a_reinstall_of_a_hook_that_is_not_installed_installs_nothing, a_reinstall_reports_an_old_hook_that_windows_removed_as_gone, the_daemons_reinstall_of_a_hook_never_installed_makes_no_call}`, `main_thread::tests::a_reinstall_that_finds_no_hook_resets_nothing`, `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing`; M21, M22 | verified (fake installer; the S3 mutations are the developer's) |
| No test can install a hook (S1-S4) | `installer::tests::the_hook_installers_are_compiled_out_of_the_tests` (red on 29060a1 by assertion, green on HEAD), `the_system_installer_panics_at_every_call_under_test`; the grep above; M3 incidentally showed the SYSTEM installer panicking under test | verified |
| No key data kept or logged | `watchdog::tests::no_key_data_leaves_the_raw_input_read`; `Watchdog` fields are counts and instants (decision.rs:77-99) | verified (scan) |
| Re-entry refusal counter | `main_thread::tests::a_refused_reentry_is_counted_in_main_state_refused` | verified (no mutation of mine; the bump is one line at main_thread.rs:220) |
| Panic hook with a stated bound | 5 `logging::panic::tests` (incl. the child-process test), M17; 5 reruns green | verified |
| Debug flag absent from release | `the_debug_hold_is_compiled_only_into_debug_builds` (M16), `debug_hold::tests` x2 (M15), `cargo build --release --locked` ok, the release test harness compiled for the bench | verified |
| I3 order: watchdog starts before `install`, stops after `uninstall` and the control window | `core::app::tests::the_hook_watchdog_starts_before_the_hooks_and_stops_after_them` (M12); the raw-input gate: `raw_input_before_the_hook_is_installed_is_no_evidence`, and mine `the_install_marks_the_hook_installed_and_the_removal_clears_it` (M8, M9) | verified (the order test is a source scan, not a runtime check) |
| 2.3 leftovers (P1, P2, P3, `invariant:` expect) | commits 1aad2b1, 53b2499, 4f2d406, fcfd66e; the `MovingQuery` test is green | verified by the green gate and diff; P1's re-check not re-mutated by me |
| KS-08 end to end (cut.md manual checks M1-M8) | M1 hold + `swch status`; M2 ten idle minutes; M3 five minutes of typing incl. an elevated app; M4 Win+L; M5 UAC and Ctrl+Alt+Del; M6 AutoHotkey `CapsLock::return`; M7 `swch status` / `swch open`; M8 `swch exit` | manual, not verified |

## Failures
### F1. R14 latency breach (RED)
Row R14 (`src/bench.rs`, `r14_ordinary_key_with_the_watchdog`): HEAD median p50 22.7 us (22.7, 21.5, 23.1), p99 62.2 / 53.7 / 65.9 us. Bound (cut D11): p50 <= R12 + 300 ns = 5.0 us, p99 <= R12 p99 + 2 us = 9.0 us. R12 on the same HEAD: 4.7 us / 7.0 us. The developer saw the same gap (22.2-22.8 us) and left it as open point 1. Suspected cause: not the heartbeat (R14/settled 5.0 us) but the watchdog thread's wake-up and WM_INPUT work colliding with the callback at the instant of the call (a property of the bench's construction, post just before the call, and of the design, raw input on a second thread). Whether real typing hits it is not measurable without a live hook (manual M6 also decides Q6). Decision needed from the orchestrator or owner: accept (a), batch at ticks (b), or raw input on main (c, +3.9 us per key on main). Not a defect I can fix in test code.

### Closed during this round (not open)
The gaps behind M5/M7, M8/M9, M14 and M18 were test gaps in correct code; each is closed by one of the four `Tests:` commits and proved red under the mutation, green without it.

## Notes
- Worktrees: `wt/parent`, `wt/mut`, `wt/bench-master`, `wt/bench-head` all removed with `git worktree remove`; `git worktree list` shows only the lane tree. Target dirs remain under `D:/tmp/ks-orch/p2-2.4-watchdog/target-wt/` (outside the repo). The lane tree is clean at 140b831.
- Logs: `gate_test.log`, `gate_test_final.log`, `bench_run.sh`, `bench-tester-*.{md,log}` in the lane directory.
- The `MainStateRefused` bump and the 4 reinstall scenarios were not mutated by me (time); they are carried by the developer's mutations in impl.md.
- Noticed, not verified further: `WatchdogTimerNotSet` doc (counters.rs:51-53) says the retry comes "at its next wake, which the next raw key event brings", while `Ticker::wake_by` retries at the tick through the wait timeout (doc wording for the reviewer); impl.md open point 3 (a start that times out after 5 s while the thread then reports ready).
- The 28 developer commits include red-only commits that fail to compile by design; every one is followed by its green commit.
