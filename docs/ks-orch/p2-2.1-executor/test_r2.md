# Tester r2: p2-2.1-executor / roadmap 2.1 (KS-08), delta since 6a1db71

HEAD `6c83406` (one tester `Tests:` commit on top of developer head `a875960`); tree clean (`git status --short` empty), `git worktree list` shows only the lane tree.

Verdict: GREEN. The one CONFIRMED item (F1) is red on `a0a8b82` and green on HEAD; seven mutations, each caught by a named test; one gap (drain counts only the first job) found and closed by a new test.

## Gate (HEAD)
- `cargo fmt --check`: OK (exit 0), also after the new test.
- `cargo clippy --all-targets --locked -- -D warnings`: OK (sources touched first; "Checking key-switch-rs ... Finished"); re-run after the new test: Finished, no warnings.
- `cargo test --locked --no-fail-fast` on HEAD `6c83406`:
  - lib: `running 592 tests` / `test result: ok. 587 passed; 0 failed; 5 ignored`
  - swch: `running 0 tests` (no tests in that binary)
  - `tests/ipc_roundtrip.rs`: `running 6 tests` / `6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests`
  - (At `a875960` before my commit: `running 591 tests` / `586 passed; 0 failed; 5 ignored`, same other targets.)
  - Ignored (not run, manual): `core::executor::tests::the_win32_runner_opens_a_url`, `data::condition::tests::foreground_context_names_the_focused_notepad`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`.
- `cargo core::executor -- --test-threads=1` on `a875960`: `running 34 tests` / `33 passed; 0 failed; 1 ignored` (the order-dependence of F1 is gone).
- Release build not re-run: no production line changed in round 2 (delta is test-only; `git diff --stat 6a1db71..HEAD` touches only `#[cfg(test)]` hunks) and the brief narrows the gate to clippy and tests.

## Red-first replay (F1, the only CONFIRMED item of triage_r1)
Parent `a0a8b82`, scratch worktree `D:/tmp/ks-orch/p2-2.1-executor/wt/parent2`, own `CARGO_TARGET_DIR`, each test `--exact`.

| Test | Parent `a0a8b82` | HEAD | Note |
|---|---|---|---|
| core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment | `running 1 test`; red (assert): `panicked at src\core\executor\mod.rs:1357:9`, `left: Some(APTTYPE(3)) right: Some(APTTYPE(0))`; `1 failed` | `running 1 test`; `1 passed; 0 failed` | F1 |
| core::executor::tests::the_apartment_guard_uninitialises_com_on_drop | `running 1 test`; red (assert): `panicked at mod.rs:1372:9`, same left/right; `1 failed` | `running 1 test`; `1 passed; 0 failed` | F1 |

Worktree removed, `git worktree prune`, target dir deleted; `git worktree list` shows only the lane tree.
F2 items (M18, M21, M22, M26) are REFUTED by triage (tests added in r1; the code is correct on each parent, so no red-first on a parent exists); they are exercised by mutations below.

## Mutations (scratch worktree `wt/mut2` at `a875960`, own target dir; each reverted, `git diff --exit-code` clean; worktree removed)
All run `--lib <full test path> -- --exact`, each `running 1 test`.

| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| MA | mod.rs `enter_sta` (:381) | `COINIT_APARTMENTTHREADED` -> `COINIT_MULTITHREADED` | the_executor_runs_jobs_in_a_single_threaded_apartment and the_apartment_guard_uninitialises_com_on_drop | caught: panics at `mod.rs:1367:9` and `:1382:9` (the new `is_sta` asserts) |
| MB | mod.rs `enter_sta` (:384) | `initialized: hr.is_ok()` -> `false` (Drop never uninitialises) | the_apartment_guard_uninitialises_com_on_drop | caught: panic at `mod.rs:1383:9` (`!is_sta(after)`), alone with `--exact`: the F1 fix now catches this for the right reason |
| ME | mod.rs run_executor epilogue (:567) | `while queue.try_recv().is_ok()` -> `if` (counts only the first job) | a_job_still_queued_when_the_executor_dies_is_counted | NOT CAUGHT (old test queues one job: `1 passed`) -> gap, closed by new test below, which then fails (`left: 1 right: 3`) |
| M18 | mod.rs run_executor (:565) | `if end == End::Died` -> `&& false` (no drain) | a_job_still_queued_when_the_executor_dies_is_counted | caught: `left: 0 right: 1` at `mod.rs:1275:9`; also caught by the new test (`left: 0 right: 3`) |
| M21 | counters.rs `bump_if_err` (:123) | `is_err()` -> `is_ok()` | logging::counters::tests::bump_if_err_ignores_an_ok_result | caught: `counters.rs:309:9`, `an Ok result counts nothing: [0,0,0,0,0,0,1]` |
| M22 | mod.rs `for_action` (:90) | `Launch` also reads the foreground | core::executor::tests::launch_and_open_url_carry_no_window_and_ask_for_none | caught: `a Launch never reads its window`, `left: TargetWindow(196676) right: TargetWindow(0)` (this desktop has a foreground window; vacuous on a desktop without one, as the test's comment states) |
| M26 | runs.rs `Runs::next` (:155) | `.min()` -> `.max()` | core::executor::runs::tests::the_nearest_deadline_of_several_waiting_runs_is_next | caught: `runs.rs:400:9`, `the timer is armed for the earliest deadline` |

## Acceptance (delta scope)
| Criterion | Evidence | Status |
|---|---|---|
| F1: COM tests accept the process's main STA, order-independent | both tests green alone (`--exact`) and `--test-threads=1`; red on `a0a8b82`; MA, MB caught | verified |
| F1: guard test catches a missing CoUninitialize for the right reason | MB: fails at `mod.rs:1383:9` alone | verified |
| F2: M18, M21, M22, M26 closed by r1 tests | mutations above | verified |
| 2.1 acceptance items outside this delta | unchanged since r1 (`test_r1.md`); no production change | not re-run |

## Failures
None on HEAD. One test gap found (ME above): the epilogue test passed when only the first of several queued jobs is counted. Closed by the commit below.

## Tests added by the tester
- `6c83406` `Tests: every job still queued when the executor dies is counted once`: `src/core/executor/mod.rs` (test code only, inside `mod tests`): `core::executor::tests::every_job_still_queued_when_the_executor_dies_is_counted`. Green on HEAD, 5 reruns of the filter `1 passed` each (0.20 s); red under ME (`left: 1 right: 3`) and M18 (`left: 0 right: 3`). It posts `WM_QUIT` only to the test's own executor thread (SAFETY comment present), uses per-test counters, calls no daemon, hook, or input API. Trailer: `Co-Authored-By: Claude Sonnet 5.5`.

## Notes
- `--exact` needs the full module path: a bare test name printed `running 0 tests`; every quoted run above uses the full path and shows `running 1 test`.
- The new and the old M18 tests contain a 200 ms `thread::sleep` in test code (outside the X6 scan; a slow machine can only make them fail, never pass vacuously).
- review m1 (`pub fn thread` lacks `///`) and the OQ (wake to a dead executor id) remain NOTED, unchanged.
- Scratch worktrees and target dirs for this round are removed; nothing is uncommitted in the lane tree.
