# Results analysis: p1-1.1-repeat / roadmap 1.1 "Per-binding repeat flag" (KS-10)

Branch `feat/repeat-flag` @ `10bcf94`, cut from `master` `dbd55ca` (merge-base = master tip). 6 commits.

## VERDICT: REWORK -> tester

**Summary**: The product change is correct, small and in the right place: one guarded `fire.push` in the Immediate arm (`src/engine/mod.rs:407-420`), the default resolved once at load time through an exhaustive `BindAction::repeats_by_default`, and the A3 suppression block left byte-identical. All four roadmap/brief acceptance bullets and KS-10 are pinned by scenario tests that were red first. The gate is green in my own run, the docs are in sync and the review is APPROVED. One spec bullet is not met: "deferred triggers ... ignore repeats; keep that and **pin it with a test**". For the `Hold` arm the pin cannot fail for the regression this lane makes plausible, which is the new `repeat` flag being honoured in the Hold arm. I checked this by mutation: the whole suite passes (386/386) while a held `Hold(200)` + `PressKey` fires twice. The fix is one test-only commit.

## Quality gate (own run, tree `D:/key-switch-rs/key-switch-rs` @ `10bcf94`, clean)
| Check | Result |
|---|---|
| cargo fmt --check | OK (exit 0; `analyst-fmt.txt`) |
| cargo clippy --all-targets --locked -- -D warnings | OK. The first run was cached (0.22 s), so I touched `src/lib.rs`, `src/main.rs` and `src/bin/swch.rs` and re-ran: `Checking key-switch-rs ... Finished` 1.78 s, exit 0, no lints |
| cargo test --locked | lib 386 passed / 1 ignored (387), ipc_roundtrip 6 passed, swch 0, doc 0; exit 0 (`analyst-test.txt`). Master: 363 lib + 6 (derived from red1.log: `5c43211` = master + 5 tests = 368). +24 tests |
| ignored (list only) | 1, pre-existing: `data::condition::tests::foreground_context_names_the_focused_notepad` (manual). None added |
| cargo doc --no-deps | 4 warnings, all pre-existing on master (`loader.rs:156`, `:176`, `keyboard_hook.rs:132`, `vk_name.rs:4`); the new intra-doc link `[BindAction::repeats_by_default]` resolves |
| mdbook build | OK (`D:/tmp/ks-orch/p1-1.1-repeat/analyst-book`); only the known mdbook-mermaid version notice. The developer's link check reported 2004 internal links with 0 broken (`impl.md:60`) |
| Trailers | 6/6 commits. Five are `Claude Opus 5.5`. `10bcf94` (tester `Tests:` commit) is `Claude Sonnet 5.5`, as the master rule `c69cebf` requires (`CLAUDE.md:105`, `.claude/agents/tester.md`) |

## Acceptance criteria
| # | Criterion (roadmap 1.1 + brief) | Evidence | Status |
|---|---|---|---|
| A1 | CapsLock down, 10 repeats, up: exactly one SwitchLanguage, every event suppressed | `engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once` (`src/engine/scenarios.rs:1633`): the shipped template via `include_str!`, `check_press_symmetry` + `check_nothing_stuck`, and `hold_with_repeats` asserting `expect_suppressed` on the down, every repeat and the up. Red on `6a8e20c` (red2.log:272). Manual #1 = Checkpoint 1 smoke #2 (`roadmap.md:306`) | PASS |
| A2 | PressKey("Backspace") held with 10 repeats fires 11 times | `press_key_backspace_held_with_ten_repeats_fires_eleven_times` (`:1641`), `expect_total_fired` 11. Green characterization; the tester's mutation M2 (flag ignored) turns it red. Manual #3 | PASS |
| A3 | `repeat: true` on SwitchLanguage restores firing on every repeat | `repeat_true_on_switch_language_fires_on_every_repeat` (`:1613`), total 11, red on `5c43211` (red1.log:313). `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` (`:1828`). Manual #5 | PASS |
| A4 | Media(VolumeUp) repeats by default, Media(PlayPause) does not | `volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not` (`:1651`), totals 4/4/1, red on `6a8e20c`. `only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default` (`:1674`, all 16 actions). `core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default` (`windows_actions.rs:322`). Manual #4 | PASS |
| S1 | Defaults per owner decision: true only for PressKey, VolumeUp, VolumeDown | `repeats_by_default` (`windows_actions.rs:165-185`): exhaustive, nested `MediaKey` match, no `_` arm. `repeat_defaults_follow_the_action_and_an_explicit_value_wins` (`loader.rs:1926`), `a_binding_takes_its_repeat_default_from_its_action` (`binding.rs:257`) | PASS |
| S2 | A non-repeating repeat is still suppressed when the key-down was (A3) | KS-06 block `mod.rs:510-543` unchanged. `hold_with_repeats` expectations. Tester M4 (suppression mutation) caught by 25 tests. `engine::properties::every_press_is_suppressed_or_passed_as_a_whole` | PASS |
| S3 | Deferred triggers, sequences and chords keep ignoring repeats; **pinned by a test** | Tap: `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` (`:1849`) + `tap_held_past_its_term_...`. DoubleTap: `:1849`. Sequence: `a_repeat_of_a_sequence_step_does_not_count_as_the_next_step` (`:1933`). Chord: `repeats_of_both_chord_keys_do_not_complete_the_chord_again` (`:1956`). I confirmed each by a leak-form mutation (below). **Hold: no test catches the flag leaking into the Hold arm** (P1) | **FAIL (Hold only)** |
| S4 | Field on `RawBinding`, `deny_unknown_fields` stays | `schema.rs:69-75` (`Option<bool>`, `#[serde(default)]`); `an_unknown_binding_field_lists_repeat_among_the_valid_names` (`loader.rs:1942`); the `repaet` row of `unknown_fields_are_errors_that_name_the_field`; `repeat_false_loads_on_every_kind_and_trigger` (`:1911`); `repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings` (`:1891`) | PASS |
| S5 | README, default_config.ron, book config page, CHANGELOG | All four updated (section Docs). `default_config_parses`, `default_config_commented_examples_load` and `readme_matching_examples_load` pass; book RON validated with cfgcheck (`impl.md:61`) | PASS |

Mutations I ran (scratch `git archive` export in the session scratchpad, own target dir; nothing in the repo touched). Mutation: `if was_repeat {` -> `if was_repeat && !binding.repeat {` (the `repeat` flag leaks into a deferred/sequence/chord arm):

| Arm (`src/engine/mod.rs`) | Result |
|---|---|
| Tap `:422` | caught: `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` |
| Hold `:434` | **NOT caught: `386 passed; 0 failed`** |
| DoubleTap `:448` | caught: `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` |
| Sequence `:464` | caught: `a_repeat_of_a_sequence_step_does_not_count_as_the_next_step` |
| Chord `:489` | caught: `repeats_of_both_chord_keys_do_not_complete_the_chord_again` |

The Hold mutant is not equivalent. My scratch probe (`Hold(200)` + `PressKey("Backspace")`, down, +500, 10 repeats 33 ms apart, up, total one Backspace) passes on HEAD and fails on the mutant with `scenario: expected the scenario to fire [PressKey(VIRTUAL_KEY(8))] in total, it fired [PressKey(VIRTUAL_KEY(8)), PressKey(VIRTUAL_KEY(8))]`.

## Issues claimed closed
| KS-ID | Regression test (audit scenario) | Fails on old code? | Status |
|---|---|---|---|
| KS-10 | CapsLock about 30 Hz: `default_capslock_held_with_ten_repeats_switches_the_language_once`. Shift+CapsLock flicker: `held_shift_capslock_toggles_capslock_once` (`:1725`). `Launch` per repeat and `Sequence` worker per repeat: `held_launch_and_sequence_hotkeys_fire_once_per_press` (`:1736`). Property: `auto_repeats_fire_only_actions_that_repeat` (`properties.rs:254`) | Yes. Red commit `6a8e20c` comes before fix `9461a41` (`git log`). The tester replayed it on the parent, whose engine is master: `371 passed; 13 failed` (red2.log:610), each for the predicted reason | CLOSED |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/*` not in the diff. On a non-repeating repeat the engine skips one `BindAction` clone and one `Vec` push under `with_engine` (`mod.rs:411-416`), so the hot path does less work. No new `expect`, indexing or panic path |
| Engine purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]` kept. Only `Binding.repeat`, an immutable `bool` resolved at load (`binding.rs:96`), is read. Slot rule: the first Immediate match takes a repeat even without firing, pinned by `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`:1761`; tester M3 caught by it alone). Exact modifiers and `allow_extra_modifiers` untouched |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No new thread, `SendInput`, atomic, `unsafe`, `static mut` or `Mutex` (grep of the added `src` lines: no match). `PressKey`/`Media` repeats still go through `press_vk` with `injected_sentinel()` |
| Config reload, IPC | OK | `repeat: true` outside `keys` + `Immediate` is a binding-numbered load error (`loader.rs:374-383`), checked after the existing checks, so a binding still reports its first error. A failed reload keeps the live bindings (unchanged path; manual #7). IPC untouched |
| No key logging | OK | No `println!`/`eprintln!` added. `Display` adds `[repeat]`/`[no repeat]` only for overrides (`binding.rs:187-196`), and the default banner is unchanged |

## Review status
- Architecture critique: CHANGES_REQUESTED (I1, M1-M4). All are resolved in the tree or in `impl.md`/`cut.md` addendum. I checked I1 in the tree: `CHANGELOG.md:241-244` and `known-issues.md` row 63 no longer promise item 2.3 for swallowed keys, and `scenarios.rs:1516` pins the case.
- Code review r1 (`review_r1.md`, HEAD `10bcf94` including the tester delta): **APPROVED** (Critical 0, Important 0, Minor 2). No triage was needed. Its M1 is CONFIRMED and is P1 below, which my mutation verifies. Its M2 (CHANGELOG reflow) is cosmetic and still open.
- Tester r1 (`test_r1.md`, against `10bcf94`): **GREEN**. Red-first replay, 10 mutations and an acceptance map. The tester noted its M10 (Hold guard deletion) but did not try the leak form, which is the one that matters.
- Reviewer open question 2 (Sonnet trailer and tester commit) is settled by master `c69cebf`: `tester.md` allows `Tests:` commits with the Sonnet trailer. `docs/WORKFLOW.md:23` still says "no commits" for the tester. That is stale process documentation on master, not a lane defect.

## Docs and hygiene
- `CHANGELOG.md` `[Unreleased]`: Added (`:48-55`), Changed **Breaking:** (`:125-131`), Fixed KS-10 (`:235-244`), and the KS-06 sentence (`:218-219`). Keep a Changelog headings are intact. `:242` is 123 characters long, not reflowed (cosmetic).
- `README.md`: field list, the triggers row, the behaviour rule, the per-binding flags row (with the toggle/modifier pitfall), the quoted error, a new error bullet and the policy bullet.
- `src/config/default_config.ron`: the trigger line, the `repeat` block and a commented `Ctrl+Alt+PageDown` `Media(Next), repeat: true` example (loads through `default_config_commented_examples_load`). The active bindings are unchanged. The em dash on the edited line was already there on master.
- Book: nine `book/src/config/*.md` pages. Row 8 is marked Fixed and rows 62-64 are added. The audit, roadmap, architecture, overview, glossary and gaps status lines are left to the post-merge book sync by design (`docs/WORKFLOW.md:103-107`, `cut.md:199`).
- No new dependency; `Cargo.toml`/`Cargo.lock` untouched. The branch contains only this lane's work and can merge `--no-ff` without conflict (base = master tip).

## Technical debt
- `book/src/config/known-issues.md` row 63 / `scenarios.rs:1516`: after a lost key-up of a **swallowed** trigger key (lock screen or secure desktop while it is held), the next press reads as a repeat and now fires nothing for a non-repeating binding; on master it fired. Recorded as Open with "no roadmap item". **Owner question Q1 is pending** (`cut.md:511-514`): accept it, or extend roadmap 2.3's acceptance to swallowed keys. Not in `roadmap.md`. The orchestrator should put it to the owner.
- Critic open question 2 (`critique.md:141-145`): after a focus change mid-hold, a `when:`-gated pair on one key hands the repeats to the repeating fallback (same as master). Proposed as a note for item 1.6. **Not recorded in `roadmap.md` 1.6, so it is lost unless the orchestrator or the book sync adds it.**
- Critic open question 1 (`critique.md:137-140`): for a key whose key-down passed, the OS key state can tell a lost-key-up press from a real repeat. That is KS-29/2.3 scope. **Not recorded in roadmap 2.3.**
- `engine::testing::check_repeat_fire` checks only `on_key` fires, not timer fires on repeat steps (`test_r1.md:92`). It is test-harness scope; record it for item 3.4, which reuses the check.
- Post-merge book sync list: `impl.md:94` (audit `:33`/`:349`, roadmap `:614` plus a 1.1 status block, architecture, glossary, overview, gaps; replace `feat/repeat-flag` with the merge sha in the config pages). The known-issues heading 19.9 retitle (`impl.md:96`) is for the doc-writer to judge.
- No new `TODO`/`FIXME` in the diff.

## Problems

### P1. The Hold arm's "ignores repeats" pin does not catch the `repeat` flag leaking into it
- **What**: The spec requires "Deferred triggers, sequences and chords already ignore repeats; keep that and pin it with a test". The cut says `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` "goes red if the `PressKey` default leaks into the deferred ... arms" (`cut.md:280`). That is false for its Hold subcase (`src/engine/scenarios.rs:1867-1879`). It sends 3 repeats at +500/+533/+566 ms and releases at +599 ms, before a Hold re-armed at the first repeat (+500 + 200 = +700) could fire. With `src/engine/mod.rs:434` mutated to `if was_repeat && !binding.repeat { continue; }`, the full lib suite passes (`386 passed; 0 failed`). The Hold guard-deletion case is caught only by `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:2225`), which uses a non-repeating `Launch` and so cannot see the leak.
- **Impact**: A later change that applies the flag to deferred triggers, for example the next lane on this code (1.6, "one winner per event", which regroups these trigger arms), passes every test. Meanwhile a held `( keys: "F13", action: PressKey("Backspace"), trigger: Hold(200) )` re-fires about every 233 ms (+200, +700, +933, ...). That is extra keystrokes on the owner's machine. On the current code the behaviour is correct.
- **Root cause**: The Hold subcase is too short. `fire_hold` removes the entry at the term (`gestures.rs:173-179`), so a leaked `arm_hold` on a repeat starts a fresh term, and `release_hold` cancels it (`gestures.rs:158-169`) because the key-up arrives before first repeat + term. The test therefore never reaches the instant where the leak shows. The tester's mutation M10 tried only guard deletion, not the leak form, so the gap was noted but not closed.
- **Return to**: `tester`. The evidence is weak; the code and the spec are sound.
- **Re-check condition**:
  1. One test-only `Tests:` commit (Sonnet trailer) that lengthens the Hold subcase of `deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, or adds a scenario. With `PressKey("Backspace")` on `Hold(200)` the key is held with repeats past first repeat + term (at least 7 repeats 33 ms apart, so the release comes after +700 ms) and `expect_total_fired` is exactly one Backspace.
  2. Replayed with `src/engine/mod.rs:434` mutated to `if was_repeat && !binding.repeat {`, it fails: expected `... it fired [PressKey(VIRTUAL_KEY(8)), PressKey(VIRTUAL_KEY(8))]`. It is green on HEAD.
  3. Full gate green, no production change, lib count >= 387.

Optional, not blocking: reflow `CHANGELOG.md:242` (reviewer M2, cosmetic). The doc-writer can do it in the post-merge book sync.

## Positives
- The decision stays in the pure engine, and the adapter is untouched. The hot path loses a clone and a push per non-repeating repeat.
- The slot rule (a non-repeating winner still takes the repeat) prevents a fall-through to a later binding at 30 Hz. It is reasoned, documented and pinned with a single-purpose test (M3).
- The exhaustive `repeats_by_default` makes a future `BindAction` variant a compile error until its default is decided.
- The red-first discipline is clean, with exact predicted messages. The master-replay difference (critic M3) is spelled out in the commit body.
- The tester found and closed the sequence and chord leak gap (`10bcf94`), and the reviewer traced the remaining Hold gap precisely.
- The docs are honest about the KS-29 corner and the `PressKey` toggle/modifier pitfall (row 64).

## Notes for later lanes
- 1.6 restructures the trigger arms. Re-run the five leak-form mutations above (Tap/Hold/DoubleTap/Sequence/Chord) after the refactor.
- 3.4 can reuse `Scenario::check_repeats_fire_only`. Extend it to timer fires on repeat steps.
- `docs/WORKFLOW.md:23` (tester "no commits") contradicts `.claude/agents/tester.md` (`Tests:` commits allowed). Fix it in a chore branch.
- Q1 (KS-29, swallowed keys) needs an owner answer before 2.3 is cut.
