# Implementation: p0-A5-ipc round 1

Verdict: DONE. Branch `fix/ipc-control` (from `master` `b056b72`), HEAD `2b32441`, tree clean.
Cut: `cut.md` (CUT READY); critique: `critique.md` (CHANGES_REQUESTED, Critical 0, Important 2, Minor 4).
Critique response and deviations: `cut.md`, section "Developer addendum: critique response (round 1)" (line 784 on).

## Items

- KS-31 harness + naive reset (C1, `04b5375`) - `src/engine/testing.rs:301-339` (`remapping_off`/`remapping_on`), `:447-454` (key events skip the engine while off), `foreground_changed` fails while off; `src/engine/mod.rs` `Engine::reset` (naive). Red-first against a **no-op** `Engine::reset` (critique M2), then green with the naive reset:
  - `remapping_off_drops_a_pending_hold` (`scenarios.rs:2282`): red "advance 300 ms to +350 ms: expected to fire [], fired [SwitchLanguage]" -> green
  - `tap_swallowed_before_remapping_off_works_again_after_remapping_on` (`:2308`): red "up F13 at +1070 ms: expected to fire [SwitchLanguage], fired []" -> green
  - `remapping_off_drops_a_pending_chord` (`:2332`): red "down K at +10 ms: expected to fire [], fired [SwitchLanguage]" -> green
  - `remapping_off_drops_a_pending_sequence` (`:2352`): red "down F14 at +0 ms: expected the event to be passed, it was suppressed" -> green
- KS-11/KS-31 declared red commit (C2, `867da7c`); exactly these two fail on it, every other engine test passes (167 passed, 2 failed):
  - `stale_hold_timer_from_before_remapping_off_does_not_fire_a_newer_press` (`:2371`): red "advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]" (as predicted) -> green in C3
  - `key_held_through_remapping_on_keeps_its_repeats_and_its_release` (`:2394`): red "repeat F13 at +500 ms: expected the event to be passed, it was suppressed" (as predicted) -> green in C3
- Final reset (C3, `79b8760`) - `src/engine/mod.rs:358` `Engine::reset` (gestures cleared keeping `last_generation`, fresh chords/sequences, `KeyState::reset_from_os`); `src/engine/gestures.rs:226` `Gestures::clear`; `src/engine/keys.rs:117` `reset_from_os` + `MOUSE_BUTTONS`; `env.rs` doc.
  - `clear_drops_every_entry_and_keeps_the_generation` (`gestures.rs:529`): red E0599 "no method named `clear` found for struct `gestures::Gestures`" -> green
  - `reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` (`keys.rs:549`): red E0599 "no method named `reset_from_os` found for struct `keys::KeyState`" -> green
- KS-01 gate (C4, `7bdc72c`) - `src/hook/keyboard_hook.rs:59` `enabled: AtomicBool`, `:125` `is_enabled`, `:146` `set_enabled`; `enable`/`disable` deleted; `:162` `install` and `:226` `uninstall` `pub(crate)` with main-thread doc; handle fields private; D10 at `:229-238`. `src/hook/keyboard_hook_callback.rs:245-251` fast gate after `code < 0`, before `lparam` is read; `:174-182` re-check under the lock in `decide_event`; `:287-297` `foreground_changed` gate (both checks); `HookEnv`/`new` `pub(super)`. `ipc_server::dispatch` via `set_enabled`/`is_enabled` (interim). Red for all: E0599 "no method named `set_enabled` found for struct `keyboard_hook::KeyboardHook`" / "`is_enabled`":
  - K1 `toggling_reports_changes_and_never_installs_a_hook` -> green
  - K1b `turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it` -> green
  - K1c `set_enabled_refuses_a_nested_call` -> green
  - K2 `disabled_hook_passes_every_event_without_asking_the_engine` -> green
  - K3 `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` (renamed from `extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one`, old asserts kept, toggle asserts appended) -> green
- KS-02 SDDL (C5, `8dc6843`) - `src/hook/ipc_server.rs:61` `PIPE_SDDL` (commented per ACE, no `O:`/`G:` with the reason), `pipe_security`, `security_from_sddl`, `:94` `create_listener` (`ListenerOptionsExt::security_descriptor`). Red E0425 "cannot find function `create_listener` / `pipe_security` / `security_from_sddl`":
  - P1 `pipe_sddl_parses` -> green
  - P2 `malformed_sddl_is_rejected` -> green
  - P3 `listener_with_the_pipe_security_admits_this_process` -> green (CreateNamedPipeW accepted the ME label unprivileged; the DACL admitted the test token)
- swch timeouts (C6, `bf669f3`) - `src/ipc.rs:37` `REPLY_TIMEOUT`, `pipe_path`, `:47` `RequestError` (+ `Display`, `Error`, `daemon_running`), `:94` `request` (helper thread `ipc-request` + `recv_timeout`), `:120` `exchange` (`connect_by_path_with_wait_mode(.., ConnectWaitMode::Timeout)`, no flush), `:139` `classify_connect_error`. `src/bin/swch.rs`: `send` and `cmd_open` per D8, usage text, placeholder/`connect`/`round_trip` deleted, comment at `:95` reworded (I2). Red E0432 "unresolved imports `key_switch_rs::ipc::REPLY_TIMEOUT`, `RequestError`, `pipe_path`, `request`" / E0425 "cannot find function `classify_connect_error`":
  - C-1 `connect_errors_are_classified`, C-2 `only_a_missing_pipe_means_no_daemon`, C-3 `pipe_path_is_the_local_pipe_namespace`, C-4 `timed_out_names_the_deadline` -> green
  - T2 `request_times_out_against_a_server_that_never_replies` (acceptance 3) -> green, ~5.0 s
  - T3 `request_reports_not_running_without_a_pipe`, T4 `request_reports_access_denied_for_a_pipe_this_process_may_not_open`, T5 `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running` (observed `TimedOut(500ms)` after 502 ms), T6 `request_reads_the_reply_of_a_server_that_answers` -> green
- DaemonControl (C7, `f0b2b79`) - `src/hook/ipc_server.rs:30` `DaemonControl`, `DaemonStatus`, `:106` `spawn(pipe_name, Arc<dyn DaemonControl>) -> io::Result<()>`, `:158` `dispatch(&dyn DaemonControl)`; `MAIN_THREAD_ID`, `set_main_thread_id`, `windows` and `KeyboardHook` imports deleted. `src/core/app.rs:26-58` `DaemonHandle` + impl; `run` spawns with it and prints the two warnings on `Err`. Red E0432 "unresolved imports `...ipc_server::DaemonControl`, `...DaemonStatus`" / E0405 / E0422 "`DaemonHandle`":
  - I1 `dispatch_switches_remapping_and_status_reports_the_flag`, I2 `dispatch_reports_a_control_failure_as_err`, I3 `dispatch_exit_asks_the_daemon_to_quit_once` -> green
  - A1 `daemon_handle_status_follows_the_hook_flag`, A2 `daemon_handle_refuses_exit_without_a_main_thread` -> green
  - T1 `status_off_on_status_round_trip_without_a_hook` (acceptance 1) -> green
- Comment-only (`2b32441`): rewrap of the `with_engine` doc.

D9 respected: no `dispatch` test before C7; no test was adapted to compile on a parent.

## Remarks

- I1 (M5 suspends the hook owner) - RESOLVED in `cut.md` addendum: M5 now suspends only the `ipc-server` thread (Process Explorer, Threads tab), with an explicit warning never to suspend the whole process (KS-08 silent hook removal). Automated cover: T2.
- I2 (rg check 3 false positive) - RESOLVED at `src/bin/swch.rs:95` ("The daemon binary sits next to swch.exe."); new module docs avoid the tokens. Check 3 on HEAD prints nothing (rc 1). Same false-positive class found in check 4 (`-A 30` reaches the pre-existing `App::config_path` doc at `app.rs:62`); addendum gives `-A 26`, which covers exactly the impl (`app.rs:32-58`).
- M1 (affinity by comment only) - REFUTED as a change for this lane: no current defect (remark says so); `ipc_server` cannot name `KeyboardHook`; `DaemonHandle` is private to `core::app` and calls only `set_enabled`/`is_enabled`/`bindings`; a structural marker changes D1's `install` signature, an architect decision (natural home: 2.4). Evidence in addendum.
- M2 (KS-31 never red) - RESOLVED: S1-S4 red against a no-op reset, failures quoted in the body of `04b5375` and above.
- M3 (dependents / book sync) - RESOLVED: 2.2 added to dependents in the addendum and in the `set_enabled` doc (`keyboard_hook.rs:142-145`); `roadmap.md:297` added to the book-sync list in the addendum.
- M4 (IU widens control) - RESOLVED: RK10 corrected in the addendum; CHANGELOG KS-02 entry and README `swch` section state that any signed-in interactive user can toggle/stop the daemon or answer `swch` until 4.1.
- Open Q1 (label read-back) - answered: not automated (needs `Win32_Security*`); manual M2 is the only proof the label reaches the pipe.
- Open Q2 (symmetry bookkeeping) - RESOLVED at `src/engine/testing.rs:301-339`: `remapping_off` clears `press_suppressed`; `remapping_on` keeps it (keys pressed while off are recorded as passed and seeded as passed).
- Open Q3 (SD parsed per instance) - RESOLVED: code docs say parsed once in `create_listener`, cloned once by interprocess for later instances.

## Deviations from the cut

- The engine "Threads and locking" doc note landed in C4 instead of C1/C3 (it became true in C4).
- `keyboard_hook_callback.rs` module steps renumbered (new step 2) instead of "1a".
- One extra comment-only commit (`2b32441`).
- `dispatch` error contexts: `could not turn remapping on: ..`, `could not turn remapping off: ..`, `could not shut down: ..` (the cut gives only the `ERR: <context>: m` shape).
- `remapping_off`/`remapping_on` fail the script when the switch is already in that position (the cut does not say; it matches `set_enabled`, which resets only on a real change).

## Unsafe / atomics added

- `src/hook/keyboard_hook_callback.rs:247-250` - SAFETY: "passes the event on with the exact arguments the system gave this hook procedure. Remapping is off (`swch off`): the event is not even read, so injected and physical events pass alike."
- `src/hook/keyboard_hook.rs:231-234` (rewritten from the old block) - SAFETY: "the handle came from SetWinEventHook in `install`, and the swap above took it out of the field, so it is unhooked at most once."
- `src/core/app.rs:52-56` (moved from `ipc_server.rs`) - SAFETY: "PostThreadMessageW takes the thread id and the message by value and reads no memory of ours; WM_QUIT carries no pointer in its parameters. A thread id that is no longer valid only makes the call fail, and that error is returned."
- `KeyboardHook::enabled` (`keyboard_hook.rs:59`): `swap(AcqRel)` only in `set_enabled`, under the engine lock; `load(Acquire)` in the LL callback (before the lock), in `decide_event`/`foreground_changed` (under the lock) and in `is_enabled`. Pairing: the swap's release pairs with the loads' acquire; the flag publishes no other data (engine state is behind the mutex), so a stale fast-path read only lets one event reach the lock, where the locked re-check decides. Documented on the field.
- Test-only atomics: `FakeControl`/`TestControl` use Release/Acquire/AcqRel; `tests/ipc_roundtrip.rs` `COUNTER.fetch_add(1, Relaxed)` with the written justification (unique numbers only, publishes nothing).
- No `static mut`, no new lock on the callback path. `DaemonHandle.main_thread_id` is a plain `u32` published through `Arc` and the thread spawn.

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass (no warnings), at every commit.
- Tests run (touched modules) on HEAD: `cargo test --locked --lib -- engine:: hook:: ipc:: core::app` -> 204 passed, 0 failed; `cargo test --locked --test ipc_roundtrip` -> 6 passed, 0 failed (5.02 s). Full `cargo test --locked` left to the tester.
- C2 is the only declared red commit (S5, S6 fail; 167 other engine tests pass).
- Acceptance rg checks on HEAD (exact output in the cut addendum): check 1 only `keyboard_hook.rs` (import, comments, calls in `install`/`uninstall`); check 2 only `keyboard_hook.rs` definitions/error path/`Drop` and `app.rs:131,189` in `App::run`; check 3 no match; check 4 no install call (`-A 26` for the `DaemonHandle` impl). `rg -n "Duration::from_secs\(5\)" src/bin/swch.rs`: no match; `REPLY_TIMEOUT` in `ipc.rs` and `swch.rs`.
- Manual ignores added: none (no automatable test needs a live hook). Manual checks for a human: M1-M7 from the cut, with M5 replaced per the addendum (suspend only the `ipc-server` thread). M2 is the only proof that the medium label reaches the pipe.

## Docs

- `CHANGELOG.md` `[Unreleased]`: Fixed - KS-01 (with KS-31), KS-02 (with the IU cross-user note, M4), swch timeouts/`open` detection (KS-20 partly, KS-39); Added - `tests/ipc_roundtrip.rs`.
- `README.md`: layout lines (`tests/ipc_roundtrip.rs`, `ipc.rs`, `ipc_server.rs`), `keyboard_hook.rs`/`keyboard_hook_callback.rs` bullets, typical session, `swch` command table, `swch.exe` description, the pipe/timeout/IU paragraph.
- `src/config/default_config.ron`: unchanged (no config meaning changes). `book/src/**`: unchanged (phase book sync, Q5; add `roadmap.md:297`, M3).

## Commits

- `04b5375` Engine: Engine::reset and remapping off/on steps in the scenario harness
- `867da7c` Tests: red-first: a reset keeps the Hold generation and the keys Windows holds (KS-11, KS-31)
- `79b8760` Engine: a reset keeps the Hold generation and takes held keys from Windows (KS-11, KS-31)
- `7bdc72c` Hook: swch on/off toggle an enabled flag; the hooks stay installed on the main thread (KS-01, KS-31)
- `8dc6843` IPC: the pipe gets an explicit security descriptor (KS-02)
- `bf669f3` swch: bounded connect, 5 s reply deadline; open counts denied, busy and silent daemons as running
- `f0b2b79` IPC: the server takes its pipe name and a DaemonControl; the daemon passes a DaemonHandle
- `2b32441` Hook: rewrap the with_engine doc after the reset note

## Open points / pre-existing warnings

- No pre-existing clippy warnings.
- Pre-existing `unsafe` blocks without a SAFETY comment in `src/core/app.rs` (`GetCurrentThreadId` in `run`, `SetConsoleCtrlHandler`, the message loop, `console_ctrl_handler`'s `PostThreadMessageW`): untouched, outside the cut.
- T2 parks one dummy-server thread (30 s) and one `ipc-request` thread until the test binary exits (accepted, cut RK13/Q6). T5 parks the `ipc-request` thread until `WaitNamedPipeW` returns.
- `README.md` "All commands except `open` connect to the daemon's named pipe" is still literally true for the command sense, but `open` now also probes the pipe; the `swch open` table row says so. Left as the cut's texts give it.
