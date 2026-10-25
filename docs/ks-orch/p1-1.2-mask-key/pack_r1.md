# Context pack r1: p1-1.2-mask-key (roadmap 1.2: KS-12, rest of KS-05, KS-35)

Readers: tester, code-reviewer. Facts only; inferred items are marked UNVERIFIED. Sources: the tree at `feat/mask-key`, `D:/tmp/ks-orch/p1-1.2-mask-key/cut.md`, `D:/tmp/ks-orch/p1-1.2-mask-key/impl.md`.

## Scope and delta

- Branch read: `feat/mask-key`, HEAD `0187cde`; `master` = `ece1767`. `git status --short` empty (tree clean).
- `git log --oneline master..feat/mask-key` (oldest last):
  - `0187cde` Book: the menu mask key, MaskOnly and the blocker's when in the config reference (1.2)
  - `ad7fe86` Config: an auto-blocker follows the when of the bindings that ask for it (1.2, KS-35)  [green]
  - `047da43` Config: red-first: an auto-blocker follows its binding's when (1.2, KS-35)  [declared red]
  - `58b71dd` Config: block_original_combo: MaskOnly lets the hotkey through and masks it (1.2, KS-05)  [green]
  - `a3e1149` Config: red-first: block_original_combo: MaskOnly (1.2, KS-05)  [declared red]
  - `11fa3e9` Hook: send the menu mask key before the hook returns (1.2, KS-12)  [green]
  - `44adabe` Hook: red-first: the menu mask key's inputs and carry-out (1.2, KS-12)  [declared red, compile]
  - `82ae3f2` Engine: ask for the menu mask after a swallowed event while Win or Alt is down in Windows (1.2, KS-12)  [green]
  - `54e0673` Engine: red-first: the menu mask decision (1.2, KS-12)  [declared red]
- `git diff --stat master...feat/mask-key`: 24 files, +1749 / -236.
  - Code: `src/engine/scenarios.rs` (+639), `src/config/loader.rs` (319), `src/hook/keyboard_hook_callback.rs` (161), `src/engine/mod.rs` (136), `src/engine/testing.rs` (123), `src/data/binding.rs` (108), `src/engine/keys.rs` (85), `src/engine/properties.rs` (68), `src/config/schema.rs` (26), `src/core/constants.rs` (16), `src/engine/matching.rs` (7), `src/engine/winner.rs` (2).
  - Docs/config text: `CHANGELOG.md` (101), `book/src/config/matching.md` (84), `README.md` (33), `src/config/default_config.ron` (25), `book/src/config/example.md` (16), `actions.md`, `known-issues.md`, `triggers-conditions.md` (8 each), `errors.md` (6), `README.md`/`ron-syntax.md`/`structure.md` under `book/src/config` (2 each).
- Not in the delta: `Cargo.toml`, `Cargo.lock`, `src/hook/keyboard_hook.rs`, `src/system/*`, `tests/`, `book/src/{architecture,audit,roadmap,overview,glossary,build-and-run,methodology}.md`.

## File map (HEAD line numbers)

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/core/constants.rs:34-48` | `MENU_MASK_VK: u16 = 0xE8` + doc | D3; R2 (tagged with the sentinel) |
| `src/engine/keys.rs:39-42` | `MENU_KEYS = [VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN]` | D1 key set (generic `VK_MENU` included) |
| `src/engine/keys.rs:96-111` | `sync_modifiers` (unchanged): skips the event key and swallowed modifiers | D1 relies on it for `active` == OS state |
| `src/engine/keys.rs:122-135` | `reset_from_os` (unchanged): held keys recorded `Passed` | S9 (Win held through `swch on`) |
| `src/engine/keys.rs:139-144` | `key_down_swallowed`: `Suppressed` or `Forgotten` | what "did not reach the OS" means |
| `src/engine/keys.rs:229-241` | `alt_or_win_down_in_windows()` | the D1 predicate |
| `src/engine/keys.rs:602-667` | `Down` test env + unit test `alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed` | unit proof of D1 |
| `src/engine/mod.rs:146-158` | module doc, rule 4 auto-blocker bullet (when after match, Swallow/MaskOnly) | doc vs code |
| `src/engine/mod.rs:184-192` | module doc, mask rule paragraph | doc vs code |
| `src/engine/mod.rs:286-301` | `Decision` (fields `suppress`, `fire`, `timers`, `mask`), `mask` doc 294-300 | spec field |
| `src/engine/mod.rs:314-322` | `Engine.mask_owed: bool` | D5 state |
| `src/engine/mod.rs:343-362` | `on_key`: `mem::take(mask_owed)` at 348, OR at 360 | D5 owed mask |
| `src/engine/mod.rs:366-372` | `on_timer`: never touches `mask` | timer never masks |
| `src/engine/mod.rs:393-396` | `on_foreground_changed`: leaves `mask_owed` | D5 |
| `src/engine/mod.rs:414-420` | `reset`: `mask_owed = false` at 418 | S21 |
| `src/engine/mod.rs:436-467` | blocker branch: `blocker_matches(..) && condition.evaluate(..)` 449-451; `Swallow` 454; `MaskOnly if !was_repeat` 459-462; repeat no-op 463 | D5, D8 |
| `src/engine/mod.rs:580-615` | key-down record update, then `mask_if_swallowed_under_alt_or_win` at 613 | D1 ordering |
| `src/engine/mod.rs:617-644` | key-up: `release`/`take_blocked`, then mask at 642 | D1 on key-ups |
| `src/engine/mod.rs:646-654` | `mask_if_swallowed_under_alt_or_win`: sets `mask = true` only, never clears | D1 |
| `src/engine/mod.rs:685-750` | `ForegroundForbiddenEnv` (685), `a_when_condition_is_asked_only_for_a_binding_whose_keys_match` (703), E1 `a_blocker_when_condition_is_asked_only_once_its_hotkey_matches` (727) | D8 / KS-08 |
| `src/engine/matching.rs:81-92` | `blocker_matches` doc (code unchanged) | D5 |
| `src/engine/winner.rs:64` | `binding.is_auto_blocker()` skip in election | D6 |
| `src/engine/testing.rs:26-40` | harness module doc for `expect_masked`/`check_mask_rule` | D9 |
| `src/engine/testing.rs:130` | `ALT_AND_WIN = [VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN]` (oracle set, no generic `VK_MENU`) | oracle definition |
| `src/engine/testing.rs:137-139, 166` | `Step.masked`, `check_mask` flag | D9 |
| `src/engine/testing.rs:187` | `from_ron_with_language_hotkey(text, keys)` | used by S16-S21, S13, S14 |
| `src/engine/testing.rs:248-256` | `check_mask_rule()` | D9 |
| `src/engine/testing.rs:405-416` | `advance_ms`: `assert!(!decision.mask, "a timer has no event to mask")` at 415 | S12 |
| `src/engine/testing.rs:460-471` | `expect_masked` / `expect_unmasked` | D9 |
| `src/engine/testing.rs:528-556` | key step: `masked = decision.mask` 536; OS model updated only if not suppressed; `check_mask_of` at 553 | oracle timing |
| `src/engine/testing.rs:558-578` | `check_mask_of`: `want = suppressed && ALT_AND_WIN any os_down` | oracle |
| `src/engine/testing.rs:696-724` | transcript ", masked" marker | deviation noted in impl |
| `src/engine/properties.rs:10-20, 41-44, 80-83` | doc, imports, `Config.blocker: Option<BlockerMode>` | D9 |
| `src/engine/properties.rs:169-174` | `mask_only` config appended last | seed stability (impl deviation) |
| `src/engine/properties.rs:242-262` | `run`: blocker by mode 244-249; `check_mask_rule()` unless `MaskOnly` 257-261 | D9 |
| `src/engine/properties.rs:290-302` | the three property tests | red-first targets |
| `src/engine/scenarios.rs:32-48` | `with_alt_shift_blocker`, `with_alt_shift_mask_only_blocker`, private `with_alt_shift_blocker_of(mode, ron)` | harness |
| `src/engine/scenarios.rs:1220, 2591, 2618, 2649` | pre-existing blocker scenarios touched (call sites, `.expect_masked()`/`.expect_unmasked()` added) | KS-05 with `true` pinned |
| `src/engine/scenarios.rs:3897-4256` | section "Menu mask key (KS-12)": S1-S12 | acceptance line 1 |
| `src/engine/scenarios.rs:4258-4417` | section "MaskOnly (KS-05)": `MASK_ONLY_CONFIG` 4262-4265, S16-S21 | acceptance line 2 |
| `src/engine/scenarios.rs:4419-4478` | section "auto-blocker's when (KS-35)": S13, S14 | acceptance line 3 |
| `src/hook/keyboard_hook_callback.rs:20-27` | module doc step 6 (mask first, then timers, then actions) | D10 |
| `src/hook/keyboard_hook_callback.rs:58-82` | Re-entry doc (SendInput re-enters the hook) | D10, R3 |
| `src/hook/keyboard_hook_callback.rs:173-196` | `decide_event`: sentinel check 179 before `with_engine` 191 | R1/R2 ordering for re-entered E8 events |
| `src/hook/keyboard_hook_callback.rs:198-220` | `carry_out`: destructures `Decision` without `..` (205-210); `send_menu_mask()` first (211-213), timers 214-216, actions 217-219 | D2 |
| `src/hook/keyboard_hook_callback.rs:222-239` | `menu_mask_inputs() -> [INPUT; 2]` | D2; R2 |
| `src/hook/keyboard_hook_callback.rs:241-251` | `inject_menu_mask()`: `unsafe SendInput` with SAFETY 246-249 | new `unsafe` |
| `src/hook/keyboard_hook_callback.rs:253-263` | `#[cfg(not(test))] send_menu_mask`: short-count `eprintln!` 260-262 | R1 (I/O on short count only) |
| `src/hook/keyboard_hook_callback.rs:265-277` | `#[cfg(test)] send_menu_mask` (counter only) + `thread_local! MASKS_SENT` | safety seam |
| `src/hook/keyboard_hook_callback.rs:286-300` | `start_hold_timer`: `carry_out(decision)` on `hold-timer` thread at 293 (unchanged) | mask path off the hook thread only if a timer decision masks |
| `src/hook/keyboard_hook_callback.rs:315-350` | `keyboard_hook_callback`: `carry_out` at 341 before `LRESULT(1)` at 343 / `CallNextHookEx` at 349 | "before it returns" |
| `src/hook/keyboard_hook_callback.rs:486-547` | `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` (drives `carry_out` with live `HookEnv`) | why the seam exists |
| `src/hook/keyboard_hook_callback.rs:549-586` | H1 (550), H2 (570), H3 ignored (583-586) | adapter proofs |
| `src/config/schema.rs:51-56` | `RawBinding.block_original_combo: RawBlockOriginalCombo`, `#[serde(default)]` | D4 |
| `src/config/schema.rs:87-106` | `RawBlockOriginalCombo { #[serde(rename="true")] On, #[default] #[serde(rename="false")] Off, MaskOnly }` | D4 |
| `src/data/binding.rs:69-89` | `pub enum BlockerMode { Swallow, MaskOnly }` + `Display` ("swallow", "mask only") | R6 |
| `src/data/binding.rs:100-103, 129-132` | `pub block_original_combo: Option<BlockerMode>`; `pub(crate) auto_blocker: Option<BlockerMode>` | D6 (public API change) |
| `src/data/binding.rs:156-176` | `new_auto_blocker(combination, mode)` (condition `Always`), `is_auto_blocker()` | R5 |
| `src/data/binding.rs:207-209` | `duplicates` uses `is_auto_blocker()` | D6 |
| `src/data/binding.rs:221-223` | `with_block_original_combo(Option<BlockerMode>)` | R5 |
| `src/data/binding.rs:283-292` | blocker `Display` arm: `(blocked)` / `(masked, experimental)` | R6 |
| `src/data/binding.rs:356-390` | tests `display_marks_a_mask_only_blocker` (362), `blocker_mode_display` (387) | D6 |
| `src/config/loader.rs:236-237` | warnings printed as `[WARN] {warning}` (in `reload`) | where the MaskOnly fallback warning surfaces |
| `src/config/loader.rs:292-294` | `parse_ron_str` threads `&mut warnings` into `expand_with_auto_blockers` | D7 |
| `src/config/loader.rs:321-401` | `expand_with_auto_blockers`: requests 345-362 (MaskOnly fallback + warning 353-360), KS-18 check 373-379, per-mode blockers with OR'ed conditions 380-395, appended after file bindings 398-400 | D5, D7, R7 |
| `src/config/loader.rs:403-422` | `any_of_conditions` | D7 |
| `src/config/loader.rs:500` | `.with_block_original_combo(raw_blocker_to_mode(..))` | D4 mapping |
| `src/config/loader.rs:602-608` | `raw_blocker_to_mode` | D4 mapping |
| `src/config/loader.rs:1123-1280` | L1, L2, L4, L7, L8, L9, L5 tests | acceptance |
| `src/config/loader.rs:2193, 2505` | `readme_matching_examples_load`, `the_template_and_its_examples_have_no_duplicate_bindings` | docs-backed tests |
| `README.md:101, 106, 119-120, 262-271, 455-458, 539, 613-615, 635-638` | fields, adapter mask bullet, "Win and Alt" paragraph, lone Alt/Win gesture rule, `block_original_combo` row, Errors message + warning | docs acceptance |
| `src/config/default_config.ron:76-80, 165-189, 239-240` | mask sentence, `block_original_combo: true \| false \| MaskOnly` comment (when-inheritance 187-189), Win+Q comment | docs acceptance |
| `CHANGELOG.md:65-88` (Added MaskOnly), `:182-183`, `:222-236` (Changed **Breaking** KS-35), `:285-289`, `:302-314`, `:356-375` (Fixed KS-12), `:409-410` | Keep a Changelog entries | docs acceptance |
| `book/src/config/matching.md:3-40, 61, 80-98, 103-116, 118-153, 179-181` | §15.1, §15.3, §15.4 (Alt+Tab switcher warning at 109), §15.5 `MaskOnly` + manual check, §16.2 | docs acceptance |
| `book/src/config/known-issues.md:75, 89, 93, 134` | rows 31 (Fixed), 39 (Partly fixed / MaskOnly), 43 (Fixed for when, by design for trigger), 66 (open, not covered) | docs |
| `book/src/config/{example.md:6,33-39,234-235; errors.md:84,91,122,180,193; actions.md:62-63,254-255; triggers-conditions.md:135,218-222; structure.md:150; README.md:42; ron-syntax.md:93}` | remaining config pages | docs |

## Facts

1. `Decision::mask` is set in exactly three places: `src/engine/mod.rs:360` (owed mask OR), `:460` (MaskOnly match), `:652` (D1 rule). `on_timer` (`:366-372`) builds `Decision::default()` and pushes only `fire`.
2. D1 rule: `mask_if_swallowed_under_alt_or_win` sets `mask = true` iff `decision.suppress && self.keys.alt_or_win_down_in_windows()` (`src/engine/mod.rs:650-654`); it is called after the record update on key-down (`:613`, after `mark_blocked`/`mark_passed` at `:598-612`) and on key-up (`:642`, after `release` `:619` and `take_blocked` `:639`).
3. `alt_or_win_down_in_windows` returns true if any of `MENU_KEYS` has `active[index] && !key_down_swallowed(index)` (`src/engine/keys.rs:236-241`); `key_down_swallowed` is `Suppressed | Forgotten` (`:139-144`). `MENU_KEYS` contains generic `VK_MENU` but not generic `VK_LWIN`/`VK_RWIN` aliases (there are none) (`:42`).
4. `sync_modifiers` writes `active` from `env.os_key_down` for the 11 `MODIFIERS` except the event key and swallowed ones (`src/engine/keys.rs:25-37, 102-111`); it runs at the start of every `on_key` (`src/engine/mod.rs:354`).
5. The harness oracle uses `ALT_AND_WIN = [VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN]` against the modelled OS, read after the decision; the OS model changes only for non-suppressed events (`src/engine/testing.rs:130, 538-541, 562-564`).
6. Properties turn `check_mask_rule()` on for every config except `mask_only` (`src/engine/properties.rs:257-261`); the `mask_only` config is the last entry (`:169-174`) and the seed formula is `seed.wrapping_mul(0x100) + index` (`:243`).
7. `mask_owed` is taken with `std::mem::take` at the start of `on_key` before `sync_modifiers` (`src/engine/mod.rs:348`), OR'ed into the final decision (`:360`), cleared by `reset` (`:418`), untouched by `on_foreground_changed` (`:393-396`) and `on_timer`.
8. A `MaskOnly` blocker never sets `suppress`; on a match with `!was_repeat` it sets `mask` and `mask_owed`; on a repeat it does nothing (`src/engine/mod.rs:453-464`).
9. The blocker's condition is evaluated only after `blocker_matches` returns true (short-circuit `&&`, `src/engine/mod.rs:449-451`); Sequence/Chord bindings still evaluate `when:` before their feed (`:534`, `:559`).
10. `carry_out` destructures `Decision { suppress: _, fire, timers, mask }` with no `..` (`src/hook/keyboard_hook_callback.rs:205-210`) and calls `send_menu_mask()` before timers and actions (`:211-219`).
11. `carry_out` runs after `decide_event` returned, i.e. after `with_engine` released the lock (`src/hook/keyboard_hook_callback.rs:191-195, 339-341`), and before `LRESULT(1)` (`:343`) or `CallNextHookEx` (`:349`).
12. `menu_mask_inputs` builds two `INPUT_KEYBOARD` records, `wVk = VIRTUAL_KEY(MENU_MASK_VK)`, `wScan = 0`, flags `0` then `KEYEVENTF_KEYUP`, `time = 0`, `dwExtraInfo = constants::injected_sentinel()` (`src/hook/keyboard_hook_callback.rs:225-239`); stack array, no allocation.
13. `inject_menu_mask` calls `SendInput(&inputs, size_of::<INPUT>() as i32)` in an `unsafe` block with a `// SAFETY:` comment (`src/hook/keyboard_hook_callback.rs:244-251`).
14. Non-test `send_menu_mask` logs `[WARN] SendInput inserted {sent} of 2 menu mask key events` via `eprintln!` when `sent != 2` (`src/hook/keyboard_hook_callback.rs:257-263`); the same short-count pattern exists in `press_vk` (`src/core/windows_actions.rs:220-228`).
15. Test `send_menu_mask` only increments the thread-local `MASKS_SENT` and does not call `inject_menu_mask`/`SendInput` (`src/hook/keyboard_hook_callback.rs:268-277`).
16. `tests/` contains only `tests/ipc_roundtrip.rs`; `rg "carry_out|keyboard_hook_callback|send_menu_mask" tests/` finds nothing.
17. Injected events are passed by `decide_event` at `src/hook/keyboard_hook_callback.rs:179-181`, before `hook.bindings()` (`:185`) and `with_engine` (`:191`).
18. Every `SendInput` call site in `src` sets `dwExtraInfo: injected_sentinel()`: `src/core/windows_actions.rs:200, 212, 220`; `src/data/sequence.rs:157, 176, 188, 228, 247`; `src/system/system_function.rs:131, 143, 151`; `src/hook/keyboard_hook_callback.rs:234, 250`.
19. `start_hold_timer` calls `carry_out(decision)` on the `hold-timer` thread (`src/hook/keyboard_hook_callback.rs:286-295`); the harness asserts timer decisions never mask (`src/engine/testing.rs:415`).
20. `RawBlockOriginalCombo` variants in declaration order: `On` (`rename = "true"`), `Off` (`#[default]`, `rename = "false"`), `MaskOnly` (`src/config/schema.rs:94-106`); the field is `#[serde(default)]` (`:54-56`).
21. `raw_blocker_to_mode`: `On -> Some(Swallow)`, `Off -> None`, `MaskOnly -> Some(MaskOnly)` (`src/config/loader.rs:602-608`).
22. `expand_with_auto_blockers` downgrades `MaskOnly` to `Swallow` and pushes the warning `binding #{index}: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the {hotkey} hotkey is blocked as with true` when `!hotkey.is_modifier_only()` (`src/config/loader.rs:353-360`); `index` is the position in `user_bindings` (`:346`).
23. One blocker per (hotkey, mode), modes in first-request order, condition = `any_of_conditions(owners)` (`src/config/loader.rs:380-395`); `any_of_conditions` returns `Always` on any `Always` owner or on none, one distinct condition as-is, several as `Condition::Or` (`:407-422`).
24. The KS-18 skip (a `keys` binding with the same combination) is checked once per hotkey, before any mode, and prints `[INFO] no auto-blocker for {hotkey}: binding #{index} uses the same keys` (`src/config/loader.rs:373-379`).
25. Blockers are appended after all file bindings (`src/config/loader.rs:398-400`).
26. `Binding::new_auto_blocker(combination, mode)` sets `block_original_combo: None`, `auto_blocker: Some(mode)` (`src/data/binding.rs:159-171`); the blocker condition is set by `.with_condition(..)` in the loader (`src/config/loader.rs:392-393`).
27. Blocker `Display` arm: `[AUTO-BLOCK] {:<24} -> {}{}{}{}` with `(blocked)` / `(masked, experimental)` (`src/data/binding.rs:283-292`).
28. Public API change: `Binding::block_original_combo` is `Option<BlockerMode>` (`src/data/binding.rs:103`) and `with_block_original_combo` takes `Option<BlockerMode>` (`:221`); `book/src/methodology.md:408-409` (cfgcheck snippet) formats `b.block_original_combo` with `{}`.
29. Book config pages link code at commit `ad7fe86` (22 matches of `feat/mask-key|ad7fe86` in `book/src/config/*.md`); `book/src/config/matching.md:115` says "Fixed on `feat/mask-key`".
30. `book/src/config/matching.md:109` carries the warning "The mask key in the Alt+Tab switcher" (not verified, manual check M3).
31. `src/engine/mod.rs` keeps `#![forbid(unsafe_code)]` (`src/engine/mod.rs:198`).
32. `MASK_ONLY_CONFIG` = default pair with `block_original_combo: MaskOnly` on CapsLock (`src/engine/scenarios.rs:4262-4265`).

## Red-first tests and acceptance proofs

Gate (tester owns the full run): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Developer ran only module filters (impl.md "Gates"): `cargo test --locked --lib config::` 100 passed; `--lib engine::` 248 passed; `--lib data::` 58 passed, 1 ignored (pre-existing); `--lib hook::` 32 passed, 1 ignored (H3). Full suite and `tests/` not run by the developer.

Single-test command form: `cargo test --locked --lib <path::to::test> -- --exact`. Expected on HEAD: `ok` for every row below except H3 (ignored; never run with `--ignored`).

### Commit `54e0673` (red) -> `82ae3f2` (green): KS-12 decision

| test (`engine::scenarios::` unless noted) | file:line | red on `54e0673` (impl.md) |
|---|---|---|
| S1 `win_q_post_message_masks_the_swallowed_q_and_its_release` | scenarios.rs:3907 | `down Q at +0 ms: expected the menu mask, got none` |
| S2 `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift` | :3932 | `down LShift at +0 ms: expected the menu mask, got none` |
| S3 `right_alt_and_right_win_mask_a_swallowed_key` | :3960 | `down J at +0 ms: expected the menu mask, got none` |
| S4 `swallowed_repeats_and_key_up_while_win_is_held_are_masked` | :3998 | `repeat Q at +500 ms: expected the menu mask, got none` |
| S8 `alt_held_before_the_hook_saw_it_masks_a_swallowed_key` | :4132 | `down J at +0 ms: ...` |
| S9 `win_held_through_remapping_on_masks_a_swallowed_key` | :4147 | `down Q at +0 ms: ...` |
| S10 `win_held_across_a_foreground_change_masks_a_swallowed_key` | :4168 | `down Q at +0 ms: ...` |
| S11 `sequence_and_chord_keys_swallowed_while_win_is_held_are_masked` | :4189 | `down F13 at +0 ms: ...` |
| `engine::properties::every_press_is_suppressed_or_passed_as_a_whole` | properties.rs:290 | `down K at +5019 ms: mask false, the rule says true (suppressed true, Alt or Win down in Windows true)` (config default, seed 0, Strict) |
| `engine::properties::nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` | :295 | `repeat CapsLock at +1697 ms: mask false, the rule says true ...` (config default, seed 0, Focus) |
| `engine::properties::auto_repeats_fire_only_actions_that_repeat` | :300 | same as above |
| guard S5 `a_swallowed_event_without_alt_or_win_in_windows_is_not_masked` | :4023 | green on red commit (kills mutation `mask = suppress`) |
| guard S6 `a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed` | :4060 | green (kills "count Alt/Win whatever its record") |
| guard S7 `a_passed_event_is_not_masked` | :4101 | green (kills mask without suppress) |
| guard S12 `a_hold_timer_never_masks` | :4240 | green (kills mask in `on_timer`, via `testing.rs:415`) |
| unit `engine::keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed` | keys.rs:621 | added in `82ae3f2` (no red) |

### Commit `44adabe` (red, compile) -> `11fa3e9` (green): adapter

| test (`hook::keyboard_hook_callback::tests::`) | file:line | red / status |
|---|---|---|
| H1 `menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel` | keyboard_hook_callback.rs:550 | `error[E0425]: cannot find function menu_mask_inputs` |
| H2 `carry_out_sends_the_menu_mask_only_when_the_decision_asks` | :570 | `error[E0425]: cannot find value MASKS_SENT in this scope` |
| H3 `the_menu_mask_key_is_inserted_into_the_input_stream` | :583-586 | `#[ignore = "manual: injects vk 0xE8 down and up into the live input stream; ..."]`; `cannot find function inject_menu_mask` on red; MANUAL ONLY (M8), report as manual, not a pass |

### Commit `a3e1149` (red) -> `58b71dd` (green): `MaskOnly` (KS-05 v2)

| test | file:line | red on `a3e1149` |
|---|---|---|
| L1 `config::loader::tests::block_original_combo_takes_true_false_and_mask_only` | loader.rs:1123 | `block_original_combo: MaskOnly,: config parse error (<in-memory>): 1:80: Expected boolean` |
| L2 `config::loader::tests::a_misspelled_block_original_combo_value_names_the_valid_ones` | :1149 | `Maskonly: config parse error (<in-memory>): 1:80: Expected boolean`; green text: ``1:88: Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead`` |
| L4 `config::loader::tests::mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns` | :1166 | `config does not load: ... 2:78: Expected boolean` |
| L5 `config::loader::tests::true_and_mask_only_on_one_hotkey_give_one_blocker_each` | :1264 | `... 3:81: Expected boolean` |
| S16 `mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey` | scenarios.rs:4268 | `scenario: config does not load: config parse error (<in-memory>): 2:70: Expected boolean` |
| S17 `mask_only_blocker_lets_ctrl_shift_t_reach_the_application` | :4293 | same |
| S18 `mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held` | :4320 | same |
| S19 `mask_only_blocker_masks_only_the_first_event_after_the_match` | :4344 | same |
| S20 `alt_shift_x_fires_next_to_a_mask_only_blocker` | :4369 | same at `2:78` |
| S21 `remapping_off_drops_an_owed_mask` | :4400 | same |
| unit `data::binding::tests::display_marks_a_mask_only_blocker`, `blocker_mode_display` | binding.rs:362, 387 | added in `58b71dd` |

### Commit `047da43` (red) -> `ad7fe86` (green): `when:` inheritance (KS-35)

| test | file:line | red on `047da43` |
|---|---|---|
| L7 `config::loader::tests::the_blocker_takes_the_when_of_the_binding_that_asked_for_it` | loader.rs:1203 | `left: Always, right: AppEquals("x.exe")` |
| L8 `config::loader::tests::blockers_of_one_hotkey_combine_the_when_of_their_owners` | :1213 | `two apps: left: [(Some(Swallow), Always)] right: [(Some(Swallow), Or([AppEquals("a.exe"), AppEquals("b.exe")]))]` |
| L9 `config::loader::tests::a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when` | :1249 | `left: [(Some(Swallow), Always), (Some(MaskOnly), Always)]` |
| S13 `a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps` | scenarios.rs:4422 | `down LShift at +0 ms: expected the event to be passed, it was suppressed` |
| S14 `blockers_act_where_any_owner_when_holds` | :4457 | same (context c.exe) |
| E1 `engine::tests::a_blocker_when_condition_is_asked_only_once_its_hotkey_matches` | mod.rs:727 | panic `the foreground was asked for although no binding with a when: matches this key` |

### Acceptance mapping (cut.md "Acceptance mapping")

| acceptance line | proofs |
|---|---|
| mask = true exactly when suppressed after Alt/Win reached the OS, false otherwise | true: S1-S4, S8-S11; false: S5-S7, S12; property oracle `check_mask_rule` (Fact 5, 6) |
| opt-in v2 passes modifier key-downs and masks on exact match | S16-S20, L1, L5; owed mask pinned by S16, S17, S19, S21; property config `mask_only` (symmetry, nothing stuck, no mask oracle) |
| blocker of a `when: AppEquals("x.exe")` binding inactive elsewhere | S13 (+ S14, L7-L9, E1) |
| injection only in `#[ignore = "manual: ..."]` tests | H3 (H1, H2 inject nothing; Fact 15) |
| README, `default_config.ron`, book config pages, CHANGELOG | commits `11fa3e9`, `58b71dd`, `ad7fe86`, `0187cde`; file map rows above |
| manual checks listed | M1-M9 below |

### Pre-existing scenarios whose expectations changed

- `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing` (`scenarios.rs:2591`): `.expect_unmasked()` added (Ctrl case).
- `shift_pressed_after_tab_was_released_is_still_swallowed` (`scenarios.rs:2618`): `.expect_masked()` added.
- `auto_blocker_matches_its_modifiers_exactly` (`:1220`), `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (`:2649`): call sites updated to `BlockerMode`.

### Manual checks (human only; cut.md "Manual checks", impl.md)

M1 Alt, Shift in Notepad with default config: no menu bar, `fx` typed, layout unchanged. M2 `Win+Q -> PostMessage(WM_CLOSE)`: Notepad closes, Start stays closed. M3 Alt held, Tab, Shift, Tab: switcher steps forward, no menu bar (failure = RETHINK at results). M4 (v2, Ctrl+Shift hotkey, `MaskOnly`): Ctrl+Shift+T reopens tab; bare Ctrl+Shift x5 never switches; CapsLock switches. M5 (v2, Alt+Shift): bare Alt+Shift x5 never switches; Alt+Shift+Tab steps back; Notepad Alt, Shift, release opens no menu. M6 `true` + `when: AppEquals("notepad.exe")`: no switch in Notepad, switch elsewhere. M7 `` ` `` hotkey + `MaskOnly`: startup `[WARN] binding #0: block_original_combo: MaskOnly needs ...`; lone `` ` `` swallowed. M8 debug build, Notepad focused: H3 via `--ignored` passes, nothing typed. M9 `osk.exe` shows no modifier held after M1-M6. If M4/M5 fail: keep `true` default, record in audit status.

## Open points (from impl.md)

1. Public API change: `Binding::block_original_combo: Option<BlockerMode>` (was `bool`), `with_block_original_combo(Option<BlockerMode>)`, `new_auto_blocker(combo, mode)`; external users (methodology cfgcheck, `book/src/methodology.md:408-409`) must adapt.
2. Phase book sync hand-off (not edited in this lane): `book/src/architecture.md:268, 271, 544`; `book/src/build-and-run.md:415, 457-465`; `book/src/glossary.md:141, 313`; `audit.md` status lines for KS-05, KS-12, KS-35; roadmap status and Checkpoint-1 item 4 wording (`book/src/roadmap.md:308`); methodology cfgcheck snippet.
3. Checkpoint-1 item 4 passes only with `block_original_combo: MaskOnly` (M4/M5); with `true`, Alt, Shift, Tab still swallows Shift and Ctrl, Shift, T arrives as Ctrl+T (critic M2, resolved by statement; `results.md` must say so).
4. Mask in the Alt+Tab switcher (critic open question 1): documented as a warning (`book/src/config/matching.md:109`), settled by M3 only.
5. H2 fail-safety seam at `src/hook/keyboard_hook_callback.rs:253-277`; reviewer checklist line from cut.md addendum: the `cfg(test)` `send_menu_mask` must not call `inject_menu_mask`/`SendInput`, and no file under `tests/` may reach `carry_out` or `keyboard_hook_callback` (Fact 15, 16).
6. Book GitHub links pinned to `ad7fe86` resolve only after the merge step pushes (Fact 29).
7. Deviations from the cut (impl.md): ", masked" transcript marker; shared private `with_alt_shift_blocker_of`; `mask_only` property config appended last; D1 helper never clears `mask`; README Errors section additions; `triggers-conditions.md` §12.4 bullet rewritten beyond the blocker part; `matching.md` §15.4 Alt+Tab warning; `example.md:6` intro changed; no new `default_config.ron` example binding.
8. Cut non-goals still open: KS-19 (3.1), known-issues row 66 (lone Alt/Win Tap/Hold/DoubleTap passing, not covered by the mask; `book/src/config/known-issues.md:134`), KS-17 (1.4), KS-36 (scan code 0 on E8, 3.1), once-per-press masking (cut Q2).
9. impl.md "Unsafe / atomics added": one new `unsafe` in production (`keyboard_hook_callback.rs:246-250`), one in test H1 (`:554-556`); no atomics; `mask_owed` lives inside `Mutex<Engine>`.

## Invariants in play

- **R1 hook callback**: mask `SendInput` runs on the hook thread inside the callback after the lock (`src/hook/keyboard_hook_callback.rs:211-213, 339-343`); stack-only inputs (`:225-239`); `eprintln!` only on short count (`:260-262`); `code < 0` and remapping-off early returns unchanged (`:320-333`). KS-08.
- **R2 sentinel**: E8 records tagged (`:234`); sentinel check precedes lock (`:179`); all call sites (Fact 18).
- **R3 shared state / re-entry**: `SendInput` re-enters the hook (doc `:58-82`); `mask_owed` is plain data in `Mutex<Engine>` (`src/engine/mod.rs:321`); no new statics except `cfg(test)` thread-local (`:273-277`).
- **R4 atomics**: none added (impl.md).
- **R5 builders**: `new_auto_blocker(combination, mode)` two positional args, `.with_condition` (`src/data/binding.rs:159`; `src/config/loader.rs:392-393`); `with_block_original_combo` consuming (`binding.rs:221`).
- **R6 Display**: `BlockerMode` Display (`binding.rs:82-89`); blocker arm both modes (`:283-292`); book `structure.md:150`, `matching.md` §15.5.
- **R7 specificity order**: blockers appended last, first-request order (`src/config/loader.rs:380-400`); `sort_by_specificity` unchanged (`src/engine/mod.rs:309-311`). KS-18 skip at `loader.rs:373-379`.
- **R8 key identity**: mask key set includes L/R and generic Alt, L/R Win (`keys.rs:42`); oracle excludes generic `VK_MENU` (`testing.rs:130`). KS-07.
- **R9 registry**: unchanged; `MaskOnly` fallback uses `KeyCombination::is_modifier_only` on the registry hotkey (`loader.rs:353`). KS-17 out of scope.
- **R10 message loop/shutdown**: unchanged; no install/uninstall touched.
- **Matching/suppression**: suppression symmetry pinned by `check_press_symmetry` in S16-S21, S13, S14; KS-05 (`true` swallows second modifier, `scenarios.rs:2591, 2618`), KS-06 repeat rule (`mod.rs:580-604`).
- **Timers**: `on_timer` never masks (`mod.rs:366-372`; `testing.rs:415`); KS-11/25 unchanged.
- **Config/reload**: `RawBlockOriginalCombo` parse (`schema.rs:87-106`); warnings threaded (`loader.rs:292-293`), printed at `loader.rs:236-237`; owed mask survives reload/foreground change (cut "Risks", `mod.rs:393-396`). KS-35 (`loader.rs:334-337, 380-395`).

## Not covered

- Not read: `src/engine/{gestures,chords,sequences}.rs`, `src/hook/keyboard_hook.rs` (`with_engine`, `update_bindings`), `src/data/{condition,key_combination}.rs` (only `is_modifier_only` and `Condition` equality are cited via the cut), `src/system/*`, `src/core/windows_actions.rs` beyond lines 200-228.
- Scenario bodies S1-S12 (`scenarios.rs:3897-4256`) and loader tests L1-L9 bodies were located by name, not read line by line; S13, S14, S16-S21 bodies were read.
- No `cargo` command was run by this scout (no fmt, clippy, test); all pass/count figures above are the developer's (impl.md).
- `critique.md` was not mined beyond what impl.md and the cut addendum restate (M1, M2, open questions 1-3).
- Book pages outside `book/src/config/` and the CHANGELOG body beyond lines 60-92 were not read; doc ranges come from `git diff -U0` hunk headers.
- Red-first reproduction on the red commits was not performed; the red messages are quoted from impl.md.
