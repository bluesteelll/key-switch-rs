CONTEXT PACK READY

# Context pack, round 2: p2-2.4-watchdog (fix round 1 = ruling L1 + round-3 rulings)

Readers: tester and code-reviewer of round 2. Facts only; `UNVERIFIED` marks anything not read in code.
Paths are relative to `D:/key-switch-rs/key-switch-rs` unless absolute. Lane files: `D:/tmp/ks-orch/p2-2.4-watchdog/`.

## Scope and delta

- Branch `feat/hook-watchdog`, HEAD `959fce1` (read on 2026-10-19; work tree clean, `git worktree list` shows only the main tree).
- `master` = `fa20b30` = merge-base of the branch. Round-1 HEAD (base of this round) = `140b831`; `fa20b30..140b831` is 32 commits (round 1, not re-scoped here).
- Delta of this round, `git log --oneline 140b831..feat/hook-watchdog` (20 commits, oldest last):

```
959fce1 Tests: the SAFETY comments of the queue characterisations state their invariants
a90b38b Tests: a range-filtered peek leaves a posted WM_QUIT for the pump
e75def6 Docs: the README and the watchdog's docs give the measured cost of the first key of a burst
e69ce7f Bench: R14 times steady typing with the watchdog ticking, R16 the first key of a burst
a0aaf7c Hook: the hook watchdog reads raw input in batches at its ticks
d668a13 Tests: the watchdog reads raw input in batches at its ticks (red)
384ddf5 Tests: a ticking watchdog thread is woken by every synthetic raw event (red)
0ac825a Hook: the drain of the watchdog's raw input
c605d0f Tests: a drain reads raw input until the queue is empty, within a time budget (red)
30ac79d Hook: the watchdog credits callbacks against raw events by accumulated counts
ea8f19d Tests: a burst whose raw input is read late is matched by the callbacks already counted (red)
76551be Hook: count the raw key events of a buffered read
e712a2b Tests: raw key events in a buffered read are counted in one pass (red)
f222c5c Hook: the wait mask and the pump ranges of a batched read
f58c95c Tests: the watchdog's wait and pump leave the batched raw input in the queue (red)
0478a2c Hook: the watchdog counts its wakes and logs them when it stops
4c711ed Tests: the watchdog reports its wakes and raw key events when it stops (red)
65c4d84 Tests: a message wait leaves out a pending message its wake mask does not name
0e7165b Tests: the installer scan reads whole files, every SetWindowsHook spelling and build.rs
ee9ef4d Tests: the installer scan misses test modules, other spellings and build.rs (red)
```

- `git diff --stat 140b831..feat/hook-watchdog`: 8 files, +2188 / -253.

| File | +/- |
|---|---|
| `src/hook/watchdog/mod.rs` | 1739 (most of it tests) |
| `src/hook/watchdog/decision.rs` | 283 (+261 / -22) |
| `src/bench.rs` | 199 |
| `src/hook/installer.rs` | 176 (test module only) |
| `CHANGELOG.md` | 26 |
| `src/logging/counters.rs` | 11 (doc comments only) |
| `README.md` | 4 |
| `book/src/config/errors.md` | 3 |

- Not in the delta (unchanged since `140b831`): `keyboard_hook_callback.rs`, `keyboard_hook.rs`, `main_thread.rs`, `health.rs`, `core/app.rs`, `ipc.rs`, `bin/swch.rs`, `tests/ipc_roundtrip.rs`, `logging/panic.rs`, `debug_hold.rs`, `platform/*`.
- Governing docs: `rulings.md` rounds 2 (L1, `:70-96`) and 3 (V3-*, `:98-139`); `cut.md` "Delta after ruling L1" `:783-1131`; Developer addendum `cut.md:704-781`; `fix_r1.md` (developer report); `triage_r1.md`.

## File map

| Range (HEAD) | What lives there | Why it matters this round |
|---|---|---|
| `src/hook/watchdog/mod.rs:1-52` | module docs: thread, `# Cost` (R16 ~50 us figure at `:33-38`), no key data | doc claims vs code; M6/V3-M2 text |
| `src/hook/watchdog/mod.rs:120-140` | `Plan`, `plan()` timer plan | ruling I1 |
| `src/hook/watchdog/mod.rs:142-157` | `LoopStats {wakes, raw_keys}`, `Context` | F2/F3 stop line |
| `src/hook/watchdog/mod.rs:159-344` | `Ticker`: `is_ticking` `:194`, `read_at_stop` `:199`, `fired` `:205`, `wake_by` `:213`, `pass` `:227-263`, `take_answer` `:265`, `ask` `:287`, `arm` `:318-343` | order raw_batch -> take_answer -> tick -> arm (rule 4, M5, I1) |
| `src/hook/watchdog/mod.rs:346-407` | `SYNTHETIC_RAW` (test), `Source::{batched_wake, batched_message, input_here, drain}` | keyboard vs synthetic mask/message |
| `src/hook/watchdog/mod.rs:409-527` | `HookWatchdog` spawn/start/stop, `done: Receiver<Option<LoopStats>>`, `stop_counting` (test) | T-note, stop bound |
| `src/hook/watchdog/mod.rs:529-605` | `SinkWindow`, `Watched`, `RawInputRegistration` (drop order) | R10 thread ownership |
| `src/hook/watchdog/mod.rs:607-678` | `run_watchdog`: prologue, `run_loop` call, stop line `:672-676` | the `[INFO]` stop line read by M2/M9 |
| `src/hook/watchdog/mod.rs:680-744` | `run_loop`: `next_wait` -> `MsgWaitForMultipleObjectsEx` -> `pump` -> drain -> `pass`; quit drain `:737-741` | core of L1 |
| `src/hook/watchdog/mod.rs:746-803` | `wake_mask`, `pump_ranges`, `wait_ms`, `pump` | E4, E7, OQ2 |
| `src/hook/watchdog/mod.rs:805-917` | `raw_key_event` (standard read), `RAW_BUFFER_BYTES`, `RAWINPUT_ALIGN`, `RawBuffer`, `BlockCount`, `u32_at`/`u16_at`, `count_raw_keys` | buffered-read byte walk (E11), no key data |
| `src/hook/watchdog/mod.rs:919-1067` | `BUFFERED`, `DRAIN_BUDGET` (5 ms), `Read`, `RawReads`, `Drained`, `drain`, `next_wait` | V3-I1 drain-until-empty, V3-M3 stuck guard |
| `src/hook/watchdog/mod.rs:1069-1195` | `KeyboardReads` (Win32 reads), `drain_keyboard`, `SyntheticReads` (test), `qualifies` | 3 new `unsafe` blocks + `GetQueueStatus` |
| `src/hook/watchdog/mod.rs:1197-1502` | tests: `Recorder`, `FakeTimer`, batched `Drive` model `:1323-1496` | the loop model all L1.7 rows run on |
| `src/hook/watchdog/mod.rs:1504-1910` | loop rows (I1, L1.7, flood rows, desktop rows) | V3-I1 acceptance |
| `src/hook/watchdog/mod.rs:1910-1964` | F4 tests: mask, pump ranges, RIDEV scan | E10 |
| `src/hook/watchdog/mod.rs:1965-2100` | F6 byte-walk tests | E11 |
| `src/hook/watchdog/mod.rs:2102-2357` | `FakeReads`, drain tests, `next_wait` test, OQ3 smoke test `:2319` | V3-I1, V3-M3, OQ3 |
| `src/hook/watchdog/mod.rs:2376-2653` | real-thread tests: start/stop, F1 `:2428`, synthetic request rows, stats, OQ2 `:2561`, F8a `:2618` | timing-sensitive tests on the real clock |
| `src/hook/watchdog/mod.rs:2655-2777` | scans: `the_watchdog_injects_no_input` `:2690`, `no_key_data_leaves_the_raw_input_read` `:2748` | acceptance "no injection", "no key data" |
| `src/hook/watchdog/decision.rs:1-56` | docs: credit/baseline rule (V3-I1), anti-loop rule | stated rule |
| `src/hook/watchdog/decision.rs:58-271` | `Watchdog`: `credit` `:101`, `raw_batch` `:138-149`, `raw_event` (test) `:153`, `tick(now, callbacks, backlog)` `:171-222`, `keeps_ticking` `:227`, `answered`/`not_sent`/`declined`/`start_over` `:241-270` | the pure decision |
| `src/hook/watchdog/decision.rs:273-927` | `Feeder` harness `:284-375`; V3-I1 rows `:698-853`; `a_batch_counts_as_that_many_raw_events` `:856` | red-first G1 |
| `src/hook/installer.rs:381-647` | S4 scan: `INSTALLERS` `:386-392`, `scanned_sources` `:414-419`, `code_of` `:424`, `cfg_not_test_block` `:452`, `scan_source` `:486-512`, 3 new tests `:527-577`, main scan `:580-647` | R-I1 fix; reviewer re-checks (rulings.md:96) |
| `src/bench.rs:53-80` | row docs R12-R16 | |
| `src/bench.rs:436-606` | `Unanswered`, `bind_f13_to_do_nothing`, `spin_for`, `paced_calls`, R14 `:498-515`, `ParkedQueue` `:520-564`, R14c `:570-574`, R16 `:583-606` | L1.6 / V3-I2 / V3-M2 rows |
| `src/bench.rs:1020-1022, 1099-1130` | ignored `latency_bench` and its run command; row list | bench command |
| `src/logging/counters.rs:48-57` | doc of `RawInputUnread`, `WatchdogTimerNotSet` | R-m1 |
| `README.md:905`, `:939` | Technologies line, watchdog bullet | M6 / V3-M2 cost text |
| `CHANGELOG.md:179-215` (approx.), `:250-257` | watchdog entry; bench entry | |
| `book/src/config/errors.md:215`, `:231` | stop-line row; counters row (`raw_input_unread` wording) | |

## Red-first tests and acceptance proofs (from `fix_r1.md:18-36`, checked against the commits)

Every red commit below touches only test code (hunks all inside `mod tests` / the test module), except `ea8f19d`, which also adds the production parameter `backlog` to `Watchdog::tick` as a stub (`let _ = backlog;`) and passes `false` at `mod.rs` (then `:236-238`). In every row the red is observed **on the red commit itself, which is the parent of the fix commit**; the parent of the red commit does not contain the test. `master` (`fa20b30`) has none of these seams (no `src/hook/watchdog/` changes of this round), so no row is red-checkable on `master`.

Reproducing a red needs a detached worktree outside the repo, for example `git worktree add --detach D:/tmp/ks-orch/p2-2.4-watchdog/wt/<sha> <sha>` with `CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.4-watchdog/target-wt/parent` (existing dir). Disk: `D:` 18 G free, 93 % used at the time of reading; `fix_r1.md:149` records `D:` hitting 0 bytes during round-1 diagnostics.

| # | Red commit (= parent of fix) | Fix commit (green) | Command | Expected red | Expected green |
|---|---|---|---|---|---|
| 1 | `ee9ef4d` (R-I1) | `0e7165b` | `cargo test --locked --lib hook::installer` | assertion: `an_installer_call_in_a_tests_module_is_found` "the call in the tests module: []"; `every_spelling_of_the_hook_installer_is_found` "SetWindowsHookExA is not found: []"; `the_build_script_is_scanned` "build.rs is not among the scanned sources"; 2 passed, 3 failed | 5 passed |
| 2 | — (`65c4d84`, F1) | — | `cargo test --locked --lib hook::watchdog::tests::a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name` | declared green characterisation of Windows (E4), no red | 1 passed |
| 3 | `4c711ed` (F2) | `0478a2c` (F3) | `cargo test --locked --lib hook::watchdog` | compile: `E0422 cannot find struct ... LoopStats`; `E0599 no method named stats found for struct Ticker<T>`; `E0599 no method named stop_counting found for struct HookWatchdog` | green |
| 4 | `f58c95c` (F4) | `f222c5c` (F5) | same | compile: `E0425 cannot find function wake_mask`, `... pump_ranges`; `E0599 no method named batched_wake / batched_message found for enum Source`. The RIDEV scan test is green on arrival | green |
| 5 | `e712a2b` (F6) | `76551be` (F7) | same | compile: `E0425 count_raw_keys`; `E0422 BlockCount`; `E0425 cannot find type RawBuffer`; `... value RAW_BUFFER_BYTES`; `cannot find macro offset_of` | green |
| 6 | `ea8f19d` (G1, V3-I1 decision) | `30ac79d` (G2) | `cargo test --locked --lib hook::watchdog::decision` | assertion: `a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it` `left: [1250] right: []`; `a_hook_that_dies_during_a_burst_asks_for_exactly_one_reinstall` `left: 0 right: 1`. Green on both commits: `a_hook_that_dies_after_a_burst_...`, `long_drift_with_a_live_hook_asks_for_nothing`, `a_hook_that_dies_after_long_drift_is_found_within_its_window` | green |
| 7 | `c605d0f` (F8c) | `0ac825a` (F8d) | `cargo test --locked --lib hook::watchdog` | compile: `E0405 cannot find trait RawReads`; `E0425 drain / next_wait / drain_keyboard`; `E0422 Drained`; `E0433 cannot find type Read` | green |
| 8 | `384ddf5` (F8a) | `a0aaf7c` (F9) | `cargo test --locked --lib hook::watchdog::tests::a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events` | assertion: "a ticking watchdog woke 194 times for 190 raw key events (at most 12)" (counts vary with timer resolution). Only observable on `384ddf5`: on `d668a13` the crate's tests do not compile | green at `a0aaf7c` |
| 9 | `d668a13` (F8b) | `a0aaf7c` (F9) | `cargo test --locked --lib hook::watchdog` | compile: `E0599 no method named raw_batch found for struct Watchdog`; `E0599 no method named is_ticking found for struct Ticker<T>`; `E0061 this method takes 2 arguments but 4 arguments were supplied` | `hook::watchdog`: 69 passed (fix_r1:28) |
| 10 | — (`e69ce7f`, F10 bench) | — | bench, below | no red (ignored bench) | — |
| 11 | — (`a90b38b`, OQ2) | — | `cargo test --locked --lib hook::watchdog::tests::a_range_filtered_peek_leaves_a_posted_wm_quit_for_the_pump` | declared green characterisation | 1 passed |

Developer's run at HEAD (fix_r1:82): `cargo test --locked --lib -- hook::watchdog hook::installer logging:: core::app hook::health` -> `test result: ok. 141 passed; 0 failed`. Full suite not run by the developer. Round-1 baseline for the full suite (`gate_test_final.log`): lib 752 passed, 7 ignored; `tests/ipc_roundtrip.rs` 7 passed. HEAD's full count: UNVERIFIED.

Gates the developer reports as passed at HEAD (fix_r1:81): `cargo fmt --check`; `cargo check --all-targets --locked`; `cargo clippy --all-targets --locked -- -D warnings`, also with `--release`. Book: `mdbook build book -d D:/tmp/ks-orch/p2-2.4-watchdog/book` built.

### Acceptance tests at HEAD, by ruling (test names; module `hook::watchdog::tests` unless `decision::`)

| Ruling / item | Tests (line) | Expected |
|---|---|---|
| V3-I1 critic trace (10 000 events, then silence: no reinstall) | `decision::a_burst_read_over_several_ticks_...` `decision.rs:699`; loop `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` `mod.rs:1774` | no request; last read at 749 ms |
| V3-I1 dead hook during/after burst: exactly one | `decision::a_hook_that_dies_during_a_burst_...` `:728` (request in 1250..=1500); `decision::a_hook_that_dies_after_a_burst_...` `:755`; loop `a_hook_that_dies_during_a_flood_is_reinstalled_once` `mod.rs:1787` | 1 request |
| V3-I1 long drift, live hook: none | `decision::long_drift_with_a_live_hook_asks_for_nothing` `:793` (3 variants, 600 s); `decision::a_hook_that_dies_after_long_drift_is_found_within_its_window` `:835` | 0; then 1 within last callback + 1000..=1500 ms |
| V3-I1 drain until empty, time budget | `a_drain_reads_until_the_queue_is_empty` `:2151`; `a_drain_that_spends_its_budget_leaves_the_rest_for_the_next` `:2171`; `the_next_wait_never_waits_while_raw_input_is_left` `:2287` | keys 9, 4 buffered calls; 7 600 over several drains; timeout 0 when pending |
| V3-M3 stuck queue guard | `raw_input_that_no_buffered_read_returns_is_taken_by_one_standard_read` `:2221`; `raw_input_that_no_read_returns_is_counted_and_the_wait_backs_off_for_a_tick` `:2243` | 1 standard read; `stuck`, mask without `QS_RAWINPUT`, timeout 250 |
| OQ3 FFI smoke | `the_buffered_read_of_a_thread_without_raw_input_reads_nothing` `:2319` | outcome 0 (unread 0) or `u32::MAX` (unread 1); `Drained::default()`; dev observed 0 |
| OQ2 | `a_range_filtered_peek_leaves_a_posted_wm_quit_for_the_pump` `:2561` | filtered peek `None`; pump ranges take `WM_QUIT`; `PostQuitMessage` quit passes any filter |
| L1 per-key wake gone | `while_ticking_raw_key_events_wake_nothing` `:1549` (`raw_wakes == 1`, passes <= 22, raw_keys 101); `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events` `:2618` (wakes <= 12) | |
| L1 first key after idle wakes | `the_first_raw_key_event_after_idle_wakes_the_watchdog` `:1568` | `raw_wakes == 2`, `Set(TICK)` at 5000 |
| M1 on batches | `a_single_batched_raw_event_stops_ticking_after_two_seconds` `:1665` | last pass <= 2250; `(due, timeout) == (None, None)` |
| M5 on batches | `a_batch_is_evidence_only_from_the_next_tick` `:1643`; `decision::evidence_is_checked_before_this_intervals_raw_events_are_added` `:602` | no request |
| Rule 4 | `raw_events_read_with_an_answer_are_no_evidence_after_it` `:1730` | requests == [1000] |
| I1 | `an_early_timer_signal_does_not_lose_the_tick` `:1505`; `a_timer_that_refuses_once_is_set_again_at_the_next_pass` `:1521` | |
| Anti-loop | `a_busy_main_thread_gives_one_request_with_batched_reads` `:1685`; `batched_reinstalls_without_callbacks_are_spaced_out` `:1708` (requests [1000, 3250, 7500, 15750, 32000]) | |
| Batch = n events | `decision::a_batch_counts_as_that_many_raw_events` `decision.rs:856` | |
| Stop line / stats | `the_loop_counts_its_wakes_and_raw_key_events` `:2529`; `the_watchdog_thread_reports_its_wakes_and_raw_key_events_when_it_stops` `:2545` | raw_keys 11; raw_keys 5, wakes >= 1 |
| E10 | `the_raw_input_registration_asks_for_no_device_notifications` `:1949` | `RIDEV_INPUTSINK` present (`installer.rs:134`), no `RIDEV_DEVNOTIFY` |
| E11 byte walk | `:2009`, `:2032` (64-bit only), `:2046`, `:2057`, `:2070`, `:2089` | |
| S1-S4 | `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests` `installer.rs:580` + the three at `:527-577` | |
| No injection / no key data | `the_watchdog_injects_no_input` `:2690`; `no_key_data_leaves_the_raw_input_read` `:2748` (allows `raw_key_event`, `count_raw_keys`, `qualifies`) | |

### Mutations the developer ran, each caught (fix_r1:34-36)

- Decision: credit never lapses; whole interval's callbacks as credit; backlog ignored; evidence kept at a live tick.
- Loop: `wake_mask` ignores ticking; pump takes every message; backlog not passed to tick; batch counted after the answer; no drain after `WM_QUIT`; no rounding in the walk.
- Rows the cut's tester table expects (`cut.md:1063-1073`): add the drain only at due ticks (F8a) and `RIDEV_DEVNOTIFY` added (F4 scan).

### Bench (ruling L1 acceptance line, rulings.md:94-95 and :116-118)

- Command (`src/bench.rs:1021`): `KS_BENCH_OUT=<file> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`. Round-1 tester script, which also sets `KS_BENCH_LABEL` and a per-tree `CARGO_TARGET_DIR`: `D:/tmp/ks-orch/p2-2.4-watchdog/bench_run.sh`. Its worktrees `wt/bench-{master,head}` no longer exist (`wt/` is empty); targets `target-wt/bench-{master,head}` exist.
- `master` (`fa20b30`) has rows up to R13 only (`git show master:src/bench.rs` `:777-795`). R14, R14c, R15, R16 exist on HEAD only.
- Bounds:
  - R14 p50 <= R14c p50 + 300 ns; R14 p99 <= R14c p99 + 2 us (rulings.md:116-118; cut.md:761). R14 vs R12 reported for context.
  - R16 sanity bound (cut.md:984): p50 <= R12 p50 + 25 us; p99 <= R12 p99 + 100 us.
- Developer's single run (`bench-dev-r1.md`, HEAD `e69ce7f`):

| Row | p50 us | p99 us |
|---|---|---|
| R12 | 4.7 | 5.9 |
| R14 | 4.8 | 102.3 |
| R14c | 4.8 | 98.5 |
| R15 | 3.9 | 6.0 |
| R16 | 53.5 | 168.4 |

  R14 p99 is +3.8 us over R14c (rule +2 us); R16 is above its sanity bound.
- Row shapes: R14 `bench.rs:498-515` (sets `hook_installed` true `:501`, 1 post + 5 ms spin, then `paced_calls(2000, 1 ms, post)`); R14c `:570-574` (`ParkedQueue` post, no watchdog); R16 `:583-606` (n = 1000 `:1128`, watchdog per sample, 2 ms spin, post, timed key-down).
- `cut.md:972` still says `r16_first_key_of_a_burst(100)`; ruling V3-M2 (rulings.md:121-122) and the code (`bench.rs:1128`) use 1000.

### Manual checks (owner only; fix_r1:105-113)

- M9: batching on the live daemon; PASS iff K >= 400 and W <= K / 2 (developer's rule, fix_r1:46,109; ruling V3-M1's example was `W <= K / 10 + 20`, rulings.md:119-120).
- M10: handle, USER and GDI object leak check.
- M2 amended: W <= 100 after 10 idle minutes.
- M1: re-run on the batched debug build.
- M3-M8 unchanged.
- No `#[ignore = "manual: ..."]` was added this round (fix_r1:85).

## Facts

### Decision (`decision.rs`)
1. `raw_batch(0, _)` returns without change (`decision.rs:139-141`). The first batch while not ticking sets `next_tick = now + TICK`, `quiet_since = now` and `credit = 0` (`:144-148`).
2. `tick` takes `answered = callbacks.wrapping_sub(last_callbacks)` and `arrived = raw_events.saturating_sub(raw_at_last_tick)` (`decision.rs:172-173`).
3. `explained = arrived.min(credit)`; `unexplained = arrived - explained`; `kept = if backlog { credit - explained } else { 0 }` (`decision.rs:180-182`).
4. With `answered > 0`: `quiet_since = now`, `evidence = 0`, `credit = kept + answered.saturating_sub(unexplained)`, `attempts = 0` (`decision.rs:185-192`).
5. With `answered == 0`: `credit = kept`; `decide` is computed from the evidence **before** `unexplained` is added (`decision.rs:194-199`), with the conditions `!outstanding && evidence >= MIN_EVIDENCE (2) && now >= quiet_since + window()` (`:196-198`).
6. `window()` = `SILENCE * 2^min(attempts, 6)` (`decision.rs:164-166`). Constants: `SILENCE` 1 s, `TICK` 250 ms, `ACTIVE_FOR` 2 s, `MAX_BACKOFF_SHIFT` 6 (`:66-74`).
7. `keeps_ticking` is true while outstanding, within `ACTIVE_FOR` of `last_raw_at`, or with evidence >= 2 before `last_raw_at + window() + TICK` (`decision.rs:227-234`).
8. `start_over` zeroes evidence and credit, sets `quiet_since` and re-bases `raw_at_last_tick` (`decision.rs:265-270`). `answered` and `declined` call it; `not_sent` only clears `outstanding` (`:241-263`).
9. `decision.rs` is `#![forbid(unsafe_code)]` (`:58`).

### Ticker and loop (`mod.rs`)
10. `Ticker::pass` adds `drained.keys` to `stats` before the `hook_installed` gate (`mod.rs:234-235`), calls `raw_batch` only when the hook is installed (`:240-246`), then `take_answer` (`:247`). It ticks only when `next_tick <= now` (`:248`), passing `drained.pending` as `backlog` (`:257`). `arm` runs on every pass (`:262`).
11. `hook_answered()` is called when the heartbeat moved since the last tick (`mod.rs:250-255`), per ruling M3.
12. A `set` failure bumps `WatchdogTimerNotSet` and calls `set_timer_failing(true)`; a success calls `set_timer_failing(false)` (`mod.rs:331-340`). `wake_by` returns `next_tick` only while no timer is armed (`:213-218`).
13. `next_wait` builds its mask as `wake_mask(batched, ticking || drained.stuck)` and returns timeout 0 when `drained.pending` (`mod.rs:1056-1059`). A stuck drain adds a back-off of one `TICK` (`:1060-1064`). Otherwise the timeout is `INFINITE` when there is no `wake_by` (`:1065`).
14. `wake_mask(b, true)` = `QS_ALLINPUT & !b`; `wake_mask(b, false)` = `QS_ALLINPUT` (`mod.rs:750-756`). `Source::Keyboard` batches `QS_RAWINPUT`/`WM_INPUT`; the test-only `Source::Synthetic` batches `QS_POSTMESSAGE`/`SYNTHETIC_RAW = WM_APP + 2` (`:349`, `:366-382`).
15. While ticking, the synthetic source's wait therefore leaves out every posted message, `WM_QUIT` included. The bench relies on `stop` returning by the next tick (`bench.rs:511-512`, `:601-602`). The keyboard source's ticking mask keeps `QS_POSTMESSAGE` (`mod.rs:368`, `:752`).
16. `run_loop` waits with `MsgWaitForMultipleObjectsEx([timer], timeout, mask, MWMO_INPUTAVAILABLE)` (`mod.rs:708-715`). On `WAIT_OBJECT_0` it calls `fired()`; on `WAIT_FAILED` it logs `[ERROR]` and returns (`:716-724`). Every other return makes a pass (`:725-742`).
17. Each wake runs `pump` first. If the pump saw `WM_QUIT`, the loop skips the drain for that pass; otherwise it drains (`mod.rs:729-736`). On quit it counts the drained keys plus one more drain via `read_at_stop`, then returns without a pass (`:737-741`).
18. `pump` peeks with `hwnd = None` over `pump_ranges(skip)` = `[(0, skip-1), (skip+1, 0xFFFF)]`, dispatches every message and loops until a full round takes none. It returns true at `WM_QUIT` (`mod.rs:764-766`, `:780-803`).
19. `drain` loops until a read returns `Empty` with `raw_pending()` false, a `Quit`, or the 5 ms budget by `clock` (`mod.rs:987-1042`). Budget exhaustion sets `pending` (`:997-1000`).
    - `Failed` bumps `RawInputUnread` and switches to standard reads for the rest of the drain (`:1008-1011`).
    - `Empty` with the queue bit still set tries one standard read (only if still buffered). If that finds nothing and the bit is still set, it bumps `RawInputUnread` once and sets `stuck` (`:1016-1038`).
20. `Drained::add` bumps `RawInputUnread` once per unread record in a loop (`mod.rs:971-976`).
21. `KeyboardReads::buffered` calls `GetRawInputBuffer(buffer, &mut size = RAW_BUFFER_BYTES, sizeof(RAWINPUTHEADER))`: 0 gives `Empty`, `u32::MAX` gives `Failed`, otherwise `count_raw_keys` (`mod.rs:1079-1097`).
22. `KeyboardReads::standard` peeks `WM_INPUT` filtered to the sink window, classifies the event with `raw_key_event` (`GetRawInputData`), then `DispatchMessageW` (`mod.rs:1100-1123`).
23. `raw_pending` reads the high word of `GetQueueStatus(QS_RAWINPUT)` (`mod.rs:1125-1131`).
24. `BUFFERED = cfg!(target_pointer_width = "64")` (`mod.rs:922`); `drain_keyboard` passes it (`:1148`).
25. `SyntheticReads::raw_pending` is always false and `buffered` = `standard` (`mod.rs:1161-1185`). The stuck path is therefore never reached through the synthetic source (doc `:1151-1154`).
26. `count_raw_keys` walks plain bytes with `u32_at`/`u16_at` (bounds-checked `get`). Each step is `(at + size).next_multiple_of(RAWINPUT_ALIGN = size_of::<usize>())`. A header that is missing, has `size < header` or overruns the buffer marks the rest as unread and ends the walk. A keyboard record shorter than `body + RAWKEYBOARD` counts 1 unread (`mod.rs:867-917`).
27. `RawBuffer` is `#[repr(C, align(8))]` with `64 * size_of::<RAWINPUT>()` bytes (`mod.rs:844`, `:855-856`), and lives on the watchdog thread's stack (`:659`).
28. `qualifies` = make code != 0xFF and vkey in 1..=254 (`mod.rs:1193-1195`).
29. The stop line `[INFO] hook watchdog stopped after {wakes} wakes for {raw_keys} raw key events` is logged once in `run_watchdog` after `run_loop` returns (`mod.rs:672-676`). It is not logged when the prologue fails or `ready.send` fails (`:618-621`, `:649-651`).
30. `HookWatchdog::stop_thread` posts `WM_QUIT` and waits `STOP_TIMEOUT` = 1 s on `done`. On timeout it returns `None` without joining (`mod.rs:424`, `:500-520`). `spawn` waits `START_TIMEOUT` = 5 s on `ready` (`:422`, `:464-466`).
31. The registration is removed before the window is destroyed: `_registration` is declared after `window` (`mod.rs:630-641`). Both drop on the watchdog thread (`:559-567`, `:599-605`).
32. Added `unsafe` blocks with SAFETY comments (fix_r1:70-78):
    - production: `mod.rs:784-787` (pump peek), `:792-796` (dispatch), `:1081-1092` (`GetRawInputBuffer`), `:1101-1105` (standard peek), `:1115-1118` (dispatch), `:1126-1128` (`GetQueueStatus`), `:705-715` (wait);
    - test only: `:1167-1170` (synthetic peek), and in tests `:2332-2342`, `:2436-2470`, `:2571-2604`;
    - bench: `bench.rs:532-538`, `:551-553`.
33. `fix_r1.md:69`: no new atomics or shared state. `LoopStats` crosses threads only through the `done` channel (`mod.rs:416`, `:457`, `:462`).

### Scan (`installer.rs`)
34. `scan_source` checks every line of a file, test modules included. Only the lines of the `#[cfg(not(test))]` `mod win32 {` block of `src/hook/installer.rs` are allowed (`installer.rs:486-512`).
35. `INSTALLERS` matches the prefix `SetWindowsHook`, plus `UnhookWindowsHookEx`, `SetWinEventHook`, `UnhookWinEvent` and `RegisterRawInputDevices` (`installer.rs:386-392`).
36. `scanned_sources` = `build.rs` + every `.rs` under `src`, recursively (`installer.rs:414-419`). `tests/` is read with a non-recursive `read_dir` (`:610-619`).
37. `code_of` drops `//` comments and string-literal contents per line (`installer.rs:424-446`). It does not track char literals or multi-line strings (fix_r1:150, "open point 4").

### Docs
38. `counters.rs:48-57` now says that a failed timer set is retried at the tick through the wait's timeout (R-m1).
39. README `:939` quotes R16 "about 50 microseconds slower at the median on the developer's machine"; `mod.rs:35-37` says the same. `cut.md:1040` (planned) said "about 20 us".
40. `errors.md:215` adds the stop-line row, and `errors.md:231` extends the `raw_input_unread` wording. No `Counter` variant was added this round: the `counters.rs` diff is doc comments only.
41. The `cut.md` E7 claim "a filtered peek still returns WM_QUIT" (`cut.md:870`) is contradicted for a posted `WM_QUIT` by the test at `mod.rs:2561-2615` (its assertion `:2608`).

## Invariants in play

- **S1-S4** (rulings.md:7-23): installer calls only in `installer.rs` `#[cfg(not(test))] mod win32` (`installer.rs:60`); `cfg(test)` `System` panics (`:162-194`); scan `:580`. The watchdog reaches registration only through `installer::SYSTEM` (`mod.rs:594`, `:601`). The synthetic source never registers (`mod.rs:634-641`).
- **R1 hook callback**: not touched this round. The per-key main-thread cost is the heartbeat increment (README `:939`).
- **R2 sentinel / no injection**: `the_watchdog_injects_no_input` `mod.rs:2690` scans `watchdog/mod.rs`, `decision.rs`, `health.rs`, `installer.rs` and the 4 reinstall fns (non-test lines only, `:2657-2664`).
- **R3 shared state**: no new shared state (fact 33). Tests use `leaked_counters()` / `leaked_health()` / `Box::leak` health (`mod.rs:1368`, `:2370`), per ruling M4.
- **R4 orderings**: no new atomics this round (fix_r1:69). Not re-read in `health.rs`.
- **R6 Display**: new types `LoopStats`, `Drained`, `BlockCount`, `Read`, `Plan` derive `Debug` only (`mod.rs:121`, `:144`, `:860`, `:930`, `:956`). No new `Counter` variant.
- **R10 message loop / shutdown**: the watchdog thread pumps itself (`pump`); stop via posted `WM_QUIT` (`mod.rs:508`); `App::run` starts the watchdog before `install` and stops it after (`core/app.rs:217-228`, `:321`, `:332-336`; test `app.rs:469`). Ruling I3.
- **I1** tick never lost: `pass`->`arm` every wake (`mod.rs:262`); `wake_by` timeout (`:213`, `:1061-1065`).
- **M1** idle: fact 7; `mod.rs:1665`.
- **M5**: fact 5.
- **V3-I1**: facts 3-5, 19; `next_wait` timeout 0 on pending (fact 13).
- **V3-M3**: facts 19, 13.
- **OQ2**: `pump` has no window filter (fact 18).
- **KS-08** (the issue), **KS-01** (reinstall on the installing thread; `main_thread.rs`, unchanged this round).

## Open points

### From `triage_r1.md`, with their status at HEAD
- **T-F1 = R-I2** (R14 regression, `triage_r1.md:17-31`, `:48-52`): addressed by ruling L1 as built (`a0aaf7c`). Developer: R14 p50 4.8 = R14c p50 4.8; R14 p99 +3.8 us over R14c in one run, against a +2 us rule (fix_r1:39, :98, :148). Per fix_r1 the tester's 3 runs decide.
- **R-I1** (S4 scan, `triage_r1.md:32-46`): `ee9ef4d`/`0e7165b`. Reviewer re-check required (rulings.md:96). Residual limits: facts 36-37.
- **R-m1**: doc fixed (fact 38).
- **R-m2** NOTED, unchanged: `src/logging/panic.rs:93` `let _ = report_on(tx, line, deadline);`.
- **R-m3** NOTED, unchanged: the "never two, and never none" wording at `src/hook/keyboard_hook.rs:263-267`.
- **R-m4** NOTED, unchanged: README tree `README.md:64-74` lists `health.rs`, `installer.rs`, `watchdog/` (`:69-71`). `src/hook/debug_hold.rs` exists, and `grep debug_hold README.md` finds nothing.
- **T-note** NOTED, unchanged: `START_TIMEOUT` 5 s (`mod.rs:422`, `:464-466`); `ready` is `sync_channel(1)` (`:456`); the thread continues if `ready.send` succeeds (`:649-651`); `App::run` logs the watchdog off on error (`core/app.rs:220-228`).
- Rulings Delta Q5 keep m2-m4 and the T-note NOTED (fix_r1:43).

### From `fix_r1.md` "Open points"
1. R16 above its sanity bound: p50 53.5 / p99 168.4 us against 29.7 / 105.9 us. It needs an orchestrator ruling (fix_r1:1, :147).
   - Diagnostics in `bench-dev-diag.md`: R16's construction on `0e7165b` gives p50 35.6 us; at `THREAD_PRIORITY_HIGHEST` 58.5 us.
2. R14 p99 vs R14c p99 (above).
3. Disk space (Scope and delta).
4. Scan limits (facts 36-37).
- Deviation 11: the M9 pass rule differs from ruling V3-M1's example (fix_r1:46, :65).
- OQ4 (`HRAWINPUT` against USER quota): unverified, observed by M9/M10 (rulings.md:131-132).

## Not covered

- Not read this round: `src/hook/health.rs`, `main_thread.rs`, `keyboard_hook.rs` (beyond `:258-270`), `keyboard_hook_callback.rs`, `core/app.rs` (beyond grep lines), `platform/*`, `logging/panic.rs` (beyond `:88-96`), `ipc.rs`, `bin/swch.rs`, `tests/ipc_roundtrip.rs`. None is in the delta.
- Not read in `mod.rs`: tests `:1504-1547`, `:1805-1909`, `:1965-2148`, `:2376-2426`, `:2492-2527` beyond their names; `decision.rs` tests `:274-697`, `:856-927` beyond names.
- `cut.md` was read only at `:861-877` (L1.3) and `:962-1086` (L1.6-L1.11), plus a heading grep. `critique_v3.md`, `review_r1.md`, `test_r1.md` (beyond grep) and `impl.md` were not read.
- Microsoft Learn was not fetched. The `GetRawInputBuffer` `pcbSize` in/out semantics and E2-E4 are taken from `cut.md` L1.3, not verified here (UNVERIFIED).
- No build, test, clippy or bench was run by the scout. Every count and result above is quoted from `fix_r1.md`, the commit messages or round-1 logs.
