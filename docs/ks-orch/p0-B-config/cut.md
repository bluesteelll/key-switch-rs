# Cut: p0-B-config — Workstream B: KS-21, KS-22, KS-23, KS-24, KS-27, documentation drift

Tree: `D:/key-switch-rs/key-switch-rs`, `master` @ `4177540` (clean; `.git/refs/heads/master`). Branch to cut:
`fix/config-edge-cases`. Locked crates (`Cargo.lock`): `notify` 8.2.0 (`:134-135`), `notify-types` 2.1.0 (`:152-153`),
`ron` 0.10.1 (`:185-186`), `serde` 1.0.228 (`:213-214`), `windows` 0.62.2 (`:291-292`). Crate facts below are read from
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/<crate>/...` and from upstream sources at the locked tags.

## Verdict

CUT READY

## Spec re-located

The book cites `7cb3521`; "now" is `4177540`.

### Defects

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| KS-21 first CLI argument passed through unchanged `main.rs:28-31` | `src/main.rs:24-31` (`PathBuf::from(arg)` `:29`) | unchanged | defect present |
| KS-21 `spawn_watcher(path)` call `app.rs:107` | `src/core/app.rs:161-178` (call `:162`) | moved | |
| KS-21 `target_path` `config_watcher.rs:43`; filter `p == &target_path` `:47` | `src/hook/config_watcher.rs:43`, `:47` | unchanged | defect present (relative path and letter case) |
| KS-21 watch on parent, `expect("config_path must have a parent directory")` | `src/hook/config_watcher.rs:65-69` | unchanged | panics on `C:\`; replaced in C2 |
| KS-21 `default_config_path()` `loader.rs:83-95` | `src/config/loader.rs:84-94` | moved | already absolute (`current_exe`) |
| notify: relative watch path joined to `current_dir()`; event path = watched dir + reported name | `notify-8.2.0/src/windows.rs:531-535`, `:547-551`; `:395-398`; handler thread `"notify-rs windows loop"` `:102-103` | n/a | basis of D2 |
| KS-22 no `deny_unknown_fields`, `schema.rs:18-86` | `src/config/schema.rs`: `RawConfig` `:18-24`, `RawBinding` `:26-63`, `SequenceSpec` `:79-84`, `ChordSpec` `:88-93`; struct variants `RawAction::PostMessage` `:136-142`, `RawAction::Launch` `:146-150`, `RawStep::Launch` `:193-197` | moved | defect present; `allow_extra_modifiers` (A2) at `:57-62` is also unchecked |
| KS-22 `#[serde(default)]` on `bindings` `schema.rs:22-23` | `src/config/schema.rs:20-23` | unchanged | |
| KS-22 empty list only warns `loader.rs:157-161` | `src/config/loader.rs:171-173` | moved | |
| KS-22 reload swaps any `Ok` `config_watcher.rs:100-105` | `src/hook/config_watcher.rs:100-110` | unchanged | |
| KS-22 README promise `README.md:122-124` | `README.md:133-135`, `:606` | moved | true only after C6/C8 |
| KS-23 missing file -> `write_default` `loader.rs:101-104` | `src/config/loader.rs:100-106` (`!path.exists()` `:101`, `write_default` `:102`) | moved | defect present, at startup and on reload |
| KS-23 watcher forwards `Remove` and renames `config_watcher.rs:51-54` | `src/hook/config_watcher.rs:51-54` | unchanged | |
| KS-41 `write_default` creates parent folders `loader.rs:194-207` | `src/config/loader.rs:221-234` | moved | an explicit typo path creates folders and a file |
| KS-24 `fs::read_to_string` keeps U+FEFF `loader.rs:109` | `src/config/loader.rs:108-111`; RON parse `:152-155` | moved | defect present |
| KS-27 terms copied unchecked `loader.rs:288` | `raw_trigger_to_trigger` `src/config/loader.rs:365-372`, called `:245` | moved | defect present |
| KS-27 `max_gap_ms` `loader.rs:307` | `src/config/loader.rs:324` | moved | |
| KS-27 `window_ms` `loader.rs:324-331` | `src/config/loader.rs:348` | moved | |
| KS-27 effects `chord_state.rs:83`, `sequence_state.rs:71` | gone; now `src/engine/chords.rs:112`, `src/engine/sequences.rs:82`, `:113`, gesture deadlines `src/engine/mod.rs:420`, `:432`, `:446` | gone (A1 rewrite) | engine untouched by this lane |
| errors carry `binding #<index> [<keys>]` | `src/config/loader.rs:236-240` (0-based `index`) | unchanged | used by D6 |
| default template test | `src/config/loader.rs:506-513` `default_config_parses` | unchanged | acceptance |

Result: KS-21, KS-22, KS-23, KS-24 and KS-27 are all present on `4177540`. Nothing is already fixed.

### Documentation drift (`book/src/gaps.md:154-202`, plus the brief's list)

| Drift row (book @7cb3521) | Now | Status | Action |
|---|---|---|---|
| template "Reload requires restarting" `default_config.ron:3` | `src/config/default_config.ron:3` | unchanged | C1 |
| template examples `["g","g"]`, `["j","k"]` `:17-21` (KS-15) | `:11` ("j+k together"), `:17-21` | unchanged | C1: F13/F14 |
| template: five step variants `:87-98` | `:102-113` | moved | C1: add `Launch`, `OpenUrl`, `Media` |
| template `Launch` "(detached)" `:57` | `:72` | moved | C1 |
| template Tap/Hold "resolves whichever wins" `:78-81` | `:93-96` | moved | C1 (equal terms exclusive since KS-25 part; Hold sends one press) |
| template key names `:32-41` | `:32-44` | moved | C1: aliases and media keys |
| README "Five step variants" `:321` | `README.md:398` | moved | C1 |
| README module tree `:19-55` (no `condition.rs`, `trigger.rs`; stray line `:27`) | `README.md:19-64`; stray `└──` `:28`; `data/` `:39-44` lacks both files; `engine/` `:45-54` lacks `properties.rs` | moved | C1 |
| README Modules: six `BindAction` variants, `Binding` = combo + two flags `:66-84` | `README.md:78-85`, `:87-94` | moved | C1 |
| README "Hot-path allocations: none" `:497`, "lock-free hot path" `:481` | gone; `README.md:595`, `:605`, `:611` were rewritten by Workstream A and are accurate | gone | none |
| README seven `windows` features `:465-472` | `README.md:579-586`; `Cargo.toml:41-50` has eight (`Win32_UI_Shell` unlisted; `Win32_System_ProcessStatus` removed by KS-03) | moved | C1 adds `Win32_UI_Shell`; C4 adds `Win32_Globalization` |
| README "Not macros" `:487` | `README.md:601` | moved | C1 |
| README "more specific checked first" `:493` (KS-13) | `README.md:607`; KS-13 still open (`src/engine/mod.rs:371-407`, only Immediate is exclusive) | moved | C1 |
| README Tap/Hold "resolve independently", "holding sends LShift" `:252-254` | `README.md:304-306`, `:309` | moved | C1 |
| README `Launch` "detached" `:194` | `README.md:243` | moved | C1 |
| README "observe ... UAC consent UI" `:419` | `README.md:524` | moved | C1 |
| README "UAC prompt - accept once" `:428`, `:439` | `README.md:535` (`:546` already fixed by A5) | moved | C1 (`:535`) |
| README key tables `:171-179` | `README.md:182-190` (`LeftShift` now listed; `Caps`, `Spacebar`, `Back`, `UpArrow`.., `Scroll`, `Break`, `PrtSc`/`PrtScn`, media keys missing; `src/data/vk_name.rs:130-171`) | moved | C1 |
| README `swch` table aliases `:435-444`; usage omits `help` `swch.rs:59-68` | `README.md:542-551`; aliases `src/bin/swch.rs:34-47`; usage `:57-66` | moved | C1 |
| README malformed save never bricks `:122-124`, `:492` | `README.md:133-135`, `:606` | moved | C6, C8, C10, C12 |
| README/template LShift DoubleTap never fires | `README.md:317-320`, template `:226-228` | fixed by A2 (KS-07) | none |
| README `RWin` / sided names `:173`, `:181-183` | `README.md:184`, `:192-199` | fixed by A2 | none |
| README deferred trigger repeats `:289-291` | `README.md:352-370` | fixed by A3 (KS-06) | none |
| README/template/condition.rs case-insensitive `:361` | `README.md:439-440`, template `:141` | fixed by A6 (KS-28) | none |
| README `swch` as normal user `:422-423` | `README.md:527-530` | fixed by A5 (KS-02) | none |
| README on/off idempotent `:440-441` | `README.md:547-548` | fixed by A5 (KS-01) | none |
| `config/mod.rs:1` "config.toml"; `parsing.rs:39` "in TOML" | `src/config/mod.rs:1`; `src/config/parsing.rs:56` | unchanged / moved | C1 |
| `build.rs:5-10` "default manifest keeps compatibility shims"; `app.manifest` unused | `build.rs:5-10` | unchanged | C1 fixes the comment; `app.manifest` stays (roadmap "Project infrastructure", version row, `roadmap.md:494`) |
| `condition.rs:133-135` LIMITED enough for GetModuleBaseNameW | gone (A6) | gone | none |
| `windows_actions.rs:35-36` Launch "Detached ... no console" | `src/core/windows_actions.rs:35-36` | unchanged | C1 |
| `loader.rs:169-171` auto-blocker dedup comment | `src/config/loader.rs:178-186` accurate (KS-18) | fixed by A4 | none |
| `sequence_state.rs:91-94`, `tap_state.rs:109-110` | files gone (A1) | gone | none |
| `trigger.rs:34-35` `is_immediate` "used by the hot path" | `src/data/trigger.rs:34-35`; callers `src/data/binding.rs:163`, `src/config/loader.rs:250` | unchanged | C1 |
| `#[allow(dead_code)]` "unused builder API" `binding.rs:111-132`, `windows_actions.rs:20` | `src/data/binding.rs:121`, `:132`, `:138`; `src/core/windows_actions.rs:16` | moved | C1 removes them |
| `.claude/agents/key-switch-*.md` stale rules | gone (nine roles in `.claude/agents/`) | gone | none |
| roadmap B: mark `LShift`+`DoubleTap` as not working until 1.3 | `README.md:317-320`, template `:226-228` | done by A2 | none |

## Goal and non-goals

**After the lane**

- The config path from the command line is made absolute once, on the main thread, before the startup load and the watcher
  (KS-21). The watcher reloads on any create, modify, rename or remove whose file name equals the config's name, compared
  with the OS uppercase table (`CompareStringOrdinal`, ignore case).
- The default template is written only at startup, only at `<exe_dir>\config.ron`, only if that file is missing (KS-23). A
  missing command-line path is a fatal error and creates nothing (KS-41, partly). A file missing at reload keeps the live
  bindings and logs the existing `[ERROR] config reload failed:` / `[INFO] keeping previous bindings live` lines.
- Unknown fields are load errors in `RawConfig`, `RawBinding`, `SequenceSpec`, `ChordSpec` and in the struct variants of
  `RawAction` and `RawStep`; `bindings` is required; `bindings: []` still loads with the existing warning (KS-22).
- A leading U+FEFF is ignored (KS-24).
- `Tap`/`Hold`/`DoubleTap` terms, `max_gap_ms` and `window_ms` must be 1..=10000 ms; the error names the binding (KS-27).
- README, template and code comments no longer contradict the code (rows marked C1 above).
- Real-`notify` tests of the watcher exist (also roadmap 4.1's "A `config_watcher` test exists", `roadmap.md:408`).

**Non-goals**

| Left out | Owner |
|---|---|
| Reset pending gestures on reload (KS-31 reload part) | roadmap 2.3 (`roadmap.md:297`, `:304`) |
| Reload errors in a log file; smoke item 8's "error to the log" | roadmap 1.7 (`roadmap.md:215-223`, `:236`) |
| Argument parsing, `-`-prefixed args, last-good fallback (rest of KS-41) | roadmap 4.2 (`roadmap.md:412`) |
| `swch reload` / `swch check` | roadmap 4.3 (reuses `config::reload`, which never writes) |
| `%APPDATA%` config location, Program Files ACLs | roadmap 4.4 |
| Embed or delete `app.manifest` | roadmap "Project infrastructure" (`roadmap.md:494`) |
| Internal serde names (`RawConfig`, `RawBinding`) in RON messages | follow-up, Q4 |
| 8.3 short names in change notifications; buffer-overflow rescan events; transient read error retry (known-issues row 5); UTF-16 files | follow-ups, no KS id (Risks RK3, RK4, RK9) |
| `Delay(ms)` range | not in Workstream B |
| Template `Ctrl+Pause` example (known-issues row 51), README "Equivalent to the auto-generated default" comment (`README.md:150`) | follow-up, Q6 (not in gaps.md drift) |

**Later items that depend on this lane:** all of Phase 1 (`roadmap.md:594`, `B --> P1`); 1.1 (`repeat` field must be added to
`RawBinding`, which now rejects unknown fields); 1.7; 4.2; 4.3; 4.4 (adds a third `ConfigPath` source); the phase book sync.

## Decisions

### D1: `ConfigPath` resolves the path once, in the library, called from `main`

- **What.** In `src/config/loader.rs`, re-exported by `src/config/mod.rs`:

  ```rust
  /// Where the config file is, decided once at startup.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum ConfigPath {
      /// The first command-line argument, made absolute against the current directory.
      CommandLine(PathBuf),
      /// `<exe_dir>\config.ron`.
      BesideExe(PathBuf),
  }
  impl ConfigPath {
      pub fn from_arg(arg: Option<OsString>) -> Result<Self, ConfigError>;
      pub fn path(&self) -> &Path;
      pub fn into_path(self) -> PathBuf;
  }
  ```

  `from_arg(Some(a))` = `std::path::absolute(&a)` -> `CommandLine`; an error (empty argument, no current directory) is
  `ConfigError::Other(format!("config path {a:?} cannot be used: {e}"))`. `from_arg(None)` = `BesideExe(default_config_path()?)`.
  `main` becomes: `let config = config::ConfigPath::from_arg(std::env::args_os().nth(1))?;`
  `let bindings = config::load(&config)?;` `App::new().with_config_watcher(config.into_path())`.
  `default_config_path` leaves the `pub use` (`src/config/mod.rs:15`); it stays a private-module `pub fn` used by `from_arg`.
- **Why.** Learn, GetFullPathNameW: "If a relative path is received, it should be consumed exactly once, ... by converting it
  to an absolute path and using the absolute path from that point forward." The daemon binary has `test = false`
  (`Cargo.toml:30-34`), so logic inside `main.rs` cannot be tested; one library call from `main` is the "once in main" the
  spec asks for. The enum carries what KS-23 needs (explicit or default) without a bool.
- **Rejected.** Inline `absolute` in `main.rs`: untestable. `fs::canonicalize`: needs the file to exist, returns `\\?\` verbatim
  paths and resolves symlinks/junctions, so the watched folder could differ from the one the user named. Absolutizing inside
  `spawn_watcher` only: the startup load and the watcher would see different paths.
- **Cost.** One `GetFullPathNameW` at startup.

### D2: The watcher matches the file name with `CompareStringOrdinal(.., bIgnoreCase = TRUE)`

- **What.** Private in `src/hook/config_watcher.rs`:

  ```rust
  /// The config file as the watcher sees it.
  struct WatchTarget { dir: PathBuf, name: Vec<u16> }   // C2: { path: PathBuf }
  impl WatchTarget {
      /// `None` when `config_path` has no parent folder or no file name (`C:\`).
      fn new(config_path: &Path) -> Option<Self>;
      /// The folder to watch, non-recursively.
      fn dir(&self) -> &Path;
      /// A create, modify (data, metadata, rename) or remove of an entry whose file name is the config's.
      fn wants_reload(&self, event: &Event) -> bool;
  }
  /// Whether two file names are equal for the file system: ordinal, case folded with the OS uppercase table.
  fn same_file_name(a: &[u16], b: &[u16]) -> bool;
  ```

  `wants_reload` keeps today's kind set (`Modify(_) | Create(_) | Remove(_)`, `config_watcher.rs:51-54`) and compares
  `p.file_name()` of each event path (`OsStrExt::encode_wide`) with `name`. `same_file_name` returns `false` if either slice is
  empty, else `CompareStringOrdinal(a, b, true) == CSTR_EQUAL`. `spawn_watcher` builds the target first and maps `None` to
  `notify::Error::generic("the config path has no parent folder or file name").add_path(config_path.clone())`
  (`notify-8.2.0/src/error.rs:55`, `:75`); `App::run` already turns a watcher error into `[WARN]` + "continuing without
  hot-reload" (`src/core/app.rs:167-174`).
- **Why.**
  - The watch is non-recursive on the parent (`config_watcher.rs:65-69`); Learn, ReadDirectoryChangesW: with
    `bWatchSubtree` FALSE "the function monitors only the directory specified by the hDirectory parameter", and
    FILE_NOTIFY_INFORMATION's `FileName` is "the file name relative to the directory handle". So the file name alone
    identifies the entry "within the watched directory" (spec), and the folder part of the path (8.3 temp folders on CI
    runners, `..`, letter case) cannot cause a miss.
  - Learn, Handling Sorting: CompareStringOrdinal is for "non-linguistic strings ... NTFS file names", and with bIgnoreCase
    TRUE "the function maps case using the operating system uppercasing table". That is the file system's notion of "same
    name", including non-ASCII names.
- **Rejected.** `OsStr::eq_ignore_ascii_case`: ASCII only (the KS-28 class of bug). `str::to_lowercase` on both sides: not the
  OS table (e.g. U+1E9E vs U+00DF), and lossy for unpaired surrogates. Watching the file itself: notify emulates it with an
  exact `PathBuf` compare (`notify-8.2.0/src/windows.rs:400-405`), same case problem, and temp+rename saves break it. Only
  absolutizing (D1) without the name compare: fixes the relative path but not `MY.RON` vs `my.ron` (roadmap B bullet,
  known-issues row 4).
- **Cost.** One `Vec<u16>` per event path and one kernel32 call, on the notify thread. `Cargo.toml` gains the feature
  `Win32_Globalization` (`windows-0.62.2/Cargo.toml:418`: `["Win32"]`, no new crate, `Cargo.lock` unchanged).

### D3: The template is written only by the startup load of `BesideExe`; a missing file is `ConfigError::Missing`

- **What.**
  - New variant `ConfigError::Missing { path: PathBuf }`, Display `config file {} does not exist` (path `.display()`),
    `source()` -> `None` (`loader.rs:74-82` falls to `_ => None`; keep that arm).
  - `fn read_config(path: &Path) -> Result<String, ConfigError>`: `fs::read_to_string`; `ErrorKind::NotFound` -> `Missing`;
    other errors -> `Io` as today. No `exists()` pre-check (TOCTOU).
  - `fn parse_config(path: &Path, text: &str) -> Result<Vec<Binding>, ConfigError>`: today's `from_ron_str(..).map_err(..)`
    that attaches the path to `Parse` (`loader.rs:113-120`).
  - `pub fn load(config: &ConfigPath)`: `CommandLine` -> `read_config`, then print `[INFO] loading config from <path>`, then
    `parse_config`; `Missing` goes back to `main`, which prints it as `[FATAL]`. `BesideExe` -> the same, except that on
    `Missing` it calls `write_default`, prints `[INFO] generated default config at <path>` (and no "loading" line, as today),
    then `read_config` + `parse_config`.
  - `pub fn reload(path: &Path)`: `read_config`, print `[INFO] loading config from <path>` (as today's reload does),
    `parse_config`. Never writes.
  - `write_default` (`loader.rs:221-234`) keeps `create_dir_all(parent)` (only reached for the exe folder) and writes with
    `OpenOptions::new().write(true).create_new(true)`; `ErrorKind::AlreadyExists` -> `Ok(())` (someone created the file
    between the read and the write; read it as it is). Other write errors stay `Io` (fatal at startup, as today).
- **Why.** KS-23's suggested fix (`audit.md:525`) plus the spec's "missing EXPLICIT CLI path is an error". Mapping `NotFound`
  covers a missing parent folder too, so a typo path creates nothing (KS-41 scenario, `audit.md:694`). The existing reload
  `Err` branch (`config_watcher.rs:106-109`) already "keeps the current bindings and logs".
- **Rejected.** Dropping `Remove` events in the watcher: renames arrive as `Modify(Name(From))` (`windows.rs:425-430`) and a
  file can be gone when the debounce fires, so `load` would still regenerate. A separate `[WARN]` text for a deleted file:
  needs `ConfigError` exported to `hook` for one message; the Display text is already specific. `fs::write` after an
  `exists()` check: TOCTOU, may clobber a file created meanwhile.
- **Cost.** Behaviour change for users who relied on `key-switch-rs.exe new.ron` creating `new.ron`: CHANGELOG **Breaking**.

### D4: `#[serde(deny_unknown_fields)]` on six containers; `bindings` required; explicit empty list still loads

- **What.** `src/config/schema.rs`: add `#[serde(deny_unknown_fields)]` to `RawConfig`, `RawBinding`, `SequenceSpec`,
  `ChordSpec`, `RawAction`, `RawStep`. Remove `#[serde(default)]` from `RawConfig::bindings` (`:22`) and rewrite its doc
  (`:20-21`): required; a missing or misspelled `bindings` is a load error (KS-22); `bindings: []` loads with a warning.
  `RawTrigger`, `RawCondition`, `MediaKeyRef`, `RawWindowKind`, `MessageRef` have no named fields: untouched. The
  `[WARN] config contains no bindings` stays (`loader.rs:171-173`).
- **Why.** serde's container attribute is read through `cattrs.deny_unknown_fields()` in `serde_derive/src/de/struct_.rs`,
  which serves struct variants of an externally tagged enum as well, so it covers `Launch(exe, args)` and
  `PostMessage(msg, wparam, lparam)`; the book counts their silently dropped fields under KS-22
  (`book/src/config/errors.md:178`, `actions.md:31`). RON reports it as
  ``Unexpected field named `<f>` in `<Container>`, <expected list>`` (ron 0.10.1 `error.rs`, `NoSuchStructField` Display;
  `Identifier` prints `` `<f>` ``), so "the error names the field". With required `bindings`, `()` fails too. An explicit
  `bindings: []` is a deliberate choice (everything commented out), and rejecting it only on reload would make a file that
  loads at startup fail at reload.
- **Rejected.** Reject an empty list on reload only: asymmetric. Warn on unknown fields instead of failing: serde has no
  hook for it without a new crate (`serde_ignored`), which the roadmap does not name. Only the four structs the brief lists:
  leaves `Launch(cwd: ..)` silently ignored (Q7).
- **Cost.** **Breaking** for configs with typos or extra fields (roadmap "Planned changes", `roadmap.md:504`). RON stops at
  the first such error, so a file with two typos shows one at a time.

### D5: Strip one leading U+FEFF in `from_ron_str_with_hotkeys`

- **What.** First statement of `from_ron_str_with_hotkeys` (`loader.rs:141-144`):
  `let text = text.strip_prefix('\u{feff}').unwrap_or(text);` with a comment (KS-24: Notepad "UTF-8 with BOM", Windows
  PowerShell 5.1 `-Encoding UTF8`; ron 0.10.1 does not skip it).
- **Why.** One place for every text source (startup, reload, tests, a future `swch check`); pure tests reach it. Positions in
  RON errors then count from the first visible character, as editors show them.
- **Rejected.** Strip in `read_config` only: pure tests would not cover it. Remove every U+FEFF: changes string contents
  (`Text("..")` macro steps).
- **Cost.** None.

### D6: Range 1..=10000 ms in the loader, message with the binding number

- **What.** In `src/config/loader.rs`:

  ```rust
  /// Every timing value of a binding: the Tap, Hold and DoubleTap terms, `max_gap_ms` and `window_ms` (KS-27).
  const TIMING_RANGE_MS: RangeInclusive<u64> = 1..=10_000;
  /// `Ok(ms)` inside the range, else "`<written>` is out of range: timings are 1 to 10000 ms".
  fn check_timing_ms(written: &str, ms: u64) -> Result<u64, String>;
  fn raw_trigger_to_trigger(raw: RawTrigger) -> Result<Trigger, String>;   // was infallible
  ```

  `written` is the value as the config spells it: `Tap(0)`, `Hold(3600000)`, `DoubleTap(0)`, `max_gap_ms: 0`,
  `window_ms: 10001`; the message's bounds come from `TIMING_RANGE_MS.start()`/`.end()`. Call sites:
  `raw_binding_to_binding` maps the trigger error with `err_prefix` right after the action (`loader.rs:245`), before the
  deferred-trigger rule (`:250`); `sequence_spec_to_kind` checks `max_gap_ms` after the steps parse (`:317-321`);
  `chord_spec_to_kind` checks `window_ms` after the keys loop (`:333-345`). Full line, e.g.
  `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`.
- **Why.** The loader is where the binding number is known (`loader.rs:236-240`); the engine stays total and untouched.
  Numbers count from 0 like every existing loader message and the KS-18 INFO line (`loader.rs:210`) (Q3).
- **Rejected.** Clamping silently: hides the mistake. A custom serde deserializer: its error has no binding number. A check in
  `Trigger`/the engine: runtime types are built directly by tests with any value (`keyboard_hook_callback.rs:360`).
- **Cost.** **Breaking** for values outside the range. Defaults 500 and 50 (`schema.rs:95-101`) are inside.

### D7: Seams first, so every red test fails by assertion on its parent

- **What.** C2 introduces `ConfigPath`, `load(&ConfigPath)`, `reload(&Path)`, `WatchTarget` and
  `reload_into(&Path, &KeyboardHook)` with **today's** semantics; C3, C5, C7, C9, C11 then compile on their parents and fail
  on assertions. Precedent: A6 C1 (`D:/tmp/ks-orch/p0-A6-appequals/cut.md:233-238`).
- **Why.** A compile-error red proves nothing about the defect; the tester replays each red on its parent commit.
- **Rejected.** Red tests against new API: compile errors on `master`.

### D8: File-system tests in a per-test temp folder; real-`notify` tests run by default

- **What.** New `src/config/testing.rs` (`#[cfg(test)] pub(crate) mod testing;` in `src/config/mod.rs`):

  ```rust
  /// A new empty folder under `std::env::temp_dir()`, removed (best effort) on drop.
  pub(crate) struct TempDir(PathBuf);
  impl TempDir {
      /// `<temp>\key-switch-rs-test-<pid>-<n>-<label>`; `n` from a static `AtomicUsize`.
      pub(crate) fn new(label: &str) -> Self;
      pub(crate) fn path(&self) -> &Path;
  }
  impl Drop for TempDir { /* let _ = fs::remove_dir_all(..) */ }
  ```

  The counter uses `fetch_add(1, Ordering::Relaxed)`; pairing argument for the comment: only the uniqueness of the returned
  value matters, an RMW is atomic under any ordering, and no other data is published through it. Two tests (T7, T14) start a
  real watcher with `spawn_watcher` on a leaked `KeyboardHook::new()` (pattern of `src/core/app.rs:222-229`).
- **Why.** No `tempfile` crate (no new dependency). ReadDirectoryChangesW on a temp folder is not a hook, injects nothing and
  writes no registry or setting, so the safety rules do not require `#[ignore]` (Q5). They are the only tests that exercise
  notify's real event names, which is how KS-21 escaped.
- **Rejected.** Only synthetic `notify::Event`s: miss what notify really reports. `#[ignore]`: nothing unsafe to ignore for.
  Tests that change the current directory: process-global, breaks parallel tests.

### D9: Changelog classification

- **Changed, Breaking:** KS-22 (unknown fields, required `bindings`), KS-27 (range), the command-line path must exist (KS-23
  part, KS-41 partly). Precedent: A2 Breaking entries under Changed (`CHANGELOG.md:60-90`).
- **Fixed:** KS-21, KS-23 (reload keeps bindings), KS-24, documentation drift.

## Threads and Win32 API choices

| Code path | Thread | Notes |
|---|---|---|
| `ConfigPath::from_arg` (`GetFullPathNameW` via `std::path::absolute`; `GetModuleFileNameW` via `current_exe`) | main, in `run()` before `App::run`; no other thread exists yet | |
| `config::load` (read, maybe `write_default`, parse, registry read for auto-blockers) | main, before `hook.install()` (`app.rs:131`) | unchanged thread |
| `WatchTarget::wants_reload`, `same_file_name` (`CompareStringOrdinal`) | notify backend thread `notify-rs windows loop` (`notify-8.2.0/src/windows.rs:102-103`), under notify's handler mutex (`:416-421`) | no lock of ours, no hook state |
| `reload_into` -> `config::reload` -> `KeyboardHook::update_bindings` | `config-watcher` (`config_watcher.rs:71-74`) | `update_bindings` sorts and does one `ArcSwap` store (`keyboard_hook.rs:110-115`); no store on error |
| LL keyboard callback, WinEvent callback, `with_engine`, Hold timers | unchanged | no code on these paths changes |

No hook, window, timer, message pump or COM apartment is added, moved or removed.

| Purpose | `windows` path / std | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Make the CLI path absolute | `std::path::absolute(&OsStr) -> io::Result<PathBuf>` (std 1.79; MSRV 1.88 `Cargo.toml:8`) | n/a | main | "If `path` is syntactically invalid; in particular, if it is empty"; "If getting the current directory fails" -> `ConfigError::Other` | <https://doc.rust-lang.org/std/path/fn.absolute.html> ("On Windows ... currently equivalent to calling GetFullPathNameW"); <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew> |
| Compare file names | `windows::Win32::Globalization::{CompareStringOrdinal, CSTR_EQUAL}`; `pub unsafe fn CompareStringOrdinal(lpstring1: &[u16], lpstring2: &[u16], bignorecase: bool) -> COMPARESTRING_RESULT` (`windows-0.62.2/src/Windows/Win32/Globalization/mod.rs:20-23`; `CSTR_EQUAL` `:7242`) | `Win32_Globalization`: **absent** (`Cargo.toml:41-50`); add in C4 | notify backend | returns 0 on `ERROR_INVALID_PARAMETER`: anything but `CSTR_EQUAL` is "different" (no reload). The wrapper converts `len()` to `i32` with `unwrap`: file-name components are far below `i32::MAX`; empty slices are filtered before the call | <https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-comparestringordinal>; <https://learn.microsoft.com/en-us/windows/win32/intl/handling-sorting-in-your-applications> |
| Directory change events | `notify` 8.2 `ReadDirectoryChangesWatcher` (existing) | n/a | notify backend | existing | <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw>; <https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-file_notify_information> |

`unsafe`: exactly one block, in `same_file_name`, with `// SAFETY: CompareStringOrdinal reads a.len() and b.len() UTF-16 units
from the two slices, which are borrowed for the whole call; it writes nothing and keeps no pointer.` Import grouped:
`use windows::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};`.

**Injection, flags, key state:** no `SendInput`, no key handling; `LLKHF_INJECTED`, extended flag, `scanCode`, sided keys and
auto-repeat are not involved.

**Elevation and UIPI:** the elevated release daemon still writes the template only in its own folder (unchanged); a
command-line path is now only read. No pipe, child process or token change.

## Touch set

| File | Change | Commit | Why |
|---|---|---|---|
| `src/main.rs` | use `ConfigPath::from_arg`, `config::load(&config)`, `with_config_watcher(config.into_path())`; comment `:24-27` says the path is made absolute and must exist (C6) | C2, C6 | D1 |
| `src/config/loader.rs` | C1: test T0. C2: `ConfigPath`, `load(&ConfigPath)`, `reload(&Path)` on old semantics. C3/C5/C7/C9/C11: tests. C4: `absolute` in `from_arg`. C6: `ConfigError::Missing`, `read_config`, `parse_config`, startup-only template, `create_new`. C10: BOM strip. C12: `TIMING_RANGE_MS`, `check_timing_ms`, fallible `raw_trigger_to_trigger` | all but C8 | D1, D3, D5, D6 |
| `src/config/schema.rs` | `deny_unknown_fields` x6; `bindings` required; doc `:20-21` | C8 | D4 |
| `src/config/mod.rs` | C1: doc `:1-2` "config.ron". C2: `pub use loader::{ConfigPath, load, reload};` (drop `default_config_path`). C3: `#[cfg(test)] pub(crate) mod testing;` | C1, C2, C3 | D1, D8 |
| `src/config/testing.rs` (new) | `TempDir` | C3 | D8 |
| `src/config/parsing.rs` | doc `:54-56` "numeric literal in the config" | C1 | drift |
| `src/config/default_config.ron` | C1 drift (header, examples, steps, Launch, Tap/Hold, key names); C6/C8/C10/C12 one header line each | C1, C6, C8, C10, C12 | config meaning |
| `src/hook/config_watcher.rs` | C2: `WatchTarget` (old compare), `reload_into`, no-parent error. C3/C5/C7: tests. C4: name compare, `same_file_name`, T8. Module doc `:1-12` gains the name rule (C4) and "a missing file keeps the bindings" (C6) | C2-C7 | D2, D3 |
| `Cargo.toml` | `"Win32_Globalization",` after `"Win32_Foundation",` (`:42`) | C4 | D2 |
| `src/data/trigger.rs` | doc `:34-35` | C1 | drift |
| `src/data/binding.rs` | remove `#[allow(dead_code)]` `:121`, `:132`, `:138` | C1 | drift |
| `src/core/windows_actions.rs` | remove `#[allow(dead_code)]` `:16`; doc `:35-36` | C1 | drift |
| `src/bin/swch.rs` | usage gains `help` (`:57-66`) | C1 | drift |
| `build.rs` | comment `:5-10` | C1 | drift |
| `README.md` | C1 drift; C3 tree line; C4, C6, C8, C10, C12 meaning | C1, C3, C4, C6, C8, C10, C12 | rules |
| `CHANGELOG.md` | entries per commit (texts below) | C1, C4, C6, C8, C10, C12 | rules |
| not touched | `src/engine/**`, `src/hook/keyboard_hook*.rs`, `src/hook/ipc_server.rs`, `src/ipc.rs`, `src/core/app.rs`, `Cargo.lock`, `app.manifest`, `CLAUDE.md` (Q2), `book/` (book sync, Q1) | | |

## Commits, in order

Rules:

- Gate on every commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
- **Declared red commits: C3, C5, C7, C9, C11.** fmt and clippy pass on them; exactly the named tests fail, with the texts
  below (panic messages are the `expect`/`assert` texts the cut gives). Any other red test: stop and report.
- Replay: each red commit's tests compile on its parent (C2, C4, C6, C8, C10). On `master` the C3/C5 tests do not compile
  (`ConfigPath`, `WatchTarget`, `reload_into`, `config::reload` missing); replay them on C2/C4.
- Line numbers in this section are those of `master` (`4177540`); earlier commits of the lane shift them, so locate each
  edit by the quoted text.
- Every message: subject `<Area>: <what>`, red commits quote the failing lines in the body, blank line, then
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Source stays ASCII: non-ASCII test text as `\u{..}` escapes, BOM bytes as `b"\xEF\xBB\xBF"`; new template lines in ASCII.
- New tests avoid `block_original_combo` (it reads the registry) except T0 (hotkeys stubbed) and T12 (loads the template,
  like `default_config_parses`; read only).
- Common test configs: `ONE` = `( bindings: [ ( keys: "F13", action: DoNothing ) ] )`,
  `TWO` = `( bindings: [ ( keys: "F13", action: DoNothing ), ( keys: "F14", action: DoNothing ) ] )`.

### C1. `Docs: fix documentation drift in the README, the template and code comments` (green)

Template `src/config/default_config.ron`:

- `:3` -> "Changes are applied when you save: the daemon reloads this file about 250 ms after the save, without a restart.
  If the new version does not load, the error is printed and the previous bindings stay active." (three comment lines).
- `:11` "(j+k together)" -> "(F13+F14 together)". `:17-21`: `steps: ["F13", "F14"]` "// press F13, then F14";
  `chord: (keys: ["F13", "F14"], window_ms: 50)` "// press F13 and F14 together".
- `:40-43` Named: add `Caps`, `Spacebar`, `Back`, `UpArrow`/`DownArrow`/`LeftArrow`/`RightArrow`, `Scroll`, `Break`,
  `PrtSc`/`PrtScn`; new line `Media : MediaPlayPause (PlayPause), MediaStop, MediaNext (NextTrack), MediaPrev (MediaPrevious,
  PrevTrack), VolumeUp (VolUp), VolumeDown (VolDown), VolumeMute (VolMute, Mute)` (`src/data/vk_name.rs:142-170`).
- `:72` Launch: "- spawn a process; until roadmap item 3.3 it gets the daemon's administrator rights and console".
- `:93-96`: both gestures arm on key-down; with equal terms exactly one fires (Tap if released before 200 ms, Hold otherwise);
  Hold sends one LShift press at that moment, not a held Shift.
- `:102-113`: add `Launch(exe: "...", args: [...])`, `OpenUrl("...")`, `Media(<key>)` "(same as the action)".

README:

- Tree `:19-64`: delete the stray `    └──` line `:28`; add `condition.rs` (`when:` conditions and the foreground app and title)
  and `trigger.rs` (`Immediate`, `Tap`, `Hold`, `DoubleTap`) under `data/`; `properties.rs` (test-only property tests of the
  suppression invariant) under `engine/`.
- Modules `:78-85`: add `Sequence(Arc<Vec<SequenceStep>>)`, `Launch { exe, args }`, `OpenUrl(String)`, `Media(MediaKey)`.
  `:87-94`: `kind` (`BindingKind`: `Combo` / `Sequence` / `Chord`), `action`, `block_default`, `block_original_combo`,
  `condition`, `trigger`, `allow_extra_modifiers`, crate-private `is_auto_blocker`.
- Combo table `:189` Named row: the aliases above; new row `Media` with the names above.
- Actions table `:243`: "Spawn a process. `args` defaults to `[]`. Until roadmap item 3.3 the program gets the daemon's
  administrator rights (release build) and, for a console program, its console (KS-09, KS-34)."
- Triggers `:304-306`: "Tap and Hold both arm on key-down; with equal terms exactly one of them fires (see below)." `:309`
  comment: "holding sends one LShift press".
- `:398` "Eight step variants:".
- Run `:524`: drop "UAC consent UI" (Secure Desktop; `:623-628` already says no process can hook it). `:535`: "(UAC prompt on
  every launch)".
- `swch` table `:544-551`: add an aliases note: `open` = `start` = `launch`; `exit` = `quit` = `shutdown` = `stop`;
  `status` = `ping`; `help` = `--help` = `-h`.
- Technologies `:579-586`: add `Win32_UI_Shell` (`ShellExecuteW` for `OpenUrl` and `swch open`).
- `:601` "Native APIs first: language switching, CapsLock, window operations and messages use Win32 calls; key output
  (`PressKey`, `Media`, macro `Text`/`Key`/`Combo`) is synthesized with `SendInput`."
- `:607` "Priority bindings: among the `Immediate` bindings that match one press, the one with more keys fires; a matching
  deferred binding still arms until roadmap item 1.6 (KS-13)."

Code comments: `config/mod.rs:1` "`config.ron` loading"; `parsing.rs:56` "as a numeric literal in the config";
`trigger.rs:34-35` "True for the default trigger. The loader uses it to reject deferred triggers on `sequence` and `chord`,
and `Binding`'s `Display` to omit it."; `windows_actions.rs:35-36` "Spawn a process with `Command::spawn`. The child outlives
the daemon but inherits its console and, in a release build, its administrator token (KS-09, KS-34; roadmap item 3.3).";
remove `#[allow(dead_code)]` at `binding.rs:121`, `:132`, `:138` and `windows_actions.rs:16` (all are `pub` items of `pub`
modules, `src/data/mod.rs:1`, `src/core/mod.rs:3`, so no lint fires; if clippy reports `dead_code` for one, keep that
attribute and replace its comment with the true reason); `build.rs:5-10`: "We set only the UAC fragment, through the MSVC
linker's `/MANIFESTUAC`; the linker generates the rest of the manifest. The audit's probe build of this setup embedded only
the `trustInfo` block, so there is no `supportedOS` section. `app.manifest` in the repository root is not passed to the linker
(no `/MANIFESTINPUT`); the roadmap's version item decides whether to embed or delete it." `swch.rs:57-66`: add
`eprintln!("  help    Print this text");`.

Test T0 `config::loader::tests::default_config_commented_examples_load` (green): take `DEFAULT_CONFIG`; the lines after the
one containing `// ---- examples (uncomment to enable) ----` up to (excluding) the first line whose `trim()` is `],`; for each,
`trim_start()`, then strip a leading `// ` or turn a bare `//` into an empty line; join; wrap in `( bindings: [ ... ] )`;
`from_ron_str_with_hotkeys` with a hotkey stub that returns `None` must be `Ok` and non-empty. Guards the examples against
drift and, after C8, against unknown fields.

CHANGELOG `### Fixed`, last entry:

```markdown
- Documentation drift (roadmap Workstream B): the generated `config.ron` no
  longer says that changes need a restart; its `sequence` and `chord`
  examples use F13 and F14 instead of `g`, `j` and `k`, which such bindings
  swallow while you type (KS-15); it lists all eight macro steps, as the
  README now does (it said five). The README lists the key names it left
  out (`Caps`, `Break`, `PrtSc`, `Scroll`, the media keys and more), the
  `swch` command aliases, every `BindAction` variant and `Binding` field,
  and the `windows` features in use. `Launch` is no longer called
  detached: until roadmap item 3.3 the program it starts gets the daemon's
  administrator rights and console (KS-09, KS-34). The README no longer
  says that the elevated daemon sees keys in the UAC consent prompt, which
  runs on the Secure Desktop, or that the UAC prompt appears only once.
  `swch help` is listed in the usage text.
```

### C2. `Config: seams for the config path, the watcher filter and the reload step` (green, behaviour-preserving)

- `ConfigPath` per D1, but `from_arg(Some(a))` = `CommandLine(PathBuf::from(a))` (no `absolute` yet).
- Today's body of `load` (`loader.rs:100-121`: generate when missing) becomes a private `fn load_or_generate(path: &Path)`;
  `pub fn load(config: &ConfigPath)` calls it with `config.path()`, `pub fn reload(path: &Path)` with `path`. `reload`'s doc
  states that, for now, it also writes the template (KS-23); C6 changes that. Re-exports per touch set; `main.rs` per D1.
- `WatchTarget { path: PathBuf }`: `new` = `Some` iff `config_path.parent()` is `Some`; `dir()` = the parent;
  `wants_reload` = today's filter (`event.paths.iter().any(|p| p == &self.path)` and the kind set). `spawn_watcher` builds it
  first (no-parent error per D2, replacing the `expect` at `:65-68`) and moves it into the closure after copying `dir()`.
- `fn reload_into(config_path: &Path, hook: &KeyboardHook)`: today's match at `:100-110`, calling `config::reload`;
  `debounce_loop` calls it.
- No test changes; no CHANGELOG (the replaced `expect` was unreachable: `load` fails first for a path without a file name).

### C3. `Tests: red-first: relative and differently cased config paths (KS-21)` (RED)

Adds `src/config/testing.rs` (D8), the `mod` line, and the README tree line `testing.rs - test-only temporary folders for
the loader and watcher tests` under `config/`.

| Test | Script | PREDICTED RED on C2 |
|---|---|---|
| T1 `config::loader::tests::a_relative_config_argument_is_made_absolute` | `cwd = std::env::current_dir()`. For `"my.ron"` and `r".\sub\..\my.ron"`: `let config = ConfigPath::from_arg(Some(arg.into())).expect("a relative path is valid");` `assert!(matches!(config, ConfigPath::CommandLine(_)))`; `assert_eq!(config.path(), cwd.join("my.ron"), "{arg}")` | `assertion `left == right` failed: my.ron` with `left: "my.ron"`, `right: "<cwd>\\my.ron"` |
| T2 `config::loader::tests::an_empty_config_argument_is_an_error` | `ConfigPath::from_arg(Some(OsString::new())).expect_err("an empty config path must be an error")` | panic `an empty config path must be an error: CommandLine("")` |
| T3 `hook::config_watcher::tests::an_event_for_a_relative_config_path_matches` | `let target = WatchTarget::new(Path::new("my.ron")).expect("has a parent");` event `Event::new(EventKind::Modify(ModifyKind::Any)).add_path(cwd.join("my.ron"))` (what notify reports, `windows.rs:395-398`); `assert!(target.wants_reload(&event), "a Modify of <cwd>\\my.ron must reload the config given as my.ron (KS-21)")` | panic with that text |
| T4 `hook::config_watcher::tests::an_event_matches_the_config_file_name_in_any_case` | target `WatchTarget::new(&cwd.join("my.ron"))`; events `Modify(Any)` on `cwd.join("MY.RON")`, `Create(Any)` on `cwd.join("My.Ron")`, `Remove(Any)` on `cwd.join("my.ron")`, `Modify(Name(RenameMode::To))` on `cwd.join("MY.RON")`: each `assert!(target.wants_reload(&e), "{e:?}")` | first assert, the `MY.RON` Modify |
| T5 `hook::config_watcher::tests::other_files_and_other_event_kinds_do_not_match` | target as T4; `Modify(Any)` on `my.ron.tmp`, `Create(Any)` on `other.ron`, `Access(AccessKind::Read)` on `my.ron`, `EventKind::Any` on `my.ron`: all false | green (stays green) |
| T6 `hook::config_watcher::tests::a_root_path_has_no_watch_target` | `assert!(WatchTarget::new(Path::new(r"C:\")).is_none())` | green |
| T7 `hook::config_watcher::tests::a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file` | `TempDir::new("case")`; write `ONE` to `dir/config.ron` (the name on disk); leaked hook loaded with `config::reload(&dir/config.ron)`; `let _watcher = spawn_watcher(dir.join("CONFIG.RON"), hook).expect("watcher starts");` (declare after `dir`, so it drops first); loop up to 10 s: write `TWO` to `dir/config.ron` once per second, poll every 50 ms; `assert!(reloaded, "no reload within 10 s: the watcher of CONFIG.RON never swapped in the 2 bindings of config.ron")` | panic with that text after ~10 s |

### C4. `Config: make the config path absolute and match watcher events by file name (KS-21)` (green; T1-T7 green)

- `from_arg` per D1 (`std::path::absolute`). `WatchTarget { dir, name }` and `same_file_name` per D2; `new` returns `None` when
  `file_name()` or `parent()` is `None`. `Cargo.toml` feature. Module doc of `config_watcher.rs` states the name rule and why
  the folder part is not compared.
- T8 `hook::config_watcher::tests::same_file_name_ignores_case_like_the_file_system` (green on arrival): equal for
  (`config.ron`, `config.ron`), (`config.ron`, `CONFIG.RON`), (`\u{43a}\u{43e}\u{43d}\u{444}\u{438}\u{433}.ron`,
  `\u{41a}\u{41e}\u{41d}\u{424}\u{418}\u{413}.RON`); different for (`config.ron`, `config.ron.bak`), (`config.ron`,
  `konfig.ron`), (`config.ron`, empty). Encode with `str::encode_utf16`.
- README `:141-142` item 1: "A relative path is resolved once at startup against the daemon's current folder (for a Scheduled
  Task, its "Start in" folder); the console shows the full path." After item 3: "Hot reload watches the file's folder and
  reacts to the file's name in any letter case." Technologies: add `Win32_Globalization` (`CompareStringOrdinal`: the config
  watcher compares file names the way the file system does).
- CHANGELOG `### Fixed`, after KS-18 (`:216-229`), before KS-28 (`:230`):

  ```markdown
  - KS-21: hot reload works when the config path given on the command line
    is relative (`key-switch-rs.exe my.ron`, or a Scheduled Task with a
    relative argument and a "Start in" folder) or differs in letter case
    from the file on disk (`MY.RON` for `my.ron`). The daemon makes the path
    absolute once at startup, so the console shows the full path, and the
    watcher compares file names the way Windows does, ignoring case. It used
    to print `Watching my.ron for changes` and then never reload.
  ```

### C5. `Tests: red-first: a missing config file is never created except beside the exe (KS-23)` (RED)

| Test | Script | PREDICTED RED on C4 |
|---|---|---|
| T9 `config::loader::tests::a_missing_command_line_config_is_an_error_and_is_not_created` | `path = TempDir::new("missing-cli").path().join("my.ron")`; `let err = load(&ConfigPath::CommandLine(path.clone())).expect_err("a missing command-line config must be an error");` `assert!(!path.exists(), "a missing command-line config must not be created");` `assert!(err.to_string().contains("does not exist"), "{err}")` | panic `a missing command-line config must be an error: [Binding { kind: Combo(..` (the template's bindings) |
| T10 `config::loader::tests::a_missing_folder_of_a_command_line_config_is_not_created` | `path = dir.join("nope").join("my.ron")`; `expect_err("a command-line config in a missing folder must be an error")`; `assert!(!dir.join("nope").exists())` | panic `a command-line config in a missing folder must be an error: [..` |
| T11 `config::loader::tests::reload_of_a_missing_config_is_an_error_and_writes_nothing` | `reload(&dir.join("config.ron")).expect_err("reloading a missing config must be an error")`; `!exists`; message contains `does not exist` | panic `reloading a missing config must be an error: [..` |
| T12 `config::loader::tests::a_missing_config_beside_the_exe_is_generated_and_loaded` | `load(&ConfigPath::BesideExe(dir.join("config.ron")))` is `Ok`, non-empty; `assert_eq!(fs::read_to_string(..), DEFAULT_CONFIG)` | green (characterization) |
| T13 `hook::config_watcher::tests::a_deleted_config_keeps_the_live_bindings` | write `ONE`; `let hook = KeyboardHook::new(); hook.update_bindings(config::reload(&path).expect("loads"));` `fs::remove_file(&path)`; `reload_into(&path, &hook)`; `assert!(!path.exists(), "a deleted config must not be recreated");` then `assert_eq!(hook.bindings().len(), 1)` and its combination is `KeySpec::Key(VK_F13)` | panic `a deleted config must not be recreated` |
| T14 `hook::config_watcher::tests::a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted` | write `ONE`; leaked hook loaded; `spawn_watcher(path.clone(), hook)`; first prove the watcher is live: write `TWO` once per second until 2 bindings are live (10 s cap, `assert!` "the watcher never reloaded"); then `fs::remove_file`, sleep 1500 ms (6 x the 250 ms debounce); `assert!(!path.exists(), "a deleted config must not be recreated")`; `assert_eq!(hook.bindings().len(), 2)` | panic `a deleted config must not be recreated` after ~2 s |

### C6. `Config: write the template only at startup beside the exe (KS-23)` (green; T9-T14 green)

- D3. `reload`'s doc: never writes; a missing file is `ConfigError::Missing`. `main.rs:24-27` comment: a command-line path
  must exist. `config_watcher.rs` module doc item 4: a missing file also keeps the bindings.
- Template, after the new `:3` lines: "Deleting or renaming this file keeps the current bindings. The daemon writes this
  template only when it starts and finds no config.ron next to key-switch-rs.exe; a config path given on the command line
  must exist."
- README "Where the file lives" `:139-145`: item 1 adds "The file must exist: a missing file stops the daemon with `config file
  <path> does not exist`, and nothing is created." Item 3 becomes "If `<exe_dir>\config.ron` is missing at startup, the
  default template is written there and used." Add: "A file deleted or renamed while the daemon runs is not recreated: the
  bindings loaded last stay live, and putting the file back reloads it." `:133-135`: "If the file does not load (a syntax
  error, an unknown key name, ...) or is deleted or renamed, the error is printed and the **previously-running bindings stay
  live** ...". Errors `:477-485`: add `config file <path> does not exist`.
- CHANGELOG `### Changed`, after the KS-18 Breaking entry (`:82-90`):

  ```markdown
  - **Breaking:** a config path given on the command line must exist. With
    a missing file the daemon stops with `[FATAL] config file <path> does
    not exist` instead of writing the default template there (creating its
    folders) and starting with it; `key-switch-rs.exe --help` no longer
    creates a file named `--help` (KS-41, an unverified report, partly;
    roadmap item 4.2 adds argument parsing). Without an argument nothing
    changes: a missing `config.ron` next to `key-switch-rs.exe` is still
    created from the template at startup.
  ```

  `### Fixed`, after KS-21:

  ```markdown
  - KS-23: deleting or renaming `config.ron` while the daemon runs keeps the
    live bindings. The daemon used to write the default template in its
    place about 250 ms later and switch to it, so CapsLock switched the
    language again. The console shows `[ERROR] config reload failed:` with
    `config file <path> does not exist`, and putting the file back reloads
    it. The template is written only at startup.
  ```

### C7. `Tests: red-first: unknown fields and a missing bindings list (KS-22)` (RED)

| Test | Script | PREDICTED RED on C6 |
|---|---|---|
| T15 `config::loader::tests::a_misspelled_bindings_field_is_an_error_that_names_it` (acceptance) | `from_ron_str(r#"( bindngs: [ ( keys: "F13", action: DoNothing ) ] )"#).expect_err("a misspelled `bindings` must be an error")`; `assert!(matches!(err, ConfigError::Parse { .. }))`; `assert!(err.to_string().contains("named `bindngs`"), "{err}")` | panic ``a misspelled `bindings` must be an error: []`` |
| T16 `config::loader::tests::unknown_fields_are_errors_that_name_the_field` | rows `(field, binding)`, each wrapped `( bindings: [ <binding> ] )` except row 1: 1 `version`: `( bindings: [ ( keys: "F13", action: DoNothing ) ], version: 1 )`; 2 `tigger`: `( keys: "CapsLock", action: SwitchLanguage, tigger: Tap(200) )`; 3 `block_defualt`: `.., block_defualt: false )`; 4 `whne`: `.., whne: AppEquals("code.exe") )`; 5 `allow_extra_modifier`: `.., allow_extra_modifier: true )`; 6 `maxgap_ms`: `( sequence: (steps: ["F13", "F14"], maxgap_ms: 300), action: DoNothing )`; 7 `windw_ms`: `( chord: (keys: ["F13", "F14"], windw_ms: 80), action: DoNothing )`; 8 `cwd`: `( keys: "F13", action: Launch(exe: "notepad.exe", cwd: "C:/temp") )`; 9 `wparm`: `( keys: "F13", action: PostMessage(msg: "WM_CLOSE", wparm: 1) )`; 10 `argz`: `( keys: "F13", action: Sequence([ Launch(exe: "notepad.exe", argz: ["x"]) ]) )`. Per row: `expect_err(&format!("{field}: an unknown field must be an error"))`, `Parse`, message contains ``named `{field}` `` (with the backticks, so a valid field such as `allow_extra_modifiers` in the expected list cannot satisfy it) | panic `version: an unknown field must be an error: [Binding {..` |
| T17 `config::loader::tests::a_config_without_bindings_is_an_error` | `from_ron_str("()").expect_err("a config without `bindings` must be an error")`; `Parse`; message contains `` `bindings` `` | panic ``a config without `bindings` must be an error: []`` |
| T18 `hook::config_watcher::tests::a_save_with_a_misspelled_field_keeps_the_live_bindings` (KS-22 reload scenario) | write `ONE`, load into `KeyboardHook::new()`; overwrite with `( bindngs: [ ( keys: "F14", action: DoNothing ) ] )`; `reload_into`; `assert_eq!(hook.bindings().len(), 1, "a misspelled `bindings` must keep the live bindings (KS-22)")` | `left: 0`, `right: 1` with that message |

Existing `empty_config_is_ok_but_warned` (`loader.rs:876-881`, explicit `bindings: []`) must stay green.

### C8. `Config: reject unknown fields and require bindings (KS-22)` (green; T15-T18 green)

- D4. README `:133-135` list gains "an unknown or misspelled field"; `:606` "A save that does not load (a syntax error, an
  unknown field, a bad value) or a deleted file keeps the previous config live." Errors `:477-485`: add the unknown-field and
  missing-`bindings` lines, **copied from the T15/T17 output** (expected shape ``Unexpected field named `tigger` in
  `RawBinding`, expected one of ...``), plus "Field names are case-sensitive; RON reports the first such error only."
- Template header: "Field names are checked: an unknown or misspelled field is an error."
- CHANGELOG `### Changed`, before the command-line-path entry (KS order):

  ```markdown
  - **Breaking:** unknown and misspelled field names are load errors
    (KS-22): in a binding (`tigger: Tap(200)`, `block_defualt: false`,
    `whne: AppEquals("code.exe")`, `allow_extra_modifier: true`), at the top
    level (`bindngs: [...]`), in `sequence: (...)` and `chord: (...)`, and
    in `Launch(...)`, `PostMessage(...)` and a macro's `Launch(...)` step,
    where `cwd: "C:/temp"` used to be dropped silently. `bindings` is
    required: `()` is an error. The message names the field, for example
    <T15's message>. Such a file stops the daemon at startup; saved while
    it runs, it is rejected and the previous bindings stay live, where a
    misspelled `bindings` used to leave zero bindings live.
    `bindings: []` still loads, with `[WARN] config contains no bindings`.
  ```

### C9. `Tests: red-first: a UTF-8 BOM (KS-24)` (RED)

| Test | Script | PREDICTED RED on C8 |
|---|---|---|
| T19 `config::loader::tests::a_utf8_bom_before_the_config_is_ignored` | `from_ron_str("\u{feff}( bindings: [ ( keys: \"F13\", action: DoNothing ) ] )").unwrap_or_else(\|e\| panic!("a UTF-8 BOM must be ignored: {e}"))` has 1 binding; then `from_ron_str("\u{feff}( bindings: [ ( keys: \"F13\", action: DoNothing ) ]")` (final `)` missing) is an error whose message contains `1:51:` (columns count after the BOM; `book/src/config/ron-syntax.md:52` gives `1:51` without a BOM) | panic ``a UTF-8 BOM must be ignored: config parse error (<in-memory>): 1:1: Expected opening `(` for struct `RawConfig` `` |
| T20 `config::loader::tests::reload_reads_a_file_saved_with_a_bom` | write `b"\xEF\xBB\xBF"` + `ONE` bytes to `dir/config.ron`; `reload(&path).unwrap_or_else(\|e\| panic!("a file saved with a BOM must load: {e}"))` has 1 binding | panic ``a file saved with a BOM must load: config parse error (<path>): 1:1: Expected opening `(` ..`` |

### C10. `Config: ignore a UTF-8 BOM (KS-24)` (green; T19, T20 green)

- D5. README `:133-135`: add "The file is UTF-8, with or without a BOM." Template header: "Save as UTF-8, with or without a BOM."
- CHANGELOG `### Fixed`, after KS-23:

  ```markdown
  - KS-24: a `config.ron` saved as UTF-8 with a byte-order mark (Notepad's
    "UTF-8 with BOM", or `Set-Content -Encoding UTF8` in Windows PowerShell
    5.1) loads. It used to stop the daemon at startup with
    ``1:1: Expected opening `(` for struct `RawConfig` `` and was rejected
    on every reload.
  ```

### C11. `Tests: red-first: timing values outside 1..=10000 ms (KS-27)` (RED)

Prefix every config with a valid binding #0: `( bindings: [ ( keys: "F20", action: DoNothing ), <row> ] )`.

| Test | Script | PREDICTED RED on C10 |
|---|---|---|
| T21 `config::loader::tests::a_zero_tap_term_is_an_error_that_names_the_binding` (acceptance) | row `( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(0) )`; `expect_err("Tap(0) must be an error")`; `Bindings(errs)`; `assert_eq!(errs, ["binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms"])` | panic `Tap(0) must be an error: [Binding {..` |
| T22 `config::loader::tests::timing_values_outside_1_to_10000_ms_are_errors_that_name_the_binding` | rows -> expected substring: `trigger: Hold(0)` -> `Hold(0)`; `trigger: DoubleTap(0)` -> `DoubleTap(0)`; `trigger: Tap(10001)` -> `Tap(10001)`; `trigger: Hold(3600000)` -> `Hold(3600000)` (all on `( keys: "CapsLock", action: SwitchLanguage, .. )`); `( sequence: (steps: ["F13", "F14"], max_gap_ms: 0), action: DoNothing )` -> `max_gap_ms: 0`; same with `10001`; `( chord: (keys: ["F13", "F14"], window_ms: 0), action: DoNothing )` -> `window_ms: 0`; same with `10001`. Per row: `expect_err(&format!("{written} must be an error"))`, one error, `starts_with("binding #1 ")`, contains `written` and `1 to 10000 ms` | panic `Hold(0) must be an error: [..` |
| T23 `config::loader::tests::timing_values_at_the_bounds_load` | `( bindings: [ (keys: "F13", action: DoNothing, trigger: Tap(1)), (keys: "F14", action: DoNothing, trigger: Hold(10000)), (keys: "F15", action: DoNothing, trigger: DoubleTap(10000)), (sequence: (steps: ["F16", "F17"], max_gap_ms: 1), action: DoNothing), (chord: (keys: ["F18", "F19"], window_ms: 10000), action: DoNothing) ] )` loads 5 bindings | green (guards off-by-one) |

### C12. `Config: accept only 1..=10000 ms for terms, max_gap_ms and window_ms (KS-27)` (green; T21-T23 green)

- D6. README kinds table `:261-262`: "`max_gap_ms` (default 500, 1 to 10000)", "`window_ms` (default 50, 1 to 10000, ...)";
  after the triggers table `:327`: "`<ms>` is 1 to 10000; another value is a load error."; `:133-135` list gains "a timing
  value outside 1 to 10000 ms"; Errors: add the T21 line and "Bindings and steps are numbered from 0 in file order."
  (also covers `:421-423`).
- Template: trigger section `:85-91` "Terms are 1 to 10000 ms."; kinds `:9-11` area: "max_gap_ms (default 500) and window_ms
  (default 50) are 1 to 10000 ms."
- CHANGELOG `### Changed`, after the command-line-path entry:

  ```markdown
  - **Breaking:** timing values must be 1 to 10000 ms: the `Tap`, `Hold`
    and `DoubleTap` terms, `max_gap_ms` and `window_ms` (KS-27, an
    unverified report). `Tap(0)` used to swallow the key and almost never
    fire, and `window_ms: 0` gave a chord that swallowed its keys and never
    completed. The error names the binding, counting from 0:
    `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`.
  ```

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| Spec KS-21: absolute once in `main`, before load and watcher (`std::path::absolute`) | C4 `from_arg`, called first in `main` (C2); T1, T2; M1 |
| Spec KS-21: watcher compares case-insensitive file names within the watched folder | T3, T4, T5, T8; T7 (real notify); M2 |
| Roadmap B acceptance: "a relative path" | T1, T3 (relative target vs notify's absolute event path); M1 |
| Spec KS-23: template only on startup, default location missing | T12 (generated), T9, T10 (command line never), T11 (reload never) |
| Spec KS-23 / acceptance "a deleted file": reload keeps bindings and logs | T13, T14 (real notify), T11; M3, M4 |
| Spec KS-23: missing explicit CLI path is an error | T9, T10; M5 |
| Spec KS-22: `deny_unknown_fields` on the structs, incl. `allow_extra_modifiers` | T16 rows 1-10 (row 5 is A2's field) |
| Spec KS-22: `bindings` required / empty result not swapped | T17; T18 (reload of a typo keeps 1 binding); `empty_config_is_ok_but_warned` (deliberate `[]`) |
| Acceptance "the typo `bindngs:` (the error names the field)" | T15; M7 |
| Acceptance "a BOM" (KS-24) | T19, T20; M6 |
| Spec KS-27: 1..=10000 ms for terms, `window_ms`, `max_gap_ms`, error names the binding | T22, T23 |
| Acceptance "`Tap(0)` (an error that names the binding number)" | T21; M9 |
| Acceptance "the default template still loads, through its test" | `default_config_parses` (`loader.rs:506-513`) on every commit; T0; T12 |
| Spec docs drift items | C1 (rows in "Spec re-located"); T0; reviewer reads the C1 diff against the drift table |
| Roadmap smoke item 8 (`roadmap.md:236`) | M3, M7 (the "to the log" half waits for 1.7) |
| Roadmap 4.1 "A `config_watcher` test exists" | T3-T8, T13, T14, T18 |

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Release build; a folder, e.g. `C:\tools`, with `my.ron` = `ONE`; elevated console in that folder | `key-switch-rs.exe my.ron`; add a second binding to `my.ron` and save | console: `Watching C:\tools\my.ron for changes`; within ~1 s `[INFO] config reloaded (2 bindings live)`; `swch status` says 2 | none (needs the daemon) |
| M2 | Same files | `key-switch-rs.exe C:\TOOLS\MY.RON`; edit and save `my.ron` in Notepad | `[INFO] config reloaded` | none |
| M3 | Daemon from `swch open` with the default config | delete `config.ron` to the Recycle Bin; wait 2 s; press CapsLock; restore the file | `[ERROR] config reload failed:` / `config file ...\config.ron does not exist` / `[INFO] keeping previous bindings live`; no new `config.ron`; CapsLock still switches; after the restore `[INFO] config reloaded` | none |
| M4 | Same | rename `config.ron` to `config.bak`, wait, rename back | as M3 | none |
| M5 | No daemon; elevated `cmd` in `C:\Windows\System32` | `key-switch-rs.exe nope.ron`, Enter; `key-switch-rs.exe --help`, Enter | `[FATAL] config file C:\Windows\System32\nope.ron does not exist` (and `...\--help`); neither file exists afterwards | none |
| M6 | Daemon running | in Windows PowerShell 5.1: `(Get-Content config.ron) \| Set-Content -Encoding UTF8 config.ron` | `[INFO] config reloaded`; restart the daemon: it starts | none |
| M7 | Daemon running | rename `bindings:` to `bindngs:` and save | `[ERROR] config reload failed:` naming `` `bindngs` ``; `swch status` keeps the old count; CapsLock still switches | none |
| M8 | A folder with only `key-switch-rs.exe` | start it | `[INFO] generated default config at <exe_dir>\config.ron`; CapsLock switches the layout | none |
| M9 | Daemon running | set `trigger: Tap(0)` on a binding and save | `[ERROR] config reload failed:` with `binding #N [..]: Tap(0) is out of range: timings are 1 to 10000 ms`; old bindings live | none |

## Risks and edge cases

| Id | Risk | Handling |
|---|---|---|
| RK1 | Breaking: existing configs with typos, extra fields (`cwd:`), `()` or out-of-range timings stop the daemon at startup (`[FATAL]` + "Press Enter") | CHANGELOG Breaking (D9); message names the field or binding; README Errors |
| RK2 | One RON error at a time (RON stops at the first) | documented in README Errors (C8) |
| RK3 | FILE_NOTIFY_INFORMATION: "If there is both a short and long name for the file, the function will return one of these names, but it is unspecified which one"; ReadDirectoryChangesW: "If you opened the file using the short name, you can receive change notifications for the short name" | pre-existing; `config.ron` itself is an 8.3-valid name; follow-up (compare with `GetShortPathNameW` too) |
| RK4 | Buffer overflow: ReadDirectoryChangesW discards the batch; notify then emits a path-less event, which the filter drops, so one save can be missed | pre-existing; follow-up (treat `Event::need_rescan()` as a reload) |
| RK5 | Real-notify tests (T7, T14) depend on timing; Defender or indexer may add events | extra events only cause extra reloads of the same content; caps 10 s and 1.5 s; T14 first proves the watcher is live, so a missed delete event cannot pass vacuously against the bug; Q5 |
| RK6 | `ConfigPath` variants are public, so a caller can build a relative `CommandLine` | `main` uses `from_arg`; a relative path still works (the watcher compares names; notify absolutizes the folder, `windows.rs:531-535`) |
| RK7 | `std::path::absolute` reads the process-global current folder | called once in `run()` before any thread starts (Learn: "consumed exactly once"); tests never change it |
| RK8 | The template written by the elevated daemon is owned by Administrators in a Program Files install | unchanged; roadmap 4.4 |
| RK9 | UTF-16 files (PowerShell 5.1 `Out-File` default) still fail with "stream did not contain valid UTF-8" | pre-existing, readable; follow-up |
| RK10 | Reload, `swch off`/`on`, foreground change, shutdown during a pending gesture | unchanged: the lane only changes when `update_bindings` is called; a rejected or missing file no longer swaps, so fewer swaps reach pending gestures (reset on reload stays 2.3); shutdown drops the watcher after the loop as before (`app.rs:161-189`) |
| RK11 | `deny_unknown_fields` makes every later schema field mandatory to declare | intended; 1.1 adds `repeat` to `RawBinding` |
| RK12 | Sided/unsided modifiers, IME, layouts | not touched |

**R1-R10 (code-reviewer areas).**

- **R1.** The LL callback and the engine are unchanged; no code path of this lane runs on the hook thread after `install()`.
- **R2.** No `SendInput`.
- **R3.** No new shared state; `KeyboardHook` is used only through `update_bindings`/`bindings()`; tests build their own
  `KeyboardHook::new()` or a leaked one, never `get_hook()`.
- **R4.** One test-only `AtomicUsize` with `Relaxed` and the written argument (D8). No production atomics.
- **R5.** No builder change; `ConfigPath` is a plain enum, `App::with_config_watcher(PathBuf)` unchanged.
- **R6.** `ConfigError::Missing` gets its own `Display` arm; `source()` keeps `_ => None`.
- **R7.** Reload still goes through `update_bindings` (stable sort + one store); the keep-bindings paths store nothing.
- **R8.** No key-identity change.
- **R9.** No new registry or process read; the registry read for auto-blockers is unchanged and read-only.
- **R10.** The message loop is untouched; the only `expect` on startup's watcher path becomes an error that `App::run` turns
  into a warning, so a bad path can no longer panic the main thread before the loop.

## Docs impact

- **This lane (developer):** README (C1, C3, C4, C6, C8, C10, C12 as above), `src/config/default_config.ron` (C1, C6, C8,
  C10, C12), `CHANGELOG.md` (C1, C4, C6, C8, C10, C12), code comments (C1, C2, C4, C6).
- **Phase book sync (doc-writer), not this lane:**
  - `audit.md` status lines: KS-21 (`:487-499`), KS-22 (`:501-513`), KS-23 (`:515-527`), KS-24 (`:529-541`), KS-27
    (`:576-582`), KS-41 partly (`:688-694`).
  - `roadmap.md`: Workstream B (`:103-116`) done; issue coverage `:537-540`, `:548`, `:562` (KS-41 partly by B).
  - `gaps.md`: drift rows fixed (`:165-201`), "No range or semantic validation" (`:146`), test coverage item 7 (`:262`).
  - `architecture.md`: startup steps 2-3 (`:93-94`), thread table (`:165-166`), "Config loading and hot reload" (`:392-439`),
    the row at `:529`.
  - `config/structure.md` §2.1 (`:11`, `:19-21`), §2.2 (`:31`, `:46`, `:53-54`, `:61`), §2.3 (`:71`, `:73`, `:88-94`), §4
    (`:119-128`), §5.3 (`:180-201`); `config/errors.md` (`:64`, `:68`, `:155-157`, `:178-180`, new messages);
    `config/ron-syntax.md` (`:11-16`, `:62`, `:138`); `config/triggers-conditions.md:34`; `config/kinds.md:69`, `:139`;
    `config/actions.md:29-31`, `:189`; `config/README.md:28`, `:74`, `:79`; `config/example.md:20`, `:242-243`;
    `config/known-issues.md` rows 1-4, 6, 33, 46, 47, 49.
  - `build-and-run.md:80`, `:83`, `:93`, `:117`, `:289`, `:501`; `overview.md:40`, `:169`.
  - Every changed RON example validated with `cfgcheck`.

## Questions for the orchestrator

1. Book pages go to the phase book sync (list above), as in A2-A6 (`p0-A6-appequals/cut.md:377-390`). Recommendation: yes.
   Not blocking.
2. `CLAUDE.md:30` says "`build.rs` + `app.manifest`: the release daemon embeds a `requireAdministrator` manifest", but
   `app.manifest` is unused (`build.rs:20-46`). Recommendation: after the merge the orchestrator rewrites it to "`build.rs`:
   ... (`app.manifest` in the root is not used; roadmap version item)". Not blocking.
3. Binding numbers count from 0 (`binding #1` is the second binding), as in every existing message. Recommendation: keep;
   switching to 1-based or `line:col` is a UX change for roadmap 4.3 (`swch check` with `line:col`). Not blocking.
4. RON messages show internal names (`RawConfig`, `RawBinding`, `SequenceSpec`, `ChordSpec`), now more often (KS-24's side
   remark). Recommendation: follow-up, not this lane; `#[serde(rename = ..)]` on the containers also changes which optional
   RON struct name is accepted (`RawConfig( .. )`), so it needs its own decision. Not blocking.
5. T7 and T14 use a real `notify` watcher on `%TEMP%` and add about 3 s to `cargo test` (10 s only when red). Recommendation:
   run them by default (no hook, no injection, no registry write); if CI shows flakiness, mark them
   `#[ignore = "manual: real ReadDirectoryChangesW timing"]` in a follow-up commit. Not blocking.
6. Two small inaccuracies outside the gaps.md drift list: the template's `Ctrl+Pause` example (known-issues row 51,
   unverified) and README `:150` "Equivalent to the auto-generated default" above a block with two extra bindings.
   Recommendation: leave them to the book sync or a follow-up, to keep this lane on its list. Not blocking.
7. `deny_unknown_fields` also on `RawAction` and `RawStep` (beyond the brief's four structs) so that `Launch(cwd: ..)` and
   `PostMessage(wparm: ..)` fail. Recommendation: yes; the book files these under KS-22 (`errors.md:178`, `actions.md:31`).
   Not blocking.

## Cut readiness checklist

- Spec
  - [x] KS-21 (`audit.md:487-499`), KS-22 (`:501-513`), KS-23 (`:515-527`), KS-24 (`:529-541`), KS-27 (`:576-582`) read, plus
    KS-41 (`:688-694`). Scenarios: `my.ron` from `C:\tools` (T1, T3, M1), `tigger`/`block_defualt`/`whne` (T16), `bindingz`
    on reload (T18, M7), delete/rename (T13, T14, M3, M4), Notepad/PS 5.1 BOM (T20, M6), `window_ms: 0`/`max_gap_ms: 0`/
    `Tap(0)`/`Hold(3600000)` (T21, T22), `--help` (M5)
  - [x] Every book reference re-located (two tables); gone ones marked
  - [x] Goal, non-goals, dependent items stated
- Design
  - [x] D1-D9 each with rationale and rejected alternatives
  - [x] Owner decisions: RON kept; `println!`/`eprintln!` until 1.7; modifiers, repeat, licence untouched
  - [x] Thread named for every new path (thread table); nothing new in the LL callback
  - [x] N/A install/uninstall/timers: none added or moved
  - [x] N/A `SendInput`: none
  - [x] N/A sided modifiers: no key handling
  - [x] Reload, `swch off`/`on`, foreground change, shutdown: RK10
  - [x] Atomics: one test-only `Relaxed` counter with its argument; no `static mut`
  - [x] No new module cycle (`hook -> config` exists; `config` imports nothing from `hook`); no decision logic added, the
    engine is untouched; loader logic is unit-tested directly
- Win32
  - [x] `CompareStringOrdinal`: path, feature absent in `Cargo.toml:41-50` (added in C4), thread, errors, links;
    `std::path::absolute` / GetFullPathNameW with links
  - [x] Elevation, UIPI, pipe DACL, token inheritance considered (n/a beyond RK8)
- Commits and tests
  - [x] Red-first: C3, C5, C7, C9, C11 before C4, C6, C8, C10, C12, with exact predicted reds, compiling on their parents (D7)
  - [x] No test installs a hook or injects input; file tests use `%TEMP%`; engine tests unchanged
  - [x] N/A `#[ignore]`: no test needs a hook, injection or a registry write; M1-M9 need the daemon and have no ignored test
  - [x] Every commit passes the gate except the declared red ones
  - [x] Bisectable; README, template and CHANGELOG ride in the commit that changes the meaning
- Acceptance
  - [x] Every roadmap and spec line mapped (table)
  - [x] Every question has a recommendation

## Addendum (developer, implementation round 1): critique remarks

### I1 (Important) - cfgcheck after `load(&ConfigPath)`: RESOLVED

- **Entry point after the lane:** `key_switch_rs::config::reload(&Path)` (critique option a, no shim). It takes a path,
  is public (`src/config/mod.rs`: `pub use loader::{ConfigPath, load, reload};`) and, from C6 on, never writes a file
  (`src/config/loader.rs:178`, a missing file is `ConfigError::Missing`). cfgcheck's existence pre-check and exit code 2
  stay valid; with `reload` they are no longer needed for safety, only for the distinct exit code.
- **Evidence:** a cfgcheck built from the methodology source with `load` replaced by `reload` compiles and runs
  against every lane commit from C2 on (`D:/tmp/ks-orch/p0-B-config/cfgcheck/`, used for the validation runs in
  `impl.md`).
- **Book:** the verbatim harness at `book/src/methodology.md:376-428` is left as it is: it is also step 3 of the audit
  reproduction, built against the `7cb3521` snapshot (`methodology.md:567`), where `reload` does not exist. C2 adds one
  sentence after the exit-code table: a cfgcheck built against a tree after Workstream B calls
  `config::reload(&path)` in place of `config::load(&path)`. So the doc-writer of the phase book sync, who follows
  `doc-writer.md:80-84` to that page, builds a harness that compiles.
- **Added to the phase book-sync list:** `methodology.md:376-428` prose ("the only public loading function"; the
  existence check "matters because `load` writes the default config") for the post-lane tree; `glossary.md:437`,
  `config/known-issues.md:114`, `config/example.md:8` (they name `config::load` as the validator's entry point; true
  for the `7cb3521` runs they describe); `architecture.md:23` (M1).
- **Orchestrator items (like Q2, `.claude/` is not in the touch set):** `.claude/agents/doc-writer.md:83` ("`config::load`
  writes the default config to any missing path": false after C6; the procedure itself still works) and
  `.claude/agents/code-reviewer.md:25` ("then `config::load` and `update_bindings`": the reload path is now
  `config::reload`).

### M1 (Minor) - README tree and architecture.md name `default_config_path`: RESOLVED

README tree line `mod.rs - public API: ConfigPath, load(), reload()` in C2 (`5acbe61`); `architecture.md:23` is on the
book-sync list above.

### M2 (Minor) - "Not macros" replacement lists ToggleCapsLock as native: RESOLVED

C1 (`13ce323`) README Implementation Features: "language switching, window operations and messages use Win32 calls;
key output (`ToggleCapsLock`, `PressKey`, `Media`, macro `Text`/`Key`/`Combo`) is synthesized with `SendInput`".
