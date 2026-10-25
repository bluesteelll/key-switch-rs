# Results analysis v3: p2-2.1-executor / roadmap 2.1 Action executor (KS-08), re-check after rework 2

Branch `feat/action-executor` @ `9690172` (39 commits; merge-base `7d757a1`; `master` = `62e36a6`). The tree is clean:
`git status --porcelain` is empty and the branch is checked out.
Inputs: results_v2.md (my previous verdict), rework_2.md, rulings.md including **R3-T** (rulings.md:135-153), the cut
addendum "Round-3 ruling R3-T" (cut.md:640-660), and the 7 rework commits `a183b36..9690172`. I read the rework diff
in full: `src/platform/timers.rs` (all 212 lines), `src/core/executor/mod.rs` (the start path :275-325, the timer and
the loop :405-701, and the changed tests), `src/lib.rs`, `src/platform/mod.rs` and the four doc files.

## VERDICT: ACCEPT

**Summary**: Rework 2 resolves results_v2 P1 by option A, as ruling R3-T prescribes. A macro `Delay` now wakes on a
high-resolution waitable timer through one `MsgWaitForMultipleObjectsEx` wait. My own probe of the real executor
measures it 0.27-0.46 ms late on average and never more than 1 ms late (it was about 10 ms on average at `bb0a0aa`).
100 x `Delay(5)` takes 0.525 s, against 1.56 s before. The red-first tests are red for the right reason on the parent
and green at HEAD. Three of my four new mutations of the fix were killed. The gate is green four times on a clean export,
the hook path is byte-identical to the version I measured in v2, and the merge has no conflict. What remains is Minor,
or work handed to the orchestrator and the Phase-2 book sync.

## Quality gate (own run, clean export of `9690172`)
Export: `git archive 9690172` into `D:/tmp/ks-orch/p2-2.1-executor/analyst/v3/src-export`, with a new
`CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.1-executor/analyst/v3/target` that started empty, so every dependency was
compiled again. Output: `analyst/v3/gate.txt`.

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0), on the fresh target dir: the crate was compiled and linted ("Compiling key-switch-rs v0.1.0 (…\v3\src-export)", gate.txt:13) |
| `cargo test --locked` | lib 592 passed / 5 ignored; `ipc_roundtrip` 6 passed; swch and doc-tests 0. **598 passed, 0 failed, 5 ignored.** Log: `D:/tmp/ks-orch/p2-2.1-executor/analyst-test-v3.txt`. The suite prints nothing beyond libtest. Three more full runs (`analyst/v3/suite-run-{2,3,4}.txt`) are all green, timing tests included |
| test count vs master | 603 listed (`analyst/v3/branch_tests.txt`), against 598 at v2 and 543 on master. All 543 master tests are present (`comm -23` is empty). Removed since v2: `timer_ms_rounds_up_and_clamps` and `the_delay_timer_is_kept_only_while_nothing_ran`. Both were added by this lane for the thread timer, and they are replaced by `due_time_is_relative_rounds_up_to_100_ns_and_stops_at_longest` and `the_delay_timer_is_set_again_only_for_another_deadline`. Added: `a_delay_ends_within_a_few_ms_of_its_deadline` and the five `platform::timers` tests |
| `cargo test --locked -- --list --ignored` | 5, all `manual:` and unchanged since v1 (`the_win32_runner_opens_a_url`, `foreground_context_names_the_focused_notepad`, `hold_the_daemon_pipe_name_for_a_manual_start`, `the_menu_mask_key_is_inserted_into_the_input_stream`, `live_registry_with_the_language_hotkey_not_assigned_has_no_combination`) |
| `cargo doc --no-deps` | the same 4 warnings as master, none new |
| `mdbook build … -d analyst/v3/book` | OK (only the known mdbook-mermaid version warning). The anchors the edited lines use exist: `actions.html#s13-1`, `#s14-3` and `roadmap.html#21-action-executor` |
| trailers | 39 of 39 commits end with `Co-Authored-By: Claude (Opus|Sonnet) 5.5 <noreply@anthropic.com>`. The 7 rework commits are Opus. `bb0a0aa` is still an ancestor, so nothing was amended |
| merge | `git merge-tree --write-tree master HEAD` exits 0. Master's only new commit, `62e36a6`, touches only `CLAUDE.md` and `roadmap.md`, and the lane touches neither |
| Cargo.toml / Cargo.lock / build.rs | untouched. Every new API comes from `windows` features the crate already enables |
| hygiene of the delta | 0 non-ASCII lines added under `src`; no TODO, FIXME, print macro, `SeqCst` or `Relaxed`; `keyboard_hook_callback.rs` names no `log::` |

## Re-check of results_v2 P1 (option A)
| Re-check condition | Status | Evidence |
|---|---|---|
| 1. A ruling that amends R1 and V2-M3, recorded in the cut, which states the effect on the path | MET | R3-T (rulings.md:135-153) overrides R1's `SetTimer` and V2-M3's `SetCoalescableTimer`, and it corrects the false premise of rulings.md:22 in its own text. The cut addendum (cut.md:640-660) has one row per R3-T line, the deviations and M13. Hook-path effect: none (`git diff bb0a0aa HEAD -- src/hook src/engine` is empty; `submit`, `wake` and `TargetWindow` in mod.rs are outside every changed hunk). Note: no architecture-critic pass was run on R3-T. The ruling fixes the design down to the API calls and the workflow provides no third critique, so I checked the design against the section-4 invariants myself (below) |
| 2. A red-first real-thread test: never early, within a bound the cut fixes, red on the old loop and green after | MET | `a_delay_ends_within_a_few_ms_of_its_deadline` (mod.rs:921) runs d = 5, 20 and 50. Every gap must be at least d, and the median lateness must be under 5 ms; the reason for the bound is written at :922-929. **My replay** on a `git archive` of `8ba1a0b` (the old `GetMessageW` loop and thread timer, both red tests present): "Delay(5) ended 10.7228ms late in the median" (`analyst/v3/red-8ba1a0b.txt:63`). Green at HEAD in all 4 of my runs. "Never early" also holds by construction: a deadline is the `Instant` taken after the previous key plus d, and `Instant` alone decides when a run resumes |
| 3. The V2-I1 and R3 death tests stay green; nothing new on the hook path; every new `unsafe` has a concrete SAFETY | MET | `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` (:1092), `an_executor_ended_by_wm_quit_counts_every_later_submit` (:1260) and both death tests are green. Mutation F below shows that V2-I1 is still pinned under the new wait. There are 9 SAFETY comments on the new or changed `unsafe` blocks (timers.rs:76, :85, :93, :101, :127; mod.rs:530, :644, :678, :693), each naming the handle or buffer and how long it lives |
| 4. CHANGELOG, actions.md and README describe the new precision; M13 tightened | MET | The `### Changed` bullet "A macro `Delay` is coarser" is gone. The KS-08 `### Fixed` entry says the wait ends "within about 1 ms of its time and never before". README.md:539 (`Delay` row), :152-153 and the layout tree (:39, :67-68, `platform/timers.rs`). actions.md:318 gives about 0.3 ms on average, 100 x `Delay(5)` in about 0.5 s, and the pre-1803 fallback. My probe agrees with every number. errors.md:195, :198 and :201 name the new failure causes, and those match the code (app.rs:199-200 and mod.rs:523, :637, :660). M13 is tightened in the cut addendum (cut.md:660); impl.md:130 still has the old line (hand-off) |
| 5. Gate green, test count at least 598 | MET | 598 passed, 603 listed |
| results_v2 note "the `code_ran` distrust can be simplified" | DONE | `plan(armed, next, now)` (mod.rs:442). `fired()` clears `armed` (:484), so a signal the wait took is never "kept". A signal that comes early by the `Instant` clock re-arms for the remainder and cannot leave a run stuck |
| Review m1 / m2 (optional, from my hand-off) | DONE | m1: `Executor::thread` doc (mod.rs:322). m2: `die_with_lost_jobs` / `counts_left_by_lost_jobs` (:1302-1348). A defect that runs a lost job every time still fails ("a lost job ran twice"), and the count assertions are unchanged. Mutation replay below |

### Delay pacing (my probe on the real `Executor` at HEAD)
A scratch copy of the export (`analyst/v3/mut`) adds one test that calls the lane's own `delay_lateness` (a real executor
thread with a timestamping `FakeRunner`; no hook, no input). Debug build. Output: `analyst/v3/probe-head-1.txt`.

| `Delay(d)` | HEAD: late avg / median / max | `bb0a0aa` (v2): avg late | `thread::sleep` (v2): avg late |
|---|---|---|---|
| 1 | 0.457 / 0.517 / 0.645 ms | 14.4 ms | 0.36 ms |
| 5 | 0.289 / 0.282 / 0.989 ms | 10.7 ms | 0.16 ms |
| 10 | 0.270 / 0.261 / 0.550 ms | 7.3 ms | 0.24 ms |
| 16 | 0.302 / 0.277 / 0.685 ms | 11.7 ms | 0.29 ms |
| 20 | 0.329 / 0.338 / 0.965 ms | 10.9 ms | 0.27 ms |
| 50 | 0.328 / 0.344 / 0.638 ms | 12.2 ms | 0.24 ms |
| 100 | 0.362 / 0.458 / 0.511 ms | 9.6 ms | 0.28 ms |
| 100 x `[Key, Delay(5)]` | 0.525 s | 1.56 s | 0.520 s |
| 20 x `[Key, Delay(50)]` | 1.005 s (M13: about 1.00-1.05 s) | | |

No gap ended early (asserted for every gap). The rework's own probe (`rework2/probe-head.txt`) gives the same
picture. The owner's latency decision of `62e36a6` is now met for macro `Delay`, to within about 0.1 ms of
`thread::sleep`.

### Mutations (own, at the fix delta, plus the replay of the one the rework named)
All of them were run in `analyst/v3/mut` with `CARGO_TARGET_DIR=analyst/v3/target-mut`, one at a time, each reverted
before the next.

| ID | Mutation | Result | Log |
|---|---|---|---|
| A | `WaitableTimer::new` asks for a plain timer (`Self::create(0)`, timers.rs:42) | **killed** by `a_timer_signals_once_after_its_due_time` ("Windows 10 1803 and later make a high-resolution timer") and by `a_delay_ends_within_a_few_ms_of_its_deadline` ("Delay(5) ended 10.4746ms late in the median"). Probe under the mutation: 100 x `Delay(5)` = 1.556 s | `mut-A-plain-timer.txt` |
| E | `plan` keeps any set timer, even for a sooner deadline (`Next::At(_) if armed.is_some() => Plan::Keep`, mod.rs:446) | **killed** by `the_delay_timer_is_set_again_only_for_another_deadline` (left `Keep`, right `Arm { after: 50ms }`). Only this pure unit test pins it, which is enough for a pure function | `mut-E-keep-stale.txt` |
| F | The inner loop also ends after a pass that ran code (`if calls.made == 0 \|\| calls.made > 0`, mod.rs:627), so the wait can follow a runner call (V2-I1) | **killed** by `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` ("did not run within 1 s") | `mut-F-no-final-drain.txt` |
| G | The wait drops `MWMO_INPUTAVAILABLE` (flags 0, mod.rs:652) | **survived**: all 79 `core::executor` and `hook::` tests are green. See Minor m-1 | `mut-G-no-inputavailable.txt` |
| m2 (rework's) | The epilogue's drain-and-count loop removed (mod.rs:562-564) | **killed**: both death tests are red by assertion with `(ExecutorUnavailable, 0)` | `mut-m2-replay.txt` |

## Acceptance criteria
Rework 2 changes only the executor's wake primitive and its tests. Every row of results.md and results_v2.md stands,
and every test named there is green in my four runs.

| # | Criterion | Evidence | Status |
|---|---|---|---|
| A1 | `rg "\.execute\(\)" src/hook` finds nothing | `the_hook_path_names_no_execute_call` (keyboard_hook_callback.rs:886) | PASS |
| A2 | 100 Sequence runs create no threads | `a_hundred_sequence_runs_use_the_one_executor_thread` | PASS |
| A3 | `Delay(10000)` stops within 100 ms of an epoch change / `swch off` | `delay_10000_stops_within_100_ms_of_an_epoch_change`, `swch_off_stops_a_running_macro_within_100_ms`, `a_reload_stops_a_running_macro_within_100_ms`; manual M2-M4, M10. The epoch bump's `WAKE` now ends the wait through `pump` (mod.rs:689-690) | PASS |
| A4 | 20 slow jobs: each enqueue < 5 ms; the callback < 5 ms with a cold browser | `twenty_slow_jobs_each_enqueue_in_under_5_ms`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`; manual M1 | PASS (manual M1 open) |
| A5 | A full queue drops and counts | `a_full_queue_drops_the_job_and_counts_it` | PASS |
| S1-S10 | executor thread, STA, queue, job, placement, Hold split, gauge, start and stop order, injectable runner, Phase-1 leftovers | as in results.md:43-52 | PASS |
| S11 | README, actions.md, CHANGELOG updated | now describe the high-resolution `Delay` (re-check 4 above) | PASS |
| R3-T | the `Delay` keeps millisecond precision (ruling) | `a_delay_ends_within_a_few_ms_of_its_deadline`, the `platform::timers` tests, `macros_never_sleep_or_block_in_a_wait` (exactly one wait, no `GetMessageW`); probe above | PASS |

## Issues claimed closed
| ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-08 (slow actions; the watchdog is 2.4) | `the_hook_path_names_no_execute_call`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`, `a_hold_runs_its_slow_action_on_the_executor` | yes (results.md: red at `fcce19b`, compile-red at `6edfa0f`) | Closed for the slow actions |
| gaps "No cancellation for macro workers" | the A3 tests, `a_cancelled_waiting_run_never_resumes`, `a_job_queued_before_an_epoch_change_does_not_run` | yes (`cf3c9f6`, `ba3122f`) | Closed (no per-binding run limit, N1) |
| KS-33 (Sequence half) | `a_hundred_sequence_runs_use_the_one_executor_thread`; `spawn_sequence` removed | compile-red X1 | Closed for Sequence |
| results_v2 P1 (lane regression, not an audit ID) | `a_delay_ends_within_a_few_ms_of_its_deadline` | yes: my replay on `8ba1a0b`, median 10.72 ms | Resolved |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking or allocation, symmetry) | OK | Unchanged since v2 (empty diff for `src/hook` and `src/engine`; `submit` and `wake` are untouched). The v2 cost table stands: about 63 ns per event for the gauge, about 6 µs per slow fire |
| Engine purity, matching per owner decisions | OK | `src/engine` changed only in docs over the whole lane |
| Thread ownership, sentinel, atomics, SAFETY | OK | The timer is created, set, cancelled, waited on and closed only on the executor thread. `WaitableTimer` is `!Send` because `HANDLE` is a raw-pointer struct with no `Send` impl (windows-0.62.2 Foundation/mod.rs:5291). It is made before the queue exists and outside the `catch_unwind` closure (mod.rs:520), and it is dropped in the epilogue (:556) on every path, panic included. No atomic added. The SAFETY comments are concrete (above) |
| The wait loop (new) | OK | One wait (mod.rs:647-654). It can only return the timer (auto-reset synchronization timer: `fired()` and `continue`), input (`pump`) or `WAIT_FAILED` (`[ERROR]` and `Died`); `INFINITE` and the absence of `MWMO_ALERTABLE` rule out the other codes. `pump` (:677-701) returns at the first `WAKE` and goes back to the drain. Messages left behind wake the next wait through `MWMO_INPUTAVAILABLE`, `WM_QUIT` is R3's death, and everything else is dispatched (STA). V2-I1 holds: the last act before the wait is a drain that found the queue `Empty`, followed only by `timer.arm` and, on failure, `drop_timed`, neither of which calls the runner. A stale or early signal causes one extra poll and a re-arm (`fired()` clears `armed`), never a stuck run or a busy loop: the timer is consumed by the wait and signals at most once per `set`. Input cannot be starved, because the timer wins at most one wait per arm |
| Start failure | OK | A timer that cannot be made in either form fails `Executor::start` (mod.rs:520-527, :300-303), so the daemon exits through the existing `[FATAL]` path before it installs any hook (app.rs:199-209). This is safe for typing. Declared as deviation 2, with the alternative weighed; I agree |
| Config reload, IPC | OK | unchanged |
| No key logging | OK | unchanged |
| Owner decision "lowest possible latency" (`62e36a6`) | OK (was FAIL in v2) | Hook path as in v2. Macro `Delay` within 1 ms (probe). A new job's first step still starts without a timer (`Resume::Now`, runs.rs:104-121, unchanged) |

## Review status
- review_r2: APPROVED (Critical 0, Important 0, Minor 2), and both Minors are now done. test_r2 is GREEN at `6c83406`.
  triage_r1: F1 CONFIRMED and resolved, F2 REFUTED. No CONFIRMED Critical or Important item is open.
- **Process note**: rework 2 changes source code (a new `platform` module with 5 `unsafe` blocks and a rewritten wait
  loop), but no code-reviewer or tester round ran on it. The orchestrator routed it straight to this re-check. To
  make up for that, I did the delta-scoped verify that WORKFLOW.md:73-74 describes:
  - red-first replayed on the parent;
  - four new mutations at the fix, three killed;
  - the rework's mutation replayed;
  - four full suites on HEAD;
  - a full read of the delta against section 4.

  I found nothing that rates Critical or Important. If the orchestrator wants the reviewer's separate eyes on
  `platform/timers.rs` before 2.2 reuses it on the main thread, that review fits 2.2's own verify round.

## Docs and hygiene
- In sync for everything the lane changes (re-check 4). `default_config.ron:151` makes no claim about precision.
- Unchanged hand-offs to the Phase-2 book sync (ruling Q2): `architecture.md` (still describes actions on the hook
  thread, architecture.md:215, :577, and it now also needs the executor's single wait and timer), `audit.md`,
  `roadmap.md`, `gaps.md`, `glossary.md`, `overview.md`, `build-and-run.md`.
- New: master's roadmap 2.2 "Files" (roadmap.md:471 on master) says "new `src/platform/timers.rs`". After this merge,
  2.2 reuses that file and does not create it.

## Technical debt
- Minor m-1, `MWMO_INPUTAVAILABLE` is unpinned (mutation G survived), at mod.rs:652. R3-T prescribes the flag and the
  code has it, with the reason at :666-668. Without it, a message left in the queue behind a `WAKE` (a COM message, a
  second `WAKE`) would wait for the next new input. No job is lost, because the drain has already taken them, but a
  foreign message on the STA could be delayed indefinitely. Recorded only here. It belongs in 2.2's notes, because
  2.2 adds the same wait on the main thread, where a test that posts a foreign message behind a `WAKE` (or behind
  the timer) fits naturally.
- N1, unbounded active runs: known-issues row 36.
- R-7, a hold-timer wake to a reused thread id at exit: lane files only; it goes to roadmap 2.2's notes (book sync).
- `SystemFunction::{LockWorkstation, ShowDesktop, TaskManager}` have no placement: impl.md only.
- Blocking steps serialise the executor: declared in CHANGELOG, README and actions.md (ruling R1).
- Manual checks M1-M13 are unrun (M11 and M12 included). The table is in impl.md, with M13 superseded by cut.md:660.
  It goes to a Checkpoint 2 list in the book sync.
- The `UOI_TIMERPROC_EXCEPTION_SUPPRESSION` item of v2 is moot: the executor no longer has a thread timer of its own.
- No new TODO, FIXME or HACK.

## Problems
None that block. Minor m-1 above is optional: a test, owned by the `tester`, best done in item 2.2.

## Positives
- The fix follows R3-T line by line, and the wrapper sits where 2.2 can reuse it. A timer's signal cannot be eaten by a
  modal loop, so the old `code_ran` distrust is gone and the plan got simpler.
- The red test fails by about 2x its bound on the old loop and passes with about 10x margin at HEAD, so it is both a
  sharp regression test and CI-tolerant. The red was replayed independently.
- The rework measured with the real executor and documented the measured numbers, and those agree with mine to
  within 0.1 ms.
- The retry in the death tests removes a scheduling flake without weakening either assertion (the m2 mutation is still
  killed).

## Hand-offs
- **Orchestrator**:
  - `rulings.md:22` and cut.md D6 (:118-128) keep the superseded thread-timer text; R3-T overrides both.
  - CLAUDE.md (master) should list `src/platform/` (`timers.rs`) in the layout, change "Macro steps run on worker
    threads" (CLAUDE.md:83) to the executor, and take the other CLAUDE.md and agent-file hand-offs of
    results.md:144-156.
- **Phase-2 book sync**:
  - as results.md:137-142;
  - plus `architecture.md` (the executor's wait and timer);
  - the roadmap 2.2 "Files" wording above;
  - R-7 and m-1 in 2.2's notes;
  - the M1-M13 table, with M13 from cut.md:660, in a Checkpoint 2 list.
- **CI watch**: `a_delay_ends_within_a_few_ms_of_its_deadline` and `a_timer_signals_once_after_its_due_time` are
  the first real-timer tests to reach CI. They assert a high-resolution timer exists and that the median lateness is
  under 5 ms. A flake there goes to the orchestrator; ruling open question 3 forbids loosening the test without a new
  ruling.

## Notes for later lanes
- 2.2: reuse `platform::timers::WaitableTimer` and the `MsgWaitForMultipleObjectsEx` / `pump` shape for the main
  loop. The probe shape (the lane's `delay_lateness` with a timestamping runner) also works as the timer row of 2.2's
  latency bench.
- 2.4 can read `longest_callback_us`, `executor_unavailable` and `active_runs`. 3.3: de-elevation goes in
  `Win32Runner::action` and `run_step`. 3.5: `Binding.action: Arc` and N1.
