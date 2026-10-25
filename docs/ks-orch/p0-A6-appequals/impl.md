# Implementation: p0-A6-appequals round 1

Branch `fix/app-equals` from `master` @ `a1b5a27`, HEAD `32703a8`, tree clean. Cut: `cut.md` (CUT READY), critique:
`critique.md` round 1 (0 Critical, 1 Important, 4 Minor). Developer addendum appended to `cut.md` ("Critique response").

## Items

- C1 refactor - `src/data/condition.rs` `fetch_app` keeps HWND -> pid, calls private `process_image_name(pid) ->
  windows::core::Result<String>` (old `GetModuleBaseNameW` body; length 0 -> `Error::from_thread()`). No behaviour change;
  condition tests 7/7 green.
- KS-03 - `src/data/condition.rs:139-154` (`fetch_app` + doc), `:180-227` (`IMAGE_PATH_CAPACITY`, `process_image_name`
  with `Owned<HANDLE>` and `QueryFullProcessImageNameW(.., PROCESS_NAME_NATIVE, ..)`, `image_file_name`); guard, `c_void`
  import and dead const removed; `Cargo.toml` drops `Win32_System_ProcessStatus` (Cargo.lock unchanged).
  - red-first `process_image_name_resolves_the_current_process` (T1): red on C1 "process_image_name(own pid) failed:
    Access is denied. (0x80070005)" -> green on C3.
  - red-first `app_equals_matches_the_current_process` (T2): red on C1 "AppEquals matches the test's own process (KS-03)"
    -> green on C3.
  - red-first `foreground_context_reads_a_window_of_this_process` (T7, critique M2): red on C1
    "ForegroundContext::app() of a window of this process is None, not "key_switch_rs-9c8e127322721cde.exe" (KS-03)" ->
    green on C3 (app, `AppEquals` through `ForegroundInfo for ForegroundContext`, and title).
  - `image_file_name_is_the_part_after_the_last_backslash` (T5): pure, green on arrival (C3), 5 rows of the cut.
  - `foreground_context_names_the_focused_notepad` (T6): `#[ignore = "manual: ..."]`, see Manual checks M4.
- KS-28 (folding part) - `src/data/condition.rs:61-92` (`AppEquals`/`TitleEquals` -> `equal_ignoring_case`: ASCII fast path
  `eq_ignore_ascii_case`, else `to_lowercase() ==`), module doc `:11-14`, variant docs `:34-44`.
  - red-first `equals_conditions_fold_case_like_title_contains` (T3, `src/data/condition.rs:459-497`): red on C3
    "TitleEquals ignores Cyrillic case like TitleContains" -> green on C5 (Cyrillic, Greek final sigma, Kelvin sign).
  - red-first `engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii` (T4,
    `src/engine/scenarios.rs:274-309`): red on C3 "scenario: down F13 at +0 ms: expected the event to be suppressed, it
    was passed" -> green on C5.

All four red-firsts failed with exactly the predicted text and no other test failed.

## Remarks

- I1 - RESOLVED at `CHANGELOG.md:137-144`, `README.md:430`, `src/config/default_config.ron:131-132` (auto-blocker ignores
  `when:` until roadmap 1.2 / KS-35, `Not(AppEquals("game.exe"))` consequence spelled out); D7 corrected in the addendum.
  Pinned by the real loader: that binding loads with an `[AUTO-BLOCK] Alt + Shift` blocker that has no `when:`.
- M1 - RESOLVED at `src/data/condition.rs:197-213`: `PROCESS_NAME_NATIVE`. Measured (release, read-only, 20000 lookups,
  own process + explorer.exe, 2 rounds): WIN32 p50 19.9-24.2 us / p99 56-104 us / max 0.32-0.85 ms; NATIVE p50 2.2-2.5 us
  / p99 4.7-7.2 us / max 28-86 us. Same final component. Pinned by T1 (machine) and T5 (both path shapes).
- M2 - RESOLVED at `src/data/condition.rs:390-455`, pinned by T7 (kills a `fetch_app -> None` and a `fetch_title -> None`
  mutant).
- M3 - RESOLVED at `CHANGELOG.md:145-148` ("presses the A key ... types that key's letter in the active layout"); manual
  checks below phrased the same way.
- M4 - RESOLVED: no comment names `CloseHandle` or `GetModuleBaseNameW`; both acceptance greps exit 1 at HEAD
  (`rg -n "ProcessHandleGuard|CloseHandle" src/data/condition.rs`, `rg -n "GetModuleBaseNameW|ProcessStatus" src Cargo.toml`).

## Deviations from the cut

- D1: `PROCESS_NAME_NATIVE` instead of `PROCESS_NAME_WIN32` (M1, numbers above). Alternative kept by the cut: flag 0,
  ten times slower on the hook thread with no gain for the file name.
- D2: T7 adds a message-only window test (the alternative D2 rejected) to cover the `fetch_app` wiring (M2). It lives in
  C2 with T1/T2, so it is red on C1 too. Two `unsafe` blocks in test code (create, destroy), each with SAFETY.
- C3 docs: README `block_original_combo` row and template flag comment gain the KS-35 sentence (I1; the cut said C3
  needs no template edit). The CHANGELOG KS-03 text differs from the cut's verbatim block by the KS-35 sentences (I1),
  the layout-independent manual check (M3) and "compares the executable's file name".
- The KS-03 code comment says "Reading the module list" rather than naming the removed function (M4).
- Per-commit gate: fmt --check, clippy -D warnings and the touched test modules, per the brief; the cut's per-commit
  full `cargo test --locked` is left to the tester (one owner per heavy gate).

## Unsafe / atomics added

- `src/data/condition.rs:191-194` - SAFETY: "OpenProcess takes no pointers. In `Ok` it returns a valid handle that
  nothing else owns or closes, so `Owned` may close it."
- `src/data/condition.rs:203-213` - SAFETY: "`*process` is a live handle with the access right the call needs. The PWSTR
  points at `buffer`, which outlives the call, and `size` holds its capacity in UTF-16 units, so the call writes within
  it."
- `src/data/condition.rs:403-424` (test) - SAFETY: "STATIC is a system class that user32 registers in every process;
  `title` is a NUL-terminated UTF-16 buffer that lives until the call returns; HWND_MESSAGE as the parent makes the
  window message-only, so it is never shown or activated."
- `src/data/condition.rs:430-433` (test) - SAFETY: "self.0 was created in `create` and nothing else destroys it. HWND
  wraps a raw pointer, so this type is not Send and the drop runs on the creating thread, as DestroyWindow requires."
- Removed: the guard's `unsafe` close and the `GetModuleBaseNameW` block. No atomics, locks, threads or statics.

## Gates

- fmt --check / check --all-targets --locked / clippy --all-targets --locked -D warnings: pass on every commit (C2, C4
  are red only in their declared tests).
- tests run at HEAD: `cargo test --locked --lib -- data::condition engine::scenarios config::loader`: 172 passed, 0 failed,
  1 ignored (T6). Includes the loader's README combo-section and template tests. Full suite left to the tester.
- Red-first replay: T1/T2/T7 on C1 (`276caf1`), T3/T4 on C3 (`de03573`); on `master` T1/T2/T7 do not compile
  (`process_image_name` missing).
- RON examples of CHANGELOG/README/manual checks (5 configs) loaded with `key_switch_rs::config::load` from a temporary,
  uncommitted integration test (deleted); all load. The M1 timing probe was a temporary, uncommitted test too.
- manual ignores added: T6 `foreground_context_names_the_focused_notepad` (M4 below).

## Manual checks (owner)

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | Release build, daemon via `swch open`; `( bindings: [ ( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") ) ] )` | Focus Notepad, F9 three times; focus a browser address bar, F9 | Notepad gets three A key presses (the A key's letter in the active layout); in the browser F9 acts as without the daemon, no A key press |
| M2 | Same daemon; `( keys: "F9", action: PressKey("B"), when: Not(AppEquals("notepad.exe")) )` | F9 in Notepad, then in the browser address bar | Nothing in Notepad; the B key's letter in the browser |
| M3 | Same daemon; `( keys: "F9", action: ToggleCapsLock, when: AppEquals("taskmgr.exe") )`, Task Manager open | F9 in Task Manager, then in Notepad | CapsLock LED toggles in Task Manager only |
| M4 | Non-elevated console, no daemon | `cargo test --locked --lib -- --ignored foreground_context_names_the_focused_notepad`, focus Notepad within 5 s | test passes |
| M5 (optional, I1) | Same daemon; `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: Not(AppEquals("notepad.exe")) )`, Windows hotkey Alt+Shift | CapsLock and Alt+Shift in Notepad, then in another window | In Notepad CapsLock toggles caps lock and Alt+Shift does not switch the layout (KS-35, roadmap 1.2); elsewhere CapsLock switches the layout |

## Docs

- `CHANGELOG.md` Unreleased / Fixed: KS-03 (`:130-148`, with the manual-check note and the KS-35 interaction), KS-28
  (`:230-233`).
- `README.md`: `:430` (KS-35 sentence), `:439-440` (folding, Cyrillic example), `:466` (AppEquals row), `:472-474` (UWP
  note).
- `src/config/default_config.ron`: `:131-132` (KS-35), `:141` (folding wording).
- Book: none (phase book sync, cut Q1; list in cut "Docs impact").

## Commits

- `276caf1` Conditions: look up the app name by process id
- `9b5c2db` Tests: red-first: AppEquals resolves this test process (KS-03)  (RED: T1, T2, T7)
- `de03573` Conditions: read the app name with QueryFullProcessImageNameW (KS-03)
- `cd1628d` Tests: red-first: TitleEquals and AppEquals ignore case beyond ASCII (KS-28)  (RED: T3, T4)
- `32703a8` Conditions: TitleEquals and AppEquals fold case like TitleContains (KS-28)

## Open points / pre-existing warnings

- No pre-existing warnings seen in fmt, check or clippy.
- `.claude/agents/code-reviewer.md:58,128` (R9) still describe `GetModuleBaseNameW`: orchestrator, after the merge (cut
  Q2). R9 should now read `QueryFullProcessImageNameW(.., PROCESS_NAME_NATIVE, ..)` on a limited handle held by
  `Owned<HANDLE>`.
- Book sync: the cut's list, plus the KS-35 interaction (`config/*.md` block_original_combo text) and the native path
  form wherever the book quotes the call.
- KS-35 is now observable (roadmap 1.2). RK3 (non-elevated daemon vs elevated foreground) unverified, as accepted.
- `fetch_title`'s 512-unit buffer (titles over 511 units never satisfy `TitleEquals`): follow-up note, untouched.
- rustfmt rewrote `src/data/condition.rs` with LF in the working copy; git (autocrlf) stores LF as before and the
  status is clean. No content effect.
