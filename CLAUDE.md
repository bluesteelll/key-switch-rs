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
- `src/core/`: `app.rs` (builder + `GetMessageW` loop), `constants.rs` (the sentinel), `windows_actions.rs`.
- `src/data/`: `binding.rs`, `key_combination.rs`, `trigger.rs`, `condition.rs`, `sequence.rs`, `vk_name.rs`.
- `src/hook/`: `keyboard_hook.rs` (`KeyboardHook`), `keyboard_hook_callback.rs` (the Win32 adapter),
  `config_watcher.rs`, `ipc_server.rs`.
- `src/ipc.rs` (the pipe protocol: one request line, one reply line) and `src/system/` (`system_function.rs`, `registry.rs`).
- `src/engine/`: the pure decision core (`#![forbid(unsafe_code)]`; no Win32 calls, threads, locks or clock).
  `Engine::on_key(event, &bindings, &dyn Env)` and `Engine::on_timer(token)` return a `Decision` (`suppress`,
  `fire`, `timers`; `mask` is reserved for roadmap 1.2). `Env` supplies the clock, the OS key state and the
  foreground window. State: `keys.rs`, `gestures.rs`, `chords.rs`, `sequences.rs`. Scenario tests live in
  `engine::scenarios` on the `engine::testing` harness (`Scenario`, fake `Env`). New matching, suppression and
  gesture logic goes here; the hook callback only adapts Win32 to it and carries out the `Decision`.
- `build.rs` + `app.manifest`: the release daemon embeds a `requireAdministrator` manifest. Only
  the daemon gets it, and the switches go only to the MSVC linker.
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
```

The three gates are the definition of green, both locally and in `ci.yml`. A test that needs a
live hook, injected input or a registry write is `#[ignore = "manual: <what a human checks>"]`, and
`cargo test` does not run it. Report those tests as manual checks. Do not count them as passes.

## Coding principles

1. **The hook callback is the hot path.** Windows stalls all keyboard input until it returns (KS-08).
   No blocking, sleeping, I/O, process creation, new allocations, new locks or Win32 lookups in it
   beyond what the cut prescribes. Never catch panics there. A swallowed panic leaves a phantom-blocked keyboard.
2. **Every path ends in exactly one outcome:** `LRESULT(1)` (drop) or `CallNextHookEx` (pass).
   `code < 0` short-circuits to `CallNextHookEx`. Suppression invariant: a key-up is suppressed
   if and only if its key-down was.
3. **Injected-event sentinel.** Every synthesized event sets `dwExtraInfo: injected_sentinel()`
   (`core/constants.rs`, PID-mixed `(pid << 32) | 0xDEADBEEF`). The callback passes those events
   through via `is_injected_event` before it mutates any state. A `SendInput` path without the
   sentinel causes a feedback loop. Never hard-code the constant.
4. **`unsafe`:** Rust 2024 needs `unsafe { }` inside `unsafe fn`. Every `unsafe` block carries
   `// SAFETY: <concrete invariant>`. A comment like "caller's responsibility" or an empty SAFETY line is a defect.
5. **Shared state:** the hook singleton is `static KEYBOARD_HOOK: LazyLock<KeyboardHook>`, reached
   through `get_hook() -> &'static KeyboardHook` (shared, because the callback is re-entrant). Never use
   `static mut`, `&'static mut`, `RefCell` or a hook-wide `Mutex` for hook state. The one exception is
   `Mutex<Engine>` in `KeyboardHook`, until roadmap item 2.2 moves the Hold timers to the main thread. Only
   `KeyboardHook::with_engine` takes it, and only while the engine decides: no action, `SendInput`, thread
   spawn or message pump under it. `with_engine` refuses a nested call on the same thread (the event passes)
   instead of deadlocking. Other shared state is atomics or
   `ArcSwap`, with `Release` stores, `Acquire` loads and `AcqRel` RMW. `Relaxed` needs a written pairing argument.
   Bindings change only through `KeyboardHook::update_bindings`, which sorts by key count descending
   (stable sort) and then does one atomic store.
6. **Thread ownership:** the main thread installs and uninstalls both hooks, owns their windows and
   pumps `GetMessageW`. Hooks belong to the installing thread (KS-01). A thread that uses Win32 UI
   APIs needs its own pump. Macro steps run on worker threads. Moving hook work off the main thread is
   a design question, so escalate it.
7. **Win32 errors:** propagate `windows::core::Result<()>`, check handles and close them on every
   path (use `Drop` guards like the registry key's), and remember that `RegQueryValueExW` lengths are in bytes. Grouped
   `windows::Win32::{..}` imports. Build `VIRTUAL_KEY(raw)` only at the hook boundary or in
   `vk_name.rs`, and pass a wide-string `PCWSTR` only while its buffer is alive.
8. **House style:** use consuming `with_*` builders and no public setters. Every new variant gets a `Display`
   arm in the same commit. Write `expect("invariant: ...")` instead of `unwrap()`. Keep source ASCII. Comments say why.
   Add no new dependencies unless the cut names them, and use no async runtime and no `tracing`.
9. **Tests:** unit tests go in `#[cfg(test)] mod tests` at the bottom of the file, with real `VK_*` constants and no
   mocks. Decision logic is pinned by scenario tests in `engine::scenarios`. Work is red-first: a regression test fails for the
   right reason before the fix lands.

## Safety rules for agents (non-negotiable)

- The owner uses this Windows machine while agents work.
- Never run `key-switch-rs.exe` or the daemon. Never install a global keyboard or WinEvent hook.
  Never call `SendInput` or otherwise inject input. Never write the registry or change system
  settings. Reads are fine.
- Tests that would need any of that are `#[ignore = "manual: <what a human checks>"]`.
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
- Logging uses the `log` crate plus an in-house rotating file sink. Until that item lands, use `println!`/`eprintln!`.
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
  `book/src/methodology.md#validation-harness`). Build the book and check its links before reporting.
- `audit.md` is the historical audit pinned to `7cb3521`. Update only status lines
  ("Fixed in <sha> (<branch>)"), never the findings themselves.

## Where knowledge lives

- `book/src/audit.md`: issues KS-01..KS-42 with evidence.
- `book/src/roadmap.md`: items with acceptance criteria, which are the definition of done.
- `book/src/architecture.md`: modules, threads, data flow.
- `book/src/config/*.md`: the config reference, pitfalls and error messages.
- `book/src/gaps.md`: documentation drift and gaps. `book/src/methodology.md` records how the audit was done and the validation harness.
- `book/src/overview.md` and `build-and-run.md`: the current state, building, and `swch` usage.
