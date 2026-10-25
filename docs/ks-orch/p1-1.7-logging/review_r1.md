# Code review: p1-1.7-logging @ 9b6414d
APPROVED (Critical 0, Important 0, Minor 2)

Reviewed `git diff master...feat/file-logging` (master `1058ddb`). The context pack was written for `bb9c80f`.
While this review ran, the tester added `9b6414d` ("Tests: an empty daemon.log is not rotated by a line longer
than the limit", +26 lines in the test module of `src/logging/sink.rs`). Every production `file:line` below is the
same on `bb9c80f` and `9b6414d`; that commit changes no production code. I read the code through `git show`/`git diff`.
Also read: `rulings.md`, `cut.md` with its developer addendum, `impl.md`.

## Gate: fmt ✅ · clippy ✅
- `bb9c80f` (19:30, before the tester's commit): `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0
  (logs: `D:/tmp/ks-orch/p1-1.7-logging/review_r1_fmt.log`, `review_r1_clippy.log`).
- `9b6414d` (rerun, tree clean): fmt exit 0, clippy exit 0.
- Separate target dir `D:/tmp/ks-orch/p1-1.7-logging/review-target`. `cargo test` was not run (the tester owns it).

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor (optional)

#### m1. A second `init()` starts a throwaway writer that takes the global counters (R3, R4)
**Where**: `src/logging/mod.rs:104-111` (doc + `init`), `src/logging/writer.rs:174-176`  **Confidence**: PLAUSIBLE (the
mechanism follows from the code; nothing in the tree calls `init` twice)
**Problem**: `init` calls `writer::spawn(.., &counters::COUNTERS, ..)` before `log::set_logger`. A later call fails
`set_logger` and returns at `:111`. That drops the only sender of the new channel. The new `log-writer` thread then
takes `RecvTimeoutError::Disconnected`, which runs `self.report_counters()` → `COUNTERS.take()`. The doc says "a later
call ... does nothing".
**Failure**: a tool that calls `logging::init()` twice (the doc recommends `init` for harnesses such as `cfgcheck`)
while a counter is non-zero gets the counts printed once on stderr by the second thread, without a timestamp. They
never reach `daemon.log`, and the real writer's next report misses them. Each extra call also starts a thread for
nothing.
**What to do**: make `init` idempotent before anything is spawned (check whether the logger is already installed, or
use a `Once`/`OnceLock`), or fix the doc. Unreachable in the daemon today.

#### m2. `attach_file` doc says it fails only without a writer
**Where**: `src/logging/mod.rs:124-127` vs `:135-136` and `src/logging/writer.rs:108-111`  **Confidence**: CONFIRMED (read)
**Problem**: "Fails only if there is no writer to hand the file to." But `writer::attach_on` also returns
`SendFailure::TimedOut` when the queue stays full for `FLUSH_TIMEOUT`, and `attach_file` maps that to `Err`.
**Failure**: 1024 records queued behind a stalled console at startup make `attach_file` fail with "the log writer's
queue stayed full". `main` then logs `[WARN] no log file in <dir>: ...` although a writer exists. Only the doc is wrong.
**What to do**: say "fails if there is no writer, or if its queue stays full for 1 s".

## Checklist walked (no reachable failure found)

**Plan / roadmap conformance**
- Spec lines: log file at `%LOCALAPPDATA%\key-switch-rs\daemon.log` (`main.rs:90-107`, `known_folder.rs:18-30`, `mod.rs:152-154`);
  the folder is created on demand (`sink.rs:234-244`); ISO-8601 UTC + level (`sink.rs:120-133`, `:55`); 1 MiB / 3 files
  (`sink.rs:20,24,219-260`); stderr console (`mod.rs:109`); the only print sites left are the banner (verified:
  `git grep -E "println!|eprintln!" feat/file-logging -- src ':!src/bin/**'` → `app.rs:105,106,109,112,114,116`, all in
  `print_welcome`); no `print!`/`eprint!`/`dbg!`/direct `stderr()` outside `logging` and `main.rs`'s two `writeln!`
  (Q2); the hook callback module has no `log::`/`print` match and only bumps counters (`keyboard_hook_callback.rs:262-264,300-303`);
  `swch`, `src/engine`, `src/ipc.rs`, `build.rs`, `.github`, `CLAUDE.md`: empty diff.
- Rulings: Q1 UTC (README says so); Q2 prompt via `writeln!` on `stderr().lock()` (`main.rs:58`); Q3/Q4 `gaps.md` rows;
  Q5 `CLAUDE.md` untouched; Q6 no `CoInitialize`. I1: one `String` + offsets on the caller (`sink.rs:45-68`), and the
  per-line `{ts} [TAG] ` expansion runs on the writer (`sink.rs:74-85`, called at `writer.rs:200`; `testing::record` only
  under `cfg(test)`). Trailing `\r`/`\n` are trimmed (`:61-62`); the console form puts the tag on the first line only
  (`:90-98`). I2: `init` is public and console-only until `attach_file`; harness updated; outputs in `impl.md`. M1: stops at
  the first failed rename (`sink.rs:251-260`), pinned by `a_rotation_stops_at_the_first_rename_that_fails`. M2: file before
  console (`writer.rs:186-192`). N1-N5 done as the addendum states.
- Acceptance tests can fail: `a_reload_error_is_logged_with_a_timestamp` asserts 3 physical lines, each timestamped
  + `[ERROR]`, plus the `[INFO] keeping previous bindings live` line and 1 live binding (`config_watcher.rs:351-398`).
  Rotation is pinned at the real `MAX_FILE_BYTES` (`daemon_log_rotates_at_one_mebibyte`) and at the file count
  (`rotation_keeps_...`, exact names, no `daemon.3.log`). The scan test also rejects `log::` in the callback module
  (N3). The tester's new test pins the `size > 0` guard.
- No test calls `logging::init`/`attach_file`. Every disk test uses `config::testing::TempDir`. `local_app_data_is_an_existing_absolute_folder`
  only reads. `capture` installs `LOGGER` with no writer, so no test thread writes a real file.
- CHANGELOG: entries under `## [Unreleased]` → Added/Changed, citing roadmap 1.7, with manual-check lines (incl. the 1.5
  M1/M1b hand-off). README "Log file", build-and-run "Log file" + console table, errors.md 18.1/18.6, actions.md,
  structure.md, triggers-conditions.md, overview.md, architecture.md and methodology.md (harness with `{:?}` and flushes)
  match the code I read. No stale `✓ ...`, `SendInput dropped events`, `could not start a hold timer` or
  `key-switch-rs: already running` strings are left in `book/src`, README or the template.

**Unsafe (section 2)**: three new blocks, all in `known_folder.rs`, each with a concrete SAFETY comment. `TaskMemString`
is built only after `?` on `Ok`, so `Drop`/`CoTaskMemFree` sees only a shell-allocated pointer, freed once. The `as_wide`
slice is copied before `path` drops. The `Err` path of `SHGetKnownFolderPath` may leak the out-pointer (the wrapper
drops it; cut risk 9, once per process). No new `unsafe` anywhere else. Pre-existing and not counted: the
`PostThreadMessageW` block in `console_ctrl_handler` (`app.rs:248-250`) still has no `// SAFETY:` comment. Its lines are
unchanged; the developer lists it as an open point.

**Atomics (R4)**: only `Counters::bump` `fetch_add(AcqRel)` and `take` `swap(0, AcqRel)` (`counters.rs:73,80`). No `Relaxed`,
no `SeqCst`. `take` is a per-slot RMW, so no bump is lost or counted twice. `MAIN_THREAD_ID` is untouched.

**Hot path (R1, section 5)**: the callback body is unchanged apart from the two counter bumps. `counters::slot` uses
constant indices into `[AtomicU64; 4]`, so it has no bound panic. Action-failure records reached from `carry_out`
(`windows_actions.rs:238,254,280`, `system_function.rs:58,77,89,99,105,114`) cost one `LogLine` `String` built with
`write!` (no `format!`, no indexing; `truncate` at a trimmed boundary), a lock-free `OnceLock::get` and a `try_send`.
`Full` bumps a counter; `Disconnected` falls back to stderr only when the writer is gone. Nothing in
`DaemonLogger::log`/`send_line` can panic. `flush()`/`send_by` (sleeps) are reachable only from main before the
install or after the uninstall, from `fatal`, and from the console-control thread. No `with_engine` closure logs
(`keyboard_hook_callback.rs:192-195,295,302`; `keyboard_hook.rs:236` is in `uninstall`, outside the lock). The engine
is untouched.

**Key names / text (spec)**: every `log::` site (44, listed with `git grep`) interpolates config text (exe, URL, path,
registry/config combos at load or reload), counts, or Win32/io errors. The `vk` is gone from `sequence.rs:190`,
`press_vk` and `SystemFunction::press_key` (now counters). `type_text` logs counts only. The banner (macro `Text`) is
not logged (`app.rs:100-117`). `registry.rs:95` (registry combo) runs only from `config::loader` (load/reload), not from
the hook thread (`get_system_combination` callers: `loader.rs:223` and tests).

**Writer (brief)**: `Flush` is queued behind earlier messages; the writer reports the counters, flushes the console,
then acks on `sync_channel(1)` (never blocks). `flush_on` takes at most 1 s in total (`send_by` deadline + remaining
`recv_timeout`) and returns `true` on `Gone`. The writer has no panic path (`Instant + 60 s`, `format!` of the rotation
names, `get`-based slicing). Its own records bypass the queue, so a report cannot be dropped. Shutdown: flushes at
`main.rs:32,39`, `fatal` `:52`, `app.rs:104` (banner order), `app.rs:257` (ctrl handler; `CTRL_CLOSE_EVENT`
5 s budget). Known lost-line windows: abort, `TerminateProcess`, lines after the final flush, and the main thread's
"Removing keyboard hook..." on a closed console (documented). Degraded mode (no writer): records go to stderr from the
caller, the same as master's `eprintln!` hazard. Streak notices are reported once, and "works again" goes to the file
too. `Attach` resets the streak.

**Win32 / threads (R10, section 4)**: install/uninstall stay on main. The new `log-writer` uses no UI API and needs no
pump. `SHGetKnownFolderPath(None)` on main after the guard (KS-20: a refused start never attaches, `main.rs:27-34`).
`log::info!("Removing keyboard hook...")` between `GetMessageW` and `uninstall` cannot block (`try_send`). No new
`SendInput`. All three builders keep `injected_sentinel()` (R2).

**R3** new statics `LOGGER` (`OnceLock<SyncSender>`) and `COUNTERS` (atomics); no `static mut`, no production `Mutex`/`RefCell`
(the `RefCell`/`Mutex` are in `cfg(test)` `testing.rs`). **R5** no new builders/setters. **R6** `Counter`, `WriteError`,
`SendFailure` have explicit arms. The only `_ =>` is in test code (`testing.rs:76`). **R7/R8** no binding,
sorting or key-identity change. **R9** message macros only, reads only. **Section 7** `log = "0.4"` (MIT OR Apache-2.0),
`Cargo.lock` gains only the `"log"` dependency line (0.4.29 was already locked); `Win32_System_Com` is a feature of an
existing crate. No `#[allow]` added. Commit trailers name Opus (developer) and Sonnet (tester's `9b6414d`).

## Positive
- The caller side is truly non-blocking. File and console I/O, rotation and the per-line I1 expansion all run on one
  owner thread, and file-before-console keeps a held conhost selection off the hook thread (M7 is the right manual
  check).
- The hook-path diagnostics are now counters. The vk-naming messages are gone, and the scan test turns "the callback
  never logs" into a gate check.
- Rotation is handle-free open-append-close, stops at the first failed rename, and is pinned by tests that hold files
  without delete sharing, a real 1 MiB boundary test and an exact-limit `>`-vs-`>=` test.
- `fatal` orders `[FATAL]` before the prompt through the flush ack, and falls back to a direct write when the writer
  is stuck.

## Open questions
- None blocking. Before merge, the manual checks M1-M8 (`impl.md:180-191`) are the only proof of the `main.rs` wiring,
  the ctrl-handler flush and the QuickEdit behaviour (KS-38: no harness for `main`).
