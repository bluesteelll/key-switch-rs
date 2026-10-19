# key-switch-rs

A Windows-only keyboard remapper in Rust 2024 (GitHub `bluesteelll/key-switch-rs`). A daemon runs a
`WH_KEYBOARD_LL` hook and an `EVENT_SYSTEM_FOREGROUND` WinEvent hook. It reads a RON `config.ron`,
hot-reloaded through `notify` + `ArcSwap`, with Tap/Hold/DoubleTap triggers, Combo/Sequence/Chord
kinds, `when:` conditions on the foreground app/title, and macro Sequences sent via `SendInput`. The
`swch` CLI talks to the daemon over a named pipe (`interprocess`).

**Orchestrator: read [docs/WORKFLOW.md](docs/WORKFLOW.md) before launching lanes.** Lane reports
live in `D:/tmp/ks-orch/<lane>/`.

## Layout

One package with a library, two binaries, a build script and the book:

- `src/lib.rs`: the library both binaries import.
- `src/main.rs` builds `key-switch-rs.exe` (the daemon), and `src/bin/swch.rs` builds `swch.exe` (the CLI).
- `src/config/`: `schema.rs` (serde `Raw*`), `loader.rs`, `parsing.rs`, `default_config.ron`.
- `src/core/`: `app.rs` (builder + `GetMessageW` loop), `constants.rs` (the sentinel), `windows_actions.rs`
  (`BindAction`, `FastAction`), `executor/` (`mod.rs` the `action-executor` thread, `runs.rs` the pure macro runs,
  `testing.rs` the `FakeRunner`).
- `src/platform/`: `timers.rs` (`WaitableTimer`, a high-resolution waitable timer for threads that wait on deadlines),
  `priority.rs` (`RaisedPriority`: the main thread runs at `THREAD_PRIORITY_HIGHEST`), `foreground.rs`
  (`ForegroundCache`: the executable per (HWND, pid), the title read live), `desktop.rs` (`UOI_IO`).
- `src/data/`: `binding.rs`, `key_combination.rs`, `trigger.rs`, `condition.rs`, `sequence.rs`, `vk_name.rs`.
- `src/hook/`: `keyboard_hook.rs` (`KeyboardHook`), `keyboard_hook_callback.rs` (the Win32 adapter),
  `main_thread.rs` (`MainState` with the engine, the gesture timers and the foreground cache, `with_main`, the
  `MainThread::run` loop, reload requests, input signals), `control_window.rs` (`ControlWindow`, the hidden
  top-level window for session notifications and session end), `health.rs` (`HealthState`: the callback heartbeat,
  `HookHealth`, reinstall requests), `installer.rs` (the only Win32 hook and raw-input installers, compiled out under
  test; `FakeInstaller`), `watchdog/` (`mod.rs` the `hook-watchdog` thread, `decision.rs` pure), `debug_hold.rs`
  (debug builds only), `config_watcher.rs`, `ipc_server.rs`.
- `src/ipc.rs` (the pipe protocol: one request line, one reply line) and `src/system/` (`system_function.rs`,
  `registry.rs`, `single_instance.rs`, `known_folder.rs`).
- `src/logging/`: `mod.rs` (`init`, `attach_file`, `flush`, `FATAL_TARGET`), `sink.rs` (line format, rotation),
  `writer.rs` (the `log-writer` thread), `counters.rs` (hook-path counters), `panic.rs` (the panic hook, a 250 ms
  bounded flush, installed by `init`), `testing.rs` (`capture`, test only).
- `src/engine/`: the pure decision core (`#![forbid(unsafe_code)]`; no Win32 calls, threads, locks or clock).
  `Engine::on_key(event, &bindings, &dyn Env)` and `Engine::on_timer(token)` return a `Decision` (`suppress`,
  `fire`, `timers`, and `mask`, the menu mask key of roadmap 1.2). `Env` supplies the clock, the OS key state and the
  foreground window. State: `keys.rs`, `gestures.rs`, `chords.rs`, `sequences.rs`; `schedule.rs` has the
  `Scheduler` trait and the pure `TimerQueue`. `Engine::reset(env, ResetReason)` answers `InputSignal`s. Scenario tests live in
  `engine::scenarios` on the `engine::testing` harness (`Scenario`, fake `Env`). New matching, suppression and
  gesture logic goes here; the hook callback only adapts Win32 to it and carries out the `Decision`.
- `build.rs` + `app.manifest`: the release daemon embeds a `requireAdministrator` manifest. Only
  the daemon gets it, and the switches go only to the MSVC linker.
- `src/bench.rs` (test only): the latency bench, an ignored `bench:` test (see Build and quality gate).
- `book/`: the English mdBook (`book.toml`, `src/SUMMARY.md`, mermaid assets).
- `.github/workflows/`: `ci.yml` runs the quality gate, and `book.yml` builds the book and deploys it to Pages on push to `master`.
- `.claude/`: `agents/` (nine roles), `workflows/` (`ks-lane.js`, `ks-phase.js`),
  `hooks/clarify_gate.py`, `settings.json`.

## Build and quality gate

```powershell
cargo check --all-targets --locked                      # fast type check
cargo build --release --locked                          # release daemon + swch
cargo fmt --check                                       # gate 1
cargo clippy --all-targets --locked -- -D warnings      # gate 2
cargo test --locked                                     # gate 3
mdbook build book -d D:/tmp/ks-orch/<lane>/book         # book; output outside the repo
$env:KS_BENCH_OUT='<file>'; cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1   # latency bench
```

The three gates are the definition of green, both locally and in `ci.yml`. A test that needs a
live hook, injected input or a registry write is `#[ignore = "manual: <what a human checks>"]`, and
`cargo test` does not run it. Report those tests as manual checks. Do not count them as passes.
The latency bench (`#[ignore = "bench: ..."]`, rows R1-R8, `KS_BENCH_LOAD=1` adds R8) is counted apart and never as a pass.
Every lane runs it on its parent and on HEAD (3 runs each) and applies the comparison rules of
`D:/tmp/ks-orch/p2-2.2-timers/cut.md` D10 (R1-R8), `p2-2.3-control-window/cut.md` D14 (R9-R13; R11, the witness,
against 5.4 us) and `p2-2.4-watchdog/cut.md` (R14 p50 <= R14c p50 + 300 ns; the 1 ms-paced rows gate on p50 only;
R15 is evidence; R16, one wake per burst, about 50 us, is declared and not gated); a breach is a defect unless the
cut justified the extra work. A cut that sets a new latency bound probes it with a diagnostic row first.

## Coding principles

1. **The hook callback is the hot path.** Windows stalls all keyboard input until it returns (KS-08).
   No blocking, sleeping, I/O, process creation, new allocations, new locks or Win32 lookups in it
   beyond what the cut prescribes. Never catch panics there. A swallowed panic leaves a phantom-blocked keyboard.
   `keyboard_hook_callback.rs` names no `log::` outside its tests: a per-event condition is a
   `logging::counters::bump(..)`. `logging::flush()` waits up to 1 s, so never call it on the hook path or under `with_main`.
   The hook path runs only `FastAction`s inline; every other action goes through `KeyboardHook::submit` to the executor.
   The watchdog heartbeat (`health().callback_seen()`, one atomic RMW after `code < 0`, before the remapping gate) and
   the debug-only hold are the only other additions.
2. **Every path ends in exactly one outcome:** `LRESULT(1)` (drop) or `CallNextHookEx` (pass).
   `code < 0` short-circuits to `CallNextHookEx`. Suppression invariant: a key-up is suppressed
   if and only if its key-down was, unless Windows has the key down (the OS witness, roadmap 2.3), so Windows
   gets a key-up exactly when it has the key down. The witness is the one sanctioned read of the event's own key.
3. **Injected-event sentinel.** Every synthesized event sets `dwExtraInfo: injected_sentinel()`
   (`core/constants.rs`, PID-mixed `(pid << 32) | 0xDEADBEEF`). The callback passes those events
   through via `is_injected_event` before it mutates any state. A `SendInput` path without the
   sentinel causes a feedback loop. Never hard-code the constant.
4. **`unsafe`:** Rust 2024 needs `unsafe { }` inside `unsafe fn`. Every `unsafe` block carries
   `// SAFETY: <concrete invariant>`. A comment like "caller's responsibility" or an empty SAFETY line is a defect.
5. **Shared state:** the hook singleton is `static KEYBOARD_HOOK: LazyLock<KeyboardHook>`, reached
   through `get_hook() -> &'static KeyboardHook` (shared, because the callback is re-entrant). Never use
   `static mut`, `&'static mut`, `RefCell` or a `Mutex` for shared hook state. The engine and the gesture timers are
   not shared: they live in `hook::main_thread::MAIN`, a thread-local `RefCell<Option<MainState>>` of the main thread,
   reached only through `with_main` (`try_borrow_mut`: a re-entrant call is refused and the event passes, never a
   panic or a deadlock). No action, `SendInput` or message pump runs under the borrow except the one
   `SetWaitableTimer` the timers need. The remapping switch flips only on main, with the `MainState`; a reload
   swaps the bindings on main (`MainState::reload`). The hook path never queries a process: `when:` reads the
   foreground cache, which the foreground callback fills, and the title of another process's window. Other shared state is atomics (`logging::counters::COUNTERS`), a `OnceLock` (the writer queue in `logging::LOGGER`) or
   `ArcSwap`, with `Release` stores, `Acquire` loads and `AcqRel` RMW. `Relaxed` needs a written pairing argument.
   The `ArcSwapOption<ExecutorHandle>` in `KeyboardHook` is stored only by main (`attach_executor`); `HealthState`
   (atomics in `KeyboardHook`) is the watchdog's shared state.
   Bindings change only through `KeyboardHook::update_bindings`, which sorts by key count descending
   (stable sort) and then does one atomic store.
6. **Thread ownership:** the main thread installs and uninstalls both hooks, owns their windows, the engine and
   the gesture timers, and runs `MainThread::run` at `THREAD_PRIORITY_HIGHEST`: drain the requests and due timers,
   then `MsgWaitForMultipleObjectsEx(QS_ALLINPUT, MWMO_INPUTAVAILABLE)` on its high-resolution timer, then pump.
   Hold and waiting-Tap actions run there. `swch on`/`off` and reloads are requests the main thread serves. Main
   installs and removes three hooks (keyboard, foreground, desktop switch) and owns the control window, which is
   dropped after `uninstall` and before `executor.stop()`. A window procedure or WinEvent callback only notes a
   signal and posts `MAIN_WAKE`; resets run in the drain, and a lost-input reset reads no OS state. Hooks belong to the installing thread (KS-01). A thread that uses Win32 UI
   APIs needs its own pump. `Launch`, `OpenUrl` and `Sequence` run on the `action-executor` thread, which owns its STA,
   its bounded queue, its `Delay` timer and its message pump; main starts it before the hooks and stops it after
   they are removed. The `hook-watchdog` thread owns a message-only window, the process's raw keyboard registration
   and a waitable timer; it reads atomics and posts a reinstall request, and only main installs, reinstalls or
   removes hooks. It starts before the hooks and stops after them. The `log-writer` thread alone writes the console
   and the log files; every other thread only queues a line. Moving hook work off the main thread is
   a design question, so escalate it.
7. **Win32 errors:** propagate `windows::core::Result<()>`, check handles and close them on every
   path (use `Drop` guards like the registry key's), and remember that `RegQueryValueExW` lengths are in bytes. Grouped
   `windows::Win32::{..}` imports. Build `VIRTUAL_KEY(raw)` only at the hook boundary or in
   `vk_name.rs`, and pass a wide-string `PCWSTR` only while its buffer is alive.
8. **House style:** use consuming `with_*` builders and no public setters. Every new variant gets a `Display`
   arm in the same commit. Write `expect("invariant: ...")` instead of `unwrap()`. Keep source ASCII. Comments say why.
   Add no new dependencies unless the cut names them, and use no async runtime and no `tracing`. Diagnostics are
   path-qualified `log::info!`/`warn!`/`error!`. The print macros appear only in `core/app.rs` `print_welcome` and
   in `src/bin`; a scan test fails on the tokens anywhere else under `src`, comments and strings included.
9. **Tests:** unit tests go in `#[cfg(test)] mod tests` at the bottom of the file, with real `VK_*` constants and no
   mocks. Decision logic is pinned by scenario tests in `engine::scenarios`. Work is red-first: a regression test fails for the
   right reason before the fix lands. Tests never call `logging::init` or `attach_file`; they use
   `logging::testing::capture` and `config::testing::TempDir`, and the suite prints nothing. Under `cfg(test)`
   `FastAction::run` only records; a test that reaches the engine makes its thread a main thread (`MainThread::new()`).

## Safety rules for agents (non-negotiable)

- The owner uses this Windows machine while agents work.
- Never run `key-switch-rs.exe` or the daemon. Never install a global keyboard or WinEvent hook.
  Never call `SendInput` or otherwise inject input. Never write the registry or change system
  settings. Reads are fine.
- Tests that would need any of that are `#[ignore = "manual: <what a human checks>"]`.
- Every hook or raw-input registration goes through `hook::installer`, whose Win32 code is compiled out under test;
  tests use `FakeInstaller` or a synthetic source, and the installer scan test fails on any other call site.
- Never push, tag, release or change GitHub settings unless the brief is the merge step of a lane.

## Git flow

- One branch per feature, cut from `master`. It is merged into `master` with `git merge --no-ff` (no pull
  requests), and then `master` is pushed. Only the lane's merge step pushes.
- One coherent item per commit, with the subject `<Area>: <what>`. Every commit message ends with a blank line and
  `Co-Authored-By: Claude <model> 5.5 <noreply@anthropic.com>`, naming the model that wrote it: `Opus` for every role
  except the tester, which runs on `Sonnet`.
- Fix rounds add follow-up commits and never amend. Never use `--force`, `--no-verify`, `reset --hard`, `stash`
  or `clean` (`settings.json` denies them).
- `CHANGELOG.md` follows Keep a Changelog. Entries go under `## [Unreleased]` (Added/Changed/Fixed/Removed)
  and cite the KS-xx or roadmap ID.

## Owner decisions (implement against them; never re-open)

- Exact modifier matching, with a per-binding `allow_extra_modifiers`.
- Auto-repeat on by default only for `PressKey` / `VolumeUp` / `VolumeDown`.
- The config stays RON.
- Logging uses the `log` crate plus the in-house rotating file sink in `src/logging` (roadmap 1.7).
- After a key-up the hook never saw (Win+L), the next press of a swallowed, non-repeating trigger key fires (roadmap 2.3, 2026-10-18).
- Another key-down cancels a pending `Tap`/`Hold`/`DoubleTap` whose key passes, as AutoHotkey does (roadmap 2.2, 2026-10-18).
- Binds run with the lowest possible latency (2026-10-19): the key-to-action path is the top non-functional
  priority. Every cut states its effect on that path (per-event work, allocations, locks, Win32 calls, timer
  resolution, wake-ups), and added delay there is a defect unless the cut justifies it and the orchestrator accepts it.
- Licence: MIT OR Apache-2.0.
- Full roadmap scope, new features included.

## Agents, model routing and effort

Nine roles live in [.claude/agents/](.claude/agents/README.md): `architect`, `architecture-critic`,
`developer`, `code-reviewer`, `tester`, `results-analyst`, `project-analyst`, `researcher`,
`doc-writer`. Duties are separate: the developer does not run the full suite, the reviewer does not fix,
the critic does not design, and the analyst does not edit.

**Model routing** (owner, 2026-10-14, mirrors boyko-engine): every role runs on **Opus**
except `tester`, which runs on **Sonnet**. **Effort:** the architect, critic, developer, code-reviewer,
results-analyst and project-analyst run `xhigh`. The tester, researcher and doc-writer run `high`. Nothing runs below `high`.
The frontmatter and the lane templates' `MODEL`/`EFFORT` maps agree, so a workflow never inherits
the session's settings. One per-call override: `project-analyst` as the context-pack scout runs `high`. Never downgrade a role because it is "mechanical". Optimise effort, batch size
and redundant arms instead of the tier. The orchestrator stays on the session model.
(rationale and measurements: [docs/WORKFLOW.md#model-routing](docs/WORKFLOW.md#model-routing))

## Orchestration discipline

- **Clarify before acting.** If a request is ambiguous, use `AskUserQuestion` or enter Plan Mode
  BEFORE any Write/Edit, and never guess. Only values and scope questions go to the owner. Decide technical forks
  yourself and give evidence.
- **Plan-Mode threshold:** a change touching **>= 3 files**, or one you cannot describe in one sentence,
  goes through Plan Mode before the first edit.
- **Backstop hook:** [.claude/hooks/clarify_gate.py](.claude/hooks/clarify_gate.py) (`UserPromptSubmit`)
  adds a reminder when a prompt is imperative but names no file, path or symbol. It reminds and never blocks.
- **Subagents cannot ask the user.** On an ambiguity they stop and return `BLOCKED: <question>` (or
  `CUT BLOCKED` / `STOPPED`) to the orchestrator, which asks the owner.

## Token economy in workflows

Cost is roughly turns times context. Most spend is re-reading context that grows inside one agent.
(details: [docs/WORKFLOW.md#token-economy](docs/WORKFLOW.md#token-economy))

- Do not merge stages to save agents. Keep every checking arm (critique, triage, tester ∥ reviewer).
- Parallel checkers start from one context pack (`pack_rN.md`, at most 15k tokens) written by a scout.
- One owner per heavy gate per round. Rounds >= 2 are delta-scoped.
- Hand off through files: briefs carry paths. Logs stay on disk, so read them with `rg -n` and ranged reads.
  Workflows return verdicts and paths, not full agent texts.

## Docs and the book

- Every repository artifact is in English. Chat with the owner may be in Russian.
- The English mdBook in `book/src` is the public layer, published to
  <https://bluesteelll.github.io/key-switch-rs/> by `book.yml` on push to `master`. Register new pages in
  `book/src/SUMMARY.md`. The `doc-writer` owns the book. A developer whose change alters config
  semantics updates the matching `book/src/config/*.md` page, `README.md` and `default_config.ron` in the same lane.
- Every published RON example is validated against the real loader first (the `cfgcheck` harness,
  `book/src/methodology.md#validation-harness`), which calls `logging::init()` first and `logging::flush()` before
  every exit; read its output with `2>&1`. Build the book and check its links before reporting.
- `audit.md` is the historical audit pinned to `7cb3521`. Update only status lines
  ("Fixed in <sha> (<branch>)"), never the findings themselves.

## Where knowledge lives

- `book/src/audit.md`: issues KS-01..KS-42 with evidence.
- `book/src/roadmap.md`: items with acceptance criteria, which are the definition of done.
- `book/src/architecture.md`: modules, threads, data flow.
- `book/src/config/*.md`: the config reference, pitfalls and error messages.
- `book/src/gaps.md`: documentation drift and gaps. `book/src/methodology.md` records how the audit was done and the validation harness.
- `book/src/overview.md` and `build-and-run.md`: the current state, building, and `swch` usage.
