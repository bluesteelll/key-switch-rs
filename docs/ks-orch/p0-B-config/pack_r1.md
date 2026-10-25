# Context pack r1: p0-B-config (Workstream B: KS-21, KS-22, KS-23, KS-24, KS-27, docs drift)

Scout: project-analyst, read-only. Facts only; `UNVERIFIED` marks inference. Readers: tester, code-reviewer.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/config-edge-cases`, HEAD `983edeb`, base `master` = `4177540`
  (`git merge-base --is-ancestor master HEAD` true). Tree clean, `git worktree list` shows only the main tree.
- Inputs: `D:/tmp/ks-orch/p0-B-config/cut.md` (CUT READY, D1-D9, C1-C12, T0-T23, M1-M9),
  `.../critique.md` (I1, M1, M2; all marked RESOLVED in `cut.md:790-824`), `.../impl.md`.
- Gates: impl.md "Gates" says fmt/check/clippy pass on every commit and 272 lib tests pass at HEAD for the filters
  `config:: hook::config_watcher core::app engine:: build`; full `cargo test --locked` and `tests/` were left to the
  tester. The scout did not re-run any gate.

### Commits (`git log --oneline master..fix/config-edge-cases`, oldest first)

| # | SHA | Parent | Subject | Red? |
|---|---|---|---|---|
| C1 | `13ce323` | `4177540` | Docs: fix documentation drift in the README, the template and code comments | green |
| C2 | `5acbe61` | `13ce323` | Config: seams for the config path, the watcher filter and the reload step | green |
| C3 | `cdbeace` | `5acbe61` | Tests: red-first: relative and differently cased config paths (KS-21) | RED |
| C4 | `c568508` | `cdbeace` | Config: make the config path absolute and match watcher events by file name (KS-21) | green |
| C5 | `d0cadd1` | `c568508` | Tests: red-first: a missing config file is never created except beside the exe (KS-23) | RED |
| C6 | `a03c9ac` | `d0cadd1` | Config: write the template only at startup beside the exe (KS-23) | green |
| C7 | `fe5ebca` | `a03c9ac` | Tests: red-first: unknown fields and a missing bindings list (KS-22) | RED |
| C8 | `abd3031` | `fe5ebca` | Config: reject unknown fields and require bindings (KS-22) | green |
| C9 | `efed5c6` | `abd3031` | Tests: red-first: a UTF-8 BOM (KS-24) | RED |
| C10 | `249c7fa` | `efed5c6` | Config: ignore a UTF-8 BOM (KS-24) | green |
| C11 | `8584cfb` | `249c7fa` | Tests: red-first: timing values outside 1..=10000 ms (KS-27) | RED |
| C12 | `983edeb` | `8584cfb` | Config: accept only 1..=10000 ms for terms, max_gap_ms and window_ms (KS-27) | green |

Red commits touch only tests plus test scaffolding: C3 adds `src/config/testing.rs`, the `mod testing` line and one README
tree line; C5, C7, C9 and C11 touch `src/config/loader.rs` and/or `src/hook/config_watcher.rs` test modules only
(`git show --stat`).

### `git diff --stat master..fix/config-edge-cases` (17 files, +981 / -125)

`CHANGELOG.md` 57, `Cargo.toml` 1, `README.md` 102, `book/src/methodology.md` 2, `build.rs` 11, `src/bin/swch.rs` 1,
`src/config/default_config.ron` 51, `src/config/loader.rs` 459, `src/config/mod.rs` 6, `src/config/parsing.rs` 2,
`src/config/schema.rs` 17, `src/config/testing.rs` 49 (new), `src/core/windows_actions.rs` 7, `src/data/binding.rs` 3,
`src/data/trigger.rs` 5, `src/hook/config_watcher.rs` 318, `src/main.rs` 15.
Not in the diff: `src/engine/**`, `src/hook/keyboard_hook.rs`, `src/hook/keyboard_hook_callback.rs`,
`src/hook/ipc_server.rs`, `src/ipc.rs`, `src/core/app.rs`, `Cargo.lock`, `book/src/config/*.md`, `.claude/**`.

## File map

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/main.rs:22-38` | `run()`: `ConfigPath::from_arg(args_os().nth(1))` :28, `config::load(&config)` :30, `with_config_watcher(config.into_path())` :32; comment :23-27 | KS-21 "absolute once in main, before load and watcher"; KS-23 CLI path must exist |
| `src/config/mod.rs:1-22` | module doc :1-9 (now "`config.ron` loading" :1); `#[cfg(test)] pub(crate) mod testing` :14-15; `pub use loader::{ConfigPath, load, reload}` :17 | public API change (`default_config_path` no longer re-exported; master had `pub use loader::{default_config_path, load}`) |
| `src/config/loader.rs:30-37` | `DEFAULT_CONFIG` :32, `TIMING_RANGE_MS = 1..=10_000` :37 | KS-27 bounds |
| `src/config/loader.rs:39-96` | `ConfigError` with new `Missing { path }` :42-44; Display arm :67-69 (`config file {} does not exist`); `source()` `_ => None` :93 | R6 Display completeness; KS-23 |
| `src/config/loader.rs:98-149` | `default_config_path` :99-108 (still `pub fn`, private module); `ConfigPath` enum :111-118; `from_arg` :127-136 (`std::path::absolute`), `path` :138, `into_path` :144 | KS-21 |
| `src/config/loader.rs:151-209` | `load(&ConfigPath)` :158-173; `reload(&Path)` :178-182; `read_config` :187-197 (`NotFound` -> `Missing`); `parse_config` :200-209 | KS-23 startup-only template; reload never writes |
| `src/config/loader.rs:229-269` | `from_ron_str_with_hotkeys`: BOM strip :236; RON parse :245-248; per-binding error collection :253-262; empty-list warning :264-266 | KS-24; KS-22 empty list |
| `src/config/loader.rs:314-339` | `write_default`: `create_dir_all(parent)` :318-325; `OpenOptions::write(true).create_new(true)` :330-334; `AlreadyExists` -> `Ok(())` :336 | KS-23 / KS-41 partly |
| `src/config/loader.rs:341-382` | `raw_binding_to_binding`: kind :345, action :347-348, trigger :350-351, deferred-trigger rule :356-362, `allow_extra_modifiers` rule :367-372 | KS-27 error prefix `binding #{index} [{keys}]` :343 |
| `src/config/loader.rs:416-458` | `sequence_spec_to_kind` (`max_gap_ms` check :428); `chord_spec_to_kind` (`window_ms` check :453) | KS-27 |
| `src/config/loader.rs:473-497` | fallible `raw_trigger_to_trigger` :473-482; `check_timing_ms` :487-497 | KS-27 |
| `src/config/loader.rs:625-661` | test module head; `default_config_parses` :633-639; T0 `default_config_commented_examples_load` :642-661 | acceptance "template still loads" |
| `src/config/loader.rs:1024-1121` | T15 :1025, T16 :1035-1097, T17 :1100, T19 :1107-1121 | KS-22, KS-24 |
| `src/config/loader.rs:1124-1128` | `empty_config_is_ok_but_warned` (explicit `bindings: []`) | must stay green |
| `src/config/loader.rs:1702-1773` | `after_binding_0` :1706; T21 :1711-1723; T22 :1726-1760; T23 :1763-1773 | KS-27 |
| `src/config/loader.rs:1847-1927` | T1 :1849, T2 :1861, T9 :1867, T10 :1880, T11 :1892, T20 :1904, T12 :1915 | KS-21, KS-23, KS-24 |
| `src/config/schema.rs:18-29` | comment :18-19; `RawConfig` `deny_unknown_fields` :22; `bindings` without `#[serde(default)]` :24-27 | KS-22 |
| `src/config/schema.rs:30-68` | `RawBinding` `deny_unknown_fields` :31; fields incl. `allow_extra_modifiers` :66-67 | KS-22 (A2 field covered) |
| `src/config/schema.rs:82-108` | `SequenceSpec` :85-90 (`max_gap_ms` default 500 :102-104); `ChordSpec` :95-100 (`window_ms` default 50 :106-108) | KS-22, KS-27 |
| `src/config/schema.rs:133-170` | `RawAction` `deny_unknown_fields` :137; struct variants `PostMessage` :146-152, `Launch` :156-160 | KS-22 struct variants |
| `src/config/schema.rs:193-210` | `RawStep` `deny_unknown_fields` :197; `Delay(u64)` :200; struct variant `Launch` :204-208 | KS-22 macro `Launch` |
| `src/config/testing.rs:1-49` | `ONE` :8, `TWO` :11-14, `TempDir` :18-49 (`Relaxed` counter :24-28, pre-removal :35, Drop :45-49) | R4 test-only atomic; test isolation |
| `src/config/default_config.ron:1-20`, `:26-30`, `:49-58`, `:87-89`, `:110-117`, `:135-138` | header (reload, delete, CLI path, field names, BOM) :3-10; kinds line :18; timing line :20; F13/F14 examples :26-30; key names :49-58; Launch :87-89; "Terms are 1 to 10000 ms." :110; Tap/Hold :112-117; three new steps :135-138 | docs drift; T0 parses examples :188-288 |
| `src/hook/config_watcher.rs:1-21` | module doc: name rule :7-14, missing file keeps bindings :18-21 | docs |
| `src/hook/config_watcher.rs:42-91` | `WatchTarget { dir, name: Vec<u16> }` :43-47; `new` :51-56; `dir` :59; `wants_reload` :66-75; `same_file_name` :82-91 (the only new `unsafe`, :86-90) | KS-21; unsafe inventory |
| `src/hook/config_watcher.rs:96-170` | `spawn_watcher` :96-129 (no-parent error :100-103, replaces master's `expect` at master `:67`); `debounce_loop` :131-154; `reload_into` :158-170 | KS-21/22/23 reload path |
| `src/hook/config_watcher.rs:172-374` | tests: helpers :184-215; T3 :218, T4 :230, T5 :246, T8 :259, T6 :281, T7 :286-300, T13 :303-324, T18 :327-346, T14 :349-373 | acceptance |
| `src/core/app.rs:155-178` | (unchanged) watcher spawned after hook install; `Err` -> `[WARN]` + `[INFO] continuing without hot-reload` :167-174 | R10: watcher error is non-fatal |
| `src/hook/keyboard_hook.rs:110-120` | (unchanged) `update_bindings` sort :113 + one `ArcSwap` store :114; `bindings()` :120 | R7; reload path |
| `Cargo.toml:30-34`, `:41-50` | `[[bin]] key-switch-rs` `test = false` :33; feature `Win32_Globalization` added :43 | D1 rationale; new feature, `Cargo.lock` unchanged |
| `README.md:29`, `:33`, `:41`, `:45`, `:57` | tree: `mod.rs - public API: ConfigPath, load(), reload()`, `testing.rs`, `condition.rs`, `trigger.rs`, `properties.rs` | docs drift (module tree) |
| `README.md:69-131` | Modules: `BindAction` variants :79-92, `Binding` fields :98-105 | docs drift (Modules) |
| `README.md:139-170` | hot reload, error list, BOM :139-149; "Where the file lives" :151-170 | KS-21/22/23/24/27 docs |
| `README.md:287-288`, `:355`, `:426-437`, `:505-528`, `:631`, `:643-655` | kinds table ranges; `<ms>` rule; "Eight step variants" + table; Errors; `Win32_UI_Shell`; Features / Implementation Features | docs drift + new rules |
| `CHANGELOG.md:91-116`, `:256-273`, `:303-315` | Changed (Breaking) KS-22, CLI path (KS-23/KS-41 partly), KS-27; Fixed KS-21, KS-23, KS-24; Fixed docs drift | Keep a Changelog |
| `book/src/methodology.md:430` | one added sentence: cfgcheck after Workstream B calls `config::reload(&path)` | deviation from cut touch set (impl.md "Deviations") |
| `build.rs:5-11`, `src/bin/swch.rs:66`, `src/core/windows_actions.rs:16-37`, `src/data/binding.rs:121-138`, `src/data/trigger.rs:34-36`, `src/config/parsing.rs:56` | comment and `#[allow(dead_code)]` drift fixes | C1 docs drift |

## Facts

### KS-21 (path and watcher)

1. `ConfigPath::from_arg(Some(arg))` maps `std::path::absolute(&arg)` to `CommandLine`, and an error to
   `ConfigError::Other("config path {arg:?} cannot be used: {e}")` (`src/config/loader.rs:127-133`); `None` maps to
   `BesideExe(default_config_path()?)` (`:134`).
2. `main` calls `from_arg` before `config::load` and passes `config.into_path()` to `with_config_watcher`
   (`src/main.rs:28-32`); master used `PathBuf::from(arg)` (`git diff master..HEAD -- src/main.rs`).
3. `WatchTarget::new` returns `None` if `parent()` or `file_name()` is `None` (`src/hook/config_watcher.rs:51-56`);
   for `Path::new("my.ron")`, std returns `parent() == Some("")` (std `Path::parent` docs), so a relative target has
   `dir == ""` (T3 `config_watcher.rs:219` uses it; whether `notify` accepts `watch("")` is UNVERIFIED and not
   exercised by any test).
4. `wants_reload` accepts only `Modify(_) | Create(_) | Remove(_)` and compares only `file_name()` of each event path
   with `same_file_name` (`config_watcher.rs:66-75`); the folder part is not compared (doc `:11-14`).
5. `same_file_name` returns `false` if either slice is empty, else `CompareStringOrdinal(a, b, true) == CSTR_EQUAL`
   (`config_watcher.rs:82-91`); the SAFETY comment is at `:86-89`.
6. `wants_reload` allocates one `Vec<u16>` per event path (`config_watcher.rs:72`); it runs in notify's callback
   closure (`:107-115`), not on the hook thread.
7. `spawn_watcher` maps a `None` target to `notify::Error::generic("the config path has no parent folder or file
   name").add_path(..)` (`config_watcher.rs:100-103`); `App::run` turns a watcher `Err` into a warning and continues
   (`src/core/app.rs:162-175`).
8. The watch is `RecursiveMode::NonRecursive` on `target.dir()` (`config_watcher.rs:104`, `:121`).
9. `.expect("spawn config-watcher thread")` remains in production code (`config_watcher.rs:126`, present on master).

### KS-23 (missing file)

10. `read_config` maps `io::ErrorKind::NotFound` to `ConfigError::Missing`, every other error to `Io`; no `exists()`
    pre-check (`src/config/loader.rs:187-197`).
11. `load` calls `write_default` only for `(ConfigPath::BesideExe(_), Err(Missing))`, prints `[INFO] generated default
    config at ..`, then reads again (`loader.rs:160-165`); every other combination propagates the read result and prints
    `[INFO] loading config from ..` (`:166-170`).
12. `reload` = `read_config` + print + `parse_config`; it has no write path (`loader.rs:178-182`).
13. `write_default` uses `create_new(true)`; `AlreadyExists` returns `Ok(())` (`loader.rs:330-337`), after which
    `load` still prints "generated default config" (`:163`) (impl.md "Open points").
14. `reload_into` stores via `update_bindings` only on `Ok`; on `Err` it prints `[ERROR] config reload failed:\n{e}` and
    `[INFO] keeping previous bindings live` (`config_watcher.rs:158-170`).

### KS-22 (unknown fields)

15. `#[serde(deny_unknown_fields)]` is on `RawConfig` (`src/config/schema.rs:22`), `RawBinding` (`:31`),
    `SequenceSpec` (`:85`), `ChordSpec` (`:95`), `RawAction` (`:137`) and `RawStep` (`:197`).
16. `RawConfig::bindings` has no `#[serde(default)]` (`schema.rs:24-27`); `bindings: []` still loads and prints
    `[WARN] config contains no bindings` (`loader.rs:264-266`; test `:1124-1128`).
17. `RawTrigger`, `RawCondition`, `MediaKeyRef`, `MessageRef`, `RawWindowKind` carry no `deny_unknown_fields`
    (`schema.rs:74`, `:122`, `:173`, `:188`, `:214`); cut D4 says they have no named fields (`cut.md:230`).
18. Schema doc comments still show `sequence: (steps: ["g", "g"])` (`schema.rs:82`) and `chord: (keys: ["j", "k"])`
    (`schema.rs:93`); `schema.rs:14` mentions TOML as the format RON was chosen over.

### KS-24 (BOM)

19. `from_ron_str_with_hotkeys` strips one leading `'\u{feff}'` before RON parsing (`loader.rs:236`); every loader
    entry (`load`, `reload`, `from_ron_str`) passes through it (`:200-201`, `:222-223`).

### KS-27 (timing range)

20. `check_timing_ms` accepts `TIMING_RANGE_MS.contains(&ms)` and otherwise returns `"{written} is out of range:
    timings are 1 to 10000 ms"` with bounds read from the constant (`loader.rs:487-497`).
21. Checked values: `Tap`/`Hold`/`DoubleTap` (`loader.rs:476-479`), `max_gap_ms` (`:428`), `window_ms` (`:453`).
    `Delay(u64)` steps are not range-checked (`schema.rs:200`; cut non-goal `cut.md:114`).
22. Order in `raw_binding_to_binding`: kind (incl. `max_gap_ms`/`window_ms`) `:345`, action `:347-348`, trigger
    `:350-351`, then the deferred-trigger rule `:356-362`; errors carry `binding #{index} [{keys}]` with a 0-based index
    (`:343`, `:253-254`).
23. In `sequence_spec_to_kind` the range check runs after the step-count and step-parse checks (`loader.rs:417-428`);
    in `chord_spec_to_kind` after the key loop (`:436-453`).
24. Engine-side `Trigger::Tap/Hold/DoubleTap(u64)` are unchanged (`src/data/trigger.rs:23-30`); the range lives only in
    the loader.

### Docs drift (C1 and per-item README/template edits)

25. Template header no longer says a restart is needed (`src/config/default_config.ron:3-5`); kinds line uses
    "(F13+F14 together)" (`:18`); the top-of-file examples use `["F13", "F14"]` (`:26-30`).
26. Template lists eight step variants incl. `Launch`, `OpenUrl`, `Media` (`default_config.ron:125-138`); lines
    `:135-138` use ASCII `-` while older lines use `—` (e.g. `:125`, `:130`).
27. README says "Eight step variants:" (`README.md:426`) and its table has 8 rows ending with `Media(<key>)` (`:430-437`).
28. README tree lists `condition.rs` (`README.md:41`), `trigger.rs` (`:45`), `engine/` with `matching.rs` and
    `properties.rs` (`:47-57`); every listed `src/engine/*.rs` file exists (`ls src/engine`).
29. README Modules lists 10 `BindAction` variants (`README.md:83-92`) and the `Binding` fields incl. `kind`,
    `condition`, `trigger`, `allow_extra_modifiers`, `is_auto_blocker` (`:98-105`).
30. "Hot-path allocations: none" does not occur in `README.md` (grep `-i alloc` finds no README hit); cut marks the
    row "gone, rewritten by Workstream A" (`cut.md:57`). README `:643` and `:653` describe the binding list and the
    engine `Mutex`.
31. `src/config/mod.rs:1` now says "`config.ron` loading"; `src/config/parsing.rs:56` says "numeric literal in the
    config"; `grep -rni toml src` (excluding `Cargo.toml`) returns only `schema.rs:14`.
32. README Errors: the paragraph "Bindings and steps are numbered from 0 ..." (`README.md:526-527`) sits between the
    list items at `:522-524` and the item `- unknown variant 'FlyToMars'` at `:528`.
33. README `:175` still has the comment `// Equivalent to the auto-generated default.` (cut Q6, left as follow-up,
    `cut.md:115`, `:749-751`).
34. Gaps rows marked "fixed by A2..A6" or "gone" are not touched by this lane (`cut.md:68-83`).

### Public API and changelog

35. Library API changes: `config::load(&Path)` -> `config::load(&ConfigPath)`; new `config::reload(&Path)` and
    `config::ConfigPath`; `config::default_config_path` no longer re-exported (`src/config/mod.rs:17` vs master
    `pub use loader::{default_config_path, load};`). `grep -n "default_config_path\|ConfigPath" CHANGELOG.md` finds no
    entry for this API change.
36. CHANGELOG `### Changed` gains three **Breaking** entries (`CHANGELOG.md:91-116`) and `### Fixed` gains KS-21,
    KS-23, KS-24 (`:256-273`) and the docs-drift entry (`:303-315`).
37. Stale agent docs outside the lane: `.claude/agents/doc-writer.md:38` (exports "only `load` and
    `default_config_path`"), `:83` (`config::load` writes to any missing path), `.claude/agents/code-reviewer.md:25`
    (`config::load` in the reload path); impl.md lists `:83` and `:25` as orchestrator items. `src/core/app.rs:84`
    names `config::load` / `config::from_ron_str` in a doc comment.

### Unsafe, atomics, threads

38. The only new `unsafe` block is `config_watcher.rs:90` (with SAFETY `:86-89`); the only new atomic is the test-only
    `AtomicUsize::fetch_add(.., Relaxed)` with a pairing comment (`src/config/testing.rs:24-28`).
39. Every `.expect(` added in the diff is inside a `#[cfg(test)]` module (`git diff master..HEAD -- src | grep expect`:
    loader tests, `config_watcher` tests, `testing.rs:36`).
40. T7 and T14 start a real `notify` watcher on `%TEMP%` with a `Box::leak`ed `KeyboardHook::new()` that is never
    installed (`config_watcher.rs:194-196`, `:286-300`, `:349-373`); `_watcher` is declared after the `TempDir` so it
    drops first (`:292-293`, `:355-356`).
41. `TempDir::new` names folders `<temp>\key-switch-rs-test-<pid>-<n>-<label>` and removes a same-named leftover first
    (`testing.rs:29-36`).

## Red-first replay (exact commands)

Convention (`.claude/agents/tester.md:70-85`): scratch worktree under `L=D:/tmp/ks-orch/p0-B-config`, separate
`CARGO_TARGET_DIR`, remove the worktree afterwards. Each red commit contains the test on its parent's code, so the
replay point can be the red commit itself (or its parent with the test hunks copied); the fix commit and HEAD must be
green. On `master` the C3/C5 tests do not compile (`cut.md:376-377`), so `master` is not the red point.

```bash
L=D:/tmp/ks-orch/p0-B-config
git -C D:/key-switch-rs/key-switch-rs worktree add --detach "$L/wt/red" <RED_SHA>
CARGO_TARGET_DIR="$L/target-wt/red" cargo test --locked --manifest-path "$L/wt/red/Cargo.toml" --lib <TEST_PATH> -- --exact
git -C D:/key-switch-rs/key-switch-rs worktree remove --force "$L/wt/red"; git -C D:/key-switch-rs/key-switch-rs worktree prune
```

| Red SHA (code of) | Fix SHA | Test path (`--lib`, `--exact`) | Expected red (impl.md, commit body) | Expected on fix/HEAD |
|---|---|---|---|---|
| `cdbeace` (C2) | `c568508` | `config::loader::tests::a_relative_config_argument_is_made_absolute` | ``assertion `left == right` failed: my.ron`` left `"my.ron"`, right `"<cwd>\\my.ron"` | pass |
| `cdbeace` | `c568508` | `config::loader::tests::an_empty_config_argument_is_an_error` | `an empty config path must be an error: CommandLine("")` | pass |
| `cdbeace` | `c568508` | `hook::config_watcher::tests::an_event_for_a_relative_config_path_matches` | `a Modify of <cwd>\my.ron must reload the config given as my.ron (KS-21)` | pass |
| `cdbeace` | `c568508` | `hook::config_watcher::tests::an_event_matches_the_config_file_name_in_any_case` | panic message `Event { kind: Modify(Any), paths: [..\\MY.RON"] ..` | pass |
| `cdbeace` | `c568508` | `hook::config_watcher::tests::a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file` | `no reload within 10 s: the watcher of CONFIG.RON never swapped in the 2 bindings of config.ron` (~10 s) | pass (~0.3 s) |
| `cdbeace` | `c568508` | `hook::config_watcher::tests::other_files_and_other_event_kinds_do_not_match`, `...::a_root_path_has_no_watch_target` | green on C2 (characterization) | pass |
| `d0cadd1` (C4) | `a03c9ac` | `config::loader::tests::a_missing_command_line_config_is_an_error_and_is_not_created` | `a missing command-line config must be an error: [Binding { kind: Combo(..` | pass |
| `d0cadd1` | `a03c9ac` | `config::loader::tests::a_missing_folder_of_a_command_line_config_is_not_created` | `a command-line config in a missing folder must be an error: [Binding ..` | pass |
| `d0cadd1` | `a03c9ac` | `config::loader::tests::reload_of_a_missing_config_is_an_error_and_writes_nothing` | `reloading a missing config must be an error: [Binding ..` | pass |
| `d0cadd1` | `a03c9ac` | `hook::config_watcher::tests::a_deleted_config_keeps_the_live_bindings` | `a deleted config must not be recreated` | pass |
| `d0cadd1` | `a03c9ac` | `hook::config_watcher::tests::a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted` | `a deleted config must not be recreated` (~1.8 s) | pass |
| `d0cadd1` | `a03c9ac` | `config::loader::tests::a_missing_config_beside_the_exe_is_generated_and_loaded` | green (characterization; reads the registry read-only via the template's `block_original_combo`) | pass |
| `fe5ebca` (C6) | `abd3031` | `config::loader::tests::a_misspelled_bindings_field_is_an_error_that_names_it` | ``a misspelled `bindings` must be an error: []`` | pass |
| `fe5ebca` | `abd3031` | `config::loader::tests::unknown_fields_are_errors_that_name_the_field` | `version: an unknown field must be an error: [Binding { .. VIRTUAL_KEY(124) ..` | pass |
| `fe5ebca` | `abd3031` | `config::loader::tests::a_config_without_bindings_is_an_error` | ``a config without `bindings` must be an error: []`` | pass |
| `fe5ebca` | `abd3031` | `hook::config_watcher::tests::a_save_with_a_misspelled_field_keeps_the_live_bindings` | ``a misspelled `bindings` must keep the live bindings (KS-22)`` left 0, right 1 | pass |
| `efed5c6` (C8) | `249c7fa` | `config::loader::tests::a_utf8_bom_before_the_config_is_ignored` | ``a UTF-8 BOM must be ignored: config parse error (<in-memory>): 1:1: Expected opening `(` for struct `RawConfig` `` | pass (also asserts `1:51:`) |
| `efed5c6` | `249c7fa` | `config::loader::tests::reload_reads_a_file_saved_with_a_bom` | ``a file saved with a BOM must load: config parse error (<temp>\config.ron): 1:1: Expected opening `(` ..`` | pass |
| `8584cfb` (C10) | `983edeb` | `config::loader::tests::a_zero_tap_term_is_an_error_that_names_the_binding` | `Tap(0) must be an error: [Binding { kind: Combo(.. VIRTUAL_KEY(131) ..` | pass; exact `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms` |
| `8584cfb` | `983edeb` | `config::loader::tests::timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding` | `Hold(0) must be an error: [Binding ..` | pass (8 rows) |
| `8584cfb` | `983edeb` | `config::loader::tests::timing_values_at_the_bounds_load` | green throughout (off-by-one guard) | pass |

Green-on-arrival tests (no red point): T0 `config::loader::tests::default_config_commented_examples_load` (C1),
T8 `hook::config_watcher::tests::same_file_name_ignores_case_like_the_file_system` (C4); existing
`config::loader::tests::default_config_parses` and `empty_config_is_ok_but_warned` must stay green.

Full gate on HEAD (tester owns): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`;
`cargo test --locked` (read `test result:` lines). impl.md: no `#[ignore]` added; T7/T14 add about 0.3 s and 2 s.

## Acceptance mapping (from `cut.md:648-667`)

| Acceptance (brief / `book/src/roadmap.md:117`) | Tests |
|---|---|
| relative path (the watcher matches) | T1 `loader.rs:1849`, T3 `config_watcher.rs:218`, T7 `:286` (real notify, case) |
| case-insensitive file name in the watched folder | T4 `:230`, T5 `:246`, T8 `:259`, T7 |
| deleted file (bindings kept) | T13 `config_watcher.rs:303`, T14 `:349`, T11 `loader.rs:1892` |
| missing explicit CLI path is an error, nothing created | T9 `loader.rs:1867`, T10 `:1880` |
| template only at startup beside the exe | T12 `loader.rs:1915` |
| typo `bindngs:` names the field | T15 `loader.rs:1025`; reload keeps bindings T18 `config_watcher.rs:327` |
| `deny_unknown_fields` incl. `allow_extra_modifiers` | T16 `loader.rs:1035` (10 rows; row 5 `allow_extra_modifier`), T17 `:1100` |
| BOM | T19 `loader.rs:1107`, T20 `:1904` |
| `Tap(0)` names the binding | T21 `loader.rs:1711`; ranges T22 `:1726`, bounds T23 `:1763` |
| default template still loads, through its test | `default_config_parses` `loader.rs:633`, T0 `:642`, T12 |
| docs drift | C1 diff vs `cut.md:44-84`; facts 25-34 |

Manual checks (human only, no ignored test): M1-M9 in `cut.md:669-681` (relative `my.ron` from `C:\tools`;
`C:\TOOLS\MY.RON`; delete and rename keep bindings; `nope.ron` / `--help` give `[FATAL] config file .. does not exist`;
PS 5.1 `Set-Content -Encoding UTF8`; `bindngs:` save; first start beside the exe; `Tap(0)` save).

## Open points from impl.md

1. `write_default` meeting `AlreadyExists` still prints `[INFO] generated default config at ..` (`loader.rs:163`,
   `:336`).
2. Out of scope per cut: 8.3 short names in notifications (RK3, `cut.md:689`), buffer-overflow rescan (RK4, `:690`),
   UTF-16 files (RK9, `:695`), internal serde names `RawConfig`/`RawBinding` in RON messages (Q4, `:743-745`).
3. Orchestrator items: `.claude/agents/doc-writer.md:83`, `.claude/agents/code-reviewer.md:25` (fact 37).
4. Deviations declared in impl.md: one sentence in `book/src/methodology.md:430` (cut said `book/` untouched);
   `TempDir::new` pre-removes a leftover folder (`testing.rs:33-35`); `ONE`/`TWO` in `config::testing`; T16 rows 3-5
   spelled out; template Named line extra aliases; README Errors texts captured with cfgcheck
   (`D:/tmp/ks-orch/p0-B-config/cfgcheck/`, examples in `.../ron/`).
5. Book pages (`book/src/config/*.md`, `audit.md` status lines, `roadmap.md`, `gaps.md`, `architecture.md`) are left to
   the phase book sync (`cut.md:719-732`, addendum `:806-809`).

## Invariants in play

- **R1** hook callback: untouched (`src/hook/keyboard_hook_callback.rs`, `src/engine/**` not in the diff).
- **R2** sentinel: no `SendInput` added (diff scan).
- **R3** shared state: tests use `KeyboardHook::new()` or a leaked one, never `get_hook()`
  (`config_watcher.rs:194-196`, `:307`, `:331`); `reload_into` touches the hook only via `update_bindings`
  (`:162`).
- **R4** orderings: test-only `Relaxed` counter with comment (`testing.rs:25-28`); no production atomics.
- **R5** builders: `ConfigPath` is a plain enum; `App::with_config_watcher(PathBuf)` unchanged (`src/core/app.rs:94`);
  `#[allow(dead_code)]` removed from `with_block_default`/`with_condition`/`with_trigger` (`src/data/binding.rs:121-138`).
- **R6** Display: `ConfigError::Missing` arm at `loader.rs:67-69`.
- **R7** specificity: reload still ends in `update_bindings` sort + store (`keyboard_hook.rs:110-115`); error paths
  store nothing (`config_watcher.rs:165-168`).
- **R9** registry: unchanged; `from_ron_str` reads hotkeys only for `block_original_combo` bindings
  (`loader.rs:222-223`, `:286-288`).
- **R10** startup/shutdown: master's `expect("config_path must have a parent directory")` (master
  `config_watcher.rs:67`) is now a returned error (`:100-103`) handled at `app.rs:167-174`.
- **KS-21** `audit.md` (pinned) -> `main.rs:28`, `loader.rs:127-136`, `config_watcher.rs:42-91`.
- **KS-22** -> `schema.rs:22-27`, `:31`, `:85`, `:95`, `:137`, `:197`.
- **KS-23** / **KS-41** (partly) -> `loader.rs:158-197`, `:317-339`.
- **KS-24** -> `loader.rs:236`.
- **KS-27** -> `loader.rs:37`, `:428`, `:453`, `:473-497`.
- **KS-15** (template examples eat typed keys) -> template `default_config.ron:26-30`, `:260-272`.

## Not covered

- Not read: `src/engine/**`, `src/hook/keyboard_hook_callback.rs`, `src/hook/ipc_server.rs`, `src/ipc.rs`,
  `tests/ipc_roundtrip.rs`, `book/src/config/*.md`, the README sections outside the ranges cited above (e.g. `:172-425`
  apart from listed lines, `:530-640`, `:662-700`), and the full CHANGELOG text beyond `:86-116`.
- Not verified by the scout: gate results, test timings, RON message texts at run time, `notify` behaviour for an empty
  watch directory (fact 3), and the cfgcheck validation runs under `D:/tmp/ks-orch/p0-B-config/ron/`.
- `git blame` was not used; per-line commit attribution beyond `git show --stat` per commit is not given.
