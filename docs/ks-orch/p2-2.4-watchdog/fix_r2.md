DONE (deviations: 2, listed below)

# Implementation: p2-2.4-watchdog fix round 2

Branch `feat/hook-watchdog`, from `f51dc7d` to HEAD `4d6b228`, tree clean. This is the resumed run. The first run of this round STOPPED for a ruling on R16, and the ruling is `rulings.md` round 4 (L2).
Inputs read: `triage_r2.md`, `rulings.md` (rounds 1-4), `spec.md`, `review_r2.md` (I1, m1, m2, open questions), `test_r2.md` (bench), `fix_r1.md`, `bench-dev-diag.md`, `cut.md` (L1.6, L1.11-L1.14, manual checks, addendum), `impl.md` (follow-ups, manual checks), and `src/hook/watchdog/mod.rs` (`run_loop`, the drain, `RawBuffer`, `count_raw_keys`, `KeyboardReads`, the scans).

## Items

- **T-F1 = R-I1, ruling L2 (R16 is a declared evidence row)**: RESOLVED as the ruling says. No code changed and no test was added: a bench row is `#[ignore]`d and has no assertion.
  - `cut.md` L1.6, row R16. R16 is "declared evidence row: measured and reported, not gated". The row gives the measured figure: about 50 us at p50 above R12, once per burst of keys after at least `ACTIVE_FOR` = 2 s without keys, in the worst order of Q6.
    - The tester's HEAD runs: p50 54.6 / 52.7 / 46.1 us, p99 107.6 / 101.2 / 121.9 us, R12 4.7 us.
    - The developer's run: 53.5 / 168.4 us.
    - The withdrawn sanity bound stays in the row as "as first written".
  - Other places in `cut.md` that named a planned number now name the measured one or point to L2:
    - L1.13 (the planned "18-25 us");
    - L1.9 (the README sentence planned with "20 us");
    - L1.11 ("sanity bound");
    - L1.14 Q2;
    - the addendum's V3-M2 open point and its Delta Q1-Q6 line.
  - `README.md:939`, the Hook watchdog bullet. A callback that runs during the wake is about 50 us slower at the median than R12 (bench row R16, the developer's machine), at most once per burst and only in the worst order. If raw input comes after the hook call, the key's own callback never meets the wake.
  - `src/hook/watchdog/mod.rs:33-42`, the "# Cost" docs: the same, plus "a declared row, measured and not gated: ruling L2", and "manual check M6 settles it".
  - `src/bench.rs:75-82`, the R16 docs: "A declared row, measured and reported, not gated (ruling L2)", which times the worst order of Q6, once per burst.
- **L2, the p99 of the 1 ms-paced rows**: `cut.md` L1.6.
  - Row R14 gates on p50 against R14c (ruling V3-I2). Its p99 is reported as the largest of 3 runs against R14c's largest of 3.
  - A new row R14c says that the control is reported.
  - The addendum's V3-I2 line has a note.
- **L2, Q6 goes to the owner**: manual check M6 is amended (full text below). It is in the `cut.md` manual checks table and in `impl.md` (Manual checks).
- **L2, the 3.5 follow-up**: `impl.md`, "Follow-ups from Phase 2", FU-4 (text below).
- **R-m2, key codes do not outlive their batch**:
  - Code:
    - `src/hook/watchdog/mod.rs:864-882`: `RawBuffer::take(written)` maps the value `GetRawInputBuffer` returned (0, `(UINT)-1`, or the records written) to `Read`.
    - It then clears the whole buffer (`fill(0)`) after any read that may have written to it, that is, records or a failed read.
    - `KeyboardReads::buffered` calls it (`:1120`).
    - The module's "# No key data" docs (`:51-58`) say so.
  - Red-first test `a_batchs_key_codes_do_not_outlive_its_count` (`:2128`).
    - Red on `3117b0a`: `assertion 'left == right' failed: bytes of a counted batch are left`, `left: 12`, `right: 0`.
    - Green on `fef60f9`.
  - Mutation (developer, informal): clearing only after a batch of records is caught by the failed-read half of the test (`bytes of a failed read are left`, `left: 4`).
- **R-m1, the stop-line wording**: `book/src/config/errors.md:215`.
  - About 10 wakes per isolated shortcut, and about 4 a second for longer typing. So N above M is normal for isolated bursts: 100 Ctrl+C presses give M = 400 and N about 1000.
  - N stays far below M only in sustained typing: a 20 s hold adds about 600 to M and about 90 to N.
  - The per-key symptom is now named for sustained typing only.
  - This is a doc fix, so no test can be red for it.

## Manual check M6 (amended, ruling L2; for the owner)

Purpose: R-1, and cut Q6. Q6 asks whether Windows hands a key's raw input to the watchdog before or after it calls the low-level hook for that key, and the answer decides whether a real key ever pays R16.

**Part A (release daemon)**
- Setup: start the release daemon (`swch open`). Then start an AutoHotkey script with `CapsLock::return`, after the daemon.
- Steps: hold CapsLock alone for 3 s, then type normally for 10 s.
- Expected: at most one reinstall line (residual R-1), and none after it.
- Reading for Q6:
  - **No reinstall line.** Keys that a later program's hook swallows reach no raw input reader, so Windows makes raw input only after the hook chain. Q6 = after, and part B is optional.
  - **One reinstall line within about 1.5 s of pressing CapsLock.** Raw input exists for swallowed keys. It comes either before the hook chain or after it whatever its verdict, and part B decides which.

**Part B (debug build, the log's millisecond timestamps)**
- Precondition (ruling M7):
  1. Quit the AutoHotkey script.
  2. Run `swch exit`, then check that `swch status` says the daemon is not running.
  3. Build with `cargo build --locked` (debug, not elevated).
  4. Note the value of `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` (read only; "absent" is an answer).
  5. Choose a key the config does not bind. F9 is used here.
- Steps:
  1. In console B, run `while(1){ $k=[Console]::ReadKey($true); "$((Get-Date).ToUniversalTime().ToString('HH:mm:ss.fff')) $($k.Key)" }`. (`ReadKey` skips ScrollLock, so a key it returns is used.)
  2. In console A, type `$env:KS_DEBUG_HOOK_HOLD='F9'; Start-Sleep -s 3; .\target\debug\key-switch-rs.exe`, press Enter, and take your hands off the keyboard. The sleep lets Enter's release pass before the daemon starts, so F9 is the first key event the watchdog counts.
  3. When `[WARN] debug build: the keyboard callback holds 1.5 s at each F9 key-down (KS_DEBUG_HOOK_HOLD)` appears, click console B with the mouse and wait 3 s.
  4. Press F9 once, then wait 5 s.
  5. Read console B's F9 line: this is t_key, in UTC. Then run `Select-String 'raw keyboard input arrives' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" | Select-Object -Last 1` (read only): this is t_raw, in UTC, from this run.
  6. Stop console B's loop. Run `swch exit`, then start the release daemon again with `swch open`.
- Expected and reading. The hold keeps F9's callback busy for 1.5 s, so console B gets F9 only after Windows gives up on the hook (its timeout, at most 1 s).
  - **t_raw is earlier than t_key by about that timeout (100 ms or more).** The watchdog read F9's raw input while F9's hook call was still running, so raw input comes **before** the hook call (Q6 = before). The first key of each burst pays R16, about 50 us at the median, and FU-4 applies.
  - **t_raw is within about 20 ms of t_key, either way.** Raw input comes **after** the hook chain (Q6 = after). A key's own callback never meets the wake it causes, and FU-4 lapses.
  - **Anything else, or no `raw keyboard input arrives` line.** Report both times and the timeout value.
- No code was added for part B. The first-arrival line, with its UTC millisecond timestamp, exists in every build. Nothing logs on the hook path.

## Follow-up FU-4 (in `impl.md`, "Follow-ups from Phase 2")

- FU-4 belongs to roadmap item 3.5 (hot-path budget) and comes from ruling L2.
- If M6 shows that a key's raw input reaches the watchdog before its hook call, item 3.5 trims the user32 work of the watchdog's first pass after idle. A keyboard callback that runs at the same time pays for that work (R16: about 50 us at the median above R12, once per burst of keys).
- If M6 shows raw input after the hook chain, FU-4 lapses.

## Remarks

- **T-F1** (triage_r2, CONFIRMED): RESOLVED by ruling L2. It is carried out in `cut.md` L1.6 (row R16), `README.md:939`, `src/hook/watchdog/mod.rs:33-42` and `src/bench.rs:75-82`. M6 settles Q6. No test pins it, because the bench row is ignored and the ruling makes it ungated.
- **R-I1** (triage_r2, CONFIRMED, the same defect as T-F1): RESOLVED, as T-F1.
- **R-m1** (triage_r2 NOTED; the ruling puts it in this lane): RESOLVED at `book/src/config/errors.md:215`. Doc only, no test.
- **R-m2** (triage_r2 NOTED; the ruling puts it in this lane): RESOLVED at `src/hook/watchdog/mod.rs:864-882`, pinned by `a_batchs_key_codes_do_not_outlive_its_count`.
- **Tester's R14 p99 note** (triage_r2, not triaged): settled by L2. The paced rows gate on p50, and `cut.md` L1.6 says so.

## Deviations from the cut

1. **The seam commit.** The behaviour-neutral seam commit `62cf30a` (`RawBuffer::take`, which maps the read's return value as before) comes before R-m2's red test, so the test is red on its assertion and not on a missing function. The alternative, a red commit that fails to compile as the round-1 red commits did, was rejected because the developer rules ask for an assertion red.
2. **The clear also covers a failed read.** R-m2 also clears the buffer after a failed buffered read (`(UINT)-1`), not only after a batch of records: Microsoft Learn does not say what a failed `GetRawInputBuffer` leaves in the buffer. An empty read (0) wrote no record, so it does not clear.

The standard read is unchanged. Its record is a local of `raw_key_event`, as it was before L1, and the reviewer recorded it as such (`review_r2.md` m2). The module docs say so.

## Unsafe / atomics added

- None. `fill(0)` is safe code.
- The buffer's address goes to `GetRawInputBuffer` at every buffered read, so a later read can observe the clear.

## Gates

- On HEAD `4d6b228`:
  - `cargo fmt --check`: pass.
  - `cargo check --all-targets --locked`: pass.
  - `cargo clippy --all-targets --locked -- -D warnings`: pass. Also on each commit of this round. The red commit's test compiles and fails by assertion.
- Tests run by the developer:
  - `cargo test --locked --lib a_batchs_key_codes_do_not_outlive_its_count`: red on `3117b0a`, green on `fef60f9`.
  - `cargo test --locked --lib hook::watchdog`: 74 passed, after the fix and again after the docs commit.
  - `cargo test --locked --lib buffered_read`: 5 passed, after the seam.
  - The full `cargo test --locked` run is left to the tester.
- Bench: not re-run. The production change is a 3 KB `fill(0)` after a read that returned records, on the watchdog thread, about 4 times a second while keys are typed. That is not on the key path. The tester decides on re-runs.
- Book: `mdbook build book -d D:/tmp/ks-orch/p2-2.4-watchdog/book` built. The mermaid preprocessor warns about its mdbook version, as before. A link check of the built book found 0 broken links: 23 pages and 2676 relative links and anchors, with print/toc/404 skipped.
- Manual ignores added: none. M6 needs a live hook.

## Docs

- `README.md:939`: the R16 sentence (L2).
- `book/src/config/errors.md:215`: the stop-line row (R-m1).
- `src/hook/watchdog/mod.rs`: the "# Cost" docs (L2) and the "# No key data" docs (R-m2).
- `src/bench.rs`: the R16 docs (L2).
- CHANGELOG: no change. No behaviour a user sees changed. The unreleased watchdog entry ("keeps counts and times, never key data") and the bench entry stay true.
- No config meaning changed, so `default_config.ron` and the config pages are unchanged.
- Lane files: `cut.md` (as above, plus the "Fix round 2" addendum, which records L2, R-m1, R-m2 and the deviations) and `impl.md` (FU-4, M6).

## Commits

- `62cf30a` Hook: a buffered read's answer is taken from its buffer by one method
- `3117b0a` Tests: a batch's key codes do not outlive its count (red)
- `fef60f9` Hook: the watchdog clears its read buffer after each batch is counted
- `1db14ca` Docs: the watchdog's stop line is read right for isolated bursts
- `4d6b228` Docs: R16 is stated as measured, once per burst and in the worst order

## Open points / pre-existing warnings

1. Part B of M6 depends on the debug daemon counting F9 as its first raw key event. A key typed between the start and F9 makes the first-arrival line that key's, and the run is then void and must be repeated. Part B says to keep hands off the keyboard.
2. These stay with the orchestrator; triage_r2 did not triage them:
   - `review_r2.md` open question 2: hand the owner M9 from `fix_r1.md`, not `cut.md` L1.12.
   - Open question 3: E4 for `QS_RAWINPUT`, which only M9 settles.
   - Open question 4: `KEYEVENTF_UNICODE` `wScan` as `MakeCode`. Its weight is lower now that the buffer is cleared after each batch.
3. Pre-existing warnings: none in the gates.
