# Tester: p2-2.4-watchdog / roadmap 2.4 (KS-08), round 3 (delta since 959fce1)

HEAD when tested: `4d6b228` (tree clean). The tester added two `Tests:` commits, so the branch HEAD is now `17ca0de` (test code only; production code identical to `4d6b228`).

Verdict: GREEN. Two of my own mutations survived the round-2 tests (M3, M4) plus one more (M6); each is now caught by a `Tests:` test I added and showed red under the mutation, green without it.

## Gate (HEAD)
- `cargo fmt --check`: OK (on `4d6b228` and after each Tests commit).
- `cargo clippy --all-targets --locked -- -D warnings`: OK (`Finished dev profile`, after `touch src/lib.rs`; again after each Tests commit).
- `cargo test --locked --no-fail-fast` on `4d6b228`: lib `running 805 tests`, `798 passed; 0 failed; 7 ignored`; integration `running 7 tests`, `7 passed; 0 failed; 0 ignored`; two `running 0 tests` targets (bins, no tests). Log `D:/tmp/ks-orch/p2-2.4-watchdog/r3/full_test.log`.
- `cargo test --locked --no-fail-fast` on `17ca0de` (with my tests): lib `running 808 tests`, `801 passed; 0 failed; 7 ignored`; `7 passed` integration; no failure.
- `cargo build --release --locked`: OK (`Finished release profile`).
- Ignored (7 in lib, counted separately, not passes): `bench::latency_bench` (bench), `core::executor::tests::the_win32_runner_opens_a_url`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it`, `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (all `manual: ...`). I never ran `--ignored`, except the bench by its own documented command.

## Red-first replay (CONFIRMED / fixed items of triage_r2 + the round's regression test)
| Test | Parent | HEAD | Note |
|---|---|---|---|
| `hook::watchdog::tests::a_batchs_key_codes_do_not_outlive_its_count` (R-m2) | `3117b0a` (fresh worktree + fresh `CARGO_TARGET_DIR`): `running 1 test`, `FAILED`, `mod.rs:2126:9 assertion left == right failed: bytes of a counted batch are left, left: 12, right: 0`; `test result: FAILED. 0 passed; 1 failed; ... 804 filtered out` | `running 1 test`, `test result: ok. 1 passed; 0 failed` | red by assertion (not compile-red); the seam commit `62cf30a` precedes it |

- T-F1 = R-I1 (R16) and R-m1: docs-only items (ruling L2). No test can be red for them. Checked by reading: `README.md:939`, `errors.md:215`, `bench.rs:75-82`, `mod.rs` "# Cost" all state R16 as a declared row, about 50 us at p50, once per burst, worst order, and the stop line as about 10 wakes per isolated shortcut / 100 Ctrl+C = M 400, N about 1000. Round-3 R16 measured 47.7 us (median of 3 p50s) and agrees with "about 50".
- Worktree `3117b0a` removed; `git worktree list` shows only the lane tree.

## Mutations (scratch worktree at the tested commit, `CARGO_TARGET_DIR=r3/target-wt/mut`; every one reverted, `git diff --exit-code` clean after each)
| # | File | Mutation | Named test (before running) | Result |
|---|---|---|---|---|
| M1 | mod.rs `RawBuffer::take` | clear runs before the count | `a_batchs_key_codes_do_not_outlive_its_count` | caught (that test FAILED, 73 passed 1 failed) |
| M2 | mod.rs `take` | failed read (`u32::MAX`) no longer cleared | `a_batchs_key_codes_do_not_outlive_its_count` (failed-read half) | caught |
| M3 | mod.rs `KeyboardReads::buffered` | bypass `take`: inline `match` + `count_raw_keys`, no clear | none known | **NOT CAUGHT on 4d6b228** (`74 passed; 0 failed`). Added `the_buffered_read_answers_through_the_method_that_clears_its_buffer` (commit `06ae0d7`); on `06ae0d7` the mutation FAILS that test (75 passed, 1 failed) |
| M4 | mod.rs `take` | `self.0[..RAW_BUFFER_BYTES / 2].fill(0)` (partial clear) | `a_batchs_key_codes_...` (3 records at offset 0) | **NOT CAUGHT on 4d6b228** (`74 passed; 0 failed`). Added `a_counted_batch_leaves_no_byte_of_the_whole_buffer` (76 key records + last byte 0xFF); on `06ae0d7` FAILS it |
| M5 | decision.rs:182 | `if backlog \|\| true` (backlog ignored) | a V3-I1 backlog test | caught by `a_hook_that_dies_after_a_burst_asks_for_exactly_one_reinstall` and `a_hook_that_dies_after_long_drift_is_found_within_its_window` |
| M6 | mod.rs `wait_ms` | cap `ms.min(INFINITE - 1)` -> `ms.min(INFINITE)` | `a_wait_is_rounded_up_to_whole_milliseconds_and_stays_below_infinite` (T2) | **NOT CAUGHT** (`76 passed; 0 failed`): T2's 10-year case fails `u32::try_from` and never reaches the cap. Added `a_wait_of_exactly_the_largest_finite_timeout_is_not_infinite` (commit `17ca0de`); on `17ca0de` the mutation FAILS it (76 passed, 1 failed) |
| M7 | mod.rs `Ticker::pass` | `due <= now` -> `due < now` | tick-boundary tests | caught by 7 tests incl. `an_early_timer_signal_leaves_the_ticks_where_they_were` (T3) |
| M8 | mod.rs `take` | `0 => Read::Failed` | `the_buffered_read_of_a_thread_without_raw_input_reads_nothing` | caught |
| M9 | decision.rs:182 | `kept = credit` (the round-2 M10) | T1 `credit_matched_to_raw_events_is_not_kept_for_the_ones_after_them` | caught |

## Acceptance (delta items)
| Criterion | Evidence | Status |
|---|---|---|
| No key code outlives its batch (R-m2 / spec "no key data is kept") | `a_batchs_key_codes_do_not_outlive_its_count`, `a_counted_batch_leaves_no_byte_of_the_whole_buffer`, `the_buffered_read_answers_through_the_method_that_clears_its_buffer` (all ok), plus scan `no_key_data_leaves_the_raw_input_read` ok | verified (the production call into the real `GetRawInputBuffer` with records is not testable without raw input: manual smoke) |
| R16 stated as a declared, ungated row (L2) | docs read: README.md:939, mod.rs "# Cost", bench.rs R16 docs; bench R16 47.7 us | verified by reading (no test applies) |
| Stop-line wording for isolated bursts (R-m1) | errors.md:215 matches `TICK` 250 ms / `ACTIVE_FOR` 2 s | verified by reading |
| M6, M9, M2, M1, M10 | live daemon needed | manual, not verified |

## Latency bench (watchdog drain changed; 3 runs each, 959fce1 vs 17ca0de whose production code is 4d6b228; own worktrees and target dirs; files `r3/bench-r3-{prev,head}-{1,2,3}.md`)
Median of 3 p50 / largest p99 (us):

| Row | prev p50 / p99 | head p50 / p99 |
|---|---|---|
| R4 | 4.7 / 6.7 | 4.8 / 6.7 |
| R5 | 4.8 / 30.4 | 4.9 / 5.4 |
| R6/50 | 301.8 / 581.3 | 324.4 / 9051.4 (one outlier run, n = 10; p50 gate is met) |
| R7 | 10.6 / 42.8 | 10.5 / 19.8 |
| R9-R12 | 4.7, 5.0, 5.3, 4.7 | 4.7, 5.1, 5.3 (witness R11, vs 5.4 us ok), 4.7 |
| R13 | 10.4 / 20.8 | 10.3 / 19.6 |
| R14 / R14c | 4.8 / 48.5 ; 4.8 / 10.9 | 4.8 / 14.8 ; 4.8 / 26.5 |
| R15 | 3.9 / 4.8 | 3.9 / 6.5 |
| R16 (declared, not gated) | 49.2 / 124.8 | 47.7 / 145.8 |
R1-R3: 0.1 / 0.2 / 0.1 on both. Rules (D10, D14, L2): R14 p50 4.8 <= R14c 4.8 + 0.3: ok; R4, R5, R6 (p50 <= 1 ms and <= prev + 0.5 ms), R7 all within bounds; no breach. All six runs exited 0.

## Failures
None open. The three survivors M3, M4, M6 were test gaps, not production bugs; closed below.

## Tests added by the tester (`Tests:` commits on feat/hook-watchdog)
- `06ae0d7` Tests: the buffered read clears through RawBuffer::take and over the whole buffer. In `src/hook/watchdog/mod.rs`: `the_buffered_read_answers_through_the_method_that_clears_its_buffer`, `a_counted_batch_leaves_no_byte_of_the_whole_buffer`.
- `17ca0de` Tests: a wait of exactly u32::MAX milliseconds is not INFINITE. In `src/hook/watchdog/mod.rs`: `a_wait_of_exactly_the_largest_finite_timeout_is_not_infinite`.
Both contain test code only; fmt, clippy and the full `cargo test --locked` pass on `17ca0de`. Tree is clean, `git worktree list` shows only the lane tree.

## Notes
- The scan test in `06ae0d7` is string-based (`.take(` present and `count_raw_keys(` absent inside `impl RawReads for KeyboardReads`); it fails if the impl is renamed (`fn_lines` panics) rather than passing.
- Open questions the orchestrator still owns (not test matters): review OQ2 (two M9 rules), OQ3 (E4), OQ4 (KEYEVENTF_UNICODE wScan).
- The R6/50 9 ms p99 in one head run is a single outlier of 10 samples (that run's p50 is 324 us); the gated p50 holds.
