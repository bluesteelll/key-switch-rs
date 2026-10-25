# Results analysis v2 (re-check after rework 1): p1-1.1-repeat / roadmap 1.1 "Per-binding repeat flag" (KS-10)

Branch `feat/repeat-flag` @ `5879bda` (previous verdict at `10bcf94`). Merge-base `dbd55ca`. Seven commits. The rework delta `10bcf94..5879bda` is one test-only commit: `src/engine/scenarios.rs`, +22 lines.

## VERDICT: ACCEPT

**Summary**: The one problem from the previous verdict (P1: the Hold arm's "ignores repeats" pin did not catch the `repeat` flag leaking into it) is resolved. The new scenario `held_hold_binding_with_repeating_action_fires_once_across_many_repeats` holds the key past first repeat + term. I re-ran the named mutation on a clean export, and the new test is now the one test that fails, with the exact predicted message. The other four leak mutations and the Hold guard-deletion mutation are still caught. The full gate is green on a clean `git archive` export with its own `CARGO_TARGET_DIR`. The lane tree is clean. No production code changed since the APPROVED review, so every other finding of `results.md` still holds.

## Quality gate (own run, clean export of `5879bda`, `CARGO_TARGET_DIR` = scratchpad `v2/target-head`)
| Check | Result |
|---|---|
| cargo fmt --check | OK, exit 0 (`analyst-v2-fmt.txt`) |
| cargo clippy --all-targets --locked -- -D warnings | OK, exit 0. Fresh target dir, so `Compiling key-switch-rs v0.1.0 (...v2\head)` ran in full: no cache, no lints (`analyst-v2-clippy.txt`) |
| cargo test --locked | exit 0. lib `running 388 tests`: 387 passed, 0 failed, 1 ignored. ipc_roundtrip 6 passed; swch 0; doc 0 (`analyst-v2-test.txt:41`, `:431`, `:449`). Master lib: 363 (master moved only `.claude/workflows/ks-lane.js`, see Hygiene). +25 tests |
| ignored (list only) | 1, pre-existing: `data::condition::tests::foreground_context_names_the_focused_notepad` (`src/data/condition.rs:519`, `manual:` reason). None added (`analyst-v2-ignored.txt`) |
| cargo doc --no-deps | 4 warnings, the same pre-existing ones as on master (`loader.rs:156`, `:176`, `keyboard_hook.rs:132`, `vk_name.rs:4`) |
| mdbook build | Not re-run. The delta touches no book file. The previous run was OK (`results.md:17`) |
| Trailers | 7/7. `5879bda` and `10bcf94` (`Tests:` commits, tester) carry `Claude Sonnet 5.5`; the other five carry `Claude Opus 5.5` |
| Lane tree | `git status`: clean. `git worktree list`: only `D:/key-switch-rs/key-switch-rs 5879bda [feat/repeat-flag]`. The tester's scratch dirs `D:/tmp/ks-orch/p1-1.1-repeat/wt/` (empty) and `target-wt/` are outside the repo |

## Re-check of P1 (previous verdict)
| Re-check condition (`results.md:88-90`) | Evidence | Status |
|---|---|---|
| 1. One test-only `Tests:` commit with the Sonnet trailer: `PressKey("Backspace")` on `Hold(200)`, held with at least 7 repeats 33 ms apart (release after +700 ms), `expect_total_fired` exactly one Backspace | `5879bda`, `src/engine/scenarios.rs:1932-1952`: down, `advance_ms(500)` with one Backspace fired at the term, then `repeats(scenario, "F13", 10, &[])` (repeats at +500 .. +797, each `expect_suppressed` + `expect_fired(&[])`, helper `scenarios.rs:1582-1591`), up at +830 `expect_suppressed` + `expect_nothing_fired`, `expect_total_fired([Backspace])`. The diff touches only this file. The existing test is unchanged | MET |
| 2. With `src/engine/mod.rs:434` mutated to `if was_repeat && !binding.repeat {` the test fails with `... it fired [PressKey(VIRTUAL_KEY(8)), PressKey(VIRTUAL_KEY(8))]`; green on HEAD | Own run (`analyst-v2-mutations.txt`): `test result: FAILED. 386 passed; 1 failed`. The only failure is `held_hold_binding_with_repeating_action_fires_once_across_many_repeats`, panicked at `scenarios.rs:1951:10`: `scenario: expected the scenario to fire [PressKey(VIRTUAL_KEY(8))] in total, it fired [PressKey(VIRTUAL_KEY(8)), PressKey(VIRTUAL_KEY(8))]`. On HEAD it passes (`analyst-v2-test.txt:282`) | MET |
| 3. Full gate green, no production change, lib count >= 387 | Gate above. `git diff --stat 10bcf94 5879bda` shows `src/engine/scenarios.rs | 22 +` only, inside `#[cfg(test)]` scenario code. Lib count 388 | MET |

### Mutation replay (all mutations the previous verdict named, plus the guard deletion)
These ran on a separate clean export (`v2/mut`, own target dir `v2/target-mut`), one mutation at a time. The file was restored from the HEAD export after each run, and `diff -r` confirms the tree is back to HEAD. Nothing in the repo was touched.

| Mutation in `src/engine/mod.rs` | Result | Caught by |
|---|---|---|
| Hold `:434` `if was_repeat && !binding.repeat {` (leak) | FAILED 386/1 (was 386/0 at `10bcf94`) | `held_hold_binding_with_repeating_action_fires_once_across_many_repeats` |
| Tap `:422` leak | FAILED 385/2 | `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` |
| DoubleTap `:448` leak | FAILED 386/1 | `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` |
| Sequence `:464` leak | FAILED 386/1 | `a_repeat_of_a_sequence_step_does_not_count_as_the_next_step` |
| Chord `:489` leak | FAILED 386/1 | `repeats_of_both_chord_keys_do_not_complete_the_chord_again` |
| Hold `:434` guard deleted (`if false {`) | FAILED 385/2 | `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed`, `held_hold_binding_with_repeating_action_fires_once_across_many_repeats` |

Every repeat guard of a deferred, sequence or chord arm is now pinned against both the leak form and deletion.

## Acceptance criteria (unchanged except S3)
| # | Criterion (roadmap 1.1 + brief) | Evidence | Status |
|---|---|---|---|
| A1 | CapsLock down, 10 repeats, up: exactly one SwitchLanguage, every event suppressed | `default_capslock_held_with_ten_repeats_switches_the_language_once` (`scenarios.rs:1633`), red on `6a8e20c` | PASS (unchanged) |
| A2 | PressKey("Backspace") held with 10 repeats fires 11 times | `press_key_backspace_held_with_ten_repeats_fires_eleven_times` (`:1641`) | PASS (unchanged) |
| A3 | `repeat: true` on SwitchLanguage fires on every repeat | `repeat_true_on_switch_language_fires_on_every_repeat` (`:1613`), red on `5c43211` | PASS (unchanged) |
| A4 | Media(VolumeUp) repeats by default, Media(PlayPause) does not | `volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not` (`:1651`), `only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default` (`:1674`), `windows_actions.rs:322` | PASS (unchanged) |
| S1 | Defaults per owner decision | exhaustive `BindAction::repeats_by_default` (`windows_actions.rs:165-185`), loader and binding tests | PASS (unchanged) |
| S2 | A non-repeating repeat is still suppressed (A3 invariant) | KS-06 block `mod.rs:510-543` untouched; `hold_with_repeats` and `repeats` expect `expect_suppressed` on every repeat; property `every_press_is_suppressed_or_passed_as_a_whole` | PASS (unchanged) |
| S3 | Deferred triggers, sequences and chords keep ignoring repeats; pinned by a test | Tap, DoubleTap, Sequence, Chord as before; **Hold: `held_hold_binding_with_repeating_action_fires_once_across_many_repeats` (`scenarios.rs:1933`)**. All five leak mutations are caught (table above) | **PASS** (was FAIL for Hold) |
| S4 | Field on `RawBinding`, `deny_unknown_fields` stays | `schema.rs:69-75`, loader tests | PASS (unchanged) |
| S5 | README, default_config.ron, book config page, CHANGELOG | unchanged since `results.md` (all four in sync) | PASS (unchanged) |

## Issues claimed closed
| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-10 | as in `results.md:48` (`default_capslock_held_with_ten_repeats_switches_the_language_once`, `held_shift_capslock_toggles_capslock_once`, `held_launch_and_sequence_hotkeys_fire_once_per_press`, property `auto_repeats_fire_only_actions_that_repeat`) | Yes: red commit `6a8e20c` precedes fix `9461a41` (`git log --oneline master..feat/repeat-flag`). The new Hold test is a guard pin. Hold ignored repeats on master too, so the test cannot be red-first; the mutation stands in for that | CLOSED |

## Invariants
| Area | Status | Notes |
|---|---|---|
| Hook callback | OK | `src/hook/*` not in the branch diff (unchanged since `results.md`) |
| Engine purity, owner decisions | OK | No production change in the delta |
| Threads, sentinel, atomics, SAFETY | OK | No `unsafe`, atomics or threads added in the delta |
| Config reload, IPC | OK | Unchanged |
| No key logging | OK | Unchanged |

## Review status
- Code review r1: APPROVED at `10bcf94` (Critical 0, Important 0). Its M1 (the Hold pin gap) was CONFIRMED and is now resolved in the tree. I reviewed the 22-line test delta in full: it uses the existing `repeats` helper and `std::slice::from_ref(&back)` like its neighbours, is ASCII-only and adds no `TODO`. The delta is test-only and well inside the reviewed surface, so no further review round is needed.
- Tester: rework round 1 (`rework_1.md`) is green, with a red-under-mutation check. I reproduced both independently.
- Reviewer M2 (`CHANGELOG.md:242` reflow): still open, cosmetic. Per `rulings.md` it goes to the phase book sync.

## Docs and hygiene
- No doc change in the delta. The `results.md` findings stand: CHANGELOG `[Unreleased]` Added/Changed/Fixed, README, `default_config.ron` and nine `book/src/config/*.md` pages are in sync.
- **Master moved** since the previous verdict: `master` = `2a207a3` (`f63a33f` + merge), merge-base `dbd55ca`. The master delta touches only `.claude/workflows/ks-lane.js`, and the branch delta touches none of that (`git diff --name-only` of both sides is disjoint). So `git merge --no-ff feat/repeat-flag` has no conflict, and the master test count is unchanged (363 lib).
- No new dependency, and `Cargo.lock` is untouched (`--locked` passes).

## Technical debt
- `book/src/config/known-issues.md:131` (row 63, KS-29 corner). The orchestrator ruled it ACCEPTED (`rulings.md:3-6`) with "The root fix is roadmap item 2.3". **The row itself, and roadmap 2.3's text (`roadmap.md:377`: the resync skips "blocked ones"), say the opposite: 2.3's key resync leaves a swallowed trigger key alone, so 2.3 as written does not repair this case.** Its acceptance (`roadmap.md:386`, "Nothing is stuck after Win+L") does not pin it either. This does not block the lane, because the docs describe the behaviour honestly. Before 2.3 is cut, the phase book sync or the 2.3 architect should do one of two things: add a 2.3 acceptance bullet ("after Win+L with a swallowed non-repeating trigger key held, the next press fires"), or correct the ruling's premise. Q1 was framed as a scope question for the owner, so it is for the orchestrator to confirm the ruling with the owner.
- Critic notes for 1.6 (focus change mid-hold hands repeats to the repeating fallback) and 2.3 (the OS key state tells a lost key-up from a repeat): `rulings.md:7-8` defers them to the phase book sync. They are still not in `roadmap.md`, so the sync list must carry them.
- `engine::testing::check_repeat_fire` checks only `on_key` fires, not timer fires on repeat steps. It is test-harness scope; record it for item 3.4.
- Minor (not blocking, optional for a later touch): the header comment of `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` (`scenarios.rs:1850-1851`) says "the flag must not reach the deferred ... arms", but its own Hold subcase (`:1867-1879`) cannot see that; the new test does. A one-line cross-reference would keep a future reader from trimming the new test as redundant.
- No new `TODO`/`FIXME`.

## Problems
None open. P1 is resolved (see "Re-check of P1").

## Positives
- The rework is minimal and exact: one new scenario. Its comment states the timing argument (+500 repeat, +700 leaked term, release at +830), so the reason for its length is recorded where a future trimmer would see it.
- The tester verified red-under-mutation before reporting. I reproduced the failure message byte for byte.

## Notes for later lanes
- 1.6 regroups these trigger arms. Re-run the six mutations in the table above after the refactor; all six are expected to stay caught.
- Fix the KS-29 / 2.3 premise mismatch (Technical debt, first item) before 2.3 is cut.
- `docs/WORKFLOW.md:23` (tester "no commits") still contradicts `.claude/agents/tester.md`. Fix it in a chore branch.
