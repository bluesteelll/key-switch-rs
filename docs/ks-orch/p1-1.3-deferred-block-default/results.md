# Results analysis: p1-1.3-deferred-block-default / roadmap item 1.3 (`block_default` for Tap, Hold and DoubleTap)

## VERDICT: ACCEPT

**Summary**: The lane does what item 1.3 asks with the smallest change that can work. `block_default` is `Option<bool>` in the schema, the loader resolves it once with `Binding::blocks_by_default`, and the engine has three one-line gates. The key-up and repeat rules are untouched, so a press keeps following the record of its key-down. Every spec and acceptance bullet has a scenario or loader test. 18 of those tests were shown red on master by assertion, and six mutations were caught. My own gate run is green. Two Minor doc remarks (m1, m2) and an open owner question (I1) remain. None blocks the merge, and all are listed below for the orchestrator.

Branch `feat/deferred-block-default` @ `1562acd`, cut from `master` @ `1afa720` (merge-base = master tip). Work tree clean before and after this analysis.

## Quality gate (own run)

Run in `D:/key-switch-rs/key-switch-rs` at `1562acd`. Clippy and the tests used a fresh `CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.3-deferred-block-default/target-analyst`, so no stale artefacts.

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0; fresh target dir, crate re-checked after a `touch src/lib.rs`) |
| `cargo test --locked` | OK. lib: `running 412 tests` / `411 passed; 0 failed; 1 ignored`; swch: 0 tests; `tests/ipc_roundtrip.rs`: 6 passed; doc-tests: 0. Total **417 passed, 1 ignored**. Log: `D:/tmp/ks-orch/p1-1.3-deferred-block-default/analyst-test.txt` |
| Test count vs master | `#[test]` attributes: master 385, HEAD 409 (+24); lib tests master 388, HEAD 412. Audit baseline 120. No test was deleted. |
| Ignored (`--list --ignored`, not run) | 1, pre-existing: `data::condition::tests::foreground_context_names_the_focused_notepad` (`manual:` reason present). The lane adds no `#[ignore]`. |
| `cargo doc --no-deps` | 4 warnings, all pre-existing and outside the lane's lines: `loader.rs:156`, `:176` (identical on master), `hook/keyboard_hook.rs:132` and `data/vk_name.rs:4` (files not touched) |
| `mdbook build book -d .../book-analyst` | builds (mermaid preprocessor version notice only) |
| cfgcheck (real loader) on the lane's 11 RON examples, `cfgcheck/ex/*.ron` | all load as documented. The book example gives `OK: 26 bindings`. `ron-bad-boc.ron` gives `Expected boolean`, and `ron-bad-bd-old.ron` (`block_default: Some(true)`) now loads, matching the new `ron-syntax.md` §3.6 |
| Trailers | 4 of 4 commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. No merge commits on the branch |

## Acceptance criteria

| # | Criterion (spec / `roadmap.md:251-259`) | Evidence | Status |
|---|---|---|---|
| S1 | `block_default` becomes `Option<bool>` in the schema | `src/config/schema.rs:49-50` (`#[serde(default)] pub block_default: Option<bool>`; `default_true` removed). Test `config::loader::tests::block_default_takes_some_and_none_like_the_other_optional_fields` (`loader.rs:746`) was red on master with `Expected boolean` (`logs/tester_master_red.log`) | PASS |
| S2 | It applies to Tap, Hold and DoubleTap (on master they suppress unconditionally) | Gates at `src/engine/mod.rs:434-436`, `:450-452`, `:466-468`. Scenarios `explicit_block_default_false_on_a_{tap,hold,double_tap}_passes_every_event_and_still_fires` (`scenarios.rs:2931`, `:2954`, `:2981`), red on master with "down F13 at +0 ms: expected the event to be passed". The tester's M1 mutation (DoubleTap gate forced on) was caught | PASS |
| S3 | Default: `true` for combos with a non-modifier key, `false` for combos made only of modifiers | `Binding::blocks_by_default` (`binding.rs:130-135`) with `KeyCombination::is_modifier_only` (`key_combination.rs:75-77`, shared with `is_trigger_key`, `matching.rs:58-63`). Loader L1 `block_default_defaults_to_false_only_for_deferred_bindings_made_only_of_modifiers` (`loader.rs:706`, red on master). Unit tests `blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers` (`binding.rs:286`) and `is_modifier_only_needs_every_key_to_be_a_modifier` (`key_combination.rs:153`). Mutations M2, M3 and M6 were caught | PASS |
| S4 | With `false`, every event of the key reaches the OS while the gesture is detected; the suppression invariant holds | Scenarios a, b and c, plus `passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too` (`:3098`, repeats), `one_long_press_of_a_passing_shift_does_not_fire_its_double_tap` (`:3119`, rollover, unsided Shift, KS-14 case 3 on the pass path), `modifier_only_hold_and_tap_pass_their_keys_by_default` (`:3148`) and m1-m3 (`:3262` foreground change, `:3278` `swch off/on`, `:3300` reload both directions). All of these run `check_press_symmetry` / `check_nothing_stuck`. `engine::properties::*` (3 tests) now generate scripts over pass and swallow rows: F15 Tap `false`, LShift/RShift by default, explicit `true` rows (`properties.rs:100-107`, `:133`; F15 is in `KEYS`, `:59`). `engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` (`mod.rs:632`). M5, which weakened the key-up rule, was caught by 10 tests, all 3 properties among them | PASS |
| S5 | Immediate semantics (default `true`) stay unchanged | Immediate gate `mod.rs:420-422` is untouched. L1 rows `LShift`/`Alt+Shift` Immediate resolve to `true`. The existing Immediate scenarios and `loader.rs:784` pass unchanged | PASS |
| A1 | A double LShift with DoubleTap(250) fires ToggleCapsLock, and every LShift event passes | `lshift_double_tap_fires_and_every_lshift_event_passes` (`scenarios.rs:3003`) and the flipped `lshift_double_tap_fires_only_for_the_left_shift` (`:901`, which also checks that RShift does not fire). Both were red on master | PASS |
| A2 | With LShift held, A passes together with Shift and nothing is swallowed | `lshift_held_with_its_double_tap_bound_types_a_capital_a` (`:3027`), with `expect_down_in_windows("LShift")` at A's key-down (`:3039`). The harness check has its own `should_panic` self-test (`:3413`). Red on master | PASS |
| A3 | A CapsLock Tap with the default `block_default` still swallows CapsLock | `capslock_tap_with_the_default_block_default_still_swallows_capslock` (`:3050`), down/up/repeat/up all suppressed. A guard, green on master by design; mutation M6 (every deferred binding defaults to passing) breaks it | PASS |
| A4 | An explicit `block_default: true` on a modifier-only DoubleTap swallows | `explicit_block_default_true_on_a_modifier_only_double_tap_swallows` (`:3076`) and loader L2 `an_explicit_block_default_wins_over_its_default` (`loader.rs:731`). Guards; mutation M4 (explicit value ignored) breaks both | PASS |
| D1 | Update README, `default_config.ron` (mark the LShift DoubleTap example as working), the book config pages and CHANGELOG | Template `default_config.ron:273-278` marks the example as working and carries the `<T>` caveat. L4 `the_template_lshift_double_tap_example_lets_the_left_shift_through` (`loader.rs:691`) was red on master. README `:340-351`, `:385-410`, `:490`, `:693`. CHANGELOG Unreleased: Changed/Breaking (`:132-151`), Fixed (`:362-368`), KS-07 edit (`:253-254`). Ten `book/src/config/*.md` pages. RON re-validated by me (above) | PASS (Minor remarks m1, m2 below) |
| M | Roadmap "LShift + A types 'A'" on a real keyboard; CapsLock really toggles | Manual: Checkpoint 1 smoke #6 (`roadmap.md:310`), with steps in impl.md M1 (including the `<T>` and `A B` steps). M2 and M3 cover pitfall rows 65 and 66. Every engine decision behind these is automated. The Windows reaction (KS-19 ordering of the injected CapsLock against the passed Shift) is not. | manual, listed |

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| none claimed | The roadmap row for 1.3 lists no issue (`roadmap.md:218`). The CHANGELOG KS-07 entry only loses its "until 1.3" caveat (KS-07 fixed in `82f542f`). `known-issues.md` row 54 (no KS-ID) is marked Fixed and backed by A1, A2 and the j scenario | yes: 18 tests red on `master + 6950476` tests (`logs/tester_master_red.log`: `391 passed; 18 failed`) and in the developer's replay (`logs/commit2_red.log:589`). The test commit `6950476` precedes the fix `eea58bf` | n/a |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/` is not in the diff. The engine adds one `bool` read per matching deferred binding (`mod.rs:434`, `:450`, `:466`): no allocation, lock, I/O or Win32 call. Symmetry follows the per-key record: a passed key-down is `mark_passed` (`mod.rs:544-551`), its repeats pass (`:537-543`) and its key-up passes (`:576-579`, `was_passed`). Nothing turns `Passed` into `Suppressed` mid-press (`mark_completed`, `forget_blocked`, `reset_from_os`; checked by critic and reviewer, and pinned by m1-m3 and properties) |
| Engine purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]` kept. The default policy lives in the loader (`loader.rs:385-387`), not the engine. Exact modifiers untouched (`combo_matches` unchanged). The repeat defaults are untouched: deferred arms still `continue` on repeat before the gate. One definition of "modifier-only" (`is_modifier_only`), used by the trigger-key rule and the default |
| Thread ownership, sentinel, atomics, SAFETY | OK | No new thread, timer, hook, `SendInput` path, static, atomic, lock or `unsafe`. Added `src` lines contain no `unsafe`, `unwrap()`, `SeqCst`/`Relaxed`, `println!` or non-ASCII byte (grep of `+` lines) |
| Config reload, IPC | OK | `RawBinding` keeps `#[serde(deny_unknown_fields)]` (`schema.rs:31`). The resolution runs inside `raw_binding_to_binding` for startup, reload and cfgcheck alike. Failed reloads are unchanged. A reload mid-gesture is pinned by m3. IPC untouched |
| No key logging | OK | No logging added |
| Owner decisions | OK | Exact modifiers, repeat defaults, RON, logging and licence are all untouched. The new modifier-only default is the spec's (roadmap item 1.3, written by the owner) and is declared **Breaking** in CHANGELOG |
| Typing on the owner's machine | OK | The default config's live bindings are both `Immediate` (`default_config.ron:197-205`), so with the shipped config nothing changes. The template's LShift example, which is commented out, improves on master: master swallowed every left Shift. Configs with modifier-only deferred bindings change as declared (Breaking entry, pitfall rows 65-68) |

## Review status

- `review_r1.md`: **APPROVED** (Critical 0, Important 0, Minor 2). Its gate (fmt, clippy) was run from a `git archive` of `1562acd`. No triage round was needed: the review was clean and the tester GREEN, per the `docs/WORKFLOW.md` flow.
- `test_r1.md`: **GREEN**. The full gate ran. The red-first replay on master was done in a scratch worktree (18 red by assertion). Six own mutations M1-M6 were all caught. All four acceptance lines map to passing tests.
- `critique.md`: CHANGES_REQUESTED (Critical 0, Important 1, Minor 2). The Important went to the developer, as WORKFLOW prescribes for non-Critical items.
  - **I1** (pass-through gestures are not cancelled by another key) was resolved by the critic's option (b). The caveat is in the template, CHANGELOG, README and book (§11.8, `kinds.md` §10, known-issues row 67). Two known-issue scenarios are pinned (`scenarios.rs:3341`, `:3376`), and M1 has the `<T>` / `A B` steps. **The owner question itself is still unanswered** (see Notes).
  - **M1** was resolved by documentation and pinned by scenario k's second config.
  - **M2** was resolved: the KS-13 text now covers deferred bindings in five pages.
- No CONFIRMED Critical or Important item is open.
- Open Minor items worth doing while the context is fresh: m1 and m2 (below, under Problems as P1 and P2).

## Docs and hygiene

- The config meaning changed, so README, the template, ten config pages and CHANGELOG (Unreleased, Keep a Changelog headings `### Changed` `:57` / `### Fixed` `:153`) change in the same lane. That matches CLAUDE.md.
- `audit.md`: nothing to mark, since no KS-ID is closed.
- `roadmap.md` 1.3 status, the `:199` expectation and the other non-config pages were deferred by the cut (cut.md:176) to the Phase-1 book sync (doc-writer, `docs/book-phase-1`). The `feat/repeat-flag` lane did the same: `overview.md:23` and `architecture.md:207` still describe pre-1.1 repeat behaviour. The deferral is therefore process, but it is now load-bearing. Until that sync runs, the published book contradicts itself in the places listed under Technical debt.
- No new dependencies. `Cargo.toml` and `Cargo.lock` are untouched, and `--locked` passes.
- The branch holds only this lane's four commits. It merges with `--no-ff` without conflict (merge-base = master tip).

## Technical debt

- **Stale book lines outside `config/`** (deferred to the Phase-1 book sync): `architecture.md:207-208` and `:238` ("always suppress"); `glossary.md:83-85` (`block_default`); `overview.md:23` and `:41` ("swallows every left-Shift press until roadmap item 1.3"); `build-and-run.md:418` ("Remove the binding until roadmap item 1.3"); `roadmap.md:151`, `:158`, `:199`, the 1.3 status block and `:594`. All but one are listed in cut.md:176. **`gaps.md:182`** ("the README says it swallows the left Shift") is **not** on the cut's list and would be lost.
- **I1 interrupt rule** (a key pressed meanwhile does not cancel a pass-through gesture). It is recorded in `known-issues.md` row 67 as "a follow-up of roadmap item 1.3, with no roadmap item yet" and pinned at `scenarios.rs:3341` and `:3376`. It has no roadmap item, so it needs an owner decision (2.2 acceptance or a new item).
- **Lone Alt/Win tap with a passing gesture** (`known-issues.md` row 66, CHANGELOG `:149`, README `:408`) is attributed to item 1.2. As written, 1.2 does not deliver this fix (P1).
- **Hold/DoubleTap output combined with the passed modifier** (KS-19). Row 30 records it generally and row 68 records it for `ToggleCapsLock`. It is not in the Breaking entry (P2).
- **F2**: `block_default` is accepted and ignored on `sequence`/`chord` (`errors.md:204`; no KS-ID). It is recorded only in cut.md and impl.md, not in `gaps.md` or the roadmap.
- **F6**: stale R8 text in `.claude/agents/code-reviewer.md:126` (`active_keys`). It is recorded only in cut.md and impl.md.
- 4 pre-existing `cargo doc` warnings (`loader.rs:156`, `:176`, `keyboard_hook.rs:132`, `vk_name.rs:4`). They are not the lane's, and `cargo doc` is not part of the gate.
- No new `TODO`/`FIXME` in the diff.

## Problems

None blocks ACCEPT. Two Minor doc findings from `review_r1.md` remain open. Their fix is cheap, and they are listed so they do not get lost.

### P1. The lone Alt/Win regression is attributed to item 1.2, which as written does not fix it (review m1, CONFIRMED)
- **What**: The docs say "until roadmap item 1.2" in `CHANGELOG.md:149`, `README.md:408` and `book/src/config/known-issues.md:134` (row 66, "Open until roadmap item 1.2 for the menu bar and Start").
- **Impact**: With `( keys: "LWin", action: ToggleCapsLock, trigger: Tap(200) )` and the new default, LWin down and up both pass. Item 1.2 injects the mask only "when `decide()` suppresses an event while Win or Alt is held" (`roadmap.md:238`; its acceptance says the same). Nothing is suppressed in this case, so Start keeps opening after 1.2 ships. The row can then be closed by mistake, or the regression ends up with no owner.
- **Root cause**: The cut's Risks section (cut.md:321) assumed that 1.2's mask covers a passed lone modifier. 1.2's spec only covers a suppressed event after the modifier reached the OS.
- **Return to**: orchestrator first. It can widen 1.2's spec when cutting it: a mask on the key-up of a lone Win/Alt whose key-down passed, when a gesture fires on it. Then the text is right as it stands. Otherwise it goes to `doc-writer`: relabel row 66 "no roadmap owner yet" (as row 67 does), align README and CHANGELOG, and align the CHANGELOG sub-bullet's "A `Tap` on `Alt` or `Win`" with README's "`Tap`, `Hold` or `DoubleTap`".
- **Re-check condition**: either 1.2's roadmap text names the passed-lone-modifier case, or rows/README/CHANGELOG no longer name 1.2 as its fix.

### P2. The Breaking entry does not say that a modifier-only Hold now sends its output with that modifier down (review m2, CONFIRMED for Hold)
- **What**: `CHANGELOG.md:132-151`. Known-issues row 68 covers only the `ToggleCapsLock` of an `LShift` `DoubleTap`.
- **Impact**: A config that does not change changes behaviour with the version:
  - `( keys: "LCtrl", action: PressKey("Escape"), trigger: Hold(300) )` used to send a plain Escape. Now LCtrl reaches Windows, and the `hold-timer` sends Ctrl+Esc, which opens Start.
  - `( keys: "LWin", action: PressKey("L"), trigger: Hold(..) )` becomes Win+L and locks the session.
  The general mechanism is in row 30 (KS-19), but nothing connects it to this Breaking change.
- **Root cause**: The default `false` for modifier-only combos (the spec) leaves the modifier down in Windows while the action runs. Releasing held modifiers around injected output is item 3.1, so the docs must carry this until then.
- **Return to**: `doc-writer` (one Breaking sub-bullet, plus widening row 68 or pointing it to row 30). This can be done in the Phase-1 book sync or in a doc commit before release, since the entry is under Unreleased.
- **Re-check condition**: the Breaking entry names this consequence and the `block_default: true` remedy.

## Positives

- The default is resolved once, in the loader. The engine holds no policy, builder order cannot overwrite an explicit value, and cfgcheck and the startup list see the resolved value.
- The invariant comes from the per-key record, not from the binding. Reload in both directions, a foreground change and `swch off/on` each have a pass-path scenario. The property generator now covers both paths.
- The D9 scenarios keep the KS-06/KS-14 swallow-path regressions explicit (`block_default: true`) and add pass-path twins, so no coverage was lost by flipping expectations.
- `expect_down_in_windows` turns acceptance 2 ("together with Shift") into a literal check, and the check has its own self-test.
- The red-first replay matches the declared set exactly (18 tests, all by assertion). The commit body lists each first failing line.
- Known limitations are pinned as scenarios ("Known issue, pinned until an interrupt rule is decided"), so the follow-up starts red-first.

## Notes for later lanes

1. **Owner question (orchestrator, before or right after the merge)**: should a pass-through gesture be cancelled when another key goes down (AutoHotkey `A_PriorKey` style)? If yes, it becomes either an acceptance line of 2.2 or a new item. The two pinned scenarios (`scenarios.rs:3341`, `:3376`) then flip. Today's behaviour matches the roadmap's 1.3 acceptance and is strictly better than master for the template example, so merging does not pre-empt either answer.
2. **Item 1.2 cut**: decide P1 there. Critic open question 1 (merge 1.2 first?) rests on a premise P1 refutes: 1.2 as written does not remove the lone-tap behaviour.
3. **Phase-1 book sync**: include `gaps.md:182` in addition to cut.md:176's list. Also update the repeat-flag leftovers (`overview.md:23`, `architecture.md:207`).
4. **Item 1.6**: the OR suppression now spans deferred bindings too (scenario l, `scenarios.rs:3238`). 1.6's acceptance "a winner with `block_default: false` lets the key through" should include a deferred winner.
5. **Item 3.1**: the passed-modifier output of Hold/DoubleTap (P2, row 68) is a concrete regression case for KS-19.
6. **Checkpoint 1**: smoke #6 and impl.md M1-M3 are human checks. M1 records `HKCU\Keyboard Layout\Attributes` (read only) to explain any wrong CapsLock toggle under "Press the SHIFT key".
7. **Follow-ups F2 and F6** live only in lane reports. Record F2 in `gaps.md` or the roadmap, and fix F6 in a chore commit.
