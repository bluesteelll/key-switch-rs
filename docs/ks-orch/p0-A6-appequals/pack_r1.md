# Context pack r1: p0-A6-appequals (KS-03, folding part of KS-28)

For: tester and code-reviewer, round 1. Written by project-analyst (scout), read-only.

## Scope and delta

- Branch `fix/app-equals`, HEAD `32703a8`, tree clean. Base `master` @ `a1b5a27` ("Merge branch 'fix/ipc-control'").
- Inputs: `D:/tmp/ks-orch/p0-A6-appequals/cut.md` (CUT READY, with "Critique response" addendum at `:433-472`),
  `critique.md` (round 1: 0 Critical, 1 Important I1, 4 Minor M1-M4), `impl.md`.

`git log --oneline master..fix/app-equals`:

| Commit | Subject | Cut id | Files (`git show --stat`) |
|---|---|---|---|
| `276caf1` | Conditions: look up the app name by process id | C1 (green refactor) | `src/data/condition.rs` +22/-17 |
| `9b5c2db` | Tests: red-first: AppEquals resolves this test process (KS-03) | C2 (RED: T1, T2, T7) | `src/data/condition.rs` +128 (tests module only) |
| `de03573` | Conditions: read the app name with QueryFullProcessImageNameW (KS-03) | C3 | CHANGELOG +19, Cargo.toml -1, README +8/-?, default_config.ron +2, condition.rs +110/-45 net |
| `cd1628d` | Tests: red-first: TitleEquals and AppEquals ignore case beyond ASCII (KS-28) | C4 (RED: T3, T4) | `src/data/condition.rs` +41, `src/engine/scenarios.rs` +37 |
| `32703a8` | Conditions: TitleEquals and AppEquals fold case like TitleContains (KS-28) | C5 | CHANGELOG +4, README +2/-1, default_config.ron +1/-1, condition.rs +21/-13 |

`git diff --stat master..fix/app-equals`: `CHANGELOG.md` +23, `Cargo.toml` -1, `README.md` 11 (+/-), `src/config/default_config.ron` 4,
`src/data/condition.rs` 333 (+/-), `src/engine/scenarios.rs` +37. Total 6 files, +351/-58.

Untouched (verified `git diff --stat master..fix/app-equals -- src/engine/env.rs src/engine/testing.rs src/hook/` is empty):
`src/engine/env.rs`, `src/engine/testing.rs`, `src/hook/*`, `Cargo.lock` (not in diff stat), `book/`.

## File map

| path:range | What lives there | Why it matters this round |
|---|---|---|
| `src/data/condition.rs:1-14` | module doc; `:11-14` folding wording | C5 doc change (KS-28) |
| `src/data/condition.rs:16-26` | imports: `Foundation::*`, `Threading::{OpenProcess, PROCESS_NAME_NATIVE, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW}`, `core::{Owned, PWSTR}` | C3; `ProcessStatus` and `c_void` imports gone |
| `src/data/condition.rs:28-51` | `enum Condition`; variant docs `:34-44` | C5 doc change; no new variant |
| `src/data/condition.rs:61-80` | `Condition::evaluate`; `AppEquals` `:64-67`, `TitleContains` `:68-71`, `TitleEquals` `:72-75` | C5 routes AppEquals/TitleEquals through `equal_ignoring_case` |
| `src/data/condition.rs:83-92` | `fn equal_ignoring_case` (ASCII fast path `eq_ignore_ascii_case`, else `to_lowercase() ==`) | KS-28 fold; hot path (conditions run per key-down) |
| `src/data/condition.rs:94-101` | `trait ForegroundInfo` (`app`, `title`) | spec: abstraction kept |
| `src/data/condition.rs:103-178` | `ForegroundContext` (`OnceCell` caches), `capture` `:117-125`, `fetch_app` `:139-154`, `fetch_title` `:156-167`, `impl ForegroundInfo` `:170-178` | C1/C3: `fetch_app` keeps HWND -> pid, calls `process_image_name(pid).ok()` `:153` |
| `src/data/condition.rs:180-182` | `const IMAGE_PATH_CAPACITY: usize = 1024` | D4 buffer |
| `src/data/condition.rs:184-219` | `fn process_image_name(pid) -> windows::core::Result<String>`: `OpenProcess` in `Owned::new` `:193-194`, stack buffer `:196-197`, `QueryFullProcessImageNameW(.., PROCESS_NAME_NATIVE, ..)` `:207-214`, clamp `:215`, `ERROR_INVALID_DATA` on empty name `:216-217` | KS-03 core; 2 `unsafe` blocks; RAII; R9 |
| `src/data/condition.rs:221-227` | `fn image_file_name(&[u16]) -> Option<&[u16]>` (`rsplit` on `\`, empty -> `None`) | pure helper, T5 |
| `src/data/condition.rs:229-238` | test-module imports (`CreateWindowExW`, `DestroyWindow`, `HWND_MESSAGE`, `WINDOW_EX_STYLE`, `WINDOW_STYLE`, `PCWSTR`, `w`) | added in C2 for T7 |
| `src/data/condition.rs:240-335` | pre-existing tests (FakeForeground, leaf, always, not, and/or, nested) | unchanged context |
| `src/data/condition.rs:337-345` | helper `test_exe_file_name()` from `std::env::current_exe()` | used by T1, T2, T7 |
| `src/data/condition.rs:347-360` | T1 `process_image_name_resolves_the_current_process` | spec "non-ignored test resolves the current pid" |
| `src/data/condition.rs:362-388` | T2 `app_equals_matches_the_current_process` | roadmap A acceptance `book/src/roadmap.md:97` |
| `src/data/condition.rs:390-435` | `MessageOnlyWindow` (create `:398-425`, `Drop` `:428-435`), 2 test `unsafe` blocks | T7 support (critique M2) |
| `src/data/condition.rs:437-456` | T7 `foreground_context_reads_a_window_of_this_process` | `fetch_app`/`fetch_title` wiring |
| `src/data/condition.rs:458-497` | T3 `equals_conditions_fold_case_like_title_contains` | KS-28 (Cyrillic, Greek final sigma, Kelvin sign) |
| `src/data/condition.rs:499-516` | T5 `image_file_name_is_the_part_after_the_last_backslash` | 5 rows from cut `:257-263` |
| `src/data/condition.rs:518-527` | T6 `foreground_context_names_the_focused_notepad`, `#[ignore = "manual: ..."]` | manual check M4 |
| `src/engine/scenarios.rs:274-309` | T4 `when_title_equals_and_app_equals_ignore_case_beyond_ascii` | KS-28 via `Engine::on_key` on `FakeEnv`; roadmap `:385` |
| `src/engine/env.rs:47-52` | `Env::foreground_app` / `foreground_title` | abstraction, untouched |
| `src/engine/env.rs:55-69` | `EnvForeground` impl `ForegroundInfo` over `Env` | abstraction, untouched |
| `src/engine/mod.rs:365-378` | `on_key_down`: `EnvForeground(env)` `:368`, `binding.condition.evaluate(&foreground)` `:376` | conditions evaluated for every binding before matching (hot path, RK1) |
| `src/engine/testing.rs:95-101`, `:217-221` | `FakeEnv::foreground_app/title`; `Scenario::foreground(app, title)` | fake used by T4 |
| `src/hook/keyboard_hook_callback.rs:105-141` | `HookEnv { foreground: OnceCell<ForegroundContext> }`, `foreground()` `:116-118`, `Env::foreground_app/title` `:134-140` | real implementation path to `fetch_app` |
| `src/hook/keyboard_hook_callback.rs:173`, `:291`; `src/hook/keyboard_hook.rs:147` | `HookEnv::new()` call sites (callback, WinEvent foreground change, reset) | thread context of the lookup (main thread) |
| `Cargo.toml:41-51` | `windows` features; `Win32_UI_WindowsAndMessaging` `:43`, `Win32_System_Threading` `:49`; `Win32_System_ProcessStatus` removed | D6 |
| `CHANGELOG.md:130-148` | Fixed / KS-03 entry, manual check `:144-148`, KS-35 interaction `:136-142` | spec "manual-check note in CHANGELOG"; critique I1, M3 |
| `CHANGELOG.md:230-233` | Fixed / KS-28 entry | C5 |
| `README.md:430` | `block_original_combo` row, KS-35 sentence | critique I1 |
| `README.md:439-440` | folding sentence with Cyrillic example | C5 |
| `README.md:466` | `AppEquals` table row | C3 |
| `README.md:472-474` | UWP / `ApplicationFrameHost.exe` note | C3, RK2 |
| `src/config/default_config.ron:131-132` | flag comment, KS-35 | I1 |
| `src/config/default_config.ron:141` | "string comparisons ignore case, in every alphabet" | C5 |

## Facts

Code (HEAD `32703a8`)

1. `fetch_app` returns `None` for an invalid hwnd (`src/data/condition.rs:143-145`) and for pid 0 (`:150-152`), else `process_image_name(pid).ok()` (`:153`).
2. `process_image_name` opens the process with `PROCESS_QUERY_LIMITED_INFORMATION`, `bInheritHandle=false` (`src/data/condition.rs:194`), wrapped in `windows::core::Owned::new` (`:193-194`); the `?` on `OpenProcess` propagates its error before `Owned` is built (`:194`).
3. `windows` 0.62.2 `OpenProcess` returns `Err(Error::from_thread())` for an invalid handle (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/src/Windows/Win32/System/Threading/mod.rs:1192-1196`).
4. `Owned<T: Free>::drop` calls `free()` (`windows-core-0.62.2/src/handles.rs:29-33`); `impl Free for HANDLE` calls `CloseHandle` when `!is_invalid()` (`windows-0.62.2/src/Windows/Win32/Foundation/mod.rs:5297-5307`).
5. `QueryFullProcessImageNameW` is called with `*process`, `PROCESS_NAME_NATIVE`, `PWSTR(buffer.as_mut_ptr())`, `&mut size` and `?` (`src/data/condition.rs:207-214`); `buffer` is `[0u16; 1024]` and `size` starts at `1024` (`:196-197`, `:182`).
6. The crate wrapper maps the BOOL result with `.ok()` (`windows-0.62.2/.../Threading/mod.rs:1258-1261`).
7. After the call the path slice is `&buffer[..(size as usize).min(IMAGE_PATH_CAPACITY)]` (`src/data/condition.rs:215`); an empty file name maps to `Error::from(ERROR_INVALID_DATA)` (`:216-217`); the name is decoded with `String::from_utf16_lossy` (`:218`).
8. `image_file_name` splits on `u16::from(b'\\')`, takes the last piece and filters empty (`src/data/condition.rs:223-227`).
9. Comments at `src/data/condition.rs:186-189` (why the limited right suffices) and `:198-201` (native form, "about 2 us per lookup instead of 20 us") precede the two calls.
10. SAFETY comments: `OpenProcess`/`Owned::new` `src/data/condition.rs:191-192`; `QueryFullProcessImageNameW` `:203-206`; test `CreateWindowExW` `:403-406`; test `DestroyWindow` `:430-432`; pre-existing `GetForegroundWindow` `:118`, `GetWindowThreadProcessId` `:146-147`, `GetWindowTextW` `:161`.
11. `equal_ignoring_case` uses `eq_ignore_ascii_case` when both strings are ASCII and `a.to_lowercase() == b.to_lowercase()` otherwise (`src/data/condition.rs:86-92`); `TitleContains` still uses `actual.to_lowercase().contains(&needle.to_lowercase())` (`:69`).
12. `AppEquals` and `TitleEquals` both call `equal_ignoring_case(actual, target)` (`src/data/condition.rs:65`, `:73`); `None` from the context yields `false` (`:66`, `:74`).
13. `ForegroundContext::fetch_title` still uses a 512-unit buffer (`src/data/condition.rs:160`).
14. `rg -n "GetModuleBaseNameW|ProcessStatus" src Cargo.toml` exits 1 at HEAD; outside `book/` the only match is the CHANGELOG prose `CHANGELOG.md:131`. (`.claude/agents/code-reviewer.md:58,128` per `impl.md` open points; not re-read here.)
15. `rg -n "ProcessHandleGuard|CloseHandle" src/data/condition.rs` exits 1 at HEAD.
16. `Cargo.toml` diff removes only the line `"Win32_System_ProcessStatus",` (was `Cargo.toml:50` on master); `Cargo.lock` is not in the diff.
17. Non-ASCII characters in `src/data/condition.rs`: 5 lines both on master and at HEAD (em dashes in doc comments at `:30`, `:47`, `:49`, `:104`, `:114`); the new test strings use `\u{..}` escapes (`:461-462`, `:483-495`; `src/engine/scenarios.rs:278-279`, `:302`).
18. `CHANGELOG.md` has 0 non-ASCII lines on master and 1 at HEAD (`CHANGELOG.md:232`, Cyrillic example); `README.md:440` has the same Cyrillic example (README had 90 non-ASCII lines on master).
19. `HookEnv` captures `ForegroundContext` lazily through `OnceCell::get_or_init(ForegroundContext::capture)` (`src/hook/keyboard_hook_callback.rs:116-118`) and forwards `foreground_app/title` (`:134-140`).
20. The engine wraps the `Env` in `EnvForeground(env)` (`src/engine/mod.rs:368`) and evaluates each binding's condition before matching (`src/engine/mod.rs:374-378`).

Tests (as written at HEAD)

21. T1 (`src/data/condition.rs:347-360`) calls `process_image_name(std::process::id())`, panics with `process_image_name(own pid) failed: {e}` on `Err`, and asserts ASCII-case-insensitive equality with `test_exe_file_name()`, no `\`, and an `.exe` suffix.
22. T2 (`src/data/condition.rs:362-388`) uses local `OwnProcess(process_image_name(own pid).ok())`; asserts `AppEquals(exe.to_ascii_uppercase())` true (message `AppEquals matches the test's own process (KS-03)`), `And([AppEquals(exe), Always])` true, `Not(AppEquals(exe))` false, `AppEquals("notepad.exe")` false.
23. T7 (`src/data/condition.rs:440-456`) builds `ForegroundContext { hwnd: <message-only STATIC window>, .. }` directly (private fields, same module) and asserts `app()` equals the test exe name (message `ForegroundContext::app() of a window of this process is {:?}, not {exe:?} (KS-03)`), `AppEquals(exe)` through `impl ForegroundInfo for ForegroundContext`, and `title() == Some("key-switch-rs condition test")` (`:396`, `:455`). The window uses parent `HWND_MESSAGE` (`:417`).
24. T3 (`src/data/condition.rs:458-497`): Cyrillic title/app, reference `TitleContains(title.to_uppercase())` (`:468`), `TitleEquals(title.to_uppercase())` with message `TitleEquals ignores Cyrillic case like TitleContains` (`:469-472`), `AppEquals(app.to_uppercase())` (`:473-476`), prefix negative (`:477`), Greek final sigma (`:481-487`), Kelvin sign `\u{212a}.exe` vs `k.exe` / `K.EXE` (`:491-496`).
25. T5 (`src/data/condition.rs:499-516`): `C:\Windows\System32\notepad.exe` -> `notepad.exe`; `\Device\HarddiskVolume3\Apps\App.EXE` -> `App.EXE`; `notepad.exe` -> itself; `C:\dir\` -> `None`; `""` -> `None`.
26. T6 (`src/data/condition.rs:518-527`) is `#[ignore = "manual: start it, then focus a Notepad window within 5 s; it expects ForegroundContext::capture().app() to be notepad.exe in any case"]`, sleeps 5 s, reads the real foreground (read-only).
27. T4 (`src/engine/scenarios.rs:274-309`): config built with `format!` naming the upper-cased Cyrillic title (`F13`, `SwitchLanguage`, `TitleEquals`) and app (`F14`, `ToggleCapsLock`, `AppEquals`); scenario A `.foreground(app, title)` expects F13 down suppressed + fired `[SwitchLanguage]`, up suppressed, F14 down suppressed + fired `[ToggleCapsLock]`, up suppressed (`:290-301`); scenario B `.foreground(app, "\u{41a}\u{430}\u{43b}")` expects F13 down passed, nothing fired, up passed (`:303-308`).
28. Commit `9b5c2db` touches only `src/data/condition.rs` and its hunks are inside `mod tests` (hunks `@@ -205,6 +205,13 @@` imports, `@@ -303,4 +310,125 @@`); commit `cd1628d` adds only test hunks (`condition.rs @@ -443,6 +443,47 @@`, `scenarios.rs @@ -271,6 +271,43 @@`).

## Red-first and acceptance proofs (from cut.md / impl.md)

Red replay points: the tree of the red commit itself is "old code + new tests" (C2 = `9b5c2db` on C1 code; C4 = `cd1628d` on C3 code).
On `master` T1/T2/T7 do not compile (`process_image_name` missing; `cut.md:230-231`, `impl.md` Gates).

| Test | Red at | Predicted / recorded red text (`impl.md`, commit body of `9b5c2db`) | Green at |
|---|---|---|---|
| T1 `data::condition::tests::process_image_name_resolves_the_current_process` | `9b5c2db` | `process_image_name(own pid) failed: Access is denied. (0x80070005)` (message text follows UI language; code `0x80070005` fixed, `cut.md:247`) | `de03573` |
| T2 `data::condition::tests::app_equals_matches_the_current_process` | `9b5c2db` | `AppEquals matches the test's own process (KS-03)` | `de03573` |
| T7 `data::condition::tests::foreground_context_reads_a_window_of_this_process` | `9b5c2db` | `ForegroundContext::app() of a window of this process is None, not "key_switch_rs-9c8e127322721cde.exe" (KS-03)` (hash varies by build) | `de03573` |
| T5 `data::condition::tests::image_file_name_is_the_part_after_the_last_backslash` | n/a (arrives green in `de03573`; compile error on C2) | n/a | `de03573` |
| T3 `data::condition::tests::equals_conditions_fold_case_like_title_contains` | `cd1628d` | `TitleEquals ignores Cyrillic case like TitleContains` (step 3) | `32703a8` |
| T4 `engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii` | `cd1628d` | `scenario: down F13 at +0 ms: expected the event to be suppressed, it was passed` (format `src/engine/testing.rs:461-462`, `:549-555`, `:585` per `cut.md:301`) | `32703a8` |
| T6 `data::condition::tests::foreground_context_names_the_focused_notepad` | ignored | manual (M4) | n/a |

Commands (from `cut.md:223`, `impl.md` Gates and Manual checks):

- Gate at HEAD: `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Expected: all green, T6 reported as 1 ignored in the lib tests (manual, not a pass).
- Developer's scoped run at HEAD (`impl.md` Gates): `cargo test --locked --lib -- data::condition engine::scenarios config::loader` -> recorded `172 passed, 0 failed, 1 ignored`. Full `cargo test --locked` was not run by the developer (left to the tester).
- Red replay (per `impl.md`): T1/T2/T7 at the tree of `9b5c2db`, T3/T4 at the tree of `cd1628d`; e.g. `cargo test --locked --lib -- data::condition` and `cargo test --locked --lib -- engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii`. Expected: exactly the named tests fail with the texts above, no other test fails (`cut.md:224-225`). Replay needs a separate work tree of those commits (the scout did not run it).
- Acceptance greps (`cut.md:323-324`, addendum `:469-472`): `rg -n "GetModuleBaseNameW|ProcessStatus" src Cargo.toml` and `rg -n "ProcessHandleGuard|CloseHandle" src/data/condition.rs` -> both exit 1 (verified at HEAD, Facts 14-15).
- Manual (owner only, never by agents): `cargo test --locked --lib -- --ignored foreground_context_names_the_focused_notepad` from a non-elevated console, focus Notepad within 5 s (M4). M1-M3 and optional M5 need the daemon (`impl.md` Manual checks table).

Acceptance mapping (`cut.md:319-332`): spec replace API -> C3, T1, T5, grep; RAII -> `Owned<HANDLE>` (Facts 2, 4), grep; abstraction kept -> `env.rs`/`testing.rs` untouched, T4 on `FakeEnv`; non-ignored current-pid test -> T1; roadmap A `roadmap.md:97` -> T2; Unicode folding -> T3, T4; roadmap `:385` folding line -> T4 scenario A, T3 step 3; manual-check note in CHANGELOG -> `CHANGELOG.md:144-148`; smoke checklist 7 `roadmap.md:235` -> M1.

Deviations recorded in `impl.md`: D1 `PROCESS_NAME_NATIVE` instead of the cut's `PROCESS_NAME_WIN32` (timing table `cut.md:451-454`: native p50 2.2-2.5 us, Win32 p50 19.9-24.2 us); D2 T7 added (message-only window); README `:430` and template `:131-132` KS-35 sentences (I1); CHANGELOG KS-03 text differs from the cut's verbatim block (`cut.md:275-290`); per-commit full `cargo test` not run by the developer.

## Open points (verbatim scope from impl.md "Open points / pre-existing warnings")

1. No pre-existing warnings seen in fmt, check or clippy.
2. `.claude/agents/code-reviewer.md:58,128` (R9) still describe `GetModuleBaseNameW`; orchestrator after the merge (cut Q2).
3. Book sync (phase, not this lane): cut "Docs impact" list (`cut.md:377-385`) plus the KS-35 interaction in `config/*.md` and the native path form.
4. KS-35 now observable (roadmap 1.2). RK3 (non-elevated daemon vs elevated foreground) unverified, accepted (`cut.md:349`, Q5 `:398-400`).
5. `fetch_title`'s 512-unit buffer (titles over 511 units never satisfy `TitleEquals`): follow-up, untouched (Fact 13).
6. rustfmt wrote `src/data/condition.rs` with LF in the working copy; git status clean.

## Invariants in play

- **R1 Hook callback**: the lookup runs inside the callback on the main thread via `HookEnv` (`src/hook/keyboard_hook_callback.rs:116-118`, `:134-136`, `:173`) and `src/engine/mod.rs:376`; per-lookup work: `GetWindowThreadProcessId` (`condition.rs:149`), `OpenProcess` (`:194`), `QueryFullProcessImageNameW` (`:207-214`), `CloseHandle` via `Drop` (Fact 4), one `String` (`:218`); non-ASCII fold allocates two `String`s (`:90`). Panic paths in non-test code: slice bounded by `.min` (`:215`); no `unwrap`/`expect` in `:139-227`. RK1 hot-path cost accepted, roadmap 3.5 (`cut.md:347`).
- **R2 Sentinel**: no `SendInput` added (cut R2 `:362`).
- **R3 Shared state**: `ForegroundContext` per event on the stack inside `HookEnv` (`keyboard_hook_callback.rs:105-107`, `:173`); no static, lock or atomic added.
- **R6 Display**: no new `Condition` variant (`condition.rs:28-51`).
- **R9 Registry and process reads**: `PROCESS_QUERY_LIMITED_INFORMATION` (`condition.rs:194`), handle closed by `Owned` on every path including the `?` at `:213` and `:217` (Fact 4); buffer length in UTF-16 units (`:197`, `:205-206`); KS-03 (`book/src/audit.md:185`).
- **R10**: no thread, window or pump added in non-test code; T7's window is created and destroyed on the test thread (`condition.rs:398-435`).
- **KS-03** (`book/src/audit.md:23`, `:185`), **KS-28** (`book/src/audit.md:585-590`), **KS-35** (`book/src/audit.md:646`, interaction documented at `CHANGELOG.md:136-142`, `README.md:430`, `default_config.ron:131-132`).
- Source ASCII rule (CLAUDE.md principle 8): Fact 17.

## Not covered

- No gate or test was run by the scout (fmt, clippy, test, red replay): owned by the tester.
- `critique.md` read only by headings (`:20-141`); its full text not re-checked against code.
- `cut.md:1-206` (spec table, decisions D1-D7, threads, Win32 API choices) not re-read beyond its headings; `cut.md:207-472` read.
- `.claude/agents/code-reviewer.md:58,128` not re-read.
- Book pages listed for book sync not read; `book/src/roadmap.md` read only at `:97`, `:385`.
- Microsoft Learn semantics of `QueryFullProcessImageNameW` / `PROCESS_NAME_NATIVE` and of the limited right against protected processes not fetched (`condition.rs:186-189` claim is UNVERIFIED by the scout).
- Line-level diff of `README.md` beyond the hunks at `:427-430`, `:436-440`, `:462-475` not otherwise inspected (diff shows only those hunks).
