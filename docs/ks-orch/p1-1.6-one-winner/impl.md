# Implementation: p1-1.6-one-winner round 1

Branch `feat/one-winner`, cut from `master` @ `9357cfa`; head `f7d54ef`; tree clean. Cut: `cut.md` (CUT READY). Critique: `critique.md` (APPROVED, Critical 0, Important 0, Minor 3). Red logs: `red1.log` (commit 1 on master code), `red4.log` (commit 4 on commit-3 code).

## Items

- **Roadmap 1.6 / KS-13, election and group (D1-D8)**
  - `src/engine/winner.rs:1-180` (new): `Winner`, `rank` (`:41-43`), `elect` (`:44-82`), `Winner::admits` (`:84-115`), `GroupArms::claim` (`:117-140`), unit tests `rank_puts_more_keys_before_a_when_condition`, `group_arms_claims_each_deferred_kind_once`.
  - `src/engine/mod.rs:388-570` `on_key_down`: `winner::elect` once after `keys.press` (`:397`); `immediate_fired` removed; the `when:` check moved into the blocker (`:405-422`), `Sequence` (`:487-492`) and `Chord` branches; a non-blocker `Combo` continues unless `admits` (`:424-429`); the Immediate arm fires on `!was_repeat || repeat` (`:431-438`); each deferred arm continues on `was_repeat || !arms.claim(..)` (`:439-484`). From the repeat/record block on (`:536-569`) and `on_key_up` unchanged. Docs: rule 4 (`:112-153`), known defects (`:176-178`), `sort_by_specificity` (`:282-290`).
  - `src/data/condition.rs:28-33`: `#[derive(Debug, Clone, Default, PartialEq, Eq)]` plus the structural-equality doc.
  - Doc lines: `src/data/trigger.rs:9-14`, `src/data/binding.rs:35-40`, `src/hook/keyboard_hook.rs:106-109` and `:312-315` (M2).
  - Red-first (commit 1 `ede8472`, run on master code), first failing line of each:
    - W1 `shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers`: red "up CapsLock at +70 ms: expected to fire [], fired [SwitchLanguage]" -> green
    - W2 `shift_capslock_held_past_the_term_fires_no_capslock_hold`: red "advance 300 ms to +320 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]" -> green
    - W3 `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order`: red (context `unconditional binding first`) "down Pause at +0 ms: expected to fire [Media(PlayPause)], fired [SwitchLanguage]" -> green
    - W4 `a_more_specific_winner_with_block_default_false_lets_the_key_through`: red "down F13 at +0 ms: expected the event to be passed, it was suppressed" -> green
    - W5 `a_conditional_winner_with_block_default_false_lets_the_key_through`: red "down F14 at +0 ms: expected the event to be passed, it was suppressed" -> green
    - W6 `a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing`: red "down F22 at +0 ms: expected the event to be passed, it was suppressed" -> green
    - W7 `a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding`: red "down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]" -> green
    - W8 `a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together`: red "advance 300 ms to +300 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]" -> green
    - W9 `two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press`: red "down F13 at +0 ms: expected to fire [], fired [ToggleCapsLock]" -> green
    - W10 `a_more_specific_tap_keeps_a_less_specific_hold_from_arming`: red "advance 300 ms to +300 ms: expected to fire [], fired [SwitchLanguage]" -> green
    - W14 `tap_and_hold_on_capslock_still_arm_together_as_one_group`: red "down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]" -> green
    - W16 `engine::tests::a_when_condition_is_asked_only_for_a_binding_whose_keys_match`: red panic "the foreground was asked for although no binding with a when: matches this key" -> green
    - Green guards on both trees: W11 `a_gesture_armed_by_the_winner_resolves_after_a_reload_adds_a_more_specific_binding`, W12 `a_passing_user_binding_does_not_stop_the_auto_blocker`, W13 `a_group_member_arms_only_if_it_matches_the_press_itself`, W15 the three `engine::properties` tests with the new `winners` config (`properties.rs:143-157`).
    - Every red matched the cut's predicted line exactly. The 12 failures were the only failures of `cargo test --lib engine::` on commit 1 (211 passed).
  - Scenario section `src/engine/scenarios.rs:3402-3877`; rename `first_binding_whose_condition_holds_wins_on_a_shared_key` -> `a_binding_whose_condition_holds_wins_on_a_shared_key` (`:258`); comment of `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`:3239-3241`).
- **Roadmap 1.6, duplicate warning (D9, D10)**
  - Seam (commit 3 `3da80f0`): `src/config/loader.rs:226-294`, `from_ron_str_with_hotkeys` prints `[WARN] {warning}` per warning; `ParsedConfig` (`:244-247`); `parse_ron_str` (`:251-294`).
  - Predicates (commit 5 `ed623c5`): `Trigger::same_kind` (`src/data/trigger.rs:43-55`), `BindingKind::same_keys` (`src/data/binding.rs:49-66`), `Binding::duplicates` (`src/data/binding.rs:167-181`); `duplicate_warnings` (`src/config/loader.rs:299-321`), called at `:289-295` before `expand_with_auto_blockers`.
  - Unit tests: `data::trigger::tests::same_kind_ignores_the_term`, `data::binding::tests::same_keys_compares_combos_and_chords_as_sets_and_steps_in_order`, `duplicates_needs_the_same_kind_keys_trigger_kind_when_and_flag`, `an_auto_blocker_duplicates_nothing`.
  - Red-first (commit 4 `69a8406`, run on commit-3 code), `config::loader::tests::` at `src/config/loader.rs:2196-2272`:
    - L1 `a_binding_that_duplicates_an_earlier_one_is_warned_about`: red "assertion `left == right` failed / left: [] / right: [\"binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them\"]" -> green
    - L3 `duplicates_are_found_across_key_order_terms_timings_and_binding_kinds`: red "assertion `left == right` failed / left: []" -> green
    - L2 `bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates` and L4 `the_template_and_its_examples_have_no_duplicate_bindings`: green guards. `template_examples_ron` split out of `template_examples` (`:712-736`).

## Remarks

- critique M1 (Repeats risk sentence wrong) - RESOLVED: corrected sentence in the `cut.md` addendum; behaviour pinned by `engine::scenarios::repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` (`src/engine/scenarios.rs:3843-3877`, commit 2) and documented in CHANGELOG (Breaking, last consequence), README behaviour rules (`block_default` bullet), `triggers-conditions.md` 11.3 and `known-issues.md` row 73.
- critique M2 (stale comments in `keyboard_hook.rs`) - RESOLVED at `src/hook/keyboard_hook.rs:106-109` and `:312-315` (doc only), commit 2. Guard: `hook::keyboard_hook::tests::equal_specificity_keeps_config_order` (unchanged body, green).
- critique M3 (duplicate warning can drop an auto-blocker) - RESOLVED in the docs: README Errors paragraph and CHANGELOG Added (commit 5), `book/src/config/errors.md` 18.6 row (commit 6). L1-L4 unchanged. Excluding such pairs rejected (it would hide a binding that never acts).
- critic's open question on Q1 (a passing winner and a sequence/chord on the same key) - addressed in `matching.md` 16.2 next to the suppression rule, and in README "Which binding wins".

## Deviations from the cut

- README "Which binding wins" sits after the AltGr paragraph instead of directly after the exact-modifiers RON block, so the AltGr note stays with exact modifiers. It is still inside "### Combo syntax", so `readme_matching_examples_load` validates its RON block.
- Added scenario `repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` in commit 2 (the critic called an M1 pin optional). It is not in the commit-1 red set. On master it would fail at its first `down J ... expect_passed` because the losing `J` binding swallows the key-down, the mechanism W6 shows red; it was not replayed on master.
- `known-issues.md` gets rows 69-73 (73 is the M1 case) instead of 69-72.
- In the `Sequence` and `Chord` branches the check is `was_repeat || !condition.evaluate(..)`: still before the feed, but the cheap repeat test first (both pure within one event, so no decision changes; a repeat no longer evaluates a sequence/chord `when:`).
- `Winner::admits` does not test `is_auto_blocker`: the only caller reaches it after the blocker branch has `continue`d.
- CHANGELOG Fixed KS-13 entry adds one sentence on the D7 effect (a `keys` binding's `when:` is evaluated only once its keys match), without citing KS-08 as fixed (KS-08 is about actions on the hook thread).
- Template comment (`default_config.ron:121-127`) qualifies the Immediate/deferred sentence: they never act on the same press, and with the same `when:` the higher one wins.
- The cut's per-commit gate names `cargo test --locked`; per the developer role the full suite is the tester's. Per commit I ran the touched modules (below).
- Book code links point at `ed623c5` (the last code commit of the lane); the commit is reachable on GitHub once `master` is pushed by the merge step.

## Unsafe / atomics added

- None. No `unsafe`, atomic, lock, `static`, thread, timer or `SendInput` path added. `Winner` borrows one binding for one key-down; `GroupArms` is three bools on the stack; no allocation per key-down.

## Gates

- `cargo fmt --check`: pass; `cargo check --all-targets --locked`: pass; `cargo clippy --all-targets --locked -- -D warnings`: pass (on `f7d54ef`, and after every commit).
- Tests run on `f7d54ef`: `cargo test --locked --lib engine::` (226 passed), `config::` (93 passed), `data::` (56 passed, 1 ignored pre-existing manual test), `hook::keyboard_hook::tests` (10 passed). Full suite left to the tester.
- Scenario configs that are duplicates on purpose (W9, `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`) print a `[WARN] ... duplicates ...` line into captured test output; harmless.
- Manual ignores added: none (every acceptance line is an engine scenario or a loader test). Manual checks for a human, from the cut: M1 (Shift+CapsLock with the CapsLock Tap/Hold pair that allows extra modifiers), M2 (F9 conditional vs unconditional, both file orders, with reload), M3 (Ctrl+Z passing winner over a `Z` DoNothing loser), M4 (copy of the first default binding: `[WARN] binding #2 [CapsLock] duplicates binding #0 [CapsLock]; remove one of them` at reload and startup), M5 (README Tap/Hold pair replacing the default CapsLock). Not run by me (they need the daemon).

## Docs

- README: "Which binding wins" paragraph + RON block; triggers intro and CapsLock example comment ("replace the default CapsLock binding"); resolution bullet; behaviour rule replacing the OR sentence, with the M1 repeat case; `block_default` row; feature bullet; Errors paragraph quoting the duplicate warning with the `block_original_combo` caveat; one sentence pointing to it.
- `src/config/default_config.ron`: coexist text + precedence (`:114-127`), `block_default` comment, CapsLock Tap/Hold example comment ("Replace the CapsLock binding at the top with these two").
- CHANGELOG `[Unreleased]`: Added (duplicate warning), Changed (Breaking: one winner, six consequences), Fixed (KS-13; two DoubleTaps).
- Book: `book/src/config/{matching,triggers-conditions,kinds,actions,known-issues,errors,example}.md` per the touch set. RON validated with cfgcheck (`D:/tmp/ks-orch/p1-1.6-one-winner/cfgcheck`, inputs in `cfgcheck/ex/`): README blocks, CHANGELOG and README examples, the M1/M3 configs, the full `example.md` config (26 bindings, no warning). `mdbook build book -d D:/tmp/ks-orch/p1-1.6-one-winner/book` builds; internal links and anchors of the output: 23 pages, 0 broken.

## Commits

- `ede8472` Engine: red-first: one winner per key event (1.6, KS-13)
- `006ff5e` Engine: one binding or gesture group wins each key-down (1.6, KS-13)
- `3da80f0` Config: parse_ron_str keeps the loader's warnings next to the bindings (1.6)
- `69a8406` Config: red-first: duplicate bindings are warned about (1.6)
- `ed623c5` Config: warn about a binding that duplicates an earlier one (1.6)
- `f7d54ef` Book: one winner per key event in the config reference (1.6)

## Open points / pre-existing warnings

- Pre-existing warnings: none from fmt, check or clippy.
- Deferred to the doc-writer's Phase-1 book sync, as the cut says: `architecture.md:207,212,543`, `glossary.md:85,401`, `gaps.md:185`, `overview.md:17`, `roadmap.md:283-289` status and `:617`, `audit.md:36,394` status lines (Fixed in `<sha>` on `feat/one-winner`). Also `kinds.md:91` (identical sequences share one pending entry) could mention that the loader now warns about exact duplicates.
- Follow-ups from the cut, unchanged: F1 sequence/chord in the election (3.2/3.4), F2 warning for Immediate vs deferred on the same keys and `when:` (3.4), F3 semantic `when:` equality, F4 per-press winner lock (now pinned by the M1 scenario), F5 blocker-replacement warning (1.2), F6 stale reviewer R8 text.
- Some working-copy files were written with LF line endings (`core.autocrlf=true`); git normalises on commit, so the committed blobs are unaffected.
