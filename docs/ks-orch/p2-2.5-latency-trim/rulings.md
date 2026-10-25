# Orchestrator rulings for lane p2-2.5-latency-trim (2026-10-20)

Binding for every stage of this lane. Where a ruling differs from `cut.md`, the ruling wins. The design changes,
so the architect revises `cut.md` against this file ("Revision after rulings" at the top, the touched decisions,
commits, tests, bench rows, acceptance and manual checks rewritten in place, "Critique response" at the end). The
architecture-critic checks the revision in `critique_v2.md` before any code is written.

The owner was asked about the two trade-offs below and declined to choose ("continue"). The orchestrator applies
its stated recommendations and reports them to the owner, who can still overrule them.

## Design rulings

- **W1 The OS witness keeps reading on every swallowed key-up. Critique I1 is answered by not skipping it.**
  The skip cannot be proven safe. Residuals R-E1, R-E3 and R-E4 rest on behaviour the cut itself calls
  undocumented or unverified. The spec's rule "an unknown case keeps the read", and the owner's "no key may ever
  stick", decide it: about 0.6 us on a swallowed key-up is cheaper than any stuck key. D2's
  stamp-at-last-release and the witness side of D1 leave the lane. R11 is unchanged; record it as such.
- **W2 Modifier resync leaves the key path. This is the cut's Q5, folded into this lane.** Every key-down
  calls `KeyState::sync_modifiers`, which reads the OS state of the 11 modifiers. That costs about 4 us of the
  4.7 us of an ordinary key event, the largest cost on the key path.
  - The keyboard callback reads no modifier state. The engine tracks the modifier flags from the events it
    sees, its own and the sentinel-tagged injected ones included where they change Windows' state.
  - The modifier resync from the OS runs on the main thread, between events, in the drain. It runs when the
    hiding count of the cut has changed since the last resync, and never inside a callback. The cut's hiding
    count (foreground change, lost-input reset, remapping switch, reinstall, refused event, a gap the watchdog
    saw, a late callback) is reused as the trigger.
  - The architect decides, with an argument, whether an extra slow resync is needed while keys are active.
    Either way it runs between events, so no key event pays for it.
  - Prove the failure mode: when a modifier event is hidden and the count does not move, only matching can be
    wrong, so one binding misfires or misses until the next resync. A modifier key-up whose key-down was
    swallowed stays covered by the witness and the suppression records, so no key sticks and no lone key-up
    reaches Windows. Trace it against `keys.rs` (`sync_modifiers`, `resync_from_os`, `key_down_swallowed`) and
    the KS-14 rule that a swallowed modifier's flag is not overwritten.
  - Pin every case with a scenario:
    - a hidden modifier key-up followed by a binding press heals at the next resync;
    - each source of the hiding count triggers a resync;
    - no stuck key and no lone key-up across the property modes of 2.3.
  - Declare the residual matching misses in known issues.
  - Bench: R12 (ordinary key) and R1/R4 must show the reads gone. The cut sets the new R12 bound from a
    diagnostic row (CLAUDE.md: a new latency bound is probed first). Add a row for the resync itself on main.
- **W3 The R16 fix (cut D7) is adopted.** On the first wake after idle, the watchdog makes no user32 read. It only
  starts its timer, and the raw input waits for the first tick. Critique M3 applies: each burst now costs one
  more wake, and D7's Cost line and M6 say so. R16's bound is provisional (cut Q3). The developer probes it right
  after the commit, and if the probe misses, stops and reports with the numbers.
- **W4 `hook_late` (critique I2).** It is now only a resync trigger, so a false positive costs one extra
  resync between events, never correctness. It must not produce a WARN line in normal typing.
  - Count it without making the minute line a warning by itself, or report it at INFO.
  - A manual check asks the owner to confirm that `hook_late` stays at 0 while typing on the release daemon.
  - The cut marks the `KBDLLHOOKSTRUCT.time` / `GetTickCount` base as unverified, cites what Microsoft Learn
    does say, and makes sure an injector's own time cannot flood the trigger.

## Critique, Minor (adopted)

- **M1** Declare the E5 ordering residual: a key event dispatched in the same awake stretch after a stall is
  decided before the note. Or move the note so it lands first.
- **M2** The W-table gains a short `SendInput` count and injected generic modifier codes. Test the R8 behaviour
  change.
- **M3** see W3.
- **M4** Every compile-red test has a named mutation that the tester runs.

## Architect's questions

- **Q1** Moot under W1, because the stamp was for the witness skip.
- **Q2** Moot for the witness under W1. For W2, the residuals are matching misses only; they are declared, and the
  owner sees them in the lane report.
- **Q3** see W3.
- **Q4 and the rest** accepted as the cut recommends, unless W1-W4 change them; the revision says which.
- **Q5** see W2.

## Latency (owner decision)

The results analyst states in `results.md`:
- R12 and R1/R4 before and after (the modifier reads);
- R11 unchanged;
- R16 before and after;
- the resync's cost on main.

# Round 2: rulings on the revised cut and critique_v2.md (2026-10-20)

critique_v2: CHANGES_REQUESTED (Critical 0, Important 2, Minor 5). W1, W3, W4 and the stuck-key / lone-key-up side of
W2 hold (Lemmas 1 and 2, D10, D15). Both Important findings are stale flags that no between-events read repairs. I2 is
a regression against master on every press wherever a later hook swallows a modifier (an AutoHotkey remap, a Remote
Desktop client). It is not accepted as a declared residual, so W2 is amended. The architect revises `cut.md` in place
against this round ("Revision 2 after rulings" at the top, the touched decisions, W-table, D11, row 88, commits,
tests, mutations, bench rows, acceptance and manual checks rewritten, "Critique v2 response" at the end). The critic
checks it in `critique_v3.md` before any code is written. Backups of the round-1 revision are in `backup-r2/`.

## W2a: a clear-only check of the modifiers held for the engine, at a decided event (amends W2)

- At a decided (non-sentinel) key event, before its decision reads any modifier flag, the engine checks only the
  modifier slots whose flag is set. It skips the event's own key and every modifier whose key-down was swallowed
  (KS-14, `key_down_swallowed`; the architect states the rule for D10's `RepeatSuppressed`). For each slot left, it
  reads that code's OS state and **clears** the flag when Windows has the key up. It never sets a flag from this read.
  A generic slot (`VK_SHIFT`, `VK_CONTROL`, `VK_MENU`) reads its generic code, which reads up only when neither side is
  down (GetAsyncKeyState), so a clear-only read cannot bring back D3's phantom.
- **Cost.** One `GetAsyncKeyState` per modifier held for the engine, about 0.36 us each, on the key path. Ordinary
  typing with no modifier held reads nothing, so R12 keeps the whole saving. A shortcut pays for the modifiers it
  holds: Ctrl+J one read, Ctrl+Shift+X two, against master's eleven on every key. The orchestrator accepts this under
  the owner's latency decision. A regression on every press, wherever another hook is installed, outweighs about
  0.4 us per held modifier on a shortcut.
- **Why it is safe to read there.** Master's per-event sync has read every modifier except the event's own key at every
  event. If Windows had not yet applied an earlier event's key at the next decided event, every held modifier would
  read up on master. So it relies on the same ordering, and the cut cites LowLevelKeyboardProc's note that the
  asynchronous state is updated after the callback. If any decided event can run before an earlier event's key is
  applied (a re-entrant decided call inside `carry_out`, if one exists), the architect shows it and skips those keys,
  as D5 does. A modifier is never skipped only because it is fresh in D5's drain sense: that would leave I2 open
  whenever no drain ran between the modifier's key-down and the shortcut key.
- **What it repairs:** I2, a passed modifier that a later hook swallowed, for as long as it is held; I1, a generic
  phantom; D7's case, a phantom-held modifier after an unnoticed key-up; and M1(b), the blocker swallowing a whole
  Alt or Shift press on a phantom-held partner. All of these are repaired at the event that would have used the stale
  flag, before matching.
- **D7, the 1 s read while a modifier is held.** The architect decides, with an argument, whether anything is left for
  it under W2a. The recommendation is to drop it, and with it `RESYNC_PERIOD` (Q11), if every reader of a set flag
  (D11's list: matching, repeat detection, the menu mask, `mark_completed`) runs at a decided event that the check
  precedes.
- **The drain read stays for the other direction.** It runs on main between events, on the hiding count. It sets and
  clears the sided slots, and it clears a generic slot only when the generic code reads up (critique I1's option 1).
  That direction is phantom-released: Windows has a modifier down and the engine does not, after a hidden key-down.
  The W-table, D11 and row 88 are re-derived for it.
- **W10 is corrected** (Hooks Overview: `SetWindowsHookEx` installs at the beginning of the chain, so a hook installed
  earlier, or installed before the daemon's 2.4 reinstall, runs after the daemon's). Its "what repairs it" column
  becomes W2a.
- **Pin it red-first:**
  - the critic's AutoHotkey trace: LAlt passes and a later hook swallows it (`os_down` stays up), injected LCtrl, then
    J. Exact `Ctrl+J` fires on every press, auto-repeats of LAlt included;
  - the blocker trace: LShift passes and a later hook swallows it, then LAlt. The blocker does not swallow the LAlt,
    and Alt+Tab reaches Windows as Alt+Tab;
  - I1's scenario: a generic key-down, an unnoticed generic key-up, `hidden_input_noted()`, then CapsLock fires
    `SwitchLanguage`;
  - a test that counts the reads: zero with no modifier held, one per held unswallowed modifier, none for a
    swallowed one;
  - each has a named mutation the tester runs: the check removed, the check allowed to set, the KS-14 skip removed,
    the generic read replaced by a sided one.
- **Bench.** R12 (nothing held) keeps the new bound. A new row R12m times a key with one held modifier, which costs
  one read, and R12m2 with two. Each is bounded from a diagnostic row against that row's probe, as M5 below says. R1
  and R4 are restated for W2a.

## Critique v2, Minor (all adopted)

- **M1** Goal 3 and the rulings' Q2 drop "matching only". Row 88 and the lane report name what the user sees for each
  residual left after W2a. Example: a phantom-released Alt or Win that gets no mask, so the menu bar or Start opens.
  Another: the blocker missing on a phantom-released partner, so the layout switches.
- **M2** Re-derive the scenario-to-mutation map under W2a. No scenario is listed against a mutation it passes under.
  S9, or its successor, asserts `expect_nothing_fired()` on the second LShift down.
- **M3** `errors.md:215` keeps "about 10" and "about 1000". D12's Cost, the module Cost doc, M6, M7 and the CHANGELOG
  say: one more tick only for a burst made of a single raw event, such as a key-up whose key-down came 2 s or more
  earlier.
- **M4** Every compile-red test of every commit has a named mutation, including those the critic lists for commits 4, 5,
  7 and 11 and every new W2a test.
- **M5** Every gate compares numbers from one session:
  - The tester runs the diagnostic probes itself, in its own session, on the parent tree with the one-line change
    applied in a scratch worktree, next to its parent and HEAD runs.
  - The bounds of R12, R12m, R12m2 and R4 are relative to those same-session probes. The developer's own probe run
    only sets the expected values and the stop rules.
  - R1 gates against the same-session D-R1n, or else requires a drop. It must fail if the removal is reverted on the
    engine path.

## Critic's open questions

- **OQ1 (U4)** Recorded as unverified. Under W2a it only sets how likely I1's trigger is, not whether the engine is
  right, so no manual check is needed.
- **OQ2 (StickyKeys)** A W-table row: a modifier state that Windows changes with no LL hook event. Manual check: turn
  StickyKeys on, tap Shift, type a letter, then press CapsLock; the layout switches. If it fails, a follow-up for 3.5
  goes in `impl.md`.
- **OQ3 (Remote Desktop server stamps)** Recorded as unverified under W4/U1. A wrong stamp costs extra reads between
  events and an `[INFO]` line. Name it in M8's notes.

## Architect's questions (round 2)

- **Q4** (E4, 17 ms, one `GetTickCount` per physical callback) accepted with its stop rule.
- **Q9** accepted: a chore after the merge, with the content updated for W2a. Principle 1's sanctioned reads become the
  witness, the KS-05 blocker read and W2a's check.
- **Q10** accepted: the KS-05 blocker read stays on the key path. It is the KS-05 witness, not a resync. With W2a, its
  hotkey's flags are checked before it.
- **Q11** see W2a. `RESYNC_PERIOD` goes if D7 goes. If D7 stays, 1 s is accepted.
- **Q12** accepted: if a seed of the `Unnoticed` property fails, the developer stops and reports the seed and the
  transcript. No workaround. The property gains the later-hook case: a passed event that Windows does not apply.
- **Q13** accepted: D10's corner-case rule change closes a lone-key-up hole that exists on master. The CHANGELOG lists it
  under Fixed, citing roadmap 2.5.

## Latency (owner decision), round 2

The results analyst states in `results.md`:
- R12, R12m and R12m2 before and after, against their same-session probes;
- R1 and R4 before and after;
- R11's witness term unchanged;
- R16 before and after;
- R18, the read on main, and how often it runs;
- the W2a trade in one line: zero reads with nothing held, one per held modifier.

# Round 3: rulings on revision 2 and critique_v3.md (2026-10-20)

critique_v3: CHANGES_REQUESTED (Critical 0, Important 1, Minor 5). The round-2 rulings are carried out. D16 runs before
every flag reader, and it only clears, so no key sticks and no key-up reaches Windows alone. There is no further cut
round, because every item below either removes something or is local. The developer implements `cut.md` (revision 2)
with these rulings applied, as in 2.2 and 2.4. Each one is recorded in the "Developer addendum" at the end of
`cut.md`, which lists every section, test, mutation row and CHANGELOG line that a ruling removes or changes. Backups
are in `backup-r2/` (`cut-rev2.md`, `critique_v3.md`).

## V3-I1: D17 is dropped. The check keeps master's reading of a nested decided event.

- A nested decided event can see an outer modifier key-down in two ways:
  - the hooks after the daemon's pass the outer key-down, and Windows applies it after the nested event: D17 is right
    and master is wrong;
  - the later hook that injected the nested event swallows the outer key-down: D17 is wrong and master is right.
- The only systematic source on record is the second one. PowerToys Keyboard Manager sends its remap from inside its
  hook and returns 1 (its developer docs and source, cited in critique_v3 I1). With D17, its remapped keys would be
  decided with a phantom modifier on every press, as with Win to F13 or an Alt/Ctrl swap. The first case has no
  documented source (U7). Master has the same behaviour there, so dropping D17 restores master parity in both cases,
  and D11's "the check is a subset of master" becomes true.
- Removed: D17, `UNAPPLIED`, the `Unapplied` guard, `unapplied_while_running`, `Env::unapplied`, `HookEnv::unapplied`,
  their tests and mutation rows, and their mentions in principle 1/5 (Q9), the thread map, the touch set and the
  CHANGELOG. W11's "D17 covers a callback still on the stack" is replaced by the declared residual below.
- **Declared** in the W-table and row 88:
  - A key event decided inside the callback of a passed modifier key-down, while every later hook passes that key-down,
    reads the modifier up and clears it, as master does.
  - The modifier then counts as released (direction B) until its next event, an auto-repeat or a key-up. What the user
    sees: that one nested key misses a binding that names the modifier.
  - No source of such an event is known (U7).
- **Pin:** an engine test with a nested decided event whose outer modifier Windows keeps up (the PowerToys case): the
  flag is cleared, and an exact `F13` binding fires at the nested F13. Its mutation: the outer modifier is skipped
  (D17 restored).
- **Manual check M11:** start PowerToys Keyboard Manager first, remapping Left Win to F13; then start the daemon, so
  PowerToys' hook runs after the daemon's. Add a temporary `( keys: "F13", action: SwitchLanguage )` and remove it
  afterwards. Every LWin press switches the layout, and Start does not open.

## Critique v3, Minor (all adopted)

- **M1** R-F gains a passing key-up with an unswallowed LAlt held, which must ask 0, or X31 is split and each variant
  lists only the tests that catch it.
- **M2** The `Unnoticed` property gets a named mutation that its fixed seeds fail under (X4, D10 reverted, or the
  witness term removed). The developer confirms it fails, and the tester repeats it.
- **M3** M9 gains a latched step: tap LShift, then press CapsLock with nothing in between. Expected: `ToggleCapsLock`,
  which master gives if the latch reads down. The owner records what HEAD does. W13's phantom-released cell says
  what the user sees: a Shift binding misses while Shift is latched. A difference from master is a follow-up for 3.5
  in `impl.md`.
- **M4** D11, row 88 and the owner summary add the effect of direction B on the blocker: Windows has Ctrl or Win down
  unseen, so the `Alt+Shift` blocker swallows the Shift or Alt key-down, and Ctrl+Alt+Shift+K arrives as Ctrl+Alt+K.
- **M5** Wording, no extra reads (latency first). Goal 4, row 88, the owner summary and `keys.md:194` say "the next
  key-down". The D7 argument and D11's "as on master" are corrected: master also repaired at passing key-ups. The
  case the critic traces is row 63's kind and is declared: a lost LCtrl key-up, then a passing C key-up, then an LCtrl
  press read as a repeat.

## Architect's questions (revision 2)

- **Q1** (R16 provisional, D-R16d, HEAD after the last commit, stop and report on a miss) accepted.
- **Q2** accepted: E4 and E5 on `GetTickCount`, with the stop rule (R17t p50 > 100 ns: E4 drops its age part, and E5
  goes back to `Instant`).
- **Q3, Q4** accepted.
- **Q5** (the chore after the merge) accepted, without D17's guard and `UNAPPLIED`.
- **Q6** (the KS-05 read stays; the check runs before it), **Q7** (an `Unnoticed` seed that fails stops the developer)
  and **Q8** (D10 under Fixed) accepted.
- **Q9** accepted. Commits 7 and 8 carry the I1/I2 regression for two commits, so commit 8's tests are red by assertion.
  The tester replays commit 8's red-first tests on commit 7, which the developer names by sha in `impl.md`, not on
  master. First-parent `master` never holds commit 7's state.
- **Q10** accepted: the tester's parent runs use commit 2 (P). It checks `git diff --stat master <commit 2> -- src`
  shows bench code only.
- **Q11** accepted as a follow-up for 3.5, listed in `impl.md`: the own-key skip keeps row 63's modifier case.

## Latency (owner decision), round 3

Unchanged from round 2. `results.md` also states that dropping D17 removes its TLS guard from the passed
modifier key-down path.

# Round 4: rulings on the developer's stop rules (2026-10-20)

`impl.md` (head `ea944da`) stopped on two stop rules of D13. The owner asked for this lane to be finished and merged,
and for development to stop after it. These rulings close the stop rules. Nothing else is reopened.

## L3: D12 is reverted. R16 becomes a gated row.

- The data in `impl.md`:
  - **P (master plus the bench):** D-R16d misses in all six runs, 28.7 to 131.2 us against R12 + 20 us. So D12 alone
    does not fix R16.
  - **N and commit 11, without D12:** it passes in all five runs. R16 at commit 11 is 4.3 to 4.4 us.
  - **HEAD, with D12:** R16 is 5.1 to 5.6 us. D12 shows no measurable gain.
  - The drop from about 50 us came with the removal of the per-event reads (W2/W2a). The callback is now too short to
    overlap the watchdog's wake.
- So D12 buys no latency, yet it costs code, tests and one more tick for a burst made of a single raw event. Under
  the owner's latency decision, code on that path must earn its place by measurement.
- **The developer reverts `ea944da` and `ddb9f3a` with `git revert`**, as follow-up commits with their own
  `Watchdog:` subjects that cite ruling L3. Never amend or rewrite.
  - The revert restores `errors.md`, `README.md`, the module docs, the bench construction and the loop-model
    expectations of Q6, and removes the D12 CHANGELOG entries.
  - The Developer addendum and `impl.md` record L3. They also list the D12 tests and mutation rows (X46 to X49) as
    removed.
  - Manual checks M6 and M7 go back to their 2.4 wording.
- **R16 is gated from now on** (2.4 ruling L2 is superseded): HEAD R16 p50 ≤ HEAD R12 p50 + 20 us, in the tester's
  session, median of 3 runs, p99 reported. The 3.5 follow-up from 2.4 (trim the watchdog's first-pass user32 work if
  M6 shows raw input before the hook call) stays listed, with the new numbers: it is likely moot.

## L4: R17a's stop rule is accepted as met within noise

- The p50 median over 19 runs is 0.613 us, against 0.6. At low load it is 0.561 to 0.576. Several runs had 79 to 100 %
  CPU load from the owner's own programs.
- The rule was there to catch a read that costs a system call or more. It did not catch one.
- One held modifier costs about 0.6 us on the callback, and two cost 1.1 to 1.4 us. Master reads eleven codes on every
  event, 4.8 us, and an ordinary key now costs about 0.2 us (R12).
- The cut's and the rulings' "about 0.36 us each" becomes the measured "about 0.6 us each" in the addendum, the
  CHANGELOG and `results.md`.
- R17a stays evidence, with no new gate.

## For the tester

The bench runs in one session (P = `62c9fd8`, N, HEAD after the revert) under ruling M5. The owner's machine is
loaded at times, so record the load of every run. A breach that comes only from a run at 79 % load or more is re-run
once before it is reported as a defect.
