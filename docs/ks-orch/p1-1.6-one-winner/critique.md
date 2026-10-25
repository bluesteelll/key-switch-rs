# Architecture critique: p1-1.6-one-winner / roadmap 1.6, KS-13 (round 1)
Verdict: APPROVED (Critical 0, Important 0, Minor 3)
Plan: D:/tmp/ks-orch/p1-1.6-one-winner/cut.md   Code read at: master 9357cfa (work tree D:/key-switch-rs/key-switch-rs)

Read: the whole cut (1-533); src/engine/{mod,matching,gestures,env,chords,keys(mark_completed),properties,testing}.rs;
src/engine/scenarios.rs (every multi-binding config, 1-45, 890-950, 1415-1860, 2005-2185, 2625-2700, 2910-2970,
3146-3390); src/data/{binding,trigger,condition,key_combination}.rs; src/config/loader.rs:180-509, 643-689, 1880-1940;
src/config/default_config.ron; src/hook/keyboard_hook_callback.rs:55-268; src/hook/keyboard_hook.rs:95-135, 300-493;
README.md:203-502, 686-725; audit.md KS-13 (391-403); roadmap.md 1.6 (281-289); matching.md 105-133;
known-issues.md rows 10, 11, 25; docs/WORKFLOW.md (red-first, book sync).

## Critical (implementation must not start)

None.

## Important (must be resolved)

None.

## Minor (improvements, not blockers)

### M1. The "Repeats" risk says the post-lane behaviour equals today's; it does not
Where: plan Risks "Repeats" (cut.md:460), D6 (cut.md:134-139); src/engine/mod.rs:420-422, :537-551
Problem: the paragraph says that after Ctrl is released under a held `Ctrl+F22` (Tap, `block_default: false`),
F22's repeats elect `F22` (`PressKey("Left")`) "and inject Left per repeat while F22's own repeats pass. That is
today's behaviour, too." On master the losing `F22` binding's `block_default: true` swallows F22's key-down
(mod.rs:420-422, then `mark_blocked` at :544-545), so every later repeat is swallowed by the record (:537-538).
After the lane the winner (`Ctrl+F22`, passing) decides, the key-down is marked passed (:546-551), and every
repeat passes.
Consequence: `( keys: "J", action: PressKey("Down"), allow_extra_modifiers: true )` plus
`( keys: "Ctrl+J", action: SwitchLanguage, trigger: Tap(200), block_default: false )`; hold Ctrl+J, release Ctrl,
keep J held: on master the editor gets one Down per repeat and no `j`; after the lane it gets `j` and Down
interleaved at the repeat rate. No key sticks (the record keeps the press whole), and it follows from the spec's
"only the winner decides suppression", but it is a new user-visible result that neither W6 (Ctrl stays held) nor
the CHANGELOG/known-issues rows describe.
Confidence: CONFIRMED (traced on master and against D6/D8)
Breaks: accuracy of the Risks section and of the Breaking entry (KS-06 record rule interacting with KS-13)
What is needed: correct the sentence, and record the case where users will look for it (a known-issues row next to
the planned 69-72, or a clause in the Breaking entry). A scenario pin is optional.

### M2. Two "why" comments in src/hook/keyboard_hook.rs go stale and are not in the touch set
Where: plan touch set "Not touched" (cut.md:298); src/hook/keyboard_hook.rs:107-109, :312-314
Problem: the plan rewrites the twin doc of `sort_by_specificity` (mod.rs:267-273) because, after D2, list order no
longer makes a conditional binding win. The same reason survives in `update_bindings`' doc ("so more specific
combinations are checked first in the callback") and in `equal_specificity_keeps_config_order` ("two bindings on
one key with different `when` conditions rely on this").
Consequence: a later reader or mutation tester takes the test as the guard of `when:` precedence. It guards only
the file-order tie-break; `when:` precedence is pinned by W3/W5/W7 and `rank_puts_more_keys_before_a_when_condition`.
Confidence: CONFIRMED
Breaks: CLAUDE.md principle 8 ("Comments say why")
What is needed: add the two comment lines to commit 2 (doc-only, no code change in `src/hook`), or say in the cut
why they stay.

### M3. The duplicate warning's "remove one of them" can drop an auto-blocker silently
Where: plan D9 (cut.md:216-222, message at :222); src/config/loader.rs:285-291
Problem: `Binding::duplicates` ignores `block_original_combo`, which is right for the election (the later copy never
acts), but `expand_with_auto_blockers` collects hotkeys from every user binding, dead copies included
(loader.rs:285-291). The message names the later binding as the duplicate and invites removing either one.
Consequence: `( keys: "CapsLock", action: SwitchLanguage )` then
`( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true )` prints
`binding #1 [CapsLock] duplicates binding #0 [CapsLock]; remove one of them`. A user who deletes #1 removes the only
flag, the Alt+Shift blocker disappears on reload, and Windows' own hotkey switches the layout again next to CapsLock.
Confidence: CONFIRMED (loader path); the user reaction is the plausible part
Breaks: nothing in the spec; a usability trap in the new message
What is needed: either let the message or the docs (errors.md row, README Errors paragraph) say that a flag such as
`block_original_combo` on the removed copy must move to the kept one, or exclude such pairs. The developer can
settle this in commit 5 without changing L1-L4.

## Previous findings

Round 1: none.

## Strengths to keep

- Suppression is monotone. After the lane a key-down is suppressed only if the winner or group, a blocker, or a
  sequence/chord asks. Before the lane every matching binding could ask, so the new set of suppressed key-downs is
  a subset of the old one. The `LastDown` record (keys.rs, mod.rs:519-551) and the key-up rule (:555-581) are
  untouched. So no stuck key, no new lone-Win/Alt swallow (KS-12) and no new feedback path (R2) is possible.
- D2's explicit rank, rather than extending the sort, keeps sequence/chord/blocker visit order unchanged, and ties
  still resolve by the stable sort's file order. The D3 last bullet holds: an equal rank means the same key count
  and the same `has when:`, so every admitted member comes after the winner, and `GroupArms` can never skip the
  winner.
- D7 is a real KS-08 gain. A conditional `keys` binding no longer calls `GetForegroundWindow`, `OpenProcess` or
  `GetWindowTextW` on presses of other keys (today mod.rs:382 evaluates every `when:` first). W16 pins it with a
  red-first whose failure is a panic. I verified that it panics on master.
- The red-firsts are traced and fail for the stated reason on master (W1-W10, W14, W16) and on commit 3 (L1, L3). I
  checked each predicted first-failing line against the harness message formats (testing.rs:440, :613) and the
  script order. Every listed mutation (blocker as a candidate, `>=` ties, no condition rank, no group, no keys,
  `when:` or `combo_matches` check in the group, no `GroupArms`, OR over losers) has a killer test.
- I found no existing scenario that changes outcome. Every multi-binding config in scenarios.rs and properties.rs
  either has a single matching binding per event or already matches the new rule (e.g. :258, :441, :1765, :2052,
  :2632, :3148, :3238).
- D9 keeps `config -> engine` acyclic, D10 adds a test seam that 1.7 can route to `log`, and nothing in `src/hook`,
  threads, timers, locks, atomics, handles or `SendInput` changes.

## Open questions for the architect / orchestrator

- Q2 (Immediate shadowing a deferred group on the same keys and `when:`): I agree with the recommendation as far as
  the spec goes. One piece of evidence for the owner: the shipped template says "uncomment to enable"
  (default_config.ron:207), and its own CapsLock Tap/Hold example (:259-271) sits below the default `CapsLock`
  binding. So a user who follows the template literally gets an inert pair after this lane, with only a comment
  to explain it. If the owner wants it, the cut already names the cost: one disjunct and one message.
- Q1: with D1, "a winner with `block_default: false` lets the key through" holds only when no `sequence`/`chord` uses
  the key too. The §16.2 rewrite should say so in the same paragraph as the acceptance-style example, which the cut
  plans.

## Absent topics (walked, not applicable)

Sections 1-11 walked.
- Thread affinity / install-uninstall (§2): no hook, window, timer or thread change.
- Hold timer generations (§5): unchanged. The election holds no state across events, so reload, `swch off/on` and
  foreground change need nothing new (W11 plus existing scenarios).
- Config schema / RON compatibility (§7): no schema change; the behaviour changes are declared Breaking with
  consequences.
- Process boundary, pipe DACL, token inheritance (§9): untouched.
- Input synthesis (§10): no new `SendInput` builder or path.
- Builders / Display (§11): no new field, setter or enum variant.
