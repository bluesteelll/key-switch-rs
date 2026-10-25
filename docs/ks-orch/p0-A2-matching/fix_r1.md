# Fix round 1: p0-A2-matching

DONE. Base 965ac4d, HEAD c8ca4a5 on `fix/matching`, 3 follow-up commits (no amend), tree clean.

Input: `triage_r1.md` (1 CONFIRMED: I1 paths (a), (b) and doc claims; I1(c) DEFERRED; M1, M2, T-note NOTED).

## Items

- **I1 (review r1, CONFIRMED)**: a repeat after a focus change passed and fired, because `on_foreground_changed` clears every
  blocked bit and the C9 repeat rule read a clear bit as "the key-down passed".
  - Red-first, commit c59a110, `src/engine/scenarios.rs:1407-1515`. Each was run alone on 965ac4d plus the scenarios.
    - `immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (reviewer (a), triage p1): red
      `repeat F13 at +500 ms: expected the event to be suppressed, it was passed` -> green.
    - `capslock_held_across_a_focus_change_does_not_reach_windows` (reviewer (b), triage p2, default config): red
      `repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed` -> green.
    - `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed` (triage p5, roadmap 2.3 acceptance, Immediate):
      red `repeat T at +500 ms: expected the event to be suppressed, it was passed` -> green.
    - `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding` (the reviewer's second requested scenario:
      down / up_unseen / foreground_changed / down): red `down F13 at +1000 ms: expected the event to be suppressed, it was
      passed` -> green.
    - Guard `repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change` (the I2 outcome kept across a focus change):
      green before and after. It pins that a focus change does not forget passed key-downs.
  - Fix, commit 938df54:
    - `src/engine/keys.rs:41-71`: `blocked: [bool; 256]` becomes `last_down: [LastDown; 256]`, where `LastDown` is
      `Unknown | Passed | Suppressed`.
    - `:101-147`: `mark_blocked`, `is_blocked` and `take_blocked` keep their names and their meaning for suppressed records.
      `take_blocked` resets the record to `Unknown`. New: `mark_passed` and `is_passed`.
    - `clear_all_blocked` now turns only `Suppressed` into `Unknown`.
    - `src/engine/mod.rs:431-461`, a repeat is decided as follows:
      - record `Suppressed`: suppress;
      - record `Passed`: pass;
      - record `Unknown`: keep what the bindings asked (master's rule) and record nothing.
    - A first key-down records `Suppressed` or `Passed` (`mark_passed` replaces C9's `take_blocked`).
    - Unit tests `clear_all_keeps_passed_key_downs` and `latest_key_down_replaces_the_record_and_take_forgets_it`
      (`keys.rs:269-298`).
  - Docs (rustdoc, 938df54):
    - the module State bullet (`mod.rs:38-41`);
    - rule 5 (`mod.rs:117-124`);
    - the `on_foreground_changed` doc (`mod.rs:298-301`);
    - the repeat and C9 comments (`mod.rs:431-461`);
    - the `keys.rs` module and field docs.

## Design choices inside I1 (constraints that fixed the shape)

The reviewer named three directions. The triage left the shape open.

1. **Keep the bits on a focus change: rejected.** It contradicts the pinned `foreground_change_forgets_pending_key_up_suppression`
   (`scenarios.rs:173`). It is also roadmap 2.3's own item ("In an elevated daemon, `foreground_changed` no longer clears every
   blocked bit").
2. **Resync non-modifier `active` slots: rejected.** It does not fix p1, p2 or p5, because the key is still physically held, so
   its next event is still a repeat. It is also 2.3's `reset_transient_state`.
3. **The binding decides a repeat that has no record: implemented.** This needs a third state, because one bit cannot tell
   "forgotten" from "passed". The I2 scenario needs "passed" to win, and p1, p2 and p5 need "forgotten" to fall back to the
   bindings.

Two sub-choices follow.

- **A focus change forgets only suppressed records.**
  - Passed records stay, so every path that the lane passed before is unchanged: the guard, and p3/p3b (KS-29, deferred).
  - The fix therefore changes behaviour only for keys whose key-down was suppressed before the focus change. Those are exactly
    the confirmed paths.
  - A mutation that also forgets `Passed` fails the guard and `clear_all_keeps_passed_key_downs` (run locally, then reverted).
- **A repeat with no record records nothing, so its key-up passes.** Master re-marked it, which swallowed the key-up.
  - Reason: an `Unknown` record says nothing about whether the OS has the key down. Two examples:
    - a modifier that the resync found held while its key-down went to a higher-integrity window;
    - the auto-blocker `Alt+Shift` matching such an Alt's repeat.
  - In those cases a swallowed key-up leaves the key held in Windows. A passed key-up at worst ends a press the OS never saw.
    That is the trade the pinned `foreground_change_forgets_pending_key_up_suppression` already makes.
  - The scenarios therefore expect `up ... passed` after the repeats.
  - A mutation that re-marks (master's rule) fails `immediate_key_held_...`, `capslock_held_...` and `win_alt_t_held_...` at
    their key-up step (run locally, then reverted).
  - Consequence: the triage's probes p1 and p5, which assert master's suppressed key-up, now fail only at that last step. Their
    repeats are suppressed (see the probe matrix). Roadmap 2.3's acceptance names only the repeats.

## Remarks

- **I1 (CONFIRMED)**: RESOLVED at `src/engine/keys.rs:41-147` and `src/engine/mod.rs:431-461`. Pinned by the four red-first
  scenarios above, the guard and the two `keys.rs` unit tests.
  - Doc claims RESOLVED, commit c8ca4a5. `README.md:355-368` now lists two remaining gaps. `CHANGELOG.md:98-104` (KS-06 entry)
    names the focus-change case and the KS-14 case.
  - The rustdoc claims are fixed in 938df54.
  - The cut-addendum sentence "the bit the repeat rule reads always describes the key's latest key-down" is superseded by the
    tri-state record. `cut.md` is not edited; this report is the correction.
- **I1(c) (DEFERRED by triage, KS-29, roadmap 2.3)**: not changed.
  - p3/p3b on HEAD: `down L at +0 ms: passed, fired [SwitchLanguage]`, the same as 965ac4d.
  - L's earlier key-down passed, and that record is kept across the focus change.
  - The reviewer's single-key variant, a swallowed key-down followed by a lost key-up and a focus change, is now pinned by
    `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding`: it fires and is suppressed.
- **M1 (NOTED, pre-existing KS-14)**: RESOLVED as a docs narrowing; the code fix stays with the KS-14 item of Workstream A.
  - The C9 comment now says a pending Tap or Hold from an earlier press still swallows the key-up (`mod.rs:454-459`).
  - The README second gap bullet and the CHANGELOG KS-06 entry describe the `"Ctrl+Shift"` + `Hold(1000)`, Ctrl-first case.
  - The scenario is not pinned, because the `scenarios.rs` header excludes known defects.
  - Triage p4 on HEAD is unchanged: `up LShift at +800 ms: ... suppressed`.
- **M2 (NOTED, trailer of 965ac4d)**: no developer action is possible. The git flow forbids amend and rebase, so 965ac4d keeps
  `Claude Sonnet 5.5`.
  - All three commits of this round carry `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
  - The exception for the tester commit is for the orchestrator and the results-analyst.
- **T-note (trigger rule stricter than the brief wording, NOTED)**: REFUTED as a defect. It is cut D2/Q1, the behaviour
  satisfies the brief's necessary condition, and the triage agrees. Owner confirmation of the CapsLock-then-Shift order stays
  an orchestrator question.

## Deviations from the cut

- C9's single blocked bit becomes a three-state record (above). D1-D12 are unchanged.
- Behaviour change against master, by design: after a focus change, the key-up of a key whose repeats an Immediate binding
  swallowed passes. Master swallowed it.

## Unsafe / atomics added

None. `src/engine` keeps `#![forbid(unsafe_code)]`. No new lock, allocation or Win32 call.

`clear_all_blocked` loops over 256 entries on a foreground change. That runs in the WinEvent callback under `with_engine`, not
on the keyboard path, and the old code assigned a 256-entry array there.

## Gates (HEAD c8ca4a5)

- `cargo fmt --check`: pass.
- `cargo check --all-targets --locked`: pass.
- `cargo clippy --all-targets --locked -- -D warnings`: pass.
- Tests run:
  - the 5 new scenarios on c59a110: 4 red as listed, the guard green;
  - `cargo test --locked --lib engine::`: 124 passed;
  - `hook::`: 10 passed;
  - `engine:: hook:: readme default_config` on HEAD: 136 passed, 0 failed.
- Full suite: left to the tester.
- Manual ignores added: none. Every case is a pure engine scenario.
- Triage probes on HEAD (scratch export `D:/tmp/ks-orch/p0-A2-matching/fix_r1/tree`, own target dir, output
  `fix_r1/probe_head.txt`). Each probe asserts master's behaviour.

| Probe | 965ac4d | HEAD c8ca4a5 |
|---|---|---|
| p1 Immediate PressKey, repeat after focus change | repeat passed + fired | repeat suppressed + fired; fails only at `up F13 ... passed` (by design) |
| p2 default CapsLock, repeat after focus change | repeat passed + fired | ok |
| p5 Win+Alt+T, repeats after focus change | repeat passed | repeats suppressed; fails only at `up T ... passed` (by design) |
| p3 / p3b lost L key-up (KS-29, deferred) | L passed + fired | unchanged |
| p4 M1 (KS-14) | up LShift suppressed | unchanged |

## Docs

- `README.md:355-368`: the auto-repeat rule, plus two gap bullets (focus change, roadmap 2.3; pending Tap/Hold modifier,
  KS-14).
- `CHANGELOG.md:98-104`: KS-06 Fixed entry narrowed.
- Book: not edited (cut D12/Q2, deferred to the phase book sync).

## Commits

- c59a110 Engine: red-first: a repeat after a focus change follows its bindings (KS-06)
- 938df54 Engine: a repeat whose swallowed key-down a focus change forgot follows its bindings (KS-06)
- c8ca4a5 Docs: the KS-06 entry and README name the focus-change and KS-14 gaps

## Open points / pre-existing warnings

- Pre-existing, unchanged, roadmap 2.3: a deferred binding (`Tap`/`Hold`/`DoubleTap`, chord, sequence) whose key-down was
  swallowed still passes its repeats after a focus change. Deferred arms ignore repeats, so the bindings ask nothing. Master
  and 965ac4d behave the same.
  - A pending Tap or Hold on that key then still swallows the key-up, and the key can be left held. The pending gesture proves
    that the key-down was swallowed, but using it is beyond this item.
- Pre-existing rustdoc warning at `src/data/vk_name.rs:4`: untouched.
- Scratch: `D:/tmp/ks-orch/p0-A2-matching/fix_r1/` (exported tree, target dir, probe output). No repository file is outside
  the commits.
