# Orchestrator rulings for lane p2-2.1-executor (2026-10-18)

Binding for every stage of this lane. Where a ruling differs from `cut.md`, the ruling wins. Rulings R1 and
R2 change the design, so the architect revises `cut.md` against this file (a "Revision after rulings"
section, with the touched decisions, commits, tests, acceptance mapping and manual checks rewritten in
place), and the architecture-critic checks the revision (`critique_v2.md`) before any code is written.

## Design rulings

- **R1 Macros keep running concurrently: a `Delay` never holds up another job (overrides D2 "Cost", D6, Q3).**
  The cut serialises macros: one blocking, pumped `Delay` stops every later `Launch`, `OpenUrl` and macro
  for its whole length, and it lists this as a Breaking change. Today every macro runs on its own thread, so
  a `Delay(5000)` macro does not delay an `OpenUrl` fired a moment later. The lane keeps that behaviour
  without a thread per run:
  - The executor keeps a list of active runs (a Sequence's steps, the next step index, its target window,
    its epoch, and a wake-up deadline). A new job starts at once and runs its steps until the first `Delay`
    or its end. A `Delay` stores a deadline (`Instant` + `checked_add`; `None` = until cancelled) and returns
    to the loop. When a deadline passes, its run continues from the next step.
  - The thread still waits in a `GetMessageW` loop. While a run is waiting, one thread timer
    (`SetTimer(None, 0, ms, None)` on the executor thread, re-armed for the nearest deadline and killed when
    no run waits) wakes it. The architect picks the exact mechanism and documents its granularity
    (about 15.6 ms, the same as the old `thread::sleep`). `thread::sleep` and a blocking pumped wait are not used.
  - An epoch bump drops every run and every queued job of the old epoch at the next wake. The bump posts
    `WAKE`, so no further step of a cancelled run starts once the executor wakes, and the test of the
    acceptance line "Delay(10000) stops within 100 ms" shows exactly that.
  - Steps that block in Win32 (`Launch`, `OpenUrl`, a `ShellExecuteW` dialog) still hold up the thread
    while they run. That is the only remaining serialisation; the cut and the CHANGELOG say so, and say
    "a running step finishes first" where they promise the 100 ms stop.
  - Runs advance in a fixed order (oldest first); two macros fired together interleave by step, as two
    threads did before. No Breaking entry for macro order.
  - The `Runner` keeps `action` and `step`; the executor owns iteration, `Delay`, deadlines and epochs, so
    all of them are real code under test with the fake runner. Tests cover:
    - a `Delay(5000)` macro followed by a fast job: the fast job runs within 100 ms;
    - two interleaved macros keep each one's step order;
    - a cancelled waiting run never resumes;
    - a `Delay(u64::MAX)` run waits until cancelled and arms no timer longer than `u32::MAX - 1` ms.
- **R2 `Window(...)` steps use `ShowWindowAsync` (critique I1).** A minimise, maximise or restore of a hung
  window must not block the executor. `ShowWindowAsync` is documented for exactly this case (posting the
  show state to the window's own thread). `Close` stays `PostMessageW(WM_CLOSE)`. The step's failure is
  logged on the executor thread. A manual check names the case (a `Window(Minimize)` macro against a
  Not Responding window, then an `OpenUrl`: the URL opens). If a window op exists that has no async form,
  the cut names it, and its cost is declared.
- **R3 Executor death is visible (critique M4, Q7).** `WM_QUIT` or `GetMessageW` returning `-1` still end the
  executor with one `[ERROR]` line, as D2 says. Every later submit that finds the executor gone while
  `stopping` is false is counted (a new counter, for example `executor_unavailable`, with its `Display`
  arm). The writer's periodic report then shows it. Closed during a requested stop stays silent and uncounted.

## Architect's questions

- **Q1** `BindAction` by value in `Job`: accepted (no allocation on the hook path; the roadmap's `Arc` is
  item 3.5's). Record it as a declared deviation.
- **Q2 / critique M3, book pages.** Pages whose config semantics change are done in this lane by the developer:
  - `README.md` (including `:415-418`);
  - `book/src/config/actions.md`, `config/README.md:55`, `config/example.md:189,:207,:237`,
    `config/kinds.md:77,:142`, `config/triggers-conditions.md:41`, `config/matching.md:150`;
  - `config/known-issues.md` row 36 (fixed) and row 26 (partly fixed).

  Non-config pages (`architecture.md`, `glossary.md`, `overview.md`, `build-and-run.md`, `gaps.md`,
  `roadmap.md`, `audit.md`) go to the Phase-2 book sync. `impl.md` lists each one with its file:line.
- **Q3** superseded by R1.
- **Q4** `SLOW_CALLBACK` 5 ms: accepted.
- **Q5** `QUEUE_CAPACITY` 32, `STOP_TIMEOUT` 2 s, `START_TIMEOUT` 5 s: accepted.
- **Q6** failed posts are counted without their error text: accepted.
- **Q7** see R3.

## Critique, Minor and open questions

- **M1** adopted. The no-log scan of X7 covers every function reachable from the hook path that this lane
  adds or changes (`ExecutorHandle::submit`, `TargetWindow::for_action`/`foreground`,
  `KeyboardHook::submit`/`cancel_actions`, `FastAction::run` and what it calls). Keep the list in one place
  in the test, with a comment that names why each one is on it.
- **M2** adopted. L7: land the seam with today's order first, then the red test (a runtime red by
  assertion), then the fix. X3: land the `.execute()` scan test alone, red by assertion on the then-current
  tree, before the commit that removes the calls.
- **M3** see Q2.
- **M5** adopted: a test drives the Hold path's split with a `FakeRunner` executor and an `F13` `Hold(1)`
  `OpenUrl` binding. The constraint is absolute: no hook is installed and no input is injected. If that
  turns out impossible, the developer stops and reports, rather than weakening the constraint.
- **Critic open question 1** (D8 detaches after 2 s, the brief says "joins"): accepted as a declared
  deviation. A bounded join, then detach with a `[WARN]`.
- **Critic open question 2** (Win+L through an injected L is unverified): do not publish it. Keep only the
  Ctrl+Esc example. Validate both RON snippets with `cfgcheck` (init + flush, `2>&1`).
- **Critic open question 3** (the 5 ms enqueue bound on CI): assert every one of the 20 enqueues under 5 ms,
  with one retry of the whole batch. No further loosening without a new ruling; a flake goes to the
  orchestrator.
- The residual hold-timer race at shutdown (R-7, a wake posted to a reused thread id) is accepted until 2.2.

# Round 2 (after the cut revision and critique_v2.md, 2026-10-18)

The developer implements the revised `cut.md` with these rulings applied, and records each one in the
developer addendum at the end of `cut.md`.

- **V2-I1 Drain the queue before every wait (critique_v2 I1).** "Drain before every `GetMessageW`" must hold
  literally. The last thing the loop does before `GetMessageW` is a `try_recv` that returned `Empty`, and
  no runner call (`action` or `step`, either of which can pump a modal loop) comes after it. Structure: repeat
  {drain the channel; poll the due runs} until one full pass ran no runner call, then arm the timer and wait.
  Red-first test: on a real executor thread, a fake step removes its own thread's posted `WAKE` messages
  (`PeekMessageW` with `PM_REMOVE` for the wake id, standing in for a modal loop) while another thread
  submits a job. The job must run with no further wake. It fails on the structure of the cut as written.
  No input is injected.
- **V2-M1** adopted: `book/src/config/actions.md:309` (focus after `Window(Minimize)`) is corrected in X4.
- **V2-M2** adopted: M11 also runs once on `master` as the control, and `impl.md` says so. If the control
  does not hang either (DWM's ghost window), M11 is recorded as "does not reproduce the hazard". R2 still stands,
  on Microsoft's documentation.
- **V2-M3 / N3, the timers.** Use `SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)` instead of
  `SetTimer`: same feature, same thread, and macro pacing must not depend on the default coalescing. After
  any pass that ran a runner call, the timer is re-armed (killed and recreated for the nearest deadline). It
  is kept only when nothing ran. Add the critic's manual timing line: `Sequence` of 20 x
  `[Key("F13"), Delay(50)]` finishes in about 1.0-1.4 s.
- **N1** accepted: active runs stay unbounded, and the queue bounds the backlog. Revisit with 3.5.
- **N2** accepted: runs interleave at `Delay` steps. A stretch of `Text`/`Key` steps stays whole.
- **N4** accepted: `Window(...)` no longer waits for the window. This goes under Changed, with the advice to put a `Delay`
  after `Window(Restore)`.
- **Critic open question 1, a loop panic.** The receiver and the delay timer are owned outside the closure that can unwind, so the
  death epilogue can drain and count the queued jobs and kill the timer. State the ownership in the addendum.
  A test of a loop panic is wanted only if a natural seam exists. Otherwise the ownership is a review item.
- **Critic open question 3** adopted as manual check M12: `[Window(Minimize), Delay(2000), Window(Restore)]`
  against the foreground window. Record whether the window comes back to the front. If it does not, the X4
  Changed entry says so.

# Round 3: owner priority on latency (2026-10-19)

The owner: binds must run with the lowest possible latency. The key-to-action path is the top
non-functional priority, recorded in CLAUDE.md "Owner decisions" on `master` (`62e36a6`). This lane's
branch was cut before that commit. For this lane:
- The fast actions stay inline on the hook thread, which is the lowest-latency path, and they must not move.
  The slow path may add only what the cut prescribes on the hook thread: `as_fast`, one atomic load, a
  `try_send` into a preallocated slot and one `PostThreadMessageW`.
- The longest-callback gauge costs two `Instant::now()` reads and one `fetch_max` per event. It is accepted,
  because it is the measurement this priority needs.
- The reviewer rates any other per-event work this lane adds to the hook path as at least Important: an
  allocation, a lock, a Win32 call, a log call or a syscall.
- A new job's first step must start without waiting for a timer. Only `Delay` waits.
- The results analyst states the lane's per-event cost on the hook path, before and after, in `results.md`.
- **R3-T Delay keeps millisecond precision (overrides R1's `SetTimer` and V2-M3's `SetCoalescableTimer`).**
  The old `thread::sleep` was accurate to about 1 ms, because std on Windows sleeps on a high-resolution
  waitable timer (`library/std/src/sys/thread/windows.rs:119`, `WaitableTimer::high_resolution`). A
  thread timer rounds every `Delay` to the system tick (about 15.6 ms, at least 10 ms), and commit
  `bb0a0aa` documents that as a change. Under the latency priority it is a regression and must not ship:
  - The executor owns one waitable timer created with `CreateWaitableTimerExW(.., CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS)`.
    If the flag is refused (before Windows 10 1803), it falls back to a normal waitable timer, as std does.
    The handle is closed by a `Drop` guard on every path.
  - The loop's single wait point is `MsgWaitForMultipleObjectsEx(1, &timer, INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`.
    - On the timer: poll the due runs.
    - On input: a `PeekMessageW(PM_REMOVE)` loop. `WAKE` drains the queue, `WM_QUIT` is the death of R3, and
      every other message is translated and dispatched, so the STA still pumps.
    - `SetWaitableTimer` gets a relative due time for the nearest deadline, and `CancelWaitableTimer` runs when no run waits.
    - `Instant` still decides. V2-I1 (the last act before the wait is a `try_recv` that returned `Empty`) is unchanged.
  - Put the timer wrapper where item 2.2 can reuse it for the gesture timers (roadmap 2.2 names `src/platform/timers.rs`).
  - A real-timer test shows a `Delay(50)` run resuming well under one system tick late. The developer sets
    the CI-safe bound with a written reason, for example the median of 10 runs under 5 ms late.
  - The CHANGELOG, README and `actions.md` lines of `bb0a0aa` return to "about 1 ms".
  - This is fixed in this lane, as a follow-up commit on the branch, while the lane is open. If the lane has already merged when this ruling is read, item 2.2 does it as its first commit group, before any gesture timer work.
