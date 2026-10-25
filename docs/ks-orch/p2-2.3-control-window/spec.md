# Lane spec: p2-2.3-control-window - Control window and state reset (KS-29, KS-31, KS-06 status)

Branch: feat/control-window

Roadmap item 2.3 (book/src/roadmap.md 'Control window and state reset', including 'Status after Phase 0', the 'Latency' bullet moved from 3.5, and 'Note after Phase 1'); unverified KS-29 and KS-31; the KS-06 status line. Owner decisions in CLAUDE.md that apply: binds run with the lowest possible latency; after a key-up the hook never saw (Win+L), the next press of a swallowed, non-repeating trigger key fires (roadmap open question 9).
- A hidden top-level control window owned by the main thread (not HWND_MESSAGE, because the tray of item 4.6 needs TaskbarCreated). WTSRegisterSessionNotification (NOTIFY_FOR_THIS_SESSION), WM_QUERYENDSESSION, WM_ENDSESSION and the WM_APP_* messages go through it; EVENT_SYSTEM_DESKTOPSWITCH is hooked the way the foreground WinEvent hook is. Every handle and registration is released on every exit path (Drop guards). Item 2.2 noted that RegisterClassW needs a windows feature that is not enabled: name the feature in the cut.
- reset_transient_state() clears pending gestures, cancels macros (the 2.1 executor epoch through KeyboardHook::cancel_actions) and resyncs all 256 keys except the ones whose key-down was swallowed. It runs on swch off and swch on, after a reload, on WTS_SESSION_UNLOCK and on EVENT_SYSTEM_DESKTOPSWITCH, as a request served by the main thread (the 2.2 request queue and MAIN_WAKE; for example a Request::Reset).
- In an elevated daemon, a foreground change no longer clears every blocked bit; holding Win+Alt+T while the foreground window changes keeps the T repeats suppressed.
- Owner decision (open question 9): after Win+L with a swallowed, non-repeating trigger key held, the next press of that key fires. The roadmap's follow-up notes that for a key whose key-down passed, the OS key state can tell such a press from an auto-repeat (unverified); the cut chooses the mechanism with evidence and covers both the swallowed and the passed case.
- Pin the lone key-up case of the KS-06 status line: after a reset, the release of a key whose key-down was swallowed must not reach Windows as a lone key-up that a program acts on (PrintScreen is the candidate); decide and test the rule.
- Latency (moved here from 3.5): the executable name and the window title of the foreground window are cached per HWND from EVENT_SYSTEM_FOREGROUND, so evaluating a when: on the hook path never queries the process. A title can change inside one window (browser tabs): the cut chooses how the cached title stays current (for example EVENT_OBJECT_NAMECHANGE for the foreground HWND, or a read that is proven cheap and non-blocking), with evidence. The cache is filled on the main thread, outside the keyboard callback.
- Leftovers from item 2.2 (D:/tmp/ks-orch/p2-2.2-timers/results.md, 'Technical debt', 'Problems' and 'Hand-off: orchestrator'), as small separate commits:
  - reviewer m2: the dropped-gesture count and the hold_timer_not_started counter overcount (src/hook/main_thread.rs:227-240, :448-450; errors.md:198,208; counters.rs:27-30): count only what forget_timer really dropped, or reword;
  - reviewer OQ3: clear armed when SetWaitableTimer fails in GestureTimers::start (src/hook/main_thread.rs:256-263);
  - P1: a scenario that pins the clock rule for a waiting Tap whose DoubleTap window closed while its timer was late (TAP_AND_DOUBLE_TAP_F13, advance_ms_stalled(301), then down("F13"));
  - a passing multi-key DoubleTap (for example Ctrl+Shift) must not be cancelled by its own keys: 'another key-down' in the open-question-10 rule means a key that is not part of the pending gesture's combo (src/engine/gestures.rs:312; reviewer OQ2). Red-first scenario, then the fix;
  - src/core/executor/mod.rs:6 still names a hold-timer thread;
  - src/hook/config_watcher.rs:126 .expect on the watcher thread spawn becomes an error path (KS-33-like).
Acceptance:
- With a Tap armed, a reload followed by the key-up does not run the old action (scenario or adapter test without a hook).
- Holding Win+Alt+T while the foreground window changes keeps the T repeats suppressed (test without a hook).
- Nothing is stuck after Win+L: the reset on WTS_SESSION_UNLOCK clears gestures, cancels macros and resyncs keys (test the request path without a session change; the live unlock is a manual check).
- After Win+L with a swallowed, non-repeating trigger key held, the next press of that key fires (scenario test).
- 100 presses of a conditional binding in one window cause one process-name query (test with a counting stand-in for the query, no hook).
- The lone key-up rule after a reset is pinned by a test.
- The latency bench (tester.md section 4b) shows no unjustified regression; the per-HWND cache shows as a lower hook-path cost for a conditional binding (add a bench row for it).
Update README, default_config.ron comments, the book config pages whose semantics change (triggers-conditions.md for when:, known-issues.md rows 63 and the KS-06/KS-29 rows) and CHANGELOG. Non-config book pages go to the Phase-2 book sync, listed in impl.md with file:line.
