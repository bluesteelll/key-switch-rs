# Code review: p1-1.4-hotkey-tristate @ 9fcbc93
APPROVED (Critical 0, Important 0, Minor 0)

Scope: `git diff master...fix/language-hotkey-tristate` (merge-base `f28da35`, 5 commits `06f385d`..`9fcbc93`, 9 files, +380/-64), read via `git show`/`git diff` only. Also read: the cut (`cut.md`, including the developer addendum), `critique.md`, `impl.md`, `red_commit3.log`, `pack_r1.md`. Every touched source file was read in full at the lane ref (`src/system/registry.rs` 1-557, the touched and neighbouring parts of `src/config/loader.rs` 200-410, 763-830, 880-1070, 2465-2500), and so were the call sites of the changed contract (`src/core/windows_actions.rs:144-158`, `src/system/system_function.rs:38-40,175-213`, `src/engine/testing.rs:165-200`, `src/engine/mod.rs:700-800`, `src/engine/scenarios.rs` blocker scenarios, `src/hook/config_watcher.rs:280-370`).

## Gate: fmt ✅ · clippy ✅

Run on a `git archive fix/language-hotkey-tristate` export of `9fcbc93` (scratchpad tree, not the shared work tree), `CARGO_TARGET_DIR` of its own:
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 (`Checking key-switch-rs v0.1.0 ... Finished`), toolchain rustc 1.98.1.
- Additionally, `mdbook build book` on the same export: builds. The anchors used by the new links resolve (`roadmap.html#14-tri-state-language-hotkey`, `audit.html#ks-17`, `config/matching.html#s15-2`, `#s15-4`).
- `cargo test` was not run (the tester owns it).

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor
None.

## Checklist walked (each item checked, no reachable failure found)

**1. Plan and roadmap conformance**
- D1: `HotkeySetting { Combo, Disabled, Unknown }` is private (`registry.rs:17-28`). `parser: fn(&str) -> HotkeySetting` (`:14`). `parse_language_hotkey` maps `"1"`/`"2"`/`"4"` to `Combo`, `"3"` to `Disabled` and anything else to `Unknown` on `trim()` (`:226-234`). `parse_win_key_combo` never yields `Disabled` (`:236-246`), and it matches master's `?`-chain exactly: `len()==1` and `char_to_vk`, else `Unknown`.
- D2/D3: in `hotkey_from_values` (`:87-116`), the first `Combo` returns (`:94-97`) and the first `Disabled` returns `None` with the new `[INFO]` (`:100-105`). `Unknown` and `None` are skipped (`:92` `.flatten()`, `:106`). With no deciding value, the default applies (`:111-115`). This is the spec rule "only a missing or unparsable value falls back".
- D4: `open_key` (`:148-172`) and `read_string_value` (`:177-224`) are master's `:96-118` and `:134-177`, moved verbatim. The only changes are the reworded SAFETY comment and the `&HKeyGuard` parameter.
- D5: loader production code is untouched. `None` already means no blocker and no `MaskOnly` warning (`loader.rs:350-352`).
- D6: the two blocker-counting tests are hermetic (`loader.rs:896`, `:969`). `test_get_system_hotkey_returns_combinations` accepts `None` (`registry.rs:466-474`).
- Acceptance, each pinned by a test that can fail:
  - `("3","3")` gives no blocker: `registry.rs:442` and `loader.rs:1053-1070`. Red on `8ef0ab7` for the right reason: the fallback Alt+Shift, `left: 1 / right: 0` (`red_commit3.log`).
  - `("2", anything)` gives Ctrl+Shift and `("1", anything)` gives Alt+Shift: `registry.rs:368-378`, over `ANYTHING`, which includes `"3"` and missing.
  - No values give Alt+Shift: `registry.rs:380-393`, the absent key included; `loader.rs:1036`.
  - `Disabled` gives no fallback: `registry.rs:441-450` rows `["3",None]`, `[None,"3"]`, `["3","2"]`.
  - Critique I1: the name-to-position mapping is pinned by `registry.rs:347-366`, so swapping the value names fails it.
- Bisectability: `5493b3a` keeps master's behaviour (`Disabled | Unknown => {}`, fact checked via `git show 5493b3a:src/system/registry.rs`). `38029df` flips one arm (`git diff 8ef0ab7 38029df -- src/`).

**2. Unsafe**
- The lane adds no new block. All three blocks have a `// SAFETY:` comment that holds:
  - `:138-141` `RegCloseKey`: `HKeyGuard` is built only on a successful open (`:168-171`) and never copied or closed elsewhere.
  - `:157-167` `RegOpenKeyExW`: `subkey_wide` is a named local, NUL-terminated, alive through the call.
  - `:187-200` `RegQueryValueExW`: `key` is borrowed for the call. The buffer size is in bytes (`:184`, `size_of_val` of `[u16;128]` = 256). The result is clamped to the bytes written (`:217-218`) and scanned for NUL (`:219-222`).
- The guard outlives the lazy value iterator: `key` is bound in the `if let` chain, and `hotkey_from_values` consumes the iterator before the `return` (`:66-75`).
- No `extern "system"` path is touched.

**3. Atomics:** none added or changed.

**4. Win32 thread affinity**
- No hook, window, timer, COM or pipe change.
- The registry is read only through `loader.rs:223`, which is the only product caller of `get_system_combination` (repo grep). That runs on the main thread at startup or on `config-watcher`, never in the LL callback.
- `HKEY_CURRENT_USER` is never closed. Only the opened subkey is.

**5. Hot path:** `src/hook/**` and `src/engine/**` are not in the diff. The new `String` per value read (`:223`) is on config load only.

**6. Correctness**
- Other kinds and triggers: the binding list differs only by the absence of the appended blocker on a `"3"` machine. The engine and the `update_bindings` sort are unchanged.
- `to_system_function` maps only the two language actions and `ToggleCapsLock` (`windows_actions.rs:144-151`), so Lock/ShowDesktop/TaskManager lookups stay unreachable from configs. Their resolution is unchanged anyway (`registry.rs:414-435`).
- Laziness: values after the deciding one are not read, pinned for `Combo` (`:402-412`) and for `Disabled` (`:451-458`).
- Suppression invariant: unchanged code. A reload that drops a blocker while its swallowed Shift is held keeps the key-up following the press record. This is pre-existing and the same as removing the flag by hand (cut risk 5).

**7. Style, integration, build**
- `pub(crate) hotkey_from_values` is reachable from the test-only import `loader.rs:766`. There is no `config -> system` product dependency.
- No `#[allow]` and no new dependency.
- Added lines are ASCII.
- `.unwrap()` appears only in two test lines carried over from master (`loader.rs:896`, `:969`).
- All five commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, and their subjects follow the house form.

**Docs**
- `CHANGELOG.md:411-422` sits under `## [Unreleased]` / `### Fixed` (section `:239`) in KS order and cites KS-17 and roadmap 1.4.
- `README.md:539` gives the correct value precedence and the `"3"` rule.
- `default_config.ron:172-174` is a comment only, above the examples parser marker.
- Book: `matching.md` §15.2 (`:48`, `:52-53`, `:55`) matches the code, including the `Hotkey`="3" stop and the console line. Also updated: the §15.4 note (`:112-113`), `known-issues.md` row 40, `errors.md:178` and `example.md:8`.

**R1-R10**
- R1 hook callback: not touched.
- R2 sentinel: no `SendInput` or `dwExtraInfo` added (grep of `+` lines).
- R3 shared state: no static, lock or `ArcSwap` change.
- R4 orderings: none.
- R5 builders: none added.
- R6 Display: `HotkeySetting` is private, `Debug`-only, never user-facing. No public enum gained a variant. Its `_ => Unknown` arm is the intended catch-all.
- R7 specificity: blockers are still appended after user bindings in `expand_with_auto_blockers`. There is no second publish path.
- R8 key identity: the language combos stay `AnySide` (`registry.rs:507-524`).
- R9 registry:
  - read-only (no `RegSet*`, `RegCreate*`, `RegDelete*` or `SystemParametersInfo*` in `+` lines);
  - the guard closes the key on every path;
  - lengths are in bytes;
  - only `REG_SZ`/`REG_EXPAND_SZ` are parsed;
  - `"3"` means disabled (KS-17).
- R10 message loop: not touched.

**Safety and tests**
- No test writes the registry, installs a hook or injects input.
- The live check is `#[ignore = "manual: ..."]` (`registry.rs:481-490`), and it reads only.
- The remaining live-registry tests are read-only and accept every machine state:
  - `registry.rs:461-479`;
  - `system_function.rs:179-205`;
  - `loader.rs:771`, `:2477`.
- Every other `block_original_combo` test in `loader.rs` and `engine/scenarios.rs` uses a stand-in (`language_hotkey`, `toggle_values`, `from_ron_with_language_hotkey`).

## Positive
- The I/O/decision split is minimal and well placed. `hotkey_from_values` takes exactly the strings `reg query` shows, so the acceptance lines are literal unit tests. It needs no trait and no registry write.
- `toggle_values` (`loader.rs:1017-1028`) runs the real resolution through `to_system_function`. The loader-level tests therefore exercise the production chain except the registry read itself.
- The red-first is genuine and replayable. It fails on master's behaviour, with the predicted messages recorded in the commit body and the log.
- Critique I1 was closed with a test that a value-name swap fails. The extra `Disabled` laziness assertion (`registry.rs:451-458`) pins a D2 property nothing else covered.
- The SAFETY comment of `RegQueryValueExW` is stronger than master's: it now states the handle and name invariants.

## Open questions
1. **Which value Windows honours when `Hotkey` and `Language Hotkey` disagree** (cut Q1, critique OQ1). This is a known open question, not a defect: the spec fixes `Hotkey` first. Under D2, a stale `Hotkey`="1" next to `Language Hotkey`="3" still yields an Alt+Shift blocker, which is the pre-fix behaviour. The question is settled by step 1 of manual check M1 (`reg query "HKCU\Keyboard Layout\Toggle"` after choosing Not assigned in the Windows dialog). If Windows writes only `Language Hotkey`, a later lane should revisit the precedence.
2. **Book pages outside `config/` still list only the two old `[INFO]` lines.** One example is `book/src/build-and-run.md:119`. Like the `audit.md:40`/`roadmap.md:621` status lines and the `1dfd80a` permalinks at `matching.md:46`, the cut (Q3) and WORKFLOW assign them to the phase book sync. This is a hand-off note for the doc-writer, not a lane remark.
