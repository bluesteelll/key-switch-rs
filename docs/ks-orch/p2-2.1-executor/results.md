# Results analysis: p2-2.1-executor / roadmap 2.1 Action executor (KS-08)

Branch `feat/action-executor` @ `6c83406`, 31 commits on `master` @ `7d757a1` (merge-base = master tip), tree clean.
Inputs read: spec.md, rulings.md (rounds 1 and 2), cut.md (incl. developer addendum), critique.md, critique_v2.md,
impl.md, test_r1.md, review_r1.md, triage_r1.md, fix_r1.md, test_r2.md, review_r2.md; roadmap 2.1, audit KS-08, gaps
"No cancellation for macro workers"; the whole diff `git diff master...feat/action-executor` and every changed source
file in full (executor mod.rs/runs.rs/testing.rs, windows_actions.rs, keyboard_hook.rs, keyboard_hook_callback.rs,
app.rs, sequence.rs, counters.rs, logging/mod.rs, writer.rs, loader.rs, system_function.rs, main.rs, engine docs).

## VERDICT: REWORK -> doc-writer

**Summary**: The code meets every acceptance line of roadmap 2.1 and the spec, carries out every ruling, and my own
gate is green (593 passed, 5 manual ignored; master has 543 tests and none was lost). The last tester round is GREEN,
the last review is APPROVED, and no CONFIRMED Critical or Important item is open. One gap blocks ACCEPT criterion 7.
Macro `Delay` steps are now much coarser than before: at least 10 ms, and up to about one clock tick late, where they
used to be accurate to about 1 ms. The CHANGELOG does not say so, and the config reference
(`book/src/config/actions.md:318`) wrongly says the new granularity is "as with the `thread::sleep` it replaces".
The fix is docs only.

## Quality gate (own run, lane tree @ 6c83406)
| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (after touching the three crate roots: "Checking key-switch-rs ... Finished") |
| `cargo test --locked` | lib 587 passed / 5 ignored; ipc_roundtrip 6 passed; swch and doc-tests 0. Total 593 passed, 0 failed, 5 ignored. Log: `D:/tmp/ks-orch/p2-2.1-executor/analyst-test.txt`. The suite prints nothing beyond libtest. |
| `cargo test --locked --lib -- --test-threads=1` | 587 passed, 0 failed (the F1 order dependence is gone) |
| `core::executor` + `hook::` x3 | 78 passed, 0 failed, 3 ignored each time |
| test count vs master | master `7d757a1`: 543 tests (`--list`, on a `git archive` extract). Branch: 598. `comm -23` of the two sorted lists is empty, so no test was removed or renamed. No `-#[test]` or `-#[ignore]` lines in the diff. |
| `cargo test --locked -- --list --ignored` | 5 tests, all `manual:`; the lane adds one: `core::executor::tests::the_win32_runner_opens_a_url` (M9) |
| `cargo doc --no-deps` | the same 4 warnings as master (loader.rs:156, :176, keyboard_hook.rs:170, vk_name.rs:4); none new |
| `mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/analyst/book` | OK (only the known mdbook-mermaid version warning); new anchors s13-1, s14-1, s15-3, s18-6, s11-8 exist; developer link check: 7877 links, 0 broken |
| trailers | 31 of 31 commits carry `Co-Authored-By: Claude (Opus|Sonnet) 5.5`; Sonnet only on the 4 tester commits |
| Cargo.toml / Cargo.lock | untouched; no new dependency |

## Acceptance criteria
| # | Criterion (spec / roadmap 2.1) | Evidence | Status |
|---|---|---|---|
| A1 | `rg "\.execute\(\)" src/hook` finds nothing | Grep over `src/hook`: no matches. `hook::keyboard_hook_callback::tests::the_hook_path_names_no_execute_call` (keyboard_hook_callback.rs:886); red by assertion at `fcce19b` (`:219: action.execute();`, tester replay). `BindAction::execute` and `Binding::execute` were removed. | PASS |
| A2 | 100 `Sequence` runs create no threads | `core::executor::tests::a_hundred_sequence_runs_use_the_one_executor_thread` (executor/mod.rs:748): every record is on `action-executor`, not on the test thread. `spawn_sequence` is gone (grep). The only non-test spawn added is the executor itself (`mod.rs:290`). | PASS |
| A3 | `Delay(10000)` stops within 100 ms of an epoch change (real thread, Delay-only job, fake runner); roadmap: of `swch off` | `delay_10000_stops_within_100_ms_of_an_epoch_change` (mod.rs:768); `hook::keyboard_hook::tests::swch_off_stops_a_running_macro_within_100_ms` (keyboard_hook.rs:581) and `a_reload_stops_a_running_macro_within_100_ms` (:588); exit: `stop_interrupts_a_delay_drops_the_queue_and_joins` (mod.rs:1160). Manual M2-M4, M10. | PASS |
| A4 | 20 slow jobs (500 ms fake runner): each enqueue < 5 ms; roadmap "callback stays under 5 ms" with a cold browser | `twenty_slow_jobs_each_enqueue_in_under_5_ms` (mod.rs:1019) and `carry_out_returns_in_under_5_ms_while_slow_actions_block` (keyboard_hook_callback.rs:818), both "every one of 20, one whole-batch retry" as ruled. The cold-browser run is manual M1 (cut.md manual table). | PASS (manual M1 open) |
| A5 | A full queue drops the job and bumps the counter | `a_full_queue_drops_the_job_and_counts_it` (mod.rs:1072); the tester's mutations M9 and M27 make it red | PASS |
| S1 | Executor thread: own `GetMessageW` loop, STA COM (`COINIT_APARTMENTTHREADED \| COINIT_DISABLE_OLE1DDE`), balanced `CoUninitialize` | `the_executor_runs_jobs_in_a_single_threaded_apartment` (mod.rs:1402), `the_apartment_guard_uninitialises_com_on_drop` (:1417), green alone after F1. `a_delay_resumes_its_run_after_its_deadline` (:861) shows WM_TIMER delivered through the loop. Code: mod.rs:370-397, :527-575, :590-685. | PASS |
| S2 | Bounded queue; overflow counted; the hook path never calls `log::` | A5; `logging::tests::the_functions_on_the_hook_path_log_nothing` (logging/mod.rs:392), red at `37b1847` on the 3 predicted sites; `only_the_startup_banner_prints_outside_src_bin` (:321) covers the callback file whole | PASS |
| S3 | Job = (action, hwnd, epoch) | `jobs_run_in_the_order_they_were_queued_with_their_target` (mod.rs:1130), `a_job_queued_before_an_epoch_change_does_not_run` (:1107), `runs::tests::a_job_of_an_old_epoch_never_starts` (runs.rs:465). The action travels by value, not as `Arc`: declared deviation 1, ruling Q1. | PASS (deviation accepted) |
| S4 | Sequence on the executor; no thread per macro run; `Delay` is a wait, not `thread::sleep` | `macros_never_sleep_or_block_in_a_wait` (mod.rs:1439), red at `ba3122f` (`sequence.rs:133`) | PASS |
| S5 | Only the fast actions stay in the callback | `core::windows_actions::tests::only_launch_open_url_and_sequence_leave_the_firing_thread` (windows_actions.rs:396), `carry_out_queues_the_slow_actions_in_order_and_runs_none_here` (keyboard_hook_callback.rs:757); `as_fast` is exhaustive, with no `_` arm | PASS |
| S6 | A Hold on `hold-timer` follows the same split | `a_hold_runs_its_slow_action_on_the_executor` (keyboard_hook_callback.rs:847): GLOBAL_HOOK, FakeRunner, nothing installed or injected; mutation M20 makes it red. Manual M6. | PASS |
| S7 | Longest-callback counter, reported by the log writer | `the_keyboard_callback_records_how_long_it_ran` (keyboard_hook_callback.rs:691), `the_longest_callback_is_kept_until_taken` (counters.rs:249), `a_slow_callback_alone_is_reported_and_a_fast_one_is_not` (:265), `logging::writer::tests::a_flush_reports_a_slow_callback` (writer.rs:359) | PASS |
| S8 | Main starts the executor before the hooks and stops it after they are removed; exit joins before the final flush | No harness (KS-38). Read in code: app.rs:197-206 (start → `update_bindings` → attach → `install`), :266-267 (`uninstall` → `stop`); `AttachedExecutor` Drop covers every `?`; `logging::flush` stays in main after `run`. Bounded 2 s join, then detach with `[WARN]`: declared deviation 2. Manual M4. | PASS (review + manual) |
| S9 | Injectable runner; no test runs a real action, `SendInput`, `ShellExecuteW` or a program | `FakeRunner` (executor/testing.rs) everywhere; `Win32Runner` only in the ignored M9 test; inline only `DoNothing` | PASS |
| S10 | Phase-1 leftovers as separate commits | L3 `mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing` (loader.rs:1258, red `fbd6b59`), L5 `an_empty_config_warning_is_returned_not_logged` (:1611, red `47f2b88`), L8 `a_second_init_starts_no_writer` (logging/mod.rs:226, red `7827ec8` by assertion on the L7 seam); L1, L2, L10, L11 are doc or comment only (diffs read; L2 RON validated with cfgcheck, impl.md) | PASS |
| S11 | README, `actions.md`, CHANGELOG updated | Done for every behaviour except the Delay pacing change (P1) | FAIL (P1) |

Rulings: R1 (resumable runs; the four tests at mod.rs:794, :821, :840 plus runs.rs:417, :449, and mod.rs:879, :901), R2 (`ShowWindowAsync`, sequence.rs:249-279; `a_window_step_never_waits_for_its_window` sequence.rs:336, red `fd5993e`; M11), R3 (`Submitted::Unavailable`, `executor_unavailable` with its Display arm, death drain; mod.rs:1203, :1242, :1288), Q1-Q7, M1, M2, M5, OQ1-OQ3, V2-I1 (`a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` mod.rs:1035, red `a364b77`), V2-M1 (actions.md §14.2), V2-M2 (M11 with a master control, impl.md), V2-M3 (`SetCoalescableTimer(.., TIMERV_NO_COALESCING)` mod.rs:474, `plan()` :436, `the_delay_timer_is_kept_only_while_nothing_ran` :963, M13), N1/N2/N4, loop-panic ownership (mod.rs:547-575): all carried out. The only exception is the factual premise of R1 itself (P1).

## Issues claimed closed
| ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-08 (partly; the watchdog is 2.4) | `the_hook_path_names_no_execute_call` (the audit's inline `.execute()`), `carry_out_returns_in_under_5_ms_while_slow_actions_block` (a 500 ms "cold browser" leaves the callback < 5 ms), `carry_out_queues_the_slow_actions_in_order_and_runs_none_here`, `a_hold_runs_its_slow_action_on_the_executor`, the STA test (COM uninitialised in the audit) | Yes: X5 red by assertion at `fcce19b`; X7 compile-red at `6edfa0f` (old `carry_out(decision)` signature), replayed by the tester; commit order puts every red before its fix | Closed for the slow actions; KS-08 stays open for 2.4 (watchdog), as declared |
| gaps "No cancellation for macro workers" | `delay_10000_stops_within_100_ms_of_an_epoch_change`, `swch_off_…`, `a_reload_…`, `a_cancelled_waiting_run_never_resumes` (both levels), `a_job_queued_before_an_epoch_change_does_not_run` | Yes: X1 compile-red at `cf3c9f6`; X6 red by assertion at `ba3122f` on the sleeping `Delay` | Closed (the per-binding run limit the gaps entry also wants is not done: N1, accepted) |
| KS-33 (Sequence half, unverified) | `a_hundred_sequence_runs_use_the_one_executor_thread`; `spawn_sequence` and its panicking `thread::spawn` removed | compile-red X1 | Closed for Sequence |
| known-issues rows 28, 29, 36, 60 (fixed), 26 (partly) | as KS-08 / cancellation above | as above | Closed in the table |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, outcome, no blocking or allocation, symmetry) | OK | `code < 0` first (keyboard_hook_callback.rs:345); remapping-off pass next; the sentinel check comes before any state (`decide_event` :186). One outcome on every path (:368-384). New per-event work: 2 x `Instant::now` + one `fetch_max` (:363, :377). Per slow fire: `as_fast` match, `GetForegroundWindow` for a Sequence only (same lookup `spawn_sequence` did), one `ArcSwapOption::load`, one atomic load, `try_send` into a preallocated `sync_channel` slot, `PostThreadMessageW` (does not wait). No `log::`, lock, new allocation or panic path. `suppress` comes from the engine, unchanged. |
| Engine purity, matching per owner decisions | OK | `src/engine` changed in docs only (12 lines in mod.rs). `repeats_by_default`, matching and `allow_extra_modifiers` are untouched. |
| Threads, sentinel, atomics, SAFETY | OK | Hooks still installed and removed on main; the executor's COM, queue, timer and pump stay on its own thread. No new `SendInput`; all 6 sites unchanged. Orderings: epoch `fetch_add AcqRel`/`load Acquire`, `stopping` and `active_runs` Release/Acquire, gauge `fetch_max`/`swap AcqRel`. No new `Relaxed`, `SeqCst` or `static mut` (the 2 `Relaxed` hits are pre-existing test helpers). Every new `unsafe` block has a concrete `// SAFETY:` (scripted check: none missing). `Apartment` is `!Send`. |
| Config reload, IPC | OK | The cancel follows the single sorted store in `update_bindings` (keyboard_hook.rs:120-127). A failed or deleted reload never calls it (config_watcher.rs:162-167), so live bindings and running macros are kept. The pipe is untouched. |
| No key logging | OK | New log lines carry counts, a `WindowOp`, an HRESULT or a Win32 error only. |
| Owner decisions | OK | RON, the `log` crate with its sink, exact modifiers, repeat defaults and the licence are unchanged. |

## Review status
- review_r2: APPROVED (Critical 0, Important 0, Minor 2). review_r1: APPROVED (Minor 1).
- triage_r1: F1 CONFIRMED (the COM tests read MAINSTA when run alone), resolved by `a875960`. In my own runs both tests
  pass alone and with `--test-threads=1`. F2 (M18, M21, M22, M26) REFUTED, because tester tests closed each one.
- test_r2: GREEN @ `6c83406` (one more gap, ME, closed by `6c83406`).
- Open Minor/NOTED items worth doing while the context is fresh (none blocks): review m1, the undocumented
  `pub fn thread` (`executor/mod.rs:317`), owned by the developer; review m2, the fixed 200 ms margin in the two
  executor-death tests (mod.rs:1254, :1299, false red only), owned by the tester.

## Docs and hygiene
- In sync: README (tree, hook, callback, app and new `executor/` sections, Hold, the macro table, parallel macros, `swch off`, counters, `Win32_System_Com`), `default_config.ron` (Sequence, Window and Delay comments; added text is ASCII), the config pages that ruling Q2 names, known-issues rows 26/28/29/36/60, and errors.md §18.6. The CHANGELOG has Added, Changed and Fixed entries citing roadmap 2.1 and KS IDs, plus the KS-01 and KS-33 edits and the L1/L2/L4 edits.
- Not in sync: the macro `Delay` pacing (P1).
- Branch hygiene: cut from the master tip, so `--no-ff` merges cleanly. Only this lane's work and its specified leftovers. Every red commit precedes its fix. `d269caf` is a non-test commit on X3's red tree (declared in the addendum; a nit).
- audit.md, roadmap.md, gaps.md and the other non-config pages are deferred to the Phase-2 book sync by ruling Q2, and impl.md lists them (see the hand-offs below).

## Technical debt
- N1: active runs are unbounded (runs.rs `Runs.active`); accepted, revisit with 3.5. Recorded in known-issues row 36 ("not capped").
- R-7: the residual hold-timer wake to a reused thread id at exit; accepted until 2.2. Recorded only in the lane files: the book sync should add it to roadmap 2.2's notes.
- `SystemFunction::{LockWorkstation, ShowDesktop, TaskManager}` have no `BindAction` and keep `log::` calls (impl.md). New feature 1 must place them; the exhaustive `as_fast` and `Win32Runner::action` force that. Recorded in impl.md only.
- `UOI_TIMERPROC_EXCEPTION_SUPPRESSION` is not set (cut Risks follow-up note); not recorded in the repo.
- Blocking steps (`Launch`, `OpenUrl`, a `ShellExecuteW` dialog) serialise the executor: declared in the CHANGELOG, README and actions.md.
- M12 (does `Window(Restore)` bring a minimised window back to the front under `ShowWindowAsync`?) is still unrun. If it does not, the CHANGELOG needs a sentence (impl.md open point).
- No new TODO, FIXME or HACK in the diff.

## Problems

### P1. Macro `Delay` lost its millisecond precision; the CHANGELOG is silent and the config reference claims parity
- **What**: a `Delay` now resumes on one win32k thread timer (`SetCoalescableTimer(None, 0, ms, None, TIMERV_NO_COALESCING)`, `src/core/executor/mod.rs:474`, pumped by `GetMessageW`). Microsoft documents that `uElapse` below `USER_TIMER_MINIMUM` (10 ms) is raised to 10 ms, and the timer is delivered at the system clock tick. Before the lane, `Delay` was `thread::sleep`. On this toolchain (rustc 1.98.1) that is a high-resolution waitable timer: `library/std/src/sys/thread/windows.rs:117-131` calls `high_precision_sleep` → `CreateWaitableTimerExW(.., CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, ..)` (`sys/pal/windows/time.rs:81-90`). I measured both on this machine with a scratch probe that ran on its own thread only (no hook, no input), 20 runs each:

  | `Delay` | thread timer: avg / max | `thread::sleep` (high-res): avg / max |
  |---|---|---|
  | 1 ms | 18.1 / 30.0 ms | 1.4 / 1.6 ms |
  | 5 ms | 18.1 / 28.6 ms | 5.3 / 5.5 ms |
  | 20 ms | 26.3 / 28.6 ms | 20.2 / 20.5 ms |
  | 50 ms | 61.4 / 74.5 ms | 50.2 / 50.5 ms |

- **Impact**: a `Delay` never ends early, and nothing gets stuck or eaten. But macros that pace keys with short Delays now run up to about 3.5x slower (100 x `Delay(5)`: about 1.8 s instead of about 0.5 s). Every `Delay` can end up to about one tick (about 15.6 ms) late, where it was within about 1 ms. This is a user-visible change in what a `Delay(ms)` value does, and it is undeclared:
  - The CHANGELOG `[Unreleased]` has no entry for it.
  - `book/src/config/actions.md:318` says the timer wakes "usually within about one tick of the system clock (about 15.6 ms) after the deadline, as with the `thread::sleep` it replaces". The comparison is false, and the 10 ms floor is not mentioned.
  - Manual check M13 (20 x `[Key, Delay(50)]` in 1.0-1.4 s) passes (about 1.2 s), so it cannot reveal the regression.
- **Root cause**: ruling R1 (rulings.md:22, "about 15.6 ms, the same as the old `thread::sleep`") and cut D6 ("the same as the old `thread::sleep`") rest on a false premise. Since Rust 1.75, std's Windows `sleep` uses a high-resolution waitable timer. The critics did not check it, and the developer copied the claim into the book.
- **Return to**: `doc-writer`. With this lane's scope and the rulings as they stand, the mechanism is the one ruled (R1, V2-M3), so the defect to fix in this lane is the undeclared change and the false text.
  - *Orchestrator decision first*: is ~16 ms Delay granularity acceptable? I recommend accepting it. Documented examples use `Delay(100)` and up, and roadmap open question 7 already accepts this granularity for gesture terms of 100 ms and up. Extend open question 7 to macro `Delay`.
  - If the owner wants millisecond pacing, that is a new architect item, not part of this rework. One option: an idle wait on a high-resolution waitable timer through `MsgWaitForMultipleObjectsEx(QS_ALLINPUT)` in place of the thread timer, which still pumps on the one thread. It needs a change to R1/V2-M3.
- **Re-check condition**:
  1. `CHANGELOG.md` `[Unreleased]` → **Changed** has a bullet citing roadmap item 2.1. It says that a macro `Delay` now waits on the action executor's thread timer: a `Delay` under 10 ms waits at least 10 ms, and any `Delay` can end up to about one clock tick (about 15.6 ms) late, where it used to be accurate to about 1 ms. Macros with many short Delays run slower, and a `Delay` still never ends early.
  2. `book/src/config/actions.md` §14.3 (`:318`) drops "as with the `thread::sleep` it replaces" and states the 10 ms floor and the up-to-one-tick lateness.
  3. `README.md:536` (the `Delay(<ms>)` row) and `src/config/default_config.ron:151` do not contradict this. Mentioning it there is optional; any added `.ron` text stays ASCII.
  4. No source change.
  5. `cargo fmt --check` and `mdbook build book -d D:/tmp/...` stay green.

  Optional in the same pass: where the docs say "a reload (saving `config.ron`) stops a running macro" (README, actions.md §13.1 and §14.1, the CHANGELOG "No cancellation" bullet), say a reload that loads. A save that does not load keeps the old bindings and the running macros (config_watcher.rs:162-167).

## Positives
- The design makes the dangerous cases impossible to write by construction. `FastAction` with an exhaustive `as_fast` cannot run a slow action inline. The `Decision` destructuring without `..` and the exhaustive `Win32Runner::action` force every new action to pick a placement.
- `runs.rs` is pure with a fake clock, so iteration, deadlines, epochs and interleaving are real code under test. `plan()` separates the timer decision from Win32, and the `code_ran` rule keeps a modal loop from eating a `WM_TIMER` unnoticed.
- The red-first discipline is unusually strict: every red was replayed by the tester, 34 mutations were run (4 + 1 gaps closed by new tests), and the "drain before every wait" structure has a deterministic red/green test.
- The death epilogue owns the queue, the timer and the runs outside the unwinding closure, so even a loop panic is counted (R3).
- The hook path stays lean and log-free, and one commented scan list pins that.

## Hand-offs

### Phase-2 book sync (doc-writer)
- `roadmap.md:421` (the "OpenUrl can freeze input" known issue: fixed on merge), `:449-455` (2.1 status after Phase 2), `:727` (KS-08 row). Add a Checkpoint 2 manual-check table with M1-M13 from cut.md and impl.md (M1 cold browser x20, M2-M4 and M10 cancel paths, M5 side-by-side macros, M6 Hold, M7 fast actions in order, M8 OpenUrl targets, M9 the ignored `the_win32_runner_opens_a_url`, M11 hung window with its master control, M12 Restore to front, M13 Delay pacing). Extend open question 7 (`:786`) to macro `Delay` (P1). Add R-7 to item 2.2's notes.
- `audit.md:312` KS-08 status: "Partly fixed in <merge sha> (feat/action-executor); watchdog: item 2.4". KS-33 status: the Sequence half fixed.
- `gaps.md:35`, `:145-147` ("No cancellation for macro workers": fixed, without the per-binding run limit), `:166` (the Sequence thread panic is gone).
- `architecture.md:188` (thread table: `action-executor` instead of macro workers), `:211`, `:213-239` (KS-08 section), `:434-436`, `:445` (`spawn_sequence`), `:577`, `:588`.
- `glossary.md:107`, `:317`; `overview.md:20-21`, `:68`; `build-and-run.md:118-141` (the new executor lines, the removed post errors, the counters line), `:251`, `:270`, `:461`.

### Orchestrator (CLAUDE.md, agent files, rulings)
- Correct the false premise in rulings.md R1 ("the same as the old `thread::sleep`") and decide P1's fork (accept and document, or a new architect item).
- `CLAUDE.md:19` layout: add `executor/` (`mod.rs`, `runs.rs`, `testing.rs`) under `src/core/`.
- `CLAUDE.md:83`: replace "Macro steps run on worker threads" with: `Launch`, `OpenUrl` and `Sequence` run on the `action-executor` thread, which owns its STA, its queue, its Delay timer and its message pump.
- `CLAUDE.md` principle 1: the hook path runs only `FastAction`s and `KeyboardHook::submit`. Principle 5: the `ArcSwapOption<ExecutorHandle>` in `KeyboardHook`, stored only by main.
- Agent files that still describe macro workers or actions on the hook thread:
  - `.claude/agents/architect.md:41`
  - `.claude/agents/architecture-critic.md:45`, `:137`
  - `.claude/agents/project-analyst.md:44`
  - `.claude/agents/researcher.md:34`
  - `.claude/agents/developer.md:28`
- Optional, while the context is fresh: review m1 (a `///` line on `Executor::thread`, developer), review m2 (the 200 ms margin in the two death tests, tester). The lines at `engine/mod.rs:89` (123 characters) and `keyboard_hook_callback.rs:25` are too long (cosmetic).
- Note: if `hook.uninstall()?` fails in `App::run` (app.rs:266), `AttachedExecutor`'s Drop stops the executor while a hook may still be installed. The existing `[FATAL]` path already blocks main on Enter in that state, so nothing new; record only.

## Notes for later lanes
- 2.2: the Hold timers move to main; R-7 then closes. `KeyboardHook::cancel_actions` is the cancel 2.3's state reset should call.
- 2.4: it can read `longest_callback_us`, `executor_unavailable` and `active_runs`.
- 3.3: de-elevation belongs in `Win32Runner::action` and `run_step`.
- 3.5: `Binding.action: Arc` and the bound on active runs (N1).
- New feature 1: the exhaustive `as_fast` / `Win32Runner::action` force a placement for every new action. The system functions that spawn a process or call the shell must map to `None`.
