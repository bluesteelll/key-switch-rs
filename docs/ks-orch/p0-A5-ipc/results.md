# Results analysis: p0-A5-ipc (Workstream A, IPC part: KS-01, KS-02, swch read timeout; KS-31 reset; KS-39/KS-20 side of `swch open`)

Branch `fix/ipc-control` @ `2baadf1`. It was cut from `master` @ `b056b72`, which is still the merge-base, `master` and `origin/master`. 9 commits, 15 files, +1561/-236. The tree is clean, `git worktree list` shows only the lane tree, and the branch is not pushed.

## VERDICT: ACCEPT

**Summary**: Every brief acceptance line and spec bullet has automated evidence. The three roadmap A lines are covered by tests plus manual checks M1/M2/M5 and Checkpoint 1 item 1. I replayed the red-first chain myself. `867da7c` fails exactly S5 and S6 with the predicted texts. A no-op `Engine::reset` turns S1-S6 red. Removing the locked re-check turns K2 red. The gate is green in my own run: 338 passed, 0 failed, 0 ignored, against 307 on `master`. The hooks are now installed only by `App::run`, and the IPC server cannot name the hook (rg checks 1-4 clean). The hot path gains one `Acquire` load. Review r1 is APPROVED (0 Critical, 0 Important) and tester r1 is GREEN. The remaining items are not blocking: the tester's Sonnet trailer (fourth lane in a row; ACCEPT criterion 8 is not strictly met, see P1); the fast gate that no test pins (reviewer m1, which I confirmed by mutation); a README sentence; one new rustdoc warning; and the book-sync and manual-check backlog.

## Quality gate (own run, HEAD 2baadf1)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (rc 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK. Fresh `CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-A5-ipc/target-analyst` (the first run was cached), 35 s. The lib, `swch` and `ipc_roundtrip` were all checked (rmeta present) |
| `cargo test --locked` | **338 passed, 0 failed, 0 ignored**: lib 332 (0.82 s), swch 0, `tests/ipc_roundtrip.rs` 6 (5.02 s), doc 0. Log: `D:/tmp/ks-orch/p0-A5-ipc/analyst-test.txt`. master: 307 (diff: +31 `#[test]`, 0 removed, 1 renamed with its asserts kept) |
| `cargo test --locked -- --list --ignored` | 0 ignored tests (listed only, none run) |
| `cargo doc --no-deps` | 2 warnings. One is **new**: `src/hook/keyboard_hook.rs:132`, public doc of `set_enabled` links the `pub(crate)` `with_engine` (P5). One is pre-existing: `src/data/vk_name.rs:4`. CI does not run `cargo doc` |
| `mdbook build` | not run: `book/src/**` is untouched in this lane |
| Red-first replay (mine, `git archive` exports, engine and hook filters only) | `867da7c`: `167 passed; 2 failed`. S6 fails with `scenario: repeat F13 at +500 ms: expected the event to be passed, it was suppressed` and S5 with `scenario: advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]` (`analyst-red/c2-engine.txt`). HEAD with a no-op `Engine::reset`: 6 failed, S1-S6 (`analyst-red/mut-noop-reset.txt`). HEAD without the locked re-check: K2 FAILED (`analyst-red/mut-no-locked-recheck.txt`). HEAD without the callback's fast gate: **21/21 hook tests pass** (`analyst-red/mut-no-fast-gate.txt`; see P2) |
| Trailer count | 8 of 9 commits carry `Co-Authored-By: Claude Opus 5.5`. `2baadf1` (tester) carries `Claude Sonnet 5.5` (P1) |

## Acceptance criteria

| # | Criterion | Evidence | Status |
|---|---|---|---|
| B1 | Brief: the integration test is green | `status_off_on_status_round_trip_without_a_hook`, `tests/ipc_roundtrip.rs:79`. The real `ipc_server::spawn` runs on `unique_pipe_name` (`:28`, pid + counter) with a stand-in `TestControl` and no hook. It asserts status running, off, status disabled, on, status running, and toggles `[false, true]`. Own run: `6 passed ... 5.02s` | OK |
| B2 | Brief: no `SetWindowsHookExW` reachable from IPC code (rg) | My rg runs. (1) Hook APIs appear only in `src/hook/keyboard_hook.rs` (`:8` import, `:174`, `:192`, `:234`, `:246`, plus comments). (2) `install`/`uninstall` are called only at `src/core/app.rs:131` and `:189` (`App::run`), plus `keyboard_hook.rs:205` (install rollback) and `:261` (`Drop`). (3) `KeyboardHook\|keyboard_hook\|install\|fn (enable\|disable)` in `ipc_server.rs`, `ipc.rs` and `swch.rs` gives rc 1, no match. (4) The `DaemonHandle` impl (`app.rs:32-58`) has no install token, rc 1. Structure: `DaemonControl` (`ipc_server.rs:30`); `install`/`uninstall` are `pub(crate)` (`keyboard_hook.rs:162`, `:226`); `enable`/`disable` are deleted. Tests: K1 `keyboard_hook.rs:405` and A1 `app.rs:232` assert `!is_installed()` | OK |
| B3 | Brief: a read against a server that never replies errors in about 5 s | T2 `request_times_out_against_a_server_that_never_replies`, `tests/ipc_roundtrip.rs:117`: `Err(TimedOut)` with elapsed in `[4.95 s, 7 s)`. The whole file took 5.02 s in my run. The tester's M7 (deadline x2) was caught | OK |
| S1 | Spec: hooks installed once on main, never reinstalled from IPC workers | `app.rs:131` / `:189`, B2 above. `keyboard_hook.rs:157-161` and `:224-225` document the main-thread rule | OK |
| S2 | Spec: `on`/`off` flip an `AtomicBool` checked first; while disabled, only `CallNextHookEx` | K2 `disabled_hook_passes_every_event_without_asking_the_engine`, `keyboard_hook_callback.rs:380`. K3 `extern_callback_swallows_a_bound_key_only_while_remapping_is_on`, `:402`: the extern callback passes while off and swallows again after on. "Checked first" holds by code reading only (`:245-251`, after `code < 0` at `:238`, before the `lparam` deref at `:256`). My mutation that deletes it is not caught (P2) | OK (ordering by reading) |
| S3 | Spec: reset the transient engine state on toggle | `engine::scenarios` S1 `:2282`, S2 `:2308`, S3 `:2332`, S4 `:2352`, S5 `:2371`, S6 `:2394`; U1 `gestures.rs:529`; U2 `keys.rs:549`; K1b `keyboard_hook.rs:440` (a no-op toggle keeps the Hold; a real one drops it). All red-replayed by me (see the gate) | OK |
| S4 | Spec: status reports the real flag | I1 `ipc_server.rs:234`, A1 `app.rs:232` (the tester's M10, "status from `is_installed`", is caught), T1 | OK |
| S5 | Spec: explicit SD via the `ListenerOptions` security descriptor; SY/BA full, IU read/write, medium label; SDDL with comments; a parse test | `PIPE_SDDL` `ipc_server.rs:61-77`, one comment per part; `create_listener` `:94-100` uses `ListenerOptionsExt::security_descriptor`. P1 `pipe_sddl_parses` `:275`; P2 `malformed_sddl_is_rejected` `:280` (shows P1 can fail); shape test `:286` (red under the tester's M6, label deleted); P3 `listener_with_the_pipe_security_admits_this_process` `:313` (red under M4 IU->GR and M5 ME->HI). That the label reaches a pipe created by an elevated daemon is manual (M2) | OK |
| S6 | Spec: swch connect timeout and a 5 s read deadline | `ipc.rs:37` `REPLY_TIMEOUT`; `request` `:94-116` (one `recv_timeout` deadline); `exchange` `:120-136` (`ConnectWaitMode::Timeout`). T2, T5 `tests/ipc_roundtrip.rs:178` (busy pipe, bounded < 3 s), C-1 `ipc.rs:273`. `swch.rs:76`, `:146` use `REPLY_TIMEOUT` | OK |
| S7 | Spec: ACCESS_DENIED / PIPE_BUSY on connect mean "running" for `swch open` | T4 `tests/ipc_roundtrip.rs:160` (SY-only DACL gives `AccessDenied`, `daemon_running() == Some(true)`), T5, C-1, C-2 `ipc.rs:302`. The `cmd_open` branch `swch.rs:76-93` is verified by reading; manual M3 | OK |
| S8 | Spec: remove the dead placeholder | `rg "Duration::from_secs\(5\)" src/bin/swch.rs`: rc 1. `connect`/`round_trip` are deleted | OK |
| S9 | Spec: injectable name, a test without a hook, control behind a trait | `spawn(pipe_name, Arc<dyn DaemonControl>)` `ipc_server.rs:106`; `request(pipe_name, ..)` `ipc.rs:94`; T1. `rg PIPE_NAME tests` finds nothing | OK |
| R1 | Roadmap A `roadmap.md:94`: after `swch off; swch on` the bindings work | K3 (extern path), K2, K1b, S1-S6; manual M1; Checkpoint 1 smoke item 1 (`roadmap.md`, "After `swch off; swch on`, CapsLock switches the language") | OK (live: manual) |
| R2 | Roadmap A `:95`: a non-elevated `swch status` gets an answer from an elevated daemon | P3 admits this (non-elevated) token through the IU entry. The tester's M5 proves the test process is not elevated: `CreateNamedPipeW` refused an `HI` label with error 1314. The elevated case is manual M2. The Checkpoint 1 header (daemon started by `swch open`, `swch` in a non-elevated console) plus item 1 exercise it. No `#[ignore]` stub exists; the brief accepts "listed as a manual check" | OK (manual) |
| R3 | Roadmap A `:96`: with a hung daemon, `swch` exits within 5 s | T2 (automated). Manual M5 suspends only the `ipc-server` thread (critique I1, resolved in the cut addendum) | OK |

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-01 | K3 `keyboard_hook_callback.rs:402` (off passes, on swallows again), K1 `keyboard_hook.rs:405` (no toggle installs a hook), A1 `app.rs:232` (status follows the flag), T1 | Compile-red on `master` (`set_enabled`/`is_enabled`/`DaemonControl` absent; quoted in the bodies of `7bdc72c` and `f0b2b79`). A behavioural replay on `master` is deliberately forbidden: `dispatch(On)` would install a global hook (cut D9). The status symptom is carried by M10 (A1 red). The thread-affinity root cause is proven structurally (rg checks, `pub(crate)`, the trait seam) | Closed |
| KS-02 | P1, P2, P3, shape test (`ipc_server.rs:275-331`) | Compile-red (`create_listener`/`pipe_security` absent, `8dc6843` body). DACL and label regressions are caught by M4, M5 and M6. The audit's elevated-daemon scenario is manual M2 | Closed (live confirmation by M2) |
| KS-31 | S1-S4 (`scenarios.rs:2282-2370`), K1b | Yes. Against a no-op reset S1-S4 fail for the KS-31 reason (quoted in `04b5375`; I replicated it). The `install` check-then-act race is gone structurally: the main thread is the only caller | Closed for on/off. Reset on reload/unlock stays with 2.3 |
| KS-11 (no regression across the reset) | S5 `scenarios.rs:2371`, U1 | Yes: red on `867da7c` (replicated) | Not claimed closed; correctly left to 2.2 |
| KS-39 (side effect, unverified report) | No test drives `cmd_open`. T6 `tests/ipc_roundtrip.rs:205` and T1 show that `request` sends a full line and reads the reply. `swch.rs:76` makes the probe a `request(.., Status, ..)` (by reading) | n/a (the binary is untested; exercising `cmd_open` on `NotRunning` would launch `runas`) | WARN: indirect evidence, acceptable for an unverified side-effect report |
| KS-20 (partly) | T4, T5, C-2 | compile-red | Detection only; the guard is 1.5 |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK / WARN | `code < 0` -> `CallNextHookEx` at `keyboard_hook_callback.rs:238-243`. Gate `:245-251` (one `Acquire` load, before `lparam` is read). The sentinel check `:166` comes before any engine state is touched. Every path ends in `LRESULT(1)` `:261` or `CallNextHookEx` `:242/:250/:267`. Nothing new is allocated, written to a file, spawned or slept on the path. Under the lock there is one extra load. Symmetry is kept for keys held through `on` (seeded `Passed`, S6) and for any event that races the flip (locked re-check, K2). **Deliberate, documented exception**: a key swallowed before `off` and released while off gives Windows a lone key-up (RK6). The spec mandates it ("only `CallNextHookEx`"), S2 pins it and the CHANGELOG KS-01 entry says so. It is not a stuck key. RK7 (a key-down in flight within microseconds of the `on` reset may read as up, and a bound key's first repeat would then be decided as a fresh press) is documented at `keys.rs:107-117`; the window is microseconds per toggle |
| Engine purity, matching per owner decisions | OK | `Engine::reset` `engine/mod.rs:358` reads the OS only through `Env::os_key_down`: no Win32 call, clock or thread. `reset_from_os` (`keys.rs:117-130`) indexes `usize::from(code)` with `code <= 254`, with no allocation. `VIRTUAL_KEY(code)` at `keys.rs:122` follows the `all_held` precedent already on `master` (`keys.rs:209`). Matching, exact modifiers, `allow_extra_modifiers` and the repeat defaults are untouched |
| Thread ownership, sentinel, atomics, SAFETY | OK | Install and uninstall run only on main (`app.rs:131/189`). `set_enabled` takes the engine lock on the IPC worker only to flip and reset (at most 249 `GetAsyncKeyState`; no action, `SendInput`, spawn or pump), the same class as the `hold-timer` taker. Its nested refusal is pinned by K1c `keyboard_hook.rs:478`. `enabled`: `swap(AcqRel)` under the lock `:149`, `load(Acquire)` `:126`, with the pairing documented `:50-59`. No `SeqCst`. One `Relaxed` (test counter, justified, `tests/ipc_roundtrip.rs:29-31`). The three new `unsafe` blocks have concrete SAFETY comments (`keyboard_hook_callback.rs:247-250`, `keyboard_hook.rs:231-234`, `app.rs:51-55`). The pre-existing uncommented blocks `app.rs:115,122,180,202` are untouched. No new `SendInput` |
| Config reload, IPC | OK | Reload is untouched; a reload while off leaves the engine alone, and `on` resets it. The pipe now has an explicit DACL (`ipc_server.rs:61-77`) and `swch` has timeouts. The server-side read deadline, instance cap and exit race stay with 4.1 (KS-26/KS-37/KS-40), as declared non-goals |
| No key logging | OK | The only new output is `[WARN] could not remove the foreground hook` (`keyboard_hook.rs:236`) and the moved startup lines |

## Review status
- `review_r1.md`: **APPROVED**, Critical 0, Important 0, Minor 3. No triage was needed. Critique I1/I2 are RESOLVED in the cut addendum. I checked I2 on the tree: rg check 3 gives rc 1.
- `test_r1.md`: **GREEN**: 338 passed, 11 mutations all caught (M6 only after the tester added `2baadf1`), T1/T2 stable over 5 reruns.
- Open Minor items, worth doing while the context is fresh (none blocks):
  - m1: the fast gate is unpinned. **I confirmed it**: deleting `keyboard_hook_callback.rs:245-251` leaves 21/21 hook tests green. See P2.
  - m2: `README.md:548` still says `open` does not use the pipe. See P3.
  - m3: the trailer. See P1.

## Docs and hygiene
- `CHANGELOG.md` `## [Unreleased]`: Added (`tests/ipc_roundtrip.rs`) and Fixed (KS-01 with KS-31, KS-02 with the cross-user caveat, the swch timeout with KS-20 partly and KS-39), in Keep a Changelog form, citing IDs. The lane changes no config meaning, so `default_config.ron` and `book/src/config/*.md` correctly stay unchanged.
- `README.md`: layout, the `keyboard_hook*` bullets, the typical session, the command table and the pipe/timeout/IU paragraph are updated. One stale sentence remains at `:548` (P3).
- Book, `audit.md` status lines and `roadmap.md` are deferred to the phase book sync (`docs/WORKFLOW.md:104-107`), as in A2-A4. The cut's list (`cut.md:435-441`, plus `roadmap.md:297` from M3) **misses** several stale lines (P4).
- No new dependency; `Cargo.toml`, `Cargo.lock`, `build.rs`, `main.rs` and `lib.rs` are untouched, so `--locked` passes.
- The branch contains only this lane's work, and its merge-base is the current `master`, so the `--no-ff` merge is clean. 8/9 commits carry the trailer (P1).

## Technical debt
- `keys.rs:107-117` RK7, the in-flight key-down at `on`: recorded only in the code doc and `cut.md`. Book sync: `config/known-issues.md`.
- RK6, the lone key-up at `off`: CHANGELOG KS-01 entry and S2. Recorded.
- RK8, a Hold whose `on_timer` ran just before `off` and fires right after `OK: disabled`: **only in `cut.md`**. It belongs to 2.2.
- Review open Q1 (RK9 territory): a macro that still holds a key at the `on` reset is seeded as held+`Passed`, so one activation of a binding on that key is missed. **Only in `review_r1.md`**. Belongs to 2.1/2.3.
- Review open Q2: IU's `GW` includes `FILE_CREATE_PIPE_INSTANCE`, so any interactive process can exhaust instances, and the `ipc-server` loop (`ipc_server.rs:111-123`) then spins on `[WARN] IPC accept error`. **Only in the review**. Belongs to 4.1 (KS-26).
- Review open Q3: a non-interactive logon of the same user (SSH, or Task Scheduler "run whether logged on or not") is now denied by a debug (non-elevated) daemon, which the old owner ACE admitted. A narrow regression. **Not recorded in the repo**. Book sync: `known-issues.md`, and a note for 4.1.
- IU cross-session control and rogue instances: CHANGELOG KS-02 entry and README `:551-557`. Recorded; 4.1.
- Roadmap 2.2 must move the flip, the reset and the re-check to the engine's owner thread: only in the `set_enabled` doc (`keyboard_hook.rs:142-145`) and the cut addendum. Book sync: roadmap 2.2.
- `roadmap.md:297` (2.3: "except blocked ones" at off/on) contradicts S2/S6. On the cut's book-sync list.
- `CLAUDE.md:69-71` ("Only `KeyboardHook::with_engine` takes it, and only while the engine decides") does not mention the IPC worker's reset. Orchestrator.
- The manual checks M1-M7 live only in `D:/tmp/ks-orch/p0-A5-ipc/cut.md:642-650` plus the addendum's M5, outside the repo. Only M1 has a durable line (Checkpoint 1 item 1). P4.
- T2 parks one dummy-server thread and one `ipc-request` thread until the test binary exits (cut RK13/Q6, accepted).
- No new `TODO`/`FIXME` in the diff.

## Problems

### P1. `2baadf1` carries `Co-Authored-By: Claude Sonnet 5.5`, the fourth lane in a row
- **What**: the tester's `Tests:` commit ends with the Sonnet trailer. `CLAUDE.md` (Git flow) and `docs/WORKFLOW.md:233-236` require `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` on every commit, so ACCEPT criterion 8 is not strictly met. `master` already holds three such commits from the same cause (`965ac4d`, `5d0fada`, `3d65ac9`).
- **Impact**: hygiene only. The commit is test-only (`src/hook/ipc_server.rs` +27 inside `mod tests`) and correct: it kills mutation M6, the dropped label. It cannot be fixed without rewriting history. Amend and force are forbidden, and a revert plus re-commit keeps it in history.
- **Root cause**: still the one A2 P1, A3 P2 and A4 P1 named. `.claude/agents/tester.md:22` and `.claude/workflows/ks-lane.js:134` say only "with the Co-Authored-By trailer", without the literal string, while the brief template at `ks-lane.js:107` has it for other roles. The Sonnet tester writes its own model name. `docs/WORKFLOW.md:23` still says the tester makes "no commits", which contradicts `tester.md:22`.
- **Return to**: none for this lane (orchestrator). Options: accept it as before, or rebuild the lane on a new branch from `2b32441` with the test re-landed under the Opus trailer before the merge. Either way, put the literal trailer into `tester.md:22` and `ks-lane.js:134`, and fix `WORKFLOW.md:23`.
- **Re-check condition**: the next lane's `Tests:` commits carry the Opus trailer.

### P2. No test pins the callback's fast gate (reviewer m1, confirmed)
- **What**: deleting `keyboard_hook_callback.rs:245-251` leaves every test green (my run: `21 passed` in `hook::`). The locked re-check in `decide_event` (`:178-182`) gives the same pass result, so the spec's "checked first ... only `CallNextHookEx`" holds by code reading only. K3's comment (`:403-404`, "and of the gate before the engine lock") overclaims.
- **Impact**: none today. A regression would put the bindings snapshot, `HookEnv` and the engine lock on every disabled keystroke, and the disabled path would wait out a reset. That costs hot-path time while off, not correctness.
- **Root cause**: the gate's two layers have the same observable output, and the only test that separates them would have to observe the lock.
- **Return to**: `tester` (optional, non-blocking). For example: hold `get_hook()`'s engine lock on a helper thread and assert that the disabled extern callback returns within a bound. Or have the developer narrow K3's comment.
- **Re-check condition**: deleting `:245-251` turns a test red, or the K3 comment claims only what it proves.

### P3. `README.md:548` says `open` does not use the pipe
- **What**: "All commands except `open` connect to the daemon's named pipe". Since `bf669f3`, `swch open` sends `status` over the pipe (`swch.rs:76`), as the table row `:541` and the swch module doc `swch.rs:15-18` say.
- **Impact**: the README contradicts itself, and a reader concludes that `open` cannot be affected by a hung or denied pipe.
- **Root cause**: the cut's README texts left the line as it was (impl.md:101).
- **Return to**: `doc-writer` at the phase book sync, or a one-line developer fix before the merge (non-blocking).
- **Re-check condition**: the sentence says every command connects and `open` uses the pipe only to detect a running daemon.

### P4. Book-sync backlog: stale lines missing from the cut's list, and manual checks with no durable home
- **What**: besides `cut.md:437-441` and `roadmap.md:297`, these lines go stale at the merge and are not on the list: `gaps.md:145` (on/off "only reinstall the hook"), `:184-185`, `:260-261` (hook lifecycle and IPC server "untested", `PIPE_NAME` not injectable), `:275-276`; `overview.md:16`, `:176`, `:195`. The checks M2 (elevated daemon, non-elevated `swch`, `exit` without `[FATAL]`), M3, M5 and M7 exist only in `cut.md`. Review open Q3 and RK7/RK8 are not in `known-issues.md`.
- **Impact**: once the lane merges, the published book keeps telling users to work around KS-01/KS-02 (for example `build-and-run.md:231`, `:383`, `:391-392`), and KS-02's only live proof (M2) is lost.
- **Root cause**: book work is deferred to the phase sync by design, and the cut's line list was not grepped against `gaps.md`/`overview.md` for KS-01/KS-02 references.
- **Return to**: `doc-writer` (phase book sync).
- **Re-check condition**: `rg -n "KS-01|KS-02|swch off|read timeout" book/src` shows no claim that contradicts the merged code. The Checkpoint 1 smoke checklist (or the lane's own) contains M2, M3, M5 (thread-level suspend only) and M7. `known-issues.md` lists RK7 and review Q3. The KS-01/KS-02 status lines in `audit.md` read "Fixed in <sha> (fix/ipc-control)".

### P5. New rustdoc warning
- **What**: `src/hook/keyboard_hook.rs:132`: the public doc of `set_enabled` links the `pub(crate)` `Self::with_engine` (`rustdoc::private_intra_doc_links`).
- **Impact**: cosmetic. CI does not run `cargo doc`.
- **Return to**: `developer` (optional; drop the link or write it as plain code).
- **Re-check condition**: `cargo doc --no-deps` shows only the pre-existing `vk_name.rs:4` warning.

## Positives
- The `DaemonControl` seam makes KS-01 structural: `ipc_server.rs` cannot name the hook, and T1 drives the real server with no hook. A later regression cannot install a global hook from `cargo test`.
- The flip and the reset share one critical section, and every event re-checks the flag under the lock. Every race with a toggle resolves to "old position, old state" or "new position, reset state". My mutation proves the re-check is pinned (K2).
- The reset avoids the two traps a naive one falls into: it keeps the Hold generation (S5, KS-11) and seeds keys held in Windows as `Passed` (S6, no stuck key), with behavioural red-first tests I replicated.
- The client bounds connect, write and read with one deadline and adds no `unsafe` (`ipc.rs:94-136`). The interprocess behaviours it depends on are traced to source in the cut.
- The critique's Important items were handled well. M5 no longer kills the owner's hook (thread-level suspend), and rg check 3 is clean on a correct tree.

## Notes for later lanes
- 2.2: when `Mutex<Engine>` goes away, `set_enabled`'s flip and reset and the re-checks in `decide_event`/`foreground_changed` must move to the engine-owning thread (`keyboard_hook.rs:142-145`). RK8 belongs there too.
- 2.3: extend `Engine::reset` with a variant that keeps blocked records for reload and unlock. The off/on variant must keep dropping them (S2, S6). Fix `roadmap.md:297` accordingly.
- 2.1: macro cancellation on `off`, and the macro-held key at the `on` reset (review Q1).
- 4.1: instance limit and backoff on accept errors (review Q2), per-session pipe name (KS-40, removes IU's cross-session reach), a server-PID check against rogue instances, KS-26/KS-37. Extend `tests/ipc_roundtrip.rs`.
- 1.5: it builds on the SDDL and on `swch open`'s classification (`RequestError::daemon_running`).
