TRIAGE: 1 confirmed, 4 refuted, 0 deferred

# Triage r1: p2-2.1-executor @ a0a8b82

Inputs: `test_r1.md` (red items F1, F2 = M18/M21/M22/M26, and the M17 consequence of F1), `review_r1.md` (no
Critical or Important; Minor m1 and one open question). Checked against `feat/action-executor` HEAD `a0a8b82`
through `git show`, the spec and `rulings.md`. The working tree was not used as evidence.

## Gate (static half, on `git archive a0a8b82`, `D:/tmp/ks-orch/p2-2.1-executor/triage-r1/tree`, own target dir)
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0. The rerun after touching the three crate roots
  printed "Checking key-switch-rs v0.1.0 ... Finished".
- `cargo test` was not run (the tester owns it).
- The three tester commits (`c94acd5`, `2e51c2a`, `a0a8b82`) add lines only inside `mod tests` hunks
  (`mod.rs:1239+`, `runs.rs:392+`, `counters.rs:303+`). Each carries the Sonnet trailer.

## Verdicts

F1 CONFIRMED: `src/core/executor/mod.rs:1357` (`assert_eq!(kind, Some(APTTYPE_STA))`) and `:1372-1377` (`assert_eq!(inside, Some(APTTYPE_STA))`, `assert_ne!(after, Some(APTTYPE_STA))`) assume that the probed thread is a plain STA. A thread that enters an STA while no other STA thread is alive becomes the process's main STA, and `CoGetApartmentType` reports that as `APTTYPE_MAINSTA` (3). So both tests fail whenever no other test's executor thread holds an STA at that moment: always under `--test-threads=1` or `--exact`, and by scheduling chance with a few threads. CI runs `cargo test --locked` on `windows-latest` (`ci.yml:27,39`) with the default thread count, so the gate is order-dependent. The scratch probe confirms the mechanism (windows 0.62.2, the same `COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE`, `D:/tmp/ks-orch/p2-2.1-executor/triage-r1/probe`). Output, with APTTYPE STA=0 and MAINSTA=3:
  - The first STA alone: `(Some(3), None)`. Again after it exited: `(Some(3), None)`.
  - A second STA while a holder STA lives: `(Some(0), None)`.
  - After the holder exited: `(Some(3), None)`.

  This is a test defect, not a production one: `Apartment::enter_sta` (`:370-386`) and its `Drop` (`:388-396`) are correct, and MAINSTA is a single-threaded apartment. The failure is reached as follows:
  - Path: `cargo test --locked --lib core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment -- --exact`.
  - It starts one executor, `enter_sta` makes that thread the main STA, and the probe sends `Some(APTTYPE(3))`, so the assert fails.

  Consequence (the M17 note): a lone run of the guard test is red under M17 for the wrong reason, at the first assert. A full run catches M17 for the right reason only when another STA thread happens to be alive.

  What to do: in both tests, accept `APTTYPE_STA` or `APTTYPE_MAINSTA` for the inside reading, and in the guard test assert that `after` is neither. The tester's scratch fix does this (`is_sta` helper). With it, `core::executor -- --test-threads=1` was 33 passed and 0 failed, and M17 fails the guard test with "after the guard the thread has no apartment of its own".

F2/M18 REFUTED (closed at HEAD by `c94acd5`):
- `a_job_still_queued_when_the_executor_dies_is_counted` (`mod.rs:1240-1283`) queues a job by `try_send` with no wake, then posts `WM_QUIT` to its own executor thread.
- `run_loop` returns `End::Died` on `GetMessageW == 0` (`:663-668`), and the epilogue counts every job left in the queue as `ExecutorUnavailable` (`:565-571`). With M18 (no drain) the count is 0, so the `== 1` assert fails.
- The counters are per test (`leaked_counters`, `testing.rs:158`).
- The 200 ms `thread::sleep` is test code, outside the X6 scan. A slow machine can only make the test fail ("job ran"), never pass vacuously.
- Nothing of the user's is touched: the `WM_QUIT` goes to the test's own thread.

F2/M21 REFUTED (closed at HEAD by `2e51c2a`):
- `bump_if_err_ignores_an_ok_result` (`counters.rs`, the test module from `:303`) bumps on one `Ok` only and asserts a sum of 0.
- With `is_err` inverted to `is_ok`, the sum is 1, so the test fails. The older two-call test could not tell the two apart, as the tester said.

F2/M22 REFUTED (closed at HEAD by `2e51c2a`):
- `launch_and_open_url_carry_no_window_and_ask_for_none` (`mod.rs:1286-1305`) asserts that `for_action` (`:85-91`) returns `NONE` for a `Launch` and an `OpenUrl`.
- Under the mutation, a desktop with a foreground window gives a non-null target, so the test fails.
- NOTED: where `GetForegroundWindow` returns null (no interactive desktop) the test is vacuous; its own comment says so. It can never fail falsely, and it calls no Win32 function unless mutated.

F2/M26 REFUTED (closed at HEAD by `a0a8b82`):
- `runs::tests::the_nearest_deadline_of_several_waiting_runs_is_next` (`runs.rs:393-414`) is pure and runs on the fake clock.
- The earliest deadline (50 ms) belongs to the second run started, so the test tells `min` from `max` (900) and from "oldest run" (200). It also checks that the next deadline after the first fires is 200 ms.
- `Runs::next` uses `.min()` (`runs.rs:148-156`).

review m1 NOTED: `pub fn thread(&self) -> &Thread` (`mod.rs:317-319`) still has no `///` line at HEAD. It is a doc-only fix with no runtime failure.

review OQ (wakes posted to the id of an executor that died mid-session) NOTED:
- No reachable failure in this lane. `ExecutorHandle::wake` (`mod.rs:254-260`) ignores a failed post.
- The `Executor` keeps the thread's `JoinHandle` open for the session (`AttachedExecutor`). That keeps the thread object, and with it its id, alive.
- This is the mid-session form of the R-7 race that `rulings.md` accepts until roadmap 2.2.

tester note (process-wide `COUNTERS` in `the_keyboard_callback_records_how_long_it_ran`) NOTED:
- The test (`keyboard_hook_callback.rs:707`) cannot be flaky. `git grep COUNTERS` at HEAD shows that nothing in the suite takes or lowers the global gauge.
- Writers only `fetch_max` it. The only reader that resets it is the writer started by `logging::init` (`logging/mod.rs:112`), which no test calls. `a_second_init_starts_no_writer` uses closures and never reaches that writer.

## Summary
One defect survives: F1. Two COM tests assert `APTTYPE_STA`, but a lone STA thread reports `APTTYPE_MAINSTA`, so the
gate depends on test order. This is confirmed by a probe, and the fix is test-only. The F2 gaps are closed by the
tester's test-only commits: each new test is per-test or pure, safe, and fails under its mutation. The fmt and clippy
gate is green at `a0a8b82`.
