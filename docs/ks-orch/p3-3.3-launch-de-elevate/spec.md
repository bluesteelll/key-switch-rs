# Lane spec: p3-3.3-launch-de-elevate - `Launch` and `OpenUrl` without elevation (roadmap item 3.3; KS-09, KS-34)

Branch: fix/launch-de-elevate

Roadmap item 3.3 (book/src/roadmap.md '`Launch` and `OpenUrl` without elevation', :698-706 at 252daef; syntax plan :871;
'Nothing runs silently as administrator when Explorer is unavailable', :935). Binding: the owner's answers to plan_draft.md
section 5 Q4 (uiAccess, roadmap open question 3), Q5 (what `elevated: true` does), Q6 (`OpenUrl` and `elevated`) and Q7
(the default working directory), relayed in the brief; the latency decision: no new per-event work on the key path, and a
`Launch`/`OpenUrl` fire must not allocate more on the hook path than today. Scope report:
<orch>/phase-3/scope/3.3-launch.md (sections 1-9).

Before the cut: the researcher writes <orch>/p3-3.3-launch-de-elevate/research.md (the scoping round's
research-3.3.md was never written). Questions, each with primary sources (Microsoft Learn, The Old New Thing, e.g.
https://devblogs.microsoft.com/oldnewthing/20131118-00/?p=2643): how `IShellDispatch2::ShellExecute` reports a missing
program and whether it returns before the child starts; the child's working directory and environment when `vDir` is
empty; the `runas` verb through Explorer; `CreateProcessW` with `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` as the
alternative (handles, errors, PID); the uiAccess flag and child tokens; prior art (AutoHotkey `Run`, PowerToys).

1. A new `src/platform/launch.rs` starts programs at the shell's integrity level. Route: the roadmap's Explorer chain
   (`IShellWindows::FindWindowSW(SWC_DESKTOP)`, `IShellBrowser`, `IShellView`, `IShellFolderViewDual`,
   `IShellDispatch2::ShellExecute`) on the executor's STA (`src/core/executor/mod.rs:377-392`), unless the research
   finds a blocker; the cut decides (scope fork T1) and says how a missing program is reported. It is called only on the
   executor thread, from `Win32Runner::action` (`src/core/executor/mod.rs:126-148`) and `sequence::run_step`
   (`src/data/sequence.rs:113-114`), and replaces `launch_process` and `shell_open` (`src/core/windows_actions.rs:312-351`).
   One route for elevated and non-elevated daemons, so the debug build exercises it (scope T5). Without Explorer the
   action is refused with one `[ERROR]` line and nothing runs (roadmap :935; no fallback).
2. `Launch` gains `cwd:` (absolute; a relative one is a load error naming the field; not checked for existence, as `exe`
   is not) and `elevated:` (default `false`; Q5 decides what `true` does), as an action and as a macro step:
   `src/config/schema.rs:193-198`, :242-246; `src/config/loader.rs:716-721`, :765-770. Without `cwd:` the child starts in
   Q7's folder. `OpenUrl` changes only if Q6 is yes (a RON newtype, schema.rs:199-201, :247, needs a second accepted
   form). Variant shape: `src/core/windows_actions.rs:35-42`, `PartialEq` :108, `Display` :364-370;
   `src/data/sequence.rs:52-53`, `Display` :91-97 (R6). The launch payload sits behind an `Arc`, so the engine's clones
   (`src/engine/mod.rs:846`, `gestures.rs:217-226`, `chords.rs:121`, `sequences.rs:119`, :128) do not allocate; it must
   not change the type of `Decision.fire` (`src/engine/mod.rs:396`), which p3-3.4-zero-alloc owns.
3. Console and inheritance (KS-34): a console program gets its own console; the child inherits neither the daemon's
   console, working directory nor handles; the single-instance mutex and the pipe stay non-inheritable
   (`src/system/single_instance.rs:46-50`). If Q5 keeps a direct elevated spawn, it uses `CREATE_NEW_CONSOLE`, null
   stdio and `current_dir`, and no daemon handle becomes inheritable.
4. `Cargo.toml:41-59` gains the `windows` features `Win32_System_Ole` and `Win32_System_Variant` (named by the roadmap,
   :704; `IShellWindows::FindWindowSW` needs both in windows 0.62.2). No new dependency.
5. Follow-up 'a step that blocks inside Windows holds up the executor' (roadmap :980): measure how long one launch holds
   the executor (row R25, manual M6) and state it in the CHANGELOG and actions.md section 13.1 (:51). A separate
   launcher thread is the architect's fork; if the cut does not take it, it names the row's new home.

   Note (lane feat-clipboard-paste, merged before Phase 3): the executor runs jobs in firing order behind a `Paste`
   that waits (for released modifiers, a busy clipboard or the restore of the paste before it): the jobs fired after
   it are held (`Runs::held` and the head rule, `src/core/executor/runs.rs`; known issue 97). A launcher thread keeps
   that order: a `Launch` fired after a waiting `Paste` starts after it, never before, or its window can take the
   foreground the paste checks before its keystroke, and the paste is skipped. A `Launch` that blocks the executor
   while the paste session's window owns the clipboard can also hold up another program's copy (known issue 95,
   not verified); a launcher thread would take that wait off the executor.
6. Docs of the trust boundary (scope F-i): after this lane a plain `Launch` runs at medium integrity, but whoever can
   write `config.ron` still reaches elevated code through `elevated: true` (if Q5 is silent) and through `Sequence`
   input typed into elevated windows (from code reading). Say that the risk narrows and does not close
   (build-and-run.md :311-314 at the book sync); record a follow-up for item 4.4, which moves the config to a folder
   that medium-integrity code can write.

Acceptance:
- Loader unit: `cwd` and `elevated` parse as action and as step; the defaults hold; a relative `cwd` is a load error
  that names the field; the `cwd` row of `unknown_fields_are_errors_that_name_the_field` (`loader.rs:1553-1555`) is
  replaced by another misspelling.
- Unit: the route decision is a pure table (elevated wanted, daemon elevated, Explorer found -> Explorer open /
  Explorer `runas` / direct spawn / refuse); the command line built from `args` round-trips through
  `CommandLineToArgvW` (parse only) for spaces, quotes, trailing backslashes and empty args; the `Display` arms (R6).
- Unit: cloning a `BindAction::Launch` shares its payload (`Arc::ptr_eq`).
- Unit: the executor's FakeRunner tests stay green, and the Launch test (`src/core/executor/runs.rs:531-547`) shows
  `cwd` and `elevated` in its text. `held_launch_and_sequence_hotkeys_fire_once_per_press` (scenarios.rs:2899-2902)
  stays green after the helper update.
- Unit (scan): outside tests, `Command::new` and `ShellExecuteW` occur only in `src/platform/launch.rs` (if a direct
  spawn stays), `src/bin/swch.rs` and the unreachable `src/system/system_function.rs` (new feature 1).
- Bench (counted apart): R24, the F13 `Launch(exe, 3 args, cwd)` key-down through the callback with a FakeRunner
  executor attached: H p50 <= P p50; R25, the Explorer lookup chain without `ShellExecute` and without starting a
  process (evidence, interactive desktop needed); R1-R18 and the R19 family do not regress.
- Manual (owner, later batch): M1 `Launch(exe: "cmd.exe", args: ["/k", "whoami /groups"])` from the release daemon opens
  its own console and reports `Medium Mandatory Level`, in Q7's folder; M2 `OpenUrl("https://example.com")` with no
  browser running starts it not elevated (Task Manager, Details, Elevated = No); M3 `elevated: true` reports
  `High Mandatory Level` (after a UAC prompt if Q5 = prompt); M4 `cwd:` is honoured; M5 with Explorer stopped the
  action is refused and logged, and works after Explorer restarts; M6 the executor block of one launch (F13 `Launch`
  notepad, then within 100 ms F16 `Sequence([Text("x")])`; note when `x` appears); M7 `Launch(exe: "regedit.exe")`
  without `elevated` shows a UAC prompt; M8 a debug daemon behaves as M1.
- Gate green.

Latency: no change to per-event work, locks, Win32 calls, timers or wake-ups on the key path; the firing thread still
makes no Win32 call for `Launch`/`OpenUrl` (`TargetWindow::for_action` returns `NONE`, executor/mod.rs:93-99). A fire
allocates nothing for its payload (today 2 + n allocations for a `Launch` with n args, 1 for `OpenUrl`). Executor side:
the cross-process COM calls to Explorer per launch, measured by R25 and declared; `swch exit` stays bounded by
`STOP_TIMEOUT` (executor/mod.rs:66).

Docs: CHANGELOG `[Unreleased]`: Fixed (KS-09, KS-34; roadmap 3.3); Added (`cwd`, `elevated`); Changed, **Breaking:**
children run at medium integrity, a console child gets its own console, the default working directory, the child takes
Explorer's environment and lookup, std's `.bat` argument check is gone, an admin-manifest target prompts, error texts.
README.md (:369-370, :614-615, :927-928; :1012-1015 if Q4 changes the deployment); `src/config/default_config.ron`
(:95-100, :162-163, :265-268); `book/src/config/actions.md` sections 13.6 (:173-205), 13.7 (:207-229), 14.7
(:379-381), 13.1 (:51); `structure.md:191-199` (a new unknown-field example); `errors.md` (:69, :85, :132, :195-198, the
refusal line); `known-issues.md` rows 32-33 (:76-77); `example.md` (:98-101, :205-215, :235); every RON example through
`cfgcheck`. Book-sync list in impl.md with file:line: architecture.md :573-575, :609; build-and-run.md :311-314 (item 6);
the audit status lines of KS-09 and KS-34; roadmap 3.3 status, :422, :888, :918, :980 and open question 3 (:942);
gaps.md :285, :302.
