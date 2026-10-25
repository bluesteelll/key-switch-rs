# Results analysis v2: p2-2.1-executor / roadmap 2.1 Action executor (KS-08), re-check after rework 1

Branch `feat/action-executor` @ `bb0a0aa` (32 commits; merge-base `7d757a1`). `master` is now `62e36a6`, one commit
ahead of the merge-base. Tree clean (`git status --porcelain` empty).
Inputs: results.md (my round-1 verdict), rework_1.md, rulings.md including the new **Round 3** (rulings.md:121-134,
written after results.md), commit `bb0a0aa` in full, master's new commit `62e36a6` in full, and the executor, callback and
hook code paths that Round 3 asks about.

## VERDICT: REWORK -> architect

**Summary**: Rework 1 resolves every point of round-1 P1. The docs now state the new `Delay` pacing, my own probe of the
real executor confirms the numbers, and no source changed. My gate on a clean export is green (593 passed, 5 manual
ignored, 598 listed; the list is identical to round 1). One thing changed since round 1. `master` `62e36a6` records the
owner decision "binds run with the lowest possible latency", which names timer resolution, and roadmap 2.2 now forbids
rounding deadlines to the 15.6 ms tick. Under that decision, the lane's coarser macro `Delay` (on average +10 ms, up
to about +22-30 ms, where it was about 0.3 ms; short-Delay macros run 3x slower) is "added delay ... a defect unless the
cut justifies it and the orchestrator accepts it". The cut's justification is a false premise, and nobody has accepted
the regression with the correct facts. The hook path itself meets Round 3: about 63 ns per event, and a slow fire
costs about 6 µs instead of 27 µs (Sequence) or a blocking shell or process call (Launch, OpenUrl).

## Quality gate (own run, clean export of `bb0a0aa`)
Export: `git archive bb0a0aa` into `D:/tmp/ks-orch/p2-2.1-executor/analyst/v2/src-export`, with
`CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.1-executor/analyst/v2/target`. Output: `analyst/v2/gate.txt`.

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0). Fresh target dir, then a second run after touching the crate roots: "Checking key-switch-rs v0.1.0 (…src-export) … Finished" |
| `cargo test --locked` | lib 587 passed / 5 ignored; `ipc_roundtrip` 6 passed; swch and doc-tests 0. **593 passed, 0 failed, 5 ignored.** Log: `D:/tmp/ks-orch/p2-2.1-executor/analyst-test-v2.txt`. The suite prints nothing beyond libtest. |
| test count vs master | 598 listed (`analyst/v2/branch_tests.txt`), byte-identical to round 1's list. All 543 tests of master `7d757a1` are present (`comm -23` is empty). `62e36a6` changes no code. |
| `cargo test --locked -- --list --ignored` | 5, all `manual:` (`the_win32_runner_opens_a_url`, `foreground_context_names_the_focused_notepad`, `hold_the_daemon_pipe_name_for_a_manual_start`, `the_menu_mask_key_is_inserted_into_the_input_stream`, `live_registry_with_the_language_hotkey_not_assigned_has_no_combination`) |
| `cargo doc --no-deps` | the same 4 warnings as master; none new |
| `mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/analyst/v2/book` | OK (only the known mdbook-mermaid version warning). The new link `../roadmap.html#21-action-executor` resolves, and every in-page anchor of `config/actions.html` exists |
| trailers | 32 of 32 commits end with `Co-Authored-By: Claude (Opus|Sonnet) 5.5 <noreply@anthropic.com>`. Sonnet only on the 4 tester commits (`c94acd5`, `2e51c2a`, `a0a8b82`, `6c83406`) |
| merge | No file overlap between `master...feat/action-executor` and `62e36a6` (CLAUDE.md, roadmap.md), so the `--no-ff` merge has no conflict |
| Cargo.toml / Cargo.lock / build.rs | untouched by the lane |

## Re-check of round-1 problems
| Item | Status | Evidence |
|---|---|---|
| P1.1 CHANGELOG `[Unreleased]` -> Changed bullet citing 2.1 | MET | `CHANGELOG.md:371-380`, inside `### Changed` (`:132`-`:386`) |
| P1.2 actions.md §14.3 drops the parity claim, states the floor and the lateness | MET | `book/src/config/actions.md:318` |
| P1.3 README Delay row and the template do not contradict | MET | `README.md:537` rewritten. `default_config.ron:151` is unchanged and makes no precision claim |
| P1.4 no source change | MET | `git diff --stat 6c83406 bb0a0aa -- src Cargo.toml Cargo.lock build.rs` is empty |
| P1.5 fmt and mdbook green | MET | gate above |
| Optional reload wording | DONE | "a saved `config.ron` that loads" at CHANGELOG.md:511-517, actions.md:51, :297 and README.md:511-515. It matches the code: only `Ok` calls `update_bindings` (`config_watcher.rs:157-169`), and only that calls `cancel_actions` (`keyboard_hook.rs:120-127`) |
| The developer's two wording deviations | ACCEPTED, they are correct | My round-1 prescription was wrong on two facts. In my probe (below), `Delay(1)` and `Delay(5)` sometimes end in 8.1 and 6.8 ms, so "at least 10 ms" would be false. The worst lateness exceeds one tick (22.4 ms here, 30.3 ms in rework_1.md). The text as written agrees with every sample. |
| Mutations | none named | P1 named no mutation. Instead I re-ran the measurement behind P1, this time on the lane's real `Executor` (rework_1 measured a re-implementation). |
| Review m1 / m2 (Minor) | open | `executor/mod.rs:317` `pub fn thread` still has no `///`; the 200 ms margins at `mod.rs:1254`, `:1299` are unchanged |

## Round 3: cost on the hook path, before (master `7d757a1` = `62e36a6` code) and after (`bb0a0aa`)
Probe: two scratch crates in the session scratchpad, built `--release --offline` against `Cargo.lock`, with no
`requireAdministrator` in either binary. `bench-after` drives the lane's real `Executor::start`, `ExecutorHandle::submit`,
`TargetWindow::for_action` and `counters::record_callback` with a runner that only takes timestamps. `bench-before` calls
master's `spawn_sequence` with an empty macro. No hook, no `SendInput`, no program start, no registry write. The figures
are from the owner's machine under its normal load.

| Path | Before | After | Note |
|---|---|---|---|
| Every event that passes `code < 0` and remapping-off | `decide_event` + engine + outcome | same code (`decide_event`/`HookEnv` are textually identical) **+ 2 x `Instant::now` + 1 `fetch_max`** (keyboard_hook_callback.rs:363, :376): **62.6 / 65.1 / 63.1 ns** (3 runs x 2M) | accepted by Round 3. The two early passes are not timed and unchanged |
| A fast fire (`PressKey`, `Media`, `ToggleCapsLock`, `SwitchLanguage*`, `PostMessage`, `DoNothing`) | the action inline | the same functions inline (`FastAction::run`, windows_actions.rs:142-156). A failed post is counted, not logged (`6a1db71`) | no added work; the failure path got cheaper |
| A `Sequence` fire | `GetForegroundWindow` + `thread::spawn` (allocation, CreateThread, the KS-33 `expect`): median **27.0 µs**, avg 32.3, p99 96.0, max 475.2 µs (2000 runs) | `as_fast` + `for_action` (the same `GetForegroundWindow`) + `ArcSwapOption::load` (4.8 ns) + `epoch` Acquire load + `try_send` into a preallocated slot + one `PostThreadMessageW`: median **5.6 µs**, avg 5.4, p99 20.0, max 53.0 µs | about 5x cheaper. No allocation, lock or log; the `PostThreadMessageW` syscall dominates |
| A `Launch` / `OpenUrl` fire | `Command::spawn` / `ShellExecuteW` **on the hook thread** (KS-08: a cold browser stalls every keyboard event for seconds) | the same submit path: median **6.7 µs**, avg 7.4, p99 26.0, max 98.6 µs (OpenUrl) | the KS-08 stall is gone |
| From submit to the first step of a new job (Round 3: "without waiting for a timer") | (a new thread per macro) | median **7.1 / 8.9 µs** (OpenUrl / Sequence), p99 40.7 / 37.0, max 152.4 / 62.0 µs | MET. `Runs::start` queues `Resume::Now` (runs.rs:104-121), and the loop polls in the same pass (mod.rs:618-619). Pinned by `runs::tests::a_run_goes_up_to_its_first_delay_and_resumes_at_its_deadline` (runs.rs:363: the first poll at t = 0 runs F13) |
| A Hold timer | a thread per Hold | unchanged | item 2.2 |
| **Macro `Delay` pacing** (40 gaps per row; none ended early in 320 gaps) | `thread::sleep` (high resolution): see right | executor thread timer: see below | the regression, P1 |

| `Delay(d)` | executor avg / max gap (ms) | `thread::sleep` avg / max (ms) |
|---|---|---|
| 1 | 15.39 / 17.05 (min 8.11) | 1.36 / 1.88 |
| 5 | 15.68 / 24.62 (min 6.76) | 5.16 / 5.55 |
| 9 | 15.72 / 20.56 | 9.24 / 9.49 |
| 10 | 17.31 / 25.57 | 10.24 / 11.14 |
| 16 | 27.65 / 38.37 (median 30.56) | 16.29 / 16.65 |
| 20 | 30.93 / 37.26 | 20.27 / 20.55 |
| 50 | 62.20 / 67.72 | 50.24 / 50.57 |
| 100 | 109.62 / 116.33 | 100.28 / 100.61 |
| 100 x `Delay(5)`, whole macro | 1.561 s, 1.559 s | 0.520 s, 0.520 s |

## Acceptance criteria
The code and the tests are unchanged since round 1 (empty source diff; identical test list; all green in my run), so
every row of results.md stands. The only change is S11.

| # | Criterion | Evidence | Status |
|---|---|---|---|
| A1 | `rg "\.execute\(\)" src/hook` finds nothing | `the_hook_path_names_no_execute_call` (keyboard_hook_callback.rs:886), red at `fcce19b` | PASS |
| A2 | 100 Sequence runs create no threads | `a_hundred_sequence_runs_use_the_one_executor_thread` (executor/mod.rs:748) | PASS |
| A3 | `Delay(10000)` stops within 100 ms of an epoch change / `swch off` | `delay_10000_stops_within_100_ms_of_an_epoch_change` (mod.rs:768), `swch_off_stops_a_running_macro_within_100_ms` and `a_reload_stops_a_running_macro_within_100_ms` (keyboard_hook.rs:581, :588); manual M2-M4, M10 | PASS |
| A4 | 20 slow jobs: each enqueue < 5 ms; the callback < 5 ms with a cold browser | `twenty_slow_jobs_each_enqueue_in_under_5_ms` (mod.rs:1019), `carry_out_returns_in_under_5_ms_while_slow_actions_block` (keyboard_hook_callback.rs:818); manual M1. My probe: p99 26 µs | PASS (manual M1 open) |
| A5 | A full queue drops and counts | `a_full_queue_drops_the_job_and_counts_it` (mod.rs:1072) | PASS |
| S1-S10 | executor thread, STA, queue, job, placement, Hold split, gauge, start and stop order, injectable runner, Phase-1 leftovers | as in results.md:43-52; every named test is green in analyst-test-v2.txt | PASS |
| S11 | README, actions.md, CHANGELOG updated | now including the Delay pacing (CHANGELOG.md:371-380, actions.md:318, README.md:537) | PASS |

## Issues claimed closed
| ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-08 (slow actions; the watchdog is 2.4) | `the_hook_path_names_no_execute_call`, `carry_out_returns_in_under_5_ms_while_slow_actions_block`, `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`, `a_hold_runs_its_slow_action_on_the_executor` | yes: red by assertion at `fcce19b`, compile-red at `6edfa0f`; each red precedes its fix | Closed for the slow actions |
| gaps "No cancellation for macro workers" | the A3 tests, `a_cancelled_waiting_run_never_resumes` (both levels), `a_job_queued_before_an_epoch_change_does_not_run` | yes: `cf3c9f6`, `ba3122f` | Closed (no per-binding run limit, N1) |
| KS-33 (Sequence half) | `a_hundred_sequence_runs_use_the_one_executor_thread`; `spawn_sequence` removed | compile-red X1 | Closed for Sequence |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback | OK | `code < 0` first (keyboard_hook_callback.rs:345); the sentinel check before any state (`decide_event`, :187); one outcome on every path (:377-383). Per-event addition: the Round-3 gauge only. Slow fire: only the prescribed operations plus one lock-free `ArcSwapOption::load` (4.8 ns; the hook thread's arc-swap node exists already from `bindings()`); no allocation, lock, log or new Win32 call. Symmetry: `suppress` comes from the engine, unchanged |
| Engine purity, owner matching decisions | OK | `src/engine` changed only in docs |
| Threads, sentinel, atomics, SAFETY | OK | unchanged since round 1 (results.md:70) |
| Config reload, IPC | OK | a failed or deleted reload keeps the bindings and the running macros (config_watcher.rs:157-169); the docs now say so |
| No key logging | OK | unchanged |
| Owner decision "lowest possible latency" (`62e36a6`) | **FAIL (P1)** | hook path OK (table above); macro `Delay` timer resolution regressed, and nobody has justified or accepted it |

## Review status
- review_r2: APPROVED (Critical 0, Important 0, Minor 2). The last tester round, test_r2, is GREEN at `6c83406`. Rework 1 changes
  docs only, so both still cover the code, and my gate covers `bb0a0aa`.
- triage_r1: F1 CONFIRMED and resolved (`a875960`); F2 REFUTED. No CONFIRMED Critical or Important item is open.
- No review round has applied Round 3 (rulings.md:131-132, written after review_r2). I applied it above and found nothing
  on the hook path that rates Important.
- Process note: rework_1.md:4-6 says the developer "took the analyst's recommendation as [the orchestrator's]
  decision" because no P1 ruling existed. The fork was the orchestrator's to decide. It is still undecided, and the owner
  decision of `62e36a6` has since changed the facts it rests on.

## Docs and hygiene
- In sync for everything the lane changes, now including the Delay pacing and the reload wording. Lines added under
  `src` are ASCII except em dashes kept from the template lines they rewrite: non-ASCII lines under `src` went down
  from 105 to 101.
- Unchanged hand-offs: audit.md, roadmap.md, gaps.md and the other non-config pages go to the Phase-2 book sync
  (ruling Q2). One round-1 hand-off is obsolete: open question 7 is now decided (`62e36a6`), so it cannot be "extended
  to macro `Delay`". P1 replaces that hand-off.

## Technical debt
- N1, unbounded active runs: known-issues row 36 records it ("not capped").
- R-7, a hold-timer wake to a reused thread id at exit: recorded in the lane files only. It belongs in roadmap 2.2's notes (book sync).
- `SystemFunction::{LockWorkstation, ShowDesktop, TaskManager}` have no placement (new feature 1): recorded in impl.md only.
- `UOI_TIMERPROC_EXCEPTION_SUPPRESSION` is not set: recorded nowhere in the repo. Moot if P1 is fixed by option A
  (there is no thread timer).
- Blocking steps serialise the executor: declared in the CHANGELOG, README and actions.md. Ruled (R1).
- M12 (`Window(Restore)` to the front) is still unrun (impl.md open point).
- Delay pacing (P1): recorded in the CHANGELOG and actions.md as behaviour. If option B is taken, it must also become a
  2.2 acceptance line, or it is lost as debt.
- No new TODO, FIXME or HACK.

## Problems

### P1. The macro `Delay` adds about 10 ms on average (up to about 22-30 ms) per step, against the owner's latency decision
- **What**: a `Delay` now resumes on a win32k thread timer (`SetCoalescableTimer(None, 0, ms, None,
  TIMERV_NO_COALESCING)`, `src/core/executor/mod.rs:474`, woken through `GetMessageW` at `:653`). Windows raises it to
  `USER_TIMER_MINIMUM` (10 ms) and delivers it on a clock tick. Master's `thread::sleep` uses std's high-resolution
  waitable timer. Measured on the real executor: `Delay(5)` averages 15.7 ms (sleep: 5.2), `Delay(20)` 30.9 ms (20.3),
  and 100 x `Delay(5)` takes 1.56 s (0.52 s). No Delay ended early, and nothing is stuck or eaten.
- **Why it now blocks ACCEPT** (criterion 5): `62e36a6`, which landed after my round-1 verdict, adds the owner decision
  that "the key-to-action path is the top non-functional priority. Every cut states its effect on that path
  (… timer resolution, wake-ups), and added delay there is a defect unless the cut justifies it and the orchestrator
  accepts it" (CLAUDE.md on master). Roadmap 2.2 now says that "gesture deadlines are not rounded to the 15.6 ms system
  tick" and decides open question 7, which was the basis of my round-1 advice to accept. As things stand:
  - The cut justifies the timer only by a false premise ("the same as the old `thread::sleep`", cut.md:124), and the
    acceptance in rulings.md:22 repeats it. Neither has been corrected.
  - No P1 ruling exists (rework_1.md:4).
  - Round 3 (rulings.md:121-134) requires that the first step not wait for a timer, which is met. It says nothing on
    Delay resolution.

  The steps after a `Delay` are the bind's output, and the decision lists timer resolution as part of the path, so I
  treat this as in scope. The orchestrator may rule otherwise (option B).
- **Root cause**: cut D6 and rulings R1 and V2-M3 chose the thread timer because it was believed to have the same
  granularity as std's sleep. Since Rust 1.75, std's Windows sleep uses
  `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`. The mechanism is a design choice set by the rulings, so the fix starts with
  the cut, not with the code.
- **Return to**: `architect`. The orchestrator amends R1 and V2-M3 in the same step.
- **What to keep**: everything except the wake primitive: `Runs` with `Instant` deadlines, epochs, `plan()`, the
  drain-before-wait structure of V2-I1, the R3 death path and the hook side.
- **Option A (recommended)**: replace `DelayTimer`'s thread timer and the bare `GetMessageW` wait with these steps:
  1. Create a high-resolution waitable timer: `CreateWaitableTimerExW(None, PCWSTR::null(),
     CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS)` and arm it with `SetWaitableTimer` (relative, in 100 ns
     units). Fall back to a plain waitable timer if the flag is refused.
  2. Wait with `MsgWaitForMultipleObjectsEx(Some(&[timer]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`.
  3. Then drain with `PeekMessageW(PM_REMOVE)`, keeping today's handling of `WAKE`, `WM_QUIT` (`[ERROR]` and `Died`),
     a failed wait (`WAIT_FAILED` takes the place of `-1`) and the dispatch of foreign messages.
  4. Close the handle with a `Drop` guard on every path, in the epilogue too.

  All of these APIs are in `windows` 0.62.2 under features the crate already enables (`Win32_System_Threading`,
  `Win32_UI_WindowsAndMessaging`), so no new dependency is needed. A modal loop cannot eat a kernel timer's signal, so
  the `code_ran` distrust (mod.rs:430-446) can be simplified, and the two-tick re-arm effect of rework_1.md goes away.
  Item 2.2 prescribes the same primitive for the main loop, so one helper can serve both.
- **Option B (orchestrator only, no code round)**: the orchestrator writes a P1 ruling that accepts the added Delay
  latency under the latency rule, with the measured numbers above, and corrects rulings.md:22 and cut.md:124. Roadmap 2.2
  then gets an acceptance line that moves the executor's `Delay` onto 2.2's high-resolution timer before v0.2.0, for
  example "a macro `Delay(5)` ends within N ms and never early", on master or in the named Phase-2 book sync.
  Defensible, because Phase 2 ships only as v0.2.0, after 2.2.
- **Re-check condition for option A**:
  1. A revised cut and a ruling that amends R1 and V2-M3, critic-checked as the workflow asks. The cut states the path
     effect as the latency rule requires.
  2. A red-first real-thread executor test: a run of `[Key, Delay(d)] x 20` (for example d = 5 and 20) whose gaps are
     never below d and stay within a bound the cut fixes. For reference, std's sleep here: avg d + 0.2-0.4 ms, max d + 0.5-1.1 ms over all eight rows. It must be red on `bb0a0aa` (avg d + 10.7-10.9 ms for d = 5 and 20) and green after.
  3. `a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` and the R3 death tests stay green; nothing new
     on the hook path; every new `unsafe` has a concrete `// SAFETY:`.
  4. CHANGELOG.md:371-380, actions.md:318 and README.md:537 describe the new precision (the Changed bullet goes or is
     rewritten), and manual check M13 is tightened.
  5. The gate is green and the test count is at least 598.
- **Re-check condition for option B**: the ruling and the corrections exist, and the 2.2 acceptance line exists or its
  owner is named. Then I ACCEPT with no further change.

## Positives
- The hook path is lean and faster than master's on every slow fire, and costs about 63 ns per event for the gauge that
  the latency priority needs.
- A new job's first step starts in microseconds with no timer. This holds by construction (`Resume::Now`), and a fake-clock
  test pins it.
- The rework measured instead of copying the prescribed sentence and corrected two of my facts. The docs now match the
  real executor to within the scatter of the probe.
- Red-first discipline, exhaustive placement matches and the death epilogue's ownership (round 1) are unchanged.

## Hand-offs
- **Orchestrator**: decide P1, option A or option B, and write it into rulings.md either way; correct rulings.md:22.
  The CLAUDE.md and agent-file hand-offs of results.md:144-156 still apply (the `executor/` layout line, "Macro steps
  run on worker threads" at CLAUDE.md:83, principles 1 and 5, six agent files).
- **Phase-2 book sync**: as results.md:137-142, minus the open-question-7 item (decided by `62e36a6`). Add R-7 to 2.2's
  notes and the M1-M13 manual table to a Checkpoint 2 list.
- **Optional, while fresh** (folds into option A's developer round): review m1 (`Executor::thread`, mod.rs:317) and
  review m2 (the 200 ms margins, mod.rs:1254, :1299).

## Notes for later lanes
- 2.2: the latency bench it adds should cover `ExecutorHandle::submit` (hook side, about 6 µs here), submit to first
  step (about 9 µs) and `Delay(d)` pacing, next to `Engine::on_key`/`on_timer`. The probe shape above works without
  a hook, using `Executor::start` with a timestamping `Runner`.
- 2.4: it can read `longest_callback_us`, `executor_unavailable` and `active_runs`. 3.3: de-elevation goes in
  `Win32Runner::action` and `run_step`. 3.5: `Binding.action: Arc` and N1.
