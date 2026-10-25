DONE

# Implementation: p1-1.4-hotkey-tristate round 1

Branch `fix/language-hotkey-tristate` from `master` @ `f28da35`, HEAD `9fcbc93`, tree clean. Five commits, in the cut's order.

## Items
- Roadmap 1.4 / KS-17, commit 1 (test-only): `src/config/loader.rs:884-910` (`parse_simple_binding`) and `:950-972` (`auto_blocker_deduped_when_two_bindings_want_the_same_combo`) use `from_ron_str_with_hotkeys(.., &language_hotkey(ALT_SHIFT))`. `src/system/registry.rs:462-479` (`test_get_system_hotkey_returns_combinations`): `SwitchLanguage` is `None` or one of Alt+Shift, Ctrl+Shift and the grave accent (`Option::is_none_or`). Green; no product change.
- Roadmap 1.4 / KS-17, commit 2 (no behaviour change):
  - `registry.rs:17-28`: private `HotkeySetting { Combo, Disabled, Unknown }`, `Debug`/`Clone`/`PartialEq`/`Eq`. The `parser` field becomes `fn(&str) -> HotkeySetting` (`:14`).
  - `:226-245`: tri-state `parse_language_hotkey` and `parse_win_key_combo`.
  - `:148-170` `open_key` and `:177-217` `read_string_value`: the code is moved verbatim from `read_from_registry` / `read_value_and_parse`.
  - `:79-117` `pub(crate) hotkey_from_values`: lazy, takes the first deciding value. `Disabled` was skipped here, with the KS-17 comment.
  - `:65-77` `get_system_hotkey`: same `pub` signature. It passes `value_names.iter().map(read_string_value)` while the guard is alive, and an empty iterator with no location or no key.
  - Tests (all green): `parse_language_hotkey_is_tri_state`, `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` (I1), `the_hotkey_value_decides_before_the_language_hotkey_value`, `no_usable_value_falls_back_to_alt_shift`, `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide`, `the_values_after_the_deciding_one_are_not_read`, `other_functions_keep_their_rules`; `language_hotkeys_are_either_side_modifiers` and `test_parse_win_key_combo` rewritten with `let .. else panic!` (plus `"1"`, `""` and `"LL"` → `Unknown`).
  - Loader: helper `toggle_values` (`loader.rs:1017-1028`) and `the_registry_values_choose_the_auto_blocker_keys` (`:1030-1051`, template; `[Some("2"),None]` → Ctrl+Shift, `[None,None]` and `[Some("1"),Some("3")]` → Alt+Shift).
- Roadmap 1.4 / KS-17, commit 3 (declared red, fmt and clippy green): `registry.rs:437-459` `a_language_hotkey_not_assigned_in_windows_has_no_combination` and `loader.rs:1053-1070` `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker`.
  - Red on `8ef0ab7` (log: `D:/tmp/ks-orch/p1-1.4-hotkey-tristate/red_commit3.log`), exactly as the cut predicted:
    - registry: ``assertion `left == right` failed: [Some("3"), Some("3")]`` / `left: Some(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] })` / `right: None`;
    - loader: ``assertion `left == right` failed`` / `left: 1` / `right: 0`.
  - Both failing lines are in the commit body. Replay point: commit `8ef0ab7` itself (or `5493b3a` plus the tests). On `master` the tests do not compile, because `hotkey_from_values` and `toggle_values` arrive in `5493b3a`.
- Roadmap 1.4 / KS-17, commit 4 (the fix): `registry.rs:97-106` makes the `Disabled` arm print `[INFO] {function:?}: hotkey not assigned in Windows, no auto-blocker` and return `None`. The doc comments at `:22-25` and `:79-86` say that it decides. Both commit-3 tests go green (`2 passed`). It also adds the manual `live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (`:481-490`), plus the README, template and CHANGELOG changes.
- Commit 5 (docs): `book/src/config/{matching,known-issues,errors,example}.md`.

## Remarks
- I1 (Important): RESOLVED in commit 2, `src/system/registry.rs:347-366`, by option (a). It is pinned by `the_language_hotkey_is_read_from_hotkey_then_language_hotkey`: `value_names == ["Hotkey", "Language Hotkey"]`, `subkey == "Keyboard Layout\\Toggle"` and `hkey == HKEY_CURRENT_USER` for both language functions. A swap at `registry.rs:35` fails it. Recorded in the developer addendum at the end of `cut.md`.
- M1 (Minor): RESOLVED in commit 5:
  - `book/src/config/matching.md:48`: a missing, unreadable or unknown value passes on to `Language Hotkey`, and a value that names a combination or says `"3"` stops the lookup;
  - `book/src/config/example.md:8`: the machine has no values under `HKCU\Keyboard Layout\Toggle`.
- Critic open question 2 (Layout Hotkey wording): the wording is adjusted (see Deviations). Open questions 1 and 3 are addressed to the architect, are informational, and need no code.

## Deviations from the cut
- **Extra assertion** in `a_language_hotkey_not_assigned_in_windows_has_no_combination` (`registry.rs:451-458`): `once(Some("3")).chain(repeat_with(panic))` → `None`. D2 says the values after a `Disabled` are not read either, and nothing else pins that.
- **SAFETY comment of `RegQueryValueExW`** (`registry.rs:187-190`): the moved comment gains the handle and name invariants ("key.0 is an open key handle (the guard closes it only on drop, and it is borrowed for this call); value_name_wide is null-terminated and outlives the call"). The function now takes `&HKeyGuard` instead of a raw `HKEY`.
- **`HKeyGuard` doc** (`:132-133`): it named the removed `read_value_and_parse`. It now says the guard closes the key on every path out of `get_system_hotkey`, a panic included.
- **Wording** (critic open question 2):
  - the template says "there is no language hotkey to block", not "nothing to block";
  - the CHANGELOG says "with no Windows language hotkey to protect";
  - the README sentence ends "neither value names a hotkey or says `"3"`", because `("3", missing)` names no hotkey and still gets no fallback.
- **Format args**: the two existing `[INFO]` lines use inline format args (`{function:?}`). The output is identical.
- **Book**: `example.md:8` was also edited (critique M1). It is outside the cut's touch set but inside `config/`.

## Unsafe / atomics added
- No new `unsafe` block, atomic or shared state. The two moved blocks:
  - `registry.rs:157-166` (`RegOpenKeyExW`): "subkey_wide is null-terminated and outlives the call; &mut hkey points to a stack-local that is valid for the duration of the call." (unchanged)
  - `registry.rs:187-200` (`RegQueryValueExW`): "key.0 is an open key handle (the guard closes it only on drop, and it is borrowed for this call); value_name_wide is null-terminated and outlives the call; data buffer is sized and writable; data_size carries its capacity in bytes per RegQueryValueExW's contract."
- Registry access stays read-only: `git diff master..HEAD` adds no `RegSetValue*`, `RegCreateKey*`, `RegDelete*`, `SystemParametersInfo*` or `SendInput`.

## Gates
- `cargo fmt --check`, `cargo check --all-targets --locked` and `cargo clippy --all-targets --locked -- -D warnings` pass on every commit and on HEAD. On commit 3, fmt and clippy pass and only the two declared tests are red.
- Tests run:
  - each commit: the named tests above, plus `cargo test --locked --lib -- system:: config::loader`;
  - HEAD: `cargo test --locked --lib -- config:: system:: template default_` gave `132 passed; 0 failed; 1 ignored`.
  - The full suite is left to the tester.
- Manual ignores added: `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. A human sets Settings > Time & language > Typing > Advanced keyboard settings > Input language hot keys > Switch Input Language to (Not assigned), runs `cargo test --locked -- --ignored live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (read-only), checks that it passes, and restores the setting.
- Manual checks (cut M1-M3, human only):
  - M1: Not assigned. `reg query "HKCU\Keyboard Layout\Toggle"` should show `Hotkey`, `Language Hotkey` and `Layout Hotkey`; record all three (critic open questions 1 and 2). With the daemon on the default config: the new `[INFO]` line, no `[AUTO-BLOCK]` line, nothing swallowed for Alt, Shift, X, and CapsLock switches the layout.
  - M2: Left Alt+Shift gives `[INFO] SwitchLanguage: registry combination Alt+Shift` and one `[AUTO-BLOCK] Alt + Shift` line.
  - M3: Ctrl+Shift gives the `Ctrl + Shift` blocker.

## Docs
- `CHANGELOG.md`: a `### Fixed` entry for KS-17 (roadmap 1.4), placed before KS-18.
- `README.md`: the `block_original_combo` row gains the registry source, the `"3"` rule and when the fallback applies.
- `src/config/default_config.ron`: three comment lines under the flag comment (ASCII). `default_config_parses`, the template-example tests and the template scenarios stay green.
- Book:
  - `matching.md` §15.2: the lookup rule, the `"3"` bullet, the skip and fallback bullet, and the console line;
  - `matching.md` §15.4: the warning becomes a fixed note;
  - `known-issues.md` row 40: Fixed;
  - `errors.md`: the new `[INFO]` line and its cause;
  - `example.md:8`.
- Book check: `mdbook build book -d D:/tmp/ks-orch/p1-1.4-hotkey-tristate/book` passes, and all 264 internal links and anchors of the four pages resolve. No new RON block, so no `cfgcheck` run was needed.
- Left for the phase book sync (cut Q3): `audit.md` status lines, `roadmap.md`, `architecture.md`, `overview.md`, `glossary.md`, `build-and-run.md`, `gaps.md`, and the `1dfd80a` permalinks in `matching.md:46`.

## Commits
- `06f385d` Tests: loader tests that count auto-blockers no longer read the registry (1.4)
- `5493b3a` System: the language hotkey parser returns Combo, Disabled or Unknown; the resolution takes injected values (1.4, KS-17)
- `8ef0ab7` System: red-first: a language hotkey not assigned in Windows gives no hotkey and no auto-blocker (1.4, KS-17)
- `38029df` System: "3" (not assigned) decides the language hotkey: no auto-blocker, no Alt+Shift fallback (1.4, KS-17)
- `9fcbc93` Book: the language hotkey not assigned in Windows in the config reference (1.4, KS-17)

## Open points / pre-existing warnings
- No pre-existing warnings under clippy `-D warnings`.
- Tests that still read the live registry, read-only: `test_get_system_hotkey_returns_combinations`, `system_function::tests::test_hardcoded_combinations` / `test_capslock_has_no_combination`, `loader::tests::default_config_parses` and `a_missing_config_beside_the_exe_is_generated_and_loaded`. Each accepts every machine state (critic open question 3).
- `test_default_combinations` (`registry.rs:493`) keeps its pre-existing `.unwrap()`; the cut did not touch it.
- `auto_blocker_not_added_without_flag` and `auto_blocker_not_added_for_non_system_actions` still call `from_ron_str`. They ask for no language blocker, so their results do not depend on the machine.
