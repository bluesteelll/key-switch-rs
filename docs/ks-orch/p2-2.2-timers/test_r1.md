# Tester: p2-2.2-timers / roadmap 2.2 (HEAD bfe931f vs master c9c1373; developer HEAD d738632)

Verdict: GREEN. Gate green, every red-first red on its parent and green on HEAD, 23 own mutations all caught, every acceptance bullet mapped. One test gap found (the callback's own `is_enabled` gate, mutation M22 uncaught) and closed by a `Tests:` commit, shown red under the mutation and green without it.

Tree: `D:/key-switch-rs/key-switch-rs`, branch `feat/main-thread-timers`, `git status --short` empty, `git worktree list` shows only the lane tree. All scratch worktrees (`wt/parent`, `wt/mut`, `wt/benchm`, `wt/benchh`) removed.

## Gate (HEAD bfe931f = d738632 + one Tests: commit)
- `cargo fmt --check`: OK (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: OK (after `touch src/lib.rs`, "Checking key-switch-rs ... Finished", not stale).
- `cargo test --locked --no-fail-fast` (final HEAD bfe931f):
  - `running 636 tests` / `test result: ok. 630 passed; 0 failed; 6 ignored` (lib)
  - `running 0 tests` (swch bin, no tests), `running 6 tests` / `ok. 6 passed; 0 failed; 0 ignored` (tests/ipc_roundtrip.rs), `running 0 tests` (doc-tests)
  - At d738632 (before my commit): `running 635 tests`, `629 passed; 0 failed; 6 ignored`.
  - 6 ignored: `bench::latency_bench` (`ignored, bench:`), plus 5 `manual:` (`the_win32_runner_opens_a_url`, `foreground_context_names_the_focused_notepad`, `hold_the_daemon_pipe_name_for_a_manual_start`, `the_menu_mask_key_is_inserted_into_the_input_stream`, `live_registry_with_the_language_hotkey_not_assigned_has_no_combination`). Not run, not counted as passes.
  - The suite prints nothing (log scanned for any line other than test/result/cargo lines: none).
- `cargo build --release --locked`: OK (not run).
- Safety scans of the diff (`master..HEAD -- src tests`): no `key-switch-rs.sock`, no `config.ron` write, no `SendInput(` call added (one comment hit), no `SetWindowsHookEx`, no `RegSetValue/RegCreate`, no `Relaxed|SeqCst|static mut`, no print macros. The `FastAction::run` under `cfg(test)` only records (`windows_actions.rs:148-165`), so no test injects.
- Real-clock flake check: `hook::main_thread` (11 tests, includes the 200 ms Hold and the sent-message test) run 5 times: `11 passed` x5, 1.02 s each. `hook::` x5 after my commit: `56 passed; 0 failed; 2 ignored` x5.

## Red-first replay (worktree `wt/parent`, own `CARGO_TARGET_DIR=target-wt/parent`)

| Test | Parent / red commit | HEAD | Note |
|---|---|---|---|
| `scenarios::stale_tap_gc_eats_second_quick_tap`, `scenarios::stale_hold_timer_fires_new_press_early` (C2 e969dd7) | red at 7cb3521 by assertion (repro replay, below); green on master by design | ok (in the 25-test run) | Q6 |
| `engine::schedule::tests::*` (2), `engine::tests::{a_token_from_before_a_reset_is_stale, a_cancelled_hold_leaves_its_token_stale, a_timer_the_scheduler_refuses_is_forgotten}` (C3 68b13a0) | compile-red: `E0432 unresolved imports crate::engine::Scheduler, crate::engine::TimerQueue` and `E0432 unresolved import super::super::TimerSlot` | ok | covered by M4b, M5, M6, M8, M9 |
| `hook::keyboard_hook_callback::tests::the_gesture_path_starts_no_thread` (C5 e9fc2f4) | red by assertion: `running 606 tests`, `FAILED. 599 passed; 1 failed`, panic `threads started on the gesture path:` | ok | also M20 |
| 11 `hook::main_thread::tests::*`, `a_thousand_hold_presses...`, `remapping_off_callbacks_pass_without_asking_for_the_engine`, `a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop`, `core::app::tests::daemon_handle_switches_remapping_through_the_main_thread` (C6 1aa717f) | compile-red: 41 errors (`E0433 MainThread` x10, `E0425 with_main` x9, `plan` x6, `REFUSED`, `MAIN_WAKE`, `E0560 DaemonHandle has no field main`, `E0603 decide_event is private`) | ok | covered by M1-M3, M11, M12, M13, M17, M19, M21, M22 |
| C8 scenarios (b68c7f2) | red by assertion: `running 268 tests`, `FAILED. 257 passed; 11 failed` (10 scenarios + `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`), e.g. `up CapsLock at +250 ms: expected to fire [], fired [SwitchLanguage]`, `down CapsLock at +80 ms: expected to fire [], fired [SwitchLanguage]` (KI 74), `the Tap waits for the window` | ok | |
| C10 (c4609ea) `a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap`, `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap`, `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold` | red by assertion: `FAILED. 265 passed; 3 failed`; `down LShift at +150 ms: expected to fire [], fired [ToggleCapsLock]`; `up LCtrl at +90 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`; `advance 800 ms to +1130 ms: expected to fire [], fired [SwitchLanguage]` | ok | |
| I1 `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` | green on c4609ea by design; red as mutation M1 of c5ef59a | ok | M1 |
| C12 (64b4ead) `a_chord_start_asks_for_a_timer_at_the_end_of_its_window`, `a_chord_expiry_timer_drops_the_pending_chord`, `each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale` | red by assertion: `FAILED. 269 passed; 3 failed`, `one expiry timer, got []` | ok | |
| `platform::priority::tests::raise_sets_highest_and_drop_restores_normal` (C15 0a8caec) | compile-red: `E0432 unresolved import crate::platform::priority::RaisedPriority` | ok | M7 |
| `core::windows_actions::tests::under_test_a_fast_action_is_only_recorded` (M7 7cb6868) | not run red (would post to the foreground window; declared in impl.md/cut.md) | ok | safety test, behaviour read in code |

Green run of the named acceptance and red-first tests on HEAD (d738632): `running 25 tests` ... `test result: ok. 25 passed; 0 failed; 0 ignored`.

KS-11 repro replay at 7cb3521 (C2): `git archive 7cb3521` into `D:/tmp/ks-orch/p2-2.2-timers/repro-r1/snapshot`, crate `gesture-repro` of methodology.md with the two tests, each in original and inverted form, `--test-threads=1 --nocapture`:
```
running 4 tests
test inverted_stale_hold_timer_fires_new_press_early ... press#2 hold still pending at +110ms = false
  panicked ... assertion failed: hold_still_pending()  FAILED
test inverted_stale_tap_gc_eats_second_quick_tap ... second tap fired = false
  panicked ... assertion failed: tap_second_fired()  FAILED
test original_stale_hold ... ok
test original_stale_tap ... ok
test result: FAILED. 2 passed; 2 failed
```
The originals (bug present) pass and the inverted ones fail at 7cb3521, same timings (20/100/110 ms) as the ported scenarios, which are green on HEAD.

## Mutations (scratch worktree `wt/mut`, own target dir; each reverted with `git -C wt/mut checkout -- .`, `git diff --exit-code` clean, worktree removed)

| # | File | Mutation | Named test (before run) | Result |
|---|---|---|---|---|
| M1 | gestures.rs:319 | drop `&& !hold...now >= *due` (ruling I1) | `scenarios::a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` | caught: `running 1 test`, FAILED |
| M2 | main_thread.rs:462 | `QS_ALLINPUT` -> `QS_ALLEVENTS` (OQ1) | `main_thread::tests::a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer` | caught, panic at `main_thread.rs:880`: `every sent message is answered: [1.0048224s, ...]` (each sent message waited 1 s; 100 s run) |
| M3 | main_thread.rs:463 | `MWMO_INPUTAVAILABLE` -> flags 0 | `a_message_posted_behind_a_wake_is_dispatched_without_more_input` | caught, `the WM_CLOSE posted behind a MAIN_WAKE was not dispatched within 1 s` |
| M4a | gestures.rs on_timer | drop `p.seq == seq` in the press branch | `gestures::tests::a_stale_timer_does_not_fire_a_newer_press` | caught (only that test: the scenario `stale_hold_timer_fires_new_press_early` is shielded by queue pruning, `is_live`, so the unit test carries it) |
| M4b | mod.rs on_timer + gestures.rs | drop the generation guard and the seq check | `engine::tests::a_token_from_before_a_reset_is_stale` | caught, panic `mod.rs:946` (note: dropping only the generation guard is not caught, M4; it is redundant with `gestures.clear` and the never-resetting `seq`, an equivalent mutation, defence in depth) |
| M5 | schedule.rs take_due | ties handed out in reverse start order | `schedule::tests::the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order` | caught |
| M6 | schedule.rs retain | keep the stale, drop the live | `schedule::tests::retain_keeps_only_live_timers` | caught |
| M7 | priority.rs Drop | no restore to NORMAL | `platform::priority::tests::raise_sets_highest_and_drop_restores_normal` | caught |
| M8 | mod.rs start_timers | no `forget_timer` for a refused timer | `engine::tests::a_timer_the_scheduler_refuses_is_forgotten` | caught |
| M9 | mod.rs start_timers | no `scheduler.retain(is_live)` | `engine::tests::a_cancelled_hold_leaves_its_token_stale` | caught |
| M10 | chords.rs expire | `p.seq != seq` -> `==` | `engine::tests::a_chord_expiry_timer_drops_the_pending_chord` | caught (the start test stays ok, as it should) |
| M11 | main_thread.rs plan | `at <= now` -> `at < now` | `main_thread::tests::plan_sets_the_timer_only_for_another_deadline` | caught |
| M12 | keyboard_hook_callback.rs decide_event | sentinel check off | `injected_and_unknown_events_bypass_the_engine` | caught |
| M13 | main_thread.rs set_enabled | no `engine.reset` | `main_thread::tests::remapping_switches_through_the_main_thread_and_drops_pending_timers` | caught |
| M14 | gestures.rs:250 | Hold term boundary `now >= due` -> `>` | `scenarios::equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` | caught (3 fail: that, `gestures::tests::a_tap_and_a_hold_of_one_press_fire_exactly_one`, `a_hold_is_cancelled_before_its_term_and_left_to_its_timer_after_it`) |
| M15 | gestures.rs other_key_down | cancel a swallowed waiting press too | `scenarios::another_key_does_not_cancel_a_swallowed_double_tap` | caught (2 fail) |
| M16 | gestures.rs other_key_down | cancel a Held press even when its key is swallowed | `scenarios::key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` | caught (4 fail) |
| M17 | main_thread.rs start | wake only for a LATER deadline (`<` -> `>`) | `main_thread::tests::a_timer_started_for_an_earlier_deadline_resets_the_wake` | caught |
| M18 | mod.rs on_timer | `decision.mask = true` | `scenarios::a_hold_timer_never_masks` | caught (50 fail via the harness assert) |
| M19 | main_thread.rs start | wake 30 ms late | `main_thread::tests::a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` | caught: `a 200 ms Hold fired 30.6342ms late in the median` |
| M20 | main_thread.rs start | `std::thread::spawn` on the gesture path | `keyboard_hook_callback::tests::the_gesture_path_starts_no_thread` | caught: `src/hook/main_thread.rs:255: self.queue.start(request); let _ = std::thread::spawn(|| {});` |
| M21 | main_thread.rs GestureTimers::retain | no-op | `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` | caught: `1000 timers queued after 1000 presses` |
| M22 | keyboard_hook_callback.rs:326 | remove the callback's own `is_enabled` gate | `remapping_off_callbacks_pass_without_asking_for_the_engine` | **NOT CAUGHT** (ok), see F1; after my test: caught |
| M23 | keyboard_hook_callback.rs decide_event | remove decide_event's second `is_enabled` check | `disabled_hook_passes_every_event_without_asking_the_engine` | caught |

## Acceptance

| Criterion | Evidence | Status |
|---|---|---|
| Ports of `stale_tap_gc_eats_second_quick_tap` / `stale_hold_timer_fires_new_press_early` inverted and pass; red at 7cb3521 | ok on HEAD (25-test run); repro replay above: inverted FAILED at 7cb3521, originals pass | verified |
| Tap(200)+Hold(200): 199 ms only Tap, 201 ms only Hold | `scenarios::tap_and_hold_with_equal_terms_fire_exactly_one` ok (releases/advances at 199 and 201); `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` ok; M14 caught | verified |
| No thread per press over 1000 presses | `the_gesture_path_starts_no_thread` ok (M20 caught); `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` ok (M21 caught). No thread count is measured, the structural scan plus the queue bound is the accepted form | verified |
| Real-timer 200 ms within a few ms | `main_thread::tests::a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` ok, 5 reruns ok; M19 caught | verified |
| Another key-down cancels a pending pass-through gesture; KI-67 flips | the three C10 tests ok (red on c4609ea, green on HEAD); swallowed kept: `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`, `another_key_does_not_cancel_a_swallowed_double_tap` ok (M15, M16 caught); I1 ok (M1 caught) | verified |
| KI-74 no first-binding fire | `scenarios::a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins` ok (red on b68c7f2) | verified |
| Latency bench runs; master and HEAD numbers in impl.md and test_r1.md; no unjustified regression | tester runs below; impl.md has the developer's | verified, see Bench |
| Timers on the main thread, tokens, stale token does nothing | C3 tests, M4a/M4b/M8/M9 caught; `daemon_handle_switches_remapping_through_the_main_thread` ok | verified |
| Deadlines not rounded; hook callbacks delivered without delay | M19 caught; `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer` ok (M2 caught); `platform::timers::tests::a_timer_signals_once_after_its_due_time` ok; bench R6/R7 | verified (hook delivery with a real hook: manual M9/M10) |
| Drain before every wait, `MWMO_INPUTAVAILABLE` pinned | `a_message_posted_behind_a_wake_is_dispatched_without_more_input` ok (M3 caught) | verified |
| `Mutex<Engine>` gone; fast gate kept; safe re-entry | `rg "Mutex<Engine>\|with_engine" src` exit 1 (empty); `rg "Mutex<Engine>\|with_engine\|DECIDING\|DecidingGuard\|start_hold_timer\|sleep_until" src` exit 1 (empty); `remapping_off_callbacks_pass_without_asking_for_the_engine` ok; `remapping_off_keyboard_callback_returns_before_it_times_itself` (mine) pins the first gate; `with_main_refuses_reentry_on_the_same_thread`, `set_enabled_refuses_a_nested_call` ok | verified |
| Tap waits for DoubleTap only when both | C8 tests (`tap_and_double_tap_on_one_key_wait_for_the_double_tap_window_before_the_tap`, ...) ok, `tap_100_and_hold_200_released_at_150_fire_nothing`, `a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up` ok | verified |
| Chord and Sequence expiry timers | C12 tests ok (M10 caught) | verified |
| Scheduler trait; scenarios drive time through the fake | `engine::schedule::tests::*` ok (M5, M6 caught) | verified |
| `on_timer` never masks | `a_hold_timer_never_masks` ok; M18 caught | verified |
| Thread priority evidence | `raise_sets_highest_and_drop_restores_normal` ok (M7 caught); R8 below (0 of 3 worse), D9 stays | verified (live effect: manual M10) |
| R-7 hold-timer half | no thread on the gesture path (M20) | verified; config-watcher half stays open (impl.md) |
| Test-only safety (ruling M7) | `under_test_a_fast_action_is_only_recorded` ok | verified |

Manual, not verified (no ignored test; impl.md M1-M10, `cfg/manual-*.ron`): M1 Tap+Hold CapsLock feel, M2 `<T>` with LShift DoubleTap, M3 Ctrl+C / Ctrl+click, M4 Ctrl+Shift Hold(1000) selection, M5 Tap/DoubleTap F9 timing, M6 KI-74 live, M7 `swch off/on/status/exit` answers within 1 s, M8 Ctrl+C in the daemon console while a Hold is held, M9 WinEvent delivery through the new loop, M10 typing under CPU load and base priority 10. Existing 5 `manual:` ignored tests unchanged. Documentation text (README, CHANGELOG, book) was not reviewed here; that is the reviewer's.

## Bench (D10; apart from tests, never a pass)
Worktrees at 611c83f (master side, C1) and d738632 (HEAD side), release, own target dirs, `--ignored --exact --test-threads=1`, 3 interleaved runs each. Raw: `D:/tmp/ks-orch/p2-2.2-timers/bench-r1/{master-611c83f,head-d738632,head-load}.md`; summary script `benchsum.py`. Median of p50, largest p99, us.

| Row | master p50 | master p99 max | HEAD p50 | HEAD p99 max | D10 rule | Verdict |
|---|---|---|---|---|---|---|
| R1 on_key typing | 0.100 | 0.300 | 0.100 | 0.400 | p50 <= 0.225 | pass |
| R2 on_key gestures | 0.200 | 0.500 | 0.200 | 0.600 | p50 <= 0.350 | pass |
| R3 on_timer | 0.100 | 0.300 | 0.100 | 0.200 | p50 <= 0.225 | pass |
| R4 callback | 5.300 | 64.0 | 5.200 | 47.6 | p50 <= 7.125, p99 <= 98.0 | pass |
| R5 Hold key-down | 35.3 | 161.5 | 5.3 | 63.2 | p50 <= master | pass (6.7x lower) |
| R6/5 lateness | 676.4 | 3013.0 | 417.3 | 662.6 | p50 <= 1000 and <= master + 500 | pass |
| R6/20 | 673.3 | 957.7 | 405.6 | 2163.5 | same | pass |
| R6/50 | 607.8 | 1334.6 | 485.6 | 782.6 | same | pass |
| R6/200 | 773.5 | 1545.8 | 357.5 | 1667.1 | same | pass |
| R7 round trip | 12.6 | 108.7 | 15.5 | 186.5 | p50 <= 32.6, p99 <= 1.5 x 108.7 + 50 = 213.1 | pass (p50 +2.9 us) |

R8 (`KS_BENCH_LOAD=1`, all cores loaded, HEAD d738632, one-way delivery, p99 us; rule: HIGHEST worse iff p99_H > 1.25 x p99_N + 20):

| Run | NORMAL p50/p99 | HIGHEST p50/p99 | Worse? |
|---|---|---|---|
| 1 | 19.3 / 482.9 | 23.7 / 372.2 | no (372.2 <= 623.6) |
| 2 | 18.6 / 537.3 | 23.4 / 281.8 | no (281.8 <= 691.6) |
| 3 | 18.9 / 467.5 | 20.9 / 386.7 | no (386.7 <= 604.4) |

0 of 3 runs worse: D9 (HIGHEST) stays. HIGHEST p50 is 2-5 us higher in every run, p99 is 23-48% lower. No rule breach in any row on the first try. The machine was in normal use by the owner during the runs.

## Failures
### F1. Uncaught mutation M22 (fixed by a tester commit)
- File: `src/hook/keyboard_hook_callback.rs:326` (`keyboard_hook_callback`'s own `if !hook.is_enabled()` fast gate).
- Mutation `if false {` there: `remapping_off_callbacks_pass_without_asking_for_the_engine` stays `ok`, the whole `hook::` run reported 1 failure only for M23, not M22.
- Cause: on master the switch was checked only under the engine lock inside `decide_event`, so the callback's gate was what kept the lock free and the successor-of-A5-m1 test saw it. This lane added a second `is_enabled` check at the top of `decide_event` (before `with_main`), so the `REFUSED` counter reads 0 with or without the callback gate. The spec asks to keep the fast gate "pinned by ... or its successor"; the successor no longer pins the first one.
- Fix (tests only): `Tests: the keyboard callback's own switch check is pinned by the callback gauge` = commit `bfe931f`. New test `hook::keyboard_hook_callback::tests::remapping_off_keyboard_callback_returns_before_it_times_itself`: with remapping off, 100 down/up pairs through the extern callback leave `COUNTERS.longest_callback()` at zero (only code after the first gate records it), with remapping on they set it (control). Under M22: FAILED `with remapping off the callback passes the event before it times itself, left: 12.4µs`; on HEAD: ok; 5 reruns of `hook::` ok. The gauge is written only by callbacks, and every test calling the callback holds `GLOBAL_HOOK`, so the reset-and-read cannot race.

No other red.

## Tests added by the tester (`Tests:` commits)
- `bfe931f` `src/hook/keyboard_hook_callback.rs`: `remapping_off_keyboard_callback_returns_before_it_times_itself` (test code only, branch `feat/main-thread-timers`, trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`). Tree clean after it.

## Notes
- Equivalent mutation recorded: removing only the `generation` guard in `Engine::on_timer` is not observable (reset clears every machine and `seq` never resets); keep it as defence in depth, no test needed.
- `stale_hold_timer_fires_new_press_early` (scenario) does not catch a missing seq check in `Gestures::on_timer` because the harness queue is pruned by `is_live`; the gestures unit test `a_stale_timer_does_not_fire_a_newer_press` does (M4a). Both layers are covered.
- The test commits C3, C6, C15 do not compile their tests by design (declared); clippy/tests were not run on those intermediate commits.
- `cargo test` `--ignored` was never run; the bench ran only through its own `--ignored --exact bench::latency_bench`, which installs no hook and injects nothing (bench.rs header, grepped: no `SendInput`, `SetWindowsHookEx`, registry).
- Residual (not this lane): config-watcher `update_bindings -> cancel_actions` wake at exit; follow-ups listed in impl.md.
