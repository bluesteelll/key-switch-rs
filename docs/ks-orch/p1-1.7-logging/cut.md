# Cut: p1-1.7-logging - roadmap item 1.7 (File logging); touches KS-08, KS-33, KS-20, KS-22/23 (smoke #8)

## Verdict
CUT READY

Tree: `master` = `1058ddb` (merge of `feat/single-instance`), read from `D:/key-switch-rs/key-switch-rs`
(`.git/refs/heads/master`). Branch to cut: `feat/file-logging`. All `file:line` below are on `1058ddb`.

## Spec re-located

| Spec reference (book @7cb3521 unless noted) | Current file:line @1058ddb | Status | Note |
|---|---|---|---|
| roadmap 1.7 Why: reload errors at `src/hook/config_watcher.rs:107` | `src/hook/config_watcher.rs:165-168` (`reload_into` Err arm: `eprintln!` x2) | moved | Ok arm `println!` at :163 |
| roadmap 1.7 Files: "the nine files outside `src/bin` that call println/eprintln" | 11 files, 53 lines: `system/system_function.rs` 57,76,88,98,104,113,153; `data/sequence.rs` 159,190,230,272,292; `hook/keyboard_hook_callback.rs` 261,297; `core/windows_actions.rs` 222,241,257,283; `system/registry.rs` 95,101,113,209; `core/app.rs` 101,102,105,108,110,112 (banner), 154,167,171,182,186,191,210,241; `main.rs` 23,36,40; `hook/ipc_server.rs` 138,158,170,374 (374 is inside an ignored manual test); `hook/keyboard_hook.rs` 236; `config/loader.rs` 163,168,180,237,287,377; `hook/config_watcher.rs` 163,166,167 | moved/grew | 6 banner + 47 others. `src/bin/swch.rs` has 18 more (untouched) |
| roadmap 1.7 acceptance `rg "println!\|eprintln!" src --glob '!src/bin/**'` | finds the 53 lines above today | unchanged (spec) | target: only the 6 lines of `core/app.rs` `print_welcome` |
| gaps.md:222 "63 println!/eprintln! calls in src/" | 71 in `src` (53 + 18 in swch) | moved | status cell to update |
| build-and-run.md:114 (@1dfd80a) "logs only to its console" | still true: no `log` use anywhere (`rg "log::" src` empty); `Cargo.toml:40-57` has no `log` | unchanged | `log 0.4.29` is already in `Cargo.lock:116` (transitive via notify/mio) |
| architecture.md:358 (@1dfd80a) "adapter logs the hold-timer failure" | `src/hook/keyboard_hook_callback.rs:286-300` (`start_hold_timer`, eprintln at :297) | moved | KS-33 partly fixed there |
| Startup banner | `src/core/app.rs:100-113` (`print_welcome`, 6 `println!`) | unchanged | stays on stdout |
| Hook-path per-event logs | mask short count `keyboard_hook_callback.rs:257-263`; hold timer `:296-299`; `press_vk` short count names the vk `core/windows_actions.rs:220-228`; `SystemFunction::press_key` names the vk `system/system_function.rs:151-159` | unchanged | `press_vk` runs on the hook thread for every PressKey repeat (repeat on by default, owner decision) |
| KS-08 actions on the hook thread (@7cb3521 `windows_actions.rs:123-124`) | `carry_out` `keyboard_hook_callback.rs:202-220` runs `BindAction::execute` (`windows_actions.rs:118-142`) on the hook thread | moved, still open (2.1) | action-failure logs are reachable from the hook thread |
| main fatal path / already-running (@1dfd80a main.rs:6-20) | `src/main.rs:17-27` (guard), `:23` (already running), `:35-44` (`fatal`, prompt) | moved | 1.5 landed |
| Console control handler | `src/core/app.rs:238-254` (`println!` at :241) | moved | runs on a system-created thread |
| Doc comments that promise printing | `system/registry.rs:86`, `config/loader.rs:228-230`, `hook/keyboard_hook_callback.rs:253-256` and `:282`, `hook/config_watcher.rs:18-21` | unchanged | reword in the commit that changes each |
| Roadmap smoke strings: #12 `[ERROR] config reload failed:`, #13 `Watching <full path>\my.ron for changes`, `[INFO] config reloaded`; 1.5 "prints already running" | `config_watcher.rs:163-167`, `app.rs:182`, `main.rs:23` | unchanged | D5 keeps every tagged console string byte-identical |
| README "the error is printed" / already-running line / [FATAL] pipe line | `README.md:148-153`, `:734-737`, `:723-726` | unchanged | :736 text changes (D8) |
| Template header "the error is printed" | `src/config/default_config.ron:3-5` | unchanged | comment only |
| Temp-dir test helper | `src/config/testing.rs:16-49` (`TempDir` under `std::env::temp_dir()`) | unchanged | reused by every disk test of this lane |

KS issues: roadmap 1.7 cites none. Related and re-read: KS-08 (`audit.md:308-315`, hook budget), KS-33 (`audit.md:653-659`, spawn failure), KS-22/KS-23 (`audit.md:525-547`, smoke #8), KS-20 (single instance, order of init), KS-09 (elevation side effects).

## Goal and non-goals

Goal (after the lane, on `master`):
- Every diagnostic outside `src/bin` goes through the `log` facade into an in-house logger. A `log-writer` thread writes each record to stderr (`[LEVEL] message`) and, once the daemon owns the session, to `%LOCALAPPDATA%\key-switch-rs\daemon.log` (`YYYY-MM-DDTHH:MM:SS.mmmZ [LEVEL] message`). The folder is created on demand. At 1 MiB the file rotates (`daemon.log` -> `daemon.1.log` -> `daemon.2.log`, 3 files kept).
- The hook callback module contains no log call. Its per-event diagnostics, and the per-key-press `SendInput` short counts, become atomic counters that the `log-writer` reports every 60 s and at every flush.
- The only `println!` lines left outside `src/bin` are the startup banner (`core/app.rs` `print_welcome`). `swch` is byte-identical.

Non-goals (left to later items or follow-ups):
- `swch logs` (4.3); log-level configuration (no RON field, no env var: fixed at `Info`); local-time timestamps (Q1); logging panics through a panic hook (Q4); moving action-failure logs off the hook thread (2.1 moves the actions themselves); the hook watchdog (2.4, which will reuse `logging::counters` and the writer tick); background mode (4.2); hardening against reparse points in the log folder (Q3); writing the banner to the file (D9).
- No config/RON change, no schema change, no IPC change.

Later items that depend on this one: 2.4 (watchdog reports through `log`), 4.2 (with no console the file is the only output), 4.3 (`swch logs` reads `daemon.log`).

## Decisions

### D1: `log` facade, one `log-writer` thread owns all output, callers only format and `try_send`
- **What.** `src/logging/` installs `static LOGGER: DaemonLogger` with `log::set_logger` and `log::set_max_level(LevelFilter::Info)`. `DaemonLogger::log` builds a `LogLine` (timestamp + tag + message; one `String`) on the caller's thread. It hands the line to `SyncSender::try_send` on a bounded `std::sync::mpsc::sync_channel(1024)`. The named `log-writer` thread owns the console (`io::stderr()`) and the `LogFiles` sink. It writes the console part, then the file part. Queue full: the line is dropped and `Counter::LogLinesDropped` is bumped. Channel gone, or the writer never started: the caller writes the console part to stderr itself (degraded mode).
- **Why.** Action failures (`Launch`, `OpenUrl`, `PostMessage`, language switch) still run on the hook thread until 2.1 (`keyboard_hook_callback.rs:217-219`). The roadmap names those messages as the reason for 1.7, so they must stay messages. Through D1 they cost one allocation and one non-blocking `try_send` on the hook thread: no file I/O, no console write, no lock. A console write can block while the user holds a QuickEdit selection in the daemon console. Today `eprintln!` on the hook thread already has that hazard (`windows_actions.rs:257`). D1 removes it. The architecture-critic section 1 lists "logging that flushes synchronously" as a hook-budget violation. The writer is the only owner of the file, so no `Mutex` is needed anywhere (R3).
- **Rejected.** (a) A synchronous `Mutex<File>` sink: file I/O, rotation renames and console writes on the hook thread on failure paths, and a lock contended with `config-watcher`. (b) An unbounded `mpsc::channel`: unbounded memory when the writer stalls. (c) `fern`/`simplelog`/`flexi_logger`/`tracing`: owner decision (in-house sink, `log` only). (d) Writing the console synchronously and only the file asynchronously: keeps the QuickEdit stall on the hook thread.
- **Cost.** One thread. Queued lines are lost on an abnormal exit (abort, TerminateProcess), so D10 flushes on every orderly exit path. Lines appear asynchronously, so the banner must flush first (D9).

### D2: Logger before the instance guard, file after it
- **What.** The first statement of `main` is `logging::init()`, which installs the logger and spawns the writer with console output only. `run()` calls `logging::attach_file(..)` as its first step, after `InstanceGuard::acquire` succeeded (`main.rs:17-27`).
- **Why.** A refused second start must still print its line (1.5 acceptance, `README.md:734-737`). It must never open, append to or rotate the file of the daemon that owns the session (KS-20). The guard is `Local\` (`single_instance.rs:24`), so it scopes the file per session.
- **Rejected.** Opening the file in `init()`: the second instance would write, and could rotate, under the running daemon.
- **Cost.** `InstanceError::Os` before the guard and the already-running line reach the console only.

### D3: Log folder from `SHGetKnownFolderPath(FOLDERID_LocalAppData)`
- **What.** New `src/system/known_folder.rs`: `pub fn local_app_data() -> windows::core::Result<PathBuf>`, with a `Drop` guard that calls `CoTaskMemFree` on the returned `PWSTR`. `logging::log_dir(local_app_data) = local_app_data.join("key-switch-rs")`, and the file is `daemon.log` in it. On error the daemon logs `[WARN] no log file: ...` and runs with console output only.
- **Why.** This is the documented API for the folder ("This function replaces SHGetFolderPath"). It does not depend on the environment block of whoever started the elevated daemon (`swch open` via `runas`, or Task Scheduler). With `hToken = NULL` it returns the folder of the account the daemon runs as. That is the same folder `%LOCALAPPDATA%` names in that account's environment, so the spec's path holds.
- **Rejected.** `std::env::var_os("LOCALAPPDATA")`: an inherited or edited environment can change it, and it can be missing in some launch contexts. `<exe_dir>`: under `Program Files` it needs admin, and it mixes logs with binaries. `%TEMP%`: it gets cleaned.
- **Cost.** One more `windows` feature, `Win32_System_Com` for `CoTaskMemFree`. That is a feature of an existing dependency, not a new crate. One `unsafe` call with a SAFETY comment.

### D4: Rotation by open-append-close per line
- **What.** `LogFiles { dir, limit }` keeps no open handle. `write_line(text)` does these steps:
  1. Opens `daemon.log` with `OpenOptions::new().create(true).append(true)`. On `NotFound` it runs `fs::create_dir_all(dir)` and retries once. This is the "created on demand" part.
  2. Reads `size` from `File::metadata().len()`.
  3. If `size > 0 && size + text.len() > limit`, it closes the file and rotates. For `i` in `(1..KEPT_FILES).rev()` it renames `name(i-1)` to `name(i)`. `std::fs::rename` replaces the target, and a missing source (`NotFound`) counts as success. It then reopens `daemon.log`.
  4. Appends `text` with one `write_all`.
  If the rotation fails, the line still goes into `daemon.log` and `Err(WriteError::Rotate(e))` is returned. The next line retries the rotation. `limit = MAX_FILE_BYTES = 1024 * 1024`, `KEPT_FILES = 3`, `name(0) = LOG_FILE_NAME` (`"daemon.log"`), `name(i) = "daemon.{i}.log"`.
- **Why.** No long-held handle means the user can open, delete or rename the files at any time. A deleted folder or file is recreated on the next line. A second same-user daemon in another session can append without a stale size (FILE_APPEND_DATA appends; the pipe stops that daemon right away anyway). Size checked before writing means a file exceeds the limit only when a single line is larger than the whole limit (then it goes alone into an empty file). Volume is low (no per-event logging), and the I/O runs on `log-writer`.
- **Rejected.** A long-lived handle with an in-memory size: lines are lost after an external delete, and the size goes stale with two writers. Truncating when the rename fails: loses data. `1_000_000` bytes: "1 MB" is read as 1 MiB, and that is documented.
- **Cost.** About 4 syscalls per line on the writer thread. If another program holds `daemon.1.log`/`daemon.2.log`/`daemon.log` open without `FILE_SHARE_DELETE`, rotation waits and `daemon.log` grows until that program closes it. The writer reports this once (D6).

### D5: Line format, tags and the console line
- **What.** File line: `{ts} [{TAG}] {prefix}{message}\n`.
  - `ts`: ISO-8601 UTC with milliseconds, `YYYY-MM-DDTHH:MM:SS.mmmZ` (24 chars, `TIMESTAMP_LEN`). It is computed from `SystemTime::now()` with a civil-from-days conversion. A time before 1970 is clamped to the epoch.
  - `TAG`: `FATAL` when `level == Error && target == logging::FATAL_TARGET` (`"key_switch_rs::fatal"`). Otherwise the level's `Display` (`ERROR`, `WARN`, `INFO`).
  - `prefix`: empty when `target == "key_switch_rs"` or starts with `"key_switch_rs::"`. Otherwise `"{target}: "`. Records of `notify` and other crates are named this way.
  - Console line: the same `String` from `console_from` on (the byte after `"{ts} "`), returned by `LogLine::console_text`. There is no second format and no second allocation.
  - A message with newlines is one record with one trailing `\n`.
  - Formatting uses `let _ = write!(text, ...)` into the `String`, never `format!`. `format!` panics when a `Display` impl returns `Err`, and `log()` must not panic on the hook thread.
- **Why.** Every documented console string (`[WARN] binding #2 ...`, `[FATAL] config file ... does not exist`, `[ERROR] config reload failed:`, roadmap smoke #12/#13) stays byte-identical on the console. Users get the timestamps in the file. UTC is pure, DST-proof and pinned by tests.
- **Rejected.** Local time: needs a time-zone offset from Win32 and is ambiguous across DST (Q1). A timestamp on the console too: churn in about 100 doc strings for a live view. Losing `[FATAL]` (log has no such level): it is documented in about 10 places (README:725, build-and-run, config/structure, errors).

### D6: Hook-path diagnostics become counters, reported by the writer
- **What.** `src/logging/counters.rs`: `pub enum Counter { MenuMaskShort, KeyPressShort, HoldTimerNotStarted, LogLinesDropped }`. Its `Display` gives the snake-case names `menu_mask_short`, `key_press_short`, `hold_timer_not_started`, `log_lines_dropped`. `pub struct Counters([AtomicU64; 4])` has `const fn new`, `Default`, `bump` (`fetch_add(1, AcqRel)`) and `take` (`swap(0, AcqRel)` per slot). It also provides `pub fn summary(&[u64; 4]) -> Option<String>`, `pub static COUNTERS` and `pub fn bump(Counter)` on the global. The writer reports the counts every `REPORT_PERIOD = 60 s`, and before it acks every flush, when any count is non-zero. The report is one record, written to the console and the file: `[WARN] counters since the last report: menu_mask_short=0 key_press_short=2 hold_timer_not_started=0 log_lines_dropped=0`. Call sites:

  | Site | Thread | Today | After |
  |---|---|---|---|
  | `keyboard_hook_callback.rs:257-263` `send_menu_mask` (`cfg(not(test))`) | main (hook) | `eprintln!` per short insert | `counters::bump(Counter::MenuMaskShort)` |
  | `keyboard_hook_callback.rs:296-299` spawn failure | main (hook) | `eprintln!` + `forget_timer` | `bump(HoldTimerNotStarted)` + `forget_timer` (unchanged) |
  | `windows_actions.rs:220-228` `press_vk` | main (hook) or `hold-timer` | `eprintln!` naming the vk | `bump(KeyPressShort)` |
  | `system_function.rs:151-159` `press_key` (ToggleCapsLock) | main (hook) or `hold-timer` | `eprintln!` naming the vk | `bump(KeyPressShort)` |
  | `logging` dispatch on `TrySendError::Full` | any | n/a | `bump(LogLinesDropped)` |

- **Why.** This is the spec ("the hook callback ... only updates atomic counters that are logged elsewhere"). It removes the key names from hook-path messages. Per-repeat warnings at the auto-repeat rate would also churn the 3 MiB budget. The writer already wakes, so no extra thread is needed. `AcqRel` is the house default; on x86-64 every RMW is a locked instruction anyway, so it costs the same as `Relaxed`, and no `Relaxed` pairing argument is needed (R4).
- **Rejected.** Rate-limited logs: they still format on the hook thread per event. A separate reporter thread: redundant. A `swch status` field: protocol change, scope of 4.3. Keeping the vk in the message: spec ("no key names ... on the hook path").
- **Cost.** The spawn-failure `io::Error` text and which key was cut short are lost. Only counts remain.

### D7: Action-failure logs on the hook thread stay `log::error!`
- **What.** These calls stay log records (D8 table): `windows_actions.rs:241` (PostMessage), `:257` (Launch), `:283` (OpenUrl), `system_function.rs:57,76` (language switch). They run on the hook thread inside `carry_out` until 2.1.
- **Why.** The roadmap names `Launch` errors as a reason for 1.7. Through D1 each call costs a format plus a non-blocking `try_send`, and only on a failure path. The arguments are config text (exe, URL, msg number) or a Win32 error, never a pressed key or typed text.
- **Cost.** With `repeat: true` on a failing `Launch`, one record per auto-repeat. The queue absorbs that, and a full queue drops and counts.

### D8: Exact mapping of every call site (message text kept)
Rule: the message text stays as it is. Only these change: the bracket tag (now the level), a leading `✓ ` (dropped), and leading or trailing `\n` padding (dropped). Untagged action errors become `error`, and untagged notices become `info`. Call macros path-qualified (`log::info!`, `log::warn!`, `log::error!`) with inlined args. Never put the tokens `println!`/`eprintln!` in prose, strings or comments under `src` outside `src/bin` (the scan in C6 and the literal rg count them).

| Site | New call |
|---|---|
| `system_function.rs:57` | `log::error!("Language switch error: {e:?}")` |
| `system_function.rs:76` | `log::error!("Language switch backward error: {e:?}")` |
| `system_function.rs:88` | `log::error!("LockWorkStation spawn error ({}): {e}", exe.display())` |
| `system_function.rs:98` | `log::warn!("Shell_TrayWnd not found - cannot show desktop")` |
| `system_function.rs:104` | `log::error!("Show desktop error: {e:?}")` |
| `system_function.rs:113` | `log::error!("Taskmgr spawn error ({}): {e}", exe.display())` |
| `system_function.rs:153` | counter (D6) |
| `sequence.rs:159` | `log::warn!("type_text: SendInput sent {sent} of {} inputs", inputs.len())` |
| `sequence.rs:190` | `log::warn!("press_key: SendInput sent {sent} of {}", inputs.len())` (the vk is dropped: no key names in the log) |
| `sequence.rs:230` | `log::warn!("press_combo: SendInput sent {sent} of {} inputs", inputs.len())` |
| `sequence.rs:272` | `log::warn!("window step skipped: no foreground HWND captured")` |
| `sequence.rs:292` | `log::warn!("window close PostMessage failed: {e:?}")` |
| `windows_actions.rs:222` | counter (D6) |
| `windows_actions.rs:241` | `log::error!("Post message error {msg:#X}: {e:?}")` |
| `windows_actions.rs:257` | `log::error!("Launch {exe:?}: {e}")` |
| `windows_actions.rs:283` | `log::error!("OpenUrl {target:?}: ShellExecuteW returned error code {val}")` |
| `registry.rs:95` / `:101` / `:113` | `log::info!` with the same text |
| `registry.rs:209` | `log::warn!("registry value {value_name:?} has type {reg_type:?}, expected REG_SZ/REG_EXPAND_SZ")` |
| `loader.rs:163` / `:168` / `:180` / `:377` | `log::info!` with the same text |
| `loader.rs:237` / `:287` | `log::warn!("{warning}")` / `log::warn!("config contains no bindings")` |
| `config_watcher.rs:163` | `log::info!("config reloaded ({count} bindings live)")` |
| `config_watcher.rs:166-167` | `log::error!("config reload failed:\n{e}")` then `log::info!("keeping previous bindings live")` |
| `app.rs:154` | `log::info!("Hook installed successfully")` |
| `app.rs:167` | `log::warn!("could not remove the keyboard hook: {error}")` |
| `app.rs:171` | ``log::info!("IPC server listening (use `swch on/off/exit`)")`` |
| `app.rs:182` | `log::info!("Watching {} for changes", path.display())` |
| `app.rs:186-190` / `:191` | `log::warn!("could not start config file watcher for {}: {e}", path.display())` / `log::info!("continuing without hot-reload")` |
| `app.rs:210` | `log::info!("Removing keyboard hook...")` |
| `app.rs:241` | `log::info!("Received shutdown signal...")`, then the existing `PostThreadMessageW`, then `crate::logging::flush()` before `BOOL(1)` |
| `main.rs:23` | `log::error!("{}", InstanceError::AlreadyRunning)` -> console `[ERROR] already running in this session; use ...` |
| `main.rs:36` | `log::error!(target: logging::FATAL_TARGET, "{error}")` -> `[FATAL] <error>` |
| `main.rs:40` | not a record: `fatal()` calls `logging::flush()`. If that returns `false`, it writes `[FATAL] {error}` to `io::stderr()` itself. Then it writes `"\nPress Enter to exit..."` with `writeln!` on `io::stderr().lock()`, errors ignored (Q2). |
| `ipc_server.rs:138` / `:158` / `:170` | `log::warn!` with the same text |
| `ipc_server.rs:374` | deleted. Its instruction moves into the `#[ignore = "manual: ..."]` reason at `:367` ("... start the release daemon now, press CapsLock in Notepad, then Enter in the daemon's console") |
| `keyboard_hook.rs:236` | `log::warn!("could not remove the foreground hook: {error}")` |

New lines (main thread, in `run`, after `attach_file`): `log::info!("key-switch-rs {} started (pid {}), logging to {}", env!("CARGO_PKG_VERSION"), std::process::id(), path.display())` on success; `log::warn!("no log file: {reason}")` when `local_app_data()` or `attach_file` fails (the reason names the path and the error).

Doc comments reworded in the same commits: `registry.rs:86` ("Logs the [INFO] line"), `loader.rs:228-230` ("Warnings ... are logged with `log::warn!` on the thread that loads it"), `keyboard_hook_callback.rs:253-256` and `:282` ("counted in `logging::counters`; the log writer reports it"), `config_watcher.rs:18` ("Errors during reload are logged").

### D9: The banner stays console-only and is printed after a flush
`print_welcome` (`app.rs:100-113`) keeps its `println!` lines on stdout. It is not logged: binding lines can contain macro `Text(...)` content (`sequence.rs:72-81`). Its first statement becomes `crate::logging::flush();`, so the loader's `[INFO]` lines that were queued earlier appear above the banner.

### D10: Flushing
- `pub fn logging::flush() -> bool` sends `WriterMsg::Flush(ack)` and waits at most `FLUSH_TIMEOUT = 1 s` for the ack. The send is a `try_send` retried every 1 ms until the deadline. The writer reports the counters, then acks. The function returns `true` when there is no writer or the ack came in time. `impl Log::flush` calls it and drops the result.
- Callers and their threads:
  - `print_welcome` (main, before the hooks are installed).
  - The end of `main` (main, after `App::run` returned and `uninstall` ran, or on the already-running path).
  - `fatal()` (main).
  - `console_ctrl_handler` (console-control thread; `CTRL_CLOSE_EVENT` allows 5000 ms).
- Never called on the hook callback path.

### D11: Tests never touch `%LOCALAPPDATA%` or the global writer
- Sink and writer tests build their own `LogFiles` / writer in `config::testing::TempDir` (under `std::env::temp_dir()`) with an injected console (`Box<dyn Write + Send>` over `Arc<Mutex<Vec<u8>>>`, test-only) and a leaked local `Counters`.
- No test calls `logging::init` or `logging::attach_file`.
- Under `cfg(test)`, `DaemonLogger::log` also pushes the full line (with timestamp) into a thread-local capture when one is active (`logging::testing::capture`). The degraded console fallback is a no-op under `cfg(test)`, so the suite prints nothing.
- `capture` installs `LOGGER` once (`std::sync::Once`: `set_logger`, `set_max_level(MAX_LEVEL)`) and asserts that the installed logger is ours (`std::ptr::addr_eq`).

### D12: Module placement
- `src/logging/` (`mod.rs`, `sink.rs`, `writer.rs`, `counters.rs`, `testing.rs` under `cfg(test)`) is a leaf. It imports only `std` and `log`; its tests may import `crate::config::testing::TempDir`, which itself imports only `std`.
- `system/known_folder.rs` imports only `windows`.
- `hook`, `core`, `system` and `main` import `logging`, and `logging` imports none of them, so no new cycle. `engine` is untouched (`#![forbid(unsafe_code)]` intact).
- No decision logic changes, so there are no new scenario tests.

### Type sketches (signatures only)
`clippy -D warnings` fails on `dead_code`: every `pub(crate)` item needs a non-test use, or must be `#[cfg(test)]`. Make `counters` `pub` (the lib API): its call sites only arrive in commit 5.

```rust
// src/logging/mod.rs
pub mod counters; mod sink; mod writer; #[cfg(test)] pub(crate) mod testing;
pub const LOG_DIR_NAME: &str = "key-switch-rs";
pub const LOG_FILE_NAME: &str = "daemon.log";
pub const FATAL_TARGET: &str = "key_switch_rs::fatal";
pub const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Info;
const FLUSH_TIMEOUT: Duration = Duration::from_secs(1);
const REPORT_PERIOD: Duration = Duration::from_secs(60);
struct DaemonLogger { writer: OnceLock<SyncSender<writer::WriterMsg>> }
static LOGGER: DaemonLogger = DaemonLogger { writer: OnceLock::new() };
impl log::Log for DaemonLogger { /* enabled: level <= MAX_LEVEL; log: LogLine::new + (test) capture + dispatch; flush: flush() */ }
pub fn init();                                     // once, first line of main; spawn failure -> warn, console-only fallback
pub fn attach_file(dir: &Path) -> io::Result<PathBuf>; // LogFiles::probe (create_dir_all + open), try_send Attach; Err if no writer
pub fn flush() -> bool;
pub fn log_dir(local_app_data: &Path) -> PathBuf; // local_app_data.join(LOG_DIR_NAME)

// src/logging/sink.rs
pub(crate) const TIMESTAMP_LEN: usize = 24;        // used by testing::is_timestamped; #[cfg(test)] if LogLine does not use it
pub(crate) const MAX_FILE_BYTES: u64 = 1024 * 1024;
pub(crate) const KEPT_FILES: usize = 3;
pub(crate) struct LogLine { text: String, console_from: usize }
impl LogLine { pub(crate) fn new(now: SystemTime, level: log::Level, target: &str, args: &fmt::Arguments<'_>) -> Self;
               pub(crate) fn file_text(&self) -> &str; pub(crate) fn console_text(&self) -> &str; }
pub(crate) struct LogFiles { dir: PathBuf, limit: u64 }
pub(crate) enum WriteError { Rotate(io::Error) /* line written */, Write(io::Error) /* line lost */ } // + Display
impl LogFiles { pub(crate) fn new(dir: PathBuf) -> Self; #[cfg(test)] fn with_limit(dir: PathBuf, limit: u64) -> Self;
                pub(crate) fn path(&self) -> PathBuf; pub(crate) fn probe(&self) -> io::Result<()>;
                pub(crate) fn write_line(&self, text: &str) -> Result<(), WriteError>; fn rotate(&self) -> io::Result<()>; }

// src/logging/writer.rs  (thread "log-writer"; no Win32, no message loop)
pub(crate) const QUEUE_CAPACITY: usize = 1024;
pub(crate) enum WriterMsg { Line(LogLine), Attach(LogFiles), Flush(SyncSender<()>) }
pub(crate) fn spawn(console: Box<dyn Write + Send>, counters: &'static Counters, report_every: Duration) -> io::Result<SyncSender<WriterMsg>>;
pub(crate) fn send_line(tx: &SyncSender<WriterMsg>, line: LogLine, counters: &Counters) -> Result<(), LogLine>; // Full: count, Ok; Disconnected: Err(line)
pub(crate) fn flush_on(tx: &SyncSender<WriterMsg>, timeout: Duration) -> bool;
// Writer state: console, file: Option<LogFiles>, failing: bool, counters, report_every.
// Loop: recv_timeout(until next report) -> Line: console then file; Attach: set file; Flush: report counters, ack.
// File error: first of a streak -> console-only "[WARN] log file <path>: <WriteError>"; first Ok after a streak -> "[INFO] log file <path> works again" (console + file).

// src/logging/counters.rs
pub enum Counter { MenuMaskShort, KeyPressShort, HoldTimerNotStarted, LogLinesDropped } // Display: snake_case
impl Counter { pub const ALL: [Counter; 4]; }
pub struct Counters { /* [AtomicU64; 4] */ } // const fn new, Default (clippy::new_without_default), bump, take
pub fn summary(counts: &[u64; 4]) -> Option<String>;
pub static COUNTERS: Counters;
pub fn bump(counter: Counter);

// src/system/known_folder.rs
pub fn local_app_data() -> windows::core::Result<PathBuf>;
struct TaskMemString(windows::core::PWSTR); // Drop: CoTaskMemFree(Some(ptr as *const c_void))
```

## Threading

| Code path | Thread | Blocking? |
|---|---|---|
| `logging::init` (set_logger, spawn `log-writer`) | main, first statement of `main` | spawn only |
| `known_folder::local_app_data`, `logging::attach_file` | main, first step of `run` (after the guard) | short file I/O, before any hook exists |
| `DaemonLogger::log` (format + `try_send`) | caller: main (startup, shutdown, hook-thread actions), `hold-timer`, `config-watcher`, `ipc-server`, connection workers, sequence workers, console-control, notify backend | never (no lock, no I/O; degraded fallback writes stderr) |
| writer loop: console, file, rotation, counter reports | `log-writer` (new, std thread, detached, no Win32 UI, no message loop) | yes, its own thread only |
| `logging::flush` | main (`print_welcome`, end of `main`, `fatal`), console-control (`console_ctrl_handler`) | at most 1 s; never the hook callback |
| `counters::bump` | main (hook callback, actions), `hold-timer` (Hold actions), any logging thread (`LogLinesDropped`) | one atomic RMW |

Hooks, windows, timers: unchanged (main installs/uninstalls both hooks and pumps `GetMessageW`; `hold-timer` threads unchanged). No COM apartment is created; `SHGetKnownFolderPath`/`CoTaskMemFree` are called without `CoInitialize` (Q6).

## Win32 API choices

| Purpose | `windows` path / std item | Feature flag (present?) | Thread | Errors | Doc link + constraint relied on |
|---|---|---|---|---|---|
| LocalAppData path | `windows::Win32::UI::Shell::SHGetKnownFolderPath(rfid: *const GUID, dwflags: KNOWN_FOLDER_FLAG, htoken: Option<HANDLE>) -> windows::core::Result<PWSTR>` with `&FOLDERID_LocalAppData`, `KF_FLAG_DEFAULT`, `None`. Verified in windows-0.62.2 `src/Windows/Win32/UI/Shell/mod.rs:3301`, `:8711`, `:51549` | `Win32_UI_Shell` (present, `Cargo.toml:47`) | main | `Err(HRESULT)` -> `[WARN] no log file: ...`, console only. On `Err` the wrapper drops the out-pointer; per the doc, a failing call needs a `CoTaskMemFree` too, so a possible leak of a few bytes once per process (risk 9) | https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath : "If this parameter is NULL, which is the most common usage, the function requests the known folder for the current user." "The calling process is responsible for freeing this resource ... by calling CoTaskMemFree, whether SHGetKnownFolderPath succeeds or not. The returned path does not include a trailing backslash." |
| Read the path | `windows::core::PWSTR::as_wide` (`unsafe fn as_wide(&self) -> &[u16]`, windows-strings-0.5.1 `src/pwstr.rs:52`) -> `OsString::from_wide` -> `PathBuf` | (core) | main | none | Safety doc: "valid for reads up until and including the next \0" |
| Free the path | `windows::Win32::System::Com::CoTaskMemFree(pv: Option<*const c_void>)`, windows-0.62.2 `src/Windows/Win32/System/Com/mod.rs:552` | **`Win32_System_Com` (absent: ADD)** | main, in `Drop` | none (void) | https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cotaskmemfree : "If this parameter is NULL, the function has no effect." |
| Console control (existing, now flushes) | `SetConsoleCtrlHandler` (unchanged) | `Win32_System_Console` (present) | console-control | n/a | https://learn.microsoft.com/en-us/windows/console/handlerroutine : "the system creates a new thread in the process to execute the function"; `CTRL_CLOSE_EVENT` timeout "SPI_GETHUNGAPPTIMEOUT, 5000ms"; "Return TRUE. In this case ... the system terminates the process." |
| Rotation rename | `std::fs::rename` | n/a | `log-writer` | `io::Error` (NotFound tolerated) | https://doc.rust-lang.org/std/fs/fn.rename.html : "replacing the original file if `to` already exists"; Windows: "`MoveFileExW` with a fallback to `SetFileInformationByHandle`" |
| Share mode of our opens | `std::fs::OpenOptions` default | n/a | `log-writer` | n/a | https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html : "By default `share_mode` is set to `FILE_SHARE_READ \| FILE_SHARE_WRITE \| FILE_SHARE_DELETE`." |
| Queue | `std::sync::mpsc::sync_channel`, `SyncSender::try_send` | n/a | any -> `log-writer` | `TrySendError::{Full, Disconnected}` | https://doc.rust-lang.org/std/sync/mpsc/struct.SyncSender.html : "Attempts to send a value on this channel without blocking"; `impl<T> Sync for SyncSender<T> where T: Send` |
| Logger install | `log::set_logger(&'static dyn Log) -> Result<(), SetLoggerError>`, `log::set_max_level` | crate `log` 0.4 (ADD to `[dependencies]`; 0.4.29 already locked) | main | second call -> `Err`, ignored | https://docs.rs/log/0.4.29/log/fn.set_logger.html : "An error is returned if a logger has already been set." The max level starts at Off (`log-0.4.29/src/lib.rs:458`), so `set_max_level(Info)` is required |
| stderr with no console | `std::io::stderr()` | n/a | `log-writer` | ignored (`let _ =`) | std treats an invalid or null std handle as a successful write (unverified as documented; the design ignores the error either way) |

Injection flags (`LLKHF_INJECTED`, extended-key, `scanCode`): not relevant. The lane adds no `SendInput`, and the three existing builders keep `injected_sentinel()` (R2). Only their short-count reporting changes.

Elevation/UIPI/DACL/token: the elevated daemon creates files in the user's own `%LOCALAPPDATA%`, and they inherit that folder's DACL. Whether a high-integrity creator stamps a High mandatory label on them is unverified (risk 8). The lane adds no pipe, process or token change.

## Touch set

| File | Change | Why |
|---|---|---|
| `Cargo.toml` | `log = "0.4"` in `[dependencies]` (C1); `"Win32_System_Com"` in the `windows` features (C2) | facade (owner decision); `CoTaskMemFree` |
| `Cargo.lock` | `key-switch-rs` dependency list gains `"log"` (0.4.29 already locked) (C1) | `--locked` gate. Update with one plain `cargo check`, then confirm `git diff Cargo.lock` shows only that line |
| `src/lib.rs` | `pub mod logging;` (C1) | new module |
| `src/logging/mod.rs` | new (C1) + scan test (C6) | logger, init/attach/flush, constants |
| `src/logging/sink.rs` | new (C1) | `LogLine`, timestamp, `LogFiles` rotation |
| `src/logging/writer.rs` | new (C1) | `log-writer` thread, queue, flush, counter reports |
| `src/logging/counters.rs` | new (C1) | hook-path counters |
| `src/logging/testing.rs` | new, `cfg(test)` (C1) | capture, `is_timestamped` |
| `src/system/mod.rs` | `pub mod known_folder;` (C2) | |
| `src/system/known_folder.rs` | new (C2) | `local_app_data()` |
| `src/main.rs` | `init` first, `run` attaches the file and logs the start line, `fatal` via `FATAL_TARGET` + flush + prompt by `writeln!`, flush at the end; `already running` via `log::error!` (C2) | D2, D8, D10 |
| `src/core/app.rs` | `print_welcome` flushes first (C2); 8 non-banner prints -> log, ctrl handler flush (C7) | D9, D8, D10 |
| `src/hook/config_watcher.rs` | 3 sites -> log, doc line 18 (C4); red test (C3) | acceptance (reload error) |
| `src/config/loader.rs` | 6 sites -> log, doc 228-230 (C4) | D8 |
| `src/system/registry.rs` | 4 sites -> log, doc 86 (C4) | D8 |
| `src/config/default_config.ron` | header lines 4-5: "...the error is written to the console and to %LOCALAPPDATA%\key-switch-rs\daemon.log, and the previous bindings stay active." (ASCII) (C4) | where reload errors go |
| `src/hook/keyboard_hook_callback.rs` | 2 sites -> counters, docs 253-256 and 282 (C5) | D6 |
| `src/core/windows_actions.rs` | `press_vk` -> counter (C5); 3 sites -> log (C7) | D6, D8 |
| `src/system/system_function.rs` | `press_key` -> counter (C5); 6 sites -> log (C7) | D6, D8 |
| `src/data/sequence.rs` | 5 sites -> log, vk dropped (C7) | D8 |
| `src/hook/ipc_server.rs` | 3 sites -> log; the manual test's print removed, ignore reason extended (C7) | D8 |
| `src/hook/keyboard_hook.rs` | 1 site -> log (C7) | D8 |
| `README.md` | new `### Log file` after "Direct launch" (path, account, format sample, rotation, never keys/text, how to open, `[WARN] no log file` case); `:734-737` new already-running text; Technologies: `log` facade + in-house sink, `Win32_System_Com` (`CoTaskMemFree`), `Win32_UI_Shell` also `SHGetKnownFolderPath` (C2); `:148-153` "written to the console and to the log file" (C4) | brief: README says where the log lives |
| `CHANGELOG.md` | Unreleased/Added: file logging (C2), counters (C5); Changed: already-running line (C2), console lines on stderr with tags, untagged errors now `[ERROR]`, `✓` lines now `[INFO]`, three warnings replaced by counters, macro `Key` warning without key name (C5/C7). Cite "roadmap item 1.7" | Keep a Changelog |
| `book/src/build-and-run.md` | `:114` paragraph -> log file; new "Log file" subsection; console table `:116-134` (✓ rows -> `[INFO]`, hold-timer row -> counters line, add `[WARN] no log file: ...`, `[WARN] counters since the last report: ...`, `[INFO] key-switch-rs <v> started ...`); troubleshooting `:419` "console or daemon.log" (C8) | brief |
| `book/src/architecture.md` | thread table (after `:166`): `log-writer` row; `:358` "counts the failure (`hold_timer_not_started`)" (C8) | new thread |
| `book/src/gaps.md` | `:222` status -> "**Done** on `feat/file-logging` (item 1.7): `%LOCALAPPDATA%\key-switch-rs\daemon.log`" (C8) | drift |
| `book/src/config/errors.md` | `:170` intro (console and daemon.log); rows `:185`, `:192`, `:193` -> one counters row; `:186`, `:187` gain `[ERROR]`; add rows for `[WARN] no log file`, `[WARN] log file <path>: ...` (C8) | message table must stay true |
| `book/src/config/actions.md` | `:82`, `:144` add `[ERROR]`; `:166` -> counter `key_press_short`; `:192` "console and daemon.log" (C8) | same |
| `book/src/config/structure.md` | `:105` "Errors are visible in the daemon's console and in `daemon.log`" (C8) | same |

`swch` (`src/bin/swch.rs`), `src/engine/**`, `src/ipc.rs`, `build.rs`, the CI yml: untouched.

## Commits, in order
Each message: subject `<Area>: <what> (1.7)`, red-first commits put the failing line in the body, trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

1. **`Logging: the log facade with an in-house rotating file sink (1.7)`**. Touches `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/logging/*`. A new module, so its tests ship with it. PREDICTED on master: the module does not exist, and the tests do not compile (`E0433 failed to resolve: could not find logging in the crate root`). Gate green. Tests:
   - `logging::sink::tests::`
     - `timestamps_are_iso_8601_utc_with_milliseconds`: `UNIX_EPOCH + ms`: 0 -> `1970-01-01T00:00:00.000Z`; 951_868_799_999 -> `2000-02-29T23:59:59.999Z`; 1_792_143_667_123 -> `2026-10-16T09:41:07.123Z`; 4_107_542_399_999 -> `2100-02-28T23:59:59.999Z`; 4_107_542_400_000 -> `2100-03-01T00:00:00.000Z`.
     - `a_time_before_1970_is_written_as_the_epoch`
     - `a_line_is_the_timestamp_the_level_and_the_message`: file `2026-10-16T09:41:07.123Z [INFO] config reloaded (2 bindings live)\n`, console `[INFO] config reloaded (2 bindings live)\n`.
     - `an_error_with_the_fatal_target_is_tagged_fatal`: also Error + other target -> `[ERROR]`, Info + `FATAL_TARGET` -> `[INFO]`.
     - `a_record_of_another_crate_names_its_target`: `notify::windows` -> `[WARN] notify::windows: ...`.
     - `a_multi_line_message_is_one_record_with_one_final_newline`
     - `the_first_line_creates_the_folder_and_daemon_log`: nested missing folder.
     - `lines_are_appended_to_an_existing_daemon_log`
     - `a_line_that_would_pass_the_limit_rotates_first`: limit 100; 3x30 B then 30 B -> `daemon.1.log` 90 B, `daemon.log` 30 B.
     - `a_line_that_reaches_the_limit_exactly_does_not_rotate`: pins `>` against `>=`.
     - `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest`: 4 rotations with distinct markers; the folder holds exactly the three names; the contents are the three newest batches in order; there is no `daemon.3.log`.
     - `a_line_longer_than_the_limit_goes_into_an_empty_file`
     - `daemon_log_rotates_at_one_mebibyte`: real `MAX_FILE_BYTES`. Pre-size `daemon.log` with `File::set_len(MAX_FILE_BYTES - 10)`. A 10-byte line does not rotate; the next 1-byte line does.
     - `the_limit_is_one_mebibyte_and_three_files_are_kept`
     - `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log`: hold `daemon.1.log` open via `OpenOptionsExt::share_mode(0x1)` (FILE_SHARE_READ: no delete sharing). Then `write_line` returns `Err(WriteError::Rotate(_))`, the line is in `daemon.log`, and `daemon.1.log` is unchanged. After the blocker drops, the next line rotates.
     - `a_deleted_folder_is_created_again_on_the_next_line`
   - `logging::counters::tests::`
     - `no_summary_while_every_count_is_zero`
     - `the_summary_names_every_counter_once_one_is_non_zero`: exact string `counters since the last report: menu_mask_short=3 key_press_short=0 hold_timer_not_started=1 log_lines_dropped=0`.
     - `take_returns_the_counts_and_resets_them`: local `Counters`.
     - `counter_names_are_snake_case`
   - `logging::writer::tests::`
     - `the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console`
     - `lines_before_the_file_is_attached_reach_the_console_only`
     - `a_flush_returns_once_earlier_lines_are_written`
     - `a_flush_reports_non_zero_counters_before_it_returns`
     - `the_writer_reports_counters_every_period`: period 20 ms, poll the console up to 5 s.
     - `a_failing_log_file_is_reported_once_on_the_console`: the folder path is an existing file; two lines give one warning.
     - `a_full_queue_drops_the_line_and_counts_it`: `sync_channel(1)` with nobody reading.
   - `logging::tests::`
     - `log_macros_reach_the_logger_with_a_timestamp`: capture; `log::info!` gives 1 line, `is_timestamped`, `[INFO] ...`.
     - `debug_and_trace_records_are_not_logged`
     - `the_log_folder_is_key_switch_rs_under_local_app_data`: `log_dir(Path::new(r"C:\Users\u\AppData\Local"))` is `C:\Users\u\AppData\Local\key-switch-rs`; `LOG_FILE_NAME == "daemon.log"`.
     - `flush_without_a_writer_returns_true_at_once`
2. **`Daemon: start the logger first and write daemon.log once this daemon owns the session (1.7)`**. Touches `Cargo.toml` (`Win32_System_Com`), `src/system/mod.rs`, `src/system/known_folder.rs`, `src/main.rs`, `src/core/app.rs` (`print_welcome` flush only), `README.md` (Log file section, `:734-737`, Technologies), `CHANGELOG.md` (Added: file logging; Changed: already-running line). Test: `system::known_folder::tests::local_app_data_is_an_existing_absolute_folder` (a read only: absolute and `is_dir()`; it runs on a test thread without `CoInitialize`). PREDICTED RED on master: `E0432 unresolved import crate::system::known_folder`. `main.rs` has no harness (`Cargo.toml:30-34`, KS-38), so the wiring is proven by manual checks M1, M3, M4. Gate green.
3. **`Config: red-first: a reload error reaches the log with a timestamp (1.7)`**. Adds `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp`. The test writes `ONE` to a TempDir, loads it, rewrites the file as `( bindngs: [] )`, then `let ((), lines) = logging::testing::capture(|| reload_into(&path, &hook));`. It asserts:
   - a line contains `[ERROR] config reload failed:`; otherwise it panics with `a reload error must reach the log; captured: {lines:?}`;
   - that line `is_timestamped`, and its text from `TIMESTAMP_LEN + 1` starts with `[ERROR] config reload failed:\n`;
   - another line ends with `[INFO] keeping previous bindings live\n`;
   - the hook still has 1 binding.
   PREDICTED RED on commit 2: panic `a reload error must reach the log; captured: []`, because `reload_into` and `config::reload` still print directly. Gate: fmt and clippy green, this test red (the declared red commit).
4. **`Config: loading, reloading and hotkey lookups log through log (1.7)`**. Touches `config_watcher.rs` (`:163-167`, doc `:18`), `loader.rs` (`:163,168,180,237,287,377`, doc `:228-230`), `registry.rs` (`:95,101,113,209`, doc `:86`), `default_config.ron:4-5` and `README.md:148-153`. Commit 3 turns green. The invariant holds: a rejected reload keeps the live bindings (`a_save_with_a_misspelled_field_keeps_the_live_bindings`, `a_deleted_config_keeps_the_live_bindings`). Gate green.
5. **`Hook: count short SendInputs and unstarted Hold timers instead of logging them (1.7)`**. Touches `keyboard_hook_callback.rs:253-263`, `:279-300` (`send_menu_mask`, `start_hold_timer` and their docs), `windows_actions.rs:220-228`, `system_function.rs:151-159`, `CHANGELOG.md` (Added: counters; Changed: the three warnings). No red-first: the changed lines run only under a live `SendInput` that is cut short or a failed thread spawn, which no test may cause (safety rules). Proof: the counters and writer tests of commit 1, the R1 review (`rg -n "log::|print" src/hook/keyboard_hook_callback.rs` is empty outside tests), and manual check M7. Invariants kept: `forget_timer` still runs on a spawn failure; the hook thread does nothing beyond one atomic RMW. Gate green.
6. **`Logging: red-first: only the startup banner prints outside src/bin (1.7)`**. Adds `logging::tests::only_the_startup_banner_prints_outside_src_bin`. The test walks `env!("CARGO_MANIFEST_DIR")/src` recursively, skipping `src/bin`, over `*.rs` files. It collects every line containing `println!` or `eprintln!`. The patterns are built with `concat!("print", "ln!")` so the test's own source does not contain them. A line is allowed only if it is in `src/core/app.rs`, inside `fn print_welcome` (from the `fn print_welcome(` line until its braces balance), and does not contain `eprintln!`. On failure it reports `"{n} print sites outside the startup banner:\n{path:line: text}..."`. PREDICTED RED on commit 5: the message starts `27 print sites outside the startup banner:`. The listed sites (master line numbers; after commits 2 and 5 the numbers in `app.rs`, `windows_actions.rs` and `system_function.rs` shift, but the count of 27 is exact):
   - `system_function.rs` 57,76,88,98,104,113
   - `sequence.rs` 159,190,230,272,292
   - `windows_actions.rs` 241,257,283
   - `app.rs` 154,167,171,182,186,191,210,241
   - `ipc_server.rs` 138,158,170,374
   - `keyboard_hook.rs` 236
   Gate: fmt and clippy green, this test red (the declared red commit).
7. **`Logging: every remaining diagnostic goes through log (1.7)`**. Applies the D8 rows for `app.rs` (+ the ctrl handler flush), `system_function.rs`, `sequence.rs`, `windows_actions.rs`, `ipc_server.rs` (+ the ignore reason) and `keyboard_hook.rs`, plus `CHANGELOG.md` (Changed: console tags, `✓` lines, untagged errors now `[ERROR]`, macro `Key` warning without the key). Commit 6 turns green. The literal `rg "println!|eprintln!" src --glob '!src/bin/**'` prints only the 6 `print_welcome` lines of `src/core/app.rs` (master 101,102,105,108,110,112, plus 1 after the commit 2 flush line). Gate green.
8. **`Book: the log file, its lines and the log-writer thread (1.7)`**. Touches the book rows of the touch set. Check with `mdbook build book -d D:/tmp/ks-orch/p1-1.7-logging/book` (links resolve). There is no RON example change, so `cfgcheck` is not needed. Gate green (no code).

Bisect: 1 and 2 add capability without changing output, except the already-running line. 3 and 6 are the only red commits, each followed by its green commit. Every other commit passes fmt, `clippy --all-targets --locked -D warnings` and `test --locked`.

## Acceptance mapping

| Acceptance line (roadmap 1.7 + brief) | Covered by |
|---|---|
| Log at `%LOCALAPPDATA%\key-switch-rs\daemon.log`, folder created on demand | `the_log_folder_is_key_switch_rs_under_local_app_data`, `local_app_data_is_an_existing_absolute_folder`, `the_first_line_creates_the_folder_and_daemon_log`, `a_deleted_folder_is_created_again_on_the_next_line`; M1 |
| Each line has an ISO-8601 timestamp and the level | `timestamps_are_iso_8601_utc_with_milliseconds`, `a_line_is_the_timestamp_the_level_and_the_message`, `an_error_with_the_fatal_target_is_tagged_fatal`, `log_macros_reach_the_logger_with_a_timestamp` |
| Rotation at 1 MB keeping daemon.log, daemon.1.log, daemon.2.log | `daemon_log_rotates_at_one_mebibyte`, `a_line_that_would_pass_the_limit_rotates_first`, `a_line_that_reaches_the_limit_exactly_does_not_rotate`, `rotation_keeps_daemon_log_daemon_1_log_and_daemon_2_log_and_drops_the_oldest`, `the_limit_is_one_mebibyte_and_three_files_are_kept`, `a_rotation_that_cannot_rename_keeps_writing_to_daemon_log` |
| Sink unit tests write to a TEMP dir, never the real %LOCALAPPDATA% | every disk test uses `config::testing::TempDir` (`std::env::temp_dir()`). Review: `rg -n "attach_file\|logging::init\|local_app_data\(" src` shows only `main.rs` and the read-only `known_folder` test |
| Console output on stderr stays when a console is attached | `the_writer_puts_the_line_in_the_file_and_the_line_without_its_timestamp_on_the_console`, `lines_before_the_file_is_attached_reach_the_console_only`; M1 |
| Replace println!/eprintln! outside src/bin except the banner; the literal rg finds only the banner | `only_the_startup_banner_prints_outside_src_bin`; the tester runs the literal rg on HEAD |
| The hook callback never logs per event; atomic counters logged elsewhere | counters tests; `a_flush_reports_non_zero_counters_before_it_returns`, `the_writer_reports_counters_every_period`. Review: no `log::` in `keyboard_hook_callback.rs` |
| Pressed keys and typed text are never logged; no key names on the hook path (review) | Review list: the only log calls reachable from `carry_out` are `windows_actions.rs:241,257,283` and `system_function.rs:57,76` (config text / Win32 errors). The vk-naming sites become counters (D6), and the sequence `press_key` drops the vk. The banner, which can contain `Text(...)`, is not logged (D9). M5 |
| A reload error is logged with a timestamp | `a_reload_error_is_logged_with_a_timestamp`; M2 (smoke #8) |
| swch unchanged | tester: `git diff master...feat/file-logging -- src/bin/swch.rs` is empty |
| README (where the log lives), book build-and-run page, CHANGELOG | commits 2, 4, 5, 7, 8; mdbook build |

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | debug build, no daemon, rename `%LOCALAPPDATA%\key-switch-rs` away if it exists | `cargo run --locked --bin key-switch-rs` in a console, wait 2 s, Ctrl+C | The folder and `daemon.log` are created. The first file line is `<ts> [INFO] key-switch-rs 0.1.0 started (pid <n>), logging to <path>`. The console shows the same lines without timestamps, the loader `[INFO]` lines above the banner, then `[INFO] Hook installed successfully`. After Ctrl+C the file ends with `[INFO] Received shutdown signal...` and `[INFO] Removing keyboard hook...` | none (runs the daemon) |
| M2 | M1 running | Save `config.ron` with `bindngs:`, wait 1 s, press CapsLock, fix and save | `daemon.log` has `<ts> [ERROR] config reload failed:`, the error, and `<ts> [INFO] keeping previous bindings live`. CapsLock still switches. After the fix: `[INFO] config reloaded (3 bindings live)` | none |
| M3 | M1 running | Start a second `key-switch-rs.exe` from a console; `echo $LASTEXITCODE` | Console: `[ERROR] already running in this session; use ...`; exit code 2; no new line in `daemon.log` | none |
| M4 | no daemon | `key-switch-rs.exe missing.ron` | Console: `[FATAL] config file <cwd>\missing.ron does not exist`, a blank line, `Press Enter to exit...`. Before Enter, `daemon.log` already ends with `<ts> [FATAL] config file ... does not exist` | none |
| M5 | release daemon via `swch open`, default config plus `( keys: "F13", action: PressKey("A") )` | Type a distinctive sentence in Notepad, toggle the layout with CapsLock, then `swch exit` | `Select-String` for a typed word over the three files finds nothing. No line names a key except config-derived loader lines | none |
| M6 | release daemon, binding `( keys: "F13", action: Launch(exe: "no-such-program.exe") )` | Press F13 | `<ts> [ERROR] Launch "no-such-program.exe": program not found` in `daemon.log`. Typing is not delayed | none |
| M7 | M6 setup, daemon in a visible console | Start a text selection in the daemon console and hold it; press F13 five times and type in Notepad; release the selection | Typing stays responsive while the selection is held (D1). The five `[ERROR] Launch` lines appear after the release | none |
| M8 | optional: Scheduled Task per build-and-run.md | `schtasks /Run /TN "key-switch-rs"`, then edit the config with a typo | The reload error is in `daemon.log` with a timestamp although nobody watches the console | none |

## Risks and edge cases (R1-R10 and lane-specific)

- **R1 Hook callback.** After commit 5, `keyboard_hook_callback.rs` has no log call. The only hook-thread log calls left are action failures (D7). Each one: `LogLine::new` (one `String`, built with `write!`, so no panic), `OnceLock::get` (lock-free), `try_send` (non-blocking). No per-key-down allocation in normal operation. `counters::bump` is one RMW. `log()` has no `unwrap`/`expect`/indexing: `console_from` is computed from the written timestamp length. A panic there would abort inside an `extern "system"` callback, and the reviewer must check this.
- **R2 Sentinel.** No new `SendInput`. `press_vk`, `press_key` and the menu mask keep `injected_sentinel()`. Only their short-count branch changes.
- **R3 Shared state.** New statics: `LOGGER` (`OnceLock<SyncSender>`) and `COUNTERS` (atomics). No `static mut`, no new `Mutex` in production code (the test console uses one). The engine lock is never held around a log call: the engine does not log, and `log()` takes no lock, so there is no lock-order issue.
- **R4 Orderings.** Counters use `fetch_add`/`swap` with `AcqRel` and publish no other data. `OnceLock` and `log`'s internals are std/crate code.
- **R5/R7/R8.** N/A (no binding, specificity or key-identity change).
- **R6 Display.** The new `Counter` enum gets an explicit `Display` arm per variant, and `WriteError` gets `Display`. The `BindAction`/`SequenceStep` `Display` impls are unchanged (the banner still uses them).
- **R9 Registry.** Only the message macros change. The new Win32 call is a read (`SHGetKnownFolderPath`) with a `Drop`-guarded `CoTaskMemFree`.
- **R10 Shutdown.** Between `GetMessageW` returning and `uninstall()` the only new work is `log::info!("Removing keyboard hook...")`, which is non-blocking. `flush` (at most 1 s) runs after `uninstall` at the end of `main`, or on the console-control thread. `CTRL_CLOSE_EVENT` still ends the process when the handler returns (pre-existing race); the handler's flush makes its own line durable.

Lane-specific:
1. **Lost lines.** An abort (a panic on the hook thread), `TerminateProcess` or a console close can kill `log-writer` with queued lines. Every orderly path flushes (D10). Panic messages still reach only stderr (Q4).
2. **Writer stall.** A hung disk or AV filter driver blocks `log-writer`. The queue (1024) fills, lines are dropped and counted, and the console also stalls, since the writer owns it. `fatal()` falls back to a direct stderr write when `flush()` returns `false`, so the `[FATAL]` line can then appear twice.
3. **Rotation blocked.** A program that holds a log file without `FILE_SHARE_DELETE` makes `daemon.log` grow past 1 MiB until it closes the file. Rotation is retried per line and reported once. The README tells users to open the log with Notepad or `Get-Content -Tail`, not to keep it open.
4. **Two writers.** Two daemons of one user in two sessions: the `Local\` mutex does not stop the second one. It dies at the machine-wide pipe (`app.rs:141-142`), but after `attach_file`, so its `[FATAL]` line goes into the shared file. Appends are separate `FILE_APPEND_DATA` writes. A rotation at that instant can misplace one line. Acceptable.
5. **Fatal after guard release.** `run` drops the guard before `fatal()` (`main.rs:46-48`). A restarted daemon could append to the file during the few microseconds before the `[FATAL]` flush. Negligible.
6. **Elevated writes in a user-writable folder.** The elevated daemon creates and renames files in `%LOCALAPPDATA%\key-switch-rs`, which medium-integrity code of the same user can pre-plant with junctions or object-manager links to redirect a rename. That is the classic log-rotation redirection pattern. Microsoft does not treat UAC as a security boundary, and the location is the owner's spec. Q3 asks the orchestrator to accept it and record a hardening follow-up.
7. **stdout to stderr.** `[INFO]` lines move from stdout to stderr; only the banner stays on stdout. Scripts that redirect stdout see fewer lines (CHANGELOG Changed).
8. **Integrity label.** Files created by the high-integrity daemon may carry a High label (unverified). A later medium-integrity debug daemon could then fail to append: `[WARN] log file ...`, console only.
9. **`SHGetKnownFolderPath` failure path.** The wrapper drops the out-pointer on `Err`, while the doc asks for `CoTaskMemFree` "whether ... succeeds or not". A possible leak of a few bytes, once per process, on an error that makes logging console-only anyway.
10. **UTC timestamps** differ from the clock on the wall (Q1). A wall-clock change makes timestamps non-monotonic. That is harmless: ordering comes from the queue.
11. **Third-party records.** `notify` warnings and errors now reach the file, prefixed with their target. No keys are involved.
12. **Config fragments in errors.** A reload error can quote config text: key names, field names, and an exe or URL in `Launch`/`OpenUrl` failures. That is the user's own config, not typed input.
13. **Reload, `swch off`/`on`, foreground change, pending gesture.** Logging does not interact with them. `set_enabled` and the engine do not log.
14. **The scan test is strict on purpose.** A future `println!` (or the token in a comment) under `src` outside `src/bin` and the banner fails `cargo test`.
15. **Follow-ups (out of scope).** `spawn_sequence` still uses `thread::spawn`, which panics on failure (KS-33-like, `sequence.rs:116`). Re-entry refusals of `with_engine` are not counted (candidate counter for 2.4). Actions still run on the hook thread (KS-08, item 2.1).

## Questions for the orchestrator

1. **Timestamps in UTC (`...Z`) or local time with an offset?** Recommendation: UTC. It is pure and testable, unambiguous across DST, and valid ISO-8601. Local time needs a time-zone lookup per line and makes the conversion untestable without Win32. Non-blocking.
2. **The `Press Enter to exit...` prompt is written with `writeln!` on stderr, not as a log record.** That satisfies the literal rg, and it is interactive console UI like the banner, not a diagnostic. Recommendation: accept. Logging it would put "Press Enter" into the file, and keeping `eprintln!` would fail the acceptance line. Non-blocking.
3. **Accept that the elevated daemon writes and rotates in the user-writable `%LOCALAPPDATA%\key-switch-rs` (risk 6)?** Recommendation: accept for 1.7. It is the owner's location, and UAC is not a security boundary per Microsoft. Record a follow-up: refuse a reparse-point folder or file, or open with `FILE_FLAG_OPEN_REPARSE_POINT` and handle-relative renames, with items 3.3 or 4.1. Non-blocking.
4. **Log panic messages through a panic hook?** Recommendation: not in 1.7, which has no such acceptance line. Add it with 2.4 (watchdog). It needs a synchronous file write in the hook path of a dying process, which D1 deliberately avoids. Non-blocking.
5. **Update `CLAUDE.md` (Layout gains `src/logging/`; the coding principles say diagnostics use `log::*!` and print macros only in the banner) in commit 8?** Recommendation: yes, two lines in commit 8, if the orchestrator allows lane commits to touch `CLAUDE.md`; otherwise the orchestrator does it after the merge. Non-blocking.
6. **COM initialisation for `SHGetKnownFolderPath`/`CoTaskMemFree`.** Neither doc requires `CoInitialize` (unverified beyond that silence). Recommendation: call them without it. `local_app_data_is_an_existing_absolute_folder` runs on a libtest thread with no COM initialised, so CI would catch a failure. Non-blocking.

## Cut readiness checklist

### Spec
- [x] Every cited KS issue read and mapped. 1.7 cites none. Related: KS-08 (D1/D7, M7), KS-33 (D6 counter), KS-20 (D2, M3), KS-22/23 (smoke #8 = M2, `a_reload_error_is_logged_with_a_timestamp`).
- [x] Every book reference re-located (table). None is gone.
- [x] Goal, non-goals and dependants (2.4, 4.2, 4.3) stated.

### Design
- [x] Every decision has a rationale and rejected alternatives (D1-D12).
- [x] Owner decisions respected: `log` plus the in-house sink, no tracing, RON untouched, exact modifiers and repeat defaults untouched, licence of `log` is MIT OR Apache-2.0.
- [x] Thread named for every new path (Threading table). Nothing heavy in the LL callback: counters only; action failures do a format + `try_send`.
- [x] Install, uninstall and timers: unchanged, main thread.
- [x] SendInput sentinel: N/A, no new `SendInput`.
- [x] Sided/unsided modifiers: N/A, no matching change.
- [x] Reload / `swch off`/`on` / foreground / shutdown: risk 13 and R10. Shutdown flush in D10.
- [x] Atomic orderings stated (`AcqRel`). No `static mut`.
- [x] No new module cycle (D12). Engine untouched.

### Win32
- [x] Each API: path, feature checked against `Cargo.toml` (`Win32_System_Com` to add), thread, errors, doc link.
- [x] Elevation and labels considered (risks 6, 8; Q3). Pipe/token: N/A.

### Commits and tests
- [x] Red-first: commits 3 and 6 with exact runtime reds. New-module tests carry a predicted compile red on master (commits 1, 2). Commit 5 cannot be unit-tested (live `SendInput`/spawn failure), which is stated with its review and manual proof.
- [x] Tests use TempDir, an injected console, local counters and a thread-local capture. No hook, no injection, no `%LOCALAPPDATA%` write. Engine scenario tests are N/A (no decision change).
- [x] Real-Win32 checks are daemon runs (M1-M8). No new `#[ignore]` test is needed. The existing ignored `hold_the_daemon_pipe_name_for_a_manual_start` keeps its ignore and gets a longer reason.
- [x] Every commit except 3 and 6 passes the gate. The lock update is in commit 1.
- [x] README/CHANGELOG/default_config.ron land in the commit that makes them true (2, 4, 5, 7). The book is in commit 8.

### Acceptance
- [x] Every acceptance line mapped.
- [x] Every question has a recommendation; none blocks.

## Developer addendum (2026-10-18, branch `feat/file-logging`, HEAD `bb9c80f`)

Implemented against this cut as amended by `rulings.md`. Commits: `8217583` (C1), `95c9b3a` (C2),
`848e95c` (C3, red), `ef3f656` (C4), `783f383` (C5), `eebace9` (C6, red), `0dc008e` (C7), `c21c8f6` (C8),
`bb9c80f` (follow-up, D11).

### Critique, Important
- **I1 (option a): RESOLVED.** `LogLine` keeps one `String` (`{ts} [{TAG}] {target: }{message}`, trailing
  `\r`/`\n` trimmed) and two offsets, built on the caller's thread (`src/logging/sink.rs:45-70`). The writer
  thread expands it: `file_text` repeats `{ts} [{TAG}] ` on every physical line, the target prefix on the first
  only (`sink.rs:74-85`); `console_text` is today's layout, the tag on the first line, continuation lines as
  they are, one final `\n` (`sink.rs:90-98`). The caller's cost is unchanged: one `String`, one `try_send`.
  Pinned by `sink::tests::every_line_of_a_multi_line_record_carries_its_timestamp_and_tag_in_the_file` (replaces
  `a_multi_line_message_is_one_record_with_one_final_newline`; covers a trailing `\n`, `\r\n`, a target prefix)
  and `hook::config_watcher::tests::a_reload_error_is_logged_with_a_timestamp`, which checks every line of the
  record (the `cfg(test)` capture stores the file form, `mod.rs:76-77`, `testing.rs`). `errors.md` 18.1, the
  README "Log file" sample and build-and-run "Log file" follow the rule.
- **I2: RESOLVED.** `logging::init()` is public and console-only until `attach_file` (`mod.rs:108-122`). The
  harness in `methodology.md#validation-harness` calls `logging::init()` first and `logging::flush()` after the
  load (before every exit) and before the missing-file exit, prints `block_original_combo` with `{:?}`, uses
  `config::reload`, and shows the new output. Built under `D:/tmp/ks-orch/p1-1.7-logging/cfgcheck/` (own
  target dir, path dependency on the lane tree) and run on `default_config.ron`, a duplicate-binding file and
  the complete example of `example.md` (still `[INFO] SwitchLanguage: using default combination` and
  `OK: 26 bindings`); outputs in `impl.md`.

### Critique, Minor and notes
- **M1: RESOLVED.** `rotate` stops at the first failed rename (`sink.rs:251-259`, doc says why); pinned by
  `a_rotation_stops_at_the_first_rename_that_fails` (holds `daemon.2.log` with `FILE_SHARE_READ`; `daemon.1.log`
  keeps its content, the line lands in `daemon.log`). A mutation that ignores rename errors fails it.
- **M2: RESOLVED.** The writer writes the file part first, then the console (`writer.rs:186-192`). The remaining
  limit (a held conhost selection delays later records; `flush` gives up after 1 s at exit) is stated in the
  README "Log file" section and in build-and-run "Log file". No console thread (4.2).
- **M3: RESOLVED.** Manual check M7 names `conhost.exe` with QuickEdit on and a control run on `master`
  (`impl.md`, CHANGELOG manual-check line).
- **M4: RESOLVED.** `actions.md` (step errors, `[ERROR]` tags, `key_press_short`), `errors.md` 18.1 layout,
  `architecture.md` startup step 1 (logger first, file after the guard). Widened further: `triggers-conditions.md`
  (the hold-timer warning no longer exists), `structure.md` table row, `overview.md` thread list.
- **N1:** the module doc of `writer.rs` and the `log-writer` row in `architecture.md` say "no I/O; the only lock
  is the channel's own, held for O(1) list work when it wakes the parked writer".
- **N2:** there is no probe. `attach_file` hands `LogFiles` to the writer whenever a writer exists; the first
  line (the start line) creates the folder and the file, and a failure is reported once by the writer's streak
  logic and retried per line. `LogFiles::probe` of the type sketch was not added.
- **N3:** `only_the_startup_banner_prints_outside_src_bin` also rejects any path to the `log` crate in
  `src/hook/keyboard_hook_callback.rs` before its `#[cfg(test)] mod tests` (mutation check: a `log::warn!` there
  is reported as `1 log calls in src/hook/keyboard_hook_callback.rs outside its tests`).
- **N4:** reds of C3 and C6 replay on their parents `95c9b3a` and `783f383`; C1/C2 tests are new capability.
- **N5:** `ipc_server.rs` manual test: the `let path` binding and the print are gone, the instruction is in the
  ignore reason (`ipc_server.rs:367`).
- **Q1-Q6:** UTC (README says so); prompt by `writeln!` on stderr (`main.rs:58`); `gaps.md` rows "Log folder
  hardening" (3.3/4.1) and "Panic messages in the log" (2.4); `CLAUDE.md` untouched; no `CoInitialize`
  (`local_app_data_is_an_existing_absolute_folder` passes on a libtest thread).

### Deviations
- `LogLine::file_text` / `console_text` return a `String` built on the writer thread (I1 needs a second form).
- `attach_file` sends `Attach` through a retrying `try_send` with the 1 s deadline (`writer::attach_on`,
  `SendFailure`), so a full queue at startup does not lose the file; without a writer it returns `Err`, and main
  logs `[WARN] no log file in <folder>: <error>` (a second form of the `no log file` warning).
- The red-first reload test uses an invalid binding (`PressKey("Foo")`) instead of `( bindngs: [] )`, so the
  record has three physical lines and a trailing line break; the misspelled field stays pinned by
  `a_save_with_a_misspelled_field_keeps_the_live_bindings`.
- `Counters` reaches its slots through a `match` with constant indices, so `bump` on the hook thread has no
  runtime bound check.
- Commit 6 was amended once, locally, before any hand-off: clippy `reversed_empty_ranges` on an empty range in
  the scan test. Never pushed; no fix round had started.
