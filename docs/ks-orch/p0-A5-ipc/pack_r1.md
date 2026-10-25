# Context pack, round 1: p0-A5-ipc (KS-01, KS-02, swch timeouts)

For the tester and the code-reviewer. Facts only, each with `file:line`. Anything taken from a lane report without my own re-check is marked UNVERIFIED.
Inputs: `D:/tmp/ks-orch/p0-A5-ipc/cut.md` (CUT READY, 867 lines), `impl.md` (DONE), `critique.md` (not re-read here; impl.md says Critical 0, Important 2, Minor 4, all answered in cut.md:784-867).

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/ipc-control`, HEAD `2b32441`, tree clean. `master` = `b056b72`.
- `git log --oneline master..fix/ipc-control` (oldest last):
  - `2b32441` Hook: rewrap the with_engine doc after the reset note (comment only)
  - `f0b2b79` IPC: the server takes its pipe name and a DaemonControl; the daemon passes a DaemonHandle (C7)
  - `bf669f3` swch: bounded connect, 5 s reply deadline; open counts denied, busy and silent daemons as running (C6)
  - `8dc6843` IPC: the pipe gets an explicit security descriptor (KS-02) (C5)
  - `7bdc72c` Hook: swch on/off toggle an enabled flag; the hooks stay installed on the main thread (KS-01, KS-31) (C4)
  - `79b8760` Engine: a reset keeps the Hold generation and takes held keys from Windows (KS-11, KS-31) (C3)
  - `867da7c` Tests: red-first: a reset keeps the Hold generation and the keys Windows holds (KS-11, KS-31) (C2, declared RED)
  - `04b5375` Engine: Engine::reset and remapping off/on steps in the scenario harness (C1)
- `git diff --stat master..fix/ipc-control`: 15 files, +1534 / -236:
  CHANGELOG.md +41; README.md 35; src/bin/swch.rs 119; src/core/app.rs 121; src/engine/env.rs 4; src/engine/gestures.rs +44; src/engine/keys.rs +89; src/engine/mod.rs 31; src/engine/scenarios.rs +145; src/engine/testing.rs 70; src/hook/ipc_server.rs 309; src/hook/keyboard_hook.rs 206; src/hook/keyboard_hook_callback.rs 126; src/ipc.rs 206; tests/ipc_roundtrip.rs +224 (new).
- Unchanged: `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/main.rs`, `src/lib.rs`, `src/config/**`, `book/src/**` (cut.md:404-425).

## File map

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/hook/keyboard_hook.rs:40-67` | `KeyboardHook` fields; `enabled: AtomicBool` `:59` with ordering doc `:50-58`; `engine` doc `:60-65` | KS-01 gate state; R3/R4 |
| `src/hook/keyboard_hook.rs:82-104` | `with_engine` (nested-call refusal `:96-98`, poison takeover `:99`) | R3; lock now also taken by IPC worker |
| `src/hook/keyboard_hook.rs:124-155` | `is_enabled` `:125-127`; `set_enabled` `:146-155` | the toggle + reset under the lock |
| `src/hook/keyboard_hook.rs:157-250` | `install` `pub(crate)` `:162`, `uninstall` `pub(crate)` `:226`, D10 log `:234-237`; `is_installed` still `pub` `:217` | rg acceptance; R10 |
| `src/hook/keyboard_hook.rs:259-263` | `Drop` calls `uninstall()` | local test hooks drop on test threads |
| `src/hook/keyboard_hook.rs:404-492` | tests K1 `:405`, `QuietEnv` `:422-437`, K1b `:440`, K1c `:478` | red-first tests |
| `src/hook/keyboard_hook_callback.rs:1-69` | module doc: steps 1-7 `:4-25`, IPC worker under lock `:44-47`, re-entry `:56-69` | doc vs code |
| `src/hook/keyboard_hook_callback.rs:105-141` | `HookEnv` now `pub(super)` `:105`, `new` `pub(super)` `:110`; `os_key_down` = `GetAsyncKeyState` `:126-132` | used by `set_enabled` |
| `src/hook/keyboard_hook_callback.rs:160-183` | `decide_event`: sentinel `:166-168`, locked re-check `:178-182` | R1, R2, race close |
| `src/hook/keyboard_hook_callback.rs:233-268` | `keyboard_hook_callback`: `code < 0` `:238-243`, fast gate `:245-251`, `lparam` deref `:256`, outcomes `:261`, `:267` | R1 ordering |
| `src/hook/keyboard_hook_callback.rs:278-299` | `foreground_changed` gate `:288-290`, locked re-check `:294-298` | D3 |
| `src/hook/keyboard_hook_callback.rs:379-461` | K2 `:380`; K3 `:402` (only test touching `get_hook()`) | red-first tests |
| `src/hook/ipc_server.rs:1-49` | module doc; `DaemonControl` trait `:30-40`; `DaemonStatus` `:43-49` | D4 seam |
| `src/hook/ipc_server.rs:51-100` | `PIPE_SDDL` `:61-77`; `pipe_security` `:80-82`; `security_from_sddl` `:84-89`; `create_listener` `:94-100` | KS-02 |
| `src/hook/ipc_server.rs:102-127` | `spawn(pipe_name, Arc<dyn DaemonControl>) -> io::Result<()>`; workers via `thread::spawn` `:118` | injectable name; KS-26 untouched |
| `src/hook/ipc_server.rs:129-155` | `handle_connection` (unbounded `read_line` `:139`, server `flush` `:154`) | unchanged logic except control param (KS-26/37 out of scope) |
| `src/hook/ipc_server.rs:157-178` | `dispatch(cmd, &dyn DaemonControl)` and reply strings | I1-I3, T1 |
| `src/hook/ipc_server.rs:180-305` | `FakeControl` `:192-231`; I1 `:234`, I2 `:258`, I3 `:268`, P1 `:275`, P2 `:280`, P3 `:286` | tests |
| `src/ipc.rs:1-43` | module doc `:14-17`; imports `:19-29`; `PIPE_NAME` `:33`; `REPLY_TIMEOUT` `:37`; `pipe_path` `:41-43` | swch timeout |
| `src/ipc.rs:45-85` | `RequestError` `:47-58`; `daemon_running` `:64-70`; `Display` `:73-83` (R6) | open detection |
| `src/ipc.rs:87-155` | `request` `:94-116`; `exchange` `:120-136`; `classify_connect_error` `:139-155` | deadline mechanism |
| `src/ipc.rs:157-219` | `Command` docs `:159-171` (On/Off text rewritten) ; `parse_response` `:209-219` (no-prefix = success, KS-37) | |
| `src/ipc.rs:272-330` | C-1 `:273`, C-2 `:302`, C-3 `:317`, C-4 `:325` | tests |
| `src/bin/swch.rs:1-164` | module doc `:1-18`; import `:22`; usage `:57-66`; `cmd_open` `:70-141` (probe `:76-93`); `send` `:145-164` | D8 |
| `src/core/app.rs:1-58` | imports `:14-18`; single `MAIN_THREAD_ID` `:20`; `DaemonHandle` `:26-30`; `impl DaemonControl` `:32-58` (unsafe `PostThreadMessageW` `:55`) | D4, R4 |
| `src/core/app.rs:121-192` | `run`: tid `:122-123`, `install` `:131`, IPC spawn + warnings `:135-153`, loop `:180-186`, `uninstall` `:189` | R10, KS-01 |
| `src/core/app.rs:213-262` | A1 `:232`, A2 `:256` (leaked local hook `:222-229`) | tests |
| `src/engine/mod.rs:340-363` | `Engine::reset` `:358-363` and doc `:343-357`; doc notes `:47-49`, `:85-87` | KS-31 |
| `src/engine/gestures.rs:221-230` | `Gestures::clear` `:226-230` (keeps `last_generation` `:81`, incremented `:138`); U1 `:529` | KS-11 |
| `src/engine/keys.rs:40-43, 105-130` | `MOUSE_BUTTONS` `:42-43`; `reset_from_os` `:117-130`; `sync_modifiers` `:97` (reference); U2 `:549`, `KeysAndButtonsDown` env `:530-545` | R8 |
| `src/engine/env.rs:40-45` | `os_key_down` doc addition | doc |
| `src/engine/testing.rs:140-191, 286-339, 444-497` | `enabled` field `:146`/`:191`; `foreground_changed` fails while off `:288-294`; `remapping_off` `:307-320` (clears `press_suppressed` `:316`); `remapping_on` `:327-339`; `key_event` skips engine while off `:447-454`; `check_press` `:474-497` | harness semantics |
| `src/engine/scenarios.rs:2273-2416` | `HOLD_F13` `:2275`, `TAP_F13` `:2278`; S1 `:2282`, S2 `:2308`, S3 `:2332`, S4 `:2352`, S5 `:2371`, S6 `:2394` | KS-31 / KS-11 scenarios |
| `tests/ipc_roundtrip.rs:1-224` | `unique_pipe_name` `:28-33` (Relaxed + justification `:29-31`); `TestControl` `:50-76`; T1 `:79`, T2 `:117`, T3 `:146`, T4 `:160`, T5 `:178`, T6 `:205` | acceptance 1 and 3 |
| `CHANGELOG.md:43-47, 94-129` | Added (`tests/ipc_roundtrip.rs` `:43`); Fixed KS-01 `:94`, KS-02 `:108`, swch timeouts after | docs |
| `README.md` hunks `:22, :25, :59, :100-108, :523-525, :532-533, :541-544, :550-557` | layout, hook bullets, swch table, pipe/timeout/IU paragraph | docs |

## Facts

### KS-01: gate and toggle

1. `KeyboardHook::enabled` is an `AtomicBool` initialised `true` (`src/hook/keyboard_hook.rs:59`, `:77`).
2. `is_enabled` is `self.enabled.load(Ordering::Acquire)` (`src/hook/keyboard_hook.rs:125-127`).
3. `set_enabled` builds `HookEnv::new()` before taking the lock (`:147`), then inside `with_engine` does `self.enabled.swap(enabled, Ordering::AcqRel)`; if the old value equals the new one it returns `false` without reset, else calls `engine.reset(&env)` and returns `true` (`:148-154`). Return type `Option<bool>`; `None` comes from `with_engine` refusing a nested call (`:95-98`).
4. `rg -n "enabled\.(swap|store)" src` matches `src/hook/keyboard_hook.rs:149` (the `KeyboardHook` swap) and `src/hook/ipc_server.rs:213` (the test-only `FakeControl` store); `KeyboardHook::enabled` has no other writer.
5. `enable()` / `disable()` no longer exist: rg `\.enable\(\)|\.disable\(\)` over `src tests` finds nothing. On master they were `pub fn` at `keyboard_hook.rs:169` / `:178` and called from `ipc_server.rs:115` / `:123` (`git show master:...`).
6. `install` and `uninstall` are `pub(crate)` (`src/hook/keyboard_hook.rs:162`, `:226`); on master both were `pub` (`master:keyboard_hook.rs:108`, `:185`). Fields `hook_handle` / `foreground_hook` are private (`:48-49`); on master `pub` (`master:keyboard_hook.rs:45-46`).
7. `is_installed` remains `pub` and only tests the stored `hook_handle` for non-null (`src/hook/keyboard_hook.rs:217-219`).
8. `uninstall` now logs a failed `UnhookWinEvent` via `BOOL::ok()` + `eprintln!("[WARN] could not remove the foreground hook: ...")` and continues to `UnhookWindowsHookEx`, whose error is returned with `?` (`src/hook/keyboard_hook.rs:234-247`).
9. `install` still refuses a second install when `hook_handle` is non-null (`:163-168`) and rolls back the LL hook if `SetWinEventHook` fails (`:202-210`).
10. In `keyboard_hook_callback`, order is: `code < 0` -> `CallNextHookEx` (`src/hook/keyboard_hook_callback.rs:238-243`); `get_hook()` + `!is_enabled()` -> `CallNextHookEx` (`:245-251`); then `lparam` is dereferenced (`:256`); then `decide_event` (`:257`).
11. The disabled fast path returns before the sentinel check, so while off injected and physical events both pass unread (`:246-250`; sentinel check is inside `decide_event` at `:166`).
12. `decide_event` returns `None` for an injected event before taking the bindings snapshot or the lock (`:166-168`), then re-checks `hook.is_enabled()` inside `with_engine` and only then calls `engine.on_key` (`:178-182`).
13. Every path of `keyboard_hook_callback` ends in `CallNextHookEx` (`:242`, `:250`, `:267`) or `LRESULT(1)` (`:261`).
14. `foreground_changed` returns when `!is_enabled()` (`:288-290`) and checks again under the lock before `engine.on_foreground_changed` (`:294-298`).
15. `HookEnv` and `HookEnv::new` changed from private to `pub(super)` (`src/hook/keyboard_hook_callback.rs:105`, `:110`) so `keyboard_hook.rs` can build one (`src/hook/keyboard_hook.rs:16`, `:147`).
16. `HookEnv::foreground` is a lazily filled `OnceCell` (`src/hook/keyboard_hook_callback.rs:106`, `:116-118`); `Engine::reset` calls only `gestures.clear`, `Chords::new`, `Sequences::new`, `keys.reset_from_os` (`src/engine/mod.rs:358-363`), and `reset_from_os` calls only `env.os_key_down` (`src/engine/keys.rs:117-130`). So the reset under the lock performs at most 254 `GetAsyncKeyState` calls (code range `1..=254`, `keys.rs:121`) and no foreground capture.
17. `with_engine` is the only accessor of `engine` (doc `src/hook/keyboard_hook.rs:64-65`; impl `:95-104`). Callers: `decide_event` (`keyboard_hook_callback.rs:178`), `start_hold_timer` (`:209`, `:216`), `foreground_changed` (`:294`), `set_enabled` (`keyboard_hook.rs:148`), tests.
18. `App::run` is the only non-test caller of `install` / `uninstall`: `src/core/app.rs:131` and `:189`; the remaining matches are `keyboard_hook.rs:205` (rollback in `install`) and `:261` (`Drop`) (rg check 2 output below).
19. `config_watcher.rs` has no `install`/`set_enabled`/`get_hook` call (rg: one doc-comment match at `src/hook/config_watcher.rs:5`).

### Engine reset (KS-31, KS-11)

20. `Gestures::clear` clears `taps`, `holds`, `double_taps` and leaves `last_generation` (`src/engine/gestures.rs:226-230`; field `:81`; increment `wrapping_add(1)` at `:138`).
21. `KeyState::reset_from_os` sets `*self = Self::new()`, then for each `code in 1..=254u16` skips `MOUSE_BUTTONS` and keys the OS reports up, else sets `active[index] = true` and `last_down[index] = LastDown::Passed` (`src/engine/keys.rs:117-130`). `MOUSE_BUTTONS = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]` (`:42-43`).
22. `reset_from_os` writes sided and generic modifier codes exactly as the `Env` reports them; U2 asserts `VK_A`, `VK_LSHIFT`, `VK_SHIFT` active and passed (`src/engine/keys.rs:559-565`, env at `:530-545`).
23. `Engine::reset` is `pub` (`src/engine/mod.rs:358`); its doc says call it only on a real change (`:354-356`).
24. Harness: `remapping_off` calls `engine.reset`, sets `enabled = false`, clears `press_suppressed` (`src/engine/testing.rs:314-316`); `remapping_on` calls `engine.reset` and keeps `press_suppressed` (`:334-335`); both `fail` if already in that position (`:308-313`, `:328-333`). Pending harness `timers` are kept across both steps (no change to `self.timers` in `:307-339`).
25. Harness `key_event` while off uses `Decision::default()` and still updates `env.os_down` for the passed event (`src/engine/testing.rs:447-460`).

### KS-02: pipe security descriptor

26. `PIPE_SDDL` = `"D:P" "(A;;GA;;;SY)" "(A;;GA;;;BA)" "(A;;GRGW;;;IU)" "S:(ML;;NWNR;;;ME)"`, one comment per component, no `O:` / `G:` with the reason given (`src/hook/ipc_server.rs:57-77`).
27. `security_from_sddl` converts with `ToWtf16::to_wtf_16` (error -> `io::ErrorKind::InvalidInput`) and calls `SecurityDescriptor::deserialize` (`src/hook/ipc_server.rs:84-89`).
28. `create_listener` = `pipe_name.to_ns_name::<GenericNamespaced>()?` then `ListenerOptions::new().name(..).security_descriptor(pipe_security()?).create_sync()` (`src/hook/ipc_server.rs:94-100`); `security_descriptor` comes from `interprocess::os::windows::local_socket::ListenerOptionsExt` (`:22-24`; interprocess-2.4.2 `src/os/windows/local_socket.rs:20-28` stores it in the options).
29. interprocess-2.4.2 `src/os/windows/security_descriptor/try_clone.rs:66-68` calls `new_sd.set_owner(group, ..)` for the group (verified by reading), which is the reason given at `src/hook/ipc_server.rs:57-60`.
30. Master's listener was `ListenerOptions::new().name(name).create_sync()` with no SD (`master:src/hook/ipc_server.rs:47`).
31. The pipe name in the daemon is still the machine-wide `ipc::PIPE_NAME = "key-switch-rs.sock"` (`src/ipc.rs:33`; `src/core/app.rs:139`) (KS-40 out of scope).
32. No code reads the SD/label back from a created pipe (impl.md:50; cut.md:857: manual M2 is the only proof the ME label reaches the pipe). P3 only proves `CreateNamedPipeW` accepts the SD and the DACL admits the test token (`src/hook/ipc_server.rs:286-304`).

### IPC server seam (D4, D5)

33. `ipc_server.rs` imports nothing from `crate::hook` or `windows`; its only crate import is `crate::ipc::{Command, format_err, format_ok}` (`src/hook/ipc_server.rs:16-26`).
34. `DaemonControl: Send + Sync` with `set_enabled(bool) -> Result<(), String>`, `status() -> DaemonStatus`, `request_exit() -> Result<(), String>` (`src/hook/ipc_server.rs:30-40`).
35. `spawn` returns `io::Result<()>`: `Err` if the listener cannot be created or the `ipc-server` thread cannot be spawned (`src/hook/ipc_server.rs:106-127`); connection workers still use bare `thread::spawn` (`:118`).
36. Replies (`src/hook/ipc_server.rs:158-178`): On -> `OK: enabled` / `ERR: could not turn remapping on: <m>`; Off -> `OK: disabled` / `ERR: could not turn remapping off: <m>`; Status -> `OK: running (N bindings live)` or `OK: disabled (N bindings live)`; Exit -> `OK: shutting down` / `ERR: could not shut down: <m>`.
37. `dispatch` answers `OK: enabled` / `OK: disabled` for both a change and a no-op, because `DaemonHandle::set_enabled` maps `Some(_)` to `Ok(())` via `.map(drop)` (`src/core/app.rs:33-38`); `None` becomes `Err("the engine is busy on this thread")` (`:37`).
38. `DaemonHandle::status` = `{ enabled: hook.is_enabled(), bindings: hook.bindings().len() }` (`src/core/app.rs:40-45`).
39. `DaemonHandle::request_exit` returns `Err` for `main_thread_id == 0`, else `PostThreadMessageW(main_thread_id, WM_QUIT, 0, 0)` mapped to `String` (`src/core/app.rs:47-57`), with a SAFETY comment `:51-54`.
40. One `static MAIN_THREAD_ID` remains (`src/core/app.rs:20`, store `Release` `:123`, load `Acquire` `:200`); the copy and `set_main_thread_id` in `ipc_server.rs` are gone (rg finds none; on master `master:src/hook/ipc_server.rs:29-33`).
41. `App::run` order: `update_bindings` -> `install()?` -> `ipc_server::spawn(ipc::PIPE_NAME, Arc::new(DaemonHandle{hook, main_thread_id: main_tid}))` -> watcher -> `GetMessageW` loop -> `uninstall()?` (`src/core/app.rs:128-189`). The two `[WARN]`/`[INFO]` lines on `Err` match master's text (`src/core/app.rs:142-150`; `master:src/hook/ipc_server.rs:50-57`).
42. `handle_connection` still has an unbounded `read_line` (`src/hook/ipc_server.rs:139`) and calls `stream.flush()` after the write (`:154`) (KS-26, KS-37 deferred to 4.1, cut.md:76).

### swch client (D7, D8)

43. `REPLY_TIMEOUT = Duration::from_secs(5)` (`src/ipc.rs:37`); `pipe_path(name) = format!(r"\\.\pipe\{name}")` (`:41-43`).
44. `request` spawns a named `ipc-request` thread (`thread::Builder`, spawn error -> `RequestError::Io`) that sends `exchange(..)` into `mpsc::sync_channel(1)`; the caller waits `recv_timeout(timeout)`: `Timeout` -> `TimedOut(timeout)`, `Disconnected` -> `Io(other(..))` (`src/ipc.rs:94-116`).
45. `exchange` connects with `DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path_with_wait_mode(path, ConnectWaitMode::Timeout(timeout))`, writes with `write_all` and no flush, then `BufReader::read_line` (`src/ipc.rs:120-136`). The same `timeout` bounds both the connect wait inside the thread and the outer `recv_timeout`.
46. interprocess-2.4.2 rejects I/O timeouts on named-pipe local-socket streams (`src/os/windows/named_pipe/local_socket/stream.rs:25-26`, `:53`, `:55`); `ConnectWaitMode::Timeout` drives `WaitNamedPipeW(remaining)` in `connect_spin_loop` (`src/os/windows/named_pipe/stream/impl/ctor.rs:55-77`, `c_wrappers.rs:201-203`); `Deferred` is `Unsupported` for this path (`ctor.rs:33-35`).
47. `classify_connect_error` (`src/ipc.rs:139-155`): `ERROR_FILE_NOT_FOUND` -> `NotRunning`; `ERROR_ACCESS_DENIED` -> `AccessDenied`; `ERROR_PIPE_BUSY | ERROR_SEM_TIMEOUT` -> `Busy`; `ErrorKind::TimedOut` -> `Busy`; else `Io`. The code is converted with `u32::try_from` + `WIN32_ERROR`, no `as` cast (`:140-143`).
48. Only connect errors are classified; write and read errors after connect become `RequestError::Io` (`src/ipc.rs:128-134`).
49. `daemon_running`: `NotRunning -> Some(false)`; `AccessDenied | Busy | TimedOut -> Some(true)`; `Io -> None` (`src/ipc.rs:64-70`). `Display` has one arm per variant (`:73-83`); `TimedOut(5 s)` renders `the daemon did not answer within 5s` (C-4 `:325-330`).
50. `swch open` probes with `request(PIPE_NAME, &Command::Status, REPLY_TIMEOUT)`: `Ok` -> "already running"; `Some(true)` -> "already running (<e>)", exit 0; `Some(false)` -> launch via `ShellExecuteW("runas")`; `None` -> `Err("cannot tell whether a daemon is running (<e>); not starting another")` (`src/bin/swch.rs:76-93`, launch `:95-140`).
51. `send`: `Ok((true,m))` -> stdout, exit 0; `Ok((false,m))` -> `swch: m`, exit 1; `Err(NotRunning)` -> ``swch: the daemon is not running. Use `swch open` to start it.``, exit 1; other `Err(e)` -> `swch: cannot reach daemon (<e>).`, exit 1 (`src/bin/swch.rs:145-164`).
52. The old `connect`, `round_trip` and the placeholder `let _ = Duration::from_secs(5);` are gone (master `swch.rs:159`, `:166`, `:176`; rg `Duration::from_secs\(5\)` in `src/bin/swch.rs` finds nothing on HEAD). `swch.rs` imports only `key_switch_rs::ipc::{Command, PIPE_NAME, REPLY_TIMEOUT, RequestError, request}` plus local `windows` uses in `cmd_open` (`:22`, `:108-111`).
53. `src/ipc.rs` now imports `interprocess::os::windows::named_pipe` and `windows::Win32::Foundation` (`:25-29`); its module doc `:10-12` still says "on Unix it would be a Unix domain socket".

### Other

54. New non-ASCII in added lines: only the moved `✓ IPC server listening` line (`src/core/app.rs:140`), whose text existed on master at `master:src/core/app.rs:95`.
55. No `#[ignore]` attribute exists in `src` or `tests` on HEAD (rg count 0); impl.md:77 states no manual-ignore stubs were added and lists manual checks M1-M7 (M5 replaced, cut.md:792-796).
56. New `unsafe` blocks with SAFETY comments: `keyboard_hook_callback.rs:247-250`, `keyboard_hook.rs:231-234` (rewritten), `app.rs:51-55` (moved from ipc_server). Pre-existing `unsafe` without SAFETY in `app.rs:115`, `:122`, `:180`, `:202` (impl.md:99, untouched).
57. Book pages still describe the pre-lane behaviour: `book/src/architecture.md:170-172` ("Why `swch on` kills the hook"), `:447-449`, `:472`, `:483`; `book/src/gaps.md:29`, `:108-110`; `book/src/overview.md:172`, `:187`; `book/src/roadmap.md:297` (2.3 "except blocked ones"). Book sync is deferred to the phase (cut.md:425, `:435-441`; Q5 `:743`).

## Red-first tests and acceptance proofs (from cut.md / impl.md)

Red-replay rule (cut.md:360-361, D9; RK15 `:725`): every new test touching `set_enabled`, `dispatch`, `spawn` or `request` must fail to *compile* on its parent. Never adapt such a test to compile on `master`: there `dispatch(Command::On, hook)` reaches `install()` (`master:ipc_server.rs:115` -> `master:keyboard_hook.rs:169-174`) and would install a real global LL hook in the test process.

| ID | Test (file:line on HEAD) | Red on parent (as reported) | Green on |
|---|---|---|---|
| S1 | `remapping_off_drops_a_pending_hold` `src/engine/scenarios.rs:2282` | vs no-op reset: `advance 300 ms to +350 ms: expected to fire [], fired [SwitchLanguage]` (impl.md:10; quoted in body of `04b5375`, UNVERIFIED) | `04b5375` |
| S2 | `tap_swallowed_before_remapping_off_works_again_after_remapping_on` `:2308` | `up F13 at +1070 ms: expected to fire [SwitchLanguage], fired []` | `04b5375` |
| S3 | `remapping_off_drops_a_pending_chord` `:2332` | `down K at +10 ms: expected to fire [], fired [SwitchLanguage]` | `04b5375` |
| S4 | `remapping_off_drops_a_pending_sequence` `:2352` | `down F14 at +0 ms: expected the event to be passed, it was suppressed` | `04b5375` |
| S5 | `stale_hold_timer_from_before_remapping_off_does_not_fire_a_newer_press` `:2371` | on `867da7c` (declared RED): `scenario: advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]` | `79b8760` |
| S6 | `key_held_through_remapping_on_keeps_its_repeats_and_its_release` `:2394` | on `867da7c`: `scenario: repeat F13 at +500 ms: expected the event to be passed, it was suppressed` | `79b8760` |
| U1 | `clear_drops_every_entry_and_keeps_the_generation` `src/engine/gestures.rs:529` | E0599 no method `clear` on `Gestures` | `79b8760` |
| U2 | `reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` `src/engine/keys.rs:549` | E0599 no method `reset_from_os` on `KeyState` | `79b8760` |
| K1 | `toggling_reports_changes_and_never_installs_a_hook` `src/hook/keyboard_hook.rs:405` | E0599 no `set_enabled`/`is_enabled` | `7bdc72c` |
| K1b | `turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it` `:440` | same | `7bdc72c` |
| K1c | `set_enabled_refuses_a_nested_call` `:478` | same | `7bdc72c` |
| K2 | `disabled_hook_passes_every_event_without_asking_the_engine` `src/hook/keyboard_hook_callback.rs:380` | same | `7bdc72c` |
| K3 | `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` `:402` (renamed from `extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one`) | same | `7bdc72c` |
| P1 | `pipe_sddl_parses` `src/hook/ipc_server.rs:275` | E0425 `pipe_security` | `8dc6843` |
| P2 | `malformed_sddl_is_rejected` `:280` | E0425 `security_from_sddl` | `8dc6843` |
| P3 | `listener_with_the_pipe_security_admits_this_process` `:286` | E0425 `create_listener` | `8dc6843` |
| C-1..C-4 | `connect_errors_are_classified` `src/ipc.rs:273`, `only_a_missing_pipe_means_no_daemon` `:302`, `pipe_path_is_the_local_pipe_namespace` `:317`, `timed_out_names_the_deadline` `:325` | E0425 `classify_connect_error`; E0432 imports | `bf669f3` |
| T2-T6 | `tests/ipc_roundtrip.rs` `:117`, `:146`, `:160`, `:178`, `:205` | E0432 unresolved `REPLY_TIMEOUT`, `RequestError`, `pipe_path`, `request` | `bf669f3` |
| I1-I3 | `src/hook/ipc_server.rs:234`, `:258`, `:268` | E0432/E0405 `DaemonControl`, `DaemonStatus` | `f0b2b79` |
| A1, A2 | `src/core/app.rs:232`, `:256` | E0422 `DaemonHandle` | `f0b2b79` |
| T1 | `status_off_on_status_round_trip_without_a_hook` `tests/ipc_roundtrip.rs:79` | E0061 `spawn` takes 1 arg; `DaemonControl` not found | `f0b2b79` |

Expected per-test outcomes from cut.md: T2 `Err(TimedOut(_))`, elapsed in `[REPLY_TIMEOUT - 50 ms, REPLY_TIMEOUT + 2 s)` (`tests/ipc_roundtrip.rs:131-142`), impl.md reports ~5.0 s; T3 `NotRunning` in < 1 s (`:149-156`); T4 `AccessDenied` with SDDL `D:P(A;;GA;;;SY)` (`:162-174`); T5 `Busy | TimedOut(_)` in < 3 s, observed `TimedOut(500ms)` after 502 ms (impl.md:33, cut.md:866); T6 `Ok((true, "got status"))` (`:220-223`); T1 replies and `toggles == [false, true]` (`:92-113`).

### Commands and expected results

| Command | Expected (source) |
|---|---|
| `cargo fmt --check` | pass (impl.md:73) |
| `cargo clippy --all-targets --locked -- -D warnings` | pass, no warnings, at every commit (impl.md:73) |
| `cargo test --locked` | full run not done by the developer (impl.md:74: "left to the tester"); no ignored tests exist to report |
| `cargo test --locked --lib -- engine:: hook:: ipc:: core::app` | 204 passed, 0 failed (impl.md:74) |
| `cargo test --locked --test ipc_roundtrip` | 6 passed, 0 failed, ~5.02 s (impl.md:74) |
| C2 red replay (`867da7c`) | exactly S5 and S6 fail with the texts above; 167 other engine tests pass (impl.md:14, :75; cut.md:448) |

Acceptance rg checks (cut.md:625-632, revised `:798-826`); output I got on HEAD `2b32441`:

1. `rg -n "SetWindowsHookExW|SetWinEventHook\(|UnhookWinEvent|UnhookWindowsHookEx" src tests` -> only `src\hook\keyboard_hook.rs` lines `8` (use), `170` (comment), `174`, `192`, `225` (doc), `234`, `244` (comment), `246`. Matches the expected output in cut.md:808-816.
2. `rg -n "fn (install|uninstall)|\.install\(\)|\.uninstall\(\)" src tests` -> `src\core\app.rs:131`, `:189`; `src\hook\keyboard_hook.rs:162`, `:205`, `:226`, `:261`. Matches cut.md:817-823.
3. `rg -n "KeyboardHook|keyboard_hook|install|fn (enable|disable)\b" src/hook/ipc_server.rs src/ipc.rs src/bin/swch.rs` -> no output, exit 1.
4. `rg -n "fn set_enabled" -A 20 src/hook/keyboard_hook.rs` shows `:146-155` with no install call; `rg -n "impl DaemonControl for DaemonHandle" -A 26 src/core/app.rs` contains no `install`/`uninstall` token (filtered rg exit 1). Use `-A 26`, not `-A 30`: `-A 30` reaches the `App::config_path` doc at `src/core/app.rs:62` ("install a notify-based file watcher"), a doc comment, not a call (cut.md:803).
5. `rg -n "Duration::from_secs\(5\)" src/bin/swch.rs` -> no output; `rg -n "REPLY_TIMEOUT" src` -> `src/ipc.rs:16,37,307` and `src/bin/swch.rs:16,22,76,146`.

Acceptance mapping: cut.md:605-623. Manual checks (human only, no `#[ignore]` stubs): M1-M7 at cut.md:642-650, with M5 replaced by the thread-level suspend of `ipc-server` at cut.md:792-796 (never suspend the whole process: KS-08).

## Open points from impl.md

- impl.md:98: no pre-existing clippy warnings.
- impl.md:99: pre-existing `unsafe` blocks without a SAFETY comment in `src/core/app.rs` (`GetCurrentThreadId` `:122`, `SetConsoleCtrlHandler` `:115`, the message loop `:180`, `console_ctrl_handler`'s `PostThreadMessageW` `:202`): untouched, outside the cut.
- impl.md:100: T2 parks one dummy-server thread (30 s sleep, `tests/ipc_roundtrip.rs:121-125`) and one `ipc-request` thread until the test binary exits; T5 parks its `ipc-request` thread until `WaitNamedPipeW` returns (accepted, cut RK13/Q6 at `:723`, `:744`).
- impl.md:101: README line "All commands except `open` connect to the daemon's named pipe" kept; `open` now also probes the pipe (the `swch open` table row says so).
- impl.md:44-52 remarks: I1, I2, M2, M3, M4 RESOLVED; M1 (hook-thread affinity by comment only) REFUTED as a change for this lane, deferred as an architect decision to 2.4; Q1 label read-back not automated (M2 manual only); Q2 symmetry bookkeeping defined in `testing.rs:301-339`; Q3 SD parsed once in `create_listener`.
- impl.md:54-60 deviations: engine "Threads and locking" note landed in C4; adapter steps renumbered (new step 2); extra comment-only commit `2b32441`; `dispatch` error contexts as in Fact 36; harness `remapping_off`/`remapping_on` fail when already in that position.
- Cut questions Q1-Q6 (cut.md:737-744), all "not blocking": IU cross-session reach (KS-40, RK11 `:721`), lone key-up after `off` (RK6 `:716`, pinned by S2), `swch open` refuses on `Io`, KS-26/37/40 deferred to 4.1, book sync deferred, T2 uses the real 5 s.
- Out-of-scope follow-ups listed at cut.md:728-735 (4.1, 2.1, 2.3, 2.4).

## Invariants in play

- **R1 Hook callback**: `code < 0` first (`keyboard_hook_callback.rs:238-243`); new fast gate is one `Acquire` load before `lparam` deref (`:245-256`); outcomes `:242,:250,:261,:267`; under-lock work added by this lane is the `is_enabled` re-check (`:179`) on the hook thread, and `Engine::reset` (<= 254 `GetAsyncKeyState`, `keys.rs:121-127`) on the IPC worker (`keyboard_hook.rs:148-154`), which the hook thread may wait on once per toggle (cut RK2 `:712`).
- **R2 Sentinel**: check still precedes state mutation (`keyboard_hook_callback.rs:166-168`); no new `SendInput` path in the delta (cut RK16 `:726`).
- **R3 Shared state**: `Mutex<Engine>` reached only through `with_engine` (`keyboard_hook.rs:95-104`); new taker is `set_enabled` on the IPC worker (`:146-155`); nested refusal covered by K1c (`:478-492`). `install`/`uninstall` unreachable from IPC (KS-01; rg checks 1-4).
- **R4 Orderings**: `enabled` swap `AcqRel` / loads `Acquire` with pairing doc (`keyboard_hook.rs:50-59`, `:126`, `:149`); one `MAIN_THREAD_ID` (`app.rs:20`, `:123`, `:200`); test `COUNTER` `Relaxed` with written justification (`tests/ipc_roundtrip.rs:29-31`).
- **R6 Display**: new `RequestError` with one arm per variant (`src/ipc.rs:73-83`).
- **R8 Key identity**: `reset_from_os` seeds sided and generic codes as reported (`keys.rs:117-130`, U2 `:559-565`); mouse codes skipped (`:42-43`).
- **R10 Message loop / shutdown**: install and uninstall only in `App::run` on the main thread (`app.rs:131`, `:189`); `exit` posts `WM_QUIT` from the IPC worker (`app.rs:55`); `UnhookWinEvent` failure logged (`keyboard_hook.rs:234-237`).
- **Timers (KS-11/25)**: Hold generation preserved across reset (`gestures.rs:226-230`, S5, U1, K1b).
- **Process boundary**: KS-02 SDDL (`ipc_server.rs:61-77`); KS-20 partly (`swch.rs:76-93`); KS-39 side effect (status probe); KS-26/KS-37/KS-40 unchanged (`ipc_server.rs:139`, `ipc.rs:209-219`, `ipc.rs:33`).
- **KS-31**: reset on real transition only (`keyboard_hook.rs:149-153`; S1-S6, K1b).

## Not covered

- `critique.md` was not re-read; its findings are taken from impl.md:4 and cut.md:784-867.
- I did not run `cargo fmt`, `clippy` or any test; all pass counts and red texts above are as reported by impl.md / commit bodies (UNVERIFIED by me). I ran only the rg checks.
- Commit bodies of `04b5375` and `867da7c` (quoted red output) not read.
- `src/engine/testing.rs` outside `:140-191`, `:280-339`, `:444-504`; `src/engine/scenarios.rs` outside `:2268-2416`; `src/engine/mod.rs` outside the diff hunks.
- README.md and CHANGELOG.md text was not compared word for word with cut.md:652-705; only hunk positions are listed.
- interprocess internals beyond the lines cited in Facts 28, 29, 46 (e.g. default instance limit of `ListenerOptions`, `create_instance.rs`) not re-read.
- Book pages beyond the drift line list in Fact 57.
