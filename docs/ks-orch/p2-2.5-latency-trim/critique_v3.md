# Architecture critique: p2-2.5-latency-trim / roadmap 2.5 Latency trim (round 3)

Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 5)

- Plan: `D:/tmp/ks-orch/p2-2.5-latency-trim/cut.md`, Revision 2.
- Rulings: `rulings.md`, round 2.
- Code read at: `master` `3cde1d0`, the main checkout, read only.

## Summary

The round-2 rulings are carried out. Both v2 Important findings are resolved: the generic phantom and W10.

The core of W2a holds:

- D16 runs before every reader of a flag. I traced each reader below.
- The check only clears.
- A generic slot reads its generic code.
- The KS-14 skip and the `RepeatSuppressed` rule match master.
- No path makes a key stick or sends a lone key-up.

The one Important finding comes from D17, which the revision added. D17 keeps the outer modifier at a nested decided event. That is wrong when the later hook that injects the nested event also swallows the outer key. PowerToys Keyboard Manager does exactly that. In that case the cut's "strictly better than master" claim is false, and so is its "subset of master" claim.

The five Minor findings are a mutation mapping (M1), a missing mutation (M2), a manual check that cannot fail (M3), one undeclared visible effect (M4) and text that overstates master parity (M5).

## Rulings of round 2: carried out?

| Ruling | Where in the cut | Status |
|---|---|---|
| W2a, the check: clear-only, own key and KS-14 skipped, generic code read | D16 (:958-1008) | Done. The reader list is traced below |
| W2a, the `RepeatSuppressed` rule | D10 (:626-627, :645), D16 (:972) | Done: read like a key with no record, as master read it (`keys.rs:105`) |
| W2a, a re-entrant decided call | D17 (:1010-1057), U7 | Shown and skipped as ruled. **But the skip is harmful in the systematic instance (I1)** |
| W2a, fresh never used by the check | D5 (:447), D16 (:985, :995) | Done |
| W2a, D7 decided with an argument | D7 dropped (:487-508) | Done. The argument overstates master parity (M5) |
| W2a, the drain's direction | D3, D4, R-G, X33, X34 | Done |
| W2a, W10 corrected | W10, W14 (:407, :411) | Done. The Hooks Overview quote is verified |
| W2a, red-first pins and the four mutations | R-A, R-B, R-C (+R-D), R-F; X26-X29 | Done, traced on commit 7. One X31 variant is not caught (M1) |
| W2a, bench | R12m, R12m2, R1, R4 (D13) | Done |
| v2 M1 to M5 | as in "Critique v2 response" | Done, apart from M1 (one effect missing, M4 here), M2 (X31, M1 here) and M4 (one test, M2 here) |
| OQ1 to OQ3 | U4; W13 and M9; U10 and M8 | Done as ruled. M9 has lost its power to fail under D16 (M3) |
| Q4, Q9 to Q13 | Questions 2, 5, 6, 7, 8, 11 | Done |
| Latency list for `results.md` | D13 (:880-889) | Done |

## Critical (implementation must not start)

None.

## Important (must be resolved)

### I1. D17 keeps a modifier that the injecting later hook never lets Windows apply, so the remap's own key is decided with a phantom modifier on every press

**Where (plan):**

- D17 (cut:1010-1057), in particular :1043-1045: "So D17 is strictly better than master".
- :708: "its flag is the state Windows is about to have".
- U7 (:1083): "D17 makes the engine right whether or not it happens", and "no practical manual check".
- D11 (:687-695): "The check therefore creates no state that master could not reach at the same event".

**Where (code):**

- `src/hook/keyboard_hook_callback.rs:398-400`: the pass path returns what `CallNextHookEx` returns. A later hook that returns 1 drops the event.
- `:85-88`: the hook re-enters on the main thread while it waits.
- `src/engine/mod.rs:518-522`: master syncs every modifier except the event's own key at every event, the nested one included.

**Problem.**

- D17 is right only when the hooks after the daemon's pass the outer key-down. Then Windows will apply it.
- A nested decided event can happen by timing accident. The systematic source, the one that fires on every press, is a later hook that injects from inside its own callback. The usual such hook is a remapper, and a remapper swallows the outer key.
- PowerToys Keyboard Manager behaves this way. Its developer docs say: "If it is remapped to a key, we send the key down/up message for the target key and suppress the current key event". Its test mock models "as soon as SendInput is called, the low level hook is called for the new input event".
- The source code confirms it. `HandleSingleKeyRemapEvent` calls `ii.SendVirtualInput` inside the hook and returns 1. Its injected events carry PowerToys' own flag, not the daemon's sentinel, so the daemon decides them.
- PowerToys first injects a key-up of the source only when a modifier is remapped to a non-modifier. That key-up would clear the daemon's flag through an ordinary decided key-up. But:
  - the source code excludes LWin, RWin and "Win (Both)" from that path;
  - a modifier-to-modifier remap never takes it, because the condition requires a non-modifier target;
  - `ResetIfModifierKeyForLowerLevelKeyHandlers` acts only for a CapsLock target.

**Consequence.**

**(a) Win remapped to a key.** PowerToys remaps Left Win to F13, and its hook runs after the daemon's. That is the case when PowerToys started first, or after the 2.4 watchdog reinstalled the daemon's hook, which puts it first in the chain. The config has `( keys: "F13", action: SwitchLanguage )`.

1. LWin goes down. The daemon passes it, D17 marks `{LWin}`, and the daemon calls `CallNextHookEx`.
2. PowerToys' hook injects F13 down from inside its callback.
3. The daemon's hook is re-entered with that F13 key-down (U7).
4. D16 skips LWin as unapplied, so the election sees Win+F13. The exact `F13` binding misses, and F13 passes. A `Win+F13` binding would fire in its place.
5. PowerToys returns 1, so Windows never applies LWin. The next top-level key-down clears the flag, which is too late.

This happens on every press. On master, the sync at F13 reads LWin up, and the binding fires. D16 without D17 does the same.

**(b) Alt and Ctrl swapped.** PowerToys swaps Left Alt and Left Ctrl, a common remap for Mac keyboard layouts. The config has `( keys: "LCtrl", action: SwitchLanguage, trigger: Tap(200) )`.

- The nested LCtrl key-down is decided with LAlt held, so the exact Tap never arms. Every tap misses.
- Master arms it, because its sync reads LAlt up there.

**What does not break.**

- No key sticks, and no key-up reaches Windows alone. D17 only keeps a flag.
- `mark_completed` leaves the outer key's `Passed` record alone (`keys.rs:188`).

**Confidence: PLAUSIBLE.**

- PowerToys' in-hook `SendInput` and its suppression come from its source and its developer docs.
- What is not verified on a machine is the nested delivery to the daemon's thread while it waits in `CallNextHookEx`. That is U7. The cut treats U7 as possible, since D17 exists for it, and the module's Re-entry docs describe the same delivery for `SendInput`.
- Contrary to U7's "no practical manual check", a PowerToys remap is a practical manual check.

**Breaks:**

- the safety argument of ruling W2a ("the check is a subset of master", D11 :687-695);
- D17's own claim;
- the round-2 precedent: a regression against master on every press, wherever another hook is installed, is not accepted as a declared residual (rulings, round 2, preamble).

**What is needed.** A requirement, not a design.

- A nested decided event can see an outer key in two ways:
  - Windows will apply it. Then D17 is right and master is wrong.
  - The injecting later hook swallows it. Then D17 is wrong and master is right.
- Weigh both, and decide with an argument which one the design serves.
- Correct :1043-1045, :1083 and :694.
- Declare the instance left open in the W-table and in row 88.
- Pin the choice with an engine test: a nested event with the outer code unapplied, and Windows keeping it up.
- Add a manual check with a PowerToys remap whose hook runs after the daemon's.

Options:

- **Drop D17.** This gives master parity in both cases, and the subset-of-master argument becomes true.
- **Keep D17** and declare the remapper case.
- **A narrower rule** that the architect can argue.

W2a asked for the skip ("skips those keys"), so the orchestrator may need to rule.

## Minor (improvements, not blockers)

### M1. R-F does not catch the second variant of X31

**Where:**

- X31 (cut:1713): "the key-up check runs over every modifier, or at a key-up that passes | R-F (A up with LCtrl held asks 1; F13 up asks 3)".
- R-F's steps (:1364).

**Problem.**

- With variant 2, the check runs at a key-up that passes, but its set is still MENU. So it asks only about a held Alt or Win whose key-down was not swallowed.
- R-F's passing key-ups are these:
  - the four A ups, with nothing held, LCtrl, LCtrl+LShift, and LCtrl+LShift plus the swallowed LWin;
  - the LAlt, LShift and LCtrl ups, which come after LAlt is released.
- None of them has an unswallowed Alt or Win held, so every count stays as expected.
- "A up with LCtrl held asks 1" happens only under both variants at once. Variant 1 alone is caught: LWin up asks 2 and F13 up asks 3.

**Consequence.** The tester applies variant 2 and R-F stays green. That breaks ruling M2: "No scenario is listed against a mutation it passes under".

**Confidence:** CONFIRMED (trace against D16 :969-971).

**What is needed.** Do one of these:

- add to R-F a passing key-up with an unswallowed LAlt held, which must ask 0;
- split X31 and list only what catches each variant.

### M2. The new `Unnoticed` property has no named mutation

**Where:** cut:943 and :1349; the mutation table :1678-1732, where no row names it; ruling M4.

**Problem.**

- `no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` is new in commit 7 and builds on new API. M4 therefore asks for a mutation that makes it fail.
- The property draws random scripts. Which mutation its seeds catch must be shown, not assumed.

**Consequence.**

- `tester.md:178` ("Every compile-red test is covered by a mutation") leaves a row empty.
- A property that passes vacuously would go unnoticed. That would happen if its unnoticed steps never reach a modifier with a record, which is the D10 path.

**Confidence:** CONFIRMED.

**What is needed.** Name a mutation that its fixed seeds fail under, and have the developer confirm it. Candidates: X4 (D10 reverted), or the witness term removed.

### M3. Under D16, M9 cannot fail, and it does not exercise the StickyKeys direction that can differ from master

**Where:** W13 (cut:410), M9 (:1526), U9 (:1085); ruling OQ2.

**Problem.**

- M9 fails only if a CapsLock toggles caps lock. That needs a set Shift flag while Windows has Shift up, and D16 clears exactly that before matching.
- Both CapsLock presses of M9 come after the latch has ended, at the letter or at the click. So the expected result holds on HEAD whatever StickyKeys does.
- The direction where HEAD can differ from master is the latched one:
  - Suppose the hook sees the Shift key-up while Windows keeps Shift latched down with no LL event.
  - Then the engine has Shift up, direction B, and the check never sets a flag.
  - Master's per-event sync would read the latch, if `GetAsyncKeyState` reports it.
- M9 never presses a key while Shift is latched.

**Consequence.**

- Suppose a StickyKeys user taps Shift and then presses CapsLock. On every latched use they get `SwitchLanguage` instead of `ToggleCapsLock`, and every Shift binding misses.
- M9 still reports green.

**Confidence.**

- CONFIRMED that M9 cannot fail on HEAD (trace against D16).
- PLAUSIBLE for the latched behaviour. [STICKYKEYS](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys) documents that a modifier "latches the key down until the user presses a non-modifier key or clicks a mouse button". It does not say what the LL hook sees.

**What is needed.**

- Add a latched step to M9: tap LShift, then press CapsLock with nothing in between.
  - Expected: `ToggleCapsLock`, which is what master gives if the latch reads down.
  - Have the owner record what HEAD does.
- Fill W13's phantom-released cell with what the user sees.

### M4. Row 88 and D11 miss one visible effect of direction B: the `Alt+Shift` blocker matches when the unseen modifier is not its partner

**Where (plan):** D11 (cut:712-724), the owner summary (:737-746), row 88 (:1161-1171).

**Where (code):**

- `src/engine/matching.rs:93-102`: Windows is asked only about keys held for the engine.
- `matching.rs:262-269`: the existing test, where `with_ctrl` and `all_down` give no match.

**Problem.**

- Windows has Ctrl or Win down from a key-down the daemon did not see. The engine holds only Alt and Shift.
- `blocker_matches` never asks about Ctrl, and the `Swallow` blocker swallows the Shift or Alt key-down.
- On master, the sync at that event would have set LCtrl, and the KS-05 read would have found it down, so there would be no match.

**Consequence.**

- In a direction-B residual case (R-E1 to R-E4, R-L, R-M1), the user presses Ctrl+Alt+Shift+K, and the application receives Ctrl+Alt+K.
- Row 88 names the blocker only for the opposite case: when the unseen key is its partner, the blocker misses and Windows switches the layout.

**Confidence:** CONFIRMED (trace).

**What is needed.** Add the effect to D11, row 88 and the owner summary. Ruling M1 asks what the user sees for each residual.

### M5. The own-key exception is wider than master by every passing key-up, and the texts say otherwise

**Where (plan):**

- D7 (cut:496-498): "Master's sync never read that key either".
- D11 (:705-706): "with no decided event in between" and "as on master".
- Goal 4 (:119-120) and row 88 (:1170-1171): "stops counting at the next key event".
- The owner summary (:747-748) and `keys.md:194` ("once a key event finds").
- D16's key-up rule (:969).

**Problem.**

- Master syncs at every event, key-ups included: `mod.rs:518-522` runs before the match on the direction.
- HEAD checks at key-downs only, plus, for Alt and Win, at key-ups it swallows. A passing key-up repairs nothing.

**Consequence.**

1. The user holds LCtrl and C.
2. LCtrl's key-up is lost with nothing noticed (a residual case of row 88's kind).
3. The user releases C, and the key-up passes.
4. The user presses LCtrl again.
   - On HEAD, the press reads as an auto-repeat, so an `LCtrl` Tap or Hold misses it (row 63's kind).
   - On master, the C key-up cleared LCtrl, so the press counts as a press.

The record is `Passed`, so the event passes, and no key sticks.

**Confidence:** CONFIRMED (trace).

**What is needed.** One of the following, at the architect's choice:

- write "the next key-down" in Goal 4, row 88, the owner summary and `keys.md:194`, and correct the D7 argument and D11's "as on master";
- or run the check at passing key-ups too, at one read per held modifier.

## Previous findings (critique_v2.md)

- **I1, the drain never repairs a generic slot: RESOLVED.**
  - The drain clears a set generic slot whose code reads up and never sets one (D3, D4).
  - D16 reads a generic slot through its generic code.
  - The tests are R-C (red on commit 7, green only with the drain repair or the check, X26 + X33), R-D, R-G and C-2. The mutations are X25, X29, X33 and X34.
  - I checked the generic-code quote again on [GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate).
- **I2, W10 argued backwards: RESOLVED.**
  - W10 is corrected, and W14 holds the old argument where it is true. I verified the [Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks) quotes.
  - The repair happens at the event, auto-repeats included.
  - R-A and R-B are red on commit 7 with the predicted messages. I checked the formats against `testing.rs:584-586` and `:799-804`.
  - D17, which came with this repair, is the new I1.
- **M1, "matching only": RESOLVED.** The four effects are named. One more effect is missing (M4 here).
- **M2, the scenario-to-mutation map: RESOLVED, apart from X31 (M1 here).**
  - I traced S3 to S8 under the mutation of each one's source: X6, X7 (variant 1 only, as listed), X10, X11, X13, X14 and X24.
  - Each fails on HEAD under its mutation, and none is listed against X21 or X26.
  - S9 now asserts `expect_nothing_fired()` and has its A key-down, so it catches both X17 and X28.
- **M3, the deferral costs one more wake only for a single-event burst: RESOLVED** in D12's Cost, the touch set for `errors.md:215`, M6, M7 and the CHANGELOG entry of commit 13.
- **M4, mutations for compile-red tests: PARTLY OPEN.** Every test that critique_v2 listed is covered, by X1-X3, X16, X23, X48 and X49. One new test has none (M2 here).
- **M5, the bench gates: RESOLVED.**
  - P, N and H are interleaved in one session.
  - R12, R12m, R12m2 and R4 gate against N.
  - R1 gates with 0.5 Δ and fails on a revert.
  - I have one open question below.
- **Open questions 1 to 3: carried out as ruled.** M9's power to fail is now M3.

## Strengths to keep

**D16 runs before every reader of a flag.** I traced each one against `mod.rs:668-899`.

- At a key-down:
  - the election (`:677`);
  - `other_key_down`, which reads records only (`gestures.rs:333`);
  - `deferred_group` (`:693`);
  - `blocker_matches` (`:721-723`);
  - `admits` (`:742`);
  - the sequences (`:767-774`);
  - `mark_completed` (`:789`, `:815`);
  - rule 5, which reads records only;
  - the mask (`:854`).
- At a key-up, only the mask reads a flag (`:897`). `gestures.release` (`:869`) takes no `KeyState`.
- Timers and resets read no flag.
- Repeat detection reads only the event's own key, which the ruling exempts.

**The check is clear-only, outside D17.**

- No path sets a flag.
- Lemma 1 holds independently of flags.
- Lemma 2 holds, because every seen key-down still leaves a record (D10). I traced a cleared modifier whose next repeat is decided as a first press: it ends `Suppressed` or `Passed`, and its key-up follows the witness.

**The red-first tests fail for the stated reason.**

- I traced R-A to R-H on commit 7, with their messages.
- I traced R-F's counts step by step, on HEAD and on commit 7. The first mismatch is `left: 0, right: 1` at the first A down with LCtrl held.
- R-G's assertion order makes the clear the first failure.
- Commit 3's scenario still exercises D10 on HEAD: the LShift press is its own key, so the check skips it.

**Other strengths.**

- **D7's drop is argued soundly** for the drain: no drain can reach the own-key case while the user is idle. M5 corrects only the master comparison.
- **The bench has one session per comparison.** On P and N, a sentinel event returns at `keyboard_hook_callback.rs:222-224` before any state is touched, so R12m and R12m2 isolate exactly the reads that D2 and D16 add.
- **No text of the dropped design remains.** Grep finds `RESYNC_PERIOD`, `held_since`, `last_read_at`, `unswallowed_modifier_held`, E7 and "matching only" only in the history and removal lists.

## Open questions for the architect

1. **How much slack does R1's gate leave for HEAD's own work?**
   - Stop rule (b) accepts Δ = 10 ns, and the gate then allows 5 ns over N.
   - HEAD adds per event about 3 ns of its own: the 11-slot scan at key-downs, and `Resync::event`'s OR and store. Read noise across the three runs comes on top.
   - Near the stop rule, the gate can report a breach that is not a revert.
   - To settle it: raise stop rule (b) to a Δ whose half covers HEAD's declared additions plus the spread of the three means, or gate H at N plus the declared additions and below P.
2. **Two public types named `ModifierSet`.**
   - The new `engine::ModifierSet` is a set of 11 codes. The existing `data::key_spec::ModifierSet` (`src/data/key_spec.rs:95`) is a set of four families.
   - Both live in the matching domain. `matching.rs:26` imports the family set, and `HeldKeys::modifiers()` returns it.
   - This is no correctness issue. A distinct name chosen at cut level would save the code-reviewer a round.
3. **Mutation IDs X6 and X7 are reused.** Cut :49 and :508 say that "the old X6/X7" were removed, while the table defines new X6 and X7. A tester who reads both may be confused, so one sentence in the table would settle it.

## Absent topics (walked, not applicable)

- **§1 Hot path.**
  - What is added is costed: k reads per key-down (accepted by W2a), two TLS accesses (D17), one `GetTickCount` (E4) and one `with_main` per sentinel modifier.
  - There is no allocation, lock, `log::` or post.
  - `code < 0`, the heartbeat and the remapping gate keep their order, and every path still ends in exactly one outcome.
  - A panic in the `extern "system"` callback aborts, so the guard cannot leave `UNAPPLIED` set.
- **§2 Threads.** No install or uninstall moves, and no thread is added. The drain's read runs on main, under the borrow, without a pump.
- **§6 Purity.** `engine::modifiers` and `engine::resync` are pure, and `unapplied` arrives through `Env`. No new module cycle appears.
- **§7 Config.** There is no schema change, and `default_config.ron` is untouched.
- **§8 Shared state.** `UNAPPLIED` and `HIDDEN_INPUT` are main-thread TLS `Cell`s. `unseen_input` uses `AcqRel` for the increment and `Acquire` for the load. Nothing uses `Relaxed`.
- **§9 Process boundary.** Nothing changes.
- **§10 Input synthesis.** No `SendInput` is added. Every daemon injection still pairs its key-down and key-up in one call (`sequence.rs:163-212` re-read).

## Microsoft Learn checked this round

- [Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks): "always installs a hook procedure at the beginning of a hook chain", "Each hook procedure in the chain determines whether to pass the event", and "passes an event to the next procedure by calling the CallNextHookEx function". All verified.
- [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): called "in the context of the thread that installed it ... by sending a message"; the injected-input context switch; "called before the asynchronous state of the key is updated"; the timeout behaviour and the 1000 ms maximum. All verified.
- [STICKYKEYS](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys): the latch quote is verified. The page says nothing about LL hooks.
- [GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate): the three failure reasons and the generic-code sentence are verified.
- Not Microsoft Learn, used for I1:
  - [PowerToys Keyboard Manager event handlers](https://microsoft.github.io/PowerToys/modules/keyboardmanager/keyboardeventhandlers/);
  - [`KeyboardEventHandlers.cpp`](https://raw.githubusercontent.com/microsoft/PowerToys/main/src/modules/keyboardmanager/KeyboardManagerEngineLibrary/KeyboardEventHandlers.cpp).
