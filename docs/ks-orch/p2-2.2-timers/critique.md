# Architecture critique: p2-2.2-timers / roadmap 2.2 Timers on the main thread (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 8)
Plan: D:/tmp/ks-orch/p2-2.2-timers/cut.md   Code read at: master c9c1373 (clean)

Read in full: spec.md, cut.md; src/hook/{keyboard_hook.rs, keyboard_hook_callback.rs}, src/core/app.rs,
src/core/executor/mod.rs (wait, pump, plan, delay_lateness), src/platform/timers.rs, src/engine/{mod.rs,
gestures.rs, winner.rs, chords.rs, sequences.rs, testing.rs, env.rs, keys.rs}, the scenarios the cut names,
src/logging/mod.rs scan tests, src/core/windows_actions.rs, roadmap.md 2.2 and open questions 5/7/10,
known-issues.md rows 23/24/49/66-74, gaps.md Tap-vs-DoubleTap, methodology.md repro crate, 2.1 results_v3 m-1.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. D6 cancels a Hold whose term has already passed when its timer is only late, so the outcome depends on wake order, not on the clock
Where: cut.md:149-161 (D6 table, row `Held` + "key-down passed" -> cancel), cut.md:84-87 (D3 drain/pump), cut.md:351 ("Timer and input ready together ... Both orders are correct ... the engine decides by Instant"); src/engine/gestures.rs:156-169 and src/engine/mod.rs:72-78 (today: an overdue Hold is left to its late timer), src/engine/env.rs:23-30 ("events and timers are thus decided in the order of this one clock"), src/engine/testing.rs:425-434 (`advance_ms_stalled`).
Problem: D5 turns a Held press into a `DueHold` stub only at the key-up (cut.md:137). D6's `other_key_down` treats every `Held` press of a passing key as pending and drops it, without asking whether `now >= hold.due`. A Hold timer is serviced only in the drain (D3 step 1), while hook callbacks run inside `PeekMessageW` in the pump (step 4), which loops until the queue is empty. A key-down dispatched in the same pump pass as, or before the drain of, a timer that is already due is therefore decided first and cancels a Hold whose term has run out. The cut's own risk note claims both orders are correct "by Instant"; that holds for key-ups (DueHold) but not for D6 key-downs.
Consequence: `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )` (passing by default, the cut's M4). Hold Ctrl+Shift for 1000 ms, then press Right at about 1000.5 ms. If the main thread is awake and pumping at that moment, or is a few ms late under CPU load (the very case D9 is about), the Right key-down is decided before the due timer and the layout does not switch. When the machine is idle, the same keystrokes switch it. In the harness: `.down("LCtrl").down("LShift").advance_ms_stalled(1001).down("Right")` drops the Hold, and the same script with `advance_ms(1001)` fires it. The engine then gives different results for one input timeline, which the existing late-timer scenarios (`equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`, scenarios.rs:503-532) exist to rule out.
Confidence: CONFIRMED (traced: D6 table vs D5 key-up rule vs D3 loop order; the harness already models the stall).
Breaks: R7 / determinism of the core (critic section 5: a sleeper must not decide by wake order); the cut's own risk claim at cut.md:351.
What is needed: in `other_key_down`, decide by the clock. A Held press whose Hold term has passed (`now >= hold.due`) counts as resolved and is kept for its timer, like `DueHold`, whichever is drained first. Pin it with a C10 scenario that uses `advance_ms_stalled` past the term, then another key-down, then `advance_ms`, and expects the Hold to fire exactly once.

## Minor (improvements, not blockers)

### M1. The C7 test-migration list misses tests that break when the engine leaves `KeyboardHook`
Where: cut.md:270; src/hook/keyboard_hook_callback.rs:492-518 (`physical_key_events_reach_the_engine`) and :520-541 (`injected_and_unknown_events_bypass_the_engine`): both `.expect(..)` a `Some` from `decide_event`, which returns `None` once `with_main` finds no state on the test thread; src/hook/keyboard_hook.rs:592-610 (`a_repeated_swch_on_keeps_the_running_macro`, calls the removed `set_enabled`); src/core/app.rs:365-372 (`daemon_handle_refuses_exit_without_a_main_thread` builds `DaemonHandle { main_thread_id: 0 }`).
Consequence: C7 is not green as declared, and under the gate the developer may drop or weaken these asserts to get green, which "no assertion weakened" forbids.
Confidence: CONFIRMED.
Direction: list every test that reaches the engine, `set_enabled` or `DaemonHandle`, with its successor.

### M2. A Tap that waits for the DoubleTap window fires into whatever has focus when the window closes, and after the key-ups of other keys
Where: cut.md:137-145 (D5), :158 ("key-ups of other keys change nothing"), :352 ("Foreground change: unchanged semantics"); src/core/windows_actions.rs:271-283 and src/system/system_function.rs:45-80 (fast actions act on the current foreground).
Consequence: with `CapsLock Tap(200) -> SwitchLanguage` plus `CapsLock DoubleTap(300)`, a tap followed by a mouse click into another window within 300 ms switches the layout of the new window, and a `when: AppEquals(..)` is not checked again. With `Shift+F9 Tap -> PressKey("A")` plus a DoubleTap on `Shift+F9`, releasing Shift inside the window sends `a`, not `A` (fast output combines with the modifiers Windows holds, known issue 68 / KS-19). The cut declares only the timing cost.
Confidence: CONFIRMED from code reading (not observed).
Direction: declare both in the new known-issues row 75 and in triggers-conditions.md §11.6, or flush the waiting Tap on more events and say which.

### M3. The new row 67 text must say that only key-downs cancel, and mouse clicks do not
Where: cut.md:148-161, :240.
Consequence: with M3's `( keys: "LCtrl", action: PressKey("Escape"), trigger: Tap(200) )`, a quick Ctrl+click in Explorer still sends Escape (the hook sees no mouse button). Users reading "fixed" will expect AutoHotkey's behaviour, which also counts mouse buttons when a mouse hook exists.
Confidence: CONFIRMED (keyboard hook only, src/hook/keyboard_hook.rs:209-261).
Direction: keep row 67 as "fixed for key-downs" and add the mouse case as a residual line.

### M4. Edge behaviour of the request channel is unspecified
Where: cut.md:76-80 (D2), :349 (R10).
Problems and consequences:
(a) The capacity and send mode of the `reply` channel are not fixed. The main thread must never block on it. Use capacity 1 and `try_send`, or the equivalent.
(b) A request whose worker gave up after 2 s (`NoAnswer`) stays queued and runs later. `swch off` prints "did not answer" and remapping then turns off anyway. An `off` that timed out followed by `on` runs both, with two resets.
(c) Between `run` returning and `MainThread` dropping (the executor stop takes up to 2 s), requests are still accepted and each waits the full 2 s. They should fail at once, as the cut intends.
Confidence: PLAUSIBLE ((a) depends on the implementation); CONFIRMED for (b) and (c) from the cut's text.
Direction: drop or skip a request whose requester stopped waiting, and close the receiver when `run` returns.

### M5. R8 cannot isolate what D9 claims, and its landing rule passes on noise
Where: cut.md:174, :192, :291.
Problem: R8 times the whole `SendMessageTimeoutW` round trip from a normal-priority bench thread while every core is busy. In the daemon the sender is the system's input thread, which is not starved. In R8 the sender's own rescheduling after the reply is part of both arms, and the rule `p99(HIGHEST) <= p99(NORMAL)` on a single run of 300 samples lands C15/C16 whenever the two arms tie.
Consequence: whether C15/C16 land becomes close to a coin flip, so the "evidence" line in impl.md would not support either answer. On the desktop no harm follows either way, since HIGHEST is base 10.
Confidence: PLAUSIBLE.
Direction: measure one-way delivery at the receiver (a timestamp taken on entry to the window procedure, against one taken before the send), or raise the sender to stand in for the input thread. Run it 3 times like the other rows. Land only on a difference larger than the run-to-run spread, or state that the Microsoft Learn guidance alone decides.

### M6. Two D5 transitions are not defined
Where: cut.md:102-103 (`forget_timer` "generalised"), :133-139 (D5 table).
(a) A refused window timer of an `AwaitSecond { deferred_tap: Some }`: is the Tap dropped, or does it wait for the next key-down? Today's model (KS-33) drops only a refused Hold.
(b) `press(k, group)` when k already has a `Held` press: a passing modifier whose key-up was lost and then resynced as up by `sync_modifiers` (src/engine/keys.rs:148-160; mod.rs:356), with no other key-down in between, which would have cancelled it under D6.
Consequence: if the implementation copies `arm_tap`'s "first one wins" rule (gestures.rs:91-112), the stale entry answers the next key-up, and one press of that key fires nothing.
Confidence: PLAUSIBLE.
Direction: add both rows to the D5 table: replace a stale Held press, and decide what a refused window timer does with its Tap.

### M7. State the action rule for every test that carries out a decision
Where: cut.md:262-268 (C6), :188-190 (R4-R6).
Problem: under `cfg(test)` only `send_menu_mask` is stubbed (keyboard_hook_callback.rs:288-294). `FastAction::run` sends real `SendInput` (windows_actions.rs:232-266). `remapping_switches_through_the_main_thread_and_drops_pending_timers` and `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` run the real loop or the extern callback, and their actions are not named.
Consequence: if one of them used `PressKey` and its "never fires" assertion failed, the run would type into the owner's foreground window. That breaks the safety rule.
Confidence: CONFIRMED (no stub exists).
Direction: the cut should require `DoNothing`, or a slow action sent to a `FakeRunner`, in every test that reaches `carry_out`, as the module comment at keyboard_hook_callback.rs:433-442 already says for today's tests.

### M8. The real-timer acceptance test needs its reference instant taken before `decide_event`
Where: cut.md:264.
Consequence: the engine's `now` is read inside `decide_event`. If the test reads `t_down` after the call returns, `record.at - t_down - 200 ms` can come out a few microseconds below 0, and "every lateness >= 0" fails without a defect. A flake like that goes to the orchestrator under the 2.1 rule.
Confidence: PLAUSIBLE (the cut does not say where `t_down` is read).
Direction: read `t_down` before the call, or compare against the `TimerRequest::due` the engine produced.

## Previous findings
Round 1: none.

## Strengths to keep
- D1/D3 put hook ownership, the engine and the gesture timer on one thread, with `MainGuard: !Send` and `WaitableTimer: !Send`. `with_main` uses only `try_borrow_mut`, the form roadmap 3.4 prescribes (roadmap.md:564). Re-entry can neither panic nor deadlock, and no lock is left to invert under D9.
- The waitable timer is set inside `GestureTimers::start`, so correctness does not depend on whether a sent message can run inside the kernel wait. `Instant` decides, and a stale or early signal costs one drain. This copies the 2.1 executor shape, which has been measured (executor/mod.rs:442-499).
- The MWMO pin test can really fail. I traced it: without `MWMO_INPUTAVAILABLE`, the pump's `PeekMessageW` takes `MAIN_WAKE` and clears the post bit, `WM_CLOSE` stops counting as new input, and the wait blocks until the watchdog. That closes 2.1's m-1 for the main loop.
- D2 identifies the actual race correctly: a flip on the worker followed by a posted reset lets a sent hook call decide on stale state. Moving the flip and the reset onto the main thread is the right cut.
- D5's exclusive per-press machine, with a new seq at `AwaitSecond`, removes KS-25 for every term pair without firing a Hold from a key-up (known issue 68 ordering kept). Using the group, keys plus `when:`, as the identity survives reloads.
- The red-first predictions I traced against master are correct: C5 (`:91`, `:312`), C8 (all six messages), C10 (three messages), C12. C2's honest "green on master, red on the 7cb3521 repro replay" fits the spec's escape clause, and the repro threads only run `DoNothing`.
- Every acceptance line maps to a named test. The bench defines its command, rows and comparison rule and keeps the default run silent. Docs, CHANGELOG (two declared Breaking lines) and the book-sync list are complete.

## Open questions for the architect
1. `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer` (cut.md:263) is meant to show that the wait sees sent messages. Microsoft does not document whether `MsgWaitForMultipleObjectsEx` itself dispatches a sent message when `QS_SENDMESSAGE` is missing from the mask. If it does, the test cannot go red. Settle it with one mutation run (`QS_ALLINPUT` -> `QS_ALLEVENTS`) in test_r1.md. If it stays green, call it a smoke test, not a pin.
2. Q2 / open question 10: the owner's text says another key-down "cancels a pending Tap ... whose key passes". D6 instead fires a passing Tap that waits for the DoubleTap window. That is the AutoHotkey reading, where a Tap resolves at its key-up, and it is argued at cut.md:159-160. The orchestrator should confirm this reading of the decision instead of letting it pass silently.
3. Q1: I agree that `hook::main_thread::MAIN` is thread-owned state, not shared hook state. The reviewer's R3 rule still forbids `RefCell` until the orchestrator rules, so the ruling should reach the reviewer's brief before verify round 1.
4. C5's scan covers only `src/hook` and `src/engine`, and its tokens miss `thread::scope`. The gesture path also runs `core/windows_actions.rs` (`FastAction::run`), `system/system_function.rs` and `platform/timers.rs`. Either add those as function-scoped entries, as the log scan does, or name the test for what it covers.
5. `KeyboardHook::switch_enabled` is `pub(crate)` on a `Sync` type, and "main thread only" is stated only in a comment. A call from an IPC worker would bring back exactly the flip-without-reset race that D2 removes. Consider reaching it only through `MainState`.

## Absent topics (walked, not applicable)
- Registry (R9/KS-17/KS-21), config schema and RON compatibility (no schema change; the semantic breaks are declared as Breaking in CHANGELOG).
- Pipe DACL, single instance, elevation and child tokens (KS-02/KS-09/KS-20/KS-26): unchanged; requests stay in the process.
- Input synthesis (R2/KS-36): no new `SendInput` builder. Hold fast actions use the existing sentinel-tagged builders and pass before `with_main`.
- COM: none on the main thread. Auto-blocker: untouched. Exact modifiers, repeat defaults, `log` sink and licence: untouched.
