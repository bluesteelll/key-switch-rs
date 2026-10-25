# Code review: p1-1.5-single-instance @ 2a91ec6 (round 1)
APPROVED (Critical 0, Important 0, Minor 1)

Ref reviewed: `master...feat/single-instance`. Merge base `f0048a3`, head `2a91ec6`. The pack was written at `5725499`. The tester added `2a91ec6` while this review ran ("Tests: pin InstanceError::source ...", test-only, +19 lines in `src/system/single_instance.rs`), and it is reviewed here too. All code was read through `git show` / `git diff`. The gate ran on `git archive` exports in the reviewer's scratchpad, never on the shared work tree.

## Gate: fmt ✅ · clippy ✅
- `cargo fmt --check`: exit 0 at `2a91ec6` and at `5725499`, and also at every intermediate commit (`545c18f`, `32704d0`, `9fe63ea`, `e83e977`).
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 with no warnings at `2a91ec6`, `5725499` and every intermediate commit. The lane's claim that every commit passes the static gate holds, so the branch bisects.
- `cargo test` was not run (the tester's job).

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. The comment on `run(_instance)` gives a wrong reason
**Where**: `src/main.rs:46-48`. **Confidence**: CONFIRMED (a test program was compiled and run).
**Problem**: The comment reads:
```rust
/// `_instance` is a binding, not `_`, so the guard lives until `run`
/// returns: ...
fn run(_instance: InstanceGuard) -> ...
```
This implies that a `_` parameter pattern would drop the guard early. It would not. Function parameters are dropped when the function body ends, whatever the pattern (Rust Reference, "Destructors: scopes of function parameters"). I compiled a two-function program with `rustc --edition 2024`. Both `fn underscore(_: G)` and `fn named(_g: G)` printed their body line before `drop`.
**Failure**: The code is not affected; it is correct as written. A later maintainer (lanes 1.7 and 4.2 must edit this function, cut addendum) may trust a language rule that is false. The real reason for the binding is readability.
**What to do**: Optional. Either state the true reason (the parameter lives until `run` returns, after `App::run` has removed the hooks and before the FATAL prompt), or drop the "not `_`" clause. cut.md D3 has the same wording, but cut.md is not a repository file.

## Checklist walked (absent items have no reachable failure)

**Section 1: conformance**
- The spec's points are each implemented:
  - The mutex `Local\key-switch-rs` is created by `CreateMutexW` in the first statement of `main` (`main.rs:17`), before `from_arg` and `load` (`main.rs:55-57`).
  - `ERROR_ALREADY_EXISTS` leads to one stderr line containing "already running" and `ExitCode::from(2)` (`main.rs:19-25`).
  - The pipe is bound before `install` (`app.rs:141` < `:153`). A bind failure returns before any hook exists, and a `serve` failure calls `uninstall` on the main thread (`app.rs:164-170`).
  - `swch.rs` and `ipc.rs` are byte-identical to `master`.
  - The A5-m1 gate pin is in place.
  - The mutex name is injectable: `acquire(name)`, unique test names, and no test passes `INSTANCE_MUTEX_NAME`.
- D1-D7 of the cut are implemented. The deviations listed in impl.md are wording only, and they match the code.
- Roadmap 1.5 acceptance ("a second acquisition returns `Err(AlreadyRunning)`") is covered by T1, T2 and T4. "A second start prints already running" is manual (M1).
- CHANGELOG: `### Fixed` KS-20 and `### Changed` fatal pipe (KS-20, KS-40) are under `[Unreleased]` and cite IDs. README: tree, Modules bullet, Direct-launch paragraph, pipe sentence, feature line. No `book/src/config` page changes meaning. The other book pages go to the phase book sync (WORKFLOW `ks-phase`).

**Section 2: unsafe**
- `single_instance.rs:45`, `:50`, `:53`, `:71`, `:203` (T4) and `:243` (T7) each have a concrete SAFETY comment, and each holds:
  - `wide` is a NUL-terminated local that lives through the call.
  - T4's `only_system` is interprocess's absolute `#[repr(C)] SecurityDescriptor(SECURITY_DESCRIPTOR)`, and `as_sd()` points at that local (`interprocess-2.4.2/.../owned.rs:24-33`), so it is alive across the call.
- The handle is closed exactly once:
  - On the `ALREADY_EXISTS` path, the guard is built at `:54-56` and then dropped at `:60`.
  - The `Os` path has no handle.
  - Otherwise the handle is closed by `Drop`.
- The wrapper does nothing between the FFI return and `GetLastError`: `windows-0.62.2 Threading/mod.rs:306-307` only runs `is_invalid` / `then_some`, and `from_thread` runs only on the error path.
- In `app.rs`, `:116`, `:134` and `:198` have new SAFETY comments that hold. `console_ctrl_handler`'s inner block (`app.rs:245-247`) is pre-existing and untouched.
- In the tests, `call_keyboard_callback` (`keyboard_hook_callback.rs:438-449`) and the `foreground_changed` call (`:620-631`) both have SAFETY comments. `foreground_changed` really reads none of its parameters (`:360-381`).
- No `extern "system"` callback changed. Product code in `keyboard_hook_callback.rs` is unchanged: the first diff hunk is at `:383`.

**Section 3: atomics**
- The only new atomic is the test-only `COUNTER`, `fetch_add(Relaxed)`, with a written justification that it publishes no data.
- `MAIN_THREAD_ID` keeps its `Release`/`Acquire` pair. `ipc_server.rs` no longer holds a copy of it.

**Section 4: Win32 contract**
- Install and uninstall run on the main thread only. The added `uninstall` runs on the installing thread (KS-01).
- `serve` comes after `install`, so the main thread has a message queue before any IPC `exit` can post `WM_QUIT`.
- No new `SendInput`.
- The mutex handle is not inheritable (`None` attributes). The pipe handle is not inheritable either: interprocess `PipeListenerOptions::new()` sets `inheritable: false` (`options.rs:93`), and local_socket does not override it (`local_socket/listener.rs:35-41`). So a `Launch` child spawned through `Command::spawn` (`windows_actions.rs:252`) keeps neither name alive after the daemon exits (KS-09). This matters more now that a taken pipe is fatal.
- Pipe DACL unchanged.

**Section 5: hot path**
- Both callbacks' product code is unchanged. No allocation, lock or I/O was added.

**Section 6: correctness**
- `acquire` has no check-then-act race: one `CreateMutexW` is atomic open-or-create. Two daemons starting at once leave exactly one.
- `ACCESS_DENIED` counts as `AlreadyRunning`. This covers an elevated holder seen from medium integrity: the DACL grants BA/SY only (BA is deny-only in a medium token), and no-write-up stops `MUTEX_ALL_ACCESS`. The reverse direction gives `ALREADY_EXISTS`.
- T4 pins the DACL path on the real kernel. It also holds for an elevated CI runner: owner implicit rights do not cover `SYNCHRONIZE`/`DELETE`.
- Exit paths, each checked:
  - bind fails: nothing to undo.
  - Ctrl+C setup fails: return before install, listener dropped.
  - install fails: install's own rollback runs, and the listener is dropped on return (`app.rs:153`).
  - serve fails: `uninstall`, then fatal.
  - normal exit: `uninstall`, then the guard is dropped at the end of `main::run`, before the FATAL prompt.
- The guard is released before the prompt waits, so a corrected restart is not refused.

**Section 7: style and build**
- Doc comments are present on the public `InstanceGuard`, `InstanceError`, `INSTANCE_MUTEX_NAME`, `bind`, `IpcListener` and `serve`.
- No `#[allow]`, no `unwrap()` and no `_ =>` arm was added.
- `Win32_Security` is a feature of the existing `windows` dependency. `Cargo.lock` is unchanged, and `--locked` builds.
- Source is ASCII, except the moved `✓` line, which is pre-existing text.

**R1-R10**
- R1: callbacks unchanged. `code < 0` and gate order unchanged. The new test only calls the callbacks with no hook installed, as K3 already did.
- R2: no `SendInput` or `dwExtraInfo` change. Grep of added lines confirms it.
- R3:
  - No new product static. `InstanceGuard` is a local of `main`/`run`.
  - The test-only `GLOBAL_HOOK: Mutex<()>` guards no hook state. It serialises the only two tests that call `get_hook()` (grep: `keyboard_hook_callback.rs:540-602` only).
  - `RemappingOnAtEnd` is declared after the lock guard, so it is dropped first.
- R4: see section 3.
- R5: no setters. `serve(self, ..)` consumes, and the `App` builder is unchanged.
- R6: `InstanceError` `Display` has one arm per variant (`single_instance.rs:89-96`), pinned by T8. `Error::source` is pinned by `2a91ec6`.
- R7: no change to sort or publish.
- R8: no change to key identity.
- R9: no registry access. The new kernel handle is closed on every path.
- R10: the order is guard → config → bind → banner/Ctrl+C → install → serve → watcher → loop → uninstall. The only window with a live hook and no served pipe is `install`..`serve` on the same thread, microseconds long. The pre-existing edges are unchanged: a failed shutdown `uninstall` (cut risk 9) and Ctrl+C before the queue exists.

**Tests that can fail**
- These go red under mutation, as the developer reports and the code supports: T1, T2, T4, T5 (MU-C/D/E); A5-m1 (MU-A and MU-B independently, deterministic about 5 s against a 1 s bound, cannot hang thanks to bounded `recv_timeout`s); I1; P1/P2; and `2a91ec6`.

## Positive
- Detection works by existence, with `bInitialOwner = false`. There is no thread affinity and no abandoned-mutex state, and a crash frees the name.
- The handle is closed on the refused path, and T2 pins it. Without this, a refused start would keep "already running" alive after the real daemon exits.
- `bind`/`serve` split: no hook is ever live because of a pipe failure, and the common failure leaves nothing to undo. `serve` after `install` prevents a server answering `running` for a daemon whose install failed.
- The A5-m1 pin observes "does not touch the engine" through the lock, without instrumenting `with_engine`. Each gate is pinned on its own.
- `pipe_error_message` keeps the OS error text and adds the owner hint only for error 5. I1 links that input to the real kernel result.

## Open questions
1. The bind-before-install order (MU-G) and the code-2 / no-prompt mapping in `main.rs` have no automated pin. A refactor that moves `ipc_server::bind` below `hook.install()?` passes all three gates. Both are declared manual checks (M4, M1/M1b) and were accepted at cut and critique time, so they are not counted as remarks. Settled by: the human running M1, M1b and M4 before merge.
2. T7 cannot go red on this OS under MU-F: Windows 11 26200 clears the last error when it creates the mutex. So `SetLastError(ERROR_SUCCESS)` (`single_instance.rs:45`) has no pin here. This is declared in cut.md:314 and is harmless either way.
3. The branch moved during the round: `2a91ec6` is a tester commit (Sonnet trailer, test-only). The merge step should merge `2a91ec6`, not the pack's `5725499`.
4. For the orchestrator, not the lane: the `CLAUDE.md` Layout line for `src/system/` does not list `single_instance.rs`. Book pages outside `config/` are stale (`architecture.md:99`, `:161`; `build-and-run.md:125`, `:441`; `audit.md:43`) and belong to the phase book sync.
