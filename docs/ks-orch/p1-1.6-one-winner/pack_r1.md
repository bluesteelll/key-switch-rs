# Context pack, round 1: p1-1.6-one-winner (roadmap 1.6, KS-13)

Scout: project-analyst (read-only). For: tester and code-reviewer. Facts only; `UNVERIFIED` marks inference.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/one-winner`, HEAD `f7d54ef`, `git status --short` empty.
- `master` = `9357cfa` = merge-base of `master` and `feat/one-winner`.
- `git log --oneline master..feat/one-winner` (oldest last). Every commit carries the `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` trailer.

| Commit | Subject | Files |
|---|---|---|
| `f7d54ef` | Book: one winner per key event in the config reference (1.6) | 7 `book/src/config/*.md` |
| `ed623c5` | Config: warn about a binding that duplicates an earlier one (1.6) | CHANGELOG, README, `config/loader.rs`, `data/binding.rs`, `data/trigger.rs` |
| `69a8406` | Config: red-first: duplicate bindings are warned about (1.6) | `config/loader.rs` (tests only) |
| `3da80f0` | Config: parse_ron_str keeps the loader's warnings next to the bindings (1.6) | `config/loader.rs` |
| `006ff5e` | Engine: one binding or gesture group wins each key-down (1.6, KS-13) | CHANGELOG, README, `default_config.ron`, `data/{binding,condition,trigger}.rs`, `engine/{mod,scenarios,winner}.rs`, `hook/keyboard_hook.rs` |
| `ede8472` | Engine: red-first: one winner per key event (1.6, KS-13) | `engine/{mod,properties,scenarios}.rs` |

- `git diff --stat master..feat/one-winner`: 19 files, +1310 / -150. Code: `src/engine/winner.rs` (+180, new), `src/engine/mod.rs` (182), `src/engine/scenarios.rs` (+485), `src/engine/properties.rs` (22), `src/config/loader.rs` (145), `src/data/binding.rs` (130), `src/data/trigger.rs` (38), `src/data/condition.rs` (6), `src/hook/keyboard_hook.rs` (9, doc only), `src/config/default_config.ron` (25). Docs: `README.md` (80), `CHANGELOG.md` (+60), `book/src/config/{actions,errors,example,kinds,known-issues,matching,triggers-conditions}.md`.
- Not in the delta: `src/hook/keyboard_hook_callback.rs`, `src/engine/{matching,gestures,keys,chords,sequences,env,testing}.rs`, `src/config/schema.rs`, `Cargo.toml`, `Cargo.lock`.
- Lane inputs: `cut.md` (CUT READY), `critique.md` (APPROVED, Critical 0, Important 0, Minor 3), `impl.md`, `red1.log`, `red4.log`, `cfgcheck/`, `book/` (mdbook output), all under `D:/tmp/ks-orch/p1-1.6-one-winner/`.

## File map

| path:Lstart-Lend | What lives there | Why it matters this round |
|---|---|---|
| `src/engine/winner.rs:1-23` | Module doc: candidates, rank, group, non-candidates | States the 1.6 rule in code |
| `src/engine/winner.rs:33-43` | `Winner { index, binding }`; `rank(b) = (key_count, !condition.is_always())` | Precedence (D2) |
| `src/engine/winner.rs:53-82` | `elect`: skip non-`Combo`, skip `is_auto_blocker`, skip if winner rank `>=`, then `combo_matches`, then `condition.evaluate` | Election, D7 order, tie to earlier |
| `src/engine/winner.rs:93-114` | `Winner::admits`: same index; both deferred; `combo ==`, `condition ==`, `combo_matches` with member's own flag | Gesture group (D3) |
| `src/engine/winner.rs:118-139` | `GroupArms { tap, hold, double_tap }`, `claim` | One per trigger kind (D4) |
| `src/engine/winner.rs:142-180` | Unit tests `rank_puts_more_keys_before_a_when_condition`, `group_arms_claims_each_deferred_kind_once` | |
| `src/engine/mod.rs:112-153` | Module doc rule 4 rewritten (election, group, suppression, blockers/sequence/chord) | Doc vs code |
| `src/engine/mod.rs:176-178` | Known-defects line now lists only KS-15, KS-16 | KS-13 dropped |
| `src/engine/mod.rs:188`, `:215` | `mod winner;`, `use winner::GroupArms;` | |
| `src/engine/mod.rs:282-290` | `sort_by_specificity` doc + stable `sort_by_key(Reverse(key_count))` | R7 |
| `src/engine/mod.rs:388-534` | `on_key_down` election + act loop | Core change |
| `src/engine/mod.rs:536-569` | Repeat / record block (KS-06), unchanged by diff | Suppression invariant |
| `src/engine/mod.rs:572-598` | `on_key_up`, unchanged by diff | Key-up rule |
| `src/engine/mod.rs:627-668` | `ForegroundForbiddenEnv` + W16 test | D7 pin |
| `src/engine/scenarios.rs:258` | Renamed `a_binding_whose_condition_holds_wins_on_a_shared_key` | Only rename |
| `src/engine/scenarios.rs:3238-3259` | `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (comment changed only) | OR inside group |
| `src/engine/scenarios.rs:3402-3877` | New section W1-W14 + M1 repeat scenario | Acceptance |
| `src/engine/properties.rs:4-10`, `:79`, `:143-157` | Doc, `CONFIGS: [Config; 7]`, new `winners` config | W15 |
| `src/data/condition.rs:28-32` | `#[derive(Debug, Clone, Default, PartialEq, Eq)]` on `Condition` + doc | Group and duplicate equality |
| `src/data/trigger.rs:9-14`, `:43-55` | Module doc; `Trigger::same_kind` | Duplicate predicate |
| `src/data/binding.rs:35-40`, `:49-66` | `key_count` doc; `BindingKind::same_keys` | Duplicate predicate |
| `src/data/binding.rs:167-181` | `Binding::duplicates` | Duplicate predicate |
| `src/data/binding.rs:388-460` (approx.) | Tests `same_keys_compares_...`(`:388`), `duplicates_needs_...`(`:413`), `an_auto_blocker_duplicates_nothing`(`:445`) | |
| `src/config/loader.rs:226-240` | `from_ron_str_with_hotkeys` = `parse_ron_str` + `eprintln!("[WARN] {warning}")` per warning | D10 seam, stderr |
| `src/config/loader.rs:242-247` | private `struct ParsedConfig { bindings, warnings }` | |
| `src/config/loader.rs:251-294` | `parse_ron_str` (BOM strip, ron parse, per-entry conversion, early error return, `duplicate_warnings` before `expand_with_auto_blockers`) | |
| `src/config/loader.rs:299-321` (fn `:306`) | `duplicate_warnings`: O(n^2), first earlier match via `position` | Message format |
| `src/config/loader.rs:712-736` | `template_examples` / `template_examples_ron` split | L4 |
| `src/config/loader.rs:1956-1980` | `readme_matching_examples_load` (validates RON blocks under README `### Combo syntax`) | New README block |
| `src/config/loader.rs:2196-2272` | `warnings_of` helper + L1-L4 | Acceptance 5 |
| `src/hook/keyboard_hook.rs:106-115` | `update_bindings` doc changed; code `sort_by_specificity` + one `store` unchanged | R7 |
| `src/hook/keyboard_hook.rs:310-316` | Comment of `equal_specificity_keeps_config_order` | Doc only |
| `src/config/default_config.ron:114-127`, `:155-159`, `:268-271` | Coexist + precedence text; `block_default` comment; CapsLock Tap/Hold example comment | Template docs |
| `README.md:261-284` | "Which binding wins." paragraph + RON block (inside `### Combo syntax`, heading `:203`) | Acceptance doc |
| `README.md:357-366`, `:394-398`, `:420-429`, `:526`, `:605-620`, `:749` | Triggers intro, CapsLock example comment, resolution bullet, behaviour rule, `block_default` row, Errors paragraph (under `### Errors` `:575`), feature bullet | |
| `CHANGELOG.md:56-64` | Added: duplicate warning (under `### Added` `:14`) | |
| `CHANGELOG.md:161-196` | Changed: Breaking one-winner entry (under `### Changed` `:66`) | |
| `CHANGELOG.md:320-334` | Fixed: KS-13 + two DoubleTaps (under `### Fixed` `:198`) | |
| `book/src/config/matching.md:113-140` | §16.2 / §16.3 rewritten | Book |
| `book/src/config/triggers-conditions.md:41-117` | §11.3, 11.4, 11.6, 11.7 | Book |
| `book/src/config/known-issues.md:36-37`, `:63`, `:137-141` | Rows 10, 11, 25 marked fixed; new rows 69-73 | Book |
| `book/src/config/kinds.md:169-170`, `:181-182`; `actions.md:257-260`; `errors.md:175`, `:203`; `example.md:126-130` | Book rows | Book |

## Facts

Election and act loop

1. `elect` is called once per key-down, after `self.keys.press(vk)` (`src/engine/mod.rs:390`) and before the act loop (`src/engine/mod.rs:397`); `GroupArms::default()` is created at `src/engine/mod.rs:398`.
2. `elect` considers only `BindingKind::Combo` (`src/engine/winner.rs:61-63`) and skips `is_auto_blocker` (`src/engine/winner.rs:64-66`).
3. `elect` skips a candidate when the current winner's rank is `>=` its rank, before calling `combo_matches` (`src/engine/winner.rs:67-75`); `condition.evaluate` runs only after `combo_matches` succeeds (`src/engine/winner.rs:76-78`).
4. `rank` is `(binding.kind.key_count(), !binding.condition.is_always())` (`src/engine/winner.rs:41-43`); `is_always` is `matches!(self, Condition::Always)` (`src/data/condition.rs:61-63`).
5. `admits` returns `true` for `index == self.index` (`src/engine/winner.rs:100-102`), `false` if either trigger is Immediate (`:103-105`), else requires `combo == winner_combo`, `binding.condition == self.binding.condition` and `combo_matches(.., binding.allow_extra_modifiers)` (`:106-113`).
6. `KeyCombination` equality is length-equal plus containment, order-insensitive (`src/data/key_combination.rs:16-30`).
7. In the act loop the auto-blocker branch still evaluates its own `when:` first and then `blocker_matches` (`src/engine/mod.rs:405-422`); it sets `decision.suppress` and `continue`s, so it never reaches `admits`.
8. A non-blocker `Combo` `continue`s unless `winner.is_some_and(|w| w.admits(index, binding, vk, &self.keys))` (`src/engine/mod.rs:424-429`).
9. Immediate arm: fires when `!was_repeat || binding.repeat`, suppresses if `block_default` (`src/engine/mod.rs:431-438`); the master `immediate_fired` flag is gone (master `src/engine/mod.rs:377`).
10. Tap / Hold / DoubleTap arms each `continue` on `was_repeat || !arms.claim(binding.trigger)` before arming and before reading `block_default` (`src/engine/mod.rs:440`, `:454`, `:470`).
11. `GroupArms::claim` returns `true` for `Immediate` always and `!mem::replace(flag, true)` per deferred kind (`src/engine/winner.rs:131-139`).
12. `Sequence` and `Chord` branches check `was_repeat || !binding.condition.evaluate(..)` before their feed (`src/engine/mod.rs:487-492`, `:514-517`); on master the `when:` check ran for every binding at the loop top before the kind match (master `src/engine/mod.rs:382-384`) and the repeat check sat inside each branch.
13. The repeat/record block (`src/engine/mod.rs:536-569`) and `on_key_up` (`src/engine/mod.rs:572-598`) have no hunks in `git diff master..feat/one-winner -- src/engine/mod.rs`.
14. No allocation, lock, atomic, thread or `static` is introduced in `winner.rs` (`src/engine/winner.rs:1-140`, by reading); the act loop's `binding.action.clone()` push (`src/engine/mod.rs:433`) predates the lane.
15. `src/hook/keyboard_hook_callback.rs` is not in the diff; the engine is entered through `hook.with_engine` (`src/hook/keyboard_hook_callback.rs:178`), the sentinel check is at `:166`, `code < 0` at `:238`.
16. `update_bindings` still does `engine::sort_by_specificity` then one `ArcSwap::store` (`src/hook/keyboard_hook.rs:110-115`); the sort is `sort_by_key(|b| Reverse(b.kind.key_count()))` (`src/engine/mod.rs:288-290`).
17. Scenario harness `from_ron` / `reload` load through `from_ron_str` (registry read of the system hotkey, no write) and call `sort_by_specificity` (`src/engine/testing.rs:164-166`, `:237-243`).

Existing tests touched

18. The only edits to pre-existing scenarios are the rename at `src/engine/scenarios.rs:258` (from `first_binding_whose_condition_holds_wins_on_a_shared_key`) and the comment at `src/engine/scenarios.rs:3239-3241`; no pre-existing expectation line changed (`git diff master..feat/one-winner -- src/engine/scenarios.rs` hunks `-255`, `-3236`, `-3399` (append)).
19. `properties.rs` `CONFIGS` grows from 6 to 7 with `winners` (`blocker: false`) at `src/engine/properties.rs:143-157`; the property key set is `LShift RShift LCtrl LAlt CapsLock F13 F14 F15 F16 J K Space A` (`src/engine/properties.rs:62-65`); the repeat check allows only `PressKey(VK_LEFT)` (`src/engine/properties.rs:242`).
20. The `winners` config contains `F15` with `when: Not(AppEquals("game.exe"))` (`src/engine/properties.rs:155`); whether the property runs ever set a foreground app is UNVERIFIED (`foreground_changed` at `src/engine/properties.rs:216`; `FakeEnv` starts with `app: None`, `src/engine/testing.rs:73-79`).

Duplicate warning

21. `Binding::duplicates` = neither is auto-blocker, `kind.same_keys`, `trigger.same_kind`, `condition ==`, `allow_extra_modifiers ==` (`src/data/binding.rs:174-181`); action and `block_default`, `repeat`, `block_original_combo` are not compared.
22. `same_keys`: `Combo` by `KeyCombination ==`; `Sequence` by `Vec<KeyCombination> ==` (element-wise, in order; each step order-insensitive); `Chord` by length + containment; cross-kind `false` (`src/data/binding.rs:53-66`); `max_gap` / `window` ignored.
23. `Trigger::same_kind` is a `matches!` over the four same-kind pairs (`src/data/trigger.rs:47-55`).
24. `Condition` derives `PartialEq, Eq` (`src/data/condition.rs:32`); equality is structural, case-sensitive in strings (doc `src/data/condition.rs:28-31`), while `evaluate` compares app names case-insensitively (`src/data/condition.rs:68-70`).
25. `duplicate_warnings` runs on the user bindings in file order, before `expand_with_auto_blockers` and before any sort (`src/config/loader.rs:290-296`), and names only the first earlier match (`src/config/loader.rs:309-311`).
26. Message format: `binding #{later} [{keys}] duplicates binding #{earlier} [{keys}]; remove one of them`, keys via `format_raw_binding_keys(&raw[..])` (`src/config/loader.rs:313-317`); function at `src/config/loader.rs:306-321`, doc `:299-305`.
27. Warnings are printed with `eprintln!("[WARN] {warning}")` in `from_ron_str_with_hotkeys` (`src/config/loader.rs:236-238`); the existing `[WARN] config contains no bindings` is still printed directly inside `parse_ron_str` and is not in `warnings` (`src/config/loader.rs:286-288`).
28. Any conversion error returns before `duplicate_warnings` (`src/config/loader.rs:282-284`), so `raw.bindings[k]` and `user_bindings[k]` index the same entry (comment `src/config/loader.rs:290-291`, call `:292`).
29. L1 asserts `parsed.bindings.len() == 3` and exactly one warning (`src/config/loader.rs:2206-2220`); L3 expects 5 warnings incl. `\u{2192}` in sequence keys (`src/config/loader.rs:2243-2265`); L4 asserts no warnings for `DEFAULT_CONFIG` and `template_examples_ron()` (`src/config/loader.rs:2268-2271`).

Docs

30. README "Which binding wins." sits at `README.md:261-276` with a RON block `README.md:278-284`, inside `### Combo syntax` (`README.md:203`), so `readme_matching_examples_load` (`src/config/loader.rs:1956`) loads it.
31. README links `#combo-syntax-the-keys-field` (`README.md:359`, `:526`, `:615`) and `#errors` (`README.md:276`); headings `README.md:203` and `README.md:575`.
32. `book/src/config/errors.md:175` links code at `blob/ed623c5/src/config/loader.rs#L299-L321`; per `impl.md` the book links point at `ed623c5`.
33. `known-issues.md` adds rows 69-73 at `book/src/config/known-issues.md:137-141` (cut planned 69-72; row 73 is the M1 repeat case).
34. `audit.md:36` still reads `Open: item 1.6`; `roadmap.md:617` still reads `Open` for KS-13 (deferred to the Phase-1 book sync per `impl.md` Open points).

## Red-first tests and acceptance proofs

Red runs (from `impl.md` and the logs; the cargo filter is inferred from the test names in each log):

| Run | Tree | Command (inferred) | Expected result | Log |
|---|---|---|---|---|
| Red 1 | `ede8472` (master code + commit-1 tests) | `cargo test --locked --lib engine::` | `test result: FAILED. 211 passed; 12 failed` | `red1.log:353`; failure list `red1.log:339-351` |
| Red 4 | `69a8406` (commit-3 code + L1-L4) | `cargo test --locked --lib config::loader::tests::` | `test result: FAILED. 78 passed; 2 failed` | `red4.log:109`; failures `red4.log:106-107` |

Red 1 set (each red on `ede8472`, green on `f7d54ef`; first failing line from `impl.md`):

| ID | Test (`engine::scenarios::` unless named) | Line | Red line on master code |
|---|---|---|---|
| W1 | `shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers` | `:3412` | `up CapsLock at +70 ms: expected to fire [], fired [SwitchLanguage]` |
| W2 | `shift_capslock_held_past_the_term_fires_no_capslock_hold` | `:3443` | `advance 300 ms to +320 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]` |
| W3 | `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order` | `:3469` | context `unconditional binding first`: `down Pause at +0 ms: expected to fire [Media(PlayPause)], fired [SwitchLanguage]` |
| W4 | `a_more_specific_winner_with_block_default_false_lets_the_key_through` | `:3509` | `down F13 at +0 ms: expected the event to be passed, it was suppressed` |
| W5 | `a_conditional_winner_with_block_default_false_lets_the_key_through` | `:3540` | `down F14 at +0 ms: expected the event to be passed, it was suppressed` |
| W6 | `a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing` | `:3568` | `down F22 at +0 ms: expected the event to be passed, it was suppressed` |
| W7 | `a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding` | `:3609` | `down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W8 | `a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together` | `:3652` | `advance 300 ms to +300 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]` |
| W9 | `two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press` | `:3689` | `down F13 at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W10 | `a_more_specific_tap_keeps_a_less_specific_hold_from_arming` | `:3715` | `advance 300 ms to +300 ms: expected to fire [], fired [SwitchLanguage]` |
| W14 | `tap_and_hold_on_capslock_still_arm_together_as_one_group` | `:3812` | `down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W16 | `engine::tests::a_when_condition_is_asked_only_for_a_binding_whose_keys_match` | `src/engine/mod.rs:645` | panic `the foreground was asked for although no binding with a when: matches this key` |

Green guards on both trees: W11 `a_gesture_armed_by_the_winner_resolves_after_a_reload_adds_a_more_specific_binding` (`:3743`), W12 `a_passing_user_binding_does_not_stop_the_auto_blocker` (`:3768`), W13 `a_group_member_arms_only_if_it_matches_the_press_itself` (`:3788`), W15 the three `engine::properties` tests (`src/engine/properties.rs:266`, `:271`, `:276`) with `winners`.

Added in commit 2, not in the red set and not replayed on master: `repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` (`src/engine/scenarios.rs:3843-3877`); `impl.md` states it would fail on master at its first `down J ... expect_passed`.

Red 4 set (`config::loader::tests::`): L1 `a_binding_that_duplicates_an_earlier_one_is_warned_about` (`:2206`) red `assertion left == right failed / left: [] / right: ["binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them"]`; L3 `duplicates_are_found_across_key_order_terms_timings_and_binding_kinds` (`:2243`) red `left: []`. Green guards: L2 `bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates` (`:2223`), L4 `the_template_and_its_examples_have_no_duplicate_bindings` (`:2268`).

Unit tests added: `engine::winner::tests::{rank_puts_more_keys_before_a_when_condition, group_arms_claims_each_deferred_kind_once}` (`src/engine/winner.rs:151-179`); `data::trigger::tests::same_kind_ignores_the_term` (`src/data/trigger.rs:83`); `data::binding::tests::{same_keys_compares_combos_and_chords_as_sets_and_steps_in_order, duplicates_needs_the_same_kind_keys_trigger_kind_when_and_flag, an_auto_blocker_duplicates_nothing}` (`src/data/binding.rs:388`, `:413`, `:445`).

Acceptance mapping (spec line -> proof, from `cut.md:392-406`):

| Acceptance | Proof |
|---|---|
| Shift+CapsLock (Immediate) + CapsLock (Tap, `allow_extra_modifiers`) fires only ToggleCapsLock | W1, W2 |
| Conditional Pause wins in either file order | W3 |
| Winner with `block_default: false` passes although a loser has `true` | W4 (more keys), W5 (condition), W6 (deferred winner) |
| Tap(200)+Hold(200) on CapsLock arm as one group | W14; existing `tap_and_hold_with_equal_terms_fire_exactly_one` (`scenarios.rs:441`), `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` (`:492`); W7 |
| Duplicates warned in a loader test | L1, L3 (L2, L4 negatives) |
| Auto-blockers keep their rule | W12; `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (`:2632`), `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` (`:2662`) |
| Ties go to the earlier binding | `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`:1765`) |
| Suppression symmetry | `check_press_symmetry` in W1-W14; W15 |

Regression review list named by the cut (`cut.md:367`): `more_specific_immediate_binding_wins` (`:122`), `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`:1765`), `tap_and_hold_with_equal_terms_fire_exactly_one` (`:441`), `shift_held_by_a_double_tap_still_counts_for_shift_capslock` (`:2052`), `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (`:2632`), `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` (`:2662`), `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default` (`:3179`), `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`:3238`), `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` (`:1832`).

Commands (run in `D:/key-switch-rs/key-switch-rs`; no daemon, no `--ignored`):

```
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                                   # full suite (tester)
cargo test --locked --lib engine::                    # impl.md: 226 passed on f7d54ef
cargo test --locked --lib config::                    # impl.md: 93 passed
cargo test --locked --lib data::                      # impl.md: 56 passed, 1 ignored (pre-existing manual)
cargo test --locked --lib hook::keyboard_hook::tests  # impl.md: 10 passed
mdbook build book -d D:/tmp/ks-orch/p1-1.6-one-winner/book   # impl.md: 23 pages, 0 broken links
```

- Expected stderr noise: scenario configs that are duplicates on purpose (W9; `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`) print `[WARN] ... duplicates ...` in captured output (`impl.md` Gates).
- Manual ignores added: none. Manual checks M1-M5 for a human only (`cut.md:408-418`); none run.

## Open points (from `impl.md`)

1. Full `cargo test --locked` not run by the developer; left to the tester.
2. Deferred to the doc-writer's Phase-1 book sync: `architecture.md:207,212,543`, `glossary.md:85,401`, `gaps.md:185`, `overview.md:17`, `roadmap.md:283-289` status and `:617`, `audit.md:36,394` status lines; plus `kinds.md:91` could mention the new duplicate warning.
3. Follow-ups unchanged from the cut: F1 sequence/chord in the election (3.2/3.4); F2 warning for Immediate vs deferred on the same keys and `when:` (3.4); F3 semantic `when:` equality; F4 per-press winner lock (pinned by the M1 scenario); F5 blocker-replacement warning (1.2); F6 stale reviewer R8 text (`.claude/agents/code-reviewer.md`).
4. Deviations declared: README paragraph placed after the AltGr note (still in `### Combo syntax`); extra M1 repeat scenario; known-issues rows 69-73 instead of 69-72; Sequence/Chord check order `was_repeat` before `evaluate`; `admits` does not test `is_auto_blocker` (caller `continue`s earlier, `src/engine/mod.rs:405-422`); CHANGELOG KS-13 entry adds a D7 sentence; template comment qualified (`default_config.ron:121-127`); per-commit gate ran module filters, not the full suite; book links pinned to `ed623c5` (reachable on GitHub only after the merge push).
5. Critique M1-M3 marked RESOLVED (`impl.md` Remarks); M3 settled in docs only (`README.md:614-617`, CHANGELOG Added, `errors.md:175`).
6. Working-copy files written with LF; `core.autocrlf=true` normalises on commit (`impl.md`).

## Invariants in play

- R1 hook callback: callback unchanged (fact 15); engine work added per key-down is `elect` (one extra list walk, no allocation) at `src/engine/mod.rs:397`, `src/engine/winner.rs:53-82`; `when:` evaluation moved after key match (`src/engine/winner.rs:73-78`) - KS-08 budget, pinned by W16.
- R2 sentinel: no new `SendInput` path (no diff in `src/hook/keyboard_hook_callback.rs`, `src/core/`, `src/data/sequence.rs`, `src/system/`).
- R3 shared state: no new `static`/lock; engine reached via `with_engine` (`src/hook/keyboard_hook_callback.rs:178`).
- R4 atomics: none added.
- R5 builders: no new field or setter; new query methods only (`src/data/trigger.rs:47`, `src/data/binding.rs:53`, `:174`).
- R6 Display: no new enum variant; `Condition` gains derives only (`src/data/condition.rs:32`).
- R7 specificity order: sort + store unchanged (`src/hook/keyboard_hook.rs:110-115`); precedence now from `rank` (`src/engine/winner.rs:41-43`); tie-break relies on stable sort (`src/engine/mod.rs:288-290`); blockers skipped in election (`src/engine/winner.rs:64-66`) - KS-18.
- R8 key identity: group and duplicate equality via `KeySpec` equality in `KeyCombination ==` (`src/data/key_combination.rs:16-30`); `LShift+F13` vs `Shift+F13` are different keys (known-issues row 71, `book/src/config/known-issues.md:139`).
- R9 registry: tests read the system hotkey via `from_ron_str` (`src/engine/testing.rs:166`); loader tests use `&|_| None` (`src/config/loader.rs:2200`).
- R10 shutdown: warnings printed during `config::load` on main thread or `config-watcher` (`src/config/loader.rs:236-238`); no hook-lifecycle change.
- Matching/suppression: KS-13 (target); KS-06 repeat record unchanged (`src/engine/mod.rs:536-569`); suppression invariant held by records (`src/engine/keys.rs:158-198`).
- Timers: KS-11/25 untouched; `arm_tap`/`arm_hold` first-wins per vk (`src/engine/gestures.rs:94`, `:128`); `double_tap` (`src/engine/gestures.rs:200`) now guarded per group by `GroupArms`.
- Config/reload: duplicates warned, file still loads (`src/config/loader.rs:290-296`); auto-blocker expansion order unchanged.

## Not covered

- Did not run any cargo command, mdbook or cfgcheck; all pass counts above are quoted from `impl.md` and the red logs.
- Did not read `src/engine/matching.rs`, `gestures.rs`, `keys.rs`, `chords.rs`, `sequences.rs` bodies (only signature lines).
- Did not read the scenario bodies W1-W14 line by line (only names, lines and the M1 body `:3843-3877`); scripts are as specified in `cut.md:339-356` per `impl.md`.
- Did not read the book diffs' prose in full, `CHANGELOG.md:161-196` and `:320-334` in full, or validate links/anchors.
- Did not check whether a combo with a repeated key could make `KeyCombination ==` asymmetric (length + containment) for `same_keys`.
- Did not read `critique.md` beyond its header (`critique.md:1-20`).
