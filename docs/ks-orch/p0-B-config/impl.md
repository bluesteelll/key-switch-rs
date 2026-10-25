# Implementation: p0-B-config round 1

Branch `fix/config-edge-cases` from `master` @ `4177540`, HEAD `983edeb`, tree clean. 12 commits, exactly the cut's
C1-C12. Declared red commits: C3, C5, C7, C9, C11 (fmt and clippy pass on each; only the named tests fail).

## Items

- **Docs drift (C1, `13ce323`)**: `src/config/default_config.ron` (header :3-5, kinds :18, examples :26-30, Named/Media
  key names :49-58, Launch :87-89, Tap/Hold :112-117, step variants :137-140), `README.md` (tree, Modules, key table,
  Launch row, triggers, "Eight step variants", Run, swch aliases, `Win32_UI_Shell`, "Native APIs first", "Priority
  bindings"), `src/config/mod.rs:1`, `src/config/parsing.rs:56`, `src/data/trigger.rs:34-36`,
  `src/core/windows_actions.rs` (`#[allow(dead_code)]` removed, Launch doc), `src/data/binding.rs` (three
  `#[allow(dead_code)]` removed; no lint fired), `build.rs:5-11`, `src/bin/swch.rs` usage `help`. Test
  `config::loader::tests::default_config_commented_examples_load` (green on arrival, guards the examples).
- **Seams (C2, `5acbe61`)**: `ConfigPath` (`src/config/loader.rs:112`), `load(&ConfigPath)` / `reload(&Path)` on the old
  semantics, `WatchTarget` (old exact compare) and `reload_into` (`src/hook/config_watcher.rs`), `main.rs:28-34`;
  the `expect` on a parentless path became a `notify::Error::generic(..)`. Behaviour unchanged, no test changes.
- **KS-21** - `src/config/loader.rs:127-137` (`std::path::absolute`), `src/hook/config_watcher.rs:43-91` (`WatchTarget
  { dir, name }`, `same_file_name` with `CompareStringOrdinal(.., true)`), `Cargo.toml:43` `Win32_Globalization`
  (`Cargo.lock` unchanged) - red-first (C3 `cdbeace`, on C2) -> green (C4 `c568508`):
  - `a_relative_config_argument_is_made_absolute`: red "assertion `left == right` failed: my.ron / left: "my.ron" /
    right: "D:\\key-switch-rs\\key-switch-rs\\my.ron"" -> green
  - `an_empty_config_argument_is_an_error`: red "an empty config path must be an error: CommandLine("")" -> green
  - `an_event_for_a_relative_config_path_matches`: red "a Modify of <cwd>\my.ron must reload the config given as my.ron
    (KS-21)" -> green
  - `an_event_matches_the_config_file_name_in_any_case`: red "Event { kind: Modify(Any), paths:
    ["D:\\key-switch-rs\\key-switch-rs\\MY.RON"], .. }" -> green
  - `a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file` (real notify): red "no reload within 10 s: the
    watcher of CONFIG.RON never swapped in the 2 bindings of config.ron" (after 10.0 s) -> green (~0.3 s)
  - `other_files_and_other_event_kinds_do_not_match`, `a_root_path_has_no_watch_target`: green on C2 and after
  - `same_file_name_ignores_case_like_the_file_system` (C4): green on arrival (ASCII, Cyrillic, `.bak`, empty)
- **KS-23** (+ KS-41 partly) - `src/config/loader.rs:42-44` (`ConfigError::Missing`), `:67-69` (Display), `:158-175`
  (`load`), `:178-182` (`reload`, never writes), `:187-197` (`read_config`, `NotFound` -> `Missing`), `:200`
  (`parse_config`), `:317-339` (`write_default` with `create_new`, `AlreadyExists` -> `Ok`) - red-first (C5 `d0cadd1`,
  on C4) -> green (C6 `a03c9ac`):
  - `a_missing_command_line_config_is_an_error_and_is_not_created`: red "a missing command-line config must be an
    error: [Binding { kind: Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }), .." -> green
  - `a_missing_folder_of_a_command_line_config_is_not_created`: red "a command-line config in a missing folder must be
    an error: [Binding .." -> green
  - `reload_of_a_missing_config_is_an_error_and_writes_nothing`: red "reloading a missing config must be an error:
    [Binding .." -> green
  - `hook::config_watcher::tests::a_deleted_config_keeps_the_live_bindings`: red "a deleted config must not be
    recreated" -> green
  - `hook::config_watcher::tests::a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted` (real notify; first
    proves the watcher reloads): red "a deleted config must not be recreated" (~1.8 s) -> green
  - `a_missing_config_beside_the_exe_is_generated_and_loaded`: green on C4 and after (characterization)
- **KS-22** - `src/config/schema.rs:22`, `:31`, `:85`, `:95`, `:137`, `:197` (`deny_unknown_fields` on `RawConfig`,
  `RawBinding`, `SequenceSpec`, `ChordSpec`, `RawAction`, `RawStep`), `:24-27` (`bindings` required) - red-first (C7
  `fe5ebca`, on C6) -> green (C8 `abd3031`):
  - `a_misspelled_bindings_field_is_an_error_that_names_it`: red "a misspelled `bindings` must be an error: []" -> green
  - `unknown_fields_are_errors_that_name_the_field` (10 rows, incl. `allow_extra_modifier`, `Launch(cwd:)`,
    `PostMessage(wparm:)`, macro `Launch(argz:)`): red "version: an unknown field must be an error: [Binding { kind:
    Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(124))] }), .." -> green
  - `a_config_without_bindings_is_an_error`: red "a config without `bindings` must be an error: []" -> green
  - `hook::config_watcher::tests::a_save_with_a_misspelled_field_keeps_the_live_bindings`: red "assertion `left ==
    right` failed: a misspelled `bindings` must keep the live bindings (KS-22) / left: 0 / right: 1" -> green
  - `empty_config_is_ok_but_warned` (explicit `bindings: []`): green throughout
- **KS-24** - `src/config/loader.rs:236` (`strip_prefix('\u{feff}')` first in `from_ron_str_with_hotkeys`) - red-first
  (C9 `efed5c6`, on C8) -> green (C10 `249c7fa`):
  - `a_utf8_bom_before_the_config_is_ignored`: red "a UTF-8 BOM must be ignored: config parse error (<in-memory>): 1:1:
    Expected opening `(` for struct `RawConfig`" -> green (also asserts `1:51:` for the truncated file)
  - `reload_reads_a_file_saved_with_a_bom`: red "a file saved with a BOM must load: config parse error
    (<temp>\config.ron): 1:1: Expected opening `(` for struct `RawConfig`" -> green
- **KS-27** - `src/config/loader.rs:37` (`TIMING_RANGE_MS`), `:473-484` (fallible `raw_trigger_to_trigger`), `:487-497`
  (`check_timing_ms`), `:428`, `:453` (sequence/chord), trigger error prefixed in `raw_binding_to_binding` - red-first
  (C11 `8584cfb`, on C10) -> green (C12 `983edeb`):
  - `a_zero_tap_term_is_an_error_that_names_the_binding`: red "Tap(0) must be an error: [Binding { kind:
    Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(131))] }), .." -> green (exact message `binding #1 [CapsLock]: Tap(0)
    is out of range: timings are 1 to 10000 ms`)
  - `timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding` (8 rows): red "Hold(0) must be an error:
    [Binding .." -> green
  - `timing_values_at_the_bounds_load` (1 and 10000): green throughout

Tests added: 24. `config::loader::tests` (15): T0, T1, T2, T9-T12, T15-T17, T19-T23. `hook::config_watcher::tests` (9):
T3-T8, T13, T14, T18. New test-only module `src/config/testing.rs` (`TempDir`,
`ONE`, `TWO`).

## Remarks

- I1 (Important) - RESOLVED: the post-lane cfgcheck entry point is `config::reload(&Path)` (public, never writes from
  C6, `src/config/loader.rs:178`); `book/src/methodology.md:430` (C2) adds one sentence after the exit-code table so a
  cfgcheck for the current tree calls `reload`. The verbatim block stays, because it is also step 3 of the `7cb3521`
  audit reproduction (`methodology.md:567`). Evidence: `D:/tmp/ks-orch/p0-B-config/cfgcheck/` (methodology source with
  `reload`) builds and runs on C2..C12. Remaining book prose, `doc-writer.md:83`, `code-reviewer.md:25`: addendum at
  the end of `cut.md` (book-sync list and orchestrator items).
- M1 (Minor) - RESOLVED at `README.md:29` (C2: `public API: ConfigPath, load(), reload()`); `architecture.md:23` added to
  the book-sync list in the cut addendum.
- M2 (Minor) - RESOLVED in C1, README "Native APIs first" bullet: `ToggleCapsLock` is listed with the `SendInput`
  outputs.
- Critique open questions 1 (book deferral) and 2 (Q5, real-notify tests by default): orchestrator's; implemented as
  the cut says (no book config pages touched; T7 and T14 run by default). Open question 3 (cleanup race on red
  parents): no stray `key-switch-rs-test-*` folder in `%TEMP%` after the C3/C5 red runs (checked).

## Deviations from the cut

- `book/src/methodology.md` gets one sentence (C2), although the cut lists `book/` as not touched: needed to resolve I1
  (see Remarks). Alternative considered: editing the verbatim harness (breaks the `7cb3521` reproduction) or only the
  addendum (the book sync would build a harness that does not compile).
- `TempDir::new` removes a leftover folder of the same name before creating it (a reused pid could hand a test stale
  files); the cut only says "a new empty folder".
- `ONE`/`TWO` live in `config::testing` so the loader and watcher tests share them.
- T16 rows 3-5 use `( keys: "CapsLock", action: SwitchLanguage, .. )` (the cut writes `..`). T14 sleeps
  `6 * DEBOUNCE_MS` (1500 ms, as specified).
- C1 template Named line also lists `Escape`, `Return`, `Del`, `Ins`, `PgUp`, `PgDn` (as the README table does), and
  the Launch note says "for a console program, its console" (the code inherits the console only for console programs).
- `same_file_name`'s doc notes that the `windows` wrapper converts the lengths to `i32` with a panic on overflow, which
  a Windows path cannot reach.
- README Errors list uses the real RON texts captured with cfgcheck (`tigger`, missing `bindings`, `bindngs`); the
  CHANGELOG KS-22 example is the T15 message `1:10: Unexpected field named `bindngs` in `RawConfig`, expected
  `bindings` instead`.

## Unsafe / atomics added

- `src/hook/config_watcher.rs:90` - SAFETY: "CompareStringOrdinal reads a.len() and b.len() UTF-16 units from the two
  slices, which are borrowed for the whole call; it writes nothing and keeps no pointer. A failure returns 0, which is
  not CSTR_EQUAL, so it counts as different names." Runs on notify's backend thread, no hook state.
- `src/config/testing.rs` (test-only) - `AtomicUsize::fetch_add(1, Relaxed)`: "only the uniqueness of the returned value
  matters, an RMW is atomic under any ordering, and no other data is published through the counter."
- No production atomics, no lock, nothing on the hook callback path.

## Gates

- `cargo fmt --check` / `cargo check --all-targets --locked` / `cargo clippy --all-targets --locked -- -D warnings`:
  pass on every commit (HEAD rechecked).
- Tests run: every red-first above, on its parent and on its fix commit; per commit `cargo test --locked --lib --
  config:: hook::config_watcher` (plus `engine::` after C8 and C12, `core::app` after C2/C6, `readme` after C1): HEAD
  272 passed, 0 failed (`config:: hook::config_watcher core::app engine:: build`). Full suite and integration tests
  left to the tester.
- Manual ignores added: none (no test needs a hook, injection or a registry write; T7 and T14 use ReadDirectoryChangesW
  on `%TEMP%`, ~0.3 s and ~2 s). Manual checks for a human: cut M1-M9 (relative path `my.ron` from `C:\tools` reloads;
  `C:\TOOLS\MY.RON` reloads; delete and rename `config.ron` keep bindings, no new file; `key-switch-rs.exe nope.ron`
  and `--help` stop with `[FATAL] config file ... does not exist` and create nothing; PS 5.1 `Set-Content -Encoding
  UTF8` save reloads and restarts; `bindngs:` save rejected, old count kept; first start beside the exe generates the
  template; `Tap(0)` save rejected with the binding number).
- Loader validation (cfgcheck, `reload` entry point, `D:/tmp/ks-orch/p0-B-config/ron/`): template OK 3 bindings; the six
  README `ron` blocks OK (5, 3, 4, 4, 1, 2 bindings); `bindngs` / `()` / `tigger` / `Launch(cwd:)` / BOM+truncated /
  `Tap(0)`+`max_gap_ms: 0` give the messages quoted in README and CHANGELOG; BOM file OK 1 binding.

## Docs

- README: C1 drift; C2 tree exports; C3 `testing.rs` line; C4 relative path and any-case reload, `Win32_Globalization`;
  C6 missing-file rules and error line; C8 unknown-field/missing-`bindings` errors, hot-reload bullet; C10 BOM; C12
  ranges, `<ms>` rule, `Tap(0)` error, 0-based numbering.
- `default_config.ron` header lines per C1, C6, C8, C10, C12; `default_config_parses` and T0 green.
- CHANGELOG `[Unreleased]`: Changed (Breaking) KS-22, command-line path (KS-23/KS-41 partly), KS-27; Fixed KS-21, KS-23,
  KS-24, documentation drift.
- Book: one sentence in `methodology.md` (I1). Config pages and the rest left to the phase book sync (cut Docs impact
  plus the addendum).

## Commits

- `13ce323` Docs: fix documentation drift in the README, the template and code comments
- `5acbe61` Config: seams for the config path, the watcher filter and the reload step
- `cdbeace` Tests: red-first: relative and differently cased config paths (KS-21) (RED)
- `c568508` Config: make the config path absolute and match watcher events by file name (KS-21)
- `d0cadd1` Tests: red-first: a missing config file is never created except beside the exe (KS-23) (RED)
- `a03c9ac` Config: write the template only at startup beside the exe (KS-23)
- `fe5ebca` Tests: red-first: unknown fields and a missing bindings list (KS-22) (RED)
- `abd3031` Config: reject unknown fields and require bindings (KS-22)
- `efed5c6` Tests: red-first: a UTF-8 BOM (KS-24) (RED)
- `249c7fa` Config: ignore a UTF-8 BOM (KS-24)
- `8584cfb` Tests: red-first: timing values outside 1..=10000 ms (KS-27) (RED)
- `983edeb` Config: accept only 1..=10000 ms for terms, max_gap_ms and window_ms (KS-27)

## Open points / pre-existing warnings

- No pre-existing warnings.
- Per D3, a `write_default` that meets `AlreadyExists` (file created between the read and the write) still prints
  `[INFO] generated default config at ..`; the file is loaded as it is.
- Out of scope, noted by the cut: 8.3 short names in notifications (RK3), buffer-overflow rescan (RK4), UTF-16 files
  (RK9), internal serde names in RON messages (Q4).
- Orchestrator items from the addendum: `.claude/agents/doc-writer.md:83`, `.claude/agents/code-reviewer.md:25`.
