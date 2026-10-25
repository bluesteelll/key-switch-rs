# Architecture critique: p0-A2-matching, trigger-key rule (KS-04), sided keys (KS-07), exact modifiers + `allow_extra_modifiers` (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 4)
Plan: D:/tmp/ks-orch/p0-A2-matching/cut.md   Code read at: master e67e5cb (clean)

The design is sound: `KeySpec` canonicalises once at parse time, combo, step and chord all compare the event's sided code against it, output keys stay `VIRTUAL_KEY`, and matching becomes table lookups with no allocation. Both Important findings are about suppression symmetry (KS-06, RC2) where this lane turns inert or superset-masked paths into reachable stuck keys. The plan names neither path correctly, and its manual checks walk the owner into both.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. Modifier triggers go live while KS-06 is still open, and M4/M6 lead the owner into a stuck Shift
Where: plan §D2 (cut.md:97-102), §D5 (cut.md:137-148), Risks "LShift DoubleTap example" (cut.md:454-455), Manual checks M4/M6 (cut.md:421, :423), Docs impact C2 Fixed (cut.md:470-471); src/engine/mod.rs:346-349 (DoubleTap skips repeats), :387-390 (Chord skips repeats), :409-411 (`mark_blocked` on a suppressed down), :437 (`take_blocked` on the up); src/engine/chords.rs:87-93 (completion marks keys).

Problem: on master, `keys: "LShift"` and a chord key `"Shift"` never match, so every Shift event passes; the config is inert. After C2 the first LShift key-down is suppressed and its blocked bit is set. Shift's auto-repeats then hit the `was_repeat => continue` of the DoubleTap and Chord arms, nothing suppresses them, and they pass. The key-up finds the bit and is swallowed. The plan warns that these bindings swallow every Shift press until roadmap item 1.3. It says nothing about the repeat leak, which turns "swallowed" into "stuck".

Consequence: uncomment the template's `( keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250) )` as M4 instructs. Hold left Shift for a Shift+click selection, or pause more than about 500 ms before typing a letter. From the typematic delay on, the LShift repeats reach Windows and the release is suppressed. Windows keeps LShift logically down, so every later keystroke is shifted. Pressing RShift does not help. The next LShift press is suppressed again (DoubleTap arms on it), so the state never heals while the binding exists. The M6 chord `["Shift", "Space"]` does the same in two ways:
- Shift held alone: the key-down arms the chord as Pending and is suppressed, then the repeats leak.
- Space held past the delay after the chord completes: `Completed` marked Space as blocked, so its repeats pass and its release is swallowed.

Confidence: CONFIRMED (cut.md:421/423 + the engine paths above; the scenario harness can reproduce it with `.repeat("LShift")`).
Breaks: suppression invariant (CLAUDE.md coding principle 2), KS-06, root cause "Suppression and repeat state are not kept symmetric" (book/src/audit.md:82-86).
What is needed: state the dependency on the KS-06 lane (symmetric repeat suppression, roadmap A `book/src/roadmap.md:81`). Until that lane lands, the owner must not meet this path unwarned.
Options:
- (a) Order this lane after the KS-06 lane.
- (b) Keep the order, but:
  - M4 and M6 say "release within the repeat delay" and give a recovery: remove the binding, then tap LShift;
  - the template and README warnings for the LShift example and for modifiers in chords mention the hold case;
  - the CHANGELOG KS-07 entry qualifies "works";
  - a known-defect scenario is handed to the KS-06 lane to flip.

### I2. Exact matching opens a pass-then-suppress path on auto-repeat; the Risks entry names the other, rarer direction
Where: plan §D3 (cut.md:107-122), Risks "New KS-06 path" (cut.md:443-445), M2 (cut.md:419); src/engine/mod.rs:291 (`press` marks the key held whatever the decision), :311-319 (Immediate fires and suppresses on repeats), :409-411, :437.

Problem: under exact matching, a trigger key whose key-down passed because an extra modifier was held starts to match on a later auto-repeat once that modifier is released. Then:
- the Immediate arm has no repeat guard, so it fires and sets `suppress`;
- `mark_blocked(vk)` arms the key-up for suppression, although Windows saw the key-down.

This could not happen under superset matching: releasing keys never creates a new superset match.

The Risks entry describes the opposite direction: an extra modifier pressed while the key is held. That case is mostly unreachable, because the newly pressed key takes over auto-repeat and the held key stops repeating.

The audit's KS-06 suggested fix does not cover pass-then-suppress either (`book/src/audit.md:274`). It only suppresses repeats whose bit is set and clears the bit when a key-down passes. Deferring this to "the KS-06 lane" without naming it lets it survive that lane too.

Consequence: default config (`src/config/default_config.ron:127-135`).
1. Hold LCtrl and press CapsLock. CapsLock now reaches Windows: exact mismatch, as the CHANGELOG promises.
2. Keep both held for about 1 s, release LCtrl, then release CapsLock.
3. From the first CapsLock repeat after Ctrl's release, `"CapsLock"` matches exactly. `SwitchLanguage` fires on every repeat (KS-10), and each repeat is suppressed and marked blocked.
4. CapsLock's key-up is swallowed, so Windows keeps VK_CAPITAL logically held.
5. Every later plain CapsLock press and release is suppressed by the binding. The key stays held until a Ctrl/Alt/Win+CapsLock press lets a key-up through.

The same happens with M3's `"Ctrl+J"` binding: hold Ctrl+Shift+J in Chrome past the delay and release Shift first. J's key-up is swallowed and the page sees J held. M2's "Ctrl+CapsLock reaches Windows" step can trigger the CapsLock case on the owner's desktop.

Confidence: CONFIRMED for the engine path (traced above). It also relies on standard Windows typematic behaviour: a held key keeps repeating when another key is released.
Breaks: suppression invariant (CLAUDE.md principle 2: "a key-up is suppressed if and only if its key-down was"), KS-06/RC2. It is introduced by the owner-mandated exact matching, so it must travel with it.
What is needed:
- Replace the Risks entry with this path.
- Pin it as a scenario: `down LCtrl`, `down CapsLock` (passed), `repeat CapsLock`, `up LCtrl`, `repeat CapsLock`, `up CapsLock`; the key-up must pass. Then choose one:
  - (a) Make it a red-first fixed in this lane: a repeat's suppression follows its key-down's outcome.
  - (b) Hand it to the KS-06/1.1 lane as an explicit acceptance item, and order the lanes so `master` never carries exact matching without the fix.
- Until then, M2 releases CapsLock before Ctrl.

## Minor (improvements, not blockers)

### M1. M2 has no timing bound
Where: cut.md:419 (M1 has "< 0.4 s", M2 has none). If the human holds CapsLock past the typematic delay before pressing Shift, the Immediate `CapsLock` binding fires `SwitchLanguage` on every repeat (KS-10, src/engine/mod.rs:311-319). "The language switches once" then fails for a reason that is not this lane's. Add the same bound as M1.

### M2. Test-only `HeldKeys` impls duplicate the production rule
Where: cut.md:128-130 (`#[cfg(test)]` impls for `[VIRTUAL_KEY]` and `[VIRTUAL_KEY; N]`, "the same rules"). The unit tests of `keys_down`, `combo_matches` and `Sequences` then exercise a copy of the either-side/generic-slot rule rather than `impl HeldKeys for KeyState`. If the copy diverges (for example in generic-code handling), `engine::matching::tests` stay green while production differs. No scenario injects a generic code, so nothing catches it. Direction: build a `KeyState` in those tests (`press`, or `sync_modifiers` with a small `Env`) so that one implementation is under test.

### M3. `readme_matching_examples_load` proves a copy, not the README
Where: cut.md:377-378 ("the README RON blocks of this lane, copied verbatim"), acceptance row cut.md:410. A later README edit can break an example without failing the test. Direction: read `README.md` at compile time and extract the lane's blocks, so the test reads the published text.

### M4. README:528 should drop "Hot-path allocations: none", not restate it
Where: cut.md:493-494; README.md:528; book/src/gaps.md:172 records the claim as drift. `Decision.fire` pushes, action clones on arm, and a chord arm (`to_vec`, `vec![vk]`) still allocate on key-downs. Rewriting the line with the narrower reason "builds no key list per press" while keeping the heading leaves a false claim in a line this lane edits.

## Strengths to keep

- One canonical trigger type (`KeySpec`) for combo, step and chord, fixed at parse time. Output keys stay `VIRTUAL_KEY`, so no pseudo-VK can reach `SendInput` (D1, D7). RC4 is addressed at its root.
- Removing `ActiveKeys`, the 16-key cap, the 256-slot scan and the per-event chord `canonical` Vec. Matching is a few table reads per binding with no allocation or lock (R1).
- D2 (non-modifier trigger) closes the "Shift while CapsLock is held" half of KS-04. It is the audit's own fix, and it is escalated honestly as Q1 with a fallback. Typematic behaviour supports it: Shift pressed while CapsLock is held stops CapsLock's repeat, so no later repeat re-triggers `Shift+CapsLock`.
- D10 catches the `GetAsyncKeyState`-dependent flake that exact matching would introduce into the three adapter tests.
- `allow_extra_modifiers` is rejected on `sequence`/`chord` instead of being silently ignored, and overlapping chord keys are a load error. Both breaking changes are declared.
- The red-firsts are real. I traced the predicted first failing line against the tree for T1, T2, T4, T5, T7, T8, T9 (on e67e5cb), T10-T14 (C2 semantics) and T15, T17, T20, T21 (C4 semantics); each fails by assertion where the plan says it does. Guards T3, T6, T16, T18, T19 and T22 hold at every stage. The existing scenarios `more_specific_immediate_binding_wins`, `sequence_with_a_modifier_held_through_both_steps` and `modifier_lost_by_the_os_is_resynced_before_matching` stay green under C2, C4 and C6.
- Every re-located file:line in the table checks out on e67e5cb.

## Open questions for the architect / orchestrator

1. Book pages (plan Q2). CLAUDE.md:163-164 asks the developer to update `book/src/config/*.md` in the same lane, while docs/WORKFLOW.md:105-107 runs a phase book sync. The plan escalates this correctly. The orchestrator decides; the Docs-impact list (cut.md:499-507) is complete enough for the sync.
2. Optional, roadmap 3.5. With D2 the trigger test is cheap and selective. Checking it before `binding.condition.evaluate` (src/engine/mod.rs:301) for Combo bindings would avoid a foreground capture on key-downs that trigger no conditional binding. Not required by this lane.
3. Lane order relative to the KS-06/KS-14 lane: I1 and I2 both depend on it. Please state the intended order in the cut.

## Absent topics (walked, not applicable)

- §2 thread affinity, R10 loop/shutdown: no hook install/uninstall, window, timer or COM change (cut.md:191, :207-218).
- §5 timers and generation tokens: `on_timer` untouched; deferred arms stay keyed on the event's sided code, which by D2 is now always a combo key.
- §7 reload leniency, BOM and unknown fields: Workstream B. The new field is `#[serde(default)]` and is accepted by B's future `deny_unknown_fields`.
- §8 atomics, `static mut`: nothing new is shared. Matching runs inside `with_engine`.
- §9 process boundary and §10 input synthesis: no new `SendInput`, pipe, token or registry write. The registry code changes only combo construction and the INFO line (reads only).
- §6 module cycles: `engine -> data::key_spec -> data::vk_name`, and `system -> data`. Nothing new imports `hook`, `system` or `core::windows_actions` into `engine`.
