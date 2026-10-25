# Tester: p0-A5-ipc / KS-01, KS-02, swch timeouts (HEAD 2baadf1 vs master b056b72)

Verdict: GREEN. Gate green, every red-first red on its parent (behavioural for S1-S6, compile-red for the API tests and carried by mutations), 11 own mutations all caught (one needed a new test, committed as `2baadf1`).

HEAD = `2baadf1` (the lane's `2b32441` plus one `Tests:` commit by the tester). Tree clean (`git status --short` empty) before and after; `git worktree list` shows only the lane tree.

## Gate (HEAD)
- `cargo fmt --check`: OK.
- `cargo clippy --all-targets --locked -- -D warnings`: OK. The first run said `Finished ... in 0.17s`, so I touched `src/lib.rs`, `src/main.rs`, `src/bin/swch.rs`, `tests/ipc_roundtrip.rs` and re-ran: `Checking key-switch-rs ... Finished` with exit 0. Re-run after my test commit: OK.
- `cargo test --locked --no-fail-fast` (final HEAD `2baadf1`, log of the run on `2b32441`: `D:/tmp/ks-orch/p0-A5-ipc/logs/test_head.txt`):
  - `running 332 tests` -> `test result: ok. 332 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s` (lib; was 331 on `2b32441`, +1 is my test)
  - `running 0 tests` (swch bin, no unit tests there) -> ok
  - `running 6 tests` -> `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.01s` (`tests/ipc_roundtrip.rs`)
  - `running 0 tests` (doc-tests) -> ok
  - Totals: 338 passed, 0 failed, 0 ignored. `rg "ignore" src tests` finds no `#[ignore]` attribute, so there are no manual-ignored tests to list; the manual checks are M1-M7 of cut.md:642-650 (below).
- `cargo build --release --locked`: OK (`Finished release profile [optimized] target(s) in 9.01s`; built, never run).
- Flake check: `cargo test --locked --test ipc_roundtrip` 5 times: `6 passed ... finished in 5.01s` each time. T2 and T5 use a real clock, but on real pipes where the 5 s deadline is the behaviour under test; bounds are `[5 s - 50 ms, 5 s + 2 s)` and `< 3 s`.

## Red-first replay
Master has none of the new API, so its tests can only be compile-red; the behavioural reds are replayed on `867da7c` (the declared red commit) and against a no-op `Engine::reset` (mutation M1).

| Test | Parent / replay | HEAD | Note |
|---|---|---|---|
| `engine::scenarios::stale_hold_timer_from_before_remapping_off_does_not_fire_a_newer_press` (S5) | red (assert) on `867da7c`: `scenario: advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]`; `running 107 tests`, `105 passed, 2 failed` | ok | KS-11 |
| `engine::scenarios::key_held_through_remapping_on_keeps_its_repeats_and_its_release` (S6) | red (assert) on `867da7c`: `scenario: repeat F13 at +500 ms: expected the event to be passed, it was suppressed` | ok | KS-31 |
| S1 `remapping_off_drops_a_pending_hold` | red (assert) with a no-op `Engine::reset` (M1): `advance 300 ms to +350 ms: expected to fire [], fired [SwitchLanguage]` | ok | KS-31 |
| S2 `tap_swallowed_before_remapping_off_works_again_after_remapping_on` | red (assert), M1: `up F13 at +1070 ms: expected to fire [SwitchLanguage], fired []` | ok | |
| S3 `remapping_off_drops_a_pending_chord` | red (assert), M1: `down K at +10 ms: expected to fire [], fired [SwitchLanguage]` | ok | |
| S4 `remapping_off_drops_a_pending_sequence` | red (assert), M1: `down F14 at +0 ms: expected the event to be passed, it was suppressed` | ok | |
| S1-S6 on master | compile-red (`remapping_off` / `Engine::reset` absent; rg on master finds none) | ok | S5, S6 also red by assertion on M1 |
| `tests/ipc_roundtrip.rs` T1-T6 (copied from HEAD into a master worktree, `--no-run`) | compile-red: E0432 unresolved `DaemonControl`, `DaemonStatus`; E0432 unresolved `REPLY_TIMEOUT`, `RequestError`, `pipe_path`, `request` | ok (6 passed) | carried by M7 (T2), M8 (T4), M9/M10 (T1, A1) |
| U1 `gestures::clear_drops_every_entry_and_keeps_the_generation`, U2 `keys::reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` | compile-red (`clear`, `reset_from_os` do not exist on master) | ok | U1 carried by M9; U2 by M1 (reset no longer calls `reset_from_os`; S6) |
| K1, K1b, K1c, K2, K3 (`set_enabled` / gate) | compile-red (`set_enabled`, `is_enabled` absent on master) | ok | K1 by M2, K2 by M3; K3 and K1b by M2 |
| P1-P3 (`pipe_security`, `create_listener`) | compile-red | ok | P3 by M4 and M5; P1/P2 by their own assertion (P2 shows P1 can fail) |
| C-1..C-4, I1-I3, A1, A2 | compile-red | ok | C-1 by M8, C-2 by M11, A1 by M10 |

I did not adapt any test to compile on master (cut.md D9/RK15): on master `dispatch(On)` reaches `install()`. The master worktree was used for `cargo test --no-run` only; nothing was executed there.
Worktrees removed (`git worktree remove --force`, `git worktree prune`); `git worktree list` shows only the lane tree.

## Mutations (scratch worktree `D:/tmp/ks-orch/p0-A5-ipc/wt/mut` at HEAD, own `CARGO_TARGET_DIR`; each reverted with `git -C ... checkout -- .` and `git diff --exit-code` clean)
| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| M1 | src/engine/mod.rs `Engine::reset` | body replaced by `let _ = env;` | S1-S6 | caught: `running 171 tests`, 6 FAILED: S1, S2, S3, S4, S5, S6 (messages above) |
| M2 | src/hook/keyboard_hook.rs `set_enabled` | `== enabled` -> `!= enabled` (reset on a no-op, none on a change) | K1 `toggling_reports_changes_and_never_installs_a_hook` | caught: `running 20 tests`, 4 FAILED: K1, K1b `turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it`, K2, K3 |
| M3 | src/hook/keyboard_hook_callback.rs `decide_event` | locked re-check `hook.is_enabled().then(..)` -> `Some(engine.on_key(..))` | K2 `disabled_hook_passes_every_event_without_asking_the_engine` | caught: 1 FAILED, K2 |
| M4 | src/hook/ipc_server.rs `PIPE_SDDL` | `(A;;GRGW;;;IU)` -> `(A;;GR;;;IU)` | P3 `listener_with_the_pipe_security_admits_this_process` | caught: `Access is denied` (os error 5), P3 FAILED |
| M5 | same | label `ME` -> `HI` | P3 | caught: `CreateNamedPipeW accepts the descriptor ...: A required privilege is not held by the client` (1314), P3 FAILED (depends on a non-elevated test process) |
| M6 | same | label line `S:(ML;;NWNR;;;ME)` deleted | none expected to exist | NOT CAUGHT on `2b32441` (`6 passed` in `hook::ipc_server`, `6 passed` in `ipc_roundtrip`): nothing pins the mandatory label. Test added, see below; red under M6 (`PIPE_SDDL has a SACL part that carries the mandatory label`), green on HEAD |
| M7 | src/ipc.rs `request` | `recv_timeout(timeout)` -> `recv_timeout(timeout * 2)` | T2 `request_times_out_against_a_server_that_never_replies` | caught: `gave up late, after 10.0089378s` |
| M8 | src/ipc.rs `classify_connect_error` | `ERROR_ACCESS_DENIED` -> `RequestError::NotRunning` | C-1 `connect_errors_are_classified`, T4 | caught by both (`running 11 tests` 1 FAILED; T4 FAILED) |
| M9 | src/engine/gestures.rs `Gestures::clear` | adds `self.last_generation = 0;` | U1, S5 | caught: both FAILED (`169 passed; 2 failed`) |
| M10 | src/core/app.rs `DaemonHandle::status` | `enabled: self.hook.is_enabled()` -> `is_installed()` (the KS-01 symptom) | A1 `daemon_handle_status_follows_the_hook_flag` | caught: A1 FAILED |
| M11 | src/ipc.rs `daemon_running` | `AccessDenied` -> `Some(false)` | C-2 `only_a_missing_pipe_means_no_daemon` | caught: C-2 FAILED |

## Acceptance
| Criterion | Evidence | Status |
|---|---|---|
| Brief 1: the integration test is green | `status_off_on_status_round_trip_without_a_hook` (T1) ok; whole file `6 passed; 0 failed ... 5.01s`, 5 reruns identical. T1 uses `unique_pipe_name` (pid + counter, `tests/ipc_roundtrip.rs:28`), `ipc_server::spawn(&name, control)` with a fake control, no hook; `rg PIPE_NAME tests` finds nothing, and in `src` only `swch.rs` and `app.rs:139` use it | verified |
| Brief 2: no `SetWindowsHookExW` reachable from IPC code | rg1: only `src\hook\keyboard_hook.rs` lines 8, 170, 174, 192, 225, 234, 244, 246 (inside `install`/`uninstall`). rg2: `install`/`uninstall` called only at `src\core\app.rs:131` and `:189` (`App::run`), plus `keyboard_hook.rs:205` (rollback in `install`) and `:261` (`Drop`); both are `pub(crate)`. rg3 `rg "KeyboardHook\|keyboard_hook\|install\|fn (enable\|disable)\b" src/hook/ipc_server.rs src/ipc.rs src/bin/swch.rs`: no output, exit 1. rg4: `impl DaemonControl for DaemonHandle -A 26` contains no `install`/`uninstall`, exit 1. Plus K1 `toggling_reports_changes_and_never_installs_a_hook` and A1 assert `!is_installed()` | verified |
| Brief 3: a read against a server that never replies errors in about 5 s | T2 `request_times_out_against_a_server_that_never_replies`: dummy server accepts and holds the connection; `Err(TimedOut(_))` with elapsed in `[4.95 s, 7 s)`; the whole file takes 5.01 s; red under M7 | verified |
| Spec: `on`/`off` flip an `AtomicBool` checked first in the callback; disabled -> only `CallNextHookEx` | K2, K3 (extern callback returns 1 only while on), M2, M3; rg `enabled\.(swap\|store)` only in `keyboard_hook.rs:149` and a test fake. The fast gate before the engine is not separately observable (the locked re-check gives the same result); order `code < 0`, then gate, then `lparam` read by reading `keyboard_hook_callback.rs:238-256` | verified (gate order by reading) |
| Spec: reset transient state on toggle | S1-S6, U1, U2, K1b; red under M1/M2/M9 | verified |
| Spec: status reports the real flag | I1 `dispatch_switches_remapping_and_status_reports_the_flag`, T1 (`running` -> `disabled` -> `running`), A1; red under M10 | verified |
| Spec: explicit SD, SY/BA full, IU read/write, medium label, SDDL with comments, SDDL parses | P1 `pipe_sddl_parses`, P2 `malformed_sddl_is_rejected`, P3, new `pipe_sddl_grants_exactly_system_admins_and_interactive_users_and_labels_medium` (shape, `2baadf1`); comments in `PIPE_SDDL` read | verified for parse and shape; that the label reaches the live pipe and that an elevated daemon admits a non-elevated swch is **manual, not verified** (M2) |
| Spec: swch connect timeout and 5 s read deadline | T2, T5 `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running` (`Busy \| TimedOut`, bounded), C-1 | verified (swch.rs wiring: rg5 shows `request(.., REPLY_TIMEOUT)` at `swch.rs:76`, `:146`) |
| Spec: ACCESS_DENIED and PIPE_BUSY on connect count as running for `open` | T4 (`AccessDenied`, `daemon_running() == Some(true)`), T5, C-2, M8, M11. The `cmd_open` branch itself (`swch.rs:76-93`) calls `ShellExecuteW` and is not unit-tested | verified at the `request`/`daemon_running` level; `cmd_open` end to end is **manual, not verified** (M3) |
| Spec: remove the dead placeholder | `rg -n "Duration::from_secs\(5\)" src/bin/swch.rs`: no output, exit 1 | verified |
| Spec: injectable name, no hook in the integration test | T1 (above) | verified |
| Roadmap A `:94`: after `swch off; swch on` bindings work | K1b, K2, K3, S1-S6; live check M1 | engine and adapter verified; live **manual, not verified** (M1) |
| Roadmap A `:95`: non-elevated `swch status` reaches an elevated daemon | P3 (this process's token only), T1; the elevated case is M2 | **manual, not verified** |
| Roadmap A `:96`: hung daemon, `swch` exits within 5 s | T2; live variant M5 (thread-level suspend of `ipc-server`, never the process) | T2 verified; M5 manual |

Manual only (no `#[ignore]` stubs exist; cut.md:642-650): M1 (off/on with CapsLock), M2 (elevated daemon, non-elevated swch, `exit` without `[FATAL]`), M3 (`swch open` against a running elevated daemon), M4 (Hold armed before `off`), M5 (hung daemon), M6 (debug daemon IU path), M7 (Ctrl+C after `off`).

## Failures
None. Finding F1 (closed): M6 (dropping the mandatory label) was not caught by any test on `2b32441`; closed by the added test.

## Tests added by the tester (`Tests:` commits)
- `2baadf1` `Tests: pin the pipe SDDL shape: three DACL entries and the medium label (KS-02)`, `src/hook/ipc_server.rs` (test module only): `pipe_sddl_grants_exactly_system_admins_and_interactive_users_and_labels_medium`. It asserts the SACL is exactly `(ML;;NWNR;;;ME)`, the DACL starts `D:P`, contains the three ACEs and has exactly three entries. Red under M6, green on HEAD. Limit: it pins the string, it does not read the label back from a created pipe (that needs `Win32_Security` windows features, a Cargo.toml change outside the tester's remit).

## Notes
- Test code safety: no test binds `key-switch-rs.sock` or writes a real `config.ron`; pipe names are `key-switch-rs-unit-<pid>-sd.sock` (P3) and `unique_pipe_name` in the integration file. K3 calls the extern callback with no hook installed (`CallNextHookEx(None, ..)` outside a dispatch) and `set_enabled` reads `GetAsyncKeyState` (read only); it is the only test touching `get_hook()`.
- T2 parks a 30 s dummy-server thread and an `ipc-request` thread until the test binary exits (developer-acknowledged, impl.md:100); the binary exits after 5.01 s in every run.
- Book pages still describe the old behaviour (pack fact 57); deferred to the phase by cut.md:425, not a test matter.
- Scratch logs in `D:/tmp/ks-orch/p0-A5-ipc/logs/`; scratch targets in `D:/tmp/ks-orch/p0-A5-ipc/target-wt/`; the worktree directory `wt/` is empty.
