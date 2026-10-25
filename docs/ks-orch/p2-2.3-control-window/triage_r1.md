TRIAGE: 1 confirmed, 8 refuted, 0 deferred

# Triage r1: p2-2.3-control-window @ 4539306 (developer HEAD 6f6edb3 + 3 tester `Tests:` commits)

Inputs: `test_r1.md` (tester, GREEN, with 4 bench flags and 4 mutation rows marked not caught), `review_r1.md` (CHANGES_REQUESTED, Important 1, Minor 1), `spec.md`, `rulings.md`, `cut.md` D14/D15.
Evidence read through git only (`git show feat/control-window:<path>`, `git diff dd53797 6f6edb3`, `git show <tester commit>`), the bench tables in `bench/`, and the tester's mutation logs. Scratch probe: `triage_probe/scan.py` replays the scan test's `take_while` on the HEAD text with and without the mutation. It runs no cargo and no test. No code was edited, and `cargo test` was not run.

## Verdict lines

- **R-I1 CONFIRMED** (review I1, Important): `the_watcher_thread_has_no_panic_path` cannot fail at HEAD.
  - The test keeps the lines before the first line whose trimmed text is `#[cfg(test)]` (`src/hook/config_watcher.rs:454`). The first such line is now the test-only import at `:37` (added by 4fbcc37, C17). At 5388175/5bf7f12 it was the `mod tests` attribute at `:172`/`:175`. So the scan covers lines 1-36 only, and `spawn_watcher` (`:126-162`), `debounce_loop` and `reload_into` are never scanned.
  - Probe: `python -I triage_probe/scan.py` gives `head scanned lines 1.. 36 sites: []`. With `.map_err(notify::Error::io)?;` at `:158` replaced by `.expect("spawn config-watcher thread");`, it gives `mutated scanned lines 1.. 36 sites: []`, although the mutated line is present at 158.
  - No other test covers this file. The `src/hook` thread scan in `keyboard_hook_callback.rs` excludes the config watcher, and a failed spawn cannot be induced in a test.
  - `CHANGELOG.md:874-879` names this test as the proof of the spec leftover "config_watcher.rs:126 .expect ... becomes an error path".
  - The product fix itself is correct: `:158` uses `map_err(..)?` and lines 1-214 contain no `.expect(` or `.unwrap()`. Only the pin is vacuous.
  - Fix direction (the reviewer's, endorsed): end the region at the `#[cfg(test)]` + `mod tests {` pair, as `non_test_lines` (`keyboard_hook_callback.rs:1254-1261`) does. Assert that the region contains `fn spawn_watcher(`, and replay the mutation red.
- **R-m1 NOTED** (review m1, Minor): accurate.
  - `main_thread.rs:28-30` step 1 still lists only "the requests (`swch on` and `off`), then every gesture timer"; step 4 is at `:36-37`.
  - `drain`'s doc (`:665`) says "serves the requests and carries out the due timers", while the body (`:673-676`) also runs `take_pending_signals` → `answer_input_signals` and serves `Request::Reload`.
  - The adapter's step 5 (`keyboard_hook_callback.rs:18-19`) still says "live Win32 calls", while `HookEnv::foreground_app` reads the cache (`:175-177`).
  - Doc only, with no runtime failure.
- **T-B1 REFUTED** (tester bench flag 1, R11 +0.6 us against the D14 allowance of +0.5 us).
  - The breach is real (HEAD p50 5.4 in 5 of 6 runs, C11 4.7-4.9). The governing rule does not make it RED. `tester.md:93` says "A breach is RED unless the cut justified that extra work". `cut.md:421` says "Justified extra work on the hook path: R11's one read", and ruling R1 accepts the read.
  - The extra cost is that read alone:
    - `src/bench.rs` R11 is unchanged between dd53797 and HEAD (the diff adds only R13 and `refresh_foreground` before R9).
    - On the key-up path, the only added work is `decision.suppress = swallow && !env.os_key_down(vk)` (`engine/mod.rs:885`), which is one `GetAsyncKeyState` (`keyboard_hook_callback.rs:165-170`).
  - Within R1 there is no cheaper mechanism: the read already runs only on the key-up of a swallowed key.
  - Carry-over for `results.md`, not a fix round: the witness costs about 0.6 us, about 6 times the "~0.1 us" of `cut.md:448` and the "~100 ns" premise of ruling R1. State the measured number.
- **T-B2 REFUTED** (tester bench flag 2, R9 against 0.5 x C11).
  - On the official set the D14 bound (`cut.md:410`) holds: 4.9 <= 0.5 x 9.9 = 4.95. Its second clause (<= HEAD R12 + 2 = 6.8) holds with margin.
  - The extra set's miss comes from C11's bimodal process query (C11 R9 p50s 9.9/7.8/11.5/7.3/7.4/7.4). It is not a HEAD cost. HEAD R9 equals HEAD R12 (4.8) in 4 of 6 runs, so the cache removed the whole query. A ratio below the unbound-key floor cannot be reached.
  - The spec's acceptance "the per-HWND cache shows as a lower hook-path cost" holds: about 4.85 against about 7.6 over the 6-run medians.
- **T-B3 REFUTED** (tester bench flag 3, R12 p99 40.7).
  - The 2.2 D10 rule (`p2-2.2-timers/cut.md:194`), which D14 inherits, reads "a breach is a defect unless ... a re-run within bounds".
  - The re-run exists (`bench/head_4..6.md`): R12 p99 5.6/7.9/7.1, all within 1.5 x 17.4 + 2 = 28.1. The official runs 2 and 3 also pass (6.4, 9.0).
  - Run 1 was inflated on every row (R4 p50 5.3, R7 14.6, R11 7.1, R13 17.1).
  - The unbound-key path gained no work from C11 to HEAD: `decide_event` only destructures `MainState`, and the C7/C8 own-combo arm visits pending presses, of which R12 has none. p50 is 4.8 on both.
- **T-B4 REFUTED** (tester bench flag 4, R10): official HEAD 6.5 <= C11 6.7 + 0.5 = 7.2 (`cut.md:411`). The bimodal 5.1/6.3 appears on both trees, and the title read (`GetWindowTextW`) is the same call as at C11 and is named as justified (`cut.md:421`).
- **T-M12 REFUTED** (mutation gap: the drain's remapping-off gate had no test).
  - Closed at HEAD by c7ab5c7, `hook::main_thread::tests::a_signal_noted_before_remapping_went_off_is_dropped_by_the_drain` (`main_thread.rs:1610`). That test notes `SessionLocked`, turns remapping off through `set_enabled`, and asserts that the captured drain logs neither "session locked" nor "state reset".
  - Under the mutation it fails for that reason (`mut_M12b.txt`: both lines captured).
  - The product gate was present at 6f6edb3 (`main_thread.rs:711-713`).
  - The test installs no hook and injects nothing (a private `KeyboardHook::new()` and `logging::testing::capture`).
- **T-M13 REFUTED** (mutation gap: a return signal's foreground re-read had no test).
  - Closed at HEAD by 4539306, `a_return_signal_reads_the_foreground_window_again` (`main_thread.rs:1653`). It covers both `SessionUnlocked` and `DesktopSwitchedBack`, with a test-only `MovingQuery` and no Win32 window query.
  - Under the mutation it fails on the cache value (`mut_M13b.txt`: `left: Some(Some("p4096.exe")) right: Some(Some("p8192.exe"))`).
  - The product call was present (`main_thread.rs:731-733`).
- **T-M20 REFUTED** (mutation gap: the Win+Alt+T acceptance test asserted only suppression).
  - Closed at HEAD by 7ab78ab, `engine::scenarios::win_alt_t_held_across_a_focus_change_does_not_launch_again_on_its_repeats` (`scenarios.rs:1922`). Under the mutation it fails on the second `Launch` (`mut_M20b.txt`).
  - The product is correct: `Engine::on_foreground_changed` only resyncs modifiers (`engine/mod.rs:608-610`) and frees no key.
  - `capslock_held_across_a_focus_change_switches_the_language_once` (`scenarios.rs:2547`) and the focus property caught M20 even before 7ab78ab.
- **T-M16 REFUTED** (generation-bump removal not caught). It is an equivalent mutant, as the tester said:
  - The `seq` of every slot comes from the engine-wide `Ids::next_seq` (`engine/mod.rs:332-335`; callers `:683`, `:755`, `:787`, `:857`).
  - `Ids` is never reset: `reset` only bumps `generation`, at `:654`.
  - `reset` empties every table (`gestures.clear()`, `Chords::new()`, `Sequences::new()`, `:643-645`), and `gestures.on_timer`/`is_live`/`forget` match on `(vk, seq)` (`gestures.rs:359`, `:409-413`, `:424-428`).
  - So a stale token cannot match an entry armed after the reset, with or without the bump.
  - The bump is pre-existing (`master:src/engine/mod.rs:539`). It is not a lane gap.
- **T-N1 NOTED** (tester note): `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` asserts `stopped < ms(100)` on the real clock (`main_thread.rs:1512-1515`).
  - It passed 7 of 7 runs. It can flake only under heavy load, and it does not fail for a product reason.
  - Optional: a wider bound would keep its intent ("stopped at the lock, not after the 500 ms Delay").

## Counts
- CONFIRMED: 1 (R-I1)
- REFUTED: 8 (T-B1, T-B2, T-B3, T-B4, T-M12, T-M13, T-M20, T-M16)
- DEFERRED-OUT-OF-SCOPE: 0
- NOTED (Minor): 2 (R-m1, T-N1)

## For the fix round
Only R-I1 needs code: a test-only change in `src/hook/config_watcher.rs`, red under the `:158` `.expect` mutation. R-m1 can ride along as a doc fix. T-B1 is a `results.md` obligation for the results analyst (state +0.6 us for the witness read), not a developer item.
