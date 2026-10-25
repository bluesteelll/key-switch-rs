# Cut: p1-1.4-hotkey-tristate — roadmap item 1.4 (tri-state language hotkey), KS-17

Tree: `master` @ `f28da35` (merge of `feat/mask-key`; `.git/logs/refs/heads/master:22`). Branch to cut: `fix/language-hotkey-tristate`. Every `path:line` below is at `f28da35`.

## Verdict

CUT READY

Four non-blocking questions at the end (Q1-Q4), each with a recommendation. None changes the commits.

## Spec re-located

| Spec reference (book) | Current file:line | Status | Note |
|---|---|---|---|
| audit KS-17 "`parse_language_hotkey` maps "3" to `None`", `registry.rs:196` @7cb3521 | `src/system/registry.rs:185` (`"3" => None,`) | moved | **Defect present.** Same result as an unknown value (`:187` `_ => None`) |
| audit KS-17 `parse_language_hotkey`, `registry.rs:192-199` @7cb3521 | `src/system/registry.rs:181-189` | moved | |
| audit KS-17 "`read_from_registry` treats `None` as nothing here and tries the next value name", `registry.rs:131-137` @7cb3521 | `src/system/registry.rs:120-124` (loop in `read_from_registry`, `:95-127`) | moved | Defect present |
| audit KS-17 / roadmap 1.4 "`get_system_hotkey` then falls back to the default", `registry.rs:68-72` / `:60-72` @7cb3521 | `src/system/registry.rs:52-65` (fallback `:60-64`) | moved | Defect present |
| audit KS-17 "the default is Alt+Shift", `registry.rs:77-78` @7cb3521 | `src/system/registry.rs:69-71` (`get_default_combination`, `:67-79`) | moved | Unchanged by this lane |
| gaps.md:119 value names `Hotkey`, `Language Hotkey`, `registry.rs:24-29` @7cb3521 | `src/system/registry.rs:19-26` (`value_names` `:23`, `parser` `:24`); struct field `parser: fn(&str) -> Option<KeyCombination>` `:14` | moved | Order `Hotkey` then `Language Hotkey` is kept (D2) |
| matching.md:46 / glossary.md:287 / overview.md:23,39 cite `registry.rs:17-79`, `:181-189`, `:17-26` @1dfd80a | same lines | unchanged since `1dfd80a` | Phase book sync re-pins links |
| matching.md:112, known-issues.md:90 cite `registry.rs:185`, `:60-64` @1dfd80a | same lines | unchanged | Commit 5 edits both pages |
| Value read: type check + `[WARN]`, byte-length clamp, `[u16; 128]` buffer | `src/system/registry.rs:129-179` (buffer `:139`, call `:145-154`, error `:156-158`, type check `:160-168`, clamp `:170-177`) | — | Moved verbatim into `read_string_value` (D4) |
| Key handle guard | `src/system/registry.rs:83-93` (`HKeyGuard`), open `:102-118` | — | Kept |
| Loader: `None` from the hotkey lookup means "no blocker" | `src/config/loader.rs:350-352` (`let Some(hotkey) = system_combination(&binding.action) else { continue; }`) in `expand_with_auto_blockers` `:338-401` | — | Already correct; loader production code untouched (D5) |
| Loader's registry seam | `src/config/loader.rs:211-213` (`SystemCombination`), `:222-224` (`from_ron_str` passes `BindAction::get_system_combination`), `:231-240` (`from_ron_str_with_hotkeys`) | — | |
| Call chain to the registry | `src/core/windows_actions.rs:155-158` → `src/system/system_function.rs:38-40` → `registry::get_system_hotkey` | — | Only caller of `BindAction::get_system_combination` in product code: `loader.rs:223` (Grep) |
| Threads that read the registry | startup: `src/main.rs:30` (`config::load`, main thread, before `App` is built at `:32-36`); reload: `src/hook/config_watcher.rs:159` on the `config-watcher` thread (`:123-126`) | — | Never the hook thread |
| Tests that read the live registry and count a language blocker | `src/config/loader.rs:882-912` (`parse_simple_binding`, asserts 2 bindings), `:948-972` (`auto_blocker_deduped_when_two_bindings_want_the_same_combo`, asserts 3); `src/system/registry.rs:253-260` (`test_get_system_hotkey_returns_combinations`, asserts `SwitchLanguage` is `Some`) | — | Would fail after the fix on a machine with `"3"`; commit 1 makes them hermetic |
| Existing parser tests on `Option` | `src/system/registry.rs:244-251`, `:277-292`, `:302-312` | — | Rewritten for the tri-state in commit 2 |
| Existing stand-in for the registry in tests | `src/config/loader.rs:997-1007` (`language_hotkey`), `:1009-1012` (`blockers`), `:1014` (`ALT_SHIFT`); `src/engine/testing.rs:184-196` (`from_ron_with_language_hotkey`) | — | Engine scenarios never read the registry: every `block_original_combo` in `scenarios.rs` goes through `from_ron_with_language_hotkey` or a hand-built blocker (`scenarios.rs:32-52`) |
| roadmap 1.4 Files: `src/system/registry.rs` | — | — | Product code stays in that file; tests also touch `loader.rs` |

KS issues read: KS-17 (`audit.md:450-463`), KS-05 header and fix text (`audit.md:236-241`, the breakage a wrong blocker causes), KS-18 (`audit.md:465-478`, dedup context).

## Goal and non-goals

**After the lane.**
- `parse_language_hotkey(&str) -> HotkeySetting` with `Combo(KeyCombination)` (`"1"`, `"2"`, `"4"`), `Disabled` (`"3"`) or `Unknown` (anything else).
- Resolution over the values `Hotkey`, `Language Hotkey`, in that order: the first value that is `Combo` or `Disabled` decides. `Disabled` gives no hotkey, so `block_original_combo` adds no auto-blocker and there is no `Alt+Shift` fallback. Only when no value decides (missing, unreadable, not a string, unknown text, or the key absent) is the default `Alt+Shift` used.
- The resolution is a pure function over injected strings, so the acceptance lines are unit tests. The registry is only read.
- New console line: `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker`.

**Non-goals.**
- `Layout Hotkey` and Win+Space (roadmap "New features" #7, `roadmap.md:560`, `:569`).
- Removing the unreal `LockWorkstationHotkey`/`ShowDesktopHotkey` lookups (feature #1, `roadmap.md:563`). They keep working as today: `Combo` or `Unknown`.
- Re-reading the registry when the Windows setting changes; it is still read on every config load (`matching.md:55`).
- Reading the interactive user's hive when the daemon is elevated as another account (risk 9, follow-up F1).
- A `[WARN]` for an unknown value text (Q2).
- No engine, hook, IPC or schema change. No RON change.

**Later items that depend on this.** None in the roadmap graph (`roadmap.md:219`: depends on none; nothing depends on 1.4). Feature #7 will extend the same `HotkeySetting` for `Layout Hotkey`.

## Decisions

### D1: A private tri-state `HotkeySetting`, shared by both registry parsers

- **What.** In `src/system/registry.rs`:
  ```rust
  /// What a registry value says about a Windows hotkey (KS-17).
  #[derive(Debug, Clone, PartialEq, Eq)]
  enum HotkeySetting {
      /// The value names this combination.
      Combo(KeyCombination),
      /// The value turns the hotkey off: "3", "Not assigned" in Windows' settings.
      Disabled,
      /// The value is a string this parser does not know.
      Unknown,
  }
  ```
  - `RegistryLocation::parser` (`registry.rs:14`) becomes `fn(&str) -> HotkeySetting`.
  - `parse_language_hotkey(value: &str) -> HotkeySetting`, on `value.trim()`: `"1"` → `Combo(from_keys([VK_MENU, VK_SHIFT]))`, `"2"` → `Combo(from_keys([VK_CONTROL, VK_SHIFT]))`, `"3"` → `Disabled`, `"4"` → `Combo(KeyCombination::new(VK_OEM_3))`, anything else → `Unknown`.
  - `parse_win_key_combo(value: &str) -> HotkeySetting`: `Combo(win_with(vk))` for one letter A-Z (today's rule, `:191-201`), else `Unknown`. It never yields `Disabled`.
- **Why.** The spec names the three states. One type for both parsers keeps `RegistryLocation` uniform, and the `Disabled` variant is constructed, so there is no `dead_code` under `clippy -D warnings`. Private: nothing outside `registry.rs` needs it. No `Display`: it is never shown to a user (R6 concerns user-facing `Display`); `Debug` serves test failures.
- **Rejected.** `Option<Option<KeyCombination>>`: unreadable, and the spec names the states. A separate `LanguageHotkey` enum for one parser only: two parser types in one table.
- **Cost.** None.

### D2: The first deciding value wins, in the order `Hotkey`, `Language Hotkey`

- **What.** Values are taken in `value_names` order (`registry.rs:23`, unchanged). A value that is `Combo` or `Disabled` decides, and the values after it are not read. A missing value (`None`) and an `Unknown` one are skipped. If no value decides, `get_default_combination(function)` applies.
- **Why.** The acceptance line `("2", anything) → Ctrl+Shift` makes `Hotkey` the higher-priority value. The same rule applied to `"3"` gives `("3", anything) → Disabled` and `("3", "3") → no blocker`. "Only a missing or unparsable value falls back" is literally the skip rule. A wrong `Disabled` costs only the pre-flag behaviour (Windows also switches on its hotkey), while a wrong blocker costs KS-05's swallowed Shift, so the rule errs on the cheaper side. Laziness keeps today's reads: today the loop stops at the first value that parses (`:120-124`), so a non-string `Language Hotkey` behind a valid `Hotkey` still prints no `[WARN]`.
- **Rejected.** (a) "`Disabled` only when every present value says `"3"`" (`("3","2")` → Ctrl+Shift): asymmetric with `("2","3")` → Ctrl+Shift, and it still needs a precedence rule. (b) `Language Hotkey` first: contradicts the acceptance line. Which value Windows 10/11 itself honours when they disagree is unverified (Q1).
- **Cost.** A hand-edited `("3", "1")` gives no blocker although `Language Hotkey` names Alt+Shift (risk 7).

### D3: The resolution is a pure function over injected strings

- **What.** Final form (commit 4; commit 2 lands it with `Disabled` skipped):
  ```rust
  /// The hotkey of `function`, given the data of its registry values in the
  /// order of the location's `value_names`: `Some(text)` for a value read as
  /// a string, `None` for one that is missing, unreadable or not a string.
  /// The first value whose text is `Combo` or `Disabled` decides; the values
  /// after it are not read. With no deciding value, or for a function
  /// without a registry location, the built-in default. Prints the [INFO]
  /// line that says where the hotkey came from.
  pub(crate) fn hotkey_from_values<S: AsRef<str>>(
      function: SystemFunction,
      values: impl IntoIterator<Item = Option<S>>,
  ) -> Option<KeyCombination>
  ```
  - A function without a location (`TaskManager`, `ToggleCapsLock`, `registry.rs:46-48`) ignores `values` and returns its default (Ctrl+Shift+Esc, `None`).
  - Outcomes and their lines (`function` printed with `{:?}`):

    | Outcome | Returns | Line |
    |---|---|---|
    | a value is `Combo(c)` | `Some(c)` | `[INFO] {function:?}: registry combination {c}` (unchanged, `:56`) |
    | a value is `Disabled` | `None` | `[INFO] {function:?}: hotkey not assigned in Windows, no auto-blocker` (new, commit 4) |
    | no value decides, default `Some` | the default | `[INFO] {function:?}: using default combination` (unchanged, `:62`) |
    | no value decides, default `None` | `None` | none (unchanged) |
  - `get_system_hotkey(function) -> Option<KeyCombination>` keeps its `pub` signature (`:52`). It opens the key and passes `hotkey_from_values` a lazy iterator, `location.value_names.iter().map(|name| read_string_value(&key, name))`, while the guard is alive. With no location, or a key that cannot be opened, it calls `hotkey_from_values(function, std::iter::empty::<Option<String>>())`.
- **Why.** The spec asks for resolution "testable with injected values" and a read-only registry. Strings are the boundary the registry hands over, so tests inject exactly what `reg query` shows. Generic `S` lets tests pass `&str` and production pass `String` with no conversion allocation. `pub(crate)` lets the loader test wire it into `from_ron_str_with_hotkeys` (commit 2).
- **Rejected.** (a) A `RegistrySource` trait with a fake: more surface for two strings. (b) Injecting only at the loader closure (already exists, `loader.rs:231-240`): does not exercise the registry rule. (c) Returning an enum and printing in `get_system_hotkey`: the loader-level test would need a second conversion, and printing to captured stdout in tests costs nothing.
- **Cost.** `[INFO]` lines appear in captured test output.

### D4: I/O split into `open_key` and `read_string_value`, code moved verbatim

- **What.** `read_from_registry` (`:95-127`) and `read_value_and_parse` (`:129-179`) are replaced by:
  - `fn open_key(location: &RegistryLocation) -> Option<HKeyGuard>`: today's `:96-118` (wide subkey, `RegOpenKeyExW` with `KEY_READ`, `None` on error, guard on success).
  - `fn read_string_value(key: &HKeyGuard, value_name: &str) -> Option<String>`: today's `:134-177` (wide name, `[u16; 128]` buffer, byte size, `RegQueryValueExW`, `None` on any error, the `REG_SZ`/`REG_EXPAND_SZ` check with the same `[WARN]`, clamp to the bytes written, stop at the first NUL, `from_utf16_lossy`). It returns the string instead of parsing it.
  - Both `unsafe` blocks keep their `// SAFETY:` comments; the guard closes the key on every path.
- **Why.** Separates the untestable I/O from the decision (D3) without changing anything that is read or printed.
- **Rejected.** `RegGetValueW` (null-termination guaranteed): a new API for no behaviour gain; today's clamp-and-scan already handles a missing NUL (`:170-176`).
- **Cost.** One `String` allocation per value read, on the main or `config-watcher` thread only.

### D5: The loader's production code is untouched

- **What.** `expand_with_auto_blockers` already skips a binding whose hotkey lookup is `None` (`loader.rs:350-352`), so `Disabled → None` yields no blocker, no `MaskOnly` warning and no `[INFO] no auto-blocker for ...: binding #N uses the same keys` line.
- **Why.** The KS-17 defect lies entirely in the registry reader (audit "Why"); the `Option` contract between loader and system is already right.
- **Rejected.** Passing `HotkeySetting` up to the loader so it can print its own line: widens the `SystemCombination` type (`loader.rs:213`) and every test closure for one message the registry can print.

### D6: Tests that count a language blocker stop depending on the machine (commit 1)

- **What.** `parse_simple_binding` and `auto_blocker_deduped_when_two_bindings_want_the_same_combo` call `from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT))` instead of `from_ron_str`. `test_get_system_hotkey_returns_combinations` keeps its live, read-only lookups for Lock, ShowDesktop, TaskManager and CapsLock. For `SwitchLanguage` it asserts that the result is `None` or one of Alt+Shift, Ctrl+Shift, `` ` `` (`Option::is_none_or`, stable since Rust 1.82; MSRV is 1.88).
- **Why.** After the fix, a developer or CI machine whose `Toggle` values say `"3"` would fail those three tests for reasons unrelated to the code. The engine's tests already follow this rule (`testing.rs:173-177`).
- **Rejected.** A `#[cfg(test)]` branch in `from_ron_str` that fakes the hotkey: hides the real seam and changes production code for tests.

### D7: No engine scenario test

- **What.** The lane adds unit tests in `registry.rs` and `loader.rs` only.
- **Why.** No per-key decision changes. The engine receives the binding list with or without a blocker, and its behaviour is already pinned for both. With the blocker: `scenarios.rs:2162`, `:2195`, `:2477`. Without one, keys no binding matches pass, across random scripts of the `blocker: None` configs (`properties.rs:100-168`). The KS-17 scenario ("default config + Not assigned → an Alt+Shift blocker") is reproduced at the loader with the real template (`a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker`).
- **Rejected.** A scenario that loads the template through `toggle_values`: it adds a test-only `engine` → `system` import to re-prove "no binding matches, so the key passes".

## Win32 API choices

No new API and no new Cargo feature. The existing calls move unchanged.

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Open `HKCU\Keyboard Layout\Toggle` | `windows::Win32::System::Registry::RegOpenKeyExW` (as called at `registry.rs:106-114`, which compiles on `f28da35`), `KEY_READ` | `Win32_System_Registry`, present (`Cargo.toml:49`) | main (startup), `config-watcher` (reload) | `WIN32_ERROR`; `.is_err()` → no values → default | https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regopenkeyexw |
| Read one value | `windows::Win32::System::Registry::RegQueryValueExW` (`registry.rs:145-154`) | same | same as the open | any error → `None` (skipped). Doc: "If the *lpValueName* registry value does not exist, the function returns ERROR_FILE_NOT_FOUND"; "If the *lpData* buffer is too small to receive the data, the function returns ERROR_MORE_DATA"; the size is "in bytes" | https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw |
| String may lack a NUL | (same call) | — | — | handled by the clamp and scan (`:170-176`). Doc: "the string may not have been stored with the proper terminating null characters" | same page, Remarks |
| Close the subkey | `RegCloseKey` in `HKeyGuard::drop` (`:85-93`) | same | the opening thread | ignored | https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regclosekey |
| Meaning of the values | none: Windows documents no getter; `SystemParametersInfoW` documents only `SPI_SETLANGTOGGLE` | — | — | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow — "the path in the registry is HKEY_CURRENT_USER\Keyboard Layout\Toggle. Valid values are "1" = ALT+SHIFT, "2" = CTRL+SHIFT, and "3" = none." (`"4"`, the grave accent, is not on that page; kept as today) |
| `HKCU` mapping | predefined `HKEY_CURRENT_USER` (never closed by us) | — | any | — | https://learn.microsoft.com/en-us/windows/win32/sysinfo/predefined-keys — "The mapping is based on the security context of the first thread to reference HKEY_CURRENT_USER"; "The predefined handles are not thread safe. Closing a predefined handle in one thread affects any other threads that are using the handle." We close only the subkey we opened. |

Threading: no hook, window, timer or COM apartment is touched. Reads run on the main thread at startup (`main.rs:30`, before `App` installs anything) and on `config-watcher` for reloads (`config_watcher.rs:159`). They never overlap, because the watcher starts after the startup load (`main.rs:30-32`). `SendInput`, `LLKHF_INJECTED`, the extended-key flag and `scanCode` play no part in this lane.

## Touch set

| File | Change | Why |
|---|---|---|
| `src/system/registry.rs` | `HotkeySetting`; `parser` type; tri-state parsers; `open_key`, `read_string_value`; `hotkey_from_values`; `get_system_hotkey` over them; the `Disabled` arm decides (commit 4); tests | KS-17 lives here (roadmap 1.4 Files) |
| `src/config/loader.rs` | tests only: two tests made hermetic (commit 1); helper `toggle_values` and `the_registry_values_choose_the_auto_blocker_keys` (commit 2); `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` (commit 3) | Acceptance "gives no blocker" end to end, with the real template |
| `README.md` | one sentence in the `block_original_combo` row (`README.md:539`) | Config meaning changes on `"3"` machines (commit 4) |
| `src/config/default_config.ron` | three comment lines after `:171` | Same (commit 4); comment-only, above the examples section the tests parse (`loader.rs:788-799`) |
| `CHANGELOG.md` | `### Fixed` entry before the KS-18 entry (`CHANGELOG.md:411`) | Spec; that list is in KS order |
| `book/src/config/matching.md` | §15.2 bullet `:52` and console bullet `:54`; §15.4 warning `:111-112` → fixed note | CLAUDE.md: the developer updates the config page in the lane (commit 5) |
| `book/src/config/known-issues.md` | row 40 (`:90`) → Fixed | Same |
| `book/src/config/errors.md` | row `:178` gets the new `[INFO]` line | Same |
| not in this lane (phase book sync, `WORKFLOW.md:105-107`) | `audit.md:40`, `:454` status; `roadmap.md:621`; `architecture.md:408`, `:548`; `overview.md:23`, `:39`; `glossary.md:287`; `build-and-run.md:119`; `gaps.md:124` | They need the merge sha or are outside `config/` |

No change to `system_function.rs`, `windows_actions.rs`, `engine/`, `hook/`, `Cargo.toml` or `Cargo.lock`.

## Commits, in order

Subjects follow the house form `<Area>: <what> (<item>, <KS>)`. Every message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Gate for every commit except 3: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.

1. **`Tests: loader tests that count auto-blockers no longer read the registry (1.4)`**: green, test-only.
   - `src/config/loader.rs`: `parse_simple_binding` (`:882-912`) and `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (`:948-972`) use `from_ron_str_with_hotkeys(ron_text, &language_hotkey(ALT_SHIFT))`; the asserts are unchanged.
   - `src/system/registry.rs`: in `test_get_system_hotkey_returns_combinations` (`:253-260`) the `SwitchLanguage` line becomes "`None` or one of `from_keys([VK_MENU, VK_SHIFT])`, `from_keys([VK_CONTROL, VK_SHIFT])`, `new(VK_OEM_3)`", with the value in the message; the other four lines are unchanged.
   - Invariant: no product code changes.

2. **`System: the language hotkey parser returns Combo, Disabled or Unknown; the resolution takes injected values (1.4, KS-17)`**: green, no behaviour change.
   - `registry.rs`: D1 type and parsers; D4 `open_key`, `read_string_value`; D3 `hotkey_from_values` and `get_system_hotkey`. **The `Disabled` arm is skipped like `Unknown`**, with the comment `// KS-17: "3" is still read past like an unknown value; roadmap item 1.4 makes it decide.` That keeps today's behaviour exactly: `"3"` falls through to `Language Hotkey` and then to `Alt+Shift`.
   - Tests in `registry.rs` (all green here):
     - `parse_language_hotkey_is_tri_state` (replaces `test_parse_language_hotkey`, `:244-251`): `"1"`, `"2"`, `"4"` → the three `Combo`s; `"3"` and `" 3 "` → `Disabled`; `""`, `"0"`, `"5"`, `"33"`, `"unknown"`, `"Alt+Shift"` → `Unknown`.
     - `language_hotkeys_are_either_side_modifiers` (`:277-292`) and `test_parse_win_key_combo` (`:302-312`) match on `HotkeySetting::Combo(..)` (`let ... else panic!`, no `unwrap`); add `parse_win_key_combo("1")`, `("")` and `("LL")` → `Unknown`.
     - Helper `fn language(values: [Option<&str>; 2]) -> Option<KeyCombination>`: calls `hotkey_from_values` for `SwitchLanguage` and asserts that `SwitchLanguageBackward` gives the same result. Constant `ANYTHING: [Option<&str>; 7] = [None, Some("1"), Some("2"), Some("3"), Some("4"), Some(""), Some("x")]`.
     - `the_hotkey_value_decides_before_the_language_hotkey_value`: for each `a` in `ANYTHING`, `language([Some("2"), a])` is Ctrl+Shift and `language([Some("1"), a])` is Alt+Shift.
     - `no_usable_value_falls_back_to_alt_shift`: `[None, None]`, `[Some(""), Some("x")]`, `[Some("5"), None]` and `hotkey_from_values(SystemFunction::SwitchLanguage, std::iter::empty::<Option<&str>>())` (the absent key) give Alt+Shift.
     - `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide`: `[None, Some("2")]` → Ctrl+Shift; `[Some("x"), Some("4")]` → `` ` ``; `[None, Some("1")]` → Alt+Shift.
     - `the_values_after_the_deciding_one_are_not_read`: `std::iter::once(Some("2")).chain(std::iter::repeat_with(|| -> Option<&'static str> { panic!("a value after the deciding one was read") }))` → Ctrl+Shift.
     - `other_functions_keep_their_rules`: `ToggleCapsLock` with `[Some("2")]` → `None`; `TaskManager` with `[Some("3")]` → Ctrl+Shift+Esc (no location, values ignored); `LockWorkstation` with `[Some("3")]` → Win+L (`"3"` is no Win-key letter: `Disabled` exists only for the language hotkey); `LockWorkstation` with `[Some("K")]` → Win+K.
   - Tests in `loader.rs` (green here):
     - Helper `fn toggle_values(values: [Option<&'static str>; 2]) -> impl Fn(&BindAction) -> Option<KeyCombination>`: `move |action| action.to_system_function().and_then(|f| crate::system::registry::hotkey_from_values(f, values))`. Its doc: "the `Hotkey` and `Language Hotkey` strings of `HKCU\Keyboard Layout\Toggle`, `None` for a missing value, resolved as the daemon resolves them".
     - `the_registry_values_choose_the_auto_blocker_keys`: `from_ron_str_with_hotkeys(DEFAULT_CONFIG, &toggle_values(v))` gives exactly one blocker. Its `combination()` is Ctrl+Shift for `[Some("2"), None]`, Alt+Shift for `[None, None]` and Alt+Shift for `[Some("1"), Some("3")]`.
   - Clippy: `HotkeySetting::Disabled` is constructed (`parse_language_hotkey`) and matched, so there is no `dead_code`. The new `config` → `system` import is test-only; `system` imports nothing from `config` (Grep `src/system`), so no cycle.

3. **`System: red-first: a language hotkey not assigned in Windows gives no hotkey and no auto-blocker (1.4, KS-17)`**: the declared red. fmt and clippy are green; `cargo test` is red on exactly these two tests.
   - `registry.rs::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination`. The rows, in this order, each asserted with `assert_eq!(language(values), None, "{values:?}")`: `[Some("3"), Some("3")]`, `[Some("3"), None]`, `[None, Some("3")]`, `[Some("3"), Some("2")]`, `[Some("x"), Some("3")]`, `[Some(" 3 "), Some("1")]`.
     **PREDICTED RED** (first row): ``assertion `left == right` failed: [Some("3"), Some("3")]`` / `left: Some(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] })` / `right: None`.
   - `loader.rs::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` (the KS-17 scenario, with the real template): `from_ron_str_with_hotkeys(DEFAULT_CONFIG, &toggle_values([Some("3"), Some("3")]))` → `assert_eq!(blockers(&bindings), 0)`, then `bindings.len() == 2`. Second case: `parse_ron_str(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly) ] )"#, &toggle_values([Some("3"), Some("3")]))` → 0 blockers and empty `warnings`.
     **PREDICTED RED**: ``assertion `left == right` failed`` / `left: 1` / `right: 0`.
   - Replay point: this commit's tree. On `master` the two tests do not compile (`hotkey_from_values` and `toggle_values` arrive in commit 2). Commit 2 changes no behaviour, so this red is master's behaviour. Put both failing lines in the message body (`WORKFLOW.md:234-235`).

4. **`System: "3" (not assigned) decides the language hotkey: no auto-blocker, no Alt+Shift fallback (1.4, KS-17)`**: green; makes commit 3 pass.
   - `registry.rs`: the `Disabled` arm of `hotkey_from_values` prints `[INFO] {function:?}: hotkey not assigned in Windows, no auto-blocker` and returns `None`. The KS-17 comment goes, and the doc comments of `hotkey_from_values` and `HotkeySetting::Disabled` say that it decides.
   - New test `live_registry_with_the_language_hotkey_not_assigned_has_no_combination`, marked `#[ignore = "manual: set Settings > Time & language > Typing > Advanced keyboard settings > Input language hot keys > Switch Input Language to (Not assigned) first, and restore it afterwards"]`. It asserts that `get_system_hotkey(SwitchLanguage)` and `get_system_hotkey(SwitchLanguageBackward)` are `None`. Read-only.
   - `README.md:539`: after "Use when remapping language switching so Windows' own `Alt+Shift`/`Ctrl+Shift` go silent.", insert: The hotkey comes from `HKCU\Keyboard Layout\Toggle` (the value `Hotkey`, then `Language Hotkey`). When Windows has no language hotkey ("Not assigned" in its settings, value `"3"`), no blocker is added; `Alt+Shift` is assumed only when neither value names a hotkey.
   - `src/config/default_config.ron`, after `:171` (`//                         and this flag silences it.`), ASCII only:
     ```
     //                         With the Windows language hotkey set to "Not
     //                         assigned" there is nothing to block, and no
     //                         blocker is added.
     ```
   - `CHANGELOG.md`, `### Fixed`, inserted before `- KS-18:` (`:411`):
     ```
     - KS-17 (roadmap item 1.4): with the Windows language hotkey set to
       "Not assigned" (value `"3"` under `HKCU\Keyboard Layout\Toggle`),
       `block_original_combo` adds no auto-blocker. It used to fall back to an
       `Alt+Shift` blocker, which swallowed Shift pressed after Alt (KS-05)
       with no Windows hotkey to protect. The values `Hotkey` and then
       `Language Hotkey` are read in that order, and the first one that names
       a hotkey (`"1"`, `"2"`, `"4"`) or says `"3"` decides; `Alt+Shift` is
       assumed only when neither does (a missing, unreadable or unknown
       value). The console says
       `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker`,
       and the startup list has no `[AUTO-BLOCK]` line. The registry is only
       read.
     ```
   - Invariant: the `Combo` and fallback outcomes are exactly those of commit 2 (the commit-2 tests stay green).

5. **`Book: the language hotkey not assigned in Windows in the config reference (1.4, KS-17)`**: green, docs only. Build with `mdbook build book -d D:/tmp/ks-orch/p1-1.4-hotkey-tristate/book` and check the internal links of the three pages.
   - `matching.md` §15.2, the `"3"` bullet (`:52`), says:
     - `"3"` ("Not assigned") means that Windows has no language hotkey.
     - The first value that names a combination or says `"3"` decides. So `"3"` in `Hotkey`, or in `Language Hotkey` when `Hotkey` is missing or unknown, gives no blocker and no fallback.
     - Any other text, a missing value and a value of another type are skipped.
     - If no value decides, or the key does not exist, the fallback `Alt+Shift` is used.

     The console bullet (`:54`) gains the third line. The line refs at `:46` stay pinned, and the book sync re-pins them after the merge.
   - `matching.md` §15.4: the `"3"` warning (`:111-112`) becomes a `> [!NOTE]`: "Fixed on `fix/language-hotkey-tristate` (KS-17): the value `"3"` gives no blocker; it used to give an `Alt+Shift` blocker."
   - `known-issues.md` row 40 (`:90`): Status `**Fixed**: "3" gives no blocker`, Where `` `fix/language-hotkey-tristate` ``.
   - `errors.md` row `:178`: add `` / `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker` ``, with the cause "the Windows hotkey is set to Not assigned (`"3"`), so no blocker is added ([§15.2](matching.md#s15-2))".
   - No new ` ```ron ` block, so no `cfgcheck` run is needed.

Bisect: commits 1 and 2 change no behaviour, 3 is the only red, and 4 flips one match arm. README, template and CHANGELOG change in commit 4, the commit that changes the meaning.

## Acceptance mapping

| Roadmap / brief acceptance line | Covered by |
|---|---|
| `parse_language_hotkey` returns `Combo(keys)`, `Disabled` ("3") or `Unknown` | `registry::tests::parse_language_hotkey_is_tri_state` (commit 2) |
| (`Hotkey`, `Language Hotkey`) = ("3","3") gives no blocker | `registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination` (row 1); `config::loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker`; manual M1 |
| With `Disabled`, no fallback to Alt+Shift | same registry test, rows `[Some("3"), None]`, `[None, Some("3")]`, `[Some("3"), Some("2")]` |
| ("2", anything) gives Ctrl+Shift | `registry::tests::the_hotkey_value_decides_before_the_language_hotkey_value`; loader `the_registry_values_choose_the_auto_blocker_keys` (`[Some("2"), None]`); manual M3 |
| ("1", anything) gives Alt+Shift | same registry test; loader row `[Some("1"), Some("3")]`; manual M2 |
| No values give Alt+Shift | `registry::tests::no_usable_value_falls_back_to_alt_shift` (the absent key included); loader row `[None, None]` |
| Only a missing or unparsable value falls back | `no_usable_value_falls_back_to_alt_shift`, `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide` |
| The registry stays read-only; the resolution is testable with injected values | D3/D4. Review check: the diff adds no `RegSetValue*`, `RegCreateKey*`, `RegDelete*` or `SystemParametersInfo*` (Grep) |
| CHANGELOG entry | commit 4 |

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | In Windows: Settings > Time & language > Typing > Advanced keyboard settings > Input language hot keys > Change Key Sequence > Switch Input Language: **Not assigned**. Note the old setting first. | 1. `reg query "HKCU\Keyboard Layout\Toggle"` (read-only). 2. `cargo test --locked -- --ignored live_registry_with_the_language_hotkey_not_assigned_has_no_combination`. 3. Start the release daemon with the default config. 4. In Notepad: Alt down, Shift down, release both; then Alt, Shift, X. 5. Press CapsLock. 6. Restore the old setting. | 1: `Hotkey` and `Language Hotkey` are `3` (this records what Windows writes, Q1). 2: passes. 3: `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker`, no `[AUTO-BLOCK]` line, and `swch status` counts 2 bindings. 4: no key is swallowed (Notepad sees Alt+Shift+X; the menu bar behaves as without the daemon). 5: the layout switches. | `live_registry_with_the_language_hotkey_not_assigned_has_no_combination` |
| M2 | Switch Input Language: **Left Alt+Shift** | Restart the daemon (default config) | `[INFO] SwitchLanguage: registry combination Alt+Shift` and one `[AUTO-BLOCK] Alt + Shift` line | none (regression) |
| M3 | Switch Input Language: **Ctrl+Shift** | Restart the daemon (default config) | `[INFO] SwitchLanguage: registry combination Ctrl+Shift`; the blocker is `Ctrl + Shift` | none (regression) |

## Risks and edge cases (R1-R10)

1. **R1 hook callback.** Not touched. The registry is read only while a config loads (main thread at startup, `config-watcher` on reload). `BindAction::get_system_combination` has no caller on the hook path (Grep: `loader.rs:223` only).
2. **R2 sentinel.** No `SendInput`. N/A.
3. **R3 shared state / R4 orderings.** No new shared state, atomics, statics or locks. The startup read finishes before the watcher thread exists (`main.rs:30-32`), so two reads never overlap. Each read opens and closes its own subkey handle and never closes `HKEY_CURRENT_USER` (doc quote above).
4. **R5 builders / R6 Display.** No new builder. `HotkeySetting` is private and never shown, so it gets `Debug` only; no user-facing enum gains a variant.
5. **R7 specificity.** A `"3"` machine loses one appended blocker; the `update_bindings` order rules are unchanged. A reload mid-press that drops the blocker while its swallowed Shift is held: the repeats and the key-up keep following the press's own record (`LastDown`, `src/engine/keys.rs:58`, `:193-207`). The second half of `engine::scenarios::reload_during_a_passing_double_tap_keeps_each_press_whole` (`scenarios.rs:3339-3355`, test `:3317-3356`) pins exactly that: a key-down swallowed before a reload to a passing binding keeps its key-up swallowed. That is the same as removing `block_original_combo` by hand today.
6. **R8 key identity.** The three combinations stay `AnySide` (constructors unchanged; `language_hotkeys_are_either_side_modifiers` keeps pinning it).
7. **R9 registry reads.** Read-only. The guard closes the subkey on every path; lengths are in bytes; only `REG_SZ`/`REG_EXPAND_SZ` are parsed. A value longer than 127 UTF-16 units fails with `ERROR_MORE_DATA` and counts as missing, as today. A hand-written `REG_DWORD` `3` stays `Unknown` with the existing `[WARN]` (Q4). Mixed values: under D2, `("3","1")` gives no blocker. If Windows actually honours `Language Hotkey`, Windows then still switches on Alt+Shift next to CapsLock, which is the behaviour without the flag; it never brings back the KS-05 swallowing (Q1).
8. **Hot reload, `swch off`/`on`, foreground change, shutdown during a pending gesture.** These paths are unchanged. Changing the Windows setting does not trigger a reload: the blocker follows on the next save of `config.ron` or on restart (documented, `matching.md:55`).
9. **Elevation, UIPI.** `HKCU` maps to the token's user. With UAC's split token that is the signed-in user, so the hive is the same. A daemon elevated as a **different** administrator account reads that account's `Toggle`, and the blocker follows the wrong user's setting. This is pre-existing and outside 1.4: follow-up F1 (`RegOpenCurrentUser` does not help either; the interactive user's SID would be needed). No pipe, DACL or token change.
10. **R10 message loop and shutdown.** N/A.
11. **IME and layouts.** The `"4"` grave-accent hotkey keeps `VK_OEM_3` (the `Ё` key on Russian layouts, `matching.md:51`), unchanged.
12. **Test hermeticity.** After commit 1, only `test_get_system_hotkey_returns_combinations` and `test_hardcoded_combinations` (`system_function.rs:179-196`) read the live registry, and both accept every valid machine state.

Follow-ups (not in this lane): F1, the hive of a daemon elevated as another user (no KS id; a candidate for `gaps.md`); `Layout Hotkey` and Win+Space (feature #7); the unreal Lock/ShowDesktop lookups (feature #1).

## Questions for the orchestrator

1. **Which value wins when `Hotkey` and `Language Hotkey` disagree?** The acceptance line `("2", anything)` puts `Hotkey` first, and this cut applies the same rule to `"3"`. Windows' own precedence is unverified. ReactOS's `input.cpl` clone reads only `Language Hotkey` and writes `Hotkey` alongside it (doxygen `key_settings_dialog.c`, `ReadKeysSettings`, `WriteKeysSettings`), which suggests that the two normally agree. **Recommendation:** keep the spec order. This is non-blocking, and M1 step 1 records what Windows writes. Researcher query for a later lane: "Which value under HKCU\Keyboard Layout\Toggle (Hotkey, Language Hotkey) does Windows 10/11 honour when they differ, and what does the 'Input language hot keys' dialog write for (Not assigned)?"
2. **Warn about an unknown value text, such as `"5"`?** **Recommendation:** no. Today it is silent, and the `using default combination` line already says what happened. It is outside 1.4's scope.
3. **Book pages outside `config/`** (audit status, roadmap table, architecture, overview, glossary, build-and-run, gaps). **Recommendation:** leave them to the phase book sync (`WORKFLOW.md:105-107`), which knows the merge sha. This lane edits only the three `config/` pages, as CLAUDE.md asks of a developer.
4. **`REG_DWORD` values.** **Recommendation:** keep them `Unknown` with the existing `[WARN]`. The documented values are strings (the `SPI_SETLANGTOGGLE` quote).

## Cut readiness checklist

**Spec**
- [x] KS-17 read (`audit.md:450-463`). Its scenario (default config + Not assigned) is reproduced by `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` and by M1.
- [x] Every book reference re-located (table above). None is gone, and the defect is present at `registry.rs:185`, `:120-124` and `:60-64`.
- [x] Goal, non-goals and dependants stated.

**Design**
- [x] D1-D7 each have a rationale and rejected alternatives.
- [x] Owner decisions: exact modifiers and `allow_extra_modifiers` untouched (blocker matching unchanged); `repeat` defaults untouched; RON unchanged; `println!` until 1.7; licence N/A.
- [x] Thread named for every path: main (startup load), `config-watcher` (reload); nothing in the LL callback.
- [x] Install, uninstall, timers: N/A, none touched.
- [x] `SendInput` sentinel: N/A, no injection.
- [x] Sided and unsided: the hotkeys stay `AnySide`; combo, chord and sequence matching are untouched.
- [x] Reload, `swch off`/`on`, foreground change, shutdown: risks 5 and 8.
- [x] Atomic orderings: N/A, no new shared state; no `static mut`.
- [x] No new module cycle (only a test-only `config` → `system` import). The logic runs at config time, not per key, so it sits in `system/registry.rs`, where roadmap 1.4 puts it (D7 says why there is no engine test).

**Win32**
- [x] Each API has its `windows` path, a present feature (`Cargo.toml:49`), a thread, its errors and a doc link.
- [x] Elevation considered (risk 9); UIPI, pipe DACL and token inheritance N/A.

**Commits and tests**
- [x] Red-first: commit 3 precedes the fix, with exact predicted assertions.
- [x] No test installs a hook, injects input or writes the registry; the tests run on injected strings. Engine tests N/A (no decision change, D7).
- [x] The one test that needs a setting changed by a human is `#[ignore = "manual: ..."]` and listed (M1).
- [x] Every commit passes the gate except the declared red commit 3.
- [x] README, template and CHANGELOG change in commit 4, which changes the meaning; the branch bisects.

**Acceptance**
- [x] Every acceptance line maps to a test or a manual check.
- [x] Every question has a recommendation.

## Developer addendum (critique round 1)

- **I1 (Important): RESOLVED** with option (a), in commit 2 (`5493b3a`): `src/system/registry.rs:348` `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` asserts, for `SwitchLanguage` and `SwitchLanguageBackward`, `hkey == HKEY_CURRENT_USER`, `subkey == "Keyboard Layout\Toggle"` and `value_names == ["Hotkey", "Language Hotkey"]`. Swapping the two names at `registry.rs:35` makes it fail; the positional resolution tests then name the values through that pinned order (helper `language`, `registry.rs:301-309`).
- **M1 (Minor): RESOLVED** in commit 5 (`9fcbc93`): `book/src/config/matching.md:48` (a missing, unreadable or unknown value passes on; a value that names a combination or says `"3"` stops the lookup) and `book/src/config/example.md:8` (the validator ran on a machine with no values under `HKCU\Keyboard Layout\Toggle`).
- Critic open question 2 (wording): the template comment says "there is no language hotkey to block" and the CHANGELOG "with no Windows language hotkey to protect", instead of "nothing to block" / "no Windows hotkey to protect", because `Layout Hotkey` is not read. The README sentence ends "neither value names a hotkey or says `"3"`", since `("3", missing)` names no hotkey and still gets no fallback.
