# Tester: p1-1.4-hotkey-tristate / roadmap 1.4, KS-17 (HEAD 9fcbc93 vs master f28da35)

Verdict: GREEN. Gate green on HEAD, both red-first tests red by assertion on 8ef0ab7 and green on HEAD, 7 own mutations all caught, every acceptance line mapped to a passing test.

HEAD `9fcbc93` on `fix/language-hotkey-tristate`; `git status --short` empty before and after (0 lines).

## Gate (HEAD)
- cargo fmt --check: OK (exit 0)
- cargo clippy --all-targets --locked -- -D warnings: OK. First run printed `Finished` in 0.13s (stale-looking), so I touched lib.rs, main.rs, registry.rs, loader.rs and re-ran: `Checking key-switch-rs ... Finished dev profile in 1.20s`, exit 0.
- cargo test --locked --no-fail-fast (log: `gate_test_head.log`), exit 0:
  - lib: `running 482 tests` -> `test result: ok. 479 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out`
  - bin key-switch-rs: `running 0 tests` (no tests in that target, expected)
  - integration target: `running 6 tests` -> `ok. 6 passed; 0 failed; 0 ignored`
  - swch bin: `running 0 tests`
  - Totals: 485 passed, 0 failed, 3 ignored.
  - Ignored (manual, not run, not counted as passes):
    - `data::condition::tests::foreground_context_names_the_focused_notepad` (manual: focus Notepad)
    - `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (manual: injects vk 0xE8)
    - `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (manual: set "Switch Input Language" to Not assigned, restore afterwards)
- cargo build --release --locked: OK (`Finished release profile in 5.72s`). Not run.

## Red-first replay
Named in cut.md/impl.md (commit 3, `8ef0ab7`): two tests.

| Test | master f28da35 | 8ef0ab7 (tree before the fix) | HEAD 9fcbc93 | Note |
|---|---|---|---|---|
| system::registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination | test absent (`running 0 tests ... 472 filtered out`); would be compile-red (needs `hotkey_from_values`) | red (assert) registry.rs:442:13 `[Some("3"), Some("3")]` left `Some(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] })` right `None` | ok | KS-17 |
| config::loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker | absent, compile-red as above | red (assert) loader.rs:1060:9 left `1` right `0` | ok | KS-17 |

- 8ef0ab7: `running 2 tests` ... `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 479 filtered out`. Both fail by assertion, with the old behaviour visible in the output (`[INFO] SwitchLanguage: using default combination`).
- HEAD: `running 2 tests` ... `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 480 filtered out`.
- The master replay is a vacuous `running 0 tests` and proves nothing alone; the behavioural red proof is the 8ef0ab7 tree, which differs from master only in the (behaviour-neutral) parser/resolution refactor from 5493b3a plus the tests. Because those two tests do not compile against master's API, the mutations below (M1, M3) carry them on HEAD.
- Scratch worktrees (`wt/master`, `wt/red`, `wt/mut`) removed; `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 9fcbc93 [fix/language-hotkey-tristate]`.

## Mutations (scratch worktree `wt/mut` at HEAD, `CARGO_TARGET_DIR=target-wt/mut`; lane tree untouched)
Full `cargo test --locked --lib` per mutation; each reverted with `git checkout -- .` and `git diff --exit-code` clean.

| # | File:line | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | registry.rs:104 | `Disabled` arm `return None` -> `break` (falls back to Alt+Shift) | a_language_hotkey_not_assigned_in_windows_has_no_combination | caught: that test and loader::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker FAILED (2 failed) |
| M2 | registry.rs:36 | value_names order swapped to `["Language Hotkey","Hotkey"]` | the_language_hotkey_is_read_from_hotkey_then_language_hotkey | caught: that test only (1 failed) |
| M3 | registry.rs:230 | `"3"` parsed as `Unknown` | parse_language_hotkey_is_tri_state | caught: tri_state, has_no_combination, gives_no_auto_blocker (3 failed) |
| M4 | registry.rs:106 | `Unknown => break` (stop at an unknown value) | a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide | caught: that test + has_no_combination (2 failed) |
| M5 | registry.rs:92 | `.flatten()` -> `.map_while(\|v\| v)` (a missing value stops the scan) | a_missing_or_unknown_hotkey_value_leaves_... | caught: that test + has_no_combination (2 failed) |
| M6 | registry.rs:228-229 | `"1"` and `"2"` combos swapped | the_hotkey_value_decides_before_the_language_hotkey_value | caught: that test, parse tri_state, language_hotkeys_are_either_side_modifiers, the_values_after_the_deciding_one_are_not_read, a_missing_or_unknown..., loader::the_registry_values_choose_the_auto_blocker_keys (6 failed) |
| M7 | registry.rs:121 | language default Alt+Shift -> Ctrl+Shift | no_usable_value_falls_back_to_alt_shift | caught: that test, test_default_combinations, loader::the_registry_values_choose_the_auto_blocker_keys (3 failed) |

All 7 caught by the named test.

## Acceptance (roadmap 1.4 + brief)
All commands run on HEAD; each: `running 1 test ... test result: ok. 1 passed` (the `not_assigned` pair: `running 2 tests ... 2 passed`).

| Criterion | Evidence | Status |
|---|---|---|
| parse_language_hotkey is tri-state Combo / Disabled ("3") / Unknown | registry::tests::parse_language_hotkey_is_tri_state ok | verified |
| ("3","3") gives no blocker | registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination (row ["3","3"]) ok; loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker ok (0 blockers, no MaskOnly warning) | verified |
| Disabled has no fallback to Alt+Shift | same registry test, rows ["3",None], [None,"3"], ["3","2"], ["x","3"], [" 3 ","1"] + laziness check; M1 catches | verified |
| ("2", anything) gives Ctrl+Shift | registry::tests::the_hotkey_value_decides_before_the_language_hotkey_value ok; loader::the_registry_values_choose_the_auto_blocker_keys ok | verified |
| ("1", anything) gives Alt+Shift | same two tests ok | verified |
| no values give Alt+Shift | registry::tests::no_usable_value_falls_back_to_alt_shift ok (None/None, empty iterator, unknown); loader row [None,None] | verified |
| only missing/unparsable falls back | no_usable_value_falls_back_to_alt_shift; a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide ok | verified |
| Hotkey read before Language Hotkey by name | registry::tests::the_language_hotkey_is_read_from_hotkey_then_language_hotkey ok (M2) | verified |
| Other functions keep their rules | registry::tests::other_functions_keep_their_rules ok | verified |
| Registry read-only, resolution injectable | `git diff master..HEAD` added lines matching RegSetValue/RegCreateKey/RegDelete/SendInput/SetWindowsHook: 0. `hotkey_from_values` takes injected values; remaining live reads are read-only (`KEY_READ`) | verified |
| CHANGELOG entry | CHANGELOG.md:411-422 `- KS-17 (roadmap item 1.4): ...` under Unreleased | verified |
| Real Windows setting "Not assigned" -> daemon prints new [INFO], no [AUTO-BLOCK]; Alt+Shift / Ctrl+Shift live (cut M1-M3); ignored live test `live_registry_with_the_language_hotkey_not_assigned_has_no_combination` | `#[ignore = "manual: ..."]` | manual, not verified |

No pure-function criterion is left to a manual line only.

## Failures
None.

## Tests added by the tester
none (no commits made; lane tree clean).

## Notes
- No test in the diff binds the production pipe name, writes a real config.ron, or writes the registry. Tests still reading the live registry are read-only and accept any machine state (e.g. `test_get_system_hotkey_returns_combinations` allows None for SwitchLanguage), so they pass on a machine with "Not assigned" too.
- No timing/real-clock tests in the lane, so no flake reruns were needed.
- Unverified by design: which value Windows honours when `Hotkey` and `Language Hotkey` differ (cut Q1); `Layout Hotkey` is not read (non-goal).
- Gate log: D:/tmp/ks-orch/p1-1.4-hotkey-tristate/gate_test_head.log. Target dirs under `target-wt/` are outside the repo.
