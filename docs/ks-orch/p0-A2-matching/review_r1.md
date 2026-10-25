# Code review: p0-A2-matching @ c609a69 (+ tester commit 965ac4d)
CHANGES_REQUESTED (Critical 0, Important 1, Minor 2)

Scope: `git diff master...fix/matching` (master = merge-base `e67e5cb`), developer head `c609a69` (10 commits), plus the
tester's `965ac4d` ("Tests:", `src/engine/scenarios.rs` +44, test-only), which landed on the branch during the review.
Read through `git show`/`git diff` only: `src/engine/{mod,matching,keys,chords,sequences}.rs`, `src/data/{key_spec,key_combination,binding,vk_name}.rs`,
`src/config/{loader,parsing,schema}.rs`, `src/system/registry.rs`, the hook diffs (tests only), every new scenario, README,
`default_config.ron`, CHANGELOG, and `cut.md` (incl. the developer addendum), `critique.md`, `impl.md`, `pack_r1.md`.
Master's `on_key_down` (`git show master:src/engine/mod.rs`) was read for every regression comparison.

## Gate: fmt ✅ · clippy ✅
Run on `git archive` exports (not the shared work tree), separate target dir:
- `c609a69`: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0.
- `965ac4d`: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -v -- -D warnings`: clippy-driver ran on lib,
  both bins and their test targets, exit 0.
`cargo test` not run (tester owns it).

## Remarks

### 🔴 Critical
None.

### 🟡 Important

#### I1. The repeat rule trusts a `blocked` bit that `on_foreground_changed` wipes and a repeat flag that a lost key-up makes stale: Immediate bindings now leak repeats that master swallowed (KS-06, KS-29, R1)
**Where**: `src/engine/mod.rs:427-434` (new rule), together with the unchanged `src/engine/mod.rs:298-301`
(`on_foreground_changed` -> `clear_all_blocked`) and `src/engine/keys.rs:74-79` (`press` is the repeat detector; non-modifier
slots are never resynced, KS-29). Docs that state the opposite: `src/engine/mod.rs:38-41`, `:117-121`, `README.md:355-359`,
`CHANGELOG.md:87-96`.  **Confidence**: CONFIRMED for the engine path (traced against both trees); the real-world trigger in (c) is PLAUSIBLE (KS-29 is "not independently verified").

**Problem**:
```rust
if was_repeat {
    decision.suppress = self.keys.is_blocked(vk.0);   // mod.rs:434, overrides every binding
```
On master a repeat of an Immediate key was suppressed whenever the binding still matched: the Immediate arm set `suppress` and
the tail re-marked the key (`master:src/engine/mod.rs:311-319`, `:409-411`). The new rule ignores the binding and reads only
the bit. Two existing paths leave the bit clear while the key's press was swallowed, or make a fresh press look like a repeat:
- `on_foreground_changed` clears every bit, including the bits of keys that are still held and repeating.
- A non-modifier whose key-up the hook never saw stays `active`, so its next real press counts as a repeat (KS-29), and its
  bit is whatever the old press left.

The binding still fires on that repeat (Immediate fires on repeats), but the key now passes. Fire and pass in the same
decision is exactly the mix that rule 5 was written to remove.

**Failure** (each traced line by line):
- (a) Existing scenario `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` (`scenarios.rs:136`,
  `( keys: "F13", action: PressKey("Left") )`) with `.foreground_changed()` inserted after the first `down F13`. The next
  `repeat F13` fires `PressKey(Left)`. On master it is suppressed; on the lane it is **passed**, and so is `up F13`. In use:
  hold a `PressKey` remap such as `"CapsLock+J" -> PressKey("Down")` (holding is the intended use, since repeat is on by
  default for `PressKey`). If the focus changes mid-hold, the new window receives the injected Down presses plus raw `j`
  repeats.
- (b) Default config: hold CapsLock past the repeat delay. A foreground change happens while it is held. The next CapsLock
  repeat matches `"CapsLock"`, fires `SwitchLanguage` and passes, because the bit is clear. Windows sees CapsLock go down and
  turns CapsLock on, which is the KS-06 symptom that the lane's CHANGELOG lists as fixed. Master suppressed this repeat.
- (c) KS-29 path: a `Ctrl+Alt+L` binding (Immediate, `Launch`); Win+L locks the PC and L's key-up goes to the Winlogon
  desktop. After unlock, the next Ctrl+Alt+L counts as a repeat of L. The lane runs `Launch` and also delivers Ctrl+Alt+L to
  the application (the old L press passed, so its bit is clear). Master fired and suppressed it.

No scenario combines `.foreground_changed()` or `.up_unseen()` with `.repeat()`, so the suite cannot see any of this.

**What to do**: decide what a repeat does when its key's suppression record was dropped by a foreground change, or when the
"repeat" is really a new press after a lost key-up. Then pin each case with a scenario. Two scenarios are needed: (a) above,
expecting the repeat suppressed; and `down F13` / `up_unseen F13` / `foreground_changed` / `down F13` with an Immediate binding.

The fix must keep the I2 outcome: a repeat whose key-down really reached the OS passes. It must also stay consistent with
`foreground_change_forgets_pending_key_up_suppression` (`scenarios.rs:173`), or change that test with a stated reason.
Possible directions:
- do not clear the bits of keys that are still held;
- let the binding's request decide a repeat that has no record;
- resync non-modifier `active` slots on a foreground change.

Choosing among them is the architect's call. Until then, the README and CHANGELOG sentences "exactly when its key's first
key-down was" and "never leaks repeats after a swallowed press" are false.

### 🟢 Minor

#### M1. A Tap/Hold still pending at key-up swallows the release of a modifier that the C9 clearing clause just let through (pre-existing; claim overstated) (KS-06, KS-14)
**Where**: `src/engine/mod.rs:437-443` (pass -> `take_blocked`) against `src/engine/mod.rs:463-467` (key-up suppressed whenever
a Tap fired or a Hold is pending, whatever the bit).  **Confidence**: CONFIRMED (traced; identical outcome on master).

**Failure**: `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )`. The steps:
1. Down LCtrl: passed.
2. Down LShift: suppressed, Hold armed under LShift, due at +1000.
3. +100, up LCtrl: the resync erases LShift (the OS never saw it).
4. +500, LShift auto-repeat: `press` returns false, so it is a first key-down. `"Ctrl+Shift"` no longer matches, so it passes
   and `take_blocked` clears the bit. The OS now has LShift down.
5. +800, up LShift: `release_hold` returns `Cancelled`, so the key-up is suppressed. **Shift is stuck in Windows.**

The same path works for `Tap(n)` when n is longer than the typematic delay. Master reaches the same stuck state, so the lane
does not introduce it. But the C9 comment ("This key-down reaches the OS, so its repeats and key-up must too"), README:355-359
and the KS-06 CHANGELOG entry state the opposite. The addendum ("C9 only stops it from leaving a key stuck") is also wrong for
gesture-bound keys.

**What to do**: either narrow the claims, or hand this exact scenario to the KS-14 lane as a pinned known defect (KS-14's
"sync skips blocked keys" keeps step 4 a repeat and closes it).

#### M2. Tester commit carries the wrong trailer
**Where**: `965ac4d` message: `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.  **Confidence**: CONFIRMED (`git log --format='%(trailers)'`).

**Problem**: `CLAUDE.md:104-105` and `.claude/workflows/ks-lane.js:107` require `Co-Authored-By: Claude Opus 5.5
<noreply@anthropic.com>` on every commit. The results-analyst counts exactly that string
(`.claude/agents/results-analyst.md:152`), so this commit will fail its check.

**What to do**: add a follow-up note or commit per the git-flow rules (no amend), or record the exception for the tester role.

## Checklist walked (no reachable failure found)
- **Plan conformance**: D1-D11 are implemented as cut; C7-C9 are justified in the addendum and cited in the CHANGELOG. The
  trigger-key rule is D2 (non-modifier trigger) per Q1. The loader rejects `allow_extra_modifiers` on sequence/chord, and
  overlapping chord keys are a load error.
- **Acceptance**: every brief line maps to a scenario that fails without its fix:
  - CapsLock held + `a`: T10, T11.
  - CapsLock vs Shift and the flag: T15, T16, plus tester `deferred_binding_*`.
  - Ctrl+J vs Ctrl+Shift+J: T17, T18.
  - LShift/RWin/Shift: T1-T4.
  - Shift in a chord: T5.
  - Shift+CapsLock: T22.

  I re-traced T10 (C2 superset arm) and the C8 scenario (repeat rule alone red, clearing clause green). Roadmap A "a repeat of a
  suppressed key is suppressed, key-up exactly once": `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired`.
- **Unsafe**: none added. `#![forbid(unsafe_code)]` is kept in `src/engine`. The hook production code is unchanged (the diff
  touches only `mod tests`).
- **R1 hot path**: matching is table lookups through `slot()` (`< 256`). No allocation in
  `is_trigger_key`/`keys_down`/`modifiers()`. The chord arm allocates as before. No new lock or Win32 call. No panic path.
- **R2 sentinel**: no new `SendInput`. Macro `Combo` output keys go through `parse_key_list` and are unchanged
  (`"Win"` -> `VK_LWIN`, `"Shift"` -> `VK_SHIFT`). `get_system_combination` feeds only auto-blockers, never `SendInput`.
- **R3/R4**: no new static, atomic, `RefCell` or lock. The engine is reached only via `with_engine`.
- **R5**: `with_allow_extra_modifiers(self, bool)`. `KeyCombination::new/with/from_keys` keep their consuming shapes.
- **R6**: `KeySpec`/`Modifier`/`KeyCombination` Display have explicit arms. The `Binding` suffix covers both line forms. The
  registry INFO line uses Display.
- **R7**: `key_count` = number of `KeySpec`s; the sort is untouched; one publish path.
- **R8**: one canonical `KeySpec`, compared against the event's sided code in combos, steps and chords. `AnySide` also
  accepts the generic code, and the generic resync slot counts for its family (stale generic after a passed key-up is
  overwritten by the next event's resync before matching).
- **R9**: registry reads only; `win_with` = `AnySide(Win)` + key.
- **R10**: untouched.
- **Suppression symmetry**:
  - chord `Completed` marks only `seen` codes, each suppressed as `Pending` (chords skip repeats);
  - `last_step_keys` drops modifiers (fixes the `Win` case);
  - C9 clears a stale bit on a passed first key-down;
  - the auto-blocker after Alt-release case is now balanced.

  Remaining gaps: I1, M1, and the pre-existing KS-32 rest (`"A+S"`, open point 5).
- **I2/KS-10**: a repeat that starts matching after a passed key-down fires an Immediate action and passes. This is deviation
  2, KS-10 (roadmap 1.1); master fired on those repeats too.
- **Hook tests**: the three `HookEnv` tests opt into `allow_extra_modifiers` (D10). No test installs a hook or injects input.
  No `#[ignore]` needed.
- **Docs**:
  - README combo syntax, sided modifiers, trigger key, exact modifiers, flags row, features, and the LShift warning.
  - `default_config.ron` comments.
  - CHANGELOG Added/Changed (`**Breaking:**`)/Fixed with KS IDs, Keep a Changelog.
  - `readme_matching_examples_load` reads the real README.

  The doc inaccuracies are under I1 and M1.

## Positive
- `KeySpec` fixed at parse time removes side folding, the 16-key buffer, the 256-slot scan and the per-event chord `Vec`, and
  it keeps a trigger-only notion away from `SendInput` (D1, D7).
- Every red-first is an assertion red with the predicted text. The guards are real, and the matching unit tests run against
  the production `KeyState` (critique M2 done properly).
- C9 closes several pre-existing stuck-key paths that the critique found, including two that master also had: the
  auto-blocker Alt-release, and the Alt+Shift blocker pressed Shift-first, where master left Alt stuck.

## Open questions
1. **Book pages**: CLAUDE.md:163-164 requires `book/src/config/*.md` in the same lane, but the cut deferred them to the phase
   book sync (Q2). The brief's spec does not list the book. The orchestrator should confirm the deferral. The cut's Docs-impact
   list plus the KS-06 rows in impl.md is the work list.
2. **CLAUDE.md "Layout"** (`src/data/` lacks `key_spec.rs`, the engine list lacks `matching.rs`) and reviewer **R8** (folding,
   "Any Windows key is `VK_LWIN`") are stale after merge: cut Q4, orchestrator.
3. **KS-29**: the trigger-key rule removes KS-29's "phantom key fires unrelated combos" half. Whether to mark KS-29 partly
   fixed depends on how I1 (c) is resolved.
