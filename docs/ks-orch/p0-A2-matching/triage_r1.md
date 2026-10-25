TRIAGE: 1 confirmed, 0 refuted, 1 deferred

# Triage r1: p0-A2-matching @ 965ac4d (fix/matching HEAD; master = merge-base e67e5cb)

Inputs: `test_r1.md` (tester, GREEN, "Failures: None": no red items) and `review_r1.md` (CHANGES_REQUESTED: I1 Important; M1, M2 Minor).
Code read through `git show fix/matching:<path>` and `git show master:<path>`; the shared work tree was not used as evidence.

Method: both refs were exported with `git archive` to `D:/tmp/ks-orch/p0-A2-matching/triage/{lane,master}`, and the same probe
scenarios (`triage/probe.rs`, `triage/probe5.rs`) were appended to each `src/engine/scenarios.rs`. Each tree ran
`cargo test --locked --lib triage_probe` with target dir `triage/target`. Every probe asserts master's behaviour. All probes are
pure engine scenarios with a fake `Env`: no hook, no `SendInput`, no registry.

One harness artefact was caught and redone. Both trees share one target dir, so one lane rerun silently executed the master
binary (it reported "193 filtered out", master's test count). The lane was rerun after `touch`, and "256 filtered out" in the
output confirms the lane binary. Outputs: `triage/lane_probe.txt`, `triage/master_probe_summary.txt`, `triage/p5.txt`.

## Verdicts

- **I1 CONFIRMED** (paths (a) and (b), plus the doc claims). This is a regression against master, introduced by C9's repeat
  override. Code:
  - `src/engine/mod.rs:427-434`: `if was_repeat { decision.suppress = self.keys.is_blocked(vk.0); }` discards the binding's
    request on a repeat.
  - `src/engine/mod.rs:298-301`: `on_foreground_changed` -> `clear_all_blocked()` wipes the bit of a key that is still held.
  - `src/hook/keyboard_hook_callback.rs:248-259`: this is wired to every `EVENT_SYSTEM_FOREGROUND`.
  - On master the Immediate arm's `suppress` survived on repeats and was re-marked (`master:src/engine/mod.rs:311-319`, `:409-411`).

  Probe results (each asserts that the repeat is suppressed):
  - p1 (`F13 -> PressKey(Left)`; down, foreground change, +500 ms repeat): master ok. Lane fails with
    `repeat F13 at +500 ms: expected the event to be suppressed, it was passed`, and the repeat fired `PressKey(Left)`. Fire and
    pass in one decision.
  - p2 (default-config pair `CapsLock -> SwitchLanguage` + `Shift+CapsLock`): master ok. Lane fails with
    `repeat CapsLock at +500 ms: ... it was passed`, and SwitchLanguage fired. The OS sees CapsLock go down, so it toggles
    CapsLock on.
  - p5 (roadmap 2.3's own acceptance sentence, "Holding `Win+Alt+T` while the foreground window changes keeps the T repeats
    suppressed", as an Immediate binding): master ok. Lane fails with `repeat T at +500 ms: ... it was passed`.

  So master meets that acceptance for Immediate bindings and the lane breaks it. Realistic trigger: an Immediate `Launch` or
  `OpenUrl` hotkey held past the typematic delay, where the opened window takes the foreground (roadmap 2.3 "Why"
  names exactly this).

  The lane's docs state the opposite without a caveat:
  - `README.md:355-359`: "swallowed exactly when its key's first key-down was ... never leaks repeats after a swallowed press".
  - `CHANGELOG.md:87-97` (Fixed, KS-06).
  - `src/engine/mod.rs:117-121` (rule 5).
  - cut.md addendum: "the bit the repeat rule reads always describes the key's latest key-down". That is false after a
    foreground change.

  Refutation attempts that failed:
  - No guard in the adapter re-marks held keys.
  - No scenario pins repeat-after-focus-change either way. Only `foreground_change_forgets_pending_key_up_suppression`
    (key-up only) and the chord test at `scenarios.rs:730` exist.
  - Roadmap 2.3 owns un-clearing the bits. But the roadmap does not license a lane to regress master on 2.3's own acceptance,
    so this is not out of scope.

  One fact would settle the fix shape (the architect's call): whether "no record" (cleared by a focus change) must be told
  apart from "key-down passed". With one bit they are the same state. The I2 scenario
  `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` needs "passed" to win, and p1/p2/p5 need "no record"
  to fall back to the binding (or need the bit kept). Alternatively, narrow the docs and pin p1/p5 as known defects for 2.3.
  That is the orchestrator's decision, not a refutation.

- **I1(c) DEFERRED-OUT-OF-SCOPE** (KS-29, roadmap 2.3). The lane's behaviour is as stated: p3/p3b on the lane give
  `down L: passed, fired [SwitchLanguage]` after a lost L key-up, with or without a focus change. The reviewer's comparison
  "master fired and suppressed it" is wrong. Master fires on the Alt key-down and swallows Alt (`down Alt: suppressed, fired
  [SwitchLanguage]`), then passes L unfired, because the KS-14 resync drops the swallowed Alt. That is the audit's KS-29
  description. Both refs fire once and leak a key, so this is not a lane regression. The phantom non-modifier slot is KS-29,
  owned by roadmap 2.3 (`reset_transient_state` resyncs all 256 keys on unlock).

- **M1 NOTED.** Reproduced exactly by p4 (`Ctrl+Shift`, `Hold(1000)`: down Ctrl, down LShift, +100 up Ctrl, +500 repeat LShift
  passed, +800 up LShift **suppressed**). The result is identical on master, so the defect is pre-existing (KS-14). The C9
  comment and README:355-359 overclaim for gesture-bound keys. Belongs to the KS-14 lane, or to narrowed docs.

- **M2 NOTED.** Confirmed: the trailer of `965ac4d` is `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. CLAUDE.md
  requires `Claude Opus 5.5`. Process note for the orchestrator (no amend under the git flow).

- **T-note (tester "deviation from the brief wording") NOTED, not a defect.** The spec says a combo "fires or arms ONLY when
  the normalised key just pressed belongs to its combo". That is a necessary condition. `is_trigger_key`
  (`src/engine/matching.rs:43-48`) satisfies it with a stricter rule (cut D2/Q1). It also matches roadmap A ("fires only if
  the last key pressed belongs to it"). The brief's acceptance line "Shift+CapsLock fires ToggleCapsLock" holds for
  Shift-then-CapsLock (`shift_capslock_fires_toggle_capslock_with_either_shift`). Owner confirmation of the CapsLock-then-Shift
  order stays open.

## Probe matrix (pass = master's behaviour reproduced)

| Probe | master e67e5cb | lane 965ac4d |
|---|---|---|
| p1 Immediate PressKey repeat after focus change | ok | FAILED: repeat F13 passed (fired) |
| p2 default CapsLock repeat after focus change | ok | FAILED: repeat CapsLock passed (fired) |
| p5 roadmap 2.3 Win+Alt+T repeats after focus change | ok | FAILED: repeat T passed |
| p3 lost L key-up + focus change, then Ctrl+Alt+L | FAILED: fires on Alt (Alt swallowed), L passes unfired | FAILED: L passed (fired) |
| p3b same without focus change | same as p3 | same as p3 |
| p4 reviewer M1 | FAILED: up LShift suppressed | FAILED: up LShift suppressed |

Scratch only: `D:/tmp/ks-orch/p0-A2-matching/triage/` (exported trees, probes, target dir). No repository file touched, no git
state changed.
