# Results analysis: p0-A2-matching (Workstream A: trigger-key rule KS-04, sided keys KS-07, exact modifiers + `allow_extra_modifiers`)

Branch `fix/matching` @ `c8ca4a5`, cut from `master` @ `e67e5cb` (merge-base = master HEAD). 14 commits, 20 files, +2434/-324.

## VERDICT: ACCEPT

**Summary**: Every acceptance line of the brief is pinned by a `Engine::on_key` scenario test. Each of the 26 lane scenarios fails on master by assertion (my own replay) and passes on HEAD. The gate is green in my own run (fmt, clippy, 263/263 tests against 193 on master). The design canonicalises trigger keys once, at load time (`KeySpec`), and keeps `decide()` pure. It adds no allocation, lock or Win32 call on the keyboard path and does not touch the hook adapter's production code. The lane also fixed KS-06, because exact matching and live modifier triggers would otherwise have opened new stuck-key paths. That fix went through a full critique, review, triage and fix cycle and closed with review r2 APPROVED and tester r2 GREEN. What remains for the orchestrator is listed below: one commit with the wrong trailer that cannot be fixed under the git flow, a minor docs overclaim, the book sync, and an owner FYI.

## Quality gate (own run)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (lane tree; again on a `git archive` export) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK. Fresh export, own target dir; re-run after `touch` shows `Checking key-switch-rs ... Finished`, exit 0 |
| `cargo test --locked` | **263 passed, 0 failed, 0 ignored** (lib); swch bin and doc-tests 0 (`D:/tmp/ks-orch/p0-A2-matching/analyst-test.txt`). The same on a fresh export with its own target dir. **Master `e67e5cb`: 193** (`--list` on an export) |
| `cargo test --locked -- --list --ignored` | 0 ignored tests (none added, none existed) |
| `cargo doc --no-deps` | 1 warning, pre-existing: `src/data/vk_name.rs:4` unclosed `<raw>` (untouched) |
| mdbook | not run: the lane does not touch `book/` |
| Red-first replay (own) | HEAD's `src/engine/scenarios.rs` on master's engine (export `analyst/master`): `73 tests, 47 passed, 26 failed`. All 26 are lane scenarios, each failing by assertion (`analyst/master-replay.txt`) |

Test delta: 193 -> 263 (+70). Six master tests are gone, and only these six. Each tested an API the cut removes (D1/D4):
- `key_combination::tests::{test_single_key, test_key_combination, test_extra_keys}`. `test_extra_keys` asserted superset matching, which is the KS-04 bug.
- `keys::tests::{active_keys_dedup, active_keys_capacity_bound, active_keys_fold_sided_modifiers}`, for the removed `ActiveKeys`/fold.

Their behaviour is covered by `engine::matching::tests` (8), `engine::keys::tests` (3 new) and `data::key_combination::tests` (5). No test was weakened or newly ignored. Only one existing test changed: the comment of `tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release`, whose assertions are unchanged.

## Acceptance criteria

Paths: `src/engine/scenarios.rs` = S, `src/config/loader.rs` = L (HEAD line numbers).

| # | Criterion (brief / roadmap A) | Evidence | Status |
|---|---|---|---|
| 1 | With CapsLock held, `a` passes through and nothing fires | `key_pressed_while_an_immediate_binding_key_is_held_passes` S:968 (red on master: `down A at +30 ms ... suppressed`); `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` S:986 (README Tap/Hold pair). Manual: roadmap Checkpoint 1 smoke 3; impl.md M1 | PASS |
| 2 | The CapsLock binding does NOT fire while Shift is held | `capslock_binding_does_not_fire_while_shift_is_held` S:1079 (red on master); deferred variant `deferred_binding_does_not_arm_while_shift_is_held` S:1202 | PASS |
| 2b | ... and does with `allow_extra_modifiers: true` | `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held` S:1096; `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers` S:1229; `allow_extra_modifiers_true_is_parsed` L:1297; `allow_extra_modifiers_defaults_to_false` L:1289. Tester mutation M5 (flag ignored) is caught | PASS |
| 3 | Ctrl+J does not fire on Ctrl+Shift+J | `ctrl_j_does_not_fire_on_ctrl_shift_j` S:1108 (red on master: `down J ... suppressed`); guard `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j` S:1124 | PASS |
| 4 | LShift and RWin in `keys` fire only for that side, and Shift fires for both | `lshift_in_keys_fires_only_for_the_left_shift` S:753, `rwin_and_lwin_in_keys_fire_only_for_their_side` S:776, `lshift_double_tap_fires_only_for_the_left_shift` S:842 (all red on master); `unsided_shift_and_win_in_keys_fire_for_either_side` S:811; `either_shift_counts_as_the_same_modifier` S:1136. Manual: smoke 6; impl.md M4, M5 | PASS |
| 5 | Shift in a chord fires | `unsided_shift_in_a_chord_fires_for_either_side` S:872 (red on master); `sided_shift_in_a_chord_is_only_its_side` S:901; `chords::tests::unsided_shift_chord_completes_with_either_shift` chords.rs:266; `chord_with_an_unsided_modifier_parses_to_either_side` L:1254 | PASS |
| 6 | Shift+CapsLock fires ToggleCapsLock | `shift_capslock_fires_toggle_capslock_with_either_shift` S:1173 (both Shifts); `more_specific_immediate_binding_wins` S:100 | PASS |
| 7 | Sided rule in sequence steps (spec "everywhere") | `sided_shift_in_a_sequence_step_matches_only_its_side` S:913, `win_in_the_last_sequence_step_does_not_swallow_its_key_up` S:945; `sequences::tests::sided_step_needs_its_side` | PASS |
| 8 | Trigger-key rule for Immediate and deferred, with deferred arms keyed on the trigger key | `deferred_combo_arms_only_on_its_trigger_key` S:1046, `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock` S:1011, `modifier_pressed_last_does_not_fire_a_combo` S:1031, T11 above; unit tests `matching.rs:110-143` | PASS |
| 9 | Roadmap A: "a repeat of a suppressed key is suppressed, and its key-up is suppressed exactly once" (KS-06, added by the lane) | `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` S:1327, `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once` S:1352 (the next L press passes) | PASS |
| 10 | README combo-syntax / sided-modifier sections, `default_config.ron` comments, CHANGELOG with `Breaking:` | README:175-230, flags row, features; `default_config.ron:32-60`, `:126-129`, `:219-221`; CHANGELOG Added / Changed (two `**Breaking:**` entries) / Fixed. Machine checks: `readme_matching_examples_load` L:1312 (`include_str!` of the real README), `default_config_parses` L:478 | PASS |

Every acceptance line has automated evidence. Manual checks M1-M10 (impl.md) go beyond the brief. M1, M4 and M8 map to Checkpoint 1 smoke 3, 6 and 2. The rest (M2, M3, M5, M6, M7, M9, M10) appear in no checklist; see Notes.

## Issues claimed closed

| KS-ID | Regression test(s) | Fails on old code? | Status |
|---|---|---|---|
| KS-04 (both symptoms: rollover re-fire/eat, Shift-while-CapsLock fires `Shift+CapsLock`; deferred arms on foreign keys) | `key_pressed_while_an_immediate_binding_key_is_held_passes`, `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`, `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock`, `modifier_pressed_last_does_not_fire_a_combo`, `deferred_combo_arms_only_on_its_trigger_key` | Yes. Own replay on master: `down A at +30/+50 ms`, `down LShift at +0/+350 ms`, `down LCtrl at +0 ms ... suppressed`. The red commit `46424a1` precedes the fix `2271db2` | CLOSED |
| KS-07 (sided names never match; unsided chord keys never complete) | `lshift_in_keys_...`, `rwin_and_lwin_...`, `lshift_double_tap_...`, `unsided_shift_in_a_chord_...`, `sided_shift_in_a_sequence_step_...`, `chord_keys_that_overlap_are_rejected` L:1267 | Yes. Replay: `down F13/F14 ... expected suppressed, passed`, `down LShift ... expected suppressed, passed`. Red `1bc3b0e` precedes fix `709d739` (the loader test was replayed on the red commit by the tester) | CLOSED |
| KS-06 (repeat leaks after a swallowed key-down; key-up swallowed) | `double_tap_lshift_held_past_the_repeat_delay_...`, `chord_keys_held_past_the_repeat_delay_...`, `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` (the audit's README Tap/Hold case), `immediate_combo_that_stops_matching_mid_hold_...` (the audit's Ctrl+Alt+L case), `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (the audit's auto-blocker case), plus the I2 and focus-change scenarios S:1248, S:1408-1515 | Yes, all red on master in my replay. Reds `f4b2582`/`faf3be1` precede the fix `20672b7`; red `c59a110` precedes `938df54` | CLOSED for the audit's scenarios. Residual gaps (focus change plus a pending deferred gesture; KS-14) are documented, see Technical debt |
| KS-32 (partly) | `win_in_the_last_sequence_step_does_not_swallow_its_key_up` S:945 | Yes (`up Win ... suppressed` on master) | PARTLY, as claimed. The `"A+S"` rest stays with 3.2 |

Not claimed but achieved as side effects (for the book sync): exact auto-blockers (roadmap A bullet "Makes auto-blockers match exactly", `auto_blocker_matches_its_modifiers_exactly` S:1148; KS-18 dedup not done), and the "phantom key fires unrelated combos" half of KS-29, which the trigger-key rule removes.

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/keyboard_hook_callback.rs` changes only in `mod tests` (D10: `.with_allow_extra_modifiers(true)` against live `GetAsyncKeyState` flakes). Engine matching is table reads through `slot()` (`keys.rs:36-39`, bound `< 256`). `matching.rs:46-67` does not allocate. The 256-slot `active_keys` scan and the per-event chord `canonical` Vec are gone. `clear_all_blocked` (256 steps) runs only on the WinEvent path. Symmetry: a first key-down records `Suppressed`/`Passed` (`mod.rs:451-461`), a repeat follows the record (`mod.rs:446-450`), chord completion marks only `seen` codes that were suppressed as `Pending` (`chords.rs:86-99`), and `last_step_keys` skips modifiers (`sequences.rs:125-139`). No new stuck-key path against master; several removed. Remaining pre-existing paths are listed under Technical debt |
| `decide()` purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]` kept (`mod.rs:138`); no Win32, clock or thread in `matching.rs`/`key_spec.rs`. Exact family matching with per-binding `allow_extra_modifiers` (`matching.rs:58-67`); auto-blockers stay exact (`binding.rs` `new_auto_blocker`). Sided normalisation is in one place, `KeySpec` (`data/key_spec.rs:134-170`), used by combos, steps and chords. Repeat defaults untouched (1.1). RON kept |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No new `unsafe`, atomic, static, lock, thread, `SendInput` or hook install. Output keys stay `VIRTUAL_KEY` (`parse_key_list`, `parsing.rs:23-33`), so no trigger-only code can reach `SendInput`. Registry: reads only; `win_with` = `AnySide(Win)` (`registry.rs:203-206`) |
| Config reload, IPC | OK | New field `#[serde(default)]` (`schema.rs:56-62`). `true` on sequence/chord is an error that names the binding (`loader.rs:229-236`, `err_prefix` = `binding #N [keys]`). Overlapping chord keys are an error naming both indices (`loader.rs:303-314`). IPC untouched |
| No key logging | OK | The only new `println!` is the registry INFO line (`registry.rs:56`), which prints the hotkey combo, not pressed keys. Test-only `unwrap()`s; source stays ASCII (no non-ASCII in added `src` lines) |

## Review status

- Critique: CHANGES_REQUESTED (I1, I2 Important; M1-M4 Minor). All were resolved in code (C7-C9) or docs (cut.md developer addendum; impl.md "Remarks").
- Review r1: CHANGES_REQUESTED (I1 Important; M1, M2 Minor). Triage r1: I1 CONFIRMED (repeat after a focus change passed and fired, a regression from C9); I1(c) DEFERRED (KS-29, roadmap 2.3); M1, M2 NOTED.
- Fix r1: I1 resolved with the three-state `LastDown` record (`keys.rs:41-51`, `mod.rs:446-461`), four red-first scenarios plus a guard, and narrowed docs.
- **Review r2: APPROVED** (Critical 0, Important 0, Minor 2). **Tester r2: GREEN** (263 passed; 8 mutations, all caught; the I1 reds replayed on `c59a110`).
- No CONFIRMED Critical or Important item is open.
- Open Minor items, for the orchestrator:
  - **r2 m1** (docs overclaim; behaviour pre-existing and identical on master). See P2 below.
  - **r1 M2 / r2 m2**: wrong trailer on `965ac4d`. See P1 below.

## Docs and hygiene

- README, `default_config.ron` and CHANGELOG are updated in the commits that change the meaning (C2, C4, C6, C9, `c8ca4a5`).
- CHANGELOG follows Keep a Changelog under `[Unreleased]`: Added (the flag); Changed (two `**Breaking:**` entries plus the INFO line); Fixed (KS-04, KS-06, KS-07, KS-32 partly).
- The README anchor `#combo-syntax-the-keys-field` matches the heading `### Combo syntax (the \`keys\` field)` (README:175).
- No dependency, `Cargo.toml`/`Cargo.lock`, `book/`, `docs/` or `.claude/` change.
- Commit subjects follow `<Area>: <what>`. Red-first order holds for every fix.
- The branch is clean to merge: master is an ancestor, so `--no-ff` produces no conflicts.
- **Book**: not edited. Cut Q2 deferred it to the phase book sync (WORKFLOW.md:105-107, `ks-phase` `bookSync`), as A1 did, and the brief's spec lists only README, template and CHANGELOG. The work list:
  - cut.md "Docs impact": `config/keys.md`, `kinds.md`, `matching.md`, `structure.md`, `errors.md`, `triggers-conditions.md`, `known-issues.md` rows, `example.md`, `architecture.md` key-event path, `audit.md` status lines for KS-04 and KS-07, `roadmap.md` A bullets.
  - impl.md "Docs": the KS-06 status line, `known-issues.md` KS-06 rows, roadmap A "Suppresses repeats symmetrically", and the `architecture.md` repeat rule.
- **Trailer**: 13 of 14 commits carry `Co-Authored-By: Claude Opus 5.5`. `965ac4d` (tester) carries `Claude Sonnet 5.5`. See P1.

## Technical debt

No new `TODO`/`FIXME`. Accepted limitations and deferrals:
- KS-10: on the I2 path (CapsLock pressed with Ctrl, Ctrl released first), the repeats fire the Immediate action and pass. Master fired on these repeats too, while suppressing them. Recorded in CHANGELOG (KS-06 entry) and impl.md deviation 2; owned by roadmap 1.1.
- KS-14: a modifier whose Tap or Hold is still pending can be left held (`"Ctrl+Shift"` + `Hold(1000)`, Ctrl released first). Same on master (triage p4). Recorded in the README gap bullet (README:364-367) and CHANGELOG; owned by the KS-14 item of Workstream A. Not pinned, because the scenarios header excludes known defects.
- Focus change while a deferred-bound key is held past the repeat delay, with a Tap or Hold term longer than that delay: repeats pass and the release is swallowed, so the key is stuck. Same on master. **Recorded only in fix_r1.md and review_r2.md (m1)**; README/CHANGELOG say the opposite (P2). Owner: roadmap 2.3, whose acceptance should pin it.
- After a focus change, the release of a key whose repeats an Immediate binding swallowed now passes (master swallowed it): a lone key-up. Done by design (fix_r1.md "Deviations"), owned by 2.3. Review r2 open question: does any bindable key act on a lone key-up (PrintScreen is a candidate)? Not recorded in the book.
- KS-32 rest (`"A+S"` last step marks a passed A): impl.md open point; audit KS-32; owner 3.2.
- KS-15/KS-16 (sequence trigger and exact rules), chord blocked-bit hygiene: owner 3.2 (cut non-goals).
- A `keys: "LShift"` DoubleTap swallows every left-Shift press until 1.3. README:315-318 and the `default_config.ron` warning.
- AltGr fake LCtrl: README note; new feature 2.
- Optional trigger check before `condition.evaluate` (3.5): critique open question 2, not done.
- CLAUDE.md "Layout" (lacks `key_spec.rs`, `matching.rs`) and code-reviewer R8 (folding, "any Win is `VK_LWIN`") are stale after merge. Cut Q4; the orchestrator updates them.

## Problems

### P1. Commit `965ac4d` carries `Co-Authored-By: Claude Sonnet 5.5` instead of the required Opus trailer
- **What**: CLAUDE.md:104-105 and ks-lane.js:107 require `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` on every commit. The brief's count gives 13/14.
- **Impact**: hygiene only; the code is unaffected. It cannot be repaired: the git flow forbids amend, rebase and force, and a follow-up commit cannot change it.
- **Root cause**: the tester role (Sonnet) is told to add "the Co-Authored-By trailer" (`.claude/agents/tester.md:22`, ks-lane.js:134) without the literal string, so it wrote its own model name.
- **Return to**: none for this lane. The orchestrator records the exception and spells out the literal trailer in `tester.md` and the ks-lane tester brief.
- **Re-check condition**: later lanes count 100% Opus trailers.

### P2. README/CHANGELOG/rule 5 say a release "reaches Windows" after a focus change; for a key with a Tap or Hold still pending, it is swallowed after its repeats passed
- **What**: README:360-363, CHANGELOG.md:98-102 and `src/engine/mod.rs:121-123` state this. Repro (review r2 m1, traced; I re-traced it against `mod.rs:354-357`, `:446-450`, `:481-487`): `keys: "CapsLock"`, `Hold(1000)`; down; a focus change at +100; a repeat at +500 passes (CapsLock turns on); the key-up at +700 is `Cancelled` and suppressed, so CapsLock stays held in Windows.
- **Impact**: a stuck-key edge case that the docs describe as safe. The behaviour is pre-existing; master is worse, since it leaks even without a focus change.
- **Root cause**: an `Unknown` record falls back to the bindings, and deferred arms skip repeats, so nothing asks to suppress. The pending gesture is not consulted.
- **Return to**: `doc-writer`, in the phase book sync or as a follow-up `Docs:` commit. Extend the first README gap bullet, the CHANGELOG KS-06 sentence and rule 5 with: "repeats of a `Tap`/`Hold`/`DoubleTap`/chord/sequence key pass after a focus change, and a still-pending Tap or Hold swallows its release (roadmap 2.3)". Add the scenario to 2.3's acceptance.
- **Re-check condition**: none of the three texts claims the release reaches Windows for a deferred-bound key; roadmap 2.3 lists the case.
- **Not blocking**: Minor (review r2), docs only, no regression.

## Positives

- `KeySpec` fixes RC4 at its root. One canonical trigger type is used by combos, steps and chords, and output keys stay `VIRTUAL_KEY`, so no pseudo-VK can reach `SendInput`.
- Matching became cheaper: no 256-slot scan, no 16-key cap, no per-event chord allocation.
- Evidence quality is high:
  - every red-first is an assertion red with the predicted text, confirmed independently by the tester, the triage and my master replay;
  - 22 tester mutations across two rounds were all caught;
  - the matching unit tests run against the production `KeyState`, not a test double;
  - the README example test reads the real README.
- The KS-06 extension was the right call. The critique showed that exact matching (I2) and live modifier triggers (I1) would otherwise ship new stuck-key paths. The three-state record is the smallest shape that keeps "passed key-down -> repeats and key-up pass" and "Immediate key held across a focus change -> repeats stay swallowed" at once.
- The breaking changes (exact modifiers, sided names, overlapping chord keys as a load error, `allow_extra_modifiers` rejected on sequence/chord) are each declared under `**Breaking:**` or Added, with a migration path.

## Notes for later lanes

1. **Owner FYI (trigger-key rule, cut D2/Q1)**: the rule is stricter than the brief's literal "belongs to its combo". With a payload key, only the payload key triggers: CapsLock then Shift does not fire `Shift+CapsLock`, and J then Ctrl does not fire `Ctrl+J`. This matches Windows hotkey semantics and is required to close KS-04's second symptom. Triage and tester both suggested owner confirmation; it does not block.
2. **Re-scope the planned KS-06/KS-14 lane**: KS-06 is closed here. What remains is KS-14 (`sync_modifiers` skips blocked keys), the pending-gesture stuck path (README gap 2), and possibly using a pending gesture as evidence of a swallowed key-down after a focus change (with 2.3).
3. **Roadmap 2.3** must pin P2's scenario and decide the lone-key-up question (review r2 open question 1) together with "foreground_changed no longer clears every blocked bit".
4. **Book sync**: apply cut.md "Docs impact" plus the impl.md KS-06 rows. Mark KS-04, KS-06 and KS-07 fixed and KS-32 partly; note exact auto-blockers (roadmap A / KS-05 partial) and KS-29's first half. Add manual checks M2, M3, M5, M7, M9 and M10 (impl.md) to a smoke checklist; M1, M4 and M8 are already smoke 3, 6 and 2.
5. **Orchestrator after merge**: update CLAUDE.md "Layout" and code-reviewer R8 (cut Q4), and fix the tester trailer text (P1).
6. If this lane merges outside a `ks-phase` run, schedule the doc-writer book sync explicitly. The published book still describes side folding and superset matching.
