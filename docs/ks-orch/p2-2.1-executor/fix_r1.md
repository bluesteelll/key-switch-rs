# Implementation: p2-2.1-executor fix round 1

Base `a0a8b82`, head `a875960`, branch `feat/action-executor`. Input: `triage_r1.md` (1 confirmed, 4 refuted, 0 deferred).

## Items
- F1 (triage CONFIRMED, test defect) - `src/core/executor/mod.rs:694-696` (import `APTTYPE_MAINSTA`), `:1337-1343` (new
  `is_sta` helper), `:1367` and `:1382-1386` (the two asserts).
  - red (at `a0a8b82`, each run alone with `--exact`):
    - `core::executor::tests::the_executor_runs_jobs_in_a_single_threaded_apartment`: panicked at `mod.rs:1357:9`,
      "assertion `left == right` failed, left: Some(APTTYPE(3)), right: Some(APTTYPE(0))".
    - `core::executor::tests::the_apartment_guard_uninitialises_com_on_drop`: panicked at `mod.rs:1372:9`, the same
      assertion (left `Some(APTTYPE(3))` = `APTTYPE_MAINSTA`).
  - Cause: the lone STA thread becomes the process's main STA, which `CoGetApartmentType` reports as `APTTYPE_MAINSTA`.
    Production `Apartment::enter_sta` / `Drop` (`mod.rs:370-396`) is unchanged.
  - Fix: `is_sta(kind)` is true for `Some(APTTYPE_STA)` or `Some(APTTYPE_MAINSTA)`. The executor test asserts
    `is_sta(kind)`. The guard test asserts `is_sta(inside)` and `!is_sta(after)`, and each message prints the observed kind.
  - green: both tests pass alone (`--exact`, 1 passed each). `cargo test --locked --lib core::executor -- --test-threads=1`
    gives 33 passed, 0 failed, 1 ignored. The default-parallel `core::executor` run gives 33 passed, 0 failed, 1 ignored.
  - M17 mutation (temporary `if self.initialized && false` in `Drop for Apartment`, file restored byte for byte from a
    scratch copy afterwards, `git diff` re-checked): the guard test run alone fails at `mod.rs:1383:9` with "after the guard
    the thread has no apartment of its own (or the implicit MTA), not Some(APTTYPE(3))". That is the right assertion, so a
    lone run now catches M17.

## Remarks
- F1 - RESOLVED at `src/core/executor/mod.rs:1337-1343,1367,1382-1386`, pinned by
  `the_executor_runs_jobs_in_a_single_threaded_apartment` and `the_apartment_guard_uninitialises_com_on_drop`.
- F2/M18, F2/M21, F2/M22, F2/M26 - REFUTED by triage (closed by tester commits `c94acd5`, `2e51c2a`, `a0a8b82`). No
  change.
- review m1 (`pub fn thread` lacks a `///` line), review OQ (wake to a dead executor id), and the tester note on
  process-wide `COUNTERS` - NOTED by triage, not confirmed. No change.

## Deviations from the cut
- none (test-only change)

## Unsafe / atomics added
- none

## Gates
- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`:
  pass (no warnings).
- Tests run: the two F1 tests alone (red, then green), `core::executor` with `--test-threads=1` and with the default thread
  count, and the M17 mutation run of the guard test. The full suite is left to the tester.
- Manual ignores added: none.

## Docs
- None. The change touches tests only and changes no user-visible behaviour.

## Commits
- `a875960` Tests: the COM apartment tests accept the process's main STA

## Open points / pre-existing warnings
- None.
