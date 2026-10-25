# Results analysis: p0-B-config (roadmap Workstream B: KS-21, KS-22, KS-23, KS-24, KS-27, documentation drift)

Branch `fix/config-edge-cases` @ `983edeb`, cut from `master` @ `4177540`. `4177540` is still master HEAD and the merge-base
(`git rev-list --count fix/config-edge-cases..master` = 0). 12 commits, 17 files, +981/-125. The tree is clean,
`git worktree list` shows only the lane tree, no remote branch contains HEAD, and the tester's `wt/` directory is empty.

## VERDICT: ACCEPT

**Summary**: Every acceptance line has automated evidence:
- the brief's acceptance: relative path, deleted file, `bindngs:`, BOM, `Tap(0)` and the template test;
- the roadmap's Workstream B acceptance;
- every spec bullet.

Each regression test fails by assertion on its red commit:
- the tester replayed 22 of them;
- I replayed the KS-23 and KS-22 sets myself from `git archive` copies (6 of 6 red, with the predicted texts).

The gate is green in my own run: 368 passed, 1 pre-existing manual ignore, against 339 + 6 on master. The lane does not
touch the hook callback, the engine or any thread ownership. Review r1 is APPROVED (0 Critical, 0 Important, 2 Minor) and
tester r1 is GREEN. All 12 commits carry the trailer.

What remains is non-blocking:
- two Minor review remarks and two new rustdoc warnings;
- the phase book sync (sanctioned, as in A1-A6);
- follow-up debt that so far lives only in `cut.md`;
- an owner FYI on the three Breaking changes.

## Quality gate (own run, HEAD `983edeb`)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK. Re-checked after a `touch` of `src/lib.rs`, `src/main.rs` and `src/bin/swch.rs` ("Checking key-switch-rs ... Finished in 2.71s"), exit 0 |
| `cargo test --locked` (log `D:/tmp/ks-orch/p0-B-config/analyst-test.txt`) | lib `363 tests`: 362 passed, 0 failed, 1 ignored. swch bin: 0. `tests/ipc_roundtrip.rs`: 6 passed. Doc-tests: 0. **368 passed / 1 ignored** |
| Master count | The diff adds 24 `#[test]` and removes none (`git grep -c '#\[test\]'`: 336 on master, 360 on HEAD; no `-` line with `#[test]`/`#[ignore]`). Master is 339 lib + 6 integration |
| `cargo test --locked -- --list --ignored` | `data::condition::tests::foreground_context_names_the_focused_notepad` only. Pre-existing, `manual:` reason. The lane added no ignore |
| `cargo doc --no-deps` | 4 warnings. **2 are new** (`loader.rs:156`, `:176`: the public docs of `load`/`reload` link to `ConfigError::Missing`, which is unreachable because `loader` is private and `ConfigError` is not re-exported). 2 are pre-existing (`keyboard_hook.rs:132`, `vk_name.rs:4`). See P3 |
| `mdbook build book -d D:/tmp/ks-orch/p0-B-config/book` | OK. Only the known mdbook-mermaid version note |
| Trailer | 12 of 12 commits: `%(trailers:key=Co-Authored-By)` = `Claude Opus 5.5 <noreply@anthropic.com>` on every one |
| `Cargo.lock` | unchanged. `Cargo.toml` gains only the `windows` feature `Win32_Globalization` (cut D2), no new crate |

## Acceptance criteria

| # | Criterion (brief / roadmap B) | Evidence (HEAD file:line) | Status |
|---|---|---|---|
| 1 | KS-21: the path is made absolute once in `main`, before load and watcher (`std::path::absolute`) | `ConfigPath::from_arg` at `src/config/loader.rs:127-136`. Called at `src/main.rs:29`, then `load(&config)` at `:31` and `with_config_watcher(config.into_path())` at `:33`. Tests `a_relative_config_argument_is_made_absolute` (`loader.rs:1849`, `my.ron` and `.\sub\..\my.ron`) and `an_empty_config_argument_is_an_error` (`:1861`). Tester mutation M10 caught. The call order in `main` is read, not tested: the bin has `test = false` (cut D1). Manual M1 | MET |
| 2 | Acceptance "a relative path (the watcher matches)" | `hook::config_watcher::tests::an_event_for_a_relative_config_path_matches` (`config_watcher.rs:218`): a relative target against notify's absolute event path. Real `ReadDirectoryChangesW` test `a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file` (`:286`) | MET |
| 3 | KS-21: the watcher compares file names case-insensitively within the watched folder | `WatchTarget::wants_reload` / `same_file_name` (`CompareStringOrdinal(.., true)`) at `config_watcher.rs:66-91`. Tests `an_event_matches_the_config_file_name_in_any_case` (`:230`), `other_files_and_other_event_kinds_do_not_match` (`:246`), `same_file_name_ignores_case_like_the_file_system` (`:259`, Cyrillic included), `:286`. Mutations M1 and M6 caught | MET |
| 4 | KS-23: the template is generated only at startup, when the default location is missing | `load` at `loader.rs:158-173` writes only for `(BesideExe, Missing)`. `write_default` uses `create_new` (`:317-339`). Test `a_missing_config_beside_the_exe_is_generated_and_loaded` (`:1915`). Mutation M2 caught | MET |
| 5 | Acceptance "a deleted file" (bindings kept, logged) | `reload` never writes (`loader.rs:178-182`). `reload_into` stores only on `Ok` and logs `[ERROR] config reload failed` + `[INFO] keeping previous bindings live` (`config_watcher.rs:158-170`). Tests `a_deleted_config_keeps_the_live_bindings` (`:303`), real-notify `a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted` (`:349`, proves the watcher is live first), `reload_of_a_missing_config_is_an_error_and_writes_nothing` (`loader.rs:1892`). Mutation M5 caught. Checkpoint 1 smoke item 8 (`roadmap.md:236`) | MET |
| 6 | KS-23: a missing EXPLICIT CLI path is an error, not a created file | `a_missing_command_line_config_is_an_error_and_is_not_created` (`loader.rs:1867`), `a_missing_folder_of_a_command_line_config_is_not_created` (`:1880`). `main` prints it as `[FATAL]` | MET |
| 7 | KS-22: `deny_unknown_fields` on RawConfig, RawBinding, SequenceSpec, ChordSpec, plus fields from earlier items (`allow_extra_modifiers`) | `schema.rs:22`, `:31`, `:85`, `:95`, plus `RawAction` `:137` and `RawStep` `:197` for struct variants. `unknown_fields_are_errors_that_name_the_field` (`loader.rs:1035`, 10 rows including `allow_extra_modifier`, `Launch(cwd:)`, `PostMessage(wparm:)`, macro `Launch(argz:)`). Mutations M7 and M9a-d caught | MET |
| 8 | KS-22: `bindings` required (or empty reload rejected) | `#[serde(default)]` removed (`schema.rs:24-27`). `a_config_without_bindings_is_an_error` (`loader.rs:1100`). An explicit `bindings: []` still loads with the warning (`empty_config_is_ok_but_warned`, `:1124`), the spec's first option. Mutation M8 caught. My probe: a 0-byte file, a comment-only file and a BOM-only file are all parse errors, so a reload keeps the live list | MET |
| 9 | Acceptance "the typo `bindngs:` (the error names the field)" | `a_misspelled_bindings_field_is_an_error_that_names_it` (`loader.rs:1025`, asserts ``named `bindngs` ``). Reload side: `a_save_with_a_misspelled_field_keeps_the_live_bindings` (`config_watcher.rs:327`). Checkpoint 1 smoke item 8 | MET |
| 10 | KS-24 / acceptance "a BOM" | Strip at `loader.rs:236`, on the common path of every entry point. `a_utf8_bom_before_the_config_is_ignored` (`:1107`, also pins positions `1:51:` after the BOM), `reload_reads_a_file_saved_with_a_bom` (`:1904`, real bytes `EF BB BF` on disk). Mutation M4 caught | MET |
| 11 | KS-27: 1..=10000 ms for Tap/Hold/DoubleTap, `window_ms`, `max_gap_ms`, error names the binding | `TIMING_RANGE_MS` `loader.rs:37`, `check_timing_ms` `:487-497`, triggers `:473-482` (prefixed `:350-351`), `max_gap_ms` `:428`, `window_ms` `:453`. `timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding` (`:1726`, 8 rows), `timing_values_at_the_bounds_load` (`:1763`). Mutations M3, M11, M12, M13 caught | MET |
| 12 | Acceptance "`Tap(0)` (an error that names the binding number)" | `a_zero_tap_term_is_an_error_that_names_the_binding` (`loader.rs:1711`): exact `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms` | MET |
| 13 | Acceptance "the default template still loads, through its test" | `default_config_parses` (`loader.rs:633`) green. Also `default_config_commented_examples_load` (`:642`, every commented example loads) and `:1915` | MET |
| 14 | Docs drift: template line 3; F13/F14 for `["g","g"]`/`["j","k"]`; README "Five step variants"; module tree (engine, `condition.rs`, `trigger.rs`); Modules section; "Hot-path allocations: none"; TOML in `config/mod.rs`/`parsing.rs`; the rest of `gaps.md` drift that is still true | Template header `default_config.ron:3-10`, examples `:26-30` and `:263`/`:269`. README `:426` "Eight step variants" (8 rows), tree `:28-57` (`condition.rs`, `trigger.rs`, `properties.rs`, stray `└──` removed), Modules `:85-105`, "Hot-path allocations" gone (grep empty). `mod.rs:1` "config.ron", `parsing.rs:56`. I walked every row of `gaps.md:161-202`: the rest of the C1 rows are fixed (key tables, `swch` aliases and `help`, `Launch` "detached", UAC consent/"accept once", `windows` features, "Not macros", "Priority bindings", `build.rs`, `trigger.rs`, `windows_actions.rs`, `#[allow(dead_code)]`). Rows owned by A2-A6 were already fixed. Leftover: `schema.rs:82`/`:93` doc comments (reviewer m2, P2) | MET (one Minor) |
| 15 | Roadmap B "Marks the `LShift` + `DoubleTap` example as not working until 1.3" | Done by A2: `default_config.ron:251-253` | MET (earlier lane) |

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-21 (relative path, letter case) | `a_relative_config_argument_is_made_absolute`, `an_event_for_a_relative_config_path_matches`, `an_event_matches_the_config_file_name_in_any_case`, real notify `..._case_differs_from_the_file` | Tester replay on `cdbeace` (red by assertion; real-notify test red at the 10 s cap). Seams commit `5acbe61` is behaviour-preserving (`git show`): same exact `PathBuf` compare, same `load_or_generate` | CLOSED |
| KS-23 (deleted or renamed file loads the defaults) | `a_deleted_config_keeps_the_live_bindings`, real-notify `..._when_the_file_is_deleted`, `reload_of_a_missing_config_is_an_error_and_writes_nothing`, `a_missing_command_line_config_...`, `a_missing_folder_...` | **Own replay** on `d0cadd1` (archive, own target dir): 3 of 3 red. `reload_...` "reloading a missing config must be an error: [Binding ... SwitchLanguage ...]" (the template loaded), `a_missing_command_line_...` same, `a_deleted_config_...` "a deleted config must not be recreated". Tester: 5 of 5 | CLOSED. KS-41 partly closed: a typo path or `--help` creates nothing. Argument parsing stays with 4.2 |
| KS-22 (unknown fields ignored; misspelled `bindings` empties the config) | `a_misspelled_bindings_field_is_an_error_that_names_it`, `unknown_fields_are_errors_that_name_the_field` (the audit's `tigger`, `block_defualt`, `whne`), `a_config_without_bindings_is_an_error`, `a_save_with_a_misspelled_field_keeps_the_live_bindings` (the audit's "zero bindings live" on reload) | **Own replay** on `fe5ebca`: 3 of 3 red. "a misspelled `bindings` must be an error: []", "version: an unknown field must be an error: [Binding ...]", reload `left: 0 right: 1`. The loader and schema under test equal master's | CLOSED |
| KS-24 (BOM) | `a_utf8_bom_before_the_config_is_ignored`, `reload_reads_a_file_saved_with_a_bom` | Tester replay on `efed5c6`: ``1:1: Expected opening `(` for struct `RawConfig` ``, the audit's exact message | CLOSED |
| KS-27 (unverified; unbounded timings) | `a_zero_tap_term_...`, `timing_values_outside_...` (`Tap(0)`, `Hold(3600000)`, `window_ms: 0`, `max_gap_ms: 0`, the audit's values) | Tester replay on `8584cfb`: "Tap(0) must be an error: [Binding ... Tap(0)]", "Hold(0) must be an error" | CLOSED |

Commit order confirms red-first for each item. Each `Tests:` commit touches only tests (plus `testing.rs` and one README tree
line in `cdbeace`; `git show --stat`), and its fix commit follows it: C3->C4, C5->C6, C7->C8, C9->C10, C11->C12.

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/keyboard_hook*.rs` and `src/engine/**` are not in the diff (`git diff --stat`). The only new per-event allocation (`Vec<u16>` in `wants_reload`, `config_watcher.rs:72`) runs on notify's backend thread. Suppression symmetry is untouched. A rejected or missing file now skips `update_bindings`, so fewer swaps reach pending gestures |
| Engine purity, matching per owner decisions | OK | The engine is untouched. Validation sits in the loader, where the binding number is known. Exact modifiers, `allow_extra_modifiers` and the repeat defaults are unchanged. The config stays RON |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | `from_arg` runs on main before any thread exists. No hook install or uninstall moves. No `SendInput`. One new `unsafe` (`config_watcher.rs:90`) with a concrete SAFETY comment: borrowed slices with explicit lengths, nothing written or kept, failure means "different". One test-only `Relaxed` RMW (`testing.rs:25-28`) with a written argument. No `SeqCst`. The old main-thread `expect("config_path must have a parent directory")` is now a returned error (`config_watcher.rs:100-103`), which `App::run` downgrades to `[WARN]` (`app.rs:167-174`) |
| Config reload, IPC | OK | A failed, missing, 0-byte, comment-only or misspelled reload keeps the live bindings (`reload_into` `config_watcher.rs:158-170`; my cfgcheck probe on HEAD). Unknown fields are errors. Binding errors name `binding #N [keys]`. IPC is untouched. `swch open` passes no argument (`swch.rs:123-130`), so it stays on the `BesideExe` path the lane keeps |
| No key logging | OK | No new log line carries key data |
| Test safety | OK | No test installs a hook, injects input or writes the registry. Real-notify tests watch only `%TEMP%\key-switch-rs-test-<pid>-<n>-<label>`. `KeyboardHook::new()` is never installed, and no test uses `get_hook()` or changes the current folder |

## Review status

- Critique: CHANGES_REQUESTED (I1 Important, M1 and M2 Minor). All three were resolved in the tree (cut addendum, `cut.md:790-823`):
  - I1: `methodology.md:430` points cfgcheck at `config::reload`;
  - M1: README tree `:29`;
  - M2: the README "Native APIs first" bullet lists `ToggleCapsLock` with the `SendInput` outputs.

  I verified each in the diff. cfgcheck on `reload` builds against HEAD (my own build and run).
- Code review r1: **APPROVED**, Critical 0 / Important 0 / Minor 2. Nothing needed triage, so no `triage_r*` or `fix_r*`
  report exists. Both Minor items are CONFIRMED and still open (P1, P2).
- Tester r1: **GREEN**:
  - 22 red-first tests replayed, 18 red by assertion, 4 declared guards;
  - 13 own mutations (17 runs), all caught;
  - watcher tests stable over 5 runs.

## Docs and hygiene

- README, `default_config.ron` and the CHANGELOG change in the same commits as the code that changes the meaning (cut D9, commit
  order).
- CHANGELOG `[Unreleased]`:
  - three **Breaking** entries under `### Changed`: KS-22, the command-line path (KS-23/KS-41 partly), KS-27;
  - `### Fixed`: KS-21, KS-23, KS-24 and the drift entry.

  The headings follow Keep a Changelog, and every entry cites its ID.
- The README error texts were captured from the real loader (impl.md, reviewer cfgcheck). My probe shows the same messages.
- The book is deferred to the phase book sync (`docs/WORKFLOW.md:105-107`), as in A1-A6. The only exception is the
  `methodology.md` sentence. Pages that contradict the new behaviour after the merge, until the sync:
  - `config/structure.md:27`: a missing file is generated;
  - `config/errors.md:180`: `Tap(0)` loads;
  - `config/ron-syntax.md:138`, `triggers-conditions.md:34`: no range check;
  - `architecture.md:94`;
  - `audit.md` status lines for KS-21/22/23/24/27;
  - `known-issues.md` rows 1-4 and 6.

  The list is in `cut.md:715-732`, and the addendum adds `methodology.md:376-428` prose, `glossary.md:437`,
  `known-issues.md:114`, `example.md:8` and `architecture.md:23`.
- The branch holds only this lane's work and merges cleanly with `--no-ff` (master has not moved).

## Technical debt

| Item | Where recorded |
|---|---|
| RK3: `ReadDirectoryChangesW` may report the 8.3 short name; a save that reports only the short name would be missed | `cut.md:689` only. **Not in the repo** -> book sync: `config/known-issues.md` |
| RK4: on notify buffer overflow the path-less rescan event is dropped by the filter, so one save can be missed (fix: treat `need_rescan()` as a reload) | `cut.md:690` only. **Not in the repo** -> known-issues or `gaps.md` |
| RK9: UTF-16 files (Windows PowerShell 5.1 `Out-File` default) fail with "stream did not contain valid UTF-8" | `cut.md:695` only. **Not in the repo** -> known-issues next to the closed KS-24 row 6 |
| Q4: RON messages show internal names (`RawConfig`, `RawBinding`, `SequenceSpec`, `ChordSpec`), now more often | `cut.md:743-745`. The README shows them verbatim. -> `gaps.md` follow-up |
| D3: `[INFO] generated default config` is printed even when `write_default` met `AlreadyExists` (startup create race; cosmetic) | `impl.md:168-169`, review open Q3. Not in the repo |
| `Delay(ms)` steps have no range check | `cut.md:114` non-goal. Not in the repo |
| README `:150` "Equivalent to the auto-generated default" above a block that differs (Q6) | `cut.md:749-751`. Not in the repo. The template `Ctrl+Pause` is known-issues row 51 |
| T14 passes vacuously if the delete notification is slower than about 1.25 s (T13 pins the behaviour deterministically) | Review open Q2. Accept; revisit if CI flakes (cut Q5 fallback) |
| Manual checks M1, M2, M4, M5, M6, M8, M9 (relative path from another folder, `C:\TOOLS\MY.RON`, rename, `nope.ron`/`--help` fatal, PS 5.1 BOM save, first start, `Tap(0)` under a running daemon) | `cut.md:669-681` only. Checkpoint 1 item 8 covers M3/M7. -> add to the lane's or Checkpoint 1's smoke list at the book sync |
| Stale agent files: `.claude/agents/doc-writer.md:83` (says `config::load` writes to any missing path), `.claude/agents/code-reviewer.md:25` (says reload goes through `config::load`); `CLAUDE.md:30` (`app.manifest` unused, cut Q2) | cut addendum. Orchestrator item |
| New `TODO`/`FIXME` in the diff | none (`git diff` grep) |

## Problems

None blocks the merge. They are listed for the orchestrator to handle while the context is fresh.

### P1. README "Errors": the numbering note splits the error list (reviewer m1, CONFIRMED, open)
- **What**: `README.md:526-528`. The unindented paragraph "Bindings and steps are numbered from 0 ..." sits between the `Tap(0)` item (`:522-524`) and `` - `unknown variant 'FlyToMars'` `` (`:528`).
- **Impact**: CommonMark closes the list at the paragraph, so `FlyToMars` renders as a separate one-item list below the note. The meaning is intact; only the layout breaks.
- **Root cause**: the C12 insertion went above the last old item instead of after it.
- **Return to**: `doc-writer`, as one follow-up `Docs:` commit on this branch before the merge or in the book sync. Not blocking.
- **Re-check condition**: `FlyToMars` is the last item of the same list, and the numbering note follows the list.

### P2. Schema doc comments keep the letter examples the template dropped (reviewer m2, CONFIRMED, open)
- **What**: `src/config/schema.rs:82` (`sequence: (steps: ["g", "g"])`), `:93` (`chord: (keys: ["j", "k"])`).
- **Impact**: contributor-only rustdoc. A copied example eats typed letters (KS-15). No user artifact is affected.
- **Root cause**: C1's drift list covered the template and README, not the schema rustdoc.
- **Return to**: `doc-writer` (or the developer in a follow-up). Not blocking.
- **Re-check condition**: `rg -n '"g", "g"|"j", "k"' src/config/schema.rs` is empty.

### P3. Two new rustdoc warnings
- **What**: `src/config/loader.rs:156` and `:176`. The public docs of `config::load`/`config::reload` link to `ConfigError::Missing`. `ConfigError` lives in the private `loader` module and is not re-exported (`src/config/mod.rs:17` exports only `ConfigPath`, `load`, `reload`), so the link cannot resolve.
- **Impact**: none at run time, and the gate does not run rustdoc. It also shows that external callers (cfgcheck) cannot name `ConfigError`, so cfgcheck keeps its own existence pre-check for exit code 2.
- **Root cause**: an intra-doc link to an unreachable item.
- **Return to**: `developer` (follow-up), or decide to re-export `ConfigError`. Not blocking.
- **Re-check condition**: `cargo doc --no-deps` shows only the 2 pre-existing warnings.

## Positives

- **Seams first.** C2 (`5acbe61`) is behaviour-preserving. So every red commit compiles on its parent and fails by assertion,
  which I confirmed on two of them myself.
- **No TOCTOU on either side.** `read_config` maps `NotFound` (which also covers a missing folder) instead of checking
  `exists()`, and `write_default` uses `create_new`. A typo path or `--help` creates nothing, which closes the elevated
  daemon's "write anywhere" behaviour.
- **The right model for a non-recursive watch.** Matching on the file name, folded with the OS table, also fixes the case,
  `..` and 8.3 variants of the folder part.
- **Real `ReadDirectoryChangesW` tests** now exist (roadmap 4.1's "a `config_watcher` test exists"), with a liveness check
  before the delete. They need no hook and no `#[ignore]`.
- **Messages pinned at full strength.** `named `<field>`` with backticks cannot be satisfied by RON's expected-names list, the
  `Tap(0)` message is pinned exactly, and the bounds test guards against off-by-one.
- **Trailers.** All 12 commits carry the correct trailer. This is the first phase-0 lane where criterion 8 fully holds,
  because the tester added no commit.

## Notes for later lanes

- **Owner FYI (Breaking).** A config that used to load can now stop the daemon at startup with `[FATAL]` and "Press Enter":
  - a misspelled or unknown field (`cwd:` in `Launch` included);
  - `()` without `bindings`;
  - a timing of 0 or more than 10000 ms;
  - a missing command-line path.

  Saved while the daemon runs, such a file is rejected and the old bindings stay live. Before deploying, run the live
  `config.ron` through cfgcheck (`D:/tmp/ks-orch/p0-B-config/cfgcheck`, which calls `config::reload` and never writes). No
  `config.ron` exists under `target/release` or `target/debug` on this machine to check.
- **Roadmap 1.1.** Every new field (`repeat`) must be declared in `RawBinding`, which now denies unknown fields (RK11), and the
  template and README must use it in the same commit.
- **Roadmaps 4.3/4.4.** `config::reload(&Path)` is the non-writing loader for `swch check`/`reload`. 4.4 adds a third
  `ConfigPath` source.
- **Book sync for phase 0.**
  - Use the cut's list (`cut.md:715-732`) plus its addendum (`cut.md:806-809`).
  - Add known-issues rows for RK3, RK4 and RK9.
  - Close known-issues rows 1-4 and 6.
  - Validate every changed RON example with cfgcheck on `reload`.
