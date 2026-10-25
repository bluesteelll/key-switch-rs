# Architecture critique: p0-A5-ipc (KS-01, KS-02, swch read timeout) (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 4)
Plan: D:/tmp/ks-orch/p0-A5-ipc/cut.md   Code read at: master b056b72 (clean)

I read the whole cut and the code it touches: `ipc_server.rs`, `keyboard_hook.rs`, `keyboard_hook_callback.rs`, `app.rs`, `main.rs`, `ipc.rs`, `swch.rs`, `engine/{mod,keys,gestures,env,testing}.rs`, plus audit KS-01/02/31/37/39/40 and roadmap A, 1.5, 2.2, 2.3, 2.4 and 4.1. I checked every interprocess 2.4.2 fact the design depends on against the registry source: `local_socket.rs:17-29`, `try_clone.rs:66-69` (it really does write the group into the owner slot), `named_pipe/local_socket/stream.rs:25-27,38-45` (local-socket connect ignores the wait mode and always uses Unbounded), `ctor.rs:84-105`, `c_wrappers.rs:135-174,201-203` (the client opens with GENERIC_READ|GENERIC_WRITE and turns ERROR_PIPE_BUSY into a spin with WaitNamedPipeW), `misc.rs:131-135`, `listener.rs:63-79,123-125` and `create_instance.rs:51-75`. All of them hold. I also traced the red-first predictions. On C1's naive reset, S5 fails with exactly `advance 150 ms to +250 ms: expected to fire [], fired [SwitchLanguage]` and S6 with `repeat F13 at +500 ms: expected the event to be passed, it was suppressed`, and S1-S4 stay green (`testing.rs:287-309,388-407,481-501`).

No Critical findings. The design is sound. Two Important items have to be fixed before work starts: a manual check that kills the owner's hook, and an acceptance rg check that fails on a correct implementation.

## Critical (implementation must not start)

None.

## Important (must be resolved)

### I1. Manual check M5 suspends the process that owns the LL hook, and Windows then silently removes the hook
Where: plan "Manual checks", row M5 (cut.md:648); also the acceptance mapping, roadmap A `:96` -> "T2, M5" (cut.md:622).
Problem: M5 says "suspend `key-switch-rs.exe` (elevated Resource Monitor, Suspend Process); `swch status`; resume". Suspending the process also suspends the main thread that installed `WH_KEYBOARD_LL` (`app.rs:91`) and pumps `GetMessageW` (`app.rs:125-131`). Microsoft Learn, LowLevelKeyboardProc (https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): "If the hook procedure times out, the system passes the message to the next hook. However, on Windows 7 and later, the hook is silently removed without being called. There is no way for the application to know whether the hook is removed."
Consequence: the owner types `swch status` while the daemon is suspended. The first keystroke stalls for up to LowLevelHooksTimeout (at most 1 s), then Windows removes the hook. After resume the daemon looks healthy: `swch status` answers `running (N bindings live)`, because the flag is true and 2.4's watchdog does not exist yet. But CapsLock no longer switches the layout and no binding fires until the daemon is restarted. M5's "Expected" column mentions none of this. The owner ends the lane's verification with a dead remapper and may report it as a KS-01 regression ("after the lane the bindings stop working"). That is exactly the symptom this lane fixes, so the wrong item gets blamed.
Confidence: CONFIRMED (plan row M5; documented Win32 behaviour quoted above; `swch status` reads only the flag per D4).
Breaks: KS-08 (silent hook removal); the owner's desktop.
What is needed: a manual hung-daemon check that does not stall the hook thread. Failing that, M5 must say plainly that it costs the hook and that the daemon must be restarted afterwards.
Options: (a) suspend only the `ipc-server` thread, which is named, in Process Explorer's Threads tab. A client still connects to the stored instance and its read then hangs for 5 s, while the main thread keeps pumping. (b) Stop the daemon and run a silent stand-in listener on the real pipe name. (c) Rely on T2 alone and drop M5.

### I2. The acceptance rg check 3 matches on a correct implementation
Where: plan "The four rg checks", item 3 (cut.md:631); D8, which keeps `swch.rs:80-125` "as today" (cut.md:336); Touch set row for `src/bin/swch.rs` (cut.md:419).
Problem: check 3 is `rg -n "KeyboardHook|keyboard_hook|install|fn (enable|disable)\b" src/hook/ipc_server.rs src/ipc.rs src/bin/swch.rs`, and the expected result is "matches nothing". `src/bin/swch.rs:80` reads `// Daemon lives next to us in the install directory.`, inside the launch code that D8 keeps unchanged. The pattern `install` matches it today and will still match after C6.
Consequence: the tester runs the brief's acceptance line "no `SetWindowsHookExW` call is reachable from IPC code (show with rg)" exactly as the cut specifies, gets a hit, and reports the KS-01 acceptance as FAILED on a correct implementation. That costs a fix round, or the tester quietly weakens the regex, and the acceptance is then no longer the one the cut promised.
Confidence: CONFIRMED (`rg -n install src` today: `src/bin/swch.rs:80`).
Breaks: brief acceptance 2 (a proof that fails on a correct tree).
What is needed: a check-3 pattern that targets calls or identifiers rather than the bare word, or a Touch-set line that rewords the comment at `swch.rs:80`. Then re-run the expected result against the planned touch set, including the new module docs of `ipc.rs` and `ipc_server.rs`, which must also avoid the word.

## Minor (improvements, not blockers)

### M1. Hook-thread affinity is still enforced only by a comment
Where: D1 (cut.md:92); D4 `DaemonHandle { hook: &'static KeyboardHook, .. }` (cut.md:180).
Problem: `install`/`uninstall` become `pub(crate)` with the doc line "main thread only". `DaemonHandle` runs on the IPC worker and holds a `&'static KeyboardHook`, so at the type level an IPC worker can still reach `install()`. Only review and rg check 4 stop that.
Consequence: none today. A later change to `DaemonHandle` (for example 2.4 "reinstall") could call `install()` from the worker. That brings KS-01 back, and nothing fails to compile.
Confidence: CONFIRMED (shape), no current defect.
What is needed (optional): make the affinity structural. For example, `install` and `uninstall` could require a `!Send` main-thread marker that only `App::run` can create, or `DaemonHandle` could hold a narrower handle that exposes only `set_enabled`, `is_enabled` and the binding count.

### M2. The KS-31 core symptom never goes red
Where: C1 (cut.md:460-470); D9 red-replay rule (cut.md:360).
Problem: S1-S4 land in the same commit as the naive reset and are green there. On every parent they fail only to compile. S5 and S6 are honest behavioural reds, but they pin the generation counter and the OS seeding, not "a Hold armed before `off` fires while remapping is off".
Consequence: a harness bug that hides KS-31 would go unnoticed. For example, a `remapping_off` that also drops `self.timers`, or a `key_event` while off that still consults the engine. S1 would then be green with any reset, a no-op included. The book would record KS-31 as fixed without a test that ever failed for the KS-31 reason.
Confidence: PLAUSIBLE (no such bug is planned; the gap is in the proof).
What is needed: show S1-S4 failing for the KS-31 reason once. For example, the tester replays S1-S4 against a no-op `Engine::reset` on C1 and quotes the failures, or C1 lands with a no-op reset and the naive reset follows.

### M3. Later items this lane binds are missing from the dependents list and the book-sync list
Where: "Later items that depend on this lane" (cut.md:78); D2 "Rejected" (cut.md:137); book-sync lines (cut.md:435-441).
Problem: (a) The IPC worker becomes a new engine user under `Mutex<Engine>` (D2 "Thread"). Roadmap 2.2 removes that lock and moves the engine onto the main thread. From then on `set_enabled`'s reset, and the gate re-check under the lock, must move to the main thread, but 2.2 is not listed. (b) `roadmap.md:297` (item 2.3) says `reset_transient_state()` "resyncs all 256 keys, except blocked ones. It runs on `swch off` and `swch on`". D2 deliberately rejects keeping the blocked records at `on`, yet that roadmap line is not in the book-sync list.
Consequence: a 2.2 implementer who removes the lock leaves `set_enabled` racing the hook thread, or deletes the reset. A 2.3 implementer who follows the roadmap text brings back the "blocked record survives `off`" behaviour that S2 pins, and meets a red S2 with no explanation in the roadmap.
Confidence: CONFIRMED (`roadmap.md:276-285,297`).
What is needed: add 2.2 to the dependents, with a note on what it must move, and add `roadmap.md:297` to the book-sync lines.

### M4. IU widens control to every interactive user of the machine, and the docs describe only a narrowing
Where: D6 "Known trade-off" and RK10 (cut.md:246-250,720); CHANGELOG C5 text (cut.md:664); Q1.
Problem: the default DACL gave non-admin users other than the creator read-only access (`FR` for WD; audit KS-02 verification). `(A;;GRGW;;;IU)` gives read and write, which includes `FILE_CREATE_PIPE_INSTANCE`, to every interactive logon in every session. RK10's justification, "it cannot control the daemon, and it already runs as the user", is false for a second user under fast user switching or RDP. That user can `swch off` or `swch exit` the first user's elevated daemon (RK11 notes this). That user can also create a rogue instance and answer the first user's `swch` falsely, for example "OK: disabled" while remapping stays on. The C5 CHANGELOG entry and the README text read as a pure tightening ("Everyone and anonymous logons no longer get read access").
Consequence: on a shared machine, user B can stop or toggle user A's remapper, and A's `swch` replies can be spoofed, until 4.1. The published docs do not tell the owner.
Confidence: CONFIRMED (SDDL semantics in the plan and the audit's default DACL).
What is needed: no design change (Q1 stands). Correct RK10's reasoning so the orchestrator decides Q1 on accurate facts, and make the KS-02 CHANGELOG/README text say that any interactive user on the machine can control the daemon until 4.1.

## Strengths to keep

- D4's `DaemonControl` trait. `ipc_server.rs` cannot name `KeyboardHook`, T1 runs the real server with no hook, and the second `MAIN_THREAD_ID` disappears.
- D1's re-check under the lock. It closes the race where the fast path passes, then `off` resets, then a Hold is armed. Together with the reset under the same lock, every event racing a toggle is decided deterministically. A key-down waiting on the lock during `on` sees a table seeded before its own async-state update, and a key-up sees itself as held and Passed.
- D2. The Hold generation is kept (KS-11, S5), held keys are seeded as `Passed` (S6), mouse buttons are skipped, and a no-op toggle does not reset (K1b step 3).
- D7. One helper thread and `recv_timeout` bound connect, write and read with a single deadline, with no `unsafe`. Not flushing is the correct choice, because a flush is `FlushFileBuffers` and waits for the reader. Using the named-pipe API with `ConnectWaitMode::Timeout` is correct; I checked that the local-socket connect ignores the wait mode.
- D6. The SDDL has no `O:`/`G:`, because the clone writes the group into the owner slot (verified). BA keeps `FILE_CREATE_PIPE_INSTANCE` for an elevated daemon whatever the logon type.
- D9's red-replay rule. It keeps a tester from adapting `dispatch(On)` tests onto `master`, where they would install a global hook.

## Open questions for the architect

1. Does anything prove that the `S:(ML;;NWNR;;;ME)` label reaches the pipe? P1 proves the parse succeeds (`deserialize -> to_owned_sd -> clone`, where `AddAce` copies the label ACE), and P3 proves the creator's token is admitted. Neither reads the label back, so M2 is the only proof of KS-02. That is acceptable if it is said explicitly. Check: M2, or a read-back of the label on the P3 pipe if you want an automated one.
2. How do the harness steps `remapping_off`/`remapping_on` treat `check_press_symmetry`'s `press_suppressed`? A key suppressed before `off` and released while off is asymmetric on purpose. No current scenario turns symmetry on, but a later property test that mixes off/on would fail spuriously. Settle it by defining the bookkeeping in the C1 harness text, for example by clearing press records at both steps.
3. A factual note, not a finding. The threads table (cut.md:377) and D6's "Cost" (cut.md:258) say the SD is parsed for each pipe instance. It is parsed once in `create_listener`. Later instances borrow the clone stored in the listener's options (`listener.rs:123-125`, `create_instance.rs:51-54`).

## Absent topics (walked, not applicable)

- Section 1, hook budget: the only addition is one Acquire load, placed after `code < 0` and before `lparam` is dereferenced. There is no allocation, I/O or new lock on the callback path.
- Section 3, matching model: untouched.
- Section 7, config and hot reload: the schema and `default_config.ron` are unchanged, and a reload while off leaves the engine alone.
- Section 9: registry and token inheritance do not apply. The single-instance guard (1.5) and the server-side read deadline and bounded workers (4.1) are declared non-goals with owners.
- Section 10, input synthesis: no new `SendInput`, and the sentinel is not involved (the off gate passes injected and physical events alike).
- Section 11: no new dependency, no new public enum without a `Display` arm (`RequestError` has one per variant), and the CHANGELOG and README texts are given commit by commit.
