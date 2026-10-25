# Implementation: p2-2.4-watchdog round 1

Branch `feat/hook-watchdog` in `D:/key-switch-rs/key-switch-rs`, 28 commits on `master` `fa20b30`, tree clean at `4a90d44`. Implemented per `cut.md` as amended by `rulings.md`; how each ruling was carried out is in the "Developer addendum" at the end of `cut.md` (S1-S4, I1, I3, M1-M7, Q1-Q8, deviations, and one open point that needs a ruling).

## Items

Red outputs are the commit bodies' "Red first" lines; every red commit failed to compile or failed the named test, and the next commit makes it green.

| # | Commit | Item | Red first -> green |
|---|---|---|---|
| 1 | `1aad2b1` Tests: MovingQuery's fake windows are no multiple of 4 (2.3 P1) | `main_thread.rs` test: windows `0x1001`/`0x2001`, `p4097.exe`/`p8193.exe` | not red-first (a spurious red fixed); re-check: green, and red with `input_returned` removed from `answer_input_signals` (`left: Some(Some("p4097.exe"))`, `right: Some(Some("p8193.exe"))`) |
| 2 | `53b2499` Engine: the winner's doc names the foreground cache (2.3 P2) | `winner.rs:56-58` | doc only |
| 3 | `fcfd66e` Platform: not_a_timer states its invariant | `timers.rs:71` | `expect("invariant: ...")` |
| 4 | `4f2d406` README: the foreground.rs line of the tree (2.3 P3) | `README.md:73` | doc only |
| 5 | `29060a1` Tests: no test build compiles a hook installer (red) | S4 scan `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests` | red: `src/hook/installer.rs has no #[cfg(not(test))] block mod win32 {`, `7 installer calls a test build compiles: src/hook/keyboard_hook.rs:153: SetWindowsHookExW( ...` |
| 6 | `2904d04` Hook: the hook installers sit behind a seam that no test build compiles | S1: `hook::installer` (trait, `SYSTEM`, `#[cfg(not(test))] mod win32`, test stub that panics), `install_with`/`uninstall_with` | green; `the_system_installer_panics_at_every_call_under_test` |
| 7 | `62d79da` Tests: a refused re-entry of the main state is counted (red) | `main_thread::tests::a_refused_reentry_is_counted_in_main_state_refused`, counter names | red: `E0599 no method named get found for struct Counters`, `no variant ... MainStateRefused` |
| 8 | `ef27fa4` Hook: count the refused re-entries of the main state | D8: `Counter::MainStateRefused`, `Counters::get` (test), bump in `with_main` | green |
| 9 | `6c18929` Tests: the keyboard callback counts every call (red) | `keyboard_hook_callback::tests::the_keyboard_callback_counts_every_call_also_while_remapping_is_off` | red: `E0599 no method named health found for reference &'static KeyboardHook` |
| 10 | `c914a9f` Hook: a heartbeat of the keyboard callback | D2: `hook::health::HealthState`, `callback_seen` before the switch gate | green; mutation (bump after the gate): `left: 3, right: 5` |
| 11 | `bdf1480` Tests: a reinstalled hook resets the engine (red) | 4 scenarios in `engine::scenarios` (section "A reinstalled keyboard hook") | red: `E0599 no method named hook_reinstalled found for struct Scenario` |
| 12 | `6e5717b` Engine: reset after a hook reinstall | D13: `ResetReason::HookReinstalled` (Display), reset arm, docs, harness step | green; mutation `HookReinstalled => {}`: `scenario: down F13 at +0 ms: expected to fire [SwitchLanguage], fired []` |
| 13 | `a8aa0f7` Tests: the main thread reinstalls only a silent, installed hook (red) | S3 fake-installer tests in `keyboard_hook::tests`, main-thread request and `reinstall_if_silent` tests | red: `E0425 cannot find function reinstall_if_silent`, `E0432 unresolved imports ReinstallOutcome, ReinstallRequest / OldHook, Reinstalled`, `E0599 no method named reinstall_keyboard_hook_with / request_reinstall` |
| 14 | `2fcac6b` Hook: reinstall the keyboard hook on the main thread | D4: `reinstall_keyboard_hook(_with)`, `Reinstalled`, `OldHook` (Display), `Request::ReinstallHook`, `request_reinstall`, `serve` arm, `reinstall_if_silent`, `dispatch_sent_messages`, `MainState::hook_reinstalled` | green; S3 mutations caught (addendum) |
| 15 | `3505484` Tests: the watchdog's decision (red) | 15 fake-clock rows in `hook::watchdog::decision::tests` | red: `E0425 cannot find type Watchdog`, `cannot find value SILENCE`, `E0433 cannot find type Verdict` |
| 16 | `a09ce5a` Hook: the watchdog's decision | D3 with I1 (`next_tick`), M1, M5 | green; mutations of the M5 order, the restart of the silence, the owed credit, both M1 clauses, the backoff, the outstanding gate and rule 4 each fail a row (`raw_events_before_an_answer_are_no_evidence_after_it`: `left: [1000, 3250]`, `right: [1000]`) |
| 17 | `87ea196` Tests: the hook watchdog thread (red) | loop on a fake timer and clock (I1, I3, M2), thread with synthetic raw input, two scans, `qualifies`, the I3 order scan of `App::run`, 11 counter names | red: `E0405 cannot find trait RequestReinstall / TickTimer`, `E0425 cannot find function qualifies`, `E0433 cannot find type HookWatchdog`, `E0425 cannot find type Ticker` |
| 18 | `2b42857` Hook: the hook watchdog thread | D1, D5, D6: `HookWatchdog`, `Ticker`, `SinkWindow`, `RawInputRegistration`, `raw_key_event`, `qualifies`, `App::run` start/stop, `Win32_UI_Input` | green; mutations caught: arming only after a tick (both I1 tests), no timeout fallback, no install gate, an uncounted desktop failure, a reply never taken, VK 255 kept |
| 19 | `2c624d1` Tests: swch status shows the hook health (red) | `health::tests` (Display table, recording, M3, I1 status), `ipc_server::tests::dispatch_status_appends_the_hook_health`, `tests/ipc_roundtrip.rs`, app and watchdog status asserts | red: `E0432 unresolved import crate::hook::health::HookHealth`, `E0560 struct DaemonStatus has no field named hook`, `E0599 no method named snapshot` |
| 20 | `0c1153d` IPC: swch status shows the hook health | D7 with I1 and M3: `HookHealth`, `HealthState` recording, `Watched`, `logging::utc_timestamp` | green |
| 21 | `96a43dc` Tests: a panic is logged within a bounded wait (red) | `logging::panic::tests` (5 tests and the child) | red: `E0425 cannot find function panic_text / budget_end / report_on`, `cannot find value REPORT_BUDGET` |
| 22 | `a013ee2` Logging: a panic hook writes the message through the log writer | D9: `logging::panic`, `writer::send_by` `pub(crate)`, `flush_by` | green; the child test fails with the hook writing nowhere (`daemon.log` missing) |
| 23 | `58c4370` Tests: the debug hold exists only in debug builds (red) | `debug_hold::tests` (2), callback scan | red: `E0425 cannot find function parse / should_hold`; the scan alone: `no call site of debug_hold:: in src/hook/keyboard_hook_callback.rs`, `... in src/core/app.rs` |
| 24 | `c9aa86e` Hook: a debug-build hold of the keyboard callback for the watchdog's manual check | D10 | green; release daemon and release test harness build |
| 25 | `ea622e2` Bench: R14 key path with the watchdog running, R15 main-thread cost of a raw input dispatch | D11, plus the evidence row R14/settled | ignored bench; numbers below |
| 26 | `0cd75d8` Docs: the changelog names the bench rows R14 and R15 | CHANGELOG Added | doc only |
| 27 | `381da55` Hook: the watchdog's docs state what its work costs a callback at the same time | `watchdog/mod.rs` module docs | comments only |
| 28 | `4a90d44` Tests: the watchdog thread has no top-level window (critic OQ3) | `hook::watchdog::tests::the_watchdog_thread_has_no_top_level_window` | not red-first: answers the critic's question; green |

## Tests added

- `hook::installer`: `the_system_installer_panics_at_every_call_under_test`, `the_hook_installers_are_compiled_out_of_the_tests`.
- `hook::keyboard_hook`: `a_reinstall_installs_the_new_hook_before_it_removes_the_old_one`, `a_failed_reinstall_keeps_the_old_hook`, `a_reinstall_of_a_hook_that_is_not_installed_installs_nothing`, `a_reinstall_reports_an_old_hook_that_windows_removed_as_gone`, `the_daemons_reinstall_of_a_hook_never_installed_makes_no_call`.
- `hook::main_thread`: `a_refused_reentry_is_counted_in_main_state_refused`, `a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing`, `a_reinstall_request_for_a_hook_that_is_not_installed_installs_nothing`, `a_reinstall_request_to_an_ended_main_thread_fails_at_once`, `a_reinstall_resets_the_engine_and_keeps_the_macros`, `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing`, `a_reinstall_that_finds_no_hook_resets_nothing`, `a_hook_that_answered_is_kept_without_a_replace`.
- `hook::keyboard_hook_callback`: `the_keyboard_callback_counts_every_call_also_while_remapping_is_off`, `the_debug_hold_is_compiled_only_into_debug_builds`.
- `hook::health`: `the_heartbeat_counts_each_call`, `the_heartbeat_has_a_cache_line_of_its_own`, `every_reinstall_outcome_has_its_own_text`, `the_hook_health_reads_ok_not_watched_or_failed_with_its_reinstalls`, `a_reinstall_is_recorded_with_its_time`, `a_failed_attempt_reads_reinstall_failed_until_the_next_reinstall`, `a_failed_attempt_clears_once_the_hook_answers`, `a_tick_timer_that_cannot_be_set_is_not_hook_ok`.
- `hook::watchdog::decision` (fake clock): `key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall`, `a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more`, `a_callback_that_keeps_up_asks_for_nothing`, `idle_asks_for_nothing_and_stops_ticking`, `a_busy_main_thread_gives_no_reinstall_loop`, `reinstalls_that_bring_no_callback_back_are_spaced_out`, `the_silence_resets_to_1_s_once_a_callback_is_seen`, `a_single_raw_event_without_a_callback_asks_for_nothing`, `raw_events_of_callbacks_seen_at_the_last_tick_are_explained`, `evidence_is_checked_before_this_intervals_raw_events_are_added`, `a_key_after_idle_waits_a_full_second_of_silence`, `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops`, `raw_events_before_an_answer_are_no_evidence_after_it`, `a_declined_request_needs_fresh_evidence`, `a_request_that_could_not_be_sent_is_asked_again_at_the_next_tick`.
- `hook::watchdog` (loop on a fake timer and clock; the thread with synthetic raw input; scans): `an_early_timer_signal_does_not_lose_the_tick`, `a_timer_that_refuses_once_is_set_again_at_the_next_pass`, `a_reply_of_the_main_thread_is_taken_at_the_next_pass`, `a_failed_reinstall_is_no_longer_reported_once_the_hook_answers`, `raw_input_before_the_hook_is_installed_is_no_evidence`, `a_failed_desktop_check_declines_and_is_counted`, `only_keyboard_events_the_hook_can_see_count`, `the_watchdog_thread_starts_and_stops_without_raw_input`, `the_watchdog_thread_has_no_top_level_window`, `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`, `synthetic_raw_events_with_a_live_heartbeat_send_no_request`, `the_watchdog_injects_no_input`, `no_key_data_leaves_the_raw_input_read`.
- `hook::debug_hold`: `the_hold_key_is_named_like_a_config_key`, `only_a_key_down_of_the_armed_key_holds`.
- `hook::ipc_server`: `dispatch_status_appends_the_hook_health`.
- `logging::panic`: `a_panic_line_names_the_thread_the_place_and_the_message`, `a_nested_panic_on_one_thread_shares_the_budget`, `a_stuck_writer_holds_a_panic_report_at_most_its_budget`, `a_panic_report_reaches_the_log_file_before_it_returns`, `a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook`, `panic_child` (a no-op in the suite).
- `engine::scenarios`: `a_swallowed_key_released_while_the_hook_was_gone_fires_at_its_next_press_after_the_reinstall`, `a_passed_key_held_through_a_reinstall_keeps_its_repeats_and_key_up_passing`, `a_hold_armed_before_the_hook_was_gone_fires_nothing_after_the_reinstall`, `a_key_pressed_while_the_hook_was_gone_passes_its_key_up_after_the_reinstall`.
- `core::app`: `the_hook_watchdog_starts_before_the_hooks_and_stops_after_them`.
- `tests/ipc_roundtrip.rs`: `status_reports_the_hook_health_over_the_pipe`; `status_off_on_status_round_trip_without_a_hook` expects the new text.
- Updated: `counter_names_are_snake_case` (11 names) and the counters and writer report strings; `dispatch_switches_remapping_and_status_reports_the_flag`; `daemon_handle_switches_remapping_through_the_main_thread`.

No test installs a hook, registers for raw input or injects input: the installers are compiled out of the test build (S1), the watchdog thread tests use synthetic raw input, which skips the registration and the desktop check, and the reinstall tests use the fake installer or the system one, which panics under test. The child-process panic test runs the test binary itself (`--exact logging::panic::tests::panic_child`), with stdout and stderr to null.

## Remarks

Critique (`critique.md`) and rulings (`rulings.md`):
- I1 - RESOLVED at `watchdog/decision.rs` (`next_tick`), `watchdog/mod.rs:197,208,278` (`wake_by`, `pass`, `arm`); pinned by `an_early_timer_signal_does_not_lose_the_tick`, `a_timer_that_refuses_once_is_set_again_at_the_next_pass`, `a_tick_timer_that_cannot_be_set_is_not_hook_ok`.
- I2 - RESOLVED by S1-S4 at `src/hook/installer.rs` and `keyboard_hook.rs:159,227,276`; pinned by the S3 tests and `the_hook_installers_are_compiled_out_of_the_tests`.
- I3 - RESOLVED at `app.rs:220,321` and `watchdog/mod.rs:176` (`hook_installed` gate); pinned by `raw_input_before_the_hook_is_installed_is_no_evidence`, `the_hook_watchdog_starts_before_the_hooks_and_stops_after_them`.
- M1 - RESOLVED at `decision.rs:188` (`keeps_ticking`); pinned by `a_single_raw_event_without_a_callback_asks_for_nothing`, `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops`.
- M2 - RESOLVED at `watchdog/mod.rs` `Ticker::ask` and `counters.rs` (`WatchdogDesktopCheckFailed`); pinned by `a_failed_desktop_check_declines_and_is_counted`.
- M3 - RESOLVED at `health.rs` (`hook_answered`, `record_reinstall`) and `Ticker::pass`; pinned by `a_failed_attempt_clears_once_the_hook_answers`, `a_failed_reinstall_is_no_longer_reported_once_the_hook_answers`.
- M4 - RESOLVED at `main_thread.rs` (`a_refused_reentry_is_counted_in_main_state_refused` under `GLOBAL_HOOK`, `saturating_sub`), the watchdog's `counters` parameter, the panic tests' leaked counters.
- M5 - RESOLVED at `decision.rs` `tick` (check before add); pinned by `evidence_is_checked_before_this_intervals_raw_events_are_added`.
- M6 - RESOLVED in `watchdog/mod.rs` module docs ("Cost") and the README Hook watchdog bullet; R14 and R14/settled measure the watchdog's per-key work next to the callback. The end-to-end path cannot be measured without a hook. The measurement raises the open point below.
- M7 - RESOLVED in manual check M1 below.
- OQ1 (Q6) - not decidable without a live hook: manual check M6 gathers the evidence; it also decides the R14 open point.
- OQ2 (late delivery of timed-out calls) - the M1 log timestamps settle it (step 6 of M1).
- OQ3 (default IME window) - RESOLVED: `the_watchdog_thread_has_no_top_level_window` finds no top-level window of the running watchdog thread.
- OQ4 (gesture-path scan) - RESOLVED: the exclusion of `src/hook/watchdog` is intended (it starts its thread once, at startup), stated in the doc of `the_gesture_path_starts_no_thread` (`keyboard_hook_callback.rs`).

## Deviations from the cut

The 13 deviations are listed in `cut.md`, "Developer addendum", "Deviations from the cut". In short: the `installer` seam (S1); `HookHealth::timer_failing` (I1); the first raw event after idle starts the silence, and an answer drops the raw events before it; the `Ticker`/`TickTimer`/`plan`/`wake_by` loop with a wait timeout when the timer cannot be set (I1); `HealthState::hook_installed` (I3); 11 counters; the "asking again" warning only when sent; `Watched` drops first; the fake's call names renamed; bench row R14/settled; the backoff row runs 250 s with 8 requests; the `OldHook::Gone` text; the panic hook uses `std::thread::current()`.

## Unsafe / atomics added

Unsafe blocks, each with its SAFETY comment in place:
- `src/hook/installer.rs:90,101,109,127,141,156` (`mod win32`, compiled outside tests only): `SetWindowsHookExW` ("the documented installer for WH_KEYBOARD_LL; it gets a 'static extern "system" callback and no module handle"), `UnhookWindowsHookEx` ("came from SetWindowsHookExW in this process, and the caller took it out of the field that kept it, so it is removed at most once"), `SetWinEventHook` (out of context, to this thread's queue, 'static callback), `UnhookWinEvent`, `RegisterRawInputDevices` twice ("one initialised RAWINPUTDEVICE on this stack frame, alive for the whole call ... the size passed is its size").
- `src/hook/watchdog/mod.rs`: `PostThreadMessageW` (synthetic raw, tests only; `WM_QUIT` at stop), `CreateWindowExW` ("Message" class, `HWND_MESSAGE`), `DestroyWindow` (on the creating thread, HWND not Send), `PeekMessageW` (queue creation; pump), `GetCurrentThreadId`, `MsgWaitForMultipleObjectsEx` (the timer handle kept open by `ticker`), `DispatchMessageW`, `GetRawInputData` ("the HRAWINPUT of a WM_INPUT this thread took and has not dispatched yet ... `raw` is a local RAWINPUT (8-aligned) whose size is in `size`"), the union read (`dwType` says RAWKEYBOARD); test-only `EnumThreadWindows` and its callback.
- `src/hook/main_thread.rs` `dispatch_sent_messages`: `PeekMessageW(PM_NOREMOVE | PM_QS_SENDMESSAGE)` ("leaves every posted message in this thread's queue, and the sent messages it dispatches run this thread's own hook callbacks and window procedures, outside the main state's borrow").
- `src/hook/keyboard_hook_callback.rs` test: the extern callback with an injected `KBDLLHOOKSTRUCT`.
- `src/bench.rs` (test only): R15's window procedure, `GetRawInputData` on a null handle, `DefWindowProcW`, `GetModuleHandleW`, `RegisterClassW`, `CreateWindowExW`, `PostMessageW`/`PeekMessageW`/`DispatchMessageW`, `DestroyWindow`/`UnregisterClassW`.

Atomics (no `Relaxed`, no `SeqCst`, no `static mut`):
- `HealthState::heartbeat` (own cache line): `fetch_add(AcqRel)` in the callback, `load(Acquire)` by the watchdog and the main thread.
- `hook_installed`, `watched`, `last_attempt_failed`, `timer_failing`: `Release` stores, `Acquire` loads (the last two write only on a change); they publish no other data.
- `reinstalls` (`fetch_add(AcqRel)`) after `last_reinstall_ms` (`store(Release)`); `snapshot` loads the count first (`Acquire`), then the time (`Acquire`): a reader that sees `n` sees the time of reinstall `n` or a later one.
- `debug_hold::ARMED` (debug builds only): one `Release` store before the hooks, `Acquire` loads in the callback.
- `KeyboardHook::hook_handle`: `swap(AcqRel)` in the reinstall, as in `uninstall`.
- `Counters`: three new slots, `fetch_add(AcqRel)` as before; `Counters::get` (test only) `load(Acquire)`.

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass, on every green commit (red commits fail to compile the tests by design). `cargo build --release --locked` and `cargo test --release --locked --lib --no-run` (no `debug_assertions`): pass.
- Tests run by the developer: `cargo test --locked --lib -- hook:: logging:: engine:: core::app platform::timers` (495 passed, 2 ignored) and `cargo test --locked --test ipc_roundtrip` (7 passed); each red-first test alone; the real-thread watchdog tests five times in a row (green). The full `cargo test --locked` run is left to the tester.
- Mutation runs (developer, informal; the tester runs the official set): listed per commit above and in the addendum.
- Manual ignores added: none. The live-daemon checks are M1-M8 below, which no test can stand in for.

## cfgcheck

No RON example changed in this lane: no `ron` block in README, the book or `default_config.ron` was touched (the only fenced block touched is README's `text` block of the typical session), and no config meaning changed. The cfgcheck harness was not run.

## Book

`mdbook build book -d D:/tmp/ks-orch/p2-2.4-watchdog/book`: built (the mermaid preprocessor warns about its mdbook version, as before). Link check of the built book (relative files and anchors): 23 pages, 7941 links, 0 broken. Changed in the lane: `book/src/config/errors.md` only (rows for the watchdog, the reinstall, the state reset, the panic line, the debug hold, the counters row).

## Docs

- README: tree (`main_thread.rs`, `health.rs`, `installer.rs`, `watchdog/`), the 2.3 P3 typo, the typical session and the `swch status` row, the log file sentences (counted `main_state_refused`; the panic line), Technologies (`Win32_UI_Input`), the Hook watchdog feature bullet (with its per-key cost and the debug hold), Known Limitations (the watchdog's limits: keys unremapped before the reinstall, R-1, R-5, R-7).
- CHANGELOG `[Unreleased]`: Added (main_state_refused; the hook watchdog, roadmap 2.4, KS-08; `swch status` hook health; the panic hook; the debug hold, debug builds only; bench rows R14, R14/settled, R15; the counters example line), Changed (the `swch status` reply text; Rust's default panic message replaced), Fixed (KS-08, its last half).
- `book/src/config/errors.md` (above). `default_config.ron` and the other config pages: no change.

## Bench (developer runs)

`D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev.md`. Runs 1-3 of HEAD and run 1 of master were taken while the machine was in a Remote Desktop session (Get-CimInstance said so): every Win32-heavy row was about 12 times the 2.3 numbers on both (R12 p50 57.6 us on HEAD, 57.8 us on master), so they show no difference between the two and nothing else. Representative runs (HEAD runs 4-5, master run 2; p50 / p99 in us):

| Row | master | HEAD run 4 | HEAD run 5 |
|---|---|---|---|
| R4 | 4.8 / 10.4 | 4.8 / 8.7 | 4.8 / 9.1 |
| R7 | 10.4 / 27.6 | 10.7 / 43.6 | 10.5 / 50.9 |
| R12 | 4.8 / 11.0 | 4.7 / 5.9 | 4.8 / 10.9 |
| R13 | 10.7 / 46.9 | 10.6 / 25.6 | 11.4 / 55.5 |
| R14 | - | 22.2 / 91.7 | 22.8 / 92.1 |
| R15 | - | 3.9 / 13.5 | 3.9 / 12.4 |

Runs 6-7 (a noisier spell, R12 p50 8.0 us) added R14/settled: R12 8.0, R14 27.6-29.0, R14/settled 8.5. An experiment with the bench thread at `THREAD_PRIORITY_HIGHEST` (not committed) gave the same gap (R12 4.7-5.3, R14 24.0-30.2). With a 300 us busy wait after each post (not committed, the R14/settled construction) R14 was 5.0 us against R12 4.8 us.

Reading: the callback's own extra work, the heartbeat RMW, does not show (R12 equals master's; R14/settled is within 0.5 us of R12). R14 as D11 builds it, with the watchdog handling an event at the same moment as the callback, is about 18 us slower at the median and misses D11's bound by far. R15, the rejected design, costs 3.9 us per key on the main thread. See the open point.

## Book pages left to the Phase-2 book sync (not edited here)

- `book/src/architecture.md:29` (logging module row: `panic.rs`), the hook module row (`health.rs`, `installer.rs`, `watchdog/`, `debug_hold.rs`), `:179-189` (thread table: `hook-watchdog`), `:213-215` (KS-08 section: the watchdog now watches the hook), `:577` (KS-08 row), the key-event path (the heartbeat).
- `book/src/roadmap.md:498-...` (2.4 status block; "on the 2.3 window" becomes "on its own message-only window, on the `hook-watchdog` thread", D1; the acceptance reading, Q1), `:439`, `:730` (KS-08 row), FU-1, FU-2 (below).
- `book/src/audit.md:31`, `:309` (KS-08 status lines).
- `book/src/gaps.md:155` (closed), `:164` (counter added: `main_state_refused`), `:232` (closed by the panic hook, through the writer, not a synchronous file write).
- `book/src/build-and-run.md:199`, `:231` (the `swch status` reply text), `:270` (`status` now reports the hook's health), the troubleshooting row near `:461`.
- `book/src/glossary.md` (Raw Input, watchdog, heartbeat).
- `book/src/overview.md` (current state).
- Fix round 1 (ruling L1, `fix_r1.md`): the `architecture.md` thread table's `hook-watchdog` row becomes "one wake per burst of keys; while ticking, no work per key event and one batched read (`GetRawInputBuffer`) per tick, 4 Hz"; the KS-08 section's cost sentence likewise; the errors page already has the stop line `[INFO] hook watchdog stopped after N wakes for M raw key events`.

### Follow-ups from Phase 2 (Q4)

- FU-1: an uninstall guard in `App::run`, so that a main-thread panic removes the hooks before the bounded stops (cut R10).
- FU-2: raw input as the "input left this desktop" signal (2.3 F5), now that the sink exists.
- FU-3: the README tree has no `src/logging/` entry (pre-existing).
- FU-4 (roadmap item 3.5, hot-path budget; ruling L2, fix round 2): if manual check M6 shows that a key's raw input reaches the watchdog before its hook call, trim the user32 work of the watchdog's first pass after idle, which a keyboard callback running at the same time pays (bench row R16: about 50 us at the median above R12, once per burst of keys; `fix_r2.md`). If M6 shows raw input after the hook chain, the key's own callback never meets that pass, and FU-4 lapses.

## Manual checks (owner; agents never run the daemon)

- **M1** Precondition (M7): stop the owner's release daemon first (`swch exit`, then `swch status` says the daemon is not running), because the debug daemon cannot take the pipe while it runs. `cargo build --locked` (debug, not elevated). Read and note `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` (read only; absent is an answer). In a console: `$env:KS_DEBUG_HOOK_HOLD='ScrollLock'; .\target\debug\key-switch-rs.exe`. In a second console: `while(1){ "$(Get-Date -f HH:mm:ss.fff) $(.\target\debug\swch.exe status)"; Start-Sleep -m 250 }`. In Notepad hold `a` for 1 s, press ScrollLock once while holding it, keep holding `a` 3 s, release, then press CapsLock. Expected: at startup `[WARN] debug build: the keyboard callback holds 1.5 s at each ScrollLock key-down (KS_DEBUG_HOOK_HOLD)`; within 2 s of ScrollLock the poll shows `hook ok, reinstalls: 1, last ...`; `daemon.log` has `the keyboard hook got no key event for ... ms while keys were typed; reinstalled it (...)` and `[INFO] state reset: the keyboard hook was reinstalled`; CapsLock switches the layout afterwards; the counters line, if any, shows `main_state_refused=0`. Note the time between ScrollLock and the reinstall line, and whether an `answered before its reinstall` line came first (critic OQ2). `reinstalls: 0` with CapsLock working means Windows kept the hook: report it as evidence against the premise (R-3), not as a defect. Remapping dead with `reinstalls: 0` is a defect. Afterwards `swch exit` and start the release daemon again (`swch open`).
- **M2** Release daemon (`swch open`). Note `powercfg /requests` (read only). Leave the machine untouched 10 minutes. Expected: `powercfg /requests` lists nothing for key-switch-rs; the display turns off per the power plan if its timeout is 10 minutes or less; `daemon.log` has no watchdog line for those minutes.
- **M3** Release daemon. Type 5 minutes across programs, an elevated one (Task Manager) included. Expected: `swch status` stays `hook ok, reinstalls: 0`; `daemon.log` has exactly one `hook watchdog: raw keyboard input arrives` line for the run.
- **M4** Release daemon. Win+L, type the PIN or password, unlock, type. Expected: no reinstall line; `reinstalls` unchanged.
- **M5** Release daemon, `PromptOnSecureDesktop` = 1 (read only). Open a UAC prompt and press Tab and Esc in it; open the Ctrl+Alt+Del screen and type there. Expected: no reinstall line.
- **M6** (amended in fix round 2, ruling L2: it settles R-1 and cut Q6, whether a key's raw input reaches the watchdog before or after its hook call, which decides whether a real key ever pays bench row R16). See `fix_r2.md`, "Manual check M6 (amended)", for the full text. Part A: release daemon, then an AutoHotkey script started after it with `CapsLock::return`; hold CapsLock alone 3 s, then type normally 10 s. Expected: at most one reinstall line (R-1), none after. No reinstall line: raw input comes after the hook chain. One line: part B decides. Part B: the debug build with `KS_DEBUG_HOOK_HOLD=F9`; the console's UTC time of F9 against `daemon.log`'s `[INFO] hook watchdog: raw keyboard input arrives`.
- **M7** Release daemon running: `swch status`, then `swch open`. Expected: `running (N bindings live; hook ok, reinstalls: 0)`, then `swch: daemon is already running`.
- **M8** As M1: `swch exit`. Expected: the process ends within 3 s; no `the hook watchdog did not stop within 1 s` line.

## Open points

1. **Needs a ruling (latency owner decision): R14 misses D11's bound.** R14 p50 22.2-22.8 us against R12 4.7-4.8 us; R14/settled within 0.5 us of R12. A keyboard callback that runs while the watchdog thread handles a raw event waits about 18 us. Whether that overlap happens in the daemon depends on the order of a key's raw input and its hook call (Q6, manual check M6). Options (a) accept on the argument that raw input follows the hook chain; (b) batch raw input at the ticks (wait without `QS_RAWINPUT` while ticking, drain before each tick), so that the watchdog does no per-key work; (c) raw input on the main thread (R15: about 3.9 us per key, in sequence). The developer did not choose.
2. Commit `58c4370`'s message says "naming debug_h::" where it means `debug_hold::` (a typo; not amended, per the git rules).
3. `HookWatchdog::start` gives up after 5 s; a thread that sends its ready message in the same instant runs on unstopped until the process ends (the executor's start has the same window).
4. Integration tests link the non-test library, where the system installer is real; the S4 scan keeps every file under `tests/` from naming an installer call or `core::app`.
5. Pre-existing warnings: none in the gates.
