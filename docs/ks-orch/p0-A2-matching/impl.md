# Implementation: p0-A2-matching round 1

Verdict: DONE. Branch `fix/matching` from `master` `e67e5cb`, head `c609a69`, 10 commits, tree clean.
Cut: `cut.md` C1-C6 as written; C7-C9 added to resolve critique I1/I2 in code (see Deviations and the developer
addendum at the end of `cut.md`).

## Items

- **KS-07 sided keys** (C1 `1bc3b0e` red, C2 `709d739` fix)
  - `src/data/key_spec.rs` (new): `Modifier` (`:21`), `KeySpec` (`:124`, `accepts`/`modifier`/`overlaps`/`parse` `:165`,
    `From<VIRTUAL_KEY>` `:173`, `Display` `:184`); `src/data/vk_name.rs:178` `normalize_key_name`.
  - `src/data/key_combination.rs`: `keys: Vec<KeySpec>`, generic `new`/`with`/`from_keys`, `Display`; `matches` and its 3 tests removed.
  - `src/engine/matching.rs` (new): `HeldKeys` `:22`, `keys_down` `:37`; `src/engine/keys.rs:117` `impl HeldKeys for KeyState`;
    `ActiveKeys`/`ACTIVE_KEYS_CAP`/`normalize`/`active_keys` and their 3 tests removed.
  - `src/engine/chords.rs:62-121` (`&[KeySpec]`, set-equality `same_keys`, `seen` codes, `Completed` = pressed codes);
    `src/engine/sequences.rs:57-145` (`HeldKeys`, `keys_down`, `last_step_keys` non-modifier only).
  - `src/config/parsing.rs:15-51` (`parse_combo` on `KeySpec`, `parse_key_list`, shared `parse_segments`);
    `src/config/loader.rs:299-320` (chord `KeySpec`, overlap error), `:445` (macro `Combo` via `parse_key_list`).
  - `src/data/binding.rs` (`Chord.keys: Vec<KeySpec>`, `format_kind`); `src/system/registry.rs:56` (INFO `{combo}`),
    `:74-75`, `:204` (`win_with`: `AnySide(Win)` + key); `src/hook/keyboard_hook.rs` tests (type only).
  - red-first (C1, on `e67e5cb`):
    - `lshift_in_keys_fires_only_for_the_left_shift`: red "down F13 at +0 ms: expected the event to be suppressed, it was passed" -> green
    - `rwin_and_lwin_in_keys_fire_only_for_their_side`: red "down F13 at +0 ms: expected the event to be suppressed, it was passed" -> green
    - `lshift_double_tap_fires_only_for_the_left_shift`: red "down LShift at +0 ms: expected the event to be suppressed, it was passed" -> green
    - `unsided_shift_in_a_chord_fires_for_either_side`: red "down LShift at +0 ms: expected the event to be suppressed, it was passed" -> green
    - `sided_shift_in_a_sequence_step_matches_only_its_side`: red "down F14 at +0 ms: expected the event to be suppressed, it was passed" -> green
    - `win_in_the_last_sequence_step_does_not_swallow_its_key_up`: red "up Win at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `config::loader::tests::chord_keys_that_overlap_are_rejected`: red "overlapping chord keys must be rejected: [Binding { kind: Chord { keys: [VIRTUAL_KEY(16), VIRTUAL_KEY(160)] ..." -> green
    - guards green throughout: `unsided_shift_and_win_in_keys_fire_for_either_side`, `sided_shift_in_a_chord_is_only_its_side`
- **KS-04 trigger-key rule** (C3 `46424a1` red, C4 `2271db2` fix)
  - `src/engine/matching.rs:46` `is_trigger_key`, `:58` `combo_matches`; `src/engine/mod.rs:320` Combo branch.
  - red-first (C3, against C2):
    - `key_pressed_while_an_immediate_binding_key_is_held_passes`: red "down A at +30 ms: expected the event to be passed, it was suppressed" -> green
    - `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`: red "down A at +50 ms: expected the event to be passed, it was suppressed" -> green
    - `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock`: red "down LShift at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `modifier_pressed_last_does_not_fire_a_combo`: red "down LCtrl at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `deferred_combo_arms_only_on_its_trigger_key`: red "down LShift at +350 ms: expected the event to be passed, it was suppressed" -> green
- **Exact modifiers + `allow_extra_modifiers`** (owner decision; C5 `7b65609` red, C6 `cffb5eb` fix)
  - `src/data/key_spec.rs:95` `ModifierSet`; `src/data/key_combination.rs:63` `modifiers()`; `src/engine/matching.rs:28`
    `HeldKeys::modifiers`, `:58-68` exact rule; `src/data/binding.rs:70` field, `:140` builder, `:164` Display suffix;
    `src/config/schema.rs:62` `RawBinding.allow_extra_modifiers`; `src/config/loader.rs:229-237` reject on sequence/chord;
    `src/hook/keyboard_hook_callback.rs` tests opt out (D10).
  - red-first (C5, against C4):
    - `capslock_binding_does_not_fire_while_shift_is_held`: red "down CapsLock at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `ctrl_j_does_not_fire_on_ctrl_shift_j`: red "down J at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `auto_blocker_matches_its_modifiers_exactly`: red (second LShift, Ctrl+Alt held) "down LShift at +0 ms: expected the event to be passed, it was suppressed" -> green
    - `config::loader::tests::allow_extra_modifiers_is_only_for_keys_bindings`: red "allow_extra_modifiers on a sequence/chord must be rejected: [Binding { kind: Sequence { ..." -> green
    - guards green throughout: `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held`, `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j`,
      `either_shift_counts_as_the_same_modifier`, `shift_capslock_fires_toggle_capslock_with_either_shift`
- **KS-06 / critique I1, I2: a repeat follows its key-down** (C7 `f4b2582` red, C8 `faf3be1` red, C9 `20672b7` fix)
  - `src/engine/mod.rs:427-443`: repeat -> `suppress = is_blocked(vk)`; first key-down suppressed -> `mark_blocked`;
    first key-down passed -> `take_blocked` (clears a stale bit). `src/engine/keys.rs:95` `is_blocked`.
  - red-first (C7, against C6):
    - `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (I2): red "repeat CapsLock at +533 ms: expected the event to be passed, it was suppressed" -> green
    - `double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too` (I1): red "repeat LShift at +500 ms: expected the event to be suppressed, it was passed" -> green
    - `chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too` (I1): red "repeat LShift at +500 ms: expected the event to be suppressed, it was passed" -> green
    - `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` (KS-06): red "repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed" -> green
    - `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once` (KS-06): red "repeat L at +500 ms: expected the event to be suppressed, it was passed" -> green
  - red-first (C8, against C7): `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck`: red "up LShift at +533 ms:
    expected the event to be passed, it was suppressed"; with the repeat rule alone still red at "repeat LShift at +533 ms:
    expected the event to be passed, it was suppressed"; green with the clearing clause.

Unit tests added: `data::key_spec` (11: 10 + `modifier_set_holds_families`), `data::key_combination` (5, replacing the 3 `matches`
tests), `data::binding::display_marks_allow_extra_modifiers`, `config::parsing` (3), `config::loader` (6 incl. the 2 red-firsts and
`readme_matching_examples_load`), `engine::keys` (3: either side/generic slot, sided, `is_blocked`), `engine::matching` (8),
`engine::chords` (2), `engine::sequences::sided_step_needs_its_side`, `system::registry::language_hotkeys_are_either_side_modifiers`.
Scenarios added: 26 (sections "Sided keys (KS-07)", "Trigger key (KS-04)", "Exact modifiers", "Auto-repeat follows its key-down (KS-06)").

## Remarks

- I1 - RESOLVED at `src/engine/mod.rs:427-436` (repeat suppressed iff its key is marked blocked), pinned by
  `double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too`, `chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too`.
- I2 - RESOLVED at `src/engine/mod.rs:427-443` (repeat rule + a passing first key-down clears its bit), pinned by
  `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (the critic's scenario) and
  `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck`.
- M1 - RESOLVED: manual check M2 below is bounded (< 0.4 s), like M1.
- M2 - RESOLVED: no `#[cfg(test)]` `HeldKeys` impls; `engine::matching` and `engine::sequences` tests build a real `KeyState`
  (`held()` helpers), so the production `impl HeldKeys for KeyState` is under test.
- M3 - RESOLVED at `src/config/loader.rs:1307-1336`: `include_str!("../../README.md")`, every RON block of the "Combo syntax"
  section is loaded; mutation-checked (README example changed to `LShift+F99` -> "README example does not load", reverted -> green).
- M4 - RESOLVED: README features line "Hot-path allocations: none" replaced by "Table-driven matching ... no list of held keys
  is built per key press" (no allocation claim).
- Critique open questions: book (Q2) not edited; optional 3.5 check before `condition.evaluate` not done; lane order: KS-06
  now lands with this lane, KS-14 stays with its lane. Recorded in the `cut.md` addendum.

## Deviations from the cut

1. **C7-C9 added** (I1/I2 in code, option I2(a)). The rule is the audit's KS-06 suggested fix plus I2's "a repeat never
   suppresses when its key-down passed". Alternative considered: option (b), docs warnings + handing a known-defect scenario to
   a later lane; rejected because both paths are opened by this lane (sided keys, exact matching) and break the suppression
   invariant. Effect: CHANGELOG lists KS-06 under Fixed.
2. Repeats still fire `Immediate` actions as before (KS-10, roadmap 1.1). On the I2 path a repeat that starts matching after
   its key-down passed fires the action while the repeat reaches Windows; not pinned.
3. `src/system/registry.rs:204` private helper `win_with` for the three either-Win combos instead of three inline constructions.
4. `KeyState::is_active` lost `#[cfg(test)]` and backs `impl HeldKeys` (no new accessor).
5. `keys.rs::out_of_range_codes_are_ignored`: its `active_keys()` assertion (API removed) is replaced by `is_active`/`is_down`
   checks on code 300. `ctrl_x_ctrl_s_sequence` passes `VK_LCONTROL` (a physical code) and asserts `last_step_keys == [S]` (D6).
6. Extra unit tests beyond the cut list: `modifier_families`, `display_round_trips_through_parse`, `modifier_set_holds_families`,
   `language_hotkeys_are_either_side_modifiers`; `parse_key_list_keeps_output_codes` also checks the error texts.
7. README "Exact modifiers" says that sequence steps and chords match whatever modifiers are held (D6 made explicit).
8. Comment of the existing `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` updated (its repeat is now pinned
   elsewhere); the test itself is unchanged.
9. Extra commit `c609a69` reflows one CHANGELOG paragraph.

## Unsafe / atomics added

- None. `#![forbid(unsafe_code)]` kept in `src/engine`. No new `static`, atomic, lock, thread or Win32 call.
- Hot path: per key-down per Combo binding at most keys x 3 table reads, then 12 reads for the held modifier families only after
  the trigger and key checks pass; the per-event 256-slot scan, the 16-key buffer and the chord `canonical` Vec are gone.

## Gates

- `cargo fmt --check`, `cargo check --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings`: pass at every
  commit; `cargo doc --no-deps`: only the pre-existing `src/data/vk_name.rs:4` "unclosed HTML tag `raw`" warning.
- Tests run: every red-first by name on its parent (outputs above) and on its fix; `cargo test --locked --lib -- engine:: config::
  data:: system:: hook::` at every fix commit (final: 238 passed, 0 failed, 16 filtered out). Full suite left to the tester.
- Manual ignores added: none (the cut lists none; nothing here needs a live hook beyond the human checks below).

## Manual checks (human only; release build, `swch open`, Notepad in front; check stuck keys in `osk.exe`)

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | default `config.ron` | press CapsLock and, still holding it, type `abc` quickly (< 0.4 s) | `abc` appears; the language switched once |
| M2 | default `config.ron` | within 0.4 s each: Shift+CapsLock with each Shift; CapsLock, then Shift while CapsLock is held; Ctrl+CapsLock | CapsLock toggles; second: language switches once, CapsLock state unchanged, Shift+A then types "A"; Ctrl+CapsLock reaches Windows |
| M3 | add `( keys: "Ctrl+J", action: DoNothing )`, Chrome or Edge | Ctrl+J; Ctrl+Shift+J | Ctrl+J swallowed; Ctrl+Shift+J opens the DevTools console |
| M4 | uncomment the `keys: "LShift"` `DoubleTap(250)` example | double-tap left Shift; double-tap right Shift; hold left Shift ~1 s and release | left toggles CapsLock, right does not; after the hold no Shift is stuck (left Shift does not type capitals until 1.3: expected) |
| M5 | keyboard with a right Win key; `( keys: "RWin+E", action: DoNothing )` | RWin+E; LWin+E | RWin+E swallowed; LWin+E opens Explorer (Start may open on RWin release: KS-12) |
| M6 | `( chord: (keys: ["Shift", "Space"], window_ms: 50), action: ToggleCapsLock )` | LShift+Space together; RShift+Space together; hold Shift alone ~1 s | CapsLock toggles each time; after the hold no Shift is stuck; remove the binding afterwards |
| M7 | AltGr layout (German or Polish), `( keys: "Alt+X", action: DoNothing )` | LAlt+X; AltGr+X | LAlt+X swallowed; AltGr+X passes (the fake LCtrl is an extra modifier) |
| M8 | README `Tap(200)`/`Hold(200)` CapsLock pair | hold CapsLock 2 s, release | Hold fires once; CapsLock not held in `osk.exe` after release (KS-06) |
| M9 | default `config.ron` | hold LCtrl, press CapsLock, keep both ~1 s, release LCtrl, then CapsLock | CapsLock not held in `osk.exe` afterwards; the next plain CapsLock switches the language (the language may also switch on repeats after Ctrl's release: KS-10, 1.1) |
| M10 | default `config.ron`, Windows language hotkey Alt+Shift | hold LAlt, press LShift, release LAlt, keep LShift ~1 s, release | Shift not held in `osk.exe` afterwards |

## Docs

- README: layout tree (`key_spec.rs`, `matching.rs`, `keys.rs` text), `key_combination.rs` bullets, modifier row, "Sided modifiers",
  "Which key fires a binding", "Exact modifiers" (RON block + AltGr note), kinds table "Fires when", triggers key-down bullet,
  LShift example warning, behaviour rule on auto-repeat, chord overlap restriction, flags-table row `allow_extra_modifiers`,
  features (exact sided modifiers; table-driven matching).
- `src/config/default_config.ron`: modifier list, trigger-key and exact-modifier paragraphs, `allow_extra_modifiers` flag entry,
  LShift example warning.
- CHANGELOG `[Unreleased]`: Added (`allow_extra_modifiers`); Changed (Breaking: sided names; INFO line; Breaking: exact
  modifiers); Fixed (KS-04, KS-06, KS-07, KS-32 partly).
- Book: not edited (cut D12/Q2). The phase book sync list in the cut's Docs impact applies, plus KS-06: `audit.md` status line,
  `config/known-issues.md` rows for KS-06, `roadmap.md` A bullet "Suppresses repeats symmetrically" and acceptance line, and
  `architecture.md` key-event path (repeat rule).

## Commits

- `1bc3b0e` Engine: red-first: sided keys in keys, sequences and chords (KS-07)
- `709d739` Matching: sided and unsided key specs in every trigger (KS-07)
- `46424a1` Engine: red-first: a combo fires only on its own key (KS-04)
- `2271db2` Matching: trigger-key rule for combos and deferred triggers (KS-04)
- `7b65609` Engine: red-first: exact modifier matching
- `cffb5eb` Matching: exact modifiers with per-binding allow_extra_modifiers
- `f4b2582` Engine: red-first: an auto-repeat follows its key-down (KS-06)
- `faf3be1` Engine: red-first: a key-down that reaches the OS clears its key-up block (KS-06)
- `20672b7` Engine: an auto-repeat follows its key-down's suppression (KS-06)
- `c609a69` Docs: reflow the sided-names CHANGELOG entry

## Open points / pre-existing warnings

- Pre-existing rustdoc warning `src/data/vk_name.rs:4` (unclosed `<raw>`); untouched.
- Orchestrator: book sync (Q2); CLAUDE.md "Layout" (`src/data/` list lacks `key_spec.rs`; `src/engine/` lacks `matching.rs`) and the
  code-reviewer's R8 text still describe folding and `VK_LWIN` as "any Win" (Q4).
- KS-10 interplay (deviation 2) and KS-14 (a swallowed modifier is erased by the resync and then counts as not held for exact
  matching) stay with roadmap 1.1 and the KS-14 lane.
- Sequences: a non-modifier key of the last step whose key-down passed before the completing key (`"A+S"`, A held first) is
  still marked blocked (KS-32 rest, roadmap 3.2); unchanged.
- `#[allow(dead_code)]` on `KeyCombination::with` and `Binding::new` etc. is vestigial (public API of a lib); untouched.
- Working-copy line endings: some edited files are LF in the work tree; `core.autocrlf=true` stores LF, `git status` is clean.
