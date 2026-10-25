# Orchestrator rulings for lane p2-2.4-watchdog (2026-10-19)

Binding for every stage of this lane. Where a ruling differs from `cut.md`, the ruling wins. There is no cut
revision round, because every item below is local. The developer implements `cut.md` with these rulings
applied and records each one in the "Developer addendum" at the end of `cut.md`.

## Safety first (critique I2): no test can install a global hook

- **S1** This is the owner's machine and the tester runs the full suite on it. Under `cfg(test)` no code path can
  reach `SetWindowsHookExW`, `UnhookWindowsHookEx`, `SetWinEventHook` or `RegisterRawInputDevices` for real.
  The guarantee must hold at compile time, not through a runtime guard:
  - The reinstall path takes an installer trait, or an equivalent seam.
  - The Win32 implementation is `#[cfg(not(test))]`, or its methods are `unreachable!` under test before any
    Win32 call.
  - Tests use a fake installer that models handles.
- **S2** Do S1 first. The commit that makes the real installer unreachable under test lands before any
  test that exercises the reinstall path. Until it lands, no test may call `reinstall_keyboard_hook`.
- **S3** "Never two hooks, never none" is pinned with the fake, which models two handles. Install the new one,
  then remove the old one. A failed install keeps the old one. A hook that is not installed is never installed.
  Each rule has its own test, and a mutation of each must be caught (the tester runs them).
- **S4** A scan test asserts that `SetWindowsHookExW`, `RegisterRawInputDevices` and `SetWinEventHook`
  appear only in items compiled out under `cfg(test)`, or name the item that guarantees it. The reviewer
  verifies S1-S4 before anything else.

## Critique, Important

- **I1** The watchdog loop never loses its tick. After every wake, the loop re-arms from its state, as the
  main loop's `plan_wake` does: whenever it is ticking, it sets the timer for the next tick from `Instant`.
  An early signal is ignored by the clock check. A failed `SetWaitableTimer` is counted and retried at the next
  tick, and while it keeps failing, `swch status` does not say `hook ok`. Red-first: a fake timer that signals
  early, and one that refuses once. Both must still end in a reinstall request when the callback is silent.
- **I3** Order in `App::run`: the watchdog starts before `install`, as the executor does, and stops after
  `uninstall` (and after the control window drops). Main never waits on the watchdog while the keyboard hook
  is installed and main is not pumping. The developer states that order in a comment next to the executor's.

## Critique, Minor (all adopted)

- **M1** One unexplained raw event does not keep the watchdog ticking. Ticking stops when the evidence is
  older than the decision window and no new raw event came. Pin it with a test.
- **M2** A failed desktop check is counted (a counter with a `Display` arm), so a declined request leaves a trace.
- **M3** `hook reinstall failed` clears once the hook answers again or a later reinstall succeeds.
- **M4** Tests do not race on the global `COUNTERS`. The watchdog takes a counters reference (a local instance in
  tests), or the tests that read the globals share one test mutex. Saturating arithmetic where a difference is
  taken.
- **M5** A decision-row test pins that evidence is checked before this interval's raw events are added.
- **M6** The cut and the README state the system-side cost of the raw-input registration: Windows sends one
  `WM_INPUT` per key event to the watchdog thread, never to main. The bench row adds the watchdog thread's
  per-key work. If the key path cannot be measured end to end without a hook, say so.
- **M7** M1's precondition: the owner's release daemon is stopped first (the single-instance mutex), and the
  manual check says so.

## Architect's questions

- **Q1** accepted: the bound is measured from the ScrollLock key-down, with keys typed before, during and
  after the hold, as M1 prescribes.
- **Q2** accepted: a reinstall resets the engine with `resync_from_os` and keeps the running macros.
- **Q3** accepted: the debug hold exists only under `cfg(debug_assertions)` and needs `KS_DEBUG_HOOK_HOLD`
  set at startup. A release build has no code for it; a scan or `cfg` test pins that.
- **Q4** accepted: FU-1, FU-2 and FU-3 go to `impl.md` for the Phase-2 book sync, under "Follow-ups from Phase 2".
- **Q5-Q8** accepted: each unverified behaviour has a fallback, and the manual checks gather the owner's evidence.
  These manual checks go to the owner from the start (WORKFLOW.md, Rulings).

## Latency (owner decision)

- The results analyst states in `results.md`:
  - the key-path cost before and after (the added atomic increment, R14);
  - what R15 shows the rejected option would have cost;
  - the watchdog thread's idle behaviour, which must be no wake-ups at idle.

# Round 2: ruling L1 on bench row R14 (2026-10-19)

Triage confirmed T-F1 = R-I2: with the watchdog running, the key path costs R14 p50 22.7 us (p99 65.9 us)
against R12 4.7 us. Cut D11's bound is R12 + 300 ns. The extra cost comes from the watchdog thread handling a
`WM_INPUT` while the callback runs. Under the owner's latency decision this is not accepted, so option (a)
is rejected. Option (c), raw input on main, costs about 3.9 us per key on the key path (R15) and is rejected too.

- **L1 Option (b): the watchdog reads raw input in batches at its ticks, not one wake per key.**
  - While the watchdog is idle (not ticking), its wait includes raw input, so the first key event of a burst
    wakes it and starts the ticks. That one wake per burst is the only per-key wake left, and it is declared.
  - While it is ticking, its wait leaves out `QS_RAWINPUT`. At each tick it drains and counts the pending
    raw input in one pass, using `GetRawInputBuffer`, a `WM_INPUT`-filtered `PeekMessageW`, or whatever
    Microsoft Learn supports.
  - The rules of ruling I1 (never lose the tick), M1 (stop ticking on old evidence), M5 and the D3 decision
    table keep working on the batched counts.
- **Before code**, the architect writes a "Delta after ruling L1" section in `cut.md`, and the critic
  checks it in `critique_v3.md`. The section:
  - names the mechanism with Microsoft Learn evidence: is unread raw input kept, lost or merged while
    `QS_RAWINPUT` is left out of the wait; the queue limits; whether `DefRawInputProc` or cleanup is needed;
  - updates D3, D5 and D11;
  - adds a bench row for steady typing with the watchdog ticking. The D11 bound R12 + 300 ns applies to it.
    The first key of a burst gets its own declared row.
  - adds the red-first unit tests. Example: the wait mask leaves out raw input while ticking, and the batch
    drain counts N pending events in one tick.
- The developer then implements the delta as follow-up commits. The tester re-runs R12, R14 and the new rows
  on master and HEAD, 3 runs each. R14 within D11's bound in steady typing is the acceptance line.
- R-I1 (the S4 scan) is already fixed by `ee9ef4d`/`0e7165b` (test code only). The reviewer re-checks it in round 2.

# Round 3: rulings on the L1 delta and critique_v3.md (2026-10-19)

The developer implements cut.md's "Delta after ruling L1" (F1-F10) with these rulings applied, as follow-up
commits on the branch, and records each one in the Developer addendum.

- **V3-I1 No false reinstall from a raw-input backlog.**
  - A drain reads until the raw-input queue is empty. The per-pass bound is a time budget, not a record cap.
    When the budget ends with records still pending, the loop drains again before it waits. It never
    waits while raw input is pending.
  - The decision credits callback heartbeats against raw events by accumulated counts. A burst read late,
    over several passes, is matched by the heartbeats the callback already counted, so a live hook is never
    reinstalled for it. A one-interval time credit alone is not enough.
  - Events that raw input sees and the hook legitimately does not (another desktop, UIPI) must not add up
    to a false reinstall. State the rule, for example a baseline re-taken whenever the callback is seen alive.
  - Red-first:
    - the critic's trace: about 10 000 injected events in one burst, then 1 s of silence, gives no reinstall;
    - a dead hook during or after such a burst still gives exactly one reinstall;
    - long drift with a live hook gives none.
- **V3-I2** adopted: a control row R14c has the same 1 ms pacing and the same posted message before each
  timed call, with no watchdog. The bound is R14 p50 <= R14c p50 + 300 ns, and R14 p99 is held to the same
  rule as D11. `results.md` also reports R14 against R12 for context.
- **V3-M1** adopted: M9 gets one pass rule with a stated margin for the setup's own bursts (`swch open`,
  opening Notepad), for example W <= K / 10 + 20. The two rules no longer disagree.
- **V3-M2** adopted: R16 sets `hook_installed` the way production does. It runs n >= 1000, so p99 is
  meaningful. The README quotes R16's measured number, never a planned one.
- **V3-M3 / critic OQ1** adopted: the mutation table gives the right reason for each catch. A guard
  stops a keyboard-only idle spin: if `GetRawInputBuffer` returns 0 records while `GetQueueStatus` still
  reports `QS_RAWINPUT`, the drain falls back to one standard read. If the bit is still set after that, the
  event is counted and the loop backs off for one tick. It never spins. A fake-source test covers it.
- **Critic OQ2** adopted: the general pump peeks with no window filter, so `WM_QUIT` (a thread message) is
  always seen. Only the `WM_INPUT` read may filter by window.
- **Critic OQ3** adopted: the F8b smoke test asserts the documented outcomes for a process with no raw-input
  registration, both of them, and names them.
- **Critic OQ4** recorded as unverified: do pending `HRAWINPUT` objects count against the USER object quota?
  Drain-until-empty bounds the backlog, and M9 observes it.
- **Delta Q1-Q6** accepted as the cut recommends:
  - the once-per-run `[INFO]` stop line;
  - R16's sanity bound;
  - R14/settled removed;
  - M9 settles E4 on the live daemon, and a failure means a new delta for the kernel-wait fallback;
  - triage m1 is folded into F9;
  - 32-bit builds use the standard read.

# Round 4: ruling L2 on bench row R16 (2026-10-19)

Round 2 met L1: in steady typing, R14 p50 is 4.8 us, the same as the control R14c's 4.8 us. R16, the first key of
a burst, measures a p50 median of 52.7 us against the sanity bound of 29.7 us. The developer's diagnostics
(`bench-dev-diag.md`) show that this is the cost of the watchdog thread's wake from idle. It is neither the
decision work nor the thread priority. The construction before L1 already measured 35.6 us.

- **L2 R16 becomes a declared evidence row, measured and reported, not gated.** Reasons:
  - It times the callback at once after the wake, which is the worst order of cut Q6. If Windows generates
    raw input only after the low-level hook chain returns (the expected order, unverified), the wake for a key can
    never overlap that key's callback. It could overlap the next key's callback only if that key came within
    about 50 us.
  - It happens once per burst, after an idle of at least the decision window. It never happens in steady typing.
  - Removing it would need idle ticks: periodic wake-ups forever while idle. That is a cost on every idle second
    against a pessimistic microsecond figure.
  - The cut's R16 bound, the README and the module docs state the measured figure (about 50 us p50 above R12,
    once per burst, worst order), with no planned number. The addendum records L2.
- **Q6 goes to the owner as a manual check.** The order of raw input against the low-level hook call decides
  whether R16 is ever paid. Manual check M6 states how the owner's run settles it, using the debug build's
  timestamps if needed. A follow-up for item 3.5 (hot-path budget) is listed in `impl.md`: if M6 shows that raw input
  comes before the hook call, trim the user32 work on the watchdog's first pass.
- **p99 of the 1 ms-paced rows (R14, R14c).** The control alone swings 9 to 107 us between runs, so the paced
  rows gate on p50. p99 is reported as the largest of 3 against the control's largest of 3, as the tester did.
- **R-m1 and R-m2 are fixed in this lane.**
  - R-m1: the stop-line wording on `book/src/config/errors.md` is right for isolated bursts.
  - R-m2: key codes do not outlive their batch. The read buffer is cleared after each batch is counted. No key
    data is kept in memory longer than the count needs.
