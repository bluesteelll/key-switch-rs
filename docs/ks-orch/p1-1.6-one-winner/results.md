# Results analysis: p1-1.6-one-winner / roadmap 1.6 (KS-13)

## VERDICT: ACCEPT
**Summary**: Every acceptance line of roadmap 1.6 and of the brief is proven by an automated engine scenario or loader test. Each of those tests was red on the old code (the developer's logs, and again the tester's own replay) and is green in my own gate run. The election is pure and allocation-free. It runs inside the engine, and the hook callback is unchanged. Suppression after the lane is a subset of what master suppressed, so no new eaten or stuck key is possible. What remains is doc nits (three code-review Minors, one imprecise book sentence) and status lines that the Phase-1 book sync owns.

Branch `feat/one-winner` @ `f7d54ef`. Merge-base is `9357cfa` = `master` tip. The branch has 6 commits, all from this lane, and the tree is clean.

## Quality gate (own run)
| Check | Result |
|---|---|
| cargo fmt --check | OK (exit 0) |
| cargo clippy --all-targets --locked -- -D warnings | OK, exit 0. Run in a fresh `CARGO_TARGET_DIR`, a full check in 21 s, so not a cached result |
| cargo test --locked | lib 437 passed / 1 ignored; swch 0; ipc_roundtrip 6 passed; doc 0. Total **443 passed, 0 failed, 1 ignored** (`D:/tmp/ks-orch/p1-1.6-one-winner/analyst-test.txt`) |
| Test count vs master | master export (`git archive master`) `--list`: 412 lib + 6 = **418**. Branch: 438 + 6 = **444** (+26). `#[test]` attributes 409 -> 435. No `#[test]`/`#[ignore]` line removed in the diff |
| `--list --ignored` (listed, not run) | only `data::condition::tests::foreground_context_names_the_focused_notepad`, which already existed and has a `manual:` reason. The lane adds no ignore |
| cargo doc --no-deps | 4 warnings, all already present (`loader.rs:156`, `:176`, `keyboard_hook.rs:132`, `vk_name.rs:4`). None is in a hunk of this lane |
| mdbook build | OK (`D:/tmp/ks-orch/p1-1.6-one-winner/analyst-book`). Every new `#sNN` anchor exists, and `16-one-winner-per-event` / `34-core-consolidation` are in the rendered roadmap |
| Commit trailers | 6/6 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` |

## Acceptance criteria
| # | Criterion (roadmap 1.6 + brief) | Evidence | Status |
|---|---|---|---|
| 1 | Shift+CapsLock (Immediate) together with CapsLock (Tap, `allow_extra_modifiers: true`) fires only ToggleCapsLock | `engine::scenarios::shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers` (`src/engine/scenarios.rs:3412`). The held variant is `shift_capslock_held_past_the_term_fires_no_capslock_hold` (`:3443`). Mutation M12 (no one-winner) kills both | PASS |
| 2 | A conditional Pause binding wins over an unconditional one in either file order | `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order` (`:3469`), both orders via `with_context`. Mutations M1 (rank without `when:`) and M10 kill it | PASS |
| 3 | A winner with `block_default: false` lets the key through although a loser has `block_default: true` | `a_more_specific_winner_with_block_default_false_lets_the_key_through` (`:3509`, more keys, with a repeat), `a_conditional_winner_with_block_default_false_lets_the_key_through` (`:3540`, `when:`), `a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing` (`:3568`, deferred winner) | PASS |
| 4 | Tap(200)+Hold(200) on CapsLock still arm together as one group | `tap_and_hold_on_capslock_still_arm_together_as_one_group` (`:3812`, next to an Immediate loser), the existing `tap_and_hold_with_equal_terms_fire_exactly_one` (`:441`), and `a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding` (`:3609`). M2 and M7 kill them | PASS |
| 5 | Duplicates produce a warning in a loader test | `config::loader::tests::a_binding_that_duplicates_an_earlier_one_is_warned_about` (`src/config/loader.rs:2206`) and `duplicates_are_found_across_key_order_terms_timings_and_binding_kinds` (`:2243`). Negative guards: `bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates` (`:2223`), `the_template_and_its_examples_have_no_duplicate_bindings` (`:2268`). The stderr print is `loader.rs:237` (`eprintln!("[WARN] {warning}")`, main thread or `config-watcher`). Its live output is the lane's manual check M4 (`cut.md:417`) | PASS |
| 6 | Exactly one winner; precedence keys > `when:` > file order | `winner::elect` (`src/engine/winner.rs:53-82`), `rank` (`:41-43`). Unit test `rank_puts_more_keys_before_a_when_condition`. Ties: `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`scenarios.rs:1765`, M2 kills it) | PASS |
| 7 | Only the winner or its group decides execution and suppression | Rows 1-4, plus `a_more_specific_tap_keeps_a_less_specific_hold_from_arming` (`:3715`). `GroupArms` is pinned by `two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press` (`:3689`), the group's `when:` by `a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together` (`:3652`), and the member's own match by `a_group_member_arms_only_if_it_matches_the_press_itself` (`:3788`) | PASS |
| 8 | Auto-blockers keep their own rule | `a_passing_user_binding_does_not_stop_the_auto_blocker` (`:3768`) and `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`. M5 kills them | PASS |
| 9 | Update README and CHANGELOG | README "Which binding wins" + RON block, validated by `readme_matching_examples_load` (ok). CHANGELOG `[Unreleased]` has Added / Changed (Breaking) / Fixed | PASS |

## Issues claimed closed
| KS-ID | Regression test (audit scenario) | Fails on old code? | Status |
|---|---|---|---|
| KS-13 (audit.md:391) | Scenario 1, Shift+CapsLock with the README Tap/Hold pair, under exact modifiers via `allow_extra_modifiers`: W1 `shift_capslock_fires_only_...` (Tap fires on key-up) and W2 `shift_capslock_held_past_the_term_fires_no_capslock_hold` (Hold sends LShift). Scenario 2, Shift+X with `block_default: false` while X is swallowed: W4 `a_more_specific_winner_with_block_default_false_lets_the_key_through`. Scenario 3, deferred more specific + Immediate less specific, both fire: W6 `a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing` | Yes. `ede8472` (tests) precedes `006ff5e` (fix). The tester replayed the old code (`tester_red1.log:378-392`): 12 failed by assertion, 211 passed. The loader reds were replayed on `69a8406` (`tester_red4.log:143-147`): 2 failed | Closed. The audit status line moves to the Phase-1 book sync (sanctioned by `docs/WORKFLOW.md:105-106`) |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/keyboard_hook_callback.rs` is not in the diff. `keyboard_hook.rs` changes only comments (`:106-109`, `:312-315`). On the per-key-down path, `Winner` is a borrow and `GroupArms` is three bools. `KeyCombination ==` (`key_combination.rs:16-30`) and the derived `Condition ==` do not allocate. No new indexing by `vk`. Symmetry still lives in the `LastDown` record. `mod.rs:536-569` and `on_key_up` (`:572-598`) have no hunks. Suppression after the lane is a subset of master's: the winner and the members matched on master too, and blockers, sequences and chords are unchanged |
| Engine purity, matching per owner decisions | OK | `winner.rs` uses only `matching::combo_matches`, `KeyState`, `ForegroundInfo`. It is under `#![forbid(unsafe_code)]` and has no clock, lock or Win32 call. Exact modifiers and `allow_extra_modifiers` are reused unchanged. The repeat default is unchanged: an Immediate winner fires on a repeat only with `repeat` (`mod.rs:431-438`). Sided keys go through `KeySpec` |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No new thread, timer, hook, `SendInput`, atomic, `static` or `unsafe` (checked over the added lines: no `unsafe`, `SeqCst`, `Relaxed` or `unwrap()`, and no non-ASCII in `src`). The warning prints on the main thread (`main.rs:30`) or on `config-watcher`, never on the hook thread |
| Config reload, IPC | OK | `duplicate_warnings` runs only after every entry has converted (`loader.rs:292`), so a broken file still errors and keeps the live bindings. Empty or BOM files take the existing paths. IPC is untouched. Reload mid-gesture: `a_gesture_armed_by_the_winner_resolves_after_a_reload_adds_a_more_specific_binding` (`:3743`) |
| No key logging | OK | The warning prints config keys from the file (`format_raw_binding_keys`), never pressed keys |
| Hot-path gain | + | A `keys` binding's `when:` is evaluated only after its keys match. Pinned by `engine::tests::a_when_condition_is_asked_only_for_a_binding_whose_keys_match` (`src/engine/mod.rs:645`), whose `Env` panics if the foreground is read |

## Review status
- Critique r1: APPROVED (0/0/3 Minor). The developer resolved M1-M3 (`cut.md:534-541`). The M1 repeat behaviour change is pinned by `repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` (`scenarios.rs:3843`). The tester checked that this test fails on master (`tester_m1.log:44-62`).
- Code review r1: **APPROVED** (Critical 0, Important 0, Minor 3). No triage was needed, and no CONFIRMED Critical/Important item is open.
- Tester r1: **GREEN**. Gate, red-first replay, and 11 mutations, all caught (`test_r1.md`).
- Open Minors worth doing while the context is fresh (code doc comments only, no behaviour change):
  - m1 `src/engine/winner.rs:48-49`: the doc says "first candidate whose rank is strictly greater than every earlier candidate". The code picks the first candidate of the highest rank. Same wording in `cut.md` D2.
  - m2 `src/engine/mod.rs:283-285`: "auto-blockers ... stay after the file's bindings" holds only within one key count. A 2-key blocker sorts ahead of 1-key user bindings.
  - m3 `src/config/loader.rs:242-250`: `parse_ron_str` is documented as "without the printing", yet `[WARN] config contains no bindings` and the auto-blocker `[INFO]` lines still print inside it. Item 1.7 must route those too.

## Docs and hygiene
- In sync. README (which binding wins, triggers, resolution, the `block_default` row, Errors, feature bullet); `default_config.ron` (`:114-127`, the `block_default` comment, the "Replace the CapsLock binding" example comment); CHANGELOG `[Unreleased]` (Added: duplicate warning; Changed: Breaking with 6 consequences; Fixed: KS-13 and two DoubleTaps); book `config/{matching,triggers-conditions,kinds,actions,known-issues,errors,example}.md`. In known-issues, rows 10, 11 and 25 are now Fixed, and rows 69-73 are new.
- No new dependency. `Cargo.toml` and `Cargo.lock` are untouched.
- The branch sits on the `master` tip, so `--no-ff` merges cleanly. Every commit has the trailer, and commit subjects follow `<Area>: <what>`.
- Deferred to the Phase-1 book sync (sanctioned): `audit.md:36,394` status, `roadmap.md:283-289` status and `:617`, `architecture.md:207,212,543`, `glossary.md:85,401`, `gaps.md:185`, `overview.md:17`.

## Technical debt
- `winner.rs:48-49`, `mod.rs:283-285`, `loader.rs:242-250`: review Minors m1-m3 (above). Recorded only in `review_r1.md`. Fix in a follow-up commit or in 1.7's cut.
- Immediate binding shadowing a deferred group on the same keys and `when:` gives no warning (cut Q2). Recorded as known-issues row 69, pointing to roadmap 3.4 "diagnostics" (`roadmap.md:451`). This is an owner-visible question, flagged by the critic: the template's own CapsLock example is inert below the default binding unless the user replaces it.
- Re-election per auto-repeat (cut Q3, F4). Recorded as known-issues row 73 and pinned by a scenario. It has no roadmap item.
- Semantic `when:` equality (Q4, row 72) and sided-over-unsided precedence (Q6, row 71): by design, recorded.
- `sequence`/`chord` stay out of the election (Q1, F1). Recorded in `matching.md` §16.2, owned by 3.2/3.4.
- **Not recorded anywhere in the book:** the DoubleTap table is keyed by vk across presses. A pending DoubleTap entry armed by one binding (`CapsLock`, `DoubleTap`) is consumed by the next press of the same key won by another binding (`Shift+CapsLock`, `DoubleTap`), which then fires the first binding's action (`src/engine/gestures.rs:200-219`; `cut.md:462` "unchanged, 2.2 owns it"). This existed before the lane. The lane removed the only nearby wording ("the gesture tables are keyed by the key code only", old `triggers-conditions.md` §11.7). It needs a known-issues row pointing to 2.2.
- `kinds.md:91` (identical sequences share one pending entry) could mention the new warning (from `impl.md`).
- Blocker-replacement warning (Q5, F5) is handed to 1.2's cut. The stale reviewer R8 text (F6) carries over from 1.3.

## Problems
None that blocks the merge. Minor items for the orchestrator:

### N1. One imprecise book sentence (doc-writer, book sync)
- **What**: `book/src/config/actions.md` §13.9 says a passing `DoNothing` binding "still wins the press over less specific bindings ... and over a binding without `when` on the same keys". It beats a binding without `when:` only if it has a `when:` itself, or sits higher in the file.
- **Re-check**: the sentence says "if it has a `when`, over a binding without one on the same keys".

### N2. Phase-0 manual check 7 no longer tests what it says (doc-writer, book sync)
- **What**: `book/src/roadmap.md:200` row 7, "README Tap/Hold pair on CapsLock", is run on the default config. After 1.6 the pair is inert below the default `CapsLock` binding. The row would pass vacuously: 20 switches come from the Immediate binding, and no Hold happens.
- **Re-check**: the setup says "replacing the default `CapsLock` binding", as the lane's M5 (`cut.md:418`) does. The lane's manual checks M1-M5 are carried into the Phase-1 manual-check table.

### N3. Cross-press DoubleTap consumption is unrecorded (doc-writer, book sync)
- See Technical debt. **Re-check**: a known-issues row cites `gestures.rs:200-219` and roadmap 2.2.

## Positives
- The rank is explicit and independent of the sort (D2). Sequence, chord and blocker visit order are untouched, and ties still fall back to stable file order.
- Every piece of the group rule has a killer test: keys W10, `when:` W8, own-flag match W13, `GroupArms` W9, tie direction W14 and the repeat-slot test, condition rank W3. The tester confirmed all 11 mutations independently.
- Every new scenario runs `check_press_symmetry` and `check_nothing_stuck`. The `winners` property config (`properties.rs:144`) adds random scripts over competing bindings in strict and focus modes.
- D7 cuts foreground lookups on the hook thread (KS-08 budget), and the test proves it with a panicking `Env`.
- Breaking consequences are documented in four places, each with a concrete example.

## Notes for later lanes
- 1.7: route `ParsedConfig.warnings` to `log::warn!`, and move `[WARN] config contains no bindings` into `warnings` (m3).
- 2.2: the gesture state machine should key DoubleTap entries per binding or group, not per vk (N3).
- 3.4: the `CompiledBindings` diagnostics should include Immediate-vs-deferred shadowing on the same keys and `when:` (row 69). Consider a per-press winner lock (row 73).
- Analyst tooling: a shared `CARGO_TARGET_DIR` between a `git archive` export and the work tree gives false "fresh" fingerprints, because cargo compares mtimes. Use one target directory per tree.
