# Architecture critique: p0-A3-suppression, Workstream A "suppress repeats symmetrically; sync skips blocked keys" (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 4)
Plan: D:/tmp/ks-orch/p0-A3-suppression/cut.md   Code read at: master 82f542f (src/engine/*, src/hook/keyboard_hook*.rs, src/config/loader.rs, src/data/binding.rs, README.md, CHANGELOG.md, book/src/audit.md, book/src/roadmap.md, D:/tmp/ks-orch/p0-A2-matching/{fix_r1,results}.md)

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. The plan narrows spec item 1 to "no focus change" and pins that with three new tests, but never weighs the alternative its own D1 makes available
Where: plan G2 (cut.md:44), D1 (:67-70, "for repeat and key-up decisions `Forgotten` behaves exactly like today's `Unknown`"), D3 (:94-103), T7/T8/T9 (:263-265), commit-5 README text (:272-274), Q2 (:374-375); src/engine/mod.rs:439-450 (repeat tail); the deferred, sequence and chord arms skip repeats at mod.rs:343-345, :355-357, :369-371, :385-387 and :410-412.

Problem. Spec item 1 says: "An auto-repeat of a key whose original key-down was suppressed is also suppressed, whatever the binding kind or trigger." The plan keeps A2's rule: after a focus change, a repeat follows the bindings. For Tap, Hold, DoubleTap, chord and sequence keys the bindings ask for nothing on a repeat, so the repeat passes, and D3 then records it as `Passed`. T7 (`repeat J passed`), T8 (`repeat CapsLock passed`) and T9 (`repeat F13 passed`) assert exactly this for keys whose key-down was swallowed.

A2 chose that rule because its record could not tell "forgotten" from "unknown". A2 fix_r1.md, "Design choices", says: "an `Unknown` record says nothing about whether the OS has the key down". D1 removes that premise. A `Forgotten` record means the engine swallowed this press's key-down, so Windows does not have the key down from this press.

That opens a third option, which neither D1 nor D3 lists: `Forgotten` repeats are suppressed (the record stays `Forgotten`), and the key-up rule stays as it is (pass unless gesture-bound). I traced every focus-change scenario on master, and this option flips none of them:
- `:173` checks only the key-up;
- `:731` has no repeat;
- `:1408`, `:1433` and `:1456` already expect suppressed repeats and passed key-ups;
- `:1479` still gives "suppressed, fired": the press reads as a repeat, is suppressed, and the Immediate action fires;
- `:1497` is `Passed`.

So Q2's "four pinned scenarios depend on it" is true only for the key-up half of the policy. A2's results.md:138 (note 2) also handed this lane "possibly using a pending gesture as evidence of a swallowed key-down after a focus change". The cut does not mention it.

Consequence. Config: `( keys: "CapsLock", action: Launch(exe: "wt.exe"), trigger: Hold(300) )`.
1. The user holds CapsLock. At 300 ms the Hold fires, and wt.exe takes the foreground, so CapsLock becomes `Forgotten`.
2. The first typematic repeat, at about 500 ms, passes: the Hold arm skips repeats and D3 marks the key `Passed`.
3. Windows sees CapsLock go down for the first time and turns CapsLock on in the new terminal. Later repeats and the key-up pass too.

A chord `["J","K"]` bound to `Launch` and held past the repeat delay types `jjj`/`kkk` into the launched window. That is KS-06's headline symptom, now without the stuck key.

The plan pins this behaviour as expected (T7, T8, and the commit-5 README sentence). Roadmap 2.3 moves the other way: roadmap.md:298 ("`foreground_changed` no longer clears every blocked bit") and :300 ("lets the trigger key's auto-repeat into a window that `Launch` just opened"). So 2.3 will have to flip T7-T9. The plan does not regress against master: master leaks the same repeats, and stuck-key freedom holds (see Strengths).

Confidence: CONFIRMED. I traced D3 against mod.rs:439-450 on the T8 script, and checked each pinned focus-change scenario against the option by hand.
Breaks: brief spec item 1; the RC2 invariant (CLAUDE.md principle 2); KS-06 (the `jjj` symptom); the direction of roadmap 2.3.
What is needed: decide the repeat half of the focus-change policy explicitly, with this option and its cost on the table.
Options:
- (a) Keep the plan. List the third option and its cost in D1/D3. Split Q2 into a key-up question and a repeat question, and have the orchestrator accept the narrowing of spec item 1 explicitly. Name T7-T9 as tests of the 2.3 policy.
- (b) Suppress `Forgotten` repeats like `Suppressed` ones, and keep the key-up rule. The cost: after a lost key-up plus a focus change, the key's next physical press reads as a repeat and is swallowed once, and its key-up passes as a lone key-up. Example: with `Ctrl+F13` and F13 swallowed, Win+L while F13 is held, then a plain F13 press is eaten instead of typed. For RK1's swallowed modifier, the next press is eaten instead of passed. D3 and D4 then matter only for `Unknown` repeats (RK7). T7-T9 then need other routes, or become guards.

## Minor (improvements, not blockers)

### M1. Commit 1 fails `clippy -D warnings`: nothing calls `check_nothing_stuck` until commit 2
Where: plan commit 1 (:218-224); `testing` is `#[cfg(test)] pub(crate) mod testing` (src/engine/mod.rs:147-148), and its methods are `pub(crate)`.
Problem: commit 1's three self-tests use `check_press_symmetry`, `expect_nothing_stuck` and `with_context`. Nothing calls `check_nothing_stuck` before T1 in commit 2. rustc's `dead_code` lint covers unexported items, and a `pub(crate)` method of a crate-private test module is one.
Consequence: `cargo clippy --all-targets --locked -- -D warnings` fails on commit 1, which the plan declares green, with "method `check_nothing_stuck` is never used". The developer has to deviate from the cut.
Confidence: CONFIRMED.
What is needed: a commit-1 self-test that calls `check_nothing_stuck`, or add the method in commit 2.

### M2. Docs that promise the foreground resync repairs every modifier go stale, and the touch set leaves them out
Where: touch set "`src/hook/*` none" (:208). Affected docs:
- src/hook/keyboard_hook_callback.rs:244-247: "resyncs the modifiers ... repairing what the keyboard hook may have missed";
- src/hook/keyboard_hook.rs:129-131: "re-syncs modifier state after focus returns ... that ate our key-up events";
- src/engine/mod.rs:270-272: "Repair phantom-held modifiers from any earlier focus transition";
- src/engine/keys.rs:17-20: the `MODIFIERS` doc.

Problem: after D1 and D2, the resync never repairs a modifier whose key-down was swallowed (RK1).
Consequence: the 2.3 architect reads in the adapter that the WinEvent path already repairs lost key-ups of every modifier. RK1 then appears nowhere in the code.
Confidence: CONFIRMED.
What is needed: comment-only edits at those four places in commit 3.

### M3. The repeat branch of `check_press_symmetry` is never shown to fail
Where: plan commit-1 self-test (:221-222), which tests only the key-up branch; the strict property (:302-304), whose red-first is only an optional replay by the tester ("if green, record it").
Problem: the only failing self-test produces "key-down suppressed, up passed". The repeat comparison could be wrong, for example never compared or reset at a repeat. `every_press_is_suppressed_or_passed_as_a_whole` would then stay green whatever spec item 1 does, and no committed test would show it.
Consequence: acceptance 3 can be vacuous for repeats. A later change to the repeat tail (roadmap 1.1 edits exactly that code) that breaks spec item 1 would be caught only by T1-T5, not by the property.
Confidence: CONFIRMED (plan text).
What is needed: a `should_panic` self-test whose repeat outcome differs from its key-down. Under the current policy, T9's script with `check_press_symmetry` is such a route. Under I1(b), another route or a direct test of the checker is needed.

### M4. RK1 understates its consequence and is wrong for `Forgotten` keys
Where: plan D1 Cost (:79-80), RK1 (:343), Q3 (:376-377).
Problem: RK1 says the next press "is swallowed as a 'repeat'". Under D3 that holds only for a `Suppressed` record. A lost key-up usually comes with a focus change, so the record is usually `Forgotten`; such a press follows the bindings and normally passes. RK1 also does not name the effect on the default config. While a swallowed Shift is phantom-held, CapsLock matches `Shift+CapsLock` (held families {Shift}) and fires `ToggleCapsLock` instead of `SwitchLanguage`, until Shift is pressed and released. Traced on mod.rs:323-341, matching.rs:58-67 and the D2 skip.
Consequence: Q3 would be decided on a weaker description than the real behaviour.
Confidence: CONFIRMED.
What is needed: correct RK1 and Q3.

## Strengths to keep

- **Re-location is accurate.** I checked every row of the table against 82f542f: keys.rs:74-84/:88-93/:138-146, mod.rs:273/:303/:343-461/:465-490, sequences.rs:82-88/:120-139, chords.rs:94-100, README.md:355-367/:427, CHANGELOG.md:87-104/:119-121. All hold. H1, H2, H3a and H3b are real holes.
- **The red-firsts fail for the right reason.** On master, T1 run A, T2, T3, T4, T5, T10 and `resync_leaves_a_modifier_whose_key_down_was_suppressed` each fail at the predicted step with the predicted text. T6-T9 do the same both on master and at commit 3 (D1/D2 do not touch their paths). T10 is red under alternative (A) and green only with `Forgotten`, so D1 is justified on the spec's own terms.
- **No existing test flips except T5's declared replacement.** I traced the pinned scenarios that D1, D2 and D5 touch: `modifier_lost_by_the_os_is_resynced_before_matching`, `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key`, `unsided_shift_in_a_chord_fires_for_either_side`, the focus-change set (:1408-1517), `auto_blocker_matches_its_modifiers_exactly`, and the keys.rs unit tests.
- **Stuck-key freedom holds in the model.** With D3+D4+D5, every key that FakeEnv holds down has a `Passed` record:
  - only a key-down that reached Windows sets the OS state down, and every such key-down records `Passed`;
  - `mark_completed` never touches `Passed`, and `forget_blocked` only converts `Suppressed`;
  - a `Passed` key-up always passes.

  So the focus-mode property is sound. In strict mode `active[]` tracks the script for every scripted key, so each press's record is set once at its key-down, and the strict property holds at commit 5.
- **The hot path gets cheaper.** D2 replaces `GetAsyncKeyState` calls for swallowed modifiers with an array read. No lock, allocation, Win32 call, thread or atomic is added. The adapter, hooks, timers and the sentinel are untouched.
- **D5 is exactly roadmap 3.2's rule.** It is one guarded method, and it keeps the A2 pin at :731.
- **The property tests are deterministic and cost no new dependency.** They use seeded SplitMix64 and print their context.

## Previous findings

Round 1: none.

## Open questions for the architect

1. Q4 against CLAUDE.md:163-164 ("updates the matching `book/src/config/*.md` page ... in the same lane"). RK2 changes what `block_original_combo` does for a held Shift. The plan defers the config pages to the phase book sync, as A2 did. To settle it: the orchestrator confirms that this lane runs inside a `ks-phase` with the book sync on (docs/WORKFLOW.md:105-107). Otherwise the developer edits `book/src/config/known-issues.md` and the `block_original_combo` text in this lane.
2. D2 says both "skip by record" and "generic slots resynced as today". The two conflict only if a generic code (0x10-0x12) ever gets a record. That would take another program injecting `VK_SHIFT` with the LL hook reporting it unsided, and a binding swallowing it. To settle it: check whether `WH_KEYBOARD_LL` ever reports 0x10-0x12 for injected input. If it never does, nothing needs to change.
3. Q5 (RK2 under Fixed): CHANGELOG.md:7-9 reserves "Breaking" for behaviour changes. Against 0.1.0, RK2 replaces a stuck-key bug, so Fixed is defensible. It is the orchestrator's call; this is not a finding.

## Absent topics (walked, not applicable)

- section 1, hook budget: no adapter change, and D2 lowers the per-event cost.
- section 2, thread affinity: install, uninstall and the WinEvent hook are unchanged.
- section 4, mask key / KS-12: left to 1.2, and stated.
- section 5, timers: unchanged; `on_timer` never touches `KeyState`.
- section 6, engine purity: all logic stays in `src/engine`, with no new module edge.
- section 7, config and hot reload: no schema change, and RK9 covers reload mid-press.
- section 8, atomics and shared state: none added.
- section 9, process boundary: none.
- section 10, input synthesis: no `SendInput`.
- section 11, builder style: no `Binding` change, and `LastDown` stays private with no `Display`.

Ten topics, none with a consequence.
