# Context pack, round 1: p2-2.4-watchdog (roadmap 2.4 Hook watchdog, KS-08)

For the tester and the code-reviewer. Facts with `file:line` only; inferred items are marked UNVERIFIED.
Paths are relative to `D:/key-switch-rs/key-switch-rs` unless they start with `D:/tmp`.

## Scope and delta

- Branch `feat/hook-watchdog`, HEAD `4a90d44`; `master` = merge base = `fa20b30`. Tree clean; `git worktree list` shows only the lane tree.
- Binding inputs: `D:/tmp/ks-orch/p2-2.4-watchdog/spec.md`, `cut.md` (D1-D13; commit plan `cut.md:470-595`; acceptance map `cut.md:597-616`; manual checks `cut.md:618-629`; risks `cut.md:631-677`; developer addendum `cut.md:704-746`), `rulings.md` (S1-S4, I1, I3, M1-M7, Q1-Q8; wins over the cut), `critique.md`, `impl.md`, `bench-dev.md`.
- 28 commits `master..HEAD` (oldest first; red = test-only commit that must fail):

| # | Commit | Subject |
|---|---|---|
| 1 | 1aad2b1 | Tests: MovingQuery's fake windows are no multiple of 4 (2.3 P1) |
| 2 | 53b2499 | Engine: the winner's doc names the foreground cache (2.3 P2) |
| 3 | fcfd66e | Platform: not_a_timer states its invariant |
| 4 | 4f2d406 | README: the foreground.rs line of the tree (2.3 P3) |
| 5 | 29060a1 | Tests: no test build compiles a hook installer (red) |
| 6 | 2904d04 | Hook: the hook installers sit behind a seam that no test build compiles |
| 7 | 62d79da | Tests: a refused re-entry of the main state is counted (red) |
| 8 | ef27fa4 | Hook: count the refused re-entries of the main state |
| 9 | 6c18929 | Tests: the keyboard callback counts every call (red) |
| 10 | c914a9f | Hook: a heartbeat of the keyboard callback |
| 11 | bdf1480 | Tests: a reinstalled hook resets the engine (red) |
| 12 | 6e5717b | Engine: reset after a hook reinstall |
| 13 | a8aa0f7 | Tests: the main thread reinstalls only a silent, installed hook (red) |
| 14 | 2fcac6b | Hook: reinstall the keyboard hook on the main thread |
| 15 | 3505484 | Tests: the watchdog's decision (red) |
| 16 | a09ce5a | Hook: the watchdog's decision |
| 17 | 87ea196 | Tests: the hook watchdog thread (red) |
| 18 | 2b42857 | Hook: the hook watchdog thread |
| 19 | 2c624d1 | Tests: swch status shows the hook health (red) |
| 20 | 0c1153d | IPC: swch status shows the hook health |
| 21 | 96a43dc | Tests: a panic is logged within a bounded wait (red) |
| 22 | a013ee2 | Logging: a panic hook writes the message through the log writer |
| 23 | 58c4370 | Tests: the debug hold exists only in debug builds (red) |
| 24 | c9aa86e | Hook: a debug-build hold of the keyboard callback for the watchdog's manual check |
| 25 | ea622e2 | Bench: R14 key path with the watchdog running, R15 main-thread cost of a raw input dispatch |
| 26 | 0cd75d8 | Docs: the changelog names the bench rows R14 and R15 |
| 27 | 381da55 | Hook: the watchdog's docs state what its work costs a callback at the same time |
| 28 | 4a90d44 | Tests: the watchdog thread has no top-level window (critic OQ3) |

Each commit's parent is the row above it (row 1's parent is `fa20b30`).

`git diff --stat master...HEAD`: 28 files, +4838 / -161:

```
CHANGELOG.md 92 | Cargo.toml 1 | README.md 38 | book/src/config/errors.md 13 | src/bench.rs 210
src/core/app.rs 78 | src/engine/keys.rs 7 | src/engine/mod.rs 13 | src/engine/scenarios.rs 68
src/engine/testing.rs 21 | src/engine/winner.rs 5 | src/hook/debug_hold.rs 109 (new)
src/hook/health.rs 372 (new) | src/hook/installer.rs 556 (new) | src/hook/ipc_server.rs 53
src/hook/keyboard_hook.rs 364 | src/hook/keyboard_hook_callback.rs 124 | src/hook/main_thread.rs 440
src/hook/mod.rs 5 | src/hook/watchdog/decision.rs 664 (new) | src/hook/watchdog/mod.rs 1258 (new)
src/logging/counters.rs 50 | src/logging/mod.rs 24 | src/logging/panic.rs 339 (new)
src/logging/sink.rs 2 | src/logging/writer.rs 15 | src/platform/timers.rs 2 | tests/ipc_roundtrip.rs 76
```

## File map

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/hook/installer.rs:29-51` | trait `Installer` (keyboard set/unhook, WinEvent set/unhook, raw keyboard register/unregister) | S1 seam |
| `src/hook/installer.rs:55-58` | `struct System`, `const SYSTEM` | the daemon's installer |
| `src/hook/installer.rs:60-159` | `#[cfg(not(test))] mod win32`: the only `SetWindowsHookExW` (90-92), `UnhookWindowsHookEx` (101), `SetWinEventHook` (109-119), `UnhookWinEvent` (127), `RegisterRawInputDevices` (141, 156) | S1/S4; unsafe + SAFETY |
| `src/hook/installer.rs:162-194` | `#[cfg(test)] impl Installer for System`: `unreachable!` at every method | S1 |
| `src/hook/installer.rs:198-342` | test-only `Call`, `FakeState`, `FakeInstaller` (numbered handles, `refuse_next_keyboard_hook` 247, `remove_on_its_own` 253, `keyboard_counts` 270) | S3 fake |
| `src/hook/installer.rs:344-556` | tests: `the_system_installer_panics_at_every_call_under_test` (352), S4 scan `the_hook_installers_are_compiled_out_of_the_tests` (467-555) | S4 |
| `src/hook/health.rs:14-136` | `Heartbeat` `repr(align(64))` (14-15), `HealthState` (19-39), `callback_seen` (57-61), `callbacks` (64-66), `set/hook_installed` (70-77), `set_watched` (81-83), `record_reinstall` (89-99), `record_failed_attempt` (102-104), `hook_answered` (109-113), `set_timer_failing` (117-121), `snapshot` (124-135) | D2, D7, I1, I3, M3; orderings |
| `src/hook/health.rs:146-221` | `pub struct HookHealth` (5 pub fields) + `Display` (163-183), `ReinstallRequest` (188-196), `ReinstallOutcome` + `Display` (200-221) | status text; R6 |
| `src/hook/health.rs:223-372` | 8 tests | status table, M3, I1 status, cache line |
| `src/hook/watchdog/decision.rs:1-232` | `#![forbid(unsafe_code)]` (43); constants `SILENCE` 1 s, `TICK` 250 ms, `ACTIVE_FOR` 2 s, `MAX_BACKOFF_SHIFT` 6, `MIN_EVIDENCE` 2 (51-59); `Verdict` (63-73); `Watchdog` (77-99); `raw_event` (120-128), `next_tick` (131-133), `window` (136-138), `tick` (141-183), `keeps_ticking` (188-195), `answered` (202-211), `not_sent` (215-217), `declined` (221-224), `start_over` (226-231) | D3 pure decision, anti-loop rule |
| `src/hook/watchdog/decision.rs:234-664` | `Feeder` fake clock (245-325) and 15 decision rows (334-664) | acceptance rows |
| `src/hook/watchdog/mod.rs:1-48` | module docs incl. "Cost" (22-35) | M6, R14 statement |
| `src/hook/watchdog/mod.rs:80-134` | `RequestReinstall` (80-95), `TickTimer` (99-112), `Plan`/`plan` (116-134) | I1 |
| `src/hook/watchdog/mod.rs:137-304` | `Context`, `Ticker`: `raw_event` (176-185, I3 gate 177), `fired` (189-191), `wake_by` (197-202), `pass` (208-223), `take_answer` (225-241), `ask` (247-272, M2 268), `arm` (278-303) | I1, I3, M2, M3 |
| `src/hook/watchdog/mod.rs:306-449` | `SYNTHETIC_RAW` (test), `Source`, `HookWatchdog` start/spawn/stop/Drop (348-449), `START_TIMEOUT` 5 s (358), `STOP_TIMEOUT` 1 s (360) | thread lifecycle, R10 |
| `src/hook/watchdog/mod.rs:451-580` | `SinkWindow` (453-490), `Watched` (494-507), `RawInputRegistration` (512-527), `run_watchdog` (532-580) | guards and drop order |
| `src/hook/watchdog/mod.rs:585-708` | `run_loop` (585-622), `wait_ms` (626-629), `pump` (636-662), `KEYBOARD_OVERRUN_MAKE_CODE` (666), `raw_key_event` (671-698), `qualifies` (706-708) | D5/D6; unsafe; no key data |
| `src/hook/watchdog/mod.rs:710-1258` | tests: Recorder (736-778), FakeTimer (780-807), `Drive` loop driver (820-911), 13 tests incl. two scans (1173, 1231) | I1, I3, M2, scans |
| `src/hook/keyboard_hook.rs:55-71,127` | field `health: HealthState` (57), init (71), accessor `health()` (127) | D2 |
| `src/hook/keyboard_hook.rs:153-205` | `install` -> `install_with(&SYSTEM)`; `install_with` sets `hook_installed` (169); WinEvent failures roll back via `uninstall_with` (181, 196) | S1, I3 |
| `src/hook/keyboard_hook.rs:207-254` | `is_installed` doc (207-213); `uninstall`/`uninstall_with` (220-254), clears `hook_installed` (249) | S1, I3 |
| `src/hook/keyboard_hook.rs:256-323` | `reinstall_keyboard_hook(_with)` (270-290), `Reinstalled` (295-300), `OldHook` + `Display` (304-323) | D4, S3 |
| `src/hook/keyboard_hook.rs:331-335` | `impl Drop for KeyboardHook` calls `self.uninstall()` (SYSTEM) | S1 interplay |
| `src/hook/keyboard_hook.rs:642-806` | tests: helpers `leaked`/`installed`/`removed_keyboard_hook` (642-676), 5 S3 tests (678-806) | S3 |
| `src/hook/keyboard_hook_callback.rs:352-401` | callback: `code < 0` (357-362), heartbeat (367), switch gate (368-373), timing start (378), deref (382), debug hold (383-384), decide (385-392), `record_callback` (393), outcomes (394-400) | R1 |
| `src/hook/keyboard_hook_callback.rs:851-893,1325-1335,1387-1430` | heartbeat test; `the_gesture_path_starts_no_thread` doc excluding `src/hook/watchdog`; debug-hold cfg scan | R1, D10 |
| `src/hook/main_thread.rs:212-228` | `with_main`: refusal bumps `Counter::MainStateRefused` (220) | D8 |
| `src/hook/main_thread.rs:294-303` | `MainState::hook_reinstalled` -> `reset_engine(HookReinstalled)` | D13 |
| `src/hook/main_thread.rs:452-462,544-563` | `Request::ReinstallHook` (458-461); `request_reinstall` (547-563, `try_send`, Full->`Busy`, Disconnected->`ShuttingDown`, then `wake`) | D4 |
| `src/hook/main_thread.rs:806-886` | `serve` arm (825-828), `reinstall_if_silent` (840-872), `dispatch_sent_messages` (879-886) | D4, anti-loop rule 3 |
| `src/hook/main_thread.rs:1019-1060,1801-1836,1854-2124` | refusal tests; P1 test; reinstall tests and helpers | D8, P1, D4 |
| `src/core/app.rs:87-92` | `DaemonHandle::status` adds `hook: self.hook.health().snapshot()` (91) | D7 |
| `src/core/app.rs:198-262,300-338` | `App::run` order: executor (205), `MainThread::new` (210), attach (216), `HookWatchdog::start` (220-229), `ControlWindow::create` (235), debug `arm_from_env` (239-240), `install` (245), ... `main.run` (309), `drop(main)` (313), `uninstall()?` (316), `drop(control_window)` (320), `stop_watchdog` (321), `executor.stop()` (322); `stop_watchdog` (332-337) | I3, R10 |
| `src/core/app.rs:410,469-492` | `daemon_handle_switches_remapping_through_the_main_thread`; I3 order scan | I3 |
| `src/hook/ipc_server.rs:53-54,190-198,223-316` | `DaemonStatus.hook` (54); reply format (196-198); `HOOK_OK` (227); `dispatch_status_appends_the_hook_health` (297-316) | D7, protocol |
| `src/hook/debug_hold.rs:1-109` | `ENV_VAR` (23), `HOLD` 1500 ms (26), `ARMED: AtomicU16` (31), `arm_from_env` (35-50), `parse` (53-55), `should_hold` (59-63), `hold_if_asked` (68-72), 2 tests (74-109) | D10, Q3 |
| `src/hook/mod.rs:3-12` | `#[cfg(debug_assertions)] pub(crate) mod debug_hold;` (3-4), `pub mod health;` (5), `pub(crate) mod installer;` (6), `pub(crate) mod watchdog;` (12) | module visibility |
| `src/logging/panic.rs:1-150` | `REPORT_BUDGET` 250 ms (39), `EPISODE` 1 s (42), `TARGET` (45), `WRITER_THREAD` (48), `BUDGET_END` TLS (50-53), `install` (57-59), `install_with` test (63-66), `report` (70-97), `panic_text` (104-124), `budget_end` (139-144), `report_on` (148-150) | D9 |
| `src/logging/panic.rs:152-339` | 5 tests + `panic_child` (279-299); child-process test (302-338) | D9 |
| `src/logging/mod.rs:28,107-131,175-180,459-466` | `pub(crate) mod panic` (28); `init` installs the hook after `set_max_level` (120-122); `utc_timestamp` (175-180); hook-path scan entries `health.rs fn callback_seen(`, `debug_hold.rs fn hold_if_asked(` / `fn should_hold(` (462-466) | D9, D7, R1 scan |
| `src/logging/writer.rs:99,133-140` | `send_by` now `pub(crate)` (99); `flush_on` over new `flush_by` (133-140) | D9 |
| `src/logging/counters.rs:17,44-58,63-75,88-93,150-156` | `SLOTS` 11 (17); 4 new variants (47-57); `ALL`; names; `#[cfg(test)] get` (Acquire) | D8, M2, I1 |
| `src/logging/sink.rs:120` | `write_timestamp` now `pub(crate)` | D7 |
| `src/engine/mod.rs:391-394,403,633-636,657-659` | `ResetReason::HookReinstalled` + Display + doc + reset arm shared with `RemappingSwitched` | D13 |
| `src/engine/keys.rs:112-118` | `resync_from_os` doc names the reinstall | D13 |
| `src/engine/testing.rs:67-71,490-505` | `Scenario::hook_reinstalled` (498-505) | D13 harness |
| `src/engine/scenarios.rs:2340-2407` | 4 reinstall scenarios | D13 |
| `src/engine/winner.rs:56-58` | doc (2.3 P2) | leftover |
| `src/platform/timers.rs:71` | `expect("invariant: ...")` | leftover |
| `src/bench.rs:59-75,430-480,482-590,895-1000` | row docs; `Unanswered` requester + R14 (`r14_ordinary_key_with_the_watchdog` 453-480); R15 window class/proc/row (482-590); `latency_bench` ignore string (895), rows R14 (985), R14/settled (990), R15 (995) | D11 |
| `tests/ipc_roundtrip.rs:22,51-78,102-170` | `HookHealth` import (22); `TestControl` with `hook` (51-78), `HOOK_OK` (59); `status_off_on_status_round_trip_without_a_hook` (102-146); `status_reports_the_hook_health_over_the_pipe` (148-170) | D7 acceptance |
| `Cargo.toml:46` | `"Win32_UI_Input"` feature added | D5 |
| `README.md:68-76,779,792,859-862,880-881,905,939,979-993` | tree, typical session, `swch status` row, log-file sentences, Technologies, watchdog bullet, Known Limitations | docs |
| `CHANGELOG.md:120,173-249,615-622,1002-1006` | Added / Changed / Fixed entries | docs |
| `book/src/config/errors.md:210-219,230-231` | new log-line rows; counters row; panic row | docs |

## Facts

### Safety seam (S1-S4)
1. Every non-test call of `SetWindowsHookExW`, `UnhookWindowsHookEx`, `SetWinEventHook`, `UnhookWinEvent`, `RegisterRawInputDevices` is inside `#[cfg(not(test))] mod win32` (`src/hook/installer.rs:60-159`); other occurrences under `src` are string literals (`src/hook/keyboard_hook.rs:184,199`) - verified with grep of `src` and `tests`.
2. Under `cfg(test)` every `Installer for System` method is `unreachable!` (`src/hook/installer.rs:170-194`).
3. `KeyboardHook::install`/`uninstall`/`reinstall_keyboard_hook` delegate to `*_with(&installer::SYSTEM)` (`src/hook/keyboard_hook.rs:153-155,220-222,270-272`); `RawInputRegistration` registers and unregisters through `installer::SYSTEM` (`src/hook/watchdog/mod.rs:515-526`).
4. `impl Drop for KeyboardHook` calls `self.uninstall()` (`src/hook/keyboard_hook.rs:331-335`); `uninstall_with` calls the installer only for non-null handles (`src/hook/keyboard_hook.rs:231,240,250`). Test hooks that hold fake handles are `Box::leak`ed (`src/hook/keyboard_hook.rs:642-647`, `src/hook/main_thread.rs:1912`).
5. The S4 scan reads every `.rs` under `src` recursively, the lines before the tests module with comments and strings stripped (`src/hook/installer.rs:397-466,478-509`), and the top-level entries of `tests/` (non-recursive `read_dir`), rejecting an installer name or `core::app` there (`src/hook/installer.rs:511-520`). It requires each of the 5 names inside the win32 block (`:527-533`).
6. The synthetic watchdog source skips the raw input registration (`src/hook/watchdog/mod.rs:554-561`) and the desktop check (`:322-329`).
7. Integration tests link the non-test library, where `SYSTEM` is real (`impl.md:168`, open point 4); `tests/ipc_roundtrip.rs` constructs `TestControl` with `HookHealth` and names no installer (verified by grep).

### Keyboard callback (R1)
8. Order in `keyboard_hook_callback`: `code < 0` -> `CallNextHookEx` (`src/hook/keyboard_hook_callback.rs:357-362`); `hook.health().callback_seen()` (`:367`); `!is_enabled()` -> `CallNextHookEx` (`:368-373`); `started = Instant::now()` (`:378`); deref (`:382`); debug hold (`:383-384`); `decide_event` (`:385-392`), whose sentinel check `is_injected_event` is at `:222`; `record_callback(started.elapsed())` (`:393`); `LRESULT(1)` (`:395`) or `CallNextHookEx` (`:400`).
9. `callback_seen` is `fetch_add(1, AcqRel)` on `Heartbeat(AtomicU64)` with `#[repr(align(64))]` (`src/hook/health.rs:14-15,57-61`); pinned by `the_heartbeat_has_a_cache_line_of_its_own` (`src/hook/health.rs:367-371`). The heartbeat increment precedes the sentinel check (`:367` vs `:222`); cut D6 states injected callbacks are counted on purpose (`cut.md:260`).
10. The debug hold call is `#[cfg(debug_assertions)]` (`src/hook/keyboard_hook_callback.rs:383`), sits after `started` (`:378`) and before `record_callback` (`:393`), so a hold's 1.5 s falls inside the timed span that feeds `longest_callback_us` (read by the reinstall log line, `src/hook/main_thread.rs:859`).
11. `hold_if_asked` calls `std::thread::sleep(HOLD)` when `should_hold(ARMED.load(Acquire), ...)` (`src/hook/debug_hold.rs:68-72`); `should_hold` needs `armed != 0`, matching vk, and `WM_KEYDOWN | WM_SYSKEYDOWN` (`:59-63`). `ARMED` is stored only by `arm_from_env` (`:35-50`), called only at `src/core/app.rs:239-240` under `#[cfg(debug_assertions)]` (grep: one call site).
12. The hook-path no-`log::` scan lists `src/hook/health.rs` `fn callback_seen(` and `src/hook/debug_hold.rs` `fn hold_if_asked(`/`fn should_hold(` (`src/logging/mod.rs:462-466`).
13. `with_main`'s refusal branch bumps `Counter::MainStateRefused` (`src/hook/main_thread.rs:220`) next to the test-only thread-local `REFUSED` (`:196,221-222`).

### Reinstall on the main thread (D4)
14. `reinstall_keyboard_hook_with`: null handle -> `Ok(NotInstalled)` with no call (`src/hook/keyboard_hook.rs:280-282`); `set_keyboard_hook()?` first (`:283`); then `hook_handle.swap(new, AcqRel)` (`:284`); then `unhook_keyboard(previous)`, `Err` -> `OldHook::Gone(code)` (`:285-288`).
15. `reinstall_keyboard_hook_with` does not touch `health.hook_installed` (`src/hook/keyboard_hook.rs:276-290`).
16. `request_reinstall` uses `try_send` on the request queue and then `wake()`; `Full` -> `MainThreadError::Busy`, `Disconnected` -> `ShuttingDown` (`src/hook/main_thread.rs:547-563`); reply channel is `sync_channel(1)` (`:551`).
17. `serve`'s arm calls `reinstall_if_silent(hook, request, || hook.reinstall_keyboard_hook())` and replies with `try_send` (`src/hook/main_thread.rs:825-828`).
18. `reinstall_if_silent` order: `dispatch_sent_messages()` (`:845`); heartbeat `!= request.seen` -> log INFO "answered before its reinstall", `Alive` (`:846-849`); `replace()`: `NotInstalled` silent (`:853`); `Replaced` -> `record_reinstall(SystemTime::now())` (`:855`), WARN line with `silent_for` ms and `COUNTERS.longest_callback()` (`:856-860`), `with_main(MainState::hook_reinstalled)` then INFO "state reset" (`:861-863`), `Reinstalled`; `Err` -> `record_failed_attempt` + ERROR line, `Failed` (`:866-870`).
19. `dispatch_sent_messages` is one `PeekMessageW(.., PM_NOREMOVE | PM_QS_SENDMESSAGE)` with its result ignored (`src/hook/main_thread.rs:879-886`). That it dispatches a pending LL hook call is UNVERIFIED (cut Q7, `cut.md:692`).
20. `HookReinstalled` shares the `resync_from_os` arm with `RemappingSwitched` (`src/engine/mod.rs:657-659`); `MainState::hook_reinstalled` cancels no macro (`src/hook/main_thread.rs:296-302`).

### Decision (D3)
21. `tick` computes `answered = callbacks.wrapping_sub(last_callbacks)` and `arrived = raw_events.saturating_sub(raw_at_last_tick)` (`src/hook/watchdog/decision.rs:142-145`).
22. `decide` is computed before `evidence += arrived - explained` (`decision.rs:157-160`) (ruling M5); `explained = arrived.min(owed)`, then `owed = 0` (`:154-155`).
23. `decide = !outstanding && evidence >= 2 && now >= quiet_since + window()`; `window() = SILENCE * (1 << attempts.min(6))` (`decision.rs:136-138,157-159`).
24. The first raw event after idle (`next_tick.is_none()`) sets `next_tick = now + TICK`, `quiet_since = now`, `owed = 0` (`decision.rs:120-128`) (addendum deviation 3, `cut.md:733`).
25. `keeps_ticking`: outstanding, or a raw event within `ACTIVE_FOR`, or `evidence >= 2 && now < last_raw_at + window() + TICK` (`decision.rs:188-195`) (ruling M1).
26. `answered`: `Reinstalled`/`Failed` add 1 to `attempts`; every outcome calls `start_over` (evidence 0, owed 0, `quiet_since = now`, `raw_at_last_tick = raw_events`) (`decision.rs:202-211,226-231`); `declined` also calls `start_over` (`:221-224`); `not_sent` only clears `outstanding` (`:215-217`).

### Watchdog thread (D5, I1, I3, M2, M3)
27. `Ticker::pass` runs after every wake: `take_answer`, then a tick only if `next_tick <= now` by the clock, then `arm` (`src/hook/watchdog/mod.rs:208-223`); `run_loop` calls `pass` after every non-failing wake (`:620`).
28. `arm` re-arms from `watchdog.next_tick()` through `plan` every pass; a failed `set` bumps `WatchdogTimerNotSet` and sets `timer_failing` (`mod.rs:278-303`); `wake_by` returns the next tick as the wait timeout when no timer is armed (`:197-202,593-595`).
29. `Ticker::raw_event` ignores raw events while `!health.hook_installed()` (`mod.rs:176-179`) and logs `hook watchdog: raw keyboard input arrives` once (`:180-183`).
30. A tick that sees the heartbeat move calls `health.hook_answered()` (`mod.rs:211-217`), which clears `last_attempt_failed` (`src/hook/health.rs:109-113`).
31. `ask`: `input_here()` `Ok(true)` -> `requests.request`; `Err` -> `not_sent`; `Ok(false)` -> `declined`; `Err` of the check -> bump `WatchdogDesktopCheckFailed` + `declined` (`mod.rs:247-272`); the "asking again" WARN is logged only when the request was sent and `attempt >= 2` (`:255-261`).
32. `take_answer`: `Disconnected` -> `not_sent` (`mod.rs:236-239`).
33. `run_watchdog` declares `window` (`mod.rs:550`), `_registration` (`:554`), `timer` (`:562`), `_watched` (`:566`), then `ticker` (`:578`) which owns the timer. By Rust's reverse-declaration drop order, `ticker` (and its timer) drops before `_watched`, then `_registration`, then `window`. The addendum says "`Watched` drops first on the thread (before the timer, the registration and the window)" (`cut.md:738`).
34. `Watched::new` sets `watched = true` before the ready message (`mod.rs:566,569`); its `Drop` stores false (`:503-507`).
35. `spawn` waits `START_TIMEOUT` (5 s) for ready (`mod.rs:400-403`); a thread whose `ready.send` fails returns (`:569-571`) (open point 3, `impl.md:167`).
36. `stop_thread` posts `WM_QUIT` and waits `STOP_TIMEOUT` (1 s) on `done`; on timeout it returns false without joining (`mod.rs:426-442`).
37. `pump`: `WM_INPUT` addressed to the sink window -> `raw_key_event`; every message is then `DispatchMessageW`ed (`mod.rs:645-660`).
38. `raw_key_event` checks only `read == u32::MAX` (bump `RawInputUnread`, false) and `dwType == RIM_TYPEKEYBOARD` before reading the union (`mod.rs:687-697`); `qualifies` = `make_code != 0xFF && (1..=254).contains(&vkey)` (`:706-708`).
39. The `WatchdogTimerNotSet` doc says the retry comes at "its next wake, which the next raw key event brings" (`src/logging/counters.rs:51-53`); `Ticker::wake_by`'s doc and code bring the retry at the tick through the wait's timeout (`src/hook/watchdog/mod.rs:193-202,593-595`).

### Health and status (D7)
40. `record_reinstall`: `last_reinstall_ms.store(Release)` (time, `max(1)`), then `reinstalls.fetch_add(1, AcqRel)`, then `last_attempt_failed.store(false)` (`src/hook/health.rs:89-99`); `snapshot` loads the count first, then the time (`:124-135`).
41. Display precedence: `not watched` > `reinstall failed` > `watchdog timer failing` > `ok`; then `, reinstalls: N`; then `, last <utc>` if any (`src/hook/health.rs:167-182`).
42. Reply format: `format_ok(&format!("{state} ({bindings} bindings live; {hook})"))` (`src/hook/ipc_server.rs:196-198`). `swch` prints the message unchanged (`src/bin/swch.rs:148-152`); `swch open`'s probe accepts any `Ok(_)` (`src/bin/swch.rs:77-80`).
43. `HookHealth` is `pub` with 5 `pub` fields in `pub mod health` (`src/hook/health.rs:146-161`, `src/hook/mod.rs:5`); `DaemonStatus.hook` is `pub` (`src/hook/ipc_server.rs:54`).

### Panic hook (D9)
44. `logging::init` calls `panic::install()` after `log::set_max_level`, only when it installed the logger (`src/logging/mod.rs:119-122`); `install` sets the hook with `LOGGER.writer.get()` (`src/logging/panic.rs:57-59`).
45. `report` uses `std::thread::current()` (`panic.rs:71`) (deviation 13, `cut.md:743`); on the `log-writer` thread or with no writer it calls `write_to_stderr` and does not wait (`:83-96`); otherwise deadline = `budget_end(now, BUDGET_END)` and `report_on` = `send_by(.., deadline)` then `flush_by(.., deadline)` (`:85-93,148-150`).
46. `budget_end`: previous end if `now < end + EPISODE`, else `now + REPORT_BUDGET` (`panic.rs:139-144`). The hook does not chain to the previous hook (`panic.rs:57-59,64-66`).
47. `panic_child` returns at once unless `KS_PANIC_CHILD` is set (`panic.rs:279-285`); the parent runs `current_exe()` with `--exact logging::panic::tests::panic_child --test-threads=1`, stdio null, 10 s cap (`:302-327`).

### Startup and shutdown (I3, R10)
48. `App::run`: `HookWatchdog::start` (`src/core/app.rs:220`) precedes `ControlWindow::create` (`:235`) and `hook.install()` (`:245`); a start error only warns (`:222-228`).
49. Shutdown: `drop(main)` (`app.rs:313`), `hook.uninstall()?` (`:316`), `drop(control_window)` (`:320`), `stop_watchdog(watchdog)` (`:321`), `executor.stop()` (`:322`). On an early `?` return, locals drop in reverse order: `control_window`, then `watchdog` (`HookWatchdog::drop` -> `stop_thread`, `src/hook/watchdog/mod.rs:445-449`), then `executor`.

### Bench (D11)
50. R14 feeds `post_synthetic_raw` (a thread message, no `GetRawInputData`) before each timed callback, with `health.set_hook_installed(true)` on the global hook's health during the row (`src/bench.rs:453-480`); the synthetic source is matched at `src/hook/watchdog/mod.rs:332-343,649-653`.
51. D11 bound for R14: `p50 <= R12 p50 + 300 ns; p99 <= R12 p99 + 2 us` (`cut.md:369`). Developer runs: R12 p50 4.7-4.8 us, R14 p50 22.2-22.8 us, R14/settled within 0.5 us of R12, R15 3.9 us (`impl.md:121-134`; raw data `D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev.md`). Runs 1-3 (HEAD) and master run 1 were under Remote Desktop (`impl.md:121`).

### Naming drift between cut and tree
52. `cut.md:508` names `a_reinstall_of_a_hook_that_was_never_installed_installs_nothing`; the tree has `a_reinstall_of_a_hook_that_is_not_installed_installs_nothing` (`src/hook/keyboard_hook.rs:733`) and `the_daemons_reinstall_of_a_hook_never_installed_makes_no_call` (`:778`).
53. `cut.md:537` says `counter_names_are_snake_case` lists 9 names; `SLOTS` is 11 (`src/logging/counters.rs:17`) (deviation 6, `cut.md:736`).
54. `cut.md:590` row expects 6 requests over 120 s; the test runs 250 s and asserts 8 (`src/hook/watchdog/decision.rs:450-467`) (deviation 11).

## Red-first replay and acceptance proofs

Replay per `.claude/agents/tester.md:70-85` (scratch worktree, own `CARGO_TARGET_DIR`). For every red below, the code under test does not exist on `master` `fa20b30`, so master is compile-red for all of them except row R1. "Red on" names the red commit itself, which is the green commit's parent and carries the test without the implementation. `tester.md:81` records a compile-red as `compile-red`; its behaviour then rests on the mutation named.

| # | Tests (path under `--lib`) | Red on | Expected red (commit body) | Green on | Behavioural red / mutation |
|---|---|---|---|---|---|
| R1 | `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests` | `29060a1` (also on `fa20b30` with `src/hook/installer.rs` and its `mod installer;` line copied from 29060a1) | assertion: `src/hook/installer.rs has no #[cfg(not(test))] block mod win32 {` and `7 installer calls a test build compiles: src/hook/keyboard_hook.rs:153: SetWindowsHookExW( ...` | `2904d04` | assertion-red (not compile) |
| R2 | `hook::main_thread::tests::a_refused_reentry_is_counted_in_main_state_refused`; `logging::counters::tests::counter_names_are_snake_case` | `62d79da` | compile: `E0599 no method named get ... Counters`; `no variant ... MainStateRefused` | `ef27fa4` | compile-red; mutation: drop the bump at `main_thread.rs:220` |
| R3 | `hook::keyboard_hook_callback::tests::the_keyboard_callback_counts_every_call_also_while_remapping_is_off` | `6c18929` | compile: `E0599 no method named health ... &'static KeyboardHook` | `c914a9f` | move the bump after the switch gate: `left: 3, right: 5` (`cut.md:495`) |
| R4 | 4 scenarios `engine::scenarios::a_swallowed_key_released_while_the_hook_was_gone_fires_at_its_next_press_after_the_reinstall`, `..a_passed_key_held_through_a_reinstall_keeps_its_repeats_and_key_up_passing`, `..a_hold_armed_before_the_hook_was_gone_fires_nothing_after_the_reinstall`, `..a_key_pressed_while_the_hook_was_gone_passes_its_key_up_after_the_reinstall` | `bdf1480` | compile: `E0599 no method named hook_reinstalled ... Scenario` | `6e5717b` | `HookReinstalled => {}`: `scenario: down F13 at +0 ms: expected to fire [SwitchLanguage], fired []` (`impl.md:22`) |
| R5 | `hook::keyboard_hook::tests::{a_reinstall_installs_the_new_hook_before_it_removes_the_old_one, a_failed_reinstall_keeps_the_old_hook, a_reinstall_of_a_hook_that_is_not_installed_installs_nothing, a_reinstall_reports_an_old_hook_that_windows_removed_as_gone, the_daemons_reinstall_of_a_hook_never_installed_makes_no_call}`; `hook::main_thread::tests::{a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing, a_reinstall_request_for_a_hook_that_is_not_installed_installs_nothing, a_reinstall_request_to_an_ended_main_thread_fails_at_once, a_reinstall_resets_the_engine_and_keeps_the_macros, a_failed_reinstall_keeps_the_old_hook_and_resets_nothing, a_reinstall_that_finds_no_hook_resets_nothing, a_hook_that_answered_is_kept_without_a_replace}` | `a8aa0f7` | compile: `E0425 reinstall_if_silent`; `E0432 ReinstallOutcome, ReinstallRequest / OldHook, Reinstalled`; `E0599 reinstall_keyboard_hook_with / request_reinstall` | `2fcac6b` | S3 mutations (ruling S3; developer: each caught, `cut.md:711`): unhook first; handle nulled before the install; no null guard; old hook left installed |
| R6 | 14 rows `hook::watchdog::decision::tests::*` (all but `raw_events_before_an_answer_are_no_evidence_after_it`) | `3505484` | compile: `E0425 Watchdog`, `SILENCE`; `E0433 Verdict` | `a09ce5a` | developer mutations each fail a row (`impl.md:26`): M5 order swap, silence restart, owed credit, both M1 clauses, backoff, outstanding gate, rule 4; first-raw `quiet_since` removed -> `a_key_after_idle_waits_a_full_second_of_silence` red (`cut.md:733`) |
| R7 | `hook::watchdog::tests::{an_early_timer_signal_does_not_lose_the_tick, a_timer_that_refuses_once_is_set_again_at_the_next_pass, a_reply_of_the_main_thread_is_taken_at_the_next_pass, raw_input_before_the_hook_is_installed_is_no_evidence, a_failed_desktop_check_declines_and_is_counted, only_keyboard_events_the_hook_can_see_count, the_watchdog_thread_starts_and_stops_without_raw_input, synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request, synthetic_raw_events_with_a_live_heartbeat_send_no_request, the_watchdog_injects_no_input, no_key_data_leaves_the_raw_input_read}`; `core::app::tests::the_hook_watchdog_starts_before_the_hooks_and_stops_after_them`; `counter_names_are_snake_case` (11 names) | `87ea196` | compile: `E0405 RequestReinstall / TickTimer`; `E0425 qualifies`, `Ticker`; `E0433 HookWatchdog` | `2b42857` | I1 rulings' red-first; developer mutations caught (`impl.md:28`): arm only after a tick (both I1 tests), no timeout fallback (refusal test), no install gate, uncounted desktop failure, reply never taken, VK 255 kept |
| R8 | `hook::health::tests::{the_hook_health_reads_ok_not_watched_or_failed_with_its_reinstalls, a_reinstall_is_recorded_with_its_time, a_failed_attempt_reads_reinstall_failed_until_the_next_reinstall, a_failed_attempt_clears_once_the_hook_answers, a_tick_timer_that_cannot_be_set_is_not_hook_ok}`; `hook::ipc_server::tests::dispatch_status_appends_the_hook_health`; `hook::watchdog::tests::a_failed_reinstall_is_no_longer_reported_once_the_hook_answers`; `--test ipc_roundtrip status_reports_the_hook_health_over_the_pipe` and the updated `status_off_on_status_round_trip_without_a_hook` | `2c624d1` | compile (`cargo test --locked --no-run`): `E0432 HookHealth` (lib and ipc_roundtrip); `E0560 DaemonStatus has no field hook`; `E0599 snapshot` | `0c1153d` | compile-red; no developer mutation named |
| R9 | `logging::panic::tests::{a_panic_line_names_the_thread_the_place_and_the_message, a_nested_panic_on_one_thread_shares_the_budget, a_stuck_writer_holds_a_panic_report_at_most_its_budget, a_panic_report_reaches_the_log_file_before_it_returns, a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook}` | `96a43dc` | compile: `E0425 panic_text / budget_end / report_on / REPORT_BUDGET` | `a013ee2` | hook writing nowhere: child test fails, `daemon.log` missing (`impl.md:32`) |
| R10 | `hook::debug_hold::tests::{the_hold_key_is_named_like_a_config_key, only_a_key_down_of_the_armed_key_holds}`; `hook::keyboard_hook_callback::tests::the_debug_hold_is_compiled_only_into_debug_builds` | `58c4370` | compile: `E0425 parse / should_hold`; the scan alone with the module empty: `no call site of debug_hold:: in src/hook/keyboard_hook_callback.rs` / `... in src/core/app.rs` | `c9aa86e` | scan is assertion-red with an empty module |

Tests added in green commits (no red-first): `the_system_installer_panics_at_every_call_under_test` (2904d04), `the_heartbeat_counts_each_call` and `the_heartbeat_has_a_cache_line_of_its_own` (c914a9f), `every_reinstall_outcome_has_its_own_text` (2fcac6b), `raw_events_before_an_answer_are_no_evidence_after_it` (a09ce5a; developer mutation: `left: [1000, 3250]`, `right: [1000]`, `impl.md:26`), `the_watchdog_thread_has_no_top_level_window` (4a90d44). Row 1 `a_return_signal_reads_the_foreground_window_again` (1aad2b1) is a re-check: green, and red with `input_returned` removed from `answer_input_signals` (`impl.md:11`, `cut.md:480`).

Commands (developer's, `impl.md:100-101`; the gate per CLAUDE.md):

```
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked --no-fail-fast                        # tester owns; developer ran only the subset below
cargo test --locked --lib -- hook:: logging:: engine:: core::app platform::timers   # dev: 495 passed, 2 ignored
cargo test --locked --test ipc_roundtrip                  # dev: 7 passed
cargo build --release --locked ; cargo test --release --locked --lib --no-run       # no debug_assertions: debug_hold absent
KS_BENCH_OUT=<file> KS_BENCH_LABEL=<parent|head> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1   # src/bench.rs:895; tester.md 4b
```

Real-clock tests: `synthetic_raw_events_with_a_*` run 3 s each (`src/hook/watchdog/mod.rs:1095-1138`), the request window 1.0-2.0 s (`:1121-1124`); the child-panic test waits up to 10 s (`src/logging/panic.rs:316-325`).

Acceptance (spec) -> proof:

| Spec line | Proof | Kind |
|---|---|---|
| Silent callback > 1 s: exactly one reinstall | `decision::tests::key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall` (`decision.rs:334-346`: 1 request at 1000-1500 ms, `seen == 0`, `silent_for >= SILENCE`, attempt 1); `a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more` | unit |
| Callback keeps up: none | `a_callback_that_keeps_up_asks_for_nothing` (3 variants), `raw_events_of_callbacks_seen_at_the_last_tick_are_explained`, `evidence_is_checked_before_this_intervals_raw_events_are_added` | unit |
| Idle: none | `idle_asks_for_nothing_and_stops_ticking`, `a_single_raw_event_without_a_callback_asks_for_nothing`, `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops` | unit |
| Busy main thread: no loop | `a_busy_main_thread_gives_no_reinstall_loop`, `reinstalls_that_bring_no_callback_back_are_spaced_out` (8 requests over 250 s), `main_thread::tests::a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing`, `a_hook_that_answered_is_kept_without_a_replace` | unit |
| Debug hold 1.5 s -> `reinstalls: 1` within 2 s | manual M1 (`impl.md:154`); below the hook: `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`, `a_reinstall_resets_the_engine_and_keeps_the_macros`, `the_debug_hold_is_compiled_only_into_debug_builds` | manual + unit |
| 10 min idle: no injected input | scan `the_watchdog_injects_no_input` (`src/hook/watchdog/mod.rs:1173-1229`); grep `SendInput|keybd_event|mouse_event|KEYBDINPUT|INPUT_KEYBOARD` over `src/hook/watchdog src/hook/health.rs src/hook/installer.rs src/hook/debug_hold.rs` returns no match (run for this pack); manual M2 | scan + manual |
| `swch status` shows HookHealth; ipc_roundtrip covers it | `tests/ipc_roundtrip.rs::status_reports_the_hook_health_over_the_pipe` (`:148-170`), `::status_off_on_status_round_trip_without_a_hook` (`:102-146`), `ipc_server::tests::dispatch_status_appends_the_hook_health`, `health::tests::*`; manual M7 | integration + unit |
| Latency: no unjustified regression | bench rows R1-R15 per `cut.md:356-370` and `rulings.md` "Latency" | bench (tester 4b) |
| Never two hooks, never none (S3) | the 5 `keyboard_hook::tests` of R5 (fake installer, `keyboard_counts`) | unit |
| No key data kept or logged | scan `no_key_data_leaves_the_raw_input_read` (`mod.rs:1231-1258`) | scan |
| Re-entry refusal counter | `a_refused_reentry_is_counted_in_main_state_refused` (`main_thread.rs:1037-1060`) | unit |
| Panic hook with a bound | R9 tests | unit + child process |
| Debug flag absent from release | `the_debug_hold_is_compiled_only_into_debug_builds`; `cargo build --release --locked` | scan + build |
| No test installs a hook (S1-S4) | `the_hook_installers_are_compiled_out_of_the_tests`, `the_system_installer_panics_at_every_call_under_test` | scan + unit |

Manual checks M1-M8 for the owner (agents never run them): `impl.md:152-161`; M1 needs the release daemon stopped first (ruling M7).

## Invariants in play

- **R1** callback: `src/hook/keyboard_hook_callback.rs:357-400` (heartbeat `:367`, debug hold `:383-384`); scans `src/logging/mod.rs:459-466`, `keyboard_hook_callback.rs:1334` (`the_gesture_path_starts_no_thread`, excludes `src/hook/watchdog`, doc `:1325-1333`). KS-08.
- **R2** sentinel: no new `SendInput`; heartbeat before `is_injected_event` (`keyboard_hook_callback.rs:367` vs `:222`).
- **R3** shared state: `HealthState` atomics inside `KeyboardHook` (`keyboard_hook.rs:57`); watchdog holds `&'static HealthState`, not `&KeyboardHook` (`watchdog/mod.rs:366-371`); `with_main` used once in the reinstall (`main_thread.rs:861`). KS-01: reinstall only from `serve` on main (`main_thread.rs:825-828`).
- **R4** orderings: heartbeat `fetch_add AcqRel`/`load Acquire` (`health.rs:60,65`); `record_reinstall`/`snapshot` pairing (`health.rs:96-98,125-126`); `hook_handle.swap(AcqRel)` (`keyboard_hook.rs:284`); `ARMED` Release/Acquire (`debug_hold.rs:42,69`). Grep finds no `Relaxed`, `SeqCst` or `static mut` in `health.rs`, `watchdog/`, `debug_hold.rs`, `installer.rs`, `panic.rs`.
- **R6** Display: `ResetReason::HookReinstalled` (`engine/mod.rs:403`), 4 `Counter` names (`counters.rs:88-93`), `ReinstallOutcome` (`health.rs:212-221`), `OldHook` (`keyboard_hook.rs:312-323`), `HookHealth` (`health.rs:163-183`).
- **R7** untouched: the reinstall does not call `update_bindings` (`keyboard_hook.rs:276-290`).
- **R8**: raw `VKey` used only in `qualifies` (`watchdog/mod.rs:706-708`).
- **R10** message loop and shutdown: watchdog pump (`watchdog/mod.rs:585-662`); start/stop order (`app.rs:220,245,313-322`); `DestroyWindow` on the creating thread (`watchdog/mod.rs:481-490`). KS-01, KS-20 (single-instance mutex relevant to manual M1).
- **unsafe** added (SAFETY present at each, per `impl.md:83-88`): `installer.rs:90,101,109,127,141,156`; `watchdog/mod.rs:417,434,461,486,546,549,599,645,659,678,696`; `main_thread.rs:885`; test-only blocks in `keyboard_hook_callback.rs` and `bench.rs`.
- Rulings S1-S4, I1, I3, M1-M7, Q1-Q8: `D:/tmp/ks-orch/p2-2.4-watchdog/rulings.md`; developer's carry-out `cut.md:708-728`.

## Open points from impl.md (`impl.md:163-169`)

1. Needs a ruling (latency): R14 misses D11's bound (R14 p50 22.2-22.8 us vs R12 4.7-4.8 us; R14/settled within 0.5 us of R12). Options (a) accept on the raw-input-after-hook argument, (b) batch raw input at ticks, (c) raw input on main (R15 ~3.9 us/key). Undecided; depends on Q6 / manual M6 (`cut.md:745-746`).
2. Commit `58c4370` message says "debug_h::" for `debug_hold::` (not amended).
3. `HookWatchdog::start` gives up after 5 s; a thread that sends ready in the same instant runs unstopped until process end (same window as the executor's start).
4. Integration tests link the non-test library with the real installer; the S4 scan keeps `tests/` free of installer names and `core::app`.
5. No pre-existing gate warnings.

Also from `impl.md`: Phase-2 book sync list (`impl.md:136-144`) and follow-ups FU-1..FU-3 (`impl.md:146-150`); UNVERIFIED OS behaviour Q5-Q8 (`cut.md:689-694`).

## Not covered

- Not read line by line: `src/hook/watchdog/mod.rs:710-1094` (test helpers `Drive`, `Recorder`, `FakeTimer` and the I1/I3/M2 test bodies), `decision.rs:236-664` test bodies beyond the two quoted, `installer.rs:198-342` fake internals, `main_thread.rs:1854-2124` test bodies beyond `:1908-1920`, `keyboard_hook.rs:678-806` test bodies, `engine/scenarios.rs:2340-2407` bodies, `bench.rs:482-590` R15 body.
- `src/bin/swch.rs` and `src/ipc.rs` read only at the cited lines (unchanged files).
- README, CHANGELOG and `errors.md` checked only at hunk headers and the rows quoted; not proof-read against every log string.
- `critique.md` read at its headings only; `bench-dev.md` not read (numbers taken from `impl.md`).
- No command of the gate was run for this pack; no test, build or bench result here is mine.
