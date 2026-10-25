# Tester: p2-2.3-control-window / roadmap 2.3 (HEAD 4539306 vs master c3dab6a; developer HEAD 6f6edb3)

Verdict: GREEN. Gate green, every red-first red on its red tree and green on HEAD, 19 own mutations (3 exposed test gaps, all closed by `Tests:` commits and shown red under the mutation; 1 equivalent mutant). Three bench flags for the reviewer/triage (R11 +0.6 us vs the +0.5 us allowance, R9 0.5x bound, R12 p99 in one noisy run), listed under "Bench", none an unjustified regression.

HEAD is `4539306` = developer HEAD `6f6edb3` + three tester `Tests:` commits (test code only). Tree clean at the end (`git status --short` empty); `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 4539306 [feat/control-window]`; scratch dirs `wt/` and `target-wt/` deleted.

## Gate (HEAD 4539306, run in the lane tree)
- `cargo fmt --check`: OK
- `cargo clippy --all-targets --locked -- -D warnings`: OK (sources touched first; "Checking key-switch-rs ... Finished")
- `cargo test --locked --no-fail-fast` (`gate_test_final.txt`):
  - lib: `running 686 tests` / `test result: ok. 679 passed; 0 failed; 7 ignored`
  - swch bin: `running 0 tests` (no tests, expected); `tests/ipc_roundtrip.rs`: `running 6 tests` / `6 passed; 0 failed`; doc-tests `running 0 tests` (none)
  - at developer HEAD 6f6edb3 the same run: `running 683 tests` / `676 passed; 0 failed; 7 ignored` (`gate_test_head.txt`)
  - total passed 685, failed 0, ignored 7. Repeated 5 times (`cargo test --locked --lib`): 679 passed, 0 failed each time (timing tests in `hook::main_thread` and the real-window tests did not flake).
  - Ignored (all `manual:`/`bench:`, none run): `bench::latency_bench` (bench), `core::executor::tests::the_win32_runner_opens_a_url`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` (M14), `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad` (M10), `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`.
- `cargo build --release --locked`: OK
- Safety scan of `git diff master..HEAD -- src`: no new `SendInput(` site (the six hits are pre-existing files), no `Ordering::Relaxed`, `static mut`, `unwrap()` added; no `log::` in `keyboard_hook_callback.rs` outside tests; every added `unsafe` block has a `// SAFETY:` line; no test names `key-switch-rs.sock`, writes a `config.ron` outside a temp dir, or writes the registry. The only `SetWinEventHook` additions are in `KeyboardHook::install` (production, not run by any test).

## Red-first replay
Parent worktree `wt/parent` with `CARGO_TARGET_DIR=target-wt/parent`; command `cargo test --locked --lib -- --exact <names>`; every run's `running N tests` has N equal to the number of names given. HEAD column: one run of all 56 names, `running 56 tests` / `test result: ok. 56 passed; 0 failed` (`head_redfirst.txt`).

| Red tree | Test(s) | Parent | HEAD | Note |
|---|---|---|---|---|
| 19df9d4 (C3) | `hook::main_thread::tests::a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop` | red (assert): `a failed set leaves the timer set for no deadline the loop knows; left: Some(Instant..) right: None` | ok | `running 1 test` |
| 668e80e (C5) | `engine::tests::a_refused_window_timer_drops_no_gesture`, `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`, `hook::main_thread::tests::a_wake_that_cannot_be_set_counts_only_the_holds_it_drops` | red x3 (assert): `left: 1 right: 0` (x2); `only the Hold is dropped left: 3 right: 1` | ok | `running 3 tests`, 0 passed 3 failed |
| a0ae250 (C7) | `engine::scenarios::a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys`, `...a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows` | red x2: `down LShift at +50 ms: expected to fire [ToggleCapsLock], fired []`; `down LCtrl at +50 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]` | ok | `running 2 tests` |
| 5388175 (C9) | `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path` | red (assert): `panic paths in src/hook/config_watcher.rs: 126: .expect("spawn config-watcher thread");` | ok | `running 1 test` |
| 0ef6a99 (C12) | `engine::scenarios::a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows`; `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`; `engine::properties::nothing_is_left_down_in_windows_when_the_hook_misses_key_downs` | red x3 (assert): `up CapsLock at +0 ms: VK 0x14 is down in Windows but physically up`; `a swallowed key-up asks the OS about its key once left: 0 right: 1`; `up CapsLock at +10838 ms: VK 0x14 is down in Windows but physically up` | ok | `running 4 tests`, 1 passed 3 failed |
| 0ef6a99 (C12 pins) | `...a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows`, `...pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows` (the pack's name for (b) was wrong; real name used) | green (pin) | ok | pins, as declared; carried by mutation M1 (witness removal) |
| 765e5b5 (C14) | `running 22 tests`: 15 red, 7 green. Red: `a_foreground_change_keeps_a_swallowed_key_up_swallowed`, `immediate_key_held_across_a_focus_change_keeps_its_repeats_and_release_suppressed`, `capslock_held_across_a_focus_change_does_not_reach_windows`, `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed`, `capslock_held_across_a_focus_change_switches_the_language_once`, `chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed`, `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed`, `immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed`, `press_after_a_lost_key_up_without_a_reset_reads_as_a_repeat_and_fires_only_a_repeating_binding`, `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`, `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed`, `nothing_is_stuck_after_win_l`, `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press`, properties `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` and `no_key_up_reaches_windows_alone_across_focus_changes_unseen_presses_and_desktop_switches` | red (assert), e.g. `up F13 at +533 ms: expected the event to be suppressed, it was passed`; `down L at +0 ms: expected the event to be suppressed, it was passed`; `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`; `up F13 at +0 ms: Windows got a lone key-up of F13`; property `press of CapsLock: key-down suppressed, up passed` | ok | green at 765e5b5 as declared pins: `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`, `after_input_leaves_for_the_secure_desktop_the_next_press_of_a_swallowed_key_fires`, `a_late_unlock_notification_keeps_a_press_made_after_the_unlock`, `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order`, `a_hold_armed_between_two_return_notifications_fires`, `a_second_lock_releases_a_key_pressed_after_the_first`, `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows` (pins carried by M15, M2, M1) |
| f7588e4 (C16) | `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing`, `...a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold`, `...a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands` | compile-red: `E0599 no method named reload ... &mut MainState`, `... MainThreadHandle`, `E0599 no variant named Reload ... Request` | ok | carried by M3 |
| 036a859 (C18) | `hook::keyboard_hook_callback::tests::{a_hundred_conditional_presses_in_one_window_query_its_process_once, a_title_condition_reads_the_cached_windows_title_on_each_key_down, a_window_of_this_process_is_never_asked_for_its_title}`, `platform::foreground::tests::{the_cache_queries_again_only_for_another_window, an_invalid_foreground_window_keeps_the_cache}` | compile-red: 12 errors (`E0425 cannot find type ForegroundCache`, `E0405 cannot find trait WindowQuery`, `E0432 unresolved imports`) | ok | carried by M4, M4b |
| afbe152 (C20) | `hook::main_thread::tests::{a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key, a_late_return_signal_keeps_a_running_macro_and_an_armed_hold, the_drain_logs_one_line_per_input_signal}`, `hook::control_window::tests::{only_the_lock_and_the_unlock_are_input_signals, the_session_messages_note_the_lock_and_the_unlock, the_session_end_messages_are_answered_and_end_the_loop, the_control_window_is_a_hidden_top_level_window_released_on_drop}`, `hook::keyboard_hook_callback::tests::{a_desktop_switch_is_classified_by_where_input_is, a_desktop_switch_notes_a_signal_only_while_remapping_is_on, remapping_off_callbacks_pass_without_asking_for_the_engine}` | compile-red: 26 errors (`cannot find function note_input_signal`, `control_window_proc`, `desktop_switch_note`, `take_pending_signals`, `ControlWindow`, ...) | ok | carried by M2, M5, M6, M9, M11, M12, M13, M15, M18, M19 |
| a60aa65 (C2 pin) | `engine::scenarios::a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down` | green by design | ok | red under M21 |

Worktrees removed, `git worktree prune` run, `git worktree list` shows only the lane tree.

## Mutations (scratch worktree `wt/mut`, own target dir; test named before the run; each reverted with `git checkout -- .` and `git diff --exit-code` clean)
| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| M1 | src/engine/mod.rs (witness) | `decision.suppress = swallow && !env.os_key_down(vk)` -> `= swallow` | C12 (a), `only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`, property (e) | caught x3: `VK 0x14 is down in Windows but physically up`; `a swallowed key-up asks the OS about its key once left: 0 right: 1` |
| M2 | src/engine/mod.rs `Engine::reset` | `ResetReason::InputLeft => self.keys.release_all()` -> `{}` | `nothing_is_stuck_after_win_l`, `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` | caught x2: `down L at +0 ms: expected the event to be suppressed, it was passed`; `the first F14 after the lock is a press, not a repeat left: [] right: [SwitchLanguage]` |
| M3 | src/hook/main_thread.rs `MainState::reload` | drop `reset_engine(Reloaded)` | `a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing`, `a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold` | caught x2 (the engine scenario `a_tap_armed_before_a_reload...` stays ok, as it does not go through `MainState`) |
| M4 | src/platform/foreground.rs `window_changed` | remove the same-window/pid early return | `the_cache_queries_again_only_for_another_window` | caught (`the same window asks nothing left: 2 right: 1`); `a_hundred_conditional_presses...` stays ok (the key path never calls `window_changed`) |
| M4b | M4 + `decide_event` calls `foreground.resync()` per event | per-event process query | `a_hundred_conditional_presses_in_one_window_query_its_process_once` | caught: `one process-name query for a hundred presses in one window left: 201 right: 1` |
| M5 | src/hook/control_window.rs `session_signal` | `WTS_SESSION_LOCK => None` | `only_the_lock_and_the_unlock_are_input_signals`, `the_session_messages_note_the_lock_and_the_unlock` | caught x2 |
| M6 | src/hook/keyboard_hook_callback.rs `desktop_switch_note` | `Ok(true)` -> `DesktopSwitchedAway` | `a_desktop_switch_is_classified_by_where_input_is` | caught |
| M8 | src/engine/gestures.rs own-combo arm | `if group.keys.keys.iter().any(..)` -> `if false` | the two C7 scenarios | caught x2 (`down LShift at +50 ms: expected to fire [ToggleCapsLock], fired []`) |
| M9 | src/hook/control_window.rs `session_ends` | `wparam.0 != 0` -> `true` | `the_session_end_messages_are_answered_and_end_the_loop` | caught |
| M10 | src/hook/main_thread.rs `Scheduler::start` | remove `self.armed = None` after a failed set | `a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop` | caught (`left: Some(Instant..) right: None`) |
| M11 | src/hook/control_window.rs `OwnedWindow::drop` | no `DestroyWindow` | `the_control_window_is_a_hidden_top_level_window_released_on_drop` (EnumThreadWindows count) | caught |
| M15 | src/engine/mod.rs `InputSignal::reset` | `DesktopSwitchedBack` -> `Some(InputLeft)` | `a_hold_armed_between_two_return_notifications_fires`, `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order`, `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold` | caught x3 (R2) |
| M17 | src/engine/mod.rs `start_timers` | count every refused timer | `a_refused_window_timer_drops_no_gesture` | caught (the main_thread test is a different path) |
| M17b | src/hook/main_thread.rs `plan_wake` | count every drained token | `a_wake_that_cannot_be_set_counts_only_the_holds_it_drops` | caught (`only the Hold is dropped left: 3 right: 1`) |
| M18 | src/hook/keyboard_hook_callback.rs `desktop_switched` | remove the remapping-off gate | `a_desktop_switch_notes_a_signal_only_while_remapping_is_on` | caught x2 (+ `remapping_off_callbacks_pass_without_asking_for_the_engine`) |
| M19 | src/hook/control_window.rs `control_window_proc` | remove `&& get_hook().is_enabled()` | `the_session_messages_note_the_lock_and_the_unlock` | caught |
| M21 | src/engine/gestures.rs | drop `fire.extend(deferred_tap)` | C2 pin | caught: `down F13 at +381 ms: expected to fire [SwitchLanguage], fired []` |
| M12 | src/hook/main_thread.rs `answer_input_signals` | remove the `if !hook.is_enabled() { return; }` gate | any main_thread test | **NOT CAUGHT** at first (3 candidate tests ok, then whole lib `679 passed`). Closed: test `hook::main_thread::tests::a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain` (commit c7ab5c7); red under M12 (`a drain with remapping off logs and resets nothing`), green without |
| M13 | src/hook/main_thread.rs `answer_input_signals` | `if signals.signals().any(InputSignal::input_returned)` -> `if false` (no foreground re-read on a return) | any | **NOT CAUGHT** at first (whole lib `678 passed; 0 failed`). Closed: `hook::main_thread::tests::a_return_signal_reads_the_foreground_window_again` (commit 4539306, counting `MovingQuery`, both return signals); red under M13 (`left: Some(Some("p4096.exe")) right: Some(Some("p8192.exe"))`), green without |
| M20 | src/engine/mod.rs `on_foreground_changed` | add `self.keys.release_all()` (a focus change frees held keys) | `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` | **NOT CAUGHT by that test** (ok under mutation; the acceptance test asserts suppression only, a repeat re-launching wt.exe went unseen). Caught by `capslock_held_across_a_focus_change_switches_the_language_once` and the focus property. Closed for Win+Alt+T: `engine::scenarios::win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` (commit 7ab78ab); red under M20: `repeat T at +500 ms: expected to fire [], fired [Launch { exe: "wt.exe", args: [] }]` |
| M16 | src/engine/mod.rs `Engine::reset` | remove `generation = generation.wrapping_add(1)` | any | not caught; **equivalent mutant**: `seq` is engine-wide and never reset (`Ids::next_seq`), so a stale token is already dead by `gestures.clear()`; the generation bump is defence in depth (pre-existing code, not a lane change). Not counted as a gap |

## Acceptance
| Criterion (spec / cut :790-807 / rulings) | Evidence | Status |
|---|---|---|
| Tap armed, reload, key-up: old action does not run | `engine::scenarios::a_tap_armed_before_a_reload_fires_nothing_at_its_key_up`; `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing`, `...a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold` (all ok on HEAD; M3 red) | verified; live M7 manual |
| Win+Alt+T held across a foreground change keeps T repeats suppressed | `engine::scenarios::win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (release red at 765e5b5, ok on HEAD) + tester-added `..._does_not_launch_again_on_its_repeats`; property `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` (red at 765e5b5, M20 red) | verified; M8 manual |
| Nothing stuck after Win+L (request path, no session change; declared trigger change: the lock / switch away, not WTS_SESSION_UNLOCK, architect Q2) | `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key`, `the_session_messages_note_the_lock_and_the_unlock`, `a_desktop_switch_is_classified_by_where_input_is`, `nothing_is_stuck_after_win_l` (M2 red), property `no_key_up_reaches_windows_alone_...` | verified; live M1, M2, M5 manual |
| After Win+L the next press of a swallowed non-repeating key fires | `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`, F14 part of the request-path test (M2 red) | verified; M1 manual |
| passed case | `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press` (red at 765e5b5) | verified |
| 100 conditional presses -> one process-name query | `hook::keyboard_hook_callback::tests::a_hundred_conditional_presses_in_one_window_query_its_process_once` (compile-red C18, M4b red `left: 201 right: 1`) | verified |
| Lone key-up rule after a reset pinned | `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`, `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed` (red `Windows got a lone key-up of F13` at 765e5b5), property `no_key_up_reaches_windows_alone_...` | verified; M11 manual |
| R1 no stuck key (OS witness) | C12 (a)-(e), M1 red | verified; M12, M13 manual (UIPI / secure desktop) |
| R2 reset never discards later state | the four R2 scenarios + `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold` (M15 red) | verified; M1 timestamps manual |
| Control window not HWND_MESSAGE, Drop guards, features; desktop-switch hook; title current | `hook::control_window::tests::*` (M5/M9/M11/M19 red), `a_desktop_switch_notes_a_signal_only_while_remapping_is_on` (M18 red), `a_title_condition_reads_the_cached_windows_title_on_each_key_down`; `Cargo.toml` features (Cargo.lock unchanged) | verified in code and unit tests; hook install/uninstall of the desktop WinEvent hook is Win32 (no test), M5/M9 manual |
| 2.2 leftovers | C1 doc (`git diff` on `core/executor/mod.rs`), C2 (M21), C3/C4 (M10), C5/C6 (M17, M17b), C7/C8 (M8), C9/C10 (scan test) | verified |
| Latency bench: no unjustified regression, cache shows lower cost for a conditional binding, bench row added | `bench::latency_bench` R1-R13; see Bench | verified with flags (below) |
| Docs (README, default_config.ron, known-issues, triggers-conditions, CHANGELOG) | files present in `git diff --stat master..HEAD`; mdbook build / link check not re-run by the tester (developer's `linkcheck.txt`: 0 broken) | not verified by tester |
| Manual only | M1-M9, M11-M13 (live lock, UAC, Ctrl+Alt+Del, sign-out, UIPI, Alt+Tab); M10 and M14 are the two ignored tests added | manual, not verified |

No literal `rg`/`diff` check is named in the spec, cut acceptance mapping, rulings or impl.md.

## Bench (section 4b; `bench/` has every table; release; `KS_BENCH_LOAD=1` for the official set because the lane touches `main_thread` `run`/`drain`)
Commits: master `c3dab6a` (R1-R8), C11 `dd53797` (R1-R12), HEAD `4539306` (R1-R13; product code = `6f6edb3`). Official set = 3 interleaved runs per commit, median of p50 in us (largest p99 in brackets). Each `latency_bench` run: `1 passed`.

| Row | master | C11 | HEAD | Rule | Result |
|---|---|---|---|---|---|
| R1 | 0.100 (0.5) | 0.100 (0.7) | 0.100 (0.3) | <= 1.25x master + 0.1 | ok |
| R2 | 0.200 (1.2) | 0.200 (2.1) | 0.200 (0.4) | same | ok |
| R3 | 0.100 (0.4) | 0.100 (0.5) | 0.100 (0.2) | same | ok |
| R4 | 4.8 (28.0) | 6.5 (199.1) | 4.8 (14.6) | <= 1.25x + 0.5 (+0.125) = 6.6; p99 <= 1.5x + 2 | ok |
| R5 | 4.9 (19.1) | 7.0 | 4.9 (46.3) | <= master | ok (equal) |
| R6/5,20,50,200 | 423/365/340/313 | - | 350/273/436/382 | <= 1 ms and <= master + 0.5 ms | ok |
| R7 | 10.8 (52.8) | 10.9 | 10.8 (68.9) | <= master + 20; p99 <= 1.5x + 50 = 129 | ok |
| R8 (normal / highest, one-way) | 15.9 / 20.0 | 15.2 / 18.2 | 15.6 / 19.3 | no bound beyond R7 | equal |
| R9 | - | 9.9 | 4.9 | <= 0.5x C11 (4.95) and <= HEAD R12 + 2 (6.8) | ok, marginal (see flag 2) |
| R10 | - | 6.7 | 6.5 | <= C11 + 0.5 (7.2) | ok |
| R11 | - | 4.8 | 5.4 | <= C11 + 0.5 (5.3) | **+0.6, 0.1 us over** (flag 1) |
| R12 | - | 4.8 (17.4) | 4.8 (40.7) | p50 <= 1.05x + 0.1 (5.14); p99 <= 1.5x C11 p99 + 2 (28.1) | p50 ok; p99 largest 40.7 (flag 3) |
| R13 | - | - | 10.5 | <= HEAD R7 + 20 | ok |

Extra set (3 more runs per commit for C11 and HEAD, no load, same command): C11/HEAD median p50: R4 4.8/4.9, R7 10.8/10.7, R9 7.4/4.8, R10 5.1/6.2, R11 4.8/5.4, R12 4.8/4.8, R13 -/10.7. R11 p50s HEAD [5.4, 5.4, 5.4], C11 [4.7, 4.8, 4.8].

Flags (for the reviewer and triage; none judged an unjustified regression):
1. R11, the witness (one `GetAsyncKeyState` on the key-up of a swallowed key): HEAD 5.4 vs C11 4.8 in 6 of 6 runs except one noisy run (7.1), i.e. +0.6 us against the +0.5 us allowance, one 0.1 us timer tick over. The cut (D14 and ruling R1) names this read as justified extra work on the hook path, so I do not count it RED; the numeric allowance in D14 is slightly too small and `results.md` should state +0.6 us.
2. R9: official set 4.9 vs 0.5 x 9.9 = 4.95 (pass by 0.05); the extra set has C11 at 7.4, so 0.5x would be 3.7 and HEAD 4.8 misses it. Over all 6 runs HEAD R9 median is 4.85 vs C11 7.6 (about 36 % lower, 1.6x), and HEAD R9 equals HEAD R12 (4.8): the conditional binding costs the same as an ordinary key, so the cache removed the whole process query and the 0.5x ratio is limited by the R12 floor. Acceptance "shows as a lower hook-path cost" holds; the second clause (<= R12 + 2) holds with margin.
3. R12 p99: the largest HEAD p99 (40.7, run 1) exceeds 1.5 x 17.4 + 2 = 28.1. HEAD run 1 was a noisy run (every row inflated: R12 p50 6.2, R11 7.1); the other five HEAD runs have R12 p99 5.6-9.0, C11's six runs 6.4-17.4. R12 has no code difference from C11 on the product path; treated as machine noise (the owner uses the machine during the run).
4. R10: extra-set HEAD 6.2 vs C11 5.1 (+1.1) but the six-run medians are 6.35 vs 6.55; the row reads the live title of the real foreground window (same `GetWindowTextW` as before), bimodal 5.1/6.3.

## Failures
None open. The three gaps (M12, M13, M20) were test gaps, not product defects, and are closed by tester commits.

## Tests added by the tester (`Tests:` commits on feat/control-window, test code only, trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`)
- c7ab5c7 `src/hook/main_thread.rs`: `hook::main_thread::tests::a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain` (kills M12)
- 7ab78ab `src/engine/scenarios.rs`: `engine::scenarios::win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` (kills M20)
- 4539306 `src/hook/main_thread.rs`: `hook::main_thread::tests::a_return_signal_reads_the_foreground_window_again` (kills M13; adds test-only `MovingQuery`)
fmt, clippy and the full gate were re-run after these commits (above).

## Notes
- Pack inaccuracy: the C12 pin (b) is named `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows`.
- `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` asserts the macro stops within 100 ms of the lock; real-clock, passed 7 of 7 full runs here. A heavily loaded machine could flake it.
- Not verified by the tester (OS behaviour, per impl.md): `GetAsyncKeyState` pre-event state inside the LL hook, `UOI_IO` while the lock screen has input, lock-signal latency; all on the manual list (M1, M5, M12-M14).
- Developer's R1 witness for an injected generic `VK_SHIFT` is pinned by `the_witness_of_a_generic_shift_key_up_reads_the_generic_state` (ok on HEAD); not mutated by me.
- Bench worktrees and target dirs are deleted; raw tables remain in `D:/tmp/ks-orch/p2-2.3-control-window/bench/` (`summary.txt`, `summary2.txt`).
