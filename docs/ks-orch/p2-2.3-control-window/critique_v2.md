# Architecture critique: p2-2.3-control-window / roadmap 2.3 (round 2)
Verdict: APPROVED (Critical 0, Important 0, Minor 4)
Plan: D:/tmp/ks-orch/p2-2.3-control-window/cut.md (revision 2)   Rulings: rulings.md (2026-10-19)   Code read at: master `c3dab6a`

I read the whole revised cut again, not only the changed sections. I re-traced every red-first prediction that touches R1 and R2 against the code. I also checked every new Win32 claim against Microsoft Learn (quotes below). Both rulings are carried out. The witness (D15) and the "only input leaving resets" rule (D7/D16) are sound, and I found no path that leaves a key down in Windows that the plan does not declare. What remains is four Minors:
- two manual-check or claim wordings that would mislead the tester;
- one timing assumption behind `WTS_SESSION_LOCK`;
- one exit-order slip that puts an RPC in front of `uninstall`.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

None.

## Minor (improvements, not blockers)

### m1. Manual check M1 expects exactly one `state reset` line, but D7 logs one per drain, and a Win+L normally takes two drains
Where: cut §Manual checks M1 (line 815: "exactly one `state reset: keyboard input left this desktop` before the unlock"); §D7 drain step 4 (line 296), "Nothing relies on coalescing" (line 306); `src/hook/main_thread.rs:548-553` (`pump` returns to the drain at the first `MAIN_WAKE`), `:484-499` (`drain`).

Problem: two signals answer a Win+L. `WTS_SESSION_LOCK` comes from Remote Desktop Services as a window message, and `EVENT_SYSTEM_DESKTOPSWITCH` comes from a queued WinEvent. Each one notes its flag and posts `MAIN_WAKE`, and `pump` goes back to the drain at the first wake. Two notifications that do not reach the queue together therefore give two drains, and D7 step 4 logs one `state reset` line per drain. Also, each signal line carries the drain's time, in `InputSignal::ALL` order, not in the order the signals arrived.

Consequence: the tester runs M1 with the release daemon and sees `session locked`, `state reset: …`, `keyboard input switched to another desktop`, `state reset: …`. M1 is then reported failed although the behaviour is the designed one. Or the "exactly one" is read as evidence that the two signals coalesce, the very claim round 1 withdrew. Either way, the R2 evidence the ruling asked M1 to collect gets misread.

Confidence: CONFIRMED (plan M1 against D7; the pump and drain code).

Breaks: ruling R2's last bullet (M1 is meant to be the evidence on order and timing).

What is needed:
- M1's pass criterion has to follow D7's per-drain logging. For example, the R2 invariant itself: every `state reset` line comes before the first return-signal line, there are one or two of them, and none comes after.
- M1 should say what order evidence the log can and cannot give, since lines from one drain come out in `ALL` order.

### m2. `WTS_SESSION_LOCK` is taken "by its kind", and the claim that a lock noted after the unlock needs a full main-thread stall leaves out the sender's own latency
Where: cut §D7 Why (line 308: "The other order, a lock noted after the unlock, needs the main thread stalled through the whole lock. Keys typed after the unlock would wait on that same stall."); §Risks "Late or repeated input signals" (line 850); `InputSignal::reset` (line 265: `SessionLocked => Some(InputLeft)`).

Problem: the argument assumes that any delay before the lock is noted comes from this thread. Two facts cut against that:
- The notification is sent by Remote Desktop Services: "Session change notifications are sent in the form of a WM_WTSSESSION_CHANGE message" ([WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification)).
- The desktop switch reaches the thread on a different channel, the WinEvent queue: "The system queues the event notifications so that they arrive asynchronously" ([Out-of-Context Hook Functions](https://learn.microsoft.com/en-us/windows/win32/winauto/out-of-context-hook-functions)).

Nothing documents that the service sends the lock before the user can unlock. A search found no report of such lag, and none of its absence either. A switch event is classified by `UOI_IO` when it is delivered, so a late switch event can never discard post-return state. The lock is the one input-left signal that is not checked against where input is now.

Consequence: suppose the service delivers the lock after a short lock, for example a Windows Hello face unlock about 1-2 s after Win+L, at a moment when the user has already pressed F13 (`SwitchLanguage`, fired) and holds it. The late `SessionLocked` runs `release_all`, and the next auto-repeat of F13 reads as a press and switches the layout again. This is round 1's I2 consequence through the one signal that is still unchecked. The same timing would drop a Hold armed right after the unlock and stop a macro started then. This is not a stuck key.

Confidence: PLAUSIBLE. The mechanism is CONFIRMED in D7. The sender-side delay is not established either way.

Breaks: ruling R2's invariant ("Anything the hook saw or started after input came back stays").

What is needed:
- Correct the D7 sentence so that it names sender latency as an unverified assumption.
- Then either let M1's timestamps decide whether the lock can arrive after the switch back, with the residual added to row 79 if it can, or classify the lock against `UOI_IO` too. The second option rests on the switch away, which M1 also checks, to answer a lock noted early.
- The architect picks which.

### m3. "The UAC prompt is answered through the switch away" holds only while UAC uses the secure desktop
Where: cut §D3 (line 145), known-issues row 63 rewrite (line 580: "Fixed for a lock, a Ctrl+Alt+Del screen and a UAC prompt"), CHANGELOG C21 Added and Fixed (lines 787-788), manual check M5 (line 819).

Problem: the Desktops quote D3 relies on describes the default configuration. With the policy "User Account Control: Switch to the secure desktop when prompting for elevation" disabled, "all elevation requests go to the interactive user's desktop" ([policy reference](https://learn.microsoft.com/fr-fr/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-switch-to-the-secure-desktop-when-prompting-for-elevation); a UIAccess helper such as Quick Assist can also turn it off). Then there is no desktop switch and no session notification, so no input-left signal at all. The consent window runs at System integrity, so even an elevated daemon's low-level hook very likely does not see keys typed into it (UIPI), and a key released there loses its key-up with nothing to answer it.

Consequence: on a machine with that policy off, hold CapsLock (swallowed, layout switched), let a program request elevation, release CapsLock while the prompt has focus, then answer the prompt. The next CapsLock press reads as an auto-repeat, and `SwitchLanguage` does not fire. That is KS-29, against the owner decision of 2026-10-18 for this case. It is the same as master, so it is no regression. Separately, M5 run on such a machine shows no `switched to another desktop` line, and the tester reports a failure.

Confidence: PLAUSIBLE. That no desktop switch happens is CONFIRMED by the policy text. That the hook misses those key-ups (UIPI on a low-level hook against a System-integrity window) was not checked here.

Breaks: the accuracy of the D3, row 63 and CHANGELOG claims; manual check M5.

What is needed:
- Qualify the claim to the UAC prompt on the secure desktop, the default.
- Put the other configuration in row 79 or row 63's remainder.
- Give M5 its precondition. The policy value can be read read-only, `PromptOnSecureDesktop` under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`.

### m4. Dropping the control window before `hook.uninstall()` puts an RPC between the end of the loop and `uninstall` while the keyboard hook is live
Where: cut §D6 `App::run` step 3 (line 236: "`drop(control_window)` as the first statement, before `drop(raised)`, `drop(main)`, `hook.uninstall()` and `executor.stop()`"); Threads table (line 485); Risks R10 (line 845); `src/core/app.rs:270-278`.

Problem: after `main.run` returns, nothing pumps, but the `WH_KEYBOARD_LL` hook stays installed until `hook.uninstall()` (`app.rs:277`). Today only non-blocking work runs in that gap: `drop(raised)`, `drop(main)` and a queued log line. The cut puts the `ControlWindow` drop there:
- `WTSUnRegisterSessionNotification` is a call into the session service. Its register counterpart is documented to fail with `RPC_S_INVALID_BINDING`, which points to an RPC channel.
- `DestroyWindow` of the thread's last window also tears down the default IME window, and possibly TSF's window.

Ruling M4 asks only that the window be gone before the executor stop and its bounded 2 s wait. It does not ask for it before `uninstall`.

Consequence: on `swch exit` or Ctrl+C, if the session service is slow to answer, every key pressed on the desktop in that interval waits for this unpumped hook, up to `LowLevelHooksTimeout` (at most 1 s on 1709+), and the hook may be removed silently. That is bounded and happens only at exit, but it is the "nothing that can block between the loop's end and `uninstall` while the hook is live" rule (critic §2, KS-08 shape). The `WM_ENDSESSION` path is not affected, because `end_session` uninstalls first. The early-return paths are not affected either: the window drops after the explicit `uninstall` at `app.rs:224`, or after `install` has already removed its hooks.

Confidence: PLAUSIBLE. That the unregister call can block for long is not measured. The order is CONFIRMED in the plan.

Breaks: the shutdown discipline of section 2 and R10. M4 is still met by the alternative.

What is needed: keep M4's bound, with the window gone before `executor.stop()`, but drop it after `hook.uninstall()`. Or state why it must precede `uninstall`. Update the Threads row and R10 to match.

## Previous findings

| Round 1 | Status | Reason |
|---|---|---|
| I1 rule L swallows the key-up of an unseen press; D2 premise | RESOLVED | D15 adds the witness, short-circuited after `swallow`, so a passing key-up asks nothing. The D2 premise is corrected (line 136), and Question 3 is withdrawn. C12 (a) is red on master: I traced it with `testing.rs:540-543` and `:641-643` and got exactly `VK 0x14 is down in Windows but physically up`. C12 (b) is round 1's script verbatim, green on master through `Forgotten` (`keys.rs:203-211`). C12 (c) is green on master, because `reset_from_os` clears the record. C12 (d) is red with `left: 0`, since nothing else asks the OS about F13 (`sync_modifiers` asks only `MODIFIERS`, `keys.rs:25-37`). `Mode::Unseen`, row 78, M12 and M13 are added. |
| I2 a late or second reset wipes post-return state | RESOLVED (residual m2) | Only input-left signals reset. Switch events are classified by `UOI_IO` when they are delivered, so a late switch away reads as a return and a late return during a new episode reads as a departure, which that episode justifies. `release_all` reads nothing, which makes a repeated left signal idempotent. The ruling's pins are present, and each has a mutation that I traced red. The lock-by-kind timing is m2. |
| M1 D10 Ctrl+Esc residual | RESOLVED | D10 Cost, row 68, C7 (3) with `expect_down_in_windows("LCtrl")`. |
| M2 `nothing_is_stuck_after_win_l` does not pin the reset | RESOLVED | It is now red on today's reset. Traced: `reset_from_os` reads L as held, and at `down L` the PressKey winner fires on the "repeat" while the `Passed` record passes the event (`mod.rs:622-627`, `:715-718`), so `expect_suppressed` fails as predicted. M2's mutation (`SessionLocked.reset()` returns `None`) gives the same red. |
| M3 refresh before the foreground hook | RESOLVED | Refresh right after `install`. The foreground events queued before the refresh are processed in order afterwards, and the last one names the current window, so the end state is right. |
| M4 window outlives the pump | RESOLVED (new m4 on the order relative to `uninstall`) | Created after the guard, dropped before `executor.stop()`. Rust drops locals in reverse order, so the early returns are covered. |
| M5 registration asserted | RESOLVED | `Registered` or `Unavailable`, both named. The real `UOI_IO` read is only an ignored test (M14). |
| M6 window procedure untested | RESOLVED | Direct calls for lock, unlock, logon, remapping off, `WM_QUERYENDSESSION` and `WM_ENDSESSION(0/1)`. |
| M7 stale docs | RESOLVED | Lines are in the touch set. |
| M8 foreground-race residual | RESOLVED | D9 states it exactly. Row 80, F3. |
| OQ1-OQ5 | ANSWERED | OQ1 has R13 plus a Learn quote (see open question 1). OQ2 is covered by F5. OQ3 is in Risks R1. OQ4 is adopted in C16. OQ5 goes to F6. |

## Rulings carried out

- **R1.** The witness follows the ruling, applied at every key-up the engine would swallow. That includes the gesture disjunct `gesture_bound && !was_passed`, which only widens the protection. The Learn note is quoted correctly ("the callback function is called before the asynchronous state of the key is updated", [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)). The doc's next sentence ("Consequently, the asynchronous state of the key cannot be determined by calling GetAsyncKeyState from within the callback") is about the new state, so the inference to the pre-event state stands. The cost is bounded by R11. The D2 scope is kept, with its premise corrected. The red-first script is C12 (b), and the lock variant is in C14. The property mode is `Unseen`, and the residual is row 78.
- **R2.** The mechanism is chosen and cited (`UOI_IO`, `WM_WTSSESSION_CHANGE`). The coalescing sentence is removed. Every ruling pin is met at least in intent. The literal "`desktop_switched_back; down; desktop_switched_back; repeat`" is not written out, but `DesktopSwitchedBack.reset()` returning `Some` is caught by C14's part (2) and by the Hold pin. M1 gets timestamped lines (see m1).
- **Minor rulings M1-M8, and Q1-Q8 of round 1.** All carried out. The bench renumbering from C15 to C11 is explained. The new Question 1, a fourth `windows` feature, checks out: `GetThreadDesktop` is at `windows-0.62.2/.../StationsAndDesktops/mod.rs:128`, `GetUserObjectInformationW` at `:139` and `UOI_IO` at `:377`.

## Walk of the focus areas

**R1, the witness.**
- *Own output.* All six `SendInput` sites send their downs and ups in one call (verified: `system_function.rs:125-159`, `windows_actions.rs:257-288`, `sequence.rs:123-212`, `keyboard_hook_callback.rs:258-283`). No `SequenceStep` sends a lone key-down (`sequence.rs:25-61`). The witness read happens in `decide_event`, before `carry_out` injects anything. So our output cannot be the key-down the witness sees.
- *Another injector.* The hook sees its events, and the engine records them as presses or repeats. The residual for one it cannot see is declared in row 78.
- *Keys held across a desktop switch.* `release_all` keeps the records. A key pressed again on the secure desktop or behind UIPI leaves the global async state down, the witness reads it, and the key-up passes; that this state is visible from the user's desktop is what M12 and M13 check. Swallowing a key-up while Windows has the key down needs a false zero from `GetAsyncKeyState`. All three documented zero cases ([GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate), quoted correctly in the cut) are excluded at a key-up the hook sees, except a foreground switch in the microseconds between the event and the read.
- *Sided keys.* The hook reports sided modifier codes. The read uses the same code, and `GetAsyncKeyState` distinguishes the sides (see open question 2).
- *Key table after every reset reason:*
  - `InputLeft`: no active flag survives, every record survives, and the next key event's `sync_modifiers` (`mod.rs:423`) restores the passed modifiers that are really held.
  - `RemappingSwitched`: the OS is the witness for passed keys, and a kept `Suppressed` record is overwritten by the key's next key-down or caught by D15 at a key-up after an unseen press.
  - `Reloaded`: nothing changes.

  None of the three leaves a key down in Windows or sends a lone key-up, except row 59 (released while remapping is off) and the declared residuals.
- *A side benefit.* A key-down that Windows passed because the hook timed out leaves a `Suppressed` record. The witness now lets its key-up through instead of sticking the key.

**R2, order and timing.**
- *Unlock and the switch back, in either order and across drains.* Neither resets anything.
- *Two Ctrl+Alt+Del or UAC round trips.* Each departure resets. State made between a return and the next departure is cleared by a departure it really precedes.
- *A real second lock.* It starts a new episode with a new departure signal.
- *Missed resets.* Row 79 (a stall, or a departure classified after the return), F5, m3 (UAC on the interactive desktop), and remapping off, where `swch on` resyncs anyway.
- *Discarded post-return state.* Only m2. The gap between a callback noting a departure and the drain carrying it out is bounded by the pump reaching the posted `MAIN_WAKE` (`main_thread.rs:548-553`), and no human return fits inside it.

**Red-first.** Besides the traces above, these fail for the stated reason:
- `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press`;
- the C14 flips: forgetting at `keys.rs:213-223`, `reset_from_os` at `:122-135`;
- `FocusWhole` and `Unlock`, red on C14 through forgetting and today's reset;
- `press_symmetry_check_reports_a_repeat_after_a_lock_that_differs_from_its_key_down`.

  These stay green under the witness as claimed: the `down_unseen` scenarios at `scenarios.rs:2571`, `:2765-2768`, `:4612` and `:5002-5003`, and the existing `remapping_on` scenarios (`:2184`, `:3167`, `:3190`, `:3214`, `:3234`, `:3253`, `:3276`, `:3661`, `:4623`, `:4876`).

## Strengths to keep

- D15's shape: one read, only where the engine would swallow, no allocation, decided in the pure core and modelled exactly by `FakeEnv`.
- `release_all` with records kept. It makes repeated departure signals idempotent and turns an early `WTS_SESSION_LOCK` into a harmless reset, where an OS read would have hit the early-lock race that C14 pins.
- Classifying switch events at delivery time with `UOI_IO` of the thread's own desktop. It needs no access to Winlogon. The quotes are verified: [Desktops](https://learn.microsoft.com/en-us/windows/win32/winstation/desktops), [GetUserObjectInformationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getuserobjectinformationw), [GetThreadDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getthreaddesktop) ("You do not need to call the CloseDesktop function"), [UnhookWinEvent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwinevent).
- The pure `InputSignal::reset`, which turns the R2 policy into a one-line mutation for every pin.
- The C11 baseline, placed before both the witness and the cache.

## Open questions for the architect

1. **IME and TSF windows after the control window is dropped.** Creating the thread's first window also creates a default IME window. Wine's user32 tears it down with the thread's last window ([wine-cvs 2022-07](https://www.winehq.org/pipermail/wine-cvs/2022-July/167050.html)), which suggests Windows does the same, but Learn does not say so. A one-line check settles it: in `the_control_window_is_a_hidden_top_level_window_released_on_drop`, assert that `EnumThreadWindows(GetCurrentThreadId(), ..)` finds no window after the drop. That proves M4 for the whole thread, not only for the control window.
2. **Generic modifier codes at the witness.** If the low-level hook ever reports `VK_SHIFT`, `VK_CONTROL` or `VK_MENU` (an injector sending `wVk = VK_SHIFT`), `GetAsyncKeyState` answers for either side. A swallowed injected Shift key-up would then pass while the user physically holds LShift. The harness assumes that a real keyboard never reports an unsided code. Is that true for injected input too? Settle it with a recorded `KBDLLHOOKSTRUCT`, or say so in row 78.
3. **The window class shared by the bench and the window test.** Bench row R13 and the window test both register `key-switch-rs control` in one test process. Under `--include-ignored` one of them gets `ERROR_CLASS_ALREADY_EXISTS` unless the bench takes the same test mutex.

## Absent topics (walked, not applicable)

- §10 input synthesis: there is no new `SendInput` path, and D15 relies on the existing atomic batches, verified above.
- §9 process boundary: the pipe, the single instance and token inheritance are untouched. The window gets only system senders.
- §8 shared state: no new lock and no `static mut`. `desktop_hook` uses `Release` and `AcqRel`, and `PENDING_SIGNALS` is a main-thread `Cell`.
- §7 config: no schema change, and RON stays. The semantic changes are declared, with book rows 76-80.
- §6 module cycles: `platform::{foreground, desktop}` import nothing from the crate, and the engine gains no `hook` or `system` import.
- §11 builder style and Display: no setters. `ResetReason` and `InputSignal` get their `Display` arms in the commits that add their variants.
