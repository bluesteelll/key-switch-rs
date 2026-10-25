# Orchestrator rulings for lane p2-2.2-timers (2026-10-19)

Binding for every stage of this lane: developer, tester, reviewer, triage and results. Where a ruling
differs from `cut.md`, the ruling wins. The developer records how each one was done in the "Developer
addendum" at the end of `cut.md`. No cut revision round: every item below is local enough to implement
directly.

## Critique, Important

- **I1 A Hold past its term is resolved by the clock, not by the wake order.** In `other_key_down`, a `Held`
  press whose Hold term has passed (`now >= due`) counts as resolved and is kept for its timer, as `DueHold`
  is. It is never cancelled by another key-down. The engine decides by `Instant`. A late timer must not
  change the outcome: `cut.md:351` claims that, and the code must make it true. Red-first: a C10 scenario with
  `advance_ms_stalled(1001)` followed by `down("Right")` on `Ctrl+Shift Hold(1000)` fires the Hold, exactly as
  `advance_ms(1001)` does. Apply the same rule to every other timer-driven state that a late timer could
  flip, such as a DoubleTap window that has already closed by the clock. The addendum lists each one.

## Critique, Minor (all adopted)

- **M1** The C7 migration list also covers `keyboard_hook_callback.rs:492-518` and `:520-541`,
  `keyboard_hook.rs:592-610` and `app.rs:365-372`. No test is deleted to make the migration pass. Each is
  migrated or replaced by an equivalent, and the addendum names the replacement.
- **M2** Declared in `known-issues.md` (row 75 / section 11.6) and the CHANGELOG. A Tap that waits for the DoubleTap
  window fires into the window that has focus when it fires, without re-checking `when:`, and with the
  modifier state of that moment. Today's Tap at key-up fires into the window of the press. Item 3.1 owns the
  output modifier state.
- **M3** Row 67 says that only key-downs cancel. A mouse click does not, because the low-level keyboard hook
  does not see it: Ctrl+click still fires a passing `LCtrl` `Tap`. That is declared, not fixed.
- **M4** The main thread never blocks on a request's reply: a non-blocking send whose failure is ignored. A
  request that timed out at the IPC worker still runs when main reaches it, and the cut declares that. A request that
  arrives after `run()` returned is answered at once, for example "daemon is stopping", instead of each waiting
  2 s.
- **M5** R8 measures one-way delivery at the receiver, runs 3 times, and uses a defined margin. D9 (HIGHEST)
  lands unless R8 shows HIGHEST worse than NORMAL beyond the margin in at least 2 of the 3 runs. The rule and
  the numbers go into `impl.md`.
- **M6** The cut defines two cases, and the addendum records both. First, what a refused window timer does
  with its deferred Tap: it fires at the next event or the next wake by the clock, and is never lost silently.
  Second, what `press(k)` does when `k` still has a stale `Held` press, a passing modifier whose key-up was lost.
- **M7 Safety.** Under `cfg(test)`, `FastAction::run` never reaches `SendInput`, `PostMessageW` or a language
  switch. It records into a test sink instead, as `send_menu_mask` already only counts under test. Every test that
  reaches `carry_out` then injects nothing, whatever its action. This is stronger than "use `DoNothing`", and
  it is required.
- **M8** The 200 ms real-timer test reads `t_down` before `decide_event`, so lateness is never negative.

## Critic's open questions

- **OQ1** adopted: a mutation with `QS_ALLEVENTS` instead of `QS_ALLINPUT` must turn the sent-message test red.
  The tester runs it.
- **OQ2 / cut Q2, a waiting Tap on another key-down: fire it (the cut's D6 reading).** The owner's
  decision targets keys pressed while the gesture key is held: two capitals with Shift, Ctrl+C, Ctrl+Shift
  selection. All of those cancel. A Tap that waits for the DoubleTap window has already completed, a
  press and release with no key in between. Firing it at the next key-down keeps it before that key, as a
  Tap-only binding would fire, and it fires sooner than the window would. A change of this reading stays
  confined to `other_key_down`. The orchestrator reports the reading to the owner.
- **OQ3** This file is the Q1 ruling for the reviewer.
- **OQ4** adopted: C5's scan also covers `windows_actions.rs`, `system_function.rs` and `platform/timers.rs`,
  plus `thread::scope` and `thread::Builder`.
- **OQ5** adopted where feasible: `switch_enabled` is reachable only through `MainState`. If that is not
  feasible, the addendum says why, and a test pins that only main calls it.

## Architect's questions

- **Q1** accepted. `hook::main_thread::MAIN` is a thread-local owned by the main thread and reached only
  through `with_main` (`try_borrow_mut`, never a panic, never a deadlock). It is not the shared state that
  CLAUDE.md principle 5 bans. The orchestrator updates CLAUDE.md and the agent files after the merge.
- **Q2** see OQ2.
- **Q3** accepted: `THREAD_PRIORITY_HIGHEST`, not `TIME_CRITICAL`, under the M5 rule.
- **Q4** accepted: the `Tap(100)+Hold(200)` gap stays and is documented. A loader warning goes to 3.4.
- **Q5** accepted, with these rules for the bench:
  - The tester runs it on master and on HEAD in its own worktree and reports every row against D10's rule,
    including R8 with `KS_BENCH_LOAD=1`. R8 loads every core at normal priority for a few seconds, which is allowed.
  - The bench is counted apart from the `manual:` tests and never as a pass.
  - A breach of D10's rule is a defect unless a re-run inside the bounds is recorded.
- **Q6** accepted: the KS-11 ports are red on the 7cb3521 repro replay, as the acceptance line allows.
- **Q7** accepted: the executor's `MWMO_INPUTAVAILABLE` pin goes to 3.4 (one shared wait and pump in
  `platform` with one pinning test). This lane pins the main loop's.

## Latency (owner decision, CLAUDE.md)

- The results analyst states the per-event cost on the hook path before and after (R1, R2 and R4), the gesture
  deadline lateness (R6), the sent-message delivery (R7), and the R8 verdict, all in `results.md`.
