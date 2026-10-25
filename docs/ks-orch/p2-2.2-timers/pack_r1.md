# Context pack, round 1: p2-2.2-timers (roadmap 2.2, KS-11, KS-25, KS-33)

For the tester and the code-reviewer. Facts only, each with `file:line` on HEAD `d738632` unless marked.
Lane inputs: `spec.md`, `cut.md` (the cut plus "Developer addendum" `cut.md:393-525`), `rulings.md` (wins over
the cut), `impl.md`. All paths relative to `D:/key-switch-rs/key-switch-rs` unless absolute.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/main-thread-timers`, HEAD `d738632`, tree clean
  (`git status --short` empty). `master` = `c9c1373`.
- `git diff --stat master..feat/main-thread-timers`: 31 files, +4762 / -1147. Source: `src/bench.rs` (new, 692),
  `src/hook/main_thread.rs` (new, 1006), `src/engine/schedule.rs` (new, 165), `src/platform/priority.rs` (new, 78),
  `src/engine/gestures.rs` (+-1212), `src/engine/mod.rs` (646), `src/engine/scenarios.rs` (506),
  `src/hook/keyboard_hook_callback.rs` (526), `src/hook/keyboard_hook.rs` (211), `src/core/app.rs` (117),
  `src/engine/{chords,sequences,winner,testing,properties,env}.rs`, `src/core/windows_actions.rs` (56),
  `src/logging/{mod,counters}.rs`, `src/platform/mod.rs`, `src/hook/mod.rs`, `src/lib.rs`.
  Docs: `CHANGELOG.md`, `README.md`, `src/config/default_config.ron`, `book/src/config/{actions,errors,kinds,known-issues,structure,triggers-conditions}.md`.
- Not in the diff (cut "Not touched", `cut.md:245`): `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/core/executor/*`,
  `src/platform/timers.rs`, `src/hook/ipc_server.rs`, `src/hook/config_watcher.rs`, `tests/ipc_roundtrip.rs`,
  `CLAUDE.md`, agent files, every non-config book page.

### Commits (`git log --oneline --reverse master..feat/main-thread-timers`)

| # | sha | Subject (short) | Kind | Files |
|---|---|---|---|---|
| C1 | `611c83f` | Bench: latency bench | green | bench.rs, lib.rs |
| C2 | `e969dd7` | Tests: repro stale-timer tests ported, inverted | green (red only at 7cb3521) | scenarios.rs |
| C3 | `68b13a0` | Tests: timer tokens, queue, Scheduler seam | (R) compile | engine/mod.rs, schedule.rs |
| C4 | `cfd7be6` | Engine: tokens gen/slot/seq; Scheduler; TimerQueue | green | gestures, mod, schedule, testing |
| C5 | `e9fc2f4` | Tests: no thread on the gesture path | (R) assertion | keyboard_hook_callback.rs |
| M7 | `7cb6868` | Core: under test a fast action is only recorded | green | windows_actions.rs |
| C6 | `1aa717f` | Tests: main thread owns engine, wait, swch on/off | (R) compile | app.rs, keyboard_hook_callback.rs, main_thread.rs, hook/mod.rs |
| C7 | `464d3d4` | Hook: engine and gesture timers on the main thread | green | 15 files (code + CHANGELOG/README/actions/errors/triggers) |
| C8 | `b68c7f2` | Tests: one press machine; DoubleTap per group | (R) assertion | engine/mod.rs, scenarios.rs |
| C9 | `506aad3` | Engine: one gesture machine per press (KS-25, KI 74) | green | gestures, mod, winner, properties, counters + docs |
| C10 | `c4609ea` | Tests: another key-down cancels (KI 67) | (R) assertion | scenarios.rs (+126/-15) |
| C11 | `c5ef59a` | Engine: another key-down cancels | green | gestures, mod + docs |
| C12 | `64b4ead` | Tests: chord/sequence expiry timers | (R) assertion | engine/mod.rs |
| C13 | `3e516f9` | Engine: chord/sequence expiry timers | green | chords, sequences, mod, kinds.md, CHANGELOG |
| C14 | `4e9dd62` | Bench: R8 sent-message under load | green | bench.rs, CHANGELOG |
| C15 | `0a8caec` | Tests: priority guard | (R) compile | platform/mod.rs, priority.rs |
| C16 | `2f11917` | Core: main thread pumps at HIGHEST | green | priority.rs, platform/mod.rs, app.rs + docs |
| - | `d738632` | Hook: comments only | green | engine/mod.rs, keyboard_hook_callback.rs |

## File map

| path:range | What lives there | Why it matters this round |
|---|---|---|
| `src/hook/main_thread.rs:1-47` | module docs: State, Loop steps 1-4 | D1/D3 claims the code must meet |
| `src/hook/main_thread.rs:76-111` | `MAIN` thread-local `RefCell<Option<MainState>>`, test `REFUSED`, `MainState`, `with_main` | R3, ruling Q1 |
| `src/hook/main_thread.rs:113-137` | `MainState::set_enabled` (switch, reset, retain, cancel_actions) | D2, OQ5 |
| `src/hook/main_thread.rs:139-271` | `GestureTimers`, `Wake`, `Plan`, `plan`, `plan_wake`, `impl Scheduler` (`start`, `retain`) | D3/D4; `start` runs in the callback |
| `src/hook/main_thread.rs:273-367` | `Request`, `MainThreadError` + Display, `MainThreadHandle` (`set_enabled`, `request_exit`, `wake`, test `without_a_thread`) | D2, M4, R6 |
| `src/hook/main_thread.rs:369-563` | `MainThread` (`new`, `handle`, `run`, `drain`, Drop), `take_due_timer`, `serve`, `pump` | D3 loop, unsafe blocks, R10 |
| `src/hook/main_thread.rs:565-1006` | 11 tests (list in Red-first table) | C6 acceptance |
| `src/hook/keyboard_hook_callback.rs:1-86` | module docs (steps, Threads, Re-entry) | R1 narrative |
| `src/hook/keyboard_hook_callback.rs:120-161` | `HookEnv` (`pub(super)`, `now` = `Instant::now()`) | clock per event |
| `src/hook/keyboard_hook_callback.rs:175-253` | `decide_event` (`pub(super)`), `carry_out` (`pub(super)`), `dispatch` | R1, D7 |
| `src/hook/keyboard_hook_callback.rs:313-383` | `keyboard_hook_callback`, `foreground_changed` | R1 outcome paths |
| `src/hook/keyboard_hook_callback.rs:386-1013` | tests incl. migrations, 4 new (`763-1006`), 1 manual ignored (`1008-1013`) | M1, C5, C6 |
| `src/hook/keyboard_hook.rs:24-131` | `KeyboardHook` (no engine field), `cancel_actions` `pub(crate)` `:98`, `is_enabled` `:112-114`, `switch_enabled` `:123-125` | D2, R4 |
| `src/hook/keyboard_hook.rs:235-558` | tests; migrated `set_enabled` helper `:259-261`, `set_enabled_refuses_a_nested_call` `:539-557` | M1 |
| `src/engine/mod.rs:220-248` | module list incl. `schedule`; `pub use schedule::{Scheduler, TimerQueue}` | D4 |
| `src/engine/mod.rs:287-347` | `TimerToken`, `TimerSlot`, `Ids`, `TimerRequest` | D4 |
| `src/engine/mod.rs:398-540` | `on_key`, `on_timer`, `is_live`, `start_timers`, `forget_timer`, `on_foreground_changed`, `reset` | D4, D7, D8 |
| `src/engine/mod.rs:542-763` | `on_key_down` (other_key_down, deferred_group, press, sequences, chords, repeat rule), `on_key_up` | D5, D6, D8 |
| `src/engine/mod.rs:776-1201` | engine unit tests (7 new: `:917-1164`) | C3, C8 (M6), C12 |
| `src/engine/gestures.rs:1-454` | rewritten press machine: `GroupKey`, `Press`, `Phase`, `DueHold`, `Released`, `press`, `release`, `other_key_down`, `on_timer`, `fire_closed_windows`, `is_live`, `forget`, `clear` | D5, D6, I1, M6 |
| `src/engine/gestures.rs:456-954` | 15 rewritten unit tests | M1 mapping `impl.md:71-95` |
| `src/engine/winner.rs:89-201` | `GroupArming`, `Winner::deferred_group`, `admits`, `GroupArms::claim` | D5, R7 |
| `src/engine/schedule.rs:1-165` | `Scheduler` trait `:21-28`, `TimerQueue` `:31-91`, 2 tests `:123-165` | D4 |
| `src/engine/chords.rs:32-139`, `src/engine/sequences.rs:32-148` | `seq` field, `Started{due}` / `Advanced{due}`, `expire`, `is_live` | D8 |
| `src/engine/testing.rs:147-449`, `:663-666` | `Scenario.timers: TimerQueue` `:155`, `advance_ms` `:405-420`, `advance_ms_stalled` `:426-431`, `lose_timers` `:436-448`, `drop_stale_timers` | harness = Scheduler fake |
| `src/engine/scenarios.rs:391-435`, `:495-576`, `:722-1011`, `:3723-3894`, `:4716-4732` | ports, equal-term acceptance, C8 scenarios, C10 scenarios, `a_hold_timer_never_masks` | acceptance |
| `src/engine/properties.rs:12`, `:180` | new last config `tap_double_tap` | D5 coverage |
| `src/core/app.rs:68-95`, `:180-282`, `:344-404` | `DaemonHandle { hook, main }`, `App::run` order, 2 tests | D2, D9, R10, M4(c) |
| `src/core/windows_actions.rs:138-180`, `:423-437` | `FastAction::run` cfg(test) record; `under_test_a_fast_action_is_only_recorded` | ruling M7 (safety) |
| `src/platform/priority.rs:1-78`, `src/platform/mod.rs:1-10` | `RaisedPriority` raise/Drop, test | D9 |
| `src/logging/mod.rs:433-438` | hook-path log scan entries for `main_thread.rs` | CLAUDE.md principle 1 |
| `src/logging/counters.rs:25-31` | `HoldTimerNotStarted` doc | D4 |
| `src/bench.rs:1-692` | `latency_bench` (ignored) rows R1-R8 | D10, Q5 |

## Facts

Loop and main state
1. `MAIN` is `thread_local! RefCell<Option<MainState>>` (`src/hook/main_thread.rs:76-79`); `MainState` fields `engine`, `timers` are `pub(super)` (`:89-92`).
2. `with_main` uses `MAIN.try_with` + `try_borrow_mut`; a borrowed state returns `None` and, under `cfg(test)`, bumps `REFUSED` (`main_thread.rs:100-111`).
3. `MainThread::new` builds `GestureTimers` first, logs `[WARN]` if the timer is not high-resolution, installs the state only if the slot is empty (else `Err(E_UNEXPECTED)`), then `PeekMessageW(.., WM_USER, WM_USER, PM_NOREMOVE)` and a `sync_channel(REQUEST_CAPACITY = 8)` (`main_thread.rs:384-426`, `:74`).
4. `MainThread` is not `Send` (`PhantomData<*const ()>`, `main_thread.rs:374-378`); its Drop takes the state via `try_with`/`try_borrow_mut` (`:502-512`).
5. `run` loop order: `drain` -> `plan_wake` (`Wake::Due` -> `continue`; `Failed` -> `log::error!`) -> `MsgWaitForMultipleObjectsEx(Some(&[timer]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)` -> `WAIT_OBJECT_0` sets `armed = None` and continues; `WAIT_FAILED` returns `Err(Error::from_thread())`; else `pump` (`main_thread.rs:436-480`).
6. `drain` loops until a pass serves no request (`try_recv`) and no due timer (`with_main(take_due_timer)`); each timer decision is carried out by `carry_out` after the borrow (`main_thread.rs:484-499`).
7. `take_due_timer` takes the earliest due by `Instant::now()`, calls `engine.on_timer`, starts the decision's timers under the same borrow and bumps `HoldTimerNotStarted` per refusal (`main_thread.rs:516-524`).
8. `pump` loops `PeekMessageW(PM_REMOVE)`: `WM_QUIT` -> true; thread message (`hwnd.is_invalid()`) `MAIN_WAKE` -> false (back to drain); else `TranslateMessage` + `DispatchMessageW` (`main_thread.rs:544-563`). `MAIN_WAKE = WM_APP + 1` (`:67`).
9. `plan(armed, next, now)`: `None` -> Cancel; `at <= now` -> Due; `armed == Some(at)` -> Keep; else `Arm { after, at }` (`main_thread.rs:171-181`).
10. `plan_wake` first `queue.retain(is_live)`; on `wake.set` failure it collects every queued token into a `Vec` and calls `engine.forget_timer` on each (`main_thread.rs:209-245`).
11. `GestureTimers::start` (runs inside the callback under the borrow) pushes to the queue and calls `wake.set` only if `armed.is_none_or(|a| request.due < a)`; on failure it removes that token and returns false, `armed` unchanged (`main_thread.rs:254-266`); it reads `Instant::now()` at `:257`.
12. `MainThreadHandle::set_enabled`: reply `sync_channel(1)`, `try_send` (Full -> `Busy`, Disconnected -> `ShuttingDown`), `wake()`, `recv_timeout(REPLY_TIMEOUT = 2 s)` (Timeout -> `NoAnswer`) (`main_thread.rs:317-334`, `:71`). `wake` ignores the `PostThreadMessageW` error (`:349-355`).
13. `serve` answers with `reply.try_send` and ignores the error; if `with_main` returns `None` the reply sender is dropped (`main_thread.rs:528-539`).
14. `MainState::set_enabled`: `hook.switch_enabled(enabled, self)`; only if it changed: `engine.reset(env)`, `timers.retain(is_live)`, `hook.cancel_actions()` (`main_thread.rs:122-136`). `switch_enabled` takes `_main: &mut MainState` and does `enabled.swap(enabled, AcqRel) != enabled` (`src/hook/keyboard_hook.rs:123-125`); `is_enabled` loads `Acquire` (`:112-114`).
15. `MainThreadError` Display has one arm per variant: "the daemon's main thread did not answer within 2 s", "the daemon is shutting down", "the daemon is busy; try again" (`main_thread.rs:294-302`).

Adapter
16. `keyboard_hook_callback`: `code < 0` -> `CallNextHookEx` (`keyboard_hook_callback.rs:318-323`); `!is_enabled()` -> `CallNextHookEx` without reading lparam (`:326-331`); suppress -> `LRESULT(1)` (`:350-352`); else `CallNextHookEx` (`:356`).
17. `decide_event`: sentinel check `is_injected_event` (`:188-190`) precedes `key_event` (`:191`), a second `is_enabled` check (`:196-198`), the `ArcSwap` snapshot (`:201`) and `with_main` (`:203-208`); `on_key` and `start_timers` run under one borrow; refusals bump `HoldTimerNotStarted` after it (`:211-213`).
18. `carry_out` destructures `Decision` with `timers: _` and runs mask, then actions (`keyboard_hook_callback.rs:222-239`); `dispatch` runs a fast action inline or `submit`s a slow one (`:245-253`).
19. `foreground_changed` returns early while off (`:377-379`), else `with_main(|m| m.engine.on_foreground_changed(&env))` (`:382`).
20. `rg "Mutex<Engine>|with_engine|DECIDING|DecidingGuard|start_hold_timer|sleep_until" src` returns nothing; `hold-timer` remains only in comments at `src/core/executor/mod.rs:6` and `src/logging/counters.rs:29`.
21. Under `cfg(test)`, `FastAction::run` only pushes into thread-local `FAST_ACTIONS_RUN`; the Win32 `match` is `cfg(not(test))` (`src/core/windows_actions.rs:148-165`, `:168-180`). `send_menu_mask` under test only counts (`keyboard_hook_callback.rs:302-311`).

Engine
22. `TimerToken { generation: u32, slot: TimerSlot, seq: u64 }`, private fields; `TimerSlot::{Gesture(u16), Chord, Sequence}` (`src/engine/mod.rs:294-310`); `Ids::next_seq` wraps, never resets (`:321-327`).
23. `on_timer` returns an empty `Decision` for a foreign generation, else dispatches by slot; Chord/Sequence slots fire nothing (`mod.rs:435-447`). No code path in `on_timer` sets `mask` or `suppress` (`:435-447`); the harness asserts both false per timer (`src/engine/testing.rs:411-412`).
24. `start_timers` returns 0 at once for an empty Vec, else `scheduler.retain(is_live)` then `start` each; a refused one -> `forget_timer` (`mod.rs:465-482`).
25. `forget_timer` acts only on `Gesture` slots (`mod.rs:489-498`); `Gestures::forget` drops a Hold (held or due) and keeps a press with a Tap or DoubleTap (`src/engine/gestures.rs:406-417`).
26. `reset` clears gestures, chords, sequences, `mask_owed`, re-reads keys from the OS and bumps `generation` (`mod.rs:533-540`).
27. `on_key_down`: for a first key-down only (`!was_repeat`) it calls `other_key_down` first, then `deferred_group`, `next_seq`, `gestures.press`, pushes a `Gesture` timer for a Hold due, and sets `suppress` from `group.block` (`mod.rs:554-580`); deferred triggers in the binding loop do nothing (`:629-631`).
28. Sequences and chords take a `next_seq` per binding visited on a first key-down and push a `Sequence`/`Chord` timer on `Advanced { due }` / `Started { due }` (`mod.rs:640-655`, `:672-684`).
29. `on_key_up` takes a `next_seq` on every key-up, calls `gestures.release(vk, now, was_passed, seq)` and pushes a `Gesture` timer only for `released.window_due` (`mod.rs:744-752`); suppression rule unchanged: `was_blocked || (gesture_bound && !was_passed)` (`:759-760`).
30. `Decision::timers` is `Vec<TimerRequest>` from `Default` (`mod.rs:351-368`); pushes at `mod.rs:573-575`, `:652-654`, `:681-683`, `:749-751` are on the hook path. `TimerQueue::new` reserves 16 (`src/engine/schedule.rs:43-47`).
31. `Gestures::press`: an `AwaitSecond` of the same vk completes (fires `double`, returns `None`) iff `now < due` and the new group has a DoubleTap with `keys ==` and `condition ==`; else its `deferred_tap` fires; a stale `Held` with a Hold due by the clock becomes a `DueHold` (`gestures.rs:173-208`). It clones `KeyCombination` + `Condition` only when the group has a DoubleTap (`:215-221`).
32. `Gestures::release`: Hold present and `now >= due` -> `DueHold`, return; else Tap live if `now < tap.due`; window open (`now < double.due`) -> `AwaitSecond` with the key-up's new `seq`, `window_due` set only with a Tap; Tap live, window closed -> fire now (`gestures.rs:234-283`).
33. `Gestures::other_key_down` (vk != press.vk): `AwaitSecond` with Tap -> fire Tap and drop; `AwaitSecond` without Tap -> drop iff stored `passed`; `Held` -> drop iff `keys.is_passed(press.vk)` and not (`hold` due by `now`) (ruling I1) (`gestures.rs:296-334`). `DueHold`s are a separate Vec and never visited (`:147`).
34. `Gestures::on_timer` takes a matching `DueHold` or a press whose phase waits for a timer, then fires every `AwaitSecond` Tap with `due <= that timer's due` before its own action (M6(a)) (`gestures.rs:340-389`).
35. `Winner::deferred_group` returns `None` for an Immediate winner and walks `bindings.iter().enumerate().skip(self.index)`, skipping auto-blockers, using `admits` and `GroupArms::claim`; `block` ORs members' `block_default` (`src/engine/winner.rs:109-144`).
36. Chord/sequence lazy `retain(|p| p.expires_at > now)` stays (`src/engine/chords.rs:87`, `src/engine/sequences.rs:86`); `expire(seq)` and `is_live(seq)` by seq (`chords.rs:129-138`, `sequences.rs:140-148`); a sequence advance overwrites `entry.seq` (`sequences.rs:93-95`).

App, priority, docs
37. `App::run` order: `Executor::start` (`app.rs:196-197`), `MainThread::new` (`:201-202`, error "could not create the gesture timer: ..."), `install` (`:211`), IPC `serve` with `DaemonHandle { hook, main: main.handle() }` (`:218-228`), watcher (`:237-253`), `RaisedPriority::raise` with `[WARN]` fallback (`:258-266`), `main.run(hook)` (`:270`), `drop(raised)` (`:271`), `drop(main)` (`:274`), `uninstall` (`:277`), `executor.stop()` (`:278`), `ran` error -> "the main message loop failed: ..." (`:280`).
38. `RaisedPriority::raise` = `SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST)`; Drop restores `THREAD_PRIORITY_NORMAL`, error ignored; `!Send` (`src/platform/priority.rs:25-48`).
39. The hook-path log scan lists `main_thread.rs` `fn with_main<`, `fn start(`, `fn retain(` (`src/logging/mod.rs:433-438`); `main_thread.rs` has one `fn start(` (`:254`) and its `log::` calls are at `:387` (`new`) and `:448` (`run`).
40. `the_gesture_path_starts_no_thread` scans non-test lines of `src/hook/*.rs` minus `ipc_server.rs`/`config_watcher.rs`, `src/engine/*.rs`, `src/core/windows_actions.rs`, `src/system/system_function.rs`, `src/platform/timers.rs` for `thread::spawn|Builder|scope` and `use std::thread`; asserts >= 15 files (`keyboard_hook_callback.rs:956-1006`). `src/bench.rs` is not scanned (`#[cfg(test)] mod bench;`, `src/lib.rs:17`).
41. Test functions removed vs `master` (each mapped in `cut.md:419-444` / `impl.md:71-95`): `app.rs` `daemon_handle_status_follows_the_hook_flag`; `keyboard_hook.rs` `with_engine_refuses_reentry_on_the_same_thread`, `with_engine_works_again_after_a_call`; `keyboard_hook_callback.rs` `a_hold_runs_its_slow_action_on_the_executor`, `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`; `scenarios.rs` `a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap`, `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap` (renamed); `gestures.rs` 18 old unit tests (15 new).
42. `impl.md:157` lists the declared red commits as C3, C6, C8, C10, C12, C15; `impl.md:15` and `cut.md:257` mark C5 (`e9fc2f4`) as (R) too.
43. Doc hunks on HEAD: `triggers-conditions.md` +63, 67-68, 73, 75, 77, 79, 87, 92-93, 95, 102-107, 109-112, 120, 123, 134-135; `known-issues.md` 61-62, 135, 142-143 (row 75 new at :143); `actions.md` 38, 44, 49; `errors.md` 196-200, 208; `kinds.md` 79, 143, 177, 198; `structure.md` 63; `default_config.ron` 125-126, 128-131, 302, 319-323; `CHANGELOG.md` 131-141, 210-286, 630-644, 744-749.
44. Book pages left for the Phase-2 sync are listed at `impl.md:217-227` (incl. `src/core/executor/mod.rs:6`).

## Red-first tests and acceptance proofs

Test commands (debug, default run): `cargo test --locked --lib <path> -- --exact`. A commit other than HEAD is run in a
separate worktree at that sha (the main tree stays on HEAD). "Red on" = the (R) commit, whose next commit is the fix.

| Test (path) | Location on HEAD | Red on | Expected red | Green from |
|---|---|---|---|---|
| `engine::scenarios::stale_tap_gc_eats_second_quick_tap` | `scenarios.rs:391-408` | green on master by design (Q6, `rulings.md:74`); red = repro replay at `7cb3521` | inverted repro `assert!(fired)` panics, prints `second tap fired = false` (`cut.md:254`, PREDICTED) | `e969dd7` |
| `engine::scenarios::stale_hold_timer_fires_new_press_early` | `scenarios.rs:412-435` | same | inverted `assert!(still_pending)` panics, `press#2 hold still pending at +110ms = false` (PREDICTED) | `e969dd7` |
| `engine::schedule::tests::{the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order, retain_keeps_only_live_timers}`; `engine::tests::{a_token_from_before_a_reset_is_stale, a_cancelled_hold_leaves_its_token_stale, a_timer_the_scheduler_refuses_is_forgotten}` | `schedule.rs:123-165`; `mod.rs:917-1032` | `68b13a0` (compile) | `E0432 unresolved import super::super::TimerSlot`, `E0432 crate::engine::Scheduler, crate::engine::TimerQueue` (`impl.md:13`) | `cfd7be6` |
| `hook::keyboard_hook_callback::tests::the_gesture_path_starts_no_thread` | `keyboard_hook_callback.rs:956-1006` | `e9fc2f4` | `threads started on the gesture path:` `src/hook/keyboard_hook_callback.rs:91: use std::thread;` `src/hook/keyboard_hook_callback.rs:312: let spawned = thread::Builder::new()` (`impl.md:15`) | `464d3d4` |
| `core::windows_actions::tests::under_test_a_fast_action_is_only_recorded` | `windows_actions.rs:423-437` | not run red (it would post WM_NULL to the foreground window, `impl.md:16`, `cut.md:476-478`) | n/a | `7cb6868` |
| 11 `hook::main_thread::tests::*`, `keyboard_hook_callback::tests::{a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued, remapping_off_callbacks_pass_without_asking_for_the_engine, a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop}`, `core::app::tests::daemon_handle_switches_remapping_through_the_main_thread` | `main_thread.rs:667-1005`; `keyboard_hook_callback.rs:763-898`; `app.rs:353-390` | `1aa717f` (compile) | 41 errors: `E0433 MainThread`, `E0425 with_main`, `plan`, `REFUSED`, `MAIN_WAKE`, `GestureTimers`, `E0560 DaemonHandle has no field main`, `E0603 decide_event is private` (`impl.md:17`) | `464d3d4` |
| C8 scenarios (9 red, 2 green) | `scenarios.rs:727-983`, `:985-1011`; `mod.rs:1033-1070` | `b68c7f2` | exact messages `impl.md:31-38`, e.g. `tap_300_and_hold_200_released_at_250_fire_only_the_hold`: `up CapsLock at +250 ms: expected to fire [], fired [SwitchLanguage]`; KI 74 `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins`: `down CapsLock at +80 ms: expected to fire [], fired [SwitchLanguage]`; `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`: panic `the Tap waits for the window`; green already: `tap_100_and_hold_200_released_at_150_fire_nothing`, `a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up` | `506aad3` |
| C10 KI-67 flips | `scenarios.rs:3723-3810` | `c4609ea` | `a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap`: `down LShift at +150 ms: expected to fire [], fired [ToggleCapsLock]`; `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap`: `up LCtrl at +90 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`; `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold`: `advance 800 ms to +1130 ms: expected to fire [], fired [SwitchLanguage]` (`impl.md:42-44`); green on both: `a_lone_passing_ctrl_tap_still_fires` `:3852-3865`, `another_key_does_not_cancel_a_swallowed_double_tap` `:3867-3894` | `c5ef59a` |
| I1 `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` | `scenarios.rs:3812-3850` | green on `c4609ea`; red only as a mutation of `c5ef59a`: drop `&& !hold.as_ref().is_some_and(\|(due, _)\| now >= *due)` at `gestures.rs:319` | `advance 0 ms to +1001 ms: expected to fire [SwitchLanguage], fired []` (`impl.md:46`, `cut.md:403-406`) | `c5ef59a` |
| C12 `engine::tests::{a_chord_start_asks_for_a_timer_at_the_end_of_its_window, a_chord_expiry_timer_drops_the_pending_chord, each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale}` | `mod.rs:1081-1164` | `64b4ead` | `one expiry timer, got []` x3 (`impl.md:23`) | `3e516f9` |
| `platform::priority::tests::raise_sets_highest_and_drop_restores_normal` | `priority.rs:61-77` | `0a8caec` (compile) | `E0432 unresolved import crate::platform::priority::RaisedPriority` (`impl.md:26`) | `2f11917` |

KS-11 repro replay (C2 red): build the crate of `book/src/methodology.md:234-290` against a `7cb3521` export
(`methodology.md:577-588`), change `assert!(!fired, ..)` (`:275`) to `assert!(fired)` and `assert!(!still_pending, ..)`
(`:289`) to `assert!(still_pending)`, run `cargo test --manifest-path <work>/gesture-repro/Cargo.toml --target-dir <work>/target-gesture -- --test-threads=1 --nocapture` (`methodology.md:615-619`).
PREDICTED: both panic (`cut.md:254`). The tests sleep against a 200 ms term; `methodology.md:342` says run on an idle
machine. `TapState` threads run only `DoNothing` (`cut.md:254`).

Mutations for the tester (rulings OQ1, `rulings.md:47-48`; `impl.md:270`)
- `QS_ALLINPUT` -> `QS_ALLEVENTS` at `src/hook/main_thread.rs:462`: `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer` (`:827-887`) must turn red (assertions `failed == 0` `:880`, median < 2 ms `:883-886`). Not yet run (UNVERIFIED).
- `MWMO_INPUTAVAILABLE` -> `0` at `main_thread.rs:463`: `a_message_posted_behind_a_wake_is_dispatched_without_more_input` (`:772-825`) red with "the WM_CLOSE posted behind a MAIN_WAKE was not dispatched within 1 s"; the developer ran this and restored (`cut.md:481-483`).

### Spec acceptance line -> proof

| Spec acceptance (`spec.md:17-24`) | Proof on HEAD |
|---|---|
| repro ports inverted, pass; red at 7cb3521 | rows 1-2 above |
| Tap(200)+Hold(200): 199 ms -> only Tap, 201 ms -> only Hold | `tap_and_hold_with_equal_terms_fire_exactly_one` `scenarios.rs:495-520`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` `:546-576` (pre-existing, green at every commit, `cut.md:302`); different terms: `:727-772` |
| no thread per press over 1000 presses | scan `keyboard_hook_callback.rs:956-1006` + `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` `:763-795` (asserts queue length <= 1 and 0 live; no thread count is measured) |
| real-timer 200 ms within a few ms | `a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` `main_thread.rs:889-942`: 5 presses, each `held >= 200 ms` (`:926-929`), median lateness < 5 ms (`:936-941`); `pressed` read before `decide_event` (`:915-917`, ruling M8) |
| another key-down cancels a passing gesture; KI-67 flips | C10 row; swallowed kept: `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` `scenarios.rs:1423`, `another_key_does_not_cancel_a_swallowed_double_tap` `:3867-3894` |
| KI-74 no first-binding fire | `scenarios.rs:916-945` |
| bench numbers master + HEAD in impl.md and test_r1.md | `impl.md:162-197`; `test_r1.md` does not exist yet |
| `on_timer` never masks | `a_hold_timer_never_masks` `scenarios.rs:4716-4732` + `testing.rs:412` |
| fast gate kept, safe re-entry | `remapping_off_callbacks_pass_without_asking_for_the_engine` `keyboard_hook_callback.rs:797-854`; `with_main_refuses_reentry_on_the_same_thread` `main_thread.rs:667-683`; `set_enabled_refuses_a_nested_call` `keyboard_hook.rs:539-557` |

### Bench (D10; counted apart from `manual:` tests, never a pass, `rulings.md:69-73`)

- Command (`src/bench.rs:10`, `:614`): `KS_BENCH_OUT=<file> KS_BENCH_LABEL=<label> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`; `KS_BENCH_LOAD=1` adds R8 (`bench.rs:668`), which loads `available_parallelism()` cores (`:566-568`). Panics without `KS_BENCH_OUT` (`:617-618`).
- Master side = `611c83f` (C1, master + bench); HEAD side = `d738632` (`2f11917` in impl; `d738632` is comment-only). 3 runs each; median of p50s, largest p99 (`cut.md:194`).
- Rule (`cut.md:194`): R1-R3 HEAD p50 <= 1.25 x master + 100 ns; R4 p50 <= 1.25 x master + 0.5 us, p99 <= 1.5 x master + 2 us; R5 p50 <= master; R6 p50 <= 1 ms and <= master + 0.5 ms; R7 p50 <= master + 20 us, p99 <= 1.5 x master + 50 us. R8 (`rulings.md:33-35`, `cut.md:457-461`): HIGHEST worse iff p99_H > 1.25 x p99_N + 20 us; D9 stays unless worse in >= 2 of 3 runs.
- Developer numbers: `impl.md:166-197`; files `D:/tmp/ks-orch/p2-2.2-timers/bench-master.md`, `bench-head.md`, `bench-alternate.md`, `bench-head-load.md` (R8 run "HEAD at C14", `bench-head-load.md:2`). R4 HEAD p50 7.0 us vs bound 7.125 us in the first HEAD window; interleaved reruns 5.3-5.5 us (`impl.md:171`, `:179-186`).

### Manual checks (human only; none has an ignored test)

M1-M10 at `impl.md:229-245` (configs `D:/tmp/ks-orch/p2-2.2-timers/cfg/manual-*.ron`, loaded OK in `cfgcheck-output.txt`).
Ignored tests: the 5 existing `manual:` tests unchanged, plus `bench::latency_bench` with a `bench:` reason (`impl.md:159`).

## Invariants in play

- **R1** callback outcomes `keyboard_hook_callback.rs:318-356`; sentinel before state `:188-190` vs `with_main` `:203`; no new `log::` in the callback file; under-borrow work: `on_key`, `start_timers` -> `GestureTimers::start` -> one `SetWaitableTimer` (`main_thread.rs:254-266`); hook-path Vec pushes (Fact 30); `GroupKey` clone `gestures.rs:215-221`.
- **R2** no new `SendInput` builder; menu mask builder `keyboard_hook_callback.rs:258-272` unchanged in shape; Hold fast actions now run from `drain` -> `carry_out` on the main thread outside a callback (`main_thread.rs:491-493`).
- **R3** `MAIN` thread-local, not shared (ruling Q1 `rulings.md:63-65`); `Mutex<Engine>` gone (Fact 20); `switch_enabled` needs `&mut MainState` (`keyboard_hook.rs:123`, OQ5 `cut.md:489-491`).
- **R4** `enabled` `swap(AcqRel)` / `load(Acquire)` (`keyboard_hook.rs:113`, `:124`); bench-only atomics per `impl.md:152`; added `src` lines contain no `Relaxed`, `SeqCst` or `static mut`.
- **R6** `MainThreadError` Display `main_thread.rs:294-302`; `TimerSlot` Debug only (`mod.rs:302`).
- **R7** `deferred_group` order `winner.rs:127`; `sort_by_specificity` unchanged (`mod.rs:376-378`).
- **R8** presses keyed by the event vk (`gestures.rs:173`, `:306`); group identity `KeyCombination ==` (`gestures.rs:186`).
- **R10** `App::run` order (Fact 37); `WM_QUIT` in `pump` (`main_thread.rs:549-551`); `WAIT_FAILED` (`:470-472`); MainThread dropped before `uninstall` (deviation 7, `cut.md:513-514`).
- **Timers** KS-11 (tokens, generation; `mod.rs:294-339`, `:437-439`), KS-25 (one press `gestures.rs:1-38`), KS-33 (no thread; scan), R-7 hold-timer half closed; residual config-watcher half (`impl.md:271`).
- **Matching/suppression** key-up rule unchanged `mod.rs:759-760`; repeats never reach `press`/`other_key_down` (`mod.rs:554`).
- **Owner decisions** another key-down cancels a passing pending gesture (`gestures.rs:296-334`); waiting Tap fires on another key-down, swallowed groups too (OQ2 `rulings.md:49-54`); M2 waiting-Tap focus/`when:` declared (`known-issues.md:143`); M3 mouse click does not cancel (row 67, `known-issues.md:135`).

## Open points from impl.md

1. Tester: KS-11 repro replay at `7cb3521`; OQ1 `QS_ALLEVENTS` mutation; bench on `611c83f` and HEAD in its own worktree, R8 with `KS_BENCH_LOAD=1` (`impl.md:270`).
2. Residual R-7 (2.1): config watcher `update_bindings -> cancel_actions` can post a wake to an ended executor at exit (`impl.md:271`).
3. Follow-ups: `config_watcher.rs:126` `.expect("spawn config-watcher thread")`; executor MWMO pin (3.4); re-entry refusal counter (3.4); loader warning for unequal Tap/Hold terms (3.4); `KBDLLHOOKSTRUCT.time` as event clock (`impl.md:272`).
4. Pre-existing rustdoc warnings: `load`/`reload` link private `ConfigError::Missing`; unclosed `raw` tag (`impl.md:273`).
5. Uncommitted R4 experiments in `D:/tmp/ks-orch/p2-2.2-timers/scratch-experiment.md`; no lane worktree left (`impl.md:274`).
6. Deviations from the cut, 11 items (`cut.md:497-525`); the orchestrator updates CLAUDE.md principle 5 and agent files after merge (`impl.md:227`).
7. Developer ran only touched modules: 524 passed, 0 failed, 3 ignored; full suite left to the tester (`impl.md:158`).

## Not covered

- `critique.md` not read; the remarks' resolutions are taken from `impl.md:121-134` and `cut.md:398-495`.
- Not read: `src/bench.rs` bodies (only outline and the env/ignore lines), `src/engine/properties.rs` beyond `:12`, `:180`,
  gestures unit tests `gestures.rs:456-954`, engine tests `mod.rs:776-1201` bodies, most migrated tests in
  `keyboard_hook.rs:235-558` and `keyboard_hook_callback.rs:473-760`, `app.rs:344-404` bodies, `testing.rs` beyond `:400-449`.
- No doc text was read (README, CHANGELOG, book pages): only hunk ranges (Fact 43).
- No command was run besides read-only `git`/`grep`; no test, gate, bench, mutation or repro was executed for this pack.
- `src/platform/timers.rs` (`WaitableTimer::set`/`cancel`/`handle`/`is_high_resolution`) was not read; behaviour of `set` on error is as the cut states (`cut.md:89`), UNVERIFIED here.
