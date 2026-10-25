# Implementation: p1-1.3-deferred-block-default, round 1

Verdict: DONE. The deviations from the cut are listed below. Branch `feat/deferred-block-default` was cut from `master` @ `1afa720`; HEAD is `1562acd`. The tree is clean. Line numbers are at `1562acd`.

## Items

- **Commit 1, D2: `KeyCombination::is_modifier_only`.** `src/data/key_combination.rs:70-77`. `is_trigger_key` uses it at `src/engine/matching.rs:58-63`.
  - Unit test `data::key_combination::tests::is_modifier_only_needs_every_key_to_be_a_modifier` (`:153-168`).
  - The commit is a refactor (green). The test cannot be red, because the method does not exist on master and the test would not compile. The proof that behaviour is unchanged: `engine::matching::tests` (`:143-177`) are unchanged and green.
- **Commit 2, declared red: scenarios, harness and loader tests.**
  - Harness: `Scenario::expect_down_in_windows` at `src/engine/testing.rs:475-484`, plus one sentence in the module doc at `:24-30`.
  - Scenario section `// ---- block_default on deferred triggers (roadmap item 1.3) ----` at `src/engine/scenarios.rs:2917-3400`.
  - The flip in `lshift_double_tap_fires_only_for_the_left_shift` (`:901-931`).
  - D9 `block_default: true` at `:1347`, `:2058`, `:2085`, `:2118`, `:2209`, `:2751`, each with a comment.
  - Property configs: `src/engine/properties.rs:100-107` (`gestures`: `block_default: true` on `Ctrl+Shift` Hold, new row `F15` Tap `block_default: false`) and `:133` (`mixed`: `block_default: true` on the `LShift` DoubleTap). The module doc is at `:1-8`.
  - Loader tests and the helper `template_examples` (`src/config/loader.rs:660-756`).
  - Red output, replayed on this commit (log: `D:/tmp/ks-orch/p1-1.3-deferred-block-default/logs/commit2_red.log`). The first failing line of each red test is below, in order.
    - a `explicit_block_default_false_on_a_tap_passes_every_event_and_still_fires`: red "scenario: down F13 at +0 ms: expected the event to be passed, it was suppressed" -> green
    - b `explicit_block_default_false_on_a_hold_passes_every_event_and_still_fires`: red "down F13 at +0 ms: expected the event to be passed, it was suppressed" -> green
    - c `explicit_block_default_false_on_a_double_tap_passes_every_event_and_still_fires`: red "down F13 at +0 ms: ..." -> green
    - d `lshift_double_tap_fires_and_every_lshift_event_passes` (acceptance 1): red "down LShift at +0 ms: expected the event to be passed, it was suppressed" -> green
    - e `lshift_held_with_its_double_tap_bound_types_a_capital_a` (acceptance 2): red "down LShift at +0 ms: ..." -> green
    - h `passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too`: red "down LShift at +0 ms: ..." -> green
    - i `one_long_press_of_a_passing_shift_does_not_fire_its_double_tap`: red "down LShift at +10 ms: ..." -> green
    - j `modifier_only_hold_and_tap_pass_their_keys_by_default`: red "down LShift at +20 ms: ..." -> green
    - k `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default`: red "down LShift at +20 ms: ..." -> green
    - m1 `passing_tap_fires_after_a_foreground_change_and_its_key_up_passes`: red "down F13 at +0 ms: ..." -> green
    - m2 `remapping_off_during_a_passing_hold_drops_it_and_nothing_sticks`: red "down F13 at +0 ms: ..." -> green
    - m3 `reload_during_a_passing_double_tap_keeps_each_press_whole`: red "down LShift at +0 ms: ..." -> green
    - flip `lshift_double_tap_fires_only_for_the_left_shift`: red "down LShift at +0 ms: ..." -> green
    - I1 `a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap`: red "down LShift at +0 ms: ..." -> green
    - I1 `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap`: red "down LCtrl at +0 ms: ..." -> green
    - L1 `config::loader::tests::block_default_defaults_to_false_only_for_deferred_bindings_made_only_of_modifiers`: red "assertion `left == right` failed / left: [true, true, true, true, true, true, true, true, true, true]" -> green
    - L3 `config::loader::tests::block_default_takes_some_and_none_like_the_other_optional_fields`: red "block_default takes Some and None: config parse error (<in-memory>): 2:61: Expected boolean" -> green
    - L4 `config::loader::tests::the_template_lshift_double_tap_example_lets_the_left_shift_through`: red "assertion failed: !example.block_default" -> green
  - Guards that were green on commit 2 and are green on HEAD:
    - f `capslock_tap_with_the_default_block_default_still_swallows_capslock` (acceptance 3);
    - g `explicit_block_default_true_on_a_modifier_only_double_tap_swallows` (acceptance 4);
    - l `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole`;
    - L2 `config::loader::tests::an_explicit_block_default_wins_over_its_default`;
    - s `down_in_windows_check_reports_a_key_windows_does_not_have_down` (should_panic);
    - the seven D9 scenarios and `engine::properties::*` (3 tests).
  - The commit body lists the red set with its failing lines.
- **Commit 3, D1, D3, D4, D5: the fix.**
  - `src/config/schema.rs:43-50`: `#[serde(default)] pub block_default: Option<bool>` with a doc comment. `default_true` is deleted; it had no other user.
  - `src/data/binding.rs:53-57`: the field doc. `:125-136`: `Binding::blocks_by_default(kind, trigger)`. `:286-332`: unit test `blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers`. The test cannot be red, because the function does not exist on master; it is pinned with the loader tests L1, L3 and L4.
  - `src/config/loader.rs:385-389`: `raw.block_default.unwrap_or_else(|| Binding::blocks_by_default(&kind, trigger))`, computed before `kind` moves.
  - `src/engine/mod.rs:434-436`, `:450-452`, `:466-468`: the gate `if binding.block_default { decision.suppress = true; }` for Tap, Hold and DoubleTap. `:124-131`: rule 4 text. `:633-639`: comment of `key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending`.
  - The key-up rule (`:555-581`), the repeat rule, the blocker branch, sort and publish, and `src/hook` are untouched.
  - README, `src/config/default_config.ron` and CHANGELOG are changed in the same commit.
- **Commit 4: config pages of the book.** See Docs.

## Remarks

- **I1 (Important), RESOLVED (option b).** No interrupt rule; D6 stands. The caveat is in four places:
  - the template at `src/config/default_config.ron:273-278`;
  - the CHANGELOG (Changed, Breaking sub-bullet; Fixed entry);
  - the README (the caveat paragraph after the trigger example, and a behaviour-rule bullet);
  - the book (`triggers-conditions.md` §11.8 warning, `known-issues.md` row 67, `kinds.md` §10 practical rules).

  Two scenarios pin it: `a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap` (`src/engine/scenarios.rs:3341`) and `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap` (`:3376`). M1 has the `<T>` and `A B` steps. Details are in the cut.md addendum. The owner question (does the interrupt rule join 2.2 or become a new item?) is open for the orchestrator.
- **M1 (Minor), RESOLVED (documentation option).** Documented in the CHANGELOG, the README, `known-issues.md` row 65, `matching.md` §15.3 and `triggers-conditions.md` §11.8. Pinned by the second config of `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default` (`src/engine/scenarios.rs:3179`), with `block_original_combo: true` on the replacing binding itself. The rejected alternative and its cost are in the cut.md addendum.
- **M2 (Minor), RESOLVED.** Updated `kinds.md` §10 (two table rows and the KS-13 warning), `known-issues.md` row 10, `triggers-conditions.md` §11.3, `actions.md` §13.9 and `matching.md` §16.2. Pinned by `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`src/engine/scenarios.rs:3238`).
- **Critic open question 1 (merge order against 1.2).** Phase-plan scope, so it goes to the orchestrator. The interim behaviour is documented (CHANGELOG Breaking sub-bullet, README, known-issues row 66).
- **Critic open question 2 (M3 wording), RESOLVED.** See M3 below.

## Deviations from the cut

1. **Harness self-test placement.** `down_in_windows_check_reports_a_key_windows_does_not_have_down` sits in the existing `// ---- The harness itself ----` section (`src/engine/scenarios.rs:3412-3418`), next to the other harness self-tests, instead of the new section. Its `should_panic` text is the full message `"down CapsLock at +0 ms: CapsLock is not down in Windows"`.
2. **m3 is stronger.**
   - The reload happens while LShift is still held: down passed, reload to blocking, up passed (the record wins). The cut reloaded after the key-up.
   - A second part runs the other direction: down suppressed, reload to passing, up suppressed, and the next down passes and fires.
   - The first failing line on commit 2 is unchanged (`down LShift at +0 ms`).
3. **k has two passing configs.** The cut's config, plus the same `Alt+Shift` Tap with `block_original_combo: true` on itself (critique M1). The first failing line on commit 2 is unchanged.
4. **Two extra scenarios**, the I1 known issues: red on commit 2, green on HEAD.
5. **Extra checks.** `check_press_symmetry` and `check_nothing_stuck` are on in b, c, l and m1, and the total-fired checks are in a, b, f and l, where the cut listed fewer. Strictly stronger.
6. **`ron-syntax.md` §3.6.**
   - The removed `block_default: Some(true)` gives `Expected boolean` line is replaced by `block_original_combo: Some(true)` gives `Expected boolean`. The validator confirmed the message, so the list still shows a bool field that rejects `Some`.
   - The list of `Option` fields now names `repeat` and `block_default`, with two RON lines, both validated.
7. **Book code links.** The new behaviour is cited by branch name in the text (`feat/deferred-block-default`), as the `feat/repeat-flag` rows do. The existing `1dfd80a` permalinks are kept. The facts line of every touched page names the branch.
8. **`matching.md` §16.2 and the facts lines** of `matching.md`, `actions.md`, `errors.md`, `example.md` and `book/src/config/README.md` are also edited. The cut's touch set listed `matching.md` only through critique M1.

## Unsafe / atomics added

- None. There is no new `unsafe`, atomic, static, lock, thread, timer or `SendInput` path, and `src/hook` is untouched.

## Gates

- At HEAD `1562acd`: `cargo fmt --check` pass, `cargo check --all-targets --locked` pass, `cargo clippy --all-targets --locked -- -D warnings` pass. fmt and clippy were run after every commit.
- Tests run:
  - `cargo test --locked --lib` on commit 2: 18 failed, the declared red set, and 392 passed.
  - `cargo test --locked --lib` on commits 3 and 4: 411 passed, 0 failed, 1 ignored (pre-existing).
  - This covers every test file the diff touches: `engine::scenarios`, `engine::properties`, `engine::tests`, `engine::matching`, `config::loader`, `data::binding`, `data::key_combination`, plus the hook tests in the lib.
  - Logs: `D:/tmp/ks-orch/p1-1.3-deferred-block-default/logs/{commit2_red,commit3_lib,head_lib}.log`.
  - The full `cargo test --locked` run (integration tests included) is left to the tester.
- Manual `#[ignore]` tests added: none. Every acceptance line is expressed through the engine.

## Manual checks (human only, Checkpoint 1 smoke #6 and pitfall rows)

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | Release build, `swch open` from a normal console. Uncomment the template's `LShift` `DoubleTap(250)` example. Open Notepad and `osk.exe`. Record `reg query "HKCU\Keyboard Layout" /v Attributes` (read only) | 1. Hold LShift and type `abc`. 2. LShift+A quickly. 3. Double-tap LShift, type `a`. 4. Double-tap LShift again, type `a`. 5. Double-tap RShift. 6. Hold LShift 2 s and release. 7. Type `<T>` with the left Shift pressed again quickly for the `T`. 8. Type `A B` with the left Shift for both capitals, quickly | 1. `ABC`. 2. `A`. 3. CapsLock turns on, and `a` gives `A`. 4. CapsLock turns off, and `a` gives `a`. 5. RShift toggles nothing. 6. `osk.exe` shows no Shift stuck. 7 and 8 are the known issue (known-issues row 67): CapsLock toggles, so the second capital comes out in lower case and CapsLock stays on. Record whether it happens at normal typing speed. If the toggle misbehaves while Shift is held (KS-19, row 68), note the `Attributes` value |
| M2 | Default config with the Alt+Shift Windows hotkey, plus `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )`. Repeat with `block_original_combo: true` on that binding | Read the startup lines; tap Alt+Shift; add `block_default: true`, save, and tap again | `[INFO] no auto-blocker for Alt+Shift: binding #2 uses the same keys` (`#0` in the second variant). First tap: the layout ends where it started, since Windows switches forward and the binding switches back. Second tap: one switch backward (row 65) |
| M3 | `( keys: "LWin", action: ToggleCapsLock, trigger: Tap(200) )` | Tap LWin. Then add `block_default: true`, tap LWin, and press LWin+E | First: Start opens and CapsLock toggles. Second: CapsLock toggles and Start does not open, but LWin+E types `e`: `block_default: true` itself swallows LWin while the binding matches, and item 1.2 does not change that (row 66) |

## Docs

- **README.md:**
  - the trigger example's comment and the caveat after it;
  - behaviour rules: the `block_default` bullet, plus three pitfall bullets (no cancel by another key; the hotkey's keys, also with the binding's own `block_original_combo`; lone Alt/Win);
  - the flags-table row and the feature bullet.
- **`src/config/default_config.ron`:** the flag comment (`:144-150`) and the LShift example comment (`:273-278`), marked as working, with the caveat.
- **CHANGELOG.md (Unreleased):**
  - Changed: a Breaking entry with four sub-bullets;
  - Fixed: a new entry;
  - the KS-07 entry edited.
- **Book (commit 4):** `kinds.md`, `triggers-conditions.md`, `structure.md`, `README.md`, `errors.md`, `example.md`, `known-issues.md` (rows 10, 52 and 54; new rows 65 to 68; the closing note), `ron-syntax.md`, `actions.md` and `matching.md`.
  - **Validation.** New and changed RON was validated with `cfgcheck` against the real loader (`D:/tmp/ks-orch/p1-1.3-deferred-block-default/cfgcheck/ex/*.ron`). The book's complete example still gives `OK: 26 bindings`.
  - **Build.** `mdbook build book -d D:/tmp/ks-orch/p1-1.3-deferred-block-default/book` passes, and a scan of the built HTML found 0 broken internal links or anchors.
- **Deferred to the Phase 1 book sync (doc-writer), per the cut:** `architecture.md`, `glossary.md`, `overview.md`, `build-and-run.md` and `roadmap.md` (1.3 status, the `:199` expectation, "Shipped").

## Commits

- `5adf19c` Data: KeyCombination::is_modifier_only for the trigger-key rule (1.3)
- `6950476` Engine: red-first: block_default on Tap, Hold and DoubleTap (1.3), declared red, with the failing lines in the body
- `eea58bf` Engine: block_default applies to Tap, Hold and DoubleTap, off by default for modifier-only combos (1.3)
- `1562acd` Book: block_default on deferred triggers in the config reference (1.3)

## Open points / pre-existing warnings

- No pre-existing warnings: clippy with `-D warnings` is clean.
- For the orchestrator:
  - the I1 owner question (interrupt rule: 2.2 acceptance or a new item);
  - critic open question 1 (merge order against 1.2);
  - cut follow-ups F2 (`block_default` accepted and ignored on `sequence`/`chord`), F3 (KS-19), F4 (KS-12), F5 (KS-13) and F6 (stale R8 text in `.claude/agents/code-reviewer.md`, outside this lane's files).
- Apparent dead code: none found in the touch set.
