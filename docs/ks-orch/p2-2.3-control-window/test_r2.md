# Tester r2: p2-2.3-control-window / roadmap 2.3 (HEAD 81bb38c vs 6f6edb3, delta round)

Verdict: GREEN. Gate green on 81bb38c, the one CONFIRMED item (R-I1) is red under the product mutation at HEAD and was vacuous on its parent, 8 mutations caught by named tests, tree clean.

HEAD: `81bb38c` (`git rev-parse --short HEAD`), `git status --short` empty before and after; branch feat/control-window.

## Delta check
`git diff -U0 6f6edb3..HEAD -- src` hunks: `keyboard_hook_callback.rs` @@-19 and @@-29/-37 are `//!` docs, `main_thread.rs` @@-664 a `///` doc, `main_thread.rs` @@+1613,79 inside `mod tests`; the rest is `config_watcher.rs` `mod tests` and `engine/scenarios.rs` (test-only). The 110 non-comment changed lines are all test code. No production line changed, so no hook path / main loop / executor change: latency bench (4b) not required and not run. Grep of added lines for `SendInput|SetWindowsHook|key-switch-rs.sock|RegSetValue|RegCreate`: no match.

## Gate (HEAD 81bb38c)
Logs: `t2_fmt.txt`, `t2_clippy.txt`, `t2_test.txt`, `t2_build.txt` in the lane dir.
- cargo fmt --check: OK (exit 0)
- cargo clippy --all-targets --locked -- -D warnings: OK (exit 0). First run said "Finished in 0.08s"; sources touched and re-run: `Checking key-switch-rs ... Finished dev profile in 1.88s`, no warnings.
- cargo test --locked --no-fail-fast: exit 0.
  - lib: `running 686 tests` ... `test result: ok. 679 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 2.28s`
  - bin targets: `running 0 tests` (two, expected, no unit tests in main/swch)
  - tests/ipc_roundtrip.rs: `running 6 tests` ... `test result: ok. 6 passed; 0 failed; 0 ignored`
  - Totals: 685 passed, 0 failed, 7 ignored (same as r1: no test function added in the delta).
  - 7 ignored (all `manual:` or `bench:`): bench::latency_bench (bench), core::executor::tests::the_win32_runner_opens_a_url, hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start, hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream, platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it, platform::foreground::tests::the_foreground_cache_names_the_focused_notepad, system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination. Not run, not counted as passes.
  - The four tests the delta/previous round touched, all `ok` in the full run: `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path`, `hook::main_thread::tests::a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain`, `hook::main_thread::tests::a_return_signal_reads_the_foreground_window_again`, `engine::scenarios::win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats`.
- cargo build --release --locked: OK (exit 0, `Finished release profile in 6.27s`). Daemon not run.

## Red-first replay (CONFIRMED items of triage_r1.md: R-I1 only; R-m1, T-N1 are NOTED, T-B1..B4 and T-M* REFUTED)
R-I1 has no behavioural seam (the product line `.map_err(notify::Error::io)?` at `config_watcher.rs:158` is already correct), so the red-first is: the old test is vacuous on the parent, the new test is red under the product mutation at HEAD.

| Test | Parent 4539306 + mutation (:158 -> `.expect("spawn config-watcher thread")`) | HEAD 81bb38c + same mutation | HEAD unmutated |
|---|---|---|---|
| hook::config_watcher::tests::the_watcher_thread_has_no_panic_path | `running 1 test` ... `ok`, `test result: ok. 1 passed; 0 failed; ... 685 filtered out` (vacuous: scan ends at line 36) | `running 1 test` ... `FAILED`, panicked at `src\hook\config_watcher.rs:473:9`, message `158: .expect("spawn config-watcher thread");`, `test result: FAILED. 0 passed; 1 failed` | ok (full run, 679 passed) |

Logs: `t2_parent_mutA.txt` not kept as file; output in `t2_mut_parentA.txt`, `t2_mut_A.txt`. Mutation reverted, `git diff --exit-code` clean in both trees.
Worktrees `wt/parent` (4539306) and `wt/mut` (81bb38c) with `target-wt/{parent,mut}` removed (`git worktree remove --force` + prune); `git worktree list` shows only `D:/key-switch-rs/key-switch-rs 81bb38c [feat/control-window]`.

## Mutations (tree 81bb38c, scratch worktree, one at a time, test named before running)
| # | File | Mutation | Must be caught by | Result |
|---|---|---|---|---|
| A | src/hook/config_watcher.rs:158 | `.map_err(notify::Error::io)?;` -> `.expect("spawn config-watcher thread");` | the_watcher_thread_has_no_panic_path | caught, `:473:9` `158: .expect(...)` |
| B | src/hook/config_watcher.rs (reload_into, before `let count`) | insert `let _ = config_path.to_str().unwrap();` | the_watcher_thread_has_no_panic_path | caught, `:474:9` `199: let _ = config_path.to_str().unwrap();` |
| C (new) | src/hook/config_watcher.rs debounce_loop, after `reload_into(..)` | insert `let _ = rx.try_recv().unwrap();` (region above the first lone `#[cfg(test)]` the old scan never saw past) | the_watcher_thread_has_no_panic_path | caught, `:474:9` `185: let _ = rx.try_recv().unwrap();` |
| M12 | src/hook/main_thread.rs `answer_input_signals` | delete the `if !hook.is_enabled() { return; }` gate | a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain | caught, `main_thread.rs:1624:9` `a drain with remapping off logs and resets nothing: [".. [INFO] session locked\n", ".. [INFO] state reset: keyboard input left this desktop\n"]` |
| E (new) | same gate | invert to `if hook.is_enabled() { return; }` | same test and the_drain_logs_one_line_per_input_signal | caught by both: `2 tests`, both FAILED (`"[INFO] session locked" once in []`, left 0 right 1) |
| M13 | src/hook/main_thread.rs `answer_input_signals` | `if signals.signals().any(InputSignal::input_returned) {` -> `if false {` | a_return_signal_reads_the_foreground_window_again | caught, `:1684:13` `left: Some(Some("p4096.exe")) right: Some(Some("p8192.exe"))` |
| D (new) | src/hook/main_thread.rs `MainState::input_returned` | body `self.foreground.resync();` -> no-op | a_return_signal_reads_the_foreground_window_again | caught, same assertion `:1684:13` |
| M20 | src/engine/mod.rs `on_foreground_changed` | add `self.keys.release_all();` | win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats | caught, `scenarios.rs:1938:10` `repeat T at +500 ms: expected to fire [], fired [Launch { exe: "wt.exe", args: [] }]` |

Each run: `running N tests` with N equal to the names given (1, 1, 1, 1, 2, 1, 1, 1), exit 101. After each, `git checkout -- .` and `git diff --exit-code` clean ("reverted clean").

## Acceptance (delta scope)
| Criterion | Evidence | Status |
|---|---|---|
| Config watcher thread start failure is an error, not a panic (spec leftover of 2.2, CHANGELOG cites the test) | the_watcher_thread_has_no_panic_path ok; red under A, B, C; coverage assert fails if the scan misses `fn spawn_watcher(` | verified |
| Lock signal noted while remapping is off is dropped (r1 gap) | c7ab5c7 test ok; red under M12, E | verified |
| Return signal re-reads the foreground window (r1 gap) | 4539306 test ok; red under M13, D | verified |
| Win+Alt+T held across a focus change does not launch again on repeats (r1 gap) | 7ab78ab scenario ok; red under M20 | verified |
| R-m1 docs name reload request, input-signal step, cached executable name | doc-only (81bb38c); `//!` lines read against `drain`/`HookEnv`; source-scanning tests green in the full run | verified by reading |

Not re-verified here (unchanged since r1, outside the delta): the rest of roadmap 2.3 acceptance and manual checks (Win+L, Ctrl+Alt+Del timestamps, V2-m1 / M1 stay with the owner: agents may not run the daemon).

## Failures
None.

## Tests added by the tester
None (no `Tests:` commit this round; tree clean).

## Notes
- Timing: T-N1 (`stopped < ms(100)` against a real-clock 500 ms Delay) passed in this full run; not re-run 5 times (NOTED, optional, not in the delta).
- The scan test now genuinely depends on its coverage assert: on the parent it would pass under mutation A; at HEAD it fails. The assert message names the line where the scan ends.
- No test in the delta touches the production pipe name, a real config.ron or the registry. `note_input_signal` in the c7ab5c7/4539306 tests posts a thread message to the test's own thread only.
