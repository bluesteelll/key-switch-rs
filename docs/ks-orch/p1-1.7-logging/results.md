# Results analysis: p1-1.7-logging / roadmap item 1.7 File logging

## VERDICT: ACCEPT
**Summary**: Every acceptance line of roadmap 1.7 and of the lane spec is carried by a named test or a listed
manual check, every ruling in `rulings.md` is in the tree, and my own gate run is clean (fmt, clippy on a fresh
target dir, 533 + 6 tests, 4 pre-existing manual ignores). The hook callback gained only two atomic counter
bumps, and the per-keystroke path has no new I/O, lock or allocation. The `log`-only dependency and in-house sink
follow the owner decision. The two open review items are Minor (doc and idempotence of a second `init()`), and
neither is reachable in the daemon.

Branch `feat/file-logging` @ `9b6414d` (10 commits on `master` = `1058ddb`, merge-base = master, no merges). Tree clean.

## Quality gate (own run)
| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0; `analyst-fmt.txt`) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK, run twice: the lane's `target` (exit 0) and a fresh `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.7-logging/analyst-target` (every crate re-checked, 0 warnings, exit 0; `analyst-clippy-fresh.txt`) |
| `cargo test --locked` | lib `running 537 tests` -> `533 passed; 0 failed; 4 ignored`; `ipc_roundtrip` 6 passed; swch 0; doctests 0 (`analyst-test.txt`). Total 543 vs master 502 (1.5 results: 496 lib + 6); `#[test]` lines 493 -> 534 (+41) |
| Ignored (`--list --ignored`, not run) | 4, all pre-existing `manual:` (`condition`, `ipc_server::hold_the_daemon_pipe_name_for_a_manual_start`, `keyboard_hook_callback::the_menu_mask_key_is_inserted_into_the_input_stream`, `registry::live_registry_...`). None new; the ipc_server reason was extended (N5), not weakened |
| `cargo doc --no-deps` | 4 warnings, all pre-existing lines untouched by the lane (`loader.rs:156,176`, `keyboard_hook.rs:132`, `vk_name.rs:4`); 0 in `src/logging/*` or `known_folder.rs` |
| `mdbook build book -d D:/tmp/ks-orch/p1-1.7-logging/analyst-book` | OK; own link/anchor check over the HTML: 2411 internal links, 0 broken (`#log-file`, `#17-file-logging`, `#24-hook-watchdog`, `#33-...`, `#41-...` present) |
| Acceptance rg `rg "println!\|eprintln!" src --glob '!src/bin/**'` | exactly `src\core\app.rs:105,106,109,112,114,116`, all inside `fn print_welcome` (`app.rs:103-117`) |
| Trailers | `grep -cE 'Co-Authored-By: Claude (Opus\|Sonnet) 5.5'` = 10 of 10 commits (9 Opus developer, `9b6414d` Sonnet tester) |
| Real log folder | `%LOCALAPPDATA%\key-switch-rs` does not exist after the tester's and my full suite runs, so no test touched it |
| cfgcheck harness (I2, parse only, no hook) | rebuilt from `cfgcheck/` (its `main.rs` = the methodology.md listing, `diff` clean). `dup.ron`: `[INFO] loading config from dup.ron`, `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; ...`, `OK: 3 bindings`, exit 0 (the `[WARN]` is on stderr: `2>/dev/null` leaves only `OK:`). `example.ron`: `OK: 26 bindings`, exit 0. An invalid file: loader `[INFO]` then `ERR: ...`, exit 1. Missing file: exit 2 |

## Acceptance criteria
| # | Criterion (roadmap 1.7 + lane spec) | Evidence | Status |
|---|---|---|---|
| A1 | Sink unit tests write to a TEMP dir (never real `%LOCALAPPDATA%`) and pin rotation at the size limit and the file count | `logging::sink::tests::daemon_log_rotates_at_one_mebibyte` (`sink.rs:533`, real `MAX_FILE_BYTES`), `a_line_that_would_pass_the_limit_rotates_first` (`:438`), `a_line_that_reaches_the_limit_exactly_does_not_rotate` (`:454`, `>` vs `>=`), `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest` (`:467`, exact names, no `daemon.3.log`), `the_limit_is_one_mebibyte_and_three_files_are_kept` (`:561`), `a_line_longer_than_the_limit_does_not_rotate_an_empty_daemon_log` (`:507`, tester), `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log` (`:568`), `a_rotation_stops_at_the_first_rename_that_fails` (`:601`, M1). All under `config::testing::TempDir`; `rg "attach_file\(\|logging::init\(\|local_app_data\("` finds only `main.rs`, the definitions and the read-only `known_folder` test. Tester mutations M1, M2, M5, M14 caught | PASS |
| A2 | A reload error is logged with a timestamp | `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp` (`config_watcher.rs:351`): three physical lines each `is_timestamped` + `[ERROR] `, then `[INFO] keeping previous bindings live`, 1 binding still live. Red on `848e95c` (`captured: []`, tester replay), green from `ef3f656`. Live run: manual M2 = Checkpoint 1 smoke #8 | PASS |
| A3 | `rg "println!\|eprintln!" src --glob '!src/bin/**'` finds only the startup banner | My rg run (above) + gate test `logging::tests::only_the_startup_banner_prints_outside_src_bin` (`mod.rs:269`), red on `eebace9` with the predicted 27 sites | PASS |
| A4 | No key names or text in any log call on the hook path (review) | Own read of all 44 `log::` call sites. The ones reachable from `carry_out` on the hook thread (`windows_actions.rs:238,254,280`, `system_function.rs:58,77,89,99,105,114`) carry config text (msg id, exe, URL, path) or Win32/io errors. The vk-naming sites became counters (`windows_actions.rs:223-225`, `system_function.rs:155-157`). `sequence.rs:190` dropped the vk, and `type_text` logs counts only. Registry/loader lines run only at load/reload (`BindAction::get_system_combination` has one non-test caller, `loader.rs:223`). The banner is not logged. Gate check: the scan test rejects any `log::` path in `keyboard_hook_callback.rs` before its tests (N3, tester M11b). Live: manual M5 | PASS (review + gate + manual) |
| S1 | `%LOCALAPPDATA%\key-switch-rs\daemon.log`, folder created on demand | `the_log_folder_is_key_switch_rs_under_local_app_data` (`mod.rs:198`), `local_app_data_is_an_existing_absolute_folder` (`known_folder.rs:51`, read only, no COM init, Q6), `the_first_line_creates_the_folder_and_daemon_log` (`sink.rs:414`), `a_deleted_folder_is_created_again_on_the_next_line` (`sink.rs:630`). `main.rs` wiring (no harness, KS-38): manual M1 | PASS |
| S2 | Each line has an ISO-8601 timestamp and the level | `timestamps_are_iso_8601_utc_with_milliseconds` (`sink.rs:310`, leap days 2000/2100), `a_line_is_the_timestamp_the_level_and_the_message` (`:331`), `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` (`:375`, I1), `an_error_with_the_fatal_target_is_tagged_fatal` (`:348`), `log_macros_reach_the_logger_with_a_timestamp` (`mod.rs:162`) | PASS |
| S3 | Rotation at 1 MB keeping `daemon.log`, `daemon.1.log`, `daemon.2.log` | as A1 (1 MB = 1 MiB, documented in `sink.rs:18-20`, README, book) | PASS |
| S4 | Console output on stderr stays | `the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console` (`writer.rs:284`), `lines_before_the_file_is_attached_reach_the_console_only` (`writer.rs:305`); `init` hands `io::stderr()` to the writer (`mod.rs:109`). Real console: manual M1 | PASS |
| S5 | Replace print macros outside `src/bin` with log macros, banner excepted | A3; the only direct stderr writes left are `main.rs:53` (the `[FATAL]` fallback when the writer is stuck) and `main.rs:58` (the prompt, Q2), plus the logger's own fallback (`mod.rs:97`) | PASS |
| S6 | The hook callback never logs per event; atomic counters logged elsewhere | `keyboard_hook_callback.rs:262-264,300-302` only `counters::bump`; counters tests (`counters.rs:116-153`), `a_flush_reports_non_zero_counters_before_it_returns` (`writer.rs:337`), `the_writer_reports_counters_every_period` (`writer.rs:359`), `a_full_queue_drops_the_line_and_counts_it` (`writer.rs:442`); scan test (N3) | PASS |
| S7 | Pressed keys and typed text are never logged | A4 + manual M5 (`Select-String` for a typed word over the three files) | PASS (review + manual) |
| S8 | `swch` unchanged | `git diff master...feat/file-logging -- src/bin` empty | PASS |
| S9 | `log` is the one new dependency; no tracing | `Cargo.toml` `+log = "0.4"`; `Cargo.lock` +1 line (`"log"`, 0.4.29 already locked); `Win32_System_Com` is a feature of `windows`; `cargo tree -e features -i log`: only `default`, no `max_level_*`/`release_max_level_*` feature compiles records out | PASS |
| S10 | README (where the log lives), book build-and-run, CHANGELOG | `README.md:743-793` `### Log file`; `book/src/build-and-run.md:114-132` (console table) + `:141` `### Log file`; `CHANGELOG.md:89-117` (Added), `:122-181` (Changed), under `## [Unreleased]` | PASS |

Manual checks (human only, none run): M1-M8 in `impl.md:180-191`, mirrored in the CHANGELOG manual-check lines. They
include the 1.5 hand-off (M3 = 1.5 M1/M1b) and Checkpoint 1 smoke #8 (M2).

## Issues claimed closed
| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| none | Roadmap 1.7 cites no KS issue, and the lane claims none closed. It touches KS-08 (no hook-thread console write on action failures), KS-33 (message -> counter) and KS-20 (a refused second start never attaches the file) without changing their status | n/a | n/a |

Red-first order is confirmed by the commits: `848e95c` (test only, +52 in `config_watcher.rs` tests, +4 `testing.rs`)
comes before `ef3f656`, and `eebace9` (test only, +126 in the `mod.rs` tests) comes before `0dc008e`. The tester
replayed both reds on their parents (N4). The commit-1 module tests are new capability, carried by 14 mutations (13
caught by a named test; the 14th was closed with `9b6414d`).

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | Callback body unchanged (`keyboard_hook_callback.rs:319-354`): `code < 0` first, sentinel before state (`:180`), every path `LRESULT(1)`/`CallNextHookEx`. New work: `counters::bump` (one `fetch_add(AcqRel)`, constant-index slot `counters.rs:60-67`) on the short-mask and failed-spawn paths. Action-failure records on the hook thread (D7) cost one `String` built with `write!` (no panic), a lock-free `OnceLock::get` and a `try_send`, on failure paths only. A full queue drops and counts. A stuck writer never blocks the caller. Nothing logs under `with_engine` |
| Engine purity, matching per owner decisions | OK | `src/engine` diff empty; no matching, repeat or modifier change |
| Thread ownership, sentinel, atomics, SAFETY | OK | Hooks still installed and removed on main. The new `log-writer` uses no UI API and needs no pump. No new `SendInput`; the three builders keep `injected_sentinel()`. Only `AcqRel` RMWs, no `Relaxed`/`SeqCst`. The 3 new `unsafe` blocks (`known_folder.rs:19-25,27-28,37-40`) have concrete SAFETY comments. No production `unwrap`/`expect`/indexing in `src/logging` (the `RefCell`/`Mutex` are in `cfg(test)` `testing.rs`). `flush()` (sleeps, at most 1 s) is called only from main before the hooks or after uninstall, from `fatal`, and from the console-control thread |
| Config reload, IPC | OK | Reload semantics unchanged: `a_save_with_a_misspelled_field_keeps_the_live_bindings` (`config_watcher.rs:329`), `a_deleted_config_keeps_the_live_bindings` (`:305`) green. No pipe/DACL change |
| No key logging | OK | See A4/S7. The loader's duplicate warnings quote config key names: config text, not pressed keys, stated in README "Log file" |
| KS-20 ordering | OK | `logging::init()` comes before the guard (`main.rs:19`), `attach_log_file()` is the first step of `run` (`main.rs:68`), and the already-running path logs and flushes and never attaches (`main.rs:31-33`) |

## Review status
- Critique: CHANGES_REQUESTED (I1, I2 Important; M1-M4 Minor). Rulings adopted all of them, and I verified each
  in the tree: I1 `sink.rs:45-98` (per-line `{ts} [TAG] ` built on the writer at `writer.rs:200`, trailing breaks
  trimmed, console tag on line 1 only); I2 `mod.rs:108-122` + `methodology.md` harness (my own run above); M1
  `sink.rs:251-260` + test `:601`; M2 `writer.rs:186-192` + README `:788-793`; M3 `impl.md:190`; M4
  `actions.md:289`, `errors.md:20-37`, `architecture.md:93`.
- Rulings Q1-Q6, N1-N5 and the 1.5 hand-off are all carried out: UTC (`sink.rs:118-133`, README); prompt by
  `writeln!` (`main.rs:58`); `gaps.md:223-224` follow-up rows; `CLAUDE.md` untouched; no `CoInitialize`;
  `writer.rs:4-8` lock wording; no probe; the N3 scan; `ipc_server.rs:367` `let path` gone; M3 in impl.md and the
  CHANGELOG.
- Code review r1: **APPROVED** (Critical 0, Important 0, Minor 2) at `9b6414d`. No triage was needed.
- Tester r1: **GREEN** at `9b6414d`; one gap (`size > 0` guard) closed with test commit `9b6414d`.
- Open Minor items (optional, worth doing while fresh):
  - m1 `mod.rs:104-111`: a second `init()` spawns a throwaway writer before `set_logger` fails. When it ends it
    takes `COUNTERS` and prints the counts once without a timestamp, and the doc says "does nothing".
    Unreachable in the daemon and in `cfgcheck`, which call it once.
  - m2 `mod.rs:124-127`: the `attach_file` doc omits the `TimedOut` failure (queue full for 1 s).

## Docs and hygiene
- README, `default_config.ron:4-7`, `build-and-run.md`, `architecture.md`, `gaps.md`, `overview.md`,
  `config/{actions,errors,structure,triggers-conditions}.md`, `methodology.md` and CHANGELOG `[Unreleased]`
  (Added/Changed, roadmap 1.7 cited) match the code I read. The I1 layout is shown in README, build-and-run and
  errors.md 18.1.
- `audit.md` and `known-issues.md`: nothing to mark (no KS closed, no workaround tied to 1.7).
- `roadmap.md` 1.7 has no status line yet. That matches the other Phase-1 lanes, none of which touched
  `roadmap.md` (its last change predates Phase 1), so it is a hand-off, not drift of this lane.
- Untouched as required: `src/bin`, `src/engine`, `src/ipc.rs`, `build.rs`, `.github`, `CLAUDE.md`, `docs/`,
  `.claude/`, `tests/`.
- Book code links added by the lane are pinned to `0dc008e`. They resolve on GitHub only once the `--no-ff` merge
  is pushed.
- Branch: cut from current `master`, 10 commits, only this lane's work, merge `--no-ff` clean (master is the
  merge-base). Every commit has the trailer.

## Technical debt
- `mod.rs:104-111` m1 and `mod.rs:124-127` m2: in `review_r1.md` only. Not recorded in the book.
- Log folder hardening (risk 6, Q3): recorded in `gaps.md:223` (3.3/4.1).
- Panic messages only on stderr (Q4): recorded in `gaps.md:224` (2.4).
- `app.rs:248-250` `unsafe { PostThreadMessageW }` in `console_ctrl_handler` has no `// SAFETY:`. Pre-existing on
  master; the lane edited the function around it. Recorded only in `impl.md` open points.
- `spawn_sequence` still uses `thread::spawn`, which panics on failure, on the hook thread for an Immediate
  `Sequence` (cut risk 15, KS-33-like). Recorded only in `impl.md` and the cut, not in `gaps.md`.
- Re-entry refusals of `with_engine` are not counted (cut risk 15, candidate counter for 2.4). `gaps.md:158`
  records the re-entry, not the counter idea.
- Risk 8 (unverified): a High label on files created by the elevated daemon could stop a later medium-integrity
  debug daemon from appending. Not recorded. It can be checked during M1 (`icacls` on `daemon.log` after an
  elevated run).
- Risk 9: `SHGetKnownFolderPath` `Err` path drops the out-pointer (cut only; negligible, once per process).
- Pre-existing drift, not this lane's: `architecture.md` startup steps 5-8 and the build-and-run row
  `[WARN] could not create IPC listener ...` still describe the pre-1.5 order (`impl.md` open points).

## Problems
None blocking. No REWORK item.

## Positives
- Clean caller/writer split. A thread that logs only formats one `String` and calls `try_send`. File and console
  I/O, rotation and the I1 per-line expansion all live on one owner thread, and the file is written before the
  console, so a held conhost selection cannot reach the hook thread or hold back the durable record of a line.
- Hook-path diagnostics are now one atomic RMW each, and the vk-naming messages are gone. The scan test turns
  both "only the banner prints" and "the callback never logs" into gate checks for later lanes (2.1, 2.4).
- Rotation is handle-free, stops at the first failed rename, and is pinned at the real 1 MiB boundary, at `>` vs
  `>=`, at the file count, at a held `daemon.1.log`/`daemon.2.log`, and at the empty-file guard (tester mutation
  found the gap and closed it).
- KS-20 respected: a refused second start never opens, appends to or rotates the owner's file.
- I2 handled properly: the documented validation harness keeps showing loader diagnostics, which I confirmed with
  my own run.

## Notes for later lanes
- 2.4 (watchdog) can reuse `logging::counters` (add a `Counter` variant + `Display` arm + slot; `SLOTS` is a
  constant) and the writer's 60 s tick. It is also the place for the panic hook (`gaps.md:224`).
- 4.2 (background mode) and 4.3 (`swch logs`) can rely on the I1 rule: every physical line of `daemon.log` starts
  with `YYYY-MM-DDTHH:MM:SS.mmmZ [TAG] `.
- Any future `println!`/`eprintln!` token under `src` outside `src/bin` and `print_welcome` fails `cargo test`,
  even in a comment or a string.

## Hand-offs

### Phase-1 book sync (doc-writer)
1. `roadmap.md` item 1.7: add a "Status: done" block with the merge commit, branch `feat/file-logging` and an
   evidence table (A1-A4 above), as Workstream A has. Checkpoint 1 smoke #8 ("A typo in the config writes an
   error to the log") is now deliverable.
2. `gaps.md:222` "**Done** on `feat/file-logging`": name the merge commit once it exists.
3. `audit.md:656` KS-33 status line says "a Hold timer thread that cannot start is logged". Since 1.7 it is
   counted (`hold_timer_not_started`) and reported by the log writer. Status-line wording only.
4. Pre-existing drift: `architecture.md` startup steps 5-8 (`:96-102`) and the `build-and-run.md` row
   `[WARN] could not create IPC listener ...` / `[INFO] another daemon instance may already be running` (pre-1.5
   order; since 1.5 a pipe failure is `[FATAL]` before any hook).
5. Optional: `config/structure.md:38,91,104` and `config/matching.md:55` still say "is printed" / "the console
   shows". They are true, but could add "and `daemon.log`".
6. Optional: refresh book links pinned to `1dfd80a` line numbers in the build-and-run console table, and re-pin
   the `0dc008e` links to the merge commit.
7. Consider gaps rows for the `spawn_sequence` spawn panic (candidate for 2.1) and for the `with_engine` re-entry
   counter (2.4), which are now only in `cut.md`/`impl.md`.

### Orchestrator (CLAUDE.md and agent files, after the merge; rulings Q5)
1. Layout: add `src/logging/` (`mod.rs`: `init`, `attach_file`, `flush`, `log_dir`, `FATAL_TARGET`; `sink.rs`:
   `LogLine`, `LogFiles` rotation; `writer.rs`: the `log-writer` thread; `counters.rs`; `testing.rs`, `cfg(test)`
   capture) and `src/system/known_folder.rs` (plus `single_instance.rs` if still missing).
2. Owner decisions: drop "Until that item lands, use `println!`/`eprintln!`". Diagnostics are path-qualified
   `log::info!`/`warn!`/`error!`; a fatal error is `log::error!(target: logging::FATAL_TARGET, ...)`.
3. Coding principles:
   - The print macros appear only in `core/app.rs` `print_welcome` and in `src/bin`. The scan test fails on the
     tokens anywhere else under `src`, comments and strings included.
   - `keyboard_hook_callback.rs` may not name `log::` outside its tests. A per-event condition on the hook path
     is a `logging::counters::bump(Counter::...)`.
   - `logging::flush()` sleeps (at most 1 s). Never call it on the hook callback path or under `with_engine`.
4. Shared state: new statics `logging::LOGGER` (`OnceLock<SyncSender<WriterMsg>>`) and `counters::COUNTERS`
   (atomics, `AcqRel`).
5. Thread ownership: the `log-writer` thread is the only owner of console output and of the log files. Every other
   thread uses `try_send` only.
6. Tests: never call `logging::init`/`attach_file`. Use `logging::testing::capture` and
   `config::testing::TempDir`. The suite prints nothing.
7. Docs rule and `.claude/agents/doc-writer.md:80-84`:
   - the `cfgcheck` harness must call `key_switch_rs::logging::init()` first and `logging::flush()` before every
     exit (as in `methodology.md#validation-harness`);
   - the loader's `[INFO]`/`[WARN]` lines now come on stderr, so read the harness with `2>&1`;
   - `doc-writer.md:83` still says `config::load` writes the default config to a missing path, which has been
     stale since Workstream B (`reload` never writes).
8. Optional: agent context blurbs (`code-reviewer.md:25` and others) do not list the `log-writer` thread.
9. Owner manual checks M1-M8 (`impl.md:180-191`) before or right after the merge. The M7 control on `master`
   blocks the hook thread in a console write, and Windows may then silently remove the hook (KS-08). Restart that
   daemon after the control run.
10. Optional follow-up commit for review m1 (make `init` idempotent before spawning) and m2 (the `attach_file`
    doc), now or with 2.4.
11. Pre-existing: add a `// SAFETY:` comment to `app.rs:248-250` (`PostThreadMessageW` in
    `console_ctrl_handler`) in the next lane that touches `app.rs`.
