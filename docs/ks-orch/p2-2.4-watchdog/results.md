# Results analysis: p2-2.4-watchdog / roadmap 2.4 Hook watchdog (KS-08)

Branch `feat/hook-watchdog` @ `17ca0de` (62 commits on `master` `fa20b30`; merge-base = `master`, tree clean, one worktree).

## VERDICT: ACCEPT

**Summary.** Every acceptance line of roadmap 2.4 and of the lane spec is backed by a named test. The two lines that need the live daemon (the 1.5 s debug hold, 10 minutes of idle time) also have owner manual checks. Every ruling in `rulings.md` (S1-S4, I1, I3, M1-M7, Q1-Q8, L1, V3-*, L2) is carried out in the tree or in the lane files. I re-ran the gate and it is green: 801 lib + 7 integration tests passed, 7 ignored, all pre-existing `manual:`/`bench:` reasons. The last tester round (r3) is GREEN, the last review (r3) is APPROVED with 0/0/0, and both CONFIRMED triage items are closed by rulings L1/L2. The latency rule holds. The key path costs one atomic RMW. In steady typing R14 equals its control R14c. The one wake per burst (R16, about 50 µs at the worst order) is a declared row by ruling L2, and manual check M6 settles whether a real key ever pays it.

## Quality gate (own run, `D:/key-switch-rs/key-switch-rs` @ `17ca0de`)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0, no output) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (after `touch src/lib.rs`: `Checking key-switch-rs ... Finished`, exit 0; log `D:/tmp/ks-orch/p2-2.4-watchdog/analyst/clippy.log`) |
| `cargo test --locked --no-fail-fast` | lib `running 808 tests` -> `801 passed; 0 failed; 7 ignored` (3.91 s); `swch` bin 0; `tests/ipc_roundtrip.rs` `7 passed`; doc-tests 0; exit 0. Log `D:/tmp/ks-orch/p2-2.4-watchdog/analyst-test.txt` (808 `... ok`, 7 `... ignored`) |
| Test count vs `master` | `git grep '#[test]'`: master 683, branch 806. No test fn removed in the diff (grep of removed `fn` lines = 0). The only edits to existing tests extend the expected strings with the new status suffix and counter names |
| `cargo test --locked -- --list --ignored` (listed, never run) | 7, identical to master: `bench::latency_bench` (`bench:`), `core::executor::...the_win32_runner_opens_a_url`, `hook::ipc_server::...hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::...the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::...input_is_on_this_desktop_while_a_human_uses_it`, `platform::foreground::...the_foreground_cache_names_the_focused_notepad`, `system::registry::...live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (all `manual: ...`). No new `#[ignore]` |
| `cargo check --release --all-targets --locked` | OK. A release build with no `debug_assertions`, the bench's test harness included, compiles with the debug hold compiled out |
| `cargo doc --no-deps` | 3 warnings, all pre-existing and in files the lane does not touch: `src/data/vk_name.rs:4` (`<raw>`) and two private-link warnings for `ConfigError::Missing` in the config loader |
| `mdbook build book -d D:/tmp/ks-orch/p2-2.4-watchdog/analyst/book` | built (only the known mdbook-mermaid version warning) |
| Trailers | 62/62 commits end with `Co-Authored-By: Claude (Opus\|Sonnet) 5.5`. The 9 tester `Tests:` commits are Sonnet, the rest Opus. No merges on the branch |

## Acceptance criteria

| # | Criterion (roadmap 2.4 / spec) | Evidence | Status |
|---|---|---|---|
| 1 | A pure decision, with counts and instants in and reinstall-or-not out, pinned with a fake clock: silent callback for more than 1 s gives exactly one reinstall | `hook::watchdog::decision::tests::key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall` (`decision.rs:383`: 1 request in [1000,1500] ms, `seen` 0, `silent_for >= 1 s`), `a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more` (`:398`). `decision.rs` is `#![forbid(unsafe_code)]` and has no clock, thread or Win32 call | PASS |
| 1b | A callback that keeps up gives none | `a_callback_that_keeps_up_asks_for_nothing` (`:415`: callback before, after and across the tick), `long_drift_with_a_live_hook_asks_for_nothing` (`:828`, 10 min), `a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it` (`:699`, the critic's 10 000-event trace, red at `ea8f19d`) | PASS |
| 1c | Idle gives none | `idle_asks_for_nothing_and_stops_ticking` (`:452`, 600 s, one idle tick <= 2250 ms, no tick after it); loop `a_single_batched_raw_event_stops_ticking_after_two_seconds` (`mod.rs:1704`: nothing armed, no timeout); `next_wait(.., false, None, ..) == (QS_ALLINPUT, INFINITE)` (`mod.rs:2357`) | PASS |
| 1d | A busy main thread gives no reinstall loop | `a_busy_main_thread_gives_no_reinstall_loop` (`:467`), `reinstalls_that_bring_no_callback_back_are_spaced_out` (`:499`, 8 requests at 1-64 s spacing), `failed_reinstalls_are_spaced_out_like_reinstalls` (`:518`, tester, closes mutation M14), loop `a_busy_main_thread_gives_one_request_with_batched_reads` (`mod.rs:1724`), main side `main_thread::tests::a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing` (`main_thread.rs:1908`). The rule is stated in `decision.rs:29-57` | PASS |
| 2 | Debug hold of 1.5 s gives `reinstalls: 1` within 2 s | Live part: manual M1 (`impl.md` "Manual checks", re-run on the batched build per `fix_r1.md`), with the M7 precondition. Below the hook: `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request` (`mod.rs:2611`, real thread and clock, 1 request 1-2 s after the first raw event); `a_reinstall_resets_the_engine_and_keeps_the_macros` (`main_thread.rs:1995`: status counts 1 with a time, the log lines, the reset); the S3 fake-installer tests (row 6); `debug_hold::tests::only_a_key_down_of_the_armed_key_holds` (`debug_hold.rs:92`) | PASS (automated below the hook + manual M1) |
| 2b | The debug flag cannot exist in a release build (spec; ruling Q3) | `hook::keyboard_hook_callback::tests::the_debug_hold_is_compiled_only_into_debug_builds` (`keyboard_hook_callback.rs:1393`: `#[cfg(debug_assertions)]` on the module and on every call site in the callback and in `App::run`). My release `cargo check` compiles without the module | PASS |
| 3 | Ten minutes of idle time produce no injected input | Scan `hook::watchdog::tests::the_watchdog_injects_no_input` (`mod.rs:2788`: no `SendInput`/`keybd_event`/`mouse_event`/`KEYBDINPUT`/`INPUT_KEYBOARD` in the watchdog, decision, health or installer, nor in the four reinstall functions); the idle rows of 1c; manual M2 (amended in `fix_r1.md`: `powercfg /requests`, display off, stop line `W <= 100`) | PASS (scan + manual M2) |
| 4 | `swch status` shows HookHealth; `tests/ipc_roundtrip.rs` covers it against a stand-in daemon; the reply prefix is compatible | `tests/ipc_roundtrip.rs::status_reports_the_hook_health_over_the_pipe` (`:148`, `starts_with("running (2 bindings live")` plus the full line with the UTC time), `status_off_on_status_round_trip_without_a_hook` (`:102`); `ipc_server::tests::dispatch_status_appends_the_hook_health` (`ipc_server.rs:297`); `health::tests::the_hook_health_reads_ok_not_watched_or_failed_with_its_reinstalls` (`health.rs:296`) and the precedence test `140b831`. `swch` prints the message unchanged and `swch open`'s probe accepts any `Ok` (`src/bin/swch.rs:73-94`); manual M7 | PASS |
| 5 | The latency bench shows no unjustified regression on the key path with the watchdog active | Tester r1 (master `fa20b30` vs HEAD, 3 runs each): R1-R13 within their bounds (R12 master 4.7/9.5 µs, HEAD 4.7/7.0). Tester r2 and r3 (3 runs each): R14 p50 4.8 against R14c 4.8 µs (bound +0.3 µs, met in every run). R16 is declared and not gated (ruling L2). R6/50's 9 ms p99 is one sample of n = 10 in one head run, and its p50 bound holds. Details under "Latency" below | PASS (per L1/L2) |
| 6 | (spec) A reinstall never double-installs and never leaves none; a hook not installed is never installed | `keyboard_hook::tests::a_reinstall_installs_the_new_hook_before_it_removes_the_old_one` (`keyboard_hook.rs:678`: calls `[InstallKeyboard(4), RemoveKeyboard(1)]`, counts `[2,1]`), `a_failed_reinstall_keeps_the_old_hook` (`:706`), `a_reinstall_of_a_hook_that_is_not_installed_installs_nothing` (`:733`), `a_reinstall_reports_an_old_hook_that_windows_removed_as_gone` (`:755`), `the_daemons_reinstall_of_a_hook_never_installed_makes_no_call` (`:778`, through `SYSTEM`, which panics under test); `main_thread::tests::a_failed_reinstall_keeps_the_old_hook_and_resets_nothing` (`:2061`), `a_reinstall_that_finds_no_hook_resets_nothing` (`:2086`). The developer mutation-checked each S3 rule (`cut.md:711`) | PASS |
| 7 | (spec) No key data from Raw Input is kept or logged | Scan `no_key_data_leaves_the_raw_input_read` (`mod.rs:2846`: `VKey`/`MakeCode` only in `raw_key_event`, `count_raw_keys`, `qualifies`, never on a `log::` line); `a_batchs_key_codes_do_not_outlive_its_count` (`mod.rs:2128`, red at `3117b0a` with "left: 12"), `a_counted_batch_leaves_no_byte_of_the_whole_buffer` (`:2901`), `the_buffered_read_answers_through_the_method_that_clears_its_buffer` (`:2876`); `Watchdog`'s fields are counts and instants (`decision.rs:92-120`) | PASS |
| 8 | (spec) Reuse of the 1.7 writer and counters; the 2.1 gauges where they help | four new counters with `Display` arms (`counters.rs:44-61`, SLOTS 11); `longest_callback_us` in the reinstall line (asserted in `a_reinstall_resets_the_engine_and_keeps_the_macros`). Leaving out `executor_unavailable` and `active_runs` is argued in cut D7 | PASS |
| 9 | (spec) Re-entry refusal counter for `with_main`, never logged on the hook path | `main_thread::tests::a_refused_reentry_is_counted_in_main_state_refused` (`main_thread.rs:1037`); a bump only (`main_thread.rs:218-220`) | PASS |
| 10 | (spec) A panic hook that logs through the writer with a stated bound | `logging::panic::tests::a_stuck_writer_holds_a_panic_report_at_most_its_budget` (`panic.rs:228`), `a_nested_panic_on_one_thread_shares_the_budget` (`:203`), `a_panic_report_reaches_the_log_file_before_it_returns` (`:248`), `a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook` (`:303`). The bound is 250 ms plus building one line (`panic.rs:13-23`), under the 1000 ms `LowLevelHooksTimeout` maximum | PASS |
| 11 | (spec) The 2.3 leftovers as small separate commits | `1aad2b1` (P1, odd fake PIDs `0x1001`/`0x2001`), `53b2499` (`winner.rs:56-58`), `4f2d406` (README `when:`), `fcfd66e` (`timers.rs:71` `expect("invariant: ...")`) | PASS |
| 12 | (audit KS-08) "Every binding then stops working while `swch status` still reports `running`" | status now says `hook ok/not watched/reinstall failed/watchdog timer failing, reinstalls: N, last <UTC>`; the reinstall path in rows 2 and 6; the engine after a reinstall: 4 scenarios (`scenarios.rs:2343,2363,2379,2395`, with `check_no_lone_key_up`/`check_press_symmetry`/`check_nothing_stuck`); manual M1 | PASS |

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-08 (the last half; 2.1 fixed the action half) | Detection: `decision::tests::key_events_with_a_silent_callback_...` and the loop `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`. Repair: `a_reinstall_installs_the_new_hook_before_it_removes_the_old_one`, `a_reinstall_resets_the_engine_and_keeps_the_macros`. Status: `status_reports_the_hook_health_over_the_pipe`. Heartbeat: `the_keyboard_callback_counts_every_call_also_while_remapping_is_off` | Yes. The red commits come before their fixes (`3505484` before `a09ce5a`, `87ea196` before `2b42857`, `a8aa0f7` before `2fcac6b`, `2c624d1` before `0c1153d`, `6c18929` before `c914a9f`). They are compile-red because the API is new. Tester r1 showed each one red by mutation instead: M13 (no heartbeat), M3/M4/M21/M22 (reinstall), M1/M2/M14 (decision); see `test_r1.md:21-65`. The S1 scan was assertion-red at `29060a1` | CLOSED in code. The live acceptance is the owner's manual M1. Status lines in `audit.md`/`roadmap.md` belong to the Phase-2 book sync (spec) |

## Rulings carried out (rulings.md)

| Ruling | Where | Status |
|---|---|---|
| S1 compile-time seam; S2 order; S3 one test per rule with mutations; S4 scan | `src/hook/installer.rs`: `#[cfg(not(test))] mod win32` is the only place that calls `SetWindowsHookExW`/`UnhookWindowsHookEx`/`SetWinEventHook`/`UnhookWinEvent`/`RegisterRawInputDevices`; the `cfg(test)` `System` is `unreachable!`. `29060a1` and `2904d04` land before `a8aa0f7`. S3 is row 6. S4 is `the_hook_installers_are_compiled_out_of_the_tests` (`installer.rs:580`), widened by `ee9ef4d`/`0e7165b` to whole files, every `SetWindowsHook*` spelling and `build.rs` | done |
| I1 the tick is never lost | `Ticker::pass`/`arm`/`wake_by` (`mod.rs:219-349`); `an_early_timer_signal_does_not_lose_the_tick` (`:1528`), `a_timer_that_refuses_once_is_set_again_at_the_next_pass` (`:1560`), T3 `f51dc7d`; `HookHealth::timer_failing` | done |
| I3 start/stop order | `app.rs:217-229` (start before `ControlWindow::create` and `install`, commented next to the executor), `app.rs:321` (`stop_watchdog` after `uninstall` and `drop(control_window)`); `the_hook_watchdog_starts_before_the_hooks_and_stops_after_them` (`app.rs:469`); the `hook_installed` gate (`mod.rs:246`) | done |
| M1-M7 | `keeps_ticking` (`decision.rs:227-235`); `WatchdogDesktopCheckFailed`; `hook_answered`; leaked counters plus the `GLOBAL_HOOK` lock; the M5 row (`decision.rs:602`); the cost in the README and module docs; the M7 precondition in M1 | done |
| Q1-Q8 | M1 reading (Q1); `HookReinstalled` uses `resync_from_os` and keeps macros (Q2); debug-only hold (Q3); FU-1..FU-4 in `impl.md` (Q4); Q5-Q8 are manual checks | done |
| L1 batched reads | `wake_mask`/`pump_ranges`/`drain`/`next_wait`/`RawBuffer`/`count_raw_keys` (`mod.rs:752-1172`); `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events` (`:2716`, red at `384ddf5` with "woke 198 times for 195") | done |
| V3-I1 no false reinstall from a backlog | `DRAIN_BUDGET` time budget, `pending` gives a 0 timeout; the accumulated `credit` with backlog-kept carry-over (`decision.rs:171-215`); the rule for one-sided events is stated (`decision.rs:16-27`); the three red-first rows | done |
| V3-I2 control row R14c | `bench.rs:573` | done |
| V3-M1 one M9 rule | `fix_r1.md:109` and the addendum `cut.md:762`: PASS iff K >= 400 and W <= K/2. The example rule was refuted with arithmetic, and review r2 accepted that. See hand-off H-O2: `cut.md` L1.12 still shows the older numbers | done (hand-off) |
| V3-M2 / V3-M3 / OQ1-OQ4 / Delta Q1-Q6 | R16 sets `hook_installed`, n = 1000; the stuck-queue guard and the one-tick back-off (`mod.rs:1043-1093`); the unfiltered pump; the OQ3 smoke test; OQ4 recorded | done |
| L2 | R16 declared (`cut.md:1000`, `bench.rs:75-82`, `mod.rs:33-42`, `README.md:939`, all with the measured "about 50 µs, once per burst, worst order"); Q6 goes to the owner as M6 (amended in `fix_r2.md`); FU-4 (`impl.md:152`); paced rows gate on p50; R-m1 (`errors.md:215`) and R-m2 (`RawBuffer::take`, `mod.rs:869-882`) fixed | done |

## Latency (ruling "Latency", owner decision: lowest latency)

- **Key path before and after.** The callback adds one `fetch_add(AcqRel)` on a cache line of its own (`health.rs:56-61`, called at `keyboard_hook_callback.rs:367`). R12 (unbound A, down+up): master 4.7 µs p50 / 9.5 µs largest p99, HEAD 4.7 / 7.0 (tester r1, 3 runs each); r3 HEAD 4.7. R4, R9-R11 are also equal to master within noise. The heartbeat cannot be measured at this resolution.
- **With the watchdog active (R14, steady typing at 1 ms pacing, watchdog ticking).** p50 4.8 µs against the control R14c 4.8 µs (bound +0.3 µs, met in 3/3 runs in r2 and r3). Largest p99 14.8 µs against R14c's 26.5 µs (r3). Against R12, for context: 4.8 vs 4.7 µs. Before L1 the same row was 22.7 µs p50 (tester r1), because the watchdog woke once per key event.
- **First key of a burst (R16, declared, not gated, ruling L2).** p50 47.7 µs median of 3 (r3; r2 52.7), p99 up to 145.8 µs, against R12 4.7 µs. It occurs at most once per burst, after at least 2 s without keys, and only in the worst order of Q6. Manual check M6 settles whether a real key ever meets it (FU-4 for item 3.5 if it does).
- **The rejected option (R15, raw input on the main thread).** 3.9 µs per key event on the main thread, in sequence with the callback (r1-r3: 3.9 / 3.8-3.9 / 3.9). This is an in-thread lower bound with no wake-up and no kernel copy.
- **Idle.** Idle produces no wake-ups. The idle wait is `QS_ALLINPUT` with `INFINITE` and no timer armed (`mod.rs:1076-1094`, test `:2357`). Ticking stops at most `ACTIVE_FOR + TICK` (2.25 s) after the last key (`decision.rs:227-235`; tests in row 1c). The live proof is M2's stop line (`W <= 100` for a 10-minute idle run, against about 2 400 if the watchdog woke through it).

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking or alloc, symmetry) | OK | `code < 0` still returns `CallNextHookEx` first. The only addition is the heartbeat RMW before the switch gate (`keyboard_hook_callback.rs:365-367`). It is a counter, not decision state, and it counts injected events on purpose to match raw input (cut D2). In debug builds only, `hold_if_asked` follows (`:383-384`). There is no new `log::`, allocation, lock, Win32 call or indexing by `vk_code`. Every path still ends in `LRESULT(1)` or `CallNextHookEx`. The suppression logic is unchanged, and the reinstall reset keeps swallowed records through `resync_from_os` (KS-14), as the 4 scenarios check |
| Engine purity, matching per owner decisions | OK | The engine only gains `ResetReason::HookReinstalled` with a `Display` arm, taking the `RemappingSwitched` arm (`engine/mod.rs:391-403,657-659`). Matching is untouched. The owner decision on lost key-ups holds after a reinstall (`scenarios.rs:2343`) |
| Thread ownership, sentinel, atomics, SAFETY | OK | Install, reinstall and uninstall run on main only. The reinstall runs in the drain (`main_thread.rs:806-878`), outside `with_main` and every callback. The watchdog thread holds only `&'static HealthState` and posts a request. No new `SendInput` (scan). Atomics use `Release`/`Acquire`/`AcqRel`, with no `Relaxed`/`SeqCst`/`static mut` in the added lines (grep). Every `unsafe {` in the 11 touched code files has a concrete `// SAFETY:` within the preceding lines (my awk scan; the only hits are two string literals in the S4 scan tests). The waitable timer is auto-reset (`CreateWaitableTimerExW` without `MANUAL_RESET`), so a signalled timer cannot spin the loop |
| Config reload, IPC | OK | No config change. The status reply keeps its prefix, is still one line, and is read from atomics by the IPC worker, so it answers even while main is stuck. The DACL is unchanged |
| No key logging | OK | Only counts and times are logged. The read buffer is cleared after each counted batch. The standard read's record is a local of its call (review r2 m2 accepted it as the pre-L1 baseline) |
| No test touches the system (S1) | OK | Under `cfg(test)` the installer is `unreachable!`. Watchdog threads in tests use `Source::Synthetic` (no registration). Tests post Win32 messages only to their own threads. `tests/` may not name an installer call or `core::app` (scan) |

## Review status

- Review r3 (`review_r3.md`) @ `06ae0d7`: **APPROVED** (Critical 0, Important 0, Minor 0). `17ca0de` adds one test-only commit after it.
- Triage r1: 3 CONFIRMED (T-F1 = R-I2 R14, R-I1 S4 scan). R-I1 was fixed in `ee9ef4d`/`0e7165b`, and T-F1 was closed by ruling L1 (R14 = R14c in r2/r3). Triage r2: 2 CONFIRMED (R16 = R-I1), closed by ruling L2 and `fix_r2.md`. No CONFIRMED Critical or Important item is open.
- Tester r3 (`test_r3.md`): **GREEN**. Of 9 mutations, 3 survived (M3, M4, M6); each is closed by a `Tests:` commit (`06ae0d7`, `17ca0de`) that was red under its mutation. Earlier rounds: r1 (20 mutations, 6 closed by tests), r2 (M10-M12 closed by T1-T3).
- NOTED and still open, worth doing while the context is fresh (none blocks):
  - R-m2 (r1): `logging/panic.rs:93` drops `report_on`'s result. With a dead `log-writer` and the `OnceLock` still set, a panic line goes nowhere. Ordinary log lines fall back to stderr in that case (`logging/mod.rs:80-86`); the panic hook has no fallback.
  - R-m3 (r1): the "never two hooks or none" wording (`keyboard_hook.rs:263-267`) ignores a foreign low-level hook above the daemon's.
  - R-m4 (r1): the README tree has no `debug_hold.rs` line.
  - T-note: the 5 s start window race in `HookWatchdog::spawn` (cosmetic).
  - Review r2 OQ2/OQ3/OQ4 (the two M9 rules, E4, `KEYEVENTF_UNICODE` `wScan`).

## Docs and hygiene

- README: tree (`main_thread.rs`, `health.rs`, `installer.rs`, `watchdog/`; the `when:` typo), the typical session, the `swch status` row, the counters and panic sentences, Technologies (`Win32_UI_Input`), the Hook watchdog feature bullet with R16's measured cost and the debug hold, and Known Limitations (R-1, R-2, R-5, R-6, R-7). All in sync with the code.
- CHANGELOG `[Unreleased]` follows Keep a Changelog:
  - Added: `main_state_refused`, the watchdog (KS-08, roadmap 2.4, every new log line), `swch status` health, the panic hook, the debug hold and bench rows R14/R14c/R15/R16.
  - Changed: the `swch status` reply text, and the panic message replaced.
  - Fixed: KS-08, its last half.
  - The counters example line is updated.
- `book/src/config/errors.md`: rows for every new log line and counter, plus the stop-line arithmetic (R-m1). No config meaning changed, so `default_config.ron` and the other `config/*.md` pages are correctly untouched. No RON example changed, so cfgcheck is not needed.
- New dependencies: none. `Cargo.toml` names `Win32_UI_Input` (already on through `Win32_UI_Input_KeyboardAndMouse`), and `Cargo.lock` is unchanged, so `--locked` passes.
- Non-config book pages (`architecture.md`, `roadmap.md`, `audit.md`, `gaps.md`, `build-and-run.md`, `glossary.md`, `overview.md`) are deferred to the Phase-2 book sync, as the spec prescribes. They are listed with file:line in `impl.md` "Book pages left to the Phase-2 book sync".
- Branch: cut from `master` `fa20b30`, merge-base equals `master` (no drift), so it is clean for `merge --no-ff`. It holds only this lane's work plus the four 2.3 leftovers the spec asked for. No amended commits.

## Technical debt (recorded where?)

- `src/logging/panic.rs:93`: no stderr fallback when the writer is `Gone` (R-m2 r1). Recorded only in `triage_r1.md`/`fix_r1.md`. It goes to gaps.md at the book sync.
- `src/hook/keyboard_hook.rs:263-267`, `CHANGELOG.md` watchdog entry: the "never two hooks or none" claim is too strong for a foreign hook above ours (R-m3 r1). Lane files only; it goes to the book sync.
- `README.md:64-77`: the tree lacks `debug_hold.rs` (R-m4) and `src/logging/` (FU-3, pre-existing). `impl.md` FU-3 covers the second; R-m4 lives only in the triage files.
- `src/hook/watchdog/mod.rs:470-472`: the start timeout race (T-note, `impl.md` open point 3). Cosmetic, and the executor has the same window.
- `src/hook/installer.rs` S4 scan: it is lexical (a `'"'` char literal before a call, multi-line strings, block comments), and `tests/` is read at the top level only (`fix_r1.md` open point 4). Lane files only.
- A permanently stuck raw-input queue (V3-M3 guard) means 4 wakes a second for as long as it lasts, counted in `raw_input_unread`. The ruling accepted this as "never a spin". Declared in `errors.md`'s counters row.
- Unverified OS behaviour, each with a fallback or a manual check:
  - Q5 (raw input across desktops: M4, M5);
  - Q6 (raw input order against the hook: M6);
  - Q7 (`PM_QS_SENDMESSAGE` and LL hook calls);
  - Q8 (a second panic-hook run);
  - E2/E3 (merging and queue limits: M9);
  - E4 (the wake mask and pending `WM_INPUT`: M9, with the L1.5 kernel-wait fallback);
  - E9 (cleanup after `GetRawInputBuffer`: M10);
  - E11 (`RAWINPUT_ALIGN`);
  - critic OQ4 (USER quota: M10);
  - review OQ4 (`KEYEVENTF_UNICODE` `wScan` as `MakeCode`).

  They are recorded in `cut.md` L1.3 and in the manual checks, and need a gaps/roadmap home at the book sync.
- Residuals R-1..R-7 (`cut.md:661-668`): in README Known Limitations, except R-3 (an M1 reading) and R-4 (covered by the desktop check). They go to `known-issues.md` at the book sync.
- Follow-ups FU-1..FU-4 (`impl.md:147-152`). FU-1: an uninstall guard for a main-thread panic. FU-2: raw input as the "input left" signal. FU-3: the README `src/logging/` line. FU-4: a 3.5 trim of the first pass if M6 shows raw input before the hook. They go to the roadmap's "Follow-ups from Phase 2" at the book sync.
- Pre-existing, not this lane: 3 `cargo doc` warnings (`src/data/vk_name.rs:4` `<raw>`, two `ConfigError::Missing` private links).

## Problems

None that block the merge. Nothing requires REWORK, and no root cause is open in code, tests or docs. The points below are non-blocking notes. Their owners are named in the hand-off sections.

### N1. Two manual-check rule sets exist in the lane files (V3-M1 carried out, not yet propagated)
- **What.** `cut.md` L1.12 (`:1108`, `:1110`) still shows M9 "K >= 150, W <= 130" and amended M2 "W <= 40". The binding rules are in `fix_r1.md:109-111` and the addendum `cut.md:762`: M9 PASS iff K >= 400 and W <= K/2; M2 W <= 100.
- **Impact.** The owner could be handed the wrong pass rule, and a correct build would then fail M2 (about 45 wakes against 40).
- **Root cause.** The fix round recorded the ruling in the addendum and `fix_r1.md` and did not edit the architect's L1.12 table.
- **Owner.** The orchestrator, when assembling the owner's checklist (hand-off H-O2). No code change is needed.

### N2. Ruling L1's "R12, R14 and the new rows on master and HEAD" was satisfied in substance
- **What.** Rounds 2 and 3 compared `prev` against HEAD. Master was measured in round 1 only.
- **Why it suffices.** R14, R14c and R16 do not exist on master. R12 master (4.7 µs) was measured in r1, the callback and main code are unchanged since `140b831`, and R12 on HEAD stays 4.7 µs in r2 and r3. Nothing to do. It is recorded so the reading is explicit.

## Positives

- The S1 seam makes it impossible for any test to put a hook or a raw-input sink on the owner's machine. The guarantee is in the compiler: under test the system installer is `unreachable!`, and a whole-file scan backs it up. This is a durable safety asset for every later lane.
- The decision is a small pure state machine with a written anti-loop rule. Its fake-clock rows cover each clause, and mutations proved the rows (M1, M2, M5, M9, M10, M14).
- Two latency regressions were caught by the bench and fixed by design (L1 batched reads), not by loosening the bound. R14 now matches its own control.
- Each status, log line and counter has a `Display` arm, an errors-page row and a test.
- The key-data hygiene went beyond the spec: the buffer is cleared, and a structural test pins that the clear cannot be bypassed.

## Notes for later lanes

- 3.5 (hot-path budget): the heartbeat RMW is part of the callback budget. FU-4 applies if M6 shows raw input reaching the watchdog before the hook call.
- 4.6 (tray): any raw-input use in the daemon must share the watchdog's registration, because Windows allows one window per device class and process (`mod.rs:44-49`).
- 4.2 (background mode): it relies on the panic line reaching `daemon.log`, which depends on the writer. Consider R-m2's stderr or file fallback before it.
- Any new `SetWindowsHookEx*`/`SetWinEventHook`/`RegisterRawInputDevices` call must go through `hook::installer`. The S4 scan fails otherwise.

## Hand-off: Phase-2 book sync (doc-writer)

1. `impl.md` "Book pages left to the Phase-2 book sync", every listed `file:line`:
   - `architecture.md`: module rows (`health.rs`, `installer.rs`, `watchdog/`, `debug_hold.rs`, `logging/panic.rs`); thread table `hook-watchdog` ("one wake per burst; while ticking, no work per key event and one `GetRawInputBuffer` drain per tick, 4 Hz"); the KS-08 section `:213-215`; `:577`; the key-event path (the heartbeat); the data-structures table (`HealthState`). The `with_engine`/`DECIDING` rows at `:198`, `:308` are stale since 2.2 and worth a check.
   - `roadmap.md`: the 2.4 status block. Change "on the 2.3 window" to "its own message-only window on the `hook-watchdog` thread" (D1). Add the acceptance reading Q1 (measured from the ScrollLock key-down). Update `:439` and the KS-08 row `:730`.
   - `audit.md:31`, `:312`: KS-08 status, "Fixed in <merge sha> (`feat/hook-watchdog`)" for the watchdog half (status lines only).
   - `gaps.md`: `:155` closed; `:164` (counter added; the 2.2 lock is gone); `:232` closed through the writer with a 250 ms bound, not a synchronous file write.
   - `build-and-run.md`: `:199`/`:231` (reply text), `:270` (`status` now reports the hook), `:461` (troubleshooting row: the watchdog reinstalls; keep "swch exit / open" as the fallback).
   - `glossary.md` (Raw Input, watchdog, heartbeat) and `overview.md` (current state).
2. Roadmap "Follow-ups from Phase 2": FU-1, FU-2, FU-3, FU-4 (`impl.md:147-152`).
3. Roadmap "Manual checks after Phase 2" (or Checkpoint 2), the lane's owner checklist with these texts:
   - M1 (re-run on the batched debug build, M7 precondition);
   - M2 amended (W <= 100);
   - M3, M4, M5, M7, M8 (`impl.md`);
   - M6 amended (`fix_r2.md`, parts A and B);
   - M9 (`fix_r1.md:106-109`: K >= 400, W <= K/2);
   - M10 (`fix_r1.md:110`).

   Do not copy `cut.md` L1.12's numbers (N1).
4. `known-issues.md` / `gaps.md`: residuals R-1..R-7; the unverified items (Q5-Q8, E2-E4, E9, E11, OQ4s); R-m2 (the panic hook without a fallback) and R-m3 (wording).
5. README: add the `debug_hold.rs` tree line (R-m4) and the `src/logging/` entry (FU-3).

## Hand-off: orchestrator (CLAUDE.md, agent files, workflow)

- **H-O1 Owner manual checks now.** Per ruling Q5-Q8 ("to the owner from the start"), send M1-M10 as in book-sync item 3. M6 (Q6) and M9 (E4) decide follow-ups:
  - M6 decides FU-4.
  - M9 failing (W >= K) means a new delta for the L1.5 kernel-wait fallback.
- **H-O2** Hand the owner only `fix_r1.md`'s M9/M2 rules (N1).
- **CLAUDE.md Layout:** `src/hook/health.rs` (heartbeat, `HookHealth`, reinstall request/outcome), `src/hook/installer.rs` (the only Win32 hook/raw-input installers, `#[cfg(not(test))] mod win32`; `FakeInstaller` in tests), `src/hook/watchdog/` (`mod.rs` thread and loop, `decision.rs` pure), `src/hook/debug_hold.rs` (debug builds only), `src/logging/panic.rs`.
- **CLAUDE.md principle 1:** the heartbeat `hook.health().callback_seen()` is the prescribed hook-path addition. It sits after `code < 0` and before the switch gate, and it counts injected events too. The debug-only hold is the only other addition.
- **Principle 5:** `HealthState` (atomics inside `KeyboardHook`) joins the shared-state list.
- **Principle 6:** a fourth helper thread, `hook-watchdog`. It owns a message-only window, the process's raw keyboard registration and a waitable timer. It reads atomics and posts a reinstall request. Only main installs, reinstalls or removes hooks.
- **Safety rules section:**
  - every hook or raw-input registration goes through `hook::installer`;
  - tests use `FakeInstaller` or `Source::Synthetic`;
  - `tests/` may not name installer calls or `core::app`;
  - the S4 scan enforces it.
- **Counters** (principle 1 and the logging section): `main_state_refused`, `raw_input_unread`, `watchdog_timer_not_set`, `watchdog_desktop_check_failed`. The panic hook is installed by `logging::init` (tests still never call it).
- **Bench rules** (CLAUDE.md and `tester.md` 4b):
  - R14 gates on p50 <= R14c p50 + 300 ns;
  - the 1 ms-paced rows (R14, R14c) gate on p50 only, and their p99 is reported as the largest of 3 against the control's largest of 3;
  - R15 is evidence only;
  - R16 is a declared row (about 50 µs, once per burst), not gated;
  - rows absent on master compare against HEAD's own control.
- **`code-reviewer.md` / `architect.md`:** add S1-S4 to the checklist, plus "a new latency bound in a cut is probed with a diagnostic row before the cut is final". Both R14's and R16's planned bounds failed on the first measurement and cost two ruling rounds (L1, L2).
- **`docs/WORKFLOW.md`:** a ruling that adopts an example number (V3-M1) should say "or the developer's argued number", and the fix round must update every table that carries the old number (N1).
