# Lane spec: p2-2.2-timers - Timers on the main thread (KS-11, KS-25, KS-33)

Branch: feat/main-thread-timers

Roadmap item 2.2 (book/src/roadmap.md 'Timers on the main thread', including 'Status after Phase 0', 'Status after Phase 1' and its 'Latency' bullet), KS-11, KS-25, KS-33 (the Hold half). Owner decisions in CLAUDE.md that apply: binds run with the lowest possible latency; another key-down cancels a pending pass-through gesture (roadmap open question 10).
- Gesture timers (Hold, the Tap/DoubleTap windows where they need a timer, Chord and Sequence entry expiry) are owned by the main thread and delivered through its message loop, in the same queue as the keyboard events: no hold-timer thread per press. Each timer carries a TimerToken { generation, slot, seq }; a stale token does nothing.
- Latency: deadlines are not rounded to the 15.6 ms system tick. Reuse platform::timers::WaitableTimer from item 2.1 and the executor's MsgWaitForMultipleObjectsEx + pump shape (src/core/executor/mod.rs) for the main loop, unless the cut shows a better way. The new wait must deliver the low-level hook callbacks with no added delay (argue it from Microsoft Learn; test what can be tested without a hook). Keep the 2.1 discipline: drain before every wait, MWMO_INPUTAVAILABLE, and a test that pins MWMO_INPUTAVAILABLE (D:/tmp/ks-orch/p2-2.1-executor/results_v3.md, technical debt m-1: a foreign message posted behind a wake or a timer signal is still dispatched).
- The Mutex<Engine> goes away: the engine lives on the main thread and is reached only there, so CLAUDE.md principle 5's exception ends. Keep the remapping-off fast gate (pinned by remapping_off_callbacks_pass_without_waiting_for_the_engine_lock or its successor) and a safe answer to a re-entrant callback on the same thread.
- Tap, Hold and DoubleTap on one key form one mutually exclusive state machine. When a key has both Tap and DoubleTap, Tap waits until the DoubleTap window closes (only then; declare the latency cost). DoubleTap entries are kept per binding or gesture group, not per key code (book/src/config/known-issues.md row 74; roadmap 'Follow-ups from Phase 1').
- Owner decision (open question 10): another key-down cancels a pending Tap, Hold or DoubleTap whose key passes; the two scenarios that pin known issue 67 flip. The cut states what happens to a pending gesture whose key is swallowed when another key goes down, and keeps today's behaviour for it unless it argues otherwise with evidence.
- Chord and Sequence entries expire on their own timers.
- A Scheduler trait replaces the timers in tests; engine scenarios drive time through the fake.
- Engine::on_timer never asks for the menu mask (engine::scenarios::a_hold_timer_never_masks stays).
- Latency bench, added by this item and run by every later lane on the parent and on HEAD: Engine::on_key and on_timer per event, the adapter path without a hook, and a timer row (the 2.1 delay_lateness probe shape). No new dependency, not part of the default cargo test run (the default run still prints nothing); the cut defines how it is run and compared.
- The cut evaluates, with evidence (Microsoft Learn; a measurement where one is possible without a hook), whether raising the priority of the thread that runs the hook lowers latency under CPU load without starving other input. Implement it only if the evidence supports it; otherwise record why not.
- R-7 of item 2.1 (a hold-timer wake posted to a reused thread id at exit) closes with the hold-timer threads.
Acceptance:
- stale_tap_gc_eats_second_quick_tap and stale_hold_timer_fires_new_press_early from the gesture repro crate (book/src/methodology.md#gesture-repro-crate) are ported with their assertions inverted and pass; at 7cb3521 the inverted tests fail (where a red on master needs a seam master lacks, the cut says on which commit it is red).
- With Tap(200) and Hold(200), a release at 199 ms fires only Tap and one at 201 ms only Hold (scenario tests).
- No thread per press: the thread count does not grow over 1000 presses (a test through the adapter without a hook, or a structural scan that no thread is spawned on the gesture path).
- A real-timer test shows a 200 ms gesture deadline met within a few milliseconds (the cut fixes a CI-safe bound with a reason).
- Another key-down cancels a pending pass-through gesture (scenario tests; the two known-issue-67 scenarios flip).
- The known-issue-74 case no longer fires the first binding's action.
- The latency bench runs, and its numbers for master and for HEAD are in impl.md and test_r1.md; no per-event regression on the hook path beyond what the cut justifies.
Update README, default_config.ron comments, the book config pages whose semantics change (triggers-conditions.md; known-issues.md rows 67 and 74 and the KS-11/KS-25 rows) and CHANGELOG. Non-config book pages go to the Phase-2 book sync, listed in impl.md with file:line.
