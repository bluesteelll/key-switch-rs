# Context pack r1: p0-A4-autoblockers (KS-18, KS-05 exact part)

Scout: project-analyst. Read-only. Tree `D:/key-switch-rs/key-switch-rs`, branch `fix/auto-blockers`, HEAD `f5595aa`, merge-base = `master` = `44f79ac`. Tree clean at start.
Inputs read: `cut.md` (incl. developer addendum, lines 387-397), `impl.md`. `critique.md` not re-read (impl.md:27-34 records its outcome).

## Scope and delta

`git log --oneline master..fix/auto-blockers`:

```
f5595aa Engine: an auto-blocker matches only the hotkey on its own and never takes a user binding's fire (KS-05, KS-18)
8a08253 Tests: red-first: auto-blockers act only when Windows sees exactly the hotkey, and fire nothing (KS-05, KS-18)   [declared RED]
bb4dfcd Config: an auto-blocker is skipped when any binding uses the hotkey's keys, and follows the file's bindings (KS-18)
cae1ec5 Tests: red-first: auto-blocker dedup over the whole binding list (KS-18)   [declared RED]
91937a0 Config: auto-blocker expansion takes the system hotkey lookup as a parameter
```

`git diff --stat master...fix/auto-blockers` (13 files, +789 / -49):

```
CHANGELOG.md                  |  34   README.md                 |   2
src/config/default_config.ron |   5   src/config/loader.rs      | 217
src/config/mod.rs             |   4   src/data/binding.rs       |   6
src/engine/env.rs             |   8   src/engine/keys.rs        |  27
src/engine/matching.rs        | 105   src/engine/mod.rs         |  24
src/engine/properties.rs      |   4   src/engine/scenarios.rs   | 379
src/engine/testing.rs         |  23
```

Not in the delta: `src/hook/**`, `src/core/**`, `src/system/**`, `src/data/key_*`, `Cargo.toml`, `book/**`, `.claude/**`.

## File map (line ranges on f5595aa)

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/config/loader.rs:123-125` | `pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>` | D6 seam (commit 91937a0) |
| `src/config/loader.rs:127-136` | `from_ron_str` doc + body: delegates to `from_ron_str_with_hotkeys(text, &BindAction::get_system_combination)` | production path still reads the registry |
| `src/config/loader.rs:138-176` | `from_ron_str_with_hotkeys`; calls `expand_with_auto_blockers` at `:175` | D6 |
| `src/config/loader.rs:178-219` | `expand_with_auto_blockers`: collect distinct hotkeys `:191-199`; skip if any binding's `combination()` equals hotkey `:201-214` with `[INFO]` at `:210`; append blockers after user bindings `:216-218` | KS-18 dedup + order (D4, D5) |
| `src/config/loader.rs:505-606` | pre-existing loader tests touching blockers: `default_config_parses` `:506`, `parse_simple_binding` `:516` (comment changed `:529-531`), `auto_blocker_not_added_without_flag` `:548`, `..._for_non_system_actions` `:563`, `auto_blocker_deduped_when_two_bindings_want_the_same_combo` `:582` | go through `from_ron_str` (registry) |
| `src/config/loader.rs:607-743` | new loader tests + helpers `language_hotkey` `:632`, `blockers` `:643` | acceptance 3 (loader dedup) |
| `src/config/mod.rs:16-20` | `#[cfg(test)] pub(crate) use loader::{from_ron_str, from_ron_str_with_hotkeys}` | test-only re-export |
| `src/config/default_config.ron:117-131` | `block_original_combo` comment; new lines `:126-130` | config meaning doc |
| `src/data/binding.rs:66-75` | `allow_extra_modifiers` doc (blockers ignore it), `is_auto_blocker` doc; field `pub(crate)` `:75` | spec 3 (flag) |
| `src/data/binding.rs:98-109` | `new_auto_blocker` (unchanged): DoNothing, `block_default: true`, Always, Immediate, `is_auto_blocker: true` | blocker shape |
| `src/data/binding.rs:114-119` | `combination()` returns `Some` only for `BindingKind::Combo` | dedup equality input |
| `src/engine/matching.rs:16-21` | module doc: blocker matches by what Windows sees | D2 |
| `src/engine/matching.rs:28-46` | `HeldKeys` trait; new `all_held` `:33-36` | D2 |
| `src/engine/matching.rs:48-79` | `keys_down`, `is_trigger_key`, `combo_matches` (unchanged) | user-binding path |
| `src/engine/matching.rs:81-101` | `blocker_matches` (new) | spec 1 / KS-05 |
| `src/engine/matching.rs:240-308` | unit tests U1 `blocker_matches_only_what_windows_sees_as_its_hotkey` `:241`, U2 `blocker_never_asks_windows_about_its_own_keys` `:298` | D2 pins |
| `src/engine/keys.rs:36-40` | `slot(vk)`: index = code if `< 256` | bound for `all_held` |
| `src/engine/keys.rs:204-210` | `impl HeldKeys for KeyState::all_held`: `(0..256u16)` filtered by `active`, builds `VIRTUAL_KEY(code)` | only new `VIRTUAL_KEY(raw)` construction (critique M1) |
| `src/engine/keys.rs:258-273` | U3 `all_held_visits_every_held_code` | pin |
| `src/engine/env.rs:32-43` | `Env::os_key_down` contract text (blocker asks about held keys outside its hotkey) | D2 contract |
| `src/engine/mod.rs:106-127` | module doc rule 4 incl. new blocker bullet `:119-125`, "first matching Immediate user binding" `:114` | doc |
| `src/engine/mod.rs:336-365` | `on_key_down` head; `immediate_fired` `:342`; blocker branch `:353-365` | D1/D3 hot path |
| `src/engine/mod.rs:366-383` | user Combo path: `combo_matches`, Immediate slot `:376-379` | KS-18 shadowing point |
| `src/engine/mod.rs:473-506` | rule 5 record logic (repeat / mark_blocked / mark_passed), unchanged | suppression symmetry for blocker-swallowed keys |
| `src/engine/mod.rs:509-535` | `on_key_up`, unchanged | key-up suppression = record |
| `src/engine/properties.rs:54-58` | `Config.blocker` doc text changed | doc only |
| `src/engine/testing.rs:50-93` | `FakeEnv`; `os_key_down` `:78-85` (generic codes = either side) | D2 model |
| `src/engine/testing.rs:139-162` | `from_ron` doc `:140-144`; new `from_ron_with_language_hotkey` `:151-162` | D6 in scenarios |
| `src/engine/testing.rs:388-399` | `key_event`: `os_down` updated only when the event is not suppressed `:395-398` | what "Windows has down" means in tests |
| `src/engine/testing.rs:240-268` | `up_unseen` / `down_unseen` change `os_down` without telling the engine | T9 model |
| `src/engine/scenarios.rs:21-41` | `DEFAULT_PAIR`, `with_alt_shift_blocker` (blocker first; doc `:27-31` changed) | blocker-first fixture |
| `src/engine/scenarios.rs:1169-1192`, `:1517-1559`, `:1649-1681` | F1, F2, F3 (assertions flipped at `:1180`, `:1531`, `:1548`, `:1661`, `:1679`) | D3 flips |
| `src/engine/scenarios.rs:1880-1912` | `modifier_held_before_the_engine_saw_it...` (KS-10 sentence dropped `:1888`) | comment only |
| `src/engine/scenarios.rs:1914-2273` | new scenarios T1, `DEFAULT_CONFIG` `:1958-1962`, T2..T10, M4 pin, M3 pin | acceptance 1, 2 |
| `README.md:428` | `block_original_combo` row | config doc |
| `CHANGELOG.md:77-85`, `:96-106`, `:156-169` | Changed **Breaking** (KS-18 `when:`, KS-05 grave); Fixed KS-05 (partly); Fixed KS-18 | Keep a Changelog |
| `src/hook/keyboard_hook_callback.rs:116-122`, `:155-165` | `HookEnv::os_key_down` (`GetAsyncKeyState`, MSB); adapter calls `with_engine(... on_key ...)` `:164` | unchanged; blocker's Env calls run here |
| `src/hook/keyboard_hook.rs:94-99` | `update_bindings`: `sort_by_specificity` then one `store` | unchanged (R7) |
| `src/engine/mod.rs:255-261` | `sort_by_specificity`: stable `sort_by_key(Reverse(key_count))` | unchanged (R7) |
| `src/system/registry.rs:52-79`, `:181-189` | `get_system_hotkey` (registry, then default), `get_default_combination`, `parse_language_hotkey` | unchanged; source of real hotkeys |

## Facts

Loader (KS-18, D4-D6)
1. `from_ron_str` has the same signature as on master and delegates to `from_ron_str_with_hotkeys` with `&BindAction::get_system_combination` (`src/config/loader.rs:134-136`).
2. The lookup is called only for bindings with `block_original_combo == true` (`&&` chain, `src/config/loader.rs:193-194`).
3. Hotkeys are deduplicated by `Vec::contains` (`KeyCombination::eq`) in first-request order (`src/config/loader.rs:191-199`).
4. `KeyCombination::eq` is order-free: same length and every key of `self` contained in `other` (`src/data/key_combination.rs:16-30`).
5. A hotkey is skipped when `user_bindings.iter().position(|b| b.combination() == Some(&hotkey))` is `Some`; the scan covers every user binding, the requester included, with no filter on `trigger`, `condition`, `block_default` or action (`src/config/loader.rs:205-207`).
6. `combination()` is `Some` only for `BindingKind::Combo` (`src/data/binding.rs:114-119`); sequences and chords never cause a skip.
7. The skip prints `[INFO] no auto-blocker for {hotkey}: binding #{index} uses the same keys` with the first matching index, 0-based (`src/config/loader.rs:210`).
8. The result is the user bindings in file order followed by the blockers (`src/config/loader.rs:216-218`).
9. `is_auto_blocker` is set to `true` only in `new_auto_blocker` (`src/data/binding.rs:107`); `Binding::new` sets `false` (`:94`). The field is `pub(crate)` (`:75`).
10. Readers of `is_auto_blocker` outside tests: `Display` (`src/data/binding.rs:174`, `[AUTO-BLOCK]` at `:177`) and the engine (`src/engine/mod.rs:353`).

Engine (KS-05 exact part, D1-D3)
11. In the `Combo` arm the flag is checked first; a blocker either sets `decision.suppress = true` (when `blocker_matches`) or does nothing, then `continue`s; it never touches `decision.fire` or `immediate_fired` (`src/engine/mod.rs:352-365`).
12. The blocker branch runs after the `when:` check (`src/engine/mod.rs:347-349`); `new_auto_blocker` uses `Condition::Always` (`src/data/binding.rs:104`).
13. `blocker_matches` = `is_trigger_key(combo, pressed) && keys_down(combo, held) && held.all_held(|vk| combo accepts vk || !windows_has_down(vk))` (`src/engine/matching.rs:92-101`).
14. `windows_has_down` is evaluated only for held codes no key of `combo` accepts (short-circuit `||`, `src/engine/matching.rs:100`); the engine passes `|key| env.os_key_down(key)` (`src/engine/mod.rs:359-361`).
15. `KeyState::all_held` iterates `0..256u16`, filters `self.active[usize::from(code)]`, and builds `VIRTUAL_KEY(code)` (`src/engine/keys.rs:204-210`); `active` is `[bool; 256]` (`src/engine/keys.rs:65`).
16. `KeySpec::AnySide(m).accepts` accepts left, right and generic codes; `KeySpec::Key(k)` only `k` (`src/data/key_spec.rs:133-139`).
17. Registry/default language hotkeys are built with `KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT])`, `[VK_CONTROL, VK_SHIFT]`, or `KeyCombination::new(VK_OEM_3)` (`src/system/registry.rs:69-71`, `:183-186`); Win hotkeys use `KeySpec::AnySide(Modifier::Win)` (`src/system/registry.rs:204-206`).
18. User Combo matching (`combo_matches`) is unchanged and still ignores held non-modifier keys (`src/engine/matching.rs:65-79`).
19. Immediate user bindings: only the first matching one fires (`immediate_fired`, `src/engine/mod.rs:375-379`); `block_default` suppresses (`:380-382`).
20. Repeat/record logic is unchanged: repeat of a blocked key stays suppressed, of a passed key stays passed (`src/engine/mod.rs:473-500`); first key-down suppressed -> `mark_blocked` (`:501-502`); key-up suppressed iff blocked record or pending gesture (`src/engine/mod.rs:529-533`).
21. `on_key` resyncs modifiers before matching, skipping the delivered key (`src/engine/mod.rs:294`).
22. `HookEnv::os_key_down` calls `GetAsyncKeyState` and tests bit 0x8000 (`src/hook/keyboard_hook_callback.rs:116-122`); the engine runs inside `hook.with_engine(...)` (`:164`).
23. No `unsafe`, atomic, lock, thread, allocation or Win32 import was added in `src/engine/**` per impl.md:46-48; the engine-side diff hunks are `matching.rs`, `keys.rs`, `mod.rs`, `env.rs` (doc), `properties.rs` (doc), `testing.rs`, `scenarios.rs` (from `git diff -U0`). UNVERIFIED beyond the hunk list: no full re-read of every added line for allocations.

Test harness
24. `FakeEnv::os_key_down` maps `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` to either side and other codes to `os_down[code]` (`src/engine/testing.rs:78-85`).
25. In scenarios `os_down` changes only for events the engine passes (`src/engine/testing.rs:395-398`), and for `up_unseen`/`down_unseen` (`:248`, `:266`).
26. `from_ron_with_language_hotkey` answers `Some(hotkey)` only for `SwitchLanguage`/`SwitchLanguageBackward` and loads via `from_ron_str_with_hotkeys` (`src/engine/testing.rs:151-162`).
27. `with_alt_shift_blocker` puts the blocker first and loads the rest with `crate::config::from_ron_str` (`src/engine/scenarios.rs:32-41`).
28. `Scenario::with_bindings` sorts with `sort_by_specificity` (`src/engine/testing.rs:165-167`).

Registry dependency of tests (impl.md open point 1)
29. `from_ron_str` reaches the registry only through the lookup for flagged bindings (facts 1, 2). Tests that load a flagged binding through `from_ron_str`: `default_config_parses` (`src/config/loader.rs:506-512`), `parse_simple_binding` (`:516-545`), `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (`:582-605`).
30. `parse_tap_trigger` (`src/config/loader.rs:1186-1200`) sets no `block_original_combo`; impl.md:82 lists it among registry-reading tests.
31. `parse_language_hotkey("3")` returns `None` (`src/system/registry.rs:184`); `get_system_hotkey` then falls back to `get_default_combination`, which returns Alt+Shift for both language functions (`src/system/registry.rs:52-71`). impl.md:82 states that with hotkey "3" the first two tests "would fail".
32. `with_alt_shift_blocker` callers and T7's `from_ron_str(USER)` load configs without `block_original_combo` (`src/engine/scenarios.rs:22-25`, `:2138`).

Ordering / R7
33. `update_bindings` = `sort_by_specificity` + one `ArcSwap::store` (`src/hook/keyboard_hook.rs:94-99`), unchanged; the stable sort is `src/engine/mod.rs:259-261`.
34. `.claude/agents/code-reviewer.md:124` and `.claude/agents/project-analyst.md:60` still say "auto-blockers rely on load order among equal counts (KS-18)"; not in the delta.

Docs
35. CHANGELOG hunks: `+77-85` (Changed, Breaking), `+96-106` (Fixed KS-05 partly), `+156-169` (Fixed KS-18) (`git diff -U0 -- CHANGELOG.md`).
36. README row `README.md:428` is the only README change.
37. `book/src/**` unchanged in the delta; cut.md:342 lists the pages left to the phase book sync.

Gate run by the scout
38. `cargo test --locked --lib -- config:: engine:: data::` on f5595aa: `test result: ok. 271 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out` (matches impl.md:53). fmt, clippy and the full `cargo test --locked` were not run by the scout.

## Red-first tests and acceptance proofs

Commands (from cut.md:210 and impl.md:24, :52-53):
- Gate: `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Expected on f5595aa: all green, 0 ignored added (impl.md:54).
- Red-commit reproduction, as recorded by the developer: `cargo test --locked --lib -- config:: engine::` gave **4 failed / 209 passed on cae1ec5** and **14 failed / 210 passed on 8a08253** (impl.md:24). Running it needs those commits checked out (e.g. a separate `git worktree` outside the repo dir); the scout did not run it.
- Single test: `cargo test --locked --lib <test_name>`.
- Failure-text format: `"{label}: expected the event to be {x}, it was {y}"` and `"{label}: expected to fire {expected:?}, fired {actual:?}"`, label `"{what} {key} at +{ms} ms"` (cut.md:212, citing `testing.rs`).

### Red on cae1ec5 (4), green from bb4dfcd

| Test | Location | Expected red text (impl.md:10-13) |
|---|---|---|
| L1 `auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order` | `src/config/loader.rs:650` | ``assertion `left == right` failed: flagged binding above`` / `left: 1` / `right: 0` |
| L2 `auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys` | `src/config/loader.rs:689` | ``assertion `left == right` failed`` / `left: 1` / `right: 0` |
| L3 `auto_blockers_follow_the_bindings_of_the_file` | `src/config/loader.rs:700` | panic `the file's bindings come first` |
| T1 `user_binding_on_the_language_hotkey_fires_in_either_file_order` | `src/engine/scenarios.rs:1915` | `down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]`, `context: flagged binding above` |

Green throughout (pins): L4 `auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey` (`loader.rs:728`); C1 `auto_blocker_keys_come_from_the_hotkey_lookup` (`loader.rs:608`, green from 91937a0).

### Red on 8a08253 (14), green on f5595aa

| Test | Location | Expected red text (impl.md:16-22) |
|---|---|---|
| T2 `shift_pressed_while_alt_and_tab_are_held_reaches_windows` | `scenarios.rs:1965` | `down LShift at +500 ms: expected the event to be passed, it was suppressed` (label +500 ms: deviation, impl.md:38) |
| T3 `shift_pressed_while_ctrl_and_t_are_held_reaches_windows` | `scenarios.rs:1997` | `down LShift at +0 ms: expected the event to be passed, it was suppressed` |
| T4 `alt_shift_pressed_while_a_letter_is_held_reaches_windows` | `scenarios.rs:2030` | same as T3 |
| T5 `grave_blocker_acts_only_on_a_lone_grave` | `scenarios.rs:2051` | ``down ` at +0 ms: expected the event to be passed, it was suppressed`` |
| T6 `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing` | `scenarios.rs:2082` | `down LShift at +0 ms: expected to fire [], fired [DoNothing]` |
| M4 pin `shift_pressed_after_tab_was_released_is_still_swallowed` | `scenarios.rs:2106` | same as T6 |
| T7 `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` | `scenarios.rs:2135` | `down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]`, `context: blocker first` |
| T8 `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` | `scenarios.rs:2165` | `down RShift at +0 ms: expected to fire [], fired [DoNothing]` |
| T9 `key_whose_release_the_hook_missed_does_not_disable_the_blocker` | `scenarios.rs:2198` | same as T6 |
| T10 `key_swallowed_by_a_binding_does_not_disable_the_blocker` | `scenarios.rs:2223` | same as T6 |
| M3 pin `modifier_swallowed_by_a_binding_does_not_disable_the_blocker` | `scenarios.rs:2247` | `down LShift at +0 ms: expected the event to be suppressed, it was passed` |
| F1 `auto_blocker_matches_its_modifiers_exactly` | `scenarios.rs:1170` (flip `:1180`) | same as T6 |
| F2 `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` | `scenarios.rs:1518` (flips `:1531`, `:1548`) | same as T6 |
| F3 `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` | `scenarios.rs:1650` (flips `:1661`, `:1679`) | same as T6 |

Added with the fix (cannot compile before f5595aa): U1 `matching.rs:241`, U2 `matching.rs:298`, U3 `keys.rs:259`.

### Acceptance mapping (cut.md:265-276)

| Acceptance | Proof |
|---|---|
| Alt+Shift+Tab and Ctrl+Shift+T pass through with the blocker | T2, T3, T4 (extra key down in Windows -> pass); T6 + F2 (third key in modifiers-first order passes; the second modifier is still swallowed: 1.2 residue, `roadmap.md:100-101`, cut Q1) |
| User binding on Alt+Shift fires with the blocker, either file order | T7 (blocker first / last), T1 (loader: above, below, flagged itself), T8 (sided user binding + blocker) |
| Loader dedup tests | L1, L2, L3, L4, C1; pre-existing `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (`loader.rs:582`) |
| Spec 1 exactness | T2-T5, T9, T10, M3 pin, U1, U2, F1 |
| Spec 3 distinguishable | `is_auto_blocker` (`binding.rs:75`), asserted in L3, L4, C1 |

Manual checks M1-M5 (human only, no `#[ignore]` stubs added): impl.md:56-62.

## Open points (verbatim scope from impl.md:80-85, plus cut questions)

1. impl.md:82: loader tests loading `block_original_combo` through `from_ron_str` read `HKCU\Keyboard Layout\Toggle`; could move to `from_ron_str_with_hotkeys`. Code facts relevant: facts 29-31.
2. impl.md:83: pre-existing rustdoc warning `src/data/vk_name.rs:4` "unclosed HTML tag `raw`" (not a gate).
3. impl.md:84: `src/engine/matching.rs` line endings restored to CRLF in the working copy; index content LF.
4. impl.md:85: stale agent rule texts (fact 34); cut Q6 assigns them to the orchestrator, audit status lines to the doc-writer.
5. impl.md:28 / cut.md:391: critique M1 (`VIRTUAL_KEY(code)` built in the engine at `keys.rs:208`) left for the reviewer under CLAUDE.md principle 7.
6. cut.md:397: questions 1, 2, 4-7 unanswered by the orchestrator; implementation follows the cut's recommendations (D2 reading kept, F1-F3 flipped, dedup ignores `when:`/`trigger`, book deferred).
7. Deviations recorded in impl.md:36-44 (T2 repeat order, T9 adds LWin, extra tests, stronger checks, template wording, `mod.rs:21-22` doc).

## Invariants in play

- R1 hot path: blocker branch `src/engine/mod.rs:353-365` runs inside `with_engine` (`src/hook/keyboard_hook_callback.rs:164`); `GetAsyncKeyState` per held key outside the hotkey (`matching.rs:100`, `keyboard_hook_callback.rs:116-122`); bounded table scan `keys.rs:207-209`. KS-08.
- R2 sentinel: no `SendInput` in the delta (stat above).
- R3 shared state: `Mutex<Engine>` only via `with_engine` (`keyboard_hook_callback.rs:164`); no new statics in the delta.
- R4 atomics: none in the delta.
- R5 builders: `new_auto_blocker` unchanged (`binding.rs:98-109`); no setter added.
- R6 Display: no new variant; `[AUTO-BLOCK]` arm `binding.rs:174-177`; `[INFO]` uses `KeyCombination` Display (`loader.rs:210`).
- R7 specificity: `keyboard_hook.rs:94-99`, `engine/mod.rs:259-261` unchanged; loader order changed (`loader.rs:216-218`); rule text fact 34. KS-18.
- R8 key identity: `KeySpec::accepts` sided/generic (`key_spec.rs:133-139`); generic slot visited by `all_held` (`keys.rs:268-271` test). KS-07.
- R9 registry: read path unchanged (`registry.rs:52-79`); new tests use the stand-in lookup (facts 26, 32); fact 29 lists registry-reading tests. KS-17 ("3" fallback, fact 31).
- R10: no message-loop or shutdown change.
- Matching/suppression: KS-04 trigger-key rule `matching.rs:58-63`; KS-05 `matching.rs:81-101`; KS-06/KS-10 repeat records `mod.rs:473-506`; KS-13 first-Immediate `mod.rs:375-379`; KS-14 resync skip `mod.rs:290-294`.
- Out of scope per cut.md:45-51: KS-12 / rest of KS-05 (1.2), KS-35 blocker `when:` (1.2), KS-17 (1.4), KS-13 (1.6), KS-29/KS-31 (2.3), `os_guards` (3.4).

## Not covered

- Bodies of T2-T6, T8-T10 and L1-L4 not re-read line by line (only names, locations and T1, T7, M3 bodies at `scenarios.rs:1914-1956`, `:2134-2162`, `:2247-2273`); their scripts are as in cut.md:241-252 plus impl.md:36-41 deviations, UNVERIFIED against the code.
- U1-U3 bodies read only in part (`matching.rs:240-246`, `keys.rs:258-273`).
- `critique.md` not read; `CHANGELOG.md` and `README.md:428` texts not compared word-for-word with cut.md:322-340.
- `src/engine/properties.rs` beyond `:50-62`; `src/core/app.rs` banner (`:57-70`) not re-read.
- Red-commit reproduction, `cargo fmt --check`, clippy and the full `cargo test --locked` not run by the scout (only fact 38).
- `book/src/**` drift not checked (deferred by cut Q5).
