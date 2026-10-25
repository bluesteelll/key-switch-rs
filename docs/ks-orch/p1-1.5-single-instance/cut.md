# Cut: p1-1.5-single-instance — roadmap item 1.5 (single instance), KS-20, review gap A5-m1

Tree: `master` @ `f0048a3` (merge of `fix/language-hotkey-tristate`; `.git/logs/refs/heads/master:23`). Branch to cut: `feat/single-instance`. Every `path:line` below is at `f0048a3`. Nothing was built or run for this cut (read, grep and the `windows`/`interprocess` sources in the cargo registry only).

## Verdict

CUT READY

Four non-blocking questions at the end (Q1-Q4), each with a recommendation. Q1 (a second signed-in user's daemon now stops, KS-40) is the one the orchestrator should read before the critique; the cut implements the spec as written.

## Spec re-located

| Spec reference (book) | Current file:line | Status | Note |
|---|---|---|---|
| KS-20 / roadmap 1.5 "`App::run` installs the hook before it starts IPC", `app.rs:91` @7cb3521 | `src/core/app.rs:131` (`hook.install()?`), pipe at `:139` | moved | **Defect present**: hooks first, pipe second |
| KS-20 "the result of `ipc_server::spawn` only selects which line to print", `app.rs:94-98` | `src/core/app.rs:139-153` (warning `:142-146`, `[INFO] another daemon instance may already be running` `:147-150`) | moved | **Present**: a pipe failure warns and the daemon keeps its hooks |
| KS-20 "`create_sync` fails and `spawn` only warns", `ipc_server.rs:47-60` | `src/hook/ipc_server.rs:94-100` (`create_listener`, `create_sync`), `:102-127` (`spawn`, returns `io::Result<()>`, prints nothing; doc `:102-105` "the caller decides whether to go on without IPC") | moved | The warning text moved to `app.rs` in A5 |
| KS-20 "There is no named mutex" | Grep `CreateMutex\|OpenMutex\|Local\\\\` in `src`: no match | unchanged | **Present** |
| KS-20 "`swch open` ... `connect().is_ok()`", `swch.rs:75` | `src/bin/swch.rs:71-94` (`status` probe, `daemon_running()`); `src/ipc.rs:60-71` (`daemon_running`), `:138-155` (`classify_connect_error`: `ERROR_ACCESS_DENIED` → `AccessDenied`, `ERROR_PIPE_BUSY`/`ERROR_SEM_TIMEOUT`/`TimedOut` → `Busy`) | gone (fixed in `a1b5a27`, fix commit `bf669f3`) | Spec: stays. This lane changes neither file |
| KS-20 sentinel cross-talk, `constants.rs:15-28` | `src/core/constants.rs:15` (`injected_sentinel`), `:30` (`is_injected_event`) | moved | Unchanged; one daemon per session removes the cross-talk |
| roadmap 1.5 "before it loads the config" | `src/main.rs:28` (`ConfigPath::from_arg`), `:30` (`config::load`, which writes the template when `<exe_dir>\config.ron` is missing: `src/config/loader.rs:158-173`, `write_default` `:427-449`) | — | The guard must precede `:28`, so a second start writes no file |
| roadmap 1.5 "exits with code 2" | `src/main.rs:6-20` (`main`: `SUCCESS` or `[FATAL]` + "Press Enter" + `FAILURE`) | — | No code 2 today |
| `main.rs` has no test harness | `Cargo.toml:30-34` (`[[bin]] key-switch-rs`, `test = false`, KS-38) | — | Every testable item goes into the library |
| roadmap 1.5 "failure to create the pipe is fatal" | `src/core/app.rs:139-153` | — | Today a warning |
| install rollback on a failed WinEvent hook | `src/hook/keyboard_hook.rs:202-210` (inside `install`, `:162-214`); `uninstall` `:226-250` | — | Reused unchanged |
| A5-m1 "the callback's fast gate is not pinned", `keyboard_hook_callback.rs:245-251` @A5 head (`D:/tmp/ks-orch/p0-A5-ipc/review_r1.md:30-41`) | gate `src/hook/keyboard_hook_callback.rs:327-333` (block `:328-333`); re-check under the lock `:187-194`; K3 `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` `:485-547`, its overclaiming comment `:487-493`, its `call` closure `:499-518` | moved | **Gap present**: deleting `:328-333` keeps K3 and `disabled_hook_passes_every_event_without_asking_the_engine` (`:463-483`) green |
| foreground callback gate (same shape) | `src/hook/keyboard_hook_callback.rs:369-372` | — | Pinned by the same new test (D5) |
| `windows` features | `Cargo.toml:41-51`; no `Win32_Security` | — | `CreateMutexW` is `#[cfg(feature = "Win32_Security")]`: `windows-0.62.2/src/Windows/Win32/System/Threading/mod.rs:299-308` |
| `ipc_server::spawn` callers | `src/core/app.rs:139`, `tests/ipc_roundtrip.rs:86` (doc `:2`) | — | Both move to `bind` + `serve` (D4) |
| book @1dfd80a: `architecture.md:99`, `:161`, `:549`; `build-and-run.md:125`, `:242`, `:309`, `:408`, `:414`, `:435-449`; `overview.md:26`, `:100`, `:212`; `glossary.md:221`; `gaps.md:225`; `audit.md:43`, `:499`; `roadmap.md:220`, `:271-279`, `:624` | same lines | stale after this lane | Phase book sync (Q3) |

KS issues read: KS-20 (`audit.md:495-508`), KS-02 (pipe SDDL, for the integrity reasoning), KS-38 (`Cargo.toml:18-34`), KS-40 (`audit.md:709-715`, pipe names are machine-wide), KS-09 (handle and token inheritance). A5 review `D:/tmp/ks-orch/p0-A5-ipc/review_r1.md` (m1, R10 line 80).

## Goal and non-goals

**After the lane.**
- The first statement of the daemon's `main` creates the mutex `Local\key-switch-rs`. If another daemon of this Windows session holds it, the process prints ``key-switch-rs: already running in this session; use `swch status`, `swch off` or `swch exit` to control it`` to stderr and exits with code 2. It reads no config, writes no file, installs no hook, creates no pipe and shows no "Press Enter" prompt.
- `App::run` creates the pipe before it installs either hook. A pipe it cannot create ends the daemon with `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: <error>` (plus an owner hint for `ERROR_ACCESS_DENIED`), and no hook is ever installed. If the `ipc-server` thread cannot be started after the hooks are installed, the hooks are removed on the main thread before the same kind of fatal exit.
- The mutex name is a parameter: `InstanceGuard::acquire(name)`. Tests use unique names only.
- The hook callbacks' remapping-off gates (`keyboard_hook_callback.rs:328-333`, `:370-372`) are pinned by a test that fails if either is removed (A5-m1).
- `swch` is untouched; its ACCESS_DENIED/PIPE_BUSY handling stays.

**Non-goals.**
- A per-session pipe name (roadmap 4.1, KS-40). See Q1 for the consequence.
- `swch open` learning the elevated child's exit code (follow-up F2).
- Messages without a console (4.2), autostart (4.5), log file (1.7).
- Protection against a process that squats the name (risk 8).
- No engine, config, RON, template or `swch` change.

**Later items that depend on this.** 4.5 `swch autostart` (`roadmap.md:483`: depends on 1.5 and 4.2).

## Decisions

### D1: The guard is a named mutex, held by existence, in `src/system/single_instance.rs`

- **What.**
  ```rust
  /// The daemon's instance mutex, one per Windows session (roadmap 1.5, KS-20).
  pub const INSTANCE_MUTEX_NAME: &str = r"Local\key-switch-rs";

  #[derive(Debug)]
  #[must_use = "the instance is released when the guard is dropped"]
  pub struct InstanceGuard { handle: HANDLE }

  impl InstanceGuard {
      /// Creates the named mutex `name`, or reports that another holder exists.
      pub fn acquire(name: &str) -> Result<InstanceGuard, InstanceError>;
  }
  impl Drop for InstanceGuard { /* CloseHandle, result ignored */ }
  ```
  - `acquire`: build the NUL-terminated UTF-16 name (one `Vec<u16>`, startup only), call `SetLastError(ERROR_SUCCESS)`, then `CreateMutexW(None, false, PCWSTR(wide.as_ptr()))`, then on `Ok(handle)` read `GetLastError()` with no call in between. Classification is D2.
  - The mutex is never waited on and never owned (`bInitialOwner = false`). Only the object's existence matters: it lives while any handle is open, and the kernel closes our handle when the process ends, a crash included.
  - `lpMutexAttributes = None`: default security descriptor, and the handle is not inheritable. So a `Launch` child (KS-09) never keeps the name alive after the daemon exits.
  - `system/mod.rs` gains `pub mod single_instance;`. The module imports only `std` and `windows` (tests also `interprocess`), so no new module cycle.
- **Why.** The spec names `CreateMutexW` and `Local\key-switch-rs`. `Local\` is the session namespace, the right scope because a low-level hook sees only its own session's input. Existence-based detection has no thread affinity and no abandoned-mutex states. The doc recommends `bInitialOwner = FALSE` when several processes call `CreateMutex` with the same name. `system/` holds the other OS-resource wrappers (`registry.rs`). The name is a parameter so tests never touch the real one.
- **Rejected.**
  - `Global\`: it would stop a second signed-in user's daemon by design, and the doc keeps that for cross-session singletons.
  - A locked file in the profile, or a private namespace (`CreatePrivateNamespace`): these are the doc's anti-squatting options. Both are more code, and squatting needs code already running in the user's session (risk 8).
  - `OpenMutexW` then `CreateMutexW`: a check-then-act race.
  - `bInitialOwner = true`: ownership is per thread and brings abandonment semantics for nothing.
  - `windows::core::Owned<HANDLE>` (`HANDLE: Free`, `Foundation/mod.rs:5297`): it would work, but a hand-written `Drop` matches the house guard (`HKeyGuard`, `registry.rs:132-144`).
- **Cost.** A new Cargo feature (D7). `InstanceGuard` is `!Send`, because `HANDLE` wraps a raw pointer. It never leaves the main thread.

### D2: Classification: `ERROR_ALREADY_EXISTS` and `ERROR_ACCESS_DENIED` mean "already running"; anything else is fatal

- **What.**
  ```rust
  #[derive(Debug)]
  pub enum InstanceError {
      /// Another holder exists: a daemon of this session, possibly at a higher
      /// integrity level that this process may not open.
      AlreadyRunning,
      /// `CreateMutexW` failed for another reason.
      Os(windows::core::Error),
  }
  // Display: AlreadyRunning => "already running in this session; use `swch status`, `swch off` or `swch exit` to control it"
  //          Os(e)          => "could not create the single-instance mutex: {e}"
  // std::error::Error::source: Os(e) => Some(e), AlreadyRunning => None.

  /// Sorts a failed `CreateMutexW` (pure, unit-tested).
  fn classify_open_error(error: windows::core::Error) -> InstanceError;
  ```
  | `CreateMutexW` outcome | Result |
  |---|---|
  | `Ok(handle)`, `GetLastError() != ERROR_ALREADY_EXISTS` | `Ok(InstanceGuard { handle })` |
  | `Ok(handle)`, `GetLastError() == ERROR_ALREADY_EXISTS` | close `handle` first (wrap it in a guard and drop it), then `Err(AlreadyRunning)` |
  | `Err(e)`, `e.code() == ERROR_ACCESS_DENIED.to_hresult()` | `Err(AlreadyRunning)` |
  | `Err(e)`, any other code (for example `ERROR_INVALID_HANDLE`: an event or another object type under that name) | `Err(Os(e))` |
- **Why.**
  - On an existing mutex `CreateMutexW` "requests the MUTEX_ALL_ACCESS access right". The release daemon runs elevated (`requireAdministrator`), so its mutex gets the elevated token's default DACL and a high label. A debug daemon at medium integrity then cannot open it, and the expected error is `ERROR_ACCESS_DENIED` (unverified for the label path; T4 pins the DACL path on the real kernel, M3 checks the label path by hand). The object exists, so the honest message is "already running", not a `[FATAL] Access is denied`.
  - The handle must be closed on the `ALREADY_EXISTS` path. Otherwise the refused process would keep the object alive, and a third start after the first daemon exits would still see "already running". T2 pins this.
  - `SetLastError(ERROR_SUCCESS)` first: the doc does not say that a newly created mutex resets the last error, and GetLastError's doc says "some functions set the last-error code to 0 on success and others do not". T7 pins it.
  - Between the FFI return and our `GetLastError` the `windows` wrapper only tests `is_invalid()` (`Threading/mod.rs:306-307`), so nothing overwrites the code.
- **Rejected.**
  - `ERROR_ACCESS_DENIED` as `Os` (fatal): a debug build next to the release daemon would stop with a misleading `[FATAL] Access is denied`.
  - A third variant for it: the user does the same thing in both cases, and the spec names one `Err(AlreadyRunning)`.
  - An explicit SDDL on the mutex so that medium can open it: more code, and it still cannot tell a daemon from a squatter.
- **Cost.** A squatter whose mutex denies us access reads as "already running" (risk 8).

### D3: `main` takes the guard first, passes it into `run`, and exits with 2 without a prompt

- **What** (`src/main.rs`).
  ```rust
  /// Exit code of a start that found another daemon in this session (roadmap
  /// item 1.5); a failure exits with 1.
  const EXIT_ALREADY_RUNNING: u8 = 2;

  fn main() -> ExitCode;
  /// Prints `[FATAL] {error}`, waits for Enter, returns `ExitCode::FAILURE`
  /// (today's `main.rs:9-18`, moved verbatim).
  fn fatal(error: &dyn std::error::Error) -> ExitCode;
  /// Today's `run` (`main.rs:22-38`) with one parameter: the guard lives
  /// until `run` returns.
  fn run(_instance: InstanceGuard) -> Result<(), Box<dyn std::error::Error>>;
  ```
  - `main`, in order:
    1. `InstanceGuard::acquire(INSTANCE_MUTEX_NAME)`, as the first statement.
    2. `Err(InstanceError::AlreadyRunning)`: `eprintln!("key-switch-rs: {}", InstanceError::AlreadyRunning)`, then `return ExitCode::from(EXIT_ALREADY_RUNNING)`.
    3. `Err(other)`: `return fatal(&other)`.
    4. `Ok(instance)`: `match run(instance)`, giving `Ok` → `SUCCESS` and `Err(e)` → `fatal(&*e)`.
  - The parameter is the binding `_instance`, not the pattern `_`, so it is dropped when `run` returns. Parameters drop after the body's locals, so the guard outlives `App::run`, which has already removed the hooks. The guard is also released **before** the FATAL prompt waits for Enter.
  - A comment at the call says why it comes first: no config read or template write, no hook and no pipe before the instance is known to be the only one (KS-20).
- **Why.**
  - It meets the spec order: guard (`main`) → `from_arg` (`main.rs:28`) → `load` (`:30`) → `App::run` (pipe, hooks).
  - Releasing before the prompt matters. A config error leaves a FATAL console waiting for Enter, and a second double-click after the user fixes the config must start, not report "already running".
  - No prompt on code 2: a script, the 4.5 task (`MultipleInstancesPolicy=IgnoreNew`) or a double-click must not leave a second console parked on stdin. The first daemon's console stays visible (Q2).
  - Typed control flow with no `downcast_ref` on `Box<dyn Error>`.
- **Rejected.**
  - Acquiring inside `run` and downcasting the boxed error: the exit-code path would rest on a runtime type test.
  - Acquiring in `App::run`: that is after the config load, against the spec.
  - Exit code 1: scripts could not tell "already running" from a failure.
- **Cost.** A double-click while a daemon runs shows a console that closes at once (M2).

### D4: The pipe is created before the hooks; failure is fatal (`bind` / `serve` split)

- **What.**
  - `src/hook/ipc_server.rs`:
    ```rust
    /// The daemon's pipe, created but not yet served.
    #[must_use = "the pipe is closed when an unserved listener is dropped"]
    pub struct IpcListener { listener: Listener }

    /// Creates the pipe `pipe_name` (`\\.\pipe\<pipe_name>`) with PIPE_SDDL.
    /// Starts no thread.
    pub fn bind(pipe_name: &str) -> io::Result<IpcListener>;

    impl IpcListener {
        /// Spawns the `ipc-server` thread that serves the pipe with `control`
        /// for the lifetime of the process. On error the listener, and with it
        /// the pipe, is dropped.
        pub fn serve(self, control: Arc<dyn DaemonControl>) -> io::Result<()>;
    }
    ```
    `bind` wraps `create_listener` (`:94-100`). `serve` holds today's thread body (`:108-125`) unchanged. `spawn` (`:102-127`) is removed, and the module doc (`:1-3`) says "created by `App::run` before the hooks (`bind`), served once they are installed (`serve`)".
  - `src/core/app.rs` `App::run` becomes `pub fn run(mut self) -> std::result::Result<(), Box<dyn std::error::Error>>` (inside, `?` converts the `windows::core::Error`s). New order:
    1. `MAIN_THREAD_ID` store (`:122-123`, unchanged).
    2. `ipc_server::bind(ipc::PIPE_NAME)`. On `Err(e)`: `return Err(pipe_error_message(&ipc::pipe_path(ipc::PIPE_NAME), &e).into())`. Nothing to undo.
    3. `print_welcome`, `setup_ctrl_c_handler()?` (`:125-126`).
    4. `update_bindings`, `hook.install()?` (`:128-132`). On `Err`, the listener local is dropped on return, which closes the pipe. A failed WinEvent hook already removes the keyboard hook (`keyboard_hook.rs:202-210`).
    5. `DaemonHandle` (`:135-138`), then `listener.serve(control)`. On `Err(e)`, call `hook.uninstall()` (on failure `eprintln!("[WARN] could not remove the keyboard hook: {error}")`), then `return Err(format!("could not start the IPC server thread: {e}").into())`. On `Ok`, print `✓ IPC server listening (use `swch on/off/exit`)` (the existing line).
    6. Watcher, message loop and `uninstall` (`:155-189`), unchanged.

    The `[WARN] could not create IPC listener` and `[INFO] another daemon instance may already be running` lines (`:141-151`) go.
  - `fn pipe_error_message(pipe_path: &str, error: &io::Error) -> String`, private, in `app.rs`:
    - Always: `could not create the IPC pipe {pipe_path}: {error}`.
    - When `error.raw_os_error()` maps (`u32::try_from` → `WIN32_ERROR`, as `ipc.rs:140-143` does) to `ERROR_ACCESS_DENIED`, append: `. Another program owns that pipe name: the daemon of another signed-in user (one pipe serves every session until roadmap item 4.1) or a key-switch-rs started before the single-instance guard`.
- **Why.**
  - The roadmap names the root cause: "The hook is installed before the pipe exists" (`roadmap.md:275`). R10 flags "a startup order that leaves a live hook without IPC".
  - With `bind` first, the realistic failures leave nothing to undo, and no hook is ever live without a control pipe: a foreign owner of the pipe, a pre-1.5 daemon without the mutex, another session's daemon (KS-40), a bad SDDL.
  - `FILE_FLAG_FIRST_PIPE_INSTANCE` makes a taken name fail with `ERROR_ACCESS_DENIED`. interprocess sets the flag on the first instance (`interprocess-2.4.2/src/os/windows/named_pipe/listener/create_instance.rs:85-86`), and I1 pins the outcome.
  - `serve` comes after `install`. If the server thread ran first, a failed `install` would leave it answering `swch status` with `running` while `main` waits at the FATAL prompt.
  - Only the near-impossible thread-spawn failure needs `uninstall`, which runs on the installing thread (KS-01).
  - `Box<dyn Error>`: `windows::core::Error::new` keeps a message only through `RoOriginateErrorW` (`windows-result-0.4.1/src/error.rs:91-101`, `:292-297`), and `From<io::Error>` keeps only the code (`:176-183`), so a pipe error would lose its context. `main::run` already returns `Box<dyn Error>` (`main.rs:22`).
- **Rejected.**
  - Keep the order and uninstall on a pipe failure (option b): a hook is live without a pipe for a moment, and the common failure path has to undo Win32 state.
  - `spawn` before `install` (option a): the server thread outlives a failed install, as said above.
  - Keep warn-and-continue: against the spec.
  - A trait seam that injects bind/install/serve/uninstall to unit-test the order: four calls in one function, and R10 plus M4 cover it more cheaply.
  - Keeping `spawn` as a convenience: it would have no production caller. Its two callers are updated instead.
- **Cost.** A second signed-in user's daemon now stops instead of running without IPC (KS-40, Q1). `App::run`'s public error type changes; its only caller is `main.rs:36`, which compiles unchanged.

### D5: A5-m1: pin both remapping-off gates by holding the engine lock on another thread

- **What** (`src/hook/keyboard_hook_callback.rs` tests only).
  - Test-only `static GLOBAL_HOOK: Mutex<()> = Mutex::new(());`. Every test that drives `get_hook()` takes it first with `.lock().unwrap_or_else(PoisonError::into_inner)`: K3 and the new test.
  - Helper `fn call_keyboard_callback(vk: u32, msg: u32) -> LRESULT`. It is K3's closure `:499-518` moved out, with its SAFETY comment.
  - Drop guard `struct RemappingOnAtEnd;`, whose `Drop` calls `let _ = get_hook().set_enabled(true);`. Declare it after the `GLOBAL_HOOK` guard, so that remapping is back on before the lock is released, also on panic.
  - New test `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`:

    | Step | Action | Assertion |
    |---|---|---|
    | 1 | lock `GLOBAL_HOOK`; `RemappingOnAtEnd`; `get_hook().update_bindings([F13 → DoNothing, allow_extra_modifiers])`; `get_hook().set_enabled(false).expect("not inside with_engine")` | — |
    | 2 | spawn `holder`: `get_hook().with_engine(\|_\| { locked_tx.send(()); release_rx.recv_timeout(5 s) })` | — |
    | 3 | `locked_rx.recv_timeout(5 s)` | `.expect("the holder thread took the engine lock")` |
    | 4 | `t0`; `down = call_keyboard_callback(F13, WM_KEYDOWN)`; `up = call_keyboard_callback(F13, WM_KEYUP)`; `kb = t0.elapsed()` | — |
    | 5 | `t1`; `unsafe { foreground_changed(HWINEVENTHOOK::default(), EVENT_SYSTEM_FOREGROUND, HWND::default(), 0, 0, 0, 0) }`; `fg = t1.elapsed()` | — |
    | 6 | `release_tx.send(())`; `holder.join()` | `.expect("the holder thread does not panic")` |
    | 7 | — | `assert!(kb < 1 s, "with remapping off the keyboard callback waited {kb:?} for the engine lock")`; `assert!(fg < 1 s, "with remapping off the foreground callback waited {fg:?} for the engine lock")`; `assert_ne!(down, LRESULT(1))`; `assert_ne!(up, LRESULT(1))` |

    SAFETY for step 5: `foreground_changed` reads none of its arguments (`:360-381`). The null handles are never dereferenced or passed on. No hook is installed and no message is pumped.
  - K3 takes the lock, uses the helper, and its comment `:487-493` is narrowed. It remains the only test of `LRESULT(1)` iff `Decision::suppress` at the extern level. Its remapping-off asserts hold with or without the gate before the lock, because the re-check under the lock (`:191-194`) passes the event too (A5-m1). `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` pins that gate.
- **Why.**
  - "Without touching the engine" can be observed through the lock: with the gate the callbacks never call `with_engine`.
  - Without the keyboard gate, `decide_event` blocks in `with_engine` (`:191`) until the holder's 5 s timeout, so `kb` comes out at about 5 s and the red is deterministic. The foreground gate fails the same way, independently.
  - Dropping `release_tx` on a panic ends the holder at once, so the suite never hangs.
  - Serialising the two global-hook tests stops the new test's `set_enabled(false)` from racing K3's swallow asserts. No other test touches `get_hook()` (Grep: only `:495-540`). `start_hold_timer` reaches it only for Hold decisions, and no test here produces one.
- **Rejected.**
  - A `#[cfg(test)]` counter inside `with_engine`: instrumentation in the lock function, and this lane changes no product code there.
  - Folding everything into K3: one long test whose name hides the pin.
  - Making the callback body take `&KeyboardHook`: it changes hot-path code for a test.
- **Cost.** A test-only lock and a 5 s worst case, only on a red.

### D6: No engine, config or `swch` change

- **What.** No `src/engine` change, no RON or schema change, no `default_config.ron` edit. `src/bin/swch.rs` and `src/ipc.rs` stay byte-identical.
- **Why.** No per-key decision changes. The spec keeps A5's `swch open` handling as it is, and its tests stay green unchanged: `ipc::tests::only_a_missing_pipe_means_no_daemon`, `connect_errors_are_classified`, and in `tests/ipc_roundtrip.rs` `request_reports_access_denied_for_a_pipe_this_process_may_not_open` and `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running`.

### D7: Cargo feature `Win32_Security` on the existing `windows` dependency

- **What.** `Cargo.toml` adds `"Win32_Security",` to the `windows` features after `"Win32_System_Threading",` (`:50`). No new crate. `Cargo.lock` is unchanged, because features are not recorded in it, so `--locked` holds.
- **Why.** `CreateMutexW` takes `SECURITY_ATTRIBUTES` and is gated on that feature (`Threading/mod.rs:299-301`), as is every other named-object creator.
- **Cost.** A larger `windows` compile. The feature also makes `Win32_Security`-gated items visible through the existing globs (`app.rs:4` `Threading::*`, `registry.rs:2` `Registry::*`). A same-name collision would show up only if such a name were used, and `cargo check` catches it (risk 12).

## Win32 API choices

| Purpose | `windows` path / signature | Feature (present?) | Thread | Errors | Doc link and constraint relied on |
|---|---|---|---|---|---|
| Create or open the instance mutex | `windows::Win32::System::Threading::CreateMutexW<P2: Param<PCWSTR>>(lpmutexattributes: Option<*const SECURITY_ATTRIBUTES>, binitialowner: bool, lpname: P2) -> windows::core::Result<HANDLE>` (`windows-0.62.2/src/Windows/Win32/System/Threading/mod.rs:301-308`) | `Win32_System_Threading` yes (`Cargo.toml:50`); **`Win32_Security` no: add (D7)** | main, first statement of `main` | D2 table | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw: "If the mutex is a named mutex and the object existed before this function call, the return value is a handle to the existing object, and the GetLastError function returns ERROR_ALREADY_EXISTS."; "If lpName matches the name of an existing named mutex object, this function requests the MUTEX_ALL_ACCESS access right."; "If lpName matches the name of an existing event, semaphore, waitable timer, job, or file-mapping object, the function fails and the GetLastError function returns ERROR_INVALID_HANDLE."; "If this parameter is NULL, the handle cannot be inherited by child processes."; "you should set the bInitialOwner flag to FALSE"; "Name comparison is case sensitive." |
| Clear the thread's last error first | `windows::Win32::Foundation::SetLastError(dwerrcode: WIN32_ERROR)`, `unsafe fn` (`Foundation/mod.rs:53`); `ERROR_SUCCESS` (`:4092`) | `Win32_Foundation` yes | main | none | https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror ("The last-error code is kept in thread local storage") |
| Read `ERROR_ALREADY_EXISTS` | `windows::Win32::Foundation::GetLastError() -> WIN32_ERROR`, `unsafe fn` (`Foundation/mod.rs:27`); `ERROR_ALREADY_EXISTS` (`:1171`), `ERROR_ACCESS_DENIED` (`:1143`), `WIN32_ERROR::to_hresult` (`src/extensions/Win32/Foundation/WIN32_ERROR.rs:13`) | yes | main, immediately after `CreateMutexW` | — | https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror: "some functions set the last-error code to 0 on success and others do not"; "You should call the GetLastError function immediately" |
| Close the handle | `windows::Win32::Foundation::CloseHandle(hobject: HANDLE) -> windows::core::Result<()>`, `unsafe fn` (`Foundation/mod.rs:2`) | yes | the dropping thread (main) | ignored in `Drop` | https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle: "After the last handle to an object is closed, the object is removed from the system." CreateMutexW page: "The system closes the handle automatically when the process terminates." |
| Session scope of `Local\` | (name prefix) | — | — | — | https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces: "each session has a separate namespace for these objects"; "client processes can use the "Local\" prefix to explicitly create an object in their session namespace. These keywords are case sensitive." That an elevated and a non-elevated process of one session share the namespace is not stated there (unverified; M3 checks it) |
| Test-only squatter with a restrictive DACL | `windows::Win32::Security::SECURITY_ATTRIBUTES { nLength: u32, lpSecurityDescriptor: *mut c_void, bInheritHandle: BOOL }` (`Security/mod.rs:1449-1453`); SD from `interprocess::os::windows::security_descriptor::SecurityDescriptor::deserialize` plus `AsSecurityDescriptor::as_sd()` (`interprocess-2.4.2/src/os/windows/security_descriptor/owned.rs:30-33`, re-exported `security_descriptor.rs:25`) | `Win32_Security` (added) | test thread | — | https://learn.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights: "The ACLs in the default security descriptor for a synchronization object come from the primary or impersonation token of the creator." |
| Pipe exists → creation fails | interprocess `ListenerOptions::create_sync` → `CreateNamedPipeW` with `FILE_FLAG_FIRST_PIPE_INSTANCE` on the first instance (`create_instance.rs:85-86`); reached through `create_listener` (`ipc_server.rs:94-100`) | n/a (interprocess's `windows-sys`) | main (`bind`) | `io::Error`, `raw_os_error() == Some(5)` | https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-createnamedpipew: "If you attempt to create multiple instances of a pipe with this flag, creation of the first instance succeeds, but creation of the next instance fails with ERROR_ACCESS_DENIED." |

**Threads.**

| Code path | Thread |
|---|---|
| `InstanceGuard::acquire` / `Drop` | main: the first statement of `main`; the drop at the end of `run` |
| `ipc_server::bind` | main, in `App::run` before `install` |
| `IpcListener::serve` | main spawns `ipc-server`; accept loop and per-connection workers as today (`ipc_server.rs:108-125`) |
| `install`, and `uninstall` on a serve failure | main, the installing thread (KS-01) |
| D5 test | the cargo test thread calls both extern callbacks; a `holder` thread holds the engine lock |

No window, timer or COM apartment is added. No `SendInput`. `LLKHF_INJECTED`, the extended-key flag and `scanCode` play no part.

## Touch set

| File | Change | Commit | Why |
|---|---|---|---|
| `src/hook/keyboard_hook_callback.rs` | tests only: `GLOBAL_HOOK`, `call_keyboard_callback`, `RemappingOnAtEnd`, the new test; K3 takes the lock and uses the helper; K3 comment narrowed | C1 | A5-m1 (D5) |
| `Cargo.toml` | `"Win32_Security",` after `:50` | C2 | D7 |
| `src/system/mod.rs` | `pub mod single_instance;` | C2 | D1 |
| `src/system/single_instance.rs` (new) | D1/D2 items and the tests T1-T8 | C2 | roadmap 1.5, injectable name |
| `README.md` | C2: the tree gets `│   ├── single_instance.rs  - one daemon per session: the Local\key-switch-rs mutex` between `:64` and `:65`; the feature list gets ``- `Win32_Security` (`CreateMutexW` for the single-instance guard)`` after `:746`. C3: `:66` becomes `main.rs - entry point: single-instance guard, loads config, builds App`; a paragraph after `:726` (text in C3). C5: an `app.rs` bullet after `:133`; a sentence after `:720` (text in C5) | C2, C3, C5 | Docs follow features and behaviour in the commit that changes them |
| `src/main.rs` | D3 | C3 | spec: guard before config, exit code 2 |
| `CHANGELOG.md` | C3: a `### Fixed` KS-20 bullet before `- KS-21:` (`:437`), and the parenthetical at `:273` edited. C5: two sentences appended to that bullet | C3, C5 | Keep a Changelog, `[Unreleased]` |
| `src/hook/ipc_server.rs` | C4: `IpcListener`, `bind`, `serve`; `spawn` removed; module doc `:1-14`; test I1. C5: ignored test M4 | C4, C5 | D4 |
| `tests/ipc_roundtrip.rs` | `:2` doc and `:86` call: `ipc_server::bind(&name).expect("the pipe can be created under its own name").serve(control.clone()).expect("the server thread starts")` | C4 | `spawn` removed |
| `src/core/app.rs` | C4: the call site becomes `bind(..).and_then(\|l\| l.serve(control))` in the same place, with the same warning (no behaviour change). C5: the D4 order, the return type, `pipe_error_message`, its two tests, and `use std::io` | C4, C5 | D4 |
| `src/config/default_config.ron`, `book/src/config/*.md` | none | — | No config meaning changes (D6) |
| Book outside `config/` | none in this lane | — | Phase book sync (Q3): `audit.md:43`, `:499`; `roadmap.md:220`, `:271-279` (status line), `:624`; `architecture.md:93-102` (startup sequence: guard step, pipe before hooks, fatal), `:161` (`ipc-server` row: `IpcListener::serve`), `:549`; `build-and-run.md:125` (console table: new FATAL and already-running lines), `:242`, `:309` (WARNING becomes a note: code 2), `:401-414` (flowchart node H, KS-20 row), `:435-449` (move under "Fixed since the audit"); `overview.md:26`, `:100`, `:212`; `glossary.md:221`; `gaps.md:225` |

Not touched: `src/bin/swch.rs`, `src/ipc.rs`, `src/engine/**`, `src/hook/keyboard_hook.rs`, `src/hook/config_watcher.rs`, `build.rs`, `app.manifest`, `Cargo.lock`.

## Commits, in order

Subjects follow `<Area>: <what> (<item>, <KS>)`. Every message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **Every commit passes the full gate** (`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`). No commit is a declared red. The reds are replays on `master` (C2, C4, C5: compile errors for the new API) and the mutations listed (MU-A to MU-F), which the tester replays. A compile error as the parent's red follows the house precedent (`D:/tmp/ks-orch/p0-A5-ipc/cut.md:773`). Each message body quotes the predicted red.

1. **`Tests: pin the hook callbacks' remapping-off gate before the engine lock (A5-m1)`**: test-only, D5.
   - New: `hook::keyboard_hook_callback::tests::remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`. Changed: K3 (lock, helper, comment).
   - On `master`: green, because this is a pin of existing behaviour. **Predicted red under MU-A** (delete `keyboard_hook_callback.rs:328-333`): `with remapping off the keyboard callback waited 5.0…s for the engine lock`. **Under MU-B** (delete `:370-372`): `with remapping off the foreground callback waited 5.0…s for the engine lock`. K3 and `disabled_hook_passes_every_event_without_asking_the_engine` stay green under both: that is the A5-m1 gap.
   - Invariant: no product code changes.

2. **`System: single-instance guard on a named mutex (1.5, KS-20)`**: D1, D2, D7; README tree and feature line.
   - Tests in `system::single_instance::tests`. Names come from `fn unique_name(tag: &str) -> String`: ``format!(r"Local\key-switch-rs-test-{}-{tag}-{n}", std::process::id())``, where `n` comes from a test `static COUNTER: AtomicU32` (`fetch_add(1, Relaxed)`, with the comment that the counter hands out distinct numbers and publishes no data, as at `tests/ipc_roundtrip.rs:29-31`). No test acquires `INSTANCE_MUTEX_NAME`.

     | # | Test | Asserts |
     |---|---|---|
     | T1 | `a_second_acquisition_of_a_held_name_is_already_running` | `first = acquire(n)` is `Ok`; `matches!(acquire(n), Err(InstanceError::AlreadyRunning))`, message "a second acquisition of a held name must be AlreadyRunning" |
     | T2 | `the_name_is_free_again_once_every_guard_is_dropped` | `first` Ok; second `AlreadyRunning`; `drop(first)`; `acquire(n)` is `Ok` ("the refused acquisition must not keep the mutex alive") |
     | T3 | `distinct_names_do_not_conflict` | `acquire(a)` and `acquire(b)` are both `Ok` while both guards live |
     | T4 | `a_mutex_this_process_may_not_open_counts_as_already_running` | Squatter: `SecurityDescriptor::deserialize(&"D:P(A;;GA;;;SY)".to_wtf_16()…)` (the SDDL from `tests/ipc_roundtrip.rs:162`), `SECURITY_ATTRIBUTES { nLength: size_of::<SECURITY_ATTRIBUTES>() as u32, lpSecurityDescriptor: sd.as_sd().cast_mut(), bInheritHandle: false.into() }`, `unsafe { CreateMutexW(Some(&attributes), false, PCWSTR(wide.as_ptr())) }`, wrapped as `InstanceGuard { handle }`. Then `matches!(acquire(n), Err(InstanceError::AlreadyRunning))`. Predicted path: `ERROR_ACCESS_DENIED` (unverified until the tester runs it; on another code, report it, and do not widen D2) |
     | T5 | `only_access_denied_among_open_errors_means_already_running` | `classify_open_error(ERROR_ACCESS_DENIED.into())` gives `AlreadyRunning`; `classify_open_error(ERROR_INVALID_HANDLE.into())` gives `Os(e)` with `e.code() == ERROR_INVALID_HANDLE.to_hresult()`; same for `ERROR_INVALID_NAME` |
     | T6 | `the_daemon_mutex_is_local_key_switch_rs` | `assert_eq!(INSTANCE_MUTEX_NAME, r"Local\key-switch-rs")` |
     | T7 | `a_stale_already_exists_error_does_not_make_a_fresh_name_look_held` | `unsafe { SetLastError(ERROR_ALREADY_EXISTS) }`, then `acquire(fresh)` is `Ok` |
     | T8 | `already_running_names_the_swch_commands` | `InstanceError::AlreadyRunning.to_string()` equals the D2 text exactly; `Os(ERROR_INVALID_HANDLE.into()).to_string()` starts with `could not create the single-instance mutex: ` |
   - Every `unsafe` block carries a concrete `// SAFETY:`. `SetLastError`/`GetLastError` touch only this thread's slot. For `CreateMutexW`: the name buffer is NUL-terminated and alive across the call, and `None` means a default SD and no inheritance. For T4, the attributes and the SD are stack locals alive across the call. For `CloseHandle`: a handle from a successful `CreateMutexW`, owned only by this guard, closed once.
   - **Predicted red on `master`** (tests module replayed): `error[E0412]: cannot find type InstanceGuard in this scope`, and E0425/E0433 for `INSTANCE_MUTEX_NAME`, `InstanceError`, `classify_open_error`. Behavioural reds:
     - **MU-C** (drop the `ERROR_ALREADY_EXISTS` check): T1 `a second acquisition of a held name must be AlreadyRunning`.
     - **MU-D** (`mem::forget` the handle on that path instead of closing it): T2 `the refused acquisition must not keep the mutex alive`.
     - **MU-E** (`ERROR_ACCESS_DENIED` → `Os`): T4 and T5.
     - **MU-F** (remove `SetLastError`): T7 goes red only if Windows leaves the last error alone on creation. The tester reports which, and it is not a defect either way.
   - README: the tree line and the feature line. Not wired into the daemon yet, so no behaviour changes.

3. **`Daemon: a second start in the same session prints "already running" and exits with code 2 (1.5, KS-20)`**: D3; README; CHANGELOG.
   - No automated test is possible: `main.rs` has no harness (`Cargo.toml:30-34`), and running the daemon is forbidden. The tested parts are C2's T1/T8 and the message constant. Behaviour: M1, M1b, M2, M3, M5.
   - README: `:66` (Touch set). After `:726` (Direct launch) add: "Only one daemon runs per Windows session. A second start, by double-click, from a console or from a Scheduled Task, prints ``key-switch-rs: already running in this session; use `swch status`, `swch off` or `swch exit` to control it`` and exits with code 2 before it reads the config. Daemons of other signed-in users do not count."
   - CHANGELOG `### Fixed`, before `- KS-21:` (`:437`):
     ```
     - KS-20 (roadmap item 1.5): a second daemon no longer starts in the same
       Windows session. Before it reads or writes the config and before it
       installs any hook, the daemon creates the mutex `Local\key-switch-rs`;
       when another daemon of the session holds it, the new one prints
       ``key-switch-rs: already running in this session; use `swch status`, `swch off` or `swch exit` to control it``
       and exits with code 2, without the "Press Enter to exit" prompt. A
       double-click on `key-switch-rs.exe` while the Scheduled Task's daemon
       ran used to start a second elevated daemon: its hook ran first, keys
       were remapped twice, a double `SwitchLanguage` left the layout
       unchanged, and `swch off` reached only the other daemon. Daemons of
       other signed-in users do not count.
     ```
     At `:273`, `(KS-20, partly; roadmap item 1.5 adds the single-instance guard)` becomes `(KS-20; the single-instance guard below covers the other ways to start a second daemon in a session)`.

4. **`IPC: create the pipe apart from starting its server (1.5, KS-20)`**: D4's `bind`/`serve` split, no behaviour change.
   - New test `hook::ipc_server::tests::a_second_listener_on_a_taken_pipe_name_is_refused`. Name `format!("key-switch-rs-unit-{}-taken.sock", std::process::id())` (the shape of `:315`). `let _first = bind(&name).expect(..)`; `let Err(error) = bind(&name) else { panic!("a second listener on a taken pipe name was created") }`; `assert_eq!(error.raw_os_error(), Some(5), "FILE_FLAG_FIRST_PIPE_INSTANCE refuses a taken name with ERROR_ACCESS_DENIED: {error}")`. let-else instead of `expect_err`, so `IpcListener` needs no `Debug`.
   - `tests/ipc_roundtrip.rs::status_off_on_status_round_trip_without_a_hook` runs through `bind` + `serve`, with the same asserts.
   - **Predicted red on `master`**: `error[E0425]: cannot find function bind in this scope` (and in `tests/ipc_roundtrip.rs`: `cannot find function bind in module ipc_server`).
   - Invariant: `app.rs` still warns and goes on, so behaviour is unchanged and the branch bisects.

5. **`App: create the pipe before installing the hooks; a pipe that cannot be created is fatal (1.5, KS-20)`**: the D4 order; README; CHANGELOG.
   - New tests in `core::app::tests`:
     - `a_taken_pipe_name_is_reported_with_its_likely_owners`: `pipe_error_message(r"\\.\pipe\key-switch-rs.sock", &io::Error::from_raw_os_error(5))` starts with `could not create the IPC pipe \\.\pipe\key-switch-rs.sock: ` and contains `another signed-in user` and `before the single-instance guard`.
     - `other_pipe_errors_are_reported_as_they_are`: for `io::Error::new(io::ErrorKind::InvalidInput, "bad descriptor")` the message equals `format!("could not create the IPC pipe {path}: {error}")`.
   - New ignored test `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`, `#[ignore = "manual: holds the daemon's pipe for 60 s; meanwhile start the release daemon and check that it stops with [FATAL] could not create the IPC pipe and that CapsLock is not remapped"]`. It runs `bind(crate::ipc::PIPE_NAME)` (on `Err`: panic "a daemon owns the pipe; stop it first"), `eprintln!` instructions, then `thread::sleep(60 s)`.
   - **Predicted red on `master`**: `error[E0425]: cannot find function pipe_error_message in this scope`. The order itself has no automated red (rejected seam, D4). Its proofs are M4 and review against R10. **MU-G** (move `bind` after `install`) is caught by M4 only, which is declared.
   - README: after `:133` (`app.rs` section) add the bullet "Creates the IPC pipe before it installs the hooks; a pipe it cannot create stops the daemon (KS-20)". After `:720` add: "Until then only one daemon on the machine can own the pipe: a daemon that cannot create it, for example a second signed-in user's, stops with `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: ...` before it installs any hook."
   - CHANGELOG, appended to the C3 KS-20 bullet:
     ```
       The daemon now also creates its pipe before it installs the hooks, and a
       pipe it cannot create stops it with
       `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: ...`
       with no hook installed; it used to keep remapping with no `swch`
       control. Pipe names are shared by every session (KS-40), so under fast
       user switching or Remote Desktop a second signed-in user's daemon now
       stops this way while the first user's daemon runs, until roadmap item
       4.1 gives each session its own pipe.
     ```

Bisect: C1 is test-only. C2 adds an unused library API. C3 is the KS-20 behaviour. C4 is a refactor with no behaviour change. C5 is the order/fatal behaviour. Each user-visible change carries its README and CHANGELOG lines. `default_config.ron` is untouched (D6).

## Acceptance mapping

| Spec / roadmap acceptance line | Covered by |
|---|---|
| The mutex is `Local\key-switch-rs`, created with `CreateMutexW` | T6; review: one `CreateMutexW` call site, in `single_instance.rs` |
| ... before the config loads and the hook is installed | D3: the first statement of `main`; M1b (a second start with a missing config path still prints "already running", not `config file ... does not exist`) |
| `ERROR_ALREADY_EXISTS` → prints "already running", exits with code 2 | T1, T8; M1 (`$LASTEXITCODE` is 2) |
| A second acquisition returns `Err(AlreadyRunning)` (roadmap `:279`) | T1, T2, T4 |
| The name is injectable; tests use a unique name | `acquire(name)`; `unique_name`; T1-T4, T7 |
| A failure to create the pipe is fatal: a clean exit with the hooks uninstalled | C5 order (no hook before `bind`; `uninstall` on a `serve` failure); I1 (a taken name fails with 5); `pipe_error_message` tests; M4 |
| `swch open` ACCESS_DENIED/PIPE_BUSY handling stays | unchanged `ipc::tests::only_a_missing_pipe_means_no_daemon`, `connect_errors_are_classified`, `tests/ipc_roundtrip.rs::request_reports_access_denied_for_a_pipe_this_process_may_not_open`, `request_against_a_pipe_with_no_free_instance_reports_the_daemon_running`; review: no diff in `swch.rs`, `ipc.rs` |
| A5-m1: the callback's enabled fast gate is pinned | C1 test; MU-A, MU-B |
| Manual: a second start prints "already running" (roadmap `:279`, checkpoint item 9 `:313`) | M1, M2 |
| README and CHANGELOG | C2, C3, C5 |

## Manual checks (human only)

Release build unless the row says otherwise. Run the elevated console as administrator. Agents never run these.

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | Daemon started with `swch open` | In an elevated PowerShell: `& .\target\release\key-switch-rs.exe; $LASTEXITCODE` | One line, ``key-switch-rs: already running in this session; use `swch status`, `swch off` or `swch exit` to control it``, then `2`. No banner, no `[INFO] loading config`, no prompt. Task Manager shows one `key-switch-rs.exe`. `swch status` answers `running`. CapsLock switches the layout once per press | none |
| M1b | As M1 | `& .\target\release\key-switch-rs.exe C:\does\not\exist.ron; $LASTEXITCODE` | The same line and `2`, not `[FATAL] config file ... does not exist`: the guard comes before the config load | none |
| M2 | As M1 | Double-click `key-switch-rs.exe` and accept UAC | A console opens and closes at once; still one daemon; `swch status` answers | none |
| M3 | Release daemon running (high integrity) | Normal console: `& .\target\debug\key-switch-rs.exe; $LASTEXITCODE`. Then reverse it: `swch exit`, start the debug daemon from the normal console, double-click the release exe, accept UAC, check Task Manager, `swch exit` | The debug build prints the already-running line and `2` (the `ERROR_ACCESS_DENIED` path across integrity levels, D2). In reverse, the release start closes at once and only the debug daemon remains | none |
| M4 | No daemon running | Console A: `cargo test --locked -- --ignored --nocapture hold_the_daemon_pipe_name_for_a_manual_start`. Within 60 s, elevated console B: start the release daemon. Press CapsLock in Notepad. Press Enter in B | B shows `[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: Access is denied. (os error 5). Another program owns that pipe name: ...` and `Press Enter to exit...`, with no banner and no `✓ Hook installed`. CapsLock toggles caps lock (no hook). After Enter, exit code 1 | `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start` |
| M5 | Daemon running | `swch exit`, then `swch open`. Then End task on the daemon in Task Manager, then `swch open` | Both restarts work (the mutex goes with the process) | none |
| M6 (optional) | Two signed-in users (fast user switching), the daemon running for user A | User B starts the release daemon | B gets the M4 FATAL line (Q1, KS-40) | none |

## Risks and edge cases (R1-R10 and more)

1. **R1 hook callback.** No product change in either callback. The new test calls both extern callbacks directly with no hook installed. `CallNextHookEx(None, ..)` with no hook is K3's documented precedent (`:504-510`). The holder thread blocks inside `with_engine`, which only a test does, and it is bounded by 5 s.
2. **R2 sentinel.** No `SendInput`. N/A.
3. **R3 shared state.** No new hook state. `InstanceGuard` is a local of `main`/`run`, with no static. `GLOBAL_HOOK: Mutex<()>` is test-only and guards no hook state; it serialises two tests. `with_engine` remains the only engine accessor.
4. **R4 orderings.** No new atomics in product code. The test `COUNTER` uses `Relaxed`, with the written justification.
5. **R5 builders.** The `App` builder is unchanged. `run`'s error type changes; its only caller is `main.rs:36`.
6. **R6 Display.** `InstanceError` has an explicit arm per variant and no `_ =>`. T8 pins both texts.
7. **R7, R8, R9.** No specificity, key-identity or registry change. A new kernel object: `CloseHandle` runs on every path. The `ALREADY_EXISTS` path closes before it returns (T2). Every other path holds the handle in the `Drop` guard.
8. **Squatting.** A process in the user's session can create `Local\key-switch-rs` first. A mutex makes us exit with 2; another object type makes us stop with `[FATAL] could not create the single-instance mutex`. The same holds for the pipe (KS-40). The CreateMutexW doc names this risk ("a malicious user can create this mutex before you do"). Accepted: such a process already runs with the user's rights. A private namespace would be a follow-up if wanted.
9. **R10 startup and shutdown.**
   - New order: guard → config → `bind` → banner/Ctrl+C → `install` → `serve` → watcher → loop → `uninstall`.
   - No hook is live without a pipe, except for the microseconds between `install` and `serve`.
   - Exit paths: `bind` fails, nothing to undo. `install` fails, its own rollback runs (`keyboard_hook.rs:202-210`) and the pipe closes with the dropped listener. `serve` fails, `uninstall` on main, then fatal. Normal exit unchanged.
   - The guard is released after `App::run` returns, so after `uninstall`.
   - Pre-existing edge: if `uninstall` fails at shutdown (`app.rs:189`), the FATAL prompt waits with the `ipc-server` thread still serving and the guard released. A new start then fails at `bind` with the pipe FATAL. Rare, and not made worse.
10. **Handle and token inheritance (KS-09).** The mutex handle is not inheritable (NULL attributes, doc quote), so `Launch`/`OpenUrl` children never keep the name alive. A future change to inheritable attributes would break M5. The `pub fn acquire(name)` signature takes no attributes.
11. **Elevation and UIPI.** Release daemons are high integrity, debug builds medium. Both orders are covered by D2 (ALREADY_EXISTS or ACCESS_DENIED) and M3. The pipe DACL is unchanged (`ipc_server.rs:61-77`).
12. **`Win32_Security` glob exposure.** New gated items become visible in `Threading::*` (`app.rs:4`) and `Registry::*` (`registry.rs:2`). An ambiguity error appears only if a name is used. The developer's `cargo check --all-targets` after C2 is the check.
13. **Reload, `swch off`/`on`, foreground change, pending gestures.** Unaffected: the guard and the pipe live for the process. A `swch exit` during startup is queued in the main thread's queue, created by `SetWindowsHookExW`, so `GetMessageW` sees `WM_QUIT` at once and the normal teardown runs.
14. **`swch open` race.** In the window between a daemon's guard and its `bind` (the config load), `swch open` sees no pipe and launches a second daemon, which exits with 2 in its own console while `swch` prints `daemon launched`. Harmless (follow-up F2).
15. **Later lanes.** 1.7 (logging) must keep the already-running line on the console. Its `rg "println!|eprintln!"` acceptance check (`roadmap.md:299`) has to allow it next to the banner. 4.2 (no console) must route the already-running line and the FATAL lines to its `MessageBoxW`/log. 4.1 (session pipe) removes the Q1 restriction.
16. **IME and layouts.** N/A.

Follow-ups (not in this lane):
- F1: the KS-40 cross-session pipe (4.1).
- F2: `swch open` waits for the launched daemon with `ShellExecuteExW` + `SEE_MASK_NOCLOSEPROCESS`, so that it can report exit code 2 (no KS id).
- F3: a private namespace against squatting (no KS id).

## Questions for the orchestrator

1. **A second signed-in user's daemon now stops (KS-40 interplay).** Pipe names are machine-wide (`audit.md:715`). Today user B's daemon in session 2 runs with its hooks and no IPC, and B's `swch` reaches user A's daemon. With "pipe failure is fatal", B's daemon stops with the M4 FATAL line until 4.1 puts the session ID in the pipe name. **Recommendation:** implement the spec as written. The lane documents the restriction in CHANGELOG and README and names it in the error hint. B's daemon could not be controlled before anyway, and pulling 4.1's session pipe forward would widen this lane into `ipc.rs`/`swch.rs`, which the spec freezes. Not blocking.
2. **No "Press Enter" on "already running".** **Recommendation:** no prompt (D3). Scripts and the 4.5 task must not park a second console on stdin, and the first daemon's console shows that a daemon runs. 4.2 can add a `MessageBoxW`.
3. **Book pages outside `config/`** (list in the Touch set). **Recommendation:** leave them to the phase book sync (`docs/WORKFLOW.md:103-107`), which knows the merge sha. No `book/src/config` page changes meaning in this lane.
4. **`ERROR_ACCESS_DENIED` as "already running"** cannot tell a higher-integrity daemon from a squatter. **Recommendation:** accept (D2). T4 confirms the code on the DACL path and M3 on the integrity path. If the orchestrator wants it settled on paper first, the researcher query is: "What does CreateMutexW return when a medium-integrity process opens a named mutex created by an elevated process of the same user and session with a default security descriptor: ERROR_ACCESS_DENIED, or a handle with ERROR_ALREADY_EXISTS?"

## Cut readiness checklist

**Spec**
- [x] KS-20 read (`audit.md:495-508`). Its scenario, a double-click while a daemon runs, is M1/M2. A second acquisition is T1. KS-40 and KS-09 interplay are covered by risks 9 and 10 and Q1.
- [x] Every book reference re-located (table). The `swch` half is gone (fixed in `a1b5a27`) and stays. The guard and pipe-order defects are present.
- [x] Goal, non-goals and dependants (4.5) stated.

**Design**
- [x] D1-D7 each have a rationale and rejected alternatives.
- [x] Owner decisions: matching, `repeat`, RON untouched; `eprintln!` until 1.7 (risk 15); licence N/A.
- [x] A thread is named for every new path (Threads table). Nothing is added to the LL callback.
- [x] Install and uninstall stay on the main thread; the serve-failure `uninstall` runs there too. No timers.
- [x] `SendInput`: N/A.
- [x] Sided and unsided modifiers: N/A (no matching change).
- [x] Reload, `swch off`/`on`, foreground change, shutdown during a pending gesture: risk 13. Startup and shutdown exits: risk 9.
- [x] No new product atomics and no `static mut`. The test counter is `Relaxed`, justified.
- [x] No new module cycle (`system::single_instance` imports only `std`, `windows`; tests also `interprocess`). Engine N/A (D6).

**Win32**
- [x] Each API: `windows` path, feature check (`Win32_Security` is added), thread, errors, doc link and quote. Unverified points are marked (the integrity-level error code, the shared namespace across elevation) and tested by T4/M3.
- [x] Elevation, UIPI, pipe DACL and handle inheritance: risks 10 and 11.

**Commits and tests**
- [x] Red-first: C2, C4 and C5 compile-fail on `master`. C1's red is MU-A/MU-B. Behavioural reds are MU-C to MU-E. C3 and C5's order have manual proofs, declared as such.
- [x] Nothing installs a hook or injects input. The mutexes and pipes are per-test unique names; the one ignored test holds the real pipe name, for a human.
- [x] Every test that needs a running daemon is a manual check. The one helper test is `#[ignore = "manual: ..."]` (M4).
- [x] Every commit passes fmt, clippy `-D warnings` and `test --locked`. No declared red commit.
- [x] Bisectable. README and CHANGELOG lines land in the commit that changes the meaning (C2 features, C3 guard, C5 fatal pipe). The template is untouched.

**Acceptance**
- [x] Every acceptance line maps to a test or a manual check.
- [x] Every question has a recommendation.

## Developer addendum (implementation round 1, branch `feat/single-instance` @ `5725499`)

Critique `critique.md` has no Critical or Important remark. The four Minors and the open questions end as follows (details and test names in `impl.md`).

- **M1 (D5 release send).** RESOLVED: step 6 ignores the send result (`let _ = release_tx.send(())`, `src/hook/keyboard_hook_callback.rs`, test `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock`). Measured reds: MU-A "with remapping off the keyboard callback waited 5.0145432s for the engine lock", MU-B "... foreground callback waited 5.0132333s ...". Predict "about 5 s" (the measured value lies just above 5 s, not below).
- **M2 (Fixed vs Changed).** RESOLVED: the fatal-pipe behaviour and its second-user consequence are a separate `### Changed` bullet (KS-20, roadmap 1.5, KS-40). The `### Fixed` KS-20 bullet and the README paragraph state only the guard's scope ("The check covers the Windows session the daemon starts in"); after C5 the README sentence adds "a second signed-in user's daemon stops at the pipe instead (see above)".
- **M3 (code 2 only checked by hand).** RESOLVED by recording, the critique's second option. The mapping stays in `main.rs` as cut (moving it into the library would be a design change beyond the cut). Recorded: the comment at the guard call in `src/main.rs` says the already-running path is checked by hand only (no harness, KS-38); the CHANGELOG KS-20 bullet carries the manual check (a second start prints the line, `$LASTEXITCODE` is 2). **For lanes 1.7 (logging) and 4.2 (no console): M1 is the only guard of "exit code 2, one stderr line, no Enter prompt"; both lanes must re-run M1/M1b after touching `main.rs`.** The roadmap note is the phase book sync's (book owned by the doc-writer).
- **M4 (SAFETY in `App::run`).** RESOLVED: SAFETY comments on `GetCurrentThreadId`, the message loop and `SetConsoleCtrlHandler` (`src/core/app.rs`). `console_ctrl_handler`'s inner block (untouched) still has none; listed as pre-existing.
- **Open question 2 (normal-exit window, risk 9).** Added to risk 9: on a clean exit the guard is dropped when `main::run` returns, before process exit closes the `ipc-server` pipe; a start landing in that window passes the guard and stops at `bind` with the pipe FATAL and no hook. Harmless.
- **Open question 3 (T4 setup).** The squatter's `CreateMutexW` uses `.expect("the creator of a new mutex gets a handle whatever its DACL")`; if it ever fails, report it and do not change the SDDL. On this machine it succeeds and the second acquire is `ERROR_ACCESS_DENIED` -> `AlreadyRunning` (T4 green; MU-E turns it red).
- **Open question 4 (CLAUDE.md Layout).** Not edited by the developer (agents do not change CLAUDE.md). For the orchestrator: `src/system/` now also holds `single_instance.rs`.

No deviation from the D1-D7 design. Wording deviations are listed in `impl.md`.
