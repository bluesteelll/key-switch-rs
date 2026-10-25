# Context pack r1: p1-1.7-logging (roadmap 1.7, file logging)

Scout: project-analyst (read-only). Read on `feat/file-logging`, HEAD `bb9c80f`, tree clean; `master` = `1058ddb`.
All `file:line` below are on `bb9c80f` unless a commit is named. Facts only; `UNVERIFIED` marks inference.

## Scope and delta

`git log --oneline master..feat/file-logging` (oldest last):

| # | SHA | Subject | Files |
|---|---|---|---|
| C9 | `bb9c80f` | Logging: no test calls attach_file, as the cut's D11 says (1.7) | `src/logging/mod.rs` (-9: removed test `a_file_cannot_be_attached_without_a_writer`) |
| C8 | `c21c8f6` | Book: the log file, its lines and the log-writer thread (1.7) | 9 book pages (below) |
| C7 | `0dc008e` | Logging: every remaining diagnostic goes through log (1.7) | CHANGELOG, README, `core/app.rs`, `core/windows_actions.rs`, `data/sequence.rs`, `hook/ipc_server.rs`, `hook/keyboard_hook.rs`, `system/system_function.rs` |
| C6 | `eebace9` | Logging: red-first: only the startup banner prints outside src/bin (1.7) | `src/logging/mod.rs` (+126, scan test) - declared red |
| C5 | `783f383` | Hook: count short SendInputs and unstarted Hold timers instead of logging them (1.7) | CHANGELOG, `windows_actions.rs`, `keyboard_hook_callback.rs`, `system_function.rs` |
| C4 | `ef3f656` | Config: loading, reloading and hotkey lookups log through log (1.7) | CHANGELOG, README, `default_config.ron`, `config/loader.rs`, `hook/config_watcher.rs`, `system/registry.rs` |
| C3 | `848e95c` | Config: red-first: a reload error reaches the log with a timestamp (1.7) | `hook/config_watcher.rs` (+52 test), `logging/testing.rs` (re-export `TIMESTAMP_LEN`) - declared red |
| C2 | `95c9b3a` | Daemon: start the logger first and write daemon.log once this daemon owns the session (1.7) | CHANGELOG, `Cargo.toml` (`Win32_System_Com`), README, `core/app.rs` (flush in `print_welcome`), `main.rs`, `system/known_folder.rs` (new), `system/mod.rs` |
| C1 | `8217583` | Logging: the log facade with an in-house rotating file sink (1.7) | `Cargo.toml` (`log = "0.4"`), `Cargo.lock` (+1 line `"log"`), `lib.rs`, `src/logging/{counters,mod,sink,testing,writer}.rs` |

`git diff --stat master..feat/file-logging`: 33 files, +2203 / -159. Untouched (diff empty, verified):
`src/bin/`, `src/engine/`, `src/ipc.rs`, `build.rs`, `.github/`, `CLAUDE.md`.

## File map

| path:range | what lives there | why it matters this round |
|---|---|---|
| `Cargo.toml:48`, `:59` | `"Win32_System_Com"` feature; `log = "0.4"` | the one new dependency (spec); `Cargo.lock:83` adds only `"log"` to the package deps |
| `src/lib.rs:11` | `pub mod logging;` | new module |
| `src/logging/mod.rs:20-154` | `LOGGER` static, `impl log::Log`, `init`, `attach_file`, `flush`, `log_dir`, constants | logger on every thread incl. hook thread (R1, R3) |
| `src/logging/mod.rs:156-341` | tests incl. scan test `:268-329` | acceptance `rg` as a test; N3 hook-callback check |
| `src/logging/sink.rs:14-98` | consts, `LogLine::{new,file_text,console_text}` | line format, I1 per-line prefix |
| `src/logging/sink.rs:101-157` | `tag`, `is_own_target`, `write_timestamp`, `civil_from_days` | ISO-8601 UTC, `[FATAL]` mapping |
| `src/logging/sink.rs:159-261` | `WriteError`, `LogFiles::{write_line,open,rotate}` | rotation at 1 MiB, 3 files, M1 stop-at-first-failure |
| `src/logging/sink.rs:263-615` | 17 sink tests | rotation/limit/temp-dir acceptance |
| `src/logging/writer.rs:18-142` | `QUEUE_CAPACITY`, `WriterMsg`, `spawn`, `send_line`, `SendFailure`, `send_by`, `attach_on`, `flush_on` | non-blocking caller side, bounded queue, 1 s flush |
| `src/logging/writer.rs:144-241` | `Writer::{run,write,write_files,report_counters}`, `write_console`, `writer_line` | file before console (M2), failure streak, counter reports |
| `src/logging/writer.rs:243-470` | 9 writer tests | console/file/flush/drop behaviour |
| `src/logging/counters.rs:12-108` | `Counter` (4 variants + `Display`), `Counters`, `summary`, `COUNTERS`, `bump` | hook-path counters (R4, R6) |
| `src/logging/testing.rs:1-118` | `capture` (thread-local), `record`, `is_timestamped`, `Console` | `cfg(test)` only; how tests see records |
| `src/system/known_folder.rs:1-56` | `local_app_data()` via `SHGetKnownFolderPath` + `TaskMemString` Drop guard | 3 new `unsafe` blocks (CLAUDE.md principle 4) |
| `src/main.rs:14-108` | `logging::init` first, guard, `fatal`, `run`, `attach_log_file` | D2 ordering vs KS-20; flush on exits |
| `src/core/app.rs:100-117` | `print_welcome` (flush at `:104`, 6 `println!`) | the only allowed prints |
| `src/core/app.rs:155-213` | `run` log calls (`:158,171,175,186,190-193,194,213`) | stdout -> log records |
| `src/core/app.rs:241-262` | `console_ctrl_handler`: `log::info!` `:244`, `PostThreadMessageW` `:248-250`, `flush` `:257` | runs on system-created thread; 5 s close budget |
| `src/hook/keyboard_hook_callback.rs:98`, `:254-264`, `:281-304` | `counters` import; `send_menu_mask` bump; `start_hold_timer` bump + `forget_timer` | R1: no log call in the callback module |
| `src/core/windows_actions.rs:189-226`, `:228-282` | `press_vk` -> `bump(KeyPressShort)` `:224`; `log::error!` at `:238,254,280` | action failures logged on the hook thread (D7, KS-08) |
| `src/system/system_function.rs:58,77,89,99,105,114`, `:122-158` | `log::` calls; `press_key` -> `bump(KeyPressShort)` `:156` | reachable from `carry_out` on the hook thread |
| `src/data/sequence.rs:159-162,190,225-228,266,286` | `log::warn!` on macro worker threads | `press_key` no longer names the vk (`:189-190`) |
| `src/hook/config_watcher.rs:158-170` | `reload_into` Ok `:163` / Err `:166-167` | acceptance "reload error logged with a timestamp" |
| `src/hook/config_watcher.rs:350-398` | test `a_reload_error_is_logged_with_a_timestamp` | C3 red-first |
| `src/config/loader.rs:163,168,180,237,287,377` | `log::info!`/`log::warn!` | loader diagnostics (main thread at start, `config-watcher` on reload) |
| `src/system/registry.rs:86`, `:95,101,111`, `:207-209` | doc + `log::info!` x3, `log::warn!` | hotkey source lines (main or `config-watcher`) |
| `src/hook/ipc_server.rs:138,158,170`, `:366-374` | `log::warn!` x3; manual ignored test, `let path` + print removed (N5) | |
| `src/hook/keyboard_hook.rs:236` | `log::warn!` foreground-hook removal | |
| `src/config/default_config.ron:5-7` | header comment names `daemon.log` | template text |
| `README.md:150-151`, `:623`, `:737-739`, `:743-794` (`### Log file`), `:810-811`, `:820` | where the log lives, already-running line, Technologies | brief: README |
| `CHANGELOG.md:89-118` (Added), `:122-182` (Changed), `:546-547` | Keep a Changelog entries with manual-check text | brief: CHANGELOG |
| `book/src/build-and-run.md:84`, `:114-135`, `:141-160` (`### Log file`), `:442` | console table, Log file section, troubleshooting row | brief: book build-and-run |
| `book/src/architecture.md:93`, `:160`, `:167`, `:359`, `:426`, `:453`, `:457` | startup step 1, console-control row, `log-writer` row, hold-timer text | thread model |
| `book/src/gaps.md:222-224` | 1.7 Done; follow-ups "Log folder hardening", "Panic messages in the log" | rulings Q3/Q4 |
| `book/src/config/errors.md:20-37`, `:175-202` | 18.1 layout (I1), 18.6 message table | message tables |
| `book/src/config/actions.md:82,144,166,192,289`; `structure.md:90,105`; `triggers-conditions.md:67`; `overview.md:84-85`; `methodology.md:378-459` | message tags, counters, harness (I2) | doc drift check |

## Facts

### Logger and caller side
1. `LOGGER` is `static DaemonLogger { writer: OnceLock<SyncSender<WriterMsg>> }` (`src/logging/mod.rs:49-57`); no `static mut`, no `Mutex` in production code under `src/logging` (`rg "Mutex" src/logging` hits only `testing.rs:9,82`).
2. `DaemonLogger::log` (`mod.rs:66-85`): `enabled` check, `LogLine::new(SystemTime::now(), ...)`, under `cfg(test)` `testing::record(&line)`, then `writer::send_line` via `OnceLock::get`, or `write_to_stderr` when no writer or the writer is gone.
3. `write_to_stderr` writes `console_text()` to `io::stderr()` under `cfg(not(test))` (`mod.rs:94-98`) and is a no-op under `cfg(test)` (`mod.rs:101-102`).
4. `MAX_LEVEL = LevelFilter::Info` (`mod.rs:42`); `enabled` is `level <= MAX_LEVEL` (`mod.rs:60-62`); `set_max_level(MAX_LEVEL)` at `mod.rs:113` and `testing.rs:31`.
5. `init()` (`mod.rs:108-122`) calls `writer::spawn(Box::new(io::stderr()), &COUNTERS, REPORT_PERIOD)` at `:109` BEFORE `log::set_logger` at `:110`; on `set_logger` error it returns at `:111` (the spawned sender is dropped there). On spawn error it logs `could not start the log writer, logging to the console only: {error}` at `:119`.
6. `attach_file(dir)` (`mod.rs:128-138`) builds `LogFiles::new(dir)`, errs `the log writer is not running` when `LOGGER.writer` is empty (`:131-134`), else `writer::attach_on(tx, files, FLUSH_TIMEOUT)` (`:135`). No probe/open happens in `attach_file` (rulings N2; cut addendum line 499-501).
7. `flush()` returns `true` with no writer (`mod.rs:143-148`), else `writer::flush_on(tx, 1 s)`. `FLUSH_TIMEOUT = 1 s`, `REPORT_PERIOD = 60 s` (`mod.rs:45,47`).
8. `log_dir(p) = p.join("key-switch-rs")` (`mod.rs:152-154`); `LOG_DIR_NAME`, `LOG_FILE_NAME = "daemon.log"`, `FATAL_TARGET = "key_switch_rs::fatal"` (`mod.rs:35-40`).

### Line format (sink.rs)
9. `LogLine` holds one `String` plus `console_from` and `body_from` offsets (`sink.rs:31-39`); `new` uses `write!`/`write_fmt` into the `String` with results discarded, no `format!` (`sink.rs:51-62`); trailing `\r`/`\n` are trimmed (`:61-62`).
10. `file_text()` repeats `text[..body_from]` (timestamp + `[TAG] `) before every physical line after the first, via `str::lines()`, each line ending in `\n` (`sink.rs:74-85`); the target prefix appears on the first line only.
11. `console_text()` returns `text[console_from..]` + `\n` (`sink.rs:90-98`): no timestamp, tag on the first line only.
12. `file_text`/`console_text` allocate their `String` on the thread that calls them; the writer calls them (`writer.rs:200,235`), and `testing::record` calls `file_text` under `cfg(test)` (`testing.rs:60`).
13. `tag()` returns `"FATAL"` only for `Level::Error` with target `FATAL_TARGET`, else `level.as_str()` (`sink.rs:103-109`); own-target test is prefix `key_switch_rs` followed by end or `::` (`sink.rs:112-116`).
14. Timestamp: `duration_since(UNIX_EPOCH).unwrap_or(ZERO)` then `{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}.{ms:03}Z` (`sink.rs:120-133`); `civil_from_days` at `sink.rs:138-157` (unsigned, days >= 0).
15. `TIMESTAMP_LEN = 24` is `#[cfg(test)]` (`sink.rs:15-16`); `MAX_FILE_BYTES = 1024 * 1024` (`:20`); `KEPT_FILES = 3` (`:24`).

### Files and rotation (sink.rs)
16. `LogFiles { dir, limit }` keeps no handle (`sink.rs:184-187`); `file(0) = daemon.log`, `file(i) = daemon.{i}.log` via `format!` (`sink.rs:208-214`).
17. `write_line` (`sink.rs:219-230`): `open()`, `metadata().len()`, rotate iff `size > 0 && size.saturating_add(len) > limit` (`:223`), reopen, one `write_all`; a rotate error is returned as `WriteError::Rotate` after the line was written (`:229`).
18. `open()` uses `OpenOptions::new().create(true).append(true)`; on `NotFound` it runs `fs::create_dir_all(dir)` and retries once (`sink.rs:234-244`).
19. `rotate()` iterates `(1..KEPT_FILES).rev()` renaming `file(i-1) -> file(i)`, returns the first non-`NotFound` error (stops there) (`sink.rs:251-260`).

### Writer thread (writer.rs)
20. `spawn` creates `sync_channel(QUEUE_CAPACITY = 1024)` and a thread named `log-writer` via `thread::Builder` (`writer.rs:19,41-57`).
21. `send_line` uses `try_send`; `Full` bumps `Counter::LogLinesDropped` and returns `Ok`; `Disconnected(Line)` returns the line (`writer.rs:62-77`).
22. `send_by` retries `try_send` every `RETRY_EVERY = 1 ms` with `thread::sleep` until a deadline (`writer.rs:23,99-117`); used by `attach_on` (`:121-127`) and `flush_on` (`:132-142`); `flush_on` returns `true` on `Gone`, `false` on `TimedOut`.
23. `Writer::run` (`writer.rs:154-180`): `recv_timeout` until the next report; `Line` -> `write`; `Attach` -> set files, `failing = false`; `Flush` -> `report_counters`, `console.flush()`, `ack.send(())`; `Disconnected` -> report and return.
24. `Writer::write` writes the files first, then the console, then a streak notice (`writer.rs:186-192`).
25. `write_files` (`writer.rs:198-221`): first failure of a streak returns a `[WARN] log file <path>: <WriteError>` notice (console only); first success after a streak writes `[INFO] log file <path> works again` to the file and the console.
26. `report_counters` logs `counters::summary(&take())` as one `Warn` record when non-zero (`writer.rs:224-229`); writer records use target `key_switch_rs::logging` (`writer.rs:26,239-241`).
27. `LOGGER.writer` is a static `OnceLock` that is never cleared (`mod.rs:55-57`), so in the daemon the writer's `Disconnected` arm is not reached while the process lives (UNVERIFIED as runtime behaviour; follows from the static).

### Counters
28. `Counter::{MenuMaskShort, KeyPressShort, HoldTimerNotStarted, LogLinesDropped}` with `Display` arms per variant (`counters.rs:17-48`); `slot()` maps by `match` to constant indices (`counters.rs:60-67`).
29. `bump` = `fetch_add(1, AcqRel)` (`counters.rs:70-74`); `take` = `swap(0, AcqRel)` per slot (`counters.rs:79-81`); global `COUNTERS` at `counters.rs:104` and `bump(Counter)` at `counters.rs:107-109`.
30. `summary` returns `None` when all counts are 0, else `counters since the last report: menu_mask_short=.. key_press_short=.. hold_timer_not_started=.. log_lines_dropped=..` (`counters.rs:92-101`).

### Hook path
31. `src/hook/keyboard_hook_callback.rs` has no `log::` and no `print` match (`rg -n "log::|print" src/hook/keyboard_hook_callback.rs` empty on HEAD); its only logging-module use is `counters` (`:98`).
32. `send_menu_mask` (`cfg(not(test))`, `keyboard_hook_callback.rs:260-265`) bumps `MenuMaskShort` when `inject_menu_mask() != 2`.
33. `start_hold_timer` (`keyboard_hook_callback.rs:290-304`) bumps `HoldTimerNotStarted` on spawn error and still calls `with_engine(|e| e.forget_timer(token))` (`:300-303`).
34. `carry_out` (`keyboard_hook_callback.rs:203`) calls `action.execute()` at `:219`; `BindAction::execute` is `windows_actions.rs:119`; `PressKey`/`Media` call `press_vk` (`windows_actions.rs:128,140`).
35. `press_vk` (`windows_actions.rs:192-226`) bumps `KeyPressShort` on a short `SendInput` (`:222-225`); `SystemFunction::press_key` does the same (`system_function.rs:154-157`). Both still set `injected_sentinel()` (imports `windows_actions.rs:11`, `system_function.rs:11`; builders unchanged in the diff).
36. Log calls reachable from `carry_out` on the hook thread: `windows_actions.rs:238` (`Post message error {msg:#X}: {e:?}`), `:254` (`Launch {exe:?}: {e}`), `:280` (`OpenUrl {target:?}: ... {val}`), `system_function.rs:58,77` (language switch `{e:?}`), `:89,99,105,114` (LockWorkStation / show desktop / Taskmgr). None interpolates a vk or typed text (read from the lines).
37. `data/sequence.rs` log calls run on the macro worker spawned at `sequence.rs:116` (`thread::spawn`): `:159-162` (type_text counts), `:190` (press_key counts, comment `:189` "No key named"), `:225-228`, `:266`, `:286`.
38. `press_vk`'s doc on master said it was shared with the sequence `Key(...)` step; master and HEAD both call `press_vk` only from `windows_actions.rs` (`git grep press_vk master`), and HEAD's doc (`windows_actions.rs:189-191`) names `PressKey` and `Media` only.
39. `registry.rs:95` logs `{function:?}: registry combination {combo}` (a `KeyCombination` Display, i.e. key names from the registry); runs at load/reload (`loader.rs` path), not on the hook thread (UNVERIFIED call graph beyond `hotkey_from_values`).

### Daemon wiring
40. `main` calls `logging::init()` first (`main.rs:19`), then `InstanceGuard::acquire` (`:25`); the already-running arm does `log::error!("{}", InstanceError::AlreadyRunning)`, `logging::flush()`, exit 2 (`:27-34`), no prompt.
41. `InstanceError` other than `AlreadyRunning` goes to `fatal` before any file is attached (`main.rs:35`).
42. `run` calls `attach_log_file()` first (`main.rs:68`), then `ConfigPath::from_arg`, `config::load`, `App::run` (`:75-83`); the guard `_instance` drops when `run` returns, before `fatal(&*e)` (`main.rs:37-43`, `:67`).
43. `attach_log_file` (`main.rs:90-108`): `known_folder::local_app_data()` error -> `log::warn!("no log file: the LocalAppData folder is unknown: {error}")`; `attach_file` Ok -> `log::info!("key-switch-rs {} started (pid {}), logging to {}", ...)`; Err -> `log::warn!("no log file in {}: {error}", dir)`.
44. `fatal` (`main.rs:47-62`): `log::error!(target: FATAL_TARGET, "{error}")`; if `!logging::flush()` it writes `[FATAL] {error}` with `writeln!(io::stderr(), ..)` (`:52-54`); prompt `writeln!(io::stderr().lock(), "\nPress Enter to exit...")` (`:58`); `read_line` on stdin (`:60`).
45. `main` flushes on the Ok exit (`main.rs:39`) and the already-running exit (`:32`).
46. `App::run` order: `bind(PIPE_NAME)` `app.rs:145`, `print_welcome()` `:148` (flush at `:104`), `hook.install()` `:157`, `log::info!("Hook installed successfully")` `:158`.
47. `console_ctrl_handler` (`app.rs:241-262`): `log::info!` `:244`, `PostThreadMessageW` in an `unsafe` block without a `// SAFETY:` comment `:248-250` (pre-existing; impl.md:211-212), `crate::logging::flush()` `:257`, `BOOL(1)` `:258`.
48. `known_folder::local_app_data` (`known_folder.rs:18-30`): `SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, None)` wrapped in `TaskMemString` only on `Ok` (`:23-25`, `?` on the Result); `as_wide` copy (`:28`); `Drop` calls `CoTaskMemFree` (`:35-42`). Three `unsafe` blocks, each with a `// SAFETY:` comment (`:19-22`, `:26-27`, `:37-39`). No `CoInitialize` anywhere in `src` (`rg -n CoInitialize src` returns nothing).

### Config path
49. `reload_into` (`config_watcher.rs:158-170`): Ok -> `log::info!("config reloaded ({count} bindings live)")`; Err -> `log::error!("config reload failed:\n{e}")` then `log::info!("keeping previous bindings live")`.
50. `config_watcher.rs:18-21` module doc already said "Errors during reload are logged" on master (no hunk there).
51. `loader.rs:237` `log::warn!("{warning}")`, `:287` `log::warn!("config contains no bindings")`, `:163,168,180,377` `log::info!`.

### Tests and harness
52. `testing::capture` installs `LOGGER` once with `Once`, no writer, asserts `ptr::addr_eq(log::logger(), &LOGGER)`, captures the FILE form of records logged on the calling thread only (`testing.rs:27-63`).
53. No test calls `logging::init` or `logging::attach_file`: `rg -n "attach_file\(|logging::init\(|local_app_data\(" src` lists `main.rs:19,91,99`, the definitions (`mod.rs:128`, `known_folder.rs:18`), `known_folder.rs:52` (read-only test) and `mod.rs:198` (a test name). C9 removed `a_file_cannot_be_attached_without_a_writer` (`bb9c80f`).
54. Every disk test in `sink.rs`/`writer.rs` uses `crate::config::testing::TempDir` (`sink.rs:269`, `writer.rs:248`). Writer tests start real `log-writer` threads on a `Console` (`writer.rs:249`, helper `start`).
55. Scan test `only_the_startup_banner_prints_outside_src_bin` (`mod.rs:268-329`): walks `CARGO_MANIFEST_DIR/src` skipping `src/bin` (`:272-277`); allows a `println!` only inside `fn print_welcome` of `src/core/app.rs` (brace balance, `:233-249`) and never `eprintln!` (`:290-291`); also rejects any `log::` path in `src/hook/keyboard_hook_callback.rs` before the first `#[cfg(test)]` + `mod tests {` pair (`:298-310`, pair is at `keyboard_hook_callback.rs:387-388`). It does not scan other hook-thread files (`windows_actions.rs`, `system_function.rs`) for `log::`.
56. Literal `rg -n "println!|eprintln!" src --glob '!src/bin/**'` on HEAD prints exactly `src/core/app.rs:105,106,109,112,114,116` (all in `print_welcome`, `app.rs:103-117`). `writeln!` at `main.rs:53,58` does not match the pattern.
57. `print_welcome` lines `app.rs:106,112,114` contain non-ASCII (`─`, `—`), present on master (pre-existing; CLAUDE.md principle 8 "Keep source ASCII").
58. cfgcheck harness (rulings I2) was run by the developer; outputs in `impl.md:143-178` (default config: 2 `[INFO]` lines on stderr, `OK: 3 bindings`, exit 0; dup file: `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; ...`; example.md: `OK: 26 bindings`).
59. Book links added by the lane are pinned to `0dc008e` (impl.md:138-139; e.g. `architecture.md:167`, `:359`).

## Red-first tests and acceptance proofs (exact commands)

Tester method: scratch worktree under `D:/tmp/ks-orch/p1-1.7-logging/wt/` with its own `CARGO_TARGET_DIR` (`.claude/agents/tester.md:70-85`, `docs/WORKFLOW.md:69`). Per rulings N4, C3 and C6 reds replay on their parents (`95c9b3a`, `783f383`) plus the test; the red commits `848e95c` / `eebace9` are exactly parent + test (commit stats above), so checking out the red commit is the same tree. On `master` both are compile errors (no `logging` module) - declared, not a missing red. C1/C2 tests are new capability with no runtime red (N4).

| Red-first | Red tree | Command | Expected red | Green tree | Expected green |
|---|---|---|---|---|---|
| `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp` | `848e95c` (= `95c9b3a` + test) | `cargo test --locked --lib hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp` | panic `a reload error must reach the log; captured: []` (`config_watcher.rs:370`; commit body of `848e95c`) | `ef3f656`, HEAD | pass |
| `logging::tests::only_the_startup_banner_prints_outside_src_bin` | `eebace9` (= `783f383` + test) | `cargo test --locked --lib logging::tests::only_the_startup_banner_prints_outside_src_bin` | assertion message starts `27 print sites outside the startup banner:` (commit body of `eebace9`; impl.md:43) | `0dc008e`, HEAD | pass |
| `system::known_folder::tests::local_app_data_is_an_existing_absolute_folder` (C2) | `master` + test file | `cargo test --locked --lib system::known_folder` | compile error `E0425 cannot find function local_app_data` (impl.md:29-31) | `95c9b3a`, HEAD | pass (read-only, no COM init) |

Developer-run mutations (impl.md:76-79): `rotate` ignoring rename errors fails `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log` and `a_rotation_stops_at_the_first_rename_that_fails`; a `log::warn!` added to `start_hold_timer` fails the scan with `1 log calls in src/hook/keyboard_hook_callback.rs outside its tests`.

Acceptance proofs (brief line -> proof):

| Acceptance | Proof | Command / expected |
|---|---|---|
| Sink tests write to TEMP, pin rotation at size limit and file count | `logging::sink::tests::{a_line_that_would_pass_the_limit_rotates_first (sink.rs:438), a_line_that_reaches_the_limit_exactly_does_not_rotate (:454), rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest (:467), a_line_longer_than_the_limit_goes_into_an_empty_file (:492), daemon_log_rotates_at_one_mebibyte (:507), the_limit_is_one_mebibyte_and_three_files_are_kept (:535), a_rotation_that_cannot_rename_keeps_writing_to_daemon_log (:542), a_rotation_stops_at_the_first_rename_that_fails (:575)}` | `cargo test --locked --lib logging::sink` -> 17 pass |
| ISO-8601 timestamp + level on each line | `timestamps_are_iso_8601_utc_with_milliseconds (sink.rs:310)`, `a_line_is_the_timestamp_the_level_and_the_message (:331)`, `every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file (:375)`, `logging::tests::log_macros_reach_the_logger_with_a_timestamp (mod.rs:162)` | `cargo test --locked --lib logging::` |
| Folder created on demand | `the_first_line_creates_the_folder_and_daemon_log (sink.rs:414)`, `a_deleted_folder_is_created_again_on_the_next_line (:604)` | same |
| Reload error logged with a timestamp | `a_reload_error_is_logged_with_a_timestamp (config_watcher.rs:351)`: 3 physical lines, each `is_timestamped` and `[ERROR] `, then `[INFO] keeping previous bindings live`, 1 live binding | `cargo test --locked --lib hook::config_watcher` |
| Console on stderr stays | `writer::tests::the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console (writer.rs:284)`, `lines_before_the_file_is_attached_reach_the_console_only (:305)`; `init` passes `io::stderr()` (`mod.rs:109`) | `cargo test --locked --lib logging::writer` -> 9 pass |
| Counters logged elsewhere | `a_flush_reports_non_zero_counters_before_it_returns (writer.rs:337)`, `the_writer_reports_counters_every_period (:359)`, `logging::counters::tests` (4) | `cargo test --locked --lib logging::counters` |
| `rg` finds only the banner | scan test (`mod.rs:268`) + literal rg | `rg -n "println!\|eprintln!" src --glob '!src/bin/**'` -> exactly `src/core/app.rs:105,106,109,112,114,116` (Fact 56) |
| No key names/text in hook-path log calls (review) | Facts 31-37; scan rejects `log::` in the callback module | `rg -n "log::\|print" src/hook/keyboard_hook_callback.rs` -> empty |
| swch unchanged | | `git diff master...feat/file-logging -- src/bin/swch.rs` -> empty |
| Tests never touch real %LOCALAPPDATA% | Fact 53 | `rg -n "attach_file\(\|logging::init\(\|local_app_data\(" src` -> only the sites in Fact 53 |
| README / build-and-run / CHANGELOG | File map rows | `mdbook build book -d D:/tmp/ks-orch/p1-1.7-logging/book` (developer: ok, 2191 links, 0 broken; impl.md:137-139) |

Full gate (tester owns): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Developer ran only filtered `--lib` runs (190 passed, 3 ignored) and `--test ipc_roundtrip` (6 passed) (impl.md:128-133); the full suite was left to the tester.

Manual checks (human only, not run; impl.md:180-191): M1 first start creates folder + start line; M2 reload error in file with per-line `<ts> [ERROR]`; M3 (= 1.5 M1/M1b) second start `[ERROR] already running ...`, exit 2, no prompt, no new file line; M4 `missing.ron` -> `[FATAL]` in file before Enter; M5 no typed word in the three files; M6 `Launch` failure line; M7 conhost + QuickEdit held selection, with `master` control; M8 Scheduled Task reload error. No new `#[ignore]` test was added (impl.md:140-141).

## Invariants in play

- **R1 Hook callback**: no log call in `keyboard_hook_callback.rs` (Fact 31; gate-checked by scan `mod.rs:298-310`); new hook-thread work is `counters::bump` = one RMW (`keyboard_hook_callback.rs:263,301`); `log()` on the hook thread for action failures allocates one `String` + `try_send` (`mod.rs:66-85`, `writer.rs:67`); `try_send` may take std's internal channel lock to wake a parked receiver (rulings N1; `writer.rs:4-7`).
- **R2 Sentinel**: three `SendInput` builders unchanged except the short-count branch (`windows_actions.rs:222-225`, `system_function.rs:154-157`, `keyboard_hook_callback.rs:261-264`); no new `SendInput`.
- **R3 Shared state**: new statics `LOGGER` (`mod.rs:55`), `COUNTERS` (`counters.rs:104`); `with_engine` closures contain no log call (`keyboard_hook_callback.rs:192,295,302`).
- **R4 Orderings**: `AcqRel` RMW only (`counters.rs:73,80`).
- **R6 Display**: `Counter` (`counters.rs:39-48`), `WriteError` (`sink.rs:169-179`), `SendFailure` (`writer.rs:88-95`) each have per-variant arms.
- **R10 Shutdown**: `flush` at `main.rs:32,39,52`, `app.rs:104,257`; `log::info!("Removing keyboard hook...")` before `uninstall` (`app.rs:213`).
- **KS-20**: file attached only after the guard (`main.rs:19,25,68`). **KS-33**: hold-timer spawn failure counted, `forget_timer` kept (`keyboard_hook_callback.rs:300-303`); `spawn_sequence` still `thread::spawn` (`sequence.rs:116`). **KS-08**: actions still on the hook thread (`keyboard_hook_callback.rs:219`). **KS-22/23**: reload keeps live bindings (`config_watcher.rs:164-168`; test asserts 1 binding).
- Principle 4 (`unsafe` SAFETY): `known_folder.rs:19-22,26-27,37-39`; pre-existing gap `app.rs:248`.

## Open points (from impl.md:209-217, verbatim scope)

1. `src/core/app.rs:248`: `unsafe { PostThreadMessageW(..) }` inside `console_ctrl_handler` has no `// SAFETY:` comment (pre-existing; the function was touched around it).
2. `book/src/architecture.md` startup steps 5-8 and the build-and-run console row `[WARN] could not create IPC listener ...` describe the pre-1.5 order (pre-existing drift for the doc-writer).
3. `spawn_sequence` still uses `thread::spawn`, which panics on failure (`sequence.rs:116`; cut risk 15, follow-up).
4. impl.md reports no pre-existing warnings (fmt/check/clippy clean on HEAD).

Deviations recorded by the developer (impl.md:96-110): no `LogFiles::probe`; `file_text`/`console_text` return `String`; `attach_on` retrying `try_send` with 1 s deadline and a second `no log file in <folder>` warning form; reload red-first uses `PressKey("Foo")` not `bindngs`; extra tests; widened doc touch set; README counters sentence in C7; C6 amended once locally before hand-off.

## Not covered

- Did not run any cargo command, the red-first replays or mdbook (tester's gate).
- Did not read the bodies of sink tests other than `:374-412` and `:506-602`, nor writer tests other than `:283-345` and `:440-470`; `logging::testing::Console` users beyond these.
- Did not read the full README/CHANGELOG/book prose beyond `README.md:735-794`, the CHANGELOG `+` lines, `build-and-run.md:141-161`, `architecture.md:93,160,167,359`, `gaps.md:222-224`; `errors.md`, `actions.md`, `methodology.md`, `structure.md`, `triggers-conditions.md`, `overview.md` hunks were located (File map) but not read.
- Did not trace every caller of `registry::hotkey_from_values` or the notify backend's own `log` records.
- `critique.md` read by headings only (`critique.md:1-169`); rulings and impl.md read in full; cut.md read in full.
