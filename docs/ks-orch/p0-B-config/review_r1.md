# Code review: p0-B-config @ 983edeb
APPROVED (Critical 0, Important 0, Minor 2)

Scope: `git diff master...fix/config-edge-cases` (master `4177540`, HEAD `983edeb`, 12 commits C1-C12, 17 files),
read through `git show`/`git diff`; every touched source file read in full at `983edeb`
(`src/config/loader.rs` production part and the new tests, `schema.rs`, `mod.rs`, `testing.rs`,
`src/hook/config_watcher.rs`, `src/main.rs`), call sites of the changed API (`src/core/app.rs:83-178`,
`src/hook/keyboard_hook.rs:66-120`, `src/bin/swch.rs` `cmd_open`), plus the cut (D1-D9, addendum I1/M1/M2) and impl.md.

## Gate: fmt ✅ · clippy ✅

- `cargo fmt --check` in `D:/key-switch-rs/key-switch-rs` at `983edeb`: exit 0 (`D:/tmp/ks-orch/p0-B-config/review_fmt.log`).
- `cargo clippy --all-targets --locked -- -D warnings` with `CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-B-config/target-review`
  (fresh target): exit 0, no warnings (`.../review_clippy.log`; rustc/clippy 1.98.1). Tree clean and HEAD `983edeb`
  before and after.
- `cargo test` not run (tester owns it).
- Extra, read-only: cfgcheck (`D:/tmp/ks-orch/p0-B-config/cfgcheck`, calls `config::reload`) rebuilt against `983edeb`
  into my target dir. The template and all six README `ron` blocks load (3, 5, 3, 4, 4, 1, 2 bindings). The README and
  CHANGELOG error texts match what the loader prints: `tigger` (1:65 ``Unexpected field named `tigger` in `RawBinding`, expected one of ...``),
  `bindngs` (1:10), `()` (``Unexpected missing field named `bindings` in `RawConfig` ``), `Launch(cwd:)` (rejected in
  `Launch`), `Tap(0)` + `max_gap_ms: 0` (`binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`,
  `binding #2 [...]: max_gap_ms: 0 ...`), BOM (OK), BOM + truncated (`1:51:`).

## Remarks

### 🔴 Critical

None.

### 🟡 Important

None.

### 🟢 Minor (optional)

#### m1. README Errors: the numbering note splits the error list
**Where**: `README.md:526-528` (at `983edeb`)  **Confidence**: CONFIRMED
**Problem**: The paragraph "Bindings and steps are numbered from 0 in file order: ..." is placed between the new
`Tap(0)` item (`:522-524`) and the old item `` - `unknown variant 'FlyToMars'` `` (`:528`).
**Failure**: In CommonMark the unindented paragraph ends the list. `unknown variant 'FlyToMars'` then renders as a
separate one-item list below the numbering note, cut off from the other error messages.
**What to do**: Move the paragraph after the last list item, or move the `FlyToMars` item above it.

#### m2. Schema doc comments still use the letter examples that the template dropped (KS-15)
**Where**: `src/config/schema.rs:82` (`sequence: (steps: ["g", "g"])`), `:93` (`chord: (keys: ["j", "k"])`)
**Confidence**: CONFIRMED
**Problem**: C1 replaced these examples in the template because such bindings swallow typed letters (KS-15). The
schema docs, in a file this lane touched (C8), still show the letters.
**Failure**: Only contributors read these docs, so the risk is small: someone copies `["j", "k"]` from the rustdoc
into a test or a book example, and that example eats typed `j`/`k`. No user-facing artifact is affected.
**What to do**: Use F13/F14, as the template now does.

## Checklist walked (items absent with no reachable failure)

**Plan / roadmap conformance**
- KS-21: the path is made absolute once, before load and the watcher. `main.rs:28` calls `ConfigPath::from_arg`
  (`std::path::absolute`, `loader.rs:127-136`), then `load(&config)` at `:30` and `with_config_watcher(config.into_path())`
  at `:32`. The cut moves the call into the library for testability (D1), which meets the "once in main" intent.
- The watcher compares file names only, with `CompareStringOrdinal(.., true)` (`config_watcher.rs:66-91`). That is
  correct for the non-recursive watch.
- KS-23:
  - The template is written only for `(BesideExe, Missing)` (`loader.rs:160-165`).
  - `reload` has no write path (`:178-182`).
  - `read_config` maps `NotFound`, which covers both `ERROR_FILE_NOT_FOUND` and `ERROR_PATH_NOT_FOUND`, to `Missing`
    without an `exists()` TOCTOU (`:187-197`).
  - `write_default` uses `create_new` (`:330-337`).
  - `reload_into` stores only on `Ok` (`config_watcher.rs:158-170`).
- KS-22: `deny_unknown_fields` is on `RawConfig`, `RawBinding`, `SequenceSpec`, `ChordSpec`, `RawAction` and `RawStep`
  (struct variants covered, confirmed by cfgcheck on `Launch(cwd:)`). `bindings` has no default. Explicit `bindings: []`
  still loads with the warning (D4).
- KS-24: one leading U+FEFF is stripped in `from_ron_str_with_hotkeys` (`loader.rs:236`), which every entry point passes
  through.
- KS-27: `check_timing_ms` covers Tap/Hold/DoubleTap (`:473-482`), `max_gap_ms` (`:428`) and `window_ms` (`:453`). The
  bounds come from `TIMING_RANGE_MS`, and the binding number is in the prefix (`:343`, `:350-351`). The defaults 500 and
  50 are inside the range. No in-tree RON config (`engine::scenarios`, `engine::properties`, `engine/mod.rs`, template,
  README) has an out-of-range value or an unknown field (grep plus cfgcheck).
- Roadmap B acceptance (`roadmap.md:116`):
  - relative path: T1 `loader.rs:1849`, T3 `config_watcher.rs:218`, T7 (real notify, case);
  - deleted file: T13, T14, T11;
  - `bindngs:`: T15, T18;
  - BOM: T19, T20;
  - `Tap(0)`: T21 (exact message);
  - template loads: `default_config_parses`, T0, T12.
  Each red test fails on its parent by assertion; I traced C2/C4/C6/C8/C10 semantics against each one.
- Docs drift: every row of the spec list and every still-true row of `gaps.md` "Documentation drift" is fixed in the
  README, the template or code comments. The `LShift` DoubleTap, sided names, KS-06 and KS-28 rows were already fixed
  by A2-A6. The "Hot-path allocations: none" text is already gone. `schema.rs:14`'s TOML mention is accurate ("chosen
  over TOML").
- `CHANGELOG.md`: entries under `[Unreleased]`. Breaking × 3 under Changed (KS-22, the CLI path with KS-41 partly,
  KS-27); Fixed: KS-21, KS-23, KS-24 and the drift entry.
- Book config pages are left to the phase book sync (cut Docs impact, `docs/WORKFLOW.md:105`), plus one sentence in
  `methodology.md:430` (I1).

**Unsafe**
- One new block, `config_watcher.rs:90`, with a correct SAFETY comment. The windows 0.62.2 wrapper passes
  `as_ptr()`/`len()` of borrowed slices with explicit counts, so no NUL is needed. Empty slices return early, and
  `try_into().unwrap()` cannot overflow for a path component. A failure returns 0, which is treated as "different".
  It runs on notify's thread, never in an `extern "system"` callback.
- No other `unsafe` touched.

**Atomics**
- No production atomics. The test-only `AtomicUsize::fetch_add(Relaxed)` (`testing.rs:24-28`) has a pairing argument:
  uniqueness only, nothing published.

**Win32 / threads**
- No hook, window, timer, pump or COM change.
- `from_arg` runs on the main thread before any other thread exists.
- `CompareStringOrdinal` runs on the notify backend thread.
- The reload runs on `config-watcher` and goes through `update_bindings` only.
- The master's `expect("config_path must have a parent directory")` on the main thread became a returned error
  (`config_watcher.rs:100-103`), which `App::run` downgrades to a warning (`app.rs:167-174`).
- `thread::Builder::spawn(..).expect` at `:126` is pre-existing and runs on the main thread, outside a callback.

**Hot path (R1, section 5)**
- `keyboard_hook_callback.rs` and `src/engine/**` are not in the diff.
- The per-event `Vec<u16>` in `wants_reload` is on notify's thread.

**Suppression invariant**
- Untouched. The lane only changes when `update_bindings` is called: fewer swaps (no swap on a missing file or on an
  unknown field), so pending gestures see fewer reloads. The reset on reload (KS-31) stays with roadmap 2.3.

**R2 sentinel**
- No `SendInput` or `dwExtraInfo` added.

**R3 shared state**
- No new shared state. Tests use `KeyboardHook::new()` or a leaked instance, never `get_hook()`, and never install.

**R5 builders**
- `ConfigPath` is a plain enum with no setters. `App::with_config_watcher(PathBuf)` is unchanged. The three
  `#[allow(dead_code)]` attributes are removed, and clippy stays clean.

**R6 Display**
- `ConfigError::Missing` has an explicit arm (`loader.rs:67-69`). The `source()` arm `_ => None` is correct for it.

**R7 specificity**
- Reload still ends in the single `update_bindings` (stable sort, one store). No second publish path.

**R8 key identity**
- Not touched.

**R9 registry / process reads**
- No new reads. T12 and the template parse read the language hotkey only (read-only).

**R10 message loop / shutdown**
- Startup order is unchanged (hook, IPC, watcher). `_watcher` drops after `uninstall()`.
- A missing CLI file now ends in `[FATAL]` + "Press Enter" before any hook exists. That is the pre-existing fatal path
  and is documented as Breaking.

**Tests**
- Every new test can fail for the stated reason.
- T7 and T14 use real `ReadDirectoryChangesW` on `%TEMP%`: no hook, no injection, no registry write, so no `#[ignore]`
  is needed.
- `TempDir` deletes only its own `key-switch-rs-test-<pid>-<n>-<label>` folders.
- No test changes the current directory, so T1's `current_dir()` cannot race.

**Safety**
- No test or tool run by this review installs a hook, injects input or writes the registry.

## Positive

- Seams first (C2) made every red commit compile on its parent and fail by assertion. The replay table in impl.md
  matches the code.
- Using `NotFound` instead of `exists()` and `create_new` instead of `fs::write` removes both TOCTOU windows in the old
  generate-on-missing path, and a typo path or a missing folder creates nothing (KS-41 scenario).
- Matching only the file name, folded with the OS table, is the right model for a non-recursive
  `ReadDirectoryChangesW` watch. It also fixes the 8.3/`..`/case variants of the folder part for free.
- T16 checks ``named `<field>` `` with backticks, so the expected-names list cannot satisfy it vacuously. T14 proves
  the watcher is live before the delete. T23 guards off-by-one at both bounds.
- The README error texts were captured from the real loader. I confirmed them independently with cfgcheck at `983edeb`.

## Open questions

1. **The book stays stale until the phase book sync.** From the lane's merge until the sync, `book.yml` publishes pages
   that contradict the new behaviour:
   - `config/ron-syntax.md:138` and `triggers-conditions.md:34`: the range is not checked;
   - `config/errors.md:180`: `Tap(0)` loads;
   - `architecture.md:94`: `config::load` writes any missing file.

   This is sanctioned by `docs/WORKFLOW.md:105` and cut Q1, so it is not a remark. The orchestrator decides whether
   the gap between merge and sync is acceptable.
2. **T14 passes either way if notify is slow.** If the delete notification arrives more than about 1.25 s after
   `remove_file`, T14 passes on both fixed and buggy code. It never fails falsely on correct code, and T13 pins the
   same behaviour deterministically. The fact that would settle CI behaviour: notify latency on `windows-latest` under
   load.
3. **Misleading message on a startup race.** `load` prints `[INFO] generated default config` when `write_default`
   meets `AlreadyExists` (`loader.rs:163`, `:336`). This is accepted by D3, cosmetic, and needs a create race at
   startup.
