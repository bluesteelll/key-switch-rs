# Context pack, round 1: p2-2.1-executor (roadmap 2.1, KS-08)

For the tester and the code-reviewer. Facts with `file:line` (HEAD `6a1db71` unless a commit is named). No opinions.
Design: `cut.md` (final; "Developer addendum" at `cut.md:598-639`), rulings `rulings.md` (rounds 1 and 2), developer
report `impl.md`. Inferred items are marked UNVERIFIED.

## Scope and delta

- Branch `feat/action-executor`, HEAD `6a1db71`, tree clean. Merge base = `master` = `7d757a1`. 26 commits, no merges.
  Every commit message ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (checked on all 26).
- `git diff --stat master..feat/action-executor`: 28 files, +3395 / -303.

| Commit | Item (cut) | Files |
|---|---|---|
| `28aa6e7` | L1 CHANGELOG KS-10 reflow | CHANGELOG.md |
| `3a82f6e` | L2 block_default Breaking sub-bullets | CHANGELOG.md |
| `fbd6b59` | L3 **red** | src/config/loader.rs |
| `b6e5a9f` | L4 fix | loader.rs, CHANGELOG, errors.md, matching.md |
| `47f2b88` | L5 **red** | loader.rs |
| `2ab67df` | L6 fix | loader.rs |
| `89fcf5e` | L7 seam (old order) | src/logging/mod.rs |
| `7827ec8` | L8 **red** | src/logging/mod.rs |
| `62bde43` | L9 fix + attach_file doc | src/logging/mod.rs |
| `ee6c197` | L10 `run` doc | src/main.rs |
| `1ea349d` | L11 SAFETY comment | src/core/app.rs |
| `cf3c9f6` | X1 **red** (compile) | src/core/executor/{mod,runs,testing}.rs, src/core/mod.rs |
| `dc2529c` | X2 executor impl | executor/{mod,runs}.rs, data/sequence.rs, logging/{counters,writer}.rs |
| `a364b77` | V2-I1 **red** | executor/{mod,testing}.rs |
| `5ea07d8` | V2-I1 fix (drain before wait) | executor/mod.rs |
| `fd5993e` | X3 **red** | data/sequence.rs |
| `d269caf` | comment cleanup (sits on X3 red tree) | executor/mod.rs |
| `8e28876` | X4 ShowWindowAsync + docs | sequence.rs, CHANGELOG, README, actions.md, errors.md, default_config.ron |
| `fcce19b` | X5 **red** | hook/keyboard_hook_callback.rs |
| `ba3122f` | X6 **red** | executor/mod.rs |
| `6edfa0f` | X7 **red** (compile) | executor/{mod,testing}.rs, windows_actions.rs, keyboard_hook.rs, keyboard_hook_callback.rs |
| `a4a2e32` | X8 wiring (greens X5, X6, X7) + docs | 19 files (code: app.rs, executor/mod.rs, windows_actions.rs, binding.rs, sequence.rs, engine/mod.rs (doc), keyboard_hook.rs, keyboard_hook_callback.rs, logging/mod.rs; docs: CHANGELOG, README, 7 config pages, default_config.ron) |
| `0ff3ce2` | X9 **red** (compile) | keyboard_hook_callback.rs, logging/{counters,writer}.rs |
| `4db3bb2` | X10 gauge | CHANGELOG, README, errors.md, executor/mod.rs (test helper `take_counts` -> `.counts`), keyboard_hook_callback.rs, counters.rs, writer.rs |
| `37b1847` | X11 **red** | src/logging/mod.rs |
| `6a1db71` | X12 post failures counted | CHANGELOG, README, actions.md, errors.md, windows_actions.rs, counters.rs, logging/mod.rs, writer.rs, system_function.rs |

Untouched (cut touch set `cut.md:333`): Cargo.toml, Cargo.lock, build.rs, src/engine/* code, src/ipc.rs, src/bin/swch.rs,
src/logging/testing.rs, src/hook/config_watcher.rs, src/hook/ipc_server.rs.

## File map

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/core/executor/mod.rs:1-27` | module doc | thread, queue, epoch, "a step already running finishes first" |
| `src/core/executor/mod.rs:56-65` | `QUEUE_CAPACITY` 32, `START_TIMEOUT` 5 s, `STOP_TIMEOUT` 2 s, `WAKE = WM_APP` | ruling Q5 values |
| `src/core/executor/mod.rs:67-105` | `TargetWindow(isize)`, `NONE`, `foreground`, `for_action`, `hwnd`, `from_raw` (test) | hook-path Win32 call only for `Sequence` |
| `src/core/executor/mod.rs:107-149` | `Runner` trait, `Win32Runner` (exhaustive match) | injectable runner (spec); real actions |
| `src/core/executor/mod.rs:151-173` | `Submitted {Queued, Dropped, Unavailable, Closed}`, `Stopped {Joined, TimedOut}` | R3 classification |
| `src/core/executor/mod.rs:175-197` | `Shared {epoch, stopping, active_runs}`, `live()` | orderings, epoch |
| `src/core/executor/mod.rs:199-260` | `ExecutorHandle::{submit, cancel_pending, active_runs, wake}` | hook-path entry (R1, R3, R4) |
| `src/core/executor/mod.rs:262-355` | `Executor::{start, handle, thread, is_finished, stop, stop_thread}`, `Drop` | lifecycle, bounded join (deviation 2) |
| `src/core/executor/mod.rs:357-397` | `Apartment` (STA guard, `!Send`) | COM balance |
| `src/core/executor/mod.rs:399-515` | `DelayTimer`, `Arming`, `Plan`, `plan()`, `timer_ms()` | V2-M3, R1 timer |
| `src/core/executor/mod.rs:517-581` | `End`, `run_executor` (prologue, catch_unwind, epilogue), `report_panics` | critic OQ1 ownership, R3 drain |
| `src/core/executor/mod.rs:583-685` | `run_loop` | V2-I1 drain-before-wait, GetMessageW handling |
| `src/core/executor/mod.rs:687-1364` | tests (list below), `#[ignore]` at `:1351` | acceptance tests |
| `src/core/executor/runs.rs:23-94` | `Job`, `Runs`, `Run`, `Resume`, `Next`, `Ctx`, `Calls`, `Polled` | pure run list |
| `src/core/executor/runs.rs:96-213` | `start`, `poll`, `next`, `drop_timed`, `len`, `Run::advance`, `call` (catch_unwind) | R1 semantics, D9 |
| `src/core/executor/runs.rs:215-524` | pure tests R-a..R-h with fake clock | |
| `src/core/executor/testing.rs:1-176` | `FakeRunner` (`with_block/gate/eat_wakes/panic_on/quit_on`), `Record`, `Ran`, `leaked_counters`, `next_record`, `wait_until`, `WAIT` 2 s | test-only Win32: `:128`, `:142` |
| `src/core/windows_actions.rs:118-183` | `FastAction` (Copy), `FastAction::run`, `BindAction::as_fast` | the split (D1) |
| `src/core/windows_actions.rs:229-283` | `press_vk`, `post_message_to_foreground` (counts failure) | hook path, sentinel |
| `src/core/windows_actions.rs:285-324` | `launch_process`, `shell_open` (now `pub(crate)`, log on executor) | slow actions |
| `src/core/windows_actions.rs:395-449` | test `only_launch_open_url_and_sequence_leave_the_firing_thread` | |
| `src/core/app.rs:24-63` | `AttachedExecutor {attach, stop, Drop}` | D8 order, `[WARN]` on timeout |
| `src/core/app.rs:169-270` | `App::run` doc + body: start `:199-200`, attach `:205`, install `:209`, uninstall `:266`, stop `:267` | R10, KS-20 order |
| `src/core/app.rs:294-312` | `console_ctrl_handler` with new SAFETY comment | L11 |
| `src/data/sequence.rs:1-70` | module/variant docs (`Window` docs `:46`) | X4/X8 docs |
| `src/data/sequence.rs:104-117` | `run_step` (was `execute_step`); `Delay` arm no-op `:110-111` | |
| `src/data/sequence.rs:246-279` | `apply_window_op` with `ShowWindowAsync`; `Close` = `PostMessageW(WM_CLOSE)` | R2 |
| `src/data/sequence.rs:335-359` | test `a_window_step_never_waits_for_its_window` | X3 |
| `src/data/binding.rs:244-247` | `Binding::execute` removed | D1 |
| `src/hook/keyboard_hook.rs:42-74` | `KeyboardHook` fields incl. `executor: ArcSwapOption<ExecutorHandle>` `:73` | R3 shared state |
| `src/hook/keyboard_hook.rs:115-153` | `update_bindings` (+cancel `:126`), `attach_executor`, `submit`, `cancel_actions` | epoch bumps |
| `src/hook/keyboard_hook.rs:167-202` | `set_enabled`: cancel after the lock on `Some(true)` `:198-200` | KS-01, R3 |
| `src/hook/keyboard_hook.rs:531-621` | tests: swch off / reload / repeated on / no executor | acceptance 100 ms |
| `src/hook/keyboard_hook_callback.rs:1-82` | module doc steps 6 and threads | |
| `src/hook/keyboard_hook_callback.rs:206-242` | `carry_out(hook, decision)`, `dispatch` | R1 hot path |
| `src/hook/keyboard_hook_callback.rs:302-338` | `start_hold_timer` -> `carry_out(get_hook(), ..)` `:318`; `sleep_until` | Hold split (M5), R-7 |
| `src/hook/keyboard_hook_callback.rs:340-384` | `keyboard_hook_callback` with gauge `:363`, `:376` | R1, D10 |
| `src/hook/keyboard_hook_callback.rs:690-916` | new tests (gauge, split, 5 ms, Hold, `.execute()` scan) | |
| `src/logging/counters.rs:1-200` | `Counter` (7 variants, `Display`), `SLOW_CALLBACK`, `Counters` + gauge, `Snapshot`, `summary`, free fns | R6, R4 |
| `src/logging/writer.rs:224-230` | `report_counters` on `Snapshot` | D10 |
| `src/logging/mod.rs:11-15` | doc: hook path never logs | D11 |
| `src/logging/mod.rs:105-157` | `init`, `start_logging` (install first), `attach_file` doc | L7-L9 |
| `src/logging/mod.rs:226-245`, `:280-313`, `:384-461` | `a_second_init_starts_no_writer`, `fn_lines`, `names_the_log_crate`, `the_functions_on_the_hook_path_log_nothing` | L8, X11 |
| `src/system/system_function.rs:43-81` | `switch_language_forward/backward` count failed posts | D11 |
| `src/config/loader.rs:249-296` | `parse_ron_str` pushes the empty-config warning `:287-289` | L6 |
| `src/config/loader.rs:322-416` | `expand_with_auto_blockers`; `fallback_of` `:356-362`; `hotkeys_blocked` `:367`, `:382`; warning loop `:401-411` | L4 |
| `src/main.rs:64-68` | `run` doc | L10 |
| `src/engine/mod.rs:13-17`, `:28-34`, `:83-92` | doc only | stale-doc fix |
| docs | CHANGELOG `:12-132` Added, `:132-376` Changed (X4 `:360-370`, X12 `:371-376`), Fixed KS-08 `:481-500`, cancellation `:501-`; README hunks at `:38,:93,:120,:125,:131,:141,:145-154,:434-436,:509-514,:536,:540,:550-556,:729,:795-799,:801,:839,:864`; config pages hunks below | ruling Q2 |

Config-page hunks (new-side line starts): `actions.md` 8, 20, 38, 40, 49, 56, 58, 70, 84, 146, 202, 211, 225, 274, 282,
284, 292, 296, 304, 306, 312, 318, 321, 323, 381; `errors.md` 185, 190, 195-203, 207; `example.md` 99, 188, 207, 221, 236,
239; `kinds.md` 77, 142; `known-issues.md` 64, 72-73, 80, 128; `matching.md` 150; `triggers-conditions.md` 41, 75, 77,
79, 122-123; `config/README.md` 55; `default_config.ron` 104-106, 148-153.

## Facts

### Hook path (callback, dispatch, submit)
1. `keyboard_hook_callback` keeps `code < 0` -> `CallNextHookEx` first (`keyboard_hook_callback.rs:345-350`) and the remapping-off pass before any read (`:352-358`).
2. `started = Instant::now()` is taken after both early passes (`:363`); `counters::record_callback(started.elapsed())` runs after `decide_event` + `carry_out` and before both returns (`:376-383`).
3. Outcome: `LRESULT(1)` iff `suppress` (`:377-379`), else `CallNextHookEx` (`:383`). `decide_event` still checks `constants::is_injected_event` first (`:187`).
4. `carry_out(hook, decision)` destructures `Decision` without `..` (`:213-218`), sends the mask first (`:219-221`), starts timers (`:222-224`), then `dispatch`es each `fire` action by value (`:225-227`).
5. `dispatch`: `as_fast()` `Some` -> `fast.run()` on the calling thread; `None` -> `TargetWindow::for_action(&action)` then `hook.submit(action, target)`, result discarded (`:234-242`).
6. `TargetWindow::for_action` calls `GetForegroundWindow` only for `BindAction::Sequence` (`executor/mod.rs:89-95`, `:78-83`); `Launch`/`OpenUrl` get `NONE`.
7. `KeyboardHook::submit` does one `self.executor.load()` and returns `Closed` when none is attached (`keyboard_hook.rs:140-145`).
8. `ExecutorHandle::submit`: `epoch.load(Acquire)` (`executor/mod.rs:214`), `try_send(Job)`; `Ok` -> `wake()` + `Queued` (`:220-223`); `Full` -> `bump(ExecutorJobsDropped)` + `Dropped`, the rejected `Job` dropped on the calling thread (`:224-227`); `Disconnected` + `stopping` -> `Closed` (`:228`); `Disconnected` otherwise -> `bump(ExecutorUnavailable)` + `Unavailable` (`:229-232`).
9. `wake()` = `PostThreadMessageW(thread_id, WAKE, 0, 0)`, result ignored (`executor/mod.rs:253-259`).
10. The queue is `mpsc::sync_channel(capacity.max(1))` (`executor/mod.rs:281`).
11. `FastAction` derives `Copy` (`windows_actions.rs:123`); `as_fast` has no `_` arm; `Sequence | Launch | OpenUrl` -> `None` (`:164-183`). `Win32Runner::action` has no `_` arm (`executor/mod.rs:126-143`).
12. `FastAction::run` routes `SwitchLanguage*`/`ToggleCapsLock` to `SystemFunction::execute`, `PressKey`/`Media` to `press_vk`, `PostMessage` to `post_message_to_foreground` (`windows_actions.rs:142-156`).
13. `post_message_to_foreground` counts a failed post with `counters::bump_if_err(Counter::PostMessageFailed, ..)` (`windows_actions.rs:280-281`); `switch_language_forward/backward` the same (`system_function.rs:54-60`, `:73-79`).
14. `press_vk` sets `dwExtraInfo: injected_sentinel()` on both inputs (`windows_actions.rs:242`, `:254`). No new `SendInput` site was added by the lane (sites: `windows_actions.rs:262`, `sequence.rs:139`, `:169`, `:205`, `keyboard_hook_callback.rs:272`, plus `system_function.rs` `press_key`).
15. `rg "\.execute\(\)" src/hook` returns no match at HEAD (exit 1). Remaining `.execute()` calls are `SystemFunction::execute` in `windows_actions.rs:144-146`.
16. `BindAction::to_system_function` maps only `SwitchLanguage`, `SwitchLanguageBackward`, `ToggleCapsLock` (`windows_actions.rs:185-192`); `system_function.rs:83-117` (`lock_workstation`, `show_desktop`, `open_task_manager`) keep `log::` calls (`:90`, `:100`, `:106`, `:115`).
17. Hold path: `start_hold_timer` spawns `hold-timer`, `sleep_until(due)`, `with_engine(on_timer)`, then `carry_out(get_hook(), decision)` on that thread (`keyboard_hook_callback.rs:311-325`). `sleep_until` still uses `thread::sleep` (`:330-338`); `keyboard_hook_callback.rs` is not in X6's scan list (`executor/mod.rs:1324-1328`).

### Shared state and orderings
18. `KeyboardHook.executor: ArcSwapOption<ExecutorHandle>` (`keyboard_hook.rs:73`), initialised empty (`:86`), stored only by `attach_executor` (`:133-135`).
19. `update_bindings`: sort, one `store`, then `cancel_actions()` (`keyboard_hook.rs:121-127`).
20. `set_enabled`: `with_engine` swaps `enabled` and resets the engine; `cancel_actions()` only when the result is `Some(true)`, after the lock is released (`keyboard_hook.rs:189-202`).
21. `cancel_pending` = `epoch.fetch_add(1, AcqRel)` + `wake()` (`executor/mod.rs:240-243`). `Shared::live(e)` = `!stopping.load(Acquire) && epoch.load(Acquire) == e` (`:189-196`).
22. `stop_thread`: `stopping.store(true, Release)`, `cancel_pending()`, `done.recv_timeout(STOP_TIMEOUT)`; `Ok`/`Disconnected` -> `join()` -> `Joined`; `Timeout` -> `TimedOut` with the `JoinHandle` dropped (detached) (`executor/mod.rs:333-348`). `Drop for Executor` calls `stop_thread` (`:351-355`).
23. `active_runs` stored with `Release` at `executor/mod.rs:563`, `:629`, `:643`; loaded `Acquire` at `:247`. Counters: `bump` `fetch_add AcqRel` (`counters.rs:116`), gauge `fetch_max AcqRel` (`:135`), `take` `swap AcqRel` (`:148-151`), `longest_callback` `load Acquire` (`:140`). No `Relaxed`/`SeqCst` in `src/core/executor` (grep of `Ordering::` in mod.rs lists only Acquire/Release/AcqRel).

### Executor thread
24. `Executor::start`: `thread::Builder` named `"action-executor"`, spawn error returned via `?` (`executor/mod.rs:290-295`); waits `ready.recv_timeout(START_TIMEOUT)` for the thread id (`:296-298`).
25. Prologue: `Apartment::enter_sta()` = `CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)`, failure logs `[WARN]` and runs without COM (`executor/mod.rs:370-385`); `PeekMessageW(.., WM_USER, WM_USER, PM_NOREMOVE)` before the handshake (`:539`, `:545`).
26. `Apartment` holds `PhantomData<*const ()>` (`!Send`) (`executor/mod.rs:360-363`); `Drop` calls `CoUninitialize` iff `initialized` (`:388-397`).
27. `timer`, `runs`, `queue`, `runner`, `apartment` are locals of `run_executor`, lent to the `catch_unwind` closure (`executor/mod.rs:550-554`); a caught loop panic logs one `[ERROR]` and maps to `End::Died` (`:555-560`).
28. Epilogue: `timer.kill()`, `active_runs = 0`, `drop(runs)`; on `Died` drain the queue bumping `ExecutorUnavailable` per job; then drop queue, runner, apartment (`executor/mod.rs:562-574`); `done_tx.send(())` after `run_executor` returns (`:293-294`).
29. `run_loop` inner structure: repeat { `stopping` check -> `try_recv` loop (`Ok` -> `runs.start` + `runs.poll`; `Empty` -> break; `Disconnected` -> `Stopped`) ; `runs.poll`; `report_panics`; store `active_runs`; `stopping` check; break with `next` only if `calls.made == 0` } (`executor/mod.rs:607-637`).
30. After the inner loop: `timer.arm(next, now, take(code_ran))`; `Due` -> `continue`; `Failed` -> `runs.drop_timed()` + `[ERROR]` (`executor/mod.rs:639-649`); then `GetMessageW` (`:653`).
31. `GetMessageW == -1` -> `[ERROR]` + `Died` (`:654-660`); `== 0` (`WM_QUIT`) -> `[ERROR]` + `Died` (`:661-668`); null hwnd + `WAKE` -> `continue` (`:669-671`); null hwnd + `WM_TIMER` + `lParam == 0` -> `timer.fired(wParam)` (`:672-675`); anything else -> `TranslateMessage` + `DispatchMessageW`, `code_ran = true` (`:676-683`).
32. `plan()`: `Idle` -> `Kill`; `at <= now` -> `Due`; `!code_ran && armed == Some(at)` -> `Keep`; else `Arm { ms: timer_ms(now, at), at }` (`executor/mod.rs:436-446`).
33. `DelayTimer::arm` kills before re-creating, `SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)`, id 0 -> `Failed(Error::from_thread())` (`executor/mod.rs:464-480`). `fired(id)` kills only its own id (`:487-491`).
34. `timer_ms`: ceil to ms, `u32::try_from` saturating, clamp `1..=USER_TIMER_MAXIMUM` (`executor/mod.rs:507-515`).
35. `Runs::start`: stale epoch -> nothing (`runs.rs:105-107`); `Sequence(steps)` -> pushed with `Resume::Now`, no call (`:109-117`); other actions -> `runner.action` under `call()` (`:119`).
36. `Runs::poll`: `retain(live)` (`runs.rs:127`), each due run advanced once in age order (`:128-141`), `next` = min `At` deadline or `Idle` (`:148-157`).
37. `Run::advance`: `live` check before each step (`runs.rs:179-181`); `Delay(ms)` -> `now().checked_add(ms)` -> `At` or `WhenCancelled` on overflow, yields (`:186-192`); other steps -> `runner.step` under `call()`; a panic ends the run (`:194-199`).
38. `call()` wraps each runner call in `catch_unwind(AssertUnwindSafe(..))` and returns `Calls { made: 1, panics }` (`runs.rs:207-213`).
39. `run_step`'s `Delay` arm is a no-op (`sequence.rs:110-111`). `apply_window_op`: invalid target -> `[WARN]` + return (`:247-250`); `Close` -> `PostMessageW(WM_CLOSE)` (`:259-269`); others -> `ShowWindowAsync`, `FALSE` -> `[WARN] window step failed: ShowWindowAsync did not start {op:?}` (`:271-278`). No `ShowWindow(` call remains in non-test `sequence.rs`.
40. `spawn_sequence` no longer exists in `src` (grep empty). Non-test `thread::Builder`/`spawn` under `src/core`, `src/data`: only `executor/mod.rs:290`.

### Lifecycle (main thread)
41. `App::run` order: `MAIN_THREAD_ID.store` (`app.rs:187`), pipe `bind` (`:191`), `print_welcome`, Ctrl+C handler (`:195`), `Executor::start(Win32Runner, QUEUE_CAPACITY, &counters::COUNTERS)` with `?` -> `"could not start the action executor: {error}"` (`:199-200`), `update_bindings` (`:204`), `AttachedExecutor::attach` (`:205`), `hook.install()?` (`:209`), ..., `hook.uninstall()?` (`:266`), `executor.stop()` (`:267`).
42. `AttachedExecutor::stop` detaches (`attach_executor(None)`) before `Executor::stop`; `TimedOut` -> `log::warn!("the action executor did not stop within 2 s; exiting without it")` (`app.rs:48-57`); `Drop` calls `stop` (`:59-63`), covering the `?` returns after `:205`.
43. `logging::flush()` stays in `main.rs` after `run` returns (`main.rs:32`, `:39`, `:52`).

### Counters and logging
44. `SLOTS = 7` (`counters.rs:17`); new variants `ExecutorJobsDropped`, `ExecutorUnavailable`, `PostMessageFailed` with `Display` arms (`:31-39`, `:62-64`) and constant-index slots (`:100-110`).
45. `summary` returns `None` iff all counts are 0 and `longest_callback < SLOW_CALLBACK`; the line ends with ` longest_callback_us=<us>` (`counters.rs:166-181`). `SLOW_CALLBACK = 5 ms` (`:72`).
46. `record_callback` saturates `as_nanos` to `u64` (`counters.rs:131-136`).
47. `writer::report_counters` formats `counters::summary(&self.counters.take())` (`writer.rs:224-230`).
48. `start_logging`: `install()` first, `None` without starting the writer when it returns false (`logging/mod.rs:132-140`); `init` uses it (`:109-125`). At `7827ec8` the seam ran `start_writer()` before `install()` (verified with `git show`).
49. `the_functions_on_the_hook_path_log_nothing` scans the list at `logging/mod.rs:393-434` (17 entries incl. `fn wake(`, and the whole non-test part of `counters.rs`); `fn_lines` panics when a signature is missing (`:284-301`).
50. `config/loader.rs:379` still has `log::info!("no auto-blocker for {hotkey}: ...")` inside `expand_with_auto_blockers`, which `parse_ron_str` calls (`:294`). `parse_ron_str` doc says "without logging the warnings" (`:249-250`).

### Tests and harness
51. `cargo test --locked --lib -- --list` at HEAD lists 587 tests (ignored included). Every test name in `impl.md:29-36` is present.
52. `FakeRunner` records first, then may `PostQuitMessage(0)` (`testing.rs:122-129`), sleep (`:130-132`), gate (`:133-136`), or remove its own thread's `WAKE` messages (`:137-143`); `with_panic_on` panics before recording (`:119-121`).
53. `leaked_counters()` = `Box::leak(Counters::new())` per test (`testing.rs:158-160`); `the_keyboard_callback_records_how_long_it_ran` asserts on the process-wide `counters::COUNTERS` (`keyboard_hook_callback.rs:707`).
54. `a_hold_runs_its_slow_action_on_the_executor` holds `GLOBAL_HOOK`, attaches a FakeRunner executor to `get_hook()`, `DetachAtEnd` detaches then stops on drop (`keyboard_hook_callback.rs:833-878`). `send_menu_mask` is a counter under `cfg(test)` (`:288-294`).
55. Only ignored test added: `core::executor::tests::the_win32_runner_opens_a_url` (`executor/mod.rs:1350-1363`, manual M9).
56. Drift: `cut.md:396` says X2 makes X1 green with "27 pass, 1 ignored"; `impl.md:16` says "28 pass, 1 ignored". `cut.md:88-99` (D2) and `:184` name `SetTimer`; the code uses `SetCoalescableTimer` (`executor/mod.rs:474`), declared in `cut.md:631` per ruling V2-M3 (`rulings.md:105-109`).

## Red-first tests and acceptance proofs

How to run a test on an older commit without touching the work tree (read-only on the repo):
```bash
SHA=<commit>; D=D:/tmp/ks-orch/p2-2.1-executor/red/$SHA; mkdir -p $D
git -C D:/key-switch-rs/key-switch-rs archive $SHA | tar -x -C $D
CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.1-executor/red-target cargo test --locked --manifest-path $D/Cargo.toml --lib <filter> 2>&1 | tail -40
```
(The scan tests read sources through `CARGO_MANIFEST_DIR`, so they scan the extracted tree.) "Red commit" = the test
commit, which is the parent of its fix. "Master?" says whether the test could even be red for the right reason on `master`.

| # | Test (filter) | Must be red at | Seam it needs (not on master) | Green at | Expected red (cut / impl) | Master? |
|---|---|---|---|---|---|---|
| L3 | `config::loader::tests::mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing` | `fbd6b59` | none (`language_hotkey`, `blockers` exist on master, `loader.rs:1000`, `:1011` @master) | `b6e5a9f` | `assert_eq!` left `["binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true"]`, right `[]` (`cut.md:345`, `impl.md:11`) | yes |
| L5 | `config::loader::tests::an_empty_config_warning_is_returned_not_logged` | `47f2b88` | none | `2ab67df` | left `[]`, right `["config contains no bindings"]` (`cut.md:349`); at `47f2b88` `loader.rs:287` is still `log::warn!` (verified) | yes |
| L8 | `logging::tests::a_second_init_starts_no_writer` | `7827ec8` | `start_logging` seam from `89fcf5e` (L7) | `62bde43` | panic `assertion failed: !started.get()` / message `a second init must not start a writer` (`cut.md:355`, `impl.md:13`) | no (no seam) |
| X1 | all `core::executor::tests` + `core::executor::runs::tests` | `cf3c9f6` | executor module (X2) | `dc2529c` | compile errors: impl reports E0433 `TargetWindow` x34, `Stopped` x17, E0425 `QUEUE_CAPACITY` x15, E0432 `super::runs::{Ctx, Job, Next, Runs}`, E0405 `Runner`; `cargo build` green (`impl.md:16`; cut's list `cut.md:394`) | no |
| V2-I1 | `core::executor::tests::a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` | `a364b77` | executor from `dc2529c`; `FakeRunner::with_eat_wakes` added in `a364b77` | `5ea07d8` | `left: None right: Some("open B")`, deterministic over 3 runs (`impl.md:17`; ruling `rulings.md:93-100`) | no |
| X3 | `data::sequence::tests::a_window_step_never_waits_for_its_window` | `fd5993e` and `d269caf` | none | `8e28876` | assertion listing `src/data/sequence.rs:277`, `:280`, `:283` `ShowWindow(target, SW_*)` (verified at `fd5993e`; cut predicted 276/279/282, `cut.md:638` deviation 8) | yes |
| X5 | `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` | `fcce19b` (also `ba3122f`, `6edfa0f`) | none | `a4a2e32` | `src/hook/keyboard_hook_callback.rs:219: action.execute();` (verified at `fcce19b`) | yes |
| X6 | `core::executor::tests::macros_never_sleep_or_block_in_a_wait` | `ba3122f` (also `6edfa0f`) | executor tests module | `a4a2e32` | `src/data/sequence.rs:133: SequenceStep::Delay(ms) => thread::sleep(Duration::from_millis(*ms)),` (verified at `ba3122f`; cut said `:129`) | no |
| X7 | `core::windows_actions::tests::only_launch_open_url_and_sequence_leave_the_firing_thread`, `hook::keyboard_hook_callback::tests::{carry_out_queues_the_slow_actions_in_order_and_runs_none_here, carry_out_returns_in_under_5_ms_while_slow_actions_block, a_hold_runs_its_slow_action_on_the_executor}`, `hook::keyboard_hook::tests::{swch_off_stops_a_running_macro_within_100_ms, a_reload_stops_a_running_macro_within_100_ms, a_repeated_swch_on_keeps_the_running_macro, a_slow_action_without_an_executor_is_closed}` | `6edfa0f` | executor (`dc2529c`) | `a4a2e32` | compile: E0433 `FastAction` x7, E0599 `attach_executor` x6, E0061 `carry_out` x4, E0599 `submit` x3 (`impl.md:22`) | no |
| X9 | `logging::counters::tests::{the_longest_callback_is_kept_until_taken, a_slow_callback_alone_is_reported_and_a_fast_one_is_not}`, `logging::writer::tests::a_flush_reports_a_slow_callback`, `hook::keyboard_hook_callback::tests::the_keyboard_callback_records_how_long_it_ran` | `0ff3ce2` | none beyond `a4a2e32` | `4db3bb2` | compile: E0599 `longest_callback` x3, `record_callback` x3, E0609, E0422 `Snapshot`, E0425 `SLOW_CALLBACK` (`impl.md:24`) | no |
| X11 | `logging::tests::the_functions_on_the_hook_path_log_nothing` | `37b1847` | `fn_lines` generalised in `37b1847`; scanned functions exist only after `a4a2e32` | `6a1db71` | 3 sites: `src/core/windows_actions.rs:278` (`log::error!("Post message error ...")` in `fn post_message_to_foreground(`), `src/system/system_function.rs:58`, `:77` (verified at `37b1847`) | no (on master `fn_lines` would panic on missing signatures: wrong reason) |

Whole-suite state per commit (from `cut.md:339`, `impl.md:163`): every commit not listed as red passes fmt, clippy and the
tests it touches; between `fcce19b` and `6edfa0f` X5, X6, X7 are red together; `6edfa0f`, `cf3c9f6`, `0ff3ce2` do not compile
under `cargo test` (`cargo build` green per impl).

### Acceptance proofs at HEAD

| Acceptance (spec `spec.md:12-17`, rulings) | Proof | Expected |
|---|---|---|
| `rg "\.execute\(\)" src/hook` finds nothing | `rg -n '\.execute\(\)' src/hook`; test X5 | no output, exit 1; test passes |
| 100 Sequence runs create no threads | `cargo test --locked --lib core::executor::tests::a_hundred_sequence_runs_use_the_one_executor_thread`; `rg -n 'thread::spawn\|thread::Builder' src/data src/core` | pass; only `executor/mod.rs:290` and test `:1295` |
| Delay(10000) stops within 100 ms of an epoch change | `core::executor::tests::delay_10000_stops_within_100_ms_of_an_epoch_change` (`mod.rs:766-789`); `hook::keyboard_hook::tests::swch_off_stops_a_running_macro_within_100_ms`, `a_reload_stops_a_running_macro_within_100_ms`; exit: `stop_interrupts_a_delay_drops_the_queue_and_joins` (`mod.rs:1158-1182`, stop < 1 s) | pass |
| 20 slow jobs, each enqueue < 5 ms | `core::executor::tests::twenty_slow_jobs_each_enqueue_in_under_5_ms` (`mod.rs:995-1030`), `hook::keyboard_hook_callback::tests::carry_out_returns_in_under_5_ms_while_slow_actions_block` (`:793-831`); rule: all 20 < 5 ms, one whole-batch retry (`rulings.md:83-85`) | pass; a flake goes to the orchestrator |
| full queue drops and counts | `core::executor::tests::a_full_queue_drops_the_job_and_counts_it` (`mod.rs:1070-1102`, capacity 1) | pass, `executor_jobs_dropped == 1` |
| R1 four tests | `a_delay_5000_macro_holds_up_no_other_job` (< 100 ms), `two_macros_fired_together_both_run_in_their_own_order` (`[F13,F15,F14,F16]`) + `runs::tests::two_macros_fired_together_interleave_at_their_delays_and_keep_their_order`, `a_cancelled_waiting_run_never_resumes` (both modules), `a_delay_of_u64_max_waits_until_cancelled` + `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms` + `timer_ms_rounds_up_and_clamps` | pass |
| R1 no sleep / blocking wait | `core::executor::tests::macros_never_sleep_or_block_in_a_wait` | pass |
| R2 | `data::sequence::tests::a_window_step_never_waits_for_its_window` | pass |
| R3 | `an_executor_ended_by_wm_quit_counts_every_later_submit`, `a_submit_after_stop_is_closed_and_not_counted` | pass |
| V2-M3 | `the_delay_timer_is_kept_only_while_nothing_ran` | pass |
| STA COM | `the_executor_runs_jobs_in_a_single_threaded_apartment`, `the_apartment_guard_uninitialises_com_on_drop` | pass |
| M5 Hold split | `hook::keyboard_hook_callback::tests::a_hold_runs_its_slow_action_on_the_executor` | pass, record on `action-executor` |
| D10 gauge | the four X9 tests | pass |
| D11 / M1 | `logging::tests::the_functions_on_the_hook_path_log_nothing`, `only_the_startup_banner_prints_outside_src_bin` | pass |
| leftovers | L3, L5, L8 tests above | pass |
| App::run order | review only (KS-38, no harness): `app.rs:199-209`, `:266-267` | n/a |

Gate commands (tester owns the full suite; `impl.md:69` says the developer ran only the touched modules: 314 passed, 0
failed, 5 ignored on `--lib -- core:: hook:: data:: logging:: config:: system::`):
```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked 2>&1 | rg -n "test result:|FAILED|panicked"
cargo test --locked --lib core::executor -- --test-threads=1   # optional repeat for timing tests
mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/book
```
Never `--ignored`/`--include-ignored` (the ignored tests open a browser or inject input).

### Manual checks (human only; listed, not run)
M1-M10 `cut.md:476-490`; M11 (with a `master` control run) `impl.md:128`, `rulings.md:101-104`; M12 `impl.md:129`,
`rulings.md:117-119`; M13 `impl.md:130`, `rulings.md:105-109`; M9 = the ignored test above.

## Open points from impl.md (verbatim scope, `impl.md:165-171`)
1. M12's result decides whether the X4 Changed entry (`CHANGELOG.md:360-370`) needs a sentence about `Window(Restore)` not bringing a minimised window to the front.
2. The two 5 ms timing tests follow the ruling (all 20, one batch retry); a flake goes to the orchestrator.
3. Residual race R-7 (a `WAKE` posted to a reused thread id from a `hold-timer` thread at exit) accepted until 2.2 (`rulings.md:86`, `cut.md:501`).
4. Pre-existing: 4 `cargo doc` warnings (`config/loader.rs:156`, `:176`, `keyboard_hook.rs` `set_enabled` doc link, `data/vk_name.rs:4`); mdbook-mermaid version warning.
5. `SystemFunction::{LockWorkstation, ShowDesktop, TaskManager}` have no `BindAction` and keep `log::` calls (Fact 16); new feature 1 must place them in `as_fast`.
6. Non-config book pages left for the Phase-2 book sync, with lines: `impl.md:114-123` (architecture.md, glossary.md, overview.md, build-and-run.md, gaps.md, roadmap.md, audit.md, methodology.md).
7. Critic OQ1 (loop panic): no natural seam for a test; ownership is a review item (`cut.md:616`; Fact 27-28).

## Invariants in play

| Invariant | Where |
|---|---|
| R1 hook callback: no alloc/lock/log/block; `code<0` first; one outcome | `keyboard_hook_callback.rs:340-384`, `:206-242`; `executor/mod.rs:89-95`, `:213-259`; `keyboard_hook.rs:140-153`; `windows_actions.rs:142-183`, `:232-283`; `counters.rs:113-136` |
| R2 sentinel on every SendInput | `windows_actions.rs:242`, `:254`; `sequence.rs:157`, `:223`; `keyboard_hook_callback.rs:256`; check `is_injected_event` `:187` |
| R3 shared state (`ArcSwapOption`, atomics; `with_engine` not taken on the executor; cancel after the lock) | `keyboard_hook.rs:73`, `:121-153`, `:189-202`; `executor/mod.rs:177-197` |
| R4 orderings | Facts 21-23 |
| R5 builders | no `Binding` builder change; `Binding::execute` removed (`binding.rs:244-247`) |
| R6 Display for new variants | `counters.rs:55-67` (`Counter`); `FastAction`/`Submitted`/`Stopped`/`Plan`/`Arming`/`End` are `Debug` only |
| R7 specificity sort, single store | `keyboard_hook.rs:121-127` |
| R10 loop and shutdown (executor before hooks, stop after uninstall, bounded) | `app.rs:24-63`, `:199-209`, `:266-267`; `executor/mod.rs:329-355` |
| Timers (generation/cancel) | `runs.rs:127`, `:179`; `executor/mod.rs:436-502` |
| KS-08 (fixed for slow actions), KS-33 (Sequence half), KS-01 (swch off stops macros), KS-20 (start failure before hooks), KS-38 (no daemon harness) | CHANGELOG `:481-510`, `:649-`; `app.rs:176-182` doc |

## Not covered
- Not read: `executor/runs.rs:215-524` test bodies beyond names; `executor/mod.rs:687-745` test helpers in detail;
  `keyboard_hook_callback.rs:417-690` pre-existing tests; `logging/writer.rs` beyond `:224-230` and hunk starts;
  `system_function.rs:119-165`; the text of the README, CHANGELOG and config-page hunks (only their positions);
  `critique.md`/`critique_v2.md` bodies (only as cited by rulings and the addendum).
- No test, clippy or fmt run (only `cargo test --locked --lib -- --list`, which compiles and lists). No red commit was
  built; red sites were checked with `git show <sha>:<path>` only.
- `arc-swap` `ArcSwapOption::load` allocation/lock behaviour on the hook path: not checked (UNVERIFIED).
- std `sync_channel` `try_send` lock-freedom claim (`cut.md:103`): not re-checked.
- Book build / link check: `impl.md:109` reports 0 broken of 7877; not re-run.
