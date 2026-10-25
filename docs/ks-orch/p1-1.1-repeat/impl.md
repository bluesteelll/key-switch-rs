# Implementation: p1-1.1-repeat round 1

Lane: roadmap item 1.1 "Per-binding `repeat` flag" / KS-10. Branch `feat/repeat-flag` (from `master` `dbd55ca`), HEAD `ad96e73`, tree clean.
Inputs: `D:/tmp/ks-orch/p1-1.1-repeat/cut.md` (CUT READY), `D:/tmp/ks-orch/p1-1.1-repeat/critique.md` (CHANGES_REQUESTED: I1, M1-M4). Addendum with the remark resolutions and the restated Q1: end of `cut.md`.

## Items

- **1.1 / KS-10, commit 1 (RED) `5c43211`** - tests only: `src/config/loader.rs:1866-1972` (loader tests), `src/engine/scenarios.rs:1576-1620` (section, helpers `repeats`, `hold_with_repeats`, one scenario). Red on its parent (= `master`), each for the predicted reason:
  - `config::loader::tests::repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings`: red "expected Bindings error for Tap(200), got Parse { .. NoSuchStructField { .. found: \"repeat\", outer: Some(\"RawBinding\") } .. }" -> green (commit 2)
  - `config::loader::tests::repeat_false_loads_on_every_kind_and_trigger`: red "Immediate: repeat: false must load: config parse error (<in-memory>): 1:59: Unexpected field named `repeat` in `RawBinding`, .." -> green
  - `config::loader::tests::an_unknown_binding_field_lists_repeat_among_the_valid_names`: red "... `when`, `trigger`, or `allow_extra_modifiers` instead" -> green
  - `config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default`: red "config loads: Parse { .. found: \"repeat\" .. }" -> green
  - `engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat`: red "scenario: config does not load: config parse error (<in-memory>): 1:64: Unexpected field named `repeat` in `RawBinding`, .." -> green
- **1.1 D1/D2/D4/D5, commit 2 `96528bb`** - `src/config/schema.rs:69-75` (`repeat: Option<bool>`, `deny_unknown_fields` kept); `src/core/windows_actions.rs:160-184` (`BindAction::repeats_by_default`, exhaustive, no `_` arm), tests `:315-358`; `src/data/binding.rs:72-79` (field), `:96`, `:116` (constructors), `:156-161` (`with_repeat`), `:187-209` (Display suffix), test `:256-267`; `src/config/loader.rs:374-399` (load check + builder). Green-on-arrival API tests: `core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default`, `data::binding::tests::a_binding_takes_its_repeat_default_from_its_action`, `config::loader::tests::repeat_defaults_follow_the_action_and_an_explicit_value_wins`, `repaet` row in `unknown_fields_are_errors_that_name_the_field` (`loader.rs:1078-1081`). Engine unchanged in this commit.
- **KS-10, commit 3 (RED) `6a8e20c`** - `src/engine/testing.rs:30-41` (doc), `:150-152`, `:199`, `:224-245` (`check_repeats_fire_only`, `reload`), `:494-519` (`check_repeat_fire`); `src/engine/properties.rs:8-26`, `:200-253`; `src/engine/scenarios.rs` (edited `:1307-1335`, replaced `:1515-1549`, new section `:1576-1974` (`TEMPLATE` and scenarios from `:1622`), harness self-test `:2879-2886`). Red on commit 2 (engine = `master`), all 13 exactly as predicted:
  - `default_capslock_held_with_ten_repeats_switches_the_language_once`, `capslock_held_across_a_focus_change_switches_the_language_once`, `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat`: red "scenario: repeat CapsLock at +500 ms: expected to fire [], fired [SwitchLanguage]"
  - `capslock_held_through_remapping_off_and_on_switches_the_language_once`, `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up`: red "scenario: repeat CapsLock at +533 ms: expected to fire [], fired [SwitchLanguage]"
  - `held_shift_capslock_toggles_capslock_once`: red "... repeat CapsLock at +500 ms: expected to fire [], fired [ToggleCapsLock]"
  - `held_launch_and_sequence_hotkeys_fire_once_per_press`: red "... repeat T at +500 ms: expected to fire [], fired [Launch { exe: \"wt.exe\", args: [] }]"
  - `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`: red "... repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]"
  - `only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default`: red "... repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]" / "context: SwitchLanguage"
  - `volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not`: red "... repeat F13 at +500 ms: expected to fire [], fired [Media(PlayPause)]" / "context: Media(PlayPause)"
  - `repeat_false_on_press_key_fires_once_per_press`: red "... repeat F13 at +500 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(37))]" (on `master` instead: "scenario: config does not load: .. Unexpected field named `repeat` in `RawBinding`", see M3)
  - `press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding`: red "... down F13 at +1000 ms: expected to fire [], fired [SwitchLanguage]"
  - `engine::properties::auto_repeats_fire_only_actions_that_repeat`: red "scenario: repeat CapsLock at +699 ms: fired [SwitchLanguage] on an auto-repeat; only [PressKey(VIRTUAL_KEY(37))] may fire on one" / "context: config default, seed 0, Focus mode"
  - Declared green characterizations, green on commit 2 and after: `press_key_backspace_held_with_ten_repeats_fires_eleven_times`, `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, `repeat_check_reports_an_action_fired_on_an_auto_repeat` (should_panic).
- **KS-10 D3, commit 4 `9461a41`** - `src/engine/mod.rs:405-418` (Immediate arm: slot taken on every key-down, fire only `!was_repeat || binding.repeat`), module docs `:120-126`, `:160-162`. Every commit-3 red -> green; `engine:: config:: data:: core::` 325 passed. The KS-06 suppression block and `on_key_up` are byte-identical. README, template, CHANGELOG in the same commit.
- **Docs, commit 5 `ad96e73`** - nine `book/src/config/*.md` pages; CHANGELOG KS-10 wording fix.

## Remarks

- **I1** (Important) - RESOLVED: `CHANGELOG.md` Fixed KS-10 entry and `book/src/config/known-issues.md` row 63 no longer promise item 2.3 for swallowed keys; scenario comment at `src/engine/scenarios.rs:1516-1524`; pinned by `engine::scenarios::press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding` (lost-key-up press fires nothing, the press after it fires). Q1 restated with the two cases in the `cut.md` addendum, for the orchestrator to put to the owner.
- **M1** - RESOLVED: manual check #5 below checks the marker after a restart; the marker is pinned by `config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default`.
- **M2** - RESOLVED: owner default untouched; pitfall `repeat: false` for `PressKey` of a toggle key or modifier in README (`repeat` row, Auto-repeat policy bullet worded without "like a real key"), `default_config.ron` repeat block, CHANGELOG Changed entry, `actions.md` 13.1 note and 13.5, `known-issues.md` row 64.
- **M3** - RESOLVED: `master`-replay message for `repeat_false_on_press_key_fires_once_per_press` given here, in the commit-3 body and in the `cut.md` addendum.
- **M4** - RESOLVED: `example.md` VS Code `F24` macro stays once per press (comment reworded); `repeat: true` shown on a new `( keys: "Ctrl+Alt+]", action: Media(Next), repeat: true )`; validated `OK: 26 bindings`.
- Critic's open questions for the architect (OS key state for passed keys; slot owner of a press's repeats after a focus change, item 1.6): not remarks to resolve here; listed under open points.

## Deviations from the cut

- `src/engine/scenarios.rs`: a second private helper `repeats(scenario, key, count, on_repeat)` (used by `hold_with_repeats` and by the scenarios that repeat a key without pressing/releasing it in the helper), and `template()` wrapping `Scenario::from_ron_with_language_hotkey(TEMPLATE, &[VK_MENU, VK_SHIFT])`. Test-only.
- The replaced lost-key-up scenario also pins `up F13` passed and the next `down F13` firing, the claim the CHANGELOG and the book make (I1).
- `example.md` (M4): `repeat: true` on a new `Media(Next)` binding instead of on the VS Code `F24` `Sequence`; binding count 26 (cut: stays 25). Three-key checking-order sentence updated.
- CHANGELOG KS-10 Fixed: I1 wording instead of "(KS-29, roadmap item 2.3)"; "its release passes" dropped in commit 5, because the release of such a press passes only after a foreground change (otherwise it stays blocked and is suppressed); "the next press works" holds either way.
- `known-issues.md`: row 64 added (M2); heading 19.9 "New with Phase 0" -> "New with Phase 0 and later" (anchor `s19-9` unchanged, two inbound links still resolve).
- Book pages: the "Facts describe commit `1dfd80a`" sentence of the seven edited pages, the known-issues closing note and the config README intro add that `repeat` facts describe the branch `feat/repeat-flag` (the book sync replaces it with the merge sha). `actions.md`: the KS-10 warning became a NOTE before the known-issues list ("Four" -> "Three known issues"); 13.8 `Media` gets one sentence on the non-volume keys.
- `loader.rs` test label uses `"Sequence[F13 \u{2192} F14]"` so the source stays ASCII.
- Commit subjects follow the cut (`Config: red tests ...`), not the older `Tests: red-first: ...` pattern.
- The scratch `cfgcheck` prints each binding's `Display` and `repeat` instead of the Debug fields of the methodology harness, to see the markers.

## Unsafe / atomics added

- None. No `unsafe`, no atomics, no lock, no new shared state: `repeat` is immutable data inside the `ArcSwap<Vec<Binding>>` snapshot. The hook adapter and `with_engine` are untouched.

## Gates

- `cargo fmt --check`: pass (every commit). `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass (every commit, no new `allow`).
- Tests run: every red-first above on its parent and after its fix; at HEAD `cargo test --locked --lib -- engine:: config:: data:: core:: hook::`: 355 passed, 0 failed, 1 ignored (pre-existing `src/data/condition.rs:519`, manual foreground capture). Full `cargo test --locked` left to the tester.
- Manual ignores added: none (every decision is in the engine; no test needs a live hook).
- Book: `mdbook build book -d D:/tmp/ks-orch/p1-1.1-repeat/book` ok; link check of the built HTML: 2004 internal links, 0 broken.
- cfgcheck (`D:/tmp/ks-orch/p1-1.1-repeat/cfgcheck`, `config::reload`, built on the branch): every `ron` block of `structure.md`, `triggers-conditions.md` and `example.md` loads (example: `OK: 26 bindings`, `Ctrl + Alt + ] -> media Next  [repeat]`; structure example: `[no repeat]`). The template's commented examples load through `default_config_commented_examples_load`.

## Manual checks (human only; release build, `swch open` from a normal console, default config unless noted)

| # | Setup | Steps | Expected |
|---|---|---|---|
| 1 | Notepad, `osk.exe` open, two layouts | Hold CapsLock 2 s, release | The layout switches exactly once; CapsLock state and light unchanged; nothing held in `osk.exe` |
| 2 | default | Hold Shift+CapsLock 2 s | CapsLock toggles exactly once |
| 3 | Add `( keys: "F9", action: PressKey("Backspace") )`; Notepad with text | Hold F9 1.5 s | Characters are deleted continuously at the auto-repeat rate |
| 4 | Add `( keys: "Ctrl+Alt+Up", action: Media(VolumeUp) )` and `( keys: "Ctrl+Alt+P", action: Media(PlayPause) )`; media playing | Hold each 2 s | The volume climbs continuously; playback toggles once |
| 5 | Change the CapsLock binding to `repeat: true` and save | Read the console; hold CapsLock 2 s; then restart the daemon and read the startup list | After the save `[INFO] config reloaded (N bindings live)` and the layouts cycle while CapsLock is held; after the restart the CapsLock line ends with `[repeat]` (a reload prints no binding list, M1) |
| 6 | Add `( keys: "Win+Alt+T", action: Launch(exe: "wt.exe") )` | Hold Win+Alt+T 2 s | Exactly one terminal window |
| 7 | Daemon running; save `( keys: "F9", action: SwitchLanguage, trigger: Tap(200), repeat: true )` | Read the console | `[ERROR] config reload failed:` with ``binding #N [F9]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger``; the old bindings stay live |

## Docs

- `README.md`: `Binding` field list; `Immediate` row of the triggers table; behaviour rule; per-binding flags row `repeat` (with the M2 pitfall); unknown-field message with `repeat`; new error bullet; Auto-repeat policy bullet.
- `src/config/default_config.ron`: `trigger: Immediate` line; `repeat:` block in the per-binding flags (with the M2 pitfall); Media examples comment and `( keys: "Ctrl+Alt+PageDown", action: Media(Next), repeat: true )`.
- `CHANGELOG.md` `[Unreleased]`: Added (field), Changed (**Breaking:** once per press; M2 note), Fixed (KS-10, I1 wording), KS-06 entry sentence updated.
- `book/src/config/`: `structure.md`, `README.md`, `kinds.md`, `triggers-conditions.md`, `actions.md`, `known-issues.md` (row 8 fixed, rows 62-64), `errors.md` (order sentence, row 18), `matching.md`, `example.md`.

## Commits

- `5c43211` Config: red tests for the per-binding repeat field (1.1)
- `96528bb` Config: per-binding repeat field with per-action defaults (1.1)
- `6a8e20c` Engine: red scenarios for auto-repeat of Immediate bindings (KS-10)
- `9461a41` Engine: Immediate bindings fire on auto-repeat only with repeat (KS-10)
- `ad96e73` Book: the repeat field in the config reference (1.1, KS-10)

## Open points / pre-existing warnings

- Pre-existing warnings: none. Pre-existing ignored test: `src/data/condition.rs:519` (manual).
- Q1 (restated in the `cut.md` addendum) needs the owner: accept the lost first press after a lost key-up for swallowed keys, or extend item 2.3's acceptance to swallowed keys.
- Post-merge book sync (doc-writer, needs the merge sha): `book/src/audit.md` KS-10 status (`:33`, `:349`), `roadmap.md` KS-10 row (`:614`) and a 1.1 status block, `architecture.md`, `glossary.md`, `overview.md`, `gaps.md` KS-10 mentions, and replacing "`feat/repeat-flag`" with the merge sha in the config pages (known-issues rows 8, 62, 64 Where column; page headers).
- Critic's questions for the architect: (1) a passed key's lost key-up could be told from a repeat by the OS key state (KS-29/2.3 scope); (2) after a focus change mid-hold, a `when:`-gated pair on one key hands the repeats to the repeating fallback, as on `master` (item 1.6 note).
- `book/src/config/known-issues.md` heading 19.9 now spans Phase 0 and later rows; the doc-writer may prefer another title.
