# Results analysis: p0-A6-appequals (Workstream A, AppEquals part: KS-03, plus the folding part of KS-28)

Branch `fix/app-equals` @ `32703a8`. It was cut from `master` @ `a1b5a27`, which is still master HEAD and the merge-base. It has
5 commits and touches 6 files (+351/-58). The tree is clean, `git worktree list` shows only the lane tree, no remote branch
contains HEAD, and the tester's `wt/` and `target-wt/` scratch directories are empty.

## VERDICT: ACCEPT

**Summary**: Every spec bullet and the roadmap A acceptance line ("`AppEquals` matches the test's own process") is pinned by a
non-ignored test that runs the real Win32 lookup. That includes the `ForegroundContext` type the hook uses, through a message-only
window. The folding part is pinned by a `Condition` unit test and an `Engine::on_key` scenario. I replayed both red commits myself:
`9b5c2db` fails exactly T1, T2 and T7 with `Access is denied. (0x80070005)`, and `cd1628d` fails exactly T3 and T4. All pass on
HEAD, and the gate is green in my own run (344 passed, 1 manual ignore; master 338). The engine, `Env`, `FakeEnv` and the hook
adapter are untouched. The lane does add work to the hook path: a now-successful process query plus one `String` per key-down
that evaluates an `AppEquals`. The cut prescribes that cost (RK1), roadmap 3.5 anticipates it word for word, and the developer cut
it tenfold by reading the native path form. What remains for the orchestrator is listed under "Notes for later lanes": two
optional Minor items, the book sync list (three lines more than the cut's), a CHANGELOG classification ruling and owner FYIs.

## Quality gate (own run, lane tree `D:/key-switch-rs/key-switch-rs` @ `32703a8`)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0, no warnings; cargo replayed the cached result of the tester's/developer's identical build) |
| `cargo test --locked` | **344 passed, 0 failed, 1 ignored**: lib `338 passed; 0 failed; 1 ignored`, `tests/ipc_roundtrip.rs` `6 passed`, swch and doc-tests 0. Log: `D:/tmp/ks-orch/p0-A6-appequals/analyst-test.txt`. **master: 338** (HEAD lists 345; the lane adds 7 `#[test]`, `git grep -c '#\[test\]'` 329 -> 336; no test line is removed in `git diff master...fix/app-equals`) |
| `cargo test --locked -- --list --ignored` | `data::condition::tests::foreground_context_names_the_focused_notepad` only (new, `manual:` reason; master had none). Not run |
| `cargo doc --no-deps` | 2 warnings, both pre-existing and outside the diff (`src/data/vk_name.rs:4` `<raw>`, `src/hook/keyboard_hook.rs:65,132` private link) |
| `mdbook build` | not run: `book/` is unchanged on the branch |
| Red-first replay (own, `git archive` exports in my scratchpad, own `CARGO_TARGET_DIR`) | `9b5c2db -- data::condition`: `7 passed; 3 failed` = T1 `process_image_name(own pid) failed: Access is denied. (0x80070005)`, T2 `AppEquals matches the test's own process (KS-03)`, T7 `ForegroundContext::app() of a window of this process is None, not "key_switch_rs-9c8e127322721cde.exe" (KS-03)`. `cd1628d`: `0 passed; 2 failed` = T3 `TitleEquals ignores Cyrillic case like TitleContains`, T4 `scenario: down F13 at +0 ms: expected the event to be suppressed, it was passed` |
| Tests unchanged since their red commit | `diff` of the `mod tests` section `9b5c2db` -> HEAD shows only additions after T7; `cd1628d` -> HEAD: no difference in either test file |
| Trailers | 5/5 commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; both red commits quote their failing lines |
| Non-ASCII added under `src/` or in `Cargo.toml` | none (`grep -P '[^\x00-\x7F]'` on the added lines). Test strings use `\u{..}` escapes |

## Acceptance criteria

| # | Criterion | Evidence | Status |
|---|---|---|---|
| 1 | Spec: replace `GetModuleBaseNameW` on the LIMITED handle with `QueryFullProcessImageNameW` and take the file name | `process_image_name` `src/data/condition.rs:184-219` (`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` `:193-194`, `QueryFullProcessImageNameW(.., PROCESS_NAME_NATIVE, ..)` `:207-214`), `image_file_name` `:221-227`. T1 `process_image_name_resolves_the_current_process` `:347-360`, T5 `image_file_name_is_the_part_after_the_last_backslash` `:499-516` (Win32 and `\Device\...` shapes, trailing `\`, empty). `rg GetModuleBaseNameW\|ProcessStatus src Cargo.toml`: no hit. Tester mutations M1 (split on `/`) and M4 (`fetch_app -> None`) are caught | PASS |
| 2 | Spec: close handles with RAII | `Owned<HANDLE>` `:193-194`. In windows 0.62.2, `Free for HANDLE` closes only when `!is_invalid()`, and `OpenProcess` never returns an invalid handle in `Ok` (cut D3, confirmed by the critic and the reviewer against the crate sources). Both `?` paths (`:213`, `:217`) drop the guard. The hand-written guard, the `c_void` placeholder and the `Win32_System_ProcessStatus` feature are gone | PASS (code read; a leak is not unit-testable) |
| 3 | Spec: keep the `Env`/`ForegroundInfo` abstraction: the real implementation uses it, engine tests use a fake | `git diff --stat` touches no `src/engine/env.rs`, `src/engine/testing.rs` or `src/hook/*`. Chain: `HookEnv::foreground_app` (`keyboard_hook_callback.rs:134-136`) -> `ForegroundContext::app` (`condition.rs:129-131`) -> `fetch_app` (`:142-154`) -> `process_image_name`. T4 runs on `FakeEnv` via `Scenario::foreground` | PASS |
| 4 | Spec: a non-ignored test resolves the CURRENT process id to its exe file name with the real function | T1 `:347-360`: `process_image_name(std::process::id())` equals `current_exe().file_name()`, has no `\`, ends in `.exe`. Red on `9b5c2db` (own replay) | PASS |
| 5 | Roadmap A acceptance (`roadmap.md:97`): "`AppEquals` matches the test's own process" | T2 `app_equals_matches_the_current_process` `:362-388` (`AppEquals` upper case true, `And([AppEquals, Always])` true, `Not(AppEquals)` false, `AppEquals("notepad.exe")` false). T7 `foreground_context_reads_a_window_of_this_process` `:440-456`: the real `ForegroundContext` on a message-only `STATIC` window of the test thread gives the exe name, `AppEquals` holds, and the title is read. Both red on `9b5c2db` | PASS |
| 6 | Spec: `AppEquals` and `TitleEquals` case-insensitive with Unicode-aware folding, matching `TitleContains` | `equal_ignoring_case` `:83-92` (ASCII fast path, else `to_lowercase() ==`, the mapping `TitleContains` uses at `:69`), used at `:65`, `:73`. T3 `equals_conditions_fold_case_like_title_contains` `:458-497` (Cyrillic title and exe, prefix negative, Greek final sigma against `TitleContains`, Kelvin sign). T4 `engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii` `src/engine/scenarios.rs:274-309` through `Engine::on_key`: down and up suppressed, the matching action fired, and the negative passes both ways. Tester mutations M2, M3 and M5 are caught | PASS |
| 7 | Roadmap 3.5 folding line (`roadmap.md:385`): `TitleEquals("КАЛЬКУЛЯТОР")` matches "Калькулятор" | T4 scenario A (the config names the title in upper case, the foreground has it in mixed case), T3 step 3 | PASS (the rest of 3.5, the cache and the 100 us budget, stays in 3.5) |
| 8 | Spec acceptance: tests green | Own run: 344 / 0 / 1 ignored | PASS |
| 9 | Spec acceptance: a manual-check note in CHANGELOG | `CHANGELOG.md:145-148`: "Manual check (no automated test can focus a window): with `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )` loaded, F9 presses the A key in Notepad (...) and reaches every other window unchanged." The wording works in any keyboard layout (critique M3) | PASS |
| 10 | Smoke checklist line 7 (`roadmap.md:235`): `when: AppEquals("notepad.exe")` fires only in Notepad | Manual: impl.md M1 (same binding) plus M2 (`Not(..)`), M3 (Task Manager), M4 = ignored T6 `foreground_context_names_the_focused_notepad` `condition.rs:518-527` (`manual:` reason tells a human to focus Notepad within 5 s) | MANUAL (owner) |

## Issues claimed closed

| KS-ID | Regression test (audit scenario encoded) | Fails on old code? | Status |
|---|---|---|---|
| KS-03 | T1 (own pid -> exe name with the real call), T2 (the audit's three symptoms against the real process: `AppEquals` false, `Not(AppEquals)` true, `And([AppEquals..])` false), T7 (the `ForegroundContext` path the hook uses) | Yes. Own replay on `9b5c2db`, which is master's `GetModuleBaseNameW` body behind the C1 split (I verified `git diff master 276caf1` is a pure refactor: `.ok()?` became `?`, then `.ok()` in `fetch_app`). On master itself T1/T2/T7 do not compile (no `process_image_name`), as the cut declared | CLOSED. The end-to-end daemon check is manual (smoke 7) |
| KS-28 (unverified report; the audit entry is only about folding) | T3, T4: the audit's `TitleEquals("калькулятор")` vs "Калькулятор" (as upper vs mixed case) and its "non-ASCII executable names fail `AppEquals`" | Yes. Own replay on `cd1628d`; the tester also replayed on master with the test hunks copied in | CLOSED (the whole audit entry; roadmap 3.5 still lists KS-28 in its issues column, see the book sync note) |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, outcomes, symmetry) | OK | `src/hook/keyboard_hook_callback.rs` is unchanged. Conditions are evaluated only for key-downs (`src/engine/mod.rs:376`). A repeat follows the key-down record whatever the bindings say (`:502-526`), and a key-up follows the record and pending gestures (`:538-562`). So a now-varying `AppEquals` cannot desynchronise suppression. A title flip mid-chord is already pinned (`scenarios.rs:1888-1916`) |
| Hook path cost (blocking, allocation) | WARN (prescribed, tracked) | Same three kernel calls per lookup as master (`OpenProcess`, query, close), none of which waits on the target or pumps messages. New: the query now succeeds (developer's read-only measurement, native form: p50 2.2-2.5 us, p99 4.7-7.2 us, max 28-86 us), and `fetch_app` returns one `String` per key-down that evaluates an `AppEquals` (two more for a non-ASCII fold). Only configs with a `when:` pay: `Always` never asks (`engine/mod.rs:373-376`), and the default config has no active `when:`. The `String` was already master's success branch, unreachable only because of KS-03; `fetch_title` already allocates the same way. It is prescribed by the cut (D1 cost, D5 cost, RK1), anticipated by roadmap 3.5 "Why", and removed by 3.5 (per-HWND cache) and 3.4 (counting-allocator acceptance). Not removable locally without that design. The 2 KiB stack buffer is fine. No panic path: the length is clamped (`condition.rs:215`), with no `unwrap` or indexing |
| Engine purity, matching per owner decisions | OK | `src/engine` unchanged except the T4 scenario. `equal_ignoring_case` is pure Rust in `data/condition.rs`, reached through `Condition::evaluate`. Modifiers, repeat policy, RON and licence are untouched |
| Thread ownership, sentinel, atomics, SAFETY | OK | No hook, thread, window (outside the test), timer, atomic, static or `SendInput` added. Production `unsafe`: `condition.rs:191-194` (OpenProcess/Owned) and `:203-214` (query), each with a concrete SAFETY (pointer, buffer lifetime, size in UTF-16 units). Test `unsafe`: `:403-424` (CreateWindowExW, message-only) and `:430-433` (DestroyWindow on the creating thread; `HWND` is `!Send`), both concrete |
| Config reload, IPC | OK (n/a) | No schema, loader or pipe change. `Condition`'s data and `Debug` text are unchanged |
| No key logging | OK | No log line added |
| Safety of the new tests | OK | T1/T2 read only the test's own process. T7 creates a message-only window (never shown, activated or broadcast to) on its own thread and destroys it in `Drop`. No hook, no input injection, no foreground change, no registry. T6 (sleep + `GetForegroundWindow`) is ignored |

## Review status

- Critique r1: CHANGES_REQUESTED (0 Critical, 1 Important, 4 Minor). All resolved in the tree, which I checked:
  - **I1** (the fix makes KS-35 bite `Not(AppEquals(..))` + `block_original_combo`) is declared in `CHANGELOG.md:137-144`, `README.md:430` and `src/config/default_config.ron:131-132`.
  - **M1**: `PROCESS_NAME_NATIVE` at `condition.rs:198-213`, with the measurement in the comment and the commit body.
  - **M2**: T7.
  - **M3**: the layout-independent manual note.
  - **M4**: no comment names `CloseHandle`/`GetModuleBaseNameW` in `condition.rs`.
- Code review r1: **APPROVED** (0 Critical, 0 Important, 1 Minor m1). No triage was needed.
- Tester r1: **GREEN** (344 / 0 / 1 ignored, 5 mutations all caught, flake check 5x).
- Open Minor and NOTED items are listed in "Notes for later lanes" (N1, N2).

## Docs and hygiene

- Config meaning changed (KS-03: `AppEquals` now matches; KS-28: non-ASCII folding). The same branch updates:
  - `README.md:430` (KS-35 interaction), `:439-440` (folding, Cyrillic example), `:466` (AppEquals row) and `:473-475` (UWP note);
  - `src/config/default_config.ron:131-132` and `:141`;
  - `CHANGELOG.md` `## [Unreleased]` / `### Fixed`: KS-03 at `:130-148` and KS-28 at `:230-233`, both citing their IDs.
- Book: untouched. It is deferred to the phase book sync (`docs/WORKFLOW.md:104-107`), as in A2-A5 and per cut Q1. `CLAUDE.md` asks
  the developer to update the matching `book/src/config/*.md` page in the same lane; the deferral relies on the phase's book sync
  actually running. The published pages keep saying "AppEquals never matches" until then. That is known drift, not a keyboard
  hazard.
- Dependencies: none added. The feature `Win32_System_ProcessStatus` is removed, `Cargo.lock` is unchanged and `--locked` passes.
- The branch holds only this lane's work, its merge-base is master HEAD, and it merges with `--no-ff` without conflicts expected
  (master has not moved).

## Technical debt

| Item | Where it is recorded |
|---|---|
| Per-key-down process query and `String` under the engine lock when an `AppEquals` binding exists | roadmap 3.5 "Why"/acceptance (`roadmap.md:381`, `:385`), 3.4 counting allocator (`roadmap.md:375`). Recorded |
| KS-35 now observable: the auto-blocker ignores `when:`, so `Not(AppEquals(x))` + `block_original_combo` leaves no layout hotkey inside `x` | roadmap 1.2; declared in CHANGELOG, README and the template. Recorded |
| UWP windows report `ApplicationFrameHost.exe` | `README.md:473-475`, CHANGELOG KS-03. Recorded (the book's `config/triggers-conditions.md` §12.3 already advises `TitleContains`) |
| Image path of 1024+ units reads as `None`; unpaired surrogates become U+FFFD | `fetch_app` doc `condition.rs:139-141`. Recorded in code only; acceptable |
| `fetch_title`'s 512-unit buffer: `TitleEquals` never holds for titles over 511 units (pre-existing, no KS id) | only `cut.md` non-goals and `impl.md`. **Not in the repo**: book sync -> `config/known-issues.md` or `gaps.md` |
| RK3: non-elevated (debug) daemon against an elevated foreground is unverified (the release daemon is elevated) | only `cut.md` RK3/Q5. **Not in the repo**: book sync -> `gaps.md` or `known-issues.md` |
| `.claude/agents/code-reviewer.md:58`, `:128` (R9) still describe `GetModuleBaseNameW` | cut Q2. The orchestrator edits it after the merge |
| No `TODO`/`FIXME` added | `git diff` |

## Problems

None blocking. No acceptance bullet lacks evidence, no claimed KS-ID lacks a red-first regression test, no invariant is broken,
no CONFIRMED Critical or Important item is open, and the test count went up (338 -> 344 + 1 manual).

## Positives

- The red-first chain is exact and reproducible. C1 is a pure refactor, so the red commits are "master behaviour + new tests",
  and my own replay reproduced every predicted failure text, including the HRESULT.
- T7 closes the gap that let KS-03 ship ("no test evaluates `AppEquals` against a real process"). It does so through the very type
  the hook uses, without touching the foreground or installing anything. A future 3.5 rewrite of `fetch_app` that loses the
  name now fails the gate.
- The native path form is a measured, well-argued deviation (tenfold cheaper on the hook thread; same final component; also works
  on volumes without a drive letter).
- `equal_ignoring_case` is provably consistent with `TitleContains`: the final-sigma row rules out a char-wise fold, and the
  Kelvin row pins the "one non-ASCII side takes the slow path" rule.
- The critic's I1 was handled as a user-facing declaration where users read it, verified against the real loader.

## Notes for later lanes (orchestrator)

- **N1 (Minor, optional, developer).** `condition.rs:180-181` says 1024 is "four times MAX_PATH" (4 x 260 = 1040; review m1).
  Say "1024 units, about four times MAX_PATH". One-line doc fix; can ride the book-sync branch or the next lane touching the file.
- **N2 (Minor, optional, tester).** A mutant that drops `&& b.is_ascii()` in `equal_ignoring_case` (`condition.rs:87`) survives:
  every Kelvin row has the non-ASCII side on the actual, not the config. One assertion would kill it, for example
  `AppEquals("\u{212a}.exe")` against an app `"k.exe"`. The only real input it affects is a config string with U+212A.
- **N3 (ruling, orchestrator/owner).** Cut Q4 files KS-03 under **Fixed**. The CHANGELOG header (`CHANGELOG.md:7-9`) puts "a
  breaking change to the `config.ron` ... behaviour" under **Changed** with "**Breaking:**", and the KS-18 entry (`:82-91`) set
  that precedent for a `when:`-driven change. The KS-35 consequence (a config that switched the layout in `game.exe` no longer can
  there) arguably fits it. Either confirm Q4, or add a short "**Breaking:**" pointer under Changed referring to the KS-03 entry.
  The text itself is accurate, so this does not block the merge.
- **N4 (book sync, doc-writer).** The list is in `cut.md` "Docs impact" plus `impl.md` "Open points". Add the three lines my
  grep found that the cut's list misses:
  - `book/src/config/example.md:30` (RON comment "AppEquals(...): currently always false");
  - `book/src/gaps.md:180` (the "all comparisons case-insensitive" drift row, now true);
  - `book/src/roadmap.md:40` (RC7 row "AppEquals never matches").

  Also:
  - mark KS-28 fully fixed in `audit.md` (the entry is only about folding), with KS-03 alongside;
  - say in the book that the call uses the native path form wherever it quotes it (`glossary.md:307`, `architecture.md:362-365`);
  - put the KS-35 interaction into the `block_original_combo` text of the config pages;
  - record the `fetch_title` 512-unit limit and RK3 (Technical debt above).

  `methodology.md` is the historical probe; leave it.
- **N5 (CI).** T7 creates a message-only window and T1 queries the test's own process. Both should work on the GitHub Windows
  runner (message-only windows need no interactive desktop), but they have only run on this machine. Watch the first `ci.yml` run
  after the merge push.
- **N6 (owner FYI and manual checks).**
  - Run smoke 7 / `impl.md` M1-M4 after the merge (M5 optional for the KS-35 interaction).
  - If the owner's live `config.ron` combines `Not(AppEquals(..))` with `block_original_combo`, that app loses its layout hotkey
    until roadmap 1.2.
  - Any `AppEquals` binding now costs a few microseconds per key-down until 3.5.
