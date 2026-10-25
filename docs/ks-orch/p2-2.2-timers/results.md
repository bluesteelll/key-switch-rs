# Results analysis: p2-2.2-timers / roadmap 2.2 Timers on the main thread (KS-11, KS-25, KS-33)

## VERDICT: ACCEPT

**Summary**: Every acceptance line of the spec and the roadmap has an automated test, and every orchestrator ruling (I1, M1-M8, OQ1-OQ5, Q1-Q7) is carried out in the tree. The gate is green in my own run. The last tester round is GREEN (23 mutations, all caught after one tester commit). The reviewer APPROVED with 0 Critical and 0 Important. The engine stays pure, `Mutex<Engine>` is gone, no thread is started on the gesture path, and none of the per-event bench rows regresses. What is left is Minor: two code nits (reviewer m2 and OQ3), one sentence of config-page drift, one consequence of the owner decision that is not documented, and the Phase-2 book sync.

Branch `feat/main-thread-timers`, HEAD `bfe931f`, 19 commits on `master` `c9c1373` (merge base = master tip), tree clean, no other worktree.

## Quality gate (own run, HEAD bfe931f)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0; `analyst-fmt.txt`) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK, twice: in the lane tree and in a fresh target dir (`--target-dir D:/tmp/ks-orch/p2-2.2-timers/analyst-target`, full re-check, 56 s; `analyst-clippy-fresh.txt`) |
| `cargo test --locked` | lib `running 636 tests` / `630 passed; 0 failed; 6 ignored`; `tests/ipc_roundtrip.rs` `6 passed`; swch bin and doc-tests 0. Exit 0 (`analyst-test.txt`) |
| Test count | `cargo test --locked -- --list`: **642** on HEAD against **603** on master (git-archive snapshot of `c9c1373`, `analyst-master-snap`). `#[test]` functions: 594 → 633 |
| Ignored (`--list --ignored`, not run) | 6: the 5 existing `manual:` tests unchanged, plus `bench::latency_bench` (`bench:` reason, ruling Q5: counted apart and never as a pass) |
| `cargo doc --no-deps` | 3 warnings, all pre-existing (`ConfigError::Missing` twice, unclosed `raw` tag) |
| `mdbook build book -d D:/tmp/ks-orch/p2-2.2-timers/analyst-book` | built (the mermaid version warning is a machine-setup issue); developer link check: 20 pages, 2324 links, 0 broken |
| Trailers | 19/19 commits carry `Co-Authored-By: Claude (Opus|Sonnet) 5.5`: 18 Opus, `bfe931f` Sonnet (tester) |
| Cargo.toml / Cargo.lock | untouched (no new dependency) |
| `git status --short` | empty before and after my runs |

Removed test names, each with a successor (I checked them against `cut.md` addendum M1 and `impl.md:71-91`): `with_engine_*` → `hook::main_thread::tests::with_main_refuses_reentry_on_the_same_thread` / `with_main_works_again_after_a_call`; `daemon_handle_status_follows_the_hook_flag` → `daemon_handle_switches_remapping_through_the_main_thread`; `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` → `remapping_off_callbacks_pass_without_asking_for_the_engine` + `remapping_off_keyboard_callback_returns_before_it_times_itself`; `a_hold_runs_its_slow_action_on_the_executor` → `..._from_the_main_loop`; the two known-issue-67 scenarios were renamed with their assertions flipped, as the spec demands; the 18 old `gestures::tests` were rewritten as 15 press-machine tests (mapping in `impl.md`). I read the old-to-new pairs I spot-checked (`tap_released_at_or_after_its_term_does_not_fire` → `a_tap_alone_fires_at_a_key_up_before_its_term`, gestures.rs:506-531, which keeps the "released at its term: no Tap" assert). None was weakened.

## Acceptance criteria

| # | Criterion (spec / roadmap 2.2) | Evidence | Status |
|---|---|---|---|
| A1 | `stale_tap_gc_eats_second_quick_tap` and `stale_hold_timer_fires_new_press_early` ported with inverted assertions, pass; red at 7cb3521 | `engine::scenarios::stale_tap_gc_eats_second_quick_tap` (scenarios.rs:392), `stale_hold_timer_fires_new_press_early` (:413), ok in my run. The red is shown by the tester's repro replay at `7cb3521` (`test_r1.md:37-48`): the inverted tests FAILED (`second tap fired = false`, `press#2 hold still pending at +110ms = false`), and the originals pass. The green on master is ruled Q6 | PASS |
| A2 | Tap(200)+Hold(200): release at 199 ms → only Tap, at 201 ms → only Hold | `tap_and_hold_with_equal_terms_fire_exactly_one` (scenarios.rs:496: 199 → `[SwitchLanguage]`, 201 → `[PressKey(LShift)]`), `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` (:547); tester mutation M14 caught | PASS |
| A3 | No thread per press over 1000 presses | `the_gesture_path_starts_no_thread` (keyboard_hook_callback.rs:997; red by assertion at `e9fc2f4`, mutation M20 caught) plus `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` (:764, M21 caught) | PASS (structural form, allowed by the spec) |
| A4 | Real-timer 200 ms deadline met within a few ms, CI-safe bound | `hook::main_thread::tests::a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` (main_thread.rs:890): never early (`held >= 200 ms`, M8), median of 5 < 5 ms, the reason in the comment (tick about 7.8 ms late on average); 5 tester reruns ok; M19 (30 ms late) caught | PASS |
| A5 | Another key-down cancels a pending pass-through gesture; the two known-issue-67 scenarios flip | `a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap` (:3724), `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap` (:3758), `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold` (:3783), all red at `c4609ea`; swallowed gestures kept: `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` (:1423), `another_key_does_not_cancel_a_swallowed_double_tap` (:3868); M15 and M16 caught | PASS |
| A6 | The known-issue-74 case no longer fires the first binding's action | `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins` (:917), red at `b68c7f2` with `fired [SwitchLanguage]` | PASS |
| A7 | Latency bench runs; master and HEAD numbers in impl.md and test_r1.md; no unjustified hook-path regression | `impl.md:162-197` (developer) and `test_r1.md:104-128` (tester, own worktrees). I recomputed the tester's raw files with `benchsum.py` and the numbers match. Every row is inside the D10 rule on the tester's first try (Latency section below) | PASS |
| S1 | Timers on the main thread in the same queue as key events; `TimerToken { generation, slot, seq }`; a stale token does nothing | engine/mod.rs:294-299, 435-459; `a_token_from_before_a_reset_is_stale` (mod.rs:918), `a_cancelled_hold_leaves_its_token_stale` (:958), `a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window` (scenarios.rs:892); M4a, M4b, M9 caught | PASS |
| S2 | Deadlines not rounded to the tick; hook callbacks delivered with no added delay; drain before every wait; `MWMO_INPUTAVAILABLE` pinned | Loop main_thread.rs:436-480 (drain :445, plan :446, wait `QS_ALLINPUT`/`MWMO_INPUTAVAILABLE` :459-464). `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer` (:828; OQ1 mutation `QS_ALLEVENTS` caught: each send waited 1 s). `a_message_posted_behind_a_wake_is_dispatched_without_more_input` (:773; mutation M3 caught). Live-hook delivery is manual (M9, M10) | PASS |
| S3 | `Mutex<Engine>` gone; fast gate kept; safe re-entry | `rg "Mutex<Engine>|with_engine|DECIDING|DecidingGuard|start_hold_timer|sleep_until" src` empty (tester); `with_main` uses `try_with` + `try_borrow_mut` (main_thread.rs:100-111); `with_main_refuses_reentry_on_the_same_thread`, `set_enabled_refuses_a_nested_call`; gate: `remapping_off_callbacks_pass_without_asking_for_the_engine` + `remapping_off_keyboard_callback_returns_before_it_times_itself` (:857; M22 red under the mutation) | PASS |
| S4 | Tap/Hold/DoubleTap one exclusive machine; Tap waits for the DoubleTap window only when both; latency declared | gestures.rs:57-118 (`Press`, `Phase`); C8 scenarios (scenarios.rs:728-948); cost declared in README, triggers-conditions.md §11.6 note, known-issues row 75, CHANGELOG Breaking line | PASS |
| S5 | Chord and Sequence entries expire on their own timers | `a_chord_start_asks_for_a_timer_at_the_end_of_its_window` (mod.rs:1082), `a_chord_expiry_timer_drops_the_pending_chord` (:1097), `each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale` (:1128), all red at `64b4ead` (`one expiry timer, got []`); M10 caught | PASS |
| S6 | Scheduler trait; scenarios drive time through the fake | engine/schedule.rs; testing.rs uses `TimerQueue` through `Engine::start_timers` (testing.rs:405-417, :543-548); schedule tests (M5, M6 caught) | PASS |
| S7 | `on_timer` never masks | `a_hold_timer_never_masks` (scenarios.rs:4716) plus the harness assert (testing.rs `assert!(!decision.mask, ...)` in `advance_ms`), which now covers the window and expiry timers too; M18 caught (50 failures) | PASS |
| S8 | Thread priority evaluated with evidence; implemented only if supported | D9 (Microsoft Learn citations, `cut.md:173-177`) + R8 under ruling M5 (0 of 3 runs worse, developer and tester alike); `raise_sets_highest_and_drop_restores_normal` (priority.rs; M7 caught); live effect manual M10 | PASS |
| S9 | R-7 of 2.1 (hold-timer half) closes | No hold-timer thread exists (A3 scan). The config-watcher half stays open and is declared (impl.md:271) | PASS |
| S10 | Docs: README, default_config.ron comments, triggers-conditions.md, known-issues rows 67/74/KS-11/KS-25, CHANGELOG; non-config pages listed in impl.md | All updated in the commits that change the meaning (see Docs); cfgcheck validated every new RON example and the template (`cfgcheck-output.txt`); non-config list at `impl.md:217-227` | PASS (one Minor drift, P2) |

The 10 manual checks (`impl.md:229-245`, M1-M10) supplement the automated evidence. No acceptance bullet depends on them alone. They have no ignored test and are not yet in a book smoke checklist (hand-off H-B1).

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-11 (fixed earlier in `16a0461`, ported here) | `stale_tap_gc_eats_second_quick_tap`, `stale_hold_timer_fires_new_press_early` | Yes at `7cb3521` (tester repro replay, `test_r1.md:37-48`); green on master by design (Q6) | Closed |
| KS-25 (every term pair) | `tap_300_and_hold_200_released_at_250_fire_only_the_hold` (the audit's scenario: Tap(300)+Hold(200), release 200-300 ms fired both), `a_hold_closes_the_double_tap_window`, `a_double_tap_press_arms_no_hold` | Yes: red at `b68c7f2` (`up CapsLock at +250 ms: expected to fire [], fired [SwitchLanguage]`), fixed in `506aad3`. The audit's third case (Tap(100)+Hold(200) fires neither) stays a declared gap (ruling Q4), pinned by `tap_100_and_hold_200_released_at_150_fire_nothing` | Closed (gap declared) |
| KS-33 (Hold half) | `the_gesture_path_starts_no_thread`, `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` | Yes: red by assertion at `e9fc2f4` (`keyboard_hook_callback.rs:91: use std::thread;`, `:312: thread::Builder::new()`), green from `464d3d4` | Closed (startup `config_watcher.rs` `.expect` follow-up recorded, H-B5) |
| Known issue 67 | the three C10 scenarios | Yes at `c4609ea` | Closed for key-downs (mouse residual declared, M3) |
| Known issue 74 | `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins` | Yes at `b68c7f2` | Closed |

Commit order shows red first for every item: tests `68b13a0` → fix `cfd7be6`; `e9fc2f4` → `464d3d4`; `1aa717f` → `464d3d4`; `b68c7f2` → `506aad3`; `c4609ea` → `c5ef59a`; `64b4ead` → `3e516f9`; `0a8caec` → `2f11917`. The tester replayed each red on its parent (`test_r1.md:20-35`). The I1 scenario is green on `c4609ea` by nature, because the old code cancels nothing. It goes red under the C11 cancel rule without the clock check (developer, and tester M1).

## Rulings (`rulings.md`), as carried out

| Ruling | Where | Status |
|---|---|---|
| I1 Hold past its term resolved by the clock | gestures.rs:313-320; `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` (scenarios.rs:3813: `advance_ms(1001)` and `advance_ms_stalled(1001)` both fire once); the addendum's six-state walk (`cut.md:407-418`) | Done. Item 4 of that walk (a waiting Tap whose window closed by the clock, timer late, then its own key-down) has no stalled-timer scenario; the code decides by `now < due` (gestures.rs:183). Minor test gap, H-O3 |
| M1 migrations, no weakened test | addendum `cut.md:419-444`; my removed-name check above | Done |
| M2 waiting Tap fires into the current focus, no `when:` re-check, current modifiers | known-issues row 75, triggers-conditions §11.6 note (:111-112), CHANGELOG | Done |
| M3 only key-downs cancel, mouse does not | row 67, §11.8 (:135), README, CHANGELOG, manual M3 | Done |
| M4 non-blocking reply; timed-out request still runs; post-`run` requests answered at once | `sync_channel(1)` + `try_send` (main_thread.rs:318, :535); `NoAnswer` doc (:285-287); `drop(main)` right after `run` (app.rs:270-274); `a_switch_request_to_an_ended_main_thread_fails_at_once` | Done |
| M5 R8 one-way at the receiver, 3 runs, margin | bench.rs probe window procedure (`SetWindowLongPtrW`); margin `p99_H > 1.25 x p99_N + 20 us`; 0/3 worse (developer and tester) | Done, D9 landed |
| M6 refused window timer; stale Held press | gestures.rs:401-417 (`forget` keeps the waiting Tap), :340-389 (`on_timer` fires closed windows), :195-206 (`press` replaces the stale press, keeps a due Hold); `a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down`, `a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`, `a_press_after_a_lost_key_up_...` | Done ("next event" read as the next first key-down or the next timer, justified by D6; reviewer accepted) |
| M7 `FastAction::run` only records under test | windows_actions.rs:143-165; `under_test_a_fast_action_is_only_recorded` | Done |
| M8 `t_down` read before `decide_event` | main_thread.rs:915-929 | Done |
| OQ1 `QS_ALLEVENTS` mutation | tester M2: red (each send timed out at 1 s) | Done: a pin, not a smoke test |
| OQ2 waiting Tap fires on another key-down | gestures.rs:308-311 | Done; the orchestrator reports the reading to the owner (H-O6) |
| OQ4 scan scope and tokens | keyboard_hook_callback.rs:997-1046 (3 extra files, `spawn`/`Builder`/`scope`/`use std::thread`) | Done |
| OQ5 `switch_enabled` only through `MainState` | keyboard_hook.rs `switch_enabled(&self, enabled, _main: &mut MainState)`; a `&mut MainState` exists only inside `with_main` | Done |
| Q1 thread-local `MAIN` | main_thread.rs:76-79 | Done; CLAUDE.md update is the orchestrator's (H-O1) |
| Q3 HIGHEST, not TIME_CRITICAL | priority.rs | Done |
| Q4 Tap(100)+Hold(200) gap documented | README, §11.6 table, CHANGELOG KS-25 line, pinned test | Done; loader warning goes to 3.4 |
| Q5 bench rules | bench `bench:` reason; tester ran master and HEAD in own worktrees, R8 with `KS_BENCH_LOAD=1` | Done |
| Q6 KS-11 red on the 7cb3521 replay | `test_r1.md:37-48` | Done |
| Q7 executor MWMO pin to 3.4 | only the main loop is pinned here | Done (H-B5) |

## Latency (owner decision; ruling "Latency")

Medians of three p50s, largest p99, in µs. Tester: master `611c83f` (C1) against HEAD `d738632` in its own worktrees (`bench-r1/`). Developer numbers in brackets (`impl.md:166-177`). `bfe931f` adds only a test.

| Row | What | master p50 / p99 | HEAD p50 / p99 | D10 rule | Verdict |
|---|---|---|---|---|---|
| R1 | `Engine::on_key`, typing | 0.1 / 0.3 [0.1 / 0.4] | 0.1 / 0.4 [0.1 / 0.7] | p50 ≤ 0.225 | pass. p50 sits at the clock's 0.1 µs floor |
| R2 | `Engine::on_key`, gesture presses | 0.2 / 0.5 [0.2 / 0.5] | 0.2 / 0.6 [0.2 / 0.5] | p50 ≤ 0.35 | pass |
| R4 | adapter callback without a hook | 5.3 / 64.0 [5.3 / 58.1] | 5.2 / 47.6 [7.0 / 61.5] | p50 ≤ 7.125, p99 ≤ 98.0 | pass. The developer's first HEAD window (7.0) was noise: 4 interleaved pairs gave 5.3-5.7 against 5.3-5.5 (`bench-alternate.md`) |
| R5 | Hold key-down callback | 35.3 / 161.5 | 5.3 / 63.2 | p50 ≤ master | pass, 6.7x lower (no thread spawn) |
| R6 | gesture deadline lateness, d = 5/20/50/200 ms | 676 / 673 / 608 / 774 (p99 max 3013) | 417 / 406 / 486 / 358 (p99 max 2164) | p50 ≤ 1000 and ≤ master + 500 | pass, sub-ms |
| R7 | sent-message round trip to the loop | 12.6 / 108.7 [10.2 / 171.8] | 15.5 / 186.5 [15.8 / 242.5] | p50 ≤ master + 20, p99 ≤ 1.5 x master + 50 (213.1) | pass. p50 +2.9 µs (one drain and plan per wake) |
| R8 | one-way delivery under full load, NORMAL vs HIGHEST (3 runs) | NORMAL p99 482.9 / 537.3 / 467.5 [252.1 / 561.0 / 2632.3] | HIGHEST p99 372.2 / 281.8 / 386.7 [234.1 / 204.5 / 393.7] | HIGHEST worse iff p99_H > 1.25 x p99_N + 20 | 0 of 3 worse in both sets; HIGHEST p50 2-5 µs higher, p99 23-48 % lower → D9 lands |

Per-event cost on the hook path is unchanged within the clock's resolution (R1, R2, R4). The Hold key-down got cheaper (R5). Deadlines are met in about 0.4 ms. The only measurable regression is the +3 µs median of R7, which the cut's rule justifies.

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback: passthrough, outcome, no blocking/alloc, symmetry | OK | `code < 0` → `CallNextHookEx` first (keyboard_hook_callback.rs:318-323); off gate before reading `lparam` (:326-331, now pinned by `bfe931f`); sentinel before `with_main` (:188-190); `LRESULT(1)` iff `suppress`, else `CallNextHookEx` (:350-356). No `log::` in the file. Under the borrow, only `on_key` plus `start_timers`, with at most one `SetWaitableTimer` (main_thread.rs:254-266) and no pump, inject or submit. The new allocations are the ones the cut prescribes: the `GroupKey` clone on a DoubleTap arm (gestures.rs:215-220), `Decision.timers` pushes for window, chord and sequence timers, and queue growth past 16. No new indexing by vk (`position`). The key-up rule is unchanged (mod.rs:759-760), and the strict-mode properties now include the `tap_double_tap` config |
| Engine purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]` kept; the clock comes in through `Env::now` and timers go out through `Scheduler`; nothing executes. Exact modifiers and repeat defaults untouched. Repeats never reach `press`/`other_key_down` (mod.rs:554). Presses are keyed by the sided vk; group identity uses `KeyCombination ==` + `Condition ==` |
| Thread ownership, sentinel, atomics, SAFETY | OK | Engine, timers and loop live on the main thread (`MainThread` is `!Send`, one state per thread); hooks are still installed and removed only there (app.rs). No new `SendInput` builder; Hold fast actions use the existing sentinel-tagged builders from `drain` → `carry_out`, outside the borrow. `enabled` uses `swap(AcqRel)`/`load(Acquire)`; no `Relaxed`, `SeqCst` or `static mut` added. Every new `unsafe` in main_thread.rs, priority.rs and bench.rs has a concrete `// SAFETY:` (I checked each bench block). The pseudo thread handle is never closed. The loop has no spin path: `Plan::Due` compares monotonic `Instant`s, and the next drain takes the timer |
| Config reload, IPC | OK | Reload path unchanged (pending presses keep cloned actions; `reload_during_a_passing_double_tap_keeps_each_press_whole` green). `swch on/off` go through a bounded queue with a 2 s reply timeout and fail at once after `run` (the receiver is dropped with `MainThread`). Pipe DACL unchanged; `tests/ipc_roundtrip.rs` green |
| No key logging | OK | No new log line names a key; the loop logs only timer failures and priority warnings |

## Review status

- `review_r1.md`: **APPROVED** (Critical 0, Important 0, Minor 2). No triage was needed.
  - m1 (the fast-gate successor did not pin the callback's own gate): **resolved** in the tree by the tester's `bfe931f`, which adds `remapping_off_keyboard_callback_returns_before_it_times_itself`, red under M22. I checked the claim that it cannot race: only `keyboard_hook_callback` writes the gauge, and every test that calls it holds `GLOBAL_HOOK`. No other test reads `COUNTERS`.
  - m2 (`N pending gestures dropped` and `hold_timer_not_started` also count timers whose entries survive): **open, Minor** (H-O2).
  - Open questions: OQ1 (`MAIN_WAKE` sent during a WinEvent callback) is covered by `MWMO_INPUTAVAILABLE` and by the pump looping until the queue is empty; manual M7 plus M9 settle it live. OQ2 (two-modifier DoubleTap) is H-O6/H-B3. OQ3 (`armed` kept after a failed set in `start`, main_thread.rs:256-263) is Minor (H-O2).
- `test_r1.md`: **GREEN**: 23 mutations, M22 caught after `bfe931f`, two equivalent mutations recorded (generation guard, scenario shielded by queue pruning; a unit test covers it).

## Docs and hygiene

- README (tree, `main_thread.rs` section, adapter and app bullets, trigger rules, cancel rule, Technologies, Features), `default_config.ron` (ASCII), CHANGELOG `[Unreleased]` (Added: bench; Changed: two **Breaking:** lines, main-thread timers, priority, expiry timers, `swch` replies, test-only fast actions; Fixed: KS-25 every pair, known issue 74, KS-33), `triggers-conditions.md` §11.4-§11.8, `known-issues.md` rows 23, 24, 67, 74 and new row 75, `actions.md` §13.1, `errors.md` §18.6 (5 new rows), `kinds.md`, `structure.md:63`. All are in the commits that change the meaning, and cfgcheck validated every new RON example.
- `audit.md` and `roadmap.md` status lines and the other non-config pages go to the Phase-2 book sync, as the spec says (list at `impl.md:217-227`, copied in H-B below).
- Branch: cut from `master` `c9c1373`, no merges inside, only this lane's work, clean to merge with `--no-ff`.

## Technical debt

- `src/hook/main_thread.rs:227-240`, `:448-450`, `errors.md:198,208`, `counters.rs:27-30`: the dropped-gesture count and `hold_timer_not_started` overcount (reviewer m2). Recorded only in `review_r1.md`; H-O2.
- `src/hook/main_thread.rs:256-263`: `armed` is kept after a failed `SetWaitableTimer` in `start` (reviewer OQ3). Only in `review_r1.md`; H-O2.
- `src/core/executor/mod.rs:6`: a stale "a Hold's `hold-timer` thread" doc line. In `impl.md:226` for the sync.
- `src/hook/config_watcher.rs:126` `.expect("spawn config-watcher thread")` (startup, KS-33-like); the executor's own MWMO pin (3.4); a re-entry refusal counter (3.4); a loader warning for unequal Tap/Hold terms (3.4); `KBDLLHOOKSTRUCT.time` as the event clock; residual R-7 (config watcher posts a wake to an ended executor at exit). These are recorded in `cut.md:356-357` and `impl.md:271-272` only, not yet in `roadmap.md` or `gaps.md`; H-B5.
- A passing multi-key DoubleTap (for example `Ctrl+Shift`) now completes only if one key stays held between the presses, because pressing the other modifier again is "another key-down" (gestures.rs:312; reviewer OQ2). This follows from the owner decision, but no test pins it and no page states it; H-B3 and H-O6.
- No `TODO`/`FIXME` added; no non-ASCII added under `src`.

## Problems

None blocks the merge. Everything below is Minor and goes to the hand-off lists.

### P1. Addendum I1 item 4 has no stalled-timer scenario (Minor)
- **What**: a Tap that waits for its DoubleTap window whose window has closed by the clock while its timer is late, then its own key-down. The code fires the Tap and arms a new press (gestures.rs:183-193). That matches the on-time outcome, but only the unit boundary test (gestures.rs:673-679, no Tap) and the lost-timer scenario for another key (scenarios.rs:967) pin it.
- **Impact**: a future change to `press` could break the clock rule for this state unnoticed.
- **Owner**: tester, optional, while the context is fresh (H-O3).
- **Re-check condition**: a scenario on `TAP_AND_DOUBLE_TAP_F13` with `advance_ms_stalled(301)`, then `down("F13")` expecting `[SwitchLanguage]` and no `ToggleCapsLock`, identical to the `advance_ms(301)` script.

### P2. One sentence of config-page drift (Minor)
- **What**: `book/src/config/triggers-conditions.md:77` says "`Hold` is the only trigger whose action fires outside a key event". A Tap that waits for its DoubleTap window now fires on its own timer too, as the same page says at :68 and :102.
- **Owner**: doc-writer, in the Phase-2 book sync (H-B2).
- **Re-check condition**: the sentence names both timer-fired cases.

## Positives

- One code path for production and scenarios: `Engine::start_timers` through `Scheduler`, so the harness exercises the same pruning and refusal logic as the loop.
- `GestureTimers::start` sets the wake inside the callback, so correctness does not depend on whether `MsgWaitForMultipleObjectsEx` dispatches sent messages, which is undocumented. A stale or early signal costs one drain.
- The I1 clock rule makes the engine's outcome independent of wake order for every timer-driven state the addendum walks.
- Ruling M7 closed a real safety gap: before it, a test reaching `carry_out` with a fast action could type into the owner's foreground window.
- The tester's mutation campaign (23 mutations, including the `QS_ALLEVENTS` and `MWMO` pins) gives strong evidence that the new tests can fail.
- R5 shows a 6.7x cheaper Hold key-down, and R6 shows deadlines met in about 0.4 ms, against about 0.6-0.8 ms on master's `hold-timer` threads.

## Hand-off: Phase-2 book sync (doc-writer)

- H-B1. Add the lane's manual checks M1-M10 (`impl.md:229-245`, configs in `D:/tmp/ks-orch/p2-2.2-timers/cfg/manual-*.ron`) to the Checkpoint 2 smoke checklist (`roadmap.md:508-510`), in the table form Phase 1 uses (`roadmap.md:~395-413`). Update Phase-1 manual check 9 (`roadmap.md:404`: `<T>` no longer toggles CapsLock).
- H-B2. `triggers-conditions.md:77` (P2). `errors.md:208` and the counter text should say that `hold_timer_not_started` also counts a refused window timer, whose Tap then fires at the next key-down (`counters.rs:27-30` already says so).
- H-B3. A known-issues row, or a §11.4 sentence: a passing multi-key `DoubleTap` (`Ctrl+Shift`) completes only if one key stays held between the presses (reviewer OQ2), pending the owner's answer (H-O6).
- H-B4. The non-config list at `impl.md:217-227`: `architecture.md` (engine lock, `hold-timer` threads, `GetMessageW` loop, `set_enabled` on the IPC worker → `hook::main_thread`), `overview.md:68,89,223`, the `glossary.md` entries listed, `build-and-run.md:141,229-230,253-265` (new `swch on/off` replies; a timed-out request still takes effect), `roadmap.md` (2.2 status, "Files" reuses `platform/timers.rs`, open questions 5 and 10 decided and implemented, follow-up row `:805` closed, the "Known issues until Phases 2 and 3" bullets for KS-11 and known issue 67 at `:423` and `:428`, the coverage rows for KS-11/KS-25/KS-33), `audit.md` status lines KS-11 `:364`, KS-25 `:574` → Fixed (with the declared Tap(100)+Hold(200) gap), KS-33 `:656` → Fixed, `gaps.md:30,68-91`; `src/core/executor/mod.rs:6` (code doc).
- H-B5. Record the follow-ups in `roadmap.md` or `gaps.md`: `config_watcher.rs:126` `.expect`; the executor's MWMO pin and the shared wait/pump (3.4); the re-entry refusal counter (3.4); the loader warning for unequal Tap/Hold terms (3.4); `KBDLLHOOKSTRUCT.time` as the event clock; residual R-7 (config-watcher wake at exit); reviewer m2 and OQ3 if the orchestrator does not fix them now.
- H-B6. 2.2 "Latency" acceptance: record that the bench exists, with its command and D10 rule, and that later lanes run it on parent and HEAD.

## Hand-off: orchestrator (CLAUDE.md, agent files, owner)

- H-O1. CLAUDE.md after the merge:
  - Principle 5: the `Mutex<Engine>` / `with_engine` exception ends. `hook::main_thread::MAIN` is a thread-local `RefCell<Option<MainState>>` reached only through `with_main` (`try_with` + `try_borrow_mut`, refuses re-entry, never panics). It is not shared hook state (ruling Q1). The remapping switch flips only through `KeyboardHook::switch_enabled(.., &mut MainState)`.
  - Principle 1: the hook path may make one `SetWaitableTimer` call under the borrow, and "never call `logging::flush()` under `with_engine`" becomes `with_main`.
  - Principle 6: the main thread owns the engine and the gesture timers and runs `MainThread::run` (`MsgWaitForMultipleObjectsEx` + `PeekMessageW`, `MWMO_INPUTAVAILABLE`) at `THREAD_PRIORITY_HIGHEST`. Hold and waiting-Tap fast actions run there, in the drain.
  - Principle 9: under `cfg(test)`, `FastAction::run` only records (ruling M7); a test that reaches the engine makes its thread a main thread (`MainThread::new()`).
  - Layout: `src/hook/main_thread.rs`, `src/engine/schedule.rs` (`Scheduler`, `TimerQueue`, `Engine::start_timers`), `src/platform/priority.rs`, `src/bench.rs`.
  - Build and gate: the bench command (`KS_BENCH_OUT=<file> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`, `KS_BENCH_LOAD=1` for R8). A `bench:` ignored test is counted apart from `manual:` tests and never as a pass. Every later lane runs it on parent and HEAD against the D10 rule.
- H-O2. Minor code fixes worth doing while the context is fresh (developer, one follow-up commit each, or record them via H-B5): reviewer m2 (count only the entries `forget_timer` really dropped, or reword the log line, the counter doc and `errors.md`), and reviewer OQ3 (clear `armed` when `SetWaitableTimer` fails in `GestureTimers::start`).
- H-O3. Optional tester scenario for P1.
- H-O4. Agent files: reviewer R3/R4 (the `RefCell` ban does not cover `MAIN`), tester brief and `ks-lane.js` templates (the bench step, R8 when priority or loop code changes, the D10 comparison rule, the `bench:` category).
- H-O5. Housekeeping outside the repo: `D:/tmp/ks-orch/p2-2.2-timers/{review-target,analyst-target,analyst-master-snap,target-wt,cfgcheck-target,wt,analyst-book,book}` can be deleted.
- H-O6. Report to the owner (values questions):
  - (a) the OQ2 reading: a Tap that waits for its DoubleTap window fires, rather than being cancelled, when another key goes down;
  - (b) the consequence for passing multi-key DoubleTaps (H-B3);
  - (c) `THREAD_PRIORITY_HIGHEST` adopted on the R8 evidence;
  - (d) the declared latency cost of Tap+DoubleTap groups (up to the DoubleTap term).

## Notes for later lanes

- 2.3 adds `Request::Reset` to the same request queue and runs `reset_transient_state` in `MainState`. The `MAIN_WAKE` + drain shape is ready for it.
- 3.4 should fold the executor's and the main loop's wait/pump into `platform` with one `MWMO_INPUTAVAILABLE` pin. The main loop's pin is `a_message_posted_behind_a_wake_is_dispatched_without_more_input`.
- 3.5 inherits the bench and its D10 rule. R1-R3 sit at the 0.1 µs `Instant` floor, so a finer measure (batch timing) will be needed to see engine-level regressions.
