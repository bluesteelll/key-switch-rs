# Tester: p1-1.5-single-instance / roadmap 1.5 (KS-20, A5-m1), round 1

HEAD tested: `5725499` (developer HEAD), then `2a91ec6` (HEAD after the tester's own `Tests:` commit). Parent/base: `master` = `f0048a3`.
Work tree `D:/key-switch-rs/key-switch-rs` was clean before and after (`git status --short` empty); `git worktree list` shows only the lane tree.

Verdict: GREEN. Gate green, every red-first accounted for (compile-red ones carried by mutations), 11 own mutations (9 caught, 1 equivalent on this OS, 1 catchable only by hand), every acceptance criterion mapped. Gaps listed under Notes are manual-only by design.

## Gate (HEAD)

All three cargo steps ran with `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.5-single-instance/target-wt/head` (see Notes: the lane's own `target/` is broken).

- `cargo fmt --check`: OK (also after my commit).
- `cargo clippy --all-targets --locked -- -D warnings`: OK on a fresh target dir (full `Checking` of all crates, "Finished `dev` profile ... in 25.09s"); re-run after my commit: OK.
- `cargo test --locked --no-fail-fast` on `5725499`: exit 0.
  - `running 495 tests` -> `test result: ok. 491 passed; 0 failed; 4 ignored`
  - swch: `running 0 tests` (the bin has no tests, pre-existing)
  - `ipc_roundtrip`: `running 6 tests` -> `ok. 6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests` (no doctests, pre-existing)
- `cargo test --locked --no-fail-fast` on `2a91ec6` (after my test): `running 496 tests` -> `ok. 492 passed; 0 failed; 4 ignored`; `ipc_roundtrip` 6 passed. Logs: `D:/tmp/ks-orch/p1-1.5-single-instance/gate_test_head.log`, `gate_test_head2.log`.
- `cargo build --release --locked`: OK ("Finished `release` profile [optimized] target(s) in 34.17s"). Not run.
- Ignored (4, all `manual:`): `data::condition::tests::foreground_context_names_the_focused_notepad`, `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start` (new in this lane), `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. None was run.
- Flake check: `single_instance`, `remapping_off_callbacks`, `a_second_listener`, `a_taken_pipe` filters run 5 times: `12 passed; 0 failed` x5 (`running 12 tests`). `ipc::tests`: `running 11 tests`, ok.

## Red-first replay (master `f0048a3` in scratch worktree `wt/master`, own target `target-wt/master`)

| Test | master | HEAD | Note |
|---|---|---|---|
| `hook::keyboard_hook_callback::tests::remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` (A5-m1) | ok (HEAD's callback file copied onto master; `running 1 test`, `1 passed`) | ok | Pins existing behaviour, green on master by design (cut.md:293). Red only under mutation: M4a/M4b below |
| `tests/ipc_roundtrip.rs` (HEAD version) `status_off_on_status_round_trip_without_a_hook` | compile-red: `error[E0425]: cannot find function bind in module ipc_server` | ok (6 passed) | carried by M6b |
| `hook::ipc_server::tests::a_second_listener_on_a_taken_pipe_name_is_refused` (I1, spliced onto master) | compile-red: E0425 `bind` | ok | carried by M6b |
| `core::app::tests::a_taken_pipe_name_is_reported_with_its_likely_owners` (P1, spliced) | compile-red: E0425 `pipe_error_message` | ok | carried by M5 |
| `core::app::tests::other_pipe_errors_are_reported_as_they_are` (P2) | compile-red (same fn absent) | ok | same function as P1; P2 is the negative arm, M5 flips only the positive arm, so P2 has no mutation of its own |
| `system::single_instance::tests::*` T1-T8 | compile-red: module `single_instance` does not exist on master (`ls src/system` = mod.rs, registry.rs, system_function.rs) | ok (8 passed) | carried by M1, M2, M3, M7, M8, M10 |

Splices onto master were only the test bodies, so the API-absent compile errors are the whole red. The master worktree was removed with `git worktree remove --force`; `git worktree prune`; `git worktree list` shows only the lane tree.

## Mutations (scratch worktree `wt/mut` at `5725499`, own target `target-wt/mut`; each reverted with `git show HEAD:<file> > <file>`, final `git diff --ignore-cr-at-eol --exit-code` clean; worktree removed)

| # | File:line | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | `single_instance.rs:57` | `if false && last_error == ERROR_ALREADY_EXISTS` | `a_second_acquisition_of_a_held_name_is_already_running`, `the_name_is_free_again_once_every_guard_is_dropped` | caught: both FAILED (`6 passed; 2 failed`) |
| M2 | `single_instance.rs:60` | `drop(guard)` -> `mem::forget(guard)` | `the_name_is_free_again_once_every_guard_is_dropped` | caught: FAILED "the refused acquisition must not keep the mutex alive" (`7 passed; 1 failed`) |
| M3 | `single_instance.rs:116` | ACCESS_DENIED no longer classified as AlreadyRunning | `a_mutex_this_process_may_not_open_counts_as_already_running`, `only_access_denied_among_open_errors_means_already_running` | caught: both FAILED |
| M4a | `keyboard_hook_callback.rs:328` | keyboard gate `if false && !hook.is_enabled()` | `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` | caught: "with remapping off the keyboard callback waited 5.0016062s for the engine lock" |
| M4b | `keyboard_hook_callback.rs:370` | foreground gate disabled | same | caught: "... the foreground callback waited 5.0157522s ..." |
| M5 | `app.rs:227` | `if false && taken` (ACCESS_DENIED hint dropped) | `a_taken_pipe_name_is_reported_with_its_likely_owners` | caught: FAILED at `app.rs:315` |
| M6a | `ipc_server.rs:~100` | `.try_overwrite(true)` on the listener | `a_second_listener_on_a_taken_pipe_name_is_refused` | NOT red, `1 passed`: equivalent mutant (interprocess ignores the flag on Windows); replaced by M6b |
| M6b | `ipc_server.rs:116` | `bind` falls back to `<name>-2` when the name is taken | same | caught: FAILED at `ipc_server.rs:357` |
| M7 | `single_instance.rs:24` | `Local\` -> `Global\` | `the_daemon_mutex_is_local_key_switch_rs` | caught: FAILED |
| M8 | `single_instance.rs:72` | `Drop` no longer calls `CloseHandle` | `the_name_is_free_again_once_every_guard_is_dropped` | caught: FAILED |
| M9 | `single_instance.rs:45` | `SetLastError(ERROR_SUCCESS)` removed | `a_stale_already_exists_error_does_not_make_a_fresh_name_look_held` | NOT red (`8 passed`). Equivalent on this OS: `CreateMutexW` clears the last error when it creates the mutex (the developer's MU-F says the same). T7 cannot fail on Windows 11 26200; it guards a documented-as-unspecified behaviour. Reported as a note, not a defect |
| M10 | `single_instance.rs:91` | AlreadyRunning text shortened | `already_running_names_the_swch_commands` | caught: FAILED |
| M11 | `single_instance.rs:103,104` | `Error::source` for `Os` -> `None`, and for `AlreadyRunning` -> `Some` | none existed (gap) | NOT CAUGHT by the developer's tests -> closed by tester test, see below; with the test: both mutations FAILED ("an OS failure keeps the system error as its source" / assert at line 261); without mutation: ok |
| M12 | `app.rs:141-153` | bind after install (MU-G) | none possible | not run: `App::run` installs a global hook and binding `PIPE_NAME` would hit the owner's daemon; declared manual-only in the cut (M4) |

## Acceptance (roadmap 1.5 / the lane spec)

| Criterion | Evidence | Status |
|---|---|---|
| Mutex `Local\key-switch-rs` | `INSTANCE_MUTEX_NAME` `single_instance.rs:24`; `the_daemon_mutex_is_local_key_switch_rs` ok; M7 red | verified |
| Created with `CreateMutexW` before config load and hook install | `main.rs:17` is the first statement; config load is in `run` (`main.rs:55-57`); no automated test (bin has no harness, KS-38) | manual (M1, M1b) for the ordering; `CreateMutexW` use verified by T1 |
| Second acquisition returns `Err(AlreadyRunning)` | `a_second_acquisition_of_a_held_name_is_already_running` ok; M1 red | verified |
| Mutex name injectable, tests use a unique name | `acquire(name: &str)`; `unique_name()` (pid + counter); no test touches `INSTANCE_MUTEX_NAME` (rg: only const, main.rs, T6 compare) | verified |
| Name free again after release; refused acquisition keeps no handle | `the_name_is_free_again_once_every_guard_is_dropped`; M2, M8 red | verified |
| Higher-integrity holder (ACCESS_DENIED) counts as already running | `a_mutex_this_process_may_not_open_counts_as_already_running` (real kernel DACL squatter) + `only_access_denied_among_open_errors_means_already_running`; M3 red | verified |
| Second start prints 'already running' and exits with code 2 | text pinned by `already_running_names_the_swch_commands`; `main.rs:19-25` returns `ExitCode::from(2)`; the print and the code are in the untestable bin | manual, not verified (M1: `$LASTEXITCODE` = 2) |
| Pipe creation failure is fatal, hooks uninstalled / never installed | `bind` runs before `install` (`app.rs:141-153`); message pinned by `a_taken_pipe_name_is_reported_with_its_likely_owners` / `other_pipe_errors_are_reported_as_they_are`; refusal of a taken name by `a_second_listener_on_a_taken_pipe_name_is_refused`; M5, M6b red. The ordering itself is not testable without a hook or the production pipe | message and refusal verified; order manual, not verified (M4 `hold_the_daemon_pipe_name_for_a_manual_start`) |
| `swch open` ACCESS_DENIED / PIPE_BUSY handling stays | `src/ipc.rs`, `src/bin/swch.rs` byte-identical to master (diff exit 0 per pack); `connect_errors_are_classified`, `only_a_missing_pipe_means_no_daemon`, ipc_roundtrip `request_reports_access_denied_...`, `request_against_a_pipe_with_no_free_instance_...` all ok | verified |
| A5-m1: disabled hook passes everything without touching the engine | `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` ok; M4a and M4b red | verified |
| README and CHANGELOG | `README.md:734-737`, `CHANGELOG.md:451-463`, `:238-250` quote the line and exit code 2 (rg) | verified (read) |

Manual-only items (not verified here, a human does them): M1/M1b (second start prints the line, `$LASTEXITCODE` = 2, also with a bad config path), M2 (double-click), M3 (cross-integrity), M4 (ignored test holding the pipe, then daemon start shows `[FATAL] could not create the IPC pipe ...`, no hook), M5, optional M6.

## Failures

None.

## Tests added by the tester (`Tests:` commit)

- `2a91ec6` "Tests: pin InstanceError::source for the OS and already-running cases (1.5, KS-20)": `src/system/single_instance.rs`, test `the_os_error_is_the_source_of_an_instance_error_and_already_running_has_none`. Test code only, trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. Red under M11 (two variants), green without; fmt and clippy clean; full suite 492 passed after it.

## Notes for the reviewer

- Toolchain: the lane tree's own incremental cache `target/debug/incremental` is corrupted. `cargo clippy` there ICEs after touching sources: rustc 1.98.1 "trying to record dependency on DepNode ... in a context that does not allow it", exit `0xc0000409 STATUS_STACK_BUFFER_OVERRUN`, reproducible 3 times. A clean target dir passes. This is a toolchain/cache problem, not a lane defect. Whoever runs the gate in the lane tree should use a fresh `CARGO_TARGET_DIR` or `cargo clean`.
- `hold_the_daemon_pipe_name_for_a_manual_start` binds the production pipe name, but it is `#[ignore = "manual: ..."]`, panics if a daemon already owns the name, and was not run. No non-ignored test uses `PIPE_NAME` or `INSTANCE_MUTEX_NAME` (rg quoted above in session). Not a RED finding under the rule; reviewer to confirm the exception is acceptable.
- T7 (`a_stale_already_exists_error_...`) cannot fail on this OS (M9). It is defensive coverage for unspecified OS behaviour.
- The exit code 2 and the startup ordering (mutex before config, pipe before hook) cannot be pinned by an automated test with the current structure (bin without a harness; `App::run` installs live hooks). Both are covered by manual lines only; the lane cut declares this. A follow-up could move the AlreadyRunning -> exit-code mapping into the library.
- `git restore` / `git checkout --` are denied in this environment, so scratch files were reverted with `git show HEAD:<path> > <path>`; CRLF warnings in those worktrees were line-ending only (`--ignore-cr-at-eol` diff clean).
