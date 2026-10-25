# Code review: p2-2.4-watchdog (round 3) @ 06ae0d7
APPROVED (Critical 0, Important 0, Minor 0)

Scope: `git diff 959fce1..feat/hook-watchdog`, read through `git show`/`git diff` only.
- The pack was written for HEAD `4d6b228`. While I worked, the branch moved to `06ae0d7`, the tester's round-3 commit "Tests: the buffered read clears through RawBuffer::take and over the whole buffer" (`mod tests` only, +46 lines). I reviewed it too, so this verdict is pinned to `06ae0d7`. Its production code is the same as `4d6b228`'s.
- Delta `959fce1..06ae0d7`: 5 files, +177/-16.
  - The only production change is `RawBuffer::take` (`src/hook/watchdog/mod.rs:869-882`) and its one caller (`:1120`).
  - Everything else is `mod tests` (`decision.rs:755`; `mod.rs:1544`, `:2128`, `:2377`, `:2856`, `:2880`) or docs (`README.md:939`, `book/src/config/errors.md:215`, `src/bench.rs:79-82`, `mod.rs:31-58`).
- I read these in full at `06ae0d7`:
  - `RawBuffer`/`take`, `count_raw_keys`, `drain`, `next_wait`, `KeyboardReads`, `drain_keyboard`, `SyntheticReads`, `run_loop`, `Ticker::pass`;
  - `decision.rs` production code, `Watchdog::tick`, the `Feeder` and `Drive` harnesses;
  - the scan helpers (`non_test_lines`, `fn_lines`) and the no-key-data scan;
  - the CHANGELOG watchdog and bench entries.
- I checked the work against `spec.md`, `rulings.md` (rounds 1-4, L2 above all), `triage_r2.md`, `fix_r2.md` and `cut.md` (L1.6 row R16 at `:1000`, M6 at `:627`, the addendum at `:783-796`).

## Gate: fmt ✅ · clippy ✅
- Tree: `git archive 06ae0d7`, exported to `D:/tmp/ks-orch/p2-2.4-watchdog/review-src-r3b`, which the tester cannot touch. I also checked `4d6b228` in `review-src-r3`, and it passed too. `CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.4-watchdog/target-review`.
- `cargo fmt --check`: exit 0, no output (`review-fmt-r3.log`).
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 (`review-clippy-r3.log`). I ran it a second time after touching `src/lib.rs` and `src/hook/watchdog/mod.rs`. The log shows `Checking key-switch-rs v0.1.0 (...review-src-r3b)`, so the crate was really checked, not replayed from the cache.
- I did not run `cargo test`; the tester owns it. The scout's logs show the following:
  - `hook::watchdog`: 74 passed at `4d6b228` (`scout_r3_watchdog.log:119`).
  - RF1 was red on `3117b0a` for the right reason (`scout_r3_red.log:50-53`, `bytes of a counted batch are left`, left 12). That is 3 records x 4 non-zero bytes each: dwType, dwSize, MakeCode and VKey.

## Round-2 items (triage_r2.md)
- **T-F1 = R-I1** (R16 misses its sanity bound): ✅ closed, as ruling L2 asks. It is a docs-only change, and no test pins it, because the bench row is `#[ignore]`d and has no assertion. Each place now gives the measured figure, says "once per burst" and "the worst order", and names no planned number:
  - `cut.md:1000` makes R16 a "declared evidence row: measured and reported, not gated". It keeps the withdrawn bound "as first written".
  - `README.md:939`.
  - `mod.rs:33-42`.
  - `bench.rs:79-82`.
  - `git grep` finds no planned 20 us / 25 us figure left in `README.md`, `CHANGELOG.md`, `book/src` or `src`.
  - Q6 went to the owner as M6 (`cut.md:627`, `impl.md:161`, the full text in `fix_r2.md`). FU-4 is at `impl.md:152`.
  - The paced rows' p99 rule is restated at `cut.md:998`.
- **R-m1** (stop-line wording): ✅ closed (`errors.md:215`). I checked the arithmetic against `decision.rs:68,70` and `keeps_ticking` (`:227-234`):
  - an isolated shortcut costs 1 wake plus about 9 ticks to `last_raw_at + ACTIVE_FOR`, so about 10;
  - 100 x Ctrl+C gives M = 400 and N about 1000;
  - a 20 s hold at about 30 repeats/s gives M about 600, and 80 + 1 + about 9 gives N about 90.
- **R-m2** (key codes outlive their batch): ✅ closed.
  - `take` (`mod.rs:869-882`) keeps the old `0 / u32::MAX / count` mapping exactly. It calls `count_raw_keys` before it clears, and clears the whole buffer after any read that is not empty.
  - The clear cannot be removed as a dead store while the thread runs. Every later `buffered()` hands the same pointer to `GetRawInputBuffer` (`:1113-1119`), an external call that may read it.
  - The red/green pair is `3117b0a`/`fef60f9`. The tester's `06ae0d7` adds two pins: the only buffered read answers through `take` (`:2856`), and the clear reaches the last byte (`:2880`).

## Remarks
### 🔴 Critical
None.
### 🟡 Important
None.
### 🟢 Minor
None.

## Checklist walked (absent items, no reachable failure)
- **S1-S4 / no test touches the system**:
  - The delta adds no call to an installer, `RegisterRawInputDevices`, `SendInput`, `logging::init` or `attach_file`. Grepping the added lines for `unsafe|Ordering::|SendInput|dwExtraInfo|log::|unwrap(|expect(|panic!|SetWindowsHook|RegisterRawInput|logging::init|attach_file|println|eprintln` finds only the renamed binding `let written = unsafe {` (`:1113`).
  - The six new tests are pure:
    - a `Feeder` on a fake clock;
    - `wait_ms`;
    - `Drive` on a `FakeTimer` with leaked `HealthState` and `Counters` (M4);
    - `take` on a hand-written buffer;
    - a source scan via `CARGO_MANIFEST_DIR`.
- **R1 hook callback / budget**: `keyboard_hook_callback.rs`, `keyboard_hook.rs`, `main_thread.rs`, `src/core` and `src/engine` have no diff in `959fce1..06ae0d7`. Nothing new runs on the key path or on main.
- **Key data in logs**: no log call was added. `take` names no key field; it calls `count_raw_keys`. The scan `no_key_data_leaves_the_raw_input_read` (`:2825`) still allows only `raw_key_event`, `count_raw_keys` and `qualifies`. `RawBuffer` has no `Debug`, `Clone` or `Copy`, so no implicit copy of the bytes exists. It is built once (`:665`) and passed by `&mut`.
- **R2 sentinel**: no new `SendInput` path. `the_watchdog_injects_no_input` is unchanged.
- **R3 shared state**: no new statics. The buffer is owned by the watchdog thread's stack frame.
- **R4 orderings**: no atomics added or changed.
- **R5-R9**: not touched. M6's `reg query` is a read.
- **R10 loop and shutdown**: unchanged. The quit path's last drain (`run_loop`, `source.drain(...)` after `read_at_stop`) also goes through `take`, so the final batch is cleared too.
- **Threads, timers, COM, start/stop order**: no new thread, window, timer or apartment this round.
- **Unsafe**: no new block. The existing `GetRawInputBuffer` SAFETY comment (`:1107-1112`) still holds: the buffer is 8-aligned, `size` is reset to `RAW_BUFFER_BYTES` before each call, and the header size is `RAWINPUTHEADER`'s.
- **Behaviour**: `take` returns what the old `match` returned for every input. The clear runs after the count, so a mutation that clears first is caught: `records(3, 0)` would fail.
- **Standard read's local** (considered, not a remark):
  - `raw` at `:820` leaves at most one 48-byte record in a dead stack slot after `raw_key_event` returns. On x64 that happens only on the fallback paths; a 32-bit build uses it for every read.
  - `DispatchMessageW` is called from the same frame right after (`:1141`), and that slot is not live program state. This is the residue review_r2 m2 already called the pre-L1 baseline, and ruling L2 targets the read buffer.
- **Tests can fail**:
  - T1 (`decision.rs:755`): I traced it at the ticks 250/500/750 (credit 9000, then 5000, then 2000 unexplained), giving a request at 1250. Under the mutation `kept = credit` the credit stays 9000 and no request comes, so the test is red.
  - T3 (`mod.rs:1544`): I traced it. The first raw event at 0 gives the ticks 250..1000, and evidence 9 at 250 gives a request at exactly 1000. The early timer only adds no-tick passes at 240-249.
  - T2: the values match `div_ceil` and the `INFINITE - 1` clamp.
  - The `06ae0d7` tests: 76 x 40 bytes = 3040 <= 3072, and the marker byte at 3071 is outside every record. The scan's `fn_lines` header matches the `impl` line and its braces balance.
- **Rulings**: L2 (R16 declared, Q6 to M6, FU-4 to 3.5, the p99 rule, R-m1, R-m2) is carried out and recorded in the addendum (`cut.md:783-796`). I read the M6 part A and B logic against the code and found it sound:
  - the first-arrival line is under `raw_keys > 0 && hook_installed()` (`:246-249`), in every build;
  - the debug hold is `cfg(debug_assertions)` only;
  - mouse clicks are not counted, because only the keyboard is registered.
- **Docs and CHANGELOG**:
  - The README and the module docs agree with `cut.md:1000` and `:1117`.
  - The CHANGELOG's "keeps counts and times, never key data" is now better supported. It needs no change, since nothing a user can see changed.
  - No config meaning changed.
- **Commits**: each has one subject in the `<Area>: <what>` form. The developer's commits end with `Co-Authored-By: Claude Opus 5.5`, the tester's (`7300822`, `995892f`, `f51dc7d`, `06ae0d7`) with `Claude Sonnet 5.5`. None was amended.

## Positive
- R-m2 is fixed at one seam (`take`), and two kinds of test pin it:
  - behaviour: a counted batch, a failed read and a full buffer;
  - structure: the only buffered read answers through `take`.
- The second kind closes the gap that no test can feed `GetRawInputBuffer` real records. The seam commit `62cf30a` made the red an assertion failure, not a compile error.
- The clear covers the failed read too, where Learn does not say what the buffer holds. The empty read skips it, so a drain's closing `Empty` read costs nothing.
- The errors-page row now gives arithmetic that a user can check against their own stop line.

## Open questions
1. **A bench re-run cannot see this round's production change.**
   - R14 and R16 run the watchdog on `Source::Synthetic` (`bench.rs:505`, `:592`; `mod.rs:410`), and `SyntheticReads::buffered` delegates to `standard()`, so `take` and its 3 KB `fill(0)` never run in the bench.
   - The clear runs only on the watchdog thread, on a buffer that thread owns, so it shares no cache line with the callback.
   - The `959fce1` bench numbers therefore still stand for the key path. Whether to re-run is the tester's and orchestrator's call, and a re-run would add no evidence about `fef60f9`.
2. Carried over, unchanged, and outside this round's code: review_r2 OQ2 (hand the owner only `fix_r1.md`'s M9 rule), OQ3 (E4, which only M9 settles), and OQ4 (`KEYEVENTF_UNICODE` `wScan` as MakeCode). OQ4 now matters less, because the buffer is cleared after each batch.
