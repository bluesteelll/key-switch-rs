# Architecture critique: p2-2.4-watchdog, roadmap 2.4 Hook watchdog (KS-08) (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 3, Minor 7)
Plan: D:/tmp/ks-orch/p2-2.4-watchdog/cut.md (703 lines)   Spec: D:/tmp/ks-orch/p2-2.4-watchdog/spec.md
Code read at: `master` `fa20b30` (D:/key-switch-rs/key-switch-rs, read only)

Read in full: cut.md, spec.md, src/hook/{keyboard_hook_callback,keyboard_hook,main_thread (loop, drain, serve, pump, with_main, P1 test),control_window,ipc_server,mod}.rs, src/core/app.rs, src/core/executor/mod.rs (start/stop), src/logging/{mod,counters,writer}.rs, src/platform/{desktop,timers}.rs, src/engine/{mod (reset, on_key_down),keys (resync_from_os)}.rs, src/engine/testing.rs (unseen steps), src/bench.rs (R12), src/ipc.rs, src/bin/swch.rs, tests/ipc_roundtrip.rs, Cargo.toml, CLAUDE.md (on disk), roadmap 2.4, audit KS-08, gaps.md:155/164/232, the 2.3 results and rulings. About 25 of the cut's `file:line` references were spot-checked; all were right.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. The watchdog loop can lose its tick for good, and then it never reinstalls
Where: cut §D5 loop (lines 224-237), §D3 `raw_event` (111-112) and tick step 4 (128); src/hook/main_thread.rs:48-49, :331-334, :348-382, :651-654; src/platform/timers.rs:124-127.

Problem: two pieces of state say whether a tick is pending, and nothing keeps them in step.
- The decision has `ticking`. `raw_event` asks the thread to arm a tick only for the first event after `Idle`.
- The thread's `WaitableTimer` is set only from a `tick` verdict, and the thread calls `tick` only "if the tick is due by `Instant::now()`" (step 4).
- The timer is a synchronization timer: the wait that takes its signal also resets it (timers.rs:29-31).

Two things leave `ticking == true` with no timer armed, and the plan handles neither:
- A timer signal that wakes the loop just before `Instant::now()` reaches the due instant (`TickAt`). The house treats such signals as real: "The clock decides: an early, late or stale signal costs one drain" (main_thread.rs:48-49), and timers.rs:124-127 promises no early signal only "by its own count". The main loop survives an early signal because `fired()` clears `armed` and `plan_wake` sets the timer again on every pass (main_thread.rs:331-334, :348-382, :651-654). The watchdog loop in D5 has no such step.
- A `WaitableTimer::set` that fails. D5 and the Win32 table give no rule for it (only `WAIT_FAILED` has one).

Consequence: the next pass waits `INFINITE` on messages alone. Every later `WM_INPUT` calls `raw_event`, which returns `None` because `ticking` is still true, so no tick is ever armed again. From then on the watchdog only counts. A later hook loss is never repaired, whether it comes from the debug hold or from a real `LowLevelHooksTimeout`. `watched` stays true, so `swch status` keeps saying `hook ok`. That is the KS-08 symptom this lane exists to remove, back without a trace.

Confidence: CONFIRMED that the plan has the gap (§D5 steps 1-4 against main_thread.rs:331-382). PLAUSIBLE for how often an early signal or a failed set happens.
Breaks: KS-08 (the lane's goal); R10 (loop shape); D1's claim "in the house shape".
What is needed:
- One source of truth for the pending tick, applied to the timer again on every loop pass, as `plan_wake` does after `fired()`.
- A stated rule for a failed set. For example, the watchdog ends with `watched = false` and a log line, so `swch status` reads `hook not watched` instead of `hook ok`.
- A test without a hook that fails if a timer wake that comes before the tick's due instant loses the tick. For example, a pure planning function like `plan()` (main_thread.rs:310-320) with unit tests, or a synthetic early wake.

### I2. "Never two hooks, never none" has no test, and two tests reach the real `SetWindowsHookExW` behind the guard they are testing
Where: cut §D4 (159-164, 178-186), Commits 11 (507-517), Acceptance mapping row "Reinstall never double-installs and never leaves none" (609); src/hook/keyboard_hook.rs:141-160, :233-270.

Problem:
- The spec's rule "a reinstall must never double-install the hook or leave a window with no hook" lives only in `reinstall_keyboard_hook`, which has no seam. The rule is: set the new hook, `swap`, then unhook the old one; on `Err`, keep the old handle; on a null handle, make no Win32 call.
- Each test in mapping row 609 has the same gap:
  - `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing`, `a_reinstall_that_finds_no_hook_resets_nothing` and `a_reinstall_resets_the_engine_and_keeps_the_macros` replace the whole function with the fake `replace` closure.
  - The two `NotInstalled` tests exercise only the null guard.
  - M1 cannot observe call order.
- The two `NotInstalled` tests call the real function: `a_reinstall_of_a_hook_that_was_never_installed_installs_nothing` directly, and `a_reinstall_request_for_a_hook_that_is_not_installed_installs_nothing` through `serve` on a running loop thread. The null check under test is therefore the only thing between `cargo test` and `SetWindowsHookExW(WH_KEYBOARD_LL, …)`.

Consequence:
- (a) Regressions ship green.
  - Unhook-first opens a window in which Windows' input thread finds no hook of ours, and keys pass unremapped. The cut's "no message call in between" argument (§D4 180) protects only the set-first order.
  - A failed set that also nulls the handle is worse. Every later request then answers `NotInstalled`, which D4 does not log. The watchdog never repairs again.
- (b) Safety. If a developer slip breaks the null check, or a tester mutates it (the natural way to check that these tests catch the guard), `cargo test` installs a live global LL hook on the owner's desktop:
  - In the direct-call test, the hook sits on a thread that never pumps. Every key the owner types stalls until `LowLevelHooksTimeout`.
  - In the `serve` test, the loop thread pumps. The owner's keys then run through `keyboard_hook_callback` with whatever bindings the concurrent tests have put on `get_hook()`. This lasts until that thread ends. The non-negotiable safety rule forbids exactly this.

Confidence: CONFIRMED (the structure: cut §D4 and Commits 11 against keyboard_hook.rs:141-160). The live-hook outcome needs a broken or mutated guard.
Breaks: spec ("never double-install … or leave a window with no hook"); Safety rules (no global hook from tests); "an acceptance criterion without a proof".
What is needed:
- Put the Win32 install and unhook calls behind a seam, so that unit tests with fakes pin both the null guard and the order. The order means: set before unhook, the old handle kept on `Err`, and `OldHook::Gone` on a failed unhook.
- Make sure no test can reach the real `SetWindowsHookExW` whatever the guard does.
- Name the tester's mutations for the guard and for the order, so that they run against the fakes only.

### I3. Watchdog start and stop make main wait while the keyboard hook is installed and not pumped
Where: cut §D5 `App::run` (244-247), §R10 (655-658); src/core/app.rs:196-199 (the executor starts before the hooks), :221 (`install`), :285-297 (the loop ends, then `uninstall`, then the control window and the executor); CLAUDE.md:100-109 (principle 6: the executor starts before the hooks and stops after they are removed; the control window is dropped after `uninstall`); D:/tmp/ks-orch/p2-2.3-control-window/rulings.md:97-98 (V2-m4: "so no RPC or IME teardown runs while the hook is live").

Problem:
- `HookWatchdog::start` waits up to `START_TIMEOUT` = 5 s for the thread to be ready. It runs after `hook.install()` (app.rs:221) and before `main.run`. Being ready means the thread was spawned and has run `CreateWindowExW`, `RegisterRawInputDevices` and `CreateWaitableTimerExW`.
- `stop` waits up to `STOP_TIMEOUT` = 1 s for the thread's `RIDEV_REMOVE` and `DestroyWindow`. It runs after `main.run` and before `hook.uninstall()` (:292). The cut's own warning "did not stop within 1 s" expects that this can happen.
- During both waits the `WH_KEYBOARD_LL` hook is installed and main makes no message call.

Consequence:
- At startup, a key typed while the watchdog thread is still starting waits for main until `LowLevelHooksTimeout`. Windows may then silently remove the brand-new hook: KS-08 at startup. The thread can be slow to start at logon, when 4.2 background mode starts the daemon, or under load.
  - The watchdog repairs the hook only once it is up.
  - If `start` then times out, the watchdog is off ("not reinstalled until the daemon restarts") with a dead hook.
- At exit the same happens for up to 1 s.
- Neither placement buys anything:
  - Once `drop(main)` has run, a request fails with `ShuttingDown` (main_thread.rs:477-478).
  - A late request finds a null handle and returns `NotInstalled` (D4).
  - So the watchdog can safely outlive `uninstall`. It also follows the executor's pattern (D5 "Why") without following the executor's placement.

Confidence: CONFIRMED (placement). The stall needs a slow start or stop: PLAUSIBLE.
Breaks: R10 (nothing that can block runs between install and the loop, or between the loop's end and `uninstall`, while the hook is live); CLAUDE.md principle 6; ruling V2-m4.
What is needed: no wait on another thread while the hook is live outside the loop. For example, start the watchdog before `install` and stop it after `uninstall`, next to `drop(control_window)`. Moving the start earlier needs a stated rule for raw input that arrives before the hook is installed. Without one, the first request can reinstall a hook that was installed seconds earlier and is healthy.

## Minor (improvements, not blockers)

### M1. One unexplained raw event keeps the watchdog ticking 4 times a second until the next key the hook sees
Where: cut §D3 tick steps 3-4 (124-128), `answered` / `declined` (116, 132-136), decision row 592 (`a_single_raw_event_without_a_callback_asks_for_nothing`), row 588, D1 Cost (75: "Idle: no wake-ups").

Problem: `evidence` resets only on `dc > 0`, on an answer or on a decline. A decision needs `evidence >= 2`, and step 4 keeps returning `TickAt` while `evidence > 0`. Traced: one raw event at 0 with no callback gives `evidence = 1` at 250 ms, the same value at every later tick, and `TickAt` forever. Row 592 builds exactly this state and checks only that no request is made.

Consequence: after such an event, for example a key the hook misses in the R-5 case or any single raw event the hook does not see, the thread wakes every 250 ms for as long as the machine stays idle. If that event is the last input of the day, this lasts all night. It costs nothing on the key path and injects nothing, but D1's cost claim and row 588's "no TickAt after" are wrong for this state.
Confidence: CONFIRMED (trace of the stated rule).
Direction: give ticking an end that does not wait for the next callback, for example evidence that ages out with `ACTIVE_FOR`, and make row 592 assert `Idle`.

### M2. A failed desktop check declines every request without a trace
Where: cut §D5 step 4 (235), Win32 table row "is input on this desktop" (405).
Problem: when the check fails (`Err`), D5 calls `declined(now)`, which drops the evidence. Nothing logs or counts that.
Consequence: if `GetUserObjectInformationW(UOI_IO)` keeps failing on the watchdog thread, no reinstall ever happens and `swch status` still says `hook ok`. The two possible mistakes do not cost the same:
- treating `Err` as "input is here" costs at most one needless reinstall;
- treating it as "input is elsewhere" leaves a dead hook.
Confidence: PLAUSIBLE.
Direction: choose the default with that difference in mind, or at least count or log a declined decision.

### M3. "hook reinstall failed" stays after the hook works again
Where: cut §D7 (270-276).
Problem: only `record_reinstall` clears `last_attempt_failed`. After `Failed`, the old hook is kept. If callbacks then resume (the old hook was alive), `attempts` resets and no further request comes.
Consequence: `swch status` reports `hook reinstall failed` for the rest of the daemon's life while the hook works. A 4.3 or 4.6 consumer would raise a false alarm.
Confidence: CONFIRMED (the rule as stated).
Direction: clear the flag, or let it age out, once a callback is seen after the failure; or say why not.

### M4. Tests interfere through the global counters
Where: cut Commits 5 (484-487), 19 (559-560); src/hook/keyboard_hook_callback.rs:1150 (`COUNTERS.take()` under `GLOBAL_HOOK`), :829-832; src/logging/writer.rs:168-171, :225.
Problem:
- `a_refused_reentry_is_counted_in_main_state_refused` reads `COUNTERS.get(MainStateRefused)` before and after, and does not hold `GLOBAL_HOOK`.
- `remapping_off_keyboard_callback_returns_before_it_times_itself` resets every slot with `take()`, under `GLOBAL_HOOK`.
- After D8 other tests' refusals bump the slot, so `before` is usually above 0.
Consequence:
- A `take` between the two reads makes `after - before` underflow. That is an overflow panic in the debug test build: a rare flake.
- The same family: if the commit-19 panic test gives its writer the global `COUNTERS`, every `Flush` takes them (writer.rs:168-171, :225). That can reset `longest_callback` between the calls and the assertion of `the_keyboard_callback_records_how_long_it_ran`.
Confidence: CONFIRMED (code). The race is rare.
Direction: let the new test take `GLOBAL_HOOK`, or use a saturating delta. Name leaked counters for every writer the tests start.

### M5. No decision row pins "evidence is checked before this interval's raw events are added"
Where: cut §D3 (130), decision table (581-595).
Problem: D3 says the one-tick age absorbs "raw input that arrives before its own hook call". No row exercises that rule. In every row with callbacks, `quiet_since` is recent, so `decide` is false whatever the order.
Consequence: an implementation that adds the raw events before it checks passes every row. Then, after at least 1 s of idle, a key whose two raw events land just before a tick and whose callbacks land just after sends a request. Main answers `Alive` and logs "answered before its reinstall": a needless request and log line, though no reinstall.
Confidence: CONFIRMED (no such row).
Direction: add a row: 1 s of idle, then a key down and up whose raw events land 1-2 ms before a tick and whose callbacks land 1-2 ms after: 0 requests.

### M6. The system-side cost of the raw input registration is neither stated nor measurable by R14
Where: cut Goal (41: "The key path gains one atomic increment per event"), D1 Cost (75), D11 R14 (357); CLAUDE.md:156-158 (latency decision: every cut states its per-event effects).
Problem:
- With the registration, Windows builds a `WM_INPUT` for this process, queues it and wakes the watchdog thread on every key. Whether that work comes before or after the LL hook call (Q6) decides whether it is on the key-to-action path.
- R14 runs `start_synthetic`, so it measures only the heartbeat and the cache effects. No row or manual check can see the system side.
Consequence: the acceptance line "no unjustified regression with the watchdog active" is proven for the callback only.
Confidence: PLAUSIBLE. The cost is probably microseconds, and LowLevelKeyboardProc ("about to be posted into a thread input queue") suggests it comes after the hook.
Direction: state this per-event work and where it sits, and have the orchestrator accept it explicitly.

### M7. M1 cannot run while the owner's release daemon runs
Where: cut Manual checks M1 (622); src/core/app.rs:190-191; src/hook/ipc_server.rs:111-117 (`FILE_FLAG_FIRST_PIPE_INSTANCE`).
Problem: M1 starts a debug daemon. While the release daemon holds the pipe, the debug daemon stops at once with the pipe error.
Consequence: the manual acceptance fails at setup. If the tester misses the fatal line, the `swch status` poll answers from the release daemon (`reinstalls: 0`), which could be misreported as R-3 ("Windows kept the hook").
Confidence: CONFIRMED.
Direction: start M1 with `swch exit`, and let M2-M8 say they start the release daemon again.

## Previous findings
Round 1: none.

## Strengths to keep
- D1: raw input on a dedicated thread with its own message-only window. The Learn quotes support it, and the argument that the main thread pays nothing per key holds by construction. R15 is honestly labelled as evidence that cannot overturn D1.
- Thread affinity is in the types. The watchdog holds only `&'static HealthState` and a request handle, so it cannot touch a hook. `install`, `uninstall` and the reinstall stay serialized on main in `serve`, outside `with_main` and outside every callback (KS-01).
- Rule 3 of D3: main dispatches pending sent messages, then reinstalls only if the heartbeat has not moved. This makes a busy main harmless. It also covers older systems where `LowLevelHooksTimeout` is above 1 s, because the hook calls still waiting are answered and give `Alive`.
- The new-hook-first order and the "never install what is not installed" guard. A request that arrives after `uninstall` or `WM_ENDSESSION` falls into `NotInstalled` (control_window.rs:222-231 runs inside the same message calls).
- D13: `resync_from_os` is the right reset. The owner decision on lost key-ups (CLAUDE.md:154) holds after a hook loss, and the red test with `HookReinstalled => {}` fails for the right reason (engine/mod.rs:659, keys.rs:126-141).
- `swch status` stays responsive while main is stuck, because IPC workers read atomics. The reply keeps the prefix `OK: running (N bindings live` (swch.rs:77-81, :146-151).
- D9: the panic hook routes through the writer and never chains to the default hook. The 250 ms budget is stated and shared by the second (nounwind) panic.
- D10: the debug hold exists only in debug builds, has a structural scan, and leaves no per-event cost in release.
- The re-location table is accurate.

## Open questions for the architect
1. **The cut's Q6 decides whether its residual R-1 and its manual check M6 matter at all.**
   - LowLevelKeyboardProc says the hook runs when an event "is about to be posted into a thread input queue" (https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc).
   - A single forum report says that a key blocked in a `WH_KEYBOARD_LL` hook does not reach raw input on Vista and Win7 (https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/). It is a single report, and a reply doubts the test.
   - If keys swallowed by another program's LL hook produce no `WM_INPUT`, R-1 cannot happen and the cut's M6 (AutoHotkey) has nothing to observe. It would also answer this critique's M6: the system-side work comes after the hook. The researcher query in the cut's question 5 settles it.
2. **Timed-out hook calls delivered late.** Windows may still deliver a call it gave up on, once main pumps again. If so, `dispatch_sent_messages` makes the heartbeat move, and a removed hook answers `Alive` once. Detection then takes one more round (about 1.25 s). That fits R-3 but stretches M1's 2 s. The M1 log timestamps settle it: the time from the reinstall line back to the "answered before its reinstall" line, if any.
3. **A default IME window on the watchdog thread.** Does the watchdog's message-only window get one, as 2.3's control window did (2.3 critique OQ1; the `EnumThreadWindows` test there)? It bears on the stop time in I3. The 2.3 `EnumThreadWindows` test pattern settles it.
4. **The gesture-path scan.** `the_gesture_path_starts_no_thread` (keyboard_hook_callback.rs:1273-1325) lists `src/hook` without recursing. `health.rs` and `debug_hold.rs` are scanned, so `use std::thread` in `debug_hold.rs` fails the gate. `watchdog/` escapes the scan only because it is a folder. Say whether that exclusion is intended, as it is for `ipc_server.rs` and `config_watcher.rs`.

## Absent topics (walked, not applicable)
- §3 Matching model, §7 Config and hot reload: no config type, schema, `default_config.ron` or matching change. The cut says so, and no config page changes.
- §10 Input synthesis: no new `SendInput`, and the sentinel is unchanged. The scan `the_watchdog_injects_no_input` keeps it that way. This also covers the spec's 10 minutes of idle with no injected input.
- §9 Process boundary: the pipe DACL, the single instance and token inheritance are unchanged. The registry is only read, by hand, in M1.
- §8 Shared state: no `static mut`, no new lock on the key path. The orderings follow the house rules. D7's count and time pair is correct: `Release` store, `AcqRel` RMW, `Acquire` loads.
- §1 Hook-thread budget: the release build adds one `fetch_add` before the switch gate. The debug build adds one atomic load. The callback gains no allocation, `log::` or Win32 call.
