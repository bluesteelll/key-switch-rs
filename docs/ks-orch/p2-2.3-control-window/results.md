# Results analysis: p2-2.3-control-window / roadmap 2.3 Control window and state reset (KS-29, KS-31, KS-06 status)

Branch `feat/control-window` at `81bb38c`, cut from `master` `c3dab6a` (merge base equals the `master` tip). 27 commits: the developer's C1..C22 (`0ca09d7`..`6f6edb3`), three tester `Tests:` commits (`c7ab5c7`, `7ab78ab`, `4539306`) and fix round 1 (`c4f1bca`, `81bb38c`).

## VERDICT: ACCEPT

**Summary.** Every acceptance line of roadmap 2.3 and of the lane spec has a named automated test, plus the live checks that only a human can run (M1-M14). Every claimed issue has a regression test that was red before its fix or is pinned by a mutation. I re-ran the gate myself and it is green: 685 passed, 7 ignored (all `manual:`/`bench:`), against 636 on `master`. The last tester round is GREEN and the last review is APPROVED. No CONFIRMED Critical or Important item is open. Every ruling is carried out, with one exception: the V2-m1/R2 timestamp evidence needs a live daemon, which no agent may run, so it goes to the owner.

The latency picture fits the owner's decision:
- a conditional `AppEquals` key-down costs what an unbound key costs, because the process query has left the hook path;
- an ordinary key is unchanged;
- the one new hook-path cost, the R1 witness read, measures +0.6 us against the "~0.1 us" the cut and ruling R1 assumed. It is justified by the cut, but the owner should hear the real number.

## Quality gate (own run, HEAD `81bb38c`, fresh `CARGO_TARGET_DIR` in the session scratchpad)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (fresh target dir, `Checking key-switch-rs ... Finished`, exit 0) |
| `cargo test --locked` | exit 0; lib `running 686 tests` / `679 passed; 0 failed; 7 ignored`; `tests/ipc_roundtrip.rs` 6 passed; bins and doc-tests 0. Total **685 passed / 7 ignored**. On `master`: lib 636 run / 630 passed / 6 ignored (`p2-2.2-timers/results.md:15`). Log: `D:/tmp/ks-orch/p2-2.3-control-window/analyst-test.txt` |
| `cargo test --locked -- --list --ignored` (listed, not run) | 7: `bench::latency_bench`, `core::executor::tests::the_win32_runner_opens_a_url`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` (new, M14), `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad` (replaces master's `data::condition` Notepad ignore, M10), `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. Each has a `manual:` or `bench:` reason |
| `cargo doc --no-deps --locked` | 3 warnings, all pre-existing and in files the lane does not touch (`config/loader.rs:156`, `:176`; `data/vk_name.rs:4`) |
| `mdbook build book -d .../book-analyst` + `phase-0/htmllinks.py` | built; `pages=20 links=2360 broken=0 duplicate_ids=0` (only the known mdbook-mermaid version warning) |
| Trailers | 27/27 commits: 24 `Co-Authored-By: Claude Opus 5.5`, the 3 tester commits `Claude Sonnet 5.5`; no merge commits; `Cargo.lock` unchanged |

Test names: `#[test]` functions go from 635 to 685 by name. The 13 names that exist only on `master` are all renames or migrations the cut declares:
- C14/C15 flips: `foreground_change_forgets_pending_key_up_suppression` -> `a_foreground_change_keeps_a_swallowed_key_up_swallowed`; `immediate_key_held_…`, `win_alt_t_held_…`, `press_after_a_lost_key_up_…` and `press_symmetry_check_reports_…`, each renamed;
- keys tests: `forget_blocked_*`, `forgotten_modifier_*`, `reset_from_os_*`;
- C17 reload flips: `a_gesture_armed_by_the_winner_resolves_after_a_reload_…` -> `a_tap_armed_before_a_reload_fires_nothing_at_its_key_up` (the KS-31 flip), `reload_during_a_passing_double_tap_…`;
- C18/C19: the two `foreground_context_*` tests moved to `platform::foreground`.

No assertion was dropped to get green. The flips change the expected outcome to the new rule that the spec asks for, and the CHANGELOG records each.

## Acceptance criteria

| # | Criterion (roadmap 2.3 / lane spec) | Evidence | Status |
|---|---|---|---|
| 1 | With a Tap armed, a reload followed by the key-up does not run the old action | `engine::scenarios::a_tap_armed_before_a_reload_fires_nothing_at_its_key_up` (`src/engine/scenarios.rs:4715`, flipped from master's "fires" expectation); `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing` (`main_thread.rs:1327`) and `…a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold` (`:1364`), both compile-red at C16 `f7588e4`, and red under tester mutation M3 (no `reset_engine` in `MainState::reload`); manual M7 | PASS |
| 2 | Holding Win+Alt+T while the foreground window changes keeps the T repeats suppressed | `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (`scenarios.rs:1898`; its release part is red at C14 `765e5b5`); `win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` (`:1922`, tester, red under M20); property `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` (`properties.rs:381`, red at C14); manual M8 | PASS |
| 3 | Nothing is stuck after Win+L; the reset clears gestures, cancels macros and resyncs keys (request path without a session change; live unlock manual). Declared trigger change, accepted as Architect Q2 round 2: the reset answers the input leaving (`WTS_SESSION_LOCK`, or a switch away by `UOI_IO`), not `WTS_SESSION_UNLOCK` | `hook::main_thread::tests::a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` (`main_thread.rs:1459`: Hold dropped, macro gone within 100 ms, the next F14 is a press); `hook::control_window::tests::the_session_messages_note_the_lock_and_the_unlock` (`control_window.rs:327`); `hook::keyboard_hook_callback::tests::a_desktop_switch_is_classified_by_where_input_is` (`:1071`); `engine::scenarios::nothing_is_stuck_after_win_l` (`scenarios.rs:2121`, red at C14, red under the M2 mutation `SessionLocked.reset() -> None` and the tester's `release_all -> {}`); property `no_key_up_reaches_windows_alone_…` (`properties.rs:386`); manual M1, M2, M5 | PASS |
| 4 | After Win+L with a swallowed, non-repeating trigger key held, the next press fires (owner decision, open question 9) | `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires` (`scenarios.rs:2172`); the same through the secure desktop, `after_input_leaves_for_the_secure_desktop_…` (`:2193`); the F14 part of the request-path test in row 3; manual M1 | PASS |
| 4b | (spec) the passed case | `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press` (`scenarios.rs:2151`, red at C14); manual M2 | PASS |
| 5 | 100 presses of a conditional binding in one window cause one process-name query | `hook::keyboard_hook_callback::tests::a_hundred_conditional_presses_in_one_window_query_its_process_once` (`keyboard_hook_callback.rs:630`, counting `WindowQuery`, no hook; compile-red at C18; tester mutation M4b, a per-event resync, gives `left: 201 right: 1`) | PASS |
| 6 | The lone key-up rule after a reset is pinned by a test | rule L, decided: no key-up reaches Windows alone, so whether PrintScreen acts on one stops mattering. `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed` (`scenarios.rs:2092`), `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed` (`:2106`), both with `check_no_lone_key_up` and red at C14; the property in row 3; manual M11 (PrintScreen across Ctrl+Alt+Del) | PASS |
| 7 | Latency bench: no unjustified regression; the per-HWND cache shows a lower hook-path cost for a conditional binding; a bench row for it | R9-R12 (C11 `dd53797`), R13 (C21). Numbers in "Latency" below. The R11 breach of +0.1 us over its allowance is the witness read that the cut (D14, `cut.md:421`) and ruling R1 justify (triage T-B1, REFUTED as a defect) | PASS (cost stated) |
| 8 | (spec) hidden top-level control window, not `HWND_MESSAGE`; WTS, `WM_QUERYENDSESSION`, `WM_ENDSESSION` go through it; Drop guards on every exit path; the missing feature named | `the_control_window_is_a_hidden_top_level_window_released_on_drop` (`control_window.rs:407`: `GA_PARENT == GetDesktopWindow`, not visible, `EnumThreadWindows` finds 0 windows after the drop, class re-registrable); `the_session_end_messages_are_answered_and_end_the_loop` (`:352`); guard order `_session`, `_window`, `_class` (`control_window.rs:71-75`); features `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_System_RemoteDesktop`, `Win32_System_StationsAndDesktops` (`Cargo.toml`); manual M6, M9 | PASS |
| 9 | (spec) `EVENT_SYSTEM_DESKTOPSWITCH` hooked like the foreground hook | `KeyboardHook::install` (`keyboard_hook.rs:189-218`) and `uninstall` (swap `AcqRel` + `UnhookWinEvent`), main thread only; `a_desktop_switch_notes_a_signal_only_while_remapping_is_on` (`keyboard_hook_callback.rs:1108`, red under M18); manual M5 | PASS (install itself is Win32, reviewed) |
| 10 | (spec) the title stays current inside one window | read live with `GetWindowTextW`, which sends no `WM_GETTEXT` to another process's window (Q8); windows of this process are never asked. `a_title_condition_reads_the_cached_windows_title_on_each_key_down` (`:659`), `a_window_of_this_process_is_never_asked_for_its_title`; manual M4 | PASS |
| 11 | (spec) "In an elevated daemon a foreground change no longer clears every blocked bit" | widened to every daemon by ruling R1, with the witness covering the stuck-key case: `Engine::on_foreground_changed` only resyncs the modifiers (`engine/mod.rs:~608`) | PASS |
| 12 | (spec) 2.2 leftovers: m2, OQ3, P1, OQ2, executor doc, watcher `.expect` | m2: `a_refused_window_timer_drops_no_gesture` (`engine/mod.rs:1339`) and `a_wake_that_cannot_be_set_counts_only_the_holds_it_drops` (`main_thread.rs:1061`), red at C5. OQ3: `a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop` (`:1007`), red at C3, fix `main_thread.rs:401`. P1: `a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down` (`scenarios.rs:988`, a pin, mutation M21). OQ2: `a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys` (`:4195`), red at C7. Executor doc: `core/executor/mod.rs:5-7`. Watcher: `config_watcher.rs:158` `.map_err(notify::Error::io)?`, pinned by `the_watcher_thread_has_no_panic_path` (`:452`). That pin was vacuous at C17 (review r1 I1) and is fixed in `c4f1bca`; it is now red under three mutations (tester r2 A, B, C) | PASS |
| 13 | (ruling R1) no stuck key when the hook misses a press of a swallowed key | `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows` (`scenarios.rs:2012`, red at C12 `0ef6a99`); the critic's script `…_across_focus_changes_…` and the remapping-off variant (pins, red under M1, the witness removed); the lock variant `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows` (`:2324`); `only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (`engine/mod.rs:1231`, red at C12); critic OQ2: `the_witness_of_a_generic_shift_key_up_reads_the_generic_state` (`:1279`) and `a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows` (`scenarios.rs:2067`); property `nothing_is_left_down_in_windows_when_the_hook_misses_key_downs` (`properties.rs:376`, red at C12); manual M12, M13 | PASS |
| 14 | (ruling R2) a reset never discards state created after input came back | `a_late_unlock_notification_keeps_a_press_made_after_the_unlock` (`scenarios.rs:2213`; the critic's literal script is part 1), `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order` (`:2248`), `a_hold_armed_between_two_return_notifications_fires` (`:2285`), `a_second_lock_releases_a_key_pressed_after_the_first` (`:2303`), `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold` (`main_thread.rs:1530`); all red under M15 or the R2 mutations. The timestamped M1 log is **owner only** (see Hand-off) | PASS (live evidence pending) |

## Issues claimed closed

| KS-ID | Regression test (the audit's scenario) | Fails on old code? | Status |
|---|---|---|---|
| KS-29 (Win+L leaves a phantom held key; `Ctrl+Alt+L` fires on the next Ctrl+Alt) | `nothing_is_stuck_after_win_l` (`scenarios.rs:2121`): the audit's own Win+L / Ctrl+Alt+L example, which asserts that the first L after the unlock is a press the binding swallows; `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`; `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` | Yes. Red at C14 `765e5b5` (`down L at +0 ms: expected the event to be suppressed, it was passed`), before the fix C15 `3246339` (tester red-first replay, `test_r1.md`); request-path test compile-red at C20 before C21; mutation-red under M2 | CLOSED for a lock, Ctrl+Alt+Del and a UAC prompt on the secure desktop; remainder in rows 63 and 79 |
| KS-31, reload half (pending entries keep actions of the old config) | `a_tap_armed_before_a_reload_fires_nothing_at_its_key_up` (`scenarios.rs:4715`); `a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing` (`main_thread.rs:1327`) | Yes. Master's version of the scenario asserted that the old Tap fired. The adapter tests were compile-red at C16 before C17 and are red under M3 | CLOSED. The off/on half was closed in `a1b5a27` |
| KS-06 status (a lone key-up after a focus change) | `a_foreground_change_keeps_a_swallowed_key_up_swallowed` (`scenarios.rs:206`, flipped), the Win+Alt+T pair, `capslock_held_across_a_focus_change_*`, property `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` | Yes. 9 flips red at C14 (`up <key> at +533 ms: expected the event to be suppressed, it was passed`), property red at C14 | CLOSED (known issue 55 Fixed) |

## Rulings (`rulings.md`)

| Ruling | Carried out | Where |
|---|---|---|
| R1 witness | yes: `decision.suppress = swallow && !env.os_key_down(vk)`, short-circuited, so a key-up that passes asks nothing | `src/engine/mod.rs:885`; Learn quote in cut D15; row 78; `Mode::Unseen` |
| R1 red-first script and lock variant | yes | row 13 above |
| R2 only "input left" resets; return signals never reset | yes: `InputSignal::reset` (`engine/mod.rs:~440`), `release_all` without an OS read (`keys.rs:148`), drain `answer_input_signals` (`main_thread.rs:714`) | pins in row 14 |
| R2 / V2-m1 "M1 records timestamped lines in impl.md" | **pass criterion carried out** (impl.md M1); the **timestamps need a live daemon and a real lock, which the safety rules forbid to every agent**. Left to the owner | Hand-off |
| M1-M8 | yes (M1 declared, row 68; M2 mutation run; M3 `app.rs:224`; M4 + V2-m4 `app.rs:292-297`; M5 both outcomes; M6 direct proc calls; M7 doc lines; M8 row 80) | impl.md "Remarks" |
| Q1-Q8 | yes. Q2: the reload does not resync the keys, stated in the CHANGELOG `Changed`. Q6: `Cargo.lock` unchanged. Q7: the bench ran on master, C11 (renumbered from C15) and HEAD. Q8: the title is read live, which the orchestrator reports to the owner | CHANGELOG, `bench/` |
| V2-m2 late lock | residual declared in row 79 (Windows Hello) | `known-issues.md` row 79 |
| V2-m3 UAC secure-desktop precondition | rows 63/79, CHANGELOG `Added`, M5 precondition (`PromptOnSecureDesktop`, read-only) | |
| V2-m4 window dropped after `uninstall`, before `executor.stop()` | yes | `src/core/app.rs:292`, `:296`, `:297` |
| Critic OQ1 (EnumThreadWindows), OQ2 (generic read + sided pin), OQ3 (shared class mutex) | yes | `control_window.rs:390-449`; `engine/mod.rs:1279`, `scenarios.rs:2067`; `CONTROL_CLASS` in `control_window.rs:57` and `bench.rs:529` |
| Architect Q1-Q3 (round 2) | yes (fourth feature; roadmap wording to the book sync; row 76 widened) | |
| Latency: state before and after | below | |

## Latency (owner decision; ruling "Latency")

These are the tester's official set (3 interleaved release runs per commit, `KS_BENCH_LOAD=1`, the median of the p50s, the largest p99 in brackets, in us), plus the 3-run extra set (`bench/summary.txt`, `summary2.txt`). The product code is `6f6edb3`. The r2 delta changed only tests and docs, so the bench was rightly not re-run. C11 is master's key path plus C1-C10, none of which runs in R9-R12. Rows R9-R12 do not exist on `master`, so C11 is their "before".

| Path | Row | Before | After (HEAD) | Reading |
|---|---|---|---|---|
| Conditional binding, `AppEquals` | R9 | C11 9.9 (56.6); median over 6 runs ~7.6 | 4.9 (36.7); median over 6 runs ~4.85 | The process query is gone from the hook path. HEAD R9 equals HEAD R12 (an unbound key), so a `when: AppEquals` costs nothing over an ordinary key. The D14 bound `<= 0.5 x C11` holds on the official set (4.9 <= 4.95) and is floor-limited on the extra set; `<= R12 + 2` holds with margin |
| Conditional binding, `TitleContains` | R10 | C11 6.7 (35.2); 6-run 6.55 | 6.5 (43.2); 6-run 6.35 | Unchanged: the same live `GetWindowTextW`, now skipped for this process's windows |
| Ordinary key, through the callback | R12 | C11 4.8 (17.4) | 4.8; p99 40.7 in one noisy run, 5.6-9.0 in the other five | No change. The p99 outlier is machine noise: every row of that run was inflated, and the re-runs are within the bound (triage T-B3) |
| Ordinary key, engine only | R1 / R2 / R3 | master 0.1 / 0.2 / 0.1 | 0.1 / 0.2 / 0.1 | No change |
| Mixed stream (holds one swallowed key-up) | R4 | master 4.8 (28.0) | 4.8 (14.6) | No change |
| **Witness read** (key-up of a swallowed key) | R11 | C11 4.8 (15.0), p50s 4.7-4.9 | **5.4** (41.9), p50 5.4 in 5 of 6 runs | **+0.6 us**, against D14's +0.5 us allowance and the "~0.1 us" of cut D15 / ruling R1's "~100 ns". It is exactly one `GetAsyncKeyState`, only on a key-up the engine would otherwise swallow: in the template config, every CapsLock release. The action of an `Immediate` binding fires at the key-down and is not delayed. A `Tap` decided at that key-up starts its fast action about 0.6 us later. Justified by cut D14 (`:421`) and R1 |
| Main loop | R7 / R13 | master R7 10.8 | R7 10.8; R13 (control window on the loop thread) 10.5 | No cost from the window or its IME window |
| Main-loop round trips under load | R8 | master 15.9 / 20.0 | 15.6 / 19.3 | Equal |

Verdict on latency: no unjustified regression. The bind path is faster for `when: AppEquals` and unchanged elsewhere. The witness is the only added hook-path work, at the measured +0.6 us.

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback: passthrough, one outcome, sentinel first | OK | `keyboard_hook_callback.rs:341-385` unchanged in shape: `code < 0` first, the off gate, `decide_event` checks `is_injected_event` before any state (`:211`), `LRESULT(1)` or `CallNextHookEx` on every path |
| Hook callback: no new blocking, allocation, lock, `log::` | OK | The only new work is the witness `GetAsyncKeyState` (`engine/mod.rs:885` -> `HookEnv::os_key_down`, `keyboard_hook_callback.rs:167-173`). `foreground_app` is a cache read with no call and no allocation (`:175-177`). The title read allocates one `String` as master did (`:179-183`). No `log::` before `mod tests` (`:457`); indexing goes through `slot()` |
| Suppression symmetry | OK, with the ruled exception | A key-up passes iff its key-down passed **or Windows has the key down** (R1). Windows therefore gets a key-up iff it has the key down: no stuck key, no lone key-up. `FocusWhole` checks whole-press symmetry where no event is unseen. CLAUDE.md principle 2 still states the old rule (Hand-off) |
| Engine purity, owner decisions | OK | `#![forbid(unsafe_code)]` kept. The engine gains only `std::fmt` and the `VIRTUAL_KEY` type (`gestures.rs:46`), and no Win32, clock or thread. Matching untouched: exact modifiers, repeat defaults, sided names through `KeySpec::accepts` (`gestures.rs:318-322`) |
| Thread ownership | OK | Three hooks, installed and removed in `install`/`uninstall` on the main thread (`keyboard_hook.rs:189-247`). `WM_ENDSESSION` uninstalls from the window procedure, also on the main thread (`control_window.rs:222-232`). The window is created after the executor guard (`app.rs:215`) and dropped after `uninstall` and before `executor.stop()` (`:292-297`). Resets run only in the drain (`main_thread.rs:714-738`), never in a callback |
| Atomics, `unsafe` | OK | `desktop_hook`: `store(Release)` / `swap(AcqRel)`. No `Relaxed`, `SeqCst` or `static mut` in added lines. Every added `unsafe {` has a `// SAFETY:` within the preceding lines (scripted scan of all changed `.rs` files: 0 misses) |
| Sentinel | OK | No new `SendInput` site. The witness relies on the existing sites sending downs and ups in one call (cut D15) |
| Config reload | OK | Parse on the watcher thread, swap + reset in one main-thread step (`MainState::reload`, `main_thread.rs:243`), outcome logged per `MainThreadError`. A failed parse keeps the live list. The spawn error path is at `config_watcher.rs:158` |
| IPC | OK (untouched) | |
| No key logging | OK | New lines carry signal names, `HRESULT`s and counts only |

## Review status

- Review r1 (`6f6edb3`): CHANGES_REQUESTED, I1 + m1. Triage r1: R-I1 CONFIRMED, 8 REFUTED (the bench flags T-B1..B4, the closed mutation gaps T-M12/M13/M20, the equivalent mutant T-M16), 2 NOTED.
- Fix r1: R-I1 resolved in `c4f1bca` (region ends at the `#[cfg(test)]` + `mod tests {` pair; a coverage assert names `fn spawn_watcher(`; red under both reviewer mutations). m1 resolved in `81bb38c` (doc only).
- Tester r2 (`81bb38c`): **GREEN**. R-I1 is vacuous on its parent and red at HEAD under mutations A/B/C, and 5 more mutations are caught.
- Review r2 (`81bb38c`): **APPROVED** (Critical 0, Important 0, Minor 1).
- Open Minor items worth doing while the context is fresh:
  - review r2 m1 (CONFIRMED Minor): `MovingQuery` in `a_return_signal_reads_the_foreground_window_again` (`main_thread.rs:1645-1688`) maps windows `0x1000`/`0x2000` to PIDs 4096/8192. These are real PID values, so the test fails spuriously when the test process has one of them (`foreground.rs:132-137` sets `own`). Owner phase: **tester**. Use window values that are not multiples of 4 (for example `0x1001`/`0x2001`). Re-check: the test is green and still red under M13.
  - T-N1 (NOTED): the real-clock `stopped < ms(100)` in `a_lock_signal_…` (`main_thread.rs:~1512`) passed every run so far. Leave it unless it flakes.

## Docs and hygiene

- Config meaning changed, and the same branch updates every config surface:
  - `README.md` (reload, rule L and the witness, own-combo DoubleTap, `when:` cache, features, Known Limitations);
  - `src/config/default_config.ron` header (a loading save drops pending gestures and stops macros);
  - `book/src/config/known-issues.md`: rows 9, 55 and 63 Fixed; 68 grows; 76-80 new;
  - `triggers-conditions.md`, `kinds.md` §10, `structure.md` §2.3 and `errors.md`: every new log line, checked against the code strings;
  - `CHANGELOG.md` `[Unreleased]`: Added 1, Changed 4, Fixed 8, Keep a Changelog headings, each entry citing roadmap 2.3 / KS-ID and its tests.
- The cfgcheck harness validated the new RON examples (`cfgcheck-output.txt`, all exit 0).
- Non-config pages (architecture, build-and-run, overview, glossary, roadmap, audit status lines, gaps) are left to the Phase-2 book sync, as the spec says, and are listed in `impl.md` with file:line. Additions below.
- The audit/roadmap status lines for KS-06/KS-29/KS-31 need the merge sha, so they belong to the book sync after the merge.
- Branch hygiene: it contains only this lane's work, with no merges, and can be merged into `master` with `--no-ff` without conflict (merge base = `master` tip `c3dab6a`).

## Technical debt

| Item | Where recorded |
|---|---|
| Residuals 76 (key held through a lock fires once more), 77 (reload drops pending gestures, no key resync), 78 (unseen re-press keeps repeats swallowed; generic injected release), 79 (busy main thread, no WTS registration, UAC without the secure desktop, late lock after Windows Hello), 80 (foreground race for programmatic activate-then-send) | `book/src/config/known-issues.md` rows 76-80 |
| M1: a waiting Tap of a passing `Ctrl+Shift` DoubleTap fires with Ctrl down in Windows (Ctrl+Esc) | row 68, item 3.1; pinned by `a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows` |
| Follow-ups F1 (`mark_completed` repeats -> 3.2), F3 (measure the foreground race -> 3.5), F4 (console/remote disconnect as input-left -> 4.1), F5 (no switch-away event -> 2.4 Raw Input), F6 (WTS registration retry at autostart -> 4.5) | **only in `cut.md`**. They need a row in the roadmap follow-ups at the book sync, or they are lost. F2 (`ChangeWindowMessageFilterEx`) is already in roadmap 4.6 |
| Witness cost +0.6 us per swallowed key-up | here; for the orchestrator/owner |
| Unverified OS behaviour the design relies on: `UOI_IO` FALSE while the lock/Winlogon desktop has input; `EVENT_SYSTEM_DESKTOPSWITCH` delivered to an out-of-context hook; the async key state showing a key pressed on the secure desktop or behind UIPI; lock-signal latency; generic modifier codes for injected input | manual checks M1, M5, M12, M13, M14 (impl.md). Each failure degrades to master's behaviour or to a declared row; none adds a stuck key beyond row 78's stated reliance |
| `src/engine/winner.rs:57` still says "on the hook thread a condition may capture the foreground window (KS-08)"; the executable now comes from the cache | not recorded; developer, next engine-touching lane |
| `MainThreadHandle::reload` repeats `set_enabled`'s send-wake-wait body (deviation 7) | cut addendum only; candidate for 3.4 |
| `README.md` tree line `foreground.rs - the foreground window for when:: …` (double colon) | not recorded; doc-writer, book sync |
| A window of this process gets `app = None` and no title read (`foreground.rs:132-137`). If Windows reports the daemon's PID for its own console window (the usual console-ownership idiom; unverified), a `when:` aimed at the daemon's console no longer matches, where master's lookup did | here only; negligible |
| `WaitableTimer::not_a_timer` (test only) uses `.expect("an event is made")`, not the `"invariant: ..."` form | here only; trivial |

## Problems

None blocks the merge. These are the small items, each with one owner:

### P1. `MovingQuery` PIDs can equal the test process's PID (spurious red)
- **What / Impact.** `a_return_signal_reads_the_foreground_window_again` fails with no product defect when the libtest process gets PID 4096 or 8192. That is rare per run, and more likely on a freshly booted CI runner.
- **Root cause.** `owner()` returns the HWND value as the PID, and `0x1000`/`0x2000` are multiples of 4, which is what real Windows PIDs are. `window_changed` then sets `own = true`.
- **Return to:** `tester` (test-only).
- **Re-check condition:** the fake owners are not multiples of 4; the test is green and still red under M13.

### P2. A stale doc line on the election
- **What.** `src/engine/winner.rs:57`: "on the hook thread a condition may capture the foreground window (KS-08)".
- **Root cause.** M7's touch set named `mod.rs` lines but not this sibling.
- **Return to:** `developer` (next engine lane, doc only).
- **Re-check condition:** the line says the executable comes from the cache and a title condition reads the title.

### P3. A README typo
- `README.md` tree, `foreground.rs` entry: `when::`.
- **Return to:** `doc-writer` (book sync).

## Positives

- The witness turns ruling R1 into one short-circuited read in the pure engine. It is modelled exactly by `FakeEnv`, and it covers every source of an unseen key-down, the hook timeout included, with no per-event cost elsewhere.
- R2 is solved structurally. Only "input left" resets, and `release_all` reads nothing, so repeated or reordered signals cannot discard post-return state. Each policy line is a one-line mutation that a named pin catches.
- The reload is atomic with respect to key events: swap and reset run in one main-thread step, through the existing request queue.
- The process query has left the hook path. The bench shows a conditional binding at the cost of an unbound key.
- The tester's mutation campaign found three real test gaps (M12, M13, M20) and closed them. The reviewer found a vacuous pin (I1), which was fixed with a coverage assert so that the same silent shrink cannot recur.

## Hand-off: Phase-2 book sync (doc-writer)

Everything in `impl.md` "Docs / Left to the Phase-2 book sync" (`architecture.md`, `build-and-run.md`, `overview.md:25`, `glossary.md`, `roadmap.md`, `audit.md` status lines KS-06 `:249` / KS-29 `:624` / KS-31 `:640` with the merge sha, `gaps.md:280`), plus:
1. `roadmap.md` 2.3:
   - a status block (merged sha);
   - the **What** bullet "`reset_transient_state()` … resyncs all 256 keys … after a reload, on `WTS_SESSION_UNLOCK` …": it now resets on the input leaving (lock, or a switch away by `UOI_IO`), releases every key without an OS read, and a reload resets without a key resync (Q2 round 1, Architect Q2 round 2);
   - the latency bullet: the executable is cached per (HWND, pid), and the title is read live (Q8), not cached.
2. `roadmap.md` open question 9: implemented. The follow-up row "For a key whose key-down passed, the OS key state tells a press … from a real auto-repeat" is closed: rejected by cut D3 (a), because an injector or `PressKey` makes the OS state unreliable, and the input-left release covers both cases.
3. `roadmap.md` 3.5: the per-window cache and its acceptance ("100 presses in one window cause one name query") are done in 2.3, and `src/platform/foreground.rs` exists. 3.5 keeps the `when:`-after-match for `sequence`/`chord`, the 100 us budget and F3 (measure the foreground race, row 80).
4. `roadmap.md` issue coverage rows KS-06 (`:728`), KS-29 (`:756`) and KS-31 (`:758`); the 2.3 row of the phase table (`:438`).
5. `roadmap.md` follow-ups: add F1 (-> 3.2), F4 (-> 4.1), F5 (-> 2.4, whose Raw Input sink lives on this window) and F6 (-> 4.5), and the witness cost note (-> 3.5, see Notes).
6. `roadmap.md` "Manual checks after Phase 2" (or Checkpoint 2): the lane's M1-M14 (`impl.md` "Manual checks") with their preconditions. Keep M1's pass criterion (every `state reset` line before the first return line) and M5's `PromptOnSecureDesktop = 1` precondition.
7. The suppression invariant wording wherever the book states it (`roadmap.md:38` RC2, architecture key-event path, glossary): "a key-up is suppressed iff its key-down was, unless Windows has the key down (the witness, roadmap 2.3)".
8. `gaps.md:156` ("Gesture state is never reset"): closed by 2.3. `gaps.md:164` (re-entry under the engine borrow): 2.3 adds a window whose procedure only notes signals, and the title read never asks this process's windows, so nothing pumps under the borrow. Still no counter (2.4).
9. `README.md` tree typo `when::` (P3).

## Hand-off: orchestrator

**To the owner:**
- Run manual checks M1 and M5 with the release daemon and paste the timestamped `daemon.log` lines (one Win+L, one Ctrl+Alt+Del, one UAC round trip). These are the evidence ruling R2 / V2-m1 asked for. No agent may produce it. If a `session locked` line comes after a return line, that is V2-m2's late lock: report it, and a later lane decides. Also run M11-M13 (PrintScreen, UIPI, the secure desktop) once. The witness's correctness for those sources rests on them.
- Report the witness cost: **+0.6 us per key-up of a swallowed key** (R11), 6x the cut's estimate, accepted by ruling R1. The title is read live, not cached (Q8).
- Confirm that the timestamp deferral satisfies V2-m1 (reviewer open question, r1 and r2).

**CLAUDE.md** (the version on disk; the lane changed none of it):
- Layout: `src/hook/` gains `control_window.rs` (`ControlWindow`: a hidden top-level window, WTS session notifications, `WM_QUERYENDSESSION`/`WM_ENDSESSION`, `CONTROL_CLASS` test mutex). The `main_thread.rs` entry gains config reloads (`Request::Reload`, `MainState::reload`), input signals (`PENDING_SIGNALS`, `note_input_signal`, `answer_input_signals`) and the foreground cache. `src/platform/` gains `foreground.rs` (`ForegroundCache`, `WindowQuery`: executable per (HWND, pid), title read live, never for this process's windows) and `desktop.rs` (`input_is_on_this_desktop`, `UOI_IO`). The engine line gains `Engine::reset(env, ResetReason)`, `InputSignal`, rule L and the witness.
- Principle 2: amend the suppression invariant with the witness exception (so a reviewer does not flag `engine/mod.rs:885` as a violation).
- Principle 5: under the `with_main` borrow there now also run the foreground cache's process query (in the foreground callback), the live title read (other processes only), and `update_bindings` + `cancel_actions` (reload/reset). Bindings are swapped by `MainState::reload` on the main thread.
- Principle 6: the main thread installs and removes **three** hooks and owns the control window. The window is created after the executor guard and dropped after `uninstall`, before `executor.stop()`. `WM_ENDSESSION` uninstalls from the window procedure. Reloads and input signals are served in the drain (requests, then signals, then timers). A window procedure or WinEvent callback only notes and posts `MAIN_WAKE`; it never resets.
- Build and gate: bench rows are now R1-R13 (R9/R10 conditional key-down, R11 witness, R12 unbound key, R13 loop with the control window); their comparison rule is `p2-2.3-control-window/cut.md` D14.

**Agent files:**
- `tester.md` §4b: name rows R9-R13 and D14's rule. Note that D14's R11 allowance (+0.5 us) is below the measured +0.6 us, so later lanes compare R11 against this lane's HEAD (5.4 us). Fake PIDs in tests must not be multiples of 4 (P1). A test that registers the control window class takes `CONTROL_CLASS`.
- `code-reviewer.md` / `architect.md` checklists:
  - the hook path must not query a process (`foreground_app` reads the cache);
  - a window procedure or WinEvent callback only notes a signal;
  - a reset never runs inside a callback;
  - a lost-input reset reads no OS state;
  - the witness is the only sanctioned read of the event's own key.
- `docs/WORKFLOW.md` / rulings practice: a ruling that asks for live-daemon evidence (V2-m1) should be routed to the owner as a manual check from the start, not to `impl.md`, since agents may not run the daemon.

## Notes for later lanes

- 2.4: the Raw Input sink (`RIDEV_INPUTSINK`) attaches to this control window. F5 says the sink is also the second look at input on another desktop if M1/M5 show that no switch-away event comes.
- 3.5: the witness could be skipped when nothing that hides key events has happened since the key's key-down. One way is an epoch that foreground changes, resets and remapping switches bump, recorded at `mark_blocked`. That trades away the hook-timeout and injector cases. Weigh it against the +0.6 us only if the hot-path budget needs it.
- 4.6: `ChangeWindowMessageFilterEx` for `TaskbarCreated` on this window (F2, already in the roadmap).
- 4.5: retry `WTSRegisterSessionNotification` at autostart (F6). Until then, a failed registration answers a lock only through the desktop switch (row 79).
- CI watch at the merge step: `the_control_window_is_a_hidden_top_level_window_released_on_drop` creates a real hidden top-level window and tries a WTS registration on the runner. Both outcomes are accepted (M5), but this is the first top-level window in the suite.
