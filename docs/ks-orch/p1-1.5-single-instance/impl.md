# Implementation: p1-1.5-single-instance round 1

Verdict: DONE. Branch `feat/single-instance` (from `master` @ `f0048a3`), HEAD `5725499a272b00209599462c3130f27b0148b1b9`, tree clean. Five commits, in the cut's order C1-C5.

## Items

- **A5-m1 (C1, `545c18f`)**: `src/hook/keyboard_hook_callback.rs` tests only. `GLOBAL_HOOK` :408, `RemappingOnAtEnd` :412, helper `call_keyboard_callback` :432 (K3's closure moved out, with its SAFETY), K3 :527 takes the lock, uses the helper, comment narrowed. New test `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` :582. This pins existing behaviour, so it is green on master. Red shown by mutation before the commit:
  - MU-A (keyboard gate :328-333 deleted): red "with remapping off the keyboard callback waited 5.0145432s for the engine lock". K3 and `disabled_hook_passes_every_event_without_asking_the_engine` stay green, which is the A5-m1 gap.
  - MU-B (foreground gate :370-372 deleted): red "with remapping off the foreground callback waited 5.0132333s for the engine lock".
  - Restored: green.
- **Roadmap 1.5 / KS-20 guard (C2, `32704d0`)**: new `src/system/single_instance.rs` (`INSTANCE_MUTEX_NAME` :24, `InstanceGuard::acquire` :38-64, `Drop` :67-75, `InstanceError` + `Display` + `Error::source` :77-107, `classify_open_error` :115-121, tests T1-T8 :147-260). `src/system/mod.rs` gets `pub mod single_instance;`. `Cargo.toml` gets the `Win32_Security` feature. `Cargo.lock` is unchanged.
  - Red-first: the tests module came before the API: `error[E0433]: cannot find type InstanceGuard / InstanceError`, `error[E0425]: cannot find function classify_open_error` (26 errors), then green 8/8.
  - MU-C (no `ERROR_ALREADY_EXISTS` check): T1 and T2 red, "a second acquisition of a held name must be AlreadyRunning".
  - MU-D (`mem::forget` instead of `drop` on that path): T2 red, "the refused acquisition must not keep the mutex alive: AlreadyRunning".
  - MU-E (`ERROR_ACCESS_DENIED` sorted as `Os`): T4 red "a held name this process may not open must be AlreadyRunning", and T5 red.
  - MU-F (no `SetLastError(ERROR_SUCCESS)`): T7 stays green. This Windows (11, 26200) clears the last error when `CreateMutexW` creates a mutex. The cut predicted this possibility, and it is not a defect.
  - T4 confirms on the real kernel that a mutex this process may not open fails with `ERROR_ACCESS_DENIED` (the DACL path). The cut had marked this as unverified.
- **KS-20 daemon wiring (C3, `9fe63ea`)**: `src/main.rs`. `EXIT_ALREADY_RUNNING` :9, `main` :11-31 (guard first, `AlreadyRunning` gives one stderr line and exit code 2 with no prompt, other errors go to `fatal`), `fatal` :35-44 (the old FATAL body), `run(_instance: InstanceGuard)` :49. There is no automated red: the bin has no harness (`test = false`, KS-38) and running it is forbidden. Covered by T1/T8 and manual checks M1, M1b, M2, M3, M5.
- **D4 split (C4, `e83e977`)**: `src/hook/ipc_server.rs`. `IpcListener` :107, `bind` :115, `IpcListener::serve` :123 (the old thread body, unchanged), `spawn` removed, module doc updated. New test `a_second_listener_on_a_taken_pipe_name_is_refused` :352. `tests/ipc_roundtrip.rs` doc :2-3 and call :86 now go through `bind(..).serve(..)`. `app.rs` keeps warn-and-continue, so behaviour is unchanged.
  - Red-first: `error[E0425]: cannot find function bind in this scope` (lib test) and `... in module ipc_server` (ipc_roundtrip), then green.
  - I1 confirms `raw_os_error() == Some(5)` for a taken name.
- **D4 order and fatal pipe (C5, `5725499`)**: `src/core/app.rs`. `App::run` :133 now returns `Box<dyn Error>` and runs in this order: bind, banner, Ctrl+C, install, serve, watcher, loop, uninstall. A failed `serve` uninstalls on the main thread before returning the error. `pipe_error_message` :221. Tests :308, :323. `src/hook/ipc_server.rs` gets a module-doc sentence and the ignored test `hold_the_daemon_pipe_name_for_a_manual_start` :368.
  - Red-first: `error[E0425]: cannot find function pipe_error_message in this scope`, then green.
  - Mutation (ACCESS_DENIED hint dropped): `a_taken_pipe_name_is_reported_with_its_likely_owners` red with message "could not create the IPC pipe \\.\pipe\key-switch-rs.sock: Access is denied. (os error 5)".
  - MU-G (bind after install) has no automated red, as declared in the cut. Manual check M4 covers it.

## Remarks

The critique has no Critical or Important remark.

- **M1**: RESOLVED. The send result is ignored at `keyboard_hook_callback.rs:636`, with a comment saying why. Pinned by `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`. The measured red is about 5 s: 5.01 s, slightly above 5 s, not 4.99 s.
- **M2**: RESOLVED. The fatal-pipe behaviour has its own `### Changed` bullet in `CHANGELOG.md` (cites roadmap 1.5, KS-20 and KS-40, plus the second-user consequence). The `### Fixed` KS-20 bullet and the README Direct-launch paragraph state only the guard's scope: "The check covers the Windows session the daemon starts in". C5 extends the README sentence with "a second signed-in user's daemon stops at the pipe instead (see above)". The "do not count" sentence was dropped.
- **M3**: RESOLVED by recording, the critique's second option. The comment at the guard call in `src/main.rs:12-16` says this path is checked by hand only. The CHANGELOG KS-20 bullet carries the manual check (`$LASTEXITCODE` is 2). The cut addendum states that lanes 1.7 and 4.2 must re-run M1/M1b. I did not move the mapping into the library: that would be a design change beyond the cut. The roadmap note belongs to the phase book sync.
- **M4**: RESOLVED. SAFETY comments added in `src/core/app.rs`: `SetConsoleCtrlHandler` :116, `GetCurrentThreadId` :134, message loop :198.
- **Open questions 2-4**: answered in the addendum at the end of `cut.md`. The normal-exit window is harmless. T4's squatter creation succeeds; if it ever fails, report it and do not change the SDDL. `CLAUDE.md` Layout is left to the orchestrator.

## Deviations from the cut

No deviation from the D1-D7 design. Wording and detail differences:
- README C3 paragraph and CHANGELOG `### Fixed` KS-20 bullet: the last sentence was reworded per M2. The CHANGELOG bullet also gains a manual-check sentence (M3). The C5 CHANGELOG text is its own `### Changed` bullet instead of being appended to the Fixed bullet (M2). Its wording is close to the cut's and ends with a manual-check sentence.
- The README C5 sentence after the swch section is placed right after "...gives each session its own pipe." (cut: "after :720").
- `acquire` reads `GetLastError()` immediately after `CreateMutexW` and before `map_err`. The order is the same as the cut's; nothing runs in between.
- T5 got an assertion message. T4's squatter `expect` text: "the creator of a new mutex gets a handle whatever its DACL".
- The module doc of `ipc_server.rs`: C4 describes `bind`/`serve` neutrally, and C5 adds the sentence on the order in `App::run`. Each commit's docs are true at that commit.
- `App::run` got a doc comment (none before).

## Unsafe / atomics added

- `src/system/single_instance.rs:45`: SAFETY: "SetLastError only writes this thread's last-error slot."
- `:50`: SAFETY: "`wide` is NUL-terminated and alive across the call, which only reads it. `None` gives the mutex the default security descriptor and a handle that child processes cannot inherit, so a `Launch` child never keeps the name alive (KS-09)."
- `:53`: SAFETY: "GetLastError only reads this thread's last-error slot. It is read right after CreateMutexW, with no call in between."
- `:71`: SAFETY: "`handle` is the result of a successful CreateMutexW, owned by this guard alone and closed only here, once."
- `:203` (test T4): SAFETY: "`wide` is NUL-terminated, and `attributes` and the descriptor it points to are locals of this frame; all three outlive the call, which only reads them."
- `:243` (test T7): SAFETY: "SetLastError only writes this thread's last-error slot."
- `src/hook/keyboard_hook_callback.rs` (tests): `call_keyboard_callback` keeps K3's SAFETY text verbatim. The `foreground_changed` call: "`foreground_changed` reads none of its arguments, so the null handles are never dereferenced or passed on. No hook is installed and no message is pumped here."
- `src/core/app.rs`: :116, :134 and :198 are SAFETY comments on existing blocks (M4).
- Atomics: the test-only `COUNTER` in `single_instance.rs` uses `fetch_add(Relaxed)`. Written justification: it only hands out distinct numbers and publishes no data. No product atomics.
- Test-only `GLOBAL_HOOK: Mutex<()>` serialises the two tests that use `get_hook()`. It guards no hook state.

## Gates

- `cargo fmt --check`: pass at every commit.
- `cargo check --all-targets --locked`: pass.
- `cargo clippy --all-targets --locked -- -D warnings`: pass at every commit, with no warnings. Exposing the `Win32_Security` globs (risk 12) caused no ambiguity.
- `cargo build --locked --bin key-switch-rs` (debug): builds. Not run.
- Tests run at HEAD, 38 passed, 2 ignored: `--lib` filtered to `keyboard_hook_callback`, `single_instance`, `core::app`, `ipc_server` and `ipc::tests`. `--test ipc_roundtrip`: 6 passed. The full suite is left to the tester.
- Manual ignore added: `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start` (M4). A human runs `cargo test --locked -- --ignored --nocapture hold_the_daemon_pipe_name_for_a_manual_start` and, within 60 s, starts the release daemon elevated. Expected: `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: Access is denied. (os error 5). Another program owns that pipe name: ...`, no banner, no hook, CapsLock unchanged, exit code 1 after Enter.
- Manual checks, human only, from the cut: M1, M1b, M2, M3, M4, M5, and optionally M6.

## Docs

- `README.md`: tree lines (`single_instance.rs`, and `main.rs` gains "single-instance guard"), the `Win32_Security` feature line, a Direct-launch paragraph on the one-daemon guard and exit code 2, an `app.rs` bullet, and a sentence after the swch pipe paragraph on the fatal pipe.
- `CHANGELOG.md` `[Unreleased]`: a `### Fixed` KS-20 bullet (guard, code 2, manual check), a `### Changed` bullet (fatal pipe, KS-40 consequence, manual check), and the KS-20 parenthetical in the earlier `swch open` bullet edited.
- `default_config.ron` and `book/src/config/*.md`: unchanged, since config semantics do not change (D6).
- Book pages outside `config/` are stale and go to the phase book sync (cut Q3). Confirmed by grep: `architecture.md:99`, `:161` (`ipc_server::spawn`, warn-and-continue); `build-and-run.md:125`, `:441` (the removed `[WARN]`/`[INFO]` lines); and the rest of the cut's Touch-set list (`audit.md:43`, `:499`, `roadmap.md:220`, `:271-279`, `:624`, `overview.md`, `glossary.md:221`, `gaps.md:225`).

## Commits

- `545c18f` Tests: pin the hook callbacks' remapping-off gate before the engine lock (A5-m1)
- `32704d0` System: single-instance guard on a named mutex (1.5, KS-20)
- `9fe63ea` Daemon: a second start in the same session prints "already running" and exits with code 2 (1.5, KS-20)
- `e83e977` IPC: create the pipe apart from starting its server (1.5, KS-20)
- `5725499` App: create the pipe before installing the hooks; a pipe that cannot be created is fatal (1.5, KS-20)

## Open points / pre-existing warnings

- Commit message of `5725499`: the shell collapsed `\\` in the quoted path, so the body reads `\.\pipe\key-switch-rs.sock` where the program prints `\\.\pipe\key-switch-rs.sock`. The code, README, CHANGELOG and tests carry the correct path. Not amended (house rule: follow-up commits only).
- `src/core/app.rs` `console_ctrl_handler` (its inner `unsafe` around `PostThreadMessageW`) has no SAFETY comment. It is pre-existing and untouched by this lane.
- Pre-existing behaviour noted in cut risk 9 is unchanged: if `uninstall` fails at shutdown, the FATAL prompt waits while the `ipc-server` thread still serves.
- No pre-existing clippy or rustc warnings.
- For the orchestrator: the `CLAUDE.md` Layout line for `src/system/` should list `single_instance.rs` (critique open question 4). Agents do not edit `CLAUDE.md`.
