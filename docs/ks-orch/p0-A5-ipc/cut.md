# Cut: p0-A5-ipc — Workstream A (IPC part): KS-01, KS-02, swch read timeout (+ KS-31 reset, KS-20/KS-39 side of `swch open`)

Tree: `D:/key-switch-rs/key-switch-rs`, `master` @ `b056b72` (clean). Branch: `fix/ipc-control`.

## Verdict

CUT READY

## Spec re-located

The book cites `7cb3521`. "Now" means `b056b72`.

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| KS-01 worker per connection `ipc_server.rs:72` | `src/hook/ipc_server.rs:72` | unchanged | `thread::spawn(move \|\| handle_connection(stream, hook))` |
| KS-01 `on` -> `hook.enable()` `ipc_server.rs:115` | `src/hook/ipc_server.rs:115-121` | unchanged | still installs on the worker: defect present |
| KS-01 `off` -> `hook.disable()` `ipc_server.rs:123` | `src/hook/ipc_server.rs:123-126` | unchanged | |
| KS-01 status reads `is_installed` | `src/hook/ipc_server.rs:127-135` | unchanged | reports a non-null handle, not the remapping state |
| KS-01 `enable`/`disable` `keyboard_hook.rs:143-157` | `src/hook/keyboard_hook.rs:167-183` | moved | `enable` calls `install()` on the caller's thread (`:169-174`) |
| KS-01 `install` `keyboard_hook.rs:94-121`, SAFETY `:108-110` | `src/hook/keyboard_hook.rs:108-160`, SAFETY `:134-136` | moved | `SetWindowsHookExW` `:120`, `SetWinEventHook` `:138` |
| KS-01 `is_installed` `:137-139` | `src/hook/keyboard_hook.rs:162-165` | moved | |
| KS-01 `UnhookWinEvent` result discarded `:164`; swap `:160` | `src/hook/keyboard_hook.rs:192` (`let _ =`); swap `:186-188` | moved | |
| KS-01 shutdown `app.rs:134`, `[FATAL]` `main.rs:10-17` | `src/core/app.rs:133-134`, `src/main.rs:10-19` | unchanged / moved | |
| KS-01 hook fields `pub` | `src/hook/keyboard_hook.rs:45-46` | unchanged | `pub hook_handle` and `pub foreground_hook`; `rg hook_handle src` finds no use outside the file |
| KS-02 listener without an SD `ipc_server.rs:47` | `src/hook/ipc_server.rs:47` | unchanged | defect present |
| KS-02 probe `swch.rs:75`, connect `:158-163`, `runas` `:102` | `src/bin/swch.rs:75`, `:159-164`, `:103` | unchanged / moved by one line | |
| KS-02 manifest `build.rs:21-24` | `build.rs:26-35` | moved | release builds with MSVC only |
| gaps "swch read timeout" `swch.rs:171-175`, import `:19`, read `:177-181` | `src/bin/swch.rs:172-176` (`let _ = Duration::from_secs(5);` at `:176`), `:19`, `:178-182` | moved by one line | placeholder present |
| `PIPE_NAME` `ipc.rs:16` | `src/ipc.rs:16` | unchanged | |
| `Command::On/Off` docs ("uninstalls `WH_KEYBOARD_LL`") | `src/ipc.rs:21-25` | unchanged | stale after this lane |
| `MAIN_THREAD_ID`, two copies (`app.rs:16`, `ipc_server.rs:29`) | `src/core/app.rs:16`, `src/hook/ipc_server.rs:26-33`; set at `app.rs:80-83` | unchanged | R4 "two copies" |
| KS-20 hook installed before IPC `app.rs:91-98` | `src/core/app.rs:88-98` | unchanged | out of scope (1.5) |
| KS-26 read without deadline `ipc_server.rs:66-95` | `src/hook/ipc_server.rs:66-111` | unchanged | out of scope (4.1) |
| KS-31 no reset on disable/enable; `install` check-then-act `keyboard_hook.rs:83-102` | no reset exists (`rg "fn reset\|fn clear" src` finds nothing); `install` check `:109`, store `:127` | moved | engine state: `src/engine/mod.rs:264-269` |
| KS-37 exit reply race `ipc_server.rs:137-145`, `ipc.rs:75-78` | `src/hook/ipc_server.rs:136-150`, `src/ipc.rs:75-78` | unchanged | out of scope (4.1) |
| KS-39 probe drops a connection | `src/bin/swch.rs:75` | unchanged | fixed as a side effect (D8) |
| KS-40 machine-wide pipe name | `src/ipc.rs:16` | unchanged | out of scope (4.1); see Q1 |
| architecture "Why swch on kills the hook" (`keyboard_hook.rs:160-168`) | `book/src/architecture.md:170-172`; code `keyboard_hook.rs:185-208` | moved | book sync |
| roadmap Workstream A acceptance | `book/src/roadmap.md:94-96` | unchanged | |
| Callback (adapter) | `src/hook/keyboard_hook_callback.rs:215-242`; `decide_event` `:150-165`; `foreground_changed` `:250-262`; `HookEnv` `:95-131` (private) | n/a | gate points |
| Engine | `Engine` `src/engine/mod.rs:264-285`; `on_foreground_changed` `:331-334`; `Gestures` `gestures.rs:75-82` (`last_generation` `:81`); `KeyState` `keys.rs:60-79`; `sync_modifiers` `:88-96` | n/a | |
| Harness | `Scenario` `src/engine/testing.rs:118-182`; `down_unseen` `:255-273`; `foreground_changed` `:276-282`; `key_event` `:388-407`; failure text `:504-524` | n/a | |
| interprocess 2.4.2 (`Cargo.lock:65-66`) | listener SD: `os/windows/local_socket.rs:17-29`; SDDL parse: `security_descriptor/owned.rs:52-58`; no I/O timeouts: `named_pipe/local_socket/stream.rs:25-27,53-55`; local-socket connect waits without bound: `stream.rs:38-45`, `ctor.rs:84-86`; connect with a wait mode: `ctor.rs:94-105`; busy pipe -> `WaitNamedPipeW`: `c_wrappers.rs:171,201-203`; SD clone writes the group into the owner: `try_clone.rs:66-69` | n/a | facts the design relies on |

Result of the re-check: KS-01, KS-02 and the timeout gap are all still present on `b056b72`. Nothing is already fixed.

## Goal and non-goals

**After the lane**

- `App::run` installs the hooks once, on the main thread, and removes them there at shutdown.
  - `swch on` and `swch off` only flip `KeyboardHook::enabled`.
  - While remapping is off, the LL callback only calls `CallNextHookEx`, and the foreground WinEvent callback returns at once.
  - No IPC path reaches `SetWindowsHookExW`, `SetWinEventHook` or the `Unhook*` calls (KS-01).
- A real `on`<->`off` transition resets the engine (the KS-31 part):
  - pending Tap, Hold, DoubleTap, chord and sequence state is dropped;
  - the Hold generation keeps counting;
  - the key table is rebuilt from what Windows reports held.
- `status` reports the enabled flag.
- The pipe has an explicit SDDL: SYSTEM and Administrators get full control, interactive users get read and write, and a medium label carries NW+NR. A non-elevated `swch` reaches an elevated daemon (KS-02).
- `swch`:
  - bounds how long it waits to connect, and gives up 5 s after sending (`REPLY_TIMEOUT`);
  - `swch open` counts "access denied", "busy" and "no answer" as a running daemon and starts no second one;
  - the probe of `swch open` is a real `status` request (fixes KS-39 as a side effect);
  - the placeholder is gone.
- The pipe name is injectable (`ipc_server::spawn(pipe_name, control)`, `ipc::request(pipe_name, ..)`). `tests/ipc_roundtrip.rs` runs a real pipe without any hook.

**Non-goals (later items)**

| Left out | Item |
|---|---|
| KS-20 mutex and a fatal pipe failure. This lane does only the `swch open` detection, which 1.5 also lists | 1.5 |
| Cancelling running macros on `off` (epoch) | 2.1 |
| Reset on reload, unlock or desktop switch, keeping the blocked records there. 2.3 extends `Engine::reset` | 2.3 |
| Hook watchdog and reinstall. Status cannot see a hook that Windows removed silently (KS-08) | 2.4 |
| KS-26 (read deadline, size cap, one handler), KS-37 (exit reply race, an empty reply counts as success), KS-40 (per-session name), checking the pipe server's identity | 4.1 |

**Later items that depend on this lane:** 1.5 (needs the SDDL, `roadmap.md:144`), 2.1, 2.3, 2.4, 4.1 (extends `tests/ipc_roundtrip.rs`), 4.3.

## Decisions

### D1: `on`/`off` is an `AtomicBool` gate; only the main thread installs and uninstalls

**What.**

- `KeyboardHook` gains `enabled: AtomicBool`, initially `true`.
- `pub fn is_enabled(&self) -> bool` is a `load(Acquire)`.
- `pub fn set_enabled(&self, enabled: bool) -> Option<bool>` returns `Some(changed)`, or `None` if this thread is already inside `with_engine`.
  - It builds a `HookEnv` and runs under `with_engine`: `if self.enabled.swap(enabled, AcqRel) == enabled { return false }`, then `engine.reset(&env)`, then `true`.
  - So every transition happens under the engine lock.
- `enable()` and `disable()` are deleted.
- `install` and `uninstall` become `pub(crate)` and get the doc line "main thread only: the thread that pumps `GetMessageW` (KS-01)". The fields `hook_handle` and `foreground_hook` become private.
- Callback (`keyboard_hook_callback.rs:215`):
  - Right after the `code < 0` branch, and before `lparam` is dereferenced: `let hook = get_hook(); if !hook.is_enabled() { return CallNextHookEx(None, code, wparam, lparam) }`.
  - `decide_event` re-checks under the lock: `hook.with_engine(|engine| hook.is_enabled().then(|| engine.on_key(event, &bindings, &env))).flatten()`.
- `foreground_changed` returns early when `!is_enabled()` and checks again inside its `with_engine` closure.

**Why.**

- Hooks belong to the installing thread, and that thread must pump messages:
  - LowLevelKeyboardProc: "The call is made by sending a message to the thread that installed the hook. Therefore, the thread that installed the hook must have a message loop."
  - UnhookWinEvent: "UnhookWinEvent fails if called from a thread different from the call that corresponds to SetWinEventHook."
- Never reinstalling removes the whole KS-01 chain, including the KS-31 install race and the `[FATAL]` prompt at shutdown.
- The fast check is the spec's "checked first", and it keeps the disabled path at one atomic load plus `CallNextHookEx`.
- The check under the lock closes a race. The callback passes the fast check, then `off` stores `false` and resets, then the callback takes the lock and arms a Hold. That Hold's timer would fire while remapping is off.

**Rejected.**

- Reinstalling through a `WM_APP` posted to the main thread: the spec says never reinstall, and a toggle needs no reinstall.
- A flag inside `Engine`: the spec puts the gate in the callback, before any lock.
- Keeping `enable`/`disable` as wrappers: an install path would stay one call away from IPC.

**Cost.** One `Acquire` load per event, plus one under the lock. No allocation.

### D2: Reset on a real transition only, in both directions

**What.** `Engine::reset(&mut self, env: &dyn Env)`, final form:

- `self.gestures.clear()`. The new `Gestures::clear` empties `taps`, `holds` and `double_taps` and keeps `last_generation`.
- `self.chords = Chords::new(); self.sequences = Sequences::new();`
- `self.keys.reset_from_os(env)`. The new `KeyState::reset_from_os`:
  - For every code in `1..=254` except `VK_LBUTTON`, `VK_RBUTTON`, `VK_MBUTTON`, `VK_XBUTTON1` and `VK_XBUTTON2`: `active = env.os_key_down(code)`; `last_down` becomes `Passed` if the key is down and `Unknown` otherwise.
  - Every other slot becomes `false` / `Unknown`.
  - `VIRTUAL_KEY(code)` is built in `keys.rs` with the same justification as `all_held` (`keys.rs:204-210`).

**Why.**

- Reset at `off`: otherwise a Hold armed before `off` fires on its `hold-timer` thread while remapping is off (KS-31), and chord, sequence or tap entries complete across the gap.
- Reset at `on`: the engine was blind while remapping was off. A key it swallowed before `off` and that was released during it would still read as held, so its next press would count as an auto-repeat.
- Keep the generation: if it restarts, a `hold-timer` that went to sleep before `off` matches the first Hold armed after `on`, because the token is `{vk, generation}` (`mod.rs:221-227`). That is a KS-11 regression (red test S5).
- Seed held keys from the OS as `Passed`: while remapping was off, Windows saw every key-down. A key still held at `on` must have its repeats and its key-up pass. A table cleared to `Unknown` turns the next auto-repeat into a fresh press that a binding can swallow, and the key then stays held in Windows (red test S6).
- Skip the mouse buttons: the LL keyboard hook never reports their release. A seeded button would stay held for ever and stop auto-blockers from matching (A4's rule that every other held key is up in Windows).

**Rejected.**

- `*self = Engine::new()`: breaks S5 and S6.
- Keeping the suppressed records at `on`, as 2.3's "except blocked ones" does: the engine was blind, so those records are wrong, and the key's next press would be swallowed silently as a "repeat".
- Resetting on a no-op toggle: `swch on` while already on would wipe the live records of held, suppressed keys, and their key-ups would leak to Windows.
- Scanning the OS lazily at the next event on the hook thread: about 250 `GetAsyncKeyState` calls inside the callback budget, and the state of the delivered key lags there.

**Cost.** The IPC worker holds the engine lock for at most 249 `GetAsyncKeyState` calls and three `Vec::clear`s. Estimated well under 1 ms (unverified). A key event that arrives meanwhile waits, once per toggle.

**Thread.** IPC connection worker. `GetAsyncKeyState` "may be called from any thread" (`keyboard_hook_callback.rs:117-119`).

**2.3.** `Engine::reset` is the hook point. 2.3 adds a variant that keeps the blocked records (for reload and unlock, where the engine was not blind) and adds macro cancellation.

### D3: The foreground WinEvent callback does nothing while remapping is off

**What.** As in D1.

**Why.** It keeps one simple invariant: while remapping is off, only the reset in `set_enabled` and stale `on_timer` calls touch the engine. `on` rebuilds the key table anyway.

**Rejected.** Letting it run: harmless, but pointless, and the invariant would need an exception.

### D4: A `DaemonControl` trait; `ipc_server` no longer knows `KeyboardHook`

**What.** In `src/hook/ipc_server.rs`:

```rust
pub trait DaemonControl: Send + Sync {
    fn set_enabled(&self, enabled: bool) -> Result<(), String>;
    fn status(&self) -> DaemonStatus;
    fn request_exit(&self) -> Result<(), String>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaemonStatus { pub enabled: bool, pub bindings: usize }
fn dispatch(cmd: Command, control: &dyn DaemonControl) -> String
```

Replies from `dispatch`:

| Command | Reply |
|---|---|
| `On` | `OK: enabled` (also when already on) |
| `Off` | `OK: disabled` (also when already off) |
| `Status` | `OK: running (N bindings live)` or `OK: disabled (N bindings live)`. Same words as today, now taken from the flag |
| `Exit` | `OK: shutting down` |
| control returns `Err(m)` | `ERR: <context>: m` |

The real implementation is a private `struct DaemonHandle { hook: &'static KeyboardHook, main_thread_id: u32 }` in `src/core/app.rs`:

- `set_enabled` maps `hook.set_enabled(..)`, with `None` becoming `Err("the engine is busy on this thread")`.
- `status` combines `hook.is_enabled()` and `hook.bindings().len()`.
- `request_exit` returns `Err` for thread id 0. Otherwise it calls `PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0))` and maps its error to a `String`.

Deleted from `ipc_server`: `MAIN_THREAD_ID`, `set_main_thread_id`, the `windows` imports and the `KeyboardHook` import. The call at `app.rs:82-83` goes too.

**Why.**

- The rg acceptance becomes structural: `ipc_server.rs` cannot name `install`.
- Integration tests run the real server with a fake control and never touch a `KeyboardHook`. Even a later regression cannot make a test install a hook.
- The second `MAIN_THREAD_ID` disappears (R4).
- No new module cycle: `core::app -> hook::ipc_server` already exists (`app.rs:14`), and `ipc_server` depends only on `ipc`.

**Rejected.**

- Passing `&'static KeyboardHook` and testing with a leaked hook: any later regression in `set_enabled` would install a real global hook during `cargo test`.
- Implementing the trait on `KeyboardHook`: it would have to know the main thread id.

**Cost.** One small trait, one private adapter, and one `Arc::clone` per connection.

### D5: Injectable pipe name; `spawn` returns `io::Result`

**What.**

- `pub fn spawn(pipe_name: &str, control: Arc<dyn DaemonControl>) -> io::Result<()>`.
- A private `fn create_listener(pipe_name: &str) -> io::Result<Listener>`: `pipe_name.to_ns_name::<GenericNamespaced>()`, then `ListenerOptions::new().name(..).security_descriptor(pipe_security()?).create_sync()`.
- `App::run` calls `ipc_server::spawn(ipc::PIPE_NAME, Arc::new(DaemonHandle { .. }))`:
  - on `Err(e)` it prints today's two lines verbatim (`ipc_server.rs:50-58`; the book cites them);
  - on `Ok` it prints the `✓` line.
- The thread name `ipc-server` is unchanged. The connection workers are unchanged (4.1).

**Why.** The spec asks for a unique name in tests. Startup stays non-fatal until 1.5.

### D6: The pipe's SDDL

**What.**

```rust
pub const PIPE_SDDL: &str = concat!(
    "D:P",                // protected DACL: nothing inherited
    "(A;;GA;;;SY)",       // LocalSystem: full control
    "(A;;GA;;;BA)",       // Administrators: full control (the elevated daemon creates instances)
    "(A;;GRGW;;;IU)",     // interactive users: read and write (a non-elevated swch, a debug daemon)
    "S:(ML;;NWNR;;;ME)",  // medium label: lower integrity can neither write nor read
);
pub fn pipe_security() -> io::Result<SecurityDescriptor>  // security_from_sddl(PIPE_SDDL)
fn security_from_sddl(sddl: &str) -> io::Result<SecurityDescriptor>
// = { let wide = sddl.to_wtf_16().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
//     SecurityDescriptor::deserialize(&wide) }
```

- `interprocess::os::windows::ToWtf16` is public (`os/windows.rs:14`, `path_conversion.rs:18-27`). `&Cow<U16CStr>` deref-coerces to `&U16CStr`, so the code never names `widestring` and no new dependency is needed.
- The listener takes the SD through `interprocess::os::windows::local_socket::ListenerOptionsExt::security_descriptor`. Later instances reuse it (`named_pipe/listener.rs:123-125`, `create_instance.rs:51-54`).
- **No `O:` or `G:` components.** interprocess 2.4.2's clone writes the group into the owner slot (`try_clone.rs:66-69`). Say so in a comment.

**Why.**

- Default DACL ("Named Pipe Security and Access Rights"): "grant full control to the LocalSystem account, administrators, and the creator owner. They also grant read access to members of the Everyone group and the anonymous account". An elevated creator's owner is Administrators, which is deny-only for the medium client (KS-02).
- Integrity label:
  - Mandatory Integrity Control: "objects you create receive your integrity level (medium or high)". The older design document says pipes get an implicit medium label instead. The explicit `ME` makes either reading safe.
  - Windows Integrity Mechanism Design: "The integrity level in the mandatory label can be set to a value less than or equal to the subject's integrity level. ... does not require SE_SECURITY_NAME privilege".
- `NR` keeps low-integrity (sandboxed) processes from even opening the pipe for reading, which shrinks the KS-26 surface. `WD` and `AN` are gone.
- `IU` rather than the logon SID: static, unit-testable, and it also admits a standard user whose daemon was elevated with another admin's credentials. The cross-session reach is Q1.

**Known trade-off.** `GW` includes `FILE_APPEND_DATA`, which equals `FILE_CREATE_PIPE_INSTANCE` ("Because FILE_APPEND_DATA and FILE_CREATE_PIPE_INSTANCE have the same definition, so FILE_GENERIC_WRITE enables permission to create the pipe."). The interprocess client opens the pipe with `GENERIC_READ|GENERIC_WRITE` (`c_wrappers.rs:135-147`), so IU needs it.

- Consequence: an interactive medium-integrity process can create a rogue instance and give `swch` false replies.
- It cannot control the daemon, and it already runs as the user.
- Follow-up: 4.1 checks the server PID.

**Rejected.**

- The audit's `(A;;GA;;;SY)(A;;GRGW;;;IU)` without BA: an elevated daemon must keep `FILE_CREATE_PIPE_INSTANCE` for its next instance, and only BA guarantees it, because IU can be absent (a non-interactive logon).
- `(A;;GA;;;OW)` for the owner: I could not confirm its semantics from a doc I can quote (unverified), and every supported launch already has BA or IU.
- An SDDL built at runtime with the logon SID: more `unsafe` code, new `windows` features (`Win32_Security*`), and it is unverified that linked UAC tokens share that SID.

**Cost.** One parse per pipe instance: once at startup, then once per accepted connection, on `ipc-server`.

### D7: The client lives in the library: bounded connect, 5 s deadline on a helper thread

**What.** In `src/ipc.rs`:

```rust
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(5);
pub fn pipe_path(pipe_name: &str) -> String              // format!(r"\\.\pipe\{pipe_name}")
#[derive(Debug)]
pub enum RequestError { NotRunning, AccessDenied, Busy, TimedOut(Duration), Io(io::Error) }
impl fmt::Display for RequestError   // one arm per variant, texts below
impl std::error::Error for RequestError
impl RequestError { pub fn daemon_running(&self) -> Option<bool> }  // NotRunning->Some(false); AccessDenied|Busy|TimedOut->Some(true); Io->None
pub fn request(pipe_name: &str, command: &Command, timeout: Duration) -> Result<(bool, String), RequestError>
fn exchange(path: &str, wire: &'static str, timeout: Duration) -> Result<String, RequestError>
fn classify_connect_error(error: io::Error) -> RequestError
```

`request`:

- Spawns a thread named `ipc-request` with `thread::Builder`; a spawn error becomes `Io`. The thread runs `exchange` and `send`s the result into an `mpsc::sync_channel(1)`.
- The caller waits with `recv_timeout(timeout)`:
  - `Timeout` -> `TimedOut(timeout)`;
  - `Disconnected` -> `Io(io::Error::other("the request thread ended without a result"))`;
  - `Ok(r)` -> `r.map(|line| parse_response(&line))`.

`exchange`:

1. Connect with `interprocess::os::windows::named_pipe::DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path_with_wait_mode(path, interprocess::ConnectWaitMode::Timeout(timeout))`. Errors go through `classify_connect_error`.
2. `(&stream).write_all(wire.as_bytes())`, with **no** flush: interprocess's flush is `FlushFileBuffers`, which on a pipe waits for the reader.
3. `BufReader::new(&stream).read_line(..)`. I/O errors become `Io`.

`classify_connect_error` compares `raw_os_error()` as a `WIN32_ERROR` (`u32::try_from(code).ok().map(WIN32_ERROR)`, no casts):

| Error | Result |
|---|---|
| `ERROR_FILE_NOT_FOUND` | `NotRunning` |
| `ERROR_ACCESS_DENIED` | `AccessDenied` |
| `ERROR_PIPE_BUSY` or `ERROR_SEM_TIMEOUT` | `Busy` |
| `io::ErrorKind::TimedOut` | `Busy` (interprocess turns a pipe still busy after the wait into `TimedOut`, `misc.rs:131-135`) |
| anything else | `Io` |

Display texts:

| Variant | Text |
|---|---|
| `NotRunning` | `no daemon is listening on the pipe` |
| `AccessDenied` | `access to the daemon's pipe is denied` |
| `Busy` | `the daemon's pipe is busy` |
| `TimedOut(d)` | `the daemon did not answer within {d:?}` (renders as `5s`) |
| `Io(e)` | `{e}` |

**Why.**

- interprocess named-pipe streams reject I/O timeouts (`no_timeouts()`, `stream.rs:25-27,53-55`).
- The local-socket `Stream::connect` waits on a busy pipe with `WaitNamedPipeW(FOREVER)` (`stream.rs:40`, `ctor.rs:84-86`, `c_wrappers.rs:201-203`). Only the named-pipe API takes `ConnectWaitMode::Timeout`.
- One helper thread plus `recv_timeout` bounds connect, write and read with a single deadline, with no `unsafe`, no overlapped I/O and no new feature.
- WaitNamedPipeW: "If the time-out interval expires, the WaitNamedPipe function will fail with the error ERROR_SEM_TIMEOUT", and "If no instances of the specified named pipe exist, the WaitNamedPipe function returns immediately".
- `pipe_path` is the same as interprocess's namespaced mapping ("Namespaced strings have `\\.\pipe\` prepended to them", `name_type.rs:16-18`). The round-trip tests prove that server and client meet.

**Rejected.**

- Overlapped `ReadFile` with `CancelIoEx`: needs raw handles and `unsafe`.
- Polling in `PIPE_NOWAIT` mode: Microsoft discourages it.
- `CancelSynchronousIo`: interprocess handles are overlapped.
- Separate connect and read timeouts: the acceptance is a single bound ("exits within 5 s").

**Cost.** One thread per `swch` call. A request that times out leaves its helper thread blocked until the server closes the pipe or the process exits. `swch` exits at once; tests leak one parked thread.

### D8: `swch open` probes with `status` and launches only on `NotRunning`

**What.** `swch open` sends `request(PIPE_NAME, &Command::Status, REPLY_TIMEOUT)`:

| Result | Behaviour |
|---|---|
| `Ok(_)` | prints `swch: daemon is already running` |
| `Err(e)`, `daemon_running() == Some(true)` | prints `swch: daemon is already running ({e})`, exit 0, no UAC prompt |
| `Err(e)`, `Some(false)` | launches as today (`swch.rs:80-125`) |
| `Err(e)`, `None` | `Err("cannot tell whether a daemon is running ({e}); not starting another")`, exit 1 |

`send`:

| Result | Output | Exit |
|---|---|---|
| `Ok((true, m))` | `m` on stdout | 0 |
| `Ok((false, m))` | `swch: m` | 1 |
| `Err(NotRunning)` | ``swch: the daemon is not running. Use `swch open` to start it.`` | 1 |
| any other `Err(e)` | `swch: cannot reach daemon ({e}).` | 1 |

- Usage text: `on` becomes "Turn remapping on", `off` becomes "Turn remapping off (keys pass through)".
- Deleted: `connect`, `round_trip`, the placeholder (`:172-176`), the `Duration`/`BufRead`/`interprocess` imports and the old helper's `ipc::PIPE_NAME` import.

**Why.** This is the spec's denied/busy rule, extended to a hung daemon (timeout). Only "no pipe" proves that no daemon runs. A `status` request leaves no `[WARN] IPC read error` in the daemon console (KS-39).

**Rejected.** A probe that only connects: it brings KS-39 back and cannot tell a hung daemon from a live one.

### D9: Test strategy; no test can install a hook

- The engine is tested in scenarios, using the harness steps `remapping_off`/`remapping_on`.
- The gate is tested in adapter unit tests on a local `KeyboardHook`, plus the one test that uses the global hook.
- Server and client are tested over real pipes with unique names and a fake `DaemonControl`.
- **Red-replay rule (tester).** Every new test that touches `set_enabled`, `dispatch`, `spawn` or `request` fails to *compile* on its parent. Never adapt a test so that it compiles on `master`. There, `dispatch(Command::On, hook)` calls `install()` (`ipc_server.rs:115`, `keyboard_hook.rs:169-174`) and would install a global LL hook in the test process.
- For the same reason, no `dispatch` test is added before C7: on C3, a `dispatch(On, &hook)` test would compile and install a hook.

### D10: `uninstall` reports a failed `UnhookWinEvent`

**What.** `if let Err(error) = unsafe { UnhookWinEvent(..) }.ok() { eprintln!("[WARN] could not remove the foreground hook: {error}") }`, then unhook the LL hook as today and return its `Result`.

**Why.** Step 5 of KS-01: the result was discarded. On the main thread the call should never fail, so log the failure instead of failing shutdown.

## Threads of the new code paths

| Code path | Thread | Notes |
|---|---|---|
| `if !is_enabled()` gate in the LL callback | main | one atomic load before `lparam` is read |
| gate in `foreground_changed` | main | |
| `KeyboardHook::set_enabled` -> `Engine::reset` | IPC connection worker (existing, unnamed) | under the engine lock: `GetAsyncKeyState` only; no action, `SendInput`, spawn or pump |
| `DaemonHandle::request_exit` | IPC connection worker | posts `WM_QUIT` to main; it used to live in `ipc_server.rs:136-150` |
| `create_listener`: first pipe instance / later instances | main (`App::run`) / `ipc-server` (`accept`) | SD parsed for each instance |
| `ipc::request` -> `exchange` | new `ipc-request` thread (swch, tests); the caller blocks in `recv_timeout` | |
| `hold-timer` | unchanged | after a reset its token finds nothing |

No new hook, window, timer or COM apartment. Install and uninstall stay on the main thread.

## Win32 API choices

No change to `Cargo.toml`: every feature used is already listed (`Cargo.toml:41-51`), and interprocess's own `windows-sys` features cover the security calls.

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Hook install and removal (same calls as today, now main thread only) | `windows::Win32::UI::WindowsAndMessaging::{SetWindowsHookExW, UnhookWindowsHookEx}` (return `Result`); `windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent}` (return `HWINEVENTHOOK` and `BOOL`) | `Win32_UI_WindowsAndMessaging` `:43`, `Win32_UI_Accessibility` `:45` (yes) | main only (`App::run`) | `UnhookWinEvent`: `BOOL::ok()`, logged (D10) | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc ; https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwinevent ("Call this function from the same thread that installed the event hook.") |
| Pass events through while off | `windows::Win32::UI::WindowsAndMessaging::CallNextHookEx(Option<HHOOK>, i32, WPARAM, LPARAM) -> LRESULT` | `:43` (yes) | main (LL callback) | none | lowlevelkeyboardproc: "If nCode is greater than or equal to zero, and the hook procedure did not process the message, it is highly recommended that you call CallNextHookEx" |
| OS key snapshot for `reset` | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(i32) -> i16`, through the existing `HookEnv::os_key_down` (`keyboard_hook_callback.rs:116-122`) | `Win32_UI_Input_KeyboardAndMouse` `:44` (yes) | IPC connection worker, under the engine lock | returns 0 on failure (another desktop, UIPI), read as "up": the table then starts empty, as before this lane | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| `exit` | `windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(u32, u32, WPARAM, LPARAM) -> windows::core::Result<()>` (windows-0.62.2 `WindowsAndMessaging/mod.rs:1872`) | `:43` (yes) | IPC connection worker (moved from `ipc_server.rs:146-148` into `DaemonHandle`) | `Err` becomes an `ERR:` reply. Doc: "fails if the specified thread does not have a message queue" | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew |
| Pipe SD | interprocess `os::windows::security_descriptor::SecurityDescriptor::deserialize` (wraps `ConvertStringSecurityDescriptorToSecurityDescriptorW`, `c_wrappers.rs:130-143`); `os::windows::local_socket::ListenerOptionsExt::security_descriptor`, which reaches `CreateNamedPipeW` with `SECURITY_ATTRIBUTES` (`create_instance.rs:51-75`) | none needed in our `windows` dependency | main (first instance, `App::run`); `ipc-server` (later instances, in `accept`) | `io::Error`; the parse can fail with `ERROR_INVALID_PARAMETER` or `ERROR_NONE_MAPPED` | https://learn.microsoft.com/en-us/windows/win32/api/sddl/nf-sddl-convertstringsecuritydescriptortosecuritydescriptorw ; https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights ; https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings ; https://learn.microsoft.com/en-us/windows/win32/secauthz/sid-strings ; https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control ; https://learn.microsoft.com/en-us/previous-versions/dotnet/articles/bb625963(v=msdn.10) |
| Client connect with a bounded wait | interprocess `os::windows::named_pipe::DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path_with_wait_mode(path, ConnectWaitMode::Timeout(d))` (`ctor.rs:94-105`): `CreateFileW(GENERIC_READ\|GENERIC_WRITE, FILE_FLAG_OVERLAPPED)`, then `WaitNamedPipeW(remaining)` on `ERROR_PIPE_BUSY` | n/a | `ipc-request` (swch, tests) | OS errors 2 / 5 / 231 / 121 and `TimedOut`, mapped as in D7 | https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew |
| Error codes | `windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_ACCESS_DENIED, ERROR_PIPE_BUSY, ERROR_SEM_TIMEOUT, WIN32_ERROR}` (verified in windows-0.62.2 `Foundation/mod.rs:2355, 1143, 3692, 3943`) | `Win32_Foundation` `:42` (yes) | any | n/a | (constants) |

- **Not relevant:** `LLKHF_INJECTED`, the extended-key flag and `scanCode`. The lane adds no injection and reads none of them; the off gate passes injected and physical events alike.
- **Elevation and UIPI:**
  - D6 covers the pipe.
  - `PostThreadMessageW` stays inside one process.
  - The reset's `GetAsyncKeyState` runs in the elevated daemon. In a non-elevated daemon it degrades the same way as today's modifier resync.
- **Token inheritance:** not applicable.

## Touch set

| File | Change | Why / commit |
|---|---|---|
| `src/engine/mod.rs` | `Engine::reset`: naive in C1 (`*self = Self::new(); self.keys.sync_modifiers(env, None);`), final in C3 (D2). Docs: "## State" mentions the reset; "## Threads and locking" mentions that the IPC worker toggles under the lock | D2; C1, C3 |
| `src/engine/gestures.rs` | `pub(super) fn clear(&mut self)` and test U1 | D2; C3 |
| `src/engine/keys.rs` | `const MOUSE_BUTTONS: [VIRTUAL_KEY; 5]`, `pub(super) fn reset_from_os(&mut self, env: &dyn Env)`, module doc, test U2 | D2; C3 |
| `src/engine/env.rs` | `os_key_down` doc (`:32-42`): "a reset asks about every keyboard code" | D2; C3 |
| `src/engine/testing.rs` | see the list below the table | D9; C1 |
| `src/engine/scenarios.rs` | S1-S4 (C1), S5-S6 (C2) | C1, C2 |
| `src/hook/keyboard_hook.rs` | `enabled`, `is_enabled`, `set_enabled`; delete `enable`/`disable`; `install`/`uninstall` become `pub(crate)` with the main-thread doc; fields private; D10; docs on the `engine` field (`:47-50`) and on `with_engine` (`:66-78`): "decides or resets"; tests K1, K1b, K1c | D1, D10; C4 |
| `src/hook/keyboard_hook_callback.rs` | fast gate after `code < 0`; gate under the lock in `decide_event`; gate in `foreground_changed`; `HookEnv` and `new` become `pub(super)`; module docs (`:1-60`): new step "1a", the IPC worker under the lock; test K2; K3 extends `:342-385` | D1, D3; C4 |
| `src/hook/ipc_server.rs` | C4: `dispatch` handles On/Off/Status through `set_enabled`/`is_enabled` (no new test here, D9). C5: `PIPE_SDDL`, `pipe_security`, `security_from_sddl`, `create_listener`, tests P1-P3. C7: `DaemonControl`, `DaemonStatus`, `spawn(pipe_name, control) -> io::Result<()>`, `dispatch(&dyn DaemonControl)`; delete `MAIN_THREAD_ID`, `set_main_thread_id`, the `windows` and `KeyboardHook` imports; module doc (`:1-10`); tests I1-I3. The file must contain no `install` token (rg acceptance) | D4-D6 |
| `src/core/app.rs` | C7: `DaemonHandle` and its `impl DaemonControl`. In `run`: drop `:82-83`, spawn with `Arc::new(DaemonHandle { hook, main_thread_id: main_tid })`, print the two warnings on `Err`. Tests A1, A2 | D4, D5 |
| `src/ipc.rs` | C4: `Command::On/Off` docs (`:21-25`) say "turns remapping on/off; the hooks stay as they are", with no `install` token. C6: the client of D7, module doc (`:1-12`), tests C-1 to C-4 | D7; C4, C6 |
| `src/bin/swch.rs` | C6: as in D8, plus module doc (`:1-15`) | D8 |
| `tests/ipc_roundtrip.rs` (new) | C6: helpers and T2-T6. C7: T1 | D9 |
| `README.md` | texts in "Docs texts" | the CLI visibly changes |
| `CHANGELOG.md` | texts in "Docs texts" | Keep a Changelog |
| `src/config/default_config.ron` | none: no config meaning changes | |
| `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/main.rs`, `src/lib.rs` | none | |
| `book/src/**` | none in this lane | phase book sync (Q5); lines listed below |

`src/engine/testing.rs` (C1):

- a field `enabled: bool`, initially `true`;
- `remapping_off()` and `remapping_on()`: each calls `engine.reset(&self.env)`, flips the flag, and records the step with the label `remapping off at +N ms` or `remapping on at +N ms`;
- `key_event` while off skips the engine: it uses `Decision::default()`, updates the OS state as for a passed event, and records the step;
- `foreground_changed` while off calls `fail(..)`, because the adapter does not report it while off;
- a module doc paragraph.

Book lines for the phase book sync:

- `build-and-run.md`: 153-231, 237-249, 298, 379-426, 485, 498, 500.
- `architecture.md`: 163-164, 170-172, 265-273, 441-483, 498.
- `gaps.md`: 29, 108-110.
- `overview.md`: 25, 172, 187-188.
- audit status lines for KS-01 and KS-02, plus notes for KS-31 and KS-39.

## Commits, in order

Rules for every commit:

- Gate: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
- Only C2 is a declared red commit. fmt and clippy pass, and exactly S5 and S6 fail, with the texts below; every other test passes. If any other test turns red, stop and report.
- Every message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- A red commit quotes the failing lines in its body.
- The other commits add API, so their tests fail to compile on the parent. Replay them by copying only the new test code onto the parent. D9's rule applies.

Shared scenario configs:

- `HOLD_F13 = ( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Hold(200)) ] )`
- `TAP_F13 = ( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Tap(200)) ] )`

Failure texts follow `testing.rs:504-524`: `scenario: <label>: ...`.

### C1. `Engine: Engine::reset and remapping off/on steps in the scenario harness` (green)

- The naive `Engine::reset` and the harness changes from the Touch set.
- Every new scenario is green here and stays green with the final reset. On `master` they do not compile (`no method named remapping_off`).

| Test | Script | Expected |
|---|---|---|
| S1 `remapping_off_drops_a_pending_hold` (KS-31) | HOLD_F13: down F13 (suppressed, nothing fired); advance 50; `remapping_off`; advance 300 -> nothing fired; up F13 -> passed; `remapping_on`; down F13 -> suppressed; advance 199 -> nothing; advance 1 -> fired `[SwitchLanguage]`; up F13 -> suppressed; total `[SwitchLanguage]` | green |
| S2 `tap_swallowed_before_remapping_off_works_again_after_remapping_on` | TAP_F13: down F13 suppressed; advance 20; `remapping_off`; up F13 -> passed (comment: Windows gets a lone key-up, D9/Q2); `remapping_on`; advance 1000; down F13 -> suppressed, nothing fired; advance 50; up F13 -> suppressed, fired `[SwitchLanguage]`; total `[SwitchLanguage]` | green. Without a reset the second press would count as a repeat and fire nothing |
| S3 `remapping_off_drops_a_pending_chord` | `( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )`: down J suppressed; `remapping_off`; up J passed; `remapping_on`; advance 10; down K -> suppressed, nothing fired; up K -> suppressed; total `[]` | green |
| S4 `remapping_off_drops_a_pending_sequence` | `( bindings: [ (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage) ] )`: down F13 suppressed; up F13 suppressed; `remapping_off`; `remapping_on`; down F14 -> passed, nothing fired; up F14 passed; total `[]` | green |

### C2. `Tests: red-first: a reset keeps the Hold generation and the keys Windows holds (KS-11, KS-31)` (RED)

| Test | Script | PREDICTED RED on C1 |
|---|---|---|
| S5 `stale_hold_timer_from_before_remapping_off_does_not_fire_a_newer_press` | HOLD_F13: down F13 (+0); advance 50; `remapping_off`; up F13; `remapping_on`; advance 50; down F13 (+100) -> suppressed; advance 150 -> `expect_nothing_fired`; advance 50 -> `expect_fired(&[SwitchLanguage])`; up F13 -> suppressed; total `[SwitchLanguage]` | `scenario: advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]`. The naive reset restarts `last_generation`, so the timer `{F13, 1}` of the +0 press matches the Hold of the +100 press |
| S6 `key_held_through_remapping_on_keeps_its_repeats_and_its_release` | TAP_F13, `.check_nothing_stuck()`: `remapping_off`; down F13 -> passed; `remapping_on`; advance 500; repeat F13 -> `expect_passed`, nothing fired; up F13 -> passed, nothing fired; down F13 -> suppressed; advance 50; up F13 -> suppressed, fired `[SwitchLanguage]` | `scenario: repeat F13 at +500 ms: expected the event to be passed, it was suppressed`. The naive table forgets F13, so the repeat reads as a fresh press and arms the Tap |

### C3. `Engine: a reset keeps the Hold generation and takes held keys from Windows (KS-11, KS-31)` (green; makes S5 and S6 green)

- D2 in its final form, `Gestures::clear`, `KeyState::reset_from_os`, and the doc in `env.rs`.
- U1 `clear_drops_every_entry_and_keeps_the_generation` (`gestures.rs`):
  1. Arm a tap, a double-tap and a hold (token1) on F13, then `clear`.
  2. `!tap_pending`, `!double_tap_pending`, `!hold_pending`.
  3. Arm a hold again: token2 != token1.
  4. `fire_hold(token1) == None`; `fire_hold(token2) == Some(..)`.
- U2 `reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` (`keys.rs`):
  1. A local Env reports `VK_A`, `VK_LSHIFT`, `VK_SHIFT`, `VK_LBUTTON` and `VK_XBUTTON2` as down.
  2. State before the reset: `press(VK_J)` + `mark_blocked(VK_J.0)`, and `press(VK_F13)` + `mark_passed(VK_F13.0)`.
  3. After `reset_from_os`:
     - A, LSHIFT and SHIFT are active and `is_passed`;
     - LBUTTON and XBUTTON2 are not active;
     - J is not active and `!is_blocked`;
     - F13 is not active and `!is_passed`;
     - `press(VK_A)` returns `true` (a repeat).
- Red on the parent: compile error, `no method named clear` / `reset_from_os`.
- Invariants: per-key index < 256; no allocation; the engine stays free of Win32 calls and reads the OS only through `Env`.

### C4. `Hook: swch on/off toggle an enabled flag; the hooks stay installed on the main thread (KS-01, KS-31)` (green)

- D1, D3, D10.
- `ipc_server::dispatch`:
  - On/Off: `match hook.set_enabled(..) { Some(_) => format_ok("enabled"/"disabled"), None => format_err("the engine is busy on this thread") }`;
  - Status from `is_enabled()`.
- `ipc.rs` docs. README and CHANGELOG as in "Docs texts".
- Tests. All fail to compile on C3 (`no method named set_enabled found for struct KeyboardHook`).

**K1 `toggling_reports_changes_and_never_installs_a_hook`** (`keyboard_hook.rs`):

- a new hook reports `is_enabled()`;
- `set_enabled(true) == Some(false)`; `set_enabled(false) == Some(true)`; `!is_enabled()`;
- `set_enabled(false) == Some(false)`; `set_enabled(true) == Some(true)`;
- `!is_installed()` throughout.

**K1b `turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it`:**

1. Binding: F13, `Hold(200)`, `DoNothing`, `with_allow_extra_modifiers(true)`. A local `QuietEnv(Instant)` reports nothing held and no foreground window.
2. Arm through `with_engine(|e| e.on_key(KeyEvent::down(VK_F13), &hook.bindings(), &QuietEnv(t0)))`: token1.
3. `set_enabled(true) == Some(false)`; `with_engine(on_timer(token1)).fire == [DoNothing]`.
4. Key-up at t0+300. Arm again at t0+400: token2.
5. `set_enabled(false) == Some(true)`; `with_engine(|e| e.on_timer(token2)) == Some(Decision::default())`.

**K1c `set_enabled_refuses_a_nested_call`:** on its own thread with `recv_timeout(5 s)`, the pattern of `:336-349`. `hook.with_engine(|_| hook.set_enabled(false)) == Some(None)`, and `is_enabled()` is still true.

**K2 `disabled_hook_passes_every_event_without_asking_the_engine`** (callback):

- `hook_with(F13 DoNothing allow_extra)`;
- `set_enabled(false) == Some(true)`; `decide_event` for down and for up F13 returns `None`;
- after `set_enabled(true)`, down F13 gives `suppress`.

**K3:** rename `extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` to `extern_callback_swallows_a_bound_key_only_while_remapping_is_on`. Keep its asserts and append, on the same global hook:

- `get_hook().set_enabled(false) == Some(true)`;
- `call(f13, WM_KEYDOWN) != LRESULT(1)` and `call(f13, WM_KEYUP) != LRESULT(1)`;
- `get_hook().set_enabled(true) == Some(true)`;
- down and up `== LRESULT(1)`.

It stays the only test that touches `get_hook()`.

**Invariants:**

- the gate sits after `code < 0` and before `lparam` is dereferenced;
- every path returns exactly one of `LRESULT(1)` or `CallNextHookEx`;
- the sentinel check still runs before any state changes;
- nothing new runs under the lock except `reset`.

### C5. `IPC: the pipe gets an explicit security descriptor (KS-02)` (green)

- D6. `spawn`, still with the old signature, calls `create_listener(ipc::PIPE_NAME)`. README and CHANGELOG as in "Docs texts".
- Tests. They fail to compile on C4 (`cannot find value PIPE_SDDL` / `create_listener`).
  - P1 `pipe_sddl_parses`: `pipe_security().expect(..)`.
  - P2 `malformed_sddl_is_rejected`: `security_from_sddl("D:P(A;;GA;;;NOT_A_SID)").is_err()`. This shows P1 can fail.
  - P3 `listener_with_the_pipe_security_admits_this_process`:
    - `create_listener` on the unique name `key-switch-rs-unit-{pid}-sd.sock` returns `Ok`, so `CreateNamedPipeW` accepts the ME label without a privilege;
    - on a helper thread with `recv_timeout(5 s)`, `interprocess::local_socket::Stream::connect(name)` returns `Ok`, so the DACL admits this token (IU locally, BA in CI).

### C6. `swch: bounded connect, 5 s reply deadline; open counts denied, busy and silent daemons as running` (green)

- D7, D8. README and CHANGELOG as in "Docs texts".
- Unit tests in `ipc.rs`. On C5 they fail to compile (`cannot find function request` / `RequestError`).
  - C-1 `connect_errors_are_classified`:
    - `from_raw_os_error(2)` -> `NotRunning`; `5` -> `AccessDenied`;
    - `231` -> `Busy`; `121` -> `Busy`; `io::Error::from(ErrorKind::TimedOut)` -> `Busy`;
    - `87` -> `Io`.
  - C-2 `only_a_missing_pipe_means_no_daemon`: `daemon_running` for each variant.
  - C-3 `pipe_path_is_the_local_pipe_namespace`: `pipe_path("key-switch-rs.sock") == r"\\.\pipe\key-switch-rs.sock"`.
  - C-4 `timed_out_names_the_deadline`: `TimedOut(5 s).to_string() == "the daemon did not answer within 5s"`.
- Integration tests in the new `tests/ipc_roundtrip.rs`:
  - The helper `unique_pipe_name(tag)` returns `format!("key-switch-rs-test-{}-{tag}-{}.sock", process::id(), COUNTER.fetch_add(1, Relaxed))`. `Relaxed` is enough: the counter only has to be unique and publishes no data.
  - Dummy servers use interprocess directly.
  - On `master` the file fails to compile (E0425/E0432: no `request`, `REPLY_TIMEOUT`, `RequestError` or `pipe_path` in `key_switch_rs::ipc`).

| Test | Setup and assertions |
|---|---|
| T2 `request_times_out_against_a_server_that_never_replies` (acceptance 3) | Listener from `ListenerOptions` + `GenericNamespaced`. A thread accepts one connection and holds it for 30 s, never reading or writing. `request(&name, &Command::Status, REPLY_TIMEOUT)` returns `Err(TimedOut(_))`. The elapsed time is at least `REPLY_TIMEOUT - 50 ms` and under `REPLY_TIMEOUT + 2 s`. `REPLY_TIMEOUT == 5 s` |
| T3 `request_reports_not_running_without_a_pipe` | No server: `Err(NotRunning)` in under 1 s; `daemon_running() == Some(false)` |
| T4 `request_reports_access_denied_for_a_pipe_this_process_may_not_open` | Listener with `SecurityDescriptor::deserialize("D:P(A;;GA;;;SY)")`: `Err(AccessDenied)`; `daemon_running() == Some(true)` |
| T5 `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running` | `PipeListenerOptions::new().path(pipe_path(&name)).instance_limit(NonZeroU8::new(1)).create_duplex::<pipe_mode::Bytes>()`. A first client, `DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_path(&name))`, holds the only instance. `request(.., 500 ms)` matches `Err(Busy \| TimedOut(_))`; `daemon_running() == Some(true)`; elapsed under 3 s |
| T6 `request_reads_the_reply_of_a_server_that_answers` | The dummy reads one line and writes `OK: got <line>\n`. Result: `Ok((true, "got status"))` |

### C7. `IPC: the server takes its pipe name and a DaemonControl; the daemon passes a DaemonHandle` (green)

- D4, D5. README layout line; the CHANGELOG "Added" line is extended.
- Unit tests in `ipc_server.rs`, using `FakeControl { enabled: AtomicBool, fail: bool, exits: AtomicU32 }`:
  - I1 `dispatch_switches_remapping_and_status_reports_the_flag`: Status `"OK: running (3 bindings live)\n"`; Off `"OK: disabled\n"`; Status `"OK: disabled (3 bindings live)\n"`; Off again `"OK: disabled\n"`; On `"OK: enabled\n"`; Status running.
  - I2 `dispatch_reports_a_control_failure_as_err`: with `fail`, the On and Exit replies start with `"ERR: "` and contain the control's message.
  - I3 `dispatch_exit_asks_the_daemon_to_quit_once`: `"OK: shutting down\n"` and `exits == 1`.
- Unit tests in `app.rs`:
  - A1 `daemon_handle_status_follows_the_hook_flag`, on a leaked `KeyboardHook` with one binding:
    - `status() == DaemonStatus { enabled: true, bindings: 1 }`;
    - `set_enabled(false)` is `Ok`; `!hook.is_enabled()`; `!status().enabled`;
    - `set_enabled(true)`; `!hook.is_installed()`.
  - A2 `daemon_handle_refuses_exit_without_a_main_thread`: thread id 0 gives `Err`.
- Integration test T1 `status_off_on_status_round_trip_without_a_hook` (spec; acceptance 1):
  - `TestControl { enabled: AtomicBool, bindings: 2, toggles: Mutex<Vec<bool>> }`; `ipc_server::spawn(&name, control.clone())` is `Ok`.
  - `request` results:
    - Status `== (true, "running (2 bindings live)")`;
    - Off `== (true, "disabled")`, and the flag is false;
    - Status `== (true, "disabled (2 bindings live)")`;
    - On `== (true, "enabled")`;
    - Status: running.
  - `toggles == [false, true]`.
  - On C6 it fails to compile (E0061: `spawn` takes 1 argument; `DaemonControl` not found).

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| Brief: the integration test is green | T1 (and T2-T6): `cargo test --locked --test ipc_roundtrip` |
| Brief: no `SetWindowsHookExW` reachable from IPC code (shown with rg) | The tester runs the four rg checks listed below this table and pastes their output; also K1 (`!is_installed()`) |
| Brief: a read against a silent server errors after about 5 s (dummy server) | T2 |
| Spec: hooks installed once on main; on/off flip a flag checked first; when disabled, only `CallNextHookEx` | K2, K3, the rg checks, manual M1 |
| Spec: reset the transient state on toggle | S1-S6, U1, U2, K1b |
| Spec: status reports the real flag | I1, T1, A1, M1 |
| Spec: explicit SD through the `ListenerOptions` security descriptor; SY/BA full, IU read/write, medium label; SDDL with comments; parse test | P1, P2, P3; review of `PIPE_SDDL`; M2 |
| Spec: swch connect timeout and 5 s read deadline | T2, T5, C-1; M5 |
| Spec: denied or busy on connect counts as running for `open` | T4, T5, C-2; M3 |
| Spec: remove the dead placeholder | `rg -n "Duration::from_secs\(5\)" src/bin/swch.rs` finds nothing; `rg -n "REPLY_TIMEOUT" src` finds `ipc.rs` and `swch.rs` |
| Spec: injectable name, no hook | T1 (unique name, fake control), D4 |
| Roadmap A `roadmap.md:94`: after `swch off; swch on` the bindings work | M1 (also K1b, K2, K3, S1-S6) |
| Roadmap A `:95`: a non-elevated `swch status` gets an answer from an elevated daemon | M2 (also P3, T1) |
| Roadmap A `:96`: with a hung daemon, `swch` exits within 5 s | T2, M5 |
| Smoke checklist `roadmap.md:229` (item 1) | M1 |

The four rg checks:

1. `rg -n "SetWindowsHookExW|SetWinEventHook\(|UnhookWinEvent|UnhookWindowsHookEx" src tests` matches only `src/hook/keyboard_hook.rs`, inside `install` and `uninstall`.
2. `rg -n "fn (install|uninstall)|\.install\(\)|\.uninstall\(\)" src tests` matches:
   - in `keyboard_hook.rs`: the `pub(crate) fn` definitions, the error path of `install`, and `Drop`;
   - in `src/core/app.rs`: only `App::run`, on the main thread.
3. `rg -n "KeyboardHook|keyboard_hook|install|fn (enable|disable)\b" src/hook/ipc_server.rs src/ipc.rs src/bin/swch.rs` matches nothing.
4. `rg -n "fn set_enabled" -A 20 src/hook/keyboard_hook.rs` and `rg -n "impl DaemonControl for DaemonHandle" -A 30 src/core/app.rs` show no install call.

## Manual checks (human only)

Setup for all checks:

- Build with `cargo build --release --locked` and put `key-switch-rs.exe` and `swch.exe` in one folder.
- Start the daemon with `swch open` from a **non-elevated** console unless the row says otherwise. Use the default config.
- No `#[ignore]` stubs are added, because no test that could be automated needs a live hook.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | daemon running | `swch status`; CapsLock; `swch off`; `swch status`; CapsLock twice; `swch on`; `swch status`; CapsLock | `running (N bindings live)`; layout switches; `disabled (N bindings live)`; Caps Lock toggles (the key reaches Windows) and the layout does not switch; `running ...`; layout switches again | none (K2, K3, T1) |
| M2 | daemon started elevated by `swch open` | in the same non-elevated console: `swch status`, `swch off`, `swch on`, `swch exit` | every command gets an answer, no "Access is denied"; after `exit` the daemon window closes without `[FATAL]` or "Press Enter" | none (P3, T1) |
| M3 | daemon running elevated | non-elevated `swch open`; then `tasklist /FI "IMAGENAME eq key-switch-rs.exe"` | `swch: daemon is already running`; no UAC prompt; one process | none (T4, T5) |
| M4 | config `( keys: "CapsLock", action: SwitchLanguage, trigger: Hold(3000) )` | in a second console start `cmd /c "timeout /t 1 >nul & swch off"`, then at once hold CapsLock for 4 s | the layout does not switch | none (S1, K1b) |
| M5 | daemon running elevated | suspend `key-switch-rs.exe` (elevated Resource Monitor, Suspend Process); `swch status`; resume the process | after about 5 s: `swch: cannot reach daemon (the daemon did not answer within 5s).`, exit code 1 | none (T2) |
| M6 | debug daemon started with `cargo run --bin key-switch-rs` from a normal console | `cargo run --bin swch -- status` from another normal console | answered (the IU path; regression check for development) | none (P3) |
| M7 | daemon running | `swch off`, then Ctrl+C in the daemon console | clean exit, no `[FATAL]` (the main thread runs `uninstall` while remapping is off) | none |

## Docs texts (exact)

`src/config/default_config.ron`: unchanged.

### CHANGELOG.md, `## [Unreleased]`

**C4, under Fixed, first entry (before KS-04):**

> - KS-01: `swch off` followed by `swch on` no longer leaves remapping dead. The keyboard and foreground hooks are installed once, on the daemon's main thread, and stay installed until it exits; they used to be removed and reinstalled on a short-lived IPC thread, whose exit took the new hooks with it, and `swch exit` then stopped at a `[FATAL]` prompt. `swch off` now makes the hook pass every key on untouched and `swch on` turns remapping back on; `swch status` answers `disabled` while remapping is off (it used to say `running` whenever a hook handle was stored). Each switch drops pending `Tap`, `Hold`, `DoubleTap`, `chord` and `sequence` state, so a `Hold` armed just before `swch off` no longer fires (KS-31, an unverified report), and a key still held at `swch on` counts as already pressed. A key swallowed by a binding and released while remapping is off reaches Windows as a lone key-up. A running macro keeps running after `swch off` (roadmap item 2.1).

**C5, under Fixed, after the KS-01 entry:**

> - KS-02: the daemon's pipe has an explicit security descriptor: full control for SYSTEM and Administrators, read and write for interactive users, and a medium integrity label. `swch` in a normal console now reaches a daemon started elevated by `swch open`, a double-click or a Scheduled Task; it used to fail with "Access is denied. (os error 5)". Everyone and anonymous logons no longer get read access, and processes below medium integrity cannot open the pipe.

**C6, under Fixed, after KS-02:**

> - `swch` no longer waits forever for a daemon that accepts the connection but does not answer, or whose pipe stays busy: after 5 s it prints `swch: cannot reach daemon (the daemon did not answer within 5s).` and exits with 1. Without a daemon it prints ``swch: the daemon is not running. Use `swch open` to start it.``. `swch open` treats a pipe that refuses this console, a busy pipe and a daemon that does not answer as a running daemon and starts no second one (KS-20, partly; roadmap item 1.5 adds the single-instance guard). Its check is now a `status` request, so it no longer leaves an `IPC read error` or `IPC write error` warning in the daemon console (KS-39, an unverified report).

**C6, under Added, last entry:**

> - `tests/ipc_roundtrip.rs`: the `swch` client against test pipes with unique names: a server that never answers, a missing pipe, a pipe that refuses the client, a busy pipe and a server that answers.

**C7, append to that Added entry:**

> ..., and the daemon's pipe server answering `status`, `off`, `on`, `status` through a stand-in for the daemon, with no keyboard hook installed.

### README.md

**C4**

- `:99-100` -> "- Installs the low-level keyboard hook (`WH_KEYBOARD_LL`) and the foreground-change hook (`SetWinEventHook` / `EVENT_SYSTEM_FOREGROUND`) once, on the main thread, and removes them at shutdown"
- New bullet after `:102` -> "- `swch on` / `swch off` only flip an enabled flag (`set_enabled`); each switch resets the decision core: pending gestures are dropped and the keys held are re-read from Windows"
- New bullet after `:106` -> "- While remapping is off (`swch off`), passes every event straight to `CallNextHookEx` without reading it"
- `:528` -> `> swch off       # OK: disabled       (remapping off; every key reaches Windows)`
- `:529` -> `> swch on        # OK: enabled        (remapping on again)`
- `:538` -> `| `swch on`     | Turn remapping back on (idempotent). |`
- `:539` -> `| `swch off`    | Turn remapping off (idempotent): the hook stays installed and passes every key through; pending Tap/Hold/DoubleTap, chords and sequences are dropped, a running macro finishes. Useful for games or temporarily passing the OS's defaults through. |`
- `:540` -> `| `swch status` | Print "running" or "disabled" (remapping on or off) and the live binding count. |`

**C5**

- `:520-521` -> "- **`swch.exe`** — CLI driver that talks to the running daemon via a named pipe. Runs as normal user — **no UAC prompt** — also against the elevated daemon: the pipe grants SYSTEM and administrators full control and interactive users read and write, with a medium integrity label."
- `:58` -> `│   └── ipc_server.rs       - named-pipe listener (explicit SDDL); on/off/exit/status commands`
- `:544-546` -> append "The pipe admits interactive users at medium integrity or above, so a normal console can control an elevated daemon."

**C6**

- `:24` -> `├── ipc.rs              - IPC protocol and the swch client (bounded connect, 5 s reply deadline)`
- `:537` -> `| `swch open`   | Launch the daemon (a UAC prompt on every launch of a release build). Does nothing if a daemon already answers, refuses this console, is busy, or does not answer within 5 s. |`. This also fixes the old "first time" wording.
- `:544-546` -> "... If the daemon isn't running, they print a hint and exit 1. If it does not answer within 5 s, they print `swch: cannot reach daemon (the daemon did not answer within 5s).` and exit 1."

**C7**

- After `:21` -> `├── tests/ipc_roundtrip.rs  - pipe server and swch client over a real pipe, no keyboard hook`

## Risks and edge cases

| Id | Area (reviewer R#) | Risk | Handling |
|---|---|---|---|
| RK1 | R1 hook callback | The gate must sit after `code < 0` and before `lparam` is dereferenced, and nobody may drop the check under the lock as "redundant" | D1; K2 (path under the lock), K3 (extern path); a doc comment names the race |
| RK2 | R3 shared state, lock | The IPC worker now takes `Mutex<Engine>`, and the callback waits during `reset` (about 249 `GetAsyncKeyState` calls) | No action, `SendInput`, spawn or pump under the lock; once per toggle; the re-entry refusal of `with_engine` is unaffected (K1c) |
| RK3 | R4 atomics | `enabled`: `swap(AcqRel)` only under the lock, `load(Acquire)` everywhere; a stale fast-path read is harmless because of the check under the lock | A doc comment on the field states the pairing. The `MAIN_THREAD_ID` copies become one (`app.rs:16`) plus `DaemonHandle.main_thread_id`, a plain field published through `Arc` and the thread spawn |
| RK4 | R10 shutdown | `exit` while remapping is off: `WM_QUIT`, then `uninstall` on main (M7). The KS-37 reply race is unchanged | 4.1 |
| RK5 | R8 key identity | `reset_from_os` writes the sided and the generic modifier codes as Windows reports them, like `sync_modifiers` (`keys.rs:88-96`); mouse codes are skipped | U2 |
| RK6 | correctness | A key swallowed before `off` and released while off gives Windows a lone key-up | The spec says "only `CallNextHookEx`"; S2 pins it; Q2 |
| RK7 | correctness | A key pressed microseconds before the reset at `on` may not yet show in the async state, so its first auto-repeat reads as a fresh press | Negligible window; documented on `KeyState::reset_from_os` |
| RK8 | correctness | A Hold whose `on_timer` ran just before the reset at `off` can still run its action just after `OK: disabled` | Inherent race; the window is one lock hand-off |
| RK9 | correctness | Running macros (`Sequence`) continue after `off` | 2.1; CHANGELOG and README say so |
| RK10 | UIPI, security | IU with `GW` lets any interactive medium process create a pipe instance and answer `swch` falsely (D6 quote) | It cannot control the daemon, and it already runs as the user; 4.1 checks the server PID |
| RK11 | security, KS-40 | IU spans sessions: another interactive user can toggle or stop the daemon, and `swch open` in session 2 reports "running" for session 1's daemon | Q1; per-session name in 4.1 |
| RK12 | CI | P3, T1 and T6 need the test token to be admitted. GitHub Windows runners run "as administrators with User Account Control (UAC) disabled" (https://docs.github.com/en/enterprise-server@3.17/actions/concepts/runners/github-hosted-runners, wording unverified), so BA applies | If CI ever runs as a non-admin, non-interactive user, these tests fail fast with `AccessDenied` instead of hanging |
| RK13 | tests | T2 adds about 5 s. Parked `ipc-request` and dummy threads live until the test binary exits | Accepted (Q6) |
| RK14 | interprocess | `try_clone.rs:66-69` writes the group into the owner slot | `PIPE_SDDL` has no `O:` or `G:`, and a comment says why |
| RK15 | red replay | Adapting any new test so that it compiles on `master` can install a global hook through `dispatch(On)` | D9; the tester replays compile errors as compile errors only |
| RK16 | R2, R5-R7, R9 | Nothing touches `SendInput`, builders, specificity or the registry. The only new Display-bearing enum is `RequestError` | `RequestError` has one Display arm per variant (C-4) |

**Follow-ups, outside this lane:**

| Item | Follow-up |
|---|---|
| 4.1 | KS-26, KS-37 (an empty reply counts as success, `ipc.rs:75-78`), KS-40, server-PID check, naming the connection workers and logging a failed spawn (`ipc_server.rs:72` still uses `thread::spawn`) |
| 2.1 | macro epoch |
| 2.3 | reset on reload or unlock |
| 2.4 | watchdog, for a hook that Windows removes silently |

## Questions for the orchestrator

1. **IU reaches other sessions (KS-40).** Recommendation: accept IU now; it is the spec's first option. It is static, can be tested, and works when another admin's credentials elevated the daemon. Item 4.1 makes the pipe name per-session, which removes the cross-session reach. The logon SID can be revisited then, once someone verifies that linked UAC tokens share it. Not blocking.
2. **Lone key-up after `off`** for a key swallowed before it (RK6). Recommendation: accept. The spec requires a pure pass-through while remapping is off, and avoiding the lone key-up would mean feeding the engine while "off". Not blocking.
3. **`swch open` with a connect error it cannot classify** (`RequestError::Io`). Recommendation: refuse to launch and exit 1 with the error. Only "no pipe" proves that no daemon runs, and a second daemon is worse than an explicit message (KS-20). Not blocking.
4. **KS-26, KS-37, KS-40 and connection-thread naming** wait for 4.1, although `ipc_server.rs` is touched. Recommendation: yes, to keep the lane scoped. The SDDL already removes read access for Everyone and Anonymous, which shrinks the KS-26 surface. Not blocking.
5. **Book pages** (build-and-run, architecture, gaps, overview, audit status lines) go to the phase book sync, as in A4; the exact lines are listed under the Touch set. Recommendation: yes. No config page changes meaning. Not blocking.
6. **T2 uses the real 5 s constant**, adding about 5 s to each suite run. Recommendation: keep it. The acceptance says "about 5 s", and an injected shorter timeout would only prove the mechanism. Not blocking.

## Cut readiness checklist

### Spec

- [x] KS-01 and KS-02 read; the timeout gap read in gaps.md. KS-01's scenario: M1, M2, K-tests, S-tests. KS-02's: M2, P3, T1. The gap: T2, M5. Neighbours read as well: KS-20, KS-26, KS-31, KS-37, KS-39, KS-40.
- [x] Every book reference re-located (table); none is gone.
- [x] Goal, non-goals and the later items that depend on this lane are stated.

### Design

- [x] Every decision (D1-D10) gives its rationale and the alternatives it rejects.
- [x] Owner decisions are respected. Exact modifiers and `allow_extra_modifiers`: untouched. Repeat defaults: untouched. The config stays RON and is unchanged. Logging: `eprintln!` until 1.7. Licence: no new dependency.
- [x] Every new code path names its thread (section "Threads of the new code paths"); the LL callback only adds one atomic load.
- [x] Install and uninstall happen on the main thread only; no timer is added.
- [x] No new `SendInput` path. N/A for the sentinel and for pairing repeats with key-ups.
- [x] Sided and unsided modifiers: `reset_from_os` mirrors `sync_modifiers` (RK5). Combo, chord and sequence matching is untouched.
- [x] Reload while off: the engine is untouched, and `on` resets it. `swch off`/`on` during a pending gesture: S1-S6, K1b. Foreground change while off: gated (D3). Shutdown while off: M7.
- [x] Atomic orderings are stated (RK3). No `static mut`, and the static `MAIN_THREAD_ID` copies go from two to one.
- [x] No new module cycle (D4). Engine logic sits in `engine/` and is tested by scenario tests.

### Win32

- [x] Every API has its path, feature check, thread, errors and doc link. The SD and named-pipe calls go through interprocess, at cited file:line.
- [x] Elevation, UIPI and the pipe DACL are covered (D6, RK10-RK12). Token inheritance: N/A.

### Commits and tests

- [x] Red-first: C2 is a behavioural red (S5, S6). The other commits introduce API, so their red is a stated compile error on the parent.
- [x] Tests go through `Engine`, `Env` and the fake clock, or real test pipes. Nothing installs a hook or injects input. D9 guards the replay.
- [x] No test needs real Win32 beyond pipes and `GetAsyncKeyState` reads, so there are no `#[ignore]` tests; the manual checks are M1-M7.
- [x] Every commit passes the gate except the declared red C2.
- [x] The commit order bisects. README and CHANGELOG changes land in the commit that changes the meaning.

### Acceptance

- [x] Every line of the brief and of roadmap A's IPC acceptance maps to a test or a manual check.
- [x] Every question has a recommendation; none blocks.

## Developer addendum: critique response (round 1)

Written by the developer after implementing the cut on `fix/ipc-control` (C1-C7 plus one comment-only commit). Each critique remark ends in one state.

### I1 (Important): manual check M5 suspends the process that owns the LL hook - RESOLVED (manual check replaced)

M5 is replaced. The new M5 suspends only the `ipc-server` thread, so the main thread keeps pumping `GetMessageW` and the LL hook is never timed out:

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M5 | daemon running elevated; Process Explorer started elevated | Process Explorer: `key-switch-rs.exe` > Properties > Threads; select the thread named `ipc-server` and press Suspend. Run `swch status`. Press Resume on the same thread. | After about 5 s: `swch: cannot reach daemon (the daemon did not answer within 5s).`, exit code 1. CapsLock keeps switching the layout while the thread is suspended and after it is resumed. After Resume the daemon console may print `[WARN] IPC write error` or `IPC read error` for the abandoned connection (KS-26/KS-37, 4.1); `swch status` then answers again. | none (T2) |

**Do not suspend the whole process.** The main thread owns `WH_KEYBOARD_LL`; Windows removes a hook whose procedure times out ("on Windows 7 and later, the hook is silently removed without being called", LowLevelKeyboardProc), `swch status` keeps saying `running` (the flag is still true; the watchdog is roadmap 2.4), and remapping stays dead until the daemon is restarted. Why the thread-level suspend works: the client's `CreateFileW` connects to the stored pipe instance in the kernel whatever the server thread does, its request sits in the pipe buffer, and its read waits for a reply that the suspended `ipc-server` never hands to a worker. The acceptance line "roadmap A :96" stays covered by T2 (automated) and this M5.

### I2 (Important): rg check 3 matches a correct tree - RESOLVED

- The comment at `src/bin/swch.rs:95` now reads `// The daemon binary sits next to swch.exe.` (was "... in the install directory.").
- The new module docs of `src/ipc.rs`, `src/hook/ipc_server.rs` and `src/bin/swch.rs` avoid the tokens `install`, `KeyboardHook` and `keyboard_hook` ("keyboard hook" with a space is used, which the pattern does not match).
- Output on HEAD: `rg -n "KeyboardHook|keyboard_hook|install|fn (enable|disable)\b" src/hook/ipc_server.rs src/ipc.rs src/bin/swch.rs` prints nothing, exit code 1.
- Same class of false positive found in **check 4**: `rg -n "impl DaemonControl for DaemonHandle" -A 30 src/core/app.rs` reaches `src/core/app.rs:62`, the pre-existing doc comment of `App::config_path` ("If set, install a notify-based file watcher ..."), which is not a call. The impl block spans `src/core/app.rs:32-58`; use `-A 26` to show exactly the impl. Expected: no `install`/`uninstall` token in it. The first command of check 4, `rg -n "fn set_enabled" -A 20 src/hook/keyboard_hook.rs`, shows `set_enabled` (`:146-155`) and the doc of `install` below it, but no call.

Expected outputs of checks 1 and 2 on HEAD (`2b32441`):

```text
== 1  rg -n "SetWindowsHookExW|SetWinEventHook\(|UnhookWinEvent|UnhookWindowsHookEx" src tests
src\hook\keyboard_hook.rs:8:        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
src\hook\keyboard_hook.rs:170:        // SAFETY: SetWindowsHookExW is the documented installer for WH_KEYBOARD_LL;
src\hook\keyboard_hook.rs:174:            SetWindowsHookExW(
src\hook\keyboard_hook.rs:192:            SetWinEventHook(
src\hook\keyboard_hook.rs:225:    /// `UnhookWinEvent` fails on any other thread.
src\hook\keyboard_hook.rs:234:            let removed = unsafe { UnhookWinEvent(HWINEVENTHOOK(we_handle)) };
src\hook\keyboard_hook.rs:244:            // SAFETY: handle was obtained from SetWindowsHookExW above.
src\hook\keyboard_hook.rs:246:                UnhookWindowsHookEx(HHOOK(handle))?;
== 2  rg -n "fn (install|uninstall)|\.install\(\)|\.uninstall\(\)" src tests
src\hook\keyboard_hook.rs:162:    pub(crate) fn install(&self) -> Result<()> {
src\hook\keyboard_hook.rs:205:            let _ = self.uninstall();
src\hook\keyboard_hook.rs:226:    pub(crate) fn uninstall(&self) -> Result<()> {
src\hook\keyboard_hook.rs:261:        let _ = self.uninstall();
src\core\app.rs:131:        hook.install()?;
src\core\app.rs:189:        hook.uninstall()?;
```

Check 1 also lists the `use` line (`:8`) and three comment lines (`:170`, `:225`, `:244`) besides the calls inside `install`/`uninstall`; all in `keyboard_hook.rs`.

### M1 (Minor): hook-thread affinity enforced only by a comment - REFUTED as a change for this lane

The remark states no current defect ("Consequence: none today") and marks the change optional. The reach it describes is already narrower than the shape suggests: `ipc_server.rs` cannot name `KeyboardHook` (D4; check 3 prints nothing), `DaemonHandle` is private to `src/core/app.rs`, the module whose `run` owns the main thread, and its impl (`src/core/app.rs:32-58`) calls only `set_enabled`, `is_enabled` and `bindings`; `install`/`uninstall` are `pub(crate)` with the main-thread doc (`src/hook/keyboard_hook.rs:157-162`, `:221-226`). A `!Send` marker or a narrower handle type changes the signature of `install`, which D1 fixes as `pub(crate) fn install(&self)`; that is a design change for the architect, best taken with roadmap 2.4 (watchdog and reinstall), the first item that needs to install from somewhere new.

### M2 (Minor): the KS-31 core symptom never goes red - RESOLVED

C1 was written red-first against a no-op `Engine::reset`; S1-S4 failed for the KS-31 reason before the naive reset went in. The failures are quoted in the body of `04b5375`:

```text
remapping_off_drops_a_pending_hold:                scenario: advance 300 ms to +350 ms: expected to fire [], fired [SwitchLanguage]
tap_swallowed_before_..._after_remapping_on:       scenario: up F13 at +1070 ms: expected to fire [SwitchLanguage], fired []
remapping_off_drops_a_pending_chord:               scenario: down K at +10 ms: expected to fire [], fired [SwitchLanguage]
remapping_off_drops_a_pending_sequence:            scenario: down F14 at +0 ms: expected the event to be passed, it was suppressed
```

The harness keeps `self.timers` at `remapping_off` (the Hold timer of S1 fires into the reset engine during `advance_ms`), and `key_event` while off skips the engine (`src/engine/testing.rs:447-454`), so S1 cannot be green with a no-op reset.

### M3 (Minor): dependents and book-sync lists - RESOLVED

- Dependents: add **2.2**. `KeyboardHook::set_enabled` flips the flag and calls `Engine::reset` under `Mutex<Engine>` on the IPC worker, and `decide_event`/`foreground_changed` re-check the flag under the lock. When 2.2 removes the lock, the flip and the reset must move to the thread that owns the engine (for example a `WM_APP` posted to the main thread), and the re-check must stay on that thread. Written into the doc of `set_enabled` (`src/hook/keyboard_hook.rs:142-145`).
- Book sync: add `book/src/roadmap.md:297` (item 2.3 says the reset "resyncs all 256 keys, except blocked ones" on `swch off`/`on`). After this lane the reset at `off`/`on` keeps no blocked record (D2, pinned by S2 and S6); 2.3's "except blocked ones" applies to its own reload/unlock variant only.

### M4 (Minor): IU widens control to every interactive user - RESOLVED

- RK10 corrected: "it cannot control the daemon, and it already runs as the user" holds only for the user's own processes. Under fast user switching or RDP a second interactive user is in IU too, and can send `off`/`on`/`exit` to the first user's (elevated) daemon and create a rogue pipe instance that answers the first user's `swch`. Before this lane that user had read access only (`(A;;FR;;;WD)`). Until 4.1 (per-session name, server-PID check).
- CHANGELOG (KS-02 entry) and README (`swch` section, after "hint and exit 1.") now say so: another signed-in user can switch remapping on or off, stop the daemon, or answer `swch` in its place, until roadmap item 4.1 gives each session its own pipe.

### Open questions of the critique

1. **Label read-back:** not automated. P1 proves the SDDL parses, P3 that `CreateNamedPipeW` accepts the descriptor with its `ME` label without a privilege and that the DACL admits the test token. Reading the label back needs `GetSecurityInfo` with `LABEL_SECURITY_INFORMATION`, i.e. `Win32_Security*` features the lane does not add. **Manual check M2 is the only proof that the label reaches the pipe**, said explicitly.
2. **Symmetry bookkeeping at the harness steps:** defined in code. `remapping_off` clears `press_suppressed` for every key (`src/engine/testing.rs:301-339`): a key swallowed before `off` and released while off is asymmetric on purpose. `remapping_on` keeps the records: a key pressed while off was recorded as passed (the off path calls `check_press` with `suppressed = false`), and the reset seeds it as passed, so its repeats and key-up must stay symmetric. Both steps fail the script if the switch is already in that position.
3. **SD parsed per instance:** corrected in the code docs. `create_listener` parses once (`src/hook/ipc_server.rs:91-100`); interprocess clones it once into the listener's options (`PipeListenerOptions::to_owned`, `try_clone`) and later instances borrow that clone.

### Deviations from the cut

- C1: the "Threads and locking" note of `src/engine/mod.rs` landed in C4 (when the IPC worker starts taking the lock), not C1/C3.
- C4: the adapter's numbered steps in `keyboard_hook_callback.rs` were renumbered (new step 2) instead of a "1a".
- One extra comment-only commit (`2b32441`) rewraps the `with_engine` doc line that C4 lengthened.
- T5 observed outcome: `TimedOut(500ms)` after 502 ms (the outer `recv_timeout` fires just before `WaitNamedPipeW` returns `ERROR_SEM_TIMEOUT`); the cut accepts `Busy | TimedOut(_)`.
