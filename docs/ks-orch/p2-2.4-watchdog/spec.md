# Lane spec: p2-2.4-watchdog - Hook watchdog (KS-08)

Branch: feat/hook-watchdog

Roadmap item 2.4 (book/src/roadmap.md 'Hook watchdog'), KS-08 (book/src/audit.md: Windows silently removes a low-level hook whose callback exceeds LowLevelHooksTimeout). Owner decision in CLAUDE.md that applies above all: binds run with the lowest possible latency.
- Keyboard Raw Input (RIDEV_INPUTSINK) is compared with the keyboard callback's own counter. If key events arrive but the callback stays silent for more than 1 s, the keyboard hook is reinstalled on the main thread (hooks belong to the installing thread, KS-01) and the event is logged. The watchdog never pings with SendInput: injected input resets the idle timer and breaks sleep and screen-off.
- Latency first: Raw Input delivers a WM_INPUT for every key event. The cut decides, with Microsoft Learn evidence and a bench measurement, where that work runs: on the 2.3 control window on the main thread (the roadmap's text), or on a dedicated watchdog thread with its own window and message loop that only reads atomics and posts a reinstall request to main. The key path must not get slower (bench rows R1-R13 per CLAUDE.md; add a row for a key event with the watchdog active). A reinstall must never double-install the hook or leave a window with no hook, and a busy main thread must not trigger a needless reinstall loop: state the rule.
- No key data from Raw Input is kept or logged: only counts and times.
- swch status shows a HookHealth value (for example ok, the number of reinstalls, the time of the last one), compatible with today's swch: the one-line reply still starts the way swch and swch open's probe expect (src/ipc.rs, src/bin/swch.rs, tests/ipc_roundtrip.rs).
- The watchdog reuses the 1.7 log writer and logging::counters, and reports the gauges 2.1 added (longest_callback_us, executor_unavailable, active_runs) where they help HookHealth.
- A re-entry refusal counter for with_main (a refused re-entrant callback is counted, never logged on the hook path).
- Panics: a panic hook logs a panic message through the log writer with a bounded flush (roadmap 1.7 question 4, gaps.md). A panic inside an extern "system" callback aborts; the hook must not block the dying process for long: state the bound.
- The debug flag of the acceptance (it holds the callback for 1.5 s) must not exist in a release build, or must be impossible to trigger by accident; the cut chooses and argues it.
- Leftovers from 2.3 (D:/tmp/ks-orch/p2-2.3-control-window/results.md, 'Technical debt' and 'Problems'), as small separate commits: P1 (the MovingQuery fake PIDs are multiples of 4 and can collide with the test process's PID; a test-only fix); src/engine/winner.rs:57 stale doc line; README tree line 'when::' typo; WaitableTimer::not_a_timer uses .expect without the 'invariant: ...' form.
Acceptance:
- A pure watchdog decision (counts and times in, reinstall or not out) is pinned by unit tests with a fake clock: key events with a silent callback for more than 1 s give exactly one reinstall; a callback that keeps up gives none; idle gives none; a busy main thread gives no reinstall loop.
- With the debug hold of 1.5 s the watchdog reports reinstalls: 1 within 2 s: a manual check with the live daemon (agents never run it); everything below the live hook is tested without a hook.
- Ten minutes of idle time produce no injected input: a structural scan shows no SendInput or keybd_event in the watchdog code, plus a manual check.
- swch status shows HookHealth, and tests/ipc_roundtrip.rs covers the new reply against a stand-in daemon.
- The latency bench shows no unjustified regression on the key path with the watchdog active.
Update README (swch status, the watchdog), book config pages only if a config meaning changes (none expected), the errors page for the new log lines, and CHANGELOG. Non-config book pages go to the Phase-2 book sync, listed in impl.md with file:line.
