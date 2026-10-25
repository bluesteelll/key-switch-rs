# Code review: p2-2.1-executor @ 6c83406 (round 2)
APPROVED (Critical 0, Important 0, Minor 2)

Scope: `git diff 6a1db71..feat/action-executor`. The branch is at `6c83406`. That is one commit past the `a875960` that
`pack_r2.md` describes: the tester's `6c83406` ("Tests: every job still queued when the executor dies is counted once")
landed after the pack was written, so this review covers it too. Code was read through `git show` and `git diff` only.

- Commits: `c94acd5`, `2e51c2a`, `a0a8b82`, `6c83406` (tester, Sonnet trailer) and `a875960` (developer, Opus trailer).
  Every subject starts with `Tests:`.
- Every hunk lies inside a `mod tests` block:
  - `mod.rs` hunks at `@@ -694`, `-1238`, `-1266`, `-1289`, `-1304`; the tests marker is at `:688`.
  - `runs.rs` hunk at `@@ -391`; the marker is at `:216`.
  - `counters.rs` hunk at `@@ -302`; the marker is at `:202`.
- No production line changed in round 2. The round-1 production review at `6a1db71` therefore stands as written.

## Gate: fmt ✅ · clippy ✅
Run on `git archive` extracts, not the shared work tree:
- Extracts: `D:/tmp/ks-orch/p2-2.1-executor/review-r2/tree` (`a875960`) and `.../review-r2/tree-6c83406` (`6c83406`).
- Own target dir: `.../review-r2/target`.
- `cargo fmt --check`: exit 0 on both.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 on both. The rerun after touching `src/lib.rs`,
  `src/main.rs` and `src/bin/swch.rs` printed "Checking key-switch-rs v0.1.0 ... Finished".
- `cargo test` was not run (the tester's gate).

## Previous items
- **triage_r1 F1 (CONFIRMED): ✅ closed by `a875960`.**
  - Change: `is_sta` (`mod.rs:1386` at `6c83406`) accepts `APTTYPE_STA` or `APTTYPE_MAINSTA`. The executor test asserts
    `is_sta(kind)` (`:1412`). The guard test asserts `is_sta(inside)` (`:1427`) and `!is_sta(after)` (`:1429`).
  - Production `Apartment::enter_sta`/`Drop` (`:370-397`) is unchanged.
  - Both tests can still fail:
    - with `COINIT_MULTITHREADED`, the probe reads `Some(APTTYPE_MTA)`;
    - without `CoInitializeEx`, it reads `None`, or the implicit MTA;
    - under M17 (no `CoUninitialize`), `after` reads STA or MAINSTA, and the test now fails at the right assertion.
  - Every message prints the observed `Option<APTTYPE>`.
  - Accepting MAINSTA weakens nothing, because the main STA is a single-threaded apartment.
- **triage_r1 F2 M18 / M21 / M22 / M26 (REFUTED, closed by the tester's tests): re-read, each fails under its mutation.**
  - M18 (no drain): `a_job_still_queued_when_the_executor_dies_is_counted` (`:1241-1285`) counts 0 instead of 1.
  - M18 variant (`if` instead of `while`): the new `6c83406` test `every_job_still_queued_when_the_executor_dies_is_counted`
    (`:1287-1330`) closes it. The one-job test passes under that variant; this one counts 1 instead of 3.
  - M21: `bump_if_err_ignores_an_ok_result` (`counters.rs:303-314`) sums 1 instead of 0.
  - M22: `launch_and_open_url_carry_no_window_and_ask_for_none` (`mod.rs:1332-1352`) gets a non-`NONE` target where a
    foreground window exists. Where none exists, it is vacuous, as its comment says.
  - M26: `runs.rs:392-414` puts the earliest deadline (50) on the middle run, so `.max()` (900) and "oldest run" (200)
    both fail.
- **review_r1 m1: ❌ still open (Minor).** `pub fn thread(&self) -> &Thread` (`mod.rs:317-319`) still has no `///` line.
  This is a doc-only gap; see m1 below.
- **review_r1 open question (a wake posted to a dead executor's thread id): ✅ settled.**
  - Microsoft documents that the thread identifier "is valid from the time the thread is created until all handles to
    the thread are closed and the thread object is freed; at this point, the identifier may be reused"
    (`PROCESS_INFORMATION`).
  - `Executor` holds the `JoinHandle` for the session (`AttachedExecutor`, `app.rs:29-63`).
  - `stop_thread` posts its wake (`cancel_pending`, `mod.rs:338`) before `join.join()` closes the handle (`:343`).
  - So no wake from the executor's own paths can reach a reused id. Wakes from threads that loaded the handle before
    the detach are the R-7 window, which `rulings.md` accepts until roadmap 2.2.

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. `Executor::thread` is public API without a doc comment (carried over from round 1, section 7)
**Where**: `src/core/executor/mod.rs:317-319` @ `6c83406`  **Confidence**: CONFIRMED (read)
**Problem**: This is the only public item of the new module with no `///` line.
**Failure**: Nothing fails at run time. `cargo doc` shows an undocumented method on a public type.
**What to do**: Add one line, for example "The executor thread, so that tests can check where a job ran." This is
optional and does not block.

#### m2. The two executor-death tests depend on a fixed 200 ms margin (tests only)
**Where**: `src/core/executor/mod.rs:1254` and `:1299` @ `6c83406` (`thread::sleep(Duration::from_millis(200))`)
**Confidence**: PLAUSIBLE (path traced; not reproduced)
**Problem**: Each test queues its "lost" jobs directly on `handle.jobs` with no wake and then posts `WM_QUIT`. The test
only means something if the executor is already parked in `GetMessageW` (`mod.rs:653`) by then. The only thing that
ensures this is the 200 ms sleep after the first record arrives.
**Failure**: Suppose the executor thread is not scheduled for 200 ms or more somewhere between two points:
`FakeRunner::call`'s `records.send` (`testing.rs:123`) and the loop's last empty `try_recv` before `GetMessageW`
(`mod.rs:612-624`). Then the pass that is still running drains and runs the "lost" jobs. The count comes out 0, or
fewer than 3, and the test fails even though the code is correct. This is a false red only, never a false green.
The probability is low: a running thread has to stall for 200 ms, against the suite's own 2 s bound for "at once"
(`WAIT`, `testing.rs:26`). The scenario has two copies, so the exposure is twice that of one test.
**What to do**: This is optional. The test can tell when the race happened: a record of a "lost" job means the pass
was still going. In that case it can retry once, like the whole-batch retry of the 5 ms enqueue test. In practice the
three-job test also covers what the one-job test checks, so keeping only the three-job test halves the exposure.

## Checklist walked (items looked at, with no reachable failure)
- **Hot path (R1, section 5)**: `git diff 6a1db71..6c83406 -- src/hook` is empty. The callback, `dispatch`, `submit`
  and `for_action` are unchanged since the round-1 review.
- **No key name or typed text in logs**: the new tests add no `log::` call. The only record they cause is the existing
  `[ERROR] ... received WM_QUIT` line (`mod.rs:664-666`), which carries no key or text. In tests that line goes nowhere:
  no writer runs, and `logging::testing::capture` keeps only its own thread's records.
- **Threads, loops, timers, COM, shutdown (R10, section 4)**: `WM_QUIT` is posted only to the test's own executor
  thread (`:1270`, `:1315`), while that thread is alive and its queue exists, which the `ready` handshake guarantees.
  The guard test's `CoInitializeEx` runs on a scratch thread it spawns itself. No hook is installed and nothing is
  injected.
- **Cross-thread ordering (R4)**: the tests load `epoch` with `Acquire`, which pairs with `cancel_pending`'s `AcqRel`.
  - `take_counts` follows `is_finished()`, which in std is a `Relaxed` `strong_count` load (`alloc/src/sync.rs:2027`,
    local toolchain 1.98.1). So in principle no happens-before orders the epilogue's bumps before the test's `swap`.
  - On x86-64 (CI `windows-latest` and the owner's machine), every RMW is a locked instruction, so the `swap` sees the
    bumps. This is not a defect on the target.
  - Taking the counts after `stop()`, which joins the thread, would make the ordering formal. The round-1 test
    `an_executor_ended_by_wm_quit_counts_every_later_submit` uses the same pattern.
- **Suppression invariant, other kinds and triggers (section 6)**: no production change.
- **Sentinel (R2)**: no `SendInput` site was added. The new tests reach only `FakeRunner` and the pure `Recorder`.
- **`// SAFETY:` (section 2)**: the new `unsafe` blocks at `:1270` and `:1315` carry concrete comments (`:1267-1269`,
  `:1312-1314`). `a875960` adds no `unsafe`.
- **Atomics**: no `Relaxed`, `SeqCst` or `static mut` was added.
- **Tests can fail and are safe**:
  - Each new test fails under its named mutation (see above).
  - None installs a hook, injects input, runs a real action, or calls `logging::init` or `attach_file`.
  - The test sleeps sit below the tests marker, so the X6 scan (`:1439-1470`) does not see them.
- **Rulings**: R3 (an executor death is visible and counted) is now pinned at the epilogue drain as well. V2-I1, R1, R2
  and Q1-Q7 are unaffected.
- **Docs and CHANGELOG**: the change touches tests only and alters no behaviour a user can see. No entry is due.
- **R3, R5-R9**: untouched in round 2. The round-1 findings stand.

## Positive
- `a875960` fixes the cause, not the symptom. Each message names the apartment it observed, and the change makes the
  M17 mutation fail at the right assertion when the test runs alone.
- The M26 test spreads its deadlines so that `min`, `max` and "oldest run" each give a different answer.
- `6c83406` closes a weaker form of M18 (counting only the first queued job) that the one-job test could not tell apart
  from the correct code.

## Open questions
None blocking.

Sources: [PROCESS_INFORMATION (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-process_information),
[The Old New Thing, 2014-08-22](https://devblogs.microsoft.com/oldnewthing/20140822-00/?p=173).
