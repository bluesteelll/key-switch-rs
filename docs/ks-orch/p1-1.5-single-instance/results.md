# Results analysis: p1-1.5-single-instance / roadmap 1.5 (KS-20, A5-m1)

## VERDICT: ACCEPT
**Summary**: The lane meets roadmap item 1.5 and the lane spec. `main` takes a session mutex (`Local\key-switch-rs`, injectable name) before it reads the config. A second start prints "already running" and exits with 2. `App::run` creates the pipe before either hook, so a pipe it cannot create is fatal and no hook is ever installed. The A5-m1 gate is pinned by a test, and I replayed its mutation red myself. Product hook code is unchanged, and no invariant or owner decision is broken. Two items stay manual, as the cut declared: the exit-code-2 path in the harness-less binary, and the bind-before-install order. Both are on the lane's manual list (M1/M1b, M4). M1 is also Checkpoint 1 item 9.

Branch `feat/single-instance` @ `2a91ec6` (6 commits on `master` @ `f0048a3`, merge base = master tip). **Merge `2a91ec6`, not the pack's `5725499`.**

## Quality gate (own run)
Run from `D:/key-switch-rs/key-switch-rs` at `2a91ec6` with `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.5-single-instance/target-analyst`. I used a fresh target directory because the tester reports that the lane tree's `target/debug/incremental` makes rustc 1.98.1 ICE. Work tree clean before and after (`git status --short` empty).

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0, no warnings; re-run after touching `src/lib.rs` shows `Checking key-switch-rs`) |
| `cargo test --locked` | lib `running 496 tests` -> `492 passed; 0 failed; 4 ignored`; `ipc_roundtrip` 6 passed; swch 0; doctests 0. Log: `D:/tmp/ks-orch/p1-1.5-single-instance/analyst-test.txt` |
| Test count | branch `--list` = **502** (4 ignored); master `f0048a3` (git-archive export, `--list`) = **488** (3 ignored). +14 `#[test]` (master 479 -> 493 by `git grep`), no `#[test]`/`#[ignore]` line removed in the diff |
| Ignored (`--list --ignored`, not run) | 4, all `manual:`; one new: `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start` (`src/hook/ipc_server.rs:367`) |
| `cargo doc --no-deps` | 4 warnings, all pre-existing in untouched files (`config/loader.rs` x2, `keyboard_hook.rs` `set_enabled`, `data/vk_name.rs:4`) |
| Own mutation replay (scratch export, not the repo) | MU-A (`keyboard_hook_callback.rs:328` gate disabled): `remapping_off_callbacks_...` FAILED "with remapping off the keyboard callback waited 5.0028565s for the engine lock". MU-C (`single_instance.rs:57` ALREADY_EXISTS check disabled): T1 and T2 FAILED "a second acquisition of a held name must be AlreadyRunning". Others unchanged green |
| Trailers | 6/6 commits carry `Co-Authored-By: Claude (Opus|Sonnet) 5.5` (5 Opus developer, 1 Sonnet tester `2a91ec6`) |
| `Cargo.lock` | unchanged; `Win32_Security` is a feature of the existing `windows` dependency, so `--locked` passes |
| Book | unchanged in this lane, so no `mdbook build` |

## Acceptance criteria
| # | Criterion (spec / roadmap 1.5) | Evidence | Status |
|---|---|---|---|
| 1 | Mutex `Local\key-switch-rs` created with `CreateMutexW` | `INSTANCE_MUTEX_NAME` `src/system/single_instance.rs:24`; one `CreateMutexW` call site `:50`; `the_daemon_mutex_is_local_key_switch_rs` `:236` (tester M7 `Local`->`Global` red) | MET |
| 2 | ... before the config loads and before the hook is installed | `src/main.rs:17` is `main`'s first statement; `from_arg`/`load` at `:55-57`; `App::run` only later. Binary has no harness (`Cargo.toml:30-34` `test = false`, KS-38) | MET by code, manual M1b (cut.md:385) |
| 3 | ALREADY_EXISTS -> prints "already running", exit code 2 | `main.rs:19-25` (`ExitCode::from(EXIT_ALREADY_RUNNING)`, `EXIT_ALREADY_RUNNING = 2` `:9`); text pinned by `already_running_names_the_swch_commands` `single_instance.rs:268`; classification by T1 `:148` | MET; the print and the code are manual M1 (cut.md:384), Checkpoint 1 item 9 (`roadmap.md:313`), CHANGELOG manual-check line |
| 4 | A second acquisition returns `Err(AlreadyRunning)` (roadmap `:279`) | T1 `a_second_acquisition_of_a_held_name_is_already_running` `:148`; T2 `the_name_is_free_again_once_every_guard_is_dropped` `:161` (refused start closes its handle; MU-D/M8 red); T4 `a_mutex_this_process_may_not_open_counts_as_already_running` `:184` (real-kernel ACCESS_DENIED path) | MET (I replayed MU-C red) |
| 5 | Name injectable; tests use unique names | `InstanceGuard::acquire(name: &str)` `:38`; `unique_name` (pid + counter) in tests; no test acquires `INSTANCE_MUTEX_NAME` (grep) | MET |
| 6 | Pipe creation failure is fatal: clean exit, hooks uninstalled | `ipc_server::bind` at `src/core/app.rs:141` before `hook.install()?` at `:153`. A bind error returns before any hook; a `serve` error runs `hook.uninstall()` on the main thread (`:164-170`). I1 `a_second_listener_on_a_taken_pipe_name_is_refused` `ipc_server.rs:352` (real kernel: taken name = os error 5; tester M6b red); P1/P2 `app.rs:308`, `:323` (message; tester M5 red). The order itself (MU-G) has no automated pin | MET by code plus tests on the parts; order manual M4 via `#[ignore = "manual: ..."]` `ipc_server.rs:367` (cut.md:388) |
| 7 | `swch open` ACCESS_DENIED / PIPE_BUSY handling stays | `src/bin/swch.rs`, `src/ipc.rs` not in `git diff --stat`; `connect_errors_are_classified` `src/ipc.rs:273`, `only_a_missing_pipe_means_no_daemon` `:302`, `tests/ipc_roundtrip.rs:163`, `:181` green | MET |
| 8 | A5-m1: when disabled, the callback passes everything without touching the engine | `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` `src/hook/keyboard_hook_callback.rs:582`: a holder thread keeps the engine lock, and both extern callbacks must return in < 1 s and pass. Keyboard gate (MU-A) red, reproduced by me; foreground gate (MU-B) red per developer and tester | MET |
| 9 | Manual: a second start prints "already running" | Checkpoint 1 item 9 (`roadmap.md:313`) + cut M1/M1b/M2/M3 | MANUAL (human, not run) |
| 10 | README and CHANGELOG | README tree, Modules bullet `:133`, pipe sentence after `:722`, Direct-launch paragraph `:734-739`, feature line. CHANGELOG `[Unreleased]` `### Changed` (fatal pipe, KS-20/KS-40) and `### Fixed` KS-20 bullet; KS-20 parenthetical in the A5 bullet edited | MET |

## Issues claimed closed
| ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-20 (guard half) | T1 `single_instance.rs:148`, T2 `:161`, T4 `:184` | Compile-red on `master` (the module does not exist; tester's replay), the house precedent from A5. Behavioural reds: MU-C (replayed by me), MU-D, MU-E | CLOSED. The daemon-level scenario from the audit (a double-click while the task's daemon runs) is manual M1/M2 |
| KS-20 (hook-before-pipe half, `app.rs:91-98` @7cb3521) | I1 `ipc_server.rs:352`, P1 `app.rs:308` | Compile-red on master (`bind`, `pipe_error_message` absent); M5/M6b reds | CLOSED in code; order manual (M4) |
| KS-20 (`swch` half) | already fixed in `a1b5a27`; tests above unchanged and green | n/a | unchanged, as the spec requires |
| A5-m1 (review gap) | `remapping_off_callbacks_...` `keyboard_hook_callback.rs:582` | Pins existing behaviour, so it is green on master by design; red under MU-A (reproduced) and MU-B | CLOSED |

After the merge, KS-20 can move from "Partly fixed" to "Fixed" in the phase book sync. Cross-session behaviour belongs to KS-40 / roadmap 4.1.

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | Product callback code is unchanged: the first diff hunk in `keyboard_hook_callback.rs` is `:383` (tests module). `code < 0` `:322`, gate `:328`, foreground gate `:370` unchanged |
| Engine purity, matching per owner decisions | OK | `src/engine` not touched; no matching, repeat or RON change |
| Thread ownership, sentinel, atomics, SAFETY | OK | install and uninstall only on main (`app.rs:153`, `:166`, `:211`); `serve` after `install`, so the queue exists before any IPC `exit`. No `SendInput`. No new product atomics; test `COUNTER` uses `Relaxed` with a written argument. Every new `unsafe` has a concrete SAFETY comment (`single_instance.rs:45`, `:50`, `:53`, `:71`, `:203`, `:243`; `app.rs:116`, `:134`, `:198`; test helpers `keyboard_hook_callback.rs:437`, `:619`). `InstanceGuard` is a `main`-local, not a static |
| Win32 resources | OK | The mutex handle is closed on every path: refused path `single_instance.rs:60`, `Drop` `:67-75`, no handle on `Os`. `None` attributes make it non-inheritable (KS-09). `SetLastError(ERROR_SUCCESS)` before the call and nothing between `CreateMutexW` and `GetLastError` |
| Config reload, IPC | OK | Pipe SDDL unchanged. The pipe is created before the hooks; failure is fatal. Reload is untouched |
| No key logging | OK | No new log line carries key data. The only new prints are the already-running line, the FATAL text and the manual test's instructions |

## Review status
- Critique (`critique.md`): APPROVED, Critical 0 / Important 0 / Minor 4. All four were resolved in the tree; I checked each: M1 `let _ = release_tx.send(())` at `keyboard_hook_callback.rs:636`; M2 separate `### Changed` bullet; M3 comment at `main.rs:12-16` plus the CHANGELOG manual-check line; M4 SAFETY comments at `app.rs:116`, `:134`, `:198`.
- Code review (`review_r1.md`, @`2a91ec6`): **APPROVED**, Critical 0 / Important 0 / Minor 1. No triage was needed.
- Tester (`test_r1.md`, @`2a91ec6`): **GREEN**. 11 mutations: 9 caught, M6a equivalent (replaced by M6b), M9/T7 equivalent on this OS, M12 manual only. The tester added test `2a91ec6` for the `Error::source` gap (M11).
- Open Minor, worth doing while the context is fresh (optional, not blocking): **review m1**. The doc comment `src/main.rs:46-48` says "`_instance` is a binding, not `_`, so the guard lives until `run` returns". The reviewer showed that a `_` parameter also lives to the end of the body, so the stated reason is wrong; the behaviour is correct. Lanes 1.7 and 4.2 must edit this function, so a one-line rewording (developer) avoids passing on a false language rule.

## Docs and hygiene
- README, CHANGELOG (`[Unreleased]`, Keep a Changelog headings, IDs cited): in sync with the behaviour change.
- No config meaning changed, so `default_config.ron` and `book/src/config/*.md` correctly stay untouched; `known-issues.md` has no KS-20 workaround.
- Book pages outside `config/` are now stale. They go to the phase book sync per `docs/WORKFLOW.md` (ks-phase doc-writer), which is also how the earlier Phase 1 lanes handled `audit.md` status lines (the last `audit.md` commit is `20d2189`). Hand-off list:
  - `build-and-run.md:309`: the WARNING "There is still no single-instance guard ... a double-click does start a second daemon" becomes false on merge.
  - `build-and-run.md:125`, `:441`: the `[WARN]`/`[INFO]` lines no longer exist. Also `:242`, `:401-414` (node H, KS-20 row), `:435-449`.
  - `architecture.md:99` (`ipc_server::spawn`, warn and continue), `:161`, `:549`.
  - `audit.md:43`, `:499` (status line), `roadmap.md:220`, `:271-279`, `:624`.
  - `overview.md:26`, `:100`, `:212`; `glossary.md:221`, `:389`; `gaps.md:225`.
  - Suggest that roadmap Checkpoint 1 item 9 (`roadmap.md:313`) also say "and exits with code 2", and that a line be added for the pipe-held check (M4).
- `CLAUDE.md` Layout (`src/system/`) does not list `single_instance.rs`. This is for the orchestrator; agents do not edit `CLAUDE.md`.
- Branch: cut from the `master` tip `f0048a3`, no merge commits, only this lane's work, every commit trailered, bisectable (the reviewer ran fmt and clippy at every commit). Clean for `--no-ff`.

## Technical debt
- No new `TODO`/`FIXME` in the diff.
- `src/core/app.rs:141-153`: bind-before-install has no automated pin (MU-G). It is declared manual (M4). The rejected seam (cut D4) would need a real hook under the mutation. Recorded only in `cut.md`.
- `src/main.rs:11-31`: the `AlreadyRunning` -> exit 2 / no-prompt mapping is reachable by hand only (KS-38). Lanes 1.7 and 4.2 must re-run M1/M1b (cut addendum; CHANGELOG manual-check line). This is recorded in `cut.md` but **not in the roadmap**; the phase sync should add a note to 1.7 and 4.2.
- 1.7's acceptance check `rg "println!|eprintln!"` must allow the already-running line (cut risk 15). It is not yet in `roadmap.md`'s 1.7 text.
- Follow-ups F2 (`swch open` waits for the launched daemon and reports exit code 2) and F3 (a private namespace against name squatting) exist only in `cut.md:416-419`. **Not recorded in the repository**, so the phase sync should add them to `gaps.md` or the roadmap. F1 is roadmap 4.1.
- `src/system/single_instance.rs:241` (T7) cannot go red on Windows 11 26200, because `CreateMutexW` clears the last error. It is defensive coverage, declared.
- Pre-existing and untouched: the inner `unsafe` in `console_ctrl_handler` (`src/core/app.rs:245-247`) has no SAFETY comment. This is not recorded anywhere; next lane touching `app.rs`.
- Pre-existing (cut risk 9): a failed shutdown `uninstall` leaves the FATAL prompt waiting with `ipc-server` serving.

## Problems
None blocking ACCEPT.

## Positives
- The guard works by existence (`bInitialOwner = false`, NULL attributes). There is no thread affinity and no abandoned state, a crash frees the name, and `Launch` children cannot inherit it. The refused path closes its handle (T2), so a stale "already running" cannot happen.
- `ERROR_ACCESS_DENIED` -> `AlreadyRunning` covers a debug build next to the elevated release daemon. T4 proves the DACL path on the real kernel.
- The `bind`/`serve` split means the common failure (a taken pipe name) leaves nothing to undo, and a server can never answer `running` for a daemon whose install failed.
- The A5-m1 pin observes "does not touch the engine" through the lock without instrumenting `with_engine`. It is deterministic (about 5 s against a 1 s bound) and cannot hang.
- `GLOBAL_HOOK` and `RemappingOnAtEnd` serialise the only two tests that drive `get_hook()` (grep confirms), so the new remapping-off test cannot race K3.

## Notes for later lanes
- **Human, before or right after merge:** M1, M1b, M2, M3, M4 (`cargo test --locked -- --ignored --nocapture hold_the_daemon_pipe_name_for_a_manual_start`, then start the release daemon within 60 s) and M5, as in `cut.md:384-390`. M6 is optional.
- **Values item (cut Q1, critique open question 1):** with a fatal pipe, a second signed-in user's daemon now stops (KS-40) instead of remapping without IPC. This is what `roadmap.md:273` asks for, and it is documented under `### Changed`. If the owner has not acknowledged it yet, the orchestrator should mention it when reporting the merge.
- Run the merge-step gate with a fresh `CARGO_TARGET_DIR` (or `cargo clean`), because the lane tree's incremental cache ICEs rustc (tester note).
- 1.7 (logging) and 4.2 (no console): keep the already-running line on stderr, exit 2 and no prompt; re-run M1/M1b.
- 4.1 (session pipe) removes the second-user restriction; 4.5 (autostart) relies on exit 2 with `IgnoreNew`.
