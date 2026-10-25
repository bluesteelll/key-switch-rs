# Implementation: p2-2.1-executor round 1

Branch `feat/action-executor`, 26 commits on `master` @ `7d757a1`, HEAD `6a1db71`, tree clean. Design: `cut.md` (final) with `rulings.md` rounds 1 and 2; how each remark and ruling was resolved is in `cut.md` "Developer addendum". 28 files changed, +3395 / -303.

## Items

| Item | Commit(s) | Where (HEAD) | Red-first |
|---|---|---|---|
| L1 1.1 review M2: reflow KS-10 entry | `28aa6e7` | `CHANGELOG.md` KS-10 bullet | doc only |
| L2 1.3 results P1/P2: block_default Breaking entry | `3a82f6e` | `CHANGELOG.md` Breaking `block_default` sub-bullets | doc only; both RON forms through cfgcheck (below) |
| L3/L4 1.2 results m1: MaskOnly fallback warning only where the blocker is added | `fbd6b59` red, `b6e5a9f` fix | `src/config/loader.rs` `expand_with_auto_blockers`; CHANGELOG, `errors.md`, `matching.md` | `mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing`: red `left: ["binding #0: block_original_combo: MaskOnly needs ... blocked as with true"] right: []` -> green |
| L5/L6 1.6 review m3: empty-config warning returned, not logged | `47f2b88` red, `2ab67df` fix | `src/config/loader.rs` `parse_ron_str` | `an_empty_config_warning_is_returned_not_logged`: red `left: [] right: ["config contains no bindings"]` -> green |
| L7/L8/L9 1.7 review m1/m2: start_logging seam, install before the writer, attach_file doc | `89fcf5e` seam, `7827ec8` red, `62bde43` fix | `src/logging/mod.rs` `init`, `start_logging`, `attach_file` doc | `a_second_init_starts_no_writer`: red `panicked ... a second init must not start a writer` -> green |
| L10 1.5 results m1: `run` doc | `ee6c197` | `src/main.rs` `run` doc | doc only (no harness, KS-38) |
| L11 SAFETY on `console_ctrl_handler`'s `PostThreadMessageW` | `1ea349d` | `src/core/app.rs` `console_ctrl_handler` | comment only |
| X1/X2 executor (R1, R3, V2-M3) | `cf3c9f6` red, `dc2529c` impl | `src/core/executor/{mod.rs,runs.rs,testing.rs}`, `Counter::ExecutorJobsDropped`/`ExecutorUnavailable` | compile red (E0433 `TargetWindow` x34, `Stopped` x17, E0425 `QUEUE_CAPACITY` x15, E0432 `super::runs::{Ctx, Job, Next, Runs}`, E0405 `Runner`; `cargo build` green) -> 28 pass, 1 ignored |
| V2-I1 drain before every wait | `a364b77` red, `5ea07d8` fix | `run_loop` `src/core/executor/mod.rs:590-685` | `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake`: red `left: None right: Some("open B")` (3 runs, deterministic) -> green |
| comment cleanup | `d269caf` | executor comments cite no lane ruling (sits on X3's red tree) | n/a |
| X3/X4 R2 `ShowWindowAsync` | `fd5993e` red, `8e28876` fix | `apply_window_op` `src/data/sequence.rs:246-279`; docs | `a_window_step_never_waits_for_its_window`: red lists `sequence.rs:277/280/283 ShowWindow(target, SW_*)` -> green |
| X5 `.execute()` scan | `fcce19b` red | `keyboard_hook_callback.rs` tests | `the_hook_path_names_no_execute_call`: red `src/hook/keyboard_hook_callback.rs:219: action.execute();` -> green at X8 |
| X6 no sleep / blocking wait | `ba3122f` red | `core::executor` tests | `macros_never_sleep_or_block_in_a_wait`: red `src/data/sequence.rs:133: SequenceStep::Delay(ms) => thread::sleep(...)` -> green at X8 |
| X7 split, Hold, cancel tests | `6edfa0f` red | windows_actions, keyboard_hook_callback, keyboard_hook tests | compile red (E0433 `FastAction` x7, E0599 `attach_executor` x6, E0061 `carry_out` x4, E0599 `submit` x3) -> green at X8 |
| X8 wiring | `a4a2e32` | `FastAction`/`as_fast` (`windows_actions.rs`), `KeyboardHook::{attach_executor, submit, cancel_actions}`, `carry_out(hook, ..)`/`dispatch`, Hold via `get_hook()`, `spawn_sequence` removed, `AttachedExecutor` in `App::run`; docs | makes X5, X6, X7 green |
| X9/X10 longest callback (D10) | `0ff3ce2` red, `4db3bb2` fix | `Counters::{record_callback, longest_callback}`, `Snapshot`, `SLOW_CALLBACK`, recording in `keyboard_hook_callback` | compile red (E0599 `longest_callback` x3, `record_callback` x3, E0609, E0422 `Snapshot`, E0425 `SLOW_CALLBACK`) -> green |
| X11/X12 hook path logs nothing (D11, M1) | `37b1847` red, `6a1db71` fix | `fn_lines` + scan in `logging/mod.rs` tests; `Counter::PostMessageFailed`, `bump_if_err`; three sites | `the_functions_on_the_hook_path_log_nothing`: red lists `windows_actions.rs:278`, `system_function.rs:58`, `:77` -> green |

### Tests added (all in `#[cfg(test)] mod tests` of the file under test)

- `core::executor::runs::tests` (pure, fake clock): `a_run_goes_up_to_its_first_delay_and_resumes_at_its_deadline`, `two_macros_fired_together_interleave_at_their_delays_and_keep_their_order`, `a_cancelled_waiting_run_never_resumes`, `a_job_of_an_old_epoch_never_starts`, `the_delay_starts_when_its_step_is_reached`, `a_panicking_step_ends_only_its_macro`, `a_zero_delay_lets_the_other_runs_go_first`, `launch_and_open_url_run_when_their_job_starts`.
- `core::executor::tests` (real executor thread, `FakeRunner`, own counters): `a_hundred_sequence_runs_use_the_one_executor_thread`, `delay_10000_stops_within_100_ms_of_an_epoch_change`, `a_delay_5000_macro_holds_up_no_other_job`, `two_macros_fired_together_both_run_in_their_own_order`, `a_cancelled_waiting_run_never_resumes`, `a_delay_resumes_its_run_after_its_deadline`, `a_delay_of_u64_max_waits_until_cancelled`, `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms`, `timer_ms_rounds_up_and_clamps`, `the_delay_timer_is_kept_only_while_nothing_ran` (V2-M3), `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` (V2-I1), `twenty_slow_jobs_each_enqueue_in_under_5_ms`, `a_full_queue_drops_the_job_and_counts_it`, `a_job_queued_before_an_epoch_change_does_not_run`, `jobs_run_in_the_order_they_were_queued_with_their_target`, `stop_interrupts_a_delay_drops_the_queue_and_joins`, `a_submit_after_stop_is_closed_and_not_counted`, `an_executor_ended_by_wm_quit_counts_every_later_submit`, `a_panicking_job_leaves_the_executor_running`, `the_executor_runs_jobs_in_a_single_threaded_apartment`, `the_apartment_guard_uninitialises_com_on_drop`, `macros_never_sleep_or_block_in_a_wait`, `the_win32_runner_opens_a_url` (ignored, manual M9).
- `core::windows_actions::tests::only_launch_open_url_and_sequence_leave_the_firing_thread`.
- `data::sequence::tests::a_window_step_never_waits_for_its_window`.
- `hook::keyboard_hook_callback::tests`: `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`, `a_hold_runs_its_slow_action_on_the_executor`, `the_hook_path_names_no_execute_call`, `the_keyboard_callback_records_how_long_it_ran`.
- `hook::keyboard_hook::tests`: `swch_off_stops_a_running_macro_within_100_ms`, `a_reload_stops_a_running_macro_within_100_ms`, `a_repeated_swch_on_keeps_the_running_macro`, `a_slow_action_without_an_executor_is_closed`.
- `logging::counters::tests`: `the_longest_callback_is_kept_until_taken`, `a_slow_callback_alone_is_reported_and_a_fast_one_is_not`, `bump_if_err_counts_only_an_error`; `logging::writer::tests::a_flush_reports_a_slow_callback`; `logging::tests`: `a_second_init_starts_no_writer`, `the_functions_on_the_hook_path_log_nothing`.
- `config::loader::tests`: `mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing`, `an_empty_config_warning_is_returned_not_logged`.

Existing tests changed only to follow an API or format change: `carry_out_sends_the_menu_mask_only_when_the_decision_asks` (passes its hook), counters/writer report strings and arrays (new counters, `Snapshot::counts`, the trailing `longest_callback_us`). None was weakened, deleted or ignored.

## Remarks

Each Critical/Important remark and each ruling is recorded in `cut.md` "Developer addendum". Summary:
- critique I1 (R2) - RESOLVED at `src/data/sequence.rs:246-279`, pinned by `a_window_step_never_waits_for_its_window`; manual M11.
- critique_v2 I1 (V2-I1) - RESOLVED at `src/core/executor/mod.rs:590-652`, pinned by `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake`.
- critique M1 - RESOLVED, `the_functions_on_the_hook_path_log_nothing` (one commented list). M2 - RESOLVED (L7-L9, X5/X6 alone, red by assertion). M3 / Q2 - RESOLVED (config pages done; non-config pages below). M4 / R3 - RESOLVED, `an_executor_ended_by_wm_quit_counts_every_later_submit`. M5 - RESOLVED, `a_hold_runs_its_slow_action_on_the_executor` (nothing installed or injected).
- critique_v2 M1 (V2-M1) - RESOLVED, `actions.md` §14.2 last bullet. M2 (V2-M2) - recorded as manual M11 with its master control. M3 (V2-M3) - RESOLVED, `plan()` at `src/core/executor/mod.rs:436-446`, pinned by `the_delay_timer_is_kept_only_while_nothing_ran`.
- critic open questions (round 2): OQ1 ownership stated in the addendum (no natural seam for a loop-panic test: review item); OQ2 manual M13; OQ3 manual M12.

## Deviations from the cut

Listed in the addendum: `SetCoalescableTimer` with the pure `plan(.., code_ran)` (a dispatched foreign message counts as code that ran); `Calls`/`Polled.calls`; `fired` returns `()`; `Apartment` is `!Send`; the V2-I1 pair and `d269caf` are extra commits; extra tests and shared test helpers; the always-present `longest_callback_us` field; doc placement details; drifted red line numbers. Declared deviations 1 (job carries the action by value) and 2 (bounded 2 s join, then detach with `[WARN]`) are as the cut states.

## Unsafe / atomics added

All in `src/core/executor/mod.rs` unless named:
- `TargetWindow::foreground`, `GetForegroundWindow` - SAFETY: "takes no arguments and reads no memory of ours; a null window becomes NONE".
- `ExecutorHandle::wake`, `PostThreadMessageW(WAKE)` - SAFETY: "takes the thread id and the message by value, reads no memory of ours and returns without waiting; WAKE carries no pointer. An id that is no longer valid only makes the call fail".
- `Apartment::enter_sta` `CoInitializeEx` / `Drop` `CoUninitialize` - SAFETY: no pointer of ours; balanced on the same thread (the guard is `!Send`).
- `DelayTimer::arm` `SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)`, `kill` `KillTimer(None, id)` - SAFETY: no window, no TimerProc, posts only to this thread's queue, which it pumps; id killed on the same thread.
- `run_executor` `PeekMessageW(PM_NOREMOVE)`, `GetCurrentThreadId`; `run_loop` `GetMessageW`, `TranslateMessage`/`DispatchMessageW` - SAFETY: `msg` is a local that outlives the calls.
- `src/data/sequence.rs` `apply_window_op` `PostMessageW(WM_CLOSE)` and `ShowWindowAsync` - SAFETY: non-invalid HWND from `GetForegroundWindow`, by value, posts only, a stale HWND fails without UB.
- `src/core/app.rs` `console_ctrl_handler` (L11, comment added to an existing block).
- Test only: `testing.rs` `PostQuitMessage(0)` and `PeekMessageW(WAKE, PM_REMOVE)` (own thread's queue); `CoGetApartmentType` probe.
- Atomics: `Shared.epoch` (`fetch_add AcqRel` / `load Acquire`), `Shared.stopping` (`store Release` / `load Acquire`), `Shared.active_runs` (`store Release` on the executor / `load Acquire`), `Counters.longest_callback_ns` (`fetch_max`/`swap AcqRel`, `load Acquire`). None publishes other data (a job travels through the channel). No `Relaxed`, no `static mut`. `KeyboardHook.executor: ArcSwapOption<ExecutorHandle>` (stores on main only, one load per slow fire).

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass (HEAD `6a1db71`).
- Tests run by the developer (the tests the diff touches, at each commit and on HEAD): `cargo test --locked --lib -- core:: hook:: data:: logging:: config:: system::` = 314 passed, 0 failed, 5 ignored. `core::executor` and `hook::` ran 3-5 times in a row with the same result. The full `cargo test --locked` (engine scenarios, integration test, doc tests) is left to the tester.
- `cargo doc --no-deps --locked`: the 4 warnings that `master` has (loader x2, `set_enabled`, `vk_name.rs:4`); none new.
- Manual ignores added: `core::executor::tests::the_win32_runner_opens_a_url` (M9).

## cfgcheck outputs (`D:/tmp/ks-orch/p2-2.1-executor/cfgcheck`, the harness of methodology.md verbatim, built against the work tree; `2>&1`)

L2, `( bindings: [ ( keys: "LCtrl", action: PressKey("Escape"), trigger: Hold(300) ) ] )`:
```text
[INFO] loading config from l2_hold.ron
OK: 1 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(162))] }) action=PressKey(VIRTUAL_KEY(27)) block_default=false block_original_combo=None condition=Always trigger=Hold(300)
exit=0
```
L2, the same with `block_default: true`:
```text
[INFO] loading config from l2_hold_block.ron
OK: 1 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(162))] }) action=PressKey(VIRTUAL_KEY(27)) block_default=true block_original_combo=None condition=Always trigger=Hold(300)
exit=0
```
L6, `( bindings: [] )` (output unchanged, one warning):
```text
[INFO] loading config from l6_empty.ron
[WARN] config contains no bindings
OK: 0 bindings (after auto-blocker expansion)
```
`src/config/default_config.ron` after X4 and after X8 (comments only; same as the published output):
```text
[INFO] loading config from src/config/default_config.ron
[INFO] SwitchLanguage: using default combination
OK: 3 bindings (after auto-blocker expansion)
  #0: ... action=SwitchLanguage ... block_original_combo=Some(Swallow) ...
  #1: ... action=ToggleCapsLock ...
  #2: kind=Combo(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] }) action=DoNothing ...
exit=0
```
`book/src/config/example.md` full config (X8, comments only): `OK: 26 bindings (after auto-blocker expansion)`, 26 binding lines, exit 0, the count the page states. `book/src/config/actions.md` §13.7 OpenUrl block without the `Sequence` wrapper (X8), wrapped in `( bindings: [ ... ] )`: `OK: 4 bindings`, the four `OpenUrl(..)` with `trigger=Immediate`, exit 0. Raw outputs: `D:/tmp/ks-orch/p2-2.1-executor/cfg/*.txt`.

## Docs

- Book (config pages, this lane): `actions.md` (§13, §13.1 thread table and executor paragraph, KS-08 box to a fixed note, process rights, §13.2, §13.4, §13.6, §13.7 examples and note, §14, §14.1, §14.2 incl. `:309` focus (V2-M1), §14.3, §14.7), `errors.md` §18.6 (MaskOnly row, window-step row, eight executor rows, dropped post errors, counters line), `matching.md` §15.5, `README.md`, `example.md` (comments, known-defect list), `kinds.md`, `triggers-conditions.md` (§11.3, §11.5, §11.8), `known-issues.md` rows 26 (partly fixed), 28, 29, 36, 60 (fixed). `mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/book`: built; internal links and anchors checked over the built HTML: 23 pages, 7877 links, 0 broken (`D:/tmp/ks-orch/p2-2.1-executor/linkcheck.txt`). mdbook-mermaid warns it was built for mdbook 0.5.3 (machine setup, pre-existing).
- README: tree, `windows_actions.rs`/`keyboard_hook.rs`/`keyboard_hook_callback.rs`/`app.rs` bullets, new `executor/` section, Hold, macros, `Delay` and `Window` rows, parallel sequences, `swch off`, counters paragraph, `Win32_System_Com`, thread safety.
- `default_config.ron`: `Sequence`, `Window`, `Delay` comments (added text ASCII; the lines' existing em dash kept).
- CHANGELOG `[Unreleased]`: Added (executor counters, `post_message_failed`, `longest_callback_us`; the 1.7 sample line), Changed (Window steps `ShowWindowAsync`; post errors counted), Fixed (KS-08; macro cancellation; KS-01 and KS-33 edited), plus L1, L2, L4 edits.

### Non-config book pages left to the Phase-2 book sync (line numbers at HEAD)

- `architecture.md:188` (thread table: "Sequence worker (unnamed)" -> `action-executor`), `:211` (a running macro keeps running after `swch off`), `:213-239` (§ "Why actions can stall all keyboard input (KS-08)"), `:434-436` and `:445` (action table, `spawn_sequence`), `:577` (actions still on the hook thread), `:588` (the remaining structural step).
- `glossary.md:107` (COM initialisation), `:317` (macro on a new worker thread).
- `overview.md:20-21` (actions, macros on a worker thread), `:68` (diagram node "hold-timer and macro threads").
- `build-and-run.md:118-141` (console line table: the executor lines, the dropped post errors, the counters line at `:141`), `:251` (`on`/`off` stop running macros), `:270` (`status`), `:461` (troubleshooting row: wrapping `Launch`/`OpenUrl`).
- `gaps.md:35`, `:145-147` ("No cancellation for macro workers": fixed), `:166` (Sequence thread panic: gone).
- `roadmap.md:421`, `:449-455` (2.1 status after Phase 2), `:727` (KS-08 row).
- `audit.md:312` (KS-08 status line: "Partly fixed in <sha> (feat/action-executor); watchdog: item 2.4"), KS-33 status (`:653-`, the Sequence half fixed).
- `methodology.md:442-451` (the published cfgcheck output is unchanged; nothing to do unless re-run).

## Manual checks (human only; none was run here)

Release daemon started with `swch open`; `config.ron` edited in place; Notepad focused unless stated. M1-M10 as in `cut.md` "Manual checks" (M1 cold browser x20, M2 `swch off` mid-Delay, M3 reload mid-Delay, M4 `swch exit` mid-Delay, M5 side-by-side macros and OpenUrl during a Delay, M6 Hold OpenUrl/PressKey, M7 fast actions in order, M8 OpenUrl targets, M9 `cargo test --locked -- --ignored the_win32_runner_opens_a_url`, M10 Ctrl+C mid-Delay). Added by the round-2 rulings:
- **M11 (R2, V2-M2)**: as in `cut.md` (a `Window(Minimize)` macro against a Not Responding window, then `OpenUrl` within 2 s: the URL opens within about 1 s). Run it once on `master` as the control: if the control does not hang either (DWM's ghost window), record M11 as "does not reproduce the hazard"; R2 stands on Microsoft's documentation.
- **M12 (critic OQ3)**: `( keys: "F19", action: Sequence([ Window(Minimize), Delay(2000), Window(Restore) ]) )` against the foreground Notepad: record whether the window comes back to the front after 2 s. If it does not, the X4 Changed entry in CHANGELOG needs a sentence saying so (open point below).
- **M13 (V2-M3 timing line)**: `( keys: "F20", action: Sequence([ Key("F13"), Delay(50), ... 20 times ]) )` (20 x `[Key("F13"), Delay(50)]`; bind F13 to nothing or watch it in a key viewer): the macro finishes in about 1.0-1.4 s.
- KS-38 reminder: the daemon binary has no harness, so the start/attach/stop order in `App::run` (executor before the hooks, stop after `uninstall`, `[FATAL] could not start the action executor`) is checked by review and M4.

## Commits

```text
28aa6e7 Changelog: reflow the KS-10 entry
3a82f6e Changelog: the block_default entry names the modifier held under Hold and DoubleTap output
fbd6b59 Tests: a MaskOnly fallback for a hotkey a binding takes over warns nothing
b6e5a9f Config: warn about the MaskOnly fallback only where its blocker is added
47f2b88 Tests: parse_ron_str returns the empty-config warning instead of logging it
2ab67df Config: the empty-config warning goes into the parsed warnings
89fcf5e Logging: init goes through a start_logging seam
7827ec8 Tests: a second logging init starts no writer
62bde43 Logging: install the logger before the writer starts; the attach_file doc names the full-queue failure
ee6c197 Docs: the run doc comment gives the real reason the guard lives until run returns
1ea349d Core: SAFETY comment on the PostThreadMessageW in console_ctrl_handler
cf3c9f6 Tests: the action executor runs macros side by side on its one thread, drops jobs when full and stops on an epoch change
dc2529c Core: the action executor thread with resumable macro runs
a364b77 Tests: a job queued while a step pumps messages runs without another wake
5ea07d8 Core: drain the executor's queue before every wait
fd5993e Tests: a Window step never waits for its window
d269caf Core: the executor's comments name no lane-internal ruling
8e28876 Macros: Window steps use ShowWindowAsync and do not wait for the window
fcce19b Tests: the hook path names no execute call
ba3122f Tests: no macro code sleeps or blocks in a wait
6edfa0f Tests: fast actions run where they fire, the others go to the executor, and swch off or a reload stops a macro
a4a2e32 Hook: run fast actions in place and queue Launch, OpenUrl and Sequence on the action executor
0ff3ce2 Tests: the longest keyboard callback is recorded and a slow one is reported
4db3bb2 Logging: record the longest keyboard callback and report it from 5 ms
37b1847 Tests: the functions on the hook path log nothing
6a1db71 Actions: count a failed post on the hook path instead of logging it
```
Declared red commits: `fbd6b59`, `47f2b88`, `7827ec8`, `cf3c9f6`, `a364b77`, `fd5993e` and `d269caf` (X3's test), `fcce19b`, `ba3122f`, `6edfa0f` (X5-X7, three reds in a row made green by `a4a2e32`), `0ff3ce2`, `37b1847`. Every other commit passes fmt and clippy and the tests it touches.

## Open points / pre-existing warnings

- M12's result decides whether the X4 Changed entry needs a sentence about `Window(Restore)` not bringing a minimised window to the front.
- The 5 ms timing tests (`twenty_slow_jobs_each_enqueue_in_under_5_ms`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`) follow the ruling: every one of 20, one whole-batch retry. A flake goes to the orchestrator; no further loosening was made.
- Residual race R-7 (a wake posted to a reused thread id from a `hold-timer` thread at exit) is accepted until 2.2, as ruled.
- Pre-existing: 4 `cargo doc` warnings (`config/loader.rs:156`, `:176`, `hook/keyboard_hook.rs` `set_enabled` doc link, `data/vk_name.rs:4`); the mdbook-mermaid version warning. No other warnings.
- Apparent dead code noted, not removed: `SystemFunction::{LockWorkstation, ShowDesktop, TaskManager}` have no `BindAction` and keep their `log::` calls (unreachable from the hook path; new feature 1 must give them a placement in `as_fast`).
