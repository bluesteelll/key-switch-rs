# Code review: p0-A2-matching @ c8ca4a5 (round 2; delta c609a69 + tester 965ac4d .. fix/matching)
APPROVED (Critical 0, Important 0, Minor 2)

Scope: `git diff c609a69..fix/matching` (4 commits: 965ac4d tester tests, c59a110 red-first, 938df54 fix, c8ca4a5 docs; 5 files,
+286/-29: `src/engine/{keys,mod,scenarios}.rs`, README.md, CHANGELOG.md). Read via `git show fix/matching:<path>` /
`git diff` only: `src/engine/keys.rs` and `src/engine/mod.rs` in full, every new scenario, `gestures.rs`
(`resolve_tap`/`release_hold`/`fire_hold`), `sequences.rs` (`last_step_keys`), README:300-375, CHANGELOG Unreleased, and
`master:src/engine/mod.rs:280-440` for regression comparison. Inputs: pack_r2.md, triage_r1.md, fix_r1.md, review_r1.md.

## Gate: fmt ✅ · clippy ✅
Fresh `git archive c8ca4a5` export (`<scratchpad>/rev2_c8ca4a5`), own target dir (`<scratchpad>/target-rev2`):
`cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0, no warning (re-run after
`touch src/lib.rs`: `Checking key-switch-rs v0.1.0 (...rev2_c8ca4a5)` / `Finished`). `cargo test` not run (tester owns it).

## Round-1 remarks
- **I1 ✅ closed.** `LastDown { Unknown, Passed, Suppressed }` (`keys.rs:41-51`) separates "forgotten by a focus change" from
  "passed". The repeat tail (`mod.rs:446-450`) suppresses on `Suppressed`, passes on `Passed` and otherwise keeps the
  bindings' request. (a) and the triage's p1 are pinned by `scenarios.rs:1408`, (b)/p2 by `:1433`, p5 (roadmap 2.3's
  Immediate acceptance) by `:1456`, and the requested lost-key-up scenario by `:1479`. The I2 outcome is kept across a
  focus change: guard `:1497`, `Passed` survives `clear_all_blocked` (`keys.rs:140-146`), unit test `keys.rs:270`.
  I traced mutation sensitivity for each test:
  - re-marking an `Unknown` repeat (master's rule) fails `:1408`/`:1433`/`:1456` at `up ... expect_passed`;
  - forcing an `Unknown` repeat to pass (965ac4d's rule) fails them at the first `repeat`;
  - clearing `Passed` in `clear_all_blocked`, or reverting `mark_passed` to `take_blocked` (`mod.rs:460`), fails `:1497`.

  The doc claims were narrowed (README:355-367, CHANGELOG:98-104, rule 5 `mod.rs:117-124`). One residual overclaim is
  reported as m1 below.
- **I1(c)**: DEFERRED by triage (KS-29, roadmap 2.3). Unchanged; behaviour is the same as 965ac4d (a stale `Passed`
  record, `mod.rs:448-449`).
- **M1 ✅ closed** as a docs narrowing: README:364-367, CHANGELOG:102-104 and the `mod.rs:454-459` comment. The code fix
  stays with KS-14.
- **M2 ❌ open (process only).** `965ac4d` still carries `Co-Authored-By: Claude Sonnet 5.5`. The developer cannot fix it
  without an amend, so this is for the orchestrator. All three round-2 commits carry the Opus trailer.

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. The first gap bullet says the release "reaches Windows" after a focus change; for a key with a Tap or Hold still pending, it is swallowed after its repeats passed (KS-06, roadmap 2.3; docs only, pre-existing behaviour)
**Where**: `README.md:360-363`, `CHANGELOG.md:98-102` ("its release reaches Windows"), and rule 5 at `src/engine/mod.rs:121-123`
("leaves no mark, so its key-up passes"). Mechanism:
- `mod.rs:354-357`: the Hold arm does `continue` on a repeat;
- `mod.rs:446-450`: an `Unknown` record leaves `suppress` false;
- `mod.rs:481-485`: a pending gesture makes the key-up suppressed;
- `gestures.rs:162-164`: `Cancelled` before the due time.

**Confidence**: CONFIRMED (traced; master and 965ac4d trace identically, and master leaks the repeat even without the
focus change).

**Problem**: after a focus change, only an `Immediate` binding can ask to swallow a repeat. Deferred, chord and sequence
arms skip repeats, so the repeats of a deferred-bound key pass. A Tap or Hold still pending at the key-up then swallows
the release. The docs promise the opposite pairing. fix_r1.md "Open points" names this case, but README, CHANGELOG and
rule 5 do not.

**Failure**: `( keys: "CapsLock", action: SwitchLanguage, trigger: Hold(1000) )`, Windows repeat delay 500 ms.
1. Down CapsLock at +0: suppressed, Hold armed, record `Suppressed`.
2. A foreground change at +100: record becomes `Unknown`.
3. Repeat at +500: the Hold arm skips it and no Immediate binding matches, so the repeat **passes**. Windows sees CapsLock
   go down and toggles it.
4. Up at +700: `release_hold` returns `Cancelled`, so the key-up is **suppressed**. Windows keeps CapsLock down.

The same happens for `Tap(n)` with n longer than the repeat delay. README:362-363 says the release reaches Windows.

**What to do**: extend the first gap bullet, the CHANGELOG sentence and rule 5 with the deferred case. Repeats of a
`Tap`/`Hold`/`DoubleTap`/chord/sequence key pass after a focus change, and a still-pending `Tap` or `Hold` swallows its
release. Alternatively, name it as part of roadmap 2.3. No code change is asked of this lane: the behaviour is master's,
narrowed by the lane.

#### m2. (= r1 M2, carried) Tester commit trailer
**Where**: `965ac4d` (`Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`).  **Confidence**: CONFIRMED
(`git log --format='%(trailers)'`).

**Failure**: the results-analyst's trailer count fails for this commit.

**What to do**: orchestrator records the exception, since no amend is allowed.

## Checklist walked (no reachable failure found)
- **Scope of the behaviour change.** Against 965ac4d, only a repeat whose record is `Unknown` behaves differently. Two
  states lead there:
  1. A `Suppressed` record that a focus change cleared on a still-held key.
  2. A modifier made `active` by the resync without a key-down the hook saw: a modifier held at daemon start, or a
     key-down that went to the secure desktop. A suppressed modifier never reaches this state, because the resync erases
     its `active` slot, so its next event is a first key-down.

  A non-modifier can become `active` only through a recorded key-down. In state 2 the OS has the key down: suppressing a
  repeat is harmless, and the key-up passes. Other paths are unchanged: `Passed` vs `Unknown` after a passing first
  key-down only differ on a later repeat or on a focus change, and the outcome is the same as 965ac4d.
- **Regressions by binding kind.**
  - On a repeat, Tap, Hold, DoubleTap, Sequence and Chord skip (`mod.rs:343-345`, `:355-357`, `:369-371`, `:385-387`,
    `:410-412`), so an `Unknown` repeat passes for them, as on 965ac4d and on master.
  - Immediate now swallows, as on master.
  - Sequence and chord completion still `mark_blocked` (`:404`, `:423`). Completion always suppresses the event, so a
    first key-down cannot pass and be re-marked as suppressed in the same decision.
- **Suppression invariant.**
  - A key-up is suppressed only on a `Suppressed` record or a pending Tap or Hold (`mod.rs:481-487`).
  - `Suppressed` is written only by a suppressed first key-down (`:452`), or by a sequence or chord completion for keys
    that the completion suppressed.
  - A deliberate exception: an `Unknown` key-up passes (a lone key-up). This is the pre-lane trade-off pinned by
    `foreground_change_forgets_pending_key_up_suppression` (`scenarios.rs:173`); fix_r1.md "Deviations" documents it.
  - The pre-existing KS-32 rest (a last step such as `"A+S"` re-marking a passed A) is unchanged from r1.
- **R1 / hot path.**
  - Keyboard path: `is_blocked`, `is_passed`, `mark_passed` are O(1) and bounded by `slot` (`keys.rs:36-39`); no
    allocation, lock or panic path added.
  - `clear_all_blocked` is a 256-step loop that runs only from the WinEvent path (`keyboard_hook_callback.rs:259`), not
    from the keyboard callback.
- **R2**: no `SendInput` or sentinel touched.
- **R3/R4**: no static, atomic, `RefCell` or lock added. `#![forbid(unsafe_code)]` is kept (`mod.rs:138`), and the round
  adds no `unsafe` (hence no new `// SAFETY:` needed).
- **R5-R7, R9, R10**: not touched by the round-2 diff. Hook, window, timer and COM ownership and Win32 handles are
  unchanged.
- **R8**: key identity unchanged (`keys.rs:154-165`).
- **Tests.**
  - The tester's `:1202`/`:1229` fail if the exact rule or `allow_extra_modifiers` were skipped for deferred arms (the
    CapsLock key-down would flip between suppressed and passed).
  - The round-2 scenarios are mutation-sensitive (see I1).
  - No `#[ignore]` added; every test is a pure engine scenario (no hook, no injection).
- **Docs.**
  - CHANGELOG stays under Unreleased/Fixed with KS IDs.
  - The adapter doc `keyboard_hook_callback.rs:244-247` ("forgets which key-ups were due to be swallowed") is still
    accurate.
  - Book deferral: r1 open question 1, unchanged.

## Positive
- The tri-state record is the smallest shape that satisfies both the I2 outcome and p1/p2/p5. The developer's reason for
  recording nothing on an `Unknown` repeat holds: a wrongly passed key-up at worst ends a press the OS never saw, while a
  wrongly swallowed one leaves a key held.
- Each red-first scenario names the user-visible stake (CapsLock turned on, raw F13 in the new window, roadmap 2.3's
  sentence), and the guard pins the other half of the trade.

## Open questions
1. **Lone key-up after a focus change.** The release of a key whose repeats an `Immediate` binding swallowed after a focus
   change now passes; master and 0.1.0 swallowed it. This is by design (fix_r1.md "Deviations"). CHANGELOG:98-102 frames
   the sentence with "still" and "as before", so a reader may not see this as a change.
   - Fact that would settle whether it matters: does Windows act on a key-up whose key-down it never saw for any key
     bindable here? A candidate is PrintScreen; Win and Alt are modifiers and are resynced, so they cannot reach this
     path.
   - If no such key exists, nothing to do.
2. r1 open questions 1-3 (book pages, CLAUDE.md layout and R8 wording, KS-29 partial status) are unchanged and remain for
   the orchestrator.
