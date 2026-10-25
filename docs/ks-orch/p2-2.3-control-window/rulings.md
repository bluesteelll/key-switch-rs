# Orchestrator rulings for lane p2-2.3-control-window (2026-10-19)

Binding for every stage of this lane. Where a ruling differs from `cut.md`, the ruling wins. R1 and R2
change the design, so the architect revises `cut.md` against this file: a "Revision after rulings"
section, with the touched decisions, commits, tests, acceptance mapping and manual checks rewritten in
place. The architecture-critic then checks the revision (`critique_v2.md`) before any code is written.

## Design rulings

- **R1 No stuck key: an OS witness at a swallowed key-up (critique I1, option (b)).** A stuck key is the worst
  failure this daemon can cause. Take the key-up of a key whose latest key-down record is `Suppressed`. If
  `Env::os_key_down` for that key reports it down before the event, Windows saw a key-down that the hook did
  not swallow. The key-up then passes, and the record is consumed.
  - Otherwise the key-up is swallowed, as rule L says.
  - The cut cites Microsoft Learn for the claim that the asynchronous key state seen inside the low-level
    hook is the pre-event state. If that is unverified, it says so and names the manual check that settles it.
  - The witness costs one OS key-state read on the hook path, and only on the key-up of a swallowed key.
    The cut accepts that cost under the latency decision, because a stuck key is worse than ~100 ns.
  - With the witness, D2 may keep its every-daemon scope (stop forgetting swallowed key-downs on a
    foreground change, elevated or not). Correct D2's premise and Question 3.
  - Red-first: the critic's harness script on the template config must end with `check_nothing_stuck()` green:
    `down CapsLock`, `foreground_changed`, `up_unseen CapsLock`, `down_unseen CapsLock`,
    `foreground_changed`, `up CapsLock`. Run it also after a lost-input reset whose OS read missed the key.
  - The property tests gain a Focus mode that presses keys the hook does not see. A known-issues row states
    the residual, if any.
- **R2 A reset never discards state created after input came back (critique I2).** One unlock can deliver
  `EVENT_SYSTEM_DESKTOPSWITCH` and `WM_WTSSESSION_CHANGE(WTS_SESSION_UNLOCK)` in either order and across
  several drains. Two Ctrl+Alt+Del or UAC round trips deliver several switches. The invariant: a lost-input
  reset clears only presses, gestures, `active` flags and macros that existed before the input loss it
  answers. Anything the hook saw or started after input came back stays.
  - The architect picks the mechanism with evidence and cites its documented source. Options include an
    episode flag cleared by the first key event the hook sees after the reset, or event timestamps (a
    WinEvent's `dwmsEventTime`, a message's time).
  - Scenario pins:
    - `session_unlocked; down F13; session_unlocked; repeat F13` fires nothing on the repeat;
    - the same with `desktop_switched` and the two reasons in both orders;
    - a `Hold` armed and a macro started between two notifications survive the second one.
  - D7's coalescing sentence is corrected to match. M1 records one timestamped log line per reason in `impl.md`.

## Critique, Minor (adopted)

- **M1** Declared, not fixed here. A waiting Tap of a passing multi-key DoubleTap (D10) fires while an
  own modifier is down in Windows (for example Ctrl+Esc). Known-issues row 68 grows; item 3.1 owns the output
  modifier state.
- **M2** `nothing_is_stuck_after_win_l` must fail when the reset is removed. Name the mutation in `impl.md`;
  the tester repeats it.
- **M3** `refresh_foreground()` runs after the foreground hook exists, or again right after `install`. No
  window change between the two may leave a stale cache.
- **M4** The control window does not outlive the pump. It is destroyed before the executor stop and the 2 s
  bounded wait, or the cut proves that no message can reach it after the pump ends.
- **M5** No test asserts a registration that production tolerates failing. Either the test accepts both
  outcomes and names them, or it skips with a stated reason.
- **M6** The window procedure's session wiring is tested by calling it directly with
  `WM_WTSSESSION_CHANGE` / `WTS_SESSION_UNLOCK`, `WM_QUERYENDSESSION` and `WM_ENDSESSION`. No real session
  change is involved, and nothing is injected.
- **M7** The three stale doc lines join the touch set.
- **M8** The foreground-race rejection states its residual exactly.

## Architect's questions

- **Q1** accepted: `MAIN_WAKE` stays a thread message, and the window handles window messages only.
- **Q2** accepted as a declared deviation: a reload resets gestures, chords, sequences, the owed mask and the
  timer generation, but does not resync the key table, which would race the OS. The roadmap text "after a
  reload ... resyncs all 256 keys" is answered in the CHANGELOG and the Phase-2 book sync.
- **Q3** superseded by R1.
- **Q4** accepted as documented in known-issues row 76: a swallowed key that is still auto-repeating, held
  through the whole unlock, may fire once more.
- **Q5** accepted: on `WM_ENDSESSION(TRUE)` the daemon removes the hook, logs one line, flushes for at most 1 s and posts `WM_QUIT`.
- **Q6** accepted: the three `windows` features are those of an existing dependency, and `Cargo.lock` is unchanged.
- **Q7** accepted: the tester runs the bench on master, C15 and HEAD.
- **Q8** accepted: titles are read live with `GetWindowTextW`, which is documented as non-blocking for
  another process's window, and the exe name is cached per (HWND, pid). The orchestrator reports this to the owner.

## Latency (owner decision)

- The results analyst states in `results.md` the hook-path cost before and after:
  - a conditional binding: R9/R10 against C15 and master;
  - an ordinary key, which must not get slower;
  - R1's witness read.

# Round 2 (after the cut revision and critique_v2.md: APPROVED, Minor 4)

The developer implements the revised `cut.md` with these rulings applied, and records each one in the
"Developer addendum" at the end of `cut.md`.

- **V2-m1** adopted: manual check M1 passes on the invariant, not on a count. Every `state reset` line comes
  before the first return line, and `impl.md` records the timestamps of one Win+L and one Ctrl+Alt+Del round trip.
- **V2-m2, a late lock signal.** Keep resetting on the lock signal by its kind, because the owner decision on
  open question 9 needs that reset. Correct the claim that a late lock needs a stalled main thread: the
  session service has its own latency. Declare the residual in known issues: a lock signal that is
  delivered after a very short unlock (Windows Hello) can make the next repeat of a held, swallowed,
  non-repeating trigger fire once more. M1's timestamps measure the latency of the lock signal. If they show
  it arriving after the return, report that to the orchestrator; a later lane decides.
- **V2-m3** adopted: D3, row 63 and the CHANGELOG say that a UAC prompt counts as input that left only while
  "Switch to the secure desktop when prompting for elevation" is on (the default). Manual check M5 states that
  policy as a precondition.
- **V2-m4** adopted: the control window is dropped after `hook.uninstall()` and before `executor.stop()`,
  so no RPC or IME teardown runs while the hook is live.
- **Critic OQ1** adopted: a test asserts that no window is left on the thread after the drop. It enumerates
  with `EnumThreadWindows`, which is read-only.
- **Critic OQ2:** the witness reads the exact virtual-key code of the event. The developer checks in the code
  and in Microsoft Learn whether the hook can report a generic modifier code (`VK_SHIFT`, `VK_CONTROL`,
  `VK_MENU`) for another program's injected input. If it can, the generic state is read, the reason is written
  down, and a scenario pins a generic key-up. Never let a generic read pass a key-up whose sided key-down
  was swallowed while the other side is held: test that case.
- **Critic OQ3** adopted: the bench row R13 and the window test that register the same window class share one
  test mutex (or use different class names), so `--include-ignored` cannot fail on them.
- **Architect Q1** accepted: a fourth `windows` feature, `Win32_System_StationsAndDesktops` (`GetUserObjectInformationW`).
- **Architect Q2** accepted: the reset answers the input leaving (the lock, or a switch to another input
  desktop), not `WTS_SESSION_UNLOCK`. The acceptance line "Nothing is stuck after Win+L" holds. The roadmap
  wording is answered in the Phase-2 book sync and listed in `impl.md`.
- **Architect Q3** accepted: known-issues row 76 covers every key held through a lock. Such a key, if it still
  auto-repeats after the return, reads its first repeat as a press.
