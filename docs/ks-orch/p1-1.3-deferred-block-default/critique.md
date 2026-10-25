# Architecture critique: p1-1.3-deferred-block-default / roadmap item 1.3 (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 2)
Plan: D:/tmp/ks-orch/p1-1.3-deferred-block-default/cut.md   Code read at: master @ 1afa720 (.git/refs/heads/master)

Read: the whole cut; src/engine/{mod,keys,gestures,matching,testing,properties}.rs, the scenarios it names
(:900-928, :1044-1067, :1337-1356, :2047-2213, :2405-2434, :2733-2758), src/config/{schema,loader}.rs,
src/data/{binding,key_combination,key_spec,trigger}.rs, src/hook/keyboard_hook_callback.rs,
src/system/system_function.rs:42-160, default_config.ron, README.md:326-395, CHANGELOG.md:1-80/:222-281,
roadmap.md:180-380, book/src/config/{kinds,triggers-conditions,structure,matching,errors,ron-syntax,actions,known-issues,README}.md
at the cited lines, docs/WORKFLOW.md (red-first commits are allowed with the failing line in the body).

## Critical (implementation must not start)

None.

The core claim holds. The suppression invariant follows the per-key record and not the binding.
A key-down that passes is recorded `Passed` (mod.rs:535-542). Its repeats pass (mod.rs:530-531). Its
key-up passes whatever gesture resolves (mod.rs:567-570). Nothing can turn `Passed` into
`Suppressed` while the key is held: `mark_completed` skips `Passed` (keys.rs:168-175), `forget_blocked`
keeps it (keys.rs:212-218) and `reset_from_os` re-seeds held keys as `Passed` (keys.rs:117-130). So D3
(three one-line gates at mod.rs:431/445/459) cannot leave a key stuck in Windows.

## Important (must be resolved; options may be discussed)

### I1. Q1 (no interrupt rule) is not "non-blocking": with the new default, the advertised example and every pass-through modifier gesture misfire during ordinary typing, and the template, CHANGELOG and M1 hide it
Where: cut D6, Q1, touch-set rows for `default_config.ron:268-275` and the CHANGELOG text, manual check
M1; src/engine/gestures.rs:94-121 (Tap), :200-219 (DoubleTap); src/engine/mod.rs:368-508 (a key-down
of another key never touches the gesture tables).
Problem: a pending Tap/Hold/DoubleTap is never cancelled by another key. While the gesture key was
swallowed, that hardly mattered, because the key was a dedicated gesture key. This lane makes the
gesture key work as a normal modifier by default. That turns "another key pressed in between" from an
edge case into the main use case. The cut's reasons for deferral do not hold up:
- Its pinned counter-example, `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`
  (scenarios.rs:1044-1067), is a swallowed CapsLock (non-modifier, default `true` after this lane). A rule
  limited to gestures whose key-down passed would not touch it.
- Roadmap 2.2 (roadmap.md:360) puts Tap, Hold and DoubleTap *on one key* into one state machine. It says
  nothing about cancelling a gesture when a *different* key goes down. "2.2 owns it" is therefore a scope
  addition to 2.2, not an existing home.
- The answer changes this lane's own deliverables: commit 2's tests, the template comment, the CHANGELOG
  "Fixed" text and M1. So it is not independent of the commits, as the cut's Verdict says.
Consequence (all traced, CONFIRMED):
- Template example `( keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250) )`. The user types
  `Option<T>` and re-presses the left Shift for `<` and for `T`. LShift down at t=0 arms the DoubleTap
  (gestures.rs:213-217). `,` down and up pass and do not touch it. LShift down at t~150 is before the
  deadline, so it fires `ToggleCapsLock` (gestures.rs:207-211). The injected CapsLock lands while Shift
  is held. `T` then comes out as `t`, and CapsLock stays on for the rest of the line. The new template
  comment ("every left-Shift press still reaches applications") and the CHANGELOG "Fixed" entry ("the
  left Shift types capitals again") give no warning, and M1 has no step with Shift re-pressed quickly.
- `( keys: "LCtrl", action: PressKey("Escape"), trigger: Tap(200) )`, the classic "Ctrl alone = Esc".
  Every quick Ctrl+C or Ctrl+V (Ctrl released within 200 ms) also sends Escape (mod.rs:555-560 fires the
  Tap on LCtrl's key-up). The cut's AltGr bullet shows the same thing: a quick AltGr character fires an
  `"LCtrl"` Tap.
- Test j's own config `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )`. Holding
  Ctrl+Shift to select words with Right for more than 1 s switches the layout in the middle of the
  selection.
Confidence: CONFIRMED (plan D6/Q1; gestures.rs:94-121, :200-219; mod.rs:421-460, :546-571)
Breaks: the lane's acceptance claim that the example works (roadmap.md:259; smoke #6, roadmap.md:310)
as the owner will actually use it; traceability (CHANGELOG "Fixed" overstates).
What is needed: get Q1 answered before the developer starts. Whether a pass-through gesture survives
other keys is a behaviour question for the owner, so the orchestrator must ask. Whatever the answer,
the lane's artefacts must match the behaviour that ships:
- If the rule is deferred: put the caveat at the template example and in the CHANGELOG
  (Changed/Breaking or Fixed), not only in README and the book.
- Add a known-issue scenario that pins today's misfire, with a comment like test l's
  "OR rule until 1.6", so that the follow-up is red-first.
- Add an M1 step with Shift re-pressed quickly (`<T>`, `A B`), so the owner sees it before signing off
  smoke #6.
Options:
- (a) Bring a narrow interrupt rule into 1.3. It would cancel a pending gesture whose key-down passed
  when another key goes down. It leaves the swallowed-key rollover at scenarios.rs:1044-1067 alone, and
  its red-first is the `<T>` script above.
- (b) Defer it to a follow-up item, with the documentation, the pinned scenario and the M1 step above.

## Minor (improvements, not blockers)

### M1. D7: an implicit default overrides an explicit `block_original_combo: true`, including on the binding itself; matching.md §15.3 not in the touch set
Where: cut D7, Q2; src/config/loader.rs:285-306 (the dedup counts "the flagged one included", :298-301);
book/src/config/matching.md:76 ("That binding takes the hotkey over").
Problem: the cut's example has the flag on a different binding. The sharper case is the flag on the
replacing binding itself.
Consequence: `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200), block_original_combo: true )`
with the Alt+Shift hotkey:
- The dedup drops the blocker because of this same binding.
- `blocks_by_default` resolves `false` (modifier-only, deferred).
- LShift passes, so Windows switches forward and the binding switches backward. Each tap nets no
  change.
- Before the lane it switched backward once.
The user asked for the hotkey to be kept from Windows on that very line, and an absent field now
cancels that request. matching.md §15.3 is where a user reads how the takeover works, and it still says
`block_default` "plays no role". The page is not in commit 4.
Confidence: CONFIRMED (loader.rs:298-306; plan D2/D3)
Breaks: Q2 rationale ("explicit wins", D7 Rejected) applied one-sidedly.
What is needed: choose one of these:
- Resolve an absent `block_default` to `true` on a binding that itself sets `block_original_combo: true`
  (local to `raw_binding_to_binding`; the spec says nothing about this flag).
- Name this exact case in CHANGELOG, known-issues row 65 and README.
Either way, add matching.md §15.3 (`:76`) to commit 4.

### M2. The KS-13 "OR rule" pages still say it concerns `Immediate` bindings only
Where: cut D3 "OR semantics", test l; book/src/config/kinds.md:182, known-issues.md:36 (row 10),
triggers-conditions.md:44, actions.md:260. Of these, commit 4 touches none (its kinds.md rows are
:170, :177, :193, :197).
Consequence: `( keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false ), ( keys: "F13", action: ToggleCapsLock, trigger: Hold(200) )`.
F13 is swallowed on every press, which test l pins. The pages tell the user that only another
`Immediate` binding can override `block_default: false`.
Confidence: CONFIRMED (mod.rs:417-419 plus the new gates OR into one `decision.suppress`; kinds.md:182)
What is needed: extend the KS-13 warnings (kinds.md:182, known-issues row 10) to deferred bindings in
commit 4.

## Strengths to keep

- D1 resolves the default once in the loader. The engine reads one bool and holds no policy, the rule
  sits where unit tests reach it, and builder order cannot overwrite an explicit value.
- D2 gives one definition of "made only of modifiers", shared by the trigger-key rule and the default.
- D3 is the smallest correct change. The key-up and repeat rules are untouched. The pre-placed
  `key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` (mod.rs:622-656) becomes reachable,
  as its comment foresaw.
- D9 keeps the KS-14 and KS-06 regressions on the swallow path explicitly, instead of flipping them, and
  adds pass-path twins (h, i, j, m1-m3). The property configs then cover both paths, and every new key
  (F15) is in `KEYS` (properties.rs:56-59).
- The red-first predictions are exact. I traced a-m3, flip and L1-L4 against testing.rs:595-611
  (message format), loader.rs:242-243 (`IMPLICIT_SOME`) and ron-syntax.md:91. The guards f, g, l and L2
  can fail under a wrong fix.
- No hook, thread, lock, timer, window, COM or `SendInput` change. The adapter is untouched (R1, R2,
  R3, R10 are clean).

## Open questions for the architect / orchestrator

1. Merge order against 1.2. Until 1.2's mask lands, an existing `( keys: "LWin" | "RAlt", ..., trigger: Tap(..) )`
   without `block_default` now opens Start or arms the menu bar on every tap. In Notepad the next typed
   letter becomes a menu accelerator. This follows the spec and is documented (CHANGELOG Breaking, M3).
   Should the phase merge 1.2 first, or is the interim regression on `master` acceptable? Settle it in
   the phase plan, not in this cut.
2. M3, second expectation. "LWin+E types `e` (... item 1.2)": the swallowed LWin comes from
   `block_default: true` itself. 1.2's mask would not make Win+E work. Check the wording so the owner
   does not log it against 1.2.

## Absent topics (walked, not applicable)

- §2 thread affinity, §9 process boundary and §10 input synthesis: nothing changes. No install or
  uninstall, no new thread, pipe, token or `SendInput` builder.
- §7 reload leniency: unchanged. The loader resolution runs on main or `config-watcher` as before. The
  schema change is RON-compatible: `true`, `false`, `Some(..)` and `None` load. The default change is
  declared Breaking.
- §8 atomics and `unsafe`: none added.
- §11 builders: `blocks_by_default` is an associated fn, the constructors keep two arguments, and there
  is no new variant.
