# Implementation: p1-1.7-logging round 1

Branch `feat/file-logging` (from `master` = `1058ddb`), HEAD `bb9c80f`, tree clean. Implemented against
`cut.md` as amended by `rulings.md`; the developer addendum at the end of `cut.md` records how every ruling and
critique item was handled.

## Commits

| # | SHA | Subject |
|---|---|---|
| C1 | `8217583` | Logging: the log facade with an in-house rotating file sink (1.7) |
| C2 | `95c9b3a` | Daemon: start the logger first and write daemon.log once this daemon owns the session (1.7) |
| C3 | `848e95c` | Config: red-first: a reload error reaches the log with a timestamp (1.7) (declared red) |
| C4 | `ef3f656` | Config: loading, reloading and hotkey lookups log through log (1.7) |
| C5 | `783f383` | Hook: count short SendInputs and unstarted Hold timers instead of logging them (1.7) |
| C6 | `eebace9` | Logging: red-first: only the startup banner prints outside src/bin (1.7) (declared red) |
| C7 | `0dc008e` | Logging: every remaining diagnostic goes through log (1.7) |
| C8 | `c21c8f6` | Book: the log file, its lines and the log-writer thread (1.7) |
| C9 | `bb9c80f` | Logging: no test calls attach_file, as the cut's D11 says (1.7) |

Every commit except C3 and C6 passed `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
before it was made; C3 and C6 pass both and fail only their own red-first test. C6 was amended once, locally,
before any hand-off (clippy `reversed_empty_ranges` in the first version of the scan test); never pushed.

## Items

- roadmap 1.7, sink/writer/counters - `src/logging/{mod,sink,writer,counters,testing}.rs` (new, C1) - no red
  (new capability; on `master` the module does not exist, rulings N4).
- roadmap 1.7, log folder - `src/system/known_folder.rs` (new), `src/main.rs:14-108` (C2) - red-first
  `system::known_folder::tests::local_app_data_is_an_existing_absolute_folder`: red
  `error[E0425]: cannot find function local_app_data in this scope` -> green.
- roadmap 1.7 acceptance "a reload error is logged with a timestamp" - `src/hook/config_watcher.rs:158-170`,
  `src/config/loader.rs:163,168,180,237,287,377`, `src/system/registry.rs:95,101,111,207` (C4) - red-first
  `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp`: red on `95c9b3a`+test
  `a reload error must reach the log; captured: []` (`config_watcher.rs:370`) -> green on `ef3f656`.
- roadmap 1.7 "the hook callback only updates counters" - `src/hook/keyboard_hook_callback.rs:254-264,281-304`,
  `src/core/windows_actions.rs:188-225`, `src/system/system_function.rs:121-157` (C5) - no red possible (live
  `SendInput` cut short or a failed spawn); counters and reports pinned by the C1 tests; the C6 scan pins "no log
  call in the hook callback module".
- roadmap 1.7 acceptance `rg "println!|eprintln!" src --glob '!src/bin/**'` - C7 over `app.rs`, `system_function.rs`,
  `sequence.rs`, `windows_actions.rs`, `ipc_server.rs`, `keyboard_hook.rs` - red-first
  `logging::tests::only_the_startup_banner_prints_outside_src_bin`: red on `783f383`+test
  `27 print sites outside the startup banner:` (the 27 sites the cut predicted) -> green on `0dc008e`.
  The literal rg on HEAD lists exactly the six `print_welcome` lines of `src/core/app.rs`.

## Tests added

- `logging::sink::tests`: `timestamps_are_iso_8601_utc_with_milliseconds`, `a_time_before_1970_is_written_as_the_epoch`,
  `a_line_is_the_timestamp_the_level_and_the_message`, `an_error_with_the_fatal_target_is_tagged_fatal`,
  `a_record_of_another_crate_names_its_target`, `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file`
  (I1; replaces `a_multi_line_message_is_one_record_with_one_final_newline`), `the_first_line_creates_the_folder_and_daemon_log`,
  `lines_are_appended_to_an_existing_daemon_log`, `a_line_that_would_pass_the_limit_rotates_first`,
  `a_line_that_reaches_the_limit_exactly_does_not_rotate`, `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest`,
  `a_line_longer_than_the_limit_goes_into_an_empty_file`, `daemon_log_rotates_at_one_mebibyte`,
  `the_limit_is_one_mebibyte_and_three_files_are_kept`, `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log`,
  `a_rotation_stops_at_the_first_rename_that_fails` (M1), `a_deleted_folder_is_created_again_on_the_next_line`.
- `logging::counters::tests`: `no_summary_while_every_count_is_zero`, `the_summary_names_every_counter_once_one_is_non_zero`,
  `take_returns_the_counts_and_resets_them`, `counter_names_are_snake_case`.
- `logging::writer::tests`: `the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console`,
  `lines_before_the_file_is_attached_reach_the_console_only`, `a_flush_returns_once_earlier_lines_are_written`,
  `a_flush_reports_non_zero_counters_before_it_returns`, `the_writer_reports_counters_every_period`,
  `a_failing_log_file_is_reported_once_on_the_console`, `a_log_file_that_works_again_is_reported_in_the_file_and_on_the_console`,
  `a_full_queue_drops_the_line_and_counts_it`, `a_line_for_a_writer_that_is_gone_comes_back`.
- `logging::tests`: `log_macros_reach_the_logger_with_a_timestamp`, `debug_and_trace_records_are_not_logged`,
  `a_fatal_record_is_tagged_fatal`, `the_log_folder_is_key_switch_rs_under_local_app_data`,
  `flush_without_a_writer_returns_true_at_once`, `only_the_startup_banner_prints_outside_src_bin` (C6, N3),
  `a_path_that_only_ends_in_log_does_not_count_as_the_log_crate`.
- `logging::testing::tests::a_timestamp_and_a_space_make_a_line_timestamped`.
- `system::known_folder::tests::local_app_data_is_an_existing_absolute_folder` (read only, libtest thread, no COM init).
- `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp`.

Every disk test writes under `config::testing::TempDir` (`std::env::temp_dir()`). No test calls `logging::init` or
`logging::attach_file` (C9 removed the one that did): `rg -n "attach_file\(|logging::init\(|local_app_data\(" src`
shows only `main.rs`, the definitions and the read-only `known_folder` test.

Mutation checks run by the developer (restored afterwards): `rotate` ignoring rename errors fails
`a_rotation_that_cannot_rename_keeps_writing_to_daemon_log` and `a_rotation_stops_at_the_first_rename_that_fails`;
a `log::warn!` added to `start_hold_timer` fails the scan with
`1 log calls in src/hook/keyboard_hook_callback.rs outside its tests`.

## Remarks

- I1 - RESOLVED at `src/logging/sink.rs:45-98` (one String on the caller, file/console forms on the writer),
  pinned by `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` and
  `a_reload_error_is_logged_with_a_timestamp` (every physical line timestamped and `[ERROR]`).
- I2 - RESOLVED: `logging::init` public, console only until `attach_file` (`src/logging/mod.rs:108-122`);
  harness updated in `book/src/methodology.md#validation-harness`; runs below.
- M1 - RESOLVED at `src/logging/sink.rs:251-259`, pinned by `a_rotation_stops_at_the_first_rename_that_fails`.
- M2 - RESOLVED at `src/logging/writer.rs:186-192` (file first); limit stated in README "Log file" and book
  build-and-run "Log file".
- M3 - RESOLVED: M7 below names `conhost.exe` + QuickEdit and the `master` control.
- M4 - RESOLVED: `actions.md`, `errors.md`, `architecture.md` (plus `triggers-conditions.md`, `structure.md`,
  `overview.md`).
- N1-N5, Q1-Q6, 1.5 hand-off - see the cut addendum.

## Deviations from the cut

- No `LogFiles::probe` (rulings N2): `attach_file` always attaches when a writer exists; the start line creates
  the folder and file, failures go through the writer's once-per-streak warning.
- `LogLine::file_text`/`console_text` return `String`s built on the writer (I1), not `&str` slices.
- `writer::attach_on`/`SendFailure`: the `Attach` message is sent with a retrying `try_send` (1 s deadline);
  with no writer `attach_file` errs and main logs `[WARN] no log file in <folder>: <error>`.
- The reload red-first uses an invalid binding instead of `( bindngs: [] )` (three physical lines plus a trailing
  line break, the I1 case).
- Extra tests beyond the cut's list (writer "works again", writer gone, FATAL through the facade, the
  `names_the_log_crate` helper, `is_timestamped`).
- Doc touch set widened: `book/src/config/triggers-conditions.md:67` (named the removed hold-timer warning),
  `book/src/config/structure.md:90`, `book/src/overview.md` thread list, `book/src/build-and-run.md:84`.
- README: the counters sentence landed in C7, not C5.
- C6 amended once locally (see Commits).

## Unsafe / atomics added

- `src/system/known_folder.rs:23-25` - SAFETY: "FOLDERID_LocalAppData is a static GUID, so the pointer stays
  valid for the call; KF_FLAG_DEFAULT and no token (the current user) are documented arguments. The wrapper owns
  the out-pointer and hands back the string the shell allocated, which `TaskMemString` frees."
- `src/system/known_folder.rs:28` - SAFETY: "on success the PWSTR points to a NUL-terminated string that `path`
  owns and frees only when dropped, after this copy."
- `src/system/known_folder.rs:40` (`Drop`) - SAFETY: "the pointer came from a successful SHGetKnownFolderPath,
  which allocates it with CoTaskMemAlloc for the caller to free; this guard is its only owner, so it is freed once."
- `src/logging/counters.rs`: `fetch_add(1, AcqRel)` and `swap(0, AcqRel)` on `AtomicU64`s; the counts publish no
  other data (pairing: none needed; AcqRel is the house RMW default).
- New statics: `logging::LOGGER` (`OnceLock<SyncSender<WriterMsg>>`), `counters::COUNTERS` (atomics). No
  `static mut`, no new `Mutex` in production code (the test `Console` uses one).

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass (HEAD).
- Tests run: the red-firsts above (red and green), `cargo test --locked --lib` filtered to every touched module
  (`logging::`, `system::known_folder`, `hook::config_watcher`, `config::loader`, `system::registry`, `core::app`,
  `core::windows_actions`, `system::system_function`, `data::sequence`, `hook::ipc_server`, `hook::keyboard_hook`):
  190 passed, 3 ignored (pre-existing manual tests); `cargo test --locked --test ipc_roundtrip`: 6 passed.
  Full suite left to the tester.
- Acceptance greps on HEAD: `rg "println!|eprintln!" src --glob '!src/bin/**'` lists only the six
  `print_welcome` lines; `rg -n "log::|print" src/hook/keyboard_hook_callback.rs` is empty;
  `git diff master...feat/file-logging -- src/bin/swch.rs src/engine` is empty.
- Book: `mdbook build book -d D:/tmp/ks-orch/p1-1.7-logging/book` ok; internal link and anchor check over the
  built HTML: 2191 links, 0 broken. Links to code added by this lane are pinned to `0dc008e` (they resolve once
  the merge is pushed).
- Manual ignores added: none (the existing `hold_the_daemon_pipe_name_for_a_manual_start` keeps its ignore, with
  the instruction moved into its reason).

## cfgcheck (rulings I2)

Harness: `D:/tmp/ks-orch/p1-1.7-logging/cfgcheck/` (the `main.rs` published in `methodology.md`), built with
`cargo build --offline --target-dir D:/tmp/ks-orch/p1-1.7-logging/cfgcheck/target`, path dependency on the lane
tree. Parse only: no hook, no input, no registry write, no log file (`attach_file` is never called).

`default_config.ron`, stdout and stderr together:

```text
[INFO] loading config from D:/key-switch-rs/key-switch-rs/src/config/default_config.ron
[INFO] SwitchLanguage: using default combination
OK: 3 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }) action=SwitchLanguage block_default=true block_original_combo=Some(Swallow) condition=Always trigger=Immediate
  #1: kind=Combo(KeyCombination { keys: [AnySide(Shift), Key(VIRTUAL_KEY(20))] }) action=ToggleCapsLock block_default=true block_original_combo=None condition=Always trigger=Immediate
  #2: kind=Combo(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] }) action=DoNothing block_default=true block_original_combo=None condition=Always trigger=Immediate
exit 0
```

stderr only (`2>&1 >/dev/null`): the two `[INFO]` lines.

`dup.ron` = `( bindings: [ ( keys: "CapsLock", action: SwitchLanguage ), ( keys: "F13", action: DoNothing ), ( keys: "capslock", action: ToggleCapsLock ), ] )`:

```text
[INFO] loading config from dup.ron
[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them
OK: 3 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }) action=SwitchLanguage block_default=true block_original_combo=None condition=Always trigger=Immediate
  #1: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(124))] }) action=DoNothing block_default=true block_original_combo=None condition=Always trigger=Immediate
  #2: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }) action=ToggleCapsLock block_default=true block_original_combo=None condition=Always trigger=Immediate
exit 0
```

Missing file: `ERR: file does not exist`, exit 2. The complete example of `book/src/config/example.md`
(extracted to `cfgcheck/example.ron`): `[INFO] loading config from example.ron`,
`[INFO] SwitchLanguage: using default combination`, `OK: 26 bindings (after auto-blocker expansion)`, exit 0,
so `example.md:8-13` still matches.

## Manual checks (human only; none was run, the daemon was not started)

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | debug build, no daemon, rename `%LOCALAPPDATA%\key-switch-rs` away if it exists | `cargo run --locked --bin key-switch-rs` in a console, wait 2 s, Ctrl+C | the folder and `daemon.log` are created; first file line `<ts> [INFO] key-switch-rs 0.1.0 started (pid <n>), logging to <path>`; console shows the same lines without timestamps, the loader `[INFO]` lines above the banner, then `[INFO] Hook installed successfully`; after Ctrl+C the file ends with `[INFO] Received shutdown signal...` and `[INFO] Removing keyboard hook...` |
| M2 | M1 running | save `config.ron` with `bindngs:`, wait 1 s, press CapsLock, fix and save | `daemon.log` has `<ts> [ERROR] config reload failed:`, the parse error on a line with its own `<ts> [ERROR]`, and `<ts> [INFO] keeping previous bindings live`; CapsLock still switches; after the fix `[INFO] config reloaded (3 bindings live)` |
| M3 (= 1.5 M1/M1b) | M1 running | start a second `key-switch-rs.exe` from a console; `echo $LASTEXITCODE` | console `[ERROR] already running in this session; use ...`; exit code 2; no Enter prompt; no new line in `daemon.log` |
| M4 | no daemon | `key-switch-rs.exe missing.ron` | console `[FATAL] config file <cwd>\missing.ron does not exist`, one blank line, `Press Enter to exit...`; before Enter `daemon.log` already ends with `<ts> [FATAL] config file ... does not exist` |
| M5 | release daemon via `swch open`, default config plus `( keys: "F13", action: PressKey("A") )` | type a distinctive sentence in Notepad, switch the layout with CapsLock, `swch exit` | `Select-String` for a typed word over the three files finds nothing; no line names a key except config-derived loader lines |
| M6 | release daemon, `( keys: "F13", action: Launch(exe: "no-such-program.exe") )` | press F13 | `<ts> [ERROR] Launch "no-such-program.exe": program not found` in `daemon.log`; typing not delayed |
| M7 | M6 setup, daemon started under `conhost.exe` (or the default terminal set to Windows Console Host) with QuickEdit on | hold a text selection in the daemon console, press F13 five times and type in Notepad, release the selection; repeat once on `master` as the control | on the lane: typing stays responsive while the selection is held, the five `[ERROR] Launch` lines appear after the release; on `master`: typing stalls while the selection is held (the hook thread blocks in its console write) |
| M8 | optional: Scheduled Task per build-and-run.md | `schtasks /Run /TN "key-switch-rs"`, then save the config with a typo | the reload error is in `daemon.log` with a timestamp although nobody watches the console |

## Docs

- `README.md`: new `### Log file` (path, account, UTC format with a multi-line sample, stderr/stdout, never keys
  or text, counters, rotation, how to read it, `[WARN] no log file`, `[WARN] log file`, conhost limit, 1 s flush);
  already-running line; reload errors go to the console and the log file; warnings "logged"; Technologies
  (`log` + in-house sink, `Win32_System_Com`, `SHGetKnownFolderPath`).
- `CHANGELOG.md` (Unreleased): Added file logging (with manual checks M1, M3, M4), counters; Changed
  already-running line, loading/reloading lines as records, three warnings became counts, every other line a
  record (with manual checks M5-M8 and the M7 `master` control); the KS-20 entry quotes the new line.
- `src/config/default_config.ron`: header says reload errors go to the console and `daemon.log`.
- Book: `build-and-run.md` (paragraph, console table, new "Log file" section, troubleshooting row, `:84`),
  `architecture.md` (startup step 1, console-control and `log-writer` rows, hold-timer paragraph, reload text),
  `gaps.md` (file logging done; follow-ups "Log folder hardening", "Panic messages in the log"),
  `config/errors.md` (18.1 layout, 18.6 intro and rows), `config/actions.md`, `config/structure.md`,
  `config/triggers-conditions.md`, `overview.md`, `methodology.md` (harness, I2). `CLAUDE.md` untouched (Q5).

## Open points / pre-existing

- `src/core/app.rs:248`: the `unsafe { PostThreadMessageW(..) }` inside `console_ctrl_handler` has no
  `// SAFETY:` comment (pre-existing; the function was touched only around it).
- `book/src/architecture.md` startup steps 5-8 and the build-and-run console table row
  `[WARN] could not create IPC listener ...` still describe the pre-1.5 order (pipe after the hooks, IPC failure
  only warns): pre-existing drift for the doc-writer.
- `spawn_sequence` still uses `thread::spawn`, which panics on failure (cut risk 15, follow-up).
- No pre-existing warnings: fmt, check and clippy are clean on HEAD.
