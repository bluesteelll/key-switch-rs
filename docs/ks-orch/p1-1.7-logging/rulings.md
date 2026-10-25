# Orchestrator rulings for lane p1-1.7-logging (2026-10-18)

Binding for every stage of this lane (developer, tester, reviewer, triage, results). Where a ruling
differs from `cut.md`, the ruling wins. The developer records how each one was done in the addendum at
the end of `cut.md` and in `impl.md`.

Context: the phase-1 run stopped at this lane's implement stage on a usage limit (2026-10-16). Cut and
critique (`cut.md`, `critique.md`: CHANGES_REQUESTED, Critical 0, Important 2, Minor 4) are kept as they
are; this lane resumes at the developer.

## Architect's questions (cut.md "Questions for the orchestrator")

- **Q1 UTC or local time.** UTC with milliseconds and `Z`, as D5 says. Technical fork: pure, testable,
  DST-unambiguous, valid ISO-8601. The README "Log file" section says the times are UTC.
- **Q2 the `Press Enter to exit...` prompt.** Accepted: `writeln!` on `io::stderr().lock()`, not a record.
- **Q3 elevated writes in the user-writable `%LOCALAPPDATA%\key-switch-rs`.** Accepted for 1.7. The
  hardening (refuse a reparse-point folder or file, or handle-relative renames) is a follow-up: the
  developer adds one line for it to `book/src/gaps.md` in commit 8 (with items 3.3 / 4.1 as candidates).
- **Q4 panic hook.** Not in 1.7 (no acceptance line). Recorded as a follow-up for 2.4 in the same
  `gaps.md` line group.
- **Q5 CLAUDE.md.** Agents do not edit `CLAUDE.md`. The orchestrator updates it after the merge (Layout
  gains `src/logging/` and `system/known_folder.rs`; the logging owner decision loses "until that item
  lands"). Do not touch `CLAUDE.md` in this lane.
- **Q6 COM for `SHGetKnownFolderPath` / `CoTaskMemFree`.** Accepted: no `CoInitialize`. The read-only
  test on a libtest thread is the check.

## Critique, Important

- **I1 multi-line records: option (a).** In `daemon.log`, EVERY physical line of a record carries that
  record's timestamp and tag: `{ts} [{TAG}] {prefix}` on the first line, and the same `{ts} [{TAG}] `
  on each continuation line. Trailing line breaks of the message are trimmed (no blank lines in the file
  and no blank line on the console from a message's own trailing `\n`). The console keeps today's layout
  otherwise: `[TAG] ` on the first line only, continuation lines as they are. The expansion into
  per-line prefixes happens on the `log-writer` thread, not on the caller's thread, so the hook-thread
  cost of a record does not change (one `String`, one `try_send`). Tests:
  - `a_multi_line_message_is_one_record_with_one_final_newline` is replaced by a test that pins this rule
    for the file text and the console text, including a message that ends with `\n`;
  - `a_reload_error_is_logged_with_a_timestamp` checks that EVERY line of the reload-error record in the
    captured/file output starts with a timestamp and `[ERROR]`, not only the header (the capture under
    `cfg(test)` must therefore see the file form, or the test must go through a writer into a TempDir);
  - `book/src/config/errors.md:30,32` (the blank-line layout) and the README sample follow the new rule.
- **I2 the `cfgcheck` harness must keep showing loader diagnostics.** `logging::init()` stays public and
  usable by any binary: it installs the logger with console output only until `attach_file`. The harness
  in `book/src/methodology.md#validation-harness` calls `key_switch_rs::logging::init()` first and
  `key_switch_rs::logging::flush()` before every exit (`process::exit(1)` included). Fix the harness's
  stale `{}` on `b.block_original_combo` (it is `Option<BlockerMode>` since 1.2, so `{:?}`; noted by the
  1.2 results). Update its sample output (the `[INFO]`/`[WARN]` lines now come on stderr). In this lane,
  build the updated harness under `D:/tmp/ks-orch/p1-1.7-logging/cfgcheck/` (its own target dir, path
  dependency on the lane tree) and run it once on `src/config/default_config.ron` and once on a file with
  a duplicate binding; paste both outputs into `impl.md`. The harness is parse-only and installs no hook.
  `book/src/config/example.md:8-13` must still match what the harness prints.

## Critique, Minor (all adopted)

- **M1** D4 rotation stops at the FIRST failed rename. Add a test that holds `daemon.2.log` without
  delete sharing and asserts that `daemon.1.log` keeps its content and the line lands in `daemon.log`.
- **M2** The writer writes the file part BEFORE the console part. The remaining limit (a held selection
  in a conhost console delays later records, and `flush` gives up after 1 s on exit) is stated in the
  README "Log file" section and in the cut addendum. A separate console thread is out of scope (4.2).
- **M3** Manual check M7 names the host: run it under `conhost.exe` with QuickEdit on, and once on
  `master` as the control (where the stall is expected).
- **M4** Add to the commit 8 touch set: `book/src/config/actions.md:289` (macro step errors go to the
  console and `daemon.log`), `book/src/config/errors.md:30,32` (I1 layout), `book/src/architecture.md`
  startup sequence (`:91-102`: logger first, log file after the instance guard).

## Critique, notes for the architect (adopted)

- N1: D1 and the Threading row say "no I/O; `try_send` takes a std-internal lock for O(1) list work when
  it wakes the parked writer", not "no lock".
- N2: a failed startup probe does not make the whole run console-only. `attach_file` attaches anyway and
  the writer's error-streak logic reports once and retries per line.
- N3: the scan test of commit 6 also rejects `log::` macro calls in `src/hook/keyboard_hook_callback.rs`
  outside `#[cfg(test)]`, so "the hook callback never logs" is a gate check.
- N4 (tester): the red-firsts of commits 3 and 6 replay on their PARENT commits (2 and 5), not on
  `master`, where they are compile errors (no `logging` module). The commit-1 module tests are new
  capability with no red on `master`. Treat that as declared, not as a missing red.
- N5: `ipc_server.rs:369` - remove the now-unused `let path` binding (or use it in the ignore reason) in
  commit 7, so clippy `-D warnings` stays green on every commit.

## Hand-offs from earlier lanes that concern this lane

- 1.5 results: the already-running line changes in this lane (`main.rs:23`), so its manual checks M1/M1b
  (second start prints the line and exits 2, no new line in `daemon.log`) are part of this lane's manual
  checks (cut M3). List them in `impl.md` and the CHANGELOG manual-check line.
