# Context pack r1: p1-1.5-single-instance (roadmap 1.5, KS-20, A5-m1)

Readers: tester, code-reviewer. Facts only; every `path:line` is at `feat/single-instance` @ `5725499` unless marked `master:`. Inferred items are marked UNVERIFIED. The scout ran no gate and no test.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, current branch `feat/single-instance`, HEAD `5725499`. `git merge-base master feat/single-instance` = `f0048a3` = `master` tip ("Merge branch 'fix/language-hotkey-tristate'").
- `git log --oneline master..feat/single-instance` (newest first):
  - `5725499` App: create the pipe before installing the hooks; a pipe that cannot be created is fatal (1.5, KS-20)
  - `e83e977` IPC: create the pipe apart from starting its server (1.5, KS-20)
  - `9fe63ea` Daemon: a second start in the same session prints "already running" and exits with code 2 (1.5, KS-20)
  - `32704d0` System: single-instance guard on a named mutex (1.5, KS-20)
  - `545c18f` Tests: pin the hook callbacks' remapping-off gate before the engine lock (A5-m1)
- `git diff --stat master..feat/single-instance`: 10 files, +648 / -98.

| File | +/- | Commit(s) |
|---|---|---|
| `CHANGELOG.md` | 29 | 9fe63ea, 5725499 |
| `Cargo.toml` | +1 | 32704d0 |
| `README.md` | 16 | 32704d0, 9fe63ea, 5725499 |
| `src/core/app.rs` | 100 | e83e977 (2 lines), 5725499 |
| `src/hook/ipc_server.rs` | 99 | e83e977, 5725499 |
| `src/hook/keyboard_hook_callback.rs` | 180 (+141/-39) | 545c18f (tests module only) |
| `src/main.rs` | 51 | 9fe63ea |
| `src/system/mod.rs` | +1 | 32704d0 |
| `src/system/single_instance.rs` (new) | +260 | 32704d0 |
| `tests/ipc_roundtrip.rs` | 9 | e83e977 |

- `git diff --quiet master..feat/single-instance -- src/bin/swch.rs src/ipc.rs src/engine src/hook/keyboard_hook.rs src/config Cargo.lock build.rs app.manifest book` exits 0: all byte-identical to `master`.
- Lane inputs: `D:/tmp/ks-orch/p1-1.5-single-instance/cut.md` (473 lines), `critique.md` (72 lines), `impl.md` (91 lines).

## File map

| path:range | What lives there | Why it matters this round |
|---|---|---|
| `src/system/single_instance.rs:1-24` | module doc; imports (`CloseHandle`, `ERROR_ACCESS_DENIED`, `ERROR_ALREADY_EXISTS`, `ERROR_SUCCESS`, `GetLastError`, `HANDLE`, `SetLastError`, `CreateMutexW`, `PCWSTR`); `INSTANCE_MUTEX_NAME` :24 | the spec's mutex name |
| `src/system/single_instance.rs:26-65` | `InstanceGuard { handle: HANDLE }` (`#[must_use]`, `#[derive(Debug)]`), `acquire(name: &str)` | injectable name; last-error handling; close on ALREADY_EXISTS |
| `src/system/single_instance.rs:67-75` | `Drop` -> `CloseHandle`, result ignored | handle lifetime (R9-like resource rule, CLAUDE.md principle 7) |
| `src/system/single_instance.rs:77-121` | `InstanceError { AlreadyRunning, Os(windows::core::Error) }`, `Display` :87-98, `Error::source` :100-107, `classify_open_error` :115-121 | R6 Display completeness; D2 classification |
| `src/system/single_instance.rs:123-260` | tests T1-T8, `unique_name` :140-145, `COUNTER` :136 | acceptance tests |
| `src/system/mod.rs:2` | `pub mod single_instance;` | module registration |
| `Cargo.toml:51` | `"Win32_Security",` added to `windows` features | D7; `Cargo.lock` unchanged |
| `src/main.rs:1-65` | `EXIT_ALREADY_RUNNING` :9, `main` :11-32, `fatal` :35-44, `run(_instance)` :49-65 | guard is the first statement; exit code 2; guard drop timing |
| `src/core/app.rs:125-214` | `App::run` (new order, new error type) | R10 startup/shutdown order; fatal pipe |
| `src/core/app.rs:217-236` | `pipe_error_message` | fatal text; ACCESS_DENIED hint |
| `src/core/app.rs:238-254` | `console_ctrl_handler` (unchanged) | inner `unsafe` :245-247 has no SAFETY comment (pre-existing) |
| `src/core/app.rs:307-330` | tests `a_taken_pipe_name_is_reported_with_its_likely_owners`, `other_pipe_errors_are_reported_as_they_are` | C5 tests |
| `src/hook/ipc_server.rs:1-6` | module doc (bind/serve, order in `App::run`) | doc vs code |
| `src/hook/ipc_server.rs:95-145` | `create_listener` :97-103 (unchanged), `IpcListener` :105-109, `bind` :111-117, `IpcListener::serve` :119-145 | D4 split; `spawn` removed |
| `src/hook/ipc_server.rs:351-379` | test `a_second_listener_on_a_taken_pipe_name_is_refused` :352, ignored `hold_the_daemon_pipe_name_for_a_manual_start` :366-379 | I1 and manual M4 |
| `tests/ipc_roundtrip.rs:1-3, 86-89` | doc and the round-trip test now use `bind(..).serve(..)` | `spawn` caller updated |
| `src/hook/keyboard_hook_callback.rs:315-382` | `keyboard_hook_callback` (gate :328-333), `foreground_changed` (gate :370-372) | product code unchanged; the gates the A5-m1 test pins |
| `src/hook/keyboard_hook_callback.rs:384-450` | test imports :386-387 (new `Mutex, PoisonError, mpsc`, `Duration`), module comment, `GLOBAL_HOOK` :408, `RemappingOnAtEnd` :412-418, `call_keyboard_callback` :432-450 | A5-m1 test scaffolding |
| `src/hook/keyboard_hook_callback.rs:526-579` | K3 `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` (now takes `GLOBAL_HOOK`, uses helper) | changed existing test |
| `src/hook/keyboard_hook_callback.rs:581-649` | new `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` | A5-m1 pin |
| `src/hook/keyboard_hook.rs:95-103, 125-156, 162-212, 226-` | `with_engine`, `is_enabled`, `set_enabled`, `install`, `uninstall` (unchanged) | what the tests and `App::run` call |
| `CHANGELOG.md:238-250` | `### Changed` bullet: fatal pipe (1.5, KS-20, KS-40) | docs |
| `CHANGELOG.md:286-287` | edited parenthetical in the earlier `swch open` bullet | docs |
| `CHANGELOG.md:451-463` | `### Fixed` KS-20 bullet: guard, exit code 2, manual check | docs |
| `README.md:65, 67, 133, 723-726, 734-739, 760` | tree lines, `app.rs` bullet, fatal-pipe sentence, Direct-launch paragraph, feature line | docs |

## Facts

### Guard (`src/system/single_instance.rs`)

1. `INSTANCE_MUTEX_NAME` is `r"Local\key-switch-rs"` (`single_instance.rs:24`).
2. `acquire` takes the name as a `&str` parameter (`single_instance.rs:38`); the only product call passes `INSTANCE_MUTEX_NAME` (`src/main.rs:17`).
3. `acquire` builds a NUL-terminated `Vec<u16>` (`single_instance.rs:39`), calls `SetLastError(ERROR_SUCCESS)` (`:45`), `CreateMutexW(None, false, PCWSTR(wide.as_ptr()))` (`:50`), then `GetLastError()` (`:53`) before `map_err` (`:55`).
4. On `Err` from `CreateMutexW`, `classify_open_error` maps `ERROR_ACCESS_DENIED.to_hresult()` to `AlreadyRunning`, every other code to `Os(error)` (`single_instance.rs:115-121`).
5. On `Ok` with `last_error == ERROR_ALREADY_EXISTS`, the handle is wrapped in a guard, dropped (`CloseHandle`), and `Err(AlreadyRunning)` is returned (`single_instance.rs:54-62`).
6. `bInitialOwner` is `false` and the mutex is never waited on (`single_instance.rs:50`; no `WaitForSingleObject` in the file).
7. `Drop` calls `CloseHandle(self.handle)` and ignores the result (`single_instance.rs:67-75`).
8. Every `unsafe` block in the file has a `// SAFETY:` comment: `:44-45`, `:46-50`, `:51-53`, `:69-73`, `:200-203` (T4), `:242-243` (T7).
9. `InstanceError` `Display` has one explicit arm per variant, no `_ =>` (`single_instance.rs:89-96`); texts: AlreadyRunning = "already running in this session; use `swch status`, `swch off` or `swch exit` to control it" (`:91`), Os = "could not create the single-instance mutex: {error}" (`:94`).
10. `Error::source`: `AlreadyRunning` -> `None`, `Os(e)` -> `Some(e)` (`single_instance.rs:100-107`).
11. Test names come from `format!(r"Local\key-switch-rs-test-{}-{tag}-{n}", std::process::id())` with `COUNTER.fetch_add(1, Ordering::Relaxed)` and a written justification comment (`single_instance.rs:136-145`). No test passes `INSTANCE_MUTEX_NAME` to `acquire` (T6 only compares the constant, `:236-238`).
12. T4 creates the squatter with SDDL `"D:P(A;;GA;;;SY)"` via `interprocess` `SecurityDescriptor::deserialize` and a `SECURITY_ATTRIBUTES` local (`single_instance.rs:189-205`); `interprocess` is a normal dependency (`Cargo.toml:57`).
13. `Cargo.lock` is unchanged on the branch (diff check above); `Win32_Security` is the only `Cargo.toml` change (`Cargo.toml:51`).

### Daemon entry (`src/main.rs`)

14. `InstanceGuard::acquire(INSTANCE_MUTEX_NAME)` is the first statement of `main` (`main.rs:17`), before `ConfigPath::from_arg` (`:55`) and `config::load` (`:57`), which are inside `run`.
15. `Err(AlreadyRunning)` prints `eprintln!("key-switch-rs: {}", InstanceError::AlreadyRunning)` and returns `ExitCode::from(EXIT_ALREADY_RUNNING)` = 2, with no stdin read (`main.rs:19-25`, `:9`).
16. `Err(Os(..))` goes to `fatal` (`main.rs:26`), which prints `[FATAL] {error}`, "Press Enter to exit...", reads one stdin line and returns `ExitCode::FAILURE` (`main.rs:35-44`).
17. The guard is the parameter binding `_instance` of `run` (`main.rs:49`), so it is dropped when `run` returns, before `fatal` is entered from `main.rs:30`.
18. The `[[bin]]` for the daemon has `test = false` (`Cargo.toml:30-33`, comment `:18-22` cites KS-38), so `main.rs` has no test harness.

### Startup order and fatal pipe (`src/core/app.rs`)

19. `master:src/core/app.rs` order: `hook.install()?` at `:131`, `ipc_server::spawn` at `:139`, warn lines `[WARN] could not create IPC listener` `:143` and `[INFO] another daemon instance may already be running` `:148`; `run` returned `windows::core::Result<()>` (`master:src/core/app.rs:121`).
20. Branch `App::run` returns `std::result::Result<(), Box<dyn std::error::Error>>` (`app.rs:133`); its only caller is `src/main.rs:63` (`app.run()?`).
21. Branch order in `App::run`: `GetCurrentThreadId` + `MAIN_THREAD_ID.store(.., Release)` `:136-137`; `ipc_server::bind(ipc::PIPE_NAME)` with `map_err(pipe_error_message)` and `?` `:141-142`; `print_welcome` `:144`; `setup_ctrl_c_handler()?` `:145`; `update_bindings` `:147-149`; `hook.install()?` `:153`; `listener.serve(control)` `:164`; watcher `:179-196`; `GetMessageW` loop `:202-208`; `hook.uninstall()?` `:211`.
22. On `serve` error: `hook.uninstall()` on the same (main) thread, a failure printed as `[WARN] could not remove the keyboard hook: {error}`, then `Err(format!("could not start the IPC server thread: {error}").into())` (`app.rs:164-170`).
23. On `install` error the function returns via `?` (`app.rs:153`); `listener` (an `IpcListener` local) is dropped on that return (comment `app.rs:150-152`). `install` itself removes the keyboard hook if `SetWinEventHook` fails (`keyboard_hook.rs:202-210`).
24. On `setup_ctrl_c_handler` error the function returns via `?` after the banner and before `install` (`app.rs:144-145`).
25. `pipe_error_message` always yields `could not create the IPC pipe {pipe_path}: {error}` and, when `raw_os_error()` converts to `WIN32_ERROR(5)` = `ERROR_ACCESS_DENIED`, appends `. Another program owns that pipe name: the daemon of another signed-in user (one pipe serves every session until roadmap item 4.1) or a key-switch-rs started before the single-instance guard` (`app.rs:221-236`).
26. `ipc::pipe_path(name)` is `format!(r"\\.\pipe\{pipe_name}")` (`src/ipc.rs:41-43`); `ipc::PIPE_NAME` is `"key-switch-rs.sock"` (`src/ipc.rs:33`).
27. New SAFETY comments on existing blocks: `SetConsoleCtrlHandler` `app.rs:116-118`, `GetCurrentThreadId` `:134-135`, message loop `:198-201`. `console_ctrl_handler`'s `unsafe` around `PostThreadMessageW` (`app.rs:245-247`) has no SAFETY comment (unchanged from master).
28. The old `[WARN] could not create IPC listener` / `[INFO] another daemon instance...` strings no longer occur: grep of both over `src` and `tests` finds no match.

### IPC server (`src/hook/ipc_server.rs`)

29. `pub fn spawn` is removed; `grep "ipc_server::spawn|fn spawn("` over `src` and `tests` finds no match.
30. `bind(pipe_name)` = `create_listener(pipe_name).map(|listener| IpcListener { listener })`, starts no thread (`ipc_server.rs:115-117`). `create_listener` (`:97-103`) is unchanged and ends in `.create_sync()` (`:102`).
31. `IpcListener` is `#[must_use]` and has one private field `listener: Listener` (`ipc_server.rs:105-109`); no `Debug` derive.
32. `IpcListener::serve(self, control)` spawns thread `ipc-server` with the previous accept-loop body (per-connection `thread::spawn(handle_connection)`, `[WARN] IPC accept error`) and returns `io::Result<()>` (`ipc_server.rs:123-145`).
33. Test I1 binds `format!("key-switch-rs-unit-{}-taken.sock", std::process::id())` twice and asserts `raw_os_error() == Some(5)` on the second (`ipc_server.rs:351-364`).
34. Ignored test `hold_the_daemon_pipe_name_for_a_manual_start` binds the real `crate::ipc::PIPE_NAME` and sleeps 60 s; its attribute is `#[ignore = "manual: holds the daemon's pipe for 60 s; ..."]` (`ipc_server.rs:366-379`).
35. `PIPE_SDDL` and `pipe_security` are unchanged (`ipc_server.rs:64-85`).

### A5-m1 test (`src/hook/keyboard_hook_callback.rs`)

36. Product code of the file is unchanged: the diff's first hunk starts at the test module imports (`keyboard_hook_callback.rs:386-387`).
37. Keyboard gate: `if !hook.is_enabled() { return CallNextHookEx(..) }` at `keyboard_hook_callback.rs:328-333`, before the `KBDLLHOOKSTRUCT` read (`:338`) and `decide_event` (`:339`), which calls `hook.with_engine` (`:191`) and re-checks `is_enabled` under the lock (`:192`).
38. Foreground gate: `if !hook.is_enabled() { return; }` at `keyboard_hook_callback.rs:370-372`, before `with_engine` (`:376`) and the re-check (`:377`).
39. `with_engine` blocks on `self.engine.lock()` unless the calling thread is already deciding (`keyboard_hook.rs:95-103`); `set_enabled` itself goes through `with_engine` (`keyboard_hook.rs:146-156`).
40. `GLOBAL_HOOK: Mutex<()>` (`keyboard_hook_callback.rs:408`) is taken with `.lock().unwrap_or_else(PoisonError::into_inner)` by K3 (`:536`) and the new test (`:587`).
41. `RemappingOnAtEnd::drop` calls `let _ = get_hook().set_enabled(true)` (`keyboard_hook_callback.rs:412-418`); in the new test it is declared after the `GLOBAL_HOOK` guard (`:587-588`).
42. New test flow: bind F13 -> `DoNothing` with `allow_extra_modifiers` (`:589-592`); `set_enabled(false).expect(..)` (`:593-595`); holder thread holds `with_engine` and waits `release_rx.recv_timeout(5 s)` (`:601-606`); main waits `locked_rx.recv_timeout(5 s)` (`:607-609`); times F13 down+up through `call_keyboard_callback` (`:611-615`) and one `foreground_changed` call (`:617-631`); `let _ = release_tx.send(())` and `holder.join()` (`:635-636`); asserts both durations `< 1 s` and both results `!= LRESULT(1)` (`:638-648`).
43. `call_keyboard_callback` calls `keyboard_hook_callback(0, WPARAM(msg), LPARAM(&info))` with a stack `KBDLLHOOKSTRUCT { vkCode, ..Default }` (`keyboard_hook_callback.rs:432-450`); `dwExtraInfo` is 0 (default), so the event is not the sentinel.
44. K3's remapping-off asserts are at `keyboard_hook_callback.rs:563-569`; its comment says they hold with or without the gate (`:528-535`).

### Docs

45. `CHANGELOG.md:238-250` (`### Changed`): fatal pipe, KS-40 second-user consequence, manual check via the ignored test. `CHANGELOG.md:451-463` (`### Fixed`): KS-20 guard, exit code 2, no Enter prompt, "The check covers the Windows session the daemon starts in", manual check `$LASTEXITCODE` = 2. `CHANGELOG.md:286-287`: "(KS-20; the single-instance guard below covers the other ways to start a second daemon in a session)".
46. `README.md:734-739`: one daemon per session, the already-running line, exit code 2 before the config is read, no Enter wait, second signed-in user stops at the pipe. `README.md:723-726`: the `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: ...` sentence. `README.md:133`: `app.rs` bullet. `README.md:65, 67`: tree lines. `README.md:760`: `Win32_Security` feature line.
47. `book/` is unchanged on the branch; stale book locations listed by impl.md: `architecture.md:99`, `:161`; `build-and-run.md:125`, `:441`; plus cut Touch-set list (`cut.md:283`). `book/src/audit.md:43` still reads "Partly fixed (`a1b5a27`); guard: item 1.5".
48. Roadmap 1.5 acceptance: "A second acquisition returns `Err(AlreadyRunning)`; a second start prints \"already running\"" (`book/src/roadmap.md:279`).

### Unchanged A5 `swch open` handling (spec: "stays")

49. `src/ipc.rs:146` maps `ERROR_ACCESS_DENIED` -> `RequestError::AccessDenied`; `:149` maps `ERROR_PIPE_BUSY | ERROR_SEM_TIMEOUT` -> `RequestError::Busy`. Tests: `ipc::tests::connect_errors_are_classified` (`src/ipc.rs:273`), `only_a_missing_pipe_means_no_daemon` (`:302`); `tests/ipc_roundtrip.rs:163` `request_reports_access_denied_for_a_pipe_this_process_may_not_open`, `:181` `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running`.

## Red-first tests and acceptance proofs (from cut.md / impl.md)

Commands (gate, per CLAUDE.md): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Never `--ignored`/`--include-ignored`.

| ID | Test (path) | Filter command | Expected now | Recorded red (source) |
|---|---|---|---|---|
| A5-m1 | `hook::keyboard_hook_callback::tests::remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` | `cargo test --locked --lib remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` | pass (pins existing behaviour; green on master too, `cut.md:293`) | MU-A delete `keyboard_hook_callback.rs:328-333`: "with remapping off the keyboard callback waited 5.0145432s for the engine lock"; MU-B delete `:370-372`: "... foreground callback waited 5.0132333s ..." (impl.md Items). K3 and `disabled_hook_passes_every_event_without_asking_the_engine` stay green under both |
| K3 | `..::extern_callback_swallows_a_bound_key_only_while_remapping_is_on` | `cargo test --locked --lib extern_callback_swallows` | pass | changed, not new |
| T1 | `system::single_instance::tests::a_second_acquisition_of_a_held_name_is_already_running` | `cargo test --locked --lib single_instance` (runs T1-T8) | pass | compile red on master: E0433/E0425 for `InstanceGuard`, `InstanceError`, `classify_open_error` (26 errors); MU-C (no ALREADY_EXISTS check): T1, T2 red "a second acquisition of a held name must be AlreadyRunning" |
| T2 | `..::the_name_is_free_again_once_every_guard_is_dropped` | same | pass | MU-D (`mem::forget` on that path): "the refused acquisition must not keep the mutex alive: AlreadyRunning" |
| T3 | `..::distinct_names_do_not_conflict` | same | pass | - |
| T4 | `..::a_mutex_this_process_may_not_open_counts_as_already_running` | same | pass (ERROR_ACCESS_DENIED path confirmed on this machine, impl.md) | MU-E (ACCESS_DENIED -> Os): T4 red "a held name this process may not open must be AlreadyRunning", T5 red |
| T5 | `..::only_access_denied_among_open_errors_means_already_running` | same | pass | MU-E |
| T6 | `..::the_daemon_mutex_is_local_key_switch_rs` | same | pass | - |
| T7 | `..::a_stale_already_exists_error_does_not_make_a_fresh_name_look_held` | same | pass | MU-F (no `SetLastError`): stays green on Windows 11 26200 (CreateMutexW clears the last error on creation); cut predicted either outcome, "not a defect either way" (`cut.md:314`) |
| T8 | `..::already_running_names_the_swch_commands` | same | pass | - |
| I1 | `hook::ipc_server::tests::a_second_listener_on_a_taken_pipe_name_is_refused` | `cargo test --locked --lib a_second_listener_on_a_taken_pipe_name` | pass, `raw_os_error() == Some(5)` | compile red: E0425 `bind` (lib) and in `ipc_roundtrip` |
| RT | `tests/ipc_roundtrip.rs::status_off_on_status_round_trip_without_a_hook` | `cargo test --locked --test ipc_roundtrip` (6 tests) | 6 passed | compile red E0425 `bind` in module `ipc_server` |
| P1 | `core::app::tests::a_taken_pipe_name_is_reported_with_its_likely_owners` | `cargo test --locked --lib core::app` | pass | compile red E0425 `pipe_error_message`; mutation (hint dropped): red with "could not create the IPC pipe \\.\pipe\key-switch-rs.sock: Access is denied. (os error 5)" |
| P2 | `core::app::tests::other_pipe_errors_are_reported_as_they_are` | same | pass | compile red E0425 |
| A5 | `ipc::tests::connect_errors_are_classified`, `only_a_missing_pipe_means_no_daemon`, `ipc_roundtrip` access-denied / busy tests | `cargo test --locked --lib ipc::tests`; `--test ipc_roundtrip` | pass, unchanged | - |

Developer-reported run at HEAD (impl.md "Gates"): filtered `--lib` (keyboard_hook_callback, single_instance, core::app, ipc_server, ipc::tests) 38 passed, 2 ignored; `--test ipc_roundtrip` 6 passed; fmt and clippy `-D warnings` pass at every commit. Full `cargo test --locked` not run by the developer ("left to the tester").

No automated red exists for: the `main.rs` code-2 path (no harness, KS-38; `cut.md:318`); the bind-before-install order, MU-G (`cut.md:347`). Both are covered by manual checks only.

### Manual checks (human only; agents never run these; `cut.md:378-390`)

| # | Steps | Expected |
|---|---|---|
| M1 | daemon running via `swch open`; elevated PowerShell `& .\target\release\key-switch-rs.exe; $LASTEXITCODE` | one line `key-switch-rs: already running in this session; use ...`, then `2`; no banner, no prompt; one process; `swch status` = running |
| M1b | as M1 with `C:\does\not\exist.ron` argument | same line and `2`, not the config FATAL |
| M2 | double-click exe while running, accept UAC | console opens and closes; still one daemon |
| M3 | release daemon running; debug build from a normal console; then reverse | already-running line and `2` (ACCESS_DENIED across integrity levels); reverse: release start closes, debug daemon remains |
| M4 | console A: `cargo test --locked -- --ignored --nocapture hold_the_daemon_pipe_name_for_a_manual_start`; within 60 s start release daemon elevated | `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: Access is denied. (os error 5). Another program owns that pipe name: ...`, no banner, no hook, CapsLock unchanged, exit code 1 after Enter |
| M5 | `swch exit` then `swch open`; End task then `swch open` | both restarts work |
| M6 (optional) | two signed-in users; user B starts daemon | B gets the M4 FATAL line (KS-40) |

## Open points (from impl.md "Open points / pre-existing warnings" and Remarks)

1. Commit message body of `5725499` reads `\.\pipe\key-switch-rs.sock` (shell collapsed `\\`); code, README, CHANGELOG, tests carry `\\.\pipe\...`. Not amended (house rule).
2. `console_ctrl_handler` inner `unsafe` (`app.rs:245-247`) has no SAFETY comment; pre-existing, untouched.
3. Pre-existing (cut risk 9, `cut.md:407`): if `uninstall` fails at shutdown (`app.rs:211`), the FATAL prompt waits while the `ipc-server` thread still serves and the guard is already released.
4. Normal-exit window (cut addendum, `cut.md:469`): the guard is dropped when `main::run` returns, before process exit closes the pipe; a start in that window passes the guard and stops at `bind` with the pipe FATAL, no hook.
5. `swch open` race (`cut.md:412`): between a daemon's guard and its `bind`, `swch open` sees no pipe and launches a second daemon that exits 2 while `swch` prints `daemon launched` (follow-up F2).
6. `CLAUDE.md` Layout line for `src/system/` does not list `single_instance.rs`; left to the orchestrator.
7. Book pages outside `config/` are stale (Fact 47); left to the phase book sync (`cut.md:425`).
8. Lanes 1.7 and 4.2 must re-run M1/M1b after touching `main.rs` (`cut.md:467`).
9. Critique (`critique.md`) has no Critical or Important remark; Minors M1-M4 reported RESOLVED in impl.md.

## Invariants in play

- **R1 Hook callback**: product callback unchanged (Fact 36); gates `keyboard_hook_callback.rs:328-333`, `:370-372`; `code < 0` pass `:319-324`. The new test calls the extern callbacks with no hook installed (`:432-450`, `:620-630`).
- **R2 Sentinel**: no `SendInput` added on the branch (no diff in `sequence.rs`, `system_function.rs`; callback product code unchanged).
- **R3 Shared state**: `with_engine` unchanged (`keyboard_hook.rs:95-103`); test-only `GLOBAL_HOOK` (`keyboard_hook_callback.rs:408`); `InstanceGuard` is a local of `main`/`run`, no static (`main.rs:17`, `:49`).
- **R4 Orderings**: `MAIN_THREAD_ID.store(.., Release)` `app.rs:137`, load `Acquire` `:243`; only new atomic is test `COUNTER` `Relaxed` with comment (`single_instance.rs:141-143`).
- **R5 Builders**: `App` builder unchanged (`app.rs:76-98`).
- **R6 Display**: `InstanceError` arms `single_instance.rs:89-96`.
- **R9 Handles**: mutex `CloseHandle` in `Drop` (`single_instance.rs:67-75`), early close on ALREADY_EXISTS (`:57-61`); wide string NUL-terminated (`:39`).
- **R10 Startup/shutdown**: order Fact 21; exit paths Facts 22-24; `uninstall` on serve failure runs on the installing thread (`app.rs:165-166`).
- **KS-20**: `book/src/audit.md:43`; roadmap `book/src/roadmap.md:270-279`. **KS-40** (one pipe across sessions): CHANGELOG `:238-250`, README `:723-726`. **KS-09** (handle inheritance): `single_instance.rs:46-49` (`None` attributes). **KS-38** (bin harness): `Cargo.toml:18-33`. **KS-01**: `app.rs:165`, `:200-201`.

## Not covered

- The scout ran no gate, no test and no build; all pass/red results above are as reported in `impl.md`.
- Not read: `src/engine/**`, `src/data/**`, `src/config/**`, `src/bin/swch.rs` (verified unchanged only), `keyboard_hook.rs` beyond `:90-240`, `ipc_server.rs:147-350` beyond item locations, `tests/ipc_roundtrip.rs` beyond grep and the diff.
- Not verified on Microsoft Learn this round: CreateMutexW / CreateNamedPipeW quotes are taken from `cut.md:248-254`.
- Book pages (stale ones listed in Fact 47) not read in full.
- `critique.md` read only for headings.
