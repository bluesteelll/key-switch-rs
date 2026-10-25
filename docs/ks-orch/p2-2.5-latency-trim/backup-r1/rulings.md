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
