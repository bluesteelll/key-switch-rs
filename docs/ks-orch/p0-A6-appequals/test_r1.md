# Tester: p0-A6-appequals / KS-03 + folding part of KS-28 (HEAD 32703a8 vs master a1b5a27)

Verdict: GREEN

Tree: `git status --short` empty on fix/app-equals at `32703a8`, before and after all work. `git worktree list` shows only the lane tree. No `Tests:` commit added (no gap found).

## Gate (HEAD 32703a8)
- cargo fmt --check: OK (exit 0)
- cargo clippy --all-targets --locked -- -D warnings: OK. First run printed "Finished ... 0.19s" (cached), so I touched lib.rs, main.rs, condition.rs, scenarios.rs and re-ran: `Checking key-switch-rs ... Finished dev profile ... 1.25s`, exit 0.
- cargo test --locked --no-fail-fast: 344 passed, 0 failed, 1 ignored.
  - lib: `running 339 tests` -> `test result: ok. 338 passed; 0 failed; 1 ignored`
  - swch bin: `running 0 tests` (no tests in that target, expected)
  - tests/ipc_roundtrip.rs: `running 6 tests` -> `ok. 6 passed; 0 failed; 0 ignored`
  - doc-tests: `running 0 tests`
  - ignored (manual): `data::condition::tests::foreground_context_names_the_focused_notepad` ("focus a Notepad window within 5 s"). Not run, not counted as a pass.
- cargo build --release --locked: OK (`Finished release profile [optimized] target(s) in 20.94s`; not run).
- Pipe-name check: `rg "key-switch-rs.sock" tests src/data src/engine` finds nothing. No test in the lane's diff touches a hook, SendInput, the registry or a real config.ron. T7 creates a message-only window on the test thread (no hook, no foreground change).

## Red-first replay
Replay trees: the red commits themselves ("old code + new tests"), each in its own worktree with its own `CARGO_TARGET_DIR` under `D:/tmp/ks-orch/p0-A6-appequals/target-wt`. For T3/T4 I also replayed on master by copying the HEAD test hunks onto master.

| Test | Parent / red tree | HEAD | Note |
|---|---|---|---|
| data::condition::tests::process_image_name_resolves_the_current_process (T1) | red (assert) at 9b5c2db: `process_image_name(own pid) failed: Access is denied. (0x80070005)` | `running 1 test`, ok | KS-03. On master it does not compile (fn absent): compile-red; the 9b5c2db tree (old GetModuleBaseNameW body behind `process_image_name`) is the behavioural red. |
| data::condition::tests::app_equals_matches_the_current_process (T2) | red (assert) at 9b5c2db: `AppEquals matches the test's own process (KS-03)` | ok | KS-03 |
| data::condition::tests::foreground_context_reads_a_window_of_this_process (T7) | red (assert) at 9b5c2db: `ForegroundContext::app() of a window of this process is None, not "key_switch_rs-9c8e127322721cde.exe" (KS-03)` | ok | KS-03, wiring of `fetch_app` |
| data::condition::tests::equals_conditions_fold_case_like_title_contains (T3) | red (assert) at cd1628d AND on master (HEAD test copied in): `TitleEquals ignores Cyrillic case like TitleContains` | ok | KS-28 |
| engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii (T4) | red (assert) at cd1628d AND on master (HEAD scenarios.rs copied in): `scenario: down F13 at +0 ms: expected the event to be suppressed, it was passed` | ok | KS-28 |
| data::condition::tests::image_file_name_is_the_part_after_the_last_backslash (T5) | n/a, arrives green with the fix (does not compile before it) | ok | pure helper; carried by my mutation M1 |

Quoted result lines. 9b5c2db tree, `-- data::condition`: `running 10 tests` ... `test result: FAILED. 7 passed; 3 failed; 0 ignored; ... 325 filtered out` (exactly T1, T2, T7 fail). cd1628d tree: `running 2 tests` ... `test result: FAILED. 0 passed; 2 failed` (T3, T4). Master + HEAD test copies: `running 2 tests` ... `test result: FAILED. 0 passed; 2 failed; ... 332 filtered out`.
HEAD, each with `--exact`: `running 1 test` / `test result: ok. 1 passed; 0 failed; 0 ignored; ... 338 filtered out` for all six tests (T1, T2, T7, T3, T5, T4).
Flake check: `cargo test --locked --lib -- data::condition` run 5 times at HEAD: `ok. 12 passed; 0 failed; 1 ignored` every time. None of the tests uses a real-clock term (T4 runs on the fake-clock `Scenario`).
Worktrees removed (`git worktree remove --force`, `git worktree prune`), `git worktree list` shows only the lane tree; target-wt dirs deleted.

## Mutations
All in the scratch worktree `wt/mut` at HEAD (`src/data/condition.rs` only), predicted before running. Verification command: `cargo test --locked --lib -- data::condition engine::scenarios` (`running 121 tests`).

| # | File / site | Mutation | Predicted catcher | Result |
|---|---|---|---|---|
| M1 | `image_file_name` (condition.rs:224) | split the path on `/` instead of `\` | process_image_name_resolves_the_current_process, image_file_name_is_the_part_after_the_last_backslash | caught: 4 failed: process_image_name_resolves_the_current_process, app_equals_matches_the_current_process, image_file_name_is_the_part_after_the_last_backslash, foreground_context_reads_a_window_of_this_process |
| M2 | `equal_ignoring_case` | drop the Unicode branch (always `eq_ignore_ascii_case`) | equals_conditions_fold_case_like_title_contains, when_title_equals_and_app_equals_ignore_case_beyond_ascii | caught: exactly those 2 failed |
| M3 | `Condition::TitleEquals` | equality -> `to_lowercase().contains(..)` (subset match) | equals_conditions_fold_case_like_title_contains (the `!TitleEquals("Kal")` row) | caught: that test and leaf_conditions_read_the_foreground_info failed |
| M4 | `fetch_app` | `process_image_name(pid).ok()` -> `None` (lose the app name in the real context) | foreground_context_reads_a_window_of_this_process | caught: exactly that test failed (T1/T2 call `process_image_name` directly, so only T7 guards the wiring) |
| M5 | `equal_ignoring_case` | ASCII fast path guard `&&` -> `||` | equals_conditions_fold_case_like_title_contains (Kelvin sign row) | caught: exactly that test failed |

Each mutant reverted by restoring the file from `HEAD` (`git show HEAD:src/data/condition.rs > ...`); `git diff --exit-code` clean after each (git status of the lane tree was never touched).

## Acceptance
Spec / roadmap criteria (cut.md acceptance map, spec text in the brief):

| Criterion | Evidence | Status |
|---|---|---|
| Replace GetModuleBaseNameW with QueryFullProcessImageNameW, take the file name | `rg -n "GetModuleBaseNameW\|ProcessStatus" src Cargo.toml` -> exit 1 (no match); T1 and T5 ok; M1 and M4 caught | verified |
| Close handles with RAII | `Owned::new(OpenProcess(..)?)` at condition.rs:193; `rg -n "ProcessHandleGuard\|CloseHandle" src/data/condition.rs` -> exit 1. Leak itself is not unit-testable; code-read only | verified by grep + code read (no leak test) |
| Env/ForegroundInfo abstraction kept; engine tests use a fake | `git diff --stat master..HEAD` touches no `src/engine/env.rs`, `src/engine/testing.rs`, `src/hook/`; T4 runs on `FakeEnv` via `Scenario::foreground` | verified |
| Non-ignored test resolves the current pid to its exe file name with the real function | process_image_name_resolves_the_current_process `ok` (`running 1 test`); red at 9b5c2db | verified |
| AppEquals matches (roadmap A: AppEquals true for a real process, so Not/And behave) | app_equals_matches_the_current_process ok (covers AppEquals, And, Not); foreground_context_reads_a_window_of_this_process ok | verified |
| AppEquals and TitleEquals case-insensitive, Unicode-aware, matching TitleContains | equals_conditions_fold_case_like_title_contains ok (Cyrillic, Greek final sigma, Kelvin, prefix negative); engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii ok; M2, M3, M5 caught | verified |
| Tests green; gate | fmt, clippy, `cargo test --locked --no-fail-fast` 344 passed / 0 failed / 1 ignored; release build OK | verified |
| Manual-check note in CHANGELOG | `CHANGELOG.md:145-148`: "Manual check (no automated test can focus a window): with `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )` loaded, F9 presses the A key in Notepad ... and reaches every other window unchanged." | present, verified by reading |

Manual only, not verified by me (never run per safety rules):
- foreground_context_names_the_focused_notepad (`#[ignore = "manual: ..."]`, focus Notepad within 5 s, owner runs `cargo test --locked --lib -- --ignored foreground_context_names_the_focused_notepad` from a non-elevated console).
- The F9 / Notepad binding check in the CHANGELOG (needs the daemon) and the smoke-checklist line referenced by the cut (roadmap.md:235).
- RK3 (non-elevated daemon vs elevated foreground window) is unverified and accepted in the cut.

## Failures
None.

## Tests added by the tester (`Tests:` commits)
none

## Notes
- The scratch worktree checked out files with CRLF (autocrlf); mutation script handled it. The lane tree was not affected.
- Limits of the evidence: the buffer cap (>= 1024 units returns None), invalid-UTF-16 lossy decode and the `ERROR_INVALID_DATA` path have no test; they are documented in the `fetch_app` doc comment. `PROCESS_NAME_NATIVE` (deviation D1 from the cut) is pinned by T1 on this machine and T5 for both path shapes; the Win32 form was not compared by me.
- `fetch_title` still uses a 512-unit buffer (pre-existing, recorded as an open point by the developer).
- `.claude/agents/code-reviewer.md:58,128` still mention GetModuleBaseNameW (developer open point, orchestrator after merge).
