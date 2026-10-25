# Results analysis: p0-A1-engine / Workstream A scope additions (pure decision core, Env, scenario tests)

## VERDICT: REWORK -> developer

**Summary.** The design and the committed implementation meet every acceptance criterion. Evidence:

- My own gate on a clean `git archive d3131fc` export is green: 189 passed, 0 ignored.
- The reviewer APPROVED with 0 Critical and 0 Important findings, and the tester is GREEN.
- I found no invariant violation.

Two problems still block a safe merge, and both are small:

- **P1.** The tester's four gap-closing tests sit **uncommitted** in the shared work tree (`git status`: `M src/engine/mod.rs`, `M src/hook/keyboard_hook_callback.rs`, byte-identical to `tester_tests.patch`). A merge from this state either fails at `git checkout master`, because `src/engine/mod.rs` does not exist on master, or drops the tests.
- **P2.** The open review Minor m1 (a blank line inside the CHANGELOG `### Fixed` list).

This is a narrow developer task with no design change. After it, the lane is ACCEPT material.

Branch `refactor/engine` HEAD `d3131fc7`. `master` and the merge-base are both `73bbf404`, so the merge is conflict-free (`git merge-tree` exit 0; it writes only an unreferenced tree object).

## Quality gate (own run)

Run in my scratchpad on `git archive d3131fc`, with its own `CARGO_TARGET_DIR`. The shared tree was not built.

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0) |
| `cargo test --locked` | lib: `running 189 tests` / `189 passed; 0 failed; 0 ignored`; swch and doc-tests: 0 tests (they have none). master `73bbf40`: 130 tests (120 at audit commit `7cb3521`) |
| `cargo test --locked -- --list --ignored` | `0 tests` in every target (no `#[ignore]` anywhere) |
| `cargo doc --no-deps` | 1 warning: `src/data/vk_name.rs:4` "unclosed HTML tag `raw`", which predates the lane (the file is not in the diff) |
| HEAD + the tester's uncommitted tests (work-tree copy) | fmt OK, clippy OK, `193 passed; 0 failed; 0 ignored`, the 4 `tester` tests ok |
| Test names lost against master | exactly the 21 tests D9 says are ported. All 21 counterparts exist at HEAD under the names D9 lists (checked one by one). I spot-checked 2 ported tests against master: they are equal or stronger |
| Commit trailer | 12/12 commits in `master..refactor/engine` carry `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` |
| `git diff --check master...refactor/engine` | clean |

## Acceptance criteria

| # | Criterion | Evidence | Status |
|---|---|---|---|
| 1 | The quality gate passes | My run above | MET |
| 2 | `rg 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::' src/engine` finds nothing outside comments | My run: one hit, `src/engine/mod.rs:73`, a `//!` line. 0 non-comment lines. `#![forbid(unsafe_code)]` at `src/engine/mod.rs:122`. `rg '\.execute\(\|unsafe\|Instant::now\|ForegroundContext' src/engine`: every hit is a comment, the attribute, `testing.rs` (`cfg(test)`) or inside `mod tests` | MET |
| 3a | The hook callback calls the engine for every key event | `keyboard_hook_callback.rs:215-242`: `code<0` passes, then the single path `decide_event` -> `with_engine` -> `Engine::on_key` (`:150-165`). The only bypasses are the sentinel, non-key messages and re-entry. Tests: `hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine` (`:287`, all four messages, each up after a suppressed down) and `injected_and_unknown_events_bypass_the_engine` (`:315`). Tester mutations M5 and M20 are caught. Mapping `LRESULT(1)` iff `suppress` at the extern level is proven only by the UNCOMMITTED tester test (M17), see P1 | MET (the extern mapping is review-only until P1) |
| 3b | No decision logic is left in `src/hook` | Token grep over `src/hook`: one hit, `keyboard_hook_callback.rs:318` `Trigger::Tap` inside `mod tests` (test setup). `src/hook/{tap,chord,sequence}_state.rs` are deleted. `hook/mod.rs` has 4 modules | MET |
| 4 | Scenario tests: Immediate combo | `engine::scenarios::{immediate_single_key_fires_on_down_and_swallows_both_edges, immediate_combo_fires_on_its_last_key_and_lets_the_modifiers_through, right_hand_modifiers_match_unsided_combo_keys, combo_does_not_fire_without_all_of_its_keys, more_specific_immediate_binding_wins, press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed, unbound_key_passes_through}` (`scenarios.rs:21-152`) | MET |
| 4 | Tap | `tap_fires_on_a_release_within_its_term`, `tap_held_past_its_term_fires_nothing_and_swallows_the_release`, `two_quick_taps_fire_twice` (`:252-300`) | MET |
| 4 | Hold | `hold_fires_once_its_term_passes_while_held`, `hold_released_early_is_cancelled`, `stale_hold_timer_does_not_fire_the_next_press_early`, `tap_and_hold_with_equal_terms_fire_exactly_one`, `hold_released_after_its_term_fires_when_its_late_timer_wakes`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`, `hold_released_before_its_term_with_a_late_timer_is_cancelled`, `hold_left_to_a_late_timer_does_not_block_the_next_press`, `hold_whose_timer_cannot_start_never_fires` (`:304-512`) | MET |
| 4 | DoubleTap | `double_tap_fires_on_the_second_press_within_its_term`, `double_tap_second_press_after_the_term_arms_again`, `four_quick_presses_double_tap_twice` (`:516-578`) | MET |
| 4 | Sequence | `sequence_fires_when_its_steps_follow_within_max_gap`, `sequence_times_out_after_max_gap`, `sequence_broken_by_another_key_does_not_fire`, `sequence_with_a_modifier_held_through_both_steps` (`:582-654`) | MET |
| 4 | Chord | `chord_fires_when_all_keys_go_down_within_its_window`, `chord_keys_can_go_down_in_any_order`, `chord_does_not_fire_once_its_window_has_passed`, `key_outside_the_chord_passes` (`:658-709`) | MET |
| 4 | when-condition | `when_app_equals_matches_the_foreground_app_case_insensitively`, `when_title_contains_and_not_conditions`, `first_binding_whose_condition_holds_wins_on_a_shared_key` (`:184-248`) | MET |
| 4 | block_default false | `block_default_false_fires_and_lets_both_edges_through` (`:121`) | MET |
| 5 | Architecture documented in module docs | `src/engine/mod.rs:1-120` (Architecture, purity and the scope of `forbid`, State, Time, Threads and locking, Decision rules), `env.rs:9-48`, `gestures.rs:1-27`, `testing.rs:1-25`, `keyboard_hook_callback.rs:1-60` (the 6 steps, Threads and locking, Re-entry with its residual risk). I read all of it: accurate against the code | MET |
| S1 | Spec: the engine is pure. `on_key(event,&bindings,&dyn Env)`, `on_timer(token)`, `Decision{suppress,fire,timers,mask}`, and it executes nothing | `mod.rs:200-287`. `mask` is reserved and never set (roadmap 1.2) | MET |
| S2 | Spec: `Scenario::from_ron` with `.down/.repeat/.up/.advance_ms` and a fake Env and clock | `testing.rs:124-252`, `FakeEnv` `:40-84` (OS key state modelled from passed events) | MET |
| S3 | Spec: tap/chord/sequence state files deleted | Deleted in `0873abf` | MET |
| S4 | Spec: behaviour unchanged except design-made fixes, each with a CHANGELOG Fixed entry and a test | KS-11, KS-25 (equal terms), KS-33 (partly) in `CHANGELOG.md:55-67`. Deltas checked line by line against master (see Invariants) | MET |
| S5 | Spec: "same threads, same timing" for carrying out the Decision | Review: `carry_out` runs on the hook thread for key events and on `hold-timer` for Hold, exactly like master's thread map. Live behaviour is manual only (M1-M9 below) | MET by review + manual list |
| S6 | Spec: the existing suite keeps passing | Name diff above: 21 removed = 21 ported. Nothing weakened; none newly ignored | MET |

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-11 | `engine::scenarios::{two_quick_taps_fire_twice, stale_hold_timer_does_not_fire_the_next_press_early, four_quick_presses_double_tap_twice}`, `engine::gestures::tests::stale_hold_timer_does_not_fire_a_newer_press`. These are the audit's exact scenarios (taps at 0/80/150/230, DoubleTap at 0/100/190/260) | Yes. On master the tester's scratch test failed with `panicked at src\hook\tap_state.rs:284:9: KS-11` (test_r1 replay). On HEAD, mutations M2 and M21 (generation token removed) turn them red | CLOSED |
| KS-25 (equal terms only) | `hold_released_after_its_term_fires_when_its_late_timer_wakes`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`, `hold_left_to_a_late_timer_does_not_block_the_next_press`, plus the guard `hold_released_before_its_term_with_a_late_timer_is_cancelled` | Yes. Red at `eef8629` on its parent with the panics quoted in that commit body, replayed by the tester. Mutation M11 is caught | CLOSED (partly, as claimed; unequal terms stay for 2.2) |
| KS-33 (partly) | `engine::scenarios::hold_whose_timer_cannot_start_never_fires`, `engine::gestures::tests::forgotten_hold_never_fires_and_frees_its_key`, plus review of `start_hold_timer` (`keyboard_hook_callback.rs:186-200`: `Builder::spawn` error logged, no `expect`). Tap and DoubleTap cannot request a timer by type (`arm_tap` returns `()`, `double_tap` returns `Option<BindAction>`) | No fault injection exists, as cut D5 accepted. The committed scenario does NOT catch `Engine::forget_timer` turned into a no-op (tester M4). Only the uncommitted `engine_forget_timer_makes_a_late_timer_fire_nothing` catches it | CLOSED by review; the regression gap is closed only by P1 |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking or allocation, symmetry) | OK / WARN | **OK:** `code<0` -> `CallNextHookEx` (`keyboard_hook_callback.rs:220-225`). The sentinel is checked before any state (`:156`). Every path returns `LRESULT(1)` (`:235`) or `CallNextHookEx` (`:241`). No `expect`/`unwrap` is reachable from either extern fn (poison recovered `keyboard_hook.rs:83`; `try_from(..).ok()?` at `:138`). Symmetry is unchanged against master, and the only new key-up case, `HoldRelease::Overdue`, follows an always-suppressed Hold key-down (`mod.rs:431-435`). Pre-existing asymmetries are untouched (KS-06 repeats, focus-change clearing, refused re-entry of a key-up, all documented). **WARN 1 (sanctioned):** one `Mutex<Engine>` is taken per key event. A `hold-timer` thread holds it only for `fire_hold`, and foreground lookups run under it (cut D2/D7; spec allows it until 2.2). **WARN 2 (sanctioned):** an Immediate fire now allocates a `Vec` and clones its `BindAction` (`mod.rs:313`; cut D11, zero allocation is roadmap 3.4). Chord key-downs allocate less than before (`chords.rs:66`) |
| `decide()` purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]`. All Win32 access and the clock go through `Env`. No `execute` or `Instant::now` outside tests. Matching is ported unchanged (superset KS-04, sides KS-07 and repeats KS-06/10 stay for their own lanes), so no owner decision is contradicted. The stable specificity sort is kept (`mod.rs:222`) |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | Hook install/uninstall are untouched. Hold actions run only on `hold-timer`, because the key-up never fires a Hold (`mod.rs:424-429`); this is critique I1 resolved. No new `SendInput`. No new atomics (the `[AtomicBool;256]`/`[AtomicU64;4]` tables are gone), and `DECIDING` is a const `thread_local!` (`keyboard_hook.rs:15-35`). 4 new `unsafe` blocks, each with a concrete `// SAFETY:` (`keyboard_hook_callback.rs:117-120, 221-224, 227-230, 239-241`) |
| Config reload, IPC | OK (untouched) | Bindings are passed per call. `update_bindings` -> `engine::sort_by_specificity`. IPC is not in the diff |
| No key logging | OK | The only new output is `eprintln!` on a failed spawn, with no key information (`:197`) |

## Review status

- `review_r1.md`: **APPROVED** (Critical 0, Important 0, Minor 1). m1, the CHANGELOG blank line, is **open**. I confirmed it at `CHANGELOG.md:68`: it makes the whole `### Fixed` list loose in CommonMark.
- `critique.md` I1, I2 and M1-M3 are all resolved in the tree, and I verified each:
  - I1: `on_key_up` puts only the Tap into `fire`, and a failed spawn calls `forget_timer`.
  - I2: the production-only grep is empty.
  - M2: the SYSKEY pair is in the test.
  - M3: the Re-entry docs are at `keyboard_hook_callback.rs:47-60`.
- No triage or fix round ran (round 1 was clean). The last tester round, `test_r1.md`, is **GREEN**.
- No CONFIRMED Critical or Important item is open.

## Docs and hygiene

- `CHANGELOG.md` Unreleased has a Changed entry and Fixed entries for KS-11, KS-25 (equal terms) and KS-33 (partly). They are accurate, apart from m1.
- `README.md`: the layout tree, the module bullets, "How resolution works" and the feature lines all match the code. "Hot-path allocations: none" (`README.md:528`) was already false on master; roadmap 3.4 makes it true.
- `default_config.ron`: unchanged. Config meaning is unchanged: no new field, and parsing is unchanged. Equal-term resolution now matches the existing comment.
- The book is untouched, per cut Q4 (book sync). It is now stale in several places; the list is in H2 below.
- `Cargo.toml` and `Cargo.lock` are unchanged; no new dependency.
- The branch holds only this lane's work, plus the D0 merge of master, which brought only `.claude/`, `CLAUDE.md`, `docs/WORKFLOW.md` and `.gitattributes`.
- Commit order is red-first: `eef8629` (assertion red) comes before `a50e8d7`, and `9e9057e` (compile red) comes before `941f0a7`. The failing lines are in the commit bodies.
- **Work tree dirty:** the tester's patch is uncommitted (P1).

## Technical debt

- `keyboard_hook_callback.rs:47-60`: a refused re-entry of a key-up leaves the key held and blocked. It is latent today. Roadmap 2.3 (a hidden control window) and 4.6 (tray) add own-process windows, which can make this real. The risk is recorded **only in module docs**. **Not recorded** in the roadmap or `gaps.md` (H3).
- Engine -> `core::windows_actions::BindAction` dependency. Moving `BindAction` into `data` is a 3.4 follow-up, recorded **only in the lane reports** (H3).
- `scenarios.rs:171-180` `foreground_change_forgets_pending_key_up_suppression` pins today's focus-change clearing. Roadmap 2.3 changes that behaviour. It is recorded in roadmap 2.3 "What", so the scenario will fail there and be revised. `scenarios.rs:3-5` says known defects are not pinned, which this scenario contradicts slightly (Minor).
- `Mutex<Engine>` against CLAUDE.md principle 5 ("hook-wide Mutex") and code-reviewer R3. The spec sanctions it. Rewording CLAUDE.md is assigned to the orchestrator (cut Q1/Q5) and is not done (H1).
- KS-33 remainder: a cancelled Hold's `hold-timer` still sleeps to its due time (KS-27 unbounded terms). This is recorded under roadmap 2.2.
- A failed Hold spawn followed by a release after the term fires neither Tap nor Hold. This is documented in CHANGELOG and in `start_hold_timer`'s doc.
- Allocation on Immediate fire. Roadmap 3.4 records it ("Hot-path allocations: none becomes true").
- No new `TODO`/`FIXME` in the diff.

## Problems

### P1. The tester's four gap-closing tests are uncommitted in the shared work tree
- **What.** `git status` shows `M src/engine/mod.rs` (+80 lines, `#[cfg(test)] mod tester_tests`) and `M src/hook/keyboard_hook_callback.rs` (+38 lines). `git diff` equals `D:/tmp/ks-orch/p0-A1-engine/tester_tests.patch`. The four tests:
  - `engine_forget_timer_makes_a_late_timer_fire_nothing` (catches M4: `Engine::forget_timer` turned into a no-op, which passes every committed test);
  - `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` (M15);
  - `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (M16);
  - `tester_extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` (M17: the only automated proof of `LRESULT(1)` iff `Decision::suppress` at the `extern "system"` level).

  test_r1 says they "must be kept". No stage committed them: the tester may not commit, and no fix round ran.
- **Impact.**
  - The merge step starts with "confirm the branch is clean". `git checkout master` refuses, because `src/engine/mod.rs` is modified locally and absent on master. `stash` and `restore` are denied by `settings.json`.
  - If the merge goes ahead some other way, master loses the only test that pins the KS-33 `forget_timer` contract and the extern return mapping.
  - Also, `mod tester_tests` breaks house rule 9 (unit tests live in `mod tests`), and it defeats the lane's own production-only purity check, which stops at `^mod tests {`. The tester saw `mod.rs:469 Instant::now()` leak into that check.
- **Root cause.** The pipeline has no owner for tester-written tests when round 1 is clean: triage and fix are skipped, and the tester is "no commits".
- **Return to:** `developer`.
- **Do.**
  - Commit the four tests as one follow-up commit, for example `Engine: keep the tester's round-1 gap tests`, with the trailer and the mutation each one catches in the body.
  - Move M15 and M16 into `src/engine/scenarios.rs`.
  - Put M4 in a `#[cfg(test)] mod tests` at the bottom of `src/engine/mod.rs`, keeping its `FixedEnv` local to that module.
  - Put M17 into the existing `mod tests` of `keyboard_hook_callback.rs`.
  - Drop the `tester_` prefixes and the "uncommitted" comments.
  - In M17, keep the `// SAFETY:` comment and add why `CallNextHookEx` is harmless with no hook installed (its `hhk` parameter is ignored per Microsoft Learn; nothing is injected). Critique open question 1 asked for exactly this check.
- **Re-check condition.**
  - `git status --short` is empty on `refactor/engine`.
  - `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings` are clean.
  - `cargo test --locked` gives `193 passed; 0 failed; 0 ignored`, and `--list` shows the four tests by their new names.
  - The production-only `check src/engine '...|Instant::now|...'` from the cut addendum is still empty.
  - The tester's mutations M4, M15, M16 and M17 were already shown red in test_r1. A delta check needs only to re-run M4 and M17 against the committed versions.

### P2. A blank line splits the CHANGELOG `### Fixed` list (review m1, open)
- **What.** `CHANGELOG.md:68`, between the new KS-33 item and the existing KS-38 item.
- **Impact.** CommonMark renders the whole Fixed list as loose (each item becomes a `<p>`). Cosmetic, but it is an open review remark in a file this lane edits.
- **Return to:** `developer` (same follow-up, or its own `Docs:` commit).
- **Re-check condition.** No blank line between items in `### Fixed` under `[Unreleased]`.

## Hand-overs (not blocking this lane; for the orchestrator)

- **H1, orchestrator, right after the merge and before the next Workstream A cut:**
  - `CLAUDE.md` Layout still lists `tap_state.rs`/`chord_state.rs`/`sequence_state.rs` and says "`src/engine/` does not exist yet".
  - Principle 5 forbids a "hook-wide Mutex" with no item-2.2 exception.
  - `.claude/agents/architect.md:37` and code-reviewer R3 say the same (cut Q1/Q5, review open question 1).
- **H2, doc-writer phase book sync (if this lane does not run inside `ks-phase`, schedule it):**
  - `config/triggers-conditions.md`: §11.4 (a thread per arming), §11.5 (the Tap GC thread; DoubleTap "while the mutex is held"; the Hold line per the cut addendum), §11.6 (the KS-25 warning: equal terms are now exact), §11.8 (the KS-11 warning) and the KS-08 note citing `tap_state.rs`.
  - `config/known-issues.md` rows 23-24.
  - `architecture.md` threads table and key-event path.
  - Roadmap 2.2 "Files" (`tap_state.rs` becomes `gestures.rs`, already done).
  - `audit.md` status lines: KS-11 fixed; KS-25 and KS-33 partly.
  - Add M1-M9 below to a smoke checklist.
- **H3, record during the book sync:** the re-entry key-up residual risk, as a precondition on roadmap 2.3 and 4.6 (or in `gaps.md`), and the 3.4 follow-up "move `BindAction` into `data`".

## Manual checks for the owner (no automated equivalent: they need a live hook)

These come from `cut.md` M1-M8 plus addendum M9. They are not acceptance blockers: every acceptance line is covered by an automated test. They check "same threads, same timing".

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | release build, default config | CapsLock; Shift+CapsLock; type for a minute | language switches, CapsLock toggles, nothing stuck or lost |
| M2 | `Tap(200)` SwitchLanguage + `Hold(200)` PressKey("LShift") on CapsLock | tap 20 times about 150 ms apart; hold 1 s three times | 20 switches, no stray Shift; each hold gives one Shift and no switch |
| M3 | `F13` `DoubleTap(200)` ToggleCapsLock | 4 brisk presses, twice | 2 toggles per 4 presses |
| M4 | chord `["J","K"]` window 50; sequence `["F13","F14"]` | J+K together; F13 then F14 | each fires once, keys swallowed |
| M5 | `when: TitleContains("Notepad")` on F13 | F13 in Notepad and elsewhere | fires only in Notepad |
| M6 | `Hold(300)` `Launch(exe: "notepad.exe")` | hold 0.5 s | Notepad starts, typing does not stall |
| M7 | Task Manager thread view | press a `Tap(200)` key 50 times | thread count does not grow |
| M8 | `PressKey("Left")` on F13 | hold F13 | cursor repeats, no feedback loop |
| M9 | `Hold(300)` `Launch(exe: "notepad.exe")`, CPU loaded | hold about 300-320 ms and release, 10 times | Notepad once per press held >= 300 ms, never for shorter ones; no input stall |

## Positives

- D6' is the right fix for critique I1. Hold resolution is one clock read under one lock, and only `on_timer` fires a Hold. Hold actions stay off the hook thread, and equal Tap/Hold terms are exclusive by construction. I traced all three lock orders and got exactly one fire in each.
- Generation tokens plus per-entry deadlines remove the KS-11 class by design, and the audit's own scenarios are pinned.
- The adapter is about 100 lines of logic:
  - every return path is visible;
  - the sentinel check comes first;
  - no lock or `ArcSwap` guard is held across actions, which is better than master, where the guard was held across `execute`;
  - `with_engine` turns a future message-pumping re-entry into a pass instead of a deadlock, and the residual asymmetry is documented honestly.
- The scenario harness models the OS key state from passed events only, prints a full transcript on failure and tests itself (`a_failed_expectation_panics`). Ported tests are equal or stronger than master's.
- The process was clean: red-first commits with quoted reds, every critique remark resolved in code, and a tester round that found and closed four real coverage gaps.

## Notes for later lanes

- Every later Workstream A lane (KS-04/06/07/14) adds scenario tests in `src/engine/scenarios.rs`. Roadmap Workstream A's own `decide()` acceptance bullets (CapsLock held + `a`, repeat symmetry, `LShift`/`RWin`/chord `Shift`) belong to those lanes, not this one.
- Roadmap 2.2 must keep "only the timer fires a Hold" (or move the action to the 2.1 executor) when timers move to the main thread. Both `hold_released_after_its_term_fires_when_its_late_timer_wakes` and `hold_whose_timer_cannot_start_never_fires` encode that rule.
- Any new test that drives `get_hook()` (as M17 will) shares the process-wide singleton. Keep such tests to one, or move them to a local hook.
