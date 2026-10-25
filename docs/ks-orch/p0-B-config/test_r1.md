# Tester: p0-B-config / Workstream B (KS-21, KS-22, KS-23, KS-24, KS-27, docs drift) (HEAD 983edeb vs master 4177540)

Verdict: GREEN. Gate green, 22 red-first/guard tests checked on their red commits (18 red by assertion, 4 declared characterization/guard green) and all green on HEAD, 13 own mutations (17 mutated runs) all caught, every acceptance criterion mapped to a passing test. No test added by the tester.

HEAD `983edeb` on `fix/config-edge-cases`; tree clean (`git status --short` empty) at start and end; `git worktree list` shows only the lane tree.

## Gate (HEAD)
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (re-run after `touch` of src/lib.rs, main.rs, bin/swch.rs, config/loader.rs: "Checking key-switch-rs ... Finished `dev` profile in 2.90s")
- cargo test --locked --no-fail-fast (log: D:/tmp/ks-orch/p0-B-config/test_gate_full.log):
  - lib: `running 363 tests` ... `test result: ok. 362 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.95s`
  - swch bin: `running 0 tests` (that target has no tests)
  - tests/ipc_roundtrip.rs: `running 6 tests` ... `test result: ok. 6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests`
  - Total 368 passed, 0 failed, 1 ignored. The ignored test is pre-existing: `data::condition::tests::foreground_context_names_the_focused_notepad ... ignored, manual: start it, then focus a Notepad window within 5 s ...` (manual, not verified). The lane added no `#[ignore]`.
- cargo build --release --locked: OK (`Finished release profile [optimized] target(s) in 50.02s`; built, never run)
- Safety scan of the diff (production pipe name, set_current_dir, set_var, registry write, SendInput): no hit. New tests use `TempDir` (`config::testing`) and `KeyboardHook::new()` never installed; none uses `get_hook()`. `a_missing_config_beside_the_exe_is_generated_and_loaded` reads the registry read-only via the template and writes only into a temp dir.
- Flake check: `cargo test --locked --lib hook::config_watcher` x5: `9 passed` each time (1.81-1.87 s). These use a real notify watcher; stable over 5 runs.

## Red-first replay
Master is not a usable red point for the KS-21/KS-23 tests: they use the seams added by `5acbe61` (`ConfigPath`, `WatchTarget`, `reload`, `config::testing`) and do not compile on master (compile-red proves nothing). The replay point is each `Tests:` commit itself (the test on the unfixed code of its parent), in a scratch worktree `D:/tmp/ks-orch/p0-B-config/wt/red` with `CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-B-config/target-wt/red`. Every run printed `running 1 test`. For KS-22/24/27 the loader/schema paths under test are identical on master and on the red commit, so the red result applies to master by that argument (not run on master itself).

| Test (`--lib --exact`) | Red commit | Result on red commit | HEAD |
|---|---|---|---|
| config::loader::tests::a_relative_config_argument_is_made_absolute | cdbeace | red (assert): left `"my.ron"`, right `"...\\wt\\red\\my.ron"` | ok |
| config::loader::tests::an_empty_config_argument_is_an_error | cdbeace | red (assert): `an empty config path must be an error: CommandLine("")` | ok |
| hook::config_watcher::tests::an_event_for_a_relative_config_path_matches | cdbeace | red (assert): `a Modify of <cwd>\my.ron must reload the config given as my.ron (KS-21)` | ok |
| hook::config_watcher::tests::an_event_matches_the_config_file_name_in_any_case | cdbeace | red (assert, config_watcher.rs:209) | ok |
| hook::config_watcher::tests::a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file | cdbeace | red (assert, :242, 10.05 s timeout) | ok |
| hook::config_watcher::tests::other_files_and_other_event_kinds_do_not_match | cdbeace | ok (characterization, declared) | ok |
| hook::config_watcher::tests::a_root_path_has_no_watch_target | cdbeace | ok (characterization, declared) | ok |
| config::loader::tests::a_missing_command_line_config_is_an_error_and_is_not_created | d0cadd1 | red (assert): `a missing command-line config must be an error: [Binding {...` (default template loaded) | ok |
| config::loader::tests::a_missing_folder_of_a_command_line_config_is_not_created | d0cadd1 | red (assert): same, template loaded | ok |
| config::loader::tests::reload_of_a_missing_config_is_an_error_and_writes_nothing | d0cadd1 | red (assert): `reloading a missing config must be an error` | ok |
| hook::config_watcher::tests::a_deleted_config_keeps_the_live_bindings | d0cadd1 | red (assert): `a deleted config must not be recreated` | ok |
| hook::config_watcher::tests::a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted | d0cadd1 | red (assert): `a deleted config must not be recreated` (1.82 s) | ok |
| config::loader::tests::a_missing_config_beside_the_exe_is_generated_and_loaded | d0cadd1 | ok (characterization, declared) | ok |
| config::loader::tests::a_misspelled_bindings_field_is_an_error_that_names_it | fe5ebca | red (assert): `a misspelled bindings must be an error: []` | ok |
| config::loader::tests::unknown_fields_are_errors_that_name_the_field | fe5ebca | red (assert): `version: an unknown field must be an error` | ok |
| config::loader::tests::a_config_without_bindings_is_an_error | fe5ebca | red (assert): `a config without bindings must be an error: []` | ok |
| hook::config_watcher::tests::a_save_with_a_misspelled_field_keeps_the_live_bindings | fe5ebca | red (assert_eq left 0, right 1): `a misspelled bindings must keep the live bindings (KS-22)` | ok |
| config::loader::tests::a_utf8_bom_before_the_config_is_ignored | efed5c6 | red (assert): `1:1: Expected opening ( for struct RawConfig` | ok |
| config::loader::tests::reload_reads_a_file_saved_with_a_bom | efed5c6 | red (assert): same parse error with the temp path | ok |
| config::loader::tests::a_zero_tap_term_is_an_error_that_names_the_binding | 8584cfb | red (assert): `Tap(0) must be an error: [Binding {... trigger: Tap(0)` | ok |
| config::loader::tests::timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding | 8584cfb | red (assert): `Hold(0) must be an error` | ok |
| config::loader::tests::timing_values_at_the_bounds_load | 8584cfb | ok (off-by-one guard, declared) | ok |

Counts: 22 tests replayed, 18 red by assertion on the red commit, 4 green there as declared by the developer (other_files_and_other_event_kinds_do_not_match, a_root_path_has_no_watch_target, a_missing_config_beside_the_exe_is_generated_and_loaded, timing_values_at_the_bounds_load). No compile-red.

HEAD run of 26 names (the 22 above + `default_config_commented_examples_load`, `default_config_parses`, `empty_config_is_ok_but_warned`, `same_file_name_ignores_case_like_the_file_system`): `running 26 tests` ... `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 1.82s`.
Worktree removed with `git worktree remove --force` + `prune`; `git worktree list` shows only the lane tree.

## Mutations
Scratch worktree `D:/tmp/ks-orch/p0-B-config/wt/mut` at 983edeb, `CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-B-config/target-wt/mut`. Each reverted with `git -C wt/mut checkout -- .`, `git diff --exit-code` clean; worktree removed at the end.

| # | File:line | Mutation | Caught by | Result |
|---|---|---|---|---|
| M1 | src/hook/config_watcher.rs:90 | `CompareStringOrdinal(a, b, true)` -> `false` (case-sensitive) | same_file_name_ignores_case_like_the_file_system, an_event_matches_the_config_file_name_in_any_case, a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file | caught (3 failed) |
| M2 | src/config/loader.rs:161 | `load` writes the template for any path, not only `BesideExe` | a_missing_command_line_config_is_an_error_and_is_not_created, a_missing_folder_of_a_command_line_config_is_not_created | caught (2 failed) |
| M3 | src/config/loader.rs:37 | `1..=10_000` -> `1..=9_999` | timing_values_at_the_bounds_load, a_zero_tap_term_is_an_error_that_names_the_binding, timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding | caught (3 failed) |
| M4 | src/config/loader.rs:236 | BOM strip removed | a_utf8_bom_before_the_config_is_ignored, reload_reads_a_file_saved_with_a_bom | caught (2 failed) |
| M5 | src/hook/config_watcher.rs:167 | `reload_into` Err branch calls `update_bindings(Vec::new())` | a_deleted_config_keeps_the_live_bindings, a_save_with_a_misspelled_field_keeps_the_live_bindings, a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted | caught (3 failed) |
| M6 | src/hook/config_watcher.rs:69 | `wants_reload` drops `EventKind::Remove(_)` | an_event_matches_the_config_file_name_in_any_case | caught (1 failed) |
| M7 | src/config/schema.rs:31 | `deny_unknown_fields` removed from `RawBinding` | unknown_fields_are_errors_that_name_the_field | caught |
| M8 | src/config/schema.rs:24 | `#[serde(default)]` added on `RawConfig.bindings` | a_config_without_bindings_is_an_error | caught |
| M9a-d | src/config/schema.rs:85, 95, 137, 197 | `deny_unknown_fields` removed from SequenceSpec, ChordSpec, RawAction, RawStep, one at a time | unknown_fields_are_errors_that_name_the_field (each of 4 runs) | caught x4 |
| M10 | src/config/loader.rs:129 | `from_arg` uses `PathBuf::from(&arg)` instead of `std::path::absolute` | a_relative_config_argument_is_made_absolute, an_empty_config_argument_is_an_error | caught (2 failed) |
| M11 | src/config/loader.rs:453 | chord `window_ms` range check removed | timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding | caught |
| M12 | src/config/loader.rs:428 | sequence `max_gap_ms` range check removed | timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding | caught |
| M13 | src/config/loader.rs:37 | `1..=10_000` -> `0..=10_000` | a_zero_tap_term_is_an_error_that_names_the_binding, timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding | caught (2 failed) |

Notes: one transient `link.exe` LNK1104 on the first M3 build (file lock in the scratch target dir); the identical re-run built and the mutation was caught. The first M4 `sed` did not match (no diff, 67 passed); redone with a line-addressed edit and caught. No uncaught mutation.

## Acceptance (brief and cut.md:648-667)
| Criterion | Evidence | Status |
|---|---|---|
| Relative path: the watcher matches | `a_relative_config_argument_is_made_absolute`, `an_event_for_a_relative_config_path_matches`, real notify `a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file`; all ok; M10 | verified |
| Watcher compares file name case-insensitively within the watched dir | `an_event_matches_the_config_file_name_in_any_case`, `same_file_name_ignores_case_like_the_file_system`, `other_files_and_other_event_kinds_do_not_match`; M1, M6 | verified |
| main.rs makes the path absolute once, before load and watcher | `main.rs:28-32` read: `ConfigPath::from_arg(args_os().nth(1))?`, then `config::load(&config)?`, then `with_config_watcher(config.into_path())`. `from_arg` is tested; the call order in `run()` has no automated test | partly verified; wiring is a manual check |
| Deleted file: bindings kept, file not recreated | `a_deleted_config_keeps_the_live_bindings`, `a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted`, `reload_of_a_missing_config_is_an_error_and_writes_nothing`; M5 | verified |
| Missing explicit CLI path is an error, nothing created | `a_missing_command_line_config_is_an_error_and_is_not_created`, `a_missing_folder_of_a_command_line_config_is_not_created`; M2 | verified |
| Template generated only at startup beside the exe | `a_missing_config_beside_the_exe_is_generated_and_loaded` ok | verified |
| Typo `bindngs:`: the error names the field | `a_misspelled_bindings_field_is_an_error_that_names_it` (asserts `named \`bindngs\``); reload keeps bindings: `a_save_with_a_misspelled_field_keeps_the_live_bindings` | verified |
| `deny_unknown_fields` on RawConfig, RawBinding, SequenceSpec, ChordSpec (and RawAction, RawStep), including `allow_extra_modifiers` | `unknown_fields_are_errors_that_name_the_field` (10 rows, one is `allow_extra_modifier`); M7, M9a-d | verified |
| `bindings` required; explicit `bindings: []` still loads with a warning | `a_config_without_bindings_is_an_error`, `empty_config_is_ok_but_warned`; M8 | verified |
| BOM stripped | `a_utf8_bom_before_the_config_is_ignored`, `reload_reads_a_file_saved_with_a_bom`; M4 | verified |
| `Tap(0)`: the error names the binding | `a_zero_tap_term_is_an_error_that_names_the_binding` asserts exactly `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms` | verified |
| Terms, `max_gap_ms`, `window_ms` limited to 1..=10000 | `timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding` (8 rows), `timing_values_at_the_bounds_load`; M3, M11, M12, M13 | verified |
| Default template still loads | `default_config_parses`, `default_config_commented_examples_load`, `a_missing_config_beside_the_exe_is_generated_and_loaded`, all ok | verified |
| Docs drift (greps on HEAD) | template header `default_config.ron:4` "250 ms after the save, without a restart"; examples `["F13", "F14"]` at `default_config.ron:26,29,270` and `README.md:301`; `README.md:426` "Eight step variants:"; README tree lists `condition.rs` `:41`, `trigger.rs` `:45`, `engine/` `:47`; `grep -n "Five step\|Hot-path alloc" README.md` empty; no "toml" in `src/config/mod.rs` or `parsing.rs` | verified by grep |

Manual, not verified (cut M1-M9; need a live daemon or a shell): relative `my.ron` from another folder; `C:\TOOLS\MY.RON` case form; delete/rename of the live file; `nope.ron` / `--help` giving `[FATAL] config file .. does not exist`; PowerShell 5.1 `Set-Content -Encoding UTF8` save; first start beside the exe; `bindngs:` and `Tap(0)` saved under a running daemon. Plus the pre-existing ignored Notepad test.

## Failures
None.

## Tests added by the tester (`Tests:` commits)
none

## Notes for the reviewer
- Not a failure: `src/config/schema.rs:82` and `:93` doc comments still show `sequence: (steps: ["g", "g"])` and `chord: (keys: ["j", "k"])`. They are source comments, not the user template the gap row names; the template and README were fixed. `schema.rs:14` keeps "over TOML" as rationale text, not drift.
- `load` prints `[INFO] generated default config` even when `write_default` met `AlreadyExists` (impl.md open point 1); message only.
- `run()` wiring in `main.rs` (from_arg before load and watcher) has no automated test; M10 shows the function is guarded, the call order is guarded by reading only.
- Public API change (`default_config_path` no longer re-exported; `load(&ConfigPath)`) is a library break; the scout found no CHANGELOG line naming `ConfigPath`.
- The release build and tests touched only the lane's `target/` and `D:/tmp/ks-orch/p0-B-config/target-wt/`; the lane tree has no uncommitted change and no scratch worktree remains.
