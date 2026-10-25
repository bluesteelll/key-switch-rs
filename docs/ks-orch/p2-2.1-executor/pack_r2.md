# Context pack r2: p2-2.1-executor (roadmap 2.1, KS-08)

Scout read the lane tree `D:/key-switch-rs/key-switch-rs`, branch `feat/action-executor`, HEAD `a875960`, tree clean.
`master` = `7d757a1` = merge base. Round-1 pack (whole-lane map): `D:/tmp/ks-orch/p2-2.1-executor/pack_r1.md`.
Inputs: `fix_r1.md`, `triage_r1.md`, `test_r1.md`, `review_r1.md`, `rulings.md`, `spec.md` (all in the lane directory).

## Scope and delta

`git log --oneline 6a1db71..feat/action-executor`:

| Commit | Subject | Author role (trailer) | Files |
|---|---|---|---|
| `c94acd5` | Tests: a job still queued when the executor dies is counted | tester (Sonnet 5.5) | `src/core/executor/mod.rs` +46 |
| `2e51c2a` | Tests: for_action queues no window for Launch and OpenUrl, and bump_if_err ignores an Ok | tester (Sonnet 5.5) | `mod.rs` +22, `src/logging/counters.rs` +13 |
| `a0a8b82` | Tests: the executor arms its timer for the nearest of several waiting deadlines | tester (Sonnet 5.5) | `src/core/executor/runs.rs` +24 |
| `a875960` | Tests: the COM apartment tests accept the process's main STA | developer (Opus 5.5) | `mod.rs` +16 -7 |

`git diff --stat 6a1db71..feat/action-executor`:
```
 src/core/executor/mod.rs  | 91 +++++++++++++++++++++++++++++++++++++++++++----
 src/core/executor/runs.rs | 24 +++++++++++++
 src/logging/counters.rs   | 13 +++++++
 3 files changed, 121 insertions(+), 7 deletions(-)
```
Every hunk of the delta lies inside a `#[cfg(test)] mod tests` block: `mod.rs` tests start at `:687-688`
(the 6a1db71 hunk header is `@@ -691`), `runs.rs` tests at `:216`, `counters.rs` tests at `:202`. No production line changed
in round 2.

## File map

| path:lines | what lives there | why it matters to round 2 |
|---|---|---|
| `src/core/executor/mod.rs:70-100` | `TargetWindow`, `NONE` (`:75`), `foreground` (`:78-83`), `for_action` (`:89-95`) | code under M22 test |
| `src/core/executor/mod.rs:189-204` | `Shared::stopping`/`live`; `ExecutorHandle` fields `jobs` (`:202`), `thread_id` (`:203`), `shared` (`:204`) | the M18 test reaches these private fields directly |
| `src/core/executor/mod.rs:215-260` | `submit` match (`:215-234`), `cancel_pending` (`:240-243`), `wake` (`:253-260`) | review OQ (wake to a dead executor id) |
| `src/core/executor/mod.rs:317-319` | `pub fn thread(&self) -> &Thread`, no `///` line | review m1 (open) |
| `src/core/executor/mod.rs:322-350` | `is_finished`, `stop`, `stop_thread` | used by the M18 test (`:1272`, `:1284`) |
| `src/core/executor/mod.rs:356-397` | `struct Apartment` (`:360-363`), `enter_sta` (`:370-386`), `Drop` (`:388-397`, `if self.initialized` at `:390`) | production code the F1 tests probe; M17 mutates `:390` |
| `src/core/executor/mod.rs:527-575` | `run_executor`: prologue, `catch_unwind` around `run_loop` (`:552-560`), epilogue (`:562-574`), Died drain `:565-571` | code under M18 test |
| `src/core/executor/mod.rs:590-685` | `run_loop`; `End::Died` on `GetMessageW == -1` (`:654-660`) and `== 0` / `WM_QUIT` (`:661-668`) | path the M18 test drives |
| `src/core/executor/mod.rs:688-745` | test imports (`APTTYPE_MAINSTA` at `:694-696`), helpers `start` (`:726-730`), `take_counts` (`:733-738`), `count_of` (`:740-745`) | F1 import; helpers of new tests |
| `src/core/executor/mod.rs:1241-1285` | new test `a_job_still_queued_when_the_executor_dies_is_counted` | M18 proof (`c94acd5`) |
| `src/core/executor/mod.rs:1287-1307` | new test `launch_and_open_url_carry_no_window_and_ask_for_none` | M22 proof (`2e51c2a`) |
| `src/core/executor/mod.rs:1326-1343` | `apartment_type` (`:1327-1335`), new `is_sta` (doc `:1337-1340`, fn `:1341-1343`) | F1 fix |
| `src/core/executor/mod.rs:1345-1387` | `ApartmentProbe` (`:1346-1354`), `the_executor_runs_jobs_in_a_single_threaded_apartment` (`:1356-1369`), `the_apartment_guard_uninitialises_com_on_drop` (`:1371-1387`) | F1 tests, changed asserts `:1367`, `:1382-1386` |
| `src/core/executor/mod.rs:1389-1425` | `macros_never_sleep_or_block_in_a_wait` scan: bans `thread::sleep` / `MsgWaitForMultipleObjects` in non-test lines of `mod.rs`, `runs.rs`, `data/sequence.rs` | the M18 test's `thread::sleep` (`:1254`) is below the tests marker |
| `src/core/executor/mod.rs:1427-1440` | ignored manual `the_win32_runner_opens_a_url` | the one ignored test in `core::executor` |
| `src/core/executor/runs.rs:58` / `:148-157` | `enum Next`; `Runs::next` with `.min()` at `:155` | code under M26 test |
| `src/core/executor/runs.rs:266-348` | `Fixture` (fake clock `at` `:294-296`, `ms` `:289-291`, `poll` `:326-342`, `taken` `:345-347`) | harness of the M26 test |
| `src/core/executor/runs.rs:392-414` | new test `the_nearest_deadline_of_several_waiting_runs_is_next` | M26 proof (`a0a8b82`) |
| `src/core/executor/testing.rs:157-175` | `leaked_counters` (`:158-160`), `next_record` (`:163-167`), `wait_until` (`:170-175`, polls with a 1 ms sleep, 2 s `WAIT`) | per-test counters; wait helper |
| `src/logging/counters.rs:36,50,63,107` | `Counter::ExecutorUnavailable` variant, `ALL` entry, `Display` arm `"executor_unavailable"`, slot index 5 | counted by the M18 test |
| `src/logging/counters.rs:118-126` / `:192-194` | `Counters::bump_if_err` (`is_err` at `:123`); free fn `bump_if_err` | code under M21 test |
| `src/logging/counters.rs:287-301` / `:303-314` | older `bump_if_err_counts_only_an_error`; new `bump_if_err_ignores_an_ok_result` | M21 proof (`2e51c2a`) |

## Facts

1. `a875960` changes only test code: hunks at `mod.rs:694-696` (import), `:1337-1343` (`is_sta`), `:1367`, `:1382-1386`.
2. At `a0a8b82` the two COM asserts were `assert_eq!(kind, Some(APTTYPE_STA))` at `mod.rs:1357` and `assert_eq!(inside, Some(APTTYPE_STA))` at `:1372`, then `assert_ne!(after, Some(APTTYPE_STA), ..)` at `:1373` (`git show a0a8b82:src/core/executor/mod.rs`).
3. At HEAD `is_sta(kind)` is true for `Some(APTTYPE_STA)` or `Some(APTTYPE_MAINSTA)` (`mod.rs:1341-1343`); the executor test asserts `is_sta(kind)` (`:1367`); the guard test asserts `is_sta(inside)` (`:1382`) and `!is_sta(after)` (`:1383-1386`); each message prints the observed `Option<APTTYPE>`.
4. The COM tests exist only on this branch; at `6a1db71` they sat at `mod.rs:1279` and `:1294` (`git show 6a1db71:...`). `master` has no `src/core/executor` (the module is new in this lane, `pack_r1.md`).
5. `Apartment::enter_sta` calls `CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)` and stores `initialized: hr.is_ok()` (`mod.rs:375,381-384`); `Drop` calls `CoUninitialize` only `if self.initialized` (`:388-397`). Unchanged in round 2.
6. `run_executor` owns `timer` and `runs` outside the `catch_unwind` closure (`mod.rs:547-551`, closure `:552-560`); after the loop it kills the timer (`:562`), stores `active_runs = 0` (`:563`), and on `End::Died` drains the queue with `try_recv`, bumping `ExecutorUnavailable` once per job (`:565-571`).
7. `run_loop` returns `End::Died` when `GetMessageW` returns `-1` (`mod.rs:654-660`) or `0` (`:661-668`), each after one `log::error!`.
8. The M18 test submits `open("first")`, waits for its record, sleeps 200 ms (`mod.rs:1254`), then `try_send`s a `Job` with the current epoch directly on `handle.jobs` (no `wake`), posts `WM_QUIT` to `handle.thread_id` (`:1270`, with a `// SAFETY:` comment at `:1267-1269`), waits for `is_finished` (`:1272`), and asserts `ExecutorUnavailable == 1` (`:1274-1279`), no further record (`:1280-1283`), `stop() == Stopped::Joined` (`:1284`).
9. The M18 test uses per-test counters: `start` takes `leaked_counters()` (`mod.rs:726-730`, `testing.rs:158-160`).
10. `stop_thread` returns `Stopped::Joined` at once when `join` was already taken (`mod.rs:334-336`); otherwise it sets `stopping`, posts a wake and waits `STOP_TIMEOUT` (2 s, `:62`) on `done`.
11. `TargetWindow::for_action` returns `foreground()` only for `BindAction::Sequence(_)` and `NONE` otherwise (`mod.rs:89-95`); `foreground` calls `GetForegroundWindow` (`:78-83`).
12. The M22 test asserts `for_action(Launch{..}) == NONE` and `for_action(OpenUrl(..)) == NONE` (`mod.rs:1293-1306`); its comment (`:1289-1292`) states it checks nothing where no window has the foreground.
13. `Runs::next` takes the `.min()` of `Resume::At` deadlines and maps none to `Next::Idle` (`runs.rs:148-157`).
14. The M26 test starts three runs with `Delay(200)`, `Delay(50)`, `Delay(900)` in that order (`runs.rs:395-397`), asserts `Next::At(ms(50))` (`:399-404`), moves the fake clock to 50 (`:406`), asserts `taken() == ["key F14"]` (`:408`) and `Next::At(ms(200))` (`:409-413`).
15. `Counters::bump_if_err` bumps only `if result.is_err()` (`counters.rs:122-126`). The older test bumps one `Ok` and one `Err` and expects 1 (`:288-301`); the new test bumps one `Ok` and expects a sum of 0 (`:304-314`).
16. `core::executor` holds 34 `#[test]` fns (`mod.rs` 25, `runs.rs` 9, `testing.rs` 0), one of them `#[ignore = "manual: ..."]` (`mod.rs:1428`).
17. `ci.yml` runs `cargo test --locked` on `windows-latest` with the default thread count (`.github/workflows/ci.yml:27,38-39`).
18. The `WM_QUIT` and `WAKE` (`WM_APP`, `mod.rs:65`) posts in tests target only the test's own executor thread id (`mod.rs:1270`).
19. Line drift in the round-1 reports vs HEAD: triage cites `keyboard_hook_callback.rs:707` for `the_keyboard_callback_records_how_long_it_ran`; at HEAD it is `src/hook/keyboard_hook_callback.rs:691`. Triage cites the M18 test as `mod.rs:1240-1283`; at HEAD `#[test]` is `:1241`, the fn `:1242-1285`. fix_r1 cites `is_sta` as `:1337-1343`; the doc comment is `:1337-1340`, the fn `:1341-1343`.
20. `logging::init` starts the writer over the global `COUNTERS` (`src/logging/mod.rs:112`); the writer's periodic report calls `self.counters.take()` (`src/logging/writer.rs:225`).
21. `AttachedExecutor` (session-long holder of the `Executor`) is `src/core/app.rs:29-63`, attached at `:205`.

## Red-first tests and acceptance proofs (from fix_r1.md and test_r1.md)

Method recorded by the tester (`test_r1.md`, "Red-first replay"): a scratch worktree
(`git worktree add --detach D:/tmp/ks-orch/p2-2.1-executor/wt/<name> <sha>`), its own `CARGO_TARGET_DIR`
under `D:/tmp/ks-orch/p2-2.1-executor/target-wt/`, each test run with `--exact`, every run printing `running 1 test`;
worktree removed afterwards. Not in the lane tree.

### F1 (developer commit `a875960`): red on its parent `a0a8b82` (master lacks the seam: no executor module)

| Test | Command | Expected at `a0a8b82` | Expected at `a875960` |
|---|---|---|---|
| `core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment` | `cargo test --locked --lib core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment -- --exact` | red: panic at `mod.rs:1357:9`, `left: Some(APTTYPE(3)), right: Some(APTTYPE(0))` (fix_r1; test_r1 saw it fail 3 of 3) | `1 passed` |
| `core::executor::tests::the_apartment_guard_uninitialises_com_on_drop` | same, with that test path | red: panic at `mod.rs:1372:9`, the same assertion, left `Some(APTTYPE(3))` | `1 passed` |
| whole module, serial | `cargo test --locked --lib core::executor -- --test-threads=1` | `31 passed; 2 failed` (test_r1 F1) | `33 passed; 0 failed; 1 ignored` (fix_r1) |
| whole module, default threads | `cargo test --locked --lib core::executor` | order-dependent (test_r1: `--test-threads=2` failed 2 of 3) | `33 passed; 0 failed; 1 ignored` (fix_r1) |
| M17 mutation (temporary `if self.initialized && false` at `mod.rs:390`) | guard test alone with `--exact` | at `6a1db71`/`a0a8b82` red for the wrong reason (first assert, MAINSTA) | red at `mod.rs:1383:9`: "after the guard the thread has no apartment of its own (or the implicit MTA), not Some(APTTYPE(3))" (fix_r1) |

### F2 (tester commits): red only under the named mutation; on each parent commit the code is correct and the test does not exist

| Mutation | Site | Test (commit) | Under mutation | Without |
|---|---|---|---|---|
| M18 Died epilogue does not drain/count | `mod.rs:565-571` | `core::executor::tests::a_job_still_queued_when_the_executor_dies_is_counted` (`c94acd5`) | `FAILED` (0.20 s, count 0 != 1) | passes |
| M21 `is_err` -> `is_ok` | `counters.rs:123` | `logging::counters::tests::bump_if_err_ignores_an_ok_result` (`2e51c2a`) | fails (sum 1) | passes |
| M22 `Launch` reads the foreground | `mod.rs:89-95` | `core::executor::tests::launch_and_open_url_carry_no_window_and_ask_for_none` (`2e51c2a`) | fails where a foreground window exists | passes |
| M26 `.min()` -> `.max()` | `runs.rs:155` | `core::executor::runs::tests::the_nearest_deadline_of_several_waiting_runs_is_next` (`a0a8b82`) | fails (`Next::At(900)`) | passes |

Command form for each: `cargo test --locked --lib <test path> -- --exact`. The mutations are temporary edits in a scratch
tree only (fix_r1 restored its M17 edit byte for byte and re-checked `git diff`).

### Whole-suite numbers on record

- `a0a8b82`, default threads (`cargo test --locked --no-fail-fast`): lib `running 591 tests` / `586 passed; 0 failed; 5 ignored`; `tests/ipc_roundtrip.rs` `6 passed`; swch and doc-tests `running 0 tests` (test_r1).
- `a0a8b82`, `cargo test --locked --lib -- --test-threads=1`: `584 passed; 2 failed` (test_r1, F1).
- `a875960` adds no test fn and removes none (diff, Fact 1), so the same counts are expected: `586 passed; 0 failed; 5 ignored` lib, default and `--test-threads=1`. UNVERIFIED: fix_r1 ran only `core::executor` and the two F1 tests; the full suite is left to the tester.
- Gate at `a875960` per fix_r1: `cargo fmt --check` pass; `cargo clippy --all-targets --locked -- -D warnings` pass.
- The five ignored tests (none to be run): `core::executor::tests::the_win32_runner_opens_a_url`, `data::condition::tests::foreground_context_names_the_focused_notepad`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (test_r1).

### Round-1 red-first table still on record (unchanged code; test_r1 "Red-first replay")

| Test | Red commit (run on its parent) | Parent result |
|---|---|---|
| `config::loader::tests::mask_only_on_a_hotkey_a_binding_takes_over_warns_nothing` | `fbd6b59` | red (assert); also red on master |
| `config::loader::tests::an_empty_config_warning_is_returned_not_logged` | `47f2b88` | red (assert) |
| `logging::tests::a_second_init_starts_no_writer` | `7827ec8` | red (assert, `logging/mod.rs:238`); seam `start_logging` from `89fcf5e` |
| `core::executor::tests::a_job_queued_while_a_step_pumps_messages_runs_without_another_wake` | `a364b77` | red (assert, "did not run within 1 s"); V2-I1 |
| `data::sequence::tests::a_window_step_never_waits_for_its_window` | `fd5993e` | red (assert, lists `sequence.rs:277/280/283`) |

## Open points (triage_r1.md)

| Item | Triage verdict | State at HEAD |
|---|---|---|
| F1 COM tests assume `APTTYPE_STA` | CONFIRMED (test defect) | addressed by `a875960` (Facts 1-3) |
| F2/M18, M21, M22, M26 | REFUTED (closed by `c94acd5`, `2e51c2a`, `a0a8b82`) | tests at `mod.rs:1241-1285`, `counters.rs:303-314`, `mod.rs:1287-1307`, `runs.rs:392-414` |
| M22 vacuity where `GetForegroundWindow` returns null | NOTED | comment at `mod.rs:1289-1292` |
| review m1: `pub fn thread` has no `///` | NOTED, not confirmed as a defect; fix_r1: no change | still undocumented at `mod.rs:317-319` |
| review OQ: wakes to the id of an executor that died mid-session | NOTED; `wake` ignores a failed post (`mod.rs:253-260`); `AttachedExecutor` keeps the `JoinHandle` open (`app.rs:29-63`); rulings accept R-7 until 2.2 (`rulings.md`, last bullet of "Critique, Minor and open questions") | no change |
| tester note: process-wide `COUNTERS` in `the_keyboard_callback_records_how_long_it_ran` | NOTED (writers only `fetch_max`; only the `logging::init` writer resets it, `logging/mod.rs:112`) | test at `keyboard_hook_callback.rs:691`; no change |
| fix_r1 "Open points / pre-existing warnings" | none | - |

## Invariants in play

- Test isolation: per-test `Counters` through `leaked_counters` (`testing.rs:158-160`); `the_keyboard_callback_records_how_long_it_ran` uses global `COUNTERS` (`keyboard_hook_callback.rs:691`).
- Safety rules for tests: the only Win32 posts added target the test's own executor thread (`mod.rs:1270`); `for_action` test calls `GetForegroundWindow` only through `for_action(Sequence)`, which the test never passes (`mod.rs:1293-1306`).
- CLAUDE.md principle 4 (SAFETY comments): the new `unsafe` block at `mod.rs:1270` has its comment at `:1267-1269`.
- Commit trailers: tester commits carry `Claude Sonnet 5.5`, `a875960` carries `Claude Opus 5.5` (`git log --format=%(trailers)`).
- X6 sleep/wait scan (`mod.rs:1389-1425`) covers only lines above each file's `#[cfg(test)] mod tests` marker.
- R10 / KS-01 (executor thread owns its timer and queue; Died epilogue): `mod.rs:547-574`.
- R4: `epoch` loaded with `Acquire` in the M18 test (`mod.rs:1258`, `handle.shared.epoch.load(Ordering::Acquire)`), bumped with `AcqRel` in `cancel_pending` (`:241`).

## Not covered

- No cargo command was run by the scout (no gate, no tests, no mutations); every result above is quoted from `fix_r1.md` / `test_r1.md`.
- Production code outside the executor module, the hook callback, `app.rs` beyond `:29-63,:205`, docs/book/README/CHANGELOG were not re-read in round 2 (no round-2 change touches them; see `pack_r1.md`).
- `review_r1.md` was read only for m1 and the open question; `critique*.md` and `cut.md` were not re-read.
- The scratch probe output in `D:/tmp/ks-orch/p2-2.1-executor/triage-r1/probe` and the tester logs in `t1/` were not opened.
