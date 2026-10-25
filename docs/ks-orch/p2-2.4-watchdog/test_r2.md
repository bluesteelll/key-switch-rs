# Tester round 2: p2-2.4-watchdog / roadmap 2.4 hook watchdog (HEAD f51dc7d vs previous round head 140b831)

Verdict: RED: bench row R16 (first key of a burst) breaches the cut's own sanity bound on all three runs (p50 median 52.7 us against 29.7 us). Everything else is green. The developer raised the same point as open point 1 of fix_r1.md; it needs an orchestrator ruling (restate R16 as a declared evidence row, or a new delta).

HEAD: `f51dc7d` on `feat/hook-watchdog`. The developer's head was `959fce1`; I added three `Tests:` commits (test code only, listed below). `git status --short` is empty and `git worktree list` shows only the lane tree. `git diff --stat 959fce1..HEAD` is two files, +70 lines, all inside `mod tests`.

## Gate (HEAD f51dc7d)
- `cargo fmt --check`: OK
- `cargo clippy --all-targets --locked -- -D warnings`: OK (sources touched first; `Checking key-switch-rs`, `Finished dev`)
- `cargo test --locked --no-fail-fast` (log `D:/tmp/ks-orch/p2-2.4-watchdog/gate_test_r2_final.log`):
  - lib: `running 804 tests` / `test result: ok. 797 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 3.84s`
  - `swch` bin: `running 0 tests` (no tests in that target, not a filter)
  - `tests/ipc_roundtrip.rs`: `running 7 tests` / `test result: ok. 7 passed; 0 failed`
  - doc-tests: `running 0 tests` (none)
  - Ignored (7, counted apart, none run): `bench::latency_bench` (bench), `core::executor::...the_win32_runner_opens_a_url`, `hook::ipc_server::...hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::...the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::...input_is_on_this_desktop_while_a_human_uses_it`, `platform::foreground::...the_foreground_cache_names_the_focused_notepad`, `system::registry::...live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (all `manual: ...` except the bench). No `--ignored` run.
  - Same run on `959fce1` first: 794 passed, 7 ignored (the three tester tests account for 797).
- `cargo build --release --locked`: OK (never run).
- Timing: `cargo test --locked --lib hook::watchdog` run 5 times on `959fce1`: `running 70 tests`, `70 passed; 0 failed` each time (3.01-3.26 s). Run again on HEAD inside the full suite and in each commit check: 73 passed.

## Red-first replay (CONFIRMED items of triage_r1 and the round-1 red commits)
Scratch worktree `D:/tmp/ks-orch/p2-2.4-watchdog/wt/red`, `CARGO_TARGET_DIR=.../target-wt/parent`, removed afterwards.

| Item / test | Red commit (parent of fix) | Result on the red commit | Result on HEAD |
|---|---|---|---|
| R-I1 `hook::installer::tests::an_installer_call_in_a_tests_module_is_found` | `ee9ef4d` | `running 5 tests`; FAILED `assertion left == right failed: the call in the tests module: [] left: 0 right: 1` | ok |
| R-I1 `every_spelling_of_the_hook_installer_is_found` | `ee9ef4d` | FAILED `SetWindowsHookExA is not found: [] left: 0 right: 1` | ok |
| R-I1 `the_build_script_is_scanned` | `ee9ef4d` | FAILED `build.rs is not among the scanned sources` | ok |
| (installer total) | `ee9ef4d` | `test result: FAILED. 2 passed; 3 failed` | `running 5 tests` / `ok. 5 passed; 0 failed` |
| T-F1 = R-I2 decision: `a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it`, `a_hook_that_dies_during_a_burst_asks_for_exactly_one_reinstall` | `ea8f19d` | `running 21 tests`; both FAILED by assertion (`left: [1250] right: []`; `left: 0 right: 1`); `19 passed; 2 failed` | `running 22 tests` / `ok. 22 passed` (23 with my row) |
| T-F1 loop: `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events` (F8a) | `384ddf5` | `running 1 test`; FAILED `a ticking watchdog woke 198 times for 195 raw key events (at most 12)` | ok (5 runs of `hook::watchdog`) |
| F2 stop line (`LoopStats`, `stats`, `stop_counting`) | `4c711ed` | compile-red: E0422 `LoopStats`; E0599 `stats`; E0599 `stop_counting` | ok; carried by mutation M3 below (stop thread) and the stats rows |
| F4 wake mask / pump ranges | `f58c95c` | compile-red: E0425 `wake_mask`, `pump_ranges`; E0599 `batched_wake`, `batched_message` | ok; carried by M3 and M4 |
| F6 byte walk | `e712a2b` | compile-red: E0425 `count_raw_keys`, `RawBuffer`, `RAW_BUFFER_BYTES`; E0422 `BlockCount`; `offset_of` | ok; carried by M6 |
| F8c drain | `c605d0f` | compile-red: E0405 `RawReads`; E0425 `drain`, `next_wait`, `drain_keyboard`; E0422 `Drained`; E0433/E0425 `Read` | ok; carried by M1, M2, M7 |
| F8b batched Drive rows | `d668a13` | compile-red: E0599 `raw_batch`, `is_ticking`; E0061 `tick` arity | ok; carried by M5, M8, M9 |
| `65c4d84` F1, `a90b38b` OQ2 | declared green characterisations of Windows, no red expected | not replayed (they test Windows behaviour, not a defect) | in the 797 passed |

Worktree removed; `git worktree list` clean.

## Mutations (scratch worktree `wt/mut`, `CARGO_TARGET_DIR=.../target-wt/mut`; each reverted, `git diff --exit-code` clean; worktree removed)
All run with `cargo test --locked --lib hook::watchdog` (or `hook::installer`), `running 70-73 tests`.

| # | File | Mutation | Predicted catch | Result |
|---|---|---|---|---|
| M1 | decision.rs `decide` | `now >= quiet_since + window` to `>` | a request-timing row | caught by 5 tests: `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops`, `raw_events_before_an_answer_are_no_evidence_after_it`, `a_batch_read_at_a_tick_counts_all_its_events`, `raw_events_read_with_an_answer_are_no_evidence_after_it`, `batched_reinstalls_without_callbacks_are_spaced_out` |
| M2 | mod.rs `drain` budget | `drained.pending = true` to `false` | `a_drain_that_spends_its_budget_leaves_the_rest_for_the_next` | caught (only that test) |
| M3 | mod.rs `next_wait` | `wake_mask(batched, ticking \|\| drained.stuck)` to `wake_mask(batched, ticking)` | `raw_input_that_no_read_returns_is_counted_and_the_wait_backs_off_for_a_tick` | caught (only that test) |
| M4 | mod.rs `count_raw_keys` | `qualifies(make_code, vkey)` replaced by `(make_code \| vkey) < u16::MAX` | `a_buffered_read_counts_the_key_events_the_hook_sees` | caught |
| M5 | mod.rs `drain` | `Read::Failed` no longer switches to standard reads | `a_failed_buffered_read_is_counted_and_the_standard_read_takes_the_rest` | caught |
| M6 | installer.rs `INSTALLERS` | prefix `SetWindowsHook` to `SetWindowsHookEx` | `every_spelling_of_the_hook_installer_is_found` | caught |
| M7 | keyboard_hook.rs (probe) | uncalled fn in a `#[cfg(test)] mod probe` naming `SetWindowsHookExW` | `the_hook_installers_are_compiled_out_of_the_tests` | caught: `src/hook/keyboard_hook.rs:841: ...` (end to end proof that the old blind spot is closed) |
| M8 | mod.rs `pump` | `WM_QUIT` ignored (`if false && ...`) | thread start/stop rows | caught by 6 tests (`the_watchdog_thread_starts_and_stops_without_raw_input`, `..._has_no_top_level_window`, `..._reports_its_wakes_...`, F8a, two synthetic request rows) |
| M9 | mod.rs `pass` | remove the `hook_installed()` gate on `raw_batch` | `raw_input_before_the_hook_is_installed_is_read_and_not_counted` | caught |
| **M10** | decision.rs `tick` | `kept = credit - explained` to `kept = credit` under a backlog | `a_burst_read_over_several_ticks_...` / `a_hook_that_dies_during_a_burst_...` | **NOT CAUGHT** (70 passed). Gap closed: see T1 |
| **M11** | mod.rs `wait_ms` | `div_ceil(1_000_000)` to `/ 1_000_000` | none expected by name | **NOT CAUGHT** (71 passed). Gap closed: see T2 |
| **M12** | mod.rs `pass` | tick on every pass: `is_some_and(\|due\| due <= now)` to `is_some()` | `an_early_timer_signal_does_not_lose_the_tick` | **NOT CAUGHT** (72 passed; that row accepts any request in 1000..=1500). Gap closed: see T3 |

Each of M10-M12 then red under the mutation with the new row and green without it:
- M10 with T1: `test hook::watchdog::decision::tests::credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them ... FAILED`, `one request for the 2 000 extra events: [] left: 0 right: 1`; the other 70 passed. Green on HEAD (`23 passed` in `hook::watchdog::decision`).
- M11 with T2: `a_wait_is_rounded_up_to_whole_milliseconds_and_stays_below_infinite ... FAILED` on `a fraction of a millisecond waits a whole one`; the other 71 passed. Green on HEAD.
- M12 with T3: `an_early_timer_signal_leaves_the_ticks_where_they_were ... FAILED`, `the timer 10 ms early left: [1200] right: [1000]`; the other 72 passed. Green on HEAD.

## Latency bench (section 4b)
Rows from `D:/tmp/ks-orch/p2-2.4-watchdog/bench-r2-{prev,head}-{1,2,3}.md` (logs alongside), release, each tree in its own worktree and target dir, `--test-threads=1`, machine otherwise idle, 3 runs each, `prev` = `140b831`, `head` = `959fce1` (production code identical to `f51dc7d`; my three commits are test-only). Script `bench_run_r2.sh`; all six exits 0. Master was not re-run (the brief names 140b831).

| Row | prev p50 (3 runs) | prev p99 largest | head p50 (3 runs) | head p99 (3 runs) | head p99 largest |
|---|---|---|---|---|---|
| R12 unbound A, no watchdog | 4.7 / 4.7 / 4.7 | 5.9 | 4.7 / 4.7 / 4.7 | 4.8 / 4.8 / 4.8 | 4.8 |
| R14 steady typing, watchdog ticking | 22.3 / 21.4 / 21.1 (median 21.4) | 52.9 | 4.8 / 4.8 / 4.8 (median 4.8) | 8.6 / 38.5 / 66.9 | 66.9 |
| R14c control, same pacing, no watchdog | - | - | 4.8 / 4.8 / 4.8 | 9.1 / 107.1 / 56.4 | 107.1 |
| R15 rejected design (evidence) | 3.9 / 3.8 / 3.9 | 7.8 | 3.8 / 3.9 / 3.8 | 4.8 / 5.4 / 4.7 | 5.4 |
| R16 first key of a burst | - | - | 54.6 / 52.7 / 46.1 (median 52.7) | 107.6 / 101.2 / 121.9 | 121.9 |

- L1 acceptance line (R14 steady typing): R14 p50 4.8 us against R14c 4.8 us (bound +0.3 us): **met in all 3 runs**. R14 against R12: 4.8 vs 4.7 us. The defect T-F1 (R14 p50 21-22 us at 140b831, 4.7x R12) is gone.
- R14 p99 against R14c p99 (bound +2 us): per run +(-0.5), -68.6, +10.5 us. Largest against largest: 66.9 vs 107.1, within the bound. Run 3 alone is +10.5 us; the control row with no watchdog swings 9.1 to 107.1 us between runs, so the p99 of these 1 ms paced rows is scheduler noise, not watchdog work (the pre-L1 R14 p99 was 39.6-52.9 us). Reported, not counted as a breach, per the tester.md rule (largest p99 per row). The orchestrator may want to restate the p99 half of the bound for the paced rows.
- **R16 sanity bound (cut.md:984: p50 <= R12 p50 + 25 us = 29.7 us; p99 <= R12 p99 + 100 us = 104.8 us): breached.** p50 median 52.7 us on all runs (46.1-54.6, every run above 29.7); p99 107.6 / 101.2 / 121.9 (two of three above 104.8). This reproduces the developer's single run (53.5 / 168.4). The cut declared the one wake per burst as justified extra work, but only up to this bound. R16 is a pessimistic row (it times the callback at once after the post, the worst order of cut Q6); whether a real key pays it depends on whether raw input reaches the thread before the hook chain, which no agent can measure. Needs an orchestrator ruling.
- Row R8 (`KS_BENCH_LOAD`) not run: the fix touches the watchdog thread, not the main loop, thread priority or the executor. The hook callback files are unchanged since 140b831 (R12 identical on both trees).

## Acceptance (ruling L1 and V3, from the pack)
| Criterion | Evidence | Status |
|---|---|---|
| L1: R14 within D11's bound in steady typing | bench above: p50 +0.0 us over R14c | verified (p50); p99 noisy, see above |
| L1: first key of a burst has its own declared row | R16 present; bound breached | RED (ruling needed) |
| L1: per-key wake gone while ticking | `while_ticking_raw_key_events_wake_nothing` ok; `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events` ok (red on 384ddf5) | verified |
| L1: first key after idle wakes | `the_first_raw_key_event_after_idle_wakes_the_watchdog` ok | verified |
| I1/M1/M5/rule 4 on batches | `an_early_timer_signal_...` (+T3), `a_timer_that_refuses_once_...`, `a_single_batched_raw_event_stops_ticking_after_two_seconds`, `a_batch_is_evidence_only_from_the_next_tick`, `raw_events_read_with_an_answer_are_no_evidence_after_it` ok; M1/M12 mutations catch | verified |
| V3-I1 critic trace: no reinstall | `decision::a_burst_read_over_several_ticks_...` (red on ea8f19d), `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` ok | verified |
| V3-I1 dead hook during/after a burst: exactly one | `decision::a_hook_that_dies_during_a_burst_...` (red on ea8f19d), `..._after_a_burst_...`, loop `a_hook_that_dies_during_a_flood_is_reinstalled_once`, T1 | verified (T1 added: the credit-carry case was unpinned) |
| V3-I1 long drift, live hook: none | `long_drift_with_a_live_hook_asks_for_nothing`, `a_hook_that_dies_after_long_drift_is_found_within_its_window` ok | verified |
| V3-I1 drain until empty, time budget | `a_drain_reads_until_the_queue_is_empty`, `a_drain_that_spends_its_budget_...` (M2), `the_next_wait_never_waits_while_raw_input_is_left` ok | verified |
| V3-M3 stuck guard | `raw_input_that_no_buffered_read_returns_...`, `raw_input_that_no_read_returns_...` (M3) ok | verified |
| OQ2 pump has no window filter | `a_range_filtered_peek_leaves_a_posted_wm_quit_for_the_pump` ok; M8 | verified |
| OQ3 FFI smoke | `the_buffered_read_of_a_thread_without_raw_input_reads_nothing` ok | verified |
| V3-I2 R14c control | bench row exists | verified |
| V3-M2 R16 sets `hook_installed`, n = 1000 | row `n = 1000` in the table | verified |
| R-I1 S4 scan | 3 new scan tests + main scan (red on ee9ef4d, ok); M6, M7 | verified |
| No injection / no key data | `the_watchdog_injects_no_input`, `no_key_data_leaves_the_raw_input_read` ok; grep of the added lines `140b831..HEAD` for `key-switch-rs.sock`, `config.ron`, `SendInput(`, `RegSetValue`, `RegisterRawInputDevices(` finds nothing | verified |
| M9 batching on the live daemon, M10 leak check, M2 amended, M1 re-run, OQ4 | no `#[ignore = "manual: ..."]` test; owner checklist in fix_r1.md | manual, not verified |

## Failures
### F1. Bench row R16 above its sanity bound
- Row: `bench::latency_bench` R16 (`src/bench.rs:583-606`), `D:/tmp/ks-orch/p2-2.4-watchdog/bench-r2-head-{1,2,3}.md`.
- Expected (cut.md:984): p50 <= 29.7 us, p99 <= 104.8 us. Received: p50 54.6 / 52.7 / 46.1, p99 107.6 / 101.2 / 121.9.
- Suspected cause (developer diagnostics, `bench-dev-diag.md`): the cost of a wake from idle on the watchdog thread (user32 calls on the first pass), not the decision work (53.6 us with no `raw_batch` and no timer set) and not the thread priority. Present before L1 (R16's construction on `0e7165b`: 35.6 us). This is the bench design's bound, not a test or code defect I can fix; it is an orchestrator decision.
- Options: restate R16 as a declared evidence row with the measured numbers (the p50 about 48 us above R12 at the first key of a burst, once per burst), or open a delta to shorten the idle wake.

## Tests added by the tester (`Tests:` commits on `feat/hook-watchdog`, test code only, trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`)
- `7300822` T1: `src/hook/watchdog/decision.rs`, `credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them` (kills M10).
- `995892f` T2: `src/hook/watchdog/mod.rs`, `a_wait_is_rounded_up_to_whole_milliseconds_and_stays_below_infinite` (kills M11).
- `f51dc7d` T3: `src/hook/watchdog/mod.rs`, `an_early_timer_signal_leaves_the_ticks_where_they_were` (kills M12).
- Each commit was fmt-checked, clippy-clean and run in `hook::watchdog` before it was made; the full suite ran once afterwards (797 passed).

## Notes
- Scratch artefacts outside the repo: `D:/tmp/ks-orch/p2-2.4-watchdog/target-wt/{parent,mut,bench-head,bench-prev}`, bench logs and `tester_r2_test*.patch`. All worktrees removed.
- The pack's HEAD (`959fce1`) is now two `Tests:` commits behind `f51dc7d`; the reviewer should diff `959fce1..f51dc7d`.
- Open points not tested by me: R-m2..R-m4 and the T-note stay NOTED; the S4 scan limits (char literals, multi-line strings, `tests/` not recursive) are unchanged and recorded in fix_r1.md open point 4.
- M10's trace: the existing burst rows never held a backlog together with a partly used credit and a dead hook, so doubling the credit was invisible. T1 does exactly that (12 000 raw events, 10 001 callbacks, three ticks).
