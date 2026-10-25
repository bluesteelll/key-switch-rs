# Architecture critique: p1-1.5-single-instance / roadmap 1.5, KS-20, A5-m1 (round 1)
Verdict: APPROVED (Critical 0, Important 0, Minor 4)
Plan: D:/tmp/ks-orch/p1-1.5-single-instance/cut.md   Code read at: master working tree (cut cites `f0048a3`); every `path:line` the cut relies on was re-checked and matches.

No Critical or Important findings. I walked sections 1-11. Sections 3, 4, 5, 7 and 10 do not apply (no matching, suppression, gesture, config or SendInput change); they are listed under Absent topics. The four Minors below are optional and do not block implementation.

## Critical (implementation must not start)
None.

## Important (must be resolved; options may be discussed)
None.

## Minor (improvements, not blockers)

### M1. D5: the result of the release send is unspecified, so the predicted red can be masked
Where: plan §D5 table steps 2, 6, 7 (cut.md:213, :217-218); commit 1 predicted reds (cut.md:293); src/hook/keyboard_hook.rs:95-104 (`with_engine` blocks on `self.engine.lock()`).
Problem: under MU-A the keyboard callback blocks in `with_engine` until the holder's `release_rx.recv_timeout(5 s)` gives up. The holder's closure then returns, the lock is released, and the holder thread ends and drops `release_rx`. By the time the test thread reaches step 6, the receiver may already be gone. `release_tx.send(())` then returns `Err(SendError)`. The plan does not say what step 6 does with that `Result`. If the developer writes `.expect(..)`, the test panics at step 6 before the step-7 timing assertion runs. MU-B fails the same way through `fg`.
Consequence: the tester replays MU-A and, depending on whether the holder thread has dropped `release_rx` yet, gets either the predicted `with remapping off the keyboard callback waited ...` or a `SendError` panic. A red with the wrong reason counts as a prediction mismatch and costs a round. The measured wait also reads about 4.99 s, not `5.0…s`, because the holder's 5 s timer starts before step 4.
Confidence: CONFIRMED (traced through plan §D5 steps 2-7 and `keyboard_hook.rs:95-104`).
Breaks: red-first discipline (the reason must be the predicted one).
What is needed: step 6 should ignore the send result, because the holder may already have timed out. Then the only red under MU-A or MU-B is the step-7 timing assertion. Predict "about 5 s" rather than `5.0…s`.

### M2. CHANGELOG and README: the second-user stop is filed under Fixed, and the same text says other users' daemons "do not count"
Where: cut.md:319 (README C3: "Daemons of other signed-in users do not count."), :332 (the same sentence in the CHANGELOG C3 bullet), :348 (README C5), :349-358 (C5 text appended to the same `### Fixed` bullet: "a second signed-in user's daemon now stops this way").
Problem: after C5, one `### Fixed` bullet says both "Daemons of other signed-in users do not count" and "a second signed-in user's daemon now stops". The README paragraph after `:726` and the sentence after `:720` have the same tension. The C5 half changes existing behaviour and takes something away. Before, user B's daemon remapped without IPC. After, B gets `[FATAL] could not create the IPC pipe` and no daemon. Keep a Changelog files that under `### Changed`.
Consequence: on a shared PC with fast user switching, the owner or a user who scans `### Fixed` reads a fix. That B loses remapping until 4.1 appears only halfway through the bullet, and the "do not count" sentence suggests B is unaffected.
Confidence: CONFIRMED (the planned text at the cited lines).
Breaks: CHANGELOG convention (Keep a Changelog sections, CLAUDE.md "Git flow"); accuracy of the docs.
What is needed: put the fatal-pipe behaviour and its multi-user consequence under `### Changed`, citing KS-20, roadmap 1.5 and KS-40. Reword the C3 sentence so it states only the guard's scope (one daemon per session; the guard ignores other sessions) and does not imply that other users' daemons keep working. Make the same change to the README paragraph.

### M3. The code-2 / no-prompt mapping sits where no test can reach it, and two later lanes must rewrite it
Where: plan §D3 (cut.md:120-152); commit 3 ("No automated test is possible", cut.md:318); risk 15 (cut.md:413); Cargo.toml:30-34 (`test = false`, KS-38).
Problem: the rule "`AlreadyRunning` → one stderr line, exit code 2, no prompt; `Os` → the FATAL path with prompt" is only in `main.rs`. The claim "no automated test is possible" holds for `main`, but not for the rule. The rule could live in the library next to `InstanceError` (for example the exit code and the prompt choice as data the error carries), with a unit test. `main` would then only carry it out.
Consequence: risk 15 names roadmap items 1.7 (logging) and 4.2 (no console) as lanes that must edit exactly these lines. Suppose one of them routes `AlreadyRunning` through `fatal()`. The result is exit code 1, plus a "Press Enter" prompt that parks the second instance of a logon Scheduled Task (4.5, `IgnoreNew`) on stdin. That change passes fmt, clippy and test, and only a human running M1 notices it.
Confidence: CONFIRMED (main.rs has no harness; the plan states that no automated test covers code 2).
Breaks: nothing today; it weakens the acceptance proof for "exits with code 2".
What is needed (optional): either move the mapping into the library and pin it with a unit test, or record in the cut and the CHANGELOG/roadmap notes that M1 is the only guard for code 2 and that 1.7 and 4.2 must re-run it.

### M4. C5 reorders `App::run`, whose `unsafe` blocks have no `// SAFETY:` comment
Where: plan §D4 / commit 5 (cut.md:175-183); src/core/app.rs:115-117 (`SetConsoleCtrlHandler`), :122 (`GetCurrentThreadId`), :180-186 (message loop); D:/tmp/ks-orch/p0-A5-ipc/review_r1.md:81 (A5 let them pass only because they were "on untouched lines").
Problem: C5 changes `run`'s order and return type. The new hunks will very likely include `:122` and the loop at `:180`. The touch set has no row for SAFETY comments.
Consequence: CLAUDE.md rule 4 makes a missing SAFETY comment a defect. Once the diff exposes these blocks, the code reviewer flags them and the lane needs a fix round.
Confidence: PLAUSIBLE (depends on how the diff hunks fall).
Breaks: CLAUDE.md coding principle 4.
What is needed: add SAFETY comments for the `unsafe` blocks in the `App::run` that C5 rewrites to C5's touch set.

## Strengths to keep
- **The pipe is created before the hooks (D4).** On the common failures (a taken name, a pre-1.5 daemon, another session's daemon) there is nothing to undo, and no hook is ever live without a control pipe. `serve` after `install` keeps a server thread from answering `running` for a daemon whose install failed. The only `uninstall` that was added runs on the installing thread (KS-01). Verified: `ipc_server.rs:94-127`, `app.rs:121-192`, interprocess `create_instance.rs:85-86` (`FILE_FLAG_FIRST_PIPE_INSTANCE`), and I1 pins the outcome.
- **The mutex is detected by existence, with `bInitialOwner = FALSE` and NULL attributes.** It has no thread affinity and no abandonment states, and the handle cannot be inherited, so `Launch` children (KS-09) never keep the name alive. All quotes checked against https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw ("subsequent processes with sufficient access rights simply open a handle", MUTEX_ALL_ACCESS, ERROR_INVALID_HANDLE, NULL means not inheritable).
- **The handle is closed on the `ERROR_ALREADY_EXISTS` path, and T2 pins it (MU-D).** Without that, a refused start would keep the name alive.
- **`ERROR_ACCESS_DENIED` counts as "already running" (D2).** A medium-integrity build next to the elevated daemon gets the right message, and T4 checks this on the real kernel through a DACL. The `windows` wrapper does nothing between the FFI return and `GetLastError` (`Threading/mod.rs:306-307` checked).
- **The guard is the first statement of `main` and is released before the FATAL prompt (D3).** A second start writes no config template (`loader.rs:158-173`), and a config error leaves no stale "already running".
- **The A5-m1 pin (D5) observes "without touching the engine" through the lock, deterministically.** It pins both gates (`keyboard_hook_callback.rs:328-333`, `:370-372`), and `GLOBAL_HOOK` serialises the only two tests that use `get_hook()` (grep confirms no other caller in tests). `RemappingOnAtEnd` keeps K3's precondition safe on unwind.
- **The order seam was rejected for the right reason.** A test of `App::run`'s order would install a real global hook under the MU-G mutation, which is forbidden; the declared manual check M4 is the right replacement.
- **The cut is honest about what it has not verified:** the integrity-label error code, the namespace shared across elevation, and T7 possibly unable to go red.

## Open questions for the architect
1. **Q1 (KS-40 interplay).** The roadmap text (`roadmap.md:273`, "A failure to create the pipe is fatal") already covers the plan's behaviour, so this is not a finding. Because it takes remapping away from a second signed-in user until 4.1, the orchestrator may still want the owner to acknowledge it. That is a values question and does not block the cut.
2. **Normal-exit window (risk 9).** On a clean exit the guard is dropped when `run` returns, before process exit closes the `ipc-server` pipe. A start that lands in those microseconds passes the guard and stops at `bind` with the pipe FATAL and no hook. That is harmless, so it is not a finding. One sentence in risk 9 would make the analysis complete.
3. **T4's setup step.** The plan says what to do if the second `acquire` returns an unexpected code. It should also say what the tester does if the squatter's own `CreateMutexW` with `D:P(A;;GA;;;SY)` fails. It should not fail, because the creator of a new object gets the access it asks for, but the answer should be "report it, do not change the SDDL", as for the other unverified points.
4. **CLAUDE.md "Layout"** lists `src/system/` as (`system_function.rs`, `registry.rs`). After C2 it is missing `single_instance.rs`, and the phase book sync owns only `book/src`. A one-line edit in C2, or a note for the orchestrator, would keep later lanes (1.7, 4.2, 4.5) from working from a stale map.

## Absent topics (walked, not applicable)
- **1 Hook-thread budget.** No product change in either callback. D5 calls the extern callbacks only from a test, with no hook installed, on K3's precedent (`keyboard_hook_callback.rs:504-510`).
- **2 Thread affinity.** Install and uninstall stay on main, including the uninstall added for a serve failure. `serve` runs after `install`, so the main thread's message queue exists before any `exit` can be posted (risk 13). Every exit path is checked in risk 9 and matches `app.rs`.
- **3 Matching, 4 Suppression lifetime, 5 Time and gestures.** Not touched. `swch off`/`on` and foreground resets are unaffected.
- **6 Pure core.** Not touched. The startup guard is process-level and does not belong in `src/engine`.
- **7 Config and hot reload.** No schema, RON or template change. The guard runs before `ConfigPath::from_arg`/`load`, so a second start writes nothing (M1b).
- **8 Shared state.** No new product static or atomic. The test `GLOBAL_HOOK: Mutex<()>` guards no hook state. The test counter uses `Relaxed` with a written justification.
- **9 Process boundary.** The pipe DACL is unchanged. The new single-instance guard is session-scoped (`Local\`). The handle is not inheritable. No registry access.
- **10 Input synthesis.** No `SendInput`, so the sentinel is untouched.
- **11 Fit.** The builder is untouched. `InstanceError` has a `Display` arm per variant (pinned by T8). Logging uses `eprintln!` until 1.7. `Win32_Security` is a feature of the existing `windows` dependency (`Cargo.toml:41-51`; `windows-0.62.2/Cargo.toml:509`: `Win32_Security = ["Win32"]`), which `Cargo.lock` does not record. The commits bisect: C1 tests only, C2 an unused API, C3 the guard, C4 a refactor, C5 the order. The book is deferred to the phase sync per docs/WORKFLOW.md:105-107, and no `book/src/config` page mentions instances (grep).
