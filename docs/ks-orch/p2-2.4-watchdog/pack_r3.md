# Context pack r3: p2-2.4-watchdog (roadmap 2.4, KS-08)

For the round-3 tester and reviewer. Facts only. Line numbers are at HEAD `4d6b228` unless a commit is named. Inputs: `fix_r2.md`, `triage_r2.md`, `rulings.md` (rounds 1-4), `test_r2.md`, `review_r2.md`, `cut.md`, `impl.md`.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/hook-watchdog`, HEAD `4d6b228`. `git status --short` was empty when read.
- `master` = merge base = `fa20b30`. The whole lane vs master: `git diff --stat master...feat/hook-watchdog` gives 28 files, +7024/-161. This pack covers only the round delta.
- `git log --oneline 959fce1..feat/hook-watchdog` (oldest last):

| Commit | Parent | Subject | Files |
|---|---|---|---|
| `4d6b228` | `1db14ca` | Docs: R16 is stated as measured, once per burst and in the worst order | README.md, src/bench.rs, src/hook/watchdog/mod.rs |
| `1db14ca` | `fef60f9` | Docs: the watchdog's stop line is read right for isolated bursts | book/src/config/errors.md |
| `fef60f9` | `3117b0a` | Hook: the watchdog clears its read buffer after each batch is counted | src/hook/watchdog/mod.rs |
| `3117b0a` | `62cf30a` | Tests: a batch's key codes do not outlive its count (red) | src/hook/watchdog/mod.rs |
| `62cf30a` | `f51dc7d` | Hook: a buffered read's answer is taken from its buffer by one method | src/hook/watchdog/mod.rs |
| `f51dc7d` | `995892f` | Tests: an early timer signal leaves the ticks where they were | src/hook/watchdog/mod.rs (tester, round 2) |
| `995892f` | `7300822` | Tests: a wait is rounded up to whole milliseconds and stays below INFINITE | src/hook/watchdog/mod.rs (tester, round 2) |
| `7300822` | `959fce1` | Tests: credit that matched raw events of one tick is not kept for the next | src/hook/watchdog/decision.rs (tester, round 2) |

- `git diff --stat 959fce1..feat/hook-watchdog`:
  - `README.md` 2 +-
  - `book/src/config/errors.md` 2 +-
  - `src/bench.rs` 5 +-
  - `src/hook/watchdog/decision.rs` +35
  - `src/hook/watchdog/mod.rs` 103 +-
  - Total: 5 files, +131/-16.
- The production code that changed in this round is `RawBuffer::take` and its one caller. All other changes are docs or `mod tests`. The commits `7300822`, `995892f` and `f51dc7d` (the tester's, round 2) touch only `mod tests` (`triage_r2.md`, "Not triaged").
- The last bench runs were on tree `959fce1`. Production code changed after them (`62cf30a`, `fef60f9`). `fix_r2.md` "Gates" says "Bench: not re-run ... The tester decides on re-runs".

## File map

| path:lines | What lives there | Why it matters this round |
|---|---|---|
| `src/hook/watchdog/mod.rs:23-43` | the module docs, "# Cost" | L2 wording: R16 is a declared row, the worst order, M6 settles it (`4d6b228`) |
| `src/hook/watchdog/mod.rs:51-58` | the module docs, "# No key data" | R-m2 wording: the buffer is cleared after each batch, and the standard read's record is a local (`fef60f9`) |
| `src/hook/watchdog/mod.rs:233-256` | `Ticker::pass`, first-arrival log `:249` | the `[INFO] hook watchdog: raw keyboard input arrives` line that M6 part B reads; it is guarded by `raw_keys > 0 && hook_installed()` (`:246`) |
| `src/hook/watchdog/mod.rs:665` | `let mut buffer = RawBuffer([0; RAW_BUFFER_BYTES]);` in `run_watchdog` | the buffer lives for the thread's run, then goes to `run_loop` (`:666-673`) |
| `src/hook/watchdog/mod.rs:776-779` | `wait_ms` | pinned by tester test T2 (`:2377`) |
| `src/hook/watchdog/mod.rs:819-846` | `raw_key_event` (standard read) | its record `raw` is a local (`:820`), as the "No key data" docs say |
| `src/hook/watchdog/mod.rs:850` | `RAW_BUFFER_BYTES = 64 * size_of::<RAWINPUT>()` | the size of the clear |
| `src/hook/watchdog/mod.rs:861-883` | `RawBuffer`, `impl RawBuffer { fn take }` | **R-m2 code**: the seam `62cf30a`, then the clear `fef60f9` |
| `src/hook/watchdog/mod.rs:911-946` | `count_raw_keys` | called by `take` (`:872`) before the clear |
| `src/hook/watchdog/mod.rs:958-969` | `enum Read` | the result of `take` |
| `src/hook/watchdog/mod.rs:1097-1121` | `KeyboardReads`, `buffered()` | the only production caller of `take` (`:1120`); the `GetRawInputBuffer` unsafe block is at `:1113-1119` |
| `src/hook/watchdog/mod.rs:1159-1172` | `drain_keyboard` | builds `KeyboardReads` with the thread's buffer |
| `src/hook/watchdog/mod.rs:1179-1190` | `SyntheticReads` (`cfg(test)`) | `buffered()` = `standard()`; it never calls `take` |
| `src/hook/watchdog/mod.rs:1350-1525` | test `Drive` (field `early` `:1364`, signal `:1498`), `a_burst` `:1523` | harness of T3 |
| `src/hook/watchdog/mod.rs:1543-1557` | test `an_early_timer_signal_leaves_the_ticks_where_they_were` | T3 (tester, `f51dc7d`) |
| `src/hook/watchdog/mod.rs:2027-2045` | test helpers `put_key`, `empty_buffer` | used by the R-m2 test |
| `src/hook/watchdog/mod.rs:2127-2145` | test `a_batchs_key_codes_do_not_outlive_its_count` | **R-m2 red-first** (`3117b0a`) |
| `src/hook/watchdog/mod.rs:2188-2190` | test helper `records` | used by the R-m2 test |
| `src/hook/watchdog/mod.rs:2376-2393` | test `a_wait_is_rounded_up_to_whole_milliseconds_and_stays_below_infinite` | T2 (tester, `995892f`) |
| `src/hook/watchdog/mod.rs:2725-2854` | scan tests | `the_watchdog_injects_no_input`; `no_key_data_leaves_the_raw_input_read`, whose allowed fns are `raw_key_event`, `count_raw_keys` and `qualifies` (`:2833-2835`) |
| `src/hook/watchdog/decision.rs:66-70` | `SILENCE` 1 s, `TICK` 250 ms, `ACTIVE_FOR` 2 s | the arithmetic behind the `errors.md:215` wording |
| `src/hook/watchdog/decision.rs:171-201` | `Watchdog::tick`, `kept = if backlog { credit - explained } else { 0 }` `:182` | the line that mutation M10 changes; pinned by T1 |
| `src/hook/watchdog/decision.rs:284-360` | test `Feeder` (`backlog` `:296`, `request_times` `:343`) | harness of T1 |
| `src/hook/watchdog/decision.rs:754-787` | test `credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them` | T1 (tester, `7300822`) |
| `src/bench.rs:75-82` | docs for row R16 | L2: "A declared row, measured and reported, not gated (ruling L2)" (`4d6b228`) |
| `src/bench.rs:579-606` | `r16_first_key_of_a_burst` | unchanged this round |
| `src/bench.rs:1128-1132` | row R16, `r16_first_key_of_a_burst(1_000)` | n = 1000 (V3-M2) |
| `README.md:939` | the "Hook watchdog" bullet | the R16 sentence (L2) |
| `book/src/config/errors.md:215` | the stop line `[INFO] hook watchdog stopped after N wakes for M raw key events` | R-m1 wording (`1db14ca`) |
| `src/logging/sink.rs:118-131` | line timestamp: "UTC as ISO 8601 with milliseconds" | M6 part B compares t_raw from `daemon.log` in UTC ms |

## Facts

### R-m2 (the buffer clear)
1. `RawBuffer::take(&mut self, written: u32) -> Read` (`src/hook/watchdog/mod.rs:869-882`) maps the return value of `GetRawInputBuffer`:
   - `0` gives `Read::Empty`;
   - `u32::MAX` gives `Read::Failed`;
   - any other count gives `Read::Records(count_raw_keys(&self.0, count))`.
   - It then calls `self.0.fill(0)` when `written != 0` (`:877-879`).
2. On `62cf30a`, `take` exists with the same `match` and no clear. That commit also moved the `match` out of `KeyboardReads::buffered` (diff `959fce1..HEAD`, hunk `@@ -1083,18 +1110,14 @@`).
3. `take` has no caller in master: `src/hook/watchdog/mod.rs` does not exist on `master` (`git show master:src/hook/watchdog/mod.rs` fails). The method is absent on `f51dc7d` (0 matches for `fn take(&mut self, written`) and present on `62cf30a` (1 match).
4. The only production caller of `take` is `KeyboardReads::buffered` (`mod.rs:1120`), right after the `GetRawInputBuffer` call (`:1113-1119`). The only other callers are the test's (`:2139`, `:2142`), and `SyntheticReads::buffered` delegates to `standard()` (`:1185-1187`).
5. `KeyboardReads::buffered` passes `RAW_BUFFER_BYTES as u32` as the size on every call (`mod.rs:1107`). The buffer pointer is `self.buffer.0.as_mut_ptr()` (`:1115`).
6. The buffer is built once per watchdog run (`mod.rs:665`) and passed by `&mut` to `run_loop` (`:672`). From there it goes through `drain_keyboard` (`:1161`) into `KeyboardReads` (`:1165-1170`).
7. The standard read's record is a local `RAWINPUT::default()` in `raw_key_event` (`mod.rs:820`).
8. The R-m2 test calls `take` directly on a hand-written buffer (`mod.rs:2133-2144`). No test path calls the real `GetRawInputBuffer` with records. The OQ3 smoke test reads an empty queue (`the_buffered_read_of_a_thread_without_raw_input_reads_nothing`, per `review_r2.md:83`, which gives line `:2319` at `959fce1`).
9. The "# No key data" docs (`mod.rs:51-58`) say: "The buffered read's buffer is cleared as soon as its batch is counted, and the standard read's record is a local of its call, so no key code outlives the count of its batch."
10. The production clear runs on the watchdog thread inside the drain. The key path does not call it. `fix_r2.md` "Gates" estimates the cost as "3 KB `fill(0)` ... about 4 times a second while keys are typed". That figure is UNVERIFIED by a bench: the bench was not re-run after `fef60f9`.
11. `fix_r2.md` "Unsafe / atomics added": none. The diff adds no `unsafe` block. The existing SAFETY comment at `mod.rs:1107-1112` is unchanged.

### L2 docs (R16 declared)
12. `src/hook/watchdog/mod.rs:34-42` says:
    - the wake and its pass run on another core;
    - a callback that runs at the same time "is about 50 us slower at the median ... (bench row R16 against R12, a declared row, measured and not gated: ruling L2)";
    - "That is the worst order";
    - Q6 is not documented, and "manual check M6 settles it".
13. `README.md:939` says:
    - "A keyboard callback that runs during that wake is about 50 microseconds slower at the median than one that does not (bench row R16 against R12, on the developer's machine): at most once per burst of keys, and only in the worst order";
    - if raw input comes after the hook call, "the key's own callback never meets the wake".
14. `src/bench.rs:79-82` says: "A declared row, measured and reported, not gated (ruling L2): it times the worst order of cut Q6, a key's raw input before its hook call, and the wake happens once per burst of keys."
15. `cut.md:1000` (L1.6, row R16):
    - "declared evidence row: measured and reported, not gated";
    - the tester's runs: p50 54.6 / 52.7 / 46.1 us, p99 107.6 / 101.2 / 121.9 us, against R12 p50 4.7 us;
    - the developer's run: 53.5 / 168.4 us;
    - the withdrawn bound is kept "As first written".
16. `cut.md:998` (row R14): gated on p50 <= R14c p50 + 300 ns. p99 is reported as the largest of 3 runs against R14c's largest of 3 (ruling L2, `rulings.md` round 4).
17. `rulings.md` round 4 (L2): R16 is "measured and reported, not gated". Q6 goes to the owner as manual check M6. FU-4 goes to roadmap 3.5. R-m1 and R-m2 are fixed in this lane.
18. FU-4 is at `impl.md:152`. The M6 summary is at `impl.md:161`, and the full M6 text is in `fix_r2.md`, "Manual check M6 (amended)", and `cut.md:627`.

### R-m1 (the stop-line wording)
19. `book/src/config/errors.md:215` now says:
    - "about 10 wakes for an isolated shortcut ... and about 4 a second for longer typing";
    - "100 presses of Ctrl+C ... give M = 400 and N of about 1000";
    - "holding a key for 20 s adds about 600 to M ... and about 90 to N".
20. The constants behind that text: `TICK` = 250 ms (`decision.rs:68`) and `ACTIVE_FOR` = 2 s (`decision.rs:70`).
21. The stop line is logged at `mod.rs:678-682` from `stats.wakes` and `stats.raw_keys`. Those are incremented per pass at `mod.rs:240-241`.

### M6 part B dependencies
22. `log::info!("hook watchdog: raw keyboard input arrives")` is at `mod.rs:249`. It sits under `if !self.raw_input_seen` (`:247`) inside `if raw_keys > 0 && context.health.hook_installed()` (`:246`). It is not under any `cfg`.
23. The log line timestamp is UTC ISO 8601 with milliseconds (`src/logging/sink.rs:118`, `:127`). The test is `timestamps_are_iso_8601_utc_with_milliseconds` (`sink.rs:310`).
24. The debug hold is under `#[cfg(debug_assertions)]`:
    - `src/hook/mod.rs:3`;
    - `src/hook/keyboard_hook_callback.rs:383`;
    - `src/core/app.rs:239`.
    - The env var is `KS_DEBUG_HOOK_HOLD` (`src/hook/debug_hold.rs:23`).

### Tester tests of round 2 (mutation pins, already on the branch)
25. T1 `credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them` (`decision.rs:754-787`): it feeds 1 + 1 000 + 4 000 + 7 000 raw events and 1 + 10 000 callbacks. It asserts exactly one request, in 1250..=1500 ms (`:779-786`).
26. T2 `a_wait_is_rounded_up_...` (`mod.rs:2376-2393`) checks `wait_ms`:
    - 0 for `ZERO`;
    - 1 for 1 us;
    - 2 for 1 001 us;
    - 250 for 250 ms;
    - `INFINITE - 1` for 10 years.
27. T3 `an_early_timer_signal_leaves_the_ticks_where_they_were` (`mod.rs:1543-1557`): `requests == [1000]` with the timer on time, and the same with `early = 10`.

## Red-first tests and acceptance proofs

| # | Test / proof | Command | Must be red on | Expected red | Green on |
|---|---|---|---|---|---|
| RF1 | `hook::watchdog::tests::a_batchs_key_codes_do_not_outlive_its_count` (R-m2) | `cargo test --locked --lib a_batchs_key_codes_do_not_outlive_its_count` | `3117b0a` (the red commit; its parent `62cf30a` adds the seam `RawBuffer::take` that master and `f51dc7d` lack, so the test does not compile before `62cf30a`) | panic at `src/hook/watchdog/mod.rs:2126:9`, `assertion 'left == right' failed: bytes of a counted batch are left`, `left: 12`, `right: 0` | `fef60f9` and HEAD |
| RF1-mut | the mutation "clear only after records" (`if written != 0` becomes a clear only in the `count` arm) | the same command, with the mutation applied | the mutated `fef60f9` | `bytes of a failed read are left`, `left: 4` (developer, informal, `fix_r2.md` R-m2) | unmutated |
| T1 | `decision::tests::credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them` | `cargo test --locked --lib hook::watchdog` | not red-first: it pins mutation M10 (`decision.rs:182` `kept = credit - explained` becomes `kept = credit`) | `one request for the 2 000 extra events: [] left: 0 right: 1` (`test_r2.md:59`) | HEAD |
| T2 | `tests::a_wait_is_rounded_up_to_whole_milliseconds_and_stays_below_infinite` | the same | not red-first: it pins mutation M11 (`mod.rs:777` `div_ceil(1_000_000)` becomes `/ 1_000_000`) | fails on `a fraction of a millisecond waits a whole one` (`test_r2.md:60`) | HEAD |
| T3 | `tests::an_early_timer_signal_leaves_the_ticks_where_they_were` | the same | not red-first: it pins mutation M12 (the tick check in `pass`, `is_some_and(\|due\| due <= now)` becomes `is_some()`) | `the timer 10 ms early left: [1200] right: [1000]` (`test_r2.md:61`) | HEAD |
| A-L2 | R16 is a declared row with measured figures | read `cut.md:1000`, `README.md:939`, `mod.rs:33-42`, `bench.rs:79-82` | n/a (docs) | n/a | HEAD |
| A-m1 | the stop-line wording | read `book/src/config/errors.md:215` against `decision.rs:66-70` | n/a (docs; `fix_r2.md`: "no test can be red for it") | n/a | HEAD |
| A-bench | latency bench (ignored test `bench::latency_bench`) | as `bench_run_r2.sh`: `KS_BENCH_OUT=<md> KS_BENCH_LABEL=<label> CARGO_TARGET_DIR=<dir> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`, each tree in its own worktree and target dir | n/a | gated: R14 p50 <= R14c p50 + 300 ns (`cut.md:998`). R16 is not gated (L2). | last run on `959fce1`; not re-run after `62cf30a` / `fef60f9` |
| A-gate | the three gates | `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked` | n/a | the developer reports fmt and clippy pass on HEAD and on each commit, and `hook::watchdog` 74 passed. The full suite was not run by the developer. | HEAD |

Scout verification (this pack):
- `cargo test --locked --lib hook::watchdog` on HEAD `4d6b228`: `running 74 tests`, `test result: ok. 74 passed; 0 failed; 0 ignored ... 731 filtered out`. Log: `D:/tmp/ks-orch/p2-2.4-watchdog/scout_r3_watchdog.log`. Target dir: `D:/tmp/ks-orch/p2-2.4-watchdog/target-scout-r3`.
- RF1 on `3117b0a`:
  - I extracted the commit with `git archive 3117b0a` into `D:/tmp/ks-orch/p2-2.4-watchdog/scout-r3-red` and built with the fresh target dir `target-scout-r3-red`.
  - Result: FAILED, at `mod.rs:2126:9`, `bytes of a counted batch are left`, `left: 12`, `right: 0`. Log: `D:/tmp/ks-orch/p2-2.4-watchdog/scout_r3_red.log`.
- A pitfall in that run: a `git archive` extract built with a target dir that already holds a newer build of HEAD reported a false green ("Finished ... in 0.06s", 1 passed). Cargo kept the HEAD binary because `git archive` gives the files the commit's mtime. A red-first replay needs a fresh `CARGO_TARGET_DIR` or a worktree.
- Not re-run by the scout: the full `cargo test --locked`, clippy, fmt, the bench, the RF1 mutation, and the M10-M12 mutations.

## Invariants in play

- **R1 (hook callback)**: no file of the hook path changed this round. `keyboard_hook_callback.rs` is not in `git diff --stat 959fce1..HEAD`.
- **R2 (sentinel and injection)**: the scan test `the_watchdog_injects_no_input` is at `mod.rs:2767`. Its allowed-site list is at `:2795-2801`, which covers `reinstall_keyboard_hook` (`keyboard_hook.rs`) and `reinstall_if_silent` (`main_thread.rs`). The round adds no `SendInput`.
- **R3 (shared state)**: `RawBuffer` is owned by the watchdog thread's run. It is passed by `&mut` (`mod.rs:665`, `:672`) and is not in a static.
- **Rulings S1-S4 (no real hook or raw input registration under test)**: the round adds no installer or registration call. The R-m2 test touches no Win32 call.
- **The spec's "No key data from Raw Input is kept or logged"**: `mod.rs:51-58`, `:869-882`, and the scan `no_key_data_leaves_the_raw_input_read` (`mod.rs:2825-2850`, allowed fns `:2833-2835`). `take` names no key field itself; it calls `count_raw_keys`.
- **The latency owner decision, L1/L2**: R14 is gated on p50 (`cut.md:998`), and R16 is declared and not gated (`cut.md:1000`).
- **KS-08**: the lane's issue. This round does not change the reinstall path.
- **R6 (Display)**: no new enum variant. `Read` is unchanged (`mod.rs:958-969`).

## Open points (from `triage_r2.md`, with their state after `fix_r2.md`)

1. **T-F1 = R-I1 (R16 misses its sanity bound)**: CONFIRMED in triage_r2. Ruling L2 resolved it by declaring R16 ungated. `fix_r2.md` carries it out in docs only (`cut.md:1000`, `README.md:939`, `mod.rs:33-42`, `bench.rs:79-82`). No test pins it.
2. **R-m1 (stop-line wording)**: NOTED in triage_r2. Changed at `errors.md:215` (`1db14ca`).
3. **R-m2 (key codes outlive their batch)**: NOTED (PLAUSIBLE) in triage_r2. Fixed at `mod.rs:869-882` (`fef60f9`) and pinned by RF1.
4. **The tester's R14 p99 note**: not triaged. L2 says the paced rows gate on p50 and report p99 as the largest of 3 (`cut.md:998`).
5. **Review OQ2 (two M9 rules)**: still open for the orchestrator (`fix_r2.md` Open points 2). `cut.md:1108` (L1.12, the M9 row; `review_r2.md:129` cited it as `:1092` at `959fce1`) says `K >= 150` and `W <= 130`. `cut.md:762` (the addendum) says `K >= 400` and `W <= K / 2`. `fix_r1.md:46`/`:109` say `K >= 400` and `W <= K / 2`.
6. **Review OQ3 (E4: does a pending `WM_INPUT` outside the `QS_RAWINPUT` mask leave the wait asleep)**: still open. Only manual check M9 settles it.
7. **Review OQ4 (`KEYEVENTF_UNICODE` `wScan` reported as `RAWKEYBOARD.MakeCode`?)**: still open as a research or manual question. `fix_r2.md` says its weight is lower since the clear.
8. **M6 part B**: void if any key event comes before F9 in the debug run (`fix_r2.md` Open points 1).
9. **Manual checks (owner)**: M6 (amended), M9, M2 (amended), M1 and M10 need a live daemon and are not tests (`impl.md:161`, `fix_r1.md:106-111`, `test_r2.md:98`).

## Not covered

- I did not read the other 23 files of the lane diff against master (installer, `main_thread.rs`, `health.rs`, IPC, the callback, `app.rs`). Earlier rounds covered them in `pack_r1.md` and `pack_r2.md`.
- I did not read `run_loop` (`mod.rs:694-755`), `drain` (`:1014-1075`), `next_wait` (`:1076-1095`) or `Ticker::pass` beyond `:233-256`, apart from the line numbers cited.
- I checked the full M6 text in `fix_r2.md` and `cut.md:627` against the code only for facts 22-24, not step by step.
- `CHANGELOG.md` did not change this round (`fix_r2.md` "Docs"). I did not re-check its watchdog entry (`CHANGELOG.md:179-215`).
- The book was not rebuilt by the scout. `fix_r2.md` reports a build with 0 broken links into `D:/tmp/ks-orch/p2-2.4-watchdog/book`.
