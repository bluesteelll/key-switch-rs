# Architecture critique: p1-1.4-hotkey-tristate / roadmap 1.4, KS-17 (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 1)
Plan: D:/tmp/ks-orch/p1-1.4-hotkey-tristate/cut.md
Code read at: work tree D:/key-switch-rs/key-switch-rs (the cut says `master` @ `f28da35`). I checked every file:line the cut cites in `registry.rs`, `loader.rs`, `system_function.rs`, `windows_actions.rs`, `main.rs`, `config_watcher.rs`, `engine/{keys,testing,scenarios,properties,mod}.rs`, `CHANGELOG.md`, `README.md`, `default_config.ron` and `book/src/config/{matching,known-issues,errors}.md`. They all match.

Per WORKFLOW.md:66-67, the cut needs no second critique round: there is no Critical, so I1 and M1 go to the developer, who resolves or refutes them.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. No test pins which value name wins, and the acceptance lines are stated in value names
Where: plan D2 (lines 78-83), D3 (line 110: `get_system_hotkey` passes `location.value_names.iter().map(|name| read_string_value(&key, name))`), commit 2 tests (lines 188-193: `language([Option<&str>; 2])` injects by position), D6 (line 133: the only live test no longer depends on the order); `src/system/registry.rs:23` (`value_names: &["Hotkey", "Language Hotkey"]`).

Problem: every resolution test injects values into `hotkey_from_values` by position. Only `value_names` at `registry.rs:23` decides which registry name lands in position 0, and that sits on the I/O side of the D3/D4 split, which no test covers. After commit 1, the only automated test that goes through `get_system_hotkey` (`test_get_system_hotkey_returns_combinations`) accepts `None`, Alt+Shift, Ctrl+Shift or `` ` ``, so it cannot see the order either. The manual checks M2 and M3 cannot see it: the Windows dialog writes both values with the same number. ReactOS's clone of that dialog, `input.cpl` `WriteKeysSettings`, writes `Hotkey` with the same data as `Language Hotkey`.

Consequence: swap the two names at `registry.rs:23` and `cargo test` stays green. Feature #7 has to edit that exact list to add `Layout Hotkey`, so the swap is a realistic slip. After it, the daemon reads `Language Hotkey` first:
- `Hotkey`="2", `Language Hotkey`="1": the default config gets an `Alt+Shift` blocker instead of `Ctrl+Shift`. Shift pressed after Alt is swallowed (KS-05).
- `Hotkey`="3", `Language Hotkey`="1": the blocker that KS-17 is about comes back.

The acceptance lines say "(`Hotkey`, `Language Hotkey`) = ("2", anything) gives Ctrl+Shift" and ("3","3") gives no blocker. They name the values, while the planned proof covers only positions. The tester's mandatory mutations (WORKFLOW.md:69-70) are likely to try exactly this swap. It would survive, and the lane would pay a triage and fix round.

Confidence: CONFIRMED (registry.rs:23; plan lines 110, 133, 188-193).
Breaks: the acceptance proof of roadmap 1.4 (`roadmap.md:269`); the brief's "an acceptance criterion without a proof".
What is needed: one automated, registry-free check that gives the name `Hotkey` the higher priority and `Language Hotkey` the lower. It goes in commit 2, which is green and pins today's order.
Options:
- (a) A `registry.rs` unit test that asserts the `value_names` of the `SwitchLanguage` and `SwitchLanguageBackward` locations.
- (b) Have the resolution take (name, value) pairs, so the injected tests name the values they inject.

## Minor (improvements, not blockers)

### M1. The first bullet of §15.2 and the complete example still describe the old skip rule
Where: plan commit 5 and the touch set (lines 167, 234-241). They edit `book/src/config/matching.md` only at :52, :54 and :111-112. Also `book/src/config/matching.md:48` and `book/src/config/example.md:8`.

Problem:
- `matching.md:48` says `Language Hotkey` is read when `Hotkey` "is absent or yields no combination". After commit 4, a `Hotkey` of `"3"` yields no combination and `Language Hotkey` is not read. Line 48 then contradicts the rewritten `"3"` bullet four lines below it.
- `example.md:8` says the values "did not yield a combination, so the fallback `Alt+Shift` was used". After the fix that no longer follows: `"3"` yields no combination and gives no fallback.

Consequence: a user with `Hotkey`="3" and `Language Hotkey`="2" reads the first bullet of §15.2 and expects a Ctrl+Shift blocker. The daemon adds none and prints the new `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker`, which that bullet does not explain.
Confidence: CONFIRMED.
What is needed: add both lines to commit 5.
- `:48`: a missing, unreadable or unknown value passes on to the next value, and `"3"` stops the lookup.
- `example.md:8`: describe the machine as having no values under `Toggle` (methodology.md:524).

Both pages are under `config/`, so CLAUDE.md puts them in this lane.

## Strengths to keep

- **Defect and fix location.** The defect is located correctly (`registry.rs:185`, `:120-124`, `:60-64`), and production code changes only in `registry.rs`, as roadmap 1.4 asks. D5 is right: the loader already maps `None` to no blocker and no `MaskOnly` warning (`loader.rs:347-352`).
- **The D3/D4 split** passes strings in and gets a decision out. It meets "testable with injected values" without a registry write or a trait. The guard keeps closing the subkey on every path, and no API, Cargo feature or thread is added.
- **Commit 1** makes the three machine-dependent tests hermetic before the behaviour changes. I checked that every other blocker-counting test in `loader.rs` (975-1280) and every `block_original_combo` scenario (`scenarios.rs` 2451, 2508, 2688, 3213, 3237, 4026, 4274-4425, 4463) uses a stand-in. The engine's `Scenario::reload` calls (`scenarios.rs:1855`, `:3328`, `:3346`, `:3772`) carry no flag.
- **The red-first in commit 3 can fail, and fails for the right reason.** On the commit-2 tree, `[Some("3"), Some("3")]` resolves to `Alt+Shift`: `Disabled` is skipped, the second `"3"` is skipped, and the fallback applies. The predicted message matches the derived `Debug` of `KeyCombination`/`KeySpec` (`key_combination.rs:11`, `key_spec.rs:123`). The replay point is declared: commit 2, not `master`, where the tests fail to compile with E0425. The tester's brief must carry it, because WORKFLOW.md:69 defaults to `master`.
- **The laziness test** (`once(Some("2")).chain(repeat_with(|| panic!(..)))`) pins "values after the deciding one are not read".
- **Hot path and threading.** The registry is read only on the main thread at startup (`main.rs:30`) and on `config-watcher` (`config_watcher.rs:159`). `get_system_combination` has one product caller (`loader.rs:223`). The lane touches no hook, window, timer, COM apartment or `SendInput`.
- **D2** follows the spec's acceptance line. Risk 7 argues correctly that a wrong `Disabled` costs only the behaviour without the flag (Windows also switches) and never brings back KS-05's swallowing.

## Open questions for the architect

1. **Evidence for Q1 (non-blocking; the spec is binding).** ReactOS's win32k reads only `Language Hotkey` (default 1) and `Layout Hotkey` (default 2): `gdwLanguageToggleKey = UserGetLanguageToggle(L"Language Hotkey", 1)` in `sysparams.c`. Its `input.cpl` writes `Hotkey` with the same number as `Language Hotkey`. That supports the cut's assumption that values written by the dialog agree, so D2 is harmless for them. It also suggests that if Windows honours only `Language Hotkey`, a hand-edited disagreement follows Windows less well under D2. M1 step 1 should record all three values under `Toggle`.
2. **`Layout Hotkey`="1" with `Language Hotkey`="3".** Windows may still act on Alt+Shift there (it switches the layout within the current language), and the old fallback blocker covered that by accident. After the fix nothing covers it. The new template line ("there is nothing to block") and the CHANGELOG ("with no Windows hotkey to protect") then overstate slightly. Wording such as "no language hotkey to block; `Layout Hotkey` is not read (§15.2)" would be exact. This is not raised as a finding because:
   - it affects only users with two layouts in one language;
   - the gap is already documented (`matching.md:56`, `gaps.md`, feature #7);
   - it cannot be observed on the owner's machine (`methodology.md:524`: `Toggle` has no values).
   M1 step 1 could also record `Layout Hotkey`.
3. **Risk 12 wording.** Risk 12 says only two tests read the live registry after commit 1. `default_config_parses` (`loader.rs:769-776`) and `a_missing_config_beside_the_exe_is_generated_and_loaded` (`loader.rs:2420-2431`) also read it, through `from_ron_str`/`load`. Both assert only that bindings are non-empty, so they accept every machine state. Only the wording is wrong; there is no consequence.

## Absent topics (walked, not applicable)

- §1 hook budget, §2 thread affinity, §5 timers and gestures, §8 shared state and atomics, §9 pipe/DACL/token and §10 `SendInput`: none is touched.
- §4 suppression lifetime: only reload mid-press applies. Risk 5 handles it correctly: a swallowed Shift keeps its repeats and key-up swallowed through the `LastDown` record (`engine/keys.rs:58-211`) after a reload drops the blocker.
- §3 matching: the three hotkeys stay `AnySide` (`language_hotkeys_are_either_side_modifiers` stays).
- §6 pure core: no decision changes, so D7's reasoning holds. The KS-17 scenario is a load-time fact and is proven at the loader with the real template.
- §7 config: no schema or RON change, and the reload re-reads the registry as before. `"3"` now means the hotkey is disabled, as §7 requires. The blocker is still not an ordinary binding.
- §11 fit: `HotkeySetting` is private and has `Debug` only, like other private enums (`LastDown`). The CHANGELOG entry goes under Fixed in KS order (`CHANGELOG.md:397` KS-14, `:411` KS-18). README, template and CHANGELOG change in the commit that changes the meaning.

## Sources

- [SystemParametersInfoW, SPI_SETLANGTOGGLE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow) ("1" = ALT+SHIFT, "2" = CTRL+SHIFT, "3" = none)
- [ReactOS `key_settings_dialog.c`](https://doxygen.reactos.org/df/db7/key__settings__dialog_8c_source.html) (`WriteKeysSettings` writes `Hotkey` = `Language Hotkey`)
- [ReactOS ros-diffs: win32k reads `Language Hotkey` / `Layout Hotkey`](https://lists.reactos.org/hyperkitty/list/ros-diffs@reactos.org/thread/C6TSBF6CETQGWEIU6KO2FC7TWE2QSAF7)
