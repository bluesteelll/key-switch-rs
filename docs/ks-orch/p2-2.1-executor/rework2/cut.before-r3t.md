# Cut: p2-2.1-executor — roadmap 2.1 Action executor (KS-08; gaps "No cancellation for macro workers"; KS-33 Sequence half) + Phase-1 leftovers

Branch `feat/action-executor` from `master` @ `7d757a1` (re-checked: `.git/refs/heads/master` = `7d757a1d…`, unchanged since round 1). Work tree `D:/key-switch-rs/key-switch-rs`. All `path:line` below are on that tree unless a SHA is named. Binding inputs: `critique.md` (round 1) and `rulings.md` (2026-10-18); where they differ from round 1 of this cut, the rulings win and this text follows them.

## Verdict
CUT READY

## Revision after rulings
What changed against round 1, and why. Section names are the ones below.

| # | Source | Change | Sections rewritten |
|---|---|---|---|
| V1 | Ruling R1 | Macros run concurrently on the one executor thread as **resumable runs**. A `Delay` stores a deadline and returns to the loop; one thread timer (`SetTimer(None, 0, ms, None)`, re-armed for the nearest deadline, killed when no run waits) wakes the `GetMessageW` loop. No `MsgWaitForMultipleObjectsEx`, no `thread::sleep`, no blocking wait. The pure run list lives in new `src/core/executor/runs.rs` (fake clock tests). New gauge `active_runs` makes "a cancelled run is gone" observable. The Breaking entry for macro order is removed; the 100 ms promise now says "a running step finishes first". | D2, D5, D6 (new content), D7, D9, type sketch, thread map, Win32 table, touch set, X1/X2, X7, acceptance, M5, risks, CHANGELOG |
| V2 | Ruling R2, critique I1 | `Window(Minimize/Maximize/Restore)` use `ShowWindowAsync`; a failure is logged on the executor. Every window op has a non-blocking form (`Close` already posts), so no op is left with a declared blocking cost. Two new commits: X3 (red scan) and X4 (fix). Manual check M11 (hung window). | D13 (new), Win32 table, commits X3/X4, M11, CHANGELOG, risks |
| V3 | Ruling R3, critique M4, Q7 | Executor death is visible: `Submitted::Unavailable` + new counter `executor_unavailable`; the dying executor counts the jobs still queued; a panic of the loop itself is caught and logged as a death. `Closed` stays silent only for a requested stop / no executor attached. | D3, D14 (new), type sketch, X1/X2 tests, CHANGELOG, errors.md rows |
| V4 | Critique M1 | The no-log scan (now X11) covers every hook-path function this lane adds or changes, in one commented list. | X11, acceptance |
| V5 | Critique M2 | L7 split into seam (green) → red by assertion (L8) → fix (L9); old L9/L10 become L10/L11. The `.execute()` scan lands alone (X5), red by assertion, before the compile-red tests (X7) and the removal (X8). The new no-sleep scan (X6) also lands alone. | D12, commits |
| V6 | Critique M3, Q2 ruling | Touch set gains every config page and README line this lane falsifies; non-config pages go to the Phase-2 book sync and are listed in `impl.md`. | touch set, commits X4/X8 |
| V7 | Critique M5 | A Hold test drives the `hold-timer` → `carry_out` → executor path under `GLOBAL_HOOK` with a `FakeRunner`; nothing is installed or injected. | X7, acceptance |
| V8 | Critic open questions 1-3 | Bounded join recorded as a declared deviation; L2 drops the Win+L example and validates its RON with `cfgcheck`; the 5 ms enqueue rule is "every one of 20, one whole-batch retry, a flake goes to the orchestrator". | Goal, D8, L2, X1-10, X7 |
| V9 | Rulings Q1-Q7 | Q1 accepted as a declared deviation; Q2 resolved into the touch set; **Q3 removed** (superseded by R1); Q4-Q6 accepted; Q7 replaced by R3. New non-blocking questions N1-N4. | Questions |

Commit renumbering: old L7 → L7 + L8, old L8 → L9, old L9 → L10, old L10 → L11; old X1, X2 unchanged; new X3, X4 (R2); old X3 → X5 + X7, new X6; old X4 → X8; old X5/X6 → X9/X10; old X7/X8 → X11/X12.

## Spec re-located
| Spec reference (book) | Current file:line | Status | Note |
|---|---|---|---|
| roadmap 2.1 Why: `src/hook/keyboard_hook_callback.rs:231` @7cb3521 (Immediate `.execute()`) | `src/hook/keyboard_hook_callback.rs:219` (`action.execute()` inside `carry_out` :203-221) | moved | the only `.execute()` left under `src/hook` (grep) |
| `src/hook/tap_state.rs:83`, `:116`, `:148`; `chord_state.rs:100`; `sequence_state.rs:86`, `:106` @7cb3521 | files deleted; the engine returns fires in `Decision.fire` (`src/engine/mod.rs:477`, `:523`, `:551`, `:570`, `:631`, `:369`) | gone | Workstream A; engine executes nothing |
| Status after Phase 0: `carry_out` `keyboard_hook_callback.rs:185-195` @1dfd80a | `src/hook/keyboard_hook_callback.rs:199-221` | moved | the plug-in point |
| Hold action on `hold-timer` thread | `start_hold_timer` `keyboard_hook_callback.rs:290-304`, its `carry_out` at `:297` | moved | stays a thread (2.2) |
| test seams for the Hold test (critique M5) | `GLOBAL_HOOK` `keyboard_hook_callback.rs:412`; `call_keyboard_callback` `:436-455`; `#[cfg(test)] send_menu_mask` only counts `:267-279` | unchanged | nothing installed or injected under test |
| KS-08 `src/core/windows_actions.rs:123-124` @7cb3521 | `BindAction::execute` `src/core/windows_actions.rs:119-143`; Launch/OpenUrl arms `:138-139`; Sequence arm `:134-137` | moved | |
| `launch_process` `:205-219`, `shell_open` `:221-247` @7cb3521 | `windows_actions.rs:243-257`, `:259-282` (`ShellExecuteW` `:266-273`) | moved | no `CoInitializeEx` anywhere in `src` (grep) |
| "hook callback must return fast" `windows_actions.rs:28-30` @7cb3521 | `windows_actions.rs:28-34` (Sequence variant doc) | unchanged | rewritten in X8 |
| `src/core/app.rs:91`, `:125-131` @7cb3521 | install `app.rs:157`; loop `:205-211`; uninstall `:213-214`; `run` `:137-217` | moved | |
| gaps "No cancellation": `src/data/sequence.rs:114-125` @7cb3521 | `spawn_sequence` `sequence.rs:111-122` (`thread::spawn` `:116`); `Delay` = `thread::sleep` `:129` | moved | |
| gaps:166 (KS-33) `sequence.rs:111-122` @20af28d | same | unchanged | removed by X8 |
| critique I1: `sequence.rs:273-283` (`ShowWindow`) | `apply_window_op` `src/data/sequence.rs:264-291`: `ShowWindow` `:276` (Minimize), `:279` (Maximize), `:282` (Restore); `Close` = `PostMessageW` `:285` | unchanged | R2 target |
| `README.md:339-342` @7cb3521 (parallel macros) | `README.md:528-531` | moved | |
| README lines this lane falsifies (critique M3) | `:89`, `:116` (swch on/off), `:125` (Hold bullet), `:415-418` (Hold timer), `:488-492`, `:514`, `:518` (Window row), `:528-531`, `:704` (`swch off` "a running macro finishes"), `:769-773`, `:811`, `:837` | unchanged | |
| `default_config.ron:158` @7cb3521 (`OpenUrl` Win+Alt+G) | `src/config/default_config.ron:261` (commented out); Sequence `:104`; Window `:142-146`; Delay `:147` | moved | |
| config pages (critique M3, ruling Q2) | `book/src/config/README.md:55`; `example.md:99-100`, `:186-189`, `:206-207`, `:221-223`, `:237`; `kinds.md:77`, `:142`; `triggers-conditions.md:41`, `:75`, `:77`, `:79`, `:123`; `matching.md:150`; `known-issues.md` rows 26 `:64`, 28 `:72`, 29 `:73`, 36 `:80`, 60 `:128`; `actions.md` (sections in the touch set); `errors.md` §18.6 `:177-202` | unchanged | |
| logging doc "action failures still reported [on the hook thread] until roadmap item 2.1" | `src/logging/mod.rs:11-14` | unchanged | hook-path log sites: `windows_actions.rs:238` (PostMessage), `src/system/system_function.rs:58`, `:77` (SwitchLanguage*) |
| existing hook no-log scan and its helpers | `logging/mod.rs:231-249` (`print_welcome_lines`), `:253-261` (`names_the_log_crate`), `:268-329` (`only_the_startup_banner_prints_outside_src_bin`, whole non-test `keyboard_hook_callback.rs`) | unchanged | X11 extends it |
| counters (1.7) | `src/logging/counters.rs:13` (SLOTS), `:17-48`, `:51-82`, `:92-101`, `:104-109`; reporter `src/logging/writer.rs:224-229` | unchanged | |
| 1.1 review M2: `CHANGELOG.md:242` | `CHANGELOG.md:454` (123 chars; paragraph `:447-456`) | moved | |
| 1.3 results P2: Breaking `block_default` `CHANGELOG.md:132-151` | `CHANGELOG.md:256-276` | moved | |
| 1.3 results P1: sub-bullet "A `Tap` on `Alt` or `Win`" `CHANGELOG.md:149` | `CHANGELOG.md:272-274` | moved | README wording `README.md:456-461` |
| 1.2 results m1: `loader.rs:353-360` vs `:373-379` | warning pushed `src/config/loader.rs:353-357`; KS-18 skip `:373-379` | unchanged | |
| 1.6 review m3: `loader.rs:242-250` (doc), `:286-288` (log) | doc `loader.rs:249-250`; `log::warn!` `:286-288` | moved / unchanged | |
| 1.5 results m1: `main.rs:46-48` | `src/main.rs:64-66` | moved | |
| `console_ctrl_handler` unsafe `PostThreadMessageW` | `src/core/app.rs:248-250` | n/a | no `// SAFETY:` |
| 1.7 review m1: `logging/mod.rs:104-111`, `writer.rs:174-176` | `init` `src/logging/mod.rs:108-122` (`writer::spawn` `:109` before `set_logger` `:110`); `writer.rs:174-177` | unchanged | |
| 1.7 review m2: `mod.rs:124-127` vs `:135-136` | same | unchanged | |
| roadmap 2.4 `is_installed` `keyboard_hook.rs:137-139` | `src/hook/keyboard_hook.rs:217-219` | moved | not this lane |

Defect still present on master: yes (`keyboard_hook_callback.rs:219` runs every action on the hook thread; `sequence.rs:116` spawns a thread per macro; `sequence.rs:129` sleeps uninterruptibly; `sequence.rs:276-282` call the synchronous `ShowWindow`).

## Goal and non-goals
**After the lane.**
- Only `PressKey`, `Media`, `ToggleCapsLock`, `SwitchLanguage`, `SwitchLanguageBackward`, `PostMessage` (and `DoNothing`) run on the firing thread (hook thread, or a Hold's `hold-timer` thread), before the hook returns.
- `Launch`, `OpenUrl`, `Sequence` are queued (bounded, 32) for one `action-executor` thread: STA COM (`COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE`, balanced `CoUninitialize`), own `GetMessageW` loop, injectable `Runner`.
- Macros keep running side by side, as one-thread-per-macro did, without a thread per run: each `Sequence` is a run that goes up to its next `Delay`, stores a deadline and yields; one thread timer wakes the loop for the nearest deadline. A `Delay` holds up no other job. Only a step that blocks inside Windows (`Launch`, `OpenUrl`, a `ShellExecuteW` dialog) holds up the others while it runs.
- `Window(Minimize/Maximize/Restore)` never wait for the target window (`ShowWindowAsync`).
- An epoch change (`swch off`/`on` real change, reload, exit) drops every queued job and every run of the old epoch at the executor's next wake (well under 100 ms; a step already running finishes first), and no further step of such a run starts.
- `spawn_sequence` and its panicking `thread::spawn` are gone.
- Overflow drops the job and bumps `executor_jobs_dropped`; an executor that ended on its own makes every later slow action bump `executor_unavailable`; the longest keyboard callback is recorded and reported (`longest_callback_us`); the hook path calls no `log::` (failed posts counted in `post_message_failed`).
- Exit: hooks removed, then the executor stopped and joined, bounded by 2 s, before `logging::flush` (`main.rs:39`).
- Phase-1 leftovers fixed in separate commits.

**Declared deviations from the roadmap/brief text (accepted by the rulings).**
1. Q1: a job carries the `BindAction` by value, not `Arc<BindAction>` (no allocation on the hook path; an `Arc` in `Binding` is item 3.5's).
2. Critic open question 1: exit joins with a 2 s bound; on timeout the thread is detached with `[WARN] the action executor did not stop within 2 s; exiting without it` (D8).

**Non-goals.** Hold timers stay threads (2.2). No watchdog / `HookHealth` (2.4). No reset on reload/unlock beyond the epoch (2.3). `Launch`/`OpenUrl` stay elevated (KS-09, 3.3). No `Binding.action: Arc` / hot-path allocation work (3.5). No new actions (new feature 1/3). No engine change. No cap on the number of active runs (N1).

**Later items that depend on this:** 2.2 (removes `hold-timer`, closing residual race R-7), 2.3 (`reset_transient_state` calls the cancel added here), 2.4 (can read `longest_callback`, `active_runs`, `executor_unavailable`), 3.3 (de-elevation lives in the executor's runner), new features 1 and 3 (must pick a placement in `BindAction::as_fast`'s and `Win32Runner::action`'s exhaustive matches).

## Decisions

### D1: Fast/slow split is a property of the action variant
**What.** New `pub enum FastAction` (Copy: `SwitchLanguage`, `SwitchLanguageBackward`, `ToggleCapsLock`, `PressKey(VIRTUAL_KEY)`, `PostMessage { msg, wparam, lparam }`, `Media(MediaKey)`, `DoNothing`) with `pub fn run(self)`, and `BindAction::as_fast(&self) -> Option<FastAction>`: `Some` for those, `None` for `Launch`, `OpenUrl`, `Sequence`. Every arm spelled out (no `_`), like `repeats_by_default` (`windows_actions.rs:166-186`), so a new action does not compile until its placement is decided. `BindAction::execute` (`:119-143`) and `Binding::execute` (`binding.rs:248-250`, no callers) are removed: the fast actions are reached only through `FastAction::run`, so `rg "\.execute\(\)" src/hook` is empty. `SystemFunction::execute` stays (reached from `FastAction::run` for the two language switches and CapsLock only).
**Why.** The brief/roadmap list (`roadmap.md:446`): fast actions must stay in order with the keys after them, so they run before the hook returns. A typed `FastAction` makes "run a slow action inline" unrepresentable. Microsoft: "If the application must use low level hooks, it should run the hooks on a dedicated thread that passes the work off to a worker thread and then immediately returns" (LowLevelKeyboardProc).
**Rejected.** Everything on the executor (reorders `PressKey("Left")` against the next typed key; spec forbids). A `runs_inline() -> bool` + `run_inline(&self)` with no-op slow arms (silently does nothing if misused). A per-binding config flag (config change, not asked).
**Cost.** A 7-variant mirror enum. `LockWorkstation`/`ShowDesktop`/`TaskManager` are not `BindAction` variants (`system_function.rs:18-25` vs `windows_actions.rs:17-51`), so "system functions that spawn or call the shell" have no path today; new feature 1 must map them to `None`.

### D2: One `action-executor` thread: STA COM + one `GetMessageW` loop, woken by a posted `WAKE` and by one thread timer (rewritten for ruling R1)
**What.** `Executor::start` spawns `action-executor` (`thread::Builder`, error returned, no panic). Thread prologue: (1) `Apartment::enter_sta()` = `CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)`; `hr.is_ok()` (S_OK or S_FALSE) arms a guard whose `Drop` calls `CoUninitialize` on this thread; a failure logs `[WARN] could not initialise COM on the action executor ({hr}); OpenUrl runs without it` and the executor runs without COM, as every action did before; (2) `PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE)` creates the queue; (3) `Ok(GetCurrentThreadId())` on a `sync_channel(1)` handshake; `start` waits ≤ `START_TIMEOUT` (5 s), so no `PostThreadMessageW` can precede the queue.
**Loop** (executor thread; the whole body runs inside one `catch_unwind`, D14):
1. **Jobs, oldest first.** Repeat: `stopping` set → end (`Stopped`); `try_recv` → `Ok(job)` → `runs.start(job)` then `runs.poll()` (so a due run is late by at most one job); `Empty` → leave; `Disconnected` (every handle dropped, only after `Executor` dropped, which sets `stopping` first) → end (`Stopped`).
2. **Runs.** `runs.poll()` → `Next` (D6); store `active_runs` (Release); `stopping` set → end.
3. **Timer.** `timer.arm(next, Instant::now())` (D6): `Due` → back to 1 without waiting; `Failed(e)` → `[ERROR] could not start the Delay timer ({e}); {n} macros waiting in a Delay stop` with `n = runs.drop_timed()`; `Idle`/`Waiting` → 4.
4. **Wait.** `GetMessageW(&mut msg, None, 0, 0)`: `.0 == -1` → `[ERROR] the action executor's message loop failed ({error}) and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts`, end (`Died`); `0` (`WM_QUIT`, which nothing of ours posts) → `[ERROR] the action executor received WM_QUIT and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts`, end (`Died`); null `hwnd` + `WAKE` → back to 1; null `hwnd` + `WM_TIMER` + `lParam == 0` → `timer.fired(msg.wParam.0)` (ours: killed and forgotten; anything else: a stale id of ours, nothing to dispatch), back to 1; anything else (COM's hidden-window messages, a foreign `TimerProc`'s `WM_TIMER` with non-zero `lParam`) → `TranslateMessage` + `DispatchMessageW`, back to 1.
**Epilogue:** `timer.kill()`; `active_runs` = 0; drop the runs; on `Died` drain the queue with `try_recv`, bumping `executor_unavailable` per job (D14); drop the receiver; drop the runner; drop the `Apartment` (CoUninitialize); send on `done`.
**Producer side.** `try_send(job)` first, then `PostThreadMessageW(tid, WAKE, 0, 0)`; result ignored. A lost or failed wake never loses a job: it waits in the channel until the next wake (the executor drains before every `GetMessageW`). The post follows the send, so a drain that misses the job is followed by a queued `WAKE` (no lost wakeup).
**Why.** Brief: "its own GetMessageW loop and single-threaded COM". Ruling R1: "The thread still waits in a `GetMessageW` loop … `thread::sleep` and a blocking pumped wait are not used." ShellExecute: "COM should be initialized before ShellExecute is called. Some Shell extensions require the COM single-threaded apartment (STA) type... `CoInitializeEx(NULL, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)`". CoInitializeEx: STA calls "arrive only at message-queue boundaries (when the PeekMessage or SendMessage function is called)" → the thread pumps, including while every run waits in a `Delay` (better than round 1, where a `Delay` pumped only through its own wait). PostThreadMessage: "The thread to which the message is posted must have created a message queue... call PeekMessage(&msg, NULL, WM_USER, WM_USER, PM_NOREMOVE)". WM_TIMER: "Posted to the installing thread's message queue when a timer expires. The message is posted by the GetMessage or PeekMessage function." `WM_QUIT` is not our stop signal because PostThreadMessage messages "will be lost" in a modal loop (a ShellExecuteW error dialog); the `stopping` flag + epoch cannot be lost. PostQuitMessage: "When the thread retrieves the WM_QUIT message from its message queue, it should exit its message loop" → a foreign `WM_QUIT` ends the executor (ruling R3).
**Rejected.** A thread per run (ruling R1: "without a thread per run"; KS-33). A blocking pumped `MsgWaitForMultipleObjectsEx` per `Delay` (round 1; ruling R1 forbids: it serialised every job behind a `Delay`). Waitable timer + `MsgWaitForMultipleObjectsEx` as the idle wait (not the `GetMessageW` loop the spec names). A thread pool (loses order; spec says one thread).
**Cost.** A step that blocks inside Windows (`Launch`'s `CreateProcessW`, `OpenUrl`'s `ShellExecuteW`, a dialog it shows) holds up every other job and run while it runs: the one remaining serialisation (ruling R1), in the CHANGELOG and README.

### D3: Bounded `std::sync::mpsc::sync_channel(QUEUE_CAPACITY = 32)`
**What.** `ExecutorHandle::submit(action, target) -> Submitted`: `epoch.load(Acquire)` → `try_send(Job)`: `Ok` → post `WAKE`, `Queued`; `Full` → `counters.bump(Counter::ExecutorJobsDropped)`, `Dropped`; `Disconnected` → `stopping.load(Acquire)` true → `Closed` (requested stop; silent, not counted), false → `counters.bump(Counter::ExecutorUnavailable)`, `Unavailable` (D14). `KeyboardHook::submit` returns `Closed` when no executor is attached (before attach at startup, after detach at exit). `capacity` 0 is treated as 1.
**Why.** A bounded `sync_channel` preallocates its slots, so `try_send` moves the job without allocating; the channel's waker mutex is taken only when a thread is registered on it (std `library/std/src/sync/mpmc/waker.rs:171-181`, `SyncWaker::notify` checks `is_empty` first), i.e. parked in `send`/`recv`, and nobody ever is (producers `try_send`, the executor `try_recv`s). Same queue the log writer uses (`writer.rs:59-77`). 32: 1 s of a `repeat: true` Sequence at the ~30 Hz repeat rate; bounded where the old code spawned unbounded threads.
**Rejected.** Unbounded `channel` (allocates per send, unbounded backlog). New crate (no dependency allowed).
**Cost.** The Full path drops the job on the firing thread (a free of a `Launch`'s strings).

### D4: `Job { action: BindAction, target: TargetWindow, epoch: u64 }` — action by value (declared deviation 1)
**What.** The `BindAction` is moved out of `Decision.fire` (`carry_out` iterates `fire` by value). `TargetWindow(isize)` (HWND is not `Send`) is `GetForegroundWindow()` captured on the firing thread for a `Sequence` only (`TargetWindow::for_action`); `TargetWindow::NONE` for `Launch`/`OpenUrl`, which never read it — so Launch/OpenUrl add no Win32 lookup to the hook path, and a Sequence keeps exactly the lookup `spawn_sequence` made there (`sequence.rs:114`). A run keeps the `Arc<Vec<SequenceStep>>` moved out of the job (no clone).
**Why.** The engine already hands over an owned clone (`engine/mod.rs:477`, `:291`); `Arc::new` per slow fire would add an allocation on the hook path for no sharing (one consumer). Ruling Q1: accepted.
**Rejected.** `Arc<BindAction>` (new allocation). `Binding.action: Arc<BindAction>` through the engine (engine-wide change; 3.5 scope).

### D5: Epoch cancellation
**What.** `Shared { epoch: AtomicU64, stopping: AtomicBool, active_runs: AtomicUsize }`. `ExecutorHandle::cancel_pending()`: `epoch.fetch_add(1, AcqRel)`, then post `WAKE`. `live(e) = !stopping.load(Acquire) && epoch.load(Acquire) == e` is checked: when a job is dequeued (stale → skipped, never started), at the start of every `poll` (stale runs dropped, waiting ones included), and before every step of a run (so a run in the middle of its steps stops before its next step even before the wake is handled). Bumped by: `KeyboardHook::set_enabled` after `with_engine` returns `Some(true)` (a real change, on or off; a repeated `on`/`off` changes nothing — `keyboard_hook.rs:146-155`), `KeyboardHook::update_bindings` after its store (`:110-115`; the reload path `config_watcher.rs:162`; at startup no executor is attached yet, D8), `Executor::stop` (exit). Not on a foreground change. A step already running (`Launch`, `OpenUrl`, a step inside a macro) is not interrupted (Win32 offers no way); nothing after it of the old epoch runs.
**Why.** Brief: "A change of epoch (swch off, reload, exit) interrupts a Delay step within 100 ms"; gaps.md:147 "a cancellation flag that is checked between steps and inside long Delay steps, set on off and on reload"; ruling R1: "An epoch bump drops every run and every queued job of the old epoch at the next wake. The bump posts `WAKE`, so no further step of a cancelled run starts once the executor wakes."
**Rejected.** Bump under the engine lock (the epoch is not engine data; `set_enabled` docs `keyboard_hook.rs:142-145`). Per-binding tokens (not asked). Cancel only the running job (queued `OpenUrl`s would still open after `swch off`).
**Cost.** An action decided just before `swch off` but submitted after the bump runs (it was decided while on). Saving `config.ron` unchanged also stops the running macros.

### D6: Resumable runs: a `Delay` is a deadline, resumed by one thread timer (replaces round 1's pumped wait; ruling R1)
**What — the run list (`src/core/executor/runs.rs`, pure: no Win32, no thread, no clock; `now` and `live` are passed in).** `Runs { active: Vec<Run> }`, oldest first; `Run { steps: Arc<Vec<SequenceStep>>, next: usize, target, epoch, resume: Resume }`, `Resume = Now | At(Instant) | WhenCancelled`.
- `start(job)`: not `live(job.epoch)` → dropped; `Sequence(steps)` → appended with `resume: Now`; every other action → `runner.action(&job.action)` at once.
- `poll()`: (a) drop every run that is not `live`; (b) in age order, each run that is due (`Now`, or `At(t)` with `now() >= t`) is advanced **once**: step after step — before each, `live` check (not live → run removed); `Delay(ms)` → `resume = now().checked_add(Duration::from_millis(ms))` → `At(t)` / `None` → `WhenCancelled`, and the run yields; any other step → `runner.step(step, target)`; past the last step → run removed; (c) returns `Polled { next, panics }`, `next = At(min t over runs in At)` or `Idle` (runs in `WhenCancelled` need no timer).
- Consequences: a new run "starts at once and runs its steps until the first `Delay` or its end" (ruling R1) in the poll that follows its `start`; two macros fired together interleave at their `Delay` steps, oldest first, and never inside a stretch of steps between two Delays (N2); a `Delay(0)` yields to the other runs and jobs, and the loop polls again without waiting (`Due`); the deadline is taken when the `Delay` step is reached, after any slow step before it.
**What — the timer (`DelayTimer`, executor thread only).** `armed: Option<(id, for: Instant)>`. `arm(next, now)`: `Idle` → kill the armed timer → `Idle`; `At(t)`, `t <= now` → kill → `Due`; `At(t)` equal to the armed `for` → `Waiting`; else kill the old one, `SetTimer(None, 0, timer_ms(now, t), None)` → id (0 → `Failed(Error::from_thread())`) → `Waiting`. `timer_ms(now, at)` = remaining time rounded up to whole ms, clamped to `1..=USER_TIMER_MAXIMUM` (0x7FFF_FFFF ≤ `u32::MAX - 1`). `fired(id)`: our id → `KillTimer(None, id)`, forget it (the next `arm` re-arms from `Instant`, so an early, late, stale or periodic repeat `WM_TIMER` corrects itself). `kill()`: `KillTimer` if armed (result ignored).
**Granularity.** "Because a timer's accuracy depends on the system clock rate and how often the application retrieves messages from the message queue, the time-out value is only approximate" (About timers): about one tick of the system clock (about 15.6 ms at the default rate, the same as the old `thread::sleep`), plus whatever default coalescing Windows applies (size unverified, N3). `Instant` decides; the timer is only the wake-up. A `Delay` never ends early: a run advances only when `now() >= t`.
**Why.** Ruling R1 (deadline, `checked_add`, `None` = until cancelled, one thread timer re-armed for the nearest deadline and killed when no run waits, never `u32::MAX - 1` or more). Kill-then-create instead of replace-by-id: SetTimer documents "SetTimer can reuse timer IDs in the case where hWnd is NULL", so the id is never trusted across arms. The run list is pure so iteration, deadlines and epochs are tested with a fake clock (ruling R1: "all of them are real code under test"). It is not `engine` code: the engine decides key events and holds key state; a run calls the runner (side effects) on the executor thread.
**Rejected.** `MsgWaitForMultipleObjectsEx` per `Delay` (blocking; ruling). `SetCoalescableTimer(.., TIMERV_NO_COALESCING)` (ruling names `SetTimer`; "Do not use this value unless you are certain that the timer requires no coalescing"; kept as N3's fallback). One timer per run (more timers, same effect). Step-wise round robin across runs (contradicts "runs its steps until the first `Delay`"; splits a macro's `Text`/`Key` stretch, N2). Advancing a `Delay(0)` run again inside the same poll (one long chain of `Delay(0)` would starve other runs).
**Cost.** `active` is not bounded: a held `repeat: true` macro with a long `Delay` accumulates runs (about 100 bytes each; the old code started a thread each) — N1.

### D7: Injectable `Runner`; the executor owns runs, `Delay`, deadlines, epochs and the timer
**What.** `pub trait Runner: Send + 'static { fn action(&mut self, action: &BindAction); fn step(&mut self, step: &SequenceStep, target: TargetWindow); }` (unchanged, ruling R1). `action` gets every non-`Sequence` job (`Launch`/`OpenUrl`; a fast action only if one were ever submitted); `step` gets every step except `Delay`. `pub struct Win32Runner;` is the daemon's: `action` = exhaustive match without `_`: `Launch` → `launch_process`, `OpenUrl` → `shell_open`, `Sequence(_)` → nothing (walked by the executor, never passed here), the seven fast variants → `action.as_fast()` → `run()`; `step` → `sequence::run_step(step, target.hwnd())` (the renamed `execute_step`, `sequence.rs:124-135`, now `pub(crate)`). Tests use `core::executor::testing::FakeRunner`: records `Record { thread: ThreadId, name: Option<String>, at: Instant, ran: Ran }` on an `mpsc::Sender` first, then optionally blocks (`with_block`), waits on a gate (`with_gate(started, release)`), panics (`with_panic_on`) or calls `PostQuitMessage(0)` on its own thread (`with_quit_on`). Pure `runs` tests use a local recording runner and a fake clock.
**Why.** Brief: "takes an injectable job runner, so tests run fake jobs. No test executes a real action". Cancellation, `Delay`, deadlines and interleaving are real code under test.
**Rejected.** Runner receiving whole jobs (Delay and cancel would be fake in tests).

### D8: Lifecycle in `App::run` (main thread)
**What.** Order: `bind` pipe (`app.rs:145`) → `print_welcome` → Ctrl+C handler → **`Executor::start(Win32Runner, QUEUE_CAPACITY, &counters::COUNTERS)`** (failure → `Err("could not start the action executor: …")` → `[FATAL]`, before any hook) → `update_bindings` → **`hook.attach_executor(Some(handle))`** → `install` → `serve` → watcher → loop → `uninstall` → **detach (`attach_executor(None)`), then `stop()`**. A private guard `AttachedExecutor { hook, executor: Option<Executor> }` does detach+stop in `Drop`, covering every `?`/error return; the normal path calls its `stop()` explicitly right after `uninstall`. `Executor::stop(self) -> Stopped`: `stopping.store(true, Release)`, `cancel_pending()`, wait `done.recv_timeout(STOP_TIMEOUT = 2 s)`: `Ok`/`Disconnected` → `join()` → `Joined`; `Timeout` → drop the `JoinHandle` (thread detached; process exit ends it) → `TimedOut`, and the guard logs `[WARN] the action executor did not stop within 2 s; exiting without it` (declared deviation 2). No log line on a normal stop, so `daemon.log` still ends with `[INFO] Removing keyboard hook...` (Checkpoint 1 item 16). `logging::flush` stays in `main.rs:39`, after `run` returns.
**Why stop must follow uninstall.** The LL hook "is called by sending a message to the thread that installed the hook" (LowLevelKeyboardProc); while `main` blocks in `stop`, a `SendInput` by a macro step would wait on the unpumped hook thread (up to `LowLevelHooksTimeout` per event) and risk the silent unhook. After `uninstall` nothing waits on `main`.
**Why bounded.** A `ShellExecuteW` against a dead UNC path or a modal error dialog can hold the executor for many seconds; an unbounded join would hang `swch exit` with the single-instance mutex held.
**Rejected.** Unbounded `join`. `WM_QUIT` as stop (D2). Starting the executor lazily on the first slow action (would spawn on the hook thread).

### D9: A panic in one action or step does not kill the executor
**What.** `Runs` wraps every runner call in `std::panic::catch_unwind(AssertUnwindSafe(..))`. A panicking `action` ends that job; a panicking `step` removes its run (the macro stops, as its thread died before). `Polled.panics` / `start`'s return count them; the executor logs `[ERROR] an action panicked on the action executor; it keeps running` per panic. `Runner` docs: an implementation must stay usable after a panic in one call (`Win32Runner` is stateless).
**Why.** One thread carries every slow action; a dead executor turns every later `Launch`/`OpenUrl`/`Sequence` into `Unavailable`. CLAUDE.md's "never catch panics" is about the hook callback (phantom-blocked keys); the executor holds no key state. Profile has no `panic = "abort"` (`Cargo.toml`), so unwinding works.
**Rejected.** Restart-on-death supervisor (more state; R3 asks for visibility, not restart). Skipping only the panicking step (the old code stopped the macro).

### D10: Longest-callback gauge, reported only when it matters
**What.** `Counters` gains `longest_callback_ns: AtomicU64`: `record_callback(&self, took: Duration)` = `fetch_max(ns saturated to u64, AcqRel)`; `take(&self) -> Snapshot { counts: [u64; SLOTS], longest_callback: Duration }` swaps every slot and the gauge to 0 (`AcqRel`); `longest_callback(&self) -> Duration` (`Acquire`). `pub const SLOW_CALLBACK: Duration = 5 ms`. `summary(&Snapshot)` is `Some` when a count is non-zero **or** `longest_callback >= SLOW_CALLBACK`; the line appends ` longest_callback_us=<µs>`. Recorded in `keyboard_hook_callback` (main thread): `Instant::now()` after the remapping-off early return, `record_callback(started.elapsed())` after `decide_event` + `carry_out`, before either return; excludes `CallNextHookEx` (not our work) and the two early passes (they do nothing).
**Why.** Brief: "A counter records the longest callback duration and is reported by the log writer." 5 ms is the acceptance budget (`roadmap.md:453`, ruling Q4); normal callbacks are microseconds, so a report means a real stall and normal use adds no line per minute (1.7's noise rule). Two `Instant::now()` (QPC) and one atomic RMW per event; no allocation.
**Rejected.** Reporting the gauge every period (a line every minute while typing). A lifetime maximum (a past stall would be reported forever; 2.4's `HookHealth` can add that).

### D11: The hook path calls no `log::`
**What.** `Counter::PostMessageFailed` ("post_message_failed") and `Counters::bump_if_err<T, E>(&self, counter, result: &Result<T, E>)` (+ free fn on `COUNTERS`). `post_message_to_foreground` (`windows_actions.rs:228-241`) and `switch_language_forward/backward` (`system_function.rs:43-80`) count a failed `PostMessageW` instead of `log::error!`. Slow-action failures (`launch_process`, `shell_open`, step warnings) keep logging, now on the executor thread.
**Why.** Brief "(the hook path never calls log::)"; `logging/mod.rs:11-14` reserved hook-thread action logging "until roadmap item 2.1". Ruling Q6: losing the error text is accepted.

### D12: Phase-1 leftovers, one item per commit (L1-L11)
- L1/L2 CHANGELOG text only (L2 without the Win+L example; its RON validated through `cfgcheck`).
- L3/L4: the `MaskOnly` fallback warning is pushed only where the blocker is added: record `(hotkey, mode, condition, fallback_of: Option<usize>)` in the first loop (`loader.rs:346-362`) and push the warning in the second loop after the KS-18 skip (`:373-379`), once per binding index, keeping file order.
- L5/L6: `parse_ron_str` pushes `"config contains no bindings"` into `warnings` (`loader.rs:286-288`); `from_ron_str_with_hotkeys` logs it at the same level with the same text, so console and file output are unchanged.
- L7/L8/L9 (critique M2): L7 introduces the seam `fn start_logging<W>(install: impl FnOnce() -> bool, start_writer: impl FnOnce() -> io::Result<W>) -> Option<io::Result<W>>` with **today's order** (writer first, `None` when `install` fails); L8 adds the red test (assertion); L9 swaps the order (install first; `None` = writer never started) and fixes the `attach_file` doc (both failures: no writer; queue full for 1 s).
- L10: `main.rs:64-66` doc.
- L11: `app.rs:248-250` SAFETY comment.

### D13: `Window(...)` steps never wait for the target window (ruling R2, critique I1)
**What.** `apply_window_op` (`sequence.rs:264-291`): `Minimize`/`Maximize`/`Restore` call `ShowWindowAsync(target, SW_MINIMIZE / SW_MAXIMIZE / SW_RESTORE)`; a `false` result logs `[WARN] window step failed: ShowWindowAsync did not start {op:?}` (executor thread). `Close` stays `PostMessageW(WM_CLOSE)` (already asynchronous). The invalid-target skip (`:265-268`) stays. Every `WindowOp` has a non-blocking form, so no op is left with a declared blocking cost.
**Why.** ShowWindowAsync: "This function posts a show-window event to the message queue of the given window. An application can use this function to avoid becoming nonresponsive while waiting for a nonresponsive application to finish processing a show-window event." With one executor for every slow action, a synchronous `ShowWindow` on a Not Responding window would hold up every `Launch`, `OpenUrl` and macro, and no epoch change could release it (critique I1).
**Rejected.** Skip when `IsHungAppWindow` (a window can hang right after the check; the step would still block). Keep `ShowWindow` and document it (ruling R2).
**Cost.** The step no longer waits for the window to finish the change: a following step that depends on it (typing into a window just restored) needs a `Delay` after it. The README and template examples already have `Window(Restore), Delay(100)` (`README.md:498-499`, `default_config.ron:339-340`). `Changed` entry, not Breaking (N4). The return value now means "posted" (ShowWindow's meant the previous visibility and was ignored), so a failure is logged for the first time.

### D14: An executor that ended on its own is visible (ruling R3, critique M4)
**What.** The executor ends on its own on `WM_QUIT`, on `GetMessageW == -1`, or on a panic of its loop outside the per-call `catch_unwind` (the whole loop runs in one `catch_unwind`; caught → `[ERROR] the action executor panicked and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts`). Each logs one `[ERROR]` line (D2) and then, before it drops the receiver, bumps `executor_unavailable` for every job still queued. From then on every `submit` that sees `Disconnected` while `stopping` is false bumps `executor_unavailable` and returns `Unavailable`; the log writer's minute report shows it. `Closed` during a requested stop, or with no executor attached, stays silent and uncounted.
**Why.** Ruling R3. Without it, after a shell extension's `PostQuitMessage` on the executor thread, hotkeys would silently do nothing while `swch status` says `running` (critique M4).
**Ordering argument.** A mid-session death never sets `stopping` (only `Executor::stop` does, after detach), so every such submit reads `false` and is counted: exact. At exit the handle is detached before `stop()`, so only a thread that loaded the handle before detach (a `hold-timer`, residual race R-7) can see `Disconnected`; the lane does not rely on std's channel internals for a happens-before edge to `stopping`, so such a thread could read a stale `false` and add one count to the last counters line. Accepted with R-7 until 2.2. A job sent between the dying executor's final drain and its receiver drop is lost uncounted (a window of a few instructions, only at the moment of death).
**Rejected.** Ignoring `WM_QUIT` (ruling R3; PostQuitMessage: the thread "should exit its message loop"). A `swch status` field (IPC change, not asked; 2.4 can add it from the counter).

## Win32 API choices
| Purpose | `windows` path (0.62.2, read in the registry source) | Feature (Cargo.toml:41-53) | Thread | Errors | Doc |
|---|---|---|---|---|---|
| STA COM for ShellExecute | `windows::Win32::System::Com::CoInitializeEx(None, COINIT_APARTMENTTHREADED \| COINIT_DISABLE_OLE1DDE) -> HRESULT` (COINIT has `BitOr`) | `Win32_System_Com` present (:48) | executor, first call | `hr.is_ok()` → balance; else `[WARN]`, run without COM | https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex ("each successful call..., including any call that returns S_FALSE, must be balanced by a corresponding call to CoUninitialize") |
| balance | `windows::Win32::System::Com::CoUninitialize()` | present | executor, last (guard `Drop`) | none | https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize |
| test probe | `windows::Win32::System::Com::CoGetApartmentType(*mut APTTYPE, *mut APTTYPEQUALIFIER) -> Result<()>`; `APTTYPE_STA` | present | test threads only | `Err(CO_E_NOTINITIALIZED)` = no apartment | https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cogetapartmenttype |
| create queue | `windows::Win32::UI::WindowsAndMessaging::PeekMessageW(&mut MSG, None, WM_USER, WM_USER, PM_NOREMOVE) -> BOOL` (mod.rs:1842) | `Win32_UI_WindowsAndMessaging` present (:44) | executor, before handshake | result ignored | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew (Remarks) |
| own tid | `windows::Win32::System::Threading::GetCurrentThreadId() -> u32` | `Win32_System_Threading` present (:51) | executor | cannot fail | https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentthreadid |
| the loop | `GetMessageW(*mut MSG, Option<HWND>, 0, 0) -> BOOL` (mod.rs:997); `TranslateMessage(*const MSG) -> BOOL` (:2402); `DispatchMessageW(*const MSG) -> LRESULT` (:575) | present | executor | `.0 == -1` → `[ERROR]`, end; `0` (`WM_QUIT`) → `[ERROR]`, end | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew |
| wake | `PostThreadMessageW(u32, u32, WPARAM, LPARAM) -> Result<()>` (:1872) with `WM_APP` (:6897) | present | hook, `hold-timer`, IPC worker, `config-watcher`, main | ignored (`ERROR_NOT_ENOUGH_QUOTA`, `ERROR_INVALID_THREAD_ID`): job stays queued | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew ("returns without waiting"; 10,000 posted-message limit; lost in a modal loop) |
| Delay timer, arm | `SetTimer(Option<HWND>, usize, u32, TIMERPROC) -> usize` (:2238) called as `SetTimer(None, 0, timer_ms(..), None)` | present | executor only (the timer belongs to it) | `0` → `Error::from_thread()` (windows-result 0.4.1 `error.rs:118`) → `[ERROR]`, waiting-on-deadline runs end | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-settimer ("If lpTimerFunc is NULL, the system posts a WM_TIMER message to the application queue. The hwnd member of the message's MSG structure contains the value of the hWnd parameter."; "If uElapse is greater than USER_TIMER_MAXIMUM (0x7FFFFFFF), the timeout is set to USER_TIMER_MAXIMUM"; "If the call is not intended to replace an existing timer, nIDEvent should be 0 if the hWnd is NULL"; "SetTimer can reuse timer IDs in the case where hWnd is NULL") |
| Delay timer, kill | `KillTimer(Option<HWND>, usize) -> Result<()>` (:1365) as `KillTimer(None, id)` | present | executor only | ignored | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-killtimer ("If the application calls SetTimer with hWnd set to NULL, this parameter must be the timer identifier returned by SetTimer"; "does not remove WM_TIMER messages already posted to the message queue") |
| timer delivery | `WM_TIMER` (:7128, 0x0113), `USER_TIMER_MAXIMUM` (:6677, 0x7FFF_FFFF), `USER_TIMER_MINIMUM` (:6678, 10) | present | executor | n/a | https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-timer ("Posted to the installing thread's message queue when a timer expires. The message is posted by the GetMessage or PeekMessage function."; "a low-priority message"); https://learn.microsoft.com/en-us/windows/win32/winmsg/about-timers ("Each time the specified interval … elapses, the system notifies"; "the time-out value is only approximate"; "applications should destroy timers that are no longer necessary") |
| Window steps (R2) | `ShowWindowAsync(HWND, SHOW_WINDOW_CMD) -> BOOL` (:2347) with `SW_MINIMIZE` (:6413), `SW_MAXIMIZE`, `SW_RESTORE` | present | executor | `!as_bool()` → `[WARN] window step failed: …` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync ("If the operation was successfully started, the return value is nonzero."; "posts a show-window event to the message queue of the given window") |
| `Window(Close)` (unchanged) | `PostMessageW(Some(target), WM_CLOSE, ..)` (`sequence.rs:285`) | present | executor | `[WARN] window close PostMessage failed` (unchanged) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew |
| test only: foreign quit | `PostQuitMessage(i32)` (:1862) inside `FakeRunner::action` on the executor thread of a test | present | test executor thread | none | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage ("posts a WM_QUIT message to the thread's message queue and returns immediately"; "it should exit its message loop") |
| Sequence target | `GetForegroundWindow() -> HWND` (as `sequence.rs:114`) | present | hook / `hold-timer`, Sequence only | null → `TargetWindow(0)`; `apply_window_op` skips invalid (`sequence.rs:264-268`) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow |
| OpenUrl (moved) | `windows::Win32::UI::Shell::ShellExecuteW` (unchanged body `windows_actions.rs:259-282`) | `Win32_UI_Shell` present | executor | ≤ 32 → `[ERROR]` (executor) | https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew |
| memory ordering argument | — | — | — | — | https://learn.microsoft.com/en-us/windows/win32/sync/synchronization-and-multiprocessor-issues |
| hook budget | — | — | — | — | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc ("on Windows 7 and later, the hook is silently removed"; 1000 ms max since 1709) |
| thread-id lifetime (R-7) | — | — | — | — | https://learn.microsoft.com/en-us/windows/win32/procthread/thread-handles-and-identifiers |

Removed against round 1: `MsgWaitForMultipleObjectsEx`, `WAIT_OBJECT_0`/`WAIT_TIMEOUT`/`WAIT_FAILED` and the `PeekMessageW(PM_REMOVE)` drain (ruling R1). Not used: `SetUserObjectInformationW(UOI_TIMERPROC_EXCEPTION_SUPPRESSION)`, which SetTimer's Remarks recommend; it "Sets the exception handling behavior when calling TimerProc" (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setuserobjectinformationw), the lane passes no `TimerProc` and never dispatches its own `WM_TIMER`, and the call would change a process-wide setting for foreign code on the thread (out of scope; follow-up note in Risks).

`Cargo.toml` unchanged (every feature present; no dependency). `HWND`/`HANDLE` are not `Send` in 0.62: `TargetWindow` carries an `isize`, as `sequence.rs:106-117` does. No new `SendInput` path; every macro step keeps its builder and `injected_sentinel()` (`sequence.rs:166-179`, `:232-245`); `LLKHF_INJECTED`, scan codes and the extended flag are untouched (KS-36 is 3.1's). UIPI/elevation unchanged: the executor is a thread of the same elevated process; `ShowWindowAsync` to a lower-integrity window is allowed as `ShowWindow` was; `Launch`/`OpenUrl` children still inherit the admin token (KS-09, 3.3). Pipe DACL untouched.

## Thread map (new code paths)
| Code path | Thread |
|---|---|
| `Executor::start`, `AttachedExecutor` attach/detach/stop, `[WARN]` on stop timeout | main |
| executor body: COM, queue, loop, `Runs::start`/`poll`, `DelayTimer` (`SetTimer`/`KillTimer`/`WM_TIMER`), `Runner` calls (`ShowWindowAsync` included), slow-action and executor `log::`, the death drain | `action-executor` |
| `carry_out` → `dispatch` → `FastAction::run` / `KeyboardHook::submit` → `ExecutorHandle::submit` | main (hook callback) or `hold-timer` |
| `cancel_pending` | IPC connection worker (`set_enabled`), `config-watcher` (`update_bindings`), main (`stop`) |
| `active_runs` | read by tests only; nothing in the daemon reads it yet (2.4 may) |
| `record_callback` | main (hook callback) |
| counter report | `log-writer` |

No timer is installed or moved on the main thread (2.2's job); the Delay timer is created, killed and pumped on the executor thread that owns it.

Shared state and orderings: `KeyboardHook.executor: ArcSwapOption<ExecutorHandle>` (stores on main only; one `load` per slow fire, lock-free like the `bindings.load()` every event already does, `keyboard_hook.rs:120-122`); `Shared.epoch` (`fetch_add AcqRel` / `load Acquire`); `Shared.stopping` (`store Release` / `load Acquire`); `Shared.active_runs` (`store Release` on the executor / `load Acquire`); `Counters.longest_callback_ns` (`fetch_max`/`swap AcqRel`, `load Acquire`). None publishes other data; no `Relaxed`, no `SeqCst`, no `static mut`, no new lock on the hook path (the channel's waker mutex is never taken, D3). `DelayTimer` and `Runs` are owned by the executor closure, never shared. No new module pair: `core::executor` → `core::windows_actions`, `data::sequence`, `logging`; `hook` → `core::executor` (hook→core exists); `core::app` → `core::executor`; `data` does not import `core::executor` (`run_step` takes an `HWND`).

## Type sketch (signatures only)
```rust
// src/core/executor/mod.rs  (pub: in a lib, so nothing is dead while unwired in X2)
mod runs;
#[cfg(test)] pub(crate) mod testing;
pub const QUEUE_CAPACITY: usize = 32;
pub const START_TIMEOUT: Duration = Duration::from_secs(5);
pub const STOP_TIMEOUT: Duration = Duration::from_secs(2);
const WAKE: u32 = WM_APP;
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct TargetWindow(isize);
impl TargetWindow { pub const NONE: Self; pub fn foreground() -> Self; pub fn for_action(action: &BindAction) -> Self; fn hwnd(self) -> HWND;
                    #[cfg(test)] pub(crate) const fn from_raw(raw: isize) -> Self; }
pub trait Runner: Send + 'static { fn action(&mut self, action: &BindAction); fn step(&mut self, step: &SequenceStep, target: TargetWindow); }
pub struct Win32Runner;
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Submitted { Queued, Dropped, Unavailable, Closed }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Stopped { Joined, TimedOut }
pub struct ExecutorHandle { jobs: SyncSender<Job>, thread_id: u32, shared: Arc<Shared>, counters: &'static Counters }
impl ExecutorHandle { pub fn submit(&self, action: BindAction, target: TargetWindow) -> Submitted; pub fn cancel_pending(&self);
                      pub fn active_runs(&self) -> usize; }
pub struct Executor { handle: Arc<ExecutorHandle>, thread: std::thread::Thread, join: Option<JoinHandle<()>>, done: Receiver<()> }
impl Executor { pub fn start<R: Runner>(runner: R, capacity: usize, counters: &'static Counters) -> io::Result<Self>;
                pub fn handle(&self) -> Arc<ExecutorHandle>; pub fn thread(&self) -> &std::thread::Thread;
                pub fn is_finished(&self) -> bool; pub fn stop(self) -> Stopped; }
impl Drop for Executor { /* stop if not stopped */ }
struct Shared { epoch: AtomicU64, stopping: AtomicBool, active_runs: AtomicUsize }
struct Apartment { initialized: bool }                 // Drop: CoUninitialize iff initialized
struct DelayTimer { armed: Option<(usize, Instant)> }  // executor thread only
enum Arming { Idle, Waiting, Due, Failed(windows::core::Error) }
impl DelayTimer { fn arm(&mut self, next: Next, now: Instant) -> Arming; fn fired(&mut self, id: usize) -> bool; fn kill(&mut self); }
pub(crate) fn timer_ms(now: Instant, at: Instant) -> u32;   // ceil to ms, clamped to 1..=USER_TIMER_MAXIMUM
enum End { Stopped, Died }

// src/core/executor/runs.rs  (no Win32, no threads, no clock of its own)
pub(crate) struct Job { pub(crate) action: BindAction, pub(crate) target: TargetWindow, pub(crate) epoch: u64 }
pub(crate) struct Runs { active: Vec<Run> }
struct Run { steps: Arc<Vec<SequenceStep>>, next: usize, target: TargetWindow, epoch: u64, resume: Resume }
enum Resume { Now, At(Instant), WhenCancelled }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub(crate) enum Next { Idle, At(Instant) }
pub(crate) struct Ctx<'a> { pub(crate) runner: &'a mut dyn Runner, pub(crate) live: &'a dyn Fn(u64) -> bool, pub(crate) now: &'a dyn Fn() -> Instant }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub(crate) struct Polled { pub(crate) next: Next, pub(crate) panics: u32 }
impl Runs { pub(crate) fn new() -> Self; pub(crate) fn start(&mut self, job: Job, ctx: &mut Ctx<'_>) -> u32 /* panics */;
            pub(crate) fn poll(&mut self, ctx: &mut Ctx<'_>) -> Polled; pub(crate) fn drop_timed(&mut self) -> usize;
            pub(crate) fn len(&self) -> usize; }

// src/core/executor/testing.rs  (#[cfg(test)])
pub(crate) struct FakeRunner { .. }   // FakeRunner::new(tx).with_block(d).with_gate(started, release).with_panic_on(s).with_quit_on(s)
pub(crate) struct Record { pub(crate) thread: ThreadId, pub(crate) name: Option<String>, pub(crate) at: Instant, pub(crate) ran: Ran }
pub(crate) enum Ran { Action(String), Step(String, TargetWindow) }   // Display text of the action / step
pub(crate) fn leaked_counters() -> &'static Counters;

// src/core/windows_actions.rs
pub enum FastAction { .. }  impl FastAction { pub fn run(self) }  impl BindAction { pub fn as_fast(&self) -> Option<FastAction> }
// src/data/sequence.rs
pub(crate) fn run_step(step: &SequenceStep, target: HWND);
// src/hook/keyboard_hook.rs
executor: ArcSwapOption<ExecutorHandle>,
pub(crate) fn attach_executor(&self, executor: Option<Arc<ExecutorHandle>>);   // main thread (tests: test thread)
pub(crate) fn submit(&self, action: BindAction, target: TargetWindow) -> Submitted;
fn cancel_actions(&self);                                                       // set_enabled (real change), update_bindings
// src/hook/keyboard_hook_callback.rs
fn carry_out(hook: &KeyboardHook, decision: Decision);
fn dispatch(hook: &KeyboardHook, action: BindAction);
// src/logging/counters.rs
Counter::ExecutorJobsDropped, Counter::ExecutorUnavailable (X2), Counter::PostMessageFailed (X12);
pub struct Snapshot { pub counts: [u64; SLOTS], pub longest_callback: Duration } (X10)
pub const SLOW_CALLBACK: Duration = Duration::from_millis(5);
impl Counters { pub fn record_callback(&self, took: Duration); pub fn longest_callback(&self) -> Duration; pub fn take(&self) -> Snapshot;
                pub fn bump_if_err<T, E>(&self, counter: Counter, result: &Result<T, E>); }
pub fn record_callback(took: Duration); pub fn bump_if_err<T, E>(counter: Counter, result: &Result<T, E>);
// src/logging/mod.rs
fn start_logging<W>(install: impl FnOnce() -> bool, start_writer: impl FnOnce() -> io::Result<W>) -> Option<io::Result<W>>;
```
`FastAction`, `Submitted`, `Stopped`, `Next`, `Polled`, `Arming` are never printed: `Debug` only. New `Counter` variants get `Display` arms (snake_case) in their commit.

**Log lines the lane adds** (exact text; thread):
| Line | Thread | Commit |
|---|---|---|
| `[FATAL] could not start the action executor: {error}` | main | X8 |
| `[WARN] could not initialise COM on the action executor ({hr}); OpenUrl runs without it` | executor | X2 |
| `[ERROR] the action executor received WM_QUIT and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts` | executor | X2 |
| `[ERROR] the action executor's message loop failed ({error}) and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts` | executor | X2 |
| `[ERROR] the action executor panicked and stopped; Launch, OpenUrl and Sequence do nothing until the daemon restarts` | executor | X2 |
| `[ERROR] an action panicked on the action executor; it keeps running` | executor | X2 |
| `[ERROR] could not start the Delay timer ({error}); {n} macros waiting in a Delay stop` | executor | X2 |
| `[WARN] the action executor did not stop within 2 s; exiting without it` | main | X8 |
| `[WARN] window step failed: ShowWindowAsync did not start {op:?}` | executor (macro worker before X8) | X4 |

## Touch set
| File | Change | Why |
|---|---|---|
| `src/core/executor/mod.rs` (new) | handle, executor thread and loop, `Runner`, `Win32Runner`, `TargetWindow`, `Apartment`, `DelayTimer`, `timer_ms`; tests | D2-D9, D14 |
| `src/core/executor/runs.rs` (new) | `Job`, `Runs`, `Run`, `Next`, `Ctx`, `Polled`; pure tests with a fake clock | D6 |
| `src/core/executor/testing.rs` (new, `cfg(test)`) | `FakeRunner`, `Record`, `Ran`, `leaked_counters` | D7 |
| `src/core/mod.rs` | `pub mod executor;` | module |
| `src/core/windows_actions.rs` | `FastAction`, `as_fast`; remove `execute`, `spawn_sequence` import; Sequence variant doc `:28-34`; `post_message_to_foreground` counts (X12); tests | D1, D11 |
| `src/core/app.rs` | executor start/attach/detach/stop + `AttachedExecutor`; `run` doc `:129-136`; SAFETY `:248-250` (L11) | D8 |
| `src/data/sequence.rs` | module doc `:1-11`; `Delay`/`Window` variant docs `:42-48`; `execute_step` → `pub(crate) run_step` (X2); `ShowWindowAsync` in `apply_window_op` `:264-291` + scan test (X3/X4); remove `spawn_sequence` `:102-122`, `Delay` arm no-op, unused imports (X8) | D6, D7, D13 |
| `src/data/binding.rs` | remove `Binding::execute` `:248-250` | D1 |
| `src/hook/keyboard_hook.rs` | `executor` field, `attach_executor`, `submit`, `cancel_actions` in `set_enabled`/`update_bindings`; docs `:41-47`, `:60-66`, `:129-145`; tests | D5 |
| `src/hook/keyboard_hook_callback.rs` | `carry_out(hook, ..)`, `dispatch`, Hold path, gauge recording; module docs `:1-82`; tests (scan, split, Hold, gauge) | D1, D10 |
| `src/system/system_function.rs` | failed language-switch posts counted `:52-59`, `:71-78` | D11 |
| `src/engine/mod.rs` | doc only `:7-33`, `:79-97` (`BindAction::execute` named at `:30`; Hold "runs on that timer thread") | stale docs |
| `src/logging/counters.rs` | `ExecutorJobsDropped`, `ExecutorUnavailable`, `PostMessageFailed`, gauge, `Snapshot`, `SLOW_CALLBACK`, `bump_if_err`; module doc; tests | D3, D10, D11, D14 |
| `src/logging/writer.rs` | `report_counters` takes `Snapshot`; test strings/arrays | D10 |
| `src/logging/mod.rs` | `start_logging` seam (L7) + order (L9); `attach_file` doc (L9); doc `:11-14` (X8, X12); `print_welcome_lines` generalised to `fn_lines(text, signature)` + new scan (X11) | D12, D11 |
| `src/config/loader.rs` | MaskOnly warning placement (L4); empty-config warning (L6); doc `:249-250`; tests | D12 |
| `src/main.rs` | doc `:64-66` (L10) | D12 |
| `src/config/default_config.ron` | `:104` Sequence (runs on the action executor; `swch off`, a reload or exit stops it) (X8); `:142-146` Window (does not wait for the window) (X4); `:147` Delay (waits on the action executor's timer; other macros go on; `swch off`, a reload or exit cut it short) (X8); ASCII only | config meaning |
| `README.md` | X4: `:518`. X8: tree `:35-38` (+`executor/`); `:89`; `:116`; `:118-135` (callback bullets incl. `:125`, app bullets, new `#### executor/` section); `:415-418`; `:488-492`, `:514`, `:528-531` (side by side, interleave at `Delay`, cancel, "a running step finishes first"); `:704`; `:769-773` (counters); `:811` (`Win32_System_Com` also `CoInitializeEx`); `:837`. X10, X12: `:769-773` | brief, M3 |
| `CHANGELOG.md` | L1, L2, L4; X4/X8/X10/X12 entries below | Keep a Changelog |
| `book/src/config/actions.md` | X4: §14.2 `:300-308`. X8: §13 `:8`, `:20`; §13.1 `:38-57`; §13.6 `:200-201`; §13.7 `:208-224`; §14 `:272`; §14.1 `:280-293`; §14.3 `:314-319`; §14.7 `:377`. X12: `:68`, §13.2 `:82`, §13.4 `:144` | brief |
| `book/src/config/known-issues.md` | X8: rows 28 `:72`, 29 `:73`, 36 `:80`, 60 `:128` → Fixed on `feat/action-executor`; row 26 `:64` → Partly fixed (Launch/OpenUrl/Sequence leave the callback; the fast actions stay by design) | M3, ruling Q2 |
| `book/src/config/errors.md` | L4: `:185`. X4: new row after `:199`. X8: new rows for the executor lines above; `:196` (+ `executor_jobs_dropped`, `executor_unavailable`). X10: `:196` (`longest_callback_us`). X12: `:190-191`, `:196` (`post_message_failed`) | message reference |
| `book/src/config/README.md` | X8: `:55` ("macro on the action executor (§14)") | ruling Q2 |
| `book/src/config/example.md` | X8, comments only: `:99-100`, `:186-189`, `:206-207`, `:221-223`, `:237` | ruling Q2 |
| `book/src/config/kinds.md` | X8: `:77`, `:142` (fast actions on the hook thread; Launch, OpenUrl, Sequence queued, §13.1) | ruling Q2 |
| `book/src/config/triggers-conditions.md` | X8: `:41`, `:75`, `:77` (Hold no longer the only safe trigger for Launch/OpenUrl), `:79`, `:123` (KS-08 box: fixed for slow actions) | ruling Q2 |
| `book/src/config/matching.md` | L4: `:150` (no warning when a `keys` binding takes the hotkey over) | ruling Q2 |
| `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/engine/*` (code), `src/ipc.rs`, `src/bin/swch.rs`, `src/logging/testing.rs` | untouched | — |
| non-config book pages | untouched; `impl.md` lists each for the Phase-2 book sync with its file:line: `architecture.md` (§ KS-08 `:213-239`, `:312`, `:565-577`), `glossary.md:107`, `overview.md:20`, `build-and-run.md:131`, `:270`, `:461`, `gaps.md:145-147`, `:166`, `roadmap.md:421`, `:455`, `audit.md:312` (KS-08 status line "Partly fixed in <sha> (feat/action-executor); watchdog: item 2.4") | ruling Q2 |

New book text names files and functions without pinned line links (the Phase-2 book sync pins them), as Phase-1 lanes did ("fixed on `feat/…`"). Every changed `ron` block (example.md, default_config.ron, the L2 snippets) goes through `cfgcheck` (init + flush, `2>&1`) before the commit; outputs recorded in `impl.md`.

## Commits, in order
Every subject `<Area>: <what>`; trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Gate = `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Every commit is green except the declared red ones: L3, L5, L8, X1, X3, X5, X6, X7, X9, X11. L3, L5, L8, X1, X3, X9, X11 are each followed at once by their fix; X5, X6 and X7 are three reds in a row, all made green by X8. Timing rule for the 5 ms bounds (X1-10, X7): every one of the 20 enqueues must be under 5 ms; one retry of the whole batch is allowed; no other loosening; a flake goes to the orchestrator (ruling, critic OQ3).

**L1. `Changelog: reflow the KS-10 entry`** — `CHANGELOG.md:447-456` wrapped at ~76 columns, words unchanged. Gate green.

**L2. `Changelog: the block_default entry names the modifier held under Hold and DoubleTap output`** — in the Breaking entry `CHANGELOG.md:256-276`: (P1) `:272` "A `Tap` on `Alt` or `Win` alone" → "A `Tap`, `Hold` or `DoubleTap` on `Alt` or `Win` alone" (matches `README.md:456`); (P2) new sub-bullet: a `Hold` or `DoubleTap` made only of modifiers now sends its output while that modifier is down in Windows, e.g. `( keys: "LCtrl", action: PressKey("Escape"), trigger: Hold(300) )` sends Ctrl+Esc (opens Start); write `block_default: true` for the old output (KS-19, roadmap 3.1). No Win+L example (ruling). Before committing, run both `( bindings: [ ( keys: "LCtrl", action: PressKey("Escape"), trigger: Hold(300) ) ] )` and the same with `block_default: true` through `cfgcheck` (logging init + flush, `2>&1`); both load without error; outputs in `impl.md`. Gate green.

**L3. `Tests: a MaskOnly fallback for a hotkey a binding takes over warns nothing`** (red) — `config::loader::tests::mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing`: `( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly), (keys: "`", action: DoNothing) ] )` with `language_hotkey(&[VK_OEM_3])` (`loader.rs:1000`): `parse_ron_str(..).warnings` is empty and `blockers(..) == 0`. PREDICTED RED: `assert_eq!` left `["binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true"]`, right `[]`.

**L4. `Config: warn about the MaskOnly fallback only where its blocker is added`** — `loader.rs:338-401` per D12; `mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns` (`:1221`) stays green. CHANGELOG `:75-77`, `errors.md:185` and `matching.md:150`: "…and warns, unless a `keys` binding of the file takes the hotkey over (KS-18): then there is no blocker and no warning". Invariant: blocker set unchanged.

**L5. `Tests: parse_ron_str returns the empty-config warning instead of logging it`** (red) — `config::loader::tests::an_empty_config_warning_is_returned_not_logged`: `capture(|| parse_ron_str("( bindings: [] )", &|_: &BindAction| None))` → `warnings == ["config contains no bindings"]` and no captured line; plus `capture(|| from_ron_str_with_hotkeys(..))` captures exactly one line ending ` [WARN] config contains no bindings\n` (green before and after). PREDICTED RED: left `[]`, right `["config contains no bindings"]`.

**L6. `Config: the empty-config warning goes into the parsed warnings`** — `loader.rs:286-288` pushes instead of logging; doc `:249-250` true again. Output unchanged (empty config: no duplicate warnings, no blocker lines).

**L7. `Logging: init goes through a start_logging seam`** — `start_logging` with today's order: `start_writer()` first, then `install()`; `None` if `install` fails (the started writer is dropped), else `Some(spawned)`. `init` (`mod.rs:108-122`) = `let Some(spawned) = start_logging(|| log::set_logger(&LOGGER).is_ok(), || writer::spawn(..)) else { return }; log::set_max_level(MAX_LEVEL); match spawned { .. }`. No behaviour change, no test change. Gate green.

**L8. `Tests: a second logging init starts no writer`** (red) — `logging::tests::a_second_init_starts_no_writer`: `let started = Cell::new(false); assert!(start_logging(|| false, || { started.set(true); Ok(()) }).is_none()); assert!(!started.get(), "a second init must not start a writer");` and `assert!(matches!(start_logging(|| true, || Ok(7)), Some(Ok(7))))`. No test calls `init`. PREDICTED RED: panic `assertion failed: !started.get()` with message `a second init must not start a writer` (the `is_none()` assertion passes on L7's order).

**L9. `Logging: install the logger before the writer starts; the attach_file doc names the full-queue failure`** — `start_logging` calls `install()` first and `start_writer()` only when it returns `true`; `attach_file` doc `:124-127` names both failures (no writer; queue full for 1 s). Makes L8 green.

**L10. `Docs: the run doc comment gives the real reason the guard lives until run returns`** — `main.rs:64-66`: a parameter lives until its function returns whatever its pattern; keep the "after `App::run` removed the hooks, before a fatal error waits for Enter" part. No test (the daemon binary has no harness, KS-38). Gate green.

**L11. `Core: SAFETY comment on the PostThreadMessageW in console_ctrl_handler`** — `app.rs:248-250`: id and message by value, no memory of ours read, `WM_QUIT` carries no pointer; the id was stored by `run` before the handler was registered (`:141` before `:149`); a stale id only makes the call fail, and the handler has nobody to report to. Gate green.

**X1. `Tests: the action executor runs macros side by side on its one thread, drops jobs when full and stops on an epoch change`** (red) — new `src/core/executor/mod.rs` holding only `mod runs;`, `#[cfg(test)] pub(crate) mod testing;` and `#[cfg(test)] mod tests`; `runs.rs` holding only `#[cfg(test)] mod tests`; `testing.rs` (FakeRunner); `core/mod.rs` `pub mod executor;`.
*`core::executor::runs::tests`* (pure; a fake clock `base + offset`, a local recording runner, `live` a closure over a `Cell<u64>` epoch):
- R-a `a_run_goes_up_to_its_first_delay_and_resumes_at_its_deadline` — `[Key F13, Delay(100), Key F14]`: poll at t0 → `[F13]`, `next == At(t0+100ms)`; poll at t0+99 ms → nothing; poll at t0+100 ms → `[F13, F14]`, `Idle`, `len() == 0`.
- R-b `two_macros_fired_together_interleave_at_their_delays_and_keep_their_order` — A `[F13, Delay(30), F14, Delay(30), F15]`, B `[F16, Delay(30), F17, Delay(30), F18]` started at t0; polls at t0, t0+30, t0+60 → exactly `[F13, F16, F14, F17, F15, F18]`.
- R-c `a_cancelled_waiting_run_never_resumes` — `[F13, Delay(100), F14]` at epoch 0; poll; epoch → 1; poll at t0+200 → nothing ran, `len() == 0`, `Idle`.
- R-d `a_job_of_an_old_epoch_never_starts` — `start` with a stale epoch: nothing runs, `len() == 0` (Sequence and OpenUrl).
- R-e `the_delay_starts_when_its_step_is_reached` — the runner advances the fake clock 500 ms inside `step(OpenUrl step)`; `[OpenUrl step, Delay(100), F14]` → `next == At(t0+600ms)`.
- R-f `a_panicking_step_ends_only_its_macro` — runner panics on `key F13`; A `[F13, F14]`, B `[F15]` → recorded `[F15]`, `panics == 1`, `len() == 0`.
- R-g `a_zero_delay_lets_the_other_runs_go_first` — A `[F13, Delay(0), F14]`, B `[F15]` → poll 1: `[F13, F15]`, `next == At(t <= now)`; poll 2: `F14`.
- R-h `launch_and_open_url_run_when_their_job_starts` — `start(Launch)` and `start(OpenUrl)` call `runner.action` at once in that order; `len() == 0`.
*`core::executor::tests`* (real executor thread, FakeRunner, own `leaked_counters()`; FakeRunner never touches Win32 except the COM probe and `PostQuitMessage` on its own thread):
1. `a_hundred_sequence_runs_use_the_one_executor_thread` — 100 × submit `Sequence([Key("F13")])`, waiting for each `Record`; all 100 ThreadIds equal `executor.thread().id()`, differ from the test thread, name `"action-executor"`.
2. `delay_10000_stops_within_100_ms_of_an_epoch_change` — `Sequence([Delay(10000), Key(F14)])` (a Delay-only start); wait until `active_runs() == 1` (≤ 1 s); `t0`; `cancel_pending()`; `active_runs() == 0` observed < 100 ms after `t0`; submit `Sequence([Key(F15)])` → `F15` recorded; `F14` never.
3. `a_delay_5000_macro_holds_up_no_other_job` (R1) — `[F13, Delay(5000), F14]`; on `F13` and `active_runs() == 1`: `t0`, submit `OpenUrl("https://example.com")` → its `Record.at - t0 < 100 ms`; then `cancel_pending()`, `stop() == Joined`; recorded `[F13, open https://example.com]`.
4. `two_macros_fired_together_both_run_in_their_own_order` (R1) — A `[F13, Delay(300), F14]`, B `[F15, Delay(300), F16]` submitted back to back → recorded exactly `[F13, F15, F14, F16]` within 2 s (round 1's one-at-a-time design gives `[F13, F14, F15, F16]`).
5. `a_cancelled_waiting_run_never_resumes` (R1) — `[F13, Delay(150), F14]`; at `active_runs() == 1`: `cancel_pending()`; the test thread waits 400 ms; recorded `[F13]`; a later `[F15]` runs.
6. `a_delay_resumes_its_run_after_its_deadline` — `[F13, Delay(50), F14]`: `F14.at - F13.at >= 50 ms` and `< 1 s` (the thread timer is delivered through the loop).
7. `a_delay_of_u64_max_waits_until_cancelled` (R1) — `[Delay(u64::MAX), Key(F14)]`; 50 ms later nothing ran, `is_finished() == false`, `active_runs() == 1`; `cancel_pending()` + submit `[F15]` → `[F15]` (no overflow panic).
8. `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms` (R1; `Runs` + `timer_ms`, fake clock) — poll a `[Delay(u64::MAX), F14]` run: `next` is `Idle` (deadline overflowed) or `At(t)` with `timer_ms(now, t) == USER_TIMER_MAXIMUM` and `<= u32::MAX - 1`; a poll one day later still runs nothing.
9. `timer_ms_rounds_up_and_clamps` — 0.4 ms → 1; 10.2 ms → 11; `t <= now` → 1; 30 days → `USER_TIMER_MAXIMUM`.
10. `twenty_slow_jobs_each_enqueue_in_under_5_ms` — capacity 64, FakeRunner `with_block(500 ms)`; 20 × `submit(OpenUrl("https://example.com"), NONE)`, each timed; all `Queued`; every one < 5 ms, one whole-batch retry allowed; `stop()` then returns `Joined` and the runner saw ≤ 1 call.
11. `a_full_queue_drops_the_job_and_counts_it` — capacity 1, gated runner: A starts (runner signals), B `Queued`, C `Dropped`; `counters.take()` has `executor_jobs_dropped == 1`; open gate → B runs, C never.
12. `a_job_queued_before_an_epoch_change_does_not_run` — gated A running, B queued, `cancel_pending()`, open gate: B never runs; a later C runs.
13. `jobs_run_in_the_order_they_were_queued_with_their_target` — `Launch(exe: "no-such.exe")`, `OpenUrl("https://example.com")`, `Sequence([Key(F13), Key(F14)])` with `TargetWindow::from_raw(0x1234)` → `[Action("launch no-such.exe"), Action("open https://example.com"), Step("key F13", 0x1234), Step("key F14", 0x1234)]` (Display, `windows_actions.rs:284-306`, `sequence.rs:69-99`).
14. `stop_interrupts_a_delay_drops_the_queue_and_joins` — FakeRunner `with_block(100 ms)`; `[F13, Delay(10000), F14]` waiting; `OpenUrl(A)` recorded (blocking); then B, C, D submitted; `stop() == Joined` within 1 s; recorded `[F13, open A]`.
15. `a_submit_after_stop_is_closed_and_not_counted` — a handle taken before `stop()`; `submit` after it → `Closed`; counters all 0.
16. `an_executor_ended_by_wm_quit_counts_every_later_submit` (R3) — FakeRunner `with_quit_on("open quit")`; submit `OpenUrl("quit")`; `is_finished()` within 2 s; `submit(OpenUrl("later"))` → `Unavailable`; `counters.take()` has `executor_unavailable == 1`, `executor_jobs_dropped == 0`; `stop() == Joined`; a submit after it → `Closed` and counters stay 0.
17. `a_panicking_job_leaves_the_executor_running` — runner panics on `open boom`; the next job runs.
18. `the_executor_runs_jobs_in_a_single_threaded_apartment` — a probe runner calls `CoGetApartmentType` inside `action()`: `APTTYPE_STA`.
19. `the_apartment_guard_uninitialises_com_on_drop` — scratch thread: `Apartment::enter_sta()`, probe = STA; drop; probe = `Err(CO_E_NOTINITIALIZED)` or a non-STA type (implicit MTA).
20. `the_win32_runner_opens_a_url` — `#[ignore = "manual: opens https://example.com in the default browser through Win32Runner on a real executor thread; check that the page opens and the test passes"]`.
PREDICTED RED: `cargo test --locked` fails to compile: E0412/E0422/E0425/E0433 "cannot find type `Executor` / trait `Runner` / type `TargetWindow` / enum `Submitted` / `Stopped` / struct `Apartment` / struct `Runs` / struct `Job` / enum `Next` / struct `Ctx` / function `timer_ms` / value `QUEUE_CAPACITY` in this scope", E0599 "no variant named `ExecutorJobsDropped` / `ExecutorUnavailable` found for enum `Counter`". `cargo build` green.

**X2. `Core: the action executor thread with resumable macro runs`** — implements `src/core/executor/{mod.rs, runs.rs, testing.rs}` (D2-D7, D9, D14) and `Counter::ExecutorJobsDropped` ("executor_jobs_dropped") + `Counter::ExecutorUnavailable` ("executor_unavailable"), SLOTS 6, counters/writer test arrays and strings updated; `sequence.rs`: `execute_step` → `pub(crate) fn run_step` used by `spawn_sequence` and `Win32Runner`; its `Delay` arm still sleeps (for `spawn_sequence`, until X8; the executor never passes `Delay`). Not wired: no behaviour change. Makes X1 green (27 pass, 1 ignored).

**X3. `Tests: a Window step never waits for its window`** (red) — `data::sequence::tests::a_window_step_never_waits_for_its_window`: reads `src/data/sequence.rs` (`CARGO_MANIFEST_DIR`), lines before `#[cfg(test)]` + `mod tests {`, and lists every line containing `concat!("ShowWindow", "(")` (so `ShowWindowAsync(` and the doc's `` `ShowWindow` `` do not match, and the test does not match itself). PREDICTED RED: assertion failure listing `src/data/sequence.rs:276`, `src/data/sequence.rs:279`, `src/data/sequence.rs:282`.

**X4. `Macros: Window steps use ShowWindowAsync and do not wait for the window`** — D13 in `apply_window_op` (SAFETY comment updated: `ShowWindowAsync` only posts; a stale HWND makes it return `FALSE`); variant docs `:46-48`. README `:518`; `actions.md` §14.2 `:300-308` (calls, "does not wait", failure line); `errors.md` new row `[WARN] window step failed: ShowWindowAsync did not start Minimize` (or `Maximize`, `Restore`); `default_config.ron:142-146`; CHANGELOG Changed (below). Makes X3 green. Invariant: `Close` and the invalid-target skip unchanged.

**X5. `Tests: the hook path names no execute call`** (red) — `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call`: scans every `.rs` under `src/hook` (all lines, tests included) for `concat!(".exe", "cute()")`. Compiles on X4's tree (critique M2). PREDICTED RED: assertion failure listing `src/hook/keyboard_hook_callback.rs:219: action.execute();`.

**X6. `Tests: no macro code sleeps or blocks in a wait`** (red) — `core::executor::tests::macros_never_sleep_or_block_in_a_wait`: the non-test lines of `src/core/executor/mod.rs`, `src/core/executor/runs.rs` and `src/data/sequence.rs` contain neither `concat!("thread::", "sleep")` nor `concat!("MsgWaitFor", "MultipleObjects")` (ruling R1). Compiles on X5's tree. PREDICTED RED: assertion failure listing `src/data/sequence.rs:129: SequenceStep::Delay(ms) => thread::sleep(Duration::from_millis(*ms)),`.

**X7. `Tests: fast actions run where they fire, the others go to the executor, and swch off or a reload stops a macro`** (red) —
- `core::windows_actions::tests::only_launch_open_url_and_sequence_leave_the_firing_thread` — every variant (table as `:316-351`): `as_fast()` `Some` with the payload carried (`PressKey(VK_BACK)` → `FastAction::PressKey(VK_BACK)`, `PostMessage` fields, every `MediaKey`), `None` for `Launch`, `OpenUrl`, `Sequence`. Never calls `run()`.
- `hook::keyboard_hook_callback::tests::carry_out_queues_the_slow_actions_in_order_and_runs_none_here` — local `KeyboardHook` + attached executor (FakeRunner); `carry_out(&hook, Decision { fire: vec![DoNothing, Launch(no-such.exe), OpenUrl(https://example.com), Sequence([Key(F13)])], ..Default::default() })`; runner gets the three in order on the executor thread, nothing on the test thread. Only `DoNothing` may run inline in a test.
- `…::carry_out_returns_in_under_5_ms_while_slow_actions_block` — FakeRunner `with_block(500 ms)`, capacity 64; 20 × `carry_out` with one `OpenUrl`, each < 5 ms (timing rule above).
- `…::a_hold_runs_its_slow_action_on_the_executor` (critique M5) — holds `GLOBAL_HOOK`; starts an executor with a FakeRunner and attaches it to `get_hook()` (a guard detaches on drop, also on unwind, before the executor stops); `get_hook().update_bindings(vec![Binding::new(KeyCombination::new(VK_F13), BindAction::OpenUrl("https://example.com".into())).with_trigger(Trigger::Hold(1)).with_allow_extra_modifiers(true)])`; `call_keyboard_callback(F13, WM_KEYDOWN)`; within 2 s the runner records `open https://example.com` on thread `action-executor` (so it went `hold-timer` → `carry_out(get_hook(), ..)` → `submit`); then `call_keyboard_callback(F13, WM_KEYUP)`. Nothing is installed or injected: no hook exists, `CallNextHookEx` has no next hook (`:441-447`), the mask is only counted under test (`:267-279`), and the action goes to the FakeRunner. If this proves impossible without weakening that, the developer stops and reports (ruling M5).
- `hook::keyboard_hook::tests::swch_off_stops_a_running_macro_within_100_ms` — local hook + attached executor; `hook.submit(Sequence([Key(F13), Delay(10000), Key(F14)]), NONE)`; on `F13` and `active_runs() == 1`: `t0`, `set_enabled(false) == Some(true)`; `active_runs() == 0` < 100 ms after `t0`; never `F14`.
- `…::a_reload_stops_a_running_macro_within_100_ms` — same with `hook.update_bindings(vec![])`.
- `…::a_repeated_swch_on_keeps_the_running_macro` — `Sequence([Key(F13), Delay(300), Key(F14)])`; `set_enabled(true) == Some(false)`; `F14` arrives within 2 s.
- `…::a_slow_action_without_an_executor_is_closed` — `KeyboardHook::new().submit(OpenUrl(..), NONE) == Submitted::Closed`.
PREDICTED RED: compile errors E0599 "no method named `as_fast` found for … `BindAction`", E0433 "failed to resolve: use of undeclared type `FastAction`", E0061 "this function takes 1 argument but 2 arguments were supplied" (`carry_out`), E0599 "no method named `attach_executor` / `submit` found for … `KeyboardHook`".

**X8. `Hook: run fast actions in place and queue Launch, OpenUrl and Sequence on the action executor`** — D1, D4, D5, D8: `windows_actions.rs` (`FastAction`, `as_fast`, `execute` removed), `binding.rs` (`execute` removed), `keyboard_hook.rs`, `keyboard_hook_callback.rs` (`carry_out(hook, ..)` iterates `fire` by value → `dispatch`; `start_hold_timer` → `carry_out(get_hook(), decision)`; module docs steps 6 and "Threads and locking"), `sequence.rs` (`spawn_sequence` and the `Delay` sleep removed; `Delay` arm a no-op with a comment; `thread`/`Duration` imports dropped; module doc), `app.rs` (D8), docs in `engine/mod.rs`, `logging/mod.rs:11-14` (Launch/OpenUrl/macro failures are logged on `action-executor`). README, `default_config.ron`, config pages (touch set), CHANGELOG (below); `cfgcheck` on the changed `ron` blocks. Makes X5, X6, X7 green. Invariants: suppression/outcome logic untouched (`decide_event`, `LRESULT(1)` iff `suppress`); the lock is released before `carry_out`; no allocation, lock or log added on the hook path.

**X9. `Tests: the longest keyboard callback is recorded and a slow one is reported`** (red) —
- `logging::counters::tests::the_longest_callback_is_kept_until_taken` (record 3 ms, 1 ms → 3 ms; `take()` returns it and resets to 0).
- `…::a_slow_callback_alone_is_reported_and_a_fast_one_is_not` (`summary` of all-zero counts with 4.999 ms → `None`; with 5 ms → `Some("counters since the last report: menu_mask_short=0 key_press_short=0 hold_timer_not_started=0 log_lines_dropped=0 executor_jobs_dropped=0 executor_unavailable=0 longest_callback_us=5000")`).
- `logging::writer::tests::a_flush_reports_a_slow_callback`.
- `hook::keyboard_hook_callback::tests::the_keyboard_callback_records_how_long_it_ran` (holds `GLOBAL_HOOK`; F13 `DoNothing` with `allow_extra_modifiers`; one key-down via `call_keyboard_callback`; `counters::COUNTERS.longest_callback() > Duration::ZERO`; no test ever takes the global counters, `logging/testing.rs:4`).
PREDICTED RED: E0599 "no method named `record_callback` / `longest_callback` found for struct `Counters`", E0425 "cannot find value `SLOW_CALLBACK`", E0412 "cannot find type `Snapshot`".

**X10. `Logging: record the longest keyboard callback and report it from 5 ms`** — D10 in `counters.rs`, `writer.rs`, `keyboard_hook_callback.rs`; README `:769-773`; `errors.md:196`; CHANGELOG Added. Makes X9 green.

**X11. `Tests: the functions on the hook path log nothing`** (red) — in `logging::tests`: `print_welcome_lines` generalised to `fn_lines(text, signature) -> RangeInclusive<usize>` (first match before the file's `#[cfg(test)]` + `mod tests {`; panics if missing, so a rename fails loudly); `only_the_startup_banner_prints_outside_src_bin` keeps passing. New `the_functions_on_the_hook_path_log_nothing`, one list, each entry with a comment naming why it is on it (critique M1):
  - `src/core/windows_actions.rs` `fn as_fast(` — picks the placement for every fire on the hook thread;
  - `…` `fn run(` (`FastAction::run`) — runs the fast actions on the hook / `hold-timer` thread;
  - `…` `fn press_vk(`, `fn post_message_to_foreground(` — what `run` calls for `PressKey`/`Media` and `PostMessage`;
  - `src/system/system_function.rs` `fn execute(`, `fn switch_language_forward(`, `fn switch_language_backward(`, `fn toggle_caps_lock(`, `fn press_key(` — what `run` reaches for `SwitchLanguage*` and `ToggleCapsLock`;
  - `src/core/executor/mod.rs` `fn submit(` (`ExecutorHandle::submit`), `fn for_action(`, `fn foreground(` (`TargetWindow`), `fn cancel_pending(` — every slow fire, and the cancel `set_enabled`/`update_bindings` call;
  - `src/hook/keyboard_hook.rs` `fn submit(`, `fn cancel_actions(` — the hook's entry to the executor and its cancel;
  - `src/logging/counters.rs`, the whole non-test part — `bump`, `bump_if_err`, `record_callback` run on the hook thread.
  (`keyboard_hook_callback.rs`, with `carry_out` and `dispatch`, stays covered whole by the existing scan.) A function's lines contain no `names_the_log_crate` token. PREDICTED RED: assertion lists 3 sites: `src/core/windows_actions.rs:<line of log::error!("Post message error …")> in fn post_message_to_foreground(`, `src/system/system_function.rs:58 in fn switch_language_forward(`, `src/system/system_function.rs:77 in fn switch_language_backward(`.

**X12. `Actions: count a failed post on the hook path instead of logging it`** — D11: `Counter::PostMessageFailed` ("post_message_failed", SLOTS 7; arrays/strings updated, X9's expected line gains ` post_message_failed=0` before ` longest_callback_us`), `bump_if_err` + `counters::tests::bump_if_err_counts_only_an_error` (local `Counters`); the three sites. `logging/mod.rs:11-14` final wording (the hook path never logs). README `:769-773`; `actions.md` `:68`, §13.2 `:82`, §13.4 `:144`; `errors.md:190-191`, `:196`; CHANGELOG Changed. Makes X11 green.

### CHANGELOG entries (`## [Unreleased]`; commit named)
- **Fixed, KS-08 (roadmap item 2.1) (X8):** `Launch`, `OpenUrl` and `Sequence` no longer run inside the keyboard hook; the hook queues them for the `action-executor` thread (message loop, COM initialised as a single-threaded apartment, as `ShellExecuteW` asks) and returns at once, so a cold browser, an antivirus scan or an unreachable path no longer freezes typing or lets Windows remove the hook. `PressKey`, `Media`, `ToggleCapsLock`, `SwitchLanguage`, `SwitchLanguageBackward` and `PostMessage` still run before the hook returns, so their output stays in order with the keys typed after them; a `Hold` runs them on its timer thread as before and queues the others. Up to 32 actions wait; another one is dropped and counted in `executor_jobs_dropped`. Macros still run side by side: a `Delay` waits on a timer of the executor and holds up no other action or macro; a step that blocks inside Windows (a `Launch`, an `OpenUrl`, a dialog `ShellExecuteW` shows) holds up the others while it runs. Manual checks: M1, M5.
- **Fixed (gaps "No cancellation for macro workers") (X8):** `swch off`, a reload (saving `config.ron`) and `swch exit` stop every running macro within 100 ms, also inside a `Delay` of any length, and drop the actions still queued; a step already running (a `Launch`, an `OpenUrl`) finishes first. A repeated `swch on` stops nothing. A macro no longer starts a thread of its own. Manual checks: M2-M4, M10.
- **Changed (X4):** `Window(Minimize)`, `Window(Maximize)` and `Window(Restore)` steps use `ShowWindowAsync`, so a window that is Not Responding no longer holds up the macro or the other actions. The step no longer waits for the window to finish the change: put a `Delay` after it when the next step depends on it (the examples already do). A step Windows does not accept logs `[WARN] window step failed: ShowWindowAsync did not start <op>`. Manual check: M11.
- Edit **Fixed KS-01** `:357-358` (X8): "A running macro keeps running after `swch off` (roadmap item 2.1)" → "`swch off` also stops a running macro (roadmap item 2.1)".
- Edit **Fixed KS-33** `:586-588` (X8): add "and a `Sequence` macro no longer starts a thread, so it cannot abort the daemon when one cannot be started (roadmap item 2.1)".
- **Added** (X8, extended by X10 and X12): counters of roadmap item 2.1 — `executor_jobs_dropped`; `executor_unavailable`, the actions fired after the action executor stopped on its own (it logs one `[ERROR]` line; `Launch`, `OpenUrl` and `Sequence` then do nothing until the daemon restarts); `longest_callback_us`, the longest keyboard-hook callback since the last report, which also triggers the report line when it reaches 5 ms (X10); `post_message_failed` (X12). Update the 1.7 bullet `:110-118` ("all four" → "all of them", sample line) in the commit that changes the line.
- **Changed (X12):** `[ERROR] Post message error …`, `[ERROR] Language switch error: …` and `[ERROR] Language switch backward error: …` are counted in `post_message_failed`; the error text is no longer shown.
No Breaking entry: macro order and concurrency are as before the lane (ruling R1).

## Acceptance mapping
| Acceptance (brief / roadmap 2.1 / rulings) | Covered by |
|---|---|
| `rg "\.execute\(\)" src/hook` finds nothing | X5 `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` (red by assertion on X4's tree, green at X8); `BindAction::execute`/`Binding::execute` removed (compile) |
| 100 `Sequence` runs in a row create no threads (every job on the one executor thread) | `core::executor::tests::a_hundred_sequence_runs_use_the_one_executor_thread`; `spawn_sequence` removed (X8; tester greps `thread::spawn` in `src/data`) |
| `Delay(10000)` stops within 100 ms of an epoch change (real executor thread, Delay-only job, fake runner) | `delay_10000_stops_within_100_ms_of_an_epoch_change` (`active_runs` → 0 < 100 ms; `F14` never); via `swch off`: `hook::keyboard_hook::tests::swch_off_stops_a_running_macro_within_100_ms`; reload: `a_reload_stops_a_running_macro_within_100_ms`; exit: `stop_interrupts_a_delay_drops_the_queue_and_joins`; manual M2-M4, M10 |
| 20 slow jobs (fake runner 500 ms) → each enqueue < 5 ms; cold browser is manual | `twenty_slow_jobs_each_enqueue_in_under_5_ms`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`; manual M1 |
| full queue drops the job and bumps the counter | `a_full_queue_drops_the_job_and_counts_it` |
| executor: own `GetMessageW` loop + STA COM, balanced `CoUninitialize` | `the_executor_runs_jobs_in_a_single_threaded_apartment`, `the_apartment_guard_uninitialises_com_on_drop`, `a_delay_resumes_its_run_after_its_deadline` (WM_TIMER through the loop); M8 |
| bounded queue; overflow counted; hook path never calls `log::` | X1 tests 10-11; X11 `the_functions_on_the_hook_path_log_nothing`; existing `only_the_startup_banner_prints_outside_src_bin` |
| job = (action, hwnd, epoch) | `jobs_run_in_the_order_they_were_queued_with_their_target`, `a_job_queued_before_an_epoch_change_does_not_run`, `runs::tests::a_job_of_an_old_epoch_never_starts` (by value: declared deviation 1) |
| only the fast actions stay in the callback | `only_launch_open_url_and_sequence_leave_the_firing_thread`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`; M7 |
| Hold on its `hold-timer` follows the split | `hook::keyboard_hook_callback::tests::a_hold_runs_its_slow_action_on_the_executor`; M6 |
| longest callback recorded, reported by the log writer | `the_keyboard_callback_records_how_long_it_ran`, `the_longest_callback_is_kept_until_taken`, `a_slow_callback_alone_is_reported_and_a_fast_one_is_not`, `a_flush_reports_a_slow_callback` |
| main starts the executor before the hooks, stops it after removal; exit stops pending work and joins before the final flush | `stop_interrupts_a_delay_drops_the_queue_and_joins`, `a_submit_after_stop_is_closed_and_not_counted`; order in `App::run` by review (no harness, KS-38); M4 (bounded join: declared deviation 2) |
| injectable runner; no test runs a real action / SendInput / ShellExecuteW / program | FakeRunner everywhere; only `the_win32_runner_opens_a_url` uses `Win32Runner`, `#[ignore = "manual: …"]`; tests run `FastAction::run` only for `DoNothing` |
| roadmap: "With 20 `OpenUrl` calls against a cold browser, the callback stays under 5 ms" | M1 (no `longest_callback_us` ≥ 5000 report) |
| R1: a `Delay(5000)` macro followed by a fast job: the fast job runs within 100 ms | `a_delay_5000_macro_holds_up_no_other_job`; M5 |
| R1: two interleaved macros keep each one's step order | `runs::tests::two_macros_fired_together_interleave_at_their_delays_and_keep_their_order` (exact), `two_macros_fired_together_both_run_in_their_own_order` (thread); M5 |
| R1: a cancelled waiting run never resumes | `runs::tests::a_cancelled_waiting_run_never_resumes`, `executor::tests::a_cancelled_waiting_run_never_resumes` |
| R1: `Delay(u64::MAX)` waits until cancelled and arms no timer longer than `u32::MAX - 1` ms | `a_delay_of_u64_max_waits_until_cancelled`, `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms`, `timer_ms_rounds_up_and_clamps` |
| R1: no `thread::sleep`, no blocking pumped wait | X6 `macros_never_sleep_or_block_in_a_wait` |
| R2: `Window(...)` steps never wait for a hung window | X3 `a_window_step_never_waits_for_its_window`; M11 |
| R3: executor death is counted; Closed during stop is silent | `an_executor_ended_by_wm_quit_counts_every_later_submit`, `a_submit_after_stop_is_closed_and_not_counted` |
| leftovers | L1-L11 (L3, L5, L8 red-first; L8 red by assertion on L7's seam) |

## Manual checks (human only)
Release daemon started with `swch open`; `config.ron` edited in place (hot reload); Notepad focused unless stated.
| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | `( keys: "F13", action: OpenUrl("https://example.com") )`; every browser window closed ≥ 1 min | Press F13 20 times within ~5 s, typing `abc` between presses; `swch status`; tap CapsLock; `swch exit`; read `daemon.log` | Typing never pauses; 20 tabs open; `running`; the layout switches; no counters line with `longest_callback_us` ≥ 5000; if a line appears, `executor_jobs_dropped=0` and `executor_unavailable=0` | — |
| M2 | `( keys: "F13", action: Sequence([Text("a"), Delay(10000), Text("b")]) )` | F13; within 3 s `swch off`; wait 12 s; `swch on`; F13; wait 12 s | First run types only `a`; second `a`, then `b` 10 s later | — |
| M3 | as M2 | F13; within 3 s save `config.ron` unchanged | `[INFO] config reloaded (…)`; only `a` | — |
| M4 | as M2, console visible | F13; within 3 s `swch exit` | Daemon exits within ~1 s, no `b`; `daemon.log` ends with `[INFO] Removing keyboard hook...` (a counters line may follow) | — |
| M5 | `( keys: "F16", action: Sequence([Text("1"), Delay(2000), Text("2")]) )`, `( keys: "F13", action: OpenUrl("https://example.com") )` | (a) F16 twice within 1 s; (b) F16, then F13 within 1 s | (a) `1122` over ~3 s (both run side by side, as before the lane); (b) the browser opens at once, `2` appears 2 s after F16 | — |
| M6 | `( keys: "F14", action: OpenUrl("https://example.com"), trigger: Hold(500) )`, `( keys: "F15", action: PressKey("A"), trigger: Hold(300) )` | Hold F14 1 s; hold F15 1 s | Browser opens; one `a` | — |
| M7 | Default config + `( keys: "F9", action: PressKey("Backspace") )` | Tap CapsLock; Shift+CapsLock; hold F9 1.5 s in text | Layout switches; CapsLock toggles once; Backspace at the repeat rate | — |
| M8 | `( keys: "F17", action: OpenUrl("ms-settings:display") )`, `( keys: "F18", action: OpenUrl("C:\\Windows") )` | Press each | Settings › Display; Explorer at `C:\Windows` | — |
| M9 | A default browser installed | `cargo test --locked -- --ignored the_win32_runner_opens_a_url` | Browser opens example.com; test passes | `core::executor::tests::the_win32_runner_opens_a_url` |
| M10 | Ctrl+C test: M2 config, daemon in its console | F13; Ctrl+C within 3 s | Exits promptly, no `b` | — |
| M11 (R2) | `( keys: "F19", action: Sequence([ Window(Minimize) ]) )`, `( keys: "F13", action: OpenUrl("https://example.com") )`; in a separate console: `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.Form; $f.Text = 'hung'; $f.Show(); Start-Sleep -Seconds 60"` | Click the `hung` window; wait until its title shows "(Not Responding)" (about 5 s); press F19, then F13 within 2 s; `swch status`; before the 60 s end, `swch exit` | The browser opens within ~1 s of F13 (with the synchronous `ShowWindow` it would open only after the window recovers); the window (or its ghost) minimises when Windows handles it; no `[WARN] window step failed`; `swch exit` returns within ~1 s with no `[WARN] the action executor did not stop within 2 s` | — |

## Risks and edge cases (code-reviewer checklist rev-R1..rev-R10)
- **rev-R1 hook callback.** New work per event: two `Instant::now()` + one `fetch_max`. Per slow fire: `as_fast` match, (Sequence only) `GetForegroundWindow`, one `ArcSwapOption::load`, one atomic load, `try_send` into a preallocated slot, one `PostThreadMessageW` (returns without waiting); on `Disconnected` one more load + one counter RMW. No allocation (the job is moved), no lock, no `log::`. Outcome logic untouched; `code < 0` and remapping-off paths unchanged and unmeasured.
- **rev-R2 sentinel.** No new `SendInput`; macro steps keep their builders. The executor's injected keys re-enter the hook on main and pass by sentinel before any state (`keyboard_hook_callback.rs:180`).
- **rev-R3 shared state.** `ArcSwapOption` + atomics only; `with_engine` never taken on the executor; nothing under the engine lock changed (`set_enabled` bumps after the lock). `Runs`/`DelayTimer` are executor-local.
- **rev-R4 orderings.** As listed in the thread map; the `fetch_add` → post → `GetMessageW` returns → `load` chain relies on the documented barriers of interlocked and wait/message functions ("Synchronization and Multiprocessor Issues"); D14's stop-time classification is declared best-effort.
- **rev-R5/R6.** No `Binding` builder; no new user-visible enum variant (`FastAction`/`Submitted`/`Stopped`/`Next`/`Polled`/`Arming` never printed); new `Counter` variants get `Display`. `as_fast`, `Win32Runner::action`, `run_step` and `apply_window_op` matches have no `_`.
- **rev-R7.** `update_bindings` keeps its single sorted store; the bump follows it; no second publish path.
- **rev-R8/R9.** No key-identity or registry change.
- **rev-R10 loop and shutdown.** Executor start before install; a start failure is fatal before any hook (pipe dropped, KS-20 order kept). Stop strictly after `uninstall` (D8). Bounded 2 s. A foreign `WM_QUIT`, a `GetMessageW` error or a loop panic ends the executor with an `[ERROR]`, then counted (D14). Ctrl+C on a closed console: the process may end before the stop finishes (as today).
- **Residual race R-7 (accepted until 2.2).** A `hold-timer` (or IPC/config-watcher) thread that loaded the handle before detach may post `WAKE` after the executor thread ended; thread ids are only "valid from the time the thread is created until the thread has been terminated" (Thread Handles and Identifiers), so a re-used id could receive one unknown `WM_APP` thread message; the same thread could add one `executor_unavailable` count to the last report (D14). 2.2 removes `hold-timer`.
- **Blocking steps.** A `Launch`/`OpenUrl` (action or step) or a `ShellExecuteW` dialog holds the executor until it returns: jobs wait in the queue (32), due runs resume right after it (the loop polls after every job and before every wait), `swch off` takes effect once it returns; wake and timer messages a modal loop swallows are harmless (drain-before-wait; `Instant` decides; the next `arm` re-creates the timer).
- **Hung windows (critique I1).** No window op waits any more (D13); `SendInput` steps do not address a window.
- **Timer details.** Early, late, stale (KillTimer "does not remove WM_TIMER messages already posted") or reused-id `WM_TIMER` only make the loop poll and re-arm from `Instant`; a `Delay` never ends early. Granularity about one clock tick plus default coalescing (N3). `SetTimer` failure (resource exhaustion) ends the macros waiting on a deadline with one `[ERROR]`; macros waiting "until cancelled" need no timer. `UOI_TIMERPROC_EXCEPTION_SUPPRESSION` is not changed: no `TimerProc` of ours exists (follow-up note only, not a defect of this lane).
- **Repeats.** `repeat: true` on a macro at the repeat rate: 32 queued, then drops counted; each started repeat becomes a run (old code: one thread per repeat). Active runs are unbounded (N1).
- **Interleaving.** Two macros interleave at `Delay` steps, oldest first; a stretch of steps between Delays is never split (N2); `Delay(0)` yields.
- **Order across one decision.** Fast actions of a decision run before the hook returns; its slow ones run later, in order. Documented in §13.1.
- **Reload mid-gesture / foreground change / `swch off` during a pending gesture.** Gesture state is the engine's (unchanged; 2.3). A slow action fired by a pre-reload gesture is queued with the new epoch and runs. A foreground change bumps nothing.
- **UIPI / elevation / tokens / Secure Desktop.** Unchanged; the executor is the same elevated process (KS-09 children elevated, 3.3).
- **IME/layouts.** `Text` steps unchanged; `SwitchLanguage*` stays inline.
- **Sleep/hibernate.** The deadline is an `Instant`, re-read on every wake; the timer only wakes the loop.
- **Tests.** Real executor threads create an STA, a queue and thread timers on test threads (harmless; nothing system-wide). `PostQuitMessage` in one test posts only to that test's executor thread. Timing bounds: 100 ms, and 5 ms with the ruling's single whole-batch retry. Only `DoNothing` runs inline in tests; `Win32Runner` only in the ignored test. The Hold test runs under `GLOBAL_HOOK` and detaches its executor on unwind.
- **Log tail.** No line on a normal stop keeps Checkpoint 1 item 16's last line.

## Questions for the orchestrator
Decided by the rulings (recorded, not open): Q1 accepted as declared deviation 1; Q2 resolved into the touch set and the `impl.md` hand-off list; Q4 (`SLOW_CALLBACK` 5 ms), Q5 (32 / 2 s / 5 s), Q6 (post errors counted without text) accepted; Q7 replaced by ruling R3 (D14). Q3 is withdrawn (superseded by R1). None of the following blocks the lane.

1. **N1 Active runs are not bounded.** A held `repeat: true` macro with a long `Delay` keeps adding runs (about 100 bytes each). Recommendation: accept for this lane — the old code started a thread per repeat, the queue still bounds the backlog, and a cap would be a new user-visible limit nobody asked for; revisit with 3.5 if a report shows growth.
2. **N2 Interleaving granularity.** A run advances up to its next `Delay`, so two macros interleave at `Delay` steps, not one step each per turn. Recommendation: accept — ruling R1 says a new job "runs its steps until the first `Delay` or its end"; it keeps a macro's `Text`/`Key` stretch whole; one-thread-per-macro gave no order at all, so this is one of the orders it could produce.
3. **N3 Delay lateness under default timer coalescing.** `SetTimer` uses "the system default timer coalescing" (SetCoalescableTimer: "When uToleranceDelay is set to 0, … SetCoalescableTimer behaves the same as SetTimer"); its size for a thread with no visible window is unverified. Recommendation: accept `SetTimer` as ruled; `a_delay_resumes_its_run_after_its_deadline` bounds it below 1 s and M5 shows it by eye; if late Delays are reported, switch to `SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)` (same feature, same thread). Researcher query if wanted: "Windows default timer coalescing tolerance for SetTimer thread timers (hWnd NULL) in a process with no visible windows, Windows 10/11".
4. **N4 `Window(...)` no longer waits for the window.** Listed under Changed, not Breaking. Recommendation: accept — ruling R2 mandates `ShowWindowAsync`; the documented examples already put a `Delay` after `Window(Restore)`, and the entry tells users to do the same.

## Readiness checklist
Spec
- [x] KS-08 read (`audit.md:308-328`); scenario (OpenUrl inline freezes input) → X7 timing tests + M1. gaps "No cancellation" (`gaps.md:145-147`) → X1/X7 cancel tests + M2-M4, M10. KS-33 Sequence half (`gaps.md:166`) → X8 removal. Critique I1 → D13, X3/X4, M11.
- [x] Every book reference re-located or marked gone (table).
- [x] Goal, non-goals, declared deviations, dependent items stated.
Design
- [x] Every decision has why + rejected alternatives.
- [x] Owner decisions: exact modifiers/`allow_extra_modifiers` untouched; repeat defaults untouched (`repeats_by_default` unchanged); RON unchanged; `log` + sink (executor logs through `log::`, hook path counts); licence untouched.
- [x] Thread named for every new path; the LL callback only decides, moves the job and posts.
- [x] Install/uninstall stay on main; the executor's start/stop on main; the Delay timer lives and is pumped on the executor thread that owns it; no Hold timer moved (2.2).
- [x] No new `SendInput`; sentinel paths unchanged; key-up pairing untouched (engine unchanged).
- [x] N/A sided modifiers: no matching change.
- [x] Reload, `swch off`/`on`, exit during a running macro covered (X1/X7 tests, M2-M4, M10); foreground change: no bump (documented); pending gestures unchanged (2.3).
- [x] Orderings stated; no `static mut`; no `Relaxed`.
- [x] No new module cycle; placement rule in `windows_actions` with a unit test; run logic pure in `core::executor::runs` with fake-clock tests (not key-decision logic, so not `engine`); engine unchanged.
Win32
- [x] Each API: path (with line in the 0.62.2 source), feature present (Cargo.toml:41-53), thread, errors, doc link with the quoted constraint; none unverified.
- [x] Elevation/UIPI/DACL/token inheritance considered (unchanged).
Commits and tests
- [x] Each fix preceded by a red test with a predicted failure (L3, L5, L8, X1, X3, X5, X6, X7, X9, X11); L8, X3, X5, X6, X11 red by assertion; doc-only commits L1, L2, L10, L11; L7 a no-behaviour seam.
- [x] Executor tests use a fake runner and real threads; run logic uses a fake clock; nothing installs a hook or injects input.
- [x] The one real-Win32 test is `#[ignore = "manual: …"]` and listed (M9).
- [x] Each non-red commit passes the gate; the X2 executor is `pub` so nothing is dead before X8.
- [x] README, `default_config.ron`, CHANGELOG, config pages land in the commit that changes the meaning (L4, X4, X8, X10, X12).
Acceptance
- [x] Every acceptance line, and every ruling's test list, mapped.
- [x] Every open question has a recommendation; none blocks.

## Critique response
Answers to `critique.md` (round 1). "Removed" quotes round-1 text verbatim; "Added" names the new text; "Depends" lists the sections whose invariants rest on the change.

**I1 (Important) — synchronous `ShowWindow` on the shared executor; "within 100 ms" overstated. Fixed (ruling R2).**
- D13 (new): `ShowWindowAsync` for the three arms; failure logged; `Close` unchanged; every op has a non-blocking form (none left with a declared blocking cost).
- Win32 table: row "Window steps (R2)" added with the Remarks quote.
- Commits: X3 (red scan listing `sequence.rs:276`, `:279`, `:282`) and X4 (fix, docs).
- Manual checks: M11 (Not Responding window, then `OpenUrl`: the URL opens).
- CHANGELOG — Removed: "- **Changed, Breaking:** macros run one at a time, in firing order, on the executor together with `Launch` and `OpenUrl`: a second press waits for the first run instead of interleaving with it, and a long `Delay` holds up every `Launch`, `OpenUrl` and `Sequence` fired after it." Added: the Fixed (gaps) entry with "a step already running (a `Launch`, an `OpenUrl`) finishes first", and the X4 Changed entry. README `:528-531` and `:704` carry the same wording (touch set).
- Risks: "Hung windows" bullet; "Modal UI" folded into "Blocking steps".
- Depends: D2 Cost (the one remaining serialisation), D5 (what "stops" means), acceptance rows R2.

**M1 (Minor) — nothing proves the new hook-path functions log nothing. Fixed.**
- Removed (old X7): "New `core::windows_actions::tests::the_fast_actions_log_nothing`: for `impl FastAction`'s `fn run(`, `fn press_vk(`, `fn post_message_to_foreground(` (windows_actions.rs) and `fn switch_language_forward(`, `fn switch_language_backward(`, `fn toggle_caps_lock(`, `fn press_key(` (system_function.rs)".
- Added: X11 `logging::tests::the_functions_on_the_hook_path_log_nothing`, one commented list adding `as_fast`, `SystemFunction::execute`, `ExecutorHandle::submit`, `TargetWindow::for_action`/`foreground`, `cancel_pending`, `KeyboardHook::submit`/`cancel_actions`, and the whole non-test `counters.rs`; `fn_lines` fails loudly on a missing signature.
- Depends: D11, acceptance row "hook path never calls `log::`".

**M2 (Minor) — two red-first steps never fail for the stated reason. Fixed.**
- L7 — Removed: "PREDICTED RED: compile error E0425 \"cannot find function `start_logging` in this scope\" (`cargo test`; `cargo build` green)." Added: L7 seam with today's order (green), L8 red by assertion (`assertion failed: !started.get()`, "a second init must not start a writer"), L9 the order fix.
- X3 — Removed (from the single compile-red commit): "`hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` — scans every `.rs` under `src/hook` … would list `src/hook/keyboard_hook_callback.rs:219` on master." Added: X5, the scan alone, red by assertion listing `:219`, before X7 (compile-red) and X8 (removal). The new no-sleep scan (X6) follows the same rule.
- Depends: D12, the commit list's red/fix pairing statement.

**M3 (Minor) — config pages and README places missing from the touch set. Fixed (ruling Q2).**
- Removed (touch set): "| `book/src/config/known-issues.md` | rows 26 `:64`, 28 `:72`, 29 `:73` → fixed on `feat/action-executor` | config page of the fix |".
- Added: rows 28, 29, 36, 60 → Fixed; row 26 → Partly fixed; README `:116`, `:125`, `:415-418`, `:704`, `:837`; `config/README.md:55`; `example.md:99-100`, `:186-189`, `:206-207`, `:221-223`, `:237`; `kinds.md:77`, `:142`; `triggers-conditions.md:41`, `:75`, `:77`, `:79`, `:123`; `matching.md:150` (L4); the non-config pages listed for `impl.md` and the Phase-2 sync. Beyond the critic's list: row 60 (`:128`, "keeps running after `swch off`"), `example.md:99-100`/`:221-223` and `triggers-conditions.md:75-123`, found by grep, say the same falsified things.
- Depends: commits L4, X4, X8, X10, X12.

**M4 (Minor) — after executor death every slow action vanishes without trace. Fixed (ruling R3).**
- D3 — Removed: "`Disconnected` → `Closed` (shutdown; silent, not counted)." Added: `Disconnected` + `stopping` false → `executor_unavailable` + `Unavailable`; with `stopping` true → `Closed`.
- D14 (new): death paths, the drain-count of queued jobs, the whole-loop `catch_unwind`, the ordering argument.
- Q7 — Removed: "7. A `WM_QUIT` reaching the executor (only foreign code on its thread could post one) ends it with an `[ERROR]`. Recommendation: accept — that is `WM_QUIT`'s meaning, and ignoring it could spin `GetMessageW` if the quit state persisted (whether it does is unverified, so the cut does not rely on it)." Added: replaced by R3; the critic's alternative (ignore `WM_QUIT`) is rejected per PostQuitMessage ("it should exit its message loop").
- Test: X1-16 `an_executor_ended_by_wm_quit_counts_every_later_submit`.
- Depends: D2 epilogue, D9, type sketch (`Submitted::Unavailable`), counters text in README/errors.md.

**M5 (Minor) — the Hold half of the split had no automated proof. Fixed.**
- Removed (acceptance): "| Hold on its `hold-timer` follows the split | shared `carry_out` (review: `start_hold_timer` calls `carry_out(get_hook(), ..)`); M6 |".
- Added: X7 `a_hold_runs_its_slow_action_on_the_executor` (GLOBAL_HOOK, FakeRunner executor attached to `get_hook()`, F13 `Hold(1)` `OpenUrl`, key-down via `call_keyboard_callback`, record on `action-executor`, key-up, detach guard), with the stop-and-report clause of the ruling.
- Depends: acceptance row "Hold … follows the split"; D8 (attach/detach is main-thread in the daemon, test-thread under test).

**Critic open question 1 — bounded join vs "joins".** Resolved: declared deviation 2 (Goal) and D8 cite it. Removed: nothing (D8's "`Timeout` → drop the `JoinHandle` (thread detached; process exit ends it) → `TimedOut`" stays); added "(declared deviation 2)".

**Critic open question 2 — Win+L example unverified.** Resolved. Removed (L2): "and `( keys: \"LWin\", action: PressKey(\"L\"), trigger: Hold(500) )` sends Win+L (locks the session)". Added: Ctrl+Esc only; both forms of that snippet (`block_default` absent and `true`) through `cfgcheck` with init + flush, `2>&1`, outputs in `impl.md`.

**Critic open question 3 — CI timing of the 5 ms bound.** Resolved per ruling. Removed (X1-4): "One retry batch if the first max ≥ 5 ms (a scheduler quantum is ~15.6 ms; an enqueue that waits for the runner takes ≥ 500 ms in both batches)". Added: the commit-list timing rule (every one of 20 under 5 ms, one whole-batch retry, no other loosening, a flake goes to the orchestrator), applied to X1-10 and X7.

**Strengths the critic asked to keep — kept.** D1 unchanged. D2/D3 send-then-post and drain-before-wait unchanged (now before every `GetMessageW` and after every job). D6's "`Instant` decides, `checked_add`" kept inside `Runs` and `DelayTimer`; its "the wait pumps" is now stronger: the thread is always in the one `GetMessageW` loop. D8 unchanged. FakeRunner on real threads kept, plus pure fake-clock tests. R-7 still declared.

## Developer addendum

Implemented on `feat/action-executor` (26 commits on `master` @ `7d757a1`, HEAD `6a1db71`). Full report: `D:/tmp/ks-orch/p2-2.1-executor/impl.md`. Line numbers below are at HEAD.

### Critical and Important remarks

| Remark | Ruling | Resolution | Pinned by |
|---|---|---|---|
| critique I1 (synchronous `ShowWindow` on the shared executor; "within 100 ms" overstated) | R2 | RESOLVED. `apply_window_op` calls `ShowWindowAsync` for Minimize/Maximize/Restore; a `FALSE` logs `[WARN] window step failed: ShowWindowAsync did not start {op:?}`; `Close` stays a posted `WM_CLOSE` (`src/data/sequence.rs:246-279`). Every promise of "100 ms" says "a step already running finishes first" (CHANGELOG, README, actions.md §13.1/§14.1). | `data::sequence::tests::a_window_step_never_waits_for_its_window` (red `fd5993e`, green `8e28876`); manual M11 |
| critique_v2 I1 (no drain after a resumed step) | V2-I1 | RESOLVED. `run_loop` repeats {drain the channel; poll the due runs} until a whole pass made no runner call, then arms the timer and waits; the last call before `GetMessageW` is a `try_recv` that returned `Empty` with no runner call after it (`src/core/executor/mod.rs:590-652`). The cut's structure landed first in X2, so the red is by assertion. | `core::executor::tests::a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` (red `a364b77`: left `None`, right `Some("open B")`; green `5ea07d8`) |

### Round-2 rulings

- **V2-I1**: see above. `Runs::start` returns `Calls { made, panics }` and `Polled` carries `calls` (instead of the sketch's `u32` / `panics`), so the loop knows whether a pass made a runner call.
- **V2-M1**: `actions.md` §14.2 last bullet says the focus moves once Windows has handled the minimise and asks for a `Delay` before the next typing step, also after `Window(Restore)` (X4, `8e28876`).
- **V2-M2**: M11 is listed in `impl.md` with its control run on `master`, and the "does not reproduce the hazard" outcome. Not run (no daemon may run here).
- **V2-M3 / N3**: the timer is `SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)` from X2 on (D6 named `SetTimer`). The decision is a pure `plan(armed, next, now, code_ran)` (`src/core/executor/mod.rs:436-446`): the armed timer is kept only when nothing ran since it was armed; after any pass that made a runner call it is killed and made again for the nearest deadline. A message dispatched to a foreign window procedure also counts as "code ran" (critique_v2 M3 names that path too). Pinned by `the_delay_timer_is_kept_only_while_nothing_ran` (X1). The manual timing line is M13 in `impl.md`.
- **N1, N2, N4**: as accepted. N4 is a Changed entry with the `Delay` after `Window(Restore)` advice (X4).
- **Critic open question 1 (loop panic)**: `run_executor` owns the receiver `queue`, the `DelayTimer` and the `Runs` as locals and lends them to the `catch_unwind` closure that runs `run_loop` (`src/core/executor/mod.rs:527-575`); the epilogue after the catch kills the timer, zeroes `active_runs`, drops the runs, drains and counts the queue on `Died`, then drops the receiver, the runner and the apartment. No natural seam for a loop-panic test exists (every runner call is caught per call in `runs.rs`), so the ownership is a review item.
- **Critic open question 3 (M12)**: listed in `impl.md` as manual check M12; if the window does not come back to the front, the X4 Changed entry needs one sentence (open point in `impl.md`).

### Round-1 rulings

- **R1**: resumable runs (`src/core/executor/runs.rs`), one thread timer, no `thread::sleep` or blocking wait (`macros_never_sleep_or_block_in_a_wait`, red `ba3122f` on `sequence.rs:133`, green `a4a2e32`); the four ruling tests: `a_delay_5000_macro_holds_up_no_other_job`, `runs::tests::two_macros_fired_together_interleave_at_their_delays_and_keep_their_order` + `two_macros_fired_together_both_run_in_their_own_order`, `a_cancelled_waiting_run_never_resumes` (both levels), `a_delay_of_u64_max_waits_until_cancelled` + `a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms`.
- **R2**: see I1.
- **R3**: `Submitted::Unavailable`, `Counter::ExecutorUnavailable`, death drain; `an_executor_ended_by_wm_quit_counts_every_later_submit`.
- **M1**: `logging::tests::the_functions_on_the_hook_path_log_nothing` with one commented list (adds `fn wake(`, which `submit` calls); red `37b1847` with the three predicted sites, green `6a1db71`.
- **M2**: L7 seam (`89fcf5e`) → L8 red by assertion (`7827ec8`) → L9 (`62bde43`); X5 alone, red by assertion (`fcce19b`).
- **M5**: `a_hold_runs_its_slow_action_on_the_executor` under `GLOBAL_HOOK`; nothing installed or injected.
- **Q1**: declared deviation 1 (action by value). **Q2**: config pages done; non-config pages listed in `impl.md`. **OQ1**: declared deviation 2 (bounded join). **OQ2**: Ctrl+Esc example only, both forms through `cfgcheck`. **OQ3**: every one of 20 under 5 ms, one whole-batch retry, in both timing tests.

### Deviations from the cut (none changes behaviour beyond the rulings)

1. `SetCoalescableTimer(.., TIMERV_NO_COALESCING)` instead of `SetTimer` (V2-M3), and the pure `plan()` with `code_ran` (above).
2. `Runs::start -> Calls`, `Polled { next, calls }`; `DelayTimer::fired` returns `()` (its bool had no reader).
3. `Apartment` carries `PhantomData<*const ()>`, so it is `!Send` and `CoUninitialize` can only run on the thread that initialised COM.
4. Two commits beyond the list: V2-I1's red/fix pair (`a364b77`, `5ea07d8`) after X2, and `d269caf` (comments cite no lane ruling), which sits on X3's red tree.
5. Tests beyond the cut's list: `the_delay_timer_is_kept_only_while_nothing_ran`, the V2-I1 test, `bump_if_err_counts_only_an_error` (named in X12). `FakeRunner::with_eat_wakes`; `next_record`, `wait_until`, `WAIT` moved to `core::executor::testing` in X7 and shared by the executor and hook tests. X7 also updates the existing mask test to `carry_out(&hook, ..)`.
6. The report line always ends with ` longest_callback_us=<µs>`, so the existing report strings gained that field (X10) and ` post_message_failed=0` before it (X12).
7. Doc placement: README's falsified thread-safety line was `:836` (the cut said `:837`); `actions.md` §13.7's examples drop the obsolete `Sequence([OpenUrl(..)])` wrapper (validated with `cfgcheck`); `errors.md` merges the three dropped `[ERROR]` lines into one "no longer printed" row (X12); `known-issues.md` "Where" cells name files without pinned links.
8. Predicted red line numbers drifted with earlier commits; the content matched: X3 `sequence.rs:277/280/283` (cut 276/279/282), X6 `sequence.rs:133` (cut 129), X11 `windows_actions.rs:278`.
