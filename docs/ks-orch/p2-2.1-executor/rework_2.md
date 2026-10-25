# Implementation: p2-2.1-executor rework 2 (results_v2 P1, ruling R3-T)

Branch `feat/action-executor`, HEAD `9690172`, tree clean. Base of this rework: `bb0a0aa` (rework 1). Seven commits:
four from the run that a session restart cut off (`a183b36`, `8ba1a0b`, `63f0caf`, `4cfd789`), checked against R3-T
here, and three new ones (`f2c9236`, `7d9a25e`, `9690172`). Artifacts: `D:/tmp/ks-orch/p2-2.1-executor/rework2/`.

## Items
- **R3-T / P1 option A, red test**: `a183b36`, `src/core/executor/mod.rs:894-939`.
  - Test: `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline`. A real executor thread with
    `FakeRunner` runs `[Key(F13), Delay(d)] x n` for d = 5, 20 and 50 (n = 20, 20, 10). No gap may be shorter than d,
    and the median lateness of each must stay under 5 ms. The reason for the bound is written in the test: 5 ms is a
    third of a tick, the measured lateness is well under 1 ms, and the median ignores the few gaps that a loaded
    machine delays.
  - Red, re-run here on a `git archive` export of `8ba1a0b` (the old `GetMessageW` loop):
    "Delay(5) ended 10.5099ms late in the median; sorted: [1.3199ms, 8.0914ms, ... 14.9345ms]"
    (`rework2/red-8ba1a0b.txt:62`).
  - Green at HEAD (`rework2/green-executor.txt:45`). Five more runs at HEAD were green.
- **R3-T, the one wait, red scan**: `8ba1a0b`, `mod.rs:1471-1523`.
  - Test: `macros_never_sleep_or_block_in_a_wait`. It now bans `WaitForSingleObject` and `GetMessageW` in
    `executor/mod.rs`, `runs.rs` and `sequence.rs`, and expects exactly one `MsgWaitForMultipleObjectsEx(` in
    `mod.rs`.
  - Red on the same export: "sleeps or blocking waits: src/core/executor/mod.rs:11 ... :588 ... :653 `let got = unsafe
    { GetMessageW(..) }` ... :677" (`red-8ba1a0b.txt:51-56`).
  - Green at HEAD.
- **R3-T, the timer wrapper**: `63f0caf`. New `src/platform/timers.rs:1-212`, `src/platform/mod.rs`,
  `src/lib.rs:12`.
  - `WaitableTimer::new` asks for `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` and makes a plain timer where Windows
    refuses the flag (`:41-59`).
  - `set` uses a relative due time (`due_time`, `:111-114`: negative, rounded up to 100 ns, at most
    `LONGEST` = 0x7FFF_FFFF ms). `cancel` is at `:84-88`, and `Drop` closes the handle (`:91-97`).
  - Five unit tests, green from their first run because this is new code:
    `due_time_is_relative_rounds_up_to_100_ns_and_stops_at_longest`, `a_timer_signals_once_after_its_due_time`,
    `setting_a_timer_again_replaces_its_due_time`, `a_cancelled_timer_does_not_signal`,
    `a_timer_whose_flags_windows_refuses_is_a_plain_one`.
- **R3-T, the executor waits on it**: `4cfd789`, `mod.rs`.
  - `DelayTimer` and `plan()` at `:404-500`.
  - `run_executor` at `:512-569`: the timer is made at `:520`, outside the `catch_unwind` closure, and dropped in the
    epilogue at `:556`.
  - `run_loop` at `:586-673`: the one wait is at `:648`, and `WAIT_FAILED` is the death of R3.
  - `pump` at `:677-701`: `WAKE` goes back to the drain, `WM_QUIT` is the death of R3, and every other message is
    translated and dispatched.
  - This commit turned both red tests above green.
- **R3-T, the docs return to about 1 ms**: `f2c9236`. Detail under Docs.
- **results_v2 hand-off, review m1**: `7d9a25e`, `mod.rs:322`. `Executor::thread` now has a doc line. No test
  (documentation only).
- **results_v2 hand-off, review m2**: `9690172`, `mod.rs:1297-1368`.
  - The two executor-death tests now share `die_with_lost_jobs`, which returns `None` if a lost job ran. Only a
    pass that is still running after the first job does that. `counts_left_by_lost_jobs` retries once in that
    case. Both tests keep their names and assertions.
  - Mutation, on an export of HEAD: I removed the epilogue's drain-and-count loop (`while queue.try_recv() ...
    bump(ExecutorUnavailable)`). Both tests are red by assertion: "the job left in the queue is counted when the
    executor dies: [... (ExecutorUnavailable, 0) ...]" and "each of the three jobs left in the queue is counted
    once: [... (ExecutorUnavailable, 0) ...]" (`rework2/mut-m2.txt:14`, `:22`). The retry does not weaken the
    tests.

## Remarks
- **results_v2 P1** (the macro `Delay` was about 10 ms late): **RESOLVED** by option A, as ruling R3-T
  (rulings.md:135-153) prescribes.
  - Measured with a scratch probe that drives the real `Executor` with a runner that only takes timestamps
    (no input, no hook): `rework2/probe-head.txt`, 2 runs x 40 gaps per row.
    - Executor: on average 0.21-0.37 ms late, at most 1.30 ms.
    - `thread::sleep`: on average 0.17-0.49 ms late, at most 1.51 ms.
    - No gap ended early.
    - 100 x `Delay(5)` takes 0.522-0.524 s (it was 1.56 s at `bb0a0aa`). 20 x `Delay(50)` takes 1.004-1.006 s.
  - Pinned by `a_delay_ends_within_a_few_ms_of_its_deadline`.
- **results_v2 re-check 1** (a ruling amends R1 and V2-M3, and the cut records it): **RESOLVED**.
  - R3-T is the ruling. The developer addendum of `cut.md` now has a "Round-3 ruling R3-T" section, with a row per
    R3-T line, the deviations and M13.
  - Effect on the hook path: none. `git diff bb0a0aa HEAD -- src/hook src/engine` is empty, so the hook-path cost
    in results_v2 stands.
- **results_v2 re-check 2** (a red-first real-thread test for d = 5 and 20, never early, within a fixed bound):
  **RESOLVED**, see Items. d = 50 is added for R3-T's "Delay(50)". The bound is the median under 5 ms, as R3-T's
  example suggests.
- **results_v2 re-check 3**: **RESOLVED**.
  - `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake`,
    `an_executor_ended_by_wm_quit_counts_every_later_submit` and the two death tests are green
    (`green-executor.txt:18, :35, :39, :40`).
  - Nothing new on the hook path.
  - Every new `unsafe` has a concrete SAFETY comment (listed below).
- **results_v2 re-check 4** (CHANGELOG.md:371-380, actions.md:318, README.md:537; M13 tightened): **RESOLVED**
  in `f2c9236`. M13 now reads: 20 x `[Key("F13"), Delay(50)]` finishes in about 1.00-1.05 s. It is recorded in
  the cut addendum.
- **results_v2 re-check 5** (gate green, at least 598 tests): fmt, check and clippy pass. 603 tests are listed
  (`cargo test --locked -- --list`), 5 of them ignored, all `manual:` and unchanged. The full run is the tester's.
- **results_v2 "the `code_ran` distrust can be simplified"**: **RESOLVED** in `4cfd789`.
  `plan(armed, next, now)` (`mod.rs:442`) keeps a timer that is set for the nearest deadline. Pinned by
  `the_delay_timer_is_set_again_only_for_another_deadline`. Declared as deviation 1.
- **review m1**: **RESOLVED** at `mod.rs:322`.
- **review m2**: **RESOLVED** at `mod.rs:1297-1368`. Pinned by the mutation above.

## Deviations from the cut
1. V2-M3's rule "re-arm after every pass that ran code" is gone, together with the thread timer.
   - A kernel timer's signal is not a message, so no modal loop on the executor thread can take it. Only the loop
     waits on the handle.
   - `the_delay_timer_is_kept_only_while_nothing_ran` became `the_delay_timer_is_set_again_only_for_another_deadline`.
   - `timer_ms` and `timer_ms_rounds_up_and_clamps` gave way to `platform::timers::due_time` and its test.
   - `LONGEST` (0x7FFF_FFFF ms, the old `USER_TIMER_MAXIMUM`) keeps R1's "no timer longer than u32::MAX - 1 ms"
     (`a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms`).
2. R3-T does not say what happens when no timer can be created at all, in either form.
   - What the code does: `Executor::start` fails, so the daemon exits before it installs any hook, through the
     existing fatal path: `[FATAL] could not start the action executor: could not create the Delay timer: ...`.
   - Alternative considered: run without a timer and stop every `Delay` that has a deadline, as a failed
     `SetWaitableTimer` still does per arm (`Arming::Failed`, `mod.rs:634`). Rejected: only resource exhaustion
     makes the creation fail, and a daemon that silently drops every macro `Delay` from the start is worse.
3. Review m1 and m2 were done in this round. results_v2's hand-off lists them as "optional, while fresh".

## Unsafe / atomics added
- `src/platform/timers.rs:76` `SetWaitableTimer`. SAFETY: "`self.handle` is the open timer handle this value owns;
  `due` is a local that outlives the call, which only reads it; no completion routine is passed, so nothing of ours
  is called later."
- `timers.rs:85` `CancelWaitableTimer`. SAFETY: "`self.handle` is the open timer handle this value owns; the call
  takes it by value and reads no memory of ours."
- `timers.rs:93` `CloseHandle` in `Drop`. SAFETY: "`self.handle` is open and owned by this value alone, which closes
  it once, here, and never uses it again."
- `timers.rs:101` `CreateWaitableTimerExW`. SAFETY: "no security attributes and a null name, so the call reads no
  memory of ours; the handle it returns goes straight into the WaitableTimer that closes it."
- `timers.rs:127` (test) `WaitForSingleObject`. SAFETY: "the handle is open as long as `timer`, which outlives the
  call; the call takes it by value and reads no memory of ours."
- `src/core/executor/mod.rs:644` `MsgWaitForMultipleObjectsEx`. SAFETY: "the slice holds the Delay timer's handle,
  which `timer` keeps open across the call, and lives until the call returns; the call only reads it."
- `mod.rs:678` `PeekMessageW(PM_REMOVE)`. SAFETY: "`msg` is the loop's local and outlives every call, which only
  writes it and removes the message from this thread's own queue."
- `mod.rs:693` `TranslateMessage` and `DispatchMessageW`: the existing block, with its comment updated to "filled
  by PeekMessageW above".
- `mod.rs:1328` (test) `PostThreadMessageW(WM_QUIT)`: moved from the two death tests into the helper; the SAFETY
  text is unchanged.
- No atomic was added or changed. `HANDLE` wraps a raw pointer and has no `Send` impl in windows 0.62.2
  (`Win32/Foundation/mod.rs:5291`), so `WaitableTimer` is `!Send`, as its doc says.

## Gates
- `cargo fmt --check`, `cargo check --all-targets --locked` and `cargo clippy --all-targets --locked -- -D warnings`:
  pass at HEAD.
- Tests run:
  - `core::executor` and `platform::timers`: 39 passed, 1 ignored (`rework2/green-executor.txt`).
  - `hook::`, `logging::` and `core::app`: 94 passed, 2 ignored (`rework2/green-hook-logging.txt`). This covers the
    hook tests on a real executor and the no-log scan.
  - The Delay timing test, the timer tests and the death tests: 5 more runs, all green.
  - The full suite is left to the tester.
- `mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/rework2/book`: OK. The only warning is the known
  mdbook-mermaid version notice. The anchors that the edited lines use (`roadmap.html#21-action-executor`,
  `actions.html#s14-3`, `#s13-1`) exist.
- Manual ignores added: none. Manual check M13 is tightened, as above.

## Docs
- `CHANGELOG.md`:
  - The `### Changed` bullet "A macro `Delay` is coarser" from `bb0a0aa` is removed.
  - The KS-08 `### Fixed` entry now says that a `Delay` waits on a high-resolution timer of the executor, which
    ends it within about 1 ms of its time and never before, as the `thread::sleep` it replaces did.
- `README.md`:
  - The `Delay` row (`:539`) says "within about 1 ms of its time and never before".
  - The `executor/` notes (`:152-153`) mention the high-resolution timer and pumping messages, not a
    `GetMessageW` loop.
  - The layout tree (`:39`, `:67-68`) says "message loop" and lists `platform/timers.rs`.
- `book/src/config/actions.md:318` (§14.3): the wake comes within about 1 ms (about 0.3 ms on average) and never
  early, 100 x `Delay(5)` takes about 0.5 s, and Windows 10 before 1803 falls back to the system tick.
- `book/src/config/errors.md`:
  - `:195` (`[FATAL] could not start the action executor`) also names the timer-creation cause.
  - `:198` names the failing wait, `MsgWaitForMultipleObjectsEx`.
  - `:201` names `SetWaitableTimer`.
- The "a saved `config.ron` that loads" wording of `bb0a0aa` is kept, because it is correct.
- `default_config.ron:151` makes no claim about precision and is unchanged.

## Commits
- `a183b36` Tests: a macro Delay ends within a few ms of its deadline
- `8ba1a0b` Tests: the executor's one wait is for its messages and its timer at once
- `63f0caf` Platform: a high-resolution waitable timer for threads that wait on deadlines
- `4cfd789` Core: a macro Delay wakes on a high-resolution waitable timer
- `f2c9236` Docs: a macro Delay ends within about 1 ms again, on the executor's high-resolution timer
- `7d9a25e` Core: Executor::thread says what it is for
- `9690172` Tests: the executor-death tests retry once when a pass still going ran a lost job

Every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Open points / pre-existing warnings
- For the orchestrator:
  - `rulings.md:22` still says the thread timer has "the same" granularity "as the old `thread::sleep`" (results_v2
    hand-off). R3-T overrides it, but the line itself is uncorrected.
  - `cut.md` D6 (`:118-128`) keeps the round-2 text. The addendum records that R3-T replaces it.
- The CLAUDE.md layout should list `src/platform/` (`timers.rs`). That file belongs to the orchestrator.
- For the Phase-2 book sync: `architecture.md` (the executor's wait and timer), and roadmap 2.2 "Files", which can
  now reuse `src/platform/timers.rs`.
- No pre-existing warnings in the gates.
