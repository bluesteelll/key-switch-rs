# Implementation: p2-2.2-timers round 1

Verdict: DONE. Branch `feat/main-thread-timers`, 18 commits on `master` `c9c1373`, HEAD `d738632`,
tree clean. Cut: `cut.md` as amended by `rulings.md`; how each ruling was done is in the
"Developer addendum" at the end of `cut.md`.

## Items

| # | Commit | Item | Red-first (test: red output -> green) |
|---|---|---|---|
| C1 | `611c83f` | D10 bench `src/bench.rs`, `src/lib.rs` | n/a (ignored bench); run 3x, `bench-master.md` |
| C2 | `e969dd7` | KS-11 repro ports | `stale_tap_gc_eats_second_quick_tap`, `stale_hold_timer_fires_new_press_early`: green on master by design (Q6); red only at `7cb3521` through the repro replay (tester) |
| C3 | `68b13a0` | (R) D4 tests | compile: `E0432 unresolved import super::super::TimerSlot`, `E0432 unresolved imports crate::engine::Scheduler, crate::engine::TimerQueue` -> green in C4 |
| C4 | `cfd7be6` | D4 tokens, `Scheduler`, `TimerQueue`, `start_timers`, `is_live`; harness on `TimerQueue` | behaviour-neutral; C3's 5 tests green |
| C5 | `e9fc2f4` | (R) KS-33 scan `the_gesture_path_starts_no_thread` | `threads started on the gesture path: src/hook/keyboard_hook_callback.rs:91: use std::thread; src/hook/keyboard_hook_callback.rs:312: let spawned = thread::Builder::new()` -> green in C7 |
| M7 | `7cb6868` | ruling M7: `FastAction::run` only records under test | `under_test_a_fast_action_is_only_recorded` (not run red: it would post WM_NULL to the foreground window) |
| C6 | `1aa717f` | (R) main-thread tests (11 in `main_thread.rs`, 3 adapter, 1 app) | compile, 41 errors: `E0433 MainThread`, `E0425 with_main`, `plan`, `REFUSED`, `MAIN_WAKE`, `GestureTimers`, `E0560 DaemonHandle has no field main`, `E0603 decide_event is private` -> green in C7 |
| C7 | `464d3d4` | D1-D3, D7: `hook::main_thread`, engine off the mutex, loop, `swch` requests, test migrations (M1) | C5 and C6 green |
| C8 | `b68c7f2` | (R) D5 scenarios + M6 tests | see below -> green in C9 |
| C9 | `506aad3` | D5: one press machine, waiting Tap, group DoubleTap (KS-25, known issue 74), `GroupArming`, property config `tap_double_tap` | C8 green; all earlier scenarios unchanged and green |
| C10 | `c4609ea` | (R) D6 / known issue 67 flips + I1 scenario | see below -> green in C11 |
| C11 | `c5ef59a` | D6 `other_key_down` + ruling I1 | I1 red then green, below |
| C12 | `64b4ead` | (R) D8 expiry-timer tests | `one expiry timer, got []` (x3) -> green in C13 |
| C13 | `3e516f9` | D8 chord and sequence expiry timers | C12 green |
| C14 | `4e9dd62` | bench R8 (ruling M5) | run 3x with load, `bench-head-load.md` |
| C15 | `0a8caec` | (R) `raise_sets_highest_and_drop_restores_normal` | `E0432 unresolved import crate::platform::priority::RaisedPriority` -> green in C16 |
| C16 | `2f11917` | D9 `platform::priority::RaisedPriority`, `App::run` | C15 green |
| - | `d738632` | comment-only follow-up (adapter Threads, key-down comment) | n/a |

C8 red outputs (before C9):
- `tap_300_and_hold_200_released_at_250_fire_only_the_hold`: `up CapsLock at +250 ms: expected to fire [], fired [SwitchLanguage]`
- `a_hold_closes_the_double_tap_window`: `down F13 at +280 ms: expected to fire [], fired [ToggleCapsLock]`
- `a_double_tap_press_arms_no_hold`: `advance 300 ms to +400 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]`
- `tap_and_double_tap_on_one_key_wait_for_the_double_tap_window_before_the_tap`, `a_double_press_of_a_tap_and_double_tap_key_fires_only_the_double_tap`, `another_key_fires_a_tap_that_waits_for_the_double_tap_window`, `a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down`: `up F13 at +80 ms: expected to fire [], fired [SwitchLanguage]`
- `a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window`: `up F13 at +50 ms: expected to fire [], fired [SwitchLanguage]`
- `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins`: `down CapsLock at +80 ms: expected to fire [], fired [SwitchLanguage]`
- `a_press_after_a_lost_key_up_replaces_the_stale_press_and_keeps_a_hold_due_by_the_clock`: `advance 90 ms to +330 ms: expected to fire [DoNothing], fired []`
- `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`: panicked `the Tap waits for the window`
- green already, pinning declared rows: `tap_100_and_hold_200_released_at_150_fire_nothing`, `a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up`

C10 red outputs (before C11):
- `a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap` (renamed from `..._still_completes_the_double_tap`): `down LShift at +150 ms: expected to fire [], fired [ToggleCapsLock]`
- `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap` (renamed from `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap`): `up LCtrl at +90 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`
- `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold`: `advance 800 ms to +1130 ms: expected to fire [], fired [SwitchLanguage]`
- green, staying green: `a_lone_passing_ctrl_tap_still_fires`, `another_key_does_not_cancel_a_swallowed_double_tap`
- ruling I1, `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down`: green on C10; with C11's cancel rule written without the clock check it went red, `advance 0 ms to +1001 ms: expected to fire [SwitchLanguage], fired []`; with the clock check (the committed C11) green.

## Tests added or changed

- `engine::scenarios`: `stale_tap_gc_eats_second_quick_tap`, `stale_hold_timer_fires_new_press_early`,
  `tap_300_and_hold_200_released_at_250_fire_only_the_hold`, `tap_100_and_hold_200_released_at_150_fire_nothing`,
  `a_hold_closes_the_double_tap_window`, `a_double_tap_press_arms_no_hold`,
  `tap_and_double_tap_on_one_key_wait_for_the_double_tap_window_before_the_tap`,
  `a_double_press_of_a_tap_and_double_tap_key_fires_only_the_double_tap`,
  `another_key_fires_a_tap_that_waits_for_the_double_tap_window`,
  `a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window`,
  `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins`,
  `a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up`,
  `a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down`,
  `a_press_after_a_lost_key_up_replaces_the_stale_press_and_keeps_a_hold_due_by_the_clock`,
  `a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap` (flipped),
  `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap` (flipped),
  `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold`,
  `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down`,
  `a_lone_passing_ctrl_tap_still_fires`, `another_key_does_not_cancel_a_swallowed_double_tap`.
- `engine::schedule::tests`: `the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order`, `retain_keeps_only_live_timers`.
- `engine::tests`: `a_token_from_before_a_reset_is_stale`, `a_cancelled_hold_leaves_its_token_stale`,
  `a_timer_the_scheduler_refuses_is_forgotten`, `a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`,
  `a_chord_start_asks_for_a_timer_at_the_end_of_its_window`, `a_chord_expiry_timer_drops_the_pending_chord`,
  `each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale`.
- `engine::gestures::tests` rewritten (15 tests). Mapping of the old per-table tests:
  `arm_tap_inserts_then_resolve_removes`, `resolve_tap_with_no_pending_returns_none`,
  `tap_released_at_or_after_its_term_does_not_fire` -> `a_tap_alone_fires_at_a_key_up_before_its_term`;
  `arm_hold_inserts_then_cancel_removes`, `cancel_hold_with_no_pending_returns_false`,
  `hold_timer_fires_its_own_entry_once`, `hold_released_at_or_after_its_due_is_left_to_its_timer`
  -> `a_hold_is_cancelled_before_its_term_and_left_to_its_timer_after_it`;
  `hold_left_to_its_timer_does_not_block_a_new_arm` -> `a_due_hold_does_not_block_the_next_press`;
  `stale_hold_timer_does_not_fire_a_newer_press` -> `a_stale_timer_does_not_fire_a_newer_press`;
  `forgotten_hold_never_fires_and_frees_its_key` -> `a_forgotten_hold_never_fires_and_a_forgotten_window_keeps_its_tap`;
  `double_tap_first_press_arms_second_press_fires`, `double_tap_third_press_arms_fresh_pending`,
  `double_tap_after_the_term_arms_again` -> `a_double_tap_fires_on_a_second_press_of_its_group_within_its_window`;
  `double_tap_fires_the_first_press_action` (it pinned the per-key entry of known issue 74) ->
  `a_window_closes_for_a_press_another_group_wins` and the scenario above;
  `tap_and_hold_pending_simultaneously_on_same_key` (it pinned KS-25's independence) ->
  `a_tap_and_a_hold_of_one_press_fire_exactly_one`;
  `expired_tap_is_replaced_by_the_next_arm`, `auto_repeat_keydown_does_not_stack` ->
  `a_held_press_left_by_a_lost_key_up_is_replaced` (repeats never reach `press`; pinned at the
  engine level by `held_hold_binding_with_repeating_action_fires_once_across_many_repeats` and
  `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release`);
  `clear_drops_every_entry_and_keeps_the_generation` -> `clear_drops_every_press_and_due_hold`
  (the generation moved to the engine: `a_token_from_before_a_reset_is_stale`). New:
  `a_tap_with_a_double_tap_waits_for_the_window_and_its_timer_fires_it`,
  `a_press_won_by_no_group_fires_the_waiting_tap_and_closes_the_window`,
  `another_key_down_fires_a_waiting_tap`, `a_timer_fires_the_taps_whose_windows_closed_before_it`,
  `another_key_down_cancels_a_passing_press_but_not_a_swallowed_one_or_a_due_hold`.
- `engine::chords` / `engine::sequences` unit tests: every call takes a seq; a fresh chord expects
  `ChordOutcome::Started { due }`, a sequence step `SequenceOutcome::Advanced { .. }`;
  `first_chord_key_arms_pending` also checks `is_live` and `expire`.
- `engine::properties`: new last config `tap_double_tap` (CapsLock Tap(200)+Hold(400)+DoubleTap(300);
  LShift Tap(200)+DoubleTap(300), passing); the seeds of the earlier configs are kept.
- `hook::main_thread::tests` (11): `with_main_refuses_reentry_on_the_same_thread`,
  `with_main_works_again_after_a_call`, `with_main_is_none_without_a_main_state`,
  `a_second_main_thread_on_one_thread_is_refused`, `plan_sets_the_timer_only_for_another_deadline`,
  `a_timer_started_for_an_earlier_deadline_resets_the_wake`,
  `a_message_posted_behind_a_wake_is_dispatched_without_more_input`,
  `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer`,
  `a_200_ms_hold_fires_within_a_few_ms_of_its_deadline`,
  `remapping_switches_through_the_main_thread_and_drops_pending_timers`,
  `a_switch_request_to_an_ended_main_thread_fails_at_once`.
- `hook::keyboard_hook_callback::tests`: `the_gesture_path_starts_no_thread`,
  `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued`,
  `remapping_off_callbacks_pass_without_asking_for_the_engine`,
  `a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop`; migrations as in the addendum (M1).
- `hook::keyboard_hook::tests`: migrations as in the addendum (M1).
- `core::app::tests::daemon_handle_switches_remapping_through_the_main_thread`; migrated
  `daemon_handle_refuses_exit_without_a_main_thread`.
- `core::windows_actions::tests::under_test_a_fast_action_is_only_recorded`.
- `platform::priority::tests::raise_sets_highest_and_drop_restores_normal`.
- `bench::latency_bench` (ignored, `bench:` reason; not a manual check, never a pass).

## Remarks (critique and rulings)

Every remark ends RESOLVED; details and pinning tests are in the addendum of `cut.md`.
- I1 - RESOLVED at `src/engine/gestures.rs` (`other_key_down`, Held arm), pinned by `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` and `another_key_down_cancels_a_passing_press_but_not_a_swallowed_one_or_a_due_hold`.
- M1 - RESOLVED (migration list in the addendum).
- M2 - RESOLVED in `known-issues.md` row 75, `triggers-conditions.md` §11.6, CHANGELOG.
- M3 - RESOLVED in `known-issues.md` row 67, `triggers-conditions.md` §11.8, README, CHANGELOG.
- M4 - RESOLVED: reply `sync_channel(1)` + `try_send` (`main_thread.rs` `serve`); a timed-out request still runs (documented on `MainThreadError::NoAnswer`, CHANGELOG); `App::run` drops the `MainThread` after `run`; pinned by `a_switch_request_to_an_ended_main_thread_fails_at_once`.
- M5 - RESOLVED: R8 one-way at the receiver, 3 runs, margin below.
- M6 - RESOLVED at `gestures.rs` `forget`, `on_timer`, `press`; pinned by the two M6 scenarios, the M6 engine test and two gesture unit tests.
- M7 - RESOLVED at `src/core/windows_actions.rs` `FastAction::run`; pinned by `under_test_a_fast_action_is_only_recorded`.
- M8 - RESOLVED in `a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` and bench R6.
- OQ1 - tester (QS_ALLEVENTS); the MWMO mutation was run here (red as expected, restored).
- OQ2, OQ4, OQ5, Q1-Q7 - RESOLVED as in the addendum.

## Deviations from the cut

Listed in the addendum of `cut.md` ("Deviations from the cut", 11 items): the extra M7 commit and a
comment-only commit; `fn with_main<` in the log scan; the plan step in `GestureTimers::plan_wake`;
the gesture API shape and the M6(a) flush in `on_timer`; `ChordOutcome::Started` and
`SequenceOutcome::Advanced { due }`; one `is_enabled` check in `decide_event`; `MainThread` dropped
before `uninstall` (M4c); R8 by `SetWindowLongPtrW` (no `Win32_Graphics_Gdi`, no Cargo change);
`kinds.md:177,198` and `structure.md:63` updated beyond the line list; extra tests; the executor doc
line left as the cut keeps the executor untouched.

## Unsafe / atomics added

- `src/hook/main_thread.rs`: `MainThreadHandle::request_exit` (`PostThreadMessageW(WM_QUIT)`, "takes the thread id and the message by value and reads no memory of ours; WM_QUIT carries no pointer in its parameters. A thread id that is no longer valid only makes the call fail, and that error is returned"); `wake` (`MAIN_WAKE`, same, error ignored); `MainThread::new` (`PeekMessageW(PM_NOREMOVE)` "`msg` is a local that outlives the call, which only writes it; the call creates this thread's message queue ... and removes nothing", `GetCurrentThreadId`); `run` (`MsgWaitForMultipleObjectsEx`: "the slice holds the gesture timer's handle, which the WaitableTimer in this thread's main state keeps open: only dropping `self` removes that state, and `self` is borrowed for the whole call"); `pump` (`PeekMessageW(PM_REMOVE)`, `TranslateMessage`, `DispatchMessageW`, as the executor's). Tests: `CreateWindowExW` (message-only STATIC), `PostThreadMessageW`/`PostMessageW` to the test thread's own queue, `IsWindow`, `SendMessageTimeoutW(WM_NULL)`.
- `src/platform/priority.rs`: `SetThreadPriority(GetCurrentThread(), ..)` in `raise` and `Drop` ("GetCurrentThread returns the calling thread's pseudo handle, which needs no closing; SetThreadPriority takes it and the priority by value and changes only that thread's priority"); test `GetThreadPriority`.
- `src/bench.rs` (test-only): extern callback call, `CreateWindowExW`, `SendMessageTimeoutW`, `PostThreadMessageW`, `SetThreadPriority`, `SetWindowLongPtrW(GWLP_WNDPROC)` and its restore before `DestroyWindow`, `DefWindowProcW` in the probe procedure.
- Removed: `DaemonHandle::request_exit`'s own `PostThreadMessageW` (now in `MainThreadHandle`), `start_hold_timer`/`sleep_until` (no unsafe).
- Atomics: `KeyboardHook::enabled` keeps `swap(AcqRel)` / `load(Acquire)`, now written only on the main thread. Bench only: `PROBED_AT` (Release store in the probe procedure, Acquire load by the sender after the send returned), the busy threads' stop flag (Release/Acquire). No `Relaxed`, no `SeqCst`, no `static mut`.
- Thread-locals: `MAIN: RefCell<Option<MainState>>` (ruling Q1), test-only `REFUSED: Cell<u32>` and `FAST_ACTIONS_RUN: RefCell<Vec<FastAction>>`.

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass, at HEAD and at every green commit (C3, C6, C8, C10, C12, C15 are the declared red commits; C3, C6 and C15 do not compile their tests by design).
- Tests run (touched modules): `cargo test --locked --lib -- engine:: hook:: core:: logging:: platform:: config::` at HEAD: 524 passed, 0 failed, 3 ignored (manual). Full suite left to the tester.
- Manual ignored tests added: none. The 5 existing `manual:` tests are unchanged. `bench::latency_bench` is ignored with a `bench:` reason (Q5).
- `cargo doc --lib --no-deps`: the 3 warnings left are pre-existing (`ConfigError::Missing` twice, an unclosed `raw` tag); the 3 new link warnings this lane caused were fixed.

## Latency bench (D10)

Command: `KS_BENCH_OUT=<file> KS_BENCH_LABEL=<label> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1` (`KS_BENCH_LOAD=1` adds R8). Files: `D:/tmp/ks-orch/p2-2.2-timers/bench-master.md` (C1 `611c83f`, 3 runs, then 3 reruns from a worktree after HEAD), `bench-head.md` (`2f11917`, 3 runs; `d738632` changes comments only), `bench-alternate.md` (4 interleaved master/HEAD pairs), `bench-head-load.md` (R8, 3 runs). Values in us, median of the three p50s and the largest p99.

| Row | master p50 | master p99 max | HEAD p50 | HEAD p99 max | D10 rule | Verdict |
|---|---|---|---|---|---|---|
| R1 on_key typing | 0.100 | 0.400 | 0.100 | 0.700 | p50 <= 1.25 x 0.100 + 0.1 = 0.225 | pass |
| R2 on_key gestures | 0.200 | 0.500 | 0.200 | 0.500 | p50 <= 0.350 | pass |
| R3 on_timer | 0.100 | 0.100 | 0.100 | 0.300 | p50 <= 0.225 | pass |
| R4 callback | 5.300 | 58.1 | 7.000 | 61.5 | p50 <= 1.25 x 5.3 + 0.5 = 7.125; p99 <= 1.5 x 58.1 + 2 = 89.2 | pass (see note) |
| R5 Hold key-down callback | 37.8 | 403.5 | 5.9 | 115.5 | p50 <= master p50 | pass, 6.4x lower |
| R6/5 lateness | 500.3 | 894.4 | 499.5 | 2197.2 | p50 <= 1000 and <= master + 500 | pass |
| R6/20 | 615.9 | 3145.7 | 508.1 | 1121.7 | same | pass |
| R6/50 | 582.8 | 1660.7 | 585.2 | 1047.3 | same | pass |
| R6/200 | 593.4 | 1615.9 | 378.0 | 803.9 | same | pass |
| R7 round trip | 10.2 | 171.8 | 15.8 | 242.5 | p50 <= 10.2 + 20 = 30.2; p99 <= 1.5 x 171.8 + 50 = 307.7 | pass |

Note on R4: the three HEAD runs right after the release build gave p50 7.0/7.3/6.8 us, close to the
bound. The master reruns from a worktree gave 5.3/5.6/5.4, and 4 interleaved pairs
(`bench-alternate.md`) gave master 5.5/5.3/5.5/5.7 against HEAD 5.3/5.5/5.5/5.5: no per-event
regression on the adapter path; the first HEAD window was machine noise. An experiment (not
committed) ruled out the bench thread's message queue and the test-only fast-action record as
causes. R5 is lower by 6.4x, not the 10x the cut expected; the rule is p50 <= master. R7 is a few us
higher in the median (interleaved: master 13.9, HEAD 16.6), within its rule: the new loop does a drain
and a plan per wake.

R8 (ruling M5; one-way delivery at the receiver, loaded machine, us):

| Run | NORMAL p50 | NORMAL p99 | HIGHEST p50 | HIGHEST p99 | HIGHEST worse beyond margin? (p99_H > 1.25 x p99_N + 20) |
|---|---|---|---|---|---|
| 1 | 16.4 | 252.1 | 18.8 | 234.1 | no (234.1 <= 335.1) |
| 2 | 18.6 | 561.0 | 15.1 | 204.5 | no (204.5 <= 721.3) |
| 3 | 24.0 | 2632.3 | 23.5 | 393.7 | no (393.7 <= 3310.4) |

0 of 3 runs show HIGHEST worse, so D9 lands (C15, C16). Round trips: NORMAL p99 266.6/918.6/2812.3,
HIGHEST 255.6/214.4/443.0.

## cfgcheck (methodology.md#validation-harness)

Harness `D:/tmp/ks-orch/p2-2.2-timers/cfgcheck` (verbatim from the methodology page), target
`cfgcheck-target`; inputs in `cfg/`; full output, stderr included (`2>&1`), in `cfgcheck-output.txt`.
Every file loads: `f9-tap-double.ron` (triggers-conditions.md §11.6 example) OK 2 bindings;
`f13-tap-hold.ron` (§11.6, unchanged) OK 2; `note-11-8.ron` (the three §11.8 examples) OK 3;
`row-75.ron` (known-issues row 75) OK 2; `template.ron` (`default_config.ron` with the new comments)
OK 3 (with `[INFO] SwitchLanguage: using default combination`); manual-check configs `manual-m1.ron`
OK 4, `manual-m3.ron` OK 1, `manual-m4.ron` OK 1, `manual-m6.ron` OK 2, `manual-m8.ron` OK 1.

## Docs

- README: tree (`bench.rs`, `schedule.rs`, `main_thread.rs`, `priority.rs`, `timers.rs` text, `keyboard_hook.rs` text), `keyboard_hook.rs`/`keyboard_hook_callback.rs` bullets, new `main_thread.rs` section, `app.rs` bullets, executor bullet, triggers (one gesture, Hold timer, DoubleTap group, Tap waits, cancel rule, repeats), log counter text, Technologies, Features.
- `src/config/default_config.ron`: trigger comments (one press, gap, Tap waits), the Tap/Hold example comment, the LShift DoubleTap comment (cancel). ASCII only in new text.
- CHANGELOG `[Unreleased]`: Added (bench); Changed (decision-core line edited, main-thread timers, `swch on/off` on the main thread and the new replies and messages, test-only fast actions, two Breaking lines: cancel rule and one gesture per press, priority, chord/sequence expiry); Fixed (KS-25 for every term pair, known issue 74, KS-33 closed).
- Book config pages: `triggers-conditions.md` §11.4, §11.5, §11.6, §11.7, §11.8; `known-issues.md` rows 23, 24, 67, 74, new 75; `actions.md` §13.1; `errors.md` §18.6 (5 new rows, counter text); `kinds.md` sequence item 7, chord item 5, `:177`, `:198`; `structure.md:63`.
- `mdbook build book -d D:/tmp/ks-orch/p2-2.2-timers/book`: built; internal links and anchors over the HTML (`phase-0/htmllinks.py`): 20 pages, 2324 links, 0 broken, 0 duplicate ids (`linkcheck.txt`). mdbook-mermaid warns it was built for mdbook 0.5.3 (machine setup, pre-existing).

## Book pages left to the Phase-2 book sync (file:line on `c9c1373`; none of them was touched)

- `architecture.md:121`, `:126`, `:148-150`, `:163-166`, `:181`, `:187`, `:191`, `:193-200`, `:215`, `:225`, `:249`, `:271`, `:306-307`, `:357-389`, `:397`, `:405-409`, `:426`, `:563`, `:588` (engine lock, `hold-timer` threads, `GetMessageW` loop, `set_enabled` on the IPC worker; now `hook::main_thread`).
- `overview.md:68`, `:89`, `:223`.
- `glossary.md`: Counters `:121-125` (`hold_timer_not_started` meaning), Decision `:143-145`, Enabled flag `:169-171`, Engine `:173-175`, Generation token `:201-203` (now generation + slot + seq), Gesture group `:209-213` (one press, DoubleTap window identity), Hold `:223-225`, Hook thread `:231-233`, KeyboardHook singleton `:275-277`, Message loop `:331-333`, "Tap, Hold and DoubleTap" `:457-459`, Thread affinity `:465-467`.
- `build-and-run.md:141` (counter text), `:229-230` (new `on`/`off` failure replies: "the daemon's main thread did not answer within 2 s", "the daemon is shutting down", "the daemon is busy; try again"; a timed-out request still takes effect), `:253-265`.
- `roadmap.md:28`, `:122`, `:404` (Phase-1 manual check 9: `<T>` no longer toggles), `:417-428`, `:458-477` (2.2 status; "Files" reuses `platform/timers.rs`; R-7 hold-timer half closed, m-1 pinned for the main loop), `:787` (open question 5 decided), `:792` (open question 10 implemented), `:805` (follow-up "DoubleTap entries per binding or group" closed), and the issue-coverage rows for KS-11, KS-25, KS-33.
- `audit.md:364` (KS-11 status line), `:574` (KS-25 -> Fixed on `feat/main-thread-timers`), `:656` (KS-33 -> Fixed).
- `gaps.md:30`, `:68-91` (Tap versus DoubleTap: decided, Tap waits).
- Code doc outside the cut's touch set: `src/core/executor/mod.rs:6` ("or a Hold's `hold-timer` thread").
- Orchestrator: CLAUDE.md principle 5 (the `Mutex<Engine>` exception ends; `MAIN` thread-local, Q1), reviewer R3/R4 and the agent files.

## Manual checks (human only; none has an ignored test)

Release daemon started with `swch open` from a normal console; Notepad focused; `osk.exe` open to see
held modifiers. Configs validated above (`cfg/manual-*.ron`).

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | CapsLock binding replaced by the README `Tap(200)` + `Hold(200)` pair (`manual-m1.ron`) | 20 quick CapsLock taps; then hold 1 s | 20 switches; one LShift press after 200 ms with no input stall; nothing stuck |
| M2 | The template's `LShift` `DoubleTap(250)` uncommented | Type `<T>` with the left Shift for both; then double-tap the left Shift alone | `<T>` typed and CapsLock unchanged (known issue 67 fixed); the lone double tap toggles CapsLock |
| M3 | `( keys: "LCtrl", action: PressKey("A"), trigger: Tap(200) )` | Select text, quick Ctrl+C; then a lone quick Ctrl tap; then a quick Ctrl+click | Ctrl+C copies and types nothing; the lone tap types `a`; Ctrl+click types `a` (declared, M3) |
| M4 | `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )` | Hold Ctrl+Shift and press Right 5 times over 2 s; then hold Ctrl+Shift alone for 1.5 s | Words selected, layout unchanged; the lone hold switches once |
| M5 | `f9-tap-double.ron` | Tap F9; double-tap F9; tap F9 then type `x` at once | `a` about 0.3 s after the press; only `b`; `ax` in that order |
| M6 | `manual-m6.ron` | CapsLock, then Shift+CapsLock quickly; then Shift+CapsLock twice | Nothing typed; then `B` (known issue 74) |
| M7 | Default config, daemon elevated | `swch off`, CapsLock, `swch on`, CapsLock, `swch status`, `swch exit` | Every command answers within 1 s; CapsLock reaches Windows only while off; the window closes without `[FATAL]` |
| M8 | `manual-m8.ron` | Hold F9; press Ctrl+C in the daemon console within 2 s | The daemon exits at once, no `[FATAL]`, nothing typed |
| M9 | Default config | 30 s of typing with frequent Alt+Tab and one Win+L and unlock | No stuck modifier in `osk.exe`; no `[ERROR]` in `daemon.log` (WinEvent delivery through the new loop) |
| M10 | Default config, a `cargo build --release` of any crate running (all cores busy) | Type in Notepad and tap CapsLock; Process Explorer -> key-switch-rs.exe -> Threads | Typing as responsive as when idle; the main thread shows base priority 10 |

## Commits

- `611c83f` Bench: a latency bench for the key path, the gesture timers and the message loop
- `e969dd7` Tests: the gesture repro's stale-timer tests, ported with their assertions inverted
- `68b13a0` Tests: timer tokens, the timer queue and the Scheduler seam
- `cfd7be6` Engine: timer tokens carry a generation, a slot and a sequence number; a Scheduler trait and a timer queue
- `e9fc2f4` Tests: no thread is started on the gesture path
- `7cb6868` Core: under test a fast action is only recorded
- `1aa717f` Tests: the main thread owns the engine, waits on its gesture timer and its messages at once, and answers swch on and off
- `464d3d4` Hook: the engine and its gesture timers live on the main thread
- `b68c7f2` Tests: Tap, Hold and DoubleTap on one key resolve as one press, and a DoubleTap entry belongs to its group
- `506aad3` Engine: one gesture machine per press; Tap waits for the DoubleTap window; DoubleTap entries belong to their group (KS-25, known issue 74)
- `c4609ea` Tests: another key-down cancels a pending gesture whose key passes (known issue 67)
- `c5ef59a` Engine: another key-down cancels a pending gesture whose key passes
- `64b4ead` Tests: chord and sequence entries ask for their own expiry timers
- `3e516f9` Engine: chord and sequence entries expire on their own timers
- `4e9dd62` Bench: sent-message round trips under CPU load at normal and highest priority
- `0a8caec` Tests: the priority guard raises the thread to THREAD_PRIORITY_HIGHEST and restores it
- `2f11917` Core: the main thread pumps at THREAD_PRIORITY_HIGHEST
- `d738632` Hook: the adapter's and the engine's comments name every gesture timer and the cancel rule

## Open points / pre-existing warnings

- For the tester: replay the KS-11 repro at `7cb3521` (C2); the OQ1 mutation (`QS_ALLEVENTS` in `MainThread::run`) on `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer`; the bench on master (`611c83f`) and HEAD in its own worktree, R8 with `KS_BENCH_LOAD=1`.
- Residual R-7 (2.1): the config watcher's `update_bindings -> cancel_actions` can still post a wake to an ended executor at exit; the hold-timer half is gone.
- Follow-ups named by the cut: `config_watcher.rs:126` `.expect("spawn config-watcher thread")`; the executor's own MWMO pin (3.4); a re-entry refusal counter (3.4); a loader warning for unequal Tap/Hold terms (3.4); `KBDLLHOOKSTRUCT.time` as the event clock.
- Pre-existing rustdoc warnings (not this lane): `load`/`reload` link to private `ConfigError::Missing`; unclosed HTML tag `raw`.
- `D:/tmp/ks-orch/p2-2.2-timers/scratch-experiment.md` holds the uncommitted R4 experiments; no worktree of this lane is left.
