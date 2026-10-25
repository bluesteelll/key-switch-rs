DONE (open point for a ruling: bench row R16 is above its L1.6 sanity bound; see "Open points" 1)

# Implementation: p2-2.4-watchdog fix round 1

Branch `feat/hook-watchdog`, HEAD `959fce1bbf3a0bb2f5d0b32ae254b6220dfdc9fd`, base of this round `140b831`. The tree is clean; `git worktree list` shows only the lane tree.
Inputs: `triage_r1.md` (3 CONFIRMED: T-F1, R-I1, R-I2; T-F1 = R-I2), `rulings.md` rounds 2 (L1) and 3 (V3-*), `cut.md` "Delta after ruling L1" (F1-F10), `critique_v3.md`.
This round ran in two parts: part 1 fixed R-I1 and stopped for a ruling on T-F1; part 2 (this resume) implements ruling L1 with the round-3 rulings. The Developer addendum in `cut.md` ("Fix round 1: the L1 delta with the round-3 rulings") records every ruling as carried out.

## Items

### R-I1 (part 1, kept: `ee9ef4d` red, `0e7165b` green; test code only)
- `src/hook/installer.rs`: the S4 scan reads every line of every `.rs` file under `src` (test modules included) and `build.rs`, and matches `SetWindowsHook` as a prefix.
- Red on `ee9ef4d` (`cargo test --locked --lib hook::installer`): `an_installer_call_in_a_tests_module_is_found` `assertion left == right failed: the call in the tests module: []`; `every_spelling_of_the_hook_installer_is_found` `SetWindowsHookExA is not found: []`; `the_build_script_is_scanned` `build.rs is not among the scanned sources` (2 passed, 3 failed) -> green on `0e7165b` (5 passed). End-to-end probe in a scratch worktree: the fixed scan finds 3 probe calls the old scan missed.

### T-F1 = R-I2: ruling L1 (raw input read in batches at the ticks), with rulings V3-I1..V3-M3
All in `src/hook/watchdog/mod.rs` (`mod.rs`) and `src/hook/watchdog/decision.rs` (`decision.rs`) unless named.

| Commit | Item | Where (HEAD) | Red first -> green |
|---|---|---|---|
| `65c4d84` F1 | Windows: a pending message outside the wake mask leaves an `MWMO_INPUTAVAILABLE` wait asleep (E4) | test `a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name` `mod.rs:2428` | declared green characterisation (`test result: ok. 1 passed`) |
| `4c711ed` / `0478a2c` F2/F3 | `LoopStats`, the stop line, `done: Receiver<Option<LoopStats>>`, `stop_counting` | `mod.rs:145`, `:493-524`, `run_watchdog` `:611-678`; errors.md row; CHANGELOG "New lines" | red: `error[E0422]: cannot find struct, variant or union type LoopStats`; `error[E0599]: no method named stats found for struct Ticker<T>`; `error[E0599]: no method named stop_counting found for struct HookWatchdog` -> green |
| `f58c95c` / `f222c5c` F4/F5 | `wake_mask`, `pump_ranges`, `Source::{batched_wake, batched_message}`; RIDEV_DEVNOTIFY scan | `mod.rs:750-766`, `:366-384`; tests `:1911`, `:1930`, `:1949` | red: `error[E0425]: cannot find function wake_mask`, `... pump_ranges`; `error[E0599]: no method named batched_wake / batched_message found for enum Source` -> green; the scan green on arrival |
| `e712a2b` / `76551be` F6/F7 | `RawBuffer`, `BlockCount`, `count_raw_keys` (safe byte walk at `offset_of!` offsets, step rounded to a pointer) | `mod.rs:844-917`; tests `:2009-2100` | red: `error[E0425]: cannot find function count_raw_keys`; `error[E0422]: ... BlockCount`; `error[E0425]: cannot find type RawBuffer`; `... value RAW_BUFFER_BYTES`; `cannot find macro offset_of` -> green |
| `ea8f19d` / `30ac79d` G1/G2 (V3-I1, decision) | accumulated credit: `credit` replaces `owed`; `tick(now, callbacks, backlog)`; the baseline at a live tick; the rest of an old credit kept only while the drain left raw input queued | `decision.rs:1-28` (docs), `:101`, `:171-215`; tests `:699`, `:728`, `:755`, `:793`, `:835` | red by assertion: `a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it` `assertion left == right failed  left: [1250]  right: []`; `a_hook_that_dies_during_a_burst_asks_for_exactly_one_reinstall` `assertion left == right failed: []  left: 0  right: 1` -> green. The three other tests are green on both and pin the lapse and the baseline |
| `c605d0f` / `0ac825a` F8c/F8d (V3-I1 drain, V3-M3, OQ3) | `drain` through the `RawReads` seam: until empty, 5 ms time budget (`pending`), failed buffered read -> standard read, the `GetQueueStatus` guard (`stuck`); `next_wait`; `KeyboardReads`, `drain_keyboard` | `mod.rs:922-1153`; tests `:2151-2357` | red: `error[E0405]: cannot find trait RawReads`; `error[E0425]: cannot find function drain / next_wait / drain_keyboard`; `error[E0422]: ... Drained`; `error[E0433]: cannot find type Read` -> green. Smoke test: `GetRawInputBuffer` returned 0 on the developer's Windows 11 build 26200 |
| `384ddf5` F8a | a ticking thread is not woken per raw event | test `mod.rs:2618` | red by assertion: `a ticking watchdog woke 194 times for 190 raw key events (at most 12)` -> green |
| `d668a13` F8b | batched `Drive` (queued raw events, wakes only while idle, per-pass budget, re-drain), the L1.7 rows, the V3-I1 flood rows, `a_batch_counts_as_that_many_raw_events` | `Drive` `mod.rs:1287-1498`; rows `:1505-1910`; `decision.rs:856` | red: `error[E0599]: no method named raw_batch found for struct Watchdog`; `error[E0599]: no method named is_ticking found for struct Ticker<T>`; `error[E0061]: this method takes 2 arguments but 4 arguments were supplied` -> green |
| `a0aaf7c` F9 | the batched loop: `run_loop` (`next_wait`, pump on every wake, drain, quit drain), `pump` (ranges, no window filter), `Ticker::pass(now, drained, ..)` with `raw_batch` before the answer, `SyntheticReads`; every dead-code expectation removed; docs | `mod.rs:194-260`, `:394-405`, `:688-744`, `:780-803`, `:1156-1191`; `decision.rs:138-157`; `src/logging/counters.rs:48-57`; README `:905`, `:939`; CHANGELOG; errors.md | makes F8a and F8b green (`hook::watchdog`: 69 passed) |
| `e69ce7f` F10 | bench: R14 (steady typing, 1 ms, watchdog ticking), R14c (control, V3-I2), R16 (first key of a burst, `hook_installed`, n = 1000, V3-M2); R14/settled removed | `src/bench.rs:59-80`, `:455-611`, `:1109-1130`; CHANGELOG bench entry | no red (ignored bench); developer run below |
| `e75def6` | README and module docs quote R16 as measured (V3-M2) | README `:939`, `mod.rs:33-38` | docs |
| `a90b38b` | OQ2 characterisation: a range-filtered peek leaves a posted `WM_QUIT`; the pump takes it | test `mod.rs:2561`; comments `:1109-1110`, `:1174-1175`, `:777-779` | declared green characterisation |
| `959fce1` | the two characterisations' SAFETY comments state their invariants | tests `:2428`, `:2561` | comments only |

Mutations run by the developer, each caught (reverted after each run):
- Decision: the credit never lapses (`a_hook_that_dies_after_a_burst_...`, `a_hook_that_dies_after_long_drift_...`); the whole interval's callbacks as credit (`a_hook_that_dies_during_a_burst_...`); the backlog ignored (`a_burst_read_over_several_ticks_...`); the evidence kept at a live tick (`evidence_is_checked_before_this_intervals_raw_events_are_added`).
- Loop: `wake_mask` ignores ticking (9 tests, F8a and `while_ticking_raw_key_events_wake_nothing` among them); the pump takes every message (F2's thread test, F8a, `synthetic_raw_events_with_a_silent_heartbeat_...`); the backlog not passed to the tick (`a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request`); the batch counted after the answer (`raw_events_read_with_an_answer_are_no_evidence_after_it`); no drain after `WM_QUIT` (F2's thread test, F8a); no rounding in the walk (`the_walk_steps_...`, `a_keyboard_record_too_short_...`).

## Remarks
- **T-F1** (triage): RESOLVED by ruling L1 as built (`a0aaf7c`): while ticking the watchdog does no work per key event. Developer bench (below): R14 p50 4.8 us vs R14c p50 4.8 us (bound +0.3 us: met); R14 p99 102.3 us vs R14c p99 98.5 us (+3.8 us, bound +2 us: not met in this one run; both p99s sit near 100 us and moved by tens of us between runs, R14c/high 57.3 us; the tester's 3 runs decide). Against R12 for context: p50 4.8 vs 4.7 us.
- **R-I1**: RESOLVED (part 1, `ee9ef4d`/`0e7165b`), pinned by the three scan tests and the main scan. No new installer call on HEAD: `hook::installer` tests pass (in the 141-test run).
- **R-I2**: RESOLVED with T-F1 (same defect).
- **R-m1**: RESOLVED in F9 (ruling Delta Q5): `src/logging/counters.rs:54-57` says the timer is set again at the tick through the wait's timeout.
- **R-m2, R-m3, R-m4, T-note**: NOTED by the triage and kept outside this delta by ruling Delta Q5 ("m2-m4 and the T-note stay NOTED"); not changed. Directions as in part 1 (m2: stderr fallback when the writer is `Gone`; m3: qualify "never two hooks or none" for a foreign hook; m4: README tree line `debug_hold.rs`; T-note: cosmetic 5 s start window).
- **critique_v3 I1 / V3-I1**: RESOLVED (G1/G2, F8c/F8d, F8b flood rows, F9), rule stated in the addendum; pinned by the tests named in Items.
- **critique_v3 I2 / V3-I2**: RESOLVED: R14c `src/bench.rs:570`.
- **critique_v3 M1 / V3-M1**: RESOLVED with one rule, **PASS iff K >= 400 and W <= K / 2**. The ruling's example `W <= K / 10 + 20` is refuted by M9's own arithmetic: the 20 s hold ticks about 22.25 s, about 90 wakes, above 600 / 10 + 20 = 80, so a correct build would fail it. Manual checks below; the orchestrator may restate the number.
- **critique_v3 M2 / V3-M2**: RESOLVED in code (`src/bench.rs:583`: `set_hook_installed(true)`, n = 1000; README number measured, `e75def6`). The row's result is above its sanity bound: Open points 1.
- **critique_v3 M3 / V3-M3, OQ1**: RESOLVED: the guard in `drain` (`mod.rs:1016-1043`), the back-off in `next_wait` (`mod.rs:1049-1068`), fake tests `mod.rs:2221`, `:2243`; the synthetic difference documented at `SyntheticReads` (`mod.rs:1151-1156`).
- **OQ2**: RESOLVED (`pump`, no window filter) and sharpened by `a90b38b`: cut E7's "a filtered peek still returns WM_QUIT" does not hold for a posted `WM_QUIT`. Found by a diagnostic loop without the pump, whose stop timed out; the shipped loop always pumps, so the design stands.
- **OQ3**: RESOLVED: both documented outcomes accepted and named (`mod.rs:2319`); observed 0.
- **OQ4**: recorded as unverified (addendum); M9/M10 observe it.
- **Delta Q1-Q6**: carried out as recommended (addendum).

## Deviations from the cut
1. `Ticker::pass(now, drained: Drained, context)` instead of `pass(now, raw_keys, context)`, so the `Drive` covers the backlog plumbing.
2. `RawReads` seam (`Read`, `Drained`, `drain`, `KeyboardReads`, `SyntheticReads`) and a pure `next_wait`; `DRAIN_BUDGET` = 5 ms replaces `MAX_BUFFERED_READS`/`MAX_STANDARD_READS` (ruling V3-I1: a time budget, not a record cap).
3. `Watchdog::tick` gains `backlog` (V3-I1). In G1 the parameter was a stub (`let _ = backlog;`) so that the red was an assertion, not a compile error; G2 uses it.
4. `SyntheticReads` reads one message per read and reports no queue bit; `SYNTHETIC_RAW` is `pub(crate)` (test only) for R14c.
5. A keyboard record too short for its key fields is counted unread (the cut skipped it).
6. The two idle-tick loop rows assert "nothing armed, no pass after the idle tick" instead of a `Cancel` call: the timer has signalled for that tick, so `plan` cancels nothing.
7. F1's lower bound is 80 ms, not 90 ms (one clock tick of slack).
8. The stuck queue counts in `raw_input_unread` (no new counter).
9. Commit order G1/G2 after F7, F8c/F8d before F8a/F8b; the OQ3 smoke test moved from F8b to F8c.
10. Bench helpers `bind_f13_to_do_nothing`, `spin_for`, `paced_calls`, `ParkedQueue`.
11. M9's pass rule (V3-M1) is `W <= K / 2` with `K >= 400`, not the ruling's example (Remarks).
12. Two follow-ups beyond F1-F10: `a90b38b` (OQ2 characterisation) and `959fce1` (SAFETY wording).

## Unsafe / atomics added
No new atomics or shared state; `LoopStats` crosses threads only through the `done` channel.
- `mod.rs:787` pump `PeekMessageW`: "`msg` is the loop's local and outlives every call, which only writes it and removes the message from this thread's own queue."
- `mod.rs:796` pump `DispatchMessageW`: "`msg` was filled by PeekMessageW above and outlives the call, which only reads it; a thread message goes nowhere, and one for the sink window reaches the system class's procedure."
- `mod.rs:1086` `GetRawInputBuffer`: "`self.buffer` is RAW_BUFFER_BYTES of initialised bytes, aligned to 8 as the call asks, alive for the call; `size` says how many bytes it holds, so the call writes within it; the header size is RAWINPUTHEADER's. The call reads the calling thread's queue, which is this thread's."
- `mod.rs:1105` standard read `PeekMessageW(Some(window), WM_INPUT, WM_INPUT)`: "`self.msg` outlives the call, which only writes it and removes the message from this thread's own queue; `self.window` is this thread's live sink window."
- `mod.rs:1118` standard read `DispatchMessageW`: "... the window's procedure is DefWindowProcW, which frees the raw input, as WM_INPUT asks."
- `mod.rs:1128` `GetQueueStatus`: "takes the flags by value, reads no memory of ours and reports this thread's own queue."
- `mod.rs:1170` synthetic `PeekMessageW` (test only): "`self.msg` outlives the call, which only writes it and removes the message from this thread's own queue."
- `mod.rs:708` the wait (rewritten, same invariant as before): "the slice holds the tick timer's handle, which `ticker` keeps open for the whole call ..."
- Tests: F1 (`:2441-2470`), the smoke test (`:2336`), the WM_QUIT test (`:2575-2600`), each with its own invariant; bench `ParkedQueue` (`src/bench.rs:536-553`).

## Gates
- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass; also with `--release`: pass. Each green commit was linted before it was made.
- Tests run: per commit the module under change; at HEAD `cargo test --locked --lib -- hook::watchdog hook::installer logging:: core::app hook::health`: `test result: ok. 141 passed; 0 failed`. The full suite is left to the tester.
- Book: `mdbook build book -d D:/tmp/ks-orch/p2-2.4-watchdog/book`: built (the existing mdbook-mermaid version warning only).
- Safety: no test registers raw input, installs a hook or injects input. The new Win32 tests (F1, the WM_QUIT test, the smoke test) touch only their own thread's queue; the smoke test's thread has a sink window and no registration. The S4 scan passes. I never ran the daemon.
- Manual ignores added: none (the manual checks below need the live daemon and have no ignored test, as in the cut).

## Bench (developer run, once, HEAD `e69ce7f`, release, quiet machine)
`D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev-r1.md` (log `bench-dev-r1.log`, 258 s):

| Row | n | p50 us | p99 us | max us |
|---|---|---|---|---|
| R12 | 2000 | 4.700 | 5.900 | 234.600 |
| R14 (1 ms pacing, watchdog ticking) | 2000 | 4.800 | 102.300 | 327.100 |
| R14c (same pacing and post, no watchdog) | 2000 | 4.800 | 98.500 | 366.000 |
| R15 | 2000 | 3.900 | 6.000 | 20.500 |
| R16 (first key of a burst) | 1000 | 53.500 | 168.400 | 314.300 |

- R14 vs R14c (L1's acceptance line): p50 +0.0 us (bound +0.3 us: met); p99 +3.8 us (bound +2 us: not met in this run, noise level; see Remarks T-F1).
- R16 vs its sanity bound (p50 <= R12 + 25 us = 29.7 us; p99 <= R12 p99 + 100 us = 105.9 us): **not met** (53.5 / 168.4 us). Open points 1.
- Diagnostics (`bench-dev-diag.md`; temporary ignored tests, never committed, removed after each run):
  - R16 variants at `e69ce7f`: as built p50 51.7 us; hook not installed (no `raw_batch`, no timer set) 53.6 us; the timed call 200 us after the post 16.5 us; a reused thread, idle 2.4 s before each sample (n = 20) 79.7 us.
  - R16's construction on the pre-L1 tree `0e7165b` (a wake from idle, one wake per key): p50 35.6 us (R12 4.8 us).
  - The bench thread at `THREAD_PRIORITY_HIGHEST`, as the daemon's main thread: R12 4.7, R14 4.8 (p99 89.4), R14c 4.9 (p99 57.3), R16 58.5 us at p50.

## Manual checks (owner; agents never run the daemon; to the owner from the start)
- **M9 (new): batching on the live daemon.** `cargo build --release --locked`. Precondition (ruling M7): `swch exit` any running daemon and check that `swch status` says it is not running; start the release daemon fresh (`swch open`, elevated as usual), so that its stop line covers this run only. Open Notepad.
  1. Leave the machine untouched for 60 s. 2. In Notepad hold `a` for 20 s, then release. 3. Wait 5 s. 4. Run `swch status` and record the output. 5. Run `swch exit`. 6. `Select-String 'hook watchdog stopped' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" | Select-Object -Last 1` (read only).
  - Expected: step 4 shows `hook ok, reinstalls: 0`; step 6 shows `[INFO] hook watchdog stopped after W wakes for K raw key events`; no counters line with `raw_input_unread` above 0.
  - **PASS iff K >= 400 and W <= K / 2.** Margin: a correct build wakes about 90 times for the hold (4 a second over about 22 s of ticking, plus the first) and about 75 times for the setup's own bursts (the Enter of `swch open`, opening Notepad, typing the two commands, a window switch): about W 165 for K 640 at Windows' default repeat rate, half the bound. One wake per key event gives W >= K, twice the bound: that FAIL says the wait does not leave `QS_RAWINPUT` out (E4); report it, and the orchestrator rules on the L1.5 fallback. K < 400 (a slow repeat rate) voids the run: repeat with a 40 s hold.
- **M10 (new): no leak from buffered reads (E9, OQ4).** Release daemon running; Task Manager, Details, with the columns Handles, USER objects, GDI objects and Memory (private working set). Record the daemon's four values; in Notepad hold `a` 60 s, release, wait 10 s, record again; repeat the hold once and record a third time. Expected: Handles, USER and GDI objects unchanged (+-2) between the second and third records; the private working set grows < 1 MB between them.
- **M2 (amended).** As M2 (10 minutes untouched, `powercfg /requests` read only), then `swch exit` and read the stop line as in M9 step 6. Expected as M2, and **W <= 100** (the start and the two typed commands, about 45). A watchdog that ticked through the idle minutes would show about 2 400.
- **M1 (re-run on the batched debug build).** As M1 (`impl.md`): `reinstalls: 1` within 2 s of ScrollLock; batching does not change the detection bound (L1.4).
- M3-M8 of `impl.md` are unchanged.

## Docs
- `README.md:905` (Technologies: `GetRawInputBuffer`), `:939` (watchdog bullet: batched reads, the one wake per burst, R16's measured number, Q6).
- `CHANGELOG.md` `[Unreleased]` Added: the watchdog entry (batched reads, totals matching, the stop line), the bench entry (R14, R14c, R15, R16).
- `book/src/config/errors.md`: new row for the stop line; the counters row's `raw_input_unread` wording.
- `src/logging/counters.rs:48-57` doc comments (raw_input_unread, watchdog_timer_not_set: triage m1).
- No config meaning changed: no `default_config.ron` or config page change.
- `cut.md`: Developer addendum, new subsection "Fix round 1: the L1 delta with the round-3 rulings (developer)" and the ruling line under the old open point.
- `impl.md`, "Book pages left to the Phase-2 book sync": one line added for L1 (`architecture.md` thread table and KS-08 cost sentence).

## Commits
- `ee9ef4d` Tests: the installer scan misses test modules, other spellings and build.rs (red)
- `0e7165b` Tests: the installer scan reads whole files, every SetWindowsHook spelling and build.rs
- `65c4d84` Tests: a message wait leaves out a pending message its wake mask does not name
- `4c711ed` Tests: the watchdog reports its wakes and raw key events when it stops (red)
- `0478a2c` Hook: the watchdog counts its wakes and logs them when it stops
- `f58c95c` Tests: the watchdog's wait and pump leave the batched raw input in the queue (red)
- `f222c5c` Hook: the wait mask and the pump ranges of a batched read
- `e712a2b` Tests: raw key events in a buffered read are counted in one pass (red)
- `76551be` Hook: count the raw key events of a buffered read
- `ea8f19d` Tests: a burst whose raw input is read late is matched by the callbacks already counted (red)
- `30ac79d` Hook: the watchdog credits callbacks against raw events by accumulated counts
- `c605d0f` Tests: a drain reads raw input until the queue is empty, within a time budget (red)
- `0ac825a` Hook: the drain of the watchdog's raw input
- `384ddf5` Tests: a ticking watchdog thread is woken by every synthetic raw event (red)
- `d668a13` Tests: the watchdog reads raw input in batches at its ticks (red)
- `a0aaf7c` Hook: the hook watchdog reads raw input in batches at its ticks
- `e69ce7f` Bench: R14 times steady typing with the watchdog ticking, R16 the first key of a burst
- `e75def6` Docs: the README and the watchdog's docs give the measured cost of the first key of a burst
- `a90b38b` Tests: a range-filtered peek leaves a posted WM_QUIT for the pump
- `959fce1` Tests: the SAFETY comments of the queue characterisations state their invariants (HEAD)

## Open points / pre-existing warnings
1. **R16 above its sanity bound (needs an orchestrator ruling; not a code defect found).** Measured p50 53.5 us, p99 168.4 us against bounds of 29.7 / 105.9 us. The bound was derived from the pre-L1 R14, whose posts came back to back to a busy thread; a wake from idle cost more already before L1 (R16's construction on `0e7165b`: p50 35.6 us, also above the bound), and before L1 every key of human typing woke the idle thread that way, while after L1 only the first key of a burst does. The extra cost is not the decision's work (53.6 us with no `raw_batch` and no timer set), not the bench thread's priority (58.5 us at `HIGHEST`), and lasts past the watchdog's own pass (16.5 us with the call 200 us after the post; 79.7 us after a 2.4 s idle). Whether it reaches a real key depends on cut Q6 (raw input before or after the hook call), which no agent can measure. Options for the ruling: restate R16 as a declared evidence row with the measured numbers; or a new delta that reduces the idle wake (for example fewer user32 calls on the first pass), with no guarantee of reaching +25 us given the pre-L1 figure.
2. R14 p99 vs R14c p99: +3.8 us in the developer's single run against the +2 us rule; the p50 rule holds. The paced rows' p99 swings by tens of microseconds between runs (R14c 98.5 vs 57.3 us), so the tester's 3 runs decide.
3. Disk: during the diagnostic builds drive D: reached 0 bytes free (a rustc write failed with "no space on device"). I removed my own scratch artifacts (the worktree `wt/r16-pre`, cleanly: `checkout -- .` then `git worktree remove`; `target-wt/r16-pre`; `target-wt/mut` from part 1's probe), after which D: showed 18.7 GB free; the size of that change suggests another program on the machine also freed space. `target-wt/parent`, `bench-head`, `bench-master`, `target-master`, `target-review` belong to other stages and were left.
4. From part 1, still open: the S4 scan is lexical and per line (a call after a `'"'` char literal on the same line is missed; a multi-line string's inner lines read as code); `tests/` is read only at the top level.
5. No pre-existing warnings.
