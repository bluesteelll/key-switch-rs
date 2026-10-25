# Implementation: p2-2.3-control-window fix round 1

Base: `4539306` (developer `6f6edb3` + tester commits `c7ab5c7`, `7ab78ab`, `4539306`).
HEAD after this round: `81bb38c`. Input: `triage_r1.md` (CONFIRMED 1: R-I1; NOTED 2: R-m1, T-N1; REFUTED 8).

## Items

- **R-I1** (review I1, Important) - `src/hook/config_watcher.rs:443-479` (test only; no production line changed) - red-first `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path`:
  - Vacuity at `4539306`: with `.map_err(notify::Error::io)?;` at `:158` replaced by `.expect("spawn config-watcher thread");`, the old test ran `... the_watcher_thread_has_no_panic_path ... ok` (the scan stops at the test-only `#[cfg(test)]` import at `:37`). Mutation reverted.
  - Step A (coverage assertion added, old `take_while` region kept): red "the scan of src/hook/config_watcher.rs ends after line 36 and misses `fn spawn_watcher(`" (`fix_r1_redA.txt`). Red for the right reason: the region is the defect.
  - Step B (region ends at the `#[cfg(test)]` + `mod tests {` pair, as `non_test_lines` in `keyboard_hook_callback.rs` and `tests_start` in `logging/mod.rs` do; `.expect` if the pair is missing): green (`fix_r1_greenB.txt`).
  - Mutation replay 1, `:158` `.expect("spawn config-watcher thread");`: red "panic paths in src/hook/config_watcher.rs:\n158: .expect(\"spawn config-watcher thread\");" (`fix_r1_mutA.txt`). Reverted.
  - Mutation replay 2, `let _ = config_path.to_str().unwrap();` inserted in `reload_into`: red "panic paths in src/hook/config_watcher.rs:\n200: let _ = config_path.to_str().unwrap();" (`fix_r1_mutB.txt`). Reverted.
  - After both reverts: `git diff` shows only the test; all 11 `hook::config_watcher` tests green (`fix_r1_green.txt`).
  - The searched signature is assembled (`concat!("fn spawn", "_watcher(")`) like the tokens, so the test cannot satisfy its own coverage check. The region now also covers the test-only import (`:37-38`) and impl (`:56-62`) above the tests module; they hold no panic token.
  - `CHANGELOG.md:874-879` names this test as the proof of the watcher fix; that claim holds again, so the entry is unchanged.

## Remarks

- **R-I1** - RESOLVED at `src/hook/config_watcher.rs:443-479` (commit `c4f1bca`), pinned by `the_watcher_thread_has_no_panic_path`, red under both reviewer mutations.
- **R-m1** (review m1, Minor, NOTED; the triage says it can ride along) - RESOLVED in commit `81bb38c`, doc only:
  - `src/hook/main_thread.rs:28-32` step 1 names the config reload request and the input-signal step (requests, then signals, then timers, the order `drain` runs them).
  - `src/hook/main_thread.rs:38-40` step 4: a noted input signal also posts `MAIN_WAKE` (`main_thread.rs:164-171`).
  - `src/hook/main_thread.rs:667-669` `drain`'s doc names the signal step.
  - `src/hook/keyboard_hook_callback.rs:16-22` step 5: the executable name comes from the main thread's foreground cache with no call (`HookEnv::foreground_app` -> `ForegroundCache::app`), and the title and the OS key state are live Win32 calls (`read_title` -> `GetWindowTextW`, `os_key_down` -> `GetAsyncKeyState`).
  - Tests of both touched files and the source-scanning `logging` tests: 87 passed, 1 ignored (pre-existing manual test) (`fix_r1_docs_tests.txt`).
- **T-N1** (tester note, NOTED, optional) - REFUTED as a defect; bound left unchanged:
  - `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` asserts `stopped < ms(100)` against a 500 ms `Delay`. The 400 ms margin tells "cancelled at the lock" from "ran out its Delay".
  - It passed 7 of 7 tester runs and passed again in this round. It fails for no product reason.
  - Widening it would loosen an existing assertion with no observed failure, and the triage does not confirm the item. If the tester's full-suite run ever shows it flaking, that is the evidence for a later round.
- **T-B1** - REFUTED by the triage (the +0.6 us R11 breach is the witness read that `cut.md:421` and ruling R1 justify). No developer action. The measured +0.6 us is a `results.md` obligation for the results analyst.
- **T-B2, T-B3, T-B4** - REFUTED by the triage (the D14 bounds hold on the official set or on the re-run). No developer action.
- **T-M12, T-M13, T-M20** - REFUTED by the triage (closed at `4539306` by the tester's commits `c7ab5c7`, `4539306`, `7ab78ab`). No developer action.
- **T-M16** - REFUTED by the triage (an equivalent mutant: `Ids::next_seq` is never reset and `reset` empties every table). No developer action.

## Deviations from the cut

None. Both commits stay inside the lane's touch set (`config_watcher.rs` C9/C10, `main_thread.rs` and `keyboard_hook_callback.rs` under M7).

## Unsafe / atomics added

None.

## Gates

- `cargo fmt --check`: pass (exit 0) at `81bb38c`.
- `cargo check --all-targets --locked`: pass.
- `cargo clippy --all-targets --locked -- -D warnings`: pass, no warnings.
- Tests run: `hook::config_watcher` (11 passed), `hook::main_thread` + `hook::keyboard_hook_callback` + `logging` (87 passed, 1 ignored). The full suite is left to the tester.
- Manual ignores added: none.

## Docs

- No book, README, `default_config.ron` or CHANGELOG change: R-I1 is test-only, and the CHANGELOG entry citing the test is accurate again. R-m1 changes module and function docs in `src/hook` only.

## Commits

- `c4f1bca` Tests: the watcher's panic-path scan covers every line before its tests module
- `81bb38c` Hook: the loop docs name the reload request, the input-signal step and the cached executable name

## Open points / pre-existing warnings

- None from the gates.
- Git prints "LF will be replaced by CRLF" for `src/hook/main_thread.rs` (`core.autocrlf=true`). The working copy is uniformly CRLF (1710 of 1710 lines) and the blob is LF, as for every file; the diff holds the doc lines only.
- Carry-over for the results analyst (from the triage, not code): state the witness read's measured +0.6 us (R11) against the "~0.1 us" of `cut.md:448` and ruling R1's "~100 ns".
