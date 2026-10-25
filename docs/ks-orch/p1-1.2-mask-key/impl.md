# Implementation: p1-1.2-mask-key round 1

Branch `feat/mask-key` from `master` `ece1767`, HEAD `0187cde`, tree clean. Nine commits, in the cut's order (1, 3, 5, 7 declared red; 2, 4, 6, 8, 9 green).

## Items

- **KS-12, engine decision (D1)**: `src/engine/keys.rs:39-42` (`MENU_KEYS`), `:229-241` (`alt_or_win_down_in_windows`); `src/engine/mod.rs:646-654` (`mask_if_swallowed_under_alt_or_win`, called after the record update in `on_key_down` and `on_key_up`), `Decision::mask` doc `:294-300`, module docs. Red-first (commit `54e0673`):
  - `win_q_post_message_masks_the_swallowed_q_and_its_release`: red "down Q at +0 ms: expected the menu mask, got none" -> green
  - `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift`: red "down LShift at +0 ms: expected the menu mask, got none" -> green
  - `right_alt_and_right_win_mask_a_swallowed_key`: red "down J at +0 ms: expected the menu mask, got none" -> green
  - `swallowed_repeats_and_key_up_while_win_is_held_are_masked`: red "repeat Q at +500 ms: expected the menu mask, got none" -> green
  - `alt_held_before_the_hook_saw_it_masks_a_swallowed_key`: red "down J at +0 ms: ..." -> green
  - `win_held_through_remapping_on_masks_a_swallowed_key`, `win_held_across_a_foreground_change_masks_a_swallowed_key`: red "down Q at +0 ms: ..." -> green
  - `sequence_and_chord_keys_swallowed_while_win_is_held_are_masked`: red "down F13 at +0 ms: ..." -> green
  - properties `every_press_is_suppressed_or_passed_as_a_whole`: red "down K at +5019 ms: mask false, the rule says true (suppressed true, Alt or Win down in Windows true)" (config default, seed 0, Strict) -> green; `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` and `auto_repeats_fire_only_actions_that_repeat`: red "repeat CapsLock at +1697 ms: mask false, the rule says true ..." (config default, seed 0, Focus) -> green
  - green guards (kill named mutations): `a_swallowed_event_without_alt_or_win_in_windows_is_not_masked` (mask = suppress), `a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed` (count Alt/Win whatever its record), `a_passed_event_is_not_masked` (mask without suppress), `a_hold_timer_never_masks` (mask in `on_timer`, via the new timer assert)
  - unit test `keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed` (commit `82ae3f2`)
- **Harness (D9)**: `src/engine/testing.rs`: `ALT_AND_WIN`, `Step.masked`, `check_mask_rule` `:253`, `expect_masked`/`expect_unmasked` `:462-470`, `check_mask_of` `:562`, timer assert `:415`, ", masked" in the transcript. `src/engine/properties.rs`: mask rule on in every run except the `mask_only` config.
- **KS-12, adapter (D2, D3, D10)**: `src/core/constants.rs:34-48` (`MENU_MASK_VK = 0xE8`); `src/hook/keyboard_hook_callback.rs:198-222` (`carry_out`: no `..`, mask first), `:222-240` (`menu_mask_inputs`), `:241-250` (`inject_menu_mask`), `:253-263` (`send_menu_mask`, `cfg(not(test))`, logs a short count), `:265-277` (`cfg(test)` counter `MASKS_SENT`), module docs step 6 and Re-entry `:20-25`, `:58-82`. Red-first (commit `44adabe`): compile errors "error[E0425]: cannot find value `MASKS_SENT` in this scope", "cannot find function `menu_mask_inputs`", "cannot find function `inject_menu_mask`" -> green (`menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel`, `carry_out_sends_the_menu_mask_only_when_the_decision_asks`; H3 ignored).
- **KS-05 v2, `MaskOnly` (D4, D5, D6)**: `src/config/schema.rs:87-106` (`RawBlockOriginalCombo` with `#[serde(rename = "true"/"false")]`), field `:52-56`; `src/data/binding.rs:69-89` (`BlockerMode` + `Display`), fields, `new_auto_blocker(combo, mode)` `:156-171`, `is_auto_blocker()` `:173-176`, `with_block_original_combo(Option<BlockerMode>)`, blocker `Display` arm; `src/config/loader.rs:602-608` (`raw_blocker_to_mode`), `expand_with_auto_blockers` `:321-401` (modes, `MaskOnly` fallback warning, `warnings` parameter); `src/engine/mod.rs:343-362` (`mask_owed` taken and OR'ed), `:439-467` (blocker branch by mode), `reset` clears `mask_owed`; `src/engine/winner.rs:64`; `src/engine/matching.rs:81-92` doc. Red-first (commit `a3e1149`):
  - `block_original_combo_takes_true_false_and_mask_only`: red "block_original_combo: MaskOnly,: config parse error (<in-memory>): 1:80: Expected boolean" -> green
  - `a_misspelled_block_original_combo_value_names_the_valid_ones`: red "Maskonly: config parse error (<in-memory>): 1:80: Expected boolean" -> green (actual message now ``1:88: Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead``; `Some(true)` gives the same for `` `Some` ``, checked with cfgcheck)
  - `mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns`: red "config does not load: ... 2:78: Expected boolean" -> green
  - `true_and_mask_only_on_one_hotkey_give_one_blocker_each`: red "... 3:81: Expected boolean" -> green
  - S16-S21 `mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey`, `mask_only_blocker_lets_ctrl_shift_t_reach_the_application`, `mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held`, `mask_only_blocker_masks_only_the_first_event_after_the_match`, `alt_shift_x_fires_next_to_a_mask_only_blocker`, `remapping_off_drops_an_owed_mask`: red "scenario: config does not load: config parse error (<in-memory>): 2:70: Expected boolean" (2:78 for the Alt+Shift+X one) -> green
  - unit tests `binding::tests::display_marks_a_mask_only_blocker`, `blocker_mode_display` (commit `58b71dd`); properties config `mask_only`.
- **KS-35, `when:` inheritance (D7, D8)**: `src/config/loader.rs:338-401` (requests carry the owner's condition; one blocker per hotkey and mode with `any_of_conditions` `:403-422`); `src/engine/mod.rs:448-451` (condition evaluated after `blocker_matches`). Red-first (commit `047da43`):
  - `the_blocker_takes_the_when_of_the_binding_that_asked_for_it`: red "left: Always, right: AppEquals(\"x.exe\")" -> green
  - `blockers_of_one_hotkey_combine_the_when_of_their_owners`: red "two apps: left: [(Some(Swallow), Always)] right: [(Some(Swallow), Or([AppEquals(\"a.exe\"), AppEquals(\"b.exe\")]))]" -> green
  - `a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when`: red "left: [(Some(Swallow), Always), (Some(MaskOnly), Always)]" -> green
  - `a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps`: red "down LShift at +0 ms: expected the event to be passed, it was suppressed" -> green
  - `blockers_act_where_any_owner_when_holds` (context c.exe): same red line -> green
  - `engine::tests::a_blocker_when_condition_is_asked_only_once_its_hotkey_matches`: red panic "the foreground was asked for although no binding with a when: matches this key" -> green

## Remarks

- architecture-critic M1 (KS-12 still stated as current in config/book pages): RESOLVED in `0187cde`: `book/src/config/triggers-conditions.md:135` (§11.8), `book/src/config/example.md:6, 33-39, 234-235`. `build-and-run.md:415, :457-465`, `glossary.md:141, :313`, `architecture.md:268, 271, 544`, audit status lines and the roadmap status are named for the phase book sync (cut.md addendum).
- architecture-critic M2 (Checkpoint-1 item 4): RESOLVED by statement in the cut.md addendum: item 4 passes only with `block_original_combo: MaskOnly` (M4/M5); `results.md` must say so; `roadmap.md:308` rewording goes to the phase book sync. Pinned behaviour with `true`: `shift_pressed_after_tab_was_released_is_still_swallowed`, `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing`.
- Open question 1 (mask in the Alt+Tab switcher): documented as a warning in `book/src/config/matching.md` §15.4; M3 settles it; a failure is a RETHINK at results.
- Open question 2 (D10 wording): RESOLVED at `src/hook/keyboard_hook_callback.rs:60-69` with the critic's wording.
- Open question 3 (H2 fail-safety): the seam is at `src/hook/keyboard_hook_callback.rs:253-277`; reviewer checklist line in the cut.md addendum.
- Open question 4 / cut Q1, Q2: implemented as recommended (owed mask; a mask per swallowed event).

## Deviations from the cut

- The scenario transcript marks masked steps (", masked"): failures of the mask scenarios are readable without a debugger. Small harness addition.
- `with_alt_shift_blocker` and `with_alt_shift_mask_only_blocker` share a private `with_alt_shift_blocker_of(mode, ron)`.
- The `mask_only` property config is appended last, not next to `default`: the script seed mixes in the config index, so inserting it would have changed every later config's generated scripts.
- The D1 rule lives in `Engine::mask_if_swallowed_under_alt_or_win`, which sets `mask = true` only when the rule holds (never clears it), so a `MaskOnly` mask set earlier in the same event survives.
- `blockers_of_one_hotkey_combine_the_when_of_their_owners` uses three configs as the cut lists; L1 also covers a binding without the field.
- README: besides the lines the cut names, the Errors section lists the misspelled-value message and the `MaskOnly` fallback warning. errors.md also lists `RawBlockOriginalCombo` in §18.4 and the `[WARN] SendInput inserted N of 2 menu mask key events` line.
- `triggers-conditions.md` §12.4: the bullet said every condition is checked before the keys are compared, stale since `feat/one-winner` for `keys` bindings; it now says `keys` bindings and blockers ask after the match and `sequence`/`chord` before. The cut asked only for the blocker part.
- `matching.md` §15.4 gets a warning about the mask key in the Alt+Tab switcher (critic open question 1).
- `example.md` intro sentence (line 6) changed with the comment the critic named.
- No new `default_config.ron` example binding (as the cut says).

## Unsafe / atomics added

- `src/hook/keyboard_hook_callback.rs:246-250`: SAFETY: "`inputs` is an array of two fully initialised INPUT_KEYBOARD records on this stack frame, alive for the whole call, and cbsize is the size of one INPUT, as SendInput requires; it only reads the records of the slice it is given."
- `src/hook/keyboard_hook_callback.rs:554-556` (test H1): SAFETY: "every element is built with r#type INPUT_KEYBOARD, so ki is the initialised field of the union."
- No atomics. `Engine.mask_owed` is plain data behind the existing `Mutex<Engine>`; `MASKS_SENT` is a `cfg(test)` `thread_local!` `Cell<u32>`.

## Gates

- `cargo fmt --check`, `cargo check --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings`: pass on every green commit and on HEAD.
- Tests run (the modules the diff touches): `cargo test --locked --lib config::` (100 passed), `engine::` (248 passed, properties included), `data::` (58 passed, 1 ignored, pre-existing), `hook::` (32 passed, 1 ignored: H3). Every red-first was run alone on its parent commit (outputs above). Full suite and `tests/` left to the tester.
- Manual ignore added: `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (`#[ignore = "manual: injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus and check that it passes and nothing is typed and no menu opens"]`).

## Manual checks (human only; release build via `swch open`, `osk.exe` open)

- M1: default config, hotkey Alt+Shift. In Notepad: Alt down, Shift down, Shift up, Alt up, then type `fx`: no menu bar, `fx` typed, layout unchanged.
- M2: `( keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE") )`: Win+Q on Notepad closes it; Start does not open on the Win release.
- M3: default config, three windows: hold Alt, tap Tab, press Shift, tap Tab, release all: the switcher stays open and steps forward (limit of `true`); no menu bar. A failure is a RETHINK at results.
- M4 (v2): CapsLock with `block_original_combo: MaskOnly`, hotkey Ctrl+Shift: in a browser Ctrl+Shift+T reopens a tab; a bare Ctrl+Shift five times never switches the layout; CapsLock still switches it.
- M5 (v2): same with hotkey Alt+Shift: bare Alt+Shift five times never switches; Alt+Shift+Tab steps back; Notepad Alt, Shift, release opens no menu.
- M6: `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("notepad.exe") )`: Alt+Shift in Notepad neither switches nor opens a menu; in another app Windows switches the layout.
- M7: hotkey `` ` `` (value "4") with `MaskOnly`: startup prints the `[WARN] binding #0: block_original_combo: MaskOnly needs ...` line; a lone `` ` `` is swallowed.
- M8: debug build, Notepad focused: `cargo test --lib -- --ignored the_menu_mask_key_is_inserted_into_the_input_stream` passes; nothing typed, no menu.
- M9: after M1-M6, `osk.exe` shows no modifier held.
If M4 or M5 fails, the hypothesis or the injection order is wrong: record it and keep `true` as the default.

## Docs

- `CHANGELOG.md` [Unreleased]: Added `MaskOnly` (KS-05, experimental, manual check); Changed **Breaking:** blocker `when:` inheritance (KS-35) with the `game.exe` example; Fixed KS-12 (after KS-11); edited stale text: the `Tap` on Alt/Win line, the KS-03 entry, the KS-05 entry, the KS-14 entry.
- `README.md`: Binding fields (`block_original_combo`, `auto_blocker`), the adapter section (mask key), "**Win and Alt.**" paragraph in Combo syntax (no new ```ron block), the lone Alt/Win gesture rule, the `block_original_combo` row, Errors (message and warning).
- `src/config/default_config.ron`: combo-syntax sentence on the mask key, the Win+Q example comment, the `block_original_combo: true | false | MaskOnly` flag comment with `when:` inheritance.
- Book `book/src/config/`: `matching.md` (§15.1, §15.3 with a validated example, §15.4, new §15.5, §16.2), `structure.md`, `README.md`, `ron-syntax.md`, `errors.md`, `actions.md`, `triggers-conditions.md`, `known-issues.md` (rows 31, 39, 43, 66), `example.md`. Code links point at `ad7fe86`. New RON examples validated through `config::reload` with a cfgcheck harness in the scratchpad (its methodology version prints `block_original_combo` with `{}`, which no longer compiles now that the field is `Option<BlockerMode>`; the scratch copy uses `{:?}`; `book/src/methodology.md` is the doc-writer's). `mdbook build book -d D:/tmp/ks-orch/p1-1.2-mask-key/book` succeeds; every relative link and anchor of `book/src/config/*.md` resolves (script check).

## Commits

- `54e0673` Engine: red-first: the menu mask decision (1.2, KS-12)
- `82ae3f2` Engine: ask for the menu mask after a swallowed event while Win or Alt is down in Windows (1.2, KS-12)
- `44adabe` Hook: red-first: the menu mask key's inputs and carry-out (1.2, KS-12)
- `11fa3e9` Hook: send the menu mask key before the hook returns (1.2, KS-12)
- `a3e1149` Config: red-first: block_original_combo: MaskOnly (1.2, KS-05)
- `58b71dd` Config: block_original_combo: MaskOnly lets the hotkey through and masks it (1.2, KS-05)
- `047da43` Config: red-first: an auto-blocker follows its binding's when (1.2, KS-35)
- `ad7fe86` Config: an auto-blocker follows the when of the bindings that ask for it (1.2, KS-35)
- `0187cde` Book: the menu mask key, MaskOnly and the blocker's when in the config reference (1.2)

## Open points / pre-existing warnings

- Public API change, as the cut prescribes: `Binding::block_original_combo` is `Option<BlockerMode>` (was `bool`), `with_block_original_combo` takes `Option<BlockerMode>`, `new_auto_blocker` takes a mode. External users of the library (the methodology's cfgcheck) must adapt.
- Phase book sync hand-off: `architecture.md`, `build-and-run.md`, `glossary.md`, `audit.md` status lines (KS-05 partly/opt-in, KS-12, KS-35), roadmap status and Checkpoint-1 item 4 wording, `methodology.md` cfgcheck snippet.
- The book's GitHub links to `ad7fe86` resolve only after the merge step pushes.
- No pre-existing warnings: fmt, check and clippy are clean.
