# Implementation: p0-A4-autoblockers round 1

Verdict: DONE. Branch `fix/auto-blockers` from `master` 44f79ac, HEAD `f5595aa`, tree clean.
Cut: `cut.md` (developer addendum at its end). Critique: `critique.md` (APPROVED, 0 Critical, 0 Important, 5 Minor).

## Items

- D6 seam - `src/config/loader.rs:123-145` (`SystemCombination`, `from_ron_str` delegates to `from_ron_str_with_hotkeys`), `:187-189` (expansion takes the lookup). Green, no behaviour change. Test `auto_blocker_keys_come_from_the_hotkey_lookup` (green on 91937a0 and later).
- KS-18 dedup + order (D4, D5) - `src/config/loader.rs:177-219` (`expand_with_auto_blockers`: distinct hotkeys in first-request order; skipped when any `keys` binding's `combination()` equals the hotkey, requester included; `[INFO] no auto-blocker for {hotkey}: binding #{index} uses the same keys`; blockers appended after the file's bindings), doc `:127-133`. Red-first on cae1ec5, green on bb4dfcd:
  - `auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order`: red "assertion `left == right` failed: flagged binding above / left: 1 / right: 0" -> green
  - `auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys`: red "assertion `left == right` failed / left: 1 / right: 0" -> green
  - `auto_blockers_follow_the_bindings_of_the_file`: red "the file's bindings come first" -> green
  - `user_binding_on_the_language_hotkey_fires_in_either_file_order` (scenarios): red "scenario: down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]" / "context: flagged binding above" -> green
  - `auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey`: pin, green throughout
- KS-05 exact part + KS-18 no-fire (D1, D2, D3) - `src/engine/mod.rs:353-365` (blocker branch first in the `Combo` arm: on match only `suppress = true`, never `fire`, never `immediate_fired`, then `continue`), `src/engine/matching.rs:30-36` (`HeldKeys::all_held`), `:81-101` (`blocker_matches`), `src/engine/keys.rs:204-210` (`KeyState::all_held`). Red-first on 8a08253 (verified on bb4dfcd's code), green on f5595aa:
  - `shift_pressed_while_alt_and_tab_are_held_reaches_windows`: red "down LShift at +500 ms: expected the event to be passed, it was suppressed" -> green
  - `shift_pressed_while_ctrl_and_t_are_held_reaches_windows`, `alt_shift_pressed_while_a_letter_is_held_reaches_windows`: red "down LShift at +0 ms: expected the event to be passed, it was suppressed" -> green
  - `grave_blocker_acts_only_on_a_lone_grave`: red "down ` at +0 ms: expected the event to be passed, it was suppressed" -> green
  - `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing`, `shift_pressed_after_tab_was_released_is_still_swallowed` (M4), `key_whose_release_the_hook_missed_does_not_disable_the_blocker`, `key_swallowed_by_a_binding_does_not_disable_the_blocker`, flips F1 `auto_blocker_matches_its_modifiers_exactly`, F2 `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier`, F3 `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press`: red "down LShift at +0 ms: expected to fire [], fired [DoNothing]" -> green
  - `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides`: red "down RShift at +0 ms: expected to fire [], fired [DoNothing]" -> green
  - `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`: red "down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]" / "context: blocker first" -> green
  - `modifier_swallowed_by_a_binding_does_not_disable_the_blocker` (M3): red "down LShift at +0 ms: expected the event to be suppressed, it was passed" -> green
  - Unit tests added with the fix (cannot compile before it): `blocker_matches_only_what_windows_sees_as_its_hotkey`, `blocker_never_asks_windows_about_its_own_keys` (`matching.rs`), `all_held_visits_every_held_code` (`keys.rs`).
- Exactly the listed tests were red on each red commit: `cargo test --locked --lib -- config:: engine::` gave 4 failed / 209 passed on cae1ec5 and 14 failed / 210 passed on 8a08253, all with the texts above.

## Remarks

- critique M1 (`VIRTUAL_KEY(raw)` inside the engine) - RESOLVED at `src/engine/keys.rs:204-209`: the only construction, `VIRTUAL_KEY(code)` for `code in 0..256u16` filtered by the engine's own `active` table, with the bound stated in the comment (inverse of `slot`). The cut's signature is kept because `KeySpec::accepts` and `Env::os_key_down` take `VIRTUAL_KEY`. Pinned by `all_held_visits_every_held_code`. The reviewer may still rule on principle 7; the construction is not a literal code.
- critique M2 (`'static` trait object in `SystemCombination`) - RESOLVED: both stand-ins own their `KeyCombination` (`move` closure built from `keys` up front) at `src/engine/testing.rs:154-163` and `src/config/loader.rs:632-640`. Commit 2 compiled; its reds were runtime assertions.
- critique M3 (no pin for a swallowed extra modifier) - RESOLVED at `src/engine/matching.rs:261-268` (LCONTROL held, Windows says up -> true; Windows says down -> false) and scenario `modifier_swallowed_by_a_binding_does_not_disable_the_blocker` (`src/engine/scenarios.rs:2247`). The passed extra modifier stays pinned by `auto_blocker_matches_its_modifiers_exactly` (LCtrl passed -> LShift passed).
- critique M4 (switcher wording overstated) - RESOLVED: `CHANGELOG.md:96-106` names the usual switcher order as 1.2 residue; README row `README.md:428` names it in the 1.2 sentence; template comment uses "Shift pressed while Alt and Tab are held". Pinned by `shift_pressed_after_tab_was_released_is_still_swallowed` (`src/engine/scenarios.rs:2106`). Manual M3 gains a residue step (below).
- critique M5 (behaviour change under Fixed) - RESOLVED: `CHANGELOG.md:77-85`, a **Breaking:** entry under Changed (when:-restricted binding on the hotkey's keys; grave typed with another key held).
- critique open question 3 (RK1 text) - accepted; corrected text in the cut addendum (total `GetAsyncKeyState` failure makes the blocker fail open, as before this lane). No code change.
- critique open question 2 (hotkey's own keys counted from the engine table) - not a finding per the critic; behaviour unchanged, belongs to 1.2.

## Deviations from the cut

- T2 script: the cut repeats Tab after Shift went down. A keyboard auto-repeats only the key pressed last (convention fixed in 15e901d), so Tab now repeats before Shift's key-down and Shift repeats after it. Its red label is "+500 ms" instead of "+0 ms"; the failing step and text are the same.
- T9 also presses LWin before L and releases both unseen (models Win+L as written in the cut's rationale).
- Tests added beyond the cut: `modifier_swallowed_by_a_binding_does_not_disable_the_blocker` (M3), `shift_pressed_after_tab_was_released_is_still_swallowed` (M4), the M3 cases in U1, and a generic-slot assertion in U3.
- Stronger checks than the cut lists: `check_press_symmetry` also on T7 and T8; `expect_total_fired` on T8, T9, T10.
- The C1 test uses the `language_hotkey` helper from commit 2 on (the cut allows it).
- CHANGELOG and README texts follow the cut with the M4/M5 changes above; the template comment reads "It acts only on the hotkey pressed on its own: / Shift pressed while Alt and Tab are held still / reaches Windows." instead of "so Alt+Tab, then Shift still reaches Windows" (M4: the cut's wording reads like the switcher order).
- `src/engine/mod.rs:21-22` module doc: "the operating system's view of the modifier keys" -> "of the keys held" (D2 now asks about non-modifiers too). Not in the touch set; doc only.

## Unsafe / atomics added

- None. No `unsafe`, no atomics, no locks, no new Win32 API. The blocker branch calls the existing `Env::os_key_down` (`HookEnv`, `GetAsyncKeyState`) under the existing `with_engine` lock, only for held keys outside the hotkey, only after `is_trigger_key` and `keys_down` pass. No allocation on the path (`all_held` iterates the table).

## Gates

- fmt --check / check --all-targets / clippy --all-targets -D warnings: pass on every commit (91937a0, cae1ec5, bb4dfcd, 8a08253, f5595aa).
- tests run: `cargo test --locked --lib -- config:: engine:: data::` on f5595aa: 271 passed, 0 failed (includes both `engine::properties` tests and every lane test by name); per-commit runs of `config:: engine::` as listed under Items. Full suite left to the tester.
- manual ignores added: none (no test needs a live hook, injection or a registry write; D6 keeps every new test registry-free).

## Manual checks (human only, from the cut, with the M4 addition)

- M1 default config, hotkey Alt+Shift, Notepad: Alt down, Shift down, release both; Esc; CapsLock. Layout unchanged by Alt+Shift; CapsLock switches once; nothing stuck in osk.
- M2 flagged `CapsLock` + `( keys: "Alt+Shift", action: SwitchLanguageBackward )` below it: banner has no `[AUTO-BLOCK]` line and prints `[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys`; swap the entries and save: `#0` on reload; every Alt+Shift switches backward once.
- M3 default config, hotkey Alt+Shift, three windows: hold Alt, hold Tab until the switcher cycles, press Shift while Tab is held, release Tab, tap Tab, release Shift, release Alt: the switcher steps backward after Shift; nothing stuck. Added (M4, expected residue until 1.2): hold Alt, tap Tab, then press Shift and tap Tab: the switcher moves forward (Shift swallowed); nothing stuck in osk.
- M4 hotkey set to Ctrl+Shift by the owner, a browser: (a) Ctrl, T held, Shift: every key reaches the browser; (b) Ctrl, Shift, T: a new tab (Ctrl+T, 1.2 residue); (c) bare Ctrl+Shift: layout unchanged.
- M5 default config: Win+L, unlock, immediately Alt+Shift without typing `l`: layout unchanged.

## Docs

- `README.md:428` (`block_original_combo` row): blocker wording, exactness, dedup, no fire, listed last, 1.2 residue incl. the switcher order.
- `src/config/default_config.ron` flag comment: dedup sentence (C3) and exactness sentence (C5), ASCII.
- `CHANGELOG.md` Unreleased: Changed **Breaking:** entry (KS-18 `when:` consequence, KS-05 grave); Fixed KS-05 (partly) and KS-18 (+ no-fire sentence).
- Code docs: loader (`from_ron_str`, `expand_with_auto_blockers`, `parse_simple_binding` comment), engine rule 4 and module doc, `matching.rs` module doc, `keys.rs` module doc, `env.rs` `os_key_down` contract, `binding.rs` `allow_extra_modifiers` and `is_auto_blocker`, `with_alt_shift_blocker`, `properties.rs` config doc, `scenarios.rs` KS-10 sentence dropped.
- Book: none (phase book sync, cut Q5; page list in the cut's "Docs impact").

## Commits

- 91937a0 Config: auto-blocker expansion takes the system hotkey lookup as a parameter
- cae1ec5 Tests: red-first: auto-blocker dedup over the whole binding list (KS-18) (RED, declared)
- bb4dfcd Config: an auto-blocker is skipped when any binding uses the hotkey's keys, and follows the file's bindings (KS-18)
- 8a08253 Tests: red-first: auto-blockers act only when Windows sees exactly the hotkey, and fire nothing (KS-05, KS-18) (RED, declared)
- f5595aa Engine: an auto-blocker matches only the hotkey on its own and never takes a user binding's fire (KS-05, KS-18)

## Open points / pre-existing warnings

- Pre-existing machine dependency: loader tests that load `block_original_combo` through `from_ron_str` (`parse_simple_binding`, `auto_blocker_deduped_when_two_bindings_want_the_same_combo`, `parse_tap_trigger`, `default_config_parses`) still read `HKCU\Keyboard Layout\Toggle`; with hotkey "3" (KS-17) the first two would fail. They could move to `from_ron_str_with_hotkeys`; not in the cut.
- Pre-existing rustdoc warning (not a gate): `src/data/vk_name.rs:4` "unclosed HTML tag `raw`".
- `src/engine/matching.rs` came out of `cargo fmt` with LF endings in the working copy; it was put back to CRLF like the other files (index content is LF either way).
- Agent-file rule texts (`code-reviewer.md:124`, `project-analyst.md:60`: "auto-blockers rely on load order among equal counts") are stale after this lane (cut Q6, orchestrator).
