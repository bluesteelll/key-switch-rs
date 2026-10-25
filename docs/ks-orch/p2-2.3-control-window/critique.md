# Architecture critique: p2-2.3-control-window / roadmap 2.3 (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 8)
Plan: D:/tmp/ks-orch/p2-2.3-control-window/cut.md   Code read at: master `c3dab6a` (clean tree)

The cut is thorough. Its re-location table checks out line by line against the tree: every `path:line` I sampled matches, give or take a line (`scenarios.rs:4981` is `:4982`). The reload-on-the-main-thread design, the window and session Drop-guard order, and the cross-process title read are sound. Two decisions rest on a claim that the code contradicts: the D2 "forgetting never prevents a stuck key" claim, and the D7 "the cell coalesces the two notifications of one unlock" claim. Both can be fixed before implementation, and neither blocks the rest of the cut.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. Rule L swallows the key-up of a key-down that Windows saw and the hook did not; D2 widens this beyond the spec on a false premise
Where: cut §D2 (lines 71-75), §D1 table row "no / Suppressed / not held, Suppressed kept" (line 63), Question 3 (line 362); `src/engine/keys.rs:203-211` (`take_blocked`), `:213-223` (`forget_blocked`), `src/engine/mod.rs:754-760`; spec line 8, `book/src/roadmap.md:487` ("In an elevated daemon, ...").

Problem: D2 removes `forget_blocked` for every daemon. Its "Why", repeated in Question 3, says forgetting "never prevents a stuck key and only turns a key-up that does arrive into a lone key-up". That holds only if every key-down Windows sees also reaches the hook. When a key's key-up is lost and the key is pressed again where the hook cannot see it, Windows has the key down and the engine still has the old `Suppressed` record. The next key-up the hook does see is that unseen press's release. Master forgets the record at the focus change, so this release passes (`take_blocked` returns false for `Forgotten`). Under rule L it is swallowed. The cut's own D2 paragraph names when the hook misses events: UIPI in a non-elevated daemon, and the secure desktop. With D1 keeping `Suppressed` across lost-input resets, the same applies after a reset whose OS read misses the key. The cut concedes that possibility for a reset that runs while Winlogon is still the input desktop (D3).

Consequence: harness script on the template config `CapsLock -> SwitchLanguage` with `check_nothing_stuck()`:
1. `down CapsLock` (suppressed, fires).
2. `foreground_changed`.
3. `up_unseen CapsLock`.
4. `down_unseen CapsLock`.
5. `foreground_changed`.
6. `up CapsLock`.

On master the last key-up passes and nothing is stuck. After C12 it is suppressed, and the check fails with `VK 0x14 is down in Windows but physically up`. In the real world this is a non-elevated daemon (debug `cargo run`):
- Hold CapsLock in Notepad while elevated Task Manager takes focus.
- Release CapsLock there; the hook is not called.
- Press CapsLock again there and keep it held; Windows sees it go down.
- Click back into Notepad and release.

The hook sees that key-up, swallows it, and Windows keeps CapsLock logically down. Every later CapsLock press is swallowed whole, so nothing repairs it. The property tests cannot find this, because the Focus scripts never press a key the hook does not see (`src/engine/properties.rs:34-40`, `:236-237`).

Confidence: CONFIRMED (engine trace on master and after D1/D2; harness steps exist: `testing.rs:312-343`).

Breaks: the suppression invariant (a key-up passes when Windows saw its key-down), R-stuck-key, the spec's scope ("In an elevated daemon").

What is needed:
- Correct the premise in D2 and Question 3.
- Decide the trade with this case on the table: either keep a release path for a key-up whose latest key-down Windows may have seen unseen, or scope D2 as the spec does and document the residual.
- Pin the chosen behaviour with the scenario above, and give it a known-issues row.

Options:
- (a) Keep forgetting in a non-elevated daemon, as the spec words it.
- (b) A witness at the key-up of a key that is not held and has a `Suppressed` record. `Env::os_key_down` of the key being released still reports its pre-event state (`src/engine/env.rs:36-38`): down means Windows saw a key-down the hook missed.
- (c) Accept and document.

### I2. A second (or late) lost-input reset wipes state built after input came back
Where: cut §D7 (line 142: "the cell coalesces the unlock notification and the desktop switch of one unlock into one reset"), §D1 table (line 63), D3 residual (line 81), manual M1 expected result (line 323: "`state reset: session unlocked` and/or `… input desktop switched`"); `src/engine/keys.rs:146-153` (`press` is the repeat detector).

Problem: one unlock produces two notifications, `EVENT_SYSTEM_DESKTOPSWITCH` and `WM_WTSSESSION_CHANGE(WTS_SESSION_UNLOCK)`, and two Ctrl+Alt+Del or UAC round trips produce two switches. The cell merges them only if both are dispatched before the same drain. The cut gives no evidence for that, and its own M1 expects two log lines. Each pending reset runs `reset_transient_state` in full: it clears gestures, cancels macros and clears `active` for swallowed keys (D1). A reset that lands after the user has already acted on the returned desktop therefore treats fresh state as stale.

Consequence: with `( keys: "F13", action: SwitchLanguage )`:
1. `session_unlocked`.
2. `down F13`: fires.
3. `session_unlocked`: the late WTS notification.
4. `+500 repeat F13`.

Step 4 reads as a first press and fires SwitchLanguage again (two layouts: the user ends where they started). That contradicts the owner decision that only `PressKey`/`Volume*` act on auto-repeats. The same timing has two more consequences:
- A `Hold` armed right after the unlock is dropped and re-arms on the next repeat, so it fires late or as the wrong gesture.
- A macro started right after the unlock is cut off half-way.

This is distinct from row 76, which covers a key held through the lock screen.

Confidence: PLAUSIBLE. The mechanism is CONFIRMED by D1 and D7. The gap between the two notifications, and whether a key event can fall between them, is not established. Web search turned up nothing on their order; the WM_WTSSESSION_CHANGE reference ([Microsoft Learn](https://learn.microsoft.com/en-au/windows/win32/termserv/wm-wtssession-change)) does not order them either.

Breaks: owner decision on repeat defaults, KS-25 (one gesture per press), KS-10.

What is needed:
- Either evidence that the two notifications always reach one drain (a timestamped M1, one log line per reason, reported in `impl.md`), or a rule under which a second notification of the same transition, or one that arrives after the hook has seen input on the returned desktop, does not discard post-resume state.
- A scenario pin: `session_unlocked; down F13; session_unlocked; repeat F13 -> nothing fired`.
- D7's coalescing sentence corrected to match.

## Minor (improvements, not blockers)

### M1. D10 makes a waiting Tap fire while an own modifier is down in Windows (known issue 68 grows)
Where: cut §D10 (lines 176-180); `src/engine/gestures.rs:306-321`; `src/core/windows_actions.rs:257-287` (`press_vk` does not touch held modifiers); `book/src/roadmap.md:801`.

Problem: on master an own key's key-down fires the waiting Tap inside the callback, before Windows sees that key. Under D10 the Tap waits for a non-own key-down or for the window timer, by which time the own key is down in Windows.

Consequence: `Ctrl+Shift` `Tap` `PressKey("Escape")` + `DoubleTap`. Tap once, then within the window start Ctrl+C. The Escape is sent ahead of C with LCtrl down, so Windows gets Ctrl+Esc and the Start menu opens. On master a plain Escape is sent at the LCtrl key-down.

Confidence: CONFIRMED (code trace).

Needed: extend known-issues row 68 and §11.4. Optionally add a scenario that asserts `expect_down_in_windows("LCtrl")` at the Tap's fire.

### M2. `nothing_is_stuck_after_win_l` passes with or without the reset
Where: cut C11 pins (line 286); `src/engine/mod.rs:620-627`, `:697-721`.

Problem: without a reset, L is still held for the engine. The next L press reads as an auto-repeat, the `PressKey` winner fires on it (repeat is on by default), and the event passes because L's record is `Passed`. "Nothing fired" at LCtrl/LAlt and "fired" at L both hold on master with the reset removed. The named mutation (keep `active` for swallowed keys) does not touch L, whose key-down passed.

Consequence: the KS-29 pin does not pin the reset.

Confidence: CONFIRMED.

Needed: assert `expect_suppressed()` on `down L`. With the reset, L is a first press that the binding swallows; without it, L passes and Windows gets Ctrl+Alt+L as well as F13.

### M3. `refresh_foreground()` runs before the foreground hook exists
Where: cut §D6 (line 124); `src/core/app.rs:201-211`.

Problem: a foreground change between the refresh and `install()` is never reported. Daemon start right after the UAC consent is the likely moment.

Consequence: every `when:` evaluates against the wrong window, or against no window at all if `GetForegroundWindow` was NULL ("keep the cache"), until the next focus change.

Confidence: PLAUSIBLE.

Needed: fill the cache once the WinEvent hook is live (after `install`, or in the first drain).

### M4. The control window outlives the pump at exit
Where: cut §D6 (line 124), Threads (line 199); `src/core/app.rs:270-282` (`executor.stop()` waits up to 2 s after the loop ends).

Problem: the window is destroyed at function end, after `drop(main)` and `executor.stop()`. For up to 2 s a top-level window exists on a thread that no longer pumps.

Consequence: a program that broadcasts with plain `SendMessage(HWND_BROADCAST, …)` hangs until the daemon's window is gone. A `WM_QUERYENDSESSION` sent then goes unanswered.

Confidence: PLAUSIBLE.

Needed: destroy the window, or at least stop being a broadcast target, before the blocking exit waits, or say why not.

### M5. The control-window test asserts a registration that production tolerates failing
Where: cut C18 (line 295, `has_session_notifications()`), §D6 (line 122: `RPC_S_INVALID_BINDING` is a warning).

Consequence: `cargo test` on a `windows-latest` CI runner, or any machine where Remote Desktop Services are slow, can fail the gate for an environment reason.

Confidence: PLAUSIBLE.

Needed: treat the registration as a precondition, or show that it succeeds on the runner.

### M6. The window procedure's session wiring has no test
Where: cut C18/C19 (lines 295-296).

Problem: only `session_change(wparam)` and `session_ends(wparam)` are tested. The `is_enabled()` gate and the `request_reset` call inside `control_window_proc` are not.

Consequence: a proc that maps the message but never requests the reset passes the suite.

Confidence: CONFIRMED.

Needed: the test that already creates the window can send `WM_WTSSESSION_CHANGE(WTS_SESSION_UNLOCK)` to it. `SendMessageW` to a window of the calling thread is a direct call with no system effect. Then check `take_pending_reset()` with remapping on and off.

### M7. Doc lines the change makes false, not in the touch set
Where: `src/engine/gestures.rs:25-26` ("Identity by keys and `when:` … survives a reload"; a reload now drops every press); `src/engine/mod.rs:31-38` (names `ForegroundContext::capture`, which D9 removes); `src/engine/mod.rs:590-594` ("on the hook thread it may capture the foreground window (KS-08)").

Consequence: stale module docs that review relies on.

Confidence: CONFIRMED.

Needed: add these lines to the touch set (C14 and C17).

### M8. The foreground-race rejection understates the residual
Where: cut §D9 Rejected (line 173: "for a race of microseconds") against Risks (line 351: "the relative order … is undocumented").

Problem: injected input right after a programmatic activation can be decided before the `EVENT_SYSTEM_FOREGROUND` of that activation, against the previous window's exe. Master read `GetForegroundWindow` live, so it decided against the new window. For example, an AutoHotkey script that runs `WinActivate notepad` and then `Send {F9}`.

Consequence: `when: AppEquals("notepad.exe")` F9 is not remapped in Notepad, or fires in the previous app. M3, done by hand at human speed, cannot catch it.

Confidence: PLAUSIBLE.

Needed: state this residual in the known-issues or `when:` page, or measure it under F3. No design change is required this lane.

## Strengths to keep

- **D4.** A reload becomes one main-thread request: swap and reset happen between two events, and `update_bindings` stays the only publisher. The `#[cfg(test)]`-only `ApplyReload for KeyboardHook` means the daemon cannot compile with the old watcher wiring, so the type system checks the wiring.
- **D1.** A reload does not resync keys. The argument (the async-state lag of a key-down that just passed, against no information gained) is right.
- **D6.** The guard order follows Microsoft Learn: unregister the session notification before `DestroyWindow`, and the class last. The window is `!Send`, and is created and dropped in `App::run` on the main thread. `WM_ENDSESSION` removes the hook before the 1 s flush.
- **D7.** The reset never runs inside a callback. The window procedure and the WinEvent callback only set a cell and post `MAIN_WAKE`.
- **D9.** The title read uses the documented cross-process behaviour of `GetWindowTextW`, which sends no `WM_GETTEXT`. The `own` skip removes a latent `WM_GETTEXT` under the borrow that master has. The exe name is keyed by (HWND, pid).
- **Tests.** `check_no_lone_key_up` and the two new property modes. The red-first predictions are concrete and I checked them against the code: C2, C3, C5 and C7 trace exactly as predicted.
- **D11-D13.** Small leftovers, each with a precise pin.

## Open questions for the architect

1. **IME/TSF on the hook thread.** The first top-level window on a thread gets a default IME window and input context ([ImmGetDefaultIMEWnd](https://learn.microsoft.com/en-us/windows/win32/api/imm/nf-imm-immgetdefaultimewnd)). Does IMM/TSF then run any work on the main thread outside `pump`? What settles it: `EnumThreadWindows` on a thread after `ControlWindow::create`, plus R7/R8 run with the control window present (today the bench's loop thread has only a message-only window).
2. **Fallback if `EVENT_SYSTEM_DESKTOPSWITCH` is not delivered for the switch back** (M5 fails). Ctrl+Alt+Del and UAC send no WTS notification, so the switch-away reset (every passed key read as up) would stand alone. NVDA relies on this event from an out-of-context hook, which is weak evidence that it arrives. Name the plan if M5 fails.
3. **A refused foreground update now has a lasting cost** (a stale cache until the next focus change), where today's costs nothing. It is unreachable today, because nothing pumps under the borrow; the R1 sentence "as today" should say so.
4. **A `NoAnswer` reload that lands later logs nothing.** Is a line on the main thread wanted when it lands?
5. **Retry of `WTSRegisterSessionNotification` at autostart** (`RPC_S_INVALID_BINDING`): left to 4.5?

## Absent topics (walked, not applicable)

- §10 input synthesis: no new `SendInput` path, so sentinel discipline is unchanged.
- §9 process boundary: pipe DACL, single-instance guard and token inheritance are untouched. The new window gets only system senders (UIPI considered, F2).
- §7 config: no schema change and RON stays. The semantic changes (a reload drops pending gestures, rule L, D10) are declared with book rows.
- §8 shared state: no new lock, no `static mut`. `desktop_hook` uses Release/AcqRel, as the existing handles do.
- §2 threads: no new thread or COM apartment. Install, uninstall and the window stay on the main thread (KS-01).
- §6 module cycles: `platform::foreground` imports nothing from the crate, and `engine` gains no `hook`/`system` import.
