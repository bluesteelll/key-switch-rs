# Context pack r1: p0-A2-matching (trigger-key rule KS-04, sided keys KS-07, exact modifiers + `allow_extra_modifiers`, KS-06 repeat rule)

Audience: tester and reviewer, round 1. Facts only; `UNVERIFIED` marks inference. Line numbers are on `fix/matching` `c609a69` unless prefixed `master:`.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/matching`, HEAD `c609a69`, tree clean at read time.
- `master` = `e67e5cb` = merge-base of `master` and `fix/matching`.
- `git log --oneline master..fix/matching` (newest first):
  - `c609a69` Docs: reflow the sided-names CHANGELOG entry
  - `20672b7` Engine: an auto-repeat follows its key-down's suppression (KS-06) — C9 fix
  - `faf3be1` Engine: red-first: a key-down that reaches the OS clears its key-up block (KS-06) — C8 red
  - `f4b2582` Engine: red-first: an auto-repeat follows its key-down (KS-06) — C7 red
  - `cffb5eb` Matching: exact modifiers with per-binding allow_extra_modifiers — C6 fix
  - `7b65609` Engine: red-first: exact modifier matching — C5 red
  - `2271db2` Matching: trigger-key rule for combos and deferred triggers (KS-04) — C4 fix
  - `46424a1` Engine: red-first: a combo fires only on its own key (KS-04) — C3 red
  - `709d739` Matching: sided and unsided key specs in every trigger (KS-07) — C2 fix
  - `1bc3b0e` Engine: red-first: sided keys in keys, sequences and chords (KS-07) — C1 red
- Red commits touch tests only: `1bc3b0e` (loader.rs +22, scenarios.rs +215), `46424a1` (scenarios.rs +111), `7b65609` (loader.rs +21, scenarios.rs +127), `f4b2582` (scenarios.rs +136), `faf3be1` (scenarios.rs +26).
- `git diff --stat master..fix/matching`: 20 files, +2168 / -315:

| File | +/- |
|---|---|
| CHANGELOG.md | +50 |
| README.md | 86 |
| src/config/default_config.ron | 25 |
| src/config/loader.rs | 170 |
| src/config/parsing.rs | 91 |
| src/config/schema.rs | +7 |
| src/data/binding.rs | 70 |
| src/data/key_combination.rs | 123 |
| src/data/key_spec.rs | +353 (new) |
| src/data/mod.rs | +1 |
| src/data/vk_name.rs | 23 |
| src/engine/chords.rs | 180 |
| src/engine/keys.rs | 166 |
| src/engine/matching.rs | +205 (new) |
| src/engine/mod.rs | 66 |
| src/engine/scenarios.rs | 618 |
| src/engine/sequences.rs | 160 |
| src/hook/keyboard_hook.rs | 8 (tests only) |
| src/hook/keyboard_hook_callback.rs | 26 (tests only) |
| src/system/registry.rs | 55 |

- Lane inputs read: `D:/tmp/ks-orch/p0-A2-matching/cut.md` (plan C1-C6 + developer addendum C7-C9), `critique.md` (CHANGES_REQUESTED: I1, I2, M1-M4), `impl.md` (round-1 report).

## File map

| Range | What lives there | Why it matters this round |
|---|---|---|
| `src/data/key_spec.rs:21-89` | `Modifier` {Shift, Ctrl, Alt, Win}: `ALL`, `left`, `right`, `generic` (None for Win), `of`, `name`, `bit` | family/side tables used by every match (KS-07, R8) |
| `src/data/key_spec.rs:91-108` | `ModifierSet(u8)`: `EMPTY`, `with`, `contains` | exact-modifier comparison unit |
| `src/data/key_spec.rs:116-171` | `KeySpec` {`AnySide(Modifier)`, `Key(VIRTUAL_KEY)`}: `accepts`, `modifier`, `overlaps`, `parse` | trigger-key type for keys, steps, chords |
| `src/data/key_spec.rs:173-194` | `From<VIRTUAL_KEY>` (generic codes -> `AnySide`), `Display` (`Key(VK_LWIN)` -> `"LWin"`) | R6 display; generic-code mapping |
| `src/data/key_spec.rs:196-353` | 11 unit tests | |
| `src/data/key_combination.rs:11-14` | `KeyCombination { keys: Vec<KeySpec> }` | type change; `matches` removed |
| `src/data/key_combination.rs:16-31` | set-equality `PartialEq` | auto-blocker dedup in loader |
| `src/data/key_combination.rs:34-69` | `new`/`with`/`from_keys` generic over `Into<KeySpec>`; `modifiers()` `:63-69` | R5 builders; exact rule input |
| `src/data/key_combination.rs:71-82` | `Display` joins with `+` | INFO line, Binding display |
| `src/data/binding.rs:28-31` | `BindingKind::Chord { keys: Vec<KeySpec>, window }` | chord type change |
| `src/data/binding.rs:40-46` | `key_count()` (Combo: `keys.len()`) | R7 specificity sort (unchanged) |
| `src/data/binding.rs:66-70, 89, 102, 140-143` | field `allow_extra_modifiers`; `false` in `with_kind`/`new_auto_blocker`; builder `with_allow_extra_modifiers` | owner decision; R5 |
| `src/data/binding.rs:150-184, 186-203` | `Display` suffix `"  [allow_extra_modifiers]"` `:164-168`; `format_kind` via `KeySpec::to_string` | R6 |
| `src/data/vk_name.rs:92-97, 175-183` | `parse_vk` doc (Win and LWin both `VK_LWIN`); `normalize_key_name` extracted | parse path |
| `src/config/schema.rs:56-62` | `RawBinding.allow_extra_modifiers: bool`, `#[serde(default)]` | new RON field |
| `src/config/parsing.rs:15-17` | `parse_combo` -> `KeySpec` via `parse_segments` | trigger parsing |
| `src/config/parsing.rs:23-32` | `parse_key_list` -> `Vec<VIRTUAL_KEY>` (output keys) | D7: output codes unchanged |
| `src/config/parsing.rs:35-51` | `parse_segments` (shared splitter, error texts) | |
| `src/config/loader.rs:170-189` | `expand_with_auto_blockers` (dedup by `KeyCombination ==`) | auto-blocker exactness (KS-05/18 context) |
| `src/config/loader.rs:207-247` | `raw_binding_to_binding`; reject `allow_extra_modifiers` on non-Combo `:229-237`; builder chain `:239-244` | owner decision, load error |
| `src/config/loader.rs:299-321` | `chord_spec_to_kind`: `KeySpec::parse`, overlap -> `"Chord key #i: duplicate ... (overlaps key #j)"` `:309-314` | breaking load error |
| `src/config/loader.rs:433-447` | `raw_step_to_step`; macro `Combo` via `parse_key_list` `:445` | output keys |
| `src/config/loader.rs:1253-1359` | new loader tests (see Tests) incl. `readme_matching_examples_load` `:1307-1336` (`include_str!("../../README.md")`) | doc examples under test |
| `src/engine/matching.rs:1-14` | module doc: trigger rule + exact rule | |
| `src/engine/matching.rs:22-34` | `trait HeldKeys { is_down; modifiers() default }` | |
| `src/engine/matching.rs:37-39` | `keys_down` (every key held on its side) | |
| `src/engine/matching.rs:46-51` | `is_trigger_key` | KS-04 rule |
| `src/engine/matching.rs:58-67` | `combo_matches` = trigger && keys_down && (allow \|\| families equal) | exact rule |
| `src/engine/matching.rs:69-205` | 8 unit tests on a real `KeyState` (`held()` `:79-85`) | critique M2 |
| `src/engine/keys.rs:20-32` | `MODIFIERS` (11 codes incl. generic 0x10-0x12) | resync writes generic slots |
| `src/engine/keys.rs:35-38` | `slot(vk)` bound `< 256` | R1 bounded indexing |
| `src/engine/keys.rs:63-71` | `sync_modifiers` (skips event key) | KS-14 interplay |
| `src/engine/keys.rs:88-110` | `mark_blocked`, `is_blocked` (new `:95-97`), `take_blocked`, `clear_all_blocked` | KS-06 rule |
| `src/engine/keys.rs:112-114` | `is_active` (no longer `#[cfg(test)]`) | backs `HeldKeys` |
| `src/engine/keys.rs:117-129` | `impl HeldKeys for KeyState` | sided/any-side lookup |
| `src/engine/mod.rs:89-133` | module doc "Decision rules" 1-5 (rule 3 KeySpec, rule 4 Combo, rule 5 repeat) and defect list | docs vs code |
| `src/engine/mod.rs:303-445` | `on_key_down` | core of the round |
| `src/engine/mod.rs:447-471` | `on_key_up` (unchanged logic) | suppression symmetry |
| `src/engine/chords.rs:1-19` | module doc (KeySpec, overlap) | |
| `src/engine/chords.rs:28-35` | `PendingChord { chord_keys: Vec<KeySpec>, seen: Vec<VIRTUAL_KEY>, .. }` | |
| `src/engine/chords.rs:62-114` | `Chords::on_key_down` | chord path (KS-07) |
| `src/engine/chords.rs:118-120` | `same_keys` set-equality | |
| `src/engine/sequences.rs:1-16` | module doc (sided steps, last-step non-modifier key-ups) | |
| `src/engine/sequences.rs:58-117` | `Sequences::on_key_down<H: HeldKeys>` | |
| `src/engine/sequences.rs:125-140` | `last_step_keys` (non-modifier `Key` specs only) | KS-32 partly |
| `src/engine/scenarios.rs:750-963` | section "Sided keys (KS-07)" (8 tests) | acceptance |
| `src/engine/scenarios.rs:965-1074` | section "Trigger key (KS-04)" (5 tests) | acceptance |
| `src/engine/scenarios.rs:1076-1199` | section "Exact modifiers" (7 tests) | acceptance |
| `src/engine/scenarios.rs:1201-1361` | section "Auto-repeat follows its key-down (KS-06)" (6 tests) | critique I1/I2 |
| `src/engine/scenarios.rs:284-288` | comment change in `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` | deviation 8 |
| `src/engine/testing.rs:68-75, 87-98, 314-328` | `FakeEnv::os_key_down` (generic = either side); `physical_key` (Shift->LShift, Ctrl->LCtrl, Alt->LAlt, Win->`VK_LWIN`); `key_event` updates OS state only when passed | harness semantics (not in diff) |
| `src/hook/keyboard_hook_callback.rs:271-277, 290-293, 318-322, 341-349` | D10: hook tests set `with_allow_extra_modifiers(true)` | live `GetAsyncKeyState` flake guard |
| `src/hook/keyboard_hook.rs:227-228, 295, 313-317` | test assertions now on `KeySpec` | type only |
| `src/system/registry.rs:56` | INFO line prints `{combo}` | D8 |
| `src/system/registry.rs:72-73, 197, 203-206` | `win_with(key)` = `AnySide(Win)` + key | D9 |
| `src/system/registry.rs:277-292` | test `language_hotkeys_are_either_side_modifiers` | |
| `README.md:175-230` | "Combo syntax": modifier row `:182`, "Sided modifiers" `:190-197`, "Which key fires a binding" `:199-207`, "Exact modifiers" `:209-217`, RON block `:219-226`, AltGr note `:228-230` | spec doc item; under test via `readme_matching_examples_load` |
| `README.md:41, 47-48, 71-73, 258, 289-292, 315-319, 329-334, 355-359, 420, 578, 586` | layout tree, kinds table "Fires when", chord overlap rule, LShift warning, triggers key-down bullet, repeat rule, flags row, features | doc drift checks |
| `src/config/default_config.ron:32-35, 49-60, 126-129, 219-221` | modifier list, trigger-key + exact paragraphs, flag entry, LShift example warning | spec doc item |
| `src/config/default_config.ron:145-155` | live bindings: `CapsLock` (SwitchLanguage, `block_original_combo: true`), `Shift+CapsLock` (ToggleCapsLock) | acceptance "Shift+CapsLock fires ToggleCapsLock" |
| `CHANGELOG.md:40-42, 55-75, 80-101, 112-114` | Added (flag); Changed (Breaking sided names, INFO line, Breaking exact modifiers); Fixed (KS-04, KS-06, KS-07, KS-32 partly) | spec doc item |

## Facts

### Types and parsing
1. `KeySpec::accepts`: `AnySide(m)` accepts `m.left()`, `m.right()`, and `m.generic()` when `Some`; `Key(k)` accepts only `k` (`src/data/key_spec.rs:134-139`).
2. `Modifier::Win.generic()` is `None` (`src/data/key_spec.rs:63`).
3. `KeySpec::parse`: normalised `"win"`/`"super"` -> `AnySide(Win)`; every other name -> `parse_vk(name).map(KeySpec::from)` (`src/data/key_spec.rs:165-170`).
4. `From<VIRTUAL_KEY>`: `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` -> `AnySide`; every other code, `VK_LWIN` included, -> `Key(code)` (`src/data/key_spec.rs:173-181`).
5. `KeySpec::Display` renders `Key(VK_LWIN)` as `"LWin"` and `AnySide(m)` as `m.name()`; other keys via `vk_name` (`src/data/key_spec.rs:184-193`).
6. `KeySpec::overlaps`: `AnySide(a)`/`AnySide(b)` iff `a == b`; mixed iff `AnySide(m).accepts(vk)`; `Key`/`Key` iff equal (`src/data/key_spec.rs:152-160`).
7. `Modifier::of(vk)` scans `ALL` comparing left, right, generic (`src/data/key_spec.rs:69-73`); `KeySpec::modifier` returns `Some(m)` for `AnySide(m)` and `Modifier::of(vk)` for `Key(vk)` (`:143-148`).
8. `ModifierSet` is a `u8` bitset, bit = `1 << discriminant` (`src/data/key_spec.rs:86-88, 95-108`).
9. `parse_combo` returns `KeyCombination::from_keys` of `KeySpec::parse` results (`src/config/parsing.rs:15-17`); `parse_key_list` returns deduped `parse_vk` codes (`:23-32`); both use `parse_segments` (`:35-51`).
10. Macro `Combo(...)` steps use `parse_key_list` (`src/config/loader.rs:445`), so output codes stay `VIRTUAL_KEY` (`"Win"` sends `VK_LWIN`, per `src/data/vk_name.rs:92-95` doc).
11. `KeyCombination::from_keys` dedups on `KeySpec` equality keeping order (`src/data/key_combination.rs:50-59`); `PartialEq` is set-equality (`:16-29`).
12. `KeyCombination::modifiers()` folds `key.modifier()` of every key into a `ModifierSet` (`src/data/key_combination.rs:63-69`); it is computed per call, no cached field (`:11-14`).
13. `KeyCombination::matches` no longer exists (grep: no `fn matches` in `src/data/key_combination.rs`).

### Loader
14. `RawBinding.allow_extra_modifiers: bool` with `#[serde(default)]` (`src/config/schema.rs:56-62`).
15. `raw_binding_to_binding` rejects `allow_extra_modifiers: true` on non-Combo kinds with `"{prefix}: \`allow_extra_modifiers\` applies only to \`keys\` bindings"`, after the deferred-trigger check (`src/config/loader.rs:220-237`).
16. The flag is passed through `.with_allow_extra_modifiers(raw.allow_extra_modifiers)` (`src/config/loader.rs:244`).
17. `Binding::new_auto_blocker` sets `allow_extra_modifiers: false` (`src/data/binding.rs:102`); no other code path sets it true for auto-blockers (grep `with_allow_extra_modifiers` in `src/config`: only `loader.rs:244` and tests).
18. Chord keys: each parsed with `KeySpec::parse`; any earlier key that `overlaps` gives `Chord key #{i}: duplicate {k:?} (overlaps key #{j})` (`src/config/loader.rs:303-317`).
19. `expand_with_auto_blockers` dedups blockers by `KeyCombination ==` (set-equality of `KeySpec`) (`src/config/loader.rs:178-183`).
20. `readme_matching_examples_load` reads `README.md` via `include_str!`, takes the text from `"### Combo syntax"` to the next `"\n### "`, wraps each ```` ```ron ```` block in `( bindings: [ ... ] )` and asserts it loads non-empty, and that at least one block exists (`src/config/loader.rs:1307-1336`). The section has one RON block with three bindings (`README.md:219-226`).

### Matching (engine)
21. `HeldKeys::modifiers()` default: families whose `is_down(AnySide(m))` holds (`src/engine/matching.rs:28-33`).
22. `keys_down`: `combo.keys.iter().all(|&key| held.is_down(key))` (`src/engine/matching.rs:37-39`).
23. `is_trigger_key`: some key `accepts(pressed)` and (`key.modifier().is_none()` or every key of the combo is a modifier) (`src/engine/matching.rs:46-51`).
24. `combo_matches` = `is_trigger_key && keys_down && (allow_extra_modifiers || held.modifiers() == combo.modifiers())` (`src/engine/matching.rs:58-67`).
25. `impl HeldKeys for KeyState`: `Key(vk)` -> `is_active(vk)`; `AnySide(m)` -> left || right || generic slot (`src/engine/keys.rs:117-129`).
26. `sync_modifiers` writes all 11 `MODIFIERS` slots (generic and sided) from `Env::os_key_down`, skipping the event key (`src/engine/keys.rs:20-32, 63-71`); it runs at the start of every `on_key` (`src/engine/mod.rs:270`).
27. Every per-key table access goes through `slot()` which returns `None` for `vk >= 256` (`src/engine/keys.rs:35-38`).
28. `ActiveKeys`, `ACTIVE_KEYS_CAP`, `normalize`, `active_keys` are gone from `src/engine/keys.rs` (grep: none present); `on_key_down` has no `let active` (`src/engine/mod.rs:303-310`).

### Decision path (`on_key_down`, `src/engine/mod.rs:303-445`)
29. Order per event: `now`, `was_repeat = self.keys.press(vk)`, then each binding: `condition.evaluate` first (`:304-316`).
30. Combo branch calls `matching::combo_matches(combo, vk, &self.keys, binding.allow_extra_modifiers)` and `continue`s on false (`:319-327`); the condition check precedes it (`:314`).
31. Immediate: fires only for the first matching Immediate binding (`immediate_fired`), on every matching key-down including repeats; sets `suppress` if `block_default` (`:329-337`).
32. Tap/Hold/DoubleTap: `continue` on repeat; arm under `vk` (the event key) and set `suppress` (`:338-377`).
33. Sequence: `continue` on repeat; `self.sequences.on_key_down(&self.keys, ...)`; `Completed` marks `last_step_keys` blocked (`:380-404`).
34. Chord: `continue` on repeat; `self.chords.on_key_down(vk, ..., keys, ...)`; `Completed { keys }` marks each code blocked (`:405-423`).
35. Post-loop repeat rule (C9): repeat -> `decision.suppress = self.keys.is_blocked(vk.0)` (overrides any binding's request); first key-down suppressed -> `mark_blocked`; first key-down passed -> `take_blocked` (`:427-443`).
36. `on_key_up`: `release(vk)`; Tap resolve / Hold release; if Tap fired or Hold pending -> `take_blocked` and suppress; else `suppress = take_blocked(vk)` (`src/engine/mod.rs:447-471`). Not in this lane's diff (UNVERIFIED by diff hunk list: mod.rs hunks are docs and `on_key_down`).
37. `on_foreground_changed` resyncs modifiers and `clear_all_blocked` (`src/engine/mod.rs:298-301`).
38. Module doc rule 5 states the repeat rule; defect list names KS-10, KS-13, KS-14, KS-15, KS-16 and no longer KS-04/KS-06/KS-07 (`src/engine/mod.rs:117-133`).
39. `#![forbid(unsafe_code)]` remains (`src/engine/mod.rs:135`); `mod matching;` added (`:141`).

### Chords and sequences
40. `Chords::on_key_down` returns `NotInChord` unless some chord key `accepts(vk)` (`src/engine/chords.rs:70-72`).
41. Pending entry lookup is `same_keys` (equal length + containment) (`src/engine/chords.rs:78-83, 118-120`); `seen` stores sided codes without duplicates (`:85-88`).
42. Completion: every chord key accepts some `seen` code; `Completed.keys` = `seen` codes (`src/engine/chords.rs:90-100`).
43. A fresh entry allocates `chord_keys.to_vec()`, `vec![vk]`, `action.clone()` (`src/engine/chords.rs:105-111`); `Completed` collects a `Vec<u16>` (`:97`).
44. `Sequences::on_key_down` matches a step with `keys_down(step, held)` (subset; no trigger-key or exact rule) (`src/engine/sequences.rs:76, 96`).
45. `last_step_keys` returns only non-modifier `KeySpec::Key` codes of the last step (`src/engine/sequences.rs:125-140`).

### Adapter, registry, display
46. No production change in `src/hook`: diff hunks are inside `mod tests` only (`src/hook/keyboard_hook_callback.rs:271-349`, `src/hook/keyboard_hook.rs:227-317`).
47. The three hook-adapter tests set `.with_allow_extra_modifiers(true)` with a comment on live `GetAsyncKeyState` (`src/hook/keyboard_hook_callback.rs:273-277, 293, 322, 349`).
48. Registry Win defaults and `parse_win_key_combo` build `win_with(key)` = `KeyCombination::new(KeySpec::AnySide(Modifier::Win)).with(key)` (`src/system/registry.rs:72-73, 197, 203-206`); registry access is read-only in this diff.
49. INFO line: `println!("[INFO] {:?}: registry combination {}", function, combo)` (`src/system/registry.rs:56`).
50. `Binding` Display appends `"  [allow_extra_modifiers]"` when set, for both normal and auto-blocker lines (`src/data/binding.rs:164-182`).
51. `format_kind` renders Combo keys joined by `" + "`, sequence steps via `KeyCombination::to_string` joined by `" → "`, chord keys joined by `"+"` (`src/data/binding.rs:186-203`).

### Harness semantics (unchanged file `src/engine/testing.rs`)
52. `physical_key` maps `"Shift"`->`VK_LSHIFT`, `"Ctrl"`->`VK_LCONTROL`, `"Alt"`->`VK_LMENU`; `"Win"` parses to `VK_LWIN` (`src/engine/testing.rs:87-98`).
53. `FakeEnv::os_key_down(VK_SHIFT/VK_CONTROL/VK_MENU)` = either sided slot (`src/engine/testing.rs:68-75`).
54. The fake OS state changes only for events the engine passed (`src/engine/testing.rs:319-323`); `.repeat(key)` requires the key held and sends a Down (`:163-169`).

### Counts
55. `src/` has 245 `#[test]` attributes and 0 `#[ignore` (grep at `c609a69`); no `tests/` directory exists.
56. impl.md reports no manual `#[ignore]` tests added (`D:/tmp/ks-orch/p0-A2-matching/impl.md:117`).

## Tests: red-first and acceptance proofs

### Commands (from cut.md:259-263 and impl.md:111-116)
- Gate (every commit): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; full suite (tester): `cargo test --locked`.
- Developer per-fix run: `cargo test --locked --lib -- engine:: config:: data:: system:: hook::`; reported final result at `c609a69`: **238 passed, 0 failed, 16 filtered out** (impl.md:115-116).
- Red-first runs: each red test by name on its parent commit and on its fix (impl.md:115). impl.md does not record the exact command line; the name-filter form is `cargo test --locked --lib <test_name>` run on a checkout of the named commit (UNVERIFIED as the developer's literal command).
- `cargo doc --no-deps`: only the pre-existing warning `src/data/vk_name.rs:4` "unclosed HTML tag `raw`" (impl.md:114).
- Panic text format: `scenario: <label>: ...`; labels `"{down|repeat|up} {key} at +N ms"` (`src/engine/testing.rs:325`; cut.md:262-263).

### Red-first table (red text = expected failure on the parent commit; green on the fix commit and at HEAD)

| # | Test (location at HEAD) | Red on | Expected red text | Green from |
|---|---|---|---|---|
| T1 | `engine::scenarios::lshift_in_keys_fires_only_for_the_left_shift` (`scenarios.rs:753`) | `1bc3b0e` | `down F13 at +0 ms: expected the event to be suppressed, it was passed` | `709d739` |
| T2 | `rwin_and_lwin_in_keys_fire_only_for_their_side` (`:776`) | `1bc3b0e` | `down F13 at +0 ms: expected the event to be suppressed, it was passed` | `709d739` |
| T4 | `lshift_double_tap_fires_only_for_the_left_shift` (`:842`) | `1bc3b0e` | `down LShift at +0 ms: expected the event to be suppressed, it was passed` | `709d739` |
| T5 | `unsided_shift_in_a_chord_fires_for_either_side` (`:872`) | `1bc3b0e` | `down LShift at +0 ms: expected the event to be suppressed, it was passed` | `709d739` |
| T7 | `sided_shift_in_a_sequence_step_matches_only_its_side` (`:913`) | `1bc3b0e` | `down F14 at +0 ms: expected the event to be suppressed, it was passed` | `709d739` |
| T8 | `win_in_the_last_sequence_step_does_not_swallow_its_key_up` (`:945`) | `1bc3b0e` | `up Win at +0 ms: expected the event to be passed, it was suppressed` | `709d739` |
| T9 | `config::loader::tests::chord_keys_that_overlap_are_rejected` (`loader.rs:1267`) | `1bc3b0e` | panic `overlapping chord keys must be rejected: [Binding { kind: Chord { keys: [VIRTUAL_KEY(16), VIRTUAL_KEY(160)] ...` | `709d739` |
| T10 | `key_pressed_while_an_immediate_binding_key_is_held_passes` (`:968`) | `46424a1` | `down A at +30 ms: expected the event to be passed, it was suppressed` | `2271db2` |
| T11 | `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` (`:986`) | `46424a1` | `down A at +50 ms: expected the event to be passed, it was suppressed` | `2271db2` |
| T12 | `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock` (`:1011`) | `46424a1` | `down LShift at +0 ms: expected the event to be passed, it was suppressed` | `2271db2` |
| T13 | `modifier_pressed_last_does_not_fire_a_combo` (`:1031`) | `46424a1` | `down LCtrl at +0 ms: expected the event to be passed, it was suppressed` | `2271db2` |
| T14 | `deferred_combo_arms_only_on_its_trigger_key` (`:1046`) | `46424a1` | `down LShift at +350 ms: expected the event to be passed, it was suppressed` | `2271db2` |
| T15 | `capslock_binding_does_not_fire_while_shift_is_held` (`:1079`) | `7b65609` | `down CapsLock at +0 ms: expected the event to be passed, it was suppressed` | `cffb5eb` |
| T17 | `ctrl_j_does_not_fire_on_ctrl_shift_j` (`:1108`) | `7b65609` | `down J at +0 ms: expected the event to be passed, it was suppressed` | `cffb5eb` |
| T20 | `auto_blocker_matches_its_modifiers_exactly` (`:1148`) | `7b65609` | second `down LShift at +0 ms: expected the event to be passed, it was suppressed` | `cffb5eb` |
| T21 | `config::loader::tests::allow_extra_modifiers_is_only_for_keys_bindings` (`loader.rs:1339`) | `7b65609` | panic `allow_extra_modifiers on a sequence/chord must be rejected: [Binding { kind: Sequence { ...` | `cffb5eb` |
| C7a | `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (`:1204`) | `f4b2582` | `repeat CapsLock at +533 ms: expected the event to be passed, it was suppressed` | `20672b7` |
| C7b | `double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too` (`:1231`) | `f4b2582` | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` | `20672b7` |
| C7c | `chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too` (`:1252`) | `f4b2582` | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` | `20672b7` |
| C7d | `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` (`:1283`) | `f4b2582` | `repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed` | `20672b7` |
| C7e | `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once` (`:1308`) | `f4b2582` | `repeat L at +500 ms: expected the event to be suppressed, it was passed` | `20672b7` |
| C8 | `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (`:1338`) | `faf3be1` | `up LShift at +533 ms: expected the event to be passed, it was suppressed` (with the repeat rule alone: `repeat LShift at +533 ms: expected the event to be passed, it was suppressed`) | `20672b7` |

Source: impl.md:21-61; predicted texts cut.md:265-370; C7/C8 addendum cut.md:579-587.

Guards (green at every commit per impl.md:29, :48 and cut.md:382-388): T3 `unsided_shift_and_win_in_keys_fire_for_either_side` (`:811`), T6 `sided_shift_in_a_chord_is_only_its_side` (`:901`), T16 `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held` (`:1096`), T18 `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j` (`:1124`), T19 `either_shift_counts_as_the_same_modifier` (`:1136`), T22 `shift_capslock_fires_toggle_capslock_with_either_shift` (`:1173`). Existing scenarios cut.md names as hand-checked: `right_hand_modifiers_match_unsided_combo_keys` (`:60`), `more_specific_immediate_binding_wins` (`:100`), `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` (`:136`), `modifier_lost_by_the_os_is_resynced_before_matching` (`:157`), `sequence_with_a_modifier_held_through_both_steps` (`:653`).

### Brief acceptance line -> proof (cut.md:395-410, test names verified present at HEAD)

| Acceptance line | Scenario / test |
|---|---|
| with CapsLock held, `a` passes and nothing fires | T10 (`:968`), T11 (`:986`) |
| CapsLock binding does not fire while Shift is held; does with `allow_extra_modifiers: true` | T15 (`:1079`), T16 (`:1096`), `config::loader::tests::allow_extra_modifiers_true_is_parsed` (`loader.rs:1297`) |
| Ctrl+J does not fire on Ctrl+Shift+J | T17 (`:1108`), T18 (`:1124`) |
| LShift and RWin in keys fire only for that side; Shift fires for both | T1 (`:753`), T2 (`:776`), T3 (`:811`), T4 (`:842`) |
| Shift in a chord fires | T5 (`:872`), T6 (`:901`) |
| Shift+CapsLock fires ToggleCapsLock | T22 (`:1173`), `more_specific_immediate_binding_wins` (`:100`) |
| sided rule in sequence steps | T7 (`:913`), T8 (`:945`) |
| deferred arms keyed on the trigger key | T14 (`:1046`), T11 (`:986`) |
| README / default_config / CHANGELOG updated | `config::loader::tests::default_config_parses` (`loader.rs:478`), `readme_matching_examples_load` (`loader.rs:1312`) |

### Unit tests added (impl.md:63-67; names at HEAD)
- `data::key_spec::tests` (`src/data/key_spec.rs:200-352`): 11 tests incl. `overlaps_table` `:298`, `modifier_set_holds_families` `:314`, `display_round_trips_through_parse` `:341`.
- `data::key_combination::tests` (`src/data/key_combination.rs:89-160`): `from_keys_dedups_and_keeps_order`, `equality_ignores_order`, `generic_modifier_codes_become_either_side`, `modifiers_collects_families`, `display_joins_with_plus`.
- `data::binding::tests::display_marks_allow_extra_modifiers` (`src/data/binding.rs:211`).
- `config::parsing::tests`: `parse_combo_keeps_sides` `:101`, `parse_combo_win_is_either_side_lwin_is_left` `:114`, `parse_key_list_keeps_output_codes` `:127`.
- `config::loader::tests`: `chord_with_an_unsided_modifier_parses_to_either_side` `:1254`, T9 `:1267`, `allow_extra_modifiers_defaults_to_false` `:1289`, `allow_extra_modifiers_true_is_parsed` `:1297`, `readme_matching_examples_load` `:1312`, T21 `:1339`.
- `engine::keys::tests`: `any_side_is_down_for_either_side_or_the_generic_slot` `:138`, `sided_key_is_down_only_on_its_side` `:164`, `is_blocked_reads_without_clearing` `:211`.
- `engine::matching::tests` (`src/engine/matching.rs:87-205`): 8 tests.
- `engine::chords::tests`: `unsided_shift_chord_completes_with_either_shift` `:266`, `completed_reports_the_pressed_codes` `:293`.
- `engine::sequences::tests::sided_step_needs_its_side` `:292`; `ctrl_x_ctrl_s_sequence` `:258` changed (deviation 5).
- `system::registry::tests::language_hotkeys_are_either_side_modifiers` (`src/system/registry.rs:278`).

## Open points from impl.md (verbatim scope, impl.md:83-104, :161-171)

1. Pre-existing rustdoc warning `src/data/vk_name.rs:4` (unclosed `<raw>`), untouched.
2. Orchestrator items: book sync not done (cut Q2/D12; list in cut.md "Docs impact" `:461-507` plus KS-06 rows: `audit.md` status, `config/known-issues.md`, `roadmap.md` A bullet "Suppresses repeats symmetrically", `architecture.md` key-event path); `CLAUDE.md:20` `src/data/` list lacks `key_spec.rs` and the engine file list lacks `matching.rs`; reviewer R8 text (`.claude/agents/code-reviewer.md:126`) still describes `active_keys` folding and "Any Windows key is `VK_LWIN`" (cut Q4).
3. KS-10 interplay (deviation 2): repeats still fire `Immediate` actions (`src/engine/mod.rs:329-333`); on the I2 path a repeat that starts matching after its key-down passed fires the action while the repeat reaches Windows; not pinned by a test.
4. KS-14: a swallowed modifier is erased by the resync and then counts as not held for exact matching; left to the KS-14 lane / roadmap 1.1. C9 clearing clause (`src/engine/mod.rs:437-442`) is described as only preventing a stuck key.
5. Sequences: a non-modifier key of the last step whose key-down passed before the completing key (`"A+S"`, A held first) is still marked blocked (KS-32 rest, roadmap 3.2); `src/engine/sequences.rs:125-140`, `src/engine/mod.rs:399-401`.
6. `#[allow(dead_code)]` on `KeyCombination::with` (`src/data/key_combination.rs:41`) and `Binding::new` etc. is vestigial; untouched.
7. Work-tree line endings: some edited files are LF; `core.autocrlf=true`; `git status` clean.
8. Critique open questions as answered: Q1 D2 kept (non-modifier trigger key, cut.md:511-514); optional roadmap 3.5 trigger check before `condition.evaluate` not done (condition still evaluated first, `src/engine/mod.rs:314`); lane order: KS-06 fixed in this lane (C7-C9), KS-14 stays with its lane.
9. Deviations recorded (impl.md:86-103): C7-C9 added; `win_with` helper; `KeyState::is_active` un-gated from `#[cfg(test)]`; `out_of_range_codes_are_ignored` assertions rewritten (`src/engine/keys.rs:185`); extra unit tests; README "Exact modifiers" states sequences/chords match whatever modifiers are held; comment change at `src/engine/scenarios.rs:286-287`; extra commit `c609a69`.
10. Manual checks M1-M10 (human only, release build, `swch open`, `osk.exe` for stuck keys) at impl.md:119-132; none is an `#[ignore]` test.
11. CHANGELOG "Fixed" claims KS-04, KS-06, KS-07 and KS-32 (partly) (`CHANGELOG.md:80-114`); cut Q6 says no KS-05/KS-18 status change.

## Invariants in play

| Item | Where | Note (fact) |
|---|---|---|
| R1 hook callback | `src/engine/mod.rs:303-445`; `src/engine/keys.rs:35-38`; `src/engine/chords.rs:105-111`; `src/engine/matching.rs:46-67` | per-event work is table reads; chord arm allocates (pre-existing pattern, fact 43); vk indexing bounded by `slot`; adapter unchanged (fact 46) |
| R2 sentinel | no `SendInput` in diff | output keys unchanged (facts 9-10) |
| R3 shared state | engine reached via `with_engine` (`src/hook/keyboard_hook.rs:79`) | no new static/atomic/lock (impl.md:107) |
| R4 orderings | none added | |
| R5 builders | `src/data/binding.rs:140-143`; `src/data/key_combination.rs:35-59` | consume-self `with_*` |
| R6 Display | `src/data/key_spec.rs:110-114, 184-193`; `src/data/key_combination.rs:71-82`; `src/data/binding.rs:150-203` | new field shown as suffix; book config pages not updated (open point 2) |
| R7 specificity | `src/data/binding.rs:40-46`; `src/engine/mod.rs:236-238` | Combo count = `keys.len()` of `KeySpec`s |
| R8 key identity | `src/data/key_spec.rs:134-139`; `src/engine/keys.rs:117-129`; `src/engine/chords.rs:70, 90-93`; `src/engine/sequences.rs:76, 96` | one `KeySpec` compare on combos, steps, chords; R8 rule text outdated (open point 2) |
| R9 registry | `src/system/registry.rs:56, 72-73, 197, 203-206` | read-only; combo construction changed |
| R10 loop/shutdown | not touched | |
| KS-04 | `src/engine/matching.rs:46-51`; `src/engine/mod.rs:319-327` | T10-T14 |
| KS-06 | `src/engine/mod.rs:427-443`; `src/engine/keys.rs:95-97` | C7/C8 tests |
| KS-07 | `src/data/key_spec.rs`; `src/config/loader.rs:303-317` | T1-T9 |
| KS-10 | `src/engine/mod.rs:329-333` | repeats still fire Immediate |
| KS-14 | `src/engine/keys.rs:63-71`; `src/engine/mod.rs:270` | resync vs exact matching (open point 4) |
| KS-05/KS-18 | `src/config/loader.rs:170-189`; scenario `:1148` | blockers exact; no status claim |
| KS-32 | `src/engine/sequences.rs:125-140` | partly fixed (Win in last step) |
| Suppression symmetry (CLAUDE.md principle 2) | `src/engine/mod.rs:427-443, 447-471`; `src/engine/chords.rs:30-32` | |

## Not covered

- Did not run any cargo command (no build, gate or test run); all pass/fail results above are as reported by impl.md and cut.md.
- Did not read the bodies of the KS-07 and KS-04 scenarios (`src/engine/scenarios.rs:750-1074`) beyond their names and line numbers; the Exact-modifiers and KS-06 sections (`:1076-1361`) were read.
- Did not read the unit-test bodies in `key_spec.rs`, `key_combination.rs`, `parsing.rs`, `keys.rs`, `chords.rs`, `sequences.rs`, `registry.rs` (names only).
- Did not verify red-first texts by checking out the red commits.
- Did not read `src/engine/gestures.rs`, `src/engine/env.rs`, `book/`, `docs/WORKFLOW.md`, cut.md sections "Win32 API choices", "Threading", "Risks" (`:189-256, :426-508`).
- README hunks outside "Combo syntax" were read from the diff only, not in full context.
