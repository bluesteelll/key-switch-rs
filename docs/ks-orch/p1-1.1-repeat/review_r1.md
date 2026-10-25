# Code review: p1-1.1-repeat @ 10bcf94
APPROVED (Critical 0, Important 0, Minor 2)

**Head reviewed.** `pack_r1.md` and `impl.md` name HEAD `ad96e73`. The branch moved during this review to
`10bcf94` "Tests: sequence and chord guards ignore auto-repeats (KS-10)". That commit adds 48 lines to
`src/engine/scenarios.rs` and nothing else, and it carries a `Claude Sonnet 5.5` trailer (see Open question 2). I reviewed
the full `git diff master...ad96e73` and the `ad96e73..10bcf94` delta, all through `git show` / `git diff`.

## Gate: fmt OK · clippy OK

- I ran both on `git archive` exports of `ad96e73` and of `10bcf94` in the session scratchpad, not on the
  work tree, with a separate `CARGO_TARGET_DIR`.
- `cargo fmt --check`: exit 0 on both.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 on both, with no warning lines.
- `cargo test` was not run. The tester owns it.

## Remarks

### Critical
None.

### Important
None.

### Minor

#### M1. The Hold subcase of the repeat pin cannot fail for the leak it is meant to catch (spec "pin it with a test"; cut.md:280)
**Where**: `src/engine/scenarios.rs:1867-1879` (Hold subcase of `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`), against `src/engine/mod.rs:433-436` (`Trigger::Hold(term_ms) => { if was_repeat { continue; } ...`).  **Confidence**: CONFIRMED (path traced). The claim that "no other test catches it" is PLAUSIBLE: I searched the scenarios and properties, but did not run a mutation.

**Problem**: The subcase sends its first repeat at +500 ms and releases F13 at +599 ms (`repeats(.., 3, ..)` = +500/+533/+566, then `up`). The cut says this test "goes red if the `PressKey` default leaks into the deferred ... arms". Suppose the Hold arm honours the flag, i.e. `if was_repeat && !binding.repeat { continue; }`:
- At +200 `fire_hold` has already removed the first entry (`gestures.rs:173-179`). So at +500 `arm_hold` finds no pending Hold (`gestures.rs:128-153`) and arms a new one, due at +700.
- The release at +599 is before +700, so `release_hold` cancels it (`gestures.rs:158-169`).
- Nothing fires, the key-up is suppressed (it is blocked anyway), and the total stays at one Backspace. The subcase stays green.

Other tests do not close the gap:
- Deleting the guard outright is caught by `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`scenarios.rs:2225`): its key-up turns suppressed through `gesture_bound`. That test uses `Launch`, which does not repeat, so it does not see the leak form.
- The other Hold scenarios with a repeating action send no repeats (`:383-590`) or release at +533 (`:1390`).
- The property test checks fires only on `repeat` steps, and Hold fires only from timers.

The tester's `10bcf94` closes the same gap for sequences and chords; Hold is the remaining arm.

**Failure**: Apply the mutation at `mod.rs:434`, then hold `( keys: "F13", action: PressKey("Backspace"), trigger: Hold(200) )` for 2 s. Backspace fires at +200, +700, +933, +1166 ms and so on, once per re-arm. The subcase passes.

**What to do**: In the Hold subcase, keep repeating (or advance) past "first repeat + term" before the release. For `Hold(200)` that is at least 7 repeats 33 ms apart, or an advance past +700. A re-armed Hold then fires inside the script, and `expect_total_fired` sees the second Backspace.

#### M2. A CHANGELOG line was not reflowed
**Where**: `CHANGELOG.md:242` (123 characters; the rest of that entry wraps at about 76).  **Confidence**: CONFIRMED.
**Problem / Failure**: Cosmetic only. The I1 wording was spliced into the line ("... the next press works. For a key whose key-down the daemon swallowed, which is the usual ..."). It renders the same.
**What to do**: Reflow the paragraph.

## Checklist walked (items with no reachable failure, one line each)

### Conformance to the cut and the roadmap

- **D1.** `RawBinding.repeat: Option<bool>`, `#[serde(default)]`, is the last field (`schema.rs:69-75`), and `deny_unknown_fields` stays (`:31`). The default is resolved at load (`binding.rs:96`). `with_repeat` is applied only for `Some` (`loader.rs:391-395`).
- **D2.** `repeats_by_default` is exhaustive with a nested `MediaKey` match and no `_` arm (`windows_actions.rs:165-185`). True only for `PressKey`, `VolumeUp` and `VolumeDown`, as the owner decided.
- **D3.** `mod.rs` has three hunks: two doc hunks and the Immediate arm `:407-420`. On a repeat, the first matching Immediate binding takes the slot, and the action is pushed only when `!was_repeat || binding.repeat`. The KS-06 repeat block (`:510-543`) and `on_key_up` (`:546-573`) are unchanged. The deferred, sequence and chord arms still `continue` on `was_repeat` (`:422/434/448/464/489`).
- **D4.** The loader check runs after the `allow_extra_modifiers` check (`loader.rs:374-383`), so each binding still reports its first error. The message matches errors.md row 18 and the README.
- **D5.** The Display suffix shows only overrides and only for an Immediate `Combo`. The auto-blocker branch is unchanged (`binding.rs:187-210`). The default banner is byte-identical, because the template sets no `repeat:`.
- **D7.** `check_repeats_fire_only` and `reload` are test-only. `reload` sorts with `sort_by_specificity` and does not reset the engine, as `update_bindings` does.

### Roadmap 1.1 acceptance

Each line is pinned by a test that I traced as red without the fix:
- `default_capslock_held_with_ten_repeats_switches_the_language_once`: shipped template, symmetry and stuck checks.
- `press_key_backspace_held_with_ten_repeats_fires_eleven_times`.
- `repeat_true_on_switch_language_fires_on_every_repeat` and `reload_mid_hold_...`.
- `volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not`.

The sequence and chord pins `a_repeat_of_a_sequence_step_does_not_count_as_the_next_step` (`:1933`) and `repeats_of_both_chord_keys_do_not_complete_the_chord_again` (`:1956`) go red without their guards:
- **Sequence.** At +500 the pending entry's `next_step` 1 is F13 and F13 is held, so the sequence completes on the repeat.
- **Chord.** The F13 repeat starts a new entry and the F14 repeat completes it.

### Docs

- CHANGELOG: Added; Changed (**Breaking:**); the KS-06 sentence; Fixed KS-10.
- README: field list, triggers row, behaviour rule, flags row, error quote, new error bullet, policy bullet.
- `default_config.ron`: the `repeat` block and the commented `Ctrl+Alt+PageDown` example. `pagedown` parses (`vk_name.rs:153`) and `]` parses (`:202`).
- Nine `book/src/config` pages. The remaining KS-10 mentions in config pages are the "fixed" notes.

### Other sections

- **Unsafe (section 2).** No `unsafe` was added or made newly reachable. The hook adapter and `system/*` are not in the diff.
- **Atomics (section 3, R4).** None added. The flag is immutable data in the `ArcSwap` snapshot.
- **Win32 and threads (section 4).** No new API, thread, window, timer, COM or `SendInput` site.
- **Hot path (section 5, R1).** One `bool` read. The `BindAction` clone and the `fire` push now happen only when firing, so a non-repeating repeat allocates nothing. No new lock, I/O or panic path. `with_engine` is unchanged.
- **Correctness (section 6).** A `vk >= 256` key-down is never read as a repeat (`keys.rs:143-148`), as before. Repeat identity is the raw sided code, as for combos, chords and sequences (R8). Reload mid-hold, `swch off`/`on`, a focus change and a lost key-up mid-hold each have a scenario.
- **Builders (R5).** `with_repeat(self, bool) -> Self`. There is no `set_repeat`, and the constructors still take two arguments.
- **Display (R6).** No new enum variant. The `_ => ""` in `repeat_suffix` matches a `(bool, bool)` tuple, not a variant.
- **Specificity (R7).** `sort_by_specificity` and `update_bindings` are untouched. `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` pins "first match takes the slot" on repeats.
- **Sentinel (R2), registry (R9), message loop (R10).** Untouched. `PressKey` and `Media` repeats still go through `press_vk` with `injected_sentinel()`.
- **Shared state (R3).** No new state, no `static mut` and no new `Mutex`.
- **Code-built bindings.** `app.rs:224` and `keyboard_hook.rs:280-330` use `DoNothing` or `SwitchLanguage`. None of their tests sends a repeat that expects a fire.
- **Commits.** Each subject has the `<Area>: <what>` form and ends with the trailer. Source stays ASCII: the loader test uses `\u{2192}`, and the em dash in `default_config.ron` is on a line that already had it.

## Positive

- D3 removes work from the LL callback: no clone and no `Vec` push per non-repeating repeat. It also keeps the A3 symmetry code byte-identical.
- The slot rule (a non-repeating winner still takes the repeat) is reasoned in the cut, documented in `mod.rs` and §11.3, and pinned. It prevents a later binding from firing at the repeat rate after one layout switch.
- An exhaustive `repeats_by_default` means a new `BindAction` variant fails to compile until someone decides its default.
- The property `auto_repeats_fire_only_actions_that_repeat` and the `check_repeats_fire_only` harness hook check repeat fires over random scripts.

## Open questions

1. **Q1, the KS-29 corner (owner decision pending, `cut.md:511-514`).** After a lost key-up of a swallowed key, the next press reads as a repeat and fires nothing for a non-repeating binding; on `master` it fired. This is not a lane defect. It is documented (CHANGELOG Fixed KS-10, known-issues row 63) and pinned (`scenarios.rs:1516`). The fact that settles it: whether the owner accepts it or extends the acceptance of item 2.3.
2. **Process.** `10bcf94` was committed on the lane branch with a `Claude Sonnet 5.5` trailer, which points to the tester. `docs/WORKFLOW.md:23` gives the tester "no commits". The content is sound (traced above), but the round-1 pack and `impl.md` describe `ad96e73`. The orchestrator should decide whether that commit stands and whether the tester's report is against `10bcf94`.
3. The book pages outside `book/src/config` (`audit.md`, `roadmap.md`, `architecture.md`, `overview.md`, `glossary.md`, `gaps.md`) still describe KS-10 as open. This is deferred to the post-merge book sync by design (WORKFLOW `ks-phase`, `cut.md:199`).
