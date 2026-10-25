# Lane spec: p3-3.4-groundwork - Core groundwork: one wait and pump, the property net at 10,000 scripts, the shutdown guard

Branch: refactor/core-groundwork

Roadmap item 3.4 (book/src/roadmap.md 'Core consolidation', :708-725 at 252daef) and the Phase 2 follow-ups that name it
(:982, :983, :987), plus FU-1 (:997), the lexical installer scan (:1002) and two 2.5 notes. No behaviour change in normal
operation: this lane is the safety net for the five engine-changing lanes after it, and it stops the flaky real-clock
test from failing their gates (it failed 2.5's final gate run: p2-2.5-latency-trim/gate_test_final.log:421, :887,
"861 passed; 1 failed"). Binding: lowest key-to-action latency; no key may ever stick. Scope report:
D:/tmp/ks-orch/phase-3/scope/3.4-core.md (sections 2, 8, 9, lane C).

1. One wait and one pump in a new `src/platform/message_loop.rs`, used by:
   - `MainThread::run`: the wait at `src/hook/main_thread.rs:812-819`, the pump at :1008-1027;
   - the executor loop: the wait at `src/core/executor/mod.rs:648-655`, the pump at :678-701.
   Each loop keeps `QS_ALLINPUT` and `MWMO_INPUTAVAILABLE`, its own wake id (`MAIN_WAKE`, main_thread.rs:103; `WAKE`,
   executor/mod.rs:69), its own `WM_QUIT` handling (main returns quit, :1013-1015; the executor logs and dies,
   :683-689) and main's E5 awake-stretch check before its wait (:802-806, :820). The watchdog keeps its ranged pump
   (`src/hook/watchdog/mod.rs:806-826`) and may use the shared wait primitive with its own timeout and mask.
2. A test that pins `MWMO_INPUTAVAILABLE` for the executor, the twin of
   `hook::main_thread::tests::a_message_posted_behind_a_wake_is_dispatched_without_more_input` (main_thread.rs:1369).
3. `MainThreadHandle::{set_enabled, reload, request_reinstall}` (main_thread.rs:602-659) share one send, wake and
   optional-wait helper.
4. R-7 (roadmap :982): a comment at executor/mod.rs:254-263 records why no post can reach a reused thread id: every
   post runs on main before the detach (`src/core/app.rs:52-60`, :309-322), and after a mid-session death the open
   join handle (executor/mod.rs:271) keeps the id reserved (Raymond Chen, The Old New Thing, 2014-08-22). The row
   closes at the phase book sync.
5. The property net (`src/engine/properties.rs`, `src/engine/testing.rs`):
   - the press-symmetry properties (Strict and FocusWhole, :398-400, :434, :454) run at least 10,000 generated scripts
     in total (today SEEDS 64 x 9 configs = 576 each, :98, :126);
   - the harness attributes timer fires (`advance_ms`, testing.rs:624-645) to their press and checks that each press
     fires at most one of its Tap, Hold and DoubleTap (KS-25) and that repeats fire only repeating actions (the
     Phase 1 follow-up 'the harness checks on_key fires on repeat steps, not timer fires'; `check_repeat_fire`,
     testing.rs:959-973);
   - a generator case that takes the D10 path of item 2.5 (a repeat with no record that a binding swallows,
     `LastDown::RepeatSuppressed`, `src/engine/keys.rs:64-70`), so that D10 rests on more than one scenario
     (p2-2.5-latency-trim/results.md:174: the `Unnoticed` property passes with D10 reverted);
   - an engine purity scan replaces roadmap acceptance A1, which cannot pass literally (Grep `Mutex|thread::|unsafe`
     in src/engine matches `mod.rs:31-32` doc lines, the `#![forbid(unsafe_code)]` attribute at `mod.rs:251` and the
     doc path at `schedule.rs:6`).
6. `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` (executor/mod.rs:921-940) is made robust
   without hiding a tick-granular timer (a fake clock for the logic plus a looser real-clock bound, or a retry on a
   loaded machine; the cut chooses and argues it).
7. FU-1 (roadmap :997): an uninstall guard in `App::run` (`src/core/app.rs:295-322`). Today a main-thread panic
   outside a callback unwinds through the `Drop` of the executor (bounded 2 s stop, executor/mod.rs:358-361 via
   app.rs:63-67) and of the watchdog (1 s, watchdog/mod.rs:549-553) while the keyboard hook stays installed and no
   thread pumps, so every key event system-wide waits for `LowLevelHooksTimeout` meanwhile. The guard removes the hooks
   first on every exit path; a normal exit keeps today's order (`uninstall`, control window, watchdog, executor).
8. One lexical source-scan helper for the scan tests, which handles character literals, strings over several lines
   and block comments, and reads `tests/` recursively. It closes the roadmap follow-up on
   `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests` (src/hook/installer.rs:580; roadmap
   :1002), and the purity scan, the two `MainThread::run` shape scans (main_thread.rs:2088-2151), the executor scan
   `macros_never_sleep_or_block_in_a_wait` (executor/mod.rs:1481-1522) and the hook-path no-`log::` scan
   (`src/logging/mod.rs:405`) use it. p3-3.1-input-module's `SendInput(` scan will use it too.

Acceptance:
- Unit (source scan): outside tests, `MsgWaitForMultipleObjectsEx(` occurs only in `src/platform/message_loop.rs` and
  `src/hook/watchdog/mod.rs`. The rewritten executor and `MainThread::run` shape scans still fail on a planted sleep,
  a second wait and a broken E5 order.
- Real-thread unit test (own queue, message-only window, no hook, no injection): a message posted behind a `WAKE` on
  the executor thread is dispatched without more input; red with the flag removed; the main loop's twin stays green.
- Unit: the `set_enabled`, `reload` and `request_reinstall` tests stay green, and each error path (NoAnswer, Busy,
  ShuttingDown) keeps its test.
- Unit (source scan): non-comment, non-test lines of `src/engine/*.rs` name no `Mutex`, `RwLock`, `thread::`,
  `unsafe` (the `forbid` attribute allowed), `log::` or `Instant::now`; red when one is planted.
- Unit (source scan): the installer scan is red on a call planted after a `'"'` character literal, after a
  string that spans several lines and after a block comment, and on a call in a nested `tests/` file.
- Property: the symmetry properties run >= 10,000 scripts; the timer-fire check is red under a planted double fire
  (for example a key-up that also fires a Hold left to its timer); the D10 case is red with D10 reverted (the 2.5
  mutation X4). The `engine::properties` run time is stated for parent and HEAD in impl.md.
- Unit: the delay test passes 20 runs in a row on HEAD (runs listed in test_r1.md) and still fails on a planted wait
  on the system tick.
- Unit (FakeInstaller, FakeRunner): with a panic injected into the main run through a test seam, the uninstall is
  recorded before the executor's and the watchdog's stops. If the cut finds no seam without touching the hook path,
  a review check plus a manual stub `#[ignore = "manual: ..."]`.
- Bench (counted apart): R7, R8 (with `KS_BENCH_LOAD=1`), R13, R14 and R14c, R15 and R16 within the CLAUDE.md rules,
  parent against HEAD; no other row regresses, the R19 family of p3-3.5-budget-bench included.
- Manual M-C1 (owner, later batch): start the daemon, type, `swch off`/`swch on`, run a `Sequence` with a `Delay`, a
  `Launch` and an `OpenUrl`: nothing hangs, the macros run, the keys stay normal.
- Gate green.

Latency: no new call, allocation, lock, Win32 call, timer or wake-up on the key path. The hook calls are dispatched by
the same `PeekMessageW` in the same order; the helper is `#[inline]`. The shutdown guard runs only at exit. The
property scale-up and the scans are test-only (state the added test time).

Docs: README module tree (`platform/message_loop.rs`); CHANGELOG only for a user-visible change (the FU-1 guard: Fixed,
"a main-thread panic removes the keyboard hook before the bounded stops", roadmap 3.4/FU-1). Book-sync list in impl.md
with file:line: the roadmap rows R-7, the shared wait and pump, `MainThreadHandle::reload`, FU-1, the installer scan
and the Phase 1 harness row; architecture.md (thread model, the shared loop); gaps.md test-coverage note.
