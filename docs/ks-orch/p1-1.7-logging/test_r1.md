# Tester: p1-1.7-logging / roadmap 1.7 file logging (HEAD 9b6414d vs developer HEAD bb9c80f, master 1058ddb)

Verdict: GREEN. One gap found by my own mutation (M5, the `size > 0` guard of `LogFiles::write_line`) was closed with a `Tests:` commit (9b6414d) that is red under the mutation and green without it.

HEAD is `9b6414d` (developer's `bb9c80f` + my `Tests:` commit). Tree clean (`git status --short` empty). Gate was run on `bb9c80f` first, then fully on `9b6414d`.

## Gate (HEAD 9b6414d)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (sources touched first so the fingerprints were not stale; `Checking key-switch-rs ... Finished` in 2.86 s)
- cargo test --locked --no-fail-fast: 533 + 6 = 539 passed, 0 failed, 4 ignored
  - `running 537 tests` / `test result: ok. 533 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 1.84s`
  - `running 0 tests` (swch bin, no unit tests) / `running 6 tests` (tests/ipc_roundtrip.rs) `test result: ok. 6 passed; 0 failed; 0 ignored` / `running 0 tests` (doc-tests, none exist)
  - On bb9c80f: 536 run, `532 passed; 0 failed; 4 ignored`. The difference is my one added test.
- cargo build --release --locked: OK (`Finished release profile`); the daemon was never run.
- Ignored (4, all pre-existing "manual:" tests, not run, not counted as passes):
  - `data::condition::tests::foreground_context_names_the_focused_notepad`
  - `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`
  - `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`
  - `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`
- No new `#[ignore]` in this lane (diff shows none).
- Flake check: `logging::` (39 tests) and `hook::config_watcher` (10 tests) run 5 times each, 10/10 `ok`. The writer tests that use a real clock (flush, report period) were stable.

## Red-first replay
Per rulings N4 the reds of C3 and C6 replay on their parent plus the test (`848e95c` = `95c9b3a` + test, `eebace9` = `783f383` + test, per commit stats). Own `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.7-logging/target-wt/c3`.

| Test | Red tree | Green trees | Note |
|---|---|---|---|
| `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp` | `848e95c`: `running 1 test`, FAILED at `config_watcher.rs:370:32` `a reload error must reach the log; captured: []`; `test result: FAILED. 0 passed; 1 failed; ... 534 filtered out` | `ef3f656`: `1 passed`; HEAD 9b6414d: `test result: ok. 1 passed; 0 failed; ... 535 filtered out` | red by assertion (the record never reaches the logger, the old code printed directly), matches commit body |
| `logging::tests::only_the_startup_banner_prints_outside_src_bin` | `eebace9`: `running 1 test`, FAILED at `mod.rs:337:9` `27 print sites outside the startup banner:` (27 sites listed: app.rs x8, windows_actions.rs x3, sequence.rs x5, ipc_server.rs x4, keyboard_hook.rs x1, system_function.rs x6) | `0dc008e`: `1 passed`; HEAD: `test result: ok. 1 passed; ... 535 filtered out` | red by assertion, count 27 as predicted |
| `system::known_folder::tests::local_app_data_is_an_existing_absolute_folder` (C2) | compile-red on master (module and function do not exist; declared in impl.md:29-31, N4: new capability) | HEAD: `running 1 test ... ok`, `1 passed` | carried by mutation M4 (below): returning a relative path fails it on its `is_absolute` assertion |

C1 module tests (sink, writer, counters, logging) have no red on master by N4 (new module); they are carried by mutations M1-M3, M5-M10, M13, M14 below. C5 has no red by design (needs a live short `SendInput` or a spawn failure); its proof is the scan test (M11b) plus the review in Acceptance.

Worktrees: `D:/tmp/ks-orch/p1-1.7-logging/wt/c3` and `.../wt/mut` removed with `git worktree remove --force` + `git worktree prune`. `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 9b6414d [feat/file-logging]`. `ls .../wt` is empty. Target dirs `target-wt/c3` and `target-wt/mut` stay outside the repo.

## Mutations
Scratch worktree at HEAD, `CARGO_TARGET_DIR=.../target-wt/mut`, never the lane tree. Test named before the run; each reverted with `git checkout -- .` and `git diff --exit-code` clean (printed `reverted-clean`).

| # | File:line | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| M1 | sink.rs:223 | `> self.limit` -> `>= self.limit` | `a_line_that_reaches_the_limit_exactly_does_not_rotate` | caught (also `daemon_log_rotates_at_one_mebibyte`), 2 failed of 17 |
| M2 | sink.rs:252 | `(1..KEPT_FILES)` -> `(1..=KEPT_FILES)` (4 files kept) | `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest` | caught, 1 failed of 17 |
| M3 | sink.rs:78 | continuation lines without the timestamp prefix (`if index > 0` -> `if false`) | `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` and `a_reload_error_is_logged_with_a_timestamp` | caught by both |
| M4 | known_folder.rs | `local_app_data` returns `PathBuf::from("LocalAppData")` | `local_app_data_is_an_existing_absolute_folder` | caught (also covers the C2 compile-red) |
| M5 | sink.rs:223 | drop the `size > 0 &&` guard (an empty daemon.log rotates, pushing an empty file over daemon.1.log) | `a_line_longer_than_the_limit_goes_into_an_empty_file` (by name) | **NOT CAUGHT**: `test result: ok. 17 passed`. That test writes `short` first, so daemon.log is never empty when the long line arrives. Gap closed with my test below: red under M5 (`an empty daemon.log is not pushed down over daemon.1.log; left: "" right: "keep\n"`), green on HEAD |
| M6 | writer.rs:70 | full queue no longer bumps `LogLinesDropped` | `a_full_queue_drops_the_line_and_counts_it` | caught |
| M7 | writer.rs:212 | drop `if !self.failing` (a failing file reports on every line) | `a_failing_log_file_is_reported_once_on_the_console` | caught |
| M8 | sink.rs:104 | `FATAL` tag mapping disabled | `a_fatal_record_is_tagged_fatal` | caught (also `an_error_with_the_fatal_target_is_tagged_fatal`) |
| M10 | sink.rs:54 | console text starts at the timestamp (`console_from = 0`) | `the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console` | caught (12 failed) |
| M11a | ipc_server.rs | extra `println!` outside src/bin | `only_the_startup_banner_prints_outside_src_bin` | caught |
| M11b | keyboard_hook_callback.rs `start_hold_timer` | `log::info!` added in the hook callback module | `only_the_startup_banner_prints_outside_src_bin` (N3 check) | caught |
| M12 | config_watcher.rs:166 | reload error logged as `warn` instead of `error` | `a_reload_error_is_logged_with_a_timestamp` (asserts `[ERROR] config reload failed:`) | caught |
| M13 | sink.rs:61 | trailing `\n` no longer trimmed | `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` (the trailing-`\n` case) | caught, 1 failed of 39 |
| M14 | sink.rs:256 | `rotate` continues after a failed rename (`return Err` -> `continue`) | `a_rotation_stops_at_the_first_rename_that_fails` | caught (also `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log`) |

14 mutations, 13 caught by a named test, 1 (M5) not caught and then covered by an added test.

## Acceptance
| Criterion | Evidence | Status |
|---|---|---|
| Sink unit tests write to a TEMP dir, pin rotation at the size limit and the file count | All sink/writer disk tests use `crate::config::testing::TempDir`. `logging::sink` 18 tests ok: `a_line_that_would_pass_the_limit_rotates_first`, `a_line_that_reaches_the_limit_exactly_does_not_rotate`, `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest`, `a_line_longer_than_the_limit_goes_into_an_empty_file`, `daemon_log_rotates_at_one_mebibyte` (real 1 MiB constant), `the_limit_is_one_mebibyte_and_three_files_are_kept`, M1/M2/M14 catch the limit, count and rename logic. `rg "attach_file\(\|logging::init\(\|local_app_data\("` shows only `main.rs` calls plus the read-only `known_folder` test: no test attaches the real folder | verified |
| Each line has an ISO-8601 timestamp and the level; folder created on demand | `timestamps_are_iso_8601_utc_with_milliseconds`, `a_line_is_the_timestamp_the_level_and_the_message`, `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` (M3, M13), `log_macros_reach_the_logger_with_a_timestamp`; `the_first_line_creates_the_folder_and_daemon_log`, `a_deleted_folder_is_created_again_on_the_next_line` | verified |
| Console output on stderr stays | `writer::tests::the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console` (M10), `lines_before_the_file_is_attached_reach_the_console_only`; `init` passes `io::stderr()` as the console (`mod.rs:109`) | verified (the real stderr write itself is manual M1/M7) |
| A reload error is logged with a timestamp | `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp`: red on `848e95c`, ok on HEAD; every physical line timestamped and `[ERROR]`, bindings stay live (M3, M12) | verified |
| `rg "println!\|eprintln!" src --glob '!src/bin/**'` finds only the startup banner | Run on HEAD: `src\core\app.rs:105,106,109,112,114,116`, all inside `print_welcome`. The scan test `only_the_startup_banner_prints_outside_src_bin` holds the same rule (red on `eebace9`, M11a) | verified |
| No key names or text in any log call on the hook path (review) | `rg -n "log::\|print" src/hook/keyboard_hook_callback.rs` is empty and the scan test rejects `log::` there (M11b). I read every `log::` call in `windows_actions.rs`, `system_function.rs`, `keyboard_hook.rs`, `sequence.rs`: messages carry OS error values, counts (`SendInput sent {sent} of {n}`), and configured action data (`Post message {msg:#X}`, `Launch {exe:?}`, `OpenUrl {target:?}`, `exe.display()`); no vk, no key name, no typed text (`press_key` no longer names the key). The hook thread only calls `counters::bump` (one `AcqRel` RMW). Note: the scan test covers the callback module only; `windows_actions.rs`/`system_function.rs` rely on this review | verified by review (not by a test for those two files) |
| swch unchanged | `git diff master...feat/file-logging -- src/bin/swch.rs` is empty (0 bytes) | verified |
| One new dependency, `log` only | `git diff master...feat/file-logging -- Cargo.toml Cargo.lock`: `+log = "0.4"`, `+"Win32_System_Com"` (feature for the known-folder lookup, cut-approved), Cargo.lock `+ "log",` (one line, `log` was already in the tree through `notify`) | verified |
| README, book build-and-run page, CHANGELOG updated | `README.md:746-794` (`### Log file`), `book/src/build-and-run.md:114,141+` (Log file section), `CHANGELOG.md:89-118` | verified by reading the hits; prose and links not reviewed (reviewer's job); developer reports mdbook ok, 2191 links |
| Real daemon behaviour: first start creates folder and start line; second start exits 2 with no prompt and no new file line; `missing.ron` writes `[FATAL]` before Enter; no typed word in the files; Launch failure line; conhost QuickEdit held selection (with `master` control); Scheduled Task reload error | manual checks M1-M8 of impl.md:180-191 | manual, not verified |

## Failures
None open. One finding, closed:

### F1 (closed). `LogFiles::write_line` `size > 0` guard was untested (M5)
`logging::sink::tests::a_line_longer_than_the_limit_goes_into_an_empty_file` passes with the guard removed, because it writes `short\n` first. With the guard gone a line longer than the limit arriving at an empty `daemon.log` (a fresh folder, or right after a rotation) renames the empty file over `daemon.1.log` and loses its history. Test added (below).

## Tests added by the tester (`Tests:` commits)
- `9b6414d` `Tests: an empty daemon.log is not rotated by a line longer than the limit (1.7)`, test code only, `src/logging/sink.rs` (+26 lines in `#[cfg(test)] mod tests`), trailer `Co-Authored-By: Claude Sonnet 5.5`. Test: `logging::sink::tests::a_line_longer_than_the_limit_does_not_rotate_an_empty_daemon_log`. HEAD: `running 1 test ... ok`; with the `size > 0 &&` guard removed in the scratch tree: `FAILED`, `an empty daemon.log is not pushed down over daemon.1.log; left: "" right: "keep\n"`. The test writes only to a `TempDir`. After the commit: fmt ok, clippy ok, full suite as above, `git status --short` empty.

## Notes
- The branch now has 10 commits over master (9 developer commits + my `Tests:` commit); it is not pushed.
- Pre-existing and out of scope here (impl.md open points): `unsafe` in `console_ctrl_handler` (`app.rs:248`) has no `// SAFETY:` comment; the `print_welcome` banner contains non-ASCII characters present on master.
- `src/core/app.rs:317,332` tests use the string `\\.\pipe\key-switch-rs.sock` only to format an error message (pre-existing on master); they bind nothing. No test in the diff binds the production pipe or writes a real `config.ron`/`%LOCALAPPDATA%`.
- Mutation worktree and the C3/C6 worktree were created and removed under `D:/tmp/ks-orch/p1-1.7-logging/wt`; the lane tree was edited only for the one committed test.
