# Cut: p0-A6-appequals — Workstream A (AppEquals part): KS-03, plus the folding part of KS-28

Tree: `D:/key-switch-rs/key-switch-rs`, `master` @ `a1b5a27` (clean). Branch: `fix/app-equals`. `windows` is locked at 0.62.2
(`Cargo.lock:291-292`), `windows-core` 0.62.2 (`:312-313`), `windows-result` 0.4.1 (`:374-375`). Facts about the crate below
are read from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/...`.

## Verdict

CUT READY

## Spec re-located

The book cites `7cb3521`. "Now" means `a1b5a27`.

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| KS-03 `fetch_app` `condition.rs:139-144` | `src/data/condition.rs:126-154` | moved | `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ..)` `:144`, `GetModuleBaseNameW` `:149`. Defect present |
| KS-03 false comment "LIMITED is enough for GetModuleBaseNameW" `:133-135` | `src/data/condition.rs:138-140` | moved | removed in C3 |
| KS-03 length 0 -> `None` `:145-147` | `src/data/condition.rs:150-152` | moved | |
| KS-03 `AppEquals` maps `None` to false `:60-65` | `src/data/condition.rs:62-65` | moved | unchanged by this lane |
| KS-03 import `ProcessStatus::GetModuleBaseNameW` | `src/data/condition.rs:20`; feature `Win32_System_ProcessStatus` `Cargo.toml:50` | n/a | `rg GetModuleBaseNameW\|ProcessStatus` outside `book/` finds only these and `.claude/agents/code-reviewer.md:58,128` |
| KS-03 RAII guard | `ProcessHandleGuard` `src/data/condition.rs:180-195`; `c_void` import `:15` and the "silence unused" const `:197-199` | n/a | replaced in C3 (D3) |
| KS-03 template example `default_config.ron:170-174` | `src/config/default_config.ron:192-198`; `And([AppEquals..])` example `:200-208` | moved | commented out; they work after C3 |
| KS-28 `condition.rs:62` (AppEquals), `:67` (TitleContains), `:71` (TitleEquals) | `src/data/condition.rs:63`, `:67`, `:71` | moved / unchanged / unchanged | `eq_ignore_ascii_case` at `:63`, `:71`; `to_lowercase` at `:67`. Defect present |
| KS-28 module comment "all comparisons case-insensitive" `:9-11` | `src/data/condition.rs:11-13` | moved | true after C5 |
| KS-28 README "All string comparisons are case-insensitive" `README.md:361` | `README.md:439`; variants table `:460-470` (AppEquals row `:465`, TitleEquals `:467`) | moved | |
| known-issues row 48: template "case-insensitive string comparisons" `default_config.ron:115` | `src/config/default_config.ron:139` | moved | |
| known-issues row 50: template `AppEquals` examples `:173`, `:181` | `src/config/default_config.ron:197`, `:205` | moved | |
| architecture: context collected in `handle_key_down` `keyboard_hook_callback.rs:207-211`, condition before keys `:217-221` | engine: `src/engine/mod.rs:365-378` (`EnvForeground` `:368`, `evaluate` before matching `:376`); adapter: `HookEnv` `src/hook/keyboard_hook_callback.rs:100-141` (lazy `OnceCell<ForegroundContext>` `:106-118`, `foreground_app` `:134-136`); lock `decide_event` `:178-181` | moved (restructured by A1) | lookup runs on the main thread under `with_engine` |
| architecture `ForegroundContext` `L85-L163` | `src/data/condition.rs:90-178` | moved | |
| `Env::foreground_app` / `foreground_title` | `src/engine/env.rs:47-52`; `EnvForeground` `:58-68` | n/a | unchanged (spec: keep the abstraction) |
| fake foreground for engine tests | `FakeEnv` `src/engine/testing.rs:58-102`; `Scenario::foreground` `:216-221`; `FakeForeground` `src/data/condition.rs:206-219` | n/a | unchanged |
| existing `when:` scenarios | `src/engine/scenarios.rs:208-272`, `:1851-1878` | n/a | ASCII only; stay green |
| gaps: no test evaluates `AppEquals` against a real process `gaps.md:258` | `src/data/condition.rs:201-301` (fakes and `Always` only) | unchanged | C2 adds the test |
| roadmap A bullet `roadmap.md:78`, acceptance `:97`, smoke 7 `:235`; item 3.5 `:377-385` | same lines | unchanged | |
| audit probe (flag `PROCESS_NAME_WIN32`, 1024 buffer) `methodology.md:177-181`, result `:200-206`; live check `:530` | same lines | unchanged | basis of D1, D4, M1 |
| conditions only on key-down, never in timers | `src/engine/mod.rs:365-378`; `on_foreground_changed` `:337-340` and `reset` `:358-363` never ask for the foreground | n/a | thread table below |

Result: KS-03 and the folding part of KS-28 are both present on `a1b5a27`. Nothing is already fixed.

## Goal and non-goals

**After the lane**

- `ForegroundContext::app()` returns the file name of the foreground window's executable (`"Notepad.exe"`), read with
  `QueryFullProcessImageNameW` on the existing `PROCESS_QUERY_LIMITED_INFORMATION` handle. `AppEquals` matches;
  `Not(AppEquals(..))` is false inside that app; `And([AppEquals(..), ..])` can hold (KS-03).
- The process handle is closed by `windows::core::Owned<HANDLE>` on every path.
- A non-ignored test resolves `std::process::id()` to the test binary's file name with the real function.
- `AppEquals` and `TitleEquals` ignore case with the same Unicode lowercasing as `TitleContains` (`str::to_lowercase`),
  with an allocation-free fast path when both strings are ASCII (KS-28, folding part).
- `Env`, `ForegroundInfo`, `FakeEnv` and the engine are unchanged.

**Non-goals**

| Left out | Owner |
|---|---|
| Cache app and title per HWND from `EVENT_SYSTEM_FOREGROUND`; evaluate `when:` after the keys match; the 100 us budget | roadmap 3.5 (`roadmap.md:377-385`) |
| Zero allocation in `Engine::on_key` (the app-name `String`, the non-ASCII fold) | roadmap 3.4 (`roadmap.md:375`) |
| Auto-blocker inherits the owning binding's `when:` (KS-35, unverified). Now observable: CapsLock with `when: AppEquals("game.exe")` fires only in the game while its Alt+Shift blocker acts everywhere | roadmap 1.2 (`roadmap.md:141`, `:164`) |
| UWP windows report `ApplicationFrameHost.exe`; no AUMID or child-window lookup | none; documented in README |
| `TitleContains` behaviour, `fetch_title` (512-unit buffer truncates titles over 511 units, so `TitleEquals` on them never holds; no KS id) | follow-up note only |

**Later items that depend on this lane:** 3.5 (caches this function's result; its KS-28 column becomes "none"), 3.4, 1.2, the
phase book sync.

## Decisions

### D1: Image path through `QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, ..)`, file name = UTF-16 after the last `\`

- **What.** `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)` as today, then `QueryFullProcessImageNameW` into a stack
  buffer, then the slice after the last `0x005C`, then `String::from_utf16_lossy` of that slice only.
- **Why.**
  - The function accepts the limited right: "This handle must be created with the PROCESS_QUERY_INFORMATION or
    PROCESS_QUERY_LIMITED_INFORMATION access right" (Learn, QueryFullProcessImageNameW).
  - Microsoft recommends exactly this for a remote process: "To retrieve the base name of the main executable module for a remote
    process, use the GetProcessImageFileName or QueryFullProcessImageName function to retrieve the module name and then use the
    strrchr function ... This is more efficient and more reliable than calling GetModuleBaseName with a NULL module handle"
    (Learn, GetModuleBaseNameW, Remarks).
  - The limited right also works on protected processes: the Learn list of rights "not allowed from a process to a protected
    process" includes `PROCESS_QUERY_INFORMATION` and `PROCESS_VM_READ`, not `PROCESS_QUERY_LIMITED_INFORMATION`.
  - The audit probe ran this exact call (flag `PROCESS_NAME_WIN32`, 1024 buffer) on this machine and got the path for its own
    process and for `explorer.exe` (`methodology.md:177-181`, `:200-206`).
- **Rejected.**
  - Keep `GetModuleBaseNameW` and open with `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ`: denied for protected processes; reads
    the target's loader data, which Learn says "may fail or return incorrect information" while the module list changes.
  - `GetProcessImageFileNameW`: same right, device path, keeps the `ProcessStatus` feature for no gain.
  - `PROCESS_NAME_NATIVE`: same final component; not chosen because flag 0 is the form the audit verified here. Neither format's
    cost is documented (see Risks RK4).
  - Toolhelp snapshot (`CreateToolhelp32Snapshot` + `Process32NextW`): enumerates every process per key-down; far too heavy for the
    hook thread.
- **Cost.** Same call count per lookup as master (OpenProcess, one query, CloseHandle). The calls now succeed, so the existing
  `Option<String>` cache (`condition.rs:96`, `:117`) now holds one allocated `String` per key-down that evaluates an `AppEquals`.

### D2: A pid-level function; `fetch_app` keeps the window -> pid half

- **What.** Private, in `src/data/condition.rs`:
  - `fn process_image_name(pid: u32) -> windows::core::Result<String>`: the executable's file name, e.g. `"notepad.exe"`.
  - `ForegroundContext::fetch_app` (`:126-154`) keeps its HWND check and `GetWindowThreadProcessId` (`:127-136`), then returns
    `process_image_name(pid).ok()`.
  - Errors: `OpenProcess` and `QueryFullProcessImageNameW` errors propagate (`?`); an empty file name is
    `Err(ERROR_INVALID_DATA.into())` (`From<WIN32_ERROR> for windows_core::Error`, `windows-0.62.2/src/extensions/Win32/Foundation/WIN32_ERROR.rs:39-43`).
- **Why.** The spec asks for a test that resolves the current process id. A `Result` puts the HRESULT into the test's panic text;
  the hook path maps every failure to `None`, as today. `ForegroundInfo` and `Env` stay unchanged (spec).
- **Rejected.** A test through a message-only window of the test thread (`CreateWindowExW(HWND_MESSAGE, ..)`): more `unsafe` in a
  test to cover only `GetWindowThreadProcessId`, which the ignored test T6 and M1 cover. A `Result` in `ForegroundInfo`: churn in
  the engine, which has no use for the error.

### D3: RAII with `windows::core::Owned<HANDLE>`

- **What.** `let process = unsafe { Owned::new(OpenProcess(..)?) };` with a SAFETY line ("the handle was just returned by a
  successful `OpenProcess`; nothing else owns or closes it"). Pass `*process` to the query. Delete `ProcessHandleGuard`
  (`:180-195`), the `c_void` import (`:15`) and the const at `:197-199`, whose comment is wrong.
- **Why.** `Owned<T: Free>` drops through `Free::free` (`windows-core-0.62.2/src/handles.rs:13-33`, re-exported as `windows::core`
  by `windows-0.62.2/src/lib.rs:18` and `windows-core/src/windows.rs:12-13`). `impl Free for HANDLE` calls `CloseHandle` only when
  `!is_invalid()` (`windows-0.62.2/src/Windows/Win32/Foundation/mod.rs:5297-5306`). The `OpenProcess` wrapper never returns an
  invalid handle in `Ok` (`System/Threading/mod.rs:1192-1196`). One `unsafe` replaces the guard's `unsafe` drop; the handle is
  closed on every return path, including `?`.
- **Rejected.** Keep the local guard: a copy of crate code plus a dead const. A `HANDLE` closed by hand: misses the `?` paths.

### D4: Buffer of 1024 UTF-16 units on the stack; length clamped

- **What.** `const IMAGE_PATH_CAPACITY: usize = 1024;` `let mut buffer = [0u16; IMAGE_PATH_CAPACITY];`, `size` in/out starts at
  the capacity. On `Ok`, take `&buffer[..(size as usize).min(IMAGE_PATH_CAPACITY)]` (no panic path). A path of 1024 units or more
  fails with `ERROR_INSUFFICIENT_BUFFER` and reads as `None`.
- **Why.** `lpdwSize`: "On input, specifies the size of the lpExeName buffer, in characters. On success, receives the number of
  characters written to the buffer, not including the null-terminating character" (Learn). 1024 is the probe's size and four
  times `MAX_PATH`. The documented maximum is "32,767 characters", and "approximate" (Learn, Maximum Path Length Limitation): a
  64 KiB zeroed frame per lookup on the hook thread is not worth it for paths no GUI executable has.
- **Rejected.** `MAX_PATH` (260): long-path installs exceed it. A heap retry on `ERROR_INSUFFICIENT_BUFFER`: an allocation and a
  second code path for a case not seen in practice.

### D5: One fold for the two equality conditions: `str::to_lowercase` with an ASCII fast path

- **What.** Private `fn equal_ignoring_case(a: &str, b: &str) -> bool` in `src/data/condition.rs`:
  - both ASCII: `a.eq_ignore_ascii_case(b)`;
  - otherwise: `a.to_lowercase() == b.to_lowercase()`.

  `AppEquals` (`:63`) and `TitleEquals` (`:71`) call it. `TitleContains` (`:67`) is unchanged.
- **Why.**
  - The spec says "matching TitleContains". The same mapping keeps `TitleEquals(s) => TitleContains(s)` true for every title,
    including the context-dependent Greek final sigma that `str::to_lowercase` applies.
  - The fast path is exact. For ASCII input `str::to_lowercase` lowercases byte by byte, so equality of the lowered strings is
    `eq_ignore_ascii_case`. A non-ASCII side takes the slow path even when the other side is ASCII (KELVIN SIGN U+212A lowercases
    to ASCII `k`). T3 pins both.
  - Exe names and most titles are ASCII, so the common case allocates nothing.
- **Rejected.**
  - Char-wise `a.chars().flat_map(char::to_lowercase).eq(..)`: allocation-free, but per char Σ becomes σ, so `"ΟΔΟΣ"` is unequal
    to `"οδος"` while `TitleContains("οδος")` holds. That breaks "matching TitleContains". T3 rejects it.
  - Unicode case folding (CaseFolding.txt, or the `unicase` crate): no roadmap item names a dependency, and it diverges from
    `TitleContains`.
  - `CompareStringOrdinal(.., TRUE)`, which compares the way NTFS does: a Win32 call inside `Condition::evaluate`, which the engine
    runs under scenario tests, and again a different mapping from `TitleContains`.
  - Fold the config side at load time: changes `Condition`'s data and its `Debug` text in the startup banner
    (`src/data/binding.rs:158-162`), and saves one allocation only on the non-ASCII path. That belongs to 3.4/3.5.
- **Cost.** A non-ASCII comparison allocates two `String`s per evaluated leaf, as `TitleContains` already does on every call.

### D6: Feature hygiene: drop `Win32_System_ProcessStatus`

- **What.** Remove `"Win32_System_ProcessStatus"` (`Cargo.toml:50`) in C3, together with its only user (`condition.rs:20`).
- **Why.** Dead after D1. Cargo records no features in `Cargo.lock`, so `--locked` is unaffected and the lock file does not change.
  `QueryFullProcessImageNameW` needs only `Win32_System_Threading` (present, `Cargo.toml:49`; ungated in
  `System/Threading/mod.rs:1257-1261`).

### D7: CHANGELOG classification and docs placement

- KS-03 goes under **Fixed**, not **Breaking**. README has documented a matching `AppEquals` since 0.1.0 (`README.md:465`). The
  entry spells out the one visible change for existing configs (`Not(AppEquals(..))` now turns a binding off in that app). It ends
  with the manual-check note (spec acceptance).
- KS-28 goes under **Fixed**, marked as an unverified report like KS-30/KS-32/KS-33 (`CHANGELOG.md:211-222`).
- README, template and CHANGELOG change in this lane, in the commit that changes the meaning. Book pages go to the phase book sync,
  as in A2 (Q2) and A3 (Q4). Q1 below.

## Threads, hooks, windows, timers

| Code path | Thread | Lock / constraint |
|---|---|---|
| `ForegroundContext::fetch_app` -> `process_image_name` -> `OpenProcess`, `QueryFullProcessImageNameW`, `CloseHandle` | main thread, inside `keyboard_hook_callback` -> `decide_event` -> `with_engine` -> `Engine::on_key_down` -> `Condition::evaluate` -> `EnvForeground::app` -> `HookEnv::foreground_app` (`keyboard_hook_callback.rs:178-181`, `engine/mod.rs:368-376`, `env.rs:60-63`, `keyboard_hook_callback.rs:134-136`) | engine `Mutex` held (pre-existing). None of the three calls sends or retrieves window messages, so none can re-enter the LL hook (`keyboard_hook_callback.rs:56-69`). At most once per event (`OnceCell`, `condition.rs:117`) |
| `equal_ignoring_case` | same | pure Rust |
| `hold-timer` threads | never evaluate conditions: `on_timer` takes no `Env` (`keyboard_hook_callback.rs:209`) | |
| IPC worker (`set_enabled` reset), WinEvent callback (`foreground_changed`) | create a `HookEnv` but only call `os_key_down` (`engine/mod.rs:337-340`, `:358-363`) | unchanged |
| T1-T5 | cargo test threads | read-only: own pid, no hook, no window, no injection |
| T6 (ignored) | test thread | `GetForegroundWindow` + the same reads |

No hook, window, timer or COM apartment is added, moved or owned differently. `LLKHF_INJECTED`, the extended-key flag and
`scanCode` do not matter: the lane reads no event field.

## Win32 API choices

| Purpose | `windows` path and signature (0.62.2) | Feature (present?) | Thread | Errors | Doc |
|---|---|---|---|---|---|
| Open the target (unchanged call) | `windows::Win32::System::Threading::OpenProcess(PROCESS_ACCESS_RIGHTS, bool, u32) -> windows::core::Result<HANDLE>` (`System/Threading/mod.rs:1192-1196`), `PROCESS_QUERY_LIMITED_INFORMATION` (`:2818`) | `Win32_System_Threading` (yes, `Cargo.toml:49`) | any; here the main thread | `Err` from `GetLastError` (process gone, access denied) -> `None` | <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess>; rights: <https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights> ("PROCESS_QUERY_LIMITED_INFORMATION (0x1000) Required to retrieve certain information about a process (see ... QueryFullProcessImageName)") |
| Image path | `windows::Win32::System::Threading::QueryFullProcessImageNameW(HANDLE, PROCESS_NAME_FORMAT, windows::core::PWSTR, *mut u32) -> windows::core::Result<()>` (`:1258-1261`), `PROCESS_NAME_WIN32` (`:2798`) | `Win32_System_Threading` (yes) | any; no message pump, no window | `BOOL` 0 -> `Err(GetLastError)`, e.g. `ERROR_INSUFFICIENT_BUFFER` (122) -> `None` | <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew> (handle-right and `lpdwSize` quotes in D1, D4) |
| Close | `windows::core::Owned<HANDLE>` -> `Free::free` -> `CloseHandle` (`Foundation/mod.rs:5297-5306`) | `Win32_Foundation` (yes, `:42`) | same as open | `CloseHandle` result ignored by `Free`; acceptable, nothing to recover | <https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle> |
| Window -> pid (unchanged) | `windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId` (`condition.rs:133`) | `Win32_UI_WindowsAndMessaging` (yes) | any | pid 0 -> `None` (`:134-136`) | <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid> |
| Removed | `windows::Win32::System::ProcessStatus::GetModuleBaseNameW` | `Win32_System_ProcessStatus` (removed, D6) | | "The handle must have the PROCESS_QUERY_INFORMATION and PROCESS_VM_READ access rights." | <https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getmodulebasenamew> |

**Elevation and UIPI.**

- The release daemon is elevated (`requireAdministrator`) and opens medium and high targets with the limited right. Protected
  processes allow it too (quote in D1).
- A non-elevated debug daemon reading an elevated foreground: the audit and the glossary say the limited right is granted "including
  against elevated processes" (`audit.md:194`, `glossary.md:307`). Not verified by this cut; see RK3.
- No message goes to the target, so UIPI does not apply. No child process (KS-09 n/a). No pipe change (KS-02 n/a).
- On the secure desktop `GetForegroundWindow` returns NULL and `fetch_app` returns `None` (`condition.rs:127-129`), unchanged.

## Touch set

| File | Change | Commit | Why |
|---|---|---|---|
| `src/data/condition.rs` | C1 extracts `process_image_name`. C2 adds T1, T2 and the test helper `test_exe_file_name()`. C3 switches to `QueryFullProcessImageNameW`, adds `image_file_name` and `IMAGE_PATH_CAPACITY`, uses `Owned<HANDLE>`, removes `GetModuleBaseNameW`, the guard, the `c_void` import and const and the false comment, and adds T5 and T6. C4 adds T3. C5 adds `equal_ignoring_case`, routes `AppEquals`/`TitleEquals` through it and rewrites the module doc `:11-13` and the variant docs `:32-42` | C1-C5 | D1-D5 |
| `src/engine/scenarios.rs` | T4 under `// ---- when: conditions ----` (after `:272`) | C4 | KS-28 through `Engine::on_key` |
| `Cargo.toml` | drop `"Win32_System_ProcessStatus",` (`:50`) | C3 | D6 |
| `README.md` | `:465` AppEquals row and a UWP sentence after the table (`:470`) in C3; `:439` in C5 | C3, C5 | config meaning |
| `src/config/default_config.ron` | `:139` wording | C5 | config meaning. C3 needs no template edit: `:142` ("exe base name of focused window") and the examples `:192-208` already describe the fixed behaviour |
| `CHANGELOG.md` | `### Fixed`: KS-03 entry after the `swch` entry (`:119-129`) and before KS-04 (`:130`) in C3; KS-28 entry after KS-18 (`:197-210`) and before KS-30 (`:211`) in C5 | C3, C5 | D7 |
| not touched | `src/engine/env.rs`, `src/engine/testing.rs`, `src/hook/keyboard_hook_callback.rs`, `src/config/*.rs`, `Cargo.lock`, `book/` | | spec: keep the abstraction; book via book sync |

## Commits, in order

Rules:

- Gate on every commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
- **C2 and C4 are the declared red commits.** fmt and clippy pass on them; exactly the named tests fail, with the texts below. Any
  other red test: stop and report.
- Every message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. A red commit quotes the
  failing lines in its body.
- Source stays ASCII: non-ASCII test strings are `\u{..}` escapes, and the upper-case variants are built with `to_uppercase()` at
  run time.
- Replay: the red tests compile on C1 (C2) and on C3 (C4), and those are their parents. On `master` T1/T2 do not compile
  (`cannot find function process_image_name`); replay them on C1.

### C1. `Conditions: look up the app name by process id` (green, refactor)

- D2's split on the **old** body. `process_image_name(pid)` still opens with the limited right and calls `GetModuleBaseNameW`.
  A length of 0 returns `Err(windows::core::Error::from_thread())`, evaluated before any local drops, so `GetLastError` is still
  the call's own error. `fetch_app` returns `process_image_name(pid).ok()`.
- Behaviour identical to master: every failure is still `None`. No new test. The false comment moves with the code.

### C2. `Tests: red-first: AppEquals resolves this test process (KS-03)` (RED)

Test helper in `condition.rs` tests: `fn test_exe_file_name() -> String`, the file name of `std::env::current_exe()` (rustc names
it `key_switch_rs-<hash>.exe`).

| Test (`src/data/condition.rs::tests`) | Script | PREDICTED RED on C1 |
|---|---|---|
| T1 `process_image_name_resolves_the_current_process` | `let name = process_image_name(std::process::id()).unwrap_or_else(\|e\| panic!("process_image_name(own pid) failed: {e}"));` then `assert!(name.eq_ignore_ascii_case(&test_exe_file_name()), "{name:?} is not the test binary's file name")`, `assert!(!name.contains('\\'))`, `assert!(name.to_ascii_lowercase().ends_with(".exe"))` | panic `process_image_name(own pid) failed: Access is denied. (0x80070005)`. The message text follows the system UI language; the code `0x80070005` (ERROR_ACCESS_DENIED) is fixed. The audit probe got error 5 on its own process (`methodology.md:200`, `:206`) |
| T2 `app_equals_matches_the_current_process` (roadmap A acceptance `roadmap.md:97`) | Local `struct OwnProcess(Option<String>)`: `ForegroundInfo` with `app()` = `self.0.as_deref()`, `title()` = `None`. `let own = OwnProcess(process_image_name(std::process::id()).ok()); let exe = test_exe_file_name();` 1. `assert!(Condition::AppEquals(exe.to_ascii_uppercase()).evaluate(&own), "AppEquals matches the test's own process (KS-03)")` 2. `And([AppEquals(exe), Always])` is true 3. `Not(AppEquals(exe))` is false 4. `AppEquals("notepad.exe")` is false | panic `AppEquals matches the test's own process (KS-03)`: `own.0` is `None`. The target is ASCII upper case, so master's ASCII fold is not what fails here |

### C3. `Conditions: read the app name with QueryFullProcessImageNameW (KS-03)` (green; T1, T2 green)

- D1, D3, D4, D6. `fn image_file_name(path: &[u16]) -> Option<&[u16]>`: the units after the last `0x005C`, or `None` if empty.
  `fetch_app`'s doc gains "or its image path has 1024 or more UTF-16 units".
- New comment at the call: why the limited right suffices (Learn quote, short).
- T5 `image_file_name_is_the_part_after_the_last_backslash` (pure, green on arrival; on C2 a compile error):

  | input | output |
  |---|---|
  | `C:\Windows\System32\notepad.exe` | `notepad.exe` |
  | `\Device\HarddiskVolume3\Apps\App.EXE` | `App.EXE` |
  | `notepad.exe` | `notepad.exe` |
  | `C:\dir\` | `None` |
  | empty | `None` |

- T6 `foreground_context_names_the_focused_notepad`,
  `#[ignore = "manual: start it, then focus a Notepad window within 5 s; it expects ForegroundContext::capture().app() to be notepad.exe in any case"]`:
  sleep 5 s, `ForegroundContext::capture()`, then
  `assert_eq!(ctx.app().map(str::to_ascii_lowercase).as_deref(), Some("notepad.exe"))`. It is read-only.
  Run: `cargo test --locked --lib -- --ignored foreground_context_names_the_focused_notepad`.
- README, at `:465`: `` | `AppEquals("<exe>")` | File name of the focused window's executable, with `.exe`, equals `<exe>` (case insensitive): `"code.exe"` matches `C:\...\Microsoft VS Code\Code.exe` | ``.
  After the table (`:470`): "Windows of UWP apps such as Calculator and Settings belong to `ApplicationFrameHost.exe`, so
  `AppEquals` cannot tell them apart; match them with `TitleContains`."
- CHANGELOG `### Fixed`, between the `swch` entry and KS-04:

  ```markdown
  - KS-03: `AppEquals` matches the foreground application. The daemon read
    the executable name with `GetModuleBaseNameW`, which fails on the
    limited process handle it opens, so every `AppEquals` was false,
    `Not(AppEquals(..))` was true even inside that application, and
    `And([AppEquals(..), ..])` never held. It now reads the image path with
    `QueryFullProcessImageNameW` and compares its file name: `notepad.exe`
    for `C:\Windows\System32\notepad.exe`. A binding with
    `when: Not(AppEquals("game.exe"))` now stays silent in that game.
    Windows of UWP apps such as Calculator and Settings belong to
    `ApplicationFrameHost.exe`; match those by title. Manual check (no
    automated test can focus a window): with
    `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )`
    loaded, F9 types `a` in Notepad and reaches every other window
    unchanged.
  ```

- Invariants: no panic path (no indexing past the clamp, no `unwrap`); one `unsafe` per Win32 call plus `Owned::new`, each with a
  concrete SAFETY line (`PWSTR` from `buffer.as_mut_ptr()`, live for the call, `size` equal to its capacity in characters); no new
  lock, thread or message pump on the hook path.

### C4. `Tests: red-first: TitleEquals and AppEquals ignore case beyond ASCII (KS-28)` (RED)

| Test | Script | PREDICTED RED on C3 |
|---|---|---|
| T3 `equals_conditions_fold_case_like_title_contains` (`src/data/condition.rs::tests`) | Cyrillic: `title = "\u{41a}\u{430}\u{43b}\u{44c}\u{43a}\u{443}\u{43b}\u{44f}\u{442}\u{43e}\u{440}"`, `app = "\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}.exe"`, `FakeForeground { app, title }`. 1. `assert_ne!(title.to_uppercase(), title)`. 2. `TitleContains(title.to_uppercase())` is true (the reference; green today). 3. `assert!(Condition::TitleEquals(title.to_uppercase()).evaluate(&cyrillic), "TitleEquals ignores Cyrillic case like TitleContains")`. 4. `AppEquals(app.to_uppercase())` is true. 5. `TitleEquals("\u{41a}\u{430}\u{43b}")` (a prefix) is false. Greek: title `"\u{39f}\u{394}\u{39f}\u{3a3}"`; `TitleEquals("\u{3bf}\u{3b4}\u{3bf}\u{3c2}")` (final sigma) and `TitleContains` of the same are both true. Kelvin: app `"\u{212a}.exe"`; `AppEquals("k.exe")` and `AppEquals("K.EXE")` are true | panic `TitleEquals ignores Cyrillic case like TitleContains` (step 3) |
| T4 `when_title_equals_and_app_equals_ignore_case_beyond_ascii` (`src/engine/scenarios.rs`, roadmap 3.5 acceptance `roadmap.md:385`) | Same Cyrillic `title`/`app` as T3. Config built at run time: `format!(r#"( bindings: [ (keys: "F13", action: SwitchLanguage, when: TitleEquals("{}")), (keys: "F14", action: ToggleCapsLock, when: AppEquals("{}")) ] )"#, title.to_uppercase(), app.to_uppercase())`. A: `.foreground(app, title)`; down F13 -> suppressed, fired `[SwitchLanguage]`; up F13 -> suppressed; down F14 -> suppressed, fired `[ToggleCapsLock]`; up F14 -> suppressed. B: `.foreground(app, "\u{41a}\u{430}\u{43b}")`; down F13 -> passed, nothing fired; up F13 -> passed | `scenario: down F13 at +0 ms: expected the event to be suppressed, it was passed` (format: `testing.rs:461-462`, `:549-555`, `:585`) |

### C5. `Conditions: TitleEquals and AppEquals fold case like TitleContains (KS-28)` (green; T3, T4 green)

- D5. Module doc `:11-13`: "Every string comparison ignores case with Unicode lowercasing (`str::to_lowercase`), so Cyrillic or
  Greek titles match in any case. Windows treats ...". Variant docs `:32-42` say "case-insensitive, Unicode-aware".
- README `:439`: "All string comparisons ignore case in every alphabet: `TitleEquals("КАЛЬКУЛЯТОР")` matches a window titled
  "Калькулятор"."
- Template `:139`: `// unconditionally. Variants (string comparisons ignore case, in every alphabet):` (ASCII).
- CHANGELOG `### Fixed`, after KS-18 and before KS-30:

  ```markdown
  - KS-28 (an unverified report): `TitleEquals` and `AppEquals` ignore case
    in every alphabet, with the same Unicode lowercasing as `TitleContains`:
    `TitleEquals("КАЛЬКУЛЯТОР")` matches a window titled "Калькулятор". They
    used to ignore case for ASCII letters only.
  ```

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| Spec: replace `GetModuleBaseNameW` on the limited handle with `QueryFullProcessImageNameW`, take the file name | C3; T1, T5; `rg "GetModuleBaseNameW\|ProcessStatus" src Cargo.toml` finds nothing |
| Spec: close handles with RAII | C3 `Owned<HANDLE>` (D3); `rg "ProcessHandleGuard\|CloseHandle" src/data/condition.rs` finds nothing; reviewer R9 |
| Spec: keep the Env/ForegroundInfo abstraction (real implementation uses it, engine tests use a fake) | `env.rs`, `testing.rs` untouched; `HookEnv` -> `ForegroundContext` -> `process_image_name`; T4 and `scenarios.rs:208-272`, `:1851-1878` run on `FakeEnv` |
| Spec: a non-ignored test resolves the current process id to its exe file name with the real function | T1 |
| Roadmap A: "`AppEquals` matches the test's own process" (`roadmap.md:97`) | T2 |
| Spec: AppEquals and TitleEquals case-insensitive with Unicode folding, matching TitleContains | T3 (incl. final sigma and Kelvin), T4 |
| Roadmap 3.5 folding line: `TitleEquals("КАЛЬКУЛЯТОР")` matches "Калькулятор" (`roadmap.md:385`) | T4 (scenario A), T3 step 3 |
| Spec: tests green | gate on C5 |
| Spec: a manual-check note in CHANGELOG | C3 KS-03 entry, last sentence; M1 |
| Smoke checklist 7: `when: AppEquals("notepad.exe")` fires only in Notepad (`roadmap.md:235`) | M1 |

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Release build, daemon started with `swch open`; config `( bindings: [ ( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") ) ] )` (`methodology.md:530`) | Focus Notepad, press F9 three times; focus a browser address bar, press F9 | `aaa` in Notepad; in the browser F9 does what it does without the daemon, and no `a` appears | none (needs the daemon) |
| M2 | Same daemon; binding `( keys: "F9", action: PressKey("B"), when: Not(AppEquals("notepad.exe")) )` | Press F9 in Notepad, then in the browser address bar | Nothing typed in Notepad; `b` in the browser | none |
| M3 | Same daemon; binding `( keys: "F9", action: ToggleCapsLock, when: AppEquals("taskmgr.exe") )`; Task Manager open (elevated for an admin) | Focus Task Manager, press F9; focus Notepad, press F9 | CapsLock LED toggles in Task Manager only | none |
| M4 | Non-elevated console, no daemon | `cargo test --locked --lib -- --ignored foreground_context_names_the_focused_notepad`, focus Notepad within 5 s | test passes | T6 |

## Risks and edge cases

| Id | Risk | Handling |
|---|---|---|
| RK1 | Hot path: the conditions run before matching (`engine/mod.rs:373-378`), so with any `AppEquals` binding every key-down now performs a successful lookup (3 kernel calls + 1 `String`) under the engine lock | accepted; roadmap 3.5 "Why" anticipates exactly this (`roadmap.md:381`) and adds the per-HWND cache and the reordering |
| RK2 | UWP frames report `ApplicationFrameHost.exe` | README + CHANGELOG say so; `TitleContains` is the advice (`config/triggers-conditions.md:193`) |
| RK3 | Non-elevated debug daemon with an elevated foreground: relies on the audit's claim that the limited right is granted against elevated targets | release daemon is elevated, so the shipped setup does not depend on it; Q5 |
| RK4 | Cost of the Win32-format conversion inside `QueryFullProcessImageNameW` is undocumented (unverified) | bounded by RK1's 3.5 cache to one query per foreground change |
| RK5 | Image path of 1024+ units, or a file name with unpaired surrogates (NTFS names are opaque WCHAR sequences, Learn) | `None`, or U+FFFD that no RON string matches; documented in `fetch_app` |
| RK6 | Pid reuse between `GetWindowThreadProcessId` and `OpenProcess` | wrong name for one key-down at most; negligible |
| RK7 | Unicode lowercase is not NTFS's upcase table (e.g. U+1E9E) and is locale-independent (Turkish dotless i) | same as `TitleContains`; spec says "matching TitleContains" |
| RK8 | Existing configs: `Not(AppEquals(x))` now false inside `x`; `Or([AppEquals(x), C])` now true in `x` | intended; CHANGELOG states it (D7) |
| RK9 | KS-35 becomes observable (blocker ignores `when:`) | follow-up, roadmap 1.2 |
| RK10 | Reload, `swch off`/`on`, foreground change, or shutdown during a pending gesture | unchanged: conditions are read only at key-down (`engine/mod.rs:376`), reset and focus change never ask for the foreground, and no shutdown path calls the lookup |

**R1-R10 (code-reviewer areas).**

- **R1.** The adapter and the engine are unchanged. The lookup has no panic path (the clamp in D4, `ok()` in `fetch_app`). Every
  event still ends in exactly one outcome.
- **R2.** No `SendInput` path.
- **R3.** No shared state. `ForegroundContext` stays per event on the stack (`keyboard_hook_callback.rs:173`). No lock, `Cell`
  or static.
- **R4.** No atomics.
- **R5.** No builder change.
- **R6.** No new variant, so no `Display` arm.
- **R7.** No change to binding order.
- **R8.** No key-identity change.
- **R9.** Process reads now use a right that suffices (D1); RAII per D3; read-only.
- **R10.** The message loop and shutdown are untouched; no new thread.

## Docs impact

- **This lane (developer):** `README.md:439`, `:465`, the sentence after `:470`; `src/config/default_config.ron:139`; `CHANGELOG.md`
  Fixed KS-03 (with the manual-check note) and KS-28; the doc comments in `condition.rs`.
- **Phase book sync (doc-writer), not this lane:**
  - `audit.md` status lines of KS-03 (`:185-188`) and KS-28 (`:584-587`): "Fixed in <sha> (fix/app-equals)".
  - `architecture.md:362-365`.
  - `config/triggers-conditions.md:140`, `:147-148`, §12.3 `:174-193` (keep the UWP advice).
  - `config/known-issues.md` rows 44, 45, 48, 50 (`:94`, `:95`, `:104`, `:106`).
  - `config/structure.md:55-57`, `:145`; `config/example.md:6`, `:151`, `:232`, `:240`; `config/README.md:74`.
  - `overview.md:21`, `:40`, `:189`, `:195`; `glossary.md:307`; `gaps.md:195`, `:258`.
  - `roadmap.md:78`, `:97`, `:330` (3.5 issues column), `:379`, `:385`.
  - Every changed RON example validated with `cfgcheck`.

## Questions for the orchestrator

1. Book pages go to the phase book sync (list above), as in A2 and A3. Recommendation: yes. This lane updates README, the
   template and CHANGELOG. Not blocking.
2. Rule texts after the merge: `.claude/agents/code-reviewer.md:58` and `:128` (R9) still describe `GetModuleBaseNameW`.
   Recommendation: after the merge the orchestrator changes R9 to "Process names come from `QueryFullProcessImageNameW` on a
   `PROCESS_QUERY_LIMITED_INFORMATION` handle held by `Owned<HANDLE>` (KS-03 fixed)". Not blocking.
3. Roadmap 3.5 keeps KS-28 in its issues column. Recommendation: the book sync marks the folding as done in Workstream A, so 3.5
   keeps the cache, the reordering and the budget. Not blocking.
4. KS-03 under Fixed, not Breaking, although `Not(AppEquals(..))` configs change (RK8). Recommendation: Fixed. The documented
   behaviour has been "matches" since 0.1.0, and the entry names the visible change. Not blocking.
5. RK3, the non-elevated debug daemon against an elevated foreground, is not in the manual checks. Recommendation: accept, because
   the shipped daemon is elevated. If the owner wants it verified, M3 with a debug daemon from a normal console decides it. Not
   blocking.

## Cut readiness checklist

- Spec
  - [x] KS-03 (`audit.md:184-196`) and KS-28 (`:584-590`) read. KS-03's scenarios are covered: Spotify/notepad example (M1),
    `Not(AppEquals(..))` (T2 step 3, M2), `And([AppEquals..])` (T2 step 2). KS-28's TitleEquals Cyrillic case is T3/T4, and its
    non-ASCII exe names are T3 step 4
  - [x] Every book reference re-located (table); none gone
  - [x] Goal, non-goals and dependent items (3.5, 3.4, 1.2, book sync) stated
- Design
  - [x] Every decision has a rationale and rejected alternatives (D1-D7)
  - [x] Owner decisions respected: modifiers, repeat, RON, licence untouched; logging n/a (no new log line)
  - [x] Thread named for every code path (thread table); the LL callback only adds the prescribed reads: `OpenProcess`, `QueryFullProcessImageNameW`, `CloseHandle`, one `String`, and two `String`s for a non-ASCII fold
  - [x] N/A install/uninstall/timers: none added or moved
  - [x] N/A `SendInput`: none added
  - [x] N/A sided modifiers: no key handling changes
  - [x] Reload, `swch off`/`on`, foreground change, shutdown: RK10
  - [x] N/A atomics: no new shared state; no `static mut`
  - [x] No new module cycle (`data/condition.rs` gains only `windows` imports); folding logic is pure and reached by scenario T4
- Win32
  - [x] Each API: path, feature check against `Cargo.toml:41-51`, thread, errors, doc link
  - [x] Elevation, UIPI, pipe DACL, token inheritance considered (Elevation and UIPI)
- Commits and tests
  - [x] Red-first: C2 before C3, C4 before C5, exact predicted reds
  - [x] T3/T4 run on fakes (T4 through `Engine::on_key`, `FakeEnv`, fake clock); T1/T2/T5 read only the own process; nothing installs a hook or injects input
  - [x] T6 is `#[ignore = "manual: ..."]` and listed as M4
  - [x] Every commit passes the gate except the declared red C2 and C4
  - [x] Bisectable order; README, template and CHANGELOG ride in C3 and C5, which change the meaning
- Acceptance
  - [x] Every spec and roadmap line mapped
  - [x] Every question has a recommendation

## Critique response (developer addendum)

Branch `fix/app-equals`, commits `276caf1..32703a8`. Every remark of `critique.md` round 1 is resolved; none is refuted.

- **I1 (Important) - RESOLVED (docs, option a).** The KS-03 entry (`CHANGELOG.md:130-148`) now states that the auto-blocker
  of a `block_original_combo` binding still ignores the binding's `when:` and acts in every window until roadmap item 1.2
  (KS-35), and spells out the consequence: with `when: Not(AppEquals("game.exe"))` added to the default config's CapsLock
  binding, CapsLock reaches the game as CapsLock while the Windows hotkey stays blocked there, so neither switches the
  layout in the game. The same sentence is in the README `block_original_combo` row (`README.md:430`) and, short, in the
  template's flag comment (`src/config/default_config.ron:131-132`). D7's "one visible change" is corrected: there are
  two, (1) `Not(AppEquals(x))` / `Or([AppEquals(x), ..])` change meaning inside `x`, and (2) the KS-35 interaction above.
  Checked with the real loader (`key_switch_rs::config::load`): that binding loads as
  `CapsLock -> switch language [Not(AppEquals("game.exe"))]` plus `[AUTO-BLOCK] Alt + Shift -> (blocked)`, the blocker
  without the `when:`.
- **M1 (Minor) - RESOLVED, deviation from D1: `PROCESS_NAME_NATIVE`.** Measured on this machine (critique question 2),
  release build, read-only, 20000 lookups per case of `OpenProcess` + `QueryFullProcessImageNameW` + close, on the own
  process (D: volume) and `explorer.exe` (C:), two rounds:

  | Format | p50 | p99 | max |
  |---|---|---|---|
  | `PROCESS_NAME_WIN32` (flag 0) | 19.9-24.2 us | 56-104 us | 0.32-0.85 ms |
  | `PROCESS_NAME_NATIVE` (flag 1) | 2.2-2.5 us | 4.7-7.2 us | 28-86 us |

  The Win32 form costs ten times as much and its p99 reaches roadmap 3.5's 100 us budget alone; the final path component
  is the same in both forms (`...\deps\zz_timing_probe-<hash>.exe`, `explorer.exe`). The code uses the native form
  (`src/data/condition.rs:197-213`, comment states why). T1 verifies it on this machine, T5 covers both path shapes.
- **M2 (Minor) - RESOLVED with a test.** T7 `foreground_context_reads_a_window_of_this_process`
  (`src/data/condition.rs:441-455`) builds a `ForegroundContext` on a message-only `STATIC` window of the test thread
  (`MessageOnlyWindow`, `:393-435`: never shown or activated, gets no broadcasts, destroyed by `Drop` on the creating
  thread; no hook, no input, no foreground change) and asserts `app()` is the test binary's file name,
  `AppEquals(exe)` holds through the `ForegroundInfo` impl the hook uses, and `title()` is the window's text. Red on C1
  (`ForegroundContext::app() of a window of this process is None, not "key_switch_rs-<hash>.exe" (KS-03)`), green on C3.
  A `fetch_app -> None` (or `fetch_title -> None`) mutant now fails it. This takes D2's rejected alternative for the
  wiring only; T1/T2 stay pid-level as cut.
- **M3 (Minor) - RESOLVED.** The CHANGELOG manual-check note says "F9 presses the A key in Notepad (it types that key's
  letter in the active layout)" (`CHANGELOG.md:145-148`). Manual checks M1/M2 in `impl.md` are phrased the same way.
- **M4 (Minor) - RESOLVED.** No comment in `src/data/condition.rs` names `CloseHandle` or `GetModuleBaseNameW` (the KS-03
  comment says "Reading the module list needs PROCESS_VM_READ"). At HEAD `rg -n "ProcessHandleGuard|CloseHandle"
  src/data/condition.rs` and `rg -n "GetModuleBaseNameW|ProcessStatus" src Cargo.toml` both find nothing (exit 1). A
  scoped form for later edits: `rg -n "ProcessHandleGuard|CloseHandle\(" src/data/condition.rs`.
