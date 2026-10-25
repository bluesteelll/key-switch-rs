# Context pack r1: p1-1.3-deferred-block-default (roadmap item 1.3)

For: tester and code-reviewer, round 1. Scout: project-analyst. Facts only; anything inferred is marked UNVERIFIED.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/deferred-block-default`, HEAD `1562acd`; `git status --short` is empty (clean tree).
- Base: `master` = `1afa720` (Merge branch 'feat/repeat-flag', KS-10). The cut was written against `1afa720` (cut.md:3).
- `git log --oneline master..feat/deferred-block-default`:
  - `1562acd` Book: block_default on deferred triggers in the config reference (1.3)
  - `eea58bf` Engine: block_default applies to Tap, Hold and DoubleTap, off by default for modifier-only combos (1.3)
  - `6950476` Engine: red-first: block_default on Tap, Hold and DoubleTap (1.3)  (declared red)
  - `5adf19c` Data: KeyCombination::is_modifier_only for the trigger-key rule (1.3)
- `git diff --stat master...feat/deferred-block-default`: 22 files, +897 / -111.

| File | +/- |
|---|---|
| CHANGELOG.md | 31 |
| README.md | 47 |
| book/src/config/{README,actions,errors,example,kinds,known-issues,matching,ron-syntax,structure,triggers-conditions}.md | 4, 6, 5, 7, 15, 12, 6, 10, 8, 22 |
| src/config/default_config.ron | 18 |
| src/config/loader.rs | 95 |
| src/config/schema.rs | 14 |
| src/data/binding.rs | 65 |
| src/data/key_combination.rs | 34 |
| src/engine/matching.rs | 8 |
| src/engine/mod.rs | 36 |
| src/engine/properties.rs | 10 |
| src/engine/scenarios.rs | 535 |
| src/engine/testing.rs | 20 |

- Not in the delta: anything under `src/hook/`, `src/core/`, `src/system/`, `src/ipc.rs`, `src/main.rs`, `src/bin/`, `Cargo.toml`, `Cargo.lock`, `tests/`.
- Lane inputs: `cut.md` (verdict CUT READY, cut.md:7), `critique.md` (verdict CHANGES_REQUESTED, Critical 0 / Important 1 / Minor 2, critique.md:2), `impl.md` (verdict DONE, impl.md:3). Logs: `logs/commit2_red.log`, `logs/commit3_lib.log`, `logs/head_lib.log` in the lane directory.

## File map

Line numbers at `1562acd`.

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/config/schema.rs:L43-L50` | `RawBinding::block_default: Option<bool>` with `#[serde(default)]` and doc comment | Spec "becomes Option<bool>"; `fn default_true` deleted from the end of the file (was `:229-231` on master) |
| `src/config/loader.rs:L341-L395` | `raw_binding_to_binding`; `kind` at `:345`, `trigger` at `:350`, resolution at `:385-387`, `with_block_default` at `:389` | Where `None` becomes the default (D1) |
| `src/config/loader.rs:L238-L243` | `IMPLICIT_SOME` extension | Why `block_default: false`, `Some(false)` and `None` all load (D5, test L3) |
| `src/config/loader.rs:L271-L312` | `expand_with_auto_blockers` (dedup; `[INFO] no auto-blocker` println at `:303`) | Unchanged code; doc `:276-279` says the replacing binding's own `block_default` decides (D7, scenario k) |
| `src/config/loader.rs:L657-L756` | test helper `template_examples` (`:662`), tests `default_config_commented_examples_load` (`:681`), L4 (`:691`), L1 (`:706`), L2 (`:731`), L3 (`:746`) | Red-first loader tests |
| `src/data/binding.rs:L53-L57` | field doc of `Binding::block_default: bool` | Constructors keep `true` (D4) |
| `src/data/binding.rs:L97-L119` | `with_kind` (`block_default: true` at `:102`), `new_auto_blocker` (`:115`) | Unchanged; programmatic bindings keep `true` |
| `src/data/binding.rs:L125-L136` | `Binding::blocks_by_default(kind, trigger)` | The default rule (D2) |
| `src/data/binding.rs:L286-L332` | unit test `blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers` | Pins D2 |
| `src/data/key_combination.rs:L70-L77` | `KeyCombination::is_modifier_only` | One definition of "made only of modifiers" |
| `src/data/key_combination.rs:L153-L176` | unit test `is_modifier_only_needs_every_key_to_be_a_modifier` | Pins D2 |
| `src/data/key_spec.rs:L143` | `KeySpec::modifier()` | What `is_modifier_only` relies on (unchanged) |
| `src/data/trigger.rs:L14`, `:L37-L39` | `Trigger` derives `Copy`; `is_immediate` | Used by `blocks_by_default` (unchanged) |
| `src/engine/matching.rs:L58-L63` | `is_trigger_key` now calls `combo.is_modifier_only()` | Refactor commit 1, no behaviour change claimed |
| `src/engine/mod.rs:L120-L131` | module doc rule 4 (rewritten) | Doc of the new suppression rule |
| `src/engine/mod.rs:L371-L553` | `Engine::on_key_down`: Immediate gate `:420-422`, Tap `:424-437` (gate `:434-436`), Hold `:438-453` (gate `:450-452`), DoubleTap `:454-469` (gate `:466-468`), repeat rule `:519-543`, record `:544-551` | The behavioural change (D3) |
| `src/engine/mod.rs:L555-L581` | `Engine::on_key_up`, key-up rule at `:576-579` | Unchanged; carries the suppression invariant |
| `src/engine/mod.rs:L631-L660` (approx.) | unit test `key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending`, comment `:633-639` | Comment updated only |
| `src/engine/keys.rs:L117-L218` | `reset_from_os` `:117`, `press` `:143`, `mark_blocked` `:158`, `mark_passed` `:178`, `is_blocked` `:188`, `is_passed` `:193`, `take_blocked` `:198`, `forget_blocked` `:212` | Unchanged record that m1/m2/m3 rely on |
| `src/engine/testing.rs:L22-L30` | module doc sentence on `expect_down_in_windows` | D8 |
| `src/engine/testing.rs:L475-L484` | `Scenario::expect_down_in_windows` (`#[track_caller]`, reads `self.env.os_down(physical_key(key))`) | Acceptance 2 proof |
| `src/engine/properties.rs:L1-L8`, `:L97-L107`, `:L128-L136` | module doc; `gestures` config (`Ctrl+Shift` Hold `block_default: true` at `:104`, new `F15` Tap `block_default: false` at `:106`); `mixed` config (`LShift` DoubleTap `block_default: true` at `:133`) | Property tests now cover pass and swallow paths |
| `src/engine/properties.rs:L248-L260` | `every_press_is_suppressed_or_passed_as_a_whole`, `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups`, `auto_repeats_fire_only_actions_that_repeat` | Invariant property tests |
| `src/engine/scenarios.rs:L899-L929` | `lshift_double_tap_fires_only_for_the_left_shift` (flipped to `expect_passed`) | Acceptance 1 (existing test) |
| `src/engine/scenarios.rs:L1340`, `:2052`, `:2079`, `:2110`, `:2201`, `:2744` | D9 scenarios with added `block_default: true` (configs at `:1347`, `:2058`, `:2085`, `:2118`, `:2209`, `:2751`) | Keep KS-06/KS-14/blocker regressions on the swallow path |
| `src/engine/scenarios.rs:L2917-L3400` | new section `// ---- block_default on deferred triggers (roadmap item 1.3) ----`; consts `LSHIFT_DOUBLE_TAP` `:2921`, `LSHIFT_DOUBLE_TAP_BLOCKING` `:2924`, `PASSING_TAP_F13` `:2926`, `PASSING_HOLD_F13` `:2928` | Red-first scenarios a..m3, I1 |
| `src/engine/scenarios.rs:L3402-L3418` | harness section; `down_in_windows_check_reports_a_key_windows_does_not_have_down` at `:3413-3414` | Harness self-test (deviation 1) |
| `src/hook/keyboard_hook_callback.rs:L238-L267` | adapter: `code < 0` pass `:238-243`, disabled pass `:246-251`, `decide_event` then `carry_out(decision)` then `LRESULT(1)` if suppress `:257-263`, else `CallNextHookEx` `:266` | Unchanged; `carry_out` runs before the event is passed |
| `src/config/default_config.ron:L144-L150` | flag comment for `block_default` | Template text |
| `src/config/default_config.ron:L273-L282` | LShift `DoubleTap(250)` example comment, marked working, with `<T>` caveat | Brief: "mark the LShift DoubleTap example as working"; L4 reads it |
| `README.md:L340-L351`, `:L385-L410`, `:L490`, `:L693` | trigger example + caveat; behaviour rule + three pitfall bullets; flags-table row; feature bullet | Docs |
| `CHANGELOG.md:L132-L151` (Changed, Breaking), `:L253-L254` (KS-07 entry edit), `:L362-L368` (Fixed) | Unreleased entries | Docs |
| `book/src/config/*.md` (10 pages) | hunks listed in Facts 40-41 | Config reference (commit 4) |

## Facts

Commits and gates (from lane logs; re-run by the tester)

1. `logs/commit2_red.log:589`: `test result: FAILED. 392 passed; 18 failed; 1 ignored`. Its failure list `logs/commit2_red.log:569-587` has 18 names: 3 `config::loader::tests::*` (L1, L3, L4) and 15 `engine::scenarios::*` (a, b, c, d, e, h, i, j, k, m1, m2, m3, flip, plus the two I1 known-issue scenarios).
2. `logs/commit3_lib.log:419` and `logs/head_lib.log:418`: `test result: ok. 411 passed; 0 failed; 1 ignored`. These are `cargo test --locked --lib` runs (impl.md:91-92); the full `cargo test --locked` (integration test `tests/ipc_roundtrip.rs` included) was not run by the developer (impl.md:95).
3. The single ignored test in the crate is `src/data/condition.rs:519` (`#[ignore = "manual: ... notepad.exe ..."]`); it predates the lane (impl.md:92 "pre-existing").
4. impl.md:89 records fmt, check and clippy `-D warnings` passing at `1562acd`, run after every commit. UNVERIFIED by the scout.
5. Commit 2 (`6950476`) is declared red; the cut allows exactly that (cut.md:187, :239). Its predicted red set is a, b, c, d, e, h, i, j, k, m1, m2, m3, flip, L1, L3, L4 (cut.md:224); impl.md:31-32 adds the two I1 scenarios to the red set (impl.md deviation 4, :75).
6. No line under `src/` added by the branch contains a non-ASCII byte (grep of `git diff master...HEAD -- src/` `+` lines). `src/config/default_config.ron:16-18` has pre-existing em dashes; `README.md`, `CHANGELOG.md` and `book/` added lines contain 26 non-ASCII lines (not checked against the ASCII rule's scope).

Schema and loader

7. `src/config/schema.rs:49-50`: `#[serde(default)] pub block_default: Option<bool>,`; `fn default_true` no longer exists in the file (diff removes it).
8. `src/config/loader.rs:385-387`: `let block_default = raw.block_default.unwrap_or_else(|| Binding::blocks_by_default(&kind, trigger));` is computed before `Binding::with_kind(kind, action)` at `:388`, and passed to `.with_block_default(block_default)` at `:389`.
9. `src/config/loader.rs:242-243`: the RON options enable `Extensions::IMPLICIT_SOME`.
10. `src/config/loader.rs:784`: the existing test asserts `user.block_default` for a `CapsLock` Immediate user binding (`// default`), unchanged by the branch.

Default rule

11. `src/data/binding.rs:130-135`: `blocks_by_default` returns `trigger.is_immediate() || !combo.is_modifier_only()` for `BindingKind::Combo`, and `true` for `Sequence { .. }` and `Chord { .. }`.
12. `src/data/key_combination.rs:75-77`: `is_modifier_only` is `self.keys.iter().all(|key| key.modifier().is_some())`; the doc says the parser never yields an empty combination (`:73-74`). On an empty `keys` vector `all` returns `true` (Rust semantics; UNVERIFIED whether any non-parser path builds an empty combination).
13. `src/engine/matching.rs:58-63`: `is_trigger_key` = any key with `key.accepts(pressed) && (key.modifier().is_none() || combo.is_modifier_only())`; master had the same predicate inline (`combo.keys.iter().all(|k| k.modifier().is_some())`).
14. `src/data/binding.rs:102` and `:115`: `with_kind` and `new_auto_blocker` set `block_default: true`; `src/data/binding.rs:147-148` `with_block_default` is unchanged.
15. `src/data/trigger.rs:14`: `Trigger` derives `Clone, Copy`; `is_immediate(self)` at `:37-39`.

Engine

16. `src/engine/mod.rs:434-436`, `:450-452`, `:466-468`: each of Tap, Hold, DoubleTap sets `decision.suppress = true` only `if binding.block_default`; on master each was an unconditional `decision.suppress = true;`.
17. `src/engine/mod.rs:428-433`, `:442-449`, `:458-465`: `arm_tap`, `arm_hold` (timer pushed) and `double_tap` (action pushed to `fire`) run before and regardless of the gate.
18. `src/engine/mod.rs:425-427`, `:439-441`, `:455-457`: deferred arms `continue` on a repeat, before the gate.
19. `src/engine/mod.rs:388-399`: the auto-blocker branch does not read `block_default`.
20. `src/engine/mod.rs:544-551`: a non-repeat key-down with `decision.suppress` is `mark_blocked`; otherwise `mark_passed`.
21. `src/engine/mod.rs:537-543`: a repeat follows the record: blocked -> suppressed, passed -> passed, no record and not suppressed -> `mark_passed`.
22. `src/engine/mod.rs:576-579`: `decision.suppress = was_blocked || (gesture_bound && !was_passed)`; `gesture_bound = tap_fired || hold != HoldRelease::NotPending`. The diff does not touch `:555-581`.
23. Suppression across several matching bindings in one key-down is an OR: any gate setting `decision.suppress = true` wins; nothing resets it to `false` inside the binding loop `src/engine/mod.rs:379-517`.
24. `src/engine/mod.rs:120-131`: rule 4 text now says every matching binding with `block_default` asks for suppression "whatever its trigger", and that the loader resolves a missing value to `false` only for a deferred modifier-only combo.

Adapter (unchanged)

25. `src/hook/keyboard_hook_callback.rs:257-263`: `carry_out(decision)` runs before `return LRESULT(1)` / the fall-through `CallNextHookEx` at `:266`, so a fired action (e.g. `ToggleCapsLock` on the second LShift down) is injected before that LShift key-down is passed on.

Harness

26. `src/engine/testing.rs:478-484`: `expect_down_in_windows` fails with `"{label}: {key} is not down in Windows"` where `label` is `self.last_step().label`.
27. `src/engine/scenarios.rs:3413`: `#[should_panic(expected = "down CapsLock at +0 ms: CapsLock is not down in Windows")]` on `down_in_windows_check_reports_a_key_windows_does_not_have_down`.
28. Harness methods used by the new scenarios: `from_ron_with_language_hotkey` `src/engine/testing.rs:173`, `check_press_symmetry` `:214`, `check_nothing_stuck` `:221`, `reload` `:237`, `foreground_changed` `:319`, `remapping_off` `:338`, `os_down` (FakeEnv) `:82`.

Acceptance proofs (brief acceptance lines)

29. Acceptance 1 ("double LShift DoubleTap(250) fires ToggleCapsLock, every LShift event passes"): `lshift_double_tap_fires_and_every_lshift_event_passes` `src/engine/scenarios.rs:3003-3024` (default config `LSHIFT_DOUBLE_TAP`, symmetry + stuck checks, all four LShift edges `expect_passed`, second down fires `[ToggleCapsLock]`, total `[ToggleCapsLock]`); flipped `lshift_double_tap_fires_only_for_the_left_shift` `:899-929` (also RShift never fires).
30. Acceptance 2 ("with LShift held, A passes together with Shift, nothing swallowed"): `lshift_held_with_its_double_tap_bound_types_a_capital_a` `src/engine/scenarios.rs:3027-3047` (down A passed, `expect_down_in_windows("LShift")` at `:3039`, all edges passed, total `[]`); repeats: `passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too` `:3098-3116`.
31. Acceptance 3 ("CapsLock Tap with default block_default swallows CapsLock"): `capslock_tap_with_the_default_block_default_still_swallows_capslock` `src/engine/scenarios.rs:3050-3073` (no `block_default` in config; down, up, repeat, up all `expect_suppressed`); loader L1 row 5 `CapsLock`+Tap -> `true` (`src/config/loader.rs:711`, expected vector `:723-727`).
32. Acceptance 4 ("explicit block_default: true on modifier-only DoubleTap swallows"): `explicit_block_default_true_on_a_modifier_only_double_tap_swallows` `src/engine/scenarios.rs:3076-3095` (`LSHIFT_DOUBLE_TAP_BLOCKING`, all LShift edges suppressed, fires `[ToggleCapsLock]`); loader L2 row 1 -> `true` (`src/config/loader.rs:733`, `:742`).
33. Per the cut, f, g, l, L2, s, the D9 scenarios and `engine::properties::*` were green on commit 2 as guards (cut.md:205-206, :211, :215, :217-218, :220; impl.md:36-42).

Other red-first tests (expected red line on `6950476` per impl.md:18-35 -> green on HEAD)

34. Scenarios a `:2931`, b `:2954`, c `:2981`, m1 `:3262`, m2 `:3278`: red "down F13 at +0 ms: expected the event to be passed, it was suppressed". d `:3003`, e `:3027`, h `:3098`, m3 `:3300`, flip `:900`, I1-a `:3341`: red "down LShift at +0 ms: ...". i `:3119`: red "down LShift at +10 ms: ...". j `:3148`, k `:3179`: red "down LShift at +20 ms: ...". I1-b `:3376`: red "down LCtrl at +0 ms: ...".
35. Loader tests: L1 `src/config/loader.rs:706` red "assertion `left == right` failed / left: [true, true, true, true, true, true, true, true, true, true]"; L3 `:746` red "block_default takes Some and None: config parse error (<in-memory>): 2:61: Expected boolean"; L4 `:691` red "assertion failed: !example.block_default" (impl.md:33-35).
36. Scenario l `src/engine/scenarios.rs:3238-3259`: `F13` Tap `block_default: false` + `F13` Hold (default `true`) -> down and up suppressed, Tap fires `[SwitchLanguage]` (OR rule, KS-13 comment `:3239-3241`).
37. Scenario k `src/engine/scenarios.rs:3179-3235`: two passing configs (`:3186-3192`, the second sets `block_original_combo: true` on the `Alt+Shift` Tap itself) pass LShift and fire `[SwitchLanguageBackward]`; the `block_default: true` config (`:3215-3218`) suppresses LShift down and up.
38. Scenario m3 `src/engine/scenarios.rs:3300-3338`: part 1 reloads to blocking while LShift is held (down passed, up passed, next down suppressed and fires); part 2 the reverse (down suppressed, up suppressed, next down passed and fires).
39. I1 scenarios `src/engine/scenarios.rs:3341-3373` (`<T>`: `,` typed between two passing LShift presses, DoubleTap still fires) and `:3376-3400` (LCtrl Tap -> `PressKey("Escape")` still fires after a quick Ctrl+C) pin current behaviour as a known issue; comments at `:3342-3346`, `:3377-3380`.

Docs

40. Hunks (new-side lines): README `+340`, `+344,8`, `+385,26`, `+490`, `+693`; CHANGELOG `+132,20`, `+253,2`, `+362,7`; default_config.ron `+144,7`, `+273,6`.
41. Book hunks (new-side): `README.md` 5, 41; `actions.md` 3, 248, 260; `errors.md` 3, 204; `example.md` 6, 19-20, 152; `kinds.md` 3, 169-170, 177, 182, 193, 197-198; `known-issues.md` 36, 114, 122, 133-136, 141; `matching.md` 3, 76, 117; `ron-syntax.md` 6, 83, 88-89, 91, 93; `structure.md` 8, 63, 149, 182; `triggers-conditions.md` 3, 44, 62, 64, 74, 78, 125-130, 132, 135. `book/src/config/keys.md` has no hunk.
42. impl.md:118-119: book RON validated with `cfgcheck` (`D:/tmp/ks-orch/p1-1.3-deferred-block-default/cfgcheck/ex/*.ron`), complete example "OK: 26 bindings", `mdbook build` into `D:/tmp/ks-orch/p1-1.3-deferred-block-default/book` with 0 broken internal links. UNVERIFIED by the scout.
43. Deferred to the Phase 1 book sync, not in this lane (cut.md:176, impl.md:120): `architecture.md`, `glossary.md`, `overview.md`, `build-and-run.md`, `roadmap.md` (1.3 status, `:199` expectation, `:594` "Shipped"). `book/src/roadmap.md:151` and `:158` still say the LShift example is marked not working until 1.3 (historical Phase 0 text).
44. `.claude/agents/code-reviewer.md:126` (R8) names `KeyState::active_keys`; `grep active_keys src` finds no match at HEAD (cut follow-up F6).

## Exact commands and expected results

Run from `D:/key-switch-rs/key-switch-rs`. None needs a hook, injection or the registry.

| Purpose | Command | Expected |
|---|---|---|
| Gate 1 | `cargo fmt --check` | exit 0 |
| Gate 2 | `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| Gate 3 | `cargo test --locked` | lib: `411 passed; 0 failed; 1 ignored` (per Fact 2); `tests/ipc_roundtrip.rs` and doc-tests: not recorded in lane logs |
| Acceptance 1 | `cargo test --locked --lib engine::scenarios::lshift_double_tap_fires` | 2 passed (`..._and_every_lshift_event_passes`, `..._only_for_the_left_shift`) |
| Acceptance 2 | `cargo test --locked --lib engine::scenarios::lshift_held_with_its_double_tap_bound_types_a_capital_a` | 1 passed |
| Acceptance 3 | `cargo test --locked --lib engine::scenarios::capslock_tap_with_the_default_block_default_still_swallows_capslock` | 1 passed |
| Acceptance 4 | `cargo test --locked --lib engine::scenarios::explicit_block_default_true_on_a_modifier_only_double_tap_swallows` | 1 passed |
| New section + loader + unit tests | `cargo test --locked --lib block_default` | matches a, b, c, f, g, L1, L2, L3 and the `blocks_by_default_*` unit test; all pass (count not recorded in logs) |
| Properties | `cargo test --locked --lib engine::properties` | 3 passed |
| Default rule units | `cargo test --locked --lib is_modifier_only` ; `cargo test --locked --lib blocks_by_default` | 1 passed each |
| Template | `cargo test --locked --lib config::loader::tests::the_template_lshift_double_tap_example_lets_the_left_shift_through` ; `... default_config_commented_examples_load` | 1 passed each |
| Harness self-test | `cargo test --locked --lib down_in_windows_check` | 1 passed (should_panic) |
| Red replay | not possible in the shared tree without switching commits; recorded output is `logs/commit2_red.log:569-589` | 18 failed / 392 passed / 1 ignored |
| Book | `mdbook build book -d D:/tmp/ks-orch/p1-1.3-deferred-block-default/book` | builds (impl.md:119) |

Manual checks (human only, not passes): M1 (Checkpoint 1 smoke #6, `book/src/roadmap.md:310`), M2 (pitfall row 65), M3 (row 66); setups and expected results at impl.md:100-104. No `#[ignore]` test is added (impl.md:96).

## Open points (from impl.md:129-136 and :53-65)

1. I1 (critique Important, resolved by documentation, option b): no interrupt rule (cut D6, cut.md:100-108). Owner question open: interrupt rule as 2.2 acceptance or a new item. Pinned by `src/engine/scenarios.rs:3341` and `:3376`; caveat in `src/config/default_config.ron:273-278`, CHANGELOG `:140-143`, README `:394-399`, book `triggers-conditions.md` §11.8, `known-issues.md` row 67, `kinds.md` §10.
2. M1 (Minor, resolved by documentation): an implicit `block_default` (false) on a modifier-only deferred binding wins over that binding's own `block_original_combo: true` because the dedup drops the blocker (`src/config/loader.rs:296-305`); pinned by k's second config `src/engine/scenarios.rs:3190-3192`. The rejected alternative is one condition at `src/config/loader.rs:385-387` (cut.md:373).
3. M2 (Minor, resolved): KS-13 OR-rule text extended to deferred bindings in five book pages; pinned by l `src/engine/scenarios.rs:3238`.
4. Critic open question 1: merge order against item 1.2 (mask key for lone Alt/Win, KS-12); phase-plan scope; interim behaviour documented (CHANGELOG `:148-149`, README `:406-410`, known-issues row 66).
5. Cut follow-ups: F2 `block_default` accepted and ignored on `sequence`/`chord` (Fact 11; `book/src/config/errors.md:204`); F3 KS-19 (injected CapsLock while Shift is down, Fact 25, item 3.1); F4 KS-12 (1.2); F5 KS-13 (1.6); F6 stale R8 text (Fact 44).
6. Cut Q2 (loader warning when a non-blocking binding replaces the auto-blocker): no code in 1.3 (cut.md:334). Cut Q3 (research on CapsLock toggle with Shift held): run only if M1 misbehaves (cut.md:335).

## Invariants in play

- R1 hook callback: callback unchanged (`src/hook/keyboard_hook_callback.rs:238-267`); engine change adds one bool read per matching deferred binding (`src/engine/mod.rs:434`, `:450`, `:466`), no allocation added in the gates.
- R2 sentinel: no new `SendInput` path (impl.md:85); interplay at Fact 25.
- R3/R4 shared state and atomics: no new static, lock or atomic (impl.md:85); engine state under `Mutex<Engine>` unchanged.
- R5 builders: `blocks_by_default` is an associated fn (`src/data/binding.rs:130`); `with_block_default` unchanged (`:147`).
- R6 Display: no new variant; `Display for Binding` does not print `block_default` (cut.md:52).
- R7 specificity: sort/publish untouched; OR suppression (Fact 23), scenario l.
- R8 key identity: `is_modifier_only` uses `KeySpec::modifier()` (`src/data/key_spec.rs:143`); unsided `"Shift"` DoubleTap keyed by event code (cut.md:293, `gestures.rs:207` per cut, not re-read).
- Suppression invariant (CLAUDE.md principle 2): key-up rule `src/engine/mod.rs:576-579`, repeat rule `:537-543`, record `:544-551`; checked by `check_press_symmetry` and `engine::properties`.
- KS-06 (repeats), KS-07 (smoke #6), KS-12 (lone Alt/Win), KS-13 (OR rule), KS-14 (D9 scenarios), KS-18 (blocker dedup, `src/config/loader.rs:271-312`), KS-19 (injected toggle with Shift down).

## Not covered

- Did not run any cargo command; gate and test outcomes are from lane logs and impl.md.
- Did not read book page contents beyond hunk positions, nor validate their RON or links; did not read `critique.md` beyond its headings (`:1-12`, `:23-138`).
- Did not read `src/engine/gestures.rs`, `chords.rs`, `sequences.rs`, the D9 scenario bodies, or `properties.rs` generator code.
- Did not read commit messages of `5adf19c`..`1562acd` (impl.md:43 says commit 2's body lists the red set).
- Did not check `tests/ipc_roundtrip.rs` or doc-test counts.
