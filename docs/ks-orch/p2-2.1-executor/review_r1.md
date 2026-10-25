# Code review: p2-2.1-executor @ 6a1db71
APPROVED (Critical 0, Important 0, Minor 1)

Branch `feat/action-executor`, 26 commits on `master` @ `7d757a1`. Code read through `git show 6a1db71:<path>` and
`git diff master...feat/action-executor`; every touched source file read in full (executor `mod.rs`, `runs.rs`,
`testing.rs`, `windows_actions.rs`, `app.rs`, `sequence.rs`, `keyboard_hook.rs`, `keyboard_hook_callback.rs`,
`counters.rs`, `logging/mod.rs`, `system_function.rs`, loader hunks, `main.rs`), plus CHANGELOG, README and the
config-page hunks.

## Gate: fmt ✅ · clippy ✅
On a `git archive 6a1db71` extract (`D:/tmp/ks-orch/p2-2.1-executor/review-r1/tree`, own target dir), not the shared
work tree: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0 ("Finished `dev`
profile"). `cargo test` not run (tester's gate).

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. `Executor::thread` is public API without a doc comment (section 7)
**Where**: `src/core/executor/mod.rs:317-319`  **Confidence**: CONFIRMED (read)
**Problem**: `pub fn thread(&self) -> &Thread` is the only public item of the new module with no `///` line. Every
other item in the module has one.
**Failure**: none at run time. `cargo doc` shows an undocumented method on a public type, which breaks the
"doc comments on public API" rule.
**What to do**: one line, for example "The executor thread, so that tests can check where a job ran."

## Checklist walked (items looked at, no reachable failure)

**Plan, roadmap, rulings**
- Spec and acceptance: `rg "\.execute\(\)" src/hook` is empty (git grep at 6a1db71, and X5). 100 Sequence runs happen on
  one thread (`mod.rs:745-763`). Delay(10000) stops within 100 ms (`:765-789`, plus `swch off` and reload at
  `keyboard_hook.rs:580-590`). The 20 x 5 ms enqueue checks follow the ruling: all 20 must pass, with one retry of the
  whole batch (`mod.rs:1016-1030`, `keyboard_hook_callback.rs:817-831`). A full queue drops and counts (`mod.rs:1069-1102`).
- R1: resumable runs (`runs.rs:125-201`), one `SetCoalescableTimer(.., TIMERV_NO_COALESCING)` (`mod.rs:474`, V2-M3),
  no `thread::sleep` or blocking wait (X6 scan). All four ruling tests are present. Runs go oldest first, and a stretch
  of steps between two Delays stays whole (N2).
- R2: `ShowWindowAsync` for Minimize/Maximize/Restore and a posted `WM_CLOSE` for Close; a failure logs on the executor
  (`sequence.rs:246-279`). M11 is listed.
- R3: `Disconnected` + `!stopping` is counted as `Unavailable` (`mod.rs:228-232`), and the death epilogue counts the
  jobs still queued (`:565-571`).
- V2-I1: the last thing before `GetMessageW` is a `try_recv` that returned `Empty`, with no runner call after it. The
  inner loop repeats {drain; poll} until a full pass makes 0 calls (`mod.rs:607-637`). The red at `a364b77` is
  structural: a poll with calls was followed directly by `arm`/`GetMessageW`.
- Q1 (action by value), OQ1 (bounded join + `[WARN]`), OQ2 (Ctrl+Esc only, cfgcheck outputs in `impl.md`), OQ3
  (timing rule), M1 (one commented list, `logging/mod.rs:391-434`), M2 (L7 seam, L8 red by assertion, L9 fix, checked
  at `7827ec8`), M5 (Hold test under `GLOBAL_HOOK`, `keyboard_hook_callback.rs:846-878`), N1/N2/N4, loop-panic
  ownership (`mod.rs:547-574`): all carried out.
- Leftovers L1-L11 checked against the Phase-1 reports. 1.3 P1 wording and P2 Breaking sub-bullet are in. 1.2 m1:
  the warning is pushed only for hotkeys that get a blocker (`loader.rs:401-411`). 1.6 m3: the warning goes into the
  returned list (`loader.rs:286-289`). 1.7 m1 and m2 are fixed, and 1.5 m1 (`main.rs:64-68`) and the L11 SAFETY comment
  (`app.rs:301-306`) are in place.
- CHANGELOG: Added (counters), Changed (Window steps, post errors), Fixed (KS-08, cancellation, KS-01/KS-33 edits),
  each citing roadmap 2.1. README and the config pages the Q2 ruling lists are updated. The non-config pages are handed
  to the Phase-2 sync in `impl.md:114-123`.

**Unsafe (section 2)**: every new `unsafe` block has a concrete `// SAFETY:`. The list: `GetForegroundWindow`
(`mod.rs:81`), `PostThreadMessageW` (`:258`), `CoInitializeEx`/`CoUninitialize` (`:375`, `:394`), `SetCoalescableTimer`
(`:474`), `KillTimer` (`:499`), `PeekMessageW`/`GetCurrentThreadId` (`:539`, `:542`), `GetMessageW` (`:653`),
`TranslateMessage`/`DispatchMessageW` (`:679`), `ShowWindowAsync`/`PostMessageW` (`sequence.rs:264`, `:275`), and the
test-only `PostQuitMessage`, `PeekMessageW(WAKE, PM_REMOVE)` and `CoGetApartmentType`. `HWND` travels as `isize`
(`TargetWindow`) and is never dereferenced. `Apartment` is `!Send` (`PhantomData<*const ()>`), so `CoUninitialize`
runs on the thread that initialised COM. No new FFI callback.

**Atomics (R4)**: `epoch` uses `fetch_add AcqRel` and `load Acquire`; `stopping` and `active_runs` use `store
Release` and `load Acquire`; the gauge uses `fetch_max`/`swap AcqRel` and `load Acquire`. None publishes data, since a
job travels through the channel. No `Relaxed`, `SeqCst` or `static mut` in the diff. The stop-time
`Unavailable`/`Closed` split is declared best-effort (D14, R-7).

**Win32 and threads (section 4, R10)**:
- Hooks are still installed and removed only on main.
- The executor creates its queue (`PeekMessageW`) before the `ready` handshake, so no `WAKE` can come before it.
- The timer is created, killed and pumped on the executor thread. A stale, early or reused-id `WM_TIMER` only causes
  another poll (`fired` kills our id, and `plan` re-arms from `Instant`).
- `App::run` order: bind → executor start (fatal before any hook) → `update_bindings` → attach → install → serve →
  loop → uninstall → detach + stop. Every `?` return after the attach stops the executor through `AttachedExecutor::Drop`.
- Stop follows uninstall, so a macro's `SendInput` never waits on an unpumped hook thread. The join is bounded at 2 s.
- `logging::flush` still runs in `main` after `run`.
- No new `SendInput` site. Every builder keeps `injected_sentinel()` (R2).

**Hot path (section 5, R1)**:
- New work per event: two `Instant::now()` and one `fetch_max`.
- Per slow fire: an `as_fast` match, `GetForegroundWindow` (Sequence only), one `ArcSwapOption::load` (the main
  thread's arc-swap node already exists from `bindings.load()`), one atomic load, `try_send` into a preallocated slot
  (std array channel: CAS, and the waker mutex only when a receiver is parked, which never happens because the
  executor only `try_recv`s), and one `PostThreadMessageW`.
- No `log::` (X11 list and the whole-file scan), no lock, no new allocation. On `Full` the job is dropped on the hook
  thread (declared, D3 Cost).
- No panic path: exhaustive matches, `unwrap_or` saturation, `Instant::elapsed` saturates.
- `code < 0` and remapping-off passes come first, and there is exactly one outcome (`:345-383`).

**Correctness (section 6)**:
- Suppression is unchanged: `suppress` is read before `carry_out`, and the engine is untouched.
- Epoch races: an action decided before a bump but submitted after it runs (declared, D5 Cost).
- Delay(0) yields and loops through `Due` without waiting. Every pass moves `next` forward, so no spin.
- `u64::MAX` Delay: `checked_add` gives `At(far)` or `WhenCancelled`, and the timer is clamped to
  `USER_TIMER_MAXIMUM`.
- `Arming::Failed` drops only the timed runs.
- Panics are caught per call (`runs.rs:207-213`) and for the whole loop (`mod.rs:552-560`), each ending in the epilogue.
- Stop or cancel while a runner call blocks: `live` is checked before every step and `stopping` before every wait.

**Tests**:
- No test installs a hook, injects input or runs a real action. Only `DoNothing` runs inline. `Win32Runner` appears
  only in the `#[ignore = "manual: …"]` test. The mask is counted under `cfg(test)`.
- No test calls `logging::init` or `attach_file`.
- `GLOBAL_HOOK` serialises every `get_hook()` test, and no other module touches the global hook.
- Panics on executor threads print to libtest's inherited capture, so the suite stays silent.
- Red sites spot-checked: `fcce19b:keyboard_hook_callback.rs:219`, the writer-first seam at `7827ec8`, and the
  poll-then-wait structure at `a364b77`.

**R3, R5-R9**:
- R3: no new `Mutex` on hook state, and `with_engine` is never taken on the executor.
- R5: `FakeRunner` uses `with_*` builders.
- R6: the new `Counter` variants have `Display` arms. The new enums are never printed.
- R7: `update_bindings` still does one sorted store, then the cancel.
- R8, R9: untouched.
- No pressed key name or typed text reaches a log record: the new lines carry counts, a `WindowOp`, an `HRESULT` or a
  Win32 error only. The `Launch`/`OpenUrl` lines quote the config, as before.

## Positive
- Making the run list pure (`runs.rs`, with a fake clock) puts iteration, deadlines, epochs and interleaving under real
  tests, as ruling R1 asked.
- `plan()` separates the timer decision from Win32. The `code_ran` rule closes the "a modal loop ate my `WM_TIMER`"
  hole without trusting the timer.
- The death epilogue owns the queue, timer and runs outside the unwinding closure, so a loop panic is still visible
  and counted.
- `FastAction` makes running a slow action inline impossible to write by mistake. Every match is exhaustive, so a new
  action does not compile until its placement is decided.

## Open questions
- After a mid-session executor death (foreign `WM_QUIT`), `cancel_pending` still posts `WAKE` on every `swch off`/`on`
  and every reload to the dead executor's thread id (`mod.rs:240-243`). It is harmless while that id cannot be reused.
  In practice it cannot be reused while `Executor` holds the open `JoinHandle` (the session-long `AttachedExecutor`),
  but Microsoft documents a thread id only as valid "until the thread has been terminated". What would settle it: the
  rule that a thread id stays reserved while a handle to the thread is open. If that rule does not hold, this is a
  mid-session cousin of the accepted R-7.
