# Context pack, round 1: p1-1.1-repeat (roadmap 1.1, KS-10)

Scout: project-analyst (read-only). For: tester and code-reviewer, round 1.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/repeat-flag`, HEAD `ad96e73`; `master` = `dbd55ca` = merge base.
- `git log --oneline master..feat/repeat-flag`:
  - `ad96e73` Book: the repeat field in the config reference (1.1, KS-10)
  - `9461a41` Engine: Immediate bindings fire on auto-repeat only with repeat (KS-10)
  - `6a8e20c` Engine: red scenarios for auto-repeat of Immediate bindings (KS-10)  [declared RED commit]
  - `96528bb` Config: per-binding repeat field with per-action defaults (1.1)
  - `5c43211` Config: red tests for the per-binding repeat field (1.1)  [declared RED commit]
- `git diff --stat master..feat/repeat-flag`: 20 files, +869 / -76.
  - Code: `src/config/loader.rs` +132, `src/config/schema.rs` +8, `src/core/windows_actions.rs` +72, `src/data/binding.rs` +44, `src/engine/mod.rs` 22, `src/engine/properties.rs` 34, `src/engine/scenarios.rs` +394/-5, `src/engine/testing.rs` 57.
  - Docs: `CHANGELOG.md` 28, `README.md` 15, `src/config/default_config.ron` 21, `book/src/config/{README,actions,errors,example,kinds,known-issues,matching,structure,triggers-conditions}.md`.
  - Not in the delta: `src/hook/*`, `src/core/app.rs`, `src/ipc.rs`, `src/system/*`, `Cargo.toml`, `Cargo.lock`, `build.rs`.
- Lane inputs: `D:/tmp/ks-orch/p1-1.1-repeat/cut.md` (CUT READY, D1-D7, addendum at `:498-531`), `critique.md` (I1, M1-M4), `impl.md` (developer report).

## File map

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/config/schema.rs:31-75` | `RawBinding`, `deny_unknown_fields` at `:31`, new `repeat: Option<bool>` `#[serde(default)]` at `:69-75` | spec: field on `RawBinding`, unknown fields still rejected |
| `src/core/windows_actions.rs:160-185` | `BindAction::repeats_by_default`, exhaustive nested match, no `_` arm | owner default (D2) |
| `src/core/windows_actions.rs:315-358` | new `mod tests`, `only_press_key_volume_up_and_volume_down_repeat_by_default` | table over every action and 7 `MediaKey`s |
| `src/data/binding.rs:72-79` | `pub repeat: bool` + doc | D1 |
| `src/data/binding.rs:93-96`, `:107-118` | `with_kind` sets `repeat: action.repeats_by_default()`; `new_auto_blocker` sets `repeat: false` | defaults for code-built bindings (`src/core/app.rs:224`, `src/hook/keyboard_hook.rs:280`) |
| `src/data/binding.rs:156-161` | `with_repeat(mut self, bool) -> Self` | R5 builder |
| `src/data/binding.rs:187-210` | Display: `honours_repeat`, `repeat_suffix` `"  [repeat]"` / `"  [no repeat]"`, appended after `extra_suffix` | D5, R6 |
| `src/data/binding.rs:256-267` | `a_binding_takes_its_repeat_default_from_its_action` | unit test |
| `src/config/loader.rs:341-399` | `raw_binding_to_binding`; check order: kind `:345`, action `:347`, trigger `:350`, deferred-on-seq/chord `:356`, `allow_extra_modifiers` `:367`, new repeat check `:374-383`, builder `:385-390`, `with_repeat` only when `Some` `:391-395` | D1, D4 |
| `src/config/loader.rs:238-243` | `IMPLICIT_SOME` | lets `repeat: true` load into `Option<bool>` |
| `src/config/loader.rs:1051-1100` | `unknown_fields_are_errors_that_name_the_field`, new `repaet` row `:1078-1081` | deny_unknown_fields pin |
| `src/config/loader.rs:1866-1976` | new repeat tests (`NEVER_REPEATING` `:1870`, tests `:1891`, `:1911`, `:1926`, `:1942`, `:1957`) | commit 1 reds + commit 2 API test |
| `src/engine/mod.rs:117-128`, `:160-162` | module docs: Immediate rule rewritten; KS-10 dropped from "known defects" list | doc vs code |
| `src/engine/mod.rs:368-374` | `on_key_down`: `was_repeat = self.keys.press(vk)` `:370`, `immediate_fired` `:374` | repeat detection |
| `src/engine/mod.rs:407-420` | Immediate arm: slot taken on every key-down `:411-412`; fire only `!was_repeat \|\| binding.repeat` `:413-415`; `block_default` `:417-419` | the only decision-path change (D3) |
| `src/engine/mod.rs:421-505` | Tap `:421-422`, Hold `:433-434`, DoubleTap `:447-448`, Sequence `:463-464`, Chord `:488-489`: each starts with `if was_repeat` | "deferred/seq/chord ignore repeats", unchanged |
| `src/engine/mod.rs:510-543` | KS-06 / A3 repeat-suppression block (`is_blocked` / `is_passed` / `mark_passed`) | must be unchanged |
| `src/engine/mod.rs:546-573` | `on_key_up` | must be unchanged |
| `src/engine/keys.rs:134-148` | `key_down_swallowed` `:134`, `press` `:143` (returns was-held) | repeat identity = raw vk |
| `src/engine/testing.rs:30-41` | module doc: three checks, `reload` paragraph | D7 |
| `src/engine/testing.rs:150-152`, `:199` | field `repeat_fire_allowed: Option<Vec<BindAction>>`, init `None` | D7 |
| `src/engine/testing.rs:225-245` | `check_repeats_fire_only` `:227`, `reload` `:236` (from_ron_str, `sort_by_specificity`, no engine reset, step `"reload at .."`) | D7 |
| `src/engine/testing.rs:475-503`, `:504-520` | `key_event` calls `check_repeat_fire` when `what == "repeat"` `:498`; `check_repeat_fire` `:506` | D7 |
| `src/engine/testing.rs:374-400` | `advance_ms` runs due timers inside the advance step | timer fires are not recorded on a `repeat` step |
| `src/engine/properties.rs:8-26` | module doc: third property | |
| `src/engine/properties.rs:72-136` | `CONFIGS` (6 configs) | `PressKey("Left")` Immediate at `:88`; `PressKey("LShift")` is `Hold(200)` at `:99` |
| `src/engine/properties.rs:203-255` | `run(.., repeat_check)` `:203`, check enabled `:219-221`, `run_all` `:235`, tests `:244`, `:249`, new `:254` | property pin |
| `src/engine/scenarios.rs:9` | import `MediaKey` added | |
| `src/engine/scenarios.rs:1307-1335` | edited `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (comment `:1311-1312`, `expect_nothing_fired` at `:1322`, `:1328`, `:1332`) | KS-06 + KS-10 interplay |
| `src/engine/scenarios.rs:1516-1551` | replaced lost-key-up scenario (was `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding`) | KS-29 corner (Q1) |
| `src/engine/scenarios.rs:1576-1974` | new section: helpers `repeats` `:1582`, `hold_with_repeats` `:1599`; `TEMPLATE` `:1624` (`include_str!("../config/default_config.ron")`), `template()` `:1628` (Alt+Shift stand-in hotkey); scenarios `:1613-1974` | acceptance |
| `src/engine/scenarios.rs:2879-2887` | harness self-test `repeat_check_reports_an_action_fired_on_an_auto_repeat` (`should_panic`) | D7 |
| `src/config/default_config.ron:104-106`, `:164-175`, `:226-234` | trigger comment; `repeat:` flags block (with toggle-key/modifier pitfall `:173-175`); Media examples incl. `( keys: "Ctrl+Alt+PageDown", action: Media(Next), repeat: true )` commented at `:234` | template; loaded by `TEMPLATE` scenarios and `default_config_commented_examples_load` (`loader.rs:658`) |
| `README.md:105`, `:351`, `:386-390`, `:466`, `:522`, `:532-534`, `:667` | field list, triggers table, behaviour rule, flags row, unknown-field message, new error bullet, Auto-repeat policy bullet | doc rule (CLAUDE.md "Docs and the book") |
| `CHANGELOG.md:48-55`, `:125-131`, `:218-219`, `:235-244` | Added; Changed (**Breaking:**); KS-06 entry edit; Fixed KS-10 | Keep a Changelog |
| `book/src/config/*.md` (hunks below) | config reference | doc rule |

Book hunk starts (new-side line): `README.md` 5, 44; `actions.md` 3, 42, 51-52, 54, 57, 167, 238, 293; `errors.md` 3, 136, 160; `example.md` 6, 12, 15, 18-19, 47-48, 76-77, 82-85, 105-107, 127-129, 187-188, 234, 239, 241; `kinds.md` 3, 26, 33-34, 65, 132; `known-issues.md` 34, 118, 130-132, 137; `matching.md` 3, 132; `structure.md` 8, 152, 165, 186, 195; `triggers-conditions.md` 3, 13, 46-54.

## Facts

1. `RawBinding` keeps `#[serde(deny_unknown_fields)]` at `src/config/schema.rs:31`; the new field is the last one, `pub repeat: Option<bool>` with `#[serde(default)]` at `src/config/schema.rs:74-75`.
2. `BindAction::repeats_by_default` returns `true` for `PressKey(_)`, `Media(VolumeUp)`, `Media(VolumeDown)` and `false` for every other variant, with no `_` arm (`src/core/windows_actions.rs:165-185`).
3. `Binding::with_kind` computes `repeat: action.repeats_by_default()` before moving `action` (`src/data/binding.rs:96-97`); `new_auto_blocker` sets `repeat: false` (`src/data/binding.rs:116`).
4. `with_repeat` is a consuming builder; no `set_repeat` exists (`src/data/binding.rs:158-161`; grep `set_repeat` over `src`: 0 matches).
5. Display appends `"  [repeat]"` when `repeat && !default`, `"  [no repeat]"` when `!repeat && default`, only for `BindingKind::Combo` with `Trigger::Immediate`; otherwise `""` (`src/data/binding.rs:190-196`); the auto-blocker branch does not print it (`src/data/binding.rs:198-203`).
6. The loader rejects `raw.repeat == Some(true)` unless `kind` is `Combo` and the trigger is Immediate, with message `` "{prefix}: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger" `` (`src/config/loader.rs:377-383`); this check runs after the `allow_extra_modifiers` check (`src/config/loader.rs:367-372`).
7. `repeat: None` keeps the action default; `Some(v)` applies `with_repeat(v)` (`src/config/loader.rs:391-395`).
8. `Trigger::is_immediate` existed on `master` (`git show master:src/data/trigger.rs`, line 37); no change to `src/data/trigger.rs` in the delta.
9. Engine Immediate arm: the first matching Immediate user binding sets `immediate_fired = true` on every key-down, repeats included, and pushes `binding.action.clone()` only when `!was_repeat || binding.repeat` (`src/engine/mod.rs:411-416`).
10. `block_default` still sets `decision.suppress = true` for every matching Immediate binding, independent of `was_repeat` (`src/engine/mod.rs:417-419`).
11. `git diff master..feat/repeat-flag -- src/engine/mod.rs` has three hunks only: docs `:117-128`, docs `:158-162`, Immediate arm `:405-415`. The KS-06 block (`:510-543`) and `on_key_up` (`:546-573`) are not in the diff.
12. Every deferred, sequence and chord arm returns early on `was_repeat` (`src/engine/mod.rs:422`, `:434`, `:448`, `:464`, `:489`); none reads `binding.repeat` (grep `binding.repeat` in `src/engine/mod.rs`: only `:413`).
13. No `unsafe`, atomic, lock, thread or `SendInput` is added in the delta (`impl.md:51-53`; the code diffs above contain none; `src/hook/*` is not in the diffstat).
14. `Scenario::reload` loads with `from_ron_str`, calls `sort_by_specificity` (`src/engine/mod.rs:268`, also used by `KeyboardHook::update_bindings` at `src/hook/keyboard_hook.rs:113`), replaces `self.bindings` and does not reset the engine (`src/engine/testing.rs:236-244`).
15. `check_repeat_fire` fails with `"{label}: fired {fired:?} on an auto-repeat; only {allowed:?} may fire on one"` when any action fired on the last `repeat` step is not in the allowed list (`src/engine/testing.rs:506-519`); it inspects only `on_key` fires, because timer fires are recorded in `advance_ms` steps (`src/engine/testing.rs:374-400`, `:475-496`).
16. The property `auto_repeats_fire_only_actions_that_repeat` is `run_all(Mode::Focus, true)` with allowed `[PressKey(VK_LEFT)]` (`src/engine/properties.rs:219-221`, `:254-256`); the two existing properties pass `false` (`:244-251`).
17. `hold_with_repeats` expects the key-down suppressed, every repeat suppressed and firing exactly `on_repeat`, the key-up suppressed and firing nothing; it does not check the key-down's fire (`src/engine/scenarios.rs:1594-1610`, `repeats` at `:1582-1592`).
18. `TEMPLATE` is the shipped `src/config/default_config.ron` via `include_str!` (`src/engine/scenarios.rs:1624`), loaded with `Alt+Shift` as the stand-in language hotkey (`:1628-1630`).
19. The replaced lost-key-up scenario pins: second `down F13` after `up_unseen` + `foreground_changed` is suppressed and fires nothing for `SwitchLanguage`; its `up` passes; the next `down` fires `SwitchLanguage`; with `PressKey("Left")` the lost-key-up press fires `PressKey(VK_LEFT)` (`src/engine/scenarios.rs:1525-1550`).
20. `capslock_held_across_a_focus_change_switches_the_language_once` expects the CapsLock key-up **passed** after the focus change (`src/engine/scenarios.rs:1792-1794`).
21. `capslock_held_through_remapping_off_and_on_switches_the_language_once` expects every repeat after `remapping_off` passed and firing nothing, and the key-up passed (`src/engine/scenarios.rs:1805-1824`).
22. `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` uses `PressKey("Backspace")` (default `repeat: true`) on Tap(200), Hold(200), DoubleTap(600), a 2-step sequence and a chord (`src/engine/scenarios.rs:1849-1945`).
23. `example.md` now yields `OK: 26 bindings` (25 from the file + auto-blocker) per `impl.md:36`, `:43`; the cut predicted 25 (`cut.md:309`, `:387`).
24. `CHANGELOG.md:242` is 123 characters long; it is the only line over 80 characters that the delta adds to `CHANGELOG.md` (`git diff -U0 master..feat/repeat-flag -- CHANGELOG.md`).
25. A hot reload prints only `[INFO] config reloaded (N bindings live)` (`src/hook/config_watcher.rs:163`), so the `[repeat]` marker is visible only in the startup list (`impl.md:71`, M1).
26. Book config pages say the `repeat` facts describe the branch `feat/repeat-flag` (e.g. `book/src/config/known-issues.md:137`, `book/src/config/example.md:19`, `book/src/config/actions.md:52`, `book/src/config/kinds.md:34`); `impl.md:94` lists their replacement by the merge sha as post-merge book sync.
27. `book/src/config/errors.md:160` is the new table row 18 with the D4 message; `book/src/config/known-issues.md:34` (row 8 Fixed), `:130-132` (rows 62-64).

## Red-first tests and acceptance proofs

Commands (all from `D:/key-switch-rs/key-switch-rs`; never `--ignored` / `--include-ignored`):

- Full gate (tester owns it): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Expected at HEAD: green; developer ran only `cargo test --locked --lib -- engine:: config:: data:: core:: hook::` = 355 passed, 0 failed, 1 ignored (`src/data/condition.rs:519`, pre-existing manual) (`impl.md:57-58`). Full `cargo test --locked` not yet run by anyone (`impl.md:58`).
- Red replay on a parent commit needs a detached checkout or a separate worktree; the analyst role does not do that. Commit messages carry the observed messages (`git show 6a8e20c`).
- Single test: `cargo test --locked --lib -- <module::path::name> --exact`.

### Commit 1 (`5c43211`) reds, parent = `master` `dbd55ca`; green from `96528bb`

| test | expected red on parent (`impl.md:9-13`) |
|---|---|
| `config::loader::tests::repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings` | `expected Bindings error for Tap(200), got Parse { .. NoSuchStructField { .. found: "repeat", outer: Some("RawBinding") } .. }` |
| `config::loader::tests::repeat_false_loads_on_every_kind_and_trigger` | `Immediate: repeat: false must load: config parse error (<in-memory>): 1:59: Unexpected field named `repeat` in `RawBinding`, ..` |
| `config::loader::tests::an_unknown_binding_field_lists_repeat_among_the_valid_names` | message ends ``... `trigger`, or `allow_extra_modifiers` instead`` |
| `config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default` | `config loads: Parse { .. found: "repeat" .. }` |
| `engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat` | `scenario: config does not load: config parse error (<in-memory>): 1:64: Unexpected field named `repeat` in `RawBinding`, ..` |

### Commit 2 (`96528bb`) green-on-arrival API tests (`impl.md:14`, `cut.md:243-253`)

`core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default`, `data::binding::tests::a_binding_takes_its_repeat_default_from_its_action`, `config::loader::tests::repeat_defaults_follow_the_action_and_an_explicit_value_wins` (expects `[true, true, false, false, true, false]`, `loader.rs:1938`), `repaet` row of `config::loader::tests::unknown_fields_are_errors_that_name_the_field`.

### Commit 3 (`6a8e20c`) reds, parent = `96528bb` (engine = `master`); green from `9461a41` (`impl.md:15-26`)

| test | expected red on parent |
|---|---|
| `engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once` | `scenario: repeat CapsLock at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::capslock_held_across_a_focus_change_switches_the_language_once` | same as above |
| `engine::scenarios::reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` | same as above |
| `engine::scenarios::capslock_held_through_remapping_off_and_on_switches_the_language_once` | `scenario: repeat CapsLock at +533 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` | same `+533 ms` message |
| `engine::scenarios::held_shift_capslock_toggles_capslock_once` | `... repeat CapsLock at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| `engine::scenarios::held_launch_and_sequence_hotkeys_fire_once_per_press` | `... repeat T at +500 ms: expected to fire [], fired [Launch { exe: "wt.exe", args: [] }]` |
| `engine::scenarios::non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` | `... repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default` | `... repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]`, `context: SwitchLanguage` |
| `engine::scenarios::volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not` | `... repeat F13 at +500 ms: expected to fire [], fired [Media(PlayPause)]`, `context: Media(PlayPause)` |
| `engine::scenarios::repeat_false_on_press_key_fires_once_per_press` | `... repeat F13 at +500 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(37))]`; on `master` instead `scenario: config does not load: .. Unexpected field named `repeat` in `RawBinding`` (M3, `cut.md:524-526`) |
| `engine::scenarios::press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding` | `... down F13 at +1000 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::properties::auto_repeats_fire_only_actions_that_repeat` | `scenario: repeat CapsLock at +699 ms: fired [SwitchLanguage] on an auto-repeat; only [PressKey(VIRTUAL_KEY(37))] may fire on one`, `context: config default, seed 0, Focus mode` (seed not predicted in the cut, `cut.md:284`) |

Declared GREEN characterizations, green on `96528bb` and after (`impl.md:26`): `engine::scenarios::press_key_backspace_held_with_ten_repeats_fires_eleven_times`, `engine::scenarios::deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, `engine::scenarios::repeat_check_reports_an_action_fired_on_an_auto_repeat` (`should_panic(expected = "on an auto-repeat")`).

Existing tests the cut says must stay green (`cut.md:299-300`): `engine::scenarios::press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` (`scenarios.rs:158`), `engine::scenarios::immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1445`), `config::loader::tests::default_config_parses` (`loader.rs:649`), `config::loader::tests::default_config_commented_examples_load` (`:658`), `config::loader::tests::readme_matching_examples_load` (`:1819`), `engine::properties::every_press_is_suppressed_or_passed_as_a_whole`, `engine::properties::nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups`.

### Acceptance mapping (brief line -> proof)

| acceptance | test (file:line) |
|---|---|
| CapsLock down, 10 repeats, up: one `SwitchLanguage`, every event suppressed | `default_capslock_held_with_ten_repeats_switches_the_language_once` (`scenarios.rs:1633-1638`, shipped template, `check_press_symmetry` + `check_nothing_stuck`) |
| `PressKey("Backspace")` held with 10 repeats fires 11 times | `press_key_backspace_held_with_ten_repeats_fires_eleven_times` (`:1641-1648`) |
| `repeat: true` on `SwitchLanguage` fires on every repeat | `repeat_true_on_switch_language_fires_on_every_repeat` (`:1613-1621`, total 11); `reload_mid_hold_...` (`:1828-1846`) |
| `Media(VolumeUp)` repeats by default, `Media(PlayPause)` does not | `volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not` (`:1651-1671`, totals 4/4/1); `only_press_key_..._by_default` (`:1674-1713`); `core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default` (`windows_actions.rs:322`) |
| Repeat of a non-repeating binding still suppressed when key-down was (A3) | `hold_with_repeats` expectations (`scenarios.rs:1586-1588`, `:1606-1609`); `engine::properties::every_press_is_suppressed_or_passed_as_a_whole` |
| Deferred, sequences, chords ignore repeats (pinned) | `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` (`:1849-1945`); `engine::properties::auto_repeats_fire_only_actions_that_repeat` (`properties.rs:254`); `repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings` (`loader.rs:1891`) |
| Field on `RawBinding`, `deny_unknown_fields` stays | `an_unknown_binding_field_lists_repeat_among_the_valid_names` (`loader.rs:1942`), `repaet` row (`loader.rs:1078-1081`), `repeat_false_loads_on_every_kind_and_trigger` (`:1911`) |
| README, template, book page, CHANGELOG | diff hunks in the File map; `default_config_parses`, `default_config_commented_examples_load`, `readme_matching_examples_load`; `mdbook build book -d D:/tmp/ks-orch/p1-1.1-repeat/book` (developer: ok, 2004 internal links, 0 broken, `impl.md:60`); cfgcheck at `D:/tmp/ks-orch/p1-1.1-repeat/cfgcheck` (`impl.md:61`) |

### Manual checks (human only, not run by agents; `impl.md:63-73`)

Seven rows: hold CapsLock (one switch, osk.exe), hold Shift+CapsLock (one toggle), `PressKey("Backspace")` on F9 (continuous), VolumeUp vs PlayPause, `repeat: true` on CapsLock after save + restart (`[repeat]` in startup list, M1), held `Win+Alt+T` `Launch` (one window), reload with `Tap(200), repeat: true` (reload error, old bindings live). No `#[ignore]` test was added (`impl.md:59`).

## Open points (from `impl.md:90-96`)

1. Q1 (owner, via orchestrator): after a lost key-up of a key whose key-down was **swallowed**, its next press reads as a repeat and a non-repeating binding fires nothing (on `master` it fired); roadmap item 2.3's resync excludes such keys (`book/src/roadmap.md:377` per `cut.md:504`; `src/engine/keys.rs:134-139`). Options: accept as documented, or extend item 2.3's acceptance. Restated at `cut.md:511-514`.
2. Post-merge book sync (doc-writer, needs merge sha): `book/src/audit.md` KS-10 status (`:33`, `:349`), `book/src/roadmap.md:614` and a 1.1 status block, `architecture.md`, `glossary.md`, `overview.md`, `gaps.md` KS-10 mentions, and `feat/repeat-flag` -> merge sha in config pages (Fact 26).
3. Critic's questions for the architect: (a) a passed key's lost key-up could be told from a repeat by the OS key state (KS-29 / 2.3 scope); (b) after a focus change mid-hold, a `when:`-gated pair on one key hands the repeats to the repeating fallback, as on `master` (item 1.6 note).
4. `book/src/config/known-issues.md` heading 19.9 renamed "New with Phase 0 and later" (anchor `s19-9` kept, `impl.md:45`); doc-writer may retitle.
5. Pre-existing warnings: none; pre-existing ignored test `src/data/condition.rs:519` (`impl.md:92`).

Deviations from the cut the developer declared (`impl.md:39-49`): extra helper `repeats` and `template()`; replaced lost-key-up scenario also pins `up F13` passed and the next press firing; `example.md` `repeat: true` on a new `Media(Next)` binding (26 bindings, not 25); CHANGELOG KS-10 wording per I1; `known-issues.md` row 64 added; loader test label uses `"\u{2192}"` for ASCII source; cfgcheck prints `Display` + `repeat`.

## Invariants in play

- **R1 hook/engine hot path**: decision change only at `src/engine/mod.rs:411-416` (one `bool` read; clone moved under the condition); adapter `src/hook/keyboard_hook_callback.rs` not in the delta. KS-08.
- **R2 sentinel**: no new `SendInput`; `PressKey`/`Media` repeats go through `press_vk` (`src/core/windows_actions.rs:190-215`, `dwExtraInfo: injected_sentinel()` at `:200`, `:212`).
- **R3 shared state**: `repeat` is immutable data inside the `ArcSwap<Vec<Binding>>` snapshot; `update_bindings` `src/hook/keyboard_hook.rs:110-115` unchanged; `with_engine` unchanged.
- **R5 builders**: `with_repeat` `src/data/binding.rs:158`; constructors keep two positional args (`:93`, `:107`).
- **R6 Display**: `Binding` Display suffix `src/data/binding.rs:187-208`; no new enum variant; `repeats_by_default` exhaustive (`src/core/windows_actions.rs:165-185`); book config reference updated (errors row 18 `book/src/config/errors.md:160`).
- **R7 specificity**: slot rule keeps "first matching Immediate" on repeats (`src/engine/mod.rs:411-412`), pinned by `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`src/engine/scenarios.rs:1761-1780`); `sort_by_specificity` `src/engine/mod.rs:268` unchanged.
- **R8 key identity**: repeat = raw vk held bit (`src/engine/keys.rs:143-148`).
- **KS-10**: the item fixed. **KS-06 / A3** suppression symmetry: `src/engine/mod.rs:510-543` unchanged. **KS-29**: lost key-up corner (Open point 1). **KS-22** precedent of loud errors: D4 check `src/config/loader.rs:374-383`. **KS-11/25** timers: untouched.
- Owner decision "repeat on by default only for `PressKey`/`VolumeUp`/`VolumeDown`": `src/core/windows_actions.rs:165-185`.

## Not covered

- Did not run `cargo fmt`, `clippy` or `cargo test`, and did not replay reds on parent commits (tester owns the gate).
- Did not read `critique.md` in full (only via `impl.md` / `cut.md` addendum), nor the full text of the nine book pages, `README.md` beyond the hunk lines listed, or `CHANGELOG.md` beyond `:46-60`, `:120-135`, `:214-246`.
- Did not read `src/engine/matching.rs`, `gestures.rs`, `chords.rs`, `sequences.rs`, `src/engine/mod.rs:421-505` in full (only the `was_repeat` early returns), or `src/hook/*`.
- Did not verify the cfgcheck harness output or the built book in `D:/tmp/ks-orch/p1-1.1-repeat/book`.
