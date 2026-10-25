# Context pack r1: p1-1.4-hotkey-tristate (roadmap 1.4, KS-17)

Readers: tester, code-reviewer. Facts only; `UNVERIFIED` marks what the scout did not check itself (in particular, the scout ran no cargo command).

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/language-hotkey-tristate`, HEAD `9fcbc93`, `git status --short` empty (clean).
- Parent: `master` = merge-base = `f28da35`. Round 1 red-first parent per `docs/WORKFLOW.md:69` is `master`; for this lane the red tests do not compile on `master` (see "Red-first" below).
- `git log --oneline master..fix/language-hotkey-tristate` (oldest last):
  - `9fcbc93` Book: the language hotkey not assigned in Windows in the config reference (1.4, KS-17)
  - `38029df` System: "3" (not assigned) decides the language hotkey: no auto-blocker, no Alt+Shift fallback (1.4, KS-17)
  - `8ef0ab7` System: red-first: a language hotkey not assigned in Windows gives no hotkey and no auto-blocker (1.4, KS-17)
  - `5493b3a` System: the language hotkey parser returns Combo, Disabled or Unknown; the resolution takes injected values (1.4, KS-17)
  - `06f385d` Tests: loader tests that count auto-blockers no longer read the registry (1.4)
- `git diff --stat master..fix/language-hotkey-tristate`: 9 files, +380 / -64:
  `CHANGELOG.md` 12, `README.md` 2, `book/src/config/errors.md` 2, `book/src/config/example.md` 2, `book/src/config/known-issues.md` 2, `book/src/config/matching.md` 11, `src/config/default_config.ron` 3, `src/config/loader.rs` 60, `src/system/registry.rs` 350.
- Per commit (`git log --stat`):
  - `06f385d`: `loader.rs` (4), `registry.rs` (13).
  - `5493b3a`: `loader.rs` (+37), `registry.rs` (+280/-52 incl. loader).
  - `8ef0ab7`: `loader.rs` (+19), `registry.rs` (+24). Test-only.
  - `38029df`: `CHANGELOG.md`, `README.md`, `default_config.ron`, `registry.rs` (34).
  - `9fcbc93`: the four `book/src/config/*.md` pages.
- Every commit body ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (`git log --format=%b`). The body of `8ef0ab7` carries both predicted failure excerpts.
- `git diff --check master..fix/language-hotkey-tristate`: clean.
- Lane inputs: `D:/tmp/ks-orch/p1-1.4-hotkey-tristate/cut.md` (CUT READY), `critique.md` (CHANGES_REQUESTED: I1 Important, M1 Minor), `impl.md` (DONE), `red_commit3.log`.

## File map (HEAD `9fcbc93`)

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/system/registry.rs:10-15` | `RegistryLocation`; `parser: fn(&str) -> HotkeySetting` (`:14`) | parser type changed from `Option<KeyCombination>` |
| `src/system/registry.rs:17-28` | private `enum HotkeySetting { Combo(KeyCombination), Disabled, Unknown }`, derives `Debug, Clone, PartialEq, Eq` | the tri-state of the spec; no `Display` |
| `src/system/registry.rs:30-63` | `get_registry_location`; language location `:32-39` (`HKEY_CURRENT_USER`, `"Keyboard Layout\\Toggle"`, `value_names: &["Hotkey", "Language Hotkey"]` at `:36`, `parser: parse_language_hotkey` `:37`); Lock/ShowDesktop `:41-53` use `parse_win_key_combo`; TaskManager/ToggleCapsLock `None` `:59-61` | value order decides precedence (I1) |
| `src/system/registry.rs:65-78` | `pub fn get_system_hotkey` (signature unchanged): opens key, passes lazy `value_names.iter().map(read_string_value)` to `hotkey_from_values` `:69-75`; else empty iterator `:77` | I/O side; only path that reads the registry |
| `src/system/registry.rs:80-116` | `pub(crate) fn hotkey_from_values<S: AsRef<str>>(function, values: impl IntoIterator<Item = Option<S>>) -> Option<KeyCombination>`; `.flatten()` skips `None` `:92`; `Combo` returns + `[INFO] ... registry combination` `:94-97`; `Disabled` returns `None` + `[INFO] {function:?}: hotkey not assigned in Windows, no auto-blocker` `:100-105`; `Unknown => {}` `:106`; fallback + `[INFO] ... using default combination` `:111-115` | the fix (KS-17); the pure, injectable resolution |
| `src/system/registry.rs:118-130` | `get_default_combination` (unchanged content); language default Alt+Shift `:120-122` | fallback target |
| `src/system/registry.rs:132-144` | `HKeyGuard` + `Drop` calling `RegCloseKey` (unsafe `:140`); doc `:132-133` reworded | R9 handle closing |
| `src/system/registry.rs:146-172` | `fn open_key(&RegistryLocation) -> Option<HKeyGuard>`; `RegOpenKeyExW` with `KEY_READ` (unsafe `:159-167`, SAFETY `:157-158`) | code moved from master's `read_from_registry` |
| `src/system/registry.rs:174-224` | `fn read_string_value(&HKeyGuard, &str) -> Option<String>`; `[u16; 128]` `:183`; byte size `:184`; `RegQueryValueExW` (unsafe `:191-200`, SAFETY `:187-190`, reworded); error -> `None` `:202-204`; type check + `[WARN]` `:208-214`; byte clamp + NUL scan `:217-223` | code moved from master's `read_value_and_parse`; R9 |
| `src/system/registry.rs:226-234` | `fn parse_language_hotkey(&str) -> HotkeySetting` on `value.trim()`: `"1"` Alt+Shift, `"2"` Ctrl+Shift, `"3"` `Disabled`, `"4"` `VK_OEM_3`, else `Unknown` | spec bullet 1 |
| `src/system/registry.rs:236-246` | `fn parse_win_key_combo(&str) -> HotkeySetting`: one letter -> `Combo(win_with(vk))`, else `Unknown`; never `Disabled` | parser type alignment |
| `src/system/registry.rs:285-557` | `#[cfg(test)] mod tests` (all tests listed under "Tests") | acceptance proofs |
| `src/config/loader.rs:213` | `pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>` | unchanged seam |
| `src/config/loader.rs:222-224` | `from_ron_str` passes `&BindAction::get_system_combination` (live registry) | unchanged |
| `src/config/loader.rs:231-240`, `:251` | `from_ron_str_with_hotkeys`, `parse_ron_str` (take the injected closure) | used by new tests |
| `src/config/loader.rs:338-352` | `expand_with_auto_blockers`; `let Some(hotkey) = system_combination(&binding.action) else { continue; }` `:350-352` | `None` -> no blocker, no MaskOnly warning; production code unchanged |
| `src/config/loader.rs:766` | test-only `use crate::system::registry::hotkey_from_values;` | new `config` -> `system` test import |
| `src/config/loader.rs:896`, `:969` | `parse_simple_binding` (fn `:884`) and `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (fn `:950`) now call `from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT)).unwrap()` | commit 1 hermeticity; `.unwrap()` kept from master |
| `src/config/loader.rs:997-1015` | existing helpers `language_hotkey` `:1000`, `blockers` `:1011`, `ALT_SHIFT` `:1015` | unchanged |
| `src/config/loader.rs:1017-1028` | new helper `toggle_values([Option<&'static str>; 2]) -> impl Fn(&BindAction) -> Option<KeyCombination>` via `to_system_function().and_then(hotkey_from_values)` | registry stand-in that runs the real resolution |
| `src/config/loader.rs:1030-1051` | new `the_registry_values_choose_the_auto_blocker_keys` | loader-level acceptance |
| `src/config/loader.rs:1053-1070` | new `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` | red-first, KS-17 scenario with the real template |
| `src/system/system_function.rs:38-40` | `get_system_combination` -> `registry::get_system_hotkey` | call chain |
| `src/core/windows_actions.rs:144`, `:155-158` | `BindAction::to_system_function`, `BindAction::get_system_combination` | call chain |
| `src/config/default_config.ron:172-174` | three new comment lines under the `block_original_combo` comment | template text; parsed by template tests |
| `README.md:539` | `block_original_combo` row gains the registry source and the `"3"` rule | config doc |
| `CHANGELOG.md:411-422` | new `- KS-17 (roadmap item 1.4): ...` entry, directly before `- KS-18:` (`:423`) | spec: CHANGELOG entry |
| `book/src/config/matching.md:48`, `:52-53`, `:55`, `:112-113` | §15.2 lookup rule, `"3"` bullet, console bullet; §15.4 note "Fixed on `fix/language-hotkey-tristate`" | book drift |
| `book/src/config/known-issues.md:90` | row 40 -> **Fixed** | book |
| `book/src/config/errors.md:178` | adds the new `[INFO]` line | book |
| `book/src/config/example.md:8` | machine described as having no values under `Toggle` (critique M1) | book |

## Facts

### Before (master `f28da35`)
1. `git show master:src/system/registry.rs`: `parse_language_hotkey` returns `Option<KeyCombination>` with `"3" => None` and `_ => None` (master `:181-189`).
2. Master's `get_system_hotkey` (`:52-65`) returns the registry combo from `read_from_registry`, else `get_default_combination` (Alt+Shift for language) with `[INFO] ... using default combination`.
3. Master's `read_from_registry` loops `value_names` and returns the first `Some` from `read_value_and_parse` (master `:120-124`), so `"3"` fell through to the next value and then the default.

### After (HEAD)
4. `HotkeySetting` is private (no `pub`) at `src/system/registry.rs:19`; it has no `Display` impl (Grep in file: none).
5. `value_names` for both language functions is `["Hotkey", "Language Hotkey"]` (`src/system/registry.rs:36`); one location serves `SwitchLanguage | SwitchLanguageBackward` (`:32`).
6. `hotkey_from_values` iterates `values.into_iter().flatten()` (`src/system/registry.rs:92`) and returns on the first `Combo` (`:94-97`) or `Disabled` (`:100-105`); `Unknown` continues (`:106`).
7. Functions without a registry location (`TaskManager`, `ToggleCapsLock`, `:59-61`) skip the loop (`if let Some(location)` `:91`) and get the default (`:111-115`); values are not consumed.
8. `get_system_hotkey` passes an empty iterator when there is no location or `open_key` fails (`src/system/registry.rs:77`), giving the fallback.
9. The value iterator in `get_system_hotkey` is lazy (`.map(|name| read_string_value(&key, name))`, `:71-74`) and is consumed while `key` (the guard) is alive (`:67-75`).
10. On commit `5493b3a` the arm was `HotkeySetting::Disabled | HotkeySetting::Unknown => {}` with the comment `// KS-17: "3" is still read past like an unknown value;` (`git show 5493b3a:src/system/registry.rs` `:97-99`); `38029df` splits it into the deciding `Disabled` arm.
11. `parse_language_hotkey` trims (`src/system/registry.rs:227`); `"3"` -> `Disabled` (`:230`).
12. `parse_win_key_combo` has no `Disabled` output (`src/system/registry.rs:236-246`).
13. `unsafe {` count in `registry.rs`: 3 on master, 3 on HEAD (`:140`, `:159`, `:191`); each preceded by a `// SAFETY:` comment (`:138-139`, `:157-158`, `:187-190`).
14. `git diff master..fix/language-hotkey-tristate` adds no line matching `RegSetValue|RegCreateKey|RegDelete|SystemParametersInfo|SendInput` (scout grep of `^\+` lines). Added `.unwrap()` lines: only `loader.rs:896` and `:969` (both replaced an existing `from_ron_str(ron_text).unwrap()`).
15. Only product caller of `BindAction::get_system_combination`: `src/config/loader.rs:223` (Grep over `src/`); other callers are tests in `src/system/system_function.rs:183-202`.
16. `hotkey_from_values` is referenced outside `registry.rs` only in `src/config/loader.rs:766` and `:1026` (test module).
17. `src/system/mod.rs:1` is `pub mod registry;` (the `pub(crate)` fn is reachable as `crate::system::registry::hotkey_from_values`).
18. `KeyCombination::from_keys` takes `impl IntoIterator<Item = K: Into<KeySpec>>` (`src/data/key_combination.rs:50`); tests pass arrays, production passes `vec!`.
19. Roadmap 1.4 acceptance text: "(`Hotkey`, `Language Hotkey`) = ("3", "3") give no blocker; ("2", anything) gives Ctrl+Shift; no values give Alt+Shift" (`book/src/roadmap.md:269`). The `("1", anything) -> Alt+Shift` line comes from the lane brief, not the roadmap.
20. Book status lines not touched in this lane: `book/src/audit.md:40` ("Open: item 1.4"), `book/src/roadmap.md:621` ("Open"); cut Q3 leaves them to the phase book sync.
21. Registry readers run on the main thread at startup (`src/main.rs:30`) and on `config-watcher` for reloads (`src/hook/config_watcher.rs:159`) (cut.md line 33; UNVERIFIED by the scout this round).

### Line-number drift between impl.md and HEAD (scout check)
22. impl.md cites `hotkey_from_values` at `registry.rs:79-117` and the `Disabled` arm at `:97-106`; at HEAD they are `:80-116` and `:98-105`. impl.md cites `read_string_value` at `:177-217`; at HEAD it spans `:177-224`. impl.md cites `test_get_system_hotkey_returns_combinations` at `:462-479`; at HEAD the `#[test]` is `:461`, fn `:462`. Other impl.md ranges checked match.

## Tests (HEAD `src/system/registry.rs` and `src/config/loader.rs`)

Helpers: `language([Option<&str>; 2])` resolves for both language functions and asserts they agree (`registry.rs:304-309`); `ANYTHING` = `[None, "1", "2", "3", "4", "", "x"]` (`:312-320`).

| test | file:line | commit | what it asserts |
|---|---|---|---|
| `parse_language_hotkey_is_tri_state` | `registry.rs:322-345` | 5493b3a | "1"/"2"/"4" -> Combo; "3", " 3 " -> Disabled; "", "0", "5", "33", "unknown", "Alt+Shift" -> Unknown |
| `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` | `registry.rs:347-366` | 5493b3a | both language functions: `hkey == HKEY_CURRENT_USER`, `subkey == "Keyboard Layout\\Toggle"`, `value_names == ["Hotkey","Language Hotkey"]` (critique I1, option a) |
| `the_hotkey_value_decides_before_the_language_hotkey_value` | `registry.rs:368-378` | 5493b3a | `("2", ANYTHING)` -> Ctrl+Shift; `("1", ANYTHING)` -> Alt+Shift |
| `no_usable_value_falls_back_to_alt_shift` | `registry.rs:380-393` | 5493b3a | `[None,None]`, `["", "x"]`, `["5", None]`, empty iterator -> Alt+Shift |
| `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide` | `registry.rs:395-400` | 5493b3a | `[None,"2"]` Ctrl+Shift; `["x","4"]` grave; `[None,"1"]` Alt+Shift |
| `the_values_after_the_deciding_one_are_not_read` | `registry.rs:402-412` | 5493b3a | `once("2").chain(repeat_with(panic))` -> Ctrl+Shift |
| `other_functions_keep_their_rules` | `registry.rs:414-435` | 5493b3a | CapsLock `["2"]` -> None; TaskManager `["3"]` -> Ctrl+Shift+Esc; Lock `["3"]` -> Win+L; Lock `["K"]` -> Win+K |
| `a_language_hotkey_not_assigned_in_windows_has_no_combination` | `registry.rs:437-459` | 8ef0ab7 (red) | rows `["3","3"]`, `["3",None]`, `[None,"3"]`, `["3","2"]`, `["x","3"]`, `[" 3 ","1"]` -> None; plus `once("3").chain(repeat_with(panic))` -> None (`:451-458`, impl deviation 1) |
| `test_get_system_hotkey_returns_combinations` | `registry.rs:461-479` | 06f385d | live read-only: SwitchLanguage is None or Alt+Shift/Ctrl+Shift/grave (`Option::is_none_or`); Lock/ShowDesktop/TaskManager Some; CapsLock None |
| `live_registry_with_the_language_hotkey_not_assigned_has_no_combination` | `registry.rs:481-490` | 38029df | `#[ignore = "manual: ..."]` (`:482`); both language lookups None. Manual, never run by agents |
| `test_default_combinations` | `registry.rs:492-505` | master | unchanged; uses `.unwrap()` (`:494`, `:502`) |
| `language_hotkeys_are_either_side_modifiers` | `registry.rs:507-524` | 5493b3a (rewritten) | `let HotkeySetting::Combo(..) = .. else panic!` |
| `test_parse_win_key_combo` | `registry.rs:534-556` | 5493b3a (rewritten) | adds "1", "", "LL" -> Unknown |
| `parse_simple_binding` | `loader.rs:884` (call `:896`) | 06f385d | now hermetic (ALT_SHIFT stand-in) |
| `auto_blocker_deduped_when_two_bindings_want_the_same_combo` | `loader.rs:950` (call `:969`) | 06f385d | now hermetic |
| `the_registry_values_choose_the_auto_blocker_keys` | `loader.rs:1030-1051` | 5493b3a | `DEFAULT_CONFIG` with `["2",None]` -> exactly `[Some(Ctrl+Shift)]` blocker; `[None,None]` and `["1","3"]` -> `[Some(Alt+Shift)]` |
| `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` | `loader.rs:1053-1070` | 8ef0ab7 (red) | `DEFAULT_CONFIG` with `["3","3"]`: 0 blockers, 2 bindings; one-binding `MaskOnly` RON: 0 blockers, `warnings` empty |

Tests that still read the live registry (read-only), per impl.md "Open points": `test_get_system_hotkey_returns_combinations`, `system_function::tests::test_hardcoded_combinations` (`system_function.rs:180`), `test_capslock_has_no_combination`, `loader::tests::default_config_parses` (`loader.rs:771`), `a_missing_config_beside_the_exe_is_generated_and_loaded` (`loader.rs:2477`). `auto_blocker_not_added_without_flag` (`loader.rs:916`) and `auto_blocker_not_added_for_non_system_actions` (`:931`) still call `from_ron_str`.

## Red-first and acceptance proofs (from cut.md and impl.md)

### Red-first replay
- Declared red commit: `8ef0ab7` (cut.md commit 3; impl.md). Replay point: the tree of `8ef0ab7` itself, or `5493b3a` plus the two tests. On `master` (`f28da35`) the two tests do not compile because `hotkey_from_values` and `toggle_values` arrive in `5493b3a` (impl.md; critique says E0425). Critique: "The tester's brief must carry it, because WORKFLOW.md:69 defaults to `master`."
- WORKFLOW rule (`docs/WORKFLOW.md:69-70`): replay in a scratch worktree under `wt/` with its own `CARGO_TARGET_DIR` under `target-wt/`; at least two own mutations.
- Command (UNVERIFIED as exact; `red_commit3.log` does not record the command line, it shows `running 2 tests` and `479 filtered out`): `cargo test --locked --lib -- a_language_hotkey_not_assigned_in_windows`
- Expected on `8ef0ab7` (recorded in `red_commit3.log:5-6,14-17,23-26,33` and in the `8ef0ab7` commit body):
  - `system::registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination ... FAILED`, panic at `src\system\registry.rs:442:13`: ``assertion `left == right` failed: [Some("3"), Some("3")]`` / `left: Some(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] })` / `right: None`.
  - `config::loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker ... FAILED`, panic at `src\config\loader.rs:1060:9`: ``assertion `left == right` failed`` / `left: 1` / `right: 0`.
  - `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 479 filtered out`.
- Expected on `38029df` and HEAD: same command, `2 passed` (impl.md, UNVERIFIED by scout).
- Cut: on commit 3 `cargo fmt --check` and clippy are green; only those two tests are red (cut.md commit 3; impl.md "Gates").

### Acceptance mapping (cut.md "Acceptance mapping")
| acceptance line | proof |
|---|---|
| parser returns Combo / Disabled / Unknown | `registry::tests::parse_language_hotkey_is_tri_state` |
| ("3","3") -> no blocker | `registry::...a_language_hotkey_not_assigned_in_windows_has_no_combination` row 1; `config::loader::...a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker`; manual M1 |
| Disabled -> no Alt+Shift fallback | same registry test, rows `["3",None]`, `[None,"3"]`, `["3","2"]` |
| ("2", anything) -> Ctrl+Shift | `the_hotkey_value_decides_before_the_language_hotkey_value`; loader row `["2",None]`; manual M3 |
| ("1", anything) -> Alt+Shift | same registry test; loader row `["1","3"]`; manual M2 |
| no values -> Alt+Shift | `no_usable_value_falls_back_to_alt_shift` (absent key = empty iterator); loader row `[None,None]` |
| only missing/unparsable falls back | `no_usable_value_falls_back_to_alt_shift`, `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide` |
| registry read-only; injectable | D3/D4; Fact 14 grep |
| `Hotkey` before `Language Hotkey` by name | `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` (I1) |
| CHANGELOG entry | `CHANGELOG.md:411-422` |

### Commands recorded by the developer (impl.md "Gates"; results UNVERIFIED by scout)
- Every commit: `cargo fmt --check`, `cargo check --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings` pass (commit 3: fmt + clippy pass, two declared reds).
- Per commit: the named tests plus `cargo test --locked --lib -- system:: config::loader`.
- HEAD: `cargo test --locked --lib -- config:: system:: template default_` -> `132 passed; 0 failed; 1 ignored`.
- Full `cargo test --locked`: not run by the developer; left to the tester.
- Book: `mdbook build book -d D:/tmp/ks-orch/p1-1.4-hotkey-tristate/book` passed; 264 internal links/anchors of the four pages resolve (impl.md). No new ```ron block, no `cfgcheck` run.

### Manual checks (human only; cut.md "Manual checks", impl.md)
- M1: Windows "Switch Input Language" = Not assigned; `reg query "HKCU\Keyboard Layout\Toggle"` (record `Hotkey`, `Language Hotkey`, `Layout Hotkey`); `cargo test --locked -- --ignored live_registry_with_the_language_hotkey_not_assigned_has_no_combination` passes; daemon on default config prints the new `[INFO]` line and no `[AUTO-BLOCK]`; Alt, Shift, X not swallowed; CapsLock switches; restore setting.
- M2: Left Alt+Shift -> `[INFO] SwitchLanguage: registry combination Alt+Shift` and one `[AUTO-BLOCK] Alt + Shift`.
- M3: Ctrl+Shift -> `[INFO] SwitchLanguage: registry combination Ctrl+Shift`; blocker `Ctrl + Shift`.
- These are manual checks, not passes (CLAUDE.md).

## Open points (impl.md "Open points / pre-existing warnings", "Deviations", "Remarks")
1. No pre-existing warnings under clippy `-D warnings`.
2. Live-registry tests remaining (read-only), each said to accept every machine state: `test_get_system_hotkey_returns_combinations`, `test_hardcoded_combinations`, `test_capslock_has_no_combination`, `default_config_parses`, `a_missing_config_beside_the_exe_is_generated_and_loaded` (critic open question 3).
3. `test_default_combinations` (`registry.rs:493-494`, `:502`) keeps its pre-existing `.unwrap()`; the cut did not touch it.
4. `auto_blocker_not_added_without_flag` / `auto_blocker_not_added_for_non_system_actions` still call `from_ron_str`; they request no language blocker.
5. Deviations from the cut: (a) extra laziness assertion after `Disabled` (`registry.rs:451-458`); (b) `RegQueryValueExW` SAFETY comment reworded and the function takes `&HKeyGuard` (`:177`, `:187-190`); (c) `HKeyGuard` doc reworded (`:132-133`); (d) wording: template "there is no language hotkey to block" (`default_config.ron:172-174`), CHANGELOG "with no Windows language hotkey to protect" (`CHANGELOG.md:415`), README "neither value names a hotkey or says `"3"`" (`README.md:539`); (e) inline format args in the two existing `[INFO]` lines (`registry.rs:95`, `:113`); (f) `book/src/config/example.md:8` edited (outside the cut's touch set, inside `config/`).
6. Critique I1 resolved in `5493b3a` by `the_language_hotkey_is_read_from_hotkey_then_language_hotkey`; M1 resolved in `9fcbc93` (`matching.md:48`, `example.md:8`).
7. Critic open questions 1 and 3 addressed to the architect, informational: Q1 which value Windows honours when `Hotkey` and `Language Hotkey` differ is unverified (cut Q1; critique OQ1 cites ReactOS reading only `Language Hotkey`); `Layout Hotkey` is not read (critique OQ2; non-goal, roadmap feature #7).
8. Left for the phase book sync (cut Q3): `audit.md` status lines, `roadmap.md`, `architecture.md`, `overview.md`, `glossary.md`, `build-and-run.md`, `gaps.md`, and the `1dfd80a` permalinks in `matching.md:46`.
9. Cut follow-up F1 (not in lane): an elevated daemon running as another account reads that account's `HKCU` (cut risk 9).

## Invariants in play
- R9 registry reads: guard closes subkey on every path (`registry.rs:134-144`, `:148-172`, `:177`); value type checked (`:208-214`); byte-length clamp (`:184`, `:217-218`); `"3"` meaning disabled (`:230`, `:100-105`); KS-17.
- R6 Display: `HotkeySetting` is private with `Debug` only (`registry.rs:18-19`); no user-facing enum gains a variant (cut D1, risk 4).
- R7 specificity / auto-blockers: blocker presence decided in `loader.rs:350-352`; `update_bindings` sort unchanged (not in diff); KS-05, KS-18 context (`CHANGELOG.md:423`).
- R8 key identity: language combos stay `AnySide` (`registry.rs:228-231`, test `:507-524`).
- R1 hook callback, R2 sentinel, R3 shared state, R4 orderings, R5 builders, R10 message loop: no file of `src/hook`, `src/engine`, `src/core/app.rs` or `src/core/constants.rs` is in the diff (`git diff --stat`); no `SendInput` added (Fact 14).
- CLAUDE.md principle 8: `expect("invariant: ...")` instead of `unwrap()`; added `.unwrap()` at `loader.rs:896`, `:969` (tests, carried from master's lines); `unwrap_or_else(|| panic!(..))` at `registry.rs:357`.
- CLAUDE.md principle 9 / red-first: `8ef0ab7` precedes the fix `38029df`.
- Test hermeticity: tests must not write the registry; only read-only lookups remain (Open point 2).

## Not covered
- The scout ran no cargo command (no fmt, clippy, test, mdbook); all gate results above are as reported in impl.md.
- Not read this round: `src/engine/**`, `src/hook/**`, `src/main.rs`, `src/config/parsing.rs`, `src/config/schema.rs`, `src/data/binding.rs` beyond `:174`/`:193`, the full `default_config.ron`, the rest of `loader.rs` tests (outside the lines cited), the full text of the four book pages (only changed lines located), and the doc lines of `CHANGELOG.md` outside `:409-423`.
- Windows' own precedence between `Hotkey` and `Language Hotkey`: not researched by the scout.
- The intermediate trees `06f385d`, `5493b3a` were inspected only for Fact 10; per-commit green status is not verified by the scout.
