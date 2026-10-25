# Context pack r2: p2-2.3-control-window (tester + reviewer)

Read at: work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/control-window`, HEAD `81bb38c`
(`git rev-parse --short HEAD`). Round 2 is delta-scoped: `6f6edb3..81bb38c`. Inputs: `fix_r1.md`,
`triage_r1.md`, `review_r1.md`, `test_r1.md`, `rulings.md`, the developer's raw logs `fix_r1_*.txt`.

## Scope and delta

`git log --format='%h %p %s' 6f6edb3..feat/control-window` (each commit's parent second):

| Commit | Parent | Author role | Subject |
|---|---|---|---|
| `c7ab5c7` | `6f6edb3` | tester (r1) | Tests: a signal noted before remapping went off is dropped by the drain |
| `7ab78ab` | `c7ab5c7` | tester (r1) | Tests: a Win+Alt+T held across a focus change does not launch again on its repeats |
| `4539306` | `7ab78ab` | tester (r1) | Tests: a return signal makes the drain read the foreground window again |
| `c4f1bca` | `4539306` | developer (fix r1, R-I1) | Tests: the watcher's panic-path scan covers every line before its tests module |
| `81bb38c` | `c4f1bca` | developer (fix r1, R-m1) | Hook: the loop docs name the reload request, the input-signal step and the cached executable name |

`git diff --stat 6f6edb3..feat/control-window`:

```
 src/engine/scenarios.rs            | 26 +++++++++++
 src/hook/config_watcher.rs         | 25 ++++++++--
 src/hook/keyboard_hook_callback.rs |  6 ++-
 src/hook/main_thread.rs            | 93 ++++++++++++++++++++++++++++++++++++--
 4 files changed, 138 insertions(+), 12 deletions(-)
```

Hunks (`git diff -U0 6f6edb3..feat/control-window | grep '^@@'`), new-side line numbers:
- `src/engine/scenarios.rs` `+1921,26`: whole file is test-only (`#[cfg(test)] mod scenarios;` at `src/engine/mod.rs:240-241`).
- `src/hook/config_watcher.rs` `+445,6`, `+455,14`, `-454` (old `take_while` line): all inside `mod tests` (`:215-216` opens it, ends `:506`).
- `src/hook/keyboard_hook_callback.rs` `+19,4`: module doc comment `//!` only (`mod tests {` at `:457`).
- `src/hook/main_thread.rs` `+29,4`, `+39,2`: module doc `//!`; `+667,3`: `///` doc of `drain`; `+1613,79`: inside `mod tests` (opens `:798`).

## File map

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/hook/config_watcher.rs:37-38` | `#[cfg(test)] use crate::hook::keyboard_hook::KeyboardHook;` | first lone `#[cfg(test)]` line; the old scan stopped here (R-I1) |
| `src/hook/config_watcher.rs:54-62` | `#[cfg(test)] impl ApplyReload for KeyboardHook` | test-only impl above the tests module, now inside the scanned region |
| `src/hook/config_watcher.rs:126-161` | `spawn_watcher`; `:154-158` `thread::Builder::new()...spawn(..).map_err(notify::Error::io)?;` | product line the scan must cover (mutation target A at `:158`) |
| `src/hook/config_watcher.rs:163-186` | `debounce_loop` | now in the scanned region |
| `src/hook/config_watcher.rs:190-213` | `reload_into` | now in the scanned region (mutation target B after `:199`) |
| `src/hook/config_watcher.rs:443-479` | doc + `the_watcher_thread_has_no_panic_path` | the R-I1 fix (c4f1bca) |
| `src/hook/keyboard_hook_callback.rs:16-22` | module doc step 5 | R-m1 doc fix (81bb38c) |
| `src/hook/keyboard_hook_callback.rs:161-184` | `impl Env for HookEnv`: `os_key_down` `:167-173` (`GetAsyncKeyState` `:171`), `foreground_app` `:175-177` (`self.foreground.app()`), `foreground_title` `:179-183` (`read_title` lazily via `OnceCell`) | what the step-5 doc now describes |
| `src/hook/keyboard_hook_callback.rs:1254-1262` | `non_test_lines` helper (the `#[cfg(test)]`+`mod tests {` pair, `unwrap_or(lines.len())`) | the pattern c4f1bca copies (it uses `.expect` instead of `unwrap_or`) |
| `src/hook/main_thread.rs:26-41` | module doc, loop steps 1-4 | R-m1 doc fix (81bb38c) |
| `src/hook/main_thread.rs:96-175` | `PENDING_SIGNALS`, `PendingSignals`, `note_input_signal` `:134-147`, `note_unknown_desktop_switch` `:151-158`, `take_pending_signals` `:160-165`, `wake_this_thread` `:170-175` | product code the tester tests exercise (unchanged in the delta) |
| `src/hook/main_thread.rs:220-226` | `MainState::set_enabled` | used by the c7ab5c7 test |
| `src/hook/main_thread.rs:263-275` | `foreground_changed`, `input_returned` (`self.foreground.resync()`) | used by the 4539306 test |
| `src/hook/main_thread.rs:614-616` | `MainThread::refresh_foreground` | ruling M3 context |
| `src/hook/main_thread.rs:667-691` | `drain` doc + body: requests `:673-676`, `take_pending_signals` -> `answer_input_signals` `:677-680`, due timers `:681-684` | R-m1 doc names this order |
| `src/hook/main_thread.rs:706-736` | `answer_input_signals`: enabled gate `:715-717`, per-signal log `:718-720`, unknown warn `:721-726`, reset `:727-731`, return re-read `:732-734` | mutation targets M12 (`:715-717`) and M13 (`:732`) |
| `src/hook/main_thread.rs:1459-1520` | `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key`; `stopped < ms(100)` at `:1516-1519` | T-N1 (triage NOTED) |
| `src/hook/main_thread.rs:1581-1611` | `the_drain_logs_one_line_per_input_signal` | sibling test, pre-existing |
| `src/hook/main_thread.rs:1613-1634` (fn at `:1614`) | `a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain` (c7ab5c7) | kills M12 |
| `src/hook/main_thread.rs:1636-1653` | test-only `struct MovingQuery` + `impl WindowQuery` (`foreground`, `owner`, `image_name` -> `p{pid}.exe`, `title` -> `None`) | helper of the 4539306 test |
| `src/hook/main_thread.rs:1655-1689` | `a_return_signal_reads_the_foreground_window_again` (4539306), loops `SessionUnlocked`, `DesktopSwitchedBack` | kills M13 |
| `src/engine/mod.rs:608-610` | `Engine::on_foreground_changed` -> `self.keys.sync_modifiers(env, None)` | mutation target M20 |
| `src/engine/keys.rs:148` | `KeyState::release_all` | the call M20 inserts |
| `src/engine/scenarios.rs:1897-1919` | `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (pre-existing acceptance test) | asserts suppression only (test_r1.md:60) |
| `src/engine/scenarios.rs:1921-1945` | `win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` (7ab78ab) | kills M20 |
| `CHANGELOG.md:874-879` | `[Unreleased]` entry naming `the_watcher_thread_has_no_panic_path` as proof of the watcher fix | R-I1 makes the cited proof non-vacuous; entry unchanged in the delta |

## Facts

1. No production (non-test, non-comment) line changes in `6f6edb3..81bb38c`: every hunk is in a `mod tests`, in `src/engine/scenarios.rs` (test-only, `src/engine/mod.rs:240-241`), or a `//!`/`///` doc line (hunk list above).
2. `src/hook/config_watcher.rs:37` is `#[cfg(test)]` followed by `:38` `use crate::hook::keyboard_hook::KeyboardHook;`; `:56` is `#[cfg(test)]` before `impl ApplyReload for KeyboardHook` (`:57-62`); `:215-216` is the `#[cfg(test)]` + `mod tests {` pair.
3. At `4539306` the test filtered with `.take_while(|(_, line)| line.trim() != "#[cfg(test)]")` (removed line, old `:454`), so it scanned lines 1-36 (`triage_r1.md`, probe `triage_probe/scan.py`: `head scanned lines 1.. 36 sites: []`).
4. At `81bb38c` the test (`src/hook/config_watcher.rs:452-479`) collects `lines`, finds `tests_at` as the first `windows(2)` pair with trimmed `#[cfg(test)]` then `mod tests {` (`:456-459`, `.expect("src/hook/config_watcher.rs has a tests module")`), sets `region = &lines[..tests_at]` (`:460`), asserts `region` contains `concat!("fn spawn", "_watcher(")` with message "the scan of src/hook/config_watcher.rs ends after line {} and misses `{spawner}`" (`:461-466`), then filters `region` for `concat!(".exp", "ect(")` / `concat!(".unw", "rap()")` (`:453`, `:467-472`) and asserts none (`:473-478`).
5. `src/hook/config_watcher.rs:158` is `.map_err(notify::Error::io)?;`; lines 1-214 hold no `.expect(` or `.unwrap()` (triage_r1.md, "The product fix itself is correct").
6. `src/hook/keyboard_hook_callback.rs:175-177`: `foreground_app` returns `self.foreground.app()` (no Win32 call in the callback); `:171` `GetAsyncKeyState`; `:181` `self.foreground.read_title()` (`src/platform/foreground.rs:156`).
7. `src/hook/main_thread.rs:670-691` `drain` runs, per pass: `self.requests.try_recv()` -> `serve` (`:673-676`), `take_pending_signals()` -> `answer_input_signals` (`:677-680`), `with_main(take_due_timer)` -> `carry_out` (`:681-684`); exits when no step was busy (`:685-687`). `serve` handles `Request::SetEnabled` and `Request::Reload` (`:751-770`).
8. `src/hook/main_thread.rs:715-717`: `answer_input_signals` returns before logging when `!hook.is_enabled()`; `:727-731` logs `state reset: {reason}` after `reset_transient_state`; `:732-734` calls `MainState::input_returned` when any signal has `input_returned`.
9. `note_input_signal` (`src/hook/main_thread.rs:134-147`) sets a thread-local flag and calls `wake_this_thread` (`:170-175`, `PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE, ..)`, a post to the test's own thread queue; no hook, no injection).
10. The c7ab5c7 test builds `MainThread::new()` and a private `KeyboardHook::new()` (`src/hook/main_thread.rs:1618-1619`), notes `SessionLocked` (`:1620`), asserts `with_main(|s| s.set_enabled(&hook, false)) == Some(true)` (`:1621-1625`), captures the drain with `crate::logging::testing::capture` (`:1626`), asserts no line contains "session locked" or "state reset" (`:1627-1632`) and `take_pending_signals() == None` (`:1633`).
11. The 4539306 test replaces `state.foreground` with `ForegroundCache::with_query(Box::new(MovingQuery(..)))` and calls `state.foreground_changed(HWND(0x1000))` (`src/hook/main_thread.rs:1667-1672`), moves the window to `0x2000` (`:1673`), asserts the cache still names `p4096.exe` (`:1674-1678`), notes the signal and drains (`:1681-1682`), asserts `p8192.exe` (`:1684-1688`). `MovingQuery` issues no Win32 window query (`:1639-1653`).
12. The 7ab78ab scenario (`src/engine/scenarios.rs:1921-1945`) binds `Win+Alt+T` -> `Launch(exe: "wt.exe")`, presses Win, Alt, T (T suppressed, fires `launch("wt.exe")`), `foreground_changed()`, then two repeats of T at +500 ms and +533 ms each `expect_suppressed().expect_fired(&[])`, then `up("T").expect_suppressed()`.
13. `Engine::on_foreground_changed` (`src/engine/mod.rs:608-610`) calls only `self.keys.sync_modifiers(env, None)`.
14. `81bb38c` doc text: `src/hook/main_thread.rs:28-32` step 1 lists "`swch on` and `off`, a config reload", then "the input signals noted since the last pass", then gesture timers; `:38-40` step 4 names "a request or a noted input signal waits"; `:667-669` `drain` doc names "answers the noted input signals"; `src/hook/keyboard_hook_callback.rs:16-22` step 5 names the cached executable name "with no call" and the title and OS key state as "live Win32 calls".
15. Tester r1 totals at `4539306` (`test_r1.md:10-15`, `gate_test_final.txt:693`): lib `running 686 tests`, `679 passed; 0 failed; 7 ignored`; `tests/ipc_roundtrip.rs` 6 passed; total 685 passed. `c4f1bca` and `81bb38c` add no test function (Fact 1, diff), so the lib count stays 686 / 679 passed / 7 ignored. UNVERIFIED at `81bb38c` (not run by this scout).
16. The 7 ignored tests are listed at `test_r1.md:15`; none is added in the delta.
17. Developer r1 gates at `81bb38c` (`fix_r1.md`, "Gates"): fmt, check, clippy pass; ran `hook::config_watcher` (11 passed, `fix_r1_green.txt`) and `hook::main_thread` + `hook::keyboard_hook_callback` + `logging` (87 passed, 1 ignored, `fix_r1_docs_tests.txt`). The full suite was left to the tester.
18. The r1 worktrees `wt/parent`, `wt/mut` and `target-wt/*` named in `mut.sh` and `test_r1.md:20` are absent: `D:/tmp/ks-orch/p2-2.3-control-window/` holds no `wt/` (ls), and `git worktree list` shows only the main tree at `81bb38c`. `mutate.py` (in the lane dir) replaces exactly one occurrence of `old` (CRLF fallback) and exits 3 otherwise.
19. Working copy files are CRLF, blobs LF (`core.autocrlf=true`, `fix_r1.md` "Open points"); a mutation string with a newline must be matched as CRLF in the work tree (`mutate.py` handles it).

## Red-first tests and acceptance proofs (commands and expected results)

Common command shape (as in `mut.sh`): in a scratch worktree of the named commit,
`cargo test --locked --lib -- --exact <full::test::name>`, with its own `CARGO_TARGET_DIR`; check `running N tests` equals the number of names given and read the `test result:` line. None of these tests installs a hook, injects input or writes the registry (Facts 9-11; `test_r1.md:16` safety scan).

### P1. R-I1, `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path` (c4f1bca)

No new seam: the red-first is a mutation proof on the product line plus a vacuity proof on the parent.

| # | Tree | Mutation | Expected | Developer's record |
|---|---|---|---|---|
| a | `4539306` (parent) | `src/hook/config_watcher.rs:158` `.map_err(notify::Error::io)?;` -> `.expect("spawn config-watcher thread");` | old test **ok** (vacuous; scan ends at `:36`) | `fix_r1.md` "Vacuity at 4539306" |
| b | `c4f1bca` or `81bb38c` | none | **ok** (`1 passed`) | `fix_r1_greenB.txt` |
| c | `c4f1bca` or `81bb38c` | mutation of row a | **FAILED**, panic at `src\hook\config_watcher.rs:473:9` (`:473` = the `assert!` of the empty-sites check; line shifts by 0 since `:158` is a one-line swap), message `panic paths in src/hook/config_watcher.rs:\n158: .expect("spawn config-watcher thread");` | `fix_r1_mutA.txt` |
| d | `c4f1bca` or `81bb38c` | insert line `    let _ = config_path.to_str().unwrap();` after `:199` (`let count = new_bindings.len();`) in `reload_into` | **FAILED**, panic at `:474:9`, message `panic paths in src/hook/config_watcher.rs:\n200: let _ = config_path.to_str().unwrap();` | `fix_r1_mutB.txt` |
| e | uncommitted step A (c4f1bca's coverage assert with the old `take_while` region) | none | **FAILED** at `:456:9`, `the scan of src/hook/config_watcher.rs ends after line 36 and misses \`fn spawn_watcher(\`` | `fix_r1_redA.txt` (intermediate state; no commit holds it) |
| f | `81bb38c` | none | `hook::config_watcher` filter: `11 passed; 0 failed` | `fix_r1_green.txt` |

Revert each mutation; `git diff` must then show nothing in the scratch tree.

### P2. Tester r1 mutation kills (product correct on every parent; proofs are mutations, not parent reds)

The product code these tests pin is unchanged since `6f6edb3` (Fact 1), so each test is green on its parent's product with the test cherry-picked; the evidence of non-vacuity is red under the named mutation, green without.

| Commit / test | Mutation (tree `81bb38c`) | Expected under mutation | Record |
|---|---|---|---|
| c7ab5c7 `hook::main_thread::tests::a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain` | M12: delete `src/hook/main_thread.rs:715-717` (`if !hook.is_enabled() { return; }`) | FAILED, message `a drain with remapping off logs and resets nothing: [".. [INFO] session locked\n", ".. [INFO] state reset: keyboard input left this desktop\n"]` (panic line was `:1620` at `c7ab5c7`; `:1627` at `81bb38c`, UNVERIFIED) | `mut_M12b.txt` |
| 4539306 `hook::main_thread::tests::a_return_signal_reads_the_foreground_window_again` | M13: `src/hook/main_thread.rs:732` `if signals.signals().any(InputSignal::input_returned) {` -> `if false {` | FAILED, `the drain of session unlocked reads the foreground window again`, `left: Some(Some("p4096.exe")) right: Some(Some("p8192.exe"))` (panic `:1680` at `4539306`; `:1684` at `81bb38c`, UNVERIFIED) | `mut_M13b.txt` |
| 7ab78ab `engine::scenarios::win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` | M20: add `self.keys.release_all();` in `src/engine/mod.rs:608-610` `on_foreground_changed` | FAILED at `src\engine\scenarios.rs:1938:10`, `repeat T at +500 ms: expected to fire [], fired [Launch { exe: "wt.exe", args: [] }]` | `mut_M20b.txt` |

Under M20 the pre-existing `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (`src/engine/scenarios.rs:1897`) stays ok; `capslock_held_across_a_focus_change_switches_the_language_once` (`:2547`) and `engine::properties::every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` (`src/engine/properties.rs:381`) go red (`test_r1.md:60`).

### P3. R-m1 (81bb38c), doc only

No runtime test. Proof is reading: Fact 14 against the code of Facts 6-8. The source-scanning tests over `src/hook` and `logging` passed after the edit (`fix_r1_docs_tests.txt`: `87 passed; 0 failed; 1 ignored`, filter `hook::main_thread`, `hook::keyboard_hook_callback`, `logging`).

### P4. Full gate at `81bb38c` (tester owns it this round)

`cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked` (expected per Fact 15: lib 686 tests, 679 passed, 7 ignored; `ipc_roundtrip` 6 passed). Never `--ignored` / `--include-ignored`.

## Open points from `triage_r1.md`

| ID | Triage verdict | State at `81bb38c` | Where |
|---|---|---|---|
| R-I1 | CONFIRMED (Important) | fixed in `c4f1bca` (test only) | `triage_r1.md` R-I1; P1 above |
| R-m1 | NOTED (Minor) | fixed in `81bb38c` (docs) | `triage_r1.md` R-m1; Fact 14 |
| T-N1 | NOTED (optional) | unchanged: `stopped < ms(100)` against a 500 ms `Delay` on the real clock | `src/hook/main_thread.rs:1516-1519`; `fix_r1.md` "T-N1"; passed 7 of 7 tester runs (`triage_r1.md`) |
| T-B1 | REFUTED; carry-over | R11 witness read +0.6 us (HEAD p50 5.4 vs C11 4.8) against the `cut.md:448` "~0.1 us" and ruling R1 "~100 ns"; to be stated in `results.md` by the results analyst | `test_r1.md:98`, `:105`; `triage_r1.md` T-B1 |
| T-B2, T-B3, T-B4 | REFUTED | bench bounds hold on the official set or re-run | `triage_r1.md`; `bench/` |
| T-M12, T-M13, T-M20 | REFUTED (closed by tester commits) | P2 above | `triage_r1.md` |
| T-M16 | REFUTED (equivalent mutant) | `Ids::next_seq` never reset (`src/engine/mod.rs:332-335`), `reset` clears tables (`:643-645`), bump at `:654` | `triage_r1.md` T-M16 |
| DEFERRED | 0 | - | `triage_r1.md` "Counts" |

Also open outside the triage: `review_r1.md:90-91` V2-m1 / manual check M1 (Win+L and Ctrl+Alt+Del timestamps) is deferred to the owner, since agents may not run the daemon; the review asks the orchestrator to confirm the deferral.

## Invariants in play

- R1 hook callback: the delta touches only the step-5 doc of `src/hook/keyboard_hook_callback.rs:16-22`; `HookEnv` at `:161-184` unchanged. `keyboard_hook_callback.rs` names no `log::` outside tests (CLAUDE.md principle 1) - no new line in the delta.
- R3 shared state: tests reach engine state only through `with_main` (`src/hook/main_thread.rs:199`) on the test thread's own `MainThread::new()` (`:1618`, `:1665`).
- R10 message loop: `wake_this_thread` posts to the calling thread (`src/hook/main_thread.rs:170-175`); `drain` order `:670-691`.
- Timers / KS-31: the reset path `reset_transient_state` (`src/hook/main_thread.rs:233-236`) and `Engine::reset` generation bump (`src/engine/mod.rs:654`, T-M16).
- KS-33-like spec leftover "config_watcher.rs:126 .expect becomes an error path": `src/hook/config_watcher.rs:158`, pinned by P1.
- Focus change / KS-07 adjacent: `Engine::on_foreground_changed` resyncs modifiers only (`src/engine/mod.rs:608-610`), pinned by the 7ab78ab scenario.

## Not covered

- Did not run any cargo command (counts at `81bb38c` in Fact 15 are inferred from r1 plus the diff).
- Did not re-read the product files outside the line ranges above, the r1 product delta `master..6f6edb3` (covered by `pack_r1.md`), `cut.md`, `critique*.md`, `impl.md`, the bench tables, the book or `CHANGELOG.md` beyond `:872-880`.
- Panic-line numbers for M12/M13 at `81bb38c` are computed from the test's position (`UNVERIFIED`); the r1 logs give the `c7ab5c7` / `4539306` positions.
