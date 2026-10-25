# Architecture critique: p1-1.7-logging / roadmap 1.7 File logging (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 4)
Plan: D:/tmp/ks-orch/p1-1.7-logging/cut.md   Code read at: `master` = `1058ddb` (HEAD of D:/key-switch-rs/key-switch-rs)

No Critical: nothing in the cut blocks the hook thread, moves hook ownership, touches the engine or the
sentinel, or breaks the config. The two Important findings are a spec deviation that nobody flagged and a
documented validation tool that this lane silences. The developer may resolve or refute them (WORKFLOW.md,
Critique stage).

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. Multi-line records put lines with no timestamp and no level into daemon.log, against the spec line "Each line has an ISO-8601 timestamp and the level"
Where: plan D5 (last two bullets: "A message with newlines is one record with one trailing `\n`"); D8 row
`config_watcher.rs:166-167` (`log::error!("config reload failed:\n{e}")`); D8 row `main.rs:36` (`[FATAL]`);
commit 1 test `a_multi_line_message_is_one_record_with_one_final_newline`; Acceptance mapping row 2.
Code: `src/hook/config_watcher.rs:166` (newline right after `failed:`); `src/config/loader.rs:76-81`
(`ConfigError::Bindings` writes one line per invalid binding and ends with `\n`).

Problem: the timestamp and the tag are written once per record, and the message goes into the file as it
is. The reload error, which is the acceptance case itself, always has a newline after `failed:`, and a
validation error adds one line per invalid binding plus a trailing newline. Acceptance mapping row 2 says
"Each line has an ISO-8601 timestamp and the level" is covered by tests that use only one-line messages.
The plan's own multi-line test pins the opposite behaviour, and the deviation is never named.

Consequence: save `config.ron` as `( bindings: [ ( keys: "F13", action: PressKey("Foo") ) ] )`. daemon.log then gets

```text
2026-10-16T09:41:07.123Z [ERROR] config reload failed:
config has 1 invalid binding(s):
  - binding #0 [F13]: PressKey: unknown key "Foo"
                                     <- blank, unless D5 trims the message's own trailing newline
2026-10-16T09:41:07.124Z [INFO] keeping previous bindings live
```

The lines that carry the actual cause have no timestamp and no level. `Select-String '\[ERROR\]'` over the
three files finds only the header. A line-oriented reader cannot tell a continuation line from a truncated
one. That includes `swch logs` (4.3), which the cut lists as a dependant that reads daemon.log. The
`[FATAL]` line of a startup validation error has the same shape.

Confidence: CONFIRMED (plan D5, D8; `config_watcher.rs:166`; `loader.rs:76-81`).
Breaks: lane spec, bullet 1 ("Each line has an ISO-8601 timestamp and the level"). Acceptance mapping row 2 claims a coverage that the design contradicts.
What is needed: decide what a "line" is, write the decision into the cut and pin it with tests. Options:
- (a) In the file, every physical line of a record carries the record's timestamp and tag. The console can keep today's layout, at the cost of a second console text or a different slicing than `console_from`.
- (b) Split at the call sites, so that each physical line is its own record.
- (c) Keep one record per message, make continuation lines unambiguous (for example a fixed indent), and record "line = record" as an explicit interpretation of the spec in the cut, the README and the book.

Also state what happens to a message's own trailing newline (blank line or not), because `errors.md` documents that layout (M4). The multi-line test then pins the chosen rule. `a_reload_error_is_logged_with_a_timestamp` should check the continuation lines too, not only the header.

### I2. The documented `cfgcheck` validation harness goes silent: loader warnings and `[INFO]` lines vanish without a logger
Where: plan D8 rows `loader.rs:163,168,180,237,287,377` and `registry.rs:95,101,113,209`; commit 8 ("There is
no RON example change, so `cfgcheck` is not needed"); the Win32 table row "Logger install" (the max level
starts at Off). Book: `book/src/methodology.md:376-440` publishes the harness verbatim. It calls
`key_switch_rs::config::load`/`reload` and installs no logger. `book/src/config/example.md:8-13` publishes
validator output that includes `[INFO] SwitchLanguage: using default combination`. Process:
`.claude/agents/doc-writer.md:78-86`, and the rule in CLAUDE.md "Docs and the book": every published RON
example is validated with this harness first.

Problem: today the loader prints its own `[INFO]`/`[WARN]` lines, so every program that calls it shows them.
After commits 4 and 7 they are `log` records. A process that never calls `log::set_logger` uses the no-op
logger, and `MAX_LOG_LEVEL_FILTER` starts at 0, which is Off (`log-0.4.29/src/lib.rs:458`). The published
harness installs no logger, and the cut does not touch `methodology.md`.

Consequence: the doc-writer validates an example with a duplicate binding, or with
`block_original_combo: MaskOnly` on a hotkey that is not made only of modifiers.
- Before the lane, `cfgcheck` prints `[WARN] binding #1 [...] duplicates binding #0 [...]`, or the MaskOnly warning.
- After the lane, it prints only `OK: N bindings` and exits 0.

A page can then publish an example whose warning nobody saw. The next re-validation of `example.md` cannot
reproduce the `[INFO]` line printed on that page.

Confidence: CONFIRMED.
Breaks: CLAUDE.md "Docs and the book" (validation through `cfgcheck`); doc-writer workflow steps 4-5; the brief's "missing docs".
What is needed: the lane keeps the harness able to show the loader's diagnostics and updates
`methodology.md#validation-harness`, including its sample output, whose `[INFO]` lines now go to stderr.
One way: the public `logging::init()`, which writes to the console only until `attach_file`, plus
`logging::flush()` before every exit of the harness, `process::exit(1)` included. Whichever way is chosen,
put it in the commit 8 touch set and run the updated harness once on `default_config.ron` in the lane.

## Minor (improvements, not blockers)

### M1. The cut does not say that rotation stops at the first failed rename, and the planned test cannot tell
Where: plan D4 step 3 and "If the rotation fails ..."; commit 1 test
`a_rotation_that_cannot_rename_keeps_writing_to_daemon_log`, which holds `daemon.1.log` open.
Problem: D4 never says that `rotate` returns at the first failed rename. The test blocks `daemon.1.log`,
which is the source of the i=2 rename and the target of the i=1 rename. An implementation that ignores the
i=2 failure and goes on therefore also fails at i=1, and it passes the test.
Consequence: a viewer holds `daemon.2.log` open without `FILE_SHARE_DELETE`, and the i=2 rename
(`daemon.1.log` -> `daemon.2.log`) fails. A rotation that goes on then renames `daemon.log` over
`daemon.1.log`, and the previous 1 MiB of history is gone. D4 rejects truncation because it "loses data",
so the plan does not want this outcome.
Confidence: CONFIRMED that the test does not pin the rule. The data loss needs an implementation that goes on after a failed rename.
What is needed: say "stop at the first failed rename" in D4. Add a case that holds `daemon.2.log` and asserts that `daemon.1.log` keeps its content.

### M2. The writer writes the console before the file, so a stalled console holds back daemon.log
Where: plan D1 ("It writes the console part, then the file part"); the writer-loop sketch; D10 (flush timeout 1 s); lane-specific risk 2.
Problem: the cut names the QuickEdit stall itself: a selection in a conhost window blocks console writes. D1
moves that stall off the hook thread, which is right, but onto `log-writer`, which also owns the file.
Consequence: in a conhost console, the user clicks into the daemon window and leaves a selection there, then
saves a broken config. The reload error waits in the queue behind the blocked stderr write and is not in
daemon.log. If the user then runs `swch exit`, `flush` gives up after 1 s and `main` returns. The queued
lines, the reload error among them, never reach the file, and past 1024 queued lines new lines are dropped.
The file is the item's durable record, yet it depends on the state of an interactive console.
Confidence: PLAUSIBLE. This needs a conhost window: Windows Terminal does not pause output on a selection (see M3).
What is needed: either a blocked console cannot hold back the file write, or the cut and the README state this limit and the lane accepts it.

### M3. Manual check M7 cannot fail in Windows Terminal, the default console host on Windows 11
Where: plan M7; the D1 "Why".
Problem: only conhost pauses a process while a selection is open. Windows Terminal does not pause the client
on a selection:
- [microsoft/terminal#17176](https://github.com/microsoft/terminal/issues/17176): the maintainers describe the conhost behaviour and say that a pause in Terminal is impractical under ConPTY.
- [SnapOnOutput spec #980](https://cdn05042023.gitlink.org.cn/folkslinux/windows-terminal/src/branch/main/doc/specs/%23980%20-%20SnapOnOutput.md): it contrasts the two hosts.

On the owner's Windows 11 machine, a daemon console opened by `swch open` or a double-click normally runs in
Windows Terminal. M7 then passes both before and after the lane.
Consequence: the only manual evidence for D1's hook-thread claim proves nothing.
Confidence: CONFIRMED for the host behaviour. Which host the owner actually uses is PLAUSIBLE.
What is needed: M7 names the host. Start the daemon under `conhost.exe`, or with the default terminal set to
Windows Console Host, and with QuickEdit on. Run the same steps on `master` once, so the check is seen to fail there.

### M4. Doc pages outside the touch set become false
Where: the plan's touch set (book rows);
- `book/src/config/actions.md:289` ("A step error does not abort the macro: it is only printed to the console").
- `book/src/config/errors.md:30` and `:32`: the blank-line layout of the `[FATAL]` and reload output, which D5's newline rule changes (I1).
- `book/src/architecture.md:91-102`, the startup sequence: it has no logger start and no file attach. The list is already stale at `1dfd80a`, but this lane adds a step to it.

Consequence: the published pages say that macro step errors appear only on the console, and they describe a blank-line layout that may no longer exist.
Confidence: CONFIRMED (grep over `book/src`).
What is needed: add these lines to the commit 8 touch set. In `architecture.md`, at least add "logger first, log file after the instance guard".

## Strengths to keep

- **D1.** One owner thread, a bounded `sync_channel` and `try_send` on every caller, with full queues dropped and counted. This removes today's synchronous console writes from three places:
  - on the hook thread (`windows_actions.rs:257`);
  - between the end of the message loop and `uninstall` (`app.rs:210`);
  - between the two unhooks (`keyboard_hook.rs:236`).

  That serves R1 and R10.
- **D2.** The logger starts before the instance guard, and the file opens only after it, so a refused second start never opens, appends to or rotates the owner's file (KS-20).
- **D5.** Messages are formatted with `write!` into a `String`, not `format!`, so a failing `Display` cannot panic inside the extern callback.
- **D6.** Every per-event diagnostic on the hook path becomes one atomic RMW, and the messages that named a vk leave the hook path.
- **D11.** Tests never call `init` or `attach_file`. They use TempDir, an injected console and a thread-local capture, and the scan test is built with `concat!` so that it does not match itself.
- **The re-location table is accurate.** I recounted:
  - 53 print lines outside `src/bin` (6 of them the banner), and exactly 27 left at commit 6;
  - the `windows` 0.62.2 signatures (`UI/Shell/mod.rs:3301` `SHGetKnownFolderPath(..., Option<HANDLE>) -> Result<PWSTR>`, `System/Com/mod.rs:552` `CoTaskMemFree(Option<*const c_void>)`, neither behind an extra cfg);
  - `log` 0.4.29 `lib.rs:458`;
  - `Cargo.lock:116`.
- **The red-firsts fail for the stated reason.** Commit 3's test fails on commit 2 with `captured: []`: `reload_into` and `config::reload` still print directly, and the capture is thread-local on the calling thread. Commit 6's test fails on commit 5 with 27 sites.

## Open questions and notes for the architect

1. **The R1 wording overstates "no lock".** `SyncSender::try_send` takes a lock in exactly the case that matters here. When the receiver is parked, which is the writer's idle state, std's `SyncWaker::notify` takes its internal `Mutex` and unparks the writer (`library/std/src/sync/mpmc/waker.rs:171-173`, stable toolchain source on this machine). The critical section is a few list operations and an unpark, with no I/O, so I see no consequence on the desktop. Still, D1 ("no lock") and the Threading row ("never (no lock, no I/O ...)") should say "a std-internal lock held for O(1) list work", so that the reviewer checks a true claim (CLAUDE.md principle 1: no new locks beyond what the cut prescribes).
2. **Is a failed probe meant to last the whole run?** `attach_file` probes once at startup. A probe failure, for example `daemon.log` held without sharing at logon, keeps the whole run console-only, while the same failure one line later is retried per line. If that is not intended, attach anyway and let the writer's error-streak logic report and retry.
3. **Optional: turn a review item into a gate check.** The commit 6 scan could also reject `log::` macro calls in `src/hook/keyboard_hook_callback.rs` outside `#[cfg(test)]`. The review-only proof of "the hook callback never logs per event" then becomes a gate check for later lanes (2.1, 2.4).
4. **Process note for the tester's brief.** The reds of commits 3 and 6 replay on commits 2 and 5. On `master` both tests are compile errors, because there is no `logging` module, while WORKFLOW.md says that red-firsts are replayed on the parent (`master` in round 1). Say so in the tester's brief.
5. **`ipc_server.rs:369` breaks commit 7's gate.** Once the print at `:374` is deleted, `let path` is unused, and `clippy -D warnings` fails commit 7. Remove the binding, or use it in the ignore reason, in the same D8 row.
6. **Q3 (elevated writes into a user-writable folder).** I agree with recording this as a follow-up. The same class of risk already exists for the template write to `<exe_dir>\config.ron` when the exe sits in a user-writable folder.

## Absent topics (walked, not applicable)

- **Matching (3), suppression lifetime (4), time and gesture state (5), engine purity (6) and input synthesis (10).** The cut changes no decision logic, and `src/engine` is untouched. The three `SendInput` builders keep `injected_sentinel()`; only their short-count branch changes (`windows_actions.rs:220-228`, `system_function.rs:151-159`, `keyboard_hook_callback.rs:257-263`).
- **Thread affinity (2).** Both hooks are still installed, pumped and removed on main. `log-writer` uses no Win32 UI API and needs no message loop. `SHGetKnownFolderPath`/`CoTaskMemFree` run on main before any hook exists.
- **Shared state (8).** The new statics are a `OnceLock<SyncSender>` and atomics. Nothing logs under `with_engine`: the engine does not log, and `set_enabled` and `foreground_changed` log nothing (`keyboard_hook.rs:95-155`, `keyboard_hook_callback.rs:360-381`).
- **Config and hot reload (7).** No schema or RON change. The reload semantics stay as they are, still pinned by `a_save_with_a_misspelled_field_keeps_the_live_bindings` and `a_deleted_config_keeps_the_live_bindings`.
- **Process boundary (9).** No pipe, token or DACL change. std opens files as non-inheritable, so `Launch` children do not inherit the log handle.
- **`swch`.** `src/bin/swch.rs` calls only `ipc::request`, which does not print, so its output stays byte-identical with no logger installed.
