# Architecture critique: p2-2.1-executor / roadmap 2.1 Action executor (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 5)
Plan: D:/tmp/ks-orch/p2-2.1-executor/cut.md   Code read at: master @ 7d757a1

## Critical (implementation must not start)
None.

## Important (must be resolved; options may be discussed)

### I1. The single executor runs `Window(...)` steps through a synchronous `ShowWindow`, so one unresponsive application blocks every slow action, and an epoch change cannot release it
Where: plan D2 (cut.md:61-65, "one `action-executor` thread"), D5 (cut.md:79, epoch checked only between steps and inside `Delay`), D7 (cut.md:90, `step` -> `sequence::run_step`), Risks "Modal UI" / "Long Delay" (cut.md:348-349), CHANGELOG Breaking text (cut.md:297); `src/data/sequence.rs:273-283` (`ShowWindow(target, SW_MINIMIZE / SW_MAXIMIZE / SW_RESTORE)`).
Problem: today every macro has its own thread (`sequence.rs:116`), so a step that blocks holds up only that macro. Under D2 every `Launch`, `OpenUrl` and `Sequence` shares one thread, and `ShowWindow` on another thread's window waits until that window's thread handles the show event. Microsoft documents this wait for a non-responding target: `ShowWindowAsync` exists "to avoid becoming nonresponsive while waiting for a nonresponsive application to finish processing a show-window event" (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync, Remarks). The cut's risk list names a slow `ShellExecuteW`, a modal dialog and a long `Delay`, but not this case. The new CHANGELOG text also promises that `swch off`, a reload and `swch exit` "stop the running macro within 100 ms", while D5 only checks the epoch between steps.
Consequence: say the user has `( keys: "F13", action: Sequence([ Window(Minimize) ]) )`, or the template's `Ctrl+Alt+N` macro (`Window(Restore)`), or the example's `F15+F16` chord (`Window(Maximize)`), and fires it while the foreground application shows "Not Responding". This is the moment a minimize or close hotkey is most likely to be pressed. The `action-executor` thread then waits inside `ShowWindow` until the application recovers, or for good. Every later `OpenUrl`, `Launch` and macro waits behind it. From the 33rd one on, each is dropped and counted (`executor_jobs_dropped`). `swch off` and saving `config.ron` bump the epoch, but nothing reads it until the step returns. `swch exit` reaches the 2 s `STOP_TIMEOUT` and logs `[WARN] the action executor did not stop within 2 s`. Before this lane only that one macro hung. The lane turns a per-macro hang into a hang of every slow action, and the published "within 100 ms" is false for this case.
Confidence: CONFIRMED (serialisation: plan D2/D7; blocking call: sequence.rs:276-282; the wait: Microsoft Learn above).
Breaks: KS-08's intent (nothing slow holds up the action path); R1 spirit for the shared executor; accuracy of the Breaking entry.
What is needed: the cut has to decide, not leave this open. Either no `Window` step can block the shared executor on a hung target, or the blocking is declared as an accepted cost: in Risks, in the Breaking entry, and in a manual check against a hung window. Separately, the CHANGELOG and README wording of "stops within 100 ms" must say that a step already running (a `Launch`, an `OpenUrl`, or a `Window` step on a hung window) finishes first. D5 already says this for `Launch` and `OpenUrl`.
Options: `ShowWindowAsync` for the three `ShowWindow` arms (same effect, posted; `Close` already uses `PostMessageW`), with a note that `Window(Restore)` then no longer waits for the restore to finish before the next step; or skip the step when `IsHungAppWindow(target)` and say so in the log; or keep `ShowWindow` and document it.

## Minor (improvements, not blockers)

### M1. Nothing proves that the new hook-path functions log nothing
Where: X7 scan list (cut.md:291), acceptance mapping "hook path never calls `log::`" (cut.md:312); new `ExecutorHandle::submit` / `TargetWindow::for_action` in `src/core/executor.rs` (cut.md:163-170) and `KeyboardHook::submit` / `cancel_actions` (cut.md:185-186).
Problem: the X7 test checks the fast actions only. The existing scan (`logging/mod.rs:298-310`) covers only `keyboard_hook_callback.rs`. `ExecutorHandle::submit` runs on the hook thread for every slow fire, and it lives in `executor.rs`, which logs freely on the executor thread.
Consequence: a later edit that adds, say, `log::warn!("action queue full")` on the `Full` arm of `submit` passes the whole suite. With `repeat: true` on a macro and a full queue, the hook thread would then build a `LogLine` and call `try_send` on every auto-repeat, which is the cost D11 and the brief rule out.
Confidence: CONFIRMED (plan text).
What is needed: add `fn submit(` (both files), `fn for_action(` / `fn foreground(` and `fn cancel_actions(` to the `fn_lines` scan of X7.

### M2. Two red-first steps never fail for the stated reason
Where: L7 (cut.md:243, predicted red = E0425 compile error); X3's `the_hook_path_names_no_execute_call` (cut.md:271), whose commit fails to compile as a whole (cut.md:278).
Problem: L7 fixes a call-order bug (`writer::spawn` before `set_logger`, `logging/mod.rs:109-110`), but its red is a missing function, which proves nothing about the order (CLAUDE.md principle 9: "fails for the right reason"). The X3 scan test is the direct proof of the first acceptance line, and the cut predicts it would list `keyboard_hook_callback.rs:219` on master. Inside X3, though, that runtime failure is never observable, because the crate's test target does not compile.
Consequence: the tester and the analyst cannot see either test fail for the right reason, so a scan that matches nothing, or a seam that still starts the writer first, would go green unnoticed.
Confidence: CONFIRMED.
What is needed: introduce `start_logging` with today's order before the red test, so that the red is the panic "a second init must not start a writer". Commit the `.execute()` scan test on its own ahead of X3's compile-red tests (it compiles on master), or require the tester to run it on master and report the listed site.

### M3. Config pages and README places this lane falsifies are missing from the touch set
Where: touch set `known-issues.md` rows 26, 28, 29 (cut.md:222); Q2 list (cut.md:359).
Problem: `book/src/config/known-issues.md:80` (row 36, "Macros have no limit and no cancellation: every trigger starts a new thread, and a `Delay` cannot be interrupted | Open") describes exactly what this lane fixes, but it is not listed. Row 26 ("`Tap` and `DoubleTap` actions ... run inside the hook callback") becomes partly fixed, not fixed, because the fast actions stay in the callback by design. `README.md:415-418` ("arming starts a timer thread that fires the action ... on that thread") is not in the README ranges. `book/src/config/README.md:55` ("macro on a separate thread") and `example.md:189` ("every firing starts a separate macro thread, which cannot be cancelled") and `:207` are in neither the touch set nor Q2.
Consequence: after the merge the config reference still lists the fixed issue as Open, and it still tells readers that macros cannot be cancelled. CLAUDE.md requires the config page of a semantic change to change in the same lane.
Confidence: CONFIRMED (grep of the pages).
What is needed: add row 36 (and the wording of row 26) and README.md:415-418 to X4. Add `config/README.md:55` and `example.md:189`, `:207` either to X4 or to the Q2 hand-off list.

### M4. Once the executor has died, every slow action disappears without trace
Where: D2 (cut.md:61: a foreign `WM_QUIT` or `GetMessageW == -1` ends the executor), D3 (cut.md:68: `Disconnected` -> `Closed`, "silent, not counted"), R10 (cut.md:345), Q7 (cut.md:364).
Problem: `Closed` is designed for shutdown. When the executor dies mid-session, the handle stays attached, so each later fire is a `Closed` that nothing counts.
Consequence: a shell extension on the executor thread calls `PostQuitMessage`, for example from its hidden window's `WM_DESTROY`. One `[ERROR]` line is logged. From then on, pressing an `OpenUrl` or `Launch` hotkey does nothing, no counter moves, and `swch status` says `running`.
Confidence: PLAUSIBLE (the trigger is foreign code; the silent path is CONFIRMED in the plan).
What is needed: a mid-session `Closed` must be visible, for example as a counter or in `swch status`. Alternatively, decide Q7 the other way: whether retrieving `WM_QUIT` clears the quit state, so that ignoring it cannot spin, can be checked once on Microsoft Learn or with a test that calls `PostQuitMessage` on a scratch thread.

### M5. The Hold half of the split has no automated proof, although one is possible without a live hook
Where: acceptance mapping "Hold on its `hold-timer` follows the split" (cut.md:315, "review" + M6).
Problem: the Hold path is the one place where the old code ran actions on a second thread (`keyboard_hook_callback.rs:293-298`). The cut proves the split only for `carry_out` called on the test thread.
Consequence: a regression in which `start_hold_timer` runs the decision by some other route than `carry_out(get_hook(), ..)` (for example a leftover direct run of the action) is caught only by the manual M6.
Confidence: CONFIRMED (plan text).
What is needed: one test under `GLOBAL_HOOK`. Attach a FakeRunner executor to `get_hook()`, bind `( keys: "F13", action: OpenUrl(..), trigger: Hold(1), allow_extra_modifiers: true )`, call `call_keyboard_callback(F13, WM_KEYDOWN)`, and wait until the runner records the `OpenUrl` on `action-executor`. Then send the key-up and detach. Nothing is injected: the mask is only counted under test, and the action goes to the fake runner.

## Strengths to keep
- D1: the `FastAction` mirror enum with an exhaustive `as_fast` makes "a slow action inline" impossible to write, and it answers the `rg "\.execute\(\)"` acceptance by construction.
- D2/D3: send first, then post the wake, and drain before every wait. A wake lost in a modal loop costs no job. `WM_QUIT` is rightly not the stop signal. A bounded `sync_channel` used through `try_send` and `try_recv` only adds no allocation and no lock on the hook path. The waker argument checks out against std's `SyncWaker::notify`.
- D6: `Instant` decides, the wait is only an upper bound, `checked_add` handles `Delay(u64::MAX)`, and the wait pumps messages as an STA must.
- D8: stop strictly after `uninstall`, with the right reason (a macro's `SendInput` waiting on an unpumped hook thread); the join is bounded; the guard covers every `?`.
- Tests use real executor threads with a FakeRunner. Cancellation and `Delay` are the real code under test. The one real-Win32 test is `#[ignore = "manual: ..."]`.
- R-7 (a wake posted to a thread id after the executor ended) is declared honestly and left to 2.2.

## Open questions for the architect
1. Spec deviation not listed in Questions: the brief says exit "joins the thread before the final logging::flush". D8 detaches after 2 s. The reason is sound, but it belongs next to Q1 as a declared deviation.
2. The L2 example "`( keys: "LWin", action: PressKey("L"), trigger: Hold(500) )` sends Win+L (locks the session)" is unverified. It is uncertain whether injected `L` with a physical Win down locks the workstation. Ask the owner to check it by hand before the claim is published, or keep only the Ctrl+Esc example. Both RON snippets also need the `cfgcheck` validation CLAUDE.md requires for published examples.
3. CI timing: the 5 ms enqueue bound with one retry batch (X1-4, X3) runs in parallel with CPU-bound scenario tests. If CI shows a flake, the fallback should already be decided (for example, the bound applied to the median, with an assertion that no enqueue waited for the 500 ms runner), rather than raising the bound ad hoc.

## Absent topics (walked, not applicable)
- Matching model, suppression lifetime, gesture state (sections 3-5): the engine is untouched; `LRESULT(1)` iff `suppress` and the sentinel check before any state are unchanged (`keyboard_hook_callback.rs:180`, `:343-353`).
- Config schema and hot reload (section 7): no RON change. The reload path only adds the epoch bump after the single sorted store.
- Process boundary (section 9): pipe, DACL, registry and elevation unchanged (KS-09 stays with 3.3, declared).
- Input synthesis (section 10): no new `SendInput` builder. Every step keeps `injected_sentinel()` (`sequence.rs:175`, `:241`).
- Module cycles (section 6): `engine` gains no dependency. `core::executor` -> `data::sequence` follows the existing core<->data direction.
